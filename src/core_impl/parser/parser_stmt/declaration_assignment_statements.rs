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
    let is_generator = cursor.consume_byte(b'*');
    if is_generator {
        cursor.skip_ws();
    }

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
        is_generator,
    }))
}

pub(crate) fn parse_class_decl_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    if !consume_keyword(&mut cursor, "class") {
        return Ok(None);
    }
    cursor.skip_ws();

    let Some(name) = cursor.parse_identifier() else {
        return Err(Error::ScriptParse(
            "class declaration requires a class name".into(),
        ));
    };
    cursor.skip_ws();

    let mut super_class = None;
    if consume_keyword(&mut cursor, "extends") {
        cursor.skip_ws();
        let extends_start = cursor.pos();
        let bytes = cursor.bytes();
        let mut scanner = JsLexScanner::new();
        let mut body_open = None;
        let mut i = extends_start;
        while i < bytes.len() {
            if scanner.is_top_level() && bytes[i] == b'{' {
                body_open = Some(i);
                break;
            }
            i = scanner.advance(bytes, i);
        }
        let Some(body_open) = body_open else {
            return Err(Error::ScriptParse(
                "class declaration requires a body".into(),
            ));
        };
        let super_src = stmt
            .get(extends_start..body_open)
            .unwrap_or("")
            .trim()
            .to_string();
        if super_src.is_empty() {
            return Err(Error::ScriptParse(
                "class extends requires a superclass expression".into(),
            ));
        }
        super_class = Some(parse_expr(&super_src)?);
        cursor.set_pos(body_open);
    }

    cursor.skip_ws();
    let body_src = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported class declaration tail: {stmt}"
        )));
    }

    let (constructor, methods) = parse_class_body(&body_src)?;
    Ok(Some(Stmt::ClassDecl {
        name,
        super_class,
        constructor,
        methods,
    }))
}

pub(crate) fn parse_class_body(
    body_src: &str,
) -> Result<(Option<ScriptHandler>, Vec<ClassMethodDecl>)> {
    let mut cursor = Cursor::new(body_src);
    let mut constructor = None;
    let mut methods = Vec::new();

    while !cursor.eof() {
        cursor.skip_ws();
        while cursor.consume_byte(b';') {
            cursor.skip_ws();
        }
        if cursor.eof() {
            break;
        }

        let is_async = if consume_keyword(&mut cursor, "async") {
            cursor.skip_ws();
            true
        } else {
            false
        };

        let is_generator = cursor.consume_byte(b'*');
        if is_generator {
            cursor.skip_ws();
        }

        let Some(method_name) = cursor.parse_identifier() else {
            return Err(Error::ScriptParse(
                "unsupported class element syntax".into(),
            ));
        };

        if method_name == "get" && !is_async && !is_generator {
            let getter_probe = cursor.pos();
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                let Some(getter_name) = cursor.parse_identifier() else {
                    return Err(Error::ScriptParse(
                        "class getter requires a property name".into(),
                    ));
                };
                cursor.skip_ws();
                let params_src = cursor.read_balanced_block(b'(', b')')?;
                if !params_src.trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "class getter must not have parameters".into(),
                    ));
                }
                cursor.skip_ws();

                let method_body_src = cursor.read_balanced_block(b'{', b'}')?;
                let handler = ScriptHandler {
                    params: Vec::new(),
                    stmts: parse_block_statements(&method_body_src)?,
                };
                methods.push(ClassMethodDecl {
                    name: getter_name,
                    handler,
                    is_async: false,
                    is_generator: false,
                    kind: ClassMethodKind::Getter,
                });
                cursor.skip_ws();
                cursor.consume_byte(b';');
                continue;
            }
            cursor.set_pos(getter_probe);
        }

        if method_name == "set" && !is_async && !is_generator {
            let setter_probe = cursor.pos();
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                let Some(setter_name) = cursor.parse_identifier() else {
                    return Err(Error::ScriptParse(
                        "class setter requires a property name".into(),
                    ));
                };
                cursor.skip_ws();
                let params_src = cursor.read_balanced_block(b'(', b')')?;
                let parsed_params =
                    parse_callback_parameter_list(&params_src, 1, "class setter parameters")?;
                if parsed_params.params.len() != 1 || parsed_params.params[0].is_rest {
                    return Err(Error::ScriptParse(
                        "class setter must have exactly one parameter".into(),
                    ));
                }
                cursor.skip_ws();

                let method_body_src = cursor.read_balanced_block(b'{', b'}')?;
                let method_stmts = prepend_callback_param_prologue_stmts(
                    parse_block_statements(&method_body_src)?,
                    &parsed_params.prologue,
                )?;
                let handler = ScriptHandler {
                    params: parsed_params.params,
                    stmts: method_stmts,
                };
                methods.push(ClassMethodDecl {
                    name: setter_name,
                    handler,
                    is_async: false,
                    is_generator: false,
                    kind: ClassMethodKind::Setter,
                });
                cursor.skip_ws();
                cursor.consume_byte(b';');
                continue;
            }
            cursor.set_pos(setter_probe);
        }
        cursor.skip_ws();

        let params_src = cursor.read_balanced_block(b'(', b')')?;
        let parsed_params =
            parse_callback_parameter_list(&params_src, usize::MAX, "class method parameters")?;
        cursor.skip_ws();

        let method_body_src = cursor.read_balanced_block(b'{', b'}')?;
        let method_stmts = prepend_callback_param_prologue_stmts(
            parse_block_statements(&method_body_src)?,
            &parsed_params.prologue,
        )?;
        let handler = ScriptHandler {
            params: parsed_params.params,
            stmts: method_stmts,
        };

        if method_name == "constructor" {
            if is_async || is_generator {
                return Err(Error::ScriptParse(
                    "class constructor cannot be async or generator".into(),
                ));
            }
            if constructor.is_some() {
                return Err(Error::ScriptParse(
                    "class declaration has multiple constructors".into(),
                ));
            }
            constructor = Some(handler);
        } else {
            methods.push(ClassMethodDecl {
                name: method_name,
                handler,
                is_async,
                is_generator,
                kind: ClassMethodKind::Method,
            });
        }

        cursor.skip_ws();
        cursor.consume_byte(b';');
    }

    Ok((constructor, methods))
}

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
        let targets = parse_array_destructure_pattern(name)?;
        let expr = parse_expr(expr_src)?;
        return Ok(Some(Stmt::ArrayDestructureAssign {
            targets,
            expr,
            decl_kind: Some(kind),
        }));
    }
    if name.starts_with('{') && name.ends_with('}') {
        let bindings = parse_object_destructure_pattern(name)?;
        let expr = parse_expr(expr_src)?;
        return Ok(Some(Stmt::ObjectDestructureAssign {
            bindings,
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
        let targets = parse_array_destructure_pattern(lhs)?;
        let expr = parse_expr(rhs)?;
        return Ok(Some(Stmt::ArrayDestructureAssign {
            targets,
            expr,
            decl_kind: None,
        }));
    }
    if lhs.starts_with('{') && lhs.ends_with('}') {
        let bindings = parse_object_destructure_pattern(lhs)?;
        let expr = parse_expr(rhs)?;
        return Ok(Some(Stmt::ObjectDestructureAssign {
            bindings,
            expr,
            decl_kind: None,
        }));
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
