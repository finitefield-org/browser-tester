use super::*;

pub(crate) fn parse_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    if src.is_empty() {
        return Err(Error::ScriptParse("empty expression".into()));
    }

    if let Some(expr) = parse_regex_method_expr(src)? {
        return Ok(expr);
    }

    if let Some((pattern, flags)) = parse_regex_literal_expr(src)? {
        return Ok(Expr::RegexLiteral { pattern, flags });
    }

    if let Some(expr) = parse_new_regexp_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_regexp_static_expr(src)? {
        return Ok(expr);
    }

    if let Some(handler_expr) = parse_function_expr(src)? {
        return Ok(handler_expr);
    }

    parse_comma_expr(src)
}

pub(crate) fn strip_js_comments(src: &str) -> String {
    enum State {
        Normal,
        Single,
        Double,
        Template,
        Regex { in_class: bool },
    }

    let bytes = src.as_bytes();
    let mut state = State::Normal;
    let mut i = 0usize;
    let mut previous_significant: Option<u8> = None;
    let mut previous_identifier_allows_regex = false;
    let mut out: Vec<u8> = Vec::with_capacity(src.len());

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            State::Normal => {
                if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    i += 2;
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                    if i < bytes.len() {
                        out.push(b'\n');
                        i += 1;
                    }
                    continue;
                }
                if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    i += 2;
                    while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                        i += 1;
                    }
                    if i + 1 < bytes.len() {
                        i += 2;
                    } else {
                        i = bytes.len();
                    }
                    continue;
                }
                if b == b'/' {
                    if can_start_regex_literal(previous_significant)
                        || previous_identifier_allows_regex
                    {
                        state = State::Regex { in_class: false };
                        out.push(b);
                        previous_significant = Some(b'/');
                        previous_identifier_allows_regex = false;
                        i += 1;
                        continue;
                    }
                }

                if b == b'_' || b == b'$' || b.is_ascii_alphabetic() {
                    let start = i;
                    i += 1;
                    while i < bytes.len() && is_ident_char(bytes[i]) {
                        i += 1;
                    }
                    out.extend_from_slice(&bytes[start..i]);
                    let prev = previous_significant;
                    previous_significant = bytes.get(i - 1).copied();
                    previous_identifier_allows_regex =
                        identifier_allows_regex_start(&bytes[start..i], prev);
                    continue;
                }

                match b {
                    b'\'' => {
                        state = State::Single;
                        out.push(b);
                        previous_significant = Some(b'\'');
                        previous_identifier_allows_regex = false;
                        i += 1;
                    }
                    b'"' => {
                        state = State::Double;
                        out.push(b);
                        previous_significant = Some(b'"');
                        previous_identifier_allows_regex = false;
                        i += 1;
                    }
                    b'`' => {
                        state = State::Template;
                        out.push(b);
                        previous_significant = Some(b'`');
                        previous_identifier_allows_regex = false;
                        i += 1;
                    }
                    _ => {
                        out.push(b);
                        if !b.is_ascii_whitespace() {
                            previous_significant = Some(b);
                            previous_identifier_allows_regex = false;
                        }
                        i += 1;
                    }
                }
            }
            State::Single => {
                if b == b'\\' {
                    out.push(b);
                    if i + 1 < bytes.len() {
                        out.push(bytes[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                    continue;
                }
                out.push(b);
                if b == b'\'' {
                    state = State::Normal;
                    previous_significant = Some(b'\'');
                    previous_identifier_allows_regex = false;
                }
                i += 1;
            }
            State::Double => {
                if b == b'\\' {
                    out.push(b);
                    if i + 1 < bytes.len() {
                        out.push(bytes[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                    continue;
                }
                out.push(b);
                if b == b'"' {
                    state = State::Normal;
                    previous_significant = Some(b'"');
                    previous_identifier_allows_regex = false;
                }
                i += 1;
            }
            State::Template => {
                if b == b'\\' {
                    out.push(b);
                    if i + 1 < bytes.len() {
                        out.push(bytes[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                    continue;
                }
                out.push(b);
                if b == b'`' {
                    state = State::Normal;
                    previous_significant = Some(b'`');
                    previous_identifier_allows_regex = false;
                }
                i += 1;
            }
            State::Regex { in_class } => {
                if b == b'\\' {
                    out.push(b);
                    if i + 1 < bytes.len() {
                        out.push(bytes[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                    continue;
                }
                if b == b'[' {
                    out.push(b);
                    state = State::Regex { in_class: true };
                    i += 1;
                    continue;
                }
                if b == b']' && in_class {
                    out.push(b);
                    state = State::Regex { in_class: false };
                    i += 1;
                    continue;
                }
                out.push(b);
                if b == b'/' && !in_class {
                    state = State::Normal;
                    previous_significant = Some(b'/');
                    previous_identifier_allows_regex = false;
                }
                i += 1;
            }
        }
    }

    String::from_utf8(out).unwrap_or_else(|_| src.to_string())
}

pub(crate) fn parse_comma_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let parts = split_top_level_by_char(src, b',');
    if parts.len() == 1 {
        return parse_ternary_expr(src);
    }

    let mut parsed = Vec::with_capacity(parts.len());
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            return Err(Error::ScriptParse(format!(
                "invalid comma expression: {src}"
            )));
        }
        parsed.push(parse_ternary_expr(part)?);
    }
    Ok(Expr::Comma(parsed))
}

pub(crate) fn parse_ternary_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());

    if let Some(q_pos) = find_top_level_ternary_question(src) {
        let cond_src = src[..q_pos].trim();
        let colon_pos = find_matching_ternary_colon(src, q_pos + 1).ok_or_else(|| {
            Error::ScriptParse(format!("invalid ternary expression (missing ':'): {src}"))
        })?;
        let true_src = src[q_pos + 1..colon_pos].trim();
        let false_src = src[colon_pos + 1..].trim();

        return Ok(Expr::Ternary {
            cond: Box::new(parse_ternary_expr(cond_src)?),
            on_true: Box::new(parse_ternary_expr(true_src)?),
            on_false: Box::new(parse_ternary_expr(false_src)?),
        });
    }

    parse_logical_or_expr(src)
}

pub(crate) fn parse_logical_or_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    let (parts, ops) = split_top_level_by_ops(src, &["||"]);
    if ops.is_empty() {
        return parse_nullish_expr(src);
    }
    fold_binary(parts, ops, parse_nullish_expr, |op| match op {
        "||" => BinaryOp::Or,
        _ => unreachable!(),
    })
}

pub(crate) fn parse_nullish_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    let (parts, ops) = split_top_level_by_ops(src, &["??"]);
    if ops.is_empty() {
        return parse_logical_and_expr(src);
    }
    fold_binary(parts, ops, parse_logical_and_expr, |op| match op {
        "??" => BinaryOp::Nullish,
        _ => unreachable!(),
    })
}

pub(crate) fn parse_logical_and_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    let (parts, ops) = split_top_level_by_ops(src, &["&&"]);
    if ops.is_empty() {
        return parse_bitwise_or_expr(src);
    }
    fold_binary(parts, ops, parse_bitwise_or_expr, |op| match op {
        "&&" => BinaryOp::And,
        _ => unreachable!(),
    })
}

pub(crate) fn parse_bitwise_or_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    let (parts, ops) = split_top_level_by_ops(src, &["|"]);
    if ops.is_empty() {
        return parse_bitwise_xor_expr(src);
    }
    fold_binary(parts, ops, parse_bitwise_xor_expr, |op| match op {
        "|" => BinaryOp::BitOr,
        _ => unreachable!(),
    })
}

pub(crate) fn parse_bitwise_xor_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    let (parts, ops) = split_top_level_by_ops(src, &["^"]);
    if ops.is_empty() {
        return parse_bitwise_and_expr(src);
    }
    fold_binary(parts, ops, parse_bitwise_and_expr, |op| match op {
        "^" => BinaryOp::BitXor,
        _ => unreachable!(),
    })
}

pub(crate) fn parse_bitwise_and_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    let (parts, ops) = split_top_level_by_ops(src, &["&"]);
    if ops.is_empty() {
        return parse_equality_expr(src);
    }
    fold_binary(parts, ops, parse_equality_expr, |op| match op {
        "&" => BinaryOp::BitAnd,
        _ => unreachable!(),
    })
}

pub(crate) fn parse_equality_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    let (parts, ops) = split_top_level_by_ops(src, &["!==", "===", "!=", "=="]);
    if ops.is_empty() {
        return parse_relational_expr(src);
    }
    fold_binary(parts, ops, parse_relational_expr, |op| match op {
        "===" => BinaryOp::StrictEq,
        "!==" => BinaryOp::StrictNe,
        "==" => BinaryOp::Eq,
        "!=" => BinaryOp::Ne,
        _ => unreachable!(),
    })
}

pub(crate) fn parse_relational_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    if let Some(expr) = parse_private_in_expr(src)? {
        return Ok(expr);
    }
    let (parts, ops) = split_top_level_by_ops(src, &["<=", ">=", "<", ">", "instanceof", "in"]);
    if ops.is_empty() {
        return parse_shift_expr(src);
    }
    fold_binary(parts, ops, parse_shift_expr, |op| match op {
        "<" => BinaryOp::Lt,
        ">" => BinaryOp::Gt,
        "<=" => BinaryOp::Le,
        ">=" => BinaryOp::Ge,
        "instanceof" => BinaryOp::InstanceOf,
        "in" => BinaryOp::In,
        _ => unreachable!(),
    })
}

fn parse_private_in_expr(src: &str) -> Result<Option<Expr>> {
    let (parts, ops) = split_top_level_by_ops(src, &["in"]);
    if ops.len() != 1 || parts.len() != 2 || ops[0] != "in" {
        return Ok(None);
    }
    let left = parts[0].trim();
    let Some(name) = left.strip_prefix('#') else {
        return Ok(None);
    };
    if !is_ident(name) {
        return Err(Error::ScriptParse(format!(
            "invalid private identifier in in-expression: {left}"
        )));
    }
    let right = parts[1].trim();
    if right.is_empty() {
        return Err(Error::ScriptParse(
            "private in-expression requires a right-hand operand".into(),
        ));
    }
    Ok(Some(Expr::PrivateIn {
        member: name.to_string(),
        target: Box::new(parse_expr(right)?),
    }))
}

pub(crate) fn parse_shift_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    let (parts, ops) = split_top_level_by_ops(src, &[">>>", "<<", ">>"]);
    if ops.is_empty() {
        return parse_add_expr(src);
    }
    fold_binary(parts, ops, parse_add_expr, |op| match op {
        ">>>" => BinaryOp::UnsignedShiftRight,
        "<<" => BinaryOp::ShiftLeft,
        ">>" => BinaryOp::ShiftRight,
        _ => unreachable!(),
    })
}

pub(crate) fn parse_add_expr(src: &str) -> Result<Expr> {
    let trimmed = src.trim();
    let src = strip_outer_parens(trimmed);
    if src.len() != trimmed.len() {
        return parse_expr(src);
    }
    let (parts, ops) = split_top_level_add_sub(src);
    if ops.is_empty() {
        return parse_mul_expr(src);
    }

    if parts.iter().any(|part| part.trim().is_empty()) {
        return Err(Error::ScriptParse(format!(
            "invalid additive expression: {src}"
        )));
    }

    let mut expr = parse_mul_expr(parts[0].trim())?;
    for (idx, op) in ops.iter().enumerate() {
        let rhs = parse_expr(parts[idx + 1].trim())?;
        if *op == '+' {
            expr = append_concat_expr(expr, rhs);
        } else {
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Sub,
                right: Box::new(rhs),
            };
        }
    }

    Ok(expr)
}

pub(crate) fn append_concat_expr(lhs: Expr, rhs: Expr) -> Expr {
    match lhs {
        Expr::Add(mut parts) => {
            parts.push(rhs);
            Expr::Add(parts)
        }
        other => Expr::Add(vec![other, rhs]),
    }
}

pub(crate) fn split_top_level_add_sub(src: &str) -> (Vec<&str>, Vec<char>) {
    let bytes = src.as_bytes();
    let mut parts = Vec::new();
    let mut ops = Vec::new();
    let mut start = 0usize;
    let mut scanner = JsLexScanner::new();

    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if scanner.is_top_level()
            && matches!(b, b'+' | b'-')
            && is_add_sub_binary_operator(bytes, i)
        {
            if let Some(part) = src.get(start..i) {
                parts.push(part);
            }
            ops.push(b as char);
            start = i + 1;
        }
        i = scanner.advance(bytes, i);
    }

    if let Some(part) = src.get(start..) {
        parts.push(part);
    }

    (parts, ops)
}

pub(crate) fn is_add_sub_binary_operator(bytes: &[u8], idx: usize) -> bool {
    if idx >= bytes.len() {
        return false;
    }
    let mut left = idx;
    while left > 0 && bytes[left - 1].is_ascii_whitespace() {
        left -= 1;
    }
    if left == 0 {
        return false;
    }
    let prev = bytes[left - 1];
    if matches!(prev, b'e' | b'E') && is_decimal_exponent_sign(bytes, left - 1) {
        return false;
    }
    !matches!(
        prev,
        b'(' | b'['
            | b'{'
            | b','
            | b'?'
            | b':'
            | b'='
            | b'!'
            | b'<'
            | b'>'
            | b'&'
            | b'|'
            | b'+'
            | b'-'
            | b'*'
            | b'/'
            | b'%'
    )
}

pub(crate) fn is_decimal_exponent_sign(bytes: &[u8], exponent_index: usize) -> bool {
    if exponent_index >= bytes.len() {
        return false;
    }
    if !matches!(bytes[exponent_index], b'e' | b'E') {
        return false;
    }

    let mut start = exponent_index;
    while start > 0 && (bytes[start - 1].is_ascii_digit() || bytes[start - 1] == b'.') {
        start -= 1;
    }

    if start == exponent_index {
        return false;
    }

    if start > 0 && is_ident_char(bytes[start - 1]) {
        return false;
    }

    let mut has_digit = false;
    let mut dot_count = 0usize;
    for &b in &bytes[start..exponent_index] {
        if b.is_ascii_digit() {
            has_digit = true;
            continue;
        }
        if b == b'.' {
            dot_count += 1;
            if dot_count > 1 {
                return false;
            }
            continue;
        }
        return false;
    }

    has_digit
}
