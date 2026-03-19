use super::*;

impl Harness {
    pub(crate) fn reflect_set_on_collection_receiver_object(
        &mut self,
        receiver: &Value,
        key: &str,
        value: Value,
    ) -> Result<bool> {
        match receiver {
            Value::Map(_) | Value::WeakMap(_) => {
                self.reflect_set_on_map_receiver_object(receiver, key, value)
            }
            Value::Set(_) | Value::WeakSet(_) => {
                self.reflect_set_on_set_receiver_object(receiver, key, value)
            }
            _ => Ok(false),
        }
    }
}
