use super::*;

pub(crate) fn parse_fetch_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("fetch") {
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
            "fetch requires one or two arguments".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "fetch options argument cannot be empty".into(),
        ));
    }

    let request = parse_expr(args[0].trim())?;
    let options = if args.len() == 2 {
        Some(parse_expr(args[1].trim())?)
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::Fetch {
        request: Box::new(request),
        options: options.map(Box::new),
    }))
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
    if cursor.peek() != Some(b'(') {
        // `window.matchMedia` property access should be handled by the generic
        // member-get parser, not treated as a call parse failure.
        return Ok(None);
    }

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
