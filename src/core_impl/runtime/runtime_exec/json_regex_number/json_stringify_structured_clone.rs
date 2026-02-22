use super::*;

impl Harness {
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
            | Value::WeakMapConstructor
            | Value::SetConstructor
            | Value::WeakSetConstructor
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
            | Value::WeakMap(_)
            | Value::Set(_)
            | Value::WeakSet(_)
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
            Value::WeakMap(_) => Err(Error::ScriptRuntime(
                "structuredClone value is not cloneable".into(),
            )),
            Value::Set(set) => {
                let set = set.borrow();
                Ok(Value::Set(Rc::new(RefCell::new(SetValue {
                    values: set.values.clone(),
                    properties: set.properties.clone(),
                }))))
            }
            Value::WeakSet(_) => Err(Error::ScriptRuntime(
                "structuredClone value is not cloneable".into(),
            )),
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
            | Value::WeakMapConstructor
            | Value::SetConstructor
            | Value::WeakSetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Function(_) => Err(Error::ScriptRuntime(
                "structuredClone value is not cloneable".into(),
            )),
        }
    }
}
