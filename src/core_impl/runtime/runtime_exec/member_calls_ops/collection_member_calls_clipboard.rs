use super::*;

impl Harness {
    pub(crate) fn normalize_clipboard_data_format(raw: &str) -> String {
        let normalized = raw.trim().to_ascii_lowercase();
        if normalized == "text" {
            "text/plain".to_string()
        } else {
            normalized
        }
    }

    pub(crate) fn clipboard_data_types_from_entries(
        entries: &impl ObjectEntryLookup,
    ) -> Vec<String> {
        let Some(Value::Array(types)) = Self::clipboard_data_types_array_from_entries(entries)
        else {
            return Vec::new();
        };
        types
            .borrow()
            .iter()
            .map(|value| value.as_string())
            .collect::<Vec<_>>()
    }

    fn clipboard_data_types_array_from_entries(entries: &impl ObjectEntryLookup) -> Option<Value> {
        Self::object_get_entry(entries, INTERNAL_CLIPBOARD_DATA_TYPES_KEY)
            .or_else(|| Self::object_get_entry(entries, "types"))
    }

    pub(crate) fn data_transfer_files_array_from_entries(
        entries: &impl ObjectEntryLookup,
    ) -> Option<Value> {
        Self::object_get_entry(entries, INTERNAL_DATA_TRANSFER_FILES_KEY)
            .or_else(|| Self::object_get_entry(entries, "files"))
    }

    fn data_transfer_items_value_from_entries(entries: &impl ObjectEntryLookup) -> Option<Value> {
        Self::object_get_entry(entries, INTERNAL_DATA_TRANSFER_ITEMS_KEY)
            .or_else(|| Self::object_get_entry(entries, "items"))
    }

    pub(crate) fn sync_clipboard_types_array(entries: &mut ObjectValue, types: &[String]) {
        let array = Self::clipboard_data_types_array_from_entries(entries)
            .and_then(|value| match value {
                Value::Array(array) => Some(array),
                _ => None,
            })
            .unwrap_or_else(|| {
                let value = Self::new_array_value(Vec::new());
                let Value::Array(array) = &value else {
                    unreachable!("new_array_value must return an array");
                };
                let array = array.clone();
                Self::object_set_entry(
                    entries,
                    INTERNAL_CLIPBOARD_DATA_TYPES_KEY.to_string(),
                    value,
                );
                array
            });
        array.borrow_mut().elements = types.iter().cloned().map(Value::String).collect();
    }

    pub(crate) fn clipboard_data_store_from_entries(
        entries: &impl ObjectEntryLookup,
    ) -> Option<Rc<RefCell<ObjectValue>>> {
        match Self::object_get_entry(entries, INTERNAL_CLIPBOARD_DATA_STORE_KEY) {
            Some(Value::Object(store)) => Some(store),
            _ => None,
        }
    }

    pub(crate) fn data_transfer_item_list_owner_and_event_type(
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

        if let Some(Value::Array(files)) = Self::data_transfer_files_array_from_entries(entries) {
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

    pub(crate) fn data_transfer_items_from_types_and_store(
        owner: Rc<RefCell<ObjectValue>>,
        entries: &impl ObjectEntryLookup,
        event_type: &str,
        types: &[String],
        store: &Rc<RefCell<ObjectValue>>,
    ) -> Value {
        let items = Self::data_transfer_items_from_entries(entries, types, store);
        if let Some(Value::Array(item_list)) = Self::data_transfer_items_value_from_entries(entries)
        {
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
        let (is_clipboard_data, is_data_transfer_item, own_override, builtin_deleted) = {
            let entries = object.borrow();
            (
                Self::is_clipboard_data_object(&entries),
                Self::is_data_transfer_item_object(&entries),
                Self::object_get_entry(&entries, member)
                    .is_some_and(|value| !Self::is_builtin_placeholder_value(&value)),
                Self::is_builtin_object_property_deleted(&entries, member),
            )
        };

        if !is_clipboard_data && !is_data_transfer_item {
            return Ok(None);
        }
        if own_override || builtin_deleted {
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
                Self::sync_clipboard_types_array(&mut entries, &types);

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
                    Self::object_set_entry(
                        &mut entries,
                        INTERNAL_DATA_TRANSFER_ITEMS_KEY.to_string(),
                        items,
                    );
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

                Self::sync_clipboard_types_array(&mut entries, &types);
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
                    Self::object_set_entry(
                        &mut entries,
                        INTERNAL_DATA_TRANSFER_ITEMS_KEY.to_string(),
                        items,
                    );
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
