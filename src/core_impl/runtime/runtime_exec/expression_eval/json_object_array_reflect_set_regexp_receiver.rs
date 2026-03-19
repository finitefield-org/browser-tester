use super::*;

impl Harness {
    pub(crate) fn reflect_set_on_regexp_receiver_object(
        &mut self,
        receiver: &Value,
        key: &str,
        value: Value,
    ) -> Result<bool> {
        match receiver {
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
