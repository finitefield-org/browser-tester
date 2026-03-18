use super::*;

impl Harness {
    pub(crate) fn is_node_media_property_key(key: &str) -> bool {
        Self::is_node_media_state_property_key(key) || Self::is_node_media_surface_property_key(key)
    }

    pub(crate) fn node_media_property_value(&mut self, node: NodeId, key: &str) -> Result<Value> {
        if Self::is_node_media_state_property_key(key) {
            self.node_media_state_property_value(node, key)
        } else if Self::is_node_media_surface_property_key(key) {
            self.node_media_surface_property_value(node, key)
        } else {
            Ok(Value::Undefined)
        }
    }
}
