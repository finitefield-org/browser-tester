use super::*;

pub(super) fn close_optional_description_item_start_tag(
    dom: &Dom,
    stack: &mut Vec<NodeId>,
    tag: &str,
) {
    if !(tag.eq_ignore_ascii_case("dt") || tag.eq_ignore_ascii_case("dd")) {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("dt") || open_tag.eq_ignore_ascii_case("dd") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("dl") {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

pub(super) fn close_optional_list_item_start_tag(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
    if !tag.eq_ignore_ascii_case("li") {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("li") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("ol")
            || open_tag.eq_ignore_ascii_case("ul")
            || open_tag.eq_ignore_ascii_case("menu")
        {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

pub(super) fn close_optional_option_start_tag(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
    if !(tag.eq_ignore_ascii_case("option") || tag.eq_ignore_ascii_case("optgroup")) {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("option") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("optgroup")
            || open_tag.eq_ignore_ascii_case("select")
            || open_tag.eq_ignore_ascii_case("datalist")
        {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

pub(super) fn close_optional_optgroup_start_tag(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
    if !tag.eq_ignore_ascii_case("optgroup") {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("optgroup") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("select") {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

pub(super) fn close_optional_ruby_text_start_tag(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
    if !(tag.eq_ignore_ascii_case("rt") || tag.eq_ignore_ascii_case("rp")) {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("rt") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("ruby") {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

pub(super) fn close_optional_ruby_fallback_parenthesis_start_tag(
    dom: &Dom,
    stack: &mut Vec<NodeId>,
    tag: &str,
) {
    if !(tag.eq_ignore_ascii_case("rt") || tag.eq_ignore_ascii_case("rp")) {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("rp") {
            close_index = Some(index);
            break;
        }
        if open_tag.eq_ignore_ascii_case("ruby") {
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

pub(super) fn close_optional_paragraph_start_tag(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
    if !is_optional_paragraph_terminator_tag(tag) {
        return;
    }

    let mut close_index = None;
    for index in (1..stack.len()).rev() {
        let Some(open_tag) = dom.tag_name(stack[index]) else {
            continue;
        };
        if open_tag.eq_ignore_ascii_case("p") {
            close_index = Some(index);
            break;
        }
    }

    if let Some(index) = close_index {
        stack.truncate(index);
    }
}

fn is_optional_paragraph_terminator_tag(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "details"
            | "div"
            | "dl"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hgroup"
            | "hr"
            | "main"
            | "menu"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "search"
            | "section"
            | "table"
            | "ul"
    )
}
