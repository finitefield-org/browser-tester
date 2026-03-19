use super::*;

impl Harness {
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
