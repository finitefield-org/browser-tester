use super::*;

impl Dom {
    pub(crate) fn new() -> Self {
        let root = Node {
            parent: None,
            children: Vec::new(),
            node_type: NodeType::Document,
        };
        Self {
            nodes: vec![root],
            root: NodeId(0),
            id_index: HashMap::new(),
            active_element: None,
            active_pseudo_element: None,
        }
    }

    pub(crate) fn create_node(&mut self, parent: Option<NodeId>, node_type: NodeType) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            parent,
            children: Vec::new(),
            node_type,
        });
        if let Some(parent_id) = parent {
            self.nodes[parent_id.0].children.push(id);
        }
        id
    }

    pub(crate) fn create_element(
        &mut self,
        parent: NodeId,
        tag_name: String,
        attrs: HashMap<String, String>,
    ) -> NodeId {
        let value = attrs.get("value").cloned().unwrap_or_default();
        let value_len = value.chars().count();
        let checked = attrs.contains_key("checked");
        let disabled = attrs.contains_key("disabled");
        let readonly = attrs.contains_key("readonly");
        let required = attrs.contains_key("required");
        let element = Element {
            tag_name,
            attrs,
            value,
            checked,
            indeterminate: false,
            disabled,
            readonly,
            required,
            custom_validity_message: String::new(),
            selection_start: value_len,
            selection_end: value_len,
            selection_direction: "none".to_string(),
        };
        let id = self.create_node(Some(parent), NodeType::Element(element));
        if let Some(id_attr) = self
            .element(id)
            .and_then(|element| element.attrs.get("id").cloned())
        {
            self.index_id(&id_attr, id);
        }
        id
    }

    pub(crate) fn create_detached_element(&mut self, tag_name: String) -> NodeId {
        let element = Element {
            tag_name,
            attrs: HashMap::new(),
            value: String::new(),
            checked: false,
            indeterminate: false,
            disabled: false,
            readonly: false,
            required: false,
            custom_validity_message: String::new(),
            selection_start: 0,
            selection_end: 0,
            selection_direction: "none".to_string(),
        };
        self.create_node(None, NodeType::Element(element))
    }

    pub(crate) fn create_detached_text(&mut self, text: String) -> NodeId {
        self.create_node(None, NodeType::Text(text))
    }

    pub(crate) fn create_text(&mut self, parent: NodeId, text: String) -> NodeId {
        self.create_node(Some(parent), NodeType::Text(text))
    }

    pub(crate) fn element(&self, node_id: NodeId) -> Option<&Element> {
        match &self.nodes[node_id.0].node_type {
            NodeType::Element(element) => Some(element),
            _ => None,
        }
    }

    pub(crate) fn element_mut(&mut self, node_id: NodeId) -> Option<&mut Element> {
        match &mut self.nodes[node_id.0].node_type {
            NodeType::Element(element) => Some(element),
            _ => None,
        }
    }

    pub(crate) fn tag_name(&self, node_id: NodeId) -> Option<&str> {
        self.element(node_id).map(|e| e.tag_name.as_str())
    }

    pub(crate) fn parent(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes[node_id.0].parent
    }

    pub(crate) fn is_descendant_of(&self, node_id: NodeId, ancestor: NodeId) -> bool {
        let mut cursor = self.parent(node_id);
        while let Some(current) = cursor {
            if current == ancestor {
                return true;
            }
            cursor = self.parent(current);
        }
        false
    }

    pub(crate) fn active_element(&self) -> Option<NodeId> {
        self.active_element
    }

    pub(crate) fn set_active_element(&mut self, node: Option<NodeId>) {
        self.active_element = node;
    }

    pub(crate) fn active_pseudo_element(&self) -> Option<NodeId> {
        self.active_pseudo_element
    }

    pub(crate) fn set_active_pseudo_element(&mut self, node: Option<NodeId>) {
        self.active_pseudo_element = node;
    }

    pub(crate) fn by_id(&self, id: &str) -> Option<NodeId> {
        self.id_index.get(id).and_then(|ids| ids.first().copied())
    }

    pub(crate) fn by_id_all(&self, id: &str) -> Vec<NodeId> {
        self.id_index.get(id).cloned().unwrap_or_default()
    }

    pub(crate) fn index_id(&mut self, id: &str, node_id: NodeId) {
        if id.is_empty() {
            return;
        }
        self.id_index
            .entry(id.to_string())
            .or_default()
            .push(node_id);
    }

    pub(crate) fn unindex_id(&mut self, id: &str, node_id: NodeId) {
        let Some(nodes) = self.id_index.get_mut(id) else {
            return;
        };
        nodes.retain(|candidate| *candidate != node_id);
        if nodes.is_empty() {
            self.id_index.remove(id);
        }
    }
}
