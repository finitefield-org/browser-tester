impl Dom {
    pub(super) fn checked(&self, node_id: NodeId) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("checked target is not an element".into()))?;
        Ok(element.checked)
    }

    pub(super) fn set_checked(&mut self, node_id: NodeId, checked: bool) -> Result<()> {
        if checked && is_radio_input(self, node_id) {
            self.uncheck_other_radios_in_group(node_id);
        }
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("checked target is not an element".into()))?;
        element.checked = checked;
        Ok(())
    }

    pub(super) fn uncheck_other_radios_in_group(&mut self, target: NodeId) {
        let target_name = self.attr(target, "name").unwrap_or_default();
        if target_name.is_empty() {
            return;
        }
        let target_form = self.find_ancestor_by_tag(target, "form");

        let all_nodes = self.all_element_nodes();
        for node in all_nodes {
            if node == target {
                continue;
            }
            if !is_radio_input(self, node) {
                continue;
            }
            if self.attr(node, "name").unwrap_or_default() != target_name {
                continue;
            }
            if self.find_ancestor_by_tag(node, "form") != target_form {
                continue;
            }
            if let Some(element) = self.element_mut(node) {
                element.checked = false;
            }
        }
    }

    pub(super) fn normalize_radio_groups(&mut self) -> Result<()> {
        let all_nodes = self.all_element_nodes();
        for node in all_nodes {
            if !is_radio_input(self, node) {
                continue;
            }
            if self.attr(node, "checked").is_some() {
                self.set_checked(node, true)?;
            }
        }
        Ok(())
    }

    pub(super) fn disabled(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.disabled).unwrap_or(false)
    }

    pub(super) fn readonly(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.readonly).unwrap_or(false)
    }

    pub(super) fn required(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.required).unwrap_or(false)
    }

    pub(super) fn attr(&self, node_id: NodeId, name: &str) -> Option<String> {
        self.element(node_id)
            .and_then(|e| e.attrs.get(name).cloned())
    }

    pub(super) fn has_attr(&self, node_id: NodeId, name: &str) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("hasAttribute target is not an element".into()))?;
        Ok(element.attrs.contains_key(&name.to_ascii_lowercase()))
    }

    pub(super) fn set_attr(&mut self, node_id: NodeId, name: &str, value: &str) -> Result<()> {
        let old_id = if name.eq_ignore_ascii_case("id") {
            self.element(node_id)
                .and_then(|element| element.attrs.get("id").cloned())
        } else {
            None
        };
        let connected = self.is_connected(node_id);
        let (is_option, lowered) = {
            let element = self.element_mut(node_id).ok_or_else(|| {
                Error::ScriptRuntime("setAttribute target is not an element".into())
            })?;
            let is_option = element.tag_name.eq_ignore_ascii_case("option");
            let lowered = name.to_ascii_lowercase();
            element.attrs.insert(lowered.clone(), value.to_string());

            if lowered == "value" {
                element.value = value.to_string();
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
            } else if lowered == "checked" {
                element.checked = true;
            } else if lowered == "disabled" {
                element.disabled = true;
            } else if lowered == "readonly" {
                element.readonly = true;
            } else if lowered == "required" {
                element.required = true;
            }
            (is_option, lowered)
        };

        if lowered == "id" && connected {
            if let Some(old) = old_id {
                self.unindex_id(&old, node_id);
            }
            if !value.is_empty() {
                self.index_id(value, node_id);
            }
        }

        if is_option && (lowered == "selected" || lowered == "value") {
            self.sync_select_value_for_option(node_id)?;
        }

        Ok(())
    }

    pub(super) fn remove_attr(&mut self, node_id: NodeId, name: &str) -> Result<()> {
        let lowered = name.to_ascii_lowercase();
        let old_id = if lowered == "id" {
            self.element(node_id)
                .and_then(|element| element.attrs.get("id").cloned())
        } else {
            None
        };
        let connected = self.is_connected(node_id);
        let is_option = {
            let element = self.element_mut(node_id).ok_or_else(|| {
                Error::ScriptRuntime("removeAttribute target is not an element".into())
            })?;
            let is_option = element.tag_name.eq_ignore_ascii_case("option");
            element.attrs.remove(&lowered);

            if lowered == "value" {
                element.value.clear();
                element.selection_start = 0;
                element.selection_end = 0;
                element.selection_direction = "none".to_string();
            } else if lowered == "checked" {
                element.checked = false;
            } else if lowered == "disabled" {
                element.disabled = false;
            } else if lowered == "readonly" {
                element.readonly = false;
            } else if lowered == "required" {
                element.required = false;
            }
            is_option
        };

        if lowered == "id" && connected {
            if let Some(old) = old_id {
                self.unindex_id(&old, node_id);
            }
        }

        if is_option && (lowered == "selected" || lowered == "value") {
            self.sync_select_value_for_option(node_id)?;
        }

        Ok(())
    }

    pub(super) fn append_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
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

        // Prevent cycles: parent must not be inside child's subtree.
        let mut cursor = Some(parent);
        while let Some(node) = cursor {
            if node == child {
                return Err(Error::ScriptRuntime(
                    "appendChild would create a cycle".into(),
                ));
            }
            cursor = self.parent(node);
        }

        if let Some(old_parent) = self.parent(child) {
            self.nodes[old_parent.0].children.retain(|id| *id != child);
        }
        self.nodes[child.0].parent = Some(parent);
        self.nodes[parent.0].children.push(child);
        self.rebuild_id_index();
        Ok(())
    }

    pub(super) fn prepend_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
        let reference = self.nodes[parent.0].children.first().copied();
        if let Some(reference) = reference {
            self.insert_before(parent, child, reference)
        } else {
            self.append_child(parent, child)
        }
    }

    pub(super) fn insert_before(
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
        if child == reference {
            return Ok(());
        }

        // Prevent cycles: parent must not be inside child's subtree.
        let mut cursor = Some(parent);
        while let Some(node) = cursor {
            if node == child {
                return Err(Error::ScriptRuntime(
                    "insertBefore would create a cycle".into(),
                ));
            }
            cursor = self.parent(node);
        }

        if let Some(old_parent) = self.parent(child) {
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
        self.rebuild_id_index();
        Ok(())
    }

    pub(super) fn insert_after(&mut self, target: NodeId, child: NodeId) -> Result<()> {
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

    pub(super) fn replace_with(&mut self, target: NodeId, child: NodeId) -> Result<()> {
        let Some(parent) = self.parent(target) else {
            return Ok(());
        };
        if target == child {
            return Ok(());
        }
        self.insert_before(parent, child, target)?;
        self.remove_child(parent, target)
    }

    pub(super) fn insert_adjacent_node(
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

    pub(super) fn remove_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
        if self.parent(child) != Some(parent) {
            return Err(Error::ScriptRuntime(
                "removeChild target is not a direct child".into(),
            ));
        }
        self.nodes[parent.0].children.retain(|id| *id != child);
        self.nodes[child.0].parent = None;
        self.rebuild_id_index();
        Ok(())
    }

    pub(super) fn remove_node(&mut self, node: NodeId) -> Result<()> {
        if node == self.root {
            return Err(Error::ScriptRuntime("cannot remove document root".into()));
        }
        let Some(parent) = self.parent(node) else {
            return Ok(());
        };
        self.remove_child(parent, node)
    }

}
