use super::*;

impl Harness {
    pub(super) fn execute_set_timeout_stmt(
        &mut self,
        handler: &TimerInvocation,
        delay_ms: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let delay = self.eval_expr(delay_ms, env, event_param, event)?;
        let delay = Self::value_to_i64(&delay);
        let callback_args = handler
            .args
            .iter()
            .map(|arg| self.eval_expr(arg, env, event_param, event))
            .collect::<Result<Vec<_>>>()?;
        let (callback, timer_env) =
            self.materialize_timer_callback_for_schedule(&handler.callback, env, "timeout");
        let _ = self.schedule_timeout(callback, delay, callback_args, &timer_env);
        Ok(ExecFlow::Continue)
    }

    pub(super) fn execute_set_interval_stmt(
        &mut self,
        handler: &TimerInvocation,
        delay_ms: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let interval = self.eval_expr(delay_ms, env, event_param, event)?;
        let interval = Self::value_to_i64(&interval);
        let callback_args = handler
            .args
            .iter()
            .map(|arg| self.eval_expr(arg, env, event_param, event))
            .collect::<Result<Vec<_>>>()?;
        let (callback, timer_env) =
            self.materialize_timer_callback_for_schedule(&handler.callback, env, "interval");
        let _ = self.schedule_interval(callback, interval, callback_args, &timer_env);
        Ok(ExecFlow::Continue)
    }

    pub(super) fn execute_clear_timeout_stmt(
        &mut self,
        timer_id: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let timer_id = self.eval_expr(timer_id, env, event_param, event)?;
        let timer_id = Self::value_to_i64(&timer_id);
        self.clear_timeout(timer_id);
        Ok(ExecFlow::Continue)
    }
}
