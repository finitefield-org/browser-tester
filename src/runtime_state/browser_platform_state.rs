use super::*;

#[derive(Debug)]
pub(crate) struct DomRuntimeState {
    pub(crate) window_object: Rc<RefCell<ObjectValue>>,
    pub(crate) document_object: Rc<RefCell<ObjectValue>>,
    pub(crate) location_object: Rc<RefCell<ObjectValue>>,
    pub(crate) selection_object: Rc<RefCell<ObjectValue>>,
    pub(crate) document_ready_state: String,
    pub(crate) document_visibility_state: String,
    pub(crate) document_scroll_x: i64,
    pub(crate) document_scroll_y: i64,
    pub(crate) node_event_handler_props: HashMap<(NodeId, String), ScriptHandler>,
    pub(crate) node_expando_props: HashMap<(NodeId, String), Value>,
    pub(crate) live_dom_string_maps: HashMap<NodeId, Rc<RefCell<ObjectValue>>>,
    pub(crate) live_class_lists: HashMap<NodeId, Rc<RefCell<ObjectValue>>>,
    pub(crate) live_child_nodes_lists: HashMap<NodeId, Rc<RefCell<NodeListValue>>>,
    pub(crate) live_children_lists: HashMap<NodeId, Rc<RefCell<NodeListValue>>>,
    pub(crate) live_form_elements_lists: HashMap<NodeId, Rc<RefCell<NodeListValue>>>,
    pub(crate) live_form_named_group_lists: HashMap<(NodeId, String), Rc<RefCell<NodeListValue>>>,
    pub(crate) live_select_options_lists: HashMap<NodeId, Rc<RefCell<NodeListValue>>>,
    pub(crate) live_selected_options_lists: HashMap<NodeId, Rc<RefCell<NodeListValue>>>,
    pub(crate) live_datalist_options_lists: HashMap<NodeId, Rc<RefCell<NodeListValue>>>,
    pub(crate) live_media_text_tracks_lists: HashMap<NodeId, Rc<RefCell<NodeListValue>>>,
    pub(crate) live_text_track_objects: HashMap<NodeId, Rc<RefCell<ObjectValue>>>,
    pub(crate) live_media_time_ranges_objects: HashMap<(NodeId, String), Rc<RefCell<ObjectValue>>>,
    pub(crate) live_named_node_maps: HashMap<NodeId, Rc<RefCell<ObjectValue>>>,
    pub(crate) live_document_forms_list: Option<Rc<RefCell<NodeListValue>>>,
    pub(crate) live_document_images_list: Option<Rc<RefCell<NodeListValue>>>,
    pub(crate) live_document_links_list: Option<Rc<RefCell<NodeListValue>>>,
    pub(crate) live_document_scripts_list: Option<Rc<RefCell<NodeListValue>>>,
    pub(crate) node_animations: Vec<NodeAnimationRecord>,
    pub(crate) pointer_capture_targets: HashMap<i64, NodeId>,
    pub(crate) shadow_roots: HashMap<NodeId, ShadowRootRecord>,
    pub(crate) dialog_return_values: HashMap<NodeId, String>,
    pub(crate) click_in_progress: HashSet<NodeId>,
    pub(crate) focused_form_control_values: HashMap<NodeId, String>,
    pub(crate) pending_form_control_change: HashSet<NodeId>,
}

impl Default for DomRuntimeState {
    fn default() -> Self {
        Self {
            window_object: Rc::new(RefCell::new(ObjectValue::default())),
            document_object: Rc::new(RefCell::new(ObjectValue::default())),
            location_object: Rc::new(RefCell::new(ObjectValue::default())),
            selection_object: Rc::new(RefCell::new(ObjectValue::default())),
            document_ready_state: "complete".to_string(),
            document_visibility_state: "visible".to_string(),
            document_scroll_x: 0,
            document_scroll_y: 0,
            node_event_handler_props: HashMap::new(),
            node_expando_props: HashMap::new(),
            live_dom_string_maps: HashMap::new(),
            live_class_lists: HashMap::new(),
            live_child_nodes_lists: HashMap::new(),
            live_children_lists: HashMap::new(),
            live_form_elements_lists: HashMap::new(),
            live_form_named_group_lists: HashMap::new(),
            live_select_options_lists: HashMap::new(),
            live_selected_options_lists: HashMap::new(),
            live_datalist_options_lists: HashMap::new(),
            live_media_text_tracks_lists: HashMap::new(),
            live_text_track_objects: HashMap::new(),
            live_media_time_ranges_objects: HashMap::new(),
            live_named_node_maps: HashMap::new(),
            live_document_forms_list: None,
            live_document_images_list: None,
            live_document_links_list: None,
            live_document_scripts_list: None,
            node_animations: Vec::new(),
            pointer_capture_targets: HashMap::new(),
            shadow_roots: HashMap::new(),
            dialog_return_values: HashMap::new(),
            click_in_progress: HashSet::new(),
            focused_form_control_values: HashMap::new(),
            pending_form_control_change: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NodeAnimationRecord {
    pub(crate) target: NodeId,
    pub(crate) animation: Rc<RefCell<ObjectValue>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShadowRootMode {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShadowRootRecord {
    pub(crate) root: NodeId,
    pub(crate) mode: ShadowRootMode,
    pub(crate) serializable: bool,
}

#[derive(Debug, Default)]
pub(crate) struct PlatformMockState {
    pub(crate) clipboard_text: String,
    pub(crate) clipboard_read_error: Option<String>,
    pub(crate) clipboard_write_error: Option<String>,
    pub(crate) fetch_mocks: HashMap<String, FetchMockResponse>,
    pub(crate) fetch_calls: Vec<String>,
    pub(crate) match_media_mocks: HashMap<String, bool>,
    pub(crate) match_media_calls: Vec<String>,
    pub(crate) default_match_media_matches: bool,
    pub(crate) alert_messages: Vec<String>,
    pub(crate) print_call_count: usize,
    pub(crate) confirm_responses: VecDeque<bool>,
    pub(crate) default_confirm_response: bool,
    pub(crate) prompt_responses: VecDeque<Option<String>>,
    pub(crate) default_prompt_response: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FetchMockResponse {
    pub(crate) status: i64,
    pub(crate) status_text: String,
    pub(crate) body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CookieRecord {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) domain: Option<String>,
    pub(crate) path: String,
    pub(crate) expires_ms: Option<i64>,
    pub(crate) secure: bool,
    pub(crate) same_site: Option<String>,
    pub(crate) partitioned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CacheEntryRecord {
    pub(crate) request_url: String,
    pub(crate) response_url: String,
    pub(crate) response_status: i64,
    pub(crate) response_status_text: String,
    pub(crate) response_body: String,
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
    pub(crate) cookie_store_object: Rc<RefCell<ObjectValue>>,
    pub(crate) cache_storage_object: Rc<RefCell<ObjectValue>>,
    pub(crate) caches_by_name: HashMap<String, Rc<RefCell<ObjectValue>>>,
    pub(crate) cache_names_in_order: Vec<String>,
    pub(crate) cache_entries_by_name: HashMap<String, Vec<CacheEntryRecord>>,
    pub(crate) window_closed: bool,
    pub(crate) window_screen_x: i64,
    pub(crate) window_screen_y: i64,
    pub(crate) cookies: Vec<CookieRecord>,
    pub(crate) cookie_store_change_listeners: Vec<Value>,
    pub(crate) next_blob_url_id: usize,
    pub(crate) blob_url_objects: HashMap<String, Rc<RefCell<BlobValue>>>,
    pub(crate) downloads: Vec<DownloadArtifact>,
    pub(crate) clipboard_writes: Vec<ClipboardWriteArtifact>,
}

impl Default for BrowserApiState {
    fn default() -> Self {
        Self {
            next_url_object_id: 1,
            url_objects: HashMap::new(),
            url_constructor_properties: Rc::new(RefCell::new(ObjectValue::default())),
            local_storage_object: Rc::new(RefCell::new(ObjectValue::default())),
            cookie_store_object: Rc::new(RefCell::new(ObjectValue::default())),
            cache_storage_object: Rc::new(RefCell::new(ObjectValue::default())),
            caches_by_name: HashMap::new(),
            cache_names_in_order: Vec::new(),
            cache_entries_by_name: HashMap::new(),
            window_closed: false,
            window_screen_x: 0,
            window_screen_y: 0,
            cookies: Vec::new(),
            cookie_store_change_listeners: Vec::new(),
            next_blob_url_id: 1,
            blob_url_objects: HashMap::new(),
            downloads: Vec::new(),
            clipboard_writes: Vec::new(),
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
