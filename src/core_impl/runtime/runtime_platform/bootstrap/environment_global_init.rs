use super::*;

#[path = "environment_global_init/environment_global_bindings_init.rs"]
mod environment_global_bindings_init;

/// Constructors for creating a [`Harness`] from HTML input.
impl Harness {
    const FROM_HTML_STACK_SIZE: usize = 32 * 1024 * 1024;

    /// Build a harness from HTML using `about:blank` as the document URL.
    pub fn from_html(html: &str) -> Result<Self> {
        Self::from_html_impl("about:blank", html, &[])
    }

    /// Build a harness from HTML with an explicit document URL.
    pub fn from_html_with_url(url: &str, html: &str) -> Result<Self> {
        Self::from_html_impl(url, html, &[])
    }

    /// Build a harness from HTML and seed `localStorage` deterministically.
    pub fn from_html_with_local_storage(
        html: &str,
        initial_local_storage: &[(&str, &str)],
    ) -> Result<Self> {
        Self::from_html_impl("about:blank", html, initial_local_storage)
    }

    /// Build a harness from HTML with an explicit URL and seeded `localStorage`.
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
        stacker::grow(Self::FROM_HTML_STACK_SIZE, || {
            Self::from_html_impl_on_grown_stack(url, html, initial_local_storage)
        })
    }

    fn from_html_impl_on_grown_stack(
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
        let normalized_url = Self::resolve_url_string(url, None).unwrap_or_else(|| url.to_string());
        let mut harness = Self {
            dom,
            listeners: ListenerStore::default(),
            dom_runtime: DomRuntimeState::default(),
            script_runtime: ScriptRuntimeState::default(),
            document_url: normalized_url.clone(),
            location_history: LocationHistoryState::new(&normalized_url),
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
        harness.dom_runtime.document_ready_state = "loading".to_string();

        for script in scripts {
            harness.compile_and_register_script(&script.code, script.is_module)?;
        }
        harness.finalize_document_ready_state_with_dom_content_loaded()?;

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
}
