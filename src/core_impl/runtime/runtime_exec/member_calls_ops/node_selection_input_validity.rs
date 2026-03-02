use super::*;

impl Harness {
    fn hierarchy_request_error() -> Error {
        Error::ScriptRuntime(
            "HierarchyRequestError: The operation would yield an incorrect node tree.".into(),
        )
    }

    fn is_document_fragment_node(&self, node: NodeId) -> bool {
        self.dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("#document-fragment"))
    }

    fn collect_appendable_document_nodes(&self, node: NodeId, out: &mut Vec<NodeId>) {
        if self.is_document_fragment_node(node) {
            let children = self.dom.nodes[node.0].children.clone();
            for child in children {
                self.collect_appendable_document_nodes(child, out);
            }
            return;
        }
        out.push(node);
    }

    pub(crate) fn eval_document_append_call(
        &mut self,
        document_node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if matches!(
            self.dom
                .nodes
                .get(document_node.0)
                .map(|node| &node.node_type),
            Some(NodeType::Document)
        ) {
            let mut nodes = Vec::new();
            for value in evaluated_args {
                match value {
                    Value::Node(node) => self.collect_appendable_document_nodes(*node, &mut nodes),
                    other => {
                        let text = self.dom.create_detached_text(other.as_string());
                        nodes.push(text);
                    }
                }
            }

            let mut existing_elements = self.dom.nodes[document_node.0]
                .children
                .iter()
                .copied()
                .filter(|child| {
                    self.dom.element(*child).is_some_and(|element| {
                        !element.tag_name.eq_ignore_ascii_case("#document-fragment")
                    })
                })
                .count() as i64;

            for node in &nodes {
                if self.dom.parent(*node) == Some(document_node)
                    && self.dom.element(*node).is_some_and(|element| {
                        !element.tag_name.eq_ignore_ascii_case("#document-fragment")
                    })
                {
                    existing_elements -= 1;
                }
            }

            let mut appended_elements = 0i64;
            for node in &nodes {
                match self.dom.nodes.get(node.0).map(|entry| &entry.node_type) {
                    Some(NodeType::Document) | Some(NodeType::Text(_)) => {
                        return Err(Self::hierarchy_request_error());
                    }
                    Some(NodeType::Element(element))
                        if !element.tag_name.eq_ignore_ascii_case("#document-fragment") =>
                    {
                        appended_elements += 1;
                    }
                    Some(NodeType::Element(_)) => {}
                    None => return Err(Self::hierarchy_request_error()),
                }
            }

            if existing_elements + appended_elements > 1 {
                return Err(Self::hierarchy_request_error());
            }

            for node in nodes {
                self.dom.append_child(document_node, node)?;
            }
            return Ok(Value::Undefined);
        }

        for value in evaluated_args {
            let node = match value {
                Value::Node(node) => *node,
                other => self.dom.create_detached_text(other.as_string()),
            };
            self.dom.append_child(document_node, node)?;
        }
        Ok(Value::Undefined)
    }

    pub(crate) fn eval_document_member_call(
        &mut self,
        member: &str,
        evaluated_args: &[Value],
        _event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "getElementById" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementById requires exactly one argument".into(),
                    ));
                }
                let id = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom.by_id(&id).map(Value::Node).unwrap_or(Value::Null),
                ))
            }
            "getElementsByClassName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByClassName requires exactly one argument".into(),
                    ));
                }
                let class_names = Self::class_names_from_argument(&evaluated_args[0]);
                Ok(Some(
                    self.class_names_live_list_value(self.dom.root, class_names),
                ))
            }
            "getElementsByName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(self.name_live_list_value(
                    self.dom.root,
                    evaluated_args[0].as_string(),
                )))
            }
            "getElementsByTagName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(self.tag_name_live_list_value(
                    self.dom.root,
                    Self::tag_name_from_argument(&evaluated_args[0]),
                )))
            }
            "createElement" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "createElement requires one or two arguments".into(),
                    ));
                }
                let tag_name = evaluated_args[0].as_string().to_ascii_lowercase();
                let node = self.dom.create_detached_element(tag_name);
                if let Some(is_value) =
                    Self::create_element_is_option_from_arg(evaluated_args.get(1))
                {
                    self.dom.set_attr(node, "is", &is_value)?;
                }
                Ok(Some(Value::Node(node)))
            }
            "createTextNode" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "createTextNode requires exactly one argument".into(),
                    ));
                }
                let text = evaluated_args[0].as_string();
                let node = self.dom.create_detached_text(text);
                Ok(Some(Value::Node(node)))
            }
            "createAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "createAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if !is_valid_create_attribute_name(&name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                Ok(Some(Self::new_attr_object_value(&name, "", None)))
            }
            "createDocumentFragment" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "createDocumentFragment takes no arguments".into(),
                    ));
                }
                let node = self
                    .dom
                    .create_detached_element("#document-fragment".to_string());
                Ok(Some(Value::Node(node)))
            }
            "createRange" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "createRange takes no arguments".into(),
                    ));
                }
                Ok(Some(Self::new_range_object_value(self.dom.root)))
            }
            "append" => Ok(Some(
                self.eval_document_append_call(self.dom.root, evaluated_args)?,
            )),
            "querySelector" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelector requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom
                        .query_selector(&selector)?
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "querySelectorAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelectorAll requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(Self::new_static_node_list_value(
                    self.dom.query_selector_all(&selector)?,
                )))
            }
            "createTreeWalker" => self.eval_create_tree_walker_call(evaluated_args),
            "addEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "addEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
                    Value::Function(function) => {
                        self.listeners.add(
                            self.dom.root,
                            event_type,
                            Listener {
                                capture,
                                handler: function.handler.clone(),
                                captured_env: function.captured_env.clone(),
                                captured_pending_function_decls: function
                                    .captured_pending_function_decls
                                    .clone(),
                            },
                        );
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "addEventListener callback must be a function".into(),
                    )),
                }
            }
            "removeEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "removeEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
                    Value::Function(function) => {
                        let _ = self.listeners.remove(
                            self.dom.root,
                            &event_type,
                            capture,
                            &function.handler,
                        );
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "removeEventListener callback must be a function".into(),
                    )),
                }
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn parse_listener_capture_arg(&self, value: Option<&Value>) -> Result<bool> {
        let Some(value) = value else {
            return Ok(false);
        };
        match value {
            Value::Bool(capture) => Ok(*capture),
            Value::Object(entries) => {
                let entries = entries.borrow();
                Ok(Self::object_get_entry(&entries, "capture")
                    .map(|capture| capture.truthy())
                    .unwrap_or(false))
            }
            _ => Err(Error::ScriptRuntime(
                "add/removeEventListener third argument must be true/false or options object"
                    .into(),
            )),
        }
    }

    pub(crate) fn eval_node_member_call(
        &mut self,
        node: NodeId,
        member: &str,
        evaluated_args: &[Value],
        _event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "addEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "addEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
                    Value::Function(function) => {
                        self.listeners.add(
                            node,
                            event_type,
                            Listener {
                                capture,
                                handler: function.handler.clone(),
                                captured_env: function.captured_env.clone(),
                                captured_pending_function_decls: function
                                    .captured_pending_function_decls
                                    .clone(),
                            },
                        );
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "addEventListener callback must be a function".into(),
                    )),
                }
            }
            "removeEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "removeEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
                    Value::Function(function) => {
                        let _ =
                            self.listeners
                                .remove(node, &event_type, capture, &function.handler);
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "removeEventListener callback must be a function".into(),
                    )),
                }
            }
            "getAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if name == "nonce" {
                    return Ok(Some(
                        if self.dom.attr(node, "nonce").is_some() {
                            Value::String(String::new())
                        } else {
                            Value::Null
                        },
                    ));
                }
                Ok(Some(
                    self.dom
                        .attr(node, &name)
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                ))
            }
            "setAttribute" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "setAttribute requires exactly two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if !is_valid_create_attribute_name(&name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                let value = evaluated_args[1].as_string();
                self.dom.set_attr(node, &name, &value)?;
                Ok(Some(Value::Undefined))
            }
            "setAttributeNode" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setAttributeNode requires exactly one argument".into(),
                    ));
                }
                let attr_object = match evaluated_args.first() {
                    Some(Value::Object(object)) => object.clone(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNode argument must be an Attr".into(),
                        ));
                    }
                };
                let (name, value): (String, String) = {
                    let entries = attr_object.borrow();
                    if !Self::is_attr_object(&entries) {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNode argument must be an Attr".into(),
                        ));
                    }
                    let name = Self::object_get_entry(&entries, "name")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    if !is_valid_create_attribute_name(&name) {
                        return Err(Error::ScriptRuntime(
                            "InvalidCharacterError: attribute name is not a valid XML name".into(),
                        ));
                    }
                    let value = Self::object_get_entry(&entries, "value")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default();
                    (name, value)
                };
                let replaced_value = self.dom.attr(node, &name);
                self.dom.set_attr(node, &name, &value)?;

                {
                    let mut entries = attr_object.borrow_mut();
                    Self::object_set_entry(
                        &mut entries,
                        "name".to_string(),
                        Value::String(name.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "value".to_string(),
                        Value::String(value.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "ownerElement".to_string(),
                        Value::Node(node),
                    );
                }

                Ok(Some(
                    replaced_value
                        .map(|old| Self::new_attr_object_value(&name, &old, None))
                        .unwrap_or(Value::Null),
                ))
            }
            "hasAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "hasAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                Ok(Some(Value::Bool(self.dom.has_attr(node, &name)?)))
            }
            "hasAttributes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "hasAttributes takes no arguments".into(),
                    ));
                }
                let has_attributes = self
                    .dom
                    .element(node)
                    .map(|element| !element.attrs.is_empty())
                    .ok_or_else(|| {
                        Error::ScriptRuntime("hasAttributes target is not an element".into())
                    })?;
                Ok(Some(Value::Bool(has_attributes)))
            }
            "removeAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                self.dom.remove_attr(node, &name)?;
                Ok(Some(Value::Undefined))
            }
            "getAttributeNames" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNames takes no arguments".into(),
                    ));
                }
                let element = self.dom.element(node).ok_or_else(|| {
                    Error::ScriptRuntime("getAttributeNames target is not an element".into())
                })?;
                let mut names = element.attrs.keys().cloned().collect::<Vec<_>>();
                names.sort();
                Ok(Some(Self::new_array_value(
                    names.into_iter().map(Value::String).collect(),
                )))
            }
            "toggleAttribute" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "toggleAttribute requires one or two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if !is_valid_create_attribute_name(&name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                let has = self.dom.has_attr(node, &name)?;
                let next = if evaluated_args.len() == 2 {
                    evaluated_args[1].truthy()
                } else {
                    !has
                };
                if next {
                    self.dom.set_attr(node, &name, "")?;
                } else {
                    self.dom.remove_attr(node, &name)?;
                }
                Ok(Some(Value::Bool(next)))
            }
            "matches" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "matches requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(Value::Bool(
                    self.dom.matches_selector(node, &selector)?,
                )))
            }
            "closest" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "closest requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom
                        .closest(node, &selector)?
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "querySelector" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelector requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom
                        .query_selector_from(&node, &selector)?
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "querySelectorAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelectorAll requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(Self::new_static_node_list_value(
                    self.dom.query_selector_all_from(&node, &selector)?,
                )))
            }
            "appendChild" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "appendChild requires exactly one node argument".into(),
                    ));
                }
                let child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "appendChild argument must be a Node".into(),
                        ));
                    }
                };
                self.dom.append_child(node, child)?;
                Ok(Some(Value::Node(child)))
            }
            "insertBefore" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "insertBefore requires exactly two arguments".into(),
                    ));
                }
                let child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "insertBefore first argument must be a Node".into(),
                        ));
                    }
                };
                match evaluated_args.get(1) {
                    Some(Value::Node(reference)) => {
                        self.dom.insert_before(node, child, *reference)?;
                    }
                    Some(Value::Null) | Some(Value::Undefined) => {
                        self.dom.append_child(node, child)?;
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "insertBefore second argument must be a Node or null".into(),
                        ));
                    }
                }
                Ok(Some(Value::Node(child)))
            }
            "removeChild" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeChild requires exactly one node argument".into(),
                    ));
                }
                let child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "removeChild argument must be a Node".into(),
                        ));
                    }
                };
                self.dom.remove_child(node, child)?;
                Ok(Some(Value::Node(child)))
            }
            "replaceChild" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "replaceChild requires exactly two node arguments".into(),
                    ));
                }
                let new_child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "replaceChild first argument must be a Node".into(),
                        ));
                    }
                };
                let old_child = match evaluated_args.get(1) {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "replaceChild second argument must be a Node".into(),
                        ));
                    }
                };
                self.dom.replace_child(node, new_child, old_child)?;
                Ok(Some(Value::Node(old_child)))
            }
            "hasChildNodes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "hasChildNodes takes no arguments".into(),
                    ));
                }
                Ok(Some(Value::Bool(
                    !self.dom.nodes[node.0].children.is_empty(),
                )))
            }
            "contains" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "contains requires exactly one argument".into(),
                    ));
                }
                let contains = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => false,
                    Some(Value::Node(other)) => {
                        *other == node || self.dom.is_descendant_of(*other, node)
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "contains argument must be a Node or null".into(),
                        ));
                    }
                };
                Ok(Some(Value::Bool(contains)))
            }
            "getRootNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "getRootNode supports at most one options argument".into(),
                    ));
                }
                Ok(Some(Value::Node(self.node_root(node))))
            }
            "compareDocumentPosition" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "compareDocumentPosition requires exactly one node argument".into(),
                    ));
                }
                let other = match evaluated_args.first() {
                    Some(Value::Node(other)) => *other,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "compareDocumentPosition argument must be a Node".into(),
                        ));
                    }
                };
                Ok(Some(Value::Number(
                    self.node_compare_document_position(node, other),
                )))
            }
            "isEqualNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "isEqualNode supports at most one argument".into(),
                    ));
                }
                let is_equal = match evaluated_args.first() {
                    None | Some(Value::Null) | Some(Value::Undefined) => false,
                    Some(Value::Node(other)) => self.nodes_are_equal(node, *other),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "isEqualNode argument must be a Node or null".into(),
                        ));
                    }
                };
                Ok(Some(Value::Bool(is_equal)))
            }
            "isSameNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "isSameNode supports at most one argument".into(),
                    ));
                }
                let is_same = match evaluated_args.first() {
                    None | Some(Value::Null) | Some(Value::Undefined) => false,
                    Some(Value::Node(other)) => node == *other,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "isSameNode argument must be a Node or null".into(),
                        ));
                    }
                };
                Ok(Some(Value::Bool(is_same)))
            }
            "normalize" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("normalize takes no arguments".into()));
                }
                self.normalize_node_subtree(node)?;
                Ok(Some(Value::Undefined))
            }
            "isDefaultNamespace" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "isDefaultNamespace requires exactly one namespace argument".into(),
                    ));
                }
                let namespace = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => None,
                    Some(value) => Some(value.as_string()),
                    None => None,
                };
                Ok(Some(Value::Bool(
                    self.node_is_default_namespace(node, namespace.as_deref()),
                )))
            }
            "lookupPrefix" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "lookupPrefix requires exactly one namespace argument".into(),
                    ));
                }
                let namespace = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => None,
                    Some(value) => Some(value.as_string()),
                    None => None,
                };
                Ok(Some(
                    self.node_lookup_prefix(node, namespace.as_deref())
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                ))
            }
            "lookupNamespaceURI" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "lookupNamespaceURI requires exactly one prefix argument".into(),
                    ));
                }
                let prefix = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => None,
                    Some(value) => Some(value.as_string()),
                    None => None,
                };
                Ok(Some(
                    self.node_lookup_namespace_uri(node, prefix.as_deref())
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                ))
            }
            "cloneNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "cloneNode supports at most one argument".into(),
                    ));
                }
                let deep = evaluated_args.first().is_some_and(Value::truthy);
                let cloned = self.clone_dom_node(node, deep)?;
                Ok(Some(Value::Node(cloned)))
            }
            "remove" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("remove takes no arguments".into()));
                }
                if let Some(active) = self.dom.active_element() {
                    if active == node || self.dom.is_descendant_of(active, node) {
                        self.dom.set_active_element(None);
                    }
                }
                if let Some(active_pseudo) = self.dom.active_pseudo_element() {
                    if active_pseudo == node || self.dom.is_descendant_of(active_pseudo, node) {
                        self.dom.set_active_pseudo_element(None);
                    }
                }
                self.dom.remove_node(node)?;
                Ok(Some(Value::Undefined))
            }
            "getContext" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "getContext requires one or two arguments".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                let context_kind = evaluated_args[0].as_string().to_ascii_lowercase();
                if context_kind != "2d" {
                    return Ok(Some(Value::Null));
                }
                let key = INTERNAL_CANVAS_2D_CONTEXT_NODE_EXPANDO_KEY.to_string();
                if let Some(existing) = self
                    .dom_runtime
                    .node_expando_props
                    .get(&(node, key.clone()))
                {
                    return Ok(Some(existing.clone()));
                }
                let alpha = evaluated_args
                    .get(1)
                    .map(Self::canvas_2d_alpha_from_options)
                    .unwrap_or(true);
                let context = self.new_canvas_2d_context_value(alpha);
                self.dom_runtime
                    .node_expando_props
                    .insert((node, key), context.clone());
                Ok(Some(context))
            }
            "toDataURL" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "toDataURL supports at most two arguments".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                let mime = evaluated_args
                    .first()
                    .map(Value::as_string)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "image/png".to_string());
                let mime = if mime.eq_ignore_ascii_case("image/png")
                    || mime.eq_ignore_ascii_case("image/jpeg")
                    || mime.eq_ignore_ascii_case("image/webp")
                {
                    mime.to_ascii_lowercase()
                } else {
                    "image/png".to_string()
                };
                let payload = match mime.as_str() {
                    "image/jpeg" => "/9j/4AAQSkZJRgABAQAAAQABAAD/2w==",
                    "image/webp" => "UklGRhIAAABXRUJQVlA4TA0AAAAvAAAAAA==",
                    _ => {
                        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR4nGNgYAAAAAMAASsJTYQAAAAASUVORK5CYII="
                    }
                };
                Ok(Some(Value::String(format!("data:{mime};base64,{payload}"))))
            }
            "getElementsByClassName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByClassName requires exactly one argument".into(),
                    ));
                }
                let class_names = Self::class_names_from_argument(&evaluated_args[0]);
                Ok(Some(self.class_names_live_list_value(node, class_names)))
            }
            "getElementsByTagName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(self.tag_name_live_list_value(
                    node,
                    Self::tag_name_from_argument(&evaluated_args[0]),
                )))
            }
            "checkVisibility" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "checkVisibility supports at most one argument".into(),
                    ));
                }
                Ok(Some(Value::Bool(!self.dom.has_attr(node, "hidden")?)))
            }
            "checkValidity" | "reportValidity" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(format!("{member} takes no arguments")));
                }
                let validity = self.compute_input_validity(node)?;
                if !validity.valid {
                    let _ = self.dispatch_event(node, "invalid")?;
                }
                Ok(Some(Value::Bool(validity.valid)))
            }
            "setCustomValidity" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setCustomValidity requires exactly one argument".into(),
                    ));
                }
                self.dom
                    .set_custom_validity_message(node, &evaluated_args[0].as_string())?;
                Ok(Some(Value::Undefined))
            }
            "setSelectionRange" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "setSelectionRange requires two or three arguments".into(),
                    ));
                }
                self.set_node_selection_range(
                    node,
                    Self::value_to_i64(&evaluated_args[0]),
                    Self::value_to_i64(&evaluated_args[1]),
                    evaluated_args
                        .get(2)
                        .map(Value::as_string)
                        .unwrap_or_else(|| "none".to_string()),
                )?;
                Ok(Some(Value::Undefined))
            }
            "setRangeText" => {
                if !(evaluated_args.len() == 1
                    || evaluated_args.len() == 3
                    || evaluated_args.len() == 4)
                {
                    return Err(Error::ScriptRuntime(
                        "setRangeText supports one, three, or four arguments".into(),
                    ));
                }
                self.set_node_range_text(node, evaluated_args)?;
                Ok(Some(Value::Undefined))
            }
            "showPicker" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("showPicker takes no arguments".into()));
                }
                Ok(Some(Value::Undefined))
            }
            "stepUp" | "stepDown" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} supports at most one argument"
                    )));
                }
                let count = evaluated_args.first().map(Self::value_to_i64).unwrap_or(1);
                let direction = if member == "stepDown" { -1 } else { 1 };
                self.step_input_value(node, direction, count)?;
                Ok(Some(Value::Undefined))
            }
            "animate" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "animate requires one or two arguments".into(),
                    ));
                }
                let options_arg = evaluated_args.get(1);
                let id = Self::animate_id_from_options(options_arg);
                let timeline = Self::animate_option_entry(options_arg, "timeline")
                    .unwrap_or(Value::Null);
                let range_start = Self::animate_option_entry(options_arg, "rangeStart")
                    .unwrap_or(Value::String("normal".to_string()));
                let range_end = Self::animate_option_entry(options_arg, "rangeEnd")
                    .unwrap_or(Value::String("normal".to_string()));
                let keyframes = evaluated_args[0].clone();
                let options = options_arg.cloned().unwrap_or(Value::Undefined);
                Ok(Some(Self::new_animation_object_value(
                    id,
                    keyframes,
                    options,
                    timeline,
                    range_start,
                    range_end,
                )))
            }
            "scrollIntoView" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "scrollIntoView supports zero or one argument".into(),
                    ));
                }
                self.dispatch_document_scroll_sequence(true)?;
                Ok(Some(Value::Undefined))
            }
            "scroll" | "scrollTo" | "scrollBy" => {
                if !(evaluated_args.is_empty()
                    || evaluated_args.len() == 1
                    || evaluated_args.len() == 2)
                {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} supports zero, one, or two arguments"
                    )));
                }
                let position_changed =
                    self.apply_document_scroll_operation(member, evaluated_args);
                self.dispatch_document_scroll_sequence(position_changed)?;
                Ok(Some(Value::Undefined))
            }
            "select" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("select takes no arguments".into()));
                }
                if self.node_supports_text_selection(node) {
                    let len = self.dom.value(node)?.chars().count();
                    self.set_node_selection_range(node, 0, len as i64, "none".to_string())?;
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn canvas_2d_alpha_from_options(options: &Value) -> bool {
        match options {
            Value::Object(entries) => {
                let entries = entries.borrow();
                Self::object_get_entry(&entries, "alpha")
                    .map(|value| value.truthy())
                    .unwrap_or(true)
            }
            _ => true,
        }
    }

    pub(crate) fn eval_canvas_2d_context_member_call(
        &mut self,
        context_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let context = context_object.borrow_mut();
        let fill_style = Self::object_get_entry(&context, "fillStyle")
            .map(|value| value.as_string())
            .unwrap_or_else(|| "#000000".to_string());
        let stroke_style = Self::object_get_entry(&context, "strokeStyle")
            .map(|value| value.as_string())
            .unwrap_or_else(|| "#000000".to_string());
        match member {
            "fillRect" | "clearRect" | "strokeRect" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} requires exactly four arguments"
                    )));
                }
                let _ = &fill_style;
                let _ = &stroke_style;
                Ok(Some(Value::Undefined))
            }
            "beginPath" | "closePath" | "fill" | "stroke" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(format!("{member} takes no arguments")));
                }
                Ok(Some(Value::Undefined))
            }
            "moveTo" | "lineTo" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} requires exactly two arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            "arc" => {
                if evaluated_args.len() < 5 || evaluated_args.len() > 6 {
                    return Err(Error::ScriptRuntime(
                        "arc requires five or six arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "getContextAttributes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getContextAttributes takes no arguments".into(),
                    ));
                }
                let alpha = Self::object_get_entry(&context, INTERNAL_CANVAS_2D_ALPHA_KEY)
                    .map(|value| value.truthy())
                    .unwrap_or(true);
                Ok(Some(Self::new_object_value(vec![(
                    "alpha".to_string(),
                    Value::Bool(alpha),
                )])))
            }
            "toString" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("toString takes no arguments".into()));
                }
                Ok(Some(Value::String(
                    "[object CanvasRenderingContext2D]".to_string(),
                )))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn normalized_input_type(&self, node: NodeId) -> String {
        if !self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false)
        {
            return String::new();
        }
        let raw = self
            .dom
            .attr(node, "type")
            .unwrap_or_default()
            .to_ascii_lowercase();
        match raw.as_str() {
            "button" | "checkbox" | "color" | "date" | "datetime-local" | "email" | "file"
            | "hidden" | "image" | "month" | "number" | "password" | "radio" | "range"
            | "reset" | "search" | "submit" | "tel" | "text" | "time" | "url" | "week" => raw,
            _ => "text".to_string(),
        }
    }

    pub(crate) fn node_supports_text_selection(&self, node: NodeId) -> bool {
        if self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("textarea"))
            .unwrap_or(false)
        {
            return true;
        }
        if !self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false)
        {
            return false;
        }
        matches!(
            self.normalized_input_type(node).as_str(),
            "text" | "search" | "url" | "tel" | "email" | "password"
        )
    }

    pub(crate) fn normalize_selection_direction(direction: &str) -> &'static str {
        match direction {
            "forward" => "forward",
            "backward" => "backward",
            _ => "none",
        }
    }

    fn scroll_offset_from_object_arg(value: &Value, key: &str) -> Option<i64> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        Self::object_get_entry(&entries, key).map(|entry| Self::value_to_i64(&entry))
    }

    fn animate_option_entry(options: Option<&Value>, key: &str) -> Option<Value> {
        let options = options?;
        let Value::Object(entries) = options else {
            return None;
        };
        let entries = entries.borrow();
        Self::object_get_entry(&entries, key)
    }

    fn animate_id_from_options(options: Option<&Value>) -> String {
        match Self::animate_option_entry(options, "id") {
            Some(Value::Null) | Some(Value::Undefined) | None => String::new(),
            Some(value) => value.as_string(),
        }
    }

    pub(crate) fn apply_document_scroll_operation(&mut self, method: &str, args: &[Value]) -> bool {
        let mut next_x = self.dom_runtime.document_scroll_x;
        let mut next_y = self.dom_runtime.document_scroll_y;

        match method {
            "scroll" | "scrollTo" => match args {
                [] => {}
                [single] => {
                    if matches!(single, Value::Object(_)) {
                        if let Some(left) = Self::scroll_offset_from_object_arg(single, "left") {
                            next_x = left;
                        }
                        if let Some(top) = Self::scroll_offset_from_object_arg(single, "top") {
                            next_y = top;
                        }
                    } else {
                        next_x = Self::value_to_i64(single);
                        next_y = 0;
                    }
                }
                [x, y] => {
                    next_x = Self::value_to_i64(x);
                    next_y = Self::value_to_i64(y);
                }
                _ => {}
            },
            "scrollBy" => {
                let mut delta_x = 0;
                let mut delta_y = 0;
                match args {
                    [] => {}
                    [single] => {
                        if matches!(single, Value::Object(_)) {
                            delta_x = Self::scroll_offset_from_object_arg(single, "left").unwrap_or(0);
                            delta_y = Self::scroll_offset_from_object_arg(single, "top").unwrap_or(0);
                        } else {
                            delta_x = Self::value_to_i64(single);
                        }
                    }
                    [x, y] => {
                        delta_x = Self::value_to_i64(x);
                        delta_y = Self::value_to_i64(y);
                    }
                    _ => {}
                }
                next_x = next_x.saturating_add(delta_x);
                next_y = next_y.saturating_add(delta_y);
            }
            _ => return true,
        }

        let changed =
            next_x != self.dom_runtime.document_scroll_x || next_y != self.dom_runtime.document_scroll_y;
        self.dom_runtime.document_scroll_x = next_x;
        self.dom_runtime.document_scroll_y = next_y;
        changed
    }

    pub(crate) fn set_node_selection_range(
        &mut self,
        node: NodeId,
        start: i64,
        end: i64,
        direction: String,
    ) -> Result<()> {
        if !self.node_supports_text_selection(node) {
            return Ok(());
        }
        let before_start = self.dom.selection_start(node)?;
        let before_end = self.dom.selection_end(node)?;
        let before_direction = self.dom.selection_direction(node)?;
        let start = start.max(0) as usize;
        let end = end.max(0) as usize;
        self.dom.set_selection_range(
            node,
            start,
            end,
            Self::normalize_selection_direction(direction.as_str()),
        )?;
        let after_start = self.dom.selection_start(node)?;
        let after_end = self.dom.selection_end(node)?;
        let after_direction = self.dom.selection_direction(node)?;
        if before_start != after_start || before_end != after_end || before_direction != after_direction
        {
            let _ = self.dispatch_document_selectionchange()?;
        }
        Ok(())
    }

    pub(crate) fn shift_selection_index(index: usize, delta: i64) -> usize {
        if delta >= 0 {
            index.saturating_add(delta as usize)
        } else {
            index.saturating_sub(delta.unsigned_abs() as usize)
        }
    }

    pub(crate) fn set_node_range_text(&mut self, node: NodeId, args: &[Value]) -> Result<()> {
        if !self.node_supports_text_selection(node) {
            return Ok(());
        }

        let replacement = args[0].as_string();
        let old_value = self.dom.value(node)?;
        let old_len = old_value.chars().count();
        let old_sel_start = self.dom.selection_start(node)?;
        let old_sel_end = self.dom.selection_end(node)?;

        let (mut start, mut end, mode) = match args.len() {
            1 => (old_sel_start, old_sel_end, "preserve".to_string()),
            3 => (
                Self::value_to_i64(&args[1]).max(0) as usize,
                Self::value_to_i64(&args[2]).max(0) as usize,
                "preserve".to_string(),
            ),
            4 => (
                Self::value_to_i64(&args[1]).max(0) as usize,
                Self::value_to_i64(&args[2]).max(0) as usize,
                args[3].as_string(),
            ),
            _ => {
                return Err(Error::ScriptRuntime(
                    "setRangeText supports one, three, or four arguments".into(),
                ));
            }
        };
        start = start.min(old_len);
        end = end.min(old_len);
        if end < start {
            end = start;
        }

        let start_byte = Self::char_index_to_byte(&old_value, start);
        let end_byte = Self::char_index_to_byte(&old_value, end);
        let mut next_value = String::new();
        next_value.push_str(&old_value[..start_byte]);
        next_value.push_str(&replacement);
        next_value.push_str(&old_value[end_byte..]);
        self.dom.set_value(node, &next_value)?;

        let replacement_len = replacement.chars().count();
        let replaced_len = end.saturating_sub(start);
        let delta = replacement_len as i64 - replaced_len as i64;
        let mode = mode.to_ascii_lowercase();
        let (selection_start, selection_end) = match mode.as_str() {
            "select" => (start, start + replacement_len),
            "start" => (start, start),
            "end" => {
                let caret = start + replacement_len;
                (caret, caret)
            }
            _ => {
                if old_sel_end <= start {
                    (old_sel_start, old_sel_end)
                } else if old_sel_start >= end {
                    (
                        Self::shift_selection_index(old_sel_start, delta),
                        Self::shift_selection_index(old_sel_end, delta),
                    )
                } else {
                    let caret = start + replacement_len;
                    (caret, caret)
                }
            }
        };
        self.set_node_selection_range(
            node,
            selection_start as i64,
            selection_end as i64,
            "none".to_string(),
        )
    }

    pub(crate) fn parse_attr_f64(&self, node: NodeId, name: &str) -> Option<f64> {
        self.dom.attr(node, name).and_then(|raw| {
            let raw = raw.trim();
            if raw.is_empty() {
                None
            } else {
                raw.parse::<f64>().ok().filter(|value| value.is_finite())
            }
        })
    }

    pub(crate) fn parse_attr_i64(&self, node: NodeId, name: &str) -> Option<i64> {
        self.dom.attr(node, name).and_then(|raw| {
            let raw = raw.trim();
            if raw.is_empty() {
                None
            } else {
                raw.parse::<i64>().ok()
            }
        })
    }

    pub(crate) fn parse_number_value(raw: &str) -> Option<f64> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        raw.parse::<f64>().ok().filter(|value| value.is_finite())
    }

    pub(crate) fn parse_date_input_value_ms(raw: &str) -> Option<i64> {
        let (year, month, day) = parse_date_input_components(raw)?;
        Some(Self::utc_timestamp_ms_from_components(
            year,
            i64::from(month) - 1,
            i64::from(day),
            0,
            0,
            0,
            0,
        ))
    }

    pub(crate) fn format_date_input_from_timestamp_ms(timestamp_ms: i64) -> String {
        let (year, month, day, ..) = Self::date_components_utc(timestamp_ms);
        if !(0..=9999).contains(&year) {
            return String::new();
        }
        format!("{year:04}-{month:02}-{day:02}")
    }

    pub(crate) fn parse_datetime_local_input_value_ms(raw: &str) -> Option<i64> {
        let (year, month, day, hour, minute) = parse_datetime_local_input_components(raw)?;
        Some(Self::utc_timestamp_ms_from_components(
            year,
            i64::from(month) - 1,
            i64::from(day),
            i64::from(hour),
            i64::from(minute),
            0,
            0,
        ))
    }

    pub(crate) fn format_datetime_local_input_from_timestamp_ms(timestamp_ms: i64) -> String {
        let (year, month, day, hour, minute, ..) = Self::date_components_utc(timestamp_ms);
        if !(0..=9999).contains(&year) {
            return String::new();
        }
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}")
    }

    pub(crate) fn parse_time_input_value_ms(raw: &str) -> Option<i64> {
        let (hour, minute, second, _) = parse_time_input_components(raw)?;
        let total_seconds = i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second);
        Some(total_seconds * 1_000)
    }

    pub(crate) fn format_time_input_from_timestamp_ms(timestamp_ms: i64) -> String {
        let day_ms = 86_400_000i64;
        let wrapped = timestamp_ms.rem_euclid(day_ms);
        let total_seconds = wrapped / 1_000;
        let hour = total_seconds / 3_600;
        let minute = (total_seconds % 3_600) / 60;
        let second = total_seconds % 60;
        if second == 0 {
            format!("{hour:02}:{minute:02}")
        } else {
            format!("{hour:02}:{minute:02}:{second:02}")
        }
    }

    pub(crate) fn format_number_for_input(value: f64) -> String {
        if value.fract().abs() < 1e-9 {
            format!("{:.0}", value)
        } else {
            let mut out = value.to_string();
            if out.contains('.') {
                while out.ends_with('0') {
                    out.pop();
                }
                if out.ends_with('.') {
                    out.pop();
                }
            }
            out
        }
    }

    pub(crate) fn step_input_value(
        &mut self,
        node: NodeId,
        direction: i64,
        count: i64,
    ) -> Result<()> {
        if count == 0 {
            return Ok(());
        }
        let input_type = self.normalized_input_type(node);
        if !matches!(
            input_type.as_str(),
            "number" | "range" | "date" | "datetime-local" | "time"
        ) {
            return Ok(());
        }

        if input_type == "time" {
            let step_attr = self.dom.attr(node, "step").unwrap_or_default();
            let step_seconds = if step_attr.eq_ignore_ascii_case("any") {
                60.0
            } else {
                step_attr
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
                    .unwrap_or(60.0)
            };
            let step_ms = ((step_seconds * 1_000.0).round() as i64).max(1);
            let min = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_time_input_value_ms(&raw));
            let max = self
                .dom
                .attr(node, "max")
                .and_then(|raw| Self::parse_time_input_value_ms(&raw));
            let base = min
                .or_else(|| {
                    self.dom
                        .attr(node, "value")
                        .and_then(|raw| Self::parse_time_input_value_ms(&raw))
                })
                .unwrap_or(0);
            let current = Self::parse_time_input_value_ms(&self.dom.value(node)?).unwrap_or(base);
            let delta = (direction as i128)
                .saturating_mul(count as i128)
                .saturating_mul(step_ms as i128);
            let day_ms = 86_400_000i64;
            let mut next = (((current as i128) + delta)
                .clamp(i128::from(i64::MIN), i128::from(i64::MAX))
                as i64)
                .rem_euclid(day_ms);

            if let (Some(min), Some(max)) = (min, max) {
                if min <= max {
                    if next < min {
                        next = min;
                    }
                    if next > max {
                        next = max;
                    }
                } else {
                    let in_wrapped_range = next >= min || next <= max;
                    if !in_wrapped_range {
                        next = if direction >= 0 { min } else { max };
                    }
                }
            } else {
                if let Some(min) = min {
                    if next < min {
                        next = min;
                    }
                }
                if let Some(max) = max {
                    if next > max {
                        next = max;
                    }
                }
            }

            let next_value = Self::format_time_input_from_timestamp_ms(next);
            return self.dom.set_value(node, &next_value);
        }

        if input_type == "date" {
            let step_attr = self.dom.attr(node, "step").unwrap_or_default();
            let step_days = if step_attr.eq_ignore_ascii_case("any") {
                1.0
            } else {
                step_attr
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
                    .unwrap_or(1.0)
            };
            let step_ms = ((step_days * 86_400_000.0).round() as i64).max(1);
            let base = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                .or_else(|| {
                    self.dom
                        .attr(node, "value")
                        .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                })
                .unwrap_or(0);
            let current = Self::parse_date_input_value_ms(&self.dom.value(node)?).unwrap_or(base);
            let delta = (direction as i128)
                .saturating_mul(count as i128)
                .saturating_mul(step_ms as i128);
            let mut next = ((current as i128) + delta)
                .clamp(i128::from(i64::MIN), i128::from(i64::MAX))
                as i64;
            if let Some(min) = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
            {
                if next < min {
                    next = min;
                }
            }
            if let Some(max) = self
                .dom
                .attr(node, "max")
                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
            {
                if next > max {
                    next = max;
                }
            }
            let next_value = Self::format_date_input_from_timestamp_ms(next);
            return self.dom.set_value(node, &next_value);
        }

        if input_type == "datetime-local" {
            let step_attr = self.dom.attr(node, "step").unwrap_or_default();
            let step_seconds = if step_attr.eq_ignore_ascii_case("any") {
                60.0
            } else {
                step_attr
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
                    .unwrap_or(60.0)
            };
            let step_ms = ((step_seconds * 1_000.0).round() as i64).max(1);
            let base = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                .or_else(|| {
                    self.dom
                        .attr(node, "value")
                        .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                })
                .unwrap_or(0);
            let current =
                Self::parse_datetime_local_input_value_ms(&self.dom.value(node)?).unwrap_or(base);
            let delta = (direction as i128)
                .saturating_mul(count as i128)
                .saturating_mul(step_ms as i128);
            let mut next = ((current as i128) + delta)
                .clamp(i128::from(i64::MIN), i128::from(i64::MAX))
                as i64;
            if let Some(min) = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
            {
                if next < min {
                    next = min;
                }
            }
            if let Some(max) = self
                .dom
                .attr(node, "max")
                .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
            {
                if next > max {
                    next = max;
                }
            }
            let next_value = Self::format_datetime_local_input_from_timestamp_ms(next);
            return self.dom.set_value(node, &next_value);
        }

        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
        let step = if step_attr.eq_ignore_ascii_case("any") {
            1.0
        } else {
            step_attr
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
                .unwrap_or(1.0)
        };
        let base = self
            .parse_attr_f64(node, "min")
            .or_else(|| self.parse_attr_f64(node, "value"))
            .unwrap_or(0.0);
        let current = Self::parse_number_value(&self.dom.value(node)?).unwrap_or(base);
        let mut next = current + (direction as f64) * (count as f64) * step;
        if let Some(min) = self.parse_attr_f64(node, "min") {
            if next < min {
                next = min;
            }
        }
        if let Some(max) = self.parse_attr_f64(node, "max") {
            if next > max {
                next = max;
            }
        }
        self.dom
            .set_value(node, &Self::format_number_for_input(next))
    }

    pub(crate) fn input_value_as_number(&self, node: NodeId) -> Result<f64> {
        let input_type = self.normalized_input_type(node);
        let value = self.dom.value(node)?;
        let number = match input_type.as_str() {
            "number" | "range" => Self::parse_number_value(&value).unwrap_or(f64::NAN),
            "date" => Self::parse_date_input_value_ms(&value)
                .map(|timestamp| timestamp as f64)
                .unwrap_or(f64::NAN),
            "datetime-local" => Self::parse_datetime_local_input_value_ms(&value)
                .map(|timestamp| timestamp as f64)
                .unwrap_or(f64::NAN),
            "time" => Self::parse_time_input_value_ms(&value)
                .map(|timestamp| timestamp as f64)
                .unwrap_or(f64::NAN),
            _ => f64::NAN,
        };
        Ok(number)
    }

    pub(crate) fn set_input_value_as_number(&mut self, node: NodeId, number: f64) -> Result<()> {
        let input_type = self.normalized_input_type(node);
        if input_type == "date" {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            let timestamp_ms = number as i64;
            let formatted = Self::format_date_input_from_timestamp_ms(timestamp_ms);
            return self.dom.set_value(node, &formatted);
        }
        if input_type == "datetime-local" {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            let timestamp_ms = number as i64;
            let formatted = Self::format_datetime_local_input_from_timestamp_ms(timestamp_ms);
            return self.dom.set_value(node, &formatted);
        }
        if input_type == "time" {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            let timestamp_ms = number as i64;
            let formatted = Self::format_time_input_from_timestamp_ms(timestamp_ms);
            return self.dom.set_value(node, &formatted);
        }
        if matches!(input_type.as_str(), "number" | "range") {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            return self
                .dom
                .set_value(node, &Self::format_number_for_input(number));
        }
        self.dom.set_value(node, "")
    }

    pub(crate) fn input_value_as_date_ms(&self, node: NodeId) -> Result<Option<i64>> {
        let input_type = self.normalized_input_type(node);
        if input_type == "date" {
            return Ok(Self::parse_date_input_value_ms(&self.dom.value(node)?));
        }
        if input_type == "datetime-local" {
            return Ok(Self::parse_datetime_local_input_value_ms(
                &self.dom.value(node)?,
            ));
        }
        if input_type == "time" {
            return Ok(Self::parse_time_input_value_ms(&self.dom.value(node)?));
        }
        if !matches!(input_type.as_str(), "date" | "datetime-local" | "time") {
            return Ok(None);
        }
        Ok(None)
    }

    pub(crate) fn set_input_value_as_date_ms(
        &mut self,
        node: NodeId,
        timestamp_ms: Option<i64>,
    ) -> Result<()> {
        let input_type = self.normalized_input_type(node);
        if !matches!(input_type.as_str(), "date" | "datetime-local" | "time") {
            return self.dom.set_value(node, "");
        }

        let Some(timestamp_ms) = timestamp_ms else {
            return self.dom.set_value(node, "");
        };
        let formatted = if input_type == "date" {
            Self::format_date_input_from_timestamp_ms(timestamp_ms)
        } else if input_type == "time" {
            Self::format_time_input_from_timestamp_ms(timestamp_ms)
        } else {
            Self::format_datetime_local_input_from_timestamp_ms(timestamp_ms)
        };
        self.dom.set_value(node, &formatted)
    }

    pub(crate) fn is_radio_group_checked(&self, node: NodeId) -> bool {
        let name = self.dom.attr(node, "name").unwrap_or_default();
        if name.is_empty() {
            return self.dom.checked(node).unwrap_or(false);
        }
        let form = self.dom.find_ancestor_by_tag(node, "form");
        self.dom.all_element_nodes().into_iter().any(|candidate| {
            is_radio_input(&self.dom, candidate)
                && self.dom.attr(candidate, "name").unwrap_or_default() == name
                && self.dom.find_ancestor_by_tag(candidate, "form") == form
                && self.dom.checked(candidate).unwrap_or(false)
        })
    }

    pub(crate) fn is_ascii_email_local_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                '.' | '!'
                    | '#'
                    | '$'
                    | '%'
                    | '&'
                    | '\''
                    | '*'
                    | '+'
                    | '/'
                    | '='
                    | '?'
                    | '^'
                    | '_'
                    | '`'
                    | '{'
                    | '|'
                    | '}'
                    | '~'
                    | '-'
            )
    }

    pub(crate) fn is_valid_email_domain_label(label: &str) -> bool {
        if label.is_empty() || label.len() > 63 {
            return false;
        }

        let mut chars = label.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !first.is_ascii_alphanumeric() {
            return false;
        }

        let mut last = first;
        for ch in chars {
            if !(ch.is_ascii_alphanumeric() || ch == '-') {
                return false;
            }
            last = ch;
        }

        last.is_ascii_alphanumeric()
    }

    pub(crate) fn is_valid_email_domain(domain: &str) -> bool {
        !domain.is_empty() && domain.split('.').all(Self::is_valid_email_domain_label)
    }

    pub(crate) fn is_simple_email(value: &str) -> bool {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return false;
        }
        let Some((local, domain)) = trimmed.split_once('@') else {
            return false;
        };
        if local.is_empty() || domain.is_empty() || domain.contains('@') {
            return false;
        }
        if !local.chars().all(Self::is_ascii_email_local_char) {
            return false;
        }
        Self::is_valid_email_domain(domain)
    }

    pub(crate) fn is_email_address_list(value: &str) -> bool {
        if value.trim().is_empty() {
            return true;
        }

        for part in value.split(',') {
            let part = part.trim();
            if part.is_empty() || !Self::is_simple_email(part) {
                return false;
            }
        }
        true
    }

    pub(crate) fn is_url_like(value: &str) -> bool {
        LocationParts::parse(value).is_some()
    }

    pub(crate) fn input_participates_in_constraint_validation(kind: &str) -> bool {
        !matches!(kind, "button" | "submit" | "reset" | "hidden" | "image")
    }

    pub(crate) fn compute_input_validity(&self, node: NodeId) -> Result<InputValidity> {
        let mut validity = InputValidity {
            valid: true,
            ..InputValidity::default()
        };

        if self.is_effectively_disabled(node) {
            return Ok(validity);
        }

        let Some(tag_name) = self.dom.tag_name(node) else {
            return Ok(validity);
        };
        if tag_name.eq_ignore_ascii_case("textarea") {
            let value = self.dom.value(node)?;
            let required = self.dom.required(node);
            let readonly = self.dom.readonly(node);

            if required && !readonly && value.is_empty() {
                validity.value_missing = true;
            }

            if !value.is_empty() {
                let value_len = value.chars().count() as i64;
                if let Some(min_len) = self.parse_attr_i64(node, "minlength") {
                    if min_len >= 0 && value_len < min_len {
                        validity.too_short = true;
                    }
                }
                if let Some(max_len) = self.parse_attr_i64(node, "maxlength") {
                    if max_len >= 0 && value_len > max_len {
                        validity.too_long = true;
                    }
                }
            }

            validity.custom_error = !self.dom.custom_validity_message(node)?.is_empty();
            validity.valid = !(validity.value_missing
                || validity.type_mismatch
                || validity.pattern_mismatch
                || validity.too_long
                || validity.too_short
                || validity.range_underflow
                || validity.range_overflow
                || validity.step_mismatch
                || validity.bad_input
                || validity.custom_error);
            return Ok(validity);
        }
        if !tag_name.eq_ignore_ascii_case("input") {
            let custom_error = !self.dom.custom_validity_message(node)?.is_empty();
            validity.custom_error = custom_error;
            validity.valid = !custom_error;
            return Ok(validity);
        }

        let input_type = self.normalized_input_type(node);
        if !Self::input_participates_in_constraint_validation(input_type.as_str()) {
            return Ok(validity);
        }
        let value = self.dom.value(node)?;
        let value_is_empty = value.is_empty();
        let required = self.dom.required(node);
        let readonly = self.dom.readonly(node);
        let multiple = self.dom.attr(node, "multiple").is_some();
        let email_multiple = input_type == "email" && multiple;
        let value_is_effectively_empty = if email_multiple {
            value.trim().is_empty()
        } else {
            value_is_empty
        };

        if required && !readonly && Self::input_supports_required(input_type.as_str()) {
            validity.value_missing = if input_type == "checkbox" {
                !self.dom.checked(node)?
            } else if input_type == "radio" {
                !self.is_radio_group_checked(node)
            } else if email_multiple {
                false
            } else {
                value_is_effectively_empty
            };
        }

        if !value_is_effectively_empty {
            if input_type == "email" {
                validity.type_mismatch = if email_multiple {
                    !Self::is_email_address_list(&value)
                } else {
                    !Self::is_simple_email(&value)
                };
            } else if input_type == "url" {
                validity.type_mismatch = !Self::is_url_like(&value);
            }

            if matches!(
                input_type.as_str(),
                "text" | "search" | "url" | "tel" | "email" | "password"
            ) {
                let value_len = value.chars().count() as i64;
                if let Some(min_len) = self.parse_attr_i64(node, "minlength") {
                    if min_len >= 0 && value_len < min_len {
                        validity.too_short = true;
                    }
                }
                if let Some(max_len) = self.parse_attr_i64(node, "maxlength") {
                    if max_len >= 0 && value_len > max_len {
                        validity.too_long = true;
                    }
                }

                if let Some(pattern) = self.dom.attr(node, "pattern") {
                    if !pattern.is_empty() {
                        let wrapped = format!("^(?:{})$", pattern);
                        if let Ok(regex) = Regex::new(&wrapped) {
                            if input_type == "email" && multiple {
                                for part in value.split(',') {
                                    let part = part.trim();
                                    if part.is_empty() {
                                        continue;
                                    }
                                    match regex.is_match(part) {
                                        Ok(true) => {}
                                        Ok(false) => {
                                            validity.pattern_mismatch = true;
                                            break;
                                        }
                                        Err(_) => {}
                                    }
                                }
                            } else if let Ok(false) = regex.is_match(&value) {
                                validity.pattern_mismatch = true;
                            }
                        }
                    }
                }
            }

            if input_type == "date" {
                match Self::parse_date_input_value_ms(&value) {
                    Some(date_value_ms) => {
                        if let Some(min) = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                        {
                            if date_value_ms < min {
                                validity.range_underflow = true;
                            }
                        }
                        if let Some(max) = self
                            .dom
                            .attr(node, "max")
                            .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                        {
                            if date_value_ms > max {
                                validity.range_overflow = true;
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        if !step_attr.eq_ignore_ascii_case("any") {
                            let step_days = step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(1.0);
                            let step_ms = step_days * 86_400_000.0;
                            let base = self
                                .dom
                                .attr(node, "min")
                                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                                .or_else(|| {
                                    self.dom
                                        .attr(node, "value")
                                        .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                                })
                                .unwrap_or(0) as f64;
                            let ratio = ((date_value_ms as f64) - base) / step_ms;
                            let nearest = ratio.round();
                            if (ratio - nearest).abs() > 1e-7 {
                                validity.step_mismatch = true;
                            }
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            } else if input_type == "datetime-local" {
                match Self::parse_datetime_local_input_value_ms(&value) {
                    Some(datetime_value_ms) => {
                        if let Some(min) = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                        {
                            if datetime_value_ms < min {
                                validity.range_underflow = true;
                            }
                        }
                        if let Some(max) = self
                            .dom
                            .attr(node, "max")
                            .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                        {
                            if datetime_value_ms > max {
                                validity.range_overflow = true;
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        let step_seconds = if step_attr.eq_ignore_ascii_case("any") {
                            60.0
                        } else {
                            step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(60.0)
                        };
                        let step_ms = step_seconds * 1_000.0;
                        let base = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                            .or_else(|| {
                                self.dom
                                    .attr(node, "value")
                                    .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                            })
                            .unwrap_or(0) as f64;
                        let ratio = ((datetime_value_ms as f64) - base) / step_ms;
                        let nearest = ratio.round();
                        if (ratio - nearest).abs() > 1e-7 {
                            validity.step_mismatch = true;
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            } else if input_type == "time" {
                match Self::parse_time_input_value_ms(&value) {
                    Some(time_value_ms) => {
                        let min = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_time_input_value_ms(&raw));
                        let max = self
                            .dom
                            .attr(node, "max")
                            .and_then(|raw| Self::parse_time_input_value_ms(&raw));
                        if let (Some(min), Some(max)) = (min, max) {
                            if min <= max {
                                if time_value_ms < min {
                                    validity.range_underflow = true;
                                }
                                if time_value_ms > max {
                                    validity.range_overflow = true;
                                }
                            } else {
                                let in_wrapped_range = time_value_ms >= min || time_value_ms <= max;
                                if !in_wrapped_range {
                                    validity.range_underflow = true;
                                    validity.range_overflow = true;
                                }
                            }
                        } else {
                            if let Some(min) = min {
                                if time_value_ms < min {
                                    validity.range_underflow = true;
                                }
                            }
                            if let Some(max) = max {
                                if time_value_ms > max {
                                    validity.range_overflow = true;
                                }
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        if !step_attr.eq_ignore_ascii_case("any") {
                            let step_seconds = step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(60.0);
                            let step_ms = step_seconds * 1_000.0;
                            let base = self
                                .dom
                                .attr(node, "min")
                                .and_then(|raw| Self::parse_time_input_value_ms(&raw))
                                .or_else(|| {
                                    self.dom
                                        .attr(node, "value")
                                        .and_then(|raw| Self::parse_time_input_value_ms(&raw))
                                })
                                .unwrap_or(0) as f64;
                            let ratio = ((time_value_ms as f64) - base) / step_ms;
                            let nearest = ratio.round();
                            if (ratio - nearest).abs() > 1e-7 {
                                validity.step_mismatch = true;
                            }
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            } else if matches!(input_type.as_str(), "number" | "range") {
                match Self::parse_number_value(&value) {
                    Some(numeric) => {
                        if let Some(min) = self.parse_attr_f64(node, "min") {
                            if numeric < min {
                                validity.range_underflow = true;
                            }
                        }
                        if let Some(max) = self.parse_attr_f64(node, "max") {
                            if numeric > max {
                                validity.range_overflow = true;
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        if !step_attr.eq_ignore_ascii_case("any") {
                            let step = step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(1.0);
                            let base = self
                                .parse_attr_f64(node, "min")
                                .or_else(|| self.parse_attr_f64(node, "value"))
                                .unwrap_or(0.0);
                            let ratio = (numeric - base) / step;
                            let nearest = ratio.round();
                            if (ratio - nearest).abs() > 1e-7 {
                                validity.step_mismatch = true;
                            }
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            }
        }

        validity.custom_error = !self.dom.custom_validity_message(node)?.is_empty();
        validity.valid = !(validity.value_missing
            || validity.type_mismatch
            || validity.pattern_mismatch
            || validity.too_long
            || validity.too_short
            || validity.range_underflow
            || validity.range_overflow
            || validity.step_mismatch
            || validity.bad_input
            || validity.custom_error);
        Ok(validity)
    }

    pub(crate) fn input_validity_to_value(validity: &InputValidity) -> Value {
        Self::new_object_value(vec![
            (
                "valueMissing".to_string(),
                Value::Bool(validity.value_missing),
            ),
            (
                "typeMismatch".to_string(),
                Value::Bool(validity.type_mismatch),
            ),
            (
                "patternMismatch".to_string(),
                Value::Bool(validity.pattern_mismatch),
            ),
            ("tooLong".to_string(), Value::Bool(validity.too_long)),
            ("tooShort".to_string(), Value::Bool(validity.too_short)),
            (
                "rangeUnderflow".to_string(),
                Value::Bool(validity.range_underflow),
            ),
            (
                "rangeOverflow".to_string(),
                Value::Bool(validity.range_overflow),
            ),
            (
                "stepMismatch".to_string(),
                Value::Bool(validity.step_mismatch),
            ),
            ("badInput".to_string(), Value::Bool(validity.bad_input)),
            (
                "customError".to_string(),
                Value::Bool(validity.custom_error),
            ),
            ("valid".to_string(), Value::Bool(validity.valid)),
        ])
    }
}
