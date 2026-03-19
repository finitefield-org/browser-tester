use super::*;

impl Harness {
    pub(crate) fn reflect_set_on_special_receiver_object(
        &mut self,
        receiver: &Value,
        key: &str,
        value: Value,
    ) -> Result<bool> {
        match receiver {
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
            other => self.reflect_set_on_collection_or_regexp_receiver_object(other, key, value),
        }
    }
}
