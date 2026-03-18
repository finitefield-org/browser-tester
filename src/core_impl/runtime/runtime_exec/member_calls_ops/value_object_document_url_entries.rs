use super::*;

impl Harness {
    fn object_property_from_url_search_params_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_url_search_params_object(entries) {
            return None;
        }
        if key == "size" {
            let size = Self::url_search_params_pairs_from_object_entries(entries).len();
            return Some(Value::Number(size as i64));
        }
        if self.is_iterator_property_key(key) {
            return Some(Self::new_receiver_builtin_callable(
                "url_search_params",
                "entries",
            ));
        }
        if matches!(
            key,
            "append"
                | "delete"
                | "get"
                | "getAll"
                | "has"
                | "set"
                | "sort"
                | "forEach"
                | "entries"
                | "keys"
                | "values"
                | "toString"
        ) {
            return Some(Self::new_receiver_builtin_callable(
                "url_search_params",
                key,
            ));
        }
        None
    }

    fn object_property_from_storage_entries(entries: &ObjectValue, key: &str) -> Option<Value> {
        if !Self::is_storage_object(entries) {
            return None;
        }
        if key == "length" {
            let len = Self::storage_pairs_from_object_entries(entries).len();
            return Some(Value::Number(len as i64));
        }
        if let Some(value) = Self::object_get_entry(entries, key) {
            return Some(value);
        }
        if Self::is_storage_method_name(key) {
            return Some(Self::new_receiver_builtin_callable("storage", key));
        }
        if let Some((_, value)) = Self::storage_pairs_from_object_entries(entries)
            .into_iter()
            .find(|(name, _)| name == key)
        {
            return Some(Value::String(value));
        }
        None
    }

    fn object_property_from_document_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        let is_document_object = matches!(
            Self::object_get_entry(entries, INTERNAL_DOCUMENT_OBJECT_KEY),
            Some(Value::Bool(true))
        );
        if !is_document_object {
            return None;
        }
        if let Some(value) = Self::placeholder_backed_object_builtin_property_value(entries, key) {
            return Some(value);
        }
        let value = match key {
            "nodeType" => Value::Number(self.node_type_number(self.dom.root)),
            "textContent" => self.node_text_content_value(self.dom.root),
            "body" => self.dom.body().map(Value::Node).unwrap_or(Value::Null),
            "head" => self.dom.head().map(Value::Node).unwrap_or(Value::Null),
            "documentElement" => self
                .dom
                .document_element()
                .map(Value::Node)
                .unwrap_or(Value::Null),
            "forms" => self.document_forms_live_list_value(),
            "images" => self.document_images_live_list_value(),
            "links" => self.document_links_live_list_value(),
            "scripts" => self.document_scripts_live_list_value(),
            "readyState" => Value::String(self.dom_runtime.document_ready_state.clone()),
            "cookie" => Value::String(self.document_cookie_string()),
            "hidden" => Value::Bool(self.dom_runtime.document_visibility_state == "hidden"),
            "visibilityState" => Value::String(self.dom_runtime.document_visibility_state.clone()),
            "adoptedStyleSheets" => self.ensure_document_adopted_style_sheets_property(),
            _ if key.starts_with("on") => self
                .dom_runtime
                .node_expando_props
                .get(&(self.dom.root, key.to_string()))
                .cloned()
                .unwrap_or(Value::Null),
            _ => Value::Undefined,
        };
        if matches!(value, Value::Undefined) {
            None
        } else {
            Some(value)
        }
    }

    pub(crate) fn object_property_from_range_or_selection_entries(
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        Self::placeholder_backed_object_builtin_property_value(entries, key)
    }

    pub(crate) fn object_property_from_cookie_store_or_cache_entries(
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        Self::placeholder_backed_object_builtin_property_value(entries, key)
    }

    fn object_property_from_url_entries(entries: &ObjectValue, key: &str) -> Option<Value> {
        if !Self::is_url_object(entries) {
            return None;
        }
        if key == "constructor" {
            return Some(Value::UrlConstructor);
        }
        if matches!(key, "toString" | "toJSON") {
            return Some(Self::new_receiver_builtin_callable("url", key));
        }
        None
    }

    pub(crate) fn object_property_from_storage_document_and_url_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if let Some(value) = self.object_property_from_url_search_params_entries(entries, key) {
            return Some(value);
        }
        if let Some(value) = Self::object_property_from_storage_entries(entries, key) {
            return Some(value);
        }
        if let Some(value) = self.object_property_from_document_entries(entries, key) {
            return Some(value);
        }
        Self::object_property_from_url_entries(entries, key)
    }
}
