use super::*;

#[path = "runtime_values_collections.rs"]
mod runtime_values_collections;
#[path = "runtime_values_dom.rs"]
mod runtime_values_dom;
#[path = "runtime_values_intl.rs"]
mod runtime_values_intl;

pub(crate) use runtime_values_collections::*;
pub(crate) use runtime_values_dom::*;
pub(crate) use runtime_values_intl::*;

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
    UrlSearchParamsConstructor,
    SymbolConstructor,
    RegExpConstructor,
    PromiseCapability(Rc<PromiseCapabilityFunction>),
    Symbol(Rc<SymbolValue>),
    RegExp(Rc<RefCell<RegexValue>>),
    Date(Rc<RefCell<i64>>),
    Null,
    Undefined,
    Node(NodeId),
    NodeList(Rc<RefCell<NodeListValue>>),
    FormData(Rc<RefCell<Vec<(String, String)>>>),
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

    pub(crate) fn delete_entry(&mut self, key: &str) -> bool {
        let Some(index) = self.index_by_key.remove(key) else {
            return false;
        };
        self.entries.remove(index);
        for (offset, (entry_key, _)) in self.entries.iter().enumerate().skip(index) {
            self.index_by_key.insert(entry_key.clone(), offset);
        }
        true
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
    pub(crate) function_id: usize,
    pub(crate) handler: ScriptHandler,
    pub(crate) expression_name: Option<String>,
    pub(crate) captured_env: Rc<RefCell<ScriptEnv>>,
    pub(crate) captured_pending_function_decls:
        Vec<Arc<HashMap<String, (ScriptHandler, bool, bool)>>>,
    pub(crate) captured_global_names: HashSet<String>,
    pub(crate) captured_names: HashSet<String>,
    pub(crate) local_bindings: HashSet<String>,
    pub(crate) prototype_object: Rc<RefCell<ObjectValue>>,
    pub(crate) global_scope: bool,
    pub(crate) is_async: bool,
    pub(crate) is_generator: bool,
    pub(crate) is_arrow: bool,
    pub(crate) is_method: bool,
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
            && self.is_arrow == other.is_arrow
            && self.is_method == other.is_method
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
            Self::Float(v) => *v != 0.0 && !v.is_nan(),
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
            Self::UrlSearchParamsConstructor => true,
            Self::SymbolConstructor => true,
            Self::RegExpConstructor => true,
            Self::PromiseCapability(_) => true,
            Self::Symbol(_) => true,
            Self::RegExp(_) => true,
            Self::Date(_) => true,
            Self::Null => false,
            Self::Undefined => false,
            Self::Node(_) => true,
            Self::NodeList(nodes) => !nodes.borrow().nodes.is_empty(),
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
                if let Some(Value::Bool(value)) =
                    entries.get_entry(INTERNAL_BOOLEAN_WRAPPER_VALUE_KEY)
                {
                    return if value { "true".into() } else { "false".into() };
                }
                if let Some(Value::Number(value)) =
                    entries.get_entry(INTERNAL_NUMBER_WRAPPER_VALUE_KEY)
                {
                    return value.to_string();
                }
                if let Some(Value::Float(value)) =
                    entries.get_entry(INTERNAL_NUMBER_WRAPPER_VALUE_KEY)
                {
                    return format_float(value);
                }
                if let Some(Value::BigInt(value)) =
                    entries.get_entry(INTERNAL_BIGINT_WRAPPER_VALUE_KEY)
                {
                    return value.to_string();
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
                    let is_writable_stream = matches!(
                        entries.get_entry(INTERNAL_WRITABLE_STREAM_OBJECT_KEY),
                        Some(Value::Bool(true))
                    );
                    let is_text_encoder = matches!(
                        entries.get_entry(INTERNAL_TEXT_ENCODER_OBJECT_KEY),
                        Some(Value::Bool(true))
                    );
                    let is_text_decoder = matches!(
                        entries.get_entry(INTERNAL_TEXT_DECODER_OBJECT_KEY),
                        Some(Value::Bool(true))
                    );
                    let is_text_encoder_stream = matches!(
                        entries.get_entry(INTERNAL_TEXT_ENCODER_STREAM_OBJECT_KEY),
                        Some(Value::Bool(true))
                    );
                    let is_text_decoder_stream = matches!(
                        entries.get_entry(INTERNAL_TEXT_DECODER_STREAM_OBJECT_KEY),
                        Some(Value::Bool(true))
                    );
                    let is_animation = matches!(
                        entries.get_entry(INTERNAL_ANIMATION_OBJECT_KEY),
                        Some(Value::Bool(true))
                    );
                    if is_readable_stream {
                        "[object ReadableStream]".into()
                    } else if is_writable_stream {
                        "[object WritableStream]".into()
                    } else if is_text_encoder {
                        "[object TextEncoder]".into()
                    } else if is_text_decoder {
                        "[object TextDecoder]".into()
                    } else if is_text_encoder_stream {
                        "[object TextEncoderStream]".into()
                    } else if is_text_decoder_stream {
                        "[object TextDecoderStream]".into()
                    } else if is_animation {
                        "[object Animation]".into()
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
            Self::UrlSearchParamsConstructor => "URLSearchParams".to_string(),
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
            Self::NodeList(nodes) => format!("[object {}]", nodes.borrow().kind.display_name()),
            Self::FormData(_) => "[object FormData]".into(),
            Self::Function(function) => {
                if function.function_id == usize::MAX {
                    "function () { [native code] }".to_string()
                } else {
                    format!("__bt_function_ref__({})", function.function_id)
                }
            }
        }
    }
}
