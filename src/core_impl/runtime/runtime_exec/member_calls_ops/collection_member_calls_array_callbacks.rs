use super::*;

impl Harness {
    fn array_flat_depth_arg(value: Option<&Value>) -> usize {
        let Some(value) = value else {
            return 1;
        };
        let depth = Self::coerce_number_for_global(value);
        if depth.is_nan() || depth <= 0.0 {
            0
        } else if !depth.is_finite() {
            usize::MAX
        } else {
            depth.floor().min(usize::MAX as f64) as usize
        }
    }

    fn flatten_array_value_into(out: &mut Vec<Value>, value: Value, depth: usize) {
        match value {
            Value::Array(values) if depth > 0 => {
                let snapshot = {
                    let values = values.borrow();
                    ArrayValue {
                        elements: values.elements.clone(),
                        properties: values.properties.clone(),
                    }
                };
                for index in 0..snapshot.len() {
                    if Self::array_index_is_hole(&snapshot, index) {
                        continue;
                    }
                    Self::flatten_array_value_into(
                        out,
                        snapshot[index].clone(),
                        depth.saturating_sub(1),
                    );
                }
            }
            other => out.push(other),
        }
    }

    pub(crate) fn try_eval_array_member_call_callbacks(
        &mut self,
        values: &Rc<RefCell<ArrayValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Option<Value>> {
        let value = match member {
            "forEach" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "forEach requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let _ = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                }
                Value::Undefined
            }
            "map" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "map requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut out = Vec::with_capacity(snapshot.len());
                for (idx, item) in snapshot.into_iter().enumerate() {
                    out.push(self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
                    )?);
                }
                Self::new_array_value(out)
            }
            "flat" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "flat supports zero or one argument".into(),
                    ));
                }
                let snapshot = {
                    let values = values.borrow();
                    ArrayValue {
                        elements: values.elements.clone(),
                        properties: values.properties.clone(),
                    }
                };
                let depth = Self::array_flat_depth_arg(evaluated_args.first());
                let mut out = Vec::new();
                for index in 0..snapshot.len() {
                    if Self::array_index_is_hole(&snapshot, index) {
                        continue;
                    }
                    Self::flatten_array_value_into(&mut out, snapshot[index].clone(), depth);
                }
                Self::new_array_value(out)
            }
            "flatMap" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "flatMap requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut out = Vec::new();
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let mapped = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    match mapped {
                        Value::Array(mapped_values) => {
                            let mapped_values = mapped_values.borrow();
                            for index in 0..mapped_values.len() {
                                if Self::array_index_is_hole(&mapped_values, index) {
                                    continue;
                                }
                                out.push(mapped_values[index].clone());
                            }
                        }
                        other => out.push(other),
                    }
                }
                Self::new_array_value(out)
            }
            "filter" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "filter requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut out = Vec::new();
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let keep = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item.clone(),
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if keep.truthy() {
                        out.push(item);
                    }
                }
                Self::new_array_value(out)
            }
            "reduce" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "reduce requires callback and optional initial value".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut start_index = 0usize;
                let mut acc = if let Some(initial) = evaluated_args.get(1) {
                    initial.clone()
                } else {
                    let Some(first) = snapshot.first().cloned() else {
                        return Err(Error::ScriptRuntime(
                            "reduce of empty array with no initial value".into(),
                        ));
                    };
                    start_index = 1;
                    first
                };
                for (idx, item) in snapshot.into_iter().enumerate().skip(start_index) {
                    acc = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            acc,
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                }
                acc
            }
            "find" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "find requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut found = Value::Undefined;
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let matched = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item.clone(),
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if matched.truthy() {
                        found = item;
                        break;
                    }
                }
                found
            }
            "findIndex" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "findIndex requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut found = -1i64;
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let matched = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if matched.truthy() {
                        found = idx as i64;
                        break;
                    }
                }
                Value::Number(found)
            }
            "some" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "some requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut matched = false;
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let keep = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if keep.truthy() {
                        matched = true;
                        break;
                    }
                }
                Value::Bool(matched)
            }
            "every" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "every requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut all = true;
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let keep = self.execute_callback_value_with_env(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                        caller_env,
                    )?;
                    if !keep.truthy() {
                        all = false;
                        break;
                    }
                }
                Value::Bool(all)
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
