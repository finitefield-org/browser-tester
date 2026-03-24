use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptErrorKind {
    Parse,
    Runtime,
}

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyboardEventInit {
    pub key: String,
    pub code: Option<String>,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub repeat: bool,
    pub is_composing: bool,
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
    DocumentPlugins,
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
pub struct DateValue {
    pub epoch_ms: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntlNumberFormatValue {
    pub locale: String,
    pub style: String,
    pub currency: Option<String>,
    pub use_grouping: bool,
    pub minimum_integer_digits: usize,
    pub minimum_fraction_digits: Option<usize>,
    pub maximum_fraction_digits: Option<usize>,
    pub minimum_significant_digits: Option<usize>,
    pub maximum_significant_digits: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntlDateTimeFormatValue {
    pub locale: String,
    pub time_zone: Option<String>,
    pub year: Option<String>,
    pub month: Option<String>,
    pub day: Option<String>,
    pub hour: Option<String>,
    pub minute: Option<String>,
    pub second: Option<String>,
    pub hour12: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntlCollatorValue {
    pub locale: String,
    pub numeric: bool,
    pub sensitivity: Option<String>,
    pub usage: Option<String>,
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
pub struct StringListState {
    items: Vec<String>,
}

impl StringListState {
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }

    pub fn length(&self) -> usize {
        self.items.len()
    }

    pub fn item(&self, index: usize) -> Option<&str> {
        self.items.get(index).map(String::as_str)
    }

    pub fn contains(&self, value: &str) -> bool {
        self.items.iter().any(|item| item == value)
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MimeTypeArrayState {
    items: Vec<String>,
}

impl MimeTypeArrayState {
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }

    pub fn length(&self) -> usize {
        self.items.len()
    }

    pub fn item(&self, index: usize) -> Option<&str> {
        self.items.get(index).map(String::as_str)
    }

    pub fn named_item(&self, name: &str) -> Option<&str> {
        self.items
            .iter()
            .find(|item| item.as_str() == name)
            .map(String::as_str)
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AttributeState {
    namespace_uri: Option<String>,
    name: String,
    value: String,
    owner_element: Option<ElementHandle>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttributeHandle(Rc<RefCell<AttributeState>>);

impl AttributeHandle {
    pub fn new(
        namespace_uri: Option<String>,
        name: impl Into<String>,
        value: impl Into<String>,
        owner_element: Option<ElementHandle>,
    ) -> Self {
        Self(Rc::new(RefCell::new(AttributeState {
            namespace_uri,
            name: name.into(),
            value: value.into(),
            owner_element,
        })))
    }

    pub fn namespace_uri(&self) -> Option<String> {
        self.0.borrow().namespace_uri.clone()
    }

    pub fn name(&self) -> String {
        self.0.borrow().name.clone()
    }

    pub fn value(&self) -> String {
        self.0.borrow().value.clone()
    }

    pub fn set_value(&self, value: impl Into<String>) {
        self.0.borrow_mut().value = value.into();
    }

    pub fn owner_element(&self) -> Option<ElementHandle> {
        self.0.borrow().owner_element
    }

    pub fn set_owner_element(&self, owner_element: Option<ElementHandle>) {
        self.0.borrow_mut().owner_element = owner_element;
    }

    pub fn local_name(&self) -> String {
        let name = self.name();
        name.split_once(':')
            .map(|(_, local_name)| local_name.to_string())
            .unwrap_or(name)
    }

    pub fn prefix(&self) -> Option<String> {
        self.name()
            .split_once(':')
            .map(|(prefix, _)| prefix.to_string())
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
pub(crate) enum PropertyKey {
    String(String),
    Symbol(u64),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PropertyValue {
    Data(ScriptValue),
    Accessor {
        getter: Option<ScriptFunction>,
        setter: Option<ScriptFunction>,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct ObjectState {
    properties: Vec<(PropertyKey, PropertyValue)>,
}

#[derive(Clone, Debug)]
pub struct ObjectHandle(Rc<RefCell<ObjectState>>);

impl ObjectHandle {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(ObjectState {
            properties: Vec::new(),
        })))
    }

    pub fn identity(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
}

impl PartialEq for ObjectHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ArrayState {
    items: Vec<ScriptValue>,
    properties: Vec<(PropertyKey, PropertyValue)>,
}

#[derive(Clone, Debug)]
pub struct ArrayHandle(Rc<RefCell<ArrayState>>);

impl ArrayHandle {
    pub fn new(items: Vec<ScriptValue>) -> Self {
        Self(Rc::new(RefCell::new(ArrayState {
            items,
            properties: Vec::new(),
        })))
    }

    pub fn identity(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
}

impl PartialEq for ArrayHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct MapState {
    entries: Vec<(MapKey, ScriptValue)>,
    properties: Vec<(PropertyKey, PropertyValue)>,
}

#[derive(Clone, Debug)]
pub struct MapHandle(Rc<RefCell<MapState>>);

impl MapHandle {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(MapState {
            entries: Vec::new(),
            properties: Vec::new(),
        })))
    }

    pub fn identity(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
}

impl PartialEq for MapHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum MapKey {
    Undefined,
    Null,
    Boolean(bool),
    Number(u64),
    String(String),
    Symbol(u64),
    Object(usize),
    Array(usize),
    Map(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolValue {
    id: u64,
    description: Option<String>,
}

impl SymbolValue {
    pub fn new(description: Option<String>) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            description,
        }
    }

    pub fn from_parts(id: u64, description: Option<String>) -> Self {
        Self { id, description }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegExpValue {
    pattern: String,
    flags: String,
}

impl RegExpValue {
    pub fn new(pattern: impl Into<String>, flags: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            flags: canonicalize_regexp_flags(flags.into()),
        }
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn flags(&self) -> &str {
        &self.flags
    }

    pub fn is_global(&self) -> bool {
        self.flags.contains('g')
    }

    pub fn is_ignore_case(&self) -> bool {
        self.flags.contains('i')
    }

    pub fn is_multiline(&self) -> bool {
        self.flags.contains('m')
    }

    pub fn is_dot_all(&self) -> bool {
        self.flags.contains('s')
    }

    pub fn is_unicode(&self) -> bool {
        self.flags.contains('u')
    }

    pub fn is_sticky(&self) -> bool {
        self.flags.contains('y')
    }
}

fn canonicalize_regexp_flags(flags: String) -> String {
    let mut output = String::new();
    for flag in ['g', 'i', 'm', 's', 'u', 'y'] {
        if flags.contains(flag) {
            output.push(flag);
        }
    }
    output
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
    key: Option<String>,
    code: Option<String>,
    ctrl_key: bool,
    meta_key: bool,
    shift_key: bool,
    alt_key: bool,
    repeat: bool,
    is_composing: bool,
    is_trusted: bool,
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
            key: None,
            code: None,
            ctrl_key: false,
            meta_key: false,
            shift_key: false,
            alt_key: false,
            repeat: false,
            is_composing: false,
            is_trusted: false,
        })))
    }

    pub fn new_keyboard(
        event_type: impl Into<String>,
        target: ListenerTarget,
        bubbles: bool,
        cancelable: bool,
        init: &KeyboardEventInit,
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
            key: Some(init.key.clone()),
            code: init.code.clone(),
            ctrl_key: init.ctrl_key,
            meta_key: init.meta_key,
            shift_key: init.shift_key,
            alt_key: init.alt_key,
            repeat: init.repeat,
            is_composing: init.is_composing,
            is_trusted: false,
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

    pub fn key(&self) -> Option<String> {
        self.0.borrow().key.clone()
    }

    pub fn code(&self) -> Option<String> {
        self.0.borrow().code.clone()
    }

    pub fn ctrl_key(&self) -> bool {
        self.0.borrow().ctrl_key
    }

    pub fn meta_key(&self) -> bool {
        self.0.borrow().meta_key
    }

    pub fn shift_key(&self) -> bool {
        self.0.borrow().shift_key
    }

    pub fn alt_key(&self) -> bool {
        self.0.borrow().alt_key
    }

    pub fn repeat(&self) -> bool {
        self.0.borrow().repeat
    }

    pub fn is_composing(&self) -> bool {
        self.0.borrow().is_composing
    }

    pub fn is_trusted(&self) -> bool {
        self.0.borrow().is_trusted
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
    Object(ObjectHandle),
    Array(ArrayHandle),
    Map(MapHandle),
    Symbol(SymbolValue),
    RegExp(RegExpValue),
    Element(ElementHandle),
    Attribute(AttributeHandle),
    ClassList(ElementHandle),
    Dataset(ElementHandle),
    TemplateContent(ElementHandle),
    NamedNodeMap(ElementHandle),
    HtmlCollection(HtmlCollectionTarget),
    StyleSheetList(StyleSheetListTarget),
    Storage(StorageTarget),
    MediaQueryList(MediaQueryListState),
    StringList(StringListState),
    MimeTypeArray(MimeTypeArrayState),
    Navigator,
    Clipboard,
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
    Date(DateValue),
    IntlNumberFormat(IntlNumberFormatValue),
    IntlDateTimeFormat(IntlDateTimeFormatValue),
    IntlCollator(IntlCollatorValue),
    Document,
    Window,
    ObjectNamespace,
    ArrayNamespace,
    IntlNamespace,
    Event(ScriptEventHandle),
    Function(ScriptFunction),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScriptFunction {
    pub params: Vec<String>,
    pub body_source: String,
    pub captured_bindings: Rc<BTreeMap<String, ScriptValue>>,
}

impl ScriptFunction {
    pub fn new(params: Vec<String>, body_source: impl Into<String>) -> Self {
        Self {
            params,
            body_source: body_source.into(),
            captured_bindings: Rc::new(BTreeMap::new()),
        }
    }

    pub fn with_captured_bindings(
        mut self,
        captured_bindings: BTreeMap<String, ScriptValue>,
    ) -> Self {
        self.captured_bindings = Rc::new(captured_bindings);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptError {
    kind: ScriptErrorKind,
    message: String,
}

impl ScriptError {
    pub fn new(message: impl Into<String>) -> Self {
        Self::runtime(message)
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self {
            kind: ScriptErrorKind::Parse,
            message: message.into(),
        }
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self {
            kind: ScriptErrorKind::Runtime,
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn kind(&self) -> ScriptErrorKind {
        self.kind
    }

    pub fn phase_not_ready(capability: &str) -> Self {
        Self::runtime(format!(
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

    fn document_create_element(&mut self, _tag_name: &str) -> Result<ElementHandle> {
        Err(ScriptError::phase_not_ready("document.createElement"))
    }

    fn document_create_element_ns(
        &mut self,
        _namespace_uri: &str,
        _tag_name: &str,
    ) -> Result<ElementHandle> {
        Err(ScriptError::phase_not_ready("document.createElementNS"))
    }

    fn document_create_text_node(&mut self, _text: &str) -> Result<NodeHandle> {
        Err(ScriptError::phase_not_ready("document.createTextNode"))
    }

    fn document_create_comment(&mut self, _text: &str) -> Result<NodeHandle> {
        Err(ScriptError::phase_not_ready("document.createComment"))
    }

    fn document_normalize(&mut self) -> Result<()> {
        Err(ScriptError::phase_not_ready("Document.normalize"))
    }

    fn node_clone(&mut self, _node: NodeHandle, _deep: bool) -> Result<NodeHandle> {
        Err(ScriptError::phase_not_ready("Node.cloneNode"))
    }

    fn node_normalize(&mut self, _node: NodeHandle) -> Result<()> {
        Err(ScriptError::phase_not_ready("Node.normalize"))
    }

    fn node_replace_with(&mut self, _node: NodeHandle, _children: Vec<NodeHandle>) -> Result<()> {
        Err(ScriptError::phase_not_ready("Node.replaceWith"))
    }

    fn node_before(&mut self, _node: NodeHandle, _children: Vec<NodeHandle>) -> Result<()> {
        Err(ScriptError::phase_not_ready("Node.before"))
    }

    fn node_after(&mut self, _node: NodeHandle, _children: Vec<NodeHandle>) -> Result<()> {
        Err(ScriptError::phase_not_ready("Node.after"))
    }

    fn document_contains(&mut self, _node: NodeHandle) -> Result<bool> {
        Err(ScriptError::phase_not_ready("document.contains"))
    }

    fn node_contains(&mut self, _node: NodeHandle, _other: NodeHandle) -> Result<bool> {
        Err(ScriptError::phase_not_ready("Node.contains"))
    }

    fn node_compare_document_position(
        &mut self,
        _node: NodeHandle,
        _other: NodeHandle,
    ) -> Result<u16> {
        Err(ScriptError::phase_not_ready("Node.compareDocumentPosition"))
    }

    fn node_is_equal_node(&mut self, _node: NodeHandle, _other: NodeHandle) -> Result<bool> {
        Err(ScriptError::phase_not_ready("Node.isEqualNode"))
    }

    fn template_content_is_equal_node(
        &mut self,
        _fragment: ElementHandle,
        _other: ElementHandle,
    ) -> Result<bool> {
        Err(ScriptError::phase_not_ready("template.content.isEqualNode"))
    }

    fn document_has_child_nodes(&mut self) -> Result<bool> {
        Err(ScriptError::phase_not_ready("document.hasChildNodes"))
    }

    fn node_has_child_nodes(&mut self, _node: NodeHandle) -> Result<bool> {
        Err(ScriptError::phase_not_ready("Node.hasChildNodes"))
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

    fn element_click(&mut self, _element: ElementHandle) -> Result<()> {
        Err(ScriptError::phase_not_ready("Element.click"))
    }

    fn element_focus(&mut self, _element: ElementHandle) -> Result<()> {
        Err(ScriptError::phase_not_ready("Element.focus"))
    }

    fn element_blur(&mut self, _element: ElementHandle) -> Result<()> {
        Err(ScriptError::phase_not_ready("Element.blur"))
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

    fn document_location_assign(&mut self, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.location.assign"))
    }

    fn document_location_replace(&mut self, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.location.replace"))
    }

    fn document_location_reload(&mut self) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.location.reload"))
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

    fn document_write(&mut self, _html: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.write"))
    }

    fn document_writeln(&mut self, _html: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.writeln"))
    }

    fn document_open(&mut self) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.open"))
    }

    fn document_close(&mut self) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.close"))
    }

    fn match_media(&mut self, _query: &str) -> Result<MediaQueryListState> {
        Err(ScriptError::phase_not_ready("window.matchMedia"))
    }

    fn match_media_add_listener(&mut self, _query: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("MediaQueryList.addListener"))
    }

    fn match_media_remove_listener(&mut self, _query: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready(
            "MediaQueryList.removeListener",
        ))
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

    fn window_request_animation_frame(&mut self, _callback: ScriptFunction) -> Result<u64> {
        Err(ScriptError::phase_not_ready("window.requestAnimationFrame"))
    }

    fn window_cancel_animation_frame(&mut self, _handle: u64) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.cancelAnimationFrame"))
    }

    fn window_set_timeout(&mut self, _callback: ScriptFunction, _delay_ms: i64) -> Result<u64> {
        Err(ScriptError::phase_not_ready("window.setTimeout"))
    }

    fn window_clear_timeout(&mut self, _handle: u64) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.clearTimeout"))
    }

    fn window_set_interval(&mut self, _callback: ScriptFunction, _delay_ms: i64) -> Result<u64> {
        Err(ScriptError::phase_not_ready("window.setInterval"))
    }

    fn window_clear_interval(&mut self, _handle: u64) -> Result<()> {
        Err(ScriptError::phase_not_ready("window.clearInterval"))
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

    fn window_navigator_oscpu(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("window.navigator.oscpu"))
    }

    fn window_navigator_user_language(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready(
            "window.navigator.userLanguage",
        ))
    }

    fn window_navigator_browser_language(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready(
            "window.navigator.browserLanguage",
        ))
    }

    fn window_navigator_system_language(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready(
            "window.navigator.systemLanguage",
        ))
    }

    fn window_navigator_languages(&mut self) -> Result<Vec<String>> {
        Err(ScriptError::phase_not_ready("window.navigator.languages"))
    }

    fn window_navigator_mime_types(&mut self) -> Result<Vec<String>> {
        Err(ScriptError::phase_not_ready("window.navigator.mimeTypes"))
    }

    fn clipboard_write_text(&mut self, _text: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready(
            "navigator.clipboard.writeText",
        ))
    }

    fn clipboard_read_text(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("navigator.clipboard.readText"))
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

    fn document_design_mode(&mut self) -> Result<String> {
        Err(ScriptError::phase_not_ready("document.designMode"))
    }

    fn document_set_design_mode(&mut self, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("document.designMode"))
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

    fn node_parent(&mut self, _node: NodeHandle) -> Result<Option<NodeHandle>> {
        Err(ScriptError::phase_not_ready("Node.parentNode"))
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

    fn node_namespace_uri(&mut self, _node: NodeHandle) -> Result<Option<String>> {
        Err(ScriptError::phase_not_ready("Node.namespaceURI"))
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

    fn element_is_content_editable(&mut self, _element: ElementHandle) -> Result<bool> {
        Err(ScriptError::phase_not_ready("element.isContentEditable"))
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

    fn element_indeterminate(&mut self, _element: ElementHandle) -> Result<bool> {
        Err(ScriptError::phase_not_ready("element.indeterminate"))
    }

    fn element_set_indeterminate(
        &mut self,
        _element: ElementHandle,
        _indeterminate: bool,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready(
            "element.indeterminate assignment",
        ))
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

    fn element_attribute_names(&mut self, _element: ElementHandle) -> Result<Vec<String>> {
        Err(ScriptError::phase_not_ready("Element.attributes"))
    }

    fn element_toggle_attribute(
        &mut self,
        _element: ElementHandle,
        _name: &str,
        _force: Option<bool>,
    ) -> Result<bool> {
        Err(ScriptError::phase_not_ready("element.toggleAttribute"))
    }

    fn element_append_child(&mut self, _parent: ElementHandle, _child: NodeHandle) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.appendChild"))
    }

    fn element_insert_before(
        &mut self,
        _parent: ElementHandle,
        _child: NodeHandle,
        _reference: Option<NodeHandle>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.insertBefore"))
    }

    fn element_replace_child(
        &mut self,
        _parent: ElementHandle,
        _new_child: NodeHandle,
        _old_child: NodeHandle,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.replaceChild"))
    }

    fn element_replace_children(
        &mut self,
        _parent: ElementHandle,
        _children: Vec<NodeHandle>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.replaceChildren"))
    }

    fn element_append(
        &mut self,
        _element: ElementHandle,
        _children: Vec<NodeHandle>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.append"))
    }

    fn element_prepend(
        &mut self,
        _element: ElementHandle,
        _children: Vec<NodeHandle>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.prepend"))
    }

    fn element_before(
        &mut self,
        _element: ElementHandle,
        _children: Vec<NodeHandle>,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.before"))
    }

    fn element_after(&mut self, _element: ElementHandle, _children: Vec<NodeHandle>) -> Result<()> {
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
        mut initial_bindings: BTreeMap<String, ScriptValue>,
    ) -> Result<()> {
        match evaluator::eval_program_with_bindings(program, host, &mut initial_bindings)? {
            evaluator::EvalControl::Continue => Ok(()),
            evaluator::EvalControl::Return(_) => Err(ScriptError::new("return outside function")),
            evaluator::EvalControl::Break => Err(ScriptError::new("break outside loop")),
            evaluator::EvalControl::ContinueLoop => Err(ScriptError::new("continue outside loop")),
        }
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
