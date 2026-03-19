use super::*;

impl Harness {
    pub(crate) fn reflect_set_on_set_receiver_object(
        &mut self,
        receiver: &Value,
        key: &str,
        value: Value,
    ) -> Result<bool> {
        match receiver {
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
            _ => Ok(false),
        }
    }
}
