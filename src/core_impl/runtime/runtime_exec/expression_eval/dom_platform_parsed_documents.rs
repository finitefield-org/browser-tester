use super::dom_platform_xml_validation::validate_xml_well_formed;
use super::*;

const SVG_PARSERERROR_NAMESPACE_URI: &str = "http://www.mozilla.org/newlayout/xml/parsererror.xml";

impl Harness {
    pub(crate) fn parsed_document_root_from_entries(entries: &[(String, Value)]) -> Option<NodeId> {
        match Self::object_get_entry(entries, INTERNAL_PARSED_DOCUMENT_ROOT_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }
    }

    pub(crate) fn parsed_document_value_from_root(&mut self, root: NodeId) -> Value {
        self.parsed_document_value_from_root_with_content_type(root, "text/html")
    }

    pub(crate) fn parsed_document_value_from_root_with_content_type(
        &mut self,
        root: NodeId,
        content_type: &str,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_PARSED_DOCUMENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_PARSED_DOCUMENT_ROOT_NODE_KEY.to_string(),
                Value::Node(root),
            ),
            (
                INTERNAL_PARSED_DOCUMENT_CONTENT_TYPE_KEY.to_string(),
                Value::String(content_type.to_string()),
            ),
        ])
    }

    pub(crate) fn new_empty_parsed_document_value(&mut self) -> Value {
        let root = self.dom.create_node(None, NodeType::Document);
        self.parsed_document_value_from_root(root)
    }

    fn set_parsed_document_subtree_namespace(
        dom: &mut Dom,
        root: NodeId,
        namespace_uri: Option<String>,
    ) {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            let children = dom.nodes[node.0].children.clone();
            if let NodeType::Element(element) = &mut dom.nodes[node.0].node_type
                && !element.tag_name.eq_ignore_ascii_case("#document-fragment")
            {
                element.namespace_uri = namespace_uri.clone();
            }
            stack.extend(children.into_iter().rev());
        }
    }

    fn new_parsererror_parsed_document_value(
        &mut self,
        content_type: &str,
        message: &str,
    ) -> Result<Value> {
        let parsed_root = self.dom.create_node(None, NodeType::Document);
        let parsererror = self.dom.create_detached_element_with_namespace(
            "parsererror".to_string(),
            Some(SVG_PARSERERROR_NAMESPACE_URI.to_string()),
        );
        self.dom.append_child(parsed_root, parsererror)?;
        self.dom.create_text(parsererror, message.to_string());
        Ok(self.parsed_document_value_from_root_with_content_type(parsed_root, content_type))
    }

    pub(crate) fn new_parsed_document_value_from_markup(
        &mut self,
        markup: &str,
        sanitize: bool,
        content_type: &str,
    ) -> Result<Value> {
        if content_type.eq_ignore_ascii_case("image/svg+xml")
            && let Err(message) = validate_xml_well_formed(markup)
        {
            return self.new_parsererror_parsed_document_value(content_type, &message);
        }
        let ParseOutput {
            dom: mut parsed, ..
        } = parse_html(markup)?;
        if content_type.eq_ignore_ascii_case("image/svg+xml") {
            let svg_namespace = Some("http://www.w3.org/2000/svg".to_string());
            let children = parsed.nodes[parsed.root.0].children.clone();
            for child in children {
                Self::set_parsed_document_subtree_namespace(
                    &mut parsed,
                    child,
                    svg_namespace.clone(),
                );
            }
        }
        let parsed_root = self.dom.create_node(None, NodeType::Document);
        let children = parsed.nodes[parsed.root.0].children.clone();
        for child in children {
            let _ = self
                .dom
                .clone_subtree_from_dom(&parsed, child, Some(parsed_root), sanitize)?;
        }
        Ok(self.parsed_document_value_from_root_with_content_type(parsed_root, content_type))
    }

    fn parsed_document_document_element(&self, root: NodeId) -> Option<NodeId> {
        self.dom.nodes[root.0]
            .children
            .iter()
            .find(|child| {
                self.dom
                    .tag_name(**child)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("html"))
            })
            .copied()
            .or_else(|| {
                self.dom.nodes[root.0]
                    .children
                    .iter()
                    .find(|child| self.dom.element(**child).is_some())
                    .copied()
            })
    }

    fn parsed_document_body(&self, root: NodeId) -> Option<NodeId> {
        let doc_element = self.parsed_document_document_element(root)?;
        self.dom.nodes[doc_element.0]
            .children
            .iter()
            .find(|child| {
                self.dom
                    .tag_name(**child)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
            })
            .copied()
            .or_else(|| self.dom.query_selector_from(&root, "body").ok().flatten())
            .or(Some(doc_element))
    }

    fn parsed_document_head(&self, root: NodeId) -> Option<NodeId> {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("head"))
            {
                return Some(node);
            }
            for child in self.dom.nodes[node.0].children.iter().rev() {
                stack.push(*child);
            }
        }
        None
    }

    fn find_descendant_by_id(&self, root: NodeId, id: &str) -> Option<NodeId> {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if self.dom.attr(node, "id").is_some_and(|value| value == id) {
                return Some(node);
            }
            for child in self.dom.nodes[node.0].children.iter().rev() {
                stack.push(*child);
            }
        }
        None
    }

    pub(crate) fn parsed_document_property_from_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Result<Option<Value>> {
        let Some(root) = Self::parsed_document_root_from_entries(entries) else {
            return Ok(None);
        };
        Ok(match key {
            "body" => Some(
                self.parsed_document_body(root)
                    .map(Value::Node)
                    .unwrap_or(Value::Null),
            ),
            "head" => Some(
                self.parsed_document_head(root)
                    .map(Value::Node)
                    .unwrap_or(Value::Null),
            ),
            "documentElement" => Some(
                self.parsed_document_document_element(root)
                    .map(Value::Node)
                    .unwrap_or(Value::Null),
            ),
            "contentType" => Some(
                Self::object_get_entry(entries, INTERNAL_PARSED_DOCUMENT_CONTENT_TYPE_KEY)
                    .unwrap_or_else(|| Value::String("text/html".to_string())),
            ),
            "URL" | "documentURI" => Some(Value::String("about:blank".to_string())),
            "createTreeWalker"
            | "querySelector"
            | "querySelectorAll"
            | "getElementById"
            | "getElementsByClassName"
            | "getElementsByName"
            | "getElementsByTagName"
            | "createElement"
            | "createElementNS"
            | "createTextNode"
            | "createAttribute"
            | "createDocumentFragment"
            | "createRange"
            | "append" => Self::placeholder_backed_object_builtin_property_value(entries, key),
            _ => None,
        })
    }

    pub(crate) fn dom_parser_object_property(
        &self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !matches!(
            Self::object_get_entry(entries, INTERNAL_DOM_PARSER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            return None;
        }
        match key {
            "parseFromString" => {
                Self::placeholder_backed_object_builtin_property_value(entries, key)
            }
            _ => None,
        }
    }

    pub(crate) fn xml_serializer_object_property(
        &self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !matches!(
            Self::object_get_entry(entries, INTERNAL_XML_SERIALIZER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            return None;
        }
        match key {
            "serializeToString" => {
                Self::placeholder_backed_object_builtin_property_value(entries, key)
            }
            _ => None,
        }
    }

    fn serialize_xml_target_to_string(&self, target: &Value) -> Result<String> {
        match target {
            Value::Node(node) => Ok(self.dom.dump_node(*node)),
            Value::Object(object) => {
                let entries = object.borrow();
                if matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    return Ok(self.dom.dump_node(self.dom.root));
                }
                if matches!(
                    Self::object_get_entry(&entries, INTERNAL_PARSED_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) && let Some(root) = Self::parsed_document_root_from_entries(&entries)
                {
                    return Ok(self.dom.dump_node(root));
                }
                Err(Error::ScriptRuntime(
                    "XMLSerializer.serializeToString requires a Node".into(),
                ))
            }
            _ => Err(Error::ScriptRuntime(
                "XMLSerializer.serializeToString requires a Node".into(),
            )),
        }
    }
    pub(crate) fn eval_dom_parser_member_call(
        &mut self,
        parser_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let (is_parser, shadowed) = {
            let entries = parser_object.borrow();
            (
                matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOM_PARSER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ),
                Self::placeholder_backed_object_builtin_is_shadowed(&entries, member),
            )
        };
        if !is_parser {
            return Ok(None);
        }
        if shadowed {
            return Ok(None);
        }

        match member {
            "parseFromString" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "DOMParser.parseFromString requires exactly two arguments".into(),
                    ));
                }
                let markup = evaluated_args[0].as_string();
                let mime_type = evaluated_args[1].as_string().to_ascii_lowercase();
                let mime_type = mime_type.trim();
                if !matches!(mime_type, "text/html" | "image/svg+xml") {
                    return Err(Error::ScriptRuntime(
                        "DOMParser.parseFromString supports only 'text/html' and 'image/svg+xml'"
                            .into(),
                    ));
                }

                Ok(Some(self.new_parsed_document_value_from_markup(
                    &markup, false, mime_type,
                )?))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_xml_serializer_member_call(
        &mut self,
        serializer_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let (is_serializer, shadowed) = {
            let entries = serializer_object.borrow();
            (
                matches!(
                    Self::object_get_entry(&entries, INTERNAL_XML_SERIALIZER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ),
                Self::placeholder_backed_object_builtin_is_shadowed(&entries, member),
            )
        };
        if !is_serializer {
            return Ok(None);
        }
        if shadowed {
            return Ok(None);
        }

        match member {
            "serializeToString" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "XMLSerializer.serializeToString requires exactly one argument".into(),
                    ));
                }
                Ok(Some(Value::String(
                    self.serialize_xml_target_to_string(&evaluated_args[0])?,
                )))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_parsed_document_member_call(
        &mut self,
        document_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
        _event: &EventState,
    ) -> Result<Option<Value>> {
        let (root, shadowed) = {
            let entries = document_object.borrow();
            if !matches!(
                Self::object_get_entry(&entries, INTERNAL_PARSED_DOCUMENT_OBJECT_KEY),
                Some(Value::Bool(true))
            ) {
                return Ok(None);
            }
            let Some(root) = Self::parsed_document_root_from_entries(&entries) else {
                return Ok(None);
            };
            (
                root,
                Self::placeholder_backed_object_builtin_is_shadowed(&entries, member),
            )
        };
        if shadowed {
            return Ok(None);
        }

        match member {
            "append" => Ok(Some(self.eval_document_append_call(root, evaluated_args)?)),
            "getElementById" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementById requires exactly one argument".into(),
                    ));
                }
                let id = evaluated_args[0].as_string();
                Ok(Some(
                    self.find_descendant_by_id(root, &id)
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "getElementsByClassName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByClassName requires exactly one argument".into(),
                    ));
                }
                let class_names = Self::class_names_from_argument(&evaluated_args[0]);
                Ok(Some(self.class_names_live_list_value(root, class_names)))
            }
            "getElementsByName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(
                    self.name_live_list_value(root, evaluated_args[0].as_string()),
                ))
            }
            "getElementsByTagName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(self.tag_name_live_list_value(
                    root,
                    Self::tag_name_from_argument(&evaluated_args[0]),
                )))
            }
            "querySelector" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelector requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_query_selector_value(root, &selector)?))
            }
            "querySelectorAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelectorAll requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_query_selector_all_value(root, &selector)?))
            }
            "createTreeWalker" => self.eval_create_tree_walker_call(evaluated_args),
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
                Ok(Some(Self::new_range_object_value(root)))
            }
            _ => Ok(None),
        }
    }
}
