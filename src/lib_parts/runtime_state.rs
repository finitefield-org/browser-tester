#[derive(Debug, Clone, PartialEq)]
struct ScriptHandler {
    params: Vec<FunctionParam>,
    stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
struct FunctionParam {
    name: String,
    default: Option<Expr>,
    is_rest: bool,
}

impl ScriptHandler {
    fn first_event_param(&self) -> Option<&str> {
        self.params.first().map(|param| param.name.as_str())
    }

    fn listener_callback_reference(&self) -> Option<&str> {
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
enum TimerCallback {
    Inline(ScriptHandler),
    Reference(String),
}

#[derive(Debug, Clone, PartialEq)]
struct TimerInvocation {
    callback: TimerCallback,
    args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocationParts {
    scheme: String,
    has_authority: bool,
    username: String,
    password: String,
    hostname: String,
    port: String,
    pathname: String,
    opaque_path: String,
    search: String,
    hash: String,
}

impl LocationParts {
    fn protocol(&self) -> String {
        format!("{}:", self.scheme)
    }

    fn host(&self) -> String {
        if self.port.is_empty() {
            self.hostname.clone()
        } else {
            format!("{}:{}", self.hostname, self.port)
        }
    }

    fn origin(&self) -> String {
        if self.has_authority && !self.hostname.is_empty() {
            format!("{}//{}", self.protocol(), self.host())
        } else {
            "null".to_string()
        }
    }

    fn href(&self) -> String {
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

    fn parse(input: &str) -> Option<Self> {
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

fn is_valid_url_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

fn split_hostname_and_port(authority: &str) -> (String, String) {
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

fn split_authority_components(authority: &str) -> (String, String, String, String) {
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

fn split_path_search_hash(tail: &str) -> (String, String, String) {
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

fn split_opaque_search_hash(rest: &str) -> (String, String, String) {
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

fn normalize_pathname(pathname: &str) -> String {
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

fn ensure_search_prefix(value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else if value.starts_with('?') {
        value.to_string()
    } else {
        format!("?{value}")
    }
}

fn ensure_hash_prefix(value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else if value.starts_with('#') {
        value.to_string()
    } else {
        format!("#{value}")
    }
}

#[derive(Debug, Clone)]
struct Listener {
    capture: bool,
    handler: ScriptHandler,
    captured_env: Rc<RefCell<ScriptEnv>>,
    captured_pending_function_decls: Vec<Arc<HashMap<String, (ScriptHandler, bool)>>>,
}

#[derive(Debug, Default, Clone)]
struct ListenerStore {
    map: HashMap<NodeId, HashMap<String, Vec<Listener>>>,
}

impl ListenerStore {
    fn add(&mut self, node_id: NodeId, event: String, listener: Listener) {
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

    fn remove(
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

    fn get(&self, node_id: NodeId, event: &str, capture: bool) -> Vec<Listener> {
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
struct EventState {
    event_type: String,
    target: NodeId,
    current_target: NodeId,
    event_phase: i32,
    time_stamp_ms: i64,
    default_prevented: bool,
    is_trusted: bool,
    bubbles: bool,
    cancelable: bool,
    state: Option<Value>,
    old_state: Option<String>,
    new_state: Option<String>,
    propagation_stopped: bool,
    immediate_propagation_stopped: bool,
}

impl EventState {
    fn new(event_type: &str, target: NodeId, time_stamp_ms: i64) -> Self {
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

    fn new_untrusted(event_type: &str, target: NodeId, time_stamp_ms: i64) -> Self {
        let mut event = Self::new(event_type, target, time_stamp_ms);
        event.is_trusted = false;
        event.bubbles = false;
        event.cancelable = false;
        event
    }
}

#[derive(Debug, Clone)]
struct ParseOutput {
    dom: Dom,
    scripts: Vec<String>,
}

#[derive(Debug, Clone)]
struct ScheduledTask {
    id: i64,
    due_at: i64,
    order: i64,
    interval_ms: Option<i64>,
    callback: TimerCallback,
    callback_args: Vec<Value>,
    env: ScriptEnv,
}

#[derive(Debug, Clone)]
enum ScheduledMicrotask {
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

#[derive(Debug, Clone, PartialEq)]
struct HistoryEntry {
    url: String,
    state: Value,
}

#[derive(Debug)]
struct LocationHistoryState {
    history_object: Rc<RefCell<ObjectValue>>,
    history_entries: Vec<HistoryEntry>,
    history_index: usize,
    history_scroll_restoration: String,
    location_mock_pages: HashMap<String, String>,
    location_navigations: Vec<LocationNavigation>,
    location_reload_count: usize,
}

impl LocationHistoryState {
    fn new(initial_url: &str) -> Self {
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
struct ScriptEnv {
    inner: Arc<HashMap<String, Value>>,
}

impl ScriptEnv {
    fn share(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    fn from_snapshot(env: &HashMap<String, Value>) -> Self {
        Self {
            inner: Arc::new(env.clone()),
        }
    }

    fn to_map(&self) -> HashMap<String, Value> {
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
struct ListenerCaptureFrame {
    shared_env: Option<Rc<RefCell<ScriptEnv>>>,
}

#[derive(Debug, Default)]
struct ScriptRuntimeState {
    env: ScriptEnv,
    pending_function_decls: Vec<Arc<HashMap<String, (ScriptHandler, bool)>>>,
    listener_capture_env_stack: Vec<ListenerCaptureFrame>,
}

#[derive(Debug)]
struct DomRuntimeState {
    window_object: Rc<RefCell<ObjectValue>>,
    document_object: Rc<RefCell<ObjectValue>>,
    location_object: Rc<RefCell<ObjectValue>>,
    node_event_handler_props: HashMap<(NodeId, String), ScriptHandler>,
    node_expando_props: HashMap<(NodeId, String), Value>,
    dialog_return_values: HashMap<NodeId, String>,
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
struct PlatformMockState {
    clipboard_text: String,
    fetch_mocks: HashMap<String, String>,
    fetch_calls: Vec<String>,
    match_media_mocks: HashMap<String, bool>,
    match_media_calls: Vec<String>,
    default_match_media_matches: bool,
    alert_messages: Vec<String>,
    confirm_responses: VecDeque<bool>,
    default_confirm_response: bool,
    prompt_responses: VecDeque<Option<String>>,
    default_prompt_response: Option<String>,
}

#[derive(Debug)]
struct TraceState {
    enabled: bool,
    events: bool,
    timers: bool,
    logs: VecDeque<String>,
    log_limit: usize,
    to_stderr: bool,
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
struct BrowserApiState {
    next_url_object_id: usize,
    url_objects: HashMap<usize, Rc<RefCell<ObjectValue>>>,
    url_constructor_properties: Rc<RefCell<ObjectValue>>,
    local_storage_object: Rc<RefCell<ObjectValue>>,
    next_blob_url_id: usize,
    blob_url_objects: HashMap<String, Rc<RefCell<BlobValue>>>,
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
        }
    }
}

impl BrowserApiState {
    fn allocate_url_object_id(&mut self) -> usize {
        let id = self.next_url_object_id;
        self.next_url_object_id = self.next_url_object_id.saturating_add(1);
        id
    }

    fn allocate_blob_url(&mut self) -> String {
        let object_url = format!("blob:bt-{}", self.next_blob_url_id);
        self.next_blob_url_id = self.next_blob_url_id.saturating_add(1);
        object_url
    }
}

#[derive(Debug)]
struct PromiseRuntimeState {
    next_promise_id: usize,
}

impl Default for PromiseRuntimeState {
    fn default() -> Self {
        Self { next_promise_id: 1 }
    }
}

impl PromiseRuntimeState {
    fn allocate_promise_id(&mut self) -> usize {
        let id = self.next_promise_id;
        self.next_promise_id = self.next_promise_id.saturating_add(1);
        id
    }
}

#[derive(Debug)]
struct SymbolRuntimeState {
    next_symbol_id: usize,
    symbol_registry: HashMap<String, Rc<SymbolValue>>,
    symbols_by_id: HashMap<usize, Rc<SymbolValue>>,
    well_known_symbols: HashMap<String, Rc<SymbolValue>>,
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
    fn allocate_symbol_id(&mut self) -> usize {
        let id = self.next_symbol_id;
        self.next_symbol_id = self.next_symbol_id.saturating_add(1);
        id
    }
}

#[derive(Debug)]
struct SchedulerState {
    task_queue: Vec<ScheduledTask>,
    microtask_queue: VecDeque<ScheduledMicrotask>,
    now_ms: i64,
    timer_step_limit: usize,
    next_timer_id: i64,
    next_task_order: i64,
    task_depth: usize,
    running_timer_id: Option<i64>,
    running_timer_canceled: bool,
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
    fn allocate_timer_id(&mut self) -> i64 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        id
    }

    fn allocate_task_order(&mut self) -> i64 {
        let order = self.next_task_order;
        self.next_task_order += 1;
        order
    }
}

