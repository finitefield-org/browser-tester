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
    ) -> Result<Option<Vec<String>>> {
        Self::regex_exec_internal(regex, input)
    }

    pub(crate) fn regex_exec_internal(
        regex: &Rc<RefCell<RegexValue>>,
        input: &str,
    ) -> Result<Option<Vec<String>>> {
        let mut regex = regex.borrow_mut();
        let start = if regex.global || regex.sticky {
            regex.last_index
        } else {
            0
        };
        if start > input.len() {
            regex.last_index = 0;
            return Ok(None);
        }

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
            regex.last_index = full_match.end();
        }

        let mut out = Vec::with_capacity(captures.len());
        for idx in 0..captures.len() {
            out.push(
                captures
                    .get(idx)
                    .map(|capture| capture.as_str().to_string())
                    .unwrap_or_default(),
            );
        }
        Ok(Some(out))
    }

}
