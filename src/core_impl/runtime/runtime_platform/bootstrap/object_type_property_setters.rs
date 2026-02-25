use super::*;

impl Harness {
    fn arguments_param_name_for_index(
        env: &HashMap<String, Value>,
        index: usize,
    ) -> Option<String> {
        let Some(Value::Array(bindings)) = env.get(INTERNAL_ARGUMENTS_PARAM_BINDINGS_KEY) else {
            return None;
        };
        match bindings.borrow().get(index) {
            Some(Value::String(name)) => Some(name.clone()),
            _ => None,
        }
    }

    fn arguments_param_indexes_for_name(env: &HashMap<String, Value>, name: &str) -> Vec<usize> {
        let Some(Value::Array(bindings)) = env.get(INTERNAL_ARGUMENTS_PARAM_BINDINGS_KEY) else {
            return Vec::new();
        };
        bindings
            .borrow()
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                matches!(entry, Value::String(param) if param == name).then_some(index)
            })
            .collect()
    }

    pub(crate) fn sync_arguments_after_param_write(
        &mut self,
        env: &mut HashMap<String, Value>,
        name: &str,
        value: &Value,
    ) {
        let indexes = Self::arguments_param_indexes_for_name(env, name);
        if indexes.is_empty() {
            return;
        }
        let Some(Value::Array(arguments)) = env.get("arguments").cloned() else {
            return;
        };
        let mut args_ref = arguments.borrow_mut();
        for index in indexes {
            if index < args_ref.len() {
                args_ref[index] = value.clone();
            }
        }
    }

    fn sync_param_after_arguments_write(
        &mut self,
        env: &mut HashMap<String, Value>,
        arguments_array: &Rc<RefCell<ArrayValue>>,
        index: usize,
        value: &Value,
    ) {
        let Some(Value::Array(arguments)) = env.get("arguments").cloned() else {
            return;
        };
        if !Rc::ptr_eq(&arguments, arguments_array) {
            return;
        }
        let Some(param_name) = Self::arguments_param_name_for_index(env, index) else {
            return;
        };
        env.insert(param_name, value.clone());
    }

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
            | "URL" | "HTMLElement" | "HTMLInputElement" => {
                Err(Error::ScriptRuntime(format!("window.{key} is read-only")))
            }
            "location" => self.set_location_property("href", value),
            "localStorage" => {
                Self::object_set_entry(
                    &mut self.dom_runtime.window_object.borrow_mut(),
                    "localStorage".to_string(),
                    value.clone(),
                );
                self.script_runtime
                    .env
                    .insert("localStorage".to_string(), value);
                Ok(())
            }
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
            self.dom_runtime
                .node_expando_props
                .insert((node, key.to_string()), Value::Function(function));
        } else {
            self.dom_runtime
                .node_expando_props
                .insert((node, key.to_string()), Value::Null);
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

        if key == "text"
            && self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
        {
            self.dom.set_attr(node, "text", &value.as_string())?;
            return Ok(());
        }

        match key {
            "textContent" | "innerText" | "text" => {
                self.dom.set_text_content(node, &value.as_string())?
            }
            "innerHTML" => self.dom.set_inner_html(node, &value.as_string())?,
            "outerHTML" => self.dom.set_outer_html(node, &value.as_string())?,
            "value" => self.dom.set_value(node, &value.as_string())?,
            "files" => {}
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
            "dir" => self.dom.set_attr(node, "dir", &value.as_string())?,
            "accessKey" | "accesskey" => {
                self.dom.set_attr(node, "accesskey", &value.as_string())?
            }
            "autocapitalize" => self
                .dom
                .set_attr(node, "autocapitalize", &value.as_string())?,
            "autocorrect" => self.dom.set_attr(node, "autocorrect", &value.as_string())?,
            "contentEditable" | "contenteditable" => {
                self.dom
                    .set_attr(node, "contenteditable", &value.as_string())?
            }
            "draggable" => self.dom.set_attr(
                node,
                "draggable",
                if value.truthy() { "true" } else { "false" },
            )?,
            "enterKeyHint" | "enterkeyhint" => {
                self.dom
                    .set_attr(node, "enterkeyhint", &value.as_string())?
            }
            "inert" => {
                if value.truthy() {
                    self.dom.set_attr(node, "inert", "true")?;
                } else {
                    self.dom.remove_attr(node, "inert")?;
                }
            }
            "inputMode" | "inputmode" => {
                self.dom.set_attr(node, "inputmode", &value.as_string())?
            }
            "nonce" => self.dom.set_attr(node, "nonce", &value.as_string())?,
            "popover" => self.dom.set_attr(node, "popover", &value.as_string())?,
            "spellcheck" => self.dom.set_attr(
                node,
                "spellcheck",
                if value.truthy() { "true" } else { "false" },
            )?,
            "tabIndex" | "tabindex" => {
                self.dom
                    .set_attr(node, "tabindex", &Self::value_to_i64(&value).to_string())?
            }
            "translate" => {
                self.dom
                    .set_attr(node, "translate", if value.truthy() { "yes" } else { "no" })?
            }
            "cite" => self.dom.set_attr(node, "cite", &value.as_string())?,
            "dateTime" | "datetime" => self.dom.set_attr(node, "datetime", &value.as_string())?,
            "clear" => self.dom.set_attr(node, "clear", &value.as_string())?,
            "align" => self.dom.set_attr(node, "align", &value.as_string())?,
            "aLink" | "alink" => self.dom.set_attr(node, "alink", &value.as_string())?,
            "background" => self.dom.set_attr(node, "background", &value.as_string())?,
            "bgColor" | "bgcolor" => self.dom.set_attr(node, "bgcolor", &value.as_string())?,
            "bottomMargin" | "bottommargin" => {
                self.dom
                    .set_attr(node, "bottommargin", &value.as_string())?
            }
            "leftMargin" | "leftmargin" => {
                self.dom.set_attr(node, "leftmargin", &value.as_string())?
            }
            "link" => self.dom.set_attr(node, "link", &value.as_string())?,
            "rightMargin" | "rightmargin" => {
                self.dom.set_attr(node, "rightmargin", &value.as_string())?
            }
            "topMargin" | "topmargin" => {
                self.dom.set_attr(node, "topmargin", &value.as_string())?
            }
            "vLink" | "vlink" => self.dom.set_attr(node, "vlink", &value.as_string())?,
            "title" => self.dom.set_attr(node, "title", &value.as_string())?,
            "span"
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("col") || tag.eq_ignore_ascii_case("colgroup")
                }) =>
            {
                self.set_col_span_value(node, &value)?
            }
            "src" => self.dom.set_attr(node, "src", &value.as_string())?,
            "autoplay" => {
                if value.truthy() {
                    self.dom.set_attr(node, "autoplay", "true")?;
                } else {
                    self.dom.remove_attr(node, "autoplay")?;
                }
            }
            "controls" => {
                if value.truthy() {
                    self.dom.set_attr(node, "controls", "true")?;
                } else {
                    self.dom.remove_attr(node, "controls")?;
                }
            }
            "controlsList" | "controlslist" => {
                self.dom
                    .set_attr(node, "controlslist", &value.as_string())?
            }
            "crossOrigin" | "crossorigin" => {
                self.dom.set_attr(node, "crossorigin", &value.as_string())?
            }
            "disableRemotePlayback" | "disableremoteplayback" => {
                if value.truthy() {
                    self.dom.set_attr(node, "disableremoteplayback", "true")?;
                } else {
                    self.dom.remove_attr(node, "disableremoteplayback")?;
                }
            }
            "disablePictureInPicture" | "disablepictureinpicture" => {
                if value.truthy() {
                    self.dom.set_attr(node, "disablepictureinpicture", "true")?;
                } else {
                    self.dom.remove_attr(node, "disablepictureinpicture")?;
                }
            }
            "loop" => {
                if value.truthy() {
                    self.dom.set_attr(node, "loop", "true")?;
                } else {
                    self.dom.remove_attr(node, "loop")?;
                }
            }
            "muted" => {
                if value.truthy() {
                    self.dom.set_attr(node, "muted", "true")?;
                } else {
                    self.dom.remove_attr(node, "muted")?;
                }
            }
            "preload" => self.dom.set_attr(node, "preload", &value.as_string())?,
            "playsInline" | "playsinline" => {
                if value.truthy() {
                    self.dom.set_attr(node, "playsinline", "true")?;
                } else {
                    self.dom.remove_attr(node, "playsinline")?;
                }
            }
            "poster" => self.dom.set_attr(node, "poster", &value.as_string())?,
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
            "kind"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("track")) =>
            {
                self.dom.set_attr(node, "kind", &value.as_string())?
            }
            "srclang" | "srcLang"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("track")) =>
            {
                self.dom.set_attr(node, "srclang", &value.as_string())?
            }
            "label"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("track")) =>
            {
                self.dom.set_attr(node, "label", &value.as_string())?
            }
            "default"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("track")) =>
            {
                if value.truthy() {
                    self.dom.set_attr(node, "default", "true")?;
                } else {
                    self.dom.remove_attr(node, "default")?;
                }
            }
            "media" => self.dom.set_attr(node, "media", &value.as_string())?,
            "sizes" => self.dom.set_attr(node, "sizes", &value.as_string())?,
            "srcset" | "srcSet" => self.dom.set_attr(node, "srcset", &value.as_string())?,
            "width" => self.set_canvas_dimension_value(node, "width", &value)?,
            "height" => self.set_canvas_dimension_value(node, "height", &value)?,
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
        &mut self,
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
        env: &mut HashMap<String, Value>,
        event: &EventState,
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
                let (own_setter, own_getter, own_data, mut prototype) = {
                    let entries = object.borrow();
                    (
                        Self::object_setter_from_entries(&entries, &key),
                        Self::object_getter_from_entries(&entries, &key).is_some(),
                        Self::object_get_entry(&entries, &key).is_some(),
                        Self::object_get_entry(&entries, INTERNAL_OBJECT_PROTOTYPE_KEY),
                    )
                };
                if let Some(setter) = own_setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        event,
                        None,
                        Some(container.clone()),
                    )?;
                    return Ok(());
                }
                if own_getter {
                    return Ok(());
                }
                if !own_data {
                    while let Some(Value::Object(proto)) = prototype {
                        let (setter, getter, next) = {
                            let proto_ref = proto.borrow();
                            (
                                Self::object_setter_from_entries(&proto_ref, &key),
                                Self::object_getter_from_entries(&proto_ref, &key).is_some(),
                                Self::object_get_entry(&proto_ref, INTERNAL_OBJECT_PROTOTYPE_KEY),
                            )
                        };
                        if let Some(setter) = setter {
                            if !self.is_callable_value(&setter) {
                                return Err(Error::ScriptRuntime(
                                    "object setter is not callable".into(),
                                ));
                            }
                            self.execute_callable_value_with_this_and_env(
                                &setter,
                                &[value],
                                event,
                                None,
                                Some(container.clone()),
                            )?;
                            return Ok(());
                        }
                        if getter {
                            return Ok(());
                        }
                        prototype = next;
                    }
                }
                Self::object_set_entry(&mut object.borrow_mut(), key, value);
                Ok(())
            }
            Value::Function(function) => {
                let key = self.property_key_to_storage_key(key_value);
                let (own_setter, own_getter, own_data) = {
                    if let Some(entries) = self
                        .script_runtime
                        .function_public_properties
                        .get(&function.function_id)
                    {
                        (
                            Self::object_setter_from_entries(entries, &key),
                            Self::object_getter_from_entries(entries, &key).is_some(),
                            Self::object_get_entry(entries, &key).is_some(),
                        )
                    } else {
                        (None, false, false)
                    }
                };
                if let Some(setter) = own_setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        event,
                        None,
                        Some(container.clone()),
                    )?;
                    return Ok(());
                }
                if own_getter {
                    return Ok(());
                }
                if !own_data {
                    let mut prototype = function.class_super_constructor.clone();
                    while let Some(current) = prototype {
                        match current {
                            Value::Function(proto_function) => {
                                let (setter, getter, next) = if let Some(entries) = self
                                    .script_runtime
                                    .function_public_properties
                                    .get(&proto_function.function_id)
                                {
                                    (
                                        Self::object_setter_from_entries(entries, &key),
                                        Self::object_getter_from_entries(entries, &key).is_some(),
                                        proto_function.class_super_constructor.clone(),
                                    )
                                } else {
                                    (None, false, proto_function.class_super_constructor.clone())
                                };
                                if let Some(setter) = setter {
                                    if !self.is_callable_value(&setter) {
                                        return Err(Error::ScriptRuntime(
                                            "object setter is not callable".into(),
                                        ));
                                    }
                                    self.execute_callable_value_with_this_and_env(
                                        &setter,
                                        &[value],
                                        event,
                                        None,
                                        Some(container.clone()),
                                    )?;
                                    return Ok(());
                                }
                                if getter {
                                    return Ok(());
                                }
                                prototype = next;
                            }
                            Value::Object(proto_object) => {
                                let (setter, getter, next) = {
                                    let proto_ref = proto_object.borrow();
                                    (
                                        Self::object_setter_from_entries(&proto_ref, &key),
                                        Self::object_getter_from_entries(&proto_ref, &key)
                                            .is_some(),
                                        Self::object_get_entry(
                                            &proto_ref,
                                            INTERNAL_OBJECT_PROTOTYPE_KEY,
                                        ),
                                    )
                                };
                                if let Some(setter) = setter {
                                    if !self.is_callable_value(&setter) {
                                        return Err(Error::ScriptRuntime(
                                            "object setter is not callable".into(),
                                        ));
                                    }
                                    self.execute_callable_value_with_this_and_env(
                                        &setter,
                                        &[value],
                                        event,
                                        None,
                                        Some(container.clone()),
                                    )?;
                                    return Ok(());
                                }
                                if getter {
                                    return Ok(());
                                }
                                prototype = next;
                            }
                            _ => break,
                        }
                    }
                }
                let entries = self
                    .script_runtime
                    .function_public_properties
                    .entry(function.function_id)
                    .or_default();
                Self::object_set_entry(entries, key, value);
                Ok(())
            }
            Value::UrlConstructor => {
                let key = self.property_key_to_storage_key(key_value);
                self.set_url_constructor_property(&key, value);
                Ok(())
            }
            Value::Array(array_values) => {
                if let Some(index) = self.value_as_index(key_value) {
                    let value_for_sync = value.clone();
                    {
                        let mut elements = array_values.borrow_mut();
                        if index >= elements.len() {
                            elements.resize(index + 1, Value::Undefined);
                        }
                        elements[index] = value;
                    }
                    Self::clear_array_hole(array_values, index);
                    self.sync_param_after_arguments_write(
                        env,
                        array_values,
                        index,
                        &value_for_sync,
                    );
                    return Ok(());
                }
                let key = self.property_key_to_storage_key(key_value);
                if key == "length" {
                    let mut values = array_values.borrow_mut();
                    let next = Self::value_to_i64(&value);
                    let next = if next <= 0 { 0usize } else { next as usize };
                    if next < values.len() {
                        values.truncate(next);
                    } else if next > values.len() {
                        values.resize(next, Value::Undefined);
                    }
                    return Ok(());
                }
                Self::set_array_property(array_values, key, value);
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
            Value::WeakMap(weak_map) => {
                let key = self.property_key_to_storage_key(key_value);
                Self::object_set_entry(&mut weak_map.borrow_mut().properties, key, value);
                Ok(())
            }
            Value::Set(set) => {
                let key = self.property_key_to_storage_key(key_value);
                Self::object_set_entry(&mut set.borrow_mut().properties, key, value);
                Ok(())
            }
            Value::WeakSet(weak_set) => {
                let key = self.property_key_to_storage_key(key_value);
                Self::object_set_entry(&mut weak_set.borrow_mut().properties, key, value);
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

    fn set_super_assignment_property(
        &mut self,
        super_base: &Value,
        receiver: &Value,
        key_value: &Value,
        value: Value,
        target: &str,
        env: &mut HashMap<String, Value>,
        event: &EventState,
    ) -> Result<()> {
        let key = self.property_key_to_storage_key(key_value);
        let mut current = Some(super_base.clone());
        while let Some(container) = current {
            match container {
                Value::Object(object) => {
                    let (setter, getter, next) = {
                        let entries = object.borrow();
                        (
                            Self::object_setter_from_entries(&entries, &key),
                            Self::object_getter_from_entries(&entries, &key).is_some(),
                            Self::object_get_entry(&entries, INTERNAL_OBJECT_PROTOTYPE_KEY),
                        )
                    };
                    if let Some(setter) = setter {
                        if !self.is_callable_value(&setter) {
                            return Err(Error::ScriptRuntime("object setter is not callable".into()));
                        }
                        self.execute_callable_value_with_this_and_env(
                            &setter,
                            &[value],
                            event,
                            None,
                            Some(receiver.clone()),
                        )?;
                        return Ok(());
                    }
                    if getter {
                        return Ok(());
                    }
                    current = next;
                }
                Value::Function(function) => {
                    let (setter, getter, next) = {
                        if let Some(entries) = self
                            .script_runtime
                            .function_public_properties
                            .get(&function.function_id)
                        {
                            (
                                Self::object_setter_from_entries(entries, &key),
                                Self::object_getter_from_entries(entries, &key).is_some(),
                                function.class_super_constructor.clone(),
                            )
                        } else {
                            (None, false, function.class_super_constructor.clone())
                        }
                    };
                    if let Some(setter) = setter {
                        if !self.is_callable_value(&setter) {
                            return Err(Error::ScriptRuntime("object setter is not callable".into()));
                        }
                        self.execute_callable_value_with_this_and_env(
                            &setter,
                            &[value],
                            event,
                            None,
                            Some(receiver.clone()),
                        )?;
                        return Ok(());
                    }
                    if getter {
                        return Ok(());
                    }
                    current = next;
                }
                _ => break,
            }
        }

        self.set_object_assignment_property(receiver, key_value, value, target, env, event)
    }

    pub(crate) fn execute_object_assignment_stmt(
        &mut self,
        target: &str,
        path: &[Expr],
        op: VarAssignOp,
        expr: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        if path.is_empty() {
            return Err(Error::ScriptRuntime(
                "object assignment path cannot be empty".into(),
            ));
        }

        let mut keys = Vec::with_capacity(path.len());
        for segment in path {
            keys.push(self.eval_expr(segment, env, event_param, event)?);
        }

        if target == "super" {
            let super_base = Self::super_prototype_from_env(env)?;
            let this_value = Self::super_this_from_env(env)?;

            let final_key = keys.last().ok_or_else(|| {
                Error::ScriptRuntime("object assignment key cannot be empty".into())
            })?;
            let key = self.property_key_to_storage_key(final_key);

            let mut container = super_base.clone();
            for (index, key_value) in keys.iter().take(keys.len().saturating_sub(1)).enumerate() {
                if index == 0 {
                    container = self.object_property_from_value_with_receiver(
                        &container,
                        &self.property_key_to_storage_key(key_value),
                        &this_value,
                    )?;
                } else {
                    container = self.read_object_assignment_property(&container, key_value, target)?;
                }
            }

            if matches!(
                op,
                VarAssignOp::LogicalAnd | VarAssignOp::LogicalOr | VarAssignOp::Nullish
            ) {
                let previous = if keys.len() <= 1 {
                    self.object_property_from_value_with_receiver(&super_base, &key, &this_value)?
                } else {
                    self.object_property_from_value(&container, &key)?
                };
                let should_assign = match op {
                    VarAssignOp::LogicalAnd => previous.truthy(),
                    VarAssignOp::LogicalOr => !previous.truthy(),
                    VarAssignOp::Nullish => matches!(&previous, Value::Null | Value::Undefined),
                    _ => true,
                };
                if !should_assign {
                    return Ok(());
                }
            }

            let value = self.eval_expr(expr, env, event_param, event)?;
            if keys.len() <= 1 {
                self.set_super_assignment_property(
                    &super_base,
                    &this_value,
                    final_key,
                    value,
                    target,
                    env,
                    event,
                )?;
            } else {
                self.set_object_assignment_property(
                    &container,
                    final_key,
                    value,
                    target,
                    env,
                    event,
                )?;
            }
            return Ok(());
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
        let key = self.property_key_to_storage_key(final_key);

        if matches!(
            op,
            VarAssignOp::LogicalAnd | VarAssignOp::LogicalOr | VarAssignOp::Nullish
        ) {
            let previous = self
                .object_property_from_value(&container, &key)
                .map_err(|err| match err {
                    Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                        Error::ScriptRuntime(format!(
                            "variable '{}' is not an object (key '{}')",
                            target, key
                        ))
                    }
                    other => other,
                })?;
            let should_assign = match op {
                VarAssignOp::LogicalAnd => previous.truthy(),
                VarAssignOp::LogicalOr => !previous.truthy(),
                VarAssignOp::Nullish => matches!(&previous, Value::Null | Value::Undefined),
                _ => true,
            };
            if !should_assign {
                return Ok(());
            }
        }

        let value = self.eval_expr(expr, env, event_param, event)?;

        let assigns_window_local_storage = if let Value::Object(object) = &container {
            if key == "localStorage" {
                let entries = object.borrow();
                Self::is_window_object(&entries)
            } else {
                false
            }
        } else {
            false
        };

        self.set_object_assignment_property(
            &container,
            final_key,
            value.clone(),
            target,
            env,
            event,
        )?;
        if assigns_window_local_storage {
            env.insert("localStorage".to_string(), value.clone());
            self.sync_global_binding_if_needed(env, "localStorage", &value);
        }
        Ok(())
    }
}
