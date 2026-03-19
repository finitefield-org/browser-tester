use super::*;

impl Harness {
    pub(crate) fn try_eval_array_higher_order_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match self.try_eval_array_transform_expr(expr, env, event_param, event) {
            Ok(result) => Ok(result),
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => {
                self.try_eval_array_search_expr(expr, env, event_param, event)
            }
            Err(err) => Err(err),
        }
    }
}
