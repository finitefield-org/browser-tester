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

    if args.len() > 7 {
        return Err(Error::ScriptParse(
            "new Date supports up to seven arguments".into(),
        ));
    }
    if args.iter().any(|arg| arg.trim().is_empty()) {
        return Err(Error::ScriptParse(
            "new Date argument cannot be empty".into(),
        ));
    }
    let mut parsed_args = Vec::with_capacity(args.len());
    for arg in args {
        parsed_args.push(parse_expr(arg.trim())?);
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::DateNew { args: parsed_args }))
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
