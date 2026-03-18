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
    pub(crate) fn concrete_kinds() -> &'static [Self] {
        const KINDS: [TypedArrayKind; 12] = [
            TypedArrayKind::Int8,
            TypedArrayKind::Uint8,
            TypedArrayKind::Uint8Clamped,
            TypedArrayKind::Int16,
            TypedArrayKind::Uint16,
            TypedArrayKind::Int32,
            TypedArrayKind::Uint32,
            TypedArrayKind::Float16,
            TypedArrayKind::Float32,
            TypedArrayKind::Float64,
            TypedArrayKind::BigInt64,
            TypedArrayKind::BigUint64,
        ];
        &KINDS
    }

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
    pub(crate) properties: ObjectValue,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LiveNodeListSource {
    ChildNodes {
        parent: NodeId,
    },
    ChildElements {
        parent: NodeId,
    },
    FormElements {
        form: NodeId,
    },
    FormElementsNamedGroup {
        form: NodeId,
        name: String,
    },
    SelectOptions {
        select: NodeId,
    },
    SelectedOptions {
        select: NodeId,
    },
    DataListOptions {
        datalist: NodeId,
    },
    MediaTextTracks {
        media: NodeId,
    },
    DescendantsByClassNames {
        root: NodeId,
        class_names: Vec<String>,
    },
    DescendantsByName {
        root: NodeId,
        name: String,
    },
    DescendantsByTagName {
        root: NodeId,
        tag_name: String,
    },
    DescendantsByTagNameNs {
        root: NodeId,
        namespace_uri: Option<String>,
        local_name: String,
    },
    QuerySelectorAll {
        root: NodeId,
        selector: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeListKind {
    NodeList,
    TextTrackList,
    RadioNodeList,
    HtmlCollection,
    HtmlFormControlsCollection,
    HtmlOptionsCollection,
}

impl NodeListKind {
    pub(crate) fn display_name(&self) -> &'static str {
        match self {
            Self::NodeList => "NodeList",
            Self::TextTrackList => "TextTrackList",
            Self::RadioNodeList => "RadioNodeList",
            Self::HtmlCollection => "HTMLCollection",
            Self::HtmlFormControlsCollection => "HTMLFormControlsCollection",
            Self::HtmlOptionsCollection => "HTMLOptionsCollection",
        }
    }

    pub(crate) fn is_html_collection_family(&self) -> bool {
        matches!(
            self,
            Self::HtmlCollection | Self::HtmlFormControlsCollection | Self::HtmlOptionsCollection
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NodeListValue {
    pub(crate) nodes: Vec<NodeId>,
    pub(crate) kind: NodeListKind,
    pub(crate) live_source: Option<LiveNodeListSource>,
    pub(crate) properties: ObjectValue,
}

impl NodeListValue {
    pub(crate) fn static_list(nodes: Vec<NodeId>) -> Self {
        Self {
            nodes,
            kind: NodeListKind::NodeList,
            live_source: None,
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_child_nodes(parent: NodeId, nodes: Vec<NodeId>) -> Self {
        Self {
            nodes,
            kind: NodeListKind::NodeList,
            live_source: Some(LiveNodeListSource::ChildNodes { parent }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_child_elements(parent: NodeId, nodes: Vec<NodeId>) -> Self {
        Self {
            nodes,
            kind: NodeListKind::HtmlCollection,
            live_source: Some(LiveNodeListSource::ChildElements { parent }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_form_elements(form: NodeId, nodes: Vec<NodeId>) -> Self {
        Self {
            nodes,
            kind: NodeListKind::HtmlFormControlsCollection,
            live_source: Some(LiveNodeListSource::FormElements { form }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_form_elements_named_group(
        form: NodeId,
        name: String,
        nodes: Vec<NodeId>,
    ) -> Self {
        Self {
            nodes,
            kind: NodeListKind::RadioNodeList,
            live_source: Some(LiveNodeListSource::FormElementsNamedGroup { form, name }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_select_options(select: NodeId, nodes: Vec<NodeId>) -> Self {
        Self {
            nodes,
            kind: NodeListKind::HtmlOptionsCollection,
            live_source: Some(LiveNodeListSource::SelectOptions { select }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_selected_options(select: NodeId, nodes: Vec<NodeId>) -> Self {
        Self {
            nodes,
            kind: NodeListKind::HtmlCollection,
            live_source: Some(LiveNodeListSource::SelectedOptions { select }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_datalist_options(datalist: NodeId, nodes: Vec<NodeId>) -> Self {
        Self {
            nodes,
            kind: NodeListKind::HtmlCollection,
            live_source: Some(LiveNodeListSource::DataListOptions { datalist }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_media_text_tracks(media: NodeId, nodes: Vec<NodeId>) -> Self {
        Self {
            nodes,
            kind: NodeListKind::TextTrackList,
            live_source: Some(LiveNodeListSource::MediaTextTracks { media }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_descendants_by_class_names(
        root: NodeId,
        class_names: Vec<String>,
        nodes: Vec<NodeId>,
    ) -> Self {
        Self {
            nodes,
            kind: NodeListKind::HtmlCollection,
            live_source: Some(LiveNodeListSource::DescendantsByClassNames { root, class_names }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_descendants_by_name(root: NodeId, name: String, nodes: Vec<NodeId>) -> Self {
        Self {
            nodes,
            kind: NodeListKind::NodeList,
            live_source: Some(LiveNodeListSource::DescendantsByName { root, name }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_descendants_by_tag_name(
        root: NodeId,
        tag_name: String,
        nodes: Vec<NodeId>,
    ) -> Self {
        Self {
            nodes,
            kind: NodeListKind::HtmlCollection,
            live_source: Some(LiveNodeListSource::DescendantsByTagName { root, tag_name }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_descendants_by_tag_name_ns(
        root: NodeId,
        namespace_uri: Option<String>,
        local_name: String,
        nodes: Vec<NodeId>,
    ) -> Self {
        Self {
            nodes,
            kind: NodeListKind::HtmlCollection,
            live_source: Some(LiveNodeListSource::DescendantsByTagNameNs {
                root,
                namespace_uri,
                local_name,
            }),
            properties: ObjectValue::default(),
        }
    }

    pub(crate) fn live_query_selector_all(
        root: NodeId,
        selector: String,
        kind: NodeListKind,
        nodes: Vec<NodeId>,
    ) -> Self {
        Self {
            nodes,
            kind,
            live_source: Some(LiveNodeListSource::QuerySelectorAll { root, selector }),
            properties: ObjectValue::default(),
        }
    }
}
