use super::*;
use unicode_normalization::UnicodeNormalization;

impl Harness {
    pub(crate) fn try_eval_string_transform_member_call(
        &mut self,
        text: &str,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match member {
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
