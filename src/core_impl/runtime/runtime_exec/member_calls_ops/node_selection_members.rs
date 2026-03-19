use super::*;

impl Harness {
    pub(crate) fn ensure_document_selection_object(&mut self) -> Value {
        let is_selection_object = {
            let entries = self.dom_runtime.selection_object.borrow();
            Self::is_selection_object(&entries)
        };
        if !is_selection_object {
            self.dom_runtime.selection_object =
                match Self::new_selection_object_value(self.dom.root) {
                    Value::Object(selection) => selection,
                    _ => Rc::new(RefCell::new(ObjectValue::default())),
                };
        }
        Value::Object(self.dom_runtime.selection_object.clone())
    }

    fn selection_boundary_node_from_value(&self, value: &Value) -> Result<NodeId> {
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

    fn selection_clamped_offset(&self, node: NodeId, offset: i64) -> i64 {
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

    fn selection_boundary_char_index(&self, node: NodeId, offset: i64) -> Option<usize> {
        if !self.dom.is_valid_node(node) {
            return None;
        }
        self.selection_boundary_char_index_in_subtree(self.dom.root, node, offset, 0)
    }

    fn selection_compare_points(
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

    fn selection_internal_range_object(
        &mut self,
        selection: &Rc<RefCell<ObjectValue>>,
    ) -> Rc<RefCell<ObjectValue>> {
        let existing = {
            let entries = selection.borrow();
            match Self::object_get_entry(&entries, INTERNAL_SELECTION_RANGE_KEY) {
                Some(Value::Object(range)) => Some(range),
                _ => None,
            }
        };
        if let Some(range) = existing {
            return range;
        }

        let range = match Self::new_range_object_value(self.dom.root) {
            Value::Object(range) => range,
            _ => Rc::new(RefCell::new(ObjectValue::default())),
        };
        Self::object_set_entry(
            &mut selection.borrow_mut(),
            INTERNAL_SELECTION_RANGE_KEY.to_string(),
            Value::Object(range.clone()),
        );
        range
    }

    fn selection_anchor_focus(
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

    fn selection_normalized_boundaries(
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

    fn selection_slice_text_by_char_index(value: &str, start: usize, end: usize) -> String {
        if end <= start {
            return String::new();
        }
        let start_byte = Self::char_index_to_byte(value, start);
        let end_byte = Self::char_index_to_byte(value, end);
        value[start_byte..end_byte].to_string()
    }

    fn selection_text(&self, selection: &Rc<RefCell<ObjectValue>>) -> String {
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

    fn selection_node_boundary_char_indexes(&self, node: NodeId) -> Option<(usize, usize)> {
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

    fn selection_set_state(
        &mut self,
        anchor_node: Option<NodeId>,
        anchor_offset: i64,
        focus_node: Option<NodeId>,
        focus_offset: i64,
    ) -> bool {
        let selection = match self.ensure_document_selection_object() {
            Value::Object(selection) => selection,
            _ => return false,
        };

        let before = {
            let entries = selection.borrow();
            (
                Self::object_get_entry(&entries, "anchorNode"),
                Self::object_get_entry(&entries, "anchorOffset"),
                Self::object_get_entry(&entries, "focusNode"),
                Self::object_get_entry(&entries, "focusOffset"),
                Self::object_get_entry(&entries, "rangeCount"),
                Self::object_get_entry(&entries, "direction"),
            )
        };

        if let (Some(anchor_node), Some(focus_node)) = (anchor_node, focus_node) {
            let anchor_offset = self.selection_clamped_offset(anchor_node, anchor_offset);
            let focus_offset = self.selection_clamped_offset(focus_node, focus_offset);
            let ordering =
                self.selection_compare_points(anchor_node, anchor_offset, focus_node, focus_offset);
            let (start_container, start_offset, end_container, end_offset, direction) =
                match ordering {
                    std::cmp::Ordering::Greater => (
                        focus_node,
                        focus_offset,
                        anchor_node,
                        anchor_offset,
                        "backward",
                    ),
                    std::cmp::Ordering::Equal => {
                        (anchor_node, anchor_offset, focus_node, focus_offset, "none")
                    }
                    std::cmp::Ordering::Less => (
                        anchor_node,
                        anchor_offset,
                        focus_node,
                        focus_offset,
                        "forward",
                    ),
                };
            let range_object = self.selection_internal_range_object(&selection);
            {
                let mut range_entries = range_object.borrow_mut();
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_CONTAINER_KEY.to_string(),
                    Value::Node(start_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_OFFSET_KEY.to_string(),
                    Value::Number(start_offset),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_CONTAINER_KEY.to_string(),
                    Value::Node(end_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_OFFSET_KEY.to_string(),
                    Value::Number(end_offset),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startContainer".to_string(),
                    Value::Node(start_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startOffset".to_string(),
                    Value::Number(start_offset),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endContainer".to_string(),
                    Value::Node(end_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endOffset".to_string(),
                    Value::Number(end_offset),
                );
            }
            {
                let mut entries = selection.borrow_mut();
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_SELECTION_RANGE_KEY.to_string(),
                    Value::Object(range_object),
                );
                Self::object_set_entry(
                    &mut entries,
                    "anchorNode".to_string(),
                    Value::Node(anchor_node),
                );
                Self::object_set_entry(
                    &mut entries,
                    "anchorOffset".to_string(),
                    Value::Number(anchor_offset),
                );
                Self::object_set_entry(
                    &mut entries,
                    "focusNode".to_string(),
                    Value::Node(focus_node),
                );
                Self::object_set_entry(
                    &mut entries,
                    "focusOffset".to_string(),
                    Value::Number(focus_offset),
                );
                Self::object_set_entry(
                    &mut entries,
                    "isCollapsed".to_string(),
                    Value::Bool(ordering == std::cmp::Ordering::Equal),
                );
                Self::object_set_entry(&mut entries, "rangeCount".to_string(), Value::Number(1));
                Self::object_set_entry(
                    &mut entries,
                    "type".to_string(),
                    Value::String(if ordering == std::cmp::Ordering::Equal {
                        "Caret".to_string()
                    } else {
                        "Range".to_string()
                    }),
                );
                Self::object_set_entry(
                    &mut entries,
                    "direction".to_string(),
                    Value::String(direction.to_string()),
                );
            }
        } else {
            let range_object = self.selection_internal_range_object(&selection);
            {
                let mut range_entries = range_object.borrow_mut();
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_CONTAINER_KEY.to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_OFFSET_KEY.to_string(),
                    Value::Number(0),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_CONTAINER_KEY.to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_OFFSET_KEY.to_string(),
                    Value::Number(0),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startContainer".to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startOffset".to_string(),
                    Value::Number(0),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endContainer".to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endOffset".to_string(),
                    Value::Number(0),
                );
            }
            {
                let mut entries = selection.borrow_mut();
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_SELECTION_RANGE_KEY.to_string(),
                    Value::Object(range_object),
                );
                Self::object_set_entry(&mut entries, "anchorNode".to_string(), Value::Null);
                Self::object_set_entry(&mut entries, "anchorOffset".to_string(), Value::Number(0));
                Self::object_set_entry(&mut entries, "focusNode".to_string(), Value::Null);
                Self::object_set_entry(&mut entries, "focusOffset".to_string(), Value::Number(0));
                Self::object_set_entry(&mut entries, "isCollapsed".to_string(), Value::Bool(true));
                Self::object_set_entry(&mut entries, "rangeCount".to_string(), Value::Number(0));
                Self::object_set_entry(
                    &mut entries,
                    "type".to_string(),
                    Value::String("None".to_string()),
                );
                Self::object_set_entry(
                    &mut entries,
                    "direction".to_string(),
                    Value::String("none".to_string()),
                );
            }
        }

        let after = {
            let entries = selection.borrow();
            (
                Self::object_get_entry(&entries, "anchorNode"),
                Self::object_get_entry(&entries, "anchorOffset"),
                Self::object_get_entry(&entries, "focusNode"),
                Self::object_get_entry(&entries, "focusOffset"),
                Self::object_get_entry(&entries, "rangeCount"),
                Self::object_get_entry(&entries, "direction"),
            )
        };
        before != after
    }

    pub(crate) fn eval_selection_member_call(
        &mut self,
        selection_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let (is_selection, shadowed) = {
            let entries = selection_object.borrow();
            (
                Self::is_selection_object(&entries),
                Self::placeholder_backed_object_builtin_is_shadowed(&entries, member),
            )
        };
        if !is_selection {
            return Ok(None);
        }
        if shadowed {
            return Ok(None);
        }

        match member {
            "addRange" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "addRange requires exactly one range argument".into(),
                    ));
                }
                let range = match evaluated_args.first() {
                    Some(Value::Object(object)) => object.clone(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "addRange argument must be a Range".into(),
                        ));
                    }
                };
                let (start_container, start_offset, end_container, end_offset) = {
                    let range_entries = range.borrow();
                    if !Self::is_range_object(&range_entries) {
                        return Err(Error::ScriptRuntime(
                            "addRange argument must be a Range".into(),
                        ));
                    }
                    let start_container = match Self::object_get_entry(
                        &range_entries,
                        INTERNAL_RANGE_START_CONTAINER_KEY,
                    ) {
                        Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "Range boundary container must be a Node".into(),
                            ));
                        }
                    };
                    let start_offset = match Self::object_get_entry(
                        &range_entries,
                        INTERNAL_RANGE_START_OFFSET_KEY,
                    ) {
                        Some(Value::Number(offset)) => offset,
                        _ => 0,
                    };
                    let end_container = match Self::object_get_entry(
                        &range_entries,
                        INTERNAL_RANGE_END_CONTAINER_KEY,
                    ) {
                        Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "Range boundary container must be a Node".into(),
                            ));
                        }
                    };
                    let end_offset =
                        match Self::object_get_entry(&range_entries, INTERNAL_RANGE_END_OFFSET_KEY)
                        {
                            Some(Value::Number(offset)) => offset,
                            _ => 0,
                        };
                    (start_container, start_offset, end_container, end_offset)
                };
                {
                    let mut selection_entries = selection_object.borrow_mut();
                    Self::object_set_entry(
                        &mut selection_entries,
                        INTERNAL_SELECTION_RANGE_KEY.to_string(),
                        Value::Object(range.clone()),
                    );
                }
                let changed = self.selection_set_state(
                    Some(start_container),
                    start_offset,
                    Some(end_container),
                    end_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "collapse" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "collapse requires one or two arguments".into(),
                    ));
                }
                let changed = match evaluated_args.first() {
                    Some(Value::Null) => self.selection_set_state(None, 0, None, 0),
                    Some(boundary) => {
                        let node = self.selection_boundary_node_from_value(boundary)?;
                        let offset = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                        if offset < 0 {
                            return Err(Error::ScriptRuntime(
                                "IndexSizeError: offset must be non-negative".into(),
                            ));
                        }
                        self.selection_set_state(Some(node), offset, Some(node), offset)
                    }
                    None => false,
                };
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "collapseToStart" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "collapseToStart takes no arguments".into(),
                    ));
                }
                let Some((start_container, start_offset, _, _)) =
                    self.selection_normalized_boundaries(selection_object)
                else {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Selection has no range".into(),
                    ));
                };
                let changed = self.selection_set_state(
                    Some(start_container),
                    start_offset,
                    Some(start_container),
                    start_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "collapseToEnd" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "collapseToEnd takes no arguments".into(),
                    ));
                }
                let Some((_, _, end_container, end_offset)) =
                    self.selection_normalized_boundaries(selection_object)
                else {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Selection has no range".into(),
                    ));
                };
                let changed = self.selection_set_state(
                    Some(end_container),
                    end_offset,
                    Some(end_container),
                    end_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "containsNode" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "containsNode requires one or two arguments".into(),
                    ));
                }
                let node = match evaluated_args.first() {
                    Some(Value::Node(node)) if self.dom.is_valid_node(*node) => *node,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "containsNode first argument must be a Node".into(),
                        ));
                    }
                };
                let allow_partial = evaluated_args
                    .get(1)
                    .map(|value| value.truthy())
                    .unwrap_or(false);
                let Some((start_container, start_offset, end_container, end_offset)) =
                    self.selection_normalized_boundaries(selection_object)
                else {
                    return Ok(Some(Value::Bool(false)));
                };
                let selection_start = self
                    .selection_boundary_char_index(start_container, start_offset)
                    .unwrap_or(0);
                let selection_end = self
                    .selection_boundary_char_index(end_container, end_offset)
                    .unwrap_or(selection_start);
                let Some((node_start, node_end)) = self.selection_node_boundary_char_indexes(node)
                else {
                    return Ok(Some(Value::Bool(false)));
                };
                let contains = if allow_partial {
                    node_end > selection_start && node_start < selection_end
                } else {
                    node_start >= selection_start && node_end <= selection_end
                };
                Ok(Some(Value::Bool(contains)))
            }
            "deleteFromDocument" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "deleteFromDocument takes no arguments".into(),
                    ));
                }
                let Some((start_container, start_offset, end_container, end_offset)) =
                    self.selection_normalized_boundaries(selection_object)
                else {
                    return Ok(Some(Value::Undefined));
                };
                if self.selection_compare_points(
                    start_container,
                    start_offset,
                    end_container,
                    end_offset,
                ) == std::cmp::Ordering::Equal
                {
                    return Ok(Some(Value::Undefined));
                }

                if start_container == end_container {
                    let start =
                        self.selection_clamped_offset(start_container, start_offset) as usize;
                    let end = self.selection_clamped_offset(end_container, end_offset) as usize;
                    match &mut self.dom.nodes[start_container.0].node_type {
                        NodeType::Text(text) => {
                            let start_byte = Self::char_index_to_byte(text, start);
                            let end_byte = Self::char_index_to_byte(text, end);
                            text.replace_range(start_byte..end_byte, "");
                        }
                        NodeType::Document | NodeType::Element(_) => {
                            if end > start {
                                let targets =
                                    self.dom.nodes[start_container.0].children[start..end].to_vec();
                                for child in targets {
                                    let _ = self.dom.remove_child(start_container, child);
                                }
                            }
                        }
                    }
                } else {
                    let full = self.dom.text_content(self.dom.root);
                    let start = self
                        .selection_boundary_char_index(start_container, start_offset)
                        .unwrap_or(0);
                    let end = self
                        .selection_boundary_char_index(end_container, end_offset)
                        .unwrap_or(start);
                    let start = start.min(full.chars().count());
                    let end = end.min(full.chars().count()).max(start);
                    let mut next = String::new();
                    next.push_str(&Self::selection_slice_text_by_char_index(&full, 0, start));
                    next.push_str(&Self::selection_slice_text_by_char_index(
                        &full,
                        end,
                        full.chars().count(),
                    ));
                    if let Some(body) = self.dom.body().or_else(|| self.dom.document_element()) {
                        let _ = self.dom.set_text_content(body, &next);
                    }
                }

                let changed = self.selection_set_state(
                    Some(start_container),
                    start_offset,
                    Some(start_container),
                    start_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "empty" | "removeAllRanges" => {
                if !evaluated_args.is_empty() {
                    let method = if member == "empty" {
                        "empty"
                    } else {
                        "removeAllRanges"
                    };
                    return Err(Error::ScriptRuntime(format!("{method} takes no arguments")));
                }
                let changed = self.selection_set_state(None, 0, None, 0);
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "extend" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "extend requires exactly two arguments".into(),
                    ));
                }
                let Some((anchor_node, anchor_offset, _, _)) =
                    self.selection_anchor_focus(selection_object)
                else {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Selection has no range".into(),
                    ));
                };
                let focus_node = self.selection_boundary_node_from_value(&evaluated_args[0])?;
                let focus_offset = Self::value_to_i64(&evaluated_args[1]);
                if focus_offset < 0 {
                    return Err(Error::ScriptRuntime(
                        "IndexSizeError: offset must be non-negative".into(),
                    ));
                }
                let changed = self.selection_set_state(
                    Some(anchor_node),
                    anchor_offset,
                    Some(focus_node),
                    focus_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "getComposedRanges" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "getComposedRanges supports at most one argument".into(),
                    ));
                }
                let Some((start_container, start_offset, end_container, end_offset)) =
                    self.selection_normalized_boundaries(selection_object)
                else {
                    return Ok(Some(Self::new_array_value(Vec::new())));
                };
                let range = Self::new_object_value(vec![
                    ("startContainer".to_string(), Value::Node(start_container)),
                    ("startOffset".to_string(), Value::Number(start_offset)),
                    ("endContainer".to_string(), Value::Node(end_container)),
                    ("endOffset".to_string(), Value::Number(end_offset)),
                ]);
                Ok(Some(Self::new_array_value(vec![range])))
            }
            "getRangeAt" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getRangeAt requires exactly one index argument".into(),
                    ));
                }
                let index = Self::value_to_i64(&evaluated_args[0]);
                let has_range = self
                    .selection_normalized_boundaries(selection_object)
                    .is_some();
                if index != 0 || !has_range {
                    return Err(Error::ScriptRuntime(
                        "IndexSizeError: Invalid range index".into(),
                    ));
                }
                let range = self.selection_internal_range_object(selection_object);
                Ok(Some(Value::Object(range)))
            }
            "modify" => {
                if evaluated_args.len() != 3 {
                    return Err(Error::ScriptRuntime(
                        "modify requires exactly three arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "removeRange" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeRange requires exactly one range argument".into(),
                    ));
                }
                let candidate = match evaluated_args.first() {
                    Some(Value::Object(object)) => object.clone(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "removeRange argument must be a Range".into(),
                        ));
                    }
                };
                let current = self.selection_internal_range_object(selection_object);
                if !Rc::ptr_eq(&candidate, &current) {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeRange': The given range isn't in selection"
                            .into(),
                    ));
                }
                let changed = self.selection_set_state(None, 0, None, 0);
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "selectAllChildren" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "selectAllChildren requires exactly one argument".into(),
                    ));
                }
                let node = self.selection_boundary_node_from_value(&evaluated_args[0])?;
                let end = match &self.dom.nodes[node.0].node_type {
                    NodeType::Text(text) => text.chars().count() as i64,
                    NodeType::Document | NodeType::Element(_) => {
                        self.dom.nodes[node.0].children.len() as i64
                    }
                };
                let changed = self.selection_set_state(Some(node), 0, Some(node), end);
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "setBaseAndExtent" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(
                        "setBaseAndExtent requires exactly four arguments".into(),
                    ));
                }
                let anchor_node = self.selection_boundary_node_from_value(&evaluated_args[0])?;
                let anchor_offset = Self::value_to_i64(&evaluated_args[1]);
                let focus_node = self.selection_boundary_node_from_value(&evaluated_args[2])?;
                let focus_offset = Self::value_to_i64(&evaluated_args[3]);
                if anchor_offset < 0 || focus_offset < 0 {
                    return Err(Error::ScriptRuntime(
                        "IndexSizeError: offset must be non-negative".into(),
                    ));
                }
                let changed = self.selection_set_state(
                    Some(anchor_node),
                    anchor_offset,
                    Some(focus_node),
                    focus_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "setPosition" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "setPosition requires one or two arguments".into(),
                    ));
                }
                let changed = match evaluated_args.first() {
                    Some(Value::Null) => self.selection_set_state(None, 0, None, 0),
                    Some(boundary) => {
                        let node = self.selection_boundary_node_from_value(boundary)?;
                        let offset = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                        if offset < 0 {
                            return Err(Error::ScriptRuntime(
                                "IndexSizeError: offset must be non-negative".into(),
                            ));
                        }
                        self.selection_set_state(Some(node), offset, Some(node), offset)
                    }
                    None => false,
                };
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "toString" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("toString takes no arguments".into()));
                }
                Ok(Some(Value::String(self.selection_text(selection_object))))
            }
            _ => Ok(None),
        }
    }
}
