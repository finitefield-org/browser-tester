use super::*;

impl Harness {
    pub(crate) fn try_eval_dom_read_element_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
    ) -> Result<Option<Value>> {
        let value = match prop {
            DomProp::Attributes => {
                self.dom.element(node).ok_or_else(|| {
                    Error::ScriptRuntime("attributes target is not an element".into())
                })?;
                self.named_node_map_live_value(node)
            }
            DomProp::AssignedSlot => Value::Null,
            DomProp::NodeType => Value::Number(self.node_type_number(node)),
            DomProp::TextContent => self.node_text_content_value(node),
            DomProp::InnerText => Value::String(self.dom.text_content(node)),
            DomProp::InnerHtml => Value::String(self.dom.inner_html(node)?),
            DomProp::OuterHtml => Value::String(self.dom.outer_html(node)?),
            DomProp::ClassName => {
                if self.node_explicit_own_property_overrides_dom_property(node, "className") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "className",
                    )? {
                        value
                    } else {
                        Value::String(self.dom.attr(node, "class").unwrap_or_default())
                    }
                } else {
                    Value::String(self.dom.attr(node, "class").unwrap_or_default())
                }
            }
            DomProp::ClassList => self.class_list_live_value(node),
            DomProp::ClassListLength => {
                let list = self.class_list_live_value(node);
                return self
                    .object_property_from_value_with_receiver(&list, "length", &list)
                    .map(Some);
            }
            DomProp::Part => Self::new_array_value(
                class_tokens(self.dom.attr(node, "part").as_deref())
                    .into_iter()
                    .map(Value::String)
                    .collect::<Vec<_>>(),
            ),
            DomProp::PartLength => {
                Value::Number(class_tokens(self.dom.attr(node, "part").as_deref()).len() as i64)
            }
            DomProp::Id => {
                if self.node_explicit_own_property_overrides_dom_property(node, "id") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "id",
                    )? {
                        value
                    } else {
                        Value::String(self.dom.attr(node, "id").unwrap_or_default())
                    }
                } else {
                    Value::String(self.dom.attr(node, "id").unwrap_or_default())
                }
            }
            DomProp::TagName => Value::String(self.element_tag_name(node)),
            DomProp::LocalName => Value::String(
                self.dom
                    .tag_name(node)
                    .map(|name| {
                        name.rsplit_once(':')
                            .map(|(_, local)| local)
                            .unwrap_or(name)
                            .to_ascii_lowercase()
                    })
                    .unwrap_or_default(),
            ),
            DomProp::NamespaceUri => self
                .dom
                .element(node)
                .and_then(|element| element.namespace_uri.clone())
                .map(Value::String)
                .unwrap_or(Value::Null),
            DomProp::Prefix => self
                .dom
                .tag_name(node)
                .and_then(|name| name.split_once(':').map(|(prefix, _)| prefix))
                .map(|prefix| Value::String(prefix.to_string()))
                .unwrap_or(Value::Null),
            DomProp::NextElementSibling => self
                .dom
                .next_element_sibling(node)
                .map(Value::Node)
                .unwrap_or(Value::Null),
            DomProp::PreviousElementSibling => self
                .dom
                .previous_element_sibling(node)
                .map(Value::Node)
                .unwrap_or(Value::Null),
            DomProp::Slot => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["slot"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "slot").unwrap_or_default())
                }
            }
            DomProp::Role => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["role"])?
                {
                    value
                } else {
                    Value::String(self.resolved_role_for_node(node))
                }
            }
            DomProp::ElementTiming => {
                if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                    node,
                    &["elementTiming", "elementtiming"],
                )? {
                    value
                } else {
                    Value::String(self.dom.attr(node, "elementtiming").unwrap_or_default())
                }
            }
            DomProp::HtmlFor => {
                if self.node_explicit_own_property_overrides_dom_property(node, "htmlFor") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "htmlFor",
                    )? {
                        value
                    } else {
                        Value::String(self.dom.attr(node, "for").unwrap_or_default())
                    }
                } else {
                    Value::String(self.dom.attr(node, "for").unwrap_or_default())
                }
            }
            DomProp::Name => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["name"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "name").unwrap_or_default())
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
