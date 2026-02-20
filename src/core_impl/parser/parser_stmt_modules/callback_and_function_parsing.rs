struct ParsedCallbackParams {
    params: Vec<FunctionParam>,
    prologue: Vec<String>,
}

pub(super) fn next_callback_temp_name(params: &[FunctionParam], seed: usize) -> String {
    let mut suffix = seed;
    loop {
        let candidate = format!("__bt_callback_arg_{suffix}");
        if params.iter().all(|param| param.name != candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

pub(super) fn format_array_destructure_pattern(pattern: &[Option<String>]) -> String {
    let mut out = String::from("[");
    for (idx, item) in pattern.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        if let Some(name) = item {
            out.push_str(name);
        }
    }
    out.push(']');
    out
}

pub(super) fn format_object_destructure_pattern(pattern: &[(String, String)]) -> String {
    let mut out = String::from("{");
    for (index, (source, target)) in pattern.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(source);
        if source != target {
            out.push_str(": ");
            out.push_str(target);
        }
    }
    out.push('}');
    out
}

pub(super) fn inject_callback_param_prologue(
    body: String,
    concise_body: bool,
    prologue: &[String],
) -> (String, bool) {
    if prologue.is_empty() {
        return (body, concise_body);
    }

    let mut rewritten = String::new();
    for stmt in prologue {
        rewritten.push_str(stmt.trim());
        if !stmt.trim_end().ends_with(';') {
            rewritten.push(';');
        }
        rewritten.push('\n');
    }

    if concise_body {
        rewritten.push_str("return ");
        rewritten.push_str(body.trim());
        rewritten.push(';');
        (rewritten, false)
    } else {
        rewritten.push_str(&body);
        (rewritten, false)
    }
}

pub(super) fn prepend_callback_param_prologue_stmts(
    mut stmts: Vec<Stmt>,
    prologue: &[String],
) -> Result<Vec<Stmt>> {
    if prologue.is_empty() {
        return Ok(stmts);
    }

    let mut src = String::new();
    for stmt in prologue {
        src.push_str(stmt.trim());
        if !stmt.trim_end().ends_with(';') {
            src.push(';');
        }
        src.push('\n');
    }
    let mut prefixed = parse_block_statements(&src)?;
    prefixed.append(&mut stmts);
    Ok(prefixed)
}

fn parse_callback_parameter_list(
    src: &str,
    max_params: usize,
    label: &str,
) -> Result<ParsedCallbackParams> {
    let parts = split_top_level_by_char(src.trim(), b',');
    if parts.len() == 1 && parts[0].trim().is_empty() {
        return Ok(ParsedCallbackParams {
            params: Vec::new(),
            prologue: Vec::new(),
        });
    }

    if parts.len() > max_params {
        return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
    }

    let mut params = Vec::new();
    let mut prologue = Vec::new();
    let mut bound_names: Vec<String> = Vec::new();
    let part_count = parts.len();
    for (index, raw) in parts.into_iter().enumerate() {
        let param = raw.trim();
        if param.is_empty() {
            return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
        }

        if let Some(rest_name) = param.strip_prefix("...") {
            let rest_name = rest_name.trim();
            if index + 1 != part_count || !is_ident(rest_name) {
                return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
            }
            params.push(FunctionParam {
                name: rest_name.to_string(),
                default: None,
                is_rest: true,
            });
            bound_names.push(rest_name.to_string());
            continue;
        }

        if let Some((eq_pos, op_len)) = find_top_level_assignment(param) {
            if op_len != 1 {
                return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
            }
            let name = param[..eq_pos].trim();
            let default_src = param[eq_pos + op_len..].trim();
            if default_src.is_empty() {
                return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
            }

            if is_ident(name) {
                params.push(FunctionParam {
                    name: name.to_string(),
                    default: Some(parse_expr(default_src)?),
                    is_rest: false,
                });
                bound_names.push(name.to_string());
                continue;
            }

            if name.starts_with('[') && name.ends_with(']') {
                let pattern = parse_array_destructure_pattern(name)?;
                let temp = next_callback_temp_name(&params, index);
                let pattern_src = format_array_destructure_pattern(&pattern);
                for bound_name in pattern.iter().flatten() {
                    if !bound_names.iter().any(|bound| bound == bound_name) {
                        prologue.push(format!("let {bound_name} = undefined;"));
                        bound_names.push(bound_name.clone());
                    }
                }
                prologue.push(format!("{pattern_src} = {temp};"));
                bound_names.push(temp.clone());
                params.push(FunctionParam {
                    name: temp,
                    default: Some(parse_expr(default_src)?),
                    is_rest: false,
                });
                continue;
            }

            if name.starts_with('{') && name.ends_with('}') {
                let pattern = parse_object_destructure_pattern(name)?;
                let temp = next_callback_temp_name(&params, index);
                let pattern_src = format_object_destructure_pattern(&pattern);
                for (_, bound_name) in &pattern {
                    if !bound_names.iter().any(|bound| bound == bound_name) {
                        prologue.push(format!("let {bound_name} = undefined;"));
                        bound_names.push(bound_name.clone());
                    }
                }
                prologue.push(format!("{pattern_src} = {temp};"));
                bound_names.push(temp.clone());
                params.push(FunctionParam {
                    name: temp,
                    default: Some(parse_expr(default_src)?),
                    is_rest: false,
                });
                continue;
            }

            return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
        }

        if is_ident(param) {
            params.push(FunctionParam {
                name: param.to_string(),
                default: None,
                is_rest: false,
            });
            bound_names.push(param.to_string());
            continue;
        }

        if param.starts_with('[') && param.ends_with(']') {
            let pattern = parse_array_destructure_pattern(param)?;
            let temp = next_callback_temp_name(&params, index);
            let pattern_src = format_array_destructure_pattern(&pattern);
            for name in pattern.iter().flatten() {
                if !bound_names.iter().any(|bound| bound == name) {
                    prologue.push(format!("let {name} = undefined;"));
                    bound_names.push(name.clone());
                }
            }
            prologue.push(format!("{pattern_src} = {temp};"));
            bound_names.push(temp.clone());
            params.push(FunctionParam {
                name: temp,
                default: None,
                is_rest: false,
            });
            continue;
        }

        if param.starts_with('{') && param.ends_with('}') {
            let pattern = parse_object_destructure_pattern(param)?;
            let temp = next_callback_temp_name(&params, index);
            let pattern_src = format_object_destructure_pattern(&pattern);
            for (_, name) in &pattern {
                if !bound_names.iter().any(|bound| bound == name) {
                    prologue.push(format!("let {name} = undefined;"));
                    bound_names.push(name.clone());
                }
            }
            prologue.push(format!("{pattern_src} = {temp};"));
            bound_names.push(temp.clone());
            params.push(FunctionParam {
                name: temp,
                default: None,
                is_rest: false,
            });
            continue;
        }

        return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
    }

    Ok(ParsedCallbackParams { params, prologue })
}

pub(super) fn parse_arrow_or_block_body(cursor: &mut Cursor<'_>) -> Result<(String, bool)> {
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

        if parse_expr(stripped).is_ok() {
            cursor.set_pos(cursor.i + end);
            return Ok((stripped.to_string(), true));
        }

        if let Some(rewritten) = rewrite_assignment_arrow_body(stripped)? {
            cursor.set_pos(cursor.i + end);
            return Ok((rewritten, false));
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

pub(super) fn rewrite_assignment_arrow_body(expr_src: &str) -> Result<Option<String>> {
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

pub(super) fn is_valid_callback_body_suffix(suffix: &str) -> bool {
    suffix
        .chars()
        .all(|ch| ch.is_ascii_whitespace() || matches!(ch, ')' | ']' | '}' | ',' | ';'))
}

pub(super) fn skip_arrow_whitespace_without_line_terminator(cursor: &mut Cursor<'_>) -> Result<()> {
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

pub(super) fn try_consume_async_function_prefix(cursor: &mut Cursor<'_>) -> bool {
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

pub(super) fn try_consume_async_arrow_prefix(cursor: &mut Cursor<'_>) -> bool {
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

pub(super) fn parse_function_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    {
        let mut cursor = Cursor::new(src);
        cursor.skip_ws();
        if try_consume_async_function_prefix(&mut cursor) {
            let (params, body, concise_body) =
                parse_callback(&mut cursor, usize::MAX, "function parameters")?;
            cursor.skip_ws();
            if !cursor.eof() {
                return Ok(None);
            }
            let stmts = if concise_body {
                vec![Stmt::Return {
                    value: Some(parse_expr(body.trim())?),
                }]
            } else {
                parse_block_statements(&body)?
            };
            return Ok(Some(Expr::Function {
                handler: ScriptHandler { params, stmts },
                is_async: true,
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
                        is_async: true,
                    }));
                }
            }
        }
    }

    if !src.starts_with("function") && !src.contains("=>") {
        return Ok(None);
    }

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
        is_async: false,
    }))
}

pub(super) fn parse_callback(
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

pub(super) fn parse_timer_callback(timer_name: &str, src: &str) -> Result<TimerCallback> {
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

