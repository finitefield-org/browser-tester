use super::*;

#[derive(Debug, Clone, Default)]
pub(crate) struct InputValidity {
    pub(crate) value_missing: bool,
    pub(crate) type_mismatch: bool,
    pub(crate) pattern_mismatch: bool,
    pub(crate) too_long: bool,
    pub(crate) too_short: bool,
    pub(crate) range_underflow: bool,
    pub(crate) range_overflow: bool,
    pub(crate) step_mismatch: bool,
    pub(crate) bad_input: bool,
    pub(crate) custom_error: bool,
    pub(crate) valid: bool,
}

pub(crate) trait ObjectEntryLookup {
    fn get_entry(&self, key: &str) -> Option<Value>;
}

pub(crate) trait ObjectEntryMut {
    fn set_entry(&mut self, key: String, value: Value);
}

impl ObjectEntryLookup for [(String, Value)] {
    fn get_entry(&self, key: &str) -> Option<Value> {
        self.iter()
            .find_map(|(name, value)| (name == key).then(|| value.clone()))
    }
}

impl ObjectEntryLookup for Vec<(String, Value)> {
    fn get_entry(&self, key: &str) -> Option<Value> {
        self.as_slice().get_entry(key)
    }
}

impl ObjectEntryLookup for ObjectValue {
    fn get_entry(&self, key: &str) -> Option<Value> {
        ObjectValue::get_entry(self, key)
    }
}

impl ObjectEntryLookup for std::cell::Ref<'_, ObjectValue> {
    fn get_entry(&self, key: &str) -> Option<Value> {
        ObjectValue::get_entry(&*self, key)
    }
}

impl ObjectEntryLookup for std::cell::RefMut<'_, ObjectValue> {
    fn get_entry(&self, key: &str) -> Option<Value> {
        ObjectValue::get_entry(&*self, key)
    }
}

impl ObjectEntryMut for Vec<(String, Value)> {
    fn set_entry(&mut self, key: String, value: Value) {
        if let Some((_, existing)) = self.iter_mut().find(|(name, _)| name == &key) {
            *existing = value;
        } else {
            self.push((key, value));
        }
    }
}

impl ObjectEntryMut for ObjectValue {
    fn set_entry(&mut self, key: String, value: Value) {
        ObjectValue::set_entry(self, key, value);
    }
}

impl ObjectEntryMut for std::cell::RefMut<'_, ObjectValue> {
    fn set_entry(&mut self, key: String, value: Value) {
        ObjectValue::set_entry(&mut *self, key, value);
    }
}
