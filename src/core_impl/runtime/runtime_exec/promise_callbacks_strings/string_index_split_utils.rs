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

    pub(crate) fn utf16_length(value: &str) -> usize {
        value.chars().map(char::len_utf16).sum()
    }

    pub(crate) fn utf16_index_to_byte(value: &str, utf16_index: usize) -> Option<usize> {
        if utf16_index == 0 {
            return Some(0);
        }
        let mut units = 0usize;
        for (byte, ch) in value.char_indices() {
            if units == utf16_index {
                return Some(byte);
            }
            units += ch.len_utf16();
            if units == utf16_index {
                return Some(byte + ch.len_utf8());
            }
            if units > utf16_index {
                return None;
            }
        }
        if units == utf16_index {
            Some(value.len())
        } else {
            None
        }
    }

    pub(crate) fn utf16_index_to_byte_ceil(value: &str, utf16_index: usize) -> Option<usize> {
        if utf16_index == 0 {
            return Some(0);
        }
        let mut units = 0usize;
        for (byte, ch) in value.char_indices() {
            if units == utf16_index {
                return Some(byte);
            }
            let next_units = units + ch.len_utf16();
            if utf16_index <= next_units {
                return Some(byte + ch.len_utf8());
            }
            units = next_units;
        }
        if units == utf16_index {
            Some(value.len())
        } else {
            None
        }
    }

    pub(crate) fn byte_index_to_utf16_index(value: &str, byte_index: usize) -> usize {
        let prefix = if byte_index <= value.len() {
            &value[..byte_index]
        } else {
            value
        };
        Self::utf16_length(prefix)
    }

    pub(crate) fn advance_string_index_utf16(value: &str, index: usize, unicode: bool) -> usize {
        if !unicode {
            return index + 1;
        }
        let units = value.encode_utf16().collect::<Vec<_>>();
        let len = units.len();
        if index + 1 >= len {
            return index + 1;
        }
        let first = units[index];
        let second = units[index + 1];
        if (0xD800..=0xDBFF).contains(&first) && (0xDC00..=0xDFFF).contains(&second) {
            index + 2
        } else {
            index + 1
        }
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
        let max_len = limit
            .filter(|limit| *limit >= 0)
            .map(|limit| limit as usize)
            .unwrap_or(usize::MAX);

        if max_len == 0 {
            return Ok(Vec::new());
        }

        let splitter = {
            let original = regex.borrow();
            let mut cloned = original.clone();
            cloned.global = false;
            cloned.sticky = true;
            cloned.last_index = 0;
            Rc::new(RefCell::new(cloned))
        };
        let unicode = {
            let splitter = splitter.borrow();
            splitter.unicode || splitter.unicode_sets
        };
        let input_len_utf16 = Self::utf16_length(value);

        if input_len_utf16 == 0 {
            let matched = Self::regex_exec(&splitter, value)?;
            if matched.is_some() {
                return Ok(Vec::new());
            }
            return Ok(vec![Value::String(String::new())]);
        }

        let mut parts = Vec::new();
        let mut last_last_index = 0usize;
        let mut q = 0usize;
        while q < input_len_utf16 {
            splitter.borrow_mut().last_index = q;
            let Some(result) = Self::regex_exec(&splitter, value)? else {
                q = Self::advance_string_index_utf16(value, q, unicode);
                continue;
            };

            let end_index = splitter.borrow().last_index;
            if end_index == last_last_index {
                q = Self::advance_string_index_utf16(value, q, unicode);
                continue;
            }

            let chunk_start =
                Self::utf16_index_to_byte_ceil(value, last_last_index).unwrap_or(value.len());
            let chunk_end =
                Self::utf16_index_to_byte_ceil(value, result.index).unwrap_or(value.len());

            parts.push(Value::String(value[chunk_start..chunk_end].to_string()));
            if parts.len() >= max_len {
                return Ok(parts);
            }

            for capture in result.captures.iter().skip(1) {
                if parts.len() >= max_len {
                    return Ok(parts);
                }
                parts.push(
                    capture
                        .clone()
                        .map(Value::String)
                        .unwrap_or(Value::Undefined),
                );
            }

            last_last_index = end_index;
            q = end_index;
        }

        if parts.len() < max_len {
            let tail_start =
                Self::utf16_index_to_byte_ceil(value, last_last_index).unwrap_or(value.len());
            parts.push(Value::String(value[tail_start..].to_string()));
        }

        Ok(parts)
    }

    pub(crate) fn replace_string_with_regex(
        value: &str,
        regex: &Rc<RefCell<RegexValue>>,
        replacement: &str,
    ) -> Result<String> {
        let global = regex.borrow().global;
        if global {
            regex.borrow_mut().last_index = 0;
        }

        let mut out = String::new();
        let mut last_end = 0usize;
        let mut matched_any = false;

        loop {
            let Some(result) = Self::regex_exec(regex, value)? else {
                break;
            };
            matched_any = true;
            out.push_str(&value[last_end..result.full_match_start_byte]);
            out.push_str(&Self::expand_regex_replacement_from_exec(
                replacement,
                &result.captures,
                result.groups.as_deref(),
                result.full_match_start_byte,
                result.full_match_end_byte,
                value,
            ));
            last_end = result.full_match_end_byte;

            if !global {
                break;
            }

            if result.full_match_start_byte == result.full_match_end_byte {
                let mut regex = regex.borrow_mut();
                let unicode = regex.unicode || regex.unicode_sets;
                regex.last_index =
                    Self::advance_string_index_utf16(value, regex.last_index, unicode);
            }
        }

        if !matched_any {
            return Ok(value.to_string());
        }

        out.push_str(&value[last_end..]);
        Ok(out)
    }

    pub(crate) fn replace_string_with_regex_callback(
        &mut self,
        value: &str,
        regex: &Rc<RefCell<RegexValue>>,
        callback: &Value,
        event: &EventState,
    ) -> Result<String> {
        let global = regex.borrow().global;
        if global {
            regex.borrow_mut().last_index = 0;
        }

        let mut out = String::new();
        let mut last_end = 0usize;
        let mut matched_any = false;

        loop {
            let Some(result) = Self::regex_exec(regex, value)? else {
                break;
            };
            matched_any = true;
            out.push_str(&value[last_end..result.full_match_start_byte]);
            out.push_str(&self.regex_callback_replacement_from_exec(
                callback,
                &result.captures,
                result.index,
                result.groups.as_deref(),
                value,
                event,
            )?);
            last_end = result.full_match_end_byte;

            if !global {
                break;
            }

            if result.full_match_start_byte == result.full_match_end_byte {
                let mut regex = regex.borrow_mut();
                let unicode = regex.unicode || regex.unicode_sets;
                regex.last_index =
                    Self::advance_string_index_utf16(value, regex.last_index, unicode);
            }
        }

        if !matched_any {
            return Ok(value.to_string());
        }

        out.push_str(&value[last_end..]);
        Ok(out)
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
            let mut utf16_offset = 0usize;
            for ch in &chars {
                let replacement = self.execute_callback_value(
                    callback,
                    &[
                        Value::String(String::new()),
                        Value::Number(utf16_offset as i64),
                        Value::String(value.to_string()),
                    ],
                    event,
                )?;
                out.push_str(&replacement.as_string());
                out.push(*ch);
                utf16_offset += ch.len_utf16();
            }
            let replacement = self.execute_callback_value(
                callback,
                &[
                    Value::String(String::new()),
                    Value::Number(utf16_offset as i64),
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
                        Value::Number(Self::byte_index_to_utf16_index(value, start) as i64),
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
                        Value::Number(Self::byte_index_to_utf16_index(value, start) as i64),
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

    fn regex_callback_replacement_from_exec(
        &mut self,
        callback: &Value,
        captures: &[Option<String>],
        index: usize,
        groups: Option<&[(String, Option<String>)]>,
        input: &str,
        event: &EventState,
    ) -> Result<String> {
        let mut args = Vec::new();
        args.push(
            captures
                .first()
                .and_then(|capture| capture.clone())
                .map(Value::String)
                .unwrap_or_else(|| Value::String(String::new())),
        );
        for capture in captures.iter().skip(1) {
            args.push(
                capture
                    .clone()
                    .map(Value::String)
                    .unwrap_or(Value::Undefined),
            );
        }
        args.push(Value::Number(index as i64));
        args.push(Value::String(input.to_string()));
        if let Some(groups) = groups {
            let mut entries = Vec::new();
            for (name, value) in groups {
                entries.push((
                    name.clone(),
                    value.clone().map(Value::String).unwrap_or(Value::Undefined),
                ));
            }
            args.push(Self::new_object_value(entries));
        }
        let replacement = self.execute_callback_value(callback, &args, event)?;
        Ok(replacement.as_string())
    }

    fn expand_regex_replacement_from_exec(
        template: &str,
        captures: &[Option<String>],
        groups: Option<&[(String, Option<String>)]>,
        full_match_start_byte: usize,
        full_match_end_byte: usize,
        input: &str,
    ) -> String {
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
                    if let Some(Some(full)) = captures.first() {
                        out.push_str(full);
                    }
                    i += 2;
                }
                '`' => {
                    out.push_str(&input[..full_match_start_byte]);
                    i += 2;
                }
                '\'' => {
                    out.push_str(&input[full_match_end_byte..]);
                    i += 2;
                }
                '<' => {
                    let mut j = i + 2;
                    while j < chars.len() && chars[j] != '>' {
                        j += 1;
                    }
                    if j >= chars.len() {
                        out.push('$');
                        out.push('<');
                        i += 2;
                        continue;
                    }
                    let name = chars[i + 2..j].iter().collect::<String>();
                    if let Some(groups) = groups {
                        if let Some((_, Some(value))) =
                            groups.iter().find(|(group_name, _)| group_name == &name)
                        {
                            out.push_str(value);
                        }
                    } else {
                        out.push('$');
                        out.push('<');
                        out.push_str(&name);
                        out.push('>');
                    }
                    i = j + 1;
                }
                '0'..='9' => {
                    let first_digit = (next as u8 - b'0') as usize;
                    if first_digit == 0 {
                        out.push('$');
                        out.push('0');
                        i += 2;
                        continue;
                    }

                    let group_count = captures.len().saturating_sub(1);
                    if i + 2 < chars.len() && chars[i + 2].is_ascii_digit() {
                        let candidate = first_digit * 10 + (chars[i + 2] as u8 - b'0') as usize;
                        if candidate > 0 && candidate <= group_count {
                            if let Some(Some(group)) = captures.get(candidate) {
                                out.push_str(group);
                            }
                            i += 3;
                            continue;
                        }
                    }

                    if first_digit <= group_count {
                        if let Some(Some(group)) = captures.get(first_digit) {
                            out.push_str(group);
                        }
                    } else {
                        out.push('$');
                        out.push(next);
                    }
                    i += 2;
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
