use super::*;

impl Harness {
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
}
