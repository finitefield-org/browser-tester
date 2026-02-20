pub(super) fn parse_clipboard_base(cursor: &mut Cursor<'_>) -> bool {
    let start = cursor.pos();

    if cursor.consume_ascii("navigator") {
        if cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            if cursor.consume_ascii("clipboard")
                && cursor.peek().is_none_or(|ch| !is_ident_char(ch))
            {
                return true;
            }
        }
        cursor.set_pos(start);
    }

    if cursor.consume_ascii("window") {
        if cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if !cursor.consume_ascii("navigator") || cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if cursor.consume_ascii("clipboard") && cursor.peek().is_none_or(|ch| !is_ident_char(ch)) {
            return true;
        }
    }

    cursor.set_pos(start);
    false
}

pub(super) fn parse_new_date_expr(src: &str) -> Result<Option<Expr>> {
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

pub(super) fn parse_regex_literal_expr(src: &str) -> Result<Option<(String, String)>> {
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

pub(super) fn parse_regex_literal_from_cursor(
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

pub(super) fn parse_new_regexp_expr(src: &str) -> Result<Option<Expr>> {
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

pub(super) fn parse_new_regexp_expr_from_cursor(cursor: &mut Cursor<'_>) -> Result<Option<Expr>> {
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

pub(super) fn parse_regexp_static_expr(src: &str) -> Result<Option<Expr>> {
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

pub(super) fn parse_new_function_expr(src: &str) -> Result<Option<Expr>> {
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

pub(super) fn parse_regex_method_expr(src: &str) -> Result<Option<Expr>> {
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

pub(super) fn parse_date_now_expr(src: &str) -> Result<bool> {
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

pub(super) fn parse_performance_now_expr(src: &str) -> Result<bool> {
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

pub(super) fn parse_date_static_args_expr(src: &str, method: &str) -> Result<Option<Vec<String>>> {
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

pub(super) fn parse_date_parse_expr(src: &str) -> Result<Option<Expr>> {
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

pub(super) fn parse_date_utc_expr(src: &str) -> Result<Option<Vec<Expr>>> {
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

pub(super) fn parse_intl_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Intl") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if called_with_new && cursor.peek() == Some(b'(') {
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
                    "Intl constructor argument cannot be empty".into(),
                ));
            }
            parsed.push(parse_expr(arg)?);
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlConstruct { args: parsed }));
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if member == "Collator" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::Collator,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(collator_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if collator_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.Collator".to_string())));
            }

            if collator_member == "supportedLocalesOf" {
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
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.Collator.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.Collator.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::CollatorSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
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
                "Intl.Collator supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.Collator argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::Collator,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "DateTimeFormat" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::DateTimeFormat,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(dtf_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if dtf_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.DateTimeFormat".to_string())));
            }

            if dtf_member == "supportedLocalesOf" {
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
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DateTimeFormat.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DateTimeFormat.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::DateTimeFormatSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
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
                "Intl.DateTimeFormat supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.DateTimeFormat argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::DateTimeFormat,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "DisplayNames" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::DisplayNames,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(display_names_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if display_names_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.DisplayNames".to_string())));
            }

            if display_names_member == "supportedLocalesOf" {
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
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DisplayNames.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DisplayNames.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::DisplayNamesSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
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
                "Intl.DisplayNames supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.DisplayNames argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::DisplayNames,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "DurationFormat" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::DurationFormat,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(duration_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if duration_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.DurationFormat".to_string())));
            }

            if duration_member == "supportedLocalesOf" {
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
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DurationFormat.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.DurationFormat.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::DurationFormatSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
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
                "Intl.DurationFormat supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.DurationFormat argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::DurationFormat,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "ListFormat" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::ListFormat,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(list_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if list_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.ListFormat".to_string())));
            }

            if list_member == "supportedLocalesOf" {
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
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.ListFormat.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.ListFormat.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::ListFormatSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
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
                "Intl.ListFormat supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.ListFormat argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::ListFormat,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "PluralRules" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::PluralRules,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(plural_rules_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if plural_rules_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.PluralRules".to_string())));
            }

            if plural_rules_member == "supportedLocalesOf" {
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
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.PluralRules.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.PluralRules.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::PluralRulesSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
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
                "Intl.PluralRules supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.PluralRules argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::PluralRules,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "RelativeTimeFormat" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::RelativeTimeFormat,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(relative_time_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if relative_time_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.RelativeTimeFormat".to_string())));
            }

            if relative_time_member == "supportedLocalesOf" {
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
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.RelativeTimeFormat.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.RelativeTimeFormat.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::RelativeTimeFormatSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
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
                "Intl.RelativeTimeFormat supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.RelativeTimeFormat argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::RelativeTimeFormat,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "Segmenter" {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind: IntlFormatterKind::Segmenter,
                locales: None,
                options: None,
                called_with_new: true,
            }));
        }

        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(segmenter_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if segmenter_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.Segmenter".to_string())));
            }

            if segmenter_member == "supportedLocalesOf" {
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
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.Segmenter.supportedLocalesOf requires locales and optional options"
                            .into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Intl.Segmenter.supportedLocalesOf options cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::IntlStaticMethod {
                    method: IntlStaticMethod::SegmenterSupportedLocalesOf,
                    args: parsed,
                }));
            }

            return Ok(None);
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
                "Intl.Segmenter supports up to two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "Intl.Segmenter argument cannot be empty".into(),
            ));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: IntlFormatterKind::Segmenter,
            locales,
            options,
            called_with_new,
        }));
    }

    if member == "Locale" {
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(locale_member) = cursor.parse_identifier() else {
                return Ok(None);
            };
            cursor.skip_ws();

            if locale_member == "prototype" {
                if !cursor.consume_byte(b'[') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("Symbol") {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_byte(b'.') {
                    return Ok(None);
                }
                cursor.skip_ws();
                if !cursor.consume_ascii("toStringTag") {
                    return Ok(None);
                }
                if let Some(next) = cursor.peek() {
                    if is_ident_char(next) {
                        return Ok(None);
                    }
                }
                cursor.skip_ws();
                cursor.expect_byte(b']')?;
                cursor.skip_ws();
                if !cursor.eof() {
                    return Ok(None);
                }
                return Ok(Some(Expr::String("Intl.Locale".to_string())));
            }

            return Ok(None);
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
        if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "Intl.Locale requires a locale identifier and optional options".into(),
            ));
        }
        if args.len() == 2 && args[1].trim().is_empty() {
            return Err(Error::ScriptParse(
                "Intl.Locale options cannot be empty".into(),
            ));
        }
        let tag = Box::new(parse_expr(args[0].trim())?);
        let options = if args.len() == 2 {
            Some(Box::new(parse_expr(args[1].trim())?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlLocaleConstruct {
            tag,
            options,
            called_with_new,
        }));
    }

    let intl_formatter_kind = match member.as_str() {
        "NumberFormat" => Some(IntlFormatterKind::NumberFormat),
        _ => None,
    };
    if let Some(kind) = intl_formatter_kind {
        if called_with_new && cursor.eof() {
            return Ok(Some(Expr::IntlFormatterConstruct {
                kind,
                locales: None,
                options: None,
                called_with_new: true,
            }));
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
            return Err(Error::ScriptParse(format!(
                "Intl.{member} supports up to two arguments"
            )));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(format!(
                "Intl.{member} argument cannot be empty"
            )));
        }
        let locales = args
            .first()
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|value| parse_expr(value.trim()))
            .transpose()?
            .map(Box::new);

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind,
            locales,
            options,
            called_with_new,
        }));
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
    let expr = match member.as_str() {
        "getCanonicalLocales" => {
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "Intl.getCanonicalLocales supports zero or one argument".into(),
                ));
            }
            if args.len() == 1 && args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.getCanonicalLocales argument cannot be empty".into(),
                ));
            }
            let mut parsed = Vec::new();
            if let Some(arg) = args.first() {
                parsed.push(parse_expr(arg.trim())?);
            }
            Expr::IntlStaticMethod {
                method: IntlStaticMethod::GetCanonicalLocales,
                args: parsed,
            }
        }
        "supportedValuesOf" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.supportedValuesOf requires exactly one argument".into(),
                ));
            }
            Expr::IntlStaticMethod {
                method: IntlStaticMethod::SupportedValuesOf,
                args: vec![parse_expr(args[0].trim())?],
            }
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}

pub(super) fn parse_math_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }
    if !cursor.consume_ascii("Math") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.consume_byte(b'[') {
        cursor.skip_ws();
        if !cursor.consume_ascii("Symbol") {
            return Ok(None);
        }
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
        if !cursor.consume_ascii("toStringTag") {
            return Ok(None);
        }
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        cursor.skip_ws();
        cursor.expect_byte(b']')?;
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MathConst(MathConst::ToStringTag)));
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let Some(method) = parse_math_method_name(&member) else {
            return Ok(None);
        };
        validate_math_arity(method, args.len())?;

        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "Math.{} argument cannot be empty",
                    member
                )));
            }
            parsed.push(parse_expr(arg)?);
        }

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MathMethod {
            method,
            args: parsed,
        }));
    }

    let Some(constant) = parse_math_const_name(&member) else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::MathConst(constant)))
}

pub(super) fn parse_math_const_name(name: &str) -> Option<MathConst> {
    match name {
        "E" => Some(MathConst::E),
        "LN10" => Some(MathConst::Ln10),
        "LN2" => Some(MathConst::Ln2),
        "LOG10E" => Some(MathConst::Log10E),
        "LOG2E" => Some(MathConst::Log2E),
        "PI" => Some(MathConst::Pi),
        "SQRT1_2" => Some(MathConst::Sqrt1_2),
        "SQRT2" => Some(MathConst::Sqrt2),
        _ => None,
    }
}

pub(super) fn parse_math_method_name(name: &str) -> Option<MathMethod> {
    match name {
        "abs" => Some(MathMethod::Abs),
        "acos" => Some(MathMethod::Acos),
        "acosh" => Some(MathMethod::Acosh),
        "asin" => Some(MathMethod::Asin),
        "asinh" => Some(MathMethod::Asinh),
        "atan" => Some(MathMethod::Atan),
        "atan2" => Some(MathMethod::Atan2),
        "atanh" => Some(MathMethod::Atanh),
        "cbrt" => Some(MathMethod::Cbrt),
        "ceil" => Some(MathMethod::Ceil),
        "clz32" => Some(MathMethod::Clz32),
        "cos" => Some(MathMethod::Cos),
        "cosh" => Some(MathMethod::Cosh),
        "exp" => Some(MathMethod::Exp),
        "expm1" => Some(MathMethod::Expm1),
        "floor" => Some(MathMethod::Floor),
        "f16round" => Some(MathMethod::F16Round),
        "fround" => Some(MathMethod::FRound),
        "hypot" => Some(MathMethod::Hypot),
        "imul" => Some(MathMethod::Imul),
        "log" => Some(MathMethod::Log),
        "log10" => Some(MathMethod::Log10),
        "log1p" => Some(MathMethod::Log1p),
        "log2" => Some(MathMethod::Log2),
        "max" => Some(MathMethod::Max),
        "min" => Some(MathMethod::Min),
        "pow" => Some(MathMethod::Pow),
        "random" => Some(MathMethod::Random),
        "round" => Some(MathMethod::Round),
        "sign" => Some(MathMethod::Sign),
        "sin" => Some(MathMethod::Sin),
        "sinh" => Some(MathMethod::Sinh),
        "sqrt" => Some(MathMethod::Sqrt),
        "sumPrecise" => Some(MathMethod::SumPrecise),
        "tan" => Some(MathMethod::Tan),
        "tanh" => Some(MathMethod::Tanh),
        "trunc" => Some(MathMethod::Trunc),
        _ => None,
    }
}

