use super::*;

impl Harness {
    pub(crate) fn object_set_entry(entries: &mut impl ObjectEntryMut, key: String, value: Value) {
        entries.set_entry(key, value);
    }

    pub(crate) fn object_get_entry(
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> Option<Value> {
        entries.get_entry(key)
    }

    pub(crate) fn object_getter_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_OBJECT_GETTER_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_setter_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_OBJECT_SETTER_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_undefined_getter_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_OBJECT_UNDEFINED_GETTER_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_undefined_setter_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_OBJECT_UNDEFINED_SETTER_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_non_enumerable_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_NON_ENUMERABLE_PROPERTY_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn mark_object_properties_non_enumerable(
        entries: &mut impl ObjectEntryMut,
        keys: &[&str],
    ) {
        for key in keys {
            Self::object_set_entry(
                entries,
                Self::object_non_enumerable_storage_key(key),
                Value::Bool(true),
            );
        }
    }

    pub(crate) fn object_non_writable_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_NON_WRITABLE_PROPERTY_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_non_configurable_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_NON_CONFIGURABLE_PROPERTY_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_deleted_builtin_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_DELETED_BUILTIN_PROPERTY_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_getter_from_entries(
        entries: &(impl ObjectEntryLookup + ?Sized),
        property_key: &str,
    ) -> Option<Value> {
        let getter_key = Self::object_getter_storage_key(property_key);
        Self::object_get_entry(entries, &getter_key)
    }

    pub(crate) fn object_setter_from_entries(
        entries: &(impl ObjectEntryLookup + ?Sized),
        property_key: &str,
    ) -> Option<Value> {
        let setter_key = Self::object_setter_storage_key(property_key);
        Self::object_get_entry(entries, &setter_key)
    }

    pub(crate) fn has_object_getter_property(
        entries: &(impl ObjectEntryLookup + ?Sized),
        property_key: &str,
    ) -> bool {
        Self::object_getter_from_entries(entries, property_key).is_some()
            || matches!(
                Self::object_get_entry(
                    entries,
                    &Self::object_undefined_getter_storage_key(property_key),
                ),
                Some(Value::Bool(true))
            )
    }

    pub(crate) fn has_object_setter_property(
        entries: &(impl ObjectEntryLookup + ?Sized),
        property_key: &str,
    ) -> bool {
        Self::object_setter_from_entries(entries, property_key).is_some()
            || matches!(
                Self::object_get_entry(
                    entries,
                    &Self::object_undefined_setter_storage_key(property_key),
                ),
                Some(Value::Bool(true))
            )
    }

    pub(crate) fn has_object_accessor_property(
        entries: &(impl ObjectEntryLookup + ?Sized),
        property_key: &str,
    ) -> bool {
        Self::has_object_getter_property(entries, property_key)
            || Self::has_object_setter_property(entries, property_key)
    }

    pub(crate) fn is_writable_object_key(
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> bool {
        !matches!(
            Self::object_get_entry(entries, &Self::object_non_writable_storage_key(key)),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_configurable_object_key(
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> bool {
        !matches!(
            Self::object_get_entry(entries, &Self::object_non_configurable_storage_key(key)),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn mark_builtin_object_property_deleted(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) {
        Self::object_set_entry(
            entries,
            Self::object_deleted_builtin_storage_key(key),
            Value::Bool(true),
        );
    }

    pub(crate) fn is_builtin_object_property_deleted(
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> bool {
        matches!(
            Self::object_get_entry(entries, &Self::object_deleted_builtin_storage_key(key)),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_callable_own_surface_key(key: &str) -> bool {
        matches!(key, "name" | "length")
    }

    pub(crate) fn deleted_callable_surface_fallback_value(key: &str) -> Option<Value> {
        match key {
            "name" => Some(Value::String(String::new())),
            "length" => Some(Value::Number(0)),
            _ => None,
        }
    }

    pub(crate) fn is_function_builtin_prototype_key(function: &FunctionValue, key: &str) -> bool {
        key == "prototype" && !function.is_arrow && !function.is_method
    }

    pub(crate) fn set_function_builtin_prototype_property(
        entries: &mut ObjectValue,
        value: Value,
        writable: bool,
    ) {
        Self::delete_object_property_entries(entries, "prototype");
        Self::object_set_entry(entries, "prototype".to_string(), value);
        Self::object_set_entry(
            entries,
            Self::object_non_enumerable_storage_key("prototype"),
            Value::Bool(true),
        );
        Self::object_set_entry(
            entries,
            Self::object_non_configurable_storage_key("prototype"),
            Value::Bool(true),
        );
        if !writable {
            Self::object_set_entry(
                entries,
                Self::object_non_writable_storage_key("prototype"),
                Value::Bool(true),
            );
        }
    }

    pub(crate) fn is_regexp_prototype_object(entries: &(impl ObjectEntryLookup + ?Sized)) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_REGEXP_PROTOTYPE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn regexp_default_property_value(key: &str) -> Option<Value> {
        match key {
            "source" => Some(Value::String("(?:)".to_string())),
            "flags" => Some(Value::String(String::new())),
            "global" | "ignoreCase" | "multiline" | "dotAll" | "sticky" | "hasIndices"
            | "unicode" | "unicodeSets" => Some(Value::Bool(false)),
            "lastIndex" => Some(Value::Number(0)),
            _ => None,
        }
    }

    pub(crate) fn regexp_instance_property_value(regex: &RegexValue, key: &str) -> Option<Value> {
        match key {
            "source" => Some(Value::String(regex.source.clone())),
            "flags" => Some(Value::String(regex.flags.clone())),
            "global" => Some(Value::Bool(regex.global)),
            "ignoreCase" => Some(Value::Bool(regex.ignore_case)),
            "multiline" => Some(Value::Bool(regex.multiline)),
            "dotAll" => Some(Value::Bool(regex.dot_all)),
            "sticky" => Some(Value::Bool(regex.sticky)),
            "hasIndices" => Some(Value::Bool(regex.has_indices)),
            "unicode" => Some(Value::Bool(regex.unicode)),
            "unicodeSets" => Some(Value::Bool(regex.unicode_sets)),
            "lastIndex" => Some(Value::Number(regex.last_index as i64)),
            _ => None,
        }
    }

    pub(crate) fn is_regexp_builtin_own_key(key: &str) -> bool {
        matches!(
            key,
            "source"
                | "flags"
                | "global"
                | "ignoreCase"
                | "multiline"
                | "dotAll"
                | "sticky"
                | "hasIndices"
                | "unicode"
                | "unicodeSets"
                | "lastIndex"
        )
    }

    fn invoke_object_getter(&mut self, getter: &Value, receiver: &Value) -> Result<Value> {
        if !self.is_callable_value(getter) {
            return Err(Error::ScriptRuntime("object getter is not callable".into()));
        }
        let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
        self.execute_callable_value_with_this_and_env(
            getter,
            &[],
            &event,
            None,
            Some(receiver.clone()),
        )
    }

    pub(crate) fn object_property_from_entries_with_getter(
        &mut self,
        receiver: &Value,
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> Result<Option<Value>> {
        if let Some(getter) = Self::object_getter_from_entries(entries, key) {
            return Ok(Some(self.invoke_object_getter(&getter, receiver)?));
        }
        if Self::has_object_accessor_property(entries, key) {
            return Ok(Some(Value::Undefined));
        }
        Ok(Self::object_get_entry(entries, key))
    }
}
