use super::*;

impl Harness {
    pub(crate) fn current_location_parts(&self) -> LocationParts {
        LocationParts::parse(&self.document_url).unwrap_or_else(|| LocationParts {
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
        })
    }

    pub(crate) fn window_is_secure_context(&self) -> bool {
        matches!(
            self.current_location_parts().scheme.as_str(),
            "https" | "wss"
        )
    }

    pub(crate) fn document_builtin_keys() -> &'static [&'static str] {
        &["defaultView", "location", "URL", "documentURI"]
    }

    pub(crate) fn sync_document_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.dom_runtime.document_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
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
        ];
        entries.extend(extras);
        *self.dom_runtime.document_object.borrow_mut() = entries.into();
    }

    pub(crate) fn window_builtin_keys() -> &'static [&'static str] {
        &[
            "window",
            "self",
            "top",
            "parent",
            "frames",
            "length",
            "closed",
            "location",
            "history",
            "navigator",
            "clientInformation",
            "localStorage",
            "document",
            "origin",
            "isSecureContext",
            "Intl",
            "String",
            "Boolean",
            "URL",
            "HTMLElement",
            "HTMLInputElement",
            "name",
        ]
    }

    pub(crate) fn sync_window_object(
        &mut self,
        navigator: &Value,
        intl: &Value,
        string_constructor: &Value,
        boolean_constructor: &Value,
        url_constructor: &Value,
        html_element_constructor: &Value,
        html_input_element_constructor: &Value,
        local_storage: &Value,
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
            ("window".to_string(), window_ref.clone()),
            ("self".to_string(), window_ref.clone()),
            ("top".to_string(), window_ref.clone()),
            ("parent".to_string(), window_ref.clone()),
            ("frames".to_string(), window_ref),
            ("length".to_string(), Value::Number(0)),
            ("closed".to_string(), Value::Bool(false)),
            (
                "location".to_string(),
                Value::Object(self.dom_runtime.location_object.clone()),
            ),
            (
                "history".to_string(),
                Value::Object(self.location_history.history_object.clone()),
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
            ("URL".to_string(), url_constructor.clone()),
            ("HTMLElement".to_string(), html_element_constructor.clone()),
            (
                "HTMLInputElement".to_string(),
                html_input_element_constructor.clone(),
            ),
            ("name".to_string(), name_value),
        ];
        entries.extend(extras);
        *self.dom_runtime.window_object.borrow_mut() = entries.into();
    }

    pub(crate) fn sync_window_runtime_properties(&mut self) {
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
            ("port".to_string(), Value::String(parts.port.clone())),
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
                Self::new_builtin_placeholder_function(),
            ),
            (
                "reload".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "replace".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "toString".to_string(),
                Self::new_builtin_placeholder_function(),
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
}
