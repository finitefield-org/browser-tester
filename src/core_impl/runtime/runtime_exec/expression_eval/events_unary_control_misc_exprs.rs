use super::*;

impl Harness {
    pub(crate) fn try_eval_misc_control_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::Comma(parts) => {
                    let mut last = Value::Undefined;
                    for part in parts {
                        last = self.eval_expr(part, env, event_param, event)?;
                    }
                    Ok(last)
                }
                Expr::Spread(_) => Err(Error::ScriptRuntime(
                    "spread syntax is only supported in array literals, object literals, and call arguments".into(),
                )),
                Expr::Add(parts) => {
                    if parts.is_empty() {
                        return Ok(Value::String(String::new()));
                    }
                    let mut iter = parts.iter();
                    let first = iter
                        .next()
                        .ok_or_else(|| Error::ScriptRuntime("empty add expression".into()))?;
                    let mut acc = self.eval_expr(first, env, event_param, event)?;
                    for part in iter {
                        let rhs = self.eval_expr(part, env, event_param, event)?;
                        acc = self.add_values(&acc, &rhs)?;
                    }
                    Ok(acc)
                }
                Expr::Ternary {
                    cond,
                    on_true,
                    on_false,
                } => {
                    let cond = self.eval_expr(cond, env, event_param, event)?;
                    if cond.truthy() {
                        self.eval_expr(on_true, env, event_param, event)
                    } else {
                        self.eval_expr(on_false, env, event_param, event)
                    }
                }
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
