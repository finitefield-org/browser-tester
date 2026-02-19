fn append_dom_query_member_path(target: &DomQuery, member: &str) -> Option<DomQuery> {
    match target {
        DomQuery::Var(base) => Some(DomQuery::VarPath {
            base: base.clone(),
            path: vec![member.to_string()],
        }),
        DomQuery::VarPath { base, path } => {
            let mut next_path = path.clone();
            next_path.push(member.to_string());
            Some(DomQuery::VarPath {
                base: base.clone(),
                path: next_path,
            })
        }
        _ => None,
    }
}

fn is_dom_target_chain_stop(ident: &str) -> bool {
    matches!(
        ident,
        "activeElement"
            | "addEventListener"
            | "after"
            | "append"
            | "appendChild"
            | "attributionSrc"
            | "before"
            | "blur"
            | "charset"
            | "checked"
            | "classList"
            | "className"
            | "click"
            | "close"
            | "closest"
            | "closedBy"
            | "closedby"
            | "coords"
            | "dataset"
            | "download"
            | "disabled"
            | "dispatchEvent"
            | "elements"
            | "focus"
            | "forEach"
            | "getAttribute"
            | "hash"
            | "hasAttribute"
            | "host"
            | "hostname"
            | "href"
            | "hreflang"
            | "hidden"
            | "id"
            | "interestForElement"
            | "innerHTML"
            | "innerText"
            | "insertAdjacentElement"
            | "insertAdjacentHTML"
            | "insertAdjacentText"
            | "insertBefore"
            | "length"
            | "matches"
            | "name"
            | "offsetHeight"
            | "offsetLeft"
            | "offsetTop"
            | "offsetWidth"
            | "open"
            | "origin"
            | "password"
            | "pathname"
            | "ping"
            | "port"
            | "protocol"
            | "querySelector"
            | "querySelectorAll"
            | "prepend"
            | "readOnly"
            | "readonly"
            | "referrerPolicy"
            | "rel"
            | "relList"
            | "remove"
            | "removeAttribute"
            | "removeChild"
            | "removeEventListener"
            | "requestClose"
            | "returnValue"
            | "replaceWith"
            | "required"
            | "reset"
            | "rev"
            | "search"
            | "shape"
            | "scrollHeight"
            | "scrollIntoView"
            | "scrollLeft"
            | "scrollTop"
            | "scrollWidth"
            | "setAttribute"
            | "show"
            | "showModal"
            | "style"
            | "submit"
            | "target"
            | "text"
            | "textContent"
            | "type"
            | "username"
            | "value"
    )
}

fn parse_element_target(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    cursor.skip_ws();
    let start = cursor.pos();
    let mut target = if let Ok(target) = parse_form_elements_item_target(cursor) {
        target
    } else {
        cursor.set_pos(start);
        parse_document_or_var_target(cursor)?
    };

    loop {
        cursor.skip_ws();
        let dot_pos = cursor.pos();
        if !cursor.consume_byte(b'.') {
            break;
        }

        cursor.skip_ws();
        let method = match cursor.parse_identifier() {
            Some(method) => method,
            None => {
                cursor.set_pos(dot_pos);
                break;
            }
        };

        match method.as_str() {
            "body" if matches!(target, DomQuery::DocumentRoot) => {
                target = DomQuery::DocumentBody;
            }
            "head" if matches!(target, DomQuery::DocumentRoot) => {
                target = DomQuery::DocumentHead;
            }
            "documentElement" if matches!(target, DomQuery::DocumentRoot) => {
                target = DomQuery::DocumentElement;
            }
            "querySelector" => {
                cursor.skip_ws();
                if cursor.peek() != Some(b'(') {
                    cursor.set_pos(dot_pos);
                    break;
                }
                cursor.expect_byte(b'(')?;
                cursor.skip_ws();
                let selector = cursor.parse_string_literal()?;
                cursor.skip_ws();
                cursor.expect_byte(b')')?;
                cursor.skip_ws();
                target = DomQuery::QuerySelector {
                    target: Box::new(target),
                    selector,
                };
            }
            "querySelectorAll" => {
                cursor.skip_ws();
                if cursor.peek() != Some(b'(') {
                    cursor.set_pos(dot_pos);
                    break;
                }
                cursor.expect_byte(b'(')?;
                cursor.skip_ws();
                let selector = cursor.parse_string_literal()?;
                cursor.skip_ws();
                cursor.expect_byte(b')')?;
                cursor.skip_ws();
                target = DomQuery::QuerySelectorAll {
                    target: Box::new(target),
                    selector,
                };
            }
            _ => {
                if is_dom_target_chain_stop(&method) {
                    cursor.set_pos(dot_pos);
                    break;
                }
                if let Some(next_target) = append_dom_query_member_path(&target, &method) {
                    target = next_target;
                    continue;
                }
                cursor.set_pos(dot_pos);
                break;
            }
        }
    }

    loop {
        cursor.skip_ws();
        let index_pos = cursor.pos();
        if !cursor.consume_byte(b'[') {
            break;
        }

        cursor.skip_ws();
        let index_src = match cursor.read_until_byte(b']') {
            Ok(index_src) => index_src,
            Err(_) => {
                cursor.set_pos(index_pos);
                break;
            }
        };
        cursor.skip_ws();
        cursor.expect_byte(b']')?;
        let index = parse_dom_query_index(&index_src)?;
        target = match target {
            DomQuery::BySelectorAll { selector } => {
                DomQuery::BySelectorAllIndex { selector, index }
            }
            DomQuery::QuerySelectorAll { target, selector } => DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            },
            _ => DomQuery::Index {
                target: Box::new(target),
                index,
            },
        };
        cursor.skip_ws();
    }
    Ok(target)
}

fn parse_document_or_var_target(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    let start = cursor.pos();
    if let Ok(target) = parse_document_element_call(cursor) {
        return Ok(target);
    }
    cursor.set_pos(start);
    if cursor.consume_ascii("document") {
        return Ok(DomQuery::DocumentRoot);
    }
    cursor.set_pos(start);
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            if cursor.consume_ascii("document") {
                cursor.skip_ws();
            } else {
                cursor.set_pos(start + "window".len());
            }
        }
        return Ok(DomQuery::DocumentRoot);
    }
    cursor.set_pos(start);
    if let Some(name) = cursor.parse_identifier() {
        return Ok(DomQuery::Var(name));
    }
    Err(Error::ScriptParse(format!(
        "expected element target at {}",
        start
    )))
}

fn parse_form_elements_item_target(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    let form = parse_form_elements_base(cursor)?;
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    cursor.expect_ascii("elements")?;
    cursor.skip_ws();
    cursor.expect_byte(b'[')?;
    cursor.skip_ws();
    let index_src = cursor.read_until_byte(b']')?;
    cursor.skip_ws();
    cursor.expect_byte(b']')?;
    let index = parse_dom_query_index(&index_src)?;
    Ok(DomQuery::FormElementsIndex {
        form: Box::new(form),
        index,
    })
}

fn parse_dom_query_index(src: &str) -> Result<DomIndex> {
    let src = strip_js_comments(src).trim().to_string();
    if src.is_empty() {
        return Err(Error::ScriptParse("empty index".into()));
    }

    let expr = parse_expr(&src)?;
    if let Expr::Number(index) = expr {
        return usize::try_from(index)
            .map(DomIndex::Static)
            .map_err(|_| Error::ScriptParse(format!("invalid index: {src}")));
    }

    Ok(DomIndex::Dynamic(src))
}

fn parse_form_elements_base(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    let start = cursor.pos();
    if let Ok(target) = parse_document_element_call(cursor) {
        return Ok(target);
    }
    cursor.set_pos(start);
    if let Some(name) = cursor.parse_identifier() {
        return Ok(DomQuery::Var(name));
    }
    Err(Error::ScriptParse(format!(
        "expected form target at {}",
        start
    )))
}

fn parse_document_element_call(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        cursor.expect_byte(b'.')?;
        cursor.skip_ws();
    }
    cursor.expect_ascii("document")?;
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    let method = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse("expected document method call".into()))?;
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let arg = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();

    match method.as_str() {
        "getElementById" => Ok(DomQuery::ById(arg)),
        "querySelector" => Ok(DomQuery::BySelector(arg)),
        "querySelectorAll" => Ok(DomQuery::BySelectorAll { selector: arg }),
        "getElementsByTagName" => Ok(DomQuery::BySelectorAll {
            selector: normalize_get_elements_by_tag_name(&arg)?,
        }),
        "getElementsByClassName" => Ok(DomQuery::BySelectorAll {
            selector: normalize_get_elements_by_class_name(&arg)?,
        }),
        "getElementsByName" => Ok(DomQuery::BySelectorAll {
            selector: normalize_get_elements_by_name(&arg)?,
        }),
        _ => Err(Error::ScriptParse(format!(
            "unsupported document method: {}",
            method
        ))),
    }
}

fn normalize_get_elements_by_tag_name(tag_name: &str) -> Result<String> {
    let tag_name = tag_name.trim();
    if tag_name.is_empty() {
        return Err(Error::ScriptParse(
            "getElementsByTagName requires a tag name".into(),
        ));
    }
    if tag_name == "*" {
        return Ok("*".into());
    }
    Ok(tag_name.to_ascii_lowercase())
}

fn normalize_get_elements_by_class_name(class_names: &str) -> Result<String> {
    let mut selector = String::new();
    let classes: Vec<&str> = class_names
        .split_whitespace()
        .map(str::trim)
        .filter(|class_name| !class_name.is_empty())
        .collect();

    if classes.is_empty() {
        return Err(Error::ScriptParse(
            "getElementsByClassName requires at least one class name".into(),
        ));
    }

    for class_name in classes {
        selector.push('.');
        selector.push_str(class_name);
    }
    Ok(selector)
}

fn normalize_get_elements_by_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::ScriptParse(
            "getElementsByName requires a name value".into(),
        ));
    }
    let escaped = name.replace('\\', "\\\\").replace('\'', "\\'");
    Ok(format!("[name='{}']", escaped))
}

struct ParsedCallbackParams {
    params: Vec<FunctionParam>,
    prologue: Vec<String>,
}

fn next_callback_temp_name(params: &[FunctionParam], seed: usize) -> String {
    let mut suffix = seed;
    loop {
        let candidate = format!("__bt_callback_arg_{suffix}");
        if params.iter().all(|param| param.name != candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

fn format_array_destructure_pattern(pattern: &[Option<String>]) -> String {
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

fn inject_callback_param_prologue(
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

fn prepend_callback_param_prologue_stmts(
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
    for (index, raw) in parts.into_iter().enumerate() {
        let param = raw.trim();
        if param.is_empty() {
            return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
        }

        if let Some((eq_pos, op_len)) = find_top_level_assignment(param) {
            if op_len != 1 {
                return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
            }
            let name = param[..eq_pos].trim();
            let default_src = param[eq_pos + op_len..].trim();
            if !is_ident(name) || default_src.is_empty() {
                return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
            }
            params.push(FunctionParam {
                name: name.to_string(),
                default: Some(parse_expr(default_src)?),
            });
            bound_names.push(name.to_string());
            continue;
        }

        if is_ident(param) {
            params.push(FunctionParam {
                name: param.to_string(),
                default: None,
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
            });
            continue;
        }

        return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
    }

    Ok(ParsedCallbackParams { params, prologue })
}

fn parse_arrow_or_block_body(cursor: &mut Cursor<'_>) -> Result<(String, bool)> {
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
        let expr_src = raw.trim();
        if expr_src.is_empty() {
            break;
        }

        let stripped = strip_js_comments(expr_src);
        let stripped = stripped.trim();
        if !stripped.is_empty() {
            if parse_expr(stripped).is_ok() {
                cursor.set_pos(cursor.i + expr_src.len());
                return Ok((stripped.to_string(), true));
            }
        }

        end -= 1;
    }

    Err(Error::ScriptParse("expected callback body".into()))
}

fn try_consume_async_function_prefix(cursor: &mut Cursor<'_>) -> bool {
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

fn parse_callback(
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
            }],
            prologue: Vec::new(),
        }
    };

    cursor.skip_ws();
    cursor.expect_ascii("=>")?;
    let (body, concise_body) = parse_arrow_or_block_body(cursor)?;
    let (body, concise_body) =
        inject_callback_param_prologue(body, concise_body, &parsed_params.prologue);
    Ok((parsed_params.params, body, concise_body))
}

fn parse_timer_callback(timer_name: &str, src: &str) -> Result<TimerCallback> {
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

pub(super) fn parse_block_statements(body: &str) -> Result<Vec<Stmt>> {
    let sanitized = strip_js_comments(body);
    let raw_stmts = split_top_level_statements(sanitized.as_str());
    let mut stmts = Vec::new();

    for raw in raw_stmts {
        let stmt = raw.trim();
        if stmt.is_empty() {
            continue;
        }

        if let Some(else_branch) = parse_else_fragment(stmt)? {
            if let Some(Stmt::If { else_stmts, .. }) = stmts.last_mut() {
                if else_stmts.is_empty() {
                    *else_stmts = else_branch;
                    continue;
                }
                return Err(Error::ScriptParse(format!(
                    "duplicate else branch in: {stmt}"
                )));
            }
            return Err(Error::ScriptParse(format!(
                "unexpected else without matching if: {stmt}"
            )));
        }

        let parsed = parse_single_statement(stmt)?;
        stmts.push(parsed);
    }

    Ok(stmts)
}

fn parse_single_statement(stmt: &str) -> Result<Stmt> {
    let stmt = stmt.trim();

    if let Some(parsed) = parse_if_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_do_while_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_while_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_for_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_try_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_return_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_throw_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_break_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_continue_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_query_selector_all_foreach_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_array_for_each_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_function_decl_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_var_decl(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_destructure_assign(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_var_assign(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_update_stmt(stmt) {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_form_data_append_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_dom_method_call_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_dom_assignment(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_object_assign(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_set_attribute_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_remove_attribute_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_class_list_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_insert_adjacent_element_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_insert_adjacent_text_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_insert_adjacent_html_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_set_timeout_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_set_interval_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_queue_microtask_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_clear_timeout_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_node_tree_mutation_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_node_remove_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_listener_mutation_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_dispatch_event_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_event_call_stmt(stmt) {
        return Ok(parsed);
    }

    let expr = parse_expr(stmt)?;
    Ok(Stmt::Expr(expr))
}

fn parse_else_fragment(stmt: &str) -> Result<Option<Vec<Stmt>>> {
    let trimmed = stmt.trim_start();
    let Some(rest) = strip_else_prefix(trimmed) else {
        return Ok(None);
    };
    let branch = parse_if_branch(rest.trim())?;
    Ok(Some(branch))
}

fn strip_else_prefix(src: &str) -> Option<&str> {
    if !src.starts_with("else") {
        return None;
    }
    let bytes = src.as_bytes();
    let after = 4;
    if after < bytes.len() && is_ident_char(bytes[after]) {
        return None;
    }
    Some(&src[after..])
}

fn parse_if_branch(src: &str) -> Result<Vec<Stmt>> {
    let src = src.trim();
    if src.is_empty() {
        return Err(Error::ScriptParse("empty if branch".into()));
    }

    if src.starts_with('{') {
        let mut cursor = Cursor::new(src);
        let body = cursor.read_balanced_block(b'{', b'}')?;
        cursor.skip_ws();
        cursor.consume_byte(b';');
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported trailing tokens in branch: {src}"
            )));
        }
        return parse_block_statements(&body);
    }

    let single = trim_optional_trailing_semicolon(src);
    if single.is_empty() {
        return Err(Error::ScriptParse("empty single statement branch".into()));
    }
    Ok(vec![parse_single_statement(single)?])
}

fn trim_optional_trailing_semicolon(src: &str) -> &str {
    let mut trimmed = src.trim_end();
    if let Some(without) = trimmed.strip_suffix(';') {
        trimmed = without.trim_end();
    }
    trimmed
}

fn find_top_level_else_keyword(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                b'e' if paren == 0 && bracket == 0 && brace == 0 => {
                    if i + 4 <= bytes.len()
                        && &bytes[i..i + 4] == b"else"
                        && (i == 0 || !is_ident_char(bytes[i - 1]))
                        && (i + 4 == bytes.len() || !is_ident_char(bytes[i + 4]))
                    {
                        return Some(i);
                    }
                }
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }
        i += 1;
    }

    None
}

fn is_ident_char(b: u8) -> bool {
    b == b'_' || b == b'$' || b.is_ascii_alphanumeric()
}

fn parse_if_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    if !cursor.consume_ascii("if") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }

    cursor.skip_ws();
    let cond_src = cursor.read_balanced_block(b'(', b')')?;
    let cond = parse_expr(cond_src.trim()).map_err(|err| {
        Error::ScriptParse(format!(
            "if condition parse failed: cond={:?} stmt={:?} err={err:?}",
            cond_src.trim(),
            stmt
        ))
    })?;

    let tail = cursor.src[cursor.i..].trim();
    if tail.is_empty() {
        return Err(Error::ScriptParse(format!(
            "if statement has no branch: {stmt}"
        )));
    }

    let (then_raw, else_raw) = if tail.starts_with('{') {
        let mut branch_cursor = Cursor::new(tail);
        let _ = branch_cursor.read_balanced_block(b'{', b'}')?;
        let split = branch_cursor.pos();
        let then_raw = tail
            .get(..split)
            .ok_or_else(|| Error::ScriptParse("invalid if branch slice".into()))?;
        let rest = tail
            .get(split..)
            .ok_or_else(|| Error::ScriptParse("invalid if remainder slice".into()))?
            .trim();

        if rest.is_empty() {
            (then_raw, None)
        } else if let Some(after_else) = strip_else_prefix(rest) {
            (then_raw, Some(after_else))
        } else {
            return Err(Error::ScriptParse(format!(
                "unsupported tokens after if block: {rest}"
            )));
        }
    } else {
        if let Some(pos) = find_top_level_else_keyword(tail) {
            let then_raw = tail
                .get(..pos)
                .ok_or_else(|| Error::ScriptParse("invalid then branch".into()))?;
            let else_raw = tail
                .get(pos + 4..)
                .ok_or_else(|| Error::ScriptParse("invalid else branch".into()))?;
            (then_raw, Some(else_raw))
        } else {
            (tail, None)
        }
    };

    let then_stmts = parse_if_branch(then_raw)?;
    let else_stmts = if let Some(raw) = else_raw {
        parse_if_branch(raw)?
    } else {
        Vec::new()
    };

    Ok(Some(Stmt::If {
        cond,
        then_stmts,
        else_stmts,
    }))
}

fn parse_while_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    if !cursor.consume_ascii("while") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }

    cursor.skip_ws();
    let cond_src = cursor.read_balanced_block(b'(', b')')?;
    let cond = parse_expr(cond_src.trim())?;

    cursor.skip_ws();
    let body_src = cursor.read_balanced_block(b'{', b'}')?;
    let body = parse_block_statements(&body_src)?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported while statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::While { cond, body }))
}

fn parse_do_while_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    if !cursor.consume_ascii("do") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }

    cursor.skip_ws();
    let body_src = cursor.read_balanced_block(b'{', b'}')?;
    let body = parse_block_statements(&body_src)?;

    cursor.skip_ws();
    if !cursor.consume_ascii("while") {
        return Err(Error::ScriptParse(format!(
            "unsupported do statement: {stmt}"
        )));
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Err(Error::ScriptParse(format!(
                "unsupported do statement: {stmt}"
            )));
        }
    }

    cursor.skip_ws();
    let cond_src = cursor.read_balanced_block(b'(', b')')?;
    let cond = parse_expr(cond_src.trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported do while statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::DoWhile { cond, body }))
}

fn consume_keyword(cursor: &mut Cursor<'_>, keyword: &str) -> bool {
    let start = cursor.pos();
    if !cursor.consume_ascii(keyword) {
        return false;
    }
    if cursor.peek().is_some_and(is_ident_char) {
        cursor.set_pos(start);
        return false;
    }
    true
}

fn parse_catch_binding(src: &str) -> Result<CatchBinding> {
    let src = src.trim();
    if src.is_empty() {
        return Err(Error::ScriptParse("catch binding cannot be empty".into()));
    }
    if is_ident(src) {
        return Ok(CatchBinding::Identifier(src.to_string()));
    }
    if src.starts_with('[') && src.ends_with(']') {
        let pattern = parse_array_destructure_pattern(src)?;
        return Ok(CatchBinding::ArrayPattern(pattern));
    }
    if src.starts_with('{') && src.ends_with('}') {
        let pattern = parse_object_destructure_pattern(src)?;
        return Ok(CatchBinding::ObjectPattern(pattern));
    }
    Err(Error::ScriptParse(format!(
        "unsupported catch binding pattern: {src}"
    )))
}

fn parse_try_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !consume_keyword(&mut cursor, "try") {
        return Ok(None);
    }

    cursor.skip_ws();
    let try_src = cursor.read_balanced_block(b'{', b'}')?;
    let try_stmts = parse_block_statements(&try_src)?;

    cursor.skip_ws();
    let mut catch_binding = None;
    let mut catch_stmts = None;
    let mut finally_stmts = None;

    if consume_keyword(&mut cursor, "catch") {
        cursor.skip_ws();
        if cursor.peek() == Some(b'(') {
            let binding_src = cursor.read_balanced_block(b'(', b')')?;
            catch_binding = Some(parse_catch_binding(binding_src.trim())?);
            cursor.skip_ws();
        }
        let catch_src = cursor.read_balanced_block(b'{', b'}')?;
        catch_stmts = Some(parse_block_statements(&catch_src)?);
        cursor.skip_ws();
    }

    if consume_keyword(&mut cursor, "finally") {
        cursor.skip_ws();
        let finally_src = cursor.read_balanced_block(b'{', b'}')?;
        finally_stmts = Some(parse_block_statements(&finally_src)?);
        cursor.skip_ws();
    }

    if catch_stmts.is_none() && finally_stmts.is_none() {
        return Err(Error::ScriptParse(format!(
            "try statement requires catch or finally: {stmt}"
        )));
    }

    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported try statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::Try {
        try_stmts,
        catch_binding,
        catch_stmts,
        finally_stmts,
    }))
}

fn parse_throw_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !consume_keyword(&mut cursor, "throw") {
        return Ok(None);
    }

    cursor.skip_ws();
    if cursor.eof() {
        return Err(Error::ScriptParse(
            "throw statement requires an operand".into(),
        ));
    }

    let expr_src = cursor.src.get(cursor.i..).unwrap_or_default().trim();
    let expr_src = expr_src.strip_suffix(';').unwrap_or(expr_src).trim();
    if expr_src.is_empty() {
        return Err(Error::ScriptParse(
            "throw statement requires an operand".into(),
        ));
    }
    let value = parse_expr(expr_src)?;
    Ok(Some(Stmt::Throw { value }))
}

fn parse_return_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !cursor.consume_ascii("return") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }

    cursor.skip_ws();
    if cursor.eof() {
        return Ok(Some(Stmt::Return { value: None }));
    }

    let expr_src = cursor.src.get(cursor.i..).unwrap_or_default().trim();
    let expr_src = expr_src.strip_suffix(';').unwrap_or(expr_src).trim();
    if expr_src.is_empty() {
        return Ok(Some(Stmt::Return { value: None }));
    }
    let value = parse_expr(expr_src)?;
    Ok(Some(Stmt::Return { value: Some(value) }))
}

fn parse_break_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !cursor.consume_ascii("break") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported break statement: {stmt}"
        )));
    }
    Ok(Some(Stmt::Break))
}

fn parse_continue_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !cursor.consume_ascii("continue") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported continue statement: {stmt}"
        )));
    }
    Ok(Some(Stmt::Continue))
}

fn parse_for_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    if !cursor.consume_ascii("for") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }

    cursor.skip_ws();
    let header_src = cursor.read_balanced_block(b'(', b')')?;
    if let Some((kind, item_var, iterable_src)) = parse_for_in_of_stmt(&header_src)? {
        let iterable = parse_expr(iterable_src.trim())?;
        cursor.skip_ws();
        let body_src = cursor.read_balanced_block(b'{', b'}')?;
        let body = parse_block_statements(&body_src)?;

        cursor.skip_ws();
        cursor.consume_byte(b';');
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported for statement tail: {stmt}"
            )));
        }

        let stmt = match kind {
            ForInOfKind::In => Stmt::ForIn {
                item_var,
                iterable,
                body,
            },
            ForInOfKind::Of => Stmt::ForOf {
                item_var,
                iterable,
                body,
            },
        };
        return Ok(Some(stmt));
    }

    let header_parts = split_top_level_by_char(header_src.trim(), b';');
    if header_parts.len() != 3 {
        return Err(Error::ScriptParse(format!(
            "unsupported for statement: {stmt}"
        )));
    }

    let init = parse_for_clause_stmt(header_parts[0])?;
    let cond = if header_parts[1].trim().is_empty() {
        None
    } else {
        Some(parse_expr(header_parts[1].trim())?)
    };
    let post = parse_for_clause_stmt(header_parts[2])?;

    cursor.skip_ws();
    let body_src = cursor.read_balanced_block(b'{', b'}')?;
    let body = parse_block_statements(&body_src)?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported for statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::For {
        init,
        cond,
        post,
        body,
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ForInOfKind {
    In,
    Of,
}

fn parse_for_in_of_stmt(header: &str) -> Result<Option<(ForInOfKind, String, &str)>> {
    let header = header.trim();
    if header.is_empty() {
        return Ok(None);
    }

    let mut found = None;
    for (kind, keyword) in [(ForInOfKind::In, "in"), (ForInOfKind::Of, "of")] {
        if let Some(pos) = find_top_level_in_of_keyword(header, keyword)? {
            found = Some((kind, pos, keyword));
            break;
        }
    }

    let Some((kind, pos, keyword)) = found else {
        return Ok(None);
    };

    let left = header[..pos].trim();
    let right = header[pos + keyword.len()..].trim();
    if left.is_empty() || right.is_empty() {
        return Err(Error::ScriptParse(format!(
            "unsupported for statement: {header}"
        )));
    }

    let item_var = parse_for_in_of_var(left)?;
    Ok(Some((kind, item_var, right)))
}

fn find_top_level_in_of_keyword(src: &str, keyword: &str) -> Result<Option<usize>> {
    let bytes = src.as_bytes();
    let mut state = 0u8;
    let mut i = 0usize;
    let mut paren = 0isize;
    let mut bracket = 0isize;
    let mut brace = 0isize;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            0 => match b {
                b'\'' => state = 1,
                b'"' => state = 2,
                b'`' => state = 3,
                b'(' => paren += 1,
                b')' => paren -= 1,
                b'[' => bracket += 1,
                b']' => bracket -= 1,
                b'{' => brace += 1,
                b'}' => brace -= 1,
                _ => {
                    if paren == 0 && bracket == 0 && brace == 0 {
                        if i + keyword.len() <= bytes.len() && &src[i..i + keyword.len()] == keyword
                        {
                            let prev_ok = i == 0
                                || !is_ident_char(
                                    src.as_bytes()
                                        .get(i.wrapping_sub(1))
                                        .copied()
                                        .unwrap_or_default(),
                                );
                            let next = src.as_bytes().get(i + keyword.len()).copied();
                            let next_ok = next.is_none() || !is_ident_char(next.unwrap());
                            if prev_ok && next_ok {
                                return Ok(Some(i));
                            }
                        }
                    }
                }
            },
            1 => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = 0;
                }
            }
            2 => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = 0;
                }
            }
            3 => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = 0;
                }
            }
            _ => state = 0,
        }
        i += 1;
    }

    Ok(None)
}

fn parse_for_in_of_var(raw: &str) -> Result<String> {
    let mut cursor = Cursor::new(raw);
    cursor.skip_ws();
    let first = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse(format!("invalid for statement variable: {raw}")))?;

    let name = if matches!(first.as_str(), "let" | "const" | "var") {
        cursor.skip_ws();
        let name = cursor
            .parse_identifier()
            .ok_or_else(|| Error::ScriptParse(format!("invalid for statement variable: {raw}")))?;
        name
    } else {
        first
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "invalid for statement declaration: {raw}"
        )));
    }
    if !is_ident(&name) {
        return Err(Error::ScriptParse(format!(
            "invalid for statement variable: {raw}"
        )));
    }
    Ok(name)
}

fn parse_for_clause_stmt(src: &str) -> Result<Option<Box<Stmt>>> {
    let src = src.trim();
    if src.is_empty() {
        return Ok(None);
    }

    if let Some(parsed) = parse_var_decl(src)? {
        return Ok(Some(Box::new(parsed)));
    }

    if let Some(parsed) = parse_var_assign(src)? {
        return Ok(Some(Box::new(parsed)));
    }

    if let Some(parsed) = parse_for_update_stmt(src) {
        return Ok(Some(Box::new(parsed)));
    }

    let expr = parse_expr(src)
        .map_err(|_| Error::ScriptParse(format!("unsupported for-loop clause: {src}")))?;
    Ok(Some(Box::new(Stmt::Expr(expr))))
}

fn parse_for_update_stmt(src: &str) -> Option<Stmt> {
    parse_update_stmt(src)
}

fn parse_update_stmt(stmt: &str) -> Option<Stmt> {
    let src = stmt.trim();

    if let Some(name) = src.strip_prefix("++") {
        let name = name.trim();
        if is_ident(name) {
            return Some(Stmt::VarUpdate {
                name: name.to_string(),
                delta: 1,
            });
        }
    }

    if let Some(name) = src.strip_prefix("--") {
        let name = name.trim();
        if is_ident(name) {
            return Some(Stmt::VarUpdate {
                name: name.to_string(),
                delta: -1,
            });
        }
    }

    if let Some(name) = src.strip_suffix("++") {
        let name = name.trim();
        if is_ident(name) {
            return Some(Stmt::VarUpdate {
                name: name.to_string(),
                delta: 1,
            });
        }
    }

    if let Some(name) = src.strip_suffix("--") {
        let name = name.trim();
        if is_ident(name) {
            return Some(Stmt::VarUpdate {
                name: name.to_string(),
                delta: -1,
            });
        }
    }

    None
}

fn split_top_level_statements(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;
    let mut brace_open_stack = Vec::new();

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
        Regex { in_class: bool },
    }
    let mut state = StrState::None;
    let mut previous_significant = None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => {
                if b.is_ascii_whitespace() {
                    i += 1;
                    continue;
                }
                match b {
                    b'\'' => state = StrState::Single,
                    b'"' => state = StrState::Double,
                    b'`' => state = StrState::Backtick,
                    b'/' => {
                        if can_start_regex_literal(previous_significant) {
                            state = StrState::Regex { in_class: false };
                        } else {
                            previous_significant = Some(b);
                        }
                    }
                    b'(' => {
                        paren += 1;
                        previous_significant = Some(b);
                    }
                    b')' => {
                        paren = paren.saturating_sub(1);
                        previous_significant = Some(b);
                    }
                    b'[' => {
                        bracket += 1;
                        previous_significant = Some(b);
                    }
                    b']' => {
                        bracket = bracket.saturating_sub(1);
                        previous_significant = Some(b);
                    }
                    b'{' => {
                        brace += 1;
                        brace_open_stack.push(i);
                        previous_significant = Some(b);
                    }
                    b'}' => {
                        brace = brace.saturating_sub(1);
                        let block_open = brace_open_stack.pop();
                        if paren == 0 && bracket == 0 && brace == 0 {
                            let tail = body.get(i + 1..).unwrap_or_default();
                            if should_split_after_closing_brace(body, block_open, tail) {
                                if let Some(part) = body.get(start..=i) {
                                    out.push(part.to_string());
                                }
                                start = i + 1;
                            }
                        }
                        previous_significant = Some(b);
                    }
                    b';' => {
                        if paren == 0 && bracket == 0 && brace == 0 {
                            if let Some(part) = body.get(start..i) {
                                out.push(part.to_string());
                            }
                            start = i + 1;
                        }
                        previous_significant = Some(b);
                    }
                    _ => {
                        previous_significant = Some(b);
                    }
                }
            }
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                    previous_significant = Some(b'\'');
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                    previous_significant = Some(b'"');
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                    previous_significant = Some(b'`');
                }
            }
            StrState::Regex { mut in_class } => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'[' {
                    in_class = true;
                    state = StrState::Regex { in_class };
                } else if b == b']' && in_class {
                    in_class = false;
                    state = StrState::Regex { in_class };
                } else if b == b'/' && !in_class {
                    state = StrState::None;
                    previous_significant = Some(b'/');
                } else {
                    state = StrState::Regex { in_class };
                }
            }
        }
        i += 1;
    }

    if let Some(tail) = body.get(start..) {
        if !tail.trim().is_empty() {
            out.push(tail.to_string());
        }
    }

    out
}

fn should_split_after_closing_brace(body: &str, block_open: Option<usize>, tail: &str) -> bool {
    let tail = tail.trim_start();
    if tail.is_empty() {
        return false;
    }
    if tail.starts_with('=') {
        // Preserve object destructuring assignment: `{ a, b } = value`.
        return false;
    }
    if is_keyword_prefix(tail, "else") {
        return false;
    }
    if is_keyword_prefix(tail, "catch") {
        return false;
    }
    if is_keyword_prefix(tail, "finally") {
        return false;
    }
    if is_keyword_prefix(tail, "while")
        && block_open.is_some_and(|open| is_do_block_prefix(body, open))
    {
        return false;
    }
    true
}

fn is_do_block_prefix(body: &str, block_open: usize) -> bool {
    let bytes = body.as_bytes();
    if block_open == 0 || block_open > bytes.len() {
        return false;
    }

    let mut j = block_open;
    while j > 0 && bytes[j - 1].is_ascii_whitespace() {
        j -= 1;
    }
    if j < 2 {
        return false;
    }
    if &bytes[j - 2..j] != b"do" {
        return false;
    }
    match bytes.get(j - 3) {
        Some(&b) => !is_ident_char(b),
        None => true,
    }
}

fn is_keyword_prefix(src: &str, keyword: &str) -> bool {
    let Some(rest) = src.strip_prefix(keyword) else {
        return false;
    };
    rest.is_empty() || !is_ident_char(*rest.as_bytes().first().unwrap_or(&b'\0'))
}

fn parse_function_decl_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

    let body_stmts =
        prepend_callback_param_prologue_stmts(parse_block_statements(&body)?, &parsed_params.prologue)?;

    Ok(Some(Stmt::FunctionDecl {
        name,
        handler: ScriptHandler {
            params: parsed_params.params,
            stmts: body_stmts,
        },
        is_async,
    }))
}

fn parse_var_decl(stmt: &str) -> Result<Option<Stmt>> {
    let mut rest = None;
    for kw in ["const", "let", "var"] {
        if let Some(after) = stmt.strip_prefix(kw) {
            rest = Some(after.trim_start());
            break;
        }
    }

    let Some(rest) = rest else {
        return Ok(None);
    };

    let (name, expr_src) = rest
        .split_once('=')
        .ok_or_else(|| Error::ScriptParse(format!("invalid variable declaration: {stmt}")))?;

    let name = name.trim();
    if !is_ident(name) {
        return Err(Error::ScriptParse(format!(
            "invalid variable name '{name}' in: {stmt}"
        )));
    }

    let expr = parse_expr(expr_src.trim())?;
    Ok(Some(Stmt::VarDecl {
        name: name.to_string(),
        expr,
    }))
}

fn parse_var_assign(stmt: &str) -> Result<Option<Stmt>> {
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

fn find_top_level_var_assignment(stmt: &str) -> Option<(String, usize, &str)> {
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

fn parse_destructure_assign(stmt: &str) -> Result<Option<Stmt>> {
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

fn parse_array_destructure_pattern(pattern: &str) -> Result<Vec<Option<String>>> {
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

fn parse_object_destructure_pattern(pattern: &str) -> Result<Vec<(String, String)>> {
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

fn parse_form_data_append_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    let Some(target_var) = cursor.parse_identifier() else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();

    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if method != "append" {
        return Ok(None);
    }

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Ok(None);
    }

    let name = parse_expr(args[0].trim())?;
    let value = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported FormData.append statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::FormDataAppend {
        target_var,
        name,
        value,
    }))
}

fn parse_dom_method_call_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

    let Some(method_name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let (method, accepts_optional_arg) = match method_name.as_str() {
        "focus" => (DomMethod::Focus, false),
        "blur" => (DomMethod::Blur, false),
        "click" => (DomMethod::Click, false),
        "scrollIntoView" => (DomMethod::ScrollIntoView, false),
        "submit" => (DomMethod::Submit, false),
        "reset" => (DomMethod::Reset, false),
        "show" => (DomMethod::Show, false),
        "showModal" => (DomMethod::ShowModal, false),
        "close" => (DomMethod::Close, true),
        "requestClose" => (DomMethod::RequestClose, true),
        _ => return Ok(None),
    };

    cursor.skip_ws();
    let args = cursor.read_balanced_block(b'(', b')')?;
    let arg = if accepts_optional_arg {
        let parsed_args = split_top_level_by_char(&args, b',');
        if parsed_args.len() == 1 && parsed_args[0].trim().is_empty() {
            None
        } else {
            if parsed_args.len() != 1 || parsed_args[0].trim().is_empty() {
                return Err(Error::ScriptParse(format!(
                    "{} accepts zero or one argument: {stmt}",
                    method_name
                )));
            }
            Some(parse_expr(parsed_args[0].trim())?)
        }
    } else {
        if !args.trim().is_empty() {
            return Err(Error::ScriptParse(format!(
                "{} takes no arguments: {stmt}",
                method_name
            )));
        }
        None
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported {} statement tail: {stmt}",
            method_name
        )));
    }

    Ok(Some(Stmt::DomMethodCall {
        target,
        method,
        arg,
    }))
}

fn parse_dom_assignment(stmt: &str) -> Result<Option<Stmt>> {
    let Some((eq_pos, op_len)) = find_top_level_assignment(stmt) else {
        return Ok(None);
    };

    let lhs = stmt[..eq_pos].trim();
    let rhs = stmt[eq_pos + op_len..].trim();

    if lhs.is_empty() {
        return Ok(None);
    }

    let Some((target, prop)) = parse_dom_access(lhs)? else {
        return Ok(None);
    };

    let expr = parse_expr(rhs)?;
    Ok(Some(Stmt::DomAssign { target, prop, expr }))
}

fn parse_object_assign(stmt: &str) -> Result<Option<Stmt>> {
    let Some((eq_pos, op_len)) = find_top_level_assignment(stmt) else {
        return Ok(None);
    };

    let lhs = stmt[..eq_pos].trim();
    let rhs = stmt[eq_pos + op_len..].trim();
    if lhs.is_empty() {
        return Ok(None);
    }

    let mut cursor = Cursor::new(lhs);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    let key = if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(prop) = cursor.parse_identifier() else {
            return Ok(None);
        };
        Expr::String(prop)
    } else if cursor.peek() == Some(b'[') {
        let key_src = cursor.read_balanced_block(b'[', b']')?;
        let key_src = key_src.trim();
        if key_src.is_empty() {
            return Err(Error::ScriptParse(
                "object assignment key cannot be empty".into(),
            ));
        }
        parse_expr(key_src)?
    } else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let rhs_expr = parse_expr(rhs)?;
    let op = &stmt[eq_pos..eq_pos + op_len];
    let expr = if op_len == 1 {
        rhs_expr
    } else {
        let key_name = match &key {
            Expr::String(name) => name.clone(),
            _ => {
                return Err(Error::ScriptParse(
                    "compound object assignment requires a static key".into(),
                ));
            }
        };
        let lhs_expr = Expr::ObjectGet {
            target: target.clone(),
            key: key_name,
        };
        match op {
            "+=" => append_concat_expr(lhs_expr, rhs_expr),
            "-=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Sub,
                right: Box::new(rhs_expr),
            },
            "*=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Mul,
                right: Box::new(rhs_expr),
            },
            "/=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Div,
                right: Box::new(rhs_expr),
            },
            "%=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Mod,
                right: Box::new(rhs_expr),
            },
            "**=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Pow,
                right: Box::new(rhs_expr),
            },
            _ => return Ok(None),
        }
    };
    Ok(Some(Stmt::ObjectAssign { target, key, expr }))
}

fn parse_query_selector_all_foreach_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    let source = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let method = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse(format!("invalid forEach statement: {stmt}")))?;

    let (target, selector) = match method.as_str() {
        "forEach" => {
            let (target, selector) = match &source {
                DomQuery::BySelectorAll { selector } => (None, selector.clone()),
                DomQuery::QuerySelectorAll { target, selector } => {
                    (Some(target.as_ref().clone()), selector.clone())
                }
                _ => {
                    return Ok(None);
                }
            };
            cursor.skip_ws();
            (target, selector)
        }
        "querySelectorAll" => {
            cursor.skip_ws();
            cursor.expect_byte(b'(')?;
            cursor.skip_ws();
            let selector = cursor.parse_string_literal()?;
            cursor.skip_ws();
            cursor.expect_byte(b')')?;
            cursor.skip_ws();
            if !cursor.consume_byte(b'.') {
                return Ok(None);
            }
            cursor.skip_ws();
            if !cursor.consume_ascii("forEach") {
                return Ok(None);
            }
            (
                match source {
                    DomQuery::DocumentRoot => None,
                    _ => Some(source.clone()),
                },
                selector,
            )
        }
        _ => return Ok(None),
    };
    cursor.skip_ws();

    // For consistency with current test grammar, allow optional event callback without a semicolon.
    cursor.skip_ws();

    let callback_src = cursor.read_balanced_block(b'(', b')')?;
    let (item_var, index_var, body) = parse_for_each_callback(&callback_src)?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported forEach statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::ForEach {
        target,
        selector,
        item_var,
        index_var,
        body,
    }))
}

fn parse_array_for_each_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("forEach") {
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
    if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "forEach requires a callback and optional thisArg".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse("forEach thisArg cannot be empty".into()));
    }
    let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
    if args.len() == 2 {
        let _ = parse_expr(args[1].trim())?;
    }

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported forEach statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::ArrayForEach { target, callback }))
}

pub(super) fn parse_for_each_callback(src: &str) -> Result<(String, Option<String>, Vec<Stmt>)> {
    let mut cursor = Cursor::new(src.trim());
    cursor.skip_ws();
    let mut param_prologue: Vec<String> = Vec::new();

    let (item_var, index_var) = if cursor
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
        let params_src = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        let parsed_params = parse_callback_parameter_list(
            &params_src,
            2,
            "forEach callback must have one or two parameters",
        )?;
        if parsed_params
            .params
            .iter()
            .any(|param| param.default.is_some())
        {
            return Err(Error::ScriptParse(format!(
                "forEach callback must not use default parameters: {src}"
            )));
        }
        let item_var = parsed_params
            .params
            .first()
            .map(|param| param.name.clone())
            .ok_or_else(|| {
                Error::ScriptParse(format!(
                    "forEach callback must have one or two parameters: {src}"
                ))
            })?;
        let index_var = parsed_params.params.get(1).map(|param| param.name.clone());

        cursor.skip_ws();
        let (body, concise_body) = parse_arrow_or_block_body(&mut cursor)?;
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported forEach callback tail: {src}"
            )));
        }

        let body_stmts = if concise_body {
            vec![Stmt::Expr(parse_expr(body.trim())?)]
        } else {
            parse_block_statements(&body)?
        };
        let body_stmts =
            prepend_callback_param_prologue_stmts(body_stmts, &parsed_params.prologue)?;
        return Ok((item_var, index_var, body_stmts));
    } else if cursor.consume_byte(b'(') {
        let params_src = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        let parsed_params = parse_callback_parameter_list(
            &params_src,
            2,
            "forEach callback must have one or two parameters",
        )?;
        if parsed_params
            .params
            .iter()
            .any(|param| param.default.is_some())
        {
            return Err(Error::ScriptParse(format!(
                "forEach callback must not use default parameters: {src}"
            )));
        }
        param_prologue = parsed_params.prologue;
        let item_var = parsed_params
            .params
            .first()
            .map(|param| param.name.clone())
            .ok_or_else(|| {
                Error::ScriptParse(format!(
                    "forEach callback must have one or two parameters: {src}"
                ))
            })?;
        let index_var = parsed_params.params.get(1).map(|param| param.name.clone());
        (item_var, index_var)
    } else {
        let Some(item) = cursor.parse_identifier() else {
            return Err(Error::ScriptParse(format!(
                "invalid forEach callback parameters: {src}"
            )));
        };
        (item, None)
    };

    cursor.skip_ws();
    cursor.expect_ascii("=>")?;
    cursor.skip_ws();
    let (body, concise_body) = parse_arrow_or_block_body(&mut cursor)?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported forEach callback tail: {src}"
        )));
    }

    let body_stmts = if concise_body {
        vec![Stmt::Expr(parse_expr(body.trim())?)]
    } else {
        parse_block_statements(&body)?
    };
    let body_stmts = prepend_callback_param_prologue_stmts(body_stmts, &param_prologue)?;
    Ok((item_var, index_var, body_stmts))
}

fn parse_set_attribute_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("setAttribute") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "setAttribute requires 2 arguments: {stmt}"
        )));
    }
    let name = parse_string_literal_exact(args[0].trim())?;
    let value = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported setAttribute statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::DomSetAttribute {
        target,
        name,
        value,
    }))
}

fn parse_remove_attribute_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("removeAttribute") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported removeAttribute statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::DomRemoveAttribute { target, name }))
}

fn parse_class_list_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("classList") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();

    let method = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse(format!("expected classList method in: {stmt}")))?;

    if method == "forEach" {
        cursor.skip_ws();
        let callback_src = cursor.read_balanced_block(b'(', b')')?;
        let (item_var, index_var, body) = parse_for_each_callback(&callback_src)?;

        cursor.skip_ws();
        cursor.consume_byte(b';');
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported classList statement tail: {stmt}"
            )));
        }

        return Ok(Some(Stmt::ClassListForEach {
            target,
            item_var,
            index_var,
            body,
        }));
    }

    let method = match method.as_str() {
        "add" => ClassListMethod::Add,
        "remove" => ClassListMethod::Remove,
        "toggle" => ClassListMethod::Toggle,
        _ => return Ok(None),
    };

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() {
        return Err(Error::ScriptParse(format!(
            "invalid classList arguments: {stmt}"
        )));
    }

    let force = match method {
        ClassListMethod::Toggle => {
            if args.len() > 2 {
                return Err(Error::ScriptParse(format!(
                    "invalid classList arguments: {stmt}"
                )));
            }

            if args.len() == 2 {
                Some(parse_expr(args[1].trim())?)
            } else {
                None
            }
        }
        _ => None,
    };

    let class_names = match method {
        ClassListMethod::Toggle => vec![parse_string_literal_exact(args[0].trim())?],
        _ => args
            .iter()
            .map(|arg| parse_string_literal_exact(arg.trim()))
            .collect::<Result<Vec<_>>>()?,
    };

    if !matches!(method, ClassListMethod::Toggle) && class_names.is_empty() {
        return Err(Error::ScriptParse(format!(
            "classList add/remove requires at least one argument: {stmt}"
        )));
    }

    if !matches!(method, ClassListMethod::Toggle) && force.is_some() {
        return Err(Error::ScriptParse(
            "classList add/remove do not accept a force argument".into(),
        ));
    }

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();

    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported classList statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::ClassListCall {
        target,
        method,
        class_names,
        force,
    }))
}

fn parse_insert_adjacent_position(src: &str) -> Result<InsertAdjacentPosition> {
    let lowered = src.to_ascii_lowercase();
    match lowered.as_str() {
        "beforebegin" => Ok(InsertAdjacentPosition::BeforeBegin),
        "afterbegin" => Ok(InsertAdjacentPosition::AfterBegin),
        "beforeend" => Ok(InsertAdjacentPosition::BeforeEnd),
        "afterend" => Ok(InsertAdjacentPosition::AfterEnd),
        _ => Err(Error::ScriptParse(format!(
            "unsupported insertAdjacent position: {src}"
        ))),
    }
}

pub(super) fn resolve_insert_adjacent_position(src: &str) -> Result<InsertAdjacentPosition> {
    let lowered = src.to_ascii_lowercase();
    match lowered.as_str() {
        "beforebegin" => Ok(InsertAdjacentPosition::BeforeBegin),
        "afterbegin" => Ok(InsertAdjacentPosition::AfterBegin),
        "beforeend" => Ok(InsertAdjacentPosition::BeforeEnd),
        "afterend" => Ok(InsertAdjacentPosition::AfterEnd),
        _ => Err(Error::ScriptRuntime(format!(
            "unsupported insertAdjacentHTML position: {src}"
        ))),
    }
}

fn parse_insert_adjacent_element_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("insertAdjacentElement") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "insertAdjacentElement requires 2 arguments: {stmt}"
        )));
    }

    let position = parse_insert_adjacent_position(&parse_string_literal_exact(args[0].trim())?)?;
    let node = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported insertAdjacentElement statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::InsertAdjacentElement {
        target,
        position,
        node,
    }))
}

fn parse_insert_adjacent_text_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("insertAdjacentText") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "insertAdjacentText requires 2 arguments: {stmt}"
        )));
    }

    let position = parse_insert_adjacent_position(&parse_string_literal_exact(args[0].trim())?)?;
    let text = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported insertAdjacentText statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::InsertAdjacentText {
        target,
        position,
        text,
    }))
}

fn parse_insert_adjacent_html_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("insertAdjacentHTML") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "insertAdjacentHTML requires 2 arguments: {stmt}"
        )));
    }

    let position = parse_expr(args[0].trim())?;
    let html = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported insertAdjacentHTML statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::InsertAdjacentHTML {
        target,
        position,
        html,
    }))
}

fn parse_set_timer_call(
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

fn parse_set_timeout_call(cursor: &mut Cursor<'_>) -> Result<Option<(TimerInvocation, Expr)>> {
    parse_set_timer_call(cursor, "setTimeout")
}

fn parse_set_interval_call(cursor: &mut Cursor<'_>) -> Result<Option<(TimerInvocation, Expr)>> {
    parse_set_timer_call(cursor, "setInterval")
}

fn parse_set_timeout_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

fn parse_set_interval_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

fn parse_clear_timeout_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

fn parse_node_tree_mutation_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

fn parse_node_remove_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

fn parse_dispatch_event_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

fn parse_listener_mutation_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    let (params, body, _) = parse_callback(&mut cursor, 1, "callback parameters")?;

    cursor.skip_ws();
    let capture = if cursor.consume_byte(b',') {
        cursor.skip_ws();
        if cursor.consume_ascii("true") {
            true
        } else if cursor.consume_ascii("false") {
            false
        } else {
            return Err(Error::ScriptParse(
                "add/removeEventListener third argument must be true/false".into(),
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

    let handler = ScriptHandler {
        params,
        stmts: parse_block_statements(&body)?,
    };
    Ok(Some(Stmt::ListenerMutation {
        target,
        op,
        event_type,
        capture,
        handler,
    }))
}

fn parse_event_call_stmt(stmt: &str) -> Option<Stmt> {
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
