use super::*;

impl Harness {
    pub(crate) fn new_date_value(timestamp_ms: i64) -> Value {
        Value::Date(Rc::new(RefCell::new(timestamp_ms)))
    }

    pub(crate) fn resolve_date_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<i64>>> {
        match self.resolve_listener_capture_pending_value(target)
            .flatten()
            .or_else(|| env.get(target).cloned())
        {
            Some(Value::Date(value)) => Ok(value),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a Date",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(crate) fn coerce_date_timestamp_ms(&self, value: &Value) -> i64 {
        match value {
            Value::Date(value) => *value.borrow(),
            Value::String(value) => Self::parse_date_string_to_epoch_ms(value).unwrap_or(0),
            _ => Self::value_to_i64(value),
        }
    }

    pub(crate) fn coerce_intl_date_time_timestamp_ms(&self, value: &Value) -> Result<i64> {
        let numeric = Self::coerce_number_for_global(value);
        if !numeric.is_finite() {
            return Err(Error::ScriptRuntime(
                "RangeError: Invalid time value".into(),
            ));
        }
        Ok(numeric.trunc() as i64)
    }

    pub(crate) fn resolve_array_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<ArrayValue>>> {
        match self.resolve_listener_capture_pending_value(target)
            .flatten()
            .or_else(|| env.get(target).cloned())
        {
            Some(Value::Array(values)) => Ok(values),
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

    pub(crate) fn resolve_array_buffer_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<ArrayBufferValue>>> {
        match self.resolve_listener_capture_pending_value(target)
            .flatten()
            .or_else(|| env.get(target).cloned())
        {
            Some(Value::ArrayBuffer(buffer)) => Ok(buffer),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not an ArrayBuffer",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(crate) fn resolve_typed_array_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<TypedArrayValue>>> {
        match self.resolve_listener_capture_pending_value(target)
            .flatten()
            .or_else(|| env.get(target).cloned())
        {
            Some(Value::TypedArray(array)) => Ok(array),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a TypedArray",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }
}
