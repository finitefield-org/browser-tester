use super::*;

impl Harness {
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
}
