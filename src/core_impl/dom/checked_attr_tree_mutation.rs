use super::*;

impl Dom {
    pub(crate) fn checked(&self, node_id: NodeId) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("checked target is not an element".into()))?;
        Ok(element.checked)
    }

    pub(crate) fn set_checked(&mut self, node_id: NodeId, checked: bool) -> Result<()> {
        if checked && is_radio_input(self, node_id) {
            self.uncheck_other_radios_in_group(node_id);
        }
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("checked target is not an element".into()))?;
        element.checked = checked;
        Ok(())
    }

    pub(crate) fn uncheck_other_radios_in_group(&mut self, target: NodeId) {
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

    pub(crate) fn normalize_radio_groups(&mut self) -> Result<()> {
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

    pub(crate) fn normalize_named_details_groups(&mut self) -> Result<()> {
        let mut seen_open_names = std::collections::HashSet::new();
        for node in self.all_element_nodes() {
            if !self
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
            {
                continue;
            }
            let name = self.attr(node, "name").unwrap_or_default();
            if name.is_empty() || self.attr(node, "open").is_none() {
                continue;
            }
            if !seen_open_names.insert(name.clone()) {
                self.remove_attr(node, "open")?;
            }
        }
        Ok(())
    }

    pub(crate) fn close_other_named_details_in_group(
        &mut self,
        target: NodeId,
        group_name: &str,
    ) -> Result<()> {
        if group_name.is_empty() {
            return Ok(());
        }
        for node in self.all_element_nodes() {
            if node == target {
                continue;
            }
            if !self
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
            {
                continue;
            }
            if self.attr(node, "name").as_deref() != Some(group_name) {
                continue;
            }
            if self.attr(node, "open").is_some() {
                self.remove_attr(node, "open")?;
            }
        }
        Ok(())
    }

    pub(crate) fn disabled(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.disabled).unwrap_or(false)
    }

    pub(crate) fn readonly(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.readonly).unwrap_or(false)
    }

    pub(crate) fn required(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.required).unwrap_or(false)
    }

    pub(crate) fn attr(&self, node_id: NodeId, name: &str) -> Option<String> {
        self.element(node_id)
            .and_then(|e| e.attrs.get(name).cloned())
    }

    pub(crate) fn has_attr(&self, node_id: NodeId, name: &str) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("hasAttribute target is not an element".into()))?;
        Ok(element.attrs.contains_key(&name.to_ascii_lowercase()))
    }

    pub(crate) fn set_attr(&mut self, node_id: NodeId, name: &str, value: &str) -> Result<()> {
        let old_id = if name.eq_ignore_ascii_case("id") {
            self.element(node_id)
                .and_then(|element| element.attrs.get("id").cloned())
        } else {
            None
        };
        let connected = self.is_connected(node_id);
        let mut details_open_group_to_enforce = None;
        let (is_option, lowered) = {
            let element = self.element_mut(node_id).ok_or_else(|| {
                Error::ScriptRuntime("setAttribute target is not an element".into())
            })?;
            let is_option = element.tag_name.eq_ignore_ascii_case("option");
            let is_details = element.tag_name.eq_ignore_ascii_case("details");
            let was_file_input = is_file_input_element(element);
            let lowered = name.to_ascii_lowercase();
            element.attrs.insert(lowered.clone(), value.to_string());

            if lowered == "value" {
                if is_file_input_element(element) {
                    if value.is_empty() {
                        element.files.clear();
                        element.value = normalize_file_input_value(value);
                        let len = element.value.chars().count();
                        element.selection_start = len;
                        element.selection_end = len;
                        element.selection_direction = "none".to_string();
                    }
                } else if is_image_input_element(element) {
                    element.value = normalize_image_input_value(value);
                    let len = element.value.chars().count();
                    element.selection_start = len;
                    element.selection_end = len;
                    element.selection_direction = "none".to_string();
                } else {
                    element.value = if is_color_input_element(element) {
                        normalize_color_input_value(value)
                    } else if is_date_input_element(element) {
                        normalize_date_input_value(value)
                    } else if is_datetime_local_input_element(element) {
                        normalize_datetime_local_input_value(value)
                    } else if is_time_input_element(element) {
                        normalize_time_input_value(value)
                    } else if is_number_input_element(element) {
                        normalize_number_input_value(value)
                    } else if is_range_input_element(element) {
                        normalize_range_input_value(
                            value,
                            element.attrs.get("min").map(String::as_str),
                            element.attrs.get("max").map(String::as_str),
                            element.attrs.get("step").map(String::as_str),
                            element.attrs.get("value").map(String::as_str),
                        )
                    } else if is_password_input_element(element) {
                        normalize_password_input_value(value)
                    } else {
                        value.to_string()
                    };
                    let len = element.value.chars().count();
                    element.selection_start = len;
                    element.selection_end = len;
                    element.selection_direction = "none".to_string();
                }
            } else if lowered == "type" {
                if is_color_input_element(element) {
                    let raw_value = element
                        .attrs
                        .get("value")
                        .cloned()
                        .unwrap_or_else(|| element.value.clone());
                    element.value = normalize_color_input_value(&raw_value);
                    let len = element.value.chars().count();
                    element.selection_start = len;
                    element.selection_end = len;
                    element.selection_direction = "none".to_string();
                } else if is_date_input_element(element) {
                    let raw_value = element
                        .attrs
                        .get("value")
                        .cloned()
                        .unwrap_or_else(|| element.value.clone());
                    element.value = normalize_date_input_value(&raw_value);
                    let len = element.value.chars().count();
                    element.selection_start = len;
                    element.selection_end = len;
                    element.selection_direction = "none".to_string();
                } else if is_datetime_local_input_element(element) {
                    let raw_value = element
                        .attrs
                        .get("value")
                        .cloned()
                        .unwrap_or_else(|| element.value.clone());
                    element.value = normalize_datetime_local_input_value(&raw_value);
                    let len = element.value.chars().count();
                    element.selection_start = len;
                    element.selection_end = len;
                    element.selection_direction = "none".to_string();
                } else if is_time_input_element(element) {
                    let raw_value = element
                        .attrs
                        .get("value")
                        .cloned()
                        .unwrap_or_else(|| element.value.clone());
                    element.value = normalize_time_input_value(&raw_value);
                    let len = element.value.chars().count();
                    element.selection_start = len;
                    element.selection_end = len;
                    element.selection_direction = "none".to_string();
                } else if is_number_input_element(element) {
                    let raw_value = element
                        .attrs
                        .get("value")
                        .cloned()
                        .unwrap_or_else(|| element.value.clone());
                    element.value = normalize_number_input_value(&raw_value);
                    let len = element.value.chars().count();
                    element.selection_start = len;
                    element.selection_end = len;
                    element.selection_direction = "none".to_string();
                } else if is_range_input_element(element) {
                    let raw_value = element
                        .attrs
                        .get("value")
                        .cloned()
                        .unwrap_or_else(|| element.value.clone());
                    element.value = normalize_range_input_value(
                        &raw_value,
                        element.attrs.get("min").map(String::as_str),
                        element.attrs.get("max").map(String::as_str),
                        element.attrs.get("step").map(String::as_str),
                        element.attrs.get("value").map(String::as_str),
                    );
                    let len = element.value.chars().count();
                    element.selection_start = len;
                    element.selection_end = len;
                    element.selection_direction = "none".to_string();
                } else if is_password_input_element(element) {
                    let raw_value = element
                        .attrs
                        .get("value")
                        .cloned()
                        .unwrap_or_else(|| element.value.clone());
                    element.value = normalize_password_input_value(&raw_value);
                    let len = element.value.chars().count();
                    element.selection_start = len;
                    element.selection_end = len;
                    element.selection_direction = "none".to_string();
                } else if is_image_input_element(element) {
                    element.value = normalize_image_input_value("");
                    let len = element.value.chars().count();
                    element.selection_start = len;
                    element.selection_end = len;
                    element.selection_direction = "none".to_string();
                } else if is_file_input_element(element) {
                    element.files.clear();
                    element.value = normalize_file_input_value("");
                    let len = element.value.chars().count();
                    element.selection_start = len;
                    element.selection_end = len;
                    element.selection_direction = "none".to_string();
                } else if was_file_input {
                    element.files.clear();
                }
            } else if lowered == "checked" {
                element.checked = true;
            } else if lowered == "disabled" {
                element.disabled = true;
            } else if lowered == "readonly" {
                element.readonly = true;
            } else if lowered == "required" {
                element.required = true;
            }

            if is_details && (lowered == "open" || lowered == "name") {
                details_open_group_to_enforce =
                    element.attrs.get("name").cloned().filter(|name| !name.is_empty());
            }
            (is_option, lowered)
        };

        if lowered == "checked" {
            self.set_checked(node_id, true)?;
        } else if lowered == "type"
            && self.attr(node_id, "checked").is_some()
            && is_radio_input(self, node_id)
        {
            self.set_checked(node_id, true)?;
        }
        if matches!(lowered.as_str(), "min" | "max" | "step")
            && self
                .element(node_id)
                .map(is_range_input_element)
                .unwrap_or(false)
        {
            let next_value = {
                let element = self.element(node_id).ok_or_else(|| {
                    Error::ScriptRuntime("setAttribute target is not an element".into())
                })?;
                normalize_range_input_value(
                    &element.value,
                    element.attrs.get("min").map(String::as_str),
                    element.attrs.get("max").map(String::as_str),
                    element.attrs.get("step").map(String::as_str),
                    element.attrs.get("value").map(String::as_str),
                )
            };
            let element = self.element_mut(node_id).ok_or_else(|| {
                Error::ScriptRuntime("setAttribute target is not an element".into())
            })?;
            element.value = next_value;
            let len = element.value.chars().count();
            element.selection_start = len;
            element.selection_end = len;
            element.selection_direction = "none".to_string();
        }

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
        if let Some(group_name) = details_open_group_to_enforce {
            if self.attr(node_id, "open").is_some() {
                self.close_other_named_details_in_group(node_id, &group_name)?;
            }
        }

        Ok(())
    }

    pub(crate) fn remove_attr(&mut self, node_id: NodeId, name: &str) -> Result<()> {
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
                element.value = if is_color_input_element(element) {
                    normalize_color_input_value("")
                } else if is_date_input_element(element) {
                    normalize_date_input_value("")
                } else if is_datetime_local_input_element(element) {
                    normalize_datetime_local_input_value("")
                } else if is_time_input_element(element) {
                    normalize_time_input_value("")
                } else if is_range_input_element(element) {
                    normalize_range_input_value(
                        "",
                        element.attrs.get("min").map(String::as_str),
                        element.attrs.get("max").map(String::as_str),
                        element.attrs.get("step").map(String::as_str),
                        element.attrs.get("value").map(String::as_str),
                    )
                } else if is_image_input_element(element) {
                    normalize_image_input_value("")
                } else if is_file_input_element(element) {
                    element.files.clear();
                    normalize_file_input_value("")
                } else {
                    String::new()
                };
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
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
        if matches!(lowered.as_str(), "min" | "max" | "step")
            && self
                .element(node_id)
                .map(is_range_input_element)
                .unwrap_or(false)
        {
            let next_value = {
                let element = self.element(node_id).ok_or_else(|| {
                    Error::ScriptRuntime("removeAttribute target is not an element".into())
                })?;
                normalize_range_input_value(
                    &element.value,
                    element.attrs.get("min").map(String::as_str),
                    element.attrs.get("max").map(String::as_str),
                    element.attrs.get("step").map(String::as_str),
                    element.attrs.get("value").map(String::as_str),
                )
            };
            let element = self.element_mut(node_id).ok_or_else(|| {
                Error::ScriptRuntime("removeAttribute target is not an element".into())
            })?;
            element.value = next_value;
            let len = element.value.chars().count();
            element.selection_start = len;
            element.selection_end = len;
            element.selection_direction = "none".to_string();
        }

        if is_option && (lowered == "selected" || lowered == "value") {
            self.sync_select_value_for_option(node_id)?;
        }

        Ok(())
    }

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
        self.nodes[parent.0].children.retain(|id| *id != child);
        self.nodes[child.0].parent = None;
        self.rebuild_id_index();
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
}
