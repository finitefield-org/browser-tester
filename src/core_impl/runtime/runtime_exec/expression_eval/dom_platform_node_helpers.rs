use super::*;

impl Harness {
    fn viewport_inner_height_value(&self) -> i64 {
        const DEFAULT_INNER_HEIGHT: f64 = 768.0;
        let window = self.dom_runtime.window_object.borrow();
        let raw_value = Self::object_get_entry(&window, "innerHeight");
        let parsed = match raw_value {
            Some(Value::Number(value)) => Some(value as f64),
            Some(Value::Float(value)) if value.is_finite() => Some(value),
            Some(Value::String(value)) => value.parse::<f64>().ok(),
            _ => None,
        }
        .unwrap_or(DEFAULT_INNER_HEIGHT);
        if !parsed.is_finite() {
            return DEFAULT_INNER_HEIGHT as i64;
        }
        parsed.max(0.0).trunc() as i64
    }

    fn viewport_inner_width_value(&self) -> i64 {
        const DEFAULT_INNER_WIDTH: f64 = 1024.0;
        let window = self.dom_runtime.window_object.borrow();
        let raw_value = Self::object_get_entry(&window, "innerWidth");
        let parsed = match raw_value {
            Some(Value::Number(value)) => Some(value as f64),
            Some(Value::Float(value)) if value.is_finite() => Some(value),
            Some(Value::String(value)) => value.parse::<f64>().ok(),
            _ => None,
        }
        .unwrap_or(DEFAULT_INNER_WIDTH);
        if !parsed.is_finite() {
            return DEFAULT_INNER_WIDTH as i64;
        }
        parsed.max(0.0).trunc() as i64
    }

    pub(crate) fn client_width_property_value(&self, node: NodeId) -> Result<i64> {
        let is_document_html_element = self.dom.document_element() == Some(node)
            && self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("html"));
        if is_document_html_element {
            return Ok(self.viewport_inner_width_value());
        }
        self.dom.client_width(node)
    }

    pub(crate) fn client_height_property_value(&self, node: NodeId) -> Result<i64> {
        let is_document_html_element = self.dom.document_element() == Some(node)
            && self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("html"));
        if is_document_html_element {
            return Ok(self.viewport_inner_height_value());
        }
        self.dom.client_height(node)
    }

    pub(crate) fn document_active_element_property_value(&self) -> Value {
        self.dom
            .active_element()
            .filter(|node| self.dom.is_connected(*node))
            .or_else(|| self.dom.body())
            .or_else(|| self.dom.document_element())
            .map(Value::Node)
            .unwrap_or(Value::Null)
    }

    pub(crate) fn node_type_number(&self, node: NodeId) -> i64 {
        match &self.dom.nodes[node.0].node_type {
            NodeType::Document => 9,
            NodeType::Text(_) => 3,
            NodeType::Element(element)
                if element.tag_name.eq_ignore_ascii_case("#document-fragment") =>
            {
                11
            }
            NodeType::Element(_) => 1,
        }
    }

    pub(crate) fn node_name(&self, node: NodeId) -> String {
        match &self.dom.nodes[node.0].node_type {
            NodeType::Document => "#document".to_string(),
            NodeType::Text(_) => "#text".to_string(),
            NodeType::Element(element)
                if element.tag_name.eq_ignore_ascii_case("#document-fragment") =>
            {
                "#document-fragment".to_string()
            }
            NodeType::Element(_) => self.element_tag_name(node),
        }
    }

    pub(crate) fn element_tag_name(&self, node: NodeId) -> String {
        let Some(element) = self.dom.element(node) else {
            return String::new();
        };
        if element.namespace_uri.as_deref() == Some("http://www.w3.org/1999/xhtml") {
            element.tag_name.to_ascii_uppercase()
        } else {
            element.tag_name.clone()
        }
    }

    pub(crate) fn node_value(&self, node: NodeId) -> Value {
        match &self.dom.nodes[node.0].node_type {
            NodeType::Text(text) => Value::String(text.clone()),
            _ => Value::Null,
        }
    }

    pub(crate) fn node_text_content_value(&self, node: NodeId) -> Value {
        if matches!(self.dom.nodes[node.0].node_type, NodeType::Document) {
            Value::Null
        } else {
            Value::String(self.dom.text_content(node))
        }
    }

    pub(crate) fn node_root(&self, node: NodeId) -> NodeId {
        let mut current = node;
        while let Some(parent) = self.dom.parent(current) {
            current = parent;
        }
        current
    }

    pub(crate) fn node_owner_document(&self, node: NodeId) -> Option<NodeId> {
        if matches!(self.dom.nodes[node.0].node_type, NodeType::Document) {
            return None;
        }
        let root = self.node_root(node);
        if matches!(self.dom.nodes[root.0].node_type, NodeType::Document) {
            Some(root)
        } else {
            Some(self.dom.root)
        }
    }

    pub(crate) fn node_parent_element(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.dom.parent(node)?;
        match &self.dom.nodes[parent.0].node_type {
            NodeType::Element(element)
                if !element.tag_name.eq_ignore_ascii_case("#document-fragment") =>
            {
                Some(parent)
            }
            _ => None,
        }
    }

    pub(crate) fn node_previous_sibling(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.dom.parent(node)?;
        let siblings = &self.dom.nodes[parent.0].children;
        let position = siblings.iter().position(|sibling| *sibling == node)?;
        position.checked_sub(1).map(|index| siblings[index])
    }

    pub(crate) fn node_next_sibling(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.dom.parent(node)?;
        let siblings = &self.dom.nodes[parent.0].children;
        let position = siblings.iter().position(|sibling| *sibling == node)?;
        siblings.get(position + 1).copied()
    }

    fn node_document_order_index(&self, root: NodeId, target: NodeId) -> Option<usize> {
        let mut stack = vec![root];
        let mut index = 0usize;
        while let Some(current) = stack.pop() {
            if current == target {
                return Some(index);
            }
            index += 1;
            for child in self.dom.nodes[current.0].children.iter().rev() {
                stack.push(*child);
            }
        }
        None
    }

    pub(crate) fn node_compare_document_position(&self, left: NodeId, right: NodeId) -> i64 {
        const DOCUMENT_POSITION_DISCONNECTED: i64 = 0x01;
        const DOCUMENT_POSITION_PRECEDING: i64 = 0x02;
        const DOCUMENT_POSITION_FOLLOWING: i64 = 0x04;
        const DOCUMENT_POSITION_CONTAINS: i64 = 0x08;
        const DOCUMENT_POSITION_CONTAINED_BY: i64 = 0x10;
        const DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: i64 = 0x20;

        if left == right {
            return 0;
        }

        let left_root = self.node_root(left);
        let right_root = self.node_root(right);
        if left_root != right_root {
            let disconnected_order = if left.0 < right.0 {
                DOCUMENT_POSITION_FOLLOWING
            } else {
                DOCUMENT_POSITION_PRECEDING
            };
            return DOCUMENT_POSITION_DISCONNECTED
                | DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC
                | disconnected_order;
        }

        if self.dom.is_descendant_of(right, left) {
            return DOCUMENT_POSITION_CONTAINED_BY | DOCUMENT_POSITION_FOLLOWING;
        }
        if self.dom.is_descendant_of(left, right) {
            return DOCUMENT_POSITION_CONTAINS | DOCUMENT_POSITION_PRECEDING;
        }

        let left_index = self.node_document_order_index(left_root, left).unwrap_or(0);
        let right_index = self
            .node_document_order_index(left_root, right)
            .unwrap_or(0);
        if left_index < right_index {
            DOCUMENT_POSITION_FOLLOWING
        } else {
            DOCUMENT_POSITION_PRECEDING
        }
    }

    pub(crate) fn nodes_are_equal(&self, left: NodeId, right: NodeId) -> bool {
        let left_node = &self.dom.nodes[left.0];
        let right_node = &self.dom.nodes[right.0];
        let metadata_equal = match (&left_node.node_type, &right_node.node_type) {
            (NodeType::Document, NodeType::Document) => true,
            (NodeType::Text(left_text), NodeType::Text(right_text)) => left_text == right_text,
            (NodeType::Element(left_element), NodeType::Element(right_element)) => {
                left_element
                    .tag_name
                    .eq_ignore_ascii_case(&right_element.tag_name)
                    && left_element.attrs == right_element.attrs
                    && left_element.value == right_element.value
                    && left_element.files == right_element.files
                    && left_element.checked == right_element.checked
                    && left_element.indeterminate == right_element.indeterminate
                    && left_element.disabled == right_element.disabled
                    && left_element.readonly == right_element.readonly
                    && left_element.required == right_element.required
                    && left_element.custom_validity_message == right_element.custom_validity_message
                    && left_element.selection_start == right_element.selection_start
                    && left_element.selection_end == right_element.selection_end
                    && left_element.selection_direction == right_element.selection_direction
            }
            _ => false,
        };
        if !metadata_equal {
            return false;
        }
        if left_node.children.len() != right_node.children.len() {
            return false;
        }
        left_node
            .children
            .iter()
            .zip(right_node.children.iter())
            .all(|(left_child, right_child)| self.nodes_are_equal(*left_child, *right_child))
    }

    pub(crate) fn normalize_node_subtree(&mut self, node: NodeId) -> Result<()> {
        let direct_children = self.dom.nodes[node.0].children.clone();
        for child in direct_children {
            if self.dom.parent(child) == Some(node) {
                self.normalize_node_subtree(child)?;
            }
        }

        let mut index = 0usize;
        while index < self.dom.nodes[node.0].children.len() {
            let current = self.dom.nodes[node.0].children[index];
            let Some(mut merged_text) = (match &self.dom.nodes[current.0].node_type {
                NodeType::Text(text) => Some(text.clone()),
                _ => None,
            }) else {
                index += 1;
                continue;
            };

            loop {
                let Some(next) = self.dom.nodes[node.0].children.get(index + 1).copied() else {
                    break;
                };
                let Some(next_text) = (match &self.dom.nodes[next.0].node_type {
                    NodeType::Text(text) => Some(text.clone()),
                    _ => None,
                }) else {
                    break;
                };
                merged_text.push_str(&next_text);
                self.dom.remove_child(node, next)?;
            }

            if let NodeType::Text(text) = &mut self.dom.nodes[current.0].node_type {
                *text = merged_text.clone();
            }
            if merged_text.is_empty() {
                self.dom.remove_child(node, current)?;
                continue;
            }
            index += 1;
        }

        Ok(())
    }

    pub(crate) fn node_lookup_namespace_uri(
        &self,
        node: NodeId,
        prefix: Option<&str>,
    ) -> Option<String> {
        let element = self.dom.element(node)?;
        let normalized_prefix = prefix.unwrap_or_default();
        if normalized_prefix.is_empty() {
            return element.namespace_uri.clone();
        }
        element
            .tag_name
            .split_once(':')
            .filter(|(node_prefix, _)| *node_prefix == normalized_prefix)
            .and_then(|_| element.namespace_uri.clone())
    }

    pub(crate) fn node_lookup_prefix(
        &self,
        node: NodeId,
        namespace_uri: Option<&str>,
    ) -> Option<String> {
        let element = self.dom.element(node)?;
        let Some(namespace_uri) = namespace_uri else {
            return None;
        };
        if element.namespace_uri.as_deref() != Some(namespace_uri) {
            return None;
        }
        element
            .tag_name
            .split_once(':')
            .map(|(prefix, _)| prefix.to_string())
    }

    pub(crate) fn node_is_default_namespace(
        &self,
        node: NodeId,
        namespace_uri: Option<&str>,
    ) -> bool {
        let default_namespace = self.node_lookup_namespace_uri(node, None);
        match (namespace_uri, default_namespace.as_deref()) {
            (None, None) => true,
            (Some(namespace_uri), Some(default_namespace)) => namespace_uri == default_namespace,
            _ => false,
        }
    }

    pub(crate) fn clone_dom_node(&mut self, node: NodeId, deep: bool) -> Result<NodeId> {
        let source = self.dom.clone();
        let cloned = self
            .dom
            .create_node(None, source.nodes[node.0].node_type.clone());
        if deep {
            let children = source.nodes[node.0].children.clone();
            for child in children {
                let _ = self
                    .dom
                    .clone_subtree_from_dom(&source, child, Some(cloned), false)?;
            }
        }
        Ok(cloned)
    }

    pub(crate) fn template_content_fragment_value(
        &mut self,
        template_node: NodeId,
    ) -> Result<Value> {
        let source = self.dom.clone();
        let fragment = self
            .dom
            .create_detached_element("#document-fragment".to_string());
        let children = source.nodes[template_node.0].children.clone();
        for child in children {
            let _ = self
                .dom
                .clone_subtree_from_dom(&source, child, Some(fragment), false)?;
        }
        Ok(Value::Node(fragment))
    }
}
