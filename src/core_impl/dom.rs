use super::form_controls::is_radio_input;
use super::html::{is_void_tag, parse_html};
use super::*;

fn is_checkbox_or_radio_input_element(element: &Element) -> bool {
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }
    matches!(
        element
            .attrs
            .get("type")
            .map(|kind| kind.to_ascii_lowercase())
            .as_deref(),
        Some("checkbox") | Some("radio")
    )
}

mod checked_attr_tree_mutation;
mod class_query_basics;
mod connectivity_tree_traversal;
mod core_nodes_identity;
mod dataset_style_layout_props;
mod dump_misc;
mod form_control_value_selection;
mod select_option_sync;
mod selector_matching_pseudo;
mod text_html_content;
