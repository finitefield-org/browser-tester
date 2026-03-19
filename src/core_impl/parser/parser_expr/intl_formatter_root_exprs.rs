use super::*;

#[derive(Clone, Copy)]
struct IntlFormatterConfig {
    kind: IntlFormatterKind,
    supported_locales_method: IntlStaticMethod,
    to_string_tag: &'static str,
}

pub(crate) fn parse_intl_formatter_expr(
    cursor: &mut Cursor<'_>,
    member: &str,
    called_with_new: bool,
) -> Result<Option<Expr>> {
    let Some(config) = intl_formatter_config(member) else {
        return Ok(None);
    };

    if called_with_new && cursor.eof() {
        return Ok(Some(Expr::IntlFormatterConstruct {
            kind: config.kind,
            locales: None,
            options: None,
            called_with_new: true,
        }));
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(formatter_member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();

        if formatter_member == "prototype" {
            return parse_intl_formatter_to_string_tag(cursor, config.to_string_tag);
        }

        if formatter_member == "supportedLocalesOf" {
            return parse_intl_formatter_supported_locales_of(
                cursor,
                member,
                config.supported_locales_method,
            );
        }

        return Ok(None);
    }

    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    parse_intl_formatter_construct(cursor, member, config.kind, called_with_new)
}

fn intl_formatter_config(member: &str) -> Option<IntlFormatterConfig> {
    match member {
        "Collator" => Some(IntlFormatterConfig {
            kind: IntlFormatterKind::Collator,
            supported_locales_method: IntlStaticMethod::CollatorSupportedLocalesOf,
            to_string_tag: "Intl.Collator",
        }),
        "DateTimeFormat" => Some(IntlFormatterConfig {
            kind: IntlFormatterKind::DateTimeFormat,
            supported_locales_method: IntlStaticMethod::DateTimeFormatSupportedLocalesOf,
            to_string_tag: "Intl.DateTimeFormat",
        }),
        "DisplayNames" => Some(IntlFormatterConfig {
            kind: IntlFormatterKind::DisplayNames,
            supported_locales_method: IntlStaticMethod::DisplayNamesSupportedLocalesOf,
            to_string_tag: "Intl.DisplayNames",
        }),
        "DurationFormat" => Some(IntlFormatterConfig {
            kind: IntlFormatterKind::DurationFormat,
            supported_locales_method: IntlStaticMethod::DurationFormatSupportedLocalesOf,
            to_string_tag: "Intl.DurationFormat",
        }),
        "ListFormat" => Some(IntlFormatterConfig {
            kind: IntlFormatterKind::ListFormat,
            supported_locales_method: IntlStaticMethod::ListFormatSupportedLocalesOf,
            to_string_tag: "Intl.ListFormat",
        }),
        "NumberFormat" => Some(IntlFormatterConfig {
            kind: IntlFormatterKind::NumberFormat,
            supported_locales_method: IntlStaticMethod::NumberFormatSupportedLocalesOf,
            to_string_tag: "Intl.NumberFormat",
        }),
        "PluralRules" => Some(IntlFormatterConfig {
            kind: IntlFormatterKind::PluralRules,
            supported_locales_method: IntlStaticMethod::PluralRulesSupportedLocalesOf,
            to_string_tag: "Intl.PluralRules",
        }),
        "RelativeTimeFormat" => Some(IntlFormatterConfig {
            kind: IntlFormatterKind::RelativeTimeFormat,
            supported_locales_method: IntlStaticMethod::RelativeTimeFormatSupportedLocalesOf,
            to_string_tag: "Intl.RelativeTimeFormat",
        }),
        "Segmenter" => Some(IntlFormatterConfig {
            kind: IntlFormatterKind::Segmenter,
            supported_locales_method: IntlStaticMethod::SegmenterSupportedLocalesOf,
            to_string_tag: "Intl.Segmenter",
        }),
        _ => None,
    }
}

fn parse_intl_formatter_to_string_tag(
    cursor: &mut Cursor<'_>,
    to_string_tag: &'static str,
) -> Result<Option<Expr>> {
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
    Ok(Some(Expr::String(to_string_tag.to_string())))
}

fn parse_intl_formatter_supported_locales_of(
    cursor: &mut Cursor<'_>,
    member: &str,
    method: IntlStaticMethod,
) -> Result<Option<Expr>> {
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
        return Err(Error::ScriptParse(format!(
            "Intl.{member}.supportedLocalesOf requires locales and optional options"
        )));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(format!(
            "Intl.{member}.supportedLocalesOf options cannot be empty"
        )));
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
    Ok(Some(Expr::IntlStaticMethod {
        method,
        args: parsed,
    }))
}

fn parse_intl_formatter_construct(
    cursor: &mut Cursor<'_>,
    member: &str,
    kind: IntlFormatterKind,
    called_with_new: bool,
) -> Result<Option<Expr>> {
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
    Ok(Some(Expr::IntlFormatterConstruct {
        kind,
        locales,
        options,
        called_with_new,
    }))
}
