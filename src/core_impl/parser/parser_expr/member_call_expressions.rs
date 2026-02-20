use super::*;
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
