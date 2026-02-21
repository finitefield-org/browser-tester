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
        match env.get(target) {
            Some(Value::Date(value)) => Ok(value.clone()),
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

    pub(crate) fn resolve_array_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<ArrayValue>>> {
        match env.get(target) {
            Some(Value::Array(values)) => Ok(values.clone()),
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
        match env.get(target) {
            Some(Value::ArrayBuffer(buffer)) => Ok(buffer.clone()),
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
        match env.get(target) {
            Some(Value::TypedArray(array)) => Ok(array.clone()),
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
