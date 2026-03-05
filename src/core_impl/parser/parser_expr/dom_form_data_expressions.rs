use super::*;

pub(crate) fn parse_new_form_data_expr(
    src: &str,
) -> Result<Option<(Option<DomQuery>, Option<DomQuery>)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some((form, submitter)) = parse_new_form_data_target(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((form, submitter)))
}

pub(crate) fn parse_form_data_get_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    parse_form_data_method_expr(src, "get")
}

pub(crate) fn parse_form_data_has_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    parse_form_data_method_expr(src, "has")
}

pub(crate) fn parse_form_data_get_all_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    parse_form_data_method_expr(src, "getAll")
}

pub(crate) fn parse_form_data_get_all_length_expr(
    src: &str,
) -> Result<Option<(FormDataSource, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let Some(source) = parse_form_data_source(&mut cursor)? else {
        return Ok(None);
    };
    if matches!(&source, FormDataSource::Var(name) if name == "cookieStore") {
        return Ok(None);
    }

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if method != "getAll" {
        return Ok(None);
    }

    cursor.skip_ws();
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
    if args.len() != 1 {
        return match source {
            FormDataSource::New { .. } => Err(Error::ScriptParse(
                "FormData.getAll requires exactly one string argument".into(),
            )),
            FormDataSource::Var(_) => Ok(None),
        };
    }
    let arg = args[0].trim();
    let name = match parse_string_literal_exact(arg) {
        Ok(name) => name,
        Err(_) => {
            return match source {
                FormDataSource::New { .. } => Err(Error::ScriptParse(
                    "FormData.getAll requires exactly one string argument".into(),
                )),
                FormDataSource::Var(_) => Ok(None),
            };
        }
    };
    cursor.skip_ws();

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("length") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((source, name)))
}

pub(crate) fn parse_form_data_method_expr(
    src: &str,
    method: &str,
) -> Result<Option<(FormDataSource, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let Some(source) = parse_form_data_source(&mut cursor)? else {
        return Ok(None);
    };
    if matches!(&source, FormDataSource::Var(name) if name == "cookieStore") {
        return Ok(None);
    }

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(actual_method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if actual_method != method {
        return Ok(None);
    }
    cursor.skip_ws();
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
    if args.len() != 1 {
        return match source {
            FormDataSource::New { .. } => Err(Error::ScriptParse(format!(
                "FormData.{method} requires exactly one string argument"
            ))),
            FormDataSource::Var(_) => Ok(None),
        };
    }
    let arg = args[0].trim();
    let name = match parse_string_literal_exact(arg) {
        Ok(name) => name,
        Err(_) => {
            return match source {
                FormDataSource::New { .. } => Err(Error::ScriptParse(format!(
                    "FormData.{method} requires exactly one string argument"
                ))),
                FormDataSource::Var(_) => Ok(None),
            };
        }
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((source, name)))
}

pub(crate) fn parse_form_data_source(cursor: &mut Cursor<'_>) -> Result<Option<FormDataSource>> {
    if let Some((form, submitter)) = parse_new_form_data_target(cursor)? {
        return Ok(Some(FormDataSource::New { form, submitter }));
    }

    if let Some(var_name) = cursor.parse_identifier() {
        return Ok(Some(FormDataSource::Var(var_name)));
    }

    Ok(None)
}

pub(crate) fn parse_new_form_data_target(
    cursor: &mut Cursor<'_>,
) -> Result<Option<(Option<DomQuery>, Option<DomQuery>)>> {
    cursor.skip_ws();
    let start = cursor.pos();

    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            cursor.set_pos(start);
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("FormData") {
        cursor.set_pos(start);
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };
    if args.len() > 2 {
        return Err(Error::ScriptParse(
            "new FormData supports zero, one, or two arguments".into(),
        ));
    }

    if args.is_empty() {
        return Ok(Some((None, None)));
    }

    let form_arg = args[0].trim();
    let mut arg_cursor = Cursor::new(form_arg);
    arg_cursor.skip_ws();
    let form = parse_form_elements_base(&mut arg_cursor)?;
    arg_cursor.skip_ws();
    if !arg_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported FormData form argument: {form_arg}"
        )));
    }

    if args.len() == 1 {
        return Ok(Some((Some(form), None)));
    }

    let submitter_arg = args[1].trim();
    let mut submitter_cursor = Cursor::new(submitter_arg);
    submitter_cursor.skip_ws();
    let submitter = parse_form_elements_base(&mut submitter_cursor)?;
    submitter_cursor.skip_ws();
    if !submitter_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported FormData submitter argument: {submitter_arg}"
        )));
    }

    Ok(Some((Some(form), Some(submitter))))
}
