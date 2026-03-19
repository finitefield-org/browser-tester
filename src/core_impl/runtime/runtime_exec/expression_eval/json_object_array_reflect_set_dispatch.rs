use super::*;

impl Harness {
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
}
