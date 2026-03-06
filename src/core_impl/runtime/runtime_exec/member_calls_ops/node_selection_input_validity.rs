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
        let left = self
            .dom
            .offset_left(node)?
            .saturating_sub(self.dom_runtime.document_scroll_x);
        let top = self
            .dom
            .offset_top(node)?
            .saturating_sub(self.dom_runtime.document_scroll_y);
        let width = self.dom.offset_width(node)?;
        let height = self.dom.offset_height(node)?;
        let right = left.saturating_add(width);
        let bottom = top.saturating_add(height);

        Ok(Self::new_object_value(vec![
            ("x".to_string(), Value::Number(left)),
            ("y".to_string(), Value::Number(top)),
            ("left".to_string(), Value::Number(left)),
            ("top".to_string(), Value::Number(top)),
            ("right".to_string(), Value::Number(right)),
            ("bottom".to_string(), Value::Number(bottom)),
            ("width".to_string(), Value::Number(width)),
            ("height".to_string(), Value::Number(height)),
        ]))
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
            return Ok(Self::new_array_value(Vec::new()));
        }
        let rect = self.get_bounding_client_rect_value(node)?;
        Ok(Self::new_array_value(vec![rect]))
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
            .position(|option| self.dom.attr(*option, "selected").is_some())
            .map(|index| index as i64)
            .unwrap_or(-1)
    }

    pub(crate) fn select_selected_option_nodes(&self, select_node: NodeId) -> Vec<NodeId> {
        self.select_option_nodes(select_node)
            .iter()
            .copied()
            .filter(|option| self.dom.attr(*option, "selected").is_some())
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
            let option_element = self.dom.element_mut(*option).ok_or_else(|| {
                Error::ScriptRuntime("selectedIndex option target is not an element".into())
            })?;
            if Some(index) == selected_position {
                option_element
                    .attrs
                    .insert("selected".to_string(), "true".to_string());
            } else {
                option_element.attrs.remove("selected");
            }
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
        if self.dom.find_ancestor_by_tag(button_node, "datalist").is_some() {
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
                                handler: function.handler.clone(),
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

    pub(crate) fn ensure_document_selection_object(&mut self) -> Value {
        let is_selection_object = {
            let entries = self.dom_runtime.selection_object.borrow();
            Self::is_selection_object(&entries)
        };
        if !is_selection_object {
            self.dom_runtime.selection_object =
                match Self::new_selection_object_value(self.dom.root) {
                    Value::Object(selection) => selection,
                    _ => Rc::new(RefCell::new(ObjectValue::default())),
                };
        }
        Value::Object(self.dom_runtime.selection_object.clone())
    }

    fn selection_boundary_node_from_value(&self, value: &Value) -> Result<NodeId> {
        match value {
            Value::Node(node) if self.dom.is_valid_node(*node) => Ok(*node),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    Ok(self.dom.root)
                } else {
                    Err(Error::ScriptRuntime(
                        "Selection boundary container must be a Node".into(),
                    ))
                }
            }
            _ => Err(Error::ScriptRuntime(
                "Selection boundary container must be a Node".into(),
            )),
        }
    }

    fn selection_clamped_offset(&self, node: NodeId, offset: i64) -> i64 {
        let max = match &self.dom.nodes[node.0].node_type {
            NodeType::Text(text) => text.chars().count() as i64,
            NodeType::Document | NodeType::Element(_) => {
                self.dom.nodes[node.0].children.len() as i64
            }
        };
        offset.max(0).min(max)
    }

    fn selection_boundary_char_index_in_subtree(
        &self,
        current: NodeId,
        target: NodeId,
        target_offset: i64,
        prefix: usize,
    ) -> Option<usize> {
        if current == target {
            let clamped = self.selection_clamped_offset(target, target_offset) as usize;
            let index = match &self.dom.nodes[target.0].node_type {
                NodeType::Text(_) => prefix + clamped,
                NodeType::Document | NodeType::Element(_) => {
                    let children = &self.dom.nodes[target.0].children;
                    let upto = clamped.min(children.len());
                    let mut out = prefix;
                    for child in children.iter().take(upto) {
                        out += self.dom.text_content(*child).chars().count();
                    }
                    out
                }
            };
            return Some(index);
        }

        if matches!(self.dom.nodes[current.0].node_type, NodeType::Text(_)) {
            return None;
        }

        let mut running = prefix;
        for child in &self.dom.nodes[current.0].children {
            if let Some(found) = self.selection_boundary_char_index_in_subtree(
                *child,
                target,
                target_offset,
                running,
            ) {
                return Some(found);
            }
            running += self.dom.text_content(*child).chars().count();
        }
        None
    }

    fn selection_boundary_char_index(&self, node: NodeId, offset: i64) -> Option<usize> {
        if !self.dom.is_valid_node(node) {
            return None;
        }
        self.selection_boundary_char_index_in_subtree(self.dom.root, node, offset, 0)
    }

    fn selection_compare_points(
        &self,
        left_node: NodeId,
        left_offset: i64,
        right_node: NodeId,
        right_offset: i64,
    ) -> std::cmp::Ordering {
        let left = self
            .selection_boundary_char_index(left_node, left_offset)
            .unwrap_or(0);
        let right = self
            .selection_boundary_char_index(right_node, right_offset)
            .unwrap_or(0);
        left.cmp(&right)
    }

    fn selection_internal_range_object(
        &mut self,
        selection: &Rc<RefCell<ObjectValue>>,
    ) -> Rc<RefCell<ObjectValue>> {
        let existing = {
            let entries = selection.borrow();
            match Self::object_get_entry(&entries, INTERNAL_SELECTION_RANGE_KEY) {
                Some(Value::Object(range)) => Some(range),
                _ => None,
            }
        };
        if let Some(range) = existing {
            return range;
        }

        let range = match Self::new_range_object_value(self.dom.root) {
            Value::Object(range) => range,
            _ => Rc::new(RefCell::new(ObjectValue::default())),
        };
        Self::object_set_entry(
            &mut selection.borrow_mut(),
            INTERNAL_SELECTION_RANGE_KEY.to_string(),
            Value::Object(range.clone()),
        );
        range
    }

    fn selection_anchor_focus(
        &self,
        selection: &Rc<RefCell<ObjectValue>>,
    ) -> Option<(NodeId, i64, NodeId, i64)> {
        let entries = selection.borrow();
        let anchor_node = match Self::object_get_entry(&entries, "anchorNode") {
            Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
            _ => return None,
        };
        let focus_node = match Self::object_get_entry(&entries, "focusNode") {
            Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
            _ => return None,
        };
        let anchor_offset = match Self::object_get_entry(&entries, "anchorOffset") {
            Some(Value::Number(offset)) => offset,
            _ => 0,
        };
        let focus_offset = match Self::object_get_entry(&entries, "focusOffset") {
            Some(Value::Number(offset)) => offset,
            _ => 0,
        };
        Some((anchor_node, anchor_offset, focus_node, focus_offset))
    }

    fn selection_normalized_boundaries(
        &self,
        selection: &Rc<RefCell<ObjectValue>>,
    ) -> Option<(NodeId, i64, NodeId, i64)> {
        let entries = selection.borrow();
        let has_range = matches!(
            Self::object_get_entry(&entries, "rangeCount"),
            Some(Value::Number(count)) if count > 0
        );
        if !has_range {
            return None;
        }
        let range_object = match Self::object_get_entry(&entries, INTERNAL_SELECTION_RANGE_KEY) {
            Some(Value::Object(range)) => range,
            _ => return None,
        };
        let range_entries = range_object.borrow();
        let start_container =
            match Self::object_get_entry(&range_entries, INTERNAL_RANGE_START_CONTAINER_KEY) {
                Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
                _ => return None,
            };
        let start_offset =
            match Self::object_get_entry(&range_entries, INTERNAL_RANGE_START_OFFSET_KEY) {
                Some(Value::Number(offset)) => offset,
                _ => 0,
            };
        let end_container =
            match Self::object_get_entry(&range_entries, INTERNAL_RANGE_END_CONTAINER_KEY) {
                Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
                _ => return None,
            };
        let end_offset = match Self::object_get_entry(&range_entries, INTERNAL_RANGE_END_OFFSET_KEY)
        {
            Some(Value::Number(offset)) => offset,
            _ => 0,
        };
        Some((start_container, start_offset, end_container, end_offset))
    }

    fn selection_slice_text_by_char_index(value: &str, start: usize, end: usize) -> String {
        if end <= start {
            return String::new();
        }
        let start_byte = Self::char_index_to_byte(value, start);
        let end_byte = Self::char_index_to_byte(value, end);
        value[start_byte..end_byte].to_string()
    }

    fn selection_node_boundary_char_indexes(&self, node: NodeId) -> Option<(usize, usize)> {
        if !self.dom.is_valid_node(node) {
            return None;
        }
        if node == self.dom.root {
            let total = self.dom.text_content(self.dom.root).chars().count();
            return Some((0, total));
        }
        let parent = self.dom.parent(node)?;
        let index = self.dom.nodes[parent.0]
            .children
            .iter()
            .position(|candidate| *candidate == node)?;
        let start = self
            .selection_boundary_char_index(parent, index as i64)
            .unwrap_or(0);
        let end = self
            .selection_boundary_char_index(parent, (index + 1) as i64)
            .unwrap_or(start);
        Some((start, end))
    }

    fn selection_set_state(
        &mut self,
        anchor_node: Option<NodeId>,
        anchor_offset: i64,
        focus_node: Option<NodeId>,
        focus_offset: i64,
    ) -> bool {
        let selection = match self.ensure_document_selection_object() {
            Value::Object(selection) => selection,
            _ => return false,
        };

        let before = {
            let entries = selection.borrow();
            (
                Self::object_get_entry(&entries, "anchorNode"),
                Self::object_get_entry(&entries, "anchorOffset"),
                Self::object_get_entry(&entries, "focusNode"),
                Self::object_get_entry(&entries, "focusOffset"),
                Self::object_get_entry(&entries, "rangeCount"),
                Self::object_get_entry(&entries, "direction"),
            )
        };

        if let (Some(anchor_node), Some(focus_node)) = (anchor_node, focus_node) {
            let anchor_offset = self.selection_clamped_offset(anchor_node, anchor_offset);
            let focus_offset = self.selection_clamped_offset(focus_node, focus_offset);
            let ordering =
                self.selection_compare_points(anchor_node, anchor_offset, focus_node, focus_offset);
            let (start_container, start_offset, end_container, end_offset, direction) =
                match ordering {
                    std::cmp::Ordering::Greater => (
                        focus_node,
                        focus_offset,
                        anchor_node,
                        anchor_offset,
                        "backward",
                    ),
                    std::cmp::Ordering::Equal => {
                        (anchor_node, anchor_offset, focus_node, focus_offset, "none")
                    }
                    std::cmp::Ordering::Less => (
                        anchor_node,
                        anchor_offset,
                        focus_node,
                        focus_offset,
                        "forward",
                    ),
                };
            let range_object = self.selection_internal_range_object(&selection);
            {
                let mut range_entries = range_object.borrow_mut();
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_CONTAINER_KEY.to_string(),
                    Value::Node(start_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_OFFSET_KEY.to_string(),
                    Value::Number(start_offset),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_CONTAINER_KEY.to_string(),
                    Value::Node(end_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_OFFSET_KEY.to_string(),
                    Value::Number(end_offset),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startContainer".to_string(),
                    Value::Node(start_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startOffset".to_string(),
                    Value::Number(start_offset),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endContainer".to_string(),
                    Value::Node(end_container),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endOffset".to_string(),
                    Value::Number(end_offset),
                );
            }
            {
                let mut entries = selection.borrow_mut();
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_SELECTION_RANGE_KEY.to_string(),
                    Value::Object(range_object),
                );
                Self::object_set_entry(
                    &mut entries,
                    "anchorNode".to_string(),
                    Value::Node(anchor_node),
                );
                Self::object_set_entry(
                    &mut entries,
                    "anchorOffset".to_string(),
                    Value::Number(anchor_offset),
                );
                Self::object_set_entry(
                    &mut entries,
                    "focusNode".to_string(),
                    Value::Node(focus_node),
                );
                Self::object_set_entry(
                    &mut entries,
                    "focusOffset".to_string(),
                    Value::Number(focus_offset),
                );
                Self::object_set_entry(
                    &mut entries,
                    "isCollapsed".to_string(),
                    Value::Bool(ordering == std::cmp::Ordering::Equal),
                );
                Self::object_set_entry(&mut entries, "rangeCount".to_string(), Value::Number(1));
                Self::object_set_entry(
                    &mut entries,
                    "type".to_string(),
                    Value::String(if ordering == std::cmp::Ordering::Equal {
                        "Caret".to_string()
                    } else {
                        "Range".to_string()
                    }),
                );
                Self::object_set_entry(
                    &mut entries,
                    "direction".to_string(),
                    Value::String(direction.to_string()),
                );
            }
        } else {
            let range_object = self.selection_internal_range_object(&selection);
            {
                let mut range_entries = range_object.borrow_mut();
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_CONTAINER_KEY.to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_START_OFFSET_KEY.to_string(),
                    Value::Number(0),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_CONTAINER_KEY.to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    INTERNAL_RANGE_END_OFFSET_KEY.to_string(),
                    Value::Number(0),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startContainer".to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "startOffset".to_string(),
                    Value::Number(0),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endContainer".to_string(),
                    Value::Node(self.dom.root),
                );
                Self::object_set_entry(
                    &mut range_entries,
                    "endOffset".to_string(),
                    Value::Number(0),
                );
            }
            {
                let mut entries = selection.borrow_mut();
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_SELECTION_RANGE_KEY.to_string(),
                    Value::Object(range_object),
                );
                Self::object_set_entry(&mut entries, "anchorNode".to_string(), Value::Null);
                Self::object_set_entry(&mut entries, "anchorOffset".to_string(), Value::Number(0));
                Self::object_set_entry(&mut entries, "focusNode".to_string(), Value::Null);
                Self::object_set_entry(&mut entries, "focusOffset".to_string(), Value::Number(0));
                Self::object_set_entry(&mut entries, "isCollapsed".to_string(), Value::Bool(true));
                Self::object_set_entry(&mut entries, "rangeCount".to_string(), Value::Number(0));
                Self::object_set_entry(
                    &mut entries,
                    "type".to_string(),
                    Value::String("None".to_string()),
                );
                Self::object_set_entry(
                    &mut entries,
                    "direction".to_string(),
                    Value::String("none".to_string()),
                );
            }
        }

        let after = {
            let entries = selection.borrow();
            (
                Self::object_get_entry(&entries, "anchorNode"),
                Self::object_get_entry(&entries, "anchorOffset"),
                Self::object_get_entry(&entries, "focusNode"),
                Self::object_get_entry(&entries, "focusOffset"),
                Self::object_get_entry(&entries, "rangeCount"),
                Self::object_get_entry(&entries, "direction"),
            )
        };
        before != after
    }

    pub(crate) fn eval_selection_member_call(
        &mut self,
        selection_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let is_selection = {
            let entries = selection_object.borrow();
            Self::is_selection_object(&entries)
        };
        if !is_selection {
            return Ok(None);
        }

        match member {
            "addRange" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "addRange requires exactly one range argument".into(),
                    ));
                }
                let range = match evaluated_args.first() {
                    Some(Value::Object(object)) => object.clone(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "addRange argument must be a Range".into(),
                        ));
                    }
                };
                let (start_container, start_offset, end_container, end_offset) = {
                    let range_entries = range.borrow();
                    if !Self::is_range_object(&range_entries) {
                        return Err(Error::ScriptRuntime(
                            "addRange argument must be a Range".into(),
                        ));
                    }
                    let start_container = match Self::object_get_entry(
                        &range_entries,
                        INTERNAL_RANGE_START_CONTAINER_KEY,
                    ) {
                        Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "Range boundary container must be a Node".into(),
                            ));
                        }
                    };
                    let start_offset = match Self::object_get_entry(
                        &range_entries,
                        INTERNAL_RANGE_START_OFFSET_KEY,
                    ) {
                        Some(Value::Number(offset)) => offset,
                        _ => 0,
                    };
                    let end_container = match Self::object_get_entry(
                        &range_entries,
                        INTERNAL_RANGE_END_CONTAINER_KEY,
                    ) {
                        Some(Value::Node(node)) if self.dom.is_valid_node(node) => node,
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "Range boundary container must be a Node".into(),
                            ));
                        }
                    };
                    let end_offset =
                        match Self::object_get_entry(&range_entries, INTERNAL_RANGE_END_OFFSET_KEY)
                        {
                            Some(Value::Number(offset)) => offset,
                            _ => 0,
                        };
                    (start_container, start_offset, end_container, end_offset)
                };
                {
                    let mut selection_entries = selection_object.borrow_mut();
                    Self::object_set_entry(
                        &mut selection_entries,
                        INTERNAL_SELECTION_RANGE_KEY.to_string(),
                        Value::Object(range.clone()),
                    );
                }
                let changed = self.selection_set_state(
                    Some(start_container),
                    start_offset,
                    Some(end_container),
                    end_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "collapse" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "collapse requires one or two arguments".into(),
                    ));
                }
                let changed = match evaluated_args.first() {
                    Some(Value::Null) => self.selection_set_state(None, 0, None, 0),
                    Some(boundary) => {
                        let node = self.selection_boundary_node_from_value(boundary)?;
                        let offset = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                        if offset < 0 {
                            return Err(Error::ScriptRuntime(
                                "IndexSizeError: offset must be non-negative".into(),
                            ));
                        }
                        self.selection_set_state(Some(node), offset, Some(node), offset)
                    }
                    None => false,
                };
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "collapseToStart" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "collapseToStart takes no arguments".into(),
                    ));
                }
                let Some((start_container, start_offset, _, _)) =
                    self.selection_normalized_boundaries(selection_object)
                else {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Selection has no range".into(),
                    ));
                };
                let changed = self.selection_set_state(
                    Some(start_container),
                    start_offset,
                    Some(start_container),
                    start_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "collapseToEnd" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "collapseToEnd takes no arguments".into(),
                    ));
                }
                let Some((_, _, end_container, end_offset)) =
                    self.selection_normalized_boundaries(selection_object)
                else {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Selection has no range".into(),
                    ));
                };
                let changed = self.selection_set_state(
                    Some(end_container),
                    end_offset,
                    Some(end_container),
                    end_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "containsNode" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "containsNode requires one or two arguments".into(),
                    ));
                }
                let node = match evaluated_args.first() {
                    Some(Value::Node(node)) if self.dom.is_valid_node(*node) => *node,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "containsNode first argument must be a Node".into(),
                        ));
                    }
                };
                let allow_partial = evaluated_args
                    .get(1)
                    .map(|value| value.truthy())
                    .unwrap_or(false);
                let Some((start_container, start_offset, end_container, end_offset)) =
                    self.selection_normalized_boundaries(selection_object)
                else {
                    return Ok(Some(Value::Bool(false)));
                };
                let selection_start = self
                    .selection_boundary_char_index(start_container, start_offset)
                    .unwrap_or(0);
                let selection_end = self
                    .selection_boundary_char_index(end_container, end_offset)
                    .unwrap_or(selection_start);
                let Some((node_start, node_end)) = self.selection_node_boundary_char_indexes(node)
                else {
                    return Ok(Some(Value::Bool(false)));
                };
                let contains = if allow_partial {
                    node_end > selection_start && node_start < selection_end
                } else {
                    node_start >= selection_start && node_end <= selection_end
                };
                Ok(Some(Value::Bool(contains)))
            }
            "deleteFromDocument" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "deleteFromDocument takes no arguments".into(),
                    ));
                }
                let Some((start_container, start_offset, end_container, end_offset)) =
                    self.selection_normalized_boundaries(selection_object)
                else {
                    return Ok(Some(Value::Undefined));
                };
                if self.selection_compare_points(
                    start_container,
                    start_offset,
                    end_container,
                    end_offset,
                ) == std::cmp::Ordering::Equal
                {
                    return Ok(Some(Value::Undefined));
                }

                if start_container == end_container {
                    let start =
                        self.selection_clamped_offset(start_container, start_offset) as usize;
                    let end = self.selection_clamped_offset(end_container, end_offset) as usize;
                    match &mut self.dom.nodes[start_container.0].node_type {
                        NodeType::Text(text) => {
                            let start_byte = Self::char_index_to_byte(text, start);
                            let end_byte = Self::char_index_to_byte(text, end);
                            text.replace_range(start_byte..end_byte, "");
                        }
                        NodeType::Document | NodeType::Element(_) => {
                            if end > start {
                                let targets =
                                    self.dom.nodes[start_container.0].children[start..end].to_vec();
                                for child in targets {
                                    let _ = self.dom.remove_child(start_container, child);
                                }
                            }
                        }
                    }
                } else {
                    let full = self.dom.text_content(self.dom.root);
                    let start = self
                        .selection_boundary_char_index(start_container, start_offset)
                        .unwrap_or(0);
                    let end = self
                        .selection_boundary_char_index(end_container, end_offset)
                        .unwrap_or(start);
                    let start = start.min(full.chars().count());
                    let end = end.min(full.chars().count()).max(start);
                    let mut next = String::new();
                    next.push_str(&Self::selection_slice_text_by_char_index(&full, 0, start));
                    next.push_str(&Self::selection_slice_text_by_char_index(
                        &full,
                        end,
                        full.chars().count(),
                    ));
                    if let Some(body) = self.dom.body().or_else(|| self.dom.document_element()) {
                        let _ = self.dom.set_text_content(body, &next);
                    }
                }

                let changed = self.selection_set_state(
                    Some(start_container),
                    start_offset,
                    Some(start_container),
                    start_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "empty" | "removeAllRanges" => {
                if !evaluated_args.is_empty() {
                    let method = if member == "empty" {
                        "empty"
                    } else {
                        "removeAllRanges"
                    };
                    return Err(Error::ScriptRuntime(format!("{method} takes no arguments")));
                }
                let changed = self.selection_set_state(None, 0, None, 0);
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "extend" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "extend requires exactly two arguments".into(),
                    ));
                }
                let Some((anchor_node, anchor_offset, _, _)) =
                    self.selection_anchor_focus(selection_object)
                else {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Selection has no range".into(),
                    ));
                };
                let focus_node = self.selection_boundary_node_from_value(&evaluated_args[0])?;
                let focus_offset = Self::value_to_i64(&evaluated_args[1]);
                if focus_offset < 0 {
                    return Err(Error::ScriptRuntime(
                        "IndexSizeError: offset must be non-negative".into(),
                    ));
                }
                let changed = self.selection_set_state(
                    Some(anchor_node),
                    anchor_offset,
                    Some(focus_node),
                    focus_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "getComposedRanges" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "getComposedRanges supports at most one argument".into(),
                    ));
                }
                let Some((start_container, start_offset, end_container, end_offset)) =
                    self.selection_normalized_boundaries(selection_object)
                else {
                    return Ok(Some(Self::new_array_value(Vec::new())));
                };
                let range = Self::new_object_value(vec![
                    ("startContainer".to_string(), Value::Node(start_container)),
                    ("startOffset".to_string(), Value::Number(start_offset)),
                    ("endContainer".to_string(), Value::Node(end_container)),
                    ("endOffset".to_string(), Value::Number(end_offset)),
                ]);
                Ok(Some(Self::new_array_value(vec![range])))
            }
            "getRangeAt" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getRangeAt requires exactly one index argument".into(),
                    ));
                }
                let index = Self::value_to_i64(&evaluated_args[0]);
                let has_range = self
                    .selection_normalized_boundaries(selection_object)
                    .is_some();
                if index != 0 || !has_range {
                    return Err(Error::ScriptRuntime(
                        "IndexSizeError: Invalid range index".into(),
                    ));
                }
                let range = self.selection_internal_range_object(selection_object);
                Ok(Some(Value::Object(range)))
            }
            "modify" => {
                if evaluated_args.len() != 3 {
                    return Err(Error::ScriptRuntime(
                        "modify requires exactly three arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "removeRange" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeRange requires exactly one range argument".into(),
                    ));
                }
                let candidate = match evaluated_args.first() {
                    Some(Value::Object(object)) => object.clone(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "removeRange argument must be a Range".into(),
                        ));
                    }
                };
                let current = self.selection_internal_range_object(selection_object);
                if !Rc::ptr_eq(&candidate, &current) {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeRange': The given range isn't in selection"
                            .into(),
                    ));
                }
                let changed = self.selection_set_state(None, 0, None, 0);
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "selectAllChildren" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "selectAllChildren requires exactly one argument".into(),
                    ));
                }
                let node = self.selection_boundary_node_from_value(&evaluated_args[0])?;
                let end = match &self.dom.nodes[node.0].node_type {
                    NodeType::Text(text) => text.chars().count() as i64,
                    NodeType::Document | NodeType::Element(_) => {
                        self.dom.nodes[node.0].children.len() as i64
                    }
                };
                let changed = self.selection_set_state(Some(node), 0, Some(node), end);
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "setBaseAndExtent" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(
                        "setBaseAndExtent requires exactly four arguments".into(),
                    ));
                }
                let anchor_node = self.selection_boundary_node_from_value(&evaluated_args[0])?;
                let anchor_offset = Self::value_to_i64(&evaluated_args[1]);
                let focus_node = self.selection_boundary_node_from_value(&evaluated_args[2])?;
                let focus_offset = Self::value_to_i64(&evaluated_args[3]);
                if anchor_offset < 0 || focus_offset < 0 {
                    return Err(Error::ScriptRuntime(
                        "IndexSizeError: offset must be non-negative".into(),
                    ));
                }
                let changed = self.selection_set_state(
                    Some(anchor_node),
                    anchor_offset,
                    Some(focus_node),
                    focus_offset,
                );
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "setPosition" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "setPosition requires one or two arguments".into(),
                    ));
                }
                let changed = match evaluated_args.first() {
                    Some(Value::Null) => self.selection_set_state(None, 0, None, 0),
                    Some(boundary) => {
                        let node = self.selection_boundary_node_from_value(boundary)?;
                        let offset = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                        if offset < 0 {
                            return Err(Error::ScriptRuntime(
                                "IndexSizeError: offset must be non-negative".into(),
                            ));
                        }
                        self.selection_set_state(Some(node), offset, Some(node), offset)
                    }
                    None => false,
                };
                if changed {
                    let _ = self.dispatch_document_selectionchange()?;
                }
                Ok(Some(Value::Undefined))
            }
            "toString" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("toString takes no arguments".into()));
                }
                let Some((start_container, start_offset, end_container, end_offset)) =
                    self.selection_normalized_boundaries(selection_object)
                else {
                    return Ok(Some(Value::String(String::new())));
                };
                let full = self.dom.text_content(self.dom.root);
                let start = self
                    .selection_boundary_char_index(start_container, start_offset)
                    .unwrap_or(0);
                let end = self
                    .selection_boundary_char_index(end_container, end_offset)
                    .unwrap_or(start);
                let len = full.chars().count();
                let start = start.min(len);
                let end = end.min(len).max(start);
                Ok(Some(Value::String(
                    Self::selection_slice_text_by_char_index(&full, start, end),
                )))
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
        let (is_event_target, is_match_media) = {
            let entries = object.borrow();
            (
                Self::is_event_target_object(&entries),
                Self::is_match_media_object(&entries),
            )
        };
        if !is_event_target {
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
                            handler: function.handler.clone(),
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
                                handler: function.handler.clone(),
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
                let validity = self.compute_input_validity(node)?;
                if !validity.valid {
                    let _ = self.dispatch_event(node, "invalid")?;
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
                self.dispatch_document_scroll_sequence(position_changed)?;
                Ok(Some(Value::Undefined))
            }
            "select" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("select takes no arguments".into()));
                }
                if self.node_supports_text_selection(node) {
                    let len = self.dom.value(node)?.chars().count();
                    self.set_node_selection_range(node, 0, len as i64, "none".to_string())?;
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn canvas_2d_alpha_from_options(options: &Value) -> bool {
        match options {
            Value::Object(entries) => {
                let entries = entries.borrow();
                Self::object_get_entry(&entries, "alpha")
                    .map(|value| value.truthy())
                    .unwrap_or(true)
            }
            _ => true,
        }
    }

    pub(crate) fn eval_canvas_2d_context_member_call(
        &mut self,
        context_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let mut context = context_object.borrow_mut();
        match member {
            "fillRect" | "clearRect" | "strokeRect" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} requires exactly four arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            "fillText" | "strokeText" => {
                if !(evaluated_args.len() == 3 || evaluated_args.len() == 4) {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} requires three or four arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            "measureText" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "measureText supports at most one argument".into(),
                    ));
                }
                let text = evaluated_args
                    .first()
                    .map(Value::as_string)
                    .unwrap_or_else(|| "undefined".to_string());
                let width = text.chars().count() as f64 * 10.0;
                Ok(Some(Self::new_object_value(vec![(
                    "width".to_string(),
                    Self::number_value(width),
                )])))
            }
            "beginPath" | "closePath" | "save" | "restore" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(format!("{member} takes no arguments")));
                }
                Ok(Some(Value::Undefined))
            }
            "fill" | "stroke" | "clip" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} supports at most two arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            "moveTo" | "lineTo" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} requires exactly two arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            "arc" => {
                if evaluated_args.len() < 5 || evaluated_args.len() > 6 {
                    return Err(Error::ScriptRuntime(
                        "arc requires five or six arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "arcTo" => {
                if evaluated_args.len() != 5 {
                    return Err(Error::ScriptRuntime(
                        "arcTo requires exactly five arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "bezierCurveTo" => {
                if evaluated_args.len() != 6 {
                    return Err(Error::ScriptRuntime(
                        "bezierCurveTo requires exactly six arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "quadraticCurveTo" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(
                        "quadraticCurveTo requires exactly four arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "ellipse" => {
                if evaluated_args.len() < 7 || evaluated_args.len() > 8 {
                    return Err(Error::ScriptRuntime(
                        "ellipse requires seven or eight arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "rect" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(
                        "rect requires exactly four arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "roundRect" => {
                if evaluated_args.len() != 5 {
                    return Err(Error::ScriptRuntime(
                        "roundRect requires exactly five arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "drawFocusIfNeeded" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "drawFocusIfNeeded supports at most one argument".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "isPointInPath" | "isPointInStroke" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} requires two or three arguments"
                    )));
                }
                Ok(Some(Value::Bool(false)))
            }
            "setLineDash" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setLineDash requires exactly one argument".into(),
                    ));
                }
                let mut line_dash = self.canvas_2d_line_dash_values(&evaluated_args[0])?;
                if line_dash.len() % 2 == 1 {
                    let copy = line_dash.clone();
                    line_dash.extend(copy);
                }
                Self::canvas_2d_store_line_dash(&mut context, &line_dash);
                Ok(Some(Value::Undefined))
            }
            "getLineDash" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getLineDash takes no arguments".into(),
                    ));
                }
                Ok(Some(Self::new_array_value(
                    Self::canvas_2d_read_line_dash(&context)
                        .into_iter()
                        .map(Self::number_value)
                        .collect(),
                )))
            }
            "createLinearGradient" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(
                        "createLinearGradient requires exactly four arguments".into(),
                    ));
                }
                Ok(Some(Self::new_canvas_gradient_value()))
            }
            "createRadialGradient" => {
                if evaluated_args.len() != 6 {
                    return Err(Error::ScriptRuntime(
                        "createRadialGradient requires exactly six arguments".into(),
                    ));
                }
                Ok(Some(Self::new_canvas_gradient_value()))
            }
            "createConicGradient" => {
                if evaluated_args.len() != 3 {
                    return Err(Error::ScriptRuntime(
                        "createConicGradient requires exactly three arguments".into(),
                    ));
                }
                Ok(Some(Self::new_canvas_gradient_value()))
            }
            "createPattern" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "createPattern requires exactly two arguments".into(),
                    ));
                }
                if matches!(evaluated_args[0], Value::Null | Value::Undefined) {
                    return Ok(Some(Value::Null));
                }
                Ok(Some(Self::new_canvas_pattern_value()))
            }
            "drawImage" => {
                if !(evaluated_args.len() == 3
                    || evaluated_args.len() == 5
                    || evaluated_args.len() == 9)
                {
                    return Err(Error::ScriptRuntime(
                        "drawImage requires three, five, or nine arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "createImageData" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "createImageData requires one or two arguments".into(),
                    ));
                }
                let (width, height) = if evaluated_args.len() == 1 {
                    let Value::Object(entries) = &evaluated_args[0] else {
                        return Err(Error::ScriptRuntime(
                            "createImageData(imageData) requires an ImageData-like object".into(),
                        ));
                    };
                    let entries = entries.borrow();
                    let width = Self::object_get_entry(&entries, "width")
                        .map(|value| Self::coerce_number_for_number_constructor(&value) as i64)
                        .unwrap_or(0)
                        .max(0);
                    let height = Self::object_get_entry(&entries, "height")
                        .map(|value| Self::coerce_number_for_number_constructor(&value) as i64)
                        .unwrap_or(0)
                        .max(0);
                    (width, height)
                } else {
                    (
                        Self::coerce_number_for_number_constructor(&evaluated_args[0]) as i64,
                        Self::coerce_number_for_number_constructor(&evaluated_args[1]) as i64,
                    )
                };
                Ok(Some(self.new_canvas_image_data_value(width, height)?))
            }
            "getImageData" => {
                if evaluated_args.len() != 4 {
                    return Err(Error::ScriptRuntime(
                        "getImageData requires exactly four arguments".into(),
                    ));
                }
                let width =
                    Self::coerce_number_for_number_constructor(&evaluated_args[2]).abs() as i64;
                let height =
                    Self::coerce_number_for_number_constructor(&evaluated_args[3]).abs() as i64;
                Ok(Some(self.new_canvas_image_data_value(width, height)?))
            }
            "putImageData" => {
                if !(evaluated_args.len() == 3 || evaluated_args.len() == 7) {
                    return Err(Error::ScriptRuntime(
                        "putImageData requires three or seven arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "getTransform" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getTransform takes no arguments".into(),
                    ));
                }
                let transform = Self::canvas_2d_read_transform(&context);
                Ok(Some(Self::new_canvas_transform_value(transform)))
            }
            "transform" => {
                if evaluated_args.len() != 6 {
                    return Err(Error::ScriptRuntime(
                        "transform requires exactly six arguments".into(),
                    ));
                }
                let next = [
                    Self::coerce_number_for_number_constructor(&evaluated_args[0]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[1]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[2]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[3]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[4]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[5]),
                ];
                let current = Self::canvas_2d_read_transform(&context);
                Self::canvas_2d_store_transform(
                    &mut context,
                    Self::canvas_2d_multiply_transform(current, next),
                );
                Ok(Some(Value::Undefined))
            }
            "setTransform" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 6) {
                    return Err(Error::ScriptRuntime(
                        "setTransform requires one or six arguments".into(),
                    ));
                }
                let next = if evaluated_args.len() == 1 {
                    let Value::Object(entries) = &evaluated_args[0] else {
                        return Err(Error::ScriptRuntime(
                            "setTransform(matrix) requires an object argument".into(),
                        ));
                    };
                    Self::canvas_2d_transform_from_object_entries(&entries.borrow())
                } else {
                    [
                        Self::coerce_number_for_number_constructor(&evaluated_args[0]),
                        Self::coerce_number_for_number_constructor(&evaluated_args[1]),
                        Self::coerce_number_for_number_constructor(&evaluated_args[2]),
                        Self::coerce_number_for_number_constructor(&evaluated_args[3]),
                        Self::coerce_number_for_number_constructor(&evaluated_args[4]),
                        Self::coerce_number_for_number_constructor(&evaluated_args[5]),
                    ]
                };
                Self::canvas_2d_store_transform(&mut context, next);
                Ok(Some(Value::Undefined))
            }
            "resetTransform" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "resetTransform takes no arguments".into(),
                    ));
                }
                Self::canvas_2d_store_transform(&mut context, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
                Ok(Some(Value::Undefined))
            }
            "scale" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "scale requires exactly two arguments".into(),
                    ));
                }
                let current = Self::canvas_2d_read_transform(&context);
                let next = [
                    Self::coerce_number_for_number_constructor(&evaluated_args[0]),
                    0.0,
                    0.0,
                    Self::coerce_number_for_number_constructor(&evaluated_args[1]),
                    0.0,
                    0.0,
                ];
                Self::canvas_2d_store_transform(
                    &mut context,
                    Self::canvas_2d_multiply_transform(current, next),
                );
                Ok(Some(Value::Undefined))
            }
            "translate" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "translate requires exactly two arguments".into(),
                    ));
                }
                let current = Self::canvas_2d_read_transform(&context);
                let next = [
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    Self::coerce_number_for_number_constructor(&evaluated_args[0]),
                    Self::coerce_number_for_number_constructor(&evaluated_args[1]),
                ];
                Self::canvas_2d_store_transform(
                    &mut context,
                    Self::canvas_2d_multiply_transform(current, next),
                );
                Ok(Some(Value::Undefined))
            }
            "rotate" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "rotate requires exactly one argument".into(),
                    ));
                }
                let radians = Self::coerce_number_for_number_constructor(&evaluated_args[0]);
                let current = Self::canvas_2d_read_transform(&context);
                let next = [
                    radians.cos(),
                    radians.sin(),
                    -radians.sin(),
                    radians.cos(),
                    0.0,
                    0.0,
                ];
                Self::canvas_2d_store_transform(
                    &mut context,
                    Self::canvas_2d_multiply_transform(current, next),
                );
                Ok(Some(Value::Undefined))
            }
            "reset" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("reset takes no arguments".into()));
                }
                Self::canvas_2d_reset_context_state(&mut context);
                Ok(Some(Value::Undefined))
            }
            "getContextAttributes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getContextAttributes takes no arguments".into(),
                    ));
                }
                let alpha = Self::object_get_entry(&context, INTERNAL_CANVAS_2D_ALPHA_KEY)
                    .map(|value| value.truthy())
                    .unwrap_or(true);
                Ok(Some(Self::new_object_value(vec![
                    ("alpha".to_string(), Value::Bool(alpha)),
                    ("colorSpace".to_string(), Value::String("srgb".to_string())),
                    ("desynchronized".to_string(), Value::Bool(false)),
                    ("willReadFrequently".to_string(), Value::Bool(false)),
                ])))
            }
            "isContextLost" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "isContextLost takes no arguments".into(),
                    ));
                }
                Ok(Some(Value::Bool(false)))
            }
            "toString" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("toString takes no arguments".into()));
                }
                Ok(Some(Value::String(
                    "[object CanvasRenderingContext2D]".to_string(),
                )))
            }
            _ => Ok(None),
        }
    }

    fn canvas_2d_line_dash_values(&mut self, value: &Value) -> Result<Vec<f64>> {
        let values = match value {
            Value::Array(values) => values.borrow().elements.clone(),
            Value::TypedArray(values) => self.typed_array_snapshot(values)?,
            _ => vec![value.clone()],
        };
        Ok(values
            .into_iter()
            .map(|entry| Self::coerce_number_for_number_constructor(&entry))
            .map(|entry| {
                if entry.is_finite() && entry >= 0.0 {
                    entry
                } else {
                    0.0
                }
            })
            .collect())
    }

    fn canvas_2d_store_line_dash(context: &mut ObjectValue, line_dash: &[f64]) {
        Self::object_set_entry(
            context,
            INTERNAL_CANVAS_2D_LINE_DASH_KEY.to_string(),
            Self::new_array_value(line_dash.iter().copied().map(Self::number_value).collect()),
        );
    }

    fn canvas_2d_read_line_dash(context: &ObjectValue) -> Vec<f64> {
        let Some(Value::Array(values)) =
            Self::object_get_entry(context, INTERNAL_CANVAS_2D_LINE_DASH_KEY)
        else {
            return Vec::new();
        };
        values
            .borrow()
            .elements
            .iter()
            .map(Self::coerce_number_for_number_constructor)
            .map(|value| if value.is_finite() { value } else { 0.0 })
            .collect()
    }

    fn canvas_2d_transform_from_object_entries(entries: &ObjectValue) -> [f64; 6] {
        let get = |key: &str, default: f64| {
            Self::object_get_entry(entries, key)
                .map(|value| Self::coerce_number_for_number_constructor(&value))
                .filter(|value| value.is_finite())
                .unwrap_or(default)
        };
        [
            get("a", 1.0),
            get("b", 0.0),
            get("c", 0.0),
            get("d", 1.0),
            get("e", 0.0),
            get("f", 0.0),
        ]
    }

    fn canvas_2d_read_transform(context: &ObjectValue) -> [f64; 6] {
        let Some(Value::Array(values)) =
            Self::object_get_entry(context, INTERNAL_CANVAS_2D_TRANSFORM_KEY)
        else {
            return [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        };
        let values = values.borrow();
        let read = |index: usize, default: f64| {
            values
                .elements
                .get(index)
                .map(Self::coerce_number_for_number_constructor)
                .filter(|value| value.is_finite())
                .unwrap_or(default)
        };
        [
            read(0, 1.0),
            read(1, 0.0),
            read(2, 0.0),
            read(3, 1.0),
            read(4, 0.0),
            read(5, 0.0),
        ]
    }

    fn canvas_2d_store_transform(context: &mut ObjectValue, transform: [f64; 6]) {
        Self::object_set_entry(
            context,
            INTERNAL_CANVAS_2D_TRANSFORM_KEY.to_string(),
            Self::new_array_value(transform.into_iter().map(Self::number_value).collect()),
        );
    }

    fn canvas_2d_multiply_transform(left: [f64; 6], right: [f64; 6]) -> [f64; 6] {
        [
            left[0] * right[0] + left[2] * right[1],
            left[1] * right[0] + left[3] * right[1],
            left[0] * right[2] + left[2] * right[3],
            left[1] * right[2] + left[3] * right[3],
            left[0] * right[4] + left[2] * right[5] + left[4],
            left[1] * right[4] + left[3] * right[5] + left[5],
        ]
    }

    fn new_canvas_transform_value(transform: [f64; 6]) -> Value {
        Self::new_object_value(vec![
            ("a".to_string(), Self::number_value(transform[0])),
            ("b".to_string(), Self::number_value(transform[1])),
            ("c".to_string(), Self::number_value(transform[2])),
            ("d".to_string(), Self::number_value(transform[3])),
            ("e".to_string(), Self::number_value(transform[4])),
            ("f".to_string(), Self::number_value(transform[5])),
        ])
    }

    fn new_canvas_gradient_value() -> Value {
        Self::new_object_value(vec![(
            "addColorStop".to_string(),
            Self::new_builtin_placeholder_function(),
        )])
    }

    fn new_canvas_pattern_value() -> Value {
        Self::new_object_value(vec![(
            "setTransform".to_string(),
            Self::new_builtin_placeholder_function(),
        )])
    }

    fn new_canvas_image_data_value(&mut self, width: i64, height: i64) -> Result<Value> {
        self.new_image_data_value(
            width.max(0) as usize,
            height.max(0) as usize,
            TypedArrayKind::Uint8Clamped,
            None,
            "srgb",
            "rgba-unorm8",
        )
    }

    fn canvas_2d_reset_context_state(context: &mut ObjectValue) {
        Self::object_set_entry(
            context,
            "fillStyle".to_string(),
            Value::String("#000000".to_string()),
        );
        Self::object_set_entry(
            context,
            "strokeStyle".to_string(),
            Value::String("#000000".to_string()),
        );
        Self::object_set_entry(context, "lineWidth".to_string(), Value::Number(1));
        Self::object_set_entry(
            context,
            "lineCap".to_string(),
            Value::String("butt".to_string()),
        );
        Self::object_set_entry(
            context,
            "lineJoin".to_string(),
            Value::String("miter".to_string()),
        );
        Self::object_set_entry(context, "miterLimit".to_string(), Value::Number(10));
        Self::object_set_entry(context, "lineDashOffset".to_string(), Value::Number(0));
        Self::object_set_entry(
            context,
            "font".to_string(),
            Value::String("10px sans-serif".to_string()),
        );
        Self::object_set_entry(
            context,
            "textAlign".to_string(),
            Value::String("start".to_string()),
        );
        Self::object_set_entry(
            context,
            "textBaseline".to_string(),
            Value::String("alphabetic".to_string()),
        );
        Self::object_set_entry(
            context,
            "direction".to_string(),
            Value::String("inherit".to_string()),
        );
        Self::object_set_entry(
            context,
            "letterSpacing".to_string(),
            Value::String("0px".to_string()),
        );
        Self::object_set_entry(
            context,
            "fontKerning".to_string(),
            Value::String("auto".to_string()),
        );
        Self::object_set_entry(
            context,
            "fontStretch".to_string(),
            Value::String("normal".to_string()),
        );
        Self::object_set_entry(
            context,
            "fontVariantCaps".to_string(),
            Value::String("normal".to_string()),
        );
        Self::object_set_entry(
            context,
            "textRendering".to_string(),
            Value::String("auto".to_string()),
        );
        Self::object_set_entry(
            context,
            "wordSpacing".to_string(),
            Value::String("0px".to_string()),
        );
        Self::object_set_entry(
            context,
            "lang".to_string(),
            Value::String("inherit".to_string()),
        );
        Self::object_set_entry(context, "shadowBlur".to_string(), Value::Number(0));
        Self::object_set_entry(
            context,
            "shadowColor".to_string(),
            Value::String("rgba(0, 0, 0, 0)".to_string()),
        );
        Self::object_set_entry(context, "shadowOffsetX".to_string(), Value::Number(0));
        Self::object_set_entry(context, "shadowOffsetY".to_string(), Value::Number(0));
        Self::object_set_entry(context, "globalAlpha".to_string(), Value::Number(1));
        Self::object_set_entry(
            context,
            "globalCompositeOperation".to_string(),
            Value::String("source-over".to_string()),
        );
        Self::object_set_entry(
            context,
            "imageSmoothingEnabled".to_string(),
            Value::Bool(true),
        );
        Self::object_set_entry(
            context,
            "imageSmoothingQuality".to_string(),
            Value::String("low".to_string()),
        );
        Self::object_set_entry(
            context,
            "filter".to_string(),
            Value::String("none".to_string()),
        );
        Self::canvas_2d_store_line_dash(context, &[]);
        Self::canvas_2d_store_transform(context, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    pub(crate) fn normalized_input_type(&self, node: NodeId) -> String {
        if !self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false)
        {
            return String::new();
        }
        let raw = self
            .dom
            .attr(node, "type")
            .unwrap_or_default()
            .to_ascii_lowercase();
        match raw.as_str() {
            "button" | "checkbox" | "color" | "date" | "datetime-local" | "email" | "file"
            | "hidden" | "image" | "month" | "number" | "password" | "radio" | "range"
            | "reset" | "search" | "submit" | "tel" | "text" | "time" | "url" | "week" => raw,
            _ => "text".to_string(),
        }
    }

    pub(crate) fn node_supports_text_selection(&self, node: NodeId) -> bool {
        if self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("textarea"))
            .unwrap_or(false)
        {
            return true;
        }
        if !self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false)
        {
            return false;
        }
        matches!(
            self.normalized_input_type(node).as_str(),
            "text" | "search" | "url" | "tel" | "email" | "password"
        )
    }

    pub(crate) fn normalize_selection_direction(direction: &str) -> &'static str {
        match direction {
            "forward" => "forward",
            "backward" => "backward",
            _ => "none",
        }
    }

    fn scroll_offset_from_object_arg(value: &Value, key: &str) -> Option<i64> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        Self::object_get_entry(&entries, key).map(|entry| Self::value_to_i64(&entry))
    }

    fn animate_option_entry(options: Option<&Value>, key: &str) -> Option<Value> {
        let options = options?;
        let Value::Object(entries) = options else {
            return None;
        };
        let entries = entries.borrow();
        Self::object_get_entry(&entries, key)
    }

    fn animate_id_from_options(options: Option<&Value>) -> String {
        match Self::animate_option_entry(options, "id") {
            Some(Value::Null) | Some(Value::Undefined) | None => String::new(),
            Some(value) => value.as_string(),
        }
    }

    fn get_animations_subtree_option(options: Option<&Value>) -> bool {
        let Some(Value::Object(entries)) = options else {
            return false;
        };
        let entries = entries.borrow();
        Self::object_get_entry(&entries, "subtree")
            .map(|value| value.truthy())
            .unwrap_or(false)
    }

    fn register_node_animation(&mut self, target: NodeId, animation: &Value) {
        let Value::Object(animation) = animation else {
            return;
        };
        self.dom_runtime.node_animations.push(NodeAnimationRecord {
            target,
            animation: animation.clone(),
        });
    }

    fn node_get_animations_value(&self, node: NodeId, subtree: bool) -> Value {
        let animations = self
            .dom_runtime
            .node_animations
            .iter()
            .filter(|record| {
                record.target == node || (subtree && self.dom.is_descendant_of(record.target, node))
            })
            .map(|record| Value::Object(record.animation.clone()))
            .collect::<Vec<_>>();
        Self::new_array_value(animations)
    }

    pub(crate) fn apply_document_scroll_operation(&mut self, method: &str, args: &[Value]) -> bool {
        let mut next_x = self.dom_runtime.document_scroll_x;
        let mut next_y = self.dom_runtime.document_scroll_y;

        match method {
            "scroll" | "scrollTo" => match args {
                [] => {}
                [single] => {
                    if matches!(single, Value::Object(_)) {
                        if let Some(left) = Self::scroll_offset_from_object_arg(single, "left") {
                            next_x = left;
                        }
                        if let Some(top) = Self::scroll_offset_from_object_arg(single, "top") {
                            next_y = top;
                        }
                    } else {
                        next_x = Self::value_to_i64(single);
                        next_y = 0;
                    }
                }
                [x, y] => {
                    next_x = Self::value_to_i64(x);
                    next_y = Self::value_to_i64(y);
                }
                _ => {}
            },
            "scrollBy" => {
                let mut delta_x = 0;
                let mut delta_y = 0;
                match args {
                    [] => {}
                    [single] => {
                        if matches!(single, Value::Object(_)) {
                            delta_x =
                                Self::scroll_offset_from_object_arg(single, "left").unwrap_or(0);
                            delta_y =
                                Self::scroll_offset_from_object_arg(single, "top").unwrap_or(0);
                        } else {
                            delta_x = Self::value_to_i64(single);
                        }
                    }
                    [x, y] => {
                        delta_x = Self::value_to_i64(x);
                        delta_y = Self::value_to_i64(y);
                    }
                    _ => {}
                }
                next_x = next_x.saturating_add(delta_x);
                next_y = next_y.saturating_add(delta_y);
            }
            _ => return true,
        }

        let changed = next_x != self.dom_runtime.document_scroll_x
            || next_y != self.dom_runtime.document_scroll_y;
        self.dom_runtime.document_scroll_x = next_x;
        self.dom_runtime.document_scroll_y = next_y;
        changed
    }

    pub(crate) fn set_node_selection_range(
        &mut self,
        node: NodeId,
        start: i64,
        end: i64,
        direction: String,
    ) -> Result<()> {
        if !self.node_supports_text_selection(node) {
            return Ok(());
        }
        let before_start = self.dom.selection_start(node)?;
        let before_end = self.dom.selection_end(node)?;
        let before_direction = self.dom.selection_direction(node)?;
        let start = start.max(0) as usize;
        let end = end.max(0) as usize;
        self.dom.set_selection_range(
            node,
            start,
            end,
            Self::normalize_selection_direction(direction.as_str()),
        )?;
        let after_start = self.dom.selection_start(node)?;
        let after_end = self.dom.selection_end(node)?;
        let after_direction = self.dom.selection_direction(node)?;
        if before_start != after_start
            || before_end != after_end
            || before_direction != after_direction
        {
            let _ = self.dispatch_document_selectionchange()?;
        }
        Ok(())
    }

    pub(crate) fn shift_selection_index(index: usize, delta: i64) -> usize {
        if delta >= 0 {
            index.saturating_add(delta as usize)
        } else {
            index.saturating_sub(delta.unsigned_abs() as usize)
        }
    }

    pub(crate) fn set_node_range_text(&mut self, node: NodeId, args: &[Value]) -> Result<()> {
        if !self.node_supports_text_selection(node) {
            return Ok(());
        }

        let replacement = args[0].as_string();
        let old_value = self.dom.value(node)?;
        let old_len = old_value.chars().count();
        let old_sel_start = self.dom.selection_start(node)?;
        let old_sel_end = self.dom.selection_end(node)?;

        let (mut start, mut end, mode) = match args.len() {
            1 => (old_sel_start, old_sel_end, "preserve".to_string()),
            3 => (
                Self::value_to_i64(&args[1]).max(0) as usize,
                Self::value_to_i64(&args[2]).max(0) as usize,
                "preserve".to_string(),
            ),
            4 => (
                Self::value_to_i64(&args[1]).max(0) as usize,
                Self::value_to_i64(&args[2]).max(0) as usize,
                args[3].as_string(),
            ),
            _ => {
                return Err(Error::ScriptRuntime(
                    "setRangeText supports one, three, or four arguments".into(),
                ));
            }
        };
        start = start.min(old_len);
        end = end.min(old_len);
        if end < start {
            end = start;
        }

        let start_byte = Self::char_index_to_byte(&old_value, start);
        let end_byte = Self::char_index_to_byte(&old_value, end);
        let mut next_value = String::new();
        next_value.push_str(&old_value[..start_byte]);
        next_value.push_str(&replacement);
        next_value.push_str(&old_value[end_byte..]);
        self.dom.set_value(node, &next_value)?;

        let replacement_len = replacement.chars().count();
        let replaced_len = end.saturating_sub(start);
        let delta = replacement_len as i64 - replaced_len as i64;
        let mode = mode.to_ascii_lowercase();
        let (selection_start, selection_end) = match mode.as_str() {
            "select" => (start, start + replacement_len),
            "start" => (start, start),
            "end" => {
                let caret = start + replacement_len;
                (caret, caret)
            }
            _ => {
                if old_sel_end <= start {
                    (old_sel_start, old_sel_end)
                } else if old_sel_start >= end {
                    (
                        Self::shift_selection_index(old_sel_start, delta),
                        Self::shift_selection_index(old_sel_end, delta),
                    )
                } else {
                    let caret = start + replacement_len;
                    (caret, caret)
                }
            }
        };
        self.set_node_selection_range(
            node,
            selection_start as i64,
            selection_end as i64,
            "none".to_string(),
        )
    }

    pub(crate) fn parse_attr_f64(&self, node: NodeId, name: &str) -> Option<f64> {
        self.dom.attr(node, name).and_then(|raw| {
            let raw = raw.trim();
            if raw.is_empty() {
                None
            } else {
                raw.parse::<f64>().ok().filter(|value| value.is_finite())
            }
        })
    }

    pub(crate) fn parse_attr_i64(&self, node: NodeId, name: &str) -> Option<i64> {
        self.dom.attr(node, name).and_then(|raw| {
            let raw = raw.trim();
            if raw.is_empty() {
                None
            } else {
                raw.parse::<i64>().ok()
            }
        })
    }

    pub(crate) fn parse_number_value(raw: &str) -> Option<f64> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        raw.parse::<f64>().ok().filter(|value| value.is_finite())
    }

    pub(crate) fn parse_date_input_value_ms(raw: &str) -> Option<i64> {
        let (year, month, day) = parse_date_input_components(raw)?;
        Some(Self::utc_timestamp_ms_from_components(
            year,
            i64::from(month) - 1,
            i64::from(day),
            0,
            0,
            0,
            0,
        ))
    }

    pub(crate) fn format_date_input_from_timestamp_ms(timestamp_ms: i64) -> String {
        let (year, month, day, ..) = Self::date_components_utc(timestamp_ms);
        if !(0..=9999).contains(&year) {
            return String::new();
        }
        format!("{year:04}-{month:02}-{day:02}")
    }

    pub(crate) fn parse_datetime_local_input_value_ms(raw: &str) -> Option<i64> {
        let (year, month, day, hour, minute) = parse_datetime_local_input_components(raw)?;
        Some(Self::utc_timestamp_ms_from_components(
            year,
            i64::from(month) - 1,
            i64::from(day),
            i64::from(hour),
            i64::from(minute),
            0,
            0,
        ))
    }

    pub(crate) fn format_datetime_local_input_from_timestamp_ms(timestamp_ms: i64) -> String {
        let (year, month, day, hour, minute, ..) = Self::date_components_utc(timestamp_ms);
        if !(0..=9999).contains(&year) {
            return String::new();
        }
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}")
    }

    pub(crate) fn parse_time_input_value_ms(raw: &str) -> Option<i64> {
        let (hour, minute, second, _) = parse_time_input_components(raw)?;
        let total_seconds = i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second);
        Some(total_seconds * 1_000)
    }

    pub(crate) fn format_time_input_from_timestamp_ms(timestamp_ms: i64) -> String {
        let day_ms = 86_400_000i64;
        let wrapped = timestamp_ms.rem_euclid(day_ms);
        let total_seconds = wrapped / 1_000;
        let hour = total_seconds / 3_600;
        let minute = (total_seconds % 3_600) / 60;
        let second = total_seconds % 60;
        if second == 0 {
            format!("{hour:02}:{minute:02}")
        } else {
            format!("{hour:02}:{minute:02}:{second:02}")
        }
    }

    pub(crate) fn format_number_for_input(value: f64) -> String {
        if value.fract().abs() < 1e-9 {
            format!("{:.0}", value)
        } else {
            let mut out = value.to_string();
            if out.contains('.') {
                while out.ends_with('0') {
                    out.pop();
                }
                if out.ends_with('.') {
                    out.pop();
                }
            }
            out
        }
    }

    pub(crate) fn step_input_value(
        &mut self,
        node: NodeId,
        direction: i64,
        count: i64,
    ) -> Result<()> {
        if count == 0 {
            return Ok(());
        }
        let input_type = self.normalized_input_type(node);
        if !matches!(
            input_type.as_str(),
            "number" | "range" | "date" | "datetime-local" | "time"
        ) {
            return Ok(());
        }

        if input_type == "time" {
            let step_attr = self.dom.attr(node, "step").unwrap_or_default();
            let step_seconds = if step_attr.eq_ignore_ascii_case("any") {
                60.0
            } else {
                step_attr
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
                    .unwrap_or(60.0)
            };
            let step_ms = ((step_seconds * 1_000.0).round() as i64).max(1);
            let min = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_time_input_value_ms(&raw));
            let max = self
                .dom
                .attr(node, "max")
                .and_then(|raw| Self::parse_time_input_value_ms(&raw));
            let base = min
                .or_else(|| {
                    self.dom
                        .attr(node, "value")
                        .and_then(|raw| Self::parse_time_input_value_ms(&raw))
                })
                .unwrap_or(0);
            let current = Self::parse_time_input_value_ms(&self.dom.value(node)?).unwrap_or(base);
            let delta = (direction as i128)
                .saturating_mul(count as i128)
                .saturating_mul(step_ms as i128);
            let day_ms = 86_400_000i64;
            let mut next = (((current as i128) + delta)
                .clamp(i128::from(i64::MIN), i128::from(i64::MAX))
                as i64)
                .rem_euclid(day_ms);

            if let (Some(min), Some(max)) = (min, max) {
                if min <= max {
                    if next < min {
                        next = min;
                    }
                    if next > max {
                        next = max;
                    }
                } else {
                    let in_wrapped_range = next >= min || next <= max;
                    if !in_wrapped_range {
                        next = if direction >= 0 { min } else { max };
                    }
                }
            } else {
                if let Some(min) = min {
                    if next < min {
                        next = min;
                    }
                }
                if let Some(max) = max {
                    if next > max {
                        next = max;
                    }
                }
            }

            let next_value = Self::format_time_input_from_timestamp_ms(next);
            return self.dom.set_value(node, &next_value);
        }

        if input_type == "date" {
            let step_attr = self.dom.attr(node, "step").unwrap_or_default();
            let step_days = if step_attr.eq_ignore_ascii_case("any") {
                1.0
            } else {
                step_attr
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
                    .unwrap_or(1.0)
            };
            let step_ms = ((step_days * 86_400_000.0).round() as i64).max(1);
            let base = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                .or_else(|| {
                    self.dom
                        .attr(node, "value")
                        .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                })
                .unwrap_or(0);
            let current = Self::parse_date_input_value_ms(&self.dom.value(node)?).unwrap_or(base);
            let delta = (direction as i128)
                .saturating_mul(count as i128)
                .saturating_mul(step_ms as i128);
            let mut next = ((current as i128) + delta)
                .clamp(i128::from(i64::MIN), i128::from(i64::MAX))
                as i64;
            if let Some(min) = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
            {
                if next < min {
                    next = min;
                }
            }
            if let Some(max) = self
                .dom
                .attr(node, "max")
                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
            {
                if next > max {
                    next = max;
                }
            }
            let next_value = Self::format_date_input_from_timestamp_ms(next);
            return self.dom.set_value(node, &next_value);
        }

        if input_type == "datetime-local" {
            let step_attr = self.dom.attr(node, "step").unwrap_or_default();
            let step_seconds = if step_attr.eq_ignore_ascii_case("any") {
                60.0
            } else {
                step_attr
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
                    .unwrap_or(60.0)
            };
            let step_ms = ((step_seconds * 1_000.0).round() as i64).max(1);
            let base = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                .or_else(|| {
                    self.dom
                        .attr(node, "value")
                        .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                })
                .unwrap_or(0);
            let current =
                Self::parse_datetime_local_input_value_ms(&self.dom.value(node)?).unwrap_or(base);
            let delta = (direction as i128)
                .saturating_mul(count as i128)
                .saturating_mul(step_ms as i128);
            let mut next = ((current as i128) + delta)
                .clamp(i128::from(i64::MIN), i128::from(i64::MAX))
                as i64;
            if let Some(min) = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
            {
                if next < min {
                    next = min;
                }
            }
            if let Some(max) = self
                .dom
                .attr(node, "max")
                .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
            {
                if next > max {
                    next = max;
                }
            }
            let next_value = Self::format_datetime_local_input_from_timestamp_ms(next);
            return self.dom.set_value(node, &next_value);
        }

        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
        let step = if step_attr.eq_ignore_ascii_case("any") {
            1.0
        } else {
            step_attr
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
                .unwrap_or(1.0)
        };
        let base = self
            .parse_attr_f64(node, "min")
            .or_else(|| self.parse_attr_f64(node, "value"))
            .unwrap_or(0.0);
        let current = Self::parse_number_value(&self.dom.value(node)?).unwrap_or(base);
        let mut next = current + (direction as f64) * (count as f64) * step;
        if let Some(min) = self.parse_attr_f64(node, "min") {
            if next < min {
                next = min;
            }
        }
        if let Some(max) = self.parse_attr_f64(node, "max") {
            if next > max {
                next = max;
            }
        }
        self.dom
            .set_value(node, &Self::format_number_for_input(next))
    }

    pub(crate) fn input_value_as_number(&self, node: NodeId) -> Result<f64> {
        let input_type = self.normalized_input_type(node);
        let value = self.dom.value(node)?;
        let number = match input_type.as_str() {
            "number" | "range" => Self::parse_number_value(&value).unwrap_or(f64::NAN),
            "date" => Self::parse_date_input_value_ms(&value)
                .map(|timestamp| timestamp as f64)
                .unwrap_or(f64::NAN),
            "datetime-local" => Self::parse_datetime_local_input_value_ms(&value)
                .map(|timestamp| timestamp as f64)
                .unwrap_or(f64::NAN),
            "time" => Self::parse_time_input_value_ms(&value)
                .map(|timestamp| timestamp as f64)
                .unwrap_or(f64::NAN),
            _ => f64::NAN,
        };
        Ok(number)
    }

    pub(crate) fn set_input_value_as_number(&mut self, node: NodeId, number: f64) -> Result<()> {
        let input_type = self.normalized_input_type(node);
        if input_type == "date" {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            let timestamp_ms = number as i64;
            let formatted = Self::format_date_input_from_timestamp_ms(timestamp_ms);
            return self.dom.set_value(node, &formatted);
        }
        if input_type == "datetime-local" {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            let timestamp_ms = number as i64;
            let formatted = Self::format_datetime_local_input_from_timestamp_ms(timestamp_ms);
            return self.dom.set_value(node, &formatted);
        }
        if input_type == "time" {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            let timestamp_ms = number as i64;
            let formatted = Self::format_time_input_from_timestamp_ms(timestamp_ms);
            return self.dom.set_value(node, &formatted);
        }
        if matches!(input_type.as_str(), "number" | "range") {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            return self
                .dom
                .set_value(node, &Self::format_number_for_input(number));
        }
        self.dom.set_value(node, "")
    }

    pub(crate) fn input_value_as_date_ms(&self, node: NodeId) -> Result<Option<i64>> {
        let input_type = self.normalized_input_type(node);
        if input_type == "date" {
            return Ok(Self::parse_date_input_value_ms(&self.dom.value(node)?));
        }
        if input_type == "datetime-local" {
            return Ok(Self::parse_datetime_local_input_value_ms(
                &self.dom.value(node)?,
            ));
        }
        if input_type == "time" {
            return Ok(Self::parse_time_input_value_ms(&self.dom.value(node)?));
        }
        if !matches!(input_type.as_str(), "date" | "datetime-local" | "time") {
            return Ok(None);
        }
        Ok(None)
    }

    pub(crate) fn set_input_value_as_date_ms(
        &mut self,
        node: NodeId,
        timestamp_ms: Option<i64>,
    ) -> Result<()> {
        let input_type = self.normalized_input_type(node);
        if !matches!(input_type.as_str(), "date" | "datetime-local" | "time") {
            return self.dom.set_value(node, "");
        }

        let Some(timestamp_ms) = timestamp_ms else {
            return self.dom.set_value(node, "");
        };
        let formatted = if input_type == "date" {
            Self::format_date_input_from_timestamp_ms(timestamp_ms)
        } else if input_type == "time" {
            Self::format_time_input_from_timestamp_ms(timestamp_ms)
        } else {
            Self::format_datetime_local_input_from_timestamp_ms(timestamp_ms)
        };
        self.dom.set_value(node, &formatted)
    }

    pub(crate) fn is_radio_group_checked(&self, node: NodeId) -> bool {
        let name = self.dom.attr(node, "name").unwrap_or_default();
        if name.is_empty() {
            return self.dom.checked(node).unwrap_or(false);
        }
        let form = self.dom.find_ancestor_by_tag(node, "form");
        self.dom.all_element_nodes().into_iter().any(|candidate| {
            is_radio_input(&self.dom, candidate)
                && self.dom.attr(candidate, "name").unwrap_or_default() == name
                && self.dom.find_ancestor_by_tag(candidate, "form") == form
                && self.dom.checked(candidate).unwrap_or(false)
        })
    }

    pub(crate) fn is_ascii_email_local_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                '.' | '!'
                    | '#'
                    | '$'
                    | '%'
                    | '&'
                    | '\''
                    | '*'
                    | '+'
                    | '/'
                    | '='
                    | '?'
                    | '^'
                    | '_'
                    | '`'
                    | '{'
                    | '|'
                    | '}'
                    | '~'
                    | '-'
            )
    }

    pub(crate) fn is_valid_email_domain_label(label: &str) -> bool {
        if label.is_empty() || label.len() > 63 {
            return false;
        }

        let mut chars = label.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !first.is_ascii_alphanumeric() {
            return false;
        }

        let mut last = first;
        for ch in chars {
            if !(ch.is_ascii_alphanumeric() || ch == '-') {
                return false;
            }
            last = ch;
        }

        last.is_ascii_alphanumeric()
    }

    pub(crate) fn is_valid_email_domain(domain: &str) -> bool {
        !domain.is_empty() && domain.split('.').all(Self::is_valid_email_domain_label)
    }

    pub(crate) fn is_simple_email(value: &str) -> bool {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return false;
        }
        let Some((local, domain)) = trimmed.split_once('@') else {
            return false;
        };
        if local.is_empty() || domain.is_empty() || domain.contains('@') {
            return false;
        }
        if !local.chars().all(Self::is_ascii_email_local_char) {
            return false;
        }
        Self::is_valid_email_domain(domain)
    }

    pub(crate) fn is_email_address_list(value: &str) -> bool {
        if value.trim().is_empty() {
            return true;
        }

        for part in value.split(',') {
            let part = part.trim();
            if part.is_empty() || !Self::is_simple_email(part) {
                return false;
            }
        }
        true
    }

    pub(crate) fn is_url_like(value: &str) -> bool {
        LocationParts::parse(value).is_some()
    }

    pub(crate) fn input_participates_in_constraint_validation(kind: &str) -> bool {
        !matches!(kind, "button" | "submit" | "reset" | "hidden" | "image")
    }

    pub(crate) fn compute_input_validity(&self, node: NodeId) -> Result<InputValidity> {
        let mut validity = InputValidity {
            valid: true,
            ..InputValidity::default()
        };

        if self.is_effectively_disabled(node) {
            return Ok(validity);
        }

        let Some(tag_name) = self.dom.tag_name(node) else {
            return Ok(validity);
        };
        if tag_name.eq_ignore_ascii_case("textarea") {
            let value = self.dom.value(node)?;
            let required = self.dom.required(node);
            let readonly = self.dom.readonly(node);

            if required && !readonly && value.is_empty() {
                validity.value_missing = true;
            }

            if !value.is_empty() {
                let value_len = value.chars().count() as i64;
                if let Some(min_len) = self.parse_attr_i64(node, "minlength") {
                    if min_len >= 0 && value_len < min_len {
                        validity.too_short = true;
                    }
                }
                if let Some(max_len) = self.parse_attr_i64(node, "maxlength") {
                    if max_len >= 0 && value_len > max_len {
                        validity.too_long = true;
                    }
                }
            }

            validity.custom_error = !self.dom.custom_validity_message(node)?.is_empty();
            validity.valid = !(validity.value_missing
                || validity.type_mismatch
                || validity.pattern_mismatch
                || validity.too_long
                || validity.too_short
                || validity.range_underflow
                || validity.range_overflow
                || validity.step_mismatch
                || validity.bad_input
                || validity.custom_error);
            return Ok(validity);
        }
        if tag_name.eq_ignore_ascii_case("select") {
            let value = self.dom.value(node)?;
            if self.dom.required(node) && value.is_empty() {
                validity.value_missing = true;
            }
            validity.custom_error = !self.dom.custom_validity_message(node)?.is_empty();
            validity.valid = !(validity.value_missing || validity.custom_error);
            return Ok(validity);
        }
        if tag_name.eq_ignore_ascii_case("button") {
            if !self.button_will_validate(node) {
                return Ok(validity);
            }
            let custom_error = !self.dom.custom_validity_message(node)?.is_empty();
            validity.custom_error = custom_error;
            validity.valid = !custom_error;
            return Ok(validity);
        }
        if !tag_name.eq_ignore_ascii_case("input") {
            let custom_error = !self.dom.custom_validity_message(node)?.is_empty();
            validity.custom_error = custom_error;
            validity.valid = !custom_error;
            return Ok(validity);
        }

        let input_type = self.normalized_input_type(node);
        if !Self::input_participates_in_constraint_validation(input_type.as_str()) {
            return Ok(validity);
        }
        let value = self.dom.value(node)?;
        let value_is_empty = value.is_empty();
        let required = self.dom.required(node);
        let readonly = self.dom.readonly(node);
        let multiple = self.dom.attr(node, "multiple").is_some();
        let email_multiple = input_type == "email" && multiple;
        let value_is_effectively_empty = if email_multiple {
            value.trim().is_empty()
        } else {
            value_is_empty
        };

        if required && !readonly && Self::input_supports_required(input_type.as_str()) {
            validity.value_missing = if input_type == "checkbox" {
                !self.dom.checked(node)?
            } else if input_type == "radio" {
                !self.is_radio_group_checked(node)
            } else if email_multiple {
                false
            } else {
                value_is_effectively_empty
            };
        }

        if !value_is_effectively_empty {
            if input_type == "email" {
                validity.type_mismatch = if email_multiple {
                    !Self::is_email_address_list(&value)
                } else {
                    !Self::is_simple_email(&value)
                };
            } else if input_type == "url" {
                validity.type_mismatch = !Self::is_url_like(&value);
            }

            if matches!(
                input_type.as_str(),
                "text" | "search" | "url" | "tel" | "email" | "password"
            ) {
                let value_len = value.chars().count() as i64;
                if let Some(min_len) = self.parse_attr_i64(node, "minlength") {
                    if min_len >= 0 && value_len < min_len {
                        validity.too_short = true;
                    }
                }
                if let Some(max_len) = self.parse_attr_i64(node, "maxlength") {
                    if max_len >= 0 && value_len > max_len {
                        validity.too_long = true;
                    }
                }

                if let Some(pattern) = self.dom.attr(node, "pattern") {
                    if !pattern.is_empty() {
                        let wrapped = format!("^(?:{})$", pattern);
                        if let Ok(regex) = Regex::new(&wrapped) {
                            if input_type == "email" && multiple {
                                for part in value.split(',') {
                                    let part = part.trim();
                                    if part.is_empty() {
                                        continue;
                                    }
                                    match regex.is_match(part) {
                                        Ok(true) => {}
                                        Ok(false) => {
                                            validity.pattern_mismatch = true;
                                            break;
                                        }
                                        Err(_) => {}
                                    }
                                }
                            } else if let Ok(false) = regex.is_match(&value) {
                                validity.pattern_mismatch = true;
                            }
                        }
                    }
                }
            }

            if input_type == "date" {
                match Self::parse_date_input_value_ms(&value) {
                    Some(date_value_ms) => {
                        if let Some(min) = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                        {
                            if date_value_ms < min {
                                validity.range_underflow = true;
                            }
                        }
                        if let Some(max) = self
                            .dom
                            .attr(node, "max")
                            .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                        {
                            if date_value_ms > max {
                                validity.range_overflow = true;
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        if !step_attr.eq_ignore_ascii_case("any") {
                            let step_days = step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(1.0);
                            let step_ms = step_days * 86_400_000.0;
                            let base = self
                                .dom
                                .attr(node, "min")
                                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                                .or_else(|| {
                                    self.dom
                                        .attr(node, "value")
                                        .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                                })
                                .unwrap_or(0) as f64;
                            let ratio = ((date_value_ms as f64) - base) / step_ms;
                            let nearest = ratio.round();
                            if (ratio - nearest).abs() > 1e-7 {
                                validity.step_mismatch = true;
                            }
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            } else if input_type == "datetime-local" {
                match Self::parse_datetime_local_input_value_ms(&value) {
                    Some(datetime_value_ms) => {
                        if let Some(min) = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                        {
                            if datetime_value_ms < min {
                                validity.range_underflow = true;
                            }
                        }
                        if let Some(max) = self
                            .dom
                            .attr(node, "max")
                            .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                        {
                            if datetime_value_ms > max {
                                validity.range_overflow = true;
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        let step_seconds = if step_attr.eq_ignore_ascii_case("any") {
                            60.0
                        } else {
                            step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(60.0)
                        };
                        let step_ms = step_seconds * 1_000.0;
                        let base = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                            .or_else(|| {
                                self.dom
                                    .attr(node, "value")
                                    .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                            })
                            .unwrap_or(0) as f64;
                        let ratio = ((datetime_value_ms as f64) - base) / step_ms;
                        let nearest = ratio.round();
                        if (ratio - nearest).abs() > 1e-7 {
                            validity.step_mismatch = true;
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            } else if input_type == "time" {
                match Self::parse_time_input_value_ms(&value) {
                    Some(time_value_ms) => {
                        let min = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_time_input_value_ms(&raw));
                        let max = self
                            .dom
                            .attr(node, "max")
                            .and_then(|raw| Self::parse_time_input_value_ms(&raw));
                        if let (Some(min), Some(max)) = (min, max) {
                            if min <= max {
                                if time_value_ms < min {
                                    validity.range_underflow = true;
                                }
                                if time_value_ms > max {
                                    validity.range_overflow = true;
                                }
                            } else {
                                let in_wrapped_range = time_value_ms >= min || time_value_ms <= max;
                                if !in_wrapped_range {
                                    validity.range_underflow = true;
                                    validity.range_overflow = true;
                                }
                            }
                        } else {
                            if let Some(min) = min {
                                if time_value_ms < min {
                                    validity.range_underflow = true;
                                }
                            }
                            if let Some(max) = max {
                                if time_value_ms > max {
                                    validity.range_overflow = true;
                                }
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        if !step_attr.eq_ignore_ascii_case("any") {
                            let step_seconds = step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(60.0);
                            let step_ms = step_seconds * 1_000.0;
                            let base = self
                                .dom
                                .attr(node, "min")
                                .and_then(|raw| Self::parse_time_input_value_ms(&raw))
                                .or_else(|| {
                                    self.dom
                                        .attr(node, "value")
                                        .and_then(|raw| Self::parse_time_input_value_ms(&raw))
                                })
                                .unwrap_or(0) as f64;
                            let ratio = ((time_value_ms as f64) - base) / step_ms;
                            let nearest = ratio.round();
                            if (ratio - nearest).abs() > 1e-7 {
                                validity.step_mismatch = true;
                            }
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            } else if matches!(input_type.as_str(), "number" | "range") {
                match Self::parse_number_value(&value) {
                    Some(numeric) => {
                        if let Some(min) = self.parse_attr_f64(node, "min") {
                            if numeric < min {
                                validity.range_underflow = true;
                            }
                        }
                        if let Some(max) = self.parse_attr_f64(node, "max") {
                            if numeric > max {
                                validity.range_overflow = true;
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        if !step_attr.eq_ignore_ascii_case("any") {
                            let step = step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(1.0);
                            let base = self
                                .parse_attr_f64(node, "min")
                                .or_else(|| self.parse_attr_f64(node, "value"))
                                .unwrap_or(0.0);
                            let ratio = (numeric - base) / step;
                            let nearest = ratio.round();
                            if (ratio - nearest).abs() > 1e-7 {
                                validity.step_mismatch = true;
                            }
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            }
        }

        validity.custom_error = !self.dom.custom_validity_message(node)?.is_empty();
        validity.valid = !(validity.value_missing
            || validity.type_mismatch
            || validity.pattern_mismatch
            || validity.too_long
            || validity.too_short
            || validity.range_underflow
            || validity.range_overflow
            || validity.step_mismatch
            || validity.bad_input
            || validity.custom_error);
        Ok(validity)
    }

    pub(crate) fn input_validity_to_value(validity: &InputValidity) -> Value {
        Self::new_object_value(vec![
            (
                "valueMissing".to_string(),
                Value::Bool(validity.value_missing),
            ),
            (
                "typeMismatch".to_string(),
                Value::Bool(validity.type_mismatch),
            ),
            (
                "patternMismatch".to_string(),
                Value::Bool(validity.pattern_mismatch),
            ),
            ("tooLong".to_string(), Value::Bool(validity.too_long)),
            ("tooShort".to_string(), Value::Bool(validity.too_short)),
            (
                "rangeUnderflow".to_string(),
                Value::Bool(validity.range_underflow),
            ),
            (
                "rangeOverflow".to_string(),
                Value::Bool(validity.range_overflow),
            ),
            (
                "stepMismatch".to_string(),
                Value::Bool(validity.step_mismatch),
            ),
            ("badInput".to_string(), Value::Bool(validity.bad_input)),
            (
                "customError".to_string(),
                Value::Bool(validity.custom_error),
            ),
            ("valid".to_string(), Value::Bool(validity.valid)),
        ])
    }
}
