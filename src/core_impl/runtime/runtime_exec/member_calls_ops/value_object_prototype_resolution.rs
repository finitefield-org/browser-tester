use super::*;

impl Harness {
    fn generator_constructor_prototype_value(&mut self, is_async: bool) -> Option<Value> {
        let constructor = if is_async {
            self.new_async_generator_function_constructor_value()
        } else {
            self.new_generator_function_constructor_value()
        };
        let Value::Object(constructor_entries) = constructor else {
            return None;
        };
        let constructor_entries = constructor_entries.borrow();
        Self::object_get_entry(&constructor_entries, "prototype")
    }

    fn object_property_from_generator_constructor_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if key != "constructor" {
            return None;
        }
        if Self::is_generator_object(entries) {
            return self.generator_constructor_prototype_value(false);
        }
        if Self::is_async_generator_object(entries) {
            return self.generator_constructor_prototype_value(true);
        }
        None
    }

    fn looks_like_iterator_prototype_entries(entries: &ObjectValue, is_async: bool) -> bool {
        let constructor_matches = matches!(
            Self::object_get_entry(entries, "constructor"),
            Some(Value::Object(constructor)) if {
                let constructor = constructor.borrow();
                if is_async {
                    Self::is_async_generator_function_prototype_object(&constructor)
                } else {
                    Self::is_generator_function_prototype_object(&constructor)
                }
            }
        );
        constructor_matches
            && Self::object_get_entry(entries, "next").is_some()
            && Self::object_get_entry(entries, "return").is_some()
            && Self::object_get_entry(entries, "throw").is_some()
    }

    fn object_property_from_generator_to_string_tag_entries(
        &self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !self.is_to_string_tag_property_key(key) {
            return None;
        }
        if Self::is_generator_function_prototype_object(entries) {
            return Some(Value::String("GeneratorFunction".to_string()));
        }
        if Self::is_generator_object(entries)
            || Self::is_generator_prototype_object(entries)
            || Self::looks_like_iterator_prototype_entries(entries, false)
        {
            return Some(Value::String("Generator".to_string()));
        }
        if Self::is_async_generator_function_prototype_object(entries) {
            return Some(Value::String("AsyncGeneratorFunction".to_string()));
        }
        if Self::is_async_generator_object(entries)
            || Self::is_async_generator_prototype_object(entries)
            || Self::looks_like_iterator_prototype_entries(entries, true)
        {
            return Some(Value::String("AsyncGenerator".to_string()));
        }
        None
    }

    pub(crate) fn object_property_from_callable_and_generator_entries(
        &mut self,
        value: &Value,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if Self::callable_kind_from_value(value).is_some() {
            if Self::is_callable_own_surface_key(key)
                && Self::is_builtin_object_property_deleted(entries, key)
            {
                return Self::deleted_callable_surface_fallback_value(key);
            }
            if let Some(surface_value) = self.callable_function_surface_value(value, key) {
                return Some(Self::object_get_entry(entries, key).unwrap_or(surface_value));
            }
        }
        if let Some(value) = self.object_property_from_generator_constructor_entries(entries, key) {
            return Some(value);
        }
        self.object_property_from_generator_to_string_tag_entries(entries, key)
    }

    pub(crate) fn object_property_from_entries_via_prototype_chain(
        &mut self,
        owner: &Value,
        receiver: &Value,
        entries: &ObjectValue,
        key: &str,
    ) -> Result<Value> {
        if let Some(value) =
            self.object_property_from_entries_with_getter(receiver, entries, key)?
        {
            return Ok(value);
        }
        let mut prototype = Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
            .or_else(|| self.value_internal_prototype_value(owner));
        while let Some(current) = prototype {
            if matches!(current, Value::Null | Value::Undefined) {
                break;
            }
            let value = self.object_property_from_value_with_receiver(&current, key, receiver)?;
            if !matches!(value, Value::Undefined) {
                return Ok(value);
            }
            prototype = self.value_internal_prototype_value(&current);
        }
        Ok(Value::Undefined)
    }

    pub(crate) fn value_internal_prototype_value(&mut self, value: &Value) -> Option<Value> {
        if let Some(value) = self.variant_callable_internal_prototype_value(value) {
            return Some(value);
        }
        match value {
            Value::Object(entries) => {
                let entries_ref = entries.borrow();
                if let Some(value) =
                    Self::object_get_entry(&entries_ref, INTERNAL_OBJECT_PROTOTYPE_KEY)
                {
                    return Some(value);
                }
                if Self::is_url_object(&entries_ref) {
                    return Some(self.cached_url_constructor_prototype_value());
                }
                if Self::is_url_search_params_object(&entries_ref) {
                    return Some(self.cached_url_search_params_constructor_prototype_value());
                }
                if Self::string_wrapper_value_from_object(&entries_ref).is_some() {
                    return Some(self.cached_string_constructor_prototype_value());
                }
                if Self::boolean_wrapper_value_from_object(&entries_ref).is_some() {
                    return self.constructor_prototype_from_env("Boolean");
                }
                if Self::number_wrapper_value_from_object(&entries_ref).is_some() {
                    return self.constructor_prototype_from_env("Number");
                }
                if Self::bigint_wrapper_value_from_object(&entries_ref).is_some() {
                    return self.constructor_prototype_from_env("BigInt");
                }
                if Self::symbol_wrapper_id_from_object(&entries_ref).is_some() {
                    return Some(self.cached_symbol_constructor_prototype_value());
                }
                if Self::callable_kind_from_value(value).is_some() {
                    return Some(self.cached_function_constructor_prototype_value());
                }
                Some(self.object_constructor_prototype_value())
            }
            Value::Array(values) => {
                Self::object_get_entry(&values.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
            }
            Value::Map(map) => Some(
                Self::object_get_entry(&map.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| self.cached_map_constructor_prototype_value()),
            ),
            Value::WeakMap(map) => Some(
                Self::object_get_entry(&map.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| self.cached_weak_map_constructor_prototype_value()),
            ),
            Value::Set(set) => Some(
                Self::object_get_entry(&set.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| self.cached_set_constructor_prototype_value()),
            ),
            Value::WeakSet(set) => Some(
                Self::object_get_entry(&set.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| self.cached_weak_set_constructor_prototype_value()),
            ),
            Value::RegExp(regex) => Some(
                Self::object_get_entry(&regex.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| self.cached_regexp_constructor_prototype_value()),
            ),
            Value::Date(_) => Some(self.cached_date_prototype_value()),
            Value::Promise(_) => Some(self.cached_promise_constructor_prototype_value()),
            Value::TypedArray(values) => {
                let (explicit, kind) = {
                    let values_ref = values.borrow();
                    (
                        Self::object_get_entry(
                            &values_ref.properties,
                            INTERNAL_OBJECT_PROTOTYPE_KEY,
                        ),
                        values_ref.kind,
                    )
                };
                Some(explicit.unwrap_or_else(|| {
                    self.cached_typed_array_constructor_prototype_value(
                        TypedArrayConstructorKind::Concrete(kind),
                    )
                }))
            }
            Value::Blob(_) => Some(self.cached_blob_constructor_prototype_value()),
            Value::ArrayBuffer(_) => Some(self.cached_array_buffer_constructor_prototype_value()),
            Value::NodeList(nodes) => Some(
                Self::object_get_entry(&nodes.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| match nodes.borrow().kind {
                        NodeListKind::NodeList => {
                            self.cached_node_list_constructor_prototype_value()
                        }
                        NodeListKind::TextTrackList => {
                            self.cached_text_track_list_constructor_prototype_value()
                        }
                        NodeListKind::RadioNodeList => {
                            self.cached_radio_node_list_constructor_prototype_value()
                        }
                        NodeListKind::HtmlCollection => {
                            self.cached_html_collection_constructor_prototype_value()
                        }
                        NodeListKind::HtmlFormControlsCollection => {
                            self.cached_html_form_controls_collection_constructor_prototype_value()
                        }
                        NodeListKind::HtmlOptionsCollection => {
                            self.cached_html_options_collection_constructor_prototype_value()
                        }
                    }),
            ),
            Value::String(_) => Some(self.cached_string_constructor_prototype_value()),
            Value::Bool(_) => self.constructor_prototype_from_env("Boolean"),
            Value::Number(_) | Value::Float(_) => self.constructor_prototype_from_env("Number"),
            Value::BigInt(_) => self.constructor_prototype_from_env("BigInt"),
            Value::Symbol(_) => Some(self.cached_symbol_constructor_prototype_value()),
            Value::UrlConstructor => {
                let explicit = {
                    let entries = self.browser_apis.url_constructor_properties.borrow();
                    Self::object_get_entry(&entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
                };
                Some(explicit.unwrap_or_else(|| self.cached_function_constructor_prototype_value()))
            }
            Value::Function(function) => {
                if let Some(entries) = self
                    .script_runtime
                    .function_public_properties
                    .get(&function.function_id)
                    && let Some(value) =
                        Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
                {
                    return Some(value);
                }
                if function.is_generator {
                    Some(
                        self.generator_constructor_prototype_value(function.is_async)
                            .unwrap_or_else(|| self.cached_function_constructor_prototype_value()),
                    )
                } else {
                    Some(self.cached_function_constructor_prototype_value())
                }
            }
            _ if self.is_callable_value(value) => {
                Some(self.cached_function_constructor_prototype_value())
            }
            _ => None,
        }
    }

    pub(crate) fn function_public_property_from_entries_with_receiver(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
        receiver: &Value,
    ) -> Result<Option<Value>> {
        let Some(entries) = self
            .script_runtime
            .function_public_properties
            .get(&function.function_id)
            .cloned()
        else {
            return Ok(None);
        };
        self.object_property_from_entries_with_getter(receiver, &entries, key)
    }

    pub(crate) fn inherited_property_from_function_super_constructor(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
        receiver: Option<&Value>,
    ) -> Result<Option<Value>> {
        let Some(super_constructor) = function.class_super_constructor.clone() else {
            return Ok(None);
        };
        if matches!(super_constructor, Value::Null) {
            return Ok(None);
        }
        let inherited = if let Some(receiver) = receiver {
            self.object_property_from_value_with_receiver(&super_constructor, key, receiver)?
        } else {
            self.object_property_from_value(&super_constructor, key)?
        };
        if matches!(inherited, Value::Undefined) {
            Ok(None)
        } else {
            Ok(Some(inherited))
        }
    }

    pub(crate) fn inherited_property_from_value_prototype_chain_with_receiver(
        &mut self,
        owner: &Value,
        receiver: &Value,
        key: &str,
    ) -> Result<Option<Value>> {
        let mut prototype = self.value_internal_prototype_value(owner);
        while let Some(current) = prototype {
            if matches!(current, Value::Null | Value::Undefined) {
                break;
            }
            let value = self.object_property_from_value_with_receiver(&current, key, receiver)?;
            if !matches!(value, Value::Undefined) {
                return Ok(Some(value));
            }
            prototype = self.value_internal_prototype_value(&current);
        }
        Ok(None)
    }

    pub(crate) fn inherited_property_from_value_prototype_chain(
        &mut self,
        receiver: &Value,
        key: &str,
    ) -> Result<Option<Value>> {
        self.inherited_property_from_value_prototype_chain_with_receiver(receiver, receiver, key)
    }

    pub(crate) fn callable_value_property_or_inherited(
        &mut self,
        receiver: &Value,
        key: &str,
        own_value: Value,
    ) -> Result<Value> {
        if !matches!(own_value, Value::Undefined) {
            return Ok(own_value);
        }
        Ok(self
            .inherited_property_from_value_prototype_chain(receiver, key)?
            .unwrap_or(Value::Undefined))
    }

    pub(crate) fn object_property_from_object_value(
        &mut self,
        value: &Value,
        entries: &Rc<RefCell<ObjectValue>>,
        key: &str,
    ) -> Result<Value> {
        let entries = entries.borrow();
        if (Self::is_dom_string_map_object(&entries)
            || Self::is_class_list_object(&entries)
            || Self::is_named_node_map_object(&entries))
            && let Some(value) =
                self.object_property_from_entries_with_getter(value, &entries, key)?
        {
            return Ok(value);
        }
        if let Some(value) = self.object_property_from_attr_or_class_list_entries(&entries, key) {
            return Ok(value);
        }
        if let Some(value) = self.object_property_from_web_api_entries(&entries, key)? {
            return Ok(value);
        }
        if let Some(value) = self
            .object_property_from_match_media_named_node_map_or_string_wrapper_entries(
                value, &entries, key,
            )?
        {
            return Ok(value);
        }
        if let Some(value) =
            self.object_property_from_callable_and_generator_entries(value, &entries, key)
        {
            return Ok(value);
        }
        if let Some(value) =
            self.object_property_from_storage_document_and_url_entries(&entries, key)
        {
            return Ok(value);
        }
        self.object_property_from_entries_via_prototype_chain(value, value, &entries, key)
    }

    pub(crate) fn object_property_from_function_value(
        &mut self,
        value: &Value,
        function: &Rc<FunctionValue>,
        key: &str,
    ) -> Result<Value> {
        if let Some(custom_value) =
            self.function_public_property_from_entries_with_receiver(function, key, value)?
        {
            return Ok(custom_value);
        }
        let own_value = if self
            .script_runtime
            .function_public_properties
            .get(&function.function_id)
            .is_some_and(|entries| Self::is_builtin_object_property_deleted(entries, key))
        {
            Self::deleted_callable_surface_fallback_value(key).unwrap_or(Value::Undefined)
        } else {
            self.function_own_property_value(function, key, true)
        };
        if !matches!(own_value, Value::Undefined) {
            return Ok(own_value);
        }
        if let Some(inherited) =
            self.inherited_property_from_function_super_constructor(function, key, None)?
        {
            return Ok(inherited);
        }
        Ok(self
            .inherited_property_from_value_prototype_chain(value, key)?
            .unwrap_or(Value::Undefined))
    }

    pub(crate) fn object_property_from_object_value_with_receiver(
        &mut self,
        entries: &Rc<RefCell<ObjectValue>>,
        key: &str,
        receiver: &Value,
    ) -> Result<Value> {
        let owner = Value::Object(entries.clone());
        let entries = entries.borrow();
        if (Self::is_dom_string_map_object(&entries)
            || Self::is_class_list_object(&entries)
            || Self::is_named_node_map_object(&entries))
            && let Some(value) =
                self.object_property_from_entries_with_getter(receiver, &entries, key)?
        {
            return Ok(value);
        }
        if let Some(value) = self.object_property_from_attr_or_class_list_entries(&entries, key) {
            return Ok(value);
        }
        if let Some(value) = self.object_property_from_web_api_entries(&entries, key)?
            && self.is_callable_value(&value)
            && !Self::is_builtin_placeholder_value(&value)
        {
            return Ok(value);
        }
        if let Some(value) = self
            .object_property_from_match_media_named_node_map_or_string_wrapper_entries(
                receiver, &entries, key,
            )?
        {
            return Ok(value);
        }
        if let Some(value) =
            self.object_property_from_storage_document_and_url_entries(&entries, key)
            && self.is_callable_value(&value)
            && !Self::is_builtin_placeholder_value(&value)
        {
            return Ok(value);
        }
        self.object_property_from_entries_via_prototype_chain(&owner, receiver, &entries, key)
    }

    pub(crate) fn object_property_from_function_value_with_receiver(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
        receiver: &Value,
    ) -> Result<Value> {
        if let Some(custom_value) =
            self.function_public_property_from_entries_with_receiver(function, key, receiver)?
        {
            return Ok(custom_value);
        }
        let own_value = if self
            .script_runtime
            .function_public_properties
            .get(&function.function_id)
            .is_some_and(|entries| Self::is_builtin_object_property_deleted(entries, key))
        {
            Self::deleted_callable_surface_fallback_value(key).unwrap_or(Value::Undefined)
        } else {
            self.function_own_property_value(function, key, false)
        };
        if !matches!(own_value, Value::Undefined) {
            return Ok(own_value);
        }
        if let Some(inherited) =
            self.inherited_property_from_function_super_constructor(function, key, Some(receiver))?
        {
            return Ok(inherited);
        }
        let owner = Value::Function(function.clone());
        Ok(self
            .inherited_property_from_value_prototype_chain_with_receiver(&owner, receiver, key)?
            .unwrap_or(Value::Undefined))
    }
}
