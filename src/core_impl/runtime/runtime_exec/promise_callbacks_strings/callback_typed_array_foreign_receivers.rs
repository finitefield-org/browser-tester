use super::*;

impl Harness {
    pub(crate) fn eval_typed_array_foreign_receiver_method(
        &mut self,
        target: &str,
        target_value: &Value,
        method: TypedArrayInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if let Some(value) = self.eval_cache_storage_typed_array_method_dispatch(
            target_value,
            method,
            args,
            env,
            event_param,
            event,
        )? {
            return Ok(value);
        }

        if let Value::Map(map) = target_value {
            let evaluated_args = args
                .iter()
                .map(|arg| self.eval_expr(arg, env, event_param, event))
                .collect::<Result<Vec<_>>>()?;
            return match method {
                TypedArrayInstanceMethod::Set => {
                    if evaluated_args.len() < 2 {
                        return Err(Error::ScriptRuntime(
                            "Map.set requires exactly two arguments".into(),
                        ));
                    }
                    self.map_set_entry(
                        &mut map.borrow_mut(),
                        evaluated_args[0].clone(),
                        evaluated_args[1].clone(),
                    );
                    Ok(Value::Map(map.clone()))
                }
                TypedArrayInstanceMethod::Entries => {
                    Ok(Self::new_array_value(self.map_entries_array(map)))
                }
                TypedArrayInstanceMethod::Keys => Ok(Self::new_array_value(
                    map.borrow()
                        .entries
                        .iter()
                        .map(|(key, _)| key.clone())
                        .collect(),
                )),
                TypedArrayInstanceMethod::Values => Ok(Self::new_array_value(
                    map.borrow()
                        .entries
                        .iter()
                        .map(|(_, value)| value.clone())
                        .collect(),
                )),
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a TypedArray",
                    target
                ))),
            };
        }

        if let Value::WeakMap(weak_map) = target_value {
            let evaluated_args = args
                .iter()
                .map(|arg| self.eval_expr(arg, env, event_param, event))
                .collect::<Result<Vec<_>>>()?;
            return match method {
                TypedArrayInstanceMethod::Set => {
                    if evaluated_args.len() < 2 {
                        return Err(Error::ScriptRuntime(
                            "WeakMap.set requires exactly two arguments".into(),
                        ));
                    }
                    Self::ensure_weak_map_key(&evaluated_args[0])?;
                    self.weak_map_set_entry(
                        &mut weak_map.borrow_mut(),
                        evaluated_args[0].clone(),
                        evaluated_args[1].clone(),
                    );
                    Ok(Value::WeakMap(weak_map.clone()))
                }
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a TypedArray",
                    target
                ))),
            };
        }

        if let Value::Set(set) = target_value {
            let evaluated_args = args
                .iter()
                .map(|arg| self.eval_expr(arg, env, event_param, event))
                .collect::<Result<Vec<_>>>()?;
            return match method {
                TypedArrayInstanceMethod::Entries => {
                    let _ = evaluated_args;
                    Ok(Self::new_array_value(self.set_entries_array(set)))
                }
                TypedArrayInstanceMethod::Keys | TypedArrayInstanceMethod::Values => {
                    let _ = evaluated_args;
                    Ok(Self::new_array_value(self.set_values_array(set)))
                }
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a TypedArray",
                    target
                ))),
            };
        }

        if let Value::FormData(entries) = target_value {
            let evaluated_args = args
                .iter()
                .map(|arg| self.eval_expr(arg, env, event_param, event))
                .collect::<Result<Vec<_>>>()?;
            return match method {
                TypedArrayInstanceMethod::Set => {
                    if evaluated_args.len() < 2 {
                        return Err(Error::ScriptRuntime(
                            "FormData.set requires two or three arguments".into(),
                        ));
                    }
                    let name = evaluated_args[0].as_string();
                    let value = Self::form_data_append_string_value(
                        &evaluated_args[1],
                        evaluated_args.get(2),
                    );
                    let mut entries_ref = entries.borrow_mut();
                    if let Some(first_match) = entries_ref
                        .iter()
                        .position(|(entry_name, _)| entry_name == &name)
                    {
                        entries_ref[first_match].1 = value;
                        let mut index = entries_ref.len();
                        while index > 0 {
                            index -= 1;
                            if index != first_match && entries_ref[index].0 == name {
                                entries_ref.remove(index);
                            }
                        }
                    } else {
                        entries_ref.push((name, value));
                    }
                    Ok(Value::Undefined)
                }
                TypedArrayInstanceMethod::Entries => {
                    let snapshot = entries.borrow().clone();
                    Ok(Self::new_array_value(
                        snapshot
                            .into_iter()
                            .map(|(name, value)| {
                                Self::new_array_value(vec![
                                    Value::String(name),
                                    Value::String(value),
                                ])
                            })
                            .collect::<Vec<_>>(),
                    ))
                }
                TypedArrayInstanceMethod::Keys => {
                    let snapshot = entries.borrow().clone();
                    Ok(Self::new_array_value(
                        snapshot
                            .into_iter()
                            .map(|(name, _)| Value::String(name))
                            .collect::<Vec<_>>(),
                    ))
                }
                TypedArrayInstanceMethod::Values => {
                    let snapshot = entries.borrow().clone();
                    Ok(Self::new_array_value(
                        snapshot
                            .into_iter()
                            .map(|(_, value)| Value::String(value))
                            .collect::<Vec<_>>(),
                    ))
                }
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a TypedArray",
                    target
                ))),
            };
        }

        if let Value::Object(entries) = target_value {
            if Self::is_cookie_store_object(&entries.borrow()) {
                if matches!(method, TypedArrayInstanceMethod::Set) {
                    let mut evaluated_args = Vec::with_capacity(args.len());
                    for arg in args {
                        evaluated_args.push(self.eval_expr(arg, env, event_param, event)?);
                    }
                    if let Some(value) =
                        self.eval_cookie_store_member_call(entries, "set", &evaluated_args)?
                    {
                        return Ok(value);
                    }
                }
            }
            if Self::is_url_search_params_object(&entries.borrow()) {
                let evaluated_args = args
                    .iter()
                    .map(|arg| self.eval_expr(arg, env, event_param, event))
                    .collect::<Result<Vec<_>>>()?;
                return match method {
                    TypedArrayInstanceMethod::Set => {
                        if evaluated_args.len() < 2 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.set requires exactly two arguments".into(),
                            ));
                        }
                        let name = evaluated_args[0].as_string();
                        let value = evaluated_args[1].as_string();
                        {
                            let mut object_ref = entries.borrow_mut();
                            let mut pairs =
                                Self::url_search_params_pairs_from_object_entries(&object_ref);
                            if let Some(first_match) =
                                pairs.iter().position(|(entry_name, _)| entry_name == &name)
                            {
                                pairs[first_match].1 = value;
                                let mut index = pairs.len();
                                while index > 0 {
                                    index -= 1;
                                    if index != first_match && pairs[index].0 == name {
                                        pairs.remove(index);
                                    }
                                }
                            } else {
                                pairs.push((name, value));
                            }
                            Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                        }
                        self.sync_url_search_params_owner(entries);
                        Ok(Value::Undefined)
                    }
                    TypedArrayInstanceMethod::Entries => {
                        let pairs =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        Ok(Self::new_array_value(
                            pairs
                                .into_iter()
                                .map(|(name, value)| {
                                    Self::new_array_value(vec![
                                        Value::String(name),
                                        Value::String(value),
                                    ])
                                })
                                .collect::<Vec<_>>(),
                        ))
                    }
                    TypedArrayInstanceMethod::Keys => {
                        let pairs =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        Ok(Self::new_array_value(
                            pairs
                                .into_iter()
                                .map(|(name, _)| Value::String(name))
                                .collect::<Vec<_>>(),
                        ))
                    }
                    TypedArrayInstanceMethod::Values => {
                        let pairs =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        Ok(Self::new_array_value(
                            pairs
                                .into_iter()
                                .map(|(_, value)| Value::String(value))
                                .collect::<Vec<_>>(),
                        ))
                    }
                    TypedArrayInstanceMethod::Sort => {
                        {
                            let mut object_ref = entries.borrow_mut();
                            let mut pairs =
                                Self::url_search_params_pairs_from_object_entries(&object_ref);
                            pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                            Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                        }
                        self.sync_url_search_params_owner(entries);
                        Ok(Value::Undefined)
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a TypedArray",
                        target
                    ))),
                };
            }

            if Self::is_canvas_2d_context_object(&entries.borrow()) {
                let canvas_member = match method {
                    TypedArrayInstanceMethod::Fill => Some("fill"),
                    _ => None,
                };
                if let Some(canvas_member) = canvas_member {
                    let mut evaluated_args = Vec::with_capacity(args.len());
                    for arg in args {
                        evaluated_args.push(self.eval_expr(arg, env, event_param, event)?);
                    }
                    if let Some(value) = self.eval_canvas_2d_context_member_call(
                        entries,
                        canvas_member,
                        &evaluated_args,
                    )? {
                        return Ok(value);
                    }
                }
            }
        }

        if matches!(method, TypedArrayInstanceMethod::At) {
            if args.len() > 1 {
                return Err(Error::ScriptRuntime(
                    "at supports zero or one argument".into(),
                ));
            }
            let index = if let Some(index) = args.first() {
                Self::value_to_i64(&self.eval_expr(index, env, event_param, event)?)
            } else {
                0
            };

            return match target_value {
                Value::String(value) => {
                    let len = value.chars().count() as i64;
                    let index = if index < 0 { len + index } else { index };
                    if index < 0 || index >= len {
                        Ok(Value::Undefined)
                    } else {
                        Ok(value
                            .chars()
                            .nth(index as usize)
                            .map(|ch| Value::String(ch.to_string()))
                            .unwrap_or(Value::Undefined))
                    }
                }
                Value::Object(entries) => {
                    let entries = entries.borrow();
                    if let Some(value) = Self::string_wrapper_value_from_object(&entries) {
                        let len = value.chars().count() as i64;
                        let index = if index < 0 { len + index } else { index };
                        if index < 0 || index >= len {
                            Ok(Value::Undefined)
                        } else {
                            Ok(value
                                .chars()
                                .nth(index as usize)
                                .map(|ch| Value::String(ch.to_string()))
                                .unwrap_or(Value::Undefined))
                        }
                    } else {
                        Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not a TypedArray",
                            target
                        )))
                    }
                }
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a TypedArray",
                    target
                ))),
            };
        }

        if matches!(
            method,
            TypedArrayInstanceMethod::IndexOf | TypedArrayInstanceMethod::LastIndexOf
        ) {
            if args.is_empty() || args.len() > 2 {
                return Err(Error::ScriptRuntime(
                    "indexOf requires one or two arguments".into(),
                ));
            }
            let search = self.eval_expr(&args[0], env, event_param, event)?;

            return match target_value {
                Value::String(value) => {
                    let len = value.chars().count() as i64;
                    if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                        let mut start = if args.len() == 2 {
                            Self::value_to_i64(&self.eval_expr(
                                &args[1],
                                env,
                                event_param,
                                event,
                            )?)
                        } else {
                            0
                        };
                        if start < 0 {
                            start = 0;
                        }
                        if start > len {
                            start = len;
                        }
                        let search = self.coerce_to_string_for_tostring(&search)?;
                        let index = Self::string_index_of(value, &search, start as usize)
                            .map(|idx| idx as i64)
                            .unwrap_or(-1);
                        Ok(Value::Number(index))
                    } else {
                        let mut from = if args.len() == 2 {
                            Self::value_to_i64(&self.eval_expr(
                                &args[1],
                                env,
                                event_param,
                                event,
                            )?)
                        } else {
                            len
                        };
                        if from < 0 {
                            from = 0;
                        }
                        if from > len {
                            from = len;
                        }
                        let from = from as usize;
                        let search = self.coerce_to_string_for_tostring(&search)?;
                        if search.is_empty() {
                            return Ok(Value::Number(from as i64));
                        }
                        for idx in (0..=from).rev() {
                            let byte_idx = Self::char_index_to_byte(value, idx);
                            if value[byte_idx..].starts_with(&search) {
                                return Ok(Value::Number(idx as i64));
                            }
                        }
                        Ok(Value::Number(-1))
                    }
                }
                Value::Array(values) => {
                    let from = if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                        let len = values.borrow().len() as i64;
                        let mut from = if args.len() == 2 {
                            Self::value_to_i64(&self.eval_expr(
                                &args[1],
                                env,
                                event_param,
                                event,
                            )?)
                        } else {
                            0
                        };
                        if from < 0 {
                            from = (len + from).max(0);
                        }
                        if from > len {
                            from = len;
                        }
                        from
                    } else {
                        let len = values.borrow().len() as i64;
                        let from = if args.len() == 2 {
                            Self::value_to_i64(&self.eval_expr(
                                &args[1],
                                env,
                                event_param,
                                event,
                            )?)
                        } else {
                            len - 1
                        };
                        if from < 0 {
                            (len + from).max(-1)
                        } else {
                            from.min(len - 1)
                        }
                    };

                    let values = values.borrow();
                    if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                        for (index, value) in values.iter().enumerate().skip(from as usize) {
                            if self.strict_equal(value, &search) {
                                return Ok(Value::Number(index as i64));
                            }
                        }
                        Ok(Value::Number(-1))
                    } else {
                        if from < 0 {
                            return Ok(Value::Number(-1));
                        }
                        for index in (0..=from as usize).rev() {
                            if self.strict_equal(&values[index], &search) {
                                return Ok(Value::Number(index as i64));
                            }
                        }
                        Ok(Value::Number(-1))
                    }
                }
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a TypedArray",
                    target
                ))),
            };
        }

        Err(Error::ScriptRuntime(format!(
            "variable '{}' is not a TypedArray",
            target
        )))
    }
}
