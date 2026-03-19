use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;

pub use bt_dom::{DomStore, NodeId};
use bt_runtime::SessionError;
pub use bt_runtime::{
    ClipboardMocks, DebugState, DialogMocks, DownloadCapture, DownloadMocks, FetchCall,
    FetchErrorRule, FetchMocks, FetchResponseRule, FileInputMocks, FileInputSelection,
    LocationMocks, MockRegistry, ScheduledTimer, Scheduler, Session, SessionConfig, StorageSeeds,
};
pub use bt_script::{HostBindings, ScriptError, ScriptRuntime};

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
        SessionError::Script(error) => Error::Script(error),
        SessionError::Selector(message) => Error::Selector(SelectorError::new(message)),
        SessionError::Dom(message) => Error::Dom(DomError::new(message)),
        SessionError::Event(message) => Error::Event(EventError::new(message)),
    }
}

#[derive(Clone, Debug, Default)]
pub struct HarnessBuilder {
    url: Option<String>,
    html: Option<String>,
    local_storage: BTreeMap<String, String>,
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
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.local_storage = entries
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();
        self
    }

    pub fn build(self) -> Result<Harness> {
        let config = SessionConfig {
            url: self.url.unwrap_or_else(|| SessionConfig::default().url),
            html: self.html,
            local_storage: self.local_storage,
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
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::builder().html(html).local_storage(entries).build()
    }

    pub fn from_html_with_url_and_local_storage<I, K, V>(
        url: impl Into<String>,
        html: impl Into<String>,
        entries: I,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
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
        self.session.scheduler_mut().advance_time(delta_ms);
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.session.scheduler_mut().flush();
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

    pub fn submit(&mut self, selector: &str) -> Result<()> {
        let node_id = self.resolve_action_target(selector)?;
        self.session.submit_node(node_id).map_err(map_session_error)
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
        &self.session.config().url
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
