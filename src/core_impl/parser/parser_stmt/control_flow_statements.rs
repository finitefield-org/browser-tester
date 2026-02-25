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
    let sanitized = strip_js_comments(body);
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
                )?;
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

pub(crate) fn attach_else_branch_to_if_chain(stmt: &mut Stmt, else_branch: Vec<Stmt>) -> bool {
    let Stmt::If { else_stmts, .. } = stmt else {
        return false;
    };

    if else_stmts.is_empty() {
        *else_stmts = else_branch;
        return true;
    }

    if else_stmts.len() != 1 {
        return false;
    }

    attach_else_branch_to_if_chain(&mut else_stmts[0], else_branch)
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

pub(crate) fn is_static_import_statement_prefix(stmt: &str) -> bool {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !consume_keyword(&mut cursor, "import") {
        return false;
    }
    cursor.skip_ws();
    matches!(
        cursor.peek(),
        Some(b'"' | b'\'' | b'{' | b'*' | b'_' | b'$' | b'a'..=b'z' | b'A'..=b'Z')
    )
}

pub(crate) fn contains_import_meta_expression(src: &str) -> bool {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.in_normal()
            && i + 6 <= bytes.len()
            && &bytes[i..i + 6] == b"import"
            && (i == 0 || !is_ident_char(bytes[i - 1]))
            && (i + 6 == bytes.len() || !is_ident_char(bytes[i + 6]))
        {
            let mut prev = i;
            let mut previous_significant = None;
            while prev > 0 {
                prev -= 1;
                let ch = bytes[prev];
                if !ch.is_ascii_whitespace() {
                    previous_significant = Some(ch);
                    break;
                }
            }
            if previous_significant == Some(b'.') {
                i = scanner.advance(bytes, i);
                continue;
            }

            let mut cursor = i + 6;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor >= bytes.len() || bytes[cursor] != b'.' {
                i = scanner.advance(bytes, i);
                continue;
            }
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor + 4 <= bytes.len()
                && &bytes[cursor..cursor + 4] == b"meta"
                && (cursor + 4 == bytes.len() || !is_ident_char(bytes[cursor + 4]))
            {
                return true;
            }
        }
        i = scanner.advance(bytes, i);
    }

    false
}

pub(crate) fn parse_import_attribute_type(cursor: &mut Cursor<'_>) -> Result<Option<String>> {
    if !consume_keyword(cursor, "with") {
        return Ok(None);
    }
    cursor.skip_ws();
    let attrs_src = cursor.read_balanced_block(b'{', b'}')?;
    let attrs_src = attrs_src.trim();
    if attrs_src.is_empty() {
        return Ok(None);
    }

    let mut attr_type = None;
    let mut parts = split_top_level_by_char(attrs_src, b',');
    if parts.len() > 1 && parts.last().is_some_and(|part| part.trim().is_empty()) {
        parts.pop();
    }
    for part in parts {
        let mut item = Cursor::new(part.trim());
        item.skip_ws();
        let key = if matches!(item.peek(), Some(b'"' | b'\'')) {
            item.parse_string_literal()?
        } else {
            item.parse_identifier().ok_or_else(|| {
                Error::ScriptParse(format!("invalid import attribute key: {}", part.trim()))
            })?
        };
        item.skip_ws();
        item.expect_byte(b':')?;
        item.skip_ws();
        let value = item.parse_string_literal()?;
        item.skip_ws();
        if !item.eof() {
            return Err(Error::ScriptParse(format!(
                "invalid import attribute entry: {}",
                part.trim()
            )));
        }
        if key == "type" {
            attr_type = Some(value);
        } else {
            return Err(Error::ScriptParse(format!(
                "unsupported import attribute: {key}"
            )));
        }
    }
    Ok(attr_type)
}

pub(crate) fn parse_import_specifier_list(src: &str) -> Result<Vec<ImportBinding>> {
    let src = src.trim();
    if src.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    let mut parts = split_top_level_by_char(src, b',');
    if parts.len() > 1 && parts.last().is_some_and(|part| part.trim().is_empty()) {
        parts.pop();
    }
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            return Err(Error::ScriptParse(
                "import specifier cannot be empty".into(),
            ));
        }
        let mut cursor = Cursor::new(part);
        cursor.skip_ws();
        let (imported, imported_is_string) = if matches!(cursor.peek(), Some(b'"' | b'\'')) {
            (cursor.parse_string_literal()?, true)
        } else {
            (
                cursor
                    .parse_identifier()
                    .ok_or_else(|| Error::ScriptParse("invalid import specifier".into()))?,
                false,
            )
        };
        cursor.skip_ws();
        let local = if consume_keyword(&mut cursor, "as") {
            cursor.skip_ws();
            cursor
                .parse_identifier()
                .ok_or_else(|| Error::ScriptParse("invalid import alias".into()))?
        } else if imported_is_string {
            return Err(Error::ScriptParse(
                "string import specifier requires an alias".into(),
            ));
        } else {
            imported.clone()
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported import specifier: {part}"
            )));
        }
        out.push(ImportBinding { imported, local });
    }
    Ok(out)
}

pub(crate) fn parse_import_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !consume_keyword(&mut cursor, "import") {
        return Ok(None);
    }
    cursor.skip_ws();
    if matches!(cursor.peek(), Some(b'(' | b'.')) {
        return Ok(None);
    }
    if cursor.eof() {
        return Err(Error::ScriptParse(
            "import statement requires a module specifier".into(),
        ));
    }

    let mut default_binding = None;
    let mut namespace_binding = None;
    let mut named_bindings = Vec::new();

    if matches!(cursor.peek(), Some(b'"' | b'\'')) {
        let specifier = cursor.parse_string_literal()?;
        cursor.skip_ws();
        let attribute_type = parse_import_attribute_type(&mut cursor)?;
        cursor.skip_ws();
        cursor.consume_byte(b';');
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported import statement tail: {stmt}"
            )));
        }
        return Ok(Some(Stmt::ImportDecl {
            specifier,
            default_binding,
            namespace_binding,
            named_bindings,
            attribute_type,
        }));
    }

    if cursor.peek() == Some(b'*') {
        cursor.consume_byte(b'*');
        cursor.skip_ws();
        if !consume_keyword(&mut cursor, "as") {
            return Err(Error::ScriptParse("namespace import requires `as`".into()));
        }
        cursor.skip_ws();
        namespace_binding = Some(
            cursor
                .parse_identifier()
                .ok_or_else(|| Error::ScriptParse("invalid namespace import alias".into()))?,
        );
    } else if cursor.peek() == Some(b'{') {
        let specifier_src = cursor.read_balanced_block(b'{', b'}')?;
        named_bindings = parse_import_specifier_list(&specifier_src)?;
    } else {
        default_binding = Some(
            cursor
                .parse_identifier()
                .ok_or_else(|| Error::ScriptParse("invalid default import binding".into()))?,
        );
        cursor.skip_ws();
        if cursor.consume_byte(b',') {
            cursor.skip_ws();
            if cursor.peek() == Some(b'*') {
                cursor.consume_byte(b'*');
                cursor.skip_ws();
                if !consume_keyword(&mut cursor, "as") {
                    return Err(Error::ScriptParse("namespace import requires `as`".into()));
                }
                cursor.skip_ws();
                namespace_binding =
                    Some(cursor.parse_identifier().ok_or_else(|| {
                        Error::ScriptParse("invalid namespace import alias".into())
                    })?);
            } else if cursor.peek() == Some(b'{') {
                let specifier_src = cursor.read_balanced_block(b'{', b'}')?;
                named_bindings = parse_import_specifier_list(&specifier_src)?;
            } else {
                return Err(Error::ScriptParse(
                    "invalid import clause after default binding".into(),
                ));
            }
        }
    }

    cursor.skip_ws();
    if !consume_keyword(&mut cursor, "from") {
        return Err(Error::ScriptParse("import clause requires `from`".into()));
    }
    cursor.skip_ws();
    let specifier = cursor.parse_string_literal()?;
    cursor.skip_ws();
    let attribute_type = parse_import_attribute_type(&mut cursor)?;
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported import statement tail: {stmt}"
        )));
    }

    let mut seen_locals = HashSet::new();
    if let Some(local) = &default_binding {
        seen_locals.insert(local.clone());
    }
    if let Some(local) = &namespace_binding {
        if !seen_locals.insert(local.clone()) {
            return Err(Error::ScriptParse(format!(
                "duplicate import binding name: {local}"
            )));
        }
    }
    for binding in &named_bindings {
        if !seen_locals.insert(binding.local.clone()) {
            return Err(Error::ScriptParse(format!(
                "duplicate import binding name: {}",
                binding.local
            )));
        }
    }

    Ok(Some(Stmt::ImportDecl {
        specifier,
        default_binding,
        namespace_binding,
        named_bindings,
        attribute_type,
    }))
}

pub(crate) fn parse_export_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !consume_keyword(&mut cursor, "export") {
        return Ok(None);
    }
    cursor.skip_ws();
    if cursor.eof() {
        return Err(Error::ScriptParse(
            "export statement requires a declaration".into(),
        ));
    }

    if cursor.peek() == Some(b'*') {
        return Err(Error::ScriptParse(
            "export-from declarations are not supported yet".into(),
        ));
    }

    if consume_keyword(&mut cursor, "default") {
        cursor.skip_ws();
        let remainder = cursor.src.get(cursor.i..).unwrap_or_default().trim();
        if remainder.is_empty() {
            return Err(Error::ScriptParse("export default requires a value".into()));
        }

        if starts_with_keyword(remainder, "function") || starts_with_keyword(remainder, "async") {
            if let Ok(Some(parsed)) = parse_function_decl_stmt(remainder) {
                if let Stmt::FunctionDecl { name, .. } = &parsed {
                    let local_name = name.clone();
                    return Ok(Some(Stmt::ExportDecl {
                        declaration: Box::new(parsed),
                        bindings: vec![(local_name, "default".to_string())],
                    }));
                }
                return Ok(Some(Stmt::ExportDefaultExpr {
                    expr: parse_expr(trim_optional_trailing_semicolon(remainder))?,
                }));
            }
        }
        if starts_with_keyword(remainder, "class") {
            if let Ok(Some(parsed)) = parse_class_decl_stmt(remainder) {
                if let Stmt::ClassDecl { name, .. } = &parsed {
                    let local_name = name.clone();
                    return Ok(Some(Stmt::ExportDecl {
                        declaration: Box::new(parsed),
                        bindings: vec![(local_name, "default".to_string())],
                    }));
                }
                return Ok(Some(Stmt::ExportDefaultExpr {
                    expr: parse_expr(trim_optional_trailing_semicolon(remainder))?,
                }));
            }
        }

        let expr_src = trim_optional_trailing_semicolon(remainder);
        if expr_src.is_empty() {
            return Err(Error::ScriptParse("export default requires a value".into()));
        }
        return Ok(Some(Stmt::ExportDefaultExpr {
            expr: parse_expr(expr_src)?,
        }));
    }

    if cursor.peek() == Some(b'{') {
        let specifier_src = cursor.read_balanced_block(b'{', b'}')?;
        let bindings = parse_export_specifier_list(&specifier_src)?;
        cursor.skip_ws();
        if consume_keyword(&mut cursor, "from") {
            return Err(Error::ScriptParse(
                "export-from declarations are not supported yet".into(),
            ));
        }
        cursor.consume_byte(b';');
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported export statement tail: {stmt}"
            )));
        }
        return Ok(Some(Stmt::ExportNamed { bindings }));
    }

    let remainder = cursor.src.get(cursor.i..).unwrap_or_default().trim();
    if remainder.is_empty() {
        return Err(Error::ScriptParse(
            "export statement requires a declaration".into(),
        ));
    }

    let parsed = parse_single_statement_with_flags(remainder, false, false)?;
    if !is_exportable_declaration_stmt(&parsed) {
        return Err(Error::ScriptParse(format!(
            "unsupported export declaration: {stmt}"
        )));
    }
    Ok(Some(Stmt::ExportDecl {
        bindings: export_bindings_from_declaration_stmt(&parsed),
        declaration: Box::new(parsed),
    }))
}

pub(crate) fn parse_export_specifier_list(src: &str) -> Result<Vec<(String, String)>> {
    let src = src.trim();
    if src.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    let mut parts = split_top_level_by_char(src, b',');
    if parts.len() > 1 && parts.last().is_some_and(|part| part.trim().is_empty()) {
        parts.pop();
    }
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            return Err(Error::ScriptParse(
                "export specifier cannot be empty".into(),
            ));
        }
        let mut cursor = Cursor::new(part);
        cursor.skip_ws();
        let local = parse_export_specifier_name(&mut cursor, false)?;
        cursor.skip_ws();
        let exported = if consume_keyword(&mut cursor, "as") {
            cursor.skip_ws();
            parse_export_specifier_name(&mut cursor, true)?
        } else {
            local.clone()
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported export specifier: {part}"
            )));
        }
        out.push((local, exported));
    }
    Ok(out)
}

pub(crate) fn parse_export_specifier_name(
    cursor: &mut Cursor<'_>,
    allow_string_literal: bool,
) -> Result<String> {
    if allow_string_literal && matches!(cursor.peek(), Some(b'"' | b'\'')) {
        return cursor.parse_string_literal();
    }

    cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse("invalid export specifier name".into()))
}

pub(crate) fn export_bindings_from_declaration_stmt(stmt: &Stmt) -> Vec<(String, String)> {
    match stmt {
        Stmt::VarDecl { name, .. } => vec![(name.clone(), name.clone())],
        Stmt::FunctionDecl { name, .. } => vec![(name.clone(), name.clone())],
        Stmt::ClassDecl { name, .. } => vec![(name.clone(), name.clone())],
        Stmt::ArrayDestructureAssign {
            pattern,
            decl_kind: Some(_),
            ..
        } => {
            let mut out = pattern
                .items
                .iter()
                .flatten()
                .map(|binding| (binding.target.clone(), binding.target.clone()))
                .collect::<Vec<_>>();
            if let Some(rest) = &pattern.rest {
                out.push((rest.clone(), rest.clone()));
            }
            out
        }
        Stmt::ObjectDestructureAssign {
            pattern,
            decl_kind: Some(_),
            ..
        } => {
            let mut out = pattern
                .bindings
                .iter()
                .map(|binding| (binding.target.clone(), binding.target.clone()))
                .collect::<Vec<_>>();
            if let Some(rest) = &pattern.rest {
                out.push((rest.clone(), rest.clone()));
            }
            out
        }
        _ => Vec::new(),
    }
}

pub(crate) fn is_exportable_declaration_stmt(stmt: &Stmt) -> bool {
    matches!(
        stmt,
        Stmt::VarDecl { .. }
            | Stmt::FunctionDecl { .. }
            | Stmt::ClassDecl { .. }
            | Stmt::ArrayDestructureAssign {
                decl_kind: Some(_),
                ..
            }
            | Stmt::ObjectDestructureAssign {
                decl_kind: Some(_),
                ..
            }
    )
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

pub(crate) fn parse_else_fragment(stmt: &str) -> Result<Option<Vec<Stmt>>> {
    let trimmed = stmt.trim_start();
    let Some(rest) = strip_else_prefix(trimmed) else {
        return Ok(None);
    };
    let branch = parse_if_branch(rest.trim())?;
    Ok(Some(branch))
}

pub(crate) fn strip_else_prefix(src: &str) -> Option<&str> {
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

pub(crate) fn parse_if_branch(src: &str) -> Result<Vec<Stmt>> {
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
        return Ok(vec![Stmt::Block {
            stmts: parse_block_statements(&body)?,
        }]);
    }

    let single = trim_optional_trailing_semicolon(src);
    if single.is_empty() {
        return Ok(vec![Stmt::Empty]);
    }
    let parsed = parse_single_statement(single)?;
    if matches!(
        parsed,
        Stmt::VarDecl {
            kind: VarDeclKind::Let | VarDeclKind::Const,
            ..
        } | Stmt::ClassDecl { .. }
    ) {
        return Err(Error::ScriptParse(
            "lexical declaration cannot appear in a single-statement context".into(),
        ));
    }
    Ok(vec![parsed])
}

pub(crate) fn parse_empty_stmt(stmt: &str) -> Option<Stmt> {
    let stmt = stmt.trim();
    if stmt == ";" { Some(Stmt::Empty) } else { None }
}

pub(crate) fn starts_with_keyword(src: &str, keyword: &str) -> bool {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    consume_keyword(&mut cursor, keyword)
}

pub(crate) fn trim_optional_trailing_semicolon(src: &str) -> &str {
    let mut trimmed = src.trim_end();
    if let Some(without) = trimmed.strip_suffix(';') {
        trimmed = without.trim_end();
    }
    trimmed
}

pub(crate) fn collect_top_level_if_branch_candidate_ends(src: &str) -> Vec<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();
    let mut out = Vec::new();

    while i < bytes.len() {
        let current = i;
        let b = bytes[current];
        let was_top_level = scanner.is_top_level();
        i = scanner.advance(bytes, i);
        if !was_top_level {
            continue;
        }

        match b {
            b';' => out.push(i),
            b'}' => {
                if scanner.is_top_level() {
                    out.push(i);
                }
            }
            b'e' => {
                if current + 4 <= bytes.len()
                    && &bytes[current..current + 4] == b"else"
                    && (current == 0 || !is_ident_char(bytes[current - 1]))
                    && (current + 4 == bytes.len() || !is_ident_char(bytes[current + 4]))
                {
                    out.push(current);
                }
            }
            _ => {}
        }
    }

    out.push(src.len());
    out.sort_unstable();
    out.dedup();
    out
}

pub(crate) fn is_ident_char(b: u8) -> bool {
    b == b'_' || b == b'$' || b.is_ascii_alphanumeric()
}

pub(crate) fn is_reserved_label_word(name: &str) -> bool {
    matches!(
        name,
        "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "null"
            | "true"
            | "false"
    )
}

pub(crate) fn parse_if_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

    let mut then_raw = None;
    let mut else_raw = None;
    let mut branch_error = None;

    for end in collect_top_level_if_branch_candidate_ends(tail) {
        let Some(candidate_then) = tail.get(..end) else {
            continue;
        };
        if candidate_then.trim().is_empty() {
            continue;
        }

        let rest = tail.get(end..).unwrap_or_default().trim_start();
        let candidate_else = if rest.is_empty() {
            None
        } else {
            strip_else_prefix(rest)
        };
        if !rest.is_empty() && candidate_else.is_none() {
            continue;
        }

        if let Err(err) = parse_if_branch(candidate_then) {
            branch_error = Some(err);
            continue;
        }

        if let Some(candidate_else) = candidate_else {
            if let Err(err) = parse_if_branch(candidate_else) {
                branch_error = Some(err);
                continue;
            }
            then_raw = Some(candidate_then);
            else_raw = Some(candidate_else);
        } else {
            then_raw = Some(candidate_then);
            else_raw = None;
        }
    }

    let Some(then_raw) = then_raw else {
        if let Some(err) = branch_error {
            return Err(err);
        }
        return Err(Error::ScriptParse(format!(
            "if statement has invalid branch syntax: {stmt}"
        )));
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

pub(crate) fn parse_while_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    let body_raw = cursor.src.get(cursor.i..).unwrap_or_default().trim();
    if body_raw.is_empty() {
        return Err(Error::ScriptParse(format!(
            "while statement has no body: {stmt}"
        )));
    }
    let body = parse_if_branch(body_raw)?;

    Ok(Some(Stmt::While { cond, body }))
}

pub(crate) fn parse_do_while_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    let remainder = cursor.src.get(cursor.i..).unwrap_or_default();
    let while_positions = find_top_level_keyword_positions(remainder, "while");
    for while_pos in while_positions {
        let Some(body_src) = remainder.get(..while_pos) else {
            continue;
        };
        let body_src = body_src.trim();
        if body_src.is_empty() {
            continue;
        }
        let Ok(body) = parse_if_branch(body_src) else {
            continue;
        };

        let Some(while_src) = remainder.get(while_pos..) else {
            continue;
        };
        let mut while_cursor = Cursor::new(while_src);
        while_cursor.skip_ws();
        if !consume_keyword(&mut while_cursor, "while") {
            continue;
        }
        while_cursor.skip_ws();
        let Ok(cond_src) = while_cursor.read_balanced_block(b'(', b')') else {
            continue;
        };
        let Ok(cond) = parse_expr(cond_src.trim()) else {
            continue;
        };
        while_cursor.skip_ws();
        while_cursor.consume_byte(b';');
        while_cursor.skip_ws();
        if !while_cursor.eof() {
            continue;
        }
        return Ok(Some(Stmt::DoWhile { cond, body }));
    }

    Err(Error::ScriptParse(format!(
        "unsupported do statement: {stmt}"
    )))
}

pub(crate) fn parse_switch_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    if !consume_keyword(&mut cursor, "switch") {
        return Ok(None);
    }

    cursor.skip_ws();
    let expr_src = cursor.read_balanced_block(b'(', b')')?;
    let expr = parse_expr(expr_src.trim())?;

    cursor.skip_ws();
    let body_src = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported switch statement tail: {stmt}"
        )));
    }

    let clauses = parse_switch_clauses(&body_src)?;
    Ok(Some(Stmt::Switch { expr, clauses }))
}

pub(crate) fn parse_switch_clauses(body: &str) -> Result<Vec<SwitchClause>> {
    let mut cursor = Cursor::new(body);
    let mut clauses = Vec::new();
    let mut saw_default = false;

    while !cursor.eof() {
        cursor.skip_ws();
        if cursor.eof() {
            break;
        }

        let test = if consume_keyword(&mut cursor, "case") {
            cursor.skip_ws();
            let rest = cursor.src.get(cursor.i..).unwrap_or_default();
            let Some(colon_idx) = find_first_top_level_colon(rest) else {
                return Err(Error::ScriptParse(
                    "switch case clause requires a ':'".into(),
                ));
            };
            let expr_src = rest.get(..colon_idx).unwrap_or_default().trim();
            if expr_src.is_empty() {
                return Err(Error::ScriptParse(
                    "switch case clause requires an expression".into(),
                ));
            }
            let test = parse_expr(expr_src)?;
            cursor.set_pos(cursor.pos() + colon_idx + 1);
            Some(test)
        } else if consume_keyword(&mut cursor, "default") {
            if saw_default {
                return Err(Error::ScriptParse(
                    "switch statement cannot have multiple default clauses".into(),
                ));
            }
            saw_default = true;
            cursor.skip_ws();
            cursor.expect_byte(b':')?;
            None
        } else {
            return Err(Error::ScriptParse(
                "switch clause must start with case or default".into(),
            ));
        };

        let rest = cursor.src.get(cursor.i..).unwrap_or_default();
        let next_clause_offset = find_next_top_level_switch_clause_offset(rest);
        let clause_src = if let Some(offset) = next_clause_offset {
            rest.get(..offset).unwrap_or_default()
        } else {
            rest
        };
        let stmts = parse_block_statements(clause_src)?;
        clauses.push(SwitchClause { test, stmts });

        if let Some(offset) = next_clause_offset {
            cursor.set_pos(cursor.pos() + offset);
        } else {
            cursor.set_pos(cursor.src.len());
        }
    }

    Ok(clauses)
}

pub(crate) fn find_next_top_level_switch_clause_offset(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() {
            if is_switch_clause_keyword_at(bytes, i, b"case")
                || is_switch_clause_keyword_at(bytes, i, b"default")
            {
                return Some(i);
            }
        }
        i = scanner.advance(bytes, i);
    }

    None
}

pub(crate) fn is_switch_clause_keyword_at(bytes: &[u8], index: usize, keyword: &[u8]) -> bool {
    if index + keyword.len() > bytes.len() {
        return false;
    }
    if &bytes[index..index + keyword.len()] != keyword {
        return false;
    }
    if index > 0 && is_ident_char(bytes[index - 1]) {
        return false;
    }
    if index + keyword.len() < bytes.len() && is_ident_char(bytes[index + keyword.len()]) {
        return false;
    }
    true
}

pub(crate) fn find_top_level_keyword_positions(src: &str, keyword: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let bytes = src.as_bytes();
    let keyword_bytes = keyword.as_bytes();
    if keyword_bytes.is_empty() || bytes.len() < keyword_bytes.len() {
        return positions;
    }

    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();
    while i < bytes.len() {
        if scanner.is_top_level()
            && bytes[i] == keyword_bytes[0]
            && i + keyword_bytes.len() <= bytes.len()
            && &bytes[i..i + keyword_bytes.len()] == keyword_bytes
            && (i == 0 || !is_ident_char(bytes[i - 1]))
            && (i + keyword_bytes.len() == bytes.len()
                || !is_ident_char(bytes[i + keyword_bytes.len()]))
        {
            positions.push(i);
        }
        i = scanner.advance(bytes, i);
    }
    positions
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

pub(crate) fn parse_catch_binding(src: &str) -> Result<CatchBinding> {
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

pub(crate) fn parse_try_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !consume_keyword(&mut cursor, "try") {
        return Ok(None);
    }

    cursor.skip_ws();
    let try_src = cursor.read_balanced_block(b'{', b'}')?;
    let try_stmts = vec![Stmt::Block {
        stmts: parse_block_statements(&try_src)?,
    }];

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
        catch_stmts = Some(vec![Stmt::Block {
            stmts: parse_block_statements(&catch_src)?,
        }]);
        cursor.skip_ws();
    }

    if consume_keyword(&mut cursor, "finally") {
        cursor.skip_ws();
        let finally_src = cursor.read_balanced_block(b'{', b'}')?;
        finally_stmts = Some(vec![Stmt::Block {
            stmts: parse_block_statements(&finally_src)?,
        }]);
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

pub(crate) fn parse_throw_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let throw_operand_required =
        || Error::ScriptParse("throw statement requires an operand".into());
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !consume_keyword(&mut cursor, "throw") {
        return Ok(None);
    }

    // ASI rule: `throw` cannot be followed by a line terminator.
    loop {
        match cursor.peek() {
            Some(b' ') | Some(b'\t') | Some(0x0B) | Some(0x0C) => {
                cursor.set_pos(cursor.pos() + 1);
            }
            Some(b'\n') | Some(b'\r') => return Err(throw_operand_required()),
            Some(b'/') => {
                if cursor.consume_ascii("//") {
                    return Err(throw_operand_required());
                }
                if cursor.consume_ascii("/*") {
                    let mut saw_line_terminator = false;
                    while !cursor.eof() {
                        if cursor.consume_ascii("*/") {
                            break;
                        }
                        if matches!(cursor.peek(), Some(b'\n') | Some(b'\r')) {
                            saw_line_terminator = true;
                        }
                        cursor.set_pos(cursor.pos() + 1);
                    }
                    if saw_line_terminator {
                        return Err(throw_operand_required());
                    }
                    continue;
                }
                break;
            }
            _ => break,
        }
    }

    if cursor.eof() {
        return Err(throw_operand_required());
    }

    let expr_src = cursor.src.get(cursor.i..).unwrap_or_default().trim();
    let expr_src = expr_src.strip_suffix(';').unwrap_or(expr_src).trim();
    if expr_src.is_empty() {
        return Err(throw_operand_required());
    }
    let value = parse_expr(expr_src)?;
    Ok(Some(Stmt::Throw { value }))
}

pub(crate) fn parse_return_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

    while let Some(ch) = cursor.peek() {
        match ch {
            b' ' | b'\t' | 0x0B | 0x0C => cursor.set_pos(cursor.pos() + 1),
            b'\n' | b'\r' => return Ok(Some(Stmt::Return { value: None })),
            _ => break,
        }
    }
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

pub(crate) fn parse_debugger_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !consume_keyword(&mut cursor, "debugger") {
        return Ok(None);
    }

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported debugger statement: {stmt}"
        )));
    }

    Ok(Some(Stmt::Debugger))
}

pub(crate) fn parse_break_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    let label = if cursor.peek() == Some(b';') || cursor.eof() {
        None
    } else {
        let Some(label) = cursor.parse_identifier() else {
            return Err(Error::ScriptParse(format!(
                "unsupported break statement: {stmt}"
            )));
        };
        if is_reserved_label_word(&label) {
            return Err(Error::ScriptParse(format!(
                "break label cannot use reserved word: {label}"
            )));
        }
        cursor.skip_ws();
        Some(label)
    };
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported break statement: {stmt}"
        )));
    }
    Ok(Some(Stmt::Break { label }))
}

pub(crate) fn parse_continue_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    let label = if cursor.peek() == Some(b';') || cursor.eof() {
        None
    } else {
        let Some(label) = cursor.parse_identifier() else {
            return Err(Error::ScriptParse(format!(
                "unsupported continue statement: {stmt}"
            )));
        };
        if is_reserved_label_word(&label) {
            return Err(Error::ScriptParse(format!(
                "continue label cannot use reserved word: {label}"
            )));
        }
        cursor.skip_ws();
        Some(label)
    };
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported continue statement: {stmt}"
        )));
    }
    Ok(Some(Stmt::Continue { label }))
}

pub(crate) fn parse_for_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    let is_for_await = consume_keyword(&mut cursor, "await");
    if is_for_await {
        cursor.skip_ws();
    }
    let header_src = cursor.read_balanced_block(b'(', b')')?;
    let header_src = header_src.trim();
    let header_parts = split_top_level_by_char(header_src, b';');

    let parsed_for = if is_for_await {
        if header_parts.len() != 1 {
            return Err(Error::ScriptParse(format!(
                "for await statement requires an of-clause: {stmt}"
            )));
        }
        let Some((kind, item_var, iterable_src)) = parse_for_in_of_stmt(header_src)? else {
            return Err(Error::ScriptParse(format!(
                "for await statement requires an of-clause: {stmt}"
            )));
        };
        if kind != ForInOfKind::Of {
            return Err(Error::ScriptParse(format!(
                "for await statement only supports of: {stmt}"
            )));
        }
        let iterable = parse_expr(iterable_src.trim())?;
        Stmt::ForAwaitOf {
            item_var,
            iterable,
            body: Vec::new(),
        }
    } else if header_parts.len() == 3 {
        let init = parse_for_clause_stmts(header_parts[0])?;
        let cond = if header_parts[1].trim().is_empty() {
            None
        } else {
            Some(parse_expr(header_parts[1].trim())?)
        };
        let post = parse_for_clause_stmts(header_parts[2])?;

        Stmt::For {
            init,
            cond,
            post,
            body: Vec::new(),
        }
    } else if header_parts.len() == 1 {
        let Some((kind, item_var, iterable_src)) = parse_for_in_of_stmt(header_src)? else {
            return Err(Error::ScriptParse(format!(
                "unsupported for statement: {stmt}"
            )));
        };
        if kind == ForInOfKind::Of && item_var == "async" {
            return Err(Error::ScriptParse(
                "The left-hand side of a for-of loop may not be 'async'".into(),
            ));
        }
        let iterable = parse_expr(iterable_src.trim())?;
        match kind {
            ForInOfKind::In => Stmt::ForIn {
                item_var,
                iterable,
                body: Vec::new(),
            },
            ForInOfKind::Of => Stmt::ForOf {
                item_var,
                iterable,
                body: Vec::new(),
            },
        }
    } else {
        return Err(Error::ScriptParse(format!(
            "unsupported for statement: {stmt}"
        )));
    };

    cursor.skip_ws();
    let body_raw = cursor.src.get(cursor.i..).unwrap_or_default().trim();
    if body_raw.is_empty() {
        return Err(Error::ScriptParse(format!(
            "for statement has no body: {stmt}"
        )));
    }
    let body = parse_if_branch(body_raw)?;

    let stmt = match parsed_for {
        Stmt::For {
            init, cond, post, ..
        } => Stmt::For {
            init,
            cond,
            post,
            body,
        },
        Stmt::ForIn {
            item_var, iterable, ..
        } => Stmt::ForIn {
            item_var,
            iterable,
            body,
        },
        Stmt::ForOf {
            item_var, iterable, ..
        } => Stmt::ForOf {
            item_var,
            iterable,
            body,
        },
        Stmt::ForAwaitOf {
            item_var, iterable, ..
        } => Stmt::ForAwaitOf {
            item_var,
            iterable,
            body,
        },
        _ => unreachable!(),
    };
    Ok(Some(stmt))
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

    let in_pos = find_top_level_in_of_keyword(header, "in")?;
    let of_pos = find_top_level_in_of_keyword(header, "of")?;
    let found = match (in_pos, of_pos) {
        (Some(in_pos), Some(of_pos)) if in_pos < of_pos => Some((ForInOfKind::In, in_pos, "in")),
        (Some(_), Some(of_pos)) => Some((ForInOfKind::Of, of_pos, "of")),
        (Some(in_pos), None) => Some((ForInOfKind::In, in_pos, "in")),
        (None, Some(of_pos)) => Some((ForInOfKind::Of, of_pos, "of")),
        (None, None) => None,
    };

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

pub(crate) fn find_top_level_in_of_keyword(src: &str, keyword: &str) -> Result<Option<usize>> {
    let bytes = src.as_bytes();
    let keyword_bytes = keyword.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level()
            && i + keyword_bytes.len() <= bytes.len()
            && &bytes[i..i + keyword_bytes.len()] == keyword_bytes
        {
            let prev_ok = i == 0 || !is_ident_char(bytes[i - 1]);
            let next = bytes.get(i + keyword_bytes.len()).copied();
            let next_ok = next.map_or(true, |b| !is_ident_char(b));
            if prev_ok && next_ok {
                return Ok(Some(i));
            }
        }
        i = scanner.advance(bytes, i);
    }

    Ok(None)
}

pub(crate) fn parse_for_in_of_var(raw: &str) -> Result<String> {
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

pub(crate) fn parse_for_clause_stmts(src: &str) -> Result<Vec<Stmt>> {
    let src = src.trim();
    if src.is_empty() {
        return Ok(Vec::new());
    }

    for keyword in ["const", "let", "var"] {
        let Some(after) = src.strip_prefix(keyword) else {
            continue;
        };
        if after.as_bytes().first().is_some_and(|b| is_ident_char(*b)) {
            continue;
        }
        let rest = after.trim_start();
        if rest.is_empty() {
            return Err(Error::ScriptParse(format!(
                "unsupported for-loop clause: {src}"
            )));
        }
        let parts = split_top_level_by_char(rest, b',');
        if parts.iter().any(|part| part.trim().is_empty()) {
            return Err(Error::ScriptParse(format!(
                "unsupported for-loop clause: {src}"
            )));
        }

        let mut out = Vec::with_capacity(parts.len());
        for part in parts {
            let decl_src = format!("{keyword} {}", part.trim());
            let Some(parsed) = parse_var_decl(&decl_src)? else {
                return Err(Error::ScriptParse(format!(
                    "unsupported for-loop clause: {src}"
                )));
            };
            out.push(parsed);
        }
        return Ok(out);
    }

    let parts = split_top_level_by_char(src, b',');
    if parts.iter().any(|part| part.trim().is_empty()) {
        return Err(Error::ScriptParse(format!(
            "unsupported for-loop clause: {src}"
        )));
    }

    let mut out = Vec::with_capacity(parts.len());
    for part in parts {
        let part = part.trim();

        if let Some(parsed) = parse_var_assign(part)? {
            out.push(parsed);
            continue;
        }

        if let Some(parsed) = parse_for_update_stmt(part) {
            out.push(parsed);
            continue;
        }

        let expr = parse_expr(part)
            .map_err(|_| Error::ScriptParse(format!("unsupported for-loop clause: {src}")))?;
        out.push(Stmt::Expr(expr));
    }

    Ok(out)
}

pub(crate) fn parse_for_update_stmt(src: &str) -> Option<Stmt> {
    parse_update_stmt(src)
}

pub(crate) fn parse_update_stmt(stmt: &str) -> Option<Stmt> {
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
