use super::*;

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
        if scanner.is_top_level()
            && (is_switch_clause_keyword_at(bytes, i, b"case")
                || is_switch_clause_keyword_at(bytes, i, b"default"))
        {
            return Some(i);
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
