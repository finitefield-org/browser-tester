use super::*;

impl Harness {
    pub(crate) fn is_node_form_control_property_key(key: &str) -> bool {
        Self::is_node_form_value_property_key(key) || Self::is_node_form_relation_property_key(key)
    }

    pub(crate) fn node_form_control_property_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Value> {
        if Self::is_node_form_value_property_key(key) {
            self.node_form_value_property_value(node, key)
        } else if Self::is_node_form_relation_property_key(key) {
            self.node_form_relation_property_value(node, key)
        } else {
            Ok(Value::Undefined)
        }
    }
}
