use super::*;

impl Harness {
    pub(crate) fn is_node_element_property_key(key: &str) -> bool {
        Self::is_node_element_reflected_property_key(key)
            || Self::is_node_element_metadata_property_key(key)
    }

    pub(crate) fn node_element_property_value(&mut self, node: NodeId, key: &str) -> Result<Value> {
        if Self::is_node_element_reflected_property_key(key) {
            self.node_element_reflected_property_value(node, key)
        } else if Self::is_node_element_metadata_property_key(key) {
            self.node_element_metadata_property_value(node, key)
        } else {
            Ok(Value::Undefined)
        }
    }
}
