pub(super) fn parse_mul_expr(src: &str) -> Result<Expr> {
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

pub(super) fn parse_pow_expr(src: &str) -> Result<Expr> {
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

pub(super) fn parse_unary_expr(src: &str) -> Result<Expr> {
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

pub(super) fn fold_binary<F, G>(
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

pub(super) fn parse_primary(src: &str) -> Result<Expr> {
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

    if let Some(expr) = parse_new_error_expr(src)? {
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

    if let Some(expr) = parse_object_prototype_has_own_property_call_expr(src)? {
        return Ok(expr);
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

    if let Some(handler_expr) = parse_function_expr(src)? {
        return Ok(handler_expr);
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

    if let Some(expr) = parse_member_index_get_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_member_get_expr(src)? {
        return Ok(expr);
    }

    if is_ident(src) {
        return Ok(Expr::Var(src.to_string()));
    }

    Err(Error::ScriptParse(format!("unsupported expression: {src}")))
}

pub(super) fn parse_numeric_literal(src: &str) -> Result<Option<Expr>> {
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

pub(super) fn parse_bigint_literal(src: &str) -> Result<Option<Expr>> {
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

pub(super) fn parse_prefixed_integer_literal(
    src: &str,
    prefix: &str,
    radix: u32,
) -> Result<Option<Expr>> {
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

pub(super) fn strip_keyword_operator<'a>(src: &'a str, keyword: &str) -> Option<&'a str> {
    if !src.starts_with(keyword) {
        return None;
    }

    let after = &src[keyword.len()..];
    if after.is_empty() || !is_ident_char(after.as_bytes()[0]) {
        return Some(after.trim_start());
    }

    None
}

pub(super) fn parse_element_ref_expr(src: &str) -> Result<Option<DomQuery>> {
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
    if matches!(target, DomQuery::DocumentRoot) || is_non_dom_var_target(&target) {
        return Ok(None);
    }
    Ok(Some(target))
}

pub(super) fn parse_document_create_element_expr(src: &str) -> Result<Option<String>> {
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

pub(super) fn parse_document_create_text_node_expr(src: &str) -> Result<Option<String>> {
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

pub(super) fn parse_document_has_focus_expr(src: &str) -> Result<bool> {
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

pub(super) fn parse_location_method_expr(src: &str) -> Result<Option<Expr>> {
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

pub(super) fn parse_location_base(cursor: &mut Cursor<'_>) -> bool {
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

pub(super) fn parse_history_method_expr(src: &str) -> Result<Option<Expr>> {
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

pub(super) fn parse_history_base(cursor: &mut Cursor<'_>) -> bool {
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

pub(super) fn parse_clipboard_method_expr(src: &str) -> Result<Option<Expr>> {
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

