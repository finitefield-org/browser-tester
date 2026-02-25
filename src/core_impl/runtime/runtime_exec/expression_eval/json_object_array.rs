use super::*;

impl Harness {
    pub(crate) fn eval_expr_json_object_array(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::JsonParse(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    Self::parse_json_text(&value)
                }
                Expr::JsonStringify(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    match Self::json_stringify_top_level(&value)? {
                        Some(serialized) => Ok(Value::String(serialized)),
                        None => Ok(Value::Undefined),
                    }
                }
                Expr::ObjectConstruct { value } => {
                    let value = value
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .unwrap_or(Value::Undefined);
                    match value {
                        Value::Null | Value::Undefined => Ok(Self::new_object_value(Vec::new())),
                        Value::Object(object) => Ok(Value::Object(object)),
                        Value::Array(array) => Ok(Value::Array(array)),
                        Value::Date(date) => Ok(Value::Date(date)),
                        Value::Map(map) => Ok(Value::Map(map)),
                        Value::Set(set) => Ok(Value::Set(set)),
                        Value::Blob(blob) => Ok(Value::Blob(blob)),
                        Value::ArrayBuffer(buffer) => Ok(Value::ArrayBuffer(buffer)),
                        Value::TypedArray(array) => Ok(Value::TypedArray(array)),
                        Value::Promise(promise) => Ok(Value::Promise(promise)),
                        Value::RegExp(regex) => Ok(Value::RegExp(regex)),
                        Value::Symbol(symbol) => Ok(Self::new_object_value(vec![(
                            INTERNAL_SYMBOL_WRAPPER_KEY.to_string(),
                            Value::Number(symbol.id as i64),
                        )])),
                        primitive => Ok(Self::new_object_value(vec![(
                            "value".into(),
                            Value::String(primitive.as_string()),
                        )])),
                    }
                }
                Expr::ObjectLiteral(entries) => {
                    let mut object_entries = Vec::with_capacity(entries.len());
                    for entry in entries {
                        match entry {
                            ObjectLiteralEntry::Pair(key, value) => {
                                let key = match key {
                                    ObjectLiteralKey::Static(key) => key.clone(),
                                    ObjectLiteralKey::Computed(expr) => {
                                        let key = self.eval_expr(expr, env, event_param, event)?;
                                        self.property_key_to_storage_key(&key)
                                    }
                                };

                                let value = match value {
                                    Expr::Function {
                                        handler,
                                        name: _,
                                        is_async,
                                        is_generator,
                                        is_arrow,
                                        is_method,
                                    } if *is_method => {
                                        let super_prototype = match Self::object_get_entry(
                                            &object_entries,
                                            INTERNAL_OBJECT_PROTOTYPE_KEY,
                                        ) {
                                            Some(Value::Object(proto)) => {
                                                Some(Value::Object(proto))
                                            }
                                            _ => None,
                                        };
                                        self.make_function_value_with_super(
                                            handler.clone(),
                                            env,
                                            false,
                                            *is_async,
                                            *is_generator,
                                            *is_arrow,
                                            *is_method,
                                            None,
                                            super_prototype,
                                        )
                                    }
                                    _ => self.eval_expr(value, env, event_param, event)?,
                                };

                                Self::object_set_entry(&mut object_entries, key, value);
                            }
                            ObjectLiteralEntry::ProtoSetter(expr) => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                if matches!(value, Value::Object(_) | Value::Null) {
                                    Self::object_set_entry(
                                        &mut object_entries,
                                        INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                                        value,
                                    );
                                }
                            }
                            ObjectLiteralEntry::Getter(key, handler) => {
                                let key = match key {
                                    ObjectLiteralKey::Static(key) => key.clone(),
                                    ObjectLiteralKey::Computed(expr) => {
                                        let key = self.eval_expr(expr, env, event_param, event)?;
                                        self.property_key_to_storage_key(&key)
                                    }
                                };
                                let getter = self.make_function_value(
                                    handler.clone(),
                                    env,
                                    false,
                                    false,
                                    false,
                                    false,
                                    true,
                                );
                                let getter_key = Self::object_getter_storage_key(&key);
                                Self::object_set_entry(&mut object_entries, getter_key, getter);
                                Self::object_set_entry(&mut object_entries, key, Value::Undefined);
                            }
                            ObjectLiteralEntry::Setter(key, handler) => {
                                let key = match key {
                                    ObjectLiteralKey::Static(key) => key.clone(),
                                    ObjectLiteralKey::Computed(expr) => {
                                        let key = self.eval_expr(expr, env, event_param, event)?;
                                        self.property_key_to_storage_key(&key)
                                    }
                                };
                                let setter = self.make_function_value(
                                    handler.clone(),
                                    env,
                                    false,
                                    false,
                                    false,
                                    false,
                                    true,
                                );
                                let setter_key = Self::object_setter_storage_key(&key);
                                Self::object_set_entry(&mut object_entries, setter_key, setter);
                                Self::object_set_entry(&mut object_entries, key, Value::Undefined);
                            }
                            ObjectLiteralEntry::Spread(expr) => {
                                let spread_value = self.eval_expr(expr, env, event_param, event)?;
                                match spread_value {
                                    Value::Null | Value::Undefined => {}
                                    Value::Object(entries) => {
                                        let source = Value::Object(entries.clone());
                                        let keys = entries
                                            .borrow()
                                            .iter()
                                            .filter(|(key, _)| !Self::is_internal_object_key(key))
                                            .map(|(key, _)| key.clone())
                                            .collect::<Vec<_>>();
                                        for key in keys {
                                            let value =
                                                self.object_property_from_value(&source, &key)?;
                                            Self::object_set_entry(&mut object_entries, key, value);
                                        }
                                    }
                                    Value::Array(values) => {
                                        for (index, value) in values.borrow().iter().enumerate() {
                                            Self::object_set_entry(
                                                &mut object_entries,
                                                index.to_string(),
                                                value.clone(),
                                            );
                                        }
                                    }
                                    Value::String(text) => {
                                        for (index, ch) in text.chars().enumerate() {
                                            Self::object_set_entry(
                                                &mut object_entries,
                                                index.to_string(),
                                                Value::String(ch.to_string()),
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Ok(Self::new_object_value(object_entries))
                }
                Expr::ObjectGet { target, key } => match env.get(target) {
                    Some(value) => {
                        self.object_property_from_value(value, key)
                            .map_err(|err| match err {
                                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                                    Error::ScriptRuntime(format!(
                                        "variable '{}' is not an object (key '{}')",
                                        target, key
                                    ))
                                }
                                other => other,
                            })
                    }
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ObjectPathGet { target, path } => {
                    let Some(mut value) = env.get(target).cloned() else {
                        return Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        )));
                    };
                    for key in path {
                        value = self.object_property_from_value(&value, key)?;
                    }
                    Ok(value)
                }
                Expr::ObjectGetOwnPropertySymbols(object) => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    match object {
                        Value::Object(entries) => {
                            let mut out = Vec::new();
                            for (key, _) in entries.borrow().iter() {
                                if let Some(symbol_id) = Self::symbol_id_from_storage_key(key) {
                                    if let Some(symbol) =
                                        self.symbol_runtime.symbols_by_id.get(&symbol_id)
                                    {
                                        out.push(Value::Symbol(symbol.clone()));
                                    }
                                }
                            }
                            Ok(Self::new_array_value(out))
                        }
                        _ => Err(Error::ScriptRuntime(
                            "Object.getOwnPropertySymbols argument must be an object".into(),
                        )),
                    }
                }
                Expr::ObjectKeys(object) => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    match object {
                        Value::Object(entries) => {
                            let keys = entries
                                .borrow()
                                .iter()
                                .filter(|(key, _)| !Self::is_internal_object_key(key))
                                .map(|(key, _)| Value::String(key.clone()))
                                .collect::<Vec<_>>();
                            Ok(Self::new_array_value(keys))
                        }
                        _ => Err(Error::ScriptRuntime(
                            "Object.keys argument must be an object".into(),
                        )),
                    }
                }
                Expr::ObjectValues(object) => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    match object {
                        Value::Object(entries) => {
                            let source = Value::Object(entries.clone());
                            let keys = entries
                                .borrow()
                                .iter()
                                .filter(|(key, _)| !Self::is_internal_object_key(key))
                                .map(|(key, _)| key.clone())
                                .collect::<Vec<_>>();
                            let mut values = Vec::with_capacity(keys.len());
                            for key in keys {
                                values.push(self.object_property_from_value(&source, &key)?);
                            }
                            Ok(Self::new_array_value(values))
                        }
                        _ => Err(Error::ScriptRuntime(
                            "Object.values argument must be an object".into(),
                        )),
                    }
                }
                Expr::ObjectEntries(object) => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    match object {
                        Value::Object(entries) => {
                            let source = Value::Object(entries.clone());
                            let keys = entries
                                .borrow()
                                .iter()
                                .filter(|(key, _)| !Self::is_internal_object_key(key))
                                .map(|(key, _)| key.clone())
                                .collect::<Vec<_>>();
                            let mut values = Vec::with_capacity(keys.len());
                            for key in keys {
                                let value = self.object_property_from_value(&source, &key)?;
                                values.push(Self::new_array_value(vec![Value::String(key), value]));
                            }
                            Ok(Self::new_array_value(values))
                        }
                        _ => Err(Error::ScriptRuntime(
                            "Object.entries argument must be an object".into(),
                        )),
                    }
                }
                Expr::ObjectHasOwn { object, key } => {
                    let object = self.eval_expr(object, env, event_param, event)?;
                    let key = self.eval_expr(key, env, event_param, event)?;
                    let key = self.property_key_to_storage_key(&key);
                    match object {
                        Value::Object(entries) => Ok(Value::Bool(
                            Self::object_get_entry(&entries.borrow(), &key).is_some(),
                        )),
                        _ => Err(Error::ScriptRuntime(
                            "Object.hasOwn first argument must be an object".into(),
                        )),
                    }
                }
                Expr::ObjectGetPrototypeOf(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    match value {
                        Value::TypedArrayConstructor(TypedArrayConstructorKind::Concrete(_)) => Ok(
                            Value::TypedArrayConstructor(TypedArrayConstructorKind::Abstract),
                        ),
                        Value::TypedArray(_) => Ok(Value::TypedArrayConstructor(
                            TypedArrayConstructorKind::Abstract,
                        )),
                        _ => Ok(Value::Object(Rc::new(RefCell::new(ObjectValue::default())))),
                    }
                }
                Expr::ObjectFreeze(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    match value {
                        Value::TypedArray(array) => {
                            if array.borrow().observed_length() > 0 {
                                return Err(Error::ScriptRuntime(
                                    "Cannot freeze array buffer views with elements".into(),
                                ));
                            }
                            Ok(Value::TypedArray(array))
                        }
                        other => Ok(other),
                    }
                }
                Expr::ObjectHasOwnProperty { target, key } => {
                    let key = self.eval_expr(key, env, event_param, event)?;
                    let key = self.property_key_to_storage_key(&key);
                    match env.get(target) {
                        Some(Value::Object(entries)) => Ok(Value::Bool(
                            Self::object_get_entry(&entries.borrow(), &key).is_some(),
                        )),
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an object",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArrayConstruct { args, .. } => {
                    let evaluated =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    if evaluated.is_empty() {
                        return Ok(Self::new_array_value(Vec::new()));
                    }
                    if evaluated.len() == 1 {
                        let first = &evaluated[0];
                        if let Some(length) = Self::array_constructor_length_from_value(first)? {
                            let mut out = Vec::new();
                            out.resize(length, Value::Undefined);
                            return Ok(Self::new_array_value(out));
                        }
                        return Ok(Self::new_array_value(vec![first.clone()]));
                    }
                    Ok(Self::new_array_value(evaluated))
                }
                Expr::ArrayLiteral(values) => {
                    let mut out = Vec::with_capacity(values.len());
                    for value in values {
                        match value {
                            Expr::Spread(expr) => {
                                let spread_value = self.eval_expr(expr, env, event_param, event)?;
                                out.extend(self.spread_iterable_values_from_value(&spread_value)?);
                            }
                            _ => out.push(self.eval_expr(value, env, event_param, event)?),
                        }
                    }
                    Ok(Self::new_array_value(out))
                }
                Expr::ArrayIsArray(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    Ok(Value::Bool(matches!(value, Value::Array(_))))
                }
                Expr::ArrayFrom { source, map_fn } => {
                    let source = self.eval_expr(source, env, event_param, event)?;
                    let values = self.array_like_values_from_value(&source)?;
                    if let Some(map_fn) = map_fn {
                        let callback = self.eval_expr(map_fn, env, event_param, event)?;
                        let mut mapped = Vec::with_capacity(values.len());
                        for (index, value) in values.into_iter().enumerate() {
                            mapped.push(self.execute_callback_value(
                                &callback,
                                &[value, Value::Number(index as i64)],
                                event,
                            )?);
                        }
                        return Ok(Self::new_array_value(mapped));
                    }
                    Ok(Self::new_array_value(values))
                }
                Expr::ArrayLength(target) => match env.get(target) {
                    Some(Value::Array(values)) => Ok(Value::Number(values.borrow().len() as i64)),
                    Some(Value::TypedArray(values)) => {
                        Ok(Value::Number(values.borrow().observed_length() as i64))
                    }
                    Some(Value::NodeList(nodes)) => Ok(Value::Number(nodes.len() as i64)),
                    Some(Value::String(value)) => Ok(Value::Number(value.chars().count() as i64)),
                    Some(Value::Function(function)) => {
                        let mut length = 0_i64;
                        for param in &function.handler.params {
                            if param.is_rest || param.default.is_some() {
                                break;
                            }
                            length += 1;
                        }
                        Ok(Value::Number(length))
                    }
                    Some(Value::Object(entries)) => {
                        let entries = entries.borrow();
                        if Self::is_history_object(&entries) {
                            return Ok(Self::object_get_entry(&entries, "length").unwrap_or(
                                Value::Number(self.location_history.history_entries.len() as i64),
                            ));
                        }
                        if Self::is_window_object(&entries) {
                            return Ok(Self::object_get_entry(&entries, "length")
                                .unwrap_or(Value::Number(0)));
                        }
                        if Self::is_storage_object(&entries) {
                            let len = Self::storage_pairs_from_object_entries(&entries).len();
                            return Ok(Value::Number(len as i64));
                        }
                        if let Some(value) = Self::string_wrapper_value_from_object(&entries) {
                            Ok(Value::Number(value.chars().count() as i64))
                        } else {
                            Ok(Self::object_get_entry(&entries, "length")
                                .unwrap_or(Value::Undefined))
                        }
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArrayIndex { target, index } => {
                    let index = self.eval_expr(index, env, event_param, event)?;
                    match env.get(target) {
                        Some(Value::Object(entries)) => {
                            let entries_ref = entries.borrow();
                            if let Some(value) =
                                Self::string_wrapper_value_from_object(&entries_ref)
                            {
                                let Some(index) = self.value_as_index(&index) else {
                                    return Ok(Value::Undefined);
                                };
                                return Ok(value
                                    .chars()
                                    .nth(index)
                                    .map(|ch| Value::String(ch.to_string()))
                                    .unwrap_or(Value::Undefined));
                            }
                            let key = self.property_key_to_storage_key(&index);
                            if Self::is_storage_object(&entries_ref) {
                                return Ok(Self::storage_pairs_from_object_entries(&entries_ref)
                                    .into_iter()
                                    .find_map(|(name, value)| {
                                        (name == key).then(|| Value::String(value))
                                    })
                                    .unwrap_or(Value::Undefined));
                            }
                            Ok(Self::object_get_entry(&entries_ref, &key)
                                .unwrap_or(Value::Undefined))
                        }
                        Some(Value::Array(values)) => {
                            let Some(index) = self.value_as_index(&index) else {
                                return Ok(Value::Undefined);
                            };
                            Ok(values
                                .borrow()
                                .get(index)
                                .cloned()
                                .unwrap_or(Value::Undefined))
                        }
                        Some(Value::TypedArray(values)) => {
                            let Some(index) = self.value_as_index(&index) else {
                                return Ok(Value::Undefined);
                            };
                            self.typed_array_get_index(values, index)
                        }
                        Some(Value::NodeList(nodes)) => {
                            let Some(index) = self.value_as_index(&index) else {
                                return Ok(Value::Undefined);
                            };
                            Ok(nodes
                                .get(index)
                                .copied()
                                .map(Value::Node)
                                .unwrap_or(Value::Undefined))
                        }
                        Some(Value::String(value)) => {
                            let Some(index) = self.value_as_index(&index) else {
                                return Ok(Value::Undefined);
                            };
                            Ok(value
                                .chars()
                                .nth(index)
                                .map(|ch| Value::String(ch.to_string()))
                                .unwrap_or(Value::Undefined))
                        }
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArrayPush { target, args } => {
                    let values = self.resolve_array_from_env(env, target)?;
                    let evaluated =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    let mut values = values.borrow_mut();
                    values.extend(evaluated);
                    Ok(Value::Number(values.len() as i64))
                }
                Expr::ArrayPop(target) => {
                    let values = self.resolve_array_from_env(env, target)?;
                    Ok(values.borrow_mut().pop().unwrap_or(Value::Undefined))
                }
                Expr::ArrayShift(target) => {
                    let values = self.resolve_array_from_env(env, target)?;
                    let mut values = values.borrow_mut();
                    if values.is_empty() {
                        Ok(Value::Undefined)
                    } else {
                        Ok(values.remove(0))
                    }
                }
                Expr::ArrayUnshift { target, args } => {
                    let values = self.resolve_array_from_env(env, target)?;
                    let evaluated =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    let mut values = values.borrow_mut();
                    for value in evaluated.into_iter().rev() {
                        values.insert(0, value);
                    }
                    Ok(Value::Number(values.len() as i64))
                }
                Expr::ArrayMap { target, callback } => match env.get(target) {
                    Some(Value::Array(values)) => {
                        let input = values.borrow().clone();
                        let mut out = Vec::with_capacity(input.len());
                        for (idx, item) in input.into_iter().enumerate() {
                            let mapped = self.execute_array_callback(
                                callback,
                                &[
                                    item,
                                    Value::Number(idx as i64),
                                    Value::Array(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            out.push(mapped);
                        }
                        Ok(Self::new_array_value(out))
                    }
                    Some(Value::TypedArray(values)) => {
                        let input = self.typed_array_snapshot(values)?;
                        let kind = values.borrow().kind;
                        let mut out = Vec::with_capacity(input.len());
                        for (idx, item) in input.into_iter().enumerate() {
                            let mapped = self.execute_array_callback(
                                callback,
                                &[
                                    item,
                                    Value::Number(idx as i64),
                                    Value::TypedArray(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            out.push(mapped);
                        }
                        self.new_typed_array_from_values(kind, &out)
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArrayFilter { target, callback } => match env.get(target) {
                    Some(Value::Array(values)) => {
                        let input = values.borrow().clone();
                        let mut out = Vec::new();
                        for (idx, item) in input.into_iter().enumerate() {
                            let keep = self.execute_array_callback(
                                callback,
                                &[
                                    item.clone(),
                                    Value::Number(idx as i64),
                                    Value::Array(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            if keep.truthy() {
                                out.push(item);
                            }
                        }
                        Ok(Self::new_array_value(out))
                    }
                    Some(Value::TypedArray(values)) => {
                        let input = self.typed_array_snapshot(values)?;
                        let kind = values.borrow().kind;
                        let mut out = Vec::new();
                        for (idx, item) in input.into_iter().enumerate() {
                            let keep = self.execute_array_callback(
                                callback,
                                &[
                                    item.clone(),
                                    Value::Number(idx as i64),
                                    Value::TypedArray(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            if keep.truthy() {
                                out.push(item);
                            }
                        }
                        self.new_typed_array_from_values(kind, &out)
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArrayReduce {
                    target,
                    callback,
                    initial,
                } => match env.get(target) {
                    Some(Value::Array(values)) => {
                        let input = values.borrow().clone();
                        let mut start_index = 0usize;
                        let mut acc = if let Some(initial) = initial {
                            self.eval_expr(initial, env, event_param, event)?
                        } else {
                            let Some(first) = input.first().cloned() else {
                                return Err(Error::ScriptRuntime(
                                    "reduce of empty array with no initial value".into(),
                                ));
                            };
                            start_index = 1;
                            first
                        };
                        for (idx, item) in input.into_iter().enumerate().skip(start_index) {
                            acc = self.execute_array_callback(
                                callback,
                                &[
                                    acc,
                                    item,
                                    Value::Number(idx as i64),
                                    Value::Array(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                        }
                        Ok(acc)
                    }
                    Some(Value::TypedArray(values)) => {
                        let input = self.typed_array_snapshot(values)?;
                        let mut start_index = 0usize;
                        let mut acc = if let Some(initial) = initial {
                            self.eval_expr(initial, env, event_param, event)?
                        } else {
                            let Some(first) = input.first().cloned() else {
                                return Err(Error::ScriptRuntime(
                                    "reduce of empty array with no initial value".into(),
                                ));
                            };
                            start_index = 1;
                            first
                        };
                        for (idx, item) in input.into_iter().enumerate().skip(start_index) {
                            acc = self.execute_array_callback(
                                callback,
                                &[
                                    acc,
                                    item,
                                    Value::Number(idx as i64),
                                    Value::TypedArray(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                        }
                        Ok(acc)
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArrayForEach { target, callback } => match env.get(target) {
                    Some(Value::Array(values)) => {
                        let input = values.borrow().clone();
                        for (idx, item) in input.into_iter().enumerate() {
                            let _ = self.execute_array_callback(
                                callback,
                                &[
                                    item,
                                    Value::Number(idx as i64),
                                    Value::Array(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                        }
                        Ok(Value::Undefined)
                    }
                    Some(Value::TypedArray(values)) => {
                        let input = self.typed_array_snapshot(values)?;
                        for (idx, item) in input.into_iter().enumerate() {
                            let _ = self.execute_array_callback(
                                callback,
                                &[
                                    item,
                                    Value::Number(idx as i64),
                                    Value::TypedArray(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                        }
                        Ok(Value::Undefined)
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArrayFind { target, callback } => match env.get(target) {
                    Some(Value::Array(values)) => {
                        let input = values.borrow().clone();
                        for (idx, item) in input.into_iter().enumerate() {
                            let matched = self.execute_array_callback(
                                callback,
                                &[
                                    item.clone(),
                                    Value::Number(idx as i64),
                                    Value::Array(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            if matched.truthy() {
                                return Ok(item);
                            }
                        }
                        Ok(Value::Undefined)
                    }
                    Some(Value::TypedArray(values)) => {
                        let input = self.typed_array_snapshot(values)?;
                        for (idx, item) in input.into_iter().enumerate() {
                            let matched = self.execute_array_callback(
                                callback,
                                &[
                                    item.clone(),
                                    Value::Number(idx as i64),
                                    Value::TypedArray(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            if matched.truthy() {
                                return Ok(item);
                            }
                        }
                        Ok(Value::Undefined)
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArrayFindIndex { target, callback } => match env.get(target) {
                    Some(Value::Array(values)) => {
                        let input = values.borrow().clone();
                        for (idx, item) in input.into_iter().enumerate() {
                            let matched = self.execute_array_callback(
                                callback,
                                &[
                                    item,
                                    Value::Number(idx as i64),
                                    Value::Array(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            if matched.truthy() {
                                return Ok(Value::Number(idx as i64));
                            }
                        }
                        Ok(Value::Number(-1))
                    }
                    Some(Value::TypedArray(values)) => {
                        let input = self.typed_array_snapshot(values)?;
                        for (idx, item) in input.into_iter().enumerate() {
                            let matched = self.execute_array_callback(
                                callback,
                                &[
                                    item,
                                    Value::Number(idx as i64),
                                    Value::TypedArray(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            if matched.truthy() {
                                return Ok(Value::Number(idx as i64));
                            }
                        }
                        Ok(Value::Number(-1))
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArraySome { target, callback } => match env.get(target) {
                    Some(Value::Array(values)) => {
                        let input = values.borrow().clone();
                        for (idx, item) in input.into_iter().enumerate() {
                            let matched = self.execute_array_callback(
                                callback,
                                &[
                                    item,
                                    Value::Number(idx as i64),
                                    Value::Array(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            if matched.truthy() {
                                return Ok(Value::Bool(true));
                            }
                        }
                        Ok(Value::Bool(false))
                    }
                    Some(Value::TypedArray(values)) => {
                        let input = self.typed_array_snapshot(values)?;
                        for (idx, item) in input.into_iter().enumerate() {
                            let matched = self.execute_array_callback(
                                callback,
                                &[
                                    item,
                                    Value::Number(idx as i64),
                                    Value::TypedArray(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            if matched.truthy() {
                                return Ok(Value::Bool(true));
                            }
                        }
                        Ok(Value::Bool(false))
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArrayEvery { target, callback } => match env.get(target) {
                    Some(Value::Array(values)) => {
                        let input = values.borrow().clone();
                        for (idx, item) in input.into_iter().enumerate() {
                            let matched = self.execute_array_callback(
                                callback,
                                &[
                                    item,
                                    Value::Number(idx as i64),
                                    Value::Array(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            if !matched.truthy() {
                                return Ok(Value::Bool(false));
                            }
                        }
                        Ok(Value::Bool(true))
                    }
                    Some(Value::TypedArray(values)) => {
                        let input = self.typed_array_snapshot(values)?;
                        for (idx, item) in input.into_iter().enumerate() {
                            let matched = self.execute_array_callback(
                                callback,
                                &[
                                    item,
                                    Value::Number(idx as i64),
                                    Value::TypedArray(values.clone()),
                                ],
                                env,
                                event,
                            )?;
                            if !matched.truthy() {
                                return Ok(Value::Bool(false));
                            }
                        }
                        Ok(Value::Bool(true))
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArrayIncludes {
                    target,
                    search,
                    from_index,
                } => {
                    let search = self.eval_expr(search, env, event_param, event)?;
                    match env.get(target) {
                        Some(Value::Array(values)) => {
                            let values = values.borrow();
                            let len = values.len() as i64;
                            let mut start = from_index
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .unwrap_or(0);
                            if start < 0 {
                                start = (len + start).max(0);
                            }
                            let start = start.min(len) as usize;
                            for value in values.iter().skip(start) {
                                if self.strict_equal(value, &search) {
                                    return Ok(Value::Bool(true));
                                }
                            }
                            Ok(Value::Bool(false))
                        }
                        Some(Value::TypedArray(values)) => {
                            let values_vec = self.typed_array_snapshot(values)?;
                            let len = values_vec.len() as i64;
                            let mut start = from_index
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .unwrap_or(0);
                            if start < 0 {
                                start = (len + start).max(0);
                            }
                            let start = start.min(len) as usize;
                            for value in values_vec.iter().skip(start) {
                                if self.strict_equal(value, &search) {
                                    return Ok(Value::Bool(true));
                                }
                            }
                            Ok(Value::Bool(false))
                        }
                        Some(Value::String(value)) => {
                            let search = search.as_string();
                            let len = value.chars().count() as i64;
                            let mut start = from_index
                                .as_ref()
                                .map(|value| self.eval_expr(value, env, event_param, event))
                                .transpose()?
                                .map(|value| Self::value_to_i64(&value))
                                .unwrap_or(0);
                            if start < 0 {
                                start = (len + start).max(0);
                            }
                            let start = start.min(len) as usize;
                            let start_byte = Self::char_index_to_byte(value, start);
                            Ok(Value::Bool(value[start_byte..].contains(&search)))
                        }
                        Some(_) => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        ))),
                        None => Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        ))),
                    }
                }
                Expr::ArraySlice { target, start, end } => match env.get(target) {
                    Some(Value::Array(values)) => {
                        let values = values.borrow();
                        let len = values.len();
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
                        Ok(Self::new_array_value(values[start..end].to_vec()))
                    }
                    Some(Value::TypedArray(values)) => {
                        let snapshot = self.typed_array_snapshot(values)?;
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
                    Some(Value::ArrayBuffer(buffer)) => {
                        Self::ensure_array_buffer_not_detached(buffer, "slice")?;
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
                    Some(Value::Blob(blob)) => {
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
                    Some(Value::String(value)) => {
                        let len = value.chars().count();
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
                        Ok(Value::String(Self::substring_chars(value, start, end)))
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                },
                Expr::ArraySplice {
                    target,
                    start,
                    delete_count,
                    items,
                } => {
                    let values = self.resolve_array_from_env(env, target)?;
                    let start = self.eval_expr(start, env, event_param, event)?;
                    let start = Self::value_to_i64(&start);
                    let delete_count = delete_count
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value));
                    let insert_items =
                        self.eval_call_args_with_spread(items, env, event_param, event)?;

                    let mut values = values.borrow_mut();
                    let len = values.len();
                    let start = Self::normalize_splice_start_index(len, start);
                    let delete_count = delete_count
                        .unwrap_or((len.saturating_sub(start)) as i64)
                        .max(0) as usize;
                    let delete_count = delete_count.min(len.saturating_sub(start));
                    let removed = values
                        .drain(start..start + delete_count)
                        .collect::<Vec<_>>();
                    for (offset, item) in insert_items.into_iter().enumerate() {
                        values.insert(start + offset, item);
                    }
                    Ok(Self::new_array_value(removed))
                }
                Expr::ArrayJoin { target, separator } => {
                    let separator = separator
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| value.as_string())
                        .unwrap_or_else(|| ",".to_string());
                    let values = match env.get(target) {
                        Some(Value::Array(values)) => values.borrow().clone(),
                        Some(Value::TypedArray(values)) => self.typed_array_snapshot(values)?,
                        Some(_) => {
                            return Err(Error::ScriptRuntime(format!(
                                "variable '{}' is not an array",
                                target
                            )));
                        }
                        None => {
                            return Err(Error::ScriptRuntime(format!(
                                "unknown variable: {}",
                                target
                            )));
                        }
                    };
                    let mut out = String::new();
                    for (idx, value) in values.iter().enumerate() {
                        if idx > 0 {
                            out.push_str(&separator);
                        }
                        if matches!(value, Value::Null | Value::Undefined) {
                            continue;
                        }
                        out.push_str(&value.as_string());
                    }
                    Ok(Value::String(out))
                }
                Expr::ArraySort { target, comparator } => {
                    let comparator = comparator
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?;
                    if comparator
                        .as_ref()
                        .is_some_and(|value| !self.is_callable_value(value))
                    {
                        return Err(Error::ScriptRuntime("callback is not a function".into()));
                    }

                    let values = self.resolve_array_from_env(env, target)?;
                    let mut snapshot = values.borrow().clone();
                    let len = snapshot.len();
                    for i in 0..len {
                        let end = len.saturating_sub(i + 1);
                        for j in 0..end {
                            let should_swap = if let Some(comparator) = comparator.as_ref() {
                                let compared = self.execute_callable_value(
                                    comparator,
                                    &[snapshot[j].clone(), snapshot[j + 1].clone()],
                                    event,
                                )?;
                                Self::coerce_number_for_global(&compared) > 0.0
                            } else {
                                snapshot[j].as_string() > snapshot[j + 1].as_string()
                            };
                            if should_swap {
                                snapshot.swap(j, j + 1);
                            }
                        }
                    }
                    values.borrow_mut().elements = snapshot;
                    Ok(Value::Array(values))
                }
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }

    fn array_constructor_length_from_value(value: &Value) -> Result<Option<usize>> {
        match value {
            Value::Number(value) => {
                if *value < 0 {
                    return Err(Error::ScriptRuntime("invalid array length".into()));
                }
                Ok(Some(usize::try_from(*value).map_err(|_| {
                    Error::ScriptRuntime("invalid array length".into())
                })?))
            }
            Value::Float(value) => {
                if !value.is_finite() || *value < 0.0 || value.fract() != 0.0 {
                    return Err(Error::ScriptRuntime("invalid array length".into()));
                }
                if *value > usize::MAX as f64 {
                    return Err(Error::ScriptRuntime("invalid array length".into()));
                }
                Ok(Some(*value as usize))
            }
            _ => Ok(None),
        }
    }
}
