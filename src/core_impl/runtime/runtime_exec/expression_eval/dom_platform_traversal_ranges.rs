use super::*;

impl Harness {
    fn tree_walker_mask_for_node(&self, node: NodeId) -> u32 {
        match self.node_type_number(node) {
            1 => 0x1,
            3 => 0x4,
            8 => 0x80,
            9 => 0x100,
            11 => 0x400,
            _ => 0,
        }
    }

    fn tree_walker_accepts(what_to_show: i64, node_mask: u32) -> bool {
        if what_to_show == -1 || what_to_show == 4_294_967_295 {
            return true;
        }
        ((what_to_show as u32) & node_mask) != 0
    }

    fn collect_tree_walker_traversal(&self, root: NodeId, out: &mut Vec<NodeId>) {
        out.push(root);
        for child in &self.dom.nodes[root.0].children {
            self.collect_tree_walker_traversal(*child, out);
        }
    }

    fn tree_walker_current_node_from_entries(&self, entries: &[(String, Value)]) -> Value {
        let traversal =
            match Self::object_get_entry(entries, INTERNAL_TREE_WALKER_TRAVERSAL_NODES_KEY) {
                Some(Value::Array(nodes)) => nodes,
                _ => return Value::Null,
            };
        let nodes = traversal.borrow();
        let index = match Self::object_get_entry(entries, INTERNAL_TREE_WALKER_INDEX_KEY) {
            Some(Value::Number(index)) if index >= 0 => index as usize,
            _ => 0,
        };
        nodes.get(index).cloned().unwrap_or(Value::Null)
    }

    pub(crate) fn tree_walker_property_from_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Result<Option<Value>> {
        if !matches!(
            Self::object_get_entry(entries, INTERNAL_TREE_WALKER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            return Ok(None);
        }
        Ok(match key {
            "currentNode" => Some(self.tree_walker_current_node_from_entries(entries)),
            "nextNode" => Self::placeholder_backed_object_builtin_property_value(entries, key),
            "root" => {
                let traversal =
                    Self::object_get_entry(entries, INTERNAL_TREE_WALKER_TRAVERSAL_NODES_KEY);
                match traversal {
                    Some(Value::Array(nodes)) => nodes.borrow().first().cloned(),
                    _ => Some(Value::Null),
                }
            }
            "whatToShow" => Self::object_get_entry(entries, INTERNAL_TREE_WALKER_WHAT_TO_SHOW_KEY),
            _ => None,
        })
    }
    pub(crate) fn eval_tree_walker_member_call(
        &mut self,
        walker_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let (is_tree_walker, shadowed) = {
            let entries = walker_object.borrow();
            (
                matches!(
                    Self::object_get_entry(&entries, INTERNAL_TREE_WALKER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ),
                Self::placeholder_backed_object_builtin_is_shadowed(&entries, member),
            )
        };
        if !is_tree_walker {
            return Ok(None);
        }
        if shadowed {
            return Ok(None);
        }

        match member {
            "nextNode" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("nextNode takes no arguments".into()));
                }
                let (traversal, current_index, what_to_show) = {
                    let entries = walker_object.borrow();
                    let traversal = match Self::object_get_entry(
                        &entries,
                        INTERNAL_TREE_WALKER_TRAVERSAL_NODES_KEY,
                    ) {
                        Some(Value::Array(nodes)) => nodes,
                        _ => return Ok(Some(Value::Null)),
                    };
                    let current_index =
                        match Self::object_get_entry(&entries, INTERNAL_TREE_WALKER_INDEX_KEY) {
                            Some(Value::Number(index)) if index >= 0 => index as usize,
                            _ => 0,
                        };
                    let what_to_show = match Self::object_get_entry(
                        &entries,
                        INTERNAL_TREE_WALKER_WHAT_TO_SHOW_KEY,
                    ) {
                        Some(Value::Number(mask)) => mask,
                        _ => 4_294_967_295,
                    };
                    (traversal, current_index, what_to_show)
                };

                let nodes = traversal.borrow();
                for index in (current_index + 1)..nodes.len() {
                    let Value::Node(node) = &nodes[index] else {
                        continue;
                    };
                    if !Self::tree_walker_accepts(
                        what_to_show,
                        self.tree_walker_mask_for_node(*node),
                    ) {
                        continue;
                    }
                    Self::object_set_entry(
                        &mut walker_object.borrow_mut(),
                        INTERNAL_TREE_WALKER_INDEX_KEY.to_string(),
                        Value::Number(index as i64),
                    );
                    return Ok(Some(Value::Node(*node)));
                }
                Ok(Some(Value::Null))
            }
            _ => Ok(None),
        }
    }

    fn range_boundary_node_from_value(&self, value: &Value) -> Result<NodeId> {
        match value {
            Value::Node(node) if self.dom.is_valid_node(*node) => Ok(*node),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    Ok(self.dom.root)
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_PARSED_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    Self::parsed_document_root_from_entries(&entries).ok_or_else(|| {
                        Error::ScriptRuntime("Range boundary container must be a Node".into())
                    })
                } else {
                    Err(Error::ScriptRuntime(
                        "Range boundary container must be a Node".into(),
                    ))
                }
            }
            _ => Err(Error::ScriptRuntime(
                "Range boundary container must be a Node".into(),
            )),
        }
    }

    pub(crate) fn eval_range_member_call(
        &mut self,
        range_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let (is_range, shadowed) = {
            let entries = range_object.borrow();
            (
                Self::is_range_object(&entries),
                Self::placeholder_backed_object_builtin_is_shadowed(&entries, member),
            )
        };
        if !is_range {
            return Ok(None);
        }
        if shadowed {
            return Ok(None);
        }

        match member {
            "setStart" | "setEnd" => {
                if evaluated_args.len() != 2 {
                    let message = if member == "setStart" {
                        "setStart requires exactly two arguments"
                    } else {
                        "setEnd requires exactly two arguments"
                    };
                    return Err(Error::ScriptRuntime(message.into()));
                }

                let container = self.range_boundary_node_from_value(&evaluated_args[0])?;
                let offset = Self::value_to_i64(&evaluated_args[1]);
                if offset < 0 {
                    return Err(Error::ScriptRuntime(
                        "IndexSizeError: offset must be non-negative".into(),
                    ));
                }

                let (internal_container_key, internal_offset_key, container_key, offset_key) =
                    if member == "setStart" {
                        (
                            INTERNAL_RANGE_START_CONTAINER_KEY,
                            INTERNAL_RANGE_START_OFFSET_KEY,
                            "startContainer",
                            "startOffset",
                        )
                    } else {
                        (
                            INTERNAL_RANGE_END_CONTAINER_KEY,
                            INTERNAL_RANGE_END_OFFSET_KEY,
                            "endContainer",
                            "endOffset",
                        )
                    };
                let mut entries = range_object.borrow_mut();
                Self::object_set_entry(
                    &mut entries,
                    internal_container_key.to_string(),
                    Value::Node(container),
                );
                Self::object_set_entry(
                    &mut entries,
                    internal_offset_key.to_string(),
                    Value::Number(offset),
                );
                Self::object_set_entry(
                    &mut entries,
                    container_key.to_string(),
                    Value::Node(container),
                );
                Self::object_set_entry(&mut entries, offset_key.to_string(), Value::Number(offset));

                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_create_tree_walker_call(
        &mut self,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        if evaluated_args.is_empty() {
            return Err(Error::ScriptRuntime(
                "createTreeWalker requires at least one root argument".into(),
            ));
        }

        let root = match &evaluated_args[0] {
            Value::Node(node) => *node,
            Value::Object(entries) => {
                let entries = entries.borrow();
                if matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    self.dom.root
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_PARSED_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    Self::parsed_document_root_from_entries(&entries).unwrap_or(self.dom.root)
                } else {
                    return Err(Error::ScriptRuntime(
                        "createTreeWalker root must be a Node".into(),
                    ));
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "createTreeWalker root must be a Node".into(),
                ));
            }
        };

        let what_to_show = evaluated_args
            .get(1)
            .map(Self::value_to_i64)
            .unwrap_or(4_294_967_295);

        let mut traversal = Vec::new();
        self.collect_tree_walker_traversal(root, &mut traversal);
        let traversal_values = traversal.into_iter().map(Value::Node).collect::<Vec<_>>();

        Ok(Some(Self::new_object_value(vec![
            (
                INTERNAL_TREE_WALKER_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_TREE_WALKER_TRAVERSAL_NODES_KEY.to_string(),
                Self::new_array_value(traversal_values),
            ),
            (INTERNAL_TREE_WALKER_INDEX_KEY.to_string(), Value::Number(0)),
            (
                INTERNAL_TREE_WALKER_WHAT_TO_SHOW_KEY.to_string(),
                Value::Number(what_to_show),
            ),
        ])))
    }
}
