use super::*;

impl Harness {
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
