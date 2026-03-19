use super::*;

impl Harness {
    pub(crate) fn try_eval_dom_read_form_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
    ) -> Result<Option<Value>> {
        let value = match prop {
            DomProp::Value => {
                if self.node_explicit_own_property_overrides_dom_property(node, "value") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "value",
                    )? {
                        value
                    } else if self
                        .dom
                        .tag_name(node)
                        .is_some_and(|tag| tag.eq_ignore_ascii_case("li"))
                    {
                        Value::Number(self.li_value_property(node))
                    } else {
                        Value::String(self.dom.value(node)?)
                    }
                } else if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("li"))
                {
                    Value::Number(self.li_value_property(node))
                } else {
                    Value::String(self.dom.value(node)?)
                }
            }
            DomProp::Files => self.input_files_value(node)?,
            DomProp::FilesLength => match self.input_files_value(node)? {
                Value::Array(values) => Value::Number(values.borrow().len() as i64),
                Value::Null => Value::Number(0),
                _ => Value::Number(0),
            },
            DomProp::ValueAsNumber => Self::number_value(self.input_value_as_number(node)?),
            DomProp::ValueAsDate => self
                .input_value_as_date_ms(node)?
                .map(Self::new_date_value)
                .unwrap_or(Value::Null),
            DomProp::ValueLength => Value::Number(self.dom.value(node)?.chars().count() as i64),
            DomProp::ValidationMessage => {
                let validity = self.compute_input_validity(node)?;
                if validity.custom_error {
                    Value::String(self.dom.custom_validity_message(node)?)
                } else {
                    Value::String(String::new())
                }
            }
            DomProp::Validity => {
                let validity = self.compute_input_validity(node)?;
                Self::input_validity_to_value(&validity)
            }
            DomProp::ValidityValueMissing => {
                Value::Bool(self.compute_input_validity(node)?.value_missing)
            }
            DomProp::ValidityTypeMismatch => {
                Value::Bool(self.compute_input_validity(node)?.type_mismatch)
            }
            DomProp::ValidityPatternMismatch => {
                Value::Bool(self.compute_input_validity(node)?.pattern_mismatch)
            }
            DomProp::ValidityTooLong => Value::Bool(self.compute_input_validity(node)?.too_long),
            DomProp::ValidityTooShort => Value::Bool(self.compute_input_validity(node)?.too_short),
            DomProp::ValidityRangeUnderflow => {
                Value::Bool(self.compute_input_validity(node)?.range_underflow)
            }
            DomProp::ValidityRangeOverflow => {
                Value::Bool(self.compute_input_validity(node)?.range_overflow)
            }
            DomProp::ValidityStepMismatch => {
                Value::Bool(self.compute_input_validity(node)?.step_mismatch)
            }
            DomProp::ValidityBadInput => Value::Bool(self.compute_input_validity(node)?.bad_input),
            DomProp::ValidityValid => Value::Bool(self.compute_input_validity(node)?.valid),
            DomProp::ValidityCustomError => {
                Value::Bool(self.compute_input_validity(node)?.custom_error)
            }
            DomProp::SelectionStart => {
                Value::Number(self.dom.selection_start(node).unwrap_or_default() as i64)
            }
            DomProp::SelectionEnd => {
                Value::Number(self.dom.selection_end(node).unwrap_or_default() as i64)
            }
            DomProp::SelectionDirection => Value::String(
                self.dom
                    .selection_direction(node)
                    .unwrap_or_else(|_| "none".to_string()),
            ),
            DomProp::Checked => Value::Bool(self.dom.checked(node)?),
            DomProp::Indeterminate => Value::Bool(self.dom.indeterminate(node)?),
            DomProp::Open => {
                if self.node_explicit_own_property_overrides_dom_property(node, "open") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "open",
                    )? {
                        value
                    } else {
                        Value::Bool(self.dom.has_attr(node, "open")?)
                    }
                } else {
                    Value::Bool(self.dom.has_attr(node, "open")?)
                }
            }
            DomProp::ReturnValue => Value::String(self.dialog_return_value(node)?),
            DomProp::ClosedBy => {
                if let Some(value) = self
                    .node_explicit_own_dom_property_shadow_value(node, &["closedBy", "closedby"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "closedby").unwrap_or_default())
                }
            }
            DomProp::Readonly => Value::Bool(self.dom.readonly(node)),
            DomProp::Disabled => Value::Bool(self.dom.disabled(node)),
            DomProp::Required => Value::Bool(self.dom.required(node)),
            DomProp::Action => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("form"))
                {
                    if self.node_has_explicit_own_property(node, "action") {
                        let entries = self.node_expando_entries(node);
                        if let Some(value) = self.object_property_from_entries_with_getter(
                            &Value::Node(node),
                            &entries,
                            "action",
                        )? {
                            value
                        } else {
                            Value::String(self.form_action_property_value_for_node(node))
                        }
                    } else {
                        Value::String(self.form_action_property_value_for_node(node))
                    }
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "action".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            DomProp::FormAction => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("button") || tag.eq_ignore_ascii_case("input")
                }) {
                    if self.node_explicit_own_property_overrides_dom_property(node, "formAction") {
                        let entries = self.node_expando_entries(node);
                        if let Some(value) = self.object_property_from_entries_with_getter(
                            &Value::Node(node),
                            &entries,
                            "formAction",
                        )? {
                            value
                        } else {
                            Value::String(self.submitter_form_action_property_value_for_node(node))
                        }
                    } else {
                        Value::String(self.submitter_form_action_property_value_for_node(node))
                    }
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "formAction".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
