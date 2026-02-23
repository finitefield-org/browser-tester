use super::*;

impl Harness {
    pub fn from_html(html: &str) -> Result<Self> {
        Self::from_html_impl("about:blank", html, &[])
    }

    pub fn from_html_with_url(url: &str, html: &str) -> Result<Self> {
        Self::from_html_impl(url, html, &[])
    }

    pub fn from_html_with_local_storage(
        html: &str,
        initial_local_storage: &[(&str, &str)],
    ) -> Result<Self> {
        Self::from_html_impl("about:blank", html, initial_local_storage)
    }

    pub fn from_html_with_url_and_local_storage(
        url: &str,
        html: &str,
        initial_local_storage: &[(&str, &str)],
    ) -> Result<Self> {
        Self::from_html_impl(url, html, initial_local_storage)
    }

    pub(crate) fn from_html_impl(
        url: &str,
        html: &str,
        initial_local_storage: &[(&str, &str)],
    ) -> Result<Self> {
        let ParseOutput { mut dom, scripts } = parse_html(html)?;
        if scripts
            .iter()
            .any(|script| script.code.contains("document.body"))
        {
            let _ = dom.ensure_document_body_element()?;
        }
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
        harness.seed_initial_local_storage(initial_local_storage);

        for script in scripts {
            harness.compile_and_register_script(&script.code, script.is_module)?;
        }

        Ok(harness)
    }

    pub(crate) fn seed_initial_local_storage(&mut self, initial_local_storage: &[(&str, &str)]) {
        if initial_local_storage.is_empty() {
            return;
        }

        let mut pairs = Vec::new();
        for (key, value) in initial_local_storage {
            if let Some((_, stored)) = pairs.iter_mut().find(|(name, _)| name == key) {
                *stored = (*value).to_string();
            } else {
                pairs.push(((*key).to_string(), (*value).to_string()));
            }
        }
        Self::set_storage_pairs(
            &mut self.browser_apis.local_storage_object.borrow_mut(),
            &pairs,
        );
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
        self.browser_apis
            .url_constructor_properties
            .borrow_mut()
            .clear();
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
        let iterator_constructor = self.new_iterator_constructor_value();
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
            &iterator_constructor,
            &url_constructor,
            &html_element_constructor,
            &html_input_element_constructor,
            &local_storage,
        );

        let window = Value::Object(self.dom_runtime.window_object.clone());
        let document = Value::Object(self.dom_runtime.document_object.clone());

        self.script_runtime
            .env
            .insert("document".to_string(), document);
        self.script_runtime
            .env
            .insert("navigator".to_string(), navigator.clone());
        self.script_runtime
            .env
            .insert("clientInformation".to_string(), navigator.clone());
        self.script_runtime.env.insert("Intl".to_string(), intl);
        self.script_runtime
            .env
            .insert("String".to_string(), string_constructor);
        self.script_runtime
            .env
            .insert("Boolean".to_string(), boolean_constructor);
        self.script_runtime
            .env
            .insert("Iterator".to_string(), iterator_constructor);
        self.script_runtime
            .env
            .insert("URL".to_string(), url_constructor);
        self.script_runtime
            .env
            .insert("HTMLElement".to_string(), html_element_constructor);
        self.script_runtime.env.insert(
            "HTMLInputElement".to_string(),
            html_input_element_constructor,
        );
        self.script_runtime
            .env
            .insert("location".to_string(), location);
        self.script_runtime
            .env
            .insert("history".to_string(), history);
        self.script_runtime
            .env
            .insert("localStorage".to_string(), local_storage);
        self.script_runtime
            .env
            .insert("window".to_string(), window.clone());
        self.script_runtime
            .env
            .insert("self".to_string(), window.clone());
        self.script_runtime
            .env
            .insert("top".to_string(), window.clone());
        self.script_runtime
            .env
            .insert("parent".to_string(), window.clone());
        self.script_runtime.env.insert("frames".to_string(), window);
        self.script_runtime
            .env
            .insert(INTERNAL_SCOPE_DEPTH_KEY.to_string(), Value::Number(0));
    }
}
