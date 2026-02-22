use super::*;

impl Dom {
    pub(crate) fn ensure_document_body_element(&mut self) -> Result<NodeId> {
        if let Some(body) = self.body() {
            return Ok(body);
        }

        let Some(document_element) = self.document_element() else {
            return self.wrap_root_children_with_html_body();
        };

        if !self
            .tag_name(document_element)
            .map(|tag| tag.eq_ignore_ascii_case("html"))
            .unwrap_or(false)
        {
            return self.wrap_root_children_with_html_body();
        }

        let body = self.create_element(document_element, "body".to_string(), HashMap::new());
        let html_children = self.nodes[document_element.0].children.clone();
        for child in html_children {
            if child == body {
                continue;
            }

            let keep_in_html = self
                .tag_name(child)
                .map(|tag| {
                    tag.eq_ignore_ascii_case("head")
                        || tag.eq_ignore_ascii_case("body")
                        || tag.eq_ignore_ascii_case("frameset")
                })
                .unwrap_or(false);
            if keep_in_html {
                continue;
            }
            self.append_child(body, child)?;
        }

        Ok(body)
    }

    fn wrap_root_children_with_html_body(&mut self) -> Result<NodeId> {
        let root_children = self.nodes[self.root.0].children.clone();
        let html = self.create_element(self.root, "html".to_string(), HashMap::new());
        let body = self.create_element(html, "body".to_string(), HashMap::new());
        for child in root_children {
            self.append_child(body, child)?;
        }
        Ok(body)
    }

    pub(crate) fn can_have_children(&self, node_id: NodeId) -> bool {
        matches!(
            self.nodes.get(node_id.0).map(|n| &n.node_type),
            Some(NodeType::Document | NodeType::Element(_))
        )
    }

    pub(crate) fn is_valid_node(&self, node_id: NodeId) -> bool {
        node_id.0 < self.nodes.len()
    }

    pub(crate) fn is_connected(&self, node_id: NodeId) -> bool {
        let mut cursor = Some(node_id);
        while let Some(node) = cursor {
            if node == self.root {
                return true;
            }
            cursor = self.parent(node);
        }
        false
    }

    pub(crate) fn rebuild_id_index(&mut self) {
        let mut next = HashMap::new();
        let mut stack = vec![self.root];
        while let Some(node) = stack.pop() {
            match &self.nodes[node.0].node_type {
                NodeType::Element(element) => {
                    if let Some(id) = element.attrs.get("id") {
                        Self::index_id_map(&mut next, id, node);
                    }
                }
                NodeType::Document | NodeType::Text(_) => {}
            }
            for child in self.nodes[node.0].children.iter().rev() {
                stack.push(*child);
            }
        }
        self.id_index = next;
    }

    pub(crate) fn index_id_map(next: &mut HashMap<String, Vec<NodeId>>, id: &str, node_id: NodeId) {
        if id.is_empty() {
            return;
        }
        next.entry(id.to_string()).or_default().push(node_id);
    }

    pub(crate) fn collect_elements_dfs(&self, node_id: NodeId, out: &mut Vec<NodeId>) {
        if matches!(self.nodes[node_id.0].node_type, NodeType::Element(_)) {
            out.push(node_id);
        }
        for child in &self.nodes[node_id.0].children {
            self.collect_elements_dfs(*child, out);
        }
    }

    pub(crate) fn collect_elements_descendants_dfs(&self, node_id: NodeId, out: &mut Vec<NodeId>) {
        for child in &self.nodes[node_id.0].children {
            self.collect_elements_dfs(*child, out);
        }
    }

    pub(crate) fn all_element_nodes(&self) -> Vec<NodeId> {
        let mut out = Vec::new();
        self.collect_elements_dfs(self.root, &mut out);
        out
    }

    pub(crate) fn child_elements(&self, node_id: NodeId) -> Vec<NodeId> {
        self.nodes[node_id.0]
            .children
            .iter()
            .copied()
            .filter(|child| self.element(*child).is_some())
            .collect()
    }

    pub(crate) fn child_element_count(&self, node_id: NodeId) -> usize {
        self.child_elements(node_id).len()
    }

    pub(crate) fn first_element_child(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes[node_id.0]
            .children
            .iter()
            .copied()
            .find(|child| self.element(*child).is_some())
    }

    pub(crate) fn last_element_child(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes[node_id.0]
            .children
            .iter()
            .rev()
            .copied()
            .find(|child| self.element(*child).is_some())
    }

    pub(crate) fn document_element(&self) -> Option<NodeId> {
        self.first_element_child(self.root)
    }

    pub(crate) fn head(&self) -> Option<NodeId> {
        if let Some(document_element) = self.document_element() {
            if self
                .tag_name(document_element)
                .map(|tag| tag.eq_ignore_ascii_case("html"))
                .unwrap_or(false)
            {
                return self
                    .child_elements(document_element)
                    .into_iter()
                    .find(|child| {
                        self.tag_name(*child)
                            .map(|tag| tag.eq_ignore_ascii_case("head"))
                            .unwrap_or(false)
                    });
            }
        }
        self.query_selector("head").ok().flatten()
    }

    pub(crate) fn body(&self) -> Option<NodeId> {
        if let Some(document_element) = self.document_element() {
            if self
                .tag_name(document_element)
                .map(|tag| tag.eq_ignore_ascii_case("html"))
                .unwrap_or(false)
            {
                return self
                    .child_elements(document_element)
                    .into_iter()
                    .find(|child| {
                        self.tag_name(*child)
                            .map(|tag| {
                                tag.eq_ignore_ascii_case("body")
                                    || tag.eq_ignore_ascii_case("frameset")
                            })
                            .unwrap_or(false)
                    });
            }
        }
        self.query_selector("body")
            .ok()
            .flatten()
            .or_else(|| self.query_selector("frameset").ok().flatten())
    }

    pub(crate) fn normalize_single_body_element(&mut self) -> Result<()> {
        let Some(document_element) = self.document_element() else {
            return Ok(());
        };
        if !self
            .tag_name(document_element)
            .map(|tag| tag.eq_ignore_ascii_case("html"))
            .unwrap_or(false)
        {
            return Ok(());
        }

        let body_like_children = self
            .child_elements(document_element)
            .into_iter()
            .filter(|child| {
                self.tag_name(*child)
                    .map(|tag| {
                        tag.eq_ignore_ascii_case("body") || tag.eq_ignore_ascii_case("frameset")
                    })
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        if body_like_children.len() <= 1 {
            return Ok(());
        }

        let primary_body = body_like_children[0];
        for extra_body in body_like_children.into_iter().skip(1) {
            let children = self.nodes[extra_body.0].children.clone();
            for child in children {
                self.append_child(primary_body, child)?;
            }
            self.remove_child(document_element, extra_body)?;
        }

        Ok(())
    }

    pub(crate) fn normalize_single_head_element(&mut self) -> Result<()> {
        let Some(document_element) = self.document_element() else {
            return Ok(());
        };
        if !self
            .tag_name(document_element)
            .map(|tag| tag.eq_ignore_ascii_case("html"))
            .unwrap_or(false)
        {
            return Ok(());
        }

        let head_children = self
            .child_elements(document_element)
            .into_iter()
            .filter(|child| {
                self.tag_name(*child)
                    .map(|tag| tag.eq_ignore_ascii_case("head"))
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        if head_children.len() <= 1 {
            return Ok(());
        }

        let primary_head = head_children[0];
        for extra_head in head_children.into_iter().skip(1) {
            let children = self.nodes[extra_head.0].children.clone();
            for child in children {
                self.append_child(primary_head, child)?;
            }
            self.remove_child(document_element, extra_head)?;
        }

        Ok(())
    }

    pub(crate) fn document_title(&self) -> String {
        self.query_selector("title")
            .ok()
            .flatten()
            .map(|node| self.text_content(node))
            .unwrap_or_default()
    }

    pub(crate) fn set_document_title(&mut self, title: &str) -> Result<()> {
        let title_node = if let Some(existing_title) = self.query_selector("title")? {
            existing_title
        } else {
            let head = self.ensure_head_element()?;
            self.create_element(head, "title".to_string(), HashMap::new())
        };
        self.set_text_content(title_node, title)
    }

    pub(crate) fn ensure_head_element(&mut self) -> Result<NodeId> {
        if let Some(head) = self.head() {
            return Ok(head);
        }

        let parent = if let Some(document_element) = self.document_element() {
            if self
                .tag_name(document_element)
                .map(|tag| tag.eq_ignore_ascii_case("html"))
                .unwrap_or(false)
            {
                document_element
            } else {
                self.root
            }
        } else {
            self.create_element(self.root, "html".to_string(), HashMap::new())
        };

        Ok(self.create_element(parent, "head".to_string(), HashMap::new()))
    }

    pub(crate) fn matches_selector_chain(&self, node_id: NodeId, steps: &[SelectorPart]) -> bool {
        if steps.is_empty() {
            return false;
        }
        if !self.matches_step(node_id, &steps[steps.len() - 1].step) {
            return false;
        }

        let mut current = node_id;
        for idx in (1..steps.len()).rev() {
            let prev_step = &steps[idx - 1].step;
            let combinator = steps[idx]
                .combinator
                .unwrap_or(SelectorCombinator::Descendant);

            let matched = match combinator {
                SelectorCombinator::Child => {
                    let Some(parent) = self.parent(current) else {
                        return false;
                    };
                    if self.matches_step(parent, prev_step) {
                        Some(parent)
                    } else {
                        None
                    }
                }
                SelectorCombinator::Descendant => {
                    let mut cursor = self.parent(current);
                    let mut found = None;
                    while let Some(parent) = cursor {
                        if self.matches_step(parent, prev_step) {
                            found = Some(parent);
                            break;
                        }
                        cursor = self.parent(parent);
                    }
                    found
                }
                SelectorCombinator::AdjacentSibling => self
                    .previous_element_sibling(current)
                    .filter(|sibling| self.matches_step(*sibling, prev_step)),
                SelectorCombinator::GeneralSibling => {
                    let mut cursor = self.previous_element_sibling(current);
                    let mut found = None;
                    while let Some(sibling) = cursor {
                        if self.matches_step(sibling, prev_step) {
                            found = Some(sibling);
                            break;
                        }
                        cursor = self.previous_element_sibling(sibling);
                    }
                    found
                }
            };

            let Some(matched) = matched else {
                return false;
            };
            current = matched;
        }

        true
    }
}
