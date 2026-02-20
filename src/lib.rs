//! Deterministic in-memory browser runtime for Rust tests.
//!
//! This crate provides a lightweight DOM and JavaScript-like runtime tailored
//! for deterministic unit and integration testing.
//! Use [`Harness`] as the main entry point to load HTML, simulate user actions,
//! control fake time, and assert DOM state.

use num_bigint::{BigInt as JsBigInt, Sign};
use num_traits::{One, ToPrimitive, Zero};
use regex::{Captures, Regex, RegexBuilder};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error as StdError;
use std::fmt;
use std::rc::Rc;

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

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectorAttrCondition {
    Exists { key: String },
    Eq { key: String, value: String },
    StartsWith { key: String, value: String },
    EndsWith { key: String, value: String },
    Contains { key: String, value: String },
    Includes { key: String, value: String },
    DashMatch { key: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectorPseudoClass {
    FirstChild,
    LastChild,
    FirstOfType,
    LastOfType,
    OnlyChild,
    OnlyOfType,
    Checked,
    Indeterminate,
    Disabled,
    Enabled,
    Required,
    Optional,
    Readonly,
    Readwrite,
    Empty,
    Focus,
    FocusWithin,
    Active,
    NthOfType(NthChildSelector),
    NthLastOfType(NthChildSelector),
    Not(Vec<Vec<SelectorPart>>),
    Is(Vec<Vec<SelectorPart>>),
    Where(Vec<Vec<SelectorPart>>),
    Has(Vec<Vec<SelectorPart>>),
    NthChild(NthChildSelector),
    NthLastChild(NthChildSelector),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NthChildSelector {
    Exact(usize),
    Odd,
    Even,
    AnPlusB(i64, i64),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SelectorStep {
    tag: Option<String>,
    universal: bool,
    id: Option<String>,
    classes: Vec<String>,
    attrs: Vec<SelectorAttrCondition>,
    pseudo_classes: Vec<SelectorPseudoClass>,
}

impl SelectorStep {
    fn id_only(&self) -> Option<&str> {
        if !self.universal
            && self.tag.is_none()
            && self.classes.is_empty()
            && self.attrs.is_empty()
            && self.pseudo_classes.is_empty()
        {
            self.id.as_deref()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectorCombinator {
    Descendant,
    Child,
    AdjacentSibling,
    GeneralSibling,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectorPart {
    step: SelectorStep,
    // Relation to previous (left) selector part.
    combinator: Option<SelectorCombinator>,
}

fn parse_selector_chain(selector: &str) -> Result<Vec<SelectorPart>> {
    let selector = selector.trim();
    if selector.is_empty() {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    let tokens = tokenize_selector(selector)?;
    let mut steps = Vec::new();
    let mut pending_combinator: Option<SelectorCombinator> = None;

    for token in tokens {
        if token == ">" || token == "+" || token == "~" {
            if pending_combinator.is_some() || steps.is_empty() {
                return Err(Error::UnsupportedSelector(selector.into()));
            }
            pending_combinator = Some(match token.as_str() {
                ">" => SelectorCombinator::Child,
                "+" => SelectorCombinator::AdjacentSibling,
                "~" => SelectorCombinator::GeneralSibling,
                _ => unreachable!(),
            });
            continue;
        }

        let step = parse_selector_step(&token)?;
        let combinator = if steps.is_empty() {
            None
        } else {
            Some(
                pending_combinator
                    .take()
                    .unwrap_or(SelectorCombinator::Descendant),
            )
        };
        steps.push(SelectorPart { step, combinator });
    }

    if steps.is_empty() || pending_combinator.is_some() {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    Ok(steps)
}

fn parse_selector_groups(selector: &str) -> Result<Vec<Vec<SelectorPart>>> {
    let groups = split_selector_groups(selector)?;
    let mut parsed = Vec::with_capacity(groups.len());
    for group in groups {
        parsed.push(parse_selector_chain(&group)?);
    }
    Ok(parsed)
}

fn split_selector_groups(selector: &str) -> Result<Vec<String>> {
    let mut groups = Vec::new();
    let mut current = String::new();
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;

    for ch in selector.chars() {
        match ch {
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                if bracket_depth == 0 {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                bracket_depth -= 1;
                current.push(ch);
            }
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                if paren_depth == 0 {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                paren_depth -= 1;
                current.push(ch);
            }
            ',' if bracket_depth == 0 && paren_depth == 0 => {
                let trimmed = current.trim();
                if trimmed.is_empty() {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                groups.push(trimmed.to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if bracket_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }
    if paren_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    let trimmed = current.trim();
    if trimmed.is_empty() {
        return Err(Error::UnsupportedSelector(selector.into()));
    }
    groups.push(trimmed.to_string());
    Ok(groups)
}

fn tokenize_selector(selector: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;

    for ch in selector.chars() {
        match ch {
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                if bracket_depth == 0 {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                bracket_depth -= 1;
                current.push(ch);
            }
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                if paren_depth == 0 {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                paren_depth -= 1;
                current.push(ch);
            }
            '>' | '+' | '~' if bracket_depth == 0 && paren_depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                }
                current.clear();
                tokens.push(ch.to_string());
            }
            ch if ch.is_ascii_whitespace() && bracket_depth == 0 && paren_depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if bracket_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }
    if paren_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    Ok(tokens)
}

fn parse_selector_step(part: &str) -> Result<SelectorStep> {
    let part = part.trim();
    if part.is_empty() {
        return Err(Error::UnsupportedSelector(part.into()));
    }

    let bytes = part.as_bytes();
    let mut i = 0usize;
    let mut step = SelectorStep::default();

    while i < bytes.len() {
        match bytes[i] {
            b'*' => {
                if step.universal {
                    return Err(Error::UnsupportedSelector(part.into()));
                }
                step.universal = true;
                i += 1;
            }
            b'#' => {
                i += 1;
                let Some((id, next)) = parse_selector_ident(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                if step.id.replace(id).is_some() {
                    return Err(Error::UnsupportedSelector(part.into()));
                }
                i = next;
            }
            b'.' => {
                i += 1;
                let Some((class_name, next)) = parse_selector_ident(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                step.classes.push(class_name);
                i = next;
            }
            b'[' => {
                let (attr, next) = parse_selector_attr_condition(part, i)?;
                step.attrs.push(attr);
                i = next;
            }
            b':' => {
                let Some((pseudo, next)) = parse_selector_pseudo(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                step.pseudo_classes.push(pseudo);
                i = next;
            }
            _ => {
                if step.tag.is_some()
                    || step.id.is_some()
                    || !step.classes.is_empty()
                    || step.universal
                {
                    return Err(Error::UnsupportedSelector(part.into()));
                }
                let Some((tag, next)) = parse_selector_ident(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                step.tag = Some(tag);
                i = next;
            }
        }
    }

    if step.tag.is_none()
        && step.id.is_none()
        && step.classes.is_empty()
        && step.attrs.is_empty()
        && !step.universal
        && step.pseudo_classes.is_empty()
    {
        return Err(Error::UnsupportedSelector(part.into()));
    }
    Ok(step)
}

fn parse_selector_pseudo(part: &str, start: usize) -> Option<(SelectorPseudoClass, usize)> {
    if part.as_bytes().get(start)? != &b':' {
        return None;
    }
    let start = start + 1;
    let tail = part.get(start..)?;
    if let Some(rest) = tail.strip_prefix("first-child") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "first-child".len();
            return Some((SelectorPseudoClass::FirstChild, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("last-child") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "last-child".len();
            return Some((SelectorPseudoClass::LastChild, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("first-of-type") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "first-of-type".len();
            return Some((SelectorPseudoClass::FirstOfType, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("last-of-type") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "last-of-type".len();
            return Some((SelectorPseudoClass::LastOfType, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("only-child") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "only-child".len();
            return Some((SelectorPseudoClass::OnlyChild, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("only-of-type") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "only-of-type".len();
            return Some((SelectorPseudoClass::OnlyOfType, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("checked") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "checked".len();
            return Some((SelectorPseudoClass::Checked, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("indeterminate") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "indeterminate".len();
            return Some((SelectorPseudoClass::Indeterminate, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("disabled") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "disabled".len();
            return Some((SelectorPseudoClass::Disabled, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("required") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "required".len();
            return Some((SelectorPseudoClass::Required, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("optional") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "optional".len();
            return Some((SelectorPseudoClass::Optional, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("read-only") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "read-only".len();
            return Some((SelectorPseudoClass::Readonly, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("readonly") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "readonly".len();
            return Some((SelectorPseudoClass::Readonly, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("read-write") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "read-write".len();
            return Some((SelectorPseudoClass::Readwrite, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("empty") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "empty".len();
            return Some((SelectorPseudoClass::Empty, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("focus-within") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "focus-within".len();
            return Some((SelectorPseudoClass::FocusWithin, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("focus") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "focus".len();
            return Some((SelectorPseudoClass::Focus, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("active") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "active".len();
            return Some((SelectorPseudoClass::Active, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("enabled") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "enabled".len();
            return Some((SelectorPseudoClass::Enabled, consumed));
        }
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "not(") {
        return Some((SelectorPseudoClass::Not(inners), next));
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "is(") {
        return Some((SelectorPseudoClass::Is(inners), next));
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "where(") {
        return Some((SelectorPseudoClass::Where(inners), next));
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "has(") {
        return Some((SelectorPseudoClass::Has(inners), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-last-of-type(") {
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-last-of-type(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthLastOfType(selector), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-of-type(") {
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-of-type(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthOfType(selector), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-last-child(") {
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-last-child(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthLastChild(selector), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-child(") {
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-child(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthChild(selector), next));
    }

    None
}

fn parse_pseudo_selector_list(
    part: &str,
    start: usize,
    prefix: &str,
) -> Option<(Vec<Vec<SelectorPart>>, usize)> {
    let Some(rest) = part.get(start..).and_then(|tail| tail.strip_prefix(prefix)) else {
        return None;
    };

    let Some(close_pos) = find_matching_paren(rest) else {
        return None;
    };
    let body = rest[..close_pos].trim();
    if body.is_empty() {
        return None;
    }

    let mut groups = split_selector_groups(body).ok()?;
    if groups.is_empty() {
        return None;
    }

    let mut selectors = Vec::with_capacity(groups.len());
    for group in &mut groups {
        let chain = parse_selector_chain(group.trim()).ok()?;
        if chain.is_empty() {
            return None;
        }
        selectors.push(chain);
    }

    let next = start + prefix.len() + close_pos + 1;
    if let Some(ch) = part.as_bytes().get(next) {
        if !is_selector_continuation(ch) {
            return None;
        }
    }
    Some((selectors, next))
}

fn find_matching_paren(body: &str) -> Option<usize> {
    let mut paren_depth = 1usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<u8> = None;
    let mut escaped = false;

    for (idx, b) in body.bytes().enumerate() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if b == b'\\' {
                escaped = true;
                continue;
            }
            if b == q {
                quote = None;
            }
            continue;
        }

        match b {
            b'\'' | b'"' => quote = Some(b),
            b'[' => {
                bracket_depth += 1;
            }
            b']' => {
                if bracket_depth == 0 {
                    return None;
                }
                bracket_depth -= 1;
            }
            b'(' if bracket_depth == 0 => {
                paren_depth += 1;
            }
            b')' if bracket_depth == 0 => {
                paren_depth = paren_depth.checked_sub(1)?;
                if paren_depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_nth_child_selector(raw: &str) -> Option<NthChildSelector> {
    let compact = raw
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    if compact.is_empty() {
        return None;
    }

    match compact.as_str() {
        "odd" => Some(NthChildSelector::Odd),
        "even" => Some(NthChildSelector::Even),
        other => {
            if other.contains('n') {
                parse_nth_child_expression(other)
            } else {
                if other.starts_with('+') || other.starts_with('-') {
                    None
                } else {
                    let value = other.parse::<usize>().ok()?;
                    if value == 0 {
                        None
                    } else {
                        Some(NthChildSelector::Exact(value))
                    }
                }
            }
        }
    }
}

fn parse_nth_child_expression(raw: &str) -> Option<NthChildSelector> {
    let expr = raw
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>();
    let expr = expr.to_ascii_lowercase();
    if expr.matches('n').count() != 1 {
        return None;
    }
    if expr.starts_with(|c: char| c == '+' || c == '-') && expr.len() == 1 {
        return None;
    }

    let n_pos = expr.find('n')?;
    let (a_part, rest) = expr.split_at(n_pos);
    let b_part = &rest[1..];

    let a = match a_part {
        "" => 1,
        "-" => -1,
        "+" => return None,
        _ => a_part.parse::<i64>().ok()?,
    };

    if b_part.is_empty() {
        return Some(NthChildSelector::AnPlusB(a, 0));
    }

    let mut sign = 1;
    let raw_b = if let Some(rest) = b_part.strip_prefix('+') {
        rest
    } else if let Some(rest) = b_part.strip_prefix('-') {
        sign = -1;
        rest
    } else {
        return None;
    };
    if raw_b.is_empty() {
        return None;
    }
    let b = raw_b.parse::<i64>().ok()?;
    Some(NthChildSelector::AnPlusB(a, b * sign))
}

fn is_selector_continuation(next: &u8) -> bool {
    matches!(next, b'.' | b'#' | b'[' | b':')
}

fn parse_selector_ident(src: &str, start: usize) -> Option<(String, usize)> {
    let bytes = src.as_bytes();
    if start >= bytes.len() || !is_selector_ident_char(bytes[start]) {
        return None;
    }
    let mut end = start + 1;
    while end < bytes.len() && is_selector_ident_char(bytes[end]) {
        end += 1;
    }
    Some((src.get(start..end)?.to_string(), end))
}

fn is_selector_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

fn parse_selector_attr_condition(
    src: &str,
    open_bracket: usize,
) -> Result<(SelectorAttrCondition, usize)> {
    let bytes = src.as_bytes();
    let mut i = open_bracket + 1;

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let key_start = i;
    while i < bytes.len() {
        if is_selector_attr_name_char(bytes[i]) {
            i += 1;
            continue;
        }
        break;
    }
    if key_start == i {
        return Err(Error::UnsupportedSelector(src.into()));
    }
    let key = src
        .get(key_start..i)
        .ok_or_else(|| Error::UnsupportedSelector(src.into()))?
        .to_ascii_lowercase();

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    if bytes[i] == b']' {
        return Ok((SelectorAttrCondition::Exists { key }, i + 1));
    }

    let (op, mut next) = match bytes.get(i) {
        Some(b'=') => (SelectorAttrConditionType::Eq, i + 1),
        Some(b'^') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::StartsWith, i + 2)
        }
        Some(b'$') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::EndsWith, i + 2)
        }
        Some(b'*') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::Contains, i + 2)
        }
        Some(b'~') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::Includes, i + 2)
        }
        Some(b'|') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::DashMatch, i + 2)
        }
        _ => return Err(Error::UnsupportedSelector(src.into())),
    };

    i = next;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let (value, after_value) = parse_selector_attr_value(src, i)?;
    next = after_value;

    i = next;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b']' {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let cond = match op {
        SelectorAttrConditionType::Eq => SelectorAttrCondition::Eq { key, value },
        SelectorAttrConditionType::StartsWith => SelectorAttrCondition::StartsWith { key, value },
        SelectorAttrConditionType::EndsWith => SelectorAttrCondition::EndsWith { key, value },
        SelectorAttrConditionType::Contains => SelectorAttrCondition::Contains { key, value },
        SelectorAttrConditionType::Includes => SelectorAttrCondition::Includes { key, value },
        SelectorAttrConditionType::DashMatch => SelectorAttrCondition::DashMatch { key, value },
    };

    Ok((cond, i + 1))
}

#[derive(Debug, Clone, Copy)]
enum SelectorAttrConditionType {
    Eq,
    StartsWith,
    EndsWith,
    Contains,
    Includes,
    DashMatch,
}

fn is_selector_attr_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b':'
}

fn parse_selector_attr_value(src: &str, start: usize) -> Result<(String, usize)> {
    let bytes = src.as_bytes();
    if start >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    if bytes[start] == b'"' || bytes[start] == b'\'' {
        let quote = bytes[start];
        let mut i = start + 1;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i = (i + 2).min(bytes.len());
                continue;
            }
            if bytes[i] == quote {
                let raw = src
                    .get(start + 1..i)
                    .ok_or_else(|| Error::UnsupportedSelector(src.into()))?;
                return Ok((unescape_string(raw), i + 1));
            }
            i += 1;
        }
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let start_value = start;
    let mut i = start;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() || bytes[i] == b']' {
            break;
        }
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }
        i += 1;
    }
    if i == start_value {
        return Ok(("".to_string(), i));
    }
    let raw = src
        .get(start_value..i)
        .ok_or_else(|| Error::UnsupportedSelector(src.into()))?;
    Ok((unescape_string(raw), i))
}

#[derive(Debug, Clone, PartialEq)]
struct ArrayBufferValue {
    bytes: Vec<u8>,
    max_byte_length: Option<usize>,
    detached: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlobValue {
    bytes: Vec<u8>,
    mime_type: String,
}

impl ArrayBufferValue {
    fn byte_length(&self) -> usize {
        if self.detached { 0 } else { self.bytes.len() }
    }

    fn max_byte_length(&self) -> usize {
        if self.detached {
            0
        } else {
            self.max_byte_length.unwrap_or(self.bytes.len())
        }
    }

    fn resizable(&self) -> bool {
        !self.detached && self.max_byte_length.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypedArrayKind {
    Int8,
    Uint8,
    Uint8Clamped,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float16,
    Float32,
    Float64,
    BigInt64,
    BigUint64,
}

impl TypedArrayKind {
    fn bytes_per_element(&self) -> usize {
        match self {
            Self::Int8 | Self::Uint8 | Self::Uint8Clamped => 1,
            Self::Int16 | Self::Uint16 | Self::Float16 => 2,
            Self::Int32 | Self::Uint32 | Self::Float32 => 4,
            Self::Float64 | Self::BigInt64 | Self::BigUint64 => 8,
        }
    }

    fn is_bigint(&self) -> bool {
        matches!(self, Self::BigInt64 | Self::BigUint64)
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Int8 => "Int8Array",
            Self::Uint8 => "Uint8Array",
            Self::Uint8Clamped => "Uint8ClampedArray",
            Self::Int16 => "Int16Array",
            Self::Uint16 => "Uint16Array",
            Self::Int32 => "Int32Array",
            Self::Uint32 => "Uint32Array",
            Self::Float16 => "Float16Array",
            Self::Float32 => "Float32Array",
            Self::Float64 => "Float64Array",
            Self::BigInt64 => "BigInt64Array",
            Self::BigUint64 => "BigUint64Array",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TypedArrayConstructorKind {
    Concrete(TypedArrayKind),
    Abstract,
}

#[derive(Debug, Clone, PartialEq)]
struct TypedArrayValue {
    kind: TypedArrayKind,
    buffer: Rc<RefCell<ArrayBufferValue>>,
    byte_offset: usize,
    fixed_length: Option<usize>,
}

impl TypedArrayValue {
    fn observed_length(&self) -> usize {
        let buffer_len = self.buffer.borrow().byte_length();
        let bytes_per = self.kind.bytes_per_element();
        if self.byte_offset >= buffer_len {
            return 0;
        }

        let available_bytes = buffer_len - self.byte_offset;
        if let Some(fixed_length) = self.fixed_length {
            let fixed_bytes = fixed_length.saturating_mul(bytes_per);
            if available_bytes < fixed_bytes {
                0
            } else {
                fixed_length
            }
        } else {
            available_bytes / bytes_per
        }
    }

    fn observed_byte_length(&self) -> usize {
        self.observed_length()
            .saturating_mul(self.kind.bytes_per_element())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MapValue {
    entries: Vec<(Value, Value)>,
    properties: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq)]
struct SetValue {
    values: Vec<Value>,
    properties: Vec<(String, Value)>,
}

#[derive(Debug, Clone)]
struct PromiseValue {
    id: usize,
    state: PromiseState,
    reactions: Vec<PromiseReaction>,
}

impl PartialEq for PromiseValue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone)]
enum PromiseState {
    Pending,
    Fulfilled(Value),
    Rejected(Value),
}

#[derive(Debug, Clone)]
struct PromiseReaction {
    kind: PromiseReactionKind,
}

#[derive(Debug, Clone)]
enum PromiseReactionKind {
    Then {
        on_fulfilled: Option<Value>,
        on_rejected: Option<Value>,
        result: Rc<RefCell<PromiseValue>>,
    },
    Finally {
        callback: Option<Value>,
        result: Rc<RefCell<PromiseValue>>,
    },
    FinallyContinuation {
        original: PromiseSettledValue,
        result: Rc<RefCell<PromiseValue>>,
    },
    ResolveTo {
        target: Rc<RefCell<PromiseValue>>,
    },
    All {
        state: Rc<RefCell<PromiseAllState>>,
        index: usize,
    },
    AllSettled {
        state: Rc<RefCell<PromiseAllSettledState>>,
        index: usize,
    },
    Any {
        state: Rc<RefCell<PromiseAnyState>>,
        index: usize,
    },
    Race {
        state: Rc<RefCell<PromiseRaceState>>,
    },
}

#[derive(Debug, Clone)]
enum PromiseSettledValue {
    Fulfilled(Value),
    Rejected(Value),
}

#[derive(Debug, Clone)]
struct PromiseAllState {
    result: Rc<RefCell<PromiseValue>>,
    remaining: usize,
    values: Vec<Option<Value>>,
    settled: bool,
}

#[derive(Debug, Clone)]
struct PromiseAllSettledState {
    result: Rc<RefCell<PromiseValue>>,
    remaining: usize,
    values: Vec<Option<Value>>,
}

#[derive(Debug, Clone)]
struct PromiseAnyState {
    result: Rc<RefCell<PromiseValue>>,
    remaining: usize,
    reasons: Vec<Option<Value>>,
    settled: bool,
}

#[derive(Debug, Clone)]
struct PromiseRaceState {
    result: Rc<RefCell<PromiseValue>>,
    settled: bool,
}

#[derive(Debug, Clone)]
struct PromiseCapabilityFunction {
    promise: Rc<RefCell<PromiseValue>>,
    reject: bool,
    already_called: Rc<RefCell<bool>>,
}

impl PartialEq for PromiseCapabilityFunction {
    fn eq(&self, other: &Self) -> bool {
        self.reject == other.reject
            && self.promise.borrow().id == other.promise.borrow().id
            && Rc::ptr_eq(&self.already_called, &other.already_called)
    }
}

#[derive(Debug, Clone)]
struct SymbolValue {
    id: usize,
    description: Option<String>,
    registry_key: Option<String>,
}

impl PartialEq for SymbolValue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    String(String),
    StringConstructor,
    Bool(bool),
    Number(i64),
    Float(f64),
    BigInt(JsBigInt),
    Array(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<Vec<(String, Value)>>>),
    Promise(Rc<RefCell<PromiseValue>>),
    Map(Rc<RefCell<MapValue>>),
    Set(Rc<RefCell<SetValue>>),
    Blob(Rc<RefCell<BlobValue>>),
    ArrayBuffer(Rc<RefCell<ArrayBufferValue>>),
    TypedArray(Rc<RefCell<TypedArrayValue>>),
    TypedArrayConstructor(TypedArrayConstructorKind),
    BlobConstructor,
    UrlConstructor,
    ArrayBufferConstructor,
    PromiseConstructor,
    MapConstructor,
    SetConstructor,
    SymbolConstructor,
    RegExpConstructor,
    PromiseCapability(Rc<PromiseCapabilityFunction>),
    Symbol(Rc<SymbolValue>),
    RegExp(Rc<RefCell<RegexValue>>),
    Date(Rc<RefCell<i64>>),
    Null,
    Undefined,
    Node(NodeId),
    NodeList(Vec<NodeId>),
    FormData(Vec<(String, String)>),
    Function(Rc<FunctionValue>),
}

#[derive(Debug, Clone)]
struct RegexValue {
    source: String,
    flags: String,
    global: bool,
    ignore_case: bool,
    multiline: bool,
    dot_all: bool,
    sticky: bool,
    has_indices: bool,
    unicode: bool,
    compiled: Regex,
    last_index: usize,
    properties: Vec<(String, Value)>,
}

impl PartialEq for RegexValue {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.flags == other.flags
            && self.global == other.global
            && self.ignore_case == other.ignore_case
            && self.multiline == other.multiline
            && self.dot_all == other.dot_all
            && self.sticky == other.sticky
            && self.has_indices == other.has_indices
            && self.unicode == other.unicode
            && self.last_index == other.last_index
            && self.properties == other.properties
    }
}

#[derive(Debug, Clone)]
struct FunctionValue {
    handler: ScriptHandler,
    captured_env: HashMap<String, Value>,
    captured_global_names: HashSet<String>,
    local_bindings: HashSet<String>,
    global_scope: bool,
    is_async: bool,
}

impl PartialEq for FunctionValue {
    fn eq(&self, other: &Self) -> bool {
        self.handler == other.handler
            && self.global_scope == other.global_scope
            && self.is_async == other.is_async
    }
}

#[derive(Debug, Clone, Copy)]
struct RegexFlags {
    global: bool,
    ignore_case: bool,
    multiline: bool,
    dot_all: bool,
    sticky: bool,
    has_indices: bool,
    unicode: bool,
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::String(v) => !v.is_empty(),
            Self::StringConstructor => true,
            Self::Number(v) => *v != 0,
            Self::Float(v) => *v != 0.0,
            Self::BigInt(v) => !v.is_zero(),
            Self::Array(_) => true,
            Self::Object(_) => true,
            Self::Promise(_) => true,
            Self::Map(_) => true,
            Self::Set(_) => true,
            Self::Blob(_) => true,
            Self::ArrayBuffer(_) => true,
            Self::TypedArray(_) => true,
            Self::TypedArrayConstructor(_) => true,
            Self::BlobConstructor => true,
            Self::UrlConstructor => true,
            Self::ArrayBufferConstructor => true,
            Self::PromiseConstructor => true,
            Self::MapConstructor => true,
            Self::SetConstructor => true,
            Self::SymbolConstructor => true,
            Self::RegExpConstructor => true,
            Self::PromiseCapability(_) => true,
            Self::Symbol(_) => true,
            Self::RegExp(_) => true,
            Self::Date(_) => true,
            Self::Null => false,
            Self::Undefined => false,
            Self::Node(_) => true,
            Self::NodeList(nodes) => !nodes.is_empty(),
            Self::FormData(_) => true,
            Self::Function(_) => true,
        }
    }

    fn as_string(&self) -> String {
        match self {
            Self::String(v) => v.clone(),
            Self::StringConstructor => "String".to_string(),
            Self::Bool(v) => {
                if *v {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            Self::Number(v) => v.to_string(),
            Self::Float(v) => format_float(*v),
            Self::BigInt(v) => v.to_string(),
            Self::Array(values) => {
                let values = values.borrow();
                let mut out = String::new();
                for (idx, value) in values.iter().enumerate() {
                    if idx > 0 {
                        out.push(',');
                    }
                    if matches!(value, Value::Null | Value::Undefined) {
                        continue;
                    }
                    out.push_str(&value.as_string());
                }
                out
            }
            Self::Object(entries) => {
                let entries = entries.borrow();
                match entries.iter().find_map(|(key, value)| {
                    (key == INTERNAL_STRING_WRAPPER_VALUE_KEY).then(|| value)
                }) {
                    Some(Value::String(value)) => value.clone(),
                    _ => {
                        let is_url = entries.iter().any(|(key, value)| {
                            key == INTERNAL_URL_OBJECT_KEY && matches!(value, Value::Bool(true))
                        });
                        if is_url {
                            if let Some(Value::String(href)) = entries
                                .iter()
                                .find_map(|(key, value)| (key == "href").then(|| value))
                            {
                                return href.clone();
                            }
                        }
                        let is_url_search_params = entries.iter().any(|(key, value)| {
                            key == INTERNAL_URL_SEARCH_PARAMS_OBJECT_KEY
                                && matches!(value, Value::Bool(true))
                        });
                        if is_url_search_params {
                            let mut pairs = Vec::new();
                            if let Some(Value::Array(list)) =
                                entries.iter().find_map(|(key, value)| {
                                    (key == INTERNAL_URL_SEARCH_PARAMS_ENTRIES_KEY).then(|| value)
                                })
                            {
                                let list = list.borrow();
                                for item in list.iter() {
                                    let Value::Array(pair) = item else {
                                        continue;
                                    };
                                    let pair = pair.borrow();
                                    if pair.is_empty() {
                                        continue;
                                    }
                                    let name = pair[0].as_string();
                                    let value =
                                        pair.get(1).map(Value::as_string).unwrap_or_default();
                                    pairs.push((name, value));
                                }
                            }
                            serialize_url_search_params_pairs(&pairs)
                        } else {
                            let is_readable_stream = entries.iter().any(|(key, value)| {
                                key == INTERNAL_READABLE_STREAM_OBJECT_KEY
                                    && matches!(value, Value::Bool(true))
                            });
                            if is_readable_stream {
                                "[object ReadableStream]".into()
                            } else {
                                "[object Object]".into()
                            }
                        }
                    }
                }
            }
            Self::Promise(_) => "[object Promise]".into(),
            Self::Map(_) => "[object Map]".into(),
            Self::Set(_) => "[object Set]".into(),
            Self::Blob(_) => "[object Blob]".into(),
            Self::ArrayBuffer(_) => "[object ArrayBuffer]".into(),
            Self::TypedArray(value) => {
                let value = value.borrow();
                format!("[object {}]", value.kind.name())
            }
            Self::TypedArrayConstructor(kind) => match kind {
                TypedArrayConstructorKind::Concrete(kind) => kind.name().to_string(),
                TypedArrayConstructorKind::Abstract => "TypedArray".to_string(),
            },
            Self::BlobConstructor => "Blob".to_string(),
            Self::UrlConstructor => "URL".to_string(),
            Self::ArrayBufferConstructor => "ArrayBuffer".to_string(),
            Self::PromiseConstructor => "Promise".to_string(),
            Self::MapConstructor => "Map".to_string(),
            Self::SetConstructor => "Set".to_string(),
            Self::SymbolConstructor => "Symbol".to_string(),
            Self::RegExpConstructor => "RegExp".to_string(),
            Self::PromiseCapability(_) => "[object Function]".into(),
            Self::Symbol(value) => {
                if let Some(description) = &value.description {
                    format!("Symbol({description})")
                } else {
                    "Symbol()".to_string()
                }
            }
            Self::RegExp(value) => {
                let value = value.borrow();
                format!("/{}/{}", value.source, value.flags)
            }
            Self::Date(_) => "[object Date]".into(),
            Self::Null => "null".into(),
            Self::Undefined => "undefined".into(),
            Self::Node(node) => format!("node-{}", node.0),
            Self::NodeList(_) => "[object NodeList]".into(),
            Self::FormData(_) => "[object FormData]".into(),
            Self::Function(_) => "[object Function]".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomProp {
    Attributes,
    AssignedSlot,
    Value,
    ValueLength,
    ValidationMessage,
    Validity,
    ValidityValueMissing,
    ValidityTypeMismatch,
    ValidityPatternMismatch,
    ValidityTooLong,
    ValidityTooShort,
    ValidityRangeUnderflow,
    ValidityRangeOverflow,
    ValidityStepMismatch,
    ValidityBadInput,
    ValidityValid,
    ValidityCustomError,
    SelectionStart,
    SelectionEnd,
    SelectionDirection,
    Checked,
    Indeterminate,
    Open,
    ReturnValue,
    ClosedBy,
    Readonly,
    Required,
    Disabled,
    TextContent,
    InnerText,
    InnerHtml,
    OuterHtml,
    ClassName,
    ClassList,
    ClassListLength,
    Part,
    PartLength,
    Id,
    TagName,
    LocalName,
    NamespaceUri,
    Prefix,
    NextElementSibling,
    PreviousElementSibling,
    Slot,
    Role,
    ElementTiming,
    Name,
    Lang,
    ClientWidth,
    ClientHeight,
    ClientLeft,
    ClientTop,
    CurrentCssZoom,
    OffsetWidth,
    OffsetHeight,
    OffsetLeft,
    OffsetTop,
    ScrollWidth,
    ScrollHeight,
    ScrollLeft,
    ScrollTop,
    ScrollLeftMax,
    ScrollTopMax,
    ShadowRoot,
    Dataset(String),
    Style(String),
    AriaString(String),
    AriaElementRefSingle(String),
    AriaElementRefList(String),
    ActiveElement,
    CharacterSet,
    CompatMode,
    ContentType,
    ReadyState,
    Referrer,
    Title,
    Url,
    DocumentUri,
    Location,
    LocationHref,
    LocationProtocol,
    LocationHost,
    LocationHostname,
    LocationPort,
    LocationPathname,
    LocationSearch,
    LocationHash,
    LocationOrigin,
    LocationAncestorOrigins,
    History,
    HistoryLength,
    HistoryState,
    HistoryScrollRestoration,
    DefaultView,
    Hidden,
    VisibilityState,
    Forms,
    Images,
    Links,
    Scripts,
    Children,
    ChildElementCount,
    FirstElementChild,
    LastElementChild,
    CurrentScript,
    FormsLength,
    ImagesLength,
    LinksLength,
    ScriptsLength,
    ChildrenLength,
    AnchorAttributionSrc,
    AnchorDownload,
    AnchorHash,
    AnchorHost,
    AnchorHostname,
    AnchorHref,
    AnchorHreflang,
    AnchorInterestForElement,
    AnchorOrigin,
    AnchorPassword,
    AnchorPathname,
    AnchorPing,
    AnchorPort,
    AnchorProtocol,
    AnchorReferrerPolicy,
    AnchorRel,
    AnchorRelList,
    AnchorRelListLength,
    AnchorSearch,
    AnchorTarget,
    AnchorText,
    AnchorType,
    AnchorUsername,
    AnchorCharset,
    AnchorCoords,
    AnchorRev,
    AnchorShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomIndex {
    Static(usize),
    Dynamic(String),
}

impl DomIndex {
    fn describe(&self) -> String {
        match self {
            Self::Static(index) => index.to_string(),
            Self::Dynamic(expr) => expr.clone(),
        }
    }

    fn static_index(&self) -> Option<usize> {
        match self {
            Self::Static(index) => Some(*index),
            Self::Dynamic(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomQuery {
    DocumentRoot,
    DocumentBody,
    DocumentHead,
    DocumentElement,
    ById(String),
    BySelector(String),
    BySelectorAll {
        selector: String,
    },
    BySelectorAllIndex {
        selector: String,
        index: DomIndex,
    },
    QuerySelector {
        target: Box<DomQuery>,
        selector: String,
    },
    QuerySelectorAll {
        target: Box<DomQuery>,
        selector: String,
    },
    Index {
        target: Box<DomQuery>,
        index: DomIndex,
    },
    QuerySelectorAllIndex {
        target: Box<DomQuery>,
        selector: String,
        index: DomIndex,
    },
    FormElementsIndex {
        form: Box<DomQuery>,
        index: DomIndex,
    },
    Var(String),
    VarPath {
        base: String,
        path: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FormDataSource {
    NewForm(DomQuery),
    Var(String),
}

impl DomQuery {
    fn describe_call(&self) -> String {
        match self {
            Self::DocumentRoot => "document".into(),
            Self::DocumentBody => "document.body".into(),
            Self::DocumentHead => "document.head".into(),
            Self::DocumentElement => "document.documentElement".into(),
            Self::ById(id) => format!("document.getElementById('{id}')"),
            Self::BySelector(selector) => format!("document.querySelector('{selector}')"),
            Self::BySelectorAll { selector } => format!("document.querySelectorAll('{selector}')"),
            Self::BySelectorAllIndex { selector, index } => {
                format!(
                    "document.querySelectorAll('{selector}')[{}]",
                    index.describe()
                )
            }
            Self::QuerySelector { target, selector } => {
                format!("{}.querySelector('{selector}')", target.describe_call())
            }
            Self::QuerySelectorAll { target, selector } => {
                format!("{}.querySelectorAll('{selector}')", target.describe_call())
            }
            Self::Index { target, index } => {
                format!("{}[{}]", target.describe_call(), index.describe())
            }
            Self::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                format!(
                    "{}.querySelectorAll('{selector}')[{}]",
                    target.describe_call(),
                    index.describe()
                )
            }
            Self::FormElementsIndex { form, index } => {
                format!("{}.elements[{}]", form.describe_call(), index.describe())
            }
            Self::Var(name) => name.clone(),
            Self::VarPath { base, path } => format!("{base}.{}", path.join(".")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassListMethod {
    Add,
    Remove,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinaryOp {
    Or,
    And,
    Nullish,
    Eq,
    Ne,
    StrictEq,
    StrictNe,
    BitOr,
    BitXor,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    Pow,
    Lt,
    Gt,
    Le,
    Ge,
    In,
    InstanceOf,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VarAssignOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    BitOr,
    BitXor,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    LogicalAnd,
    LogicalOr,
    Nullish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventExprProp {
    Type,
    Target,
    CurrentTarget,
    TargetName,
    CurrentTargetName,
    DefaultPrevented,
    IsTrusted,
    Bubbles,
    Cancelable,
    TargetId,
    CurrentTargetId,
    EventPhase,
    TimeStamp,
    State,
    OldState,
    NewState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchMediaProp {
    Matches,
    Media,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntlFormatterKind {
    Collator,
    DateTimeFormat,
    DisplayNames,
    DurationFormat,
    ListFormat,
    NumberFormat,
    PluralRules,
    RelativeTimeFormat,
    Segmenter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntlStaticMethod {
    CollatorSupportedLocalesOf,
    DateTimeFormatSupportedLocalesOf,
    DisplayNamesSupportedLocalesOf,
    DurationFormatSupportedLocalesOf,
    ListFormatSupportedLocalesOf,
    PluralRulesSupportedLocalesOf,
    RelativeTimeFormatSupportedLocalesOf,
    SegmenterSupportedLocalesOf,
    GetCanonicalLocales,
    SupportedValuesOf,
}

impl IntlFormatterKind {
    fn storage_name(self) -> &'static str {
        match self {
            Self::Collator => "Collator",
            Self::DateTimeFormat => "DateTimeFormat",
            Self::DisplayNames => "DisplayNames",
            Self::DurationFormat => "DurationFormat",
            Self::ListFormat => "ListFormat",
            Self::NumberFormat => "NumberFormat",
            Self::PluralRules => "PluralRules",
            Self::RelativeTimeFormat => "RelativeTimeFormat",
            Self::Segmenter => "Segmenter",
        }
    }

    fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "Collator" => Some(Self::Collator),
            "DateTimeFormat" => Some(Self::DateTimeFormat),
            "DisplayNames" => Some(Self::DisplayNames),
            "DurationFormat" => Some(Self::DurationFormat),
            "ListFormat" => Some(Self::ListFormat),
            "NumberFormat" => Some(Self::NumberFormat),
            "PluralRules" => Some(Self::PluralRules),
            "RelativeTimeFormat" => Some(Self::RelativeTimeFormat),
            "Segmenter" => Some(Self::Segmenter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct IntlDateTimeOptions {
    calendar: String,
    numbering_system: String,
    time_zone: String,
    date_style: Option<String>,
    time_style: Option<String>,
    weekday: Option<String>,
    year: Option<String>,
    month: Option<String>,
    day: Option<String>,
    hour: Option<String>,
    minute: Option<String>,
    second: Option<String>,
    fractional_second_digits: Option<u8>,
    time_zone_name: Option<String>,
    hour12: Option<bool>,
    day_period: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct IntlDateTimeComponents {
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
    weekday: u32,
    offset_minutes: i64,
}

#[derive(Debug, Clone)]
struct IntlPart {
    part_type: String,
    value: String,
}

#[derive(Debug, Clone)]
struct IntlRelativeTimePart {
    part_type: String,
    value: String,
    unit: Option<String>,
}

#[derive(Debug, Clone)]
struct IntlDisplayNamesOptions {
    style: String,
    display_type: String,
    fallback: String,
    language_display: String,
}

#[derive(Debug, Clone)]
struct IntlDurationOptions {
    style: String,
}

#[derive(Debug, Clone)]
struct IntlListOptions {
    style: String,
    list_type: String,
}

#[derive(Debug, Clone)]
struct IntlPluralRulesOptions {
    rule_type: String,
}

#[derive(Debug, Clone)]
struct IntlRelativeTimeOptions {
    style: String,
    numeric: String,
    locale_matcher: String,
}

#[derive(Debug, Clone)]
struct IntlSegmenterOptions {
    granularity: String,
    locale_matcher: String,
}

#[derive(Debug, Clone)]
struct IntlLocaleOptions {
    language: Option<String>,
    script: Option<String>,
    region: Option<String>,
    calendar: Option<String>,
    case_first: Option<String>,
    collation: Option<String>,
    hour_cycle: Option<String>,
    numbering_system: Option<String>,
    numeric: Option<bool>,
}

#[derive(Debug, Clone)]
struct IntlLocaleData {
    language: String,
    script: Option<String>,
    region: Option<String>,
    variants: Vec<String>,
    calendar: Option<String>,
    case_first: Option<String>,
    collation: Option<String>,
    hour_cycle: Option<String>,
    numbering_system: Option<String>,
    numeric: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntlLocaleMethod {
    GetCalendars,
    GetCollations,
    GetHourCycles,
    GetNumberingSystems,
    GetTextInfo,
    GetTimeZones,
    GetWeekInfo,
    Maximize,
    Minimize,
    ToString,
}

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    String(String),
    Bool(bool),
    Null,
    Undefined,
    Number(i64),
    Float(f64),
    BigInt(JsBigInt),
    DateNow,
    PerformanceNow,
    DateNew {
        value: Option<Box<Expr>>,
    },
    DateParse(Box<Expr>),
    DateUtc {
        args: Vec<Expr>,
    },
    DateGetTime(String),
    DateSetTime {
        target: String,
        value: Box<Expr>,
    },
    DateToIsoString(String),
    DateGetFullYear(String),
    DateGetMonth(String),
    DateGetDate(String),
    DateGetHours(String),
    DateGetMinutes(String),
    DateGetSeconds(String),
    IntlFormatterConstruct {
        kind: IntlFormatterKind,
        locales: Option<Box<Expr>>,
        options: Option<Box<Expr>>,
        called_with_new: bool,
    },
    IntlFormat {
        formatter: Box<Expr>,
        value: Option<Box<Expr>>,
    },
    IntlFormatGetter {
        formatter: Box<Expr>,
    },
    IntlCollatorCompare {
        collator: Box<Expr>,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    IntlCollatorCompareGetter {
        collator: Box<Expr>,
    },
    IntlDateTimeFormatToParts {
        formatter: Box<Expr>,
        value: Option<Box<Expr>>,
    },
    IntlDateTimeFormatRange {
        formatter: Box<Expr>,
        start: Box<Expr>,
        end: Box<Expr>,
    },
    IntlDateTimeFormatRangeToParts {
        formatter: Box<Expr>,
        start: Box<Expr>,
        end: Box<Expr>,
    },
    IntlDateTimeResolvedOptions {
        formatter: Box<Expr>,
    },
    IntlDisplayNamesOf {
        display_names: Box<Expr>,
        code: Box<Expr>,
    },
    IntlPluralRulesSelect {
        plural_rules: Box<Expr>,
        value: Box<Expr>,
    },
    IntlPluralRulesSelectRange {
        plural_rules: Box<Expr>,
        start: Box<Expr>,
        end: Box<Expr>,
    },
    IntlRelativeTimeFormat {
        formatter: Box<Expr>,
        value: Box<Expr>,
        unit: Box<Expr>,
    },
    IntlRelativeTimeFormatToParts {
        formatter: Box<Expr>,
        value: Box<Expr>,
        unit: Box<Expr>,
    },
    IntlSegmenterSegment {
        segmenter: Box<Expr>,
        value: Box<Expr>,
    },
    IntlStaticMethod {
        method: IntlStaticMethod,
        args: Vec<Expr>,
    },
    IntlConstruct {
        args: Vec<Expr>,
    },
    IntlLocaleConstruct {
        tag: Box<Expr>,
        options: Option<Box<Expr>>,
        called_with_new: bool,
    },
    IntlLocaleMethod {
        locale: Box<Expr>,
        method: IntlLocaleMethod,
    },
    RegexLiteral {
        pattern: String,
        flags: String,
    },
    RegexNew {
        pattern: Box<Expr>,
        flags: Option<Box<Expr>>,
    },
    RegExpConstructor,
    RegExpStaticMethod {
        method: RegExpStaticMethod,
        args: Vec<Expr>,
    },
    RegexTest {
        regex: Box<Expr>,
        input: Box<Expr>,
    },
    RegexExec {
        regex: Box<Expr>,
        input: Box<Expr>,
    },
    RegexToString {
        regex: Box<Expr>,
    },
    MathConst(MathConst),
    MathMethod {
        method: MathMethod,
        args: Vec<Expr>,
    },
    StringConstruct {
        value: Option<Box<Expr>>,
        called_with_new: bool,
    },
    StringStaticMethod {
        method: StringStaticMethod,
        args: Vec<Expr>,
    },
    StringConstructor,
    NumberConstruct {
        value: Option<Box<Expr>>,
    },
    NumberConst(NumberConst),
    NumberMethod {
        method: NumberMethod,
        args: Vec<Expr>,
    },
    NumberInstanceMethod {
        value: Box<Expr>,
        method: NumberInstanceMethod,
        args: Vec<Expr>,
    },
    BigIntConstruct {
        value: Option<Box<Expr>>,
        called_with_new: bool,
    },
    BigIntMethod {
        method: BigIntMethod,
        args: Vec<Expr>,
    },
    BigIntInstanceMethod {
        value: Box<Expr>,
        method: BigIntInstanceMethod,
        args: Vec<Expr>,
    },
    BlobConstruct {
        parts: Option<Box<Expr>>,
        options: Option<Box<Expr>>,
        called_with_new: bool,
    },
    BlobConstructor,
    UrlConstruct {
        input: Option<Box<Expr>>,
        base: Option<Box<Expr>>,
        called_with_new: bool,
    },
    UrlConstructor,
    UrlStaticMethod {
        method: UrlStaticMethod,
        args: Vec<Expr>,
    },
    ArrayBufferConstruct {
        byte_length: Option<Box<Expr>>,
        options: Option<Box<Expr>>,
        called_with_new: bool,
    },
    ArrayBufferConstructor,
    ArrayBufferIsView(Box<Expr>),
    ArrayBufferDetached(String),
    ArrayBufferMaxByteLength(String),
    ArrayBufferResizable(String),
    ArrayBufferResize {
        target: String,
        new_byte_length: Box<Expr>,
    },
    ArrayBufferSlice {
        target: String,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    ArrayBufferTransfer {
        target: String,
        to_fixed_length: bool,
    },
    TypedArrayConstructorRef(TypedArrayConstructorKind),
    TypedArrayConstruct {
        kind: TypedArrayKind,
        args: Vec<Expr>,
        called_with_new: bool,
    },
    TypedArrayConstructWithCallee {
        callee: Box<Expr>,
        args: Vec<Expr>,
        called_with_new: bool,
    },
    PromiseConstruct {
        executor: Option<Box<Expr>>,
        called_with_new: bool,
    },
    PromiseConstructor,
    PromiseStaticMethod {
        method: PromiseStaticMethod,
        args: Vec<Expr>,
    },
    PromiseMethod {
        target: Box<Expr>,
        method: PromiseInstanceMethod,
        args: Vec<Expr>,
    },
    MapConstruct {
        iterable: Option<Box<Expr>>,
        called_with_new: bool,
    },
    MapConstructor,
    MapStaticMethod {
        method: MapStaticMethod,
        args: Vec<Expr>,
    },
    MapMethod {
        target: String,
        method: MapInstanceMethod,
        args: Vec<Expr>,
    },
    UrlSearchParamsConstruct {
        init: Option<Box<Expr>>,
        called_with_new: bool,
    },
    UrlSearchParamsMethod {
        target: String,
        method: UrlSearchParamsInstanceMethod,
        args: Vec<Expr>,
    },
    SetConstruct {
        iterable: Option<Box<Expr>>,
        called_with_new: bool,
    },
    SetConstructor,
    SetMethod {
        target: String,
        method: SetInstanceMethod,
        args: Vec<Expr>,
    },
    SymbolConstruct {
        description: Option<Box<Expr>>,
        called_with_new: bool,
    },
    SymbolConstructor,
    SymbolStaticMethod {
        method: SymbolStaticMethod,
        args: Vec<Expr>,
    },
    SymbolStaticProperty(SymbolStaticProperty),
    TypedArrayStaticBytesPerElement(TypedArrayKind),
    TypedArrayStaticMethod {
        kind: TypedArrayKind,
        method: TypedArrayStaticMethod,
        args: Vec<Expr>,
    },
    TypedArrayByteLength(String),
    TypedArrayByteOffset(String),
    TypedArrayBuffer(String),
    TypedArrayBytesPerElement(String),
    TypedArrayMethod {
        target: String,
        method: TypedArrayInstanceMethod,
        args: Vec<Expr>,
    },
    EncodeUri(Box<Expr>),
    EncodeUriComponent(Box<Expr>),
    DecodeUri(Box<Expr>),
    DecodeUriComponent(Box<Expr>),
    Escape(Box<Expr>),
    Unescape(Box<Expr>),
    IsNaN(Box<Expr>),
    IsFinite(Box<Expr>),
    Atob(Box<Expr>),
    Btoa(Box<Expr>),
    ParseInt {
        value: Box<Expr>,
        radix: Option<Box<Expr>>,
    },
    ParseFloat(Box<Expr>),
    JsonParse(Box<Expr>),
    JsonStringify(Box<Expr>),
    ObjectConstruct {
        value: Option<Box<Expr>>,
    },
    ObjectLiteral(Vec<ObjectLiteralEntry>),
    ObjectGet {
        target: String,
        key: String,
    },
    ObjectPathGet {
        target: String,
        path: Vec<String>,
    },
    ObjectGetOwnPropertySymbols(Box<Expr>),
    ObjectKeys(Box<Expr>),
    ObjectValues(Box<Expr>),
    ObjectEntries(Box<Expr>),
    ObjectHasOwn {
        object: Box<Expr>,
        key: Box<Expr>,
    },
    ObjectGetPrototypeOf(Box<Expr>),
    ObjectFreeze(Box<Expr>),
    ObjectHasOwnProperty {
        target: String,
        key: Box<Expr>,
    },
    ArrayLiteral(Vec<Expr>),
    ArrayIsArray(Box<Expr>),
    ArrayFrom {
        source: Box<Expr>,
        map_fn: Option<Box<Expr>>,
    },
    ArrayLength(String),
    ArrayIndex {
        target: String,
        index: Box<Expr>,
    },
    ArrayPush {
        target: String,
        args: Vec<Expr>,
    },
    ArrayPop(String),
    ArrayShift(String),
    ArrayUnshift {
        target: String,
        args: Vec<Expr>,
    },
    ArrayMap {
        target: String,
        callback: ScriptHandler,
    },
    ArrayFilter {
        target: String,
        callback: ScriptHandler,
    },
    ArrayReduce {
        target: String,
        callback: ScriptHandler,
        initial: Option<Box<Expr>>,
    },
    ArrayForEach {
        target: String,
        callback: ScriptHandler,
    },
    ArrayFind {
        target: String,
        callback: ScriptHandler,
    },
    ArrayFindIndex {
        target: String,
        callback: ScriptHandler,
    },
    ArraySome {
        target: String,
        callback: ScriptHandler,
    },
    ArrayEvery {
        target: String,
        callback: ScriptHandler,
    },
    ArrayIncludes {
        target: String,
        search: Box<Expr>,
        from_index: Option<Box<Expr>>,
    },
    ArraySlice {
        target: String,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    ArraySplice {
        target: String,
        start: Box<Expr>,
        delete_count: Option<Box<Expr>>,
        items: Vec<Expr>,
    },
    ArrayJoin {
        target: String,
        separator: Option<Box<Expr>>,
    },
    ArraySort {
        target: String,
        comparator: Option<Box<Expr>>,
    },
    StringTrim {
        value: Box<Expr>,
        mode: StringTrimMode,
    },
    StringToUpperCase(Box<Expr>),
    StringToLowerCase(Box<Expr>),
    StringIncludes {
        value: Box<Expr>,
        search: Box<Expr>,
        position: Option<Box<Expr>>,
    },
    StringStartsWith {
        value: Box<Expr>,
        search: Box<Expr>,
        position: Option<Box<Expr>>,
    },
    StringEndsWith {
        value: Box<Expr>,
        search: Box<Expr>,
        length: Option<Box<Expr>>,
    },
    StringSlice {
        value: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    StringSubstring {
        value: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    StringMatch {
        value: Box<Expr>,
        pattern: Box<Expr>,
    },
    StringSplit {
        value: Box<Expr>,
        separator: Option<Box<Expr>>,
        limit: Option<Box<Expr>>,
    },
    StringReplace {
        value: Box<Expr>,
        from: Box<Expr>,
        to: Box<Expr>,
    },
    StringReplaceAll {
        value: Box<Expr>,
        from: Box<Expr>,
        to: Box<Expr>,
    },
    StringIndexOf {
        value: Box<Expr>,
        search: Box<Expr>,
        position: Option<Box<Expr>>,
    },
    StringLastIndexOf {
        value: Box<Expr>,
        search: Box<Expr>,
        position: Option<Box<Expr>>,
    },
    StringCharAt {
        value: Box<Expr>,
        index: Option<Box<Expr>>,
    },
    StringCharCodeAt {
        value: Box<Expr>,
        index: Option<Box<Expr>>,
    },
    StringCodePointAt {
        value: Box<Expr>,
        index: Option<Box<Expr>>,
    },
    StringAt {
        value: Box<Expr>,
        index: Option<Box<Expr>>,
    },
    StringConcat {
        value: Box<Expr>,
        args: Vec<Expr>,
    },
    StringSearch {
        value: Box<Expr>,
        pattern: Box<Expr>,
    },
    StringRepeat {
        value: Box<Expr>,
        count: Box<Expr>,
    },
    StringPadStart {
        value: Box<Expr>,
        target_length: Box<Expr>,
        pad: Option<Box<Expr>>,
    },
    StringPadEnd {
        value: Box<Expr>,
        target_length: Box<Expr>,
        pad: Option<Box<Expr>>,
    },
    StringLocaleCompare {
        value: Box<Expr>,
        compare: Box<Expr>,
        locales: Option<Box<Expr>>,
        options: Option<Box<Expr>>,
    },
    StringIsWellFormed(Box<Expr>),
    StringToWellFormed(Box<Expr>),
    StringValueOf(Box<Expr>),
    StringToString(Box<Expr>),
    StructuredClone(Box<Expr>),
    Fetch(Box<Expr>),
    MatchMedia(Box<Expr>),
    MatchMediaProp {
        query: Box<Expr>,
        prop: MatchMediaProp,
    },
    Alert(Box<Expr>),
    Confirm(Box<Expr>),
    Prompt {
        message: Box<Expr>,
        default: Option<Box<Expr>>,
    },
    FunctionConstructor {
        args: Vec<Expr>,
    },
    FunctionCall {
        target: String,
        args: Vec<Expr>,
    },
    Call {
        target: Box<Expr>,
        args: Vec<Expr>,
    },
    MemberCall {
        target: Box<Expr>,
        member: String,
        args: Vec<Expr>,
        optional: bool,
    },
    MemberGet {
        target: Box<Expr>,
        member: String,
        optional: bool,
    },
    IndexGet {
        target: Box<Expr>,
        index: Box<Expr>,
        optional: bool,
    },
    Var(String),
    DomRef(DomQuery),
    CreateElement(String),
    CreateTextNode(String),
    SetTimeout {
        handler: TimerInvocation,
        delay_ms: Box<Expr>,
    },
    SetInterval {
        handler: TimerInvocation,
        delay_ms: Box<Expr>,
    },
    RequestAnimationFrame {
        callback: TimerCallback,
    },
    Function {
        handler: ScriptHandler,
        is_async: bool,
    },
    QueueMicrotask {
        handler: ScriptHandler,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    DomRead {
        target: DomQuery,
        prop: DomProp,
    },
    LocationMethodCall {
        method: LocationMethod,
        url: Option<Box<Expr>>,
    },
    HistoryMethodCall {
        method: HistoryMethod,
        args: Vec<Expr>,
    },
    ClipboardMethodCall {
        method: ClipboardMethod,
        args: Vec<Expr>,
    },
    DocumentHasFocus,
    DomMatches {
        target: DomQuery,
        selector: String,
    },
    DomClosest {
        target: DomQuery,
        selector: String,
    },
    DomComputedStyleProperty {
        target: DomQuery,
        property: String,
    },
    ClassListContains {
        target: DomQuery,
        class_name: String,
    },
    QuerySelectorAllLength {
        target: DomQuery,
    },
    FormElementsLength {
        form: DomQuery,
    },
    FormDataNew {
        form: DomQuery,
    },
    FormDataGet {
        source: FormDataSource,
        name: String,
    },
    FormDataHas {
        source: FormDataSource,
        name: String,
    },
    FormDataGetAll {
        source: FormDataSource,
        name: String,
    },
    FormDataGetAllLength {
        source: FormDataSource,
        name: String,
    },
    DomGetAttribute {
        target: DomQuery,
        name: String,
    },
    DomHasAttribute {
        target: DomQuery,
        name: String,
    },
    EventProp {
        event_var: String,
        prop: EventExprProp,
    },
    Neg(Box<Expr>),
    Pos(Box<Expr>),
    BitNot(Box<Expr>),
    Not(Box<Expr>),
    Void(Box<Expr>),
    Delete(Box<Expr>),
    TypeOf(Box<Expr>),
    Await(Box<Expr>),
    Yield(Box<Expr>),
    YieldStar(Box<Expr>),
    Comma(Vec<Expr>),
    Spread(Box<Expr>),
    Add(Vec<Expr>),
    Ternary {
        cond: Box<Expr>,
        on_true: Box<Expr>,
        on_false: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventMethod {
    PreventDefault,
    StopPropagation,
    StopImmediatePropagation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringTrimMode {
    Both,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringStaticMethod {
    FromCharCode,
    FromCodePoint,
    Raw,
}

#[derive(Debug, Clone, PartialEq)]
enum ObjectLiteralKey {
    Static(String),
    Computed(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
enum ObjectLiteralEntry {
    Pair(ObjectLiteralKey, Expr),
    Spread(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MathConst {
    E,
    Ln10,
    Ln2,
    Log10E,
    Log2E,
    Pi,
    Sqrt1_2,
    Sqrt2,
    ToStringTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MathMethod {
    Abs,
    Acos,
    Acosh,
    Asin,
    Asinh,
    Atan,
    Atan2,
    Atanh,
    Cbrt,
    Ceil,
    Clz32,
    Cos,
    Cosh,
    Exp,
    Expm1,
    Floor,
    F16Round,
    FRound,
    Hypot,
    Imul,
    Log,
    Log10,
    Log1p,
    Log2,
    Max,
    Min,
    Pow,
    Random,
    Round,
    Sign,
    Sin,
    Sinh,
    Sqrt,
    SumPrecise,
    Tan,
    Tanh,
    Trunc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumberConst {
    Epsilon,
    MaxSafeInteger,
    MaxValue,
    MinSafeInteger,
    MinValue,
    NaN,
    NegativeInfinity,
    PositiveInfinity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumberMethod {
    IsFinite,
    IsInteger,
    IsNaN,
    IsSafeInteger,
    ParseFloat,
    ParseInt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumberInstanceMethod {
    ToExponential,
    ToFixed,
    ToLocaleString,
    ToPrecision,
    ToString,
    ValueOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypedArrayStaticMethod {
    From,
    Of,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypedArrayInstanceMethod {
    At,
    CopyWithin,
    Entries,
    Fill,
    FindIndex,
    FindLast,
    FindLastIndex,
    IndexOf,
    Keys,
    LastIndexOf,
    ReduceRight,
    Reverse,
    Set,
    Sort,
    Subarray,
    ToReversed,
    ToSorted,
    Values,
    With,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MapStaticMethod {
    GroupBy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UrlStaticMethod {
    CanParse,
    Parse,
    CreateObjectUrl,
    RevokeObjectUrl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolStaticMethod {
    For,
    KeyFor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolStaticProperty {
    AsyncDispose,
    AsyncIterator,
    Dispose,
    HasInstance,
    IsConcatSpreadable,
    Iterator,
    Match,
    MatchAll,
    Replace,
    Search,
    Species,
    Split,
    ToPrimitive,
    ToStringTag,
    Unscopables,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegExpStaticMethod {
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromiseStaticMethod {
    Resolve,
    Reject,
    All,
    AllSettled,
    Any,
    Race,
    Try,
    WithResolvers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MapInstanceMethod {
    Get,
    Has,
    Delete,
    Clear,
    ForEach,
    GetOrInsert,
    GetOrInsertComputed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UrlSearchParamsInstanceMethod {
    Append,
    Delete,
    GetAll,
    Has,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromiseInstanceMethod {
    Then,
    Catch,
    Finally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetInstanceMethod {
    Add,
    Union,
    Intersection,
    Difference,
    SymmetricDifference,
    IsDisjointFrom,
    IsSubsetOf,
    IsSupersetOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BigIntMethod {
    AsIntN,
    AsUintN,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BigIntInstanceMethod {
    ToLocaleString,
    ToString,
    ValueOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeTreeMethod {
    After,
    Append,
    AppendChild,
    Before,
    ReplaceWith,
    Prepend,
    RemoveChild,
    InsertBefore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsertAdjacentPosition {
    BeforeBegin,
    AfterBegin,
    BeforeEnd,
    AfterEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListenerRegistrationOp {
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq)]
enum Stmt {
    VarDecl {
        name: String,
        expr: Expr,
    },
    FunctionDecl {
        name: String,
        handler: ScriptHandler,
        is_async: bool,
    },
    VarAssign {
        name: String,
        op: VarAssignOp,
        expr: Expr,
    },
    VarUpdate {
        name: String,
        delta: i8,
    },
    ArrayDestructureAssign {
        targets: Vec<Option<String>>,
        expr: Expr,
    },
    ObjectDestructureAssign {
        bindings: Vec<(String, String)>,
        expr: Expr,
    },
    ObjectAssign {
        target: String,
        path: Vec<Expr>,
        expr: Expr,
    },
    FormDataAppend {
        target_var: String,
        name: Expr,
        value: Expr,
    },
    DomAssign {
        target: DomQuery,
        prop: DomProp,
        expr: Expr,
    },
    ClassListCall {
        target: DomQuery,
        method: ClassListMethod,
        class_names: Vec<String>,
        force: Option<Expr>,
    },
    ClassListForEach {
        target: DomQuery,
        item_var: String,
        index_var: Option<String>,
        body: Vec<Stmt>,
    },
    DomSetAttribute {
        target: DomQuery,
        name: String,
        value: Expr,
    },
    DomRemoveAttribute {
        target: DomQuery,
        name: String,
    },
    NodeTreeMutation {
        target: DomQuery,
        method: NodeTreeMethod,
        child: Expr,
        reference: Option<Expr>,
    },
    InsertAdjacentElement {
        target: DomQuery,
        position: InsertAdjacentPosition,
        node: Expr,
    },
    InsertAdjacentText {
        target: DomQuery,
        position: InsertAdjacentPosition,
        text: Expr,
    },
    InsertAdjacentHTML {
        target: DomQuery,
        position: Expr,
        html: Expr,
    },
    SetTimeout {
        handler: TimerInvocation,
        delay_ms: Expr,
    },
    SetInterval {
        handler: TimerInvocation,
        delay_ms: Expr,
    },
    QueueMicrotask {
        handler: ScriptHandler,
    },
    ClearTimeout {
        timer_id: Expr,
    },
    NodeRemove {
        target: DomQuery,
    },
    ForEach {
        target: Option<DomQuery>,
        selector: String,
        item_var: String,
        index_var: Option<String>,
        body: Vec<Stmt>,
    },
    ArrayForEach {
        target: String,
        callback: ScriptHandler,
    },
    ArrayForEachExpr {
        target: Expr,
        callback: ScriptHandler,
    },
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        post: Option<Box<Stmt>>,
        body: Vec<Stmt>,
    },
    ForIn {
        item_var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    ForOf {
        item_var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    DoWhile {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Try {
        try_stmts: Vec<Stmt>,
        catch_binding: Option<CatchBinding>,
        catch_stmts: Option<Vec<Stmt>>,
        finally_stmts: Option<Vec<Stmt>>,
    },
    Throw {
        value: Expr,
    },
    Return {
        value: Option<Expr>,
    },
    If {
        cond: Expr,
        then_stmts: Vec<Stmt>,
        else_stmts: Vec<Stmt>,
    },
    EventCall {
        event_var: String,
        method: EventMethod,
    },
    ListenerMutation {
        target: DomQuery,
        op: ListenerRegistrationOp,
        event_type: String,
        capture: bool,
        handler: ScriptHandler,
    },
    DispatchEvent {
        target: DomQuery,
        event_type: Expr,
    },
    DomMethodCall {
        target: DomQuery,
        method: DomMethod,
        arg: Option<Expr>,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
enum CatchBinding {
    Identifier(String),
    ArrayPattern(Vec<Option<String>>),
    ObjectPattern(Vec<(String, String)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecFlow {
    Continue,
    Break,
    ContinueLoop,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomMethod {
    Focus,
    Blur,
    Click,
    ScrollIntoView,
    Submit,
    Reset,
    Show,
    ShowModal,
    Close,
    RequestClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocationMethod {
    Assign,
    Reload,
    Replace,
    ToString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HistoryMethod {
    Back,
    Forward,
    Go,
    PushState,
    ReplaceState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipboardMethod {
    ReadText,
    WriteText,
}

#[derive(Debug, Clone, PartialEq)]
struct ScriptHandler {
    params: Vec<FunctionParam>,
    stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
struct FunctionParam {
    name: String,
    default: Option<Expr>,
    is_rest: bool,
}

impl ScriptHandler {
    fn first_event_param(&self) -> Option<&str> {
        self.params.first().map(|param| param.name.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TimerCallback {
    Inline(ScriptHandler),
    Reference(String),
}

#[derive(Debug, Clone, PartialEq)]
struct TimerInvocation {
    callback: TimerCallback,
    args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocationParts {
    scheme: String,
    has_authority: bool,
    username: String,
    password: String,
    hostname: String,
    port: String,
    pathname: String,
    opaque_path: String,
    search: String,
    hash: String,
}

impl LocationParts {
    fn protocol(&self) -> String {
        format!("{}:", self.scheme)
    }

    fn host(&self) -> String {
        if self.port.is_empty() {
            self.hostname.clone()
        } else {
            format!("{}:{}", self.hostname, self.port)
        }
    }

    fn origin(&self) -> String {
        if self.has_authority && !self.hostname.is_empty() {
            format!("{}//{}", self.protocol(), self.host())
        } else {
            "null".to_string()
        }
    }

    fn href(&self) -> String {
        if self.has_authority {
            let path = if self.pathname.is_empty() {
                "/".to_string()
            } else {
                self.pathname.clone()
            };
            let credentials = if self.username.is_empty() && self.password.is_empty() {
                String::new()
            } else if self.password.is_empty() {
                format!("{}@", self.username)
            } else {
                format!("{}:{}@", self.username, self.password)
            };
            format!(
                "{}//{}{}{}{}{}",
                self.protocol(),
                credentials,
                self.host(),
                path,
                self.search,
                self.hash
            )
        } else {
            format!(
                "{}{}{}{}",
                self.protocol(),
                self.opaque_path,
                self.search,
                self.hash
            )
        }
    }

    fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        let scheme_end = trimmed.find(':')?;
        let scheme = trimmed[..scheme_end].to_ascii_lowercase();
        if !is_valid_url_scheme(&scheme) {
            return None;
        }
        let rest = &trimmed[scheme_end + 1..];
        if let Some(without_slashes) = rest.strip_prefix("//") {
            let authority_end = without_slashes
                .find(|ch| ['/', '?', '#'].contains(&ch))
                .unwrap_or(without_slashes.len());
            let authority = &without_slashes[..authority_end];
            let tail = &without_slashes[authority_end..];
            let (username, password, hostname, port) = split_authority_components(authority);
            let (pathname, search, hash) = split_path_search_hash(tail);
            let pathname = if pathname.is_empty() {
                "/".to_string()
            } else {
                normalize_pathname(&pathname)
            };
            Some(Self {
                scheme,
                has_authority: true,
                username,
                password,
                hostname,
                port,
                pathname,
                opaque_path: String::new(),
                search,
                hash,
            })
        } else {
            let (opaque_path, search, hash) = split_opaque_search_hash(rest);
            Some(Self {
                scheme,
                has_authority: false,
                username: String::new(),
                password: String::new(),
                hostname: String::new(),
                port: String::new(),
                pathname: String::new(),
                opaque_path,
                search,
                hash,
            })
        }
    }
}

fn is_valid_url_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

fn split_hostname_and_port(authority: &str) -> (String, String) {
    if authority.is_empty() {
        return (String::new(), String::new());
    }

    if let Some(rest) = authority.strip_prefix('[') {
        if let Some(end_idx) = rest.find(']') {
            let hostname = authority[..end_idx + 2].to_string();
            let suffix = &authority[end_idx + 2..];
            if let Some(port) = suffix.strip_prefix(':') {
                return (hostname, port.to_string());
            }
            return (hostname, String::new());
        }
    }

    if let Some(idx) = authority.rfind(':') {
        let hostname = &authority[..idx];
        let port = &authority[idx + 1..];
        if !hostname.contains(':') {
            return (hostname.to_string(), port.to_string());
        }
    }
    (authority.to_string(), String::new())
}

fn split_authority_components(authority: &str) -> (String, String, String, String) {
    if authority.is_empty() {
        return (String::new(), String::new(), String::new(), String::new());
    }

    let (userinfo, hostport) = if let Some(at) = authority.rfind('@') {
        (&authority[..at], &authority[at + 1..])
    } else {
        ("", authority)
    };

    let (username, password) = if userinfo.is_empty() {
        (String::new(), String::new())
    } else if let Some((username, password)) = userinfo.split_once(':') {
        (username.to_string(), password.to_string())
    } else {
        (userinfo.to_string(), String::new())
    };

    let (hostname, port) = split_hostname_and_port(hostport);
    (username, password, hostname, port)
}

fn split_path_search_hash(tail: &str) -> (String, String, String) {
    let mut pathname = tail;
    let mut search = "";
    let mut hash = "";

    if let Some(hash_pos) = tail.find('#') {
        pathname = &tail[..hash_pos];
        hash = &tail[hash_pos..];
    }

    if let Some(search_pos) = pathname.find('?') {
        search = &pathname[search_pos..];
        pathname = &pathname[..search_pos];
    }

    (pathname.to_string(), search.to_string(), hash.to_string())
}

fn split_opaque_search_hash(rest: &str) -> (String, String, String) {
    let mut opaque_path = rest;
    let mut search = "";
    let mut hash = "";

    if let Some(hash_pos) = rest.find('#') {
        opaque_path = &rest[..hash_pos];
        hash = &rest[hash_pos..];
    }

    if let Some(search_pos) = opaque_path.find('?') {
        search = &opaque_path[search_pos..];
        opaque_path = &opaque_path[..search_pos];
    }

    (
        opaque_path.to_string(),
        search.to_string(),
        hash.to_string(),
    )
}

fn normalize_pathname(pathname: &str) -> String {
    let starts_with_slash = pathname.starts_with('/');
    let ends_with_slash = pathname.ends_with('/') && pathname.len() > 1;
    let mut parts = Vec::new();
    for segment in pathname.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            parts.pop();
            continue;
        }
        parts.push(segment);
    }
    let mut out = if starts_with_slash {
        format!("/{}", parts.join("/"))
    } else {
        parts.join("/")
    };
    if out.is_empty() {
        out.push('/');
    }
    if ends_with_slash && !out.ends_with('/') {
        out.push('/');
    }
    out
}

fn ensure_search_prefix(value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else if value.starts_with('?') {
        value.to_string()
    } else {
        format!("?{value}")
    }
}

fn ensure_hash_prefix(value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else if value.starts_with('#') {
        value.to_string()
    } else {
        format!("#{value}")
    }
}

#[derive(Debug, Clone)]
struct Listener {
    capture: bool,
    handler: ScriptHandler,
    captured_env: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone)]
struct ListenerStore {
    map: HashMap<NodeId, HashMap<String, Vec<Listener>>>,
}

impl ListenerStore {
    fn add(&mut self, node_id: NodeId, event: String, listener: Listener) {
        self.map
            .entry(node_id)
            .or_default()
            .entry(event)
            .or_default()
            .push(listener);
    }

    fn remove(
        &mut self,
        node_id: NodeId,
        event: &str,
        capture: bool,
        handler: &ScriptHandler,
    ) -> bool {
        let Some(events) = self.map.get_mut(&node_id) else {
            return false;
        };
        let Some(listeners) = events.get_mut(event) else {
            return false;
        };

        if let Some(pos) = listeners
            .iter()
            .position(|listener| listener.capture == capture && listener.handler == *handler)
        {
            listeners.remove(pos);
            if listeners.is_empty() {
                events.remove(event);
            }
            if events.is_empty() {
                self.map.remove(&node_id);
            }
            return true;
        }

        false
    }

    fn get(&self, node_id: NodeId, event: &str, capture: bool) -> Vec<Listener> {
        self.map
            .get(&node_id)
            .and_then(|events| events.get(event))
            .map(|listeners| {
                listeners
                    .iter()
                    .filter(|listener| listener.capture == capture)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
struct EventState {
    event_type: String,
    target: NodeId,
    current_target: NodeId,
    event_phase: i32,
    time_stamp_ms: i64,
    default_prevented: bool,
    is_trusted: bool,
    bubbles: bool,
    cancelable: bool,
    state: Option<Value>,
    old_state: Option<String>,
    new_state: Option<String>,
    propagation_stopped: bool,
    immediate_propagation_stopped: bool,
}

impl EventState {
    fn new(event_type: &str, target: NodeId, time_stamp_ms: i64) -> Self {
        Self {
            event_type: event_type.to_string(),
            target,
            current_target: target,
            event_phase: 2,
            time_stamp_ms,
            default_prevented: false,
            is_trusted: true,
            bubbles: true,
            cancelable: true,
            state: None,
            old_state: None,
            new_state: None,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
        }
    }

    fn new_untrusted(event_type: &str, target: NodeId, time_stamp_ms: i64) -> Self {
        let mut event = Self::new(event_type, target, time_stamp_ms);
        event.is_trusted = false;
        event.bubbles = false;
        event.cancelable = false;
        event
    }
}

#[derive(Debug, Clone)]
struct ParseOutput {
    dom: Dom,
    scripts: Vec<String>,
}

#[derive(Debug, Clone)]
struct ScheduledTask {
    id: i64,
    due_at: i64,
    order: i64,
    interval_ms: Option<i64>,
    callback: TimerCallback,
    callback_args: Vec<Value>,
    env: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
enum ScheduledMicrotask {
    Script {
        handler: ScriptHandler,
        env: HashMap<String, Value>,
    },
    Promise {
        reaction: PromiseReactionKind,
        settled: PromiseSettledValue,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTimer {
    pub id: i64,
    pub due_at: i64,
    pub order: i64,
    pub interval_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocationNavigationKind {
    Assign,
    Replace,
    HrefSet,
    Reload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationNavigation {
    pub kind: LocationNavigationKind,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq)]
struct HistoryEntry {
    url: String,
    state: Value,
}

#[derive(Debug)]
pub struct Harness {
    dom: Dom,
    listeners: ListenerStore,
    node_event_handler_props: HashMap<(NodeId, String), ScriptHandler>,
    script_env: HashMap<String, Value>,
    document_url: String,
    window_object: Rc<RefCell<Vec<(String, Value)>>>,
    document_object: Rc<RefCell<Vec<(String, Value)>>>,
    location_object: Rc<RefCell<Vec<(String, Value)>>>,
    history_object: Rc<RefCell<Vec<(String, Value)>>>,
    history_entries: Vec<HistoryEntry>,
    history_index: usize,
    history_scroll_restoration: String,
    location_mock_pages: HashMap<String, String>,
    location_navigations: Vec<LocationNavigation>,
    location_reload_count: usize,
    task_queue: Vec<ScheduledTask>,
    microtask_queue: VecDeque<ScheduledMicrotask>,
    dialog_return_values: HashMap<NodeId, String>,
    active_element: Option<NodeId>,
    now_ms: i64,
    timer_step_limit: usize,
    next_timer_id: i64,
    next_task_order: i64,
    next_promise_id: usize,
    next_symbol_id: usize,
    next_url_object_id: usize,
    url_objects: HashMap<usize, Rc<RefCell<Vec<(String, Value)>>>>,
    next_blob_url_id: usize,
    blob_url_objects: HashMap<String, Rc<RefCell<BlobValue>>>,
    task_depth: usize,
    running_timer_id: Option<i64>,
    running_timer_canceled: bool,
    rng_state: u64,
    clipboard_text: String,
    fetch_mocks: HashMap<String, String>,
    fetch_calls: Vec<String>,
    match_media_mocks: HashMap<String, bool>,
    match_media_calls: Vec<String>,
    default_match_media_matches: bool,
    alert_messages: Vec<String>,
    confirm_responses: VecDeque<bool>,
    default_confirm_response: bool,
    prompt_responses: VecDeque<Option<String>>,
    default_prompt_response: Option<String>,
    symbol_registry: HashMap<String, Rc<SymbolValue>>,
    symbols_by_id: HashMap<usize, Rc<SymbolValue>>,
    well_known_symbols: HashMap<String, Rc<SymbolValue>>,
    trace: bool,
    trace_events: bool,
    trace_timers: bool,
    trace_logs: Vec<String>,
    trace_log_limit: usize,
    trace_to_stderr: bool,
    pending_function_decls: Vec<HashMap<String, (ScriptHandler, bool)>>,
}

#[derive(Debug)]
pub struct MockWindow {
    pages: Vec<MockPage>,
    current: usize,
}

#[derive(Debug)]
pub struct MockPage {
    pub url: String,
    harness: Harness,
}

impl MockWindow {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            current: 0,
        }
    }

    pub fn open_page(&mut self, url: &str, html: &str) -> Result<usize> {
        let mut harness = Harness::from_html(html)?;
        harness.document_url = url.to_string();
        if let Some(index) = self
            .pages
            .iter()
            .position(|page| page.url.eq_ignore_ascii_case(url))
        {
            self.pages[index] = MockPage {
                url: url.to_string(),
                harness,
            };
            self.current = index;
            Ok(index)
        } else {
            self.pages.push(MockPage {
                url: url.to_string(),
                harness,
            });
            self.current = self.pages.len() - 1;
            Ok(self.current)
        }
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn switch_to(&mut self, url: &str) -> Result<()> {
        let index = self
            .pages
            .iter()
            .position(|page| page.url == url)
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown page: {url}")))?;
        self.current = index;
        Ok(())
    }

    pub fn switch_to_index(&mut self, index: usize) -> Result<()> {
        if index >= self.pages.len() {
            return Err(Error::ScriptRuntime(format!(
                "page index out of range: {index}"
            )));
        }
        self.current = index;
        Ok(())
    }

    pub fn current_url(&self) -> Result<&str> {
        self.pages
            .get(self.current)
            .map(|page| page.url.as_str())
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    pub fn current_document_mut(&mut self) -> Result<&mut Harness> {
        self.pages
            .get_mut(self.current)
            .map(|page| &mut page.harness)
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    pub fn current_document(&self) -> Result<&Harness> {
        self.pages
            .get(self.current)
            .map(|page| &page.harness)
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    pub fn with_current_document<R>(
        &mut self,
        f: impl FnOnce(&mut Harness) -> Result<R>,
    ) -> Result<R> {
        let harness = self.current_document_mut()?;
        f(harness)
    }

    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.type_text(selector, text)
    }

    pub fn set_checked(&mut self, selector: &str, checked: bool) -> Result<()> {
        let page = self.current_document_mut()?;
        page.set_checked(selector, checked)
    }

    pub fn click(&mut self, selector: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.click(selector)
    }

    pub fn submit(&mut self, selector: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.submit(selector)
    }

    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.dispatch(selector, event)
    }

    pub fn assert_text(&mut self, selector: &str, expected: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.assert_text(selector, expected)
    }

    pub fn assert_value(&mut self, selector: &str, expected: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.assert_value(selector, expected)
    }

    pub fn assert_checked(&mut self, selector: &str, expected: bool) -> Result<()> {
        let page = self.current_document_mut()?;
        page.assert_checked(selector, expected)
    }

    pub fn assert_exists(&mut self, selector: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.assert_exists(selector)
    }

    pub fn take_trace_logs(&mut self) -> Result<Vec<String>> {
        let page = self.current_document_mut()?;
        Ok(page.take_trace_logs())
    }
}

impl MockPage {
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    pub fn harness(&self) -> &Harness {
        &self.harness
    }

    pub fn harness_mut(&mut self) -> &mut Harness {
        &mut self.harness
    }
}

mod core_impl;

#[cfg(test)]
fn parse_html(html: &str) -> Result<ParseOutput> {
    core_impl::parse_html(html)
}

#[cfg(test)]
fn parse_for_each_callback(src: &str) -> Result<(String, Option<String>, Vec<Stmt>)> {
    core_impl::parse_for_each_callback(src)
}

fn unescape_string(src: &str) -> String {
    core_impl::unescape_string(src)
}

#[cfg(test)]
mod tests;
