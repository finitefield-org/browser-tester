use super::*;

impl Harness {
    pub(crate) fn eval_expr_core_date_intl(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        if let Some(value) = self.try_eval_core_date_exprs(expr, env, event_param, event)? {
            return Ok(Some(value));
        }
        if let Some(value) =
            self.try_eval_core_intl_locale_static_exprs(expr, env, event_param, event)?
        {
            return Ok(Some(value));
        }
        self.try_eval_core_intl_exprs(expr, env, event_param, event)
    }
}
