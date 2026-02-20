use super::*;

impl Dom {
    pub(crate) fn value(&self, node_id: NodeId) -> Result<String> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("value target is not an element".into()))?;
        if is_checkbox_or_radio_input_element(element) && !element.attrs.contains_key("value") {
            return Ok("on".to_string());
        }
        Ok(element.value.clone())
    }

    pub(crate) fn set_value(&mut self, node_id: NodeId, value: &str) -> Result<()> {
        if self
            .tag_name(node_id)
            .map(|tag| tag.eq_ignore_ascii_case("select"))
            .unwrap_or(false)
        {
            return self.set_select_value(node_id, value);
        }

        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("value target is not an element".into()))?;
        if is_checkbox_or_radio_input_element(element) {
            element.attrs.insert("value".to_string(), value.to_string());
        }
        element.value = value.to_string();
        let len = element.value.chars().count();
        element.selection_start = len;
        element.selection_end = len;
        element.selection_direction = "none".to_string();
        Ok(())
    }

    pub(crate) fn indeterminate(&self, node_id: NodeId) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("indeterminate target is not an element".into()))?;
        Ok(element.indeterminate)
    }

    pub(crate) fn set_indeterminate(&mut self, node_id: NodeId, indeterminate: bool) -> Result<()> {
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("indeterminate target is not an element".into()))?;
        element.indeterminate = indeterminate;
        Ok(())
    }

    pub(crate) fn custom_validity_message(&self, node_id: NodeId) -> Result<String> {
        let element = self.element(node_id).ok_or_else(|| {
            Error::ScriptRuntime("custom validity target is not an element".into())
        })?;
        Ok(element.custom_validity_message.clone())
    }

    pub(crate) fn set_custom_validity_message(
        &mut self,
        node_id: NodeId,
        message: &str,
    ) -> Result<()> {
        let element = self.element_mut(node_id).ok_or_else(|| {
            Error::ScriptRuntime("custom validity target is not an element".into())
        })?;
        element.custom_validity_message = message.to_string();
        Ok(())
    }

    pub(crate) fn selection_start(&self, node_id: NodeId) -> Result<usize> {
        let element = self.element(node_id).ok_or_else(|| {
            Error::ScriptRuntime("selectionStart target is not an element".into())
        })?;
        Ok(element.selection_start)
    }

    pub(crate) fn selection_end(&self, node_id: NodeId) -> Result<usize> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("selectionEnd target is not an element".into()))?;
        Ok(element.selection_end)
    }

    pub(crate) fn selection_direction(&self, node_id: NodeId) -> Result<String> {
        let element = self.element(node_id).ok_or_else(|| {
            Error::ScriptRuntime("selectionDirection target is not an element".into())
        })?;
        Ok(element.selection_direction.clone())
    }

    pub(crate) fn set_selection_range(
        &mut self,
        node_id: NodeId,
        start: usize,
        end: usize,
        direction: &str,
    ) -> Result<()> {
        let element = self.element_mut(node_id).ok_or_else(|| {
            Error::ScriptRuntime("setSelectionRange target is not an element".into())
        })?;
        let len = element.value.chars().count();
        let clamped_start = start.min(len);
        let clamped_end = end.min(len);
        let (selection_start, selection_end) = if clamped_end < clamped_start {
            (clamped_end, clamped_end)
        } else {
            (clamped_start, clamped_end)
        };
        element.selection_start = selection_start;
        element.selection_end = selection_end;
        element.selection_direction = direction.to_string();
        Ok(())
    }
}
