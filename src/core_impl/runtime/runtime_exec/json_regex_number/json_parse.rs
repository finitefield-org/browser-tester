use super::*;

impl Harness {
    pub(crate) fn parse_json_text(src: &str) -> Result<Value> {
        let bytes = src.as_bytes();
        let mut i = 0usize;
        Self::json_skip_ws(bytes, &mut i);
        let value = Self::parse_json_value(src, bytes, &mut i)?;
        Self::json_skip_ws(bytes, &mut i);
        if i != bytes.len() {
            return Err(Error::ScriptRuntime(
                "JSON.parse invalid JSON: trailing characters".into(),
            ));
        }
        Ok(value)
    }

    pub(crate) fn parse_json_value(src: &str, bytes: &[u8], i: &mut usize) -> Result<Value> {
        Self::json_skip_ws(bytes, i);
        let Some(&b) = bytes.get(*i) else {
            return Err(Error::ScriptRuntime(
                "JSON.parse invalid JSON: unexpected end of input".into(),
            ));
        };

        match b {
            b'{' => Self::parse_json_object(src, bytes, i),
            b'[' => Self::parse_json_array(src, bytes, i),
            b'"' => Ok(Value::String(Self::parse_json_string(src, bytes, i)?)),
            b't' => {
                if Self::json_consume_ascii(bytes, i, "true") {
                    Ok(Value::Bool(true))
                } else {
                    Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: unexpected token".into(),
                    ))
                }
            }
            b'f' => {
                if Self::json_consume_ascii(bytes, i, "false") {
                    Ok(Value::Bool(false))
                } else {
                    Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: unexpected token".into(),
                    ))
                }
            }
            b'n' => {
                if Self::json_consume_ascii(bytes, i, "null") {
                    Ok(Value::Null)
                } else {
                    Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: unexpected token".into(),
                    ))
                }
            }
            b'-' | b'0'..=b'9' => Self::parse_json_number(src, bytes, i),
            _ => Err(Error::ScriptRuntime(
                "JSON.parse invalid JSON: unexpected token".into(),
            )),
        }
    }

    pub(crate) fn parse_json_object(src: &str, bytes: &[u8], i: &mut usize) -> Result<Value> {
        *i += 1; // consume '{'
        Self::json_skip_ws(bytes, i);
        let mut entries = Vec::new();

        if bytes.get(*i) == Some(&b'}') {
            *i += 1;
            return Ok(Self::new_object_value(entries));
        }

        loop {
            Self::json_skip_ws(bytes, i);
            if bytes.get(*i) != Some(&b'"') {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: object key must be string".into(),
                ));
            }
            let key = Self::parse_json_string(src, bytes, i)?;
            Self::json_skip_ws(bytes, i);
            if bytes.get(*i) != Some(&b':') {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: expected ':' after object key".into(),
                ));
            }
            *i += 1;
            let value = Self::parse_json_value(src, bytes, i)?;
            Self::object_set_entry(&mut entries, key, value);
            Self::json_skip_ws(bytes, i);

            match bytes.get(*i) {
                Some(b',') => {
                    *i += 1;
                }
                Some(b'}') => {
                    *i += 1;
                    break;
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: expected ',' or '}'".into(),
                    ));
                }
            }
        }

        Ok(Self::new_object_value(entries))
    }

    pub(crate) fn parse_json_array(src: &str, bytes: &[u8], i: &mut usize) -> Result<Value> {
        *i += 1; // consume '['
        Self::json_skip_ws(bytes, i);
        let mut items = Vec::new();

        if bytes.get(*i) == Some(&b']') {
            *i += 1;
            return Ok(Self::new_array_value(items));
        }

        loop {
            let item = Self::parse_json_value(src, bytes, i)?;
            items.push(item);
            Self::json_skip_ws(bytes, i);
            match bytes.get(*i) {
                Some(b',') => {
                    *i += 1;
                }
                Some(b']') => {
                    *i += 1;
                    break;
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: expected ',' or ']'".into(),
                    ));
                }
            }
        }

        Ok(Self::new_array_value(items))
    }

    pub(crate) fn parse_json_string(src: &str, bytes: &[u8], i: &mut usize) -> Result<String> {
        if bytes.get(*i) != Some(&b'"') {
            return Err(Error::ScriptRuntime(
                "JSON.parse invalid JSON: expected string".into(),
            ));
        }
        *i += 1;
        let mut out = String::new();

        while *i < bytes.len() {
            let b = bytes[*i];
            if b == b'"' {
                *i += 1;
                return Ok(out);
            }
            if b < 0x20 {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: unescaped control character in string".into(),
                ));
            }
            if b == b'\\' {
                *i += 1;
                let Some(&esc) = bytes.get(*i) else {
                    return Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: unterminated escape sequence".into(),
                    ));
                };
                match esc {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{0008}'),
                    b'f' => out.push('\u{000C}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => {
                        *i += 1;
                        let first = Self::parse_json_hex4(src, i)?;
                        if (0xD800..=0xDBFF).contains(&first) {
                            let Some(b'\\') = bytes.get(*i).copied() else {
                                return Err(Error::ScriptRuntime(
                                    "JSON.parse invalid JSON: invalid unicode surrogate pair"
                                        .into(),
                                ));
                            };
                            *i += 1;
                            let Some(b'u') = bytes.get(*i).copied() else {
                                return Err(Error::ScriptRuntime(
                                    "JSON.parse invalid JSON: invalid unicode surrogate pair"
                                        .into(),
                                ));
                            };
                            *i += 1;
                            let second = Self::parse_json_hex4(src, i)?;
                            if !(0xDC00..=0xDFFF).contains(&second) {
                                return Err(Error::ScriptRuntime(
                                    "JSON.parse invalid JSON: invalid unicode surrogate pair"
                                        .into(),
                                ));
                            }
                            let codepoint = 0x10000
                                + (((first as u32 - 0xD800) << 10) | (second as u32 - 0xDC00));
                            let Some(ch) = char::from_u32(codepoint) else {
                                return Err(Error::ScriptRuntime(
                                    "JSON.parse invalid JSON: invalid unicode escape".into(),
                                ));
                            };
                            out.push(ch);
                            continue;
                        } else if (0xDC00..=0xDFFF).contains(&first) {
                            return Err(Error::ScriptRuntime(
                                "JSON.parse invalid JSON: invalid unicode surrogate pair".into(),
                            ));
                        } else {
                            let Some(ch) = char::from_u32(first as u32) else {
                                return Err(Error::ScriptRuntime(
                                    "JSON.parse invalid JSON: invalid unicode escape".into(),
                                ));
                            };
                            out.push(ch);
                            continue;
                        }
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "JSON.parse invalid JSON: invalid escape sequence".into(),
                        ));
                    }
                }
                *i += 1;
                continue;
            }

            if b.is_ascii() {
                out.push(b as char);
                *i += 1;
            } else {
                let rest = src.get(*i..).ok_or_else(|| {
                    Error::ScriptRuntime("JSON.parse invalid JSON: invalid utf-8".into())
                })?;
                let Some(ch) = rest.chars().next() else {
                    return Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: invalid utf-8".into(),
                    ));
                };
                out.push(ch);
                *i += ch.len_utf8();
            }
        }

        Err(Error::ScriptRuntime(
            "JSON.parse invalid JSON: unterminated string".into(),
        ))
    }

    pub(crate) fn parse_json_hex4(src: &str, i: &mut usize) -> Result<u16> {
        let end = i.saturating_add(4);
        let segment = src.get(*i..end).ok_or_else(|| {
            Error::ScriptRuntime("JSON.parse invalid JSON: invalid unicode escape".into())
        })?;
        if !segment.as_bytes().iter().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::ScriptRuntime(
                "JSON.parse invalid JSON: invalid unicode escape".into(),
            ));
        }
        *i = end;
        u16::from_str_radix(segment, 16).map_err(|_| {
            Error::ScriptRuntime("JSON.parse invalid JSON: invalid unicode escape".into())
        })
    }

    pub(crate) fn parse_json_number(src: &str, bytes: &[u8], i: &mut usize) -> Result<Value> {
        let start = *i;

        if bytes.get(*i) == Some(&b'-') {
            *i += 1;
        }

        match bytes.get(*i).copied() {
            Some(b'0') => {
                *i += 1;
                if bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                    return Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: invalid number".into(),
                    ));
                }
            }
            Some(b'1'..=b'9') => {
                *i += 1;
                while bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                    *i += 1;
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: invalid number".into(),
                ));
            }
        }

        if bytes.get(*i) == Some(&b'.') {
            *i += 1;
            if !bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: invalid number".into(),
                ));
            }
            while bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                *i += 1;
            }
        }

        if bytes.get(*i).is_some_and(|b| *b == b'e' || *b == b'E') {
            *i += 1;
            if bytes.get(*i).is_some_and(|b| *b == b'+' || *b == b'-') {
                *i += 1;
            }
            if !bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: invalid number".into(),
                ));
            }
            while bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                *i += 1;
            }
        }

        let token = src.get(start..*i).ok_or_else(|| {
            Error::ScriptRuntime("JSON.parse invalid JSON: invalid number".into())
        })?;
        if !token.contains('.') && !token.contains('e') && !token.contains('E') {
            if let Ok(n) = token.parse::<i64>() {
                return Ok(Value::Number(n));
            }
        }
        let n = token
            .parse::<f64>()
            .map_err(|_| Error::ScriptRuntime("JSON.parse invalid JSON: invalid number".into()))?;
        Ok(Value::Float(n))
    }

    pub(crate) fn json_skip_ws(bytes: &[u8], i: &mut usize) {
        while bytes.get(*i).is_some_and(|b| b.is_ascii_whitespace()) {
            *i += 1;
        }
    }

    pub(crate) fn json_consume_ascii(bytes: &[u8], i: &mut usize, token: &str) -> bool {
        let token_bytes = token.as_bytes();
        let end = i.saturating_add(token_bytes.len());
        if end <= bytes.len() && &bytes[*i..end] == token_bytes {
            *i = end;
            true
        } else {
            false
        }
    }
}
