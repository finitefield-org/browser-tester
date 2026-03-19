use super::*;

#[path = "environment_global_script_env.rs"]
mod environment_global_script_env;
#[path = "environment_global_window_entries.rs"]
mod environment_global_window_entries;

impl Harness {
    pub(crate) fn initialize_global_bindings(&mut self) {
        self.sync_location_object();
        self.sync_history_object();
        self.sync_navigation_object();
        self.dom_runtime.window_object = Rc::new(RefCell::new(ObjectValue::default()));
        self.dom_runtime.document_object = Rc::new(RefCell::new(ObjectValue::default()));
        self.dom_runtime.selection_object = match Self::new_selection_object_value(self.dom.root) {
            Value::Object(selection) => selection,
            _ => Rc::new(RefCell::new(ObjectValue::default())),
        };
        self.dom_runtime.live_form_elements_lists.clear();
        self.dom_runtime.live_select_options_lists.clear();
        self.dom_runtime.live_selected_options_lists.clear();
        self.dom_runtime.live_datalist_options_lists.clear();
        self.dom_runtime.live_media_text_tracks_lists.clear();
        self.dom_runtime.live_media_time_ranges_objects.clear();
        self.dom_runtime.live_document_forms_list = None;
        self.dom_runtime.live_document_images_list = None;
        self.dom_runtime.live_document_links_list = None;
        self.dom_runtime.live_document_scripts_list = None;
        self.browser_apis
            .url_constructor_properties
            .borrow_mut()
            .clear();
        self.sync_cookie_store_object();
        self.sync_cache_storage_object();
        let local_storage_items = {
            let entries = self.browser_apis.local_storage_object.borrow();
            if Self::is_storage_object(&entries) {
                Self::storage_pairs_from_object_entries(&entries)
            } else {
                Vec::new()
            }
        };
        let mut local_storage_entries =
            vec![(INTERNAL_STORAGE_OBJECT_KEY.to_string(), Value::Bool(true))];
        Self::set_storage_pairs(&mut local_storage_entries, &local_storage_items);
        if let Value::Object(prototype) = self.cached_storage_constructor_prototype_value() {
            Self::object_set_entry(
                &mut local_storage_entries,
                INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                Value::Object(prototype),
            );
        }
        *self.browser_apis.local_storage_object.borrow_mut() = local_storage_entries.into();
        let read_text = Self::new_builtin_placeholder_function();
        let write_text = Self::new_builtin_placeholder_function();
        let write = Self::new_clipboard_write_callable_value();
        let clipboard = Self::new_object_value(vec![
            (INTERNAL_CLIPBOARD_OBJECT_KEY.into(), Value::Bool(true)),
            (
                INTERNAL_CLIPBOARD_READ_TEXT_DEFAULT_KEY.into(),
                read_text.clone(),
            ),
            (
                INTERNAL_CLIPBOARD_WRITE_TEXT_DEFAULT_KEY.into(),
                write_text.clone(),
            ),
            ("readText".into(), read_text),
            ("writeText".into(), write_text),
            ("write".into(), write),
        ]);
        let location = Value::Object(self.dom_runtime.location_object.clone());
        let history = Value::Object(self.location_history.history_object.clone());
        let navigation = Value::Object(self.location_history.navigation_object.clone());
        let css = Self::new_object_value(vec![(
            "escape".into(),
            Self::new_global_css_escape_callable(),
        )]);

        let navigator = Self::new_object_value(vec![
            (INTERNAL_NAVIGATOR_OBJECT_KEY.into(), Value::Bool(true)),
            ("language".into(), Value::String(DEFAULT_LOCALE.to_string())),
            (
                "languages".into(),
                Self::new_array_value(vec![
                    Value::String(DEFAULT_LOCALE.to_string()),
                    Value::String("en".to_string()),
                ]),
            ),
            ("clipboard".into(), clipboard),
        ]);

        let mut intl_entries = vec![
            ("Collator".into(), Self::new_builtin_placeholder_function()),
            (
                "DateTimeFormat".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "DisplayNames".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "DurationFormat".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "ListFormat".into(),
                Self::new_builtin_placeholder_function(),
            ),
            ("Locale".into(), Self::new_builtin_placeholder_function()),
            (
                "NumberFormat".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "PluralRules".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "RelativeTimeFormat".into(),
                Self::new_builtin_placeholder_function(),
            ),
            ("Segmenter".into(), Self::new_builtin_placeholder_function()),
            (
                "getCanonicalLocales".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "supportedValuesOf".into(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        let to_string_tag = self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag);
        Self::object_set_entry(
            &mut intl_entries,
            to_string_tag_key.clone(),
            Value::String("Intl".to_string()),
        );
        let intl = Self::new_object_value(intl_entries);
        if let Value::Object(intl_entries) = &intl {
            for (constructor_name, tag_name) in [
                ("Collator", "Intl.Collator"),
                ("DateTimeFormat", "Intl.DateTimeFormat"),
                ("DisplayNames", "Intl.DisplayNames"),
                ("DurationFormat", "Intl.DurationFormat"),
                ("ListFormat", "Intl.ListFormat"),
                ("Locale", "Intl.Locale"),
                ("NumberFormat", "Intl.NumberFormat"),
                ("PluralRules", "Intl.PluralRules"),
                ("RelativeTimeFormat", "Intl.RelativeTimeFormat"),
                ("Segmenter", "Intl.Segmenter"),
            ] {
                let constructor = {
                    let entries = intl_entries.borrow();
                    Self::object_get_entry(&entries, constructor_name)
                };
                let Some(Value::Function(constructor_fn)) = constructor else {
                    continue;
                };
                let mut prototype = constructor_fn.prototype_object.borrow_mut();
                Self::object_set_entry(
                    &mut prototype,
                    "constructor".to_string(),
                    Value::Function(constructor_fn.clone()),
                );
                Self::object_set_entry(
                    &mut prototype,
                    Self::object_non_enumerable_storage_key("constructor"),
                    Value::Bool(true),
                );
                Self::object_set_entry(
                    &mut prototype,
                    to_string_tag_key.clone(),
                    Value::String(tag_name.to_string()),
                );
                let receiver_builtin_methods: Option<(&str, &[&str])> = match constructor_name {
                    "Collator" => Some(("intl_collator", &["resolvedOptions"])),
                    "DateTimeFormat" => Some((
                        "intl_date_time_format",
                        &[
                            "formatToParts",
                            "formatRange",
                            "formatRangeToParts",
                            "resolvedOptions",
                        ],
                    )),
                    "DisplayNames" => Some(("intl_display_names", &["of", "resolvedOptions"])),
                    "DurationFormat" => Some((
                        "intl_duration_format",
                        &["formatToParts", "resolvedOptions"],
                    )),
                    "ListFormat" => {
                        Some(("intl_list_format", &["formatToParts", "resolvedOptions"]))
                    }
                    "Locale" => Some((
                        "intl_locale",
                        &[
                            "getCalendars",
                            "getCollations",
                            "getHourCycles",
                            "getNumberingSystems",
                            "getTextInfo",
                            "getTimeZones",
                            "getWeekInfo",
                            "maximize",
                            "minimize",
                            "toString",
                        ],
                    )),
                    "NumberFormat" => Some((
                        "intl_number_format",
                        &[
                            "formatToParts",
                            "formatRange",
                            "formatRangeToParts",
                            "resolvedOptions",
                        ],
                    )),
                    "PluralRules" => Some((
                        "intl_plural_rules",
                        &["select", "selectRange", "resolvedOptions"],
                    )),
                    "RelativeTimeFormat" => Some((
                        "intl_relative_time_format",
                        &["format", "formatToParts", "resolvedOptions"],
                    )),
                    "Segmenter" => Some(("intl_segmenter", &["segment", "resolvedOptions"])),
                    _ => None,
                };
                if let Some((family, methods)) = receiver_builtin_methods {
                    for method in methods {
                        Self::object_set_entry(
                            &mut prototype,
                            (*method).to_string(),
                            Self::new_receiver_builtin_callable(family, method),
                        );
                        Self::object_set_entry(
                            &mut prototype,
                            Self::object_non_enumerable_storage_key(method),
                            Value::Bool(true),
                        );
                    }
                }
                let bound_getter_accessors: Vec<(&str, Value)> = match constructor_name {
                    "Collator" => {
                        vec![("compare", Self::new_intl_collator_compare_getter_callable())]
                    }
                    "DateTimeFormat" => {
                        vec![("format", Self::new_intl_date_time_format_getter_callable())]
                    }
                    "NumberFormat" => {
                        vec![("format", Self::new_intl_number_format_getter_callable())]
                    }
                    _ => Vec::new(),
                };
                for (property, getter) in bound_getter_accessors {
                    Self::object_set_entry(&mut prototype, property.to_string(), Value::Undefined);
                    Self::object_set_entry(
                        &mut prototype,
                        Self::object_getter_storage_key(property),
                        getter,
                    );
                    Self::object_set_entry(
                        &mut prototype,
                        Self::object_non_enumerable_storage_key(property),
                        Value::Bool(true),
                    );
                }
            }
        }
        let string_constructor = Value::StringConstructor;
        let boolean_constructor = Self::new_boolean_constructor_callable();
        let number_constructor = Self::new_number_constructor_callable();
        let bigint_constructor = Self::new_bigint_constructor_callable();
        let symbol_constructor = Value::SymbolConstructor;
        let object_constructor = Self::new_object_constructor_value();
        let reflect_object = self.new_reflect_object_value();
        let event_target_constructor = Self::new_event_target_constructor_value();
        let event_constructor = Self::new_event_constructor_value();
        let custom_event_constructor = Self::new_custom_event_constructor_value();
        let mouse_event_constructor = Self::new_mouse_event_constructor_value();
        let keyboard_event_constructor = Self::new_keyboard_event_constructor_value();
        let wheel_event_constructor = Self::new_wheel_event_constructor_value();
        let navigate_event_constructor = Self::new_navigate_event_constructor_value();
        let pointer_event_constructor = Self::new_pointer_event_constructor_value();
        let hash_change_event_constructor = Self::new_hash_change_event_constructor_value();
        let error_event_constructor = Self::new_error_event_constructor_value();
        let before_unload_event_constructor = Self::new_before_unload_event_constructor_value();
        let image_data_constructor = Self::new_image_data_constructor_value();
        let image_bitmap_constructor = self.cached_image_bitmap_constructor_value();
        let iterator_constructor = self.new_iterator_constructor_value();
        let storage_constructor = self.cached_storage_constructor_value();
        let cookie_store_constructor = if self.window_is_secure_context() {
            self.cached_cookie_store_constructor_value()
        } else {
            Value::Undefined
        };
        let cache_storage_constructor = if self.window_is_secure_context() {
            self.cached_cache_storage_constructor_value()
        } else {
            Value::Undefined
        };
        let cache_constructor = if self.window_is_secure_context() {
            self.cached_cache_constructor_value()
        } else {
            Value::Undefined
        };
        let cookie_store = self.cookie_store_global_value();
        let caches = self.cache_storage_global_value();
        let fetch_callable = Self::new_fetch_callable_value();
        let match_media_callable = Self::new_match_media_callable_value();
        let close_callable = Self::new_window_close_callable_value();
        let open_callable = Self::new_window_open_callable_value();
        let stop_callable = Self::new_window_stop_callable_value();
        let focus_callable = Self::new_window_focus_callable_value();
        let scroll_callable = Self::new_window_scroll_callable_value();
        let scroll_by_callable = Self::new_window_scroll_by_callable_value();
        let scroll_to_callable = Self::new_window_scroll_to_callable_value();
        let move_by_callable = Self::new_window_move_by_callable_value();
        let move_to_callable = Self::new_window_move_to_callable_value();
        let resize_by_callable = Self::new_window_resize_by_callable_value();
        let resize_to_callable = Self::new_window_resize_to_callable_value();
        let post_message_callable = Self::new_window_post_message_callable_value();
        let get_computed_style_callable = Self::new_window_get_computed_style_callable_value();
        let alert_callable = Self::new_window_alert_callable_value();
        let confirm_callable = Self::new_window_confirm_callable_value();
        let prompt_callable = Self::new_window_prompt_callable_value();
        let print_callable = Self::new_window_print_callable_value();
        let report_error_callable = Self::new_window_report_error_callable_value();
        let atob_callable = Self::new_global_atob_callable();
        let btoa_callable = Self::new_global_btoa_callable();
        let structured_clone_callable = Self::new_global_structured_clone_callable();
        let request_animation_frame_callable = Self::new_global_request_animation_frame_callable();
        let set_timeout_callable = Self::new_global_set_timeout_callable();
        let set_interval_callable = Self::new_global_set_interval_callable();
        let cancel_animation_frame_callable = Self::new_global_cancel_animation_frame_callable();
        let clear_interval_callable = Self::new_global_clear_interval_callable();
        let clear_timeout_callable = Self::new_global_clear_timeout_callable();
        let queue_microtask_callable = Self::new_global_queue_microtask_callable();
        let worker_constructor = self.new_worker_constructor_value();
        if let Some(worker_prototype) = self.constructor_prototype_from_value(&worker_constructor)
            && let Some(object_prototype) =
                self.constructor_prototype_from_value(&object_constructor)
            && let Value::Object(prototype) = worker_prototype
        {
            Self::set_internal_prototype(&prototype, object_prototype);
        }
        let data_transfer_constructor = self.new_data_transfer_constructor_value();
        if let Some(data_transfer_prototype) =
            self.constructor_prototype_from_value(&data_transfer_constructor)
            && let Some(object_prototype) =
                self.constructor_prototype_from_value(&object_constructor)
            && let Value::Object(prototype) = data_transfer_prototype
        {
            Self::set_internal_prototype(&prototype, object_prototype);
        }
        let option_constructor = Self::new_option_constructor_value();
        let node_list_constructor = self.cached_node_list_constructor_value();
        let text_track_constructor = self.cached_text_track_constructor_value();
        let text_track_list_constructor = self.cached_text_track_list_constructor_value();
        let time_ranges_constructor = self.cached_time_ranges_constructor_value();
        let radio_node_list_constructor = self.cached_radio_node_list_constructor_value();
        let html_collection_constructor = self.cached_html_collection_constructor_value();
        let html_form_controls_collection_constructor =
            self.cached_html_form_controls_collection_constructor_value();
        let html_options_collection_constructor =
            self.cached_html_options_collection_constructor_value();
        let text_encoder_constructor = self.cached_text_encoder_constructor_value();
        let text_decoder_constructor = self.cached_text_decoder_constructor_value();
        let text_encoder_stream_constructor = self.cached_text_encoder_stream_constructor_value();
        let text_decoder_stream_constructor = self.cached_text_decoder_stream_constructor_value();
        let css_style_sheet_constructor = Self::new_css_style_sheet_constructor_value();
        let decode_uri_callable = Self::new_global_decode_uri_callable(false);
        let decode_uri_component_callable = Self::new_global_decode_uri_callable(true);
        let create_image_bitmap_callable = Self::new_create_image_bitmap_callable();
        let request_constructor = Self::new_request_constructor_value();
        let file_constructor = Self::new_file_constructor_value();
        let clipboard_item_constructor = Self::new_clipboard_item_constructor_value();
        let headers_constructor = Self::new_headers_constructor_value();
        let url_constructor = Value::UrlConstructor;
        let core_constructor_bindings = Self::shared_core_constructor_bindings(
            &string_constructor,
            &boolean_constructor,
            &number_constructor,
            &bigint_constructor,
            &symbol_constructor,
            &object_constructor,
            &reflect_object,
        );
        let function_family_constructor_bindings = self.function_family_constructor_bindings();
        let audio_constructor = Self::new_audio_constructor_value();
        let element_constructor = Self::new_builtin_placeholder_function();
        let html_element_constructor = Self::new_builtin_placeholder_function();
        let html_anchor_element_constructor = Self::new_builtin_placeholder_function();
        let html_area_element_constructor = Self::new_builtin_placeholder_function();
        let html_body_element_constructor = Self::new_builtin_placeholder_function();
        let html_br_element_constructor = Self::new_builtin_placeholder_function();
        let html_base_element_constructor = Self::new_builtin_placeholder_function();
        let html_audio_element_constructor = Self::new_builtin_placeholder_function();
        let html_button_element_constructor = Self::new_builtin_placeholder_function();
        let html_canvas_element_constructor = Self::new_builtin_placeholder_function();
        let html_data_element_constructor = Self::new_builtin_placeholder_function();
        let html_datalist_element_constructor = Self::new_builtin_placeholder_function();
        let html_input_element_constructor = Self::new_builtin_placeholder_function();
        let html_option_element_constructor = Self::new_builtin_placeholder_function();
        let html_select_element_constructor = Self::new_builtin_placeholder_function();
        let dom_parser_constructor = Self::new_dom_parser_constructor_value();
        let xml_serializer_constructor = Self::new_xml_serializer_constructor_value();
        let document_constructor = Self::new_document_constructor_value();
        let node_constants = Self::new_object_value(vec![
            ("ELEMENT_NODE".to_string(), Value::Number(1)),
            ("ATTRIBUTE_NODE".to_string(), Value::Number(2)),
            ("TEXT_NODE".to_string(), Value::Number(3)),
            ("CDATA_SECTION_NODE".to_string(), Value::Number(4)),
            ("PROCESSING_INSTRUCTION_NODE".to_string(), Value::Number(7)),
            ("COMMENT_NODE".to_string(), Value::Number(8)),
            ("DOCUMENT_NODE".to_string(), Value::Number(9)),
            ("DOCUMENT_TYPE_NODE".to_string(), Value::Number(10)),
            ("DOCUMENT_FRAGMENT_NODE".to_string(), Value::Number(11)),
            (
                "DOCUMENT_POSITION_DISCONNECTED".to_string(),
                Value::Number(0x01),
            ),
            (
                "DOCUMENT_POSITION_PRECEDING".to_string(),
                Value::Number(0x02),
            ),
            (
                "DOCUMENT_POSITION_FOLLOWING".to_string(),
                Value::Number(0x04),
            ),
            (
                "DOCUMENT_POSITION_CONTAINS".to_string(),
                Value::Number(0x08),
            ),
            (
                "DOCUMENT_POSITION_CONTAINED_BY".to_string(),
                Value::Number(0x10),
            ),
            (
                "DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC".to_string(),
                Value::Number(0x20),
            ),
        ]);
        let node_filter_constants = Self::new_object_value(vec![
            ("SHOW_ALL".to_string(), Value::Number(4_294_967_295)),
            ("SHOW_ELEMENT".to_string(), Value::Number(0x1)),
            ("SHOW_TEXT".to_string(), Value::Number(0x4)),
            ("SHOW_COMMENT".to_string(), Value::Number(0x80)),
            ("FILTER_ACCEPT".to_string(), Value::Number(1)),
            ("FILTER_REJECT".to_string(), Value::Number(2)),
            ("FILTER_SKIP".to_string(), Value::Number(3)),
        ]);
        let local_storage = Value::Object(self.browser_apis.local_storage_object.clone());

        self.sync_document_object();
        self.sync_window_object(
            &navigator,
            &intl,
            &string_constructor,
            &boolean_constructor,
            &number_constructor,
            &bigint_constructor,
            &symbol_constructor,
            &object_constructor,
            &reflect_object,
            &event_target_constructor,
            &event_constructor,
            &custom_event_constructor,
            &mouse_event_constructor,
            &keyboard_event_constructor,
            &wheel_event_constructor,
            &navigate_event_constructor,
            &pointer_event_constructor,
            &hash_change_event_constructor,
            &error_event_constructor,
            &before_unload_event_constructor,
            &image_data_constructor,
            &image_bitmap_constructor,
            &iterator_constructor,
            &storage_constructor,
            &cookie_store_constructor,
            &cache_storage_constructor,
            &cache_constructor,
            &cookie_store,
            &caches,
            &fetch_callable,
            &match_media_callable,
            &request_constructor,
            &headers_constructor,
            &url_constructor,
            &audio_constructor,
            &data_transfer_constructor,
            &node_list_constructor,
            &text_track_constructor,
            &text_track_list_constructor,
            &time_ranges_constructor,
            &radio_node_list_constructor,
            &html_collection_constructor,
            &html_form_controls_collection_constructor,
            &html_options_collection_constructor,
            &text_encoder_constructor,
            &text_decoder_constructor,
            &text_encoder_stream_constructor,
            &text_decoder_stream_constructor,
            &element_constructor,
            &html_element_constructor,
            &html_anchor_element_constructor,
            &html_area_element_constructor,
            &html_body_element_constructor,
            &html_br_element_constructor,
            &html_base_element_constructor,
            &html_audio_element_constructor,
            &html_button_element_constructor,
            &html_canvas_element_constructor,
            &html_data_element_constructor,
            &html_datalist_element_constructor,
            &html_input_element_constructor,
            &html_select_element_constructor,
            &dom_parser_constructor,
            &xml_serializer_constructor,
            &document_constructor,
            &node_constants,
            &node_filter_constants,
            &local_storage,
            &close_callable,
            &open_callable,
            &stop_callable,
            &focus_callable,
            &scroll_callable,
            &scroll_by_callable,
            &scroll_to_callable,
            &move_by_callable,
            &move_to_callable,
            &resize_by_callable,
            &resize_to_callable,
            &post_message_callable,
            &get_computed_style_callable,
            &alert_callable,
            &confirm_callable,
            &prompt_callable,
            &print_callable,
            &report_error_callable,
            &atob_callable,
            &btoa_callable,
            &structured_clone_callable,
            &request_animation_frame_callable,
            &set_timeout_callable,
            &set_interval_callable,
            &cancel_animation_frame_callable,
            &clear_interval_callable,
            &clear_timeout_callable,
            &queue_microtask_callable,
        );
        self.initialize_global_window_entries(
            &css,
            &decode_uri_callable,
            &decode_uri_component_callable,
            &create_image_bitmap_callable,
            &object_constructor,
            &reflect_object,
            &clipboard_item_constructor,
            &file_constructor,
            &image_bitmap_constructor,
            &storage_constructor,
            &cookie_store_constructor,
            &cache_storage_constructor,
            &cache_constructor,
            &worker_constructor,
            &data_transfer_constructor,
            &option_constructor,
            &node_list_constructor,
            &text_track_constructor,
            &text_track_list_constructor,
            &time_ranges_constructor,
            &radio_node_list_constructor,
            &html_collection_constructor,
            &html_form_controls_collection_constructor,
            &html_options_collection_constructor,
            &text_encoder_constructor,
            &text_decoder_constructor,
            &text_encoder_stream_constructor,
            &text_decoder_stream_constructor,
            &css_style_sheet_constructor,
            &keyboard_event_constructor,
            &wheel_event_constructor,
            &navigate_event_constructor,
            &pointer_event_constructor,
            &hash_change_event_constructor,
            &error_event_constructor,
            &before_unload_event_constructor,
            &image_data_constructor,
            &audio_constructor,
            &html_anchor_element_constructor,
            &html_area_element_constructor,
            &html_body_element_constructor,
            &html_br_element_constructor,
            &html_base_element_constructor,
            &html_audio_element_constructor,
            &html_button_element_constructor,
            &html_canvas_element_constructor,
            &html_data_element_constructor,
            &html_datalist_element_constructor,
            &html_select_element_constructor,
            &html_option_element_constructor,
        );

        let window = Value::Object(self.dom_runtime.window_object.clone());
        let document = Value::Object(self.dom_runtime.document_object.clone());
        self.initialize_global_script_env(
            &window,
            &document,
            &navigator,
            &css,
            &intl,
            &core_constructor_bindings,
            &function_family_constructor_bindings,
            &document_constructor,
            &event_target_constructor,
            &event_constructor,
            &custom_event_constructor,
            &mouse_event_constructor,
            &keyboard_event_constructor,
            &wheel_event_constructor,
            &navigate_event_constructor,
            &pointer_event_constructor,
            &hash_change_event_constructor,
            &error_event_constructor,
            &before_unload_event_constructor,
            &image_data_constructor,
            &iterator_constructor,
            &cookie_store,
            &caches,
            &fetch_callable,
            &match_media_callable,
            &close_callable,
            &open_callable,
            &stop_callable,
            &focus_callable,
            &scroll_callable,
            &scroll_by_callable,
            &scroll_to_callable,
            &move_by_callable,
            &move_to_callable,
            &resize_by_callable,
            &resize_to_callable,
            &post_message_callable,
            &get_computed_style_callable,
            &alert_callable,
            &confirm_callable,
            &prompt_callable,
            &print_callable,
            &report_error_callable,
            &atob_callable,
            &btoa_callable,
            &structured_clone_callable,
            &request_animation_frame_callable,
            &set_timeout_callable,
            &set_interval_callable,
            &cancel_animation_frame_callable,
            &clear_interval_callable,
            &clear_timeout_callable,
            &queue_microtask_callable,
            &worker_constructor,
            &data_transfer_constructor,
            &option_constructor,
            &image_bitmap_constructor,
            &storage_constructor,
            &cookie_store_constructor,
            &cache_storage_constructor,
            &cache_constructor,
            &node_list_constructor,
            &text_track_constructor,
            &text_track_list_constructor,
            &time_ranges_constructor,
            &radio_node_list_constructor,
            &html_collection_constructor,
            &html_form_controls_collection_constructor,
            &html_options_collection_constructor,
            &text_encoder_constructor,
            &text_decoder_constructor,
            &text_encoder_stream_constructor,
            &text_decoder_stream_constructor,
            &css_style_sheet_constructor,
            &decode_uri_callable,
            &decode_uri_component_callable,
            &create_image_bitmap_callable,
            &request_constructor,
            &file_constructor,
            &clipboard_item_constructor,
            &headers_constructor,
            &audio_constructor,
            &element_constructor,
            &html_element_constructor,
            &html_anchor_element_constructor,
            &html_area_element_constructor,
            &html_body_element_constructor,
            &html_br_element_constructor,
            &html_base_element_constructor,
            &html_audio_element_constructor,
            &html_button_element_constructor,
            &html_canvas_element_constructor,
            &html_data_element_constructor,
            &html_datalist_element_constructor,
            &html_input_element_constructor,
            &html_option_element_constructor,
            &html_select_element_constructor,
            &dom_parser_constructor,
            &xml_serializer_constructor,
            &node_constants,
            &node_filter_constants,
            &location,
            &history,
            &navigation,
            &local_storage,
        );
        let _ = self.cached_function_constructor_value();
    }
}
