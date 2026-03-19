use super::*;

impl Harness {
    pub(crate) fn try_eval_scheduler_and_binary_exprs(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::SetTimeout { handler, delay_ms } => {
                    let delay = self.eval_expr(delay_ms, env, event_param, event)?;
                    let delay = Self::value_to_i64(&delay);
                    let callback_args = handler
                        .args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    let (callback, timer_env) = self.materialize_timer_callback_for_schedule(
                        &handler.callback,
                        env,
                        "timeout",
                    );
                    let id = self.schedule_timeout(callback, delay, callback_args, &timer_env);
                    Ok(Value::Number(id))
                }
                Expr::SetInterval { handler, delay_ms } => {
                    let interval = self.eval_expr(delay_ms, env, event_param, event)?;
                    let interval = Self::value_to_i64(&interval);
                    let callback_args = handler
                        .args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    let (callback, timer_env) = self.materialize_timer_callback_for_schedule(
                        &handler.callback,
                        env,
                        "interval",
                    );
                    let id = self.schedule_interval(callback, interval, callback_args, &timer_env);
                    Ok(Value::Number(id))
                }
                Expr::RequestAnimationFrame { callback } => {
                    let (callback, timer_env) =
                        self.materialize_timer_callback_for_schedule(callback, env, "raf");
                    let id = self.schedule_animation_frame(callback, &timer_env);
                    Ok(Value::Number(id))
                }
                Expr::QueueMicrotask { handler } => {
                    self.queue_microtask(handler.clone(), env);
                    Ok(Value::Undefined)
                }
                Expr::Binary { left, op, right } => match op {
                    BinaryOp::And => {
                        let mut operands =
                            Self::collect_left_associative_binary_operands(expr, BinaryOp::And)
                                .into_iter();
                        let Some(first) = operands.next() else {
                            return Ok(Value::Undefined);
                        };
                        let mut current = self.eval_expr(first, env, event_param, event)?;
                        for operand in operands {
                            if !current.truthy() {
                                return Ok(current);
                            }
                            current = self.eval_expr(operand, env, event_param, event)?;
                        }
                        Ok(current)
                    }
                    BinaryOp::Or => {
                        let mut operands =
                            Self::collect_left_associative_binary_operands(expr, BinaryOp::Or)
                                .into_iter();
                        let Some(first) = operands.next() else {
                            return Ok(Value::Undefined);
                        };
                        let mut current = self.eval_expr(first, env, event_param, event)?;
                        for operand in operands {
                            if current.truthy() {
                                return Ok(current);
                            }
                            current = self.eval_expr(operand, env, event_param, event)?;
                        }
                        Ok(current)
                    }
                    BinaryOp::Nullish => {
                        let mut operands =
                            Self::collect_left_associative_binary_operands(expr, BinaryOp::Nullish)
                                .into_iter();
                        let Some(first) = operands.next() else {
                            return Ok(Value::Undefined);
                        };
                        let mut current = self.eval_expr(first, env, event_param, event)?;
                        for operand in operands {
                            if matches!(current, Value::Null | Value::Undefined) {
                                current = self.eval_expr(operand, env, event_param, event)?;
                            } else {
                                break;
                            }
                        }
                        Ok(current)
                    }
                    _ => {
                        let left = self.eval_expr(left, env, event_param, event)?;
                        let right = self.eval_expr(right, env, event_param, event)?;
                        self.eval_binary(op, &left, &right)
                    }
                },
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
