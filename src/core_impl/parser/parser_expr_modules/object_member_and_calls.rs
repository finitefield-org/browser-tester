use super::*;

pub(crate) fn parse_parse_int_expr(src: &str) -> Result<Option<(Expr, Option<Expr>)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("parseInt") {
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
            "parseInt requires one or two arguments".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "parseInt radix argument cannot be empty".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    let radix = if args.len() == 2 {
        Some(parse_expr(args[1].trim())?)
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((value, radix)))
}

pub(crate) fn parse_parse_float_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("parseFloat") {
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
            "parseFloat requires exactly one argument".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

pub(crate) fn parse_json_parse_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("JSON") {
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
    if !cursor.consume_ascii("parse") {
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
            "JSON.parse requires exactly one argument".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

pub(crate) fn parse_json_stringify_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("JSON") {
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
    if !cursor.consume_ascii("stringify") {
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
            "JSON.stringify requires exactly one argument".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

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
        } else {
            return Err(Error::ScriptParse(
                "object literal key must be identifier, string literal, or computed key".into(),
            ));
        };

        out.push(ObjectLiteralEntry::Pair(key, parse_expr(value_src)?));
    }

    Ok(Some(out))
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

    if path.len() == 1 {
        return Ok(Some(Expr::ObjectGet {
            target,
            key: path.remove(0),
        }));
    }
    Ok(Some(Expr::ObjectPathGet { target, path }))
}

pub(crate) fn parse_call_args<'a>(
    args_src: &'a str,
    empty_err: &'static str,
) -> Result<Vec<&'a str>> {
    if args_src.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut args = split_top_level_by_char(args_src, b',');
    if args.len() > 1 && args.last().is_some_and(|arg| arg.trim().is_empty()) {
        args.pop();
    }
    if args.iter().any(|arg| arg.trim().is_empty()) {
        return Err(Error::ScriptParse(empty_err.into()));
    }
    Ok(args)
}

pub(crate) fn parse_call_arg_expr(arg_src: &str) -> Result<Expr> {
    let arg_src = arg_src.trim();
    if let Some(rest) = arg_src.strip_prefix("...") {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(Error::ScriptParse(
                "call spread source cannot be empty".into(),
            ));
        }
        Ok(Expr::Spread(Box::new(parse_expr(rest)?)))
    } else {
        parse_expr(arg_src)
    }
}

pub(crate) fn parse_member_call_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let dots = collect_top_level_char_positions(src, b'.');
    for dot in dots.into_iter().rev() {
        let Some(mut base_src) = src.get(..dot) else {
            continue;
        };
        base_src = base_src.trim_end();
        let mut optional = false;
        if let Some(stripped) = base_src.strip_suffix('?') {
            optional = true;
            base_src = stripped.trim_end();
        }
        let base_src = base_src.trim();
        if base_src.is_empty() {
            continue;
        }

        let Some(tail_src) = src.get(dot + 1..) else {
            continue;
        };
        let tail_src = tail_src.trim();
        let mut cursor = Cursor::new(tail_src);
        let Some(member) = cursor.parse_identifier() else {
            continue;
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            continue;
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        cursor.skip_ws();
        if !cursor.eof() {
            continue;
        }

        let args = parse_call_args(&args_src, "member call arguments cannot be empty")?;
        let mut parsed_args = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            parsed_args.push(parse_call_arg_expr(arg)?);
        }

        return Ok(Some(Expr::MemberCall {
            target: Box::new(parse_expr(base_src)?),
            member,
            args: parsed_args,
            optional,
        }));
    }
    Ok(None)
}

pub(crate) fn parse_member_get_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let dots = collect_top_level_char_positions(src, b'.');
    for dot in dots.into_iter().rev() {
        let Some(mut base_src) = src.get(..dot) else {
            continue;
        };
        let Some(tail_src) = src.get(dot + 1..) else {
            continue;
        };
        let tail_src = tail_src.trim();
        if tail_src.is_empty() {
            continue;
        }

        let mut cursor = Cursor::new(tail_src);
        let Some(member) = cursor.parse_identifier() else {
            continue;
        };
        cursor.skip_ws();
        if !cursor.eof() {
            continue;
        }

        let mut optional = false;
        base_src = base_src.trim_end();
        if let Some(stripped) = base_src.strip_suffix('?') {
            optional = true;
            base_src = stripped.trim_end();
        }
        if base_src.is_empty() {
            continue;
        }

        return Ok(Some(Expr::MemberGet {
            target: Box::new(parse_expr(base_src)?),
            member,
            optional,
        }));
    }
    Ok(None)
}

pub(crate) fn parse_member_index_get_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let bytes = src.as_bytes();
    let mut brackets = Vec::new();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();
    while i < bytes.len() {
        if scanner.is_top_level() && bytes[i] == b'[' {
            brackets.push(i);
        }
        i = scanner.advance(bytes, i);
    }

    for open in brackets.into_iter().rev() {
        let Some(mut base_src) = src.get(..open) else {
            continue;
        };
        base_src = base_src.trim_end();
        let mut optional = false;
        if let Some(stripped) = base_src.strip_suffix("?.") {
            optional = true;
            base_src = stripped.trim_end();
        } else if let Some(stripped) = base_src.strip_suffix('?') {
            optional = true;
            base_src = stripped.trim_end();
        }
        let base_src = base_src.trim();
        if base_src.is_empty() {
            continue;
        }

        let Some(rest) = src.get(open..) else {
            continue;
        };
        let mut cursor = Cursor::new(rest);
        let index_src = cursor.read_balanced_block(b'[', b']')?;
        cursor.skip_ws();
        if !cursor.eof() {
            continue;
        }

        let index = index_src.trim();
        if index.is_empty() {
            return Err(Error::ScriptParse("array index cannot be empty".into()));
        }

        if (index.starts_with('\'') && index.ends_with('\''))
            || (index.starts_with('"') && index.ends_with('"'))
        {
            return Ok(Some(Expr::MemberGet {
                target: Box::new(parse_expr(base_src)?),
                member: parse_string_literal_exact(index)?,
                optional,
            }));
        }
        if index.as_bytes().iter().all(|b| b.is_ascii_digit()) {
            return Ok(Some(Expr::MemberGet {
                target: Box::new(parse_expr(base_src)?),
                member: index.to_string(),
                optional,
            }));
        }

        return Ok(Some(Expr::IndexGet {
            target: Box::new(parse_expr(base_src)?),
            index: Box::new(parse_expr(index)?),
            optional,
        }));
    }
    Ok(None)
}

pub(crate) fn parse_function_call_expr(src: &str) -> Result<Option<Expr>> {
    let parse_args = |args_src: &str| -> Result<Vec<Expr>> {
        let args = parse_call_args(args_src, "function call arguments cannot be empty")?;

        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            parsed.push(parse_call_arg_expr(arg)?);
        }
        Ok(parsed)
    };

    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if let Some(target) = cursor.parse_identifier() {
        cursor.skip_ws();
        if cursor.peek() == Some(b'(') {
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            let parsed = parse_args(&args_src)?;

            cursor.skip_ws();
            if cursor.eof() {
                return Ok(Some(Expr::FunctionCall {
                    target,
                    args: parsed,
                }));
            }
            return Ok(None);
        }
    }

    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let target_src = cursor.read_balanced_block(b'(', b')')?;
    let target_src = target_src.trim();
    if target_src.is_empty() {
        return Ok(None);
    }

    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = parse_args(&args_src)?;

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::Call {
        target: Box::new(parse_expr(target_src)?),
        args,
    }))
}

pub(crate) fn parse_fetch_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "fetch", "fetch requires exactly one argument")
}

pub(crate) fn parse_match_media_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("matchMedia") {
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
            "matchMedia requires exactly one argument".into(),
        ));
    }
    let query = parse_expr(args[0].trim())?;

    cursor.skip_ws();
    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(prop_name) = cursor.parse_identifier() else {
            return Ok(None);
        };
        let prop = match prop_name.as_str() {
            "matches" => MatchMediaProp::Matches,
            "media" => MatchMediaProp::Media,
            _ => return Ok(None),
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MatchMediaProp {
            query: Box::new(query),
            prop,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::MatchMedia(Box::new(query))))
}
