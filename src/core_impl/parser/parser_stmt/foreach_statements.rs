use super::*;
pub(crate) fn parse_query_selector_all_foreach_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    let source = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let method = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse(format!("invalid forEach statement: {stmt}")))?;

    let (target, selector) = match method.as_str() {
        "forEach" => {
            let (target, selector) = match &source {
                DomQuery::BySelectorAll { selector } => (None, selector.clone()),
                DomQuery::QuerySelectorAll { target, selector } => {
                    (Some(target.as_ref().clone()), selector.clone())
                }
                _ => {
                    return Ok(None);
                }
            };
            cursor.skip_ws();
            (target, selector)
        }
        "querySelectorAll" => {
            cursor.skip_ws();
            cursor.expect_byte(b'(')?;
            cursor.skip_ws();
            let selector = cursor.parse_string_literal()?;
            cursor.skip_ws();
            cursor.expect_byte(b')')?;
            cursor.skip_ws();
            if !cursor.consume_byte(b'.') {
                return Ok(None);
            }
            cursor.skip_ws();
            if !cursor.consume_ascii("forEach") {
                return Ok(None);
            }
            (
                match source {
                    DomQuery::DocumentRoot => None,
                    _ => Some(source.clone()),
                },
                selector,
            )
        }
        _ => return Ok(None),
    };
    cursor.skip_ws();

    // For consistency with current test grammar, allow optional event callback without a semicolon.
    cursor.skip_ws();

    let callback_src = cursor.read_balanced_block(b'(', b')')?;
    let (item_var, index_var, body) = parse_for_each_callback(&callback_src)?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported forEach statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::ForEach {
        target,
        selector,
        item_var,
        index_var,
        body,
    }))
}

pub(crate) fn parse_array_for_each_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let stmt_no_semi = stmt.strip_suffix(';').map(str::trim_end).unwrap_or(stmt);

    let mut cursor = Cursor::new(stmt_no_semi);
    cursor.skip_ws();
    if let Some(target) = cursor.parse_identifier() {
        cursor.skip_ws();
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            if cursor.consume_ascii("forEach") {
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();

                let args_src = cursor.read_balanced_block(b'(', b')')?;
                let args = split_top_level_by_char(&args_src, b',');
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "forEach requires a callback and optional thisArg".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse("forEach thisArg cannot be empty".into()));
                }
                let callback = match parse_array_for_each_callback_arg(
                    args[0],
                    3,
                    "array callback parameters",
                ) {
                    Ok(callback) => callback,
                    Err(_) => return Ok(Some(Stmt::Expr(parse_expr(stmt_no_semi)?))),
                };
                if args.len() == 2 {
                    let _ = parse_expr(args[1].trim())?;
                }

                cursor.skip_ws();
                if !cursor.eof() {
                    return Err(Error::ScriptParse(format!(
                        "unsupported forEach statement tail: {stmt}"
                    )));
                }
                return Ok(Some(Stmt::ArrayForEach { target, callback }));
            }
        }
    }

    if !stmt_no_semi.contains(".forEach(") {
        return Ok(None);
    }
    if stmt_no_semi.contains(".classList.forEach(") {
        return Ok(None);
    }
    let Some(for_each_dot_pos) = find_top_level_for_each_call(stmt_no_semi) else {
        return Ok(None);
    };

    let target_src = stmt_no_semi[..for_each_dot_pos].trim();
    if target_src.is_empty() {
        return Ok(None);
    }

    let call_src = stmt_no_semi
        .get(for_each_dot_pos + 1..)
        .ok_or_else(|| Error::ScriptParse(format!("invalid forEach statement: {stmt}")))?;
    let mut cursor = Cursor::new(call_src);
    if !cursor.consume_ascii("forEach") {
        return Ok(None);
    }
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() || args.len() > 2 {
        return Err(Error::ScriptParse(
            "forEach requires a callback and optional thisArg".into(),
        ));
    }
    if args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "forEach requires a callback and optional thisArg".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse("forEach thisArg cannot be empty".into()));
    }

    let target = parse_expr(target_src)?;
    let callback = match parse_array_for_each_callback_arg(args[0], 3, "array callback parameters")
    {
        Ok(callback) => callback,
        Err(_) => return Ok(Some(Stmt::Expr(parse_expr(stmt_no_semi)?))),
    };
    if args.len() == 2 {
        let _ = parse_expr(args[1].trim())?;
    }

    Ok(Some(Stmt::ArrayForEachExpr { target, callback }))
}

pub(crate) fn parse_array_for_each_callback_arg(
    arg: &str,
    max_params: usize,
    label: &str,
) -> Result<ScriptHandler> {
    let callback_arg = strip_js_comments(arg);
    let mut callback_cursor = Cursor::new(callback_arg.as_str().trim());
    let (params, body, concise_body) = parse_callback(&mut callback_cursor, max_params, label)?;
    callback_cursor.skip_ws();
    if !callback_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported array callback: {}",
            arg.trim()
        )));
    }

    let stmts = if concise_body {
        match parse_expr(body.trim()) {
            Ok(expr) => vec![Stmt::Return { value: Some(expr) }],
            Err(_) => parse_block_statements(&format!("{};", body.trim()))?,
        }
    } else {
        parse_block_statements(&body)?
    };

    Ok(ScriptHandler { params, stmts })
}

pub(crate) fn find_top_level_for_each_call(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() && bytes[i] == b'.' {
            if src
                .get(i + 1..)
                .is_some_and(|tail| tail.starts_with("forEach("))
            {
                return Some(i);
            }
        }
        i = scanner.advance(bytes, i);
    }

    None
}

pub(crate) fn parse_for_each_callback(src: &str) -> Result<(String, Option<String>, Vec<Stmt>)> {
    let mut cursor = Cursor::new(src.trim());
    cursor.skip_ws();
    let mut param_prologue: Vec<String> = Vec::new();

    let (item_var, index_var) = if cursor
        .src
        .get(cursor.i..)
        .is_some_and(|src| src.starts_with("function"))
        && !cursor
            .bytes()
            .get(cursor.i + "function".len())
            .is_some_and(|&b| is_ident_char(b))
    {
        cursor.consume_ascii("function");
        cursor.skip_ws();
        if !cursor.consume_byte(b'(') {
            let _ = cursor
                .parse_identifier()
                .ok_or_else(|| Error::ScriptParse("expected function name".into()))?;
            cursor.skip_ws();
            cursor.expect_byte(b'(')?;
        }
        let params_src = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        let parsed_params = parse_callback_parameter_list(
            &params_src,
            2,
            "forEach callback must have one or two parameters",
        )?;
        if parsed_params
            .params
            .iter()
            .any(|param| param.default.is_some() || param.is_rest)
        {
            return Err(Error::ScriptParse(format!(
                "forEach callback must not use default or rest parameters: {src}"
            )));
        }
        let item_var = parsed_params
            .params
            .first()
            .map(|param| param.name.clone())
            .ok_or_else(|| {
                Error::ScriptParse(format!(
                    "forEach callback must have one or two parameters: {src}"
                ))
            })?;
        let index_var = parsed_params.params.get(1).map(|param| param.name.clone());

        cursor.skip_ws();
        let (body, concise_body) = parse_arrow_or_block_body(&mut cursor)?;
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported forEach callback tail: {src}"
            )));
        }

        let body_stmts = parse_for_each_callback_body_stmts(&body, concise_body)?;
        let body_stmts =
            prepend_callback_param_prologue_stmts(body_stmts, &parsed_params.prologue)?;
        return Ok((item_var, index_var, body_stmts));
    } else if cursor.consume_byte(b'(') {
        let params_src = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        let parsed_params = parse_callback_parameter_list(
            &params_src,
            2,
            "forEach callback must have one or two parameters",
        )?;
        if parsed_params
            .params
            .iter()
            .any(|param| param.default.is_some() || param.is_rest)
        {
            return Err(Error::ScriptParse(format!(
                "forEach callback must not use default or rest parameters: {src}"
            )));
        }
        param_prologue = parsed_params.prologue;
        let item_var = parsed_params
            .params
            .first()
            .map(|param| param.name.clone())
            .ok_or_else(|| {
                Error::ScriptParse(format!(
                    "forEach callback must have one or two parameters: {src}"
                ))
            })?;
        let index_var = parsed_params.params.get(1).map(|param| param.name.clone());
        (item_var, index_var)
    } else {
        let Some(item) = cursor.parse_identifier() else {
            return Err(Error::ScriptParse(format!(
                "invalid forEach callback parameters: {src}"
            )));
        };
        (item, None)
    };

    skip_arrow_whitespace_without_line_terminator(&mut cursor)?;
    cursor.expect_ascii("=>")?;
    cursor.skip_ws();
    let (body, concise_body) = parse_arrow_or_block_body(&mut cursor)?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported forEach callback tail: {src}"
        )));
    }

    let body_stmts = parse_for_each_callback_body_stmts(&body, concise_body)?;
    let body_stmts = prepend_callback_param_prologue_stmts(body_stmts, &param_prologue)?;
    Ok((item_var, index_var, body_stmts))
}

pub(crate) fn parse_for_each_callback_body_stmts(
    body: &str,
    concise_body: bool,
) -> Result<Vec<Stmt>> {
    if !concise_body {
        return parse_block_statements(body);
    }

    match parse_expr(body.trim()) {
        Ok(expr) => Ok(vec![Stmt::Expr(expr)]),
        Err(_) => parse_block_statements(&format!("{};", body.trim())),
    }
}
