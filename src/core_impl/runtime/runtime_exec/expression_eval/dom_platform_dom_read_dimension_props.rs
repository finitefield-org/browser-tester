use super::*;

impl Harness {
    pub(crate) fn try_eval_dom_read_dimension_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
    ) -> Result<Option<Value>> {
        let value = match prop {
            DomProp::Size => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("select"))
                {
                    Value::Number(self.select_size_property_value(node))
                } else if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    Value::Number(self.input_size_property_value_for_node(node))
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "size".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            DomProp::Min => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    Value::String(self.dom.attr(node, "min").unwrap_or_default())
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "min".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            DomProp::Max => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    Value::String(self.dom.attr(node, "max").unwrap_or_default())
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "max".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            DomProp::Step => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    Value::String(self.dom.attr(node, "step").unwrap_or_default())
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "step".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            DomProp::MaxLength => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("input") || tag.eq_ignore_ascii_case("textarea")
                }) {
                    Value::Number(self.max_length_property_value_for_node(node))
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "maxLength".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            DomProp::MinLength => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("input") || tag.eq_ignore_ascii_case("textarea")
                }) {
                    Value::Number(self.min_length_property_value_for_node(node))
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "minLength".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            DomProp::Rows => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("textarea"))
                {
                    Value::Number(self.textarea_rows_property_value_for_node(node))
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "rows".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            DomProp::Cols => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("textarea"))
                {
                    Value::Number(self.textarea_cols_property_value_for_node(node))
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .get(&(node, "cols".to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
