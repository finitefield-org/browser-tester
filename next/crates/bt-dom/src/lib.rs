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
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectionState {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileInputState {
    pub files: Vec<String>,
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
