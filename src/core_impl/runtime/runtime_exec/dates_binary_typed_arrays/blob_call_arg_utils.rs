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
        let async_iterator_symbol = self.eval_symbol_static_property(SymbolStaticProperty::AsyncIterator);
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
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_typed_array_member_call(
        &mut self,
        array: &Rc<RefCell<TypedArrayValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "join" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.join supports at most one argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let separator = if let Some(first) = args.first() {
                    if matches!(first, Value::Undefined) {
                        ",".to_string()
                    } else {
                        first.as_string()
                    }
                } else {
                    ",".to_string()
                };
                let joined = self
                    .typed_array_snapshot(array)?
                    .into_iter()
                    .map(|value| value.as_string())
                    .collect::<Vec<_>>()
                    .join(&separator);
                Ok(Some(Value::String(joined)))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn to_non_negative_usize(value: &Value, label: &str) -> Result<usize> {
        let n = Self::value_to_i64(value);
        if n < 0 {
            return Err(Error::ScriptRuntime(format!(
                "{label} must be a non-negative integer"
            )));
        }
        usize::try_from(n).map_err(|_| Error::ScriptRuntime(format!("{label} is too large")))
    }

    pub(crate) fn eval_call_args_with_spread(
        &mut self,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Vec<Value>> {
        let mut evaluated = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                Expr::Spread(inner) => {
                    let spread_value = self.eval_expr(inner, env, event_param, event)?;
                    evaluated.extend(self.spread_iterable_values_from_value(&spread_value)?);
                }
                _ => evaluated.push(self.eval_expr(arg, env, event_param, event)?),
            }
        }
        Ok(evaluated)
    }

    pub(crate) fn spread_iterable_values_from_value(&self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Array(values) => Ok(values.borrow().clone()),
            Value::TypedArray(values) => self.typed_array_snapshot(values),
            Value::Map(map) => {
                let map = map.borrow();
                Ok(map
                    .entries
                    .iter()
                    .map(|(key, value)| Self::new_array_value(vec![key.clone(), value.clone()]))
                    .collect::<Vec<_>>())
            }
            Value::Set(set) => Ok(set.borrow().values.clone()),
            Value::String(text) => Ok(text
                .chars()
                .map(|ch| Value::String(ch.to_string()))
                .collect::<Vec<_>>()),
            Value::NodeList(nodes) => Ok(nodes.iter().copied().map(Value::Node).collect()),
            Value::Object(entries) => {
                let is_iterator = {
                    let entries_ref = entries.borrow();
                    Self::is_iterator_object(&entries_ref)
                };
                if is_iterator {
                    return self.iterator_collect_remaining_values(entries);
                }
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    return Ok(Self::url_search_params_pairs_from_object_entries(&entries)
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>());
                }
                Err(Error::ScriptRuntime("spread source is not iterable".into()))
            }
            _ => Err(Error::ScriptRuntime("spread source is not iterable".into())),
        }
    }

    pub(crate) fn array_like_values_from_value(&self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Array(values) => Ok(values.borrow().clone()),
            Value::TypedArray(values) => self.typed_array_snapshot(values),
            Value::Map(map) => {
                let map = map.borrow();
                Ok(map
                    .entries
                    .iter()
                    .map(|(key, value)| Self::new_array_value(vec![key.clone(), value.clone()]))
                    .collect::<Vec<_>>())
            }
            Value::Set(set) => Ok(set.borrow().values.clone()),
            Value::String(text) => Ok(text
                .chars()
                .map(|ch| Value::String(ch.to_string()))
                .collect::<Vec<_>>()),
            Value::NodeList(nodes) => Ok(nodes.iter().copied().map(Value::Node).collect()),
            Value::Object(entries) => {
                let is_iterator = {
                    let entries_ref = entries.borrow();
                    Self::is_iterator_object(&entries_ref)
                };
                if is_iterator {
                    return self.iterator_collect_remaining_values(entries);
                }
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    return Ok(Self::url_search_params_pairs_from_object_entries(&entries)
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>());
                }
                let length_value =
                    Self::object_get_entry(&entries, "length").unwrap_or(Value::Number(0));
                let length = Self::to_non_negative_usize(&length_value, "array-like length")?;
                let mut out = Vec::with_capacity(length);
                for index in 0..length {
                    let key = index.to_string();
                    out.push(Self::object_get_entry(&entries, &key).unwrap_or(Value::Undefined));
                }
                Ok(out)
            }
            _ => Err(Error::ScriptRuntime(
                "expected an array-like or iterable source".into(),
            )),
        }
    }
}
