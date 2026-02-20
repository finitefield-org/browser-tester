pub(super) fn parse_node_tree_mutation_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let method = match method.as_str() {
        "after" => NodeTreeMethod::After,
        "append" => NodeTreeMethod::Append,
        "appendChild" => NodeTreeMethod::AppendChild,
        "before" => NodeTreeMethod::Before,
        "replaceWith" => NodeTreeMethod::ReplaceWith,
        "prepend" => NodeTreeMethod::Prepend,
        "removeChild" => NodeTreeMethod::RemoveChild,
        "insertBefore" => NodeTreeMethod::InsertBefore,
        _ => return Ok(None),
    };
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    let (method_name, expected_args) = match method {
        NodeTreeMethod::After => ("after", 1),
        NodeTreeMethod::Append => ("append", 1),
        NodeTreeMethod::AppendChild => ("appendChild", 1),
        NodeTreeMethod::Before => ("before", 1),
        NodeTreeMethod::ReplaceWith => ("replaceWith", 1),
        NodeTreeMethod::Prepend => ("prepend", 1),
        NodeTreeMethod::RemoveChild => ("removeChild", 1),
        NodeTreeMethod::InsertBefore => ("insertBefore", 2),
    };
    if args.len() != expected_args {
        return Err(Error::ScriptParse(format!(
            "{} requires {} argument{}: {}",
            method_name,
            expected_args,
            if expected_args == 1 { "" } else { "s" },
            stmt
        )));
    }
    let child = parse_expr(args[0].trim())?;
    let reference = if method == NodeTreeMethod::InsertBefore {
        Some(parse_expr(args[1].trim())?)
    } else {
        None
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported node tree mutation statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::NodeTreeMutation {
        target,
        method,
        child,
        reference,
    }))
}

pub(super) fn parse_node_remove_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if method != "remove" {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported remove() statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::NodeRemove { target }))
}

pub(super) fn parse_dispatch_event_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("dispatchEvent") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 {
        return Err(Error::ScriptParse(format!(
            "dispatchEvent requires 1 argument: {stmt}"
        )));
    }
    let event_type = parse_expr(args[0].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported dispatchEvent statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::DispatchEvent { target, event_type }))
}

enum ListenerCallbackParseResult {
    Inline {
        params: Vec<FunctionParam>,
        body: String,
    },
    Reference(String),
}

fn parse_listener_callback_arg(cursor: &mut Cursor<'_>) -> Result<ListenerCallbackParseResult> {
    let start = cursor.pos();
    if let Some(name) = cursor.parse_identifier() {
        cursor.skip_ws();
        if matches!(cursor.peek(), Some(b',') | Some(b')')) {
            return Ok(ListenerCallbackParseResult::Reference(name));
        }
        cursor.set_pos(start);
    }

    if try_consume_async_function_prefix(cursor) || try_consume_async_arrow_prefix(cursor) {
        let (params, body, _) = parse_callback(cursor, 1, "callback parameters")?;
        return Ok(ListenerCallbackParseResult::Inline { params, body });
    }
    cursor.set_pos(start);

    let (params, body, _) = parse_callback(cursor, 1, "callback parameters")?;
    Ok(ListenerCallbackParseResult::Inline { params, body })
}

pub(super) fn build_listener_reference_handler(callback_name: &str) -> Result<ScriptHandler> {
    let mut event_param = String::from("__bt_listener_event");
    while event_param == callback_name {
        event_param.push('_');
    }
    let stmts = parse_block_statements(&format!("{callback_name}({event_param});"))?;
    Ok(ScriptHandler {
        params: vec![FunctionParam {
            name: event_param,
            default: None,
            is_rest: false,
        }],
        stmts,
    })
}

pub(super) fn parse_listener_option_key(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'\'' && last == b'\'') || (first == b'"' && last == b'"') {
            return trimmed.get(1..trimmed.len() - 1);
        }
    }
    Some(trimmed)
}

pub(super) fn parse_listener_capture_from_options_object(src: &str) -> Result<Option<bool>> {
    let mut capture = None;
    for raw_entry in split_top_level_by_char(src, b',') {
        let entry = raw_entry.trim();
        if entry.is_empty() {
            continue;
        }
        let Some((raw_key, raw_value)) = entry.split_once(':') else {
            continue;
        };
        let Some(key) = parse_listener_option_key(raw_key) else {
            continue;
        };
        if key != "capture" {
            continue;
        }
        let value = raw_value.trim();
        if value == "true" {
            capture = Some(true);
        } else if value == "false" {
            capture = Some(false);
        } else {
            return Err(Error::ScriptParse(
                "add/removeEventListener options.capture must be true/false".into(),
            ));
        }
    }
    Ok(capture)
}

pub(super) fn parse_listener_mutation_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let op = match method.as_str() {
        "addEventListener" => ListenerRegistrationOp::Add,
        "removeEventListener" => ListenerRegistrationOp::Remove,
        _ => return Ok(None),
    };
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let event_type = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b',')?;
    cursor.skip_ws();
    let callback = parse_listener_callback_arg(&mut cursor)?;

    cursor.skip_ws();
    let capture = if cursor.consume_byte(b',') {
        cursor.skip_ws();
        if cursor.consume_ascii("true") {
            true
        } else if cursor.consume_ascii("false") {
            false
        } else if cursor.peek() == Some(b'{') {
            let options_src = cursor.read_balanced_block(b'{', b'}')?;
            parse_listener_capture_from_options_object(&options_src)?.unwrap_or(false)
        } else {
            return Err(Error::ScriptParse(
                "add/removeEventListener third argument must be true/false or options object"
                    .into(),
            ));
        }
    } else {
        false
    };

    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported listener mutation statement tail: {stmt}"
        )));
    }

    let handler = match callback {
        ListenerCallbackParseResult::Inline { params, body } => ScriptHandler {
            params,
            stmts: parse_block_statements(&body)?,
        },
        ListenerCallbackParseResult::Reference(name) => build_listener_reference_handler(&name)?,
    };
    Ok(Some(Stmt::ListenerMutation {
        target,
        op,
        event_type,
        capture,
        handler,
    }))
}

pub(super) fn parse_event_call_stmt(stmt: &str) -> Option<Stmt> {
    let stmt = stmt.trim();
    let open = stmt.find('(')?;
    let close = stmt.rfind(')')?;
    if close <= open {
        return None;
    }

    let head = stmt[..open].trim();
    let args = stmt[open + 1..close].trim();
    if !args.is_empty() {
        return None;
    }

    let (event_var, method) = head.split_once('.')?;
    if !is_ident(event_var.trim()) {
        return None;
    }

    let method = match method.trim() {
        "preventDefault" => EventMethod::PreventDefault,
        "stopPropagation" => EventMethod::StopPropagation,
        "stopImmediatePropagation" => EventMethod::StopImmediatePropagation,
        _ => return None,
    };

    Some(Stmt::EventCall {
        event_var: event_var.trim().to_string(),
        method,
    })
}
