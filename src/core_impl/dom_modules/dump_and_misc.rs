use super::*;

impl Dom {
    pub(crate) fn dump_node(&self, node_id: NodeId) -> String {
        match &self.nodes[node_id.0].node_type {
            NodeType::Document => {
                let mut out = String::new();
                for child in &self.nodes[node_id.0].children {
                    out.push_str(&self.dump_node(*child));
                }
                out
            }
            NodeType::Text(text) => escape_html_text_for_serialization(text),
            NodeType::Element(element) => {
                let mut out = String::new();
                out.push('<');
                out.push_str(&element.tag_name);
                let mut attrs = element.attrs.iter().collect::<Vec<_>>();
                attrs.sort_by(|(left, _), (right, _)| left.cmp(right));
                for (k, v) in attrs {
                    out.push(' ');
                    out.push_str(k);
                    out.push_str("=\"");
                    out.push_str(&escape_html_attr_for_serialization(v));
                    out.push('"');
                }
                out.push('>');
                if is_void_tag(&element.tag_name) {
                    return out;
                }
                let raw_text_container = element.tag_name.eq_ignore_ascii_case("script")
                    || element.tag_name.eq_ignore_ascii_case("style");
                for child in &self.nodes[node_id.0].children {
                    if raw_text_container {
                        match &self.nodes[child.0].node_type {
                            NodeType::Text(text) => out.push_str(text),
                            _ => out.push_str(&self.dump_node(*child)),
                        }
                    } else {
                        out.push_str(&self.dump_node(*child));
                    }
                }
                out.push_str("</");
                out.push_str(&element.tag_name);
                out.push('>');
                out
            }
        }
    }
}
