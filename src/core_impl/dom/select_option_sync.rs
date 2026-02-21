use super::*;

impl Dom {
    pub(crate) fn initialize_form_control_values(&mut self) -> Result<()> {
        let nodes = self.all_element_nodes();
        for node in nodes {
            let is_textarea = self
                .tag_name(node)
                .map(|tag| tag.eq_ignore_ascii_case("textarea"))
                .unwrap_or(false);
            if is_textarea {
                let text = self.text_content(node);
                let element = self.element_mut(node).ok_or_else(|| {
                    Error::ScriptRuntime("textarea target is not an element".into())
                })?;
                element.value = text;
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
                continue;
            }

            let is_color_input = self
                .element(node)
                .map(is_color_input_element)
                .unwrap_or(false);
            if is_color_input {
                let current = self.attr(node, "value").unwrap_or_default();
                let normalized = normalize_color_input_value(&current);
                let element = self
                    .element_mut(node)
                    .ok_or_else(|| Error::ScriptRuntime("input target is not an element".into()))?;
                element.value = normalized;
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
                continue;
            }

            let is_date_input = self
                .element(node)
                .map(is_date_input_element)
                .unwrap_or(false);
            if is_date_input {
                let current = self.attr(node, "value").unwrap_or_default();
                let normalized = normalize_date_input_value(&current);
                let element = self
                    .element_mut(node)
                    .ok_or_else(|| Error::ScriptRuntime("input target is not an element".into()))?;
                element.value = normalized;
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
                continue;
            }

            let is_datetime_local_input = self
                .element(node)
                .map(is_datetime_local_input_element)
                .unwrap_or(false);
            if is_datetime_local_input {
                let current = self.attr(node, "value").unwrap_or_default();
                let normalized = normalize_datetime_local_input_value(&current);
                let element = self
                    .element_mut(node)
                    .ok_or_else(|| Error::ScriptRuntime("input target is not an element".into()))?;
                element.value = normalized;
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
                continue;
            }

            let is_time_input = self
                .element(node)
                .map(is_time_input_element)
                .unwrap_or(false);
            if is_time_input {
                let current = self.attr(node, "value").unwrap_or_default();
                let normalized = normalize_time_input_value(&current);
                let element = self
                    .element_mut(node)
                    .ok_or_else(|| Error::ScriptRuntime("input target is not an element".into()))?;
                element.value = normalized;
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
                continue;
            }

            let is_number_input = self
                .element(node)
                .map(is_number_input_element)
                .unwrap_or(false);
            if is_number_input {
                let current = self.attr(node, "value").unwrap_or_default();
                let normalized = normalize_number_input_value(&current);
                let element = self
                    .element_mut(node)
                    .ok_or_else(|| Error::ScriptRuntime("input target is not an element".into()))?;
                element.value = normalized;
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
                continue;
            }

            let is_range_input = self
                .element(node)
                .map(is_range_input_element)
                .unwrap_or(false);
            if is_range_input {
                let current = self.attr(node, "value").unwrap_or_default();
                let element = self
                    .element_mut(node)
                    .ok_or_else(|| Error::ScriptRuntime("input target is not an element".into()))?;
                let normalized = normalize_range_input_value(
                    &current,
                    element.attrs.get("min").map(String::as_str),
                    element.attrs.get("max").map(String::as_str),
                    element.attrs.get("step").map(String::as_str),
                    element.attrs.get("value").map(String::as_str),
                );
                element.value = normalized;
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
                continue;
            }

            let is_password_input = self
                .element(node)
                .map(is_password_input_element)
                .unwrap_or(false);
            if is_password_input {
                let current = self.attr(node, "value").unwrap_or_default();
                let normalized = normalize_password_input_value(&current);
                let element = self
                    .element_mut(node)
                    .ok_or_else(|| Error::ScriptRuntime("input target is not an element".into()))?;
                element.value = normalized;
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
                continue;
            }

            let is_file_input = self
                .element(node)
                .map(is_file_input_element)
                .unwrap_or(false);
            if is_file_input {
                let element = self
                    .element_mut(node)
                    .ok_or_else(|| Error::ScriptRuntime("input target is not an element".into()))?;
                element.files.clear();
                element.value = normalize_file_input_value("");
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
                continue;
            }

            let is_image_input = self
                .element(node)
                .map(is_image_input_element)
                .unwrap_or(false);
            if is_image_input {
                let element = self
                    .element_mut(node)
                    .ok_or_else(|| Error::ScriptRuntime("input target is not an element".into()))?;
                element.value = normalize_image_input_value("");
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
                continue;
            }

            let is_select = self
                .tag_name(node)
                .map(|tag| tag.eq_ignore_ascii_case("select"))
                .unwrap_or(false);
            if is_select {
                self.sync_select_value(node)?;
            }
        }
        Ok(())
    }

    pub(crate) fn sync_select_value_for_option(&mut self, option_node: NodeId) -> Result<()> {
        if !self
            .tag_name(option_node)
            .map(|tag| tag.eq_ignore_ascii_case("option"))
            .unwrap_or(false)
        {
            return Ok(());
        }

        let Some(select_node) = self.find_ancestor_by_tag(option_node, "select") else {
            return Ok(());
        };
        self.sync_select_value(select_node)
    }

    pub(crate) fn set_select_value(&mut self, select_node: NodeId, requested: &str) -> Result<()> {
        let tag = self
            .tag_name(select_node)
            .ok_or_else(|| Error::ScriptRuntime("select target is not an element".into()))?;
        if !tag.eq_ignore_ascii_case("select") {
            return Err(Error::ScriptRuntime(
                "set value target is not a select".into(),
            ));
        }

        let mut options = Vec::new();
        self.collect_select_options(select_node, &mut options);

        let mut option_values = Vec::with_capacity(options.len());
        for option in options {
            option_values.push((option, self.option_effective_value(option)?));
        }

        let matched = option_values
            .iter()
            .find(|(_, value)| value == requested)
            .map(|(node, value)| (*node, value.clone()));

        for (option, _) in &option_values {
            let option_element = self
                .element_mut(*option)
                .ok_or_else(|| Error::ScriptRuntime("option target is not an element".into()))?;
            if Some(*option) == matched.as_ref().map(|(node, _)| *node) {
                option_element
                    .attrs
                    .insert("selected".to_string(), "true".to_string());
            } else {
                option_element.attrs.remove("selected");
            }
        }

        let element = self
            .element_mut(select_node)
            .ok_or_else(|| Error::ScriptRuntime("select target is not an element".into()))?;
        element.value = matched.map(|(_, value)| value).unwrap_or_default();
        Ok(())
    }

    pub(crate) fn sync_select_value(&mut self, select_node: NodeId) -> Result<()> {
        let value = self.select_value_from_options(select_node)?;
        let element = self
            .element_mut(select_node)
            .ok_or_else(|| Error::ScriptRuntime("select target is not an element".into()))?;
        element.value = value;
        Ok(())
    }

    pub(crate) fn select_value_from_options(&self, select_node: NodeId) -> Result<String> {
        let tag = self
            .tag_name(select_node)
            .ok_or_else(|| Error::ScriptRuntime("select target is not an element".into()))?;
        if !tag.eq_ignore_ascii_case("select") {
            return Err(Error::ScriptRuntime(
                "select value target is not a select".into(),
            ));
        }

        let mut options = Vec::new();
        self.collect_select_options(select_node, &mut options);
        if options.is_empty() {
            return Ok(String::new());
        }

        let selected = options
            .iter()
            .copied()
            .find(|option| self.attr(*option, "selected").is_some())
            .unwrap_or(options[0]);
        self.option_effective_value(selected)
    }

    pub(crate) fn collect_select_options(&self, node: NodeId, out: &mut Vec<NodeId>) {
        for child in &self.nodes[node.0].children {
            if self
                .tag_name(*child)
                .map(|tag| tag.eq_ignore_ascii_case("option"))
                .unwrap_or(false)
            {
                out.push(*child);
            }
            self.collect_select_options(*child, out);
        }
    }

    pub(crate) fn option_effective_value(&self, option_node: NodeId) -> Result<String> {
        let element = self
            .element(option_node)
            .ok_or_else(|| Error::ScriptRuntime("option target is not an element".into()))?;
        if !element.tag_name.eq_ignore_ascii_case("option") {
            return Err(Error::ScriptRuntime(
                "option target is not an option".into(),
            ));
        }
        if let Some(value) = element.attrs.get("value") {
            return Ok(value.clone());
        }
        Ok(self.text_content(option_node))
    }
}
