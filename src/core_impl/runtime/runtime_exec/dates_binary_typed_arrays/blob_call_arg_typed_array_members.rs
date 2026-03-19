use super::*;

impl Harness {
    pub(crate) fn eval_typed_array_member_call(
        &mut self,
        array: &Rc<RefCell<TypedArrayValue>>,
        member: &str,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Option<Value>> {
        match member {
            "at" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.at requires exactly one argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let len = array.borrow().observed_length() as i64;
                let mut index = Self::value_to_i64(&args[0]);
                if index < 0 {
                    index += len;
                }
                if index < 0 || index >= len {
                    return Ok(Some(Value::Undefined));
                }
                Ok(Some(self.typed_array_get_index(array, index as usize)?))
            }
            "copyWithin" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.copyWithin requires 2 or 3 arguments".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let len = array.borrow().observed_length();
                let target_index = Self::normalize_slice_index(len, Self::value_to_i64(&args[0]));
                let start_index = Self::normalize_slice_index(len, Self::value_to_i64(&args[1]));
                let end_index = args
                    .get(2)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(len)
                    .max(start_index);
                let count = end_index
                    .saturating_sub(start_index)
                    .min(len.saturating_sub(target_index));
                let snapshot = self.typed_array_snapshot(array)?;
                for offset in 0..count {
                    self.typed_array_set_index(
                        array,
                        target_index + offset,
                        snapshot[start_index + offset].clone(),
                    )?;
                }
                Ok(Some(Value::TypedArray(array.clone())))
            }
            "join" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.join supports at most one argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let separator = if let Some(first) = args.first() {
                    if matches!(first, Value::Undefined) {
                        ",".to_string()
                    } else {
                        first.as_string()
                    }
                } else {
                    ",".to_string()
                };
                let joined = self
                    .typed_array_snapshot(array)?
                    .into_iter()
                    .map(|value| value.as_string())
                    .collect::<Vec<_>>()
                    .join(&separator);
                Ok(Some(Value::String(joined)))
            }
            "forEach" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.forEach requires exactly one callback argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let callback = args[0].clone();
                let snapshot = self.typed_array_snapshot(array)?;
                for (index, value) in snapshot.into_iter().enumerate() {
                    let _ = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            value,
                            Value::Number(index as i64),
                            Value::TypedArray(array.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                }
                Ok(Some(Value::Undefined))
            }
            "map" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.map requires exactly one callback argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let callback = args[0].clone();
                let kind = array.borrow().kind;
                let snapshot = self.typed_array_snapshot(array)?;
                let mut out = Vec::with_capacity(snapshot.len());
                for (index, value) in snapshot.into_iter().enumerate() {
                    out.push(self.execute_callback_value_with_env(
                        &callback,
                        &[
                            value,
                            Value::Number(index as i64),
                            Value::TypedArray(array.clone()),
                        ],
                        event,
                        caller_env,
                    )?);
                }
                Ok(Some(self.new_typed_array_from_values(kind, &out)?))
            }
            "filter" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.filter requires exactly one callback argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let callback = args[0].clone();
                let kind = array.borrow().kind;
                let snapshot = self.typed_array_snapshot(array)?;
                let mut out = Vec::new();
                for (index, value) in snapshot.into_iter().enumerate() {
                    let keep = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            value.clone(),
                            Value::Number(index as i64),
                            Value::TypedArray(array.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if keep.truthy() {
                        out.push(value);
                    }
                }
                Ok(Some(self.new_typed_array_from_values(kind, &out)?))
            }
            "reduce" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.reduce requires callback and optional initial value".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let callback = args[0].clone();
                let snapshot = self.typed_array_snapshot(array)?;
                let mut iter = snapshot.into_iter().enumerate();
                let mut acc = if let Some(initial) = args.get(1) {
                    initial.clone()
                } else {
                    let Some((_, first)) = iter.next() else {
                        return Err(Error::ScriptRuntime(
                            "reduce of empty array with no initial value".into(),
                        ));
                    };
                    first
                };
                for (index, value) in iter {
                    acc = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            acc,
                            value,
                            Value::Number(index as i64),
                            Value::TypedArray(array.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                }
                Ok(Some(acc))
            }
            "reduceRight" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.reduceRight requires callback and optional initial value"
                            .into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let callback = args[0].clone();
                let snapshot = self.typed_array_snapshot(array)?;
                let mut iter = snapshot.into_iter().enumerate().rev();
                let mut acc = if let Some(initial) = args.get(1) {
                    initial.clone()
                } else {
                    let Some((_, first)) = iter.next() else {
                        return Err(Error::ScriptRuntime(
                            "reduce of empty array with no initial value".into(),
                        ));
                    };
                    first
                };
                for (index, value) in iter {
                    acc = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            acc,
                            value,
                            Value::Number(index as i64),
                            Value::TypedArray(array.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                }
                Ok(Some(acc))
            }
            "find" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.find requires exactly one callback argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let callback = args[0].clone();
                let snapshot = self.typed_array_snapshot(array)?;
                for (index, value) in snapshot.into_iter().enumerate() {
                    let matched = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            value.clone(),
                            Value::Number(index as i64),
                            Value::TypedArray(array.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if matched.truthy() {
                        return Ok(Some(value));
                    }
                }
                Ok(Some(Value::Undefined))
            }
            "findIndex" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.findIndex requires exactly one callback argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let callback = args[0].clone();
                let snapshot = self.typed_array_snapshot(array)?;
                for (index, value) in snapshot.into_iter().enumerate() {
                    let matched = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            value,
                            Value::Number(index as i64),
                            Value::TypedArray(array.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if matched.truthy() {
                        return Ok(Some(Value::Number(index as i64)));
                    }
                }
                Ok(Some(Value::Number(-1)))
            }
            "findLast" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.findLast requires exactly one callback argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let callback = args[0].clone();
                let snapshot = self.typed_array_snapshot(array)?;
                for (index, value) in snapshot.into_iter().enumerate().rev() {
                    let matched = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            value.clone(),
                            Value::Number(index as i64),
                            Value::TypedArray(array.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if matched.truthy() {
                        return Ok(Some(value));
                    }
                }
                Ok(Some(Value::Undefined))
            }
            "findLastIndex" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.findLastIndex requires exactly one callback argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let callback = args[0].clone();
                let snapshot = self.typed_array_snapshot(array)?;
                for (index, value) in snapshot.into_iter().enumerate().rev() {
                    let matched = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            value,
                            Value::Number(index as i64),
                            Value::TypedArray(array.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if matched.truthy() {
                        return Ok(Some(Value::Number(index as i64)));
                    }
                }
                Ok(Some(Value::Number(-1)))
            }
            "indexOf" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray indexOf methods require one or two arguments".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let snapshot = self.typed_array_snapshot(array)?;
                let len = snapshot.len() as i64;
                let from = args.get(1).map(Self::value_to_i64).unwrap_or(0);
                let mut from = if from < 0 { (len + from).max(0) } else { from };
                if from > len {
                    from = len;
                }
                for (index, value) in snapshot.iter().enumerate().skip(from as usize) {
                    if self.strict_equal(value, &args[0]) {
                        return Ok(Some(Value::Number(index as i64)));
                    }
                }
                Ok(Some(Value::Number(-1)))
            }
            "lastIndexOf" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray indexOf methods require one or two arguments".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let snapshot = self.typed_array_snapshot(array)?;
                let len = snapshot.len() as i64;
                let from = args.get(1).map(Self::value_to_i64).unwrap_or(len - 1);
                let from = if from < 0 {
                    (len + from).max(-1)
                } else {
                    from.min(len - 1)
                };
                if from < 0 {
                    return Ok(Some(Value::Number(-1)));
                }
                for index in (0..=from as usize).rev() {
                    if self.strict_equal(&snapshot[index], &args[0]) {
                        return Ok(Some(Value::Number(index as i64)));
                    }
                }
                Ok(Some(Value::Number(-1)))
            }
            "slice" => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.slice supports at most two arguments".into(),
                    ));
                }
                let snapshot = self.typed_array_snapshot(array)?;
                let kind = array.borrow().kind;
                let len = snapshot.len();
                let start = args
                    .first()
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(0);
                let end = args
                    .get(1)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(len)
                    .max(start);
                Ok(Some(self.new_typed_array_from_values(
                    kind,
                    &snapshot[start..end],
                )?))
            }
            "subarray" => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.subarray supports at most two arguments".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let array_ref = array.borrow();
                let kind = array_ref.kind;
                let len = array_ref.observed_length();
                let begin = args.first().map(Self::value_to_i64).unwrap_or(0);
                let end = args.get(1).map(Self::value_to_i64).unwrap_or(len as i64);
                let begin = Self::normalize_slice_index(len, begin);
                let end = Self::normalize_slice_index(len, end).max(begin);
                let byte_offset = array_ref
                    .byte_offset
                    .saturating_add(begin.saturating_mul(kind.bytes_per_element()));
                let buffer = array_ref.buffer.clone();
                drop(array_ref);
                Ok(Some(self.new_typed_array_view(
                    kind,
                    buffer,
                    byte_offset,
                    Some(end.saturating_sub(begin)),
                )?))
            }
            "with" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.with requires exactly two arguments".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let len = array.borrow().observed_length() as i64;
                let mut index = Self::value_to_i64(&args[0]);
                if index < 0 {
                    index += len;
                }
                if index < 0 || index >= len {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.with index out of range".into(),
                    ));
                }
                let kind = array.borrow().kind;
                let mut snapshot = self.typed_array_snapshot(array)?;
                snapshot[index as usize] = args[1].clone();
                Ok(Some(self.new_typed_array_from_values(kind, &snapshot)?))
            }
            "entries" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.entries does not take arguments".into(),
                    ));
                }
                let snapshot = self.typed_array_snapshot(array)?;
                Ok(Some(
                    self.new_iterator_value(
                        snapshot
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| {
                                Self::new_array_value(vec![Value::Number(index as i64), value])
                            })
                            .collect(),
                    ),
                ))
            }
            "keys" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.keys does not take arguments".into(),
                    ));
                }
                let len = self.typed_array_snapshot(array)?.len();
                Ok(Some(self.new_iterator_value(
                    (0..len).map(|index| Value::Number(index as i64)).collect(),
                )))
            }
            "some" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.some requires exactly one callback argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let callback = args[0].clone();
                let snapshot = self.typed_array_snapshot(array)?;
                for (index, value) in snapshot.into_iter().enumerate() {
                    let matched = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            value,
                            Value::Number(index as i64),
                            Value::TypedArray(array.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if matched.truthy() {
                        return Ok(Some(Value::Bool(true)));
                    }
                }
                Ok(Some(Value::Bool(false)))
            }
            "every" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.every requires exactly one callback argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let callback = args[0].clone();
                let snapshot = self.typed_array_snapshot(array)?;
                for (index, value) in snapshot.into_iter().enumerate() {
                    let matched = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            value,
                            Value::Number(index as i64),
                            Value::TypedArray(array.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if !matched.truthy() {
                        return Ok(Some(Value::Bool(false)));
                    }
                }
                Ok(Some(Value::Bool(true)))
            }
            "values" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.values does not take arguments".into(),
                    ));
                }
                Ok(Some(
                    self.new_iterator_value(self.typed_array_snapshot(array)?),
                ))
            }
            _ => Ok(None),
        }
    }
}
