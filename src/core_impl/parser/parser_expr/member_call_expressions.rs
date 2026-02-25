use super::*;

fn parse_member_name(cursor: &mut Cursor<'_>) -> Option<(String, bool)> {
    if cursor.consume_byte(b'#') {
        let name = cursor.parse_identifier()?;
        return Some((name, true));
    }
    let name = cursor.parse_identifier()?;
    Some((name, false))
}

fn has_top_level_optional_chain_marker(src: &str) -> bool {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i + 1 < bytes.len() {
        if scanner.is_top_level() && bytes[i] == b'?' && bytes[i + 1] == b'.' {
            return true;
        }
        i = scanner.advance(bytes, i);
    }

    false
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
        if !optional && has_top_level_optional_chain_marker(base_src) {
            optional = true;
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
        let Some((member, is_private)) = parse_member_name(&mut cursor) else {
            continue;
        };
        cursor.skip_ws();
        let mut optional_call = false;
        if cursor.consume_ascii("?.") {
            optional_call = true;
            cursor.skip_ws();
        }
        if cursor.peek() != Some(b'(') {
            continue;
        }
        if is_private && (optional || optional_call) {
            return Err(Error::ScriptParse(
                "optional chaining is not supported for private members".into(),
            ));
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

        let base = base_src.trim();
        if !optional && !optional_call && !is_private && base == "new" && member == "target" {
            return Ok(Some(Expr::Call {
                target: Box::new(Expr::NewTarget),
                args: parsed_args,
                optional: false,
            }));
        }

        if is_private {
            return Ok(Some(Expr::PrivateMemberCall {
                target: Box::new(parse_expr(base_src)?),
                member,
                args: parsed_args,
            }));
        }
        return Ok(Some(Expr::MemberCall {
            target: Box::new(parse_expr(base_src)?),
            member,
            args: parsed_args,
            optional,
            optional_call,
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
        let Some((member, is_private)) = parse_member_name(&mut cursor) else {
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
        if !optional && has_top_level_optional_chain_marker(base_src) {
            optional = true;
        }
        if is_private && optional {
            return Err(Error::ScriptParse(
                "optional chaining is not supported for private members".into(),
            ));
        }
        if base_src.is_empty() {
            continue;
        }

        let base = base_src.trim();
        if !optional && !is_private && base == "import" && member == "meta" {
            return Ok(Some(Expr::ImportMeta));
        }
        if !optional && !is_private && base == "new" && member == "target" {
            return Ok(Some(Expr::NewTarget));
        }

        if !optional && (base == "document" || base == "window.document") {
            let target = match member.as_str() {
                "body" => Some(DomQuery::DocumentBody),
                "head" => Some(DomQuery::DocumentHead),
                "documentElement" => Some(DomQuery::DocumentElement),
                _ => None,
            };
            if let Some(target) = target {
                return Ok(Some(Expr::DomRef(target)));
            }
        }

        if is_private {
            return Ok(Some(Expr::PrivateMemberGet {
                target: Box::new(parse_expr(base_src)?),
                member,
            }));
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
        if !optional && has_top_level_optional_chain_marker(base_src) {
            optional = true;
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
        let mut optional = false;
        if cursor.consume_ascii("?.") {
            optional = true;
            cursor.skip_ws();
        }
        if cursor.peek() == Some(b'(') {
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            if target == "import" && !optional {
                let args = parse_call_args(&args_src, "import() arguments cannot be empty")?;
                if args.is_empty() {
                    return Err(Error::ScriptParse(
                        "import() requires a module specifier".into(),
                    ));
                }
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "import() accepts at most two arguments".into(),
                    ));
                }

                let module_src = args[0].trim();
                if module_src.is_empty() {
                    return Err(Error::ScriptParse(
                        "import() requires a module specifier".into(),
                    ));
                }

                let module = parse_expr(module_src)?;
                let options = if args.len() == 2 {
                    let options_src = args[1].trim();
                    if options_src.is_empty() {
                        return Err(Error::ScriptParse(
                            "import() options argument cannot be empty".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(options_src)?))
                } else {
                    None
                };

                cursor.skip_ws();
                if cursor.eof() {
                    return Ok(Some(Expr::ImportCall {
                        module: Box::new(module),
                        options,
                    }));
                }
                return Ok(None);
            }

            let parsed = parse_args(&args_src)?;

            cursor.skip_ws();
            if cursor.eof() {
                if optional {
                    return Ok(Some(Expr::Call {
                        target: Box::new(Expr::Var(target)),
                        args: parsed,
                        optional: true,
                    }));
                }
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
    let mut optional = false;
    if cursor.consume_ascii("?.") {
        optional = true;
        cursor.skip_ws();
    }
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
        optional,
    }))
}
