pub(super) fn parse_structured_clone_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(
        src,
        "structuredClone",
        "structuredClone requires exactly one argument",
    )
}

pub(super) fn parse_alert_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "alert", "alert requires exactly one argument")
}

pub(super) fn parse_confirm_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "confirm", "confirm requires exactly one argument")
}

pub(super) fn parse_prompt_expr(src: &str) -> Result<Option<(Expr, Option<Expr>)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("prompt") {
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
    if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "prompt requires one or two arguments".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "prompt default argument cannot be empty".into(),
        ));
    }

    let message = parse_expr(args[0].trim())?;
    let default = if args.len() == 2 {
        Some(parse_expr(args[1].trim())?)
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((message, default)))
}

pub(super) fn parse_array_literal_expr(src: &str) -> Result<Option<Vec<Expr>>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if cursor.peek() != Some(b'[') {
        return Ok(None);
    }

    let items_src = cursor.read_balanced_block(b'[', b']')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let mut items = split_top_level_by_char(&items_src, b',');
    while items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty()) {
        items.pop();
    }
    if items.len() == 1 && items[0].trim().is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let item = item.trim();
        if item.is_empty() {
            return Err(Error::ScriptParse(
                "array literal does not support empty elements".into(),
            ));
        }
        if let Some(rest) = item.strip_prefix("...") {
            let rest = rest.trim();
            if rest.is_empty() {
                return Err(Error::ScriptParse(
                    "array spread source cannot be empty".into(),
                ));
            }
            out.push(Expr::Spread(Box::new(parse_expr(rest)?)));
        } else {
            out.push(parse_expr(item)?);
        }
    }
    Ok(Some(out))
}

pub(super) fn parse_array_is_array_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Array") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("isArray") {
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
            "Array.isArray requires exactly one argument".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

pub(super) fn parse_array_from_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Array") {
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
    if !cursor.consume_ascii("from") {
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
    if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Array.from requires one or two arguments".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Array.from map function cannot be empty".into(),
        ));
    }

    let source = parse_expr(args[0].trim())?;
    let map_fn = if args.len() == 2 {
        Some(Box::new(parse_expr(args[1].trim())?))
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::ArrayFrom {
        source: Box::new(source),
        map_fn,
    }))
}

pub(super) fn parse_array_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if cursor.peek() == Some(b'[') {
        let index_src = cursor.read_balanced_block(b'[', b']')?;
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        if index_src.trim().is_empty() {
            return Err(Error::ScriptParse("array index cannot be empty".into()));
        }
        let index = parse_expr(index_src.trim())?;
        return Ok(Some(Expr::ArrayIndex {
            target,
            index: Box::new(index),
        }));
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if method == "length" {
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ArrayLength(target)));
    }

    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = parse_call_args(&args_src, "array method arguments cannot be empty")?;

    let expr = match method.as_str() {
        "push" => {
            let mut parsed = Vec::with_capacity(args.len());
            for arg in args {
                parsed.push(parse_call_arg_expr(arg)?);
            }
            Expr::ArrayPush {
                target,
                args: parsed,
            }
        }
        "pop" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("pop does not take arguments".into()));
            }
            Expr::ArrayPop(target)
        }
        "shift" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("shift does not take arguments".into()));
            }
            Expr::ArrayShift(target)
        }
        "unshift" => {
            let mut parsed = Vec::with_capacity(args.len());
            for arg in args {
                parsed.push(parse_call_arg_expr(arg)?);
            }
            Expr::ArrayUnshift {
                target,
                args: parsed,
            }
        }
        "map" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "map requires exactly one callback argument".into(),
                ));
            }
            let callback = match parse_array_callback_arg(args[0], 3, "array callback parameters") {
                Ok(callback) => callback,
                Err(_) => {
                    let mut parsed_args = Vec::with_capacity(args.len());
                    for arg in &args {
                        parsed_args.push(parse_call_arg_expr(arg)?);
                    }
                    return Ok(Some(Expr::MemberCall {
                        target: Box::new(Expr::Var(target.clone())),
                        member: method.clone(),
                        args: parsed_args,
                        optional: false,
                    }));
                }
            };
            Expr::ArrayMap { target, callback }
        }
        "filter" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "filter requires exactly one callback argument".into(),
                ));
            }
            let callback = match parse_array_callback_arg(args[0], 3, "array callback parameters") {
                Ok(callback) => callback,
                Err(_) => {
                    let mut parsed_args = Vec::with_capacity(args.len());
                    for arg in &args {
                        parsed_args.push(parse_call_arg_expr(arg)?);
                    }
                    return Ok(Some(Expr::MemberCall {
                        target: Box::new(Expr::Var(target.clone())),
                        member: method.clone(),
                        args: parsed_args,
                        optional: false,
                    }));
                }
            };
            Expr::ArrayFilter { target, callback }
        }
        "reduce" => {
            if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "reduce requires callback and optional initial value".into(),
                ));
            }
            let callback = match parse_array_callback_arg(args[0], 4, "array callback parameters") {
                Ok(callback) => callback,
                Err(_) => {
                    let mut parsed_args = Vec::with_capacity(args.len());
                    for arg in &args {
                        parsed_args.push(parse_call_arg_expr(arg)?);
                    }
                    return Ok(Some(Expr::MemberCall {
                        target: Box::new(Expr::Var(target.clone())),
                        member: method.clone(),
                        args: parsed_args,
                        optional: false,
                    }));
                }
            };
            let initial = if args.len() == 2 {
                if args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "reduce initial value cannot be empty".into(),
                    ));
                }
                Some(Box::new(parse_expr(args[1].trim())?))
            } else {
                None
            };
            Expr::ArrayReduce {
                target,
                callback,
                initial,
            }
        }
        "forEach" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "forEach requires exactly one callback argument".into(),
                ));
            }
            let callback = match parse_array_callback_arg(args[0], 3, "array callback parameters") {
                Ok(callback) => callback,
                Err(_) => {
                    let mut parsed_args = Vec::with_capacity(args.len());
                    for arg in &args {
                        parsed_args.push(parse_call_arg_expr(arg)?);
                    }
                    return Ok(Some(Expr::MemberCall {
                        target: Box::new(Expr::Var(target.clone())),
                        member: method.clone(),
                        args: parsed_args,
                        optional: false,
                    }));
                }
            };
            Expr::ArrayForEach { target, callback }
        }
        "find" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "find requires exactly one callback argument".into(),
                ));
            }
            let callback = match parse_array_callback_arg(args[0], 3, "array callback parameters") {
                Ok(callback) => callback,
                Err(_) => {
                    let mut parsed_args = Vec::with_capacity(args.len());
                    for arg in &args {
                        parsed_args.push(parse_call_arg_expr(arg)?);
                    }
                    return Ok(Some(Expr::MemberCall {
                        target: Box::new(Expr::Var(target.clone())),
                        member: method.clone(),
                        args: parsed_args,
                        optional: false,
                    }));
                }
            };
            Expr::ArrayFind { target, callback }
        }
        "findIndex" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "findIndex requires exactly one callback argument".into(),
                ));
            }
            let callback = match parse_array_callback_arg(args[0], 3, "array callback parameters") {
                Ok(callback) => callback,
                Err(_) => {
                    let mut parsed_args = Vec::with_capacity(args.len());
                    for arg in &args {
                        parsed_args.push(parse_call_arg_expr(arg)?);
                    }
                    return Ok(Some(Expr::MemberCall {
                        target: Box::new(Expr::Var(target.clone())),
                        member: method.clone(),
                        args: parsed_args,
                        optional: false,
                    }));
                }
            };
            Expr::ArrayFindIndex { target, callback }
        }
        "some" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "some requires exactly one callback argument".into(),
                ));
            }
            let callback = match parse_array_callback_arg(args[0], 3, "array callback parameters") {
                Ok(callback) => callback,
                Err(_) => {
                    let mut parsed_args = Vec::with_capacity(args.len());
                    for arg in &args {
                        parsed_args.push(parse_call_arg_expr(arg)?);
                    }
                    return Ok(Some(Expr::MemberCall {
                        target: Box::new(Expr::Var(target.clone())),
                        member: method.clone(),
                        args: parsed_args,
                        optional: false,
                    }));
                }
            };
            Expr::ArraySome { target, callback }
        }
        "every" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "every requires exactly one callback argument".into(),
                ));
            }
            let callback = match parse_array_callback_arg(args[0], 3, "array callback parameters") {
                Ok(callback) => callback,
                Err(_) => {
                    let mut parsed_args = Vec::with_capacity(args.len());
                    for arg in &args {
                        parsed_args.push(parse_call_arg_expr(arg)?);
                    }
                    return Ok(Some(Expr::MemberCall {
                        target: Box::new(Expr::Var(target.clone())),
                        member: method.clone(),
                        args: parsed_args,
                        optional: false,
                    }));
                }
            };
            Expr::ArrayEvery { target, callback }
        }
        "includes" => {
            if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "includes requires one or two arguments".into(),
                ));
            }
            if args.len() == 2 && args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "includes fromIndex cannot be empty".into(),
                ));
            }
            Expr::ArrayIncludes {
                target,
                search: Box::new(parse_expr(args[0].trim())?),
                from_index: if args.len() == 2 {
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                },
            }
        }
        "slice" => {
            if args.len() > 2 {
                return Err(Error::ScriptParse(
                    "slice supports up to two arguments".into(),
                ));
            }
            let start = if !args.is_empty() {
                if args[0].trim().is_empty() {
                    return Err(Error::ScriptParse("slice start cannot be empty".into()));
                }
                Some(Box::new(parse_expr(args[0].trim())?))
            } else {
                None
            };
            let end = if args.len() == 2 {
                if args[1].trim().is_empty() {
                    return Err(Error::ScriptParse("slice end cannot be empty".into()));
                }
                Some(Box::new(parse_expr(args[1].trim())?))
            } else {
                None
            };
            Expr::ArraySlice { target, start, end }
        }
        "splice" => {
            if args.is_empty() || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "splice requires at least start index".into(),
                ));
            }
            let start = Box::new(parse_expr(args[0].trim())?);
            let delete_count = if args.len() >= 2 {
                if args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "splice deleteCount cannot be empty".into(),
                    ));
                }
                Some(Box::new(parse_expr(args[1].trim())?))
            } else {
                None
            };
            let mut items = Vec::new();
            for arg in args.iter().skip(2) {
                items.push(parse_call_arg_expr(arg)?);
            }
            Expr::ArraySplice {
                target,
                start,
                delete_count,
                items,
            }
        }
        "join" => {
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "join supports at most one argument".into(),
                ));
            }
            let separator = if args.len() == 1 {
                if args[0].trim().is_empty() {
                    return Err(Error::ScriptParse("join separator cannot be empty".into()));
                }
                Some(Box::new(parse_expr(args[0].trim())?))
            } else {
                None
            };
            Expr::ArrayJoin { target, separator }
        }
        "sort" => {
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "sort supports at most one argument".into(),
                ));
            }
            if args.len() == 1 && args[0].trim().is_empty() {
                return Err(Error::ScriptParse("sort comparator cannot be empty".into()));
            }
            Expr::ArraySort {
                target,
                comparator: if args.len() == 1 {
                    Some(Box::new(parse_expr(args[0].trim())?))
                } else {
                    None
                },
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

pub(super) fn parse_array_callback_arg(
    arg: &str,
    max_params: usize,
    label: &str,
) -> Result<ScriptHandler> {
    let callback_arg = strip_js_comments(arg);
    let mut callback_cursor = Cursor::new(callback_arg.as_str().trim());
    let (params, body, concise_body) = parse_callback(&mut callback_cursor, max_params, label)?;
    callback_cursor.skip_ws();
    if !callback_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported array callback: {}",
            arg.trim()
        )));
    }

    let stmts = if concise_body {
        vec![Stmt::Return {
            value: Some(parse_expr(body.trim())?),
        }]
    } else {
        parse_block_statements(&body)?
    };

    Ok(ScriptHandler { params, stmts })
}

pub(super) fn parse_number_method_expr(src: &str) -> Result<Option<Expr>> {
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
        let Some(method) = cursor.parse_identifier() else {
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

        let Some(method) = parse_number_instance_method_name(&method) else {
            continue;
        };

        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let parsed = match method {
            NumberInstanceMethod::ToLocaleString => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "toLocaleString supports at most two arguments".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                for arg in &args {
                    let arg = arg.trim();
                    if arg.is_empty() {
                        return Err(Error::ScriptParse(
                            "toLocaleString arguments cannot be empty".into(),
                        ));
                    }
                    parsed.push(parse_expr(arg)?);
                }
                parsed
            }
            NumberInstanceMethod::ValueOf => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse("valueOf does not take arguments".into()));
                }
                Vec::new()
            }
            NumberInstanceMethod::ToExponential
            | NumberInstanceMethod::ToFixed
            | NumberInstanceMethod::ToPrecision
            | NumberInstanceMethod::ToString => {
                if args.len() > 1 {
                    let method_name = match method {
                        NumberInstanceMethod::ToExponential => "toExponential",
                        NumberInstanceMethod::ToFixed => "toFixed",
                        NumberInstanceMethod::ToPrecision => "toPrecision",
                        NumberInstanceMethod::ToString => "toString",
                        _ => unreachable!(),
                    };
                    return Err(Error::ScriptParse(format!(
                        "{method_name} supports at most one argument"
                    )));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    let method_name = match method {
                        NumberInstanceMethod::ToExponential => "toExponential",
                        NumberInstanceMethod::ToFixed => "toFixed",
                        NumberInstanceMethod::ToPrecision => "toPrecision",
                        NumberInstanceMethod::ToString => "toString",
                        _ => unreachable!(),
                    };
                    return Err(Error::ScriptParse(format!(
                        "{method_name} argument cannot be empty"
                    )));
                }
                if args.len() == 1 {
                    vec![parse_expr(args[0].trim())?]
                } else {
                    Vec::new()
                }
            }
        };

        return Ok(Some(Expr::NumberInstanceMethod {
            value: Box::new(parse_expr(base_src)?),
            method,
            args: parsed,
        }));
    }

    Ok(None)
}

pub(super) fn parse_number_instance_method_name(name: &str) -> Option<NumberInstanceMethod> {
    match name {
        "toExponential" => Some(NumberInstanceMethod::ToExponential),
        "toFixed" => Some(NumberInstanceMethod::ToFixed),
        "toLocaleString" => Some(NumberInstanceMethod::ToLocaleString),
        "toPrecision" => Some(NumberInstanceMethod::ToPrecision),
        "toString" => Some(NumberInstanceMethod::ToString),
        "valueOf" => Some(NumberInstanceMethod::ValueOf),
        _ => None,
    }
}

pub(super) fn parse_bigint_method_expr(src: &str) -> Result<Option<Expr>> {
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
        let Some(method_name) = cursor.parse_identifier() else {
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

        let Some(method) = parse_bigint_instance_method_name(&method_name) else {
            continue;
        };

        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let parsed = match method {
            BigIntInstanceMethod::ToLocaleString => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "toLocaleString supports at most two arguments".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                for arg in &args {
                    let arg = arg.trim();
                    if arg.is_empty() {
                        return Err(Error::ScriptParse(
                            "toLocaleString arguments cannot be empty".into(),
                        ));
                    }
                    parsed.push(parse_expr(arg)?);
                }
                parsed
            }
            BigIntInstanceMethod::ValueOf => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse("valueOf does not take arguments".into()));
                }
                Vec::new()
            }
            BigIntInstanceMethod::ToString => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "toString supports at most one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "toString argument cannot be empty".into(),
                    ));
                }
                if args.len() == 1 {
                    vec![parse_expr(args[0].trim())?]
                } else {
                    Vec::new()
                }
            }
        };

        return Ok(Some(Expr::BigIntInstanceMethod {
            value: Box::new(parse_expr(base_src)?),
            method,
            args: parsed,
        }));
    }

    Ok(None)
}

pub(super) fn parse_bigint_instance_method_name(name: &str) -> Option<BigIntInstanceMethod> {
    match name {
        "toLocaleString" => Some(BigIntInstanceMethod::ToLocaleString),
        "toString" => Some(BigIntInstanceMethod::ToString),
        "valueOf" => Some(BigIntInstanceMethod::ValueOf),
        _ => None,
    }
}

pub(super) fn parse_intl_format_expr(src: &str) -> Result<Option<Expr>> {
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
        let Some(method_name) = cursor.parse_identifier() else {
            continue;
        };

        if method_name == "compare" {
            cursor.skip_ws();
            if cursor.eof() {
                return Ok(Some(Expr::IntlCollatorCompareGetter {
                    collator: Box::new(parse_expr(base_src)?),
                }));
            }
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
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.Collator.compare requires exactly two arguments".into(),
                ));
            }
            return Ok(Some(Expr::IntlCollatorCompare {
                collator: Box::new(parse_expr(base_src)?),
                left: Box::new(parse_expr(args[0].trim())?),
                right: Box::new(parse_expr(args[1].trim())?),
            }));
        }

        if method_name == "formatRangeToParts" {
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
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.DateTimeFormat.formatRangeToParts requires exactly two arguments".into(),
                ));
            }
            return Ok(Some(Expr::IntlDateTimeFormatRangeToParts {
                formatter: Box::new(parse_expr(base_src)?),
                start: Box::new(parse_expr(args[0].trim())?),
                end: Box::new(parse_expr(args[1].trim())?),
            }));
        }

        if method_name == "formatRange" {
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
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.DateTimeFormat.formatRange requires exactly two arguments".into(),
                ));
            }
            return Ok(Some(Expr::IntlDateTimeFormatRange {
                formatter: Box::new(parse_expr(base_src)?),
                start: Box::new(parse_expr(args[0].trim())?),
                end: Box::new(parse_expr(args[1].trim())?),
            }));
        }

        if method_name == "formatToParts" {
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
            if args.len() == 2 && !args[0].trim().is_empty() && !args[1].trim().is_empty() {
                return Ok(Some(Expr::IntlRelativeTimeFormatToParts {
                    formatter: Box::new(parse_expr(base_src)?),
                    value: Box::new(parse_expr(args[0].trim())?),
                    unit: Box::new(parse_expr(args[1].trim())?),
                }));
            }
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "Intl.DateTimeFormat.formatToParts supports at most one argument".into(),
                ));
            }
            if args.len() == 1 && args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.DateTimeFormat.formatToParts argument cannot be empty".into(),
                ));
            }
            return Ok(Some(Expr::IntlDateTimeFormatToParts {
                formatter: Box::new(parse_expr(base_src)?),
                value: args
                    .first()
                    .map(|arg| parse_expr(arg.trim()))
                    .transpose()?
                    .map(Box::new),
            }));
        }

        if method_name == "of" {
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
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.DisplayNames.of requires exactly one argument".into(),
                ));
            }
            return Ok(Some(Expr::IntlDisplayNamesOf {
                display_names: Box::new(parse_expr(base_src)?),
                code: Box::new(parse_expr(args[0].trim())?),
            }));
        }

        if method_name == "select" {
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
            if args.len() != 1 || args[0].trim().is_empty() {
                // Avoid hijacking non-Intl methods such as HTMLTextAreaElement.select().
                continue;
            }
            return Ok(Some(Expr::IntlPluralRulesSelect {
                plural_rules: Box::new(parse_expr(base_src)?),
                value: Box::new(parse_expr(args[0].trim())?),
            }));
        }

        if method_name == "selectRange" {
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
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                continue;
            }
            return Ok(Some(Expr::IntlPluralRulesSelectRange {
                plural_rules: Box::new(parse_expr(base_src)?),
                start: Box::new(parse_expr(args[0].trim())?),
                end: Box::new(parse_expr(args[1].trim())?),
            }));
        }

        if method_name == "segment" {
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
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl.Segmenter.segment requires exactly one argument".into(),
                ));
            }
            return Ok(Some(Expr::IntlSegmenterSegment {
                segmenter: Box::new(parse_expr(base_src)?),
                value: Box::new(parse_expr(args[0].trim())?),
            }));
        }

        let intl_locale_method = match method_name.as_str() {
            "getCalendars" => Some(IntlLocaleMethod::GetCalendars),
            "getCollations" => Some(IntlLocaleMethod::GetCollations),
            "getHourCycles" => Some(IntlLocaleMethod::GetHourCycles),
            "getNumberingSystems" => Some(IntlLocaleMethod::GetNumberingSystems),
            "getTextInfo" => Some(IntlLocaleMethod::GetTextInfo),
            "getTimeZones" => Some(IntlLocaleMethod::GetTimeZones),
            "getWeekInfo" => Some(IntlLocaleMethod::GetWeekInfo),
            "maximize" => Some(IntlLocaleMethod::Maximize),
            "minimize" => Some(IntlLocaleMethod::Minimize),
            "toString" => Some(IntlLocaleMethod::ToString),
            _ => None,
        };
        if let Some(method) = intl_locale_method {
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
            if !args.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "Intl.Locale.{method_name} does not take arguments"
                )));
            }
            return Ok(Some(Expr::IntlLocaleMethod {
                locale: Box::new(parse_expr(base_src)?),
                method,
            }));
        }

        if method_name == "resolvedOptions" {
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
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "Intl formatter resolvedOptions does not take arguments".into(),
                ));
            }
            return Ok(Some(Expr::IntlDateTimeResolvedOptions {
                formatter: Box::new(parse_expr(base_src)?),
            }));
        }

        if method_name == "format" {
            cursor.skip_ws();
            if cursor.eof() {
                return Ok(Some(Expr::IntlFormatGetter {
                    formatter: Box::new(parse_expr(base_src)?),
                }));
            }
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
            if args.len() == 2 && !args[0].trim().is_empty() && !args[1].trim().is_empty() {
                return Ok(Some(Expr::IntlRelativeTimeFormat {
                    formatter: Box::new(parse_expr(base_src)?),
                    value: Box::new(parse_expr(args[0].trim())?),
                    unit: Box::new(parse_expr(args[1].trim())?),
                }));
            }
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "Intl formatter format supports at most one argument".into(),
                ));
            }
            if args.len() == 1 && args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Intl formatter format argument cannot be empty".into(),
                ));
            }

            return Ok(Some(Expr::IntlFormat {
                formatter: Box::new(parse_expr(base_src)?),
                value: args
                    .first()
                    .map(|arg| parse_expr(arg.trim()))
                    .transpose()?
                    .map(Box::new),
            }));
        }
    }

    Ok(None)
}

pub(super) fn parse_string_method_expr(src: &str) -> Result<Option<Expr>> {
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
        let Some(method) = cursor.parse_identifier() else {
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

        if !matches!(
            method.as_str(),
            "charAt"
                | "charCodeAt"
                | "codePointAt"
                | "at"
                | "concat"
                | "trim"
                | "trimStart"
                | "trimEnd"
                | "toUpperCase"
                | "toLocaleUpperCase"
                | "toLowerCase"
                | "toLocaleLowerCase"
                | "includes"
                | "startsWith"
                | "endsWith"
                | "slice"
                | "substring"
                | "match"
                | "split"
                | "replace"
                | "replaceAll"
                | "indexOf"
                | "lastIndexOf"
                | "search"
                | "repeat"
                | "padStart"
                | "padEnd"
                | "localeCompare"
                | "isWellFormed"
                | "toWellFormed"
                | "valueOf"
                | "toString"
        ) {
            continue;
        }

        if (method == "toString" || method == "valueOf") && !args.is_empty() {
            continue;
        }

        let base_expr = if let Some(target) = parse_element_ref_expr(base_src)? {
            Expr::DomRef(target)
        } else {
            parse_expr(base_src)?
        };
        let base = Box::new(base_expr.clone());
        let expr = match method.as_str() {
            "charAt" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "charAt supports zero or one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse("charAt index cannot be empty".into()));
                }
                Expr::StringCharAt {
                    value: base,
                    index: args
                        .first()
                        .map(|arg| parse_expr(arg.trim()))
                        .transpose()?
                        .map(Box::new),
                }
            }
            "charCodeAt" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "charCodeAt supports zero or one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "charCodeAt index cannot be empty".into(),
                    ));
                }
                Expr::StringCharCodeAt {
                    value: base,
                    index: args
                        .first()
                        .map(|arg| parse_expr(arg.trim()))
                        .transpose()?
                        .map(Box::new),
                }
            }
            "codePointAt" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "codePointAt supports zero or one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "codePointAt index cannot be empty".into(),
                    ));
                }
                Expr::StringCodePointAt {
                    value: base,
                    index: args
                        .first()
                        .map(|arg| parse_expr(arg.trim()))
                        .transpose()?
                        .map(Box::new),
                }
            }
            "at" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "at supports zero or one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse("at index cannot be empty".into()));
                }
                Expr::StringAt {
                    value: base,
                    index: args
                        .first()
                        .map(|arg| parse_expr(arg.trim()))
                        .transpose()?
                        .map(Box::new),
                }
            }
            "concat" => {
                let mut parsed = Vec::with_capacity(args.len());
                for arg in args {
                    if arg.trim().is_empty() {
                        return Err(Error::ScriptParse("concat argument cannot be empty".into()));
                    }
                    parsed.push(parse_expr(arg.trim())?);
                }
                Expr::StringConcat {
                    value: base,
                    args: parsed,
                }
            }
            "trim" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse("trim does not take arguments".into()));
                }
                Expr::StringTrim {
                    value: base,
                    mode: StringTrimMode::Both,
                }
            }
            "trimStart" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "trimStart does not take arguments".into(),
                    ));
                }
                Expr::StringTrim {
                    value: base,
                    mode: StringTrimMode::Start,
                }
            }
            "trimEnd" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse("trimEnd does not take arguments".into()));
                }
                Expr::StringTrim {
                    value: base,
                    mode: StringTrimMode::End,
                }
            }
            "toUpperCase" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "toUpperCase does not take arguments".into(),
                    ));
                }
                Expr::StringToUpperCase(base)
            }
            "toLocaleUpperCase" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "toLocaleUpperCase supports up to one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "toLocaleUpperCase locale cannot be empty".into(),
                    ));
                }
                Expr::StringToUpperCase(base)
            }
            "toLowerCase" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "toLowerCase does not take arguments".into(),
                    ));
                }
                Expr::StringToLowerCase(base)
            }
            "toLocaleLowerCase" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "toLocaleLowerCase supports up to one argument".into(),
                    ));
                }
                if args.len() == 1 && args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "toLocaleLowerCase locale cannot be empty".into(),
                    ));
                }
                Expr::StringToLowerCase(base)
            }
            "includes" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "String.includes requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "String.includes position cannot be empty".into(),
                    ));
                }
                Expr::StringIncludes {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    position: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "startsWith" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "startsWith requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "startsWith position cannot be empty".into(),
                    ));
                }
                Expr::StringStartsWith {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    position: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "endsWith" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "endsWith requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "endsWith length argument cannot be empty".into(),
                    ));
                }
                Expr::StringEndsWith {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    length: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "slice" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "String.slice supports up to two arguments".into(),
                    ));
                }
                let start = if !args.is_empty() {
                    if args[0].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "String.slice start cannot be empty".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[0].trim())?))
                } else {
                    None
                };
                let end = if args.len() == 2 {
                    if args[1].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "String.slice end cannot be empty".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                };
                Expr::StringSlice {
                    value: base,
                    start,
                    end,
                }
            }
            "substring" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "substring supports up to two arguments".into(),
                    ));
                }
                let start = if !args.is_empty() {
                    if args[0].trim().is_empty() {
                        return Err(Error::ScriptParse("substring start cannot be empty".into()));
                    }
                    Some(Box::new(parse_expr(args[0].trim())?))
                } else {
                    None
                };
                let end = if args.len() == 2 {
                    if args[1].trim().is_empty() {
                        return Err(Error::ScriptParse("substring end cannot be empty".into()));
                    }
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                };
                Expr::StringSubstring {
                    value: base,
                    start,
                    end,
                }
            }
            "match" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "match requires exactly one argument".into(),
                    ));
                }
                Expr::StringMatch {
                    value: base,
                    pattern: Box::new(parse_expr(args[0].trim())?),
                }
            }
            "split" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "split supports up to two arguments".into(),
                    ));
                }
                let separator = if !args.is_empty() {
                    if args[0].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "split separator cannot be empty expression".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[0].trim())?))
                } else {
                    None
                };
                let limit = if args.len() == 2 {
                    if args[1].trim().is_empty() {
                        return Err(Error::ScriptParse("split limit cannot be empty".into()));
                    }
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                };
                Expr::StringSplit {
                    value: base,
                    separator,
                    limit,
                }
            }
            "replace" => {
                if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "replace requires exactly two arguments".into(),
                    ));
                }
                Expr::StringReplace {
                    value: base,
                    from: Box::new(parse_expr(args[0].trim())?),
                    to: Box::new(parse_expr(args[1].trim())?),
                }
            }
            "replaceAll" => {
                if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "replaceAll requires exactly two arguments".into(),
                    ));
                }
                Expr::StringReplaceAll {
                    value: base,
                    from: Box::new(parse_expr(args[0].trim())?),
                    to: Box::new(parse_expr(args[1].trim())?),
                }
            }
            "indexOf" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "indexOf requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "indexOf position cannot be empty".into(),
                    ));
                }
                Expr::StringIndexOf {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    position: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "lastIndexOf" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "lastIndexOf requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "lastIndexOf position cannot be empty".into(),
                    ));
                }
                Expr::StringLastIndexOf {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    position: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "search" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "search requires exactly one argument".into(),
                    ));
                }
                Expr::StringSearch {
                    value: base,
                    pattern: Box::new(parse_expr(args[0].trim())?),
                }
            }
            "repeat" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "repeat requires exactly one argument".into(),
                    ));
                }
                Expr::StringRepeat {
                    value: base,
                    count: Box::new(parse_expr(args[0].trim())?),
                }
            }
            "padStart" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "padStart requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "padStart pad string cannot be empty expression".into(),
                    ));
                }
                Expr::StringPadStart {
                    value: base,
                    target_length: Box::new(parse_expr(args[0].trim())?),
                    pad: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "padEnd" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "padEnd requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "padEnd pad string cannot be empty expression".into(),
                    ));
                }
                Expr::StringPadEnd {
                    value: base,
                    target_length: Box::new(parse_expr(args[0].trim())?),
                    pad: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "localeCompare" => {
                if args.is_empty() || args.len() > 3 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "localeCompare requires one to three arguments".into(),
                    ));
                }
                if args.len() >= 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "localeCompare locales argument cannot be empty".into(),
                    ));
                }
                if args.len() == 3 && args[2].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "localeCompare options argument cannot be empty".into(),
                    ));
                }
                Expr::StringLocaleCompare {
                    value: base,
                    compare: Box::new(parse_expr(args[0].trim())?),
                    locales: if args.len() >= 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                    options: if args.len() == 3 {
                        Some(Box::new(parse_expr(args[2].trim())?))
                    } else {
                        None
                    },
                }
            }
            "isWellFormed" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "isWellFormed does not take arguments".into(),
                    ));
                }
                Expr::StringIsWellFormed(base)
            }
            "toWellFormed" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "toWellFormed does not take arguments".into(),
                    ));
                }
                Expr::StringToWellFormed(base)
            }
            "valueOf" => Expr::StringValueOf(base),
            "toString" => Expr::StringToString(base),
            _ => unreachable!(),
        };

        if optional {
            return Ok(Some(Expr::Ternary {
                cond: Box::new(Expr::Binary {
                    left: Box::new(base_expr),
                    op: BinaryOp::Eq,
                    right: Box::new(Expr::Null),
                }),
                on_true: Box::new(Expr::Undefined),
                on_false: Box::new(expr),
            }));
        }

        return Ok(Some(expr));
    }

    Ok(None)
}

pub(super) fn parse_date_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
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
        "getTime" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("getTime does not take arguments".into()));
            }
            Expr::DateGetTime(target)
        }
        "setTime" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "setTime requires exactly one argument".into(),
                ));
            }
            Expr::DateSetTime {
                target,
                value: Box::new(parse_expr(args[0].trim())?),
            }
        }
        "toISOString" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "toISOString does not take arguments".into(),
                ));
            }
            Expr::DateToIsoString(target)
        }
        "getFullYear" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getFullYear does not take arguments".into(),
                ));
            }
            Expr::DateGetFullYear(target)
        }
        "getMonth" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getMonth does not take arguments".into(),
                ));
            }
            Expr::DateGetMonth(target)
        }
        "getDate" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("getDate does not take arguments".into()));
            }
            Expr::DateGetDate(target)
        }
        "getHours" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getHours does not take arguments".into(),
                ));
            }
            Expr::DateGetHours(target)
        }
        "getMinutes" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getMinutes does not take arguments".into(),
                ));
            }
            Expr::DateGetMinutes(target)
        }
        "getSeconds" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getSeconds does not take arguments".into(),
                ));
            }
            Expr::DateGetSeconds(target)
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}

pub(super) fn collect_top_level_char_positions(src: &str, target: u8) -> Vec<usize> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut scanner = JsLexScanner::new();

    while i < bytes.len() {
        if scanner.is_top_level() && bytes[i] == target {
            out.push(i);
        }
        i = scanner.advance(bytes, i);
    }

    out
}

pub(super) fn parse_set_timeout_expr(src: &str) -> Result<Option<(TimerInvocation, Expr)>> {
    let mut cursor = Cursor::new(src);
    let Some((handler, delay_ms)) = parse_set_timeout_call(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((handler, delay_ms)))
}

