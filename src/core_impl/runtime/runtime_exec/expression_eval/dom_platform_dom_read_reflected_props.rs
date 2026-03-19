use super::*;

impl Harness {
    pub(crate) fn try_eval_dom_read_reflected_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
    ) -> Result<Option<Value>> {
        let value = match prop {
            DomProp::Lang => {
                if self.node_explicit_own_property_overrides_dom_property(node, "lang") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "lang",
                    )? {
                        value
                    } else {
                        Value::String(self.dom.attr(node, "lang").unwrap_or_default())
                    }
                } else {
                    Value::String(self.dom.attr(node, "lang").unwrap_or_default())
                }
            }
            DomProp::Dir => {
                if self.node_explicit_own_property_overrides_dom_property(node, "dir") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "dir",
                    )? {
                        value
                    } else {
                        Value::String(self.resolved_dir_for_node(node))
                    }
                } else {
                    Value::String(self.resolved_dir_for_node(node))
                }
            }
            DomProp::AccessKey => {
                if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                    node,
                    &["accessKey", "accesskey"],
                )? {
                    value
                } else {
                    Value::String(self.dom.attr(node, "accesskey").unwrap_or_default())
                }
            }
            DomProp::AutoComplete => {
                Value::String(self.dom.attr(node, "autocomplete").unwrap_or_default())
            }
            DomProp::AutoCapitalize => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["autocapitalize"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "autocapitalize").unwrap_or_default())
                }
            }
            DomProp::AutoCorrect => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["autocorrect"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "autocorrect").unwrap_or_default())
                }
            }
            DomProp::ContentEditable => {
                if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                    node,
                    &["contentEditable", "contenteditable"],
                )? {
                    value
                } else {
                    Value::String(self.content_editable_property_value_for_node(node))
                }
            }
            DomProp::Draggable => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["draggable"])?
                {
                    value
                } else {
                    Value::Bool(self.draggable_property_value_for_node(node))
                }
            }
            DomProp::EnterKeyHint => {
                if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                    node,
                    &["enterKeyHint", "enterkeyhint"],
                )? {
                    value
                } else {
                    Value::String(self.dom.attr(node, "enterkeyhint").unwrap_or_default())
                }
            }
            DomProp::Inert => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["inert"])?
                {
                    value
                } else {
                    Value::Bool(self.dom.has_attr(node, "inert")?)
                }
            }
            DomProp::InputMode => {
                if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                    node,
                    &["inputMode", "inputmode"],
                )? {
                    value
                } else {
                    Value::String(self.dom.attr(node, "inputmode").unwrap_or_default())
                }
            }
            DomProp::Nonce => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["nonce"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "nonce").unwrap_or_default())
                }
            }
            DomProp::Popover => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["popover"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "popover").unwrap_or_default())
                }
            }
            DomProp::Spellcheck => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["spellcheck"])?
                {
                    value
                } else {
                    Value::Bool(self.spellcheck_property_value_for_node(node))
                }
            }
            DomProp::TabIndex => {
                if let Some(value) = self
                    .node_explicit_own_dom_property_shadow_value(node, &["tabIndex", "tabindex"])?
                {
                    value
                } else {
                    Value::Number(self.reflected_i64_attribute_or_default(node, "tabindex", -1))
                }
            }
            DomProp::Translate => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["translate"])?
                {
                    value
                } else {
                    Value::Bool(self.translate_property_value_for_node(node))
                }
            }
            DomProp::Cite => {
                if self.node_explicit_own_property_overrides_dom_property(node, "cite") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "cite",
                    )? {
                        value
                    } else {
                        Value::String(self.reflected_url_attribute_or_empty(node, "cite"))
                    }
                } else {
                    Value::String(self.reflected_url_attribute_or_empty(node, "cite"))
                }
            }
            DomProp::DateTime => {
                if let Some(value) = self
                    .node_explicit_own_dom_property_shadow_value(node, &["dateTime", "datetime"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "datetime").unwrap_or_default())
                }
            }
            DomProp::BrClear => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["clear"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "clear").unwrap_or_default())
                }
            }
            DomProp::CaptionAlign => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["align"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "align").unwrap_or_default())
                }
            }
            DomProp::ColSpan => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("col") || tag.eq_ignore_ascii_case("colgroup")
                }) {
                    Value::Number(self.col_span_value(node))
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "span".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            DomProp::TableCellColSpan => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th")
                }) {
                    Value::Number(self.table_cell_col_span_value(node))
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "colSpan".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            DomProp::RowSpan => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th")
                }) {
                    Value::Number(self.table_cell_row_span_value(node))
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "rowSpan".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            DomProp::CanvasWidth => Value::Number(self.canvas_dimension_value(node, "width")),
            DomProp::CanvasHeight => Value::Number(self.canvas_dimension_value(node, "height")),
            DomProp::NodeEventHandler(event_name) => {
                let is_body_window_alias = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
                    && event_name
                        .strip_prefix("on")
                        .is_some_and(Self::is_body_window_event_handler_alias);
                if is_body_window_alias {
                    Self::object_get_entry(&self.dom_runtime.window_object.borrow(), event_name)
                        .unwrap_or(Value::Null)
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, event_name.clone()))
                        .cloned()
                        .unwrap_or(Value::Null)
                }
            }
            DomProp::BodyDeprecatedAttr(attr_name) => {
                Value::String(self.dom.attr(node, attr_name).unwrap_or_default())
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
