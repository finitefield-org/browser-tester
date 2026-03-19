use super::*;

impl Harness {
    pub(crate) fn eval_expr_events_unary_control(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        if let Some(value) = self.try_eval_event_and_delete_expr(expr, env, event_param, event)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.try_eval_unary_and_async_expr(expr, env, event_param, event)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.try_eval_misc_control_expr(expr, env, event_param, event)? {
            return Ok(Some(value));
        }
        Ok(None)
    }
}
