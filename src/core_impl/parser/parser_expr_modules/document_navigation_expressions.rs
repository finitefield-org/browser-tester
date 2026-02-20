use super::*;
pub(crate) fn parse_element_ref_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    if matches!(target, DomQuery::DocumentRoot) || is_non_dom_var_target(&target) {
        return Ok(None);
    }
    Ok(Some(target))
}

pub(crate) fn parse_document_create_element_expr(src: &str) -> Result<Option<String>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("document") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("createElement") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let tag_name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(tag_name.to_ascii_lowercase()))
}

pub(crate) fn parse_document_create_text_node_expr(src: &str) -> Result<Option<String>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("document") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("createTextNode") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let text = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(text))
}

pub(crate) fn parse_document_has_focus_expr(src: &str) -> Result<bool> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(false);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("document") {
        return Ok(false);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(false);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("hasFocus") {
        return Ok(false);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(false);
        }
    }
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    if !args_src.trim().is_empty() {
        return Err(Error::ScriptParse(
            "document.hasFocus takes no arguments".into(),
        ));
    }
    cursor.skip_ws();
    Ok(cursor.eof())
}

pub(crate) fn parse_location_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if !parse_location_base(&mut cursor) {
        return Ok(None);
    }

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method_name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let method = match method_name.as_str() {
        "assign" => LocationMethod::Assign,
        "reload" => LocationMethod::Reload,
        "replace" => LocationMethod::Replace,
        "toString" => LocationMethod::ToString,
        _ => return Ok(None),
    };

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    let args = if args.len() == 1 && args[0].trim().is_empty() {
        Vec::new()
    } else {
        args
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let url = match method {
        LocationMethod::Assign | LocationMethod::Replace => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(format!(
                    "location.{} requires exactly one argument",
                    method_name
                )));
            }
            Some(Box::new(parse_expr(args[0].trim())?))
        }
        LocationMethod::Reload | LocationMethod::ToString => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "location.{} takes no arguments",
                    method_name
                )));
            }
            None
        }
    };

    Ok(Some(Expr::LocationMethodCall { method, url }))
}

pub(crate) fn parse_location_base(cursor: &mut Cursor<'_>) -> bool {
    let start = cursor.pos();

    if cursor.consume_ascii("location") {
        if cursor.peek().is_none_or(|ch| !is_ident_char(ch)) {
            return true;
        }
        cursor.set_pos(start);
    }

    if cursor.consume_ascii("document") {
        if cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            if cursor.consume_ascii("location") && cursor.peek().is_none_or(|ch| !is_ident_char(ch))
            {
                return true;
            }
        }
        cursor.set_pos(start);
    }

    if cursor.consume_ascii("window") {
        if cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if cursor.consume_ascii("location") && cursor.peek().is_none_or(|ch| !is_ident_char(ch)) {
            return true;
        }
        cursor.set_pos(start);
        if cursor.consume_ascii("window") {
            cursor.skip_ws();
            if !cursor.consume_byte(b'.') {
                cursor.set_pos(start);
                return false;
            }
            cursor.skip_ws();
            if !cursor.consume_ascii("document") {
                cursor.set_pos(start);
                return false;
            }
            cursor.skip_ws();
            if !cursor.consume_byte(b'.') {
                cursor.set_pos(start);
                return false;
            }
            cursor.skip_ws();
            if cursor.consume_ascii("location") && cursor.peek().is_none_or(|ch| !is_ident_char(ch))
            {
                return true;
            }
        }
        cursor.set_pos(start);
    }

    false
}

pub(crate) fn parse_history_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if !parse_history_base(&mut cursor) {
        return Ok(None);
    }

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method_name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let method = match method_name.as_str() {
        "back" => HistoryMethod::Back,
        "forward" => HistoryMethod::Forward,
        "go" => HistoryMethod::Go,
        "pushState" => HistoryMethod::PushState,
        "replaceState" => HistoryMethod::ReplaceState,
        _ => return Ok(None),
    };

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    let args = if args.len() == 1 && args[0].trim().is_empty() {
        Vec::new()
    } else {
        args
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let mut parsed_args = Vec::new();
    match method {
        HistoryMethod::Back | HistoryMethod::Forward => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "history.{} takes no arguments",
                    method_name
                )));
            }
        }
        HistoryMethod::Go => {
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "history.go accepts zero or one argument".into(),
                ));
            }
            if let Some(arg) = args.first() {
                if arg.trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "history.go argument cannot be empty".into(),
                    ));
                }
                parsed_args.push(parse_expr(arg.trim())?);
            }
        }
        HistoryMethod::PushState | HistoryMethod::ReplaceState => {
            if args.len() < 2 || args.len() > 3 {
                return Err(Error::ScriptParse(format!(
                    "history.{} requires 2 or 3 arguments",
                    method_name
                )));
            }
            for arg in args {
                let arg = arg.trim();
                if arg.is_empty() {
                    return Err(Error::ScriptParse(format!(
                        "history.{} arguments cannot be empty",
                        method_name
                    )));
                }
                parsed_args.push(parse_expr(arg)?);
            }
        }
    }

    Ok(Some(Expr::HistoryMethodCall {
        method,
        args: parsed_args,
    }))
}

pub(crate) fn parse_history_base(cursor: &mut Cursor<'_>) -> bool {
    let start = cursor.pos();

    if cursor.consume_ascii("history") {
        if cursor.peek().is_none_or(|ch| !is_ident_char(ch)) {
            return true;
        }
        cursor.set_pos(start);
    }

    if cursor.consume_ascii("window") {
        if cursor.peek().is_some_and(is_ident_char) {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            cursor.set_pos(start);
            return false;
        }
        cursor.skip_ws();
        if cursor.consume_ascii("history") && cursor.peek().is_none_or(|ch| !is_ident_char(ch)) {
            return true;
        }
    }

    cursor.set_pos(start);
    false
}

pub(crate) fn parse_clipboard_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if !parse_clipboard_base(&mut cursor) {
        return Ok(None);
    }

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method_name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let method = match method_name.as_str() {
        "readText" => ClipboardMethod::ReadText,
        "writeText" => ClipboardMethod::WriteText,
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    let args = if args.len() == 1 && args[0].trim().is_empty() {
        Vec::new()
    } else {
        args
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let mut parsed_args = Vec::new();
    match method {
        ClipboardMethod::ReadText => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "navigator.clipboard.readText takes no arguments".into(),
                ));
            }
        }
        ClipboardMethod::WriteText => {
            if args.len() != 1 {
                return Err(Error::ScriptParse(
                    "navigator.clipboard.writeText requires exactly one argument".into(),
                ));
            }
            if args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "navigator.clipboard.writeText argument cannot be empty".into(),
                ));
            }
            parsed_args.push(parse_expr(args[0].trim())?);
        }
    }

    Ok(Some(Expr::ClipboardMethodCall {
        method,
        args: parsed_args,
    }))
}
