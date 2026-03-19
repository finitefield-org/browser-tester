use super::*;

pub(crate) fn parse_var_decl(stmt: &str) -> Result<Option<Stmt>> {
    let mut decl_kind = None;
    let mut rest = None;
    for kw in ["const", "let", "var"] {
        if let Some(after) = stmt.strip_prefix(kw) {
            if after.as_bytes().first().is_some_and(|b| is_ident_char(*b)) {
                continue;
            }
            decl_kind = Some(kw);
            rest = Some(after.trim_start());
            break;
        }
    }

    let Some(rest) = rest else {
        return Ok(None);
    };
    let decl_kind = decl_kind.unwrap_or("let");
    let kind = match decl_kind {
        "var" => VarDeclKind::Var,
        "const" => VarDeclKind::Const,
        _ => VarDeclKind::Let,
    };

    let Some((eq_pos, op_len)) = find_top_level_assignment(rest) else {
        if decl_kind == "const" {
            return Err(Error::ScriptParse(format!(
                "const declaration requires initializer: {stmt}"
            )));
        }
        let name = rest.trim();
        if name.is_empty() {
            return Err(Error::ScriptParse(format!(
                "invalid variable declaration: {stmt}"
            )));
        }
        if !is_ident(name) {
            return Err(Error::ScriptParse(format!(
                "invalid variable name '{name}' in: {stmt}"
            )));
        }
        return Ok(Some(Stmt::VarDecl {
            name: name.to_string(),
            kind,
            expr: Expr::Undefined,
        }));
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
        let pattern = parse_array_destructure_assignment_pattern(name)?;
        let expr = parse_expr(expr_src)?;
        return Ok(Some(Stmt::ArrayDestructureAssign {
            pattern,
            expr,
            decl_kind: Some(kind),
        }));
    }
    if name.starts_with('{') && name.ends_with('}') {
        let pattern = parse_object_destructure_assignment_pattern(name)?;
        let expr = parse_expr(expr_src)?;
        return Ok(Some(Stmt::ObjectDestructureAssign {
            pattern,
            expr,
            decl_kind: Some(kind),
        }));
    }

    if !is_ident(name) {
        return Err(Error::ScriptParse(format!(
            "invalid variable name '{name}' in: {stmt}"
        )));
    }

    let expr = parse_expr(expr_src)?;
    Ok(Some(Stmt::VarDecl {
        name: name.to_string(),
        kind,
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
        let pattern = match parse_array_destructure_assignment_pattern(lhs) {
            Ok(pattern) => pattern,
            Err(err) => {
                if let Some(lowered) = lower_array_destructure_assignment_stmt(lhs, rhs)? {
                    return Ok(Some(lowered));
                }
                return Err(err);
            }
        };
        let expr = parse_expr(rhs)?;
        return Ok(Some(Stmt::ArrayDestructureAssign {
            pattern,
            expr,
            decl_kind: None,
        }));
    }
    if lhs.starts_with('{') && lhs.ends_with('}') {
        let pattern = parse_object_destructure_assignment_pattern(lhs)?;
        let expr = parse_expr(rhs)?;
        return Ok(Some(Stmt::ObjectDestructureAssign {
            pattern,
            expr,
            decl_kind: None,
        }));
    }

    Ok(None)
}

fn lower_array_destructure_assignment_stmt(lhs: &str, rhs: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(lhs);
    cursor.skip_ws();
    let items_src = cursor.read_balanced_block(b'[', b']')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let mut items = split_top_level_by_char(&items_src, b',');
    while items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty()) {
        items.pop();
    }

    let mut parsed_items = Vec::with_capacity(items.len());
    let mut needs_lowering = false;
    for item in items {
        let item = item.trim();
        if item.is_empty() {
            parsed_items.push(None);
            continue;
        }
        if item.strip_prefix("...").is_some() || find_top_level_assignment(item).is_some() {
            return Ok(None);
        }
        if !is_valid_destructure_assignment_target(item) {
            return Ok(None);
        }
        if !is_ident(item) {
            needs_lowering = true;
        }
        parsed_items.push(Some(item.to_string()));
    }

    if !needs_lowering {
        return Ok(None);
    }

    let temp_name = fresh_destructure_temp_name(lhs, rhs);
    let mut lowered = format!("{{ const {temp_name} = {rhs};");
    for (index, target) in parsed_items.iter().enumerate() {
        let Some(target) = target else {
            continue;
        };
        lowered.push(' ');
        lowered.push_str(target);
        lowered.push_str(" = ");
        lowered.push_str(&temp_name);
        lowered.push('[');
        lowered.push_str(&index.to_string());
        lowered.push_str("];");
    }
    lowered.push_str(" }");

    Ok(parse_block_stmt(&lowered)?)
}

fn fresh_destructure_temp_name(lhs: &str, rhs: &str) -> String {
    let mut index = 0usize;
    loop {
        let candidate = format!("__bt_array_destructure_{index}");
        if !lhs.contains(&candidate) && !rhs.contains(&candidate) {
            return candidate;
        }
        index += 1;
    }
}

fn is_valid_destructure_assignment_target(target: &str) -> bool {
    let assignment_src = format!("{target} = 0");
    let supports_assignment = |result: Result<Option<Stmt>>| match result {
        Ok(Some(_)) => true,
        Ok(None) | Err(_) => false,
    };

    supports_assignment(parse_var_assign(&assignment_src))
        || supports_assignment(parse_object_assign(&assignment_src))
        || supports_assignment(parse_private_assign(&assignment_src))
        || supports_assignment(parse_dom_assignment(&assignment_src))
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

pub(crate) fn parse_array_destructure_assignment_pattern(
    pattern: &str,
) -> Result<ArrayDestructurePattern> {
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
    let had_trailing_empty =
        items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty());
    while items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty()) {
        items.pop();
    }
    if items.len() == 1 && items[0].trim().is_empty() {
        return Ok(ArrayDestructurePattern {
            items: Vec::new(),
            rest: None,
        });
    }

    let mut parsed_items = Vec::with_capacity(items.len());
    let mut rest = None;
    for item in items {
        let item = item.trim();
        if item.is_empty() {
            if rest.is_some() {
                return Err(Error::ScriptParse(
                    "array destructuring rest element must be last".into(),
                ));
            }
            parsed_items.push(None);
            continue;
        }

        if let Some(rest_name) = item.strip_prefix("...") {
            let rest_name = rest_name.trim();
            if rest_name.is_empty() || !is_ident(rest_name) {
                return Err(Error::ScriptParse(format!(
                    "array destructuring rest target must be an identifier: {item}"
                )));
            }
            if rest.is_some() {
                return Err(Error::ScriptParse(
                    "array destructuring pattern cannot contain multiple rest elements".into(),
                ));
            }
            rest = Some(rest_name.to_string());
            continue;
        }

        if rest.is_some() {
            return Err(Error::ScriptParse(
                "array destructuring rest element must be last".into(),
            ));
        }

        let (name, default) = if let Some((eq_pos, op_len)) = find_top_level_assignment(item) {
            if op_len != 1 {
                return Err(Error::ScriptParse(format!(
                    "array destructuring target must be an identifier: {item}"
                )));
            }
            let name = item[..eq_pos].trim();
            let default_src = item[eq_pos + op_len..].trim();
            if !is_ident(name) || default_src.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "array destructuring target must be an identifier: {item}"
                )));
            }
            (name.to_string(), Some(parse_expr(default_src)?))
        } else {
            if !is_ident(item) {
                return Err(Error::ScriptParse(format!(
                    "array destructuring target must be an identifier: {item}"
                )));
            }
            (item.to_string(), None)
        };

        parsed_items.push(Some(ArrayDestructureBinding {
            target: name,
            default,
        }));
    }

    if rest.is_some() && had_trailing_empty {
        return Err(Error::ScriptParse(
            "array destructuring rest element may not have a trailing comma".into(),
        ));
    }

    Ok(ArrayDestructurePattern {
        items: parsed_items,
        rest,
    })
}

pub(crate) fn parse_object_destructure_assignment_pattern(
    pattern: &str,
) -> Result<ObjectDestructurePattern> {
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
    let had_trailing_empty =
        items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty());
    while items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty()) {
        items.pop();
    }
    if items.len() == 1 && items[0].trim().is_empty() {
        return Ok(ObjectDestructurePattern {
            bindings: Vec::new(),
            rest: None,
        });
    }

    let mut bindings = Vec::with_capacity(items.len());
    let mut rest = None;

    for item in items {
        let item = item.trim();
        if item.is_empty() {
            return Err(Error::ScriptParse(
                "object destructuring pattern does not support empty entries".into(),
            ));
        }

        if let Some(rest_name) = item.strip_prefix("...") {
            let rest_name = rest_name.trim();
            if rest_name.is_empty() || !is_ident(rest_name) {
                return Err(Error::ScriptParse(
                    "object destructuring rest property must be an identifier".into(),
                ));
            }
            if rest.is_some() {
                return Err(Error::ScriptParse(
                    "object destructuring pattern cannot contain multiple rest properties".into(),
                ));
            }
            rest = Some(rest_name.to_string());
            continue;
        }

        if rest.is_some() {
            return Err(Error::ScriptParse(
                "object destructuring rest property must be last".into(),
            ));
        }

        let binding = if let Some(colon) = find_first_top_level_colon(item) {
            let source = item[..colon].trim();
            let target_src = item[colon + 1..].trim();
            if !is_ident(source) {
                return Err(Error::ScriptParse(format!(
                    "object destructuring entry must be identifier or identifier: identifier: {item}"
                )));
            }
            let (target, default) = if let Some((eq_pos, op_len)) =
                find_top_level_assignment(target_src)
            {
                if op_len != 1 {
                    return Err(Error::ScriptParse(format!(
                        "object destructuring entry must be identifier or identifier: identifier: {item}"
                    )));
                }
                let target = target_src[..eq_pos].trim();
                let default_src = target_src[eq_pos + op_len..].trim();
                if !is_ident(target) || default_src.is_empty() {
                    return Err(Error::ScriptParse(format!(
                        "object destructuring entry must be identifier or identifier: identifier: {item}"
                    )));
                }
                (target.to_string(), Some(parse_expr(default_src)?))
            } else {
                if !is_ident(target_src) {
                    return Err(Error::ScriptParse(format!(
                        "object destructuring entry must be identifier or identifier: identifier: {item}"
                    )));
                }
                (target_src.to_string(), None)
            };
            ObjectDestructureBinding {
                source: source.to_string(),
                target,
                default,
            }
        } else {
            let (name, default) = if let Some((eq_pos, op_len)) = find_top_level_assignment(item) {
                if op_len != 1 {
                    return Err(Error::ScriptParse(format!(
                        "object destructuring entry must be identifier or identifier: identifier: {item}"
                    )));
                }
                let name = item[..eq_pos].trim();
                let default_src = item[eq_pos + op_len..].trim();
                if !is_ident(name) || default_src.is_empty() {
                    return Err(Error::ScriptParse(format!(
                        "object destructuring entry must be identifier or identifier: identifier: {item}"
                    )));
                }
                (name.to_string(), Some(parse_expr(default_src)?))
            } else {
                if !is_ident(item) {
                    return Err(Error::ScriptParse(format!(
                        "object destructuring entry must be identifier or identifier: identifier: {item}"
                    )));
                }
                (item.to_string(), None)
            };
            ObjectDestructureBinding {
                source: name.clone(),
                target: name,
                default,
            }
        };

        bindings.push(binding);
    }

    if rest.is_some() && had_trailing_empty {
        return Err(Error::ScriptParse(
            "object destructuring rest property may not have a trailing comma".into(),
        ));
    }

    Ok(ObjectDestructurePattern { bindings, rest })
}
