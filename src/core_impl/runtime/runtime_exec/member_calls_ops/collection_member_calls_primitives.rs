use super::*;
use unicode_normalization::UnicodeNormalization;

impl Harness {
    pub(crate) fn eval_date_member_call(
        &mut self,
        value: &Rc<RefCell<i64>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let result = match member {
            "getTime" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getTime does not take arguments".into(),
                    ));
                }
                Value::Number(*value.borrow())
            }
            "setTime" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setTime requires exactly one argument".into(),
                    ));
                }
                let timestamp_ms = Self::value_to_i64(&evaluated_args[0]);
                *value.borrow_mut() = timestamp_ms;
                Value::Number(timestamp_ms)
            }
            "toISOString" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "toISOString does not take arguments".into(),
                    ));
                }
                Value::String(Self::format_iso_8601_utc(*value.borrow()))
            }
            "toLocaleDateString" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "toLocaleDateString supports at most two arguments".into(),
                    ));
                }
                let requested_locales = evaluated_args
                    .first()
                    .map(|value| self.intl_collect_locales(value))
                    .transpose()?
                    .unwrap_or_default();
                let locale = Self::intl_select_locale_for_formatter(
                    IntlFormatterKind::DateTimeFormat,
                    &requested_locales,
                );
                let options =
                    self.intl_date_time_options_from_value(&locale, evaluated_args.get(1))?;
                Value::String(self.intl_format_date_time(*value.borrow(), &locale, &options))
            }
            "toString" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "toString does not take arguments".into(),
                    ));
                }
                Value::String(Self::format_iso_8601_utc(*value.borrow()))
            }
            "valueOf" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "valueOf does not take arguments".into(),
                    ));
                }
                Value::Number(*value.borrow())
            }
            "getUTCFullYear" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCFullYear does not take arguments".into(),
                    ));
                }
                let (year, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(year)
            }
            "getUTCMonth" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCMonth does not take arguments".into(),
                    ));
                }
                let (_, month, ..) = Self::date_components_utc(*value.borrow());
                Value::Number((month as i64) - 1)
            }
            "getUTCDate" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCDate does not take arguments".into(),
                    ));
                }
                let (_, _, day, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(day as i64)
            }
            "getUTCDay" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCDay does not take arguments".into(),
                    ));
                }
                let timestamp_ms = *value.borrow();
                let days = timestamp_ms.div_euclid(86_400_000);
                let weekday = ((days + 4).rem_euclid(7)) as i64;
                Value::Number(weekday)
            }
            "getUTCHours" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCHours does not take arguments".into(),
                    ));
                }
                let (_, _, _, hour, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(hour as i64)
            }
            "getUTCMinutes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCMinutes does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, minute, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(minute as i64)
            }
            "getUTCSeconds" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCSeconds does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, _, second, _) = Self::date_components_utc(*value.borrow());
                Value::Number(second as i64)
            }
            "getUTCMilliseconds" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCMilliseconds does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, _, _, millisecond) = Self::date_components_utc(*value.borrow());
                Value::Number(millisecond as i64)
            }
            "getFullYear" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getFullYear does not take arguments".into(),
                    ));
                }
                let (year, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(year)
            }
            "getMonth" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getMonth does not take arguments".into(),
                    ));
                }
                let (_, month, ..) = Self::date_components_utc(*value.borrow());
                Value::Number((month as i64) - 1)
            }
            "getDate" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getDate does not take arguments".into(),
                    ));
                }
                let (_, _, day, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(day as i64)
            }
            "getHours" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getHours does not take arguments".into(),
                    ));
                }
                let (_, _, _, hour, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(hour as i64)
            }
            "getMinutes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getMinutes does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, minute, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(minute as i64)
            }
            "getSeconds" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getSeconds does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, _, second, _) = Self::date_components_utc(*value.borrow());
                Value::Number(second as i64)
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    pub(crate) fn eval_string_member_call(
        &mut self,
        text: &str,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match member {
            "charAt" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "charAt supports zero or one argument".into(),
                    ));
                }
                let len = Self::string_char_len(text);
                let index = evaluated_args.first().map(Self::value_to_i64).unwrap_or(0);
                if index < 0 || (index as usize) >= len {
                    Value::String(String::new())
                } else {
                    Self::string_char_at(text, index as usize)
                        .map(|ch| Value::String(ch.to_string()))
                        .unwrap_or_else(|| Value::String(String::new()))
                }
            }
            "charCodeAt" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "charCodeAt supports zero or one argument".into(),
                    ));
                }
                let chars = text.chars().collect::<Vec<_>>();
                let len = chars.len();
                let index = evaluated_args.first().map(Self::value_to_i64).unwrap_or(0);
                if index < 0 || (index as usize) >= len {
                    Value::Float(f64::NAN)
                } else {
                    let ch = chars[index as usize];
                    let code_unit = crate::js_regex::deinternalize_surrogate_marker(ch)
                        .map(|value| value as i64)
                        .unwrap_or(ch as i64);
                    Value::Number(code_unit)
                }
            }
            "codePointAt" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "codePointAt supports zero or one argument".into(),
                    ));
                }
                let chars = text.chars().collect::<Vec<_>>();
                let len = chars.len();
                let index = evaluated_args.first().map(Self::value_to_i64).unwrap_or(0);
                if index < 0 || (index as usize) >= len {
                    Value::Undefined
                } else {
                    let i = index as usize;
                    let ch = chars[i];
                    if let Some(first_unit) = crate::js_regex::deinternalize_surrogate_marker(ch) {
                        if (0xD800..=0xDBFF).contains(&first_unit) {
                            if let Some(next_ch) = chars.get(i + 1).copied() {
                                if let Some(second_unit) =
                                    crate::js_regex::deinternalize_surrogate_marker(next_ch)
                                {
                                    if (0xDC00..=0xDFFF).contains(&second_unit) {
                                        let cp = 0x10000
                                            + (((first_unit - 0xD800) as u32) << 10)
                                            + ((second_unit - 0xDC00) as u32);
                                        Value::Number(cp as i64)
                                    } else {
                                        Value::Number(first_unit as i64)
                                    }
                                } else {
                                    Value::Number(first_unit as i64)
                                }
                            } else {
                                Value::Number(first_unit as i64)
                            }
                        } else {
                            Value::Number(first_unit as i64)
                        }
                    } else {
                        Value::Number(ch as i64)
                    }
                }
            }
            "at" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "at supports zero or one argument".into(),
                    ));
                }
                let len = Self::string_char_len(text) as i64;
                let index = evaluated_args.first().map(Self::value_to_i64).unwrap_or(0);
                let index = if index < 0 { len + index } else { index };
                if index < 0 || index >= len {
                    Value::Undefined
                } else {
                    Self::string_char_at(text, index as usize)
                        .map(|ch| Value::String(ch.to_string()))
                        .unwrap_or(Value::Undefined)
                }
            }
            "concat" => {
                let mut out = text.to_string();
                for arg in evaluated_args {
                    out.push_str(&self.coerce_to_string_for_tostring(arg)?);
                }
                Value::String(out)
            }
            "trim" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("trim does not take arguments".into()));
                }
                Value::String(text.trim().to_string())
            }
            "trimStart" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "trimStart does not take arguments".into(),
                    ));
                }
                Value::String(text.trim_start().to_string())
            }
            "trimEnd" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "trimEnd does not take arguments".into(),
                    ));
                }
                Value::String(text.trim_end().to_string())
            }
            "toUpperCase" | "toLocaleUpperCase" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} does not take arguments"
                    )));
                }
                Value::String(text.to_uppercase())
            }
            "toLowerCase" | "toLocaleLowerCase" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} does not take arguments"
                    )));
                }
                Value::String(text.to_lowercase())
            }
            "endsWith" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "endsWith requires one or two arguments".into(),
                    ));
                }
                let Some(search) = evaluated_args.first() else {
                    return Err(Error::ScriptRuntime(
                        "endsWith requires one or two arguments".into(),
                    ));
                };
                if self.is_regexp_like_for_string_prefix_search(search)? {
                    return Err(Error::ScriptRuntime(
                        "First argument to String.prototype.endsWith must not be a regular expression"
                            .into(),
                    ));
                }
                let search = self.coerce_to_string_for_tostring(search)?;
                let len = Self::string_char_len(text);
                let end = evaluated_args
                    .get(1)
                    .map(Self::value_to_i64)
                    .map(|value| {
                        if value < 0 {
                            0
                        } else {
                            (value as usize).min(len)
                        }
                    })
                    .unwrap_or(len);
                Value::Bool(Self::substring_chars(text, 0, end).ends_with(&search))
            }
            "includes" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "includes requires one or two arguments".into(),
                    ));
                }
                if self.is_regexp_like_for_string_prefix_search(&evaluated_args[0])? {
                    return Err(Error::ScriptRuntime(
                        "First argument to String.prototype.includes must not be a regular expression"
                            .into(),
                    ));
                }
                let search = self.coerce_to_string_for_tostring(&evaluated_args[0])?;
                let len = Self::string_char_len(text) as i64;
                let mut position = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                if position < 0 {
                    position = 0;
                }
                let position = position.min(len) as usize;
                let position_byte = Self::char_index_to_byte(text, position);
                Value::Bool(text[position_byte..].contains(&search))
            }
            "indexOf" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "indexOf requires one or two arguments".into(),
                    ));
                }
                let search = self.coerce_to_string_for_tostring(&evaluated_args[0])?;
                let len = Self::string_char_len(text) as i64;
                let mut position = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                if position < 0 {
                    position = 0;
                }
                let position = position.min(len) as usize;
                Value::Number(
                    Self::string_index_of(text, &search, position)
                        .map(|value| value as i64)
                        .unwrap_or(-1),
                )
            }
            "lastIndexOf" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "lastIndexOf requires one or two arguments".into(),
                    ));
                }
                let search = self.coerce_to_string_for_tostring(&evaluated_args[0])?;
                let len = text.chars().count() as i64;
                let position = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(len);
                let position = if position < 0 { 0 } else { position.min(len) } as usize;
                let candidate = Self::substring_chars(text, 0, position.saturating_add(1));
                let found = if search.is_empty() {
                    Some(position.min(Self::string_char_len(&candidate)))
                } else {
                    candidate
                        .rfind(&search)
                        .map(|byte| Self::string_char_len(&candidate[..byte]))
                };
                Value::Number(found.map(|idx| idx as i64).unwrap_or(-1))
            }
            "match" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "match requires exactly one argument".into(),
                    ));
                }
                if let Some(result) = self.call_string_symbol_method(
                    &evaluated_args[0],
                    SymbolStaticProperty::Match,
                    text,
                    &[],
                    event,
                )? {
                    return Ok(Some(result));
                }
                self.eval_string_match(text, evaluated_args[0].clone())?
            }
            "matchAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "matchAll requires exactly one argument".into(),
                    ));
                }
                let pattern = evaluated_args[0].clone();
                if let Value::RegExp(regex) = &pattern {
                    if !regex.borrow().global {
                        return Err(Error::ScriptRuntime(
                            "String.prototype.matchAll called with a non-global RegExp argument"
                                .into(),
                        ));
                    }
                }
                if let Some(result) = self.call_string_symbol_method(
                    &pattern,
                    SymbolStaticProperty::MatchAll,
                    text,
                    &[],
                    event,
                )? {
                    return Ok(Some(result));
                }
                let regex = match &pattern {
                    Value::RegExp(regex) => {
                        let regex_ref = regex.borrow();
                        let source = Value::String(regex_ref.source.clone());
                        let flags = Value::String(regex_ref.flags.clone());
                        let last_index = regex_ref.last_index;
                        drop(regex_ref);
                        let cloned = self.new_regex_from_values(&source, Some(&flags))?;
                        let Value::RegExp(regex) = cloned else {
                            unreachable!("RegExp constructor must return a RegExp");
                        };
                        regex.borrow_mut().last_index = last_index;
                        regex
                    }
                    other => {
                        let flags = Value::String("g".to_string());
                        let compiled = self.new_regex_from_values(other, Some(&flags))?;
                        let Value::RegExp(regex) = compiled else {
                            unreachable!("RegExp constructor must return a RegExp");
                        };
                        regex
                    }
                };
                let mut matches = Vec::new();
                loop {
                    let Some(result) = Self::regex_exec(&regex, text)? else {
                        break;
                    };
                    matches.push(Self::regex_exec_result_to_value(result.clone()));
                    if result.full_match_start_byte == result.full_match_end_byte {
                        let mut regex = regex.borrow_mut();
                        let unicode = regex.unicode || regex.unicode_sets;
                        regex.last_index =
                            Self::advance_string_index_utf16(text, regex.last_index, unicode);
                    }
                }
                self.new_iterator_value(matches)
            }
            "slice" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "String.slice supports up to two arguments".into(),
                    ));
                }
                let len = Self::string_char_len(text);
                let start = evaluated_args
                    .first()
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(0);
                let end = evaluated_args
                    .get(1)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(len)
                    .max(start);
                Value::String(Self::substring_chars(text, start, end))
            }
            "normalize" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "normalize supports at most one form argument".into(),
                    ));
                }
                let coerced_form;
                let form = match evaluated_args.first() {
                    None | Some(Value::Undefined) => "NFC",
                    Some(Value::String(value)) => value.as_str(),
                    Some(other) => {
                        coerced_form = self.coerce_to_string_for_tostring(other)?;
                        coerced_form.as_str()
                    }
                };
                let normalized = match form {
                    "NFC" => text.nfc().collect(),
                    "NFD" => text.nfd().collect(),
                    "NFKC" => text.nfkc().collect(),
                    "NFKD" => text.nfkd().collect(),
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "invalid normalization form: {form}"
                        )));
                    }
                };
                Value::String(normalized)
            }
            "split" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "split supports up to two arguments".into(),
                    ));
                }
                let separator = evaluated_args.first().cloned();
                let limit_value = evaluated_args.get(1).cloned().unwrap_or(Value::Undefined);
                if let Some(separator_value) = &separator {
                    if !matches!(separator_value, Value::Undefined) {
                        if let Some(result) = self.call_string_symbol_method(
                            separator_value,
                            SymbolStaticProperty::Split,
                            text,
                            std::slice::from_ref(&limit_value),
                            event,
                        )? {
                            return Ok(Some(result));
                        }
                    }
                }
                let limit = match evaluated_args.get(1) {
                    None | Some(Value::Undefined) => None,
                    Some(value) => Some(Self::value_to_i64(value)),
                };
                let parts = match separator {
                    None => Self::split_string(text, None, limit),
                    Some(Value::RegExp(regex)) => {
                        Self::split_string_with_regex(text, &regex, limit)?
                    }
                    Some(Value::Undefined) => Self::split_string(text, None, limit),
                    Some(value) => Self::split_string(
                        text,
                        Some(self.coerce_to_string_for_tostring(&value)?),
                        limit,
                    ),
                };
                Self::new_array_value(parts)
            }
            "replace" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "replace requires exactly two arguments".into(),
                    ));
                }
                let from = evaluated_args[0].clone();
                let to = evaluated_args[1].clone();
                if let Some(result) = self.call_string_symbol_method(
                    &from,
                    SymbolStaticProperty::Replace,
                    text,
                    std::slice::from_ref(&to),
                    event,
                )? {
                    return Ok(Some(result));
                }
                let replaced = if self.is_callable_value(&to) {
                    match from {
                        Value::RegExp(regex) => {
                            self.replace_string_with_regex_callback(text, &regex, &to, event)?
                        }
                        other => {
                            let from = self.coerce_to_string_for_tostring(&other)?;
                            self.replace_string_with_string_callback(
                                text, &from, &to, false, event,
                            )?
                        }
                    }
                } else {
                    let replacement = self.coerce_to_string_for_tostring(&to)?;
                    match from {
                        Value::RegExp(regex) => {
                            Self::replace_string_with_regex(text, &regex, &replacement)?
                        }
                        other => text.replacen(
                            &self.coerce_to_string_for_tostring(&other)?,
                            &replacement,
                            1,
                        ),
                    }
                };
                Value::String(replaced)
            }
            "replaceAll" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "replaceAll requires exactly two arguments".into(),
                    ));
                }
                let from = evaluated_args[0].clone();
                let to = evaluated_args[1].clone();
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
                    text,
                    std::slice::from_ref(&to),
                    event,
                )? {
                    return Ok(Some(result));
                }
                let replaced = if self.is_callable_value(&to) {
                    match from {
                        Value::RegExp(regex) => {
                            self.replace_string_with_regex_callback(text, &regex, &to, event)?
                        }
                        other => {
                            let from = self.coerce_to_string_for_tostring(&other)?;
                            self.replace_string_with_string_callback(text, &from, &to, true, event)?
                        }
                    }
                } else {
                    let replacement = self.coerce_to_string_for_tostring(&to)?;
                    match from {
                        Value::RegExp(regex) => {
                            Self::replace_string_with_regex(text, &regex, &replacement)?
                        }
                        other => {
                            let from = self.coerce_to_string_for_tostring(&other)?;
                            if from.is_empty() {
                                let mut out = String::new();
                                for ch in text.chars() {
                                    out.push_str(&replacement);
                                    out.push(ch);
                                }
                                out.push_str(&replacement);
                                out
                            } else {
                                text.replace(&from, &replacement)
                            }
                        }
                    }
                };
                Value::String(replaced)
            }
            "repeat" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "repeat requires exactly one argument".into(),
                    ));
                }
                let count = Self::value_to_i64(&evaluated_args[0]);
                if count < 0 {
                    return Err(Error::ScriptRuntime(
                        "Invalid count value for String.prototype.repeat".into(),
                    ));
                }
                let count = usize::try_from(count).map_err(|_| {
                    Error::ScriptRuntime("Invalid count value for String.prototype.repeat".into())
                })?;
                Value::String(text.repeat(count))
            }
            "padStart" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "padStart requires one or two arguments".into(),
                    ));
                }
                let target_length = Self::value_to_i64(&evaluated_args[0]).max(0) as usize;
                let current_len = Self::string_char_len(text);
                if target_length <= current_len {
                    return Ok(Some(Value::String(text.to_string())));
                }
                let pad = evaluated_args
                    .get(1)
                    .map(|value| self.coerce_to_string_for_tostring(value))
                    .transpose()?
                    .unwrap_or_else(|| " ".to_string());
                if pad.is_empty() {
                    return Ok(Some(Value::String(text.to_string())));
                }
                let mut filler = String::new();
                let needed = target_length - current_len;
                while Self::string_char_len(&filler) < needed {
                    filler.push_str(&pad);
                }
                let filler = filler.chars().take(needed).collect::<String>();
                Value::String(format!("{filler}{text}"))
            }
            "padEnd" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "padEnd requires one or two arguments".into(),
                    ));
                }
                let target_length = Self::value_to_i64(&evaluated_args[0]).max(0) as usize;
                let current_len = Self::string_char_len(text);
                if target_length <= current_len {
                    return Ok(Some(Value::String(text.to_string())));
                }
                let pad = evaluated_args
                    .get(1)
                    .map(|value| self.coerce_to_string_for_tostring(value))
                    .transpose()?
                    .unwrap_or_else(|| " ".to_string());
                if pad.is_empty() {
                    return Ok(Some(Value::String(text.to_string())));
                }
                let mut filler = String::new();
                let needed = target_length - current_len;
                while Self::string_char_len(&filler) < needed {
                    filler.push_str(&pad);
                }
                let filler = filler.chars().take(needed).collect::<String>();
                Value::String(format!("{text}{filler}"))
            }
            "search" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "search requires exactly one argument".into(),
                    ));
                }
                let pattern = evaluated_args[0].clone();
                if let Some(result) = self.call_string_symbol_method(
                    &pattern,
                    SymbolStaticProperty::Search,
                    text,
                    &[],
                    event,
                )? {
                    return Ok(Some(result));
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
                let result = Self::regex_exec(&regex, text)?;
                regex.borrow_mut().last_index = previous_last_index;
                let idx = result.map(|match_result| match_result.index as i64);
                Value::Number(idx.unwrap_or(-1))
            }
            "startsWith" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "startsWith requires one or two arguments".into(),
                    ));
                }
                if self.is_regexp_like_for_string_prefix_search(&evaluated_args[0])? {
                    return Err(Error::ScriptRuntime(
                        "First argument to String.prototype.startsWith must not be a regular expression"
                            .into(),
                    ));
                }
                let search = self.coerce_to_string_for_tostring(&evaluated_args[0])?;
                let len = Self::string_char_len(text) as i64;
                let mut position = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                if position < 0 {
                    position = 0;
                }
                let position = position.min(len) as usize;
                let position_byte = Self::char_index_to_byte(text, position);
                Value::Bool(text[position_byte..].starts_with(&search))
            }
            "substring" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "substring supports up to two arguments".into(),
                    ));
                }
                let len = Self::string_char_len(text);
                let start = evaluated_args
                    .first()
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_substring_index(len, value))
                    .unwrap_or(0);
                let end = evaluated_args
                    .get(1)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_substring_index(len, value))
                    .unwrap_or(len);
                let (start, end) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                Value::String(Self::substring_chars(text, start, end))
            }
            "localeCompare" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "localeCompare requires one to three arguments".into(),
                    ));
                }
                let compare = self.coerce_to_string_for_tostring(&evaluated_args[0])?;
                let locale = evaluated_args
                    .get(1)
                    .map(|locales| self.intl_collect_locales(locales))
                    .transpose()?
                    .and_then(|locales| locales.into_iter().next())
                    .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
                let mut case_first = "false".to_string();
                let mut sensitivity = "variant".to_string();
                let mut numeric = false;
                if let Some(Value::Object(entries)) = evaluated_args.get(2) {
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
                    if let Some(value) = Self::object_get_entry(&entries, "numeric") {
                        if !matches!(value, Value::Undefined) {
                            numeric = value.truthy();
                        }
                    }
                }
                Value::Number(Self::intl_collator_compare_strings(
                    text,
                    &compare,
                    &locale,
                    &case_first,
                    &sensitivity,
                    numeric,
                ))
            }
            "isWellFormed" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "isWellFormed does not take arguments".into(),
                    ));
                }
                let chars = text.chars().collect::<Vec<_>>();
                let mut i = 0usize;
                let mut well_formed = true;
                while i < chars.len() {
                    let Some(unit) = crate::js_regex::deinternalize_surrogate_marker(chars[i])
                    else {
                        i += 1;
                        continue;
                    };

                    if (0xD800..=0xDBFF).contains(&unit) {
                        let Some(next) = chars.get(i + 1).copied() else {
                            well_formed = false;
                            break;
                        };
                        let Some(next_unit) = crate::js_regex::deinternalize_surrogate_marker(next)
                        else {
                            well_formed = false;
                            break;
                        };
                        if !(0xDC00..=0xDFFF).contains(&next_unit) {
                            well_formed = false;
                            break;
                        }
                        i += 2;
                        continue;
                    }

                    if (0xDC00..=0xDFFF).contains(&unit) {
                        well_formed = false;
                        break;
                    }

                    i += 1;
                }
                Value::Bool(well_formed)
            }
            "toWellFormed" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "toWellFormed does not take arguments".into(),
                    ));
                }
                let chars = text.chars().collect::<Vec<_>>();
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
                            if let Some(next_unit) =
                                crate::js_regex::deinternalize_surrogate_marker(next)
                            {
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
                Value::String(out)
            }
            "iterator" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "String[Symbol.iterator] does not take arguments".into(),
                    ));
                }
                self.new_iterator_value(
                    text.chars()
                        .map(|ch| Value::String(ch.to_string()))
                        .collect::<Vec<_>>(),
                )
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
