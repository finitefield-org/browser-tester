use super::*;
pub(crate) fn parse_arrow_or_block_body(cursor: &mut Cursor<'_>) -> Result<(String, bool)> {
    cursor.skip_ws();
    if cursor.peek() == Some(b'{') {
        return Ok((cursor.read_balanced_block(b'{', b'}')?, false));
    }

    let src = cursor
        .src
        .get(cursor.i..)
        .ok_or_else(|| Error::ScriptParse("expected callback body".into()))?;
    let mut end = src.len();

    while end > 0 {
        let raw = src
            .get(0..end)
            .ok_or_else(|| Error::ScriptParse("invalid callback body".into()))?;
        let stripped = strip_js_comments(raw);
        let stripped = stripped.trim();
        if stripped.is_empty() {
            end -= 1;
            continue;
        }

        let suffix = src.get(end..).unwrap_or_default();
        if !is_valid_callback_body_suffix(suffix) {
            end -= 1;
            continue;
        }

        if let Some(rewritten) = rewrite_assignment_arrow_body(stripped)? {
            cursor.set_pos(cursor.i + end);
            return Ok((rewritten, false));
        }

        if parse_expr(stripped).is_ok() {
            cursor.set_pos(cursor.i + end);
            return Ok((stripped.to_string(), true));
        }

        // Keep concise callback bodies that are valid single statements even
        // when expression parsing is not yet supported.
        if parse_single_statement(stripped).is_ok() {
            cursor.set_pos(cursor.i + end);
            return Ok((stripped.to_string(), true));
        }

        end -= 1;
    }

    Err(Error::ScriptParse("expected callback body".into()))
}

pub(crate) fn rewrite_assignment_arrow_body(expr_src: &str) -> Result<Option<String>> {
    let expr_src = strip_outer_parens(expr_src).trim();
    let Some((eq_pos, op_len)) = find_top_level_assignment(expr_src) else {
        return Ok(None);
    };

    let lhs_raw = expr_src[..eq_pos].trim();
    let rhs_src = expr_src[eq_pos + op_len..].trim();
    if lhs_raw.is_empty() || rhs_src.is_empty() {
        return Ok(None);
    }

    let lhs = strip_outer_parens(lhs_raw).trim();
    if lhs.is_empty() {
        return Ok(None);
    }

    let op = expr_src
        .get(eq_pos..eq_pos + op_len)
        .ok_or_else(|| Error::ScriptParse("invalid assignment operator".into()))?;
    let assignment_src = format!("{lhs} {op} {rhs_src}");

    let supports_assignment = |result: Result<Option<Stmt>>| match result {
        Ok(Some(_)) => true,
        Ok(None) | Err(_) => false,
    };

    let supported = supports_assignment(parse_var_assign(&assignment_src))
        || supports_assignment(parse_object_assign(&assignment_src))
        || (op_len == 1 && supports_assignment(parse_dom_assignment(&assignment_src)));
    if !supported {
        return Ok(None);
    }

    if parse_expr(lhs).is_err() {
        return Ok(None);
    }

    Ok(Some(format!("{assignment_src}; return {lhs};")))
}

pub(crate) fn is_valid_callback_body_suffix(suffix: &str) -> bool {
    suffix
        .chars()
        .all(|ch| ch.is_ascii_whitespace() || matches!(ch, ')' | ']' | '}' | ',' | ';'))
}

pub(crate) fn skip_arrow_whitespace_without_line_terminator(cursor: &mut Cursor<'_>) -> Result<()> {
    while let Some(ch) = cursor.peek() {
        if ch == b' ' || ch == b'\t' || ch == 0x0B || ch == 0x0C {
            cursor.set_pos(cursor.pos() + 1);
            continue;
        }
        if ch == b'\n' || ch == b'\r' {
            return Err(Error::ScriptParse(
                "line break before => is not allowed".into(),
            ));
        }
        break;
    }
    Ok(())
}

pub(crate) fn try_consume_async_function_prefix(cursor: &mut Cursor<'_>) -> bool {
    let start = cursor.pos();
    if !cursor.consume_ascii("async") {
        return false;
    }
    if cursor.peek().is_some_and(is_ident_char) {
        cursor.set_pos(start);
        return false;
    }

    let mut saw_separator = false;
    let mut saw_line_terminator = false;
    while let Some(b) = cursor.peek() {
        if b == b' ' || b == b'\t' || b == 0x0B || b == 0x0C {
            saw_separator = true;
            cursor.set_pos(cursor.pos() + 1);
            continue;
        }
        if b == b'\n' || b == b'\r' {
            saw_separator = true;
            saw_line_terminator = true;
            cursor.set_pos(cursor.pos() + 1);
            continue;
        }
        break;
    }

    if !saw_separator || saw_line_terminator {
        cursor.set_pos(start);
        return false;
    }

    let function_pos = cursor.pos();
    if cursor
        .src
        .get(function_pos..)
        .is_some_and(|rest| rest.starts_with("function"))
        && !cursor
            .bytes()
            .get(function_pos + "function".len())
            .is_some_and(|&b| is_ident_char(b))
    {
        true
    } else {
        cursor.set_pos(start);
        false
    }
}

pub(crate) fn try_consume_async_arrow_prefix(cursor: &mut Cursor<'_>) -> bool {
    let start = cursor.pos();
    if !cursor.consume_ascii("async") {
        return false;
    }
    if cursor.peek().is_some_and(is_ident_char) {
        cursor.set_pos(start);
        return false;
    }

    let mut saw_separator = false;
    while let Some(b) = cursor.peek() {
        if b == b' ' || b == b'\t' || b == 0x0B || b == 0x0C {
            saw_separator = true;
            cursor.set_pos(cursor.pos() + 1);
            continue;
        }
        if b == b'\n' || b == b'\r' {
            cursor.set_pos(start);
            return false;
        }
        break;
    }
    if !saw_separator {
        cursor.set_pos(start);
        return false;
    }

    let pos = cursor.pos();
    if cursor
        .src
        .get(pos..)
        .is_some_and(|rest| rest.starts_with("function"))
        && !cursor
            .bytes()
            .get(pos + "function".len())
            .is_some_and(|&b| is_ident_char(b))
    {
        cursor.set_pos(start);
        return false;
    }
    true
}

fn parse_plain_function_expression_name(src: &str) -> Result<Option<String>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("function") {
        return Ok(None);
    }
    if cursor.peek().is_some_and(is_ident_char) {
        return Ok(None);
    }

    cursor.skip_ws();
    if cursor.peek() == Some(b'*') {
        return Ok(None);
    }
    if cursor.consume_byte(b'(') {
        return Ok(None);
    }

    let name = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse("expected function name".into()))?;
    Ok(Some(name))
}

pub(crate) fn parse_function_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    {
        let mut cursor = Cursor::new(src);
        cursor.skip_ws();
        if cursor.consume_ascii("function") {
            cursor.skip_ws();
            if cursor.consume_byte(b'*') {
                cursor.skip_ws();
                let function_name = if cursor.consume_byte(b'(') {
                    None
                } else {
                    let name = cursor
                        .parse_identifier()
                        .ok_or_else(|| Error::ScriptParse("expected function name".into()))?;
                    cursor.skip_ws();
                    cursor.expect_byte(b'(')?;
                    Some(name)
                };
                let params_src = cursor.read_until_byte(b')')?;
                cursor.expect_byte(b')')?;
                let parsed_params =
                    parse_callback_parameter_list(&params_src, usize::MAX, "function parameters")?;
                cursor.skip_ws();
                let body = cursor.read_balanced_block(b'{', b'}')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                let body_stmts = prepend_callback_param_prologue_stmts(
                    parse_block_statements(&body)?,
                    &parsed_params.prologue,
                )?;
                return Ok(Some(Expr::Function {
                    handler: ScriptHandler {
                        params: parsed_params.params,
                        stmts: body_stmts,
                    },
                    name: function_name,
                    is_async: false,
                    is_generator: true,
                    is_arrow: false,
                    is_method: false,
                }));
            }
        }
    }

    {
        let mut cursor = Cursor::new(src);
        cursor.skip_ws();
        if try_consume_async_function_prefix(&mut cursor) {
            cursor.consume_ascii("function");
            cursor.skip_ws();
            if cursor.consume_byte(b'*') {
                cursor.skip_ws();
                let function_name = if cursor.consume_byte(b'(') {
                    None
                } else {
                    let name = cursor
                        .parse_identifier()
                        .ok_or_else(|| Error::ScriptParse("expected function name".into()))?;
                    cursor.skip_ws();
                    cursor.expect_byte(b'(')?;
                    Some(name)
                };
                let params_src = cursor.read_until_byte(b')')?;
                cursor.expect_byte(b')')?;
                let parsed_params =
                    parse_callback_parameter_list(&params_src, usize::MAX, "function parameters")?;
                cursor.skip_ws();
                let body = cursor.read_balanced_block(b'{', b'}')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                let body_stmts = prepend_callback_param_prologue_stmts(
                    parse_block_statements(&body)?,
                    &parsed_params.prologue,
                )?;
                return Ok(Some(Expr::Function {
                    handler: ScriptHandler {
                        params: parsed_params.params,
                        stmts: body_stmts,
                    },
                    name: function_name,
                    is_async: true,
                    is_generator: true,
                    is_arrow: false,
                    is_method: false,
                }));
            }
        }
    }

    {
        let mut cursor = Cursor::new(src);
        cursor.skip_ws();
        if try_consume_async_function_prefix(&mut cursor) {
            cursor.consume_ascii("function");
            cursor.skip_ws();
            if cursor.peek() == Some(b'*') {
                return Ok(None);
            }
            let function_name = if cursor.consume_byte(b'(') {
                None
            } else {
                let name = cursor
                    .parse_identifier()
                    .ok_or_else(|| Error::ScriptParse("expected function name".into()))?;
                cursor.skip_ws();
                cursor.expect_byte(b'(')?;
                Some(name)
            };
            let params_src = cursor.read_until_byte(b')')?;
            cursor.expect_byte(b')')?;
            let parsed_params =
                parse_callback_parameter_list(&params_src, usize::MAX, "function parameters")?;
            cursor.skip_ws();
            let body = cursor.read_balanced_block(b'{', b'}')?;
            cursor.skip_ws();
            if !cursor.eof() {
                return Ok(None);
            }
            let body_stmts = prepend_callback_param_prologue_stmts(
                parse_block_statements(&body)?,
                &parsed_params.prologue,
            )?;
            return Ok(Some(Expr::Function {
                handler: ScriptHandler {
                    params: parsed_params.params,
                    stmts: body_stmts,
                },
                name: function_name,
                is_async: true,
                is_generator: false,
                is_arrow: false,
                is_method: false,
            }));
        }
    }

    {
        let mut cursor = Cursor::new(src);
        cursor.skip_ws();
        if try_consume_async_arrow_prefix(&mut cursor) {
            if let Ok((params, body, concise_body)) =
                parse_callback(&mut cursor, usize::MAX, "function parameters")
            {
                cursor.skip_ws();
                if cursor.eof() {
                    let stmts = if concise_body {
                        vec![Stmt::Return {
                            value: Some(parse_expr(body.trim())?),
                        }]
                    } else {
                        parse_block_statements(&body)?
                    };
                    return Ok(Some(Expr::Function {
                        handler: ScriptHandler { params, stmts },
                        name: None,
                        is_async: true,
                        is_generator: false,
                        is_arrow: true,
                        is_method: false,
                    }));
                }
            }
        }
    }

    if !src.starts_with("function") && !src.contains("=>") {
        return Ok(None);
    }

    let function_name = parse_plain_function_expression_name(src)?;
    let mut cursor = Cursor::new(src);
    let parsed = match parse_callback(&mut cursor, usize::MAX, "function parameters") {
        Ok(parsed) => parsed,
        Err(err) => {
            if src.starts_with("function") {
                return Err(err);
            }
            return Ok(None);
        }
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let (params, body, concise_body) = parsed;
    let stmts = if concise_body {
        vec![Stmt::Return {
            value: Some(parse_expr(body.trim())?),
        }]
    } else {
        parse_block_statements(&body)?
    };
    Ok(Some(Expr::Function {
        handler: ScriptHandler { params, stmts },
        name: function_name,
        is_async: false,
        is_generator: false,
        is_arrow: !src.starts_with("function"),
        is_method: false,
    }))
}

pub(crate) fn parse_callback(
    cursor: &mut Cursor<'_>,
    max_params: usize,
    label: &str,
) -> Result<(Vec<FunctionParam>, String, bool)> {
    cursor.skip_ws();

    let parsed_params = if cursor
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

        let params = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        let parsed_params = parse_callback_parameter_list(&params, max_params, label)?;
        cursor.skip_ws();
        let body = cursor.read_balanced_block(b'{', b'}')?;
        let (body, concise_body) =
            inject_callback_param_prologue(body, false, &parsed_params.prologue);
        return Ok((parsed_params.params, body, concise_body));
    } else if cursor.consume_byte(b'(') {
        let params = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        parse_callback_parameter_list(&params, max_params, label)?
    } else {
        let ident = cursor
            .parse_identifier()
            .ok_or_else(|| Error::ScriptParse("expected callback parameter or ()".into()))?;
        ParsedCallbackParams {
            params: vec![FunctionParam {
                name: ident,
                default: None,
                is_rest: false,
            }],
            prologue: Vec::new(),
        }
    };

    skip_arrow_whitespace_without_line_terminator(cursor)?;
    cursor.expect_ascii("=>")?;
    let (body, concise_body) = parse_arrow_or_block_body(cursor)?;
    let (body, concise_body) =
        inject_callback_param_prologue(body, concise_body, &parsed_params.prologue);
    Ok((parsed_params.params, body, concise_body))
}

pub(crate) fn parse_timer_callback(timer_name: &str, src: &str) -> Result<TimerCallback> {
    let mut cursor = Cursor::new(src);
    if let Ok((params, body, _)) =
        parse_callback(&mut cursor, usize::MAX, "timer callback parameters")
    {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(TimerCallback::Inline(ScriptHandler {
                params,
                stmts: parse_block_statements(&body)?,
            }));
        }
    }

    match parse_expr(src)? {
        Expr::Function { handler, .. } => Ok(TimerCallback::Inline(handler)),
        Expr::Var(name) => Ok(TimerCallback::Reference(name)),
        _ => Err(Error::ScriptParse(format!(
            "unsupported {timer_name} callback: {src}"
        ))),
    }
}
