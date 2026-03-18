use super::*;

impl Harness {
    pub(crate) fn construct_map_from_values(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() > 1 {
            return Err(Error::ScriptRuntime(
                "Map supports zero or one argument".into(),
            ));
        }

        let map = Rc::new(RefCell::new(MapValue {
            entries: Vec::new(),
            properties: ObjectValue::default(),
        }));
        let Some(iterable) = args.first() else {
            return Ok(Value::Map(map));
        };
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Map(map));
        }
        match iterable {
            Value::Map(source) => {
                let source = source.borrow();
                map.borrow_mut().entries = source.entries.clone();
            }
            other => {
                let entries = self.array_like_values_from_value(other)?;
                for entry in entries {
                    let pair = self.array_like_values_from_value(&entry).map_err(|_| {
                        Error::ScriptRuntime(
                            "Map constructor iterable values must be [key, value] pairs".into(),
                        )
                    })?;
                    if pair.len() < 2 {
                        return Err(Error::ScriptRuntime(
                            "Map constructor iterable values must be [key, value] pairs".into(),
                        ));
                    }
                    self.map_set_entry(&mut map.borrow_mut(), pair[0].clone(), pair[1].clone());
                }
            }
        }
        Ok(Value::Map(map))
    }

    pub(crate) fn construct_weak_map_from_values(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() > 1 {
            return Err(Error::ScriptRuntime(
                "WeakMap supports zero or one argument".into(),
            ));
        }

        let weak_map = Rc::new(RefCell::new(WeakMapValue {
            entries: Vec::new(),
            properties: ObjectValue::default(),
        }));
        let Some(iterable) = args.first() else {
            return Ok(Value::WeakMap(weak_map));
        };
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::WeakMap(weak_map));
        }
        match iterable {
            Value::WeakMap(source) => {
                let source = source.borrow();
                weak_map.borrow_mut().entries = source.entries.clone();
            }
            other => {
                let entries = self.array_like_values_from_value(other)?;
                for entry in entries {
                    let pair = self.array_like_values_from_value(&entry).map_err(|_| {
                        Error::ScriptRuntime(
                            "WeakMap constructor iterable values must be [key, value] pairs".into(),
                        )
                    })?;
                    if pair.len() < 2 {
                        return Err(Error::ScriptRuntime(
                            "WeakMap constructor iterable values must be [key, value] pairs".into(),
                        ));
                    }
                    Self::ensure_weak_map_key(&pair[0])?;
                    self.weak_map_set_entry(
                        &mut weak_map.borrow_mut(),
                        pair[0].clone(),
                        pair[1].clone(),
                    );
                }
            }
        }
        Ok(Value::WeakMap(weak_map))
    }

    pub(crate) fn construct_set_from_values(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() > 1 {
            return Err(Error::ScriptRuntime(
                "Set supports zero or one argument".into(),
            ));
        }

        let set = Rc::new(RefCell::new(SetValue {
            values: Vec::new(),
            properties: ObjectValue::default(),
        }));
        let Some(iterable) = args.first() else {
            return Ok(Value::Set(set));
        };
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Set(set));
        }
        match iterable {
            Value::Set(source) => {
                let source = source.borrow();
                set.borrow_mut().values = source.values.clone();
            }
            other => {
                for value in self.array_like_values_from_value(other)? {
                    self.set_add_value(&mut set.borrow_mut(), value);
                }
            }
        }
        Ok(Value::Set(set))
    }

    pub(crate) fn construct_weak_set_from_values(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() > 1 {
            return Err(Error::ScriptRuntime(
                "WeakSet supports zero or one argument".into(),
            ));
        }

        let weak_set = Rc::new(RefCell::new(WeakSetValue {
            values: Vec::new(),
            properties: ObjectValue::default(),
        }));
        let Some(iterable) = args.first() else {
            return Ok(Value::WeakSet(weak_set));
        };
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::WeakSet(weak_set));
        }
        match iterable {
            Value::WeakSet(source) => {
                let source = source.borrow();
                weak_set.borrow_mut().values = source.values.clone();
            }
            other => {
                for value in self.array_like_values_from_value(other)? {
                    Self::ensure_weak_map_key(&value)?;
                    self.weak_set_add_value(&mut weak_set.borrow_mut(), value);
                }
            }
        }
        Ok(Value::WeakSet(weak_set))
    }

    pub(crate) fn construct_blob_from_values(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() > 2 {
            return Err(Error::ScriptRuntime(
                "Blob supports zero, one, or two arguments".into(),
            ));
        }

        let mut bytes = Vec::new();
        if let Some(parts_value) = args.first() {
            if !matches!(parts_value, Value::Undefined | Value::Null) {
                let items = self
                    .array_like_values_from_value(parts_value)
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
        if let Some(options) = args.get(1) {
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

    pub(crate) fn construct_url_from_values(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() > 2 {
            return Err(Error::ScriptRuntime(
                "URL supports one or two constructor arguments".into(),
            ));
        }
        let Some(input) = args.first() else {
            return Err(Error::ScriptRuntime(
                "URL constructor requires a URL argument".into(),
            ));
        };
        let input = input.as_string();
        let base = args.get(1).map(Value::as_string);
        let href = Self::resolve_url_string(&input, base.as_deref())
            .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
        self.new_url_value_from_href(&href)
    }

    pub(crate) fn construct_url_search_params_from_values(&self, args: &[Value]) -> Result<Value> {
        if args.len() > 1 {
            return Err(Error::ScriptRuntime(
                "URLSearchParams supports zero or one argument".into(),
            ));
        }
        let init = args.first().cloned().unwrap_or(Value::Undefined);
        let pairs = self.url_search_params_pairs_from_init_value(&init)?;
        Ok(self.new_url_search_params_value(pairs, None))
    }

    pub(crate) fn construct_regexp_from_values(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() > 2 {
            return Err(Error::ScriptRuntime(
                "RegExp supports up to two arguments".into(),
            ));
        }
        let pattern = args
            .first()
            .cloned()
            .unwrap_or_else(|| Value::String(String::new()));
        let flags = args.get(1);
        self.new_regex_from_values(&pattern, flags)
    }

    pub(crate) fn construct_array_buffer_from_values(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() > 2 {
            return Err(Error::ScriptRuntime(
                "ArrayBuffer supports up to two arguments".into(),
            ));
        }
        let byte_length = if let Some(byte_length) = args.first() {
            Self::to_non_negative_usize(byte_length, "ArrayBuffer byteLength")?
        } else {
            0
        };
        let max_byte_length = if let Some(options) = args.get(1) {
            match options {
                Value::Undefined | Value::Null => None,
                Value::Object(entries) => {
                    let entries = entries.borrow();
                    if let Some(value) = Self::object_get_entry(&entries, "maxByteLength") {
                        Some(Self::to_non_negative_usize(
                            &value,
                            "ArrayBuffer maxByteLength",
                        )?)
                    } else {
                        None
                    }
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "ArrayBuffer options must be an object".into(),
                    ));
                }
            }
        } else {
            None
        };
        if max_byte_length.is_some_and(|max| byte_length > max) {
            return Err(Error::ScriptRuntime(
                "ArrayBuffer byteLength exceeds maxByteLength".into(),
            ));
        }
        Ok(Self::new_array_buffer_value(byte_length, max_byte_length))
    }

    pub(crate) fn construct_promise_from_values(
        &mut self,
        args: &[Value],
        event: &EventState,
    ) -> Result<Value> {
        if args.len() != 1 {
            return Err(Error::ScriptRuntime(
                "Promise constructor requires exactly one executor".into(),
            ));
        }
        let executor = args[0].clone();
        if !self.is_callable_value(&executor) {
            return Err(Error::ScriptRuntime(
                "Promise constructor executor must be a function".into(),
            ));
        }

        let promise = self.new_pending_promise();
        let (resolve, reject) = self.new_promise_capability_functions(promise.clone());
        if let Err(err) = self.execute_callable_value(&executor, &[resolve, reject], event) {
            self.promise_reject(&promise, Self::promise_error_reason(err));
        }
        Ok(Value::Promise(promise))
    }

    pub(crate) fn attach_constructor_prototype_to_instance(
        &mut self,
        constructor: &Value,
        instance: &mut Value,
    ) -> Result<()> {
        let Value::Object(instance_entries) = instance else {
            return Ok(());
        };
        let prototype = self.object_property_from_value(constructor, "prototype")?;
        let Value::Object(prototype_entries) = prototype else {
            return Ok(());
        };
        let mut instance_entries = instance_entries.borrow_mut();
        if Self::object_get_entry(&instance_entries, INTERNAL_OBJECT_PROTOTYPE_KEY).is_none() {
            Self::object_set_entry(
                &mut instance_entries,
                INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                Value::Object(prototype_entries),
            );
        }
        Ok(())
    }
}
