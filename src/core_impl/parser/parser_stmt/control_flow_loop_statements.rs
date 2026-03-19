use super::*;

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
    let mut body_prologue = Vec::new();

    let parsed_for = if is_for_await {
        if header_parts.len() != 1 {
            return Err(Error::ScriptParse(format!(
                "for await statement requires an of-clause: {stmt}"
            )));
        }
        let Some((kind, binding, iterable_src)) = parse_for_in_of_stmt(header_src)? else {
            return Err(Error::ScriptParse(format!(
                "for await statement requires an of-clause: {stmt}"
            )));
        };
        if kind != ForInOfKind::Of {
            return Err(Error::ScriptParse(format!(
                "for await statement only supports of: {stmt}"
            )));
        }
        body_prologue = binding.body_prologue;
        let iterable = parse_expr(iterable_src.trim())?;
        Stmt::ForAwaitOf {
            item_var: binding.item_var,
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
        let Some((kind, binding, iterable_src)) = parse_for_in_of_stmt(header_src)? else {
            return Err(Error::ScriptParse(format!(
                "unsupported for statement: {stmt}"
            )));
        };
        body_prologue = binding.body_prologue;
        let item_var = binding.item_var;
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
    let mut body = parse_if_branch(body_raw)?;
    if !body_prologue.is_empty() {
        let mut combined = body_prologue;
        combined.extend(body);
        body = combined;
    }

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

#[derive(Debug, Clone)]
struct ForInOfBinding {
    item_var: String,
    body_prologue: Vec<Stmt>,
}

fn parse_for_in_of_stmt(header: &str) -> Result<Option<(ForInOfKind, ForInOfBinding, &str)>> {
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

    let binding = parse_for_in_of_binding(left, header)?;
    Ok(Some((kind, binding, right)))
}

fn parse_for_in_of_binding(raw: &str, scope_src: &str) -> Result<ForInOfBinding> {
    let (decl_kind, binding_src) = parse_for_in_of_binding_prefix(raw);
    let binding_src = binding_src.trim();
    if binding_src.starts_with('[') {
        let item_var = fresh_for_in_of_binding_temp_name(scope_src);
        return Ok(ForInOfBinding {
            item_var: item_var.clone(),
            body_prologue: vec![Stmt::ArrayDestructureAssign {
                pattern: parse_array_destructure_assignment_pattern(binding_src)?,
                expr: Expr::Var(item_var),
                decl_kind,
            }],
        });
    }
    if binding_src.starts_with('{') {
        let item_var = fresh_for_in_of_binding_temp_name(scope_src);
        return Ok(ForInOfBinding {
            item_var: item_var.clone(),
            body_prologue: vec![Stmt::ObjectDestructureAssign {
                pattern: parse_object_destructure_assignment_pattern(binding_src)?,
                expr: Expr::Var(item_var),
                decl_kind,
            }],
        });
    }

    Ok(ForInOfBinding {
        item_var: parse_for_in_of_var(raw)?,
        body_prologue: Vec::new(),
    })
}

fn parse_for_in_of_binding_prefix(raw: &str) -> (Option<VarDeclKind>, &str) {
    let raw = raw.trim();
    for (keyword, kind) in [
        ("const", VarDeclKind::Const),
        ("let", VarDeclKind::Let),
        ("var", VarDeclKind::Var),
    ] {
        let Some(after) = raw.strip_prefix(keyword) else {
            continue;
        };
        if after.as_bytes().first().is_some_and(|b| is_ident_char(*b)) {
            continue;
        }
        return (Some(kind), after.trim_start());
    }
    (None, raw)
}

fn fresh_for_in_of_binding_temp_name(scope_src: &str) -> String {
    let mut index = 0usize;
    loop {
        let candidate = format!("__bt_for_in_of_binding_{index}");
        if !scope_src.contains(&candidate) {
            return candidate;
        }
        index += 1;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_for_stmt_accepts_for_of_with_array_destructuring_binding() {
        let stmt = r#"for (const [key, value] of entries) {
              output.push(key + ":" + value);
            }"#;

        let parsed = parse_for_stmt(stmt).expect("parser should not fail");
        match parsed {
            Some(Stmt::ForOf { item_var, body, .. }) => {
                assert!(
                    item_var.starts_with("__bt_for_in_of_binding_"),
                    "destructuring loop should lower through an internal temp binding"
                );
                assert!(
                    matches!(
                        body.first(),
                        Some(Stmt::ArrayDestructureAssign {
                            decl_kind: Some(VarDeclKind::Const),
                            ..
                        })
                    ),
                    "loop body should start with array destructuring declaration"
                );
            }
            other => panic!("expected for...of statement, got {other:?}"),
        }
    }

    #[test]
    fn parse_for_stmt_accepts_for_of_with_object_destructuring_binding() {
        let stmt = r#"for ({ value, unit: displayUnit } of rows) {
              output.push(value + displayUnit);
            }"#;

        let parsed = parse_for_stmt(stmt).expect("parser should not fail");
        match parsed {
            Some(Stmt::ForOf { item_var, body, .. }) => {
                assert!(
                    item_var.starts_with("__bt_for_in_of_binding_"),
                    "destructuring loop should lower through an internal temp binding"
                );
                assert!(
                    matches!(
                        body.first(),
                        Some(Stmt::ObjectDestructureAssign {
                            decl_kind: None,
                            ..
                        })
                    ),
                    "loop body should start with object destructuring assignment"
                );
            }
            other => panic!("expected for...of statement, got {other:?}"),
        }
    }
}
