use super::*;

impl Harness {
    pub(crate) fn new_symbol_value(
        &mut self,
        description: Option<String>,
        registry_key: Option<String>,
    ) -> Value {
        let id = self.symbol_runtime.allocate_symbol_id();
        let symbol = Rc::new(SymbolValue {
            id,
            description,
            registry_key,
        });
        self.symbol_runtime.symbols_by_id.insert(id, symbol.clone());
        Value::Symbol(symbol)
    }

    pub(crate) fn symbol_storage_key(id: usize) -> String {
        format!("{INTERNAL_SYMBOL_KEY_PREFIX}{id}")
    }

    pub(crate) fn symbol_id_from_storage_key(key: &str) -> Option<usize> {
        key.strip_prefix(INTERNAL_SYMBOL_KEY_PREFIX)
            .and_then(|value| value.parse::<usize>().ok())
    }

    pub(crate) fn is_symbol_storage_key(key: &str) -> bool {
        key.starts_with(INTERNAL_SYMBOL_KEY_PREFIX)
    }

    pub(crate) fn is_internal_object_key(key: &str) -> bool {
        Self::is_symbol_storage_key(key)
            || key.starts_with(INTERNAL_OBJECT_GETTER_KEY_PREFIX)
            || key.starts_with(INTERNAL_OBJECT_SETTER_KEY_PREFIX)
            || key.starts_with(INTERNAL_ARRAY_HOLE_KEY_PREFIX)
            || key == INTERNAL_OBJECT_PROTOTYPE_KEY
            || key == INTERNAL_CLASS_SUPER_PROTOTYPE_KEY
            || key == INTERNAL_CLASS_SUPER_CONSTRUCTOR_KEY
            || key == INTERNAL_SYMBOL_WRAPPER_KEY
            || key == INTERNAL_STRING_WRAPPER_VALUE_KEY
            || key.starts_with(INTERNAL_INTL_KEY_PREFIX)
            || key.starts_with(INTERNAL_CALLABLE_KEY_PREFIX)
            || key.starts_with(INTERNAL_WORKER_KEY_PREFIX)
            || key.starts_with(INTERNAL_CANVAS_KEY_PREFIX)
            || key.starts_with(INTERNAL_NAMED_NODE_MAP_KEY_PREFIX)
            || key.starts_with(INTERNAL_URL_SEARCH_PARAMS_KEY_PREFIX)
            || key.starts_with(INTERNAL_STORAGE_KEY_PREFIX)
            || key.starts_with(INTERNAL_CLIPBOARD_ITEM_KEY_PREFIX)
            || key.starts_with(INTERNAL_MOCK_FILE_KEY_PREFIX)
            || key.starts_with(INTERNAL_DOM_STRING_MAP_KEY_PREFIX)
            || key.starts_with(INTERNAL_ITERATOR_KEY_PREFIX)
            || key.starts_with(INTERNAL_ASYNC_ITERATOR_KEY_PREFIX)
            || key.starts_with(INTERNAL_ASYNC_GENERATOR_KEY_PREFIX)
            || key.starts_with(INTERNAL_GENERATOR_KEY_PREFIX)
            || key.starts_with(INTERNAL_GENERATOR_FUNCTION_KEY_PREFIX)
            || key.starts_with(INTERNAL_ASYNC_GENERATOR_FUNCTION_KEY_PREFIX)
            || key.starts_with(INTERNAL_CSS_STYLE_SHEET_KEY_PREFIX)
            || key.starts_with(INTERNAL_COMPUTED_STYLE_KEY_PREFIX)
    }

    pub(crate) fn symbol_wrapper_id_from_object(entries: &[(String, Value)]) -> Option<usize> {
        let value = Self::object_get_entry(entries, INTERNAL_SYMBOL_WRAPPER_KEY)?;
        match value {
            Value::Number(value) if value >= 0 => Some(value as usize),
            _ => None,
        }
    }

    pub(crate) fn string_wrapper_value_from_object(entries: &[(String, Value)]) -> Option<String> {
        match Self::object_get_entry(entries, INTERNAL_STRING_WRAPPER_VALUE_KEY) {
            Some(Value::String(value)) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn symbol_id_from_property_key(&self, value: &Value) -> Option<usize> {
        match value {
            Value::Symbol(symbol) => Some(symbol.id),
            Value::Object(entries) => {
                let entries = entries.borrow();
                Self::symbol_wrapper_id_from_object(&entries)
            }
            _ => None,
        }
    }

    pub(crate) fn property_key_to_storage_key(&self, value: &Value) -> String {
        if let Some(symbol_id) = self.symbol_id_from_property_key(value) {
            Self::symbol_storage_key(symbol_id)
        } else {
            value.as_string()
        }
    }

    pub(crate) fn eval_symbol_construct(
        &mut self,
        description: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if called_with_new {
            return Err(Error::ScriptRuntime("Symbol is not a constructor".into()));
        }
        let description = if let Some(description) = description {
            let value = self.eval_expr(description, env, event_param, event)?;
            if matches!(value, Value::Undefined) {
                None
            } else {
                Some(value.as_string())
            }
        } else {
            None
        };
        Ok(self.new_symbol_value(description, None))
    }

    pub(crate) fn eval_symbol_static_method(
        &mut self,
        method: SymbolStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let evaluated_args = self.eval_call_args_with_spread(args, env, event_param, event)?;
        self.eval_symbol_static_method_from_values(method, &evaluated_args)
    }

    pub(crate) fn eval_symbol_static_method_from_values(
        &mut self,
        method: SymbolStaticMethod,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        match method {
            SymbolStaticMethod::For => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Symbol.for requires exactly one argument".into(),
                    ));
                }
                let key = evaluated_args[0].as_string();
                if let Some(symbol) = self.symbol_runtime.symbol_registry.get(&key) {
                    return Ok(Value::Symbol(symbol.clone()));
                }
                let symbol = match self.new_symbol_value(Some(key.clone()), Some(key.clone())) {
                    Value::Symbol(symbol) => symbol,
                    _ => unreachable!("new_symbol_value must create Symbol"),
                };
                self.symbol_runtime
                    .symbol_registry
                    .insert(key, symbol.clone());
                Ok(Value::Symbol(symbol))
            }
            SymbolStaticMethod::KeyFor => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Symbol.keyFor requires exactly one argument".into(),
                    ));
                }
                let Value::Symbol(symbol) = &evaluated_args[0] else {
                    return Err(Error::ScriptRuntime(
                        "Symbol.keyFor argument must be a Symbol".into(),
                    ));
                };
                if let Some(key) = &symbol.registry_key {
                    Ok(Value::String(key.clone()))
                } else {
                    Ok(Value::Undefined)
                }
            }
        }
    }

    pub(crate) fn symbol_static_property_name(property: SymbolStaticProperty) -> &'static str {
        match property {
            SymbolStaticProperty::AsyncDispose => "Symbol.asyncDispose",
            SymbolStaticProperty::AsyncIterator => "Symbol.asyncIterator",
            SymbolStaticProperty::Dispose => "Symbol.dispose",
            SymbolStaticProperty::HasInstance => "Symbol.hasInstance",
            SymbolStaticProperty::IsConcatSpreadable => "Symbol.isConcatSpreadable",
            SymbolStaticProperty::Iterator => "Symbol.iterator",
            SymbolStaticProperty::Match => "Symbol.match",
            SymbolStaticProperty::MatchAll => "Symbol.matchAll",
            SymbolStaticProperty::Replace => "Symbol.replace",
            SymbolStaticProperty::Search => "Symbol.search",
            SymbolStaticProperty::Species => "Symbol.species",
            SymbolStaticProperty::Split => "Symbol.split",
            SymbolStaticProperty::ToPrimitive => "Symbol.toPrimitive",
            SymbolStaticProperty::ToStringTag => "Symbol.toStringTag",
            SymbolStaticProperty::Unscopables => "Symbol.unscopables",
        }
    }

    pub(crate) fn eval_symbol_static_property(&mut self, property: SymbolStaticProperty) -> Value {
        let name = Self::symbol_static_property_name(property).to_string();
        if let Some(symbol) = self.symbol_runtime.well_known_symbols.get(&name) {
            return Value::Symbol(symbol.clone());
        }
        let symbol = match self.new_symbol_value(Some(name.clone()), None) {
            Value::Symbol(symbol) => symbol,
            _ => unreachable!("new_symbol_value must create Symbol"),
        };
        self.symbol_runtime
            .well_known_symbols
            .insert(name, symbol.clone());
        Value::Symbol(symbol)
    }

    pub(crate) fn eval_string_static_method(
        &mut self,
        method: StringStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let evaluated_args = self.eval_call_args_with_spread(args, env, event_param, event)?;
        self.eval_string_static_method_from_values(method, &evaluated_args)
    }

    pub(crate) fn eval_string_static_method_from_values(
        &mut self,
        method: StringStaticMethod,
        evaluated_args: &[Value],
    ) -> Result<Value> {
        match method {
            StringStaticMethod::FromCharCode => {
                let mut out = String::with_capacity(evaluated_args.len());
                for value in evaluated_args {
                    let unit = Self::to_u16_for_string_from_char_code(value);
                    out.push(crate::js_regex::internalize_utf16_code_unit(unit));
                }
                Ok(Value::String(out))
            }
            StringStaticMethod::FromCodePoint => {
                let mut out = String::new();
                for value in evaluated_args {
                    let n = Self::coerce_number_for_global(&value);
                    if !n.is_finite() || n.fract() != 0.0 || !(0.0..=0x10_FFFF as f64).contains(&n)
                    {
                        return Err(Error::ScriptRuntime(
                            "Invalid code point for String.fromCodePoint".into(),
                        ));
                    }
                    let cp = n as u32;
                    if (0xD800..=0xDFFF).contains(&cp) {
                        return Err(Error::ScriptRuntime(
                            "Invalid code point for String.fromCodePoint".into(),
                        ));
                    }
                    let ch = char::from_u32(cp).ok_or_else(|| {
                        Error::ScriptRuntime("Invalid code point for String.fromCodePoint".into())
                    })?;
                    let mut units = [0u16; 2];
                    let encoded = ch.encode_utf16(&mut units);
                    for unit in encoded {
                        out.push(crate::js_regex::internalize_utf16_code_unit(*unit));
                    }
                }
                Ok(Value::String(out))
            }
            StringStaticMethod::Raw => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "String.raw requires at least one argument".into(),
                    ));
                }
                let template = evaluated_args[0].clone();
                let raw = match template {
                    Value::Object(entries) => {
                        let entries = entries.borrow();
                        Self::object_get_entry(&entries, "raw").unwrap_or(Value::Undefined)
                    }
                    other => other,
                };
                let raw_segments = self.array_like_values_from_value(&raw)?;
                let substitutions = evaluated_args
                    .iter()
                    .skip(1)
                    .map(Value::as_string)
                    .collect::<Vec<_>>();
                if raw_segments.is_empty() {
                    return Ok(Value::String(String::new()));
                }
                let mut out = String::new();
                for (idx, segment) in raw_segments.iter().enumerate() {
                    out.push_str(&segment.as_string());
                    if let Some(substitution) = substitutions.get(idx) {
                        out.push_str(substitution);
                    }
                }
                Ok(Value::String(out))
            }
        }
    }

    pub(crate) fn to_u16_for_string_from_char_code(value: &Value) -> u16 {
        let number = Self::coerce_number_for_global(value);
        if !number.is_finite() {
            return 0;
        }
        number.trunc().rem_euclid(65_536.0) as u16
    }

    pub(crate) fn eval_regexp_static_method(
        &mut self,
        method: RegExpStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }
        self.eval_regexp_static_method_from_values(method, &values)
    }

    pub(crate) fn eval_regexp_static_method_from_values(
        &mut self,
        method: RegExpStaticMethod,
        args: &[Value],
    ) -> Result<Value> {
        match method {
            RegExpStaticMethod::Escape => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "RegExp.escape requires exactly one argument".into(),
                    ));
                }
                Ok(Value::String(
                    regex_escape(&args[0].as_string()).into_owned(),
                ))
            }
        }
    }

    pub(crate) fn eval_regexp_member_call_from_values(
        &mut self,
        regex: &Rc<RefCell<RegexValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "test" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "RegExp.test requires exactly one argument".into(),
                    ));
                }
                Ok(Some(Value::Bool(Self::regex_test(
                    regex,
                    &args[0].as_string(),
                )?)))
            }
            "exec" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "RegExp.exec requires zero or one argument".into(),
                    ));
                }
                let input = args
                    .first()
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .as_string();
                let Some(result) = Self::regex_exec(regex, &input)? else {
                    return Ok(Some(Value::Null));
                };
                Ok(Some(Self::regex_exec_result_to_value(result)))
            }
            "toString" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "RegExp.toString does not take arguments".into(),
                    ));
                }
                let regex = regex.borrow();
                Ok(Some(Value::String(format!(
                    "/{}/{}",
                    regex.source, regex.flags
                ))))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_string_match(&mut self, value: &str, pattern: Value) -> Result<Value> {
        let regex = if let Value::RegExp(regex) = pattern {
            regex
        } else {
            let compiled = Self::new_regex_from_values(&pattern, None)?;
            match compiled {
                Value::RegExp(regex) => regex,
                _ => unreachable!("RegExp constructor must return a RegExp"),
            }
        };

        let global = regex.borrow().global;
        if global {
            regex.borrow_mut().last_index = 0;
            let mut matches = Vec::new();
            loop {
                let Some(result) = Self::regex_exec(&regex, value)? else {
                    break;
                };
                if let Some(Some(matched)) = result.captures.first() {
                    matches.push(Value::String(matched.clone()));
                }
                if result.full_match_start_byte == result.full_match_end_byte {
                    let mut regex = regex.borrow_mut();
                    let unicode = regex.unicode || regex.unicode_sets;
                    regex.last_index =
                        Self::advance_string_index_utf16(value, regex.last_index, unicode);
                }
            }
            if matches.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(Self::new_array_value(matches))
            }
        } else {
            let Some(result) = Self::regex_exec(&regex, value)? else {
                return Ok(Value::Null);
            };
            Ok(Self::regex_exec_result_to_value(result))
        }
    }
}
