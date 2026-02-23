use super::*;
pub(crate) struct ParsedCallbackParams {
    pub(crate) params: Vec<FunctionParam>,
    pub(crate) prologue: Vec<String>,
}

#[derive(Debug, Clone)]
struct ArrayPatternBinding {
    name: String,
    default_src: Option<String>,
}

#[derive(Debug, Clone)]
struct ObjectPatternBinding {
    source: String,
    target: String,
    default_src: Option<String>,
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
            if index + 1 != part_count {
                return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
            }

            if is_ident(rest_name) {
                params.push(FunctionParam {
                    name: rest_name.to_string(),
                    default: None,
                    is_rest: true,
                });
                bound_names.push(rest_name.to_string());
                continue;
            }

            if rest_name.starts_with('[') && rest_name.ends_with(']') {
                let pattern = parse_callback_array_destructure_pattern(rest_name)?;
                let temp = next_callback_temp_name(&params, index);
                append_array_destructure_param_prologue(
                    &mut prologue,
                    &pattern,
                    &temp,
                    &mut bound_names,
                );
                bound_names.push(temp.clone());
                params.push(FunctionParam {
                    name: temp,
                    default: None,
                    is_rest: true,
                });
                continue;
            }

            if rest_name.starts_with('{') && rest_name.ends_with('}') {
                let pattern = parse_callback_object_destructure_pattern(rest_name)?;
                let temp = next_callback_temp_name(&params, index);
                append_object_destructure_param_prologue(
                    &mut prologue,
                    &pattern,
                    &temp,
                    &mut bound_names,
                );
                bound_names.push(temp.clone());
                params.push(FunctionParam {
                    name: temp,
                    default: None,
                    is_rest: true,
                });
                continue;
            }

            return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
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
                let pattern = parse_callback_array_destructure_pattern(name)?;
                let temp = next_callback_temp_name(&params, index);
                append_array_destructure_param_prologue(
                    &mut prologue,
                    &pattern,
                    &temp,
                    &mut bound_names,
                );
                bound_names.push(temp.clone());
                params.push(FunctionParam {
                    name: temp,
                    default: Some(parse_expr(default_src)?),
                    is_rest: false,
                });
                continue;
            }

            if name.starts_with('{') && name.ends_with('}') {
                let pattern = parse_callback_object_destructure_pattern(name)?;
                let temp = next_callback_temp_name(&params, index);
                append_object_destructure_param_prologue(
                    &mut prologue,
                    &pattern,
                    &temp,
                    &mut bound_names,
                );
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
            let pattern = parse_callback_array_destructure_pattern(param)?;
            let temp = next_callback_temp_name(&params, index);
            append_array_destructure_param_prologue(&mut prologue, &pattern, &temp, &mut bound_names);
            bound_names.push(temp.clone());
            params.push(FunctionParam {
                name: temp,
                default: None,
                is_rest: false,
            });
            continue;
        }

        if param.starts_with('{') && param.ends_with('}') {
            let pattern = parse_callback_object_destructure_pattern(param)?;
            let temp = next_callback_temp_name(&params, index);
            append_object_destructure_param_prologue(
                &mut prologue,
                &pattern,
                &temp,
                &mut bound_names,
            );
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

fn parse_callback_array_destructure_pattern(
    pattern: &str,
) -> Result<Vec<Option<ArrayPatternBinding>>> {
    let mut cursor = Cursor::new(pattern);
    cursor.skip_ws();
    let items_src = cursor.read_balanced_block(b'[', b']')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "invalid array destructuring pattern: {pattern}"
        )));
    }

    let mut items = split_top_level_by_char(&items_src, b',');
    while items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty()) {
        items.pop();
    }
    if items.len() == 1 && items[0].trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut bindings = Vec::with_capacity(items.len());
    for item in items {
        let item = item.trim();
        if item.is_empty() {
            bindings.push(None);
            continue;
        }

        let (name, default_src) = if let Some((eq_pos, op_len)) = find_top_level_assignment(item) {
            if op_len != 1 {
                return Err(Error::ScriptParse(format!(
                    "array destructuring target must be an identifier: {item}"
                )));
            }
            let name = item[..eq_pos].trim();
            let default_src = item[eq_pos + op_len..].trim();
            if !is_ident(name) || default_src.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "array destructuring target must be an identifier: {item}"
                )));
            }
            (name.to_string(), Some(default_src.to_string()))
        } else {
            if !is_ident(item) {
                return Err(Error::ScriptParse(format!(
                    "array destructuring target must be an identifier: {item}"
                )));
            }
            (item.to_string(), None)
        };

        bindings.push(Some(ArrayPatternBinding { name, default_src }));
    }

    Ok(bindings)
}

fn parse_callback_object_destructure_pattern(pattern: &str) -> Result<Vec<ObjectPatternBinding>> {
    let mut cursor = Cursor::new(pattern);
    cursor.skip_ws();
    let items_src = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "invalid object destructuring pattern: {pattern}"
        )));
    }

    let mut items = split_top_level_by_char(&items_src, b',');
    while items.len() > 1 && items.last().is_some_and(|item| item.trim().is_empty()) {
        items.pop();
    }
    if items.len() == 1 && items[0].trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut bindings = Vec::with_capacity(items.len());
    for item in items {
        let item = item.trim();
        if item.is_empty() {
            return Err(Error::ScriptParse(
                "object destructuring pattern does not support empty entries".into(),
            ));
        }

        if let Some(colon) = find_first_top_level_colon(item) {
            let source = item[..colon].trim();
            let target_src = item[colon + 1..].trim();
            if !is_ident(source) {
                return Err(Error::ScriptParse(format!(
                    "object destructuring entry must be identifier or identifier: identifier: {item}"
                )));
            }

            let (target, default_src) =
                if let Some((eq_pos, op_len)) = find_top_level_assignment(target_src) {
                    if op_len != 1 {
                        return Err(Error::ScriptParse(format!(
                            "object destructuring entry must be identifier or identifier: identifier: {item}"
                        )));
                    }
                    let target = target_src[..eq_pos].trim();
                    let default_src = target_src[eq_pos + op_len..].trim();
                    if !is_ident(target) || default_src.is_empty() {
                        return Err(Error::ScriptParse(format!(
                            "object destructuring entry must be identifier or identifier: identifier: {item}"
                        )));
                    }
                    (target.to_string(), Some(default_src.to_string()))
                } else {
                    if !is_ident(target_src) {
                        return Err(Error::ScriptParse(format!(
                            "object destructuring entry must be identifier or identifier: identifier: {item}"
                        )));
                    }
                    (target_src.to_string(), None)
                };

            bindings.push(ObjectPatternBinding {
                source: source.to_string(),
                target,
                default_src,
            });
            continue;
        }

        let (name, default_src) = if let Some((eq_pos, op_len)) = find_top_level_assignment(item) {
            if op_len != 1 {
                return Err(Error::ScriptParse(format!(
                    "object destructuring entry must be identifier or identifier: identifier: {item}"
                )));
            }
            let name = item[..eq_pos].trim();
            let default_src = item[eq_pos + op_len..].trim();
            if !is_ident(name) || default_src.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "object destructuring entry must be identifier or identifier: identifier: {item}"
                )));
            }
            (name.to_string(), Some(default_src.to_string()))
        } else {
            if !is_ident(item) {
                return Err(Error::ScriptParse(format!(
                    "object destructuring entry must be identifier or identifier: identifier: {item}"
                )));
            }
            (item.to_string(), None)
        };

        bindings.push(ObjectPatternBinding {
            source: name.clone(),
            target: name,
            default_src,
        });
    }

    Ok(bindings)
}

fn append_array_destructure_param_prologue(
    prologue: &mut Vec<String>,
    pattern: &[Option<ArrayPatternBinding>],
    temp: &str,
    bound_names: &mut Vec<String>,
) {
    for (index, binding) in pattern.iter().enumerate() {
        let Some(binding) = binding else {
            continue;
        };
        if !bound_names.iter().any(|bound| bound == &binding.name) {
            prologue.push(format!("let {} = undefined;", binding.name));
            bound_names.push(binding.name.clone());
        }
        prologue.push(format!("{} = {}[{}];", binding.name, temp, index));
        if let Some(default_src) = &binding.default_src {
            prologue.push(format!(
                "if ({} === undefined) {} = {};",
                binding.name, binding.name, default_src
            ));
        }
    }
}

fn append_object_destructure_param_prologue(
    prologue: &mut Vec<String>,
    pattern: &[ObjectPatternBinding],
    temp: &str,
    bound_names: &mut Vec<String>,
) {
    for binding in pattern {
        if !bound_names.iter().any(|bound| bound == &binding.target) {
            prologue.push(format!("let {} = undefined;", binding.target));
            bound_names.push(binding.target.clone());
        }
        prologue.push(format!("{} = {}.{};", binding.target, temp, binding.source));
        if let Some(default_src) = &binding.default_src {
            prologue.push(format!(
                "if ({} === undefined) {} = {};",
                binding.target, binding.target, default_src
            ));
        }
    }
}
