use super::*;

pub(crate) fn parse_block_statements(body: &str) -> Result<Vec<Stmt>> {
    parse_block_statements_with_flags(body, false, false)
}

pub(crate) fn parse_module_block_statements(body: &str) -> Result<Vec<Stmt>> {
    parse_block_statements_with_flags(body, true, true)
}

pub(crate) fn parse_block_statements_with_flags(
    body: &str,
    allow_top_level_export: bool,
    allow_top_level_import: bool,
) -> Result<Vec<Stmt>> {
    let normalized = normalize_malformed_escaped_empty_string_literals(body);
    let sanitized = strip_js_comments(normalized.as_str());
    let raw_stmts = split_top_level_statements(sanitized.as_str());
    let mut stmts = Vec::new();

    for raw in raw_stmts {
        for stmt in split_async_function_asi_statements(raw.trim()) {
            for stmt in split_var_decl_list_statements(stmt) {
                let stmt = stmt.trim();
                if stmt.is_empty() {
                    continue;
                }

                if let Some(else_branch) = parse_else_fragment(stmt)? {
                    if let Some(last_stmt) = stmts.last_mut() {
                        if attach_else_branch_to_if_chain(last_stmt, else_branch) {
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

                let parsed = parse_single_statement_with_flags(
                    stmt,
                    allow_top_level_export,
                    allow_top_level_import,
                )
                .map_err(|err| {
                    Error::ScriptParse(format!("statement parse failed: stmt={stmt:?} err={err:?}"))
                })?;
                stmts.push(parsed);
            }
        }
    }

    Ok(stmts)
}

pub(crate) fn split_var_decl_list_statements(stmt: &str) -> Vec<String> {
    let stmt = stmt.trim();
    let mut prefix = String::new();
    let mut candidate = stmt;
    if let Some(after_export) = stmt.strip_prefix("export") {
        if !after_export
            .as_bytes()
            .first()
            .is_some_and(|b| is_ident_char(*b))
        {
            prefix = "export ".to_string();
            candidate = after_export.trim_start();
        }
    }

    let mut declaration = None;
    for kw in ["const", "let", "var"] {
        if let Some(after) = candidate.strip_prefix(kw) {
            if after.as_bytes().first().is_some_and(|b| is_ident_char(*b)) {
                continue;
            }
            declaration = Some((kw, after.trim_start()));
            break;
        }
    }

    let Some((kw, rest)) = declaration else {
        return vec![stmt.to_string()];
    };
    if rest.is_empty() {
        return vec![stmt.to_string()];
    }

    let parts = split_top_level_by_char(rest, b',');
    if parts.len() <= 1 {
        return vec![stmt.to_string()];
    }

    let mut out = Vec::with_capacity(parts.len());
    for part in parts {
        out.push(format!("{prefix}{kw} {}", part.trim()));
    }
    out
}

pub(crate) fn split_async_function_asi_statements(stmt: &str) -> Vec<&str> {
    let stmt = stmt.trim();
    if stmt.is_empty() {
        return vec![stmt];
    }
    let bytes = stmt.as_bytes();
    if !stmt.starts_with("async") {
        return vec![stmt];
    }
    if bytes.get("async".len()).is_some_and(|b| is_ident_char(*b)) {
        return vec![stmt];
    }

    let mut i = "async".len();
    let mut saw_line_terminator = false;
    while let Some(&b) = bytes.get(i) {
        match b {
            b' ' | b'\t' | 0x0B | 0x0C => {
                i += 1;
            }
            b'\n' | b'\r' => {
                saw_line_terminator = true;
                i += 1;
                if b == b'\r' && bytes.get(i) == Some(&b'\n') {
                    i += 1;
                }
                break;
            }
            _ => return vec![stmt],
        }
    }
    if !saw_line_terminator {
        return vec![stmt];
    }

    while let Some(&b) = bytes.get(i) {
        match b {
            b' ' | b'\t' | 0x0B | 0x0C => i += 1,
            b'\n' | b'\r' => i += 1,
            _ => break,
        }
    }
    let function_stmt = stmt.get(i..).unwrap_or_default();
    if !function_stmt.starts_with("function") {
        return vec![stmt];
    }
    if function_stmt
        .as_bytes()
        .get("function".len())
        .is_some_and(|b| is_ident_char(*b))
    {
        return vec![stmt];
    }

    vec!["async", function_stmt]
}

pub(crate) fn parse_single_statement(stmt: &str) -> Result<Stmt> {
    parse_single_statement_with_flags(stmt, false, false)
}

pub(crate) fn parse_single_statement_with_flags(
    stmt: &str,
    allow_top_level_export: bool,
    allow_top_level_import: bool,
) -> Result<Stmt> {
    let stmt = stmt.trim();

    if let Some(parsed) = parse_empty_stmt(stmt) {
        return Ok(parsed);
    }

    if allow_top_level_export {
        if let Some(parsed) = parse_export_stmt(stmt)? {
            return Ok(parsed);
        }
    } else if starts_with_keyword(stmt, "export") {
        return Err(Error::ScriptParse(
            "export declarations may only appear in module scripts".into(),
        ));
    }

    if allow_top_level_import {
        if let Some(parsed) = parse_import_stmt(stmt)? {
            return Ok(parsed);
        }
    } else if is_static_import_statement_prefix(stmt) {
        return Err(Error::ScriptParse(
            "import declarations may only appear at top level of module scripts".into(),
        ));
    }
    if !allow_top_level_import && contains_import_meta_expression(stmt) {
        return Err(Error::ScriptParse(
            "import.meta may only appear in module scripts".into(),
        ));
    }

    if let Some(parsed) = parse_if_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_do_while_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_switch_stmt(stmt)? {
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

    if let Some(parsed) = parse_block_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_labeled_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_return_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_throw_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_debugger_stmt(stmt)? {
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

    if let Some(parsed) = parse_class_decl_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_var_decl(stmt)? {
        return Ok(parsed);
    }

    // Comma has the lowest precedence. If a statement contains a top-level
    // comma, it must be parsed as a sequence expression instead of being
    // treated as a single assignment statement.
    if split_top_level_by_char(stmt, b',').len() > 1 {
        return Ok(Stmt::Expr(parse_expr(stmt)?));
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

    if let Some(parsed) = parse_private_assign(stmt)? {
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

pub(crate) fn parse_labeled_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    let Some(name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b':') {
        return Ok(None);
    }
    if is_reserved_label_word(&name) {
        return Err(Error::ScriptParse(format!(
            "label cannot use reserved word: {name}"
        )));
    }
    cursor.skip_ws();
    if cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "labeled statement requires a body: {stmt}"
        )));
    }
    let rest = cursor.src.get(cursor.i..).unwrap_or_default();
    let parsed = parse_single_statement(rest)?;
    if matches!(
        parsed,
        Stmt::VarDecl {
            kind: VarDeclKind::Let | VarDeclKind::Const,
            ..
        } | Stmt::ClassDecl { .. }
    ) {
        return Err(Error::ScriptParse(
            "lexical declaration cannot be labeled".into(),
        ));
    }
    if let Stmt::FunctionDecl {
        is_async,
        is_generator,
        ..
    } = &parsed
    {
        if *is_async || *is_generator {
            return Err(Error::ScriptParse(
                "only non-async, non-generator functions may be labeled".into(),
            ));
        }
    }
    Ok(Some(Stmt::Label {
        name,
        stmt: Box::new(parsed),
    }))
}

pub(crate) fn parse_block_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if cursor.peek() != Some(b'{') {
        return Ok(None);
    }

    let body = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Stmt::Block {
        stmts: parse_block_statements(&body)?,
    }))
}

pub(crate) fn parse_empty_stmt(stmt: &str) -> Option<Stmt> {
    let stmt = stmt.trim();
    if stmt == ";" { Some(Stmt::Empty) } else { None }
}

pub(crate) fn is_ident_char(b: u8) -> bool {
    b == b'_' || b == b'$' || b.is_ascii_alphanumeric()
}

pub(crate) fn consume_keyword(cursor: &mut Cursor<'_>, keyword: &str) -> bool {
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
