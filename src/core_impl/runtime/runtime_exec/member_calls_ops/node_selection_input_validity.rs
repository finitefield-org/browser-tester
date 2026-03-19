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
    fn local_name_from_qualified_name(name: &str) -> &str {
        name.rsplit_once(':')
            .map(|(_, local_name)| local_name)
            .unwrap_or(name)
    }

    fn attribute_namespace_uri_for_qualified_name(
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

    fn get_attribute_node_ns_value(
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

    fn get_attribute_ns_value(
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

    fn has_attribute_ns_value(
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

    fn remove_attribute_ns(
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

    fn get_bounding_client_rect_value(&self, node: NodeId) -> Result<Value> {
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

    fn get_client_rects_value(&self, node: NodeId) -> Result<Value> {
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

    fn eval_set_html_unsafe_call(
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

    fn attach_shadow_root(&mut self, node: NodeId, options: &Value) -> Result<NodeId> {
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

    fn element_get_html_value(&self, node: NodeId, options: Option<&Value>) -> Result<Value> {
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

    fn eval_node_after_call(&mut self, node: NodeId, evaluated_args: &[Value]) -> Result<Value> {
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

    fn eval_node_before_call(&mut self, node: NodeId, evaluated_args: &[Value]) -> Result<Value> {
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

    fn eval_node_prepend_call(&mut self, node: NodeId, evaluated_args: &[Value]) -> Result<Value> {
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

    fn eval_node_replace_children_call(
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

    fn eval_node_replace_with_call(
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

    fn eval_insert_adjacent_element_call(
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

    fn eval_insert_adjacent_html_call(
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

    fn eval_insert_adjacent_text_call(
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

    pub(crate) fn eval_document_member_call(
        &mut self,
        member: &str,
        evaluated_args: &[Value],
        _event: &EventState,
    ) -> Result<Option<Value>> {
        let shadowed = {
            let entries = self.dom_runtime.document_object.borrow();
            Self::placeholder_backed_object_builtin_is_shadowed(&entries, member)
        };
        if shadowed {
            return Ok(None);
        }

        match member {
            "getElementById" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementById requires exactly one argument".into(),
                    ));
                }
                let id = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom.by_id(&id).map(Value::Node).unwrap_or(Value::Null),
                ))
            }
            "getElementsByClassName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByClassName requires exactly one argument".into(),
                    ));
                }
                let class_names = Self::class_names_from_argument(&evaluated_args[0]);
                Ok(Some(
                    self.class_names_live_list_value(self.dom.root, class_names),
                ))
            }
            "getElementsByName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(self.name_live_list_value(
                    self.dom.root,
                    evaluated_args[0].as_string(),
                )))
            }
            "getElementsByTagName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(self.tag_name_live_list_value(
                    self.dom.root,
                    Self::tag_name_from_argument(&evaluated_args[0]),
                )))
            }
            "getElementsByTagNameNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagNameNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string();
                Ok(Some(self.tag_name_ns_live_list_value(
                    self.dom.root,
                    namespace_uri,
                    local_name,
                )))
            }
            "createElement" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "createElement requires one or two arguments".into(),
                    ));
                }
                let tag_name = evaluated_args[0].as_string().to_ascii_lowercase();
                let node = self.dom.create_detached_element(tag_name);
                if let Some(is_value) =
                    Self::create_element_is_option_from_arg(evaluated_args.get(1))
                {
                    self.dom.set_attr(node, "is", &is_value)?;
                }
                Ok(Some(Value::Node(node)))
            }
            "createElementNS" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "createElementNS requires two or three arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let raw_tag_name = evaluated_args[1].as_string();
                let tag_name = if namespace_uri.as_deref() == Some("http://www.w3.org/1999/xhtml") {
                    raw_tag_name.to_ascii_lowercase()
                } else {
                    raw_tag_name
                };
                let node = self
                    .dom
                    .create_detached_element_with_namespace(tag_name, namespace_uri);
                if let Some(is_value) =
                    Self::create_element_is_option_from_arg(evaluated_args.get(2))
                {
                    self.dom.set_attr(node, "is", &is_value)?;
                }
                Ok(Some(Value::Node(node)))
            }
            "createTextNode" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "createTextNode requires exactly one argument".into(),
                    ));
                }
                let text = evaluated_args[0].as_string();
                let node = self.dom.create_detached_text(text);
                Ok(Some(Value::Node(node)))
            }
            "createAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "createAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if !is_valid_create_attribute_name(&name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                Ok(Some(Self::new_attr_object_value(&name, "", None)))
            }
            "createDocumentFragment" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "createDocumentFragment takes no arguments".into(),
                    ));
                }
                let node = self
                    .dom
                    .create_detached_element("#document-fragment".to_string());
                Ok(Some(Value::Node(node)))
            }
            "createRange" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "createRange takes no arguments".into(),
                    ));
                }
                Ok(Some(Self::new_range_object_value(self.dom.root)))
            }
            "getSelection" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getSelection takes no arguments".into(),
                    ));
                }
                Ok(Some(self.ensure_document_selection_object()))
            }
            "append" => Ok(Some(
                self.eval_document_append_call(self.dom.root, evaluated_args)?,
            )),
            "querySelector" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelector requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.eval_query_selector_value(self.dom.root, &selector)?,
                ))
            }
            "querySelectorAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelectorAll requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.eval_query_selector_all_value(self.dom.root, &selector)?,
                ))
            }
            "createTreeWalker" => self.eval_create_tree_walker_call(evaluated_args),
            "addEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "addEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
                    Value::Function(function) => {
                        self.listeners.add(
                            self.dom.root,
                            event_type,
                            Listener {
                                capture,
                                is_event_handler_property: false,
                                is_arrow: function.is_arrow,
                                handler: function.handler.clone(),
                                function: Some(function.clone()),
                                captured_names: function.captured_names.clone(),
                                captured_env: function.captured_env.clone(),
                                captured_pending_function_decls: function
                                    .captured_pending_function_decls
                                    .clone(),
                            },
                        );
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "addEventListener callback must be a function".into(),
                    )),
                }
            }
            "removeEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "removeEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
                    Value::Function(function) => {
                        let _ = self.listeners.remove(
                            self.dom.root,
                            &event_type,
                            capture,
                            &function.handler,
                        );
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "removeEventListener callback must be a function".into(),
                    )),
                }
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_window_member_call(
        &mut self,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let shadowed = {
            let entries = self.dom_runtime.window_object.borrow();
            Self::object_get_entry(&entries, member)
                .is_some_and(|value| !Self::is_builtin_placeholder_value(&value))
                || Self::is_builtin_object_property_deleted(&entries, member)
        };
        if shadowed {
            return Ok(None);
        }

        match member {
            "getSelection" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getSelection takes no arguments".into(),
                    ));
                }
                Ok(Some(self.ensure_document_selection_object()))
            }
            _ => Ok(None),
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

    pub(crate) fn eval_event_target_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let (is_event_target, is_match_media, shadowed) = {
            let entries = object.borrow();
            (
                Self::is_event_target_object(&entries),
                Self::is_match_media_object(&entries),
                Self::placeholder_backed_object_builtin_is_shadowed(&entries, member),
            )
        };
        if !is_event_target {
            return Ok(None);
        }
        if shadowed {
            return Ok(None);
        }

        let (normalized_member, listener_event_type, capture_mode) = match member {
            "addListener" if is_match_media => ("addEventListener", "change", None),
            "removeListener" if is_match_media => ("removeEventListener", "change", None),
            "addEventListener" | "removeEventListener" => (member, "", Some(())),
            _ => return Ok(None),
        };

        let (event_type, capture, callback_value) = if capture_mode.is_none() {
            if evaluated_args.len() != 1 {
                let label = if normalized_member == "addEventListener" {
                    "addListener"
                } else {
                    "removeListener"
                };
                return Err(Error::ScriptRuntime(format!(
                    "{label} requires exactly one callback argument"
                )));
            }
            (
                listener_event_type.to_string(),
                false,
                evaluated_args.first().cloned().unwrap_or(Value::Undefined),
            )
        } else {
            if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                return Err(Error::ScriptRuntime(format!(
                    "{normalized_member} requires two or three arguments"
                )));
            }
            (
                evaluated_args[0].as_string(),
                self.parse_listener_capture_arg(evaluated_args.get(2))?,
                evaluated_args[1].clone(),
            )
        };

        let node = self.event_target_listener_node_id(object);
        let result = match normalized_member {
            "addEventListener" => match callback_value {
                Value::Function(function) => {
                    self.listeners.add(
                        node,
                        event_type,
                        Listener {
                            capture,
                            is_event_handler_property: false,
                            is_arrow: function.is_arrow,
                            handler: function.handler.clone(),
                            function: Some(function.clone()),
                            captured_names: function.captured_names.clone(),
                            captured_env: function.captured_env.clone(),
                            captured_pending_function_decls: function
                                .captured_pending_function_decls
                                .clone(),
                        },
                    );
                    Value::Undefined
                }
                Value::Null | Value::Undefined => Value::Undefined,
                _ => {
                    return Err(Error::ScriptRuntime(
                        "addEventListener callback must be a function".into(),
                    ));
                }
            },
            "removeEventListener" => match callback_value {
                Value::Function(function) => {
                    let _ = self
                        .listeners
                        .remove(node, &event_type, capture, &function.handler);
                    Value::Undefined
                }
                Value::Null | Value::Undefined => Value::Undefined,
                _ => {
                    return Err(Error::ScriptRuntime(
                        "removeEventListener callback must be a function".into(),
                    ));
                }
            },
            _ => return Ok(None),
        };

        Ok(Some(result))
    }

    pub(crate) fn eval_node_member_call(
        &mut self,
        node: NodeId,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "addEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "addEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
                    Value::Function(function) => {
                        self.listeners.add(
                            node,
                            event_type,
                            Listener {
                                capture,
                                is_event_handler_property: false,
                                is_arrow: function.is_arrow,
                                handler: function.handler.clone(),
                                function: Some(function.clone()),
                                captured_names: function.captured_names.clone(),
                                captured_env: function.captured_env.clone(),
                                captured_pending_function_decls: function
                                    .captured_pending_function_decls
                                    .clone(),
                            },
                        );
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "addEventListener callback must be a function".into(),
                    )),
                }
            }
            "removeEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "removeEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
                    Value::Function(function) => {
                        let _ =
                            self.listeners
                                .remove(node, &event_type, capture, &function.handler);
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "removeEventListener callback must be a function".into(),
                    )),
                }
            }
            "click" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("click does not take arguments".into()));
                }
                self.click_dom_method(node)?;
                Ok(Some(Value::Undefined))
            }
            "attachShadow" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "attachShadow requires exactly one options argument".into(),
                    ));
                }
                let root = self.attach_shadow_root(node, &evaluated_args[0])?;
                Ok(Some(Value::Node(root)))
            }
            "getAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if name == "nonce" {
                    return Ok(Some(if self.dom.attr(node, "nonce").is_some() {
                        Value::String(String::new())
                    } else {
                        Value::Null
                    }));
                }
                Ok(Some(
                    self.dom
                        .attr(node, &name)
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                ))
            }
            "getAttributeNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string().to_ascii_lowercase();
                Ok(Some(self.get_attribute_ns_value(
                    node,
                    namespace_uri.as_deref(),
                    &local_name,
                )))
            }
            "getBoundingClientRect" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getBoundingClientRect takes no arguments".into(),
                    ));
                }
                Ok(Some(self.get_bounding_client_rect_value(node)?))
            }
            "getClientRects" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getClientRects takes no arguments".into(),
                    ));
                }
                Ok(Some(self.get_client_rects_value(node)?))
            }
            "getHTML" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "getHTML supports zero or one options argument".into(),
                    ));
                }
                Ok(Some(
                    self.element_get_html_value(node, evaluated_args.first())?,
                ))
            }
            "getAttributeNode" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNode requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                Ok(Some(
                    self.dom
                        .attr(node, &name)
                        .map(|value| Self::new_attr_object_value(&name, &value, Some(node)))
                        .unwrap_or(Value::Null),
                ))
            }
            "getAttributeNodeNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNodeNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string().to_ascii_lowercase();
                Ok(Some(self.get_attribute_node_ns_value(
                    node,
                    namespace_uri.as_deref(),
                    &local_name,
                )))
            }
            "setAttribute" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "setAttribute requires exactly two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if !is_valid_create_attribute_name(&name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                let value = evaluated_args[1].as_string();
                self.dom.set_attr(node, &name, &value)?;
                Ok(Some(Value::Undefined))
            }
            "setAttributeNS" => {
                if evaluated_args.len() != 3 {
                    return Err(Error::ScriptRuntime(
                        "setAttributeNS requires exactly three arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let qualified_name = evaluated_args[1].as_string().to_ascii_lowercase();
                if !is_valid_qualified_attribute_name(&qualified_name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                if namespace_uri.is_none() && qualified_name.contains(':') {
                    return Err(Error::ScriptRuntime(
                        "NamespaceError: prefix requires a namespace".into(),
                    ));
                }
                let value = evaluated_args[2].as_string();
                let local_name =
                    Self::local_name_from_qualified_name(&qualified_name).to_ascii_lowercase();
                let replaced = {
                    let Some(element) = self.dom.element(node) else {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNS target is not an element".into(),
                        ));
                    };
                    let mut matches = element
                        .attrs
                        .iter()
                        .filter_map(|(existing_name, _)| {
                            let existing_local_name =
                                Self::local_name_from_qualified_name(existing_name);
                            if !existing_local_name.eq_ignore_ascii_case(&local_name) {
                                return None;
                            }
                            let existing_namespace = self
                                .attribute_namespace_uri_for_qualified_name(node, existing_name);
                            let namespace_matches =
                                match (namespace_uri.as_deref(), existing_namespace.as_deref()) {
                                    (None, None) => true,
                                    (Some(expected), Some(actual)) => expected == actual,
                                    _ => false,
                                };
                            if !namespace_matches {
                                return None;
                            }
                            Some(existing_name.clone())
                        })
                        .collect::<Vec<_>>();
                    matches.sort();
                    matches.into_iter().next()
                };
                if let Some(replaced_name) = replaced {
                    self.dom.remove_attr(node, &replaced_name)?;
                }
                self.dom.set_attr(node, &qualified_name, &value)?;
                Ok(Some(Value::Undefined))
            }
            "setAttributeNode" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setAttributeNode requires exactly one argument".into(),
                    ));
                }
                let attr_object = match evaluated_args.first() {
                    Some(Value::Object(object)) => object.clone(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNode argument must be an Attr".into(),
                        ));
                    }
                };
                let (name, value): (String, String) = {
                    let entries = attr_object.borrow();
                    if !Self::is_attr_object(&entries) {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNode argument must be an Attr".into(),
                        ));
                    }
                    let name = Self::object_get_entry(&entries, "name")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    if !is_valid_create_attribute_name(&name) {
                        return Err(Error::ScriptRuntime(
                            "InvalidCharacterError: attribute name is not a valid XML name".into(),
                        ));
                    }
                    let value = Self::object_get_entry(&entries, "value")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default();
                    (name, value)
                };
                let replaced_value = self.dom.attr(node, &name);
                self.dom.set_attr(node, &name, &value)?;

                {
                    let mut entries = attr_object.borrow_mut();
                    Self::object_set_entry(
                        &mut entries,
                        "name".to_string(),
                        Value::String(name.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "value".to_string(),
                        Value::String(value.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "ownerElement".to_string(),
                        Value::Node(node),
                    );
                }

                Ok(Some(
                    replaced_value
                        .map(|old| Self::new_attr_object_value(&name, &old, None))
                        .unwrap_or(Value::Null),
                ))
            }
            "setAttributeNodeNS" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setAttributeNodeNS requires exactly one argument".into(),
                    ));
                }
                let attr_object = match evaluated_args.first() {
                    Some(Value::Object(object)) => object.clone(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNodeNS argument must be an Attr".into(),
                        ));
                    }
                };
                let (name, value, owner_element): (String, String, Option<NodeId>) = {
                    let entries = attr_object.borrow();
                    if !Self::is_attr_object(&entries) {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNodeNS argument must be an Attr".into(),
                        ));
                    }
                    let name = Self::object_get_entry(&entries, "name")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    let value = Self::object_get_entry(&entries, "value")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default();
                    let owner_element = match Self::object_get_entry(&entries, "ownerElement") {
                        Some(Value::Node(owner)) => Some(owner),
                        _ => None,
                    };
                    (name, value, owner_element)
                };

                let namespace_uri = owner_element
                    .and_then(|owner| self.attribute_namespace_uri_for_qualified_name(owner, &name))
                    .or_else(|| self.attribute_namespace_uri_for_qualified_name(node, &name));
                let local_name = Self::local_name_from_qualified_name(&name).to_ascii_lowercase();

                let replaced = {
                    let Some(element) = self.dom.element(node) else {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNodeNS target is not an element".into(),
                        ));
                    };
                    let mut matches = element
                        .attrs
                        .iter()
                        .filter_map(|(qualified_name, existing_value)| {
                            let candidate_local_name =
                                Self::local_name_from_qualified_name(qualified_name);
                            if !candidate_local_name.eq_ignore_ascii_case(&local_name) {
                                return None;
                            }
                            let candidate_namespace = self
                                .attribute_namespace_uri_for_qualified_name(node, qualified_name);
                            let namespace_matches =
                                match (namespace_uri.as_deref(), candidate_namespace.as_deref()) {
                                    (None, None) => true,
                                    (Some(expected), Some(actual)) => expected == actual,
                                    _ => false,
                                };
                            if !namespace_matches {
                                return None;
                            }
                            Some((qualified_name.clone(), existing_value.clone()))
                        })
                        .collect::<Vec<_>>();
                    matches.sort_by(|(left, _), (right, _)| left.cmp(right));
                    matches.into_iter().next()
                };

                if let Some((replaced_name, _)) = replaced.as_ref() {
                    self.dom.remove_attr(node, replaced_name)?;
                }
                self.dom.set_attr(node, &name, &value)?;

                {
                    let mut entries = attr_object.borrow_mut();
                    Self::object_set_entry(
                        &mut entries,
                        "name".to_string(),
                        Value::String(name.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "value".to_string(),
                        Value::String(value.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "ownerElement".to_string(),
                        Value::Node(node),
                    );
                }

                Ok(Some(
                    replaced
                        .map(|(old_name, old_value)| {
                            Self::new_attr_object_value(&old_name, &old_value, None)
                        })
                        .unwrap_or(Value::Null),
                ))
            }
            "removeAttributeNode" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeAttributeNode requires exactly one argument".into(),
                    ));
                }
                let attr_object = match evaluated_args.first() {
                    Some(Value::Object(object)) => object.clone(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "removeAttributeNode argument must be an Attr".into(),
                        ));
                    }
                };
                let (name, owner_matches_node): (String, bool) = {
                    let entries = attr_object.borrow();
                    if !Self::is_attr_object(&entries) {
                        return Err(Error::ScriptRuntime(
                            "removeAttributeNode argument must be an Attr".into(),
                        ));
                    }
                    let name = Self::object_get_entry(&entries, "name")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    let owner_matches_node = matches!(Self::object_get_entry(&entries, "ownerElement"), Some(Value::Node(owner)) if owner == node);
                    (name, owner_matches_node)
                };

                let Some(current_value) = self.dom.attr(node, &name) else {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeAttributeNode': The attribute node was not found"
                            .into(),
                    ));
                };
                if !owner_matches_node {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeAttributeNode': The attribute node was not found"
                            .into(),
                    ));
                }
                self.dom.remove_attr(node, &name)?;
                {
                    let mut entries = attr_object.borrow_mut();
                    Self::object_set_entry(
                        &mut entries,
                        "name".to_string(),
                        Value::String(name.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "value".to_string(),
                        Value::String(current_value),
                    );
                    Self::object_set_entry(&mut entries, "ownerElement".to_string(), Value::Null);
                }
                Ok(Some(Value::Object(attr_object)))
            }
            "hasAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "hasAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                Ok(Some(Value::Bool(self.dom.has_attr(node, &name)?)))
            }
            "hasAttributeNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "hasAttributeNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string().to_ascii_lowercase();
                Ok(Some(Value::Bool(self.has_attribute_ns_value(
                    node,
                    namespace_uri.as_deref(),
                    &local_name,
                ))))
            }
            "hasAttributes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "hasAttributes takes no arguments".into(),
                    ));
                }
                let has_attributes = self
                    .dom
                    .element(node)
                    .map(|element| !element.attrs.is_empty())
                    .ok_or_else(|| {
                        Error::ScriptRuntime("hasAttributes target is not an element".into())
                    })?;
                Ok(Some(Value::Bool(has_attributes)))
            }
            "removeAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                self.dom.remove_attr(node, &name)?;
                Ok(Some(Value::Undefined))
            }
            "removeAttributeNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "removeAttributeNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string().to_ascii_lowercase();
                self.remove_attribute_ns(node, namespace_uri.as_deref(), &local_name)?;
                Ok(Some(Value::Undefined))
            }
            "getAttributeNames" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNames takes no arguments".into(),
                    ));
                }
                let element = self.dom.element(node).ok_or_else(|| {
                    Error::ScriptRuntime("getAttributeNames target is not an element".into())
                })?;
                let mut names = element.attrs.keys().cloned().collect::<Vec<_>>();
                names.sort();
                Ok(Some(Self::new_array_value(
                    names.into_iter().map(Value::String).collect(),
                )))
            }
            "toggleAttribute" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "toggleAttribute requires one or two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if !is_valid_create_attribute_name(&name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                let has = self.dom.has_attr(node, &name)?;
                let next = if evaluated_args.len() == 2 {
                    evaluated_args[1].truthy()
                } else {
                    !has
                };
                if next {
                    self.dom.set_attr(node, &name, "")?;
                } else {
                    self.dom.remove_attr(node, &name)?;
                }
                Ok(Some(Value::Bool(next)))
            }
            "matches" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "matches requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_matches_selector_value(node, &selector)?))
            }
            "closest" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "closest requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_closest_selector_value(node, &selector)?))
            }
            "querySelector" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelector requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_query_selector_value(node, &selector)?))
            }
            "querySelectorAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelectorAll requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_query_selector_all_value(node, &selector)?))
            }
            "replaceWith" => Ok(Some(
                self.eval_node_replace_with_call(node, evaluated_args)?,
            )),
            "replaceChildren" => Ok(Some(
                self.eval_node_replace_children_call(node, evaluated_args)?,
            )),
            "append" => Ok(Some(self.eval_document_append_call(node, evaluated_args)?)),
            "prepend" => Ok(Some(self.eval_node_prepend_call(node, evaluated_args)?)),
            "after" => Ok(Some(self.eval_node_after_call(node, evaluated_args)?)),
            "before" => Ok(Some(self.eval_node_before_call(node, evaluated_args)?)),
            "insertAdjacentElement" => Ok(Some(
                self.eval_insert_adjacent_element_call(node, evaluated_args)?,
            )),
            "insertAdjacentHTML" => Ok(Some(
                self.eval_insert_adjacent_html_call(node, evaluated_args)?,
            )),
            "setHTMLUnsafe" => Ok(Some(self.eval_set_html_unsafe_call(node, evaluated_args)?)),
            "insertAdjacentText" => Ok(Some(
                self.eval_insert_adjacent_text_call(node, evaluated_args)?,
            )),
            "appendChild" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "appendChild requires exactly one node argument".into(),
                    ));
                }
                let child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "appendChild argument must be a Node".into(),
                        ));
                    }
                };
                self.dom.append_child(node, child)?;
                Ok(Some(Value::Node(child)))
            }
            "insertBefore" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "insertBefore requires exactly two arguments".into(),
                    ));
                }
                let child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "insertBefore first argument must be a Node".into(),
                        ));
                    }
                };
                match evaluated_args.get(1) {
                    Some(Value::Node(reference)) => {
                        self.dom.insert_before(node, child, *reference)?;
                    }
                    Some(Value::Null) | Some(Value::Undefined) => {
                        self.dom.append_child(node, child)?;
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "insertBefore second argument must be a Node or null".into(),
                        ));
                    }
                }
                Ok(Some(Value::Node(child)))
            }
            "removeChild" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeChild requires exactly one node argument".into(),
                    ));
                }
                let child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "removeChild argument must be a Node".into(),
                        ));
                    }
                };
                self.dom.remove_child(node, child)?;
                Ok(Some(Value::Node(child)))
            }
            "replaceChild" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "replaceChild requires exactly two node arguments".into(),
                    ));
                }
                let new_child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "replaceChild first argument must be a Node".into(),
                        ));
                    }
                };
                let old_child = match evaluated_args.get(1) {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "replaceChild second argument must be a Node".into(),
                        ));
                    }
                };
                self.dom.replace_child(node, new_child, old_child)?;
                Ok(Some(Value::Node(old_child)))
            }
            "hasChildNodes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "hasChildNodes takes no arguments".into(),
                    ));
                }
                Ok(Some(Value::Bool(
                    !self.dom.nodes[node.0].children.is_empty(),
                )))
            }
            "contains" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "contains requires exactly one argument".into(),
                    ));
                }
                let contains = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => false,
                    Some(Value::Node(other)) => {
                        *other == node || self.dom.is_descendant_of(*other, node)
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "contains argument must be a Node or null".into(),
                        ));
                    }
                };
                Ok(Some(Value::Bool(contains)))
            }
            "getRootNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "getRootNode supports at most one options argument".into(),
                    ));
                }
                Ok(Some(Value::Node(self.node_root(node))))
            }
            "compareDocumentPosition" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "compareDocumentPosition requires exactly one node argument".into(),
                    ));
                }
                let other = match evaluated_args.first() {
                    Some(Value::Node(other)) => *other,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "compareDocumentPosition argument must be a Node".into(),
                        ));
                    }
                };
                Ok(Some(Value::Number(
                    self.node_compare_document_position(node, other),
                )))
            }
            "isEqualNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "isEqualNode supports at most one argument".into(),
                    ));
                }
                let is_equal = match evaluated_args.first() {
                    None | Some(Value::Null) | Some(Value::Undefined) => false,
                    Some(Value::Node(other)) => self.nodes_are_equal(node, *other),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "isEqualNode argument must be a Node or null".into(),
                        ));
                    }
                };
                Ok(Some(Value::Bool(is_equal)))
            }
            "isSameNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "isSameNode supports at most one argument".into(),
                    ));
                }
                let is_same = match evaluated_args.first() {
                    None | Some(Value::Null) | Some(Value::Undefined) => false,
                    Some(Value::Node(other)) => node == *other,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "isSameNode argument must be a Node or null".into(),
                        ));
                    }
                };
                Ok(Some(Value::Bool(is_same)))
            }
            "normalize" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("normalize takes no arguments".into()));
                }
                self.normalize_node_subtree(node)?;
                Ok(Some(Value::Undefined))
            }
            "isDefaultNamespace" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "isDefaultNamespace requires exactly one namespace argument".into(),
                    ));
                }
                let namespace = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => None,
                    Some(value) => Some(value.as_string()),
                    None => None,
                };
                Ok(Some(Value::Bool(
                    self.node_is_default_namespace(node, namespace.as_deref()),
                )))
            }
            "lookupPrefix" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "lookupPrefix requires exactly one namespace argument".into(),
                    ));
                }
                let namespace = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => None,
                    Some(value) => Some(value.as_string()),
                    None => None,
                };
                Ok(Some(
                    self.node_lookup_prefix(node, namespace.as_deref())
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                ))
            }
            "lookupNamespaceURI" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "lookupNamespaceURI requires exactly one prefix argument".into(),
                    ));
                }
                let prefix = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => None,
                    Some(value) => Some(value.as_string()),
                    None => None,
                };
                Ok(Some(
                    self.node_lookup_namespace_uri(node, prefix.as_deref())
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                ))
            }
            "cloneNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "cloneNode supports at most one argument".into(),
                    ));
                }
                let deep = evaluated_args.first().is_some_and(Value::truthy);
                let cloned = self.clone_dom_node(node, deep)?;
                Ok(Some(Value::Node(cloned)))
            }
            "add" => {
                if !self.is_select_element(node) {
                    return Ok(None);
                }
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "add on HTMLSelectElement requires one or two arguments".into(),
                    ));
                }
                let option = match evaluated_args.first() {
                    Some(Value::Node(option)) => *option,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "TypeError: Failed to execute 'add' on 'HTMLSelectElement': parameter 1 is not of type 'HTMLElement'"
                                .into(),
                        ));
                    }
                };
                let option_tag = self.dom.tag_name(option).unwrap_or_default();
                if !option_tag.eq_ignore_ascii_case("option")
                    && !option_tag.eq_ignore_ascii_case("optgroup")
                {
                    return Err(Error::ScriptRuntime(
                        "TypeError: Failed to execute 'add' on 'HTMLSelectElement': parameter 1 is not of type 'HTMLElement'"
                            .into(),
                    ));
                }

                let before = match evaluated_args.get(1) {
                    None | Some(Value::Undefined) | Some(Value::Null) => None,
                    Some(Value::Node(candidate)) if self.dom.parent(*candidate) == Some(node) => {
                        Some(*candidate)
                    }
                    Some(value) => self
                        .value_as_index(value)
                        .and_then(|index| self.select_option_nodes(node).get(index).copied()),
                };

                if let Some(before) = before {
                    self.dom.insert_before(node, option, before)?;
                } else {
                    self.dom.append_child(node, option)?;
                }
                self.dom.sync_select_value(node)?;
                Ok(Some(Value::Undefined))
            }
            "item" => {
                if !self.is_select_element(node) {
                    return Ok(None);
                }
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "item on HTMLSelectElement requires exactly one index argument".into(),
                    ));
                }
                let index = Self::value_to_i64(&evaluated_args[0]);
                if index < 0 {
                    return Ok(Some(Value::Null));
                }
                Ok(Some(
                    self.select_option_nodes(node)
                        .get(index as usize)
                        .copied()
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "namedItem" => {
                if !self.is_select_element(node) {
                    return Ok(None);
                }
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "namedItem on HTMLSelectElement requires exactly one name argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                Ok(Some(
                    self.select_named_item(node, &name)
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "remove" => {
                if self.is_select_element(node) {
                    match evaluated_args.len() {
                        0 => {}
                        1 => {
                            let index = Self::value_to_i64(&evaluated_args[0]);
                            if index >= 0 {
                                if let Some(option) =
                                    self.select_option_nodes(node).get(index as usize).copied()
                                {
                                    self.dom.remove_node(option)?;
                                }
                                self.dom.sync_select_value(node)?;
                            }
                            return Ok(Some(Value::Undefined));
                        }
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "remove on HTMLSelectElement supports at most one index argument"
                                    .into(),
                            ));
                        }
                    }
                } else if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("remove takes no arguments".into()));
                }
                if evaluated_args.is_empty() {
                    if let Some(active) = self.dom.active_element() {
                        if active == node || self.dom.is_descendant_of(active, node) {
                            self.dom.set_active_element(None);
                        }
                    }
                    if let Some(active_pseudo) = self.dom.active_pseudo_element() {
                        if active_pseudo == node || self.dom.is_descendant_of(active_pseudo, node) {
                            self.dom.set_active_pseudo_element(None);
                        }
                    }
                    self.dom.remove_node(node)?;
                }
                Ok(Some(Value::Undefined))
            }
            "focus" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("focus takes no arguments".into()));
                }
                self.focus_node(node)?;
                Ok(Some(Value::Undefined))
            }
            "blur" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("blur takes no arguments".into()));
                }
                self.blur_node(node)?;
                Ok(Some(Value::Undefined))
            }
            "setPointerCapture" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setPointerCapture requires exactly one pointerId argument".into(),
                    ));
                }
                let pointer_id = Self::value_to_i64(&evaluated_args[0]);
                if pointer_id <= 0 {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'setPointerCapture': No active pointer with the given id"
                            .into(),
                    ));
                }
                self.dom_runtime
                    .pointer_capture_targets
                    .insert(pointer_id, node);
                Ok(Some(Value::Undefined))
            }
            "hasPointerCapture" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "hasPointerCapture requires exactly one pointerId argument".into(),
                    ));
                }
                let pointer_id = Self::value_to_i64(&evaluated_args[0]);
                let has_capture = self
                    .dom_runtime
                    .pointer_capture_targets
                    .get(&pointer_id)
                    .is_some_and(|captured_node| *captured_node == node);
                Ok(Some(Value::Bool(has_capture)))
            }
            "releasePointerCapture" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "releasePointerCapture requires exactly one pointerId argument".into(),
                    ));
                }
                let pointer_id = Self::value_to_i64(&evaluated_args[0]);
                let Some(captured_node) = self
                    .dom_runtime
                    .pointer_capture_targets
                    .get(&pointer_id)
                    .copied()
                else {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'releasePointerCapture': No active pointer with the given id"
                            .into(),
                    ));
                };
                if captured_node == node {
                    self.dom_runtime.pointer_capture_targets.remove(&pointer_id);
                }
                Ok(Some(Value::Undefined))
            }
            "captureStream" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "captureStream supports at most one argument".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                let frame_rate = evaluated_args
                    .first()
                    .map(|value| Self::number_value(Self::value_to_i64(value) as f64))
                    .unwrap_or(Value::Undefined);
                Ok(Some(Self::new_object_value(vec![
                    (
                        INTERNAL_CANVAS_KEY_PREFIX.to_string(),
                        Value::String("canvas_capture_stream".to_string()),
                    ),
                    ("active".to_string(), Value::Bool(true)),
                    ("canvas".to_string(), Value::Node(node)),
                    ("frameRate".to_string(), frame_rate),
                ])))
            }
            "getContext" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "getContext requires one or two arguments".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                let transferred_key =
                    INTERNAL_CANVAS_TRANSFERRED_TO_OFFSCREEN_NODE_EXPANDO_KEY.to_string();
                let transferred_to_offscreen = self
                    .dom_runtime
                    .node_expando_props
                    .get(&(node, transferred_key))
                    .is_some_and(|value| value.truthy());
                if transferred_to_offscreen {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Failed to execute 'getContext': canvas has transferred control to offscreen"
                            .into(),
                    ));
                }
                let context_kind = evaluated_args[0].as_string().to_ascii_lowercase();
                let is_known_context = matches!(
                    context_kind.as_str(),
                    "2d" | "webgl" | "experimental-webgl" | "webgl2" | "webgpu" | "bitmaprenderer"
                );
                if let Some(Value::String(existing_mode)) =
                    self.dom_runtime.node_expando_props.get(&(
                        node,
                        INTERNAL_CANVAS_CONTEXT_MODE_NODE_EXPANDO_KEY.to_string(),
                    ))
                {
                    if existing_mode != &context_kind {
                        return Ok(Some(Value::Null));
                    }
                }
                if context_kind != "2d" {
                    return Ok(Some(Value::Null));
                }
                let key = INTERNAL_CANVAS_2D_CONTEXT_NODE_EXPANDO_KEY.to_string();
                if let Some(existing) = self
                    .dom_runtime
                    .node_expando_props
                    .get(&(node, key.clone()))
                {
                    return Ok(Some(existing.clone()));
                }
                let alpha = evaluated_args
                    .get(1)
                    .map(Self::canvas_2d_alpha_from_options)
                    .unwrap_or(true);
                let context = self.new_canvas_2d_context_value(node, alpha);
                self.dom_runtime
                    .node_expando_props
                    .insert((node, key), context.clone());
                if is_known_context {
                    self.dom_runtime.node_expando_props.insert(
                        (
                            node,
                            INTERNAL_CANVAS_CONTEXT_MODE_NODE_EXPANDO_KEY.to_string(),
                        ),
                        Value::String(context_kind),
                    );
                }
                Ok(Some(context))
            }
            "transferControlToOffscreen" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "transferControlToOffscreen takes no arguments".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                if self.dom_runtime.node_expando_props.contains_key(&(
                    node,
                    INTERNAL_CANVAS_CONTEXT_MODE_NODE_EXPANDO_KEY.to_string(),
                )) {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Failed to execute 'transferControlToOffscreen': canvas has an existing rendering context"
                            .into(),
                    ));
                }
                if self.dom_runtime.node_expando_props.contains_key(&(
                    node,
                    INTERNAL_CANVAS_TRANSFERRED_TO_OFFSCREEN_NODE_EXPANDO_KEY.to_string(),
                )) {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Failed to execute 'transferControlToOffscreen': canvas has already transferred control to offscreen"
                            .into(),
                    ));
                }
                self.dom_runtime.node_expando_props.insert(
                    (
                        node,
                        INTERNAL_CANVAS_TRANSFERRED_TO_OFFSCREEN_NODE_EXPANDO_KEY.to_string(),
                    ),
                    Value::Bool(true),
                );
                Ok(Some(Self::new_object_value(vec![
                    (
                        INTERNAL_CANVAS_KEY_PREFIX.to_string(),
                        Value::String("offscreen_canvas".to_string()),
                    ),
                    (
                        "width".to_string(),
                        Value::Number(self.canvas_dimension_value(node, "width")),
                    ),
                    (
                        "height".to_string(),
                        Value::Number(self.canvas_dimension_value(node, "height")),
                    ),
                ])))
            }
            "toDataURL" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "toDataURL supports at most two arguments".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                let mime = evaluated_args
                    .first()
                    .map(Value::as_string)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "image/png".to_string());
                let mime = if mime.eq_ignore_ascii_case("image/png")
                    || mime.eq_ignore_ascii_case("image/jpeg")
                    || mime.eq_ignore_ascii_case("image/webp")
                {
                    mime.to_ascii_lowercase()
                } else {
                    "image/png".to_string()
                };
                let payload = match mime.as_str() {
                    "image/jpeg" => "/9j/4AAQSkZJRgABAQAAAQABAAD/2w==",
                    "image/webp" => "UklGRhIAAABXRUJQVlA4TA0AAAAvAAAAAA==",
                    _ => {
                        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR4nGNgYAAAAAMAASsJTYQAAAAASUVORK5CYII="
                    }
                };
                Ok(Some(Value::String(format!("data:{mime};base64,{payload}"))))
            }
            "toBlob" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "toBlob requires one to three arguments".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                let callback = evaluated_args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime(
                        "toBlob callback must be callable".into(),
                    ));
                }
                let mime = evaluated_args
                    .get(1)
                    .map(Value::as_string)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "image/png".to_string());
                let mime = if mime.eq_ignore_ascii_case("image/png")
                    || mime.eq_ignore_ascii_case("image/jpeg")
                    || mime.eq_ignore_ascii_case("image/webp")
                {
                    mime.to_ascii_lowercase()
                } else {
                    "image/png".to_string()
                };
                let bytes = match mime.as_str() {
                    "image/jpeg" => vec![0xFF, 0xD8, 0xFF, 0xD9],
                    "image/webp" => b"RIFFWEBP".to_vec(),
                    _ => vec![0x89, b'P', b'N', b'G'],
                };
                let blob = Self::new_blob_value(bytes, mime);
                self.execute_callback_value(&callback, &[blob], event)?;
                Ok(Some(Value::Undefined))
            }
            "getElementsByClassName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByClassName requires exactly one argument".into(),
                    ));
                }
                let class_names = Self::class_names_from_argument(&evaluated_args[0]);
                Ok(Some(self.class_names_live_list_value(node, class_names)))
            }
            "getElementsByTagName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(self.tag_name_live_list_value(
                    node,
                    Self::tag_name_from_argument(&evaluated_args[0]),
                )))
            }
            "getElementsByTagNameNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagNameNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string();
                Ok(Some(self.tag_name_ns_live_list_value(
                    node,
                    namespace_uri,
                    local_name,
                )))
            }
            "checkVisibility" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "checkVisibility supports at most one argument".into(),
                    ));
                }
                Ok(Some(Value::Bool(!self.dom.has_attr(node, "hidden")?)))
            }
            "checkValidity" | "reportValidity" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(format!("{member} takes no arguments")));
                }
                let is_form = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("form"));
                if is_form {
                    return Ok(Some(Value::Bool(self.validate_form_submission(node)?)));
                }
                let validity = self.compute_input_validity(node)?;
                if !validity.valid {
                    let _ = self.dispatch_invalid_event(node)?;
                }
                Ok(Some(Value::Bool(validity.valid)))
            }
            "setCustomValidity" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setCustomValidity requires exactly one argument".into(),
                    ));
                }
                self.dom
                    .set_custom_validity_message(node, &evaluated_args[0].as_string())?;
                Ok(Some(Value::Undefined))
            }
            "setSelectionRange" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "setSelectionRange requires two or three arguments".into(),
                    ));
                }
                self.set_node_selection_range(
                    node,
                    Self::value_to_i64(&evaluated_args[0]),
                    Self::value_to_i64(&evaluated_args[1]),
                    evaluated_args
                        .get(2)
                        .map(Value::as_string)
                        .unwrap_or_else(|| "none".to_string()),
                )?;
                Ok(Some(Value::Undefined))
            }
            "setRangeText" => {
                if !(evaluated_args.len() == 1
                    || evaluated_args.len() == 3
                    || evaluated_args.len() == 4)
                {
                    return Err(Error::ScriptRuntime(
                        "setRangeText supports one, three, or four arguments".into(),
                    ));
                }
                self.set_node_range_text(node, evaluated_args)?;
                Ok(Some(Value::Undefined))
            }
            "showPicker" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("showPicker takes no arguments".into()));
                }
                Ok(Some(Value::Undefined))
            }
            "stepUp" | "stepDown" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} supports at most one argument"
                    )));
                }
                let count = evaluated_args.first().map(Self::value_to_i64).unwrap_or(1);
                let direction = if member == "stepDown" { -1 } else { 1 };
                self.step_input_value(node, direction, count)?;
                Ok(Some(Value::Undefined))
            }
            "getAnimations" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "getAnimations supports zero or one options argument".into(),
                    ));
                }
                let subtree = Self::get_animations_subtree_option(evaluated_args.first());
                Ok(Some(self.node_get_animations_value(node, subtree)))
            }
            "animate" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "animate requires one or two arguments".into(),
                    ));
                }
                let options_arg = evaluated_args.get(1);
                let id = Self::animate_id_from_options(options_arg);
                let timeline =
                    Self::animate_option_entry(options_arg, "timeline").unwrap_or(Value::Null);
                let range_start = Self::animate_option_entry(options_arg, "rangeStart")
                    .unwrap_or(Value::String("normal".to_string()));
                let range_end = Self::animate_option_entry(options_arg, "rangeEnd")
                    .unwrap_or(Value::String("normal".to_string()));
                let keyframes = evaluated_args[0].clone();
                let options = options_arg.cloned().unwrap_or(Value::Undefined);
                let animation = Self::new_animation_object_value(
                    id,
                    keyframes,
                    options,
                    timeline,
                    range_start,
                    range_end,
                );
                self.register_node_animation(node, &animation);
                Ok(Some(animation))
            }
            "scrollIntoView" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "scrollIntoView supports zero or one argument".into(),
                    ));
                }
                self.dispatch_document_scroll_sequence(true)?;
                Ok(Some(Value::Undefined))
            }
            "scroll" | "scrollTo" | "scrollBy" => {
                if !(evaluated_args.is_empty()
                    || evaluated_args.len() == 1
                    || evaluated_args.len() == 2)
                {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} supports zero, one, or two arguments"
                    )));
                }
                let position_changed = self.apply_document_scroll_operation(member, evaluated_args);
                self.sync_window_runtime_properties();
                self.dispatch_document_scroll_sequence(position_changed)?;
                Ok(Some(Value::Undefined))
            }
            "select" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("select takes no arguments".into()));
                }
                if self.node_supports_text_selection(node) {
                    self.focus_node(node)?;
                    let len = self.dom.value(node)?.chars().count();
                    self.set_node_selection_range(node, 0, len as i64, "none".to_string())?;
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }
}
