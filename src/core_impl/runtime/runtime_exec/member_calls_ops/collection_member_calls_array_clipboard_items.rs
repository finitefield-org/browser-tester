use super::*;

impl Harness {
    pub(crate) fn try_eval_array_member_call_clipboard_items(
        &mut self,
        values: &Rc<RefCell<ArrayValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let value = match member {
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
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
