use super::*;

pub(crate) fn parse_string_method_expr(src: &str) -> Result<Option<Expr>> {
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
