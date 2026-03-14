use super::*;
use std::collections::HashSet;

enum NormalizedOwnPropertyDescriptor {
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

    fn own_data_property_descriptor_with_attrs(
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

    fn own_property_descriptor_object_from_entries(
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

    fn descriptor_is_object_like_value(value: &Value) -> bool {
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

    fn descriptor_has_property(&mut self, descriptor: &Value, key: &str) -> Result<bool> {
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

    fn descriptor_value_field(&mut self, descriptor: &Value, key: &str) -> Result<Option<Value>> {
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

    fn descriptor_bool_field(&mut self, descriptor: &Value, key: &str) -> Result<Option<bool>> {
        Ok(self
            .descriptor_value_field(descriptor, key)?
            .map(|value| value.truthy()))
    }

    fn descriptor_is_accessor_descriptor(&mut self, descriptor: &Value) -> Result<bool> {
        Ok(self.descriptor_has_property(descriptor, "get")?
            || self.descriptor_has_property(descriptor, "set")?)
    }

    fn redefine_property_error(key: &str) -> Error {
        Error::ScriptRuntime(format!("Cannot redefine property: {key}"))
    }

    fn accessor_property_key_from_storage_key(key: &str) -> Option<&str> {
        key.strip_prefix(INTERNAL_OBJECT_GETTER_KEY_PREFIX)
            .or_else(|| key.strip_prefix(INTERNAL_OBJECT_SETTER_KEY_PREFIX))
            .or_else(|| key.strip_prefix(INTERNAL_OBJECT_UNDEFINED_GETTER_KEY_PREFIX))
            .or_else(|| key.strip_prefix(INTERNAL_OBJECT_UNDEFINED_SETTER_KEY_PREFIX))
    }

    fn normalize_property_descriptor(
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

    fn set_object_property_flags(
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

    fn array_index_is_enumerable(array: &ArrayValue, index: usize) -> bool {
        Self::is_enumerable_object_key(&array.properties, &index.to_string())
    }

    fn array_index_is_writable(array: &ArrayValue, index: usize) -> bool {
        Self::is_writable_object_key(&array.properties, &index.to_string())
    }

    fn array_index_is_configurable(array: &ArrayValue, index: usize) -> bool {
        Self::is_configurable_object_key(&array.properties, &index.to_string())
    }

    fn ordered_visible_string_keys_split(entries: &ObjectValue) -> (Vec<String>, Vec<String>) {
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

    fn ordered_visible_string_keys(entries: &ObjectValue) -> Vec<String> {
        let (integer_keys, string_keys) = Self::ordered_visible_string_keys_split(entries);
        let mut out = Vec::with_capacity(integer_keys.len() + string_keys.len());
        out.extend(integer_keys);
        out.extend(string_keys);
        out
    }

    fn ordered_enumerable_string_keys(entries: &ObjectValue) -> Vec<String> {
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

    fn merge_builtin_string_keys(
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

    fn visible_builtin_string_keys<'a>(
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

    fn function_builtin_own_property_value(
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

    fn function_builtin_own_string_keys(function: &Rc<FunctionValue>) -> Vec<&'static str> {
        let mut keys = vec!["length", "name"];
        if !function.is_arrow && !function.is_method {
            keys.push("prototype");
        }
        keys
    }

    fn regexp_builtin_own_string_keys() -> [&'static str; 1] {
        ["lastIndex"]
    }

    fn regexp_builtin_descriptor_value(&self, regex: &RegexValue, key: &str) -> Option<Value> {
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

    fn collection_property_symbol_values(&self, entries: &ObjectValue) -> Vec<Value> {
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

    fn apply_normalized_descriptor_to_object_entries(
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

    fn define_object_literal_data_entry(
        entries: &mut Vec<(String, Value)>,
        key: String,
        value: Value,
    ) {
        Self::delete_object_property_auxiliary_entries(entries, &key);
        Self::object_set_entry(entries, key, value);
    }

    fn define_object_literal_getter_entry(
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

    fn define_object_literal_setter_entry(
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

    fn callable_object_surface_descriptor_value(
        &mut self,
        object: &Value,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if Self::callable_kind_from_value(object).is_none() {
            return None;
        }
        if Self::object_get_entry(entries, key).is_some()
            || Self::has_object_accessor_property(entries, key)
            || Self::is_builtin_object_property_deleted(entries, key)
        {
            return None;
        }
        let value = self.callable_own_surface_value(object, key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            value, false, false, true,
        ))
    }

    fn placeholder_backed_object_builtin_descriptor_value(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        let stored = Self::object_get_entry(entries, key)?;
        if !Self::is_builtin_placeholder_value(&stored) {
            return None;
        }
        let value = Self::placeholder_backed_object_builtin_property_value(entries, key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            value,
            Self::is_writable_object_key(entries, key),
            Self::is_enumerable_object_key(entries, key),
            Self::is_configurable_object_key(entries, key),
        ))
    }

    fn placeholder_backed_array_builtin_descriptor_value(
        &mut self,
        array: &ArrayValue,
        key: &str,
    ) -> Option<Value> {
        let stored = Self::object_get_entry(&array.properties, key)?;
        if !Self::is_builtin_placeholder_value(&stored) {
            return None;
        }
        let value = Self::placeholder_backed_array_builtin_property_value(array, key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            value,
            Self::is_writable_object_key(&array.properties, key),
            Self::is_enumerable_object_key(&array.properties, key),
            Self::is_configurable_object_key(&array.properties, key),
        ))
    }

    fn function_own_property_descriptor_value(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
    ) -> Option<Value> {
        if let Some(entries) = self
            .script_runtime
            .function_public_properties
            .get(&function.function_id)
        {
            if let Some(descriptor) =
                Self::own_property_descriptor_object_from_entries(entries, key)
            {
                return Some(descriptor);
            }
            if Self::is_builtin_object_property_deleted(entries, key) {
                return None;
            }
        }

        match key {
            "name" | "length" => {
                return self
                    .function_builtin_own_property_value(function, key)
                    .map(|value| {
                        Self::own_data_property_descriptor_with_attrs(value, false, false, true)
                    });
            }
            "prototype" if !function.is_arrow && !function.is_method => {
                return self
                    .function_builtin_own_property_value(function, key)
                    .map(|value| {
                        Self::own_data_property_descriptor_with_attrs(value, true, false, false)
                    });
            }
            _ => {}
        }

        None
    }

    fn array_own_property_descriptor_value(
        &mut self,
        array: &Rc<RefCell<ArrayValue>>,
        key: &str,
    ) -> Option<Value> {
        let array_ref = array.borrow();
        if let Some(descriptor) =
            self.placeholder_backed_array_builtin_descriptor_value(&array_ref, key)
        {
            return Some(descriptor);
        }
        if let Some(descriptor) =
            Self::own_property_descriptor_object_from_entries(&array_ref.properties, key)
        {
            return Some(descriptor);
        }
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(array_ref.len() as i64),
                Self::is_writable_object_key(&array_ref.properties, "length"),
                false,
                false,
            ));
        }
        if let Ok(index) = key.parse::<usize>() {
            if index < array_ref.len() && !Self::array_index_is_hole(&array_ref, index) {
                return Some(Self::own_data_property_descriptor_with_attrs(
                    array_ref[index].clone(),
                    Self::array_index_is_writable(&array_ref, index),
                    Self::array_index_is_enumerable(&array_ref, index),
                    Self::array_index_is_configurable(&array_ref, index),
                ));
            }
            return None;
        }
        None
    }

    fn collection_own_property_descriptor_value(
        &mut self,
        entries: &ObjectValue,
        builtin_descriptor: Option<Value>,
        key: &str,
    ) -> Option<Value> {
        if let Some(descriptor) = Self::own_property_descriptor_object_from_entries(entries, key) {
            return Some(descriptor);
        }
        if Self::is_builtin_object_property_deleted(entries, key) {
            return None;
        }
        builtin_descriptor
    }

    pub(crate) fn dom_string_map_synthesized_keys(
        &self,
        entries: &ObjectValue,
        enumerable_only: bool,
    ) -> Option<Vec<String>> {
        if !Self::is_dom_string_map_object(entries) {
            return None;
        }
        let node = Self::dom_string_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let mut keys = self
            .dataset_entries_for_node(node)
            .into_iter()
            .map(|(key, _)| key)
            .filter(|key| Self::own_property_descriptor_object_from_entries(entries, key).is_none())
            .collect::<Vec<_>>();
        keys.extend(if enumerable_only {
            Self::ordered_enumerable_string_keys(entries)
        } else {
            Self::ordered_visible_string_keys(entries)
        });
        Some(keys)
    }

    pub(crate) fn dom_string_map_synthesized_descriptor_value(
        &self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_dom_string_map_object(entries)
            || Self::own_property_descriptor_object_from_entries(entries, key).is_some()
        {
            return None;
        }
        let node = Self::dom_string_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let attr_name = dataset_key_to_attr_name(key);
        let value = self.dom.attr(node, &attr_name)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            Value::String(value),
            true,
            true,
            true,
        ))
    }

    pub(crate) fn class_list_synthesized_keys(
        &self,
        entries: &ObjectValue,
        enumerable_only: bool,
    ) -> Option<Vec<String>> {
        if !Self::is_class_list_object(entries) {
            return None;
        }
        let node = match Self::object_get_entry(entries, INTERNAL_CLASS_LIST_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }?;
        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
        let mut integer_keys = classes
            .iter()
            .enumerate()
            .map(|(index, _)| (index as u64, index.to_string()))
            .collect::<Vec<_>>();
        let property_keys = if enumerable_only {
            Self::ordered_enumerable_string_keys(entries)
        } else {
            Self::ordered_visible_string_keys(entries)
        };
        for key in &property_keys {
            if let Some(index) = Self::own_property_integer_key(key)
                && !integer_keys.iter().any(|(existing, _)| *existing == index)
            {
                integer_keys.push((index, key.clone()));
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = integer_keys
            .into_iter()
            .map(|(_, key)| key)
            .collect::<Vec<_>>();
        if !enumerable_only {
            out.push("length".to_string());
            out.push("value".to_string());
        }
        out.extend(property_keys.into_iter().filter(|key| {
            Self::own_property_integer_key(key).is_none()
                && (enumerable_only || (key != "length" && key != "value"))
        }));
        Some(out)
    }

    fn class_list_synthesized_descriptor_value(
        &self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_class_list_object(entries)
            || Self::own_property_descriptor_object_from_entries(entries, key).is_some()
        {
            return None;
        }
        let node = match Self::object_get_entry(entries, INTERNAL_CLASS_LIST_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }?;
        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(classes.len() as i64),
                true,
                false,
                true,
            ));
        }
        if key == "value" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::String(classes.join(" ")),
                true,
                false,
                true,
            ));
        }
        let index = Self::own_property_integer_key(key)? as usize;
        let class_name = classes.get(index)?.clone();
        Some(Self::own_data_property_descriptor_with_attrs(
            Value::String(class_name),
            true,
            true,
            true,
        ))
    }

    pub(crate) fn named_node_map_synthesized_keys(
        &mut self,
        entries: &ObjectValue,
        enumerable_only: bool,
    ) -> Option<Vec<String>> {
        if !Self::is_named_node_map_object(entries) {
            return None;
        }
        let node = Self::named_node_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let attrs = self.named_node_map_entries(node);
        let mut integer_keys = attrs
            .iter()
            .enumerate()
            .map(|(index, _)| (index as u64, index.to_string()))
            .collect::<Vec<_>>();
        let property_keys = if enumerable_only {
            Self::ordered_enumerable_string_keys(entries)
        } else {
            Self::ordered_visible_string_keys(entries)
        };
        for key in &property_keys {
            if let Some(index) = Self::own_property_integer_key(key)
                && !integer_keys.iter().any(|(existing, _)| *existing == index)
            {
                integer_keys.push((index, key.clone()));
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
        out.extend(attrs.iter().map(|(name, _)| name.clone()).filter(|key| {
            !property_keys.iter().any(|existing| existing == key)
                && self.named_node_map_named_property_is_visible(entries, key)
        }));
        out.extend(property_keys.into_iter().filter(|key| {
            Self::own_property_integer_key(key).is_none() && (enumerable_only || key != "length")
        }));
        Some(out)
    }

    fn named_node_map_synthesized_descriptor_value(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_named_node_map_object(entries)
            || Self::own_property_descriptor_object_from_entries(entries, key).is_some()
        {
            return None;
        }
        let node = Self::named_node_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let attrs = self.named_node_map_entries(node);
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(attrs.len() as i64),
                true,
                false,
                true,
            ));
        }
        if let Some(index) = Self::own_property_integer_key(key) {
            let (name, value) = attrs.get(index as usize)?;
            return Some(Self::own_data_property_descriptor_with_attrs(
                Self::new_attr_object_value(name, value, Some(node)),
                true,
                true,
                true,
            ));
        }
        if !self.named_node_map_named_property_is_visible(entries, key) {
            return None;
        }
        let (name, value) = attrs.iter().find(|(name, _)| name == key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            Self::new_attr_object_value(name, value, Some(node)),
            true,
            true,
            true,
        ))
    }

    pub(crate) fn node_list_synthesized_keys(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        enumerable_only: bool,
    ) -> Vec<String> {
        let snapshot = self.node_list_snapshot(nodes);
        let mut integer_keys = snapshot
            .iter()
            .enumerate()
            .map(|(index, _)| (index as u64, index.to_string()))
            .collect::<Vec<_>>();
        let property_keys = {
            let nodes_ref = nodes.borrow();
            if enumerable_only {
                Self::ordered_enumerable_string_keys(&nodes_ref.properties)
            } else {
                Self::ordered_visible_string_keys(&nodes_ref.properties)
            }
        };
        for key in &property_keys {
            if let Some(index) = Self::own_property_integer_key(key)
                && !integer_keys.iter().any(|(existing, _)| *existing == index)
            {
                integer_keys.push((index, key.clone()));
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = integer_keys
            .into_iter()
            .map(|(_, key)| key)
            .collect::<Vec<_>>();
        let named_keys = self
            .html_collection_named_entries(nodes)
            .into_iter()
            .map(|(name, _)| name)
            .filter(|key| {
                !property_keys.iter().any(|existing| existing == key)
                    && self.html_collection_named_property_is_visible(nodes, key)
            })
            .collect::<Vec<_>>();
        if !enumerable_only {
            out.push("length".to_string());
        }
        out.extend(named_keys);
        out.extend(property_keys.into_iter().filter(|key| {
            Self::own_property_integer_key(key).is_none() && (enumerable_only || key != "length")
        }));
        out
    }

    fn node_list_synthesized_descriptor_value(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        key: &str,
    ) -> Option<Value> {
        {
            let nodes_ref = nodes.borrow();
            if Self::own_property_descriptor_object_from_entries(&nodes_ref.properties, key)
                .is_some()
            {
                return None;
            }
        }
        let snapshot = self.node_list_snapshot(nodes);
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(snapshot.len() as i64),
                false,
                false,
                true,
            ));
        }
        if let Some(index) = Self::own_property_integer_key(key) {
            if let Some(node) = snapshot.get(index as usize).copied() {
                return Some(Self::own_data_property_descriptor_with_attrs(
                    self.node_list_item_value(nodes, node),
                    false,
                    true,
                    true,
                ));
            }
        }
        self.html_collection_named_property_value(nodes, key)
            .map(|value| Self::own_data_property_descriptor_with_attrs(value, false, true, true))
    }

    pub(crate) fn node_expando_entries(&self, node: NodeId) -> Vec<(String, Value)> {
        let mut entries = self
            .dom_runtime
            .node_expando_props
            .iter()
            .filter(|((owner, _), _)| *owner == node)
            .map(|((_, key), value)| (key.clone(), value.clone()))
            .collect::<Vec<_>>();
        entries.sort_by(|(left, _), (right, _)| left.cmp(right));
        entries
    }

    pub(crate) fn replace_node_expando_entries(
        &mut self,
        node: NodeId,
        entries: Vec<(String, Value)>,
    ) {
        self.dom_runtime
            .node_expando_props
            .retain(|(owner, _), _| *owner != node);
        for (key, value) in entries {
            self.dom_runtime
                .node_expando_props
                .insert((node, key), value);
        }
    }

    pub(crate) fn node_has_explicit_own_property(&self, node: NodeId, key: &str) -> bool {
        let entries = self.node_expando_entries(node);
        Self::object_get_entry(&entries, key).is_some()
            || Self::has_object_accessor_property(&entries, key)
    }

    pub(crate) fn node_expando_enumerable_string_keys(&self, node: NodeId) -> Vec<String> {
        let entries = ObjectValue::new(self.node_expando_entries(node));
        Self::ordered_enumerable_string_keys(&entries)
    }

    pub(crate) fn node_expando_string_keys(&self, node: NodeId) -> Vec<String> {
        let entries = ObjectValue::new(self.node_expando_entries(node));
        Self::ordered_visible_string_keys(&entries)
    }

    pub(crate) fn node_expando_enumerable_symbol_values(&self, node: NodeId) -> Vec<Value> {
        let entries = self.node_expando_entries(node);
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for (key, _) in &entries {
            if !Self::is_symbol_storage_key(key) || !Self::is_enumerable_object_key(&entries, key) {
                continue;
            }
            if let Some(symbol_id) = Self::symbol_id_from_storage_key(key) {
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

    fn node_expando_symbol_values(&self, node: NodeId) -> Vec<Value> {
        let entries = self.node_expando_entries(node);
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for (key, _) in &entries {
            if let Some(symbol_id) = Self::symbol_id_from_storage_key(key) {
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

    fn html_form_own_string_keys(&mut self, form: NodeId) -> Result<Vec<String>> {
        let expando_keys = self.node_expando_string_keys(form);
        let expando_set = expando_keys.iter().cloned().collect::<HashSet<_>>();
        let mut out = expando_keys;

        for key in Self::html_form_builtin_own_string_keys() {
            if !expando_set.contains(key) {
                out.push(key.to_string());
            }
        }

        for key in self.html_form_named_property_keys(form)? {
            if !expando_set.contains(&key) && !out.iter().any(|existing| existing == &key) {
                out.push(key);
            }
        }

        Ok(out)
    }

    fn html_media_own_string_keys(&mut self, media: NodeId) -> Vec<String> {
        let expando_keys = self.node_expando_string_keys(media);
        let expando_set = expando_keys.iter().cloned().collect::<HashSet<_>>();
        let mut out = expando_keys;

        for key in Self::html_media_builtin_own_string_keys() {
            if !expando_set.contains(key) {
                out.push(key.to_string());
            }
        }

        out
    }

    fn node_own_property_descriptor_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Option<Value>> {
        let expando_entries = self.node_expando_entries(node);
        if let Some(descriptor) =
            Self::own_property_descriptor_object_from_entries(&expando_entries, key)
        {
            return Ok(Some(descriptor));
        }

        let is_form = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("form"));
        if is_form {
            if let Some(value) = self.html_form_builtin_property_value(node, key)? {
                return Ok(Some(Self::own_data_property_descriptor_with_attrs(
                    value, false, false, true,
                )));
            }

            return Ok(self.form_named_property_value(node, key)?.map(|value| {
                Self::own_data_property_descriptor_with_attrs(value, false, false, true)
            }));
        }

        let is_media = self.dom.tag_name(node).is_some_and(|tag| {
            tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
        });
        if !is_media {
            return Ok(None);
        }

        if let Some(value) = self.html_media_builtin_property_value(node, key)? {
            return Ok(Some(Self::own_data_property_descriptor_with_attrs(
                value, false, false, true,
            )));
        }

        Ok(None)
    }

    fn object_like_enumerable_keys(&mut self, object: &Value) -> Result<Vec<String>> {
        match object {
            Value::Object(entries) => {
                let entries = entries.borrow();
                Ok(Self::string_wrapper_own_string_keys(&entries, true)
                    .or_else(|| self.class_list_synthesized_keys(&entries, true))
                    .or_else(|| self.named_node_map_synthesized_keys(&entries, true))
                    .or_else(|| self.dom_string_map_synthesized_keys(&entries, true))
                    .unwrap_or_else(|| Self::ordered_enumerable_string_keys(&entries)))
            }
            Value::Array(array) => {
                let array_ref = array.borrow();
                let mut keys = array_ref
                    .iter()
                    .enumerate()
                    .filter_map(|(index, _)| {
                        (!Self::array_index_is_hole(&array_ref, index)
                            && Self::array_index_is_enumerable(&array_ref, index))
                        .then(|| index.to_string())
                    })
                    .collect::<Vec<_>>();
                keys.extend(Self::ordered_enumerable_string_keys(&array_ref.properties));
                Ok(keys)
            }
            Value::Node(node) => Ok(self.node_expando_enumerable_string_keys(*node)),
            Value::NodeList(nodes) => Ok(self.node_list_synthesized_keys(nodes, true)),
            Value::Function(function) => Ok(self
                .script_runtime
                .function_public_properties
                .get(&function.function_id)
                .map(Self::ordered_enumerable_string_keys)
                .unwrap_or_default()),
            Value::Map(map) => Ok(Self::ordered_enumerable_string_keys(
                &map.borrow().properties,
            )),
            Value::WeakMap(map) => Ok(Self::ordered_enumerable_string_keys(
                &map.borrow().properties,
            )),
            Value::Set(set) => Ok(Self::ordered_enumerable_string_keys(
                &set.borrow().properties,
            )),
            Value::WeakSet(set) => Ok(Self::ordered_enumerable_string_keys(
                &set.borrow().properties,
            )),
            Value::RegExp(regex) => Ok(Self::ordered_enumerable_string_keys(
                &regex.borrow().properties,
            )),
            _ => Err(Error::ScriptRuntime(
                "Object.keys argument must be an object".into(),
            )),
        }
    }

    fn object_like_own_string_keys(&mut self, object: &Value) -> Result<Vec<String>> {
        match object {
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(keys) = Self::string_wrapper_own_string_keys(&entries, false) {
                    return Ok(keys);
                }
                if let Some(keys) = self.class_list_synthesized_keys(&entries, false) {
                    return Ok(keys);
                }
                if let Some(keys) = self.named_node_map_synthesized_keys(&entries, false) {
                    return Ok(keys);
                }
                if let Some(keys) = self.dom_string_map_synthesized_keys(&entries, false) {
                    return Ok(keys);
                }
                let (integer_keys, string_keys) = Self::ordered_visible_string_keys_split(&entries);
                Ok(if Self::callable_kind_from_value(object).is_some() {
                    let builtin_keys =
                        Self::visible_builtin_string_keys(&entries, ["length", "name"]);
                    Self::merge_builtin_string_keys(integer_keys, string_keys, &builtin_keys)
                } else {
                    Self::ordered_visible_string_keys(&entries)
                })
            }
            Value::Node(node) => {
                let is_form = self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("form"));
                if is_form {
                    self.html_form_own_string_keys(*node)
                } else if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) {
                    Ok(self.html_media_own_string_keys(*node))
                } else {
                    Ok(self.node_expando_string_keys(*node))
                }
            }
            Value::Array(array) => {
                let array_ref = array.borrow();
                let mut integer_keys: Vec<(u64, String)> = array_ref
                    .iter()
                    .enumerate()
                    .filter_map(|(index, _)| {
                        (!Self::array_index_is_hole(&array_ref, index))
                            .then(|| (index as u64, index.to_string()))
                    })
                    .collect();
                let (property_integer_keys, property_string_keys) =
                    Self::ordered_visible_string_keys_split(&array_ref.properties);
                for key in property_integer_keys {
                    if let Some(index) = Self::own_property_integer_key(&key) {
                        if !integer_keys.iter().any(|(existing, _)| *existing == index) {
                            integer_keys.push((index, key));
                        }
                    }
                }
                integer_keys.sort_by_key(|(index, _)| *index);
                let mut out = integer_keys
                    .into_iter()
                    .map(|(_, key)| key)
                    .collect::<Vec<_>>();
                out.push("length".to_string());
                out.extend(
                    property_string_keys
                        .into_iter()
                        .filter(|key| key != "length"),
                );
                Ok(out)
            }
            Value::NodeList(nodes) => Ok(self.node_list_synthesized_keys(nodes, false)),
            Value::Function(function) => {
                let (integer_keys, string_keys, builtin_keys) = self
                    .script_runtime
                    .function_public_properties
                    .get(&function.function_id)
                    .map(|entries| {
                        let (integer_keys, string_keys) =
                            Self::ordered_visible_string_keys_split(entries);
                        (
                            integer_keys,
                            string_keys,
                            Self::visible_builtin_string_keys(
                                entries,
                                Self::function_builtin_own_string_keys(function),
                            ),
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            Vec::new(),
                            Vec::new(),
                            Self::function_builtin_own_string_keys(function),
                        )
                    });
                Ok(Self::merge_builtin_string_keys(
                    integer_keys,
                    string_keys,
                    &builtin_keys,
                ))
            }
            Value::Map(map) => {
                let map = map.borrow();
                let (integer_keys, string_keys) =
                    Self::ordered_visible_string_keys_split(&map.properties);
                let builtin_keys = Self::visible_builtin_string_keys(&map.properties, ["size"]);
                Ok(Self::merge_builtin_string_keys(
                    integer_keys,
                    string_keys,
                    &builtin_keys,
                ))
            }
            Value::WeakMap(map) => Ok(Self::ordered_visible_string_keys(&map.borrow().properties)),
            Value::Set(set) => {
                let set = set.borrow();
                let (integer_keys, string_keys) =
                    Self::ordered_visible_string_keys_split(&set.properties);
                let builtin_keys = Self::visible_builtin_string_keys(&set.properties, ["size"]);
                Ok(Self::merge_builtin_string_keys(
                    integer_keys,
                    string_keys,
                    &builtin_keys,
                ))
            }
            Value::WeakSet(set) => Ok(Self::ordered_visible_string_keys(&set.borrow().properties)),
            Value::RegExp(regex) => {
                let regex = regex.borrow();
                let (integer_keys, string_keys) =
                    Self::ordered_visible_string_keys_split(&regex.properties);
                let builtin_keys = Self::visible_builtin_string_keys(
                    &regex.properties,
                    Self::regexp_builtin_own_string_keys(),
                );
                Ok(Self::merge_builtin_string_keys(
                    integer_keys,
                    string_keys,
                    &builtin_keys,
                ))
            }
            _ => Err(Error::ScriptRuntime(
                "Object.getOwnPropertyNames argument must be an object".into(),
            )),
        }
    }

    fn object_like_own_symbol_values(&self, object: &Value) -> Result<Vec<Value>> {
        match object {
            Value::Object(entries) => Ok(self.collection_property_symbol_values(&entries.borrow())),
            Value::Array(array) => {
                Ok(self.collection_property_symbol_values(&array.borrow().properties))
            }
            Value::Node(node) => Ok(self.node_expando_symbol_values(*node)),
            Value::NodeList(nodes) => {
                Ok(self.collection_property_symbol_values(&nodes.borrow().properties))
            }
            Value::Function(function) => Ok(self
                .script_runtime
                .function_public_properties
                .get(&function.function_id)
                .map(|entries| self.collection_property_symbol_values(entries))
                .unwrap_or_default()),
            Value::Map(map) => Ok(self.collection_property_symbol_values(&map.borrow().properties)),
            Value::WeakMap(map) => {
                Ok(self.collection_property_symbol_values(&map.borrow().properties))
            }
            Value::Set(set) => Ok(self.collection_property_symbol_values(&set.borrow().properties)),
            Value::WeakSet(set) => {
                Ok(self.collection_property_symbol_values(&set.borrow().properties))
            }
            Value::RegExp(regex) => {
                Ok(self.collection_property_symbol_values(&regex.borrow().properties))
            }
            _ => Err(Error::ScriptRuntime(
                "Reflect.ownKeys target must be an object".into(),
            )),
        }
    }

    pub(crate) fn object_get_own_property_names_value(&mut self, object: &Value) -> Result<Value> {
        Ok(Self::new_array_value(
            self.object_like_own_string_keys(object)?
                .into_iter()
                .map(Value::String)
                .collect(),
        ))
    }

    pub(crate) fn reflect_own_keys_value(&mut self, object: &Value) -> Result<Value> {
        let mut keys = self
            .object_like_own_string_keys(object)?
            .into_iter()
            .map(Value::String)
            .collect::<Vec<_>>();
        keys.extend(self.object_like_own_symbol_values(object)?);
        Ok(Self::new_array_value(keys))
    }

    pub(crate) fn object_get_own_property_descriptor_value(
        &mut self,
        object: &Value,
        key: &str,
    ) -> Result<Value> {
        match object {
            Value::Object(entries) => Ok({
                let entries = entries.borrow();
                Self::string_wrapper_builtin_descriptor_value(&entries, key)
                    .or_else(|| self.class_list_synthesized_descriptor_value(&entries, key))
                    .or_else(|| self.named_node_map_synthesized_descriptor_value(&entries, key))
                    .or_else(|| {
                        self.placeholder_backed_object_builtin_descriptor_value(&entries, key)
                    })
                    .or_else(|| Self::own_property_descriptor_object_from_entries(&*entries, key))
                    .or_else(|| self.dom_string_map_synthesized_descriptor_value(&entries, key))
                    .or_else(|| {
                        self.callable_object_surface_descriptor_value(object, &entries, key)
                    })
                    .unwrap_or(Value::Undefined)
            }),
            Value::Array(array) => Ok(self
                .array_own_property_descriptor_value(array, key)
                .unwrap_or(Value::Undefined)),
            Value::Node(node) => Ok(self
                .node_own_property_descriptor_value(*node, key)?
                .unwrap_or(Value::Undefined)),
            Value::NodeList(nodes) => {
                let own = {
                    let nodes_ref = nodes.borrow();
                    Self::own_property_descriptor_object_from_entries(&nodes_ref.properties, key)
                };
                Ok(own
                    .or_else(|| self.node_list_synthesized_descriptor_value(nodes, key))
                    .unwrap_or(Value::Undefined))
            }
            Value::Function(function) => Ok(self
                .function_own_property_descriptor_value(function, key)
                .unwrap_or(Value::Undefined)),
            Value::Map(map) => Ok({
                let map = map.borrow();
                self.collection_own_property_descriptor_value(
                    &map.properties,
                    (key == "size").then(|| {
                        Self::own_data_property_descriptor_with_attrs(
                            Value::Number(map.entries.len() as i64),
                            false,
                            false,
                            true,
                        )
                    }),
                    key,
                )
                .unwrap_or(Value::Undefined)
            }),
            Value::WeakMap(map) => Ok({
                let map = map.borrow();
                self.collection_own_property_descriptor_value(&map.properties, None, key)
                    .unwrap_or(Value::Undefined)
            }),
            Value::Set(set) => Ok({
                let set = set.borrow();
                self.collection_own_property_descriptor_value(
                    &set.properties,
                    (key == "size").then(|| {
                        Self::own_data_property_descriptor_with_attrs(
                            Value::Number(set.values.len() as i64),
                            false,
                            false,
                            true,
                        )
                    }),
                    key,
                )
                .unwrap_or(Value::Undefined)
            }),
            Value::WeakSet(set) => Ok({
                let set = set.borrow();
                self.collection_own_property_descriptor_value(&set.properties, None, key)
                    .unwrap_or(Value::Undefined)
            }),
            Value::RegExp(regex) => Ok({
                let regex = regex.borrow();
                self.collection_own_property_descriptor_value(
                    &regex.properties,
                    self.regexp_builtin_descriptor_value(&regex, key),
                    key,
                )
                .unwrap_or(Value::Undefined)
            }),
            _ => Err(Error::ScriptRuntime(
                "Object.getOwnPropertyDescriptor argument must be an object".into(),
            )),
        }
    }

    fn define_property_on_object_entries(
        &mut self,
        entries: &mut ObjectValue,
        key: &str,
        descriptor: &Value,
    ) -> Result<()> {
        let current_descriptor = Self::own_property_descriptor_object_from_entries(entries, key);
        let normalized =
            self.normalize_property_descriptor(current_descriptor.as_ref(), key, descriptor)?;
        Self::apply_normalized_descriptor_to_object_entries(entries, key, normalized);
        Ok(())
    }

    pub(crate) fn object_define_property_value(
        &mut self,
        object: &Value,
        key: &str,
        descriptor: &Value,
    ) -> Result<Value> {
        if !Self::descriptor_is_object_like_value(descriptor) {
            return Err(Error::ScriptRuntime(
                "Object.defineProperty descriptor must be an object".into(),
            ));
        }

        match object {
            Value::Object(entries) => {
                let current_descriptor = {
                    let entries_ref = entries.borrow();
                    Self::string_wrapper_builtin_descriptor_value(&entries_ref, key)
                };
                if let Some(current_descriptor) = current_descriptor {
                    self.normalize_property_descriptor(Some(&current_descriptor), key, descriptor)?;
                    Self::delete_object_property_entries(&mut entries.borrow_mut(), key);
                    return Ok(object.clone());
                }
                self.define_property_on_object_entries(&mut entries.borrow_mut(), key, descriptor)?;
                Ok(object.clone())
            }
            Value::Array(array) => {
                let current_descriptor = self.array_own_property_descriptor_value(array, key);
                if key == "length" {
                    let normalized = self.normalize_property_descriptor(
                        current_descriptor.as_ref(),
                        key,
                        descriptor,
                    )?;
                    let NormalizedOwnPropertyDescriptor::Data {
                        value, writable, ..
                    } = normalized
                    else {
                        return Err(Self::redefine_property_error(key));
                    };
                    let mut values = array.borrow_mut();
                    Self::delete_object_property_entries(&mut values.properties, key);
                    let next = Self::value_to_i64(&value);
                    let next = if next <= 0 { 0usize } else { next as usize };
                    if next < values.len() {
                        values.truncate(next);
                    } else if next > values.len() {
                        values.resize(next, Value::Undefined);
                    }
                    Self::set_object_property_flags(
                        &mut values.properties,
                        key,
                        writable,
                        false,
                        false,
                    );
                    return Ok(object.clone());
                }
                if let Ok(index) = key.parse::<usize>() {
                    if !self.descriptor_is_accessor_descriptor(descriptor)? {
                        let normalized = self.normalize_property_descriptor(
                            current_descriptor.as_ref(),
                            key,
                            descriptor,
                        )?;
                        let NormalizedOwnPropertyDescriptor::Data {
                            value,
                            writable,
                            enumerable,
                            configurable,
                        } = normalized
                        else {
                            unreachable!();
                        };
                        let mut values = array.borrow_mut();
                        Self::delete_object_property_entries(&mut values.properties, key);
                        if index >= values.len() {
                            values.resize(index + 1, Value::Undefined);
                        }
                        values[index] = value;
                        Self::set_object_property_flags(
                            &mut values.properties,
                            key,
                            writable,
                            enumerable,
                            configurable,
                        );
                        drop(values);
                        Self::clear_array_hole(array, index);
                        return Ok(object.clone());
                    }
                    if current_descriptor.is_some() {
                        self.normalize_property_descriptor(
                            current_descriptor.as_ref(),
                            key,
                            descriptor,
                        )?;
                    }
                }
                self.define_property_on_object_entries(
                    &mut array.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            Value::Function(function) => {
                if Self::is_function_builtin_prototype_key(function, key) {
                    let current_descriptor = self
                        .function_own_property_descriptor_value(function, key)
                        .unwrap_or_else(|| {
                            Self::own_data_property_descriptor_with_attrs(
                                Value::Object(function.prototype_object.clone()),
                                true,
                                false,
                                false,
                            )
                        });
                    let normalized = self.normalize_property_descriptor(
                        Some(&current_descriptor),
                        key,
                        descriptor,
                    )?;
                    let NormalizedOwnPropertyDescriptor::Data {
                        value, writable, ..
                    } = normalized
                    else {
                        return Err(Self::redefine_property_error(key));
                    };

                    let entries = self
                        .script_runtime
                        .function_public_properties
                        .entry(function.function_id)
                        .or_default();
                    Self::set_function_builtin_prototype_property(entries, value, writable);
                    return Ok(object.clone());
                }

                let mut entries = self
                    .script_runtime
                    .function_public_properties
                    .remove(&function.function_id)
                    .unwrap_or_default();
                self.define_property_on_object_entries(&mut entries, key, descriptor)?;
                self.script_runtime
                    .function_public_properties
                    .insert(function.function_id, entries);
                Ok(object.clone())
            }
            Value::Node(node) => {
                let mut entries = ObjectValue::new(self.node_expando_entries(*node));
                self.define_property_on_object_entries(&mut entries, key, descriptor)?;
                self.replace_node_expando_entries(*node, entries.entries);
                Ok(object.clone())
            }
            Value::NodeList(nodes) => {
                let current_descriptor = {
                    let nodes_ref = nodes.borrow();
                    Self::own_property_descriptor_object_from_entries(&nodes_ref.properties, key)
                }
                .or_else(|| self.node_list_synthesized_descriptor_value(nodes, key));
                let normalized = self.normalize_property_descriptor(
                    current_descriptor.as_ref(),
                    key,
                    descriptor,
                )?;
                Self::apply_normalized_descriptor_to_object_entries(
                    &mut nodes.borrow_mut().properties,
                    key,
                    normalized,
                );
                Ok(object.clone())
            }
            Value::Map(map) => {
                self.define_property_on_object_entries(
                    &mut map.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            Value::WeakMap(map) => {
                self.define_property_on_object_entries(
                    &mut map.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            Value::Set(set) => {
                self.define_property_on_object_entries(
                    &mut set.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            Value::WeakSet(set) => {
                self.define_property_on_object_entries(
                    &mut set.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            Value::RegExp(regex) => {
                if key == "lastIndex" {
                    let current_descriptor = {
                        let regex_ref = regex.borrow();
                        self.regexp_builtin_descriptor_value(&regex_ref, key)
                            .unwrap_or_else(|| {
                                Self::own_data_property_descriptor_with_attrs(
                                    Value::Number(regex_ref.last_index as i64),
                                    true,
                                    false,
                                    false,
                                )
                            })
                    };
                    let normalized = self.normalize_property_descriptor(
                        Some(&current_descriptor),
                        key,
                        descriptor,
                    )?;
                    let NormalizedOwnPropertyDescriptor::Data {
                        value, writable, ..
                    } = normalized
                    else {
                        return Err(Self::redefine_property_error(key));
                    };
                    let mut regex_ref = regex.borrow_mut();
                    Self::delete_object_property_entries(&mut regex_ref.properties, key);
                    let next = Self::value_to_i64(&value);
                    regex_ref.last_index = if next <= 0 { 0 } else { next as usize };
                    Self::set_object_property_flags(
                        &mut regex_ref.properties,
                        key,
                        writable,
                        false,
                        false,
                    );
                    return Ok(object.clone());
                }
                self.define_property_on_object_entries(
                    &mut regex.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            _ => Err(Error::ScriptRuntime(
                "Object.defineProperty target must be an object".into(),
            )),
        }
    }

    fn reflect_set_on_receiver_object(
        &mut self,
        receiver: &Value,
        key: &str,
        value: Value,
    ) -> Result<bool> {
        match receiver {
            Value::Object(entries) => {
                if Self::string_wrapper_builtin_has_own_property(&entries.borrow(), key) {
                    return Ok(false);
                }
                let (setter, has_accessor, own_data, writable) = {
                    let entries_ref = entries.borrow();
                    (
                        Self::object_setter_from_entries(&*entries_ref, key),
                        Self::has_object_accessor_property(&*entries_ref, key),
                        Self::object_get_entry(&*entries_ref, key).is_some(),
                        Self::is_writable_object_key(&*entries_ref, key),
                    )
                };
                if let Some(setter) = setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        &event,
                        None,
                        Some(receiver.clone()),
                    )?;
                    return Ok(true);
                }
                if has_accessor {
                    return Ok(false);
                }
                if own_data && !writable {
                    return Ok(false);
                }
                if !own_data
                    && Self::callable_kind_from_value(receiver).is_some()
                    && Self::is_callable_own_surface_key(key)
                {
                    return Ok(false);
                }
                Self::object_set_entry(&mut entries.borrow_mut(), key.to_string(), value);
                Ok(true)
            }
            Value::Array(array) => {
                if key == "length" {
                    if !Self::is_writable_object_key(&array.borrow().properties, key) {
                        return Ok(false);
                    }
                    let mut values = array.borrow_mut();
                    let next = Self::value_to_i64(&value);
                    let next = if next <= 0 { 0usize } else { next as usize };
                    if next < values.len() {
                        values.truncate(next);
                    } else if next > values.len() {
                        values.resize(next, Value::Undefined);
                    }
                    return Ok(true);
                }
                if let Ok(index) = key.parse::<usize>() {
                    let key_string = index.to_string();
                    let (setter, has_accessor, own_data, writable) = {
                        let values = array.borrow();
                        (
                            Self::object_setter_from_entries(&values.properties, &key_string),
                            Self::has_object_accessor_property(&values.properties, &key_string),
                            Self::object_get_entry(&values.properties, &key_string).is_some(),
                            Self::is_writable_object_key(&values.properties, &key_string),
                        )
                    };
                    if let Some(setter) = setter {
                        if !self.is_callable_value(&setter) {
                            return Err(Error::ScriptRuntime(
                                "object setter is not callable".into(),
                            ));
                        }
                        let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                        self.execute_callable_value_with_this_and_env(
                            &setter,
                            &[value],
                            &event,
                            None,
                            Some(receiver.clone()),
                        )?;
                        return Ok(true);
                    }
                    if has_accessor {
                        return Ok(false);
                    }
                    if own_data && !writable {
                        return Ok(false);
                    }
                    {
                        let mut values = array.borrow_mut();
                        if index >= values.len() {
                            values.resize(index + 1, Value::Undefined);
                        }
                        values[index] = value;
                    }
                    Self::clear_array_hole(array, index);
                    return Ok(true);
                }
                let (setter, has_accessor, own_data, writable) = {
                    let values = array.borrow();
                    (
                        Self::object_setter_from_entries(&values.properties, key),
                        Self::has_object_accessor_property(&values.properties, key),
                        Self::object_get_entry(&values.properties, key).is_some(),
                        Self::is_writable_object_key(&values.properties, key),
                    )
                };
                if let Some(setter) = setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        &event,
                        None,
                        Some(receiver.clone()),
                    )?;
                    return Ok(true);
                }
                if has_accessor {
                    return Ok(false);
                }
                if own_data && !writable {
                    return Ok(false);
                }
                Self::object_set_entry(&mut array.borrow_mut().properties, key.to_string(), value);
                Ok(true)
            }
            Value::Function(function) => {
                let (setter, has_accessor, own_data, writable) = self
                    .script_runtime
                    .function_public_properties
                    .get(&function.function_id)
                    .map(|entries| {
                        (
                            Self::object_setter_from_entries(entries, key),
                            Self::has_object_accessor_property(entries, key),
                            Self::object_get_entry(entries, key).is_some(),
                            Self::is_writable_object_key(entries, key),
                        )
                    })
                    .unwrap_or((None, false, false, true));
                if let Some(setter) = setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        &event,
                        None,
                        Some(receiver.clone()),
                    )?;
                    return Ok(true);
                }
                if has_accessor {
                    return Ok(false);
                }
                if own_data && !writable {
                    return Ok(false);
                }
                if !own_data && Self::is_callable_own_surface_key(key) {
                    return Ok(false);
                }
                let entries = self
                    .script_runtime
                    .function_public_properties
                    .entry(function.function_id)
                    .or_default();
                if Self::is_function_builtin_prototype_key(function, key) {
                    Self::set_function_builtin_prototype_property(entries, value, true);
                } else {
                    Self::object_set_entry(entries, key.to_string(), value);
                }
                Ok(true)
            }
            Value::Map(map) => {
                let (setter, has_accessor, own_data, writable) = {
                    let map_ref = map.borrow();
                    (
                        Self::object_setter_from_entries(&map_ref.properties, key),
                        Self::has_object_accessor_property(&map_ref.properties, key),
                        Self::object_get_entry(&map_ref.properties, key).is_some(),
                        Self::is_writable_object_key(&map_ref.properties, key),
                    )
                };
                if let Some(setter) = setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        &event,
                        None,
                        Some(receiver.clone()),
                    )?;
                    return Ok(true);
                }
                if has_accessor || (own_data && !writable) {
                    return Ok(false);
                }
                if !own_data && key == "size" {
                    return Ok(false);
                }
                Self::object_set_entry(&mut map.borrow_mut().properties, key.to_string(), value);
                Ok(true)
            }
            Value::WeakMap(map) => {
                let (setter, has_accessor, own_data, writable) = {
                    let map_ref = map.borrow();
                    (
                        Self::object_setter_from_entries(&map_ref.properties, key),
                        Self::has_object_accessor_property(&map_ref.properties, key),
                        Self::object_get_entry(&map_ref.properties, key).is_some(),
                        Self::is_writable_object_key(&map_ref.properties, key),
                    )
                };
                if let Some(setter) = setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        &event,
                        None,
                        Some(receiver.clone()),
                    )?;
                    return Ok(true);
                }
                if has_accessor || (own_data && !writable) {
                    return Ok(false);
                }
                Self::object_set_entry(&mut map.borrow_mut().properties, key.to_string(), value);
                Ok(true)
            }
            Value::Set(set) => {
                let (setter, has_accessor, own_data, writable) = {
                    let set_ref = set.borrow();
                    (
                        Self::object_setter_from_entries(&set_ref.properties, key),
                        Self::has_object_accessor_property(&set_ref.properties, key),
                        Self::object_get_entry(&set_ref.properties, key).is_some(),
                        Self::is_writable_object_key(&set_ref.properties, key),
                    )
                };
                if let Some(setter) = setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        &event,
                        None,
                        Some(receiver.clone()),
                    )?;
                    return Ok(true);
                }
                if has_accessor || (own_data && !writable) {
                    return Ok(false);
                }
                if !own_data && key == "size" {
                    return Ok(false);
                }
                Self::object_set_entry(&mut set.borrow_mut().properties, key.to_string(), value);
                Ok(true)
            }
            Value::WeakSet(set) => {
                let (setter, has_accessor, own_data, writable) = {
                    let set_ref = set.borrow();
                    (
                        Self::object_setter_from_entries(&set_ref.properties, key),
                        Self::has_object_accessor_property(&set_ref.properties, key),
                        Self::object_get_entry(&set_ref.properties, key).is_some(),
                        Self::is_writable_object_key(&set_ref.properties, key),
                    )
                };
                if let Some(setter) = setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        &event,
                        None,
                        Some(receiver.clone()),
                    )?;
                    return Ok(true);
                }
                if has_accessor || (own_data && !writable) {
                    return Ok(false);
                }
                Self::object_set_entry(&mut set.borrow_mut().properties, key.to_string(), value);
                Ok(true)
            }
            Value::RegExp(regex) => {
                if key == "lastIndex" {
                    if !Self::is_writable_object_key(&regex.borrow().properties, key) {
                        return Ok(false);
                    }
                    let mut regex_ref = regex.borrow_mut();
                    let next = Self::value_to_i64(&value);
                    regex_ref.last_index = if next <= 0 { 0 } else { next as usize };
                } else {
                    let (setter, has_accessor, own_data, writable) = {
                        let regex_ref = regex.borrow();
                        (
                            Self::object_setter_from_entries(&regex_ref.properties, key),
                            Self::has_object_accessor_property(&regex_ref.properties, key),
                            Self::object_get_entry(&regex_ref.properties, key).is_some(),
                            Self::is_writable_object_key(&regex_ref.properties, key),
                        )
                    };
                    if let Some(setter) = setter {
                        if !self.is_callable_value(&setter) {
                            return Err(Error::ScriptRuntime(
                                "object setter is not callable".into(),
                            ));
                        }
                        let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                        self.execute_callable_value_with_this_and_env(
                            &setter,
                            &[value],
                            &event,
                            None,
                            Some(receiver.clone()),
                        )?;
                        return Ok(true);
                    }
                    if has_accessor || (own_data && !writable) {
                        return Ok(false);
                    }
                    if !own_data && Self::is_regexp_builtin_own_key(key) {
                        return Ok(false);
                    }
                    Self::object_set_entry(
                        &mut regex.borrow_mut().properties,
                        key.to_string(),
                        value,
                    );
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn reflect_set_object_property_value(
        &mut self,
        target: &Value,
        key: &str,
        value: Value,
        receiver: &Value,
        event: &EventState,
    ) -> Result<bool> {
        let Value::Object(object) = target else {
            let mut assign_env = HashMap::new();
            let key_value = if let Some(symbol_id) = Self::symbol_id_from_storage_key(key) {
                if let Some(symbol) = self.symbol_runtime.symbols_by_id.get(&symbol_id) {
                    Value::Symbol(symbol.clone())
                } else {
                    Value::String(key.to_string())
                }
            } else {
                Value::String(key.to_string())
            };
            let ok = self
                .set_object_assignment_property(
                    receiver,
                    &key_value,
                    value,
                    "Reflect.set target",
                    &mut assign_env,
                    event,
                )
                .is_ok();
            return Ok(ok);
        };

        let (own_setter, own_has_accessor, own_data, own_builtin, mut prototype) = {
            let entries = object.borrow();
            (
                Self::object_setter_from_entries(&*entries, key),
                Self::has_object_accessor_property(&*entries, key),
                Self::object_get_entry(&*entries, key).is_some(),
                Self::string_wrapper_builtin_has_own_property(&entries, key),
                Self::object_get_entry(&*entries, INTERNAL_OBJECT_PROTOTYPE_KEY),
            )
        };

        if let Some(setter) = own_setter {
            if !self.is_callable_value(&setter) {
                return Err(Error::ScriptRuntime("object setter is not callable".into()));
            }
            self.execute_callable_value_with_this_and_env(
                &setter,
                &[value],
                event,
                None,
                Some(receiver.clone()),
            )?;
            return Ok(true);
        }
        if own_has_accessor {
            return Ok(false);
        }
        if own_builtin {
            return Ok(false);
        }
        if own_data {
            return self.reflect_set_on_receiver_object(receiver, key, value);
        }

        while let Some(Value::Object(proto)) = prototype {
            let (setter, has_accessor, next) = {
                let proto_ref = proto.borrow();
                (
                    Self::object_setter_from_entries(&*proto_ref, key),
                    Self::has_object_accessor_property(&*proto_ref, key),
                    Self::object_get_entry(&*proto_ref, INTERNAL_OBJECT_PROTOTYPE_KEY),
                )
            };
            if let Some(setter) = setter {
                if !self.is_callable_value(&setter) {
                    return Err(Error::ScriptRuntime("object setter is not callable".into()));
                }
                self.execute_callable_value_with_this_and_env(
                    &setter,
                    &[value],
                    event,
                    None,
                    Some(receiver.clone()),
                )?;
                return Ok(true);
            }
            if has_accessor {
                return Ok(false);
            }
            prototype = next;
        }

        self.reflect_set_on_receiver_object(receiver, key, value)
    }

    pub(crate) fn object_get_own_property_symbols_value(
        &mut self,
        object: &Value,
    ) -> Result<Value> {
        match self.object_like_own_symbol_values(object) {
            Ok(symbols) => Ok(Self::new_array_value(symbols)),
            Err(_) => Err(Error::ScriptRuntime(
                "Object.getOwnPropertySymbols argument must be an object".into(),
            )),
        }
    }

    pub(crate) fn object_keys_value(&mut self, object: &Value) -> Result<Value> {
        let keys = self
            .object_like_enumerable_keys(object)?
            .into_iter()
            .map(Value::String)
            .collect::<Vec<_>>();
        Ok(Self::new_array_value(keys))
    }

    pub(crate) fn object_values_value(&mut self, object: &Value) -> Result<Value> {
        let keys = self.object_like_enumerable_keys(object)?;
        let mut values = Vec::with_capacity(keys.len());
        for key in keys {
            values.push(self.object_property_from_value(object, &key)?);
        }
        Ok(Self::new_array_value(values))
    }

    pub(crate) fn object_entries_value(&mut self, object: &Value) -> Result<Value> {
        let keys = self.object_like_enumerable_keys(object)?;
        let mut values = Vec::with_capacity(keys.len());
        for key in keys {
            let value = self.object_property_from_value(object, &key)?;
            values.push(Self::new_array_value(vec![Value::String(key), value]));
        }
        Ok(Self::new_array_value(values))
    }

    pub(crate) fn object_from_entries_value(&mut self, iterable: &Value) -> Result<Value> {
        let entries = self.array_like_values_from_value(iterable).map_err(|_| {
            Error::ScriptRuntime("Object.fromEntries argument must be iterable".into())
        })?;
        let object = Self::new_object_value(Vec::new());
        let Value::Object(object_entries) = &object else {
            unreachable!("new_object_value always returns an object");
        };
        let mut object_entries = object_entries.borrow_mut();
        for entry in entries {
            let pair = self.array_like_values_from_value(&entry).map_err(|_| {
                Error::ScriptRuntime(
                    "Object.fromEntries iterable values must be [key, value] pairs".into(),
                )
            })?;
            if pair.len() < 2 {
                return Err(Error::ScriptRuntime(
                    "Object.fromEntries iterable values must be [key, value] pairs".into(),
                ));
            }
            let key = self.property_key_to_storage_key(&pair[0]);
            Self::delete_object_property_auxiliary_entries(&mut object_entries, &key);
            Self::object_set_entry(&mut object_entries, key, pair[1].clone());
        }
        drop(object_entries);
        Ok(object)
    }

    pub(crate) fn object_has_own_value(&mut self, object: &Value, key: &str) -> Result<Value> {
        match object {
            Value::Object(entries) => {
                let entries = entries.borrow();
                Ok(Value::Bool(
                    Self::object_get_entry(&*entries, key).is_some()
                        || Self::has_object_accessor_property(&*entries, key)
                        || self
                            .class_list_synthesized_descriptor_value(&entries, key)
                            .is_some()
                        || self
                            .named_node_map_synthesized_descriptor_value(&entries, key)
                            .is_some()
                        || self
                            .dom_string_map_synthesized_descriptor_value(&entries, key)
                            .is_some()
                        || Self::string_wrapper_builtin_has_own_property(&entries, key)
                        || (self.callable_own_surface_value(object, key).is_some()
                            && !Self::is_builtin_object_property_deleted(&*entries, key)),
                ))
            }
            Value::Array(array) => {
                let array_ref = array.borrow();
                let has = if key == "length" {
                    true
                } else if let Ok(index) = key.parse::<usize>() {
                    index < array_ref.len() && !Self::array_index_is_hole(&array_ref, index)
                } else {
                    Self::object_get_entry(&array_ref.properties, key).is_some()
                        || Self::has_object_accessor_property(&array_ref.properties, key)
                };
                Ok(Value::Bool(has))
            }
            Value::Node(node) => {
                if self.node_has_explicit_own_property(*node, key) {
                    return Ok(Value::Bool(true));
                }
                let is_form = self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("form"));
                if !is_form {
                    let is_media = self.dom.tag_name(*node).is_some_and(|tag| {
                        tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                    });
                    if !is_media {
                        return Ok(Value::Bool(false));
                    }
                    return Ok(Value::Bool(
                        self.html_media_builtin_property_value(*node, key)?
                            .is_some(),
                    ));
                }
                Ok(Value::Bool(
                    self.html_form_builtin_property_value(*node, key)?.is_some()
                        || self.form_named_property_value(*node, key)?.is_some(),
                ))
            }
            Value::NodeList(nodes) => {
                let snapshot = self.node_list_snapshot(nodes);
                let has_own_surface = {
                    let nodes_ref = nodes.borrow();
                    Self::object_get_entry(&nodes_ref.properties, key).is_some()
                        || Self::has_object_accessor_property(&nodes_ref.properties, key)
                };
                Ok(Value::Bool(
                    key == "length"
                        || key
                            .parse::<usize>()
                            .ok()
                            .is_some_and(|index| index < snapshot.len())
                        || self
                            .html_collection_named_property_value(nodes, key)
                            .is_some()
                        || has_own_surface,
                ))
            }
            Value::Function(function) => Ok(Value::Bool(
                self.script_runtime
                    .function_public_properties
                    .get(&function.function_id)
                    .is_some_and(|entries| {
                        Self::object_get_entry(entries, key).is_some()
                            || Self::has_object_accessor_property(entries, key)
                    })
                    || self
                        .function_builtin_own_property_value(function, key)
                        .is_some()
                        && !self
                            .script_runtime
                            .function_public_properties
                            .get(&function.function_id)
                            .is_some_and(|entries| {
                                Self::is_builtin_object_property_deleted(entries, key)
                            }),
            )),
            Value::Map(map) => {
                let map = map.borrow();
                Ok(Value::Bool(
                    (key == "size"
                        && !Self::is_builtin_object_property_deleted(&map.properties, key))
                        || Self::object_get_entry(&map.properties, key).is_some()
                        || Self::has_object_accessor_property(&map.properties, key),
                ))
            }
            Value::WeakMap(map) => {
                let map = map.borrow();
                Ok(Value::Bool(
                    Self::object_get_entry(&map.properties, key).is_some()
                        || Self::has_object_accessor_property(&map.properties, key),
                ))
            }
            Value::Set(set) => {
                let set = set.borrow();
                Ok(Value::Bool(
                    (key == "size"
                        && !Self::is_builtin_object_property_deleted(&set.properties, key))
                        || Self::object_get_entry(&set.properties, key).is_some()
                        || Self::has_object_accessor_property(&set.properties, key),
                ))
            }
            Value::WeakSet(set) => {
                let set = set.borrow();
                Ok(Value::Bool(
                    Self::object_get_entry(&set.properties, key).is_some()
                        || Self::has_object_accessor_property(&set.properties, key),
                ))
            }
            Value::RegExp(regex) => {
                let regex = regex.borrow();
                Ok(Value::Bool(
                    key == "lastIndex"
                        || Self::object_get_entry(&regex.properties, key).is_some()
                        || Self::has_object_accessor_property(&regex.properties, key),
                ))
            }
            _ => Err(Error::ScriptRuntime(
                "Object.hasOwn first argument must be an object".into(),
            )),
        }
    }

    pub(crate) fn object_get_prototype_of_value(&mut self, value: &Value) -> Result<Value> {
        if let Value::TypedArrayConstructor(TypedArrayConstructorKind::Concrete(_)) = value {
            if let Some(prototype) = self.variant_callable_internal_prototype_value(value) {
                return Ok(prototype);
            }
            return Ok(Value::TypedArrayConstructor(
                TypedArrayConstructorKind::Abstract,
            ));
        }
        Ok(self
            .value_internal_prototype_value(value)
            .unwrap_or_else(|| Value::Object(Rc::new(RefCell::new(ObjectValue::default())))))
    }

    pub(crate) fn object_create_value(
        &mut self,
        prototype: &Value,
        properties: Option<&Value>,
    ) -> Result<Value> {
        if !matches!(prototype, Value::Null) && Self::is_primitive_value(prototype) {
            return Err(Error::ScriptRuntime(
                "Object prototype may only be an Object or null".into(),
            ));
        }

        let created = Self::new_object_value(Vec::new());
        let Value::Object(entries) = &created else {
            unreachable!("new_object_value always returns an object");
        };
        Self::set_internal_prototype(entries, prototype.clone());

        if let Some(properties) = properties
            && !matches!(properties, Value::Undefined)
        {
            let own_keys = self.reflect_own_keys_value(properties)?;
            let Value::Array(keys) = own_keys else {
                unreachable!("Reflect.ownKeys returns an array");
            };
            for key in keys.borrow().iter() {
                let storage_key = self.property_key_to_storage_key(key);
                let descriptor =
                    self.object_get_own_property_descriptor_value(properties, &storage_key)?;
                if !matches!(descriptor, Value::Object(_)) {
                    continue;
                }
                let enumerable = self
                    .object_property_from_value(&descriptor, "enumerable")?
                    .truthy();
                if !enumerable {
                    continue;
                }
                let property_descriptor =
                    self.object_property_from_value(properties, &storage_key)?;
                self.object_define_property_value(&created, &storage_key, &property_descriptor)?;
            }
        }

        Ok(created)
    }

    fn object_set_prototype_would_cycle(&mut self, target: &Value, prototype: &Value) -> bool {
        let mut current = Some(prototype.clone());
        let mut hops = 0usize;
        while let Some(value) = current {
            if self.strict_equal(target, &value) {
                return true;
            }
            hops += 1;
            if hops > 256 {
                break;
            }
            current = self.value_internal_prototype_value(&value);
        }
        false
    }

    fn set_object_like_internal_prototype(
        &mut self,
        target: &Value,
        prototype: Value,
    ) -> Result<()> {
        match target {
            Value::Object(entries) => {
                Self::object_set_entry(
                    &mut entries.borrow_mut(),
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::Function(function) => {
                let entries = self
                    .script_runtime
                    .function_public_properties
                    .entry(function.function_id)
                    .or_default();
                Self::object_set_entry(
                    entries,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::Array(values) => {
                Self::object_set_entry(
                    &mut values.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::Map(map) => {
                Self::object_set_entry(
                    &mut map.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::WeakMap(map) => {
                Self::object_set_entry(
                    &mut map.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::Set(set) => {
                Self::object_set_entry(
                    &mut set.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::WeakSet(set) => {
                Self::object_set_entry(
                    &mut set.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::RegExp(regex) => {
                Self::object_set_entry(
                    &mut regex.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::TypedArray(values) => {
                Self::object_set_entry(
                    &mut values.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::NodeList(nodes) => {
                Self::object_set_entry(
                    &mut nodes.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::UrlConstructor => {
                Self::object_set_entry(
                    &mut self.browser_apis.url_constructor_properties.borrow_mut(),
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            _ if Self::variant_callable_public_storage_key(target).is_some() => {
                let storage_key = Self::variant_callable_public_storage_key(target)
                    .expect("checked variant callable storage key");
                let entries = self
                    .script_runtime
                    .variant_callable_public_properties
                    .entry(storage_key)
                    .or_default();
                Self::object_set_entry(
                    entries,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            _ => Err(Error::ScriptRuntime(
                "Object.setPrototypeOf target must be an object".into(),
            )),
        }
    }

    pub(crate) fn object_set_prototype_of_value(
        &mut self,
        target: &Value,
        prototype: &Value,
    ) -> Result<Value> {
        if !matches!(prototype, Value::Null) && Self::is_primitive_value(prototype) {
            return Err(Error::ScriptRuntime(
                "Object.setPrototypeOf prototype must be an object or null".into(),
            ));
        }

        if matches!(target, Value::Null | Value::Undefined) {
            return Err(Error::ScriptRuntime(
                "Object.setPrototypeOf target must be an object".into(),
            ));
        }
        if Self::is_primitive_value(target) {
            return Ok(target.clone());
        }

        if self.object_set_prototype_would_cycle(target, prototype) {
            return Err(Error::ScriptRuntime("Cyclic __proto__ value".into()));
        }
        self.set_object_like_internal_prototype(target, prototype.clone())?;
        Ok(target.clone())
    }

    pub(crate) fn object_freeze_value(&mut self, value: &Value) -> Result<Value> {
        match value {
            Value::TypedArray(array) => {
                if array.borrow().observed_length() > 0 {
                    return Err(Error::ScriptRuntime(
                        "Cannot freeze array buffer views with elements".into(),
                    ));
                }
                Ok(Value::TypedArray(array.clone()))
            }
            other => Ok(other.clone()),
        }
    }

    pub(crate) fn resolve_target_value_with_pending(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Option<Value> {
        self.resolve_listener_capture_pending_value(target)
            .flatten()
            .or_else(|| env.get(target).cloned())
            .or_else(|| self.resolve_runtime_global_identifier(target))
    }

    pub(crate) fn resolve_runtime_global_identifier(&self, name: &str) -> Option<Value> {
        self.script_runtime.env.get(name).cloned().or_else(|| {
            if Self::is_internal_env_key(name) {
                return None;
            }
            let window = self.dom_runtime.window_object.borrow();
            Self::object_get_entry(&window, name)
        })
    }

    pub(crate) fn eval_expr_json_object_array(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::JsonParse(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    Self::parse_json_text(&value)
                }
                Expr::JsonStringify {
                    value,
                    replacer,
                    space,
                } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let _evaluated_replacer = replacer
                        .as_ref()
                        .map(|replacer| self.eval_expr(replacer, env, event_param, event))
                        .transpose()?;
                    let evaluated_space = space
                        .as_ref()
                        .map(|space| self.eval_expr(space, env, event_param, event))
                        .transpose()?;
                    match Self::json_stringify_top_level(&value, evaluated_space.as_ref())? {
                        Some(serialized) => Ok(Value::String(serialized)),
                        None => Ok(Value::Undefined),
                    }
                }
                Expr::ObjectConstruct { value } => {
                    let value = value
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .unwrap_or(Value::Undefined);
                    match value {
                        Value::Null | Value::Undefined => Ok(Self::new_object_value(Vec::new())),
                        Value::Object(object) => Ok(Value::Object(object)),
                        Value::Array(array) => Ok(Value::Array(array)),
                        Value::Date(date) => Ok(Value::Date(date)),
                        Value::Map(map) => Ok(Value::Map(map)),
                        Value::Set(set) => Ok(Value::Set(set)),
                        Value::Blob(blob) => Ok(Value::Blob(blob)),
                        Value::ArrayBuffer(buffer) => Ok(Value::ArrayBuffer(buffer)),
                        Value::TypedArray(array) => Ok(Value::TypedArray(array)),
                        Value::Promise(promise) => Ok(Value::Promise(promise)),
                        Value::RegExp(regex) => Ok(Value::RegExp(regex)),
                        primitive => Ok(Self::box_primitive_value(primitive)),
                    }
                }
                Expr::ObjectLiteral(entries) => {
                    let mut object_entries = Vec::with_capacity(entries.len());
                    for entry in entries {
                        match entry {
                            ObjectLiteralEntry::Pair(key, value) => {
                                let key = match key {
                                    ObjectLiteralKey::Static(key) => key.clone(),
                                    ObjectLiteralKey::Computed(expr) => {
                                        let key = self.eval_expr(expr, env, event_param, event)?;
                                        self.property_key_to_storage_key(&key)
                                    }
                                };

                                let value = match value {
                                    Expr::Function {
                                        handler,
                                        name: _,
                                        is_async,
                                        is_generator,
                                        is_arrow,
                                        is_method,
                                    } if *is_method => {
                                        let super_prototype = match Self::object_get_entry(
                                            &object_entries,
                                            INTERNAL_OBJECT_PROTOTYPE_KEY,
                                        ) {
                                            Some(Value::Object(proto)) => {
                                                Some(Value::Object(proto))
                                            }
                                            _ => None,
                                        };
                                        self.make_function_value_with_super(
                                            handler.clone(),
                                            env,
                                            false,
                                            *is_async,
                                            *is_generator,
                                            *is_arrow,
                                            *is_method,
                                            None,
                                            super_prototype,
                                        )
                                    }
                                    _ => self.eval_expr(value, env, event_param, event)?,
                                };

                                Self::define_object_literal_data_entry(
                                    &mut object_entries,
                                    key,
                                    value,
                                );
                            }
                            ObjectLiteralEntry::ProtoSetter(expr) => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                if matches!(value, Value::Object(_) | Value::Null) {
                                    Self::object_set_entry(
                                        &mut object_entries,
                                        INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                                        value,
                                    );
                                }
                            }
                            ObjectLiteralEntry::Getter(key, handler) => {
                                let key = match key {
                                    ObjectLiteralKey::Static(key) => key.clone(),
                                    ObjectLiteralKey::Computed(expr) => {
                                        let key = self.eval_expr(expr, env, event_param, event)?;
                                        self.property_key_to_storage_key(&key)
                                    }
                                };
                                let getter = self.make_function_value(
                                    handler.clone(),
                                    env,
                                    false,
                                    false,
                                    false,
                                    false,
                                    true,
                                );
                                Self::define_object_literal_getter_entry(
                                    &mut object_entries,
                                    key,
                                    getter,
                                );
                            }
                            ObjectLiteralEntry::Setter(key, handler) => {
                                let key = match key {
                                    ObjectLiteralKey::Static(key) => key.clone(),
                                    ObjectLiteralKey::Computed(expr) => {
                                        let key = self.eval_expr(expr, env, event_param, event)?;
                                        self.property_key_to_storage_key(&key)
                                    }
                                };
                                let setter = self.make_function_value(
                                    handler.clone(),
                                    env,
                                    false,
                                    false,
                                    false,
                                    false,
                                    true,
                                );
                                Self::define_object_literal_setter_entry(
                                    &mut object_entries,
                                    key,
                                    setter,
                                );
                            }
                            ObjectLiteralEntry::Spread(expr) => {
                                let spread_value = self.eval_expr(expr, env, event_param, event)?;
                                match spread_value {
                                    Value::Null | Value::Undefined => {}
                                    Value::Object(entries) => {
                                        let source = Value::Object(entries.clone());
                                        let keys = self.object_like_enumerable_keys(&source)?;
                                        for key in keys {
                                            let value =
                                                self.object_property_from_value(&source, &key)?;
                                            Self::define_object_literal_data_entry(
                                                &mut object_entries,
                                                key,
                                                value,
                                            );
                                        }
                                    }
                                    Value::NodeList(nodes) => {
                                        let source = Value::NodeList(nodes.clone());
                                        let keys = self.object_like_enumerable_keys(&source)?;
                                        for key in keys {
                                            let value =
                                                self.object_property_from_value(&source, &key)?;
                                            Self::define_object_literal_data_entry(
                                                &mut object_entries,
                                                key,
                                                value,
                                            );
                                        }
                                    }
                                    Value::Node(node) => {
                                        let source = Value::Node(node);
                                        let keys = self.object_like_enumerable_keys(&source)?;
                                        for key in keys {
                                            let value =
                                                self.object_property_from_value(&source, &key)?;
                                            Self::define_object_literal_data_entry(
                                                &mut object_entries,
                                                key,
                                                value,
                                            );
                                        }
                                    }
                                    Value::Array(values) => {
                                        for (index, value) in values.borrow().iter().enumerate() {
                                            let key = index.to_string();
                                            Self::define_object_literal_data_entry(
                                                &mut object_entries,
                                                key,
                                                value.clone(),
                                            );
                                        }
                                    }
                                    Value::String(text) => {
                                        for (index, ch) in text.chars().enumerate() {
                                            let key = index.to_string();
                                            Self::define_object_literal_data_entry(
                                                &mut object_entries,
                                                key,
                                                Value::String(ch.to_string()),
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Ok(Self::new_object_value(object_entries))
                }
                Expr::ObjectGet { target, key } => match self
                    .resolve_target_value_with_pending(env, target)
                {
                    _ if target == "super" => {
                        let super_prototype = Self::super_prototype_from_env(env)?;
                        let this_value = Self::super_this_from_env(env)?;
                        self.object_property_from_value_with_receiver(
                            &super_prototype,
                            key,
                            &this_value,
                        )
                    }
                    Some(value) => {
                        self.object_property_from_value(&value, key)
                            .map_err(|err| match err {
                                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                                    Error::ScriptRuntime(format!(
                                        "variable '{}' is not an object (key '{}')",
                                        target, key
                                    ))
                                }
                                other => other,
                            })
                    }
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ObjectPathGet { target, path } => {
                    if target == "super" {
                        let mut value = Self::super_prototype_from_env(env)?;
                        let this_value = Self::super_this_from_env(env)?;
                        for (index, key) in path.iter().enumerate() {
                            if index == 0 {
                                value = self.object_property_from_value_with_receiver(
                                    &value,
                                    key,
                                    &this_value,
                                )?;
                            } else {
                                value = self.object_property_from_value(&value, key)?;
                            }
                        }
                        Ok(value)
                    } else {
                        let Some(mut value) = self.resolve_target_value_with_pending(env, target)
                        else {
                            return Err(Error::ScriptRuntime(format!(
                                "unknown variable: {}",
                                target
                            )));
                        };
                        for key in path {
                            value = self.object_property_from_value(&value, key)?;
                        }
                        Ok(value)
                    }
                }
                Expr::ObjectGetOwnPropertySymbols(object) => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    self.object_get_own_property_symbols_value(&object)
                }
                Expr::ObjectGetOwnPropertyDescriptor { object, key } => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    let key = self.eval_expr(key, env, event_param, event)?;
                    let key = self.property_key_to_storage_key(&key);
                    self.object_get_own_property_descriptor_value(&object, &key)
                }
                Expr::ObjectDefineProperty {
                    object,
                    key,
                    descriptor,
                } => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    let key = self.eval_expr(key, env, event_param, event)?;
                    let key = self.property_key_to_storage_key(&key);
                    let descriptor = self.eval_expr(descriptor, env, event_param, event)?;
                    self.object_define_property_value(&object, &key, &descriptor)
                }
                Expr::ObjectGetOwnPropertyNames(object) => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    self.object_get_own_property_names_value(&object)
                }
                Expr::ObjectKeys(object) => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    self.object_keys_value(&object)
                }
                Expr::ObjectValues(object) => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    self.object_values_value(&object)
                }
                Expr::ObjectEntries(object) => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    self.object_entries_value(&object)
                }
                Expr::ObjectHasOwn { object, key } => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    let key = self.eval_expr(key, env, event_param, event)?;
                    let key = self.property_key_to_storage_key(&key);
                    self.object_has_own_value(&object, &key)
                }
                Expr::ObjectGetPrototypeOf(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    self.object_get_prototype_of_value(&value)
                }
                Expr::ObjectFreeze(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    self.object_freeze_value(&value)
                }
                Expr::ReflectSet {
                    target,
                    key,
                    value,
                    receiver,
                } => {
                    let target = self.eval_expr(target, env, event_param, event)?;
                    let key = self.eval_expr(key, env, event_param, event)?;
                    let key = self.property_key_to_storage_key(&key);
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let receiver = receiver
                        .as_ref()
                        .map(|receiver| self.eval_expr(receiver, env, event_param, event))
                        .transpose()?
                        .unwrap_or_else(|| target.clone());
                    Ok(Value::Bool(self.reflect_set_object_property_value(
                        &target, &key, value, &receiver, event,
                    )?))
                }
                Expr::ReflectOwnKeys(object) => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    self.reflect_own_keys_value(&object)
                }
                Expr::ObjectHasOwnProperty { target, key } => {
                    let key = self.eval_expr(key, env, event_param, event)?;
                    let key = self.property_key_to_storage_key(&key);
                    match self.resolve_target_value_with_pending(env, target) {
                        Some(value @ Value::Object(_)) => self.object_has_own_value(&value, &key),
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an object",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArrayConstruct { args, .. } => {
                    let evaluated =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    if evaluated.is_empty() {
                        return Ok(Self::new_array_value(Vec::new()));
                    }
                    if evaluated.len() == 1 {
                        let first = &evaluated[0];
                        if let Some(length) = Self::array_constructor_length_from_value(first)? {
                            let mut out = Vec::new();
                            out.resize(length, Value::Undefined);
                            return Ok(Self::new_array_value(out));
                        }
                        return Ok(Self::new_array_value(vec![first.clone()]));
                    }
                    Ok(Self::new_array_value(evaluated))
                }
                Expr::ArrayLiteral(values) => {
                    let mut out = Vec::with_capacity(values.len());
                    for value in values {
                        match value {
                            Expr::Spread(expr) => {
                                let spread_value = self.eval_expr(expr, env, event_param, event)?;
                                out.extend(self.spread_iterable_values_from_value(&spread_value)?);
                            }
                            _ => out.push(self.eval_expr(value, env, event_param, event)?),
                        }
                    }
                    Ok(Self::new_array_value(out))
                }
                Expr::ArrayIsArray(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    Ok(Value::Bool(matches!(value, Value::Array(_))))
                }
                Expr::ArrayFrom { source, map_fn } => {
                    let source = self.eval_expr(source, env, event_param, event)?;
                    let values = self.array_like_values_from_value_with_live_properties(&source)?;
                    if let Some(map_fn) = map_fn {
                        let callback = self.eval_expr(map_fn, env, event_param, event)?;
                        let mut mapped = Vec::with_capacity(values.len());
                        for (index, value) in values.into_iter().enumerate() {
                            mapped.push(self.execute_callback_value(
                                &callback,
                                &[value, Value::Number(index as i64)],
                                event,
                            )?);
                        }
                        return Ok(Self::new_array_value(mapped));
                    }
                    Ok(Self::new_array_value(values))
                }
                Expr::ArrayLength(target) => match self
                    .resolve_target_value_with_pending(env, target)
                {
                    Some(Value::Array(values)) => Ok(Value::Number(values.borrow().len() as i64)),
                    Some(Value::TypedArray(values)) => {
                        Ok(Value::Number(values.borrow().observed_length() as i64))
                    }
                    Some(Value::NodeList(nodes)) => {
                        let receiver = Value::NodeList(nodes.clone());
                        let own_override = {
                            let nodes_ref = nodes.borrow();
                            self.object_property_from_entries_with_getter(
                                &receiver,
                                &nodes_ref.properties,
                                "length",
                            )?
                        };
                        if let Some(value) = own_override {
                            Ok(value)
                        } else {
                            Ok(Value::Number(self.node_list_len(&nodes) as i64))
                        }
                    }
                    Some(Value::String(value)) => Ok(Value::Number(value.chars().count() as i64)),
                    Some(Value::Function(function)) => {
                        let function_value = Value::Function(function.clone());
                        if let Some(custom) = self
                            .function_public_property_from_entries_with_receiver(
                                &function,
                                "length",
                                &function_value,
                            )?
                        {
                            return Ok(custom);
                        }
                        if self
                            .script_runtime
                            .function_public_properties
                            .get(&function.function_id)
                            .is_some_and(|entries| {
                                Self::is_builtin_object_property_deleted(entries, "length")
                            })
                        {
                            return Ok(Value::Number(0));
                        }
                        let mut length = 0_i64;
                        for param in &function.handler.params {
                            if param.is_rest || param.default.is_some() {
                                break;
                            }
                            length += 1;
                        }
                        Ok(Value::Number(length))
                    }
                    Some(Value::Object(entries)) => {
                        let object = Value::Object(entries.clone());
                        let entries = entries.borrow();
                        if Self::is_history_object(&entries) {
                            return Ok(Self::object_get_entry(&entries, "length").unwrap_or(
                                Value::Number(self.location_history.history_entries.len() as i64),
                            ));
                        }
                        if Self::is_window_object(&entries) {
                            return Ok(Self::object_get_entry(&entries, "length")
                                .unwrap_or(Value::Number(0)));
                        }
                        if Self::is_storage_object(&entries) {
                            let len = Self::storage_pairs_from_object_entries(&entries).len();
                            return Ok(Value::Number(len as i64));
                        }
                        if let Some(value) = Self::string_wrapper_value_from_object(&entries) {
                            return Ok(Value::Number(value.chars().count() as i64));
                        }
                        drop(entries);
                        self.object_property_from_value(&object, "length")
                    }
                    Some(other) => self.object_property_from_value(&other, "length"),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArrayIndex { target, index } => {
                    let index = self.eval_expr(index, env, event_param, event)?;
                    let key = match &index {
                        Value::Number(value) => value.to_string(),
                        Value::BigInt(value) => value.to_string(),
                        Value::Float(value) if value.is_finite() && value.fract() == 0.0 => {
                            format!("{value:.0}")
                        }
                        other => self.property_key_to_storage_key(other),
                    };
                    if target == "super" {
                        let super_prototype = Self::super_prototype_from_env(env)?;
                        let this_value = Self::super_this_from_env(env)?;
                        return self.object_property_from_value_with_receiver(
                            &super_prototype,
                            &key,
                            &this_value,
                        );
                    }
                    match self.resolve_target_value_with_pending(env, target) {
                        Some(value) => self.object_property_from_value(&value, &key),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArrayPush { target, args } => {
                    let values = self.resolve_array_from_env(env, target)?;
                    let evaluated =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    let mut values = values.borrow_mut();
                    values.extend(evaluated);
                    Ok(Value::Number(values.len() as i64))
                }
                Expr::ArrayPop(target) => {
                    let values = self.resolve_array_from_env(env, target)?;
                    Ok(values.borrow_mut().pop().unwrap_or(Value::Undefined))
                }
                Expr::ArrayShift(target) => {
                    let values = self.resolve_array_from_env(env, target)?;
                    let mut values = values.borrow_mut();
                    if values.is_empty() {
                        Ok(Value::Undefined)
                    } else {
                        Ok(values.remove(0))
                    }
                }
                Expr::ArrayUnshift { target, args } => {
                    let values = self.resolve_array_from_env(env, target)?;
                    let evaluated =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    let mut values = values.borrow_mut();
                    for value in evaluated.into_iter().rev() {
                        values.insert(0, value);
                    }
                    Ok(Value::Number(values.len() as i64))
                }
                Expr::ArrayMap { target, callback } => {
                    match self.resolve_target_value_with_pending(env, target) {
                        Some(Value::Array(values)) => {
                            let input = values.borrow().clone();
                            let mut out = Vec::with_capacity(input.len());
                            for (idx, item) in input.into_iter().enumerate() {
                                let mapped = self.execute_array_callback(
                                    callback,
                                    &[
                                        item,
                                        Value::Number(idx as i64),
                                        Value::Array(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                out.push(mapped);
                            }
                            Ok(Self::new_array_value(out))
                        }
                        Some(Value::TypedArray(values)) => {
                            let input = self.typed_array_snapshot(&values)?;
                            let kind = values.borrow().kind;
                            let mut out = Vec::with_capacity(input.len());
                            for (idx, item) in input.into_iter().enumerate() {
                                let mapped = self.execute_array_callback(
                                    callback,
                                    &[
                                        item,
                                        Value::Number(idx as i64),
                                        Value::TypedArray(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                out.push(mapped);
                            }
                            self.new_typed_array_from_values(kind, &out)
                        }
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArrayFilter { target, callback } => {
                    match self.resolve_target_value_with_pending(env, target) {
                        Some(Value::Array(values)) => {
                            let input = values.borrow().clone();
                            let mut out = Vec::new();
                            for (idx, item) in input.into_iter().enumerate() {
                                let keep = self.execute_array_callback(
                                    callback,
                                    &[
                                        item.clone(),
                                        Value::Number(idx as i64),
                                        Value::Array(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                if keep.truthy() {
                                    out.push(item);
                                }
                            }
                            Ok(Self::new_array_value(out))
                        }
                        Some(Value::TypedArray(values)) => {
                            let input = self.typed_array_snapshot(&values)?;
                            let kind = values.borrow().kind;
                            let mut out = Vec::new();
                            for (idx, item) in input.into_iter().enumerate() {
                                let keep = self.execute_array_callback(
                                    callback,
                                    &[
                                        item.clone(),
                                        Value::Number(idx as i64),
                                        Value::TypedArray(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                if keep.truthy() {
                                    out.push(item);
                                }
                            }
                            self.new_typed_array_from_values(kind, &out)
                        }
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArrayReduce {
                    target,
                    callback,
                    initial,
                } => match self.resolve_target_value_with_pending(env, target) {
                    Some(Value::Array(values)) => {
                        let input = values.borrow().clone();
                        let mut start_index = 0usize;
                        let mut acc = if let Some(initial) = initial {
                            self.eval_expr(initial, env, event_param, event)?
                        } else {
                            let Some(first) = input.first().cloned() else {
                                return Err(Error::ScriptRuntime(
                                    "reduce of empty array with no initial value".into(),
                                ));
                            };
                            start_index = 1;
                            first
                        };
                        for (idx, item) in input.into_iter().enumerate().skip(start_index) {
                            acc = self.execute_array_callback(
                                callback,
                                &[
                                    acc,
                                    item,
                                    Value::Number(idx as i64),
                                    Value::Array(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                        }
                        Ok(acc)
                    }
                    Some(Value::TypedArray(values)) => {
                        let input = self.typed_array_snapshot(&values)?;
                        let mut start_index = 0usize;
                        let mut acc = if let Some(initial) = initial {
                            self.eval_expr(initial, env, event_param, event)?
                        } else {
                            let Some(first) = input.first().cloned() else {
                                return Err(Error::ScriptRuntime(
                                    "reduce of empty array with no initial value".into(),
                                ));
                            };
                            start_index = 1;
                            first
                        };
                        for (idx, item) in input.into_iter().enumerate().skip(start_index) {
                            acc = self.execute_array_callback(
                                callback,
                                &[
                                    acc,
                                    item,
                                    Value::Number(idx as i64),
                                    Value::TypedArray(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                        }
                        Ok(acc)
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArrayForEach { target, callback } => {
                    match self.resolve_target_value_with_pending(env, target) {
                        Some(Value::Array(values)) => {
                            let input = values.borrow().clone();
                            for (idx, item) in input.into_iter().enumerate() {
                                let _ = self.execute_array_callback(
                                    callback,
                                    &[
                                        item,
                                        Value::Number(idx as i64),
                                        Value::Array(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                            }
                            Ok(Value::Undefined)
                        }
                        Some(Value::TypedArray(values)) => {
                            let input = self.typed_array_snapshot(&values)?;
                            for (idx, item) in input.into_iter().enumerate() {
                                let _ = self.execute_array_callback(
                                    callback,
                                    &[
                                        item,
                                        Value::Number(idx as i64),
                                        Value::TypedArray(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                            }
                            Ok(Value::Undefined)
                        }
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArrayFind { target, callback } => {
                    match self.resolve_target_value_with_pending(env, target) {
                        Some(Value::Array(values)) => {
                            let input = values.borrow().clone();
                            for (idx, item) in input.into_iter().enumerate() {
                                let matched = self.execute_array_callback(
                                    callback,
                                    &[
                                        item.clone(),
                                        Value::Number(idx as i64),
                                        Value::Array(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                if matched.truthy() {
                                    return Ok(item);
                                }
                            }
                            Ok(Value::Undefined)
                        }
                        Some(Value::TypedArray(values)) => {
                            let input = self.typed_array_snapshot(&values)?;
                            for (idx, item) in input.into_iter().enumerate() {
                                let matched = self.execute_array_callback(
                                    callback,
                                    &[
                                        item.clone(),
                                        Value::Number(idx as i64),
                                        Value::TypedArray(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                if matched.truthy() {
                                    return Ok(item);
                                }
                            }
                            Ok(Value::Undefined)
                        }
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArrayFindIndex { target, callback } => {
                    match self.resolve_target_value_with_pending(env, target) {
                        Some(Value::Array(values)) => {
                            let input = values.borrow().clone();
                            for (idx, item) in input.into_iter().enumerate() {
                                let matched = self.execute_array_callback(
                                    callback,
                                    &[
                                        item,
                                        Value::Number(idx as i64),
                                        Value::Array(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                if matched.truthy() {
                                    return Ok(Value::Number(idx as i64));
                                }
                            }
                            Ok(Value::Number(-1))
                        }
                        Some(Value::TypedArray(values)) => {
                            let input = self.typed_array_snapshot(&values)?;
                            for (idx, item) in input.into_iter().enumerate() {
                                let matched = self.execute_array_callback(
                                    callback,
                                    &[
                                        item,
                                        Value::Number(idx as i64),
                                        Value::TypedArray(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                if matched.truthy() {
                                    return Ok(Value::Number(idx as i64));
                                }
                            }
                            Ok(Value::Number(-1))
                        }
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArraySome { target, callback } => {
                    match self.resolve_target_value_with_pending(env, target) {
                        Some(Value::Array(values)) => {
                            let input = values.borrow().clone();
                            for (idx, item) in input.into_iter().enumerate() {
                                let matched = self.execute_array_callback(
                                    callback,
                                    &[
                                        item,
                                        Value::Number(idx as i64),
                                        Value::Array(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                if matched.truthy() {
                                    return Ok(Value::Bool(true));
                                }
                            }
                            Ok(Value::Bool(false))
                        }
                        Some(Value::TypedArray(values)) => {
                            let input = self.typed_array_snapshot(&values)?;
                            for (idx, item) in input.into_iter().enumerate() {
                                let matched = self.execute_array_callback(
                                    callback,
                                    &[
                                        item,
                                        Value::Number(idx as i64),
                                        Value::TypedArray(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                if matched.truthy() {
                                    return Ok(Value::Bool(true));
                                }
                            }
                            Ok(Value::Bool(false))
                        }
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArrayEvery { target, callback } => {
                    match self.resolve_target_value_with_pending(env, target) {
                        Some(Value::Array(values)) => {
                            let input = values.borrow().clone();
                            for (idx, item) in input.into_iter().enumerate() {
                                let matched = self.execute_array_callback(
                                    callback,
                                    &[
                                        item,
                                        Value::Number(idx as i64),
                                        Value::Array(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                if !matched.truthy() {
                                    return Ok(Value::Bool(false));
                                }
                            }
                            Ok(Value::Bool(true))
                        }
                        Some(Value::TypedArray(values)) => {
                            let input = self.typed_array_snapshot(&values)?;
                            for (idx, item) in input.into_iter().enumerate() {
                                let matched = self.execute_array_callback(
                                    callback,
                                    &[
                                        item,
                                        Value::Number(idx as i64),
                                        Value::TypedArray(values.clone()),
                                    ],
                                    env,
                                    event,
                                )?;
                                if !matched.truthy() {
                                    return Ok(Value::Bool(false));
                                }
                            }
                            Ok(Value::Bool(true))
                        }
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArraySlice { target, start, end } => {
                    match self.resolve_target_value_with_pending(env, target) {
                        Some(Value::Array(values)) => {
                            let values = values.borrow();
                            let len = values.len();
                            let start = start
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(0);
                            let end = end
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(len);
                            let end = end.max(start);
                            Ok(Self::new_array_value(values[start..end].to_vec()))
                        }
                        Some(Value::TypedArray(values)) => {
                            let snapshot = self.typed_array_snapshot(&values)?;
                            let kind = values.borrow().kind;
                            let len = snapshot.len();
                            let start = start
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(0);
                            let end = end
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(len);
                            let end = end.max(start);
                            self.new_typed_array_from_values(kind, &snapshot[start..end])
                        }
                        Some(Value::ArrayBuffer(buffer)) => {
                            Self::ensure_array_buffer_not_detached(&buffer, "slice")?;
                            let source = buffer.borrow();
                            let len = source.bytes.len();
                            let start = start
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(0);
                            let end = end
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(len);
                            let end = end.max(start);
                            Ok(Value::ArrayBuffer(Rc::new(RefCell::new(
                                ArrayBufferValue {
                                    bytes: source.bytes[start..end].to_vec(),
                                    max_byte_length: None,
                                    detached: false,
                                },
                            ))))
                        }
                        Some(Value::Blob(blob)) => {
                            let source = blob.borrow();
                            let len = source.bytes.len();
                            let start = start
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(0);
                            let end = end
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(len);
                            let end = end.max(start);
                            Ok(Self::new_blob_value(
                                source.bytes[start..end].to_vec(),
                                String::new(),
                            ))
                        }
                        Some(Value::String(value)) => {
                            let len = value.chars().count();
                            let start = start
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(0);
                            let end = end
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(len);
                            let end = end.max(start);
                            Ok(Value::String(Self::substring_chars(&value, start, end)))
                        }
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArraySplice {
                    target,
                    start,
                    delete_count,
                    items,
                } => {
                    let values = self.resolve_array_from_env(env, target)?;
                    let start = self.eval_expr(start, env, event_param, event)?;
                    let start = Self::value_to_i64(&start);
                    let delete_count = delete_count
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value));
                    let insert_items =
                        self.eval_call_args_with_spread(items, env, event_param, event)?;

                    let mut values = values.borrow_mut();
                    let len = values.len();
                    let start = Self::normalize_splice_start_index(len, start);
                    let delete_count = delete_count
                        .unwrap_or((len.saturating_sub(start)) as i64)
                        .max(0) as usize;
                    let delete_count = delete_count.min(len.saturating_sub(start));
                    let removed = values
                        .drain(start..start + delete_count)
                        .collect::<Vec<_>>();
                    for (offset, item) in insert_items.into_iter().enumerate() {
                        values.insert(start + offset, item);
                    }
                    Ok(Self::new_array_value(removed))
                }
                Expr::ArrayJoin { target, separator } => {
                    let separator = separator
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| self.coerce_to_string_for_string_context(&value))
                        .unwrap_or_else(|| ",".to_string());
                    let values = match self.resolve_target_value_with_pending(env, target) {
                        Some(Value::Array(values)) => values.borrow().clone(),
                        Some(Value::TypedArray(values)) => self.typed_array_snapshot(&values)?,
                        Some(_) => {
                            return Err(Error::ScriptRuntime(format!(
                                "variable '{}' is not an array",
                                target
                            )));
                        }
                        None => {
                            return Err(Error::ScriptRuntime(format!(
                                "unknown variable: {}",
                                target
                            )));
                        }
                    };
                    let mut out = String::new();
                    for (idx, value) in values.iter().enumerate() {
                        if idx > 0 {
                            out.push_str(&separator);
                        }
                        if matches!(value, Value::Null | Value::Undefined) {
                            continue;
                        }
                        out.push_str(&self.coerce_to_string_for_string_context(value));
                    }
                    Ok(Value::String(out))
                }
                Expr::ArraySort { target, comparator } => {
                    let comparator = comparator
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?;
                    if let Some(Value::Object(entries)) =
                        self.resolve_target_value_with_pending(env, target)
                    {
                        if Self::is_url_search_params_object(&entries.borrow()) {
                            {
                                let mut object_ref = entries.borrow_mut();
                                let mut pairs =
                                    Self::url_search_params_pairs_from_object_entries(&object_ref);
                                pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                                Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                            }
                            self.sync_url_search_params_owner(&entries);
                            return Ok(Value::Object(entries));
                        }
                    }
                    if comparator
                        .as_ref()
                        .is_some_and(|value| !self.is_callable_value(value))
                    {
                        return Err(Error::ScriptRuntime("callback is not a function".into()));
                    }

                    let values = self.resolve_array_from_env(env, target)?;
                    let mut snapshot = values.borrow().clone();
                    let len = snapshot.len();
                    for i in 0..len {
                        let end = len.saturating_sub(i + 1);
                        for j in 0..end {
                            let should_swap = if let Some(comparator) = comparator.as_ref() {
                                let compared = self.execute_callable_value(
                                    comparator,
                                    &[snapshot[j].clone(), snapshot[j + 1].clone()],
                                    event,
                                )?;
                                Self::coerce_number_for_global(&compared) > 0.0
                            } else {
                                snapshot[j].as_string() > snapshot[j + 1].as_string()
                            };
                            if should_swap {
                                snapshot.swap(j, j + 1);
                            }
                        }
                    }
                    values.borrow_mut().elements = snapshot;
                    Ok(Value::Array(values))
                }
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }

    fn array_constructor_length_from_value(value: &Value) -> Result<Option<usize>> {
        match value {
            Value::Number(value) => {
                if *value < 0 {
                    return Err(Error::ScriptRuntime("invalid array length".into()));
                }
                Ok(Some(usize::try_from(*value).map_err(|_| {
                    Error::ScriptRuntime("invalid array length".into())
                })?))
            }
            Value::Float(value) => {
                if !value.is_finite() || *value < 0.0 || value.fract() != 0.0 {
                    return Err(Error::ScriptRuntime("invalid array length".into()));
                }
                if *value > usize::MAX as f64 {
                    return Err(Error::ScriptRuntime("invalid array length".into()));
                }
                Ok(Some(*value as usize))
            }
            _ => Ok(None),
        }
    }
}
