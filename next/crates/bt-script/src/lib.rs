use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;
use std::rc::Rc;

mod evaluator;
mod parser;
mod syntax;

#[derive(Clone, Debug, Default)]
pub struct ScriptParser;

#[derive(Clone, Debug, Default)]
pub struct Evaluator;

#[derive(Clone, Debug, Default)]
pub struct ScriptHeap;

#[derive(Clone, Debug, Default)]
pub struct GlobalEnvironment;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementHandle(u64);

impl ElementHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeHandle(u64);

impl NodeHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListenerTarget {
    Window,
    Document,
    Element(ElementHandle),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HtmlCollectionScope {
    Document,
    Element(ElementHandle),
    Node(NodeHandle),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HtmlCollectionTarget {
    Children(ElementHandle),
    ByTagName {
        scope: HtmlCollectionScope,
        tag_name: String,
    },
    ByTagNameNs {
        scope: HtmlCollectionScope,
        namespace_uri: String,
        local_name: String,
    },
    ByClassName {
        scope: HtmlCollectionScope,
        class_names: String,
    },
    FormElements(ElementHandle),
    SelectOptions(ElementHandle),
    SelectSelectedOptions(ElementHandle),
    DocumentLinks,
    DocumentAnchors,
    DocumentChildren,
    WindowFrames,
    MapAreas(ElementHandle),
    TableTBodies(ElementHandle),
    TableRows(ElementHandle),
    RowCells(ElementHandle),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StyleSheetListTarget {
    Document,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StorageTarget {
    Local,
    Session,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaQueryListState {
    media: String,
    matches: bool,
}

impl MediaQueryListState {
    pub fn new(media: impl Into<String>, matches: bool) -> Self {
        Self {
            media: media.into(),
            matches,
        }
    }

    pub fn media(&self) -> &str {
        &self.media
    }

    pub fn matches(&self) -> bool {
        self.matches
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenOrientationState {
    orientation_type: String,
    angle: i64,
}

impl ScreenOrientationState {
    pub fn new(orientation_type: impl Into<String>, angle: i64) -> Self {
        Self {
            orientation_type: orientation_type.into(),
            angle,
        }
    }

    pub fn orientation_type(&self) -> &str {
        &self.orientation_type
    }

    pub fn angle(&self) -> i64 {
        self.angle
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RadioNodeListTarget {
    FormElements {
        element: ElementHandle,
        name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StyleSheetTarget {
    OwnerNode(ElementHandle),
}

#[derive(Clone, Debug, PartialEq)]
struct CollectionEntryState {
    index: usize,
    value: ScriptValue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollectionEntryHandle(Rc<RefCell<CollectionEntryState>>);

impl CollectionEntryHandle {
    pub fn new(index: usize, value: ScriptValue) -> Self {
        Self(Rc::new(RefCell::new(CollectionEntryState { index, value })))
    }

    pub fn index(&self) -> usize {
        self.0.borrow().index
    }

    pub fn value(&self) -> ScriptValue {
        self.0.borrow().value.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HtmlCollectionNamedItem {
    Element(ElementHandle),
    RadioNodeList(RadioNodeListTarget),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeListTarget {
    Snapshot(Vec<ElementHandle>),
    ByName(String),
    Labels(ElementHandle),
    ChildNodes(HtmlCollectionScope),
}

#[derive(Clone, Debug, PartialEq)]
struct CollectionIteratorState {
    items: Vec<ScriptValue>,
    index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollectionIteratorHandle(Rc<RefCell<CollectionIteratorState>>);

impl CollectionIteratorHandle {
    pub fn new(items: Vec<ScriptValue>) -> Self {
        Self(Rc::new(RefCell::new(CollectionIteratorState {
            items,
            index: 0,
        })))
    }

    pub fn next_result(&self) -> IteratorResult {
        let mut state = self.0.borrow_mut();
        if state.index >= state.items.len() {
            return IteratorResult::new(None, true);
        }

        let value = state.items[state.index].clone();
        state.index += 1;
        IteratorResult::new(Some(value), false)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IteratorResult {
    value: Option<ScriptValue>,
    done: bool,
}

impl IteratorResult {
    pub fn new(value: Option<ScriptValue>, done: bool) -> Self {
        Self { value, done }
    }

    pub fn value(&self) -> Option<ScriptValue> {
        self.value.clone()
    }

    pub fn done(&self) -> bool {
        self.done
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EventPhase {
    None = 0,
    Capturing = 1,
    AtTarget = 2,
    Bubbling = 3,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScriptEventState {
    event_type: String,
    target: ListenerTarget,
    current_target: Option<ListenerTarget>,
    bubbles: bool,
    cancelable: bool,
    default_prevented: bool,
    propagation_stopped: bool,
    immediate_propagation_stopped: bool,
    phase: EventPhase,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptEventHandle(Rc<RefCell<ScriptEventState>>);

impl ScriptEventHandle {
    pub fn new(
        event_type: impl Into<String>,
        target: ListenerTarget,
        bubbles: bool,
        cancelable: bool,
    ) -> Self {
        Self(Rc::new(RefCell::new(ScriptEventState {
            event_type: event_type.into(),
            target,
            current_target: None,
            bubbles,
            cancelable,
            default_prevented: false,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            phase: EventPhase::None,
        })))
    }

    pub fn event_type(&self) -> String {
        self.0.borrow().event_type.clone()
    }

    pub fn target(&self) -> ListenerTarget {
        self.0.borrow().target
    }

    pub fn current_target(&self) -> Option<ListenerTarget> {
        self.0.borrow().current_target
    }

    pub fn set_current_target(&self, target: Option<ListenerTarget>) {
        self.0.borrow_mut().current_target = target;
    }

    pub fn bubbles(&self) -> bool {
        self.0.borrow().bubbles
    }

    pub fn cancelable(&self) -> bool {
        self.0.borrow().cancelable
    }

    pub fn default_prevented(&self) -> bool {
        self.0.borrow().default_prevented
    }

    pub fn propagation_stopped(&self) -> bool {
        self.0.borrow().propagation_stopped
    }

    pub fn immediate_propagation_stopped(&self) -> bool {
        self.0.borrow().immediate_propagation_stopped
    }

    pub fn event_phase(&self) -> EventPhase {
        self.0.borrow().phase
    }

    pub fn set_phase(&self, phase: EventPhase) {
        self.0.borrow_mut().phase = phase;
    }

    pub fn prevent_default(&self) {
        let mut state = self.0.borrow_mut();
        if state.cancelable {
            state.default_prevented = true;
        }
    }

    pub fn stop_propagation(&self) {
        self.0.borrow_mut().propagation_stopped = true;
    }

    pub fn stop_immediate_propagation(&self) {
        let mut state = self.0.borrow_mut();
        state.propagation_stopped = true;
        state.immediate_propagation_stopped = true;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScriptValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Element(ElementHandle),
    ClassList(ElementHandle),
    Dataset(ElementHandle),
    TemplateContent(ElementHandle),
    HtmlCollection(HtmlCollectionTarget),
    StyleSheetList(StyleSheetListTarget),
    Storage(StorageTarget),
    MediaQueryList(MediaQueryListState),
    Navigator,
    History,
    Screen,
    ScreenOrientation(ScreenOrientationState),
    StyleSheet(StyleSheetTarget),
    Node(NodeHandle),
    NodeList(NodeListTarget),
    RadioNodeList(RadioNodeListTarget),
    CollectionEntry(CollectionEntryHandle),
    CollectionIterator(CollectionIteratorHandle),
    IteratorResult(Box<IteratorResult>),
    Document,
    Window,
    Event(ScriptEventHandle),
    Function(ScriptFunction),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptFunction {
    pub params: Vec<String>,
    pub body_source: String,
}

impl ScriptFunction {
    pub fn new(params: Vec<String>, body_source: impl Into<String>) -> Self {
        Self {
            params,
            body_source: body_source.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptError {
    message: String,
}

impl ScriptError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn phase_not_ready(capability: &str) -> Self {
        Self::new(format!(
            "{capability} is planned for a later phase of browser-tester-next"
        ))
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for ScriptError {}

pub type Result<T> = std::result::Result<T, ScriptError>;

pub trait HostBindings {
    fn on_eval(&mut self, _code: &str, _source_name: &str) -> Result<()> {
        Ok(())
    }

    fn on_microtask_checkpoint(&mut self) -> Result<()> {
        Ok(())
    }

    fn document_get_element_by_id(&mut self, _id: &str) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.getElementById"))
    }

    fn document_document_element(&mut self) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.documentElement"))
    }

    fn document_head(&mut self) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.head"))
    }

    fn document_body(&mut self) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.body"))
    }

    fn document_scrolling_element(&mut self) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.scrollingElement"))
    }

    fn document_active_element(&mut self) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.activeElement"))
    }

    fn document_has_focus(&mut self) -> Result<bool> {
        Err(ScriptError::phase_not_ready("document.hasFocus"))
    }

    fn document_visibility_state(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.visibilityState"))
    }

    fn document_hidden(&mut self) -> Result<bool> {
        Err(ScriptError::phase_not_ready("document.hidden"))
    }

    fn document_title(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.title"))
    }

    fn document_set_title(&mut self, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.title"))
    }

    fn document_location(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.location"))
    }

    fn document_set_location(&mut self, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.location"))
    }

    fn document_url(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.URL"))
    }

    fn document_document_uri(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.documentURI"))
    }

    fn document_base_uri(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.baseURI"))
    }

    fn document_origin(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.origin"))
    }

    fn document_domain(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.domain"))
    }

    fn document_referrer(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.referrer"))
    }

    fn document_cookie(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.cookie"))
    }

    fn document_set_cookie(&mut self, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.cookie"))
    }

    fn match_media(&mut self, _query: &str) -> Result<MediaQueryListState> {
        Err(ScriptError::phase_not_ready("window.matchMedia"))
    }

    fn window_open(
        &mut self,
        _url: Option<&str>,
        _target: Option<&str>,
        _features: Option<&str>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.open"))
    }

    fn window_close(&mut self) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.close"))
    }

    fn window_print(&mut self) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.print"))
    }

    fn window_alert(&mut self, _message: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.alert"))
    }

    fn window_confirm(&mut self, _message: &str) -> Result<bool> {
        Err(ScriptError::phase_not_ready("window.confirm"))
    }

    fn window_prompt(
        &mut self,
        _message: &str,
        _default_text: Option<&str>,
    ) -> Result<Option<String>> {
        Err(ScriptError::phase_not_ready("window.prompt"))
    }

    fn html_collection_window_frames_items(&mut self) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("window.frames"))
    }

    fn html_collection_window_frames_named_item(
        &mut self,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("window.frames"))
    }

    fn window_navigator_user_agent(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.userAgent"))
    }

    fn window_navigator_app_code_name(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.appCodeName"))
    }

    fn window_navigator_app_name(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.appName"))
    }

    fn window_navigator_app_version(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.appVersion"))
    }

    fn window_navigator_product(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.product"))
    }

    fn window_navigator_product_sub(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.productSub"))
    }

    fn window_navigator_platform(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.platform"))
    }

    fn window_navigator_language(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.language"))
    }

    fn window_navigator_cookie_enabled(&mut self) -> Result<bool> {
        Err(ScriptError::phase_not_ready(
            "window.navigator.cookieEnabled",
        ))
    }

    fn window_navigator_on_line(&mut self) -> Result<bool> {
        Err(ScriptError::phase_not_ready("window.navigator.onLine"))
    }

    fn window_navigator_webdriver(&mut self) -> Result<bool> {
        Err(ScriptError::phase_not_ready("window.navigator.webdriver"))
    }

    fn window_navigator_vendor(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.vendor"))
    }

    fn window_navigator_vendor_sub(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.vendorSub"))
    }

    fn window_navigator_pdf_viewer_enabled(&mut self) -> Result<bool> {
        Err(ScriptError::phase_not_ready(
            "window.navigator.pdfViewerEnabled",
        ))
    }

    fn window_navigator_do_not_track(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.doNotTrack"))
    }

    fn window_navigator_java_enabled(&mut self) -> Result<bool> {
        Err(ScriptError::phase_not_ready(
            "window.navigator.javaEnabled()",
        ))
    }

    fn window_navigator_hardware_concurrency(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready(
            "window.navigator.hardwareConcurrency",
        ))
    }

    fn window_navigator_max_touch_points(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready(
            "window.navigator.maxTouchPoints",
        ))
    }

    fn window_history_length(&mut self) -> Result<usize> {
        Err(ScriptError::phase_not_ready("window.history"))
    }

    fn window_history_scroll_restoration(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready(
            "window.history.scrollRestoration",
        ))
    }

    fn set_window_history_scroll_restoration(&mut self, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready(
            "window.history.scrollRestoration",
        ))
    }

    fn window_history_state(&mut self) -> Result<Option<String>> {
        Err(ScriptError::phase_not_ready("window.history.state"))
    }

    fn window_history_push_state(
        &mut self,
        _state: Option<&str>,
        _url: Option<&str>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.history.pushState()"))
    }

    fn window_history_replace_state(
        &mut self,
        _state: Option<&str>,
        _url: Option<&str>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready(
            "window.history.replaceState()",
        ))
    }

    fn window_history_back(&mut self) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.history.back()"))
    }

    fn window_history_forward(&mut self) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.history.forward()"))
    }

    fn window_history_go(&mut self, _delta: i64) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.history.go()"))
    }

    fn window_scroll_x(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.scrollX"))
    }

    fn window_scroll_y(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.scrollY"))
    }

    fn window_page_x_offset(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.pageXOffset"))
    }

    fn window_page_y_offset(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.pageYOffset"))
    }

    fn window_device_pixel_ratio(&mut self) -> Result<f64> {
        Err(ScriptError::phase_not_ready("window.devicePixelRatio"))
    }

    fn window_inner_width(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.innerWidth"))
    }

    fn window_inner_height(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.innerHeight"))
    }

    fn window_outer_width(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.outerWidth"))
    }

    fn window_outer_height(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.outerHeight"))
    }

    fn window_screen_x(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screenX"))
    }

    fn window_screen_y(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screenY"))
    }

    fn window_screen_left(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screenLeft"))
    }

    fn window_screen_top(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screenTop"))
    }

    fn window_screen_width(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screen.width"))
    }

    fn window_screen_height(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screen.height"))
    }

    fn window_screen_avail_width(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screen.availWidth"))
    }

    fn window_screen_avail_height(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screen.availHeight"))
    }

    fn window_screen_avail_left(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screen.availLeft"))
    }

    fn window_screen_avail_top(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screen.availTop"))
    }

    fn window_screen_color_depth(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screen.colorDepth"))
    }

    fn window_screen_pixel_depth(&mut self) -> Result<i64> {
        Err(ScriptError::phase_not_ready("window.screen.pixelDepth"))
    }

    fn window_screen_orientation(&mut self) -> Result<ScreenOrientationState> {
        Err(ScriptError::phase_not_ready("window.screen.orientation"))
    }

    fn window_scroll_to(&mut self, _x: i64, _y: i64) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.scrollTo"))
    }

    fn window_scroll_by(&mut self, _x: i64, _y: i64) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.scrollBy"))
    }

    fn window_name(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.name"))
    }

    fn set_window_name(&mut self, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.name"))
    }

    fn storage_length(&mut self, target: StorageTarget) -> Result<usize> {
        Err(ScriptError::phase_not_ready(match target {
            StorageTarget::Local => "window.localStorage",
            StorageTarget::Session => "window.sessionStorage",
        }))
    }

    fn storage_get_item(&mut self, target: StorageTarget, _key: &str) -> Result<Option<String>> {
        Err(ScriptError::phase_not_ready(match target {
            StorageTarget::Local => "window.localStorage",
            StorageTarget::Session => "window.sessionStorage",
        }))
    }

    fn storage_set_item(&mut self, target: StorageTarget, _key: &str, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready(match target {
            StorageTarget::Local => "window.localStorage",
            StorageTarget::Session => "window.sessionStorage",
        }))
    }

    fn storage_remove_item(&mut self, target: StorageTarget, _key: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready(match target {
            StorageTarget::Local => "window.localStorage",
            StorageTarget::Session => "window.sessionStorage",
        }))
    }

    fn storage_clear(&mut self, target: StorageTarget) -> Result<()> {
        Err(ScriptError::phase_not_ready(match target {
            StorageTarget::Local => "window.localStorage",
            StorageTarget::Session => "window.sessionStorage",
        }))
    }

    fn storage_key(&mut self, target: StorageTarget, _index: usize) -> Result<Option<String>> {
        Err(ScriptError::phase_not_ready(match target {
            StorageTarget::Local => "window.localStorage",
            StorageTarget::Session => "window.sessionStorage",
        }))
    }

    fn document_current_script(&mut self) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.currentScript"))
    }

    fn document_ready_state(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.readyState"))
    }

    fn document_compat_mode(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.compatMode"))
    }

    fn document_character_set(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.characterSet"))
    }

    fn document_content_type(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.contentType"))
    }

    fn document_dir(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.dir"))
    }

    fn document_set_dir(&mut self, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.dir"))
    }

    fn document_query_selector(&mut self, _selector: &str) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.querySelector"))
    }

    fn document_query_selector_all(&mut self, _selector: &str) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.querySelectorAll"))
    }

    fn document_get_elements_by_name(&mut self, _name: &str) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.getElementsByName"))
    }

    fn document_style_sheets_items(&mut self) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.styleSheets"))
    }

    fn document_style_sheets_named_item(&mut self, _name: &str) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready(
            "document.styleSheets.namedItem()",
        ))
    }

    fn node_child_nodes_items(&mut self, _scope: HtmlCollectionScope) -> Result<Vec<NodeHandle>> {
        Err(ScriptError::phase_not_ready("Node.childNodes"))
    }

    fn node_text_content(&mut self, _node: NodeHandle) -> Result<String> {
        Err(ScriptError::phase_not_ready("Node.textContent"))
    }

    fn node_type(&mut self, _node: NodeHandle) -> Result<u8> {
        Err(ScriptError::phase_not_ready("Node.nodeType"))
    }

    fn node_name(&mut self, _node: NodeHandle) -> Result<String> {
        Err(ScriptError::phase_not_ready("Node.nodeName"))
    }

    fn element_children(&mut self, _element: ElementHandle) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("element.children"))
    }

    fn element_tag_name(&mut self, _element: ElementHandle) -> Result<String> {
        Err(ScriptError::phase_not_ready("element.tagName"))
    }

    fn element_base_uri(&mut self, _element: ElementHandle) -> Result<String> {
        Err(ScriptError::phase_not_ready("element.baseURI"))
    }

    fn element_origin(&mut self, _element: ElementHandle) -> Result<String> {
        Err(ScriptError::phase_not_ready("element.origin"))
    }

    fn element_labels(&mut self, _element: ElementHandle) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("element.labels"))
    }

    fn html_collection_named_item(
        &mut self,
        _element: ElementHandle,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("HTMLCollection.namedItem"))
    }

    fn html_collection_tag_name_items(
        &mut self,
        _collection: HtmlCollectionTarget,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready(
            "HTMLCollection.getElementsByTagName",
        ))
    }

    fn html_collection_tag_name_named_item(
        &mut self,
        _collection: HtmlCollectionTarget,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready(
            "HTMLCollection.getElementsByTagName",
        ))
    }

    fn html_collection_tag_name_ns_items(
        &mut self,
        _collection: HtmlCollectionTarget,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready(
            "HTMLCollection.getElementsByTagNameNS",
        ))
    }

    fn html_collection_tag_name_ns_named_item(
        &mut self,
        _collection: HtmlCollectionTarget,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready(
            "HTMLCollection.getElementsByTagNameNS",
        ))
    }

    fn html_collection_class_name_items(
        &mut self,
        _collection: HtmlCollectionTarget,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready(
            "HTMLCollection.getElementsByClassName",
        ))
    }

    fn html_collection_class_name_named_item(
        &mut self,
        _collection: HtmlCollectionTarget,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready(
            "HTMLCollection.getElementsByClassName",
        ))
    }

    fn html_collection_form_elements_items(
        &mut self,
        _element: ElementHandle,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("form.elements"))
    }

    fn html_collection_form_elements_named_item(
        &mut self,
        _element: ElementHandle,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("form.elements"))
    }

    fn html_collection_form_elements_named_items(
        &mut self,
        _element: ElementHandle,
        _name: &str,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("form.elements"))
    }

    fn radio_node_list_set_value(
        &mut self,
        _target: &RadioNodeListTarget,
        _value: &str,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready(
            "RadioNodeList.value assignment",
        ))
    }

    fn html_collection_select_options_items(
        &mut self,
        _element: ElementHandle,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("select.options"))
    }

    fn html_collection_select_options_named_item(
        &mut self,
        _element: ElementHandle,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("select.options"))
    }

    fn html_collection_select_options_add(
        &mut self,
        _element: ElementHandle,
        _option: ElementHandle,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("select.options.add"))
    }

    fn html_collection_select_options_remove(
        &mut self,
        _element: ElementHandle,
        _index: usize,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("select.options.remove"))
    }

    fn html_collection_select_selected_options_items(
        &mut self,
        _element: ElementHandle,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("select.selectedOptions"))
    }

    fn html_collection_select_selected_options_named_item(
        &mut self,
        _element: ElementHandle,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("select.selectedOptions"))
    }

    fn html_collection_document_links_items(&mut self) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.links"))
    }

    fn html_collection_document_links_named_item(
        &mut self,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.links"))
    }

    fn html_collection_document_anchors_items(&mut self) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.anchors"))
    }

    fn html_collection_document_anchors_named_item(
        &mut self,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.anchors"))
    }

    fn html_collection_document_children_items(&mut self) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.children"))
    }

    fn html_collection_document_children_named_item(
        &mut self,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.children"))
    }

    fn html_collection_map_areas_items(
        &mut self,
        _element: ElementHandle,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("map.areas"))
    }

    fn html_collection_map_areas_named_item(
        &mut self,
        _element: ElementHandle,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("map.areas"))
    }

    fn html_collection_table_rows_items(
        &mut self,
        _element: ElementHandle,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("table.rows"))
    }

    fn html_collection_table_rows_named_item(
        &mut self,
        _element: ElementHandle,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("table.rows"))
    }

    fn html_collection_table_bodies_items(
        &mut self,
        _element: ElementHandle,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("table.tBodies"))
    }

    fn html_collection_table_bodies_named_item(
        &mut self,
        _element: ElementHandle,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("table.tBodies"))
    }

    fn html_collection_row_cells_items(
        &mut self,
        _element: ElementHandle,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("tr.cells"))
    }

    fn html_collection_row_cells_named_item(
        &mut self,
        _element: ElementHandle,
        _name: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("tr.cells"))
    }

    fn element_text_content(&mut self, _element: ElementHandle) -> Result<String> {
        Err(ScriptError::phase_not_ready("element.textContent"))
    }

    fn element_set_text_content(&mut self, _element: ElementHandle, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready(
            "element.textContent assignment",
        ))
    }

    fn element_inner_html(&mut self, _element: ElementHandle) -> Result<String> {
        Err(ScriptError::phase_not_ready("element.innerHTML"))
    }

    fn element_set_inner_html(&mut self, _element: ElementHandle, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.innerHTML assignment"))
    }

    fn element_outer_html(&mut self, _element: ElementHandle) -> Result<String> {
        Err(ScriptError::phase_not_ready("element.outerHTML"))
    }

    fn element_set_outer_html(&mut self, _element: ElementHandle, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.outerHTML assignment"))
    }

    fn element_insert_adjacent_html(
        &mut self,
        _element: ElementHandle,
        _position: &str,
        _value: &str,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.insertAdjacentHTML"))
    }

    fn element_value(&mut self, _element: ElementHandle) -> Result<String> {
        Err(ScriptError::phase_not_ready("element.value"))
    }

    fn element_set_value(&mut self, _element: ElementHandle, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.value assignment"))
    }

    fn element_checked(&mut self, _element: ElementHandle) -> Result<bool> {
        Err(ScriptError::phase_not_ready("element.checked"))
    }

    fn element_set_checked(&mut self, _element: ElementHandle, _checked: bool) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.checked assignment"))
    }

    fn element_get_attribute(
        &mut self,
        _element: ElementHandle,
        _name: &str,
    ) -> Result<Option<String>> {
        Err(ScriptError::phase_not_ready("element.getAttribute"))
    }

    fn element_set_attribute(
        &mut self,
        _element: ElementHandle,
        _name: &str,
        _value: &str,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.setAttribute"))
    }

    fn element_remove_attribute(&mut self, _element: ElementHandle, _name: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.removeAttribute"))
    }

    fn element_has_attribute(&mut self, _element: ElementHandle, _name: &str) -> Result<bool> {
        Err(ScriptError::phase_not_ready("element.hasAttribute"))
    }

    fn element_toggle_attribute(
        &mut self,
        _element: ElementHandle,
        _name: &str,
        _force: Option<bool>,
    ) -> Result<bool> {
        Err(ScriptError::phase_not_ready("element.toggleAttribute"))
    }

    fn element_append_child(
        &mut self,
        _parent: ElementHandle,
        _child: ElementHandle,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.appendChild"))
    }

    fn element_insert_before(
        &mut self,
        _parent: ElementHandle,
        _child: ElementHandle,
        _reference: Option<ElementHandle>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.insertBefore"))
    }

    fn element_replace_child(
        &mut self,
        _parent: ElementHandle,
        _new_child: ElementHandle,
        _old_child: ElementHandle,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.replaceChild"))
    }

    fn element_replace_children(
        &mut self,
        _parent: ElementHandle,
        _children: Vec<ElementHandle>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.replaceChildren"))
    }

    fn element_append(
        &mut self,
        _element: ElementHandle,
        _children: Vec<ElementHandle>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.append"))
    }

    fn element_prepend(
        &mut self,
        _element: ElementHandle,
        _children: Vec<ElementHandle>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.prepend"))
    }

    fn element_before(
        &mut self,
        _element: ElementHandle,
        _children: Vec<ElementHandle>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.before"))
    }

    fn element_after(
        &mut self,
        _element: ElementHandle,
        _children: Vec<ElementHandle>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.after"))
    }

    fn element_remove(&mut self, _element: ElementHandle) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.remove"))
    }

    fn element_query_selector(
        &mut self,
        _element: ElementHandle,
        _selector: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("element.querySelector"))
    }

    fn element_query_selector_all(
        &mut self,
        _element: ElementHandle,
        _selector: &str,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("element.querySelectorAll"))
    }

    fn element_matches(&mut self, _element: ElementHandle, _selector: &str) -> Result<bool> {
        Err(ScriptError::phase_not_ready("element.matches"))
    }

    fn element_closest(
        &mut self,
        _element: ElementHandle,
        _selector: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("element.closest"))
    }

    fn register_event_listener(
        &mut self,
        _target: ListenerTarget,
        _event_type: &str,
        _handler: ScriptFunction,
    ) -> Result<()> {
        self.register_event_listener_with_capture(_target, _event_type, false, _handler)
    }

    fn register_event_listener_with_capture(
        &mut self,
        _target: ListenerTarget,
        _event_type: &str,
        _capture: bool,
        _handler: ScriptFunction,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("addEventListener"))
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScriptRuntime {
    parser: ScriptParser,
    evaluator: Evaluator,
    heap: ScriptHeap,
    globals: GlobalEnvironment,
    queued_microtasks: usize,
}

impl ScriptRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parser(&self) -> &ScriptParser {
        &self.parser
    }

    pub fn evaluator(&self) -> &Evaluator {
        &self.evaluator
    }

    pub fn heap(&self) -> &ScriptHeap {
        &self.heap
    }

    pub fn globals(&self) -> &GlobalEnvironment {
        &self.globals
    }

    pub fn eval_program<H: HostBindings>(
        &mut self,
        code: &str,
        source_name: &str,
        host: &mut H,
    ) -> Result<()> {
        host.on_eval(code, source_name)?;
        let program = self.parser.parse_program(code)?;
        self.evaluator.eval_program(&program, host)
    }

    pub fn eval_program_with_bindings<H: HostBindings>(
        &mut self,
        code: &str,
        source_name: &str,
        host: &mut H,
        initial_bindings: BTreeMap<String, ScriptValue>,
    ) -> Result<()> {
        host.on_eval(code, source_name)?;
        let program = self.parser.parse_program(code)?;
        self.evaluator
            .eval_program_with_bindings(&program, host, initial_bindings)
    }

    pub fn queue_microtask(&mut self) {
        self.queued_microtasks += 1;
    }

    pub fn queued_microtasks(&self) -> usize {
        self.queued_microtasks
    }

    pub fn run_microtasks<H: HostBindings>(&mut self, host: &mut H) -> Result<()> {
        while self.queued_microtasks > 0 {
            self.queued_microtasks -= 1;
            host.on_microtask_checkpoint()?;
        }
        Ok(())
    }
}

impl ScriptParser {
    pub(crate) fn parse_program(&self, code: &str) -> Result<syntax::Program> {
        parser::parse_program(code)
    }
}

impl Evaluator {
    pub(crate) fn eval_program<H: HostBindings>(
        &self,
        program: &syntax::Program,
        host: &mut H,
    ) -> Result<()> {
        evaluator::eval_program(program, host)
    }

    pub(crate) fn eval_program_with_bindings<H: HostBindings>(
        &self,
        program: &syntax::Program,
        host: &mut H,
        initial_bindings: BTreeMap<String, ScriptValue>,
    ) -> Result<()> {
        evaluator::eval_program_with_bindings(program, host, initial_bindings)
    }
}

#[cfg(test)]
mod tests {
    use super::{HostBindings, ScriptRuntime};

    #[derive(Default)]
    struct RecordingHost {
        evals: Vec<(String, String)>,
        microtask_ticks: usize,
    }

    impl HostBindings for RecordingHost {
        fn on_eval(&mut self, code: &str, source_name: &str) -> super::Result<()> {
            self.evals.push((source_name.to_owned(), code.to_owned()));
            Ok(())
        }

        fn on_microtask_checkpoint(&mut self) -> super::Result<()> {
            self.microtask_ticks += 1;
            Ok(())
        }

        fn document_scrolling_element(&mut self) -> super::Result<Option<super::ElementHandle>> {
            Ok(None)
        }
    }

    #[test]
    fn eval_program_delegates_to_host_bindings() {
        let mut runtime = ScriptRuntime::new();
        let mut host = RecordingHost::default();
        runtime
            .eval_program("const value = 'x';", "inline-script", &mut host)
            .expect("host callback should succeed");
        assert_eq!(
            host.evals,
            vec![(
                "inline-script".to_string(),
                "const value = 'x';".to_string(),
            )]
        );
    }

    #[test]
    fn queued_microtasks_are_drained_in_order() {
        let mut runtime = ScriptRuntime::new();
        let mut host = RecordingHost::default();
        runtime.queue_microtask();
        runtime.queue_microtask();
        runtime
            .run_microtasks(&mut host)
            .expect("microtasks should drain");
        assert_eq!(runtime.queued_microtasks(), 0);
        assert_eq!(host.microtask_ticks, 2);
    }
}
