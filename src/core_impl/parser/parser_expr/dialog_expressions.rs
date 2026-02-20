use super::*;

pub(crate) fn parse_structured_clone_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(
        src,
        "structuredClone",
        "structuredClone requires exactly one argument",
    )
}

pub(crate) fn parse_alert_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "alert", "alert requires exactly one argument")
}

pub(crate) fn parse_confirm_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "confirm", "confirm requires exactly one argument")
}

pub(crate) fn parse_prompt_expr(src: &str) -> Result<Option<(Expr, Option<Expr>)>> {
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
