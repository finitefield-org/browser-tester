use super::*;

pub(crate) fn parse_url_expr(src: &str) -> Result<Option<Expr>> {
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

    if !cursor.consume_ascii("URL") {
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
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "URL supports one or two constructor arguments".into(),
            ));
        }
        if args.first().is_none_or(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "URL constructor requires a URL argument".into(),
            ));
        }
        if args.len() == 2 && args[1].trim().is_empty() {
            return Err(Error::ScriptParse(
                "URL base argument cannot be empty".into(),
            ));
        }
        let input = args
            .first()
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);
        let base = args
            .get(1)
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::UrlConstruct {
            input,
            base,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::UrlConstruct {
                input: None,
                base: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
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
        let method = match member.as_str() {
            "canParse" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.canParse requires a URL argument and optional base".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.canParse base argument cannot be empty".into(),
                    ));
                }
                UrlStaticMethod::CanParse
            }
            "parse" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.parse requires a URL argument and optional base".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.parse base argument cannot be empty".into(),
                    ));
                }
                UrlStaticMethod::Parse
            }
            "createObjectURL" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.createObjectURL requires exactly one argument".into(),
                    ));
                }
                UrlStaticMethod::CreateObjectUrl
            }
            "revokeObjectURL" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.revokeObjectURL requires exactly one argument".into(),
                    ));
                }
                UrlStaticMethod::RevokeObjectUrl
            }
            _ => return Ok(None),
        };
        let mut parsed_args = Vec::with_capacity(args.len());
        for arg in args {
            parsed_args.push(parse_expr(arg.trim())?);
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::UrlStaticMethod {
            method,
            args: parsed_args,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::UrlConstructor))
}

pub(crate) fn parse_url_search_params_expr(src: &str) -> Result<Option<Expr>> {
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

    if !cursor.consume_ascii("URLSearchParams") {
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
                "URLSearchParams supports zero or one argument".into(),
            ));
        }
        let init = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse(
                    "URLSearchParams argument cannot be empty".into(),
                ));
            }
            Some(Box::new(parse_expr(first)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::UrlSearchParamsConstruct {
            init,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::UrlSearchParamsConstruct {
                init: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::UrlSearchParamsConstructor))
}
