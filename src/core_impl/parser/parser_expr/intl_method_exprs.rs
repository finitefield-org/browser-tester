use super::*;

pub(crate) fn parse_intl_format_expr(src: &str) -> Result<Option<Expr>> {
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
            let collator = parse_expr(base_src)?;
            if !matches!(
                &collator,
                Expr::IntlFormatterConstruct {
                    kind: IntlFormatterKind::Collator,
                    ..
                }
            ) {
                continue;
            }
            cursor.skip_ws();
            if cursor.eof() {
                return Ok(Some(Expr::IntlCollatorCompareGetter {
                    collator: Box::new(collator),
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
                collator: Box::new(collator),
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
                let formatter = parse_expr(base_src)?;
                if !matches!(&formatter, Expr::IntlFormatterConstruct { .. }) {
                    continue;
                }
                return Ok(Some(Expr::IntlFormatGetter {
                    formatter: Box::new(formatter),
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
            let formatter = parse_expr(base_src)?;
            if args.len() == 2 && !args[0].trim().is_empty() && !args[1].trim().is_empty() {
                return Ok(Some(Expr::IntlRelativeTimeFormat {
                    formatter: Box::new(formatter),
                    value: Box::new(parse_expr(args[0].trim())?),
                    unit: Box::new(parse_expr(args[1].trim())?),
                }));
            }
            if !matches!(&formatter, Expr::IntlFormatterConstruct { .. }) {
                continue;
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
                formatter: Box::new(formatter),
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
