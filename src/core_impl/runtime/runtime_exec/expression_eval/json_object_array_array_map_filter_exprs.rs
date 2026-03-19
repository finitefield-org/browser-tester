use super::*;

impl Harness {
    pub(crate) fn try_eval_array_map_filter_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Value> {
        let result = match expr {
            Expr::ArrayMap { target, callback } => {
                match self.resolve_target_value_with_pending(env, target) {
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
                        let input = self.typed_array_snapshot(&values)?;
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
                }
            }
            Expr::ArrayFilter { target, callback } => {
                match self.resolve_target_value_with_pending(env, target) {
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
                        let input = self.typed_array_snapshot(&values)?;
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
                }
            }
            _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
        }?;
        Ok(result)
    }
}
