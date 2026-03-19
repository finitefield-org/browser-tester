use super::*;

pub(crate) fn parse_intl_expr(src: &str) -> Result<Option<Expr>> {
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

    if let Some(expr) = parse_intl_formatter_expr(&mut cursor, &member, called_with_new)? {
        return Ok(Some(expr));
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
