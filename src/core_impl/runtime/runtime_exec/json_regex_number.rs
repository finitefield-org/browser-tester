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

    pub(crate) fn json_stringify_top_level(value: &Value) -> Result<Option<String>> {
        let mut array_stack = Vec::new();
        let mut object_stack = Vec::new();
        Self::json_stringify_value(value, &mut array_stack, &mut object_stack)
    }

    pub(crate) fn json_stringify_value(
        value: &Value,
        array_stack: &mut Vec<usize>,
        object_stack: &mut Vec<usize>,
    ) -> Result<Option<String>> {
        match value {
            Value::String(v) => Ok(Some(format!("\"{}\"", Self::json_escape_string(v)))),
            Value::Bool(v) => Ok(Some(if *v { "true".into() } else { "false".into() })),
            Value::Number(v) => Ok(Some(v.to_string())),
            Value::Float(v) => {
                if v.is_finite() {
                    Ok(Some(format_float(*v)))
                } else {
                    Ok(Some("null".into()))
                }
            }
            Value::BigInt(_) => Err(Error::ScriptRuntime(
                "JSON.stringify does not support BigInt values".into(),
            )),
            Value::Null => Ok(Some("null".into())),
            Value::Undefined
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::Symbol(_)
            | Value::PromiseCapability(_)
            | Value::Function(_) => Ok(None),
            Value::RegExp(_) => Ok(Some("{}".to_string())),
            Value::Date(v) => Ok(Some(format!(
                "\"{}\"",
                Self::json_escape_string(&Self::format_iso_8601_utc(*v.borrow()))
            ))),
            Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_) => Ok(Some("{}".to_string())),
            Value::Array(values) => {
                let ptr = Rc::as_ptr(values) as usize;
                if array_stack.contains(&ptr) {
                    return Err(Error::ScriptRuntime(
                        "JSON.stringify circular structure".into(),
                    ));
                }
                array_stack.push(ptr);

                let items = values.borrow();
                let mut out = String::from("[");
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        out.push(',');
                    }
                    let serialized = Self::json_stringify_value(item, array_stack, object_stack)?
                        .unwrap_or_else(|| "null".to_string());
                    out.push_str(&serialized);
                }
                out.push(']');

                array_stack.pop();
                Ok(Some(out))
            }
            Value::Object(entries) => {
                let ptr = Rc::as_ptr(entries) as usize;
                if object_stack.contains(&ptr) {
                    return Err(Error::ScriptRuntime(
                        "JSON.stringify circular structure".into(),
                    ));
                }
                object_stack.push(ptr);

                let entries = entries.borrow();
                let mut out = String::from("{");
                let mut wrote = false;
                for (key, value) in entries.iter() {
                    if Self::is_internal_object_key(key) {
                        continue;
                    }
                    let Some(serialized) =
                        Self::json_stringify_value(value, array_stack, object_stack)?
                    else {
                        continue;
                    };
                    if wrote {
                        out.push(',');
                    }
                    wrote = true;
                    out.push('"');
                    out.push_str(&Self::json_escape_string(key));
                    out.push_str("\":");
                    out.push_str(&serialized);
                }
                out.push('}');

                object_stack.pop();
                Ok(Some(out))
            }
        }
    }

    pub(crate) fn json_escape_string(src: &str) -> String {
        let mut out = String::new();
        for ch in src.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\u{0008}' => out.push_str("\\b"),
                '\u{000C}' => out.push_str("\\f"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c <= '\u{001F}' => {
                    out.push_str(&format!("\\u{:04X}", c as u32));
                }
                c => out.push(c),
            }
        }
        out
    }

    pub(crate) fn structured_clone_value(
        value: &Value,
        array_stack: &mut Vec<usize>,
        object_stack: &mut Vec<usize>,
    ) -> Result<Value> {
        match value {
            Value::String(v) => Ok(Value::String(v.clone())),
            Value::Bool(v) => Ok(Value::Bool(*v)),
            Value::Number(v) => Ok(Value::Number(*v)),
            Value::Float(v) => Ok(Value::Float(*v)),
            Value::BigInt(v) => Ok(Value::BigInt(v.clone())),
            Value::Null => Ok(Value::Null),
            Value::Undefined => Ok(Value::Undefined),
            Value::Date(v) => Ok(Value::Date(Rc::new(RefCell::new(*v.borrow())))),
            Value::RegExp(v) => {
                let v = v.borrow();
                let cloned = Self::new_regex_value(v.source.clone(), v.flags.clone())?;
                let Value::RegExp(cloned_regex) = &cloned else {
                    unreachable!("RegExp clone must produce RegExp value");
                };
                {
                    let mut cloned_regex = cloned_regex.borrow_mut();
                    cloned_regex.last_index = v.last_index;
                    cloned_regex.properties = v.properties.clone();
                }
                Ok(cloned)
            }
            Value::ArrayBuffer(buffer) => {
                let buffer = buffer.borrow();
                Ok(Value::ArrayBuffer(Rc::new(RefCell::new(
                    ArrayBufferValue {
                        bytes: buffer.bytes.clone(),
                        max_byte_length: buffer.max_byte_length,
                        detached: buffer.detached,
                    },
                ))))
            }
            Value::TypedArray(array) => {
                let array = array.borrow();
                let buffer = array.buffer.borrow();
                let cloned_buffer = Rc::new(RefCell::new(ArrayBufferValue {
                    bytes: buffer.bytes.clone(),
                    max_byte_length: buffer.max_byte_length,
                    detached: buffer.detached,
                }));
                Ok(Value::TypedArray(Rc::new(RefCell::new(TypedArrayValue {
                    kind: array.kind,
                    buffer: cloned_buffer,
                    byte_offset: array.byte_offset,
                    fixed_length: array.fixed_length,
                }))))
            }
            Value::Blob(blob) => {
                let blob = blob.borrow();
                Ok(Self::new_blob_value(
                    blob.bytes.clone(),
                    blob.mime_type.clone(),
                ))
            }
            Value::Map(map) => {
                let map = map.borrow();
                Ok(Value::Map(Rc::new(RefCell::new(MapValue {
                    entries: map.entries.clone(),
                    properties: map.properties.clone(),
                }))))
            }
            Value::Set(set) => {
                let set = set.borrow();
                Ok(Value::Set(Rc::new(RefCell::new(SetValue {
                    values: set.values.clone(),
                    properties: set.properties.clone(),
                }))))
            }
            Value::Array(values) => {
                let ptr = Rc::as_ptr(values) as usize;
                if array_stack.contains(&ptr) {
                    return Err(Error::ScriptRuntime(
                        "structuredClone does not support circular values".into(),
                    ));
                }
                array_stack.push(ptr);

                let items = values.borrow();
                let mut cloned = Vec::with_capacity(items.len());
                for item in items.iter() {
                    cloned.push(Self::structured_clone_value(
                        item,
                        array_stack,
                        object_stack,
                    )?);
                }
                array_stack.pop();

                Ok(Self::new_array_value(cloned))
            }
            Value::Object(entries) => {
                let ptr = Rc::as_ptr(entries) as usize;
                if object_stack.contains(&ptr) {
                    return Err(Error::ScriptRuntime(
                        "structuredClone does not support circular values".into(),
                    ));
                }
                object_stack.push(ptr);

                let entries = entries.borrow();
                let mut cloned = Vec::with_capacity(entries.len());
                for (key, value) in entries.iter() {
                    let value = Self::structured_clone_value(value, array_stack, object_stack)?;
                    cloned.push((key.clone(), value));
                }
                object_stack.pop();

                Ok(Self::new_object_value(cloned))
            }
            Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Promise(_)
            | Value::Symbol(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Function(_) => Err(Error::ScriptRuntime(
                "structuredClone value is not cloneable".into(),
            )),
        }
    }

    pub(crate) fn analyze_regex_flags(flags: &str) -> std::result::Result<RegexFlags, String> {
        let mut info = RegexFlags {
            global: false,
            ignore_case: false,
            multiline: false,
            dot_all: false,
            sticky: false,
            has_indices: false,
            unicode: false,
        };
        let mut seen = HashSet::new();
        for ch in flags.chars() {
            if !seen.insert(ch) {
                return Err(format!("invalid regular expression flags: {flags}"));
            }
            match ch {
                'g' => info.global = true,
                'i' => info.ignore_case = true,
                'm' => info.multiline = true,
                's' => info.dot_all = true,
                'y' => info.sticky = true,
                'd' => info.has_indices = true,
                'u' => info.unicode = true,
                'v' => {
                    return Err("invalid regular expression flags: v flag is not supported".into());
                }
                _ => return Err(format!("invalid regular expression flags: {flags}")),
            }
        }
        Ok(info)
    }

    pub(crate) fn compile_regex(
        pattern: &str,
        info: RegexFlags,
    ) -> std::result::Result<Regex, RegexError> {
        let mut builder = RegexBuilder::new(pattern);
        builder.case_insensitive(info.ignore_case);
        builder.multi_line(info.multiline);
        builder.dot_matches_new_line(info.dot_all);
        builder.build()
    }

    pub(crate) fn new_regex_value(pattern: String, flags: String) -> Result<Value> {
        let info = Self::analyze_regex_flags(&flags).map_err(Error::ScriptRuntime)?;
        let compiled = Self::compile_regex(&pattern, info).map_err(|err| {
            Error::ScriptRuntime(format!(
                "invalid regular expression: /{pattern}/{flags}: {err}"
            ))
        })?;
        Ok(Value::RegExp(Rc::new(RefCell::new(RegexValue {
            source: pattern,
            flags,
            global: info.global,
            ignore_case: info.ignore_case,
            multiline: info.multiline,
            dot_all: info.dot_all,
            sticky: info.sticky,
            has_indices: info.has_indices,
            unicode: info.unicode,
            compiled,
            last_index: 0,
            properties: ObjectValue::default(),
        }))))
    }

    pub(crate) fn new_regex_from_values(pattern: &Value, flags: Option<&Value>) -> Result<Value> {
        let pattern_text = match pattern {
            Value::RegExp(value) => value.borrow().source.clone(),
            _ => pattern.as_string(),
        };
        let flags_text = if let Some(flags) = flags {
            flags.as_string()
        } else if let Value::RegExp(value) = pattern {
            value.borrow().flags.clone()
        } else {
            String::new()
        };
        Self::new_regex_value(pattern_text, flags_text)
    }

    pub(crate) fn resolve_regex_from_value(value: &Value) -> Result<Rc<RefCell<RegexValue>>> {
        match value {
            Value::RegExp(regex) => Ok(regex.clone()),
            _ => Err(Error::ScriptRuntime("value is not a RegExp".into())),
        }
    }

    pub(crate) fn map_regex_runtime_error(err: RegexError) -> Error {
        Error::ScriptRuntime(format!("regular expression runtime error: {err}"))
    }

    pub(crate) fn regex_test(regex: &Rc<RefCell<RegexValue>>, input: &str) -> Result<bool> {
        Ok(Self::regex_exec_internal(regex, input)?.is_some())
    }

    pub(crate) fn regex_exec(
        regex: &Rc<RefCell<RegexValue>>,
        input: &str,
    ) -> Result<Option<Vec<String>>> {
        Self::regex_exec_internal(regex, input)
    }

    pub(crate) fn regex_exec_internal(
        regex: &Rc<RefCell<RegexValue>>,
        input: &str,
    ) -> Result<Option<Vec<String>>> {
        let mut regex = regex.borrow_mut();
        let start = if regex.global || regex.sticky {
            regex.last_index
        } else {
            0
        };
        if start > input.len() {
            regex.last_index = 0;
            return Ok(None);
        }

        let captures = regex
            .compiled
            .captures_from_pos(input, start)
            .map_err(Self::map_regex_runtime_error)?;

        let Some(captures) = captures else {
            if regex.global || regex.sticky {
                regex.last_index = 0;
            }
            return Ok(None);
        };

        let Some(full_match) = captures.get(0) else {
            if regex.global || regex.sticky {
                regex.last_index = 0;
            }
            return Ok(None);
        };

        if regex.sticky && full_match.start() != start {
            regex.last_index = 0;
            return Ok(None);
        }

        if regex.global || regex.sticky {
            regex.last_index = full_match.end();
        }

        let mut out = Vec::with_capacity(captures.len());
        for idx in 0..captures.len() {
            out.push(
                captures
                    .get(idx)
                    .map(|capture| capture.as_str().to_string())
                    .unwrap_or_default(),
            );
        }
        Ok(Some(out))
    }

    pub(crate) fn eval_math_method(
        &mut self,
        method: MathMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }

        let single = |values: &[Value]| Self::coerce_number_for_global(&values[0]);

        match method {
            MathMethod::Abs => Ok(Value::Float(single(&values).abs())),
            MathMethod::Acos => Ok(Value::Float(single(&values).acos())),
            MathMethod::Acosh => Ok(Value::Float(single(&values).acosh())),
            MathMethod::Asin => Ok(Value::Float(single(&values).asin())),
            MathMethod::Asinh => Ok(Value::Float(single(&values).asinh())),
            MathMethod::Atan => Ok(Value::Float(single(&values).atan())),
            MathMethod::Atan2 => Ok(Value::Float(
                Self::coerce_number_for_global(&values[0])
                    .atan2(Self::coerce_number_for_global(&values[1])),
            )),
            MathMethod::Atanh => Ok(Value::Float(single(&values).atanh())),
            MathMethod::Cbrt => Ok(Value::Float(single(&values).cbrt())),
            MathMethod::Ceil => Ok(Value::Float(single(&values).ceil())),
            MathMethod::Clz32 => Ok(Value::Number(i64::from(
                Self::to_u32_for_math(&values[0]).leading_zeros(),
            ))),
            MathMethod::Cos => Ok(Value::Float(single(&values).cos())),
            MathMethod::Cosh => Ok(Value::Float(single(&values).cosh())),
            MathMethod::Exp => Ok(Value::Float(single(&values).exp())),
            MathMethod::Expm1 => Ok(Value::Float(single(&values).exp_m1())),
            MathMethod::Floor => Ok(Value::Float(single(&values).floor())),
            MathMethod::F16Round => Ok(Value::Float(Self::math_f16round(single(&values)))),
            MathMethod::FRound => Ok(Value::Float((single(&values) as f32) as f64)),
            MathMethod::Hypot => {
                let mut sum = 0.0f64;
                for value in values {
                    let value = Self::coerce_number_for_global(&value);
                    sum += value * value;
                }
                Ok(Value::Float(sum.sqrt()))
            }
            MathMethod::Imul => {
                let left = Self::to_i32_for_math(&values[0]);
                let right = Self::to_i32_for_math(&values[1]);
                Ok(Value::Number(i64::from(left.wrapping_mul(right))))
            }
            MathMethod::Log => Ok(Value::Float(single(&values).ln())),
            MathMethod::Log10 => Ok(Value::Float(single(&values).log10())),
            MathMethod::Log1p => Ok(Value::Float(single(&values).ln_1p())),
            MathMethod::Log2 => Ok(Value::Float(single(&values).log2())),
            MathMethod::Max => {
                let mut out = f64::NEG_INFINITY;
                for value in values {
                    out = out.max(Self::coerce_number_for_global(&value));
                }
                Ok(Value::Float(out))
            }
            MathMethod::Min => {
                let mut out = f64::INFINITY;
                for value in values {
                    out = out.min(Self::coerce_number_for_global(&value));
                }
                Ok(Value::Float(out))
            }
            MathMethod::Pow => Ok(Value::Float(
                Self::coerce_number_for_global(&values[0])
                    .powf(Self::coerce_number_for_global(&values[1])),
            )),
            MathMethod::Random => Ok(Value::Float(self.next_random_f64())),
            MathMethod::Round => Ok(Value::Float(Self::js_math_round(single(&values)))),
            MathMethod::Sign => Ok(Value::Float(Self::js_math_sign(single(&values)))),
            MathMethod::Sin => Ok(Value::Float(single(&values).sin())),
            MathMethod::Sinh => Ok(Value::Float(single(&values).sinh())),
            MathMethod::Sqrt => Ok(Value::Float(single(&values).sqrt())),
            MathMethod::SumPrecise => match &values[0] {
                Value::Array(values) => Ok(Value::Float(Self::sum_precise(&values.borrow()))),
                _ => Err(Error::ScriptRuntime(
                    "Math.sumPrecise argument must be an array".into(),
                )),
            },
            MathMethod::Tan => Ok(Value::Float(single(&values).tan())),
            MathMethod::Tanh => Ok(Value::Float(single(&values).tanh())),
            MathMethod::Trunc => Ok(Value::Float(single(&values).trunc())),
        }
    }

    pub(crate) fn eval_number_method(
        &mut self,
        method: NumberMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }

        match method {
            NumberMethod::IsFinite => Ok(Value::Bool(
                Self::number_primitive_value(&values[0]).is_some_and(f64::is_finite),
            )),
            NumberMethod::IsInteger => Ok(Value::Bool(
                Self::number_primitive_value(&values[0])
                    .is_some_and(|value| value.is_finite() && value.fract() == 0.0),
            )),
            NumberMethod::IsNaN => Ok(Value::Bool(matches!(
                values[0],
                Value::Float(value) if value.is_nan()
            ))),
            NumberMethod::IsSafeInteger => Ok(Value::Bool(
                Self::number_primitive_value(&values[0]).is_some_and(|value| {
                    value.is_finite()
                        && value.fract() == 0.0
                        && value.abs() <= 9_007_199_254_740_991.0
                }),
            )),
            NumberMethod::ParseFloat => {
                Ok(Value::Float(parse_js_parse_float(&values[0].as_string())))
            }
            NumberMethod::ParseInt => {
                let radix = if values.len() == 2 {
                    Some(Self::value_to_i64(&values[1]))
                } else {
                    None
                };
                Ok(Value::Float(parse_js_parse_int(
                    &values[0].as_string(),
                    radix,
                )))
            }
        }
    }

    pub(crate) fn eval_number_instance_method(
        &mut self,
        method: NumberInstanceMethod,
        value: &Expr,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let value = self.eval_expr(value, env, event_param, event)?;
        let mut args_value = Vec::with_capacity(args.len());
        for arg in args {
            args_value.push(self.eval_expr(arg, env, event_param, event)?);
        }

        if let Value::BigInt(bigint) = &value {
            return match method {
                NumberInstanceMethod::ToLocaleString => Ok(Value::String(bigint.to_string())),
                NumberInstanceMethod::ToString => {
                    let radix = if let Some(arg) = args_value.first() {
                        let radix = Self::value_to_i64(arg);
                        if !(2..=36).contains(&radix) {
                            return Err(Error::ScriptRuntime(
                                "toString radix must be between 2 and 36".into(),
                            ));
                        }
                        radix as u32
                    } else {
                        10
                    };
                    Ok(Value::String(bigint.to_str_radix(radix)))
                }
                NumberInstanceMethod::ValueOf => Ok(Value::BigInt(bigint.clone())),
                NumberInstanceMethod::ToExponential
                | NumberInstanceMethod::ToFixed
                | NumberInstanceMethod::ToPrecision => Err(Error::ScriptRuntime(
                    "number formatting methods are not supported for BigInt values".into(),
                )),
            };
        }

        if let Value::Symbol(symbol) = &value {
            return match method {
                NumberInstanceMethod::ValueOf => Ok(Value::Symbol(symbol.clone())),
                NumberInstanceMethod::ToString | NumberInstanceMethod::ToLocaleString => {
                    if !args_value.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "Symbol.toString does not take arguments".into(),
                        ));
                    }
                    Ok(Value::String(Value::Symbol(symbol.clone()).as_string()))
                }
                NumberInstanceMethod::ToExponential
                | NumberInstanceMethod::ToFixed
                | NumberInstanceMethod::ToPrecision => Err(Error::ScriptRuntime(
                    "Cannot convert a Symbol value to a number".into(),
                )),
            };
        }

        let numeric = Self::coerce_number_for_number_constructor(&value);

        match method {
            NumberInstanceMethod::ToExponential => {
                let fraction_digits = if let Some(arg) = args_value.first() {
                    let fraction_digits = Self::value_to_i64(arg);
                    if !(0..=100).contains(&fraction_digits) {
                        return Err(Error::ScriptRuntime(
                            "toExponential fractionDigits must be between 0 and 100".into(),
                        ));
                    }
                    Some(fraction_digits as usize)
                } else {
                    None
                };
                Ok(Value::String(Self::number_to_exponential(
                    numeric,
                    fraction_digits,
                )))
            }
            NumberInstanceMethod::ToFixed => {
                let fraction_digits = if let Some(arg) = args_value.first() {
                    let fraction_digits = Self::value_to_i64(arg);
                    if !(0..=100).contains(&fraction_digits) {
                        return Err(Error::ScriptRuntime(
                            "toFixed fractionDigits must be between 0 and 100".into(),
                        ));
                    }
                    fraction_digits as usize
                } else {
                    0
                };
                Ok(Value::String(Self::number_to_fixed(
                    numeric,
                    fraction_digits,
                )))
            }
            NumberInstanceMethod::ToLocaleString => {
                Ok(Value::String(Self::format_number_default(numeric)))
            }
            NumberInstanceMethod::ToPrecision => {
                if let Some(arg) = args_value.first() {
                    let precision = Self::value_to_i64(arg);
                    if !(1..=100).contains(&precision) {
                        return Err(Error::ScriptRuntime(
                            "toPrecision precision must be between 1 and 100".into(),
                        ));
                    }
                    Ok(Value::String(Self::number_to_precision(
                        numeric,
                        precision as usize,
                    )))
                } else {
                    Ok(Value::String(Self::format_number_default(numeric)))
                }
            }
            NumberInstanceMethod::ToString => {
                let radix = if let Some(arg) = args_value.first() {
                    let radix = Self::value_to_i64(arg);
                    if !(2..=36).contains(&radix) {
                        return Err(Error::ScriptRuntime(
                            "toString radix must be between 2 and 36".into(),
                        ));
                    }
                    radix as u32
                } else {
                    10
                };
                Ok(Value::String(Self::number_to_string_radix(numeric, radix)))
            }
            NumberInstanceMethod::ValueOf => Ok(Self::number_value(numeric)),
        }
    }

    pub(crate) fn eval_bigint_method(
        &mut self,
        method: BigIntMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }
        let bits_i64 = Self::value_to_i64(&values[0]);
        if bits_i64 < 0 {
            return Err(Error::ScriptRuntime(
                "BigInt bit width must be a non-negative integer".into(),
            ));
        }
        let bits = usize::try_from(bits_i64)
            .map_err(|_| Error::ScriptRuntime("BigInt bit width is too large".into()))?;
        let value = Self::coerce_bigint_for_builtin_op(&values[1])?;
        let out = match method {
            BigIntMethod::AsIntN => Self::bigint_as_int_n(bits, &value),
            BigIntMethod::AsUintN => Self::bigint_as_uint_n(bits, &value),
        };
        Ok(Value::BigInt(out))
    }

    pub(crate) fn eval_bigint_instance_method(
        &mut self,
        method: BigIntInstanceMethod,
        value: &Expr,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let value = self.eval_expr(value, env, event_param, event)?;
        let value = Self::coerce_bigint_for_builtin_op(&value)?;
        let mut args_value = Vec::with_capacity(args.len());
        for arg in args {
            args_value.push(self.eval_expr(arg, env, event_param, event)?);
        }

        match method {
            BigIntInstanceMethod::ToLocaleString => Ok(Value::String(value.to_string())),
            BigIntInstanceMethod::ToString => {
                let radix = if let Some(arg) = args_value.first() {
                    let radix = Self::value_to_i64(arg);
                    if !(2..=36).contains(&radix) {
                        return Err(Error::ScriptRuntime(
                            "toString radix must be between 2 and 36".into(),
                        ));
                    }
                    radix as u32
                } else {
                    10
                };
                Ok(Value::String(value.to_str_radix(radix)))
            }
            BigIntInstanceMethod::ValueOf => Ok(Value::BigInt(value)),
        }
    }

    pub(crate) fn bigint_as_uint_n(bits: usize, value: &JsBigInt) -> JsBigInt {
        if bits == 0 {
            return JsBigInt::zero();
        }
        let modulo = JsBigInt::one() << bits;
        let mut out = value % &modulo;
        if out.sign() == Sign::Minus {
            out += &modulo;
        }
        out
    }

    pub(crate) fn bigint_as_int_n(bits: usize, value: &JsBigInt) -> JsBigInt {
        if bits == 0 {
            return JsBigInt::zero();
        }
        let modulo = JsBigInt::one() << bits;
        let threshold = JsBigInt::one() << (bits - 1);
        let unsigned = Self::bigint_as_uint_n(bits, value);
        if unsigned >= threshold {
            unsigned - modulo
        } else {
            unsigned
        }
    }

    pub(crate) fn coerce_bigint_for_constructor(value: &Value) -> Result<JsBigInt> {
        match value {
            Value::BigInt(value) => Ok(value.clone()),
            Value::Bool(value) => Ok(if *value {
                JsBigInt::one()
            } else {
                JsBigInt::zero()
            }),
            Value::Number(value) => Ok(JsBigInt::from(*value)),
            Value::Float(value) => {
                if value.is_finite() && value.fract() == 0.0 {
                    Ok(JsBigInt::from(*value as i64))
                } else {
                    Err(Error::ScriptRuntime(
                        "cannot convert Number value to BigInt".into(),
                    ))
                }
            }
            Value::String(value) => Self::parse_js_bigint_from_string(value),
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(
                "cannot convert null or undefined to BigInt".into(),
            )),
            Value::Date(value) => Ok(JsBigInt::from(*value.borrow())),
            Value::Array(values) => {
                let rendered = Value::Array(values.clone()).as_string();
                Self::parse_js_bigint_from_string(&rendered)
            }
            Value::Object(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Symbol(_)
            | Value::RegExp(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_) => Err(Error::ScriptRuntime(
                "cannot convert object value to BigInt".into(),
            )),
        }
    }

    pub(crate) fn coerce_bigint_for_builtin_op(value: &Value) -> Result<JsBigInt> {
        match value {
            Value::BigInt(value) => Ok(value.clone()),
            Value::Bool(value) => Ok(if *value {
                JsBigInt::one()
            } else {
                JsBigInt::zero()
            }),
            Value::String(value) => Self::parse_js_bigint_from_string(value),
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(
                "cannot convert null or undefined to BigInt".into(),
            )),
            Value::Number(_)
            | Value::Float(_)
            | Value::Date(_)
            | Value::Object(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Symbol(_)
            | Value::RegExp(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_)
            | Value::Array(_) => Err(Error::ScriptRuntime(
                "cannot convert value to BigInt".into(),
            )),
        }
    }

    pub(crate) fn parse_js_bigint_from_string(src: &str) -> Result<JsBigInt> {
        let trimmed = src.trim();
        if trimmed.is_empty() {
            return Ok(JsBigInt::zero());
        }

        if let Some(rest) = trimmed.strip_prefix('+') {
            return Self::parse_signed_decimal_bigint(rest, false);
        }
        if let Some(rest) = trimmed.strip_prefix('-') {
            return Self::parse_signed_decimal_bigint(rest, true);
        }

        if let Some(rest) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            return Self::parse_prefixed_bigint(rest, 16, trimmed);
        }
        if let Some(rest) = trimmed
            .strip_prefix("0o")
            .or_else(|| trimmed.strip_prefix("0O"))
        {
            return Self::parse_prefixed_bigint(rest, 8, trimmed);
        }
        if let Some(rest) = trimmed
            .strip_prefix("0b")
            .or_else(|| trimmed.strip_prefix("0B"))
        {
            return Self::parse_prefixed_bigint(rest, 2, trimmed);
        }

        Self::parse_signed_decimal_bigint(trimmed, false)
    }

    pub(crate) fn parse_prefixed_bigint(src: &str, radix: u32, original: &str) -> Result<JsBigInt> {
        if src.is_empty() {
            return Err(Error::ScriptRuntime(format!(
                "cannot convert {} to a BigInt",
                original
            )));
        }
        JsBigInt::parse_bytes(src.as_bytes(), radix)
            .ok_or_else(|| Error::ScriptRuntime(format!("cannot convert {} to a BigInt", original)))
    }

    pub(crate) fn parse_signed_decimal_bigint(src: &str, negative: bool) -> Result<JsBigInt> {
        let original = format!("{}{}", if negative { "-" } else { "" }, src);
        if src.is_empty() || !src.as_bytes().iter().all(u8::is_ascii_digit) {
            return Err(Error::ScriptRuntime(format!(
                "cannot convert {} to a BigInt",
                original
            )));
        }
        let mut value = JsBigInt::parse_bytes(src.as_bytes(), 10).ok_or_else(|| {
            Error::ScriptRuntime(format!("cannot convert {} to a BigInt", original))
        })?;
        if negative {
            value = -value;
        }
        Ok(value)
    }

    pub(crate) fn bigint_shift_left(value: &JsBigInt, shift: &JsBigInt) -> Result<JsBigInt> {
        if shift.sign() == Sign::Minus {
            let magnitude = (-shift)
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value >> magnitude)
        } else {
            let magnitude = shift
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value << magnitude)
        }
    }

    pub(crate) fn bigint_shift_right(value: &JsBigInt, shift: &JsBigInt) -> Result<JsBigInt> {
        if shift.sign() == Sign::Minus {
            let magnitude = (-shift)
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value << magnitude)
        } else {
            let magnitude = shift
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value >> magnitude)
        }
    }

    pub(crate) fn number_primitive_value(value: &Value) -> Option<f64> {
        match value {
            Value::Number(value) => Some(*value as f64),
            Value::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn number_value(value: f64) -> Value {
        if value == 0.0 && value.is_sign_negative() {
            return Value::Float(-0.0);
        }
        if value.is_finite()
            && value.fract() == 0.0
            && value >= i64::MIN as f64
            && value <= i64::MAX as f64
        {
            let integer = value as i64;
            if (integer as f64) == value {
                return Value::Number(integer);
            }
        }
        Value::Float(value)
    }

    pub(crate) fn coerce_number_for_number_constructor(value: &Value) -> f64 {
        match value {
            Value::Number(v) => *v as f64,
            Value::Float(v) => *v,
            Value::BigInt(v) => v.to_f64().unwrap_or_else(|| {
                if v.sign() == Sign::Minus {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                }
            }),
            Value::Bool(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            Value::String(v) => Self::parse_js_number_from_string(v),
            Value::Date(v) => *v.borrow() as f64,
            Value::Object(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Symbol(_)
            | Value::RegExp(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_) => f64::NAN,
            Value::Array(values) => {
                let rendered = Value::Array(values.clone()).as_string();
                Self::parse_js_number_from_string(&rendered)
            }
        }
    }

    pub(crate) fn parse_js_number_from_string(src: &str) -> f64 {
        let trimmed = src.trim();
        if trimmed.is_empty() {
            return 0.0;
        }
        if trimmed == "Infinity" || trimmed == "+Infinity" {
            return f64::INFINITY;
        }
        if trimmed == "-Infinity" {
            return f64::NEG_INFINITY;
        }

        if trimmed.starts_with('+') || trimmed.starts_with('-') {
            let rest = &trimmed[1..];
            if rest.starts_with("0x")
                || rest.starts_with("0X")
                || rest.starts_with("0o")
                || rest.starts_with("0O")
                || rest.starts_with("0b")
                || rest.starts_with("0B")
            {
                return f64::NAN;
            }
        }

        if let Some(digits) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            return Self::parse_prefixed_radix_to_f64(digits, 16);
        }
        if let Some(digits) = trimmed
            .strip_prefix("0o")
            .or_else(|| trimmed.strip_prefix("0O"))
        {
            return Self::parse_prefixed_radix_to_f64(digits, 8);
        }
        if let Some(digits) = trimmed
            .strip_prefix("0b")
            .or_else(|| trimmed.strip_prefix("0B"))
        {
            return Self::parse_prefixed_radix_to_f64(digits, 2);
        }

        trimmed.parse::<f64>().unwrap_or(f64::NAN)
    }

    pub(crate) fn parse_prefixed_radix_to_f64(src: &str, radix: u32) -> f64 {
        if src.is_empty() {
            return f64::NAN;
        }
        let mut out = 0.0f64;
        for ch in src.chars() {
            let Some(digit) = ch.to_digit(radix) else {
                return f64::NAN;
            };
            out = out * (radix as f64) + (digit as f64);
        }
        out
    }

    pub(crate) fn format_number_default(value: f64) -> String {
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

        if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            let integer = value as i64;
            if (integer as f64) == value {
                return integer.to_string();
            }
        }

        let out = format!("{value}");
        Self::normalize_exponential_string(out, false)
    }

    pub(crate) fn number_to_exponential(value: f64, fraction_digits: Option<usize>) -> String {
        if !value.is_finite() {
            return Self::format_number_default(value);
        }

        let out = if let Some(fraction_digits) = fraction_digits {
            format!(
                "{:.*e}",
                fraction_digits,
                if value == 0.0 { 0.0 } else { value }
            )
        } else {
            format!("{:e}", if value == 0.0 { 0.0 } else { value })
        };
        Self::normalize_exponential_string(out, fraction_digits.is_none())
    }

    pub(crate) fn normalize_exponential_string(raw: String, trim_fraction_zeros: bool) -> String {
        let Some(exp_idx) = raw.find('e').or_else(|| raw.find('E')) else {
            return raw;
        };
        let mut mantissa = raw[..exp_idx].to_string();
        let exponent_src = &raw[exp_idx + 1..];

        if trim_fraction_zeros && mantissa.contains('.') {
            while mantissa.ends_with('0') {
                mantissa.pop();
            }
            if mantissa.ends_with('.') {
                mantissa.pop();
            }
        }

        let exponent = exponent_src.parse::<i32>().unwrap_or(0);
        format!("{mantissa}e{:+}", exponent)
    }

    pub(crate) fn number_to_fixed(value: f64, fraction_digits: usize) -> String {
        if !value.is_finite() {
            return Self::format_number_default(value);
        }
        format!(
            "{:.*}",
            fraction_digits,
            if value == 0.0 { 0.0 } else { value }
        )
    }

    pub(crate) fn number_to_precision(value: f64, precision: usize) -> String {
        if !value.is_finite() {
            return Self::format_number_default(value);
        }
        if value == 0.0 {
            if precision == 1 {
                return "0".to_string();
            }
            return format!("0.{}", "0".repeat(precision - 1));
        }

        let abs = value.abs();
        let exponent = abs.log10().floor() as i32;
        if exponent < -6 || exponent >= precision as i32 {
            return Self::number_to_exponential(value, Some(precision.saturating_sub(1)));
        }

        let fraction_digits = (precision as i32 - exponent - 1).max(0) as usize;
        format!(
            "{:.*}",
            fraction_digits,
            if value == 0.0 { 0.0 } else { value }
        )
    }

    pub(crate) fn number_to_string_radix(value: f64, radix: u32) -> String {
        if radix == 10 {
            return Self::format_number_default(value);
        }
        if !value.is_finite() {
            return Self::format_number_default(value);
        }
        if value == 0.0 {
            return "0".to_string();
        }

        let sign = if value < 0.0 { "-" } else { "" };
        let abs = value.abs();
        let int_part = abs.trunc();
        let mut int_digits = Vec::new();
        let mut n = int_part;
        let radix_f64 = radix as f64;

        while n >= 1.0 {
            let digit = (n % radix_f64).floor() as u32;
            int_digits.push(Self::radix_digit_char(digit));
            n = (n / radix_f64).floor();
        }
        if int_digits.is_empty() {
            int_digits.push('0');
        }
        int_digits.reverse();
        let int_str: String = int_digits.into_iter().collect();

        let mut frac = abs - int_part;
        if frac == 0.0 {
            return format!("{sign}{int_str}");
        }

        let mut frac_str = String::new();
        let mut digits = 0usize;
        while frac > 0.0 && digits < 16 {
            frac *= radix_f64;
            let digit = frac.floor() as u32;
            frac_str.push(Self::radix_digit_char(digit));
            frac -= digit as f64;
            digits += 1;
            if frac.abs() < f64::EPSILON {
                break;
            }
        }
        while frac_str.ends_with('0') {
            frac_str.pop();
        }

        if frac_str.is_empty() {
            format!("{sign}{int_str}")
        } else {
            format!("{sign}{int_str}.{frac_str}")
        }
    }

    pub(crate) fn radix_digit_char(value: u32) -> char {
        if value < 10 {
            char::from(b'0' + value as u8)
        } else {
            char::from(b'a' + (value - 10) as u8)
        }
    }

    pub(crate) fn sum_precise(values: &[Value]) -> f64 {
        let mut sum = 0.0f64;
        let mut compensation = 0.0f64;
        for value in values {
            let value = Self::coerce_number_for_global(value);
            let adjusted = value - compensation;
            let next = sum + adjusted;
            compensation = (next - sum) - adjusted;
            sum = next;
        }
        sum
    }

    pub(crate) fn js_math_round(value: f64) -> f64 {
        if !value.is_finite() || value == 0.0 {
            return value;
        }
        if (-0.5..0.0).contains(&value) {
            return -0.0;
        }
        let floor = value.floor();
        let frac = value - floor;
        if frac < 0.5 { floor } else { floor + 1.0 }
    }

    pub(crate) fn js_math_sign(value: f64) -> f64 {
        if value.is_nan() {
            f64::NAN
        } else if value == 0.0 {
            value
        } else if value > 0.0 {
            1.0
        } else {
            -1.0
        }
    }

    pub(crate) fn to_i32_for_math(value: &Value) -> i32 {
        let numeric = Self::coerce_number_for_global(value);
        if !numeric.is_finite() {
            return 0;
        }
        let unsigned = numeric.trunc().rem_euclid(4_294_967_296.0);
        if unsigned >= 2_147_483_648.0 {
            (unsigned - 4_294_967_296.0) as i32
        } else {
            unsigned as i32
        }
    }

    pub(crate) fn to_u32_for_math(value: &Value) -> u32 {
        let numeric = Self::coerce_number_for_global(value);
        if !numeric.is_finite() {
            return 0;
        }
        numeric.trunc().rem_euclid(4_294_967_296.0) as u32
    }

    pub(crate) fn math_f16round(value: f64) -> f64 {
        let half = Self::f32_to_f16_bits(value as f32);
        Self::f16_bits_to_f32(half) as f64
    }

    pub(crate) fn f32_to_f16_bits(value: f32) -> u16 {
        let bits = value.to_bits();
        let sign = ((bits >> 16) & 0x8000) as u16;
        let exp = ((bits >> 23) & 0xff) as i32;
        let mant = bits & 0x007f_ffff;

        if exp == 0xff {
            if mant == 0 {
                return sign | 0x7c00;
            }
            return sign | 0x7e00;
        }

        let exp16 = exp - 127 + 15;
        if exp16 >= 0x1f {
            return sign | 0x7c00;
        }

        if exp16 <= 0 {
            if exp16 < -10 {
                return sign;
            }
            let mantissa = mant | 0x0080_0000;
            let shift = (14 - exp16) as u32;
            let mut half_mant = mantissa >> shift;
            let round_bit = 1u32 << (shift - 1);
            if (mantissa & round_bit) != 0
                && ((mantissa & (round_bit - 1)) != 0 || (half_mant & 1) != 0)
            {
                half_mant += 1;
            }
            return sign | (half_mant as u16);
        }

        let mut half_exp = (exp16 as u16) << 10;
        let mut half_mant = (mant >> 13) as u16;
        let round_bits = mant & 0x1fff;
        if round_bits > 0x1000 || (round_bits == 0x1000 && (half_mant & 1) != 0) {
            half_mant = half_mant.wrapping_add(1);
            if half_mant == 0x0400 {
                half_mant = 0;
                half_exp = half_exp.wrapping_add(0x0400);
                if half_exp >= 0x7c00 {
                    return sign | 0x7c00;
                }
            }
        }
        sign | half_exp | half_mant
    }

    pub(crate) fn f16_bits_to_f32(bits: u16) -> f32 {
        let sign = ((bits & 0x8000) as u32) << 16;
        let exp = ((bits >> 10) & 0x1f) as u32;
        let mant = (bits & 0x03ff) as u32;

        let out_bits = if exp == 0 {
            if mant == 0 {
                sign
            } else {
                let mut mantissa = mant;
                let mut exp_val = -14i32;
                while (mantissa & 0x0400) == 0 {
                    mantissa <<= 1;
                    exp_val -= 1;
                }
                mantissa &= 0x03ff;
                let exp32 = ((exp_val + 127) as u32) << 23;
                sign | exp32 | (mantissa << 13)
            }
        } else if exp == 0x1f {
            sign | 0x7f80_0000 | (mant << 13)
        } else {
            let exp32 = (((exp as i32) - 15 + 127) as u32) << 23;
            sign | exp32 | (mant << 13)
        };

        f32::from_bits(out_bits)
    }
}
