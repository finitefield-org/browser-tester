use super::*;

impl Harness {
    pub(crate) fn is_node_element_metadata_property_key(key: &str) -> bool {
        matches!(
            key,
            "tagName"
                | "localName"
                | "namespaceURI"
                | "prefix"
                | "className"
                | "classList"
                | "slot"
                | "role"
                | "baseURI"
                | "dataset"
        )
    }

    pub(crate) fn node_element_metadata_property_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Value> {
        let is_button = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("button"))
            .unwrap_or(false);

        match key {
            "tagName" => Ok(Value::String(self.element_tag_name(node))),
            "localName" => Ok(Value::String(
                self.dom
                    .tag_name(node)
                    .map(|name| {
                        name.rsplit_once(':')
                            .map(|(_, local)| local)
                            .unwrap_or(name)
                            .to_ascii_lowercase()
                    })
                    .unwrap_or_default(),
            )),
            "namespaceURI" => Ok(self
                .dom
                .element(node)
                .and_then(|element| element.namespace_uri.clone())
                .map(Value::String)
                .unwrap_or(Value::Null)),
            "prefix" => Ok(self
                .dom
                .tag_name(node)
                .and_then(|name| name.split_once(':').map(|(prefix, _)| prefix))
                .map(|prefix| Value::String(prefix.to_string()))
                .unwrap_or(Value::Null)),
            "className" => Ok(Value::String(
                self.dom.attr(node, "class").unwrap_or_default(),
            )),
            "classList" => Ok(self.class_list_live_value(node)),
            "slot" => Ok(Value::String(
                self.dom.attr(node, "slot").unwrap_or_default(),
            )),
            "role" => {
                let role = self.resolved_role_for_node(node);
                if role.is_empty() {
                    Ok(Value::Null)
                } else if is_button {
                    Ok(Value::String("button".to_string()))
                } else {
                    Ok(Value::String(role))
                }
            }
            "baseURI" => Ok(Value::String(self.document_base_url())),
            "dataset" => Ok(self.dom_string_map_live_value(node)),
            _ => Ok(Value::Undefined),
        }
    }
}
