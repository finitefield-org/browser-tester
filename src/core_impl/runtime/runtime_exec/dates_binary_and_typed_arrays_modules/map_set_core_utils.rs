impl Harness {
    pub(crate) fn same_value_zero(&self, left: &Value, right: &Value) -> bool {
        if let (Some(left_num), Some(right_num)) = (
            Self::number_primitive_value(left),
            Self::number_primitive_value(right),
        ) {
            if left_num.is_nan() && right_num.is_nan() {
                return true;
            }
        }
        self.strict_equal(left, right)
    }

    pub(crate) fn map_entry_index(&self, map: &MapValue, key: &Value) -> Option<usize> {
        map.entries
            .iter()
            .position(|(existing_key, _)| self.same_value_zero(existing_key, key))
    }

    pub(crate) fn map_set_entry(&self, map: &mut MapValue, key: Value, value: Value) {
        if let Some(index) = self.map_entry_index(map, &key) {
            map.entries[index].1 = value;
        } else {
            map.entries.push((key, value));
        }
    }

    pub(crate) fn map_entries_array(&self, map: &Rc<RefCell<MapValue>>) -> Vec<Value> {
        map.borrow()
            .entries
            .iter()
            .map(|(key, value)| Self::new_array_value(vec![key.clone(), value.clone()]))
            .collect::<Vec<_>>()
    }

    pub(crate) fn is_map_method_name(name: &str) -> bool {
        matches!(
            name,
            "set"
                | "get"
                | "has"
                | "delete"
                | "clear"
                | "forEach"
                | "entries"
                | "keys"
                | "values"
                | "getOrInsert"
                | "getOrInsertComputed"
        )
    }

    pub(crate) fn set_value_index(&self, set: &SetValue, value: &Value) -> Option<usize> {
        set.values
            .iter()
            .position(|existing_value| self.same_value_zero(existing_value, value))
    }

    pub(crate) fn set_add_value(&self, set: &mut SetValue, value: Value) {
        if self.set_value_index(set, &value).is_none() {
            set.values.push(value);
        }
    }

    pub(crate) fn set_values_array(&self, set: &Rc<RefCell<SetValue>>) -> Vec<Value> {
        set.borrow().values.clone()
    }

    pub(crate) fn set_entries_array(&self, set: &Rc<RefCell<SetValue>>) -> Vec<Value> {
        set.borrow()
            .values
            .iter()
            .map(|value| Self::new_array_value(vec![value.clone(), value.clone()]))
            .collect::<Vec<_>>()
    }

    pub(crate) fn set_like_keys_snapshot(&self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Set(set) => Ok(set.borrow().values.clone()),
            Value::Map(map) => Ok(map
                .borrow()
                .entries
                .iter()
                .map(|(key, _)| key.clone())
                .collect::<Vec<_>>()),
            _ => Err(Error::ScriptRuntime(
                "Set composition argument must be set-like (Set or Map)".into(),
            )),
        }
    }

    pub(crate) fn set_like_has_value(&self, value: &Value, candidate: &Value) -> Result<bool> {
        match value {
            Value::Set(set) => Ok(self.set_value_index(&set.borrow(), candidate).is_some()),
            Value::Map(map) => Ok(self.map_entry_index(&map.borrow(), candidate).is_some()),
            _ => Err(Error::ScriptRuntime(
                "Set composition argument must be set-like (Set or Map)".into(),
            )),
        }
    }

}
