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
}
