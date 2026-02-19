impl Harness {
    pub fn from_html(html: &str) -> Result<Self> {
        stacker::grow(32 * 1024 * 1024, || Self::from_html_impl(html))
    }

    fn from_html_impl(html: &str) -> Result<Self> {
        let ParseOutput { dom, scripts } = parse_html(html)?;
        let mut harness = Self {
            dom,
            listeners: ListenerStore::default(),
            node_event_handler_props: HashMap::new(),
            script_env: HashMap::new(),
            document_url: "about:blank".to_string(),
            window_object: Rc::new(RefCell::new(Vec::new())),
            document_object: Rc::new(RefCell::new(Vec::new())),
            location_object: Rc::new(RefCell::new(Vec::new())),
            history_object: Rc::new(RefCell::new(Vec::new())),
            history_entries: vec![HistoryEntry {
                url: "about:blank".to_string(),
                state: Value::Null,
            }],
            history_index: 0,
            history_scroll_restoration: "auto".to_string(),
            location_mock_pages: HashMap::new(),
            location_navigations: Vec::new(),
            location_reload_count: 0,
            task_queue: Vec::new(),
            microtask_queue: VecDeque::new(),
            dialog_return_values: HashMap::new(),
            active_element: None,
            now_ms: 0,
            timer_step_limit: 10_000,
            next_timer_id: 1,
            next_task_order: 0,
            next_promise_id: 1,
            next_symbol_id: 1,
            next_url_object_id: 1,
            url_objects: HashMap::new(),
            next_blob_url_id: 1,
            blob_url_objects: HashMap::new(),
            task_depth: 0,
            running_timer_id: None,
            running_timer_canceled: false,
            rng_state: 0x9E37_79B9_7F4A_7C15,
            clipboard_text: String::new(),
            fetch_mocks: HashMap::new(),
            fetch_calls: Vec::new(),
            match_media_mocks: HashMap::new(),
            match_media_calls: Vec::new(),
            default_match_media_matches: false,
            alert_messages: Vec::new(),
            confirm_responses: VecDeque::new(),
            default_confirm_response: false,
            prompt_responses: VecDeque::new(),
            default_prompt_response: None,
            symbol_registry: HashMap::new(),
            symbols_by_id: HashMap::new(),
            well_known_symbols: HashMap::new(),
            trace: false,
            trace_events: true,
            trace_timers: true,
            trace_logs: Vec::new(),
            trace_log_limit: 10_000,
            trace_to_stderr: true,
            pending_function_decls: Vec::new(),
        };

        harness.initialize_global_bindings();

        for script in scripts {
            harness.compile_and_register_script(&script)?;
        }

        Ok(harness)
    }

    pub(super) fn initialize_global_bindings(&mut self) {
        self.sync_location_object();
        self.sync_history_object();
        self.window_object = Rc::new(RefCell::new(Vec::new()));
        self.document_object = Rc::new(RefCell::new(Vec::new()));
        let clipboard = Self::new_object_value(vec![
            (INTERNAL_CLIPBOARD_OBJECT_KEY.into(), Value::Bool(true)),
            ("readText".into(), Self::new_builtin_placeholder_function()),
            ("writeText".into(), Self::new_builtin_placeholder_function()),
        ]);
        let location = Value::Object(self.location_object.clone());
        let history = Value::Object(self.history_object.clone());

        let navigator = Self::new_object_value(vec![
            (INTERNAL_NAVIGATOR_OBJECT_KEY.into(), Value::Bool(true)),
            ("language".into(), Value::String(DEFAULT_LOCALE.to_string())),
            (
                "languages".into(),
                Self::new_array_value(vec![
                    Value::String(DEFAULT_LOCALE.to_string()),
                    Value::String("en".to_string()),
                ]),
            ),
            ("clipboard".into(), clipboard),
        ]);

        let mut intl_entries = vec![
            ("Collator".into(), Self::new_builtin_placeholder_function()),
            (
                "DateTimeFormat".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "DisplayNames".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "DurationFormat".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "ListFormat".into(),
                Self::new_builtin_placeholder_function(),
            ),
            ("Locale".into(), Self::new_builtin_placeholder_function()),
            (
                "NumberFormat".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "PluralRules".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "RelativeTimeFormat".into(),
                Self::new_builtin_placeholder_function(),
            ),
            ("Segmenter".into(), Self::new_builtin_placeholder_function()),
            (
                "getCanonicalLocales".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "supportedValuesOf".into(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        let to_string_tag = self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag);
        Self::object_set_entry(
            &mut intl_entries,
            to_string_tag_key,
            Value::String("Intl".to_string()),
        );
        let intl = Self::new_object_value(intl_entries);
        let string_constructor = Value::StringConstructor;
        let boolean_constructor = Self::new_boolean_constructor_callable();
        let url_constructor = Value::UrlConstructor;
        let html_element_constructor = Self::new_builtin_placeholder_function();
        let html_input_element_constructor = Self::new_builtin_placeholder_function();

        self.sync_document_object();
        self.sync_window_object(
            &navigator,
            &intl,
            &string_constructor,
            &boolean_constructor,
            &url_constructor,
            &html_element_constructor,
            &html_input_element_constructor,
        );

        let window = Value::Object(self.window_object.clone());
        let document = Value::Object(self.document_object.clone());

        self.script_env.insert("document".to_string(), document);
        self.script_env
            .insert("navigator".to_string(), navigator.clone());
        self.script_env
            .insert("clientInformation".to_string(), navigator.clone());
        self.script_env.insert("Intl".to_string(), intl);
        self.script_env
            .insert("String".to_string(), string_constructor);
        self.script_env
            .insert("Boolean".to_string(), boolean_constructor);
        self.script_env.insert("URL".to_string(), url_constructor);
        self.script_env
            .insert("HTMLElement".to_string(), html_element_constructor);
        self.script_env.insert(
            "HTMLInputElement".to_string(),
            html_input_element_constructor,
        );
        self.script_env.insert("location".to_string(), location);
        self.script_env.insert("history".to_string(), history);
        self.script_env.insert("window".to_string(), window.clone());
        self.script_env.insert("self".to_string(), window.clone());
        self.script_env.insert("top".to_string(), window.clone());
        self.script_env.insert("parent".to_string(), window.clone());
        self.script_env.insert("frames".to_string(), window);
        self.script_env
            .insert(INTERNAL_SCOPE_DEPTH_KEY.to_string(), Value::Number(0));
    }

    pub(super) fn current_location_parts(&self) -> LocationParts {
        LocationParts::parse(&self.document_url).unwrap_or_else(|| LocationParts {
            scheme: "about".to_string(),
            has_authority: false,
            username: String::new(),
            password: String::new(),
            hostname: String::new(),
            port: String::new(),
            pathname: String::new(),
            opaque_path: "blank".to_string(),
            search: String::new(),
            hash: String::new(),
        })
    }

    pub(super) fn window_is_secure_context(&self) -> bool {
        matches!(
            self.current_location_parts().scheme.as_str(),
            "https" | "wss"
        )
    }

    pub(super) fn document_builtin_keys() -> &'static [&'static str] {
        &["defaultView", "location", "URL", "documentURI"]
    }

    pub(super) fn sync_document_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.document_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
                    continue;
                }
                if Self::document_builtin_keys()
                    .iter()
                    .any(|builtin| builtin == key)
                {
                    continue;
                }
                extras.push((key.clone(), value.clone()));
            }
        }

        let mut entries = vec![
            (INTERNAL_DOCUMENT_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                "defaultView".to_string(),
                Value::Object(self.window_object.clone()),
            ),
            (
                "location".to_string(),
                Value::Object(self.location_object.clone()),
            ),
            ("URL".to_string(), Value::String(self.document_url.clone())),
            (
                "documentURI".to_string(),
                Value::String(self.document_url.clone()),
            ),
        ];
        entries.extend(extras);
        *self.document_object.borrow_mut() = entries;
    }

    pub(super) fn window_builtin_keys() -> &'static [&'static str] {
        &[
            "window",
            "self",
            "top",
            "parent",
            "frames",
            "length",
            "closed",
            "location",
            "history",
            "navigator",
            "clientInformation",
            "document",
            "origin",
            "isSecureContext",
            "Intl",
            "String",
            "Boolean",
            "URL",
            "HTMLElement",
            "HTMLInputElement",
            "name",
        ]
    }

    pub(super) fn sync_window_object(
        &mut self,
        navigator: &Value,
        intl: &Value,
        string_constructor: &Value,
        boolean_constructor: &Value,
        url_constructor: &Value,
        html_element_constructor: &Value,
        html_input_element_constructor: &Value,
    ) {
        let mut extras = Vec::new();
        let mut name_value = Value::String(String::new());
        {
            let entries = self.window_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
                    continue;
                }
                if key == "name" {
                    name_value = Value::String(value.as_string());
                    continue;
                }
                if Self::window_builtin_keys()
                    .iter()
                    .any(|builtin| builtin == key)
                {
                    continue;
                }
                extras.push((key.clone(), value.clone()));
            }
        }

        let window_ref = Value::Object(self.window_object.clone());
        let mut entries = vec![
            (INTERNAL_WINDOW_OBJECT_KEY.to_string(), Value::Bool(true)),
            ("window".to_string(), window_ref.clone()),
            ("self".to_string(), window_ref.clone()),
            ("top".to_string(), window_ref.clone()),
            ("parent".to_string(), window_ref.clone()),
            ("frames".to_string(), window_ref),
            ("length".to_string(), Value::Number(0)),
            ("closed".to_string(), Value::Bool(false)),
            (
                "location".to_string(),
                Value::Object(self.location_object.clone()),
            ),
            (
                "history".to_string(),
                Value::Object(self.history_object.clone()),
            ),
            ("navigator".to_string(), navigator.clone()),
            ("clientInformation".to_string(), navigator.clone()),
            (
                "document".to_string(),
                Value::Object(self.document_object.clone()),
            ),
            (
                "origin".to_string(),
                Value::String(self.current_location_parts().origin()),
            ),
            (
                "isSecureContext".to_string(),
                Value::Bool(self.window_is_secure_context()),
            ),
            ("Intl".to_string(), intl.clone()),
            ("String".to_string(), string_constructor.clone()),
            ("Boolean".to_string(), boolean_constructor.clone()),
            ("URL".to_string(), url_constructor.clone()),
            (
                "HTMLElement".to_string(),
                html_element_constructor.clone(),
            ),
            (
                "HTMLInputElement".to_string(),
                html_input_element_constructor.clone(),
            ),
            ("name".to_string(), name_value),
        ];
        entries.extend(extras);
        *self.window_object.borrow_mut() = entries;
    }

    pub(super) fn sync_window_runtime_properties(&mut self) {
        let mut entries = self.window_object.borrow_mut();
        Self::object_set_entry(
            &mut entries,
            "origin".to_string(),
            Value::String(self.current_location_parts().origin()),
        );
        Self::object_set_entry(
            &mut entries,
            "isSecureContext".to_string(),
            Value::Bool(self.window_is_secure_context()),
        );
    }

    pub(super) fn location_builtin_keys() -> &'static [&'static str] {
        &[
            "href",
            "protocol",
            "host",
            "hostname",
            "port",
            "pathname",
            "search",
            "hash",
            "origin",
            "ancestorOrigins",
            "assign",
            "reload",
            "replace",
            "toString",
        ]
    }

    pub(super) fn sync_location_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.location_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
                    continue;
                }
                if Self::location_builtin_keys()
                    .iter()
                    .any(|builtin| builtin == key)
                {
                    continue;
                }
                extras.push((key.clone(), value.clone()));
            }
        }

        let parts = self.current_location_parts();
        let mut entries = vec![
            (INTERNAL_LOCATION_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
                Value::String(parts.href()),
            ),
            ("href".to_string(), Value::String(parts.href())),
            ("protocol".to_string(), Value::String(parts.protocol())),
            ("host".to_string(), Value::String(parts.host())),
            (
                "hostname".to_string(),
                Value::String(parts.hostname.clone()),
            ),
            ("port".to_string(), Value::String(parts.port.clone())),
            (
                "pathname".to_string(),
                Value::String(if parts.has_authority {
                    parts.pathname.clone()
                } else {
                    parts.opaque_path.clone()
                }),
            ),
            ("search".to_string(), Value::String(parts.search.clone())),
            ("hash".to_string(), Value::String(parts.hash.clone())),
            ("origin".to_string(), Value::String(parts.origin())),
            (
                "ancestorOrigins".to_string(),
                Self::new_array_value(Vec::new()),
            ),
            (
                "assign".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "reload".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "replace".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "toString".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        entries.extend(extras);
        *self.location_object.borrow_mut() = entries;
    }

    pub(super) fn is_location_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_LOCATION_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(super) fn history_builtin_keys() -> &'static [&'static str] {
        &[
            "length",
            "scrollRestoration",
            "state",
            "back",
            "forward",
            "go",
            "pushState",
            "replaceState",
        ]
    }

    pub(super) fn current_history_state(&self) -> Value {
        self.history_entries
            .get(self.history_index)
            .map(|entry| entry.state.clone())
            .unwrap_or(Value::Null)
    }

    pub(super) fn sync_history_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.history_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
                    continue;
                }
                if Self::history_builtin_keys()
                    .iter()
                    .any(|builtin| builtin == key)
                {
                    continue;
                }
                extras.push((key.clone(), value.clone()));
            }
        }

        let mut entries = vec![
            (INTERNAL_HISTORY_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                "length".to_string(),
                Value::Number(self.history_entries.len() as i64),
            ),
            (
                "scrollRestoration".to_string(),
                Value::String(self.history_scroll_restoration.clone()),
            ),
            ("state".to_string(), self.current_history_state()),
            ("back".to_string(), Self::new_builtin_placeholder_function()),
            (
                "forward".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("go".to_string(), Self::new_builtin_placeholder_function()),
            (
                "pushState".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "replaceState".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        entries.extend(extras);
        *self.history_object.borrow_mut() = entries;
    }

    pub(super) fn is_history_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_HISTORY_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(super) fn is_window_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_WINDOW_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(super) fn is_navigator_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_NAVIGATOR_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(super) fn set_navigator_property(
        &mut self,
        navigator_object: &Rc<RefCell<Vec<(String, Value)>>>,
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

    pub(super) fn set_window_property(&mut self, key: &str, value: Value) -> Result<()> {
        match key {
            "window" | "self" | "top" | "parent" | "frames" | "length" | "closed" | "history"
            | "navigator" | "clientInformation" | "document" | "origin" | "isSecureContext"
            | "URL" | "HTMLElement" | "HTMLInputElement" => {
                Err(Error::ScriptRuntime(format!("window.{key} is read-only")))
            }
            "location" => self.set_location_property("href", value),
            "name" => {
                Self::object_set_entry(
                    &mut self.window_object.borrow_mut(),
                    "name".to_string(),
                    Value::String(value.as_string()),
                );
                Ok(())
            }
            _ => {
                Self::object_set_entry(
                    &mut self.window_object.borrow_mut(),
                    key.to_string(),
                    value,
                );
                Ok(())
            }
        }
    }

    pub(super) fn set_history_property(&mut self, key: &str, value: Value) -> Result<()> {
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
                self.history_scroll_restoration = mode;
                self.sync_history_object();
                self.sync_window_runtime_properties();
                Ok(())
            }
            _ => {
                Self::object_set_entry(
                    &mut self.history_object.borrow_mut(),
                    key.to_string(),
                    value,
                );
                Ok(())
            }
        }
    }

    pub(super) fn set_node_event_handler_property(
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
                },
            );
            self.node_event_handler_props
                .insert((node, event_type), handler);
        }
        Ok(true)
    }

    pub(super) fn set_node_assignment_property(
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
            "elementTiming" => self.dom.set_attr(node, "elementtiming", &value.as_string())?,
            "name" => self.dom.set_attr(node, "name", &value.as_string())?,
            "lang" => self.dom.set_attr(node, "lang", &value.as_string())?,
            "title" => self.dom.set_document_title(&value.as_string())?,
            "attributionSrc" | "attributionsrc" => {
                self.dom.set_attr(node, "attributionsrc", &value.as_string())?
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
            "referrerPolicy" => self.dom.set_attr(node, "referrerpolicy", &value.as_string())?,
            "rel" => self.dom.set_attr(node, "rel", &value.as_string())?,
            "search" => self.set_anchor_url_property(node, "search", value.clone())?,
            "target" => self.dom.set_attr(node, "target", &value.as_string())?,
            "type" => self.dom.set_attr(node, "type", &value.as_string())?,
            "username" => self.set_anchor_url_property(node, "username", value.clone())?,
            "charset" => self.dom.set_attr(node, "charset", &value.as_string())?,
            "coords" => self.dom.set_attr(node, "coords", &value.as_string())?,
            "rev" => self.dom.set_attr(node, "rev", &value.as_string())?,
            "shape" => self.dom.set_attr(node, "shape", &value.as_string())?,
            _ => {}
        }
        Ok(())
    }

    pub(super) fn read_object_assignment_property(
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

    pub(super) fn set_object_assignment_property(
        &mut self,
        container: &Value,
        key_value: &Value,
        value: Value,
        target: &str,
    ) -> Result<()> {
        match container {
            Value::Object(object) => {
                let key = self.property_key_to_storage_key(key_value);
                let (is_location, is_history, is_window, is_navigator, is_url) = {
                    let entries = object.borrow();
                    (
                        Self::is_location_object(&entries),
                        Self::is_history_object(&entries),
                        Self::is_window_object(&entries),
                        Self::is_navigator_object(&entries),
                        Self::is_url_object(&entries),
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
                Self::object_set_entry(&mut object.borrow_mut(), key, value);
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

    pub(super) fn execute_object_assignment_stmt(
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

        let final_key = keys.last().ok_or_else(|| {
            Error::ScriptRuntime("object assignment key cannot be empty".into())
        })?;
        self.set_object_assignment_property(&container, final_key, value, target)
    }

    pub(super) fn resolve_location_target_url(&self, input: &str) -> String {
        let input = input.trim();
        if input.is_empty() {
            return self.document_url.clone();
        }

        if let Some(parts) = LocationParts::parse(input) {
            return parts.href();
        }

        let base = self.current_location_parts();
        if input.starts_with("//") {
            return LocationParts::parse(&format!("{}{}", base.protocol(), input))
                .map(|parts| parts.href())
                .unwrap_or_else(|| input.to_string());
        }

        let mut next = base.clone();
        if input.starts_with('#') {
            next.hash = ensure_hash_prefix(input);
            return next.href();
        }

        if input.starts_with('?') {
            next.search = ensure_search_prefix(input);
            next.hash.clear();
            return next.href();
        }

        if input.starts_with('/') {
            if next.has_authority {
                next.pathname = normalize_pathname(input);
            } else {
                next.opaque_path = input.to_string();
            }
            next.search.clear();
            next.hash.clear();
            return next.href();
        }

        let mut relative = input;
        let mut next_search = String::new();
        let mut next_hash = String::new();
        if let Some(hash_pos) = relative.find('#') {
            next_hash = ensure_hash_prefix(&relative[hash_pos + 1..]);
            relative = &relative[..hash_pos];
        }
        if let Some(search_pos) = relative.find('?') {
            next_search = ensure_search_prefix(&relative[search_pos + 1..]);
            relative = &relative[..search_pos];
        }

        if next.has_authority {
            let base_dir = if let Some((prefix, _)) = next.pathname.rsplit_once('/') {
                if prefix.is_empty() {
                    "/".to_string()
                } else {
                    format!("{prefix}/")
                }
            } else {
                "/".to_string()
            };
            next.pathname = normalize_pathname(&format!("{base_dir}{relative}"));
        } else {
            next.opaque_path = relative.to_string();
        }
        next.search = next_search;
        next.hash = next_hash;
        next.href()
    }

    pub(super) fn is_hash_only_navigation(from: &str, to: &str) -> bool {
        let Some(from_parts) = LocationParts::parse(from) else {
            return false;
        };
        let Some(to_parts) = LocationParts::parse(to) else {
            return false;
        };
        from_parts.scheme == to_parts.scheme
            && from_parts.has_authority == to_parts.has_authority
            && from_parts.username == to_parts.username
            && from_parts.password == to_parts.password
            && from_parts.hostname == to_parts.hostname
            && from_parts.port == to_parts.port
            && from_parts.pathname == to_parts.pathname
            && from_parts.opaque_path == to_parts.opaque_path
            && from_parts.search == to_parts.search
            && from_parts.hash != to_parts.hash
    }

    pub(super) fn navigate_location(
        &mut self,
        next_url: &str,
        kind: LocationNavigationKind,
    ) -> Result<()> {
        let from = self.document_url.clone();
        let to = self.resolve_location_target_url(next_url);
        self.document_url = to.clone();
        match kind {
            LocationNavigationKind::Replace => {
                self.history_replace_current_entry(&to, Value::Null);
            }
            LocationNavigationKind::Assign | LocationNavigationKind::HrefSet => {
                self.history_push_entry(&to, Value::Null);
            }
            LocationNavigationKind::Reload => {}
        }
        self.sync_location_object();
        self.sync_history_object();
        self.sync_document_object();
        self.sync_window_runtime_properties();
        self.location_navigations.push(LocationNavigation {
            kind,
            from,
            to: to.clone(),
        });

        if !Self::is_hash_only_navigation(
            &self
                .location_navigations
                .last()
                .map(|nav| nav.from.clone())
                .unwrap_or_default(),
            &to,
        ) {
            let _ = self.load_location_mock_page_if_exists(&to)?;
        }
        Ok(())
    }

    pub(super) fn reload_location(&mut self) -> Result<()> {
        self.location_reload_count += 1;
        let current = self.document_url.clone();
        self.location_navigations.push(LocationNavigation {
            kind: LocationNavigationKind::Reload,
            from: current.clone(),
            to: current.clone(),
        });
        self.sync_location_object();
        self.sync_history_object();
        self.sync_document_object();
        self.sync_window_runtime_properties();
        let _ = self.load_location_mock_page_if_exists(&current)?;
        Ok(())
    }

    pub(super) fn load_location_mock_page_if_exists(&mut self, url: &str) -> Result<bool> {
        let Some(html) = self.location_mock_pages.get(url).cloned() else {
            return Ok(false);
        };
        self.replace_document_with_html(&html)?;
        Ok(true)
    }

    pub(super) fn history_push_entry(&mut self, url: &str, state: Value) {
        let next = self
            .history_index
            .saturating_add(1)
            .min(self.history_entries.len());
        self.history_entries.truncate(next);
        self.history_entries.push(HistoryEntry {
            url: url.to_string(),
            state,
        });
        self.history_index = self.history_entries.len().saturating_sub(1);
    }

    pub(super) fn history_replace_current_entry(&mut self, url: &str, state: Value) {
        if self.history_entries.is_empty() {
            self.history_entries.push(HistoryEntry {
                url: url.to_string(),
                state,
            });
            self.history_index = 0;
            return;
        }
        let index = self
            .history_index
            .min(self.history_entries.len().saturating_sub(1));
        self.history_entries[index] = HistoryEntry {
            url: url.to_string(),
            state,
        };
        self.history_index = index;
    }

    pub(super) fn history_push_state(
        &mut self,
        state: Value,
        url: Option<&str>,
        replace: bool,
    ) -> Result<()> {
        let cloned = Self::structured_clone_value(&state, &mut Vec::new(), &mut Vec::new())?;
        let target_url = url.unwrap_or(&self.document_url);
        let next_url = self.resolve_location_target_url(target_url);
        self.document_url = next_url.clone();

        if replace {
            self.history_replace_current_entry(&next_url, cloned);
        } else {
            self.history_push_entry(&next_url, cloned);
        }
        self.sync_location_object();
        self.sync_history_object();
        self.sync_document_object();
        self.sync_window_runtime_properties();
        Ok(())
    }

    pub(super) fn history_go_with_env(&mut self, delta: i64) -> Result<()> {
        if delta == 0 {
            self.reload_location()?;
            return Ok(());
        }

        let current = self.history_index as i64;
        let target = current.saturating_add(delta);
        if target < 0 || target >= self.history_entries.len() as i64 {
            return Ok(());
        }
        let target = target as usize;
        if target == self.history_index {
            return Ok(());
        }

        let from = self.document_url.clone();
        self.history_index = target;
        let entry = self
            .history_entries
            .get(target)
            .cloned()
            .unwrap_or(HistoryEntry {
                url: self.document_url.clone(),
                state: Value::Null,
            });
        self.document_url = entry.url.clone();
        self.sync_location_object();
        self.sync_history_object();
        self.sync_document_object();
        self.sync_window_runtime_properties();

        if !Self::is_hash_only_navigation(&from, &entry.url) {
            let _ = self.load_location_mock_page_if_exists(&entry.url)?;
        }

        let mut pop_env = self.script_env.clone();
        let _ = self.dispatch_event_with_options(
            self.dom.root,
            "popstate",
            &mut pop_env,
            true,
            false,
            false,
            Some(entry.state),
            None,
            None,
        )?;
        self.script_env = pop_env;
        Ok(())
    }

    pub(super) fn replace_document_with_html(&mut self, html: &str) -> Result<()> {
        let ParseOutput { dom, scripts } = parse_html(html)?;
        self.dom = dom;
        self.listeners = ListenerStore::default();
        self.node_event_handler_props.clear();
        self.script_env.clear();
        self.task_queue.clear();
        self.microtask_queue.clear();
        self.active_element = None;
        self.running_timer_id = None;
        self.running_timer_canceled = false;
        self.dom.set_active_element(None);
        self.dom.set_active_pseudo_element(None);
        self.initialize_global_bindings();
        for script in scripts {
            self.compile_and_register_script(&script)?;
        }
        Ok(())
    }

    pub(super) fn set_location_property(&mut self, key: &str, value: Value) -> Result<()> {
        match key {
            "href" => self.navigate_location(&value.as_string(), LocationNavigationKind::HrefSet),
            "protocol" => {
                let mut parts = self.current_location_parts();
                let protocol = value.as_string();
                let protocol = protocol.trim_end_matches(':').to_ascii_lowercase();
                if !is_valid_url_scheme(&protocol) {
                    return Err(Error::ScriptRuntime(format!(
                        "invalid location.protocol value: {}",
                        value.as_string()
                    )));
                }
                parts.scheme = protocol;
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "host" => {
                let mut parts = self.current_location_parts();
                let host = value.as_string();
                let (hostname, port) = split_hostname_and_port(host.trim());
                parts.hostname = hostname;
                parts.port = port;
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "hostname" => {
                let mut parts = self.current_location_parts();
                parts.hostname = value.as_string();
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "port" => {
                let mut parts = self.current_location_parts();
                parts.port = value.as_string();
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "pathname" => {
                let mut parts = self.current_location_parts();
                let raw = value.as_string();
                if parts.has_authority {
                    let normalized_input = if raw.starts_with('/') {
                        raw
                    } else {
                        format!("/{raw}")
                    };
                    parts.pathname = normalize_pathname(&normalized_input);
                } else {
                    parts.opaque_path = raw;
                }
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "search" => {
                let mut parts = self.current_location_parts();
                parts.search = ensure_search_prefix(&value.as_string());
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "hash" => {
                let mut parts = self.current_location_parts();
                parts.hash = ensure_hash_prefix(&value.as_string());
                self.navigate_location(&parts.href(), LocationNavigationKind::HrefSet)
            }
            "origin" | "ancestorOrigins" => {
                Err(Error::ScriptRuntime(format!("location.{key} is read-only")))
            }
            _ => {
                Self::object_set_entry(
                    &mut self.location_object.borrow_mut(),
                    key.to_string(),
                    value,
                );
                Ok(())
            }
        }
    }

    pub(super) fn anchor_rel_tokens(&self, node: NodeId) -> Vec<String> {
        self.dom
            .attr(node, "rel")
            .unwrap_or_default()
            .split_whitespace()
            .map(|token| token.to_string())
            .collect::<Vec<_>>()
    }

    pub(super) fn resolve_anchor_href(&self, node: NodeId) -> String {
        let raw = self.dom.attr(node, "href").unwrap_or_default();
        self.resolve_location_target_url(&raw)
    }

    pub(super) fn anchor_location_parts(&self, node: NodeId) -> LocationParts {
        let href = self.resolve_anchor_href(node);
        LocationParts::parse(&href).unwrap_or_else(|| self.current_location_parts())
    }

    pub(super) fn set_anchor_url_property(
        &mut self,
        node: NodeId,
        key: &str,
        value: Value,
    ) -> Result<()> {
        match key {
            "href" => {
                self.dom.set_attr(node, "href", &value.as_string())?;
                return Ok(());
            }
            "origin" | "relList" => {
                return Err(Error::ScriptRuntime(format!("anchor.{key} is read-only")));
            }
            _ => {}
        }

        let mut parts = self.anchor_location_parts(node);
        match key {
            "protocol" => {
                let protocol = value.as_string();
                let protocol = protocol.trim_end_matches(':').to_ascii_lowercase();
                if !is_valid_url_scheme(&protocol) {
                    return Err(Error::ScriptRuntime(format!(
                        "invalid anchor.protocol value: {}",
                        value.as_string()
                    )));
                }
                parts.scheme = protocol;
            }
            "host" => {
                let host = value.as_string();
                let (hostname, port) = split_hostname_and_port(host.trim());
                parts.hostname = hostname;
                parts.port = port;
            }
            "hostname" => {
                parts.hostname = value.as_string();
            }
            "port" => {
                parts.port = value.as_string();
            }
            "pathname" => {
                let raw = value.as_string();
                if parts.has_authority {
                    let normalized_input = if raw.starts_with('/') {
                        raw
                    } else {
                        format!("/{raw}")
                    };
                    parts.pathname = normalize_pathname(&normalized_input);
                } else {
                    parts.opaque_path = raw;
                }
            }
            "search" => {
                parts.search = ensure_search_prefix(&value.as_string());
            }
            "hash" => {
                parts.hash = ensure_hash_prefix(&value.as_string());
            }
            "username" => {
                parts.username = value.as_string();
            }
            "password" => {
                parts.password = value.as_string();
            }
            _ => {
                return Err(Error::ScriptRuntime(format!(
                    "unsupported anchor URL property: {key}"
                )));
            }
        }

        self.dom.set_attr(node, "href", &parts.href())
    }

    pub fn enable_trace(&mut self, enabled: bool) {
        self.trace = enabled;
    }

    pub fn take_trace_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.trace_logs)
    }

    pub fn set_trace_stderr(&mut self, enabled: bool) {
        self.trace_to_stderr = enabled;
    }

    pub fn set_trace_events(&mut self, enabled: bool) {
        self.trace_events = enabled;
    }

    pub fn set_trace_timers(&mut self, enabled: bool) {
        self.trace_timers = enabled;
    }

    pub fn set_trace_log_limit(&mut self, max_entries: usize) -> Result<()> {
        if max_entries == 0 {
            return Err(Error::ScriptRuntime(
                "set_trace_log_limit requires at least 1 entry".into(),
            ));
        }
        self.trace_log_limit = max_entries;
        while self.trace_logs.len() > self.trace_log_limit {
            self.trace_logs.remove(0);
        }
        Ok(())
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.rng_state = if seed == 0 {
            0xA5A5_A5A5_A5A5_A5A5
        } else {
            seed
        };
    }

    pub fn set_fetch_mock(&mut self, url: &str, body: &str) {
        self.fetch_mocks.insert(url.to_string(), body.to_string());
    }

    pub fn set_clipboard_text(&mut self, text: &str) {
        self.clipboard_text = text.to_string();
    }

    pub fn clipboard_text(&self) -> String {
        self.clipboard_text.clone()
    }

    pub fn set_location_mock_page(&mut self, url: &str, html: &str) {
        let normalized = self.resolve_location_target_url(url);
        self.location_mock_pages
            .insert(normalized, html.to_string());
    }

    pub fn clear_location_mock_pages(&mut self) {
        self.location_mock_pages.clear();
    }

    pub fn take_location_navigations(&mut self) -> Vec<LocationNavigation> {
        std::mem::take(&mut self.location_navigations)
    }

    pub fn location_reload_count(&self) -> usize {
        self.location_reload_count
    }

    pub fn clear_fetch_mocks(&mut self) {
        self.fetch_mocks.clear();
    }

    pub fn take_fetch_calls(&mut self) -> Vec<String> {
        std::mem::take(&mut self.fetch_calls)
    }

    pub fn set_match_media_mock(&mut self, query: &str, matches: bool) {
        self.match_media_mocks.insert(query.to_string(), matches);
    }

    pub fn clear_match_media_mocks(&mut self) {
        self.match_media_mocks.clear();
    }

    pub fn set_default_match_media_matches(&mut self, matches: bool) {
        self.default_match_media_matches = matches;
    }

    pub fn take_match_media_calls(&mut self) -> Vec<String> {
        std::mem::take(&mut self.match_media_calls)
    }

    pub fn enqueue_confirm_response(&mut self, accepted: bool) {
        self.confirm_responses.push_back(accepted);
    }

    pub fn set_default_confirm_response(&mut self, accepted: bool) {
        self.default_confirm_response = accepted;
    }

    pub fn enqueue_prompt_response(&mut self, value: Option<&str>) {
        self.prompt_responses
            .push_back(value.map(std::string::ToString::to_string));
    }

    pub fn set_default_prompt_response(&mut self, value: Option<&str>) {
        self.default_prompt_response = value.map(std::string::ToString::to_string);
    }

    pub fn take_alert_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.alert_messages)
    }

    pub fn set_timer_step_limit(&mut self, max_steps: usize) -> Result<()> {
        if max_steps == 0 {
            return Err(Error::ScriptRuntime(
                "set_timer_step_limit requires at least 1 step".into(),
            ));
        }
        self.timer_step_limit = max_steps;
        Ok(())
    }

    fn input_supports_required(kind: &str) -> bool {
        !matches!(
            kind,
            "hidden" | "range" | "color" | "button" | "submit" | "reset" | "image"
        )
    }

    fn is_labelable_control(&self, node: NodeId) -> bool {
        let Some(tag) = self.dom.tag_name(node) else {
            return false;
        };

        if tag.eq_ignore_ascii_case("input") {
            let input_type = self
                .dom
                .attr(node, "type")
                .unwrap_or_else(|| "text".to_string())
                .to_ascii_lowercase();
            return input_type != "hidden";
        }

        tag.eq_ignore_ascii_case("button")
            || tag.eq_ignore_ascii_case("select")
            || tag.eq_ignore_ascii_case("textarea")
    }

    fn resolve_label_control(&self, label: NodeId) -> Option<NodeId> {
        if !self
            .dom
            .tag_name(label)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("label"))
        {
            return None;
        }

        if let Some(target_id) = self.dom.attr(label, "for") {
            if let Some(target) = self.dom.by_id(&target_id) {
                if self.is_labelable_control(target) {
                    return Some(target);
                }
            }
        }

        let mut descendants = Vec::new();
        self.dom.collect_elements_descendants_dfs(label, &mut descendants);
        descendants
            .into_iter()
            .find(|candidate| self.is_labelable_control(*candidate))
    }

    pub(super) fn is_effectively_disabled(&self, node: NodeId) -> bool {
        if self.dom.disabled(node) {
            return true;
        }
        if !is_form_control(&self.dom, node) {
            return false;
        }

        let mut cursor = self.dom.parent(node);
        while let Some(parent) = cursor {
            if self
                .dom
                .tag_name(parent)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("fieldset"))
                && self.dom.disabled(parent)
            {
                return true;
            }
            cursor = self.dom.parent(parent);
        }

        false
    }

    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        if self.is_effectively_disabled(target) {
            return Ok(());
        }
        if self.dom.readonly(target) {
            return Ok(());
        }

        let tag = self
            .dom
            .tag_name(target)
            .ok_or_else(|| Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input or textarea".into(),
                actual: "non-element".into(),
            })?
            .to_ascii_lowercase();

        if tag != "input" && tag != "textarea" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input or textarea".into(),
                actual: tag,
            });
        }

        self.dom.set_value(target, text)?;
        self.dispatch_event(target, "input")?;
        Ok(())
    }

    pub fn set_checked(&mut self, selector: &str, checked: bool) -> Result<()> {
        let target = self.select_one(selector)?;
        if self.is_effectively_disabled(target) {
            return Ok(());
        }
        let tag = self
            .dom
            .tag_name(target)
            .unwrap_or_default()
            .to_ascii_lowercase();
        if tag != "input" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input[type=checkbox|radio]".into(),
                actual: tag,
            });
        }

        let kind = self
            .dom
            .attr(target, "type")
            .unwrap_or_else(|| "text".into())
            .to_ascii_lowercase();
        if kind != "checkbox" && kind != "radio" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input[type=checkbox|radio]".into(),
                actual: format!("input[type={kind}]"),
            });
        }

        let current = self.dom.checked(target)?;
        if current != checked {
            self.dom.set_checked(target, checked)?;
            self.dispatch_event(target, "input")?;
            self.dispatch_event(target, "change")?;
        }

        Ok(())
    }

    pub fn click(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        self.click_node(target)
    }

    pub(super) fn click_node_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.is_effectively_disabled(target) {
            return Ok(());
        }

        self.dom.set_active_pseudo_element(Some(target));
        let result: Result<()> = (|| {
            let click_outcome = self.dispatch_event_with_env(target, "click", env, true)?;
            if click_outcome.default_prevented {
                return Ok(());
            }

            if let Some(control) = self.resolve_label_control(target) {
                if control != target {
                    self.click_node_with_env(control, env)?;
                    return Ok(());
                }
            }

            if is_checkbox_input(&self.dom, target) {
                let current = self.dom.checked(target)?;
                self.dom.set_indeterminate(target, false)?;
                self.dom.set_checked(target, !current)?;
                self.dispatch_event_with_env(target, "input", env, true)?;
                self.dispatch_event_with_env(target, "change", env, true)?;
            }

            if is_radio_input(&self.dom, target) {
                let current = self.dom.checked(target)?;
                if !current {
                    self.dom.set_checked(target, true)?;
                    self.dispatch_event_with_env(target, "input", env, true)?;
                    self.dispatch_event_with_env(target, "change", env, true)?;
                }
            }

            if is_submit_control(&self.dom, target) {
                if let Some(form_id) = self.resolve_form_for_submit(target) {
                    if !self.form_is_valid_for_submit(form_id)? {
                        return Ok(());
                    }
                    let submit_outcome =
                        self.dispatch_event_with_env(form_id, "submit", env, true)?;
                    if !submit_outcome.default_prevented {
                        self.maybe_close_dialog_for_form_submit_with_env(form_id, env)?;
                    }
                }
            }

            Ok(())
        })();
        self.dom.set_active_pseudo_element(None);
        result
    }

    pub(super) fn click_node(&mut self, target: NodeId) -> Result<()> {
        let mut env = self.script_env.clone();
        let result = stacker::grow(32 * 1024 * 1024, || self.click_node_with_env(target, &mut env));
        self.script_env = env;
        result
    }

    pub fn focus(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        self.focus_node(target)
    }

    pub fn blur(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        self.blur_node(target)
    }

    pub fn submit(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;

        let form = if self
            .dom
            .tag_name(target)
            .map(|t| t.eq_ignore_ascii_case("form"))
            .unwrap_or(false)
        {
            Some(target)
        } else {
            self.resolve_form_for_submit(target)
        };

        if let Some(form_id) = form {
            let submit_outcome = self.dispatch_event(form_id, "submit")?;
            if !submit_outcome.default_prevented {
                let mut env = self.script_env.clone();
                self.maybe_close_dialog_for_form_submit_with_env(form_id, &mut env)?;
                self.script_env = env;
            }
        }

        Ok(())
    }

    pub(super) fn submit_form_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let form = if self
            .dom
            .tag_name(target)
            .map(|t| t.eq_ignore_ascii_case("form"))
            .unwrap_or(false)
        {
            Some(target)
        } else {
            self.resolve_form_for_submit(target)
        };

        if let Some(form_id) = form {
            let submit_outcome = self.dispatch_event_with_env(form_id, "submit", env, true)?;
            if !submit_outcome.default_prevented {
                self.maybe_close_dialog_for_form_submit_with_env(form_id, env)?;
            }
        }

        Ok(())
    }

    pub(super) fn maybe_close_dialog_for_form_submit_with_env(
        &mut self,
        form: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let Some(method) = self.dom.attr(form, "method") else {
            return Ok(());
        };
        if !method.eq_ignore_ascii_case("dialog") {
            return Ok(());
        }
        let Some(dialog) = self.dom.find_ancestor_by_tag(form, "dialog") else {
            return Ok(());
        };
        let _ = self.transition_dialog_open_state_with_env(dialog, false, true, env)?;
        Ok(())
    }

    pub(super) fn reset_form_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let Some(form_id) = self.resolve_form_for_submit(target) else {
            return Ok(());
        };

        let outcome = self.dispatch_event_with_env(form_id, "reset", env, true)?;
        if outcome.default_prevented {
            return Ok(());
        }

        let controls = self.form_elements(form_id)?;
        for control in controls {
            if is_checkbox_input(&self.dom, control) || is_radio_input(&self.dom, control) {
                let default_checked = self.dom.attr(control, "checked").is_some();
                self.dom.set_checked(control, default_checked)?;
                self.dom.set_indeterminate(control, false)?;
                continue;
            }

            if self
                .dom
                .tag_name(control)
                .map(|tag| tag.eq_ignore_ascii_case("select"))
                .unwrap_or(false)
            {
                self.dom.sync_select_value(control)?;
                continue;
            }

            let default_value = self.dom.attr(control, "value").unwrap_or_default();
            self.dom.set_value(control, &default_value)?;
        }

        Ok(())
    }

    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        let mut env = self.script_env.clone();
        let _ = self.dispatch_event_with_env(target, event, &mut env, false)?;
        self.script_env = env;
        Ok(())
    }

    pub fn now_ms(&self) -> i64 {
        self.now_ms
    }

    pub fn clear_timer(&mut self, timer_id: i64) -> bool {
        let existed = self.running_timer_id == Some(timer_id)
            || self.task_queue.iter().any(|task| task.id == timer_id);
        self.clear_timeout(timer_id);
        existed
    }

    pub fn clear_all_timers(&mut self) -> usize {
        let cleared = self.task_queue.len();
        self.task_queue.clear();
        if self.running_timer_id.is_some() {
            self.running_timer_canceled = true;
        }
        self.trace_timer_line(format!("[timer] clear_all cleared={cleared}"));
        cleared
    }

    pub fn pending_timers(&self) -> Vec<PendingTimer> {
        let mut timers = self
            .task_queue
            .iter()
            .map(|task| PendingTimer {
                id: task.id,
                due_at: task.due_at,
                order: task.order,
                interval_ms: task.interval_ms,
            })
            .collect::<Vec<_>>();
        timers.sort_by_key(|timer| (timer.due_at, timer.order));
        timers
    }

    pub fn advance_time(&mut self, delta_ms: i64) -> Result<()> {
        if delta_ms < 0 {
            return Err(Error::ScriptRuntime(
                "advance_time requires non-negative milliseconds".into(),
            ));
        }
        let from = self.now_ms;
        self.now_ms = self.now_ms.saturating_add(delta_ms);
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] advance delta_ms={} from={} to={} ran_due={}",
            delta_ms, from, self.now_ms, ran
        ));
        Ok(())
    }

    pub fn advance_time_to(&mut self, target_ms: i64) -> Result<()> {
        if target_ms < self.now_ms {
            return Err(Error::ScriptRuntime(format!(
                "advance_time_to requires target >= now_ms (target={target_ms}, now_ms={})",
                self.now_ms
            )));
        }
        let from = self.now_ms;
        self.now_ms = target_ms;
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] advance_to from={} to={} ran_due={}",
            from, self.now_ms, ran
        ));
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        let from = self.now_ms;
        let ran = self.run_timer_queue(None, true)?;
        self.trace_timer_line(format!(
            "[timer] flush from={} to={} ran={}",
            from, self.now_ms, ran
        ));
        Ok(())
    }

    pub fn run_next_timer(&mut self) -> Result<bool> {
        let Some(next_idx) = self.next_task_index(None) else {
            self.trace_timer_line("[timer] run_next none".into());
            return Ok(false);
        };

        let task = self.task_queue.remove(next_idx);
        if task.due_at > self.now_ms {
            self.now_ms = task.due_at;
        }
        self.execute_timer_task(task)?;
        Ok(true)
    }

    pub fn run_next_due_timer(&mut self) -> Result<bool> {
        let Some(next_idx) = self.next_task_index(Some(self.now_ms)) else {
            self.trace_timer_line("[timer] run_next_due none".into());
            return Ok(false);
        };

        let task = self.task_queue.remove(next_idx);
        self.execute_timer_task(task)?;
        Ok(true)
    }

    pub fn run_due_timers(&mut self) -> Result<usize> {
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] run_due now_ms={} ran={}",
            self.now_ms, ran
        ));
        Ok(ran)
    }

    pub(super) fn run_due_timers_internal(&mut self) -> Result<usize> {
        self.run_timer_queue(Some(self.now_ms), false)
    }

    pub(super) fn run_timer_queue(
        &mut self,
        due_limit: Option<i64>,
        advance_clock: bool,
    ) -> Result<usize> {
        let mut steps = 0usize;
        while let Some(next_idx) = self.next_task_index(due_limit) {
            steps += 1;
            if steps > self.timer_step_limit {
                return Err(self.timer_step_limit_error(self.timer_step_limit, steps, due_limit));
            }
            let task = self.task_queue.remove(next_idx);
            if advance_clock && task.due_at > self.now_ms {
                self.now_ms = task.due_at;
            }
            self.execute_timer_task(task)?;
        }
        Ok(steps)
    }

    pub(super) fn timer_step_limit_error(
        &self,
        max_steps: usize,
        steps: usize,
        due_limit: Option<i64>,
    ) -> Error {
        let due_limit_desc = due_limit
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".into());

        let next_task_desc = self
            .next_task_index(due_limit)
            .and_then(|idx| self.task_queue.get(idx))
            .map(|task| {
                let interval_desc = task
                    .interval_ms
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".into());
                format!(
                    "id={},due_at={},order={},interval_ms={}",
                    task.id, task.due_at, task.order, interval_desc
                )
            })
            .unwrap_or_else(|| "none".into());

        Error::ScriptRuntime(format!(
            "flush exceeded max task steps (possible uncleared setInterval): limit={max_steps}, steps={steps}, now_ms={}, due_limit={}, pending_tasks={}, next_task={}",
            self.now_ms,
            due_limit_desc,
            self.task_queue.len(),
            next_task_desc
        ))
    }

    pub(super) fn next_task_index(&self, due_limit: Option<i64>) -> Option<usize> {
        self.task_queue
            .iter()
            .enumerate()
            .filter(|(_, task)| {
                if let Some(limit) = due_limit {
                    task.due_at <= limit
                } else {
                    true
                }
            })
            .min_by_key(|(_, task)| (task.due_at, task.order))
            .map(|(idx, _)| idx)
    }

    pub(super) fn execute_timer_task(&mut self, mut task: ScheduledTask) -> Result<()> {
        let interval_desc = task
            .interval_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".into());
        self.trace_timer_line(format!(
            "[timer] run id={} due_at={} interval_ms={} now_ms={}",
            task.id, task.due_at, interval_desc, self.now_ms
        ));

        self.running_timer_id = Some(task.id);
        self.running_timer_canceled = false;
        let mut event = EventState::new("timeout", self.dom.root, self.now_ms);
        self.run_in_task_context(|this| {
            this.execute_timer_task_callback(
                &task.callback,
                &task.callback_args,
                &mut event,
                &mut task.env,
            )
            .map(|_| ())
        })?;
        let canceled = self.running_timer_canceled;
        self.running_timer_id = None;
        self.running_timer_canceled = false;

        if let Some(interval_ms) = task.interval_ms {
            if !canceled {
                let delay_ms = interval_ms.max(0);
                let due_at = task.due_at.saturating_add(delay_ms);
                let order = self.next_task_order;
                self.next_task_order += 1;
                self.task_queue.push(ScheduledTask {
                    id: task.id,
                    due_at,
                    order,
                    interval_ms: Some(delay_ms),
                    callback: task.callback,
                    callback_args: task.callback_args,
                    env: task.env,
                });
                self.trace_timer_line(format!(
                    "[timer] requeue id={} due_at={} interval_ms={}",
                    task.id, due_at, delay_ms
                ));
            }
        }

        Ok(())
    }

    pub fn assert_text(&self, selector: &str, expected: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.text_content(target);
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual,
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_value(&self, selector: &str, expected: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.value(target)?;
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual,
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_checked(&self, selector: &str, expected: bool) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.checked(target)?;
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual: actual.to_string(),
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_exists(&self, selector: &str) -> Result<()> {
        let _ = self.select_one(selector)?;
        Ok(())
    }

    pub fn dump_dom(&self, selector: &str) -> Result<String> {
        let target = self.select_one(selector)?;
        Ok(self.dom.dump_node(target))
    }

    pub(super) fn select_one(&self, selector: &str) -> Result<NodeId> {
        self.dom
            .query_selector(selector)?
            .ok_or_else(|| Error::SelectorNotFound(selector.to_string()))
    }

    pub(super) fn node_snippet(&self, node_id: NodeId) -> String {
        truncate_chars(&self.dom.dump_node(node_id), 200)
    }

    pub(super) fn resolve_form_for_submit(&self, target: NodeId) -> Option<NodeId> {
        if self
            .dom
            .tag_name(target)
            .map(|t| t.eq_ignore_ascii_case("form"))
            .unwrap_or(false)
        {
            return Some(target);
        }
        self.dom.find_ancestor_by_tag(target, "form")
    }

    pub(super) fn form_elements(&self, form: NodeId) -> Result<Vec<NodeId>> {
        let tag = self
            .dom
            .tag_name(form)
            .ok_or_else(|| Error::ScriptRuntime("elements target is not an element".into()))?;
        if !tag.eq_ignore_ascii_case("form") {
            return Err(Error::ScriptRuntime(format!(
                "{}.elements target is not a form",
                self.event_node_label(form)
            )));
        }

        let mut out = Vec::new();
        self.collect_form_controls(form, &mut out);
        Ok(out)
    }

    pub(super) fn form_data_entries(&self, form: NodeId) -> Result<Vec<(String, String)>> {
        let mut out = Vec::new();
        for control in self.form_elements(form)? {
            if !self.is_successful_form_data_control(control)? {
                continue;
            }
            let name = self.dom.attr(control, "name").unwrap_or_default();
            let value = self.form_data_control_value(control)?;
            out.push((name, value));
        }
        Ok(out)
    }

    pub(super) fn is_successful_form_data_control(&self, control: NodeId) -> Result<bool> {
        if self.is_effectively_disabled(control) {
            return Ok(false);
        }
        let name = self.dom.attr(control, "name").unwrap_or_default();
        if name.is_empty() {
            return Ok(false);
        }

        let tag = self
            .dom
            .tag_name(control)
            .ok_or_else(|| Error::ScriptRuntime("FormData target is not an element".into()))?;

        if tag.eq_ignore_ascii_case("button") {
            return Ok(false);
        }

        if tag.eq_ignore_ascii_case("input") {
            let kind = self
                .dom
                .attr(control, "type")
                .unwrap_or_default()
                .to_ascii_lowercase();
            if matches!(
                kind.as_str(),
                "button" | "submit" | "reset" | "file" | "image"
            ) {
                return Ok(false);
            }
            if kind == "checkbox" || kind == "radio" {
                return self.dom.checked(control);
            }
        }

        Ok(true)
    }

    pub(super) fn form_data_control_value(&self, control: NodeId) -> Result<String> {
        self.dom.value(control)
    }

    pub(super) fn form_is_valid_for_submit(&self, form: NodeId) -> Result<bool> {
        let controls = self.form_elements(form)?;
        for control in &controls {
            if !self.required_control_satisfied(*control, &controls)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn required_control_satisfied(
        &self,
        control: NodeId,
        controls: &[NodeId],
    ) -> Result<bool> {
        if self.is_effectively_disabled(control) || !self.dom.required(control) {
            return Ok(true);
        }

        let tag = self
            .dom
            .tag_name(control)
            .ok_or_else(|| Error::ScriptRuntime("required target is not an element".into()))?;

        if tag.eq_ignore_ascii_case("input") {
            let kind = self
                .dom
                .attr(control, "type")
                .unwrap_or_else(|| "text".into())
                .to_ascii_lowercase();
            if !Self::input_supports_required(kind.as_str()) {
                return Ok(true);
            }
            if kind == "checkbox" {
                return self.dom.checked(control);
            }
            if kind == "radio" {
                if self.dom.checked(control)? {
                    return Ok(true);
                }
                let name = self.dom.attr(control, "name").unwrap_or_default();
                if name.is_empty() {
                    return Ok(false);
                }
                for candidate in controls {
                    if *candidate == control {
                        continue;
                    }
                    if !is_radio_input(&self.dom, *candidate) {
                        continue;
                    }
                    if self.dom.attr(*candidate, "name").unwrap_or_default() != name {
                        continue;
                    }
                    if self.dom.checked(*candidate)? {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            return Ok(!self.dom.value(control)?.is_empty());
        }

        if tag.eq_ignore_ascii_case("select") || tag.eq_ignore_ascii_case("textarea") {
            return Ok(!self.dom.value(control)?.is_empty());
        }

        Ok(true)
    }

    pub(super) fn eval_form_data_source(
        &mut self,
        source: &FormDataSource,
        env: &HashMap<String, Value>,
    ) -> Result<Vec<(String, String)>> {
        match source {
            FormDataSource::NewForm(form) => {
                let form_node = self.resolve_dom_query_required_runtime(form, env)?;
                self.form_data_entries(form_node)
            }
            FormDataSource::Var(name) => match env.get(name) {
                Some(Value::FormData(entries)) => Ok(entries.clone()),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a FormData instance",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown FormData variable: {}",
                    name
                ))),
            },
        }
    }

    pub(super) fn collect_form_controls(&self, node: NodeId, out: &mut Vec<NodeId>) {
        for child in &self.dom.nodes[node.0].children {
            if is_form_control(&self.dom, *child) {
                out.push(*child);
            }
            self.collect_form_controls(*child, out);
        }
    }

    pub(super) fn dispatch_event(
        &mut self,
        target: NodeId,
        event_type: &str,
    ) -> Result<EventState> {
        let mut env = self.script_env.clone();
        let event = self.dispatch_event_with_env(target, event_type, &mut env, true)?;
        self.script_env = env;
        Ok(event)
    }

    pub(super) fn dispatch_event_with_env(
        &mut self,
        target: NodeId,
        event_type: &str,
        env: &mut HashMap<String, Value>,
        trusted: bool,
    ) -> Result<EventState> {
        let event = if trusted {
            EventState::new(event_type, target, self.now_ms)
        } else {
            EventState::new_untrusted(event_type, target, self.now_ms)
        };
        self.dispatch_prepared_event_with_env(event, env)
    }

    pub(super) fn dispatch_event_with_options(
        &mut self,
        target: NodeId,
        event_type: &str,
        env: &mut HashMap<String, Value>,
        trusted: bool,
        bubbles: bool,
        cancelable: bool,
        state: Option<Value>,
        old_state: Option<&str>,
        new_state: Option<&str>,
    ) -> Result<EventState> {
        let mut event = if trusted {
            EventState::new(event_type, target, self.now_ms)
        } else {
            EventState::new_untrusted(event_type, target, self.now_ms)
        };
        event.bubbles = bubbles;
        event.cancelable = cancelable;
        event.state = state;
        event.old_state = old_state.map(str::to_string);
        event.new_state = new_state.map(str::to_string);
        self.dispatch_prepared_event_with_env(event, env)
    }

    pub(super) fn dispatch_prepared_event_with_env(
        &mut self,
        mut event: EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<EventState> {
        let target = event.target;
        self.run_in_task_context(|this| {
            let mut path = Vec::new();
            let mut cursor = Some(target);
            while let Some(node) = cursor {
                path.push(node);
                cursor = this.dom.parent(node);
            }
            path.reverse();

            if path.is_empty() {
                this.trace_event_done(&event, "empty_path");
                return Ok(());
            }

            // Capture phase.
            if path.len() >= 2 {
                for node in &path[..path.len() - 1] {
                    event.event_phase = 1;
                    event.current_target = *node;
                    this.invoke_listeners(*node, &mut event, env, true)?;
                    if event.propagation_stopped {
                        this.trace_event_done(&event, "propagation_stopped");
                        return Ok(());
                    }
                }
            }

            // Target phase: capture listeners first.
            event.event_phase = 2;
            event.current_target = target;
            this.invoke_listeners(target, &mut event, env, true)?;
            if event.propagation_stopped {
                this.trace_event_done(&event, "propagation_stopped");
                return Ok(());
            }

            // Target phase: bubble listeners.
            event.event_phase = 2;
            this.invoke_listeners(target, &mut event, env, false)?;
            if event.propagation_stopped {
                this.trace_event_done(&event, "propagation_stopped");
                return Ok(());
            }

            // Bubble phase.
            if path.len() >= 2 {
                for node in path[..path.len() - 1].iter().rev() {
                    event.event_phase = 3;
                    event.current_target = *node;
                    this.invoke_listeners(*node, &mut event, env, false)?;
                    if event.propagation_stopped {
                        this.trace_event_done(&event, "propagation_stopped");
                        return Ok(());
                    }
                }
            }

            this.trace_event_done(&event, "completed");
            Ok(())
        })?;
        Ok(event)
    }

    pub(super) fn focus_node(&mut self, node: NodeId) -> Result<()> {
        let mut env = self.script_env.clone();
        self.focus_node_with_env(node, &mut env)?;
        self.script_env = env;
        Ok(())
    }

    pub(super) fn focus_node_with_env(
        &mut self,
        node: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.is_effectively_disabled(node) {
            return Ok(());
        }

        if self.active_element == Some(node) {
            return Ok(());
        }

        if let Some(current) = self.active_element {
            self.blur_node_with_env(current, env)?;
        }

        self.active_element = Some(node);
        self.dom.set_active_element(Some(node));
        self.dispatch_event_with_env(node, "focusin", env, true)?;
        self.dispatch_event_with_env(node, "focus", env, true)?;
        Ok(())
    }

    pub(super) fn blur_node(&mut self, node: NodeId) -> Result<()> {
        let mut env = self.script_env.clone();
        self.blur_node_with_env(node, &mut env)?;
        self.script_env = env;
        Ok(())
    }

    pub(super) fn blur_node_with_env(
        &mut self,
        node: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.active_element != Some(node) {
            return Ok(());
        }

        self.dispatch_event_with_env(node, "focusout", env, true)?;
        self.dispatch_event_with_env(node, "blur", env, true)?;
        self.active_element = None;
        self.dom.set_active_element(None);
        Ok(())
    }

    pub(super) fn scroll_into_view_node_with_env(
        &mut self,
        _node: NodeId,
        _env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        Ok(())
    }

    pub(super) fn ensure_dialog_target(&self, node: NodeId, operation: &str) -> Result<()> {
        let tag = self
            .dom
            .tag_name(node)
            .ok_or_else(|| Error::ScriptRuntime(format!("{operation} target is not an element")))?;
        if tag.eq_ignore_ascii_case("dialog") {
            return Ok(());
        }
        Err(Error::ScriptRuntime(format!(
            "{operation} target is not a <dialog> element"
        )))
    }

    pub(super) fn dialog_return_value(&self, dialog: NodeId) -> Result<String> {
        self.ensure_dialog_target(dialog, "returnValue")?;
        Ok(self
            .dialog_return_values
            .get(&dialog)
            .cloned()
            .unwrap_or_default())
    }

    pub(super) fn set_dialog_return_value(&mut self, dialog: NodeId, value: String) -> Result<()> {
        self.ensure_dialog_target(dialog, "returnValue")?;
        self.dialog_return_values.insert(dialog, value);
        Ok(())
    }

    pub(super) fn show_dialog_with_env(
        &mut self,
        dialog: NodeId,
        _modal: bool,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        self.ensure_dialog_target(dialog, "show/showModal")?;
        let _ = self.transition_dialog_open_state_with_env(dialog, true, false, env)?;
        Ok(())
    }

    pub(super) fn close_dialog_with_env(
        &mut self,
        dialog: NodeId,
        return_value: Option<Value>,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        self.ensure_dialog_target(dialog, "close()")?;
        if let Some(return_value) = return_value {
            self.set_dialog_return_value(dialog, return_value.as_string())?;
        }
        let _ = self.transition_dialog_open_state_with_env(dialog, false, true, env)?;
        Ok(())
    }

    pub(super) fn request_close_dialog_with_env(
        &mut self,
        dialog: NodeId,
        return_value: Option<Value>,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        self.ensure_dialog_target(dialog, "requestClose()")?;
        if let Some(return_value) = return_value {
            self.set_dialog_return_value(dialog, return_value.as_string())?;
        }
        if !self.dom.has_attr(dialog, "open")? {
            return Ok(());
        }
        let cancel_event = self.dispatch_event_with_options(
            dialog, "cancel", env, true, false, true, None, None, None,
        )?;
        if cancel_event.default_prevented {
            return Ok(());
        }
        let _ = self.transition_dialog_open_state_with_env(dialog, false, true, env)?;
        Ok(())
    }

    pub(super) fn transition_dialog_open_state_with_env(
        &mut self,
        dialog: NodeId,
        open: bool,
        fire_close_event: bool,
        env: &mut HashMap<String, Value>,
    ) -> Result<bool> {
        let was_open = self.dom.has_attr(dialog, "open")?;
        if was_open == open {
            return Ok(false);
        }

        let (old_state, new_state) = if open {
            ("closed", "open")
        } else {
            ("open", "closed")
        };
        let beforetoggle = self.dispatch_event_with_options(
            dialog,
            "beforetoggle",
            env,
            true,
            false,
            true,
            None,
            Some(old_state),
            Some(new_state),
        )?;
        if beforetoggle.default_prevented {
            return Ok(false);
        }

        if open {
            self.dom.set_attr(dialog, "open", "true")?;
        } else {
            self.dom.remove_attr(dialog, "open")?;
        }

        let _ = self.dispatch_event_with_options(
            dialog,
            "toggle",
            env,
            true,
            false,
            false,
            None,
            Some(old_state),
            Some(new_state),
        )?;

        if !open && fire_close_event {
            let _ = self.dispatch_event_with_options(
                dialog, "close", env, true, false, false, None, None, None,
            )?;
        }

        Ok(true)
    }

    pub(super) fn invoke_listeners(
        &mut self,
        node_id: NodeId,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
        capture: bool,
    ) -> Result<()> {
        let listeners = self.listeners.get(node_id, &event.event_type, capture);
        for listener in listeners {
            let mut listener_env = env.clone();
            for (name, value) in &listener.captured_env {
                if Self::is_internal_env_key(name) {
                    continue;
                }
                if !listener_env.contains_key(name) {
                    listener_env.insert(name.clone(), value.clone());
                }
            }
            let current_keys = env.keys().cloned().collect::<Vec<_>>();
            let mut script_env_before = HashMap::new();
            for key in &current_keys {
                if let Some(value) = self.script_env.get(key).cloned() {
                    script_env_before.insert(key.clone(), value);
                }
            }
            if self.trace {
                let phase = if capture { "capture" } else { "bubble" };
                let target_label = self.trace_node_label(event.target);
                let current_label = self.trace_node_label(event.current_target);
                self.trace_event_line(format!(
                    "[event] {} target={} current={} phase={} default_prevented={}",
                    event.event_type, target_label, current_label, phase, event.default_prevented
                ));
            }
            self.execute_handler(&listener.handler, event, &mut listener_env)?;
            for key in current_keys {
                let listener_value = listener_env.get(&key).cloned();
                let before = script_env_before.get(&key);
                let after = self.script_env.get(&key).cloned();
                let script_changed = match (before, after.as_ref()) {
                    (Some(prev), Some(next)) => !self.strict_equal(prev, next),
                    (None, Some(_)) => true,
                    _ => false,
                };
                if script_changed {
                    if let Some(value) = after {
                        env.insert(key, value);
                    } else if let Some(value) = listener_value {
                        env.insert(key, value);
                    }
                } else if let Some(value) = listener_value {
                    env.insert(key, value);
                }
            }
            if event.immediate_propagation_stopped {
                break;
            }
        }
        Ok(())
    }

    pub(super) fn trace_event_done(&mut self, event: &EventState, outcome: &str) {
        let target_label = self.trace_node_label(event.target);
        let current_label = self.trace_node_label(event.current_target);
        self.trace_event_line(format!(
            "[event] done {} target={} current={} outcome={} default_prevented={} propagation_stopped={} immediate_stopped={}",
            event.event_type,
            target_label,
            current_label,
            outcome,
            event.default_prevented,
            event.propagation_stopped,
            event.immediate_propagation_stopped
        ));
    }

    pub(super) fn trace_event_line(&mut self, line: String) {
        if self.trace && self.trace_events {
            self.trace_line(line);
        }
    }

    pub(super) fn trace_timer_line(&mut self, line: String) {
        if self.trace && self.trace_timers {
            self.trace_line(line);
        }
    }

    pub(super) fn trace_line(&mut self, line: String) {
        if self.trace {
            if self.trace_to_stderr {
                eprintln!("{line}");
            }
            if self.trace_logs.len() >= self.trace_log_limit {
                self.trace_logs.remove(0);
            }
            self.trace_logs.push(line);
        }
    }

    pub(super) fn queue_microtask(&mut self, handler: ScriptHandler, env: &HashMap<String, Value>) {
        self.microtask_queue.push_back(ScheduledMicrotask::Script {
            handler,
            env: env.clone(),
        });
    }

    pub(super) fn queue_promise_reaction_microtask(
        &mut self,
        reaction: PromiseReactionKind,
        settled: PromiseSettledValue,
    ) {
        self.microtask_queue
            .push_back(ScheduledMicrotask::Promise { reaction, settled });
    }

    pub(super) fn run_microtask_queue(&mut self) -> Result<usize> {
        let mut steps = 0usize;
        self.task_depth += 1;
        let result = loop {
            let Some(task) = self.microtask_queue.pop_front() else {
                break Ok(());
            };
            steps += 1;
            if steps > self.timer_step_limit {
                break Err(self.timer_step_limit_error(
                    self.timer_step_limit,
                    steps,
                    Some(self.now_ms),
                ));
            }

            match task {
                ScheduledMicrotask::Script { handler, mut env } => {
                    let mut event = EventState::new("microtask", self.dom.root, self.now_ms);
                    let event_param = handler
                        .first_event_param()
                        .map(|event_param| event_param.to_string());
                    self.bind_handler_params(&handler, &[], &mut env, &event_param, &event)?;
                    let run =
                        self.execute_stmts(&handler.stmts, &event_param, &mut event, &mut env);
                    let run = run.map(|_| ());
                    if let Err(err) = run {
                        break Err(err);
                    }
                }
                ScheduledMicrotask::Promise { reaction, settled } => {
                    self.run_promise_reaction_task(reaction, settled)?;
                }
            }
        };
        self.task_depth -= 1;
        result?;
        Ok(steps)
    }

    pub(super) fn run_in_task_context<T>(
        &mut self,
        mut run: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<T> {
        self.task_depth += 1;
        let result = run(self);
        self.task_depth -= 1;
        let should_flush_microtasks = self.task_depth == 0;
        match result {
            Ok(value) => {
                if should_flush_microtasks {
                    self.run_microtask_queue()?;
                }
                Ok(value)
            }
            Err(err) => Err(err),
        }
    }

    pub(super) fn execute_handler(
        &mut self,
        handler: &ScriptHandler,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let event_param = handler
            .first_event_param()
            .map(|event_param| event_param.to_string());
        let event_args = if event_param.is_some() {
            vec![Self::new_object_value(Vec::new())]
        } else {
            Vec::new()
        };
        self.bind_handler_params(handler, &event_args, env, &event_param, event)?;
        let flow = self.execute_stmts(&handler.stmts, &event_param, event, env)?;
        env.remove(INTERNAL_RETURN_SLOT);
        match flow {
            ExecFlow::Continue => Ok(()),
            ExecFlow::Break => Err(Error::ScriptRuntime(
                "break statement outside of loop".into(),
            )),
            ExecFlow::ContinueLoop => Err(Error::ScriptRuntime(
                "continue statement outside of loop".into(),
            )),
            ExecFlow::Return => Ok(()),
        }
    }

    pub(super) fn execute_timer_task_callback(
        &mut self,
        callback: &TimerCallback,
        callback_args: &[Value],
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let handler = match callback {
            TimerCallback::Inline(handler) => handler.clone(),
            TimerCallback::Reference(name) => {
                let value = env
                    .get(name)
                    .cloned()
                    .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;
                let Value::Function(function) = value else {
                    return Err(Error::ScriptRuntime(format!(
                        "timer callback '{name}' is not a function"
                    )));
                };
                function.handler.clone()
            }
        };
        let event_param = handler
            .first_event_param()
            .map(|event_param| event_param.to_string());
        self.bind_handler_params(&handler, callback_args, env, &event_param, event)?;
        let flow = self.execute_stmts(&handler.stmts, &event_param, event, env)?;
        env.remove(INTERNAL_RETURN_SLOT);
        match flow {
            ExecFlow::Continue => Ok(()),
            ExecFlow::Break => Err(Error::ScriptRuntime(
                "break statement outside of loop".into(),
            )),
            ExecFlow::ContinueLoop => Err(Error::ScriptRuntime(
                "continue statement outside of loop".into(),
            )),
            ExecFlow::Return => Ok(()),
        }
    }

    fn is_internal_env_key(name: &str) -> bool {
        name.starts_with("\u{0}\u{0}bt_")
    }

    fn env_scope_depth(env: &HashMap<String, Value>) -> i64 {
        match env.get(INTERNAL_SCOPE_DEPTH_KEY) {
            Some(Value::Number(depth)) if *depth >= 0 => *depth,
            _ => 0,
        }
    }

    fn env_should_sync_global_name(env: &HashMap<String, Value>, name: &str) -> bool {
        match env.get(INTERNAL_GLOBAL_SYNC_NAMES_KEY) {
            Some(Value::Array(names)) => names
                .borrow()
                .iter()
                .any(|entry| matches!(entry, Value::String(value) if value == name)),
            _ => false,
        }
    }

    pub(super) fn sync_global_binding_if_needed(
        &mut self,
        env: &HashMap<String, Value>,
        name: &str,
        value: &Value,
    ) {
        if Self::env_should_sync_global_name(env, name) {
            self.script_env.insert(name.to_string(), value.clone());
        }
    }

    pub(super) fn make_function_value(
        &self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        is_async: bool,
    ) -> Value {
        let local_bindings = Self::collect_function_scope_bindings(&handler);
        let scope_depth = Self::env_scope_depth(env);
        let captured_env = if global_scope {
            self.script_env.clone()
        } else {
            env.clone()
        };
        let mut captured_global_names = HashSet::new();
        for (name, value) in &captured_env {
            if Self::is_internal_env_key(name) || name == INTERNAL_RETURN_SLOT {
                continue;
            }
            if scope_depth == 0 {
                captured_global_names.insert(name.clone());
                continue;
            }
            let Some(global_value) = self.script_env.get(name) else {
                continue;
            };
            if global_scope || self.strict_equal(global_value, value) {
                captured_global_names.insert(name.clone());
            }
        }
        Value::Function(Rc::new(FunctionValue {
            handler,
            captured_env,
            captured_global_names,
            local_bindings,
            global_scope,
            is_async,
        }))
    }

    pub(super) fn is_callable_value(&self, value: &Value) -> bool {
        matches!(
            value,
            Value::Function(_) | Value::PromiseCapability(_) | Value::StringConstructor
        ) || Self::callable_kind_from_value(value).is_some()
    }

    pub(super) fn execute_callable_value(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
    ) -> Result<Value> {
        self.execute_callable_value_with_env(callable, args, event, None)
    }

    pub(super) fn execute_callable_value_with_env(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        match callable {
            Value::Function(function) => {
                self.execute_function_call(function.as_ref(), args, event, caller_env)
            }
            Value::PromiseCapability(capability) => {
                self.invoke_promise_capability(capability, args)
            }
            Value::StringConstructor => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Ok(Value::String(value.as_string()))
            }
            Value::Object(_) => {
                let Some(kind) = Self::callable_kind_from_value(callable) else {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                };
                match kind {
                    "intl_collator_compare" => {
                        let (locale, case_first, sensitivity) =
                            self.resolve_intl_collator_options(callable)?;
                        let left = args
                            .first()
                            .cloned()
                            .unwrap_or(Value::Undefined)
                            .as_string();
                        let right = args.get(1).cloned().unwrap_or(Value::Undefined).as_string();
                        Ok(Value::Number(Self::intl_collator_compare_strings(
                            &left,
                            &right,
                            &locale,
                            &case_first,
                            &sensitivity,
                        )))
                    }
                    "intl_date_time_format" => {
                        let (locale, options) = self.resolve_intl_date_time_options(callable)?;
                        let timestamp_ms = args
                            .first()
                            .map(|value| self.coerce_date_timestamp_ms(value))
                            .unwrap_or(self.now_ms);
                        Ok(Value::String(self.intl_format_date_time(
                            timestamp_ms,
                            &locale,
                            &options,
                        )))
                    }
                    "intl_duration_format" => {
                        let (locale, options) = self.resolve_intl_duration_options(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(
                            self.intl_format_duration(&locale, &options, &value)?,
                        ))
                    }
                    "intl_list_format" => {
                        let (locale, options) = self.resolve_intl_list_options(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(
                            self.intl_format_list(&locale, &options, &value)?,
                        ))
                    }
                    "intl_number_format" => {
                        let (_, locale) = self.resolve_intl_formatter(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(Self::intl_format_number_for_locale(
                            Self::coerce_number_for_global(&value),
                            &locale,
                        )))
                    }
                    "intl_segmenter_segments_iterator" => {
                        let Value::Object(entries) = callable else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let entries = entries.borrow();
                        let segments = Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENTS_KEY)
                            .ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "Intl.Segmenter iterator has invalid internal state".into(),
                                )
                            })?;
                        Ok(self.new_intl_segmenter_iterator_value(segments))
                    }
                    "intl_segmenter_iterator_next" => {
                        let Value::Object(entries) = callable else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let mut entries = entries.borrow_mut();
                        let segments = Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENTS_KEY)
                            .ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "Intl.Segmenter iterator has invalid internal state".into(),
                                )
                            })?;
                        let Value::Array(values) = segments else {
                            return Err(Error::ScriptRuntime(
                                "Intl.Segmenter iterator has invalid internal state".into(),
                            ));
                        };
                        let len = values.borrow().len();
                        let index =
                            match Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENT_INDEX_KEY)
                            {
                                Some(Value::Number(value)) if value >= 0 => value as usize,
                                _ => 0,
                            };
                        if index >= len {
                            return Ok(Self::new_object_value(vec![
                                ("value".to_string(), Value::Undefined),
                                ("done".to_string(), Value::Bool(true)),
                            ]));
                        }
                        let value = values
                            .borrow()
                            .get(index)
                            .cloned()
                            .unwrap_or(Value::Undefined);
                        Self::object_set_entry(
                            &mut entries,
                            INTERNAL_INTL_SEGMENT_INDEX_KEY.to_string(),
                            Value::Number((index + 1) as i64),
                        );
                        Ok(Self::new_object_value(vec![
                            ("value".to_string(), value),
                            ("done".to_string(), Value::Bool(false)),
                        ]))
                    }
                    "boolean_constructor" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::Bool(value.truthy()))
                    }
                    _ => Err(Error::ScriptRuntime("callback is not a function".into())),
                }
            }
            _ => Err(Error::ScriptRuntime("callback is not a function".into())),
        }
    }

    pub(super) fn invoke_promise_capability(
        &mut self,
        capability: &PromiseCapabilityFunction,
        args: &[Value],
    ) -> Result<Value> {
        let mut already_called = capability.already_called.borrow_mut();
        if *already_called {
            return Ok(Value::Undefined);
        }
        *already_called = true;
        drop(already_called);

        let value = args.first().cloned().unwrap_or(Value::Undefined);
        if capability.reject {
            self.promise_reject(&capability.promise, value);
            Ok(Value::Undefined)
        } else {
            self.promise_resolve(&capability.promise, value)?;
            Ok(Value::Undefined)
        }
    }

    pub(super) fn bind_handler_params(
        &mut self,
        handler: &ScriptHandler,
        args: &[Value],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        for (index, param) in handler.params.iter().enumerate() {
            if param.is_rest {
                let rest = if index < args.len() {
                    args[index..].to_vec()
                } else {
                    Vec::new()
                };
                env.insert(param.name.clone(), Self::new_array_value(rest));
                continue;
            }

            let provided = args.get(index).cloned().unwrap_or(Value::Undefined);
            let value = if matches!(provided, Value::Undefined) {
                if let Some(default_expr) = &param.default {
                    self.eval_expr(default_expr, env, event_param, event)?
                } else {
                    Value::Undefined
                }
            } else {
                provided
            };
            env.insert(param.name.clone(), value);
        }
        Ok(())
    }

    pub(super) fn execute_function_call(
        &mut self,
        function: &FunctionValue,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        let run = |this: &mut Self, caller_env: Option<&HashMap<String, Value>>| -> Result<Value> {
            let mut call_env = if function.global_scope {
                this.script_env.clone()
            } else {
                function.captured_env.clone()
            };
            call_env.remove(INTERNAL_RETURN_SLOT);
            let scope_depth = Self::env_scope_depth(&call_env);
            call_env.insert(
                INTERNAL_SCOPE_DEPTH_KEY.to_string(),
                Value::Number(scope_depth.saturating_add(1)),
            );
            let mut global_sync_keys = HashSet::new();
            let caller_view = caller_env;
            for name in &function.captured_global_names {
                if Self::is_internal_env_key(name) || function.local_bindings.contains(name) {
                    continue;
                }
                global_sync_keys.insert(name.clone());
                if let Some(global_value) = this.script_env.get(name).cloned() {
                    call_env.insert(name.clone(), global_value);
                } else if let Some(value) = caller_view.and_then(|env| env.get(name)).cloned() {
                    call_env.insert(name.clone(), value);
                }
            }
            for (name, global_value) in &this.script_env {
                if Self::is_internal_env_key(name)
                    || function.local_bindings.contains(name)
                    || call_env.contains_key(name)
                {
                    continue;
                }
                call_env.insert(name.clone(), global_value.clone());
                global_sync_keys.insert(name.clone());
            }
            if !global_sync_keys.is_empty() {
                let mut sync_names = global_sync_keys.iter().cloned().collect::<Vec<_>>();
                sync_names.sort();
                call_env.insert(
                    INTERNAL_GLOBAL_SYNC_NAMES_KEY.to_string(),
                    Self::new_array_value(sync_names.into_iter().map(Value::String).collect()),
                );
            }
            let mut global_values_before_call = HashMap::new();
            for name in &global_sync_keys {
                if let Some(value) = this.script_env.get(name).cloned() {
                    global_values_before_call.insert(name.clone(), value);
                }
            }
            let mut call_event = event.clone();
            let event_param = None;
            this.bind_handler_params(
                &function.handler,
                args,
                &mut call_env,
                &event_param,
                &call_event,
            )?;
            let flow = this.execute_stmts(
                &function.handler.stmts,
                &event_param,
                &mut call_event,
                &mut call_env,
            )?;
            for name in &global_sync_keys {
                if Self::is_internal_env_key(name) || function.local_bindings.contains(name) {
                    continue;
                }
                let before = global_values_before_call.get(name);
                let global_after = this.script_env.get(name).cloned();
                let call_after = call_env.get(name).cloned();
                let global_changed = match (before, global_after.as_ref()) {
                    (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                    (None, Some(_)) => true,
                    (Some(_), None) => true,
                    (None, None) => false,
                };
                let call_changed = match (before, call_after.as_ref()) {
                    (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                    (None, Some(_)) => true,
                    (Some(_), None) => true,
                    (None, None) => false,
                };
                if global_changed && !call_changed {
                    continue;
                }
                if let Some(next) = call_after {
                    this.script_env.insert(name.clone(), next);
                }
            }
            match flow {
                ExecFlow::Continue => Ok(Value::Undefined),
                ExecFlow::Break => Err(Error::ScriptRuntime(
                    "break statement outside of loop".into(),
                )),
                ExecFlow::ContinueLoop => Err(Error::ScriptRuntime(
                    "continue statement outside of loop".into(),
                )),
                ExecFlow::Return => Ok(call_env
                    .remove(INTERNAL_RETURN_SLOT)
                    .unwrap_or(Value::Undefined)),
            }
        };

        if function.is_async {
            let promise = self.new_pending_promise();
            match run(self, caller_env) {
                Ok(value) => {
                    if let Err(err) = self.promise_resolve(&promise, value) {
                        self.promise_reject(&promise, Self::promise_error_reason(err));
                    }
                }
                Err(err) => self.promise_reject(&promise, Self::promise_error_reason(err)),
            }
            Ok(Value::Promise(promise))
        } else {
            run(self, caller_env)
        }
    }

    pub(super) fn error_to_catch_value(err: Error) -> std::result::Result<Value, Error> {
        match err {
            Error::ScriptThrown(value) => Ok(value.into_value()),
            Error::ScriptRuntime(message) => Ok(Value::String(message)),
            other => Err(other),
        }
    }

    pub(super) fn bind_catch_binding(
        &self,
        binding: &CatchBinding,
        caught: &Value,
        env: &mut HashMap<String, Value>,
    ) -> Result<Vec<(String, Option<Value>)>> {
        let mut previous = Vec::new();
        let mut seen = HashSet::new();
        let mut remember = |name: &str, env: &HashMap<String, Value>| {
            if seen.insert(name.to_string()) {
                previous.push((name.to_string(), env.get(name).cloned()));
            }
        };

        match binding {
            CatchBinding::Identifier(name) => {
                remember(name, env);
                env.insert(name.clone(), caught.clone());
            }
            CatchBinding::ArrayPattern(pattern) => {
                let values = self.array_like_values_from_value(caught)?;
                for (index, name) in pattern.iter().enumerate() {
                    let Some(name) = name else {
                        continue;
                    };
                    remember(name, env);
                    let value = values.get(index).cloned().unwrap_or(Value::Undefined);
                    env.insert(name.clone(), value);
                }
            }
            CatchBinding::ObjectPattern(pattern) => {
                let Value::Object(entries) = caught else {
                    return Err(Error::ScriptRuntime(
                        "catch object binding requires an object value".into(),
                    ));
                };
                let entries = entries.borrow();
                for (source_key, target_name) in pattern {
                    remember(target_name, env);
                    let value =
                        Self::object_get_entry(&entries, source_key).unwrap_or(Value::Undefined);
                    env.insert(target_name.clone(), value);
                }
            }
        }

        Ok(previous)
    }

    pub(super) fn restore_catch_binding(
        &self,
        previous: Vec<(String, Option<Value>)>,
        env: &mut HashMap<String, Value>,
    ) {
        for (name, value) in previous {
            if let Some(value) = value {
                env.insert(name, value);
            } else {
                env.remove(&name);
            }
        }
    }

    pub(super) fn execute_catch_block(
        &mut self,
        catch_binding: &Option<CatchBinding>,
        catch_stmts: &[Stmt],
        caught: Value,
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        let previous = if let Some(binding) = catch_binding {
            self.bind_catch_binding(binding, &caught, env)?
        } else {
            Vec::new()
        };
        let result = self.execute_stmts(catch_stmts, event_param, event, env);
        self.restore_catch_binding(previous, env);
        result
    }

    pub(super) fn parse_function_constructor_param_names(spec: &str) -> Result<Vec<String>> {
        let mut params = Vec::new();
        for raw in spec.split(',') {
            let raw = raw.trim();
            if raw.is_empty() {
                return Err(Error::ScriptRuntime(
                    "new Function parameter name cannot be empty".into(),
                ));
            }
            if !is_ident(raw) {
                return Err(Error::ScriptRuntime(format!(
                    "new Function parameter name is invalid: {raw}"
                )));
            }
            params.push(raw.to_string());
        }
        Ok(params)
    }

    fn collect_function_decls(stmts: &[Stmt]) -> HashMap<String, (ScriptHandler, bool)> {
        let mut out = HashMap::new();
        for stmt in stmts {
            if let Stmt::FunctionDecl {
                name,
                handler,
                is_async,
            } = stmt
            {
                out.insert(name.clone(), (handler.clone(), *is_async));
            }
        }
        out
    }

    fn collect_function_scope_bindings(handler: &ScriptHandler) -> HashSet<String> {
        let mut bindings = HashSet::new();
        for param in &handler.params {
            bindings.insert(param.name.clone());
        }
        Self::collect_scope_bindings_from_stmts(&handler.stmts, &mut bindings);
        bindings
    }

    fn collect_scope_bindings_from_stmts(stmts: &[Stmt], out: &mut HashSet<String>) {
        for stmt in stmts {
            Self::collect_scope_bindings_from_stmt(stmt, out);
        }
    }

    fn collect_scope_bindings_from_stmt(stmt: &Stmt, out: &mut HashSet<String>) {
        match stmt {
            Stmt::VarDecl { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::FunctionDecl { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::ForEach {
                item_var,
                index_var,
                body,
                ..
            }
            | Stmt::ClassListForEach {
                item_var,
                index_var,
                body,
                ..
            } => {
                out.insert(item_var.clone());
                if let Some(index_var) = index_var {
                    out.insert(index_var.clone());
                }
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::For { init, body, .. } => {
                if let Some(init) = init {
                    Self::collect_scope_bindings_from_stmt(init, out);
                }
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::ForIn { item_var, body, .. } | Stmt::ForOf { item_var, body, .. } => {
                out.insert(item_var.clone());
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::DoWhile { body, .. } | Stmt::While { body, .. } => {
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::Try {
                try_stmts,
                catch_binding,
                catch_stmts,
                finally_stmts,
            } => {
                Self::collect_scope_bindings_from_stmts(try_stmts, out);
                if let Some(catch_binding) = catch_binding {
                    Self::collect_scope_bindings_from_catch_binding(catch_binding, out);
                }
                if let Some(catch_stmts) = catch_stmts {
                    Self::collect_scope_bindings_from_stmts(catch_stmts, out);
                }
                if let Some(finally_stmts) = finally_stmts {
                    Self::collect_scope_bindings_from_stmts(finally_stmts, out);
                }
            }
            Stmt::If {
                then_stmts,
                else_stmts,
                ..
            } => {
                Self::collect_scope_bindings_from_stmts(then_stmts, out);
                Self::collect_scope_bindings_from_stmts(else_stmts, out);
            }
            _ => {}
        }
    }

    fn collect_scope_bindings_from_catch_binding(
        binding: &CatchBinding,
        out: &mut HashSet<String>,
    ) {
        match binding {
            CatchBinding::Identifier(name) => {
                out.insert(name.clone());
            }
            CatchBinding::ArrayPattern(pattern) => {
                for entry in pattern.iter().flatten() {
                    out.insert(entry.clone());
                }
            }
            CatchBinding::ObjectPattern(pattern) => {
                for (_, target) in pattern {
                    out.insert(target.clone());
                }
            }
        }
    }

    pub(super) fn resolve_pending_function_decl(
        &self,
        name: &str,
        env: &HashMap<String, Value>,
    ) -> Option<Value> {
        for scope in self.pending_function_decls.iter().rev() {
            let Some((handler, is_async)) = scope.get(name) else {
                continue;
            };
            return Some(self.make_function_value(handler.clone(), env, false, *is_async));
        }
        None
    }

    pub(super) fn execute_stmts(
        &mut self,
        stmts: &[Stmt],
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        let pending = Self::collect_function_decls(stmts);
        self.pending_function_decls.push(pending);

        let result = (|| -> Result<ExecFlow> {
            for stmt in stmts {
                match stmt {
                Stmt::VarDecl { name, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    env.insert(name.clone(), value.clone());
                    self.bind_timer_id_to_task_env(name, expr, &value);
                }
                Stmt::FunctionDecl {
                    name,
                    handler,
                    is_async,
                } => {
                    let function = self.make_function_value(handler.clone(), env, false, *is_async);
                    env.insert(name.clone(), function);
                }
                Stmt::VarAssign { name, op, expr } => {
                    let previous = env
                        .get(name)
                        .cloned()
                        .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;

                    let next = match op {
                        VarAssignOp::Assign => self.eval_expr(expr, env, event_param, event)?,
                        VarAssignOp::Add => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.add_values(&previous, &value)?
                        }
                        VarAssignOp::Sub => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::Sub, &previous, &value)?
                        }
                        VarAssignOp::Mul => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::Mul, &previous, &value)?
                        }
                        VarAssignOp::Pow => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::Pow, &previous, &value)?
                        }
                        VarAssignOp::BitOr => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::BitOr, &previous, &value)?
                        }
                        VarAssignOp::BitXor => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::BitXor, &previous, &value)?
                        }
                        VarAssignOp::BitAnd => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::BitAnd, &previous, &value)?
                        }
                        VarAssignOp::ShiftLeft => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::ShiftLeft, &previous, &value)?
                        }
                        VarAssignOp::ShiftRight => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::ShiftRight, &previous, &value)?
                        }
                        VarAssignOp::UnsignedShiftRight => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::UnsignedShiftRight, &previous, &value)?
                        }
                        VarAssignOp::Div => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::Div, &previous, &value)?
                        }
                        VarAssignOp::Mod => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::Mod, &previous, &value)?
                        }
                        VarAssignOp::LogicalAnd => {
                            if previous.truthy() {
                                self.eval_expr(expr, env, event_param, event)?
                            } else {
                                previous.clone()
                            }
                        }
                        VarAssignOp::LogicalOr => {
                            if previous.truthy() {
                                previous.clone()
                            } else {
                                self.eval_expr(expr, env, event_param, event)?
                            }
                        }
                        VarAssignOp::Nullish => {
                            if matches!(&previous, Value::Null | Value::Undefined) {
                                self.eval_expr(expr, env, event_param, event)?
                            } else {
                                previous.clone()
                            }
                        }
                    };
                    env.insert(name.clone(), next.clone());
                    self.sync_global_binding_if_needed(env, name, &next);
                    self.bind_timer_id_to_task_env(name, expr, &next);
                }
                Stmt::VarUpdate { name, delta } => {
                    let previous = env
                        .get(name)
                        .cloned()
                        .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;
                    let next = match previous {
                        Value::Number(value) => {
                            if let Some(sum) = value.checked_add(i64::from(*delta)) {
                                Value::Number(sum)
                            } else {
                                Value::Float((value as f64) + f64::from(*delta))
                            }
                        }
                        Value::Float(value) => Value::Float(value + f64::from(*delta)),
                        Value::BigInt(value) => Value::BigInt(value + JsBigInt::from(*delta)),
                        _ => {
                            return Err(Error::ScriptRuntime(format!(
                                "cannot apply update operator to '{}'",
                                name
                            )));
                        }
                    };
                    env.insert(name.clone(), next.clone());
                    self.sync_global_binding_if_needed(env, name, &next);
                }
                Stmt::ArrayDestructureAssign { targets, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    let values = self.array_like_values_from_value(&value)?;
                    for (index, target_name) in targets.iter().enumerate() {
                        let Some(target_name) = target_name else {
                            continue;
                        };
                        let next = values.get(index).cloned().unwrap_or(Value::Undefined);
                        env.insert(target_name.clone(), next.clone());
                        self.sync_global_binding_if_needed(env, target_name, &next);
                    }
                }
                Stmt::ObjectDestructureAssign { bindings, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    let Value::Object(entries) = value else {
                        return Err(Error::ScriptRuntime(
                            "object destructuring source must be an object".into(),
                        ));
                    };
                    let entries = entries.borrow();
                    for (source_key, target_name) in bindings {
                        let next = Self::object_get_entry(&entries, source_key)
                            .unwrap_or(Value::Undefined);
                        env.insert(target_name.clone(), next.clone());
                        self.sync_global_binding_if_needed(env, target_name, &next);
                    }
                }
                Stmt::ObjectAssign { target, path, expr } => {
                    self.execute_object_assignment_stmt(
                        target,
                        path,
                        expr,
                        env,
                        event_param,
                        event,
                    )?;
                }
                Stmt::FormDataAppend {
                    target_var,
                    name,
                    value,
                } => {
                    let name = self.eval_expr(name, env, event_param, event)?;
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let name = name.as_string();
                    let value = value.as_string();
                    let target = env.get_mut(target_var).ok_or_else(|| {
                        Error::ScriptRuntime(format!("unknown FormData variable: {}", target_var))
                    })?;
                    match target {
                        Value::FormData(entries) => {
                            entries.push((name, value));
                        }
                        Value::Object(entries) => {
                            if !Self::is_url_search_params_object(&entries.borrow()) {
                                return Err(Error::ScriptRuntime(format!(
                                    "variable '{}' is not a FormData instance",
                                    target_var
                                )));
                            }
                            {
                                let mut object_ref = entries.borrow_mut();
                                let mut pairs =
                                    Self::url_search_params_pairs_from_object_entries(&object_ref);
                                pairs.push((name, value));
                                Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                            }
                            self.sync_url_search_params_owner(entries);
                        }
                        _ => {
                            return Err(Error::ScriptRuntime(format!(
                                "variable '{}' is not a FormData instance",
                                target_var
                            )));
                        }
                    }
                }
                Stmt::DomAssign { target, prop, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    if let DomQuery::Var(name) = target {
                        if let Some(Value::Object(entries)) = env.get(name) {
                            if let Some(key) = Self::object_key_from_dom_prop(prop) {
                                Self::object_set_entry(
                                    &mut entries.borrow_mut(),
                                    key.to_string(),
                                    value,
                                );
                                continue;
                            }
                        }
                    }
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match prop {
                        DomProp::TextContent => {
                            self.dom.set_text_content(node, &value.as_string())?
                        }
                        DomProp::InnerText => {
                            self.dom.set_text_content(node, &value.as_string())?
                        }
                        DomProp::InnerHtml => self.dom.set_inner_html(node, &value.as_string())?,
                        DomProp::OuterHtml => self.dom.set_outer_html(node, &value.as_string())?,
                        DomProp::Value => self.dom.set_value(node, &value.as_string())?,
                        DomProp::SelectionStart => {
                            let next_start = Self::value_to_i64(&value).max(0) as usize;
                            let end = self.dom.selection_end(node).unwrap_or_default();
                            self.dom
                                .set_selection_range(node, next_start, end, "none")?;
                        }
                        DomProp::SelectionEnd => {
                            let start = self.dom.selection_start(node).unwrap_or_default();
                            let next_end = Self::value_to_i64(&value).max(0) as usize;
                            self.dom
                                .set_selection_range(node, start, next_end, "none")?;
                        }
                        DomProp::SelectionDirection => {
                            let start = self.dom.selection_start(node).unwrap_or_default();
                            let end = self.dom.selection_end(node).unwrap_or_default();
                            let direction = value.as_string();
                            let direction =
                                Self::normalize_selection_direction(direction.as_str());
                            self.dom.set_selection_range(node, start, end, direction)?;
                        }
                        DomProp::Checked => self.dom.set_checked(node, value.truthy())?,
                        DomProp::Indeterminate => {
                            self.dom.set_indeterminate(node, value.truthy())?
                        }
                        DomProp::Open => {
                            if value.truthy() {
                                self.dom.set_attr(node, "open", "true")?;
                            } else {
                                self.dom.remove_attr(node, "open")?;
                            }
                        }
                        DomProp::ReturnValue => {
                            self.set_dialog_return_value(node, value.as_string())?;
                        }
                        DomProp::ClosedBy => {
                            self.dom.set_attr(node, "closedby", &value.as_string())?
                        }
                        DomProp::Readonly => {
                            if value.truthy() {
                                self.dom.set_attr(node, "readonly", "true")?;
                            } else {
                                self.dom.remove_attr(node, "readonly")?;
                            }
                        }
                        DomProp::Required => {
                            if value.truthy() {
                                self.dom.set_attr(node, "required", "true")?;
                            } else {
                                self.dom.remove_attr(node, "required")?;
                            }
                        }
                        DomProp::Disabled => {
                            if value.truthy() {
                                self.dom.set_attr(node, "disabled", "true")?;
                            } else {
                                self.dom.remove_attr(node, "disabled")?;
                            }
                        }
                        DomProp::Hidden => {
                            if node == self.dom.root {
                                let call = self.describe_dom_prop(prop);
                                return Err(Error::ScriptRuntime(format!("{call} is read-only")));
                            }
                            if value.truthy() {
                                self.dom.set_attr(node, "hidden", "true")?;
                            } else {
                                self.dom.remove_attr(node, "hidden")?;
                            }
                        }
                        DomProp::ClassName => {
                            self.dom.set_attr(node, "class", &value.as_string())?
                        }
                        DomProp::Id => self.dom.set_attr(node, "id", &value.as_string())?,
                        DomProp::Slot => self.dom.set_attr(node, "slot", &value.as_string())?,
                        DomProp::Role => self.dom.set_attr(node, "role", &value.as_string())?,
                        DomProp::ElementTiming => {
                            self.dom.set_attr(node, "elementtiming", &value.as_string())?
                        }
                        DomProp::Name => self.dom.set_attr(node, "name", &value.as_string())?,
                        DomProp::Lang => self.dom.set_attr(node, "lang", &value.as_string())?,
                        DomProp::Title => self.dom.set_document_title(&value.as_string())?,
                        DomProp::Location | DomProp::LocationHref => self.navigate_location(
                            &value.as_string(),
                            LocationNavigationKind::HrefSet,
                        )?,
                        DomProp::LocationProtocol => {
                            self.set_location_property("protocol", value.clone())?
                        }
                        DomProp::LocationHost => {
                            self.set_location_property("host", value.clone())?
                        }
                        DomProp::LocationHostname => {
                            self.set_location_property("hostname", value.clone())?
                        }
                        DomProp::LocationPort => {
                            self.set_location_property("port", value.clone())?
                        }
                        DomProp::LocationPathname => {
                            self.set_location_property("pathname", value.clone())?
                        }
                        DomProp::LocationSearch => {
                            self.set_location_property("search", value.clone())?
                        }
                        DomProp::LocationHash => {
                            self.set_location_property("hash", value.clone())?
                        }
                        DomProp::HistoryScrollRestoration => {
                            self.set_history_property("scrollRestoration", value.clone())?
                        }
                        DomProp::AnchorAttributionSrc => {
                            self.dom
                                .set_attr(node, "attributionsrc", &value.as_string())?
                        }
                        DomProp::AnchorDownload => {
                            self.dom.set_attr(node, "download", &value.as_string())?
                        }
                        DomProp::AnchorHash => {
                            self.set_anchor_url_property(node, "hash", value.clone())?
                        }
                        DomProp::AnchorHost => {
                            self.set_anchor_url_property(node, "host", value.clone())?
                        }
                        DomProp::AnchorHostname => {
                            self.set_anchor_url_property(node, "hostname", value.clone())?
                        }
                        DomProp::AnchorHref => {
                            self.set_anchor_url_property(node, "href", value.clone())?
                        }
                        DomProp::AnchorHreflang => {
                            self.dom.set_attr(node, "hreflang", &value.as_string())?
                        }
                        DomProp::AnchorInterestForElement => {
                            self.dom.set_attr(node, "interestfor", &value.as_string())?
                        }
                        DomProp::AnchorPassword => {
                            self.set_anchor_url_property(node, "password", value.clone())?
                        }
                        DomProp::AnchorPathname => {
                            self.set_anchor_url_property(node, "pathname", value.clone())?
                        }
                        DomProp::AnchorPing => {
                            self.dom.set_attr(node, "ping", &value.as_string())?
                        }
                        DomProp::AnchorPort => {
                            self.set_anchor_url_property(node, "port", value.clone())?
                        }
                        DomProp::AnchorProtocol => {
                            self.set_anchor_url_property(node, "protocol", value.clone())?
                        }
                        DomProp::AnchorReferrerPolicy => {
                            self.dom
                                .set_attr(node, "referrerpolicy", &value.as_string())?
                        }
                        DomProp::AnchorRel => self.dom.set_attr(node, "rel", &value.as_string())?,
                        DomProp::AnchorSearch => {
                            self.set_anchor_url_property(node, "search", value.clone())?
                        }
                        DomProp::AnchorTarget => {
                            self.dom.set_attr(node, "target", &value.as_string())?
                        }
                        DomProp::AnchorText => {
                            self.dom.set_text_content(node, &value.as_string())?
                        }
                        DomProp::AnchorType => {
                            self.dom.set_attr(node, "type", &value.as_string())?
                        }
                        DomProp::AnchorUsername => {
                            self.set_anchor_url_property(node, "username", value.clone())?
                        }
                        DomProp::AnchorCharset => {
                            self.dom.set_attr(node, "charset", &value.as_string())?
                        }
                        DomProp::AnchorCoords => {
                            self.dom.set_attr(node, "coords", &value.as_string())?
                        }
                        DomProp::AnchorRev => self.dom.set_attr(node, "rev", &value.as_string())?,
                        DomProp::AnchorShape => {
                            self.dom.set_attr(node, "shape", &value.as_string())?
                        }
                        DomProp::AriaString(prop_name) => {
                            let attr_name = Self::aria_property_to_attr_name(prop_name);
                            self.dom.set_attr(node, &attr_name, &value.as_string())?
                        }
                        DomProp::Attributes
                        | DomProp::AssignedSlot
                        | DomProp::ValidationMessage
                        | DomProp::Validity
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
                        | DomProp::ClassList
                        | DomProp::ClassListLength
                        | DomProp::Part
                        | DomProp::PartLength
                        | DomProp::TagName
                        | DomProp::LocalName
                        | DomProp::NamespaceUri
                        | DomProp::Prefix
                        | DomProp::NextElementSibling
                        | DomProp::PreviousElementSibling
                        | DomProp::ClientWidth
                        | DomProp::ClientHeight
                        | DomProp::ClientLeft
                        | DomProp::ClientTop
                        | DomProp::CurrentCssZoom
                        | DomProp::ScrollLeftMax
                        | DomProp::ScrollTopMax
                        | DomProp::ShadowRoot
                        | DomProp::AriaElementRefSingle(_)
                        | DomProp::AriaElementRefList(_)
                        | DomProp::OffsetWidth
                        | DomProp::ValueLength
                        | DomProp::OffsetHeight
                        | DomProp::OffsetLeft
                        | DomProp::OffsetTop
                        | DomProp::ScrollWidth
                        | DomProp::ScrollHeight
                        | DomProp::ScrollLeft
                        | DomProp::ScrollTop
                        | DomProp::ActiveElement
                        | DomProp::CharacterSet
                        | DomProp::CompatMode
                        | DomProp::ContentType
                        | DomProp::ReadyState
                        | DomProp::Referrer
                        | DomProp::Url
                        | DomProp::DocumentUri
                        | DomProp::LocationOrigin
                        | DomProp::LocationAncestorOrigins
                        | DomProp::History
                        | DomProp::HistoryLength
                        | DomProp::HistoryState
                        | DomProp::DefaultView
                        | DomProp::VisibilityState
                        | DomProp::Forms
                        | DomProp::Images
                        | DomProp::Links
                        | DomProp::Scripts
                        | DomProp::Children
                        | DomProp::ChildElementCount
                        | DomProp::FirstElementChild
                        | DomProp::LastElementChild
                        | DomProp::CurrentScript
                        | DomProp::FormsLength
                        | DomProp::ImagesLength
                        | DomProp::LinksLength
                        | DomProp::ScriptsLength
                        | DomProp::ChildrenLength
                        | DomProp::AnchorOrigin
                        | DomProp::AnchorRelList
                        | DomProp::AnchorRelListLength => {
                            let call = self.describe_dom_prop(prop);
                            return Err(Error::ScriptRuntime(format!("{call} is read-only")));
                        }
                        DomProp::Dataset(key) => {
                            self.dom.dataset_set(node, key, &value.as_string())?
                        }
                        DomProp::Style(prop) => {
                            self.dom.style_set(node, prop, &value.as_string())?
                        }
                    }
                }
                Stmt::ClassListCall {
                    target,
                    method,
                    class_names,
                    force,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match method {
                        ClassListMethod::Add => {
                            for class_name in class_names {
                                self.dom.class_add(node, class_name)?;
                            }
                        }
                        ClassListMethod::Remove => {
                            for class_name in class_names {
                                self.dom.class_remove(node, class_name)?;
                            }
                        }
                        ClassListMethod::Toggle => {
                            let class_name = class_names.first().ok_or_else(|| {
                                Error::ScriptRuntime("toggle requires a class name".into())
                            })?;
                            if let Some(force_expr) = force {
                                let force_value = self
                                    .eval_expr(force_expr, env, event_param, event)?
                                    .truthy();
                                if force_value {
                                    self.dom.class_add(node, class_name)?;
                                } else {
                                    self.dom.class_remove(node, class_name)?;
                                }
                            } else {
                                let _ = self.dom.class_toggle(node, class_name)?;
                            }
                        }
                    }
                }
                Stmt::ClassListForEach {
                    target,
                    item_var,
                    index_var,
                    body,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let classes = class_tokens(self.dom.attr(node, "class").as_deref());
                    let prev_item = env.get(item_var).cloned();
                    let prev_index = index_var.as_ref().and_then(|v| env.get(v).cloned());

                    for (idx, class_name) in classes.iter().enumerate() {
                        let item_value = Value::String(class_name.clone());
                        env.insert(item_var.clone(), item_value.clone());
                        self.sync_global_binding_if_needed(env, item_var, &item_value);
                        if let Some(index_var) = index_var {
                            let index_value = Value::Number(idx as i64);
                            env.insert(index_var.clone(), index_value.clone());
                            self.sync_global_binding_if_needed(env, index_var, &index_value);
                        }
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::Break => break,
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }

                    if let Some(prev) = prev_item {
                        env.insert(item_var.clone(), prev.clone());
                        self.sync_global_binding_if_needed(env, item_var, &prev);
                    } else {
                        env.remove(item_var);
                    }
                    if let Some(index_var) = index_var {
                        if let Some(prev) = prev_index {
                            env.insert(index_var.clone(), prev.clone());
                            self.sync_global_binding_if_needed(env, index_var, &prev);
                        } else {
                            env.remove(index_var);
                        }
                    }
                }
                Stmt::DomSetAttribute {
                    target,
                    name,
                    value,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let value = self.eval_expr(value, env, event_param, event)?;
                    self.dom.set_attr(node, name, &value.as_string())?;
                }
                Stmt::DomRemoveAttribute { target, name } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    self.dom.remove_attr(node, name)?;
                }
                Stmt::NodeTreeMutation {
                    target,
                    method,
                    child,
                    reference,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let child = self.eval_expr(child, env, event_param, event)?;
                    let Value::Node(child) = child else {
                        return Err(Error::ScriptRuntime(
                            "before/after/replaceWith/append/appendChild/prepend/removeChild/insertBefore argument must be an element reference".into(),
                        ));
                    };
                    match method {
                        NodeTreeMethod::After => self.dom.insert_after(target_node, child)?,
                        NodeTreeMethod::Append => self.dom.append_child(target_node, child)?,
                        NodeTreeMethod::AppendChild => self.dom.append_child(target_node, child)?,
                        NodeTreeMethod::Before => {
                            let Some(parent) = self.dom.parent(target_node) else {
                                continue;
                            };
                            self.dom.insert_before(parent, child, target_node)?;
                        }
                        NodeTreeMethod::ReplaceWith => {
                            self.dom.replace_with(target_node, child)?;
                        }
                        NodeTreeMethod::Prepend => self.dom.prepend_child(target_node, child)?,
                        NodeTreeMethod::RemoveChild => self.dom.remove_child(target_node, child)?,
                        NodeTreeMethod::InsertBefore => {
                            let Some(reference) = reference else {
                                return Err(Error::ScriptRuntime(
                                    "insertBefore requires reference node".into(),
                                ));
                            };
                            let reference = self.eval_expr(reference, env, event_param, event)?;
                            let Value::Node(reference) = reference else {
                                return Err(Error::ScriptRuntime(
                                    "insertBefore reference must be an element reference".into(),
                                ));
                            };
                            self.dom.insert_before(target_node, child, reference)?;
                        }
                    }
                }
                Stmt::InsertAdjacentElement {
                    target,
                    position,
                    node,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let node = self.eval_expr(node, env, event_param, event)?;
                    let Value::Node(node) = node else {
                        return Err(Error::ScriptRuntime(
                            "insertAdjacentElement second argument must be an element reference"
                                .into(),
                        ));
                    };
                    self.dom
                        .insert_adjacent_node(target_node, *position, node)?;
                }
                Stmt::InsertAdjacentText {
                    target,
                    position,
                    text,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let text = self.eval_expr(text, env, event_param, event)?;
                    if matches!(
                        position,
                        InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
                    ) && self.dom.parent(target_node).is_none()
                    {
                        continue;
                    }
                    let text_node = self.dom.create_detached_text(text.as_string());
                    self.dom
                        .insert_adjacent_node(target_node, *position, text_node)?;
                }
                Stmt::InsertAdjacentHTML {
                    target,
                    position,
                    html,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let position = self.eval_expr(position, env, event_param, event)?;
                    let position = resolve_insert_adjacent_position(&position.as_string())?;
                    let html = self.eval_expr(html, env, event_param, event)?;
                    self.dom
                        .insert_adjacent_html(target_node, position, &html.as_string())?;
                }
                Stmt::SetTimeout { handler, delay_ms } => {
                    let delay = self.eval_expr(delay_ms, env, event_param, event)?;
                    let delay = Self::value_to_i64(&delay);
                    let callback_args = handler
                        .args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    let _ =
                        self.schedule_timeout(handler.callback.clone(), delay, callback_args, env);
                }
                Stmt::SetInterval { handler, delay_ms } => {
                    let interval = self.eval_expr(delay_ms, env, event_param, event)?;
                    let interval = Self::value_to_i64(&interval);
                    let callback_args = handler
                        .args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    let _ = self.schedule_interval(
                        handler.callback.clone(),
                        interval,
                        callback_args,
                        env,
                    );
                }
                Stmt::QueueMicrotask { handler } => {
                    self.queue_microtask(handler.clone(), env);
                }
                Stmt::ClearTimeout { timer_id } => {
                    let timer_id = self.eval_expr(timer_id, env, event_param, event)?;
                    let timer_id = Self::value_to_i64(&timer_id);
                    self.clear_timeout(timer_id);
                }
                Stmt::NodeRemove { target } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    if let Some(active) = self.active_element {
                        if active == node || self.dom.is_descendant_of(active, node) {
                            self.active_element = None;
                            self.dom.set_active_element(None);
                        }
                    }
                    if let Some(active_pseudo) = self.dom.active_pseudo_element() {
                        if active_pseudo == node || self.dom.is_descendant_of(active_pseudo, node) {
                            self.dom.set_active_pseudo_element(None);
                        }
                    }
                    self.dom.remove_node(node)?;
                }
                Stmt::ForEach {
                    target,
                    selector,
                    item_var,
                    index_var,
                    body,
                } => {
                    let items = if let Some(target) = target {
                        match self.resolve_dom_query_runtime(target, env)? {
                            Some(target_node) => {
                                self.dom.query_selector_all_from(&target_node, selector)?
                            }
                            None => Vec::new(),
                        }
                    } else {
                        self.dom.query_selector_all(selector)?
                    };
                    let prev_item = env.get(item_var).cloned();
                    let prev_index = index_var.as_ref().and_then(|v| env.get(v).cloned());

                    for (idx, node) in items.iter().enumerate() {
                        let item_value = Value::Node(*node);
                        env.insert(item_var.clone(), item_value.clone());
                        self.sync_global_binding_if_needed(env, item_var, &item_value);
                        if let Some(index_var) = index_var {
                            let index_value = Value::Number(idx as i64);
                            env.insert(index_var.clone(), index_value.clone());
                            self.sync_global_binding_if_needed(env, index_var, &index_value);
                        }
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::Break => break,
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }

                    if let Some(prev) = prev_item {
                        env.insert(item_var.clone(), prev.clone());
                        self.sync_global_binding_if_needed(env, item_var, &prev);
                    } else {
                        env.remove(item_var);
                    }
                    if let Some(index_var) = index_var {
                        if let Some(prev) = prev_index {
                            env.insert(index_var.clone(), prev.clone());
                            self.sync_global_binding_if_needed(env, index_var, &prev);
                        } else {
                            env.remove(index_var);
                        }
                    }
                }
                Stmt::ArrayForEach { target, callback } => {
                    let target_value = env
                        .get(target)
                        .cloned()
                        .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {target}")))?;
                    self.execute_array_like_foreach_in_env(
                        target_value,
                        callback,
                        env,
                        event,
                        target,
                    )?;
                }
                Stmt::ArrayForEachExpr { target, callback } => {
                    let target_value = self.eval_expr(target, env, event_param, event)?;
                    self.execute_array_like_foreach_in_env(
                        target_value,
                        callback,
                        env,
                        event,
                        "<expression>",
                    )?;
                }
                Stmt::For {
                    init,
                    cond,
                    post,
                    body,
                } => {
                    if let Some(init) = init.as_deref() {
                        match self.execute_stmts(
                            std::slice::from_ref(init),
                            event_param,
                            event,
                            env,
                        )? {
                            ExecFlow::Continue => {}
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                            ExecFlow::Break => {
                                return Err(Error::ScriptRuntime(
                                    "break statement outside of loop".into(),
                                ));
                            }
                            ExecFlow::ContinueLoop => {
                                return Err(Error::ScriptRuntime(
                                    "continue statement outside of loop".into(),
                                ));
                            }
                        }
                    }

                    loop {
                        let should_run = if let Some(cond) = cond {
                            self.eval_expr(cond, env, event_param, event)?.truthy()
                        } else {
                            true
                        };
                        if !should_run {
                            break;
                        }

                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => {
                                if let Some(post) = post.as_deref() {
                                    match self.execute_stmts(
                                        std::slice::from_ref(post),
                                        event_param,
                                        event,
                                        env,
                                    )? {
                                        ExecFlow::Continue => {}
                                        ExecFlow::Return => return Ok(ExecFlow::Return),
                                        ExecFlow::Break | ExecFlow::ContinueLoop => {
                                            return Err(Error::ScriptRuntime(
                                                "invalid loop control in post expression".into(),
                                            ));
                                        }
                                    }
                                }
                                continue;
                            }
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                        if let Some(post) = post.as_deref() {
                            match self.execute_stmts(
                                std::slice::from_ref(post),
                                event_param,
                                event,
                                env,
                            )? {
                                ExecFlow::Continue => {}
                                ExecFlow::Return => return Ok(ExecFlow::Return),
                                ExecFlow::Break | ExecFlow::ContinueLoop => {
                                    return Err(Error::ScriptRuntime(
                                        "invalid loop control in post expression".into(),
                                    ));
                                }
                            }
                        }
                    }
                }
                Stmt::ForIn {
                    item_var,
                    iterable,
                    body,
                } => {
                    let iterable = self.eval_expr(iterable, env, event_param, event)?;
                    let items = match iterable {
                        Value::NodeList(nodes) => (0..nodes.len()).collect::<Vec<_>>(),
                        Value::Array(values) => {
                            let values = values.borrow();
                            (0..values.len()).collect::<Vec<_>>()
                        }
                        Value::Null | Value::Undefined => Vec::new(),
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "for...in iterable must be a NodeList or Array".into(),
                            ));
                        }
                    };

                    let prev_item = env.get(item_var).cloned();
                    for idx in items {
                        let item_value = Value::Number(idx as i64);
                        env.insert(item_var.clone(), item_value.clone());
                        self.sync_global_binding_if_needed(env, item_var, &item_value);
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }
                    if let Some(prev) = prev_item {
                        env.insert(item_var.clone(), prev.clone());
                        self.sync_global_binding_if_needed(env, item_var, &prev);
                    } else {
                        env.remove(item_var);
                    }
                }
                Stmt::ForOf {
                    item_var,
                    iterable,
                    body,
                } => {
                    let iterable = self.eval_expr(iterable, env, event_param, event)?;
                    let nodes = match iterable {
                        Value::NodeList(nodes) => {
                            nodes.into_iter().map(Value::Node).collect::<Vec<_>>()
                        }
                        Value::Array(values) => values.borrow().clone(),
                        Value::Map(map) => self.map_entries_array(&map),
                        Value::Set(set) => set.borrow().values.clone(),
                        Value::Object(entries) => {
                            if Self::is_url_search_params_object(&entries.borrow()) {
                                Self::url_search_params_pairs_from_object_entries(&entries.borrow())
                                    .into_iter()
                                    .map(|(key, value)| {
                                        Self::new_array_value(vec![
                                            Value::String(key),
                                            Value::String(value),
                                        ])
                                    })
                                    .collect::<Vec<_>>()
                            } else {
                                return Err(Error::ScriptRuntime(
                                    "for...of iterable must be a NodeList, Array, Map, Set, or URLSearchParams"
                                        .into(),
                                ));
                            }
                        }
                        Value::Null | Value::Undefined => Vec::new(),
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "for...of iterable must be a NodeList, Array, Map, Set, or URLSearchParams"
                                    .into(),
                            ));
                        }
                    };

                    let prev_item = env.get(item_var).cloned();
                    for item in nodes {
                        env.insert(item_var.clone(), item.clone());
                        self.sync_global_binding_if_needed(env, item_var, &item);
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }
                    if let Some(prev) = prev_item {
                        env.insert(item_var.clone(), prev.clone());
                        self.sync_global_binding_if_needed(env, item_var, &prev);
                    } else {
                        env.remove(item_var);
                    }
                }
                Stmt::While { cond, body } => {
                    while self.eval_expr(cond, env, event_param, event)?.truthy() {
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }
                }
                Stmt::DoWhile { cond, body } => loop {
                    match self.execute_stmts(body, event_param, event, env)? {
                        ExecFlow::Continue => {}
                        ExecFlow::ContinueLoop => {}
                        ExecFlow::Break => break,
                        ExecFlow::Return => return Ok(ExecFlow::Return),
                    }
                    if !self.eval_expr(cond, env, event_param, event)?.truthy() {
                        break;
                    }
                },
                Stmt::If {
                    cond,
                    then_stmts,
                    else_stmts,
                } => {
                    let cond = self.eval_expr(cond, env, event_param, event)?;
                    if cond.truthy() {
                        match self.execute_stmts(then_stmts, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    } else {
                        match self.execute_stmts(else_stmts, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                }
                Stmt::Try {
                    try_stmts,
                    catch_binding,
                    catch_stmts,
                    finally_stmts,
                } => {
                    let mut completion = self.execute_stmts(try_stmts, event_param, event, env);

                    if let Err(err) = completion {
                        if let Some(catch_stmts) = catch_stmts {
                            let caught = Self::error_to_catch_value(err)?;
                            completion = self.execute_catch_block(
                                catch_binding,
                                catch_stmts,
                                caught,
                                event_param,
                                event,
                                env,
                            );
                        } else {
                            completion = Err(err);
                        }
                    }

                    if let Some(finally_stmts) = finally_stmts {
                        match self.execute_stmts(finally_stmts, event_param, event, env) {
                            Ok(ExecFlow::Continue) => {}
                            Ok(flow) => return Ok(flow),
                            Err(err) => return Err(err),
                        }
                    }

                    match completion {
                        Ok(ExecFlow::Continue) => {}
                        Ok(flow) => return Ok(flow),
                        Err(err) => return Err(err),
                    }
                }
                Stmt::Throw { value } => {
                    let thrown = self.eval_expr(value, env, event_param, event)?;
                    return Err(Error::ScriptThrown(ThrownValue::new(thrown)));
                }
                Stmt::Return { value } => {
                    let return_value = if let Some(value) = value {
                        self.eval_expr(value, env, event_param, event)?
                    } else {
                        Value::Undefined
                    };
                    env.insert(INTERNAL_RETURN_SLOT.to_string(), return_value);
                    return Ok(ExecFlow::Return);
                }
                Stmt::Break => {
                    return Ok(ExecFlow::Break);
                }
                Stmt::Continue => {
                    return Ok(ExecFlow::ContinueLoop);
                }
                Stmt::EventCall { event_var, method } => {
                    if let Some(param) = event_param {
                        if param == event_var {
                            match method {
                                EventMethod::PreventDefault => {
                                    event.default_prevented = true;
                                }
                                EventMethod::StopPropagation => {
                                    event.propagation_stopped = true;
                                }
                                EventMethod::StopImmediatePropagation => {
                                    event.immediate_propagation_stopped = true;
                                    event.propagation_stopped = true;
                                }
                            }
                        }
                    }
                }
                Stmt::ListenerMutation {
                    target,
                    op,
                    event_type,
                    capture,
                    handler,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match op {
                        ListenerRegistrationOp::Add => {
                            self.listeners.add(
                                node,
                                event_type.clone(),
                                Listener {
                                    capture: *capture,
                                    handler: handler.clone(),
                                    captured_env: env.clone(),
                                },
                            );
                        }
                        ListenerRegistrationOp::Remove => {
                            let _ = self.listeners.remove(node, event_type, *capture, handler);
                        }
                    }
                }
                Stmt::DomMethodCall {
                    target,
                    method,
                    arg,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let arg_value = arg
                        .as_ref()
                        .map(|expr| self.eval_expr(expr, env, event_param, event))
                        .transpose()?;
                    match method {
                        DomMethod::Focus => self.focus_node_with_env(node, env)?,
                        DomMethod::Blur => self.blur_node_with_env(node, env)?,
                        DomMethod::Click => self.click_node_with_env(node, env)?,
                        DomMethod::Submit => self.submit_form_with_env(node, env)?,
                        DomMethod::Reset => self.reset_form_with_env(node, env)?,
                        DomMethod::ScrollIntoView => {
                            self.scroll_into_view_node_with_env(node, env)?
                        }
                        DomMethod::Show => self.show_dialog_with_env(node, false, env)?,
                        DomMethod::ShowModal => self.show_dialog_with_env(node, true, env)?,
                        DomMethod::Close => self.close_dialog_with_env(node, arg_value, env)?,
                        DomMethod::RequestClose => {
                            self.request_close_dialog_with_env(node, arg_value, env)?
                        }
                    }
                }
                Stmt::DispatchEvent { target, event_type } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let event_name = self
                        .eval_expr(event_type, env, event_param, event)?
                        .as_string();
                    if event_name.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "dispatchEvent requires non-empty event type".into(),
                        ));
                    }
                    let _ = self.dispatch_event_with_env(node, &event_name, env, false)?;
                }
                Stmt::Expr(expr) => {
                    let _ = self.eval_expr(expr, env, event_param, event)?;
                }
                }
            }

            Ok(ExecFlow::Continue)
        })();

        self.pending_function_decls.pop();
        result
    }

}
