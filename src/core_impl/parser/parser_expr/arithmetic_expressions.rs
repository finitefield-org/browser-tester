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
            let left = parse_expr(src[..i].trim())?;
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

pub(crate) fn parse_unary_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
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
        let inner = parse_unary_expr(rest)?;
        return Ok(Expr::YieldStar(Box::new(inner)));
    }
    if let Some(rest) = strip_keyword_operator(src, "yield") {
        if rest.is_empty() {
            return Err(Error::ScriptParse(
                "yield operator requires an operand".into(),
            ));
        }
        let inner = parse_unary_expr(rest)?;
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
