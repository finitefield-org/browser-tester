use super::*;

impl Harness {
    pub(crate) fn eval_expr_string_and_webapi(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        if let Some(value) = self.try_eval_string_core_exprs(expr, env, event_param, event)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.try_eval_string_pattern_exprs(expr, env, event_param, event)? {
            return Ok(Some(value));
        }
        self.try_eval_string_platform_exprs(expr, env, event_param, event)
    }
}
