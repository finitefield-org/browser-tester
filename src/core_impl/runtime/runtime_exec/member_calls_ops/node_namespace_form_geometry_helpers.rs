use super::*;

impl Harness {
    pub(crate) fn local_name_from_qualified_name(name: &str) -> &str {
        name.rsplit_once(':')
            .map(|(_, local_name)| local_name)
            .unwrap_or(name)
    }

    pub(crate) fn attribute_namespace_uri_for_qualified_name(
        &self,
        owner: NodeId,
        qualified_name: &str,
    ) -> Option<String> {
        const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";
        const XMLNS_NS: &str = "http://www.w3.org/2000/xmlns/";
        const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

        let element = self.dom.element(owner)?;
        let Some((prefix, _)) = qualified_name.split_once(':') else {
            return if qualified_name.eq_ignore_ascii_case("xmlns") {
                Some(XMLNS_NS.to_string())
            } else {
                None
            };
        };

        if prefix.eq_ignore_ascii_case("xml") {
            return Some(XML_NS.to_string());
        }
        if prefix.eq_ignore_ascii_case("xmlns") {
            return Some(XMLNS_NS.to_string());
        }

        let xmlns_attr_name = format!("xmlns:{prefix}");
        if let Some(uri) = element.attrs.get(&xmlns_attr_name) {
            return Some(uri.clone());
        }
        if let Some((_, uri)) = element
            .attrs
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&xmlns_attr_name))
        {
            return Some(uri.clone());
        }

        if prefix.eq_ignore_ascii_case("xlink") {
            return Some(XLINK_NS.to_string());
        }

        None
    }

    pub(crate) fn get_attribute_node_ns_value(
        &self,
        node: NodeId,
        namespace_uri: Option<&str>,
        local_name: &str,
    ) -> Value {
        let Some(element) = self.dom.element(node) else {
            return Value::Null;
        };

        let mut matches = element
            .attrs
            .iter()
            .filter_map(|(qualified_name, value)| {
                let candidate_local_name = Self::local_name_from_qualified_name(qualified_name);
                if !candidate_local_name.eq_ignore_ascii_case(local_name) {
                    return None;
                }
                let candidate_namespace =
                    self.attribute_namespace_uri_for_qualified_name(node, qualified_name);
                let namespace_matches = match (namespace_uri, candidate_namespace.as_deref()) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                };
                if !namespace_matches {
                    return None;
                }
                Some((qualified_name.clone(), value.clone()))
            })
            .collect::<Vec<_>>();

        matches.sort_by(|(left, _), (right, _)| left.cmp(right));
        matches
            .into_iter()
            .next()
            .map(|(name, value)| Self::new_attr_object_value(&name, &value, Some(node)))
            .unwrap_or(Value::Null)
    }

    pub(crate) fn get_attribute_ns_value(
        &self,
        node: NodeId,
        namespace_uri: Option<&str>,
        local_name: &str,
    ) -> Value {
        let Some(element) = self.dom.element(node) else {
            return Value::Null;
        };

        let mut matches = element
            .attrs
            .iter()
            .filter_map(|(qualified_name, value)| {
                let candidate_local_name = Self::local_name_from_qualified_name(qualified_name);
                if !candidate_local_name.eq_ignore_ascii_case(local_name) {
                    return None;
                }
                let candidate_namespace =
                    self.attribute_namespace_uri_for_qualified_name(node, qualified_name);
                let namespace_matches = match (namespace_uri, candidate_namespace.as_deref()) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                };
                if !namespace_matches {
                    return None;
                }
                Some((qualified_name.clone(), value.clone()))
            })
            .collect::<Vec<_>>();

        matches.sort_by(|(left, _), (right, _)| left.cmp(right));
        matches
            .into_iter()
            .next()
            .map(|(_, value)| Value::String(value))
            .unwrap_or(Value::Null)
    }

    pub(crate) fn has_attribute_ns_value(
        &self,
        node: NodeId,
        namespace_uri: Option<&str>,
        local_name: &str,
    ) -> bool {
        !matches!(
            self.get_attribute_ns_value(node, namespace_uri, local_name),
            Value::Null
        )
    }

    pub(crate) fn remove_attribute_ns(
        &mut self,
        node: NodeId,
        namespace_uri: Option<&str>,
        local_name: &str,
    ) -> Result<()> {
        let mut matches = {
            let Some(element) = self.dom.element(node) else {
                return Err(Error::ScriptRuntime(
                    "removeAttributeNS target is not an element".into(),
                ));
            };
            element
                .attrs
                .keys()
                .filter_map(|qualified_name| {
                    let candidate_local_name = Self::local_name_from_qualified_name(qualified_name);
                    if !candidate_local_name.eq_ignore_ascii_case(local_name) {
                        return None;
                    }
                    let candidate_namespace =
                        self.attribute_namespace_uri_for_qualified_name(node, qualified_name);
                    let namespace_matches = match (namespace_uri, candidate_namespace.as_deref()) {
                        (None, None) => true,
                        (Some(expected), Some(actual)) => expected == actual,
                        _ => false,
                    };
                    if !namespace_matches {
                        return None;
                    }
                    Some(qualified_name.clone())
                })
                .collect::<Vec<_>>()
        };

        matches.sort();
        if let Some(name) = matches.into_iter().next() {
            self.dom.remove_attr(node, &name)?;
        }
        Ok(())
    }

    pub(crate) fn get_bounding_client_rect_value(&self, node: NodeId) -> Result<Value> {
        let mut left = self
            .dom
            .offset_left(node)?
            .saturating_sub(self.dom_runtime.document_scroll_x);
        let mut top = self
            .dom
            .offset_top(node)?
            .saturating_sub(self.dom_runtime.document_scroll_y);
        let width = self.dom.offset_width(node)?;
        let height = self.dom.offset_height(node)?;

        if self.node_uses_sticky_position(node) {
            if let Some(sticky_left) = self.sticky_inset_px(node, "left") {
                left = left.max(sticky_left);
            }
            if let Some(sticky_top) = self.sticky_inset_px(node, "top") {
                top = top.max(sticky_top);
            }
        }

        let right = left.saturating_add(width);
        let bottom = top.saturating_add(height);

        Ok(Self::new_dom_rect_value(
            left, top, right, bottom, width, height,
        ))
    }

    fn node_uses_sticky_position(&self, node: NodeId) -> bool {
        self.computed_style_property_value(node, None, "position")
            .map(|value| value.trim().eq_ignore_ascii_case("sticky"))
            .unwrap_or(false)
    }

    fn sticky_inset_px(&self, node: NodeId, property: &str) -> Option<i64> {
        let value = self
            .computed_style_property_value(node, None, property)
            .ok()?;
        Self::parse_css_length_to_px(&value)
    }

    fn parse_css_length_to_px(raw: &str) -> Option<i64> {
        let normalized = raw.trim().to_ascii_lowercase();
        if normalized.is_empty() || normalized == "auto" {
            return None;
        }

        let px = if let Some(stripped) = normalized.strip_suffix("px") {
            stripped.trim().parse::<f64>().ok()?
        } else if let Some(stripped) = normalized.strip_suffix("rem") {
            stripped.trim().parse::<f64>().ok()? * 16.0
        } else if let Some(stripped) = normalized.strip_suffix("em") {
            stripped.trim().parse::<f64>().ok()? * 16.0
        } else {
            normalized.parse::<f64>().ok()?
        };

        px.is_finite().then_some(px.round() as i64)
    }

    fn node_has_client_rects(&self, node: NodeId) -> bool {
        let Some(element) = self.dom.element(node) else {
            return false;
        };
        if !self.dom.is_connected(node) {
            return false;
        }
        if element.tag_name.eq_ignore_ascii_case("area") {
            return false;
        }
        if element.attrs.contains_key("hidden") {
            return false;
        }
        let display = parse_style_declarations(element.attrs.get("style").map(String::as_str))
            .into_iter()
            .find(|(name, _)| name == "display")
            .map(|(_, value)| value.trim().to_ascii_lowercase())
            .unwrap_or_default();
        display != "none"
    }

    pub(crate) fn get_client_rects_value(&self, node: NodeId) -> Result<Value> {
        if !self.node_has_client_rects(node) {
            return Ok(Self::new_dom_rect_list_value(Vec::new()));
        }
        let rect = self.get_bounding_client_rect_value(node)?;
        Ok(Self::new_dom_rect_list_value(vec![rect]))
    }

    pub(crate) fn is_select_element(&self, node: NodeId) -> bool {
        self.dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("select"))
    }

    pub(crate) fn select_option_nodes(&self, select_node: NodeId) -> Vec<NodeId> {
        if !self.is_select_element(select_node) {
            return Vec::new();
        }
        let mut options = Vec::new();
        self.dom.collect_select_options(select_node, &mut options);
        options
    }

    pub(crate) fn select_selected_index_value(&self, select_node: NodeId) -> i64 {
        let options = self.select_option_nodes(select_node);
        if options.is_empty() {
            return -1;
        }
        options
            .iter()
            .position(|option| {
                self.dom
                    .element(*option)
                    .map(|element| element.selected)
                    .unwrap_or(false)
            })
            .map(|index| index as i64)
            .unwrap_or(-1)
    }

    pub(crate) fn select_selected_option_nodes(&self, select_node: NodeId) -> Vec<NodeId> {
        self.select_option_nodes(select_node)
            .iter()
            .copied()
            .filter(|option| {
                self.dom
                    .element(*option)
                    .map(|element| element.selected)
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn select_named_item(&self, select_node: NodeId, key: &str) -> Option<NodeId> {
        if key.is_empty() {
            return None;
        }
        let options = self.select_option_nodes(select_node);
        options
            .iter()
            .copied()
            .find(|option| self.dom.attr(*option, "id").is_some_and(|id| id == key))
            .or_else(|| {
                options.iter().copied().find(|option| {
                    self.dom
                        .attr(*option, "name")
                        .is_some_and(|name| name == key)
                })
            })
    }

    pub(crate) fn set_select_selected_index(
        &mut self,
        select_node: NodeId,
        selected_index: i64,
    ) -> Result<()> {
        if !self.is_select_element(select_node) {
            return Err(Error::ScriptRuntime(
                "selectedIndex target is not a select".into(),
            ));
        }

        let options = self.select_option_nodes(select_node);
        let selected_position = usize::try_from(selected_index)
            .ok()
            .filter(|index| *index < options.len());
        let selected_value = selected_position
            .map(|index| self.dom.option_effective_value(options[index]))
            .transpose()?
            .unwrap_or_default();

        for (index, option) in options.iter().enumerate() {
            self.dom.set_option_selected_state(
                *option,
                Some(index) == selected_position,
                Some(true),
            )?;
        }

        let select_element = self
            .dom
            .element_mut(select_node)
            .ok_or_else(|| Error::ScriptRuntime("selectedIndex target is not an element".into()))?;
        select_element.value = selected_value;
        Ok(())
    }

    pub(crate) fn set_select_length(&mut self, select_node: NodeId, next_len: usize) -> Result<()> {
        if !self.is_select_element(select_node) {
            return Err(Error::ScriptRuntime("length target is not a select".into()));
        }

        let options = self.select_option_nodes(select_node);
        if next_len < options.len() {
            for option in options.iter().skip(next_len).rev() {
                self.dom.remove_node(*option)?;
            }
        } else if next_len > options.len() {
            for _ in options.len()..next_len {
                let option = self.dom.create_detached_element("option".to_string());
                self.dom.append_child(select_node, option)?;
            }
        }
        self.dom.sync_select_value(select_node)
    }

    pub(crate) fn select_size_property_value(&self, select_node: NodeId) -> i64 {
        self.dom
            .attr(select_node, "size")
            .and_then(|raw| Self::parse_non_negative_int(&raw))
            .filter(|size| *size > 0)
            .unwrap_or_else(|| {
                if self.dom.attr(select_node, "multiple").is_some() {
                    4
                } else {
                    1
                }
            })
    }

    pub(crate) fn set_select_size_property_value(
        &mut self,
        select_node: NodeId,
        value: &Value,
    ) -> Result<()> {
        if !self.is_select_element(select_node) {
            return Ok(());
        }
        let next = Self::value_to_i64(value).max(0);
        self.dom.set_attr(select_node, "size", &next.to_string())
    }

    pub(crate) fn select_type_property_value(&self, select_node: NodeId) -> String {
        if self.dom.attr(select_node, "multiple").is_some() {
            "select-multiple".to_string()
        } else {
            "select-one".to_string()
        }
    }

    pub(crate) fn select_will_validate(&self, select_node: NodeId) -> bool {
        self.is_select_element(select_node) && !self.is_effectively_disabled(select_node)
    }

    pub(crate) fn normalized_button_type(&self, button_node: NodeId) -> String {
        let Some(tag) = self.dom.tag_name(button_node) else {
            return "submit".to_string();
        };
        if !tag.eq_ignore_ascii_case("button") {
            return "submit".to_string();
        }
        let Some(raw) = self.dom.attr(button_node, "type") else {
            return "submit".to_string();
        };
        if raw.eq_ignore_ascii_case("reset") {
            "reset".to_string()
        } else if raw.eq_ignore_ascii_case("button") {
            "button".to_string()
        } else {
            "submit".to_string()
        }
    }

    pub(crate) fn button_will_validate(&self, button_node: NodeId) -> bool {
        let Some(tag) = self.dom.tag_name(button_node) else {
            return false;
        };
        if !tag.eq_ignore_ascii_case("button") {
            return false;
        }
        if self.is_effectively_disabled(button_node) {
            return false;
        }
        if self
            .dom
            .find_ancestor_by_tag(button_node, "datalist")
            .is_some()
        {
            return false;
        }
        self.normalized_button_type(button_node) == "submit"
    }

    pub(crate) fn labels_for_control_node(&self, control: NodeId) -> Vec<NodeId> {
        if !self.is_labelable_control(control) {
            return Vec::new();
        }
        self.dom
            .all_element_nodes()
            .into_iter()
            .filter(|node| {
                self.dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("label"))
            })
            .filter(|label| self.resolve_label_control(*label) == Some(control))
            .collect::<Vec<_>>()
    }
}
