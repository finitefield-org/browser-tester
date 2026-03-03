use super::*;

impl Dom {
    pub(crate) fn text_content(&self, node_id: NodeId) -> String {
        match &self.nodes[node_id.0].node_type {
            NodeType::Document => {
                let mut out = String::new();
                for child in &self.nodes[node_id.0].children {
                    out.push_str(&self.text_content(*child));
                }
                out
            }
            NodeType::Element(element) => {
                if element.tag_name.eq_ignore_ascii_case("br") {
                    return "\n".to_string();
                }
                let mut out = String::new();
                for child in &self.nodes[node_id.0].children {
                    out.push_str(&self.text_content(*child));
                }
                out
            }
            NodeType::Text(text) => text.clone(),
        }
    }

    pub(crate) fn set_text_content(&mut self, node_id: NodeId, value: &str) -> Result<()> {
        match &mut self.nodes[node_id.0].node_type {
            NodeType::Document => {
                // Per DOM behavior, setting textContent on Document is a no-op.
                return Ok(());
            }
            NodeType::Text(text) => {
                *text = value.to_string();
                return Ok(());
            }
            NodeType::Element(_) => {}
        }

        let old_children = std::mem::take(&mut self.nodes[node_id.0].children);
        for child in old_children {
            self.nodes[child.0].parent = None;
        }
        if !value.is_empty() {
            self.create_text(node_id, value.to_string());
        }
        self.rebuild_id_index();
        Ok(())
    }

    pub(crate) fn inner_html(&self, node_id: NodeId) -> Result<String> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "innerHTML target is not an element".into(),
            ));
        }
        let mut out = String::new();
        for child in &self.nodes[node_id.0].children {
            out.push_str(&self.dump_node(*child));
        }
        Ok(out)
    }

    pub(crate) fn outer_html(&self, node_id: NodeId) -> Result<String> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "outerHTML target is not an element".into(),
            ));
        }
        Ok(self.dump_node(node_id))
    }

    pub(crate) fn set_inner_html(&mut self, node_id: NodeId, html: &str) -> Result<()> {
        self.set_inner_html_with_sanitize(node_id, html, true)
    }

    pub(crate) fn set_inner_html_unsafe(&mut self, node_id: NodeId, html: &str) -> Result<()> {
        self.set_inner_html_with_sanitize(node_id, html, false)
    }

    fn set_inner_html_with_sanitize(
        &mut self,
        node_id: NodeId,
        html: &str,
        sanitize: bool,
    ) -> Result<()> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "innerHTML target is not an element".into(),
            ));
        }

        let ParseOutput { dom: fragment, .. } = parse_html(html)?;

        let old_children = std::mem::take(&mut self.nodes[node_id.0].children);
        for child in old_children {
            self.nodes[child.0].parent = None;
        }

        let children = fragment.nodes[fragment.root.0].children.clone();
        for child in children {
            let _ = self.clone_subtree_from_dom(&fragment, child, Some(node_id), sanitize)?;
        }

        self.rebuild_id_index();
        Ok(())
    }

    pub(crate) fn set_outer_html(&mut self, node_id: NodeId, html: &str) -> Result<()> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "outerHTML target is not an element".into(),
            ));
        }

        let Some(parent) = self.parent(node_id) else {
            // Per DOM behavior, setting outerHTML on a detached element is a no-op.
            return Ok(());
        };

        if matches!(self.nodes[parent.0].node_type, NodeType::Document)
            && self
                .tag_name(node_id)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("html"))
        {
            return Err(Error::ScriptRuntime(
                "NoModificationAllowedError: outerHTML cannot modify a direct Document child"
                    .into(),
            ));
        }

        let index = self.nodes[parent.0]
            .children
            .iter()
            .position(|id| *id == node_id)
            .ok_or_else(|| Error::ScriptRuntime("outerHTML target is detached".into()))?;

        let ParseOutput { dom: fragment, .. } = parse_html(html)?;

        self.nodes[parent.0].children.remove(index);
        self.nodes[node_id.0].parent = None;

        let mut insert_at = index;
        let children = fragment.nodes[fragment.root.0].children.clone();
        for child in children {
            if let Some(cloned) = self.clone_subtree_from_dom(&fragment, child, None, true)? {
                self.nodes[cloned.0].parent = Some(parent);
                self.nodes[parent.0].children.insert(insert_at, cloned);
                insert_at += 1;
            }
        }

        self.rebuild_id_index();
        Ok(())
    }

    pub(crate) fn insert_adjacent_html(
        &mut self,
        target: NodeId,
        position: InsertAdjacentPosition,
        html: &str,
    ) -> Result<()> {
        let ParseOutput { dom: fragment, .. } = parse_html(html)?;

        let mut children = fragment.nodes[fragment.root.0].children.clone();
        if matches!(position, InsertAdjacentPosition::AfterBegin) {
            children.reverse();
        }

        for child in children {
            if let Some(node) = self.clone_subtree_from_dom(&fragment, child, None, true)? {
                self.insert_adjacent_node(target, position, node)?;
            }
        }
        Ok(())
    }

    pub(crate) fn clone_subtree_from_dom(
        &mut self,
        source: &Dom,
        source_node: NodeId,
        parent: Option<NodeId>,
        sanitize: bool,
    ) -> Result<Option<NodeId>> {
        let node_type = match &source.nodes[source_node.0].node_type {
            NodeType::Document => {
                return Err(Error::ScriptRuntime(
                    "cannot clone document node into innerHTML target".into(),
                ));
            }
            NodeType::Element(element) => {
                if sanitize && should_strip_inner_html_element(&element.tag_name) {
                    return Ok(None);
                }
                let mut clone = element.clone();
                if sanitize {
                    sanitize_inner_html_element_attrs(&mut clone);
                }
                NodeType::Element(clone)
            }
            NodeType::Text(text) => NodeType::Text(text.clone()),
        };

        let node = self.create_node(parent, node_type);
        for child in &source.nodes[source_node.0].children {
            let _ = self.clone_subtree_from_dom(source, *child, Some(node), sanitize)?;
        }
        Ok(Some(node))
    }
}
