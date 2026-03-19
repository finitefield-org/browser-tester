use super::*;

impl Harness {
    pub(crate) fn try_eval_string_core_exprs(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::StringCharAt { value, index } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let len = Self::string_char_len(&value);
                    let index = index
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .unwrap_or(0);
                    if index < 0 || (index as usize) >= len {
                        Ok(Value::String(String::new()))
                    } else {
                        Ok(Self::string_char_at(&value, index as usize)
                            .map(|ch| Value::String(ch.to_string()))
                            .unwrap_or_else(|| Value::String(String::new())))
                    }
                }
                Expr::StringCharCodeAt { value, index } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
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
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
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
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let len = Self::string_char_len(&value) as i64;
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
                        Ok(Self::string_char_at(&value, index as usize)
                            .map(|ch| Value::String(ch.to_string()))
                            .unwrap_or(Value::Undefined))
                    }
                }
                Expr::StringConcat { value, args } => {
                    let base = self.eval_expr(value, env, event_param, event)?;
                    let mut out = self.coerce_string_method_receiver(&base)?;
                    for arg in args {
                        let value = self.eval_expr(arg, env, event_param, event)?;
                        out.push_str(&self.coerce_to_string_for_tostring(&value)?);
                    }
                    Ok(Value::String(out))
                }
                Expr::StringTrim { value, mode } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let value = match mode {
                        StringTrimMode::Both => value.trim().to_string(),
                        StringTrimMode::Start => value.trim_start().to_string(),
                        StringTrimMode::End => value.trim_end().to_string(),
                    };
                    Ok(Value::String(value))
                }
                Expr::StringToUpperCase(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    Ok(Value::String(value.to_uppercase()))
                }
                Expr::StringToLowerCase(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    Ok(Value::String(value.to_lowercase()))
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
                            let text = self.coerce_string_method_receiver(&other)?;
                            let len = Self::string_char_len(&text);
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
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let len = Self::string_char_len(&value);
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
                Expr::StringRepeat { value, count } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
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
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let target_length = self.eval_expr(target_length, env, event_param, event)?;
                    let target_length = Self::value_to_i64(&target_length).max(0) as usize;
                    let current_len = Self::string_char_len(&value);
                    if target_length <= current_len {
                        return Ok(Value::String(value));
                    }
                    let pad = pad
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| self.coerce_to_string_for_tostring(&value))
                        .transpose()?
                        .unwrap_or_else(|| " ".to_string());
                    if pad.is_empty() {
                        return Ok(Value::String(value));
                    }
                    let mut filler = String::new();
                    let needed = target_length - current_len;
                    while Self::string_char_len(&filler) < needed {
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
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let target_length = self.eval_expr(target_length, env, event_param, event)?;
                    let target_length = Self::value_to_i64(&target_length).max(0) as usize;
                    let current_len = Self::string_char_len(&value);
                    if target_length <= current_len {
                        return Ok(Value::String(value));
                    }
                    let pad = pad
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| self.coerce_to_string_for_tostring(&value))
                        .transpose()?
                        .unwrap_or_else(|| " ".to_string());
                    if pad.is_empty() {
                        return Ok(Value::String(value));
                    }
                    let mut filler = String::new();
                    let needed = target_length - current_len;
                    while Self::string_char_len(&filler) < needed {
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
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    let compare = self.eval_expr(compare, env, event_param, event)?;
                    let compare = self.coerce_to_string_for_tostring(&compare)?;
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
                    let mut numeric = false;
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
                            if let Some(value) = Self::object_get_entry(&entries, "numeric") {
                                if !matches!(value, Value::Undefined) {
                                    numeric = value.truthy();
                                }
                            }
                        }
                    }
                    Ok(Value::Number(Self::intl_collator_compare_strings(
                        &value,
                        &compare,
                        &locale,
                        &case_first,
                        &sensitivity,
                        numeric,
                    )))
                }
                Expr::StringIsWellFormed(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    Ok(Value::Bool(string_is_well_formed_utf16(&value)))
                }
                Expr::StringToWellFormed(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let value = self.coerce_string_method_receiver(&value)?;
                    Ok(Value::String(string_to_well_formed_utf16(&value)))
                }
                Expr::StringValueOf(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    match value {
                        Value::Object(entries) => {
                            let (
                                string_value,
                                boolean_value,
                                number_value,
                                bigint_value,
                                symbol_id,
                            ) = {
                                let entries_ref = entries.borrow();
                                (
                                    Self::string_wrapper_value_from_object(&entries_ref),
                                    Self::boolean_wrapper_value_from_object(&entries_ref),
                                    Self::number_wrapper_value_from_object(&entries_ref),
                                    Self::bigint_wrapper_value_from_object(&entries_ref),
                                    Self::symbol_wrapper_id_from_object(&entries_ref),
                                )
                            };
                            if let Some(value) = string_value {
                                Ok(Value::String(value))
                            } else if let Some(value) = boolean_value {
                                Ok(Value::Bool(value))
                            } else if let Some(value) = number_value {
                                Ok(value)
                            } else if let Some(value) = bigint_value {
                                Ok(Value::BigInt(value))
                            } else if let Some(symbol_id) = symbol_id {
                                Ok(self
                                    .symbol_runtime
                                    .symbols_by_id
                                    .get(&symbol_id)
                                    .cloned()
                                    .map(Value::Symbol)
                                    .unwrap_or(Value::Object(entries.clone())))
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
                    if let Some(text) = self.callable_source_text(&value) {
                        return Ok(Value::String(text));
                    }
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
                    Ok(Value::String(
                        self.coerce_to_string_for_string_constructor(&value)?,
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
