use super::*;
pub(crate) fn parse_insert_adjacent_position(src: &str) -> Result<InsertAdjacentPosition> {
    let lowered = src.to_ascii_lowercase();
    match lowered.as_str() {
        "beforebegin" => Ok(InsertAdjacentPosition::BeforeBegin),
        "afterbegin" => Ok(InsertAdjacentPosition::AfterBegin),
        "beforeend" => Ok(InsertAdjacentPosition::BeforeEnd),
        "afterend" => Ok(InsertAdjacentPosition::AfterEnd),
        _ => Err(Error::ScriptParse(format!(
            "unsupported insertAdjacent position: {src}"
        ))),
    }
}

pub(crate) fn resolve_insert_adjacent_position(src: &str) -> Result<InsertAdjacentPosition> {
    let lowered = src.to_ascii_lowercase();
    match lowered.as_str() {
        "beforebegin" => Ok(InsertAdjacentPosition::BeforeBegin),
        "afterbegin" => Ok(InsertAdjacentPosition::AfterBegin),
        "beforeend" => Ok(InsertAdjacentPosition::BeforeEnd),
        "afterend" => Ok(InsertAdjacentPosition::AfterEnd),
        _ => Err(Error::ScriptRuntime(format!(
            "unsupported insertAdjacentHTML position: {src}"
        ))),
    }
}

pub(crate) fn parse_insert_adjacent_element_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("insertAdjacentElement") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "insertAdjacentElement requires 2 arguments: {stmt}"
        )));
    }

    let position = parse_insert_adjacent_position(&parse_string_literal_exact(args[0].trim())?)?;
    let node = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported insertAdjacentElement statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::InsertAdjacentElement {
        target,
        position,
        node,
    }))
}

pub(crate) fn parse_insert_adjacent_text_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("insertAdjacentText") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "insertAdjacentText requires 2 arguments: {stmt}"
        )));
    }

    let position = parse_insert_adjacent_position(&parse_string_literal_exact(args[0].trim())?)?;
    let text = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported insertAdjacentText statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::InsertAdjacentText {
        target,
        position,
        text,
    }))
}

pub(crate) fn parse_insert_adjacent_html_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("insertAdjacentHTML") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "insertAdjacentHTML requires 2 arguments: {stmt}"
        )));
    }

    let position = parse_expr(args[0].trim())?;
    let html = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported insertAdjacentHTML statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::InsertAdjacentHTML {
        target,
        position,
        html,
    }))
}
