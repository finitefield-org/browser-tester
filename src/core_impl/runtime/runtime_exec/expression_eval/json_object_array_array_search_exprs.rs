use super::*;

impl Harness {
    pub(crate) fn try_eval_array_search_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        _event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match self.try_eval_array_find_expr(expr, env, event) {
            Ok(result) => Ok(result),
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => {
                self.try_eval_array_predicate_expr(expr, env, event)
            }
            Err(err) => Err(err),
        }
    }
}
