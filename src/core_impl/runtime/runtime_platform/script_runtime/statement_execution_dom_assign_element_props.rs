use super::*;

impl Harness {
    pub(crate) fn try_execute_dom_assign_element_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
        value: &Value,
        event: &mut EventState,
    ) -> Result<bool> {
        match prop {
            DomProp::TextContent | DomProp::InnerText => {
                self.dom.set_text_content(node, &value.as_string())?;
            }
            DomProp::InnerHtml => {
                let html = if matches!(value, Value::Null) {
                    String::new()
                } else {
                    value.as_string()
                };
                self.dom.set_inner_html(node, &html)?;
            }
            DomProp::OuterHtml => {
                let html = if matches!(value, Value::Null) {
                    String::new()
                } else {
                    value.as_string()
                };
                self.dom.set_outer_html(node, &html)?;
            }
            DomProp::Readonly => {
                self.set_reflected_boolean_attribute(node, "readonly", value.truthy())?;
            }
            DomProp::Required => {
                self.set_reflected_boolean_attribute(node, "required", value.truthy())?;
            }
            DomProp::Disabled => {
                self.set_reflected_boolean_attribute(node, "disabled", value.truthy())?;
            }
            DomProp::Hidden => {
                if node == self.dom.root {
                    let call = self.describe_dom_prop(prop);
                    return Err(Error::ScriptRuntime(format!("{call} is read-only")));
                }
                if self.node_explicit_own_property_overrides_dom_property(node, "hidden") {
                    self.set_node_assignment_property(node, "hidden", value.clone(), event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "hidden", value.truthy())?;
                }
            }
            DomProp::ClassName => {
                if self.node_explicit_own_property_overrides_dom_property(node, "className") {
                    self.set_node_assignment_property(
                        node,
                        "className",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "class", &value.as_string())?;
                }
            }
            DomProp::ClassList => self.dom.set_attr(node, "class", &value.as_string())?,
            DomProp::Part => self.dom.set_attr(node, "part", &value.as_string())?,
            DomProp::Id => {
                if self.node_explicit_own_property_overrides_dom_property(node, "id") {
                    self.set_node_assignment_property(node, "id", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "id", &value.as_string())?;
                }
            }
            DomProp::Slot => {
                if self.node_explicit_own_property_overrides_dom_property(node, "slot") {
                    self.set_node_assignment_property(node, "slot", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "slot", &value.as_string())?;
                }
            }
            DomProp::Role => {
                if self.node_explicit_own_property_overrides_dom_property(node, "role") {
                    self.set_node_assignment_property(node, "role", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "role", &value.as_string())?;
                }
            }
            DomProp::ElementTiming => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["elementTiming", "elementtiming"],
                ) {
                    self.set_node_assignment_property(
                        node,
                        shadow_key,
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom
                        .set_attr(node, "elementtiming", &value.as_string())?;
                }
            }
            DomProp::HtmlFor => {
                if self.node_explicit_own_property_overrides_dom_property(node, "htmlFor") {
                    self.set_node_assignment_property(
                        node,
                        "htmlFor",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "for", &value.as_string())?;
                }
            }
            DomProp::Name => {
                if self.node_explicit_own_property_overrides_dom_property(node, "name") {
                    self.set_node_assignment_property(node, "name", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "name", &value.as_string())?;
                }
            }
            DomProp::Lang => {
                if self.node_explicit_own_property_overrides_dom_property(node, "lang") {
                    self.set_node_assignment_property(node, "lang", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "lang", &value.as_string())?;
                }
            }
            DomProp::Dir => {
                if self.node_explicit_own_property_overrides_dom_property(node, "dir") {
                    self.set_node_assignment_property(node, "dir", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "dir", &value.as_string())?;
                }
            }
            DomProp::AccessKey => {
                if self.node_explicit_own_property_overrides_dom_property(node, "accessKey") {
                    self.set_node_assignment_property(
                        node,
                        "accessKey",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "accesskey", &value.as_string())?;
                }
            }
            DomProp::AutoComplete => {
                self.dom
                    .set_attr(node, "autocomplete", &value.as_string())?;
            }
            DomProp::AutoCapitalize => {
                if self.node_explicit_own_property_overrides_dom_property(node, "autocapitalize") {
                    self.set_node_assignment_property(
                        node,
                        "autocapitalize",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom
                        .set_attr(node, "autocapitalize", &value.as_string())?;
                }
            }
            DomProp::AutoCorrect => {
                if self.node_explicit_own_property_overrides_dom_property(node, "autocorrect") {
                    self.set_node_assignment_property(
                        node,
                        "autocorrect",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "autocorrect", &value.as_string())?;
                }
            }
            DomProp::ContentEditable => {
                if self.node_explicit_own_property_overrides_dom_property(node, "contentEditable") {
                    self.set_node_assignment_property(
                        node,
                        "contentEditable",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.set_content_editable_property_value(node, value)?;
                }
            }
            DomProp::Draggable => {
                if self.node_explicit_own_property_overrides_dom_property(node, "draggable") {
                    self.set_node_assignment_property(
                        node,
                        "draggable",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.set_reflected_keyword_boolean_attribute(
                        node,
                        "draggable",
                        value.truthy(),
                        "true",
                        "false",
                    )?;
                }
            }
            DomProp::EnterKeyHint => {
                if self.node_explicit_own_property_overrides_dom_property(node, "enterKeyHint") {
                    self.set_node_assignment_property(
                        node,
                        "enterKeyHint",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom
                        .set_attr(node, "enterkeyhint", &value.as_string())?;
                }
            }
            DomProp::Inert => {
                if self.node_explicit_own_property_overrides_dom_property(node, "inert") {
                    self.set_node_assignment_property(node, "inert", value.clone(), event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "inert", value.truthy())?;
                }
            }
            DomProp::InputMode => {
                if self.node_explicit_own_property_overrides_dom_property(node, "inputMode") {
                    self.set_node_assignment_property(
                        node,
                        "inputMode",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "inputmode", &value.as_string())?;
                }
            }
            DomProp::Nonce => {
                if self.node_explicit_own_property_overrides_dom_property(node, "nonce") {
                    self.set_node_assignment_property(node, "nonce", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "nonce", &value.as_string())?;
                }
            }
            DomProp::Popover => {
                if self.node_explicit_own_property_overrides_dom_property(node, "popover") {
                    self.set_node_assignment_property(
                        node,
                        "popover",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "popover", &value.as_string())?;
                }
            }
            DomProp::Spellcheck => {
                if self.node_explicit_own_property_overrides_dom_property(node, "spellcheck") {
                    self.set_node_assignment_property(
                        node,
                        "spellcheck",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.set_reflected_keyword_boolean_attribute(
                        node,
                        "spellcheck",
                        value.truthy(),
                        "true",
                        "false",
                    )?;
                }
            }
            DomProp::TabIndex => {
                if self.node_explicit_own_property_overrides_dom_property(node, "tabIndex") {
                    self.set_node_assignment_property(
                        node,
                        "tabIndex",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.set_reflected_i64_attribute(node, "tabindex", value)?;
                }
            }
            DomProp::Translate => {
                if self.node_explicit_own_property_overrides_dom_property(node, "translate") {
                    self.set_node_assignment_property(
                        node,
                        "translate",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.set_reflected_keyword_boolean_attribute(
                        node,
                        "translate",
                        value.truthy(),
                        "yes",
                        "no",
                    )?;
                }
            }
            DomProp::Cite => {
                if self.node_explicit_own_property_overrides_dom_property(node, "cite") {
                    self.set_node_assignment_property(node, "cite", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "cite", &value.as_string())?;
                }
            }
            DomProp::DateTime => {
                if let Some(shadow_key) =
                    self.node_explicit_own_dom_property_shadow_key(node, &["dateTime", "datetime"])
                {
                    self.set_node_assignment_property(
                        node,
                        shadow_key,
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "datetime", &value.as_string())?;
                }
            }
            DomProp::BrClear => {
                if self.node_explicit_own_property_overrides_dom_property(node, "clear") {
                    self.set_node_assignment_property(node, "clear", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "clear", &value.as_string())?;
                }
            }
            DomProp::CaptionAlign => {
                if self.node_explicit_own_property_overrides_dom_property(node, "align") {
                    self.set_node_assignment_property(node, "align", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "align", &value.as_string())?;
                }
            }
            DomProp::ColSpan => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("col") || tag.eq_ignore_ascii_case("colgroup")
                }) {
                    self.set_col_span_value(node, value)?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "span".to_string()), value.clone());
                }
            }
            DomProp::TableCellColSpan => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th")
                }) {
                    self.set_table_cell_col_span_value(node, value)?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "colSpan".to_string()), value.clone());
                }
            }
            DomProp::RowSpan => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th")
                }) {
                    self.set_table_cell_row_span_value(node, value)?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "rowSpan".to_string()), value.clone());
                }
            }
            DomProp::NodeEventHandler(event_name) => {
                let _ = self.set_node_event_handler_property(node, event_name, value.clone())?;
            }
            DomProp::BodyDeprecatedAttr(attr_name) => {
                self.dom.set_attr(node, attr_name, &value.as_string())?;
            }
            DomProp::AriaString(prop_name) => {
                let attr_name = Self::aria_property_to_attr_name(prop_name);
                self.dom.set_attr(node, &attr_name, &value.as_string())?;
            }
            DomProp::Dataset(key) => self.dom.dataset_set(node, key, &value.as_string())?,
            DomProp::Style(prop) => self.dom.style_set(node, prop, &value.as_string())?,
            _ => return Ok(false),
        }

        Ok(true)
    }
}
