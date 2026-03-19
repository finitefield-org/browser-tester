use super::*;

pub(crate) fn parse_promise_expr(src: &str) -> Result<Option<Expr>> {
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

    if !cursor.consume_ascii("Promise") {
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
                "Promise supports exactly one executor argument".into(),
            ));
        }
        let executor = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse(
                    "Promise executor argument cannot be empty".into(),
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
        return Ok(Some(Expr::PromiseConstruct {
            executor,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::PromiseConstruct {
                executor: None,
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
            "resolve" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.resolve supports zero or one argument".into(),
                    ));
                }
                PromiseStaticMethod::Resolve
            }
            "reject" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.reject supports zero or one argument".into(),
                    ));
                }
                PromiseStaticMethod::Reject
            }
            "all" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.all requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::All
            }
            "allSettled" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.allSettled requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::AllSettled
            }
            "any" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.any requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::Any
            }
            "race" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.race requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::Race
            }
            "try" => {
                if args.is_empty() || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.try requires at least one argument".into(),
                    ));
                }
                PromiseStaticMethod::Try
            }
            "withResolvers" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.withResolvers does not take arguments".into(),
                    ));
                }
                PromiseStaticMethod::WithResolvers
            }
            _ => return Ok(None),
        };

        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "Promise.{} argument cannot be empty",
                    member
                )));
            }
            parsed.push(parse_expr(arg)?);
        }

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::PromiseStaticMethod {
            method,
            args: parsed,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::PromiseConstructor))
}

pub(crate) fn parse_promise_method_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let dots = collect_top_level_char_positions(src, b'.');
    for dot in dots.into_iter().rev() {
        let Some(base_src) = src.get(..dot) else {
            continue;
        };
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

        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let method = match member.as_str() {
            "then" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "Promise.then supports up to two arguments".into(),
                    ));
                }
                PromiseInstanceMethod::Then
            }
            "catch" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.catch supports at most one argument".into(),
                    ));
                }
                PromiseInstanceMethod::Catch
            }
            "finally" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.finally supports at most one argument".into(),
                    ));
                }
                PromiseInstanceMethod::Finally
            }
            _ => continue,
        };

        let mut parsed_args = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "Promise.{} argument cannot be empty",
                    member
                )));
            }
            parsed_args.push(parse_expr(arg)?);
        }

        return Ok(Some(Expr::PromiseMethod {
            target: Box::new(parse_expr(base_src)?),
            method,
            args: parsed_args,
        }));
    }
    Ok(None)
}

pub(crate) fn parse_blob_expr(src: &str) -> Result<Option<Expr>> {
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

    if !cursor.consume_ascii("Blob") {
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
                "Blob supports zero, one, or two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse("Blob argument cannot be empty".into()));
        }

        let parts = args
            .first()
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::BlobConstruct {
            parts,
            options,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::BlobConstruct {
                parts: None,
                options: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::BlobConstructor))
}

pub(crate) fn parse_array_buffer_expr(src: &str) -> Result<Option<Expr>> {
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

    if !cursor.consume_ascii("ArrayBuffer") {
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
                "ArrayBuffer supports up to two arguments".into(),
            ));
        }
        if args.len() >= 1 && args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "ArrayBuffer byteLength argument cannot be empty".into(),
            ));
        }
        if args.len() == 2 && args[1].trim().is_empty() {
            return Err(Error::ScriptParse(
                "ArrayBuffer options argument cannot be empty".into(),
            ));
        }
        let byte_length = if let Some(first) = args.first() {
            Some(Box::new(parse_expr(first.trim())?))
        } else {
            None
        };
        let options = if args.len() == 2 {
            Some(Box::new(parse_expr(args[1].trim())?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ArrayBufferConstruct {
            byte_length,
            options,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::ArrayBufferConstruct {
                byte_length: None,
                options: None,
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
        if member != "isView" {
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
        if args.len() != 1 || args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "ArrayBuffer.isView requires exactly one argument".into(),
            ));
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ArrayBufferIsView(Box::new(parse_expr(
            args[0].trim(),
        )?))));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::ArrayBufferConstructor))
}
