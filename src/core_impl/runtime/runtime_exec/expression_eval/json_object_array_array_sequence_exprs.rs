use super::*;

impl Harness {
    pub(crate) fn try_eval_array_sequence_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let result = match expr {
            Expr::ArraySlice { target, start, end } => {
                match self.resolve_target_value_with_pending(env, target) {
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
                        let snapshot = self.typed_array_snapshot(&values)?;
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
                        Self::ensure_array_buffer_not_detached(&buffer, "slice")?;
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
                        Ok(Value::String(Self::substring_chars(&value, start, end)))
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
                    .map(|value| self.coerce_to_string_for_string_context(&value))
                    .unwrap_or_else(|| ",".to_string());
                let values = match self.resolve_target_value_with_pending(env, target) {
                    Some(Value::Array(values)) => values.borrow().clone(),
                    Some(Value::TypedArray(values)) => self.typed_array_snapshot(&values)?,
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
                    out.push_str(&self.coerce_to_string_for_string_context(value));
                }
                Ok(Value::String(out))
            }
            Expr::ArraySort { target, comparator } => {
                let comparator = comparator
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                if let Some(Value::Object(entries)) =
                    self.resolve_target_value_with_pending(env, target)
                {
                    if Self::is_url_search_params_object(&entries.borrow()) {
                        {
                            let mut object_ref = entries.borrow_mut();
                            let mut pairs =
                                Self::url_search_params_pairs_from_object_entries(&object_ref);
                            pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                            Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                        }
                        self.sync_url_search_params_owner(&entries);
                        return Ok(Value::Object(entries));
                    }
                }
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
        }?;
        Ok(result)
    }
}
