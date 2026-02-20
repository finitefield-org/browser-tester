use super::*;
pub(crate) struct ParsedCallbackParams {
    pub(crate) params: Vec<FunctionParam>,
    pub(crate) prologue: Vec<String>,
}

pub(crate) fn next_callback_temp_name(params: &[FunctionParam], seed: usize) -> String {
    let mut suffix = seed;
    loop {
        let candidate = format!("__bt_callback_arg_{suffix}");
        if params.iter().all(|param| param.name != candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

pub(crate) fn format_array_destructure_pattern(pattern: &[Option<String>]) -> String {
    let mut out = String::from("[");
    for (idx, item) in pattern.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        if let Some(name) = item {
            out.push_str(name);
        }
    }
    out.push(']');
    out
}

pub(crate) fn format_object_destructure_pattern(pattern: &[(String, String)]) -> String {
    let mut out = String::from("{");
    for (index, (source, target)) in pattern.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(source);
        if source != target {
            out.push_str(": ");
            out.push_str(target);
        }
    }
    out.push('}');
    out
}

pub(crate) fn inject_callback_param_prologue(
    body: String,
    concise_body: bool,
    prologue: &[String],
) -> (String, bool) {
    if prologue.is_empty() {
        return (body, concise_body);
    }

    let mut rewritten = String::new();
    for stmt in prologue {
        rewritten.push_str(stmt.trim());
        if !stmt.trim_end().ends_with(';') {
            rewritten.push(';');
        }
        rewritten.push('\n');
    }

    if concise_body {
        rewritten.push_str("return ");
        rewritten.push_str(body.trim());
        rewritten.push(';');
        (rewritten, false)
    } else {
        rewritten.push_str(&body);
        (rewritten, false)
    }
}

pub(crate) fn prepend_callback_param_prologue_stmts(
    mut stmts: Vec<Stmt>,
    prologue: &[String],
) -> Result<Vec<Stmt>> {
    if prologue.is_empty() {
        return Ok(stmts);
    }

    let mut src = String::new();
    for stmt in prologue {
        src.push_str(stmt.trim());
        if !stmt.trim_end().ends_with(';') {
            src.push(';');
        }
        src.push('\n');
    }
    let mut prefixed = parse_block_statements(&src)?;
    prefixed.append(&mut stmts);
    Ok(prefixed)
}

pub(crate) fn parse_callback_parameter_list(
    src: &str,
    max_params: usize,
    label: &str,
) -> Result<ParsedCallbackParams> {
    let parts = split_top_level_by_char(src.trim(), b',');
    if parts.len() == 1 && parts[0].trim().is_empty() {
        return Ok(ParsedCallbackParams {
            params: Vec::new(),
            prologue: Vec::new(),
        });
    }

    if parts.len() > max_params {
        return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
    }

    let mut params = Vec::new();
    let mut prologue = Vec::new();
    let mut bound_names: Vec<String> = Vec::new();
    let part_count = parts.len();
    for (index, raw) in parts.into_iter().enumerate() {
        let param = raw.trim();
        if param.is_empty() {
            return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
        }

        if let Some(rest_name) = param.strip_prefix("...") {
            let rest_name = rest_name.trim();
            if index + 1 != part_count || !is_ident(rest_name) {
                return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
            }
            params.push(FunctionParam {
                name: rest_name.to_string(),
                default: None,
                is_rest: true,
            });
            bound_names.push(rest_name.to_string());
            continue;
        }

        if let Some((eq_pos, op_len)) = find_top_level_assignment(param) {
            if op_len != 1 {
                return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
            }
            let name = param[..eq_pos].trim();
            let default_src = param[eq_pos + op_len..].trim();
            if default_src.is_empty() {
                return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
            }

            if is_ident(name) {
                params.push(FunctionParam {
                    name: name.to_string(),
                    default: Some(parse_expr(default_src)?),
                    is_rest: false,
                });
                bound_names.push(name.to_string());
                continue;
            }

            if name.starts_with('[') && name.ends_with(']') {
                let pattern = parse_array_destructure_pattern(name)?;
                let temp = next_callback_temp_name(&params, index);
                let pattern_src = format_array_destructure_pattern(&pattern);
                for bound_name in pattern.iter().flatten() {
                    if !bound_names.iter().any(|bound| bound == bound_name) {
                        prologue.push(format!("let {bound_name} = undefined;"));
                        bound_names.push(bound_name.clone());
                    }
                }
                prologue.push(format!("{pattern_src} = {temp};"));
                bound_names.push(temp.clone());
                params.push(FunctionParam {
                    name: temp,
                    default: Some(parse_expr(default_src)?),
                    is_rest: false,
                });
                continue;
            }

            if name.starts_with('{') && name.ends_with('}') {
                let pattern = parse_object_destructure_pattern(name)?;
                let temp = next_callback_temp_name(&params, index);
                let pattern_src = format_object_destructure_pattern(&pattern);
                for (_, bound_name) in &pattern {
                    if !bound_names.iter().any(|bound| bound == bound_name) {
                        prologue.push(format!("let {bound_name} = undefined;"));
                        bound_names.push(bound_name.clone());
                    }
                }
                prologue.push(format!("{pattern_src} = {temp};"));
                bound_names.push(temp.clone());
                params.push(FunctionParam {
                    name: temp,
                    default: Some(parse_expr(default_src)?),
                    is_rest: false,
                });
                continue;
            }

            return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
        }

        if is_ident(param) {
            params.push(FunctionParam {
                name: param.to_string(),
                default: None,
                is_rest: false,
            });
            bound_names.push(param.to_string());
            continue;
        }

        if param.starts_with('[') && param.ends_with(']') {
            let pattern = parse_array_destructure_pattern(param)?;
            let temp = next_callback_temp_name(&params, index);
            let pattern_src = format_array_destructure_pattern(&pattern);
            for name in pattern.iter().flatten() {
                if !bound_names.iter().any(|bound| bound == name) {
                    prologue.push(format!("let {name} = undefined;"));
                    bound_names.push(name.clone());
                }
            }
            prologue.push(format!("{pattern_src} = {temp};"));
            bound_names.push(temp.clone());
            params.push(FunctionParam {
                name: temp,
                default: None,
                is_rest: false,
            });
            continue;
        }

        if param.starts_with('{') && param.ends_with('}') {
            let pattern = parse_object_destructure_pattern(param)?;
            let temp = next_callback_temp_name(&params, index);
            let pattern_src = format_object_destructure_pattern(&pattern);
            for (_, name) in &pattern {
                if !bound_names.iter().any(|bound| bound == name) {
                    prologue.push(format!("let {name} = undefined;"));
                    bound_names.push(name.clone());
                }
            }
            prologue.push(format!("{pattern_src} = {temp};"));
            bound_names.push(temp.clone());
            params.push(FunctionParam {
                name: temp,
                default: None,
                is_rest: false,
            });
            continue;
        }

        return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
    }

    Ok(ParsedCallbackParams { params, prologue })
}
