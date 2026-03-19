use super::*;

pub(crate) fn parse_regex_literal_expr(src: &str) -> Result<Option<(String, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some((pattern, flags)) = parse_regex_literal_from_cursor(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((pattern, flags)))
}

pub(crate) fn parse_regex_literal_from_cursor(
    cursor: &mut Cursor<'_>,
) -> Result<Option<(String, String)>> {
    cursor.skip_ws();
    if cursor.peek() != Some(b'/') {
        return Ok(None);
    }
    let start = cursor.i;
    let bytes = cursor.bytes();
    let mut i = cursor.i + 1;
    let mut escaped = false;
    let mut in_class = false;

    while i < bytes.len() {
        let b = bytes[i];
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if b == b'\\' {
            escaped = true;
            i += 1;
            continue;
        }
        if b == b'[' && !in_class {
            in_class = true;
            i += 1;
            continue;
        }
        if b == b']' && in_class {
            in_class = false;
            i += 1;
            continue;
        }
        if b == b'/' && !in_class {
            break;
        }
        if b == b'\n' || b == b'\r' {
            return Err(Error::ScriptParse("unterminated regex literal".into()));
        }
        i += 1;
    }

    if i >= bytes.len() || bytes[i] != b'/' {
        return Err(Error::ScriptParse("unterminated regex literal".into()));
    }

    let pattern = cursor
        .src
        .get(start + 1..i)
        .ok_or_else(|| Error::ScriptParse("invalid regex literal".into()))?
        .to_string();
    i += 1;
    let flags_start = i;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }
    let flags = cursor
        .src
        .get(flags_start..i)
        .ok_or_else(|| Error::ScriptParse("invalid regex flags".into()))?
        .to_string();

    let info = Harness::analyze_regex_flags(&flags).map_err(Error::ScriptParse)?;
    Harness::compile_regex(&pattern, info).map_err(|err| {
        Error::ScriptParse(format!(
            "invalid regular expression: /{pattern}/{flags}: {err}"
        ))
    })?;

    cursor.i = i;
    Ok(Some((pattern, flags)))
}

pub(crate) fn parse_new_regexp_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(expr) = parse_new_regexp_expr_from_cursor(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}

pub(crate) fn parse_new_regexp_expr_from_cursor(cursor: &mut Cursor<'_>) -> Result<Option<Expr>> {
    let start = cursor.i;
    cursor.skip_ws();
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                cursor.i = start;
                return Ok(None);
            }
        }
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            cursor.i = start;
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("RegExp") {
        cursor.i = start;
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            cursor.i = start;
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        cursor.i = start;
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
            "RegExp supports up to two arguments".into(),
        ));
    }
    if !args.is_empty() && args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "RegExp pattern argument cannot be empty".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "RegExp flags argument cannot be empty".into(),
        ));
    }

    let pattern = if args.is_empty() {
        Box::new(Expr::String(String::new()))
    } else {
        Box::new(parse_expr(args[0].trim())?)
    };
    let flags = if args.len() == 2 {
        Some(Box::new(parse_expr(args[1].trim())?))
    } else {
        None
    };

    Ok(Some(Expr::RegexNew { pattern, flags }))
}

pub(crate) fn parse_regexp_static_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("RegExp") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if member != "escape" {
            return Ok(None);
        }

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() != 1 || args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "RegExp.escape requires exactly one argument".into(),
            ));
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::RegExpStaticMethod {
            method: RegExpStaticMethod::Escape,
            args: vec![parse_expr(args[0].trim())?],
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::RegExpConstructor))
}

pub(crate) fn parse_regex_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let (receiver, receiver_is_identifier) =
        if let Some((pattern, flags)) = parse_regex_literal_from_cursor(&mut cursor)? {
            (Expr::RegexLiteral { pattern, flags }, false)
        } else if let Some(expr) = parse_new_regexp_expr_from_cursor(&mut cursor)? {
            (expr, false)
        } else if let Some(name) = cursor.parse_identifier() {
            (Expr::Var(name), true)
        } else {
            return Ok(None);
        };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if !matches!(method.as_str(), "test" | "exec" | "toString") {
        return Ok(None);
    }
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    let input = match method.as_str() {
        "test" => {
            if args.len() != 1 {
                return Err(Error::ScriptParse(format!(
                    "RegExp.{} requires zero or one argument",
                    method
                )));
            }
            let arg = args[0].trim();
            if arg.is_empty() {
                Some(Box::new(Expr::Undefined))
            } else {
                Some(Box::new(parse_expr(arg)?))
            }
        }
        "exec" => {
            if args.len() != 1 {
                return Err(Error::ScriptParse(
                    "RegExp.exec requires zero or one argument".into(),
                ));
            }
            let arg = args[0].trim();
            if arg.is_empty() {
                Some(Box::new(Expr::Undefined))
            } else {
                Some(Box::new(parse_expr(arg)?))
            }
        }
        "toString" => {
            if !(args.len() == 1 && args[0].trim().is_empty()) {
                if receiver_is_identifier {
                    return Ok(None);
                }
                return Err(Error::ScriptParse(
                    "RegExp.toString does not take arguments".into(),
                ));
            }
            None
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let regex = Box::new(receiver);
    match method.as_str() {
        "test" => Ok(Some(Expr::RegexTest {
            regex,
            input: input.expect("validated"),
        })),
        "exec" => Ok(Some(Expr::RegexExec {
            regex,
            input: input.expect("validated"),
        })),
        "toString" => Ok(Some(Expr::RegexToString { regex })),
        _ => Ok(None),
    }
}
