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
            "getUTCFullYear" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCFullYear does not take arguments".into(),
                    ));
                }
                let (year, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(year)
            }
            "getUTCMonth" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCMonth does not take arguments".into(),
                    ));
                }
                let (_, month, ..) = Self::date_components_utc(*value.borrow());
                Value::Number((month as i64) - 1)
            }
            "getUTCDate" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCDate does not take arguments".into(),
                    ));
                }
                let (_, _, day, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(day as i64)
            }
            "getUTCDay" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCDay does not take arguments".into(),
                    ));
                }
                let timestamp_ms = *value.borrow();
                let days = timestamp_ms.div_euclid(86_400_000);
                let weekday = ((days + 4).rem_euclid(7)) as i64;
                Value::Number(weekday)
            }
            "getUTCHours" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCHours does not take arguments".into(),
                    ));
                }
                let (_, _, _, hour, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(hour as i64)
            }
            "getUTCMinutes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCMinutes does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, minute, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(minute as i64)
            }
            "getUTCSeconds" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCSeconds does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, _, second, _) = Self::date_components_utc(*value.borrow());
                Value::Number(second as i64)
            }
            "getUTCMilliseconds" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCMilliseconds does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, _, _, millisecond) = Self::date_components_utc(*value.borrow());
                Value::Number(millisecond as i64)
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
                            Self::object_get_entry(&owner_entries, "files")
                        {
                            files.borrow_mut().push(file.clone());
                        } else {
                            Self::object_set_entry(
                                &mut owner_entries,
                                "files".to_string(),
                                Self::new_array_value(vec![file.clone()]),
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
                        Self::object_set_entry(
                            &mut owner_entries,
                            "types".to_string(),
                            Self::new_array_value(
                                types.iter().cloned().map(Value::String).collect::<Vec<_>>(),
                            ),
                        );
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
                    Self::object_set_entry(&mut owner_entries, "items".to_string(), items);
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
                            Self::object_get_entry(&owner_entries, "files")
                        {
                            if file_index < files.borrow().len() {
                                files.borrow_mut().remove(file_index);
                            }
                        }
                    }
                    Self::object_set_entry(
                        &mut owner_entries,
                        "types".to_string(),
                        Self::new_array_value(
                            types.iter().cloned().map(Value::String).collect::<Vec<_>>(),
                        ),
                    );
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
                    Self::object_set_entry(&mut owner_entries, "items".to_string(), items);
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
                        Self::object_get_entry(&owner_entries, "files")
                    {
                        files.borrow_mut().clear();
                    }
                    Self::object_set_entry(
                        &mut owner_entries,
                        "types".to_string(),
                        Self::new_array_value(Vec::new()),
                    );
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
                    Self::object_set_entry(&mut owner_entries, "items".to_string(), items);
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

    pub(crate) fn eval_weak_map_member_call_from_values(
        &mut self,
        weak_map: &Rc<RefCell<WeakMapValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match member {
            "set" => {
                if evaluated_args.len() != 2 {
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
                if evaluated_args.len() != 1 {
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
                if evaluated_args.len() != 1 {
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
                if evaluated_args.len() != 1 {
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
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "WeakSet.add requires exactly one argument".into(),
                    ));
                }
                Self::ensure_weak_set_value(&evaluated_args[0])?;
                self.weak_set_add_value(&mut weak_set.borrow_mut(), evaluated_args[0].clone());
                Value::WeakSet(weak_set.clone())
            }
            "has" => {
                if evaluated_args.len() != 1 {
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
                if evaluated_args.len() != 1 {
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

    pub(crate) fn eval_nodelist_member_call(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "item" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "item requires exactly one index argument".into(),
                    ));
                }
                let index = Self::value_to_i64(&evaluated_args[0]);
                if index < 0 {
                    return Ok(Some(Value::Null));
                }
                Ok(Some(
                    self.node_list_get(nodes, index as usize)
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "forEach" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "forEach requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = self.node_list_snapshot(nodes);
                for (idx, node) in snapshot.iter().copied().enumerate() {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[
                            Value::Node(node),
                            Value::Number(idx as i64),
                            Value::NodeList(nodes.clone()),
                        ],
                        event,
                    )?;
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_named_node_map_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let owner = {
            let entries = object.borrow();
            if !Self::is_named_node_map_object(&entries) {
                return Ok(None);
            }
            Self::named_node_map_owner_node(&entries)
                .filter(|node| self.dom.element(*node).is_some())
        };

        let value = match member {
            "item" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.item requires exactly one index argument".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Ok(Some(Value::Null));
                };
                let index = Self::value_to_i64(&evaluated_args[0]);
                if index < 0 {
                    Value::Null
                } else {
                    self.named_node_map_entries(owner)
                        .get(index as usize)
                        .map(|(name, value)| Self::new_attr_object_value(name, value, Some(owner)))
                        .unwrap_or(Value::Null)
                }
            }
            "getNamedItem" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.getNamedItem requires exactly one argument".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Ok(Some(Value::Null));
                };
                self.eval_node_member_call(owner, "getAttributeNode", evaluated_args, event)?
                    .unwrap_or(Value::Null)
            }
            "setNamedItem" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.setNamedItem requires exactly one argument".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Err(Error::ScriptRuntime(
                        "setNamedItem target is not an element".into(),
                    ));
                };
                self.eval_node_member_call(owner, "setAttributeNode", evaluated_args, event)?
                    .unwrap_or(Value::Null)
            }
            "removeNamedItem" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.removeNamedItem requires exactly one argument".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeNamedItem': The attribute was not found"
                            .into(),
                    ));
                };
                let attr = self
                    .eval_node_member_call(owner, "getAttributeNode", evaluated_args, event)?
                    .unwrap_or(Value::Null);
                if matches!(attr, Value::Null) {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeNamedItem': The attribute was not found"
                            .into(),
                    ));
                }
                let _ =
                    self.eval_node_member_call(owner, "removeAttribute", evaluated_args, event)?;
                match attr {
                    Value::Object(attr_object) => {
                        let (name, value) = {
                            let entries = attr_object.borrow();
                            (
                                Self::object_get_entry(&entries, "name")
                                    .map(|entry| entry.as_string())
                                    .unwrap_or_default(),
                                Self::object_get_entry(&entries, "value")
                                    .map(|entry| entry.as_string())
                                    .unwrap_or_default(),
                            )
                        };
                        Self::new_attr_object_value(&name, &value, None)
                    }
                    _ => Value::Null,
                }
            }
            "getNamedItemNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.getNamedItemNS requires exactly two arguments".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Ok(Some(Value::Null));
                };
                self.eval_node_member_call(owner, "getAttributeNodeNS", evaluated_args, event)?
                    .unwrap_or(Value::Null)
            }
            "setNamedItemNS" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.setNamedItemNS requires exactly one argument".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Err(Error::ScriptRuntime(
                        "setNamedItemNS target is not an element".into(),
                    ));
                };
                self.eval_node_member_call(owner, "setAttributeNodeNS", evaluated_args, event)?
                    .unwrap_or(Value::Null)
            }
            "removeNamedItemNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.removeNamedItemNS requires exactly two arguments".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeNamedItemNS': The attribute was not found"
                            .into(),
                    ));
                };
                let attr = self
                    .eval_node_member_call(owner, "getAttributeNodeNS", evaluated_args, event)?
                    .unwrap_or(Value::Null);
                if matches!(attr, Value::Null) {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeNamedItemNS': The attribute was not found"
                            .into(),
                    ));
                }
                let _ =
                    self.eval_node_member_call(owner, "removeAttributeNS", evaluated_args, event)?;
                match attr {
                    Value::Object(attr_object) => {
                        let (name, value) = {
                            let entries = attr_object.borrow();
                            (
                                Self::object_get_entry(&entries, "name")
                                    .map(|entry| entry.as_string())
                                    .unwrap_or_default(),
                                Self::object_get_entry(&entries, "value")
                                    .map(|entry| entry.as_string())
                                    .unwrap_or_default(),
                            )
                        };
                        Self::new_attr_object_value(&name, &value, None)
                    }
                    _ => Value::Null,
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    fn normalize_clipboard_data_format(raw: &str) -> String {
        let normalized = raw.trim().to_ascii_lowercase();
        if normalized == "text" {
            "text/plain".to_string()
        } else {
            normalized
        }
    }

    pub(crate) fn eval_event_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let (is_event_object, is_navigate_event_object, is_pointer_event_object) = {
            let entries = object.borrow();
            (
                Self::is_event_object(&entries),
                Self::is_navigate_event_object(&entries),
                Self::is_pointer_event_object(&entries),
            )
        };
        if !is_event_object {
            return Ok(None);
        }

        let value = match member {
            "preventDefault" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Event.preventDefault does not take arguments".into(),
                    ));
                }
                let cancelable = {
                    let entries = object.borrow();
                    Self::object_get_entry(&entries, "cancelable")
                        .is_some_and(|value| value.truthy())
                };
                if cancelable {
                    Self::object_set_entry(
                        &mut object.borrow_mut(),
                        "defaultPrevented".to_string(),
                        Value::Bool(true),
                    );
                }
                Value::Undefined
            }
            "stopPropagation" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Event.stopPropagation does not take arguments".into(),
                    ));
                }
                Value::Undefined
            }
            "stopImmediatePropagation" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Event.stopImmediatePropagation does not take arguments".into(),
                    ));
                }
                Value::Undefined
            }
            "getModifierState" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "KeyboardEvent.getModifierState requires exactly one argument".into(),
                    ));
                }
                let modifier = evaluated_args[0].as_string();
                let entries = object.borrow();
                if !Self::is_keyboard_event_object(&entries) {
                    Value::Bool(false)
                } else {
                    Value::Bool(Self::event_modifier_state_from_entries(&entries, &modifier))
                }
            }
            "intercept" => {
                if !is_navigate_event_object {
                    return Ok(None);
                }
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "NavigateEvent.intercept supports at most one options argument".into(),
                    ));
                }

                let can_intercept = {
                    let entries = object.borrow();
                    Self::object_get_entry(&entries, "canIntercept")
                        .is_some_and(|value| value.truthy())
                };
                if !can_intercept {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Failed to execute 'intercept': navigation cannot be intercepted"
                            .into(),
                    ));
                }

                let mut handler = Value::Undefined;
                if let Some(options) = evaluated_args.first() {
                    match options {
                        Value::Null | Value::Undefined => {}
                        Value::Object(options_entries) => {
                            let options_entries = options_entries.borrow();
                            if let Some(value) = Self::object_get_entry(&options_entries, "handler")
                            {
                                if !matches!(value, Value::Null | Value::Undefined)
                                    && !self.is_callable_value(&value)
                                {
                                    return Err(Error::ScriptRuntime(
                                        "NavigateEvent.intercept handler must be callable".into(),
                                    ));
                                }
                                handler = value;
                            }
                        }
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "NavigateEvent.intercept options argument must be an object".into(),
                            ));
                        }
                    }
                }

                if !matches!(handler, Value::Null | Value::Undefined) {
                    let _ = self.execute_callback_value(&handler, &[], event)?;
                }

                Self::object_set_entry(
                    &mut object.borrow_mut(),
                    "\0\0bt_event:navigate:intercepted".to_string(),
                    Value::Bool(true),
                );
                Value::Undefined
            }
            "scroll" => {
                if !is_navigate_event_object {
                    return Ok(None);
                }
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "NavigateEvent.scroll does not take arguments".into(),
                    ));
                }
                Self::object_set_entry(
                    &mut object.borrow_mut(),
                    "\0\0bt_event:navigate:scroll_called".to_string(),
                    Value::Bool(true),
                );
                Value::Undefined
            }
            "getCoalescedEvents" => {
                if !is_pointer_event_object {
                    return Ok(None);
                }
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "PointerEvent.getCoalescedEvents does not take arguments".into(),
                    ));
                }
                Self::new_array_value(Vec::new())
            }
            "getPredictedEvents" => {
                if !is_pointer_event_object {
                    return Ok(None);
                }
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "PointerEvent.getPredictedEvents does not take arguments".into(),
                    ));
                }
                Self::new_array_value(Vec::new())
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    fn new_navigation_api_result_value(&mut self) -> Result<Value> {
        Ok(Self::new_object_value(vec![
            (
                "committed".to_string(),
                Value::Promise(self.promise_resolve_value_as_promise(Value::Undefined)?),
            ),
            (
                "finished".to_string(),
                Value::Promise(self.promise_resolve_value_as_promise(Value::Undefined)?),
            ),
        ]))
    }

    fn dispatch_navigation_simple_event(&mut self, event_type: &str) -> Result<()> {
        let _ = self.dispatch_event_target(
            self.location_history.navigation_object.clone(),
            Value::String(event_type.to_string()),
        )?;
        Ok(())
    }

    fn dispatch_navigation_navigate_event(
        &mut self,
        from_url: &str,
        destination_url: &str,
        navigation_type: &str,
    ) -> Result<()> {
        let payload = Self::new_object_value(vec![
            (INTERNAL_EVENT_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_NAVIGATE_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("type".to_string(), Value::String("navigate".to_string())),
            ("bubbles".to_string(), Value::Bool(false)),
            ("cancelable".to_string(), Value::Bool(true)),
            ("canIntercept".to_string(), Value::Bool(true)),
            (
                "destination".to_string(),
                Self::new_object_value(vec![(
                    "url".to_string(),
                    Value::String(destination_url.to_string()),
                )]),
            ),
            ("downloadRequest".to_string(), Value::Null),
            ("formData".to_string(), Value::Null),
            (
                "hashChange".to_string(),
                Value::Bool(Self::is_hash_only_navigation(from_url, destination_url)),
            ),
            ("hasUAVisualTransition".to_string(), Value::Bool(false)),
            ("info".to_string(), Value::Undefined),
            (
                "navigationType".to_string(),
                Value::String(navigation_type.to_string()),
            ),
            (
                "signal".to_string(),
                Self::new_navigate_event_default_signal_value(),
            ),
            ("sourceElement".to_string(), Value::Null),
            ("userInitiated".to_string(), Value::Bool(false)),
            (
                "intercept".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "scroll".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ]);
        let _ =
            self.dispatch_event_target(self.location_history.navigation_object.clone(), payload)?;
        Ok(())
    }

    pub(crate) fn eval_navigation_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let is_navigation_object = {
            let entries = object.borrow();
            Self::is_navigation_object(&entries)
        };
        if !is_navigation_object {
            return Ok(None);
        }

        let result = match member {
            "entries" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "navigation.entries does not take arguments".into(),
                    ));
                }
                self.navigation_entries_value()
            }
            "back" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "navigation.back does not take arguments".into(),
                    ));
                }
                if !self.navigation_can_go_back() {
                    return Ok(Some(self.new_navigation_api_result_value()?));
                }

                let from = self.document_url.clone();
                let destination = self
                    .location_history
                    .history_entries
                    .get(self.location_history.history_index.saturating_sub(1))
                    .map(|entry| entry.url.clone())
                    .unwrap_or_else(|| self.document_url.clone());
                self.dispatch_navigation_navigate_event(&from, &destination, "traverse")?;
                if let Err(err) = self.history_go_with_env(-1) {
                    let _ = self.dispatch_navigation_simple_event("navigateerror");
                    return Err(err);
                }
                self.dispatch_navigation_simple_event("currententrychange")?;
                self.dispatch_navigation_simple_event("navigatesuccess")?;
                self.new_navigation_api_result_value()?
            }
            "forward" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "navigation.forward does not take arguments".into(),
                    ));
                }
                if !self.navigation_can_go_forward() {
                    return Ok(Some(self.new_navigation_api_result_value()?));
                }

                let from = self.document_url.clone();
                let destination = self
                    .location_history
                    .history_entries
                    .get(self.location_history.history_index.saturating_add(1))
                    .map(|entry| entry.url.clone())
                    .unwrap_or_else(|| self.document_url.clone());
                self.dispatch_navigation_navigate_event(&from, &destination, "traverse")?;
                if let Err(err) = self.history_go_with_env(1) {
                    let _ = self.dispatch_navigation_simple_event("navigateerror");
                    return Err(err);
                }
                self.dispatch_navigation_simple_event("currententrychange")?;
                self.dispatch_navigation_simple_event("navigatesuccess")?;
                self.new_navigation_api_result_value()?
            }
            "traverseTo" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "navigation.traverseTo requires exactly one key argument".into(),
                    ));
                }
                let key = evaluated_args[0].as_string();
                let Some(target_index) = self.navigation_find_entry_index_by_key(&key) else {
                    return Err(Error::ScriptRuntime(format!(
                        "NotFoundError: navigation entry key not found: {key}"
                    )));
                };

                if target_index == self.location_history.history_index {
                    return Ok(Some(self.new_navigation_api_result_value()?));
                }

                let from = self.document_url.clone();
                let destination = self
                    .location_history
                    .history_entries
                    .get(target_index)
                    .map(|entry| entry.url.clone())
                    .unwrap_or_else(|| self.document_url.clone());
                self.dispatch_navigation_navigate_event(&from, &destination, "traverse")?;
                let delta = target_index as i64 - self.location_history.history_index as i64;
                if let Err(err) = self.history_go_with_env(delta) {
                    let _ = self.dispatch_navigation_simple_event("navigateerror");
                    return Err(err);
                }
                self.dispatch_navigation_simple_event("currententrychange")?;
                self.dispatch_navigation_simple_event("navigatesuccess")?;
                self.new_navigation_api_result_value()?
            }
            "navigate" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "navigation.navigate requires one or two arguments".into(),
                    ));
                }

                let raw_url = evaluated_args[0].as_string();
                let destination = self.try_resolve_location_target_url(&raw_url)?;
                let mut state = Value::Null;
                let mut replace = false;

                if let Some(options) = evaluated_args.get(1) {
                    match options {
                        Value::Null | Value::Undefined => {}
                        Value::Object(options_entries) => {
                            let options_entries = options_entries.borrow();
                            if let Some(value) = Self::object_get_entry(&options_entries, "state") {
                                state = value;
                            }
                            if Self::object_get_entry(&options_entries, "history")
                                .map(|value| value.as_string().eq_ignore_ascii_case("replace"))
                                .unwrap_or(false)
                            {
                                replace = true;
                            }
                        }
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "navigation.navigate options argument must be an object".into(),
                            ));
                        }
                    }
                }

                let from = self.document_url.clone();
                self.dispatch_navigation_navigate_event(
                    &from,
                    &destination,
                    if replace { "replace" } else { "push" },
                )?;

                let state = Self::structured_clone_value(&state, &mut Vec::new(), &mut Vec::new())?;
                self.document_url = destination.clone();
                if replace {
                    self.history_replace_current_entry(&destination, state);
                } else {
                    self.history_push_entry(&destination, state);
                }
                self.sync_location_object();
                self.sync_history_object();
                self.sync_navigation_object();
                self.sync_document_object();
                self.sync_window_runtime_properties();
                self.location_history
                    .location_navigations
                    .push(LocationNavigation {
                        kind: if replace {
                            LocationNavigationKind::Replace
                        } else {
                            LocationNavigationKind::Assign
                        },
                        from: from.clone(),
                        to: destination.clone(),
                    });
                if !Self::is_hash_only_navigation(&from, &destination) {
                    let _ = self.load_location_mock_page_if_exists(&destination)?;
                }
                self.dispatch_navigation_simple_event("currententrychange")?;
                self.dispatch_navigation_simple_event("navigatesuccess")?;
                self.new_navigation_api_result_value()?
            }
            "reload" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "navigation.reload supports zero or one options argument".into(),
                    ));
                }
                let current_url = self.document_url.clone();
                self.dispatch_navigation_navigate_event(&current_url, &current_url, "reload")?;

                let mut state_override = None;
                if let Some(options) = evaluated_args.first() {
                    match options {
                        Value::Null | Value::Undefined => {}
                        Value::Object(options_entries) => {
                            let options_entries = options_entries.borrow();
                            if let Some(value) = Self::object_get_entry(&options_entries, "state") {
                                state_override = Some(value);
                            }
                        }
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "navigation.reload options argument must be an object".into(),
                            ));
                        }
                    }
                }

                if let Err(err) = self.reload_location() {
                    let _ = self.dispatch_navigation_simple_event("navigateerror");
                    return Err(err);
                }

                if let Some(state) = state_override {
                    let cloned =
                        Self::structured_clone_value(&state, &mut Vec::new(), &mut Vec::new())?;
                    self.history_replace_current_entry(&self.document_url.clone(), cloned);
                    self.sync_history_object();
                    self.sync_navigation_object();
                    self.sync_window_runtime_properties();
                }
                self.dispatch_navigation_simple_event("navigatesuccess")?;
                self.new_navigation_api_result_value()?
            }
            "updateCurrentEntry" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "navigation.updateCurrentEntry requires exactly one options argument"
                            .into(),
                    ));
                }

                let options_entries = match &evaluated_args[0] {
                    Value::Object(options_entries) => options_entries.borrow(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "navigation.updateCurrentEntry options argument must be an object"
                                .into(),
                        ));
                    }
                };
                let state =
                    Self::object_get_entry(&options_entries, "state").unwrap_or(Value::Undefined);
                let cloned =
                    Self::structured_clone_value(&state, &mut Vec::new(), &mut Vec::new())?;
                self.history_replace_current_entry(&self.document_url.clone(), cloned);
                self.sync_history_object();
                self.sync_navigation_object();
                self.sync_window_runtime_properties();
                self.dispatch_navigation_simple_event("currententrychange")?;
                Value::Undefined
            }
            _ => return Ok(None),
        };

        Ok(Some(result))
    }

    fn clipboard_data_types_from_entries(entries: &impl ObjectEntryLookup) -> Vec<String> {
        let Some(Value::Array(types)) = Self::object_get_entry(entries, "types") else {
            return Vec::new();
        };
        types
            .borrow()
            .iter()
            .map(|value| value.as_string())
            .collect::<Vec<_>>()
    }

    fn clipboard_data_store_from_entries(
        entries: &impl ObjectEntryLookup,
    ) -> Option<Rc<RefCell<ObjectValue>>> {
        match Self::object_get_entry(entries, INTERNAL_CLIPBOARD_DATA_STORE_KEY) {
            Some(Value::Object(store)) => Some(store),
            _ => None,
        }
    }

    fn data_transfer_item_list_owner_and_event_type(
        values: &ArrayValue,
    ) -> Option<(Rc<RefCell<ObjectValue>>, String)> {
        if !Self::is_data_transfer_item_list_value(values) {
            return None;
        }
        let owner = match Self::object_get_entry(
            &values.properties,
            INTERNAL_DATA_TRANSFER_ITEM_LIST_OWNER_KEY,
        ) {
            Some(Value::Object(owner)) => owner,
            _ => return None,
        };
        let mut event_type = Self::object_get_entry(
            &values.properties,
            INTERNAL_DATA_TRANSFER_ITEM_LIST_EVENT_TYPE_KEY,
        )
        .map(|value| value.as_string().to_ascii_lowercase())
        .unwrap_or_default();
        if event_type.is_empty() {
            let owner_entries = owner.borrow();
            event_type =
                Self::object_get_entry(&owner_entries, INTERNAL_DATA_TRANSFER_EVENT_TYPE_KEY)
                    .map(|value| value.as_string().to_ascii_lowercase())
                    .unwrap_or_default();
        }
        Some((owner, event_type))
    }

    fn data_transfer_items_from_entries(
        entries: &impl ObjectEntryLookup,
        types: &[String],
        store: &Rc<RefCell<ObjectValue>>,
    ) -> Vec<Value> {
        let store_entries = store.borrow();
        let mut items = types
            .iter()
            .map(|format| {
                let data = Self::object_get_entry(&store_entries, format)
                    .map(|value| value.as_string())
                    .unwrap_or_default();
                Self::new_data_transfer_item_string_value(format, &data)
            })
            .collect::<Vec<_>>();

        if let Some(Value::Array(files)) = Self::object_get_entry(entries, "files") {
            for file in files.borrow().iter() {
                let Value::Object(file_object) = file else {
                    continue;
                };
                let file_entries = file_object.borrow();
                if !Self::is_mock_file_object(&file_entries) {
                    continue;
                }
                let mime_type = Self::object_get_entry(&file_entries, "type")
                    .map(|value| value.as_string())
                    .unwrap_or_default();
                items.push(Self::new_data_transfer_item_file_value(
                    &mime_type,
                    file.clone(),
                ));
            }
        }

        items
    }

    fn data_transfer_items_from_types_and_store(
        owner: Rc<RefCell<ObjectValue>>,
        entries: &impl ObjectEntryLookup,
        event_type: &str,
        types: &[String],
        store: &Rc<RefCell<ObjectValue>>,
    ) -> Value {
        let items = Self::data_transfer_items_from_entries(entries, types, store);
        if let Some(Value::Array(item_list)) = Self::object_get_entry(entries, "items") {
            let is_item_list = {
                let item_list_ref = item_list.borrow();
                Self::is_data_transfer_item_list_value(&item_list_ref)
            };
            if is_item_list {
                let mut item_list_ref = item_list.borrow_mut();
                item_list_ref.elements = items;
                Self::object_set_entry(
                    &mut item_list_ref.properties,
                    INTERNAL_DATA_TRANSFER_ITEM_LIST_OWNER_KEY.to_string(),
                    Value::Object(owner),
                );
                Self::object_set_entry(
                    &mut item_list_ref.properties,
                    INTERNAL_DATA_TRANSFER_ITEM_LIST_EVENT_TYPE_KEY.to_string(),
                    Value::String(event_type.to_ascii_lowercase()),
                );
                drop(item_list_ref);
                return Value::Array(item_list);
            }
        }
        Self::new_data_transfer_item_list_value(owner, event_type, items)
    }

    pub(crate) fn eval_clipboard_data_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let entries = object.borrow();
        let is_clipboard_data = Self::is_clipboard_data_object(&entries);
        let is_data_transfer_item = Self::is_data_transfer_item_object(&entries);
        drop(entries);

        if !is_clipboard_data && !is_data_transfer_item {
            return Ok(None);
        }

        if is_data_transfer_item {
            let value = match member {
                "getAsFile" => {
                    if !evaluated_args.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "DataTransferItem.getAsFile does not take arguments".into(),
                        ));
                    }
                    let entries = object.borrow();
                    let kind =
                        Self::object_get_entry(&entries, INTERNAL_DATA_TRANSFER_ITEM_KIND_KEY)
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                    if kind == "file" {
                        Self::object_get_entry(&entries, INTERNAL_DATA_TRANSFER_ITEM_DATA_KEY)
                            .unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    }
                }
                "getAsFileSystemHandle" => {
                    if !evaluated_args.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "DataTransferItem.getAsFileSystemHandle does not take arguments".into(),
                        ));
                    }
                    Value::Promise(self.promise_resolve_value_as_promise(Value::Null)?)
                }
                "webkitGetAsEntry" => {
                    if !evaluated_args.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "DataTransferItem.webkitGetAsEntry does not take arguments".into(),
                        ));
                    }
                    Value::Null
                }
                "getAsString" => {
                    if evaluated_args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "DataTransferItem.getAsString requires exactly one callback argument"
                                .into(),
                        ));
                    }
                    let callback = evaluated_args[0].clone();
                    if !self.is_callable_value(&callback) {
                        return Err(Error::ScriptRuntime(
                            "DataTransferItem.getAsString callback must be callable".into(),
                        ));
                    }
                    let entries = object.borrow();
                    let kind =
                        Self::object_get_entry(&entries, INTERNAL_DATA_TRANSFER_ITEM_KIND_KEY)
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                    if kind == "string" {
                        let data =
                            Self::object_get_entry(&entries, INTERNAL_DATA_TRANSFER_ITEM_DATA_KEY)
                                .map(|value| value.as_string())
                                .unwrap_or_default();
                        self.execute_callback_value(&callback, &[Value::String(data)], event)?;
                    }
                    Value::Undefined
                }
                _ => return Ok(None),
            };
            return Ok(Some(value));
        }

        let value = match member {
            "getData" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "clipboardData.getData requires exactly one format argument".into(),
                    ));
                }
                let format = Self::normalize_clipboard_data_format(&evaluated_args[0].as_string());
                let entries = object.borrow();
                if Self::is_data_transfer_object(&entries)
                    && !matches!(
                        Self::object_get_entry(&entries, INTERNAL_DATA_TRANSFER_EVENT_TYPE_KEY)
                            .map(|value| value.as_string().to_ascii_lowercase())
                            .as_deref(),
                        Some("dragstart" | "drop")
                    )
                {
                    return Ok(Some(Value::String(String::new())));
                }
                if let Some(store) = Self::clipboard_data_store_from_entries(&entries) {
                    if let Some(value) = Self::object_get_entry(&store.borrow(), &format) {
                        return Ok(Some(Value::String(value.as_string())));
                    }
                }
                let fallback_text =
                    Self::object_get_entry(&entries, INTERNAL_CLIPBOARD_DATA_TEXT_KEY)
                        .map(|value| value.as_string())
                        .unwrap_or_default();
                if format == "text/plain" {
                    Value::String(fallback_text)
                } else {
                    Value::String(String::new())
                }
            }
            "setData" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "clipboardData.setData requires exactly two arguments".into(),
                    ));
                }
                let format = Self::normalize_clipboard_data_format(&evaluated_args[0].as_string());
                let data = evaluated_args[1].as_string();
                let mut entries = object.borrow_mut();

                let mut types = Self::clipboard_data_types_from_entries(&entries);
                if !types.iter().any(|item| item == &format) {
                    types.push(format.clone());
                }
                Self::object_set_entry(
                    &mut entries,
                    "types".to_string(),
                    Self::new_array_value(
                        types.iter().cloned().map(Value::String).collect::<Vec<_>>(),
                    ),
                );

                let store = Self::clipboard_data_store_from_entries(&entries)
                    .unwrap_or_else(|| Rc::new(RefCell::new(ObjectValue::default())));
                Self::object_set_entry(
                    &mut store.borrow_mut(),
                    format.clone(),
                    Value::String(data.clone()),
                );
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_CLIPBOARD_DATA_STORE_KEY.to_string(),
                    Value::Object(store.clone()),
                );
                if Self::is_data_transfer_object(&entries) {
                    let event_type =
                        Self::object_get_entry(&entries, INTERNAL_DATA_TRANSFER_EVENT_TYPE_KEY)
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                    let items = Self::data_transfer_items_from_types_and_store(
                        object.clone(),
                        &entries,
                        &event_type,
                        &types,
                        &store,
                    );
                    Self::object_set_entry(&mut entries, "items".to_string(), items);
                }
                if format == "text/plain" {
                    Self::object_set_entry(
                        &mut entries,
                        INTERNAL_CLIPBOARD_DATA_TEXT_KEY.to_string(),
                        Value::String(data),
                    );
                }
                Value::Undefined
            }
            "clearData" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "clipboardData.clearData supports at most one argument".into(),
                    ));
                }

                let mut entries = object.borrow_mut();
                let writable = if Self::is_data_transfer_object(&entries) {
                    matches!(
                        Self::object_get_entry(&entries, INTERNAL_DATA_TRANSFER_EVENT_TYPE_KEY)
                            .map(|value| value.as_string().to_ascii_lowercase())
                            .as_deref(),
                        Some("dragstart")
                    )
                } else {
                    true
                };
                if !writable {
                    return Ok(Some(Value::Undefined));
                }

                let mut types = Self::clipboard_data_types_from_entries(&entries);
                let store = Self::clipboard_data_store_from_entries(&entries)
                    .unwrap_or_else(|| Rc::new(RefCell::new(ObjectValue::default())));

                if let Some(format_arg) = evaluated_args.first() {
                    let format = Self::normalize_clipboard_data_format(&format_arg.as_string());
                    if format.is_empty() {
                        types.clear();
                        store.borrow_mut().clear();
                    } else {
                        types.retain(|item| item != &format);
                        store.borrow_mut().delete_entry(&format);
                    }
                } else {
                    types.clear();
                    store.borrow_mut().clear();
                }

                Self::object_set_entry(
                    &mut entries,
                    "types".to_string(),
                    Self::new_array_value(
                        types.iter().cloned().map(Value::String).collect::<Vec<_>>(),
                    ),
                );
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_CLIPBOARD_DATA_STORE_KEY.to_string(),
                    Value::Object(store.clone()),
                );
                if Self::is_data_transfer_object(&entries) {
                    let event_type =
                        Self::object_get_entry(&entries, INTERNAL_DATA_TRANSFER_EVENT_TYPE_KEY)
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                    let items = Self::data_transfer_items_from_types_and_store(
                        object.clone(),
                        &entries,
                        &event_type,
                        &types,
                        &store,
                    );
                    Self::object_set_entry(&mut entries, "items".to_string(), items);
                }
                let text = Self::object_get_entry(&store.borrow(), "text/plain")
                    .map(|value| value.as_string())
                    .unwrap_or_default();
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_CLIPBOARD_DATA_TEXT_KEY.to_string(),
                    Value::String(text),
                );
                Value::Undefined
            }
            "setDragImage" => {
                if evaluated_args.len() != 3 {
                    return Err(Error::ScriptRuntime(
                        "dataTransfer.setDragImage requires exactly three arguments".into(),
                    ));
                }
                let mut entries = object.borrow_mut();
                if !Self::is_data_transfer_object(&entries) {
                    return Ok(None);
                }
                let writable = matches!(
                    Self::object_get_entry(&entries, INTERNAL_DATA_TRANSFER_EVENT_TYPE_KEY)
                        .map(|value| value.as_string().to_ascii_lowercase())
                        .as_deref(),
                    Some("dragstart")
                );
                if !writable {
                    return Ok(Some(Value::Undefined));
                }
                let image = match evaluated_args.first() {
                    Some(Value::Node(node)) if self.dom.element(*node).is_some() => *node,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "TypeError: Failed to execute 'setDragImage': parameter 1 is not of type 'Element'"
                                .into(),
                        ))
                    }
                };
                let x = Self::value_to_i64(&evaluated_args[1]);
                let y = Self::value_to_i64(&evaluated_args[2]);
                Self::object_set_entry(
                    &mut entries,
                    "\0\0bt_data_transfer:drag_image".to_string(),
                    Value::Node(image),
                );
                Self::object_set_entry(
                    &mut entries,
                    "\0\0bt_data_transfer:drag_image_x".to_string(),
                    Value::Number(x),
                );
                Self::object_set_entry(
                    &mut entries,
                    "\0\0bt_data_transfer:drag_image_y".to_string(),
                    Value::Number(y),
                );
                Value::Undefined
            }
            "addElement" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "dataTransfer.addElement requires exactly one argument".into(),
                    ));
                }
                let mut entries = object.borrow_mut();
                if !Self::is_data_transfer_object(&entries) {
                    return Ok(None);
                }
                let element = match evaluated_args.first() {
                    Some(Value::Node(node)) if self.dom.element(*node).is_some() => *node,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "TypeError: Failed to execute 'addElement': parameter 1 is not of type 'Element'"
                                .into(),
                        ))
                    }
                };
                Self::object_set_entry(
                    &mut entries,
                    "\0\0bt_data_transfer:drag_source_override".to_string(),
                    Value::Node(element),
                );
                Value::Undefined
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    pub(crate) fn eval_mock_file_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let entries = object.borrow();
        if !Self::is_mock_file_object(&entries) {
            return Ok(None);
        }
        let blob = match Self::object_get_entry(&entries, INTERNAL_MOCK_FILE_BLOB_KEY) {
            Some(Value::Blob(blob)) => blob,
            _ => return Ok(None),
        };
        drop(entries);

        self.eval_blob_member_call(&blob, member, evaluated_args)
    }
}
