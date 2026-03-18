use super::*;

impl Harness {
    pub(crate) fn is_node_form_relation_property_key(key: &str) -> bool {
        matches!(
            key,
            "disabled"
                | "required"
                | "multiple"
                | "readonly"
                | "readOnly"
                | "autocomplete"
                | "form"
                | "elements"
                | "action"
                | "method"
                | "enctype"
                | "encoding"
                | "acceptCharset"
                | "noValidate"
                | "command"
                | "commandForElement"
                | "formAction"
                | "formEnctype"
                | "formMethod"
                | "formNoValidate"
                | "formTarget"
                | "labels"
                | "interestForElement"
                | "popoverTargetAction"
                | "popoverTargetElement"
        )
    }

    pub(crate) fn node_form_relation_property_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Value> {
        let is_select = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("select"))
            .unwrap_or(false);
        let is_input = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false);
        let is_button = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("button"))
            .unwrap_or(false);
        let is_form = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("form"))
            .unwrap_or(false);
        let is_form_associated_control = is_form_control(&self.dom, node);
        let is_labelable_control = self.is_labelable_control(node);

        match key {
            "disabled" => Ok(Value::Bool(self.dom.disabled(node))),
            "required" => Ok(Value::Bool(self.dom.required(node))),
            "multiple" => {
                if is_select || is_input {
                    Ok(Value::Bool(self.dom.attr(node, "multiple").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "readonly" | "readOnly" => Ok(Value::Bool(self.dom.readonly(node))),
            "autocomplete" => Ok(Value::String(
                self.dom.attr(node, "autocomplete").unwrap_or_default(),
            )),
            "form" => {
                if is_form_associated_control {
                    Ok(self.resolve_form_for_submit(node).map(Value::Node).unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "elements" => {
                if is_form {
                    self.form_elements_live_list_value(node)
                } else {
                    Ok(Value::Undefined)
                }
            }
            "action" => {
                if is_form {
                    Ok(Value::String(self.form_action_property_value_for_node(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "method" => {
                if is_form {
                    Ok(Value::String(self.dom.attr(node, "method").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "enctype" | "encoding" => {
                if is_form {
                    Ok(Value::String(self.dom.attr(node, "enctype").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "acceptCharset" => {
                if is_form {
                    Ok(Value::String(
                        self.dom.attr(node, "accept-charset").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "noValidate" => {
                if is_form {
                    Ok(Value::Bool(self.dom.attr(node, "novalidate").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "command" => {
                if is_button {
                    Ok(Value::String(self.dom.attr(node, "command").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "commandForElement" => {
                if is_button {
                    Ok(self
                        .dom
                        .attr(node, "commandfor")
                        .and_then(|raw| raw.split_whitespace().next().map(str::to_string))
                        .and_then(|id_ref| self.dom.by_id(&id_ref))
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formAction" => {
                if is_button || is_input {
                    Ok(Value::String(
                        self.submitter_form_action_property_value_for_node(node),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formEnctype" => {
                if is_button {
                    Ok(Value::String(self.dom.attr(node, "formenctype").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formMethod" => {
                if is_button {
                    Ok(Value::String(self.dom.attr(node, "formmethod").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formNoValidate" => {
                if is_button {
                    Ok(Value::Bool(self.dom.attr(node, "formnovalidate").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formTarget" => {
                if is_button {
                    Ok(Value::String(self.dom.attr(node, "formtarget").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "labels" => {
                if is_labelable_control {
                    Ok(Self::new_static_node_list_value(
                        self.labels_for_control_node(node),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "interestForElement" => {
                if is_button {
                    Ok(self
                        .dom
                        .attr(node, "interestfor")
                        .and_then(|raw| raw.split_whitespace().next().map(str::to_string))
                        .and_then(|id_ref| self.dom.by_id(&id_ref))
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "popoverTargetAction" => {
                if is_button {
                    Ok(Value::String(
                        self.dom
                            .attr(node, "popovertargetaction")
                            .unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "popoverTargetElement" => {
                if is_button {
                    Ok(self
                        .dom
                        .attr(node, "popovertarget")
                        .and_then(|raw| raw.split_whitespace().next().map(str::to_string))
                        .and_then(|id_ref| self.dom.by_id(&id_ref))
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            _ => Ok(Value::Undefined),
        }
    }
}
