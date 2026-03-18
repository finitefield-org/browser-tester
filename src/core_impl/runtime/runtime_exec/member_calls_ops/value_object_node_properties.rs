use super::value_object_node_property_routes::NodePropertyRoute;
use super::*;

impl Harness {
    pub(crate) fn object_property_from_node_value(
        &mut self,
        node: &NodeId,
        key: &str,
    ) -> Result<Value> {
        let is_select = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("select"))
            .unwrap_or(false);
        if is_select {
            if let Ok(index) = key.parse::<usize>() {
                return Ok(self
                    .select_option_nodes(*node)
                    .get(index)
                    .copied()
                    .map(Value::Node)
                    .unwrap_or(Value::Undefined));
            }
        }

        if self.node_explicit_own_property_overrides_dom_property(*node, key) {
            let entries = self.node_expando_entries(*node);
            if let Some(value) =
                self.object_property_from_entries_with_getter(&Value::Node(*node), &entries, key)?
            {
                return Ok(value);
            }
        }

        if let Some(value) = self.node_receiver_builtin_method(*node, key) {
            return Ok(value);
        }

        match Self::node_property_route(key) {
            NodePropertyRoute::Tree => self.node_tree_property_value(*node, key),
            NodePropertyRoute::TemplateContent => {
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("template"))
                {
                    self.node_tree_property_value(*node, key)
                } else {
                    self.node_fallback_property_value(*node, key)
                }
            }
            NodePropertyRoute::Form => self.node_form_control_property_value(*node, key),
            NodePropertyRoute::Anchor => self.node_anchor_property_value(*node, key),
            NodePropertyRoute::Element => self.node_element_property_value(*node, key),
            NodePropertyRoute::Media => self.node_media_property_value(*node, key),
            NodePropertyRoute::Fallback => self.node_fallback_property_value(*node, key),
        }
    }
}
