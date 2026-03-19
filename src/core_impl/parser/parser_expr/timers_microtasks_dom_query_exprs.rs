use super::*;

pub(crate) fn parse_get_attribute_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
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
    if !cursor.consume_ascii("getAttribute") {
        return Ok(None);
    }
    if cursor.peek().is_some_and(is_ident_char) {
        return Ok(None);
    }
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 {
        return Ok(None);
    }
    let mut arg_cursor = Cursor::new(args[0].trim());
    arg_cursor.skip_ws();
    let name = match arg_cursor.parse_string_literal() {
        Ok(name) => name,
        Err(_) => return Ok(None),
    };
    arg_cursor.skip_ws();
    if !arg_cursor.eof() {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, name)))
}

pub(crate) fn parse_has_attribute_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
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
    if !cursor.consume_ascii("hasAttribute") {
        return Ok(None);
    }
    if cursor.peek().is_some_and(is_ident_char) {
        return Ok(None);
    }
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 {
        return Ok(None);
    }
    let mut arg_cursor = Cursor::new(args[0].trim());
    arg_cursor.skip_ws();
    let name = match arg_cursor.parse_string_literal() {
        Ok(name) => name,
        Err(_) => return Ok(None),
    };
    arg_cursor.skip_ws();
    if !arg_cursor.eof() {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, name)))
}

pub(crate) fn parse_dom_matches_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
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
    if !cursor.consume_ascii("matches") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'(') {
        return Ok(None);
    }
    cursor.skip_ws();
    let selector = match cursor.parse_string_literal() {
        Ok(selector) => selector,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, selector)))
}

pub(crate) fn parse_dom_closest_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
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
    if !cursor.consume_ascii("closest") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'(') {
        return Ok(None);
    }
    cursor.skip_ws();
    let selector = match cursor.parse_string_literal() {
        Ok(selector) => selector,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, selector)))
}

pub(crate) fn parse_dom_computed_style_property_expr(
    src: &str,
) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("getComputedStyle") {
        return Ok(None);
    }
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 {
        return Ok(None);
    }
    let mut arg_cursor = Cursor::new(args[0]);
    arg_cursor.skip_ws();
    let target = match parse_element_target(&mut arg_cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    arg_cursor.skip_ws();
    if !arg_cursor.eof() {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("getPropertyValue") {
        return Ok(None);
    }
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let method_args = cursor.read_balanced_block(b'(', b')')?;
    let method_args = split_top_level_by_char(&method_args, b',');
    if method_args.len() != 1 {
        return Ok(None);
    }
    let mut property_cursor = Cursor::new(method_args[0]);
    property_cursor.skip_ws();
    let property = match property_cursor.parse_string_literal() {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    property_cursor.skip_ws();
    if !property_cursor.eof() {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((target, property)))
}
