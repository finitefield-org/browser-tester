use super::*;

impl Harness {
    pub(crate) fn try_eval_array_predicate_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Value> {
        let result = match expr {
            Expr::ArraySome { target, callback } => {
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
                                return Ok(Value::Bool(true));
                            }
                        }
                        Ok(Value::Bool(false))
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
                                return Ok(Value::Bool(true));
                            }
                        }
                        Ok(Value::Bool(false))
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
            Expr::ArrayEvery { target, callback } => {
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
                            if !matched.truthy() {
                                return Ok(Value::Bool(false));
                            }
                        }
                        Ok(Value::Bool(true))
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
                            if !matched.truthy() {
                                return Ok(Value::Bool(false));
                            }
                        }
                        Ok(Value::Bool(true))
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
