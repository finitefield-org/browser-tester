use super::*;

impl Dom {
    pub(crate) fn append_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
        if !self.can_have_children(parent) {
            return Err(Error::ScriptRuntime(
                "appendChild target cannot have children".into(),
            ));
        }
        if child == self.root || child == parent {
            return Err(Error::ScriptRuntime("invalid appendChild node".into()));
        }
        if !self.is_valid_node(child) {
            return Err(Error::ScriptRuntime("appendChild node is invalid".into()));
        }

        if self
            .element(child)
            .is_some_and(|element| element.tag_name.eq_ignore_ascii_case("#document-fragment"))
        {
            let fragment_children = self.nodes[child.0].children.clone();
            for fragment_child in fragment_children {
                self.append_child(parent, fragment_child)?;
            }
            return Ok(());
        }

        let mut cursor = Some(parent);
        while let Some(node) = cursor {
            if node == child {
                return Err(Error::ScriptRuntime(
                    "appendChild would create a cycle".into(),
                ));
            }
            cursor = self.parent(node);
        }

        let old_parent = self.parent(child);
        let child_has_ids = self.subtree_contains_nonempty_id(child);
        if let Some(old_parent) = old_parent {
            self.nodes[old_parent.0].children.retain(|id| *id != child);
        }
        self.nodes[child.0].parent = Some(parent);
        self.nodes[parent.0].children.push(child);
        if child_has_ids {
            self.rebuild_id_index();
        }
        self.sync_selects_affected_by_tree_mutation(parent, child, old_parent)?;
        Ok(())
    }

    pub(crate) fn prepend_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
        let reference = self.nodes[parent.0].children.first().copied();
        if let Some(reference) = reference {
            self.insert_before(parent, child, reference)
        } else {
            self.append_child(parent, child)
        }
    }

    pub(crate) fn insert_before(
        &mut self,
        parent: NodeId,
        child: NodeId,
        reference: NodeId,
    ) -> Result<()> {
        if !self.can_have_children(parent) {
            return Err(Error::ScriptRuntime(
                "insertBefore target cannot have children".into(),
            ));
        }
        if child == self.root || child == parent {
            return Err(Error::ScriptRuntime("invalid insertBefore node".into()));
        }
        if !self.is_valid_node(child) || !self.is_valid_node(reference) {
            return Err(Error::ScriptRuntime("insertBefore node is invalid".into()));
        }
        if self.parent(reference) != Some(parent) {
            return Err(Error::ScriptRuntime(
                "insertBefore reference is not a direct child".into(),
            ));
        }

        if self
            .element(child)
            .is_some_and(|element| element.tag_name.eq_ignore_ascii_case("#document-fragment"))
        {
            let fragment_children = self.nodes[child.0].children.clone();
            for fragment_child in fragment_children {
                self.insert_before(parent, fragment_child, reference)?;
            }
            return Ok(());
        }

        if child == reference {
            return Ok(());
        }

        let mut cursor = Some(parent);
        while let Some(node) = cursor {
            if node == child {
                return Err(Error::ScriptRuntime(
                    "insertBefore would create a cycle".into(),
                ));
            }
            cursor = self.parent(node);
        }

        let old_parent = self.parent(child);
        let child_has_ids = self.subtree_contains_nonempty_id(child);
        if let Some(old_parent) = old_parent {
            self.nodes[old_parent.0].children.retain(|id| *id != child);
        }

        let Some(index) = self.nodes[parent.0]
            .children
            .iter()
            .position(|id| *id == reference)
        else {
            return Err(Error::ScriptRuntime(
                "insertBefore reference is missing".into(),
            ));
        };

        self.nodes[child.0].parent = Some(parent);
        self.nodes[parent.0].children.insert(index, child);
        if child_has_ids {
            self.rebuild_id_index();
        }
        self.sync_selects_affected_by_tree_mutation(parent, child, old_parent)?;
        Ok(())
    }

    pub(crate) fn insert_after(&mut self, target: NodeId, child: NodeId) -> Result<()> {
        let Some(parent) = self.parent(target) else {
            return Ok(());
        };
        let pos = self.nodes[parent.0]
            .children
            .iter()
            .position(|id| *id == target)
            .ok_or_else(|| Error::ScriptRuntime("after target is detached".into()))?;
        let next = self.nodes[parent.0].children.get(pos + 1).copied();
        if let Some(next) = next {
            self.insert_before(parent, child, next)
        } else {
            self.append_child(parent, child)
        }
    }

    pub(crate) fn replace_with(&mut self, target: NodeId, child: NodeId) -> Result<()> {
        let Some(parent) = self.parent(target) else {
            return Ok(());
        };
        if target == child {
            return Ok(());
        }
        self.insert_before(parent, child, target)?;
        self.remove_child(parent, target)
    }

    pub(crate) fn replace_child(
        &mut self,
        parent: NodeId,
        new_child: NodeId,
        old_child: NodeId,
    ) -> Result<()> {
        if !self.can_have_children(parent) {
            return Err(Error::ScriptRuntime(
                "replaceChild target cannot have children".into(),
            ));
        }
        if new_child == self.root || new_child == parent {
            return Err(Error::ScriptRuntime("invalid replaceChild node".into()));
        }
        if !self.is_valid_node(new_child) || !self.is_valid_node(old_child) {
            return Err(Error::ScriptRuntime("replaceChild node is invalid".into()));
        }
        if self.parent(old_child) != Some(parent) {
            return Err(Error::ScriptRuntime(
                "replaceChild target is not a direct child".into(),
            ));
        }
        if new_child == old_child {
            return Ok(());
        }

        let mut cursor = Some(parent);
        while let Some(node) = cursor {
            if node == new_child {
                return Err(Error::ScriptRuntime(
                    "replaceChild would create a cycle".into(),
                ));
            }
            cursor = self.parent(node);
        }

        let moved_from_parent = self.parent(new_child);
        let new_child_has_ids = self.subtree_contains_nonempty_id(new_child);
        let old_child_has_ids = self.subtree_contains_nonempty_id(old_child);
        if let Some(old_parent) = moved_from_parent {
            self.nodes[old_parent.0]
                .children
                .retain(|id| *id != new_child);
        }

        let index = self.nodes[parent.0]
            .children
            .iter()
            .position(|id| *id == old_child)
            .ok_or_else(|| Error::ScriptRuntime("replaceChild target is missing".into()))?;

        self.nodes[new_child.0].parent = Some(parent);
        self.nodes[parent.0].children[index] = new_child;
        self.nodes[old_child.0].parent = None;
        if new_child_has_ids || old_child_has_ids {
            self.rebuild_id_index();
        }
        self.sync_selects_affected_by_tree_mutation(parent, new_child, moved_from_parent)?;
        self.sync_selects_affected_by_tree_mutation(parent, old_child, Some(parent))?;
        Ok(())
    }

    pub(crate) fn insert_adjacent_node(
        &mut self,
        target: NodeId,
        position: InsertAdjacentPosition,
        node: NodeId,
    ) -> Result<()> {
        match position {
            InsertAdjacentPosition::BeforeBegin => {
                if let Some(parent) = self.parent(target) {
                    self.insert_before(parent, node, target)?;
                }
                Ok(())
            }
            InsertAdjacentPosition::AfterBegin => self.prepend_child(target, node),
            InsertAdjacentPosition::BeforeEnd => self.append_child(target, node),
            InsertAdjacentPosition::AfterEnd => self.insert_after(target, node),
        }
    }

    pub(crate) fn remove_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
        if self.parent(child) != Some(parent) {
            return Err(Error::ScriptRuntime(
                "removeChild target is not a direct child".into(),
            ));
        }
        let child_has_ids = self.subtree_contains_nonempty_id(child);
        self.nodes[parent.0].children.retain(|id| *id != child);
        self.nodes[child.0].parent = None;
        if child_has_ids {
            self.rebuild_id_index();
        }
        self.sync_selects_affected_by_tree_mutation(parent, child, Some(parent))?;
        Ok(())
    }

    pub(crate) fn remove_node(&mut self, node: NodeId) -> Result<()> {
        if node == self.root {
            return Err(Error::ScriptRuntime("cannot remove document root".into()));
        }
        let Some(parent) = self.parent(node) else {
            return Ok(());
        };
        self.remove_child(parent, node)
    }

    fn sync_selects_affected_by_tree_mutation(
        &mut self,
        parent: NodeId,
        child: NodeId,
        old_parent: Option<NodeId>,
    ) -> Result<()> {
        let mut selects = Vec::new();
        let mut selected_options = Vec::new();
        self.push_select_node_or_ancestor(parent, &mut selects);
        self.push_select_node_or_ancestor(child, &mut selects);
        self.collect_select_descendants(child, &mut selects);
        self.collect_selected_option_descendants(child, &mut selected_options);
        if let Some(old_parent) = old_parent {
            self.push_select_node_or_ancestor(old_parent, &mut selects);
        }
        for option in selected_options {
            self.sync_select_value_for_option(option)?;
        }
        selects.sort_unstable_by_key(|node| node.0);
        selects.dedup();
        for select in selects {
            self.sync_select_value(select)?;
        }
        Ok(())
    }

    fn push_select_node_or_ancestor(&self, node: NodeId, out: &mut Vec<NodeId>) {
        if !self.is_valid_node(node) {
            return;
        }
        if self
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("select"))
        {
            out.push(node);
        }
        if let Some(select) = self.find_ancestor_by_tag(node, "select") {
            out.push(select);
        }
    }

    fn collect_select_descendants(&self, node: NodeId, out: &mut Vec<NodeId>) {
        if !self.is_valid_node(node) {
            return;
        }
        for child in self.nodes[node.0].children.iter().copied() {
            if self
                .tag_name(child)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("select"))
            {
                out.push(child);
            }
            self.collect_select_descendants(child, out);
        }
    }

    fn collect_selected_option_descendants(&self, node: NodeId, out: &mut Vec<NodeId>) {
        if !self.is_valid_node(node) {
            return;
        }
        if self
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("option"))
            && self.element(node).is_some_and(|element| element.selected)
        {
            out.push(node);
        }
        for child in self.nodes[node.0].children.iter().copied() {
            self.collect_selected_option_descendants(child, out);
        }
    }
}
