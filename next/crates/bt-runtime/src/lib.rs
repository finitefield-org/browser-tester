use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;

use bt_dom::{
    DomStore, ElementData, HTML_NAMESPACE_URI, MATHML_NAMESPACE_URI, NodeId, NodeKind,
    SVG_NAMESPACE_URI,
};
use bt_script::{
    ElementHandle, EventPhase, HostBindings, HtmlCollectionScope, HtmlCollectionTarget,
    ListenerTarget, NodeHandle, ScriptError, ScriptEventHandle, ScriptFunction, ScriptRuntime,
    ScriptValue,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionConfig {
    pub url: String,
    pub html: Option<String>,
    pub local_storage: BTreeMap<String, String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            url: "https://app.local/".to_string(),
            html: None,
            local_storage: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduledTimer {
    pub id: u64,
    pub at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scheduler {
    now_ms: i64,
    timers: Vec<ScheduledTimer>,
    microtasks: usize,
    next_timer_id: u64,
    step_limit: usize,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            now_ms: 0,
            timers: Vec::new(),
            microtasks: 0,
            next_timer_id: 1,
            step_limit: 10_000,
        }
    }
}

impl Scheduler {
    pub fn now_ms(&self) -> i64 {
        self.now_ms
    }

    pub fn advance_time(&mut self, delta_ms: i64) {
        self.now_ms += delta_ms;
        let _ = self.run_due_timers();
    }

    pub fn advance_time_to(&mut self, target_ms: i64) {
        self.now_ms = self.now_ms.max(target_ms);
        let _ = self.run_due_timers();
    }

    pub fn queue_timer(&mut self, at_ms: i64) -> u64 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        self.timers.push(ScheduledTimer { id, at_ms });
        self.timers.sort_by_key(|timer| (timer.at_ms, timer.id));
        id
    }

    pub fn pending_timers(&self) -> &[ScheduledTimer] {
        &self.timers
    }

    pub fn run_due_timers(&mut self) -> Vec<ScheduledTimer> {
        let split = self
            .timers
            .iter()
            .position(|timer| timer.at_ms > self.now_ms)
            .unwrap_or(self.timers.len());
        self.timers.drain(..split).collect()
    }

    pub fn queue_microtask(&mut self) {
        self.microtasks += 1;
    }

    pub fn microtask_count(&self) -> usize {
        self.microtasks
    }

    pub fn flush(&mut self) {
        while let Some(next_due) = self.timers.first().map(|timer| timer.at_ms) {
            self.now_ms = self.now_ms.max(next_due);
            let _ = self.run_due_timers();
        }
        self.microtasks = 0;
    }

    pub fn step_limit(&self) -> usize {
        self.step_limit
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FetchResponseRule {
    pub url: String,
    pub status: u16,
    pub body: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FetchErrorRule {
    pub url: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FetchCall {
    pub url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FetchResponse {
    pub url: String,
    pub status: u16,
    pub body: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FetchMocks {
    responses: Vec<FetchResponseRule>,
    errors: Vec<FetchErrorRule>,
    calls: Vec<FetchCall>,
}

impl FetchMocks {
    pub fn respond_text(&mut self, url: impl Into<String>, status: u16, body: impl Into<String>) {
        self.responses.push(FetchResponseRule {
            url: url.into(),
            status,
            body: body.into(),
        });
    }

    pub fn fail(&mut self, url: impl Into<String>, message: impl Into<String>) {
        self.errors.push(FetchErrorRule {
            url: url.into(),
            message: message.into(),
        });
    }

    pub fn record_call(&mut self, url: impl Into<String>) {
        self.calls.push(FetchCall { url: url.into() });
    }

    pub fn responses(&self) -> &[FetchResponseRule] {
        &self.responses
    }

    pub fn errors(&self) -> &[FetchErrorRule] {
        &self.errors
    }

    pub fn calls(&self) -> &[FetchCall] {
        &self.calls
    }

    pub fn reset(&mut self) {
        self.responses.clear();
        self.errors.clear();
        self.calls.clear();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DialogMocks {
    confirm_queue: Vec<bool>,
    prompt_queue: Vec<Option<String>>,
    alert_messages: Vec<String>,
    confirm_messages: Vec<String>,
    prompt_messages: Vec<String>,
}

impl DialogMocks {
    pub fn push_confirm(&mut self, value: bool) {
        self.confirm_queue.push(value);
    }

    pub fn push_prompt(&mut self, value: Option<impl Into<String>>) {
        self.prompt_queue.push(value.map(Into::into));
    }

    pub fn record_alert(&mut self, message: impl Into<String>) {
        self.alert_messages.push(message.into());
    }

    pub fn record_confirm(&mut self, message: impl Into<String>) {
        self.confirm_messages.push(message.into());
    }

    pub fn record_prompt(&mut self, message: impl Into<String>) {
        self.prompt_messages.push(message.into());
    }

    pub fn confirm_queue(&self) -> &[bool] {
        &self.confirm_queue
    }

    pub fn prompt_queue(&self) -> &[Option<String>] {
        &self.prompt_queue
    }

    pub fn alert_messages(&self) -> &[String] {
        &self.alert_messages
    }

    pub fn confirm_messages(&self) -> &[String] {
        &self.confirm_messages
    }

    pub fn prompt_messages(&self) -> &[String] {
        &self.prompt_messages
    }

    pub fn reset(&mut self) {
        self.confirm_queue.clear();
        self.prompt_queue.clear();
        self.alert_messages.clear();
        self.confirm_messages.clear();
        self.prompt_messages.clear();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClipboardMocks {
    seeded_text: Option<String>,
    writes: Vec<String>,
}

impl ClipboardMocks {
    pub fn seed_text(&mut self, value: impl Into<String>) {
        self.seeded_text = Some(value.into());
    }

    pub fn seeded_text(&self) -> Option<&str> {
        self.seeded_text.as_deref()
    }

    pub fn record_write(&mut self, value: impl Into<String>) {
        self.writes.push(value.into());
    }

    pub fn writes(&self) -> &[String] {
        &self.writes
    }

    pub fn reset(&mut self) {
        self.seeded_text = None;
        self.writes.clear();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocationMocks {
    current_url: Option<String>,
    navigations: Vec<String>,
}

impl LocationMocks {
    pub fn set_current(&mut self, url: impl Into<String>) {
        self.current_url = Some(url.into());
    }

    pub fn record_navigation(&mut self, url: impl Into<String>) {
        self.navigations.push(url.into());
    }

    pub fn current_url(&self) -> Option<&str> {
        self.current_url.as_deref()
    }

    pub fn navigations(&self) -> &[String] {
        &self.navigations
    }

    pub fn reset(&mut self) {
        self.current_url = None;
        self.navigations.clear();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DownloadCapture {
    pub file_name: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DownloadMocks {
    artifacts: Vec<DownloadCapture>,
}

impl DownloadMocks {
    pub fn capture(&mut self, file_name: impl Into<String>, bytes: impl Into<Vec<u8>>) {
        self.artifacts.push(DownloadCapture {
            file_name: file_name.into(),
            bytes: bytes.into(),
        });
    }

    pub fn artifacts(&self) -> &[DownloadCapture] {
        &self.artifacts
    }

    pub fn reset(&mut self) {
        self.artifacts.clear();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileInputSelection {
    pub selector: String,
    pub files: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileInputMocks {
    selections: Vec<FileInputSelection>,
}

impl FileInputMocks {
    pub fn set_files(
        &mut self,
        selector: impl Into<String>,
        files: impl IntoIterator<Item = impl Into<String>>,
    ) {
        self.selections.push(FileInputSelection {
            selector: selector.into(),
            files: files.into_iter().map(Into::into).collect(),
        });
    }

    pub fn selections(&self) -> &[FileInputSelection] {
        &self.selections
    }

    pub fn reset(&mut self) {
        self.selections.clear();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StorageSeeds {
    local: BTreeMap<String, String>,
    session: BTreeMap<String, String>,
}

impl StorageSeeds {
    pub fn seed_local(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.local.insert(key.into(), value.into());
    }

    pub fn seed_session(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.session.insert(key.into(), value.into());
    }

    pub fn local(&self) -> &BTreeMap<String, String> {
        &self.local
    }

    pub fn session(&self) -> &BTreeMap<String, String> {
        &self.session
    }

    pub fn reset(&mut self) {
        self.local.clear();
        self.session.clear();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionError {
    HtmlParse(String),
    Script(ScriptError),
    Selector(String),
    Dom(String),
    Event(String),
    Mock(String),
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HtmlParse(message) => write!(f, "HTML parse error: {message}"),
            Self::Script(err) => write!(f, "Script error: {err}"),
            Self::Selector(message) => write!(f, "Selector error: {message}"),
            Self::Dom(message) => write!(f, "DOM error: {message}"),
            Self::Event(message) => write!(f, "Event error: {message}"),
            Self::Mock(message) => write!(f, "Mock error: {message}"),
        }
    }
}

impl StdError for SessionError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::HtmlParse(_) => None,
            Self::Script(err) => Some(err),
            Self::Selector(_) | Self::Dom(_) | Self::Event(_) | Self::Mock(_) => None,
        }
    }
}

impl From<ScriptError> for SessionError {
    fn from(value: ScriptError) -> Self {
        Self::Script(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScriptListenerRecord {
    target: SessionEventTarget,
    event_type: String,
    capture: bool,
    handler: ScriptFunction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionEventTarget {
    Window,
    Document,
    Element(NodeId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DefaultActionKind {
    CheckboxToggle,
    SubmitButton,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DispatchOutcome {
    default_prevented: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MockRegistry {
    fetch: FetchMocks,
    dialogs: DialogMocks,
    clipboard: ClipboardMocks,
    location: LocationMocks,
    downloads: DownloadMocks,
    file_input: FileInputMocks,
    storage: StorageSeeds,
}

impl MockRegistry {
    pub fn fetch(&self) -> &FetchMocks {
        &self.fetch
    }

    pub fn fetch_mut(&mut self) -> &mut FetchMocks {
        &mut self.fetch
    }

    pub fn dialogs(&self) -> &DialogMocks {
        &self.dialogs
    }

    pub fn dialogs_mut(&mut self) -> &mut DialogMocks {
        &mut self.dialogs
    }

    pub fn clipboard(&self) -> &ClipboardMocks {
        &self.clipboard
    }

    pub fn clipboard_mut(&mut self) -> &mut ClipboardMocks {
        &mut self.clipboard
    }

    pub fn location(&self) -> &LocationMocks {
        &self.location
    }

    pub fn location_mut(&mut self) -> &mut LocationMocks {
        &mut self.location
    }

    pub fn downloads(&self) -> &DownloadMocks {
        &self.downloads
    }

    pub fn downloads_mut(&mut self) -> &mut DownloadMocks {
        &mut self.downloads
    }

    pub fn file_input(&self) -> &FileInputMocks {
        &self.file_input
    }

    pub fn file_input_mut(&mut self) -> &mut FileInputMocks {
        &mut self.file_input
    }

    pub fn storage(&self) -> &StorageSeeds {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut StorageSeeds {
        &mut self.storage
    }

    pub fn reset_all(&mut self) {
        self.fetch.reset();
        self.dialogs.reset();
        self.clipboard.reset();
        self.location.reset();
        self.downloads.reset();
        self.file_input.reset();
        self.storage.reset();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugState {
    trace_enabled: bool,
    trace_log_limit: usize,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            trace_enabled: false,
            trace_log_limit: 1_000,
        }
    }
}

impl DebugState {
    pub fn enable_trace(&mut self) {
        self.trace_enabled = true;
    }

    pub fn trace_enabled(&self) -> bool {
        self.trace_enabled
    }

    pub fn trace_log_limit(&self) -> usize {
        self.trace_log_limit
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    dom: DomStore,
    scheduler: Scheduler,
    mocks: MockRegistry,
    script: ScriptRuntime,
    config: SessionConfig,
    debug: DebugState,
    script_event_listeners: Vec<ScriptListenerRecord>,
    default_actions: Vec<DefaultActionKind>,
    focused_node: Option<NodeId>,
}

impl Session {
    pub fn new(config: SessionConfig) -> Result<Self, SessionError> {
        let mut dom = DomStore::new_empty();
        if let Some(html) = &config.html {
            dom.bootstrap_html(html.clone())
                .map_err(SessionError::HtmlParse)?;
        }

        let mut mocks = MockRegistry::default();
        for (key, value) in &config.local_storage {
            mocks.storage_mut().seed_local(key.clone(), value.clone());
        }
        mocks.location_mut().set_current(config.url.clone());

        let mut session = Self {
            dom,
            scheduler: Scheduler::default(),
            mocks,
            script: ScriptRuntime::default(),
            config,
            debug: DebugState::default(),
            script_event_listeners: Vec::new(),
            default_actions: vec![
                DefaultActionKind::CheckboxToggle,
                DefaultActionKind::SubmitButton,
            ],
            focused_node: None,
        };
        session.bootstrap_inline_scripts()?;
        Ok(session)
    }

    pub fn dom(&self) -> &DomStore {
        &self.dom
    }

    pub fn dom_mut(&mut self) -> &mut DomStore {
        &mut self.dom
    }

    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    pub fn scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }

    pub fn mocks(&self) -> &MockRegistry {
        &self.mocks
    }

    pub fn mocks_mut(&mut self) -> &mut MockRegistry {
        &mut self.mocks
    }

    pub fn script(&self) -> &ScriptRuntime {
        &self.script
    }

    pub fn script_mut(&mut self) -> &mut ScriptRuntime {
        &mut self.script
    }

    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    pub fn debug(&self) -> &DebugState {
        &self.debug
    }

    pub fn debug_mut(&mut self) -> &mut DebugState {
        &mut self.debug
    }

    pub fn dispatch_node(&mut self, node_id: NodeId, event_type: &str) -> Result<(), SessionError> {
        let event_type = event_type.trim();
        if event_type.is_empty() {
            return Err(SessionError::Event(
                "event type must not be empty".to_string(),
            ));
        }

        self.dispatch_dom_event(node_id, event_type, true, true)?;
        Ok(())
    }

    pub fn click_node(&mut self, node_id: NodeId) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        let outcome = self.dispatch_dom_event(node_id, "click", true, true)?;
        if !outcome.default_prevented {
            self.run_click_default_actions(node_id)?;
        }
        Ok(())
    }

    pub fn type_text_node(&mut self, node_id: NodeId, text: &str) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        self.dom
            .set_form_control_value(node_id, text)
            .map_err(SessionError::Dom)?;
        self.dispatch_dom_event(node_id, "input", true, false)?;
        Ok(())
    }

    pub fn set_checked_node(&mut self, node_id: NodeId, checked: bool) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        self.dom
            .set_form_control_checked(node_id, checked)
            .map_err(SessionError::Dom)?;
        self.dispatch_dom_event(node_id, "input", true, false)?;
        self.dispatch_dom_event(node_id, "change", true, false)?;
        Ok(())
    }

    pub fn set_select_value_node(
        &mut self,
        node_id: NodeId,
        value: &str,
    ) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        self.dom
            .set_select_value(node_id, value)
            .map_err(SessionError::Dom)?;
        self.dispatch_dom_event(node_id, "input", true, false)?;
        self.dispatch_dom_event(node_id, "change", true, false)?;
        Ok(())
    }

    pub fn focus_node(&mut self, node_id: NodeId) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        if self.focused_node == Some(node_id) {
            return Ok(());
        }

        if let Some(previous) = self.focused_node.take() {
            self.dispatch_dom_event(previous, "blur", false, false)?;
        }

        self.focused_node = Some(node_id);
        self.dispatch_dom_event(node_id, "focus", false, false)?;
        Ok(())
    }

    pub fn blur_node(&mut self, node_id: NodeId) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        if self.focused_node != Some(node_id) {
            return Ok(());
        }

        self.focused_node = None;
        self.dispatch_dom_event(node_id, "blur", false, false)?;
        Ok(())
    }

    pub fn submit_node(&mut self, node_id: NodeId) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(SessionError::Dom(format!("invalid node id: {:?}", node_id)));
        };
        if matches!(&node.kind, NodeKind::Element(element) if element.tag_name == "form") {
            self.dispatch_dom_event(node_id, "submit", true, true)?;
            return Ok(());
        }

        let Some(form_id) = self.find_associated_form(node_id) else {
            return Err(SessionError::Dom(format!(
                "submit is only supported on <form> elements or submit controls with an associated form, not {:?}",
                node_id
            )));
        };

        self.dispatch_dom_event(form_id, "submit", true, true)?;
        Ok(())
    }

    pub fn text_content_for_node(&self, node_id: NodeId) -> String {
        self.dom.text_content_for_node(node_id)
    }

    pub fn value_for_node(&self, node_id: NodeId) -> String {
        self.dom.value_for_node(node_id)
    }

    pub fn checked_for_node(&self, node_id: NodeId) -> Option<bool> {
        self.dom.checked_for_node(node_id)
    }

    pub fn alert(&mut self, message: &str) {
        self.mocks.dialogs_mut().record_alert(message.to_string());
    }

    pub fn confirm(&mut self, message: &str) -> Result<bool, SessionError> {
        let dialogs = self.mocks.dialogs_mut();
        dialogs.record_confirm(message.to_string());
        if dialogs.confirm_queue.is_empty() {
            return Err(SessionError::Mock(
                "confirm() requires a queued response".to_string(),
            ));
        }
        Ok(dialogs.confirm_queue.remove(0))
    }

    pub fn prompt(&mut self, message: &str) -> Result<Option<String>, SessionError> {
        let dialogs = self.mocks.dialogs_mut();
        dialogs.record_prompt(message.to_string());
        if dialogs.prompt_queue.is_empty() {
            return Err(SessionError::Mock(
                "prompt() requires a queued response".to_string(),
            ));
        }
        Ok(dialogs.prompt_queue.remove(0))
    }

    pub fn read_clipboard(&self) -> Result<String, SessionError> {
        self.mocks
            .clipboard()
            .seeded_text()
            .map(ToString::to_string)
            .ok_or_else(|| SessionError::Mock("clipboard text has not been seeded".to_string()))
    }

    pub fn write_clipboard(&mut self, text: &str) {
        let clipboard = self.mocks.clipboard_mut();
        clipboard.record_write(text.to_string());
        clipboard.seed_text(text.to_string());
    }

    pub fn capture_download(
        &mut self,
        file_name: &str,
        bytes: impl Into<Vec<u8>>,
    ) -> Result<(), SessionError> {
        if file_name.trim().is_empty() {
            return Err(SessionError::Mock(
                "capture_download() requires a non-empty file name".to_string(),
            ));
        }

        self.mocks
            .downloads_mut()
            .capture(file_name.to_string(), bytes);
        Ok(())
    }

    pub fn fetch(&mut self, url: &str) -> Result<FetchResponse, SessionError> {
        let url = url.trim();
        if url.is_empty() {
            return Err(SessionError::Mock(
                "fetch() requires a non-empty URL".to_string(),
            ));
        }

        self.mocks.fetch_mut().record_call(url.to_string());

        if let Some(error) = self
            .mocks
            .fetch()
            .errors()
            .iter()
            .rev()
            .find(|rule| rule.url == url)
        {
            return Err(SessionError::Mock(error.message.clone()));
        }

        if let Some(response) = self
            .mocks
            .fetch()
            .responses()
            .iter()
            .rev()
            .find(|rule| rule.url == url)
        {
            return Ok(FetchResponse {
                url: url.to_string(),
                status: response.status,
                body: response.body.clone(),
            });
        }

        Err(SessionError::Mock(format!(
            "no fetch mock configured for `{url}`"
        )))
    }

    pub fn navigate(&mut self, url: &str) -> Result<(), SessionError> {
        let url = url.trim();
        if url.is_empty() {
            return Err(SessionError::Mock(
                "navigate() requires a non-empty URL".to_string(),
            ));
        }

        let location = self.mocks.location_mut();
        location.set_current(url.to_string());
        location.record_navigation(url.to_string());
        Ok(())
    }

    pub fn set_files_node(
        &mut self,
        node_id: NodeId,
        selector: &str,
        files: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        let files: Vec<String> = files.into_iter().map(Into::into).collect();
        self.dom
            .set_file_input_files(node_id, files.clone())
            .map_err(SessionError::Dom)?;
        self.mocks
            .file_input_mut()
            .set_files(selector.to_string(), files);
        self.dispatch_dom_event(node_id, "input", true, false)?;
        self.dispatch_dom_event(node_id, "change", true, false)?;
        Ok(())
    }

    fn ensure_element_node(&self, node_id: NodeId) -> Result<(), SessionError> {
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(SessionError::Dom(format!("invalid node id: {:?}", node_id)));
        };

        match &node.kind {
            NodeKind::Element(_) => Ok(()),
            _ => Err(SessionError::Dom(format!(
                "node {:?} is not an element",
                node_id
            ))),
        }
    }

    fn dispatch_dom_event(
        &mut self,
        node_id: NodeId,
        event_type: &str,
        bubbles: bool,
        cancelable: bool,
    ) -> Result<DispatchOutcome, SessionError> {
        self.ensure_element_node(node_id)?;

        let event = ScriptEventHandle::new(
            event_type.to_string(),
            Self::script_listener_target(SessionEventTarget::Element(node_id)),
            bubbles,
            cancelable,
        );
        let ancestors = self.event_ancestor_targets(node_id);

        for target in ancestors.iter().rev() {
            self.run_event_listeners(*target, event_type, true, EventPhase::Capturing, &event)?;
            if event.immediate_propagation_stopped() || event.propagation_stopped() {
                event.set_current_target(None);
                event.set_phase(EventPhase::None);
                return Ok(DispatchOutcome {
                    default_prevented: event.default_prevented(),
                });
            }
        }

        self.run_event_listeners(
            SessionEventTarget::Element(node_id),
            event_type,
            true,
            EventPhase::AtTarget,
            &event,
        )?;
        if event.immediate_propagation_stopped() {
            event.set_current_target(None);
            event.set_phase(EventPhase::None);
            return Ok(DispatchOutcome {
                default_prevented: event.default_prevented(),
            });
        }

        self.run_event_listeners(
            SessionEventTarget::Element(node_id),
            event_type,
            false,
            EventPhase::AtTarget,
            &event,
        )?;
        if event.immediate_propagation_stopped() || event.propagation_stopped() {
            event.set_current_target(None);
            event.set_phase(EventPhase::None);
            return Ok(DispatchOutcome {
                default_prevented: event.default_prevented(),
            });
        }

        if bubbles {
            for target in &ancestors {
                self.run_event_listeners(*target, event_type, false, EventPhase::Bubbling, &event)?;
                if event.immediate_propagation_stopped() || event.propagation_stopped() {
                    break;
                }
            }
        }

        event.set_current_target(None);
        event.set_phase(EventPhase::None);
        Ok(DispatchOutcome {
            default_prevented: event.default_prevented(),
        })
    }

    fn run_event_listeners(
        &mut self,
        target: SessionEventTarget,
        event_type: &str,
        capture: bool,
        phase: EventPhase,
        event: &ScriptEventHandle,
    ) -> Result<(), SessionError> {
        let listeners: Vec<ScriptListenerRecord> = self
            .script_event_listeners
            .iter()
            .filter(|listener| {
                listener.target == target
                    && listener.event_type == event_type
                    && listener.capture == capture
            })
            .cloned()
            .collect();

        for (index, listener) in listeners.iter().enumerate() {
            if event.immediate_propagation_stopped() {
                break;
            }

            event.set_current_target(Some(Self::script_listener_target(target)));
            event.set_phase(phase);
            let source_name = format!("event:{event_type}:{}:{index}", Self::phase_label(phase));
            let bindings = Self::listener_bindings(&listener.handler, event);
            self.eval_script_source_with_bindings(
                &listener.handler.body_source,
                &source_name,
                bindings,
            )?;
        }

        Ok(())
    }

    fn eval_script_source(&mut self, source: &str, source_name: &str) -> Result<(), SessionError> {
        self.eval_script_source_with_bindings(source, source_name, BTreeMap::new())
    }

    fn eval_script_source_with_bindings(
        &mut self,
        source: &str,
        source_name: &str,
        initial_bindings: BTreeMap<String, ScriptValue>,
    ) -> Result<(), SessionError> {
        let mut script = std::mem::take(&mut self.script);
        let result = script
            .eval_program_with_bindings(source, source_name, self, initial_bindings)
            .map_err(SessionError::Script);
        self.script = script;
        result
    }

    fn run_click_default_actions(&mut self, node_id: NodeId) -> Result<(), SessionError> {
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(SessionError::Dom(format!("invalid node id: {:?}", node_id)));
        };
        let NodeKind::Element(element) = &node.kind else {
            return Err(SessionError::Dom(format!(
                "node {:?} is not an element",
                node_id
            )));
        };

        let tag_name = element.tag_name.clone();
        let input_type = element.attributes.get("type").cloned();

        let actions = self.default_actions.clone();
        for action in actions {
            match action {
                DefaultActionKind::CheckboxToggle
                    if tag_name == "input" && is_checkable_input_type(input_type.as_deref()) =>
                {
                    let checked = self.checked_for_node(node_id).unwrap_or(false);
                    self.dom
                        .set_form_control_checked(node_id, !checked)
                        .map_err(SessionError::Dom)?;
                    self.dispatch_dom_event(node_id, "input", true, false)?;
                    self.dispatch_dom_event(node_id, "change", true, false)?;
                }
                DefaultActionKind::SubmitButton
                    if is_submit_control(tag_name.as_str(), input_type.as_deref()) =>
                {
                    if let Some(form_id) = self.find_associated_form(node_id) {
                        self.dispatch_dom_event(form_id, "submit", true, true)?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn find_associated_form(&self, mut node_id: NodeId) -> Option<NodeId> {
        loop {
            let node = self.dom.nodes().get(node_id.index() as usize)?;
            match &node.kind {
                NodeKind::Element(element) if element.tag_name == "form" => return Some(node_id),
                NodeKind::Element(_) | NodeKind::Text(_) | NodeKind::Comment(_) => {
                    node_id = node.parent?;
                }
                NodeKind::Document => return None,
            }
        }
    }

    fn event_ancestor_targets(&self, node_id: NodeId) -> Vec<SessionEventTarget> {
        let mut targets = Vec::new();
        let mut current = self
            .dom
            .nodes()
            .get(node_id.index() as usize)
            .and_then(|node| node.parent);

        while let Some(parent_id) = current {
            let Some(parent) = self.dom.nodes().get(parent_id.index() as usize) else {
                break;
            };

            match &parent.kind {
                NodeKind::Document => {
                    targets.push(SessionEventTarget::Document);
                    break;
                }
                NodeKind::Element(_) | NodeKind::Text(_) | NodeKind::Comment(_) => {
                    targets.push(SessionEventTarget::Element(parent_id));
                    current = parent.parent;
                }
            }
        }

        targets.push(SessionEventTarget::Window);
        targets
    }

    fn listener_bindings(
        handler: &ScriptFunction,
        event: &ScriptEventHandle,
    ) -> BTreeMap<String, ScriptValue> {
        let mut bindings = BTreeMap::new();
        bindings.insert("event".to_string(), ScriptValue::Event(event.clone()));

        for (index, param) in handler.params.iter().enumerate() {
            if index == 0 {
                bindings.insert(param.clone(), ScriptValue::Event(event.clone()));
            } else {
                bindings.insert(param.clone(), ScriptValue::Undefined);
            }
        }

        bindings
    }

    fn script_listener_target(target: SessionEventTarget) -> ListenerTarget {
        match target {
            SessionEventTarget::Window => ListenerTarget::Window,
            SessionEventTarget::Document => ListenerTarget::Document,
            SessionEventTarget::Element(node_id) => {
                ListenerTarget::Element(Self::node_id_to_handle(node_id))
            }
        }
    }

    fn phase_label(phase: EventPhase) -> &'static str {
        match phase {
            EventPhase::None => "none",
            EventPhase::Capturing => "capture",
            EventPhase::AtTarget => "target",
            EventPhase::Bubbling => "bubble",
        }
    }

    fn bootstrap_inline_scripts(&mut self) -> Result<(), SessionError> {
        let sources = self.collect_inline_script_sources()?;
        for (index, source) in sources.iter().enumerate() {
            self.eval_script_source(source, &format!("inline-script-{index}"))?;
        }
        Ok(())
    }

    fn collect_inline_script_sources(&self) -> Result<Vec<String>, SessionError> {
        let mut sources = Vec::new();
        self.collect_inline_script_sources_from(self.dom.document_id(), &mut sources)?;
        Ok(sources)
    }

    fn collect_inline_script_sources_from(
        &self,
        node_id: NodeId,
        sources: &mut Vec<String>,
    ) -> Result<(), SessionError> {
        let node = &self.dom.nodes()[node_id.index() as usize];
        if let NodeKind::Element(element) = &node.kind {
            if element.tag_name == "script" {
                if element.attributes.contains_key("src") {
                    return Err(SessionError::Script(ScriptError::new(
                        "external <script src=...> tags are not supported in this workspace yet",
                    )));
                }

                let source = self.dom.text_content_for_node(node_id);
                if !source.trim().is_empty() {
                    sources.push(source);
                }
            }
        }

        for child in &node.children {
            self.collect_inline_script_sources_from(*child, sources)?;
        }

        Ok(())
    }

    fn node_id_to_handle(node_id: NodeId) -> ElementHandle {
        ElementHandle::new(((node_id.generation() as u64) << 32) | node_id.index() as u64)
    }

    fn node_id_to_node_handle(node_id: NodeId) -> NodeHandle {
        NodeHandle::new(((node_id.generation() as u64) << 32) | node_id.index() as u64)
    }

    fn query_selector_handle(
        &self,
        scope: Option<NodeId>,
        selector: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        Ok(self
            .query_selector_handles(scope, selector)?
            .into_iter()
            .next())
    }

    fn query_selector_handles(
        &self,
        scope: Option<NodeId>,
        selector: &str,
    ) -> Result<Vec<ElementHandle>, ScriptError> {
        let scope_root = scope.or_else(|| self.dom.root_element_id());
        let matches = self
            .dom
            .select_with_scope(selector, scope_root)
            .map_err(ScriptError::new)?;
        Ok(matches
            .into_iter()
            .filter(|node_id| match scope {
                None => true,
                Some(scope_id) => self.is_descendant_of(*node_id, scope_id),
            })
            .map(Self::node_id_to_handle)
            .collect())
    }

    fn element_matches_selector(
        &self,
        node_id: NodeId,
        selector: &str,
    ) -> Result<bool, ScriptError> {
        let matches = self
            .dom
            .select_with_scope(selector, Some(node_id))
            .map_err(ScriptError::new)?;
        Ok(matches.contains(&node_id))
    }

    fn element_closest_selector(
        &self,
        node_id: NodeId,
        selector: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let matches = self
            .dom
            .select_with_scope(selector, Some(node_id))
            .map_err(ScriptError::new)?;
        let mut current = Some(node_id);

        while let Some(candidate) = current {
            if matches.contains(&candidate) {
                return Ok(Some(Self::node_id_to_handle(candidate)));
            }

            current = self
                .dom
                .nodes()
                .get(candidate.index() as usize)
                .and_then(|node| node.parent);
        }

        Ok(None)
    }

    fn collection_root_for_scope(
        &self,
        scope: &HtmlCollectionScope,
    ) -> Result<NodeId, ScriptError> {
        match scope {
            HtmlCollectionScope::Document => Ok(self.dom.document_id()),
            HtmlCollectionScope::Element(element) => self.node_id_for_handle(*element),
        }
    }

    fn collect_descendant_elements_matching<F>(
        &self,
        node_id: NodeId,
        collected: &mut Vec<NodeId>,
        matches: &F,
    ) where
        F: Fn(&ElementData) -> bool,
    {
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return;
        };

        for child in &node.children {
            self.collect_descendant_elements_matching_inner(*child, collected, matches);
        }
    }

    fn collect_descendant_elements_matching_inner<F>(
        &self,
        node_id: NodeId,
        collected: &mut Vec<NodeId>,
        matches: &F,
    ) where
        F: Fn(&ElementData) -> bool,
    {
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return;
        };

        if let NodeKind::Element(element) = &node.kind {
            if matches(element) {
                collected.push(node_id);
            }
        }

        for child in &node.children {
            self.collect_descendant_elements_matching_inner(*child, collected, matches);
        }
    }

    fn direct_child_elements_matching<F>(
        &self,
        node_id: NodeId,
        matches: &F,
    ) -> Result<Vec<NodeId>, ScriptError>
    where
        F: Fn(&ElementData) -> bool,
    {
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid node id"));
        };

        Ok(node
            .children
            .iter()
            .copied()
            .filter_map(|child_id| {
                let Some(child) = self.dom.nodes().get(child_id.index() as usize) else {
                    return None;
                };
                let NodeKind::Element(element) = &child.kind else {
                    return None;
                };
                matches(element).then_some(child_id)
            })
            .collect())
    }

    fn elements_by_tag_name(
        &self,
        scope: &HtmlCollectionScope,
        tag_name: &str,
    ) -> Result<Vec<ElementHandle>, ScriptError> {
        let root = self.collection_root_for_scope(scope)?;
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| {
                tag_name == "*" || element.tag_name.eq_ignore_ascii_case(tag_name)
            },
        );
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn elements_by_tag_name_ns(
        &self,
        scope: &HtmlCollectionScope,
        namespace_uri: &str,
        local_name: &str,
    ) -> Result<Vec<ElementHandle>, ScriptError> {
        let root = self.collection_root_for_scope(scope)?;
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| {
                Self::matches_namespace_uri(element, namespace_uri)
                    && (local_name == "*" || element.local_name.eq_ignore_ascii_case(local_name))
            },
        );
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn named_item_for_tag_name_collection(
        &self,
        scope: &HtmlCollectionScope,
        tag_name: &str,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let root = self.collection_root_for_scope(scope)?;
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| {
                tag_name == "*" || element.tag_name.eq_ignore_ascii_case(tag_name)
            },
        );
        Ok(self.first_named_item_in_nodes(&collected, name))
    }

    fn named_item_for_tag_name_ns_collection(
        &self,
        scope: &HtmlCollectionScope,
        namespace_uri: &str,
        local_name: &str,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let root = self.collection_root_for_scope(scope)?;
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| {
                Self::matches_namespace_uri(element, namespace_uri)
                    && (local_name == "*" || element.local_name.eq_ignore_ascii_case(local_name))
            },
        );
        Ok(self.first_named_item_in_nodes(&collected, name))
    }

    fn elements_by_class_name(
        &self,
        scope: &HtmlCollectionScope,
        class_names: &str,
    ) -> Result<Vec<ElementHandle>, ScriptError> {
        let class_tokens = Self::ordered_class_names(class_names);
        if class_tokens.is_empty() {
            return Ok(Vec::new());
        }

        let root = self.collection_root_for_scope(scope)?;
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| {
                let Some(value) = element.attributes.get("class") else {
                    return false;
                };

                class_tokens.iter().all(|class_name| {
                    value
                        .split_ascii_whitespace()
                        .any(|candidate| candidate == class_name)
                })
            },
        );
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn named_item_for_class_name_collection(
        &self,
        scope: &HtmlCollectionScope,
        class_names: &str,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let class_tokens = Self::ordered_class_names(class_names);
        if class_tokens.is_empty() {
            return Ok(None);
        }

        let root = self.collection_root_for_scope(scope)?;
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| {
                let Some(value) = element.attributes.get("class") else {
                    return false;
                };

                class_tokens.iter().all(|class_name| {
                    value
                        .split_ascii_whitespace()
                        .any(|candidate| candidate == class_name)
                })
            },
        );
        Ok(self.first_named_item_in_nodes(&collected, name))
    }

    fn elements_by_name(&self, name: &str) -> Result<Vec<ElementHandle>, ScriptError> {
        let root = self.dom.document_id();
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| {
                element
                    .attributes
                    .get("name")
                    .is_some_and(|value| value == name)
            },
        );
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn element_labels(&self, element: ElementHandle) -> Result<Vec<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(element)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element_data) = &node.kind else {
            return Err(ScriptError::new("node is not a labelable element"));
        };

        if !Self::is_labelable_element(element_data) {
            return Err(ScriptError::new("node is not a labelable element"));
        }

        let target_id = element_data.attributes.get("id").cloned();
        let root = self.dom.document_id();
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| element.tag_name == "label",
        );

        let mut labels = Vec::new();
        for label_id in collected {
            let Some(label_node) = self.dom.nodes().get(label_id.index() as usize) else {
                continue;
            };
            let NodeKind::Element(label_element) = &label_node.kind else {
                continue;
            };

            let explicit = target_id.as_deref().is_some_and(|target_id| {
                label_element
                    .attributes
                    .get("for")
                    .is_some_and(|value| value == target_id)
            });
            let implicit = self.is_descendant_of(node_id, label_id);
            if explicit || implicit {
                labels.push(Self::node_id_to_handle(label_id));
            }
        }

        Ok(labels)
    }

    fn form_elements(&self, form: ElementHandle) -> Result<Vec<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(form)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new("node is not a form element"));
        };

        if !matches!(element.tag_name.as_str(), "form" | "fieldset") {
            return Err(ScriptError::new("node is not a form element"));
        }

        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            node_id,
            &mut collected,
            &|element: &ElementData| Self::is_form_control_element(element),
        );
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn form_elements_named_item(
        &self,
        form: ElementHandle,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let items = self.form_elements_named_items(form, name)?;
        Ok(items.first().copied())
    }

    fn form_elements_named_items(
        &self,
        form: ElementHandle,
        name: &str,
    ) -> Result<Vec<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(form)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new("node is not a form element"));
        };

        if !matches!(element.tag_name.as_str(), "form" | "fieldset") {
            return Err(ScriptError::new("node is not a form element"));
        }

        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            node_id,
            &mut collected,
            &|element: &ElementData| Self::is_form_control_element(element),
        );
        Ok(self
            .named_items_in_nodes(&collected, name)
            .into_iter()
            .map(Self::node_id_to_handle)
            .collect())
    }

    fn select_options(&self, select: ElementHandle) -> Result<Vec<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(select)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new("node is not a select element"));
        };

        if !matches!(element.tag_name.as_str(), "select" | "datalist") {
            return Err(ScriptError::new("node is not a select element"));
        }

        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            node_id,
            &mut collected,
            &|element: &ElementData| element.tag_name == "option",
        );
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn select_options_named_item(
        &self,
        select: ElementHandle,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(select)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new("node is not a select element"));
        };

        if !matches!(element.tag_name.as_str(), "select" | "datalist") {
            return Err(ScriptError::new("node is not a select element"));
        }

        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            node_id,
            &mut collected,
            &|element: &ElementData| element.tag_name == "option",
        );
        Ok(self.first_named_item_in_nodes(&collected, name))
    }

    fn selected_options(&self, select: ElementHandle) -> Result<Vec<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(select)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new("node is not a select element"));
        };

        if element.tag_name != "select" {
            return Err(ScriptError::new("node is not a select element"));
        }

        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            node_id,
            &mut collected,
            &|element: &ElementData| {
                element.tag_name == "option" && element.attributes.contains_key("selected")
            },
        );
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn selected_options_named_item(
        &self,
        select: ElementHandle,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let selected = self.selected_options(select)?;
        let selected_ids: Vec<NodeId> = selected
            .into_iter()
            .map(|handle| self.node_id_for_handle(handle))
            .collect::<Result<_, _>>()?;
        Ok(self.first_named_item_in_nodes(&selected_ids, name))
    }

    fn document_links(&self) -> Result<Vec<ElementHandle>, ScriptError> {
        let root = self.dom.document_id();
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| Self::is_document_link_element(element),
        );
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn document_links_named_item(&self, name: &str) -> Result<Option<ElementHandle>, ScriptError> {
        let root = self.dom.document_id();
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| Self::is_document_link_element(element),
        );
        Ok(self.first_named_item_in_nodes(&collected, name))
    }

    fn document_style_sheets(&self) -> Result<Vec<ElementHandle>, ScriptError> {
        let root = self.dom.document_id();
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| Self::is_document_style_sheet_element(element),
        );
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn document_children(&self) -> Result<Vec<ElementHandle>, ScriptError> {
        let root = self.dom.document_id();
        let Some(node) = self.dom.nodes().get(root.index() as usize) else {
            return Err(ScriptError::new("invalid document node"));
        };

        let children = node
            .children
            .iter()
            .copied()
            .filter(|child_id| {
                matches!(
                    self.dom
                        .nodes()
                        .get(child_id.index() as usize)
                        .map(|node| &node.kind),
                    Some(NodeKind::Element(_))
                )
            })
            .map(Self::node_id_to_handle)
            .collect();

        Ok(children)
    }

    fn document_children_named_item(
        &self,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let root = self.dom.document_id();
        let Some(node) = self.dom.nodes().get(root.index() as usize) else {
            return Err(ScriptError::new("invalid document node"));
        };

        let children: Vec<NodeId> = node
            .children
            .iter()
            .copied()
            .filter(|child_id| {
                matches!(
                    self.dom
                        .nodes()
                        .get(child_id.index() as usize)
                        .map(|node| &node.kind),
                    Some(NodeKind::Element(_))
                )
            })
            .collect();

        Ok(self.first_named_item_in_nodes(&children, name))
    }

    fn node_child_nodes(&self, scope: HtmlCollectionScope) -> Result<Vec<NodeHandle>, ScriptError> {
        let root = self.collection_root_for_scope(&scope)?;
        let Some(node) = self.dom.nodes().get(root.index() as usize) else {
            return Err(ScriptError::new("invalid node id"));
        };

        Ok(node
            .children
            .iter()
            .copied()
            .map(Self::node_id_to_node_handle)
            .collect())
    }

    fn map_areas(&self, map: ElementHandle) -> Result<Vec<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(map)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new(
                "node is not a supported map.areas host element",
            ));
        };

        if element.tag_name != "map" {
            return Err(ScriptError::new(
                "node is not a supported map.areas host element",
            ));
        }

        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            node_id,
            &mut collected,
            &|element: &ElementData| element.tag_name == "area",
        );
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn map_areas_named_item(
        &self,
        map: ElementHandle,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(map)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new(
                "node is not a supported map.areas host element",
            ));
        };

        if element.tag_name != "map" {
            return Err(ScriptError::new(
                "node is not a supported map.areas host element",
            ));
        }

        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            node_id,
            &mut collected,
            &|element: &ElementData| element.tag_name == "area",
        );
        Ok(self.first_named_item_in_nodes(&collected, name))
    }

    fn table_bodies(&self, table: ElementHandle) -> Result<Vec<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(table)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new(
                "node is not a supported table.tBodies host element",
            ));
        };

        if element.tag_name != "table" {
            return Err(ScriptError::new(
                "node is not a supported table.tBodies host element",
            ));
        }

        let collected = self
            .direct_child_elements_matching(node_id, &|element: &ElementData| {
                element.tag_name == "tbody"
            })?;
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn table_bodies_named_item(
        &self,
        table: ElementHandle,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(table)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new(
                "node is not a supported table.tBodies host element",
            ));
        };

        if element.tag_name != "table" {
            return Err(ScriptError::new(
                "node is not a supported table.tBodies host element",
            ));
        }

        let bodies = self.table_bodies(table)?;
        let body_ids: Vec<NodeId> = bodies
            .into_iter()
            .map(|handle| self.node_id_for_handle(handle))
            .collect::<Result<_, _>>()?;
        Ok(self.first_named_item_in_nodes(&body_ids, name))
    }

    fn table_rows(&self, table: ElementHandle) -> Result<Vec<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(table)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new("node is not a table.rows host element"));
        };

        let collected = match element.tag_name.as_str() {
            "table" => {
                let mut collected = Vec::new();
                for child_id in &node.children {
                    let Some(child) = self.dom.nodes().get(child_id.index() as usize) else {
                        continue;
                    };
                    let NodeKind::Element(child_element) = &child.kind else {
                        continue;
                    };

                    match child_element.tag_name.as_str() {
                        "tr" => collected.push(*child_id),
                        "thead" | "tbody" | "tfoot" => {
                            collected.extend(self.direct_child_elements_matching(
                                *child_id,
                                &|element: &ElementData| element.tag_name == "tr",
                            )?);
                        }
                        _ => {}
                    }
                }
                collected
            }
            "thead" | "tbody" | "tfoot" => self
                .direct_child_elements_matching(node_id, &|element: &ElementData| {
                    element.tag_name == "tr"
                })?,
            _ => {
                return Err(ScriptError::new(
                    "node is not a supported table.rows host element",
                ));
            }
        };

        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn table_rows_named_item(
        &self,
        table: ElementHandle,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(table)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new("node is not a table.rows host element"));
        };

        match element.tag_name.as_str() {
            "table" | "thead" | "tbody" | "tfoot" => {
                let rows = self.table_rows(table)?;
                let row_ids: Vec<NodeId> = rows
                    .into_iter()
                    .map(|handle| self.node_id_for_handle(handle))
                    .collect::<Result<_, _>>()?;
                Ok(self.first_named_item_in_nodes(&row_ids, name))
            }
            _ => Err(ScriptError::new(
                "node is not a supported table.rows host element",
            )),
        }
    }

    fn row_cells(&self, row: ElementHandle) -> Result<Vec<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(row)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new("node is not a tr.cells host element"));
        };

        if element.tag_name != "tr" {
            return Err(ScriptError::new(
                "node is not a supported tr.cells host element",
            ));
        }

        let collected = self
            .direct_child_elements_matching(node_id, &|element: &ElementData| {
                matches!(element.tag_name.as_str(), "td" | "th")
            })?;

        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn row_cells_named_item(
        &self,
        row: ElementHandle,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let node_id = self.node_id_for_handle(row)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(ScriptError::new("node is not a tr.cells host element"));
        };

        if element.tag_name != "tr" {
            return Err(ScriptError::new(
                "node is not a supported tr.cells host element",
            ));
        }

        let cells = self.row_cells(row)?;
        let cell_ids: Vec<NodeId> = cells
            .into_iter()
            .map(|handle| self.node_id_for_handle(handle))
            .collect::<Result<_, _>>()?;
        Ok(self.first_named_item_in_nodes(&cell_ids, name))
    }

    fn document_anchors(&self) -> Result<Vec<ElementHandle>, ScriptError> {
        let root = self.dom.document_id();
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| Self::is_document_anchor_element(element),
        );
        Ok(collected.into_iter().map(Self::node_id_to_handle).collect())
    }

    fn document_anchors_named_item(
        &self,
        name: &str,
    ) -> Result<Option<ElementHandle>, ScriptError> {
        let root = self.dom.document_id();
        let mut collected = Vec::new();
        self.collect_descendant_elements_matching(
            root,
            &mut collected,
            &|element: &ElementData| Self::is_document_anchor_element(element),
        );
        Ok(self.first_named_item_in_nodes(&collected, name))
    }

    fn first_named_item_in_nodes(&self, collected: &[NodeId], name: &str) -> Option<ElementHandle> {
        for node_id in collected {
            let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
                continue;
            };
            let NodeKind::Element(element) = &node.kind else {
                continue;
            };

            if element
                .attributes
                .get("id")
                .is_some_and(|value| value == name)
                || element
                    .attributes
                    .get("name")
                    .is_some_and(|value| value == name)
            {
                return Some(Self::node_id_to_handle(*node_id));
            }
        }

        None
    }

    fn named_items_in_nodes(&self, collected: &[NodeId], name: &str) -> Vec<NodeId> {
        let mut matches = Vec::new();
        for node_id in collected {
            let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
                continue;
            };
            let NodeKind::Element(element) = &node.kind else {
                continue;
            };

            if element
                .attributes
                .get("id")
                .is_some_and(|value| value == name)
                || element
                    .attributes
                    .get("name")
                    .is_some_and(|value| value == name)
            {
                matches.push(*node_id);
            }
        }

        matches
    }

    fn is_form_control_element(element: &ElementData) -> bool {
        matches!(
            element.tag_name.as_str(),
            "input" | "select" | "textarea" | "button"
        )
    }

    fn is_labelable_element(element: &ElementData) -> bool {
        match element.tag_name.as_str() {
            "button" | "fieldset" | "meter" | "output" | "progress" | "select" | "textarea" => true,
            "input" => !element
                .attributes
                .get("type")
                .is_some_and(|value| value.eq_ignore_ascii_case("hidden")),
            _ => false,
        }
    }

    fn is_document_link_element(element: &ElementData) -> bool {
        matches!(element.tag_name.as_str(), "a" | "area") && element.attributes.contains_key("href")
    }

    fn is_document_style_sheet_element(element: &ElementData) -> bool {
        if element.tag_name == "style" {
            return true;
        }

        if element.tag_name != "link" {
            return false;
        }

        let Some(rel) = element.attributes.get("rel") else {
            return false;
        };

        rel.split_ascii_whitespace()
            .any(|token| token.eq_ignore_ascii_case("stylesheet"))
    }

    fn is_document_anchor_element(element: &ElementData) -> bool {
        element.tag_name == "a" && element.attributes.contains_key("name")
    }

    fn matches_namespace_uri(element: &ElementData, namespace_uri: &str) -> bool {
        match namespace_uri {
            "*" => true,
            HTML_NAMESPACE_URI | SVG_NAMESPACE_URI | MATHML_NAMESPACE_URI => {
                element.namespace_uri == namespace_uri
            }
            _ => false,
        }
    }

    fn ordered_class_names(class_names: &str) -> Vec<String> {
        class_names
            .split_ascii_whitespace()
            .filter(|class_name| !class_name.is_empty())
            .map(str::to_string)
            .collect()
    }

    fn is_descendant_of(&self, node_id: NodeId, ancestor_id: NodeId) -> bool {
        let mut current = self
            .dom
            .nodes()
            .get(node_id.index() as usize)
            .and_then(|node| node.parent);

        while let Some(parent_id) = current {
            if parent_id == ancestor_id {
                return true;
            }

            current = self
                .dom
                .nodes()
                .get(parent_id.index() as usize)
                .and_then(|node| node.parent);
        }

        false
    }

    fn node_id_for_handle(&self, handle: ElementHandle) -> Result<NodeId, ScriptError> {
        let raw = handle.raw();
        let index = (raw & 0xffff_ffff) as u32;
        let generation = (raw >> 32) as u32;
        let node_id = NodeId::new(index, generation);
        let Some(record) = self.dom.nodes().get(index as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };
        if record.id.generation() != generation {
            return Err(ScriptError::new("invalid element handle"));
        }
        Ok(node_id)
    }

    fn node_id_for_node_handle(&self, handle: NodeHandle) -> Result<NodeId, ScriptError> {
        let raw = handle.raw();
        let index = (raw & 0xffff_ffff) as u32;
        let generation = (raw >> 32) as u32;
        let node_id = NodeId::new(index, generation);
        let Some(record) = self.dom.nodes().get(index as usize) else {
            return Err(ScriptError::new("invalid node handle"));
        };
        if record.id.generation() != generation {
            return Err(ScriptError::new("invalid node handle"));
        }
        Ok(node_id)
    }

    fn register_script_listener(
        &mut self,
        target: SessionEventTarget,
        event_type: String,
        capture: bool,
        handler: ScriptFunction,
    ) {
        self.script_event_listeners.push(ScriptListenerRecord {
            target,
            event_type,
            capture,
            handler,
        });
    }
}

impl HostBindings for Session {
    fn document_get_element_by_id(&mut self, id: &str) -> bt_script::Result<Option<ElementHandle>> {
        let Some(node_id) = self.dom.indexes().id_index.get(id).copied() else {
            return Ok(None);
        };

        Ok(Some(Self::node_id_to_handle(node_id)))
    }

    fn document_query_selector(
        &mut self,
        selector: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.query_selector_handle(None, selector)
    }

    fn document_query_selector_all(
        &mut self,
        selector: &str,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.query_selector_handles(None, selector)
    }

    fn document_get_elements_by_name(
        &mut self,
        name: &str,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.elements_by_name(name)
    }

    fn document_style_sheets_items(&mut self) -> bt_script::Result<Vec<ElementHandle>> {
        Session::document_style_sheets(self)
    }

    fn node_child_nodes_items(
        &mut self,
        scope: HtmlCollectionScope,
    ) -> bt_script::Result<Vec<NodeHandle>> {
        self.node_child_nodes(scope)
    }

    fn node_text_content(&mut self, node: NodeHandle) -> bt_script::Result<String> {
        let node_id = self.node_id_for_node_handle(node)?;
        Ok(self.dom.text_content_for_node(node_id))
    }

    fn node_type(&mut self, node: NodeHandle) -> bt_script::Result<u8> {
        let node_id = self.node_id_for_node_handle(node)?;
        let Some(record) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid node handle"));
        };

        let node_type = match &record.kind {
            NodeKind::Document => 9,
            NodeKind::Element(_) => 1,
            NodeKind::Text(_) => 3,
            NodeKind::Comment(_) => 8,
        };
        Ok(node_type)
    }

    fn node_name(&mut self, node: NodeHandle) -> bt_script::Result<String> {
        let node_id = self.node_id_for_node_handle(node)?;
        let Some(record) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid node handle"));
        };

        let node_name = match &record.kind {
            NodeKind::Document => "#document".to_string(),
            NodeKind::Element(element) => element.tag_name.clone(),
            NodeKind::Text(_) => "#text".to_string(),
            NodeKind::Comment(_) => "#comment".to_string(),
        };
        Ok(node_name)
    }

    fn html_collection_tag_name_items(
        &mut self,
        collection: HtmlCollectionTarget,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        match collection {
            HtmlCollectionTarget::Children(element) => self.element_children(element),
            HtmlCollectionTarget::ByTagName { scope, tag_name } => {
                self.elements_by_tag_name(&scope, &tag_name)
            }
            HtmlCollectionTarget::ByTagNameNs {
                scope,
                namespace_uri,
                local_name,
            } => self.elements_by_tag_name_ns(&scope, &namespace_uri, &local_name),
            HtmlCollectionTarget::ByClassName { scope, class_names } => {
                self.elements_by_class_name(&scope, &class_names)
            }
            HtmlCollectionTarget::FormElements(element) => self.form_elements(element),
            HtmlCollectionTarget::SelectOptions(element) => self.select_options(element),
            HtmlCollectionTarget::SelectSelectedOptions(element) => self.selected_options(element),
            HtmlCollectionTarget::DocumentLinks => self.document_links(),
            HtmlCollectionTarget::DocumentAnchors => self.document_anchors(),
            HtmlCollectionTarget::DocumentChildren => self.document_children(),
            HtmlCollectionTarget::MapAreas(element) => self.map_areas(element),
            HtmlCollectionTarget::TableTBodies(element) => self.table_bodies(element),
            HtmlCollectionTarget::TableRows(element) => self.table_rows(element),
            HtmlCollectionTarget::RowCells(element) => self.row_cells(element),
        }
    }

    fn html_collection_tag_name_named_item(
        &mut self,
        collection: HtmlCollectionTarget,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        match collection {
            HtmlCollectionTarget::Children(element) => {
                self.html_collection_named_item(element, name)
            }
            HtmlCollectionTarget::ByTagName { scope, tag_name } => {
                self.named_item_for_tag_name_collection(&scope, &tag_name, name)
            }
            HtmlCollectionTarget::ByTagNameNs {
                scope,
                namespace_uri,
                local_name,
            } => self.named_item_for_tag_name_ns_collection(
                &scope,
                &namespace_uri,
                &local_name,
                name,
            ),
            HtmlCollectionTarget::ByClassName { scope, class_names } => {
                self.named_item_for_class_name_collection(&scope, &class_names, name)
            }
            HtmlCollectionTarget::FormElements(element) => {
                self.form_elements_named_item(element, name)
            }
            HtmlCollectionTarget::SelectOptions(element) => {
                self.select_options_named_item(element, name)
            }
            HtmlCollectionTarget::SelectSelectedOptions(element) => {
                self.selected_options_named_item(element, name)
            }
            HtmlCollectionTarget::DocumentLinks => self.document_links_named_item(name),
            HtmlCollectionTarget::DocumentAnchors => self.document_anchors_named_item(name),
            HtmlCollectionTarget::DocumentChildren => self.document_children_named_item(name),
            HtmlCollectionTarget::MapAreas(element) => self.map_areas_named_item(element, name),
            HtmlCollectionTarget::TableTBodies(element) => {
                self.table_bodies_named_item(element, name)
            }
            HtmlCollectionTarget::TableRows(element) => self.table_rows_named_item(element, name),
            HtmlCollectionTarget::RowCells(element) => self.row_cells_named_item(element, name),
        }
    }

    fn html_collection_class_name_items(
        &mut self,
        collection: HtmlCollectionTarget,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        match collection {
            HtmlCollectionTarget::Children(element) => self.element_children(element),
            HtmlCollectionTarget::ByTagName { scope, tag_name } => {
                self.elements_by_tag_name(&scope, &tag_name)
            }
            HtmlCollectionTarget::ByTagNameNs {
                scope,
                namespace_uri,
                local_name,
            } => self.elements_by_tag_name_ns(&scope, &namespace_uri, &local_name),
            HtmlCollectionTarget::ByClassName { scope, class_names } => {
                self.elements_by_class_name(&scope, &class_names)
            }
            HtmlCollectionTarget::FormElements(element) => self.form_elements(element),
            HtmlCollectionTarget::SelectOptions(element) => self.select_options(element),
            HtmlCollectionTarget::SelectSelectedOptions(element) => self.selected_options(element),
            HtmlCollectionTarget::DocumentLinks => self.document_links(),
            HtmlCollectionTarget::DocumentAnchors => self.document_anchors(),
            HtmlCollectionTarget::DocumentChildren => self.document_children(),
            HtmlCollectionTarget::MapAreas(element) => self.map_areas(element),
            HtmlCollectionTarget::TableTBodies(element) => self.table_bodies(element),
            HtmlCollectionTarget::TableRows(element) => self.table_rows(element),
            HtmlCollectionTarget::RowCells(element) => self.row_cells(element),
        }
    }

    fn html_collection_class_name_named_item(
        &mut self,
        collection: HtmlCollectionTarget,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        match collection {
            HtmlCollectionTarget::Children(element) => {
                self.html_collection_named_item(element, name)
            }
            HtmlCollectionTarget::ByTagName { scope, tag_name } => {
                self.named_item_for_tag_name_collection(&scope, &tag_name, name)
            }
            HtmlCollectionTarget::ByTagNameNs {
                scope,
                namespace_uri,
                local_name,
            } => self.named_item_for_tag_name_ns_collection(
                &scope,
                &namespace_uri,
                &local_name,
                name,
            ),
            HtmlCollectionTarget::ByClassName { scope, class_names } => {
                self.named_item_for_class_name_collection(&scope, &class_names, name)
            }
            HtmlCollectionTarget::FormElements(element) => {
                self.form_elements_named_item(element, name)
            }
            HtmlCollectionTarget::SelectOptions(element) => {
                self.select_options_named_item(element, name)
            }
            HtmlCollectionTarget::SelectSelectedOptions(element) => {
                self.selected_options_named_item(element, name)
            }
            HtmlCollectionTarget::DocumentLinks => self.document_links_named_item(name),
            HtmlCollectionTarget::DocumentAnchors => self.document_anchors_named_item(name),
            HtmlCollectionTarget::DocumentChildren => self.document_children_named_item(name),
            HtmlCollectionTarget::MapAreas(element) => self.map_areas_named_item(element, name),
            HtmlCollectionTarget::TableTBodies(element) => {
                self.table_bodies_named_item(element, name)
            }
            HtmlCollectionTarget::TableRows(element) => self.table_rows_named_item(element, name),
            HtmlCollectionTarget::RowCells(element) => self.row_cells_named_item(element, name),
        }
    }

    fn html_collection_form_elements_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.form_elements(element)
    }

    fn html_collection_tag_name_ns_items(
        &mut self,
        collection: HtmlCollectionTarget,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        match collection {
            HtmlCollectionTarget::ByTagNameNs {
                scope,
                namespace_uri,
                local_name,
            } => self.elements_by_tag_name_ns(&scope, &namespace_uri, &local_name),
            HtmlCollectionTarget::DocumentAnchors => self.document_anchors(),
            other => self.html_collection_tag_name_items(other),
        }
    }

    fn html_collection_tag_name_ns_named_item(
        &mut self,
        collection: HtmlCollectionTarget,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        match collection {
            HtmlCollectionTarget::ByTagNameNs {
                scope,
                namespace_uri,
                local_name,
            } => self.named_item_for_tag_name_ns_collection(
                &scope,
                &namespace_uri,
                &local_name,
                name,
            ),
            HtmlCollectionTarget::DocumentAnchors => self.document_anchors_named_item(name),
            other => self.html_collection_tag_name_named_item(other, name),
        }
    }

    fn html_collection_form_elements_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        Session::form_elements_named_item(self, element, name)
    }

    fn html_collection_form_elements_named_items(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        Session::form_elements_named_items(self, element, name)
    }

    fn html_collection_select_options_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        Session::select_options(self, element)
    }

    fn html_collection_select_options_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        Session::select_options_named_item(self, element, name)
    }

    fn html_collection_select_selected_options_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        Session::selected_options(self, element)
    }

    fn html_collection_select_selected_options_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        Session::selected_options_named_item(self, element, name)
    }

    fn html_collection_document_links_items(&mut self) -> bt_script::Result<Vec<ElementHandle>> {
        Session::document_links(self)
    }

    fn html_collection_document_links_named_item(
        &mut self,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        Session::document_links_named_item(self, name)
    }

    fn html_collection_document_anchors_items(&mut self) -> bt_script::Result<Vec<ElementHandle>> {
        Session::document_anchors(self)
    }

    fn html_collection_document_anchors_named_item(
        &mut self,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        Session::document_anchors_named_item(self, name)
    }

    fn html_collection_document_children_items(&mut self) -> bt_script::Result<Vec<ElementHandle>> {
        Session::document_children(self)
    }

    fn html_collection_document_children_named_item(
        &mut self,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        Session::document_children_named_item(self, name)
    }

    fn html_collection_map_areas_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        Session::map_areas(self, element)
    }

    fn html_collection_map_areas_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        Session::map_areas_named_item(self, element, name)
    }

    fn html_collection_table_bodies_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        Session::table_bodies(self, element)
    }

    fn html_collection_table_bodies_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        Session::table_bodies_named_item(self, element, name)
    }

    fn html_collection_table_rows_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        Session::table_rows(self, element)
    }

    fn html_collection_table_rows_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        Session::table_rows_named_item(self, element, name)
    }

    fn html_collection_row_cells_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        Session::row_cells(self, element)
    }

    fn html_collection_row_cells_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        Session::row_cells_named_item(self, element, name)
    }

    fn element_text_content(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        let node_id = self.node_id_for_handle(element)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        match &node.kind {
            NodeKind::Element(_)
            | NodeKind::Document
            | NodeKind::Text(_)
            | NodeKind::Comment(_) => Ok(self.dom.text_content_for_node(node_id)),
        }
    }

    fn element_set_text_content(
        &mut self,
        element: ElementHandle,
        value: &str,
    ) -> bt_script::Result<()> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .set_text_content(node_id, value)
            .map_err(ScriptError::new)
    }

    fn element_inner_html(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .inner_html_for_node(node_id)
            .map_err(ScriptError::new)
    }

    fn element_set_inner_html(
        &mut self,
        element: ElementHandle,
        value: &str,
    ) -> bt_script::Result<()> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .set_inner_html(node_id, value)
            .map_err(ScriptError::new)
    }

    fn element_outer_html(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .outer_html_for_node(node_id)
            .map_err(ScriptError::new)
    }

    fn element_set_outer_html(
        &mut self,
        element: ElementHandle,
        value: &str,
    ) -> bt_script::Result<()> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .set_outer_html(node_id, value)
            .map_err(ScriptError::new)
    }

    fn element_insert_adjacent_html(
        &mut self,
        element: ElementHandle,
        position: &str,
        value: &str,
    ) -> bt_script::Result<()> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .insert_adjacent_html(node_id, position, value)
            .map_err(ScriptError::new)
    }

    fn element_value(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        let node_id = self.node_id_for_handle(element)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };
        match &node.kind {
            NodeKind::Element(_)
            | NodeKind::Document
            | NodeKind::Text(_)
            | NodeKind::Comment(_) => Ok(self.dom.value_for_node(node_id)),
        }
    }

    fn element_children(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        let node_id = self.node_id_for_handle(element)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let children = node
            .children
            .iter()
            .copied()
            .filter(|child_id| {
                matches!(
                    self.dom
                        .nodes()
                        .get(child_id.index() as usize)
                        .map(|node| &node.kind),
                    Some(NodeKind::Element(_))
                )
            })
            .map(Self::node_id_to_handle)
            .collect();

        Ok(children)
    }

    fn element_tag_name(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        let node_id = self.node_id_for_handle(element)?;
        let Some(record) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        let NodeKind::Element(element) = &record.kind else {
            return Err(ScriptError::new("invalid element handle"));
        };

        Ok(element.tag_name.clone())
    }

    fn element_labels(&mut self, element: ElementHandle) -> bt_script::Result<Vec<ElementHandle>> {
        Session::element_labels(self, element)
    }

    fn html_collection_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        let node_id = self.node_id_for_handle(element)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        for child_id in &node.children {
            let Some(child) = self.dom.nodes().get(child_id.index() as usize) else {
                continue;
            };
            let NodeKind::Element(element) = &child.kind else {
                continue;
            };
            if element
                .attributes
                .get("id")
                .is_some_and(|value| value == name)
                || element
                    .attributes
                    .get("name")
                    .is_some_and(|value| value == name)
            {
                return Ok(Some(Self::node_id_to_handle(*child_id)));
            }
        }

        Ok(None)
    }

    fn element_set_value(&mut self, element: ElementHandle, value: &str) -> bt_script::Result<()> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .set_form_control_value(node_id, value)
            .map_err(ScriptError::new)
    }

    fn element_checked(&mut self, element: ElementHandle) -> bt_script::Result<bool> {
        let node_id = self.node_id_for_handle(element)?;
        Ok(self.dom.checked_for_node(node_id).unwrap_or(false))
    }

    fn element_set_checked(
        &mut self,
        element: ElementHandle,
        checked: bool,
    ) -> bt_script::Result<()> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .set_form_control_checked(node_id, checked)
            .map_err(ScriptError::new)
    }

    fn element_get_attribute(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<String>> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .get_attribute(node_id, name)
            .map_err(ScriptError::new)
    }

    fn element_set_attribute(
        &mut self,
        element: ElementHandle,
        name: &str,
        value: &str,
    ) -> bt_script::Result<()> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .set_attribute(node_id, name, value)
            .map_err(ScriptError::new)
    }

    fn element_remove_attribute(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<()> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .remove_attribute(node_id, name)
            .map_err(ScriptError::new)?;
        Ok(())
    }

    fn element_has_attribute(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<bool> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .has_attribute(node_id, name)
            .map_err(ScriptError::new)
    }

    fn element_toggle_attribute(
        &mut self,
        element: ElementHandle,
        name: &str,
        force: Option<bool>,
    ) -> bt_script::Result<bool> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom
            .toggle_attribute(node_id, name, force)
            .map_err(ScriptError::new)
    }

    fn element_append_child(
        &mut self,
        parent: ElementHandle,
        child: ElementHandle,
    ) -> bt_script::Result<()> {
        let parent_id = self.node_id_for_handle(parent)?;
        let child_id = self.node_id_for_handle(child)?;
        self.dom
            .append_child(parent_id, child_id)
            .map_err(ScriptError::new)
    }

    fn element_insert_before(
        &mut self,
        parent: ElementHandle,
        child: ElementHandle,
        reference: Option<ElementHandle>,
    ) -> bt_script::Result<()> {
        let parent_id = self.node_id_for_handle(parent)?;
        let child_id = self.node_id_for_handle(child)?;
        match reference {
            Some(reference) => {
                let reference_id = self.node_id_for_handle(reference)?;
                self.dom
                    .insert_before(parent_id, child_id, reference_id)
                    .map_err(ScriptError::new)
            }
            None => self
                .dom
                .append_child(parent_id, child_id)
                .map_err(ScriptError::new),
        }
    }

    fn element_replace_child(
        &mut self,
        parent: ElementHandle,
        new_child: ElementHandle,
        old_child: ElementHandle,
    ) -> bt_script::Result<()> {
        let parent_id = self.node_id_for_handle(parent)?;
        let new_child_id = self.node_id_for_handle(new_child)?;
        let old_child_id = self.node_id_for_handle(old_child)?;
        self.dom
            .replace_child(parent_id, new_child_id, old_child_id)
            .map_err(ScriptError::new)
    }

    fn element_replace_children(
        &mut self,
        parent: ElementHandle,
        children: Vec<ElementHandle>,
    ) -> bt_script::Result<()> {
        let parent_id = self.node_id_for_handle(parent)?;
        let children = children
            .into_iter()
            .map(|child| self.node_id_for_handle(child))
            .collect::<Result<Vec<_>, _>>()?;
        self.dom
            .replace_children(parent_id, children)
            .map_err(ScriptError::new)
    }

    fn element_append(
        &mut self,
        element: ElementHandle,
        children: Vec<ElementHandle>,
    ) -> bt_script::Result<()> {
        let parent_id = self.node_id_for_handle(element)?;
        let children = children
            .into_iter()
            .map(|child| self.node_id_for_handle(child))
            .collect::<Result<Vec<_>, _>>()?;
        self.dom
            .append_children(parent_id, children)
            .map_err(ScriptError::new)
    }

    fn element_prepend(
        &mut self,
        element: ElementHandle,
        children: Vec<ElementHandle>,
    ) -> bt_script::Result<()> {
        let parent_id = self.node_id_for_handle(element)?;
        let children = children
            .into_iter()
            .map(|child| self.node_id_for_handle(child))
            .collect::<Result<Vec<_>, _>>()?;
        self.dom
            .prepend_children(parent_id, children)
            .map_err(ScriptError::new)
    }

    fn element_before(
        &mut self,
        element: ElementHandle,
        children: Vec<ElementHandle>,
    ) -> bt_script::Result<()> {
        let node_id = self.node_id_for_handle(element)?;
        let Some(parent_id) = self
            .dom
            .nodes()
            .get(node_id.index() as usize)
            .and_then(|node| node.parent)
        else {
            return Ok(());
        };
        let children = children
            .into_iter()
            .map(|child| self.node_id_for_handle(child))
            .collect::<Result<Vec<_>, _>>()?;
        self.dom
            .insert_children_before(parent_id, node_id, children)
            .map_err(ScriptError::new)
    }

    fn element_after(
        &mut self,
        element: ElementHandle,
        children: Vec<ElementHandle>,
    ) -> bt_script::Result<()> {
        let node_id = self.node_id_for_handle(element)?;
        let Some(parent_id) = self
            .dom
            .nodes()
            .get(node_id.index() as usize)
            .and_then(|node| node.parent)
        else {
            return Ok(());
        };
        let children = children
            .into_iter()
            .map(|child| self.node_id_for_handle(child))
            .collect::<Result<Vec<_>, _>>()?;
        self.dom
            .insert_children_after(parent_id, node_id, children)
            .map_err(ScriptError::new)
    }

    fn element_remove(&mut self, element: ElementHandle) -> bt_script::Result<()> {
        let node_id = self.node_id_for_handle(element)?;
        self.dom.remove_node(node_id).map_err(ScriptError::new)
    }

    fn element_query_selector(
        &mut self,
        element: ElementHandle,
        selector: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        let node_id = self.node_id_for_handle(element)?;
        self.query_selector_handle(Some(node_id), selector)
    }

    fn element_query_selector_all(
        &mut self,
        element: ElementHandle,
        selector: &str,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        let node_id = self.node_id_for_handle(element)?;
        self.query_selector_handles(Some(node_id), selector)
    }

    fn element_matches(
        &mut self,
        element: ElementHandle,
        selector: &str,
    ) -> bt_script::Result<bool> {
        let node_id = self.node_id_for_handle(element)?;
        self.element_matches_selector(node_id, selector)
    }

    fn element_closest(
        &mut self,
        element: ElementHandle,
        selector: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        let node_id = self.node_id_for_handle(element)?;
        self.element_closest_selector(node_id, selector)
    }

    fn register_event_listener_with_capture(
        &mut self,
        target: ListenerTarget,
        event_type: &str,
        capture: bool,
        handler: ScriptFunction,
    ) -> bt_script::Result<()> {
        let target = match target {
            ListenerTarget::Window => SessionEventTarget::Window,
            ListenerTarget::Document => SessionEventTarget::Document,
            ListenerTarget::Element(handle) => {
                let node_id = self.node_id_for_handle(handle)?;
                if self.dom.nodes().get(node_id.index() as usize).is_none() {
                    return Err(ScriptError::new("invalid element handle"));
                }
                SessionEventTarget::Element(node_id)
            }
        };

        self.register_script_listener(target, event_type.to_string(), capture, handler);
        Ok(())
    }
}

fn is_checkable_input_type(input_type: Option<&str>) -> bool {
    matches!(input_type.unwrap_or("text"), "checkbox" | "radio")
}

fn is_submit_control(tag_name: &str, input_type: Option<&str>) -> bool {
    match tag_name {
        "button" => !matches!(input_type, Some("button") | Some("reset")),
        "input" => matches!(input_type.unwrap_or("text"), "submit" | "image"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::Session;
    use super::SessionConfig;
    use super::SessionEventTarget;

    #[test]
    fn session_bootstraps_empty_dom_and_storage_seed() {
        let mut local_storage = BTreeMap::new();
        local_storage.insert("token".to_string(), "abc".to_string());
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some("<main id='app'></main>".to_string()),
            local_storage,
        };

        let session = Session::new(config).expect("session should parse HTML");
        assert_eq!(session.dom().source_html(), Some("<main id='app'></main>"));
        assert_eq!(session.dom().node_count(), 2);
        assert_eq!(
            session
                .mocks()
                .storage()
                .local()
                .get("token")
                .map(String::as_str),
            Some("abc")
        );
    }

    #[test]
    fn session_rejects_malformed_html() {
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some("<main><span></main>".to_string()),
            local_storage: BTreeMap::new(),
        };

        let error = Session::new(config).expect_err("malformed HTML should fail");
        assert!(error.to_string().contains("mismatched closing tag"));
    }

    #[test]
    fn session_bootstraps_inline_scripts_in_document_order() {
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some(
                "<main id='out'></main><script>document.getElementById('out').textContent = 'Hello';</script>"
                    .to_string(),
            ),
            local_storage: BTreeMap::new(),
        };

        let session = Session::new(config).expect("session should execute inline scripts");
        assert_eq!(
            session.dom().dump_dom(),
            "#document\n  <main id=\"out\">\n    \"Hello\"\n  </main>\n  <script>\n    \"document.getElementById('out').textContent = 'Hello';\"\n  </script>"
        );
    }

    #[test]
    fn session_registers_event_listeners_from_inline_scripts() {
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some(
                "<button id='run'></button><script>document.getElementById('run').addEventListener('click', () => { document.getElementById('run').textContent = 'clicked'; });</script>"
                    .to_string(),
            ),
            local_storage: BTreeMap::new(),
        };

        let session = Session::new(config).expect("session should register listeners");
        assert_eq!(session.script_event_listeners.len(), 1);
        assert_eq!(session.script_event_listeners[0].event_type, "click");
        assert!(!session.script_event_listeners[0].capture);
        match &session.script_event_listeners[0].target {
            SessionEventTarget::Element(node_id) => assert_eq!(node_id.index(), 1),
            other => panic!("unexpected listener target: {:?}", other),
        }
        assert!(
            session.script_event_listeners[0]
                .handler
                .body_source
                .contains("textContent = 'clicked'")
        );
    }

    #[test]
    fn session_bubbles_click_events_beyond_target_phase() {
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some(
                "<div id='parent'><div id='child'></div></div><div id='out'></div><script>document.getElementById('child').addEventListener('click', () => { document.getElementById('out').textContent = 'target'; }); document.getElementById('parent').addEventListener('click', () => { document.getElementById('out').textContent += ':parent'; }); document.addEventListener('click', () => { document.getElementById('out').textContent += ':document'; }); window.addEventListener('click', () => { document.getElementById('out').textContent += ':window'; });</script>"
                    .to_string(),
            ),
            local_storage: BTreeMap::new(),
        };

        let mut session = Session::new(config).expect("session should register listeners");
        let child_id = session.dom().select("#child").unwrap()[0];
        let out_id = session.dom().select("#out").unwrap()[0];

        session.click_node(child_id).expect("click should bubble");

        assert_eq!(
            session.dom().text_content_for_node(out_id),
            "target:parent:document:window"
        );
    }

    #[test]
    fn session_stop_propagation_blocks_ancestor_listeners() {
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some(
                "<div id='parent'><div id='child'></div></div><div id='out'></div><script>document.getElementById('child').addEventListener('click', (event) => { event.stopPropagation(); document.getElementById('out').textContent = 'target'; }); document.getElementById('parent').addEventListener('click', () => { document.getElementById('out').textContent += ':parent'; }); document.addEventListener('click', () => { document.getElementById('out').textContent += ':document'; });</script>"
                    .to_string(),
            ),
            local_storage: BTreeMap::new(),
        };

        let mut session = Session::new(config).expect("session should register listeners");
        let child_id = session.dom().select("#child").unwrap()[0];
        let out_id = session.dom().select("#out").unwrap()[0];

        session
            .click_node(child_id)
            .expect("click should still succeed");

        assert_eq!(session.dom().text_content_for_node(out_id), "target");
    }

    #[test]
    fn session_click_default_action_is_cancelable() {
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some(
                "<input id='agree' type='checkbox'><div id='out'></div><script>document.getElementById('agree').addEventListener('click', (event) => { event.preventDefault(); }); document.getElementById('agree').addEventListener('change', () => { document.getElementById('out').textContent = String(document.getElementById('agree').checked); });</script>"
                    .to_string(),
            ),
            local_storage: BTreeMap::new(),
        };

        let mut session = Session::new(config).expect("session should register listeners");
        let agree_id = session.dom().select("#agree").unwrap()[0];
        let out_id = session.dom().select("#out").unwrap()[0];

        session
            .click_node(agree_id)
            .expect("canceling click should still succeed");

        assert_eq!(session.dom().checked_for_node(agree_id), Some(false));
        assert_eq!(session.dom().text_content_for_node(out_id), "");
    }

    #[test]
    fn session_focus_and_blur_dispatch_in_order() {
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some(
                "<input id='first'><input id='second'><div id='out'></div><script>document.getElementById('first').addEventListener('blur', () => { document.getElementById('second').textContent = 'after-blur'; }); document.getElementById('second').addEventListener('focus', () => { document.getElementById('out').textContent = document.getElementById('second').textContent; });</script>"
                    .to_string(),
            ),
            local_storage: BTreeMap::new(),
        };

        let mut session = Session::new(config).expect("session should register listeners");
        let first_id = session.dom().select("#first").unwrap()[0];
        let second_id = session.dom().select("#second").unwrap()[0];
        let out_id = session.dom().select("#out").unwrap()[0];

        session.focus_node(first_id).expect("focus should work");
        session
            .focus_node(second_id)
            .expect("focus should blur the previous element");

        assert_eq!(session.dom().text_content_for_node(second_id), "after-blur");
        assert_eq!(session.dom().text_content_for_node(out_id), "after-blur");
    }

    #[test]
    fn session_set_select_value_updates_option_state_and_dispatches_change() {
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some(
                "<select id='mode'><option value='a'>A</option><option value='b'>B</option></select><div id='out'></div><script>document.getElementById('mode').addEventListener('change', () => { document.getElementById('out').textContent = document.getElementById('mode').value; });</script>"
                    .to_string(),
            ),
            local_storage: BTreeMap::new(),
        };

        let mut session = Session::new(config).expect("session should register listeners");
        let mode_id = session.dom().select("#mode").unwrap()[0];
        let option_ids = session.dom().select("option").unwrap();
        let out_id = session.dom().select("#out").unwrap()[0];

        session
            .set_select_value_node(mode_id, "b")
            .expect("select should accept a matching value");

        assert_eq!(session.dom().value_for_node(mode_id), "b");
        assert_eq!(
            session.dom().select("[selected]").unwrap(),
            vec![option_ids[1]]
        );
        assert_eq!(session.dom().text_content_for_node(out_id), "b");
    }

    #[test]
    fn session_bootstraps_form_control_state_through_script_bindings() {
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some(
                "<input id='name'><input id='agree' type='checkbox'><div id='out'></div><script>document.getElementById('name').value = 'Alice'; document.getElementById('agree').checked = true; document.getElementById('out').textContent = document.getElementById('name').value + ':' + String(document.getElementById('agree').checked);</script>"
                    .to_string(),
            ),
            local_storage: BTreeMap::new(),
        };

        let session = Session::new(config).expect("session should execute form-control scripts");
        let name_id = session.dom().select("#name").unwrap()[0];
        let agree_id = session.dom().select("#agree").unwrap()[0];
        let out_id = session.dom().select("#out").unwrap()[0];

        assert_eq!(session.dom().value_for_node(name_id), "Alice");
        assert_eq!(session.dom().checked_for_node(agree_id), Some(true));
        assert_eq!(session.dom().text_content_for_node(out_id), "Alice:true");
    }

    #[test]
    fn session_click_toggles_checkbox_and_dispatches_input_listener() {
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some(
                "<input id='agree' type='checkbox'><div id='out'></div><script>document.getElementById('agree').addEventListener('input', () => { document.getElementById('out').textContent = String(document.getElementById('agree').checked); });</script>"
                    .to_string(),
            ),
            local_storage: BTreeMap::new(),
        };

        let mut session = Session::new(config).expect("session should register listeners");
        let agree_id = session.dom().select("#agree").unwrap()[0];
        let out_id = session.dom().select("#out").unwrap()[0];

        session
            .click_node(agree_id)
            .expect("click should toggle checkbox");

        assert_eq!(session.dom().checked_for_node(agree_id), Some(true));
        assert_eq!(session.dom().text_content_for_node(out_id), "true");
    }

    #[test]
    fn session_clicking_submit_button_dispatches_form_submit_listener() {
        let config = SessionConfig {
            url: "https://app.local/".to_string(),
            html: Some(
                "<form id='profile'><input id='name'><button id='submit' type='submit'>Save</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', () => { document.getElementById('out').textContent = document.getElementById('name').value; });</script>"
                    .to_string(),
            ),
            local_storage: BTreeMap::new(),
        };

        let mut session = Session::new(config).expect("session should register submit listener");
        let name_id = session.dom().select("#name").unwrap()[0];
        let submit_id = session.dom().select("#submit").unwrap()[0];
        let out_id = session.dom().select("#out").unwrap()[0];

        session
            .type_text_node(name_id, "Alice")
            .expect("typing should update the input");
        session
            .click_node(submit_id)
            .expect("clicking submit should dispatch submit");

        assert_eq!(session.dom().value_for_node(name_id), "Alice");
        assert_eq!(session.dom().text_content_for_node(out_id), "Alice");
    }
}
