use super::*;

pub(crate) fn parse_new_error_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("Error") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.eof() {
        return Ok(Some(Expr::String("Error".to_string())));
    }
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
    if args.len() > 2 {
        return Err(Error::ScriptParse(
            "Error constructor supports up to two arguments".into(),
        ));
    }
    if args.first().is_some_and(|arg| arg.trim().is_empty()) {
        return Err(Error::ScriptParse(
            "Error message argument cannot be empty".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Error options argument cannot be empty".into(),
        ));
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    if let Some(message) = args.first() {
        return Ok(Some(parse_expr(message.trim())?));
    }
    Ok(Some(Expr::String("Error".to_string())))
}

pub(crate) fn parse_new_callee_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    let rest = cursor.src[cursor.i..].trim();
    if rest.is_empty() {
        return Ok(None);
    }
    let grouped_callee = rest.starts_with('(');
    if rest.starts_with('.') {
        return Ok(None);
    }
    let rest_bytes = rest.as_bytes();
    if collect_top_level_char_positions(rest, b'.')
        .into_iter()
        .any(|dot| dot > 0 && rest_bytes[dot - 1] == b')' && !grouped_callee)
    {
        return Ok(None);
    }
    if collect_top_level_char_positions(rest, b'[')
        .into_iter()
        .any(|index| index > 0 && rest_bytes[index - 1] == b')' && !grouped_callee)
    {
        return Ok(None);
    }

    let mut callee_src = rest;
    let mut args_src = None;
    if rest.ends_with(')') {
        for open in collect_top_level_char_positions(rest, b'(')
            .into_iter()
            .rev()
        {
            let Some(candidate) = rest.get(open..) else {
                continue;
            };
            let mut arg_cursor = Cursor::new(candidate);
            let Ok(parsed_args_src) = arg_cursor.read_balanced_block(b'(', b')') else {
                continue;
            };
            arg_cursor.skip_ws();
            if !arg_cursor.eof() {
                continue;
            }
            let Some(prefix) = rest.get(..open) else {
                continue;
            };
            let prefix = prefix.trim();
            if prefix.is_empty() {
                continue;
            }
            callee_src = prefix;
            args_src = Some(parsed_args_src);
            break;
        }
    }

    if is_reserved_new_constructor_callee_root(callee_src) {
        return Ok(None);
    }

    let callee = parse_expr(callee_src)?;
    let parsed = if let Some(args_src) = args_src {
        let args = parse_call_args(&args_src, "constructor argument cannot be empty")?;
        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            parsed.push(parse_call_arg_expr(arg)?);
        }
        parsed
    } else {
        Vec::new()
    };

    Ok(Some(Expr::TypedArrayConstructWithCallee {
        callee: Box::new(callee),
        args: parsed,
        called_with_new: true,
    }))
}

fn new_callee_root_identifier(callee_src: &str) -> Option<String> {
    let mut cursor = Cursor::new(callee_src.trim());
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            return cursor.parse_identifier();
        }
        return Some("window".to_string());
    }
    cursor.parse_identifier()
}

fn is_reserved_new_constructor_callee_root(callee_src: &str) -> bool {
    new_callee_root_identifier(callee_src)
        .is_some_and(|name| is_reserved_new_constructor_name(name.as_str()))
}

pub(crate) fn is_reserved_new_constructor_name(name: &str) -> bool {
    matches!(
        name,
        "Date"
            | "RegExp"
            | "Function"
            | "Error"
            | "String"
            | "Number"
            | "BigInt"
            | "Boolean"
            | "Object"
            | "Blob"
            | "URL"
            | "URLSearchParams"
            | "FormData"
            | "Array"
            | "ArrayBuffer"
            | "Promise"
            | "Map"
            | "WeakMap"
            | "Set"
            | "WeakSet"
            | "Symbol"
            | "Intl"
            | "Int8Array"
            | "Uint8Array"
            | "Uint8ClampedArray"
            | "Int16Array"
            | "Uint16Array"
            | "Int32Array"
            | "Uint32Array"
            | "Float32Array"
            | "Float64Array"
            | "BigInt64Array"
            | "BigUint64Array"
            | "TypedArray"
    )
}

pub(crate) fn parse_new_function_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Function") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    if args.is_empty() {
        return Err(Error::ScriptParse(
            "new Function requires at least one argument".into(),
        ));
    }

    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Error::ScriptParse(
                "new Function arguments cannot be empty".into(),
            ));
        }
        parsed.push(parse_expr(arg)?);
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::FunctionConstructor { args: parsed }))
}
