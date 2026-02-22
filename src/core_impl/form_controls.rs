use super::*;

fn normalized_button_type(value: Option<&str>) -> &'static str {
    let Some(value) = value else {
        return "submit";
    };
    let normalized = value.trim();
    if normalized.eq_ignore_ascii_case("reset") {
        "reset"
    } else if normalized.eq_ignore_ascii_case("button") {
        "button"
    } else {
        "submit"
    }
}

pub(super) fn is_form_control(dom: &Dom, node_id: NodeId) -> bool {
    let Some(element) = dom.element(node_id) else {
        return false;
    };

    element.tag_name.eq_ignore_ascii_case("input")
        || element.tag_name.eq_ignore_ascii_case("select")
        || element.tag_name.eq_ignore_ascii_case("textarea")
        || element.tag_name.eq_ignore_ascii_case("button")
        || element.tag_name.eq_ignore_ascii_case("output")
}

pub(super) fn is_checkbox_input(dom: &Dom, node_id: NodeId) -> bool {
    let Some(element) = dom.element(node_id) else {
        return false;
    };

    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("checkbox"))
        .unwrap_or(false)
}

pub(super) fn is_radio_input(dom: &Dom, node_id: NodeId) -> bool {
    let Some(element) = dom.element(node_id) else {
        return false;
    };

    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("radio"))
        .unwrap_or(false)
}

pub(super) fn is_submit_control(dom: &Dom, node_id: NodeId) -> bool {
    let Some(element) = dom.element(node_id) else {
        return false;
    };

    if element.tag_name.eq_ignore_ascii_case("button") {
        return normalized_button_type(element.attrs.get("type").map(String::as_str)) == "submit";
    }

    if element.tag_name.eq_ignore_ascii_case("input") {
        return element
            .attrs
            .get("type")
            .map(|kind| kind.eq_ignore_ascii_case("submit") || kind.eq_ignore_ascii_case("image"))
            .unwrap_or(false);
    }

    false
}

pub(super) fn is_reset_control(dom: &Dom, node_id: NodeId) -> bool {
    let Some(element) = dom.element(node_id) else {
        return false;
    };

    if element.tag_name.eq_ignore_ascii_case("button") {
        return normalized_button_type(element.attrs.get("type").map(String::as_str)) == "reset";
    }

    if element.tag_name.eq_ignore_ascii_case("input") {
        return element
            .attrs
            .get("type")
            .map(|kind| kind.eq_ignore_ascii_case("reset"))
            .unwrap_or(false);
    }

    false
}
