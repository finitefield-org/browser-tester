use super::*;
use unicode_normalization::UnicodeNormalization;

impl Harness {
    pub(crate) fn eval_date_member_call(
        &mut self,
        value: &Rc<RefCell<i64>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let result = match member {
            "getTime" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getTime does not take arguments".into(),
                    ));
                }
                Value::Number(*value.borrow())
            }
            "setTime" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setTime requires exactly one argument".into(),
                    ));
                }
                let timestamp_ms = Self::value_to_i64(&evaluated_args[0]);
                *value.borrow_mut() = timestamp_ms;
                Value::Number(timestamp_ms)
            }
            "toISOString" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "toISOString does not take arguments".into(),
                    ));
                }
                Value::String(Self::format_iso_8601_utc(*value.borrow()))
            }
            "getFullYear" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getFullYear does not take arguments".into(),
                    ));
                }
                let (year, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(year)
            }
            "getMonth" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getMonth does not take arguments".into(),
                    ));
                }
                let (_, month, ..) = Self::date_components_utc(*value.borrow());
                Value::Number((month as i64) - 1)
            }
            "getDate" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getDate does not take arguments".into(),
                    ));
                }
                let (_, _, day, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(day as i64)
            }
            "getHours" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getHours does not take arguments".into(),
                    ));
                }
                let (_, _, _, hour, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(hour as i64)
            }
            "getMinutes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getMinutes does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, minute, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(minute as i64)
            }
            "getSeconds" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getSeconds does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, _, second, _) = Self::date_components_utc(*value.borrow());
                Value::Number(second as i64)
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    pub(crate) fn eval_string_member_call(
        &mut self,
        text: &str,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let value = match member {
            "concat" => {
                let mut out = text.to_string();
                for arg in evaluated_args {
                    out.push_str(&arg.as_string());
                }
                Value::String(out)
            }
            "normalize" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "normalize supports at most one form argument".into(),
                    ));
                }
                let coerced_form;
                let form = match evaluated_args.first() {
                    None | Some(Value::Undefined) => "NFC",
                    Some(Value::String(value)) => value.as_str(),
                    Some(other) => {
                        coerced_form = other.as_string();
                        coerced_form.as_str()
                    }
                };
                let normalized = match form {
                    "NFC" => text.nfc().collect(),
                    "NFD" => text.nfd().collect(),
                    "NFKC" => text.nfkc().collect(),
                    "NFKD" => text.nfkd().collect(),
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "invalid normalization form: {form}"
                        )));
                    }
                };
                Value::String(normalized)
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    pub(crate) fn eval_array_member_call(
        &mut self,
        values: &Rc<RefCell<ArrayValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match member {
            "forEach" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "forEach requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                }
                Value::Undefined
            }
            "map" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "map requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut out = Vec::with_capacity(snapshot.len());
                for (idx, item) in snapshot.into_iter().enumerate() {
                    out.push(self.execute_callback_value(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?);
                }
                Self::new_array_value(out)
            }
            "filter" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "filter requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut out = Vec::new();
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let keep = self.execute_callback_value(
                        &callback,
                        &[
                            item.clone(),
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                    if keep.truthy() {
                        out.push(item);
                    }
                }
                Self::new_array_value(out)
            }
            "reduce" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "reduce requires callback and optional initial value".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut start_index = 0usize;
                let mut acc = if let Some(initial) = evaluated_args.get(1) {
                    initial.clone()
                } else {
                    let Some(first) = snapshot.first().cloned() else {
                        return Err(Error::ScriptRuntime(
                            "reduce of empty array with no initial value".into(),
                        ));
                    };
                    start_index = 1;
                    first
                };
                for (idx, item) in snapshot.into_iter().enumerate().skip(start_index) {
                    acc = self.execute_callback_value(
                        &callback,
                        &[
                            acc,
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                }
                acc
            }
            "find" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "find requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut found = Value::Undefined;
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let matched = self.execute_callback_value(
                        &callback,
                        &[
                            item.clone(),
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                    if matched.truthy() {
                        found = item;
                        break;
                    }
                }
                found
            }
            "findIndex" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "findIndex requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut found = -1i64;
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let matched = self.execute_callback_value(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                    if matched.truthy() {
                        found = idx as i64;
                        break;
                    }
                }
                Value::Number(found)
            }
            "some" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "some requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut matched = false;
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let keep = self.execute_callback_value(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                    if keep.truthy() {
                        matched = true;
                        break;
                    }
                }
                Value::Bool(matched)
            }
            "every" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "every requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut all = true;
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let keep = self.execute_callback_value(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                    if !keep.truthy() {
                        all = false;
                        break;
                    }
                }
                Value::Bool(all)
            }
            "values" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "values does not take arguments".into(),
                    ));
                }
                self.new_iterator_value(values.borrow().clone())
            }
            "keys" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("keys does not take arguments".into()));
                }
                self.new_iterator_value(
                    (0..values.borrow().len())
                        .map(|index| Value::Number(index as i64))
                        .collect(),
                )
            }
            "entries" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "entries does not take arguments".into(),
                    ));
                }
                self.new_iterator_value(
                    values
                        .borrow()
                        .iter()
                        .enumerate()
                        .map(|(index, value)| {
                            Self::new_array_value(vec![Value::Number(index as i64), value.clone()])
                        })
                        .collect(),
                )
            }
            "fill" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "fill requires 1 to 3 arguments".into(),
                    ));
                }
                let fill_value = evaluated_args[0].clone();
                let mut values_ref = values.borrow_mut();
                let len = values_ref.len();
                let start = evaluated_args
                    .get(1)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(0);
                let end = evaluated_args
                    .get(2)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(len)
                    .max(start);
                for value in values_ref.iter_mut().take(end).skip(start) {
                    *value = fill_value.clone();
                }
                Value::Array(values.clone())
            }
            "includes" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "includes requires one or two arguments".into(),
                    ));
                }
                let search = evaluated_args[0].clone();
                let values_ref = values.borrow();
                let len = values_ref.len() as i64;
                let mut start = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                if start < 0 {
                    start = (len + start).max(0);
                }
                let start = start.min(len) as usize;
                let mut found = false;
                for value in values_ref.iter().skip(start) {
                    if self.strict_equal(value, &search) {
                        found = true;
                        break;
                    }
                }
                Value::Bool(found)
            }
            "slice" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "slice supports up to two arguments".into(),
                    ));
                }
                let values_ref = values.borrow();
                let len = values_ref.len();
                let start = evaluated_args
                    .first()
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(0);
                let end = evaluated_args
                    .get(1)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(len);
                let end = end.max(start);
                Self::new_array_value(values_ref[start..end].to_vec())
            }
            "join" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "join supports zero or one separator argument".into(),
                    ));
                }
                let separator = evaluated_args
                    .first()
                    .map(Value::as_string)
                    .unwrap_or_else(|| ",".to_string());
                let values_ref = values.borrow();
                let mut out = String::new();
                for (idx, value) in values_ref.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(&separator);
                    }
                    if matches!(value, Value::Null | Value::Undefined) {
                        continue;
                    }
                    out.push_str(&value.as_string());
                }
                Value::String(out)
            }
            "concat" => {
                let mut out = values.borrow().clone();
                for arg in evaluated_args {
                    match arg {
                        Value::Array(other) => out.extend(other.borrow().iter().cloned()),
                        _ => out.push(arg.clone()),
                    }
                }
                Self::new_array_value(out)
            }
            "push" => {
                let mut values_ref = values.borrow_mut();
                values_ref.extend(evaluated_args.iter().cloned());
                Value::Number(values_ref.len() as i64)
            }
            "pop" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("pop does not take arguments".into()));
                }
                values.borrow_mut().pop().unwrap_or(Value::Undefined)
            }
            "shift" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("shift does not take arguments".into()));
                }
                let mut values_ref = values.borrow_mut();
                if values_ref.is_empty() {
                    Value::Undefined
                } else {
                    values_ref.remove(0)
                }
            }
            "unshift" => {
                let mut values_ref = values.borrow_mut();
                for value in evaluated_args.iter().cloned().rev() {
                    values_ref.insert(0, value);
                }
                Value::Number(values_ref.len() as i64)
            }
            "splice" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "splice requires at least a start index".into(),
                    ));
                }
                let start = Self::value_to_i64(&evaluated_args[0]);
                let delete_count = evaluated_args.get(1).map(Self::value_to_i64);
                let mut values_ref = values.borrow_mut();
                let len = values_ref.len();
                let start = Self::normalize_splice_start_index(len, start);
                let delete_count = delete_count
                    .unwrap_or((len.saturating_sub(start)) as i64)
                    .max(0) as usize;
                let delete_count = delete_count.min(len.saturating_sub(start));
                let removed = values_ref
                    .drain(start..start + delete_count)
                    .collect::<Vec<_>>();
                for (offset, item) in evaluated_args.iter().skip(2).cloned().enumerate() {
                    values_ref.insert(start + offset, item);
                }
                Self::new_array_value(removed)
            }
            "sort" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "sort supports zero or one comparator argument".into(),
                    ));
                }
                if evaluated_args
                    .first()
                    .is_some_and(|value| !self.is_callable_value(value))
                {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let comparator = evaluated_args.first().cloned();
                let mut snapshot = values.borrow().clone();
                let len = snapshot.len();
                for i in 0..len {
                    let end = len.saturating_sub(i + 1);
                    for j in 0..end {
                        let should_swap = if let Some(comparator) = comparator.as_ref() {
                            let compared = self.execute_callable_value(
                                comparator,
                                &[snapshot[j].clone(), snapshot[j + 1].clone()],
                                event,
                            )?;
                            Self::coerce_number_for_global(&compared) > 0.0
                        } else {
                            snapshot[j].as_string() > snapshot[j + 1].as_string()
                        };
                        if should_swap {
                            snapshot.swap(j, j + 1);
                        }
                    }
                }
                values.borrow_mut().elements = snapshot;
                Value::Array(values.clone())
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    pub(crate) fn eval_map_member_call_from_values(
        &mut self,
        map: &Rc<RefCell<MapValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match member {
            "set" => {
                if evaluated_args.len() != 2 {
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
                if evaluated_args.len() != 1 {
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
                if evaluated_args.len() != 1 {
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
                if evaluated_args.len() != 1 {
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
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Map.clear does not take arguments".into(),
                    ));
                }
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
            "entries" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Map.entries does not take arguments".into(),
                    ));
                }
                Self::new_array_value(self.map_entries_array(map))
            }
            "keys" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Map.keys does not take arguments".into(),
                    ));
                }
                Self::new_array_value(
                    map.borrow()
                        .entries
                        .iter()
                        .map(|(key, _)| key.clone())
                        .collect::<Vec<_>>(),
                )
            }
            "values" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Map.values does not take arguments".into(),
                    ));
                }
                Self::new_array_value(
                    map.borrow()
                        .entries
                        .iter()
                        .map(|(_, value)| value.clone())
                        .collect::<Vec<_>>(),
                )
            }
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

    pub(crate) fn eval_nodelist_member_call(
        &mut self,
        nodes: &[NodeId],
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "forEach" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "forEach requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = nodes.to_vec();
                for (idx, node) in snapshot.iter().copied().enumerate() {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[
                            Value::Node(node),
                            Value::Number(idx as i64),
                            Value::NodeList(snapshot.clone()),
                        ],
                        event,
                    )?;
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }
}
