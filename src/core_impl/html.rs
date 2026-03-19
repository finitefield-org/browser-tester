use super::*;

#[path = "html/html_optional_tag_rules.rs"]
mod html_optional_tag_rules;
#[path = "html/html_raw_end_tag_scanner.rs"]
mod html_raw_end_tag_scanner;
#[path = "html/html_script_helpers.rs"]
mod html_script_helpers;
#[path = "html/html_tag_parsing.rs"]
mod html_tag_parsing;
#[path = "html/html_text_entities.rs"]
mod html_text_entities;

use html_optional_tag_rules::*;
use html_raw_end_tag_scanner::*;
use html_script_helpers::*;
use html_tag_parsing::*;
use html_text_entities::*;

pub(super) fn unescape_string(src: &str) -> String {
    let mut out = String::new();
    let chars = src.chars().collect::<Vec<_>>();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => {
                    out.push('\n');
                    i += 2;
                }
                'r' => {
                    out.push('\r');
                    i += 2;
                }
                't' => {
                    out.push('\t');
                    i += 2;
                }
                '\\' => {
                    out.push('\\');
                    i += 2;
                }
                '\'' => {
                    out.push('\'');
                    i += 2;
                }
                '"' => {
                    out.push('"');
                    i += 2;
                }
                '`' => {
                    out.push('`');
                    i += 2;
                }
                '$' => {
                    out.push('$');
                    i += 2;
                }
                'u' if i + 5 < chars.len() => {
                    let hex = [chars[i + 2], chars[i + 3], chars[i + 4], chars[i + 5]];
                    let mut parsed = String::new();
                    for ch in hex {
                        parsed.push(ch);
                    }
                    if parsed.chars().all(|ch| ch.is_ascii_hexdigit()) {
                        if let Ok(codepoint) = u16::from_str_radix(&parsed, 16) {
                            out.push(crate::js_regex::internalize_utf16_code_unit(codepoint));
                            i += 6;
                            continue;
                        }
                    }
                    out.push('u');
                    i += 2;
                }
                'x' if i + 3 < chars.len() => {
                    let hex = [chars[i + 2], chars[i + 3]];
                    let mut parsed = String::new();
                    for ch in hex {
                        parsed.push(ch);
                    }
                    if parsed.chars().all(|ch| ch.is_ascii_hexdigit()) {
                        if let Ok(codepoint) = u32::from_str_radix(&parsed, 16) {
                            if let Some(ch) = char::from_u32(codepoint) {
                                out.push(ch);
                                i += 4;
                                continue;
                            }
                        }
                    }
                    out.push('x');
                    i += 2;
                }
                other => {
                    out.push(other);
                    i += 2;
                }
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

pub(super) fn parse_html(html: &str) -> Result<ParseOutput> {
    let mut dom = Dom::new();
    let mut scripts = Vec::new();

    let mut stack = vec![dom.root];
    let bytes = html.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        if starts_with_at(bytes, i, b"<!--") {
            if let Some(end) = find_subslice(bytes, i + 4, b"-->") {
                i = end + 3;
            } else {
                return Err(Error::HtmlParse("unclosed HTML comment".into()));
            }
            continue;
        }

        if bytes[i] == b'<' {
            if starts_with_at(bytes, i, b"</") {
                let (tag, next) = parse_end_tag(html, i)?;
                i = next;

                while stack.len() > 1 {
                    let top = *stack
                        .last()
                        .ok_or_else(|| Error::HtmlParse("invalid stack state".into()))?;
                    let top_tag = dom.tag_name(top).unwrap_or("");
                    stack.pop();
                    if top_tag.eq_ignore_ascii_case(&tag) {
                        break;
                    }
                }
                continue;
            }

            if starts_with_at(bytes, i, b"<!") {
                i = parse_declaration_tag(html, i)?;
                continue;
            }

            let (tag, attrs, self_closing, next) = parse_start_tag(html, i)?;
            i = next;
            let inside_template = stack.iter().any(|node| {
                dom.tag_name(*node)
                    .is_some_and(|open_tag| open_tag.eq_ignore_ascii_case("template"))
            });
            let script_src_attr = attrs.get("src").cloned();
            let executable_script = tag.eq_ignore_ascii_case("script")
                && !inside_template
                && is_executable_script_type(attrs.get("type").map(String::as_str));
            let module_script = tag.eq_ignore_ascii_case("script")
                && !inside_template
                && is_module_script_type(attrs.get("type").map(String::as_str));
            close_optional_description_item_start_tag(&dom, &mut stack, &tag);
            close_optional_list_item_start_tag(&dom, &mut stack, &tag);
            close_optional_option_start_tag(&dom, &mut stack, &tag);
            close_optional_optgroup_start_tag(&dom, &mut stack, &tag);
            close_optional_ruby_text_start_tag(&dom, &mut stack, &tag);
            close_optional_ruby_fallback_parenthesis_start_tag(&dom, &mut stack, &tag);
            close_optional_paragraph_start_tag(&dom, &mut stack, &tag);

            let parent = *stack
                .last()
                .ok_or_else(|| Error::HtmlParse("missing parent element".into()))?;
            let node = dom.create_element(parent, tag.clone(), attrs);

            if tag.eq_ignore_ascii_case("script") {
                let close = if executable_script {
                    find_case_insensitive_end_tag(bytes, i, b"script")
                        .or_else(|| find_case_insensitive_raw_end_tag(bytes, i, b"script"))
                } else {
                    find_case_insensitive_raw_end_tag(bytes, i, b"script")
                }
                .ok_or_else(|| Error::HtmlParse("unclosed <script>".into()))?;
                if let Some(script_body) = html.get(i..close) {
                    if !script_body.is_empty() {
                        dom.create_text(node, script_body.to_string());
                    }
                    if executable_script {
                        if let Some(src) = script_src_attr.as_deref() {
                            if let Some(source) = decode_data_script_source(src)? {
                                scripts.push(ScriptSource {
                                    code: source,
                                    is_module: module_script,
                                });
                            }
                        } else if !script_body.is_empty() {
                            scripts.push(ScriptSource {
                                code: script_body.to_string(),
                                is_module: module_script,
                            });
                        }
                    }
                }
                i = close;
                let (_, after_end) = parse_end_tag(html, i)?;
                i = after_end;
                continue;
            }

            if tag.eq_ignore_ascii_case("noscript") && !self_closing {
                let close = find_case_insensitive_raw_end_tag(bytes, i, b"noscript")
                    .ok_or_else(|| Error::HtmlParse("unclosed <noscript>".into()))?;
                if let Some(noscript_body) = html.get(i..close) {
                    if !noscript_body.is_empty() {
                        dom.create_text(node, noscript_body.to_string());
                    }
                }
                i = close;
                let (_, after_end) = parse_end_tag(html, i)?;
                i = after_end;
                continue;
            }

            if tag.eq_ignore_ascii_case("title") && !self_closing {
                let close = find_case_insensitive_raw_end_tag(bytes, i, b"title")
                    .ok_or_else(|| Error::HtmlParse("unclosed <title>".into()))?;
                if let Some(title_body) = html.get(i..close) {
                    if !title_body.is_empty() {
                        let decoded = decode_html_character_references(title_body);
                        if !decoded.is_empty() {
                            dom.create_text(node, decoded);
                        }
                    }
                }
                i = close;
                let (_, after_end) = parse_end_tag(html, i)?;
                i = after_end;
                continue;
            }

            if !self_closing && !is_void_tag(&tag) {
                stack.push(node);
            }
            continue;
        }

        let text_start = i;
        while i < bytes.len() && bytes[i] != b'<' {
            i += 1;
        }

        if let Some(text) = html.get(text_start..i) {
            if !text.is_empty() {
                let parent = *stack
                    .last()
                    .ok_or_else(|| Error::HtmlParse("missing parent element".into()))?;
                let mut decoded = decode_html_character_references(text);
                if should_strip_initial_pre_newline(&dom, parent) {
                    decoded = strip_initial_pre_newline(&decoded);
                }
                if !decoded.is_empty() {
                    dom.create_text(parent, decoded);
                }
            }
        }
    }

    dom.initialize_form_control_values()?;
    dom.normalize_radio_groups()?;
    dom.normalize_named_details_groups()?;
    dom.normalize_single_head_element()?;
    dom.normalize_single_body_element()?;
    dom.normalize_implied_table_bodies()?;
    Ok(ParseOutput { dom, scripts })
}

pub(super) fn is_void_tag(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

pub(super) fn can_start_regex_literal(previous: Option<u8>) -> bool {
    match previous {
        None => true,
        Some(byte) => matches!(
            byte,
            b'(' | b'['
                | b'{'
                | b','
                | b';'
                | b':'
                | b'='
                | b'!'
                | b'?'
                | b'&'
                | b'|'
                | b'^'
                | b'~'
                | b'<'
                | b'>'
                | b'+'
                | b'-'
                | b'*'
                | b'%'
                | b'/'
        ),
    }
}
