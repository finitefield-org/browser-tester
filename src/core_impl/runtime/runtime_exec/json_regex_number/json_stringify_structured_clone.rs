use super::*;

#[derive(Default)]
struct StructuredCloneState {
    transfer_array_buffers: HashSet<usize>,
    dates: HashMap<usize, Rc<RefCell<i64>>>,
    regexps: HashMap<usize, Rc<RefCell<RegexValue>>>,
    arrays: HashMap<usize, Rc<RefCell<ArrayValue>>>,
    objects: HashMap<usize, Rc<RefCell<ObjectValue>>>,
    array_buffers: HashMap<usize, Rc<RefCell<ArrayBufferValue>>>,
    typed_arrays: HashMap<usize, Rc<RefCell<TypedArrayValue>>>,
    blobs: HashMap<usize, Rc<RefCell<BlobValue>>>,
    maps: HashMap<usize, Rc<RefCell<MapValue>>>,
    sets: HashMap<usize, Rc<RefCell<SetValue>>>,
}

impl Harness {
    pub(crate) fn json_stringify_top_level(
        value: &Value,
        space: Option<&Value>,
    ) -> Result<Option<String>> {
        let gap = Self::json_stringify_gap(space);
        let mut array_stack = Vec::new();
        let mut object_stack = Vec::new();
        Self::json_stringify_value(value, &mut array_stack, &mut object_stack, &gap, 0)
    }

    fn json_stringify_gap(space: Option<&Value>) -> String {
        match space {
            Some(Value::Number(width)) => " ".repeat((*width).clamp(0, 10) as usize),
            Some(Value::Float(width)) if width.is_finite() => {
                " ".repeat(width.trunc().clamp(0.0, 10.0) as usize)
            }
            Some(Value::String(text)) => text.chars().take(10).collect(),
            _ => String::new(),
        }
    }

    pub(crate) fn json_stringify_value(
        value: &Value,
        array_stack: &mut Vec<usize>,
        object_stack: &mut Vec<usize>,
        gap: &str,
        depth: usize,
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
                let out = if items.is_empty() {
                    "[]".to_string()
                } else if gap.is_empty() {
                    let mut out = String::from("[");
                    for (idx, item) in items.iter().enumerate() {
                        if idx > 0 {
                            out.push(',');
                        }
                        let serialized = Self::json_stringify_value(
                            item,
                            array_stack,
                            object_stack,
                            gap,
                            depth + 1,
                        )?
                        .unwrap_or_else(|| "null".to_string());
                        out.push_str(&serialized);
                    }
                    out.push(']');
                    out
                } else {
                    let indent = gap.repeat(depth + 1);
                    let closing_indent = gap.repeat(depth);
                    let mut lines = Vec::with_capacity(items.len());
                    for item in items.iter() {
                        let serialized = Self::json_stringify_value(
                            item,
                            array_stack,
                            object_stack,
                            gap,
                            depth + 1,
                        )?
                        .unwrap_or_else(|| "null".to_string());
                        lines.push(format!("{indent}{serialized}"));
                    }
                    format!("[\n{}\n{closing_indent}]", lines.join(",\n"))
                };

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
                let mut pairs = Vec::new();
                for (key, value) in entries.iter() {
                    if Self::is_internal_object_key(key) {
                        continue;
                    }
                    let Some(serialized) = Self::json_stringify_value(
                        value,
                        array_stack,
                        object_stack,
                        gap,
                        depth + 1,
                    )?
                    else {
                        continue;
                    };
                    if gap.is_empty() {
                        pairs.push(format!(
                            "\"{}\":{}",
                            Self::json_escape_string(key),
                            serialized
                        ));
                    } else {
                        let indent = gap.repeat(depth + 1);
                        pairs.push(format!(
                            "{indent}\"{}\": {}",
                            Self::json_escape_string(key),
                            serialized
                        ));
                    }
                }
                let out = if pairs.is_empty() {
                    "{}".to_string()
                } else if gap.is_empty() {
                    format!("{{{}}}", pairs.join(","))
                } else {
                    let closing_indent = gap.repeat(depth);
                    format!("{{\n{}\n{closing_indent}}}", pairs.join(",\n"))
                };

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

    fn structured_clone_not_cloneable_error() -> Error {
        Error::ScriptRuntime("DataCloneError: structuredClone value is not cloneable".into())
    }

    fn structured_clone_transfer_error(message: &str) -> Error {
        Error::ScriptRuntime(format!("DataCloneError: {message}"))
    }

    fn structured_clone_transfer_array_buffer_ids(options: Option<&Value>) -> Result<HashSet<usize>> {
        let Some(options) = options else {
            return Ok(HashSet::new());
        };

        match options {
            Value::Undefined | Value::Null => Ok(HashSet::new()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                let transfer = Self::object_get_entry(&entries, "transfer");
                match transfer {
                    None | Some(Value::Undefined | Value::Null) => Ok(HashSet::new()),
                    Some(Value::Array(values)) => {
                        let values = values.borrow();
                        let mut ids = HashSet::new();
                        for value in values.iter() {
                            let Value::ArrayBuffer(buffer) = value else {
                                return Err(Self::structured_clone_transfer_error(
                                    "structuredClone transfer list items must be transferable",
                                ));
                            };
                            if buffer.borrow().detached {
                                return Err(Self::structured_clone_transfer_error(
                                    "Cannot transfer a detached ArrayBuffer",
                                ));
                            }
                            let id = Rc::as_ptr(&buffer) as usize;
                            if !ids.insert(id) {
                                return Err(Self::structured_clone_transfer_error(
                                    "structuredClone transfer list contains duplicate items",
                                ));
                            }
                        }
                        Ok(ids)
                    }
                    Some(_) => Err(Self::structured_clone_transfer_error(
                        "structuredClone transfer option must be an array",
                    )),
                }
            }
            _ => Err(Error::ScriptRuntime(
                "TypeError: structuredClone options must be an object".into(),
            )),
        }
    }

    fn structured_clone_array_buffer_ref(
        buffer: &Rc<RefCell<ArrayBufferValue>>,
        state: &mut StructuredCloneState,
    ) -> Result<Rc<RefCell<ArrayBufferValue>>> {
        let source_id = Rc::as_ptr(buffer) as usize;
        if let Some(cloned) = state.array_buffers.get(&source_id) {
            return Ok(cloned.clone());
        }

        let cloned = if state.transfer_array_buffers.contains(&source_id) {
            let mut source = buffer.borrow_mut();
            if source.detached {
                return Err(Self::structured_clone_transfer_error(
                    "Cannot transfer a detached ArrayBuffer",
                ));
            }

            let bytes = source.bytes.clone();
            let max_byte_length = source.max_byte_length;
            source.bytes.clear();
            source.max_byte_length = None;
            source.detached = true;
            drop(source);

            Rc::new(RefCell::new(ArrayBufferValue {
                bytes,
                max_byte_length,
                detached: false,
            }))
        } else {
            let source = buffer.borrow();
            Rc::new(RefCell::new(ArrayBufferValue {
                bytes: source.bytes.clone(),
                max_byte_length: source.max_byte_length,
                detached: source.detached,
            }))
        };

        state.array_buffers.insert(source_id, cloned.clone());
        Ok(cloned)
    }

    fn structured_clone_internal(value: &Value, state: &mut StructuredCloneState) -> Result<Value> {
        match value {
            Value::String(v) => Ok(Value::String(v.clone())),
            Value::Bool(v) => Ok(Value::Bool(*v)),
            Value::Number(v) => Ok(Value::Number(*v)),
            Value::Float(v) => Ok(Value::Float(*v)),
            Value::BigInt(v) => Ok(Value::BigInt(v.clone())),
            Value::Null => Ok(Value::Null),
            Value::Undefined => Ok(Value::Undefined),
            Value::Date(v) => {
                let source_id = Rc::as_ptr(v) as usize;
                if let Some(cloned) = state.dates.get(&source_id) {
                    return Ok(Value::Date(cloned.clone()));
                }
                let cloned = Rc::new(RefCell::new(*v.borrow()));
                state.dates.insert(source_id, cloned.clone());
                Ok(Value::Date(cloned))
            }
            Value::RegExp(v) => {
                let source_id = Rc::as_ptr(v) as usize;
                if let Some(cloned) = state.regexps.get(&source_id) {
                    return Ok(Value::RegExp(cloned.clone()));
                }

                let v = v.borrow();
                let cloned = Self::new_regex_value(v.source.clone(), v.flags.clone())?;
                let Value::RegExp(cloned_regex) = &cloned else {
                    unreachable!("RegExp clone must produce RegExp value");
                };
                state.regexps.insert(source_id, cloned_regex.clone());
                {
                    let mut cloned_regex = cloned_regex.borrow_mut();
                    cloned_regex.last_index = v.last_index;
                    cloned_regex.properties = v.properties.clone();
                }
                Ok(Value::RegExp(cloned_regex.clone()))
            }
            Value::ArrayBuffer(buffer) => {
                let cloned = Self::structured_clone_array_buffer_ref(buffer, state)?;
                Ok(Value::ArrayBuffer(cloned))
            }
            Value::TypedArray(array) => {
                let source_id = Rc::as_ptr(array) as usize;
                if let Some(cloned) = state.typed_arrays.get(&source_id) {
                    return Ok(Value::TypedArray(cloned.clone()));
                }

                let source = array.borrow();
                let cloned_buffer = Self::structured_clone_array_buffer_ref(&source.buffer, state)?;
                let cloned = Rc::new(RefCell::new(TypedArrayValue {
                    kind: source.kind,
                    buffer: cloned_buffer,
                    byte_offset: source.byte_offset,
                    fixed_length: source.fixed_length,
                }));
                state.typed_arrays.insert(source_id, cloned.clone());
                Ok(Value::TypedArray(cloned))
            }
            Value::Blob(blob) => {
                let source_id = Rc::as_ptr(blob) as usize;
                if let Some(cloned) = state.blobs.get(&source_id) {
                    return Ok(Value::Blob(cloned.clone()));
                }

                let blob = blob.borrow();
                let cloned = match Self::new_blob_value(blob.bytes.clone(), blob.mime_type.clone()) {
                    Value::Blob(cloned) => cloned,
                    _ => unreachable!("Blob clone must produce Blob value"),
                };
                state.blobs.insert(source_id, cloned.clone());
                Ok(Value::Blob(cloned))
            }
            Value::Map(map) => {
                let source_id = Rc::as_ptr(map) as usize;
                if let Some(cloned) = state.maps.get(&source_id) {
                    return Ok(Value::Map(cloned.clone()));
                }

                let map = map.borrow();
                let cloned = Rc::new(RefCell::new(MapValue {
                    entries: Vec::with_capacity(map.entries.len()),
                    properties: map.properties.clone(),
                }));
                state.maps.insert(source_id, cloned.clone());

                let mut cloned_entries = Vec::with_capacity(map.entries.len());
                for (key, value) in map.entries.iter() {
                    let cloned_key = Self::structured_clone_internal(key, state)?;
                    let cloned_value = Self::structured_clone_internal(value, state)?;
                    cloned_entries.push((cloned_key, cloned_value));
                }
                cloned.borrow_mut().entries = cloned_entries;

                Ok(Value::Map(cloned))
            }
            Value::WeakMap(_) => Err(Self::structured_clone_not_cloneable_error()),
            Value::Set(set) => {
                let source_id = Rc::as_ptr(set) as usize;
                if let Some(cloned) = state.sets.get(&source_id) {
                    return Ok(Value::Set(cloned.clone()));
                }

                let set = set.borrow();
                let cloned = Rc::new(RefCell::new(SetValue {
                    values: Vec::with_capacity(set.values.len()),
                    properties: set.properties.clone(),
                }));
                state.sets.insert(source_id, cloned.clone());

                let mut cloned_values = Vec::with_capacity(set.values.len());
                for value in set.values.iter() {
                    cloned_values.push(Self::structured_clone_internal(value, state)?);
                }
                cloned.borrow_mut().values = cloned_values;

                Ok(Value::Set(cloned))
            }
            Value::WeakSet(_) => Err(Self::structured_clone_not_cloneable_error()),
            Value::Array(values) => {
                let source_id = Rc::as_ptr(values) as usize;
                if let Some(cloned) = state.arrays.get(&source_id) {
                    return Ok(Value::Array(cloned.clone()));
                }

                let items = values.borrow();
                let cloned = Rc::new(RefCell::new(ArrayValue::new(Vec::new())));
                state.arrays.insert(source_id, cloned.clone());

                let mut cloned_items = Vec::with_capacity(items.len());
                for item in items.iter() {
                    cloned_items.push(Self::structured_clone_internal(item, state)?);
                }
                let mut cloned_properties = Vec::with_capacity(items.properties.len());
                for (key, value) in items.properties.iter() {
                    let value = Self::structured_clone_internal(value, state)?;
                    cloned_properties.push((key.clone(), value));
                }

                let mut cloned_ref = cloned.borrow_mut();
                cloned_ref.elements = cloned_items;
                cloned_ref.properties = ObjectValue::from(cloned_properties);
                drop(cloned_ref);

                Ok(Value::Array(cloned))
            }
            Value::Object(entries) => {
                let source_id = Rc::as_ptr(entries) as usize;
                if let Some(cloned) = state.objects.get(&source_id) {
                    return Ok(Value::Object(cloned.clone()));
                }

                let cloned = Rc::new(RefCell::new(ObjectValue::default()));
                state.objects.insert(source_id, cloned.clone());

                let entries = entries.borrow();
                let mut cloned_entries = Vec::with_capacity(entries.len());
                for (key, value) in entries.iter() {
                    let value = Self::structured_clone_internal(value, state)?;
                    cloned_entries.push((key.clone(), value));
                }
                *cloned.borrow_mut() = ObjectValue::from(cloned_entries);

                Ok(Value::Object(cloned))
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
            | Value::Function(_) => Err(Self::structured_clone_not_cloneable_error()),
        }
    }

    pub(crate) fn structured_clone_value_with_options(
        value: &Value,
        options: Option<&Value>,
    ) -> Result<Value> {
        let transfer_array_buffers = Self::structured_clone_transfer_array_buffer_ids(options)?;
        let mut state = StructuredCloneState {
            transfer_array_buffers,
            ..StructuredCloneState::default()
        };
        Self::structured_clone_internal(value, &mut state)
    }

    pub(crate) fn structured_clone_value(
        value: &Value,
        _array_stack: &mut Vec<usize>,
        _object_stack: &mut Vec<usize>,
    ) -> Result<Value> {
        Self::structured_clone_value_with_options(value, None)
    }
}
