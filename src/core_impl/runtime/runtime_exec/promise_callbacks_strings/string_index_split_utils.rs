use super::*;

impl Harness {
    pub(crate) fn normalize_slice_index(len: usize, index: i64) -> usize {
        if index < 0 {
            len.saturating_sub(index.unsigned_abs() as usize)
        } else {
            (index as usize).min(len)
        }
    }

    pub(crate) fn normalize_splice_start_index(len: usize, start: i64) -> usize {
        if start < 0 {
            len.saturating_sub(start.unsigned_abs() as usize)
        } else {
            (start as usize).min(len)
        }
    }

    pub(crate) fn normalize_substring_index(len: usize, index: i64) -> usize {
        if index < 0 {
            0
        } else {
            (index as usize).min(len)
        }
    }

    pub(crate) fn char_index_to_byte(value: &str, char_index: usize) -> usize {
        value
            .char_indices()
            .nth(char_index)
            .map(|(idx, _)| idx)
            .unwrap_or(value.len())
    }

    pub(crate) fn substring_chars(value: &str, start: usize, end: usize) -> String {
        if start >= end {
            return String::new();
        }
        value.chars().skip(start).take(end - start).collect()
    }

    pub(crate) fn split_string(
        value: &str,
        separator: Option<String>,
        limit: Option<i64>,
    ) -> Vec<Value> {
        let mut parts = match separator {
            None => vec![Value::String(value.to_string())],
            Some(separator) => {
                if separator.is_empty() {
                    value
                        .chars()
                        .map(|ch| Value::String(ch.to_string()))
                        .collect::<Vec<_>>()
                } else {
                    value
                        .split(&separator)
                        .map(|part| Value::String(part.to_string()))
                        .collect::<Vec<_>>()
                }
            }
        };

        if let Some(limit) = limit {
            if limit == 0 {
                parts.clear();
            } else if limit > 0 {
                parts.truncate(limit as usize);
            }
        }

        parts
    }

    pub(crate) fn split_string_with_regex(
        value: &str,
        regex: &Rc<RefCell<RegexValue>>,
        limit: Option<i64>,
    ) -> Result<Vec<Value>> {
        let compiled = regex.borrow().compiled.clone();
        let mut parts = Vec::new();
        for part in compiled
            .split_all(value)
            .map_err(Self::map_regex_runtime_error)?
        {
            parts.push(Value::String(part));
        }
        if let Some(limit) = limit {
            if limit == 0 {
                parts.clear();
            } else if limit > 0 {
                parts.truncate(limit as usize);
            }
        }
        Ok(parts)
    }

    pub(crate) fn expand_regex_replacement(template: &str, captures: &Captures) -> String {
        let chars = template.chars().collect::<Vec<_>>();
        let mut i = 0usize;
        let mut out = String::new();
        while i < chars.len() {
            if chars[i] != '$' {
                out.push(chars[i]);
                i += 1;
                continue;
            }
            if i + 1 >= chars.len() {
                out.push('$');
                i += 1;
                continue;
            }
            let next = chars[i + 1];
            match next {
                '$' => {
                    out.push('$');
                    i += 2;
                }
                '&' => {
                    if let Some(full) = captures.get(0) {
                        out.push_str(full.as_str());
                    }
                    i += 2;
                }
                '0'..='9' => {
                    let mut idx = (next as u8 - b'0') as usize;
                    let mut consumed = 2usize;
                    if i + 2 < chars.len() && chars[i + 2].is_ascii_digit() {
                        let candidate = idx * 10 + (chars[i + 2] as u8 - b'0') as usize;
                        if captures.get(candidate).is_some() {
                            idx = candidate;
                            consumed = 3;
                        }
                    }
                    if idx > 0 {
                        if let Some(group) = captures.get(idx) {
                            out.push_str(group.as_str());
                        }
                    } else {
                        out.push('$');
                        out.push('0');
                    }
                    i += consumed;
                }
                _ => {
                    out.push('$');
                    out.push(next);
                    i += 2;
                }
            }
        }
        out
    }

    pub(crate) fn replace_string_with_regex(
        value: &str,
        regex: &Rc<RefCell<RegexValue>>,
        replacement: &str,
    ) -> Result<String> {
        let (compiled, global) = {
            let regex = regex.borrow();
            (regex.compiled.clone(), regex.global)
        };

        if global {
            let mut out = String::new();
            let mut last_end = 0usize;
            for captures in compiled
                .captures_all(value)
                .map_err(Self::map_regex_runtime_error)?
            {
                let Some(full) = captures.get(0) else {
                    continue;
                };
                out.push_str(&value[last_end..full.start()]);
                out.push_str(&Self::expand_regex_replacement(replacement, &captures));
                last_end = full.end();
            }
            out.push_str(&value[last_end..]);
            Ok(out)
        } else if let Some(captures) = compiled
            .captures(value)
            .map_err(Self::map_regex_runtime_error)?
        {
            if let Some(full) = captures.get(0) {
                let mut out = String::new();
                out.push_str(&value[..full.start()]);
                out.push_str(&Self::expand_regex_replacement(replacement, &captures));
                out.push_str(&value[full.end()..]);
                Ok(out)
            } else {
                Ok(value.to_string())
            }
        } else {
            Ok(value.to_string())
        }
    }

    pub(crate) fn replace_string_with_regex_callback(
        &mut self,
        value: &str,
        regex: &Rc<RefCell<RegexValue>>,
        callback: &Value,
        event: &EventState,
    ) -> Result<String> {
        let (compiled, global) = {
            let regex = regex.borrow();
            (regex.compiled.clone(), regex.global)
        };

        if global {
            let mut out = String::new();
            let mut last_end = 0usize;
            for captures in compiled
                .captures_all(value)
                .map_err(Self::map_regex_runtime_error)?
            {
                let Some(full) = captures.get(0) else {
                    continue;
                };
                out.push_str(&value[last_end..full.start()]);
                out.push_str(&self.regex_callback_replacement(
                    callback,
                    &captures,
                    full.start(),
                    value,
                    event,
                )?);
                last_end = full.end();
            }
            out.push_str(&value[last_end..]);
            Ok(out)
        } else if let Some(captures) = compiled
            .captures(value)
            .map_err(Self::map_regex_runtime_error)?
        {
            if let Some(full) = captures.get(0) {
                let mut out = String::new();
                out.push_str(&value[..full.start()]);
                out.push_str(&self.regex_callback_replacement(
                    callback,
                    &captures,
                    full.start(),
                    value,
                    event,
                )?);
                out.push_str(&value[full.end()..]);
                Ok(out)
            } else {
                Ok(value.to_string())
            }
        } else {
            Ok(value.to_string())
        }
    }

    pub(crate) fn replace_string_with_string_callback(
        &mut self,
        value: &str,
        from: &str,
        callback: &Value,
        replace_all: bool,
        event: &EventState,
    ) -> Result<String> {
        if from.is_empty() {
            if !replace_all {
                let replacement = self.execute_callback_value(
                    callback,
                    &[
                        Value::String(String::new()),
                        Value::Number(0),
                        Value::String(value.to_string()),
                    ],
                    event,
                )?;
                return Ok(format!("{}{}", replacement.as_string(), value));
            }

            let mut out = String::new();
            let chars = value.chars().collect::<Vec<_>>();
            for (idx, ch) in chars.iter().enumerate() {
                let replacement = self.execute_callback_value(
                    callback,
                    &[
                        Value::String(String::new()),
                        Value::Number(idx as i64),
                        Value::String(value.to_string()),
                    ],
                    event,
                )?;
                out.push_str(&replacement.as_string());
                out.push(*ch);
            }
            let replacement = self.execute_callback_value(
                callback,
                &[
                    Value::String(String::new()),
                    Value::Number(chars.len() as i64),
                    Value::String(value.to_string()),
                ],
                event,
            )?;
            out.push_str(&replacement.as_string());
            return Ok(out);
        }

        if !replace_all {
            if let Some(start) = value.find(from) {
                let end = start + from.len();
                let mut out = String::new();
                out.push_str(&value[..start]);
                let replacement = self.execute_callback_value(
                    callback,
                    &[
                        Value::String(from.to_string()),
                        Value::Number(value[..start].chars().count() as i64),
                        Value::String(value.to_string()),
                    ],
                    event,
                )?;
                out.push_str(&replacement.as_string());
                out.push_str(&value[end..]);
                Ok(out)
            } else {
                Ok(value.to_string())
            }
        } else {
            let mut out = String::new();
            let mut cursor = 0usize;
            while let Some(rel) = value[cursor..].find(from) {
                let start = cursor + rel;
                let end = start + from.len();
                out.push_str(&value[cursor..start]);
                let replacement = self.execute_callback_value(
                    callback,
                    &[
                        Value::String(from.to_string()),
                        Value::Number(value[..start].chars().count() as i64),
                        Value::String(value.to_string()),
                    ],
                    event,
                )?;
                out.push_str(&replacement.as_string());
                cursor = end;
            }
            out.push_str(&value[cursor..]);
            Ok(out)
        }
    }

    fn regex_callback_replacement(
        &mut self,
        callback: &Value,
        captures: &Captures,
        byte_start: usize,
        input: &str,
        event: &EventState,
    ) -> Result<String> {
        let mut args = Vec::new();
        if let Some(full) = captures.get(0) {
            args.push(Value::String(full.as_str().to_string()));
        } else {
            args.push(Value::String(String::new()));
        }
        for idx in 1..captures.len() {
            if let Some(group) = captures.get(idx) {
                args.push(Value::String(group.as_str().to_string()));
            } else {
                args.push(Value::Undefined);
            }
        }
        args.push(Value::Number(input[..byte_start].chars().count() as i64));
        args.push(Value::String(input.to_string()));
        let replacement = self.execute_callback_value(callback, &args, event)?;
        Ok(replacement.as_string())
    }

    pub(crate) fn string_index_of(
        value: &str,
        search: &str,
        start_char_idx: usize,
    ) -> Option<usize> {
        let start_byte = Self::char_index_to_byte(value, start_char_idx);
        let pos = value.get(start_byte..)?.find(search)?;
        Some(value[..start_byte + pos].chars().count())
    }
}
