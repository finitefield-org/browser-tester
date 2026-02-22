use super::*;

pub(crate) fn parse_new_date_expr(src: &str) -> Result<Option<Expr>> {
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

    if !cursor.consume_ascii("Date") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args: Vec<String> = if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args.into_iter().map(|arg| arg.to_string()).collect()
        }
    } else {
        Vec::new()
    };

    if args.len() > 1 {
        return Err(Error::ScriptParse(
            "new Date supports zero or one argument".into(),
        ));
    }

    let value = if args.len() == 1 {
        if args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "new Date argument cannot be empty".into(),
            ));
        }
        Some(Box::new(parse_expr(args[0].trim())?))
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::DateNew { value }))
}

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
    let callee = if cursor.peek() == Some(b'(') {
        let callee_src = cursor.read_balanced_block(b'(', b')')?;
        parse_expr(callee_src.trim())?
    } else if let Some(name) = cursor.parse_identifier() {
        if name != "GeneratorFunction" && name != "AsyncGeneratorFunction" {
            return Ok(None);
        }
        Expr::Var(name)
    } else {
        return Ok(None);
    };
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };
    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Error::ScriptParse(
                "constructor argument cannot be empty".into(),
            ));
        }
        parsed.push(parse_expr(arg)?);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::TypedArrayConstructWithCallee {
        callee: Box::new(callee),
        args: parsed,
        called_with_new: true,
    }))
}

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
        "test" | "exec" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(format!(
                    "RegExp.{} requires exactly one argument",
                    method
                )));
            }
            Some(Box::new(parse_expr(args[0].trim())?))
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

pub(crate) fn parse_date_now_expr(src: &str) -> Result<bool> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(false);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Date") {
        return Ok(false);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(false);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(false);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("now") {
        return Ok(false);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    Ok(cursor.eof())
}

pub(crate) fn parse_performance_now_expr(src: &str) -> Result<bool> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(false);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("performance") {
        return Ok(false);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(false);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(false);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("now") {
        return Ok(false);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(false);
        }
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    Ok(cursor.eof())
}

pub(crate) fn parse_date_static_args_expr(src: &str, method: &str) -> Result<Option<Vec<String>>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Date") {
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
    if !cursor.consume_ascii(method) {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',')
        .into_iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>();
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(args))
}

pub(crate) fn parse_date_parse_expr(src: &str) -> Result<Option<Expr>> {
    let Some(args) = parse_date_static_args_expr(src, "parse")? else {
        return Ok(None);
    };
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Date.parse requires exactly one argument".into(),
        ));
    }
    Ok(Some(parse_expr(args[0].trim())?))
}

pub(crate) fn parse_date_utc_expr(src: &str) -> Result<Option<Vec<Expr>>> {
    let Some(args) = parse_date_static_args_expr(src, "UTC")? else {
        return Ok(None);
    };

    if args.len() < 2 || args.len() > 7 {
        return Err(Error::ScriptParse(
            "Date.UTC requires between 2 and 7 arguments".into(),
        ));
    }

    let mut out = Vec::with_capacity(args.len());
    for arg in args {
        if arg.trim().is_empty() {
            return Err(Error::ScriptParse(
                "Date.UTC argument cannot be empty".into(),
            ));
        }
        out.push(parse_expr(arg.trim())?);
    }
    Ok(Some(out))
}
