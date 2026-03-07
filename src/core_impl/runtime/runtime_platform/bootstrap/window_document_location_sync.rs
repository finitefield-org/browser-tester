use super::*;

impl Harness {
    pub(crate) fn current_location_parts(&self) -> LocationParts {
        let mut parts = LocationParts::parse(&self.document_url).unwrap_or_else(|| LocationParts {
            scheme: "about".to_string(),
            has_authority: false,
            username: String::new(),
            password: String::new(),
            hostname: String::new(),
            port: String::new(),
            pathname: String::new(),
            opaque_path: "blank".to_string(),
            search: String::new(),
            hash: String::new(),
        });
        Self::normalize_url_parts_for_serialization(&mut parts);
        parts
    }

    pub(crate) fn window_is_secure_context(&self) -> bool {
        matches!(
            self.current_location_parts().scheme.as_str(),
            "https" | "wss"
        )
    }

    pub(crate) fn document_builtin_keys() -> &'static [&'static str] {
        &[
            "defaultView",
            "location",
            "URL",
            "documentURI",
            "cookie",
            "adoptedStyleSheets",
            "createElement",
            "createElementNS",
            "createTextNode",
            "createAttribute",
            "createDocumentFragment",
            "createRange",
            "getSelection",
            "append",
            "getElementById",
            "getElementsByClassName",
            "getElementsByName",
            "getElementsByTagName",
            "getElementsByTagNameNS",
            "querySelector",
            "querySelectorAll",
            "createTreeWalker",
        ]
    }

    pub(crate) fn sync_document_object(&mut self) {
        let mut extras = Vec::new();
        let mut adopted_style_sheets: Option<Value> = None;
        {
            let entries = self.dom_runtime.document_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
                    continue;
                }
                if key == "adoptedStyleSheets" {
                    adopted_style_sheets = Some(value.clone());
                    continue;
                }
                if Self::document_builtin_keys()
                    .iter()
                    .any(|builtin| builtin == key)
                {
                    continue;
                }
                extras.push((key.clone(), value.clone()));
            }
        }

        let adopted_style_sheets = adopted_style_sheets.unwrap_or_else(|| {
            Self::new_adopted_style_sheets_array_value(Value::Object(
                self.dom_runtime.document_object.clone(),
            ))
        });
        if let Value::Array(values) = &adopted_style_sheets {
            self.mark_as_adopted_style_sheets_array(
                values,
                Value::Object(self.dom_runtime.document_object.clone()),
            );
        }

        let mut entries = vec![
            (INTERNAL_DOCUMENT_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                "defaultView".to_string(),
                Value::Object(self.dom_runtime.window_object.clone()),
            ),
            (
                "location".to_string(),
                Value::Object(self.dom_runtime.location_object.clone()),
            ),
            ("URL".to_string(), Value::String(self.document_url.clone())),
            (
                "documentURI".to_string(),
                Value::String(self.document_url.clone()),
            ),
            (
                "cookie".to_string(),
                Value::String(self.document_cookie_string()),
            ),
            ("adoptedStyleSheets".to_string(), adopted_style_sheets),
            (
                "createElement".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createElementNS".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createTextNode".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createAttribute".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createDocumentFragment".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createRange".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getSelection".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "append".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getElementById".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getElementsByClassName".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getElementsByName".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getElementsByTagName".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getElementsByTagNameNS".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "querySelector".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "querySelectorAll".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createTreeWalker".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        entries.extend(extras);
        *self.dom_runtime.document_object.borrow_mut() = entries.into();
    }

    pub(crate) fn window_builtin_keys() -> &'static [&'static str] {
        &[
            "window",
            "globalThis",
            "self",
            "top",
            "parent",
            "frames",
            "length",
            "closed",
            "close",
            "stop",
            "focus",
            "scroll",
            "scrollBy",
            "scrollTo",
            "moveBy",
            "moveTo",
            "resizeBy",
            "resizeTo",
            "postMessage",
            "getComputedStyle",
            "alert",
            "confirm",
            "prompt",
            "print",
            "reportError",
            "atob",
            "btoa",
            "structuredClone",
            "requestAnimationFrame",
            "setTimeout",
            "setInterval",
            "cancelAnimationFrame",
            "clearInterval",
            "clearTimeout",
            "queueMicrotask",
            "screenX",
            "screenY",
            "screenLeft",
            "screenTop",
            "location",
            "history",
            "navigation",
            "navigator",
            "clientInformation",
            "localStorage",
            "document",
            "origin",
            "isSecureContext",
            "Intl",
            "String",
            "Boolean",
            "Number",
            "BigInt",
            "Symbol",
            "Int8Array",
            "Uint8Array",
            "Uint8ClampedArray",
            "Int16Array",
            "Uint16Array",
            "Int32Array",
            "Uint32Array",
            "Float16Array",
            "Float32Array",
            "Float64Array",
            "BigInt64Array",
            "BigUint64Array",
            "Object",
            "EventTarget",
            "Event",
            "CustomEvent",
            "MouseEvent",
            "KeyboardEvent",
            "WheelEvent",
            "NavigateEvent",
            "PointerEvent",
            "HashChangeEvent",
            "ErrorEvent",
            "BeforeUnloadEvent",
            "ImageData",
            "Iterator",
            "cookieStore",
            "caches",
            "fetch",
            "Request",
            "Headers",
            "URL",
            "Audio",
            "DataTransfer",
            "TextEncoder",
            "TextDecoder",
            "TextEncoderStream",
            "TextDecoderStream",
            "Element",
            "HTMLElement",
            "HTMLAnchorElement",
            "HTMLAreaElement",
            "HTMLBodyElement",
            "HTMLBRElement",
            "HTMLBaseElement",
            "HTMLAudioElement",
            "HTMLButtonElement",
            "HTMLCanvasElement",
            "HTMLDataElement",
            "HTMLDataListElement",
            "HTMLInputElement",
            "HTMLSelectElement",
            "DOMParser",
            "Document",
            "Node",
            "NodeFilter",
            "name",
            "getSelection",
        ]
    }

    pub(crate) fn sync_window_object(
        &mut self,
        navigator: &Value,
        intl: &Value,
        string_constructor: &Value,
        boolean_constructor: &Value,
        number_constructor: &Value,
        bigint_constructor: &Value,
        symbol_constructor: &Value,
        object_constructor: &Value,
        event_target_constructor: &Value,
        event_constructor: &Value,
        custom_event_constructor: &Value,
        mouse_event_constructor: &Value,
        keyboard_event_constructor: &Value,
        wheel_event_constructor: &Value,
        navigate_event_constructor: &Value,
        pointer_event_constructor: &Value,
        hash_change_event_constructor: &Value,
        error_event_constructor: &Value,
        before_unload_event_constructor: &Value,
        image_data_constructor: &Value,
        iterator_constructor: &Value,
        cookie_store: &Value,
        caches: &Value,
        fetch_callable: &Value,
        request_constructor: &Value,
        headers_constructor: &Value,
        url_constructor: &Value,
        audio_constructor: &Value,
        data_transfer_constructor: &Value,
        text_encoder_constructor: &Value,
        text_decoder_constructor: &Value,
        text_encoder_stream_constructor: &Value,
        text_decoder_stream_constructor: &Value,
        element_constructor: &Value,
        html_element_constructor: &Value,
        html_anchor_element_constructor: &Value,
        html_area_element_constructor: &Value,
        html_body_element_constructor: &Value,
        html_br_element_constructor: &Value,
        html_base_element_constructor: &Value,
        html_audio_element_constructor: &Value,
        html_button_element_constructor: &Value,
        html_canvas_element_constructor: &Value,
        html_data_element_constructor: &Value,
        html_datalist_element_constructor: &Value,
        html_input_element_constructor: &Value,
        html_select_element_constructor: &Value,
        dom_parser_constructor: &Value,
        document_constructor: &Value,
        node_constants: &Value,
        node_filter_constants: &Value,
        local_storage: &Value,
        close_callable: &Value,
        stop_callable: &Value,
        focus_callable: &Value,
        scroll_callable: &Value,
        scroll_by_callable: &Value,
        scroll_to_callable: &Value,
        move_by_callable: &Value,
        move_to_callable: &Value,
        resize_by_callable: &Value,
        resize_to_callable: &Value,
        post_message_callable: &Value,
        get_computed_style_callable: &Value,
        alert_callable: &Value,
        confirm_callable: &Value,
        prompt_callable: &Value,
        print_callable: &Value,
        report_error_callable: &Value,
        atob_callable: &Value,
        btoa_callable: &Value,
        structured_clone_callable: &Value,
        request_animation_frame_callable: &Value,
        set_timeout_callable: &Value,
        set_interval_callable: &Value,
        cancel_animation_frame_callable: &Value,
        clear_interval_callable: &Value,
        clear_timeout_callable: &Value,
        queue_microtask_callable: &Value,
    ) {
        let mut extras = Vec::new();
        let mut name_value = Value::String(String::new());
        {
            let entries = self.dom_runtime.window_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
                    continue;
                }
                if key == "name" {
                    name_value = Value::String(value.as_string());
                    continue;
                }
                if Self::window_builtin_keys()
                    .iter()
                    .any(|builtin| builtin == key)
                {
                    continue;
                }
                extras.push((key.clone(), value.clone()));
            }
        }

        let window_ref = Value::Object(self.dom_runtime.window_object.clone());
        let mut entries = vec![
            (INTERNAL_WINDOW_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_EVENT_TARGET_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("window".to_string(), window_ref.clone()),
            ("globalThis".to_string(), window_ref.clone()),
            ("self".to_string(), window_ref.clone()),
            ("top".to_string(), window_ref.clone()),
            ("parent".to_string(), window_ref.clone()),
            ("frames".to_string(), window_ref),
            ("length".to_string(), Value::Number(0)),
            (
                "closed".to_string(),
                Value::Bool(self.browser_apis.window_closed),
            ),
            ("close".to_string(), close_callable.clone()),
            ("stop".to_string(), stop_callable.clone()),
            ("focus".to_string(), focus_callable.clone()),
            ("scroll".to_string(), scroll_callable.clone()),
            ("scrollBy".to_string(), scroll_by_callable.clone()),
            ("scrollTo".to_string(), scroll_to_callable.clone()),
            ("moveBy".to_string(), move_by_callable.clone()),
            ("moveTo".to_string(), move_to_callable.clone()),
            ("resizeBy".to_string(), resize_by_callable.clone()),
            ("resizeTo".to_string(), resize_to_callable.clone()),
            ("postMessage".to_string(), post_message_callable.clone()),
            (
                "getComputedStyle".to_string(),
                get_computed_style_callable.clone(),
            ),
            ("alert".to_string(), alert_callable.clone()),
            ("confirm".to_string(), confirm_callable.clone()),
            ("prompt".to_string(), prompt_callable.clone()),
            ("print".to_string(), print_callable.clone()),
            ("reportError".to_string(), report_error_callable.clone()),
            ("atob".to_string(), atob_callable.clone()),
            ("btoa".to_string(), btoa_callable.clone()),
            (
                "structuredClone".to_string(),
                structured_clone_callable.clone(),
            ),
            (
                "requestAnimationFrame".to_string(),
                request_animation_frame_callable.clone(),
            ),
            ("setTimeout".to_string(), set_timeout_callable.clone()),
            ("setInterval".to_string(), set_interval_callable.clone()),
            (
                "cancelAnimationFrame".to_string(),
                cancel_animation_frame_callable.clone(),
            ),
            ("clearInterval".to_string(), clear_interval_callable.clone()),
            ("clearTimeout".to_string(), clear_timeout_callable.clone()),
            (
                "queueMicrotask".to_string(),
                queue_microtask_callable.clone(),
            ),
            (
                "screenX".to_string(),
                Value::Number(self.browser_apis.window_screen_x),
            ),
            (
                "screenY".to_string(),
                Value::Number(self.browser_apis.window_screen_y),
            ),
            (
                "screenLeft".to_string(),
                Value::Number(self.browser_apis.window_screen_x),
            ),
            (
                "screenTop".to_string(),
                Value::Number(self.browser_apis.window_screen_y),
            ),
            (
                "location".to_string(),
                Value::Object(self.dom_runtime.location_object.clone()),
            ),
            (
                "history".to_string(),
                Value::Object(self.location_history.history_object.clone()),
            ),
            (
                "navigation".to_string(),
                Value::Object(self.location_history.navigation_object.clone()),
            ),
            ("navigator".to_string(), navigator.clone()),
            ("clientInformation".to_string(), navigator.clone()),
            (
                "document".to_string(),
                Value::Object(self.dom_runtime.document_object.clone()),
            ),
            ("localStorage".to_string(), local_storage.clone()),
            (
                "origin".to_string(),
                Value::String(self.current_location_parts().origin()),
            ),
            (
                "isSecureContext".to_string(),
                Value::Bool(self.window_is_secure_context()),
            ),
            ("Intl".to_string(), intl.clone()),
            ("String".to_string(), string_constructor.clone()),
            ("Boolean".to_string(), boolean_constructor.clone()),
            ("Number".to_string(), number_constructor.clone()),
            ("BigInt".to_string(), bigint_constructor.clone()),
            ("Symbol".to_string(), symbol_constructor.clone()),
            ("Object".to_string(), object_constructor.clone()),
            ("EventTarget".to_string(), event_target_constructor.clone()),
            ("Event".to_string(), event_constructor.clone()),
            ("CustomEvent".to_string(), custom_event_constructor.clone()),
            ("MouseEvent".to_string(), mouse_event_constructor.clone()),
            (
                "KeyboardEvent".to_string(),
                keyboard_event_constructor.clone(),
            ),
            ("WheelEvent".to_string(), wheel_event_constructor.clone()),
            (
                "NavigateEvent".to_string(),
                navigate_event_constructor.clone(),
            ),
            (
                "PointerEvent".to_string(),
                pointer_event_constructor.clone(),
            ),
            (
                "HashChangeEvent".to_string(),
                hash_change_event_constructor.clone(),
            ),
            ("ErrorEvent".to_string(), error_event_constructor.clone()),
            (
                "BeforeUnloadEvent".to_string(),
                before_unload_event_constructor.clone(),
            ),
            ("ImageData".to_string(), image_data_constructor.clone()),
            ("Iterator".to_string(), iterator_constructor.clone()),
            ("cookieStore".to_string(), cookie_store.clone()),
            ("caches".to_string(), caches.clone()),
            ("fetch".to_string(), fetch_callable.clone()),
            ("Request".to_string(), request_constructor.clone()),
            ("Headers".to_string(), headers_constructor.clone()),
            ("URL".to_string(), url_constructor.clone()),
            ("Audio".to_string(), audio_constructor.clone()),
            (
                "DataTransfer".to_string(),
                data_transfer_constructor.clone(),
            ),
            ("TextEncoder".to_string(), text_encoder_constructor.clone()),
            ("TextDecoder".to_string(), text_decoder_constructor.clone()),
            (
                "TextEncoderStream".to_string(),
                text_encoder_stream_constructor.clone(),
            ),
            (
                "TextDecoderStream".to_string(),
                text_decoder_stream_constructor.clone(),
            ),
            ("Element".to_string(), element_constructor.clone()),
            ("HTMLElement".to_string(), html_element_constructor.clone()),
            (
                "HTMLAnchorElement".to_string(),
                html_anchor_element_constructor.clone(),
            ),
            (
                "HTMLAreaElement".to_string(),
                html_area_element_constructor.clone(),
            ),
            (
                "HTMLBodyElement".to_string(),
                html_body_element_constructor.clone(),
            ),
            (
                "HTMLBRElement".to_string(),
                html_br_element_constructor.clone(),
            ),
            (
                "HTMLBaseElement".to_string(),
                html_base_element_constructor.clone(),
            ),
            (
                "HTMLAudioElement".to_string(),
                html_audio_element_constructor.clone(),
            ),
            (
                "HTMLButtonElement".to_string(),
                html_button_element_constructor.clone(),
            ),
            (
                "HTMLCanvasElement".to_string(),
                html_canvas_element_constructor.clone(),
            ),
            (
                "HTMLDataElement".to_string(),
                html_data_element_constructor.clone(),
            ),
            (
                "HTMLDataListElement".to_string(),
                html_datalist_element_constructor.clone(),
            ),
            (
                "HTMLInputElement".to_string(),
                html_input_element_constructor.clone(),
            ),
            (
                "HTMLSelectElement".to_string(),
                html_select_element_constructor.clone(),
            ),
            ("DOMParser".to_string(), dom_parser_constructor.clone()),
            ("Document".to_string(), document_constructor.clone()),
            ("Node".to_string(), node_constants.clone()),
            ("NodeFilter".to_string(), node_filter_constants.clone()),
            ("name".to_string(), name_value),
            (
                "getSelection".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        for kind in TypedArrayKind::concrete_kinds() {
            entries.push((
                kind.name().to_string(),
                Value::TypedArrayConstructor(TypedArrayConstructorKind::Concrete(*kind)),
            ));
        }
        entries.extend(extras);
        *self.dom_runtime.window_object.borrow_mut() = entries.into();
    }

    pub(crate) fn sync_window_runtime_properties(&mut self) {
        let cookie_store = self.cookie_store_global_value();
        let caches = self.cache_storage_global_value();
        self.script_runtime
            .env
            .insert("cookieStore".to_string(), cookie_store.clone());
        self.script_runtime
            .env
            .insert("caches".to_string(), caches.clone());

        let mut entries = self.dom_runtime.window_object.borrow_mut();
        Self::object_set_entry(
            &mut entries,
            "origin".to_string(),
            Value::String(self.current_location_parts().origin()),
        );
        Self::object_set_entry(
            &mut entries,
            "isSecureContext".to_string(),
            Value::Bool(self.window_is_secure_context()),
        );
        Self::object_set_entry(
            &mut entries,
            "closed".to_string(),
            Value::Bool(self.browser_apis.window_closed),
        );
        Self::object_set_entry(
            &mut entries,
            "screenX".to_string(),
            Value::Number(self.browser_apis.window_screen_x),
        );
        Self::object_set_entry(
            &mut entries,
            "screenY".to_string(),
            Value::Number(self.browser_apis.window_screen_y),
        );
        Self::object_set_entry(
            &mut entries,
            "screenLeft".to_string(),
            Value::Number(self.browser_apis.window_screen_x),
        );
        Self::object_set_entry(
            &mut entries,
            "screenTop".to_string(),
            Value::Number(self.browser_apis.window_screen_y),
        );
        Self::object_set_entry(&mut entries, "cookieStore".to_string(), cookie_store);
        Self::object_set_entry(&mut entries, "caches".to_string(), caches);
    }

    pub(crate) fn location_builtin_keys() -> &'static [&'static str] {
        &[
            "href",
            "protocol",
            "host",
            "hostname",
            "port",
            "pathname",
            "search",
            "hash",
            "origin",
            "ancestorOrigins",
            "assign",
            "reload",
            "replace",
            "toString",
        ]
    }

    pub(crate) fn sync_location_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.dom_runtime.location_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
                    continue;
                }
                if Self::location_builtin_keys()
                    .iter()
                    .any(|builtin| builtin == key)
                {
                    continue;
                }
                extras.push((key.clone(), value.clone()));
            }
        }

        let parts = self.current_location_parts();
        let mut entries = vec![
            (INTERNAL_LOCATION_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
                Value::String(parts.href()),
            ),
            ("href".to_string(), Value::String(parts.href())),
            ("protocol".to_string(), Value::String(parts.protocol())),
            ("host".to_string(), Value::String(parts.host())),
            (
                "hostname".to_string(),
                Value::String(parts.hostname.clone()),
            ),
            ("port".to_string(), Value::String(parts.effective_port())),
            (
                "pathname".to_string(),
                Value::String(if parts.has_authority {
                    parts.pathname.clone()
                } else {
                    parts.opaque_path.clone()
                }),
            ),
            ("search".to_string(), Value::String(parts.search.clone())),
            ("hash".to_string(), Value::String(parts.hash.clone())),
            ("origin".to_string(), Value::String(parts.origin())),
            (
                "ancestorOrigins".to_string(),
                Self::new_array_value(Vec::new()),
            ),
            (
                "assign".to_string(),
                Self::new_receiver_builtin_callable("location", "assign"),
            ),
            (
                "reload".to_string(),
                Self::new_receiver_builtin_callable("location", "reload"),
            ),
            (
                "replace".to_string(),
                Self::new_receiver_builtin_callable("location", "replace"),
            ),
            (
                "toString".to_string(),
                Self::new_receiver_builtin_callable("location", "toString"),
            ),
        ];
        entries.extend(extras);
        *self.dom_runtime.location_object.borrow_mut() = entries.into();
    }

    pub(crate) fn is_location_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_LOCATION_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn history_builtin_keys() -> &'static [&'static str] {
        &[
            "length",
            "scrollRestoration",
            "state",
            "back",
            "forward",
            "go",
            "pushState",
            "replaceState",
        ]
    }

    pub(crate) fn navigation_builtin_keys() -> &'static [&'static str] {
        &[
            "activation",
            "canGoBack",
            "canGoForward",
            "currentEntry",
            "transition",
            "back",
            "entries",
            "forward",
            "navigate",
            "reload",
            "traverseTo",
            "updateCurrentEntry",
            "addEventListener",
            "removeEventListener",
            "dispatchEvent",
        ]
    }

    pub(crate) fn current_history_state(&self) -> Value {
        self.location_history
            .history_entries
            .get(self.location_history.history_index)
            .map(|entry| entry.state.clone())
            .unwrap_or(Value::Null)
    }

    pub(crate) fn sync_history_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.location_history.history_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
                    continue;
                }
                if Self::history_builtin_keys()
                    .iter()
                    .any(|builtin| builtin == key)
                {
                    continue;
                }
                extras.push((key.clone(), value.clone()));
            }
        }

        let mut entries = vec![
            (INTERNAL_HISTORY_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                "length".to_string(),
                Value::Number(self.location_history.history_entries.len() as i64),
            ),
            (
                "scrollRestoration".to_string(),
                Value::String(self.location_history.history_scroll_restoration.clone()),
            ),
            ("state".to_string(), self.current_history_state()),
            ("back".to_string(), Self::new_builtin_placeholder_function()),
            (
                "forward".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("go".to_string(), Self::new_builtin_placeholder_function()),
            (
                "pushState".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "replaceState".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        entries.extend(extras);
        *self.location_history.history_object.borrow_mut() = entries.into();
    }

    pub(crate) fn sync_navigation_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.location_history.navigation_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
                    continue;
                }
                if Self::navigation_builtin_keys()
                    .iter()
                    .any(|builtin| builtin == key)
                {
                    continue;
                }
                extras.push((key.clone(), value.clone()));
            }
        }

        let mut entries = vec![
            (
                INTERNAL_NAVIGATION_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_EVENT_TARGET_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("activation".to_string(), Value::Null),
            (
                "canGoBack".to_string(),
                Value::Bool(self.navigation_can_go_back()),
            ),
            (
                "canGoForward".to_string(),
                Value::Bool(self.navigation_can_go_forward()),
            ),
            (
                "currentEntry".to_string(),
                self.navigation_current_entry_value(),
            ),
            ("transition".to_string(), Value::Null),
            ("back".to_string(), Self::new_builtin_placeholder_function()),
            (
                "entries".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "forward".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "navigate".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "reload".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "traverseTo".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "updateCurrentEntry".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
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
        ];
        entries.extend(extras);
        *self.location_history.navigation_object.borrow_mut() = entries.into();
    }
}
