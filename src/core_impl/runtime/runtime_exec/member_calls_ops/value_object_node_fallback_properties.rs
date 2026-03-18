use super::*;

impl Harness {
    pub(crate) fn node_fallback_property_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Value> {
        let is_form = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("form"))
            .unwrap_or(false);
        let is_media = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video"))
            .unwrap_or(false);

        if key.starts_with("on") {
            let is_body_window_alias = self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
                && key
                    .strip_prefix("on")
                    .map(|event_type| event_type.to_ascii_lowercase())
                    .is_some_and(|event_type| {
                        Self::is_body_window_event_handler_alias(event_type.as_str())
                    });
            return if is_body_window_alias {
                Ok(
                    Self::object_get_entry(&self.dom_runtime.window_object.borrow(), key)
                        .unwrap_or(Value::Null),
                )
            } else {
                Ok(self
                    .dom_runtime
                    .node_expando_props
                    .get(&(node, key.to_string()))
                    .cloned()
                    .unwrap_or(Value::Null))
            };
        }

        Ok(self
            .dom_runtime
            .node_expando_props
            .get(&(node, key.to_string()))
            .cloned()
            .or(if is_media {
                self.html_media_builtin_property_value(node, key)?
            } else {
                None
            })
            .or(if is_form {
                self.form_builtin_property_value(key)
            } else {
                None
            })
            .or(if is_form {
                self.form_named_property_value(node, key)?
            } else {
                None
            })
            .unwrap_or(Value::Undefined))
    }
}
