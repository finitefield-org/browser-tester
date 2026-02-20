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

pub(crate) fn parse_new_form_data_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(form) = parse_new_form_data_target(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(form))
}

pub(crate) fn parse_form_data_get_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    parse_form_data_method_expr(src, "get")
}

pub(crate) fn parse_form_data_has_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    parse_form_data_method_expr(src, "has")
}

pub(crate) fn parse_form_data_get_all_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    parse_form_data_method_expr(src, "getAll")
}

pub(crate) fn parse_form_data_get_all_length_expr(
    src: &str,
) -> Result<Option<(FormDataSource, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let Some(source) = parse_form_data_source(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if method != "getAll" {
        return Ok(None);
    }

    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
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

    Ok(Some((source, name)))
}

pub(crate) fn parse_form_data_method_expr(
    src: &str,
    method: &str,
) -> Result<Option<(FormDataSource, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let Some(source) = parse_form_data_source(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(actual_method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if actual_method != method {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((source, name)))
}

pub(crate) fn parse_form_data_source(cursor: &mut Cursor<'_>) -> Result<Option<FormDataSource>> {
    if let Some(form) = parse_new_form_data_target(cursor)? {
        return Ok(Some(FormDataSource::NewForm(form)));
    }

    if let Some(var_name) = cursor.parse_identifier() {
        return Ok(Some(FormDataSource::Var(var_name)));
    }

    Ok(None)
}

pub(crate) fn parse_new_form_data_target(cursor: &mut Cursor<'_>) -> Result<Option<DomQuery>> {
    cursor.skip_ws();
    let start = cursor.pos();

    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            cursor.set_pos(start);
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("FormData") {
        cursor.set_pos(start);
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(args_src.trim(), b',');
    if args.len() != 1 {
        return Err(Error::ScriptParse(
            "new FormData requires exactly one argument".into(),
        ));
    }

    let arg = args[0].trim();
    let mut arg_cursor = Cursor::new(arg);
    arg_cursor.skip_ws();
    let form = parse_form_elements_base(&mut arg_cursor)?;
    arg_cursor.skip_ws();
    if !arg_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported FormData argument: {arg}"
        )));
    }

    Ok(Some(form))
}

pub(crate) fn parse_string_literal_exact(src: &str) -> Result<String> {
    let bytes = src.as_bytes();
    if bytes.len() < 2 {
        return Err(Error::ScriptParse("invalid string literal".into()));
    }
    let quote = bytes[0];
    if (quote != b'\'' && quote != b'"') || bytes[bytes.len() - 1] != quote {
        return Err(Error::ScriptParse(format!("invalid string literal: {src}")));
    }

    let mut escaped = false;
    let mut i = 1;
    while i + 1 < bytes.len() {
        let b = bytes[i];
        if escaped {
            escaped = false;
        } else if b == b'\\' {
            escaped = true;
        } else if b == quote {
            return Err(Error::ScriptParse(format!("unexpected quote in: {src}")));
        }
        i += 1;
    }

    Ok(unescape_string(&src[1..src.len() - 1]))
}

pub(crate) fn strip_outer_parens(mut src: &str) -> &str {
    loop {
        let trimmed = src.trim();
        if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
            return trimmed;
        }

        if !is_fully_wrapped_in_parens(trimmed) {
            return trimmed;
        }

        src = &trimmed[1..trimmed.len() - 1];
    }
}

pub(crate) fn is_fully_wrapped_in_parens(src: &str) -> bool {
    let bytes = src.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'(' || bytes[bytes.len() - 1] != b')' {
        return false;
    }

    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.in_normal() && bytes[i] == b')' && scanner.paren == 1 {
            let mut tail = i + 1;
            while tail < bytes.len() && bytes[tail].is_ascii_whitespace() {
                tail += 1;
            }
            if tail < bytes.len() {
                return false;
            }
        }
        i = scanner.advance(bytes, i);
    }

    scanner.in_normal() && scanner.paren == 0 && scanner.bracket == 0 && scanner.brace == 0
}

pub(crate) fn find_top_level_assignment(src: &str) -> Option<(usize, usize)> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() && bytes[i] == b'=' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                i = scanner.advance(bytes, i);
                continue;
            }
            if i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                i = scanner.advance(bytes, i);
                continue;
            }
            if i >= 3 && &bytes[i - 3..=i] == b">>>=" {
                return Some((i - 3, 4));
            }
            if i >= 2
                && matches!(
                    &bytes[i - 2..=i],
                    b"&&=" | b"||=" | b"??=" | b"**=" | b"<<=" | b">>="
                )
            {
                return Some((i - 2, 3));
            }
            if i > 0 {
                let prev = bytes[i - 1];
                if matches!(prev, b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^') {
                    return Some((i - 1, 2));
                }
                if matches!(prev, b'!' | b'<' | b'>' | b'=') {
                    i = scanner.advance(bytes, i);
                    continue;
                }
            }
            return Some((i, 1));
        }
        i = scanner.advance(bytes, i);
    }

    None
}

pub(crate) fn find_top_level_ternary_question(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() && bytes[i] == b'?' {
            if i + 1 < bytes.len() && (bytes[i + 1] == b'?' || bytes[i + 1] == b'.') {
                i = scanner.advance(bytes, i);
                continue;
            }
            if i > 0 && bytes[i - 1] == b'?' {
                i = scanner.advance(bytes, i);
                continue;
            }
            return Some(i);
        }
        i = scanner.advance(bytes, i);
    }

    None
}

pub(crate) fn find_matching_ternary_colon(src: &str, from: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if from >= bytes.len() {
        return None;
    }

    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();
    let mut nested_ternary = 0usize;

    while i < bytes.len() {
        if i >= from && scanner.is_top_level() {
            let b = bytes[i];
            if b == b'?' {
                if i + 1 < bytes.len() && (bytes[i + 1] == b'?' || bytes[i + 1] == b'.') {
                    i = scanner.advance(bytes, i);
                    continue;
                }
                if i > 0 && bytes[i - 1] == b'?' {
                    i = scanner.advance(bytes, i);
                    continue;
                }
                nested_ternary += 1;
            } else if b == b':' {
                if nested_ternary == 0 {
                    return Some(i);
                }
                nested_ternary -= 1;
            }
        }
        i = scanner.advance(bytes, i);
    }

    None
}
