use super::*;

pub(crate) fn parse_date_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
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

    let expr = match method.as_str() {
        "getTime" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("getTime does not take arguments".into()));
            }
            Expr::DateGetTime(target)
        }
        "setTime" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "setTime requires exactly one argument".into(),
                ));
            }
            Expr::DateSetTime {
                target,
                value: Box::new(parse_expr(args[0].trim())?),
            }
        }
        "toISOString" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "toISOString does not take arguments".into(),
                ));
            }
            Expr::DateToIsoString(target)
        }
        "getUTCFullYear" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getUTCFullYear does not take arguments".into(),
                ));
            }
            Expr::DateGetUTCFullYear(target)
        }
        "getFullYear" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getFullYear does not take arguments".into(),
                ));
            }
            Expr::DateGetFullYear(target)
        }
        "getMonth" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getMonth does not take arguments".into(),
                ));
            }
            Expr::DateGetMonth(target)
        }
        "getDate" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("getDate does not take arguments".into()));
            }
            Expr::DateGetDate(target)
        }
        "getHours" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getHours does not take arguments".into(),
                ));
            }
            Expr::DateGetHours(target)
        }
        "getMinutes" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getMinutes does not take arguments".into(),
                ));
            }
            Expr::DateGetMinutes(target)
        }
        "getSeconds" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getSeconds does not take arguments".into(),
                ));
            }
            Expr::DateGetSeconds(target)
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}
