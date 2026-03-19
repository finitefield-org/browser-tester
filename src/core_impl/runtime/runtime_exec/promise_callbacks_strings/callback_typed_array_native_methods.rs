use super::*;

impl Harness {
    pub(crate) fn eval_typed_array_native_method(
        &mut self,
        array: Rc<RefCell<TypedArrayValue>>,
        kind: TypedArrayKind,
        len: usize,
        this_value: Value,
        method: TypedArrayInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
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
                Ok(self.new_iterator_value(
                    self.typed_array_snapshot(&array)?
                        .into_iter()
                        .enumerate()
                        .map(|(index, value)| {
                            Self::new_array_value(vec![Value::Number(index as i64), value])
                        })
                        .collect(),
                ))
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
                Ok(self.new_iterator_value(
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
                Ok(self.new_iterator_value(self.typed_array_snapshot(&array)?))
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
