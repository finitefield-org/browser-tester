use super::*;

impl Harness {
    fn function_length(function: &Rc<FunctionValue>) -> i64 {
        let mut length = 0_i64;
        for param in &function.handler.params {
            if param.is_rest || param.default.is_some() {
                break;
            }
            length += 1;
        }
        length
    }

    fn function_display_name(&self, function: &Rc<FunctionValue>) -> String {
        self.script_runtime
            .function_public_properties
            .get(&function.function_id)
            .and_then(|entries| Self::object_get_entry(entries, "name"))
            .map(|value| value.as_string())
            .unwrap_or_else(|| function.expression_name.clone().unwrap_or_default())
    }

    fn object_backed_callable_name_and_length(kind: &str) -> Option<(&'static str, i64)> {
        match kind {
            "generator_function_constructor" => Some(("GeneratorFunction", 1)),
            "async_generator_function_constructor" => Some(("AsyncGeneratorFunction", 1)),
            "boolean_constructor" => Some(("Boolean", 1)),
            "number_constructor" => Some(("Number", 1)),
            "bigint_constructor" => Some(("BigInt", 1)),
            "object_constructor" => Some(("Object", 1)),
            "function_constructor" => Some(("Function", 1)),
            "node_list_constructor" => Some(("NodeList", 0)),
            "image_bitmap_constructor" => Some(("ImageBitmap", 0)),
            "text_track_constructor" => Some(("TextTrack", 0)),
            "text_track_list_constructor" => Some(("TextTrackList", 0)),
            "time_ranges_constructor" => Some(("TimeRanges", 0)),
            "storage_constructor" => Some(("Storage", 0)),
            "cookie_store_constructor" => Some(("CookieStore", 0)),
            "cache_storage_constructor" => Some(("CacheStorage", 0)),
            "cache_constructor" => Some(("Cache", 0)),
            "radio_node_list_constructor" => Some(("RadioNodeList", 0)),
            "html_collection_constructor" => Some(("HTMLCollection", 0)),
            "html_form_controls_collection_constructor" => Some(("HTMLFormControlsCollection", 0)),
            "html_options_collection_constructor" => Some(("HTMLOptionsCollection", 0)),
            "function_call" => Some(("call", 1)),
            "function_apply" => Some(("apply", 2)),
            "function_bind" => Some(("bind", 1)),
            "function_to_string" => Some(("toString", 0)),
            "event_target_constructor" => Some(("EventTarget", 0)),
            "event_constructor" => Some(("Event", 1)),
            "custom_event_constructor" => Some(("CustomEvent", 1)),
            "mouse_event_constructor" => Some(("MouseEvent", 1)),
            "keyboard_event_constructor" => Some(("KeyboardEvent", 1)),
            "wheel_event_constructor" => Some(("WheelEvent", 1)),
            "navigate_event_constructor" => Some(("NavigateEvent", 1)),
            "pointer_event_constructor" => Some(("PointerEvent", 1)),
            "error_event_constructor" => Some(("ErrorEvent", 1)),
            "hash_change_event_constructor" => Some(("HashChangeEvent", 1)),
            "before_unload_event_constructor" => Some(("BeforeUnloadEvent", 1)),
            "image_data_constructor" => Some(("ImageData", 2)),
            "dom_parser_constructor" => Some(("DOMParser", 0)),
            "xml_serializer_constructor" => Some(("XMLSerializer", 0)),
            "document_constructor" => Some(("Document", 0)),
            "document_parse_html" => Some(("parseHTML", 1)),
            "document_parse_html_unsafe" => Some(("parseHTMLUnsafe", 1)),
            "fetch_function" => Some(("fetch", 1)),
            "match_media_function" => Some(("matchMedia", 1)),
            "window_close_function" => Some(("close", 0)),
            "window_open_function" => Some(("open", 0)),
            "window_stop_function" => Some(("stop", 0)),
            "window_focus_function" => Some(("focus", 0)),
            "window_scroll_function" => Some(("scroll", 0)),
            "window_scroll_by_function" => Some(("scrollBy", 0)),
            "window_scroll_to_function" => Some(("scrollTo", 0)),
            "window_move_by_function" => Some(("moveBy", 2)),
            "window_move_to_function" => Some(("moveTo", 2)),
            "window_resize_by_function" => Some(("resizeBy", 2)),
            "window_resize_to_function" => Some(("resizeTo", 2)),
            "window_post_message_function" => Some(("postMessage", 1)),
            "window_get_computed_style_function" => Some(("getComputedStyle", 1)),
            "computed_style_item" => Some(("item", 1)),
            "dom_rect_list_item" => Some(("item", 1)),
            "window_alert_function" => Some(("alert", 0)),
            "window_confirm_function" => Some(("confirm", 0)),
            "window_print_function" => Some(("print", 0)),
            "window_report_error_function" => Some(("reportError", 1)),
            "window_prompt_function" => Some(("prompt", 0)),
            "popup_window_close_function" => Some(("close", 0)),
            "popup_window_focus_function" => Some(("focus", 0)),
            "popup_window_print_function" => Some(("print", 0)),
            "popup_document_open_function" => Some(("open", 0)),
            "popup_document_write_function" => Some(("write", 0)),
            "popup_document_close_function" => Some(("close", 0)),
            "global_css_escape" => Some(("escape", 1)),
            "intl_collator_compare" => Some(("compare", 2)),
            "intl_date_time_format" => Some(("format", 1)),
            "intl_duration_format" => Some(("format", 1)),
            "intl_list_format" => Some(("format", 1)),
            "intl_number_format" => Some(("format", 1)),
            "clipboard_item_constructor" => Some(("ClipboardItem", 1)),
            "clipboard_write" => Some(("write", 1)),
            "request_constructor" => Some(("Request", 1)),
            "file_constructor" => Some(("File", 2)),
            "headers_constructor" => Some(("Headers", 0)),
            "worker_constructor" => Some(("Worker", 1)),
            "data_transfer_constructor" => Some(("DataTransfer", 0)),
            "option_constructor" => Some(("Option", 0)),
            "audio_constructor" => Some(("Audio", 0)),
            "text_encoder_constructor" => Some(("TextEncoder", 0)),
            "text_decoder_constructor" => Some(("TextDecoder", 0)),
            "text_encoder_stream_constructor" => Some(("TextEncoderStream", 0)),
            "text_decoder_stream_constructor" => Some(("TextDecoderStream", 0)),
            "css_style_sheet_constructor" => Some(("CSSStyleSheet", 0)),
            "text_encoder_get_encoding" => Some(("encoding", 0)),
            "text_encoder_encode" => Some(("encode", 0)),
            "text_encoder_encode_into" => Some(("encodeInto", 2)),
            "text_decoder_get_encoding" => Some(("encoding", 0)),
            "text_decoder_get_fatal" => Some(("fatal", 0)),
            "text_decoder_get_ignore_bom" => Some(("ignoreBOM", 0)),
            "text_decoder_decode" => Some(("decode", 0)),
            "text_encoder_stream_get_encoding" => Some(("encoding", 0)),
            "text_encoder_stream_get_readable" => Some(("readable", 0)),
            "text_encoder_stream_get_writable" => Some(("writable", 0)),
            "text_decoder_stream_get_encoding" => Some(("encoding", 0)),
            "text_decoder_stream_get_fatal" => Some(("fatal", 0)),
            "text_decoder_stream_get_ignore_bom" => Some(("ignoreBOM", 0)),
            "text_decoder_stream_get_readable" => Some(("readable", 0)),
            "text_decoder_stream_get_writable" => Some(("writable", 0)),
            "class_list_add" => Some(("add", 1)),
            "class_list_remove" => Some(("remove", 1)),
            "class_list_toggle" => Some(("toggle", 1)),
            "class_list_contains" => Some(("contains", 1)),
            "class_list_replace" => Some(("replace", 2)),
            "class_list_item" => Some(("item", 1)),
            "class_list_for_each" => Some(("forEach", 1)),
            "class_list_keys" => Some(("keys", 0)),
            "class_list_values" => Some(("values", 0)),
            "class_list_entries" => Some(("entries", 0)),
            "class_list_to_string" => Some(("toString", 0)),
            "named_node_map_item" => Some(("item", 1)),
            "named_node_map_get_named_item" => Some(("getNamedItem", 1)),
            "named_node_map_set_named_item" => Some(("setNamedItem", 1)),
            "named_node_map_remove_named_item" => Some(("removeNamedItem", 1)),
            "named_node_map_get_named_item_ns" => Some(("getNamedItemNS", 2)),
            "named_node_map_set_named_item_ns" => Some(("setNamedItemNS", 1)),
            "named_node_map_remove_named_item_ns" => Some(("removeNamedItemNS", 2)),
            "named_node_map_for_each" => Some(("forEach", 1)),
            "named_node_map_keys" => Some(("keys", 0)),
            "named_node_map_values" => Some(("values", 0)),
            "named_node_map_entries" => Some(("entries", 0)),
            _ => None,
        }
    }

    fn static_object_method_name_and_length(value: &Value) -> Option<(String, i64)> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        let method = match Self::object_get_entry(&entries, INTERNAL_STATIC_METHOD_NAME_KEY) {
            Some(Value::String(method)) => method,
            _ => return None,
        };
        let length = match method.as_str() {
            "create" => 2,
            "assign" => 2,
            "getOwnPropertyDescriptor" => 2,
            "defineProperty" => 3,
            "getOwnPropertyNames" => 1,
            "getOwnPropertySymbols" => 1,
            "keys" => 1,
            "values" => 1,
            "entries" => 1,
            "hasOwn" => 2,
            "getPrototypeOf" => 1,
            "setPrototypeOf" => 2,
            "freeze" => 1,
            _ => return None,
        };
        Some((method, length))
    }

    fn static_reflect_method_name_and_length(value: &Value) -> Option<(String, i64)> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        let method = match Self::object_get_entry(&entries, INTERNAL_STATIC_METHOD_NAME_KEY) {
            Some(Value::String(method)) => method,
            _ => return None,
        };
        let length = match method.as_str() {
            "set" => 3,
            "ownKeys" => 1,
            _ => return None,
        };
        Some((method, length))
    }

    fn receiver_builtin_callable_name_and_length(value: &Value) -> Option<(String, i64)> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        let family = match Self::object_get_entry(&entries, "__bt_receiver_builtin_family") {
            Some(Value::String(family)) => family,
            _ => return None,
        };
        let member = match Self::object_get_entry(&entries, "__bt_receiver_builtin_member") {
            Some(Value::String(member)) => member,
            _ => return None,
        };
        let (name, length) = match (family.as_str(), member.as_str()) {
            ("worker", "postMessage") => ("postMessage", 1),
            ("worker", "terminate") => ("terminate", 0),
            ("boolean", "toString") => ("toString", 0),
            ("boolean", "valueOf") => ("valueOf", 0),
            ("number", "toExponential") => ("toExponential", 1),
            ("number", "toFixed") => ("toFixed", 1),
            ("number", "toLocaleString") => ("toLocaleString", 0),
            ("number", "toPrecision") => ("toPrecision", 1),
            ("number", "toString") => ("toString", 1),
            ("number", "valueOf") => ("valueOf", 0),
            ("bigint", "toLocaleString") => ("toLocaleString", 0),
            ("bigint", "toString") => ("toString", 1),
            ("bigint", "valueOf") => ("valueOf", 0),
            ("symbol", "toString") => ("toString", 0),
            ("symbol", "valueOf") => ("valueOf", 0),
            ("string", "at") => ("at", 1),
            ("string", "charAt") => ("charAt", 1),
            ("string", "charCodeAt") => ("charCodeAt", 1),
            ("string", "concat") => ("concat", 1),
            ("string", "codePointAt") => ("codePointAt", 1),
            ("string", "endsWith") => ("endsWith", 1),
            ("string", "includes") => ("includes", 1),
            ("string", "indexOf") => ("indexOf", 1),
            ("string", "isWellFormed") => ("isWellFormed", 0),
            ("string", "lastIndexOf") => ("lastIndexOf", 1),
            ("string", "localeCompare") => ("localeCompare", 1),
            ("string", "match") => ("match", 1),
            ("string", "matchAll") => ("matchAll", 1),
            ("string", "normalize") => ("normalize", 0),
            ("string", "padEnd") => ("padEnd", 1),
            ("string", "padStart") => ("padStart", 1),
            ("string", "replace") => ("replace", 2),
            ("string", "replaceAll") => ("replaceAll", 2),
            ("string", "repeat") => ("repeat", 1),
            ("string", "search") => ("search", 1),
            ("string", "slice") => ("slice", 2),
            ("string", "split") => ("split", 2),
            ("string", "startsWith") => ("startsWith", 1),
            ("string", "substring") => ("substring", 2),
            ("string", "toLocaleLowerCase") => ("toLocaleLowerCase", 0),
            ("string", "toLocaleUpperCase") => ("toLocaleUpperCase", 0),
            ("string", "toLowerCase") => ("toLowerCase", 0),
            ("string", "toString") => ("toString", 0),
            ("string", "toUpperCase") => ("toUpperCase", 0),
            ("string", "toWellFormed") => ("toWellFormed", 0),
            ("string", "trim") => ("trim", 0),
            ("string", "trimEnd") => ("trimEnd", 0),
            ("string", "trimStart") => ("trimStart", 0),
            ("string", "valueOf") => ("valueOf", 0),
            ("node", "append") => ("append", 0),
            ("node", "prepend") => ("prepend", 0),
            ("node", "replaceChildren") => ("replaceChildren", 0),
            ("node", "before") => ("before", 0),
            ("node", "after") => ("after", 0),
            ("node", "replaceWith") => ("replaceWith", 0),
            ("node", "remove") => ("remove", 0),
            ("node", "appendChild") => ("appendChild", 1),
            ("node", "insertBefore") => ("insertBefore", 2),
            ("node", "removeChild") => ("removeChild", 1),
            ("node", "replaceChild") => ("replaceChild", 2),
            ("node", "hasChildNodes") => ("hasChildNodes", 0),
            ("node", "contains") => ("contains", 1),
            ("node", "getRootNode") => ("getRootNode", 0),
            ("node", "compareDocumentPosition") => ("compareDocumentPosition", 1),
            ("node", "isEqualNode") => ("isEqualNode", 1),
            ("node", "isSameNode") => ("isSameNode", 1),
            ("node", "normalize") => ("normalize", 0),
            ("node", "isDefaultNamespace") => ("isDefaultNamespace", 1),
            ("node", "lookupPrefix") => ("lookupPrefix", 1),
            ("node", "lookupNamespaceURI") => ("lookupNamespaceURI", 1),
            ("node", "cloneNode") => ("cloneNode", 0),
            ("node", "querySelector") => ("querySelector", 1),
            ("node", "querySelectorAll") => ("querySelectorAll", 1),
            ("node", "getAttributeNames") => ("getAttributeNames", 0),
            ("node", "toggleAttribute") => ("toggleAttribute", 1),
            ("node", "matches") => ("matches", 1),
            ("node", "closest") => ("closest", 1),
            ("node", "insertAdjacentElement") => ("insertAdjacentElement", 2),
            ("node", "insertAdjacentHTML") => ("insertAdjacentHTML", 2),
            ("node", "insertAdjacentText") => ("insertAdjacentText", 2),
            ("node", "setHTMLUnsafe") => ("setHTMLUnsafe", 1),
            ("node_list", "item") => ("item", 1),
            ("node_list", "namedItem") => ("namedItem", 1),
            ("node_list", "forEach") => ("forEach", 1),
            ("node_list", "entries") => ("entries", 0),
            ("node_list", "keys") => ("keys", 0),
            ("node_list", "values") => ("values", 0),
            ("image_bitmap", "width_get") => ("get width", 0),
            ("image_bitmap", "height_get") => ("get height", 0),
            ("image_bitmap", "close") => ("close", 0),
            ("text_track", "id_get") => ("get id", 0),
            ("text_track", "kind_get") => ("get kind", 0),
            ("text_track", "label_get") => ("get label", 0),
            ("text_track", "language_get") => ("get language", 0),
            ("text_track", "mode_get") => ("get mode", 0),
            ("text_track", "mode_set") => ("set mode", 1),
            ("text_track", "cues_get") => ("get cues", 0),
            ("text_track", "active_cues_get") => ("get activeCues", 0),
            ("text_track", "in_band_metadata_track_dispatch_type_get") => {
                ("get inBandMetadataTrackDispatchType", 0)
            }
            ("time_ranges", "length_get") => ("get length", 0),
            ("time_ranges", "start") => ("start", 1),
            ("time_ranges", "end") => ("end", 1),
            ("animation", "cancel") => ("cancel", 0),
            ("animation", "finish") => ("finish", 0),
            ("animation", "pause") => ("pause", 0),
            ("animation", "play") => ("play", 0),
            ("animation", "reverse") => ("reverse", 0),
            ("animation", "updatePlaybackRate") => ("updatePlaybackRate", 1),
            ("animation", "commitStyles") => ("commitStyles", 0),
            ("animation", "persist") => ("persist", 0),
            ("radio_node_list", "value_get") => ("get value", 0),
            ("radio_node_list", "value_set") => ("set value", 1),
            ("html_form", "submit") => ("submit", 0),
            ("html_form", "requestSubmit") => ("requestSubmit", 1),
            ("html_form", "reset") => ("reset", 0),
            ("html_form", "checkValidity") => ("checkValidity", 0),
            ("html_form", "reportValidity") => ("reportValidity", 0),
            ("html_media", "play") => ("play", 0),
            ("html_media", "pause") => ("pause", 0),
            ("html_media", "load") => ("load", 0),
            ("html_media", "canPlayType") => ("canPlayType", 1),
            ("html_media", "fastSeek") => ("fastSeek", 1),
            ("html_collection", "item") => ("item", 1),
            ("html_collection", "namedItem") => ("namedItem", 1),
            ("html_collection", "forEach") => ("forEach", 1),
            ("html_collection", "entries") => ("entries", 0),
            ("html_collection", "keys") => ("keys", 0),
            ("html_collection", "values") => ("values", 0),
            ("date", "getTime") => ("getTime", 0),
            ("date", "setTime") => ("setTime", 1),
            ("date", "toISOString") => ("toISOString", 0),
            ("date", "toLocaleDateString") => ("toLocaleDateString", 0),
            ("date", "toString") => ("toString", 0),
            ("date", "valueOf") => ("valueOf", 0),
            ("date", "getUTCFullYear") => ("getUTCFullYear", 0),
            ("date", "getUTCMonth") => ("getUTCMonth", 0),
            ("date", "getUTCDate") => ("getUTCDate", 0),
            ("date", "getUTCDay") => ("getUTCDay", 0),
            ("date", "getUTCHours") => ("getUTCHours", 0),
            ("date", "getUTCMinutes") => ("getUTCMinutes", 0),
            ("date", "getUTCSeconds") => ("getUTCSeconds", 0),
            ("date", "getUTCMilliseconds") => ("getUTCMilliseconds", 0),
            ("date", "getFullYear") => ("getFullYear", 0),
            ("date", "getMonth") => ("getMonth", 0),
            ("date", "getDate") => ("getDate", 0),
            ("date", "getHours") => ("getHours", 0),
            ("date", "getMinutes") => ("getMinutes", 0),
            ("date", "getSeconds") => ("getSeconds", 0),
            ("regexp", "source") => ("get source", 0),
            ("regexp", "flags") => ("get flags", 0),
            ("regexp", "global") => ("get global", 0),
            ("regexp", "ignoreCase") => ("get ignoreCase", 0),
            ("regexp", "multiline") => ("get multiline", 0),
            ("regexp", "dotAll") => ("get dotAll", 0),
            ("regexp", "sticky") => ("get sticky", 0),
            ("regexp", "hasIndices") => ("get hasIndices", 0),
            ("regexp", "unicode") => ("get unicode", 0),
            ("regexp", "unicodeSets") => ("get unicodeSets", 0),
            ("regexp", "exec") => ("exec", 1),
            ("regexp", "test") => ("test", 1),
            ("regexp", "toString") => ("toString", 0),
            ("regexp", "match") => ("[Symbol.match]", 1),
            ("regexp", "matchAll") => ("[Symbol.matchAll]", 1),
            ("regexp", "replace") => ("[Symbol.replace]", 2),
            ("regexp", "search") => ("[Symbol.search]", 1),
            ("regexp", "split") => ("[Symbol.split]", 2),
            ("intl_collator", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_date_time_format", "formatToParts") => ("formatToParts", 0),
            ("intl_date_time_format", "formatRange") => ("formatRange", 2),
            ("intl_date_time_format", "formatRangeToParts") => ("formatRangeToParts", 2),
            ("intl_date_time_format", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_display_names", "of") => ("of", 1),
            ("intl_display_names", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_duration_format", "formatToParts") => ("formatToParts", 1),
            ("intl_duration_format", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_list_format", "formatToParts") => ("formatToParts", 1),
            ("intl_list_format", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_locale", "getCalendars") => ("getCalendars", 0),
            ("intl_locale", "getCollations") => ("getCollations", 0),
            ("intl_locale", "getHourCycles") => ("getHourCycles", 0),
            ("intl_locale", "getNumberingSystems") => ("getNumberingSystems", 0),
            ("intl_locale", "getTextInfo") => ("getTextInfo", 0),
            ("intl_locale", "getTimeZones") => ("getTimeZones", 0),
            ("intl_locale", "getWeekInfo") => ("getWeekInfo", 0),
            ("intl_locale", "maximize") => ("maximize", 0),
            ("intl_locale", "minimize") => ("minimize", 0),
            ("intl_locale", "toString") => ("toString", 0),
            ("intl_number_format", "formatToParts") => ("formatToParts", 1),
            ("intl_number_format", "formatRange") => ("formatRange", 2),
            ("intl_number_format", "formatRangeToParts") => ("formatRangeToParts", 2),
            ("intl_number_format", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_plural_rules", "select") => ("select", 1),
            ("intl_plural_rules", "selectRange") => ("selectRange", 2),
            ("intl_plural_rules", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_relative_time_format", "format") => ("format", 2),
            ("intl_relative_time_format", "formatToParts") => ("formatToParts", 2),
            ("intl_relative_time_format", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_segmenter", "segment") => ("segment", 1),
            ("intl_segmenter", "resolvedOptions") => ("resolvedOptions", 0),
            ("object", "hasOwnProperty") => ("hasOwnProperty", 1),
            ("object", "isPrototypeOf") => ("isPrototypeOf", 1),
            ("object", "propertyIsEnumerable") => ("propertyIsEnumerable", 1),
            ("object", "toString") => ("toString", 0),
            ("object", "valueOf") => ("valueOf", 0),
            ("document", "createElement") => ("createElement", 1),
            ("document", "createElementNS") => ("createElementNS", 2),
            ("document", "createTextNode") => ("createTextNode", 1),
            ("document", "createAttribute") => ("createAttribute", 1),
            ("document", "createDocumentFragment") => ("createDocumentFragment", 0),
            ("document", "createRange") => ("createRange", 0),
            ("document", "getSelection") => ("getSelection", 0),
            ("document", "append") => ("append", 0),
            ("document", "getElementById") => ("getElementById", 1),
            ("document", "getElementsByClassName") => ("getElementsByClassName", 1),
            ("document", "getElementsByName") => ("getElementsByName", 1),
            ("document", "getElementsByTagName") => ("getElementsByTagName", 1),
            ("document", "getElementsByTagNameNS") => ("getElementsByTagNameNS", 2),
            ("document", "querySelector") => ("querySelector", 1),
            ("document", "querySelectorAll") => ("querySelectorAll", 1),
            ("document", "createTreeWalker") => ("createTreeWalker", 1),
            ("document", "addEventListener") => ("addEventListener", 2),
            ("document", "removeEventListener") => ("removeEventListener", 2),
            ("parsed_document", "createTreeWalker") => ("createTreeWalker", 1),
            ("parsed_document", "querySelector") => ("querySelector", 1),
            ("parsed_document", "querySelectorAll") => ("querySelectorAll", 1),
            ("parsed_document", "getElementById") => ("getElementById", 1),
            ("parsed_document", "getElementsByClassName") => ("getElementsByClassName", 1),
            ("parsed_document", "getElementsByName") => ("getElementsByName", 1),
            ("parsed_document", "getElementsByTagName") => ("getElementsByTagName", 1),
            ("parsed_document", "createElement") => ("createElement", 1),
            ("parsed_document", "createElementNS") => ("createElementNS", 2),
            ("parsed_document", "createTextNode") => ("createTextNode", 1),
            ("parsed_document", "createAttribute") => ("createAttribute", 1),
            ("parsed_document", "createDocumentFragment") => ("createDocumentFragment", 0),
            ("parsed_document", "createRange") => ("createRange", 0),
            ("parsed_document", "append") => ("append", 0),
            ("dom_parser", "parseFromString") => ("parseFromString", 2),
            ("xml_serializer", "serializeToString") => ("serializeToString", 1),
            ("tree_walker", "nextNode") => ("nextNode", 0),
            ("range", "setStart") => ("setStart", 2),
            ("range", "setEnd") => ("setEnd", 2),
            ("selection", "addRange") => ("addRange", 1),
            ("selection", "collapse") => ("collapse", 1),
            ("selection", "collapseToEnd") => ("collapseToEnd", 0),
            ("selection", "collapseToStart") => ("collapseToStart", 0),
            ("selection", "containsNode") => ("containsNode", 1),
            ("selection", "deleteFromDocument") => ("deleteFromDocument", 0),
            ("selection", "empty") => ("empty", 0),
            ("selection", "extend") => ("extend", 2),
            ("selection", "getComposedRanges") => ("getComposedRanges", 0),
            ("selection", "getRangeAt") => ("getRangeAt", 1),
            ("selection", "modify") => ("modify", 3),
            ("selection", "removeAllRanges") => ("removeAllRanges", 0),
            ("selection", "removeRange") => ("removeRange", 1),
            ("selection", "selectAllChildren") => ("selectAllChildren", 1),
            ("selection", "setBaseAndExtent") => ("setBaseAndExtent", 4),
            ("selection", "setPosition") => ("setPosition", 1),
            ("selection", "toString") => ("toString", 0),
            ("event_target", "addEventListener") => ("addEventListener", 2),
            ("event_target", "removeEventListener") => ("removeEventListener", 2),
            ("event_target", "dispatchEvent") => ("dispatchEvent", 1),
            ("event", "preventDefault") => ("preventDefault", 0),
            ("event", "stopPropagation") => ("stopPropagation", 0),
            ("event", "stopImmediatePropagation") => ("stopImmediatePropagation", 0),
            ("keyboard_event", "getModifierState") => ("getModifierState", 1),
            ("pointer_event", "getCoalescedEvents") => ("getCoalescedEvents", 0),
            ("pointer_event", "getPredictedEvents") => ("getPredictedEvents", 0),
            ("navigate_event", "intercept") => ("intercept", 1),
            ("navigate_event", "scroll") => ("scroll", 0),
            ("data_transfer", "getData") => ("getData", 1),
            ("data_transfer", "setData") => ("setData", 2),
            ("data_transfer", "clearData") => ("clearData", 0),
            ("data_transfer", "setDragImage") => ("setDragImage", 3),
            ("data_transfer", "addElement") => ("addElement", 1),
            ("data_transfer_item", "getAsFile") => ("getAsFile", 0),
            ("data_transfer_item", "getAsFileSystemHandle") => ("getAsFileSystemHandle", 0),
            ("data_transfer_item", "getAsString") => ("getAsString", 1),
            ("data_transfer_item", "webkitGetAsEntry") => ("webkitGetAsEntry", 0),
            ("data_transfer_item_list", "add") => ("add", 1),
            ("data_transfer_item_list", "remove") => ("remove", 1),
            ("data_transfer_item_list", "clear") => ("clear", 0),
            ("match_media", "addEventListener") => ("addEventListener", 2),
            ("match_media", "removeEventListener") => ("removeEventListener", 2),
            ("match_media", "dispatchEvent") => ("dispatchEvent", 1),
            ("match_media", "addListener") => ("addListener", 1),
            ("match_media", "removeListener") => ("removeListener", 1),
            ("cookie_store", "set") => ("set", 1),
            ("cookie_store", "get") => ("get", 1),
            ("cookie_store", "getAll") => ("getAll", 1),
            ("cookie_store", "delete") => ("delete", 1),
            ("cookie_store", "addEventListener") => ("addEventListener", 2),
            ("cookie_store", "removeEventListener") => ("removeEventListener", 2),
            ("cache_storage", "open") => ("open", 1),
            ("cache_storage", "match") => ("match", 1),
            ("cache_storage", "has") => ("has", 1),
            ("cache_storage", "delete") => ("delete", 1),
            ("cache_storage", "keys") => ("keys", 0),
            ("cache", "match") => ("match", 1),
            ("cache", "put") => ("put", 2),
            ("cache", "delete") => ("delete", 1),
            ("cache", "keys") => ("keys", 0),
            ("cache", "add") => ("add", 1),
            ("cache", "addAll") => ("addAll", 1),
            ("canvas_2d_context", "toString") => ("toString", 0),
            _ => return None,
        };
        Some((name.to_string(), length))
    }

    fn callable_name_and_length(&mut self, value: &Value) -> Option<(String, i64)> {
        match value {
            Value::Function(function) => Some((
                self.function_display_name(function),
                Self::function_length(function),
            )),
            Value::StringConstructor => Some(("String".to_string(), 1)),
            Value::RegExpConstructor => Some(("RegExp".to_string(), 2)),
            Value::BlobConstructor => Some(("Blob".to_string(), 0)),
            Value::UrlConstructor => Some(("URL".to_string(), 1)),
            Value::ArrayBufferConstructor => Some(("ArrayBuffer".to_string(), 1)),
            Value::PromiseConstructor => Some(("Promise".to_string(), 1)),
            Value::MapConstructor => Some(("Map".to_string(), 0)),
            Value::WeakMapConstructor => Some(("WeakMap".to_string(), 0)),
            Value::SetConstructor => Some(("Set".to_string(), 0)),
            Value::WeakSetConstructor => Some(("WeakSet".to_string(), 0)),
            Value::UrlSearchParamsConstructor => Some(("URLSearchParams".to_string(), 0)),
            Value::SymbolConstructor => Some(("Symbol".to_string(), 0)),
            Value::TypedArrayConstructor(kind) => Some((
                match kind {
                    TypedArrayConstructorKind::Concrete(kind) => kind.name(),
                    TypedArrayConstructorKind::Abstract => "TypedArray",
                }
                .to_string(),
                3,
            )),
            Value::Object(_) => match Self::callable_kind_from_value(value) {
                Some("bound_function") => {
                    let (target, _bound_this, bound_args) =
                        Self::bound_callable_components(value).ok()?;
                    let (target_name, target_length) = self.callable_name_and_length(&target)?;
                    let bound_name = format!("bound {target_name}");
                    let bound_length = target_length.saturating_sub(bound_args.len() as i64).max(0);
                    Some((bound_name, bound_length))
                }
                Some("receiver_builtin_method") => {
                    Self::receiver_builtin_callable_name_and_length(value)
                }
                Some("object_static_method") => Self::static_object_method_name_and_length(value),
                Some("reflect_static_method") => Self::static_reflect_method_name_and_length(value),
                Some(kind) => Self::object_backed_callable_name_and_length(kind)
                    .map(|(name, length)| (name.to_string(), length)),
                None => None,
            },
            _ => None,
        }
    }

    pub(crate) fn callable_source_text(&mut self, value: &Value) -> Option<String> {
        match value {
            Value::Function(function) if function.function_id != usize::MAX => {
                return Some(format!("__bt_function_ref__({})", function.function_id));
            }
            _ if !self.is_callable_value(value) => return None,
            Value::Object(_)
                if matches!(
                    Self::callable_kind_from_value(value),
                    Some("bound_function")
                ) =>
            {
                return Some("function () { [native code] }".to_string());
            }
            _ => {}
        }

        let name = self
            .callable_name_and_length(value)
            .map(|(name, _)| name)
            .unwrap_or_default();
        if name.is_empty() {
            Some("function () { [native code] }".to_string())
        } else {
            Some(format!("function {name}() {{ [native code] }}"))
        }
    }

    fn coerce_object_like_to_string_via_primitive_methods(
        &mut self,
        value: &Value,
        allow_symbol: bool,
    ) -> Result<String> {
        let mut saw_callable = false;
        for method_name in ["toString", "valueOf"] {
            let method = self.object_property_from_value(value, method_name)?;
            if !self.is_callable_value(&method) {
                continue;
            }
            saw_callable = true;
            let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
            let coerced = self.execute_callable_value_with_this_and_env(
                &method,
                &[],
                &event,
                None,
                Some(value.clone()),
            )?;
            if Self::is_primitive_value(&coerced) {
                if matches!(coerced, Value::Symbol(_)) {
                    if !allow_symbol {
                        return Err(Error::ScriptRuntime(
                            "Cannot convert a Symbol value to a string".into(),
                        ));
                    }
                }
                return Ok(self.coerce_to_string_for_string_context(&coerced));
            }
        }
        if saw_callable {
            return Err(Error::ScriptRuntime(
                "Cannot convert object to primitive value".into(),
            ));
        }
        Ok(self.coerce_to_string_for_string_context(value))
    }

    pub(crate) fn coerce_to_string_for_tostring(&mut self, value: &Value) -> Result<String> {
        match value {
            Value::Symbol(_) => Err(Error::ScriptRuntime(
                "Cannot convert a Symbol value to a string".into(),
            )),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(wrapped) = Self::string_wrapper_value_from_object(&entries) {
                    return Ok(wrapped);
                }
                if Self::symbol_wrapper_id_from_object(&entries).is_some() {
                    return Err(Error::ScriptRuntime(
                        "Cannot convert a Symbol value to a string".into(),
                    ));
                }
                drop(entries);
                self.coerce_object_like_to_string_via_primitive_methods(value, false)
            }
            Value::Date(_) => self.coerce_object_like_to_string_via_primitive_methods(value, false),
            _ => Ok(self.coerce_to_string_for_string_context(value)),
        }
    }

    pub(crate) fn coerce_to_string_for_string_constructor(
        &mut self,
        value: &Value,
    ) -> Result<String> {
        match value {
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(wrapped) = Self::string_wrapper_value_from_object(&entries) {
                    return Ok(wrapped);
                }
                drop(entries);
                self.coerce_object_like_to_string_via_primitive_methods(value, true)
            }
            Value::Date(_) => self.coerce_object_like_to_string_via_primitive_methods(value, true),
            _ => Ok(self.coerce_to_string_for_string_context(value)),
        }
    }

    pub(crate) fn coerce_to_string_for_string_context(&mut self, value: &Value) -> String {
        self.callable_source_text(value)
            .unwrap_or_else(|| value.as_string())
    }

    pub(crate) fn callable_function_surface_value(
        &mut self,
        value: &Value,
        key: &str,
    ) -> Option<Value> {
        match key {
            "call" | "apply" | "bind" | "toString" => {
                return Some(self.cached_function_surface_method_value(key));
            }
            "name" => {
                let (name, _) = self.callable_name_and_length(value)?;
                return Some(Value::String(name));
            }
            "length" => {
                let (_, length) = self.callable_name_and_length(value)?;
                return Some(Value::Number(length));
            }
            _ => {}
        }
        None
    }

    pub(crate) fn variant_callable_public_storage_key(value: &Value) -> Option<String> {
        match value {
            Value::StringConstructor => Some("String".to_string()),
            Value::SymbolConstructor => Some("Symbol".to_string()),
            Value::MapConstructor => Some("Map".to_string()),
            Value::WeakMapConstructor => Some("WeakMap".to_string()),
            Value::SetConstructor => Some("Set".to_string()),
            Value::WeakSetConstructor => Some("WeakSet".to_string()),
            Value::PromiseConstructor => Some("Promise".to_string()),
            Value::BlobConstructor => Some("Blob".to_string()),
            Value::ArrayBufferConstructor => Some("ArrayBuffer".to_string()),
            Value::RegExpConstructor => Some("RegExp".to_string()),
            Value::UrlSearchParamsConstructor => Some("URLSearchParams".to_string()),
            Value::TypedArrayConstructor(kind) => Some(format!(
                "TypedArrayConstructor:{}",
                match kind {
                    TypedArrayConstructorKind::Concrete(kind) => kind.name(),
                    TypedArrayConstructorKind::Abstract => "TypedArray",
                }
            )),
            _ => None,
        }
    }

    pub(crate) fn variant_callable_internal_prototype_value(&self, value: &Value) -> Option<Value> {
        let storage_key = Self::variant_callable_public_storage_key(value)?;
        let entries = self
            .script_runtime
            .variant_callable_public_properties
            .get(&storage_key)?;
        Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
    }

    pub(crate) fn new_string_wrapper_value(value: String) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
            Value::String(value),
        )])
    }

    pub(crate) fn new_boolean_wrapper_value(value: bool) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_BOOLEAN_WRAPPER_VALUE_KEY.to_string(),
            Value::Bool(value),
        )])
    }

    pub(crate) fn new_number_wrapper_value(value: Value) -> Value {
        Self::new_object_value(vec![(INTERNAL_NUMBER_WRAPPER_VALUE_KEY.to_string(), value)])
    }

    pub(crate) fn new_bigint_wrapper_value(value: JsBigInt) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_BIGINT_WRAPPER_VALUE_KEY.to_string(),
            Value::BigInt(value),
        )])
    }

    pub(crate) fn new_symbol_wrapper_value(symbol_id: usize) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_SYMBOL_WRAPPER_KEY.to_string(),
            Value::Number(symbol_id as i64),
        )])
    }

    pub(crate) fn box_primitive_value(value: Value) -> Value {
        match value {
            Value::String(text) => Self::new_string_wrapper_value(text),
            Value::Bool(value) => Self::new_boolean_wrapper_value(value),
            Value::Number(value) => Self::new_number_wrapper_value(Value::Number(value)),
            Value::Float(value) => Self::new_number_wrapper_value(Value::Float(value)),
            Value::BigInt(value) => Self::new_bigint_wrapper_value(value),
            Value::Symbol(symbol) => Self::new_symbol_wrapper_value(symbol.id),
            other => other,
        }
    }

    pub(crate) fn function_own_property_value(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
        include_to_string: bool,
    ) -> Value {
        match key {
            "constructor" => {
                if function.is_generator {
                    if function.is_async {
                        self.new_async_generator_function_constructor_value()
                    } else {
                        self.new_generator_function_constructor_value()
                    }
                } else {
                    Value::Undefined
                }
            }
            "prototype" => {
                if function.is_arrow || function.is_method {
                    Value::Undefined
                } else {
                    Value::Object(function.prototype_object.clone())
                }
            }
            "length" => Value::Number(Self::function_length(function)),
            "name" => Value::String(self.function_display_name(function)),
            "call" | "apply" | "bind" => self.cached_function_surface_method_value(key),
            "toString" if include_to_string => self.cached_function_surface_method_value(key),
            _ => Value::Undefined,
        }
    }
}
