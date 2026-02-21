use super::*;

impl Dom {
    pub(crate) fn dataset_get(&self, node_id: NodeId, key: &str) -> Result<String> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "dataset target is not an element".into(),
            ));
        }
        let name = dataset_key_to_attr_name(key);
        Ok(self.attr(node_id, &name).unwrap_or_default())
    }

    pub(crate) fn dataset_set(&mut self, node_id: NodeId, key: &str, value: &str) -> Result<()> {
        let name = dataset_key_to_attr_name(key);
        self.set_attr(node_id, &name, value)
    }

    pub(crate) fn style_get(&self, node_id: NodeId, key: &str) -> Result<String> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("style target is not an element".into()))?;
        let name = js_prop_to_css_name(key);
        let decls = parse_style_declarations(element.attrs.get("style").map(String::as_str));
        Ok(decls
            .iter()
            .find(|(prop, _)| prop == &name)
            .map(|(_, value)| value.clone())
            .unwrap_or_default())
    }

    pub(crate) fn style_set(&mut self, node_id: NodeId, key: &str, value: &str) -> Result<()> {
        let name = js_prop_to_css_name(key);
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("style target is not an element".into()))?;

        let mut decls = parse_style_declarations(element.attrs.get("style").map(String::as_str));
        if let Some(pos) = decls.iter().position(|(prop, _)| prop == &name) {
            if value.is_empty() {
                decls.remove(pos);
            } else {
                decls[pos].1 = value.to_string();
            }
        } else if !value.is_empty() {
            decls.push((name, value.to_string()));
        }

        if decls.is_empty() {
            // Keep an empty style attribute to match CSSStyleDeclaration behavior.
            element.attrs.insert("style".to_string(), String::new());
        } else {
            element
                .attrs
                .insert("style".to_string(), serialize_style_declarations(&decls));
        }

        Ok(())
    }

    pub(crate) fn offset_left(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetLeft target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn offset_top(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetTop target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn offset_width(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetWidth target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn offset_height(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetHeight target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn scroll_width(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollWidth target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn scroll_height(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollHeight target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn scroll_left(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollLeft target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn scroll_top(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollTop target is not an element".into(),
            ));
        }
        Ok(0)
    }
}
