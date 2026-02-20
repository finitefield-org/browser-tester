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
