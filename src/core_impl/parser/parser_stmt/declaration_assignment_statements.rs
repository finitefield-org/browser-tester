use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrivateElementDeclKind {
    Value,
    Getter,
    Setter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PrivateElementDeclState {
    is_static: bool,
    has_value: bool,
    has_getter: bool,
    has_setter: bool,
}

fn parse_class_element_name(
    cursor: &mut Cursor<'_>,
    missing_err: &'static str,
) -> Result<(String, bool)> {
    if cursor.consume_byte(b'#') {
        let Some(name) = cursor.parse_identifier() else {
            return Err(Error::ScriptParse(missing_err.into()));
        };
        if name == "constructor" {
            return Err(Error::ScriptParse(
                "private identifier cannot be #constructor".into(),
            ));
        }
        return Ok((name, true));
    }

    let Some(name) = cursor.parse_identifier() else {
        return Err(Error::ScriptParse(missing_err.into()));
    };
    Ok((name, false))
}

fn register_private_decl(
    declared: &mut HashMap<String, PrivateElementDeclState>,
    name: &str,
    is_static: bool,
    kind: PrivateElementDeclKind,
) -> Result<()> {
    let state = declared
        .entry(name.to_string())
        .or_insert(PrivateElementDeclState {
            is_static,
            has_value: false,
            has_getter: false,
            has_setter: false,
        });

    if state.is_static != is_static {
        return Err(Error::ScriptParse(format!(
            "duplicate private identifier '#{name}'"
        )));
    }

    match kind {
        PrivateElementDeclKind::Value => {
            if state.has_value || state.has_getter || state.has_setter {
                return Err(Error::ScriptParse(format!(
                    "duplicate private identifier '#{name}'"
                )));
            }
            state.has_value = true;
        }
        PrivateElementDeclKind::Getter => {
            if state.has_value || state.has_getter {
                return Err(Error::ScriptParse(format!(
                    "duplicate private identifier '#{name}'"
                )));
            }
            state.has_getter = true;
        }
        PrivateElementDeclKind::Setter => {
            if state.has_value || state.has_setter {
                return Err(Error::ScriptParse(format!(
                    "duplicate private identifier '#{name}'"
                )));
            }
            state.has_setter = true;
        }
    }

    Ok(())
}

fn read_class_field_initializer(cursor: &mut Cursor<'_>) -> Result<String> {
    let start = cursor.pos();
    let bytes = cursor.bytes();
    let mut scanner = JsLexScanner::new();
    let mut i = start;
    while i < bytes.len() {
        if scanner.is_top_level() && bytes[i] == b';' {
            break;
        }
        i = scanner.advance(bytes, i);
    }

    let initializer = cursor
        .src
        .get(start..i)
        .ok_or_else(|| Error::ScriptParse("invalid class field initializer".into()))?
        .trim()
        .to_string();
    cursor.set_pos(i);
    Ok(initializer)
}

fn skip_ws_and_comments(src: &[u8], mut i: usize) -> usize {
    loop {
        while i < src.len() && src[i].is_ascii_whitespace() {
            i += 1;
        }
        if i + 1 >= src.len() || src[i] != b'/' {
            break;
        }
        if src[i + 1] == b'/' {
            i += 2;
            while i < src.len() && src[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if src[i + 1] == b'*' {
            i += 2;
            while i + 1 < src.len() && !(src[i] == b'*' && src[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < src.len() {
                i += 2;
            }
            continue;
        }
        break;
    }
    i
}

fn validate_static_block_source(block_src: &str) -> Result<()> {
    let bytes = block_src.as_bytes();
    let mut scanner = JsLexScanner::new();
    let mut i = 0;
    while i < bytes.len() {
        let in_code = matches!(
            scanner.mode,
            JsLexMode::Normal | JsLexMode::TemplateExpr { .. }
        );
        if in_code && (bytes[i] == b'_' || bytes[i] == b'$' || bytes[i].is_ascii_alphabetic()) {
            let mut end = i + 1;
            while end < bytes.len() && is_ident_char(bytes[end]) {
                end += 1;
            }
            let ident = &bytes[i..end];
            if ident == b"arguments" {
                return Err(Error::ScriptParse(
                    "arguments is not allowed in class static initialization block".into(),
                ));
            }
            if ident == b"super" {
                let next = skip_ws_and_comments(bytes, end);
                if bytes.get(next) == Some(&b'(') {
                    return Err(Error::ScriptParse(
                        "super() is not allowed in class static initialization block".into(),
                    ));
                }
            }
        }
        i = scanner.advance(bytes, i);
    }
    Ok(())
}

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

    let (constructor, fields, methods, static_initializers) = parse_class_body(&body_src)?;
    Ok(Some(Stmt::ClassDecl {
        name,
        super_class,
        constructor,
        fields,
        methods,
        static_initializers,
    }))
}

pub(crate) fn parse_class_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if !consume_keyword(&mut cursor, "class") {
        return Ok(None);
    }
    cursor.skip_ws();

    let parsed_name = cursor.parse_identifier();
    if parsed_name.is_some() {
        cursor.skip_ws();
    }

    let (class_name, class_decl_src) = if let Some(name) = parsed_name {
        (name, src.to_string())
    } else {
        let mut temp_index = 0usize;
        let generated_name = loop {
            let candidate = format!("__bt_class_expr_{temp_index}");
            if !src.contains(&candidate) {
                break candidate;
            }
            temp_index += 1;
        };

        let rest = src
            .get(cursor.pos()..)
            .ok_or_else(|| Error::ScriptParse("invalid class expression".into()))?;
        (
            generated_name.clone(),
            format!("class {generated_name} {rest}"),
        )
    };

    let Some(_) = parse_class_decl_stmt(&class_decl_src)? else {
        return Ok(None);
    };

    let lowered = format!("(() => {{ {class_decl_src}; return {class_name}; }})()");
    Ok(Some(parse_expr(&lowered)?))
}

pub(crate) fn parse_class_body(
    body_src: &str,
) -> Result<(
    Option<ScriptHandler>,
    Vec<ClassFieldDecl>,
    Vec<ClassMethodDecl>,
    Vec<ClassStaticInitializerDecl>,
)> {
    let mut cursor = Cursor::new(body_src);
    let mut constructor = None;
    let mut fields = Vec::new();
    let mut methods = Vec::new();
    let mut static_initializers = Vec::new();
    let mut private_decls = HashMap::new();

    while !cursor.eof() {
        cursor.skip_ws();
        while cursor.consume_byte(b';') {
            cursor.skip_ws();
        }
        if cursor.eof() {
            break;
        }

        let mut is_static = false;
        let static_probe = cursor.pos();
        let mut is_static_block = false;
        if consume_keyword(&mut cursor, "static") {
            cursor.skip_ws();
            match cursor.peek() {
                Some(b'(') | Some(b'=') | Some(b';') | Some(b'}') | None => {
                    cursor.set_pos(static_probe);
                }
                Some(b'{') => {
                    is_static = true;
                    is_static_block = true;
                }
                _ => {
                    is_static = true;
                }
            }
        }

        if is_static_block {
            let block_src = cursor.read_balanced_block(b'{', b'}')?;
            validate_static_block_source(&block_src)?;
            static_initializers.push(ClassStaticInitializerDecl::Block(ScriptHandler {
                params: Vec::new(),
                stmts: parse_block_statements(&block_src)?,
            }));
            cursor.skip_ws();
            cursor.consume_byte(b';');
            continue;
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

        let mut computed_name = None;
        let (method_name, method_name_is_private) = if cursor.peek() == Some(b'[') {
            let computed_src = cursor.read_balanced_block(b'[', b']')?;
            let computed_src = computed_src.trim();
            if computed_src.is_empty() {
                return Err(Error::ScriptParse(
                    "computed class element name cannot be empty".into(),
                ));
            }
            computed_name = Some(parse_expr(computed_src)?);
            (String::new(), false)
        } else {
            parse_class_element_name(&mut cursor, "unsupported class element syntax")?
        };

        if computed_name.is_none()
            && is_static
            && !method_name_is_private
            && method_name == "prototype"
        {
            return Err(Error::ScriptParse(
                "static class property name cannot be prototype".into(),
            ));
        }

        if computed_name.is_none()
            && method_name == "get"
            && !method_name_is_private
            && !is_async
            && !is_generator
        {
            let getter_probe = cursor.pos();
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                let (getter_name, getter_is_private) =
                    parse_class_element_name(&mut cursor, "class getter requires a property name")?;
                if getter_name == "constructor" && !getter_is_private {
                    return Err(Error::ScriptParse(
                        "class constructor cannot be getter or setter".into(),
                    ));
                }
                if is_static && !getter_is_private && getter_name == "prototype" {
                    return Err(Error::ScriptParse(
                        "static class property name cannot be prototype".into(),
                    ));
                }
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
                    is_private: getter_is_private,
                    is_static,
                    handler,
                    is_async: false,
                    is_generator: false,
                    kind: ClassMethodKind::Getter,
                });
                if getter_is_private {
                    register_private_decl(
                        &mut private_decls,
                        &methods.last().expect("pushed getter").name,
                        is_static,
                        PrivateElementDeclKind::Getter,
                    )?;
                }
                cursor.skip_ws();
                cursor.consume_byte(b';');
                continue;
            }
            cursor.set_pos(getter_probe);
        }

        if computed_name.is_none()
            && method_name == "set"
            && !method_name_is_private
            && !is_async
            && !is_generator
        {
            let setter_probe = cursor.pos();
            cursor.skip_ws();
            if cursor.peek() != Some(b'(') {
                let (setter_name, setter_is_private) =
                    parse_class_element_name(&mut cursor, "class setter requires a property name")?;
                if setter_name == "constructor" && !setter_is_private {
                    return Err(Error::ScriptParse(
                        "class constructor cannot be getter or setter".into(),
                    ));
                }
                if is_static && !setter_is_private && setter_name == "prototype" {
                    return Err(Error::ScriptParse(
                        "static class property name cannot be prototype".into(),
                    ));
                }
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
                    is_private: setter_is_private,
                    is_static,
                    handler,
                    is_async: false,
                    is_generator: false,
                    kind: ClassMethodKind::Setter,
                });
                if setter_is_private {
                    register_private_decl(
                        &mut private_decls,
                        &methods.last().expect("pushed setter").name,
                        is_static,
                        PrivateElementDeclKind::Setter,
                    )?;
                }
                cursor.skip_ws();
                cursor.consume_byte(b';');
                continue;
            }
            cursor.set_pos(setter_probe);
        }
        cursor.skip_ws();

        if cursor.peek() != Some(b'(') {
            if is_async || is_generator {
                return Err(Error::ScriptParse(
                    "class field cannot be async or generator".into(),
                ));
            }
            let initializer = if cursor.consume_byte(b'=') {
                cursor.skip_ws();
                let initializer_src = read_class_field_initializer(&mut cursor)?;
                if initializer_src.is_empty() {
                    return Err(Error::ScriptParse(
                        "class field initializer cannot be empty".into(),
                    ));
                }
                Some(parse_expr(&initializer_src)?)
            } else {
                None
            };
            if computed_name.is_none() && method_name == "constructor" && !method_name_is_private {
                return Err(Error::ScriptParse(
                    "class field name cannot be constructor".into(),
                ));
            }
            if method_name_is_private {
                register_private_decl(
                    &mut private_decls,
                    &method_name,
                    is_static,
                    PrivateElementDeclKind::Value,
                )?;
            }
            fields.push(ClassFieldDecl {
                name: method_name,
                computed_name,
                is_private: method_name_is_private,
                is_static,
                initializer,
            });
            if is_static {
                static_initializers.push(ClassStaticInitializerDecl::Field(fields.len() - 1));
            }
            cursor.skip_ws();
            cursor.consume_byte(b';');
            continue;
        }

        if computed_name.is_some() {
            return Err(Error::ScriptParse(
                "computed class methods are not supported".into(),
            ));
        }

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

        if method_name == "constructor" && !method_name_is_private && !is_static {
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
            if method_name_is_private {
                register_private_decl(
                    &mut private_decls,
                    &method_name,
                    is_static,
                    PrivateElementDeclKind::Value,
                )?;
            }
            methods.push(ClassMethodDecl {
                name: method_name,
                is_private: method_name_is_private,
                is_static,
                handler,
                is_async,
                is_generator,
                kind: ClassMethodKind::Method,
            });
        }

        cursor.skip_ws();
        cursor.consume_byte(b';');
    }

    Ok((constructor, fields, methods, static_initializers))
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
        let pattern = parse_array_destructure_assignment_pattern(lhs)?;
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
    let had_trailing_empty = items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty());
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
    let had_trailing_empty = items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty());
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
            let (target, default) =
                if let Some((eq_pos, op_len)) = find_top_level_assignment(target_src) {
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
