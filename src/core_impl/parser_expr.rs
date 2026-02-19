pub(super) fn parse_expr(src: &str) -> Result<Expr> {
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

fn strip_js_comments(src: &str) -> String {
    enum State {
        Normal,
        Single,
        Double,
        Template,
    }

    let bytes = src.as_bytes();
    let mut state = State::Normal;
    let mut i = 0usize;
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

                match b {
                    b'\'' => {
                        state = State::Single;
                        out.push(b);
                        i += 1;
                    }
                    b'"' => {
                        state = State::Double;
                        out.push(b);
                        i += 1;
                    }
                    b'`' => {
                        state = State::Template;
                        out.push(b);
                        i += 1;
                    }
                    _ => {
                        out.push(b);
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
                }
                i += 1;
            }
        }
    }

    String::from_utf8(out).unwrap_or_else(|_| src.to_string())
}

fn parse_comma_expr(src: &str) -> Result<Expr> {
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

fn parse_ternary_expr(src: &str) -> Result<Expr> {
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

fn parse_logical_or_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["||"]);
    if ops.is_empty() {
        return parse_nullish_expr(src);
    }
    fold_binary(parts, ops, parse_nullish_expr, |op| match op {
        "||" => BinaryOp::Or,
        _ => unreachable!(),
    })
}

fn parse_nullish_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["??"]);
    if ops.is_empty() {
        return parse_logical_and_expr(src);
    }
    fold_binary(parts, ops, parse_logical_and_expr, |op| match op {
        "??" => BinaryOp::Nullish,
        _ => unreachable!(),
    })
}

fn parse_logical_and_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["&&"]);
    if ops.is_empty() {
        return parse_bitwise_or_expr(src);
    }
    fold_binary(parts, ops, parse_bitwise_or_expr, |op| match op {
        "&&" => BinaryOp::And,
        _ => unreachable!(),
    })
}

fn parse_bitwise_or_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["|"]);
    if ops.is_empty() {
        return parse_bitwise_xor_expr(src);
    }
    fold_binary(parts, ops, parse_bitwise_xor_expr, |op| match op {
        "|" => BinaryOp::BitOr,
        _ => unreachable!(),
    })
}

fn parse_bitwise_xor_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["^"]);
    if ops.is_empty() {
        return parse_bitwise_and_expr(src);
    }
    fold_binary(parts, ops, parse_bitwise_and_expr, |op| match op {
        "^" => BinaryOp::BitXor,
        _ => unreachable!(),
    })
}

fn parse_bitwise_and_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["&"]);
    if ops.is_empty() {
        return parse_equality_expr(src);
    }
    fold_binary(parts, ops, parse_equality_expr, |op| match op {
        "&" => BinaryOp::BitAnd,
        _ => unreachable!(),
    })
}

fn parse_equality_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
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

fn parse_relational_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
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

fn parse_shift_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
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

fn parse_add_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
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

fn append_concat_expr(lhs: Expr, rhs: Expr) -> Expr {
    match lhs {
        Expr::Add(mut parts) => {
            parts.push(rhs);
            Expr::Add(parts)
        }
        other => Expr::Add(vec![other, rhs]),
    }
}

fn split_top_level_add_sub(src: &str) -> (Vec<&str>, Vec<char>) {
    let bytes = src.as_bytes();
    let mut parts = Vec::new();
    let mut ops = Vec::new();
    let mut start = 0usize;
    let mut scanner = JsLexScanner::new();

    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if scanner.is_top_level() && matches!(b, b'+' | b'-') && is_add_sub_binary_operator(bytes, i) {
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

fn is_add_sub_binary_operator(bytes: &[u8], idx: usize) -> bool {
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

fn parse_mul_expr(src: &str) -> Result<Expr> {
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

fn parse_pow_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
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

fn parse_unary_expr(src: &str) -> Result<Expr> {
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

fn fold_binary<F, G>(parts: Vec<&str>, ops: Vec<&str>, parse_leaf: F, map_op: G) -> Result<Expr>
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

fn parse_primary(src: &str) -> Result<Expr> {
    let src = src.trim();

    if src == "true" {
        return Ok(Expr::Bool(true));
    }
    if src == "false" {
        return Ok(Expr::Bool(false));
    }
    if src == "null" {
        return Ok(Expr::Null);
    }
    if src == "undefined" {
        return Ok(Expr::Undefined);
    }
    if src == "NaN" {
        return Ok(Expr::Float(f64::NAN));
    }
    if src == "Infinity" {
        return Ok(Expr::Float(f64::INFINITY));
    }
    if let Some(numeric) = parse_numeric_literal(src)? {
        return Ok(numeric);
    }

    if src.starts_with('`') && src.ends_with('`') && src.len() >= 2 {
        return parse_template_literal(src);
    }

    if (src.starts_with('\'') && src.ends_with('\''))
        || (src.starts_with('"') && src.ends_with('"'))
    {
        let value = parse_string_literal_exact(src)?;
        return Ok(Expr::String(value));
    }

    if let Some((pattern, flags)) = parse_regex_literal_expr(src)? {
        return Ok(Expr::RegexLiteral { pattern, flags });
    }

    if let Some(expr) = parse_new_date_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_new_regexp_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_new_function_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_new_callee_expr(src)? {
        return Ok(expr);
    }

    if parse_date_now_expr(src)? {
        return Ok(Expr::DateNow);
    }

    if parse_performance_now_expr(src)? {
        return Ok(Expr::PerformanceNow);
    }

    if let Some(value) = parse_date_parse_expr(src)? {
        return Ok(Expr::DateParse(Box::new(value)));
    }

    if let Some(args) = parse_date_utc_expr(src)? {
        return Ok(Expr::DateUtc { args });
    }

    if let Some(expr) = parse_intl_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_string_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_math_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_number_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_bigint_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_blob_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_array_buffer_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_typed_array_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_promise_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_map_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_url_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_url_search_params_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_set_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_symbol_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_regexp_static_expr(src)? {
        return Ok(expr);
    }

    if let Some(value) = parse_encode_uri_component_expr(src)? {
        return Ok(Expr::EncodeUriComponent(Box::new(value)));
    }

    if let Some(value) = parse_encode_uri_expr(src)? {
        return Ok(Expr::EncodeUri(Box::new(value)));
    }

    if let Some(value) = parse_decode_uri_component_expr(src)? {
        return Ok(Expr::DecodeUriComponent(Box::new(value)));
    }

    if let Some(value) = parse_decode_uri_expr(src)? {
        return Ok(Expr::DecodeUri(Box::new(value)));
    }

    if let Some(value) = parse_escape_expr(src)? {
        return Ok(Expr::Escape(Box::new(value)));
    }

    if let Some(value) = parse_unescape_expr(src)? {
        return Ok(Expr::Unescape(Box::new(value)));
    }

    if let Some(value) = parse_is_nan_expr(src)? {
        return Ok(Expr::IsNaN(Box::new(value)));
    }

    if let Some(value) = parse_is_finite_expr(src)? {
        return Ok(Expr::IsFinite(Box::new(value)));
    }

    if let Some(value) = parse_atob_expr(src)? {
        return Ok(Expr::Atob(Box::new(value)));
    }

    if let Some(value) = parse_btoa_expr(src)? {
        return Ok(Expr::Btoa(Box::new(value)));
    }

    if let Some((value, radix)) = parse_parse_int_expr(src)? {
        return Ok(Expr::ParseInt {
            value: Box::new(value),
            radix: radix.map(Box::new),
        });
    }

    if let Some(value) = parse_parse_float_expr(src)? {
        return Ok(Expr::ParseFloat(Box::new(value)));
    }

    if let Some(value) = parse_json_parse_expr(src)? {
        return Ok(Expr::JsonParse(Box::new(value)));
    }

    if let Some(value) = parse_json_stringify_expr(src)? {
        return Ok(Expr::JsonStringify(Box::new(value)));
    }

    if let Some(entries) = parse_object_literal_expr(src)? {
        return Ok(Expr::ObjectLiteral(entries));
    }

    if let Some(expr) = parse_object_static_expr(src)? {
        return Ok(expr);
    }

    if let Some(value) = parse_structured_clone_expr(src)? {
        return Ok(Expr::StructuredClone(Box::new(value)));
    }

    if let Some(value) = parse_fetch_expr(src)? {
        return Ok(Expr::Fetch(Box::new(value)));
    }

    if let Some(expr) = parse_match_media_expr(src)? {
        return Ok(expr);
    }

    if let Some(value) = parse_alert_expr(src)? {
        return Ok(Expr::Alert(Box::new(value)));
    }

    if let Some(value) = parse_confirm_expr(src)? {
        return Ok(Expr::Confirm(Box::new(value)));
    }

    if let Some((message, default)) = parse_prompt_expr(src)? {
        return Ok(Expr::Prompt {
            message: Box::new(message),
            default: default.map(Box::new),
        });
    }

    if let Some(values) = parse_array_literal_expr(src)? {
        return Ok(Expr::ArrayLiteral(values));
    }

    if let Some(value) = parse_array_is_array_expr(src)? {
        return Ok(Expr::ArrayIsArray(Box::new(value)));
    }

    if let Some(expr) = parse_array_from_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_array_access_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_array_buffer_access_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_typed_array_access_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_url_search_params_access_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_map_access_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_set_access_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_promise_method_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_location_method_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_history_method_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_clipboard_method_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_string_method_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_date_method_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_number_method_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_bigint_method_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_intl_format_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_regex_method_expr(src)? {
        return Ok(expr);
    }

    if let Some(tag_name) = parse_document_create_element_expr(src)? {
        return Ok(Expr::CreateElement(tag_name));
    }

    if let Some(text) = parse_document_create_text_node_expr(src)? {
        return Ok(Expr::CreateTextNode(text));
    }

    if parse_document_has_focus_expr(src)? {
        return Ok(Expr::DocumentHasFocus);
    }

    if let Some(handler_expr) = parse_function_expr(src)? {
        return Ok(handler_expr);
    }

    if let Some((handler, delay_ms)) = parse_set_timeout_expr(src)? {
        return Ok(Expr::SetTimeout {
            handler,
            delay_ms: Box::new(delay_ms),
        });
    }

    if let Some((handler, delay_ms)) = parse_set_interval_expr(src)? {
        return Ok(Expr::SetInterval {
            handler,
            delay_ms: Box::new(delay_ms),
        });
    }

    if let Some(callback) = parse_request_animation_frame_expr(src)? {
        return Ok(Expr::RequestAnimationFrame { callback });
    }

    if let Some(handler) = parse_queue_microtask_expr(src)? {
        return Ok(Expr::QueueMicrotask { handler });
    }

    if let Some((target, class_name)) = parse_class_list_contains_expr(src)? {
        return Ok(Expr::ClassListContains { target, class_name });
    }

    if let Some(target) = parse_query_selector_all_length_expr(src)? {
        return Ok(Expr::QuerySelectorAllLength { target });
    }

    if let Some(form) = parse_form_elements_length_expr(src)? {
        return Ok(Expr::FormElementsLength { form });
    }

    if let Some((source, name)) = parse_form_data_get_all_length_expr(src)? {
        return Ok(Expr::FormDataGetAllLength { source, name });
    }

    if let Some((source, name)) = parse_form_data_get_all_expr(src)? {
        return Ok(Expr::FormDataGetAll { source, name });
    }

    if let Some((source, name)) = parse_form_data_get_expr(src)? {
        return Ok(Expr::FormDataGet { source, name });
    }

    if let Some((source, name)) = parse_form_data_has_expr(src)? {
        return Ok(Expr::FormDataHas { source, name });
    }

    if let Some(form) = parse_new_form_data_expr(src)? {
        return Ok(Expr::FormDataNew { form });
    }

    if let Some((target, name)) = parse_get_attribute_expr(src)? {
        return Ok(Expr::DomGetAttribute { target, name });
    }

    if let Some((target, name)) = parse_has_attribute_expr(src)? {
        return Ok(Expr::DomHasAttribute { target, name });
    }

    if let Some((target, selector)) = parse_dom_matches_expr(src)? {
        return Ok(Expr::DomMatches { target, selector });
    }

    if let Some((target, selector)) = parse_dom_closest_expr(src)? {
        return Ok(Expr::DomClosest { target, selector });
    }

    if let Some((target, property)) = parse_dom_computed_style_property_expr(src)? {
        return Ok(Expr::DomComputedStyleProperty { target, property });
    }

    if let Some((event_var, prop)) = parse_event_property_expr(src)? {
        return Ok(Expr::EventProp { event_var, prop });
    }

    if let Some((target, prop)) = parse_dom_access(src)? {
        return Ok(Expr::DomRead { target, prop });
    }

    if let Some(expr) = parse_object_has_own_property_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_object_get_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_function_call_expr(src)? {
        return Ok(expr);
    }

    if let Some(target) = parse_element_ref_expr(src)? {
        return Ok(Expr::DomRef(target));
    }

    if let Some(expr) = parse_member_call_expr(src)? {
        return Ok(expr);
    }

    if is_ident(src) {
        return Ok(Expr::Var(src.to_string()));
    }

    Err(Error::ScriptParse(format!("unsupported expression: {src}")))
}

fn parse_numeric_literal(src: &str) -> Result<Option<Expr>> {
    if src.is_empty() {
        return Ok(None);
    }

    if let Some(value) = parse_bigint_literal(src)? {
        return Ok(Some(value));
    }

    if let Some(value) = parse_prefixed_integer_literal(src, "0x", 16)? {
        return Ok(Some(value));
    }
    if let Some(value) = parse_prefixed_integer_literal(src, "0o", 8)? {
        return Ok(Some(value));
    }
    if let Some(value) = parse_prefixed_integer_literal(src, "0b", 2)? {
        return Ok(Some(value));
    }

    if src.as_bytes().iter().any(|b| matches!(b, b'e' | b'E')) {
        if !matches!(src.as_bytes().first(), Some(b) if b.is_ascii_digit() || *b == b'.') {
            return Ok(None);
        }
        let n: f64 = src
            .parse()
            .map_err(|_| Error::ScriptParse(format!("invalid numeric literal: {src}")))?;
        if !n.is_finite() {
            return Err(Error::ScriptParse(format!(
                "invalid numeric literal: {src}"
            )));
        }
        return Ok(Some(Expr::Float(n)));
    }

    if src.as_bytes().iter().all(|b| b.is_ascii_digit()) {
        let n: i64 = src
            .parse()
            .map_err(|_| Error::ScriptParse(format!("invalid numeric literal: {src}")))?;
        return Ok(Some(Expr::Number(n)));
    }

    let mut dot_count = 0usize;
    for b in src.as_bytes() {
        if *b == b'.' {
            dot_count += 1;
        } else if !b.is_ascii_digit() {
            return Ok(None);
        }
    }

    if dot_count != 1 {
        return Ok(None);
    }
    if src.starts_with('.') || src.ends_with('.') {
        return Ok(None);
    }

    let n: f64 = src
        .parse()
        .map_err(|_| Error::ScriptParse(format!("invalid numeric literal: {src}")))?;
    if !n.is_finite() {
        return Err(Error::ScriptParse(format!(
            "invalid numeric literal: {src}"
        )));
    }
    Ok(Some(Expr::Float(n)))
}

fn parse_bigint_literal(src: &str) -> Result<Option<Expr>> {
    let Some(raw) = src.strip_suffix('n') else {
        return Ok(None);
    };
    if raw.is_empty() || !raw.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        return Ok(None);
    }

    let (digits, radix) =
        if let Some(hex) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
            (hex, 16u32)
        } else if let Some(octal) = raw.strip_prefix("0o").or_else(|| raw.strip_prefix("0O")) {
            (octal, 8u32)
        } else if let Some(binary) = raw.strip_prefix("0b").or_else(|| raw.strip_prefix("0B")) {
            (binary, 2u32)
        } else {
            if raw.len() > 1 && raw.starts_with('0') {
                return Err(Error::ScriptParse(format!(
                    "invalid numeric literal: {src}"
                )));
            }
            (raw, 10u32)
        };

    if digits.is_empty() {
        return Err(Error::ScriptParse(format!(
            "invalid numeric literal: {src}"
        )));
    }

    let value = JsBigInt::parse_bytes(digits.as_bytes(), radix)
        .ok_or_else(|| Error::ScriptParse(format!("invalid numeric literal: {src}")))?;
    Ok(Some(Expr::BigInt(value)))
}

fn parse_prefixed_integer_literal(src: &str, prefix: &str, radix: u32) -> Result<Option<Expr>> {
    let src = src.to_ascii_lowercase();
    if !src.starts_with(prefix) {
        return Ok(None);
    }

    let digits = &src[prefix.len()..];
    if digits.is_empty() {
        return Err(Error::ScriptParse(format!(
            "invalid numeric literal: {src}"
        )));
    }

    let n = i64::from_str_radix(digits, radix)
        .map_err(|_| Error::ScriptParse(format!("invalid numeric literal: {src}")))?;
    Ok(Some(Expr::Number(n)))
}

fn strip_keyword_operator<'a>(src: &'a str, keyword: &str) -> Option<&'a str> {
    if !src.starts_with(keyword) {
        return None;
    }

    let after = &src[keyword.len()..];
    if after.is_empty() || !is_ident_char(after.as_bytes()[0]) {
        return Some(after.trim_start());
    }

    None
}

fn parse_element_ref_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    if matches!(target, DomQuery::Var(_) | DomQuery::DocumentRoot) {
        return Ok(None);
    }
    Ok(Some(target))
}

fn parse_document_create_element_expr(src: &str) -> Result<Option<String>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("document") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("createElement") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let tag_name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(tag_name.to_ascii_lowercase()))
}

fn parse_document_create_text_node_expr(src: &str) -> Result<Option<String>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("document") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("createTextNode") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let text = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(text))
}

fn parse_document_has_focus_expr(src: &str) -> Result<bool> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(false);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("document") {
        return Ok(false);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(false);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("hasFocus") {
        return Ok(false);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(false);
        }
    }
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    if !args_src.trim().is_empty() {
        return Err(Error::ScriptParse(
            "document.hasFocus takes no arguments".into(),
        ));
    }
    cursor.skip_ws();
    Ok(cursor.eof())
}

fn parse_location_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if !parse_location_base(&mut cursor) {
        return Ok(None);
    }

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method_name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let method = match method_name.as_str() {
        "assign" => LocationMethod::Assign,
        "reload" => LocationMethod::Reload,
        "replace" => LocationMethod::Replace,
        "toString" => LocationMethod::ToString,
        _ => return Ok(None),
    };

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    let args = if args.len() == 1 && args[0].trim().is_empty() {
        Vec::new()
    } else {
        args
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let url = match method {
        LocationMethod::Assign | LocationMethod::Replace => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(format!(
                    "location.{} requires exactly one argument",
                    method_name
                )));
            }
            Some(Box::new(parse_expr(args[0].trim())?))
        }
        LocationMethod::Reload | LocationMethod::ToString => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "location.{} takes no arguments",
                    method_name
                )));
            }
            None
        }
    };

    Ok(Some(Expr::LocationMethodCall { method, url }))
}

fn parse_location_base(cursor: &mut Cursor<'_>) -> bool {
    let start = cursor.pos();

    if cursor.consume_ascii("location") {
        if cursor.peek().is_none_or(|ch| !is_ident_char(ch)) {
            return true;
        }
        cursor.set_pos(start);
    }

    if cursor.consume_ascii("document") {
        if cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            if cursor.consume_ascii("location") && cursor.peek().is_none_or(|ch| !is_ident_char(ch))
            {
                return true;
            }
        }
        cursor.set_pos(start);
    }

    if cursor.consume_ascii("window") {
        if cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if cursor.consume_ascii("location") && cursor.peek().is_none_or(|ch| !is_ident_char(ch)) {
            return true;
        }
        cursor.set_pos(start);
        if cursor.consume_ascii("window") {
            cursor.skip_ws();
            if !cursor.consume_byte(b'.') {
                cursor.set_pos(start);
                return false;
            }
            cursor.skip_ws();
            if !cursor.consume_ascii("document") {
                cursor.set_pos(start);
                return false;
            }
            cursor.skip_ws();
            if !cursor.consume_byte(b'.') {
                cursor.set_pos(start);
                return false;
            }
            cursor.skip_ws();
            if cursor.consume_ascii("location") && cursor.peek().is_none_or(|ch| !is_ident_char(ch))
            {
                return true;
            }
        }
        cursor.set_pos(start);
    }

    false
}

fn parse_history_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if !parse_history_base(&mut cursor) {
        return Ok(None);
    }

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method_name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let method = match method_name.as_str() {
        "back" => HistoryMethod::Back,
        "forward" => HistoryMethod::Forward,
        "go" => HistoryMethod::Go,
        "pushState" => HistoryMethod::PushState,
        "replaceState" => HistoryMethod::ReplaceState,
        _ => return Ok(None),
    };

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    let args = if args.len() == 1 && args[0].trim().is_empty() {
        Vec::new()
    } else {
        args
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let mut parsed_args = Vec::new();
    match method {
        HistoryMethod::Back | HistoryMethod::Forward => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "history.{} takes no arguments",
                    method_name
                )));
            }
        }
        HistoryMethod::Go => {
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "history.go accepts zero or one argument".into(),
                ));
            }
            if let Some(arg) = args.first() {
                if arg.trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "history.go argument cannot be empty".into(),
                    ));
                }
                parsed_args.push(parse_expr(arg.trim())?);
            }
        }
        HistoryMethod::PushState | HistoryMethod::ReplaceState => {
            if args.len() < 2 || args.len() > 3 {
                return Err(Error::ScriptParse(format!(
                    "history.{} requires 2 or 3 arguments",
                    method_name
                )));
            }
            for arg in args {
                let arg = arg.trim();
                if arg.is_empty() {
                    return Err(Error::ScriptParse(format!(
                        "history.{} arguments cannot be empty",
                        method_name
                    )));
                }
                parsed_args.push(parse_expr(arg)?);
            }
        }
    }

    Ok(Some(Expr::HistoryMethodCall {
        method,
        args: parsed_args,
    }))
}

fn parse_history_base(cursor: &mut Cursor<'_>) -> bool {
    let start = cursor.pos();

    if cursor.consume_ascii("history") {
        if cursor.peek().is_none_or(|ch| !is_ident_char(ch)) {
            return true;
        }
        cursor.set_pos(start);
    }

    if cursor.consume_ascii("window") {
        if cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if cursor.consume_ascii("history") && cursor.peek().is_none_or(|ch| !is_ident_char(ch)) {
            return true;
        }
    }

    cursor.set_pos(start);
    false
}

fn parse_clipboard_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if !parse_clipboard_base(&mut cursor) {
        return Ok(None);
    }

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method_name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let method = match method_name.as_str() {
        "readText" => ClipboardMethod::ReadText,
        "writeText" => ClipboardMethod::WriteText,
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    let args = if args.len() == 1 && args[0].trim().is_empty() {
        Vec::new()
    } else {
        args
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let mut parsed_args = Vec::new();
    match method {
        ClipboardMethod::ReadText => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "navigator.clipboard.readText takes no arguments".into(),
                ));
            }
        }
        ClipboardMethod::WriteText => {
            if args.len() != 1 {
                return Err(Error::ScriptParse(
                    "navigator.clipboard.writeText requires exactly one argument".into(),
                ));
            }
            if args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "navigator.clipboard.writeText argument cannot be empty".into(),
                ));
            }
            parsed_args.push(parse_expr(args[0].trim())?);
        }
    }

    Ok(Some(Expr::ClipboardMethodCall {
        method,
        args: parsed_args,
    }))
}

fn parse_clipboard_base(cursor: &mut Cursor<'_>) -> bool {
    let start = cursor.pos();

    if cursor.consume_ascii("navigator") {
        if cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            if cursor.consume_ascii("clipboard")
                && cursor.peek().is_none_or(|ch| !is_ident_char(ch))
            {
                return true;
            }
        }
        cursor.set_pos(start);
    }

    if cursor.consume_ascii("window") {
        if cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if !cursor.consume_ascii("navigator") || cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if cursor.consume_ascii("clipboard") && cursor.peek().is_none_or(|ch| !is_ident_char(ch)) {
            return true;
        }
    }

    cursor.set_pos(start);
    false
}

fn parse_new_date_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Date") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args: Vec<String> = if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args.into_iter().map(|arg| arg.to_string()).collect()
        }
    } else {
        Vec::new()
    };

    if args.len() > 1 {
        return Err(Error::ScriptParse(
            "new Date supports zero or one argument".into(),
        ));
    }

    let value = if args.len() == 1 {
        if args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "new Date argument cannot be empty".into(),
            ));
        }
        Some(Box::new(parse_expr(args[0].trim())?))
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::DateNew { value }))
}

fn parse_regex_literal_expr(src: &str) -> Result<Option<(String, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some((pattern, flags)) = parse_regex_literal_from_cursor(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((pattern, flags)))
}

fn parse_regex_literal_from_cursor(cursor: &mut Cursor<'_>) -> Result<Option<(String, String)>> {
    cursor.skip_ws();
    if cursor.peek() != Some(b'/') {
        return Ok(None);
    }
    let start = cursor.i;
    let bytes = cursor.bytes();
    let mut i = cursor.i + 1;
    let mut escaped = false;
    let mut in_class = false;

    while i < bytes.len() {
        let b = bytes[i];
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if b == b'\\' {
            escaped = true;
            i += 1;
            continue;
        }
        if b == b'[' && !in_class {
            in_class = true;
            i += 1;
            continue;
        }
        if b == b']' && in_class {
            in_class = false;
            i += 1;
            continue;
        }
        if b == b'/' && !in_class {
            break;
        }
        if b == b'\n' || b == b'\r' {
            return Err(Error::ScriptParse("unterminated regex literal".into()));
        }
        i += 1;
    }

    if i >= bytes.len() || bytes[i] != b'/' {
        return Err(Error::ScriptParse("unterminated regex literal".into()));
    }

    let pattern = cursor
        .src
        .get(start + 1..i)
        .ok_or_else(|| Error::ScriptParse("invalid regex literal".into()))?
        .to_string();
    i += 1;
    let flags_start = i;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }
    let flags = cursor
        .src
        .get(flags_start..i)
        .ok_or_else(|| Error::ScriptParse("invalid regex flags".into()))?
        .to_string();

    let info = Harness::analyze_regex_flags(&flags).map_err(Error::ScriptParse)?;
    Harness::compile_regex(&pattern, info).map_err(|err| {
        Error::ScriptParse(format!(
            "invalid regular expression: /{pattern}/{flags}: {err}"
        ))
    })?;

    cursor.i = i;
    Ok(Some((pattern, flags)))
}

fn parse_new_regexp_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(expr) = parse_new_regexp_expr_from_cursor(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}

fn parse_new_regexp_expr_from_cursor(cursor: &mut Cursor<'_>) -> Result<Option<Expr>> {
    let start = cursor.i;
    cursor.skip_ws();
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                cursor.i = start;
                return Ok(None);
            }
        }
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            cursor.i = start;
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("RegExp") {
        cursor.i = start;
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            cursor.i = start;
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        cursor.i = start;
        return Ok(None);
    }
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    if args.len() > 2 {
        return Err(Error::ScriptParse(
            "RegExp supports up to two arguments".into(),
        ));
    }
    if !args.is_empty() && args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "RegExp pattern argument cannot be empty".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "RegExp flags argument cannot be empty".into(),
        ));
    }

    let pattern = if args.is_empty() {
        Box::new(Expr::String(String::new()))
    } else {
        Box::new(parse_expr(args[0].trim())?)
    };
    let flags = if args.len() == 2 {
        Some(Box::new(parse_expr(args[1].trim())?))
    } else {
        None
    };

    Ok(Some(Expr::RegexNew { pattern, flags }))
}

fn parse_regexp_static_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("RegExp") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if member != "escape" {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() != 1 || args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "RegExp.escape requires exactly one argument".into(),
            ));
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::RegExpStaticMethod {
            method: RegExpStaticMethod::Escape,
            args: vec![parse_expr(args[0].trim())?],
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::RegExpConstructor))
}

fn parse_new_function_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Function") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    if args.is_empty() {
        return Err(Error::ScriptParse(
            "new Function requires at least one argument".into(),
        ));
    }

    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Error::ScriptParse(
                "new Function arguments cannot be empty".into(),
            ));
        }
        parsed.push(parse_expr(arg)?);
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::FunctionConstructor { args: parsed }))
}

fn parse_regex_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let (receiver, receiver_is_identifier) =
        if let Some((pattern, flags)) = parse_regex_literal_from_cursor(&mut cursor)? {
            (Expr::RegexLiteral { pattern, flags }, false)
        } else if let Some(expr) = parse_new_regexp_expr_from_cursor(&mut cursor)? {
            (expr, false)
        } else if let Some(name) = cursor.parse_identifier() {
            (Expr::Var(name), true)
        } else {
            return Ok(None);
        };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if !matches!(method.as_str(), "test" | "exec" | "toString") {
        return Ok(None);
    }
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    let input = match method.as_str() {
        "test" | "exec" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(format!(
                    "RegExp.{} requires exactly one argument",
                    method
                )));
            }
            Some(Box::new(parse_expr(args[0].trim())?))
        }
        "toString" => {
            if !(args.len() == 1 && args[0].trim().is_empty()) {
                if receiver_is_identifier {
                    return Ok(None);
                }
                return Err(Error::ScriptParse(
                    "RegExp.toString does not take arguments".into(),
                ));
            }
            None
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let regex = Box::new(receiver);
    match method.as_str() {
        "test" => Ok(Some(Expr::RegexTest {
            regex,
            input: input.expect("validated"),
        })),
        "exec" => Ok(Some(Expr::RegexExec {
            regex,
            input: input.expect("validated"),
        })),
        "toString" => Ok(Some(Expr::RegexToString { regex })),
        _ => Ok(None),
    }
}

fn parse_date_now_expr(src: &str) -> Result<bool> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(false);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Date") {
        return Ok(false);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(false);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(false);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("now") {
        return Ok(false);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    Ok(cursor.eof())
}

fn parse_performance_now_expr(src: &str) -> Result<bool> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(false);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("performance") {
        return Ok(false);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(false);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(false);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("now") {
        return Ok(false);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(false);
        }
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    Ok(cursor.eof())
}

fn parse_date_static_args_expr(src: &str, method: &str) -> Result<Option<Vec<String>>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Date") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii(method) {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',')
        .into_iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>();
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(args))
}

fn parse_date_parse_expr(src: &str) -> Result<Option<Expr>> {
    let Some(args) = parse_date_static_args_expr(src, "parse")? else {
        return Ok(None);
    };
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Date.parse requires exactly one argument".into(),
        ));
    }
    Ok(Some(parse_expr(args[0].trim())?))
}

fn parse_date_utc_expr(src: &str) -> Result<Option<Vec<Expr>>> {
    let Some(args) = parse_date_static_args_expr(src, "UTC")? else {
        return Ok(None);
    };

    if args.len() < 2 || args.len() > 7 {
        return Err(Error::ScriptParse(
            "Date.UTC requires between 2 and 7 arguments".into(),
        ));
    }

    let mut out = Vec::with_capacity(args.len());
    for arg in args {
        if arg.trim().is_empty() {
            return Err(Error::ScriptParse(
                "Date.UTC argument cannot be empty".into(),
            ));
        }
        out.push(parse_expr(arg.trim())?);
    }
    Ok(Some(out))
}

fn parse_intl_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Intl") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if called_with_new && cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(
                    "Intl constructor argument cannot be empty".into(),
                ));
            }
            parsed.push(parse_expr(arg)?);
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlConstruct { args: parsed }));
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if member == "Collator" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::Collator,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(collator_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if collator_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.Collator".to_string())));
            }

            if collator_member == "supportedLocalesOf" {
                if cursor.peek() != Some(b'(') {
                    return Ok(None);
                }
                let args_src = cursor.read_balanced_block(b'(', b')')?;
                let raw_args = split_top_level_by_char(&args_src, b',');
                let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                    Vec::new()
                } else {
                    raw_args
                };
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.Collator.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.Collator.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::CollatorSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "Intl.Collator supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.Collator argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::Collator,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "DateTimeFormat" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::DateTimeFormat,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(dtf_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if dtf_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.DateTimeFormat".to_string())));
            }

            if dtf_member == "supportedLocalesOf" {
                if cursor.peek() != Some(b'(') {
                    return Ok(None);
                }
                let args_src = cursor.read_balanced_block(b'(', b')')?;
                let raw_args = split_top_level_by_char(&args_src, b',');
                let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                    Vec::new()
                } else {
                    raw_args
                };
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DateTimeFormat.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DateTimeFormat.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::DateTimeFormatSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "Intl.DateTimeFormat supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.DateTimeFormat argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::DateTimeFormat,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "DisplayNames" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::DisplayNames,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(display_names_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if display_names_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.DisplayNames".to_string())));
            }

            if display_names_member == "supportedLocalesOf" {
                if cursor.peek() != Some(b'(') {
                    return Ok(None);
                }
                let args_src = cursor.read_balanced_block(b'(', b')')?;
                let raw_args = split_top_level_by_char(&args_src, b',');
                let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                    Vec::new()
                } else {
                    raw_args
                };
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DisplayNames.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DisplayNames.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::DisplayNamesSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "Intl.DisplayNames supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.DisplayNames argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::DisplayNames,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "DurationFormat" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::DurationFormat,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(duration_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if duration_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.DurationFormat".to_string())));
            }

            if duration_member == "supportedLocalesOf" {
                if cursor.peek() != Some(b'(') {
                    return Ok(None);
                }
                let args_src = cursor.read_balanced_block(b'(', b')')?;
                let raw_args = split_top_level_by_char(&args_src, b',');
                let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                    Vec::new()
                } else {
                    raw_args
                };
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DurationFormat.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DurationFormat.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::DurationFormatSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "Intl.DurationFormat supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.DurationFormat argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::DurationFormat,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "ListFormat" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::ListFormat,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(list_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if list_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.ListFormat".to_string())));
            }

            if list_member == "supportedLocalesOf" {
                if cursor.peek() != Some(b'(') {
                    return Ok(None);
                }
                let args_src = cursor.read_balanced_block(b'(', b')')?;
                let raw_args = split_top_level_by_char(&args_src, b',');
                let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                    Vec::new()
                } else {
                    raw_args
                };
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.ListFormat.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.ListFormat.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::ListFormatSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "Intl.ListFormat supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.ListFormat argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::ListFormat,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "PluralRules" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::PluralRules,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(plural_rules_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if plural_rules_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.PluralRules".to_string())));
            }

            if plural_rules_member == "supportedLocalesOf" {
                if cursor.peek() != Some(b'(') {
                    return Ok(None);
                }
                let args_src = cursor.read_balanced_block(b'(', b')')?;
                let raw_args = split_top_level_by_char(&args_src, b',');
                let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                    Vec::new()
                } else {
                    raw_args
                };
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.PluralRules.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.PluralRules.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::PluralRulesSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "Intl.PluralRules supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.PluralRules argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::PluralRules,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "RelativeTimeFormat" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::RelativeTimeFormat,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(relative_time_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if relative_time_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.RelativeTimeFormat".to_string())));
            }

            if relative_time_member == "supportedLocalesOf" {
                if cursor.peek() != Some(b'(') {
                    return Ok(None);
                }
                let args_src = cursor.read_balanced_block(b'(', b')')?;
                let raw_args = split_top_level_by_char(&args_src, b',');
                let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                    Vec::new()
                } else {
                    raw_args
                };
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.RelativeTimeFormat.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.RelativeTimeFormat.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::RelativeTimeFormatSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "Intl.RelativeTimeFormat supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.RelativeTimeFormat argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::RelativeTimeFormat,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "Segmenter" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::Segmenter,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(segmenter_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if segmenter_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.Segmenter".to_string())));
            }

            if segmenter_member == "supportedLocalesOf" {
                if cursor.peek() != Some(b'(') {
                    return Ok(None);
                }
                let args_src = cursor.read_balanced_block(b'(', b')')?;
                let raw_args = split_top_level_by_char(&args_src, b',');
                let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                    Vec::new()
                } else {
                    raw_args
                };
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.Segmenter.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.Segmenter.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::SegmenterSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "Intl.Segmenter supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.Segmenter argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::Segmenter,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "Locale" {
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(locale_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if locale_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.Locale".to_string())));
            }

            return Ok(None);
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "Intl.Locale requires a locale identifier and optional options".into(),
            ));
        }
        if args.len() == 2 && args[1].trim().is_empty() {
            return Err(Error::ScriptParse(
                "Intl.Locale options cannot be empty".into(),
            ));
        }
        let tag = Box::new(parse_expr(args[0].trim())?);
        let options = if args.len() == 2 {
            Some(Box::new(parse_expr(args[1].trim())?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlLocaleConstruct {
            tag,
            options,
            called_with_new,
        }));
    }

    let intl_formatter_kind = match member.as_str() {
        "NumberFormat" => Some(IntlFormatterKind::NumberFormat),
        _ => None,
    };
    if let Some(kind) = intl_formatter_kind {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(format!(
                "Intl.{member} supports up to two arguments"
            )));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(format!(
                "Intl.{member} argument cannot be empty"
            )));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind,
            locales,
            options,
            called_with_new,
        }));
    }

    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };
    let expr = match member.as_str() {
        "getCanonicalLocales" => {
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "Intl.getCanonicalLocales supports zero or one argument".into(),
                ));
            }
            if args.len() == 1 && args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.getCanonicalLocales argument cannot be empty".into(),
                ));
            }
            let mut parsed = Vec::new();
            if let Some(arg) = args.first() {
                parsed.push(parse_expr(arg.trim())?);
            }
            Expr::IntlStaticMethod {
                method: IntlStaticMethod::GetCanonicalLocales,
                args: parsed,
            }
        }
        "supportedValuesOf" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.supportedValuesOf requires exactly one argument".into(),
                ));
            }
            Expr::IntlStaticMethod {
                method: IntlStaticMethod::SupportedValuesOf,
                args: vec![parse_expr(args[0].trim())?],
            }
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}

fn parse_math_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }
    if !cursor.consume_ascii("Math") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.consume_byte(b'[') {
        cursor.skip_ws();
        if !cursor.consume_ascii("Symbol") {
            return Ok(None);
        }
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
        if !cursor.consume_ascii("toStringTag") {
            return Ok(None);
        }
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        cursor.skip_ws();
        cursor.expect_byte(b']')?;
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MathConst(MathConst::ToStringTag)));
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let Some(method) = parse_math_method_name(&member) else {
            return Ok(None);
        };
        validate_math_arity(method, args.len())?;

        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "Math.{} argument cannot be empty",
                    member
                )));
            }
            parsed.push(parse_expr(arg)?);
        }

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MathMethod {
            method,
            args: parsed,
        }));
    }

    let Some(constant) = parse_math_const_name(&member) else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::MathConst(constant)))
}

fn parse_math_const_name(name: &str) -> Option<MathConst> {
    match name {
        "E" => Some(MathConst::E),
        "LN10" => Some(MathConst::Ln10),
        "LN2" => Some(MathConst::Ln2),
        "LOG10E" => Some(MathConst::Log10E),
        "LOG2E" => Some(MathConst::Log2E),
        "PI" => Some(MathConst::Pi),
        "SQRT1_2" => Some(MathConst::Sqrt1_2),
        "SQRT2" => Some(MathConst::Sqrt2),
        _ => None,
    }
}

fn parse_math_method_name(name: &str) -> Option<MathMethod> {
    match name {
        "abs" => Some(MathMethod::Abs),
        "acos" => Some(MathMethod::Acos),
        "acosh" => Some(MathMethod::Acosh),
        "asin" => Some(MathMethod::Asin),
        "asinh" => Some(MathMethod::Asinh),
        "atan" => Some(MathMethod::Atan),
        "atan2" => Some(MathMethod::Atan2),
        "atanh" => Some(MathMethod::Atanh),
        "cbrt" => Some(MathMethod::Cbrt),
        "ceil" => Some(MathMethod::Ceil),
        "clz32" => Some(MathMethod::Clz32),
        "cos" => Some(MathMethod::Cos),
        "cosh" => Some(MathMethod::Cosh),
        "exp" => Some(MathMethod::Exp),
        "expm1" => Some(MathMethod::Expm1),
        "floor" => Some(MathMethod::Floor),
        "f16round" => Some(MathMethod::F16Round),
        "fround" => Some(MathMethod::FRound),
        "hypot" => Some(MathMethod::Hypot),
        "imul" => Some(MathMethod::Imul),
        "log" => Some(MathMethod::Log),
        "log10" => Some(MathMethod::Log10),
        "log1p" => Some(MathMethod::Log1p),
        "log2" => Some(MathMethod::Log2),
        "max" => Some(MathMethod::Max),
        "min" => Some(MathMethod::Min),
        "pow" => Some(MathMethod::Pow),
        "random" => Some(MathMethod::Random),
        "round" => Some(MathMethod::Round),
        "sign" => Some(MathMethod::Sign),
        "sin" => Some(MathMethod::Sin),
        "sinh" => Some(MathMethod::Sinh),
        "sqrt" => Some(MathMethod::Sqrt),
        "sumPrecise" => Some(MathMethod::SumPrecise),
        "tan" => Some(MathMethod::Tan),
        "tanh" => Some(MathMethod::Tanh),
        "trunc" => Some(MathMethod::Trunc),
        _ => None,
    }
}

fn validate_math_arity(method: MathMethod, count: usize) -> Result<()> {
    let method_name = match method {
        MathMethod::Abs => "abs",
        MathMethod::Acos => "acos",
        MathMethod::Acosh => "acosh",
        MathMethod::Asin => "asin",
        MathMethod::Asinh => "asinh",
        MathMethod::Atan => "atan",
        MathMethod::Atan2 => "atan2",
        MathMethod::Atanh => "atanh",
        MathMethod::Cbrt => "cbrt",
        MathMethod::Ceil => "ceil",
        MathMethod::Clz32 => "clz32",
        MathMethod::Cos => "cos",
        MathMethod::Cosh => "cosh",
        MathMethod::Exp => "exp",
        MathMethod::Expm1 => "expm1",
        MathMethod::Floor => "floor",
        MathMethod::F16Round => "f16round",
        MathMethod::FRound => "fround",
        MathMethod::Hypot => "hypot",
        MathMethod::Imul => "imul",
        MathMethod::Log => "log",
        MathMethod::Log10 => "log10",
        MathMethod::Log1p => "log1p",
        MathMethod::Log2 => "log2",
        MathMethod::Max => "max",
        MathMethod::Min => "min",
        MathMethod::Pow => "pow",
        MathMethod::Random => "random",
        MathMethod::Round => "round",
        MathMethod::Sign => "sign",
        MathMethod::Sin => "sin",
        MathMethod::Sinh => "sinh",
        MathMethod::Sqrt => "sqrt",
        MathMethod::SumPrecise => "sumPrecise",
        MathMethod::Tan => "tan",
        MathMethod::Tanh => "tanh",
        MathMethod::Trunc => "trunc",
    };

    let valid = match method {
        MathMethod::Random => count == 0,
        MathMethod::Atan2 | MathMethod::Imul | MathMethod::Pow => count == 2,
        MathMethod::Hypot | MathMethod::Max | MathMethod::Min => true,
        MathMethod::SumPrecise => count == 1,
        _ => count == 1,
    };

    if valid {
        return Ok(());
    }

    let message = match method {
        MathMethod::Random => format!("Math.{method_name} does not take arguments"),
        MathMethod::Atan2 | MathMethod::Imul | MathMethod::Pow => {
            format!("Math.{method_name} requires exactly two arguments")
        }
        MathMethod::SumPrecise => format!("Math.{method_name} requires exactly one argument"),
        _ => format!("Math.{method_name} requires exactly one argument"),
    };
    Err(Error::ScriptParse(message))
}

fn parse_string_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("String") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "String supports zero or one argument".into(),
            ));
        }
        let value = if let Some(arg) = args.first() {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse("String argument cannot be empty".into()));
            }
            Some(Box::new(parse_expr(arg)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::StringConstruct {
            value,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::StringConstruct {
                value: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let method = match member.as_str() {
            "fromCharCode" => StringStaticMethod::FromCharCode,
            "fromCodePoint" => StringStaticMethod::FromCodePoint,
            "raw" => StringStaticMethod::Raw,
            _ => return Ok(None),
        };

        if matches!(method, StringStaticMethod::Raw) && args.is_empty() {
            return Err(Error::ScriptParse(
                "String.raw requires at least one argument".into(),
            ));
        }

        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "String.{member} argument cannot be empty"
                )));
            }
            parsed.push(parse_expr(arg)?);
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::StringStaticMethod {
            method,
            args: parsed,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::StringConstructor))
}

fn parse_number_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut has_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        has_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Number") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if has_new && cursor.peek() != Some(b'(') {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::NumberConstruct { value: None }));
        }
        return Ok(None);
    }

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Number supports zero or one argument".into(),
            ));
        }
        let value = if let Some(arg) = args.first() {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse("Number argument cannot be empty".into()));
            }
            Some(Box::new(parse_expr(arg)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::NumberConstruct { value }));
    }

    if has_new {
        return Ok(None);
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let Some(method) = parse_number_method_name(&member) else {
            return Ok(None);
        };

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let parsed = match method {
            NumberMethod::IsFinite
            | NumberMethod::IsInteger
            | NumberMethod::IsNaN
            | NumberMethod::IsSafeInteger
            | NumberMethod::ParseFloat => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(format!(
                        "Number.{member} requires exactly one argument"
                    )));
                }
                vec![parse_expr(args[0].trim())?]
            }
            NumberMethod::ParseInt => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Number.parseInt requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Number.parseInt radix argument cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                parsed
            }
        };

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::NumberMethod {
            method,
            args: parsed,
        }));
    }

    let Some(constant) = parse_number_const_name(&member) else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::NumberConst(constant)))
}

fn parse_number_const_name(name: &str) -> Option<NumberConst> {
    match name {
        "EPSILON" => Some(NumberConst::Epsilon),
        "MAX_SAFE_INTEGER" => Some(NumberConst::MaxSafeInteger),
        "MAX_VALUE" => Some(NumberConst::MaxValue),
        "MIN_SAFE_INTEGER" => Some(NumberConst::MinSafeInteger),
        "MIN_VALUE" => Some(NumberConst::MinValue),
        "NaN" => Some(NumberConst::NaN),
        "NEGATIVE_INFINITY" => Some(NumberConst::NegativeInfinity),
        "POSITIVE_INFINITY" => Some(NumberConst::PositiveInfinity),
        _ => None,
    }
}

fn parse_number_method_name(name: &str) -> Option<NumberMethod> {
    match name {
        "isFinite" => Some(NumberMethod::IsFinite),
        "isInteger" => Some(NumberMethod::IsInteger),
        "isNaN" => Some(NumberMethod::IsNaN),
        "isSafeInteger" => Some(NumberMethod::IsSafeInteger),
        "parseFloat" => Some(NumberMethod::ParseFloat),
        "parseInt" => Some(NumberMethod::ParseInt),
        _ => None,
    }
}

fn parse_bigint_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("BigInt") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "BigInt supports zero or one argument".into(),
            ));
        }
        let value = if let Some(arg) = args.first() {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse("BigInt argument cannot be empty".into()));
            }
            Some(Box::new(parse_expr(arg)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::BigIntConstruct {
            value,
            called_with_new,
        }));
    }

    if called_with_new {
        return Ok(None);
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let Some(method) = parse_bigint_method_name(&member) else {
        return Ok(None);
    };
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };
    if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
        return Err(Error::ScriptParse(format!(
            "BigInt.{member} requires exactly two arguments"
        )));
    }

    let parsed = vec![parse_expr(args[0].trim())?, parse_expr(args[1].trim())?];
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::BigIntMethod {
        method,
        args: parsed,
    }))
}

fn parse_bigint_method_name(name: &str) -> Option<BigIntMethod> {
    match name {
        "asIntN" => Some(BigIntMethod::AsIntN),
        "asUintN" => Some(BigIntMethod::AsUintN),
        _ => None,
    }
}

fn parse_typed_array_kind_name(name: &str) -> Option<TypedArrayKind> {
    match name {
        "Int8Array" => Some(TypedArrayKind::Int8),
        "Uint8Array" => Some(TypedArrayKind::Uint8),
        "Uint8ClampedArray" => Some(TypedArrayKind::Uint8Clamped),
        "Int16Array" => Some(TypedArrayKind::Int16),
        "Uint16Array" => Some(TypedArrayKind::Uint16),
        "Int32Array" => Some(TypedArrayKind::Int32),
        "Uint32Array" => Some(TypedArrayKind::Uint32),
        "Float16Array" => Some(TypedArrayKind::Float16),
        "Float32Array" => Some(TypedArrayKind::Float32),
        "Float64Array" => Some(TypedArrayKind::Float64),
        "BigInt64Array" => Some(TypedArrayKind::BigInt64),
        "BigUint64Array" => Some(TypedArrayKind::BigUint64),
        _ => None,
    }
}

fn parse_typed_array_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    let Some(constructor_name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let Some(kind) = parse_typed_array_kind_name(&constructor_name) else {
        return Ok(None);
    };
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "{} argument cannot be empty",
                    constructor_name
                )));
            }
            parsed.push(parse_expr(arg)?);
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::TypedArrayConstruct {
            kind,
            args: parsed,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::TypedArrayConstruct {
                kind,
                args: Vec::new(),
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();

        if member == "BYTES_PER_ELEMENT" {
            if !cursor.eof() {
                return Ok(None);
            }
            return Ok(Some(Expr::TypedArrayStaticBytesPerElement(kind)));
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        let method = match member.as_str() {
            "from" => TypedArrayStaticMethod::From,
            "of" => TypedArrayStaticMethod::Of,
            _ => return Ok(None),
        };
        if matches!(method, TypedArrayStaticMethod::From)
            && (args.len() != 1 || args[0].trim().is_empty())
        {
            return Err(Error::ScriptParse(format!(
                "{}.from requires exactly one argument",
                constructor_name
            )));
        }
        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "{}.{} argument cannot be empty",
                    constructor_name, member
                )));
            }
            parsed.push(parse_expr(arg)?);
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::TypedArrayStaticMethod {
            kind,
            method,
            args: parsed,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::TypedArrayConstructorRef(
        TypedArrayConstructorKind::Concrete(kind),
    )))
}

fn parse_promise_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Promise") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Promise supports exactly one executor argument".into(),
            ));
        }
        let executor = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse(
                    "Promise executor argument cannot be empty".into(),
                ));
            }
            Some(Box::new(parse_expr(first)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::PromiseConstruct {
            executor,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::PromiseConstruct {
                executor: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let method = match member.as_str() {
            "resolve" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.resolve supports zero or one argument".into(),
                    ));
                }
                PromiseStaticMethod::Resolve
            }
            "reject" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.reject supports zero or one argument".into(),
                    ));
                }
                PromiseStaticMethod::Reject
            }
            "all" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.all requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::All
            }
            "allSettled" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.allSettled requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::AllSettled
            }
            "any" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.any requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::Any
            }
            "race" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.race requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::Race
            }
            "try" => {
                if args.is_empty() || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.try requires at least one argument".into(),
                    ));
                }
                PromiseStaticMethod::Try
            }
            "withResolvers" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.withResolvers does not take arguments".into(),
                    ));
                }
                PromiseStaticMethod::WithResolvers
            }
            _ => return Ok(None),
        };

        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "Promise.{} argument cannot be empty",
                    member
                )));
            }
            parsed.push(parse_expr(arg)?);
        }

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::PromiseStaticMethod {
            method,
            args: parsed,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::PromiseConstructor))
}

fn parse_promise_method_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let dots = collect_top_level_char_positions(src, b'.');
    for dot in dots.into_iter().rev() {
        let Some(base_src) = src.get(..dot) else {
            continue;
        };
        let base_src = base_src.trim();
        if base_src.is_empty() {
            continue;
        }

        let Some(tail_src) = src.get(dot + 1..) else {
            continue;
        };
        let tail_src = tail_src.trim();
        let mut cursor = Cursor::new(tail_src);
        let Some(member) = cursor.parse_identifier() else {
            continue;
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            continue;
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        cursor.skip_ws();
        if !cursor.eof() {
            continue;
        }

        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let method = match member.as_str() {
            "then" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "Promise.then supports up to two arguments".into(),
                    ));
                }
                PromiseInstanceMethod::Then
            }
            "catch" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.catch supports at most one argument".into(),
                    ));
                }
                PromiseInstanceMethod::Catch
            }
            "finally" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.finally supports at most one argument".into(),
                    ));
                }
                PromiseInstanceMethod::Finally
            }
            _ => continue,
        };

        let mut parsed_args = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "Promise.{} argument cannot be empty",
                    member
                )));
            }
            parsed_args.push(parse_expr(arg)?);
        }

        return Ok(Some(Expr::PromiseMethod {
            target: Box::new(parse_expr(base_src)?),
            method,
            args: parsed_args,
        }));
    }
    Ok(None)
}

fn parse_blob_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Blob") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "Blob supports zero, one, or two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse("Blob argument cannot be empty".into()));
        }

        let parts = args
            .first()
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::BlobConstruct {
            parts,
            options,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::BlobConstruct {
                parts: None,
                options: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::BlobConstructor))
}

fn parse_map_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Map") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Map supports zero or one argument".into(),
            ));
        }
        let iterable = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse("Map argument cannot be empty".into()));
            }
            Some(Box::new(parse_expr(first)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MapConstruct {
            iterable,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::MapConstruct {
                iterable: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if member != "groupBy" {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
            return Err(Error::ScriptParse(
                "Map.groupBy requires exactly two arguments".into(),
            ));
        }
        let parsed = vec![parse_expr(args[0].trim())?, parse_expr(args[1].trim())?];
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MapStaticMethod {
            method: MapStaticMethod::GroupBy,
            args: parsed,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::MapConstructor))
}

fn parse_url_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("URL") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "URL supports one or two constructor arguments".into(),
            ));
        }
        if args.first().is_none_or(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "URL constructor requires a URL argument".into(),
            ));
        }
        if args.len() == 2 && args[1].trim().is_empty() {
            return Err(Error::ScriptParse(
                "URL base argument cannot be empty".into(),
            ));
        }
        let input = args
            .first()
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);
        let base = args
            .get(1)
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::UrlConstruct {
            input,
            base,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::UrlConstruct {
                input: None,
                base: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        let method = match member.as_str() {
            "canParse" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.canParse requires a URL argument and optional base".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.canParse base argument cannot be empty".into(),
                    ));
                }
                UrlStaticMethod::CanParse
            }
            "parse" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.parse requires a URL argument and optional base".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.parse base argument cannot be empty".into(),
                    ));
                }
                UrlStaticMethod::Parse
            }
            "createObjectURL" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.createObjectURL requires exactly one argument".into(),
                    ));
                }
                UrlStaticMethod::CreateObjectUrl
            }
            "revokeObjectURL" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.revokeObjectURL requires exactly one argument".into(),
                    ));
                }
                UrlStaticMethod::RevokeObjectUrl
            }
            _ => return Ok(None),
        };
        let mut parsed_args = Vec::with_capacity(args.len());
        for arg in args {
            parsed_args.push(parse_expr(arg.trim())?);
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::UrlStaticMethod {
            method,
            args: parsed_args,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::UrlConstructor))
}

fn parse_url_search_params_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("URLSearchParams") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "URLSearchParams supports zero or one argument".into(),
            ));
        }
        let init = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse(
                    "URLSearchParams argument cannot be empty".into(),
                ));
            }
            Some(Box::new(parse_expr(first)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::UrlSearchParamsConstruct {
            init,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::UrlSearchParamsConstruct {
                init: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    Ok(None)
}

fn parse_set_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Set") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Set supports zero or one argument".into(),
            ));
        }
        let iterable = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse("Set argument cannot be empty".into()));
            }
            Some(Box::new(parse_expr(first)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::SetConstruct {
            iterable,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::SetConstruct {
                iterable: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::SetConstructor))
}

fn parse_symbol_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Symbol") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Symbol supports zero or one argument".into(),
            ));
        }
        if args.len() == 1 && args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "Symbol description argument cannot be empty".into(),
            ));
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::SymbolConstruct {
            description: if args.is_empty() {
                None
            } else {
                Some(Box::new(parse_expr(args[0].trim())?))
            },
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::SymbolConstruct {
                description: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();

        if cursor.peek() == Some(b'(') {
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            let method = match member.as_str() {
                "for" => {
                    if args.len() != 1 || args[0].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "Symbol.for requires exactly one argument".into(),
                        ));
                    }
                    SymbolStaticMethod::For
                }
                "keyFor" => {
                    if args.len() != 1 || args[0].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "Symbol.keyFor requires exactly one argument".into(),
                        ));
                    }
                    SymbolStaticMethod::KeyFor
                }
                _ => return Ok(None),
            };

            let mut parsed = Vec::with_capacity(args.len());
            for arg in args {
                parsed.push(parse_expr(arg.trim())?);
            }
            cursor.skip_ws();
            if !cursor.eof() {
                return Ok(None);
            }
            return Ok(Some(Expr::SymbolStaticMethod {
                method,
                args: parsed,
            }));
        }

        let property = match member.as_str() {
            "asyncDispose" => SymbolStaticProperty::AsyncDispose,
            "asyncIterator" => SymbolStaticProperty::AsyncIterator,
            "dispose" => SymbolStaticProperty::Dispose,
            "hasInstance" => SymbolStaticProperty::HasInstance,
            "isConcatSpreadable" => SymbolStaticProperty::IsConcatSpreadable,
            "iterator" => SymbolStaticProperty::Iterator,
            "match" => SymbolStaticProperty::Match,
            "matchAll" => SymbolStaticProperty::MatchAll,
            "replace" => SymbolStaticProperty::Replace,
            "search" => SymbolStaticProperty::Search,
            "species" => SymbolStaticProperty::Species,
            "split" => SymbolStaticProperty::Split,
            "toPrimitive" => SymbolStaticProperty::ToPrimitive,
            "toStringTag" => SymbolStaticProperty::ToStringTag,
            "unscopables" => SymbolStaticProperty::Unscopables,
            _ => return Ok(None),
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::SymbolStaticProperty(property)));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::SymbolConstructor))
}

fn parse_array_buffer_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("ArrayBuffer") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "ArrayBuffer supports up to two arguments".into(),
            ));
        }
        if args.len() >= 1 && args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "ArrayBuffer byteLength argument cannot be empty".into(),
            ));
        }
        if args.len() == 2 && args[1].trim().is_empty() {
            return Err(Error::ScriptParse(
                "ArrayBuffer options argument cannot be empty".into(),
            ));
        }
        let byte_length = if let Some(first) = args.first() {
            Some(Box::new(parse_expr(first.trim())?))
        } else {
            None
        };
        let options = if args.len() == 2 {
            Some(Box::new(parse_expr(args[1].trim())?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ArrayBufferConstruct {
            byte_length,
            options,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::ArrayBufferConstruct {
                byte_length: None,
                options: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if member != "isView" {
            return Ok(None);
        }
        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() != 1 || args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "ArrayBuffer.isView requires exactly one argument".into(),
            ));
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ArrayBufferIsView(Box::new(parse_expr(
            args[0].trim(),
        )?))));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::ArrayBufferConstructor))
}

fn parse_new_callee_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let callee_src = cursor.read_balanced_block(b'(', b')')?;
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };
    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Error::ScriptParse(
                "constructor argument cannot be empty".into(),
            ));
        }
        parsed.push(parse_expr(arg)?);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    let callee = parse_expr(callee_src.trim())?;
    Ok(Some(Expr::TypedArrayConstructWithCallee {
        callee: Box::new(callee),
        args: parsed,
        called_with_new: true,
    }))
}

fn parse_array_buffer_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    let property_expr = match member.as_str() {
        "detached" => Some(Expr::ArrayBufferDetached(target.clone())),
        "maxByteLength" => Some(Expr::ArrayBufferMaxByteLength(target.clone())),
        "resizable" => Some(Expr::ArrayBufferResizable(target.clone())),
        _ => None,
    };
    if let Some(expr) = property_expr {
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(expr));
    }

    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    match member.as_str() {
        "resize" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.resize requires exactly one argument".into(),
                ));
            }
            Ok(Some(Expr::ArrayBufferResize {
                target,
                new_byte_length: Box::new(parse_expr(args[0].trim())?),
            }))
        }
        "slice" => {
            if args.len() > 2 {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.slice supports up to two arguments".into(),
                ));
            }
            if args.len() >= 1 && args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.slice start cannot be empty".into(),
                ));
            }
            if args.len() == 2 && args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.slice end cannot be empty".into(),
                ));
            }
            let start = if let Some(first) = args.first() {
                Some(Box::new(parse_expr(first.trim())?))
            } else {
                None
            };
            let end = if args.len() == 2 {
                Some(Box::new(parse_expr(args[1].trim())?))
            } else {
                None
            };
            Ok(Some(Expr::ArrayBufferSlice { target, start, end }))
        }
        "transfer" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.transfer does not take arguments".into(),
                ));
            }
            Ok(Some(Expr::ArrayBufferTransfer {
                target,
                to_fixed_length: false,
            }))
        }
        "transferToFixedLength" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.transferToFixedLength does not take arguments".into(),
                ));
            }
            Ok(Some(Expr::ArrayBufferTransfer {
                target,
                to_fixed_length: true,
            }))
        }
        _ => Ok(None),
    }
}

fn parse_typed_array_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    let property_expr = match member.as_str() {
        "byteLength" => Some(Expr::TypedArrayByteLength(target.clone())),
        "byteOffset" => Some(Expr::TypedArrayByteOffset(target.clone())),
        "buffer" => Some(Expr::TypedArrayBuffer(target.clone())),
        "BYTES_PER_ELEMENT" => Some(Expr::TypedArrayBytesPerElement(target.clone())),
        _ => None,
    };
    if let Some(expr) = property_expr {
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(expr));
    }

    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };
    let method = match member.as_str() {
        "at" => TypedArrayInstanceMethod::At,
        "copyWithin" => TypedArrayInstanceMethod::CopyWithin,
        "entries" => TypedArrayInstanceMethod::Entries,
        "fill" => TypedArrayInstanceMethod::Fill,
        "findIndex" => TypedArrayInstanceMethod::FindIndex,
        "findLast" => TypedArrayInstanceMethod::FindLast,
        "findLastIndex" => TypedArrayInstanceMethod::FindLastIndex,
        "indexOf" => TypedArrayInstanceMethod::IndexOf,
        "keys" => TypedArrayInstanceMethod::Keys,
        "lastIndexOf" => TypedArrayInstanceMethod::LastIndexOf,
        "reduceRight" => TypedArrayInstanceMethod::ReduceRight,
        "reverse" => TypedArrayInstanceMethod::Reverse,
        "set" => TypedArrayInstanceMethod::Set,
        "sort" => TypedArrayInstanceMethod::Sort,
        "subarray" => TypedArrayInstanceMethod::Subarray,
        "toReversed" => TypedArrayInstanceMethod::ToReversed,
        "toSorted" => TypedArrayInstanceMethod::ToSorted,
        "values" => TypedArrayInstanceMethod::Values,
        "with" => TypedArrayInstanceMethod::With,
        _ => return Ok(None),
    };

    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Error::ScriptParse(format!(
                "{} argument cannot be empty",
                member
            )));
        }
        parsed.push(parse_expr(arg)?);
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::TypedArrayMethod {
        target,
        method,
        args: parsed,
    }))
}

fn parse_url_search_params_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if !matches!(member.as_str(), "append" | "getAll" | "has" | "delete") {
        return Ok(None);
    }
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let method = match member.as_str() {
        "append" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "URLSearchParams.append requires exactly two arguments".into(),
                ));
            }
            UrlSearchParamsInstanceMethod::Append
        }
        "getAll" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "URLSearchParams.getAll requires exactly one argument".into(),
                ));
            }
            UrlSearchParamsInstanceMethod::GetAll
        }
        "has" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Ok(None);
            }
            UrlSearchParamsInstanceMethod::Has
        }
        "delete" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Ok(None);
            }
            UrlSearchParamsInstanceMethod::Delete
        }
        _ => unreachable!(),
    };

    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        parsed.push(parse_expr(arg.trim())?);
    }

    Ok(Some(Expr::UrlSearchParamsMethod {
        target,
        method,
        args: parsed,
    }))
}

fn parse_map_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    let method = match member.as_str() {
        "get" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.get requires exactly one argument".into(),
                ));
            }
            MapInstanceMethod::Get
        }
        "has" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.has requires exactly one argument".into(),
                ));
            }
            MapInstanceMethod::Has
        }
        "delete" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.delete requires exactly one argument".into(),
                ));
            }
            MapInstanceMethod::Delete
        }
        "clear" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "Map.clear does not take arguments".into(),
                ));
            }
            MapInstanceMethod::Clear
        }
        "forEach" => {
            if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.forEach requires a callback and optional thisArg".into(),
                ));
            }
            if args.len() == 2 && args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.forEach thisArg cannot be empty".into(),
                ));
            }
            MapInstanceMethod::ForEach
        }
        "getOrInsert" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.getOrInsert requires exactly two arguments".into(),
                ));
            }
            MapInstanceMethod::GetOrInsert
        }
        "getOrInsertComputed" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.getOrInsertComputed requires exactly two arguments".into(),
                ));
            }
            MapInstanceMethod::GetOrInsertComputed
        }
        _ => return Ok(None),
    };

    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Error::ScriptParse(format!(
                "Map.{} argument cannot be empty",
                member
            )));
        }
        parsed.push(parse_expr(arg)?);
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::MapMethod {
        target,
        method,
        args: parsed,
    }))
}

fn parse_set_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    let method = match member.as_str() {
        "add" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.add requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::Add
        }
        "union" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.union requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::Union
        }
        "intersection" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.intersection requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::Intersection
        }
        "difference" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.difference requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::Difference
        }
        "symmetricDifference" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.symmetricDifference requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::SymmetricDifference
        }
        "isDisjointFrom" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.isDisjointFrom requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::IsDisjointFrom
        }
        "isSubsetOf" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.isSubsetOf requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::IsSubsetOf
        }
        "isSupersetOf" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.isSupersetOf requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::IsSupersetOf
        }
        _ => return Ok(None),
    };

    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Error::ScriptParse(format!(
                "Set.{} argument cannot be empty",
                member
            )));
        }
        parsed.push(parse_expr(arg)?);
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::SetMethod {
        target,
        method,
        args: parsed,
    }))
}

fn parse_is_nan_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "isNaN", "isNaN requires exactly one argument")
}

fn parse_encode_uri_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "encodeURI", "encodeURI requires exactly one argument")
}

fn parse_encode_uri_component_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(
        src,
        "encodeURIComponent",
        "encodeURIComponent requires exactly one argument",
    )
}

fn parse_decode_uri_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "decodeURI", "decodeURI requires exactly one argument")
}

fn parse_decode_uri_component_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(
        src,
        "decodeURIComponent",
        "decodeURIComponent requires exactly one argument",
    )
}

fn parse_escape_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "escape", "escape requires exactly one argument")
}

fn parse_unescape_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "unescape", "unescape requires exactly one argument")
}

fn parse_is_finite_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "isFinite", "isFinite requires exactly one argument")
}

fn parse_atob_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "atob", "atob requires exactly one argument")
}

fn parse_btoa_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "btoa", "btoa requires exactly one argument")
}

fn parse_global_single_arg_expr(
    src: &str,
    function_name: &str,
    arg_error: &str,
) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii(function_name) {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(arg_error.into()));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

fn parse_parse_int_expr(src: &str) -> Result<Option<(Expr, Option<Expr>)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("parseInt") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "parseInt requires one or two arguments".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "parseInt radix argument cannot be empty".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    let radix = if args.len() == 2 {
        Some(parse_expr(args[1].trim())?)
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((value, radix)))
}

fn parse_parse_float_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("parseFloat") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "parseFloat requires exactly one argument".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

fn parse_json_parse_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("JSON") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("parse") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "JSON.parse requires exactly one argument".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

fn parse_json_stringify_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("JSON") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("stringify") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "JSON.stringify requires exactly one argument".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

fn parse_object_literal_expr(src: &str) -> Result<Option<Vec<ObjectLiteralEntry>>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if cursor.peek() != Some(b'{') {
        return Ok(None);
    }

    let entries_src = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let mut entries = split_top_level_by_char(&entries_src, b',');
    while entries.len() > 1 && entries.last().is_some_and(|entry| entry.trim().is_empty()) {
        entries.pop();
    }
    if entries.len() == 1 && entries[0].trim().is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() {
            return Err(Error::ScriptParse(
                "object literal does not support empty entries".into(),
            ));
        }

        if let Some(rest) = entry.strip_prefix("...") {
            let rest = rest.trim();
            if rest.is_empty() {
                return Err(Error::ScriptParse(
                    "object spread source cannot be empty".into(),
                ));
            }
            out.push(ObjectLiteralEntry::Spread(parse_expr(rest)?));
            continue;
        }

        let Some(colon) = find_first_top_level_colon(entry) else {
            if is_ident(entry) {
                out.push(ObjectLiteralEntry::Pair(
                    ObjectLiteralKey::Static(entry.to_string()),
                    Expr::Var(entry.to_string()),
                ));
                continue;
            }
            return Err(Error::ScriptParse(
                "object literal entry must use key: value".into(),
            ));
        };

        let key_src = entry[..colon].trim();
        let value_src = entry[colon + 1..].trim();
        if value_src.is_empty() {
            return Err(Error::ScriptParse(
                "object literal value cannot be empty".into(),
            ));
        }

        let key = if key_src.starts_with('[') && key_src.ends_with(']') && key_src.len() >= 2 {
            let computed_src = key_src[1..key_src.len() - 1].trim();
            if computed_src.is_empty() {
                return Err(Error::ScriptParse(
                    "object literal computed key cannot be empty".into(),
                ));
            }
            ObjectLiteralKey::Computed(Box::new(parse_expr(computed_src)?))
        } else if (key_src.starts_with('\'') && key_src.ends_with('\''))
            || (key_src.starts_with('"') && key_src.ends_with('"'))
        {
            ObjectLiteralKey::Static(parse_string_literal_exact(key_src)?)
        } else if is_ident(key_src) {
            ObjectLiteralKey::Static(key_src.to_string())
        } else {
            return Err(Error::ScriptParse(
                "object literal key must be identifier, string literal, or computed key".into(),
            ));
        };

        out.push(ObjectLiteralEntry::Pair(key, parse_expr(value_src)?));
    }

    Ok(Some(out))
}

fn find_first_top_level_colon(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                b':' if paren == 0 && bracket == 0 && brace == 0 => return Some(i),
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }
        i += 1;
    }

    None
}

fn parse_object_static_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Object") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Object supports zero or one argument".into(),
            ));
        }
        if args.len() == 1 && args[0].trim().is_empty() {
            return Err(Error::ScriptParse("Object argument cannot be empty".into()));
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ObjectConstruct {
            value: if args.is_empty() {
                None
            } else {
                Some(Box::new(parse_expr(args[0].trim())?))
            },
        }));
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    let expr = match method.as_str() {
        "getOwnPropertySymbols" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.getOwnPropertySymbols requires exactly one argument".into(),
                ));
            }
            Expr::ObjectGetOwnPropertySymbols(Box::new(parse_expr(args[0].trim())?))
        }
        "keys" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.keys requires exactly one argument".into(),
                ));
            }
            Expr::ObjectKeys(Box::new(parse_expr(args[0].trim())?))
        }
        "values" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.values requires exactly one argument".into(),
                ));
            }
            Expr::ObjectValues(Box::new(parse_expr(args[0].trim())?))
        }
        "entries" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.entries requires exactly one argument".into(),
                ));
            }
            Expr::ObjectEntries(Box::new(parse_expr(args[0].trim())?))
        }
        "hasOwn" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.hasOwn requires exactly two arguments".into(),
                ));
            }
            Expr::ObjectHasOwn {
                object: Box::new(parse_expr(args[0].trim())?),
                key: Box::new(parse_expr(args[1].trim())?),
            }
        }
        "getPrototypeOf" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.getPrototypeOf requires exactly one argument".into(),
                ));
            }
            Expr::ObjectGetPrototypeOf(Box::new(parse_expr(args[0].trim())?))
        }
        "freeze" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.freeze requires exactly one argument".into(),
                ));
            }
            Expr::ObjectFreeze(Box::new(parse_expr(args[0].trim())?))
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}

fn parse_object_has_own_property_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("hasOwnProperty") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "hasOwnProperty requires exactly one argument".into(),
        ));
    }
    let key = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::ObjectHasOwnProperty {
        target,
        key: Box::new(key),
    }))
}

fn parse_object_get_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    let mut path = Vec::new();
    loop {
        cursor.skip_ws();
        let Some(key) = cursor.parse_identifier() else {
            return Ok(None);
        };
        path.push(key);
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            break;
        }
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    if path.len() == 1 {
        return Ok(Some(Expr::ObjectGet {
            target,
            key: path.remove(0),
        }));
    }
    Ok(Some(Expr::ObjectPathGet { target, path }))
}

fn parse_call_args<'a>(args_src: &'a str, empty_err: &'static str) -> Result<Vec<&'a str>> {
    if args_src.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut args = split_top_level_by_char(args_src, b',');
    if args.len() > 1 && args.last().is_some_and(|arg| arg.trim().is_empty()) {
        args.pop();
    }
    if args.iter().any(|arg| arg.trim().is_empty()) {
        return Err(Error::ScriptParse(empty_err.into()));
    }
    Ok(args)
}

fn parse_member_call_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let dots = collect_top_level_char_positions(src, b'.');
    for dot in dots.into_iter().rev() {
        let Some(base_src) = src.get(..dot) else {
            continue;
        };
        let base_src = base_src.trim();
        if base_src.is_empty() {
            continue;
        }

        let Some(tail_src) = src.get(dot + 1..) else {
            continue;
        };
        let tail_src = tail_src.trim();
        let mut cursor = Cursor::new(tail_src);
        let Some(member) = cursor.parse_identifier() else {
            continue;
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            continue;
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        cursor.skip_ws();
        if !cursor.eof() {
            continue;
        }

        let args = parse_call_args(&args_src, "member call arguments cannot be empty")?;
        let mut parsed_args = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            parsed_args.push(parse_expr(arg)?);
        }

        return Ok(Some(Expr::MemberCall {
            target: Box::new(parse_expr(base_src)?),
            member,
            args: parsed_args,
        }));
    }
    Ok(None)
}

fn parse_function_call_expr(src: &str) -> Result<Option<Expr>> {
    let parse_args = |args_src: &str| -> Result<Vec<Expr>> {
        let args = parse_call_args(args_src, "function call arguments cannot be empty")?;

        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            parsed.push(parse_expr(arg)?);
        }
        Ok(parsed)
    };

    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if let Some(target) = cursor.parse_identifier() {
        cursor.skip_ws();
        if cursor.peek() == Some(b'(') {
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            let parsed = parse_args(&args_src)?;

            cursor.skip_ws();
            if cursor.eof() {
                return Ok(Some(Expr::FunctionCall {
                    target,
                    args: parsed,
                }));
            }
            return Ok(None);
        }
    }

    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let target_src = cursor.read_balanced_block(b'(', b')')?;
    let target_src = target_src.trim();
    if target_src.is_empty() {
        return Ok(None);
    }

    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = parse_args(&args_src)?;

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::Call {
        target: Box::new(parse_expr(target_src)?),
        args,
    }))
}

fn parse_fetch_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "fetch", "fetch requires exactly one argument")
}

fn parse_match_media_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("matchMedia") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "matchMedia requires exactly one argument".into(),
        ));
    }
    let query = parse_expr(args[0].trim())?;

    cursor.skip_ws();
    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(prop_name) = cursor.parse_identifier() else {
            return Ok(None);
        };
        let prop = match prop_name.as_str() {
            "matches" => MatchMediaProp::Matches,
            "media" => MatchMediaProp::Media,
            _ => return Ok(None),
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MatchMediaProp {
            query: Box::new(query),
            prop,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::MatchMedia(Box::new(query))))
}

fn parse_structured_clone_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(
        src,
        "structuredClone",
        "structuredClone requires exactly one argument",
    )
}

fn parse_alert_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "alert", "alert requires exactly one argument")
}

fn parse_confirm_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "confirm", "confirm requires exactly one argument")
}

fn parse_prompt_expr(src: &str) -> Result<Option<(Expr, Option<Expr>)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("prompt") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "prompt requires one or two arguments".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "prompt default argument cannot be empty".into(),
        ));
    }

    let message = parse_expr(args[0].trim())?;
    let default = if args.len() == 2 {
        Some(parse_expr(args[1].trim())?)
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((message, default)))
}

fn parse_array_literal_expr(src: &str) -> Result<Option<Vec<Expr>>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if cursor.peek() != Some(b'[') {
        return Ok(None);
    }

    let items_src = cursor.read_balanced_block(b'[', b']')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let mut items = split_top_level_by_char(&items_src, b',');
    while items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty()) {
        items.pop();
    }
    if items.len() == 1 && items[0].trim().is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let item = item.trim();
        if item.is_empty() {
            return Err(Error::ScriptParse(
                "array literal does not support empty elements".into(),
            ));
        }
        if let Some(rest) = item.strip_prefix("...") {
            let rest = rest.trim();
            if rest.is_empty() {
                return Err(Error::ScriptParse(
                    "array spread source cannot be empty".into(),
                ));
            }
            out.push(Expr::Spread(Box::new(parse_expr(rest)?)));
        } else {
            out.push(parse_expr(item)?);
        }
    }
    Ok(Some(out))
}

fn parse_array_is_array_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Array") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("isArray") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Array.isArray requires exactly one argument".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

fn parse_array_from_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Array") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("from") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Array.from requires one or two arguments".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Array.from map function cannot be empty".into(),
        ));
    }

    let source = parse_expr(args[0].trim())?;
    let map_fn = if args.len() == 2 {
        Some(Box::new(parse_expr(args[1].trim())?))
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::ArrayFrom {
        source: Box::new(source),
        map_fn,
    }))
}

fn parse_array_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if cursor.peek() == Some(b'[') {
        let index_src = cursor.read_balanced_block(b'[', b']')?;
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        if index_src.trim().is_empty() {
            return Err(Error::ScriptParse("array index cannot be empty".into()));
        }
        let index = parse_expr(index_src.trim())?;
        return Ok(Some(Expr::ArrayIndex {
            target,
            index: Box::new(index),
        }));
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if method == "length" {
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ArrayLength(target)));
    }

    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = parse_call_args(&args_src, "array method arguments cannot be empty")?;

    let expr = match method.as_str() {
        "push" => {
            let mut parsed = Vec::with_capacity(args.len());
            for arg in args {
                if arg.trim().is_empty() {
                    return Err(Error::ScriptParse("push argument cannot be empty".into()));
                }
                parsed.push(parse_expr(arg.trim())?);
            }
            Expr::ArrayPush {
                target,
                args: parsed,
            }
        }
        "pop" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("pop does not take arguments".into()));
            }
            Expr::ArrayPop(target)
        }
        "shift" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("shift does not take arguments".into()));
            }
            Expr::ArrayShift(target)
        }
        "unshift" => {
            let mut parsed = Vec::with_capacity(args.len());
            for arg in args {
                if arg.trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "unshift argument cannot be empty".into(),
                    ));
                }
                parsed.push(parse_expr(arg.trim())?);
            }
            Expr::ArrayUnshift {
                target,
                args: parsed,
            }
        }
        "map" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "map requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArrayMap { target, callback }
        }
        "filter" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "filter requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArrayFilter { target, callback }
        }
        "reduce" => {
            if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "reduce requires callback and optional initial value".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 4, "array callback parameters")?;
            let initial = if args.len() == 2 {
                if args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "reduce initial value cannot be empty".into(),
                    ));
                }
                Some(Box::new(parse_expr(args[1].trim())?))
            } else {
                None
            };
            Expr::ArrayReduce {
                target,
                callback,
                initial,
            }
        }
        "forEach" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "forEach requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArrayForEach { target, callback }
        }
        "find" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "find requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArrayFind { target, callback }
        }
        "some" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "some requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArraySome { target, callback }
        }
        "every" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "every requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArrayEvery { target, callback }
        }
        "includes" => {
            if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "includes requires one or two arguments".into(),
                ));
            }
            if args.len() == 2 && args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "includes fromIndex cannot be empty".into(),
                ));
            }
            Expr::ArrayIncludes {
                target,
                search: Box::new(parse_expr(args[0].trim())?),
                from_index: if args.len() == 2 {
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                },
            }
        }
        "slice" => {
            if args.len() > 2 {
                return Err(Error::ScriptParse(
                    "slice supports up to two arguments".into(),
                ));
            }
            let start = if !args.is_empty() {
                if args[0].trim().is_empty() {
                    return Err(Error::ScriptParse("slice start cannot be empty".into()));
                }
                Some(Box::new(parse_expr(args[0].trim())?))
            } else {
                None
            };
            let end = if args.len() == 2 {
                if args[1].trim().is_empty() {
                    return Err(Error::ScriptParse("slice end cannot be empty".into()));
                }
                Some(Box::new(parse_expr(args[1].trim())?))
            } else {
                None
            };
            Expr::ArraySlice { target, start, end }
        }
        "splice" => {
            if args.is_empty() || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "splice requires at least start index".into(),
                ));
            }
            let start = Box::new(parse_expr(args[0].trim())?);
            let delete_count = if args.len() >= 2 {
                if args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "splice deleteCount cannot be empty".into(),
                    ));
                }
                Some(Box::new(parse_expr(args[1].trim())?))
            } else {
                None
            };
            let mut items = Vec::new();
            for arg in args.iter().skip(2) {
                if arg.trim().is_empty() {
                    return Err(Error::ScriptParse("splice item cannot be empty".into()));
                }
                items.push(parse_expr(arg.trim())?);
            }
            Expr::ArraySplice {
                target,
                start,
                delete_count,
                items,
            }
        }
        "join" => {
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "join supports at most one argument".into(),
                ));
            }
            let separator = if args.len() == 1 {
                if args[0].trim().is_empty() {
                    return Err(Error::ScriptParse("join separator cannot be empty".into()));
                }
                Some(Box::new(parse_expr(args[0].trim())?))
            } else {
                None
            };
            Expr::ArrayJoin { target, separator }
        }
        "sort" => {
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "sort supports at most one argument".into(),
                ));
            }
            if args.len() == 1 && args[0].trim().is_empty() {
                return Err(Error::ScriptParse("sort comparator cannot be empty".into()));
            }
            Expr::ArraySort {
                target,
                comparator: if args.len() == 1 {
                    Some(Box::new(parse_expr(args[0].trim())?))
                } else {
                    None
                },
            }
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}

fn parse_array_callback_arg(arg: &str, max_params: usize, label: &str) -> Result<ScriptHandler> {
    let callback_arg = strip_js_comments(arg);
    let mut callback_cursor = Cursor::new(callback_arg.as_str().trim());
    let (params, body, concise_body) = parse_callback(&mut callback_cursor, max_params, label)?;
    callback_cursor.skip_ws();
    if !callback_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported array callback: {}",
            arg.trim()
        )));
    }

    let stmts = if concise_body {
        vec![Stmt::Return {
            value: Some(parse_expr(body.trim())?),
        }]
    } else {
        parse_block_statements(&body)?
    };

    Ok(ScriptHandler { params, stmts })
}

fn parse_number_method_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let dots = collect_top_level_char_positions(src, b'.');
    for dot in dots.into_iter().rev() {
        let Some(base_src) = src.get(..dot) else {
            continue;
        };
        let base_src = base_src.trim();
        if base_src.is_empty() {
            continue;
        }
        let Some(tail_src) = src.get(dot + 1..) else {
            continue;
        };
        let tail_src = tail_src.trim();

        let mut cursor = Cursor::new(tail_src);
        let Some(method) = cursor.parse_identifier() else {
            continue;
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            continue;
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        cursor.skip_ws();
        if !cursor.eof() {
            continue;
        }

        let Some(method) = parse_number_instance_method_name(&method) else {
            continue;
        };

        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let parsed = match method {
            NumberInstanceMethod::ToLocaleString | NumberInstanceMethod::ValueOf => {
                if !args.is_empty() {
                    let method_name = match method {
                        NumberInstanceMethod::ToLocaleString => "toLocaleString",
                        NumberInstanceMethod::ValueOf => "valueOf",
                        _ => unreachable!(),
                    };
                    return Err(Error::ScriptParse(format!(
                        "{method_name} does not take arguments"
                    )));
                }
                Vec::new()
            }
            NumberInstanceMethod::ToExponential
            | NumberInstanceMethod::ToFixed
            | NumberInstanceMethod::ToPrecision
            | NumberInstanceMethod::ToString => {
                if args.len() > 1 {
                    let method_name = match method {
                        NumberInstanceMethod::ToExponential => "toExponential",
                        NumberInstanceMethod::ToFixed => "toFixed",
                        NumberInstanceMethod::ToPrecision => "toPrecision",
                        NumberInstanceMethod::ToString => "toString",
                        _ => unreachable!(),
                    };
                    return Err(Error::ScriptParse(format!(
                        "{method_name} supports at most one argument"
                    )));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    let method_name = match method {
                        NumberInstanceMethod::ToExponential => "toExponential",
                        NumberInstanceMethod::ToFixed => "toFixed",
                        NumberInstanceMethod::ToPrecision => "toPrecision",
                        NumberInstanceMethod::ToString => "toString",
                        _ => unreachable!(),
                    };
                    return Err(Error::ScriptParse(format!(
                        "{method_name} argument cannot be empty"
                    )));
                }
                if args.len() == 1 {
                    vec![parse_expr(args[0].trim())?]
                } else {
                    Vec::new()
                }
            }
        };

        return Ok(Some(Expr::NumberInstanceMethod {
            value: Box::new(parse_expr(base_src)?),
            method,
            args: parsed,
        }));
    }

    Ok(None)
}

fn parse_number_instance_method_name(name: &str) -> Option<NumberInstanceMethod> {
    match name {
        "toExponential" => Some(NumberInstanceMethod::ToExponential),
        "toFixed" => Some(NumberInstanceMethod::ToFixed),
        "toLocaleString" => Some(NumberInstanceMethod::ToLocaleString),
        "toPrecision" => Some(NumberInstanceMethod::ToPrecision),
        "toString" => Some(NumberInstanceMethod::ToString),
        "valueOf" => Some(NumberInstanceMethod::ValueOf),
        _ => None,
    }
}

fn parse_bigint_method_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let dots = collect_top_level_char_positions(src, b'.');
    for dot in dots.into_iter().rev() {
        let Some(base_src) = src.get(..dot) else {
            continue;
        };
        let base_src = base_src.trim();
        if base_src.is_empty() {
            continue;
        }
        let Some(tail_src) = src.get(dot + 1..) else {
            continue;
        };
        let tail_src = tail_src.trim();

        let mut cursor = Cursor::new(tail_src);
        let Some(method_name) = cursor.parse_identifier() else {
            continue;
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            continue;
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        cursor.skip_ws();
        if !cursor.eof() {
            continue;
        }

        let Some(method) = parse_bigint_instance_method_name(&method_name) else {
            continue;
        };

        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let parsed = match method {
            BigIntInstanceMethod::ToLocaleString | BigIntInstanceMethod::ValueOf => {
                if !args.is_empty() {
                    let method_name = match method {
                        BigIntInstanceMethod::ToLocaleString => "toLocaleString",
                        BigIntInstanceMethod::ValueOf => "valueOf",
                        _ => unreachable!(),
                    };
                    return Err(Error::ScriptParse(format!(
                        "{method_name} does not take arguments"
                    )));
                }
                Vec::new()
            }
            BigIntInstanceMethod::ToString => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "toString supports at most one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "toString argument cannot be empty".into(),
                    ));
                }
                if args.len() == 1 {
                    vec![parse_expr(args[0].trim())?]
                } else {
                    Vec::new()
                }
            }
        };

        return Ok(Some(Expr::BigIntInstanceMethod {
            value: Box::new(parse_expr(base_src)?),
            method,
            args: parsed,
        }));
    }

    Ok(None)
}

fn parse_bigint_instance_method_name(name: &str) -> Option<BigIntInstanceMethod> {
    match name {
        "toLocaleString" => Some(BigIntInstanceMethod::ToLocaleString),
        "toString" => Some(BigIntInstanceMethod::ToString),
        "valueOf" => Some(BigIntInstanceMethod::ValueOf),
        _ => None,
    }
}

fn parse_intl_format_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let dots = collect_top_level_char_positions(src, b'.');
    for dot in dots.into_iter().rev() {
        let Some(base_src) = src.get(..dot) else {
            continue;
        };
        let base_src = base_src.trim();
        if base_src.is_empty() {
            continue;
        }
        let Some(tail_src) = src.get(dot + 1..) else {
            continue;
        };
        let tail_src = tail_src.trim();

        let mut cursor = Cursor::new(tail_src);
        let Some(method_name) = cursor.parse_identifier() else {
            continue;
        };

        if method_name == "compare" {
            cursor.skip_ws();
            if cursor.eof() {
                return Ok(Some(Expr::IntlCollatorCompareGetter {
                    collator: Box::new(parse_expr(base_src)?),
                }));
            }
            if cursor.peek() != Some(b'(') {
                continue;
            }
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            cursor.skip_ws();
            if !cursor.eof() {
                continue;
            }
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.Collator.compare requires exactly two arguments".into(),
                ));
            }
            return Ok(Some(Expr::IntlCollatorCompare {
                collator: Box::new(parse_expr(base_src)?),
                left: Box::new(parse_expr(args[0].trim())?),
                right: Box::new(parse_expr(args[1].trim())?),
            }));
        }

        if method_name == "formatRangeToParts" {
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                continue;
            }
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            cursor.skip_ws();
            if !cursor.eof() {
                continue;
            }
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.DateTimeFormat.formatRangeToParts requires exactly two arguments".into(),
                ));
            }
            return Ok(Some(Expr::IntlDateTimeFormatRangeToParts {
                formatter: Box::new(parse_expr(base_src)?),
                start: Box::new(parse_expr(args[0].trim())?),
                end: Box::new(parse_expr(args[1].trim())?),
            }));
        }

        if method_name == "formatRange" {
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                continue;
            }
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            cursor.skip_ws();
            if !cursor.eof() {
                continue;
            }
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.DateTimeFormat.formatRange requires exactly two arguments".into(),
                ));
            }
            return Ok(Some(Expr::IntlDateTimeFormatRange {
                formatter: Box::new(parse_expr(base_src)?),
                start: Box::new(parse_expr(args[0].trim())?),
                end: Box::new(parse_expr(args[1].trim())?),
            }));
        }

        if method_name == "formatToParts" {
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                continue;
            }
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            cursor.skip_ws();
            if !cursor.eof() {
                continue;
            }
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            if args.len() == 2 && !args[0].trim().is_empty() && !args[1].trim().is_empty() {
                return Ok(Some(Expr::IntlRelativeTimeFormatToParts {
                    formatter: Box::new(parse_expr(base_src)?),
                    value: Box::new(parse_expr(args[0].trim())?),
                    unit: Box::new(parse_expr(args[1].trim())?),
                }));
            }
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "Intl.DateTimeFormat.formatToParts supports at most one argument".into(),
                ));
            }
            if args.len() == 1 && args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.DateTimeFormat.formatToParts argument cannot be empty".into(),
                ));
            }
            return Ok(Some(Expr::IntlDateTimeFormatToParts {
                formatter: Box::new(parse_expr(base_src)?),
                value: args
                    .first()
                    .map(|arg| parse_expr(arg.trim()))
                    .transpose()?
                    .map(Box::new),
            }));
        }

        if method_name == "of" {
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                continue;
            }
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            cursor.skip_ws();
            if !cursor.eof() {
                continue;
            }
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.DisplayNames.of requires exactly one argument".into(),
                ));
            }
            return Ok(Some(Expr::IntlDisplayNamesOf {
                display_names: Box::new(parse_expr(base_src)?),
                code: Box::new(parse_expr(args[0].trim())?),
            }));
        }

        if method_name == "select" {
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                continue;
            }
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            cursor.skip_ws();
            if !cursor.eof() {
                continue;
            }
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.PluralRules.select requires exactly one argument".into(),
                ));
            }
            return Ok(Some(Expr::IntlPluralRulesSelect {
                plural_rules: Box::new(parse_expr(base_src)?),
                value: Box::new(parse_expr(args[0].trim())?),
            }));
        }

        if method_name == "selectRange" {
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                continue;
            }
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            cursor.skip_ws();
            if !cursor.eof() {
                continue;
            }
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.PluralRules.selectRange requires exactly two arguments".into(),
                ));
            }
            return Ok(Some(Expr::IntlPluralRulesSelectRange {
                plural_rules: Box::new(parse_expr(base_src)?),
                start: Box::new(parse_expr(args[0].trim())?),
                end: Box::new(parse_expr(args[1].trim())?),
            }));
        }

        if method_name == "segment" {
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                continue;
            }
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            cursor.skip_ws();
            if !cursor.eof() {
                continue;
            }
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.Segmenter.segment requires exactly one argument".into(),
                ));
            }
            return Ok(Some(Expr::IntlSegmenterSegment {
                segmenter: Box::new(parse_expr(base_src)?),
                value: Box::new(parse_expr(args[0].trim())?),
            }));
        }

        let intl_locale_method = match method_name.as_str() {
            "getCalendars" => Some(IntlLocaleMethod::GetCalendars),
            "getCollations" => Some(IntlLocaleMethod::GetCollations),
            "getHourCycles" => Some(IntlLocaleMethod::GetHourCycles),
            "getNumberingSystems" => Some(IntlLocaleMethod::GetNumberingSystems),
            "getTextInfo" => Some(IntlLocaleMethod::GetTextInfo),
            "getTimeZones" => Some(IntlLocaleMethod::GetTimeZones),
            "getWeekInfo" => Some(IntlLocaleMethod::GetWeekInfo),
            "maximize" => Some(IntlLocaleMethod::Maximize),
            "minimize" => Some(IntlLocaleMethod::Minimize),
            "toString" => Some(IntlLocaleMethod::ToString),
            _ => None,
        };
        if let Some(method) = intl_locale_method {
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                continue;
            }
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            cursor.skip_ws();
            if !cursor.eof() {
                continue;
            }
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            if !args.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "Intl.Locale.{method_name} does not take arguments"
                )));
            }
            return Ok(Some(Expr::IntlLocaleMethod {
                locale: Box::new(parse_expr(base_src)?),
                method,
            }));
        }

        if method_name == "resolvedOptions" {
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                continue;
            }
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            cursor.skip_ws();
            if !cursor.eof() {
                continue;
            }
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "Intl formatter resolvedOptions does not take arguments".into(),
                ));
            }
            return Ok(Some(Expr::IntlDateTimeResolvedOptions {
                formatter: Box::new(parse_expr(base_src)?),
            }));
        }

        if method_name == "format" {
            cursor.skip_ws();
            if cursor.eof() {
                return Ok(Some(Expr::IntlFormatGetter {
                    formatter: Box::new(parse_expr(base_src)?),
                }));
            }
            if cursor.peek() != Some(b'(') {
                continue;
            }
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            cursor.skip_ws();
            if !cursor.eof() {
                continue;
            }

            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            if args.len() == 2 && !args[0].trim().is_empty() && !args[1].trim().is_empty() {
                return Ok(Some(Expr::IntlRelativeTimeFormat {
                    formatter: Box::new(parse_expr(base_src)?),
                    value: Box::new(parse_expr(args[0].trim())?),
                    unit: Box::new(parse_expr(args[1].trim())?),
                }));
            }
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "Intl formatter format supports at most one argument".into(),
                ));
            }
            if args.len() == 1 && args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl formatter format argument cannot be empty".into(),
                ));
            }

            return Ok(Some(Expr::IntlFormat {
                formatter: Box::new(parse_expr(base_src)?),
                value: args
                    .first()
                    .map(|arg| parse_expr(arg.trim()))
                    .transpose()?
                    .map(Box::new),
            }));
        }
    }

    Ok(None)
}

fn parse_string_method_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let dots = collect_top_level_char_positions(src, b'.');
    for dot in dots.into_iter().rev() {
        let Some(base_src) = src.get(..dot) else {
            continue;
        };
        let base_src = base_src.trim();
        if base_src.is_empty() {
            continue;
        }
        let Some(tail_src) = src.get(dot + 1..) else {
            continue;
        };
        let tail_src = tail_src.trim();

        let mut cursor = Cursor::new(tail_src);
        let Some(method) = cursor.parse_identifier() else {
            continue;
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            continue;
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        cursor.skip_ws();
        if !cursor.eof() {
            continue;
        }

        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        if !matches!(
            method.as_str(),
            "charAt"
                | "charCodeAt"
                | "codePointAt"
                | "at"
                | "concat"
                | "trim"
                | "trimStart"
                | "trimEnd"
                | "toUpperCase"
                | "toLocaleUpperCase"
                | "toLowerCase"
                | "toLocaleLowerCase"
                | "includes"
                | "startsWith"
                | "endsWith"
                | "slice"
                | "substring"
                | "match"
                | "split"
                | "replace"
                | "replaceAll"
                | "indexOf"
                | "lastIndexOf"
                | "search"
                | "repeat"
                | "padStart"
                | "padEnd"
                | "localeCompare"
                | "isWellFormed"
                | "toWellFormed"
                | "valueOf"
                | "toString"
        ) {
            continue;
        }

        if (method == "toString" || method == "valueOf") && !args.is_empty() {
            continue;
        }

        let base_expr = if let Some(target) = parse_element_ref_expr(base_src)? {
            Expr::DomRef(target)
        } else {
            parse_expr(base_src)?
        };
        let base = Box::new(base_expr);
        let expr = match method.as_str() {
            "charAt" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "charAt supports zero or one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse("charAt index cannot be empty".into()));
                }
                Expr::StringCharAt {
                    value: base,
                    index: args
                        .first()
                        .map(|arg| parse_expr(arg.trim()))
                        .transpose()?
                        .map(Box::new),
                }
            }
            "charCodeAt" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "charCodeAt supports zero or one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "charCodeAt index cannot be empty".into(),
                    ));
                }
                Expr::StringCharCodeAt {
                    value: base,
                    index: args
                        .first()
                        .map(|arg| parse_expr(arg.trim()))
                        .transpose()?
                        .map(Box::new),
                }
            }
            "codePointAt" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "codePointAt supports zero or one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "codePointAt index cannot be empty".into(),
                    ));
                }
                Expr::StringCodePointAt {
                    value: base,
                    index: args
                        .first()
                        .map(|arg| parse_expr(arg.trim()))
                        .transpose()?
                        .map(Box::new),
                }
            }
            "at" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "at supports zero or one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse("at index cannot be empty".into()));
                }
                Expr::StringAt {
                    value: base,
                    index: args
                        .first()
                        .map(|arg| parse_expr(arg.trim()))
                        .transpose()?
                        .map(Box::new),
                }
            }
            "concat" => {
                let mut parsed = Vec::with_capacity(args.len());
                for arg in args {
                    if arg.trim().is_empty() {
                        return Err(Error::ScriptParse("concat argument cannot be empty".into()));
                    }
                    parsed.push(parse_expr(arg.trim())?);
                }
                Expr::StringConcat {
                    value: base,
                    args: parsed,
                }
            }
            "trim" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse("trim does not take arguments".into()));
                }
                Expr::StringTrim {
                    value: base,
                    mode: StringTrimMode::Both,
                }
            }
            "trimStart" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "trimStart does not take arguments".into(),
                    ));
                }
                Expr::StringTrim {
                    value: base,
                    mode: StringTrimMode::Start,
                }
            }
            "trimEnd" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse("trimEnd does not take arguments".into()));
                }
                Expr::StringTrim {
                    value: base,
                    mode: StringTrimMode::End,
                }
            }
            "toUpperCase" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "toUpperCase does not take arguments".into(),
                    ));
                }
                Expr::StringToUpperCase(base)
            }
            "toLocaleUpperCase" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "toLocaleUpperCase supports up to one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "toLocaleUpperCase locale cannot be empty".into(),
                    ));
                }
                Expr::StringToUpperCase(base)
            }
            "toLowerCase" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "toLowerCase does not take arguments".into(),
                    ));
                }
                Expr::StringToLowerCase(base)
            }
            "toLocaleLowerCase" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "toLocaleLowerCase supports up to one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "toLocaleLowerCase locale cannot be empty".into(),
                    ));
                }
                Expr::StringToLowerCase(base)
            }
            "includes" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "String.includes requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "String.includes position cannot be empty".into(),
                    ));
                }
                Expr::StringIncludes {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    position: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "startsWith" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "startsWith requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "startsWith position cannot be empty".into(),
                    ));
                }
                Expr::StringStartsWith {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    position: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "endsWith" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "endsWith requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "endsWith length argument cannot be empty".into(),
                    ));
                }
                Expr::StringEndsWith {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    length: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "slice" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "String.slice supports up to two arguments".into(),
                    ));
                }
                let start = if !args.is_empty() {
                    if args[0].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "String.slice start cannot be empty".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[0].trim())?))
                } else {
                    None
                };
                let end = if args.len() == 2 {
                    if args[1].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "String.slice end cannot be empty".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                };
                Expr::StringSlice {
                    value: base,
                    start,
                    end,
                }
            }
            "substring" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "substring supports up to two arguments".into(),
                    ));
                }
                let start = if !args.is_empty() {
                    if args[0].trim().is_empty() {
                        return Err(Error::ScriptParse("substring start cannot be empty".into()));
                    }
                    Some(Box::new(parse_expr(args[0].trim())?))
                } else {
                    None
                };
                let end = if args.len() == 2 {
                    if args[1].trim().is_empty() {
                        return Err(Error::ScriptParse("substring end cannot be empty".into()));
                    }
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                };
                Expr::StringSubstring {
                    value: base,
                    start,
                    end,
                }
            }
            "match" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "match requires exactly one argument".into(),
                    ));
                }
                Expr::StringMatch {
                    value: base,
                    pattern: Box::new(parse_expr(args[0].trim())?),
                }
            }
            "split" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "split supports up to two arguments".into(),
                    ));
                }
                let separator = if !args.is_empty() {
                    if args[0].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "split separator cannot be empty expression".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[0].trim())?))
                } else {
                    None
                };
                let limit = if args.len() == 2 {
                    if args[1].trim().is_empty() {
                        return Err(Error::ScriptParse("split limit cannot be empty".into()));
                    }
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                };
                Expr::StringSplit {
                    value: base,
                    separator,
                    limit,
                }
            }
            "replace" => {
                if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "replace requires exactly two arguments".into(),
                    ));
                }
                Expr::StringReplace {
                    value: base,
                    from: Box::new(parse_expr(args[0].trim())?),
                    to: Box::new(parse_expr(args[1].trim())?),
                }
            }
            "replaceAll" => {
                if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "replaceAll requires exactly two arguments".into(),
                    ));
                }
                Expr::StringReplaceAll {
                    value: base,
                    from: Box::new(parse_expr(args[0].trim())?),
                    to: Box::new(parse_expr(args[1].trim())?),
                }
            }
            "indexOf" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "indexOf requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "indexOf position cannot be empty".into(),
                    ));
                }
                Expr::StringIndexOf {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    position: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "lastIndexOf" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "lastIndexOf requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "lastIndexOf position cannot be empty".into(),
                    ));
                }
                Expr::StringLastIndexOf {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    position: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "search" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "search requires exactly one argument".into(),
                    ));
                }
                Expr::StringSearch {
                    value: base,
                    pattern: Box::new(parse_expr(args[0].trim())?),
                }
            }
            "repeat" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "repeat requires exactly one argument".into(),
                    ));
                }
                Expr::StringRepeat {
                    value: base,
                    count: Box::new(parse_expr(args[0].trim())?),
                }
            }
            "padStart" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "padStart requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "padStart pad string cannot be empty expression".into(),
                    ));
                }
                Expr::StringPadStart {
                    value: base,
                    target_length: Box::new(parse_expr(args[0].trim())?),
                    pad: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "padEnd" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "padEnd requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "padEnd pad string cannot be empty expression".into(),
                    ));
                }
                Expr::StringPadEnd {
                    value: base,
                    target_length: Box::new(parse_expr(args[0].trim())?),
                    pad: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "localeCompare" => {
                if args.is_empty() || args.len() > 3 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "localeCompare requires one to three arguments".into(),
                    ));
                }
                if args.len() >= 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "localeCompare locales argument cannot be empty".into(),
                    ));
                }
                if args.len() == 3 && args[2].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "localeCompare options argument cannot be empty".into(),
                    ));
                }
                Expr::StringLocaleCompare {
                    value: base,
                    compare: Box::new(parse_expr(args[0].trim())?),
                    locales: if args.len() >= 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                    options: if args.len() == 3 {
                        Some(Box::new(parse_expr(args[2].trim())?))
                    } else {
                        None
                    },
                }
            }
            "isWellFormed" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "isWellFormed does not take arguments".into(),
                    ));
                }
                Expr::StringIsWellFormed(base)
            }
            "toWellFormed" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "toWellFormed does not take arguments".into(),
                    ));
                }
                Expr::StringToWellFormed(base)
            }
            "valueOf" => Expr::StringValueOf(base),
            "toString" => Expr::StringToString(base),
            _ => unreachable!(),
        };

        return Ok(Some(expr));
    }

    Ok(None)
}

fn parse_date_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    let expr = match method.as_str() {
        "getTime" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("getTime does not take arguments".into()));
            }
            Expr::DateGetTime(target)
        }
        "setTime" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "setTime requires exactly one argument".into(),
                ));
            }
            Expr::DateSetTime {
                target,
                value: Box::new(parse_expr(args[0].trim())?),
            }
        }
        "toISOString" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "toISOString does not take arguments".into(),
                ));
            }
            Expr::DateToIsoString(target)
        }
        "getFullYear" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getFullYear does not take arguments".into(),
                ));
            }
            Expr::DateGetFullYear(target)
        }
        "getMonth" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getMonth does not take arguments".into(),
                ));
            }
            Expr::DateGetMonth(target)
        }
        "getDate" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("getDate does not take arguments".into()));
            }
            Expr::DateGetDate(target)
        }
        "getHours" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getHours does not take arguments".into(),
                ));
            }
            Expr::DateGetHours(target)
        }
        "getMinutes" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getMinutes does not take arguments".into(),
                ));
            }
            Expr::DateGetMinutes(target)
        }
        "getSeconds" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getSeconds does not take arguments".into(),
                ));
            }
            Expr::DateGetSeconds(target)
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}

fn collect_top_level_char_positions(src: &str, target: u8) -> Vec<usize> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                _ => {
                    if b == target && paren == 0 && bracket == 0 && brace == 0 {
                        out.push(i);
                    }
                }
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }
        i += 1;
    }

    out
}

fn parse_set_timeout_expr(src: &str) -> Result<Option<(TimerInvocation, Expr)>> {
    let mut cursor = Cursor::new(src);
    let Some((handler, delay_ms)) = parse_set_timeout_call(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((handler, delay_ms)))
}

fn parse_set_interval_expr(src: &str) -> Result<Option<(TimerInvocation, Expr)>> {
    let mut cursor = Cursor::new(src);
    let Some((handler, delay_ms)) = parse_set_interval_call(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((handler, delay_ms)))
}

fn parse_request_animation_frame_expr(src: &str) -> Result<Option<TimerCallback>> {
    let mut cursor = Cursor::new(src);
    let callback = parse_request_animation_frame_call(&mut cursor)?;
    cursor.skip_ws();
    if cursor.eof() { Ok(callback) } else { Ok(None) }
}

fn parse_request_animation_frame_call(cursor: &mut Cursor<'_>) -> Result<Option<TimerCallback>> {
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("requestAnimationFrame") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "requestAnimationFrame requires exactly one argument".into(),
        ));
    }

    let callback_arg = strip_js_comments(args[0]);
    let callback = parse_timer_callback("requestAnimationFrame", callback_arg.as_str().trim())?;
    Ok(Some(callback))
}

fn parse_queue_microtask_expr(src: &str) -> Result<Option<ScriptHandler>> {
    let mut cursor = Cursor::new(src);
    let handler = parse_queue_microtask_call(&mut cursor)?;
    cursor.skip_ws();
    if cursor.eof() { Ok(handler) } else { Ok(None) }
}

fn parse_queue_microtask_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let Some(handler) = parse_queue_microtask_call(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported queueMicrotask statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::QueueMicrotask { handler }))
}

fn parse_queue_microtask_call(cursor: &mut Cursor<'_>) -> Result<Option<ScriptHandler>> {
    cursor.skip_ws();
    if !cursor.consume_ascii("queueMicrotask") {
        return Ok(None);
    }

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() {
        return Err(Error::ScriptParse(
            "queueMicrotask requires 1 argument".into(),
        ));
    }
    if args.len() != 1 {
        return Err(Error::ScriptParse(
            "queueMicrotask supports only 1 argument".into(),
        ));
    }

    let callback_arg = strip_js_comments(args[0]);
    let mut callback_cursor = Cursor::new(callback_arg.as_str().trim());
    let (params, body, _) = parse_callback(&mut callback_cursor, 1, "callback parameters")?;
    callback_cursor.skip_ws();
    if !callback_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported queueMicrotask callback: {}",
            args[0].trim()
        )));
    }

    Ok(Some(ScriptHandler {
        params,
        stmts: parse_block_statements(&body)?,
    }))
}

fn parse_template_literal(src: &str) -> Result<Expr> {
    let inner = &src[1..src.len() - 1];
    let bytes = inner.as_bytes();

    let mut parts: Vec<Expr> = Vec::new();
    let mut text_start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }

        if bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            if let Some(text) = inner.get(text_start..i) {
                let text = unescape_string(text);
                if !text.is_empty() {
                    parts.push(Expr::String(text));
                }
            }
            let expr_start = i + 2;
            let expr_end = find_matching_brace(inner, expr_start)?;
            let expr_src = inner
                .get(expr_start..expr_end)
                .ok_or_else(|| Error::ScriptParse("invalid template expression".into()))?;
            let expr = parse_expr(expr_src.trim())?;
            parts.push(expr);

            i = expr_end + 1;
            text_start = i;
            continue;
        }

        i += 1;
    }

    if let Some(text) = inner.get(text_start..) {
        let text = unescape_string(text);
        if !text.is_empty() {
            parts.push(Expr::String(text));
        }
    }

    if parts.is_empty() {
        return Ok(Expr::String(String::new()));
    }

    if parts.len() == 1 {
        return Ok(parts.remove(0));
    }

    Ok(Expr::Add(parts))
}

fn starts_with_window_member_access(src: &str) -> bool {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("window") {
        return false;
    }
    cursor.skip_ws();
    cursor.consume_byte(b'.')
}

fn parse_dom_access(src: &str) -> Result<Option<(DomQuery, DomProp)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();

    let head = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse(format!("expected property name in: {src}")))?;

    cursor.skip_ws();
    let nested = if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        Some(
            cursor
                .parse_identifier()
                .ok_or_else(|| Error::ScriptParse(format!("expected nested property in: {src}")))?,
        )
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let is_anchor_target = !matches!(target, DomQuery::DocumentRoot)
        && !matches!(
            &target,
            DomQuery::Var(name)
                if matches!(
                    name.as_str(),
                    "location" | "history" | "window" | "document" | "navigator" | "clipboard"
                )
        );

    let prop = match (head.as_str(), nested.as_ref()) {
        ("value", None) => DomProp::Value,
        ("value", Some(length)) if length == "length" => DomProp::ValueLength,
        ("checked", None) => DomProp::Checked,
        ("open", None) => DomProp::Open,
        ("returnValue", None) => DomProp::ReturnValue,
        ("closedBy", None) | ("closedby", None) => DomProp::ClosedBy,
        ("readonly", None) | ("readOnly", None) => DomProp::Readonly,
        ("required", None) => DomProp::Required,
        ("disabled", None) => DomProp::Disabled,
        ("textContent", None) => DomProp::TextContent,
        ("innerText", None) if !matches!(target, DomQuery::DocumentRoot) => DomProp::InnerText,
        ("innerHTML", None) => DomProp::InnerHtml,
        ("className", None) => DomProp::ClassName,
        ("id", None) => DomProp::Id,
        ("name", None) => DomProp::Name,
        ("lang", None) => DomProp::Lang,
        ("offsetWidth", None) => DomProp::OffsetWidth,
        ("offsetHeight", None) => DomProp::OffsetHeight,
        ("offsetLeft", None) => DomProp::OffsetLeft,
        ("offsetTop", None) => DomProp::OffsetTop,
        ("scrollWidth", None) => DomProp::ScrollWidth,
        ("scrollHeight", None) => DomProp::ScrollHeight,
        ("scrollLeft", None) => DomProp::ScrollLeft,
        ("scrollTop", None) => DomProp::ScrollTop,
        ("activeElement", None) if matches!(target, DomQuery::DocumentRoot) => {
            DomProp::ActiveElement
        }
        ("characterSet", None) | ("charset", None) | ("inputEncoding", None)
            if matches!(target, DomQuery::DocumentRoot) =>
        {
            DomProp::CharacterSet
        }
        ("compatMode", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::CompatMode,
        ("contentType", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::ContentType,
        ("readyState", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::ReadyState,
        ("referrer", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Referrer,
        ("title", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Title,
        ("URL", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Url,
        ("documentURI", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::DocumentUri,
        ("location", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Location,
        ("location", Some(href)) if matches!(target, DomQuery::DocumentRoot) && href == "href" => {
            DomProp::LocationHref
        }
        ("location", Some(protocol))
            if matches!(target, DomQuery::DocumentRoot) && protocol == "protocol" =>
        {
            DomProp::LocationProtocol
        }
        ("location", Some(host)) if matches!(target, DomQuery::DocumentRoot) && host == "host" => {
            DomProp::LocationHost
        }
        ("location", Some(hostname))
            if matches!(target, DomQuery::DocumentRoot) && hostname == "hostname" =>
        {
            DomProp::LocationHostname
        }
        ("location", Some(port)) if matches!(target, DomQuery::DocumentRoot) && port == "port" => {
            DomProp::LocationPort
        }
        ("location", Some(pathname))
            if matches!(target, DomQuery::DocumentRoot) && pathname == "pathname" =>
        {
            DomProp::LocationPathname
        }
        ("location", Some(search))
            if matches!(target, DomQuery::DocumentRoot) && search == "search" =>
        {
            DomProp::LocationSearch
        }
        ("location", Some(hash)) if matches!(target, DomQuery::DocumentRoot) && hash == "hash" => {
            DomProp::LocationHash
        }
        ("location", Some(origin))
            if matches!(target, DomQuery::DocumentRoot) && origin == "origin" =>
        {
            DomProp::LocationOrigin
        }
        ("location", Some(ancestor_origins))
            if matches!(target, DomQuery::DocumentRoot)
                && ancestor_origins == "ancestorOrigins" =>
        {
            DomProp::LocationAncestorOrigins
        }
        ("history", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::History,
        ("history", Some(length))
            if matches!(target, DomQuery::DocumentRoot) && length == "length" =>
        {
            DomProp::HistoryLength
        }
        ("history", Some(state))
            if matches!(target, DomQuery::DocumentRoot) && state == "state" =>
        {
            DomProp::HistoryState
        }
        ("history", Some(scroll_restoration))
            if matches!(target, DomQuery::DocumentRoot)
                && scroll_restoration == "scrollRestoration" =>
        {
            DomProp::HistoryScrollRestoration
        }
        ("defaultView", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::DefaultView,
        ("hidden", None) => DomProp::Hidden,
        ("visibilityState", None) if matches!(target, DomQuery::DocumentRoot) => {
            DomProp::VisibilityState
        }
        ("forms", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Forms,
        ("images", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Images,
        ("links", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Links,
        ("scripts", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Scripts,
        ("children", None) if matches!(target, DomQuery::DocumentRoot) => DomProp::Children,
        ("childElementCount", None) if matches!(target, DomQuery::DocumentRoot) => {
            DomProp::ChildElementCount
        }
        ("firstElementChild", None) if matches!(target, DomQuery::DocumentRoot) => {
            DomProp::FirstElementChild
        }
        ("lastElementChild", None) if matches!(target, DomQuery::DocumentRoot) => {
            DomProp::LastElementChild
        }
        ("currentScript", None) if matches!(target, DomQuery::DocumentRoot) => {
            DomProp::CurrentScript
        }
        ("forms", Some(length))
            if matches!(target, DomQuery::DocumentRoot) && length == "length" =>
        {
            DomProp::FormsLength
        }
        ("images", Some(length))
            if matches!(target, DomQuery::DocumentRoot) && length == "length" =>
        {
            DomProp::ImagesLength
        }
        ("links", Some(length))
            if matches!(target, DomQuery::DocumentRoot) && length == "length" =>
        {
            DomProp::LinksLength
        }
        ("scripts", Some(length))
            if matches!(target, DomQuery::DocumentRoot) && length == "length" =>
        {
            DomProp::ScriptsLength
        }
        ("children", Some(length))
            if matches!(target, DomQuery::DocumentRoot) && length == "length" =>
        {
            DomProp::ChildrenLength
        }
        ("attributionSrc", None) | ("attributionsrc", None) if is_anchor_target => {
            DomProp::AnchorAttributionSrc
        }
        ("download", None) if is_anchor_target => DomProp::AnchorDownload,
        ("hash", None) if is_anchor_target => DomProp::AnchorHash,
        ("host", None) if is_anchor_target => DomProp::AnchorHost,
        ("hostname", None) if is_anchor_target => DomProp::AnchorHostname,
        ("href", None) if is_anchor_target => DomProp::AnchorHref,
        ("hreflang", None) if is_anchor_target => DomProp::AnchorHreflang,
        ("interestForElement", None) if is_anchor_target => DomProp::AnchorInterestForElement,
        ("origin", None) if is_anchor_target => DomProp::AnchorOrigin,
        ("password", None) if is_anchor_target => DomProp::AnchorPassword,
        ("pathname", None) if is_anchor_target => DomProp::AnchorPathname,
        ("ping", None) if is_anchor_target => DomProp::AnchorPing,
        ("port", None) if is_anchor_target => DomProp::AnchorPort,
        ("protocol", None) if is_anchor_target => DomProp::AnchorProtocol,
        ("referrerPolicy", None) if is_anchor_target => DomProp::AnchorReferrerPolicy,
        ("rel", None) if is_anchor_target => DomProp::AnchorRel,
        ("relList", None) if is_anchor_target => DomProp::AnchorRelList,
        ("relList", Some(length)) if is_anchor_target && length == "length" => {
            DomProp::AnchorRelListLength
        }
        ("search", None) if is_anchor_target => DomProp::AnchorSearch,
        ("target", None) if is_anchor_target => DomProp::AnchorTarget,
        ("text", None) if is_anchor_target => DomProp::AnchorText,
        ("type", None) if is_anchor_target => DomProp::AnchorType,
        ("username", None) if is_anchor_target => DomProp::AnchorUsername,
        ("charset", None) if is_anchor_target => DomProp::AnchorCharset,
        ("coords", None) if is_anchor_target => DomProp::AnchorCoords,
        ("rev", None) if is_anchor_target => DomProp::AnchorRev,
        ("shape", None) if is_anchor_target => DomProp::AnchorShape,
        ("dataset", Some(key)) => DomProp::Dataset(key.clone()),
        ("style", Some(name)) => DomProp::Style(name.clone()),
        _ => {
            if matches!(target, DomQuery::DocumentRoot) && starts_with_window_member_access(src) {
                return Ok(None);
            }
            if matches!(target, DomQuery::Var(_) | DomQuery::VarPath { .. }) {
                return Ok(None);
            }
            let prop_label = if let Some(nested) = nested {
                format!("{head}.{nested}")
            } else {
                head
            };
            return Err(Error::ScriptParse(format!(
                "unsupported DOM property '{}' in: {src}",
                prop_label
            )));
        }
    };

    Ok(Some((target, prop)))
}

fn parse_get_attribute_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("getAttribute") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, name)))
}

fn parse_has_attribute_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("hasAttribute") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, name)))
}

fn parse_dom_matches_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("matches") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'(') {
        return Ok(None);
    }
    cursor.skip_ws();
    let selector = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, selector)))
}

fn parse_dom_closest_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("closest") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'(') {
        return Ok(None);
    }
    cursor.skip_ws();
    let selector = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, selector)))
}

fn parse_dom_computed_style_property_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("getComputedStyle") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let target = parse_element_target(&mut cursor)?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    if !cursor.consume_ascii("getPropertyValue") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let property = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((target, property)))
}

fn parse_event_property_expr(src: &str) -> Result<Option<(String, EventExprProp)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let Some(event_var) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(head) = cursor.parse_identifier() else {
        return Ok(None);
    };

    let mut nested = None;
    cursor.skip_ws();
    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        nested = cursor.parse_identifier();
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    if event_var == "history"
        && matches!(
            (head.as_str(), nested.as_deref()),
            ("state", None) | ("oldState", None) | ("newState", None)
        )
    {
        return Ok(None);
    }

    let prop = match (head.as_str(), nested.as_deref()) {
        ("type", None) => EventExprProp::Type,
        ("target", None) => EventExprProp::Target,
        ("currentTarget", None) => EventExprProp::CurrentTarget,
        ("target", Some("name")) => EventExprProp::TargetName,
        ("currentTarget", Some("name")) => EventExprProp::CurrentTargetName,
        ("defaultPrevented", None) => EventExprProp::DefaultPrevented,
        ("isTrusted", None) => EventExprProp::IsTrusted,
        ("bubbles", None) => EventExprProp::Bubbles,
        ("cancelable", None) => EventExprProp::Cancelable,
        ("target", Some("id")) => EventExprProp::TargetId,
        ("currentTarget", Some("id")) => EventExprProp::CurrentTargetId,
        ("eventPhase", None) => EventExprProp::EventPhase,
        ("timeStamp", None) => EventExprProp::TimeStamp,
        ("state", None) => EventExprProp::State,
        ("oldState", None) => EventExprProp::OldState,
        ("newState", None) => EventExprProp::NewState,
        _ => return Ok(None),
    };

    Ok(Some((event_var, prop)))
}

fn parse_class_list_contains_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("classList") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    if !cursor.consume_ascii("contains") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let class_name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, class_name)))
}

fn parse_query_selector_all_length_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    let is_list_target = matches!(
        target,
        DomQuery::BySelectorAll { .. } | DomQuery::QuerySelectorAll { .. } | DomQuery::Var(_)
    );
    if !is_list_target {
        return Ok(None);
    }

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("length") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(target))
}

fn parse_form_elements_length_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let form = match parse_form_elements_base(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("elements") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("length") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(form))
}

fn parse_new_form_data_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(form) = parse_new_form_data_target(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(form))
}

fn parse_form_data_get_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    parse_form_data_method_expr(src, "get")
}

fn parse_form_data_has_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    parse_form_data_method_expr(src, "has")
}

fn parse_form_data_get_all_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    parse_form_data_method_expr(src, "getAll")
}

fn parse_form_data_get_all_length_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let Some(source) = parse_form_data_source(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if method != "getAll" {
        return Ok(None);
    }

    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("length") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((source, name)))
}

fn parse_form_data_method_expr(
    src: &str,
    method: &str,
) -> Result<Option<(FormDataSource, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let Some(source) = parse_form_data_source(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(actual_method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if actual_method != method {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((source, name)))
}

fn parse_form_data_source(cursor: &mut Cursor<'_>) -> Result<Option<FormDataSource>> {
    if let Some(form) = parse_new_form_data_target(cursor)? {
        return Ok(Some(FormDataSource::NewForm(form)));
    }

    if let Some(var_name) = cursor.parse_identifier() {
        return Ok(Some(FormDataSource::Var(var_name)));
    }

    Ok(None)
}

fn parse_new_form_data_target(cursor: &mut Cursor<'_>) -> Result<Option<DomQuery>> {
    cursor.skip_ws();
    let start = cursor.pos();

    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            cursor.set_pos(start);
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("FormData") {
        cursor.set_pos(start);
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(args_src.trim(), b',');
    if args.len() != 1 {
        return Err(Error::ScriptParse(
            "new FormData requires exactly one argument".into(),
        ));
    }

    let arg = args[0].trim();
    let mut arg_cursor = Cursor::new(arg);
    arg_cursor.skip_ws();
    let form = parse_form_elements_base(&mut arg_cursor)?;
    arg_cursor.skip_ws();
    if !arg_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported FormData argument: {arg}"
        )));
    }

    Ok(Some(form))
}

fn parse_string_literal_exact(src: &str) -> Result<String> {
    let bytes = src.as_bytes();
    if bytes.len() < 2 {
        return Err(Error::ScriptParse("invalid string literal".into()));
    }
    let quote = bytes[0];
    if (quote != b'\'' && quote != b'"') || bytes[bytes.len() - 1] != quote {
        return Err(Error::ScriptParse(format!("invalid string literal: {src}")));
    }

    let mut escaped = false;
    let mut i = 1;
    while i + 1 < bytes.len() {
        let b = bytes[i];
        if escaped {
            escaped = false;
        } else if b == b'\\' {
            escaped = true;
        } else if b == quote {
            return Err(Error::ScriptParse(format!("unexpected quote in: {src}")));
        }
        i += 1;
    }

    Ok(unescape_string(&src[1..src.len() - 1]))
}

fn strip_outer_parens(mut src: &str) -> &str {
    loop {
        let trimmed = src.trim();
        if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
            return trimmed;
        }

        if !is_fully_wrapped_in_parens(trimmed) {
            return trimmed;
        }

        src = &trimmed[1..trimmed.len() - 1];
    }
}

fn is_fully_wrapped_in_parens(src: &str) -> bool {
    let bytes = src.as_bytes();
    let mut depth = 0isize;
    let mut i = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 && i + 1 < bytes.len() {
                        return false;
                    }
                }
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }
        i += 1;
    }

    depth == 0
}

fn find_top_level_assignment(src: &str) -> Option<(usize, usize)> {
    let bytes = src.as_bytes();
    let mut i = 0usize;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                b'=' => {
                    if paren == 0 && bracket == 0 && brace == 0 {
                        if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                            if i + 2 < bytes.len() && bytes[i + 2] == b'=' {
                                i += 2;
                            } else {
                                i += 1;
                            }
                        } else if i >= 2 && &bytes[i - 2..=i] == b"&&=" {
                            return Some((i - 2, 3));
                        } else if i >= 2 && &bytes[i - 2..=i] == b"||=" {
                            return Some((i - 2, 3));
                        } else if i >= 2 && &bytes[i - 2..=i] == b"??=" {
                            return Some((i - 2, 3));
                        } else if i >= 2 && &bytes[i - 2..=i] == b"**=" {
                            return Some((i - 2, 3));
                        } else if i >= 3 && &bytes[i - 3..=i] == b">>>=" {
                            return Some((i - 3, 4));
                        } else if i >= 2 && &bytes[i - 2..=i] == b"<<=" {
                            return Some((i - 2, 3));
                        } else if i >= 2 && &bytes[i - 2..=i] == b">>=" {
                            return Some((i - 2, 3));
                        } else if i > 0
                            && matches!(
                                bytes[i - 1],
                                b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^'
                            )
                        {
                            return Some((i - 1, 2));
                        } else {
                            return Some((i, 1));
                        }
                    }
                }
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }
        i += 1;
    }

    None
}

fn find_top_level_ternary_question(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                b'?' if paren == 0 && bracket == 0 && brace == 0 => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'?' {
                        i += 1;
                    } else {
                        return Some(i);
                    }
                }
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }

        i += 1;
    }

    None
}

fn find_matching_ternary_colon(src: &str, from: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = from;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;
    let mut nested_ternary = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                b'?' if paren == 0 && bracket == 0 && brace == 0 => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'?' {
                        i += 1;
                    } else {
                        nested_ternary += 1;
                    }
                }
                b':' if paren == 0 && bracket == 0 && brace == 0 => {
                    if nested_ternary == 0 {
                        return Some(i);
                    }
                    nested_ternary -= 1;
                }
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }

        i += 1;
    }

    None
}

fn split_top_level_by_char(src: &str, target: u8) -> Vec<&str> {
    let bytes = src.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        let b = bytes[i];
        if scanner.is_top_level() && b == target {
            if let Some(part) = src.get(start..i) {
                parts.push(part);
            }
            start = i + 1;
        }
        i = scanner.advance(bytes, i);
    }

    if let Some(last) = src.get(start..) {
        parts.push(last);
    }

    parts
}

fn split_top_level_by_ops<'a>(src: &'a str, ops: &[&'a str]) -> (Vec<&'a str>, Vec<&'a str>) {
    let bytes = src.as_bytes();
    let mut parts = Vec::new();
    let mut found_ops = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() {
            let mut matched = None;
            for op in ops {
                let op_bytes = op.as_bytes();
                if i + op_bytes.len() <= bytes.len() && &bytes[i..i + op_bytes.len()] == op_bytes {
                    if op_bytes.iter().all(|b| b.is_ascii_alphabetic()) {
                        if i > 0 && is_ident_char(bytes[i - 1]) {
                            continue;
                        }
                        if i + op_bytes.len() < bytes.len() && is_ident_char(bytes[i + op_bytes.len()]) {
                            continue;
                        }
                    } else if op.len() == 1 && (op == &"<" || op == &">") {
                        let prev = if i == 0 { None } else { Some(bytes[i - 1]) };
                        let next = bytes.get(i + 1).copied();
                        if prev == Some(b'<')
                            || prev == Some(b'>')
                            || next == Some(b'<')
                            || next == Some(b'>')
                        {
                            continue;
                        }
                    }
                    matched = Some(*op);
                    break;
                }
            }
            if let Some(op) = matched {
                if let Some(part) = src.get(start..i) {
                    parts.push(part);
                    found_ops.push(op);
                    scanner.consume_significant_bytes(b"=");
                    i += op.len();
                    start = i;
                    continue;
                }
            }
        }
        i = scanner.advance(bytes, i);
    }

    if let Some(last) = src.get(start..) {
        parts.push(last);
    }

    (parts, found_ops)
}

fn find_matching_brace(src: &str, start: usize) -> Result<usize> {
    let bytes = src.as_bytes();
    let mut i = start;
    let mut depth = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'{' => depth += 1,
                b'}' => {
                    if depth == 0 {
                        return Ok(i);
                    }
                    depth -= 1;
                }
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }
        i += 1;
    }

    Err(Error::ScriptParse("unclosed template expression".into()))
}
