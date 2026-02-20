use js_regex::{Captures, Regex, RegexBuilder, RegexError, escape as regex_escape};
use num_bigint::{BigInt as JsBigInt, Sign};
use num_traits::{One, ToPrimitive, Zero};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error as StdError;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

const INTERNAL_RETURN_SLOT: &str = "__bt_internal_return_value__";
const INTERNAL_SYMBOL_KEY_PREFIX: &str = "\u{0}\u{0}bt_symbol_key:";
const INTERNAL_SYMBOL_WRAPPER_KEY: &str = "\u{0}\u{0}bt_symbol_wrapper";
const INTERNAL_STRING_WRAPPER_VALUE_KEY: &str = "\u{0}\u{0}bt_string_wrapper_value";
const INTERNAL_INTL_KEY_PREFIX: &str = "\u{0}\u{0}bt_intl:";
const INTERNAL_INTL_KIND_KEY: &str = "\u{0}\u{0}bt_intl:kind";
const INTERNAL_INTL_LOCALE_KEY: &str = "\u{0}\u{0}bt_intl:locale";
const INTERNAL_INTL_OPTIONS_KEY: &str = "\u{0}\u{0}bt_intl:options";
const INTERNAL_INTL_LOCALE_DATA_KEY: &str = "\u{0}\u{0}bt_intl:localeData";
const INTERNAL_INTL_CASE_FIRST_KEY: &str = "\u{0}\u{0}bt_intl:caseFirst";
const INTERNAL_INTL_SENSITIVITY_KEY: &str = "\u{0}\u{0}bt_intl:sensitivity";
const INTERNAL_INTL_SEGMENTS_KEY: &str = "\u{0}\u{0}bt_intl:segments";
const INTERNAL_INTL_SEGMENT_INDEX_KEY: &str = "\u{0}\u{0}bt_intl:segmentIndex";
const INTERNAL_CALLABLE_KEY_PREFIX: &str = "\u{0}\u{0}bt_callable:";
const INTERNAL_CALLABLE_KIND_KEY: &str = "\u{0}\u{0}bt_callable:kind";
const INTERNAL_LOCATION_OBJECT_KEY: &str = "\u{0}\u{0}bt_location";
const INTERNAL_HISTORY_OBJECT_KEY: &str = "\u{0}\u{0}bt_history";
const INTERNAL_WINDOW_OBJECT_KEY: &str = "\u{0}\u{0}bt_window";
const INTERNAL_DOCUMENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_document";
const INTERNAL_SCOPE_DEPTH_KEY: &str = "\u{0}\u{0}bt_scope_depth";
const INTERNAL_GLOBAL_SYNC_NAMES_KEY: &str = "\u{0}\u{0}bt_global_sync_names";
const INTERNAL_NAVIGATOR_OBJECT_KEY: &str = "\u{0}\u{0}bt_navigator";
const INTERNAL_CLIPBOARD_OBJECT_KEY: &str = "\u{0}\u{0}bt_clipboard";
const INTERNAL_READABLE_STREAM_OBJECT_KEY: &str = "\u{0}\u{0}bt_readable_stream";
const INTERNAL_URL_OBJECT_KEY: &str = "\u{0}\u{0}bt_url:object";
const INTERNAL_URL_OBJECT_ID_KEY: &str = "\u{0}\u{0}bt_url:id";
const INTERNAL_URL_SEARCH_PARAMS_KEY_PREFIX: &str = "\u{0}\u{0}bt_url_search_params:";
const INTERNAL_URL_SEARCH_PARAMS_OBJECT_KEY: &str = "\u{0}\u{0}bt_url_search_params:object";
const INTERNAL_URL_SEARCH_PARAMS_ENTRIES_KEY: &str = "\u{0}\u{0}bt_url_search_params:entries";
const INTERNAL_URL_SEARCH_PARAMS_OWNER_ID_KEY: &str = "\u{0}\u{0}bt_url_search_params:owner_id";
const INTERNAL_STORAGE_KEY_PREFIX: &str = "\u{0}\u{0}bt_storage:";
const INTERNAL_STORAGE_OBJECT_KEY: &str = "\u{0}\u{0}bt_storage:object";
const INTERNAL_STORAGE_ENTRIES_KEY: &str = "\u{0}\u{0}bt_storage:entries";
const DEFAULT_LOCALE: &str = "en-US";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    HtmlParse(String),
    ScriptParse(String),
    ScriptRuntime(String),
    ScriptThrown(ThrownValue),
    SelectorNotFound(String),
    UnsupportedSelector(String),
    TypeMismatch {
        selector: String,
        expected: String,
        actual: String,
    },
    AssertionFailed {
        selector: String,
        expected: String,
        actual: String,
        dom_snippet: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HtmlParse(msg) => write!(f, "html parse error: {msg}"),
            Self::ScriptParse(msg) => write!(f, "script parse error: {msg}"),
            Self::ScriptRuntime(msg) => write!(f, "script runtime error: {msg}"),
            Self::ScriptThrown(value) => {
                write!(f, "script thrown value: {}", value.as_string())
            }
            Self::SelectorNotFound(selector) => write!(f, "selector not found: {selector}"),
            Self::UnsupportedSelector(selector) => write!(f, "unsupported selector: {selector}"),
            Self::TypeMismatch {
                selector,
                expected,
                actual,
            } => write!(
                f,
                "type mismatch for {selector}: expected {expected}, actual {actual}"
            ),
            Self::AssertionFailed {
                selector,
                expected,
                actual,
                dom_snippet,
            } => write!(
                f,
                "assertion failed for {selector}: expected {expected}, actual {actual}, snippet {dom_snippet}"
            ),
        }
    }
}

impl StdError for Error {}

#[derive(Debug, Clone, PartialEq)]
pub struct ThrownValue {
    value: Value,
}

impl ThrownValue {
    fn new(value: Value) -> Self {
        Self { value }
    }

    fn into_value(self) -> Value {
        self.value
    }

    fn as_string(&self) -> String {
        self.value.as_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeId(usize);

#[derive(Debug, Clone)]
enum NodeType {
    Document,
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone)]
struct Node {
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    node_type: NodeType,
}

#[derive(Debug, Clone)]
struct Element {
    tag_name: String,
    attrs: HashMap<String, String>,
    value: String,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    readonly: bool,
    required: bool,
    custom_validity_message: String,
    selection_start: usize,
    selection_end: usize,
    selection_direction: String,
}

#[derive(Debug, Clone)]
struct Dom {
    nodes: Vec<Node>,
    root: NodeId,
    id_index: HashMap<String, Vec<NodeId>>,
    active_element: Option<NodeId>,
    active_pseudo_element: Option<NodeId>,
}

fn has_class(element: &Element, class_name: &str) -> bool {
    element
        .attrs
        .get("class")
        .map(|classes| classes.split_whitespace().any(|c| c == class_name))
        .unwrap_or(false)
}

fn should_strip_inner_html_element(tag_name: &str) -> bool {
    tag_name.eq_ignore_ascii_case("script")
}

fn sanitize_inner_html_element_attrs(element: &mut Element) {
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
    element.value = element.attrs.get("value").cloned().unwrap_or_default();
    let len = element.value.chars().count();
    element.custom_validity_message.clear();
    element.selection_start = len;
    element.selection_end = len;
    element.selection_direction = "none".to_string();
}

fn is_javascript_url_attr(name: &str) -> bool {
    matches!(
        name,
        "href" | "src" | "xlink:href" | "action" | "formaction"
    )
}

fn is_javascript_scheme(value: &str) -> bool {
    let mut normalized = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_whitespace() || ch.is_ascii_control() {
            continue;
        }
        normalized.push(ch.to_ascii_lowercase());
    }
    normalized.starts_with("javascript:")
}

fn escape_html_text_for_serialization(value: &str) -> String {
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

fn escape_html_attr_for_serialization(value: &str) -> String {
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

fn class_tokens(class_attr: Option<&str>) -> Vec<String> {
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

fn set_class_attr(element: &mut Element, classes: &[String]) {
    if classes.is_empty() {
        element.attrs.remove("class");
    } else {
        element.attrs.insert("class".to_string(), classes.join(" "));
    }
}

fn dataset_key_to_attr_name(key: &str) -> String {
    format!("data-{}", js_prop_to_css_name(key))
}

fn js_prop_to_css_name(prop: &str) -> String {
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

fn parse_style_declarations(style_attr: Option<&str>) -> Vec<(String, String)> {
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
            (Some(q), _) if ch == b'\\' => {
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

fn push_style_declaration(raw_decl: &str, out: &mut Vec<(String, String)>) {
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
            (Some(q), _) if ch == b'\\' => {
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

fn serialize_style_declarations(decls: &[(String, String)]) -> String {
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

fn format_float(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value == f64::INFINITY {
        return "Infinity".to_string();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_string();
    }
    if value == 0.0 {
        return "0".to_string();
    }

    let raw = format!("{value}");
    let Some(exp_idx) = raw.find('e').or_else(|| raw.find('E')) else {
        return raw;
    };
    let mantissa = &raw[..exp_idx];
    let exponent_src = &raw[exp_idx + 1..];
    let exponent = exponent_src.parse::<i32>().unwrap_or(0);
    format!("{mantissa}e{:+}", exponent)
}

fn parse_js_parse_float(src: &str) -> f64 {
    let src = src.trim_start();
    if src.is_empty() {
        return f64::NAN;
    }

    let bytes = src.as_bytes();
    let mut i = 0usize;

    if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
        i += 1;
    }

    if src[i..].starts_with("Infinity") {
        return if matches!(bytes.first(), Some(b'-')) {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }

    let mut int_digits = 0usize;
    while matches!(bytes.get(i), Some(b) if b.is_ascii_digit()) {
        int_digits += 1;
        i += 1;
    }

    let mut frac_digits = 0usize;
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        while matches!(bytes.get(i), Some(b) if b.is_ascii_digit()) {
            frac_digits += 1;
            i += 1;
        }
    }

    if int_digits + frac_digits == 0 {
        return f64::NAN;
    }

    if matches!(bytes.get(i), Some(b'e') | Some(b'E')) {
        let exp_start = i;
        i += 1;
        if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }

        let mut exp_digits = 0usize;
        while matches!(bytes.get(i), Some(b) if b.is_ascii_digit()) {
            exp_digits += 1;
            i += 1;
        }

        if exp_digits == 0 {
            i = exp_start;
        }
    }

    src[..i].parse::<f64>().unwrap_or(f64::NAN)
}

fn parse_js_parse_int(src: &str, radix: Option<i64>) -> f64 {
    let src = src.trim_start();
    if src.is_empty() {
        return f64::NAN;
    }

    let bytes = src.as_bytes();
    let mut i = 0usize;
    let negative = if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
        let is_negative = bytes[i] == b'-';
        i += 1;
        is_negative
    } else {
        false
    };

    let mut radix = radix.unwrap_or(0);
    if radix != 0 {
        if !(2..=36).contains(&radix) {
            return f64::NAN;
        }
    } else {
        radix = 10;
        if src[i..].starts_with("0x") || src[i..].starts_with("0X") {
            radix = 16;
            i += 2;
        }
    }

    if radix == 16 && (src[i..].starts_with("0x") || src[i..].starts_with("0X")) {
        i += 2;
    }

    let mut parsed_any = false;
    let mut value = 0.0f64;
    for ch in src[i..].chars() {
        let Some(digit) = ch.to_digit(36) else {
            break;
        };
        if i64::from(digit) >= radix {
            break;
        }
        parsed_any = true;
        value = (value * (radix as f64)) + (digit as f64);
    }

    if !parsed_any {
        return f64::NAN;
    }

    if negative { -value } else { value }
}

fn encode_binary_string_to_base64(src: &str) -> Result<String> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut bytes = Vec::with_capacity(src.len());
    for ch in src.chars() {
        let code = ch as u32;
        if code > 0xFF {
            return Err(Error::ScriptRuntime(
                "btoa input contains non-Latin1 character".into(),
            ));
        }
        bytes.push(code as u8);
    }

    let mut out = String::new();
    let mut i = 0usize;
    while i + 3 <= bytes.len() {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];

        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(TABLE[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        out.push(TABLE[(b2 & 0x3F) as usize] as char);
        i += 3;
    }

    let rem = bytes.len().saturating_sub(i);
    if rem == 1 {
        let b0 = bytes[i];
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[((b0 & 0x03) << 4) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(TABLE[((b1 & 0x0F) << 2) as usize] as char);
        out.push('=');
    }

    Ok(out)
}

fn decode_base64_to_binary_string(src: &str) -> Result<String> {
    let mut bytes: Vec<u8> = src.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if bytes.is_empty() {
        return Ok(String::new());
    }

    match bytes.len() % 4 {
        0 => {}
        2 => bytes.extend_from_slice(b"=="),
        3 => bytes.push(b'='),
        _ => {
            return Err(Error::ScriptRuntime("atob invalid base64 input".into()));
        }
    }

    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];
        let b3 = bytes[i + 3];

        let v0 = decode_base64_char(b0)?;
        let v1 = decode_base64_char(b1)?;
        out.push((v0 << 2) | (v1 >> 4));

        if b2 == b'=' {
            if b3 != b'=' {
                return Err(Error::ScriptRuntime("atob invalid base64 input".into()));
            }
            i += 4;
            continue;
        }

        let v2 = decode_base64_char(b2)?;
        out.push(((v1 & 0x0F) << 4) | (v2 >> 2));

        if b3 == b'=' {
            i += 4;
            continue;
        }

        let v3 = decode_base64_char(b3)?;
        out.push(((v2 & 0x03) << 6) | v3);
        i += 4;
    }

    Ok(out.into_iter().map(char::from).collect())
}

fn decode_base64_char(ch: u8) -> Result<u8> {
    let value = match ch {
        b'A'..=b'Z' => ch - b'A',
        b'a'..=b'z' => ch - b'a' + 26,
        b'0'..=b'9' => ch - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => {
            return Err(Error::ScriptRuntime("atob invalid base64 input".into()));
        }
    };
    Ok(value)
}

fn encode_uri_like(src: &str, component: bool) -> String {
    let mut out = String::new();
    for b in src.as_bytes() {
        if is_unescaped_uri_byte(*b, component) {
            out.push(*b as char);
        } else {
            out.push('%');
            out.push(to_hex_upper((*b >> 4) & 0x0F));
            out.push(to_hex_upper(*b & 0x0F));
        }
    }
    out
}

fn encode_uri_like_preserving_percent(src: &str, component: bool) -> String {
    let mut out = String::new();
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && from_hex_digit(bytes[i + 1]).is_some()
            && from_hex_digit(bytes[i + 2]).is_some()
        {
            out.push('%');
            out.push((bytes[i + 1] as char).to_ascii_uppercase());
            out.push((bytes[i + 2] as char).to_ascii_uppercase());
            i += 3;
            continue;
        }

        let ch = src[i..].chars().next().unwrap_or_default();
        let mut encoded = [0u8; 4];
        let encoded = ch.encode_utf8(&mut encoded);
        for b in encoded.as_bytes() {
            if is_unescaped_uri_byte(*b, component) {
                out.push(*b as char);
            } else {
                out.push('%');
                out.push(to_hex_upper((*b >> 4) & 0x0F));
                out.push(to_hex_upper(*b & 0x0F));
            }
        }
        i += ch.len_utf8();
    }
    out
}

fn decode_uri_like(src: &str, component: bool) -> Result<String> {
    let preserve_reserved = !component;
    let bytes = src.as_bytes();
    let mut out = String::new();
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] != b'%' {
            let ch = src[i..]
                .chars()
                .next()
                .ok_or_else(|| Error::ScriptRuntime("malformed URI sequence".into()))?;
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        let first = parse_percent_byte(bytes, i)?;
        if first < 0x80 {
            let ch = first as char;
            if preserve_reserved && is_decode_uri_reserved_char(ch) {
                let raw = src
                    .get(i..i + 3)
                    .ok_or_else(|| Error::ScriptRuntime("malformed URI sequence".into()))?;
                out.push_str(raw);
            } else {
                out.push(ch);
            }
            i += 3;
            continue;
        }

        let len = utf8_sequence_len(first)
            .ok_or_else(|| Error::ScriptRuntime("malformed URI sequence".into()))?;
        let mut raw_end = i + 3;
        let mut chunk = Vec::with_capacity(len);
        chunk.push(first);
        for _ in 1..len {
            if raw_end >= bytes.len() || bytes[raw_end] != b'%' {
                return Err(Error::ScriptRuntime("malformed URI sequence".into()));
            }
            chunk.push(parse_percent_byte(bytes, raw_end)?);
            raw_end += 3;
        }
        let decoded = std::str::from_utf8(&chunk)
            .map_err(|_| Error::ScriptRuntime("malformed URI sequence".into()))?;
        out.push_str(decoded);
        i = raw_end;
    }

    Ok(out)
}

fn parse_url_search_params_pairs_from_query_string(query: &str) -> Result<Vec<(String, String)>> {
    let query = query.strip_prefix('?').unwrap_or(query);
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let mut pairs = Vec::new();
    for part in query.split('&') {
        if part.is_empty() {
            continue;
        }
        let (raw_name, raw_value) = if let Some((name, value)) = part.split_once('=') {
            (name, value)
        } else {
            (part, "")
        };
        let name = decode_form_urlencoded_component(raw_name)?;
        let value = decode_form_urlencoded_component(raw_value)?;
        pairs.push((name, value));
    }
    Ok(pairs)
}

fn serialize_url_search_params_pairs(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
        .map(|(name, value)| {
            format!(
                "{}={}",
                encode_form_urlencoded_component(name),
                encode_form_urlencoded_component(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn encode_form_urlencoded_component(src: &str) -> String {
    let mut out = String::new();
    for b in src.as_bytes() {
        if is_form_urlencoded_unescaped_byte(*b) {
            out.push(*b as char);
        } else if *b == b' ' {
            out.push('+');
        } else {
            out.push('%');
            out.push(to_hex_upper((*b >> 4) & 0x0F));
            out.push(to_hex_upper(*b & 0x0F));
        }
    }
    out
}

fn decode_form_urlencoded_component(src: &str) -> Result<String> {
    let bytes = src.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams malformed percent-encoding".into(),
                    ));
                }
                let hi = from_hex_digit(bytes[i + 1]).ok_or_else(|| {
                    Error::ScriptRuntime("URLSearchParams malformed percent-encoding".into())
                })?;
                let lo = from_hex_digit(bytes[i + 2]).ok_or_else(|| {
                    Error::ScriptRuntime("URLSearchParams malformed percent-encoding".into())
                })?;
                out.push((hi << 4) | lo);
                i += 3;
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8(out)
        .map_err(|_| Error::ScriptRuntime("URLSearchParams malformed UTF-8 sequence".into()))
}

fn is_form_urlencoded_unescaped_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'*' | b'-' | b'.' | b'_')
}

fn js_escape(src: &str) -> String {
    let mut out = String::new();
    for unit in src.encode_utf16() {
        if unit <= 0x7F && is_unescaped_legacy_escape_byte(unit as u8) {
            out.push(unit as u8 as char);
            continue;
        }

        if unit <= 0xFF {
            let value = unit as u8;
            out.push('%');
            out.push(to_hex_upper((value >> 4) & 0x0F));
            out.push(to_hex_upper(value & 0x0F));
            continue;
        }

        out.push('%');
        out.push('u');
        out.push(to_hex_upper(((unit >> 12) & 0x0F) as u8));
        out.push(to_hex_upper(((unit >> 8) & 0x0F) as u8));
        out.push(to_hex_upper(((unit >> 4) & 0x0F) as u8));
        out.push(to_hex_upper((unit & 0x0F) as u8));
    }
    out
}

fn js_unescape(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut units: Vec<u16> = Vec::with_capacity(src.len());
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 5 < bytes.len()
                && matches!(bytes[i + 1], b'u' | b'U')
                && from_hex_digit(bytes[i + 2]).is_some()
                && from_hex_digit(bytes[i + 3]).is_some()
                && from_hex_digit(bytes[i + 4]).is_some()
                && from_hex_digit(bytes[i + 5]).is_some()
            {
                let u = ((from_hex_digit(bytes[i + 2]).unwrap_or(0) as u16) << 12)
                    | ((from_hex_digit(bytes[i + 3]).unwrap_or(0) as u16) << 8)
                    | ((from_hex_digit(bytes[i + 4]).unwrap_or(0) as u16) << 4)
                    | (from_hex_digit(bytes[i + 5]).unwrap_or(0) as u16);
                units.push(u);
                i += 6;
                continue;
            }

            if i + 2 < bytes.len()
                && from_hex_digit(bytes[i + 1]).is_some()
                && from_hex_digit(bytes[i + 2]).is_some()
            {
                let u = ((from_hex_digit(bytes[i + 1]).unwrap_or(0) << 4)
                    | from_hex_digit(bytes[i + 2]).unwrap_or(0)) as u16;
                units.push(u);
                i += 3;
                continue;
            }
        }

        let ch = src[i..].chars().next().unwrap_or_default();
        let mut buf = [0u16; 2];
        for unit in ch.encode_utf16(&mut buf).iter().copied() {
            units.push(unit);
        }
        i += ch.len_utf8();
    }

    String::from_utf16_lossy(&units)
}

fn is_unescaped_uri_byte(b: u8, component: bool) -> bool {
    if b.is_ascii_alphanumeric() {
        return true;
    }
    if matches!(
        b,
        b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')'
    ) {
        return true;
    }
    if !component
        && matches!(
            b,
            b';' | b',' | b'/' | b'?' | b':' | b'@' | b'&' | b'=' | b'+' | b'$' | b'#'
        )
    {
        return true;
    }
    false
}

fn is_decode_uri_reserved_char(ch: char) -> bool {
    matches!(
        ch,
        ';' | ',' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | '#'
    )
}

fn is_unescaped_legacy_escape_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'*' | b'+' | b'-' | b'.' | b'/' | b'@' | b'_')
}

fn parse_percent_byte(bytes: &[u8], offset: usize) -> Result<u8> {
    if offset + 2 >= bytes.len() || bytes[offset] != b'%' {
        return Err(Error::ScriptRuntime("malformed URI sequence".into()));
    }
    let hi = from_hex_digit(bytes[offset + 1])
        .ok_or_else(|| Error::ScriptRuntime("malformed URI sequence".into()))?;
    let lo = from_hex_digit(bytes[offset + 2])
        .ok_or_else(|| Error::ScriptRuntime("malformed URI sequence".into()))?;
    Ok((hi << 4) | lo)
}

fn utf8_sequence_len(first: u8) -> Option<usize> {
    match first {
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

fn from_hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn to_hex_upper(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'A' + (nibble - 10)) as char,
        _ => '?',
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut it = value.chars();
    let mut out = String::new();
    for _ in 0..max_chars {
        let Some(ch) = it.next() else {
            return out;
        };
        out.push(ch);
    }
    if it.next().is_some() {
        out.push_str("...");
    }
    out
}

