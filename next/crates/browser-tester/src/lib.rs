use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;

pub use bt_dom::{DomStore, NodeId};
use bt_runtime::SessionError;
pub use bt_runtime::{
    ClipboardMocks, CloseCall, CloseMocks, DebugState, DialogMocks, DownloadCapture, DownloadMocks,
    FetchCall, FetchErrorRule, FetchMocks, FetchResponse, FetchResponseRule, FileInputMocks,
    FileInputSelection, LocationMocks, MatchMediaCall, MatchMediaListenerCall, MatchMediaMocks,
    MockRegistry, OpenCall, OpenMocks, PrintCall, PrintMocks, ScheduledTimer, Scheduler,
    ScrollCall, ScrollMethod, ScrollMocks, Session, SessionConfig, StorageSeeds,
};
pub use bt_script::{HostBindings, KeyboardEventInit, ScriptError, ScriptErrorKind, ScriptRuntime};

macro_rules! message_error {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            message: String,
        }

        impl $name {
            pub fn new(message: impl Into<String>) -> Self {
                Self {
                    message: message.into(),
                }
            }

            pub fn message(&self) -> &str {
                &self.message
            }

            pub fn contains(&self, needle: &str) -> bool {
                self.message.contains(needle)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.message)
            }
        }

        impl StdError for $name {}
    };
}

message_error!(HtmlParseError);
message_error!(JsSetupError);
message_error!(SelectorError);
message_error!(DomError);
message_error!(EventError);
message_error!(TimerError);
message_error!(MockError);
message_error!(AssertionError);

#[derive(Debug)]
pub enum Error {
    HtmlParse(HtmlParseError),
    JsSetup(JsSetupError),
    Script(ScriptError),
    ScriptParse(String),
    ScriptRuntime(String),
    Selector(SelectorError),
    Dom(DomError),
    Event(EventError),
    Timer(TimerError),
    Mock(MockError),
    Assertion(AssertionError),
    Unsupported(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HtmlParse(err) => write!(f, "HTML parse error: {err}"),
            Self::JsSetup(err) => write!(f, "JS setup error: {err}"),
            Self::Script(err) => write!(f, "Script error: {err}"),
            Self::ScriptParse(message) => {
                write!(f, "Script error (parse): {message}")
            }
            Self::ScriptRuntime(message) => {
                write!(f, "Script error (runtime): {message}")
            }
            Self::Selector(err) => write!(f, "Selector error: {err}"),
            Self::Dom(err) => write!(f, "DOM error: {err}"),
            Self::Event(err) => write!(f, "Event error: {err}"),
            Self::Timer(err) => write!(f, "Timer error: {err}"),
            Self::Mock(err) => write!(f, "Mock error: {err}"),
            Self::Assertion(err) => write!(f, "Assertion error: {err}"),
            Self::Unsupported(message) => f.write_str(message),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::HtmlParse(err) => Some(err),
            Self::JsSetup(err) => Some(err),
            Self::Script(err) => Some(err),
            Self::ScriptParse(_) | Self::ScriptRuntime(_) => None,
            Self::Selector(err) => Some(err),
            Self::Dom(err) => Some(err),
            Self::Event(err) => Some(err),
            Self::Timer(err) => Some(err),
            Self::Mock(err) => Some(err),
            Self::Assertion(err) => Some(err),
            Self::Unsupported(_) => None,
        }
    }
}

impl From<ScriptError> for Error {
    fn from(value: ScriptError) -> Self {
        Self::Script(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn map_session_error(error: SessionError) -> Error {
    match error {
        SessionError::HtmlParse(message) => Error::HtmlParse(HtmlParseError::new(message)),
        SessionError::Script(error) => match error.kind() {
            ScriptErrorKind::Parse => Error::ScriptParse(error.to_string()),
            ScriptErrorKind::Runtime => Error::ScriptRuntime(error.to_string()),
        },
        SessionError::Selector(message) => Error::Selector(SelectorError::new(message)),
        SessionError::Dom(message) => Error::Dom(DomError::new(message)),
        SessionError::Event(message) => Error::Event(EventError::new(message)),
        SessionError::Mock(message) => Error::Mock(MockError::new(message)),
    }
}

#[derive(Clone, Debug, Default)]
pub struct HarnessBuilder {
    url: Option<String>,
    html: Option<String>,
    local_storage: BTreeMap<String, String>,
    match_media: BTreeMap<String, bool>,
    open_failure: Option<String>,
    close_failure: Option<String>,
    print_failure: Option<String>,
    scroll_failure: Option<String>,
}

impl HarnessBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    pub fn local_storage<I, K, V>(mut self, entries: I) -> Self
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: ToString,
        V: ToString,
    {
        self.local_storage = entries
            .into_iter()
            .map(|entry| {
                let (key, value): &(K, V) = entry.borrow();
                (key.to_string(), value.to_string())
            })
            .collect();
        self
    }

    pub fn match_media<I, K>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = (K, bool)>,
        K: Into<String>,
    {
        self.match_media = entries
            .into_iter()
            .map(|(query, matches)| (query.into(), matches))
            .collect();
        self
    }

    pub fn open_failure(mut self, message: impl Into<String>) -> Self {
        self.open_failure = Some(message.into());
        self
    }

    pub fn close_failure(mut self, message: impl Into<String>) -> Self {
        self.close_failure = Some(message.into());
        self
    }

    pub fn print_failure(mut self, message: impl Into<String>) -> Self {
        self.print_failure = Some(message.into());
        self
    }

    pub fn scroll_failure(mut self, message: impl Into<String>) -> Self {
        self.scroll_failure = Some(message.into());
        self
    }

    pub fn build(self) -> Result<Harness> {
        let mut local_storage = self.local_storage;
        for (query, matches) in self.match_media {
            local_storage.insert(
                format!("__browser_tester_match_media__{query}"),
                matches.to_string(),
            );
        }
        if let Some(message) = self.open_failure {
            local_storage.insert("__browser_tester_open_failure__".to_string(), message);
        }
        if let Some(message) = self.close_failure {
            local_storage.insert("__browser_tester_close_failure__".to_string(), message);
        }
        if let Some(message) = self.print_failure {
            local_storage.insert("__browser_tester_print_failure__".to_string(), message);
        }
        if let Some(message) = self.scroll_failure {
            local_storage.insert("__browser_tester_scroll_failure__".to_string(), message);
        }
        let config = SessionConfig {
            url: self.url.unwrap_or_else(|| SessionConfig::default().url),
            html: self.html,
            local_storage,
        };
        let session = Session::new(config).map_err(map_session_error)?;
        Ok(Harness { session })
    }
}

#[derive(Clone, Debug)]
pub struct Harness {
    session: Session,
}

impl Harness {
    pub fn builder() -> HarnessBuilder {
        HarnessBuilder::new()
    }

    pub fn from_html(html: impl Into<String>) -> Result<Self> {
        Self::builder().html(html).build()
    }

    pub fn from_html_with_url(url: impl Into<String>, html: impl Into<String>) -> Result<Self> {
        Self::builder().url(url).html(html).build()
    }

    pub fn from_html_with_local_storage<I, K, V>(
        html: impl Into<String>,
        entries: I,
    ) -> Result<Self>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: ToString,
        V: ToString,
    {
        Self::builder().html(html).local_storage(entries).build()
    }

    pub fn from_html_with_url_and_local_storage<I, K, V>(
        url: impl Into<String>,
        html: impl Into<String>,
        entries: I,
    ) -> Result<Self>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: ToString,
        V: ToString,
    {
        Self::builder()
            .url(url)
            .html(html)
            .local_storage(entries)
            .build()
    }

    pub fn now_ms(&self) -> i64 {
        self.session.scheduler().now_ms()
    }

    pub fn advance_time(&mut self, delta_ms: i64) -> Result<()> {
        if delta_ms < 0 {
            return Err(Error::Timer(TimerError::new(
                "advance_time requires a non-negative delta",
            )));
        }
        self.session
            .advance_time(delta_ms)
            .map_err(map_session_error)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.session.flush().map_err(map_session_error)?;
        Ok(())
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.session.set_random_seed(seed);
    }

    pub fn enable_trace(&mut self, enabled: bool) {
        self.session.enable_trace(enabled);
    }

    pub fn set_trace_stderr(&mut self, enabled: bool) {
        self.session.set_trace_stderr(enabled);
    }

    pub fn set_trace_events(&mut self, enabled: bool) {
        self.session.set_trace_events(enabled);
    }

    pub fn set_trace_timers(&mut self, enabled: bool) {
        self.session.set_trace_timers(enabled);
    }

    pub fn set_trace_log_limit(&mut self, max_entries: usize) -> Result<()> {
        self.session
            .set_trace_log_limit(max_entries)
            .map_err(map_session_error)?;
        Ok(())
    }

    pub fn take_trace_logs(&mut self) -> Vec<String> {
        self.session.take_trace_logs()
    }

    pub fn set_timer_step_limit(&mut self, max_steps: usize) -> Result<()> {
        self.session
            .set_timer_step_limit(max_steps)
            .map_err(map_session_error)?;
        Ok(())
    }

    pub fn click(&mut self, selector: &str) -> Result<()> {
        let node_id = self.resolve_action_target(selector)?;
        self.session.click_node(node_id).map_err(map_session_error)
    }

    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()> {
        let node_id = self.resolve_action_target(selector)?;
        self.session
            .type_text_node(node_id, text)
            .map_err(map_session_error)
    }

    pub fn set_checked(&mut self, selector: &str, checked: bool) -> Result<()> {
        let node_id = self.resolve_action_target(selector)?;
        self.session
            .set_checked_node(node_id, checked)
            .map_err(map_session_error)
    }

    pub fn set_select_value(&mut self, selector: &str, value: &str) -> Result<()> {
        let node_id = self.resolve_action_target(selector)?;
        self.session
            .set_select_value_node(node_id, value)
            .map_err(map_session_error)
    }

    pub fn focus(&mut self, selector: &str) -> Result<()> {
        let node_id = self.resolve_action_target(selector)?;
        self.session.focus_node(node_id).map_err(map_session_error)
    }

    pub fn blur(&mut self, selector: &str) -> Result<()> {
        let node_id = self.resolve_action_target(selector)?;
        self.session.blur_node(node_id).map_err(map_session_error)
    }

    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()> {
        let node_id = self.resolve_action_target(selector)?;
        self.session
            .dispatch_node(node_id, event)
            .map_err(map_session_error)
    }

    pub fn dispatch_keyboard(
        &mut self,
        selector: &str,
        event: &str,
        init: KeyboardEventInit,
    ) -> Result<()> {
        self.session
            .dispatch_keyboard(selector, event, init)
            .map_err(map_session_error)
    }

    pub fn submit(&mut self, selector: &str) -> Result<()> {
        let node_id = self.resolve_action_target(selector)?;
        self.session.submit_node(node_id).map_err(map_session_error)
    }

    pub fn alert(&mut self, message: &str) -> Result<()> {
        self.session.alert(message);
        Ok(())
    }

    pub fn print(&mut self) -> Result<()> {
        self.session.print().map_err(map_session_error)
    }

    pub fn scroll_to(&mut self, x: i64, y: i64) -> Result<()> {
        self.session.scroll_to(x, y).map_err(map_session_error)
    }

    pub fn scroll_by(&mut self, x: i64, y: i64) -> Result<()> {
        self.session.scroll_by(x, y).map_err(map_session_error)
    }

    pub fn close(&mut self) -> Result<()> {
        self.session.close().map_err(map_session_error)
    }

    pub fn open(&mut self, url: &str) -> Result<()> {
        self.session
            .open(Some(url), None, None)
            .map_err(map_session_error)
    }

    pub fn confirm(&mut self, message: &str) -> Result<bool> {
        self.session.confirm(message).map_err(map_session_error)
    }

    pub fn prompt(&mut self, message: &str) -> Result<Option<String>> {
        self.session.prompt(message).map_err(map_session_error)
    }

    pub fn read_clipboard(&self) -> Result<String> {
        self.session.read_clipboard().map_err(map_session_error)
    }

    pub fn write_clipboard(&mut self, text: &str) -> Result<()> {
        self.session.write_clipboard(text).map_err(map_session_error)?;
        Ok(())
    }

    pub fn set_clipboard_text(&mut self, text: &str) {
        self.session.mocks_mut().clipboard_mut().seed_text(text);
    }

    pub fn set_clipboard_read_error(&mut self, error: Option<&str>) {
        self.session
            .mocks_mut()
            .clipboard_mut()
            .set_read_error(error);
    }

    pub fn set_clipboard_write_error(&mut self, error: Option<&str>) {
        self.session
            .mocks_mut()
            .clipboard_mut()
            .set_write_error(error);
    }

    pub fn clear_clipboard_errors(&mut self) {
        self.session.mocks_mut().clipboard_mut().clear_errors();
    }

    pub fn capture_download(&mut self, file_name: &str, bytes: impl Into<Vec<u8>>) -> Result<()> {
        self.session
            .capture_download(file_name, bytes)
            .map_err(map_session_error)
    }

    pub fn take_downloads(&mut self) -> Vec<DownloadCapture> {
        self.session.take_downloads()
    }

    pub fn take_print_call_count(&mut self) -> usize {
        self.session.mocks_mut().print_mut().take().len()
    }

    pub fn set_match_media_mock(&mut self, query: &str, matches: bool) {
        self.session
            .mocks_mut()
            .match_media_mut()
            .respond_matches(query.to_string(), matches);
    }

    pub fn take_match_media_calls(&mut self) -> Vec<String> {
        self.session
            .mocks_mut()
            .match_media_mut()
            .take_calls()
            .into_iter()
            .map(|call| call.query)
            .collect()
    }

    pub fn clipboard_text(&self) -> String {
        self.session
            .mocks()
            .clipboard()
            .seeded_text()
            .unwrap_or_default()
            .to_string()
    }

    pub fn pending_timers(&self) -> &[ScheduledTimer] {
        self.session.scheduler().pending_timers()
    }

    pub fn run_due_timers(&mut self) -> Result<usize> {
        self.session.run_due_timers().map_err(map_session_error)
    }

    pub fn fetch(&mut self, url: &str) -> Result<FetchResponse> {
        self.session.fetch(url).map_err(map_session_error)
    }

    pub fn navigate(&mut self, url: &str) -> Result<()> {
        self.session.navigate(url).map_err(map_session_error)
    }

    pub fn set_files<I, K>(&mut self, selector: &str, files: I) -> Result<()>
    where
        I: IntoIterator<Item = K>,
        K: Into<String>,
    {
        let node_id = self.resolve_action_target(selector)?;
        self.session
            .set_files_node(node_id, selector, files)
            .map_err(map_session_error)
    }

    pub fn assert_text(&self, selector: &str, expected: &str) -> Result<()> {
        let matches = self
            .session
            .dom()
            .select(selector)
            .map_err(|message| Error::Selector(SelectorError::new(message)))?;

        let Some(node_id) = matches.first().copied() else {
            return Err(Error::Assertion(AssertionError::new(format!(
                "expected selector `{}` to match at least one node\nDOM:\n{}",
                selector,
                self.session.dom().dump_dom()
            ))));
        };

        let actual = self.session.dom().text_content_for_node(node_id);
        if actual != expected {
            return Err(Error::Assertion(AssertionError::new(format!(
                "expected selector `{selector}` to have text `{expected}`, got `{actual}`\nDOM:\n{}",
                self.session.dom().dump_dom()
            ))));
        }

        Ok(())
    }

    pub fn assert_value(&self, selector: &str, expected: &str) -> Result<()> {
        let matches = self
            .session
            .dom()
            .select(selector)
            .map_err(|message| Error::Selector(SelectorError::new(message)))?;

        let Some(node_id) = matches.first().copied() else {
            return Err(Error::Assertion(AssertionError::new(format!(
                "expected selector `{}` to match at least one node\nDOM:\n{}",
                selector,
                self.session.dom().dump_dom()
            ))));
        };

        let actual = self.session.dom().value_for_node(node_id);
        if actual != expected {
            return Err(Error::Assertion(AssertionError::new(format!(
                "expected selector `{selector}` to have value `{expected}`, got `{actual}`\nDOM:\n{}",
                self.session.dom().dump_dom()
            ))));
        }

        Ok(())
    }

    pub fn assert_checked(&self, selector: &str, expected: bool) -> Result<()> {
        let matches = self
            .session
            .dom()
            .select(selector)
            .map_err(|message| Error::Selector(SelectorError::new(message)))?;

        let Some(node_id) = matches.first().copied() else {
            return Err(Error::Assertion(AssertionError::new(format!(
                "expected selector `{}` to match at least one node\nDOM:\n{}",
                selector,
                self.session.dom().dump_dom()
            ))));
        };

        let Some(actual) = self.session.dom().checked_for_node(node_id) else {
            return Err(Error::Assertion(AssertionError::new(format!(
                "expected selector `{selector}` to refer to a checkable control\nDOM:\n{}",
                self.session.dom().dump_dom()
            ))));
        };

        if actual != expected {
            return Err(Error::Assertion(AssertionError::new(format!(
                "expected selector `{selector}` to be checked `{expected}`, got `{actual}`\nDOM:\n{}",
                self.session.dom().dump_dom()
            ))));
        }

        Ok(())
    }

    pub fn assert_exists(&self, selector: &str) -> Result<()> {
        let matches = self
            .session
            .dom()
            .select(selector)
            .map_err(|message| Error::Selector(SelectorError::new(message)))?;

        if matches.is_empty() {
            return Err(Error::Assertion(AssertionError::new(format!(
                "expected selector `{}` to match at least one node\nDOM:\n{}",
                selector,
                self.session.dom().dump_dom()
            ))));
        }

        Ok(())
    }

    pub fn mocks_mut(&mut self) -> MockRegistryView<'_> {
        MockRegistryView {
            inner: self.session.mocks_mut(),
        }
    }

    pub fn debug(&self) -> DebugView<'_> {
        DebugView {
            session: &self.session,
        }
    }

    pub fn dump_dom(&self, selector: &str) -> Result<String> {
        let matches = self
            .session
            .dom()
            .select(selector)
            .map_err(|message| Error::Selector(SelectorError::new(message)))?;

        let Some(node_id) = matches.first().copied() else {
            return Err(Error::Assertion(AssertionError::new(format!(
                "expected selector `{}` to match at least one node\nDOM:\n{}",
                selector,
                self.session.dom().dump_dom()
            ))));
        };

        self.session
            .dom()
            .outer_html_for_node(node_id)
            .map_err(|message| Error::Dom(DomError::new(message)))
    }

    fn resolve_action_target(&self, selector: &str) -> Result<NodeId> {
        let matches = self
            .session
            .dom()
            .select(selector)
            .map_err(|message| Error::Selector(SelectorError::new(message)))?;

        let Some(node_id) = matches.first().copied() else {
            return Err(Error::Dom(DomError::new(format!(
                "selector `{selector}` did not match any elements"
            ))));
        };

        Ok(node_id)
    }
}

pub struct MockRegistryView<'a> {
    inner: &'a mut MockRegistry,
}

impl<'a> MockRegistryView<'a> {
    pub fn fetch(&mut self) -> &mut FetchMocks {
        self.inner.fetch_mut()
    }

    pub fn dialogs(&mut self) -> &mut DialogMocks {
        self.inner.dialogs_mut()
    }

    pub fn clipboard(&mut self) -> &mut ClipboardMocks {
        self.inner.clipboard_mut()
    }

    pub fn location(&mut self) -> &mut LocationMocks {
        self.inner.location_mut()
    }

    pub fn match_media(&mut self) -> &mut MatchMediaMocks {
        self.inner.match_media_mut()
    }

    pub fn open(&mut self) -> &mut OpenMocks {
        self.inner.open_mut()
    }

    pub fn close(&mut self) -> &mut CloseMocks {
        self.inner.close_mut()
    }

    pub fn print(&mut self) -> &mut PrintMocks {
        self.inner.print_mut()
    }

    pub fn scroll(&mut self) -> &mut ScrollMocks {
        self.inner.scroll_mut()
    }

    pub fn downloads(&mut self) -> &mut DownloadMocks {
        self.inner.downloads_mut()
    }

    pub fn file_input(&mut self) -> &mut FileInputMocks {
        self.inner.file_input_mut()
    }

    pub fn storage(&mut self) -> &mut StorageSeeds {
        self.inner.storage_mut()
    }

    pub fn reset_all(&mut self) {
        self.inner.reset_all();
    }
}

pub struct DebugView<'a> {
    session: &'a Session,
}

impl<'a> DebugView<'a> {
    pub fn url(&self) -> &str {
        self.session
            .mocks()
            .location()
            .current_url()
            .unwrap_or(self.session.config().url.as_str())
    }

    pub fn source_html(&self) -> Option<&str> {
        self.session.dom().source_html()
    }

    pub fn dom_node_count(&self) -> usize {
        self.session.dom().node_count()
    }

    pub fn trace_enabled(&self) -> bool {
        self.session.debug().trace_enabled()
    }

    pub fn local_storage(&self) -> &BTreeMap<String, String> {
        self.session.mocks().storage().local()
    }

    pub fn dump_dom(&self) -> String {
        self.session.dom().dump_dom()
    }
}
