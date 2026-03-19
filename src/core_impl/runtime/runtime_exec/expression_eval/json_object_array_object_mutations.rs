use super::*;

impl Harness {
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
}
