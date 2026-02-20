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

include!("dom_modules/core_nodes_and_identity.rs");
include!("dom_modules/text_and_html_content.rs");
include!("dom_modules/form_control_value_and_selection.rs");
include!("dom_modules/select_and_option_sync.rs");
include!("dom_modules/checked_attr_and_tree_mutation.rs");
include!("dom_modules/dataset_style_and_layout_props.rs");
include!("dom_modules/class_and_query_basics.rs");
include!("dom_modules/connectivity_and_tree_traversal.rs");
include!("dom_modules/selector_matching_and_pseudo.rs");
include!("dom_modules/dump_and_misc.rs");
