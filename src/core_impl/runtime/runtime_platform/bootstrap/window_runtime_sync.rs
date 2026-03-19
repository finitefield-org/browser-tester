use super::*;

impl Harness {
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
            "open",
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
            "scrollX",
            "scrollY",
            "pageXOffset",
            "pageYOffset",
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
            "Function",
            "GeneratorFunction",
            "AsyncGeneratorFunction",
            "Boolean",
            "Number",
            "BigInt",
            "Symbol",
            "RegExp",
            "Blob",
            "URLSearchParams",
            "ArrayBuffer",
            "Promise",
            "Reflect",
            "Map",
            "WeakMap",
            "Set",
            "WeakSet",
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
            "NodeList",
            "TextTrack",
            "TextTrackList",
            "TimeRanges",
            "RadioNodeList",
            "HTMLCollection",
            "HTMLFormControlsCollection",
            "HTMLOptionsCollection",
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
            "ImageBitmap",
            "Storage",
            "CookieStore",
            "CacheStorage",
            "Cache",
            "Iterator",
            "cookieStore",
            "caches",
            "fetch",
            "matchMedia",
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
            "XMLSerializer",
            "Document",
            "Node",
            "NodeFilter",
            "name",
            "getSelection",
        ]
    }

    #[allow(clippy::too_many_arguments)]
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
        reflect_object: &Value,
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
        image_bitmap_constructor: &Value,
        iterator_constructor: &Value,
        storage_constructor: &Value,
        cookie_store_constructor: &Value,
        cache_storage_constructor: &Value,
        cache_constructor: &Value,
        cookie_store: &Value,
        caches: &Value,
        fetch_callable: &Value,
        match_media_callable: &Value,
        request_constructor: &Value,
        headers_constructor: &Value,
        _url_constructor: &Value,
        audio_constructor: &Value,
        data_transfer_constructor: &Value,
        node_list_constructor: &Value,
        text_track_constructor: &Value,
        text_track_list_constructor: &Value,
        time_ranges_constructor: &Value,
        radio_node_list_constructor: &Value,
        html_collection_constructor: &Value,
        html_form_controls_collection_constructor: &Value,
        html_options_collection_constructor: &Value,
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
        xml_serializer_constructor: &Value,
        document_constructor: &Value,
        node_constants: &Value,
        node_filter_constants: &Value,
        local_storage: &Value,
        close_callable: &Value,
        open_callable: &Value,
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
        let core_constructor_bindings = Self::shared_core_constructor_bindings(
            string_constructor,
            boolean_constructor,
            number_constructor,
            bigint_constructor,
            symbol_constructor,
            object_constructor,
            reflect_object,
        );
        let function_family_constructor_bindings = self.function_family_constructor_bindings();
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
            ("open".to_string(), open_callable.clone()),
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
                "scrollX".to_string(),
                Value::Number(self.dom_runtime.document_scroll_x),
            ),
            (
                "scrollY".to_string(),
                Value::Number(self.dom_runtime.document_scroll_y),
            ),
            (
                "pageXOffset".to_string(),
                Value::Number(self.dom_runtime.document_scroll_x),
            ),
            (
                "pageYOffset".to_string(),
                Value::Number(self.dom_runtime.document_scroll_y),
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
            ("ImageBitmap".to_string(), image_bitmap_constructor.clone()),
            ("Storage".to_string(), storage_constructor.clone()),
            ("CookieStore".to_string(), cookie_store_constructor.clone()),
            (
                "CacheStorage".to_string(),
                cache_storage_constructor.clone(),
            ),
            ("Cache".to_string(), cache_constructor.clone()),
            ("Iterator".to_string(), iterator_constructor.clone()),
            ("cookieStore".to_string(), cookie_store.clone()),
            ("caches".to_string(), caches.clone()),
            ("fetch".to_string(), fetch_callable.clone()),
            ("matchMedia".to_string(), match_media_callable.clone()),
            ("Request".to_string(), request_constructor.clone()),
            ("Headers".to_string(), headers_constructor.clone()),
            ("Audio".to_string(), audio_constructor.clone()),
            (
                "DataTransfer".to_string(),
                data_transfer_constructor.clone(),
            ),
            ("NodeList".to_string(), node_list_constructor.clone()),
            ("TextTrack".to_string(), text_track_constructor.clone()),
            (
                "TextTrackList".to_string(),
                text_track_list_constructor.clone(),
            ),
            ("TimeRanges".to_string(), time_ranges_constructor.clone()),
            (
                "RadioNodeList".to_string(),
                radio_node_list_constructor.clone(),
            ),
            (
                "HTMLCollection".to_string(),
                html_collection_constructor.clone(),
            ),
            (
                "HTMLFormControlsCollection".to_string(),
                html_form_controls_collection_constructor.clone(),
            ),
            (
                "HTMLOptionsCollection".to_string(),
                html_options_collection_constructor.clone(),
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
            (
                "XMLSerializer".to_string(),
                xml_serializer_constructor.clone(),
            ),
            ("Document".to_string(), document_constructor.clone()),
            ("Node".to_string(), node_constants.clone()),
            ("NodeFilter".to_string(), node_filter_constants.clone()),
            ("name".to_string(), name_value),
            (
                "getSelection".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        entries.extend(core_constructor_bindings);
        entries.extend(function_family_constructor_bindings);
        entries.extend(extras);
        Self::mark_object_properties_non_enumerable(&mut entries, &["getSelection"]);
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
        Self::object_set_entry(
            &mut entries,
            "scrollX".to_string(),
            Value::Number(self.dom_runtime.document_scroll_x),
        );
        Self::object_set_entry(
            &mut entries,
            "scrollY".to_string(),
            Value::Number(self.dom_runtime.document_scroll_y),
        );
        Self::object_set_entry(
            &mut entries,
            "pageXOffset".to_string(),
            Value::Number(self.dom_runtime.document_scroll_x),
        );
        Self::object_set_entry(
            &mut entries,
            "pageYOffset".to_string(),
            Value::Number(self.dom_runtime.document_scroll_y),
        );
        Self::object_set_entry(&mut entries, "cookieStore".to_string(), cookie_store);
        Self::object_set_entry(&mut entries, "caches".to_string(), caches);
    }
}
