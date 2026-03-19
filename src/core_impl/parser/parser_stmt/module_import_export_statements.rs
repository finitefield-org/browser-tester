use super::*;

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
