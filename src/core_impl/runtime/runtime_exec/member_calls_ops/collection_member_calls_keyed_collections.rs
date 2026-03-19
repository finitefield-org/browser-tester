use super::*;

impl Harness {
    pub(crate) fn eval_map_member_call_from_values(
        &mut self,
        map: &Rc<RefCell<MapValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match member {
            "set" => {
                if evaluated_args.len() < 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.set requires exactly two arguments".into(),
                    ));
                }
                self.map_set_entry(
                    &mut map.borrow_mut(),
                    evaluated_args[0].clone(),
                    evaluated_args[1].clone(),
                );
                Value::Map(map.clone())
            }
            "get" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Map.get requires exactly one argument".into(),
                    ));
                }
                let map_ref = map.borrow();
                if let Some(index) = self.map_entry_index(&map_ref, &evaluated_args[0]) {
                    map_ref.entries[index].1.clone()
                } else {
                    Value::Undefined
                }
            }
            "has" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Map.has requires exactly one argument".into(),
                    ));
                }
                let has = self
                    .map_entry_index(&map.borrow(), &evaluated_args[0])
                    .is_some();
                Value::Bool(has)
            }
            "delete" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Map.delete requires exactly one argument".into(),
                    ));
                }
                let mut map_ref = map.borrow_mut();
                if let Some(index) = self.map_entry_index(&map_ref, &evaluated_args[0]) {
                    map_ref.entries.remove(index);
                    Value::Bool(true)
                } else {
                    Value::Bool(false)
                }
            }
            "clear" => {
                map.borrow_mut().entries.clear();
                Value::Undefined
            }
            "forEach" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.forEach requires a callback and optional thisArg".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = map.borrow().entries.clone();
                for (key, value) in snapshot {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[value, key, Value::Map(map.clone())],
                        event,
                    )?;
                }
                Value::Undefined
            }
            "entries" => Self::new_array_value(self.map_entries_array(map)),
            "keys" => Self::new_array_value(
                map.borrow()
                    .entries
                    .iter()
                    .map(|(key, _)| key.clone())
                    .collect::<Vec<_>>(),
            ),
            "values" => Self::new_array_value(
                map.borrow()
                    .entries
                    .iter()
                    .map(|(_, value)| value.clone())
                    .collect::<Vec<_>>(),
            ),
            "getOrInsert" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.getOrInsert requires exactly two arguments".into(),
                    ));
                }
                let key = evaluated_args[0].clone();
                let default_value = evaluated_args[1].clone();
                let mut map_ref = map.borrow_mut();
                if let Some(index) = self.map_entry_index(&map_ref, &key) {
                    map_ref.entries[index].1.clone()
                } else {
                    map_ref.entries.push((key, default_value.clone()));
                    default_value
                }
            }
            "getOrInsertComputed" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.getOrInsertComputed requires exactly two arguments".into(),
                    ));
                }
                let key = evaluated_args[0].clone();
                {
                    let map_ref = map.borrow();
                    if let Some(index) = self.map_entry_index(&map_ref, &key) {
                        return Ok(Some(map_ref.entries[index].1.clone()));
                    }
                }
                let callback = evaluated_args[1].clone();
                let computed =
                    self.execute_callback_value(&callback, std::slice::from_ref(&key), event)?;
                map.borrow_mut().entries.push((key, computed.clone()));
                computed
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    pub(crate) fn eval_set_member_call_from_values(
        &mut self,
        set: &Rc<RefCell<SetValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match member {
            "add" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.add requires exactly one argument".into(),
                    ));
                }
                self.set_add_value(&mut set.borrow_mut(), evaluated_args[0].clone());
                Value::Set(set.clone())
            }
            "has" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.has requires exactly one argument".into(),
                    ));
                }
                Value::Bool(
                    self.set_value_index(&set.borrow(), &evaluated_args[0])
                        .is_some(),
                )
            }
            "delete" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.delete requires exactly one argument".into(),
                    ));
                }
                let mut set_ref = set.borrow_mut();
                if let Some(index) = self.set_value_index(&set_ref, &evaluated_args[0]) {
                    set_ref.values.remove(index);
                    Value::Bool(true)
                } else {
                    Value::Bool(false)
                }
            }
            "clear" => {
                set.borrow_mut().values.clear();
                Value::Undefined
            }
            "forEach" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Set.forEach requires a callback and optional thisArg".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = set.borrow().values.clone();
                for value in snapshot {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[value.clone(), value, Value::Set(set.clone())],
                        event,
                    )?;
                }
                Value::Undefined
            }
            "entries" => Self::new_array_value(self.set_entries_array(set)),
            "keys" | "values" => Self::new_array_value(self.set_values_array(set)),
            "union" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.union requires exactly one argument".into(),
                    ));
                }
                let other_keys = self.set_like_keys_snapshot(&evaluated_args[0])?;
                let mut out = SetValue {
                    values: set.borrow().values.clone(),
                    properties: ObjectValue::default(),
                };
                for key in other_keys {
                    self.set_add_value(&mut out, key);
                }
                Value::Set(Rc::new(RefCell::new(out)))
            }
            "intersection" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.intersection requires exactly one argument".into(),
                    ));
                }
                let snapshot = set.borrow().values.clone();
                let mut out = SetValue {
                    values: Vec::new(),
                    properties: ObjectValue::default(),
                };
                for value in snapshot {
                    if self.set_like_has_value(&evaluated_args[0], &value)? {
                        self.set_add_value(&mut out, value);
                    }
                }
                Value::Set(Rc::new(RefCell::new(out)))
            }
            "difference" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.difference requires exactly one argument".into(),
                    ));
                }
                let snapshot = set.borrow().values.clone();
                let mut out = SetValue {
                    values: Vec::new(),
                    properties: ObjectValue::default(),
                };
                for value in snapshot {
                    if !self.set_like_has_value(&evaluated_args[0], &value)? {
                        self.set_add_value(&mut out, value);
                    }
                }
                Value::Set(Rc::new(RefCell::new(out)))
            }
            "symmetricDifference" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.symmetricDifference requires exactly one argument".into(),
                    ));
                }
                let other_keys = self.set_like_keys_snapshot(&evaluated_args[0])?;
                let mut out = SetValue {
                    values: set.borrow().values.clone(),
                    properties: ObjectValue::default(),
                };
                for key in other_keys {
                    if let Some(index) = self.set_value_index(&out, &key) {
                        out.values.remove(index);
                    } else {
                        out.values.push(key);
                    }
                }
                Value::Set(Rc::new(RefCell::new(out)))
            }
            "isDisjointFrom" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.isDisjointFrom requires exactly one argument".into(),
                    ));
                }
                for value in &set.borrow().values {
                    if self.set_like_has_value(&evaluated_args[0], value)? {
                        return Ok(Some(Value::Bool(false)));
                    }
                }
                Value::Bool(true)
            }
            "isSubsetOf" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.isSubsetOf requires exactly one argument".into(),
                    ));
                }
                for value in &set.borrow().values {
                    if !self.set_like_has_value(&evaluated_args[0], value)? {
                        return Ok(Some(Value::Bool(false)));
                    }
                }
                Value::Bool(true)
            }
            "isSupersetOf" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.isSupersetOf requires exactly one argument".into(),
                    ));
                }
                for value in self.set_like_keys_snapshot(&evaluated_args[0])? {
                    if self.set_value_index(&set.borrow(), &value).is_none() {
                        return Ok(Some(Value::Bool(false)));
                    }
                }
                Value::Bool(true)
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    pub(crate) fn eval_weak_map_member_call_from_values(
        &mut self,
        weak_map: &Rc<RefCell<WeakMapValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match member {
            "set" => {
                if evaluated_args.len() < 2 {
                    return Err(Error::ScriptRuntime(
                        "WeakMap.set requires exactly two arguments".into(),
                    ));
                }
                Self::ensure_weak_map_key(&evaluated_args[0])?;
                self.weak_map_set_entry(
                    &mut weak_map.borrow_mut(),
                    evaluated_args[0].clone(),
                    evaluated_args[1].clone(),
                );
                Value::WeakMap(weak_map.clone())
            }
            "get" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "WeakMap.get requires exactly one argument".into(),
                    ));
                }
                if !Self::weak_map_accepts_key(&evaluated_args[0]) {
                    return Ok(Some(Value::Undefined));
                }
                let weak_map_ref = weak_map.borrow();
                if let Some(index) = self.weak_map_entry_index(&weak_map_ref, &evaluated_args[0]) {
                    weak_map_ref.entries[index].1.clone()
                } else {
                    Value::Undefined
                }
            }
            "has" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "WeakMap.has requires exactly one argument".into(),
                    ));
                }
                if !Self::weak_map_accepts_key(&evaluated_args[0]) {
                    return Ok(Some(Value::Bool(false)));
                }
                let has = self
                    .weak_map_entry_index(&weak_map.borrow(), &evaluated_args[0])
                    .is_some();
                Value::Bool(has)
            }
            "delete" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "WeakMap.delete requires exactly one argument".into(),
                    ));
                }
                if !Self::weak_map_accepts_key(&evaluated_args[0]) {
                    return Ok(Some(Value::Bool(false)));
                }
                let mut weak_map_ref = weak_map.borrow_mut();
                if let Some(index) = self.weak_map_entry_index(&weak_map_ref, &evaluated_args[0]) {
                    weak_map_ref.entries.remove(index);
                    Value::Bool(true)
                } else {
                    Value::Bool(false)
                }
            }
            "getOrInsert" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "WeakMap.getOrInsert requires exactly two arguments".into(),
                    ));
                }
                let key = evaluated_args[0].clone();
                Self::ensure_weak_map_key(&key)?;
                let default_value = evaluated_args[1].clone();
                let mut weak_map_ref = weak_map.borrow_mut();
                if let Some(index) = self.weak_map_entry_index(&weak_map_ref, &key) {
                    weak_map_ref.entries[index].1.clone()
                } else {
                    weak_map_ref.entries.push((key, default_value.clone()));
                    default_value
                }
            }
            "getOrInsertComputed" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "WeakMap.getOrInsertComputed requires exactly two arguments".into(),
                    ));
                }
                let key = evaluated_args[0].clone();
                Self::ensure_weak_map_key(&key)?;
                {
                    let weak_map_ref = weak_map.borrow();
                    if let Some(index) = self.weak_map_entry_index(&weak_map_ref, &key) {
                        return Ok(Some(weak_map_ref.entries[index].1.clone()));
                    }
                }
                let callback = evaluated_args[1].clone();
                let computed =
                    self.execute_callback_value(&callback, std::slice::from_ref(&key), event)?;
                weak_map.borrow_mut().entries.push((key, computed.clone()));
                computed
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    pub(crate) fn eval_weak_set_member_call_from_values(
        &mut self,
        weak_set: &Rc<RefCell<WeakSetValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let value = match member {
            "add" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "WeakSet.add requires exactly one argument".into(),
                    ));
                }
                Self::ensure_weak_set_value(&evaluated_args[0])?;
                self.weak_set_add_value(&mut weak_set.borrow_mut(), evaluated_args[0].clone());
                Value::WeakSet(weak_set.clone())
            }
            "has" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "WeakSet.has requires exactly one argument".into(),
                    ));
                }
                if !Self::weak_set_accepts_value(&evaluated_args[0]) {
                    return Ok(Some(Value::Bool(false)));
                }
                let has = self
                    .weak_set_value_index(&weak_set.borrow(), &evaluated_args[0])
                    .is_some();
                Value::Bool(has)
            }
            "delete" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "WeakSet.delete requires exactly one argument".into(),
                    ));
                }
                if !Self::weak_set_accepts_value(&evaluated_args[0]) {
                    return Ok(Some(Value::Bool(false)));
                }
                let mut weak_set_ref = weak_set.borrow_mut();
                if let Some(index) = self.weak_set_value_index(&weak_set_ref, &evaluated_args[0]) {
                    weak_set_ref.values.remove(index);
                    Value::Bool(true)
                } else {
                    Value::Bool(false)
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
