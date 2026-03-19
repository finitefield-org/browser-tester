use super::*;

impl Harness {
    fn array_flat_depth_arg(value: Option<&Value>) -> usize {
        let Some(value) = value else {
            return 1;
        };
        let depth = Self::coerce_number_for_global(value);
        if depth.is_nan() || depth <= 0.0 {
            0
        } else if !depth.is_finite() {
            usize::MAX
        } else {
            depth.floor().min(usize::MAX as f64) as usize
        }
    }

    fn flatten_array_value_into(out: &mut Vec<Value>, value: Value, depth: usize) {
        match value {
            Value::Array(values) if depth > 0 => {
                let snapshot = {
                    let values = values.borrow();
                    ArrayValue {
                        elements: values.elements.clone(),
                        properties: values.properties.clone(),
                    }
                };
                for index in 0..snapshot.len() {
                    if Self::array_index_is_hole(&snapshot, index) {
                        continue;
                    }
                    Self::flatten_array_value_into(
                        out,
                        snapshot[index].clone(),
                        depth.saturating_sub(1),
                    );
                }
            }
            other => out.push(other),
        }
    }

    pub(crate) fn eval_array_member_call(
        &mut self,
        values: &Rc<RefCell<ArrayValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Option<Value>> {
        {
            let values_ref = values.borrow();
            if Self::is_data_transfer_item_list_value(&values_ref)
                && matches!(member, "add" | "remove" | "clear")
            {
                let own_override = Self::object_get_entry(&values_ref.properties, member)
                    .is_some_and(|value| !Self::is_builtin_placeholder_value(&value));
                let builtin_deleted =
                    Self::is_builtin_object_property_deleted(&values_ref.properties, member);
                if own_override || builtin_deleted {
                    return Ok(None);
                }
            }
        }

        let value = match member {
            "item" if Self::is_dom_rect_list_value(&values.borrow()) => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "item requires zero or one argument".into(),
                    ));
                }
                let index = evaluated_args
                    .first()
                    .map(Self::value_to_i64)
                    .unwrap_or(0)
                    .max(0) as usize;
                values.borrow().get(index).cloned().unwrap_or(Value::Null)
            }
            "forEach" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "forEach requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let _ = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
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
                    out.push(self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
                    )?);
                }
                Self::new_array_value(out)
            }
            "flat" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "flat supports zero or one argument".into(),
                    ));
                }
                let snapshot = {
                    let values = values.borrow();
                    ArrayValue {
                        elements: values.elements.clone(),
                        properties: values.properties.clone(),
                    }
                };
                let depth = Self::array_flat_depth_arg(evaluated_args.first());
                let mut out = Vec::new();
                for index in 0..snapshot.len() {
                    if Self::array_index_is_hole(&snapshot, index) {
                        continue;
                    }
                    Self::flatten_array_value_into(&mut out, snapshot[index].clone(), depth);
                }
                Self::new_array_value(out)
            }
            "flatMap" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "flatMap requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut out = Vec::new();
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let mapped = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    match mapped {
                        Value::Array(mapped_values) => {
                            let mapped_values = mapped_values.borrow();
                            for index in 0..mapped_values.len() {
                                if Self::array_index_is_hole(&mapped_values, index) {
                                    continue;
                                }
                                out.push(mapped_values[index].clone());
                            }
                        }
                        other => out.push(other),
                    }
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
                    let keep = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item.clone(),
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
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
                    acc = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            acc,
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
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
                    let matched = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item.clone(),
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
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
                    let matched = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
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
                    let keep = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
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
                    let keep = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
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
            "indexOf" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "indexOf requires one or two arguments".into(),
                    ));
                }
                let search = evaluated_args[0].clone();
                let values_ref = values.borrow();
                let len = values_ref.len() as i64;
                let mut from = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                if from < 0 {
                    from = (len + from).max(0);
                } else {
                    from = from.min(len);
                }
                let mut found = -1i64;
                for index in from as usize..values_ref.len() {
                    if Self::array_index_is_hole(&values_ref, index) {
                        continue;
                    }
                    if self.strict_equal(&values_ref[index], &search) {
                        found = index as i64;
                        break;
                    }
                }
                Value::Number(found)
            }
            "lastIndexOf" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "lastIndexOf requires one or two arguments".into(),
                    ));
                }
                let search = evaluated_args[0].clone();
                let values_ref = values.borrow();
                let len = values_ref.len() as i64;
                let from = evaluated_args
                    .get(1)
                    .map(Self::value_to_i64)
                    .unwrap_or(len - 1);
                let from = if from < 0 {
                    (len + from).max(-1)
                } else {
                    from.min(len - 1)
                };
                if from < 0 {
                    Value::Number(-1)
                } else {
                    let mut found = -1i64;
                    for index in (0..=from as usize).rev() {
                        if Self::array_index_is_hole(&values_ref, index) {
                            continue;
                        }
                        if self.strict_equal(&values_ref[index], &search) {
                            found = index as i64;
                            break;
                        }
                    }
                    Value::Number(found)
                }
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
                    .map(|value| self.coerce_to_string_for_string_context(value))
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
                    out.push_str(&self.coerce_to_string_for_string_context(value));
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
            "add" => {
                let (owner, event_type) = {
                    let values_ref = values.borrow();
                    let Some(meta) =
                        Self::data_transfer_item_list_owner_and_event_type(&values_ref)
                    else {
                        return Ok(None);
                    };
                    meta
                };
                if !event_type.eq_ignore_ascii_case("dragstart") {
                    Value::Null
                } else if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "DataTransferItemList.add requires one or two arguments".into(),
                    ));
                } else {
                    let mut owner_entries = owner.borrow_mut();
                    let mut types = Self::clipboard_data_types_from_entries(&owner_entries);
                    let store = Self::clipboard_data_store_from_entries(&owner_entries)
                        .unwrap_or_else(|| Rc::new(RefCell::new(ObjectValue::default())));
                    let added = if evaluated_args.len() == 1 {
                        let file = evaluated_args[0].clone();
                        let Value::Object(file_object) = &file else {
                            return Err(Error::ScriptRuntime(
                                "TypeError: Failed to execute 'add' on 'DataTransferItemList': parameter 1 is not of type 'File'"
                                    .into(),
                            ));
                        };
                        {
                            let file_entries = file_object.borrow();
                            if !Self::is_mock_file_object(&file_entries) {
                                return Err(Error::ScriptRuntime(
                                    "TypeError: Failed to execute 'add' on 'DataTransferItemList': parameter 1 is not of type 'File'"
                                        .into(),
                                ));
                            }
                        }
                        if let Some(Value::Array(files)) =
                            Self::data_transfer_files_array_from_entries(&owner_entries)
                        {
                            files.borrow_mut().push(file.clone());
                        } else {
                            let files = Self::new_array_value(vec![file.clone()]);
                            Self::object_set_entry(
                                &mut owner_entries,
                                INTERNAL_DATA_TRANSFER_FILES_KEY.to_string(),
                                files,
                            );
                        }
                        let mime_type = {
                            let file_entries = file_object.borrow();
                            Self::object_get_entry(&file_entries, "type")
                                .map(|value| value.as_string())
                                .unwrap_or_default()
                        };
                        Self::new_data_transfer_item_file_value(&mime_type, file)
                    } else {
                        let data = evaluated_args[0].as_string();
                        let format =
                            Self::normalize_clipboard_data_format(&evaluated_args[1].as_string());
                        if format.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "DataTransferItemList.add requires a non-empty type for string data"
                                    .into(),
                            ));
                        }
                        if !types.iter().any(|item| item == &format) {
                            types.push(format.clone());
                        }
                        Self::object_set_entry(
                            &mut store.borrow_mut(),
                            format.clone(),
                            Value::String(data.clone()),
                        );
                        Self::sync_clipboard_types_array(&mut owner_entries, &types);
                        Self::object_set_entry(
                            &mut owner_entries,
                            INTERNAL_CLIPBOARD_DATA_STORE_KEY.to_string(),
                            Value::Object(store.clone()),
                        );
                        if format == "text/plain" {
                            Self::object_set_entry(
                                &mut owner_entries,
                                INTERNAL_CLIPBOARD_DATA_TEXT_KEY.to_string(),
                                Value::String(data.clone()),
                            );
                        }
                        Self::new_data_transfer_item_string_value(&format, &data)
                    };
                    let items = Self::data_transfer_items_from_types_and_store(
                        owner.clone(),
                        &owner_entries,
                        &event_type,
                        &types,
                        &store,
                    );
                    Self::object_set_entry(
                        &mut owner_entries,
                        INTERNAL_DATA_TRANSFER_ITEMS_KEY.to_string(),
                        items,
                    );
                    added
                }
            }
            "remove" => {
                let (owner, event_type) = {
                    let values_ref = values.borrow();
                    let Some(meta) =
                        Self::data_transfer_item_list_owner_and_event_type(&values_ref)
                    else {
                        return Ok(None);
                    };
                    meta
                };
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "DataTransferItemList.remove requires exactly one index argument".into(),
                    ));
                }
                if !event_type.eq_ignore_ascii_case("dragstart") {
                    Value::Undefined
                } else {
                    let index = Self::value_to_i64(&evaluated_args[0]);
                    if index < 0 {
                        return Ok(Some(Value::Undefined));
                    }
                    let mut owner_entries = owner.borrow_mut();
                    let mut types = Self::clipboard_data_types_from_entries(&owner_entries);
                    let store = Self::clipboard_data_store_from_entries(&owner_entries)
                        .unwrap_or_else(|| Rc::new(RefCell::new(ObjectValue::default())));
                    let index = index as usize;
                    if index < types.len() {
                        let removed = types.remove(index);
                        store.borrow_mut().delete_entry(&removed);
                    } else if let Some(file_index) = index.checked_sub(types.len()) {
                        if let Some(Value::Array(files)) =
                            Self::data_transfer_files_array_from_entries(&owner_entries)
                        {
                            if file_index < files.borrow().len() {
                                files.borrow_mut().remove(file_index);
                            }
                        }
                    }
                    Self::sync_clipboard_types_array(&mut owner_entries, &types);
                    Self::object_set_entry(
                        &mut owner_entries,
                        INTERNAL_CLIPBOARD_DATA_STORE_KEY.to_string(),
                        Value::Object(store.clone()),
                    );
                    let text = Self::object_get_entry(&store.borrow(), "text/plain")
                        .map(|value| value.as_string())
                        .unwrap_or_default();
                    Self::object_set_entry(
                        &mut owner_entries,
                        INTERNAL_CLIPBOARD_DATA_TEXT_KEY.to_string(),
                        Value::String(text),
                    );
                    let items = Self::data_transfer_items_from_types_and_store(
                        owner.clone(),
                        &owner_entries,
                        &event_type,
                        &types,
                        &store,
                    );
                    Self::object_set_entry(
                        &mut owner_entries,
                        INTERNAL_DATA_TRANSFER_ITEMS_KEY.to_string(),
                        items,
                    );
                    Value::Undefined
                }
            }
            "clear" => {
                let (owner, event_type) = {
                    let values_ref = values.borrow();
                    let Some(meta) =
                        Self::data_transfer_item_list_owner_and_event_type(&values_ref)
                    else {
                        return Ok(None);
                    };
                    meta
                };
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "DataTransferItemList.clear does not take arguments".into(),
                    ));
                }
                if !event_type.eq_ignore_ascii_case("dragstart") {
                    Value::Undefined
                } else {
                    let mut owner_entries = owner.borrow_mut();
                    let types = Vec::<String>::new();
                    let store = Self::clipboard_data_store_from_entries(&owner_entries)
                        .unwrap_or_else(|| Rc::new(RefCell::new(ObjectValue::default())));
                    store.borrow_mut().clear();
                    if let Some(Value::Array(files)) =
                        Self::data_transfer_files_array_from_entries(&owner_entries)
                    {
                        files.borrow_mut().clear();
                    }
                    Self::sync_clipboard_types_array(&mut owner_entries, &types);
                    Self::object_set_entry(
                        &mut owner_entries,
                        INTERNAL_CLIPBOARD_DATA_STORE_KEY.to_string(),
                        Value::Object(store.clone()),
                    );
                    Self::object_set_entry(
                        &mut owner_entries,
                        INTERNAL_CLIPBOARD_DATA_TEXT_KEY.to_string(),
                        Value::String(String::new()),
                    );
                    let items = Self::data_transfer_items_from_types_and_store(
                        owner.clone(),
                        &owner_entries,
                        &event_type,
                        &types,
                        &store,
                    );
                    Self::object_set_entry(
                        &mut owner_entries,
                        INTERNAL_DATA_TRANSFER_ITEMS_KEY.to_string(),
                        items,
                    );
                    Value::Undefined
                }
            }
            "push" => {
                let adopted_owner_document = {
                    let values_ref = values.borrow();
                    Self::adopted_style_sheets_owner_document(&values_ref)
                };
                if let Some(owner_document) = adopted_owner_document {
                    for item in evaluated_args {
                        if !self.is_css_style_sheet_for_document(item, &owner_document) {
                            return Err(Self::adopted_style_sheets_not_allowed_error());
                        }
                    }
                }
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
            "reverse" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "reverse does not take arguments".into(),
                    ));
                }
                let mut snapshot = values.borrow().clone();
                snapshot.reverse();
                values.borrow_mut().elements = snapshot;
                Value::Array(values.clone())
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
