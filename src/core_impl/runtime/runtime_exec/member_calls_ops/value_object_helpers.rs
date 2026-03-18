use super::*;

impl Harness {
    pub(crate) fn new_array_value(values: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(ArrayValue::new(values))))
    }

    pub(crate) fn set_array_property(array: &Rc<RefCell<ArrayValue>>, key: String, value: Value) {
        Self::object_set_entry(&mut array.borrow_mut().properties, key, value);
    }

    pub(crate) fn array_hole_storage_key(index: usize) -> String {
        format!("{INTERNAL_ARRAY_HOLE_KEY_PREFIX}{index}")
    }

    pub(crate) fn array_index_is_hole(array: &ArrayValue, index: usize) -> bool {
        let hole_key = Self::array_hole_storage_key(index);
        Self::object_get_entry(&array.properties, &hole_key).is_some()
    }

    pub(crate) fn clear_array_hole(array: &Rc<RefCell<ArrayValue>>, index: usize) {
        let hole_key = Self::array_hole_storage_key(index);
        array.borrow_mut().properties.delete_entry(&hole_key);
    }

    pub(crate) fn mark_array_hole(array: &Rc<RefCell<ArrayValue>>, index: usize) {
        let hole_key = Self::array_hole_storage_key(index);
        Self::object_set_entry(
            &mut array.borrow_mut().properties,
            hole_key,
            Value::Bool(true),
        );
    }

    pub(crate) fn delete_object_getter_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let getter_key = Self::object_getter_storage_key(key);
        let mut deleted = entries.delete_entry(&getter_key);
        let undefined_getter_key = Self::object_undefined_getter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_getter_key);
        deleted
    }

    pub(crate) fn delete_object_setter_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let setter_key = Self::object_setter_storage_key(key);
        let mut deleted = entries.delete_entry(&setter_key);
        let undefined_setter_key = Self::object_undefined_setter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_setter_key);
        deleted
    }

    pub(crate) fn delete_object_property_auxiliary_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let mut deleted = Self::delete_object_getter_entries(entries, key);
        deleted |= Self::delete_object_setter_entries(entries, key);
        let non_enumerable_key = Self::object_non_enumerable_storage_key(key);
        deleted |= entries.delete_entry(&non_enumerable_key);
        let non_writable_key = Self::object_non_writable_storage_key(key);
        deleted |= entries.delete_entry(&non_writable_key);
        let non_configurable_key = Self::object_non_configurable_storage_key(key);
        deleted |= entries.delete_entry(&non_configurable_key);
        deleted
    }

    pub(crate) fn delete_object_property_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let mut deleted = entries.delete_entry(key);
        let getter_key = Self::object_getter_storage_key(key);
        deleted |= entries.delete_entry(&getter_key);
        let setter_key = Self::object_setter_storage_key(key);
        deleted |= entries.delete_entry(&setter_key);
        let undefined_getter_key = Self::object_undefined_getter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_getter_key);
        let undefined_setter_key = Self::object_undefined_setter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_setter_key);
        let non_enumerable_key = Self::object_non_enumerable_storage_key(key);
        deleted |= entries.delete_entry(&non_enumerable_key);
        let non_writable_key = Self::object_non_writable_storage_key(key);
        deleted |= entries.delete_entry(&non_writable_key);
        let non_configurable_key = Self::object_non_configurable_storage_key(key);
        deleted |= entries.delete_entry(&non_configurable_key);
        deleted
    }

    pub(crate) fn new_object_value(entries: Vec<(String, Value)>) -> Value {
        Value::Object(Rc::new(RefCell::new(ObjectValue::new(entries))))
    }

    pub(crate) fn new_receiver_builtin_constructor_object(
        callable_kind: Option<&str>,
        family: &str,
        methods: &[&str],
    ) -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let mut constructor_entries = Vec::new();
        if let Some(kind) = callable_kind {
            constructor_entries.push((
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String(kind.to_string()),
            ));
        }
        constructor_entries.push(("prototype".to_string(), prototype.clone()));
        let constructor = Self::new_object_value(constructor_entries);
        if let Value::Object(prototype_entries) = &prototype {
            let mut prototype_entries = prototype_entries.borrow_mut();
            Self::object_set_entry(
                &mut prototype_entries,
                "constructor".to_string(),
                constructor.clone(),
            );
            for method in methods {
                Self::object_set_entry(
                    &mut prototype_entries,
                    (*method).to_string(),
                    Self::new_receiver_builtin_callable(family, method),
                );
            }
        }
        if let Value::Object(prototype_entries) = &prototype {
            Self::mark_existing_public_properties_non_enumerable(prototype_entries);
        }
        if let Value::Object(constructor_entries) = &constructor {
            Self::mark_property_non_enumerable(constructor_entries, "prototype");
        }
        constructor
    }

    pub(crate) fn new_object_backed_constructor_with_prototype(
        callable_kind: &str,
        extra_public_entries: Vec<(String, Value)>,
    ) -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let mut constructor_entries = vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String(callable_kind.to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ];
        constructor_entries.extend(extra_public_entries);
        let constructor = Self::new_object_value(constructor_entries);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
            Self::mark_existing_public_properties_non_enumerable(prototype_entries);
        }
        if let Value::Object(constructor_entries) = &constructor {
            Self::mark_existing_public_properties_non_enumerable(constructor_entries);
        }
        constructor
    }

    pub(crate) fn new_worker_context_post_message_callable(worker: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("worker_context_post_message".to_string()),
            ),
            (INTERNAL_WORKER_TARGET_KEY.to_string(), worker),
        ])
    }

    pub(crate) fn new_worker_main_post_message_callable(worker: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("worker_main_post_message".to_string()),
            ),
            (INTERNAL_WORKER_TARGET_KEY.to_string(), worker),
        ])
    }

    pub(crate) fn new_worker_terminate_callable(worker: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("worker_terminate".to_string()),
            ),
            (INTERNAL_WORKER_TARGET_KEY.to_string(), worker),
        ])
    }

    pub(crate) fn new_intl_collator_compare_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("intl_collator_get_compare".to_string()),
        )])
    }

    pub(crate) fn new_intl_date_time_format_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("intl_date_time_format_get_format".to_string()),
        )])
    }

    pub(crate) fn new_intl_number_format_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("intl_number_format_get_format".to_string()),
        )])
    }

    pub(crate) fn new_global_decode_uri_callable(component: bool) -> Value {
        let kind = if component {
            "global_decode_uri_component"
        } else {
            "global_decode_uri"
        };
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String(kind.to_string()),
        )])
    }

    pub(crate) fn new_global_atob_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_atob".to_string()),
        )])
    }

    pub(crate) fn new_global_btoa_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_btoa".to_string()),
        )])
    }

    pub(crate) fn new_global_structured_clone_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_structured_clone".to_string()),
        )])
    }

    pub(crate) fn new_global_css_escape_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_css_escape".to_string()),
        )])
    }

    pub(crate) fn new_global_request_animation_frame_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_request_animation_frame".to_string()),
        )])
    }

    pub(crate) fn new_global_set_timeout_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_set_timeout".to_string()),
        )])
    }

    pub(crate) fn new_global_set_interval_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_set_interval".to_string()),
        )])
    }

    pub(crate) fn new_global_cancel_animation_frame_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_cancel_animation_frame".to_string()),
        )])
    }

    pub(crate) fn new_global_clear_interval_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_clear_interval".to_string()),
        )])
    }

    pub(crate) fn new_global_clear_timeout_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_clear_timeout".to_string()),
        )])
    }

    pub(crate) fn new_global_queue_microtask_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_queue_microtask".to_string()),
        )])
    }

    pub(crate) fn new_create_image_bitmap_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("create_image_bitmap".to_string()),
        )])
    }

    pub(crate) fn new_dom_parser_instance_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_DOM_PARSER_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )])
    }

    pub(crate) fn new_xml_serializer_instance_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_XML_SERIALIZER_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )])
    }

    pub(crate) fn new_number_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("number_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    pub(crate) fn new_object_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("object_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    pub(crate) fn new_reflect_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("reflect_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    pub(crate) fn new_bigint_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("bigint_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    pub(crate) fn new_regexp_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("regexp_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    pub(crate) fn new_promise_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("promise_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    pub(crate) fn new_array_buffer_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("array_buffer_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    pub(crate) fn new_symbol_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("symbol_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    pub(crate) fn new_typed_array_static_method_callable(
        kind: TypedArrayConstructorKind,
        method: &str,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("typed_array_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_TYPED_ARRAY_KIND_KEY.to_string(),
                Value::TypedArrayConstructor(kind),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    pub(crate) fn new_bound_function_callable(
        target: Value,
        bound_this: Value,
        bound_args: Vec<Value>,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("bound_function".to_string()),
            ),
            (INTERNAL_BOUND_CALLABLE_TARGET_KEY.to_string(), target),
            (INTERNAL_BOUND_CALLABLE_THIS_KEY.to_string(), bound_this),
            (
                INTERNAL_BOUND_CALLABLE_ARGS_KEY.to_string(),
                Self::new_array_value(bound_args),
            ),
            ("call".to_string(), Self::new_function_call_callable()),
            ("apply".to_string(), Self::new_function_apply_callable()),
            ("bind".to_string(), Self::new_function_bind_callable()),
        ])
    }

    pub(crate) fn new_receiver_builtin_callable(family: &str, member: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("receiver_builtin_method".to_string()),
            ),
            (
                "__bt_receiver_builtin_family".to_string(),
                Value::String(family.to_string()),
            ),
            (
                "__bt_receiver_builtin_member".to_string(),
                Value::String(member.to_string()),
            ),
        ])
    }

    pub(crate) fn new_receiver_builtin_prototype_value(
        constructor: Value,
        family: &str,
        methods: &[&str],
    ) -> Value {
        let mut entries = vec![("constructor".to_string(), constructor)];
        for method in methods {
            entries.push((
                (*method).to_string(),
                Self::new_receiver_builtin_callable(family, method),
            ));
        }
        let prototype = Self::new_object_value(entries);
        if let Value::Object(entries) = &prototype {
            Self::mark_existing_public_properties_non_enumerable(entries);
        }
        prototype
    }

    pub(crate) fn new_receiver_builtin_prototype_with_iterator_value(
        &mut self,
        constructor: Value,
        family: &str,
        methods: &[&str],
        iterator_member: Option<&str>,
    ) -> Value {
        let prototype = Self::new_receiver_builtin_prototype_value(constructor, family, methods);
        let Some(iterator_member) = iterator_member else {
            return prototype;
        };
        let Value::Object(entries) = &prototype else {
            return prototype;
        };
        let iterator_symbol = self.eval_symbol_static_property(SymbolStaticProperty::Iterator);
        let iterator_key = self.property_key_to_storage_key(&iterator_symbol);
        Self::object_set_entry(
            &mut entries.borrow_mut(),
            iterator_key,
            Self::new_receiver_builtin_callable(family, iterator_member),
        );
        prototype
    }

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

    pub(crate) fn callable_kind_from_value(value: &Value) -> Option<&str> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        match Self::object_get_entry(&entries, INTERNAL_CALLABLE_KIND_KEY) {
            Some(Value::String(kind)) => Some(match kind.as_str() {
                "intl_collator_compare" => "intl_collator_compare",
                "intl_date_time_format" => "intl_date_time_format",
                "intl_duration_format" => "intl_duration_format",
                "intl_list_format" => "intl_list_format",
                "intl_number_format" => "intl_number_format",
                "intl_segmenter_segments_iterator" => "intl_segmenter_segments_iterator",
                "intl_segmenter_iterator_next" => "intl_segmenter_iterator_next",
                "readable_stream_async_iterator" => "readable_stream_async_iterator",
                "named_node_map_iterator" => "named_node_map_iterator",
                "iterator_self" => "iterator_self",
                "async_iterator_next" => "async_iterator_next",
                "async_iterator_return" => "async_iterator_return",
                "async_iterator_throw" => "async_iterator_throw",
                "async_iterator_self" => "async_iterator_self",
                "async_iterator_async_dispose" => "async_iterator_async_dispose",
                "async_generator_result_value" => "async_generator_result_value",
                "async_generator_result_done" => "async_generator_result_done",
                "async_generator_function_constructor" => "async_generator_function_constructor",
                "generator_function_constructor" => "generator_function_constructor",
                "boolean_constructor" => "boolean_constructor",
                "number_constructor" => "number_constructor",
                "bigint_constructor" => "bigint_constructor",
                "object_constructor" => "object_constructor",
                "object_static_method" => "object_static_method",
                "function_constructor" => "function_constructor",
                "node_list_constructor" => "node_list_constructor",
                "image_bitmap_constructor" => "image_bitmap_constructor",
                "text_track_constructor" => "text_track_constructor",
                "text_track_list_constructor" => "text_track_list_constructor",
                "time_ranges_constructor" => "time_ranges_constructor",
                "storage_constructor" => "storage_constructor",
                "cookie_store_constructor" => "cookie_store_constructor",
                "cache_storage_constructor" => "cache_storage_constructor",
                "cache_constructor" => "cache_constructor",
                "radio_node_list_constructor" => "radio_node_list_constructor",
                "html_collection_constructor" => "html_collection_constructor",
                "html_form_controls_collection_constructor" => {
                    "html_form_controls_collection_constructor"
                }
                "html_options_collection_constructor" => "html_options_collection_constructor",
                "event_target_constructor" => "event_target_constructor",
                "event_constructor" => "event_constructor",
                "custom_event_constructor" => "custom_event_constructor",
                "mouse_event_constructor" => "mouse_event_constructor",
                "keyboard_event_constructor" => "keyboard_event_constructor",
                "wheel_event_constructor" => "wheel_event_constructor",
                "navigate_event_constructor" => "navigate_event_constructor",
                "pointer_event_constructor" => "pointer_event_constructor",
                "error_event_constructor" => "error_event_constructor",
                "hash_change_event_constructor" => "hash_change_event_constructor",
                "before_unload_event_constructor" => "before_unload_event_constructor",
                "image_data_constructor" => "image_data_constructor",
                "dom_parser_constructor" => "dom_parser_constructor",
                "xml_serializer_constructor" => "xml_serializer_constructor",
                "document_constructor" => "document_constructor",
                "document_parse_html" => "document_parse_html",
                "document_parse_html_unsafe" => "document_parse_html_unsafe",
                "fetch_function" => "fetch_function",
                "match_media_function" => "match_media_function",
                "window_close_function" => "window_close_function",
                "window_open_function" => "window_open_function",
                "window_stop_function" => "window_stop_function",
                "window_focus_function" => "window_focus_function",
                "window_scroll_function" => "window_scroll_function",
                "window_scroll_by_function" => "window_scroll_by_function",
                "window_scroll_to_function" => "window_scroll_to_function",
                "window_move_by_function" => "window_move_by_function",
                "window_move_to_function" => "window_move_to_function",
                "window_resize_by_function" => "window_resize_by_function",
                "window_resize_to_function" => "window_resize_to_function",
                "window_post_message_function" => "window_post_message_function",
                "window_get_computed_style_function" => "window_get_computed_style_function",
                "computed_style_item" => "computed_style_item",
                "dom_rect_list_item" => "dom_rect_list_item",
                "window_alert_function" => "window_alert_function",
                "window_confirm_function" => "window_confirm_function",
                "window_print_function" => "window_print_function",
                "window_report_error_function" => "window_report_error_function",
                "window_prompt_function" => "window_prompt_function",
                "popup_window_close_function" => "popup_window_close_function",
                "popup_window_focus_function" => "popup_window_focus_function",
                "popup_window_print_function" => "popup_window_print_function",
                "popup_document_open_function" => "popup_document_open_function",
                "popup_document_write_function" => "popup_document_write_function",
                "popup_document_close_function" => "popup_document_close_function",
                "request_constructor" => "request_constructor",
                "file_constructor" => "file_constructor",
                "clipboard_item_constructor" => "clipboard_item_constructor",
                "clipboard_write" => "clipboard_write",
                "headers_constructor" => "headers_constructor",
                "worker_constructor" => "worker_constructor",
                "data_transfer_constructor" => "data_transfer_constructor",
                "option_constructor" => "option_constructor",
                "audio_constructor" => "audio_constructor",
                "text_encoder_constructor" => "text_encoder_constructor",
                "text_decoder_constructor" => "text_decoder_constructor",
                "text_encoder_stream_constructor" => "text_encoder_stream_constructor",
                "text_decoder_stream_constructor" => "text_decoder_stream_constructor",
                "text_encoder_get_encoding" => "text_encoder_get_encoding",
                "text_encoder_encode" => "text_encoder_encode",
                "text_encoder_encode_into" => "text_encoder_encode_into",
                "text_decoder_get_encoding" => "text_decoder_get_encoding",
                "text_decoder_get_fatal" => "text_decoder_get_fatal",
                "text_decoder_get_ignore_bom" => "text_decoder_get_ignore_bom",
                "text_decoder_decode" => "text_decoder_decode",
                "text_encoder_stream_get_encoding" => "text_encoder_stream_get_encoding",
                "text_encoder_stream_get_readable" => "text_encoder_stream_get_readable",
                "text_encoder_stream_get_writable" => "text_encoder_stream_get_writable",
                "text_decoder_stream_get_encoding" => "text_decoder_stream_get_encoding",
                "text_decoder_stream_get_fatal" => "text_decoder_stream_get_fatal",
                "text_decoder_stream_get_ignore_bom" => "text_decoder_stream_get_ignore_bom",
                "text_decoder_stream_get_readable" => "text_decoder_stream_get_readable",
                "text_decoder_stream_get_writable" => "text_decoder_stream_get_writable",
                "css_style_sheet_constructor" => "css_style_sheet_constructor",
                "css_style_sheet_replace_sync" => "css_style_sheet_replace_sync",
                "css_style_sheet_insert_rule" => "css_style_sheet_insert_rule",
                "computed_style_get_property_value" => "computed_style_get_property_value",
                "class_list_add" => "class_list_add",
                "class_list_remove" => "class_list_remove",
                "class_list_toggle" => "class_list_toggle",
                "class_list_contains" => "class_list_contains",
                "class_list_replace" => "class_list_replace",
                "class_list_item" => "class_list_item",
                "class_list_for_each" => "class_list_for_each",
                "class_list_keys" => "class_list_keys",
                "class_list_values" => "class_list_values",
                "class_list_entries" => "class_list_entries",
                "class_list_to_string" => "class_list_to_string",
                "named_node_map_item" => "named_node_map_item",
                "named_node_map_get_named_item" => "named_node_map_get_named_item",
                "named_node_map_set_named_item" => "named_node_map_set_named_item",
                "named_node_map_remove_named_item" => "named_node_map_remove_named_item",
                "named_node_map_get_named_item_ns" => "named_node_map_get_named_item_ns",
                "named_node_map_set_named_item_ns" => "named_node_map_set_named_item_ns",
                "named_node_map_remove_named_item_ns" => "named_node_map_remove_named_item_ns",
                "named_node_map_for_each" => "named_node_map_for_each",
                "named_node_map_keys" => "named_node_map_keys",
                "named_node_map_values" => "named_node_map_values",
                "named_node_map_entries" => "named_node_map_entries",
                "worker_main_post_message" => "worker_main_post_message",
                "worker_context_post_message" => "worker_context_post_message",
                "worker_terminate" => "worker_terminate",
                "intl_collator_get_compare" => "intl_collator_get_compare",
                "intl_date_time_format_get_format" => "intl_date_time_format_get_format",
                "intl_number_format_get_format" => "intl_number_format_get_format",
                "global_decode_uri" => "global_decode_uri",
                "global_decode_uri_component" => "global_decode_uri_component",
                "global_atob" => "global_atob",
                "global_btoa" => "global_btoa",
                "global_css_escape" => "global_css_escape",
                "global_structured_clone" => "global_structured_clone",
                "global_request_animation_frame" => "global_request_animation_frame",
                "global_set_timeout" => "global_set_timeout",
                "global_set_interval" => "global_set_interval",
                "global_cancel_animation_frame" => "global_cancel_animation_frame",
                "global_clear_interval" => "global_clear_interval",
                "global_clear_timeout" => "global_clear_timeout",
                "global_queue_microtask" => "global_queue_microtask",
                "create_image_bitmap" => "create_image_bitmap",
                "string_static_from_char_code" => "string_static_from_char_code",
                "string_static_from_code_point" => "string_static_from_code_point",
                "string_static_raw" => "string_static_raw",
                "number_static_method" => "number_static_method",
                "bigint_static_method" => "bigint_static_method",
                "regexp_static_method" => "regexp_static_method",
                "promise_static_method" => "promise_static_method",
                "array_buffer_static_method" => "array_buffer_static_method",
                "symbol_static_method" => "symbol_static_method",
                "typed_array_static_method" => "typed_array_static_method",
                "reflect_static_method" => "reflect_static_method",
                "function_call" => "function_call",
                "function_apply" => "function_apply",
                "function_bind" => "function_bind",
                "function_to_string" => "function_to_string",
                "bound_function" => "bound_function",
                "receiver_builtin_method" => "receiver_builtin_method",
                _ => return None,
            }),
            _ => None,
        }
    }

    pub(crate) fn data_attr_name_to_dataset_key(attr_name: &str) -> Option<String> {
        let raw = attr_name.strip_prefix("data-")?;
        if raw.is_empty() {
            return None;
        }
        let normalized = raw.to_ascii_lowercase();
        let chars = normalized.chars().collect::<Vec<_>>();
        let mut out = String::with_capacity(chars.len());
        let mut index = 0usize;
        while index < chars.len() {
            let ch = chars[index];
            if ch == '-' {
                if let Some(next) = chars.get(index + 1).copied() {
                    if next.is_ascii_lowercase() {
                        out.push(next.to_ascii_uppercase());
                        index += 2;
                        continue;
                    }
                }
                out.push(ch);
            } else {
                out.push(ch);
            }
            index += 1;
        }
        if out.is_empty() { None } else { Some(out) }
    }

    pub(crate) fn dataset_entries_for_node(&self, node: NodeId) -> Vec<(String, Value)> {
        let Some(element) = self.dom.element(node) else {
            return Vec::new();
        };
        let mut entries = element
            .attrs
            .iter()
            .filter_map(|(attr_name, attr_value)| {
                Self::data_attr_name_to_dataset_key(attr_name)
                    .map(|key| (key, Value::String(attr_value.clone())))
            })
            .collect::<Vec<_>>();
        entries.sort_by(|(left, _), (right, _)| left.cmp(right));
        entries
    }

    pub(crate) fn is_to_string_tag_property_key(&self, key: &str) -> bool {
        Self::symbol_id_from_storage_key(key)
            .and_then(|symbol_id| self.symbol_runtime.symbols_by_id.get(&symbol_id))
            .and_then(|symbol| symbol.description.as_deref())
            .is_some_and(|description| description == "Symbol.toStringTag")
            || key == "Symbol.toStringTag"
    }

    pub(crate) fn is_iterator_property_key(&self, key: &str) -> bool {
        Self::symbol_id_from_storage_key(key)
            .and_then(|symbol_id| self.symbol_runtime.symbols_by_id.get(&symbol_id))
            .and_then(|symbol| symbol.description.as_deref())
            .is_some_and(|description| description == "Symbol.iterator")
            || key == "Symbol.iterator"
    }

    pub(crate) fn is_string_method_name(name: &str) -> bool {
        matches!(
            name,
            "concat"
                | "endsWith"
                | "includes"
                | "normalize"
                | "slice"
                | "split"
                | "startsWith"
                | "substring"
        )
    }

    pub(crate) fn is_array_method_name(name: &str) -> bool {
        matches!(
            name,
            "forEach"
                | "map"
                | "flat"
                | "flatMap"
                | "filter"
                | "reduce"
                | "find"
                | "findIndex"
                | "some"
                | "every"
                | "values"
                | "keys"
                | "entries"
                | "fill"
                | "includes"
                | "indexOf"
                | "lastIndexOf"
                | "slice"
                | "join"
                | "concat"
                | "add"
                | "remove"
                | "clear"
                | "push"
                | "pop"
                | "shift"
                | "unshift"
                | "splice"
                | "sort"
                | "reverse"
        )
    }

    pub(crate) fn is_class_list_method_name(name: &str) -> bool {
        matches!(
            name,
            "add"
                | "remove"
                | "toggle"
                | "contains"
                | "replace"
                | "item"
                | "forEach"
                | "keys"
                | "values"
                | "entries"
                | "toString"
        )
    }

    pub(crate) fn is_named_node_map_method_name(name: &str) -> bool {
        matches!(
            name,
            "item"
                | "getNamedItem"
                | "setNamedItem"
                | "removeNamedItem"
                | "getNamedItemNS"
                | "setNamedItemNS"
                | "removeNamedItemNS"
                | "forEach"
                | "keys"
                | "values"
                | "entries"
        )
    }

    pub(crate) fn is_typed_array_method_name(name: &str) -> bool {
        matches!(
            name,
            "at" | "copyWithin"
                | "entries"
                | "join"
                | "keys"
                | "slice"
                | "subarray"
                | "values"
                | "with"
        )
    }
}
