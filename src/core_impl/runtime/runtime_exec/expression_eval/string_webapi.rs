use super::*;

impl Harness {
    pub(crate) fn eval_expr_string_and_webapi(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::StringCharAt { value, index } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let len = value.chars().count();
                    let index = index
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .unwrap_or(0);
                    if index < 0 || (index as usize) >= len {
                        Ok(Value::String(String::new()))
                    } else {
                        Ok(value
                            .chars()
                            .nth(index as usize)
                            .map(|ch| Value::String(ch.to_string()))
                            .unwrap_or_else(|| Value::String(String::new())))
                    }
                }
                Expr::StringCharCodeAt { value, index } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let chars = value.chars().collect::<Vec<_>>();
                    let len = chars.len();
                    let index = index
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .unwrap_or(0);
                    if index < 0 || (index as usize) >= len {
                        Ok(Value::Float(f64::NAN))
                    } else {
                        let ch = chars[index as usize];
                        let code_unit = crate::js_regex::deinternalize_surrogate_marker(ch)
                            .map(|value| value as i64)
                            .unwrap_or(ch as i64);
                        Ok(Value::Number(code_unit))
                    }
                }
                Expr::StringCodePointAt { value, index } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let chars = value.chars().collect::<Vec<_>>();
                    let len = chars.len();
                    let index = index
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .unwrap_or(0);
                    if index < 0 || (index as usize) >= len {
                        Ok(Value::Undefined)
                    } else {
                        let i = index as usize;
                        let ch = chars[i];
                        if let Some(first_unit) =
                            crate::js_regex::deinternalize_surrogate_marker(ch)
                        {
                            if (0xD800..=0xDBFF).contains(&first_unit) {
                                if let Some(next_ch) = chars.get(i + 1).copied() {
                                    if let Some(second_unit) =
                                        crate::js_regex::deinternalize_surrogate_marker(next_ch)
                                    {
                                        if (0xDC00..=0xDFFF).contains(&second_unit) {
                                            let cp = 0x10000
                                                + (((first_unit - 0xD800) as u32) << 10)
                                                + ((second_unit - 0xDC00) as u32);
                                            return Ok(Value::Number(cp as i64));
                                        }
                                    }
                                }
                            }
                            Ok(Value::Number(first_unit as i64))
                        } else {
                            Ok(Value::Number(ch as i64))
                        }
                    }
                }
                Expr::StringAt { value, index } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let len = value.chars().count() as i64;
                    let index = index
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .unwrap_or(0);
                    let index = if index < 0 { len + index } else { index };
                    if index < 0 || index >= len {
                        Ok(Value::Undefined)
                    } else {
                        Ok(value
                            .chars()
                            .nth(index as usize)
                            .map(|ch| Value::String(ch.to_string()))
                            .unwrap_or(Value::Undefined))
                    }
                }
                Expr::StringConcat { value, args } => {
                    let mut out = self.eval_expr(value, env, event_param, event)?.as_string();
                    for arg in args {
                        let value = self.eval_expr(arg, env, event_param, event)?;
                        out.push_str(&value.as_string());
                    }
                    Ok(Value::String(out))
                }
                Expr::StringTrim { value, mode } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let value = match mode {
                        StringTrimMode::Both => value.trim().to_string(),
                        StringTrimMode::Start => value.trim_start().to_string(),
                        StringTrimMode::End => value.trim_end().to_string(),
                    };
                    Ok(Value::String(value))
                }
                Expr::StringToUpperCase(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    Ok(Value::String(value.to_uppercase()))
                }
                Expr::StringToLowerCase(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    Ok(Value::String(value.to_lowercase()))
                }
                Expr::StringIncludes {
                    value,
                    search,
                    position,
                } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let search = self.eval_expr(search, env, event_param, event)?;
                    if matches!(search, Value::RegExp(_)) {
                        return Err(Error::ScriptRuntime(
                        "First argument to String.prototype.includes must not be a regular expression"
                            .into(),
                    ));
                    }
                    let search = search.as_string();
                    let len = value.chars().count() as i64;
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
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let search = self.eval_expr(search, env, event_param, event)?;
                    if matches!(search, Value::RegExp(_)) {
                        return Err(Error::ScriptRuntime(
                        "First argument to String.prototype.startsWith must not be a regular expression"
                            .into(),
                    ));
                    }
                    let search = search.as_string();
                    let len = value.chars().count() as i64;
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
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let search = self.eval_expr(search, env, event_param, event)?;
                    if matches!(search, Value::RegExp(_)) {
                        return Err(Error::ScriptRuntime(
                        "First argument to String.prototype.endsWith must not be a regular expression"
                            .into(),
                    ));
                    }
                    let search = search.as_string();
                    let len = value.chars().count();
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
                Expr::StringSlice { value, start, end } => {
                    let source = self.eval_expr(value, env, event_param, event)?;
                    match source {
                        Value::Array(values) => {
                            let values_ref = values.borrow();
                            let len = values_ref.len();
                            let start = start
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(0);
                            let end = end
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(len);
                            let end = end.max(start);
                            Ok(Self::new_array_value(values_ref[start..end].to_vec()))
                        }
                        Value::TypedArray(values) => {
                            let snapshot = self.typed_array_snapshot(&values)?;
                            let kind = values.borrow().kind;
                            let len = snapshot.len();
                            let start = start
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(0);
                            let end = end
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(len);
                            let end = end.max(start);
                            self.new_typed_array_from_values(kind, &snapshot[start..end])
                        }
                        Value::ArrayBuffer(buffer) => {
                            Self::ensure_array_buffer_not_detached(&buffer, "slice")?;
                            let source = buffer.borrow();
                            let len = source.bytes.len();
                            let start = start
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(0);
                            let end = end
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(len);
                            let end = end.max(start);
                            Ok(Value::ArrayBuffer(Rc::new(RefCell::new(
                                ArrayBufferValue {
                                    bytes: source.bytes[start..end].to_vec(),
                                    max_byte_length: None,
                                    detached: false,
                                },
                            ))))
                        }
                        Value::Blob(blob) => {
                            let source = blob.borrow();
                            let len = source.bytes.len();
                            let start = start
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(0);
                            let end = end
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(len);
                            let end = end.max(start);
                            Ok(Self::new_blob_value(
                                source.bytes[start..end].to_vec(),
                                String::new(),
                            ))
                        }
                        other => {
                            let text = other.as_string();
                            let len = text.chars().count();
                            let start = start
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(0);
                            let end = end
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .map(|value| Self::normalize_slice_index(len, value))
                                .unwrap_or(len);
                            let end = end.max(start);
                            Ok(Value::String(Self::substring_chars(&text, start, end)))
                        }
                    }
                }
                Expr::StringSubstring { value, start, end } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let len = value.chars().count();
                    let start = start
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_substring_index(len, value))
                        .unwrap_or(0);
                    let end = end
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_substring_index(len, value))
                        .unwrap_or(len);
                    let (start, end) = if start <= end {
                        (start, end)
                    } else {
                        (end, start)
                    };
                    Ok(Value::String(Self::substring_chars(&value, start, end)))
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
                    self.eval_string_match(&target_value.as_string(), pattern)
                }
                Expr::StringSplit {
                    value,
                    separator,
                    limit,
                } => {
                    let text = self.eval_expr(value, env, event_param, event)?.as_string();
                    let separator = separator
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?;
                    let limit = limit
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value));
                    let parts = match separator {
                        None => Self::split_string(&text, None, limit),
                        Some(Value::RegExp(regex)) => {
                            Self::split_string_with_regex(&text, &regex, limit)?
                        }
                        Some(value) => Self::split_string(&text, Some(value.as_string()), limit),
                    };
                    Ok(Self::new_array_value(parts))
                }
                Expr::StringReplace { value, from, to } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let to = self.eval_expr(to, env, event_param, event)?;
                    let from = self.eval_expr(from, env, event_param, event)?;
                    let replaced = if self.is_callable_value(&to) {
                        match from {
                            Value::RegExp(regex) => {
                                self.replace_string_with_regex_callback(&value, &regex, &to, event)?
                            }
                            other => {
                                let from = other.as_string();
                                self.replace_string_with_string_callback(
                                    &value, &from, &to, false, event,
                                )?
                            }
                        }
                    } else {
                        let replacement = to.as_string();
                        match from {
                            Value::RegExp(regex) => {
                                Self::replace_string_with_regex(&value, &regex, &replacement)?
                            }
                            other => value.replacen(&other.as_string(), &replacement, 1),
                        }
                    };
                    Ok(Value::String(replaced))
                }
                Expr::StringReplaceAll { value, from, to } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let to = self.eval_expr(to, env, event_param, event)?;
                    let from = self.eval_expr(from, env, event_param, event)?;
                    let replaced = if self.is_callable_value(&to) {
                        match from {
                            Value::RegExp(regex) => {
                                if !regex.borrow().global {
                                    return Err(Error::ScriptRuntime(
                                    "String.prototype.replaceAll called with a non-global RegExp argument"
                                        .into(),
                                ));
                                }
                                self.replace_string_with_regex_callback(&value, &regex, &to, event)?
                            }
                            other => {
                                let from = other.as_string();
                                self.replace_string_with_string_callback(
                                    &value, &from, &to, true, event,
                                )?
                            }
                        }
                    } else {
                        let replacement = to.as_string();
                        match from {
                            Value::RegExp(regex) => {
                                if !regex.borrow().global {
                                    return Err(Error::ScriptRuntime(
                                    "String.prototype.replaceAll called with a non-global RegExp argument"
                                        .into(),
                                ));
                                }
                                Self::replace_string_with_regex(&value, &regex, &replacement)?
                            }
                            other => {
                                let from = other.as_string();
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
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let search = self.eval_expr(search, env, event_param, event)?.as_string();
                    let len = value.chars().count() as i64;
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
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let search = self.eval_expr(search, env, event_param, event)?.as_string();
                    let len = value.chars().count() as i64;
                    let position = position
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .unwrap_or(len);
                    let position = if position < 0 { 0 } else { position.min(len) } as usize;
                    let candidate = Self::substring_chars(&value, 0, position.saturating_add(1));
                    let found = if search.is_empty() {
                        Some(position.min(candidate.chars().count()))
                    } else {
                        candidate
                            .rfind(&search)
                            .map(|byte| candidate[..byte].chars().count())
                    };
                    Ok(Value::Number(found.map(|idx| idx as i64).unwrap_or(-1)))
                }
                Expr::StringSearch { value, pattern } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let pattern = self.eval_expr(pattern, env, event_param, event)?;
                    let regex = if let Value::RegExp(regex) = pattern {
                        regex
                    } else {
                        let built = Self::new_regex_from_values(&pattern, None)?;
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
                Expr::StringRepeat { value, count } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let count = self.eval_expr(count, env, event_param, event)?;
                    let count = Self::value_to_i64(&count);
                    if count < 0 {
                        return Err(Error::ScriptRuntime(
                            "Invalid count value for String.prototype.repeat".into(),
                        ));
                    }
                    let count = usize::try_from(count).map_err(|_| {
                        Error::ScriptRuntime(
                            "Invalid count value for String.prototype.repeat".into(),
                        )
                    })?;
                    Ok(Value::String(value.repeat(count)))
                }
                Expr::StringPadStart {
                    value,
                    target_length,
                    pad,
                } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let target_length = self.eval_expr(target_length, env, event_param, event)?;
                    let target_length = Self::value_to_i64(&target_length).max(0) as usize;
                    let current_len = value.chars().count();
                    if target_length <= current_len {
                        return Ok(Value::String(value));
                    }
                    let pad = pad
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| value.as_string())
                        .unwrap_or_else(|| " ".to_string());
                    if pad.is_empty() {
                        return Ok(Value::String(value));
                    }
                    let mut filler = String::new();
                    let needed = target_length - current_len;
                    while filler.chars().count() < needed {
                        filler.push_str(&pad);
                    }
                    let filler = filler.chars().take(needed).collect::<String>();
                    Ok(Value::String(format!("{filler}{value}")))
                }
                Expr::StringPadEnd {
                    value,
                    target_length,
                    pad,
                } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let target_length = self.eval_expr(target_length, env, event_param, event)?;
                    let target_length = Self::value_to_i64(&target_length).max(0) as usize;
                    let current_len = value.chars().count();
                    if target_length <= current_len {
                        return Ok(Value::String(value));
                    }
                    let pad = pad
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| value.as_string())
                        .unwrap_or_else(|| " ".to_string());
                    if pad.is_empty() {
                        return Ok(Value::String(value));
                    }
                    let mut filler = String::new();
                    let needed = target_length - current_len;
                    while filler.chars().count() < needed {
                        filler.push_str(&pad);
                    }
                    let filler = filler.chars().take(needed).collect::<String>();
                    Ok(Value::String(format!("{value}{filler}")))
                }
                Expr::StringLocaleCompare {
                    value,
                    compare,
                    locales,
                    options,
                } => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let compare = self
                        .eval_expr(compare, env, event_param, event)?
                        .as_string();
                    let locale = locales
                        .as_ref()
                        .map(|locales| self.eval_expr(locales, env, event_param, event))
                        .transpose()?
                        .map(|locales| self.intl_collect_locales(&locales))
                        .transpose()?
                        .and_then(|locales| locales.into_iter().next())
                        .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
                    let mut case_first = "false".to_string();
                    let mut sensitivity = "variant".to_string();
                    if let Some(options) = options {
                        let options = self.eval_expr(options, env, event_param, event)?;
                        if let Value::Object(entries) = options {
                            let entries = entries.borrow();
                            if let Some(Value::String(value)) =
                                Self::object_get_entry(&entries, "caseFirst")
                            {
                                case_first = value;
                            }
                            if let Some(Value::String(value)) =
                                Self::object_get_entry(&entries, "sensitivity")
                            {
                                sensitivity = value;
                            }
                        }
                    }
                    Ok(Value::Number(Self::intl_collator_compare_strings(
                        &value,
                        &compare,
                        &locale,
                        &case_first,
                        &sensitivity,
                    )))
                }
                Expr::StringIsWellFormed(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    Ok(Value::Bool(string_is_well_formed_utf16(&value)))
                }
                Expr::StringToWellFormed(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    Ok(Value::String(string_to_well_formed_utf16(&value)))
                }
                Expr::StringValueOf(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    match value {
                        Value::Object(entries) => {
                            let entries_ref = entries.borrow();
                            if let Some(value) =
                                Self::string_wrapper_value_from_object(&entries_ref)
                            {
                                Ok(Value::String(value))
                            } else {
                                Ok(Value::Object(entries.clone()))
                            }
                        }
                        Value::String(value) => Ok(Value::String(value)),
                        other => Ok(other),
                    }
                }
                Expr::StringToString(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    if let Value::Node(node) = &value {
                        if let Some(tag_name) = self.dom.tag_name(*node) {
                            if tag_name.eq_ignore_ascii_case("a")
                                || tag_name.eq_ignore_ascii_case("area")
                            {
                                return Ok(Value::String(self.resolve_anchor_href(*node)));
                            }
                        }
                        return Ok(Value::String(Value::Node(*node).as_string()));
                    }
                    if let Value::Object(entries) = &value {
                        if Self::is_url_search_params_object(&entries.borrow()) {
                            let pairs = Self::url_search_params_pairs_from_object_entries(
                                &entries.borrow(),
                            );
                            return Ok(Value::String(serialize_url_search_params_pairs(&pairs)));
                        }
                    }
                    Ok(Value::String(value.as_string()))
                }
                Expr::StructuredClone { value, options } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let options = options
                        .as_ref()
                        .map(|options| self.eval_expr(options, env, event_param, event))
                        .transpose()?;
                    Self::structured_clone_value_with_options(&value, options.as_ref())
                }
                Expr::Fetch { request, options } => {
                    self.eval_fetch_call(request, options.as_deref(), env, event_param, event)
                }
                Expr::MatchMedia(query) => {
                    let query = self.eval_expr(query, env, event_param, event)?.as_string();
                    self.platform_mocks.match_media_calls.push(query.clone());
                    let matches = self
                        .platform_mocks
                        .match_media_mocks
                        .get(&query)
                        .copied()
                        .unwrap_or(self.platform_mocks.default_match_media_matches);
                    Ok(Self::new_object_value(vec![
                        (INTERNAL_MATCH_MEDIA_OBJECT_KEY.into(), Value::Bool(true)),
                        (
                            INTERNAL_MATCH_MEDIA_QUERY_KEY.into(),
                            Value::String(query.clone()),
                        ),
                        (INTERNAL_EVENT_TARGET_OBJECT_KEY.into(), Value::Bool(true)),
                        ("matches".into(), Value::Bool(matches)),
                        ("media".into(), Value::String(query)),
                        ("onchange".into(), Value::Null),
                        (
                            "addEventListener".into(),
                            Self::new_builtin_placeholder_function(),
                        ),
                        (
                            "removeEventListener".into(),
                            Self::new_builtin_placeholder_function(),
                        ),
                        (
                            "dispatchEvent".into(),
                            Self::new_builtin_placeholder_function(),
                        ),
                        (
                            "addListener".into(),
                            Self::new_builtin_placeholder_function(),
                        ),
                        (
                            "removeListener".into(),
                            Self::new_builtin_placeholder_function(),
                        ),
                    ]))
                }
                Expr::MatchMediaProp { query, prop } => {
                    let query = self.eval_expr(query, env, event_param, event)?.as_string();
                    self.platform_mocks.match_media_calls.push(query.clone());
                    let matches = self
                        .platform_mocks
                        .match_media_mocks
                        .get(&query)
                        .copied()
                        .unwrap_or(self.platform_mocks.default_match_media_matches);
                    match prop {
                        MatchMediaProp::Matches => Ok(Value::Bool(matches)),
                        MatchMediaProp::Media => Ok(Value::String(query)),
                    }
                }
                Expr::Alert(message) => {
                    let message = self
                        .eval_expr(message, env, event_param, event)?
                        .as_string();
                    self.platform_mocks.alert_messages.push(message);
                    Ok(Value::Undefined)
                }
                Expr::Confirm(message) => {
                    let _ = self.eval_expr(message, env, event_param, event)?;
                    let accepted = self
                        .platform_mocks
                        .confirm_responses
                        .pop_front()
                        .unwrap_or(self.platform_mocks.default_confirm_response);
                    Ok(Value::Bool(accepted))
                }
                Expr::Prompt { message, default } => {
                    let _ = self.eval_expr(message, env, event_param, event)?;
                    let default_value = default
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| value.as_string());
                    let response = self
                        .platform_mocks
                        .prompt_responses
                        .pop_front()
                        .unwrap_or_else(|| {
                            self.platform_mocks
                                .default_prompt_response
                                .clone()
                                .or(default_value)
                        });
                    match response {
                        Some(value) => Ok(Value::String(value)),
                        None => Ok(Value::Null),
                    }
                }
                Expr::FunctionConstructor { args } => {
                    if args.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "new Function requires at least one argument".into(),
                        ));
                    }

                    let mut parts = Vec::with_capacity(args.len());
                    for arg in args {
                        let part = self.eval_expr(arg, env, event_param, event)?.as_string();
                        parts.push(part);
                    }

                    let body_src = parts.last().cloned().ok_or_else(|| {
                        Error::ScriptRuntime("new Function requires body argument".into())
                    })?;
                    let mut params = Vec::new();
                    for part in parts.iter().take(parts.len().saturating_sub(1)) {
                        let names = Self::parse_function_constructor_param_names(part)?;
                        params.extend(names.into_iter().map(|name| FunctionParam {
                            name,
                            default: None,
                            is_rest: false,
                        }));
                    }

                    let stmts = parse_block_statements(&body_src).map_err(|err| {
                        Error::ScriptRuntime(format!("new Function body parse failed: {err}"))
                    })?;
                    Ok(self.make_function_value(
                        ScriptHandler { params, stmts },
                        env,
                        true,
                        false,
                        false,
                        false,
                        false,
                    ))
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

impl Harness {
    pub(crate) fn is_fetch_response_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_FETCH_RESPONSE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_fetch_request_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_FETCH_REQUEST_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_headers_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_HEADERS_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    fn fetch_type_error_value(reason: &str) -> Value {
        Value::String(format!("TypeError: {reason}"))
    }

    fn fetch_rejected_promise(&mut self, reason: &str) -> Result<Value> {
        let promise = self.new_pending_promise();
        self.promise_reject(&promise, Self::fetch_type_error_value(reason));
        Ok(Value::Promise(promise))
    }

    fn resolve_fetch_url(&self, input: &str) -> Result<String> {
        let base = self.document_base_url();
        let resolved = Self::resolve_url_string(input, Some(&base))
            .ok_or_else(|| Error::ScriptRuntime("TypeError: Invalid URL".into()))?;
        let parts = LocationParts::parse(&resolved)
            .ok_or_else(|| Error::ScriptRuntime("TypeError: Invalid URL".into()))?;
        if !parts.username.is_empty() || !parts.password.is_empty() {
            return Err(Error::ScriptRuntime(
                "TypeError: URL with credentials is not allowed".into(),
            ));
        }
        Ok(resolved)
    }

    pub(crate) fn fetch_request_input_and_url_from_value(
        &self,
        value: &Value,
    ) -> Result<(String, String)> {
        match value {
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_fetch_request_object(&entries) {
                    let input =
                        match Self::object_get_entry(&entries, INTERNAL_FETCH_REQUEST_INPUT_KEY) {
                            Some(Value::String(input)) => input,
                            _ => String::new(),
                        };
                    let url = match Self::object_get_entry(&entries, INTERNAL_FETCH_REQUEST_URL_KEY)
                    {
                        Some(Value::String(url)) => url,
                        _ => self.resolve_fetch_url(&input)?,
                    };
                    return Ok((input, url));
                }
                if Self::is_url_object(&entries) {
                    let href = Self::object_get_entry(&entries, "href")
                        .map(|value| value.as_string())
                        .unwrap_or_default();
                    let url = self.resolve_fetch_url(&href)?;
                    return Ok((href, url));
                }
                let input = value.as_string();
                let url = self.resolve_fetch_url(&input)?;
                Ok((input, url))
            }
            _ => {
                let input = value.as_string();
                let url = self.resolve_fetch_url(&input)?;
                Ok((input, url))
            }
        }
    }

    fn normalize_header_name(name: &str) -> String {
        name.trim().to_ascii_lowercase()
    }

    fn headers_pairs_from_value(&self, value: &Value) -> Result<Vec<(String, String)>> {
        match value {
            Value::Undefined | Value::Null => Ok(Vec::new()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_headers_object(&entries) {
                    let Some(Value::Object(header_entries)) =
                        Self::object_get_entry(&entries, INTERNAL_HEADERS_ENTRIES_KEY)
                    else {
                        return Ok(Vec::new());
                    };
                    let header_entries = header_entries.borrow();
                    let mut pairs = header_entries
                        .iter()
                        .filter(|(name, _)| !Self::is_internal_object_key(name))
                        .map(|(name, value)| (name.clone(), value.as_string()))
                        .collect::<Vec<_>>();
                    pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                    return Ok(pairs);
                }
                let mut pairs = entries
                    .iter()
                    .filter(|(name, _)| !Self::is_internal_object_key(name))
                    .map(|(name, value)| (Self::normalize_header_name(name), value.as_string()))
                    .filter(|(name, _)| !name.is_empty())
                    .collect::<Vec<_>>();
                pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                Ok(pairs)
            }
            _ => Err(Error::ScriptRuntime(
                "TypeError: RequestInit.headers must be an object or Headers".into(),
            )),
        }
    }

    fn fetch_options_from_value(&self, value: &Value) -> Result<(String, Vec<(String, String)>)> {
        match value {
            Value::Undefined | Value::Null => Ok(("GET".to_string(), Vec::new())),
            Value::Object(entries) => {
                let entries = entries.borrow();
                let method = Self::object_get_entry(&entries, "method")
                    .filter(|value| !matches!(value, Value::Undefined))
                    .map(|value| value.as_string().to_ascii_uppercase())
                    .filter(|method| !method.trim().is_empty())
                    .unwrap_or_else(|| "GET".to_string());
                let headers = Self::object_get_entry(&entries, "headers")
                    .map(|headers| self.headers_pairs_from_value(&headers))
                    .transpose()?
                    .unwrap_or_default();
                Ok((method, headers))
            }
            _ => Err(Error::ScriptRuntime(
                "TypeError: RequestInit must be an object".into(),
            )),
        }
    }

    pub(crate) fn new_headers_value_from_pairs(&self, pairs: &[(String, String)]) -> Value {
        let mut entries = ObjectValue::default();
        for (name, value) in pairs {
            if name.is_empty() {
                continue;
            }
            Self::object_set_entry(
                &mut entries,
                Self::normalize_header_name(name),
                Value::String(value.clone()),
            );
        }
        Self::new_object_value(vec![
            (INTERNAL_HEADERS_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_HEADERS_ENTRIES_KEY.to_string(),
                Value::Object(Rc::new(RefCell::new(entries))),
            ),
            ("set".to_string(), Self::new_builtin_placeholder_function()),
            ("get".to_string(), Self::new_builtin_placeholder_function()),
            ("has".to_string(), Self::new_builtin_placeholder_function()),
            (
                "append".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "delete".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ])
    }

    pub(crate) fn new_headers_value_from_call_args(&self, args: &[Value]) -> Result<Value> {
        if args.len() > 1 {
            return Err(Error::ScriptRuntime(
                "Headers constructor supports at most one argument".into(),
            ));
        }
        let pairs = args
            .first()
            .map(|value| self.headers_pairs_from_value(value))
            .transpose()?
            .unwrap_or_default();
        Ok(self.new_headers_value_from_pairs(&pairs))
    }

    pub(crate) fn new_fetch_request_value(
        &self,
        input: &str,
        url: &str,
        method: &str,
        headers: &[(String, String)],
    ) -> Value {
        let headers_value = self.new_headers_value_from_pairs(headers);
        Self::new_object_value(vec![
            (
                INTERNAL_FETCH_REQUEST_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_FETCH_REQUEST_INPUT_KEY.to_string(),
                Value::String(input.to_string()),
            ),
            (
                INTERNAL_FETCH_REQUEST_URL_KEY.to_string(),
                Value::String(url.to_string()),
            ),
            (
                INTERNAL_FETCH_REQUEST_METHOD_KEY.to_string(),
                Value::String(method.to_string()),
            ),
            ("url".to_string(), Value::String(url.to_string())),
            ("method".to_string(), Value::String(method.to_string())),
            ("headers".to_string(), headers_value),
            (
                "clone".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ])
    }

    pub(crate) fn new_fetch_request_value_from_call_args(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Err(Error::ScriptRuntime(
                "Request constructor requires at least one argument".into(),
            ));
        }
        if args.len() > 2 {
            return Err(Error::ScriptRuntime(
                "Request constructor supports one or two arguments".into(),
            ));
        }
        let (input, url) = self.fetch_request_input_and_url_from_value(&args[0])?;
        let (method, headers) = args
            .get(1)
            .map(|options| self.fetch_options_from_value(options))
            .transpose()?
            .unwrap_or_else(|| ("GET".to_string(), Vec::new()));
        Ok(self.new_fetch_request_value(&input, &url, &method, &headers))
    }

    pub(crate) fn new_fetch_response_value(
        &self,
        url: &str,
        status: i64,
        status_text: &str,
        body: &str,
    ) -> Value {
        let headers = self.new_headers_value_from_pairs(&[]);
        Self::new_object_value(vec![
            (
                INTERNAL_FETCH_RESPONSE_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_FETCH_RESPONSE_BODY_KEY.to_string(),
                Value::String(body.to_string()),
            ),
            (
                INTERNAL_FETCH_RESPONSE_STATUS_KEY.to_string(),
                Value::Number(status),
            ),
            (
                INTERNAL_FETCH_RESPONSE_STATUS_TEXT_KEY.to_string(),
                Value::String(status_text.to_string()),
            ),
            (
                INTERNAL_FETCH_RESPONSE_URL_KEY.to_string(),
                Value::String(url.to_string()),
            ),
            ("ok".to_string(), Value::Bool((200..=299).contains(&status))),
            ("status".to_string(), Value::Number(status)),
            (
                "statusText".to_string(),
                Value::String(status_text.to_string()),
            ),
            ("url".to_string(), Value::String(url.to_string())),
            ("headers".to_string(), headers),
            ("text".to_string(), Self::new_builtin_placeholder_function()),
            ("json".to_string(), Self::new_builtin_placeholder_function()),
            ("blob".to_string(), Self::new_builtin_placeholder_function()),
            (
                "arrayBuffer".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "clone".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ])
    }

    pub(crate) fn fetch_response_property_from_entries(
        &self,
        entries: &[(String, Value)],
        key: &str,
    ) -> Option<Value> {
        if !Self::is_fetch_response_object(entries) {
            return None;
        }
        match key {
            "ok" | "status" | "statusText" | "url" | "headers" | "text" | "json" | "blob"
            | "arrayBuffer" | "clone" => Self::object_get_entry(entries, key),
            _ => None,
        }
    }

    pub(crate) fn fetch_request_property_from_entries(
        &self,
        entries: &[(String, Value)],
        key: &str,
    ) -> Option<Value> {
        if !Self::is_fetch_request_object(entries) {
            return None;
        }
        match key {
            "url" | "method" | "headers" | "clone" => Self::object_get_entry(entries, key),
            _ => None,
        }
    }

    pub(crate) fn headers_property_from_entries(
        &self,
        entries: &[(String, Value)],
        key: &str,
    ) -> Option<Value> {
        if !Self::is_headers_object(entries) {
            return None;
        }
        match key {
            "set" | "get" | "has" | "append" | "delete" => Self::object_get_entry(entries, key),
            _ => None,
        }
    }

    pub(crate) fn eval_fetch_call(
        &mut self,
        request: &Expr,
        options: Option<&Expr>,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let request_value = self.eval_expr(request, env, event_param, event)?;
        let options_value = options
            .map(|expr| self.eval_expr(expr, env, event_param, event))
            .transpose()?;
        self.eval_fetch_call_impl(&request_value, options_value.as_ref())
    }

    fn eval_fetch_call_impl(
        &mut self,
        request_value: &Value,
        options_value: Option<&Value>,
    ) -> Result<Value> {
        let (input_key, request_url) =
            match self.fetch_request_input_and_url_from_value(request_value) {
                Ok(ok) => ok,
                Err(_) => return self.fetch_rejected_promise("Invalid URL"),
            };

        if let Some(options) = options_value {
            if self.fetch_options_from_value(options).is_err() {
                return self.fetch_rejected_promise("RequestInit is invalid");
            }
        }

        self.platform_mocks.fetch_calls.push(input_key.clone());
        let mock = self
            .platform_mocks
            .fetch_mocks
            .get(&input_key)
            .cloned()
            .or_else(|| self.platform_mocks.fetch_mocks.get(&request_url).cloned());
        let Some(mock) = mock else {
            return self.fetch_rejected_promise("Failed to fetch");
        };

        let response =
            self.new_fetch_response_value(&request_url, mock.status, &mock.status_text, &mock.body);
        let promise = self.new_pending_promise();
        self.promise_resolve(&promise, response)?;
        Ok(Value::Promise(promise))
    }

    pub(crate) fn eval_fetch_call_from_values(&mut self, args: &[Value]) -> Result<Value> {
        if args.is_empty() || args.len() > 2 {
            return Err(Error::ScriptRuntime(
                "fetch requires one or two arguments".into(),
            ));
        }
        self.eval_fetch_call_impl(&args[0], args.get(1))
    }

    pub(crate) fn eval_fetch_response_member_call(
        &mut self,
        response_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let (is_response, body, status, status_text, url) = {
            let entries = response_object.borrow();
            (
                Self::is_fetch_response_object(&entries),
                Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_BODY_KEY),
                Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_STATUS_KEY),
                Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_STATUS_TEXT_KEY),
                Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_URL_KEY),
            )
        };
        if !is_response {
            return Ok(None);
        }
        let body = body.map(|value| value.as_string()).unwrap_or_default();
        let status = status
            .map(|value| Self::value_to_i64(&value))
            .unwrap_or(200);
        let status_text = status_text
            .map(|value| value.as_string())
            .unwrap_or_default();
        let url = url.map(|value| value.as_string()).unwrap_or_default();

        match member {
            "text" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Response.text does not take arguments".into(),
                    ));
                }
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, Value::String(body))?;
                Ok(Some(Value::Promise(promise)))
            }
            "json" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Response.json does not take arguments".into(),
                    ));
                }
                let promise = self.new_pending_promise();
                match Self::parse_json_text(&body) {
                    Ok(value) => {
                        self.promise_resolve(&promise, value)?;
                    }
                    Err(_) => {
                        self.promise_reject(
                            &promise,
                            Value::String(
                                "SyntaxError: Response body is not valid JSON".to_string(),
                            ),
                        );
                    }
                }
                Ok(Some(Value::Promise(promise)))
            }
            "blob" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Response.blob does not take arguments".into(),
                    ));
                }
                let promise = self.new_pending_promise();
                self.promise_resolve(
                    &promise,
                    Self::new_blob_value(body.into_bytes(), String::new()),
                )?;
                Ok(Some(Value::Promise(promise)))
            }
            "arrayBuffer" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Response.arrayBuffer does not take arguments".into(),
                    ));
                }
                let promise = self.new_pending_promise();
                self.promise_resolve(
                    &promise,
                    Value::ArrayBuffer(Rc::new(RefCell::new(ArrayBufferValue {
                        bytes: body.into_bytes(),
                        max_byte_length: None,
                        detached: false,
                    }))),
                )?;
                Ok(Some(Value::Promise(promise)))
            }
            "clone" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Response.clone does not take arguments".into(),
                    ));
                }
                Ok(Some(self.new_fetch_response_value(
                    &url,
                    status,
                    &status_text,
                    &body,
                )))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_fetch_request_member_call(
        &mut self,
        request_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let is_request = {
            let entries = request_object.borrow();
            Self::is_fetch_request_object(&entries)
        };
        if !is_request {
            return Ok(None);
        }
        match member {
            "clone" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Request.clone does not take arguments".into(),
                    ));
                }
                let cloned = Value::Object(Rc::new(RefCell::new(request_object.borrow().clone())));
                Ok(Some(cloned))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_headers_member_call(
        &mut self,
        headers_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let header_entries = {
            let entries = headers_object.borrow();
            if !Self::is_headers_object(&entries) {
                return Ok(None);
            }
            match Self::object_get_entry(&entries, INTERNAL_HEADERS_ENTRIES_KEY) {
                Some(Value::Object(header_entries)) => header_entries,
                _ => {
                    return Err(Error::ScriptRuntime(
                        "headers object has invalid internal state".into(),
                    ));
                }
            }
        };

        match member {
            "set" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Headers.set requires exactly two arguments".into(),
                    ));
                }
                let name = Self::normalize_header_name(&args[0].as_string());
                let value = args[1].as_string();
                if !name.is_empty() {
                    Self::object_set_entry(
                        &mut header_entries.borrow_mut(),
                        name,
                        Value::String(value),
                    );
                }
                Ok(Some(Value::Undefined))
            }
            "append" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Headers.append requires exactly two arguments".into(),
                    ));
                }
                let name = Self::normalize_header_name(&args[0].as_string());
                let value = args[1].as_string();
                if !name.is_empty() {
                    let mut entries = header_entries.borrow_mut();
                    let existing = Self::object_get_entry(&entries, &name)
                        .map(|value| value.as_string())
                        .unwrap_or_default();
                    let next = if existing.is_empty() {
                        value
                    } else {
                        format!("{existing}, {value}")
                    };
                    Self::object_set_entry(&mut entries, name, Value::String(next));
                }
                Ok(Some(Value::Undefined))
            }
            "get" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Headers.get requires exactly one argument".into(),
                    ));
                }
                let name = Self::normalize_header_name(&args[0].as_string());
                let value = if name.is_empty() {
                    Value::Null
                } else {
                    Self::object_get_entry(&header_entries.borrow(), &name).unwrap_or(Value::Null)
                };
                Ok(Some(value))
            }
            "has" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Headers.has requires exactly one argument".into(),
                    ));
                }
                let name = Self::normalize_header_name(&args[0].as_string());
                let has = !name.is_empty()
                    && Self::object_get_entry(&header_entries.borrow(), &name).is_some();
                Ok(Some(Value::Bool(has)))
            }
            "delete" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Headers.delete requires exactly one argument".into(),
                    ));
                }
                let name = Self::normalize_header_name(&args[0].as_string());
                if !name.is_empty() {
                    let _ = header_entries.borrow_mut().delete_entry(&name);
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }
}

fn string_is_well_formed_utf16(value: &str) -> bool {
    let chars = value.chars().collect::<Vec<_>>();
    let mut i = 0usize;
    while i < chars.len() {
        let Some(unit) = crate::js_regex::deinternalize_surrogate_marker(chars[i]) else {
            i += 1;
            continue;
        };

        if (0xD800..=0xDBFF).contains(&unit) {
            let Some(next) = chars.get(i + 1).copied() else {
                return false;
            };
            let Some(next_unit) = crate::js_regex::deinternalize_surrogate_marker(next) else {
                return false;
            };
            if !(0xDC00..=0xDFFF).contains(&next_unit) {
                return false;
            }
            i += 2;
            continue;
        }

        if (0xDC00..=0xDFFF).contains(&unit) {
            return false;
        }

        i += 1;
    }
    true
}

fn string_to_well_formed_utf16(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        let Some(unit) = crate::js_regex::deinternalize_surrogate_marker(ch) else {
            out.push(ch);
            i += 1;
            continue;
        };

        if (0xD800..=0xDBFF).contains(&unit) {
            if let Some(next) = chars.get(i + 1).copied() {
                if let Some(next_unit) = crate::js_regex::deinternalize_surrogate_marker(next) {
                    if (0xDC00..=0xDFFF).contains(&next_unit) {
                        out.push(ch);
                        out.push(next);
                        i += 2;
                        continue;
                    }
                }
            }
            out.push('\u{FFFD}');
            i += 1;
            continue;
        }

        if (0xDC00..=0xDFFF).contains(&unit) {
            out.push('\u{FFFD}');
            i += 1;
            continue;
        }

        out.push(ch);
        i += 1;
    }
    out
}
