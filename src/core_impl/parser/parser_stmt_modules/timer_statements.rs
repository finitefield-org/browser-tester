use super::*;

pub(crate) fn parse_set_timer_call(
    cursor: &mut Cursor<'_>,
    timer_name: &str,
) -> Result<Option<(TimerInvocation, Expr)>> {
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii(timer_name) {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() {
        return Err(Error::ScriptParse(format!(
            "{timer_name} requires at least 1 argument"
        )));
    }

    let callback_arg = strip_js_comments(args[0]);
    let callback = parse_timer_callback(timer_name, callback_arg.as_str().trim())?;

    let delay_ms = if args.len() >= 2 {
        let delay_src = strip_js_comments(args[1]).trim().to_string();
        if delay_src.is_empty() {
            Expr::Number(0)
        } else {
            parse_expr(&delay_src)?
        }
    } else {
        Expr::Number(0)
    };

    let mut extra_args = Vec::new();
    for arg in args.iter().skip(2) {
        let arg_src = strip_js_comments(arg);
        if arg_src.trim().is_empty() {
            continue;
        }
        extra_args.push(parse_expr(arg_src.trim())?);
    }

    Ok(Some((
        TimerInvocation {
            callback,
            args: extra_args,
        },
        delay_ms,
    )))
}

pub(crate) fn parse_set_timeout_call(
    cursor: &mut Cursor<'_>,
) -> Result<Option<(TimerInvocation, Expr)>> {
    parse_set_timer_call(cursor, "setTimeout")
}

pub(crate) fn parse_set_interval_call(
    cursor: &mut Cursor<'_>,
) -> Result<Option<(TimerInvocation, Expr)>> {
    parse_set_timer_call(cursor, "setInterval")
}

pub(crate) fn parse_set_timeout_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let Some((handler, delay_ms)) = parse_set_timeout_call(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported setTimeout statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::SetTimeout { handler, delay_ms }))
}

pub(crate) fn parse_set_interval_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let Some((handler, delay_ms)) = parse_set_interval_call(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported setInterval statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::SetInterval { handler, delay_ms }))
}

pub(crate) fn parse_clear_timeout_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }
    let method = if cursor.consume_ascii("clearTimeout") {
        "clearTimeout"
    } else if cursor.consume_ascii("clearInterval") {
        "clearInterval"
    } else if cursor.consume_ascii("cancelAnimationFrame") {
        "cancelAnimationFrame"
    } else {
        return Ok(None);
    };
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(format!(
            "{method} requires 1 argument: {stmt}"
        )));
    }
    let timer_id = parse_expr(args[0].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported {method} statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::ClearTimeout { timer_id }))
}
