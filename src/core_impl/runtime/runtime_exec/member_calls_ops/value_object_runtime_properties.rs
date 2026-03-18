use super::*;

impl Harness {
    fn object_property_from_string_value(&self, text: &str, key: &str) -> Value {
        if key == "length" {
            Value::Number(Self::string_char_len(text) as i64)
        } else if key == "constructor" {
            Value::StringConstructor
        } else if self.is_iterator_property_key(key) {
            Self::new_receiver_builtin_callable("string", "iterator")
        } else if matches!(key, "toString" | "valueOf") || Self::is_string_method_name(key) {
            Self::new_receiver_builtin_callable("string", key)
        } else if let Ok(index) = key.parse::<usize>() {
            Self::string_char_at(text, index)
                .map(|ch| Value::String(ch.to_string()))
                .unwrap_or(Value::Undefined)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_bool_value(&self, key: &str) -> Value {
        if key == "constructor" {
            self.script_runtime
                .env
                .get("Boolean")
                .cloned()
                .unwrap_or_else(Self::new_boolean_constructor_callable)
        } else if matches!(key, "toString" | "valueOf") {
            Self::new_receiver_builtin_callable("boolean", key)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_number_value(&self, key: &str) -> Value {
        if key == "constructor" {
            self.script_runtime
                .env
                .get("Number")
                .cloned()
                .unwrap_or_else(Self::new_number_constructor_callable)
        } else if matches!(
            key,
            "toExponential" | "toFixed" | "toLocaleString" | "toPrecision" | "toString" | "valueOf"
        ) {
            Self::new_receiver_builtin_callable("number", key)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_bigint_value(&self, key: &str) -> Value {
        if key == "constructor" {
            self.script_runtime
                .env
                .get("BigInt")
                .cloned()
                .unwrap_or_else(Self::new_bigint_constructor_callable)
        } else if matches!(key, "toLocaleString" | "toString" | "valueOf") {
            Self::new_receiver_builtin_callable("bigint", key)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_array_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        values: &Rc<RefCell<ArrayValue>>,
        key: &str,
    ) -> Result<Value> {
        let values = values.borrow();
        if Self::is_dom_rect_list_value(&values) && self.is_to_string_tag_property_key(key) {
            return Ok(Value::String("DOMRectList".to_string()));
        }
        if key == "length" {
            return Ok(Value::Number(values.len() as i64));
        }
        let has_placeholder_builtin =
            Self::placeholder_backed_array_builtin_surface_exists(&values, key);
        if has_placeholder_builtin {
            if let Some(value) = Self::placeholder_backed_array_builtin_property_value(&values, key)
            {
                return Ok(value);
            }
            return Ok(Value::Undefined);
        }
        let has_explicit_prototype =
            Self::object_get_entry(&values.properties, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
        if let Ok(index) = key.parse::<usize>() {
            if index < values.len() && !Self::array_index_is_hole(&values, index) {
                return Ok(values[index].clone());
            }
            drop(values);
            if has_explicit_prototype {
                return Ok(self
                    .inherited_property_from_value_prototype_chain_with_receiver(
                        owner, receiver, key,
                    )?
                    .unwrap_or(Value::Undefined));
            }
            return Ok(Value::Undefined);
        }
        if let Some(value) = Self::object_get_entry(&values.properties, key) {
            return Ok(value);
        }
        if !has_explicit_prototype {
            if self.is_iterator_property_key(key) {
                return Ok(Self::new_receiver_builtin_callable("array", "values"));
            }
            if Self::is_array_method_name(key) {
                return Ok(Self::new_receiver_builtin_callable("array", key));
            }
            return Ok(Value::Undefined);
        }
        drop(values);
        Ok(self
            .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
            .unwrap_or(Value::Undefined))
    }

    fn object_property_from_node_list_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        nodes: &Rc<RefCell<NodeListValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_override = {
            let nodes_ref = nodes.borrow();
            self.object_property_from_entries_with_getter(receiver, &nodes_ref.properties, key)?
        };
        if let Some(value) = own_override {
            return Ok(value);
        }
        let has_explicit_prototype = {
            let nodes_ref = nodes.borrow();
            Self::object_get_entry(&nodes_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some()
        };
        if key == "length" {
            return Ok(Value::Number(self.node_list_len(nodes) as i64));
        }
        if let Ok(index) = key.parse::<usize>() {
            if let Some(node) = self.node_list_get(nodes, index) {
                return Ok(self.node_list_item_value(nodes, node));
            }
            if has_explicit_prototype {
                return Ok(self
                    .inherited_property_from_value_prototype_chain_with_receiver(
                        owner, receiver, key,
                    )?
                    .unwrap_or(Value::Undefined));
            }
            return Ok(Value::Undefined);
        }
        if let Some(value) = self.html_collection_named_property_value(nodes, key) {
            return Ok(value);
        }
        Ok(self
            .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
            .unwrap_or(Value::Undefined))
    }

    fn object_property_from_typed_array_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        values: &Rc<RefCell<TypedArrayValue>>,
        key: &str,
    ) -> Result<Value> {
        let value_ref = values.borrow();
        let has_explicit_prototype =
            Self::object_get_entry(&value_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
        let kind = value_ref.kind;
        drop(value_ref);
        if key == "length" {
            return Ok(Value::Number(values.borrow().observed_length() as i64));
        }
        if let Ok(index) = key.parse::<usize>() {
            let snapshot = self.typed_array_snapshot(values)?;
            if let Some(value) = snapshot.get(index) {
                return Ok(value.clone());
            }
            if has_explicit_prototype {
                return Ok(self
                    .inherited_property_from_value_prototype_chain_with_receiver(
                        owner, receiver, key,
                    )?
                    .unwrap_or(Value::Undefined));
            }
            return Ok(Value::Undefined);
        }
        if let Some(value) = Self::object_get_entry(&values.borrow().properties, key) {
            return Ok(value);
        }
        if !has_explicit_prototype {
            match key {
                "constructor" => {
                    return Ok(Value::TypedArrayConstructor(
                        TypedArrayConstructorKind::Concrete(kind),
                    ));
                }
                "byteLength" => {
                    return Ok(Value::Number(values.borrow().observed_byte_length() as i64));
                }
                "byteOffset" => {
                    let value_ref = values.borrow();
                    let byte_offset = if value_ref.observed_length() == 0
                        && value_ref.byte_offset >= value_ref.buffer.borrow().byte_length()
                    {
                        0
                    } else {
                        value_ref.byte_offset
                    };
                    return Ok(Value::Number(byte_offset as i64));
                }
                "buffer" => {
                    return Ok(Value::ArrayBuffer(values.borrow().buffer.clone()));
                }
                "BYTES_PER_ELEMENT" => {
                    return Ok(Value::Number(kind.bytes_per_element() as i64));
                }
                _ => {}
            }
            if self.is_iterator_property_key(key) {
                return Ok(Self::new_receiver_builtin_callable("typed_array", "values"));
            }
            if Self::is_typed_array_method_name(key) {
                return Ok(Self::new_receiver_builtin_callable("typed_array", key));
            }
            return Ok(Value::Undefined);
        }
        Ok(self
            .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
            .unwrap_or(Value::Undefined))
    }

    fn object_property_from_promise_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        promise: &Rc<RefCell<PromiseValue>>,
        key: &str,
    ) -> Result<Value> {
        if !self.strict_equal(owner, receiver) {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }
        if key == "constructor" {
            return Ok(Value::PromiseConstructor);
        }
        if matches!(key, "then" | "catch" | "finally") {
            return Ok(Self::new_receiver_builtin_callable("promise", key));
        }
        let promise = promise.borrow();
        if key == "status" {
            let status = match &promise.state {
                PromiseState::Pending => "pending",
                PromiseState::Fulfilled(_) => "fulfilled",
                PromiseState::Rejected(_) => "rejected",
            };
            Ok(Value::String(status.to_string()))
        } else {
            Ok(Value::Undefined)
        }
    }

    fn object_property_from_map_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        map: &Rc<RefCell<MapValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_value = {
            let map_ref = map.borrow();
            self.object_property_from_entries_with_getter(receiver, &map_ref.properties, key)?
        };
        if let Some(value) = own_value {
            return Ok(value);
        }
        let use_prototype_chain = {
            let map_ref = map.borrow();
            !self.strict_equal(owner, receiver)
                || Self::object_get_entry(&map_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .is_some()
        };
        if use_prototype_chain {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }

        let map = map.borrow();
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        Ok(if key == "size" {
            Value::Number(map.entries.len() as i64)
        } else if key_is_to_string_tag {
            Value::String("Map".to_string())
        } else if key == "constructor" {
            Value::MapConstructor
        } else if self.is_iterator_property_key(key) {
            Self::new_receiver_builtin_callable("map", "entries")
        } else if Self::is_map_method_name(key) {
            Self::new_receiver_builtin_callable("map", key)
        } else {
            Value::Undefined
        })
    }

    fn object_property_from_weak_map_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        weak_map: &Rc<RefCell<WeakMapValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_value = {
            let weak_map_ref = weak_map.borrow();
            self.object_property_from_entries_with_getter(receiver, &weak_map_ref.properties, key)?
        };
        if let Some(value) = own_value {
            return Ok(value);
        }
        let use_prototype_chain = {
            let weak_map_ref = weak_map.borrow();
            !self.strict_equal(owner, receiver)
                || Self::object_get_entry(&weak_map_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .is_some()
        };
        if use_prototype_chain {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }

        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        Ok(if key_is_to_string_tag {
            Value::String("WeakMap".to_string())
        } else if key == "constructor" {
            Value::WeakMapConstructor
        } else if Self::is_weak_map_method_name(key) {
            Self::new_receiver_builtin_callable("weak_map", key)
        } else {
            Value::Undefined
        })
    }

    fn object_property_from_weak_set_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        weak_set: &Rc<RefCell<WeakSetValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_value = {
            let weak_set_ref = weak_set.borrow();
            self.object_property_from_entries_with_getter(receiver, &weak_set_ref.properties, key)?
        };
        if let Some(value) = own_value {
            return Ok(value);
        }
        let use_prototype_chain = {
            let weak_set_ref = weak_set.borrow();
            !self.strict_equal(owner, receiver)
                || Self::object_get_entry(&weak_set_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .is_some()
        };
        if use_prototype_chain {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }

        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        Ok(if key_is_to_string_tag {
            Value::String("WeakSet".to_string())
        } else if key == "constructor" {
            Value::WeakSetConstructor
        } else if Self::is_weak_set_method_name(key) {
            Self::new_receiver_builtin_callable("weak_set", key)
        } else {
            Value::Undefined
        })
    }

    fn object_property_from_set_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        set: &Rc<RefCell<SetValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_value = {
            let set_ref = set.borrow();
            self.object_property_from_entries_with_getter(receiver, &set_ref.properties, key)?
        };
        if let Some(value) = own_value {
            return Ok(value);
        }
        let use_prototype_chain = {
            let set_ref = set.borrow();
            !self.strict_equal(owner, receiver)
                || Self::object_get_entry(&set_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .is_some()
        };
        if use_prototype_chain {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }

        let set = set.borrow();
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        Ok(if key == "size" {
            Value::Number(set.values.len() as i64)
        } else if key_is_to_string_tag {
            Value::String("Set".to_string())
        } else if key == "constructor" {
            Value::SetConstructor
        } else if self.is_iterator_property_key(key) {
            Self::new_receiver_builtin_callable("set", "values")
        } else if Self::is_set_method_name(key) {
            Self::new_receiver_builtin_callable("set", key)
        } else {
            Value::Undefined
        })
    }

    fn object_property_from_form_data_value(
        &self,
        _entries: &Rc<RefCell<Vec<(String, String)>>>,
        key: &str,
    ) -> Value {
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        if key_is_to_string_tag {
            Value::String("FormData".to_string())
        } else if self.is_iterator_property_key(key) {
            Self::new_receiver_builtin_callable("form_data", "entries")
        } else if matches!(
            key,
            "append" | "set" | "delete" | "entries" | "keys" | "values" | "get" | "getAll" | "has"
        ) {
            Self::new_receiver_builtin_callable("form_data", key)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_blob_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        blob: &Rc<RefCell<BlobValue>>,
        key: &str,
    ) -> Result<Value> {
        if !self.strict_equal(owner, receiver) {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }
        let blob = blob.borrow();
        Ok(match key {
            "size" => Value::Number(blob.bytes.len() as i64),
            "type" => Value::String(blob.mime_type.clone()),
            "constructor" => Value::BlobConstructor,
            "arrayBuffer" | "bytes" | "slice" | "stream" | "text" => {
                Self::new_receiver_builtin_callable("blob", key)
            }
            _ => Value::Undefined,
        })
    }

    fn object_property_from_array_buffer_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        buffer: &Rc<RefCell<ArrayBufferValue>>,
        key: &str,
    ) -> Result<Value> {
        if !self.strict_equal(owner, receiver) {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }
        let buffer = buffer.borrow();
        Ok(match key {
            "byteLength" => Value::Number(buffer.byte_length() as i64),
            "detached" => Value::Bool(buffer.detached),
            "maxByteLength" => Value::Number(buffer.max_byte_length() as i64),
            "resizable" => Value::Bool(buffer.resizable()),
            "constructor" => Value::ArrayBufferConstructor,
            "resize" | "slice" | "transfer" | "transferToFixedLength" => {
                Self::new_receiver_builtin_callable("array_buffer", key)
            }
            _ => Value::Undefined,
        })
    }

    fn object_property_from_symbol_value(symbol: &Rc<SymbolValue>, key: &str) -> Value {
        match key {
            "description" => symbol
                .description
                .as_ref()
                .map(|value| Value::String(value.clone()))
                .unwrap_or(Value::Undefined),
            "constructor" => Value::SymbolConstructor,
            "toString" | "valueOf" => Self::new_receiver_builtin_callable("symbol", key),
            _ => Value::Undefined,
        }
    }

    fn object_property_from_regexp_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        regex: &Rc<RefCell<RegexValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_value = {
            let regex_ref = regex.borrow();
            self.object_property_from_entries_with_getter(receiver, &regex_ref.properties, key)?
        };
        if let Some(value) = own_value {
            return Ok(value);
        }
        let use_prototype_chain = {
            let regex_ref = regex.borrow();
            !self.strict_equal(owner, receiver)
                || Self::object_get_entry(&regex_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .is_some()
        };
        if use_prototype_chain {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }

        let regex = regex.borrow();
        if key == "lastIndex" {
            Ok(Value::Number(regex.last_index as i64))
        } else {
            Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined))
        }
    }

    pub(crate) fn object_property_from_value(&mut self, value: &Value, key: &str) -> Result<Value> {
        match value {
            Value::Node(node) => self.object_property_from_node_value(node, key),
            Value::String(text) => Ok(self.object_property_from_string_value(text, key)),
            Value::Bool(_) => Ok(self.object_property_from_bool_value(key)),
            Value::Number(_) | Value::Float(_) => Ok(self.object_property_from_number_value(key)),
            Value::BigInt(_) => Ok(self.object_property_from_bigint_value(key)),
            Value::Array(values) => {
                self.object_property_from_array_value(value, value, values, key)
            }
            Value::NodeList(nodes) => {
                self.object_property_from_node_list_value(value, value, nodes, key)
            }
            Value::TypedArray(values) => {
                self.object_property_from_typed_array_value(value, value, values, key)
            }
            Value::Object(entries) => self.object_property_from_object_value(value, entries, key),
            Value::Promise(promise) => {
                self.object_property_from_promise_value(value, value, promise, key)
            }
            Value::Map(map) => self.object_property_from_map_value(value, value, map, key),
            Value::WeakMap(weak_map) => {
                self.object_property_from_weak_map_value(value, value, weak_map, key)
            }
            Value::WeakSet(weak_set) => {
                self.object_property_from_weak_set_value(value, value, weak_set, key)
            }
            Value::Set(set) => self.object_property_from_set_value(value, value, set, key),
            Value::FormData(entries) => Ok(self.object_property_from_form_data_value(entries, key)),
            Value::Blob(blob) => self.object_property_from_blob_value(value, value, blob, key),
            Value::ArrayBuffer(buffer) => {
                self.object_property_from_array_buffer_value(value, value, buffer, key)
            }
            Value::Symbol(symbol) => Ok(Self::object_property_from_symbol_value(symbol, key)),
            Value::RegExp(regex) => {
                self.object_property_from_regexp_value(value, value, regex, key)
            }
            Value::Date(_) => Ok(self
                .inherited_property_from_value_prototype_chain(value, key)?
                .unwrap_or(Value::Undefined)),
            Value::Function(function) => {
                self.object_property_from_function_value(value, function, key)
            }
            _ => self
                .object_property_from_constructor_value(value, key)
                .unwrap_or_else(|| Err(Error::ScriptRuntime("value is not an object".into()))),
        }
    }

    pub(crate) fn object_property_from_value_with_receiver(
        &mut self,
        value: &Value,
        key: &str,
        receiver: &Value,
    ) -> Result<Value> {
        match value {
            Value::Object(entries) => {
                self.object_property_from_object_value_with_receiver(entries, key, receiver)
            }
            Value::Function(function) => {
                self.object_property_from_function_value_with_receiver(function, key, receiver)
            }
            Value::Array(values) => {
                self.object_property_from_array_value(value, receiver, values, key)
            }
            Value::NodeList(nodes) => {
                self.object_property_from_node_list_value(value, receiver, nodes, key)
            }
            Value::TypedArray(values) => {
                self.object_property_from_typed_array_value(value, receiver, values, key)
            }
            Value::Promise(promise) => {
                self.object_property_from_promise_value(value, receiver, promise, key)
            }
            Value::Map(map) => self.object_property_from_map_value(value, receiver, map, key),
            Value::WeakMap(weak_map) => {
                self.object_property_from_weak_map_value(value, receiver, weak_map, key)
            }
            Value::WeakSet(weak_set) => {
                self.object_property_from_weak_set_value(value, receiver, weak_set, key)
            }
            Value::Set(set) => self.object_property_from_set_value(value, receiver, set, key),
            Value::Blob(blob) => self.object_property_from_blob_value(value, receiver, blob, key),
            Value::ArrayBuffer(buffer) => {
                self.object_property_from_array_buffer_value(value, receiver, buffer, key)
            }
            Value::RegExp(regex) => {
                self.object_property_from_regexp_value(value, receiver, regex, key)
            }
            _ => self.object_property_from_value(value, key),
        }
    }

    pub(crate) fn object_property_from_named_value(
        &mut self,
        variable_name: &str,
        value: &Value,
        key: &str,
    ) -> Result<Value> {
        self.object_property_from_value(value, key)
            .map_err(|err| match err {
                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                    Error::ScriptRuntime(format!(
                        "variable '{}' is not an object (key '{}')",
                        variable_name, key
                    ))
                }
                other => other,
            })
    }
}
