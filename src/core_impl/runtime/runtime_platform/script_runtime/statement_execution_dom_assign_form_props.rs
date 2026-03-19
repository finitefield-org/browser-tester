use super::*;

impl Harness {
    pub(crate) fn try_execute_dom_assign_form_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
        value: &Value,
        env: &mut HashMap<String, Value>,
        event: &mut EventState,
    ) -> Result<bool> {
        match prop {
            DomProp::Value => {
                if self.node_explicit_own_property_overrides_dom_property(node, "value") {
                    self.set_node_assignment_property(node, "value", value.clone(), event, false)?;
                } else if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("li"))
                {
                    let next = Self::value_to_i64(value);
                    self.dom.set_attr(node, "value", &next.to_string())?;
                } else {
                    self.dom.set_value(node, &value.as_string())?;
                }
            }
            DomProp::ValueAsNumber => {
                self.set_input_value_as_number(
                    node,
                    Self::coerce_number_for_number_constructor(value),
                )?;
            }
            DomProp::ValueAsDate => {
                let timestamp_ms = match value {
                    Value::Date(timestamp) => Some(*timestamp.borrow()),
                    Value::Null | Value::Undefined => None,
                    _ => None,
                };
                self.set_input_value_as_date_ms(node, timestamp_ms)?;
            }
            DomProp::SelectionStart => {
                let next_start = Self::value_to_i64(value).max(0) as usize;
                let end = self.dom.selection_end(node).unwrap_or_default();
                self.set_node_selection_range(
                    node,
                    next_start as i64,
                    end as i64,
                    "none".to_string(),
                )?;
            }
            DomProp::SelectionEnd => {
                let start = self.dom.selection_start(node).unwrap_or_default();
                let next_end = Self::value_to_i64(value).max(0) as usize;
                self.set_node_selection_range(
                    node,
                    start as i64,
                    next_end as i64,
                    "none".to_string(),
                )?;
            }
            DomProp::SelectionDirection => {
                let start = self.dom.selection_start(node).unwrap_or_default();
                let end = self.dom.selection_end(node).unwrap_or_default();
                let direction = value.as_string();
                let direction = Self::normalize_selection_direction(direction.as_str());
                self.set_node_selection_range(
                    node,
                    start as i64,
                    end as i64,
                    direction.to_string(),
                )?;
            }
            DomProp::Checked => self.dom.set_checked(node, value.truthy())?,
            DomProp::Indeterminate => self.dom.set_indeterminate(node, value.truthy())?,
            DomProp::Open => {
                if self.node_explicit_own_property_overrides_dom_property(node, "open") {
                    self.set_node_assignment_property(node, "open", value.clone(), event, false)?;
                } else if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
                {
                    let _ = self.set_details_open_state_with_env(node, value.truthy(), env)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "open", value.truthy())?;
                }
            }
            DomProp::ReturnValue => {
                self.set_dialog_return_value(node, value.as_string())?;
            }
            DomProp::ClosedBy => {
                if let Some(shadow_key) =
                    self.node_explicit_own_dom_property_shadow_key(node, &["closedBy", "closedby"])
                {
                    self.set_node_assignment_property(
                        node,
                        shadow_key,
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "closedby", &value.as_string())?;
                }
            }
            DomProp::Action => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("form"))
                {
                    self.dom.set_attr(node, "action", &value.as_string())?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "action".to_string()), value.clone());
                }
            }
            DomProp::FormAction => {
                if self.node_explicit_own_property_overrides_dom_property(node, "formAction") {
                    self.set_node_assignment_property(
                        node,
                        "formAction",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("button") || tag.eq_ignore_ascii_case("input")
                }) {
                    self.dom.set_attr(node, "formaction", &value.as_string())?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "formAction".to_string()), value.clone());
                }
            }
            DomProp::Size => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("select"))
                {
                    self.set_select_size_property_value(node, value)?;
                } else if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    self.set_input_size_property_value(node, value)?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "size".to_string()), value.clone());
                }
            }
            DomProp::Min => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    self.dom.set_attr(node, "min", &value.as_string())?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "min".to_string()), value.clone());
                }
            }
            DomProp::Max => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    self.dom.set_attr(node, "max", &value.as_string())?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "max".to_string()), value.clone());
                }
            }
            DomProp::Step => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    self.dom.set_attr(node, "step", &value.as_string())?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "step".to_string()), value.clone());
                }
            }
            DomProp::MaxLength => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("input") || tag.eq_ignore_ascii_case("textarea")
                }) {
                    self.set_max_length_property_value(node, value)?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "maxLength".to_string()), value.clone());
                }
            }
            DomProp::MinLength => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("input") || tag.eq_ignore_ascii_case("textarea")
                }) {
                    self.set_min_length_property_value(node, value)?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "minLength".to_string()), value.clone());
                }
            }
            DomProp::Rows => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("textarea"))
                {
                    self.set_textarea_rows_property_value(node, value)?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "rows".to_string()), value.clone());
                }
            }
            DomProp::Cols => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("textarea"))
                {
                    self.set_textarea_cols_property_value(node, value)?;
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "cols".to_string()), value.clone());
                }
            }
            DomProp::Files => {
                let files = self.mock_files_from_input_assignment_value(value)?;
                self.dom.set_file_input_files(node, &files)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
