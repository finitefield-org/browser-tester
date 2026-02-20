use super::*;

pub(crate) fn parse_fetch_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "fetch", "fetch requires exactly one argument")
}

pub(crate) fn parse_match_media_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("matchMedia") {
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
            "matchMedia requires exactly one argument".into(),
        ));
    }
    let query = parse_expr(args[0].trim())?;

    cursor.skip_ws();
    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(prop_name) = cursor.parse_identifier() else {
            return Ok(None);
        };
        let prop = match prop_name.as_str() {
            "matches" => MatchMediaProp::Matches,
            "media" => MatchMediaProp::Media,
            _ => return Ok(None),
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MatchMediaProp {
            query: Box::new(query),
            prop,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::MatchMedia(Box::new(query))))
}
