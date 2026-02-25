use super::*;
use crate::core_impl::parser::parser_stmt::{
    consume_keyword, parse_callback_parameter_list, prepend_callback_param_prologue_stmts,
};
pub(crate) fn parse_object_literal_expr(src: &str) -> Result<Option<Vec<ObjectLiteralEntry>>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if cursor.peek() != Some(b'{') {
        return Ok(None);
    }

    let entries_src = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let mut entries = split_top_level_by_char(&entries_src, b',');
    while entries.len() > 1 && entries.last().is_some_and(|entry| entry.trim().is_empty()) {
        entries.pop();
    }
    if entries.len() == 1 && entries[0].trim().is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut out = Vec::with_capacity(entries.len());
    let mut seen_proto_setter = false;
    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() {
            return Err(Error::ScriptParse(
                "object literal does not support empty entries".into(),
            ));
        }

        if let Some(rest) = entry.strip_prefix("...") {
            let rest = rest.trim();
            if rest.is_empty() {
                return Err(Error::ScriptParse(
                    "object spread source cannot be empty".into(),
                ));
            }
            out.push(ObjectLiteralEntry::Spread(parse_expr(rest)?));
            continue;
        }

        if let Some((key, handler)) = parse_object_literal_getter_entry(entry)? {
            out.push(ObjectLiteralEntry::Getter(key, handler));
            continue;
        }

        if let Some((key, handler)) = parse_object_literal_setter_entry(entry)? {
            out.push(ObjectLiteralEntry::Setter(key, handler));
            continue;
        }

        if let Some((key, value)) = parse_object_literal_method_entry(entry)? {
            out.push(ObjectLiteralEntry::Pair(key, value));
            continue;
        }

        let Some(colon) = find_first_top_level_colon(entry) else {
            if is_ident(entry) {
                out.push(ObjectLiteralEntry::Pair(
                    ObjectLiteralKey::Static(entry.to_string()),
                    Expr::Var(entry.to_string()),
                ));
                continue;
            }
            return Err(Error::ScriptParse(
                "object literal entry must use key: value".into(),
            ));
        };

        let key_src = entry[..colon].trim();
        let value_src = entry[colon + 1..].trim();
        if value_src.is_empty() {
            return Err(Error::ScriptParse(
                "object literal value cannot be empty".into(),
            ));
        }

        let key = if key_src.starts_with('[') && key_src.ends_with(']') && key_src.len() >= 2 {
            let computed_src = key_src[1..key_src.len() - 1].trim();
            if computed_src.is_empty() {
                return Err(Error::ScriptParse(
                    "object literal computed key cannot be empty".into(),
                ));
            }
            ObjectLiteralKey::Computed(Box::new(parse_expr(computed_src)?))
        } else if (key_src.starts_with('\'') && key_src.ends_with('\''))
            || (key_src.starts_with('"') && key_src.ends_with('"'))
        {
            ObjectLiteralKey::Static(parse_string_literal_exact(key_src)?)
        } else if is_ident(key_src) {
            ObjectLiteralKey::Static(key_src.to_string())
        } else if let Some(numeric_key) = parse_numeric_literal(key_src)? {
            match numeric_key {
                Expr::Number(_) | Expr::Float(_) => {
                    ObjectLiteralKey::Computed(Box::new(numeric_key))
                }
                _ => {
                    return Err(Error::ScriptParse(
                        "object literal key must be identifier, string literal, numeric literal, or computed key".into(),
                    ));
                }
            }
        } else {
            return Err(Error::ScriptParse(
                "object literal key must be identifier, string literal, numeric literal, or computed key".into(),
            ));
        };

        let value = parse_expr(value_src)?;
        if matches!(&key, ObjectLiteralKey::Static(key) if key == "__proto__") {
            if seen_proto_setter {
                return Err(Error::ScriptParse(
                    "duplicate __proto__ fields are not allowed in object literals".into(),
                ));
            }
            seen_proto_setter = true;
            out.push(ObjectLiteralEntry::ProtoSetter(value));
            continue;
        }

        out.push(ObjectLiteralEntry::Pair(key, value));
    }

    Ok(Some(out))
}

fn parse_object_literal_getter_entry(
    entry: &str,
) -> Result<Option<(ObjectLiteralKey, ScriptHandler)>> {
    let mut cursor = Cursor::new(entry);
    cursor.skip_ws();
    if !consume_keyword(&mut cursor, "get") {
        return Ok(None);
    }

    cursor.skip_ws();
    if cursor.eof() || cursor.peek() == Some(b':') {
        return Ok(None);
    }

    let key = if cursor.peek() == Some(b'[') {
        let computed_src = cursor.read_balanced_block(b'[', b']')?;
        let computed_src = computed_src.trim();
        if computed_src.is_empty() {
            return Err(Error::ScriptParse(
                "object literal getter computed key cannot be empty".into(),
            ));
        }
        ObjectLiteralKey::Computed(Box::new(parse_expr(computed_src)?))
    } else if cursor.peek().is_some_and(|b| b == b'\'' || b == b'"') {
        let key = cursor.parse_string_literal()?;
        ObjectLiteralKey::Static(key)
    } else if let Some(name) = cursor.parse_identifier() {
        ObjectLiteralKey::Static(name)
    } else {
        return Ok(None);
    };

    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let params_src = cursor.read_balanced_block(b'(', b')')?;
    if !params_src.trim().is_empty() {
        return Err(Error::ScriptParse(
            "object literal getter must not have parameters".into(),
        ));
    }
    cursor.skip_ws();

    if cursor.peek() != Some(b'{') {
        return Err(Error::ScriptParse(
            "object literal getter requires a body".into(),
        ));
    }
    let body_src = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(
            "unsupported object literal getter syntax".into(),
        ));
    }

    Ok(Some((
        key,
        ScriptHandler {
            params: Vec::new(),
            stmts: parse_block_statements(&body_src)?,
        },
    )))
}

fn parse_object_literal_setter_entry(
    entry: &str,
) -> Result<Option<(ObjectLiteralKey, ScriptHandler)>> {
    let mut cursor = Cursor::new(entry);
    cursor.skip_ws();
    if !consume_keyword(&mut cursor, "set") {
        return Ok(None);
    }

    cursor.skip_ws();
    if cursor.eof() || cursor.peek() == Some(b':') {
        return Ok(None);
    }

    let key = if cursor.peek() == Some(b'[') {
        let computed_src = cursor.read_balanced_block(b'[', b']')?;
        let computed_src = computed_src.trim();
        if computed_src.is_empty() {
            return Err(Error::ScriptParse(
                "object literal setter computed key cannot be empty".into(),
            ));
        }
        ObjectLiteralKey::Computed(Box::new(parse_expr(computed_src)?))
    } else if cursor.peek().is_some_and(|b| b == b'\'' || b == b'"') {
        let key = cursor.parse_string_literal()?;
        ObjectLiteralKey::Static(key)
    } else if let Some(name) = cursor.parse_identifier() {
        ObjectLiteralKey::Static(name)
    } else {
        return Ok(None);
    };

    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let params_src = cursor.read_balanced_block(b'(', b')')?;
    let parsed_params =
        parse_callback_parameter_list(&params_src, 1, "object literal setter parameters")?;
    if parsed_params.params.len() != 1 || parsed_params.params[0].is_rest {
        return Err(Error::ScriptParse(
            "object literal setter must have exactly one parameter".into(),
        ));
    }
    cursor.skip_ws();

    if cursor.peek() != Some(b'{') {
        return Err(Error::ScriptParse(
            "object literal setter requires a body".into(),
        ));
    }
    let body_src = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(
            "unsupported object literal setter syntax".into(),
        ));
    }

    let stmts = prepend_callback_param_prologue_stmts(
        parse_block_statements(&body_src)?,
        &parsed_params.prologue,
    )?;

    Ok(Some((
        key,
        ScriptHandler {
            params: parsed_params.params,
            stmts,
        },
    )))
}

fn parse_object_literal_method_entry(entry: &str) -> Result<Option<(ObjectLiteralKey, Expr)>> {
    let mut cursor = Cursor::new(entry);
    cursor.skip_ws();
    let start = cursor.pos();

    let mut is_async = false;
    if consume_keyword(&mut cursor, "async") {
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') && cursor.peek() != Some(b':') {
            is_async = true;
        } else {
            cursor.set_pos(start);
        }
    }

    cursor.skip_ws();
    let is_generator = cursor.consume_byte(b'*');
    if is_generator {
        cursor.skip_ws();
    }

    let key = if cursor.peek() == Some(b'[') {
        let computed_src = cursor.read_balanced_block(b'[', b']')?;
        let computed_src = computed_src.trim();
        if computed_src.is_empty() {
            return Err(Error::ScriptParse(
                "object literal computed method key cannot be empty".into(),
            ));
        }
        ObjectLiteralKey::Computed(Box::new(parse_expr(computed_src)?))
    } else if cursor.peek().is_some_and(|b| b == b'\'' || b == b'"') {
        ObjectLiteralKey::Static(cursor.parse_string_literal()?)
    } else if let Some(name) = cursor.parse_identifier() {
        ObjectLiteralKey::Static(name)
    } else {
        return Ok(None);
    };

    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let params_src = cursor.read_balanced_block(b'(', b')')?;
    let parsed_params =
        parse_callback_parameter_list(&params_src, usize::MAX, "object method parameters")?;
    cursor.skip_ws();
    if cursor.peek() != Some(b'{') {
        return Err(Error::ScriptParse(
            "object method definition requires a body".into(),
        ));
    }
    let body_src = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(
            "unsupported object method syntax".into(),
        ));
    }

    let stmts = prepend_callback_param_prologue_stmts(
        parse_block_statements(&body_src)?,
        &parsed_params.prologue,
    )?;

    Ok(Some((
        key,
        Expr::Function {
            handler: ScriptHandler {
                params: parsed_params.params,
                stmts,
            },
            name: None,
            is_async,
            is_generator,
            is_arrow: false,
            is_method: true,
        },
    )))
}

pub(crate) fn find_first_top_level_colon(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() && bytes[i] == b':' {
            return Some(i);
        }
        i = scanner.advance(bytes, i);
    }

    None
}

pub(crate) fn parse_object_static_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Object") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Object supports zero or one argument".into(),
            ));
        }
        if args.len() == 1 && args[0].trim().is_empty() {
            return Err(Error::ScriptParse("Object argument cannot be empty".into()));
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ObjectConstruct {
            value: if args.is_empty() {
                None
            } else {
                Some(Box::new(parse_expr(args[0].trim())?))
            },
        }));
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if !matches!(
        method.as_str(),
        "getOwnPropertySymbols"
            | "keys"
            | "values"
            | "entries"
            | "hasOwn"
            | "getPrototypeOf"
            | "freeze"
    ) {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    let expr = match method.as_str() {
        "getOwnPropertySymbols" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.getOwnPropertySymbols requires exactly one argument".into(),
                ));
            }
            Expr::ObjectGetOwnPropertySymbols(Box::new(parse_expr(args[0].trim())?))
        }
        "keys" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.keys requires exactly one argument".into(),
                ));
            }
            Expr::ObjectKeys(Box::new(parse_expr(args[0].trim())?))
        }
        "values" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.values requires exactly one argument".into(),
                ));
            }
            Expr::ObjectValues(Box::new(parse_expr(args[0].trim())?))
        }
        "entries" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.entries requires exactly one argument".into(),
                ));
            }
            Expr::ObjectEntries(Box::new(parse_expr(args[0].trim())?))
        }
        "hasOwn" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.hasOwn requires exactly two arguments".into(),
                ));
            }
            Expr::ObjectHasOwn {
                object: Box::new(parse_expr(args[0].trim())?),
                key: Box::new(parse_expr(args[1].trim())?),
            }
        }
        "getPrototypeOf" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.getPrototypeOf requires exactly one argument".into(),
                ));
            }
            Expr::ObjectGetPrototypeOf(Box::new(parse_expr(args[0].trim())?))
        }
        "freeze" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.freeze requires exactly one argument".into(),
                ));
            }
            Expr::ObjectFreeze(Box::new(parse_expr(args[0].trim())?))
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}

pub(crate) fn parse_object_prototype_has_own_property_call_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Object") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("prototype") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("hasOwnProperty") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("call") {
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
    if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Object.prototype.hasOwnProperty.call requires exactly two arguments".into(),
        ));
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::ObjectHasOwn {
        object: Box::new(parse_expr(args[0].trim())?),
        key: Box::new(parse_expr(args[1].trim())?),
    }))
}

pub(crate) fn parse_object_has_own_property_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if matches!(target.as_str(), "document" | "window") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("hasOwnProperty") {
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
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "hasOwnProperty requires exactly one argument".into(),
        ));
    }
    let key = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::ObjectHasOwnProperty {
        target,
        key: Box::new(key),
    }))
}

pub(crate) fn parse_object_get_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    let mut path = Vec::new();
    loop {
        cursor.skip_ws();
        let Some(key) = cursor.parse_identifier() else {
            return Ok(None);
        };
        path.push(key);
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            break;
        }
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    if target == "import" && path.first().is_some_and(|key| key == "meta") {
        return Ok(None);
    }
    if target == "new" && path.first().is_some_and(|key| key == "target") {
        return Ok(None);
    }

    if path.len() == 1 {
        return Ok(Some(Expr::ObjectGet {
            target,
            key: path.remove(0),
        }));
    }
    Ok(Some(Expr::ObjectPathGet { target, path }))
}
