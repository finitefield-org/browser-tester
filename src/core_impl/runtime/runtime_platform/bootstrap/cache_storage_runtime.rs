use super::*;

impl Harness {
    pub(crate) fn cache_storage_builtin_keys() -> &'static [&'static str] {
        &["open", "match", "has", "delete", "keys"]
    }

    pub(crate) fn cache_builtin_keys() -> &'static [&'static str] {
        &["match", "put", "delete", "keys", "add", "addAll"]
    }

    pub(crate) fn sync_cache_storage_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.browser_apis.cache_storage_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
                    continue;
                }
                if Self::cache_storage_builtin_keys()
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
                INTERNAL_CACHE_STORAGE_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("open".to_string(), Self::new_builtin_placeholder_function()),
            ("match".to_string(), Self::new_builtin_placeholder_function()),
            ("has".to_string(), Self::new_builtin_placeholder_function()),
            ("delete".to_string(), Self::new_builtin_placeholder_function()),
            ("keys".to_string(), Self::new_builtin_placeholder_function()),
        ];
        entries.extend(extras);
        *self.browser_apis.cache_storage_object.borrow_mut() = entries.into();
    }

    pub(crate) fn cache_storage_global_value(&self) -> Value {
        if self.window_is_secure_context() {
            Value::Object(self.browser_apis.cache_storage_object.clone())
        } else {
            Value::Undefined
        }
    }

    pub(crate) fn is_cache_storage_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_CACHE_STORAGE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_cache_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_CACHE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn cache_name_from_object_entries(entries: &[(String, Value)]) -> Option<String> {
        match Self::object_get_entry(entries, INTERNAL_CACHE_NAME_KEY) {
            Some(Value::String(name)) => Some(name),
            Some(other) => Some(other.as_string()),
            None => None,
        }
    }

    pub(crate) fn ensure_cache_object(&mut self, cache_name: &str) -> Rc<RefCell<ObjectValue>> {
        if let Some(existing) = self.browser_apis.caches_by_name.get(cache_name).cloned() {
            return existing;
        }

        let mut cache_entries = vec![
            (INTERNAL_CACHE_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_CACHE_NAME_KEY.to_string(),
                Value::String(cache_name.to_string()),
            ),
        ];
        for builtin in Self::cache_builtin_keys() {
            cache_entries.push(((*builtin).to_string(), Self::new_builtin_placeholder_function()));
        }
        let cache_object = Rc::new(RefCell::new(ObjectValue::new(cache_entries)));

        self.browser_apis
            .caches_by_name
            .insert(cache_name.to_string(), cache_object.clone());
        if !self
            .browser_apis
            .cache_names_in_order
            .iter()
            .any(|name| name == cache_name)
        {
            self.browser_apis
                .cache_names_in_order
                .push(cache_name.to_string());
        }
        self.browser_apis
            .cache_entries_by_name
            .entry(cache_name.to_string())
            .or_default();
        cache_object
    }

    pub(crate) fn cache_storage_names_snapshot(&self) -> Vec<String> {
        self.browser_apis
            .cache_names_in_order
            .iter()
            .filter(|name| self.browser_apis.caches_by_name.contains_key((*name).as_str()))
            .cloned()
            .collect()
    }

    pub(crate) fn remove_named_cache(&mut self, cache_name: &str) -> bool {
        let existed = self.browser_apis.caches_by_name.remove(cache_name).is_some();
        self.browser_apis
            .cache_entries_by_name
            .remove(cache_name);
        self.browser_apis
            .cache_names_in_order
            .retain(|name| name != cache_name);
        existed
    }
}
