use super::*;
pub(crate) fn parse_node_tree_mutation_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

pub(crate) fn parse_node_remove_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
