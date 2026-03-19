use super::*;

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
        let is_top_level = scanner.is_top_level();

        match b {
            b';' if was_top_level => out.push(i),
            b'}' => {
                if is_top_level {
                    out.push(i);
                }
            }
            b'e' => {
                if was_top_level
                    && current + 4 <= bytes.len()
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

fn consume_simple_statement_len(src: &str) -> usize {
    let bytes = src.as_bytes();
    let mut scanner = JsLexScanner::new();
    let mut i = 0usize;

    while i < bytes.len() {
        let current = i;
        let b = bytes[current];
        let was_top_level = scanner.is_top_level();
        i = scanner.advance(bytes, i);
        if was_top_level && b == b';' {
            let mut cursor = Cursor::new(src);
            cursor.set_pos(i);
            cursor.skip_ws();
            return cursor.i;
        }
    }

    src.len()
}

fn consume_single_statement_len(src: &str) -> Result<usize> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.eof() {
        return Err(Error::ScriptParse("empty single statement".into()));
    }

    if cursor.peek() == Some(b'{') {
        let _ = cursor.read_balanced_block(b'{', b'}')?;
        cursor.skip_ws();
        cursor.consume_byte(b';');
        cursor.skip_ws();
        return Ok(cursor.i);
    }

    let statement_start = cursor.i;

    if consume_keyword(&mut cursor, "if") {
        cursor.skip_ws();
        let _ = cursor.read_balanced_block(b'(', b')')?;
        let then_len = consume_single_statement_len(&src[cursor.i..])?;
        cursor.set_pos(cursor.i + then_len);
        cursor.skip_ws();
        if consume_keyword(&mut cursor, "else") {
            cursor.skip_ws();
            let else_len = consume_single_statement_len(&src[cursor.i..])?;
            cursor.set_pos(cursor.i + else_len);
        }
        cursor.skip_ws();
        return Ok(cursor.i);
    }

    cursor.set_pos(statement_start);
    if consume_keyword(&mut cursor, "for") {
        cursor.skip_ws();
        if consume_keyword(&mut cursor, "await") {
            cursor.skip_ws();
        }
        let _ = cursor.read_balanced_block(b'(', b')')?;
        let body_len = consume_single_statement_len(&src[cursor.i..])?;
        cursor.set_pos(cursor.i + body_len);
        cursor.skip_ws();
        return Ok(cursor.i);
    }

    cursor.set_pos(statement_start);
    if consume_keyword(&mut cursor, "while") || consume_keyword(&mut cursor, "with") {
        cursor.skip_ws();
        let _ = cursor.read_balanced_block(b'(', b')')?;
        let body_len = consume_single_statement_len(&src[cursor.i..])?;
        cursor.set_pos(cursor.i + body_len);
        cursor.skip_ws();
        return Ok(cursor.i);
    }

    cursor.set_pos(statement_start);
    if consume_keyword(&mut cursor, "do") {
        cursor.skip_ws();
        let body_len = consume_single_statement_len(&src[cursor.i..])?;
        cursor.set_pos(cursor.i + body_len);
        cursor.skip_ws();
        if !consume_keyword(&mut cursor, "while") {
            return Err(Error::ScriptParse("do statement requires while".into()));
        }
        cursor.skip_ws();
        let _ = cursor.read_balanced_block(b'(', b')')?;
        cursor.skip_ws();
        cursor.consume_byte(b';');
        cursor.skip_ws();
        return Ok(cursor.i);
    }

    cursor.set_pos(statement_start);
    if consume_keyword(&mut cursor, "switch") {
        cursor.skip_ws();
        let _ = cursor.read_balanced_block(b'(', b')')?;
        cursor.skip_ws();
        if cursor.peek() != Some(b'{') {
            return Err(Error::ScriptParse(
                "switch statement requires a block".into(),
            ));
        }
        let _ = cursor.read_balanced_block(b'{', b'}')?;
        cursor.skip_ws();
        cursor.consume_byte(b';');
        cursor.skip_ws();
        return Ok(cursor.i);
    }

    cursor.set_pos(statement_start);
    if consume_keyword(&mut cursor, "try") {
        cursor.skip_ws();
        if cursor.peek() != Some(b'{') {
            return Err(Error::ScriptParse("try statement requires a block".into()));
        }
        let _ = cursor.read_balanced_block(b'{', b'}')?;
        cursor.skip_ws();
        if consume_keyword(&mut cursor, "catch") {
            cursor.skip_ws();
            if cursor.peek() == Some(b'(') {
                let _ = cursor.read_balanced_block(b'(', b')')?;
                cursor.skip_ws();
            }
            if cursor.peek() != Some(b'{') {
                return Err(Error::ScriptParse("catch clause requires a block".into()));
            }
            let _ = cursor.read_balanced_block(b'{', b'}')?;
            cursor.skip_ws();
        }
        if consume_keyword(&mut cursor, "finally") {
            cursor.skip_ws();
            if cursor.peek() != Some(b'{') {
                return Err(Error::ScriptParse("finally clause requires a block".into()));
            }
            let _ = cursor.read_balanced_block(b'{', b'}')?;
            cursor.skip_ws();
        }
        cursor.consume_byte(b';');
        cursor.skip_ws();
        return Ok(cursor.i);
    }

    Ok(statement_start + consume_simple_statement_len(&src[statement_start..]))
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

    if let Ok(then_end) = consume_single_statement_len(tail) {
        if let Some(candidate_then) = tail.get(..then_end) {
            let candidate_then = candidate_then.trim_end();
            let rest = tail.get(then_end..).unwrap_or_default().trim_start();
            if !candidate_then.is_empty() {
                if rest.is_empty() {
                    then_raw = Some(candidate_then);
                    else_raw = None;
                } else if let Some(candidate_else_src) = strip_else_prefix(rest) {
                    if let Ok(else_end) = consume_single_statement_len(candidate_else_src) {
                        let candidate_else = candidate_else_src
                            .get(..else_end)
                            .unwrap_or_default()
                            .trim_end();
                        let extra = candidate_else_src
                            .get(else_end..)
                            .unwrap_or_default()
                            .trim_start();
                        if !candidate_else.is_empty() && extra.is_empty() {
                            then_raw = Some(candidate_then);
                            else_raw = Some(candidate_else);
                        }
                    }
                }
            }
        }
    }

    if then_raw.is_none() && tail.starts_with('{') {
        let mut branch_cursor = Cursor::new(tail);
        if branch_cursor.read_balanced_block(b'{', b'}').is_ok() {
            branch_cursor.skip_ws();
            branch_cursor.consume_byte(b';');
            branch_cursor.skip_ws();

            let candidate_then = tail[..branch_cursor.i].trim_end();
            let rest = tail[branch_cursor.i..].trim_start();
            if parse_if_branch(candidate_then).is_ok() {
                if rest.is_empty() {
                    then_raw = Some(candidate_then);
                    else_raw = None;
                } else if let Some(candidate_else) = strip_else_prefix(rest) {
                    match parse_if_branch(candidate_else) {
                        Ok(_) => {
                            then_raw = Some(candidate_then);
                            else_raw = Some(candidate_else);
                        }
                        Err(err) => branch_error = Some(err),
                    }
                }
            }
        }
    }

    for end in collect_top_level_if_branch_candidate_ends(tail) {
        if then_raw.is_some() {
            break;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_if_stmt_accepts_else_if_branch_with_array_destructure_assignment() {
        let stmt = r#"if (action === "duplicate") {
              state.rows.splice(index + 1, 0, createRow(state.rows[index]));
            } else if (action === "delete") {
              if (state.rows.length === 1) {
                state.rows[0] = createRow();
              } else {
                state.rows.splice(index, 1);
              }
            } else if (action === "up" && index > 0) {
              [state.rows[index - 1], state.rows[index]] = [state.rows[index], state.rows[index - 1]];
            } else if (action === "down" && index < state.rows.length - 1) {
              [state.rows[index + 1], state.rows[index]] = [state.rows[index], state.rows[index + 1]];
            }"#;

        let parsed = parse_if_stmt(stmt).expect("parser should not fail");
        assert!(parsed.is_some(), "expected if statement to parse");
    }

    #[test]
    fn parse_if_stmt_accepts_else_if_pair_with_array_destructure_assignment() {
        let stmt = r#"if (action === "up" && index > 0) {
              [state.rows[index - 1], state.rows[index]] = [state.rows[index], state.rows[index - 1]];
            } else if (action === "down" && index < state.rows.length - 1) {
              [state.rows[index + 1], state.rows[index]] = [state.rows[index], state.rows[index + 1]];
            }"#;

        let parsed = parse_if_stmt(stmt).expect("parser should not fail");
        assert!(parsed.is_some(), "expected if statement to parse");
    }

    #[test]
    fn parse_if_stmt_accepts_nested_if_block_before_else_if_pair() {
        let stmt = r#"if (action === "delete") {
              if (state.rows.length === 1) {
                state.rows[0] = createRow();
              } else {
                state.rows.splice(index, 1);
              }
            } else if (action === "up" && index > 0) {
              [state.rows[index - 1], state.rows[index]] = [state.rows[index], state.rows[index - 1]];
            } else if (action === "down" && index < state.rows.length - 1) {
              [state.rows[index + 1], state.rows[index]] = [state.rows[index], state.rows[index + 1]];
            }"#;

        let parsed = parse_if_stmt(stmt).expect("parser should not fail");
        assert!(parsed.is_some(), "expected if statement to parse");
    }

    #[test]
    fn parse_if_stmt_keeps_dangling_else_attached_to_inner_if() {
        let stmt = r#"if (outer) if (inner) work(); else recover();"#;

        let parsed = parse_if_stmt(stmt).expect("parser should not fail");
        match parsed {
            Some(Stmt::If {
                then_stmts,
                else_stmts,
                ..
            }) => {
                assert!(else_stmts.is_empty(), "outer if should not have else");
                match then_stmts.as_slice() {
                    [Stmt::If { else_stmts, .. }] => {
                        assert!(
                            !else_stmts.is_empty(),
                            "inner if should retain the dangling else branch"
                        );
                    }
                    other => panic!("expected inner if branch, got {other:?}"),
                }
            }
            other => panic!("expected if statement, got {other:?}"),
        }
    }
}
