use super::super::parser_stmt::{
    parse_dom_assignment, parse_object_assign, parse_private_assign, parse_var_assign,
};
use super::*;
pub(crate) fn parse_mul_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    if let Some(expr) = parse_regex_method_expr(src)? {
        return Ok(expr);
    }
    if let Some((pattern, flags)) = parse_regex_literal_expr(src)? {
        return Ok(Expr::RegexLiteral { pattern, flags });
    }
    if src.strip_prefix("yield*").is_some() {
        return parse_pow_expr(src);
    }
    let bytes = src.as_bytes();
    let mut parts = Vec::new();
    let mut ops: Vec<u8> = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        let b = bytes[i];
        if scanner.is_top_level() {
            if b == b'/' && !scanner.slash_starts_comment_or_regex(bytes, i) {
                if let Some(part) = src.get(start..i) {
                    parts.push(part);
                    ops.push(b'/');
                    start = i + 1;
                }
            } else if b == b'%' {
                if let Some(part) = src.get(start..i) {
                    parts.push(part);
                    ops.push(b'%');
                    start = i + 1;
                }
            } else if b == b'*'
                && !(i + 1 < bytes.len() && bytes[i + 1] == b'*')
                && !(i > 0 && bytes[i - 1] == b'*')
            {
                if let Some(part) = src.get(start..i) {
                    parts.push(part);
                    ops.push(b'*');
                    start = i + 1;
                }
            }
        }
        i = scanner.advance(bytes, i);
    }

    if let Some(last) = src.get(start..) {
        parts.push(last);
    }

    if ops.is_empty() {
        return parse_pow_expr(src);
    }

    let mut expr = parse_pow_expr(parts[0].trim())?;
    for (idx, op) in ops.iter().enumerate() {
        let rhs = parse_pow_expr(parts[idx + 1].trim())?;
        let op = match op {
            b'/' => BinaryOp::Div,
            b'%' => BinaryOp::Mod,
            _ => BinaryOp::Mul,
        };
        expr = Expr::Binary {
            left: Box::new(expr),
            op,
            right: Box::new(rhs),
        };
    }
    Ok(expr)
}

pub(crate) fn parse_pow_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        let b = bytes[i];
        if scanner.is_top_level() && b == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            let left_src = src[..i].trim();
            if has_unparenthesized_unary_pow_base(left_src) {
                return Err(Error::ScriptParse(
                    "unparenthesized unary expression cannot appear on the left-hand side of '**'"
                        .into(),
                ));
            }

            let left = parse_expr(left_src)?;
            let right = parse_pow_expr(src[i + 2..].trim())?;
            return Ok(Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Pow,
                right: Box::new(right),
            });
        }
        i = scanner.advance(bytes, i);
    }

    parse_unary_expr(src)
}

pub(crate) fn has_unparenthesized_unary_pow_base(src: &str) -> bool {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return false;
    }
    if strip_outer_parens(trimmed).len() != trimmed.len() {
        return false;
    }

    if trimmed.starts_with("++")
        || trimmed.starts_with("--")
        || trimmed.starts_with('+')
        || trimmed.starts_with('-')
        || trimmed.starts_with('!')
        || trimmed.starts_with('~')
    {
        return true;
    }

    strip_keyword_operator(trimmed, "await").is_some()
        || strip_keyword_operator(trimmed, "typeof").is_some()
        || strip_keyword_operator(trimmed, "void").is_some()
        || strip_keyword_operator(trimmed, "delete").is_some()
        || strip_keyword_operator(trimmed, "yield*").is_some()
        || strip_keyword_operator(trimmed, "yield").is_some()
}

pub(crate) fn parse_unary_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    if let Some(update) = parse_update_expr(src)? {
        return Ok(update);
    }
    if let Some(rest) = strip_keyword_operator(src, "await") {
        if rest.is_empty() {
            return Err(Error::ScriptParse(
                "await operator requires an operand".into(),
            ));
        }
        let inner = parse_unary_expr(rest)?;
        return Ok(Expr::Await(Box::new(inner)));
    }
    if let Some(rest) = src.strip_prefix("yield*") {
        let rest = rest.trim_start();
        if rest.is_empty() {
            return Err(Error::ScriptParse(
                "yield* operator requires an operand".into(),
            ));
        }
        let inner = parse_expr(rest)?;
        return Ok(Expr::YieldStar(Box::new(inner)));
    }
    if let Some(rest) = strip_keyword_operator(src, "yield") {
        if rest.is_empty() {
            return Err(Error::ScriptParse(
                "yield operator requires an operand".into(),
            ));
        }
        let inner = parse_expr(rest)?;
        return Ok(Expr::Yield(Box::new(inner)));
    }
    if let Some(rest) = strip_keyword_operator(src, "typeof") {
        let inner = parse_unary_expr(rest)?;
        return Ok(Expr::TypeOf(Box::new(inner)));
    }
    if let Some(rest) = strip_keyword_operator(src, "void") {
        let inner = parse_unary_expr(rest)?;
        return Ok(Expr::Void(Box::new(inner)));
    }
    if let Some(rest) = strip_keyword_operator(src, "delete") {
        let inner = parse_unary_expr(rest)?;
        if matches!(inner, Expr::PrivateMemberGet { .. }) {
            return Err(Error::ScriptParse(
                "private elements cannot be deleted".into(),
            ));
        }
        return Ok(Expr::Delete(Box::new(inner)));
    }
    if let Some(rest) = src.strip_prefix('+') {
        let inner = parse_unary_expr(rest.trim())?;
        return Ok(Expr::Pos(Box::new(inner)));
    }
    if let Some(rest) = src.strip_prefix('-') {
        let inner = parse_unary_expr(rest.trim())?;
        return Ok(Expr::Neg(Box::new(inner)));
    }
    if let Some(rest) = src.strip_prefix('!') {
        let inner = parse_unary_expr(rest.trim())?;
        return Ok(Expr::Not(Box::new(inner)));
    }
    if let Some(rest) = src.strip_prefix('~') {
        let inner = parse_unary_expr(rest.trim())?;
        return Ok(Expr::BitNot(Box::new(inner)));
    }
    parse_primary(src)
}

pub(crate) fn parse_update_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    if src.is_empty() {
        return Ok(None);
    }

    if let Some(rest) = src.strip_prefix("++") {
        return parse_prefix_update_expr(rest.trim(), 1);
    }
    if let Some(rest) = src.strip_prefix("--") {
        return parse_prefix_update_expr(rest.trim(), -1);
    }
    if let Some(rest) = src.strip_suffix("++") {
        return parse_postfix_update_expr(rest.trim(), 1);
    }
    if let Some(rest) = src.strip_suffix("--") {
        return parse_postfix_update_expr(rest.trim(), -1);
    }

    Ok(None)
}

pub(crate) fn parse_prefix_update_expr(target: &str, delta: i8) -> Result<Option<Expr>> {
    if target.is_empty() {
        return Err(Error::ScriptParse(
            "update operator requires an operand".into(),
        ));
    }
    if !is_valid_update_target(target) {
        return Err(Error::ScriptParse(
            "invalid left-hand side expression in prefix operation".into(),
        ));
    }

    let mut temp_index = 0usize;
    let temp_name = loop {
        let candidate = format!("__bt_update_prev_{temp_index}");
        if !target.contains(&candidate) {
            break candidate;
        }
        temp_index += 1;
    };
    let next = build_update_numeric_expr(&temp_name, delta);
    let lowered = format!(
        "(() => {{ const {temp_name} = {target}; {target} = {next}; return {next}; }})()"
    );
    Ok(Some(parse_expr(&lowered)?))
}

pub(crate) fn parse_postfix_update_expr(target: &str, delta: i8) -> Result<Option<Expr>> {
    if target.is_empty() {
        return Err(Error::ScriptParse(
            "update operator requires an operand".into(),
        ));
    }
    if !is_valid_update_target(target) {
        return Err(Error::ScriptParse(
            "invalid left-hand side expression in postfix operation".into(),
        ));
    }

    let mut temp_index = 0usize;
    let temp_name = loop {
        let candidate = format!("__bt_update_prev_{temp_index}");
        if candidate != target {
            break candidate;
        }
        temp_index += 1;
    };
    let next = build_update_numeric_expr(&temp_name, delta);
    let lowered = format!(
        "(() => {{ const {temp_name} = {target}; {target} = {next}; return {temp_name}; }})()"
    );
    Ok(Some(parse_expr(&lowered)?))
}

pub(crate) fn build_update_numeric_expr(source: &str, delta: i8) -> String {
    if delta >= 0 {
        format!("(typeof {source} === 'bigint' ? ({source} + 1n) : (+{source} + 1))")
    } else {
        format!("(typeof {source} === 'bigint' ? ({source} - 1n) : (+{source} - 1))")
    }
}

pub(crate) fn is_valid_update_target(target: &str) -> bool {
    let assignment_src = format!("{target} = 0");
    let supports_assignment = |result: Result<Option<Stmt>>| match result {
        Ok(Some(_)) => true,
        Ok(None) | Err(_) => false,
    };

    supports_assignment(parse_var_assign(&assignment_src))
        || supports_assignment(parse_object_assign(&assignment_src))
        || supports_assignment(parse_private_assign(&assignment_src))
        || supports_assignment(parse_dom_assignment(&assignment_src))
}

pub(crate) fn fold_binary<F, G>(
    parts: Vec<&str>,
    ops: Vec<&str>,
    parse_leaf: F,
    map_op: G,
) -> Result<Expr>
where
    F: Fn(&str) -> Result<Expr>,
    G: Fn(&str) -> BinaryOp,
{
    if parts.is_empty() {
        return Err(Error::ScriptParse("invalid binary expression".into()));
    }
    let mut expr = parse_leaf(parts[0].trim())?;
    for (idx, op) in ops.iter().enumerate() {
        let rhs = parse_leaf(parts[idx + 1].trim())?;
        expr = Expr::Binary {
            left: Box::new(expr),
            op: map_op(op),
            right: Box::new(rhs),
        };
    }
    Ok(expr)
}
pub(crate) fn strip_keyword_operator<'a>(src: &'a str, keyword: &str) -> Option<&'a str> {
    if !src.starts_with(keyword) {
        return None;
    }

    let after = &src[keyword.len()..];
    if after.is_empty() || !is_ident_char(after.as_bytes()[0]) {
        return Some(after.trim_start());
    }

    None
}
