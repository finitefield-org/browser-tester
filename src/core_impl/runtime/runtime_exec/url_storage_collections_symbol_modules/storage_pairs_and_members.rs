use super::*;

impl Harness {
    pub(crate) fn is_storage_method_name(name: &str) -> bool {
        matches!(name, "getItem" | "setItem" | "removeItem" | "clear" | "key")
    }

    pub(crate) fn storage_pairs_to_value(pairs: &[(String, String)]) -> Value {
        Self::new_array_value(
            pairs
                .iter()
                .map(|(name, value)| {
                    Self::new_array_value(vec![
                        Value::String(name.clone()),
                        Value::String(value.clone()),
                    ])
                })
                .collect::<Vec<_>>(),
        )
    }

    pub(crate) fn storage_pairs_from_object_entries(
        entries: &[(String, Value)],
    ) -> Vec<(String, String)> {
        let Some(Value::Array(list)) =
            Self::object_get_entry(entries, INTERNAL_STORAGE_ENTRIES_KEY)
        else {
            return Vec::new();
        };
        let snapshot = list.borrow().clone();
        let mut pairs = Vec::new();
        for item in snapshot {
            let Value::Array(pair) = item else {
                continue;
            };
            let pair = pair.borrow();
            if pair.is_empty() {
                continue;
            }
            let name = pair[0].as_string();
            let value = pair.get(1).map(Value::as_string).unwrap_or_default();
            pairs.push((name, value));
        }
        pairs
    }

    pub(crate) fn set_storage_pairs(entries: &mut impl ObjectEntryMut, pairs: &[(String, String)]) {
        Self::object_set_entry(
            entries,
            INTERNAL_STORAGE_ENTRIES_KEY.to_string(),
            Self::storage_pairs_to_value(pairs),
        );
    }

    pub(crate) fn eval_storage_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "getItem" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Storage.getItem requires exactly one argument".into(),
                    ));
                }
                let key = args[0].as_string();
                let value = Self::storage_pairs_from_object_entries(&object.borrow())
                    .into_iter()
                    .find_map(|(name, value)| (name == key).then_some(value))
                    .map(Value::String)
                    .unwrap_or(Value::Null);
                Ok(Some(value))
            }
            "setItem" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Storage.setItem requires exactly two arguments".into(),
                    ));
                }
                let key = args[0].as_string();
                let value = args[1].as_string();
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::storage_pairs_from_object_entries(&entries);
                    if let Some((_, stored)) = pairs.iter_mut().find(|(name, _)| name == &key) {
                        *stored = value;
                    } else {
                        pairs.push((key, value));
                    }
                    Self::set_storage_pairs(&mut entries, &pairs);
                }
                Ok(Some(Value::Undefined))
            }
            "removeItem" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Storage.removeItem requires exactly one argument".into(),
                    ));
                }
                let key = args[0].as_string();
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::storage_pairs_from_object_entries(&entries);
                    pairs.retain(|(name, _)| name != &key);
                    Self::set_storage_pairs(&mut entries, &pairs);
                }
                Ok(Some(Value::Undefined))
            }
            "clear" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Storage.clear does not take arguments".into(),
                    ));
                }
                Self::set_storage_pairs(&mut object.borrow_mut(), &[]);
                Ok(Some(Value::Undefined))
            }
            "key" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Storage.key requires exactly one argument".into(),
                    ));
                }
                let Some(index) = self.value_as_index(&args[0]) else {
                    return Ok(Some(Value::Null));
                };
                let value = Self::storage_pairs_from_object_entries(&object.borrow())
                    .get(index)
                    .map(|(name, _)| Value::String(name.clone()))
                    .unwrap_or(Value::Null);
                Ok(Some(value))
            }
            _ => Ok(None),
        }
    }
}
