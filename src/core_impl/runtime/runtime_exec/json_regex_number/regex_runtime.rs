use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegexExecResult {
    pub(crate) captures: Vec<Option<String>>,
    pub(crate) index: usize,
    pub(crate) input: String,
    pub(crate) groups: Option<Vec<(String, Option<String>)>>,
    pub(crate) indices: Option<Vec<Option<(usize, usize)>>>,
    pub(crate) indices_groups: Option<Vec<(String, Option<(usize, usize)>)>>,
    pub(crate) full_match_start_byte: usize,
    pub(crate) full_match_end_byte: usize,
}

impl Harness {
    pub(crate) fn analyze_regex_flags(flags: &str) -> std::result::Result<RegexFlags, String> {
        let mut info = RegexFlags {
            global: false,
            ignore_case: false,
            multiline: false,
            dot_all: false,
            sticky: false,
            has_indices: false,
            unicode: false,
        };
        let mut seen = HashSet::new();
        for ch in flags.chars() {
            if !seen.insert(ch) {
                return Err(format!("invalid regular expression flags: {flags}"));
            }
            match ch {
                'g' => info.global = true,
                'i' => info.ignore_case = true,
                'm' => info.multiline = true,
                's' => info.dot_all = true,
                'y' => info.sticky = true,
                'd' => info.has_indices = true,
                'u' => info.unicode = true,
                'v' => {
                    return Err("invalid regular expression flags: v flag is not supported".into());
                }
                _ => return Err(format!("invalid regular expression flags: {flags}")),
            }
        }
        Ok(info)
    }

    pub(crate) fn compile_regex(
        pattern: &str,
        info: RegexFlags,
    ) -> std::result::Result<Regex, RegexError> {
        let mut builder = RegexBuilder::new(pattern);
        builder.case_insensitive(info.ignore_case);
        builder.multi_line(info.multiline);
        builder.dot_matches_new_line(info.dot_all);
        builder.unicode_mode(info.unicode);
        builder.build()
    }

    pub(crate) fn new_regex_value(pattern: String, flags: String) -> Result<Value> {
        let info = Self::analyze_regex_flags(&flags).map_err(Error::ScriptRuntime)?;
        let compiled = Self::compile_regex(&pattern, info).map_err(|err| {
            Error::ScriptRuntime(format!(
                "invalid regular expression: /{pattern}/{flags}: {err}"
            ))
        })?;
        Ok(Value::RegExp(Rc::new(RefCell::new(RegexValue {
            source: pattern,
            flags,
            global: info.global,
            ignore_case: info.ignore_case,
            multiline: info.multiline,
            dot_all: info.dot_all,
            sticky: info.sticky,
            has_indices: info.has_indices,
            unicode: info.unicode,
            compiled,
            last_index: 0,
            properties: ObjectValue::default(),
        }))))
    }

    pub(crate) fn new_regex_from_values(pattern: &Value, flags: Option<&Value>) -> Result<Value> {
        let pattern_text = match pattern {
            Value::RegExp(value) => value.borrow().source.clone(),
            _ => pattern.as_string(),
        };
        let flags_text = if let Some(flags) = flags {
            flags.as_string()
        } else if let Value::RegExp(value) = pattern {
            value.borrow().flags.clone()
        } else {
            String::new()
        };
        Self::new_regex_value(pattern_text, flags_text)
    }

    pub(crate) fn resolve_regex_from_value(value: &Value) -> Result<Rc<RefCell<RegexValue>>> {
        match value {
            Value::RegExp(regex) => Ok(regex.clone()),
            _ => Err(Error::ScriptRuntime("value is not a RegExp".into())),
        }
    }

    pub(crate) fn map_regex_runtime_error(err: RegexError) -> Error {
        Error::ScriptRuntime(format!("regular expression runtime error: {err}"))
    }

    pub(crate) fn regex_test(regex: &Rc<RefCell<RegexValue>>, input: &str) -> Result<bool> {
        Ok(Self::regex_exec_internal(regex, input)?.is_some())
    }

    pub(crate) fn regex_exec(
        regex: &Rc<RefCell<RegexValue>>,
        input: &str,
    ) -> Result<Option<RegexExecResult>> {
        Self::regex_exec_internal(regex, input)
    }

    pub(crate) fn regex_exec_result_to_value(result: RegexExecResult) -> Value {
        let RegexExecResult {
            captures,
            index,
            input,
            groups,
            indices,
            indices_groups,
            ..
        } = result;

        let values = captures
            .into_iter()
            .map(|capture| capture.map(Value::String).unwrap_or(Value::Undefined))
            .collect::<Vec<_>>();
        let array = Self::new_array_value(values);
        let Value::Array(array_ref) = &array else {
            unreachable!("new_array_value must create Value::Array");
        };

        let groups_value = if let Some(groups) = groups {
            let entries = groups
                .into_iter()
                .map(|(name, value)| (name, value.map(Value::String).unwrap_or(Value::Undefined)))
                .collect::<Vec<_>>();
            Self::new_object_value(entries)
        } else {
            Value::Undefined
        };

        Self::set_array_property(array_ref, "index".to_string(), Value::Number(index as i64));
        Self::set_array_property(array_ref, "input".to_string(), Value::String(input));
        Self::set_array_property(array_ref, "groups".to_string(), groups_value);

        if let Some(indices) = indices {
            let entries = indices
                .into_iter()
                .map(|span| match span {
                    Some((start, end)) => Self::new_array_value(vec![
                        Value::Number(start as i64),
                        Value::Number(end as i64),
                    ]),
                    None => Value::Undefined,
                })
                .collect::<Vec<_>>();
            let indices_value = Self::new_array_value(entries);
            let Value::Array(indices_ref) = &indices_value else {
                unreachable!("new_array_value must create Value::Array");
            };
            let groups_value = if let Some(groups) = indices_groups {
                let entries = groups
                    .into_iter()
                    .map(|(name, span)| {
                        let value = if let Some((start, end)) = span {
                            Self::new_array_value(vec![
                                Value::Number(start as i64),
                                Value::Number(end as i64),
                            ])
                        } else {
                            Value::Undefined
                        };
                        (name, value)
                    })
                    .collect::<Vec<_>>();
                Self::new_object_value(entries)
            } else {
                Value::Undefined
            };
            Self::set_array_property(indices_ref, "groups".to_string(), groups_value);
            Self::set_array_property(array_ref, "indices".to_string(), indices_value);
        }
        array
    }

    pub(crate) fn regex_exec_internal(
        regex: &Rc<RefCell<RegexValue>>,
        input: &str,
    ) -> Result<Option<RegexExecResult>> {
        let mut regex = regex.borrow_mut();
        let has_indices = regex.has_indices;
        let start_utf16 = if regex.global || regex.sticky {
            regex.last_index
        } else {
            0
        };
        if start_utf16 > Self::utf16_length(input) {
            regex.last_index = 0;
            return Ok(None);
        }
        let start = if let Some(start) = Self::utf16_index_to_byte(input, start_utf16) {
            start
        } else if regex.sticky {
            regex.last_index = 0;
            return Ok(None);
        } else if let Some(start) = Self::utf16_index_to_byte_ceil(input, start_utf16) {
            start
        } else {
            regex.last_index = 0;
            return Ok(None);
        };

        let captures = regex
            .compiled
            .captures_from_pos(input, start)
            .map_err(Self::map_regex_runtime_error)?;

        let Some(captures) = captures else {
            if regex.global || regex.sticky {
                regex.last_index = 0;
            }
            return Ok(None);
        };

        let Some(full_match) = captures.get(0) else {
            if regex.global || regex.sticky {
                regex.last_index = 0;
            }
            return Ok(None);
        };

        if regex.sticky && full_match.start() != start {
            regex.last_index = 0;
            return Ok(None);
        }

        if regex.global || regex.sticky {
            regex.last_index = Self::byte_index_to_utf16_index(input, full_match.end());
        }

        let mut out = Vec::with_capacity(captures.len());
        let mut indices = has_indices.then(|| Vec::with_capacity(captures.len()));
        for idx in 0..captures.len() {
            if let Some(capture) = captures.get(idx) {
                out.push(Some(capture.as_str().to_string()));
                if let Some(indices) = indices.as_mut() {
                    indices.push(Some((
                        Self::byte_index_to_utf16_index(input, capture.start()),
                        Self::byte_index_to_utf16_index(input, capture.end()),
                    )));
                }
            } else {
                out.push(None);
                if let Some(indices) = indices.as_mut() {
                    indices.push(None);
                }
            }
        }
        let index = Self::byte_index_to_utf16_index(input, full_match.start());
        let groups = if captures.has_named_groups() {
            let mut named = Vec::new();
            for name in captures.named_group_names() {
                named.push((
                    name.clone(),
                    captures
                        .get_named(&name)
                        .map(|capture| capture.as_str().to_string()),
                ));
            }
            Some(named)
        } else {
            None
        };
        let indices_groups = if has_indices && captures.has_named_groups() {
            let mut named = Vec::new();
            for name in captures.named_group_names() {
                named.push((
                    name.clone(),
                    captures.get_named(&name).map(|capture| {
                        (
                            Self::byte_index_to_utf16_index(input, capture.start()),
                            Self::byte_index_to_utf16_index(input, capture.end()),
                        )
                    }),
                ));
            }
            Some(named)
        } else {
            None
        };
        Ok(Some(RegexExecResult {
            captures: out,
            index,
            input: input.to_string(),
            groups,
            indices,
            indices_groups,
            full_match_start_byte: full_match.start(),
            full_match_end_byte: full_match.end(),
        }))
    }
}
