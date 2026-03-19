use super::*;

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
        "getOwnPropertyDescriptor"
            | "defineProperty"
            | "getOwnPropertyNames"
            | "getOwnPropertySymbols"
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
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    let expr = match method.as_str() {
        "getOwnPropertyDescriptor" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.getOwnPropertyDescriptor requires exactly two arguments".into(),
                ));
            }
            Expr::ObjectGetOwnPropertyDescriptor {
                object: Box::new(parse_expr(args[0].trim())?),
                key: Box::new(parse_expr(args[1].trim())?),
            }
        }
        "defineProperty" => {
            if args.len() != 3
                || args[0].trim().is_empty()
                || args[1].trim().is_empty()
                || args[2].trim().is_empty()
            {
                return Err(Error::ScriptParse(
                    "Object.defineProperty requires exactly three arguments".into(),
                ));
            }
            Expr::ObjectDefineProperty {
                object: Box::new(parse_expr(args[0].trim())?),
                key: Box::new(parse_expr(args[1].trim())?),
                descriptor: Box::new(parse_expr(args[2].trim())?),
            }
        }
        "getOwnPropertyNames" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Object.getOwnPropertyNames requires exactly one argument".into(),
                ));
            }
            Expr::ObjectGetOwnPropertyNames(Box::new(parse_expr(args[0].trim())?))
        }
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

pub(crate) fn parse_reflect_static_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") || cursor.consume_ascii("self") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Reflect") {
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
    let method = if cursor.consume_ascii("set") {
        "set"
    } else if cursor.consume_ascii("ownKeys") {
        "ownKeys"
    } else {
        return Ok(None);
    };
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    match method {
        "set" => {
            if args.len() < 3 || args.len() > 4 {
                return Err(Error::ScriptParse(
                    "Reflect.set requires three or four arguments".into(),
                ));
            }
            if args.iter().any(|arg| arg.trim().is_empty()) {
                return Err(Error::ScriptParse(
                    "Reflect.set arguments cannot be empty".into(),
                ));
            }

            Ok(Some(Expr::ReflectSet {
                target: Box::new(parse_expr(args[0].trim())?),
                key: Box::new(parse_expr(args[1].trim())?),
                value: Box::new(parse_expr(args[2].trim())?),
                receiver: args
                    .get(3)
                    .map(|receiver| parse_expr(receiver.trim()))
                    .transpose()?
                    .map(Box::new),
            }))
        }
        "ownKeys" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Reflect.ownKeys requires exactly one argument".into(),
                ));
            }
            Ok(Some(Expr::ReflectOwnKeys(Box::new(parse_expr(
                args[0].trim(),
            )?))))
        }
        _ => Ok(None),
    }
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
