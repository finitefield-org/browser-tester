use super::*;

impl Harness {
    pub(crate) fn try_eval_array_find_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Value> {
        let result = match expr {
            Expr::ArrayForEach { target, callback } => {
                match self.resolve_target_value_with_pending(env, target) {
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
                        let input = self.typed_array_snapshot(&values)?;
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
                }
            }
            Expr::ArrayFind { target, callback } => {
                match self.resolve_target_value_with_pending(env, target) {
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
                        let input = self.typed_array_snapshot(&values)?;
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
                }
            }
            Expr::ArrayFindIndex { target, callback } => {
                match self.resolve_target_value_with_pending(env, target) {
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
                        let input = self.typed_array_snapshot(&values)?;
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
                }
            }
            _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
        }?;
        Ok(result)
    }
}
