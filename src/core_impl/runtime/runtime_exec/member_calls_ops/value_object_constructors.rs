use super::*;

impl Harness {
    pub(crate) fn shared_core_constructor_bindings(
        string_constructor: &Value,
        boolean_constructor: &Value,
        number_constructor: &Value,
        bigint_constructor: &Value,
        symbol_constructor: &Value,
        object_constructor: &Value,
        reflect_object: &Value,
    ) -> Vec<(String, Value)> {
        let object_prototype = match object_constructor {
            Value::Object(entries) => {
                let entries = entries.borrow();
                Self::object_get_entry(&entries, "prototype")
            }
            _ => None,
        };
        if let Some(object_prototype) = object_prototype {
            for constructor in [boolean_constructor, number_constructor, bigint_constructor] {
                let Value::Object(entries) = constructor else {
                    continue;
                };
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype_entries)) = prototype {
                    Self::set_internal_prototype(&prototype_entries, object_prototype.clone());
                }
            }
        }

        let mut bindings = vec![
            ("String".to_string(), string_constructor.clone()),
            ("Boolean".to_string(), boolean_constructor.clone()),
            ("Number".to_string(), number_constructor.clone()),
            ("BigInt".to_string(), bigint_constructor.clone()),
            ("Symbol".to_string(), symbol_constructor.clone()),
            ("RegExp".to_string(), Value::RegExpConstructor),
            ("Object".to_string(), object_constructor.clone()),
            ("Reflect".to_string(), reflect_object.clone()),
            ("Blob".to_string(), Value::BlobConstructor),
            ("URL".to_string(), Value::UrlConstructor),
            (
                "URLSearchParams".to_string(),
                Value::UrlSearchParamsConstructor,
            ),
            ("ArrayBuffer".to_string(), Value::ArrayBufferConstructor),
            ("Promise".to_string(), Value::PromiseConstructor),
            ("Map".to_string(), Value::MapConstructor),
            ("WeakMap".to_string(), Value::WeakMapConstructor),
            ("Set".to_string(), Value::SetConstructor),
            ("WeakSet".to_string(), Value::WeakSetConstructor),
        ];
        for kind in TypedArrayKind::concrete_kinds() {
            bindings.push((
                kind.name().to_string(),
                Value::TypedArrayConstructor(TypedArrayConstructorKind::Concrete(*kind)),
            ));
        }
        bindings
    }

    pub(crate) fn new_boolean_constructor_callable() -> Value {
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("boolean_constructor"),
            "boolean",
            &["toString", "valueOf"],
        );
        if let Value::Object(entries) = &constructor {
            Self::mark_existing_public_properties_non_enumerable(entries);
        }
        constructor
    }

    pub(crate) fn new_number_constructor_callable() -> Value {
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("number_constructor"),
            "number",
            &[
                "toExponential",
                "toFixed",
                "toLocaleString",
                "toPrecision",
                "toString",
                "valueOf",
            ],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let mut entries = constructor_entries.borrow_mut();
        for (key, value) in [
            (
                "isFinite",
                Self::new_number_static_method_callable("isFinite"),
            ),
            (
                "isInteger",
                Self::new_number_static_method_callable("isInteger"),
            ),
            ("isNaN", Self::new_number_static_method_callable("isNaN")),
            (
                "isSafeInteger",
                Self::new_number_static_method_callable("isSafeInteger"),
            ),
            (
                "parseFloat",
                Self::new_number_static_method_callable("parseFloat"),
            ),
            (
                "parseInt",
                Self::new_number_static_method_callable("parseInt"),
            ),
        ] {
            Self::object_set_entry(&mut entries, key.to_string(), value);
        }
        for (key, value) in [
            ("EPSILON", Value::Float(f64::EPSILON)),
            ("MAX_SAFE_INTEGER", Value::Number(9_007_199_254_740_991)),
            ("MAX_VALUE", Value::Float(f64::MAX)),
            ("MIN_SAFE_INTEGER", Value::Number(-9_007_199_254_740_991)),
            ("MIN_VALUE", Value::Float(f64::from_bits(1))),
            ("NaN", Value::Float(f64::NAN)),
            ("NEGATIVE_INFINITY", Value::Float(f64::NEG_INFINITY)),
            ("POSITIVE_INFINITY", Value::Float(f64::INFINITY)),
        ] {
            Self::object_set_entry(&mut entries, key.to_string(), value);
        }
        drop(entries);
        Self::mark_existing_public_properties_non_enumerable(constructor_entries);
        constructor
    }

    pub(crate) fn new_bigint_constructor_callable() -> Value {
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("bigint_constructor"),
            "bigint",
            &["toLocaleString", "toString", "valueOf"],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let mut entries = constructor_entries.borrow_mut();
        for (key, value) in [
            ("asIntN", Self::new_bigint_static_method_callable("asIntN")),
            (
                "asUintN",
                Self::new_bigint_static_method_callable("asUintN"),
            ),
        ] {
            Self::object_set_entry(&mut entries, key.to_string(), value);
        }
        drop(entries);
        Self::mark_existing_public_properties_non_enumerable(constructor_entries);
        constructor
    }

    pub(crate) fn new_object_constructor_value() -> Value {
        let prototype = Self::new_object_value(vec![
            (
                "toString".to_string(),
                Self::new_receiver_builtin_callable("object", "toString"),
            ),
            (
                "valueOf".to_string(),
                Self::new_receiver_builtin_callable("object", "valueOf"),
            ),
            (
                "hasOwnProperty".to_string(),
                Self::new_receiver_builtin_callable("object", "hasOwnProperty"),
            ),
            (
                "isPrototypeOf".to_string(),
                Self::new_receiver_builtin_callable("object", "isPrototypeOf"),
            ),
            (
                "propertyIsEnumerable".to_string(),
                Self::new_receiver_builtin_callable("object", "propertyIsEnumerable"),
            ),
        ]);
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("object_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
            (
                "create".to_string(),
                Self::new_object_static_method_callable("create"),
            ),
            (
                "assign".to_string(),
                Self::new_object_static_method_callable("assign"),
            ),
            (
                "getOwnPropertyDescriptor".to_string(),
                Self::new_object_static_method_callable("getOwnPropertyDescriptor"),
            ),
            (
                "defineProperty".to_string(),
                Self::new_object_static_method_callable("defineProperty"),
            ),
            (
                "getOwnPropertyNames".to_string(),
                Self::new_object_static_method_callable("getOwnPropertyNames"),
            ),
            (
                "getOwnPropertySymbols".to_string(),
                Self::new_object_static_method_callable("getOwnPropertySymbols"),
            ),
            (
                "keys".to_string(),
                Self::new_object_static_method_callable("keys"),
            ),
            (
                "values".to_string(),
                Self::new_object_static_method_callable("values"),
            ),
            (
                "entries".to_string(),
                Self::new_object_static_method_callable("entries"),
            ),
            (
                "fromEntries".to_string(),
                Self::new_object_static_method_callable("fromEntries"),
            ),
            (
                "hasOwn".to_string(),
                Self::new_object_static_method_callable("hasOwn"),
            ),
            (
                "getPrototypeOf".to_string(),
                Self::new_object_static_method_callable("getPrototypeOf"),
            ),
            (
                "setPrototypeOf".to_string(),
                Self::new_object_static_method_callable("setPrototypeOf"),
            ),
            (
                "freeze".to_string(),
                Self::new_object_static_method_callable("freeze"),
            ),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            let mut prototype_entries = prototype_entries.borrow_mut();
            Self::object_set_entry(
                &mut prototype_entries,
                "constructor".to_string(),
                constructor.clone(),
            );
            Self::object_set_entry(
                &mut prototype_entries,
                INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                Value::Null,
            );
        }
        if let Value::Object(prototype_entries) = &prototype {
            Self::mark_existing_public_properties_non_enumerable(prototype_entries);
        }
        if let Value::Object(constructor_entries) = &constructor {
            Self::mark_existing_public_properties_non_enumerable(constructor_entries);
        }
        constructor
    }

    pub(crate) fn new_reflect_object_value(&mut self) -> Value {
        let to_string_tag = self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag);
        let reflect = Self::new_object_value(vec![
            (
                "set".to_string(),
                Self::new_reflect_static_method_callable("set"),
            ),
            (
                "ownKeys".to_string(),
                Self::new_reflect_static_method_callable("ownKeys"),
            ),
            (to_string_tag_key, Value::String("Reflect".to_string())),
        ]);
        if let Value::Object(entries) = &reflect {
            Self::mark_existing_public_properties_non_enumerable(entries);
        }
        reflect
    }

    pub(crate) fn new_event_target_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("event_target_constructor", vec![])
    }

    pub(crate) fn new_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("event_constructor", vec![])
    }

    pub(crate) fn new_custom_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("custom_event_constructor", vec![])
    }

    pub(crate) fn new_mouse_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("mouse_event_constructor", vec![])
    }

    pub(crate) fn new_keyboard_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype(
            "keyboard_event_constructor",
            vec![
                ("DOM_KEY_LOCATION_STANDARD".to_string(), Value::Number(0x00)),
                ("DOM_KEY_LOCATION_LEFT".to_string(), Value::Number(0x01)),
                ("DOM_KEY_LOCATION_RIGHT".to_string(), Value::Number(0x02)),
                ("DOM_KEY_LOCATION_NUMPAD".to_string(), Value::Number(0x03)),
            ],
        )
    }

    pub(crate) fn new_wheel_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype(
            "wheel_event_constructor",
            vec![
                ("DOM_DELTA_PIXEL".to_string(), Value::Number(0)),
                ("DOM_DELTA_LINE".to_string(), Value::Number(1)),
                ("DOM_DELTA_PAGE".to_string(), Value::Number(2)),
            ],
        )
    }

    pub(crate) fn new_navigate_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("navigate_event_constructor", vec![])
    }

    pub(crate) fn new_pointer_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("pointer_event_constructor", vec![])
    }

    pub(crate) fn new_error_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("error_event_constructor", vec![])
    }

    pub(crate) fn new_hash_change_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("hash_change_event_constructor", vec![])
    }

    pub(crate) fn new_before_unload_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype(
            "before_unload_event_constructor",
            vec![],
        )
    }

    pub(crate) fn new_image_data_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("image_data_constructor", vec![])
    }

    pub(crate) fn new_navigate_event_default_signal_value() -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_EVENT_TARGET_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("aborted".to_string(), Value::Bool(false)),
            ("onabort".to_string(), Value::Null),
            (
                "addEventListener".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "removeEventListener".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "dispatchEvent".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ])
    }

    pub(crate) fn new_dom_parser_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("dom_parser_constructor".to_string()),
        )])
    }

    pub(crate) fn new_xml_serializer_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("xml_serializer_constructor".to_string()),
        )])
    }

    pub(crate) fn new_document_parse_html_callable(sanitize: bool) -> Value {
        let kind = if sanitize {
            "document_parse_html"
        } else {
            "document_parse_html_unsafe"
        };
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String(kind.to_string()),
        )])
    }

    pub(crate) fn new_document_constructor_value() -> Value {
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("document_constructor".to_string()),
            ),
            (
                "parseHTML".to_string(),
                Self::new_document_parse_html_callable(true),
            ),
            (
                "parseHTMLUnsafe".to_string(),
                Self::new_document_parse_html_callable(false),
            ),
        ]);
        if let Value::Object(entries) = &constructor {
            Self::mark_existing_public_properties_non_enumerable(entries);
        }
        constructor
    }

    pub(crate) fn new_fetch_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("fetch_function".to_string()),
        )])
    }

    pub(crate) fn new_match_media_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("match_media_function".to_string()),
        )])
    }

    pub(crate) fn new_window_close_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_close_function".to_string()),
        )])
    }

    pub(crate) fn new_window_open_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_open_function".to_string()),
        )])
    }

    pub(crate) fn new_window_stop_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_stop_function".to_string()),
        )])
    }

    pub(crate) fn new_window_focus_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_focus_function".to_string()),
        )])
    }

    pub(crate) fn new_window_scroll_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_scroll_function".to_string()),
        )])
    }

    pub(crate) fn new_window_scroll_by_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_scroll_by_function".to_string()),
        )])
    }

    pub(crate) fn new_window_scroll_to_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_scroll_to_function".to_string()),
        )])
    }

    pub(crate) fn new_window_move_by_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_move_by_function".to_string()),
        )])
    }

    pub(crate) fn new_window_move_to_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_move_to_function".to_string()),
        )])
    }

    pub(crate) fn new_window_resize_by_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_resize_by_function".to_string()),
        )])
    }

    pub(crate) fn new_window_resize_to_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_resize_to_function".to_string()),
        )])
    }

    pub(crate) fn new_window_post_message_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_post_message_function".to_string()),
        )])
    }

    pub(crate) fn new_window_get_computed_style_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_get_computed_style_function".to_string()),
        )])
    }

    pub(crate) fn new_window_alert_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_alert_function".to_string()),
        )])
    }

    pub(crate) fn new_window_confirm_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_confirm_function".to_string()),
        )])
    }

    pub(crate) fn new_window_print_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_print_function".to_string()),
        )])
    }

    pub(crate) fn new_window_report_error_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_report_error_function".to_string()),
        )])
    }

    pub(crate) fn new_window_prompt_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_prompt_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_window_close_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_window_close_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_window_focus_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_window_focus_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_window_print_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_window_print_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_document_open_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_document_open_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_document_write_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_document_write_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_document_close_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_document_close_function".to_string()),
        )])
    }

    pub(crate) fn new_request_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("request_constructor".to_string()),
        )])
    }

    pub(crate) fn new_file_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("file_constructor", vec![])
    }

    pub(crate) fn new_clipboard_item_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("clipboard_item_constructor".to_string()),
        )])
    }

    pub(crate) fn new_clipboard_write_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("clipboard_write".to_string()),
        )])
    }

    pub(crate) fn new_headers_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("headers_constructor".to_string()),
        )])
    }

    pub(crate) fn new_worker_constructor_value(&mut self) -> Value {
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("worker_constructor"),
            "worker",
            &["postMessage", "terminate"],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String("Worker".to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        constructor
    }

    pub(crate) fn new_data_transfer_constructor_value(&mut self) -> Value {
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("data_transfer_constructor"),
            "data_transfer",
            &[
                "getData",
                "setData",
                "clearData",
                "setDragImage",
                "addElement",
            ],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String("DataTransfer".to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        constructor
    }

    pub(crate) fn new_option_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("option_constructor".to_string()),
        )])
    }

    pub(crate) fn new_audio_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("audio_constructor".to_string()),
        )])
    }

    pub(crate) fn new_css_style_sheet_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("css_style_sheet_constructor", vec![])
    }
}
