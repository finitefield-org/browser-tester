use super::*;

impl Harness {
    pub(crate) fn normalize_blob_type(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.as_bytes().iter().all(|b| (0x20..=0x7e).contains(b)) {
            trimmed.to_ascii_lowercase()
        } else {
            String::new()
        }
    }

    pub(crate) fn new_blob_value(bytes: Vec<u8>, mime_type: String) -> Value {
        Value::Blob(Rc::new(RefCell::new(BlobValue { bytes, mime_type })))
    }

    pub(crate) fn new_readable_stream_placeholder_value(&mut self, chunks: Vec<Value>) -> Value {
        let async_iterator_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::AsyncIterator);
        let async_iterator_key = self.property_key_to_storage_key(&async_iterator_symbol);
        Self::new_object_value(vec![
            (
                INTERNAL_READABLE_STREAM_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                async_iterator_key,
                self.new_readable_stream_async_iterator_callable(chunks),
            ),
        ])
    }

    pub(crate) fn new_writable_stream_placeholder_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_WRITABLE_STREAM_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )])
    }

    pub(crate) fn new_uint8_typed_array_from_bytes(bytes: &[u8]) -> Value {
        let buffer = Rc::new(RefCell::new(ArrayBufferValue {
            bytes: bytes.to_vec(),
            max_byte_length: None,
            detached: false,
        }));
        Value::TypedArray(Rc::new(RefCell::new(TypedArrayValue {
            kind: TypedArrayKind::Uint8,
            buffer,
            byte_offset: 0,
            fixed_length: Some(bytes.len()),
            properties: ObjectValue::default(),
        })))
    }

    pub(crate) fn typed_array_raw_bytes(&self, array: &Rc<RefCell<TypedArrayValue>>) -> Vec<u8> {
        let (buffer, byte_offset, byte_length) = {
            let array = array.borrow();
            (
                array.buffer.clone(),
                array.byte_offset,
                array.observed_byte_length(),
            )
        };
        if byte_length == 0 {
            return Vec::new();
        }
        let buffer = buffer.borrow();
        let start = byte_offset.min(buffer.byte_length());
        let end = start.saturating_add(byte_length).min(buffer.byte_length());
        if end <= start {
            Vec::new()
        } else {
            buffer.bytes[start..end].to_vec()
        }
    }

    pub(crate) fn blob_part_bytes(&self, part: &Value) -> Vec<u8> {
        match part {
            Value::Blob(blob) => blob.borrow().bytes.clone(),
            Value::ArrayBuffer(buffer) => buffer.borrow().bytes.clone(),
            Value::TypedArray(array) => self.typed_array_raw_bytes(array),
            Value::String(text) => text.as_bytes().to_vec(),
            other => other.as_string().into_bytes(),
        }
    }

    pub(crate) fn eval_blob_construct(
        &mut self,
        parts: &Option<Box<Expr>>,
        options: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Blob constructor must be called with new".into(),
            ));
        }

        let mut bytes = Vec::new();
        if let Some(parts) = parts {
            let parts_value = self.eval_expr(parts, env, event_param, event)?;
            if !matches!(parts_value, Value::Undefined | Value::Null) {
                let items = self
                    .array_like_values_from_value(&parts_value)
                    .map_err(|_| {
                        Error::ScriptRuntime(
                            "Blob constructor first argument must be an array-like or iterable"
                                .into(),
                        )
                    })?;
                for item in items {
                    bytes.extend(self.blob_part_bytes(&item));
                }
            }
        }

        let mut mime_type = String::new();
        if let Some(options) = options {
            let options = self.eval_expr(options, env, event_param, event)?;
            match options {
                Value::Undefined | Value::Null => {}
                Value::Object(entries) => {
                    let entries = entries.borrow();
                    if let Some(value) = Self::object_get_entry(&entries, "type") {
                        mime_type = Self::normalize_blob_type(&value.as_string());
                    }
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "Blob options must be an object".into(),
                    ));
                }
            }
        }

        Ok(Self::new_blob_value(bytes, mime_type))
    }

    pub(crate) fn new_file_value_from_constructor_args(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() < 2 {
            return Err(Error::ScriptRuntime(
                "File constructor requires at least two arguments".into(),
            ));
        }

        let mut bytes = Vec::new();
        let bits_value = args.first().cloned().unwrap_or(Value::Undefined);
        if !matches!(bits_value, Value::Undefined | Value::Null) {
            let items = self
                .array_like_values_from_value(&bits_value)
                .map_err(|_| {
                    Error::ScriptRuntime(
                        "File constructor first argument must be an array-like or iterable".into(),
                    )
                })?;
            for item in items {
                bytes.extend(self.blob_part_bytes(&item));
            }
        }

        let mut mime_type = String::new();
        let mut last_modified = self.scheduler.now_ms;
        if let Some(options) = args.get(2) {
            match options {
                Value::Undefined | Value::Null => {}
                Value::Object(entries) => {
                    let entries = entries.borrow();
                    if let Some(value) = Self::object_get_entry(&entries, "type") {
                        mime_type = Self::normalize_blob_type(&value.as_string());
                    }
                    if let Some(value) = Self::object_get_entry(&entries, "lastModified") {
                        last_modified = Self::value_to_i64(&value);
                    }
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "File options must be an object".into(),
                    ));
                }
            }
        }

        let file = MockFile {
            name: args.get(1).map(Value::as_string).unwrap_or_default(),
            size: bytes.len() as i64,
            mime_type,
            last_modified,
            webkit_relative_path: String::new(),
            bytes,
        };
        Ok(Self::mock_file_to_value(&file))
    }

    pub(crate) fn eval_blob_member_call(
        &mut self,
        blob: &Rc<RefCell<BlobValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "text" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.text does not take arguments".into(),
                    ));
                }
                let text = String::from_utf8_lossy(&blob.borrow().bytes).to_string();
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, Value::String(text))?;
                Ok(Some(Value::Promise(promise)))
            }
            "arrayBuffer" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.arrayBuffer does not take arguments".into(),
                    ));
                }
                let bytes = blob.borrow().bytes.clone();
                let promise = self.new_pending_promise();
                self.promise_resolve(
                    &promise,
                    Value::ArrayBuffer(Rc::new(RefCell::new(ArrayBufferValue {
                        bytes,
                        max_byte_length: None,
                        detached: false,
                    }))),
                )?;
                Ok(Some(Value::Promise(promise)))
            }
            "bytes" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.bytes does not take arguments".into(),
                    ));
                }
                let bytes = blob.borrow().bytes.clone();
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, Self::new_uint8_typed_array_from_bytes(&bytes))?;
                Ok(Some(Value::Promise(promise)))
            }
            "stream" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.stream does not take arguments".into(),
                    ));
                }
                let bytes = blob.borrow().bytes.clone();
                let chunks = if bytes.is_empty() {
                    Vec::new()
                } else {
                    vec![Self::new_uint8_typed_array_from_bytes(&bytes)]
                };
                Ok(Some(self.new_readable_stream_placeholder_value(chunks)))
            }
            "slice" => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Blob.slice supports up to two arguments".into(),
                    ));
                }
                let source = blob.borrow();
                let len = source.bytes.len();
                let start = args
                    .first()
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(0);
                let end = args
                    .get(1)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(len);
                let end = end.max(start);
                Ok(Some(Self::new_blob_value(
                    source.bytes[start..end].to_vec(),
                    String::new(),
                )))
            }
            _ => Ok(None),
        }
    }
}
