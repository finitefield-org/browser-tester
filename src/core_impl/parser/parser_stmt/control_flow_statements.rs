use super::*;

pub(crate) fn parse_block_statements(body: &str) -> Result<Vec<Stmt>> {
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

pub(crate) fn parse_single_statement(stmt: &str) -> Result<Stmt> {
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
        return parse_block_statements(&body);
    }

    let single = trim_optional_trailing_semicolon(src);
    if single.is_empty() {
        return Err(Error::ScriptParse("empty single statement branch".into()));
    }
    Ok(vec![parse_single_statement(single)?])
}

pub(crate) fn trim_optional_trailing_semicolon(src: &str) -> &str {
    let mut trimmed = src.trim_end();
    if let Some(without) = trimmed.strip_suffix(';') {
        trimmed = without.trim_end();
    }
    trimmed
}

pub(crate) fn find_top_level_else_keyword(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() && bytes[i] == b'e' {
            if i + 4 <= bytes.len()
                && &bytes[i..i + 4] == b"else"
                && (i == 0 || !is_ident_char(bytes[i - 1]))
                && (i + 4 == bytes.len() || !is_ident_char(bytes[i + 4]))
            {
                return Some(i);
            }
        }
        i = scanner.advance(bytes, i);
    }

    None
}

pub(crate) fn is_ident_char(b: u8) -> bool {
    b == b'_' || b == b'$' || b.is_ascii_alphanumeric()
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

pub(crate) fn parse_throw_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported break statement: {stmt}"
        )));
    }
    Ok(Some(Stmt::Break))
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
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported continue statement: {stmt}"
        )));
    }
    Ok(Some(Stmt::Continue))
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
    let header_src = cursor.read_balanced_block(b'(', b')')?;
    if let Some((kind, item_var, iterable_src)) = parse_for_in_of_stmt(&header_src)? {
        let iterable = parse_expr(iterable_src.trim())?;
        cursor.skip_ws();
        let body_raw = cursor.src.get(cursor.i..).unwrap_or_default().trim();
        if body_raw.is_empty() {
            return Err(Error::ScriptParse(format!(
                "for statement has no body: {stmt}"
            )));
        }
        let body = parse_if_branch(body_raw)?;

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
    let body_raw = cursor.src.get(cursor.i..).unwrap_or_default().trim();
    if body_raw.is_empty() {
        return Err(Error::ScriptParse(format!(
            "for statement has no body: {stmt}"
        )));
    }
    let body = parse_if_branch(body_raw)?;

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

pub(crate) fn parse_for_clause_stmt(src: &str) -> Result<Option<Box<Stmt>>> {
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
