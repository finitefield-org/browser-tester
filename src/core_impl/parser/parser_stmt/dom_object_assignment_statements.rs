use super::*;
use crate::core_impl::parser::parser_expr::collect_top_level_char_positions;

pub(crate) fn parse_form_data_append_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    let Some(target_var) = cursor.parse_identifier() else {
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
    if method != "append" {
        return Ok(None);
    }

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Ok(None);
    }

    let name = parse_expr(args[0].trim())?;
    let value = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported FormData.append statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::FormDataAppend {
        target_var,
        name,
        value,
    }))
}

pub(crate) fn parse_dom_method_call_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();

    let Some(method_name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let (method, accepts_optional_arg) = match method_name.as_str() {
        "focus" => (DomMethod::Focus, false),
        "blur" => (DomMethod::Blur, false),
        "click" => (DomMethod::Click, false),
        "scrollIntoView" => (DomMethod::ScrollIntoView, true),
        "submit" => (DomMethod::Submit, false),
        "requestSubmit" => (DomMethod::RequestSubmit, true),
        "reset" => (DomMethod::Reset, false),
        "show" => (DomMethod::Show, false),
        "showModal" => (DomMethod::ShowModal, false),
        "close" => (DomMethod::Close, true),
        "requestClose" => (DomMethod::RequestClose, true),
        _ => return Ok(None),
    };

    cursor.skip_ws();
    let args = cursor.read_balanced_block(b'(', b')')?;
    let arg = if accepts_optional_arg {
        let parsed_args = split_top_level_by_char(&args, b',');
        if parsed_args.len() == 1 && parsed_args[0].trim().is_empty() {
            None
        } else {
            if parsed_args.len() != 1 || parsed_args[0].trim().is_empty() {
                return Err(Error::ScriptParse(format!(
                    "{} accepts zero or one argument: {stmt}",
                    method_name
                )));
            }
            Some(parse_expr(parsed_args[0].trim())?)
        }
    } else {
        if !args.trim().is_empty() {
            return Err(Error::ScriptParse(format!(
                "{} takes no arguments: {stmt}",
                method_name
            )));
        }
        None
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported {} statement tail: {stmt}",
            method_name
        )));
    }

    Ok(Some(Stmt::DomMethodCall {
        target,
        method,
        arg,
    }))
}

pub(crate) fn parse_dom_assignment(stmt: &str) -> Result<Option<Stmt>> {
    let Some((eq_pos, op_len)) = find_top_level_assignment(stmt) else {
        return Ok(None);
    };

    let lhs = stmt[..eq_pos].trim();
    let rhs = stmt[eq_pos + op_len..].trim();

    if lhs.is_empty() {
        return Ok(None);
    }

    let Some((target, prop)) = parse_dom_access(lhs)? else {
        return Ok(None);
    };

    let rhs_expr = parse_expr(rhs)?;
    let op = &stmt[eq_pos..eq_pos + op_len];
    let expr = if op_len == 1 {
        rhs_expr
    } else {
        let lhs_expr = Expr::DomRead {
            target: target.clone(),
            prop: prop.clone(),
        };
        match op {
            "+=" => append_concat_expr(lhs_expr, rhs_expr),
            "-=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Sub,
                right: Box::new(rhs_expr),
            },
            "*=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Mul,
                right: Box::new(rhs_expr),
            },
            "/=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Div,
                right: Box::new(rhs_expr),
            },
            "%=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Mod,
                right: Box::new(rhs_expr),
            },
            "**=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Pow,
                right: Box::new(rhs_expr),
            },
            _ => rhs_expr,
        }
    };
    Ok(Some(Stmt::DomAssign { target, prop, expr }))
}

pub(crate) fn parse_object_assignment_target(lhs: &str) -> Result<Option<(String, Vec<Expr>)>> {
    let mut cursor = Cursor::new(lhs);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };

    let mut path = Vec::new();
    loop {
        cursor.skip_ws();
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            let Some(prop) = cursor.parse_identifier() else {
                return Ok(None);
            };
            path.push(Expr::String(prop));
            continue;
        }

        if cursor.peek() == Some(b'[') {
            let key_src = cursor.read_balanced_block(b'[', b']')?;
            let key_src = key_src.trim();
            if key_src.is_empty() {
                return Err(Error::ScriptParse(
                    "object assignment key cannot be empty".into(),
                ));
            }
            path.push(parse_expr(key_src)?);
            continue;
        }
        break;
    }

    cursor.skip_ws();
    if !cursor.eof() || path.is_empty() {
        return Ok(None);
    }
    Ok(Some((target, path)))
}

pub(crate) fn parse_object_assign(stmt: &str) -> Result<Option<Stmt>> {
    let Some((eq_pos, op_len)) = find_top_level_assignment(stmt) else {
        return Ok(None);
    };

    let lhs = stmt[..eq_pos].trim();
    let rhs = stmt[eq_pos + op_len..].trim();
    if lhs.is_empty() {
        return Ok(None);
    }

    let Some((target, path)) = parse_object_assignment_target(lhs)? else {
        return Ok(None);
    };

    let rhs_expr = parse_expr(rhs)?;
    let op = &stmt[eq_pos..eq_pos + op_len];
    let expr = if op_len == 1 {
        rhs_expr
    } else {
        let mut static_path = Vec::with_capacity(path.len());
        for segment in &path {
            if let Expr::String(name) = segment {
                static_path.push(name.clone());
            } else {
                return Err(Error::ScriptParse(
                    "compound object assignment requires a static key".into(),
                ));
            }
        }
        let lhs_expr = if static_path.len() == 1 {
            Expr::ObjectGet {
                target: target.clone(),
                key: static_path.remove(0),
            }
        } else {
            Expr::ObjectPathGet {
                target: target.clone(),
                path: static_path,
            }
        };
        match op {
            "+=" => append_concat_expr(lhs_expr, rhs_expr),
            "-=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Sub,
                right: Box::new(rhs_expr),
            },
            "*=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Mul,
                right: Box::new(rhs_expr),
            },
            "/=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Div,
                right: Box::new(rhs_expr),
            },
            "%=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Mod,
                right: Box::new(rhs_expr),
            },
            "**=" => Expr::Binary {
                left: Box::new(lhs_expr),
                op: BinaryOp::Pow,
                right: Box::new(rhs_expr),
            },
            _ => return Ok(None),
        }
    };
    Ok(Some(Stmt::ObjectAssign { target, path, expr }))
}

pub(crate) fn parse_private_assign(stmt: &str) -> Result<Option<Stmt>> {
    let Some((eq_pos, op_len)) = find_top_level_assignment(stmt) else {
        return Ok(None);
    };
    if op_len != 1 {
        return Ok(None);
    }

    let lhs = stmt[..eq_pos].trim();
    let rhs = stmt[eq_pos + op_len..].trim();
    if lhs.is_empty() || rhs.is_empty() {
        return Ok(None);
    }

    let dots = collect_top_level_char_positions(lhs, b'.');
    for dot in dots.into_iter().rev() {
        let Some(base_src) = lhs.get(..dot) else {
            continue;
        };
        let base_src = base_src.trim();
        if base_src.is_empty() {
            continue;
        }
        let Some(tail) = lhs.get(dot + 1..) else {
            continue;
        };
        let tail = tail.trim();
        let Some(private_name) = tail.strip_prefix('#') else {
            continue;
        };
        if !is_ident(private_name) {
            continue;
        }
        return Ok(Some(Stmt::PrivateAssign {
            target: parse_expr(base_src)?,
            member: private_name.to_string(),
            expr: parse_expr(rhs)?,
        }));
    }

    Ok(None)
}
