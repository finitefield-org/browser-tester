use super::*;
pub(crate) fn parse_function_decl_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    let is_async = if try_consume_async_function_prefix(&mut cursor) {
        cursor.consume_ascii("function");
        true
    } else {
        if !cursor.consume_ascii("function") {
            return Ok(None);
        }
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        false
    };
    cursor.skip_ws();

    let Some(name) = cursor.parse_identifier() else {
        return Err(Error::ScriptParse(
            "function declaration requires a function name".into(),
        ));
    };
    cursor.skip_ws();
    let params_src = cursor.read_balanced_block(b'(', b')')?;
    let parsed_params =
        parse_callback_parameter_list(&params_src, usize::MAX, "function parameters")?;
    cursor.skip_ws();
    let body = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported function declaration tail: {stmt}"
        )));
    }

    let body_stmts = prepend_callback_param_prologue_stmts(
        parse_block_statements(&body)?,
        &parsed_params.prologue,
    )?;

    Ok(Some(Stmt::FunctionDecl {
        name,
        handler: ScriptHandler {
            params: parsed_params.params,
            stmts: body_stmts,
        },
        is_async,
    }))
}

pub(crate) fn parse_var_decl(stmt: &str) -> Result<Option<Stmt>> {
    let mut rest = None;
    for kw in ["const", "let", "var"] {
        if let Some(after) = stmt.strip_prefix(kw) {
            if after
                .as_bytes()
                .first()
                .is_some_and(|b| is_ident_char(*b))
            {
                continue;
            }
            rest = Some(after.trim_start());
            break;
        }
    }

    let Some(rest) = rest else {
        return Ok(None);
    };

    let Some((eq_pos, op_len)) = find_top_level_assignment(rest) else {
        return Err(Error::ScriptParse(format!(
            "invalid variable declaration: {stmt}"
        )));
    };
    if op_len != 1 {
        return Err(Error::ScriptParse(format!(
            "invalid variable declaration: {stmt}"
        )));
    }

    let name = rest[..eq_pos].trim();
    let expr_src = rest[eq_pos + op_len..].trim();
    if name.is_empty() || expr_src.is_empty() {
        return Err(Error::ScriptParse(format!(
            "invalid variable declaration: {stmt}"
        )));
    }

    if name.starts_with('[') && name.ends_with(']') {
        let targets = parse_array_destructure_pattern(name)?;
        let expr = parse_expr(expr_src)?;
        return Ok(Some(Stmt::ArrayDestructureAssign { targets, expr }));
    }
    if name.starts_with('{') && name.ends_with('}') {
        let bindings = parse_object_destructure_pattern(name)?;
        let expr = parse_expr(expr_src)?;
        return Ok(Some(Stmt::ObjectDestructureAssign { bindings, expr }));
    }

    if !is_ident(name) {
        return Err(Error::ScriptParse(format!(
            "invalid variable name '{name}' in: {stmt}"
        )));
    }

    let expr = parse_expr(expr_src)?;
    Ok(Some(Stmt::VarDecl {
        name: name.to_string(),
        expr,
    }))
}

pub(crate) fn parse_var_assign(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let Some((name, op_len, value_src)) = find_top_level_var_assignment(stmt) else {
        return Ok(None);
    };

    if name.is_empty() || !is_ident(&name) {
        return Ok(None);
    }

    let split_pos = stmt.len() - value_src.len();
    let op = match &stmt[split_pos - op_len..split_pos] {
        "=" => VarAssignOp::Assign,
        "+=" => VarAssignOp::Add,
        "-=" => VarAssignOp::Sub,
        "*=" => VarAssignOp::Mul,
        "/=" => VarAssignOp::Div,
        "**=" => VarAssignOp::Pow,
        "%=" => VarAssignOp::Mod,
        "|=" => VarAssignOp::BitOr,
        "^=" => VarAssignOp::BitXor,
        "&=" => VarAssignOp::BitAnd,
        "<<=" => VarAssignOp::ShiftLeft,
        ">>=" => VarAssignOp::ShiftRight,
        ">>>=" => VarAssignOp::UnsignedShiftRight,
        "&&=" => VarAssignOp::LogicalAnd,
        "||=" => VarAssignOp::LogicalOr,
        "??=" => VarAssignOp::Nullish,
        _ => {
            return Err(Error::ScriptParse(format!(
                "unsupported assignment operator: {stmt}"
            )));
        }
    };

    let expr = parse_expr(value_src)?;
    Ok(Some(Stmt::VarAssign {
        name: name.to_string(),
        op,
        expr,
    }))
}

pub(crate) fn find_top_level_var_assignment(stmt: &str) -> Option<(String, usize, &str)> {
    let (eq_pos, op_len) = find_top_level_assignment(stmt)?;
    let lhs = stmt[..eq_pos].trim();
    if lhs.is_empty() {
        return None;
    }

    Some((
        lhs.to_string(),
        op_len,
        stmt.get(eq_pos + op_len..).unwrap_or_default(),
    ))
}

pub(crate) fn parse_destructure_assign(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let Some((eq_pos, op_len)) = find_top_level_assignment(stmt) else {
        return Ok(None);
    };
    if op_len != 1 {
        return Ok(None);
    }

    let lhs = stmt[..eq_pos].trim();
    let rhs = stmt[eq_pos + op_len..].trim();
    if lhs.is_empty() || rhs.is_empty() {
        return Ok(None);
    }

    if lhs.starts_with('[') && lhs.ends_with(']') {
        let targets = parse_array_destructure_pattern(lhs)?;
        let expr = parse_expr(rhs)?;
        return Ok(Some(Stmt::ArrayDestructureAssign { targets, expr }));
    }
    if lhs.starts_with('{') && lhs.ends_with('}') {
        let bindings = parse_object_destructure_pattern(lhs)?;
        let expr = parse_expr(rhs)?;
        return Ok(Some(Stmt::ObjectDestructureAssign { bindings, expr }));
    }

    Ok(None)
}

pub(crate) fn parse_array_destructure_pattern(pattern: &str) -> Result<Vec<Option<String>>> {
    let mut cursor = Cursor::new(pattern);
    cursor.skip_ws();
    let items_src = cursor.read_balanced_block(b'[', b']')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "invalid array destructuring pattern: {pattern}"
        )));
    }

    let mut items = split_top_level_by_char(&items_src, b',');
    while items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty()) {
        items.pop();
    }
    if items.len() == 1 && items[0].trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut targets = Vec::with_capacity(items.len());
    for item in items {
        let item = item.trim();
        if item.is_empty() {
            targets.push(None);
            continue;
        }
        if !is_ident(item) {
            return Err(Error::ScriptParse(format!(
                "array destructuring target must be an identifier: {item}"
            )));
        }
        targets.push(Some(item.to_string()));
    }
    Ok(targets)
}

pub(crate) fn parse_object_destructure_pattern(pattern: &str) -> Result<Vec<(String, String)>> {
    let mut cursor = Cursor::new(pattern);
    cursor.skip_ws();
    let items_src = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "invalid object destructuring pattern: {pattern}"
        )));
    }

    let mut items = split_top_level_by_char(&items_src, b',');
    while items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty()) {
        items.pop();
    }
    if items.len() == 1 && items[0].trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut bindings = Vec::with_capacity(items.len());
    for item in items {
        let item = item.trim();
        if item.is_empty() {
            return Err(Error::ScriptParse(
                "object destructuring pattern does not support empty entries".into(),
            ));
        }

        if let Some(colon) = find_first_top_level_colon(item) {
            let source = item[..colon].trim();
            let target = item[colon + 1..].trim();
            if !is_ident(source) || !is_ident(target) {
                return Err(Error::ScriptParse(format!(
                    "object destructuring entry must be identifier or identifier: identifier: {item}"
                )));
            }
            bindings.push((source.to_string(), target.to_string()));
        } else {
            if !is_ident(item) {
                return Err(Error::ScriptParse(format!(
                    "object destructuring entry must be an identifier: {item}"
                )));
            }
            bindings.push((item.to_string(), item.to_string()));
        }
    }

    Ok(bindings)
}
