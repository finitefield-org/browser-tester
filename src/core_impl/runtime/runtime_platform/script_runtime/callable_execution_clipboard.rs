use super::*;

impl Harness {
    pub(crate) fn new_clipboard_item_value_from_constructor_args(
        &self,
        args: &[Value],
    ) -> Result<Value> {
        if args.is_empty() {
            return Err(Error::ScriptRuntime(
                "ClipboardItem constructor requires at least one argument".into(),
            ));
        }
        if args.len() > 2 {
            return Err(Error::ScriptRuntime(
                "ClipboardItem constructor supports up to two arguments".into(),
            ));
        }

        let Value::Object(entries) = &args[0] else {
            return Err(Error::ScriptRuntime(
                "ClipboardItem constructor requires a data object".into(),
            ));
        };
        let entries = entries.borrow();
        let mut instance_entries = vec![
            (
                INTERNAL_CLIPBOARD_ITEM_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                "presentationStyle".to_string(),
                Value::String("unspecified".to_string()),
            ),
        ];
        let mut types = Vec::new();

        for (mime_type, payload) in entries.iter() {
            if Self::is_internal_object_key(mime_type) {
                continue;
            }
            let mime_type = mime_type.to_ascii_lowercase();
            let blob = Self::clipboard_payload_to_blob(payload, &mime_type)?;
            instance_entries.push((mime_type.clone(), Value::Blob(blob)));
            types.push(Value::String(mime_type));
        }

        if types.is_empty() {
            return Err(Error::ScriptRuntime(
                "ClipboardItem constructor requires at least one clipboard type".into(),
            ));
        }
        instance_entries.push(("types".to_string(), Self::new_array_value(types)));
        Ok(Self::new_object_value(instance_entries))
    }

    pub(crate) fn eval_clipboard_write_call(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(Error::ScriptRuntime(
                "navigator.clipboard.write requires exactly one argument".into(),
            ));
        }

        let promise = self.new_pending_promise();
        if let Some(reason) = self.platform_mocks.clipboard_write_error.clone() {
            self.promise_reject(&promise, Value::String(reason));
            return Ok(Value::Promise(promise));
        }

        let mut write_artifact = ClipboardWriteArtifact {
            payloads: Vec::new(),
        };
        let Value::Array(items) = &args[0] else {
            self.promise_reject(
                &promise,
                Value::String(
                    "navigator.clipboard.write requires an array of ClipboardItem".into(),
                ),
            );
            return Ok(Value::Promise(promise));
        };

        for item in items.borrow().iter() {
            let payloads = self.clipboard_payloads_from_item_value(item)?;
            write_artifact.payloads.extend(payloads);
        }

        self.browser_apis.clipboard_writes.push(write_artifact);
        self.promise_resolve(&promise, Value::Undefined)?;
        Ok(Value::Promise(promise))
    }

    fn clipboard_payloads_from_item_value(
        &self,
        item: &Value,
    ) -> Result<Vec<ClipboardPayloadArtifact>> {
        let Value::Object(entries) = item else {
            return Err(Error::ScriptRuntime(
                "Clipboard.write items must be objects".into(),
            ));
        };
        let entries = entries.borrow();

        let types = if Self::is_clipboard_item_object(&entries) {
            match Self::object_get_entry(&entries, "types") {
                Some(Value::Array(types)) => types
                    .borrow()
                    .iter()
                    .map(Value::as_string)
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            }
        } else {
            entries
                .iter()
                .filter_map(|(key, _)| {
                    Self::is_enumerable_object_key(&entries, key).then_some(key.clone())
                })
                .collect::<Vec<_>>()
        };

        let mut payloads = Vec::new();
        for mime_type in types {
            let Some(payload) = Self::object_get_entry(&entries, &mime_type) else {
                continue;
            };
            let blob = Self::clipboard_payload_to_blob(&payload, &mime_type)?;
            let blob = blob.borrow();
            payloads.push(ClipboardPayloadArtifact {
                mime_type: mime_type.clone(),
                bytes: blob.bytes.clone(),
            });
        }

        if payloads.is_empty() {
            return Err(Error::ScriptRuntime(
                "ClipboardItem must provide at least one payload".into(),
            ));
        }
        Ok(payloads)
    }

    fn clipboard_payload_to_blob(
        payload: &Value,
        mime_type_hint: &str,
    ) -> Result<Rc<RefCell<BlobValue>>> {
        match payload {
            Value::Blob(blob) => Ok(blob.clone()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if !Self::is_mock_file_object(&entries) {
                    return Err(Error::ScriptRuntime(
                        "ClipboardItem payload must be a Blob or mock file".into(),
                    ));
                }
                let blob = match Self::object_get_entry(&entries, INTERNAL_MOCK_FILE_BLOB_KEY) {
                    Some(Value::Blob(blob)) => blob,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "ClipboardItem payload has invalid mock file blob".into(),
                        ));
                    }
                };
                Ok(blob)
            }
            Value::String(text) => Ok(Rc::new(RefCell::new(BlobValue {
                bytes: text.as_bytes().to_vec(),
                mime_type: mime_type_hint.to_string(),
            }))),
            _ => Err(Error::ScriptRuntime(
                "ClipboardItem payload must be a Blob or string".into(),
            )),
        }
    }
}
