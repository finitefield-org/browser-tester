use std::collections::BTreeMap;

use bt_dom::DomStore;
use bt_script::ScriptRuntime;

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
}

impl Session {
    pub fn new(config: SessionConfig) -> Result<Self, String> {
        let mut dom = DomStore::new_empty();
        if let Some(html) = &config.html {
            dom.bootstrap_html(html.clone())?;
        }

        let mut mocks = MockRegistry::default();
        for (key, value) in &config.local_storage {
            mocks.storage_mut().seed_local(key.clone(), value.clone());
        }

        Ok(Self {
            dom,
            scheduler: Scheduler::default(),
            mocks,
            script: ScriptRuntime::default(),
            config,
            debug: DebugState::default(),
        })
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
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::Session;
    use super::SessionConfig;

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
        assert!(error.contains("mismatched closing tag"));
    }
}
