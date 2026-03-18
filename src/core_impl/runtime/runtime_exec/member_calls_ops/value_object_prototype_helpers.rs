use super::*;

impl Harness {
    pub(crate) fn set_internal_prototype(entries: &Rc<RefCell<ObjectValue>>, prototype: Value) {
        Self::object_set_entry(
            &mut entries.borrow_mut(),
            INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
            prototype,
        );
    }

    pub(crate) fn mark_property_non_enumerable(
        entries: &Rc<RefCell<ObjectValue>>,
        property_key: &str,
    ) {
        Self::object_set_entry(
            &mut entries.borrow_mut(),
            Self::object_non_enumerable_storage_key(property_key),
            Value::Bool(true),
        );
    }

    pub(crate) fn mark_existing_public_properties_non_enumerable(
        entries: &Rc<RefCell<ObjectValue>>,
    ) {
        let keys = entries
            .borrow()
            .iter()
            .filter(|(key, _)| !Self::is_internal_object_key(key))
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        for key in keys {
            Self::mark_property_non_enumerable(entries, &key);
        }
    }

    pub(crate) fn mark_constructor_non_enumerable(entries: &Rc<RefCell<ObjectValue>>) {
        Self::mark_property_non_enumerable(entries, "constructor");
        Self::object_set_entry(
            &mut entries.borrow_mut(),
            INTERNAL_NON_ENUMERABLE_CONSTRUCTOR_KEY.to_string(),
            Value::Bool(true),
        );
    }

    pub(crate) fn constructor_prototype_from_value(
        &mut self,
        constructor: &Value,
    ) -> Option<Value> {
        match self.object_property_from_value(constructor, "prototype") {
            Ok(Value::Object(prototype)) => Some(Value::Object(prototype)),
            _ => None,
        }
    }

    pub(crate) fn constructor_prototype_from_env(&mut self, name: &str) -> Option<Value> {
        let constructor = self.script_runtime.env.get(name).cloned()?;
        self.constructor_prototype_from_value(&constructor)
    }

    pub(crate) fn object_constructor_prototype_value(&mut self) -> Value {
        self.constructor_prototype_from_env("Object")
            .unwrap_or_else(|| Self::new_object_value(Vec::new()))
    }

    pub(crate) fn cached_constructor_static_method_value(
        &mut self,
        cache_key: &str,
        make_value: impl FnOnce() -> Value,
    ) -> Value {
        if let Some(value) = self
            .script_runtime
            .constructor_static_methods
            .get(cache_key)
            .cloned()
        {
            return value;
        }
        let value = make_value();
        self.script_runtime
            .constructor_static_methods
            .insert(cache_key.to_string(), value.clone());
        value
    }

    fn object_to_string_tag_property(&mut self, value: &Value) -> Result<Option<String>> {
        let symbol = self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let key = self.property_key_to_storage_key(&symbol);
        match self.object_property_from_value(value, &key)? {
            Value::String(tag) if !tag.is_empty() => Ok(Some(tag)),
            _ => Ok(None),
        }
    }

    fn object_prototype_to_string_tag(&mut self, value: &Value) -> Result<String> {
        let tag = match value {
            Value::Null => "Null".to_string(),
            Value::Undefined => "Undefined".to_string(),
            Value::Bool(_) => "Boolean".to_string(),
            Value::Number(_) | Value::Float(_) => "Number".to_string(),
            Value::BigInt(_) => "BigInt".to_string(),
            Value::String(_) => "String".to_string(),
            Value::Symbol(_) => "Symbol".to_string(),
            Value::Array(values) => {
                if Self::is_dom_rect_list_value(&values.borrow()) {
                    "DOMRectList".to_string()
                } else {
                    "Array".to_string()
                }
            }
            Value::Promise(_) => "Promise".to_string(),
            Value::Map(_) => "Map".to_string(),
            Value::WeakMap(_) => "WeakMap".to_string(),
            Value::Set(_) => "Set".to_string(),
            Value::WeakSet(_) => "WeakSet".to_string(),
            Value::Blob(_) => "Blob".to_string(),
            Value::ArrayBuffer(_) => "ArrayBuffer".to_string(),
            Value::TypedArray(values) => values.borrow().kind.name().to_string(),
            Value::RegExp(_) => "RegExp".to_string(),
            Value::Date(_) => "Date".to_string(),
            Value::NodeList(nodes) => Self::node_list_display_name(nodes).to_string(),
            Value::FormData(_) => "FormData".to_string(),
            Value::Function(_) => "Function".to_string(),
            Value::StringConstructor
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::WeakMapConstructor
            | Value::SetConstructor
            | Value::WeakSetConstructor
            | Value::UrlSearchParamsConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::TypedArrayConstructor(_)
            | Value::PromiseCapability(_) => "Function".to_string(),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::string_wrapper_value_from_object(&entries).is_some() {
                    "String".to_string()
                } else if Self::boolean_wrapper_value_from_object(&entries).is_some() {
                    "Boolean".to_string()
                } else if Self::number_wrapper_value_from_object(&entries).is_some() {
                    "Number".to_string()
                } else if Self::bigint_wrapper_value_from_object(&entries).is_some() {
                    "BigInt".to_string()
                } else if Self::symbol_wrapper_id_from_object(&entries).is_some() {
                    "Symbol".to_string()
                } else if Self::callable_kind_from_value(value).is_some() {
                    "Function".to_string()
                } else if let Some(tag) = self.object_to_string_tag_property(value)? {
                    tag
                } else if Self::is_url_object(&entries) {
                    "URL".to_string()
                } else if Self::is_location_object(&entries) {
                    "Location".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "Document".to_string()
                } else if Self::is_range_object(&entries) {
                    "Range".to_string()
                } else if Self::is_selection_object(&entries) {
                    "Selection".to_string()
                } else if Self::is_match_media_object(&entries) {
                    "MediaQueryList".to_string()
                } else if Self::is_named_node_map_object(&entries) {
                    "NamedNodeMap".to_string()
                } else if Self::is_attr_object(&entries) {
                    "Attr".to_string()
                } else if Self::is_canvas_2d_context_object(&entries) {
                    "CanvasRenderingContext2D".to_string()
                } else if Self::is_class_list_object(&entries) {
                    "DOMTokenList".to_string()
                } else if Self::is_dom_rect_object(&entries) {
                    "DOMRect".to_string()
                } else if Self::is_image_bitmap_object(&entries) {
                    "ImageBitmap".to_string()
                } else if Self::is_text_track_object(&entries) {
                    "TextTrack".to_string()
                } else if Self::is_dom_string_map_object(&entries) {
                    "DOMStringMap".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOM_PARSER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "DOMParser".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_XML_SERIALIZER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "XMLSerializer".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_TREE_WALKER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "TreeWalker".to_string()
                } else if Self::is_css_style_sheet_object(&entries) {
                    "CSSStyleSheet".to_string()
                } else if Self::is_computed_style_object(&entries) {
                    "CSSStyleDeclaration".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_READABLE_STREAM_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "ReadableStream".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_WRITABLE_STREAM_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "WritableStream".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_TEXT_ENCODER_STREAM_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "TextEncoderStream".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "TextDecoderStream".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_ANIMATION_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "Animation".to_string()
                } else if Self::is_event_object(&entries) {
                    "Event".to_string()
                } else if Self::is_hash_change_event_object(&entries) {
                    "HashChangeEvent".to_string()
                } else if Self::is_error_event_object(&entries) {
                    "ErrorEvent".to_string()
                } else if Self::is_before_unload_event_object(&entries) {
                    "BeforeUnloadEvent".to_string()
                } else if Self::is_keyboard_event_object(&entries) {
                    "KeyboardEvent".to_string()
                } else if Self::is_wheel_event_object(&entries) {
                    "WheelEvent".to_string()
                } else if Self::is_navigate_event_object(&entries) {
                    "NavigateEvent".to_string()
                } else if Self::is_pointer_event_object(&entries) {
                    "PointerEvent".to_string()
                } else {
                    "Object".to_string()
                }
            }
            Value::Node(_) => "Object".to_string(),
        };
        Ok(tag)
    }

    pub(crate) fn object_prototype_to_string_value(&mut self, value: &Value) -> Result<Value> {
        Ok(Value::String(format!(
            "[object {}]",
            self.object_prototype_to_string_tag(value)?
        )))
    }

    pub(crate) fn object_prototype_value_of_value(&mut self, value: &Value) -> Result<Value> {
        match value {
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(
                "Object.valueOf called on null or undefined".into(),
            )),
            _ => Ok(value.clone()),
        }
    }

    fn object_prototype_reflection_target(
        &mut self,
        value: &Value,
        method_name: &str,
    ) -> Result<Value> {
        match value {
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(format!(
                "Object.{method_name} called on null or undefined"
            ))),
            Value::String(_)
            | Value::Bool(_)
            | Value::Number(_)
            | Value::Float(_)
            | Value::BigInt(_)
            | Value::Symbol(_) => Ok(Self::box_primitive_value(value.clone())),
            _ => Ok(value.clone()),
        }
    }

    pub(crate) fn object_prototype_has_own_property_value(
        &mut self,
        value: &Value,
        key: &Value,
    ) -> Result<Value> {
        let target = self.object_prototype_reflection_target(value, "hasOwnProperty")?;
        let key = self.property_key_to_storage_key(key);
        self.object_has_own_value(&target, &key)
    }

    pub(crate) fn object_prototype_property_is_enumerable_value(
        &mut self,
        value: &Value,
        key: &Value,
    ) -> Result<Value> {
        let target = self.object_prototype_reflection_target(value, "propertyIsEnumerable")?;
        let key = self.property_key_to_storage_key(key);
        let descriptor = self.object_get_own_property_descriptor_value(&target, &key)?;
        let Value::Object(_) = descriptor else {
            return Ok(Value::Bool(false));
        };
        Ok(Value::Bool(
            self.object_property_from_value(&descriptor, "enumerable")?
                .truthy(),
        ))
    }

    pub(crate) fn object_prototype_is_prototype_of_value(
        &mut self,
        prototype: &Value,
        value: &Value,
    ) -> Result<Value> {
        if matches!(prototype, Value::Null | Value::Undefined) {
            return Err(Error::ScriptRuntime(
                "Object.isPrototypeOf called on null or undefined".into(),
            ));
        }
        if Self::is_primitive_value(value) {
            return Ok(Value::Bool(false));
        }
        let mut current = self.value_internal_prototype_value(value);
        let mut hops = 0usize;
        while let Some(next) = current {
            if self.strict_equal(prototype, &next) {
                return Ok(Value::Bool(true));
            }
            hops += 1;
            if hops > 256 {
                break;
            }
            current = self.value_internal_prototype_value(&next);
        }
        Ok(Value::Bool(false))
    }
}
