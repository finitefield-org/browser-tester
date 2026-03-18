use super::*;

/// Deterministic browser-like test harness.
///
/// `Harness` is the main public entry point of the crate. It lets tests load
/// HTML, drive user-like interactions, control fake time, seed deterministic
/// mocks, and assert DOM state from Rust.
///
/// Public API categories:
///
/// - Constructors: [`from_html`](Self::from_html),
///   [`from_html_with_url`](Self::from_html_with_url),
///   [`from_html_with_local_storage`](Self::from_html_with_local_storage),
///   [`from_html_with_url_and_local_storage`](Self::from_html_with_url_and_local_storage)
/// - Actions: [`type_text`](Self::type_text), [`set_select_value`](Self::set_select_value),
///   [`set_input_files`](Self::set_input_files), [`set_checked`](Self::set_checked),
///   [`click`](Self::click), [`focus`](Self::focus), [`blur`](Self::blur),
///   [`press_enter`](Self::press_enter), [`copy`](Self::copy), [`paste`](Self::paste),
///   [`cut`](Self::cut), [`submit`](Self::submit), [`dispatch`](Self::dispatch),
///   [`dispatch_keyboard`](Self::dispatch_keyboard)
/// - Assertions and inspection: [`assert_text`](Self::assert_text),
///   [`assert_value`](Self::assert_value), [`assert_checked`](Self::assert_checked),
///   [`assert_exists`](Self::assert_exists), [`dump_dom`](Self::dump_dom)
/// - Time and scheduler controls: [`now_ms`](Self::now_ms),
///   [`advance_time`](Self::advance_time), [`advance_time_to`](Self::advance_time_to),
///   [`run_due_timers`](Self::run_due_timers), [`run_next_timer`](Self::run_next_timer),
///   [`run_next_due_timer`](Self::run_next_due_timer), [`flush`](Self::flush),
///   [`pending_timers`](Self::pending_timers), [`clear_timer`](Self::clear_timer),
///   [`clear_all_timers`](Self::clear_all_timers)
/// - Determinism, mocks, and trace: [`set_random_seed`](Self::set_random_seed),
///   [`set_fetch_mock`](Self::set_fetch_mock), [`set_clipboard_text`](Self::set_clipboard_text),
///   [`set_location_mock_page`](Self::set_location_mock_page),
///   [`enable_trace`](Self::enable_trace), [`take_trace_logs`](Self::take_trace_logs)
///
/// See the crate README for the quick-start flow, `doc/mock-guide.md` for mock
/// examples, and `doc/capability-matrix.md` for the current support-level
/// classification.
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
/// Deterministic multi-page convenience wrapper around [`Harness`].
///
/// Use `MockWindow` when a test needs to manage more than one document while
/// still delegating actions and assertions to the current page's `Harness`.
pub struct MockWindow {
    pub(crate) pages: Vec<MockPage>,
    pub(crate) current: usize,
}

#[derive(Debug)]
/// A single page stored inside [`MockWindow`].
pub struct MockPage {
    pub(crate) harness: Harness,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Extra fields for [`Harness::dispatch_keyboard`].
pub struct KeyboardEventInit {
    pub key: String,
    pub code: Option<String>,
    pub location: i64,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub repeat: bool,
    pub is_composing: bool,
}

/// `MockWindow` constructors, page switching, actions, and assertions.
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

    /// Create an empty mock window.
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            current: 0,
        }
    }

    /// Open or replace a page and make it the current page.
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

    /// Return the number of pages currently stored in the window.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Switch the current page by URL.
    pub fn switch_to(&mut self, url: &str) -> Result<()> {
        let index = self
            .pages
            .iter()
            .position(|page| page.harness.document_url == url)
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown page: {url}")))?;
        self.current = index;
        Ok(())
    }

    /// Switch the current page by index.
    pub fn switch_to_index(&mut self, index: usize) -> Result<()> {
        if index >= self.pages.len() {
            return Err(Error::ScriptRuntime(format!(
                "page index out of range: {index}"
            )));
        }
        self.current = index;
        Ok(())
    }

    /// Return the current page URL.
    pub fn current_url(&self) -> Result<&str> {
        self.pages
            .get(self.current)
            .map(|page| page.harness.document_url.as_str())
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    /// Borrow the current page harness mutably.
    pub fn current_document_mut(&mut self) -> Result<&mut Harness> {
        self.pages
            .get_mut(self.current)
            .map(|page| &mut page.harness)
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    /// Borrow the current page harness immutably.
    pub fn current_document(&self) -> Result<&Harness> {
        self.pages
            .get(self.current)
            .map(|page| &page.harness)
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    /// Run a closure against the current page harness.
    pub fn with_current_document<R>(
        &mut self,
        f: impl FnOnce(&mut Harness) -> Result<R>,
    ) -> Result<R> {
        self.with_current_harness_mut(f)
    }

    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.type_text(selector, text))
    }

    pub fn set_select_value(&mut self, selector: &str, value: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.set_select_value(selector, value))
    }

    pub fn set_input_files(&mut self, selector: &str, files: &[MockFile]) -> Result<()> {
        self.with_current_harness_mut(|page| page.set_input_files(selector, files))
    }

    pub fn set_checked(&mut self, selector: &str, checked: bool) -> Result<()> {
        self.with_current_harness_mut(|page| page.set_checked(selector, checked))
    }

    pub fn click(&mut self, selector: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.click(selector))
    }

    pub fn press_enter(&mut self, selector: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.press_enter(selector))
    }

    pub fn copy(&mut self, selector: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.copy(selector))
    }

    pub fn paste(&mut self, selector: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.paste(selector))
    }

    pub fn cut(&mut self, selector: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.cut(selector))
    }

    pub fn submit(&mut self, selector: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.submit(selector))
    }

    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()> {
        self.with_current_harness_mut(|page| page.dispatch(selector, event))
    }

    pub fn dispatch_keyboard(
        &mut self,
        selector: &str,
        event: &str,
        init: KeyboardEventInit,
    ) -> Result<()> {
        self.with_current_harness_mut(move |page| page.dispatch_keyboard(selector, event, init))
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

    pub fn take_downloads(&mut self) -> Result<Vec<DownloadArtifact>> {
        self.with_current_harness_mut(|page| Ok(page.take_downloads()))
    }

    pub fn take_clipboard_writes(&mut self) -> Result<Vec<ClipboardWriteArtifact>> {
        self.with_current_harness_mut(|page| Ok(page.take_clipboard_writes()))
    }
}

/// Accessors for a page stored inside [`MockWindow`].
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
