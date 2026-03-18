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

    fn new_class_list_method_callable(kind: &str) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String(kind.to_string()),
        )])
    }

    fn new_named_node_map_method_callable(kind: &str) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String(kind.to_string()),
        )])
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

    fn is_string_method_name(name: &str) -> bool {
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

    fn is_array_method_name(name: &str) -> bool {
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

    fn is_class_list_method_name(name: &str) -> bool {
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

    fn is_named_node_map_method_name(name: &str) -> bool {
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

    fn is_typed_array_method_name(name: &str) -> bool {
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

    fn object_property_from_node_value(&mut self, node: &NodeId, key: &str) -> Result<Value> {
        let is_canvas = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("canvas"))
            .unwrap_or(false);
        let is_select = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("select"))
            .unwrap_or(false);
        let is_datalist = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("datalist"))
            .unwrap_or(false);
        let is_input = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false);
        let is_option = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("option"))
            .unwrap_or(false);
        let is_textarea = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("textarea"))
            .unwrap_or(false);
        let is_output = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("output"))
            .unwrap_or(false);
        let is_button = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("button"))
            .unwrap_or(false);
        let is_form = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("form"))
            .unwrap_or(false);
        let is_media = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video"))
            .unwrap_or(false);
        let is_form_associated_control = is_form_control(&self.dom, *node);
        let is_labelable_control = self.is_labelable_control(*node);
        let is_col_or_colgroup = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("col") || tag.eq_ignore_ascii_case("colgroup"))
            .unwrap_or(false);
        let is_table_cell = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th"))
            .unwrap_or(false);

        if is_select {
            if let Ok(index) = key.parse::<usize>() {
                return Ok(self
                    .select_option_nodes(*node)
                    .get(index)
                    .copied()
                    .map(Value::Node)
                    .unwrap_or(Value::Undefined));
            }
        }

        if self.node_explicit_own_property_overrides_dom_property(*node, key) {
            let entries = self.node_expando_entries(*node);
            if let Some(value) =
                self.object_property_from_entries_with_getter(&Value::Node(*node), &entries, key)?
            {
                return Ok(value);
            }
        }

        if let Some(value) = self.node_receiver_builtin_method(*node, key) {
            return Ok(value);
        }

        match key {
            "nodeType" => Ok(Value::Number(self.node_type_number(*node))),
            "nodeName" => Ok(Value::String(self.node_name(*node))),
            "nodeValue" => Ok(self.node_value(*node)),
            "ownerDocument" => Ok(self
                .node_owner_document(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "parentNode" => Ok(self
                .dom
                .parent(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "parentElement" => Ok(self
                .node_parent_element(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "nextSibling" => Ok(self
                .node_next_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "previousSibling" => Ok(self
                .node_previous_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "isConnected" => Ok(Value::Bool(self.dom.is_connected(*node))),
            "childNodes" => Ok(self.child_nodes_live_list_value(*node)),
            "attributes" => {
                if self.dom.element(*node).is_some() {
                    Ok(self.named_node_map_live_value(*node))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "children" => Ok(self.child_elements_live_list_value(*node)),
            "childElementCount" => Ok(Value::Number(self.dom.child_element_count(*node) as i64)),
            "firstChild" => Ok(self.dom.nodes[node.0]
                .children
                .first()
                .copied()
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "lastChild" => Ok(self.dom.nodes[node.0]
                .children
                .last()
                .copied()
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "firstElementChild" => Ok(self
                .dom
                .first_element_child(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "lastElementChild" => Ok(self
                .dom
                .last_element_child(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "nextElementSibling" => Ok(self
                .dom
                .next_element_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "previousElementSibling" => Ok(self
                .dom
                .previous_element_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "shadowRoot" => Ok(self.shadow_root_property_value(*node)),
            "content"
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("template")) =>
            {
                self.template_content_fragment_value(*node)
            }
            "textContent" => Ok(self.node_text_content_value(*node)),
            "innerText" => Ok(Value::String(self.dom.text_content(*node))),
            "innerHTML" => Ok(Value::String(self.dom.inner_html(*node)?)),
            "outerHTML" => Ok(Value::String(self.dom.outer_html(*node)?)),
            "defaultValue" => {
                if is_input || is_textarea || is_output {
                    Ok(Value::String(
                        self.dom
                            .element(*node)
                            .map(|element| element.default_value.clone())
                            .unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "value" => Ok(Value::String(self.dom.value(*node)?)),
            "files" => self.input_files_value(*node),
            "valueAsNumber" => Ok(Self::number_value(self.input_value_as_number(*node)?)),
            "valueAsDate" => Ok(self
                .input_value_as_date_ms(*node)?
                .map(Self::new_date_value)
                .unwrap_or(Value::Null)),
            "defaultChecked" => {
                if is_input {
                    Ok(Value::Bool(
                        self.dom
                            .element(*node)
                            .map(|element| element.default_checked)
                            .unwrap_or(false),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "checked" => Ok(Value::Bool(self.dom.checked(*node)?)),
            "defaultSelected" => {
                if is_option {
                    Ok(Value::Bool(
                        self.dom
                            .element(*node)
                            .map(|element| element.default_selected)
                            .unwrap_or(false),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "selected" => {
                if is_option {
                    Ok(Value::Bool(
                        self.dom
                            .element(*node)
                            .map(|element| element.selected)
                            .unwrap_or(false),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "disabled" => Ok(Value::Bool(self.dom.disabled(*node))),
            "required" => Ok(Value::Bool(self.dom.required(*node))),
            "multiple" => {
                if is_select || is_input {
                    Ok(Value::Bool(self.dom.attr(*node, "multiple").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "readonly" | "readOnly" => Ok(Value::Bool(self.dom.readonly(*node))),
            "autocomplete" => Ok(Value::String(
                self.dom.attr(*node, "autocomplete").unwrap_or_default(),
            )),
            "form" => {
                if is_form_associated_control {
                    Ok(self
                        .resolve_form_for_submit(*node)
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "elements" => {
                if is_form {
                    self.form_elements_live_list_value(*node)
                } else {
                    Ok(Value::Undefined)
                }
            }
            "action" => {
                if is_form {
                    Ok(Value::String(
                        self.form_action_property_value_for_node(*node),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "method" => {
                if is_form {
                    Ok(Value::String(
                        self.dom.attr(*node, "method").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "enctype" | "encoding" => {
                if is_form {
                    Ok(Value::String(
                        self.dom.attr(*node, "enctype").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "target" => {
                if is_form
                    || self.dom.tag_name(*node).is_some_and(|tag| {
                        tag.eq_ignore_ascii_case("a")
                            || tag.eq_ignore_ascii_case("area")
                            || tag.eq_ignore_ascii_case("base")
                    })
                {
                    Ok(Value::String(
                        self.dom.attr(*node, "target").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "acceptCharset" => {
                if is_form {
                    Ok(Value::String(
                        self.dom.attr(*node, "accept-charset").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "noValidate" => {
                if is_form {
                    Ok(Value::Bool(self.dom.attr(*node, "novalidate").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "command" => {
                if is_button {
                    Ok(Value::String(
                        self.dom.attr(*node, "command").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "commandForElement" => {
                if is_button {
                    Ok(self
                        .dom
                        .attr(*node, "commandfor")
                        .and_then(|raw| raw.split_whitespace().next().map(str::to_string))
                        .and_then(|id_ref| self.dom.by_id(&id_ref))
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formAction" => {
                if is_button || is_input {
                    Ok(Value::String(
                        self.submitter_form_action_property_value_for_node(*node),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "href" => Ok(Value::String(self.resolve_anchor_href(*node))),
            "download"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "download").unwrap_or_default(),
                ))
            }
            "hreflang"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("link")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "hreflang").unwrap_or_default(),
                ))
            }
            "ping"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "ping").unwrap_or_default(),
                ))
            }
            "referrerPolicy" | "referrerpolicy"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("img")
                        || tag.eq_ignore_ascii_case("link")
                        || tag.eq_ignore_ascii_case("script")
                        || tag.eq_ignore_ascii_case("iframe")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "referrerpolicy").unwrap_or_default(),
                ))
            }
            "rel"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("link")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "rel").unwrap_or_default(),
                ))
            }
            "alt"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("img")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "alt").unwrap_or_default(),
                ))
            }
            "charset"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "charset").unwrap_or_default(),
                ))
            }
            "coords"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "coords").unwrap_or_default(),
                ))
            }
            "rev"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "rev").unwrap_or_default(),
                ))
            }
            "shape"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "shape").unwrap_or_default(),
                ))
            }
            "noHref" | "nohref"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "nohref").is_some()))
            }
            "formEnctype" => {
                if is_button {
                    Ok(Value::String(
                        self.dom.attr(*node, "formenctype").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formMethod" => {
                if is_button {
                    Ok(Value::String(
                        self.dom.attr(*node, "formmethod").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formNoValidate" => {
                if is_button {
                    Ok(Value::Bool(
                        self.dom.attr(*node, "formnovalidate").is_some(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formTarget" => {
                if is_button {
                    Ok(Value::String(
                        self.dom.attr(*node, "formtarget").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "labels" => {
                if is_labelable_control {
                    Ok(Self::new_static_node_list_value(
                        self.labels_for_control_node(*node),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "id" => Ok(Value::String(
                self.dom.attr(*node, "id").unwrap_or_default(),
            )),
            "name" => Ok(Value::String(
                self.dom.attr(*node, "name").unwrap_or_default(),
            )),
            "interestForElement" => {
                if is_button {
                    Ok(self
                        .dom
                        .attr(*node, "interestfor")
                        .and_then(|raw| raw.split_whitespace().next().map(str::to_string))
                        .and_then(|id_ref| self.dom.by_id(&id_ref))
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "popoverTargetAction" => {
                if is_button {
                    Ok(Value::String(
                        self.dom
                            .attr(*node, "popovertargetaction")
                            .unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "popoverTargetElement" => {
                if is_button {
                    Ok(self
                        .dom
                        .attr(*node, "popovertarget")
                        .and_then(|raw| raw.split_whitespace().next().map(str::to_string))
                        .and_then(|id_ref| self.dom.by_id(&id_ref))
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "lang" => Ok(Value::String(
                self.dom.attr(*node, "lang").unwrap_or_default(),
            )),
            "dir" => Ok(Value::String(self.resolved_dir_for_node(*node))),
            "accessKey" | "accesskey" => Ok(Value::String(
                self.dom.attr(*node, "accesskey").unwrap_or_default(),
            )),
            "autocapitalize" => Ok(Value::String(
                self.dom.attr(*node, "autocapitalize").unwrap_or_default(),
            )),
            "autocorrect" => Ok(Value::String(
                self.dom.attr(*node, "autocorrect").unwrap_or_default(),
            )),
            "contentEditable" | "contenteditable" => Ok(Value::String(
                self.content_editable_property_value_for_node(*node),
            )),
            "draggable" => Ok(Value::Bool(self.draggable_property_value_for_node(*node))),
            "enterKeyHint" | "enterkeyhint" => Ok(Value::String(
                self.dom.attr(*node, "enterkeyhint").unwrap_or_default(),
            )),
            "inert" => Ok(Value::Bool(self.dom.has_attr(*node, "inert")?)),
            "inputMode" | "inputmode" => Ok(Value::String(
                self.dom.attr(*node, "inputmode").unwrap_or_default(),
            )),
            "nonce" => Ok(Value::String(
                self.dom.attr(*node, "nonce").unwrap_or_default(),
            )),
            "popover" => Ok(Value::String(
                self.dom.attr(*node, "popover").unwrap_or_default(),
            )),
            "spellcheck" => Ok(Value::Bool(self.spellcheck_property_value_for_node(*node))),
            "tabIndex" | "tabindex" => Ok(Value::Number(
                self.reflected_i64_attribute_or_default(*node, "tabindex", -1),
            )),
            "translate" => Ok(Value::Bool(self.translate_property_value_for_node(*node))),
            "cite" => Ok(Value::String(
                self.reflected_url_attribute_or_empty(*node, "cite"),
            )),
            "dateTime" | "datetime" => Ok(Value::String(
                self.dom.attr(*node, "datetime").unwrap_or_default(),
            )),
            "clear" => Ok(Value::String(
                self.dom.attr(*node, "clear").unwrap_or_default(),
            )),
            "align" => Ok(Value::String(
                self.dom.attr(*node, "align").unwrap_or_default(),
            )),
            "aLink" | "alink" => Ok(Value::String(
                self.dom.attr(*node, "alink").unwrap_or_default(),
            )),
            "background" => Ok(Value::String(
                self.dom.attr(*node, "background").unwrap_or_default(),
            )),
            "bgColor" | "bgcolor" => Ok(Value::String(
                self.dom.attr(*node, "bgcolor").unwrap_or_default(),
            )),
            "bottomMargin" | "bottommargin" => Ok(Value::String(
                self.dom.attr(*node, "bottommargin").unwrap_or_default(),
            )),
            "leftMargin" | "leftmargin" => Ok(Value::String(
                self.dom.attr(*node, "leftmargin").unwrap_or_default(),
            )),
            "link" => Ok(Value::String(
                self.dom.attr(*node, "link").unwrap_or_default(),
            )),
            "rightMargin" | "rightmargin" => Ok(Value::String(
                self.dom.attr(*node, "rightmargin").unwrap_or_default(),
            )),
            "text" => Ok(Value::String(
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
                {
                    self.dom.attr(*node, "text").unwrap_or_default()
                } else {
                    self.dom.text_content(*node)
                },
            )),
            "topMargin" | "topmargin" => Ok(Value::String(
                self.dom.attr(*node, "topmargin").unwrap_or_default(),
            )),
            "vLink" | "vlink" => Ok(Value::String(
                self.dom.attr(*node, "vlink").unwrap_or_default(),
            )),
            "title" => Ok(Value::String(
                self.dom.attr(*node, "title").unwrap_or_default(),
            )),
            "colSpan" | "colspan" if is_table_cell => {
                Ok(Value::Number(self.table_cell_col_span_value(*node)))
            }
            "rowSpan" | "rowspan" if is_table_cell => {
                Ok(Value::Number(self.table_cell_row_span_value(*node)))
            }
            "span" if is_col_or_colgroup => Ok(Value::Number(self.col_span_value(*node))),
            "type" => {
                if is_select {
                    Ok(Value::String(self.select_type_property_value(*node)))
                } else if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("button"))
                {
                    let normalized = self
                        .dom
                        .attr(*node, "type")
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                        .map(|value| {
                            if value.eq_ignore_ascii_case("reset") {
                                "reset".to_string()
                            } else if value.eq_ignore_ascii_case("button") {
                                "button".to_string()
                            } else {
                                "submit".to_string()
                            }
                        })
                        .unwrap_or_else(|| "submit".to_string());
                    Ok(Value::String(normalized))
                } else {
                    Ok(Value::String(
                        self.dom.attr(*node, "type").unwrap_or_default(),
                    ))
                }
            }
            "kind" if self.is_track_element(*node) => {
                Ok(Value::String(self.normalized_track_kind(*node)))
            }
            "track" if self.is_track_element(*node) => Ok(self.text_track_object_value(*node)),
            "srclang" | "srcLang" if self.is_track_element(*node) => Ok(Value::String(
                self.dom.attr(*node, "srclang").unwrap_or_default(),
            )),
            "label" if self.is_track_element(*node) => Ok(Value::String(
                self.dom.attr(*node, "label").unwrap_or_default(),
            )),
            "default" if self.is_track_element(*node) => {
                Ok(Value::Bool(self.dom.attr(*node, "default").is_some()))
            }
            "readyState" if self.is_track_element(*node) => Ok(Value::Number(0)),
            "defaultMuted"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "muted").is_some()))
            }
            "autoplay"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "autoplay").is_some()))
            }
            "controls"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "controls").is_some()))
            }
            "loop"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "loop").is_some()))
            }
            "muted"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "muted").is_some()))
            }
            "controlsList" | "controlslist" => Ok(Value::String(
                self.dom.attr(*node, "controlslist").unwrap_or_default(),
            )),
            "crossOrigin" | "crossorigin" => Ok(Value::String(
                self.dom.attr(*node, "crossorigin").unwrap_or_default(),
            )),
            "disableRemotePlayback" | "disableremoteplayback" => Ok(Value::Bool(
                self.dom.attr(*node, "disableremoteplayback").is_some(),
            )),
            "disablePictureInPicture" | "disablepictureinpicture" => Ok(Value::Bool(
                self.dom.attr(*node, "disablepictureinpicture").is_some(),
            )),
            "media" => Ok(Value::String(
                self.dom.attr(*node, "media").unwrap_or_default(),
            )),
            "playsInline" | "playsinline" => {
                Ok(Value::Bool(self.dom.attr(*node, "playsinline").is_some()))
            }
            "paused"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_boolean_state_value(*node, INTERNAL_MEDIA_PAUSED_KEY, true))
            }
            "ended"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(false))
            }
            "seeking"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(false))
            }
            "networkState"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                let state = if self.resolve_media_src(*node).is_empty() {
                    0
                } else {
                    1
                };
                Ok(Value::Number(state))
            }
            "readyState"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Number(0))
            }
            "currentTime"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_numeric_state_value(*node, INTERNAL_MEDIA_CURRENT_TIME_KEY, 0.0))
            }
            "volume"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_numeric_state_value(*node, INTERNAL_MEDIA_VOLUME_KEY, 1.0))
            }
            "duration"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_numeric_state_value(*node, INTERNAL_MEDIA_DURATION_KEY, f64::NAN))
            }
            "playbackRate"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_numeric_state_value(*node, INTERNAL_MEDIA_PLAYBACK_RATE_KEY, 1.0))
            }
            "defaultPlaybackRate"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_numeric_state_value(
                    *node,
                    INTERNAL_MEDIA_DEFAULT_PLAYBACK_RATE_KEY,
                    1.0,
                ))
            }
            "textTracks"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_text_tracks_live_list_value(*node))
            }
            "buffered"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_time_ranges_live_value(*node, "buffered"))
            }
            "seekable"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_time_ranges_live_value(*node, "seekable"))
            }
            "played"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_time_ranges_live_value(*node, "played"))
            }
            "currentSrc" | "currentsrc"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("img")
                        || tag.eq_ignore_ascii_case("audio")
                        || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::String(self.resolve_media_src(*node)))
            }
            "complete"
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("img")) =>
            {
                Ok(Value::Bool(true))
            }
            "naturalWidth"
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("img")) =>
            {
                Ok(Value::Number(self.image_natural_dimension_value(*node)))
            }
            "naturalHeight"
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("img")) =>
            {
                Ok(Value::Number(self.image_natural_dimension_value(*node)))
            }
            "src" => Ok(Value::String(self.resolve_media_src(*node))),
            "poster" => Ok(Value::String(
                self.reflected_url_attribute_or_empty(*node, "poster"),
            )),
            "attributionSrc" | "attributionsrc" => Ok(Value::String(
                self.dom.attr(*node, "attributionsrc").unwrap_or_default(),
            )),
            "data" => {
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("object"))
                {
                    Ok(Value::String(
                        self.reflected_url_attribute_or_empty(*node, "data"),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "srcdoc" | "srcDoc" => {
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("iframe"))
                {
                    Ok(Value::String(
                        self.dom.attr(*node, "srcdoc").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "preload" => Ok(Value::String(
                self.dom.attr(*node, "preload").unwrap_or_default(),
            )),
            "sizes" => Ok(Value::String(
                self.dom.attr(*node, "sizes").unwrap_or_default(),
            )),
            "srcset" | "srcSet" => Ok(Value::String(
                self.dom.attr(*node, "srcset").unwrap_or_default(),
            )),
            "useMap" | "usemap" => {
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("img") || tag.eq_ignore_ascii_case("object")
                }) {
                    Ok(Value::String(
                        self.dom.attr(*node, "usemap").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "width" => Ok(Value::Number(self.canvas_dimension_value(*node, "width"))),
            "height" => Ok(Value::Number(self.canvas_dimension_value(*node, "height"))),
            "mozOpaque" | "mozopaque" => {
                if is_canvas {
                    Ok(Value::Bool(self.dom.attr(*node, "moz-opaque").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "mozPrintCallback" | "mozprintcallback" => {
                if is_canvas {
                    Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(*node, key.to_string()))
                        .cloned()
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "tagName" => Ok(Value::String(self.element_tag_name(*node))),
            "localName" => Ok(Value::String(
                self.dom
                    .tag_name(*node)
                    .map(|name| {
                        name.rsplit_once(':')
                            .map(|(_, local)| local)
                            .unwrap_or(name)
                            .to_ascii_lowercase()
                    })
                    .unwrap_or_default(),
            )),
            "namespaceURI" => Ok(self
                .dom
                .element(*node)
                .and_then(|element| element.namespace_uri.clone())
                .map(Value::String)
                .unwrap_or(Value::Null)),
            "prefix" => Ok(self
                .dom
                .tag_name(*node)
                .and_then(|name| name.split_once(':').map(|(prefix, _)| prefix))
                .map(|prefix| Value::String(prefix.to_string()))
                .unwrap_or(Value::Null)),
            "className" => Ok(Value::String(
                self.dom.attr(*node, "class").unwrap_or_default(),
            )),
            "classList" => Ok(self.class_list_live_value(*node)),
            "slot" => Ok(Value::String(
                self.dom.attr(*node, "slot").unwrap_or_default(),
            )),
            "role" => {
                let role = self.resolved_role_for_node(*node);
                if role.is_empty() {
                    Ok(Value::Null)
                } else if is_button {
                    Ok(Value::String("button".to_string()))
                } else {
                    Ok(Value::String(role))
                }
            }
            "baseURI" => Ok(Value::String(self.document_base_url())),
            "dataset" => Ok(self.dom_string_map_live_value(*node)),
            "open" => Ok(Value::Bool(self.dom.has_attr(*node, "open")?)),
            "closedBy" | "closedby" => Ok(Value::String(
                self.dom.attr(*node, "closedby").unwrap_or_default(),
            )),
            "htmlFor" => Ok(Value::String(
                self.dom.attr(*node, "for").unwrap_or_default(),
            )),
            "elementTiming" | "elementtiming" => Ok(Value::String(
                self.dom.attr(*node, "elementtiming").unwrap_or_default(),
            )),
            "options" => {
                if is_select {
                    return Ok(self.select_options_live_list_value(*node));
                }
                if is_datalist {
                    return Ok(self.datalist_options_live_list_value(*node));
                }
                Ok(Value::Undefined)
            }
            "selectedIndex" => {
                if !is_select {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(self.select_selected_index_value(*node)))
            }
            "selectedOptions" => {
                if !is_select {
                    return Ok(Value::Undefined);
                }
                Ok(self.selected_options_live_list_value(*node))
            }
            "size" => {
                if is_select {
                    return Ok(Value::Number(self.select_size_property_value(*node)));
                }
                if is_input {
                    return Ok(Value::Number(
                        self.input_size_property_value_for_node(*node),
                    ));
                }
                Ok(Value::Undefined)
            }
            "min" | "max" | "step" => {
                if !is_input {
                    return Ok(Value::Undefined);
                }
                Ok(Value::String(self.dom.attr(*node, key).unwrap_or_default()))
            }
            "maxLength" | "maxlength" => {
                if !(is_input || is_textarea) {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(
                    self.max_length_property_value_for_node(*node),
                ))
            }
            "minLength" | "minlength" => {
                if !(is_input || is_textarea) {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(
                    self.min_length_property_value_for_node(*node),
                ))
            }
            "rows" => {
                if !is_textarea {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(
                    self.textarea_rows_property_value_for_node(*node),
                ))
            }
            "cols" => {
                if !is_textarea {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(
                    self.textarea_cols_property_value_for_node(*node),
                ))
            }
            "validationMessage" => {
                let validity = self.compute_input_validity(*node)?;
                if validity.custom_error {
                    Ok(Value::String(self.dom.custom_validity_message(*node)?))
                } else {
                    Ok(Value::String(String::new()))
                }
            }
            "validity" => {
                let validity = self.compute_input_validity(*node)?;
                Ok(Self::input_validity_to_value(&validity))
            }
            "willValidate" => {
                let will_validate = if is_select {
                    self.select_will_validate(*node)
                } else if is_button {
                    self.button_will_validate(*node)
                } else if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("textarea"))
                {
                    !self.is_effectively_disabled(*node)
                } else if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    Self::input_participates_in_constraint_validation(
                        self.normalized_input_type(*node).as_str(),
                    ) && !self.is_effectively_disabled(*node)
                } else {
                    false
                };
                Ok(Value::Bool(will_validate))
            }
            "length" => {
                if is_form {
                    return Ok(Value::Number(self.form_elements(*node)?.len() as i64));
                }
                if !is_select {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(self.select_option_nodes(*node).len() as i64))
            }
            "captureStream"
            | "getContext"
            | "toDataURL"
            | "toBlob"
            | "transferControlToOffscreen" => {
                if !is_canvas {
                    return Ok(Value::Undefined);
                }
                Ok(self
                    .dom_runtime
                    .node_expando_props
                    .get(&(*node, key.to_string()))
                    .cloned()
                    .unwrap_or_else(Self::new_builtin_placeholder_function))
            }
            _ if key.starts_with("on") => {
                let is_body_window_alias = self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
                    && key
                        .strip_prefix("on")
                        .map(|event_type| event_type.to_ascii_lowercase())
                        .is_some_and(|event_type| {
                            Self::is_body_window_event_handler_alias(event_type.as_str())
                        });
                if is_body_window_alias {
                    Ok(
                        Self::object_get_entry(&self.dom_runtime.window_object.borrow(), key)
                            .unwrap_or(Value::Null),
                    )
                } else {
                    Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(*node, key.to_string()))
                        .cloned()
                        .unwrap_or(Value::Null))
                }
            }
            _ => Ok(self
                .dom_runtime
                .node_expando_props
                .get(&(*node, key.to_string()))
                .cloned()
                .or(if is_media {
                    self.html_media_builtin_property_value(*node, key)?
                } else {
                    None
                })
                .or(if is_form {
                    self.form_builtin_property_value(key)
                } else {
                    None
                })
                .or(if is_form {
                    self.form_named_property_value(*node, key)?
                } else {
                    None
                })
                .unwrap_or(Value::Undefined)),
        }
    }

    pub(crate) fn object_property_from_attr_or_class_list_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if Self::is_attr_object(entries) {
            let value = match key {
                "ownerElement" => {
                    Self::object_get_entry(entries, "ownerElement").unwrap_or(Value::Null)
                }
                "name" => Self::object_get_entry(entries, "name")
                    .unwrap_or_else(|| Value::String(String::new())),
                "value" => Self::object_get_entry(entries, "value")
                    .unwrap_or_else(|| Value::String(String::new())),
                "nodeType" => Value::Number(2),
                "nodeName" => Self::object_get_entry(entries, "name")
                    .unwrap_or_else(|| Value::String(String::new())),
                "nodeValue" => Self::object_get_entry(entries, "value")
                    .unwrap_or_else(|| Value::String(String::new())),
                "parentNode" | "parentElement" | "previousSibling" | "nextSibling" => Value::Null,
                _ => Value::Undefined,
            };
            if !matches!(value, Value::Undefined) {
                return Some(value);
            }
        }

        if Self::is_dom_string_map_object(entries) {
            let Some(node) = Self::dom_string_map_owner_node(entries) else {
                return None;
            };
            if self.dom.element(node).is_none() {
                return None;
            }
            if Self::is_symbol_storage_key(key) {
                return Some(Self::object_get_entry(entries, key).unwrap_or(Value::Undefined));
            }
            if self.is_to_string_tag_property_key(key) {
                return Some(Value::String("DOMStringMap".to_string()));
            }
            let attr_name = dataset_key_to_attr_name(key);
            return self.dom.attr(node, &attr_name).map(Value::String);
        }

        if Self::is_class_list_object(entries) {
            let Some(node) = (match Self::object_get_entry(entries, INTERNAL_CLASS_LIST_NODE_KEY) {
                Some(Value::Node(node)) => Some(node),
                _ => None,
            }) else {
                return None;
            };
            let classes = class_tokens(self.dom.attr(node, "class").as_deref());
            let has_explicit_prototype =
                Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
            let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
            if key == "length" {
                return Some(Value::Number(classes.len() as i64));
            }
            if key == "value" {
                return Some(Value::String(classes.join(" ")));
            }
            if key_is_to_string_tag {
                return (!has_explicit_prototype)
                    .then_some(Value::String("DOMTokenList".to_string()));
            }
            if !has_explicit_prototype {
                if let Some(kind) = match key {
                    "add" => Some("class_list_add"),
                    "remove" => Some("class_list_remove"),
                    "toggle" => Some("class_list_toggle"),
                    "contains" => Some("class_list_contains"),
                    "replace" => Some("class_list_replace"),
                    "item" => Some("class_list_item"),
                    "forEach" => Some("class_list_for_each"),
                    "keys" => Some("class_list_keys"),
                    "values" => Some("class_list_values"),
                    "entries" => Some("class_list_entries"),
                    "toString" => Some("class_list_to_string"),
                    _ if self.is_iterator_property_key(key) => Some("class_list_values"),
                    _ => None,
                } {
                    return Some(Self::new_class_list_method_callable(kind));
                }
            }
            if let Ok(index) = key.parse::<usize>() {
                return classes.get(index).cloned().map(Value::String);
            }
            if let Some(value) = Self::object_get_entry(entries, key) {
                return Some(value);
            }
            return None;
        }

        None
    }

    pub(crate) fn object_property_from_web_api_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Result<Option<Value>> {
        if Self::is_event_target_object(entries)
            && let Some(value) =
                Self::placeholder_backed_object_builtin_property_value(entries, key)
        {
            return Ok(Some(value));
        }
        if let Some(value) = Self::object_property_from_event_entries(entries, key) {
            return Ok(Some(value));
        }
        if let Some(value) = Self::object_property_from_data_transfer_entries(entries, key) {
            return Ok(Some(value));
        }
        if let Some(value) = Self::object_property_from_range_or_selection_entries(entries, key) {
            return Ok(Some(value));
        }
        if let Some(value) = Self::object_property_from_cookie_store_or_cache_entries(entries, key)
        {
            return Ok(Some(value));
        }
        if let Some(value) = self.computed_style_object_property_from_entries(entries, key)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.fetch_response_property_from_entries(entries, key) {
            return Ok(Some(value));
        }
        if let Some(value) = self.fetch_request_property_from_entries(entries, key) {
            return Ok(Some(value));
        }
        if let Some(value) = self.headers_property_from_entries(entries, key) {
            return Ok(Some(value));
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_DOM_PARSER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            if let Some(value) = self.dom_parser_object_property(entries, key) {
                return Ok(Some(value));
            }
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_XML_SERIALIZER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            if let Some(value) = self.xml_serializer_object_property(entries, key) {
                return Ok(Some(value));
            }
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_PARSED_DOCUMENT_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            if let Some(value) = self.parsed_document_property_from_entries(entries, key)? {
                return Ok(Some(value));
            }
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_TREE_WALKER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            if let Some(value) = self.tree_walker_property_from_entries(entries, key)? {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    fn object_property_from_event_entries(entries: &ObjectValue, key: &str) -> Option<Value> {
        if Self::is_event_object(entries)
            || Self::is_keyboard_event_object(entries)
            || Self::is_pointer_event_object(entries)
            || Self::is_navigate_event_object(entries)
        {
            return Self::placeholder_backed_object_builtin_property_value(entries, key);
        }
        None
    }

    fn object_property_from_data_transfer_entries(
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if Self::is_data_transfer_object(entries)
            || Self::is_clipboard_data_object(entries)
            || Self::is_data_transfer_item_object(entries)
        {
            return Self::placeholder_backed_object_builtin_property_value(entries, key);
        }
        None
    }

    fn object_property_from_match_media_entries(
        &mut self,
        receiver: &Value,
        entries: &ObjectValue,
        key: &str,
        key_is_to_string_tag: bool,
    ) -> Result<Option<Value>> {
        if !Self::is_match_media_object(entries) {
            return Ok(None);
        }
        if matches!(key, "matches" | "media")
            && let Some(value) =
                self.object_property_from_entries_with_getter(receiver, entries, key)?
        {
            return Ok(Some(value));
        }
        let query = Self::object_get_entry(entries, INTERNAL_MATCH_MEDIA_QUERY_KEY)
            .map(|value| value.as_string())
            .unwrap_or_default();
        if key == "matches" {
            let matches = self
                .platform_mocks
                .match_media_mocks
                .get(&query)
                .copied()
                .unwrap_or(self.platform_mocks.default_match_media_matches);
            return Ok(Some(Value::Bool(matches)));
        }
        if key == "media" {
            return Ok(Some(Value::String(query)));
        }
        if let Some(value) = Self::placeholder_backed_object_builtin_property_value(entries, key) {
            return Ok(Some(value));
        }
        if key_is_to_string_tag {
            return Ok(Some(Value::String("MediaQueryList".to_string())));
        }
        Ok(None)
    }

    fn object_property_from_named_node_map_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_named_node_map_object(entries) {
            return None;
        }
        let has_explicit_prototype =
            Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
        if self.is_to_string_tag_property_key(key) {
            return (!has_explicit_prototype).then_some(Value::String("NamedNodeMap".to_string()));
        }
        if !has_explicit_prototype {
            if let Some(kind) = match key {
                "item" => Some("named_node_map_item"),
                "getNamedItem" => Some("named_node_map_get_named_item"),
                "setNamedItem" => Some("named_node_map_set_named_item"),
                "removeNamedItem" => Some("named_node_map_remove_named_item"),
                "getNamedItemNS" => Some("named_node_map_get_named_item_ns"),
                "setNamedItemNS" => Some("named_node_map_set_named_item_ns"),
                "removeNamedItemNS" => Some("named_node_map_remove_named_item_ns"),
                "forEach" => Some("named_node_map_for_each"),
                "keys" => Some("named_node_map_keys"),
                "values" => Some("named_node_map_values"),
                "entries" => Some("named_node_map_entries"),
                _ if self.is_iterator_property_key(key) => Some("named_node_map_values"),
                _ => None,
            } {
                return Some(Self::new_named_node_map_method_callable(kind));
            }
        }
        let owner = Self::named_node_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some());
        let attrs = owner
            .map(|owner_node| self.named_node_map_entries(owner_node))
            .unwrap_or_default();
        if key == "length" {
            return Some(Value::Number(attrs.len() as i64));
        }
        if let Ok(index) = key.parse::<usize>() {
            return attrs.get(index).and_then(|(name, value)| {
                owner.map(|owner_node| Self::new_attr_object_value(name, value, Some(owner_node)))
            });
        }
        if !self.named_node_map_named_property_is_visible(entries, key) {
            return None;
        }
        if let Some(owner_node) = owner {
            if let Some((name, value)) = attrs.iter().find(|(name, _)| name == key) {
                return Some(Self::new_attr_object_value(name, value, Some(owner_node)));
            }
        }
        None
    }

    fn object_property_from_string_wrapper_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        let text = Self::string_wrapper_value_from_object(entries)?;
        if key == "length" {
            return Some(Value::Number(text.chars().count() as i64));
        }
        let is_url_like = Self::is_url_object(entries) || Self::is_location_object(entries);
        if key == "constructor" && !is_url_like {
            return Some(Value::StringConstructor);
        }
        if !is_url_like {
            if self.is_iterator_property_key(key) {
                return Some(Self::new_receiver_builtin_callable("string", "iterator"));
            }
            if matches!(key, "toString" | "valueOf") || Self::is_string_method_name(key) {
                return Some(Self::new_receiver_builtin_callable("string", key));
            }
        }
        if let Ok(index) = key.parse::<usize>() {
            return text
                .chars()
                .nth(index)
                .map(|ch| Value::String(ch.to_string()));
        }
        None
    }

    pub(crate) fn object_property_from_match_media_named_node_map_or_string_wrapper_entries(
        &mut self,
        receiver: &Value,
        entries: &ObjectValue,
        key: &str,
    ) -> Result<Option<Value>> {
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        if let Some(value) = self.object_property_from_match_media_entries(
            receiver,
            entries,
            key,
            key_is_to_string_tag,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) = self.object_property_from_named_node_map_entries(entries, key) {
            return Ok(Some(value));
        }
        Ok(self.object_property_from_string_wrapper_entries(entries, key))
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
            Value::MapConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_map_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::WeakMapConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_weak_map_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::SetConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_set_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::WeakSetConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_weak_set_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::TypedArrayConstructor(kind) => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => {
                        self.cached_typed_array_constructor_prototype_value(kind.clone())
                    }
                    "from" if matches!(kind, TypedArrayConstructorKind::Concrete(_)) => {
                        self.cached_typed_array_static_method_value(kind.clone(), "from")
                    }
                    "of" if matches!(kind, TypedArrayConstructorKind::Concrete(_)) => {
                        self.cached_typed_array_static_method_value(kind.clone(), "of")
                    }
                    "BYTES_PER_ELEMENT" => match kind {
                        TypedArrayConstructorKind::Concrete(kind) => {
                            Value::Number(kind.bytes_per_element() as i64)
                        }
                        TypedArrayConstructorKind::Abstract => Value::Undefined,
                    },
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::BlobConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_blob_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::RegExpConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_regexp_constructor_prototype_value(),
                    "escape" => self.cached_regexp_static_method_value("escape"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::UrlConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = if key == "prototype" {
                    self.cached_url_constructor_prototype_value()
                } else if let Some(value) = Self::object_get_entry(
                    &self.browser_apis.url_constructor_properties.borrow(),
                    key,
                ) {
                    value
                } else if Self::is_url_static_method_name(key) {
                    Self::new_builtin_placeholder_function()
                } else {
                    Value::Undefined
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::ArrayBufferConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_array_buffer_constructor_prototype_value(),
                    "isView" => self.cached_array_buffer_static_method_value("isView"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::PromiseConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_promise_constructor_prototype_value(),
                    "resolve" => self.cached_promise_static_method_value("resolve"),
                    "reject" => self.cached_promise_static_method_value("reject"),
                    "all" => self.cached_promise_static_method_value("all"),
                    "allSettled" => self.cached_promise_static_method_value("allSettled"),
                    "any" => self.cached_promise_static_method_value("any"),
                    "race" => self.cached_promise_static_method_value("race"),
                    "try" => self.cached_promise_static_method_value("try"),
                    "withResolvers" => self.cached_promise_static_method_value("withResolvers"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::UrlSearchParamsConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_url_search_params_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::StringConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_string_constructor_prototype_value(),
                    "fromCharCode" => self.cached_string_static_method_value("fromCharCode"),
                    "fromCodePoint" => self.cached_string_static_method_value("fromCodePoint"),
                    "raw" => self.cached_string_static_method_value("raw"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::SymbolConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_symbol_constructor_prototype_value(),
                    "for" => self.cached_symbol_static_method_value("for"),
                    "keyFor" => self.cached_symbol_static_method_value("keyFor"),
                    "asyncDispose" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::AsyncDispose)
                    }
                    "asyncIterator" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::AsyncIterator)
                    }
                    "dispose" => self.eval_symbol_static_property(SymbolStaticProperty::Dispose),
                    "hasInstance" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::HasInstance)
                    }
                    "isConcatSpreadable" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::IsConcatSpreadable)
                    }
                    "iterator" => self.eval_symbol_static_property(SymbolStaticProperty::Iterator),
                    "match" => self.eval_symbol_static_property(SymbolStaticProperty::Match),
                    "matchAll" => self.eval_symbol_static_property(SymbolStaticProperty::MatchAll),
                    "replace" => self.eval_symbol_static_property(SymbolStaticProperty::Replace),
                    "search" => self.eval_symbol_static_property(SymbolStaticProperty::Search),
                    "species" => self.eval_symbol_static_property(SymbolStaticProperty::Species),
                    "split" => self.eval_symbol_static_property(SymbolStaticProperty::Split),
                    "toPrimitive" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::ToPrimitive)
                    }
                    "toStringTag" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag)
                    }
                    "unscopables" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::Unscopables)
                    }
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            _ => Err(Error::ScriptRuntime("value is not an object".into())),
        }
    }

    pub(crate) fn object_synthesized_own_property_exists(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> bool {
        if Self::is_class_list_object(entries) {
            let has_explicit_prototype =
                Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
            if key == "length" || key == "value" {
                return true;
            }
            if Self::is_class_list_method_name(key) || self.is_iterator_property_key(key) {
                return !has_explicit_prototype;
            }
        }
        if Self::is_named_node_map_object(entries) {
            let has_explicit_prototype =
                Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
            if key == "length" {
                return true;
            }
            if Self::is_named_node_map_method_name(key) || self.is_iterator_property_key(key) {
                return !has_explicit_prototype;
            }
        }
        self.object_property_from_attr_or_class_list_entries(entries, key)
            .is_some()
            || self
                .object_property_from_named_node_map_entries(entries, key)
                .is_some()
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
