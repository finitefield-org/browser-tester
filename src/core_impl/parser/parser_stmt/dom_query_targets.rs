use super::*;

pub(crate) fn append_dom_query_member_path(target: &DomQuery, member: &str) -> Option<DomQuery> {
    match target {
        DomQuery::Var(base) => Some(DomQuery::VarPath {
            base: base.clone(),
            path: vec![member.to_string()],
        }),
        DomQuery::VarPath { base, path } => {
            let mut next_path = path.clone();
            next_path.push(member.to_string());
            Some(DomQuery::VarPath {
                base: base.clone(),
                path: next_path,
            })
        }
        _ => None,
    }
}

pub(crate) fn is_dom_target_chain_stop(ident: &str) -> bool {
    if ident.starts_with("aria") {
        return true;
    }
    matches!(
        ident,
        "activeElement"
            | "addEventListener"
            | "after"
            | "append"
            | "appendChild"
            | "assignedSlot"
            | "attributes"
            | "attributionSrc"
            | "before"
            | "blur"
            | "charset"
            | "checked"
            | "checkVisibility"
            | "checkValidity"
            | "childElementCount"
            | "classList"
            | "className"
            | "click"
            | "clientHeight"
            | "clientLeft"
            | "clientTop"
            | "clientWidth"
            | "close"
            | "closest"
            | "closedBy"
            | "closedby"
            | "coords"
            | "currentCSSZoom"
            | "dataset"
            | "download"
            | "disabled"
            | "dispatchEvent"
            | "elementTiming"
            | "elements"
            | "focus"
            | "forEach"
            | "getAttribute"
            | "getAttributeNames"
            | "getElementsByClassName"
            | "getElementsByTagName"
            | "hash"
            | "hasAttribute"
            | "hasAttributes"
            | "host"
            | "hostname"
            | "href"
            | "hreflang"
            | "hidden"
            | "id"
            | "indeterminate"
            | "interestForElement"
            | "innerHTML"
            | "innerText"
            | "insertAdjacentElement"
            | "insertAdjacentHTML"
            | "insertAdjacentText"
            | "insertBefore"
            | "lastElementChild"
            | "length"
            | "localName"
            | "matches"
            | "name"
            | "namespaceURI"
            | "nextElementSibling"
            | "offsetHeight"
            | "offsetLeft"
            | "offsetTop"
            | "offsetWidth"
            | "open"
            | "origin"
            | "outerHTML"
            | "password"
            | "part"
            | "pathname"
            | "ping"
            | "port"
            | "prefix"
            | "previousElementSibling"
            | "protocol"
            | "querySelector"
            | "querySelectorAll"
            | "prepend"
            | "readOnly"
            | "readonly"
            | "referrerPolicy"
            | "rel"
            | "relList"
            | "remove"
            | "removeAttribute"
            | "removeChild"
            | "removeEventListener"
            | "requestClose"
            | "requestSubmit"
            | "role"
            | "returnValue"
            | "replaceWith"
            | "reportValidity"
            | "required"
            | "reset"
            | "rev"
            | "scroll"
            | "scrollBy"
            | "search"
            | "selectionDirection"
            | "selectionEnd"
            | "selectionStart"
            | "setCustomValidity"
            | "setRangeText"
            | "setSelectionRange"
            | "shape"
            | "scrollHeight"
            | "scrollIntoView"
            | "scrollLeft"
            | "scrollLeftMax"
            | "scrollTo"
            | "scrollTop"
            | "scrollTopMax"
            | "scrollWidth"
            | "setAttribute"
            | "shadowRoot"
            | "show"
            | "showPicker"
            | "showModal"
            | "slot"
            | "stepDown"
            | "stepUp"
            | "style"
            | "submit"
            | "target"
            | "tagName"
            | "text"
            | "textContent"
            | "toggleAttribute"
            | "type"
            | "username"
            | "value"
            | "validationMessage"
            | "validity"
            | "firstElementChild"
            | "children"
    )
}

pub(crate) fn parse_element_target(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    cursor.skip_ws();
    let start = cursor.pos();
    let mut target = if let Ok(target) = parse_form_elements_item_target(cursor) {
        target
    } else {
        cursor.set_pos(start);
        parse_document_or_var_target(cursor)?
    };

    loop {
        cursor.skip_ws();
        let dot_pos = cursor.pos();
        if !cursor.consume_byte(b'.') {
            break;
        }

        cursor.skip_ws();
        let method = match cursor.parse_identifier() {
            Some(method) => method,
            None => {
                cursor.set_pos(dot_pos);
                break;
            }
        };

        match method.as_str() {
            "body" if matches!(target, DomQuery::DocumentRoot) => {
                target = DomQuery::DocumentBody;
            }
            "head" if matches!(target, DomQuery::DocumentRoot) => {
                target = DomQuery::DocumentHead;
            }
            "documentElement" if matches!(target, DomQuery::DocumentRoot) => {
                target = DomQuery::DocumentElement;
            }
            "querySelector" => {
                cursor.skip_ws();
                if cursor.peek() != Some(b'(') {
                    cursor.set_pos(dot_pos);
                    break;
                }
                cursor.expect_byte(b'(')?;
                cursor.skip_ws();
                let selector = cursor.parse_string_literal()?;
                cursor.skip_ws();
                cursor.expect_byte(b')')?;
                cursor.skip_ws();
                target = DomQuery::QuerySelector {
                    target: Box::new(target),
                    selector,
                };
            }
            "querySelectorAll" => {
                cursor.skip_ws();
                if cursor.peek() != Some(b'(') {
                    cursor.set_pos(dot_pos);
                    break;
                }
                cursor.expect_byte(b'(')?;
                cursor.skip_ws();
                let selector = cursor.parse_string_literal()?;
                cursor.skip_ws();
                cursor.expect_byte(b')')?;
                cursor.skip_ws();
                target = DomQuery::QuerySelectorAll {
                    target: Box::new(target),
                    selector,
                };
            }
            _ => {
                if is_dom_target_chain_stop(&method) {
                    cursor.set_pos(dot_pos);
                    break;
                }
                if let Some(next_target) = append_dom_query_member_path(&target, &method) {
                    target = next_target;
                    continue;
                }
                cursor.set_pos(dot_pos);
                break;
            }
        }
    }

    loop {
        cursor.skip_ws();
        let index_pos = cursor.pos();
        if !cursor.consume_byte(b'[') {
            break;
        }

        cursor.skip_ws();
        let index_src = match cursor.read_until_byte(b']') {
            Ok(index_src) => index_src,
            Err(_) => {
                cursor.set_pos(index_pos);
                break;
            }
        };
        cursor.skip_ws();
        cursor.expect_byte(b']')?;
        let index = parse_dom_query_index(&index_src)?;
        target = match target {
            DomQuery::BySelectorAll { selector } => {
                DomQuery::BySelectorAllIndex { selector, index }
            }
            DomQuery::QuerySelectorAll { target, selector } => DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            },
            _ => DomQuery::Index {
                target: Box::new(target),
                index,
            },
        };
        cursor.skip_ws();
    }
    Ok(target)
}

pub(crate) fn parse_document_or_var_target(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    let start = cursor.pos();
    if let Ok(target) = parse_document_element_call(cursor) {
        return Ok(target);
    }
    cursor.set_pos(start);
    if cursor.consume_ascii("document") {
        return Ok(DomQuery::DocumentRoot);
    }
    cursor.set_pos(start);
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            if cursor.consume_ascii("document") {
                cursor.skip_ws();
            } else {
                cursor.set_pos(start + "window".len());
            }
        }
        return Ok(DomQuery::DocumentRoot);
    }
    cursor.set_pos(start);
    if let Some(name) = cursor.parse_identifier() {
        return Ok(DomQuery::Var(name));
    }
    Err(Error::ScriptParse(format!(
        "expected element target at {}",
        start
    )))
}

pub(crate) fn parse_form_elements_item_target(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    let form = parse_form_elements_base(cursor)?;
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    cursor.expect_ascii("elements")?;
    cursor.skip_ws();
    cursor.expect_byte(b'[')?;
    cursor.skip_ws();
    let index_src = cursor.read_until_byte(b']')?;
    cursor.skip_ws();
    cursor.expect_byte(b']')?;
    let index = parse_dom_query_index(&index_src)?;
    Ok(DomQuery::FormElementsIndex {
        form: Box::new(form),
        index,
    })
}

pub(crate) fn parse_dom_query_index(src: &str) -> Result<DomIndex> {
    let src = strip_js_comments(src).trim().to_string();
    if src.is_empty() {
        return Err(Error::ScriptParse("empty index".into()));
    }

    let expr = parse_expr(&src)?;
    if let Expr::Number(index) = expr {
        return usize::try_from(index)
            .map(DomIndex::Static)
            .map_err(|_| Error::ScriptParse(format!("invalid index: {src}")));
    }

    Ok(DomIndex::Dynamic(src))
}

pub(crate) fn parse_form_elements_base(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    let start = cursor.pos();
    if let Ok(target) = parse_document_element_call(cursor) {
        return Ok(target);
    }
    cursor.set_pos(start);
    if let Some(name) = cursor.parse_identifier() {
        return Ok(DomQuery::Var(name));
    }
    Err(Error::ScriptParse(format!(
        "expected form target at {}",
        start
    )))
}

pub(crate) fn parse_document_element_call(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        cursor.expect_byte(b'.')?;
        cursor.skip_ws();
    }
    cursor.expect_ascii("document")?;
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    let method = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse("expected document method call".into()))?;
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let arg = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();

    match method.as_str() {
        "getElementById" => Ok(DomQuery::ById(arg)),
        "querySelector" => Ok(DomQuery::BySelector(arg)),
        "querySelectorAll" => Ok(DomQuery::BySelectorAll { selector: arg }),
        "getElementsByTagName" => Ok(DomQuery::BySelectorAll {
            selector: normalize_get_elements_by_tag_name(&arg)?,
        }),
        "getElementsByClassName" => Ok(DomQuery::BySelectorAll {
            selector: normalize_get_elements_by_class_name(&arg)?,
        }),
        "getElementsByName" => Ok(DomQuery::BySelectorAll {
            selector: normalize_get_elements_by_name(&arg)?,
        }),
        _ => Err(Error::ScriptParse(format!(
            "unsupported document method: {}",
            method
        ))),
    }
}

pub(crate) fn normalize_get_elements_by_tag_name(tag_name: &str) -> Result<String> {
    let tag_name = tag_name.trim();
    if tag_name.is_empty() {
        return Err(Error::ScriptParse(
            "getElementsByTagName requires a tag name".into(),
        ));
    }
    if tag_name == "*" {
        return Ok("*".into());
    }
    Ok(tag_name.to_ascii_lowercase())
}

pub(crate) fn normalize_get_elements_by_class_name(class_names: &str) -> Result<String> {
    let mut selector = String::new();
    let classes: Vec<&str> = class_names
        .split_whitespace()
        .map(str::trim)
        .filter(|class_name| !class_name.is_empty())
        .collect();

    if classes.is_empty() {
        return Err(Error::ScriptParse(
            "getElementsByClassName requires at least one class name".into(),
        ));
    }

    for class_name in classes {
        selector.push('.');
        selector.push_str(class_name);
    }
    Ok(selector)
}

pub(crate) fn normalize_get_elements_by_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::ScriptParse(
            "getElementsByName requires a name value".into(),
        ));
    }
    let escaped = name.replace('\\', "\\\\").replace('\'', "\\'");
    Ok(format!("[name='{}']", escaped))
}
