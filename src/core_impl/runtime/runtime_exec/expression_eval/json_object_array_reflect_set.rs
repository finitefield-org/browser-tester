use super::*;

impl Harness {
    pub(crate) fn reflect_set_on_receiver_object(
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
}
