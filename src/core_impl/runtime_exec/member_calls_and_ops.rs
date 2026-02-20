use super::*;

impl Harness {
    pub(crate) fn eval_array_member_call(
        &mut self,
        values: &Rc<RefCell<Vec<Value>>>,
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
                *values.borrow_mut() = snapshot;
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

    pub(crate) fn eval_node_member_call(
        &mut self,
        node: NodeId,
        member: &str,
        evaluated_args: &[Value],
        _event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "getAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                Ok(Some(Value::String(
                    self.dom.attr(node, &name).unwrap_or_default(),
                )))
            }
            "setAttribute" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "setAttribute requires exactly two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let value = evaluated_args[1].as_string();
                self.dom.set_attr(node, &name, &value)?;
                Ok(Some(Value::Undefined))
            }
            "hasAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "hasAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                Ok(Some(Value::Bool(self.dom.has_attr(node, &name)?)))
            }
            "hasAttributes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "hasAttributes takes no arguments".into(),
                    ));
                }
                let has_attributes = self
                    .dom
                    .element(node)
                    .map(|element| !element.attrs.is_empty())
                    .ok_or_else(|| {
                        Error::ScriptRuntime("hasAttributes target is not an element".into())
                    })?;
                Ok(Some(Value::Bool(has_attributes)))
            }
            "removeAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                self.dom.remove_attr(node, &name)?;
                Ok(Some(Value::Undefined))
            }
            "getAttributeNames" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNames takes no arguments".into(),
                    ));
                }
                let element = self.dom.element(node).ok_or_else(|| {
                    Error::ScriptRuntime("getAttributeNames target is not an element".into())
                })?;
                let mut names = element.attrs.keys().cloned().collect::<Vec<_>>();
                names.sort();
                Ok(Some(Self::new_array_value(
                    names.into_iter().map(Value::String).collect(),
                )))
            }
            "toggleAttribute" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "toggleAttribute requires one or two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let has = self.dom.has_attr(node, &name)?;
                let next = if evaluated_args.len() == 2 {
                    evaluated_args[1].truthy()
                } else {
                    !has
                };
                if next {
                    self.dom.set_attr(node, &name, "")?;
                } else {
                    self.dom.remove_attr(node, &name)?;
                }
                Ok(Some(Value::Bool(next)))
            }
            "matches" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "matches requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(Value::Bool(self.dom.matches_selector(node, &selector)?)))
            }
            "closest" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "closest requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom
                        .closest(node, &selector)?
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "querySelector" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelector requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom
                        .query_selector_from(&node, &selector)?
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "querySelectorAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelectorAll requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(Value::NodeList(
                    self.dom.query_selector_all_from(&node, &selector)?,
                )))
            }
            "getElementsByClassName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByClassName requires exactly one argument".into(),
                    ));
                }
                let classes = evaluated_args[0]
                    .as_string()
                    .split_whitespace()
                    .filter(|name| !name.is_empty())
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();
                if classes.is_empty() {
                    return Ok(Some(Value::NodeList(Vec::new())));
                }
                let selector = classes
                    .iter()
                    .map(|class_name| format!(".{class_name}"))
                    .collect::<String>();
                Ok(Some(Value::NodeList(
                    self.dom.query_selector_all_from(&node, &selector)?,
                )))
            }
            "getElementsByTagName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagName requires exactly one argument".into(),
                    ));
                }
                let tag_name = evaluated_args[0].as_string();
                if tag_name == "*" {
                    let mut nodes = Vec::new();
                    self.dom.collect_elements_descendants_dfs(node, &mut nodes);
                    return Ok(Some(Value::NodeList(nodes)));
                }
                Ok(Some(Value::NodeList(
                    self.dom.query_selector_all_from(&node, &tag_name)?,
                )))
            }
            "checkVisibility" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "checkVisibility supports at most one argument".into(),
                    ));
                }
                Ok(Some(Value::Bool(!self.dom.has_attr(node, "hidden")?)))
            }
            "checkValidity" | "reportValidity" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} takes no arguments"
                    )));
                }
                let validity = self.compute_input_validity(node)?;
                if !validity.valid {
                    let _ = self.dispatch_event(node, "invalid")?;
                }
                Ok(Some(Value::Bool(validity.valid)))
            }
            "setCustomValidity" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setCustomValidity requires exactly one argument".into(),
                    ));
                }
                self.dom
                    .set_custom_validity_message(node, &evaluated_args[0].as_string())?;
                Ok(Some(Value::Undefined))
            }
            "setSelectionRange" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "setSelectionRange requires two or three arguments".into(),
                    ));
                }
                self.set_node_selection_range(
                    node,
                    Self::value_to_i64(&evaluated_args[0]),
                    Self::value_to_i64(&evaluated_args[1]),
                    evaluated_args
                        .get(2)
                        .map(Value::as_string)
                        .unwrap_or_else(|| "none".to_string()),
                )?;
                Ok(Some(Value::Undefined))
            }
            "setRangeText" => {
                if !(evaluated_args.len() == 1
                    || evaluated_args.len() == 3
                    || evaluated_args.len() == 4)
                {
                    return Err(Error::ScriptRuntime(
                        "setRangeText supports one, three, or four arguments".into(),
                    ));
                }
                self.set_node_range_text(node, evaluated_args)?;
                Ok(Some(Value::Undefined))
            }
            "showPicker" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("showPicker takes no arguments".into()));
                }
                Ok(Some(Value::Undefined))
            }
            "stepUp" | "stepDown" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} supports at most one argument"
                    )));
                }
                let count = evaluated_args
                    .first()
                    .map(Self::value_to_i64)
                    .unwrap_or(1);
                let direction = if member == "stepDown" { -1 } else { 1 };
                self.step_input_value(node, direction, count)?;
                Ok(Some(Value::Undefined))
            }
            "scrollIntoView" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "scrollIntoView takes no arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "scroll" | "scrollTo" | "scrollBy" => {
                if !(evaluated_args.is_empty()
                    || evaluated_args.len() == 1
                    || evaluated_args.len() == 2)
                {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} supports zero, one, or two arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            "select" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("select takes no arguments".into()));
                }
                if self.node_supports_text_selection(node) {
                    let len = self.dom.value(node)?.chars().count();
                    self.dom.set_selection_range(node, 0, len, "none")?;
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }

    fn normalized_input_type(&self, node: NodeId) -> String {
        if !self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false)
        {
            return String::new();
        }
        let raw = self.dom.attr(node, "type").unwrap_or_default().to_ascii_lowercase();
        match raw.as_str() {
            "button" | "checkbox" | "color" | "date" | "datetime-local" | "email" | "file"
            | "hidden" | "image" | "month" | "number" | "password" | "radio" | "range"
            | "reset" | "search" | "submit" | "tel" | "text" | "time" | "url" | "week" => raw,
            _ => "text".to_string(),
        }
    }

    fn node_supports_text_selection(&self, node: NodeId) -> bool {
        if self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("textarea"))
            .unwrap_or(false)
        {
            return true;
        }
        if !self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false)
        {
            return false;
        }
        matches!(
            self.normalized_input_type(node).as_str(),
            "text" | "search" | "url" | "tel" | "email" | "password"
        )
    }

    pub(crate) fn normalize_selection_direction(direction: &str) -> &'static str {
        match direction {
            "forward" => "forward",
            "backward" => "backward",
            _ => "none",
        }
    }

    fn set_node_selection_range(
        &mut self,
        node: NodeId,
        start: i64,
        end: i64,
        direction: String,
    ) -> Result<()> {
        if !self.node_supports_text_selection(node) {
            return Ok(());
        }
        let start = start.max(0) as usize;
        let end = end.max(0) as usize;
        self.dom.set_selection_range(
            node,
            start,
            end,
            Self::normalize_selection_direction(direction.as_str()),
        )
    }

    fn shift_selection_index(index: usize, delta: i64) -> usize {
        if delta >= 0 {
            index.saturating_add(delta as usize)
        } else {
            index.saturating_sub(delta.unsigned_abs() as usize)
        }
    }

    fn set_node_range_text(&mut self, node: NodeId, args: &[Value]) -> Result<()> {
        if !self.node_supports_text_selection(node) {
            return Ok(());
        }

        let replacement = args[0].as_string();
        let old_value = self.dom.value(node)?;
        let old_len = old_value.chars().count();
        let old_sel_start = self.dom.selection_start(node)?;
        let old_sel_end = self.dom.selection_end(node)?;

        let (mut start, mut end, mode) = match args.len() {
            1 => (old_sel_start, old_sel_end, "preserve".to_string()),
            3 => (
                Self::value_to_i64(&args[1]).max(0) as usize,
                Self::value_to_i64(&args[2]).max(0) as usize,
                "preserve".to_string(),
            ),
            4 => (
                Self::value_to_i64(&args[1]).max(0) as usize,
                Self::value_to_i64(&args[2]).max(0) as usize,
                args[3].as_string(),
            ),
            _ => {
                return Err(Error::ScriptRuntime(
                    "setRangeText supports one, three, or four arguments".into(),
                ))
            }
        };
        start = start.min(old_len);
        end = end.min(old_len);
        if end < start {
            end = start;
        }

        let start_byte = Self::char_index_to_byte(&old_value, start);
        let end_byte = Self::char_index_to_byte(&old_value, end);
        let mut next_value = String::new();
        next_value.push_str(&old_value[..start_byte]);
        next_value.push_str(&replacement);
        next_value.push_str(&old_value[end_byte..]);
        self.dom.set_value(node, &next_value)?;

        let replacement_len = replacement.chars().count();
        let replaced_len = end.saturating_sub(start);
        let delta = replacement_len as i64 - replaced_len as i64;
        let mode = mode.to_ascii_lowercase();
        let (selection_start, selection_end) = match mode.as_str() {
            "select" => (start, start + replacement_len),
            "start" => (start, start),
            "end" => {
                let caret = start + replacement_len;
                (caret, caret)
            }
            _ => {
                if old_sel_end <= start {
                    (old_sel_start, old_sel_end)
                } else if old_sel_start >= end {
                    (
                        Self::shift_selection_index(old_sel_start, delta),
                        Self::shift_selection_index(old_sel_end, delta),
                    )
                } else {
                    let caret = start + replacement_len;
                    (caret, caret)
                }
            }
        };
        self.dom
            .set_selection_range(node, selection_start, selection_end, "none")
    }

    fn parse_attr_f64(&self, node: NodeId, name: &str) -> Option<f64> {
        self.dom.attr(node, name).and_then(|raw| {
            let raw = raw.trim();
            if raw.is_empty() {
                None
            } else {
                raw.parse::<f64>().ok().filter(|value| value.is_finite())
            }
        })
    }

    fn parse_attr_i64(&self, node: NodeId, name: &str) -> Option<i64> {
        self.dom.attr(node, name).and_then(|raw| {
            let raw = raw.trim();
            if raw.is_empty() {
                None
            } else {
                raw.parse::<i64>().ok()
            }
        })
    }

    fn parse_number_value(raw: &str) -> Option<f64> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        raw.parse::<f64>().ok().filter(|value| value.is_finite())
    }

    fn format_number_for_input(value: f64) -> String {
        if value.fract().abs() < 1e-9 {
            format!("{:.0}", value)
        } else {
            let mut out = value.to_string();
            if out.contains('.') {
                while out.ends_with('0') {
                    out.pop();
                }
                if out.ends_with('.') {
                    out.pop();
                }
            }
            out
        }
    }

    fn step_input_value(&mut self, node: NodeId, direction: i64, count: i64) -> Result<()> {
        if count == 0 {
            return Ok(());
        }
        let input_type = self.normalized_input_type(node);
        if !matches!(input_type.as_str(), "number" | "range") {
            return Ok(());
        }

        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
        let step = if step_attr.eq_ignore_ascii_case("any") {
            1.0
        } else {
            step_attr
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
                .unwrap_or(1.0)
        };
        let base = self
            .parse_attr_f64(node, "min")
            .or_else(|| self.parse_attr_f64(node, "value"))
            .unwrap_or(0.0);
        let current = Self::parse_number_value(&self.dom.value(node)?).unwrap_or(base);
        let mut next = current + (direction as f64) * (count as f64) * step;
        if let Some(min) = self.parse_attr_f64(node, "min") {
            if next < min {
                next = min;
            }
        }
        if let Some(max) = self.parse_attr_f64(node, "max") {
            if next > max {
                next = max;
            }
        }
        self.dom.set_value(node, &Self::format_number_for_input(next))
    }

    fn is_radio_group_checked(&self, node: NodeId) -> bool {
        let name = self.dom.attr(node, "name").unwrap_or_default();
        if name.is_empty() {
            return self.dom.checked(node).unwrap_or(false);
        }
        let form = self.dom.find_ancestor_by_tag(node, "form");
        self.dom.all_element_nodes().into_iter().any(|candidate| {
            is_radio_input(&self.dom, candidate)
                && self.dom.attr(candidate, "name").unwrap_or_default() == name
                && self.dom.find_ancestor_by_tag(candidate, "form") == form
                && self.dom.checked(candidate).unwrap_or(false)
        })
    }

    fn is_simple_email(value: &str) -> bool {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return false;
        }
        let Some((local, domain)) = trimmed.split_once('@') else {
            return false;
        };
        !local.is_empty() && !domain.is_empty() && !domain.contains('@')
    }

    fn is_url_like(value: &str) -> bool {
        LocationParts::parse(value).is_some()
    }

    fn input_participates_in_constraint_validation(kind: &str) -> bool {
        !matches!(kind, "button" | "submit" | "reset" | "hidden")
    }

    pub(crate) fn compute_input_validity(&self, node: NodeId) -> Result<InputValidity> {
        let mut validity = InputValidity {
            valid: true,
            ..InputValidity::default()
        };

        if self.is_effectively_disabled(node) {
            return Ok(validity);
        }

        let Some(tag_name) = self.dom.tag_name(node) else {
            return Ok(validity);
        };
        if !tag_name.eq_ignore_ascii_case("input") {
            let custom_error = !self.dom.custom_validity_message(node)?.is_empty();
            validity.custom_error = custom_error;
            validity.valid = !custom_error;
            return Ok(validity);
        }

        let input_type = self.normalized_input_type(node);
        if !Self::input_participates_in_constraint_validation(input_type.as_str()) {
            return Ok(validity);
        }
        let value = self.dom.value(node)?;
        let value_is_empty = value.is_empty();
        let required = self.dom.required(node);
        let multiple = self.dom.attr(node, "multiple").is_some();

        if required {
            validity.value_missing = if input_type == "checkbox" {
                !self.dom.checked(node)?
            } else if input_type == "radio" {
                !self.is_radio_group_checked(node)
            } else {
                value_is_empty
            };
        }

        if !value_is_empty {
            if input_type == "email" {
                validity.type_mismatch = if multiple {
                    let mut has_part = false;
                    let mut all_ok = true;
                    for part in value.split(',') {
                        let part = part.trim();
                        if part.is_empty() {
                            continue;
                        }
                        has_part = true;
                        if !Self::is_simple_email(part) {
                            all_ok = false;
                            break;
                        }
                    }
                    !has_part || !all_ok
                } else {
                    !Self::is_simple_email(&value)
                };
            } else if input_type == "url" {
                validity.type_mismatch = !Self::is_url_like(&value);
            }

            if matches!(
                input_type.as_str(),
                "text" | "search" | "url" | "tel" | "email" | "password"
            ) {
                let value_len = value.chars().count() as i64;
                if let Some(min_len) = self.parse_attr_i64(node, "minlength") {
                    if min_len >= 0 && value_len < min_len {
                        validity.too_short = true;
                    }
                }
                if let Some(max_len) = self.parse_attr_i64(node, "maxlength") {
                    if max_len >= 0 && value_len > max_len {
                        validity.too_long = true;
                    }
                }

                if let Some(pattern) = self.dom.attr(node, "pattern") {
                    if !pattern.is_empty() {
                        let wrapped = format!("^(?:{})$", pattern);
                        if let Ok(regex) = Regex::new(&wrapped) {
                            if input_type == "email" && multiple {
                                for part in value.split(',') {
                                    let part = part.trim();
                                    if part.is_empty() {
                                        continue;
                                    }
                                    match regex.is_match(part) {
                                        Ok(true) => {}
                                        Ok(false) => {
                                            validity.pattern_mismatch = true;
                                            break;
                                        }
                                        Err(_) => {}
                                    }
                                }
                            } else if let Ok(false) = regex.is_match(&value) {
                                validity.pattern_mismatch = true;
                            }
                        }
                    }
                }
            }

            if matches!(input_type.as_str(), "number" | "range") {
                match Self::parse_number_value(&value) {
                    Some(numeric) => {
                        if let Some(min) = self.parse_attr_f64(node, "min") {
                            if numeric < min {
                                validity.range_underflow = true;
                            }
                        }
                        if let Some(max) = self.parse_attr_f64(node, "max") {
                            if numeric > max {
                                validity.range_overflow = true;
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        if !step_attr.eq_ignore_ascii_case("any") {
                            let step = step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(1.0);
                            let base = self
                                .parse_attr_f64(node, "min")
                                .or_else(|| self.parse_attr_f64(node, "value"))
                                .unwrap_or(0.0);
                            let ratio = (numeric - base) / step;
                            let nearest = ratio.round();
                            if (ratio - nearest).abs() > 1e-7 {
                                validity.step_mismatch = true;
                            }
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            }
        }

        validity.custom_error = !self.dom.custom_validity_message(node)?.is_empty();
        validity.valid = !(validity.value_missing
            || validity.type_mismatch
            || validity.pattern_mismatch
            || validity.too_long
            || validity.too_short
            || validity.range_underflow
            || validity.range_overflow
            || validity.step_mismatch
            || validity.bad_input
            || validity.custom_error);
        Ok(validity)
    }

    pub(crate) fn input_validity_to_value(validity: &InputValidity) -> Value {
        Self::new_object_value(vec![
            ("valueMissing".to_string(), Value::Bool(validity.value_missing)),
            ("typeMismatch".to_string(), Value::Bool(validity.type_mismatch)),
            (
                "patternMismatch".to_string(),
                Value::Bool(validity.pattern_mismatch),
            ),
            ("tooLong".to_string(), Value::Bool(validity.too_long)),
            ("tooShort".to_string(), Value::Bool(validity.too_short)),
            (
                "rangeUnderflow".to_string(),
                Value::Bool(validity.range_underflow),
            ),
            (
                "rangeOverflow".to_string(),
                Value::Bool(validity.range_overflow),
            ),
            ("stepMismatch".to_string(), Value::Bool(validity.step_mismatch)),
            ("badInput".to_string(), Value::Bool(validity.bad_input)),
            ("customError".to_string(), Value::Bool(validity.custom_error)),
            ("valid".to_string(), Value::Bool(validity.valid)),
        ])
    }

    pub(crate) fn collect_left_associative_binary_operands<'a>(
        expr: &'a Expr,
        op: BinaryOp,
    ) -> Vec<&'a Expr> {
        let mut right_operands = Vec::new();
        let mut cursor = expr;
        loop {
            match cursor {
                Expr::Binary {
                    left,
                    op: inner_op,
                    right,
                } if *inner_op == op => {
                    right_operands.push(right.as_ref());
                    cursor = left.as_ref();
                }
                _ => break,
            }
        }

        let mut out = Vec::with_capacity(right_operands.len() + 1);
        out.push(cursor);
        while let Some(operand) = right_operands.pop() {
            out.push(operand);
        }
        out
    }

    pub(crate) fn eval_binary(&self, op: &BinaryOp, left: &Value, right: &Value) -> Result<Value> {
        if matches!(left, Value::Symbol(_)) || matches!(right, Value::Symbol(_)) {
            if matches!(
                op,
                BinaryOp::BitOr
                    | BinaryOp::BitXor
                    | BinaryOp::BitAnd
                    | BinaryOp::ShiftLeft
                    | BinaryOp::ShiftRight
                    | BinaryOp::UnsignedShiftRight
                    | BinaryOp::Pow
                    | BinaryOp::Lt
                    | BinaryOp::Gt
                    | BinaryOp::Le
                    | BinaryOp::Ge
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Mod
                    | BinaryOp::Div
            ) {
                return Err(Error::ScriptRuntime(
                    "Cannot convert a Symbol value to a number".into(),
                ));
            }
        }
        let out = match op {
            BinaryOp::Or => {
                if left.truthy() {
                    left.clone()
                } else {
                    right.clone()
                }
            }
            BinaryOp::And => {
                if left.truthy() {
                    right.clone()
                } else {
                    left.clone()
                }
            }
            BinaryOp::Nullish => {
                if matches!(left, Value::Null | Value::Undefined) {
                    right.clone()
                } else {
                    left.clone()
                }
            }
            BinaryOp::Eq => Value::Bool(self.loose_equal(left, right)),
            BinaryOp::Ne => Value::Bool(!self.loose_equal(left, right)),
            BinaryOp::StrictEq => Value::Bool(self.strict_equal(left, right)),
            BinaryOp::StrictNe => Value::Bool(!self.strict_equal(left, right)),
            BinaryOp::In => Value::Bool(self.value_in(left, right)),
            BinaryOp::InstanceOf => Value::Bool(self.value_instance_of(left, right)),
            BinaryOp::BitOr => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l | r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                Value::Number(i64::from(
                    self.to_i32_for_bitwise(left) | self.to_i32_for_bitwise(right),
                ))
            }
            BinaryOp::BitXor => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l ^ r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                Value::Number(i64::from(
                    self.to_i32_for_bitwise(left) ^ self.to_i32_for_bitwise(right),
                ))
            }
            BinaryOp::BitAnd => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l & r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                Value::Number(i64::from(
                    self.to_i32_for_bitwise(left) & self.to_i32_for_bitwise(right),
                ))
            }
            BinaryOp::ShiftLeft => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(Self::bigint_shift_left(l, r)?));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_i32_for_bitwise(left) << shift))
            }
            BinaryOp::ShiftRight => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(Self::bigint_shift_right(l, r)?));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_i32_for_bitwise(left) >> shift))
            }
            BinaryOp::UnsignedShiftRight => {
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "BigInt values do not support unsigned right shift".into(),
                    ));
                }
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_u32_for_bitwise(left) >> shift))
            }
            BinaryOp::Pow => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    if r.sign() == Sign::Minus {
                        return Err(Error::ScriptRuntime(
                            "BigInt exponent must be non-negative".into(),
                        ));
                    }
                    let exp = r.to_u32().ok_or_else(|| {
                        Error::ScriptRuntime("BigInt exponent is too large".into())
                    })?;
                    return Ok(Value::BigInt(l.pow(exp)));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                Value::Float(self.numeric_value(left).powf(self.numeric_value(right)))
            }
            BinaryOp::Lt => Value::Bool(self.compare(left, right, |l, r| l < r)),
            BinaryOp::Gt => Value::Bool(self.compare(left, right, |l, r| l > r)),
            BinaryOp::Le => Value::Bool(self.compare(left, right, |l, r| l <= r)),
            BinaryOp::Ge => Value::Bool(self.compare(left, right, |l, r| l >= r)),
            BinaryOp::Sub => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l - r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                Value::Float(self.numeric_value(left) - self.numeric_value(right))
            }
            BinaryOp::Mul => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l * r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                Value::Float(self.numeric_value(left) * self.numeric_value(right))
            }
            BinaryOp::Mod => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    if r.is_zero() {
                        return Err(Error::ScriptRuntime("modulo by zero".into()));
                    }
                    return Ok(Value::BigInt(l % r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                let rhs = self.numeric_value(right);
                if rhs == 0.0 {
                    return Err(Error::ScriptRuntime("modulo by zero".into()));
                }
                Value::Float(self.numeric_value(left) % rhs)
            }
            BinaryOp::Div => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    if r.is_zero() {
                        return Err(Error::ScriptRuntime("division by zero".into()));
                    }
                    return Ok(Value::BigInt(l / r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                Value::Float(self.numeric_value(left) / self.numeric_value(right))
            }
        };
        Ok(out)
    }

    pub(crate) fn loose_equal(&self, left: &Value, right: &Value) -> bool {
        if self.strict_equal(left, right) {
            return true;
        }

        match (left, right) {
            (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null) => true,
            (Value::BigInt(l), Value::String(r)) => {
                Self::parse_js_bigint_from_string(r).is_ok_and(|parsed| parsed == *l)
            }
            (Value::String(l), Value::BigInt(r)) => {
                Self::parse_js_bigint_from_string(l).is_ok_and(|parsed| parsed == *r)
            }
            (Value::BigInt(_), Value::Number(_) | Value::Float(_))
            | (Value::Number(_) | Value::Float(_), Value::BigInt(_)) => {
                Self::number_bigint_loose_equal(left, right)
            }
            (Value::Number(_) | Value::Float(_), Value::String(_))
            | (Value::String(_), Value::Number(_) | Value::Float(_)) => {
                Self::coerce_number_for_global(left) == Self::coerce_number_for_global(right)
            }
            (Value::Bool(_), _) => {
                let coerced = Value::Float(Self::coerce_number_for_global(left));
                self.loose_equal(&coerced, right)
            }
            (_, Value::Bool(_)) => {
                let coerced = Value::Float(Self::coerce_number_for_global(right));
                self.loose_equal(left, &coerced)
            }
            _ if Self::is_loose_primitive(left) && Self::is_loose_object(right) => {
                let prim = self.to_primitive_for_loose(right);
                self.loose_equal(left, &prim)
            }
            _ if Self::is_loose_object(left) && Self::is_loose_primitive(right) => {
                let prim = self.to_primitive_for_loose(left);
                self.loose_equal(&prim, right)
            }
            _ => false,
        }
    }

    pub(crate) fn is_loose_primitive(value: &Value) -> bool {
        matches!(
            value,
            Value::String(_)
                | Value::Bool(_)
                | Value::Number(_)
                | Value::Float(_)
                | Value::BigInt(_)
                | Value::Symbol(_)
                | Value::Null
                | Value::Undefined
        )
    }

    pub(crate) fn is_loose_object(value: &Value) -> bool {
        matches!(
            value,
            Value::Array(_)
                | Value::Object(_)
                | Value::Promise(_)
                | Value::Map(_)
                | Value::Set(_)
                | Value::Blob(_)
                | Value::ArrayBuffer(_)
                | Value::TypedArray(_)
                | Value::StringConstructor
                | Value::TypedArrayConstructor(_)
                | Value::BlobConstructor
                | Value::UrlConstructor
                | Value::ArrayBufferConstructor
                | Value::PromiseConstructor
                | Value::MapConstructor
                | Value::SetConstructor
                | Value::SymbolConstructor
                | Value::RegExpConstructor
                | Value::PromiseCapability(_)
                | Value::RegExp(_)
                | Value::Date(_)
                | Value::Node(_)
                | Value::NodeList(_)
                | Value::FormData(_)
                | Value::Function(_)
        )
    }

    pub(crate) fn to_primitive_for_loose(&self, value: &Value) -> Value {
        match value {
            Value::Object(entries) => {
                if let Some(wrapped) = Self::string_wrapper_value_from_object(&entries.borrow()) {
                    return Value::String(wrapped);
                }
                if let Some(id) = Self::symbol_wrapper_id_from_object(&entries.borrow()) {
                    if let Some(symbol) = self.symbol_runtime.symbols_by_id.get(&id) {
                        return Value::Symbol(symbol.clone());
                    }
                }
                Value::String(value.as_string())
            }
            Value::Array(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::RegExp(_)
            | Value::Date(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_) => Value::String(value.as_string()),
            _ => value.clone(),
        }
    }

    pub(crate) fn value_in(&self, left: &Value, right: &Value) -> bool {
        match right {
            Value::NodeList(nodes) => self
                .value_as_index(left)
                .is_some_and(|index| index < nodes.len()),
            Value::Array(values) => self
                .value_as_index(left)
                .is_some_and(|index| index < values.borrow().len()),
            Value::TypedArray(values) => self
                .value_as_index(left)
                .is_some_and(|index| index < values.borrow().observed_length()),
            Value::Object(entries) => {
                let key = self.property_key_to_storage_key(left);
                entries.borrow().iter().any(|(name, _)| name == &key)
            }
            Value::FormData(entries) => {
                let key = left.as_string();
                entries.iter().any(|(name, _)| name == &key)
            }
            _ => false,
        }
    }

    pub(crate) fn value_instance_of(&self, left: &Value, right: &Value) -> bool {
        if let Value::Node(node) = left {
            if self.is_named_constructor_value(right, "HTMLElement") {
                return self.dom.element(*node).is_some();
            }
            if self.is_named_constructor_value(right, "HTMLInputElement") {
                return self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("input"))
                    .unwrap_or(false);
            }
        }

        match (left, right) {
            (Value::Node(left), Value::Node(right)) => left == right,
            (Value::Node(left), Value::NodeList(nodes)) => nodes.contains(left),
            (Value::Array(left), Value::Array(right)) => Rc::ptr_eq(left, right),
            (Value::Map(left), Value::Map(right)) => Rc::ptr_eq(left, right),
            (Value::Set(left), Value::Set(right)) => Rc::ptr_eq(left, right),
            (Value::Promise(left), Value::Promise(right)) => Rc::ptr_eq(left, right),
            (Value::TypedArray(left), Value::TypedArray(right)) => Rc::ptr_eq(left, right),
            (Value::Blob(left), Value::Blob(right)) => Rc::ptr_eq(left, right),
            (Value::ArrayBuffer(left), Value::ArrayBuffer(right)) => Rc::ptr_eq(left, right),
            (Value::Object(left), Value::Object(right)) => Rc::ptr_eq(left, right),
            (Value::RegExp(left), Value::RegExp(right)) => Rc::ptr_eq(left, right),
            (Value::Symbol(left), Value::Symbol(right)) => left.id == right.id,
            (Value::Date(left), Value::Date(right)) => Rc::ptr_eq(left, right),
            (Value::FormData(left), Value::FormData(right)) => left == right,
            (Value::Blob(_), Value::BlobConstructor) => true,
            (Value::Object(left), Value::UrlConstructor) => Self::is_url_object(&left.borrow()),
            (Value::Object(left), Value::StringConstructor) => {
                Self::string_wrapper_value_from_object(&left.borrow()).is_some()
            }
            _ => false,
        }
    }

    fn is_named_constructor_value(&self, value: &Value, name: &str) -> bool {
        self.script_runtime.env
            .get(name)
            .is_some_and(|expected| self.strict_equal(value, expected))
    }

    pub(crate) fn value_as_index(&self, value: &Value) -> Option<usize> {
        match value {
            Value::Number(v) => usize::try_from(*v).ok(),
            Value::Float(v) => {
                if !v.is_finite() || v.fract() != 0.0 || *v < 0.0 {
                    None
                } else {
                    usize::try_from(*v as i64).ok()
                }
            }
            Value::BigInt(v) => v.to_usize(),
            Value::String(s) => {
                if let Ok(int) = s.parse::<i64>() {
                    usize::try_from(int).ok()
                } else if let Ok(float) = s.parse::<f64>() {
                    if float.fract() == 0.0 && float >= 0.0 {
                        usize::try_from(float as i64).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn strict_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::Float(l), Value::Float(r)) => l == r,
            (Value::Number(l), Value::Float(r)) => (*l as f64) == *r,
            (Value::Float(l), Value::Number(r)) => *l == (*r as f64),
            (Value::BigInt(l), Value::BigInt(r)) => l == r,
            (Value::Symbol(l), Value::Symbol(r)) => l.id == r.id,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Node(l), Value::Node(r)) => l == r,
            (Value::Array(l), Value::Array(r)) => Rc::ptr_eq(l, r),
            (Value::Map(l), Value::Map(r)) => Rc::ptr_eq(l, r),
            (Value::Set(l), Value::Set(r)) => Rc::ptr_eq(l, r),
            (Value::Promise(l), Value::Promise(r)) => Rc::ptr_eq(l, r),
            (Value::TypedArray(l), Value::TypedArray(r)) => Rc::ptr_eq(l, r),
            (Value::Blob(l), Value::Blob(r)) => Rc::ptr_eq(l, r),
            (Value::ArrayBuffer(l), Value::ArrayBuffer(r)) => Rc::ptr_eq(l, r),
            (Value::StringConstructor, Value::StringConstructor) => true,
            (Value::TypedArrayConstructor(l), Value::TypedArrayConstructor(r)) => l == r,
            (Value::BlobConstructor, Value::BlobConstructor) => true,
            (Value::UrlConstructor, Value::UrlConstructor) => true,
            (Value::ArrayBufferConstructor, Value::ArrayBufferConstructor) => true,
            (Value::PromiseConstructor, Value::PromiseConstructor) => true,
            (Value::MapConstructor, Value::MapConstructor) => true,
            (Value::SetConstructor, Value::SetConstructor) => true,
            (Value::SymbolConstructor, Value::SymbolConstructor) => true,
            (Value::RegExpConstructor, Value::RegExpConstructor) => true,
            (Value::PromiseCapability(l), Value::PromiseCapability(r)) => Rc::ptr_eq(l, r),
            (Value::Object(l), Value::Object(r)) => Rc::ptr_eq(l, r),
            (Value::RegExp(l), Value::RegExp(r)) => Rc::ptr_eq(l, r),
            (Value::Date(l), Value::Date(r)) => Rc::ptr_eq(l, r),
            (Value::Function(l), Value::Function(r)) => Rc::ptr_eq(l, r),
            (Value::FormData(l), Value::FormData(r)) => l == r,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Undefined) => true,
            _ => false,
        }
    }

    pub(crate) fn compare<F>(&self, left: &Value, right: &Value, op: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Value::String(l), Value::String(r)) => {
                let ordering = l.cmp(r);
                let cmp = if ordering.is_lt() {
                    -1.0
                } else if ordering.is_gt() {
                    1.0
                } else {
                    0.0
                };
                return op(cmp, 0.0);
            }
            (Value::BigInt(l), Value::BigInt(r)) => {
                return op(
                    l.to_f64().unwrap_or_else(|| {
                        if l.sign() == Sign::Minus {
                            f64::NEG_INFINITY
                        } else {
                            f64::INFINITY
                        }
                    }),
                    r.to_f64().unwrap_or_else(|| {
                        if r.sign() == Sign::Minus {
                            f64::NEG_INFINITY
                        } else {
                            f64::INFINITY
                        }
                    }),
                );
            }
            (Value::BigInt(l), Value::Number(_) | Value::Float(_)) => {
                let r = self.numeric_value(right);
                if r.is_nan() {
                    return false;
                }
                if let Some(rb) = Self::f64_to_bigint_if_integral(r) {
                    return op(
                        l.to_f64().unwrap_or_else(|| {
                            if l.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                        rb.to_f64().unwrap_or_else(|| {
                            if rb.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                    );
                }
                return op(
                    l.to_f64().unwrap_or_else(|| {
                        if l.sign() == Sign::Minus {
                            f64::NEG_INFINITY
                        } else {
                            f64::INFINITY
                        }
                    }),
                    r,
                );
            }
            (Value::Number(_) | Value::Float(_), Value::BigInt(r)) => {
                let l = self.numeric_value(left);
                if l.is_nan() {
                    return false;
                }
                if let Some(lb) = Self::f64_to_bigint_if_integral(l) {
                    return op(
                        lb.to_f64().unwrap_or_else(|| {
                            if lb.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                        r.to_f64().unwrap_or_else(|| {
                            if r.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                    );
                }
                return op(
                    l,
                    r.to_f64().unwrap_or_else(|| {
                        if r.sign() == Sign::Minus {
                            f64::NEG_INFINITY
                        } else {
                            f64::INFINITY
                        }
                    }),
                );
            }
            _ => {}
        }
        let l = self.numeric_value(left);
        let r = self.numeric_value(right);
        op(l, r)
    }

    pub(crate) fn number_bigint_loose_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::BigInt(l), Value::Number(r)) => *l == JsBigInt::from(*r),
            (Value::BigInt(l), Value::Float(r)) => {
                Self::f64_to_bigint_if_integral(*r).is_some_and(|rb| rb == *l)
            }
            (Value::Number(l), Value::BigInt(r)) => JsBigInt::from(*l) == *r,
            (Value::Float(l), Value::BigInt(r)) => {
                Self::f64_to_bigint_if_integral(*l).is_some_and(|lb| lb == *r)
            }
            _ => false,
        }
    }

    pub(crate) fn f64_to_bigint_if_integral(value: f64) -> Option<JsBigInt> {
        if !value.is_finite() || value.fract() != 0.0 {
            return None;
        }
        if value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            return Some(JsBigInt::from(value as i64));
        }
        let rendered = format!("{value:.0}");
        JsBigInt::parse_bytes(rendered.as_bytes(), 10)
    }

    pub(crate) fn add_values(&self, left: &Value, right: &Value) -> Result<Value> {
        if matches!(left, Value::Symbol(_)) || matches!(right, Value::Symbol(_)) {
            return Err(Error::ScriptRuntime(
                "Cannot convert a Symbol value to a string".into(),
            ));
        }
        if matches!(left, Value::String(_)) || matches!(right, Value::String(_)) {
            return Ok(Value::String(format!(
                "{}{}",
                left.as_string(),
                right.as_string()
            )));
        }

        if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
            return match (left, right) {
                (Value::BigInt(l), Value::BigInt(r)) => Ok(Value::BigInt(l + r)),
                _ => Err(Error::ScriptRuntime(
                    "cannot mix BigInt and other types in addition".into(),
                )),
            };
        }

        match (left, right) {
            (Value::Number(l), Value::Number(r)) => {
                if let Some(sum) = l.checked_add(*r) {
                    Ok(Value::Number(sum))
                } else {
                    Ok(Value::Float((*l as f64) + (*r as f64)))
                }
            }
            _ => Ok(Value::Float(
                self.numeric_value(left) + self.numeric_value(right),
            )),
        }
    }

    pub(crate) fn new_array_value(values: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(values)))
    }

    pub(crate) fn new_object_value(entries: Vec<(String, Value)>) -> Value {
        Value::Object(Rc::new(RefCell::new(ObjectValue::new(entries))))
    }

    pub(crate) fn new_boolean_constructor_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("boolean_constructor".to_string()),
        )])
    }

    pub(crate) fn new_string_wrapper_value(value: String) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
            Value::String(value),
        )])
    }

    pub(crate) fn object_set_entry(
        entries: &mut impl ObjectEntryMut,
        key: String,
        value: Value,
    ) {
        entries.set_entry(key, value);
    }

    pub(crate) fn object_get_entry(
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> Option<Value> {
        entries.get_entry(key)
    }

    pub(crate) fn callable_kind_from_value(value: &Value) -> Option<&str> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        match Self::object_get_entry(&entries, INTERNAL_CALLABLE_KIND_KEY) {
            Some(Value::String(kind)) => Some(match kind.as_str() {
                "intl_collator_compare" => "intl_collator_compare",
                "intl_date_time_format" => "intl_date_time_format",
                "intl_duration_format" => "intl_duration_format",
                "intl_list_format" => "intl_list_format",
                "intl_number_format" => "intl_number_format",
                "intl_segmenter_segments_iterator" => "intl_segmenter_segments_iterator",
                "intl_segmenter_iterator_next" => "intl_segmenter_iterator_next",
                "boolean_constructor" => "boolean_constructor",
                _ => return None,
            }),
            _ => None,
        }
    }

    pub(crate) fn object_property_from_value(&self, value: &Value, key: &str) -> Result<Value> {
        match value {
            Value::Node(node) => {
                let is_select = self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("select"))
                    .unwrap_or(false);
                let select_options = || {
                    let mut options = Vec::new();
                    self.dom.collect_select_options(*node, &mut options);
                    options
                };

                match key {
                    "textContent" | "innerText" => Ok(Value::String(self.dom.text_content(*node))),
                    "innerHTML" => Ok(Value::String(self.dom.inner_html(*node)?)),
                    "outerHTML" => Ok(Value::String(self.dom.outer_html(*node)?)),
                    "value" => Ok(Value::String(self.dom.value(*node)?)),
                    "checked" => Ok(Value::Bool(self.dom.checked(*node)?)),
                    "disabled" => Ok(Value::Bool(self.dom.disabled(*node))),
                    "required" => Ok(Value::Bool(self.dom.required(*node))),
                    "readonly" | "readOnly" => Ok(Value::Bool(self.dom.readonly(*node))),
                    "id" => Ok(Value::String(self.dom.attr(*node, "id").unwrap_or_default())),
                    "name" => Ok(Value::String(self.dom.attr(*node, "name").unwrap_or_default())),
                    "type" => Ok(Value::String(self.dom.attr(*node, "type").unwrap_or_default())),
                    "tagName" => Ok(Value::String(
                        self.dom
                            .tag_name(*node)
                            .unwrap_or_default()
                            .to_ascii_uppercase(),
                    )),
                    "className" => Ok(Value::String(
                        self.dom.attr(*node, "class").unwrap_or_default(),
                    )),
                    "options" => {
                        if !is_select {
                            return Ok(Value::Undefined);
                        }
                        Ok(Value::NodeList(select_options()))
                    }
                    "selectedIndex" => {
                        if !is_select {
                            return Ok(Value::Undefined);
                        }
                        let options = select_options();
                        if options.is_empty() {
                            return Ok(Value::Number(-1));
                        }
                        let selected = options
                            .iter()
                            .position(|option| self.dom.attr(*option, "selected").is_some())
                            .unwrap_or(0);
                        Ok(Value::Number(selected as i64))
                    }
                    "length" => {
                        if !is_select {
                            return Ok(Value::Undefined);
                        }
                        Ok(Value::Number(select_options().len() as i64))
                    }
                    _ => Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(*node, key.to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)),
                }
            }
            Value::String(text) => {
                if key == "length" {
                    Ok(Value::Number(text.chars().count() as i64))
                } else if key == "constructor" {
                    Ok(Value::StringConstructor)
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(text
                        .chars()
                        .nth(index)
                        .map(|ch| Value::String(ch.to_string()))
                        .unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Array(values) => {
                let values = values.borrow();
                if key == "length" {
                    Ok(Value::Number(values.len() as i64))
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(values.get(index).cloned().unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::NodeList(nodes) => {
                if key == "length" {
                    Ok(Value::Number(nodes.len() as i64))
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(nodes
                        .get(index)
                        .copied()
                        .map(Value::Node)
                        .unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::TypedArray(values) => {
                let snapshot = self.typed_array_snapshot(values)?;
                if key == "length" {
                    Ok(Value::Number(snapshot.len() as i64))
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(snapshot.get(index).cloned().unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(text) = Self::string_wrapper_value_from_object(&entries) {
                    if key == "length" {
                        return Ok(Value::Number(text.chars().count() as i64));
                    }
                    if key == "constructor" {
                        return Ok(Value::StringConstructor);
                    }
                    if let Ok(index) = key.parse::<usize>() {
                        return Ok(text
                            .chars()
                            .nth(index)
                            .map(|ch| Value::String(ch.to_string()))
                            .unwrap_or(Value::Undefined));
                    }
                }
                if Self::is_url_search_params_object(&entries) {
                    if key == "size" {
                        let size =
                            Self::url_search_params_pairs_from_object_entries(&entries).len();
                        return Ok(Value::Number(size as i64));
                    }
                }
                if Self::is_storage_object(&entries) {
                    if key == "length" {
                        let len = Self::storage_pairs_from_object_entries(&entries).len();
                        return Ok(Value::Number(len as i64));
                    }
                    if let Some(value) = Self::object_get_entry(&entries, key) {
                        return Ok(value);
                    }
                    if Self::is_storage_method_name(key) {
                        return Ok(Self::new_builtin_placeholder_function());
                    }
                    if let Some((_, value)) = Self::storage_pairs_from_object_entries(&entries)
                        .into_iter()
                        .find(|(name, _)| name == key)
                    {
                        return Ok(Value::String(value));
                    }
                    return Ok(Value::Undefined);
                }
                if Self::is_url_object(&entries) && key == "constructor" {
                    return Ok(Value::UrlConstructor);
                }
                Ok(Self::object_get_entry(&entries, key)
                    .unwrap_or(Value::Undefined))
            }
            Value::Promise(promise) => {
                if key == "constructor" {
                    Ok(Value::PromiseConstructor)
                } else {
                    let promise = promise.borrow();
                    if key == "status" {
                        let status = match &promise.state {
                            PromiseState::Pending => "pending",
                            PromiseState::Fulfilled(_) => "fulfilled",
                            PromiseState::Rejected(_) => "rejected",
                        };
                        Ok(Value::String(status.to_string()))
                    } else {
                        Ok(Value::Undefined)
                    }
                }
            }
            Value::Map(map) => {
                let map = map.borrow();
                if key == "size" {
                    Ok(Value::Number(map.entries.len() as i64))
                } else if key == "constructor" {
                    Ok(Value::MapConstructor)
                } else if let Some(value) = Self::object_get_entry(&map.properties, key) {
                    Ok(value)
                } else if Self::is_map_method_name(key) {
                    Ok(Self::new_builtin_placeholder_function())
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Set(set) => {
                let set = set.borrow();
                if key == "size" {
                    Ok(Value::Number(set.values.len() as i64))
                } else if key == "constructor" {
                    Ok(Value::SetConstructor)
                } else {
                    Ok(Self::object_get_entry(&set.properties, key).unwrap_or(Value::Undefined))
                }
            }
            Value::Blob(blob) => {
                let blob = blob.borrow();
                match key {
                    "size" => Ok(Value::Number(blob.bytes.len() as i64)),
                    "type" => Ok(Value::String(blob.mime_type.clone())),
                    "constructor" => Ok(Value::BlobConstructor),
                    _ => Ok(Value::Undefined),
                }
            }
            Value::ArrayBuffer(_) => {
                if key == "constructor" {
                    Ok(Value::ArrayBufferConstructor)
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Symbol(symbol) => {
                let value = match key {
                    "description" => symbol
                        .description
                        .as_ref()
                        .map(|value| Value::String(value.clone()))
                        .unwrap_or(Value::Undefined),
                    "constructor" => Value::SymbolConstructor,
                    _ => Value::Undefined,
                };
                Ok(value)
            }
            Value::RegExp(regex) => {
                let regex = regex.borrow();
                let value = match key {
                    "source" => Value::String(regex.source.clone()),
                    "flags" => Value::String(regex.flags.clone()),
                    "global" => Value::Bool(regex.global),
                    "ignoreCase" => Value::Bool(regex.ignore_case),
                    "multiline" => Value::Bool(regex.multiline),
                    "dotAll" => Value::Bool(regex.dot_all),
                    "sticky" => Value::Bool(regex.sticky),
                    "hasIndices" => Value::Bool(regex.has_indices),
                    "unicode" => Value::Bool(regex.unicode),
                    "unicodeSets" => Value::Bool(false),
                    "lastIndex" => Value::Number(regex.last_index as i64),
                    "constructor" => Value::RegExpConstructor,
                    _ => Self::object_get_entry(&regex.properties, key).unwrap_or(Value::Undefined),
                };
                Ok(value)
            }
            Value::UrlConstructor => {
                if let Some(value) =
                    Self::object_get_entry(&self.browser_apis.url_constructor_properties.borrow(), key)
                {
                    return Ok(value);
                }
                if Self::is_url_static_method_name(key) {
                    return Ok(Self::new_builtin_placeholder_function());
                }
                Ok(Value::Undefined)
            }
            Value::StringConstructor => Ok(Value::Undefined),
            _ => Err(Error::ScriptRuntime("value is not an object".into())),
        }
    }

    pub(crate) fn object_property_from_named_value(
        &self,
        variable_name: &str,
        value: &Value,
        key: &str,
    ) -> Result<Value> {
        self.object_property_from_value(value, key)
            .map_err(|err| match err {
                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                    Error::ScriptRuntime(format!(
                        "variable '{}' is not an object (key '{}')",
                        variable_name, key
                    ))
                }
                other => other,
            })
    }

    pub(crate) fn eval_event_prop_fallback(
        &self,
        event_var: &str,
        value: &Value,
        prop: EventExprProp,
    ) -> Result<Value> {
        let read =
            |value: &Value, key: &str| self.object_property_from_named_value(event_var, value, key);
        match prop {
            EventExprProp::Type => read(value, "type"),
            EventExprProp::Target => read(value, "target"),
            EventExprProp::CurrentTarget => read(value, "currentTarget"),
            EventExprProp::TargetName => {
                let target = read(value, "target")?;
                read(&target, "name")
            }
            EventExprProp::CurrentTargetName => {
                let target = read(value, "currentTarget")?;
                read(&target, "name")
            }
            EventExprProp::DefaultPrevented => read(value, "defaultPrevented"),
            EventExprProp::IsTrusted => read(value, "isTrusted"),
            EventExprProp::Bubbles => read(value, "bubbles"),
            EventExprProp::Cancelable => read(value, "cancelable"),
            EventExprProp::TargetId => {
                let target = read(value, "target")?;
                read(&target, "id")
            }
            EventExprProp::CurrentTargetId => {
                let target = read(value, "currentTarget")?;
                read(&target, "id")
            }
            EventExprProp::EventPhase => read(value, "eventPhase"),
            EventExprProp::TimeStamp => read(value, "timeStamp"),
            EventExprProp::State => read(value, "state"),
            EventExprProp::OldState => read(value, "oldState"),
            EventExprProp::NewState => read(value, "newState"),
        }
    }

    pub(crate) fn aria_property_to_attr_name(prop_name: &str) -> String {
        if !prop_name.starts_with("aria") || prop_name.len() <= 4 {
            return prop_name.to_ascii_lowercase();
        }
        format!("aria-{}", prop_name[4..].to_ascii_lowercase())
    }

    pub(crate) fn aria_element_ref_attr_name(prop_name: &str) -> Option<&'static str> {
        match prop_name {
            "ariaActiveDescendantElement" => Some("aria-activedescendant"),
            "ariaControlsElements" => Some("aria-controls"),
            "ariaDescribedByElements" => Some("aria-describedby"),
            "ariaDetailsElements" => Some("aria-details"),
            "ariaErrorMessageElements" => Some("aria-errormessage"),
            "ariaFlowToElements" => Some("aria-flowto"),
            "ariaLabelledByElements" => Some("aria-labelledby"),
            "ariaOwnsElements" => Some("aria-owns"),
            _ => None,
        }
    }

    pub(crate) fn resolve_aria_single_element_property(
        &self,
        node: NodeId,
        prop_name: &str,
    ) -> Option<NodeId> {
        let attr_name = Self::aria_element_ref_attr_name(prop_name)?;
        let raw = self.dom.attr(node, attr_name)?;
        let id_ref = raw.split_whitespace().next()?;
        self.dom.by_id(id_ref)
    }

    pub(crate) fn resolve_aria_element_list_property(
        &self,
        node: NodeId,
        prop_name: &str,
    ) -> Vec<NodeId> {
        let Some(attr_name) = Self::aria_element_ref_attr_name(prop_name) else {
            return Vec::new();
        };
        let Some(raw) = self.dom.attr(node, attr_name) else {
            return Vec::new();
        };
        raw.split_whitespace()
            .filter_map(|id_ref| self.dom.by_id(id_ref))
            .collect()
    }

    pub(crate) fn object_key_from_dom_prop(prop: &DomProp) -> Option<&'static str> {
        match prop {
            DomProp::Attributes => Some("attributes"),
            DomProp::AssignedSlot => Some("assignedSlot"),
            DomProp::Value => Some("value"),
            DomProp::ValidationMessage => Some("validationMessage"),
            DomProp::Validity => Some("validity"),
            DomProp::SelectionStart => Some("selectionStart"),
            DomProp::SelectionEnd => Some("selectionEnd"),
            DomProp::SelectionDirection => Some("selectionDirection"),
            DomProp::Checked => Some("checked"),
            DomProp::Indeterminate => Some("indeterminate"),
            DomProp::Open => Some("open"),
            DomProp::ReturnValue => Some("returnValue"),
            DomProp::ClosedBy => Some("closedBy"),
            DomProp::Readonly => Some("readOnly"),
            DomProp::Required => Some("required"),
            DomProp::Disabled => Some("disabled"),
            DomProp::TextContent => Some("textContent"),
            DomProp::InnerText => Some("innerText"),
            DomProp::InnerHtml => Some("innerHTML"),
            DomProp::OuterHtml => Some("outerHTML"),
            DomProp::ClassName => Some("className"),
            DomProp::ClassList => Some("classList"),
            DomProp::Part => Some("part"),
            DomProp::Id => Some("id"),
            DomProp::TagName => Some("tagName"),
            DomProp::LocalName => Some("localName"),
            DomProp::NamespaceUri => Some("namespaceURI"),
            DomProp::Prefix => Some("prefix"),
            DomProp::NextElementSibling => Some("nextElementSibling"),
            DomProp::PreviousElementSibling => Some("previousElementSibling"),
            DomProp::Slot => Some("slot"),
            DomProp::Role => Some("role"),
            DomProp::ElementTiming => Some("elementTiming"),
            DomProp::Name => Some("name"),
            DomProp::Lang => Some("lang"),
            DomProp::ClientWidth => Some("clientWidth"),
            DomProp::ClientHeight => Some("clientHeight"),
            DomProp::ClientLeft => Some("clientLeft"),
            DomProp::ClientTop => Some("clientTop"),
            DomProp::CurrentCssZoom => Some("currentCSSZoom"),
            DomProp::OffsetWidth => Some("offsetWidth"),
            DomProp::OffsetHeight => Some("offsetHeight"),
            DomProp::OffsetLeft => Some("offsetLeft"),
            DomProp::OffsetTop => Some("offsetTop"),
            DomProp::ScrollWidth => Some("scrollWidth"),
            DomProp::ScrollHeight => Some("scrollHeight"),
            DomProp::ScrollLeft => Some("scrollLeft"),
            DomProp::ScrollTop => Some("scrollTop"),
            DomProp::ScrollLeftMax => Some("scrollLeftMax"),
            DomProp::ScrollTopMax => Some("scrollTopMax"),
            DomProp::ShadowRoot => Some("shadowRoot"),
            DomProp::Children => Some("children"),
            DomProp::ChildElementCount => Some("childElementCount"),
            DomProp::FirstElementChild => Some("firstElementChild"),
            DomProp::LastElementChild => Some("lastElementChild"),
            DomProp::Title => Some("title"),
            DomProp::AnchorAttributionSrc => Some("attributionSrc"),
            DomProp::AnchorDownload => Some("download"),
            DomProp::AnchorHash => Some("hash"),
            DomProp::AnchorHost => Some("host"),
            DomProp::AnchorHostname => Some("hostname"),
            DomProp::AnchorHref => Some("href"),
            DomProp::AnchorHreflang => Some("hreflang"),
            DomProp::AnchorInterestForElement => Some("interestForElement"),
            DomProp::AnchorOrigin => Some("origin"),
            DomProp::AnchorPassword => Some("password"),
            DomProp::AnchorPathname => Some("pathname"),
            DomProp::AnchorPing => Some("ping"),
            DomProp::AnchorPort => Some("port"),
            DomProp::AnchorProtocol => Some("protocol"),
            DomProp::AnchorReferrerPolicy => Some("referrerPolicy"),
            DomProp::AnchorRel => Some("rel"),
            DomProp::AnchorRelList => Some("relList"),
            DomProp::AnchorSearch => Some("search"),
            DomProp::AnchorTarget => Some("target"),
            DomProp::AnchorText => Some("text"),
            DomProp::AnchorType => Some("type"),
            DomProp::AnchorUsername => Some("username"),
            DomProp::AnchorCharset => Some("charset"),
            DomProp::AnchorCoords => Some("coords"),
            DomProp::AnchorRev => Some("rev"),
            DomProp::AnchorShape => Some("shape"),
            DomProp::Dataset(_)
            | DomProp::Style(_)
            | DomProp::ClassListLength
            | DomProp::PartLength
            | DomProp::AriaString(_)
            | DomProp::AriaElementRefSingle(_)
            | DomProp::AriaElementRefList(_)
            | DomProp::ValueLength
            | DomProp::ValidityValueMissing
            | DomProp::ValidityTypeMismatch
            | DomProp::ValidityPatternMismatch
            | DomProp::ValidityTooLong
            | DomProp::ValidityTooShort
            | DomProp::ValidityRangeUnderflow
            | DomProp::ValidityRangeOverflow
            | DomProp::ValidityStepMismatch
            | DomProp::ValidityBadInput
            | DomProp::ValidityValid
            | DomProp::ValidityCustomError
            | DomProp::ActiveElement
            | DomProp::CharacterSet
            | DomProp::CompatMode
            | DomProp::ContentType
            | DomProp::ReadyState
            | DomProp::Referrer
            | DomProp::Url
            | DomProp::DocumentUri
            | DomProp::Location
            | DomProp::LocationHref
            | DomProp::LocationProtocol
            | DomProp::LocationHost
            | DomProp::LocationHostname
            | DomProp::LocationPort
            | DomProp::LocationPathname
            | DomProp::LocationSearch
            | DomProp::LocationHash
            | DomProp::LocationOrigin
            | DomProp::LocationAncestorOrigins
            | DomProp::History
            | DomProp::HistoryLength
            | DomProp::HistoryState
            | DomProp::HistoryScrollRestoration
            | DomProp::DefaultView
            | DomProp::Hidden
            | DomProp::VisibilityState
            | DomProp::Forms
            | DomProp::Images
            | DomProp::Links
            | DomProp::Scripts
            | DomProp::CurrentScript
            | DomProp::FormsLength
            | DomProp::ImagesLength
            | DomProp::LinksLength
            | DomProp::ScriptsLength
            | DomProp::ChildrenLength
            | DomProp::AnchorRelListLength => None,
        }
    }

}
