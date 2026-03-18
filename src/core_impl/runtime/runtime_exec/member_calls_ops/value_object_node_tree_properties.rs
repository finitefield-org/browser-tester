use super::*;

impl Harness {
    pub(crate) fn is_node_tree_property_key(key: &str) -> bool {
        matches!(
            key,
            "nodeType"
                | "nodeName"
                | "nodeValue"
                | "ownerDocument"
                | "parentNode"
                | "parentElement"
                | "nextSibling"
                | "previousSibling"
                | "isConnected"
                | "childNodes"
                | "attributes"
                | "children"
                | "childElementCount"
                | "firstChild"
                | "lastChild"
                | "firstElementChild"
                | "lastElementChild"
                | "nextElementSibling"
                | "previousElementSibling"
                | "shadowRoot"
                | "textContent"
                | "innerText"
                | "innerHTML"
                | "outerHTML"
        )
    }

    pub(crate) fn is_node_template_content_property_key(key: &str) -> bool {
        key == "content"
    }

    pub(crate) fn node_tree_property_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Value> {
        match key {
            "nodeType" => Ok(Value::Number(self.node_type_number(node))),
            "nodeName" => Ok(Value::String(self.node_name(node))),
            "nodeValue" => Ok(self.node_value(node)),
            "ownerDocument" => Ok(self.node_owner_document(node).map(Value::Node).unwrap_or(Value::Null)),
            "parentNode" => Ok(self.dom.parent(node).map(Value::Node).unwrap_or(Value::Null)),
            "parentElement" => Ok(self.node_parent_element(node).map(Value::Node).unwrap_or(Value::Null)),
            "nextSibling" => Ok(self.node_next_sibling(node).map(Value::Node).unwrap_or(Value::Null)),
            "previousSibling" => Ok(self
                .node_previous_sibling(node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "isConnected" => Ok(Value::Bool(self.dom.is_connected(node))),
            "childNodes" => Ok(self.child_nodes_live_list_value(node)),
            "attributes" => {
                if self.dom.element(node).is_some() {
                    Ok(self.named_node_map_live_value(node))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "children" => Ok(self.child_elements_live_list_value(node)),
            "childElementCount" => Ok(Value::Number(self.dom.child_element_count(node) as i64)),
            "firstChild" => Ok(self.dom.nodes[node.0]
                .children
                .first()
                .copied()
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "lastChild" => Ok(self.dom.nodes[node.0]
                .children
                .last()
                .copied()
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "firstElementChild" => Ok(self
                .dom
                .first_element_child(node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "lastElementChild" => Ok(self
                .dom
                .last_element_child(node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "nextElementSibling" => Ok(self
                .dom
                .next_element_sibling(node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "previousElementSibling" => Ok(self
                .dom
                .previous_element_sibling(node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "shadowRoot" => Ok(self.shadow_root_property_value(node)),
            "content"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("template")) =>
            {
                self.template_content_fragment_value(node)
            }
            "textContent" => Ok(self.node_text_content_value(node)),
            "innerText" => Ok(Value::String(self.dom.text_content(node))),
            "innerHTML" => Ok(Value::String(self.dom.inner_html(node)?)),
            "outerHTML" => Ok(Value::String(self.dom.outer_html(node)?)),
            _ => Ok(Value::Undefined),
        }
    }
}
