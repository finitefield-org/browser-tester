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
            || key == INTERNAL_SYMBOL_WRAPPER_KEY
            || key == INTERNAL_STRING_WRAPPER_VALUE_KEY
            || key.starts_with(INTERNAL_INTL_KEY_PREFIX)
            || key.starts_with(INTERNAL_CALLABLE_KEY_PREFIX)
            || key.starts_with(INTERNAL_CANVAS_KEY_PREFIX)
            || key.starts_with(INTERNAL_URL_SEARCH_PARAMS_KEY_PREFIX)
            || key.starts_with(INTERNAL_STORAGE_KEY_PREFIX)
            || key.starts_with(INTERNAL_ITERATOR_KEY_PREFIX)
            || key.starts_with(INTERNAL_ASYNC_ITERATOR_KEY_PREFIX)
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
        match method {
            SymbolStaticMethod::For => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Symbol.for requires exactly one argument".into(),
                    ));
                }
                let key = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
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
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Symbol.keyFor requires exactly one argument".into(),
                    ));
                }
                let symbol = self.eval_expr(&args[0], env, event_param, event)?;
                let Value::Symbol(symbol) = symbol else {
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
        match method {
            StringStaticMethod::FromCharCode => {
                let mut out = String::with_capacity(args.len());
                for arg in args {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    let unit = (Self::value_to_i64(&value) as i128).rem_euclid(1 << 16) as u16;
                    out.push(crate::js_regex::internalize_utf16_code_unit(unit));
                }
                Ok(Value::String(out))
            }
            StringStaticMethod::FromCodePoint => {
                let mut out = String::new();
                for arg in args {
                    let value = self.eval_expr(arg, env, event_param, event)?;
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
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "String.raw requires at least one argument".into(),
                    ));
                }
                let template = self.eval_expr(&args[0], env, event_param, event)?;
                let raw = match template {
                    Value::Object(entries) => {
                        let entries = entries.borrow();
                        Self::object_get_entry(&entries, "raw").unwrap_or(Value::Undefined)
                    }
                    other => other,
                };
                let raw_segments = self.array_like_values_from_value(&raw)?;
                let mut substitutions = Vec::with_capacity(args.len().saturating_sub(1));
                for arg in args.iter().skip(1) {
                    substitutions.push(self.eval_expr(arg, env, event_param, event)?.as_string());
                }
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

    pub(crate) fn eval_regexp_static_method(
        &mut self,
        method: RegExpStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            RegExpStaticMethod::Escape => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "RegExp.escape requires exactly one argument".into(),
                    ));
                }
                let value = self.eval_expr(&args[0], env, event_param, event)?;
                Ok(Value::String(regex_escape(&value.as_string()).into_owned()))
            }
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
