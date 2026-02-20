use super::*;

#[derive(Debug)]
pub struct Harness {
    pub(crate) dom: Dom,
    pub(crate) listeners: ListenerStore,
    pub(crate) dom_runtime: DomRuntimeState,
    pub(crate) script_runtime: ScriptRuntimeState,
    pub(crate) document_url: String,
    pub(crate) location_history: LocationHistoryState,
    pub(crate) scheduler: SchedulerState,
    pub(crate) promise_runtime: PromiseRuntimeState,
    pub(crate) symbol_runtime: SymbolRuntimeState,
    pub(crate) browser_apis: BrowserApiState,
    pub(crate) rng_state: u64,
    pub(crate) platform_mocks: PlatformMockState,
    pub(crate) trace_state: TraceState,
}

#[derive(Debug)]
pub struct MockWindow {
    pub(crate) pages: Vec<MockPage>,
    pub(crate) current: usize,
}

#[derive(Debug)]
pub struct MockPage {
    pub(crate) harness: Harness,
}

impl MockWindow {
    pub(crate) fn with_current_harness_mut<R>(
        &mut self,
        f: impl FnOnce(&mut Harness) -> Result<R>,
    ) -> Result<R> {
        let page = self
            .pages
            .get_mut(self.current)
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))?;
        f(&mut page.harness)
    }

    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            current: 0,
        }
    }

    pub fn open_page(&mut self, url: &str, html: &str) -> Result<usize> {
        let harness = Harness::from_html_with_url(url, html)?;
        if let Some(index) = self
            .pages
            .iter()
            .position(|page| page.harness.document_url == url)
        {
            self.pages[index] = MockPage { harness };
            self.current = index;
            Ok(index)
        } else {
            self.pages.push(MockPage { harness });
            self.current = self.pages.len() - 1;
            Ok(self.current)
        }
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn switch_to(&mut self, url: &str) -> Result<()> {
        let index = self
            .pages
            .iter()
            .position(|page| page.harness.document_url == url)
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown page: {url}")))?;
        self.current = index;
        Ok(())
    }

    pub fn switch_to_index(&mut self, index: usize) -> Result<()> {
        if index >= self.pages.len() {
            return Err(Error::ScriptRuntime(format!(
                "page index out of range: {index}"
            )));
        }
        self.current = index;
        Ok(())
    }

    pub fn current_url(&self) -> Result<&str> {
        self.pages
            .get(self.current)
            .map(|page| page.harness.document_url.as_str())
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    pub fn current_document_mut(&mut self) -> Result<&mut Harness> {
        self.pages
            .get_mut(self.current)
            .map(|page| &mut page.harness)
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    pub fn current_document(&self) -> Result<&Harness> {
        self.pages
            .get(self.current)
            .map(|page| &page.harness)
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    pub fn with_current_document<R>(
        &mut self,
        f: impl FnOnce(&mut Harness) -> Result<R>,
    ) -> Result<R> {
        self.with_current_harness_mut(f)
    }

    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.type_text(selector, text))
    }

    pub fn set_checked(&mut self, selector: &str, checked: bool) -> Result<()> {
        self.with_current_harness_mut(|page| page.set_checked(selector, checked))
    }

    pub fn click(&mut self, selector: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.click(selector))
    }

    pub fn submit(&mut self, selector: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.submit(selector))
    }

    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.dispatch(selector, event))
    }

    pub fn assert_text(&mut self, selector: &str, expected: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.assert_text(selector, expected))
    }

    pub fn assert_value(&mut self, selector: &str, expected: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.assert_value(selector, expected))
    }

    pub fn assert_checked(&mut self, selector: &str, expected: bool) -> Result<()> {
        self.with_current_harness_mut(|page| page.assert_checked(selector, expected))
    }

    pub fn assert_exists(&mut self, selector: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.assert_exists(selector))
    }

    pub fn take_trace_logs(&mut self) -> Result<Vec<String>> {
        self.with_current_harness_mut(|page| Ok(page.take_trace_logs()))
    }
}

impl MockPage {
    pub fn url(&self) -> &str {
        self.harness.document_url.as_str()
    }

    pub fn harness(&self) -> &Harness {
        &self.harness
    }

    pub fn harness_mut(&mut self) -> &mut Harness {
        &mut self.harness
    }
}
