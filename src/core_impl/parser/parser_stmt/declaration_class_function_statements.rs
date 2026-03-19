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
