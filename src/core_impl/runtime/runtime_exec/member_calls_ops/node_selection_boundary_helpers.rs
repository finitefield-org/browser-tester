use super::*;

impl Harness {
    pub(crate) fn selection_boundary_node_from_value(&self, value: &Value) -> Result<NodeId> {
        match value {
            Value::Node(node) if self.dom.is_valid_node(*node) => Ok(*node),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    Ok(self.dom.root)
                } else {
                    Err(Error::ScriptRuntime(
                        "Selection boundary container must be a Node".into(),
                    ))
                }
            }
            _ => Err(Error::ScriptRuntime(
                "Selection boundary container must be a Node".into(),
            )),
        }
    }

    pub(crate) fn selection_clamped_offset(&self, node: NodeId, offset: i64) -> i64 {
        let max = match &self.dom.nodes[node.0].node_type {
            NodeType::Text(text) => text.chars().count() as i64,
            NodeType::Document | NodeType::Element(_) => {
                self.dom.nodes[node.0].children.len() as i64
            }
        };
        offset.max(0).min(max)
    }

    fn selection_boundary_char_index_in_subtree(
        &self,
        current: NodeId,
        target: NodeId,
        target_offset: i64,
        prefix: usize,
    ) -> Option<usize> {
        if current == target {
            let clamped = self.selection_clamped_offset(target, target_offset) as usize;
            let index = match &self.dom.nodes[target.0].node_type {
                NodeType::Text(_) => prefix + clamped,
                NodeType::Document | NodeType::Element(_) => {
                    let children = &self.dom.nodes[target.0].children;
                    let upto = clamped.min(children.len());
                    let mut out = prefix;
                    for child in children.iter().take(upto) {
                        out += self.dom.text_content(*child).chars().count();
                    }
                    out
                }
            };
            return Some(index);
        }

        if matches!(self.dom.nodes[current.0].node_type, NodeType::Text(_)) {
            return None;
        }

        let mut running = prefix;
        for child in &self.dom.nodes[current.0].children {
            if let Some(found) = self.selection_boundary_char_index_in_subtree(
                *child,
                target,
                target_offset,
                running,
            ) {
                return Some(found);
            }
            running += self.dom.text_content(*child).chars().count();
        }
        None
    }

    pub(crate) fn selection_boundary_char_index(&self, node: NodeId, offset: i64) -> Option<usize> {
        if !self.dom.is_valid_node(node) {
            return None;
        }
        self.selection_boundary_char_index_in_subtree(self.dom.root, node, offset, 0)
    }

    pub(crate) fn selection_compare_points(
        &self,
        left_node: NodeId,
        left_offset: i64,
        right_node: NodeId,
        right_offset: i64,
    ) -> std::cmp::Ordering {
        let left = self
            .selection_boundary_char_index(left_node, left_offset)
            .unwrap_or(0);
        let right = self
            .selection_boundary_char_index(right_node, right_offset)
            .unwrap_or(0);
        left.cmp(&right)
    }

    pub(crate) fn selection_anchor_focus(
        &self,
        selection: &Rc<RefCell<ObjectValue>>,
    ) -> Option<(NodeId, i64, NodeId, i64)> {
        let entries = selection.borrow();
        let anchor_node = match Self::object_get_entry(&entries, "anchorNode") {
            Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
            _ => return None,
        };
        let focus_node = match Self::object_get_entry(&entries, "focusNode") {
            Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
            _ => return None,
        };
        let anchor_offset = match Self::object_get_entry(&entries, "anchorOffset") {
            Some(Value::Number(offset)) => offset,
            _ => 0,
        };
        let focus_offset = match Self::object_get_entry(&entries, "focusOffset") {
            Some(Value::Number(offset)) => offset,
            _ => 0,
        };
        Some((anchor_node, anchor_offset, focus_node, focus_offset))
    }

    pub(crate) fn selection_normalized_boundaries(
        &self,
        selection: &Rc<RefCell<ObjectValue>>,
    ) -> Option<(NodeId, i64, NodeId, i64)> {
        let entries = selection.borrow();
        let has_range = matches!(
            Self::object_get_entry(&entries, "rangeCount"),
            Some(Value::Number(count)) if count > 0
        );
        if !has_range {
            return None;
        }
        let range_object = match Self::object_get_entry(&entries, INTERNAL_SELECTION_RANGE_KEY) {
            Some(Value::Object(range)) => range,
            _ => return None,
        };
        let range_entries = range_object.borrow();
        let start_container =
            match Self::object_get_entry(&range_entries, INTERNAL_RANGE_START_CONTAINER_KEY) {
                Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
                _ => return None,
            };
        let start_offset =
            match Self::object_get_entry(&range_entries, INTERNAL_RANGE_START_OFFSET_KEY) {
                Some(Value::Number(offset)) => offset,
                _ => 0,
            };
        let end_container =
            match Self::object_get_entry(&range_entries, INTERNAL_RANGE_END_CONTAINER_KEY) {
                Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
                _ => return None,
            };
        let end_offset = match Self::object_get_entry(&range_entries, INTERNAL_RANGE_END_OFFSET_KEY)
        {
            Some(Value::Number(offset)) => offset,
            _ => 0,
        };
        Some((start_container, start_offset, end_container, end_offset))
    }

    pub(crate) fn selection_slice_text_by_char_index(
        value: &str,
        start: usize,
        end: usize,
    ) -> String {
        if end <= start {
            return String::new();
        }
        let start_byte = Self::char_index_to_byte(value, start);
        let end_byte = Self::char_index_to_byte(value, end);
        value[start_byte..end_byte].to_string()
    }

    pub(crate) fn selection_text(&self, selection: &Rc<RefCell<ObjectValue>>) -> String {
        let Some((start_container, start_offset, end_container, end_offset)) =
            self.selection_normalized_boundaries(selection)
        else {
            return String::new();
        };
        let full = self.dom.text_content(self.dom.root);
        let start = self
            .selection_boundary_char_index(start_container, start_offset)
            .unwrap_or(0);
        let end = self
            .selection_boundary_char_index(end_container, end_offset)
            .unwrap_or(start);
        let len = full.chars().count();
        let start = start.min(len);
        let end = end.min(len).max(start);
        Self::selection_slice_text_by_char_index(&full, start, end)
    }

    pub(crate) fn selection_node_boundary_char_indexes(
        &self,
        node: NodeId,
    ) -> Option<(usize, usize)> {
        if !self.dom.is_valid_node(node) {
            return None;
        }
        if node == self.dom.root {
            let total = self.dom.text_content(self.dom.root).chars().count();
            return Some((0, total));
        }
        let parent = self.dom.parent(node)?;
        let index = self.dom.nodes[parent.0]
            .children
            .iter()
            .position(|candidate| *candidate == node)?;
        let start = self
            .selection_boundary_char_index(parent, index as i64)
            .unwrap_or(0);
        let end = self
            .selection_boundary_char_index(parent, (index + 1) as i64)
            .unwrap_or(start);
        Some((start, end))
    }
}
