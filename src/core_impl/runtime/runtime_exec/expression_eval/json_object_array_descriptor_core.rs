use super::*;

impl Harness {
    pub(crate) fn callable_object_surface_descriptor_value(
        &mut self,
        object: &Value,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if Self::callable_kind_from_value(object).is_none() {
            return None;
        }
        if Self::object_get_entry(entries, key).is_some()
            || Self::has_object_accessor_property(entries, key)
            || Self::is_builtin_object_property_deleted(entries, key)
        {
            return None;
        }
        let value = self.callable_own_surface_value(object, key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            value, false, false, true,
        ))
    }

    pub(crate) fn placeholder_backed_object_builtin_descriptor_value(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        let stored = Self::object_get_entry(entries, key)?;
        if !Self::is_builtin_placeholder_value(&stored) {
            return None;
        }
        let value = Self::placeholder_backed_object_builtin_property_value(entries, key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            value,
            Self::is_writable_object_key(entries, key),
            Self::is_enumerable_object_key(entries, key),
            Self::is_configurable_object_key(entries, key),
        ))
    }

    pub(crate) fn placeholder_backed_array_builtin_descriptor_value(
        &mut self,
        array: &ArrayValue,
        key: &str,
    ) -> Option<Value> {
        let stored = Self::object_get_entry(&array.properties, key)?;
        if !Self::is_builtin_placeholder_value(&stored) {
            return None;
        }
        let value = Self::placeholder_backed_array_builtin_property_value(array, key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            value,
            Self::is_writable_object_key(&array.properties, key),
            Self::is_enumerable_object_key(&array.properties, key),
            Self::is_configurable_object_key(&array.properties, key),
        ))
    }

    pub(crate) fn function_own_property_descriptor_value(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
    ) -> Option<Value> {
        if let Some(entries) = self
            .script_runtime
            .function_public_properties
            .get(&function.function_id)
        {
            if let Some(descriptor) =
                Self::own_property_descriptor_object_from_entries(entries, key)
            {
                return Some(descriptor);
            }
            if Self::is_builtin_object_property_deleted(entries, key) {
                return None;
            }
        }

        match key {
            "name" | "length" => {
                return self
                    .function_builtin_own_property_value(function, key)
                    .map(|value| {
                        Self::own_data_property_descriptor_with_attrs(value, false, false, true)
                    });
            }
            "prototype" if !function.is_arrow && !function.is_method => {
                return self
                    .function_builtin_own_property_value(function, key)
                    .map(|value| {
                        Self::own_data_property_descriptor_with_attrs(value, true, false, false)
                    });
            }
            _ => {}
        }

        None
    }

    pub(crate) fn array_own_property_descriptor_value(
        &mut self,
        array: &Rc<RefCell<ArrayValue>>,
        key: &str,
    ) -> Option<Value> {
        let array_ref = array.borrow();
        if let Some(descriptor) =
            self.placeholder_backed_array_builtin_descriptor_value(&array_ref, key)
        {
            return Some(descriptor);
        }
        if let Some(descriptor) =
            Self::own_property_descriptor_object_from_entries(&array_ref.properties, key)
        {
            return Some(descriptor);
        }
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(array_ref.len() as i64),
                Self::is_writable_object_key(&array_ref.properties, "length"),
                false,
                false,
            ));
        }
        if let Ok(index) = key.parse::<usize>() {
            if index < array_ref.len() && !Self::array_index_is_hole(&array_ref, index) {
                return Some(Self::own_data_property_descriptor_with_attrs(
                    array_ref[index].clone(),
                    Self::array_index_is_writable(&array_ref, index),
                    Self::array_index_is_enumerable(&array_ref, index),
                    Self::array_index_is_configurable(&array_ref, index),
                ));
            }
            return None;
        }
        None
    }

    pub(crate) fn collection_own_property_descriptor_value(
        &mut self,
        entries: &ObjectValue,
        builtin_descriptor: Option<Value>,
        key: &str,
    ) -> Option<Value> {
        if let Some(descriptor) = Self::own_property_descriptor_object_from_entries(entries, key) {
            return Some(descriptor);
        }
        if Self::is_builtin_object_property_deleted(entries, key) {
            return None;
        }
        builtin_descriptor
    }
}
