use super::*;

impl Harness {
    pub fn from_html(html: &str) -> Result<Self> {
        Self::from_html_with_url("about:blank", html)
    }

    pub fn from_html_with_url(url: &str, html: &str) -> Result<Self> {
        stacker::grow(32 * 1024 * 1024, || Self::from_html_impl(url, html))
    }

    fn from_html_impl(url: &str, html: &str) -> Result<Self> {
        let ParseOutput { dom, scripts } = parse_html(html)?;
        let mut harness = Self {
            dom,
            listeners: ListenerStore::default(),
            dom_runtime: DomRuntimeState::default(),
            script_runtime: ScriptRuntimeState::default(),
            document_url: url.to_string(),
            location_history: LocationHistoryState::new(url),
            scheduler: SchedulerState::default(),
            promise_runtime: PromiseRuntimeState::default(),
            symbol_runtime: SymbolRuntimeState::default(),
            browser_apis: BrowserApiState::default(),
            rng_state: 0x9E37_79B9_7F4A_7C15,
            platform_mocks: PlatformMockState::default(),
            trace_state: TraceState::default(),
        };

        harness.initialize_global_bindings();

        for script in scripts {
            harness.compile_and_register_script(&script)?;
        }

        Ok(harness)
    }

    pub(crate) fn with_script_env<R>(
        &mut self,
        f: impl FnOnce(&mut Self, &mut HashMap<String, Value>) -> Result<R>,
    ) -> Result<R> {
        let mut env = self.script_runtime.env.share();
        match f(self, &mut env) {
            Ok(value) => {
                self.script_runtime.env = env;
                Ok(value)
            }
            Err(err) => Err(err),
        }
    }

    pub(crate) fn with_script_env_always<R>(
        &mut self,
        f: impl FnOnce(&mut Self, &mut HashMap<String, Value>) -> Result<R>,
    ) -> Result<R> {
        let mut env = self.script_runtime.env.share();
        let result = f(self, &mut env);
        self.script_runtime.env = env;
        result
    }

    pub(crate) fn initialize_global_bindings(&mut self) {
        self.sync_location_object();
        self.sync_history_object();
        self.dom_runtime.window_object = Rc::new(RefCell::new(ObjectValue::default()));
        self.dom_runtime.document_object = Rc::new(RefCell::new(ObjectValue::default()));
        self.browser_apis.url_constructor_properties.borrow_mut().clear();
        let local_storage_items = {
            let entries = self.browser_apis.local_storage_object.borrow();
            if Self::is_storage_object(&entries) {
                Self::storage_pairs_from_object_entries(&entries)
            } else {
                Vec::new()
            }
        };
        let mut local_storage_entries =
            vec![(INTERNAL_STORAGE_OBJECT_KEY.to_string(), Value::Bool(true))];
        Self::set_storage_pairs(&mut local_storage_entries, &local_storage_items);
        *self.browser_apis.local_storage_object.borrow_mut() = local_storage_entries.into();
        let clipboard = Self::new_object_value(vec![
            (INTERNAL_CLIPBOARD_OBJECT_KEY.into(), Value::Bool(true)),
            ("readText".into(), Self::new_builtin_placeholder_function()),
            ("writeText".into(), Self::new_builtin_placeholder_function()),
        ]);
        let location = Value::Object(self.dom_runtime.location_object.clone());
        let history = Value::Object(self.location_history.history_object.clone());

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
        let local_storage = Value::Object(self.browser_apis.local_storage_object.clone());

        self.sync_document_object();
        self.sync_window_object(
            &navigator,
            &intl,
            &string_constructor,
            &boolean_constructor,
            &url_constructor,
            &html_element_constructor,
            &html_input_element_constructor,
            &local_storage,
        );

        let window = Value::Object(self.dom_runtime.window_object.clone());
        let document = Value::Object(self.dom_runtime.document_object.clone());

        self.script_runtime.env.insert("document".to_string(), document);
        self.script_runtime.env
            .insert("navigator".to_string(), navigator.clone());
        self.script_runtime.env
            .insert("clientInformation".to_string(), navigator.clone());
        self.script_runtime.env.insert("Intl".to_string(), intl);
        self.script_runtime.env
            .insert("String".to_string(), string_constructor);
        self.script_runtime.env
            .insert("Boolean".to_string(), boolean_constructor);
        self.script_runtime.env.insert("URL".to_string(), url_constructor);
        self.script_runtime.env
            .insert("HTMLElement".to_string(), html_element_constructor);
        self.script_runtime.env.insert(
            "HTMLInputElement".to_string(),
            html_input_element_constructor,
        );
        self.script_runtime.env.insert("location".to_string(), location);
        self.script_runtime.env.insert("history".to_string(), history);
        self.script_runtime.env
            .insert("localStorage".to_string(), local_storage);
        self.script_runtime.env.insert("window".to_string(), window.clone());
        self.script_runtime.env.insert("self".to_string(), window.clone());
        self.script_runtime.env.insert("top".to_string(), window.clone());
        self.script_runtime.env.insert("parent".to_string(), window.clone());
        self.script_runtime.env.insert("frames".to_string(), window);
        self.script_runtime.env
            .insert(INTERNAL_SCOPE_DEPTH_KEY.to_string(), Value::Number(0));
    }

    pub(crate) fn current_location_parts(&self) -> LocationParts {
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

    pub(crate) fn window_is_secure_context(&self) -> bool {
        matches!(
            self.current_location_parts().scheme.as_str(),
            "https" | "wss"
        )
    }

    pub(crate) fn document_builtin_keys() -> &'static [&'static str] {
        &["defaultView", "location", "URL", "documentURI"]
    }

    pub(crate) fn sync_document_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.dom_runtime.document_object.borrow();
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
                Value::Object(self.dom_runtime.window_object.clone()),
            ),
            (
                "location".to_string(),
                Value::Object(self.dom_runtime.location_object.clone()),
            ),
            ("URL".to_string(), Value::String(self.document_url.clone())),
            (
                "documentURI".to_string(),
                Value::String(self.document_url.clone()),
            ),
        ];
        entries.extend(extras);
        *self.dom_runtime.document_object.borrow_mut() = entries.into();
    }

    pub(crate) fn window_builtin_keys() -> &'static [&'static str] {
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
            "localStorage",
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

    pub(crate) fn sync_window_object(
        &mut self,
        navigator: &Value,
        intl: &Value,
        string_constructor: &Value,
        boolean_constructor: &Value,
        url_constructor: &Value,
        html_element_constructor: &Value,
        html_input_element_constructor: &Value,
        local_storage: &Value,
    ) {
        let mut extras = Vec::new();
        let mut name_value = Value::String(String::new());
        {
            let entries = self.dom_runtime.window_object.borrow();
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

        let window_ref = Value::Object(self.dom_runtime.window_object.clone());
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
                Value::Object(self.dom_runtime.location_object.clone()),
            ),
            (
                "history".to_string(),
                Value::Object(self.location_history.history_object.clone()),
            ),
            ("navigator".to_string(), navigator.clone()),
            ("clientInformation".to_string(), navigator.clone()),
            (
                "document".to_string(),
                Value::Object(self.dom_runtime.document_object.clone()),
            ),
            ("localStorage".to_string(), local_storage.clone()),
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
        *self.dom_runtime.window_object.borrow_mut() = entries.into();
    }

    pub(crate) fn sync_window_runtime_properties(&mut self) {
        let mut entries = self.dom_runtime.window_object.borrow_mut();
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

    pub(crate) fn location_builtin_keys() -> &'static [&'static str] {
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

    pub(crate) fn sync_location_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.dom_runtime.location_object.borrow();
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
        *self.dom_runtime.location_object.borrow_mut() = entries.into();
    }

    pub(crate) fn is_location_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_LOCATION_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn history_builtin_keys() -> &'static [&'static str] {
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

    pub(crate) fn current_history_state(&self) -> Value {
        self.location_history.history_entries
            .get(self.location_history.history_index)
            .map(|entry| entry.state.clone())
            .unwrap_or(Value::Null)
    }

    pub(crate) fn sync_history_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.location_history.history_object.borrow();
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
                Value::Number(self.location_history.history_entries.len() as i64),
            ),
            (
                "scrollRestoration".to_string(),
                Value::String(self.location_history.history_scroll_restoration.clone()),
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
        *self.location_history.history_object.borrow_mut() = entries.into();
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
            self.dom_runtime.node_event_handler_props
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
            _ => {
                self.dom_runtime.node_expando_props.insert((node, key.to_string()), value);
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

        let final_key = keys.last().ok_or_else(|| {
            Error::ScriptRuntime("object assignment key cannot be empty".into())
        })?;
        self.set_object_assignment_property(&container, final_key, value, target)
    }

    pub(crate) fn resolve_location_target_url(&self, input: &str) -> String {
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

    pub(crate) fn is_hash_only_navigation(from: &str, to: &str) -> bool {
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

    pub(crate) fn navigate_location(
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
        self.location_history.location_navigations.push(LocationNavigation {
            kind,
            from,
            to: to.clone(),
        });

        if !Self::is_hash_only_navigation(
            &self
                .location_history
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

    pub(crate) fn reload_location(&mut self) -> Result<()> {
        self.location_history.location_reload_count += 1;
        let current = self.document_url.clone();
        self.location_history.location_navigations.push(LocationNavigation {
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

    pub(crate) fn load_location_mock_page_if_exists(&mut self, url: &str) -> Result<bool> {
        let Some(html) = self.location_history.location_mock_pages.get(url).cloned() else {
            return Ok(false);
        };
        self.replace_document_with_html(&html)?;
        Ok(true)
    }

    pub(crate) fn history_push_entry(&mut self, url: &str, state: Value) {
        let next = self
            .location_history
            .history_index
            .saturating_add(1)
            .min(self.location_history.history_entries.len());
        self.location_history.history_entries.truncate(next);
        self.location_history.history_entries.push(HistoryEntry {
            url: url.to_string(),
            state,
        });
        self.location_history.history_index = self.location_history.history_entries.len().saturating_sub(1);
    }

    pub(crate) fn history_replace_current_entry(&mut self, url: &str, state: Value) {
        if self.location_history.history_entries.is_empty() {
            self.location_history.history_entries.push(HistoryEntry {
                url: url.to_string(),
                state,
            });
            self.location_history.history_index = 0;
            return;
        }
        let index = self
            .location_history
            .history_index
            .min(self.location_history.history_entries.len().saturating_sub(1));
        self.location_history.history_entries[index] = HistoryEntry {
            url: url.to_string(),
            state,
        };
        self.location_history.history_index = index;
    }

    pub(crate) fn history_push_state(
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

    pub(crate) fn history_go_with_env(&mut self, delta: i64) -> Result<()> {
        if delta == 0 {
            self.reload_location()?;
            return Ok(());
        }

        let current = self.location_history.history_index as i64;
        let target = current.saturating_add(delta);
        if target < 0 || target >= self.location_history.history_entries.len() as i64 {
            return Ok(());
        }
        let target = target as usize;
        if target == self.location_history.history_index {
            return Ok(());
        }

        let from = self.document_url.clone();
        self.location_history.history_index = target;
        let entry = self
            .location_history
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

        self.with_script_env_always(|this, env| {
            let _ = this.dispatch_event_with_options(
                this.dom.root,
                "popstate",
                env,
                true,
                false,
                false,
                Some(entry.state),
                None,
                None,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub(crate) fn replace_document_with_html(&mut self, html: &str) -> Result<()> {
        let ParseOutput { dom, scripts } = parse_html(html)?;
        self.dom = dom;
        self.listeners = ListenerStore::default();
        self.dom_runtime.node_event_handler_props.clear();
        self.dom_runtime.node_expando_props.clear();
        self.script_runtime.env.clear();
        self.scheduler.task_queue.clear();
        self.scheduler.microtask_queue.clear();
        self.scheduler.running_timer_id = None;
        self.scheduler.running_timer_canceled = false;
        self.script_runtime.pending_function_decls.clear();
        self.script_runtime.listener_capture_env_stack.clear();
        self.dom.set_active_element(None);
        self.dom.set_active_pseudo_element(None);
        self.initialize_global_bindings();
        for script in scripts {
            self.compile_and_register_script(&script)?;
        }
        Ok(())
    }

    pub(crate) fn set_location_property(&mut self, key: &str, value: Value) -> Result<()> {
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
                    &mut self.dom_runtime.location_object.borrow_mut(),
                    key.to_string(),
                    value,
                );
                Ok(())
            }
        }
    }

    pub(crate) fn anchor_rel_tokens(&self, node: NodeId) -> Vec<String> {
        self.dom
            .attr(node, "rel")
            .unwrap_or_default()
            .split_whitespace()
            .map(|token| token.to_string())
            .collect::<Vec<_>>()
    }

    pub(crate) fn resolve_anchor_href(&self, node: NodeId) -> String {
        let raw = self.dom.attr(node, "href").unwrap_or_default();
        self.resolve_location_target_url(&raw)
    }

    pub(crate) fn anchor_location_parts(&self, node: NodeId) -> LocationParts {
        let href = self.resolve_anchor_href(node);
        LocationParts::parse(&href).unwrap_or_else(|| self.current_location_parts())
    }

    pub(crate) fn set_anchor_url_property(
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
        self.trace_state.enabled = enabled;
    }

    pub fn take_trace_logs(&mut self) -> Vec<String> {
        self.trace_state.logs.drain(..).collect()
    }

}
