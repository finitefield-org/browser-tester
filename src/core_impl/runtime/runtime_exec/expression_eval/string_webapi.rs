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
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    let pattern = self.eval_expr(pattern, env, event_param, event)?;
                    self.eval_string_match(&value, pattern)
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
                Expr::StructuredClone(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    Self::structured_clone_value(&value, &mut Vec::new(), &mut Vec::new())
                }
                Expr::Fetch(request) => {
                    let request = self
                        .eval_expr(request, env, event_param, event)?
                        .as_string();
                    self.platform_mocks.fetch_calls.push(request.clone());
                    let response = self
                        .platform_mocks
                        .fetch_mocks
                        .get(&request)
                        .cloned()
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "fetch mock not found for request: {request}"
                            ))
                        })?;
                    Ok(Value::String(response))
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
                        ("matches".into(), Value::Bool(matches)),
                        ("media".into(), Value::String(query)),
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
