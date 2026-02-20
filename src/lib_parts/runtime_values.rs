#[derive(Debug, Clone, PartialEq)]
struct ArrayBufferValue {
    bytes: Vec<u8>,
    max_byte_length: Option<usize>,
    detached: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlobValue {
    bytes: Vec<u8>,
    mime_type: String,
}

impl ArrayBufferValue {
    fn byte_length(&self) -> usize {
        if self.detached { 0 } else { self.bytes.len() }
    }

    fn max_byte_length(&self) -> usize {
        if self.detached {
            0
        } else {
            self.max_byte_length.unwrap_or(self.bytes.len())
        }
    }

    fn resizable(&self) -> bool {
        !self.detached && self.max_byte_length.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypedArrayKind {
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
    fn bytes_per_element(&self) -> usize {
        match self {
            Self::Int8 | Self::Uint8 | Self::Uint8Clamped => 1,
            Self::Int16 | Self::Uint16 | Self::Float16 => 2,
            Self::Int32 | Self::Uint32 | Self::Float32 => 4,
            Self::Float64 | Self::BigInt64 | Self::BigUint64 => 8,
        }
    }

    fn is_bigint(&self) -> bool {
        matches!(self, Self::BigInt64 | Self::BigUint64)
    }

    fn name(&self) -> &'static str {
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
enum TypedArrayConstructorKind {
    Concrete(TypedArrayKind),
    Abstract,
}

#[derive(Debug, Clone, PartialEq)]
struct TypedArrayValue {
    kind: TypedArrayKind,
    buffer: Rc<RefCell<ArrayBufferValue>>,
    byte_offset: usize,
    fixed_length: Option<usize>,
}

impl TypedArrayValue {
    fn observed_length(&self) -> usize {
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

    fn observed_byte_length(&self) -> usize {
        self.observed_length()
            .saturating_mul(self.kind.bytes_per_element())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MapValue {
    entries: Vec<(Value, Value)>,
    properties: ObjectValue,
}

#[derive(Debug, Clone, PartialEq)]
struct SetValue {
    values: Vec<Value>,
    properties: ObjectValue,
}

#[derive(Debug, Clone)]
struct PromiseValue {
    id: usize,
    state: PromiseState,
    reactions: Vec<PromiseReaction>,
}

impl PartialEq for PromiseValue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone)]
enum PromiseState {
    Pending,
    Fulfilled(Value),
    Rejected(Value),
}

#[derive(Debug, Clone)]
struct PromiseReaction {
    kind: PromiseReactionKind,
}

#[derive(Debug, Clone)]
enum PromiseReactionKind {
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
enum PromiseSettledValue {
    Fulfilled(Value),
    Rejected(Value),
}

#[derive(Debug, Clone)]
struct PromiseAllState {
    result: Rc<RefCell<PromiseValue>>,
    remaining: usize,
    values: Vec<Option<Value>>,
    settled: bool,
}

#[derive(Debug, Clone)]
struct PromiseAllSettledState {
    result: Rc<RefCell<PromiseValue>>,
    remaining: usize,
    values: Vec<Option<Value>>,
}

#[derive(Debug, Clone)]
struct PromiseAnyState {
    result: Rc<RefCell<PromiseValue>>,
    remaining: usize,
    reasons: Vec<Option<Value>>,
    settled: bool,
}

#[derive(Debug, Clone)]
struct PromiseRaceState {
    result: Rc<RefCell<PromiseValue>>,
    settled: bool,
}

#[derive(Debug, Clone)]
struct PromiseCapabilityFunction {
    promise: Rc<RefCell<PromiseValue>>,
    reject: bool,
    already_called: Rc<RefCell<bool>>,
}

impl PartialEq for PromiseCapabilityFunction {
    fn eq(&self, other: &Self) -> bool {
        self.reject == other.reject
            && self.promise.borrow().id == other.promise.borrow().id
            && Rc::ptr_eq(&self.already_called, &other.already_called)
    }
}

#[derive(Debug, Clone)]
struct SymbolValue {
    id: usize,
    description: Option<String>,
    registry_key: Option<String>,
}

impl PartialEq for SymbolValue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    String(String),
    StringConstructor,
    Bool(bool),
    Number(i64),
    Float(f64),
    BigInt(JsBigInt),
    Array(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<ObjectValue>>),
    Promise(Rc<RefCell<PromiseValue>>),
    Map(Rc<RefCell<MapValue>>),
    Set(Rc<RefCell<SetValue>>),
    Blob(Rc<RefCell<BlobValue>>),
    ArrayBuffer(Rc<RefCell<ArrayBufferValue>>),
    TypedArray(Rc<RefCell<TypedArrayValue>>),
    TypedArrayConstructor(TypedArrayConstructorKind),
    BlobConstructor,
    UrlConstructor,
    ArrayBufferConstructor,
    PromiseConstructor,
    MapConstructor,
    SetConstructor,
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
struct ObjectValue {
    entries: Vec<(String, Value)>,
    index_by_key: HashMap<String, usize>,
}

impl ObjectValue {
    fn new(entries: Vec<(String, Value)>) -> Self {
        let mut value = Self::default();
        for (key, entry_value) in entries {
            value.set_entry(key, entry_value);
        }
        value
    }

    fn set_entry(&mut self, key: String, value: Value) {
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

    fn get_entry(&self, key: &str) -> Option<Value> {
        self.index_by_key
            .get(key)
            .and_then(|index| self.entries.get(*index))
            .map(|(_, value)| value.clone())
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.index_by_key.clear();
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
struct RegexValue {
    source: String,
    flags: String,
    global: bool,
    ignore_case: bool,
    multiline: bool,
    dot_all: bool,
    sticky: bool,
    has_indices: bool,
    unicode: bool,
    compiled: Regex,
    last_index: usize,
    properties: ObjectValue,
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
            && self.last_index == other.last_index
            && self.properties == other.properties
    }
}

#[derive(Debug, Clone)]
struct FunctionValue {
    handler: ScriptHandler,
    captured_env: Rc<RefCell<ScriptEnv>>,
    captured_pending_function_decls: Vec<Arc<HashMap<String, (ScriptHandler, bool)>>>,
    captured_global_names: HashSet<String>,
    local_bindings: HashSet<String>,
    global_scope: bool,
    is_async: bool,
}

impl PartialEq for FunctionValue {
    fn eq(&self, other: &Self) -> bool {
        self.handler == other.handler
            && self.global_scope == other.global_scope
            && self.is_async == other.is_async
    }
}

#[derive(Debug, Clone, Copy)]
struct RegexFlags {
    global: bool,
    ignore_case: bool,
    multiline: bool,
    dot_all: bool,
    sticky: bool,
    has_indices: bool,
    unicode: bool,
}

impl Value {
    fn truthy(&self) -> bool {
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
            Self::Set(_) => true,
            Self::Blob(_) => true,
            Self::ArrayBuffer(_) => true,
            Self::TypedArray(_) => true,
            Self::TypedArrayConstructor(_) => true,
            Self::BlobConstructor => true,
            Self::UrlConstructor => true,
            Self::ArrayBufferConstructor => true,
            Self::PromiseConstructor => true,
            Self::MapConstructor => true,
            Self::SetConstructor => true,
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

    fn as_string(&self) -> String {
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
            Self::Set(_) => "[object Set]".into(),
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
            Self::SetConstructor => "Set".to_string(),
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
enum DomProp {
    Attributes,
    AssignedSlot,
    Value,
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
    Name,
    Lang,
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
enum DomIndex {
    Static(usize),
    Dynamic(String),
}

impl DomIndex {
    fn describe(&self) -> String {
        match self {
            Self::Static(index) => index.to_string(),
            Self::Dynamic(expr) => expr.clone(),
        }
    }

    fn static_index(&self) -> Option<usize> {
        match self {
            Self::Static(index) => Some(*index),
            Self::Dynamic(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomQuery {
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
enum FormDataSource {
    NewForm(DomQuery),
    Var(String),
}

impl DomQuery {
    fn describe_call(&self) -> String {
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
enum ClassListMethod {
    Add,
    Remove,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinaryOp {
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
enum VarAssignOp {
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
enum EventExprProp {
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
enum MatchMediaProp {
    Matches,
    Media,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntlFormatterKind {
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
enum IntlStaticMethod {
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
    fn storage_name(self) -> &'static str {
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

    fn from_storage_name(value: &str) -> Option<Self> {
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
struct IntlDateTimeOptions {
    calendar: String,
    numbering_system: String,
    time_zone: String,
    date_style: Option<String>,
    time_style: Option<String>,
    weekday: Option<String>,
    year: Option<String>,
    month: Option<String>,
    day: Option<String>,
    hour: Option<String>,
    minute: Option<String>,
    second: Option<String>,
    fractional_second_digits: Option<u8>,
    time_zone_name: Option<String>,
    hour12: Option<bool>,
    day_period: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct IntlDateTimeComponents {
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
    weekday: u32,
    offset_minutes: i64,
}

#[derive(Debug, Clone)]
struct IntlPart {
    part_type: String,
    value: String,
}

#[derive(Debug, Clone)]
struct IntlRelativeTimePart {
    part_type: String,
    value: String,
    unit: Option<String>,
}

#[derive(Debug, Clone)]
struct IntlDisplayNamesOptions {
    style: String,
    display_type: String,
    fallback: String,
    language_display: String,
}

#[derive(Debug, Clone)]
struct IntlDurationOptions {
    style: String,
}

#[derive(Debug, Clone)]
struct IntlListOptions {
    style: String,
    list_type: String,
}

#[derive(Debug, Clone)]
struct IntlPluralRulesOptions {
    rule_type: String,
}

#[derive(Debug, Clone)]
struct IntlRelativeTimeOptions {
    style: String,
    numeric: String,
    locale_matcher: String,
}

#[derive(Debug, Clone)]
struct IntlSegmenterOptions {
    granularity: String,
    locale_matcher: String,
}

#[derive(Debug, Clone)]
struct IntlLocaleOptions {
    language: Option<String>,
    script: Option<String>,
    region: Option<String>,
    calendar: Option<String>,
    case_first: Option<String>,
    collation: Option<String>,
    hour_cycle: Option<String>,
    numbering_system: Option<String>,
    numeric: Option<bool>,
}

#[derive(Debug, Clone)]
struct IntlLocaleData {
    language: String,
    script: Option<String>,
    region: Option<String>,
    variants: Vec<String>,
    calendar: Option<String>,
    case_first: Option<String>,
    collation: Option<String>,
    hour_cycle: Option<String>,
    numbering_system: Option<String>,
    numeric: Option<bool>,
}

