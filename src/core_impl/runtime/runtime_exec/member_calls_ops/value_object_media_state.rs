use super::*;

impl Harness {
    pub(crate) fn media_numeric_state_value(&self, node: NodeId, key: &str, default: f64) -> Value {
        self.dom_runtime
            .node_expando_props
            .get(&(node, key.to_string()))
            .cloned()
            .unwrap_or_else(|| Self::number_value(default))
    }

    pub(crate) fn media_boolean_state_value(
        &self,
        node: NodeId,
        key: &str,
        default: bool,
    ) -> Value {
        match self
            .dom_runtime
            .node_expando_props
            .get(&(node, key.to_string()))
        {
            Some(Value::Bool(value)) => Value::Bool(*value),
            Some(value) => Value::Bool(value.truthy()),
            None => Value::Bool(default),
        }
    }

    pub(crate) fn media_numeric_state_number(&self, node: NodeId, key: &str, default: f64) -> f64 {
        match self
            .dom_runtime
            .node_expando_props
            .get(&(node, key.to_string()))
        {
            Some(Value::Number(value)) => *value as f64,
            Some(Value::Float(value)) => *value,
            Some(value) => Self::coerce_number_for_number_constructor(value),
            None => default,
        }
    }

    pub(crate) fn set_media_numeric_state_value(&mut self, node: NodeId, key: &str, value: &Value) {
        let next = Self::coerce_number_for_number_constructor(value);
        self.dom_runtime
            .node_expando_props
            .insert((node, key.to_string()), Self::number_value(next));
    }

    pub(crate) fn set_media_boolean_state_value(&mut self, node: NodeId, key: &str, next: bool) {
        self.dom_runtime
            .node_expando_props
            .insert((node, key.to_string()), Value::Bool(next));
    }

    pub(crate) fn media_time_ranges_snapshot(&self, media: NodeId, kind: &str) -> Vec<(f64, f64)> {
        let has_src = !self.resolve_media_src(media).is_empty();
        if !has_src {
            return Vec::new();
        }

        let current_time = self
            .media_numeric_state_number(media, INTERNAL_MEDIA_CURRENT_TIME_KEY, 0.0)
            .max(0.0);
        match kind {
            "buffered" | "seekable" => vec![(0.0, current_time)],
            "played" if current_time > 0.0 => vec![(0.0, current_time)],
            _ => Vec::new(),
        }
    }

    fn image_has_resolved_source(&self, node: NodeId) -> bool {
        !self.resolve_media_src(node).is_empty()
    }

    pub(crate) fn image_natural_dimension_value(&self, node: NodeId) -> i64 {
        if self.image_has_resolved_source(node) {
            1
        } else {
            0
        }
    }

    fn radio_node_list_value_string_from_nodes(&self, nodes: &[NodeId]) -> Result<String> {
        for node in nodes {
            if is_radio_input(&self.dom, *node) && self.dom.checked(*node)? {
                return Ok(self.dom.value(*node)?);
            }
        }
        Ok(String::new())
    }

    pub(crate) fn radio_node_list_value_string(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
    ) -> Result<String> {
        let snapshot = self.node_list_snapshot(nodes);
        self.radio_node_list_value_string_from_nodes(&snapshot)
    }

    pub(crate) fn set_radio_node_list_value(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        next_value: &str,
    ) -> Result<()> {
        let snapshot = self.node_list_snapshot(nodes);
        for node in snapshot {
            if is_radio_input(&self.dom, node) && self.dom.value(node)? == next_value {
                self.dom.set_checked(node, true)?;
                break;
            }
        }
        Ok(())
    }
}
