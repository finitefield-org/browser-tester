use super::*;

impl Harness {
    fn coerce_object_like_to_string_via_primitive_methods(
        &mut self,
        value: &Value,
        allow_symbol: bool,
    ) -> Result<String> {
        let mut saw_callable = false;
        for method_name in ["toString", "valueOf"] {
            let method = self.object_property_from_value(value, method_name)?;
            if !self.is_callable_value(&method) {
                continue;
            }
            saw_callable = true;
            let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
            let coerced = self.execute_callable_value_with_this_and_env(
                &method,
                &[],
                &event,
                None,
                Some(value.clone()),
            )?;
            if Self::is_primitive_value(&coerced) {
                if matches!(coerced, Value::Symbol(_)) && !allow_symbol {
                    return Err(Error::ScriptRuntime(
                        "Cannot convert a Symbol value to a string".into(),
                    ));
                }
                return Ok(self.coerce_to_string_for_string_context(&coerced));
            }
        }
        if saw_callable {
            return Err(Error::ScriptRuntime(
                "Cannot convert object to primitive value".into(),
            ));
        }
        Ok(self.coerce_to_string_for_string_context(value))
    }

    pub(crate) fn coerce_to_string_for_tostring(&mut self, value: &Value) -> Result<String> {
        match value {
            Value::Symbol(_) => Err(Error::ScriptRuntime(
                "Cannot convert a Symbol value to a string".into(),
            )),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(wrapped) = Self::string_wrapper_value_from_object(&entries) {
                    return Ok(wrapped);
                }
                if Self::symbol_wrapper_id_from_object(&entries).is_some() {
                    return Err(Error::ScriptRuntime(
                        "Cannot convert a Symbol value to a string".into(),
                    ));
                }
                drop(entries);
                self.coerce_object_like_to_string_via_primitive_methods(value, false)
            }
            Value::Date(_) => self.coerce_object_like_to_string_via_primitive_methods(value, false),
            _ => Ok(self.coerce_to_string_for_string_context(value)),
        }
    }

    pub(crate) fn coerce_to_string_for_string_constructor(
        &mut self,
        value: &Value,
    ) -> Result<String> {
        match value {
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(wrapped) = Self::string_wrapper_value_from_object(&entries) {
                    return Ok(wrapped);
                }
                drop(entries);
                self.coerce_object_like_to_string_via_primitive_methods(value, true)
            }
            Value::Date(_) => self.coerce_object_like_to_string_via_primitive_methods(value, true),
            _ => Ok(self.coerce_to_string_for_string_context(value)),
        }
    }

    pub(crate) fn coerce_to_string_for_string_context(&mut self, value: &Value) -> String {
        self.callable_source_text(value)
            .unwrap_or_else(|| value.as_string())
    }
}
