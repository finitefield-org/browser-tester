use super::*;

impl Harness {
    pub(crate) fn eval_map_construct(
        &mut self,
        iterable: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Map constructor must be called with new".into(),
            ));
        }

        let map = Rc::new(RefCell::new(MapValue {
            entries: Vec::new(),
            properties: ObjectValue::default(),
        }));

        let Some(iterable) = iterable else {
            return Ok(Value::Map(map));
        };

        let iterable = self.eval_expr(iterable, env, event_param, event)?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Map(map));
        }

        match iterable {
            Value::Map(source) => {
                let source = source.borrow();
                map.borrow_mut().entries = source.entries.clone();
            }
            other => {
                let entries = self.array_like_values_from_value(&other)?;
                for entry in entries {
                    let pair = self.array_like_values_from_value(&entry).map_err(|_| {
                        Error::ScriptRuntime(
                            "Map constructor iterable values must be [key, value] pairs".into(),
                        )
                    })?;
                    if pair.len() < 2 {
                        return Err(Error::ScriptRuntime(
                            "Map constructor iterable values must be [key, value] pairs".into(),
                        ));
                    }
                    self.map_set_entry(&mut map.borrow_mut(), pair[0].clone(), pair[1].clone());
                }
            }
        }

        Ok(Value::Map(map))
    }

    pub(crate) fn eval_map_static_method(
        &mut self,
        method: MapStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            MapStaticMethod::GroupBy => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.groupBy requires exactly two arguments".into(),
                    ));
                }
                let iterable = self.eval_expr(&args[0], env, event_param, event)?;
                let callback = self.eval_expr(&args[1], env, event_param, event)?;
                let values = self.array_like_values_from_value(&iterable)?;
                let map = Rc::new(RefCell::new(MapValue {
                    entries: Vec::new(),
                    properties: ObjectValue::default(),
                }));
                for (index, item) in values.into_iter().enumerate() {
                    let group_key = self.execute_callback_value(
                        &callback,
                        &[item.clone(), Value::Number(index as i64)],
                        event,
                    )?;
                    let mut map_ref = map.borrow_mut();
                    if let Some(entry_index) = self.map_entry_index(&map_ref, &group_key) {
                        match &mut map_ref.entries[entry_index].1 {
                            Value::Array(group_values) => group_values.borrow_mut().push(item),
                            _ => {
                                map_ref.entries[entry_index].1 = Self::new_array_value(vec![item]);
                            }
                        }
                    } else {
                        map_ref
                            .entries
                            .push((group_key, Self::new_array_value(vec![item])));
                    }
                }
                Ok(Value::Map(map))
            }
        }
    }

    pub(crate) fn eval_map_method(
        &mut self,
        target: &str,
        method: MapInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let target_value = env
            .get(target)
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {}", target)))?;

        if let Value::Set(set) = target_value {
            let set = set.clone();
            return match method {
                MapInstanceMethod::Has => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.has requires exactly one argument".into(),
                        ));
                    }
                    let key = self.eval_expr(&args[0], env, event_param, event)?;
                    Ok(Value::Bool(
                        self.set_value_index(&set.borrow(), &key).is_some(),
                    ))
                }
                MapInstanceMethod::Delete => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.delete requires exactly one argument".into(),
                        ));
                    }
                    let key = self.eval_expr(&args[0], env, event_param, event)?;
                    let mut set_ref = set.borrow_mut();
                    if let Some(index) = self.set_value_index(&set_ref, &key) {
                        set_ref.values.remove(index);
                        Ok(Value::Bool(true))
                    } else {
                        Ok(Value::Bool(false))
                    }
                }
                MapInstanceMethod::Clear => {
                    if !args.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "Map.clear does not take arguments".into(),
                        ));
                    }
                    set.borrow_mut().values.clear();
                    Ok(Value::Undefined)
                }
                MapInstanceMethod::ForEach => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Map.forEach requires a callback and optional thisArg".into(),
                        ));
                    }
                    let callback = self.eval_expr(&args[0], env, event_param, event)?;
                    if args.len() == 2 {
                        let _ = self.eval_expr(&args[1], env, event_param, event)?;
                    }
                    let snapshot = set.borrow().values.clone();
                    for value in snapshot {
                        let _ = self.execute_callback_value(
                            &callback,
                            &[value.clone(), value, Value::Set(set.clone())],
                            event,
                        )?;
                    }
                    Ok(Value::Undefined)
                }
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a Map",
                    target
                ))),
            };
        }

        if let Value::FormData(entries) = target_value {
            let entries = entries.clone();
            return match method {
                MapInstanceMethod::Get => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.get requires exactly one argument".into(),
                        ));
                    }
                    let key = self
                        .eval_expr(&args[0], env, event_param, event)?
                        .as_string();
                    let value = entries
                        .iter()
                        .find_map(|(entry_name, value)| (entry_name == &key).then(|| value.clone()))
                        .unwrap_or_default();
                    Ok(Value::String(value))
                }
                MapInstanceMethod::Has => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.has requires exactly one argument".into(),
                        ));
                    }
                    let key = self
                        .eval_expr(&args[0], env, event_param, event)?
                        .as_string();
                    let has = entries.iter().any(|(entry_name, _)| entry_name == &key);
                    Ok(Value::Bool(has))
                }
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a Map",
                    target
                ))),
            };
        }

        if let Value::Object(entries) = target_value {
            let entries = entries.clone();
            if Self::is_storage_object(&entries.borrow()) {
                return match method {
                    MapInstanceMethod::Clear => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Storage.clear does not take arguments".into(),
                            ));
                        }
                        Self::set_storage_pairs(&mut entries.borrow_mut(), &[]);
                        Ok(Value::Undefined)
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a Map",
                        target
                    ))),
                };
            }
            if Self::is_url_search_params_object(&entries.borrow()) {
                return match method {
                    MapInstanceMethod::Get => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.get requires exactly one argument".into(),
                            ));
                        }
                        let name = self
                            .eval_expr(&args[0], env, event_param, event)?
                            .as_string();
                        let pairs =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        let value = pairs
                            .into_iter()
                            .find_map(|(entry_name, entry_value)| {
                                (entry_name == name).then_some(entry_value)
                            })
                            .map(Value::String)
                            .unwrap_or(Value::Null);
                        Ok(value)
                    }
                    MapInstanceMethod::Has => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.has requires exactly one argument".into(),
                            ));
                        }
                        let name = self
                            .eval_expr(&args[0], env, event_param, event)?
                            .as_string();
                        let pairs =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        Ok(Value::Bool(
                            pairs.into_iter().any(|(entry_name, _)| entry_name == name),
                        ))
                    }
                    MapInstanceMethod::Delete => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.delete requires exactly one argument".into(),
                            ));
                        }
                        let name = self
                            .eval_expr(&args[0], env, event_param, event)?
                            .as_string();
                        {
                            let mut object_ref = entries.borrow_mut();
                            let mut pairs =
                                Self::url_search_params_pairs_from_object_entries(&object_ref);
                            pairs.retain(|(entry_name, _)| entry_name != &name);
                            Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                        }
                        self.sync_url_search_params_owner(&entries);
                        Ok(Value::Undefined)
                    }
                    MapInstanceMethod::ForEach => {
                        if args.is_empty() || args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.forEach requires a callback and optional thisArg"
                                    .into(),
                            ));
                        }
                        let callback = self.eval_expr(&args[0], env, event_param, event)?;
                        if args.len() == 2 {
                            let _ = self.eval_expr(&args[1], env, event_param, event)?;
                        }
                        let snapshot =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        for (entry_name, entry_value) in snapshot {
                            let _ = self.execute_callback_value(
                                &callback,
                                &[
                                    Value::String(entry_value),
                                    Value::String(entry_name),
                                    Value::Object(entries.clone()),
                                ],
                                event,
                            )?;
                        }
                        Ok(Value::Undefined)
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a Map",
                        target
                    ))),
                };
            }
        }

        let Value::Map(map) = target_value else {
            if matches!(method, MapInstanceMethod::Get | MapInstanceMethod::Has) {
                return Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a FormData instance",
                    target
                )));
            }
            return Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a Map",
                target
            )));
        };
        let map = map.clone();
        match method {
            MapInstanceMethod::Get => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Map.get requires exactly one argument".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let map_ref = map.borrow();
                if let Some(index) = self.map_entry_index(&map_ref, &key) {
                    Ok(map_ref.entries[index].1.clone())
                } else {
                    Ok(Value::Undefined)
                }
            }
            MapInstanceMethod::Has => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Map.has requires exactly one argument".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let has = self.map_entry_index(&map.borrow(), &key).is_some();
                Ok(Value::Bool(has))
            }
            MapInstanceMethod::Delete => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Map.delete requires exactly one argument".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let mut map_ref = map.borrow_mut();
                if let Some(index) = self.map_entry_index(&map_ref, &key) {
                    map_ref.entries.remove(index);
                    Ok(Value::Bool(true))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            MapInstanceMethod::Clear => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Map.clear does not take arguments".into(),
                    ));
                }
                map.borrow_mut().entries.clear();
                Ok(Value::Undefined)
            }
            MapInstanceMethod::ForEach => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.forEach requires a callback and optional thisArg".into(),
                    ));
                }
                let callback = self.eval_expr(&args[0], env, event_param, event)?;
                if args.len() == 2 {
                    let _ = self.eval_expr(&args[1], env, event_param, event)?;
                }
                let snapshot = map.borrow().entries.clone();
                for (key, value) in snapshot {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[value, key, Value::Map(map.clone())],
                        event,
                    )?;
                }
                Ok(Value::Undefined)
            }
            MapInstanceMethod::GetOrInsert => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.getOrInsert requires exactly two arguments".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let default_value = self.eval_expr(&args[1], env, event_param, event)?;
                let mut map_ref = map.borrow_mut();
                if let Some(index) = self.map_entry_index(&map_ref, &key) {
                    Ok(map_ref.entries[index].1.clone())
                } else {
                    map_ref.entries.push((key, default_value.clone()));
                    Ok(default_value)
                }
            }
            MapInstanceMethod::GetOrInsertComputed => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.getOrInsertComputed requires exactly two arguments".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                {
                    let map_ref = map.borrow();
                    if let Some(index) = self.map_entry_index(&map_ref, &key) {
                        return Ok(map_ref.entries[index].1.clone());
                    }
                }
                let callback = self.eval_expr(&args[1], env, event_param, event)?;
                let computed =
                    self.execute_callback_value(&callback, std::slice::from_ref(&key), event)?;
                map.borrow_mut().entries.push((key, computed.clone()));
                Ok(computed)
            }
        }
    }

    pub(crate) fn eval_set_construct(
        &mut self,
        iterable: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Set constructor must be called with new".into(),
            ));
        }

        let set = Rc::new(RefCell::new(SetValue {
            values: Vec::new(),
            properties: ObjectValue::default(),
        }));

        let Some(iterable) = iterable else {
            return Ok(Value::Set(set));
        };

        let iterable = self.eval_expr(iterable, env, event_param, event)?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Set(set));
        }

        let values = self.array_like_values_from_value(&iterable)?;
        for value in values {
            self.set_add_value(&mut set.borrow_mut(), value);
        }
        Ok(Value::Set(set))
    }

    pub(crate) fn eval_set_method(
        &mut self,
        target: &str,
        method: SetInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let target_value = env
            .get(target)
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {}", target)))?;
        let Value::Set(set) = target_value else {
            return Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a Set",
                target
            )));
        };
        let set = set.clone();

        match method {
            SetInstanceMethod::Add => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.add requires exactly one argument".into(),
                    ));
                }
                let value = self.eval_expr(&args[0], env, event_param, event)?;
                self.set_add_value(&mut set.borrow_mut(), value);
                Ok(Value::Set(set))
            }
            SetInstanceMethod::Union => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.union requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let other_keys = self.set_like_keys_snapshot(&other)?;
                let mut out = SetValue {
                    values: set.borrow().values.clone(),
                    properties: ObjectValue::default(),
                };
                for key in other_keys {
                    self.set_add_value(&mut out, key);
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::Intersection => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.intersection requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = set.borrow().values.clone();
                let mut out = SetValue {
                    values: Vec::new(),
                    properties: ObjectValue::default(),
                };
                for value in snapshot {
                    if self.set_like_has_value(&other, &value)? {
                        self.set_add_value(&mut out, value);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::Difference => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.difference requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = set.borrow().values.clone();
                let mut out = SetValue {
                    values: Vec::new(),
                    properties: ObjectValue::default(),
                };
                for value in snapshot {
                    if !self.set_like_has_value(&other, &value)? {
                        self.set_add_value(&mut out, value);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::SymmetricDifference => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.symmetricDifference requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let other_keys = self.set_like_keys_snapshot(&other)?;
                let mut out = SetValue {
                    values: set.borrow().values.clone(),
                    properties: ObjectValue::default(),
                };
                for key in other_keys {
                    if let Some(index) = self.set_value_index(&out, &key) {
                        out.values.remove(index);
                    } else {
                        out.values.push(key);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::IsDisjointFrom => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.isDisjointFrom requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                for value in &set.borrow().values {
                    if self.set_like_has_value(&other, value)? {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            SetInstanceMethod::IsSubsetOf => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.isSubsetOf requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                for value in &set.borrow().values {
                    if !self.set_like_has_value(&other, value)? {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            SetInstanceMethod::IsSupersetOf => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.isSupersetOf requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                for value in self.set_like_keys_snapshot(&other)? {
                    if self.set_value_index(&set.borrow(), &value).is_none() {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
        }
    }
}
