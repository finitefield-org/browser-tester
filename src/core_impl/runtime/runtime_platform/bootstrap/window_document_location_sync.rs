use super::*;

impl Harness {
    pub(crate) fn document_method_keys() -> &'static [&'static str] {
        &[
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

    fn preserved_placeholder_backed_property_state(
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Vec<(String, Value)>> {
        let stored = Self::object_get_entry(entries, key);
        let should_preserve = stored.as_ref().is_some_and(|value| {
            !Self::is_builtin_placeholder_value(value)
                || !Self::is_non_enumerable_object_key(entries, key)
                || !Self::is_writable_object_key(entries, key)
                || !Self::is_configurable_object_key(entries, key)
        }) || Self::has_object_accessor_property(entries, key)
            || Self::is_builtin_object_property_deleted(entries, key);
        if !should_preserve {
            return None;
        }

        let mut state = Vec::new();
        for state_key in [
            key.to_string(),
            Self::object_getter_storage_key(key),
            Self::object_setter_storage_key(key),
            Self::object_undefined_getter_storage_key(key),
            Self::object_undefined_setter_storage_key(key),
            Self::object_non_enumerable_storage_key(key),
            Self::object_non_writable_storage_key(key),
            Self::object_non_configurable_storage_key(key),
            Self::object_deleted_builtin_storage_key(key),
        ] {
            if let Some(value) = Self::object_get_entry(entries, &state_key) {
                state.push((state_key, value));
            }
        }
        Some(state)
    }

    fn restore_preserved_object_property_state(
        entries: &mut ObjectValue,
        key: &str,
        state: &[(String, Value)],
    ) {
        Self::delete_object_property_entries(entries, key);
        entries.delete_entry(&Self::object_deleted_builtin_storage_key(key));
        for (state_key, value) in state {
            Self::object_set_entry(entries, state_key.clone(), value.clone());
        }
    }

    pub(crate) fn sync_document_object(&mut self) {
        let mut extras = Vec::new();
        let mut adopted_style_sheets: Option<Value> = None;
        let mut preserved_method_states = Vec::new();
        {
            let entries = self.dom_runtime.document_object.borrow();
            for key in Self::document_method_keys() {
                if let Some(state) =
                    Self::preserved_placeholder_backed_property_state(&entries, key)
                {
                    preserved_method_states.push(((*key).to_string(), state));
                }
            }
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
        Self::mark_object_properties_non_enumerable(&mut entries, Self::document_method_keys());
        let mut entries = ObjectValue::new(entries);
        for (key, state) in preserved_method_states {
            Self::restore_preserved_object_property_state(&mut entries, &key, &state);
        }
        *self.dom_runtime.document_object.borrow_mut() = entries;
    }
}
