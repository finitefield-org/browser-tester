use super::*;

pub(crate) fn parse_event_property_expr(src: &str) -> Result<Option<(String, EventExprProp)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let Some(event_var) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(head) = cursor.parse_identifier() else {
        return Ok(None);
    };

    let mut nested = None;
    cursor.skip_ws();
    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        nested = cursor.parse_identifier();
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    if event_var == "history"
        && matches!(
            (head.as_str(), nested.as_deref()),
            ("state", None) | ("oldState", None) | ("newState", None)
        )
    {
        return Ok(None);
    }

    let prop = match (head.as_str(), nested.as_deref()) {
        ("type", None) => EventExprProp::Type,
        ("target", None) => EventExprProp::Target,
        ("currentTarget", None) => EventExprProp::CurrentTarget,
        ("target", Some("name")) => EventExprProp::TargetName,
        ("currentTarget", Some("name")) => EventExprProp::CurrentTargetName,
        ("defaultPrevented", None) => EventExprProp::DefaultPrevented,
        ("isTrusted", None) => EventExprProp::IsTrusted,
        ("bubbles", None) => EventExprProp::Bubbles,
        ("cancelable", None) => EventExprProp::Cancelable,
        ("target", Some("id")) => EventExprProp::TargetId,
        ("currentTarget", Some("id")) => EventExprProp::CurrentTargetId,
        ("eventPhase", None) => EventExprProp::EventPhase,
        ("timeStamp", None) => EventExprProp::TimeStamp,
        ("state", None) => EventExprProp::State,
        ("oldState", None) => EventExprProp::OldState,
        ("newState", None) => EventExprProp::NewState,
        _ => return Ok(None),
    };

    Ok(Some((event_var, prop)))
}

pub(crate) fn parse_class_list_contains_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("classList") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    if !cursor.consume_ascii("contains") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let class_name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, class_name)))
}

pub(crate) fn parse_query_selector_all_length_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    let is_list_target = matches!(
        target,
        DomQuery::BySelectorAll { .. } | DomQuery::QuerySelectorAll { .. } | DomQuery::Var(_)
    );
    if !is_list_target {
        return Ok(None);
    }

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("length") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(target))
}

pub(crate) fn parse_form_elements_length_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let form = match parse_form_elements_base(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("elements") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("length") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(form))
}
