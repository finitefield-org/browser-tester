use super::*;

#[derive(Debug, Clone)]
enum SetHtmlUnsafeSanitizer {
    None,
    Default,
    Config(SetHtmlUnsafeSanitizerConfig),
}

#[derive(Debug, Clone, Default)]
struct SetHtmlUnsafeSanitizerConfig {
    allowed_elements: Option<std::collections::HashSet<String>>,
    removed_elements: std::collections::HashSet<String>,
}

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
            .into_iter()
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

    fn parse_set_html_unsafe_tag_set(value: &Value) -> Result<std::collections::HashSet<String>> {
        match value {
            Value::Array(items) => Ok(items
                .borrow()
                .iter()
                .map(Value::as_string)
                .map(|entry| entry.to_ascii_lowercase())
                .collect()),
            _ => Err(Error::ScriptRuntime(
                "TypeError: Failed to execute 'setHTMLUnsafe': sanitizer config entries must be arrays"
                    .into(),
            )),
        }
    }

    fn parse_set_html_unsafe_sanitizer(
        &self,
        options: Option<&Value>,
    ) -> Result<SetHtmlUnsafeSanitizer> {
        let Some(options) = options else {
            return Ok(SetHtmlUnsafeSanitizer::None);
        };
        let Value::Object(entries) = options else {
            return Err(Error::ScriptRuntime(
                "TypeError: Failed to execute 'setHTMLUnsafe': options must be an object".into(),
            ));
        };
        let entries = entries.borrow();
        let Some(sanitizer) = Self::object_get_entry(&entries, "sanitizer") else {
            return Ok(SetHtmlUnsafeSanitizer::None);
        };
        match sanitizer {
            Value::Undefined => Ok(SetHtmlUnsafeSanitizer::None),
            Value::String(value) => {
                if value == "default" {
                    Ok(SetHtmlUnsafeSanitizer::Default)
                } else {
                    Err(Error::ScriptRuntime(
                        "TypeError: Failed to execute 'setHTMLUnsafe': options.sanitizer string must be 'default'"
                            .into(),
                    ))
                }
            }
            Value::Object(config_entries) => {
                let config_entries = config_entries.borrow();
                let allowed_raw = Self::object_get_entry(&config_entries, "elements");
                let removed_raw = Self::object_get_entry(&config_entries, "removeElements");
                if allowed_raw.is_some() && removed_raw.is_some() {
                    return Err(Error::ScriptRuntime(
                        "TypeError: Failed to execute 'setHTMLUnsafe': sanitizer config cannot include both elements and removeElements"
                            .into(),
                    ));
                }
                let allowed_elements = allowed_raw
                    .as_ref()
                    .map(Self::parse_set_html_unsafe_tag_set)
                    .transpose()?;
                let removed_elements = removed_raw
                    .as_ref()
                    .map(Self::parse_set_html_unsafe_tag_set)
                    .transpose()?
                    .unwrap_or_default();
                Ok(SetHtmlUnsafeSanitizer::Config(
                    SetHtmlUnsafeSanitizerConfig {
                        allowed_elements,
                        removed_elements,
                    },
                ))
            }
            _ => Err(Error::ScriptRuntime(
                "TypeError: Failed to execute 'setHTMLUnsafe': options.sanitizer must be a Sanitizer, SanitizerConfig, or 'default'"
                    .into(),
            )),
        }
    }

    fn apply_set_html_unsafe_config_sanitizer_to_subtree(
        &mut self,
        node: NodeId,
        config: &SetHtmlUnsafeSanitizerConfig,
    ) -> Result<()> {
        let mut stack = self.dom.nodes[node.0].children.clone();
        while let Some(current) = stack.pop() {
            let remove_current = self.dom.tag_name(current).is_some_and(|tag| {
                let tag = tag.to_ascii_lowercase();
                config.removed_elements.contains(&tag)
                    || config
                        .allowed_elements
                        .as_ref()
                        .is_some_and(|allowed| !allowed.contains(&tag))
            });

            if remove_current {
                if let Some(parent) = self.dom.parent(current) {
                    self.dom.remove_child(parent, current)?;
                }
                continue;
            }

            let mut children = self.dom.nodes[current.0].children.clone();
            children.reverse();
            stack.extend(children);
        }
        Ok(())
    }

    fn parse_declarative_shadow_root_mode(value: &str) -> Option<ShadowRootMode> {
        match value {
            "open" => Some(ShadowRootMode::Open),
            "closed" => Some(ShadowRootMode::Closed),
            _ => None,
        }
    }

    fn apply_single_declarative_shadow_root_template(&mut self, template: NodeId) -> Result<()> {
        if !self
            .dom
            .tag_name(template)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("template"))
        {
            return Ok(());
        }
        let Some(mode_value) = self.dom.attr(template, "shadowrootmode") else {
            return Ok(());
        };
        let Some(mode) = Self::parse_declarative_shadow_root_mode(&mode_value.to_ascii_lowercase())
        else {
            return Ok(());
        };
        let Some(host) = self.dom.parent(template) else {
            return Ok(());
        };
        if self.dom.element(host).is_none() || self.is_document_fragment_node(host) {
            return Ok(());
        }

        if let Some(existing) = self.dom_runtime.shadow_roots.get(&host).copied() {
            self.dom.remove_child(host, template)?;
            self.dom.append_child(existing.root, template)?;
            return Ok(());
        }
        if !self.can_attach_shadow_root_to_host(host) {
            return Ok(());
        }

        let root = self
            .dom
            .create_detached_element("#document-fragment".to_string());
        self.dom_runtime.shadow_roots.insert(
            host,
            ShadowRootRecord {
                root,
                mode,
                serializable: false,
            },
        );

        let children = self.dom.nodes[template.0].children.clone();
        for child in children {
            self.dom.remove_child(template, child)?;
            self.dom.append_child(root, child)?;
        }
        self.dom.remove_child(host, template)?;
        Ok(())
    }

    fn apply_declarative_shadow_roots_in_subtree(&mut self, node: NodeId) -> Result<()> {
        let mut templates = Vec::new();
        let mut stack = self.dom.nodes[node.0].children.clone();
        stack.reverse();
        while let Some(current) = stack.pop() {
            if self
                .dom
                .tag_name(current)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("template"))
                && self.dom.attr(current, "shadowrootmode").is_some()
            {
                templates.push(current);
            }
            let mut children = self.dom.nodes[current.0].children.clone();
            children.reverse();
            stack.extend(children);
        }

        for template in templates {
            self.apply_single_declarative_shadow_root_template(template)?;
        }
        Ok(())
    }

    pub(crate) fn eval_set_html_unsafe_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
            return Err(Error::ScriptRuntime(
                "setHTMLUnsafe requires one or two arguments".into(),
            ));
        }
        if self.dom.element(node).is_none() || self.is_document_fragment_node(node) {
            return Err(Error::ScriptRuntime(
                "TypeError: setHTMLUnsafe target must be an Element".into(),
            ));
        }

        let input = evaluated_args[0].as_string();
        let sanitizer = self.parse_set_html_unsafe_sanitizer(evaluated_args.get(1))?;
        match sanitizer {
            SetHtmlUnsafeSanitizer::None => self.dom.set_inner_html_unsafe(node, &input)?,
            SetHtmlUnsafeSanitizer::Default => self.dom.set_inner_html(node, &input)?,
            SetHtmlUnsafeSanitizer::Config(config) => {
                self.dom.set_inner_html_unsafe(node, &input)?;
                self.apply_set_html_unsafe_config_sanitizer_to_subtree(node, &config)?;
            }
        }
        self.apply_declarative_shadow_roots_in_subtree(node)?;
        Ok(Value::Undefined)
    }

    fn hierarchy_request_error() -> Error {
        Error::ScriptRuntime(
            "HierarchyRequestError: The operation would yield an incorrect node tree.".into(),
        )
    }

    fn is_document_fragment_node(&self, node: NodeId) -> bool {
        self.dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("#document-fragment"))
    }

    fn collect_appendable_document_nodes(&self, node: NodeId, out: &mut Vec<NodeId>) {
        if self.is_document_fragment_node(node) {
            let children = self.dom.nodes[node.0].children.clone();
            for child in children {
                self.collect_appendable_document_nodes(child, out);
            }
            return;
        }
        out.push(node);
    }

    fn shadow_root_mode_from_attach_options(&self, options: &Value) -> Result<ShadowRootMode> {
        let Value::Object(entries) = options else {
            return Err(Error::ScriptRuntime(
                "TypeError: attachShadow options must be an object".into(),
            ));
        };

        let mode_value = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "mode")
        };
        let mode_value = mode_value.ok_or_else(|| {
            Error::ScriptRuntime("TypeError: attachShadow options.mode is required".into())
        })?;
        match mode_value.as_string().as_str() {
            "open" => Ok(ShadowRootMode::Open),
            "closed" => Ok(ShadowRootMode::Closed),
            _ => Err(Error::ScriptRuntime(
                "TypeError: attachShadow options.mode must be 'open' or 'closed'".into(),
            )),
        }
    }

    fn shadow_root_serializable_from_attach_options(&self, options: &Value) -> bool {
        let Value::Object(entries) = options else {
            return false;
        };
        let entries = entries.borrow();
        Self::object_get_entry(&entries, "serializable")
            .map(|value| value.truthy())
            .unwrap_or(false)
    }

    fn is_autonomous_custom_element_name(tag_name: &str) -> bool {
        tag_name.contains('-')
    }

    fn can_attach_shadow_root_to_host(&self, node: NodeId) -> bool {
        let Some(element) = self.dom.element(node) else {
            return false;
        };

        if element.namespace_uri.as_deref() != Some("http://www.w3.org/1999/xhtml") {
            return false;
        }

        let tag_name = element.tag_name.to_ascii_lowercase();
        if Self::is_autonomous_custom_element_name(&tag_name) {
            return true;
        }

        matches!(
            tag_name.as_str(),
            "article"
                | "aside"
                | "blockquote"
                | "body"
                | "div"
                | "footer"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "main"
                | "nav"
                | "p"
                | "section"
                | "span"
        )
    }

    pub(crate) fn shadow_root_property_value(&self, node: NodeId) -> Value {
        self.dom_runtime
            .shadow_roots
            .get(&node)
            .and_then(|record| {
                if record.mode == ShadowRootMode::Open {
                    Some(record.root)
                } else {
                    None
                }
            })
            .map(Value::Node)
            .unwrap_or(Value::Null)
    }

    pub(crate) fn attach_shadow_root(&mut self, node: NodeId, options: &Value) -> Result<NodeId> {
        if self.dom.element(node).is_none() {
            return Err(Error::ScriptRuntime(
                "attachShadow target is not an element".into(),
            ));
        }
        if !self.can_attach_shadow_root_to_host(node) {
            return Err(Error::ScriptRuntime(
                "NotSupportedError: shadow root cannot be attached to this element".into(),
            ));
        }
        if self.dom_runtime.shadow_roots.contains_key(&node) {
            return Err(Error::ScriptRuntime(
                "NotSupportedError: shadow root already attached".into(),
            ));
        }
        let mode = self.shadow_root_mode_from_attach_options(options)?;
        let serializable = self.shadow_root_serializable_from_attach_options(options);
        let root = self
            .dom
            .create_detached_element("#document-fragment".to_string());
        self.dom_runtime.shadow_roots.insert(
            node,
            ShadowRootRecord {
                root,
                mode,
                serializable,
            },
        );
        Ok(root)
    }

    fn parse_get_html_options(&self, options: Option<&Value>) -> (bool, Vec<NodeId>) {
        let Some(Value::Object(entries)) = options else {
            return (false, Vec::new());
        };
        let entries = entries.borrow();
        let include_serializable = Self::object_get_entry(&entries, "serializableShadowRoots")
            .map(|value| value.truthy())
            .unwrap_or(false);
        let explicit_shadow_roots = match Self::object_get_entry(&entries, "shadowRoots") {
            Some(Value::Array(values)) => values
                .borrow()
                .iter()
                .filter_map(|value| match value {
                    Value::Node(node) => Some(*node),
                    _ => None,
                })
                .filter(|node| {
                    self.dom_runtime
                        .shadow_roots
                        .values()
                        .any(|record| record.root == *node)
                })
                .collect(),
            _ => Vec::new(),
        };
        (include_serializable, explicit_shadow_roots)
    }

    fn dump_node_for_get_html(
        &self,
        node_id: NodeId,
        include_serializable_shadow_roots: bool,
        explicit_shadow_roots: &[NodeId],
    ) -> String {
        match &self.dom.nodes[node_id.0].node_type {
            NodeType::Document => {
                let mut out = String::new();
                for child in &self.dom.nodes[node_id.0].children {
                    out.push_str(&self.dump_node_for_get_html(
                        *child,
                        include_serializable_shadow_roots,
                        explicit_shadow_roots,
                    ));
                }
                out
            }
            NodeType::Text(text) => escape_html_text_for_serialization(text),
            NodeType::Element(element) => {
                let mut out = String::new();
                out.push('<');
                out.push_str(&element.tag_name);
                let mut attrs = element.attrs.iter().collect::<Vec<_>>();
                attrs.sort_by(|(left, _), (right, _)| left.cmp(right));
                for (key, value) in attrs {
                    out.push(' ');
                    out.push_str(key);
                    out.push_str("=\"");
                    out.push_str(&escape_html_attr_for_serialization(value));
                    out.push('"');
                }
                out.push('>');

                if crate::core_impl::html::is_void_tag(&element.tag_name) {
                    return out;
                }

                if let Some(record) = self.dom_runtime.shadow_roots.get(&node_id) {
                    let include_shadow_root = explicit_shadow_roots.contains(&record.root)
                        || (include_serializable_shadow_roots && record.serializable);
                    if include_shadow_root {
                        let mode = match record.mode {
                            ShadowRootMode::Open => "open",
                            ShadowRootMode::Closed => "closed",
                        };
                        out.push_str("<template shadowrootmode=\"");
                        out.push_str(mode);
                        out.push_str("\">");
                        for child in &self.dom.nodes[record.root.0].children {
                            out.push_str(&self.dump_node_for_get_html(
                                *child,
                                include_serializable_shadow_roots,
                                explicit_shadow_roots,
                            ));
                        }
                        out.push_str("</template>");
                    }
                }

                let raw_text_container = element.tag_name.eq_ignore_ascii_case("script")
                    || element.tag_name.eq_ignore_ascii_case("style");
                for child in &self.dom.nodes[node_id.0].children {
                    if raw_text_container {
                        match &self.dom.nodes[child.0].node_type {
                            NodeType::Text(text) => out.push_str(text),
                            _ => out.push_str(&self.dump_node_for_get_html(
                                *child,
                                include_serializable_shadow_roots,
                                explicit_shadow_roots,
                            )),
                        }
                    } else {
                        out.push_str(&self.dump_node_for_get_html(
                            *child,
                            include_serializable_shadow_roots,
                            explicit_shadow_roots,
                        ));
                    }
                }
                out.push_str("</");
                out.push_str(&element.tag_name);
                out.push('>');
                out
            }
        }
    }

    pub(crate) fn element_get_html_value(
        &self,
        node: NodeId,
        options: Option<&Value>,
    ) -> Result<Value> {
        if self.dom.element(node).is_none() {
            return Err(Error::ScriptRuntime(
                "getHTML target is not an element".into(),
            ));
        }
        let (include_serializable_shadow_roots, explicit_shadow_roots) =
            self.parse_get_html_options(options);
        let mut out = String::new();
        for child in &self.dom.nodes[node.0].children {
            out.push_str(&self.dump_node_for_get_html(
                *child,
                include_serializable_shadow_roots,
                &explicit_shadow_roots,
            ));
        }
        Ok(Value::String(out))
    }

    pub(crate) fn eval_document_append_call(
        &mut self,
        document_node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if matches!(
            self.dom
                .nodes
                .get(document_node.0)
                .map(|node| &node.node_type),
            Some(NodeType::Document)
        ) {
            let mut nodes = Vec::new();
            for value in evaluated_args {
                match value {
                    Value::Node(node) => self.collect_appendable_document_nodes(*node, &mut nodes),
                    other => {
                        let text = self.dom.create_detached_text(other.as_string());
                        nodes.push(text);
                    }
                }
            }

            let mut existing_elements = self.dom.nodes[document_node.0]
                .children
                .iter()
                .copied()
                .filter(|child| {
                    self.dom.element(*child).is_some_and(|element| {
                        !element.tag_name.eq_ignore_ascii_case("#document-fragment")
                    })
                })
                .count() as i64;

            for node in &nodes {
                if self.dom.parent(*node) == Some(document_node)
                    && self.dom.element(*node).is_some_and(|element| {
                        !element.tag_name.eq_ignore_ascii_case("#document-fragment")
                    })
                {
                    existing_elements -= 1;
                }
            }

            let mut appended_elements = 0i64;
            for node in &nodes {
                match self.dom.nodes.get(node.0).map(|entry| &entry.node_type) {
                    Some(NodeType::Document) | Some(NodeType::Text(_)) => {
                        return Err(Self::hierarchy_request_error());
                    }
                    Some(NodeType::Element(element))
                        if !element.tag_name.eq_ignore_ascii_case("#document-fragment") =>
                    {
                        appended_elements += 1;
                    }
                    Some(NodeType::Element(_)) => {}
                    None => return Err(Self::hierarchy_request_error()),
                }
            }

            if existing_elements + appended_elements > 1 {
                return Err(Self::hierarchy_request_error());
            }

            for node in nodes {
                self.dom.append_child(document_node, node)?;
            }
            return Ok(Value::Undefined);
        }

        for value in evaluated_args {
            let node = match value {
                Value::Node(node) => *node,
                other => self.dom.create_detached_text(other.as_string()),
            };
            self.dom.append_child(document_node, node)?;
        }
        Ok(Value::Undefined)
    }

    pub(crate) fn eval_node_after_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if self.dom.parent(node).is_none() {
            return Ok(Value::Undefined);
        }

        let mut insertion_anchor = node;
        for value in evaluated_args {
            let (child, new_anchor) = match value {
                Value::Node(child) => {
                    let new_anchor = if self.is_document_fragment_node(*child) {
                        self.dom.nodes[child.0].children.last().copied()
                    } else {
                        Some(*child)
                    };
                    (*child, new_anchor)
                }
                other => {
                    let text = self.dom.create_detached_text(other.as_string());
                    (text, Some(text))
                }
            };
            self.dom.insert_after(insertion_anchor, child)?;
            if let Some(new_anchor) = new_anchor {
                if self.dom.parent(new_anchor).is_some() {
                    insertion_anchor = new_anchor;
                }
            }
        }

        Ok(Value::Undefined)
    }

    pub(crate) fn eval_node_before_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        let Some(parent) = self.dom.parent(node) else {
            return Ok(Value::Undefined);
        };

        for value in evaluated_args {
            let child = match value {
                Value::Node(child) => *child,
                other => self.dom.create_detached_text(other.as_string()),
            };
            self.dom.insert_before(parent, child, node)?;
        }

        Ok(Value::Undefined)
    }

    pub(crate) fn eval_node_prepend_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if matches!(
            self.dom.nodes.get(node.0).map(|entry| &entry.node_type),
            Some(NodeType::Document)
        ) {
            let mut nodes = Vec::new();
            for value in evaluated_args {
                match value {
                    Value::Node(candidate) => {
                        self.collect_appendable_document_nodes(*candidate, &mut nodes)
                    }
                    other => nodes.push(self.dom.create_detached_text(other.as_string())),
                }
            }

            let mut existing_elements = self.dom.nodes[node.0]
                .children
                .iter()
                .copied()
                .filter(|child| {
                    self.dom.element(*child).is_some_and(|element| {
                        !element.tag_name.eq_ignore_ascii_case("#document-fragment")
                    })
                })
                .count() as i64;

            for candidate in &nodes {
                if self.dom.parent(*candidate) == Some(node)
                    && self.dom.element(*candidate).is_some_and(|element| {
                        !element.tag_name.eq_ignore_ascii_case("#document-fragment")
                    })
                {
                    existing_elements -= 1;
                }
            }

            let mut prepended_elements = 0i64;
            for candidate in &nodes {
                match self
                    .dom
                    .nodes
                    .get(candidate.0)
                    .map(|entry| &entry.node_type)
                {
                    Some(NodeType::Document) | Some(NodeType::Text(_)) => {
                        return Err(Self::hierarchy_request_error());
                    }
                    Some(NodeType::Element(element))
                        if !element.tag_name.eq_ignore_ascii_case("#document-fragment") =>
                    {
                        prepended_elements += 1;
                    }
                    Some(NodeType::Element(_)) => {}
                    None => return Err(Self::hierarchy_request_error()),
                }
            }

            if existing_elements + prepended_elements > 1 {
                return Err(Self::hierarchy_request_error());
            }

            for candidate in nodes.into_iter().rev() {
                self.dom.prepend_child(node, candidate)?;
            }
            return Ok(Value::Undefined);
        }

        for value in evaluated_args.iter().rev() {
            let child = match value {
                Value::Node(child) => *child,
                other => self.dom.create_detached_text(other.as_string()),
            };
            self.dom.prepend_child(node, child)?;
        }

        Ok(Value::Undefined)
    }

    pub(crate) fn eval_node_replace_children_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        let mut replacements = Vec::with_capacity(evaluated_args.len());
        for value in evaluated_args {
            let child = match value {
                Value::Node(child) => *child,
                other => self.dom.create_detached_text(other.as_string()),
            };
            let Some(child_node) = self.dom.nodes.get(child.0) else {
                return Err(Self::hierarchy_request_error());
            };
            if matches!(child_node.node_type, NodeType::Document)
                || child == node
                || self.dom.is_descendant_of(node, child)
            {
                return Err(Self::hierarchy_request_error());
            }
            replacements.push(child);
        }

        let Some(node_entry) = self.dom.nodes.get(node.0) else {
            return Err(Self::hierarchy_request_error());
        };
        let existing_children = node_entry.children.clone();
        for child in existing_children {
            self.dom.remove_child(node, child)?;
        }
        for child in replacements {
            self.dom
                .append_child(node, child)
                .map_err(|_| Self::hierarchy_request_error())?;
        }

        Ok(Value::Undefined)
    }

    pub(crate) fn eval_node_replace_with_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        let Some(parent) = self.dom.parent(node) else {
            return Ok(Value::Undefined);
        };

        let mut replacements = Vec::with_capacity(evaluated_args.len());
        for value in evaluated_args {
            let child = match value {
                Value::Node(child) => *child,
                other => self.dom.create_detached_text(other.as_string()),
            };
            let Some(child_node) = self.dom.nodes.get(child.0) else {
                return Err(Self::hierarchy_request_error());
            };
            if matches!(child_node.node_type, NodeType::Document)
                || child == parent
                || self.dom.is_descendant_of(parent, child)
            {
                return Err(Self::hierarchy_request_error());
            }
            replacements.push(child);
        }

        let next_sibling = self.dom.nodes.get(parent.0).and_then(|entry| {
            let idx = entry.children.iter().position(|child| *child == node)?;
            entry.children.get(idx + 1).copied()
        });

        self.dom
            .remove_child(parent, node)
            .map_err(|_| Self::hierarchy_request_error())?;

        for child in replacements {
            if let Some(reference) = next_sibling {
                if self.dom.parent(reference) == Some(parent) {
                    self.dom
                        .insert_before(parent, child, reference)
                        .map_err(|_| Self::hierarchy_request_error())?;
                    continue;
                }
            }
            self.dom
                .append_child(parent, child)
                .map_err(|_| Self::hierarchy_request_error())?;
        }

        Ok(Value::Undefined)
    }

    pub(crate) fn eval_insert_adjacent_element_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if evaluated_args.len() != 2 {
            return Err(Error::ScriptRuntime(
                "insertAdjacentElement requires exactly two arguments".into(),
            ));
        }
        if self.dom.element(node).is_none() || self.is_document_fragment_node(node) {
            return Err(Error::ScriptRuntime(
                "TypeError: insertAdjacentElement target must be an Element".into(),
            ));
        }

        let position_text = evaluated_args[0].as_string();
        let position = resolve_insert_adjacent_position(&position_text).map_err(|_| {
            Error::ScriptRuntime(format!(
                "SyntaxError: Failed to execute 'insertAdjacentElement': invalid position '{position_text}'"
            ))
        })?;

        let element = match evaluated_args.get(1) {
            Some(Value::Node(element))
                if self.dom.element(*element).is_some()
                    && !self.is_document_fragment_node(*element) =>
            {
                *element
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Failed to execute 'insertAdjacentElement': parameter 2 is not of type 'Element'"
                        .into(),
                ));
            }
        };

        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(node) else {
                return Ok(Value::Null);
            };
            if self.dom.element(parent).is_none() || self.is_document_fragment_node(parent) {
                return Ok(Value::Null);
            }
        }

        if self
            .dom
            .insert_adjacent_node(node, position, element)
            .is_err()
        {
            return Ok(Value::Null);
        }
        Ok(Value::Node(element))
    }

    pub(crate) fn eval_insert_adjacent_html_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if evaluated_args.len() != 2 {
            return Err(Error::ScriptRuntime(
                "insertAdjacentHTML requires exactly two arguments".into(),
            ));
        }
        if self.dom.element(node).is_none() || self.is_document_fragment_node(node) {
            return Err(Error::ScriptRuntime(
                "TypeError: insertAdjacentHTML target must be an Element".into(),
            ));
        }

        let position_text = evaluated_args[0].as_string();
        let position = resolve_insert_adjacent_position(&position_text).map_err(|_| {
            Error::ScriptRuntime(format!(
                "SyntaxError: Failed to execute 'insertAdjacentHTML': invalid position '{position_text}'"
            ))
        })?;

        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(node) else {
                return Err(Error::ScriptRuntime(
                    "NoModificationAllowedError: Failed to execute 'insertAdjacentHTML' because the target has no parent element"
                        .into(),
                ));
            };
            if self.dom.element(parent).is_none() || self.is_document_fragment_node(parent) {
                return Err(Error::ScriptRuntime(
                    "NoModificationAllowedError: Failed to execute 'insertAdjacentHTML' on a node whose parent is not an Element"
                        .into(),
                ));
            }
        }

        let input = evaluated_args[1].as_string();
        match self.dom.insert_adjacent_html(node, position, &input) {
            Ok(()) => Ok(Value::Undefined),
            Err(Error::ScriptParse(message)) => {
                Err(Error::ScriptRuntime(format!("SyntaxError: {message}")))
            }
            Err(other) => Err(other),
        }
    }

    pub(crate) fn eval_insert_adjacent_text_call(
        &mut self,
        node: NodeId,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        if evaluated_args.len() != 2 {
            return Err(Error::ScriptRuntime(
                "insertAdjacentText requires exactly two arguments".into(),
            ));
        }
        if self.dom.element(node).is_none() || self.is_document_fragment_node(node) {
            return Err(Error::ScriptRuntime(
                "TypeError: insertAdjacentText target must be an Element".into(),
            ));
        }

        let position_text = evaluated_args[0].as_string();
        let position = resolve_insert_adjacent_position(&position_text).map_err(|_| {
            Error::ScriptRuntime(format!(
                "SyntaxError: Failed to execute 'insertAdjacentText': invalid position '{position_text}'"
            ))
        })?;

        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(node) else {
                return Ok(Value::Undefined);
            };
            if self.dom.element(parent).is_none() || self.is_document_fragment_node(parent) {
                return Ok(Value::Undefined);
            }
        }

        let text = self.dom.create_detached_text(evaluated_args[1].as_string());
        let _ = self.dom.insert_adjacent_node(node, position, text);
        Ok(Value::Undefined)
    }

    pub(crate) fn eval_closest_selector_value(
        &self,
        node: NodeId,
        selector: &str,
    ) -> Result<Value> {
        match self.dom.closest(node, selector) {
            Ok(Some(matched)) => Ok(Value::Node(matched)),
            Ok(None) => Ok(Value::Null),
            Err(Error::UnsupportedSelector(_)) => Err(Error::ScriptRuntime(
                "SyntaxError: The provided selector is invalid".into(),
            )),
            Err(other) => Err(other),
        }
    }

    pub(crate) fn eval_matches_selector_value(
        &self,
        node: NodeId,
        selector: &str,
    ) -> Result<Value> {
        match self.dom.matches_selector(node, selector) {
            Ok(matched) => Ok(Value::Bool(matched)),
            Err(Error::UnsupportedSelector(_)) => Err(Error::ScriptRuntime(
                "SyntaxError: The provided selector is invalid".into(),
            )),
            Err(other) => Err(other),
        }
    }

    pub(crate) fn eval_query_selector_value(&self, node: NodeId, selector: &str) -> Result<Value> {
        match self.dom.query_selector_from(&node, selector) {
            Ok(Some(matched)) => Ok(Value::Node(matched)),
            Ok(None) => Ok(Value::Null),
            Err(Error::UnsupportedSelector(_)) => Err(Error::ScriptRuntime(
                "SyntaxError: The provided selector is invalid".into(),
            )),
            Err(other) => Err(other),
        }
    }

    pub(crate) fn eval_query_selector_all_value(
        &self,
        node: NodeId,
        selector: &str,
    ) -> Result<Value> {
        match self.dom.query_selector_all_from(&node, selector) {
            Ok(nodes) => Ok(Self::new_static_node_list_value(nodes)),
            Err(Error::UnsupportedSelector(_)) => Err(Error::ScriptRuntime(
                "SyntaxError: The provided selector is invalid".into(),
            )),
            Err(other) => Err(other),
        }
    }

    pub(crate) fn parse_listener_capture_arg(&self, value: Option<&Value>) -> Result<bool> {
        let Some(value) = value else {
            return Ok(false);
        };
        match value {
            Value::Bool(capture) => Ok(*capture),
            Value::Object(entries) => {
                let entries = entries.borrow();
                Ok(Self::object_get_entry(&entries, "capture")
                    .map(|capture| capture.truthy())
                    .unwrap_or(false))
            }
            _ => Err(Error::ScriptRuntime(
                "add/removeEventListener third argument must be true/false or options object"
                    .into(),
            )),
        }
    }
}
