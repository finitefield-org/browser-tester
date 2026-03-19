use super::*;

impl Harness {
    pub(crate) fn try_eval_array_reduce_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let result = match expr {
            Expr::ArrayReduce {
                target,
                callback,
                initial,
            } => match self.resolve_target_value_with_pending(env, target) {
                Some(Value::Array(values)) => {
                    let input = values.borrow().clone();
                    let mut start_index = 0usize;
                    let mut acc = if let Some(initial) = initial {
                        self.eval_expr(initial, env, event_param, event)?
                    } else {
                        let Some(first) = input.first().cloned() else {
                            return Err(Error::ScriptRuntime(
                                "reduce of empty array with no initial value".into(),
                            ));
                        };
                        start_index = 1;
                        first
                    };
                    for (idx, item) in input.into_iter().enumerate().skip(start_index) {
                        acc = self.execute_array_callback(
                            callback,
                            &[
                                acc,
                                item,
                                Value::Number(idx as i64),
                                Value::Array(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                    }
                    Ok(acc)
                }
                Some(Value::TypedArray(values)) => {
                    let input = self.typed_array_snapshot(&values)?;
                    let mut start_index = 0usize;
                    let mut acc = if let Some(initial) = initial {
                        self.eval_expr(initial, env, event_param, event)?
                    } else {
                        let Some(first) = input.first().cloned() else {
                            return Err(Error::ScriptRuntime(
                                "reduce of empty array with no initial value".into(),
                            ));
                        };
                        start_index = 1;
                        first
                    };
                    for (idx, item) in input.into_iter().enumerate().skip(start_index) {
                        acc = self.execute_array_callback(
                            callback,
                            &[
                                acc,
                                item,
                                Value::Number(idx as i64),
                                Value::TypedArray(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                    }
                    Ok(acc)
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
        }?;
        Ok(result)
    }
}
