use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NodePropertyRoute {
    Tree,
    TemplateContent,
    Form,
    Anchor,
    Element,
    Media,
    Fallback,
}

impl Harness {
    pub(crate) fn node_property_route(key: &str) -> NodePropertyRoute {
        if Self::is_node_tree_property_key(key) {
            NodePropertyRoute::Tree
        } else if Self::is_node_template_content_property_key(key) {
            NodePropertyRoute::TemplateContent
        } else if Self::is_node_form_control_property_key(key) {
            NodePropertyRoute::Form
        } else if Self::is_node_anchor_property_key(key) {
            NodePropertyRoute::Anchor
        } else if Self::is_node_element_property_key(key) {
            NodePropertyRoute::Element
        } else if Self::is_node_media_property_key(key) {
            NodePropertyRoute::Media
        } else {
            NodePropertyRoute::Fallback
        }
    }
}
