const UNHANDLED_EXPR_CHUNK: &str = "__bt_unhandled_eval_expr_chunk__";

impl Harness {
    pub(crate) fn bind_timer_id_to_task_env(&mut self, name: &str, expr: &Expr, value: &Value) {
        if !matches!(
            expr,
            Expr::SetTimeout { .. } | Expr::SetInterval { .. } | Expr::RequestAnimationFrame { .. }
        ) {
            return;
        }
        let Value::Number(timer_id) = value else {
            return;
        };
        for task in self
            .scheduler
            .task_queue
            .iter_mut()
            .filter(|task| task.id == *timer_id)
        {
            task.env.insert(name.to_string(), value.clone());
        }
    }

    pub(crate) fn eval_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if let Some(value) = self.eval_expr_core_date_intl(expr, env, event_param, event)? {
            return Ok(value);
        }
        if let Some(value) = self.eval_expr_regex_numbers_and_builtins(expr, env, event_param, event)? {
            return Ok(value);
        }
        if let Some(value) = self.eval_expr_json_object_array(expr, env, event_param, event)? {
            return Ok(value);
        }
        if let Some(value) = self.eval_expr_string_and_webapi(expr, env, event_param, event)? {
            return Ok(value);
        }
        if let Some(value) = self.eval_expr_calls_timers_binary(expr, env, event_param, event)? {
            return Ok(value);
        }
        if let Some(value) = self.eval_expr_dom_and_platform(expr, env, event_param, event)? {
            return Ok(value);
        }
        if let Some(value) = self.eval_expr_events_unary_control(expr, env, event_param, event)? {
            return Ok(value);
        }
        Err(Error::ScriptRuntime("unsupported expression".into()))
    }
}
