use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;

pub use bt_dom::{DomStore, NodeId};
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
        Ok(Harness {
            session: Session::new(config),
        })
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

    pub fn click(&mut self, _selector: &str) -> Result<()> {
        Err(Error::Unsupported(
            "click is planned for Phase 3 after selector and event support land",
        ))
    }

    pub fn type_text(&mut self, _selector: &str, _text: &str) -> Result<()> {
        Err(Error::Unsupported(
            "type_text is planned for Phase 3 after DOM and form support land",
        ))
    }

    pub fn set_checked(&mut self, _selector: &str, _checked: bool) -> Result<()> {
        Err(Error::Unsupported(
            "set_checked is planned for Phase 3 after form-control support lands",
        ))
    }

    pub fn set_select_value(&mut self, _selector: &str, _value: &str) -> Result<()> {
        Err(Error::Unsupported(
            "set_select_value is planned for Phase 3 after form-control support lands",
        ))
    }

    pub fn focus(&mut self, _selector: &str) -> Result<()> {
        Err(Error::Unsupported(
            "focus is planned for Phase 3 after selector and event support land",
        ))
    }

    pub fn blur(&mut self, _selector: &str) -> Result<()> {
        Err(Error::Unsupported(
            "blur is planned for Phase 3 after selector and event support land",
        ))
    }

    pub fn dispatch(&mut self, _selector: &str, _event: &str) -> Result<()> {
        Err(Error::Unsupported(
            "dispatch is planned for Phase 3 after event support lands",
        ))
    }

    pub fn assert_text(&self, _selector: &str, _expected: &str) -> Result<()> {
        Err(Error::Unsupported(
            "assert_text is planned for Phase 1 after selector support lands",
        ))
    }

    pub fn assert_value(&self, _selector: &str, _expected: &str) -> Result<()> {
        Err(Error::Unsupported(
            "assert_value is planned for Phase 3 after form-control support lands",
        ))
    }

    pub fn assert_checked(&self, _selector: &str, _expected: bool) -> Result<()> {
        Err(Error::Unsupported(
            "assert_checked is planned for Phase 3 after form-control support lands",
        ))
    }

    pub fn assert_exists(&self, _selector: &str) -> Result<()> {
        Err(Error::Unsupported(
            "assert_exists is planned for Phase 1 after selector support lands",
        ))
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
}
