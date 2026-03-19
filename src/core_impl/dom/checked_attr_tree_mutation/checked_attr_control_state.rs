use super::*;

impl Dom {
    pub(crate) fn control_form_owner(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(form_id) = self.attr(node_id, "form").filter(|id| !id.is_empty()) {
            if let Some(form_node) = self.by_id(&form_id) {
                if self
                    .tag_name(form_node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("form"))
                {
                    return Some(form_node);
                }
            }
        }
        self.find_ancestor_by_tag(node_id, "form")
    }

    pub(crate) fn set_checked_state(
        &mut self,
        node_id: NodeId,
        checked: bool,
        dirty_state: Option<bool>,
    ) -> Result<()> {
        if checked && is_radio_input(self, node_id) {
            self.uncheck_other_radios_in_group(node_id, dirty_state);
        }
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("checked target is not an element".into()))?;
        element.checked = checked;
        if let Some(dirty) = dirty_state {
            element.checked_dirty = dirty;
        }
        Ok(())
    }

    pub(crate) fn checked(&self, node_id: NodeId) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("checked target is not an element".into()))?;
        Ok(element.checked)
    }

    pub(crate) fn set_checked(&mut self, node_id: NodeId, checked: bool) -> Result<()> {
        self.set_checked_state(node_id, checked, Some(true))
    }

    pub(crate) fn uncheck_other_radios_in_group(
        &mut self,
        target: NodeId,
        dirty_state: Option<bool>,
    ) {
        let target_name = self.attr(target, "name").unwrap_or_default();
        if target_name.is_empty() {
            return;
        }
        let target_form = self.control_form_owner(target);

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
            if self.control_form_owner(node) != target_form {
                continue;
            }
            if let Some(element) = self.element_mut(node) {
                element.checked = false;
                if let Some(dirty) = dirty_state {
                    element.checked_dirty = dirty;
                }
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
                self.set_checked_state(node, true, Some(false))?;
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
        let Some(element) = self.element(node_id) else {
            return false;
        };
        if element.disabled {
            return true;
        }
        if element.tag_name.eq_ignore_ascii_case("option") {
            let mut cursor = self.parent(node_id);
            while let Some(parent) = cursor {
                if self
                    .tag_name(parent)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("optgroup"))
                    && self.attr(parent, "disabled").is_some()
                {
                    return true;
                }
                if self.tag_name(parent).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("select") || tag.eq_ignore_ascii_case("datalist")
                }) {
                    break;
                }
                cursor = self.parent(parent);
            }
        }
        false
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
                let current_is_dirty = element.dirty_value && uses_dirty_value_state(element);
                let is_checkbox_or_radio = is_checkbox_or_radio_input_element(element);
                let default_value = if is_file_input_element(element) {
                    normalize_file_input_value("")
                } else if is_image_input_element(element) {
                    normalize_image_input_value(value)
                } else if is_color_input_element(element) {
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
                element.default_value = default_value.clone();
                if is_file_input_element(element) {
                    if value.is_empty() {
                        element.files.clear();
                        Self::write_current_value_state(
                            element,
                            normalize_file_input_value(value),
                            Some(false),
                        );
                    }
                } else if is_image_input_element(element) {
                    Self::write_current_value_state(
                        element,
                        normalize_image_input_value(value),
                        Some(false),
                    );
                } else if is_checkbox_or_radio || !current_is_dirty {
                    Self::write_current_value_state(element, default_value, Some(false));
                }
                if element.tag_name.eq_ignore_ascii_case("progress") {
                    element.indeterminate = false;
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
                element.default_checked = true;
            } else if lowered == "disabled" {
                element.disabled = true;
            } else if lowered == "readonly" {
                element.readonly = true;
            } else if lowered == "required" {
                element.required = true;
            }

            if is_details && (lowered == "open" || lowered == "name") {
                details_open_group_to_enforce = element
                    .attrs
                    .get("name")
                    .cloned()
                    .filter(|name| !name.is_empty());
            }
            (is_option, lowered)
        };

        if lowered == "checked" {
            let should_sync_current = self
                .element(node_id)
                .map(|element| !element.checked_dirty)
                .unwrap_or(false);
            if should_sync_current {
                self.set_checked_state(node_id, true, Some(false))?;
            }
        } else if lowered == "type"
            && self.attr(node_id, "checked").is_some()
            && is_radio_input(self, node_id)
        {
            self.set_checked_state(node_id, true, Some(false))?;
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
            self.rebuild_id_index();
        }

        if lowered == "selected" && is_option {
            if let Some(element) = self.element_mut(node_id) {
                let should_sync_current = !element.selected_dirty;
                element.default_selected = true;
                if should_sync_current {
                    element.selected = true;
                }
            }
        }

        if is_option && (lowered == "selected" || lowered == "value") {
            self.sync_select_value_for_option(node_id)?;
        }
        if matches!(lowered.as_str(), "name" | "form")
            && self
                .element(node_id)
                .map(|element| element.checked)
                .unwrap_or(false)
            && is_radio_input(self, node_id)
        {
            self.uncheck_other_radios_in_group(node_id, None);
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
        let connected = self.is_connected(node_id);
        let is_option = {
            let element = self.element_mut(node_id).ok_or_else(|| {
                Error::ScriptRuntime("removeAttribute target is not an element".into())
            })?;
            let is_option = element.tag_name.eq_ignore_ascii_case("option");
            element.attrs.remove(&lowered);

            if lowered == "value" {
                let current_is_dirty = element.dirty_value && uses_dirty_value_state(element);
                let is_checkbox_or_radio = is_checkbox_or_radio_input_element(element);
                let default_value = if is_color_input_element(element) {
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
                element.default_value = default_value.clone();
                if is_checkbox_or_radio || !current_is_dirty {
                    Self::write_current_value_state(element, default_value, Some(false));
                }
                if element.tag_name.eq_ignore_ascii_case("progress") {
                    element.indeterminate = true;
                }
            } else if lowered == "checked" {
                element.default_checked = false;
            } else if lowered == "disabled" {
                element.disabled = false;
            } else if lowered == "readonly" {
                element.readonly = false;
            } else if lowered == "required" {
                element.required = false;
            }
            is_option
        };

        if lowered == "checked" {
            let should_sync_current = self
                .element(node_id)
                .map(|element| !element.checked_dirty)
                .unwrap_or(false);
            if should_sync_current {
                self.set_checked_state(node_id, false, Some(false))?;
            }
        }

        if lowered == "id" && connected {
            self.rebuild_id_index();
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

        if lowered == "selected" && is_option {
            if let Some(element) = self.element_mut(node_id) {
                let should_sync_current = !element.selected_dirty;
                element.default_selected = false;
                if should_sync_current {
                    element.selected = false;
                }
            }
        }

        if is_option && (lowered == "selected" || lowered == "value") {
            self.sync_select_value_for_option(node_id)?;
        }

        if matches!(lowered.as_str(), "name" | "form")
            && self
                .element(node_id)
                .map(|element| element.checked)
                .unwrap_or(false)
            && is_radio_input(self, node_id)
        {
            self.uncheck_other_radios_in_group(node_id, None);
        }

        Ok(())
    }
}
