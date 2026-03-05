use super::*;

impl Harness {
    pub fn from_html(html: &str) -> Result<Self> {
        Self::from_html_impl("about:blank", html, &[])
    }

    pub fn from_html_with_url(url: &str, html: &str) -> Result<Self> {
        Self::from_html_impl(url, html, &[])
    }

    pub fn from_html_with_local_storage(
        html: &str,
        initial_local_storage: &[(&str, &str)],
    ) -> Result<Self> {
        Self::from_html_impl("about:blank", html, initial_local_storage)
    }

    pub fn from_html_with_url_and_local_storage(
        url: &str,
        html: &str,
        initial_local_storage: &[(&str, &str)],
    ) -> Result<Self> {
        Self::from_html_impl(url, html, initial_local_storage)
    }

    pub(crate) fn from_html_impl(
        url: &str,
        html: &str,
        initial_local_storage: &[(&str, &str)],
    ) -> Result<Self> {
        let ParseOutput { mut dom, scripts } = parse_html(html)?;
        if scripts
            .iter()
            .any(|script| script.code.contains("document.body"))
        {
            let _ = dom.ensure_document_body_element()?;
        }
        let mut harness = Self {
            dom,
            listeners: ListenerStore::default(),
            dom_runtime: DomRuntimeState::default(),
            script_runtime: ScriptRuntimeState::default(),
            document_url: url.to_string(),
            location_history: LocationHistoryState::new(url),
            scheduler: SchedulerState::default(),
            promise_runtime: PromiseRuntimeState::default(),
            symbol_runtime: SymbolRuntimeState::default(),
            browser_apis: BrowserApiState::default(),
            rng_state: 0x9E37_79B9_7F4A_7C15,
            platform_mocks: PlatformMockState::default(),
            trace_state: TraceState::default(),
        };

        harness.initialize_global_bindings();
        harness.seed_initial_local_storage(initial_local_storage);
        harness.dom_runtime.document_ready_state = "loading".to_string();

        for script in scripts {
            harness.compile_and_register_script(&script.code, script.is_module)?;
        }
        harness.finalize_document_ready_state_with_dom_content_loaded()?;

        Ok(harness)
    }

    pub(crate) fn seed_initial_local_storage(&mut self, initial_local_storage: &[(&str, &str)]) {
        if initial_local_storage.is_empty() {
            return;
        }

        let mut pairs = Vec::new();
        for (key, value) in initial_local_storage {
            if let Some((_, stored)) = pairs.iter_mut().find(|(name, _)| name == key) {
                *stored = (*value).to_string();
            } else {
                pairs.push(((*key).to_string(), (*value).to_string()));
            }
        }
        Self::set_storage_pairs(
            &mut self.browser_apis.local_storage_object.borrow_mut(),
            &pairs,
        );
    }

    pub(crate) fn with_script_env<R>(
        &mut self,
        f: impl FnOnce(&mut Self, &mut HashMap<String, Value>) -> Result<R>,
    ) -> Result<R> {
        let mut env = self.script_runtime.env.share();
        match f(self, &mut env) {
            Ok(value) => {
                self.script_runtime.env = env;
                Ok(value)
            }
            Err(err) => Err(err),
        }
    }

    pub(crate) fn with_script_env_always<R>(
        &mut self,
        f: impl FnOnce(&mut Self, &mut HashMap<String, Value>) -> Result<R>,
    ) -> Result<R> {
        let mut env = self.script_runtime.env.share();
        let result = f(self, &mut env);
        self.script_runtime.env = env;
        result
    }

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
                    to_string_tag_key.clone(),
                    Value::String(tag_name.to_string()),
                );
            }
        }
        let string_constructor = Value::StringConstructor;
        let boolean_constructor = Self::new_boolean_constructor_callable();
        let object_constructor = Self::new_object_constructor_value();
        let event_target_constructor = Self::new_event_target_constructor_value();
        let event_constructor = Self::new_event_constructor_value();
        let custom_event_constructor = Self::new_custom_event_constructor_value();
        let mouse_event_constructor = Self::new_mouse_event_constructor_value();
        let keyboard_event_constructor = Self::new_keyboard_event_constructor_value();
        let wheel_event_constructor = Self::new_wheel_event_constructor_value();
        let navigate_event_constructor = Self::new_navigate_event_constructor_value();
        let pointer_event_constructor = Self::new_pointer_event_constructor_value();
        let iterator_constructor = self.new_iterator_constructor_value();
        let cookie_store = self.cookie_store_global_value();
        let caches = self.cache_storage_global_value();
        let fetch_callable = Self::new_fetch_callable_value();
        let close_callable = Self::new_window_close_callable_value();
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
        let worker_constructor = Self::new_worker_constructor_value();
        let data_transfer_constructor = Self::new_data_transfer_constructor_value();
        let option_constructor = Self::new_option_constructor_value();
        let text_encoder_constructor = Self::new_text_encoder_constructor_value();
        let text_decoder_constructor = Self::new_text_decoder_constructor_value();
        let text_encoder_stream_constructor = Self::new_text_encoder_stream_constructor_value();
        let text_decoder_stream_constructor = Self::new_text_decoder_stream_constructor_value();
        let css_style_sheet_constructor = Self::new_css_style_sheet_constructor_value();
        let decode_uri_callable = Self::new_global_decode_uri_callable(false);
        let decode_uri_component_callable = Self::new_global_decode_uri_callable(true);
        let create_image_bitmap_callable = Self::new_create_image_bitmap_callable();
        let request_constructor = Self::new_request_constructor_value();
        let file_constructor = Self::new_file_constructor_value();
        let clipboard_item_constructor = Self::new_clipboard_item_constructor_value();
        let headers_constructor = Self::new_headers_constructor_value();
        let url_constructor = Value::UrlConstructor;
        let element_constructor = Self::new_builtin_placeholder_function();
        let html_element_constructor = Self::new_builtin_placeholder_function();
        let html_button_element_constructor = Self::new_builtin_placeholder_function();
        let html_input_element_constructor = Self::new_builtin_placeholder_function();
        let html_option_element_constructor = Self::new_builtin_placeholder_function();
        let html_select_element_constructor = Self::new_builtin_placeholder_function();
        let dom_parser_constructor = Self::new_dom_parser_constructor_value();
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
            &object_constructor,
            &event_target_constructor,
            &event_constructor,
            &custom_event_constructor,
            &mouse_event_constructor,
            &keyboard_event_constructor,
            &wheel_event_constructor,
            &navigate_event_constructor,
            &pointer_event_constructor,
            &iterator_constructor,
            &cookie_store,
            &caches,
            &fetch_callable,
            &request_constructor,
            &headers_constructor,
            &url_constructor,
            &data_transfer_constructor,
            &text_encoder_constructor,
            &text_decoder_constructor,
            &text_encoder_stream_constructor,
            &text_decoder_stream_constructor,
            &element_constructor,
            &html_element_constructor,
            &html_button_element_constructor,
            &html_input_element_constructor,
            &html_select_element_constructor,
            &dom_parser_constructor,
            &document_constructor,
            &node_constants,
            &node_filter_constants,
            &local_storage,
            &close_callable,
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
        {
            let mut window_entries = self.dom_runtime.window_object.borrow_mut();
            Self::object_set_entry(
                &mut window_entries,
                "decodeURI".to_string(),
                decode_uri_callable.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "decodeURIComponent".to_string(),
                decode_uri_component_callable.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "createImageBitmap".to_string(),
                create_image_bitmap_callable.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "Object".to_string(),
                object_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "ClipboardItem".to_string(),
                clipboard_item_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "File".to_string(),
                file_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "Worker".to_string(),
                worker_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "DataTransfer".to_string(),
                data_transfer_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "Option".to_string(),
                option_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "TextEncoder".to_string(),
                text_encoder_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "TextDecoder".to_string(),
                text_decoder_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "TextEncoderStream".to_string(),
                text_encoder_stream_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "TextDecoderStream".to_string(),
                text_decoder_stream_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "CSSStyleSheet".to_string(),
                css_style_sheet_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "KeyboardEvent".to_string(),
                keyboard_event_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "WheelEvent".to_string(),
                wheel_event_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "NavigateEvent".to_string(),
                navigate_event_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "PointerEvent".to_string(),
                pointer_event_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "HTMLButtonElement".to_string(),
                html_button_element_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "HTMLSelectElement".to_string(),
                html_select_element_constructor.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "HTMLOptionElement".to_string(),
                html_option_element_constructor.clone(),
            );
        }

        let window = Value::Object(self.dom_runtime.window_object.clone());
        let document = Value::Object(self.dom_runtime.document_object.clone());

        self.script_runtime
            .env
            .insert("document".to_string(), document);
        self.script_runtime
            .env
            .insert("Document".to_string(), document_constructor);
        self.script_runtime
            .env
            .insert("navigator".to_string(), navigator.clone());
        self.script_runtime
            .env
            .insert("clientInformation".to_string(), navigator.clone());
        self.script_runtime.env.insert("Intl".to_string(), intl);
        self.script_runtime
            .env
            .insert("String".to_string(), string_constructor);
        self.script_runtime
            .env
            .insert("Boolean".to_string(), boolean_constructor);
        self.script_runtime
            .env
            .insert("Object".to_string(), object_constructor);
        self.script_runtime
            .env
            .insert("EventTarget".to_string(), event_target_constructor);
        self.script_runtime
            .env
            .insert("Event".to_string(), event_constructor);
        self.script_runtime
            .env
            .insert("CustomEvent".to_string(), custom_event_constructor);
        self.script_runtime
            .env
            .insert("MouseEvent".to_string(), mouse_event_constructor);
        self.script_runtime
            .env
            .insert("KeyboardEvent".to_string(), keyboard_event_constructor);
        self.script_runtime
            .env
            .insert("WheelEvent".to_string(), wheel_event_constructor);
        self.script_runtime
            .env
            .insert("NavigateEvent".to_string(), navigate_event_constructor);
        self.script_runtime
            .env
            .insert("PointerEvent".to_string(), pointer_event_constructor);
        self.script_runtime
            .env
            .insert("Iterator".to_string(), iterator_constructor);
        self.script_runtime
            .env
            .insert("cookieStore".to_string(), cookie_store);
        self.script_runtime.env.insert("caches".to_string(), caches);
        self.script_runtime
            .env
            .insert("fetch".to_string(), fetch_callable);
        self.script_runtime
            .env
            .insert("close".to_string(), close_callable);
        self.script_runtime
            .env
            .insert("stop".to_string(), stop_callable);
        self.script_runtime
            .env
            .insert("focus".to_string(), focus_callable);
        self.script_runtime
            .env
            .insert("scroll".to_string(), scroll_callable);
        self.script_runtime
            .env
            .insert("scrollBy".to_string(), scroll_by_callable);
        self.script_runtime
            .env
            .insert("scrollTo".to_string(), scroll_to_callable);
        self.script_runtime
            .env
            .insert("moveBy".to_string(), move_by_callable);
        self.script_runtime
            .env
            .insert("moveTo".to_string(), move_to_callable);
        self.script_runtime
            .env
            .insert("resizeBy".to_string(), resize_by_callable);
        self.script_runtime
            .env
            .insert("resizeTo".to_string(), resize_to_callable);
        self.script_runtime
            .env
            .insert("postMessage".to_string(), post_message_callable);
        self.script_runtime
            .env
            .insert("getComputedStyle".to_string(), get_computed_style_callable);
        self.script_runtime
            .env
            .insert("alert".to_string(), alert_callable);
        self.script_runtime
            .env
            .insert("confirm".to_string(), confirm_callable);
        self.script_runtime
            .env
            .insert("prompt".to_string(), prompt_callable);
        self.script_runtime
            .env
            .insert("print".to_string(), print_callable);
        self.script_runtime
            .env
            .insert("reportError".to_string(), report_error_callable);
        self.script_runtime
            .env
            .insert("atob".to_string(), atob_callable);
        self.script_runtime
            .env
            .insert("btoa".to_string(), btoa_callable);
        self.script_runtime
            .env
            .insert("structuredClone".to_string(), structured_clone_callable);
        self.script_runtime.env.insert(
            "requestAnimationFrame".to_string(),
            request_animation_frame_callable,
        );
        self.script_runtime
            .env
            .insert("setTimeout".to_string(), set_timeout_callable);
        self.script_runtime
            .env
            .insert("setInterval".to_string(), set_interval_callable);
        self.script_runtime.env.insert(
            "cancelAnimationFrame".to_string(),
            cancel_animation_frame_callable,
        );
        self.script_runtime
            .env
            .insert("clearInterval".to_string(), clear_interval_callable);
        self.script_runtime
            .env
            .insert("clearTimeout".to_string(), clear_timeout_callable);
        self.script_runtime
            .env
            .insert("queueMicrotask".to_string(), queue_microtask_callable);
        self.script_runtime
            .env
            .insert("Worker".to_string(), worker_constructor);
        self.script_runtime
            .env
            .insert("DataTransfer".to_string(), data_transfer_constructor);
        self.script_runtime
            .env
            .insert("Option".to_string(), option_constructor);
        self.script_runtime
            .env
            .insert("TextEncoder".to_string(), text_encoder_constructor);
        self.script_runtime
            .env
            .insert("TextDecoder".to_string(), text_decoder_constructor);
        self.script_runtime.env.insert(
            "TextEncoderStream".to_string(),
            text_encoder_stream_constructor,
        );
        self.script_runtime.env.insert(
            "TextDecoderStream".to_string(),
            text_decoder_stream_constructor,
        );
        self.script_runtime
            .env
            .insert("CSSStyleSheet".to_string(), css_style_sheet_constructor);
        self.script_runtime
            .env
            .insert("decodeURI".to_string(), decode_uri_callable);
        self.script_runtime.env.insert(
            "decodeURIComponent".to_string(),
            decode_uri_component_callable,
        );
        self.script_runtime.env.insert(
            "createImageBitmap".to_string(),
            create_image_bitmap_callable,
        );
        self.script_runtime
            .env
            .insert("Request".to_string(), request_constructor);
        self.script_runtime
            .env
            .insert("File".to_string(), file_constructor);
        self.script_runtime
            .env
            .insert("ClipboardItem".to_string(), clipboard_item_constructor);
        self.script_runtime
            .env
            .insert("Headers".to_string(), headers_constructor);
        self.script_runtime
            .env
            .insert("URL".to_string(), url_constructor);
        self.script_runtime
            .env
            .insert("Element".to_string(), element_constructor);
        self.script_runtime
            .env
            .insert("HTMLElement".to_string(), html_element_constructor);
        self.script_runtime.env.insert(
            "HTMLButtonElement".to_string(),
            html_button_element_constructor,
        );
        self.script_runtime.env.insert(
            "HTMLInputElement".to_string(),
            html_input_element_constructor,
        );
        self.script_runtime.env.insert(
            "HTMLOptionElement".to_string(),
            html_option_element_constructor,
        );
        self.script_runtime.env.insert(
            "HTMLSelectElement".to_string(),
            html_select_element_constructor,
        );
        self.script_runtime
            .env
            .insert("DOMParser".to_string(), dom_parser_constructor);
        self.script_runtime
            .env
            .insert("Node".to_string(), node_constants);
        self.script_runtime
            .env
            .insert("NodeFilter".to_string(), node_filter_constants);
        self.script_runtime
            .env
            .insert("location".to_string(), location);
        self.script_runtime
            .env
            .insert("history".to_string(), history);
        self.script_runtime
            .env
            .insert("navigation".to_string(), navigation);
        self.script_runtime
            .env
            .insert("localStorage".to_string(), local_storage);
        self.script_runtime
            .env
            .insert("window".to_string(), window.clone());
        self.script_runtime
            .env
            .insert("globalThis".to_string(), window.clone());
        self.script_runtime
            .env
            .insert("this".to_string(), window.clone());
        self.script_runtime
            .env
            .insert("self".to_string(), window.clone());
        self.script_runtime
            .env
            .insert("top".to_string(), window.clone());
        self.script_runtime
            .env
            .insert("parent".to_string(), window.clone());
        self.script_runtime.env.insert("frames".to_string(), window);
        self.script_runtime
            .env
            .insert(INTERNAL_SCOPE_DEPTH_KEY.to_string(), Value::Number(0));
    }
}
