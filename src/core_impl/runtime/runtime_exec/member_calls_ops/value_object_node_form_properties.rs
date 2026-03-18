use super::*;

impl Harness {
    pub(crate) fn node_form_control_property_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Value> {
        let is_select = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("select"))
            .unwrap_or(false);
        let is_datalist = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("datalist"))
            .unwrap_or(false);
        let is_input = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false);
        let is_option = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("option"))
            .unwrap_or(false);
        let is_textarea = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("textarea"))
            .unwrap_or(false);
        let is_output = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("output"))
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
            "defaultValue" => {
                if is_input || is_textarea || is_output {
                    Ok(Value::String(
                        self.dom
                            .element(node)
                            .map(|element| element.default_value.clone())
                            .unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "value" => Ok(Value::String(self.dom.value(node)?)),
            "files" => self.input_files_value(node),
            "valueAsNumber" => Ok(Self::number_value(self.input_value_as_number(node)?)),
            "valueAsDate" => Ok(self
                .input_value_as_date_ms(node)?
                .map(Self::new_date_value)
                .unwrap_or(Value::Null)),
            "defaultChecked" => {
                if is_input {
                    Ok(Value::Bool(
                        self.dom
                            .element(node)
                            .map(|element| element.default_checked)
                            .unwrap_or(false),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "checked" => Ok(Value::Bool(self.dom.checked(node)?)),
            "defaultSelected" => {
                if is_option {
                    Ok(Value::Bool(
                        self.dom
                            .element(node)
                            .map(|element| element.default_selected)
                            .unwrap_or(false),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "selected" => {
                if is_option {
                    Ok(Value::Bool(
                        self.dom
                            .element(node)
                            .map(|element| element.selected)
                            .unwrap_or(false),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
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
            "options" => {
                if is_select {
                    Ok(self.select_options_live_list_value(node))
                } else if is_datalist {
                    Ok(self.datalist_options_live_list_value(node))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "selectedIndex" => {
                if is_select {
                    Ok(Value::Number(self.select_selected_index_value(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "selectedOptions" => {
                if is_select {
                    Ok(self.selected_options_live_list_value(node))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "size" => {
                if is_select {
                    Ok(Value::Number(self.select_size_property_value(node)))
                } else if is_input {
                    Ok(Value::Number(self.input_size_property_value_for_node(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "min" | "max" | "step" => {
                if is_input {
                    Ok(Value::String(self.dom.attr(node, key).unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "maxLength" | "maxlength" => {
                if is_input || is_textarea {
                    Ok(Value::Number(self.max_length_property_value_for_node(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "minLength" | "minlength" => {
                if is_input || is_textarea {
                    Ok(Value::Number(self.min_length_property_value_for_node(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "rows" => {
                if is_textarea {
                    Ok(Value::Number(self.textarea_rows_property_value_for_node(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "cols" => {
                if is_textarea {
                    Ok(Value::Number(self.textarea_cols_property_value_for_node(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "validationMessage" => {
                let validity = self.compute_input_validity(node)?;
                if validity.custom_error {
                    Ok(Value::String(self.dom.custom_validity_message(node)?))
                } else {
                    Ok(Value::String(String::new()))
                }
            }
            "validity" => {
                let validity = self.compute_input_validity(node)?;
                Ok(Self::input_validity_to_value(&validity))
            }
            "willValidate" => {
                let will_validate = if is_select {
                    self.select_will_validate(node)
                } else if is_button {
                    self.button_will_validate(node)
                } else if is_textarea {
                    !self.is_effectively_disabled(node)
                } else if is_input {
                    Self::input_participates_in_constraint_validation(
                        self.normalized_input_type(node).as_str(),
                    ) && !self.is_effectively_disabled(node)
                } else {
                    false
                };
                Ok(Value::Bool(will_validate))
            }
            "length" => {
                if is_form {
                    Ok(Value::Number(self.form_elements(node)?.len() as i64))
                } else if is_select {
                    Ok(Value::Number(self.select_option_nodes(node).len() as i64))
                } else {
                    Ok(Value::Undefined)
                }
            }
            _ => Ok(Value::Undefined),
        }
    }
}
