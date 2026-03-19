use super::*;

impl Harness {
    pub(crate) fn try_eval_array_member_call_sequence(
        &mut self,
        values: &Rc<RefCell<ArrayValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let value = match member {
            "values" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "values does not take arguments".into(),
                    ));
                }
                self.new_iterator_value(values.borrow().clone())
            }
            "keys" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("keys does not take arguments".into()));
                }
                self.new_iterator_value(
                    (0..values.borrow().len())
                        .map(|index| Value::Number(index as i64))
                        .collect(),
                )
            }
            "entries" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "entries does not take arguments".into(),
                    ));
                }
                self.new_iterator_value(
                    values
                        .borrow()
                        .iter()
                        .enumerate()
                        .map(|(index, value)| {
                            Self::new_array_value(vec![Value::Number(index as i64), value.clone()])
                        })
                        .collect(),
                )
            }
            "fill" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "fill requires 1 to 3 arguments".into(),
                    ));
                }
                let fill_value = evaluated_args[0].clone();
                let mut values_ref = values.borrow_mut();
                let len = values_ref.len();
                let start = evaluated_args
                    .get(1)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(0);
                let end = evaluated_args
                    .get(2)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(len)
                    .max(start);
                for value in values_ref.iter_mut().take(end).skip(start) {
                    *value = fill_value.clone();
                }
                Value::Array(values.clone())
            }
            "includes" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "includes requires one or two arguments".into(),
                    ));
                }
                let search = evaluated_args[0].clone();
                let values_ref = values.borrow();
                let len = values_ref.len() as i64;
                let mut start = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                if start < 0 {
                    start = (len + start).max(0);
                }
                let start = start.min(len) as usize;
                let mut found = false;
                for value in values_ref.iter().skip(start) {
                    if self.strict_equal(value, &search) {
                        found = true;
                        break;
                    }
                }
                Value::Bool(found)
            }
            "indexOf" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "indexOf requires one or two arguments".into(),
                    ));
                }
                let search = evaluated_args[0].clone();
                let values_ref = values.borrow();
                let len = values_ref.len() as i64;
                let mut from = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                if from < 0 {
                    from = (len + from).max(0);
                } else {
                    from = from.min(len);
                }
                let mut found = -1i64;
                for index in from as usize..values_ref.len() {
                    if Self::array_index_is_hole(&values_ref, index) {
                        continue;
                    }
                    if self.strict_equal(&values_ref[index], &search) {
                        found = index as i64;
                        break;
                    }
                }
                Value::Number(found)
            }
            "lastIndexOf" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "lastIndexOf requires one or two arguments".into(),
                    ));
                }
                let search = evaluated_args[0].clone();
                let values_ref = values.borrow();
                let len = values_ref.len() as i64;
                let from = evaluated_args
                    .get(1)
                    .map(Self::value_to_i64)
                    .unwrap_or(len - 1);
                let from = if from < 0 {
                    (len + from).max(-1)
                } else {
                    from.min(len - 1)
                };
                if from < 0 {
                    Value::Number(-1)
                } else {
                    let mut found = -1i64;
                    for index in (0..=from as usize).rev() {
                        if Self::array_index_is_hole(&values_ref, index) {
                            continue;
                        }
                        if self.strict_equal(&values_ref[index], &search) {
                            found = index as i64;
                            break;
                        }
                    }
                    Value::Number(found)
                }
            }
            "slice" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "slice supports up to two arguments".into(),
                    ));
                }
                let values_ref = values.borrow();
                let len = values_ref.len();
                let start = evaluated_args
                    .first()
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(0);
                let end = evaluated_args
                    .get(1)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(len);
                let end = end.max(start);
                Self::new_array_value(values_ref[start..end].to_vec())
            }
            "join" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "join supports zero or one separator argument".into(),
                    ));
                }
                let separator = evaluated_args
                    .first()
                    .map(|value| self.coerce_to_string_for_string_context(value))
                    .unwrap_or_else(|| ",".to_string());
                let values_ref = values.borrow();
                let mut out = String::new();
                for (idx, value) in values_ref.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(&separator);
                    }
                    if matches!(value, Value::Null | Value::Undefined) {
                        continue;
                    }
                    out.push_str(&self.coerce_to_string_for_string_context(value));
                }
                Value::String(out)
            }
            "concat" => {
                let mut out = values.borrow().clone();
                for arg in evaluated_args {
                    match arg {
                        Value::Array(other) => out.extend(other.borrow().iter().cloned()),
                        _ => out.push(arg.clone()),
                    }
                }
                Self::new_array_value(out)
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
