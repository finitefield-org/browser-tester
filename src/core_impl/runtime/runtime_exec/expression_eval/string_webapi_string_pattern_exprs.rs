use super::*;

impl Harness {
    pub(crate) fn try_eval_string_pattern_exprs(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::StringIncludes {
                    value,
                    search,
                    position,
                } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let search = self.eval_expr(search, env, event_param, event)?;
                    if let Value::Array(values) = &value {
                        let values_ref = values.borrow();
                        let len = values_ref.len() as i64;
                        let mut position = position
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .unwrap_or(0);
                        if position < 0 {
                            position = (len + position).max(0);
                        }
                        let position = position.min(len) as usize;
                        for item in values_ref.iter().skip(position) {
                            if self.strict_equal(item, &search) {
                                return Ok(Value::Bool(true));
                            }
                        }
                        return Ok(Value::Bool(false));
                    }
                    if let Value::TypedArray(values) = &value {
                        let values_vec = self.typed_array_snapshot(values)?;
                        let len = values_vec.len() as i64;
                        let mut position = position
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .unwrap_or(0);
                        if position < 0 {
                            position = (len + position).max(0);
                        }
                        let position = position.min(len) as usize;
                        for item in values_vec.iter().skip(position) {
                            if self.strict_equal(item, &search) {
                                return Ok(Value::Bool(true));
                            }
                        }
                        return Ok(Value::Bool(false));
                    }
                    let value = self.coerce_string_method_receiver(&value)?;
                    if self.is_regexp_like_for_string_prefix_search(&search)? {
                        return Err(Error::ScriptRuntime(
                            "First argument to String.prototype.includes must not be a regular expression"
                                .into(),
                        ));
                    }
                    let search = self.coerce_to_string_for_tostring(&search)?;
                    let len = Self::string_char_len(&value) as i64;
                    let mut position = position
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .unwrap_or(0);
                    if position < 0 {
                        position = 0;
                    }
                    let position = position.min(len) as usize;
                    let position_byte = Self::char_index_to_byte(&value, position);
                    Ok(Value::Bool(value[position_byte..].contains(&search)))
                }
                Expr::StringStartsWith {
                    value,
                    search,
                    position,
                } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let search = self.eval_expr(search, env, event_param, event)?;
                    if self.is_regexp_like_for_string_prefix_search(&search)? {
                        return Err(Error::ScriptRuntime(
                            "First argument to String.prototype.startsWith must not be a regular expression"
                                .into(),
                        ));
                    }
                    let search = self.coerce_to_string_for_tostring(&search)?;
                    let len = Self::string_char_len(&value) as i64;
                    let mut position = position
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .unwrap_or(0);
                    if position < 0 {
                        position = 0;
                    }
                    let position = position.min(len) as usize;
                    let position_byte = Self::char_index_to_byte(&value, position);
                    Ok(Value::Bool(value[position_byte..].starts_with(&search)))
                }
                Expr::StringEndsWith {
                    value,
                    search,
                    length,
                } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let search = self.eval_expr(search, env, event_param, event)?;
                    if self.is_regexp_like_for_string_prefix_search(&search)? {
                        return Err(Error::ScriptRuntime(
                            "First argument to String.prototype.endsWith must not be a regular expression"
                                .into(),
                        ));
                    }
                    let search = self.coerce_to_string_for_tostring(&search)?;
                    let len = Self::string_char_len(&value);
                    let end = length
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| {
                            if value < 0 {
                                0
                            } else {
                                (value as usize).min(len)
                            }
                        })
                        .unwrap_or(len);
                    let hay = Self::substring_chars(&value, 0, end);
                    Ok(Value::Bool(hay.ends_with(&search)))
                }
                Expr::StringMatch { value, pattern } => {
                    let target_value = self.eval_expr(value, env, event_param, event)?;
                    let pattern = self.eval_expr(pattern, env, event_param, event)?;
                    if let Value::Object(object) = &target_value {
                        if let Some(value) = self.eval_cache_storage_member_call(
                            object,
                            "match",
                            std::slice::from_ref(&pattern),
                        )? {
                            return Ok(value);
                        }
                        if let Some(value) = self.eval_cache_member_call(
                            object,
                            "match",
                            std::slice::from_ref(&pattern),
                        )? {
                            return Ok(value);
                        }
                    }
                    let text = self.coerce_string_method_receiver(&target_value)?;
                    if let Some(result) = self.call_string_symbol_method(
                        &pattern,
                        SymbolStaticProperty::Match,
                        &text,
                        &[],
                        event,
                    )? {
                        return Ok(result);
                    }
                    self.eval_string_match(&text, pattern)
                }
                Expr::StringSplit {
                    value,
                    separator,
                    limit,
                } => {
                    let text = self.eval_expr(value, env, event_param, event)?;
                    let text = self.coerce_string_method_receiver(&text)?;
                    let separator = separator
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?;
                    let limit_value = limit
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .unwrap_or(Value::Undefined);
                    if let Some(separator_value) = &separator {
                        if !matches!(separator_value, Value::Undefined) {
                            if let Some(result) = self.call_string_symbol_method(
                                separator_value,
                                SymbolStaticProperty::Split,
                                &text,
                                std::slice::from_ref(&limit_value),
                                event,
                            )? {
                                return Ok(result);
                            }
                        }
                    }
                    let limit = if matches!(limit_value, Value::Undefined) {
                        None
                    } else {
                        Some(Self::value_to_i64(&limit_value))
                    };
                    let parts = match separator {
                        None => Self::split_string(&text, None, limit),
                        Some(Value::RegExp(regex)) => {
                            Self::split_string_with_regex(&text, &regex, limit)?
                        }
                        Some(value) => Self::split_string(
                            &text,
                            Some(self.coerce_to_string_for_tostring(&value)?),
                            limit,
                        ),
                    };
                    Ok(Self::new_array_value(parts))
                }
                Expr::StringReplace { value, from, to } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let to = self.eval_expr(to, env, event_param, event)?;
                    let from = self.eval_expr(from, env, event_param, event)?;
                    if let Some(result) = self.call_string_symbol_method(
                        &from,
                        SymbolStaticProperty::Replace,
                        &value,
                        std::slice::from_ref(&to),
                        event,
                    )? {
                        return Ok(result);
                    }
                    let replaced = if self.is_callable_value(&to) {
                        match from {
                            Value::RegExp(regex) => {
                                self.replace_string_with_regex_callback(&value, &regex, &to, event)?
                            }
                            other => {
                                let from = self.coerce_to_string_for_tostring(&other)?;
                                self.replace_string_with_string_callback(
                                    &value, &from, &to, false, event,
                                )?
                            }
                        }
                    } else {
                        let replacement = self.coerce_to_string_for_tostring(&to)?;
                        match from {
                            Value::RegExp(regex) => {
                                Self::replace_string_with_regex(&value, &regex, &replacement)?
                            }
                            other => value.replacen(
                                &self.coerce_to_string_for_tostring(&other)?,
                                &replacement,
                                1,
                            ),
                        }
                    };
                    Ok(Value::String(replaced))
                }
                Expr::StringReplaceAll { value, from, to } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let to = self.eval_expr(to, env, event_param, event)?;
                    let from = self.eval_expr(from, env, event_param, event)?;
                    if let Value::RegExp(regex) = &from {
                        if !regex.borrow().global {
                            return Err(Error::ScriptRuntime(
                                "String.prototype.replaceAll called with a non-global RegExp argument"
                                    .into(),
                            ));
                        }
                    }
                    if let Some(result) = self.call_string_symbol_method(
                        &from,
                        SymbolStaticProperty::Replace,
                        &value,
                        std::slice::from_ref(&to),
                        event,
                    )? {
                        return Ok(result);
                    }
                    let replaced = if self.is_callable_value(&to) {
                        match from {
                            Value::RegExp(regex) => {
                                self.replace_string_with_regex_callback(&value, &regex, &to, event)?
                            }
                            other => {
                                let from = self.coerce_to_string_for_tostring(&other)?;
                                self.replace_string_with_string_callback(
                                    &value, &from, &to, true, event,
                                )?
                            }
                        }
                    } else {
                        let replacement = self.coerce_to_string_for_tostring(&to)?;
                        match from {
                            Value::RegExp(regex) => {
                                Self::replace_string_with_regex(&value, &regex, &replacement)?
                            }
                            other => {
                                let from = self.coerce_to_string_for_tostring(&other)?;
                                if from.is_empty() {
                                    let mut out = String::new();
                                    for ch in value.chars() {
                                        out.push_str(&replacement);
                                        out.push(ch);
                                    }
                                    out.push_str(&replacement);
                                    out
                                } else {
                                    value.replace(&from, &replacement)
                                }
                            }
                        }
                    };
                    Ok(Value::String(replaced))
                }
                Expr::StringIndexOf {
                    value,
                    search,
                    position,
                } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let search = self.eval_expr(search, env, event_param, event)?;
                    let search = self.coerce_to_string_for_tostring(&search)?;
                    let len = Self::string_char_len(&value) as i64;
                    let mut position = position
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .unwrap_or(0);
                    if position < 0 {
                        position = 0;
                    }
                    let position = position.min(len) as usize;
                    Ok(Value::Number(
                        Self::string_index_of(&value, &search, position)
                            .map(|value| value as i64)
                            .unwrap_or(-1),
                    ))
                }
                Expr::StringLastIndexOf {
                    value,
                    search,
                    position,
                } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let search = self.eval_expr(search, env, event_param, event)?;
                    let search = self.coerce_to_string_for_tostring(&search)?;
                    let len = Self::string_char_len(&value) as i64;
                    let position = position
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .unwrap_or(len);
                    let position = if position < 0 { 0 } else { position.min(len) } as usize;
                    let candidate = Self::substring_chars(&value, 0, position.saturating_add(1));
                    let found = if search.is_empty() {
                        Some(position.min(Self::string_char_len(&candidate)))
                    } else {
                        candidate
                            .rfind(&search)
                            .map(|byte| Self::string_char_len(&candidate[..byte]))
                    };
                    Ok(Value::Number(found.map(|idx| idx as i64).unwrap_or(-1)))
                }
                Expr::StringSearch { value, pattern } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let pattern = self.eval_expr(pattern, env, event_param, event)?;
                    if let Some(result) = self.call_string_symbol_method(
                        &pattern,
                        SymbolStaticProperty::Search,
                        &value,
                        &[],
                        event,
                    )? {
                        return Ok(result);
                    }
                    let regex = if let Value::RegExp(regex) = pattern {
                        regex
                    } else {
                        let built = self.new_regex_from_values(&pattern, None)?;
                        let Value::RegExp(regex) = built else {
                            unreachable!("RegExp constructor must return a RegExp");
                        };
                        regex
                    };
                    let previous_last_index = regex.borrow().last_index;
                    regex.borrow_mut().last_index = 0;
                    let result = Self::regex_exec(&regex, &value)?;
                    regex.borrow_mut().last_index = previous_last_index;
                    let idx = result.map(|match_result| match_result.index as i64);
                    Ok(Value::Number(idx.unwrap_or(-1)))
                }
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
