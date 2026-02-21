use super::*;

pub(super) fn is_form_control(dom: &Dom, node_id: NodeId) -> bool {
    let Some(element) = dom.element(node_id) else {
        return false;
    };

    element.tag_name.eq_ignore_ascii_case("input")
        || element.tag_name.eq_ignore_ascii_case("select")
        || element.tag_name.eq_ignore_ascii_case("textarea")
        || element.tag_name.eq_ignore_ascii_case("button")
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
        return element
            .attrs
            .get("type")
            .map(|kind| kind.eq_ignore_ascii_case("submit"))
            .unwrap_or(true);
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
        return element
            .attrs
            .get("type")
            .map(|kind| kind.eq_ignore_ascii_case("reset"))
            .unwrap_or(false);
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
