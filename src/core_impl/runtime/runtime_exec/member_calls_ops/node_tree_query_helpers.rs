use super::*;

impl Harness {
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

    pub(crate) fn eval_node_after_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if self.dom.parent(node).is_none() {
            return Ok(Value::Undefined);
        }

        let mut insertion_anchor = node;
        for value in evaluated_args {
            let (child, new_anchor) = match value {
                Value::Node(child) => {
                    let new_anchor = if self.is_document_fragment_node(*child) {
                        self.dom.nodes[child.0].children.last().copied()
                    } else {
                        Some(*child)
                    };
                    (*child, new_anchor)
                }
                other => {
                    let text = self.dom.create_detached_text(other.as_string());
                    (text, Some(text))
                }
            };
            self.dom.insert_after(insertion_anchor, child)?;
            if let Some(new_anchor) = new_anchor {
                if self.dom.parent(new_anchor).is_some() {
                    insertion_anchor = new_anchor;
                }
            }
        }

        Ok(Value::Undefined)
    }

    pub(crate) fn eval_node_before_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        let Some(parent) = self.dom.parent(node) else {
            return Ok(Value::Undefined);
        };

        for value in evaluated_args {
            let child = match value {
                Value::Node(child) => *child,
                other => self.dom.create_detached_text(other.as_string()),
            };
            self.dom.insert_before(parent, child, node)?;
        }

        Ok(Value::Undefined)
    }

    pub(crate) fn eval_node_prepend_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if matches!(
            self.dom.nodes.get(node.0).map(|entry| &entry.node_type),
            Some(NodeType::Document)
        ) {
            let mut nodes = Vec::new();
            for value in evaluated_args {
                match value {
                    Value::Node(candidate) => {
                        self.collect_appendable_document_nodes(*candidate, &mut nodes)
                    }
                    other => nodes.push(self.dom.create_detached_text(other.as_string())),
                }
            }

            let mut existing_elements = self.dom.nodes[node.0]
                .children
                .iter()
                .copied()
                .filter(|child| {
                    self.dom.element(*child).is_some_and(|element| {
                        !element.tag_name.eq_ignore_ascii_case("#document-fragment")
                    })
                })
                .count() as i64;

            for candidate in &nodes {
                if self.dom.parent(*candidate) == Some(node)
                    && self.dom.element(*candidate).is_some_and(|element| {
                        !element.tag_name.eq_ignore_ascii_case("#document-fragment")
                    })
                {
                    existing_elements -= 1;
                }
            }

            let mut prepended_elements = 0i64;
            for candidate in &nodes {
                match self
                    .dom
                    .nodes
                    .get(candidate.0)
                    .map(|entry| &entry.node_type)
                {
                    Some(NodeType::Document) | Some(NodeType::Text(_)) => {
                        return Err(Self::hierarchy_request_error());
                    }
                    Some(NodeType::Element(element))
                        if !element.tag_name.eq_ignore_ascii_case("#document-fragment") =>
                    {
                        prepended_elements += 1;
                    }
                    Some(NodeType::Element(_)) => {}
                    None => return Err(Self::hierarchy_request_error()),
                }
            }

            if existing_elements + prepended_elements > 1 {
                return Err(Self::hierarchy_request_error());
            }

            for candidate in nodes.into_iter().rev() {
                self.dom.prepend_child(node, candidate)?;
            }
            return Ok(Value::Undefined);
        }

        for value in evaluated_args.iter().rev() {
            let child = match value {
                Value::Node(child) => *child,
                other => self.dom.create_detached_text(other.as_string()),
            };
            self.dom.prepend_child(node, child)?;
        }

        Ok(Value::Undefined)
    }

    pub(crate) fn eval_node_replace_children_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        let mut replacements = Vec::with_capacity(evaluated_args.len());
        for value in evaluated_args {
            let child = match value {
                Value::Node(child) => *child,
                other => self.dom.create_detached_text(other.as_string()),
            };
            let Some(child_node) = self.dom.nodes.get(child.0) else {
                return Err(Self::hierarchy_request_error());
            };
            if matches!(child_node.node_type, NodeType::Document)
                || child == node
                || self.dom.is_descendant_of(node, child)
            {
                return Err(Self::hierarchy_request_error());
            }
            replacements.push(child);
        }

        let Some(node_entry) = self.dom.nodes.get(node.0) else {
            return Err(Self::hierarchy_request_error());
        };
        let existing_children = node_entry.children.clone();
        for child in existing_children {
            self.dom.remove_child(node, child)?;
        }
        for child in replacements {
            self.dom
                .append_child(node, child)
                .map_err(|_| Self::hierarchy_request_error())?;
        }

        Ok(Value::Undefined)
    }

    pub(crate) fn eval_node_replace_with_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        let Some(parent) = self.dom.parent(node) else {
            return Ok(Value::Undefined);
        };

        let mut replacements = Vec::with_capacity(evaluated_args.len());
        for value in evaluated_args {
            let child = match value {
                Value::Node(child) => *child,
                other => self.dom.create_detached_text(other.as_string()),
            };
            let Some(child_node) = self.dom.nodes.get(child.0) else {
                return Err(Self::hierarchy_request_error());
            };
            if matches!(child_node.node_type, NodeType::Document)
                || child == parent
                || self.dom.is_descendant_of(parent, child)
            {
                return Err(Self::hierarchy_request_error());
            }
            replacements.push(child);
        }

        let next_sibling = self.dom.nodes.get(parent.0).and_then(|entry| {
            let idx = entry.children.iter().position(|child| *child == node)?;
            entry.children.get(idx + 1).copied()
        });

        self.dom
            .remove_child(parent, node)
            .map_err(|_| Self::hierarchy_request_error())?;

        for child in replacements {
            if let Some(reference) = next_sibling {
                if self.dom.parent(reference) == Some(parent) {
                    self.dom
                        .insert_before(parent, child, reference)
                        .map_err(|_| Self::hierarchy_request_error())?;
                    continue;
                }
            }
            self.dom
                .append_child(parent, child)
                .map_err(|_| Self::hierarchy_request_error())?;
        }

        Ok(Value::Undefined)
    }

    pub(crate) fn eval_insert_adjacent_element_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if evaluated_args.len() != 2 {
            return Err(Error::ScriptRuntime(
                "insertAdjacentElement requires exactly two arguments".into(),
            ));
        }
        if self.dom.element(node).is_none() || self.is_document_fragment_node(node) {
            return Err(Error::ScriptRuntime(
                "TypeError: insertAdjacentElement target must be an Element".into(),
            ));
        }

        let position_text = evaluated_args[0].as_string();
        let position = resolve_insert_adjacent_position(&position_text).map_err(|_| {
            Error::ScriptRuntime(format!(
                "SyntaxError: Failed to execute 'insertAdjacentElement': invalid position '{position_text}'"
            ))
        })?;

        let element = match evaluated_args.get(1) {
            Some(Value::Node(element))
                if self.dom.element(*element).is_some()
                    && !self.is_document_fragment_node(*element) =>
            {
                *element
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Failed to execute 'insertAdjacentElement': parameter 2 is not of type 'Element'"
                        .into(),
                ));
            }
        };

        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(node) else {
                return Ok(Value::Null);
            };
            if self.dom.element(parent).is_none() || self.is_document_fragment_node(parent) {
                return Ok(Value::Null);
            }
        }

        if self
            .dom
            .insert_adjacent_node(node, position, element)
            .is_err()
        {
            return Ok(Value::Null);
        }
        Ok(Value::Node(element))
    }

    pub(crate) fn eval_insert_adjacent_html_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if evaluated_args.len() != 2 {
            return Err(Error::ScriptRuntime(
                "insertAdjacentHTML requires exactly two arguments".into(),
            ));
        }
        if self.dom.element(node).is_none() || self.is_document_fragment_node(node) {
            return Err(Error::ScriptRuntime(
                "TypeError: insertAdjacentHTML target must be an Element".into(),
            ));
        }

        let position_text = evaluated_args[0].as_string();
        let position = resolve_insert_adjacent_position(&position_text).map_err(|_| {
            Error::ScriptRuntime(format!(
                "SyntaxError: Failed to execute 'insertAdjacentHTML': invalid position '{position_text}'"
            ))
        })?;

        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(node) else {
                return Err(Error::ScriptRuntime(
                    "NoModificationAllowedError: Failed to execute 'insertAdjacentHTML' because the target has no parent element"
                        .into(),
                ));
            };
            if self.dom.element(parent).is_none() || self.is_document_fragment_node(parent) {
                return Err(Error::ScriptRuntime(
                    "NoModificationAllowedError: Failed to execute 'insertAdjacentHTML' on a node whose parent is not an Element"
                        .into(),
                ));
            }
        }

        let input = evaluated_args[1].as_string();
        match self.dom.insert_adjacent_html(node, position, &input) {
            Ok(()) => Ok(Value::Undefined),
            Err(Error::ScriptParse(message)) => {
                Err(Error::ScriptRuntime(format!("SyntaxError: {message}")))
            }
            Err(other) => Err(other),
        }
    }

    pub(crate) fn eval_insert_adjacent_text_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if evaluated_args.len() != 2 {
            return Err(Error::ScriptRuntime(
                "insertAdjacentText requires exactly two arguments".into(),
            ));
        }
        if self.dom.element(node).is_none() || self.is_document_fragment_node(node) {
            return Err(Error::ScriptRuntime(
                "TypeError: insertAdjacentText target must be an Element".into(),
            ));
        }

        let position_text = evaluated_args[0].as_string();
        let position = resolve_insert_adjacent_position(&position_text).map_err(|_| {
            Error::ScriptRuntime(format!(
                "SyntaxError: Failed to execute 'insertAdjacentText': invalid position '{position_text}'"
            ))
        })?;

        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(node) else {
                return Ok(Value::Undefined);
            };
            if self.dom.element(parent).is_none() || self.is_document_fragment_node(parent) {
                return Ok(Value::Undefined);
            }
        }

        let text = self.dom.create_detached_text(evaluated_args[1].as_string());
        let _ = self.dom.insert_adjacent_node(node, position, text);
        Ok(Value::Undefined)
    }

    pub(crate) fn eval_closest_selector_value(
        &self,
        node: NodeId,
        selector: &str,
    ) -> Result<Value> {
        match self.dom.closest(node, selector) {
            Ok(Some(matched)) => Ok(Value::Node(matched)),
            Ok(None) => Ok(Value::Null),
            Err(Error::UnsupportedSelector(_)) => Err(Error::ScriptRuntime(
                "SyntaxError: The provided selector is invalid".into(),
            )),
            Err(other) => Err(other),
        }
    }

    pub(crate) fn eval_matches_selector_value(
        &self,
        node: NodeId,
        selector: &str,
    ) -> Result<Value> {
        match self.dom.matches_selector(node, selector) {
            Ok(matched) => Ok(Value::Bool(matched)),
            Err(Error::UnsupportedSelector(_)) => Err(Error::ScriptRuntime(
                "SyntaxError: The provided selector is invalid".into(),
            )),
            Err(other) => Err(other),
        }
    }

    pub(crate) fn eval_query_selector_value(&self, node: NodeId, selector: &str) -> Result<Value> {
        match self.dom.query_selector_from(&node, selector) {
            Ok(Some(matched)) => Ok(Value::Node(matched)),
            Ok(None) => Ok(Value::Null),
            Err(Error::UnsupportedSelector(_)) => Err(Error::ScriptRuntime(
                "SyntaxError: The provided selector is invalid".into(),
            )),
            Err(other) => Err(other),
        }
    }

    pub(crate) fn eval_query_selector_all_value(
        &self,
        node: NodeId,
        selector: &str,
    ) -> Result<Value> {
        match self.dom.query_selector_all_from(&node, selector) {
            Ok(nodes) => Ok(Self::new_static_node_list_value(nodes)),
            Err(Error::UnsupportedSelector(_)) => Err(Error::ScriptRuntime(
                "SyntaxError: The provided selector is invalid".into(),
            )),
            Err(other) => Err(other),
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
}
