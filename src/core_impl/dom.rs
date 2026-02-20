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

#[path = "dom_modules/checked_attr_tree_mutation.rs"]
mod checked_attr_tree_mutation;
#[path = "dom_modules/class_query_basics.rs"]
mod class_query_basics;
#[path = "dom_modules/connectivity_tree_traversal.rs"]
mod connectivity_tree_traversal;
#[path = "dom_modules/core_nodes_identity.rs"]
mod core_nodes_identity;
#[path = "dom_modules/dataset_style_layout_props.rs"]
mod dataset_style_layout_props;
#[path = "dom_modules/dump_misc.rs"]
mod dump_misc;
#[path = "dom_modules/form_control_value_selection.rs"]
mod form_control_value_selection;
#[path = "dom_modules/select_option_sync.rs"]
mod select_option_sync;
#[path = "dom_modules/selector_matching_pseudo.rs"]
mod selector_matching_pseudo;
#[path = "dom_modules/text_html_content.rs"]
mod text_html_content;
