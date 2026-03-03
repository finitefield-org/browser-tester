use super::*;
pub(crate) fn parse_set_attribute_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("setAttribute") {
        return Ok(None);
    }
    if cursor.peek().is_some_and(is_ident_char) {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "setAttribute requires 2 arguments: {stmt}"
        )));
    }
    let name = match parse_string_literal_exact(args[0].trim()) {
        Ok(name) => name,
        Err(_) => return Ok(None),
    };
    let value = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported setAttribute statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::DomSetAttribute {
        target,
        name,
        value,
    }))
}

pub(crate) fn parse_remove_attribute_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("removeAttribute") {
        return Ok(None);
    }
    if cursor.peek().is_some_and(is_ident_char) {
        return Ok(None);
    }
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 {
        return Err(Error::ScriptParse(format!(
            "removeAttribute requires exactly one argument: {stmt}"
        )));
    }
    let name = match parse_string_literal_exact(args[0].trim()) {
        Ok(name) => name,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported removeAttribute statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::DomRemoveAttribute { target, name }))
}

pub(crate) fn parse_class_list_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    let mut optional = if cursor.consume_ascii("?.") {
        true
    } else if cursor.consume_byte(b'.') {
        false
    } else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_ascii("classList") {
        return Ok(None);
    }
    cursor.skip_ws();
    if cursor.consume_ascii("?.") {
        optional = true;
    } else {
        cursor.expect_byte(b'.')?;
    }
    cursor.skip_ws();

    let method = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse(format!("expected classList method in: {stmt}")))?;

    if method == "forEach" {
        cursor.skip_ws();
        let callback_src = cursor.read_balanced_block(b'(', b')')?;
        let (item_var, index_var, body) =
            super::foreach_statements::parse_for_each_callback(&callback_src)?;

        cursor.skip_ws();
        cursor.consume_byte(b';');
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported classList statement tail: {stmt}"
            )));
        }

        return Ok(Some(Stmt::ClassListForEach {
            target,
            optional,
            item_var,
            index_var,
            body,
        }));
    }

    let method = match method.as_str() {
        "add" => ClassListMethod::Add,
        "remove" => ClassListMethod::Remove,
        "toggle" => ClassListMethod::Toggle,
        "replace" => ClassListMethod::Replace,
        _ => return Ok(None),
    };

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() {
        return Err(Error::ScriptParse(format!(
            "invalid classList arguments: {stmt}"
        )));
    }

    let force = match method {
        ClassListMethod::Toggle => {
            if args.len() > 2 {
                return Err(Error::ScriptParse(format!(
                    "invalid classList arguments: {stmt}"
                )));
            }

            if args.len() == 2 {
                Some(parse_expr(args[1].trim())?)
            } else {
                None
            }
        }
        _ => None,
    };

    let class_names = match method {
        ClassListMethod::Toggle => vec![parse_expr(args[0].trim())?],
        ClassListMethod::Replace => {
            if args.len() != 2 {
                return Err(Error::ScriptParse(format!(
                    "invalid classList arguments: {stmt}"
                )));
            }
            vec![parse_expr(args[0].trim())?, parse_expr(args[1].trim())?]
        }
        _ => args
            .iter()
            .map(|arg| parse_expr(arg.trim()))
            .collect::<Result<Vec<_>>>()?,
    };

    if matches!(method, ClassListMethod::Add | ClassListMethod::Remove) && class_names.is_empty() {
        return Err(Error::ScriptParse(format!(
            "classList add/remove requires at least one argument: {stmt}"
        )));
    }

    if !matches!(method, ClassListMethod::Toggle) && force.is_some() {
        return Err(Error::ScriptParse(
            "classList add/remove do not accept a force argument".into(),
        ));
    }

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();

    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported classList statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::ClassListCall {
        target,
        optional,
        method,
        class_names,
        force,
    }))
}
