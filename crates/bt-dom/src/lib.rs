use std::collections::BTreeMap;

pub const HTML_NAMESPACE_URI: &str = "http://www.w3.org/1999/xhtml";
pub const SVG_NAMESPACE_URI: &str = "http://www.w3.org/2000/svg";
pub const MATHML_NAMESPACE_URI: &str = "http://www.w3.org/1998/Math/MathML";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId {
    index: u32,
    generation: u32,
}

impl NodeId {
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    pub const fn index(self) -> u32 {
        self.index
    }

    pub const fn generation(self) -> u32 {
        self.generation
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocumentState {
    pub title: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ElementData {
    pub tag_name: String,
    pub local_name: String,
    pub namespace_uri: String,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextData {
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Document,
    Element(ElementData),
    Text(TextData),
    Comment(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeRecord {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub kind: NodeKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FormControlState {
    pub value: String,
    pub checked: bool,
    pub indeterminate: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectionState {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileInputFile {
    pub name: String,
    pub mime_type: Option<String>,
    pub bytes: Vec<u8>,
    pub read_error: Option<String>,
}

impl FileInputFile {
    pub fn from_bytes(name: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            name: name.into(),
            mime_type: None,
            bytes: bytes.into(),
            read_error: None,
        }
    }

    pub fn from_text(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self::from_bytes(name, text.into().into_bytes())
    }

    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    pub fn with_read_error(mut self, error: impl Into<String>) -> Self {
        self.read_error = Some(error.into());
        self
    }

    pub fn size(&self) -> usize {
        self.bytes.len()
    }
}

impl From<String> for FileInputFile {
    fn from(value: String) -> Self {
        Self::from_bytes(value, Vec::<u8>::new())
    }
}

impl From<&str> for FileInputFile {
    fn from(value: &str) -> Self {
        Self::from_bytes(value, Vec::<u8>::new())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileInputState {
    pub files: Vec<FileInputFile>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DialogState {
    pub open: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayoutStubState {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DomIndexes {
    pub id_index: BTreeMap<String, NodeId>,
    pub name_index: BTreeMap<String, Vec<NodeId>>,
    pub tag_index: BTreeMap<String, Vec<NodeId>>,
    pub class_index: BTreeMap<String, Vec<NodeId>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DomSideTables {
    pub form_controls: BTreeMap<NodeId, FormControlState>,
    pub selection: BTreeMap<NodeId, SelectionState>,
    pub file_inputs: BTreeMap<NodeId, FileInputState>,
    pub dialogs: BTreeMap<NodeId, DialogState>,
    pub layout_stub: BTreeMap<NodeId, LayoutStubState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomStore {
    nodes: Vec<NodeRecord>,
    document: DocumentState,
    document_id: NodeId,
    indexes: DomIndexes,
    side_tables: DomSideTables,
    focused_node: Option<NodeId>,
    target_fragment: Option<String>,
    source_html: Option<String>,
}

impl Default for DomStore {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl DomStore {
    pub fn new_empty() -> Self {
        let document_id = NodeId::new(0, 0);
        let nodes = vec![NodeRecord {
            id: document_id,
            parent: None,
            children: Vec::new(),
            kind: NodeKind::Document,
        }];
        Self {
            nodes,
            document: DocumentState::default(),
            document_id,
            indexes: DomIndexes::default(),
            side_tables: DomSideTables::default(),
            focused_node: None,
            target_fragment: None,
            source_html: None,
        }
    }

    pub fn source_html(&self) -> Option<&str> {
        self.source_html.as_deref()
    }

    pub fn document_id(&self) -> NodeId {
        self.document_id
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn nodes(&self) -> &[NodeRecord] {
        &self.nodes
    }

    pub fn indexes(&self) -> &DomIndexes {
        &self.indexes
    }

    pub fn side_tables(&self) -> &DomSideTables {
        &self.side_tables
    }

    pub fn focused_node(&self) -> Option<NodeId> {
        self.focused_node
    }

    pub fn set_focused_node(&mut self, focused_node: Option<NodeId>) {
        self.focused_node = focused_node;
    }

    pub fn target_fragment(&self) -> Option<&str> {
        self.target_fragment.as_deref()
    }

    pub fn set_target_fragment(&mut self, target_fragment: Option<String>) {
        self.target_fragment = target_fragment;
    }

    pub fn document_state(&self) -> &DocumentState {
        &self.document
    }
}

mod html_dom;

#[cfg(test)]
mod tests {
    use super::{DomStore, NodeId, NodeKind};

    #[test]
    fn empty_store_has_document_root() {
        let store = DomStore::new_empty();
        assert_eq!(store.node_count(), 1);
        assert_eq!(store.nodes()[0].kind, NodeKind::Document);
        assert_eq!(store.source_html(), None);
    }

    #[test]
    fn bootstrap_html_is_recorded_for_phase_zero() {
        let mut store = DomStore::new_empty();
        store
            .bootstrap_html("<p>Hello</p>")
            .expect("HTML should parse");
        assert_eq!(store.source_html(), Some("<p>Hello</p>"));
        assert_eq!(store.node_count(), 3);
        assert_eq!(store.select("#missing").unwrap(), Vec::<NodeId>::new());
        assert_eq!(store.dump_dom(), "#document\n  <p>\n    \"Hello\"\n  </p>");
    }
}
