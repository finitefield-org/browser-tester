use super::*;
use std::collections::HashSet;

pub(crate) enum NormalizedOwnPropertyDescriptor {
    Data {
        value: Value,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    },
    Accessor {
        get: Value,
        set: Value,
        enumerable: bool,
        configurable: bool,
    },
}

impl Harness {
    pub(crate) fn own_property_integer_key(key: &str) -> Option<u64> {
        if key.is_empty() || !key.as_bytes().iter().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let value = key.parse::<u64>().ok()?;
        (value.to_string() == key).then_some(value)
    }

    pub(crate) fn own_data_property_descriptor_with_attrs(
        value: Value,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    ) -> Value {
        Self::new_object_value(vec![
            ("value".to_string(), value),
            ("writable".to_string(), Value::Bool(writable)),
            ("enumerable".to_string(), Value::Bool(enumerable)),
            ("configurable".to_string(), Value::Bool(configurable)),
        ])
    }

    pub(crate) fn own_property_descriptor_object_from_entries(
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> Option<Value> {
        let getter = Self::object_getter_from_entries(entries, key);
        let setter = Self::object_setter_from_entries(entries, key);
        let has_getter = Self::has_object_getter_property(entries, key);
        let has_setter = Self::has_object_setter_property(entries, key);
        let data = Self::object_get_entry(entries, key);
        if !has_getter && !has_setter && data.is_none() {
            return None;
        }

        let mut descriptor = Vec::new();
        if has_getter || has_setter {
            descriptor.push(("get".to_string(), getter.unwrap_or(Value::Undefined)));
            descriptor.push(("set".to_string(), setter.unwrap_or(Value::Undefined)));
        } else {
            descriptor.push(("value".to_string(), data.unwrap_or(Value::Undefined)));
            descriptor.push((
                "writable".to_string(),
                Value::Bool(Self::is_writable_object_key(entries, key)),
            ));
        }
        descriptor.push((
            "enumerable".to_string(),
            Value::Bool(Self::is_enumerable_object_key(entries, key)),
        ));
        descriptor.push((
            "configurable".to_string(),
            Value::Bool(Self::is_configurable_object_key(entries, key)),
        ));
        Some(Self::new_object_value(descriptor))
    }

    pub(crate) fn descriptor_is_object_like_value(value: &Value) -> bool {
        !matches!(
            value,
            Value::String(_)
                | Value::Bool(_)
                | Value::Number(_)
                | Value::Float(_)
                | Value::BigInt(_)
                | Value::Symbol(_)
                | Value::Null
                | Value::Undefined
        )
    }

    pub(crate) fn descriptor_has_property(
        &mut self,
        descriptor: &Value,
        key: &str,
    ) -> Result<bool> {
        if !Self::descriptor_is_object_like_value(descriptor) {
            return Ok(false);
        }
        match self.object_has_own_value(descriptor, key) {
            Ok(value) => {
                if value.truthy() {
                    return Ok(true);
                }
            }
            Err(_) => {
                return Ok(!matches!(
                    self.object_property_from_value(descriptor, key)?,
                    Value::Undefined
                ));
            }
        }

        let mut prototype = self.value_internal_prototype_value(descriptor);
        while let Some(Value::Object(object)) = prototype {
            let proto_value = Value::Object(object.clone());
            if self.object_has_own_value(&proto_value, key)?.truthy() {
                return Ok(true);
            }
            prototype = {
                let object_ref = object.borrow();
                Self::object_get_entry(&*object_ref, INTERNAL_OBJECT_PROTOTYPE_KEY)
            };
        }
        Ok(false)
    }

    pub(crate) fn descriptor_value_field(
        &mut self,
        descriptor: &Value,
        key: &str,
    ) -> Result<Option<Value>> {
        if !self.descriptor_has_property(descriptor, key)? {
            return Ok(None);
        }
        if let Value::Object(entries) = descriptor {
            let mut current = Some(entries.clone());
            while let Some(object) = current {
                let (value, next) = {
                    let entries = object.borrow();
                    let value = if let Some(getter) =
                        Self::object_getter_from_entries(&*entries, key)
                    {
                        if !self.is_callable_value(&getter) {
                            return Err(Error::ScriptRuntime(
                                "object getter is not callable".into(),
                            ));
                        }
                        let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                        Some(self.execute_callable_value_with_this_and_env(
                            &getter,
                            &[],
                            &event,
                            None,
                            Some(descriptor.clone()),
                        )?)
                    } else if Self::has_object_accessor_property(&*entries, key) {
                        Some(Value::Undefined)
                    } else {
                        Self::object_get_entry(&*entries, key)
                    };
                    (
                        value,
                        Self::object_get_entry(&*entries, INTERNAL_OBJECT_PROTOTYPE_KEY),
                    )
                };
                if let Some(value) = value {
                    return Ok(Some(value));
                }
                current = match next {
                    Some(Value::Object(next)) => Some(next),
                    _ => None,
                };
            }
            return Ok(Some(Value::Undefined));
        }
        Ok(Some(self.object_property_from_value(descriptor, key)?))
    }

    pub(crate) fn descriptor_bool_field(
        &mut self,
        descriptor: &Value,
        key: &str,
    ) -> Result<Option<bool>> {
        Ok(self
            .descriptor_value_field(descriptor, key)?
            .map(|value| value.truthy()))
    }

    pub(crate) fn descriptor_is_accessor_descriptor(&mut self, descriptor: &Value) -> Result<bool> {
        Ok(self.descriptor_has_property(descriptor, "get")?
            || self.descriptor_has_property(descriptor, "set")?)
    }

    pub(crate) fn redefine_property_error(key: &str) -> Error {
        Error::ScriptRuntime(format!("Cannot redefine property: {key}"))
    }

    pub(crate) fn accessor_property_key_from_storage_key(key: &str) -> Option<&str> {
        key.strip_prefix(INTERNAL_OBJECT_GETTER_KEY_PREFIX)
            .or_else(|| key.strip_prefix(INTERNAL_OBJECT_SETTER_KEY_PREFIX))
            .or_else(|| key.strip_prefix(INTERNAL_OBJECT_UNDEFINED_GETTER_KEY_PREFIX))
            .or_else(|| key.strip_prefix(INTERNAL_OBJECT_UNDEFINED_SETTER_KEY_PREFIX))
    }

    pub(crate) fn normalize_property_descriptor(
        &mut self,
        current_descriptor: Option<&Value>,
        key: &str,
        descriptor: &Value,
    ) -> Result<NormalizedOwnPropertyDescriptor> {
        let enumerable = self.descriptor_bool_field(descriptor, "enumerable")?;
        let configurable = self.descriptor_bool_field(descriptor, "configurable")?;
        let value = self.descriptor_value_field(descriptor, "value")?;
        let writable = self.descriptor_bool_field(descriptor, "writable")?;
        let get = self.descriptor_value_field(descriptor, "get")?;
        let set = self.descriptor_value_field(descriptor, "set")?;

        let requested_accessor = get.is_some() || set.is_some();
        let requested_data = value.is_some() || writable.is_some();
        if requested_accessor && requested_data {
            return Err(Error::ScriptRuntime(
                "Invalid property descriptor. Cannot both specify accessors and a value or writable attribute"
                    .into(),
            ));
        }

        if let Some(getter) = get.as_ref() {
            if !matches!(getter, Value::Undefined) && !self.is_callable_value(getter) {
                return Err(Error::ScriptRuntime(
                    "Object.defineProperty getter must be callable or undefined".into(),
                ));
            }
        }
        if let Some(setter) = set.as_ref() {
            if !matches!(setter, Value::Undefined) && !self.is_callable_value(setter) {
                return Err(Error::ScriptRuntime(
                    "Object.defineProperty setter must be callable or undefined".into(),
                ));
            }
        }

        let Some(current_descriptor) = current_descriptor else {
            return Ok(if requested_accessor {
                NormalizedOwnPropertyDescriptor::Accessor {
                    get: get.unwrap_or(Value::Undefined),
                    set: set.unwrap_or(Value::Undefined),
                    enumerable: enumerable.unwrap_or(false),
                    configurable: configurable.unwrap_or(false),
                }
            } else {
                NormalizedOwnPropertyDescriptor::Data {
                    value: value.unwrap_or(Value::Undefined),
                    writable: writable.unwrap_or(false),
                    enumerable: enumerable.unwrap_or(false),
                    configurable: configurable.unwrap_or(false),
                }
            });
        };

        let current_is_accessor = self.descriptor_is_accessor_descriptor(current_descriptor)?;
        let current_enumerable = self
            .object_property_from_value(current_descriptor, "enumerable")?
            .truthy();
        let current_configurable = self
            .object_property_from_value(current_descriptor, "configurable")?
            .truthy();

        if !current_configurable {
            if configurable == Some(true) {
                return Err(Self::redefine_property_error(key));
            }
            if enumerable.is_some_and(|next| next != current_enumerable) {
                return Err(Self::redefine_property_error(key));
            }
            if (requested_accessor && !current_is_accessor)
                || (requested_data && current_is_accessor)
            {
                return Err(Self::redefine_property_error(key));
            }
        }

        let target_is_accessor = if requested_accessor {
            true
        } else if requested_data {
            false
        } else {
            current_is_accessor
        };

        if target_is_accessor {
            let current_get = if current_is_accessor {
                self.object_property_from_value(current_descriptor, "get")?
            } else {
                Value::Undefined
            };
            let current_set = if current_is_accessor {
                self.object_property_from_value(current_descriptor, "set")?
            } else {
                Value::Undefined
            };
            let next_get = get.clone().unwrap_or_else(|| {
                if requested_accessor && !current_is_accessor {
                    Value::Undefined
                } else {
                    current_get.clone()
                }
            });
            let next_set = set.clone().unwrap_or_else(|| {
                if requested_accessor && !current_is_accessor {
                    Value::Undefined
                } else {
                    current_set.clone()
                }
            });
            if !current_configurable && current_is_accessor {
                if get
                    .as_ref()
                    .is_some_and(|_| !self.strict_equal(&next_get, &current_get))
                {
                    return Err(Self::redefine_property_error(key));
                }
                if set
                    .as_ref()
                    .is_some_and(|_| !self.strict_equal(&next_set, &current_set))
                {
                    return Err(Self::redefine_property_error(key));
                }
            }
            return Ok(NormalizedOwnPropertyDescriptor::Accessor {
                get: next_get,
                set: next_set,
                enumerable: enumerable.unwrap_or(current_enumerable),
                configurable: configurable.unwrap_or(current_configurable),
            });
        }

        let current_value = if current_is_accessor {
            Value::Undefined
        } else {
            self.object_property_from_value(current_descriptor, "value")?
        };
        let current_writable = if current_is_accessor {
            false
        } else {
            self.object_property_from_value(current_descriptor, "writable")?
                .truthy()
        };
        let next_value = value.clone().unwrap_or_else(|| {
            if requested_data && current_is_accessor {
                Value::Undefined
            } else {
                current_value.clone()
            }
        });
        let next_writable = writable.unwrap_or_else(|| {
            if requested_data && current_is_accessor {
                false
            } else {
                current_writable
            }
        });

        if !current_configurable && !current_is_accessor && !current_writable {
            if writable == Some(true) {
                return Err(Self::redefine_property_error(key));
            }
            if value
                .as_ref()
                .is_some_and(|_| !self.strict_equal(&next_value, &current_value))
            {
                return Err(Self::redefine_property_error(key));
            }
        }

        Ok(NormalizedOwnPropertyDescriptor::Data {
            value: next_value,
            writable: next_writable,
            enumerable: enumerable.unwrap_or(current_enumerable),
            configurable: configurable.unwrap_or(current_configurable),
        })
    }

    pub(crate) fn set_object_property_flags(
        entries: &mut impl ObjectEntryMut,
        key: &str,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    ) {
        if !enumerable {
            Self::object_set_entry(
                entries,
                Self::object_non_enumerable_storage_key(key),
                Value::Bool(true),
            );
        }
        if !writable {
            Self::object_set_entry(
                entries,
                Self::object_non_writable_storage_key(key),
                Value::Bool(true),
            );
        }
        if !configurable {
            Self::object_set_entry(
                entries,
                Self::object_non_configurable_storage_key(key),
                Value::Bool(true),
            );
        }
    }

    pub(crate) fn array_index_is_enumerable(array: &ArrayValue, index: usize) -> bool {
        Self::is_enumerable_object_key(&array.properties, &index.to_string())
    }

    pub(crate) fn array_index_is_writable(array: &ArrayValue, index: usize) -> bool {
        Self::is_writable_object_key(&array.properties, &index.to_string())
    }

    pub(crate) fn array_index_is_configurable(array: &ArrayValue, index: usize) -> bool {
        Self::is_configurable_object_key(&array.properties, &index.to_string())
    }

    pub(crate) fn ordered_visible_string_keys_split(
        entries: &ObjectValue,
    ) -> (Vec<String>, Vec<String>) {
        let mut integer_keys: Vec<(u64, String)> = Vec::new();
        let mut string_keys: Vec<String> = Vec::new();
        let mut seen = HashSet::new();
        for (key, _) in entries.iter() {
            if let Some(accessor_key) = Self::accessor_property_key_from_storage_key(key) {
                if Self::is_symbol_storage_key(accessor_key)
                    || !seen.insert(accessor_key.to_string())
                {
                    continue;
                }
                if let Some(index) = Self::own_property_integer_key(accessor_key) {
                    integer_keys.push((index, accessor_key.to_string()));
                } else {
                    string_keys.push(accessor_key.to_string());
                }
                continue;
            }
            if Self::is_internal_object_key(key) {
                continue;
            }
            if !seen.insert(key.to_string()) {
                continue;
            }
            if let Some(index) = Self::own_property_integer_key(key) {
                integer_keys.push((index, key.to_string()));
            } else {
                string_keys.push(key.to_string());
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        (
            integer_keys.into_iter().map(|(_, key)| key).collect(),
            string_keys,
        )
    }

    pub(crate) fn ordered_visible_string_keys(entries: &ObjectValue) -> Vec<String> {
        let (integer_keys, string_keys) = Self::ordered_visible_string_keys_split(entries);
        let mut out = Vec::with_capacity(integer_keys.len() + string_keys.len());
        out.extend(integer_keys);
        out.extend(string_keys);
        out
    }

    pub(crate) fn ordered_enumerable_string_keys(entries: &ObjectValue) -> Vec<String> {
        let mut integer_keys: Vec<(u64, String)> = Vec::new();
        let mut string_keys: Vec<String> = Vec::new();
        let mut seen = HashSet::new();
        for (key, _) in entries.iter() {
            if let Some(accessor_key) = Self::accessor_property_key_from_storage_key(key) {
                if Self::is_symbol_storage_key(accessor_key)
                    || !Self::is_enumerable_object_key(entries, accessor_key)
                    || !seen.insert(accessor_key.to_string())
                {
                    continue;
                }
                if let Some(index) = Self::own_property_integer_key(accessor_key) {
                    integer_keys.push((index, accessor_key.to_string()));
                } else {
                    string_keys.push(accessor_key.to_string());
                }
                continue;
            }
            if !Self::is_enumerable_object_key(entries, key) || !seen.insert(key.to_string()) {
                continue;
            }
            if let Some(index) = Self::own_property_integer_key(key) {
                integer_keys.push((index, key.to_string()));
            } else {
                string_keys.push(key.to_string());
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = Vec::with_capacity(integer_keys.len() + string_keys.len());
        out.extend(integer_keys.into_iter().map(|(_, key)| key));
        out.extend(string_keys);
        out
    }

    pub(crate) fn merge_builtin_string_keys(
        integer_keys: Vec<String>,
        string_keys: Vec<String>,
        builtin_keys: &[&str],
    ) -> Vec<String> {
        let mut out =
            Vec::with_capacity(integer_keys.len() + string_keys.len() + builtin_keys.len());
        out.extend(integer_keys);
        out.extend(builtin_keys.iter().map(|key| key.to_string()));
        out.extend(
            string_keys
                .into_iter()
                .filter(|key| !builtin_keys.contains(&key.as_str())),
        );
        out
    }

    pub(crate) fn string_wrapper_own_string_keys(
        entries: &ObjectValue,
        enumerable_only: bool,
    ) -> Option<Vec<String>> {
        let text = Self::string_wrapper_value_from_object(entries)?;
        let mut integer_keys = text
            .chars()
            .enumerate()
            .map(|(index, _)| (index as u64, index.to_string()))
            .collect::<Vec<_>>();
        let property_keys = if enumerable_only {
            Self::ordered_enumerable_string_keys(entries)
        } else {
            Self::ordered_visible_string_keys(entries)
        };
        for key in &property_keys {
            if let Some(index) = Self::own_property_integer_key(&key) {
                if !integer_keys.iter().any(|(existing, _)| *existing == index) {
                    integer_keys.push((index, key.clone()));
                }
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = integer_keys
            .into_iter()
            .map(|(_, key)| key)
            .collect::<Vec<_>>();
        if !enumerable_only {
            out.push("length".to_string());
        }
        out.extend(property_keys.into_iter().filter(|key| {
            Self::own_property_integer_key(key).is_none() && (enumerable_only || key != "length")
        }));
        Some(out)
    }

    pub(crate) fn string_wrapper_builtin_has_own_property(
        entries: &ObjectValue,
        key: &str,
    ) -> bool {
        let Some(text) = Self::string_wrapper_value_from_object(entries) else {
            return false;
        };
        if key == "length" {
            return true;
        }
        Self::own_property_integer_key(key)
            .is_some_and(|index| (index as usize) < Self::string_char_len(&text))
    }

    pub(crate) fn string_wrapper_builtin_descriptor_value(
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        let text = Self::string_wrapper_value_from_object(entries)?;
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(Self::string_char_len(&text) as i64),
                false,
                false,
                false,
            ));
        }
        let index = Self::own_property_integer_key(key)? as usize;
        let ch = Self::string_char_at(&text, index)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            Value::String(ch.to_string()),
            false,
            true,
            false,
        ))
    }

    pub(crate) fn visible_builtin_string_keys<'a>(
        entries: &(impl ObjectEntryLookup + ?Sized),
        builtin_keys: impl IntoIterator<Item = &'a str>,
    ) -> Vec<&'a str> {
        builtin_keys
            .into_iter()
            .filter(|key| !Self::is_builtin_object_property_deleted(entries, key))
            .collect()
    }

    pub(crate) fn callable_own_surface_value(
        &mut self,
        object: &Value,
        key: &str,
    ) -> Option<Value> {
        match key {
            "name" | "length" => self.callable_function_surface_value(object, key),
            _ => None,
        }
    }

    pub(crate) fn function_builtin_own_property_value(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
    ) -> Option<Value> {
        match key {
            "name" | "length" => Some(self.function_own_property_value(function, key, false)),
            "prototype" if !function.is_arrow && !function.is_method => {
                Some(self.function_own_property_value(function, key, false))
            }
            _ => None,
        }
        .filter(|value| !matches!(value, Value::Undefined))
    }

    pub(crate) fn function_builtin_own_string_keys(
        function: &Rc<FunctionValue>,
    ) -> Vec<&'static str> {
        let mut keys = vec!["length", "name"];
        if !function.is_arrow && !function.is_method {
            keys.push("prototype");
        }
        keys
    }

    pub(crate) fn regexp_builtin_own_string_keys() -> [&'static str; 1] {
        ["lastIndex"]
    }

    pub(crate) fn regexp_builtin_descriptor_value(
        &self,
        regex: &RegexValue,
        key: &str,
    ) -> Option<Value> {
        match key {
            "lastIndex" => Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(regex.last_index as i64),
                Self::is_writable_object_key(&regex.properties, "lastIndex"),
                false,
                false,
            )),
            _ => None,
        }
    }

    pub(crate) fn collection_property_symbol_values(&self, entries: &ObjectValue) -> Vec<Value> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for (key, _) in entries.iter() {
            if let Some(symbol_id) = Self::symbol_id_from_storage_key(key).or_else(|| {
                Self::accessor_property_key_from_storage_key(key)
                    .and_then(Self::symbol_id_from_storage_key)
            }) {
                if !seen.insert(symbol_id) {
                    continue;
                }
                if let Some(symbol) = self.symbol_runtime.symbols_by_id.get(&symbol_id) {
                    out.push(Value::Symbol(symbol.clone()));
                }
            }
        }
        out
    }

    pub(crate) fn apply_normalized_descriptor_to_object_entries(
        entries: &mut ObjectValue,
        key: &str,
        descriptor: NormalizedOwnPropertyDescriptor,
    ) {
        Self::delete_object_property_entries(entries, key);
        match descriptor {
            NormalizedOwnPropertyDescriptor::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => {
                Self::object_set_entry(entries, key.to_string(), value);
                Self::set_object_property_flags(entries, key, writable, enumerable, configurable);
            }
            NormalizedOwnPropertyDescriptor::Accessor {
                get,
                set,
                enumerable,
                configurable,
            } => {
                if !matches!(get, Value::Undefined) {
                    Self::object_set_entry(entries, Self::object_getter_storage_key(key), get);
                } else {
                    Self::object_set_entry(
                        entries,
                        Self::object_undefined_getter_storage_key(key),
                        Value::Bool(true),
                    );
                }
                if !matches!(set, Value::Undefined) {
                    Self::object_set_entry(entries, Self::object_setter_storage_key(key), set);
                } else {
                    Self::object_set_entry(
                        entries,
                        Self::object_undefined_setter_storage_key(key),
                        Value::Bool(true),
                    );
                }
                Self::set_object_property_flags(entries, key, true, enumerable, configurable);
            }
        }
    }

    pub(crate) fn define_object_literal_data_entry(
        entries: &mut Vec<(String, Value)>,
        key: String,
        value: Value,
    ) {
        Self::delete_object_property_auxiliary_entries(entries, &key);
        Self::object_set_entry(entries, key, value);
    }

    pub(crate) fn define_object_literal_getter_entry(
        entries: &mut Vec<(String, Value)>,
        key: String,
        getter: Value,
    ) {
        let existing_setter = Self::object_setter_from_entries(entries, &key);
        Self::delete_object_property_entries(entries, &key);
        Self::object_set_entry(entries, Self::object_getter_storage_key(&key), getter);
        if let Some(setter) = existing_setter {
            Self::object_set_entry(entries, Self::object_setter_storage_key(&key), setter);
        }
    }

    pub(crate) fn define_object_literal_setter_entry(
        entries: &mut Vec<(String, Value)>,
        key: String,
        setter: Value,
    ) {
        let existing_getter = Self::object_getter_from_entries(entries, &key);
        Self::delete_object_property_entries(entries, &key);
        if let Some(getter) = existing_getter {
            Self::object_set_entry(entries, Self::object_getter_storage_key(&key), getter);
        }
        Self::object_set_entry(entries, Self::object_setter_storage_key(&key), setter);
    }
}
