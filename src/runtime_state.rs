use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScriptHandler {
    pub(crate) params: Vec<FunctionParam>,
    pub(crate) stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FunctionParam {
    pub(crate) name: String,
    pub(crate) default: Option<Expr>,
    pub(crate) is_rest: bool,
}

impl ScriptHandler {
    pub(crate) fn first_event_param(&self) -> Option<&str> {
        self.params.first().map(|param| param.name.as_str())
    }

    pub(crate) fn listener_callback_reference(&self) -> Option<&str> {
        if self.params.len() != 1 || self.stmts.len() != 1 {
            return None;
        }
        let event_param = self.params[0].name.as_str();
        match &self.stmts[0] {
            Stmt::Expr(Expr::FunctionCall { target, args }) if args.len() == 1 => match &args[0] {
                Expr::Var(arg_name) if arg_name == event_param => Some(target.as_str()),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TimerCallback {
    Inline(ScriptHandler),
    Reference(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TimerInvocation {
    pub(crate) callback: TimerCallback,
    pub(crate) args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocationParts {
    pub(crate) scheme: String,
    pub(crate) has_authority: bool,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) hostname: String,
    pub(crate) port: String,
    pub(crate) pathname: String,
    pub(crate) opaque_path: String,
    pub(crate) search: String,
    pub(crate) hash: String,
}

impl LocationParts {
    pub(crate) fn protocol(&self) -> String {
        format!("{}:", self.scheme)
    }

    pub(crate) fn host(&self) -> String {
        if self.port.is_empty() {
            self.hostname.clone()
        } else {
            format!("{}:{}", self.hostname, self.port)
        }
    }

    pub(crate) fn origin(&self) -> String {
        if self.has_authority && !self.hostname.is_empty() {
            format!("{}//{}", self.protocol(), self.host())
        } else {
            "null".to_string()
        }
    }

    pub(crate) fn href(&self) -> String {
        if self.has_authority {
            let path = if self.pathname.is_empty() {
                "/".to_string()
            } else {
                self.pathname.clone()
            };
            let credentials = if self.username.is_empty() && self.password.is_empty() {
                String::new()
            } else if self.password.is_empty() {
                format!("{}@", self.username)
            } else {
                format!("{}:{}@", self.username, self.password)
            };
            format!(
                "{}//{}{}{}{}{}",
                self.protocol(),
                credentials,
                self.host(),
                path,
                self.search,
                self.hash
            )
        } else {
            format!(
                "{}{}{}{}",
                self.protocol(),
                self.opaque_path,
                self.search,
                self.hash
            )
        }
    }

    pub(crate) fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        let scheme_end = trimmed.find(':')?;
        let scheme = trimmed[..scheme_end].to_ascii_lowercase();
        if !is_valid_url_scheme(&scheme) {
            return None;
        }
        let rest = &trimmed[scheme_end + 1..];
        if let Some(without_slashes) = rest.strip_prefix("//") {
            let authority_end = without_slashes
                .find(|ch| ['/', '?', '#'].contains(&ch))
                .unwrap_or(without_slashes.len());
            let authority = &without_slashes[..authority_end];
            let tail = &without_slashes[authority_end..];
            let (username, password, hostname, port) = split_authority_components(authority);
            let (pathname, search, hash) = split_path_search_hash(tail);
            let pathname = if pathname.is_empty() {
                "/".to_string()
            } else {
                normalize_pathname(&pathname)
            };
            Some(Self {
                scheme,
                has_authority: true,
                username,
                password,
                hostname,
                port,
                pathname,
                opaque_path: String::new(),
                search,
                hash,
            })
        } else {
            let (opaque_path, search, hash) = split_opaque_search_hash(rest);
            Some(Self {
                scheme,
                has_authority: false,
                username: String::new(),
                password: String::new(),
                hostname: String::new(),
                port: String::new(),
                pathname: String::new(),
                opaque_path,
                search,
                hash,
            })
        }
    }
}

pub(crate) fn is_valid_url_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

pub(crate) fn split_hostname_and_port(authority: &str) -> (String, String) {
    if authority.is_empty() {
        return (String::new(), String::new());
    }

    if let Some(rest) = authority.strip_prefix('[') {
        if let Some(end_idx) = rest.find(']') {
            let hostname = authority[..end_idx + 2].to_string();
            let suffix = &authority[end_idx + 2..];
            if let Some(port) = suffix.strip_prefix(':') {
                return (hostname, port.to_string());
            }
            return (hostname, String::new());
        }
    }

    if let Some(idx) = authority.rfind(':') {
        let hostname = &authority[..idx];
        let port = &authority[idx + 1..];
        if !hostname.contains(':') {
            return (hostname.to_string(), port.to_string());
        }
    }
    (authority.to_string(), String::new())
}

pub(crate) fn split_authority_components(authority: &str) -> (String, String, String, String) {
    if authority.is_empty() {
        return (String::new(), String::new(), String::new(), String::new());
    }

    let (userinfo, hostport) = if let Some(at) = authority.rfind('@') {
        (&authority[..at], &authority[at + 1..])
    } else {
        ("", authority)
    };

    let (username, password) = if userinfo.is_empty() {
        (String::new(), String::new())
    } else if let Some((username, password)) = userinfo.split_once(':') {
        (username.to_string(), password.to_string())
    } else {
        (userinfo.to_string(), String::new())
    };

    let (hostname, port) = split_hostname_and_port(hostport);
    (username, password, hostname, port)
}

pub(crate) fn split_path_search_hash(tail: &str) -> (String, String, String) {
    let mut pathname = tail;
    let mut search = "";
    let mut hash = "";

    if let Some(hash_pos) = tail.find('#') {
        pathname = &tail[..hash_pos];
        hash = &tail[hash_pos..];
    }

    if let Some(search_pos) = pathname.find('?') {
        search = &pathname[search_pos..];
        pathname = &pathname[..search_pos];
    }

    (pathname.to_string(), search.to_string(), hash.to_string())
}

pub(crate) fn split_opaque_search_hash(rest: &str) -> (String, String, String) {
    let mut opaque_path = rest;
    let mut search = "";
    let mut hash = "";

    if let Some(hash_pos) = rest.find('#') {
        opaque_path = &rest[..hash_pos];
        hash = &rest[hash_pos..];
    }

    if let Some(search_pos) = opaque_path.find('?') {
        search = &opaque_path[search_pos..];
        opaque_path = &opaque_path[..search_pos];
    }

    (
        opaque_path.to_string(),
        search.to_string(),
        hash.to_string(),
    )
}

pub(crate) fn normalize_pathname(pathname: &str) -> String {
    let starts_with_slash = pathname.starts_with('/');
    let ends_with_slash = pathname.ends_with('/') && pathname.len() > 1;
    let mut parts = Vec::new();
    for segment in pathname.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            parts.pop();
            continue;
        }
        parts.push(segment);
    }
    let mut out = if starts_with_slash {
        format!("/{}", parts.join("/"))
    } else {
        parts.join("/")
    };
    if out.is_empty() {
        out.push('/');
    }
    if ends_with_slash && !out.ends_with('/') {
        out.push('/');
    }
    out
}

pub(crate) fn ensure_search_prefix(value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else if value.starts_with('?') {
        value.to_string()
    } else {
        format!("?{value}")
    }
}

pub(crate) fn ensure_hash_prefix(value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else if value.starts_with('#') {
        value.to_string()
    } else {
        format!("#{value}")
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Listener {
    pub(crate) capture: bool,
    pub(crate) handler: ScriptHandler,
    pub(crate) captured_env: Rc<RefCell<ScriptEnv>>,
    pub(crate) captured_pending_function_decls:
        Vec<Arc<HashMap<String, (ScriptHandler, bool, bool)>>>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ListenerStore {
    pub(crate) map: HashMap<NodeId, HashMap<String, Vec<Listener>>>,
}

impl ListenerStore {
    pub(crate) fn add(&mut self, node_id: NodeId, event: String, listener: Listener) {
        let listeners = self
            .map
            .entry(node_id)
            .or_default()
            .entry(event)
            .or_default();

        // Match browser semantics: dedupe only when the same callback reference
        // is re-registered for the same type/capture pair.
        if let Some(new_callback_ref) = listener.handler.listener_callback_reference() {
            if listeners.iter().any(|existing| {
                existing.capture == listener.capture
                    && existing
                        .handler
                        .listener_callback_reference()
                        .is_some_and(|existing_ref| existing_ref == new_callback_ref)
            }) {
                return;
            }
        }

        listeners.push(listener);
    }

    pub(crate) fn remove(
        &mut self,
        node_id: NodeId,
        event: &str,
        capture: bool,
        handler: &ScriptHandler,
    ) -> bool {
        let Some(events) = self.map.get_mut(&node_id) else {
            return false;
        };
        let Some(listeners) = events.get_mut(event) else {
            return false;
        };

        if let Some(pos) = listeners
            .iter()
            .position(|listener| listener.capture == capture && listener.handler == *handler)
        {
            listeners.remove(pos);
            if listeners.is_empty() {
                events.remove(event);
            }
            if events.is_empty() {
                self.map.remove(&node_id);
            }
            return true;
        }

        false
    }

    pub(crate) fn get(&self, node_id: NodeId, event: &str, capture: bool) -> Vec<Listener> {
        self.map
            .get(&node_id)
            .and_then(|events| events.get(event))
            .map(|listeners| {
                listeners
                    .iter()
                    .filter(|listener| listener.capture == capture)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EventState {
    pub(crate) event_type: String,
    pub(crate) target: NodeId,
    pub(crate) current_target: NodeId,
    pub(crate) event_phase: i32,
    pub(crate) time_stamp_ms: i64,
    pub(crate) default_prevented: bool,
    pub(crate) is_trusted: bool,
    pub(crate) bubbles: bool,
    pub(crate) cancelable: bool,
    pub(crate) state: Option<Value>,
    pub(crate) old_state: Option<String>,
    pub(crate) new_state: Option<String>,
    pub(crate) propagation_stopped: bool,
    pub(crate) immediate_propagation_stopped: bool,
}

impl EventState {
    pub(crate) fn new(event_type: &str, target: NodeId, time_stamp_ms: i64) -> Self {
        Self {
            event_type: event_type.to_string(),
            target,
            current_target: target,
            event_phase: 2,
            time_stamp_ms,
            default_prevented: false,
            is_trusted: true,
            bubbles: true,
            cancelable: true,
            state: None,
            old_state: None,
            new_state: None,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
        }
    }

    pub(crate) fn new_untrusted(event_type: &str, target: NodeId, time_stamp_ms: i64) -> Self {
        let mut event = Self::new(event_type, target, time_stamp_ms);
        event.is_trusted = false;
        event.bubbles = false;
        event.cancelable = false;
        event
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScriptSource {
    pub(crate) code: String,
    pub(crate) is_module: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ParseOutput {
    pub(crate) dom: Dom,
    pub(crate) scripts: Vec<ScriptSource>,
}

#[derive(Debug, Clone)]
pub(crate) struct ScheduledTask {
    pub(crate) id: i64,
    pub(crate) due_at: i64,
    pub(crate) order: i64,
    pub(crate) interval_ms: Option<i64>,
    pub(crate) callback: TimerCallback,
    pub(crate) callback_args: Vec<Value>,
    pub(crate) env: ScriptEnv,
}

#[derive(Debug, Clone)]
pub(crate) enum ScheduledMicrotask {
    Script {
        handler: ScriptHandler,
        env: ScriptEnv,
    },
    Promise {
        reaction: PromiseReactionKind,
        settled: PromiseSettledValue,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTimer {
    pub id: i64,
    pub due_at: i64,
    pub order: i64,
    pub interval_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocationNavigationKind {
    Assign,
    Replace,
    HrefSet,
    Reload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationNavigation {
    pub kind: LocationNavigationKind,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadArtifact {
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HistoryEntry {
    pub(crate) url: String,
    pub(crate) state: Value,
}

#[derive(Debug)]
pub(crate) struct LocationHistoryState {
    pub(crate) history_object: Rc<RefCell<ObjectValue>>,
    pub(crate) history_entries: Vec<HistoryEntry>,
    pub(crate) history_index: usize,
    pub(crate) history_scroll_restoration: String,
    pub(crate) location_mock_pages: HashMap<String, String>,
    pub(crate) location_navigations: Vec<LocationNavigation>,
    pub(crate) location_reload_count: usize,
}

impl LocationHistoryState {
    pub(crate) fn new(initial_url: &str) -> Self {
        Self {
            history_object: Rc::new(RefCell::new(ObjectValue::default())),
            history_entries: vec![HistoryEntry {
                url: initial_url.to_string(),
                state: Value::Null,
            }],
            history_index: 0,
            history_scroll_restoration: "auto".to_string(),
            location_mock_pages: HashMap::new(),
            location_navigations: Vec::new(),
            location_reload_count: 0,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ScriptEnv {
    pub(crate) inner: Arc<HashMap<String, Value>>,
}

impl ScriptEnv {
    pub(crate) fn share(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    pub(crate) fn from_snapshot(env: &HashMap<String, Value>) -> Self {
        Self {
            inner: Arc::new(env.clone()),
        }
    }

    pub(crate) fn to_map(&self) -> HashMap<String, Value> {
        self.inner.as_ref().clone()
    }
}

impl std::ops::Deref for ScriptEnv {
    type Target = HashMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl std::ops::DerefMut for ScriptEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.inner)
    }
}

#[derive(Debug, Default)]
pub(crate) struct ListenerCaptureFrame {
    pub(crate) shared_env: Option<Rc<RefCell<ScriptEnv>>>,
    pub(crate) pending_env_updates: HashMap<String, Option<Value>>,
}

#[derive(Debug, Clone)]
pub(crate) enum ModuleExportBinding {
    Local(String),
    Value(Value),
}

#[derive(Debug, Default, Clone)]
pub(crate) struct TdzScopeFrame {
    pub(crate) declared: HashSet<String>,
    pub(crate) pending: HashSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrivateBindingKind {
    Field,
    Method,
    Accessor,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PrivateBindingRuntime {
    pub(crate) name: String,
    pub(crate) slot_id: usize,
    pub(crate) is_static: bool,
    pub(crate) kind: PrivateBindingKind,
    pub(crate) has_getter: bool,
    pub(crate) has_setter: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PrivateInitializerRuntime {
    pub(crate) binding: PrivateBindingRuntime,
    pub(crate) initializer: Option<Expr>,
    pub(crate) value: Option<Value>,
    pub(crate) setter_value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PublicFieldInitializerRuntime {
    pub(crate) name: String,
    pub(crate) initializer: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ConstructorInstanceInitializerRuntime {
    Private(PrivateInitializerRuntime),
    Public(PublicFieldInitializerRuntime),
}

#[derive(Debug, Default)]
pub(crate) struct ScriptRuntimeState {
    pub(crate) env: ScriptEnv,
    pub(crate) pending_function_decls: Vec<Arc<HashMap<String, (ScriptHandler, bool, bool)>>>,
    pub(crate) listener_capture_env_stack: Vec<ListenerCaptureFrame>,
    pub(crate) generator_yield_stack: Vec<Rc<RefCell<Vec<Value>>>>,
    pub(crate) pending_loop_labels: Vec<Vec<String>>,
    pub(crate) loop_label_stack: Vec<HashSet<String>>,
    pub(crate) tdz_scope_stack: Vec<TdzScopeFrame>,
    pub(crate) module_export_stack: Vec<Rc<RefCell<HashMap<String, ModuleExportBinding>>>>,
    pub(crate) module_referrer_stack: Vec<String>,
    pub(crate) module_cache: HashMap<String, HashMap<String, Value>>,
    pub(crate) module_namespace_cache: HashMap<String, Value>,
    pub(crate) loading_modules: HashSet<String>,
    pub(crate) next_function_id: usize,
    pub(crate) next_private_slot_id: usize,
    pub(crate) function_private_bindings: HashMap<usize, HashMap<String, PrivateBindingRuntime>>,
    pub(crate) function_public_properties: HashMap<usize, ObjectValue>,
    pub(crate) constructor_instance_initializers:
        HashMap<usize, Vec<ConstructorInstanceInitializerRuntime>>,
    pub(crate) constructor_call_stack: Vec<usize>,
    pub(crate) constructor_instance_initialized_stack: Vec<bool>,
    pub(crate) private_binding_stack: Vec<HashMap<String, PrivateBindingRuntime>>,
    pub(crate) private_instance_slots: HashMap<usize, HashMap<usize, Value>>,
    pub(crate) private_static_slots: HashMap<usize, HashMap<usize, Value>>,
}

impl ScriptRuntimeState {
    pub(crate) fn allocate_function_id(&mut self) -> usize {
        let id = self.next_function_id;
        self.next_function_id = self.next_function_id.saturating_add(1);
        id
    }

    pub(crate) fn allocate_private_slot_id(&mut self) -> usize {
        let id = self.next_private_slot_id;
        self.next_private_slot_id = self.next_private_slot_id.saturating_add(1);
        id
    }
}

#[derive(Debug)]
pub(crate) struct DomRuntimeState {
    pub(crate) window_object: Rc<RefCell<ObjectValue>>,
    pub(crate) document_object: Rc<RefCell<ObjectValue>>,
    pub(crate) location_object: Rc<RefCell<ObjectValue>>,
    pub(crate) node_event_handler_props: HashMap<(NodeId, String), ScriptHandler>,
    pub(crate) node_expando_props: HashMap<(NodeId, String), Value>,
    pub(crate) dialog_return_values: HashMap<NodeId, String>,
}

impl Default for DomRuntimeState {
    fn default() -> Self {
        Self {
            window_object: Rc::new(RefCell::new(ObjectValue::default())),
            document_object: Rc::new(RefCell::new(ObjectValue::default())),
            location_object: Rc::new(RefCell::new(ObjectValue::default())),
            node_event_handler_props: HashMap::new(),
            node_expando_props: HashMap::new(),
            dialog_return_values: HashMap::new(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct PlatformMockState {
    pub(crate) clipboard_text: String,
    pub(crate) fetch_mocks: HashMap<String, String>,
    pub(crate) fetch_calls: Vec<String>,
    pub(crate) match_media_mocks: HashMap<String, bool>,
    pub(crate) match_media_calls: Vec<String>,
    pub(crate) default_match_media_matches: bool,
    pub(crate) alert_messages: Vec<String>,
    pub(crate) confirm_responses: VecDeque<bool>,
    pub(crate) default_confirm_response: bool,
    pub(crate) prompt_responses: VecDeque<Option<String>>,
    pub(crate) default_prompt_response: Option<String>,
}

#[derive(Debug)]
pub(crate) struct TraceState {
    pub(crate) enabled: bool,
    pub(crate) events: bool,
    pub(crate) timers: bool,
    pub(crate) logs: VecDeque<String>,
    pub(crate) log_limit: usize,
    pub(crate) to_stderr: bool,
}

impl Default for TraceState {
    fn default() -> Self {
        Self {
            enabled: false,
            events: true,
            timers: true,
            logs: VecDeque::new(),
            log_limit: 10_000,
            to_stderr: true,
        }
    }
}

#[derive(Debug)]
pub(crate) struct BrowserApiState {
    pub(crate) next_url_object_id: usize,
    pub(crate) url_objects: HashMap<usize, Rc<RefCell<ObjectValue>>>,
    pub(crate) url_constructor_properties: Rc<RefCell<ObjectValue>>,
    pub(crate) local_storage_object: Rc<RefCell<ObjectValue>>,
    pub(crate) next_blob_url_id: usize,
    pub(crate) blob_url_objects: HashMap<String, Rc<RefCell<BlobValue>>>,
    pub(crate) downloads: Vec<DownloadArtifact>,
}

impl Default for BrowserApiState {
    fn default() -> Self {
        Self {
            next_url_object_id: 1,
            url_objects: HashMap::new(),
            url_constructor_properties: Rc::new(RefCell::new(ObjectValue::default())),
            local_storage_object: Rc::new(RefCell::new(ObjectValue::default())),
            next_blob_url_id: 1,
            blob_url_objects: HashMap::new(),
            downloads: Vec::new(),
        }
    }
}

impl BrowserApiState {
    pub(crate) fn allocate_url_object_id(&mut self) -> usize {
        let id = self.next_url_object_id;
        self.next_url_object_id = self.next_url_object_id.saturating_add(1);
        id
    }

    pub(crate) fn allocate_blob_url(&mut self) -> String {
        let object_url = format!("blob:bt-{}", self.next_blob_url_id);
        self.next_blob_url_id = self.next_blob_url_id.saturating_add(1);
        object_url
    }
}

#[derive(Debug)]
pub(crate) struct PromiseRuntimeState {
    pub(crate) next_promise_id: usize,
}

impl Default for PromiseRuntimeState {
    fn default() -> Self {
        Self { next_promise_id: 1 }
    }
}

impl PromiseRuntimeState {
    pub(crate) fn allocate_promise_id(&mut self) -> usize {
        let id = self.next_promise_id;
        self.next_promise_id = self.next_promise_id.saturating_add(1);
        id
    }
}

#[derive(Debug)]
pub(crate) struct SymbolRuntimeState {
    pub(crate) next_symbol_id: usize,
    pub(crate) symbol_registry: HashMap<String, Rc<SymbolValue>>,
    pub(crate) symbols_by_id: HashMap<usize, Rc<SymbolValue>>,
    pub(crate) well_known_symbols: HashMap<String, Rc<SymbolValue>>,
}

impl Default for SymbolRuntimeState {
    fn default() -> Self {
        Self {
            next_symbol_id: 1,
            symbol_registry: HashMap::new(),
            symbols_by_id: HashMap::new(),
            well_known_symbols: HashMap::new(),
        }
    }
}

impl SymbolRuntimeState {
    pub(crate) fn allocate_symbol_id(&mut self) -> usize {
        let id = self.next_symbol_id;
        self.next_symbol_id = self.next_symbol_id.saturating_add(1);
        id
    }
}

#[derive(Debug)]
pub(crate) struct SchedulerState {
    pub(crate) task_queue: Vec<ScheduledTask>,
    pub(crate) microtask_queue: VecDeque<ScheduledMicrotask>,
    pub(crate) now_ms: i64,
    pub(crate) timer_step_limit: usize,
    pub(crate) next_timer_id: i64,
    pub(crate) next_task_order: i64,
    pub(crate) task_depth: usize,
    pub(crate) running_timer_id: Option<i64>,
    pub(crate) running_timer_canceled: bool,
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self {
            task_queue: Vec::new(),
            microtask_queue: VecDeque::new(),
            now_ms: 0,
            timer_step_limit: 10_000,
            next_timer_id: 1,
            next_task_order: 0,
            task_depth: 0,
            running_timer_id: None,
            running_timer_canceled: false,
        }
    }
}

impl SchedulerState {
    pub(crate) fn allocate_timer_id(&mut self) -> i64 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        id
    }

    pub(crate) fn allocate_task_order(&mut self) -> i64 {
        let order = self.next_task_order;
        self.next_task_order += 1;
        order
    }
}
