use super::*;

impl Harness {
    pub(crate) fn try_eval_dom_read_layout_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
    ) -> Result<Option<Value>> {
        let value = match prop {
            DomProp::ClientWidth => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["clientWidth"])?
                {
                    value
                } else {
                    Value::Number(self.client_width_property_value(node)?)
                }
            }
            DomProp::ClientHeight => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["clientHeight"])?
                {
                    value
                } else {
                    Value::Number(self.client_height_property_value(node)?)
                }
            }
            DomProp::ClientLeft => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["clientLeft"])?
                {
                    value
                } else {
                    Value::Number(self.dom.client_left(node)?)
                }
            }
            DomProp::ClientTop => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["clientTop"])?
                {
                    value
                } else {
                    Value::Number(self.dom.client_top(node)?)
                }
            }
            DomProp::CurrentCssZoom => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["currentCSSZoom"])?
                {
                    value
                } else {
                    Value::Number(1)
                }
            }
            DomProp::Dataset(key) => {
                let map = self.dom_string_map_live_value(node);
                return self
                    .object_property_from_value_with_receiver(&map, key, &map)
                    .map(Some);
            }
            DomProp::Style(prop) => Value::String(self.dom.style_get(node, prop)?),
            DomProp::OffsetWidth => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["offsetWidth"])?
                {
                    value
                } else {
                    Value::Number(self.dom.offset_width(node)?)
                }
            }
            DomProp::OffsetHeight => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["offsetHeight"])?
                {
                    value
                } else {
                    Value::Number(self.dom.offset_height(node)?)
                }
            }
            DomProp::OffsetLeft => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["offsetLeft"])?
                {
                    value
                } else {
                    Value::Number(self.dom.offset_left(node)?)
                }
            }
            DomProp::OffsetTop => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["offsetTop"])?
                {
                    value
                } else {
                    Value::Number(self.dom.offset_top(node)?)
                }
            }
            DomProp::ScrollWidth => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["scrollWidth"])?
                {
                    value
                } else {
                    Value::Number(self.dom.scroll_width(node)?)
                }
            }
            DomProp::ScrollHeight => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["scrollHeight"])?
                {
                    value
                } else {
                    Value::Number(self.dom.scroll_height(node)?)
                }
            }
            DomProp::ScrollLeft => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["scrollLeft"])?
                {
                    value
                } else {
                    Value::Number(self.dom.scroll_left(node)?)
                }
            }
            DomProp::ScrollTop => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["scrollTop"])?
                {
                    value
                } else {
                    Value::Number(self.dom.scroll_top(node)?)
                }
            }
            DomProp::ScrollLeftMax => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["scrollLeftMax"])?
                {
                    value
                } else {
                    Value::Number(0)
                }
            }
            DomProp::ScrollTopMax => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["scrollTopMax"])?
                {
                    value
                } else {
                    Value::Number(0)
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
