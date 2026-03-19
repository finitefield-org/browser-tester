use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;

use bt_dom::{DomStore, NodeId, NodeKind};
use bt_script::{
    ElementHandle, HostBindings, ListenerTarget, ScriptError, ScriptFunction, ScriptRuntime,
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
    }

    pub fn advance_time_to(&mut self, target_ms: i64) {
        self.now_ms = target_ms;
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

    pub fn confirm_queue(&self) -> &[bool] {
        &self.confirm_queue
    }

    pub fn prompt_queue(&self) -> &[Option<String>] {
        &self.prompt_queue
    }

    pub fn alert_messages(&self) -> &[String] {
        &self.alert_messages
    }

    pub fn reset(&mut self) {
        self.confirm_queue.clear();
        self.prompt_queue.clear();
        self.alert_messages.clear();
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
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HtmlParse(message) => write!(f, "HTML parse error: {message}"),
            Self::Script(err) => write!(f, "Script error: {err}"),
            Self::Selector(message) => write!(f, "Selector error: {message}"),
            Self::Dom(message) => write!(f, "DOM error: {message}"),
            Self::Event(message) => write!(f, "Event error: {message}"),
        }
    }
}

impl StdError for SessionError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::HtmlParse(_) => None,
            Self::Script(err) => Some(err),
            Self::Selector(_) | Self::Dom(_) | Self::Event(_) => None,
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
    handler: ScriptFunction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

        self.ensure_element_node(node_id)?;
        self.dispatch_listeners(SessionEventTarget::Element(node_id), event_type)?;
        Ok(())
    }

    pub fn click_node(&mut self, node_id: NodeId) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        self.dispatch_node(node_id, "click")?;
        self.run_click_default_actions(node_id)
    }

    pub fn type_text_node(&mut self, node_id: NodeId, text: &str) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        self.dom
            .set_form_control_value(node_id, text)
            .map_err(SessionError::Dom)?;
        self.dispatch_node(node_id, "input")
    }

    pub fn set_checked_node(&mut self, node_id: NodeId, checked: bool) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        self.dom
            .set_form_control_checked(node_id, checked)
            .map_err(SessionError::Dom)?;
        self.dispatch_node(node_id, "input")?;
        self.dispatch_node(node_id, "change")
    }

    pub fn submit_node(&mut self, node_id: NodeId) -> Result<(), SessionError> {
        self.ensure_element_node(node_id)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(SessionError::Dom(format!("invalid node id: {:?}", node_id)));
        };
        if matches!(&node.kind, NodeKind::Element(element) if element.tag_name == "form") {
            return self.dispatch_node(node_id, "submit");
        }

        let Some(form_id) = self.find_associated_form(node_id) else {
            return Err(SessionError::Dom(format!(
                "submit is only supported on <form> elements or submit controls with an associated form, not {:?}",
                node_id
            )));
        };

        self.dispatch_node(form_id, "submit")
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

    fn dispatch_listeners(
        &mut self,
        target: SessionEventTarget,
        event_type: &str,
    ) -> Result<(), SessionError> {
        let listeners: Vec<ScriptListenerRecord> = self
            .script_event_listeners
            .iter()
            .filter(|listener| listener.target == target && listener.event_type == event_type)
            .cloned()
            .collect();

        for (index, listener) in listeners.iter().enumerate() {
            let source_name = format!("event:{event_type}:{index}");
            self.eval_script_source(&listener.handler.body_source, &source_name)?;
        }

        Ok(())
    }

    fn eval_script_source(
        &mut self,
        source: &str,
        source_name: &str,
    ) -> Result<(), SessionError> {
        let mut script = std::mem::take(&mut self.script);
        let result = script
            .eval_program(source, source_name, self)
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
                    self.dispatch_node(node_id, "input")?;
                    self.dispatch_node(node_id, "change")?;
                }
                DefaultActionKind::SubmitButton
                    if is_submit_control(tag_name.as_str(), input_type.as_deref()) =>
                {
                    if let Some(form_id) = self.find_associated_form(node_id) {
                        self.dispatch_node(form_id, "submit")?;
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

    fn register_script_listener(
        &mut self,
        target: SessionEventTarget,
        event_type: String,
        handler: ScriptFunction,
    ) {
        self.script_event_listeners.push(ScriptListenerRecord {
            target,
            event_type,
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

    fn element_text_content(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        let node_id = self.node_id_for_handle(element)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };

        match &node.kind {
            NodeKind::Element(_) | NodeKind::Document | NodeKind::Text(_) | NodeKind::Comment(_) => {
                Ok(self.dom.text_content_for_node(node_id))
            }
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

    fn element_value(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        let node_id = self.node_id_for_handle(element)?;
        let Some(node) = self.dom.nodes().get(node_id.index() as usize) else {
            return Err(ScriptError::new("invalid element handle"));
        };
        match &node.kind {
            NodeKind::Element(_) | NodeKind::Document | NodeKind::Text(_) | NodeKind::Comment(_) => {
                Ok(self.dom.value_for_node(node_id))
            }
        }
    }

    fn element_set_value(
        &mut self,
        element: ElementHandle,
        value: &str,
    ) -> bt_script::Result<()> {
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

    fn register_event_listener(
        &mut self,
        target: ListenerTarget,
        event_type: &str,
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

        self.register_script_listener(target, event_type.to_string(), handler);
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
        assert_eq!(
            session.script_event_listeners[0].event_type,
            "click"
        );
        match &session.script_event_listeners[0].target {
            SessionEventTarget::Element(node_id) => assert_eq!(node_id.index(), 1),
            other => panic!("unexpected listener target: {:?}", other),
        }
        assert!(session.script_event_listeners[0]
            .handler
            .body_source
            .contains("textContent = 'clicked'"));
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

        session.click_node(agree_id).expect("click should toggle checkbox");

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
