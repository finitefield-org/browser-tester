use super::*;
pub(crate) fn parse_number_method_expr(src: &str) -> Result<Option<Expr>> {
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

pub(crate) fn parse_number_instance_method_name(name: &str) -> Option<NumberInstanceMethod> {
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

pub(crate) fn parse_bigint_method_expr(src: &str) -> Result<Option<Expr>> {
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

pub(crate) fn parse_bigint_instance_method_name(name: &str) -> Option<BigIntInstanceMethod> {
    match name {
        "toLocaleString" => Some(BigIntInstanceMethod::ToLocaleString),
        "toString" => Some(BigIntInstanceMethod::ToString),
        "valueOf" => Some(BigIntInstanceMethod::ValueOf),
        _ => None,
    }
}
