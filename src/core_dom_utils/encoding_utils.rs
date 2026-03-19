use super::*;

pub(crate) fn parse_js_parse_float(src: &str) -> f64 {
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

pub(crate) fn parse_js_parse_int(src: &str, radix: Option<i64>) -> f64 {
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

pub(crate) fn encode_binary_string_to_base64(src: &str) -> Result<String> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut bytes = Vec::with_capacity(src.len());
    for ch in src.chars() {
        let code = ch as u32;
        if code > 0xFF {
            return Err(Error::ScriptRuntime(
                "InvalidCharacterError: btoa input contains non-Latin1 character".into(),
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

pub(crate) fn decode_base64_to_binary_string(src: &str) -> Result<String> {
    let mut bytes: Vec<u8> = src.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if bytes.is_empty() {
        return Ok(String::new());
    }

    match bytes.len() % 4 {
        0 => {}
        2 => bytes.extend_from_slice(b"=="),
        3 => bytes.push(b'='),
        _ => {
            return Err(Error::ScriptRuntime(
                "InvalidCharacterError: atob invalid base64 input".into(),
            ));
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
                return Err(Error::ScriptRuntime(
                    "InvalidCharacterError: atob invalid base64 input".into(),
                ));
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

pub(crate) fn decode_base64_char(ch: u8) -> Result<u8> {
    let value = match ch {
        b'A'..=b'Z' => ch - b'A',
        b'a'..=b'z' => ch - b'a' + 26,
        b'0'..=b'9' => ch - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => {
            return Err(Error::ScriptRuntime(
                "InvalidCharacterError: atob invalid base64 input".into(),
            ));
        }
    };
    Ok(value)
}

pub(crate) fn encode_uri_like(src: &str, component: bool) -> String {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UrlPercentEncodeSet {
    UserInfo,
    Path,
    Query,
    SpecialQuery,
    Fragment,
    OpaquePath,
}

pub(crate) fn encode_url_component_preserving_percent(
    src: &str,
    encode_set: UrlPercentEncodeSet,
) -> String {
    let mut out = String::new();
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i].is_ascii() && is_unescaped_url_component_byte(bytes[i], encode_set) {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        }

        let ch = src[i..].chars().next().unwrap_or_default();
        let mut encoded = [0u8; 4];
        let encoded = ch.encode_utf8(&mut encoded);
        for b in encoded.as_bytes() {
            if is_unescaped_url_component_byte(*b, encode_set) {
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

pub(crate) fn decode_uri_like(src: &str, component: bool) -> Result<String> {
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

pub(crate) fn parse_url_search_params_pairs_from_query_string(
    query: &str,
) -> Vec<(String, String)> {
    let query = query.strip_prefix('?').unwrap_or(query);
    if query.is_empty() {
        return Vec::new();
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
        let name = decode_form_urlencoded_component(raw_name);
        let value = decode_form_urlencoded_component(raw_value);
        pairs.push((name, value));
    }
    pairs
}

pub(crate) fn serialize_url_search_params_pairs(pairs: &[(String, String)]) -> String {
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

pub(crate) fn encode_form_urlencoded_component(src: &str) -> String {
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

pub(crate) fn decode_form_urlencoded_component(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let (Some(hi), Some(lo)) =
                    (from_hex_digit(bytes[i + 1]), from_hex_digit(bytes[i + 2]))
                {
                    out.push((hi << 4) | lo);
                    i += 3;
                    continue;
                }
                out.push(b'%');
                i += 1;
            }
            _ => {
                let ch = src[i..].chars().next().unwrap_or_default();
                let mut encoded = [0u8; 4];
                let encoded = ch.encode_utf8(&mut encoded);
                out.extend_from_slice(encoded.as_bytes());
                i += ch.len_utf8();
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn is_form_urlencoded_unescaped_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'*' | b'-' | b'.' | b'_')
}

pub(crate) fn js_escape(src: &str) -> String {
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

pub(crate) fn js_unescape(src: &str) -> String {
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

pub(crate) fn render_js_string_for_display(src: &str) -> String {
    let mut units = Vec::with_capacity(src.len());
    for ch in src.chars() {
        if let Some(unit) = crate::js_regex::deinternalize_surrogate_marker(ch) {
            units.push(unit);
            continue;
        }
        let mut buf = [0u16; 2];
        units.extend_from_slice(ch.encode_utf16(&mut buf));
    }
    String::from_utf16_lossy(&units)
}

pub(crate) fn is_unescaped_uri_byte(b: u8, component: bool) -> bool {
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

pub(crate) fn is_unescaped_url_component_byte(b: u8, encode_set: UrlPercentEncodeSet) -> bool {
    if !b.is_ascii() || b < 0x20 || b == 0x7F {
        return false;
    }
    if b.is_ascii_alphanumeric() {
        return true;
    }

    match encode_set {
        UrlPercentEncodeSet::UserInfo => matches!(
            b,
            b'!' | b'$'
                | b'&'
                | b'%'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b'-'
                | b'.'
                | b'_'
                | b'~'
        ),
        UrlPercentEncodeSet::Path => matches!(
            b,
            b'!' | b'$'
                | b'&'
                | b'%'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b'-'
                | b'.'
                | b'/'
                | b':'
                | b';'
                | b'='
                | b'@'
                | b'_'
                | b'~'
                | b'|'
                | b'['
                | b']'
                | b'\\'
        ),
        UrlPercentEncodeSet::Query => matches!(
            b,
            b'!' | b'$'
                | b'&'
                | b'%'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b'-'
                | b'.'
                | b'/'
                | b':'
                | b';'
                | b'='
                | b'?'
                | b'@'
                | b'_'
                | b'~'
                | b'`'
                | b'{'
                | b'}'
                | b'|'
                | b'^'
                | b'['
                | b']'
                | b'\\'
        ),
        UrlPercentEncodeSet::SpecialQuery => matches!(
            b,
            b'!' | b'$'
                | b'&'
                | b'%'
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b'-'
                | b'.'
                | b'/'
                | b':'
                | b';'
                | b'='
                | b'?'
                | b'@'
                | b'_'
                | b'~'
                | b'`'
                | b'{'
                | b'}'
                | b'|'
                | b'^'
                | b'['
                | b']'
                | b'\\'
        ),
        UrlPercentEncodeSet::Fragment => matches!(
            b,
            b'!' | b'$'
                | b'&'
                | b'%'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b'-'
                | b'.'
                | b'/'
                | b':'
                | b';'
                | b'='
                | b'?'
                | b'@'
                | b'_'
                | b'~'
                | b'#'
                | b'{'
                | b'}'
                | b'|'
                | b'^'
                | b'['
                | b']'
                | b'\\'
        ),
        UrlPercentEncodeSet::OpaquePath => true,
    }
}

pub(crate) fn is_decode_uri_reserved_char(ch: char) -> bool {
    matches!(
        ch,
        ';' | ',' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | '#'
    )
}

pub(crate) fn is_unescaped_legacy_escape_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'*' | b'+' | b'-' | b'.' | b'/' | b'@' | b'_')
}

pub(crate) fn parse_percent_byte(bytes: &[u8], offset: usize) -> Result<u8> {
    if offset + 2 >= bytes.len() || bytes[offset] != b'%' {
        return Err(Error::ScriptRuntime("malformed URI sequence".into()));
    }
    let hi = from_hex_digit(bytes[offset + 1])
        .ok_or_else(|| Error::ScriptRuntime("malformed URI sequence".into()))?;
    let lo = from_hex_digit(bytes[offset + 2])
        .ok_or_else(|| Error::ScriptRuntime("malformed URI sequence".into()))?;
    Ok((hi << 4) | lo)
}

pub(crate) fn utf8_sequence_len(first: u8) -> Option<usize> {
    match first {
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

pub(crate) fn from_hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn to_hex_upper(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'A' + (nibble - 10)) as char,
        _ => '?',
    }
}
