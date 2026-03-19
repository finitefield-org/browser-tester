use super::*;

impl Harness {
    pub(crate) fn try_eval_string_search_member_call(
        &mut self,
        text: &str,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match member {
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
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
