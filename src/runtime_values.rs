use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ArrayBufferValue {
    pub(crate) bytes: Vec<u8>,
    pub(crate) max_byte_length: Option<usize>,
    pub(crate) detached: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlobValue {
    pub(crate) bytes: Vec<u8>,
    pub(crate) mime_type: String,
}

impl ArrayBufferValue {
    pub(crate) fn byte_length(&self) -> usize {
        if self.detached { 0 } else { self.bytes.len() }
    }

    pub(crate) fn max_byte_length(&self) -> usize {
        if self.detached {
            0
        } else {
            self.max_byte_length.unwrap_or(self.bytes.len())
        }
    }

    pub(crate) fn resizable(&self) -> bool {
        !self.detached && self.max_byte_length.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypedArrayKind {
    Int8,
    Uint8,
    Uint8Clamped,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float16,
    Float32,
    Float64,
    BigInt64,
    BigUint64,
}

impl TypedArrayKind {
    pub(crate) fn bytes_per_element(&self) -> usize {
        match self {
            Self::Int8 | Self::Uint8 | Self::Uint8Clamped => 1,
            Self::Int16 | Self::Uint16 | Self::Float16 => 2,
            Self::Int32 | Self::Uint32 | Self::Float32 => 4,
            Self::Float64 | Self::BigInt64 | Self::BigUint64 => 8,
        }
    }

    pub(crate) fn is_bigint(&self) -> bool {
        matches!(self, Self::BigInt64 | Self::BigUint64)
    }

    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Int8 => "Int8Array",
            Self::Uint8 => "Uint8Array",
            Self::Uint8Clamped => "Uint8ClampedArray",
            Self::Int16 => "Int16Array",
            Self::Uint16 => "Uint16Array",
            Self::Int32 => "Int32Array",
            Self::Uint32 => "Uint32Array",
            Self::Float16 => "Float16Array",
            Self::Float32 => "Float32Array",
            Self::Float64 => "Float64Array",
            Self::BigInt64 => "BigInt64Array",
            Self::BigUint64 => "BigUint64Array",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypedArrayConstructorKind {
    Concrete(TypedArrayKind),
    Abstract,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TypedArrayValue {
    pub(crate) kind: TypedArrayKind,
    pub(crate) buffer: Rc<RefCell<ArrayBufferValue>>,
    pub(crate) byte_offset: usize,
    pub(crate) fixed_length: Option<usize>,
}

impl TypedArrayValue {
    pub(crate) fn observed_length(&self) -> usize {
        let buffer_len = self.buffer.borrow().byte_length();
        let bytes_per = self.kind.bytes_per_element();
        if self.byte_offset >= buffer_len {
            return 0;
        }

        let available_bytes = buffer_len - self.byte_offset;
        if let Some(fixed_length) = self.fixed_length {
            let fixed_bytes = fixed_length.saturating_mul(bytes_per);
            if available_bytes < fixed_bytes {
                0
            } else {
                fixed_length
            }
        } else {
            available_bytes / bytes_per
        }
    }

    pub(crate) fn observed_byte_length(&self) -> usize {
        self.observed_length()
            .saturating_mul(self.kind.bytes_per_element())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MapValue {
    pub(crate) entries: Vec<(Value, Value)>,
    pub(crate) properties: ObjectValue,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WeakMapValue {
    pub(crate) entries: Vec<(Value, Value)>,
    pub(crate) properties: ObjectValue,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetValue {
    pub(crate) values: Vec<Value>,
    pub(crate) properties: ObjectValue,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WeakSetValue {
    pub(crate) values: Vec<Value>,
    pub(crate) properties: ObjectValue,
}

#[derive(Debug, Clone)]
pub(crate) struct PromiseValue {
    pub(crate) id: usize,
    pub(crate) state: PromiseState,
    pub(crate) reactions: Vec<PromiseReaction>,
}

impl PartialEq for PromiseValue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PromiseState {
    Pending,
    Fulfilled(Value),
    Rejected(Value),
}

#[derive(Debug, Clone)]
pub(crate) struct PromiseReaction {
    pub(crate) kind: PromiseReactionKind,
}

#[derive(Debug, Clone)]
pub(crate) enum PromiseReactionKind {
    Then {
        on_fulfilled: Option<Value>,
        on_rejected: Option<Value>,
        result: Rc<RefCell<PromiseValue>>,
    },
    Finally {
        callback: Option<Value>,
        result: Rc<RefCell<PromiseValue>>,
    },
    FinallyContinuation {
        original: PromiseSettledValue,
        result: Rc<RefCell<PromiseValue>>,
    },
    ResolveTo {
        target: Rc<RefCell<PromiseValue>>,
    },
    All {
        state: Rc<RefCell<PromiseAllState>>,
        index: usize,
    },
    AllSettled {
        state: Rc<RefCell<PromiseAllSettledState>>,
        index: usize,
    },
    Any {
        state: Rc<RefCell<PromiseAnyState>>,
        index: usize,
    },
    Race {
        state: Rc<RefCell<PromiseRaceState>>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum PromiseSettledValue {
    Fulfilled(Value),
    Rejected(Value),
}

#[derive(Debug, Clone)]
pub(crate) struct PromiseAllState {
    pub(crate) result: Rc<RefCell<PromiseValue>>,
    pub(crate) remaining: usize,
    pub(crate) values: Vec<Option<Value>>,
    pub(crate) settled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PromiseAllSettledState {
    pub(crate) result: Rc<RefCell<PromiseValue>>,
    pub(crate) remaining: usize,
    pub(crate) values: Vec<Option<Value>>,
}

#[derive(Debug, Clone)]
pub(crate) struct PromiseAnyState {
    pub(crate) result: Rc<RefCell<PromiseValue>>,
    pub(crate) remaining: usize,
    pub(crate) reasons: Vec<Option<Value>>,
    pub(crate) settled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PromiseRaceState {
    pub(crate) result: Rc<RefCell<PromiseValue>>,
    pub(crate) settled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PromiseCapabilityFunction {
    pub(crate) promise: Rc<RefCell<PromiseValue>>,
    pub(crate) reject: bool,
    pub(crate) already_called: Rc<RefCell<bool>>,
}

impl PartialEq for PromiseCapabilityFunction {
    fn eq(&self, other: &Self) -> bool {
        self.reject == other.reject
            && self.promise.borrow().id == other.promise.borrow().id
            && Rc::ptr_eq(&self.already_called, &other.already_called)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SymbolValue {
    pub(crate) id: usize,
    pub(crate) description: Option<String>,
    pub(crate) registry_key: Option<String>,
}

impl PartialEq for SymbolValue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    String(String),
    StringConstructor,
    Bool(bool),
    Number(i64),
    Float(f64),
    BigInt(JsBigInt),
    Array(Rc<RefCell<ArrayValue>>),
    Object(Rc<RefCell<ObjectValue>>),
    Promise(Rc<RefCell<PromiseValue>>),
    Map(Rc<RefCell<MapValue>>),
    WeakMap(Rc<RefCell<WeakMapValue>>),
    Set(Rc<RefCell<SetValue>>),
    WeakSet(Rc<RefCell<WeakSetValue>>),
    Blob(Rc<RefCell<BlobValue>>),
    ArrayBuffer(Rc<RefCell<ArrayBufferValue>>),
    TypedArray(Rc<RefCell<TypedArrayValue>>),
    TypedArrayConstructor(TypedArrayConstructorKind),
    BlobConstructor,
    UrlConstructor,
    ArrayBufferConstructor,
    PromiseConstructor,
    MapConstructor,
    WeakMapConstructor,
    SetConstructor,
    WeakSetConstructor,
    SymbolConstructor,
    RegExpConstructor,
    PromiseCapability(Rc<PromiseCapabilityFunction>),
    Symbol(Rc<SymbolValue>),
    RegExp(Rc<RefCell<RegexValue>>),
    Date(Rc<RefCell<i64>>),
    Null,
    Undefined,
    Node(NodeId),
    NodeList(Vec<NodeId>),
    FormData(Vec<(String, String)>),
    Function(Rc<FunctionValue>),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ObjectValue {
    pub(crate) entries: Vec<(String, Value)>,
    pub(crate) index_by_key: HashMap<String, usize>,
}

impl ObjectValue {
    pub(crate) fn new(entries: Vec<(String, Value)>) -> Self {
        let mut value = Self::default();
        for (key, entry_value) in entries {
            value.set_entry(key, entry_value);
        }
        value
    }

    pub(crate) fn set_entry(&mut self, key: String, value: Value) {
        if let Some(index) = self.index_by_key.get(&key).copied() {
            if let Some((_, existing)) = self.entries.get_mut(index) {
                *existing = value;
                return;
            }
        }
        let index = self.entries.len();
        self.entries.push((key.clone(), value));
        self.index_by_key.insert(key, index);
    }

    pub(crate) fn get_entry(&self, key: &str) -> Option<Value> {
        self.index_by_key
            .get(key)
            .and_then(|index| self.entries.get(*index))
            .map(|(_, value)| value.clone())
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.index_by_key.clear();
    }
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct ArrayValue {
    pub(crate) elements: Vec<Value>,
    pub(crate) properties: ObjectValue,
}

impl ArrayValue {
    pub(crate) fn new(elements: Vec<Value>) -> Self {
        Self {
            elements,
            properties: ObjectValue::default(),
        }
    }
}

impl std::ops::Deref for ArrayValue {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl std::ops::DerefMut for ArrayValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elements
    }
}

impl From<Vec<(String, Value)>> for ObjectValue {
    fn from(entries: Vec<(String, Value)>) -> Self {
        Self::new(entries)
    }
}

impl std::ops::Deref for ObjectValue {
    type Target = [(String, Value)];

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl PartialEq for ObjectValue {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RegexValue {
    pub(crate) source: String,
    pub(crate) flags: String,
    pub(crate) global: bool,
    pub(crate) ignore_case: bool,
    pub(crate) multiline: bool,
    pub(crate) dot_all: bool,
    pub(crate) sticky: bool,
    pub(crate) has_indices: bool,
    pub(crate) unicode: bool,
    pub(crate) unicode_sets: bool,
    pub(crate) compiled: Regex,
    pub(crate) last_index: usize,
    pub(crate) properties: ObjectValue,
}

impl PartialEq for RegexValue {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.flags == other.flags
            && self.global == other.global
            && self.ignore_case == other.ignore_case
            && self.multiline == other.multiline
            && self.dot_all == other.dot_all
            && self.sticky == other.sticky
            && self.has_indices == other.has_indices
            && self.unicode == other.unicode
            && self.unicode_sets == other.unicode_sets
            && self.last_index == other.last_index
            && self.properties == other.properties
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionValue {
    pub(crate) handler: ScriptHandler,
    pub(crate) captured_env: Rc<RefCell<ScriptEnv>>,
    pub(crate) captured_pending_function_decls:
        Vec<Arc<HashMap<String, (ScriptHandler, bool, bool)>>>,
    pub(crate) captured_global_names: HashSet<String>,
    pub(crate) local_bindings: HashSet<String>,
    pub(crate) prototype_object: Rc<RefCell<ObjectValue>>,
    pub(crate) global_scope: bool,
    pub(crate) is_async: bool,
    pub(crate) is_generator: bool,
    pub(crate) is_class_constructor: bool,
    pub(crate) class_super_constructor: Option<Value>,
    pub(crate) class_super_prototype: Option<Value>,
}

impl PartialEq for FunctionValue {
    fn eq(&self, other: &Self) -> bool {
        self.handler == other.handler
            && self.global_scope == other.global_scope
            && self.is_async == other.is_async
            && self.is_generator == other.is_generator
            && self.is_class_constructor == other.is_class_constructor
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RegexFlags {
    pub(crate) global: bool,
    pub(crate) ignore_case: bool,
    pub(crate) multiline: bool,
    pub(crate) dot_all: bool,
    pub(crate) sticky: bool,
    pub(crate) has_indices: bool,
    pub(crate) unicode: bool,
    pub(crate) unicode_sets: bool,
}

impl Value {
    pub(crate) fn truthy(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::String(v) => !v.is_empty(),
            Self::StringConstructor => true,
            Self::Number(v) => *v != 0,
            Self::Float(v) => *v != 0.0,
            Self::BigInt(v) => !v.is_zero(),
            Self::Array(_) => true,
            Self::Object(_) => true,
            Self::Promise(_) => true,
            Self::Map(_) => true,
            Self::WeakMap(_) => true,
            Self::Set(_) => true,
            Self::WeakSet(_) => true,
            Self::Blob(_) => true,
            Self::ArrayBuffer(_) => true,
            Self::TypedArray(_) => true,
            Self::TypedArrayConstructor(_) => true,
            Self::BlobConstructor => true,
            Self::UrlConstructor => true,
            Self::ArrayBufferConstructor => true,
            Self::PromiseConstructor => true,
            Self::MapConstructor => true,
            Self::WeakMapConstructor => true,
            Self::SetConstructor => true,
            Self::WeakSetConstructor => true,
            Self::SymbolConstructor => true,
            Self::RegExpConstructor => true,
            Self::PromiseCapability(_) => true,
            Self::Symbol(_) => true,
            Self::RegExp(_) => true,
            Self::Date(_) => true,
            Self::Null => false,
            Self::Undefined => false,
            Self::Node(_) => true,
            Self::NodeList(nodes) => !nodes.is_empty(),
            Self::FormData(_) => true,
            Self::Function(_) => true,
        }
    }

    pub(crate) fn as_string(&self) -> String {
        match self {
            Self::String(v) => v.clone(),
            Self::StringConstructor => "String".to_string(),
            Self::Bool(v) => {
                if *v {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            Self::Number(v) => v.to_string(),
            Self::Float(v) => format_float(*v),
            Self::BigInt(v) => v.to_string(),
            Self::Array(values) => {
                let values = values.borrow();
                let mut out = String::new();
                for (idx, value) in values.iter().enumerate() {
                    if idx > 0 {
                        out.push(',');
                    }
                    if matches!(value, Value::Null | Value::Undefined) {
                        continue;
                    }
                    out.push_str(&value.as_string());
                }
                out
            }
            Self::Object(entries) => {
                let entries = entries.borrow();
                if let Some(Value::String(value)) =
                    entries.get_entry(INTERNAL_STRING_WRAPPER_VALUE_KEY)
                {
                    return value;
                }
                let is_url = matches!(
                    entries.get_entry(INTERNAL_URL_OBJECT_KEY),
                    Some(Value::Bool(true))
                );
                if is_url {
                    if let Some(Value::String(href)) = entries.get_entry("href") {
                        return href;
                    }
                }
                let is_url_search_params = matches!(
                    entries.get_entry(INTERNAL_URL_SEARCH_PARAMS_OBJECT_KEY),
                    Some(Value::Bool(true))
                );
                if is_url_search_params {
                    let mut pairs = Vec::new();
                    if let Some(Value::Array(list)) =
                        entries.get_entry(INTERNAL_URL_SEARCH_PARAMS_ENTRIES_KEY)
                    {
                        let list = list.borrow();
                        for item in list.iter() {
                            let Value::Array(pair) = item else {
                                continue;
                            };
                            let pair = pair.borrow();
                            if pair.is_empty() {
                                continue;
                            }
                            let name = pair[0].as_string();
                            let value = pair.get(1).map(Value::as_string).unwrap_or_default();
                            pairs.push((name, value));
                        }
                    }
                    serialize_url_search_params_pairs(&pairs)
                } else {
                    let is_readable_stream = matches!(
                        entries.get_entry(INTERNAL_READABLE_STREAM_OBJECT_KEY),
                        Some(Value::Bool(true))
                    );
                    if is_readable_stream {
                        "[object ReadableStream]".into()
                    } else {
                        "[object Object]".into()
                    }
                }
            }
            Self::Promise(_) => "[object Promise]".into(),
            Self::Map(_) => "[object Map]".into(),
            Self::WeakMap(_) => "[object WeakMap]".into(),
            Self::Set(_) => "[object Set]".into(),
            Self::WeakSet(_) => "[object WeakSet]".into(),
            Self::Blob(_) => "[object Blob]".into(),
            Self::ArrayBuffer(_) => "[object ArrayBuffer]".into(),
            Self::TypedArray(value) => {
                let value = value.borrow();
                format!("[object {}]", value.kind.name())
            }
            Self::TypedArrayConstructor(kind) => match kind {
                TypedArrayConstructorKind::Concrete(kind) => kind.name().to_string(),
                TypedArrayConstructorKind::Abstract => "TypedArray".to_string(),
            },
            Self::BlobConstructor => "Blob".to_string(),
            Self::UrlConstructor => "URL".to_string(),
            Self::ArrayBufferConstructor => "ArrayBuffer".to_string(),
            Self::PromiseConstructor => "Promise".to_string(),
            Self::MapConstructor => "Map".to_string(),
            Self::WeakMapConstructor => "WeakMap".to_string(),
            Self::SetConstructor => "Set".to_string(),
            Self::WeakSetConstructor => "WeakSet".to_string(),
            Self::SymbolConstructor => "Symbol".to_string(),
            Self::RegExpConstructor => "RegExp".to_string(),
            Self::PromiseCapability(_) => "[object Function]".into(),
            Self::Symbol(value) => {
                if let Some(description) = &value.description {
                    format!("Symbol({description})")
                } else {
                    "Symbol()".to_string()
                }
            }
            Self::RegExp(value) => {
                let value = value.borrow();
                format!("/{}/{}", value.source, value.flags)
            }
            Self::Date(_) => "[object Date]".into(),
            Self::Null => "null".into(),
            Self::Undefined => "undefined".into(),
            Self::Node(node) => format!("node-{}", node.0),
            Self::NodeList(_) => "[object NodeList]".into(),
            Self::FormData(_) => "[object FormData]".into(),
            Self::Function(_) => "[object Function]".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomProp {
    Attributes,
    AssignedSlot,
    Value,
    Files,
    FilesLength,
    ValueAsNumber,
    ValueAsDate,
    ValueLength,
    ValidationMessage,
    Validity,
    ValidityValueMissing,
    ValidityTypeMismatch,
    ValidityPatternMismatch,
    ValidityTooLong,
    ValidityTooShort,
    ValidityRangeUnderflow,
    ValidityRangeOverflow,
    ValidityStepMismatch,
    ValidityBadInput,
    ValidityValid,
    ValidityCustomError,
    SelectionStart,
    SelectionEnd,
    SelectionDirection,
    Checked,
    Indeterminate,
    Open,
    ReturnValue,
    ClosedBy,
    Readonly,
    Required,
    Disabled,
    TextContent,
    InnerText,
    InnerHtml,
    OuterHtml,
    ClassName,
    ClassList,
    ClassListLength,
    Part,
    PartLength,
    Id,
    TagName,
    LocalName,
    NamespaceUri,
    Prefix,
    NextElementSibling,
    PreviousElementSibling,
    Slot,
    Role,
    ElementTiming,
    HtmlFor,
    Name,
    Lang,
    Dir,
    AccessKey,
    AutoCapitalize,
    AutoCorrect,
    ContentEditable,
    Draggable,
    EnterKeyHint,
    Inert,
    InputMode,
    Nonce,
    Popover,
    Spellcheck,
    TabIndex,
    Translate,
    Cite,
    DateTime,
    BrClear,
    CaptionAlign,
    ColSpan,
    CanvasWidth,
    CanvasHeight,
    NodeEventHandler(String),
    BodyDeprecatedAttr(String),
    ClientWidth,
    ClientHeight,
    ClientLeft,
    ClientTop,
    CurrentCssZoom,
    OffsetWidth,
    OffsetHeight,
    OffsetLeft,
    OffsetTop,
    ScrollWidth,
    ScrollHeight,
    ScrollLeft,
    ScrollTop,
    ScrollLeftMax,
    ScrollTopMax,
    ShadowRoot,
    Dataset(String),
    Style(String),
    AriaString(String),
    AriaElementRefSingle(String),
    AriaElementRefList(String),
    ActiveElement,
    CharacterSet,
    CompatMode,
    ContentType,
    ReadyState,
    Referrer,
    Title,
    Url,
    DocumentUri,
    BaseUri,
    Location,
    LocationHref,
    LocationProtocol,
    LocationHost,
    LocationHostname,
    LocationPort,
    LocationPathname,
    LocationSearch,
    LocationHash,
    LocationOrigin,
    LocationAncestorOrigins,
    History,
    HistoryLength,
    HistoryState,
    HistoryScrollRestoration,
    DefaultView,
    Hidden,
    VisibilityState,
    Forms,
    Images,
    Links,
    Scripts,
    Children,
    ChildElementCount,
    FirstElementChild,
    LastElementChild,
    CurrentScript,
    FormsLength,
    ImagesLength,
    LinksLength,
    ScriptsLength,
    ChildrenLength,
    AudioSrc,
    AudioAutoplay,
    AudioControls,
    AudioControlsList,
    AudioCrossOrigin,
    AudioDisableRemotePlayback,
    AudioLoop,
    AudioMuted,
    AudioPreload,
    VideoDisablePictureInPicture,
    VideoPlaysInline,
    VideoPoster,
    AnchorAttributionSrc,
    AnchorDownload,
    AnchorHash,
    AnchorHost,
    AnchorHostname,
    AnchorHref,
    AnchorHreflang,
    AnchorInterestForElement,
    AnchorOrigin,
    AnchorPassword,
    AnchorPathname,
    AnchorPing,
    AnchorPort,
    AnchorProtocol,
    AnchorReferrerPolicy,
    AnchorRel,
    AnchorRelList,
    AnchorRelListLength,
    AnchorSearch,
    AnchorTarget,
    AnchorText,
    AnchorType,
    AnchorUsername,
    AnchorCharset,
    AnchorCoords,
    AnchorRev,
    AnchorShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomIndex {
    Static(usize),
    Dynamic(String),
}

impl DomIndex {
    pub(crate) fn describe(&self) -> String {
        match self {
            Self::Static(index) => index.to_string(),
            Self::Dynamic(expr) => expr.clone(),
        }
    }

    pub(crate) fn static_index(&self) -> Option<usize> {
        match self {
            Self::Static(index) => Some(*index),
            Self::Dynamic(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomQuery {
    DocumentRoot,
    DocumentBody,
    DocumentHead,
    DocumentElement,
    ById(String),
    BySelector(String),
    BySelectorAll {
        selector: String,
    },
    BySelectorAllIndex {
        selector: String,
        index: DomIndex,
    },
    QuerySelector {
        target: Box<DomQuery>,
        selector: String,
    },
    QuerySelectorAll {
        target: Box<DomQuery>,
        selector: String,
    },
    Index {
        target: Box<DomQuery>,
        index: DomIndex,
    },
    QuerySelectorAllIndex {
        target: Box<DomQuery>,
        selector: String,
        index: DomIndex,
    },
    FormElementsIndex {
        form: Box<DomQuery>,
        index: DomIndex,
    },
    Var(String),
    VarPath {
        base: String,
        path: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FormDataSource {
    NewForm(DomQuery),
    Var(String),
}

impl DomQuery {
    pub(crate) fn describe_call(&self) -> String {
        match self {
            Self::DocumentRoot => "document".into(),
            Self::DocumentBody => "document.body".into(),
            Self::DocumentHead => "document.head".into(),
            Self::DocumentElement => "document.documentElement".into(),
            Self::ById(id) => format!("document.getElementById('{id}')"),
            Self::BySelector(selector) => format!("document.querySelector('{selector}')"),
            Self::BySelectorAll { selector } => format!("document.querySelectorAll('{selector}')"),
            Self::BySelectorAllIndex { selector, index } => {
                format!(
                    "document.querySelectorAll('{selector}')[{}]",
                    index.describe()
                )
            }
            Self::QuerySelector { target, selector } => {
                format!("{}.querySelector('{selector}')", target.describe_call())
            }
            Self::QuerySelectorAll { target, selector } => {
                format!("{}.querySelectorAll('{selector}')", target.describe_call())
            }
            Self::Index { target, index } => {
                format!("{}[{}]", target.describe_call(), index.describe())
            }
            Self::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                format!(
                    "{}.querySelectorAll('{selector}')[{}]",
                    target.describe_call(),
                    index.describe()
                )
            }
            Self::FormElementsIndex { form, index } => {
                format!("{}.elements[{}]", form.describe_call(), index.describe())
            }
            Self::Var(name) => name.clone(),
            Self::VarPath { base, path } => format!("{base}.{}", path.join(".")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClassListMethod {
    Add,
    Remove,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinaryOp {
    Or,
    And,
    Nullish,
    Eq,
    Ne,
    StrictEq,
    StrictNe,
    BitOr,
    BitXor,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    Pow,
    Lt,
    Gt,
    Le,
    Ge,
    In,
    InstanceOf,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VarAssignOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    BitOr,
    BitXor,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    LogicalAnd,
    LogicalOr,
    Nullish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EventExprProp {
    Type,
    Target,
    CurrentTarget,
    TargetName,
    CurrentTargetName,
    DefaultPrevented,
    IsTrusted,
    Bubbles,
    Cancelable,
    TargetId,
    CurrentTargetId,
    EventPhase,
    TimeStamp,
    State,
    OldState,
    NewState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MatchMediaProp {
    Matches,
    Media,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntlFormatterKind {
    Collator,
    DateTimeFormat,
    DisplayNames,
    DurationFormat,
    ListFormat,
    NumberFormat,
    PluralRules,
    RelativeTimeFormat,
    Segmenter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntlStaticMethod {
    CollatorSupportedLocalesOf,
    DateTimeFormatSupportedLocalesOf,
    DisplayNamesSupportedLocalesOf,
    DurationFormatSupportedLocalesOf,
    ListFormatSupportedLocalesOf,
    PluralRulesSupportedLocalesOf,
    RelativeTimeFormatSupportedLocalesOf,
    SegmenterSupportedLocalesOf,
    GetCanonicalLocales,
    SupportedValuesOf,
}

impl IntlFormatterKind {
    pub(crate) fn storage_name(self) -> &'static str {
        match self {
            Self::Collator => "Collator",
            Self::DateTimeFormat => "DateTimeFormat",
            Self::DisplayNames => "DisplayNames",
            Self::DurationFormat => "DurationFormat",
            Self::ListFormat => "ListFormat",
            Self::NumberFormat => "NumberFormat",
            Self::PluralRules => "PluralRules",
            Self::RelativeTimeFormat => "RelativeTimeFormat",
            Self::Segmenter => "Segmenter",
        }
    }

    pub(crate) fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "Collator" => Some(Self::Collator),
            "DateTimeFormat" => Some(Self::DateTimeFormat),
            "DisplayNames" => Some(Self::DisplayNames),
            "DurationFormat" => Some(Self::DurationFormat),
            "ListFormat" => Some(Self::ListFormat),
            "NumberFormat" => Some(Self::NumberFormat),
            "PluralRules" => Some(Self::PluralRules),
            "RelativeTimeFormat" => Some(Self::RelativeTimeFormat),
            "Segmenter" => Some(Self::Segmenter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IntlDateTimeOptions {
    pub(crate) calendar: String,
    pub(crate) numbering_system: String,
    pub(crate) time_zone: String,
    pub(crate) date_style: Option<String>,
    pub(crate) time_style: Option<String>,
    pub(crate) weekday: Option<String>,
    pub(crate) year: Option<String>,
    pub(crate) month: Option<String>,
    pub(crate) day: Option<String>,
    pub(crate) hour: Option<String>,
    pub(crate) minute: Option<String>,
    pub(crate) second: Option<String>,
    pub(crate) fractional_second_digits: Option<u8>,
    pub(crate) time_zone_name: Option<String>,
    pub(crate) hour12: Option<bool>,
    pub(crate) day_period: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IntlDateTimeComponents {
    pub(crate) year: i64,
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) hour: u32,
    pub(crate) minute: u32,
    pub(crate) second: u32,
    pub(crate) millisecond: u32,
    pub(crate) weekday: u32,
    pub(crate) offset_minutes: i64,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlPart {
    pub(crate) part_type: String,
    pub(crate) value: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlRelativeTimePart {
    pub(crate) part_type: String,
    pub(crate) value: String,
    pub(crate) unit: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlDisplayNamesOptions {
    pub(crate) style: String,
    pub(crate) display_type: String,
    pub(crate) fallback: String,
    pub(crate) language_display: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlDurationOptions {
    pub(crate) style: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlListOptions {
    pub(crate) style: String,
    pub(crate) list_type: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlPluralRulesOptions {
    pub(crate) rule_type: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlRelativeTimeOptions {
    pub(crate) style: String,
    pub(crate) numeric: String,
    pub(crate) locale_matcher: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlSegmenterOptions {
    pub(crate) granularity: String,
    pub(crate) locale_matcher: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlLocaleOptions {
    pub(crate) language: Option<String>,
    pub(crate) script: Option<String>,
    pub(crate) region: Option<String>,
    pub(crate) calendar: Option<String>,
    pub(crate) case_first: Option<String>,
    pub(crate) collation: Option<String>,
    pub(crate) hour_cycle: Option<String>,
    pub(crate) numbering_system: Option<String>,
    pub(crate) numeric: Option<bool>,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlLocaleData {
    pub(crate) language: String,
    pub(crate) script: Option<String>,
    pub(crate) region: Option<String>,
    pub(crate) variants: Vec<String>,
    pub(crate) calendar: Option<String>,
    pub(crate) case_first: Option<String>,
    pub(crate) collation: Option<String>,
    pub(crate) hour_cycle: Option<String>,
    pub(crate) numbering_system: Option<String>,
    pub(crate) numeric: Option<bool>,
}
