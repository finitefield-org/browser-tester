use super::*;

pub(crate) fn parse_set_interval_expr(src: &str) -> Result<Option<(TimerInvocation, Expr)>> {
    let mut cursor = Cursor::new(src);
    let Some((handler, delay_ms)) = parse_set_interval_call(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((handler, delay_ms)))
}

pub(crate) fn parse_set_timeout_expr(src: &str) -> Result<Option<(TimerInvocation, Expr)>> {
    let mut cursor = Cursor::new(src);
    let Some((handler, delay_ms)) = parse_set_timeout_call(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((handler, delay_ms)))
}

pub(crate) fn parse_request_animation_frame_expr(src: &str) -> Result<Option<TimerCallback>> {
    let mut cursor = Cursor::new(src);
    let callback = parse_request_animation_frame_call(&mut cursor)?;
    cursor.skip_ws();
    if cursor.eof() { Ok(callback) } else { Ok(None) }
}

fn parse_request_animation_frame_call(cursor: &mut Cursor<'_>) -> Result<Option<TimerCallback>> {
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("requestAnimationFrame") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "requestAnimationFrame requires at least one argument".into(),
        ));
    }

    let callback_arg = strip_js_comments(args[0]);
    let callback = parse_timer_callback("requestAnimationFrame", callback_arg.as_str().trim())?;
    Ok(Some(callback))
}

pub(crate) fn parse_queue_microtask_expr(src: &str) -> Result<Option<ScriptHandler>> {
    let mut cursor = Cursor::new(src);
    let handler = parse_queue_microtask_call(&mut cursor)?;
    cursor.skip_ws();
    if cursor.eof() { Ok(handler) } else { Ok(None) }
}

pub(crate) fn parse_queue_microtask_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let Some(handler) = parse_queue_microtask_call(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported queueMicrotask statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::QueueMicrotask { handler }))
}

fn parse_queue_microtask_call(cursor: &mut Cursor<'_>) -> Result<Option<ScriptHandler>> {
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("queueMicrotask") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }

    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "queueMicrotask requires exactly one argument".into(),
        ));
    }

    let callback_arg = strip_js_comments(args[0]);
    let mut callback_cursor = Cursor::new(callback_arg.as_str().trim());
    let (params, body, _) = match parse_callback(&mut callback_cursor, 1, "callback parameters") {
        Ok(parsed) => parsed,
        Err(_) => return Ok(None),
    };
    callback_cursor.skip_ws();
    if !callback_cursor.eof() {
        return Ok(None);
    }

    Ok(Some(ScriptHandler {
        params,
        stmts: parse_block_statements(&body)?,
    }))
}
