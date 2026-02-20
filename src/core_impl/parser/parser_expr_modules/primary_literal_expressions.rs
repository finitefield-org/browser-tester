use super::*;
pub(crate) fn parse_primary(src: &str) -> Result<Expr> {
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

pub(crate) fn parse_template_literal(src: &str) -> Result<Expr> {
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

pub(crate) fn parse_numeric_literal(src: &str) -> Result<Option<Expr>> {
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

pub(crate) fn parse_bigint_literal(src: &str) -> Result<Option<Expr>> {
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

pub(crate) fn parse_prefixed_integer_literal(
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
