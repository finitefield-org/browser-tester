use super::*;

impl Harness {
    pub(crate) fn new_array_value(values: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(ArrayValue::new(values))))
    }

    pub(crate) fn set_array_property(array: &Rc<RefCell<ArrayValue>>, key: String, value: Value) {
        Self::object_set_entry(&mut array.borrow_mut().properties, key, value);
    }

    pub(crate) fn array_hole_storage_key(index: usize) -> String {
        format!("{INTERNAL_ARRAY_HOLE_KEY_PREFIX}{index}")
    }

    pub(crate) fn array_index_is_hole(array: &ArrayValue, index: usize) -> bool {
        let hole_key = Self::array_hole_storage_key(index);
        Self::object_get_entry(&array.properties, &hole_key).is_some()
    }

    pub(crate) fn clear_array_hole(array: &Rc<RefCell<ArrayValue>>, index: usize) {
        let hole_key = Self::array_hole_storage_key(index);
        array.borrow_mut().properties.delete_entry(&hole_key);
    }

    pub(crate) fn mark_array_hole(array: &Rc<RefCell<ArrayValue>>, index: usize) {
        let hole_key = Self::array_hole_storage_key(index);
        Self::object_set_entry(
            &mut array.borrow_mut().properties,
            hole_key,
            Value::Bool(true),
        );
    }

    pub(crate) fn delete_object_getter_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let getter_key = Self::object_getter_storage_key(key);
        let mut deleted = entries.delete_entry(&getter_key);
        let undefined_getter_key = Self::object_undefined_getter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_getter_key);
        deleted
    }

    pub(crate) fn delete_object_setter_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let setter_key = Self::object_setter_storage_key(key);
        let mut deleted = entries.delete_entry(&setter_key);
        let undefined_setter_key = Self::object_undefined_setter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_setter_key);
        deleted
    }

    pub(crate) fn delete_object_property_auxiliary_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let mut deleted = Self::delete_object_getter_entries(entries, key);
        deleted |= Self::delete_object_setter_entries(entries, key);
        let non_enumerable_key = Self::object_non_enumerable_storage_key(key);
        deleted |= entries.delete_entry(&non_enumerable_key);
        let non_writable_key = Self::object_non_writable_storage_key(key);
        deleted |= entries.delete_entry(&non_writable_key);
        let non_configurable_key = Self::object_non_configurable_storage_key(key);
        deleted |= entries.delete_entry(&non_configurable_key);
        deleted
    }

    pub(crate) fn delete_object_property_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let mut deleted = entries.delete_entry(key);
        let getter_key = Self::object_getter_storage_key(key);
        deleted |= entries.delete_entry(&getter_key);
        let setter_key = Self::object_setter_storage_key(key);
        deleted |= entries.delete_entry(&setter_key);
        let undefined_getter_key = Self::object_undefined_getter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_getter_key);
        let undefined_setter_key = Self::object_undefined_setter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_setter_key);
        let non_enumerable_key = Self::object_non_enumerable_storage_key(key);
        deleted |= entries.delete_entry(&non_enumerable_key);
        let non_writable_key = Self::object_non_writable_storage_key(key);
        deleted |= entries.delete_entry(&non_writable_key);
        let non_configurable_key = Self::object_non_configurable_storage_key(key);
        deleted |= entries.delete_entry(&non_configurable_key);
        deleted
    }

    pub(crate) fn new_object_value(entries: Vec<(String, Value)>) -> Value {
        Value::Object(Rc::new(RefCell::new(ObjectValue::new(entries))))
    }
}
