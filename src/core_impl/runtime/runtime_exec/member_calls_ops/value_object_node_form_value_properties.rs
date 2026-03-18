use super::*;

impl Harness {
    pub(crate) fn is_node_form_value_property_key(key: &str) -> bool {
        matches!(
            key,
            "defaultValue"
                | "value"
                | "files"
                | "valueAsNumber"
                | "valueAsDate"
                | "defaultChecked"
                | "checked"
                | "defaultSelected"
                | "selected"
                | "options"
                | "selectedIndex"
                | "selectedOptions"
                | "size"
                | "min"
                | "max"
                | "step"
                | "maxLength"
                | "maxlength"
                | "minLength"
                | "minlength"
                | "rows"
                | "cols"
                | "validationMessage"
                | "validity"
                | "willValidate"
                | "length"
        )
    }

    pub(crate) fn node_form_value_property_value(
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
                    Ok(Value::Number(
                        self.textarea_rows_property_value_for_node(node),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "cols" => {
                if is_textarea {
                    Ok(Value::Number(
                        self.textarea_cols_property_value_for_node(node),
                    ))
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
