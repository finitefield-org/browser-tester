use super::*;

impl Harness {
    pub(crate) fn execute_callback_value(
        &mut self,
        callback: &Value,
        args: &[Value],
        event: &EventState,
    ) -> Result<Value> {
        self.execute_callable_value(callback, args, event)
    }

    pub(crate) fn eval_typed_array_method(
        &mut self,
        target: &str,
        method: TypedArrayInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !matches!(env.get(target), Some(Value::TypedArray(_))) {
            let Some(target_value) = env.get(target) else {
                return Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                )));
            };

            if let Value::Map(map) = target_value {
                return match method {
                    TypedArrayInstanceMethod::Set => {
                        if args.len() != 2 {
                            return Err(Error::ScriptRuntime(
                                "Map.set requires exactly two arguments".into(),
                            ));
                        }
                        let key = self.eval_expr(&args[0], env, event_param, event)?;
                        let value = self.eval_expr(&args[1], env, event_param, event)?;
                        self.map_set_entry(&mut map.borrow_mut(), key, value);
                        Ok(Value::Map(map.clone()))
                    }
                    TypedArrayInstanceMethod::Entries => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Map.entries does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(self.map_entries_array(map)))
                    }
                    TypedArrayInstanceMethod::Keys => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Map.keys does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(
                            map.borrow()
                                .entries
                                .iter()
                                .map(|(key, _)| key.clone())
                                .collect(),
                        ))
                    }
                    TypedArrayInstanceMethod::Values => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Map.values does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(
                            map.borrow()
                                .entries
                                .iter()
                                .map(|(_, value)| value.clone())
                                .collect(),
                        ))
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a TypedArray",
                        target
                    ))),
                };
            }

            if let Value::Set(set) = target_value {
                return match method {
                    TypedArrayInstanceMethod::Entries => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Set.entries does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(self.set_entries_array(set)))
                    }
                    TypedArrayInstanceMethod::Keys | TypedArrayInstanceMethod::Values => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Set.keys/values does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(self.set_values_array(set)))
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a TypedArray",
                        target
                    ))),
                };
            }

            if let Value::Object(entries) = target_value {
                if Self::is_url_search_params_object(&entries.borrow()) {
                    return match method {
                        TypedArrayInstanceMethod::Set => {
                            if args.len() != 2 {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.set requires exactly two arguments".into(),
                                ));
                            }
                            let name = self
                                .eval_expr(&args[0], env, event_param, event)?
                                .as_string();
                            let value = self
                                .eval_expr(&args[1], env, event_param, event)?
                                .as_string();
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
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.entries does not take arguments".into(),
                                ));
                            }
                            let pairs = Self::url_search_params_pairs_from_object_entries(
                                &entries.borrow(),
                            );
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
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.keys does not take arguments".into(),
                                ));
                            }
                            let pairs = Self::url_search_params_pairs_from_object_entries(
                                &entries.borrow(),
                            );
                            Ok(Self::new_array_value(
                                pairs
                                    .into_iter()
                                    .map(|(name, _)| Value::String(name))
                                    .collect::<Vec<_>>(),
                            ))
                        }
                        TypedArrayInstanceMethod::Values => {
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.values does not take arguments".into(),
                                ));
                            }
                            let pairs = Self::url_search_params_pairs_from_object_entries(
                                &entries.borrow(),
                            );
                            Ok(Self::new_array_value(
                                pairs
                                    .into_iter()
                                    .map(|(_, value)| Value::String(value))
                                    .collect::<Vec<_>>(),
                            ))
                        }
                        TypedArrayInstanceMethod::Sort => {
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.sort does not take arguments".into(),
                                ));
                            }
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
                            let index =
                                Self::string_index_of(value, &search.as_string(), start as usize)
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
                            let search = search.as_string();
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

            return Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a TypedArray",
                target
            )));
        }

        let array = self.resolve_typed_array_from_env(env, target)?;
        if array.borrow().buffer.borrow().detached {
            return Err(Error::ScriptRuntime(
                "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
            ));
        }
        let kind = array.borrow().kind;
        let len = array.borrow().observed_length();
        let this_value = Value::TypedArray(array.clone());

        match method {
            TypedArrayInstanceMethod::At => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.at requires exactly one argument".into(),
                    ));
                }
                let index = self.eval_expr(&args[0], env, event_param, event)?;
                let mut index = Self::value_to_i64(&index);
                let len_i64 = len as i64;
                if index < 0 {
                    index += len_i64;
                }
                if index < 0 || index >= len_i64 {
                    return Ok(Value::Undefined);
                }
                self.typed_array_get_index(&array, index as usize)
            }
            TypedArrayInstanceMethod::CopyWithin => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.copyWithin requires 2 or 3 arguments".into(),
                    ));
                }
                let target_index =
                    Self::value_to_i64(&self.eval_expr(&args[0], env, event_param, event)?);
                let start_index =
                    Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?);
                let end_index = if args.len() == 3 {
                    Self::value_to_i64(&self.eval_expr(&args[2], env, event_param, event)?)
                } else {
                    len as i64
                };
                let target_index = Self::normalize_slice_index(len, target_index);
                let start_index = Self::normalize_slice_index(len, start_index);
                let end_index = Self::normalize_slice_index(len, end_index);
                let end_index = end_index.max(start_index);
                let count = end_index
                    .saturating_sub(start_index)
                    .min(len.saturating_sub(target_index));
                let snapshot = self.typed_array_snapshot(&array)?;
                for offset in 0..count {
                    self.typed_array_set_index(
                        &array,
                        target_index + offset,
                        snapshot[start_index + offset].clone(),
                    )?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::Entries => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.entries does not take arguments".into(),
                    ));
                }
                let snapshot = self.typed_array_snapshot(&array)?;
                let out = snapshot
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| {
                        Self::new_array_value(vec![Value::Number(index as i64), value])
                    })
                    .collect::<Vec<_>>();
                Ok(Self::new_array_value(out))
            }
            TypedArrayInstanceMethod::Fill => {
                if args.is_empty() || args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.fill requires 1 to 3 arguments".into(),
                    ));
                }
                let value = self.eval_expr(&args[0], env, event_param, event)?;
                let start = if args.len() >= 2 {
                    Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                } else {
                    0
                };
                let end = if args.len() == 3 {
                    Self::value_to_i64(&self.eval_expr(&args[2], env, event_param, event)?)
                } else {
                    len as i64
                };
                let start = Self::normalize_slice_index(len, start);
                let end = Self::normalize_slice_index(len, end).max(start);
                for index in start..end {
                    self.typed_array_set_index(&array, index, value.clone())?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::FindIndex
            | TypedArrayInstanceMethod::FindLast
            | TypedArrayInstanceMethod::FindLastIndex => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray find callback methods require exactly one argument".into(),
                    ));
                }
                let callback = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = self.typed_array_snapshot(&array)?;
                let iter: Box<dyn Iterator<Item = (usize, Value)>> = match method {
                    TypedArrayInstanceMethod::FindLast
                    | TypedArrayInstanceMethod::FindLastIndex => {
                        Box::new(snapshot.into_iter().enumerate().rev())
                    }
                    _ => Box::new(snapshot.into_iter().enumerate()),
                };
                for (index, value) in iter {
                    let matched = self.execute_callback_value(
                        &callback,
                        &[
                            value.clone(),
                            Value::Number(index as i64),
                            this_value.clone(),
                        ],
                        event,
                    )?;
                    if matched.truthy() {
                        return if matches!(
                            method,
                            TypedArrayInstanceMethod::FindLastIndex
                                | TypedArrayInstanceMethod::FindIndex
                        ) {
                            Ok(Value::Number(index as i64))
                        } else {
                            Ok(value)
                        };
                    }
                }
                if matches!(
                    method,
                    TypedArrayInstanceMethod::FindLastIndex | TypedArrayInstanceMethod::FindIndex
                ) {
                    Ok(Value::Number(-1))
                } else {
                    Ok(Value::Undefined)
                }
            }
            TypedArrayInstanceMethod::IndexOf | TypedArrayInstanceMethod::LastIndexOf => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray indexOf methods require one or two arguments".into(),
                    ));
                }
                let search = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = self.typed_array_snapshot(&array)?;
                if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                    let from = if args.len() == 2 {
                        Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                    } else {
                        0
                    };
                    let mut from = if from < 0 {
                        (len as i64 + from).max(0)
                    } else {
                        from
                    };
                    if from > len as i64 {
                        from = len as i64;
                    }
                    for (index, value) in snapshot.iter().enumerate().skip(from as usize) {
                        if self.strict_equal(value, &search) {
                            return Ok(Value::Number(index as i64));
                        }
                    }
                    Ok(Value::Number(-1))
                } else {
                    let from = if args.len() == 2 {
                        Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                    } else {
                        (len as i64) - 1
                    };
                    let from = if from < 0 {
                        (len as i64 + from).max(-1)
                    } else {
                        from.min((len as i64) - 1)
                    };
                    if from < 0 {
                        return Ok(Value::Number(-1));
                    }
                    for index in (0..=from as usize).rev() {
                        if self.strict_equal(&snapshot[index], &search) {
                            return Ok(Value::Number(index as i64));
                        }
                    }
                    Ok(Value::Number(-1))
                }
            }
            TypedArrayInstanceMethod::Keys => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.keys does not take arguments".into(),
                    ));
                }
                Ok(Self::new_array_value(
                    (0..len).map(|index| Value::Number(index as i64)).collect(),
                ))
            }
            TypedArrayInstanceMethod::ReduceRight => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.reduceRight requires callback and optional initial value"
                            .into(),
                    ));
                }
                let callback = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = self.typed_array_snapshot(&array)?;
                let mut iter = snapshot.into_iter().enumerate().rev();
                let mut acc = if args.len() == 2 {
                    self.eval_expr(&args[1], env, event_param, event)?
                } else {
                    let Some((_, first)) = iter.next() else {
                        return Err(Error::ScriptRuntime(
                            "reduce of empty array with no initial value".into(),
                        ));
                    };
                    first
                };
                for (index, value) in iter {
                    acc = self.execute_callback_value(
                        &callback,
                        &[acc, value, Value::Number(index as i64), this_value.clone()],
                        event,
                    )?;
                }
                Ok(acc)
            }
            TypedArrayInstanceMethod::Reverse => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.reverse does not take arguments".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                snapshot.reverse();
                for (index, value) in snapshot.into_iter().enumerate() {
                    self.typed_array_set_index(&array, index, value)?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::Set => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.set requires source and optional offset".into(),
                    ));
                }
                let source = self.eval_expr(&args[0], env, event_param, event)?;
                let source_values = self.array_like_values_from_value(&source)?;
                let offset = if args.len() == 2 {
                    Self::to_non_negative_usize(
                        &self.eval_expr(&args[1], env, event_param, event)?,
                        "TypedArray.set offset",
                    )?
                } else {
                    0
                };
                if offset > len || source_values.len() > len.saturating_sub(offset) {
                    return Err(Error::ScriptRuntime(
                        "source array is too large for target TypedArray".into(),
                    ));
                }
                for (index, value) in source_values.into_iter().enumerate() {
                    self.typed_array_set_index(&array, offset + index, value)?;
                }
                Ok(Value::Undefined)
            }
            TypedArrayInstanceMethod::Sort => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.sort supports at most one argument".into(),
                    ));
                }
                if args.len() == 1 {
                    return Err(Error::ScriptRuntime(
                        "custom comparator for TypedArray.sort is not supported".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                if kind.is_bigint() {
                    snapshot.sort_by(|left, right| {
                        let left = match left {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        let right = match right {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        left.cmp(&right)
                    });
                } else {
                    snapshot.sort_by(|left, right| {
                        let left = Self::coerce_number_for_global(left);
                        let right = Self::coerce_number_for_global(right);
                        match (left.is_nan(), right.is_nan()) {
                            (true, true) => std::cmp::Ordering::Equal,
                            (true, false) => std::cmp::Ordering::Greater,
                            (false, true) => std::cmp::Ordering::Less,
                            (false, false) => left
                                .partial_cmp(&right)
                                .unwrap_or(std::cmp::Ordering::Equal),
                        }
                    });
                }
                for (index, value) in snapshot.into_iter().enumerate() {
                    self.typed_array_set_index(&array, index, value)?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::Subarray => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.subarray supports at most two arguments".into(),
                    ));
                }
                let begin = if !args.is_empty() {
                    Self::value_to_i64(&self.eval_expr(&args[0], env, event_param, event)?)
                } else {
                    0
                };
                let end = if args.len() == 2 {
                    Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                } else {
                    len as i64
                };
                let begin = Self::normalize_slice_index(len, begin);
                let end = Self::normalize_slice_index(len, end).max(begin);
                let byte_offset = array
                    .borrow()
                    .byte_offset
                    .saturating_add(begin.saturating_mul(kind.bytes_per_element()));
                self.new_typed_array_view(
                    kind,
                    array.borrow().buffer.clone(),
                    byte_offset,
                    Some(end.saturating_sub(begin)),
                )
            }
            TypedArrayInstanceMethod::ToReversed => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.toReversed does not take arguments".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                snapshot.reverse();
                self.new_typed_array_from_values(kind, &snapshot)
            }
            TypedArrayInstanceMethod::ToSorted => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.toSorted supports at most one argument".into(),
                    ));
                }
                if args.len() == 1 {
                    return Err(Error::ScriptRuntime(
                        "custom comparator for TypedArray.toSorted is not supported".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                if kind.is_bigint() {
                    snapshot.sort_by(|left, right| {
                        let left = match left {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        let right = match right {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        left.cmp(&right)
                    });
                } else {
                    snapshot.sort_by(|left, right| {
                        let left = Self::coerce_number_for_global(left);
                        let right = Self::coerce_number_for_global(right);
                        match (left.is_nan(), right.is_nan()) {
                            (true, true) => std::cmp::Ordering::Equal,
                            (true, false) => std::cmp::Ordering::Greater,
                            (false, true) => std::cmp::Ordering::Less,
                            (false, false) => left
                                .partial_cmp(&right)
                                .unwrap_or(std::cmp::Ordering::Equal),
                        }
                    });
                }
                self.new_typed_array_from_values(kind, &snapshot)
            }
            TypedArrayInstanceMethod::Values => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.values does not take arguments".into(),
                    ));
                }
                Ok(Self::new_array_value(self.typed_array_snapshot(&array)?))
            }
            TypedArrayInstanceMethod::With => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.with requires exactly two arguments".into(),
                    ));
                }
                let index =
                    Self::value_to_i64(&self.eval_expr(&args[0], env, event_param, event)?);
                let value = self.eval_expr(&args[1], env, event_param, event)?;
                let index = if index < 0 {
                    (len as i64) + index
                } else {
                    index
                };
                if index < 0 || index >= len as i64 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.with index out of range".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                snapshot[index as usize] = value;
                self.new_typed_array_from_values(kind, &snapshot)
            }
        }
    }
}
