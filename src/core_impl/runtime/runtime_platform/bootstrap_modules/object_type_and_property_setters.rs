impl Harness {
    pub(crate) fn is_history_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_HISTORY_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_window_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_WINDOW_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_navigator_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_NAVIGATOR_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_storage_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_STORAGE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn set_navigator_property(
        &mut self,
        navigator_object: &Rc<RefCell<ObjectValue>>,
        key: &str,
        value: Value,
    ) -> Result<()> {
        match key {
            "clipboard" => Err(Error::ScriptRuntime(
                "navigator.clipboard is read-only".into(),
            )),
            _ => {
                Self::object_set_entry(&mut navigator_object.borrow_mut(), key.to_string(), value);
                Ok(())
            }
        }
    }

    pub(crate) fn set_window_property(&mut self, key: &str, value: Value) -> Result<()> {
        match key {
            "window" | "self" | "top" | "parent" | "frames" | "length" | "closed" | "history"
            | "navigator" | "clientInformation" | "document" | "origin" | "isSecureContext"
            | "URL" | "HTMLElement" | "HTMLInputElement" | "localStorage" => {
                Err(Error::ScriptRuntime(format!("window.{key} is read-only")))
            }
            "location" => self.set_location_property("href", value),
            "name" => {
                Self::object_set_entry(
                    &mut self.dom_runtime.window_object.borrow_mut(),
                    "name".to_string(),
                    Value::String(value.as_string()),
                );
                Ok(())
            }
            _ => {
                Self::object_set_entry(
                    &mut self.dom_runtime.window_object.borrow_mut(),
                    key.to_string(),
                    value,
                );
                Ok(())
            }
        }
    }

    pub(crate) fn set_url_constructor_property(&mut self, key: &str, value: Value) {
        Self::object_set_entry(
            &mut self.browser_apis.url_constructor_properties.borrow_mut(),
            key.to_string(),
            value,
        );
    }

    pub(crate) fn set_storage_object_property(
        &mut self,
        storage_object: &Rc<RefCell<ObjectValue>>,
        key: &str,
        value: Value,
    ) -> Result<()> {
        match key {
            "length" => Err(Error::ScriptRuntime("Storage.length is read-only".into())),
            "getItem" | "setItem" | "removeItem" | "clear" | "key" => {
                Self::object_set_entry(&mut storage_object.borrow_mut(), key.to_string(), value);
                Ok(())
            }
            _ => {
                let mut entries = storage_object.borrow_mut();
                let mut pairs = Self::storage_pairs_from_object_entries(&entries);
                if let Some((_, stored)) = pairs.iter_mut().find(|(name, _)| name == key) {
                    *stored = value.as_string();
                } else {
                    pairs.push((key.to_string(), value.as_string()));
                }
                Self::set_storage_pairs(&mut entries, &pairs);
                Ok(())
            }
        }
    }

    pub(crate) fn set_history_property(&mut self, key: &str, value: Value) -> Result<()> {
        match key {
            "length" => Err(Error::ScriptRuntime("history.length is read-only".into())),
            "state" => Err(Error::ScriptRuntime("history.state is read-only".into())),
            "scrollRestoration" => {
                let mode = value.as_string();
                if mode != "auto" && mode != "manual" {
                    return Err(Error::ScriptRuntime(
                        "history.scrollRestoration must be 'auto' or 'manual'".into(),
                    ));
                }
                self.location_history.history_scroll_restoration = mode;
                self.sync_history_object();
                self.sync_window_runtime_properties();
                Ok(())
            }
            _ => {
                Self::object_set_entry(
                    &mut self.location_history.history_object.borrow_mut(),
                    key.to_string(),
                    value,
                );
                Ok(())
            }
        }
    }

    pub(crate) fn set_node_event_handler_property(
        &mut self,
        node: NodeId,
        key: &str,
        value: Value,
    ) -> Result<bool> {
        let Some(raw_event_type) = key.strip_prefix("on") else {
            return Ok(false);
        };
        if raw_event_type.is_empty() {
            return Ok(false);
        }

        let event_type = raw_event_type.to_ascii_lowercase();
        if let Some(previous_handler) = self
            .dom_runtime
            .node_event_handler_props
            .remove(&(node, event_type.clone()))
        {
            let _ = self
                .listeners
                .remove(node, &event_type, false, &previous_handler);
        }

        if let Value::Function(function) = value {
            let handler = function.handler.clone();
            self.listeners.add(
                node,
                event_type.clone(),
                Listener {
                    capture: false,
                    handler: handler.clone(),
                    captured_env: function.captured_env.clone(),
                    captured_pending_function_decls: function
                        .captured_pending_function_decls
                        .clone(),
                },
            );
            self.dom_runtime
                .node_event_handler_props
                .insert((node, event_type), handler);
        }
        Ok(true)
    }

    pub(crate) fn set_node_assignment_property(
        &mut self,
        node: NodeId,
        key: &str,
        value: Value,
    ) -> Result<()> {
        if self.set_node_event_handler_property(node, key, value.clone())? {
            return Ok(());
        }

        match key {
            "textContent" | "innerText" | "text" => {
                self.dom.set_text_content(node, &value.as_string())?
            }
            "innerHTML" => self.dom.set_inner_html(node, &value.as_string())?,
            "outerHTML" => self.dom.set_outer_html(node, &value.as_string())?,
            "value" => self.dom.set_value(node, &value.as_string())?,
            "checked" => self.dom.set_checked(node, value.truthy())?,
            "indeterminate" => self.dom.set_indeterminate(node, value.truthy())?,
            "open" => {
                if value.truthy() {
                    self.dom.set_attr(node, "open", "true")?;
                } else {
                    self.dom.remove_attr(node, "open")?;
                }
            }
            "returnValue" => {
                self.set_dialog_return_value(node, value.as_string())?;
            }
            "closedBy" | "closedby" => self.dom.set_attr(node, "closedby", &value.as_string())?,
            "readOnly" | "readonly" => {
                if value.truthy() {
                    self.dom.set_attr(node, "readonly", "true")?;
                } else {
                    self.dom.remove_attr(node, "readonly")?;
                }
            }
            "required" => {
                if value.truthy() {
                    self.dom.set_attr(node, "required", "true")?;
                } else {
                    self.dom.remove_attr(node, "required")?;
                }
            }
            "disabled" => {
                if value.truthy() {
                    self.dom.set_attr(node, "disabled", "true")?;
                } else {
                    self.dom.remove_attr(node, "disabled")?;
                }
            }
            "hidden" => {
                if node == self.dom.root {
                    return Err(Error::ScriptRuntime("hidden is read-only".into()));
                }
                if value.truthy() {
                    self.dom.set_attr(node, "hidden", "true")?;
                } else {
                    self.dom.remove_attr(node, "hidden")?;
                }
            }
            "className" => self.dom.set_attr(node, "class", &value.as_string())?,
            "id" => self.dom.set_attr(node, "id", &value.as_string())?,
            "slot" => self.dom.set_attr(node, "slot", &value.as_string())?,
            "role" => self.dom.set_attr(node, "role", &value.as_string())?,
            "elementTiming" => self
                .dom
                .set_attr(node, "elementtiming", &value.as_string())?,
            "name" => self.dom.set_attr(node, "name", &value.as_string())?,
            "lang" => self.dom.set_attr(node, "lang", &value.as_string())?,
            "title" => self.dom.set_document_title(&value.as_string())?,
            "attributionSrc" | "attributionsrc" => {
                self.dom
                    .set_attr(node, "attributionsrc", &value.as_string())?
            }
            "download" => self.dom.set_attr(node, "download", &value.as_string())?,
            "hash" => self.set_anchor_url_property(node, "hash", value.clone())?,
            "host" => self.set_anchor_url_property(node, "host", value.clone())?,
            "hostname" => self.set_anchor_url_property(node, "hostname", value.clone())?,
            "href" => self.set_anchor_url_property(node, "href", value.clone())?,
            "hreflang" => self.dom.set_attr(node, "hreflang", &value.as_string())?,
            "interestForElement" => self.dom.set_attr(node, "interestfor", &value.as_string())?,
            "password" => self.set_anchor_url_property(node, "password", value.clone())?,
            "pathname" => self.set_anchor_url_property(node, "pathname", value.clone())?,
            "ping" => self.dom.set_attr(node, "ping", &value.as_string())?,
            "port" => self.set_anchor_url_property(node, "port", value.clone())?,
            "protocol" => self.set_anchor_url_property(node, "protocol", value.clone())?,
            "referrerPolicy" => self
                .dom
                .set_attr(node, "referrerpolicy", &value.as_string())?,
            "rel" => self.dom.set_attr(node, "rel", &value.as_string())?,
            "search" => self.set_anchor_url_property(node, "search", value.clone())?,
            "target" => self.dom.set_attr(node, "target", &value.as_string())?,
            "type" => self.dom.set_attr(node, "type", &value.as_string())?,
            "username" => self.set_anchor_url_property(node, "username", value.clone())?,
            "charset" => self.dom.set_attr(node, "charset", &value.as_string())?,
            "coords" => self.dom.set_attr(node, "coords", &value.as_string())?,
            "rev" => self.dom.set_attr(node, "rev", &value.as_string())?,
            "shape" => self.dom.set_attr(node, "shape", &value.as_string())?,
            _ => {
                self.dom_runtime
                    .node_expando_props
                    .insert((node, key.to_string()), value);
            }
        }
        Ok(())
    }

    pub(crate) fn read_object_assignment_property(
        &self,
        container: &Value,
        key_value: &Value,
        target: &str,
    ) -> Result<Value> {
        let key = self.property_key_to_storage_key(key_value);
        let value = self
            .object_property_from_value(container, &key)
            .map_err(|err| match err {
                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                    Error::ScriptRuntime(format!(
                        "variable '{}' is not an object (key '{}')",
                        target, key
                    ))
                }
                other => other,
            })?;

        if matches!(value, Value::Null | Value::Undefined) {
            let kind = if matches!(value, Value::Null) {
                "null"
            } else {
                "undefined"
            };
            return Err(Error::ScriptRuntime(format!(
                "cannot set property '{}' of {}",
                key, kind
            )));
        }
        Ok(value)
    }

    pub(crate) fn set_object_assignment_property(
        &mut self,
        container: &Value,
        key_value: &Value,
        value: Value,
        target: &str,
    ) -> Result<()> {
        match container {
            Value::Object(object) => {
                let key = self.property_key_to_storage_key(key_value);
                let (is_location, is_history, is_window, is_navigator, is_url, is_storage) = {
                    let entries = object.borrow();
                    (
                        Self::is_location_object(&entries),
                        Self::is_history_object(&entries),
                        Self::is_window_object(&entries),
                        Self::is_navigator_object(&entries),
                        Self::is_url_object(&entries),
                        Self::is_storage_object(&entries),
                    )
                };
                if is_location {
                    self.set_location_property(&key, value)?;
                    return Ok(());
                }
                if is_history {
                    self.set_history_property(&key, value)?;
                    return Ok(());
                }
                if is_window {
                    self.set_window_property(&key, value)?;
                    return Ok(());
                }
                if is_navigator {
                    self.set_navigator_property(object, &key, value)?;
                    return Ok(());
                }
                if is_url {
                    self.set_url_object_property(object, &key, value)?;
                    return Ok(());
                }
                if is_storage {
                    self.set_storage_object_property(object, &key, value)?;
                    return Ok(());
                }
                Self::object_set_entry(&mut object.borrow_mut(), key, value);
                Ok(())
            }
            Value::UrlConstructor => {
                let key = self.property_key_to_storage_key(key_value);
                self.set_url_constructor_property(&key, value);
                Ok(())
            }
            Value::Array(values) => {
                let Some(index) = self.value_as_index(key_value) else {
                    return Ok(());
                };
                let mut values = values.borrow_mut();
                if index >= values.len() {
                    values.resize(index + 1, Value::Undefined);
                }
                values[index] = value;
                Ok(())
            }
            Value::TypedArray(values) => {
                let Some(index) = self.value_as_index(key_value) else {
                    return Ok(());
                };
                self.typed_array_set_index(values, index, value)
            }
            Value::Map(map) => {
                let key = self.property_key_to_storage_key(key_value);
                Self::object_set_entry(&mut map.borrow_mut().properties, key, value);
                Ok(())
            }
            Value::Set(set) => {
                let key = self.property_key_to_storage_key(key_value);
                Self::object_set_entry(&mut set.borrow_mut().properties, key, value);
                Ok(())
            }
            Value::RegExp(regex) => {
                let key = self.property_key_to_storage_key(key_value);
                if key == "lastIndex" {
                    let mut regex = regex.borrow_mut();
                    let next = Self::value_to_i64(&value);
                    regex.last_index = if next <= 0 { 0 } else { next as usize };
                } else {
                    Self::object_set_entry(&mut regex.borrow_mut().properties, key, value);
                }
                Ok(())
            }
            Value::Node(node) => {
                let key = self.property_key_to_storage_key(key_value);
                self.set_node_assignment_property(*node, &key, value)
            }
            _ => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not an object (assignment target)",
                target
            ))),
        }
    }

    pub(crate) fn execute_object_assignment_stmt(
        &mut self,
        target: &str,
        path: &[Expr],
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        if path.is_empty() {
            return Err(Error::ScriptRuntime(
                "object assignment path cannot be empty".into(),
            ));
        }

        let value = self.eval_expr(expr, env, event_param, event)?;

        let mut keys = Vec::with_capacity(path.len());
        for segment in path {
            keys.push(self.eval_expr(segment, env, event_param, event)?);
        }

        let mut container = env
            .get(target)
            .cloned()
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {}", target)))?;
        for key in keys.iter().take(keys.len().saturating_sub(1)) {
            container = self.read_object_assignment_property(&container, key, target)?;
        }

        let final_key = keys
            .last()
            .ok_or_else(|| Error::ScriptRuntime("object assignment key cannot be empty".into()))?;
        self.set_object_assignment_property(&container, final_key, value, target)
    }

}
