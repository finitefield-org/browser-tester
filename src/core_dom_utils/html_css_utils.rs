use super::*;

pub(crate) fn should_strip_inner_html_element(tag_name: &str) -> bool {
    tag_name.eq_ignore_ascii_case("script")
}

pub(crate) fn sanitize_inner_html_element_attrs(element: &mut Element) {
    element.attrs.retain(|name, value| {
        if name.starts_with("on") {
            return false;
        }
        if is_javascript_url_attr(name) && is_javascript_scheme(value) {
            return false;
        }
        true
    });
    element.checked = element.attrs.contains_key("checked");
    element.indeterminate = false;
    element.disabled = element.attrs.contains_key("disabled");
    element.readonly = element.attrs.contains_key("readonly");
    element.required = element.attrs.contains_key("required");
    element.value = if is_color_input_element(element) {
        normalize_color_input_value(element.attrs.get("value").map(String::as_str).unwrap_or(""))
    } else if is_date_input_element(element) {
        normalize_date_input_value(element.attrs.get("value").map(String::as_str).unwrap_or(""))
    } else if is_datetime_local_input_element(element) {
        normalize_datetime_local_input_value(
            element.attrs.get("value").map(String::as_str).unwrap_or(""),
        )
    } else if is_time_input_element(element) {
        normalize_time_input_value(element.attrs.get("value").map(String::as_str).unwrap_or(""))
    } else if is_number_input_element(element) {
        normalize_number_input_value(element.attrs.get("value").map(String::as_str).unwrap_or(""))
    } else if is_range_input_element(element) {
        normalize_range_input_value(
            element.attrs.get("value").map(String::as_str).unwrap_or(""),
            element.attrs.get("min").map(String::as_str),
            element.attrs.get("max").map(String::as_str),
            element.attrs.get("step").map(String::as_str),
            element.attrs.get("value").map(String::as_str),
        )
    } else if is_password_input_element(element) {
        normalize_password_input_value(element.attrs.get("value").map(String::as_str).unwrap_or(""))
    } else if is_file_input_element(element) {
        normalize_file_input_value(element.attrs.get("value").map(String::as_str).unwrap_or(""))
    } else if is_image_input_element(element) {
        normalize_image_input_value(element.attrs.get("value").map(String::as_str).unwrap_or(""))
    } else {
        element.attrs.get("value").cloned().unwrap_or_default()
    };
    if is_file_input_element(element) {
        element.files.clear();
    }
    let len = element.value.chars().count();
    element.custom_validity_message.clear();
    element.selection_start = len;
    element.selection_end = len;
    element.selection_direction = "none".to_string();
}

pub(crate) fn is_javascript_url_attr(name: &str) -> bool {
    matches!(
        name,
        "href" | "src" | "xlink:href" | "action" | "formaction"
    )
}

pub(crate) fn is_javascript_scheme(value: &str) -> bool {
    let mut normalized = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_whitespace() || ch.is_ascii_control() {
            continue;
        }
        normalized.push(ch.to_ascii_lowercase());
    }
    normalized.starts_with("javascript:")
}

pub(crate) fn escape_html_text_for_serialization(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

pub(crate) fn escape_html_attr_for_serialization(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

pub(crate) fn class_tokens(class_attr: Option<&str>) -> Vec<String> {
    class_attr
        .map(|value| {
            value
                .split_whitespace()
                .filter(|token| !token.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(crate) fn set_class_attr(element: &mut Element, classes: &[String]) {
    if classes.is_empty() {
        element.attrs.remove("class");
    } else {
        element.attrs.insert("class".to_string(), classes.join(" "));
    }
}

pub(crate) fn dataset_key_to_attr_name(key: &str) -> String {
    format!("data-{}", js_prop_to_css_name(key))
}

pub(crate) fn js_prop_to_css_name(prop: &str) -> String {
    let mut out = String::new();
    for ch in prop.chars() {
        if ch.is_ascii_uppercase() {
            out.push('-');
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

pub(crate) fn parse_style_declarations(style_attr: Option<&str>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Some(style_attr) = style_attr else {
        return out;
    };

    let mut start = 0usize;
    let mut i = 0usize;
    let bytes = style_attr.as_bytes();
    let mut paren_depth = 0isize;
    let mut quote: Option<u8> = None;

    while i < bytes.len() {
        let ch = bytes[i];
        match (quote, ch) {
            (Some(_q), _) if ch == b'\\' => {
                if i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
            }
            (Some(q), _) if ch == q => {
                quote = None;
            }
            (Some(_), _) => {}
            (None, b'\'') | (None, b'"') => {
                quote = Some(ch);
            }
            (None, b'(') => paren_depth += 1,
            (None, b')') => paren_depth = paren_depth.saturating_sub(1),
            (None, b';') if paren_depth == 0 => {
                let decl = &style_attr[start..i];
                push_style_declaration(decl, &mut out);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }

    let decl = &style_attr[start..];
    push_style_declaration(decl, &mut out);

    out
}

pub(crate) fn push_style_declaration(raw_decl: &str, out: &mut Vec<(String, String)>) {
    let decl = raw_decl.trim();
    if decl.is_empty() {
        return;
    }

    let bytes = decl.as_bytes();
    let mut colon = None;
    let mut paren_depth = 0isize;
    let mut quote: Option<u8> = None;
    let mut i = 0usize;

    while i < bytes.len() {
        let ch = bytes[i];
        match (quote, ch) {
            (Some(_q), _) if ch == b'\\' => {
                if i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
            }
            (Some(q), _) if ch == q => quote = None,
            (Some(_), _) => {}
            (None, b'\'') | (None, b'"') => quote = Some(ch),
            (None, b'(') => paren_depth += 1,
            (None, b')') => paren_depth = paren_depth.saturating_sub(1),
            (None, b':') if paren_depth == 0 && colon.is_none() => {
                colon = Some(i);
                break;
            }
            _ => {}
        }
        i += 1;
    }

    let Some(colon) = colon else {
        return;
    };

    let name = decl[..colon].trim().to_ascii_lowercase();
    if name.is_empty() {
        return;
    }

    let value = decl[colon + 1..].trim().to_string();

    if let Some(pos) = out.iter().position(|(existing, _)| existing == &name) {
        out[pos].1 = value;
    } else {
        out.push((name, value));
    }
}

pub(crate) fn serialize_style_declarations(decls: &[(String, String)]) -> String {
    let mut out = String::new();
    for (idx, (name, value)) in decls.iter().enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        out.push_str(name);
        out.push_str(": ");
        out.push_str(value);
        out.push(';');
    }
    out
}
