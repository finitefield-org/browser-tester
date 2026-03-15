use super::*;

impl Harness {
    const EVAL_EXPR_STACK_RED_ZONE: usize = 64 * 1024;
    const EVAL_EXPR_STACK_SIZE: usize = 32 * 1024 * 1024;

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
            let env_snapshot = task.env.to_map();
            for entry in env_snapshot.values() {
                let Value::Function(function) = entry else {
                    continue;
                };
                if function.global_scope
                    || function.local_bindings.contains(name)
                    || !function.captured_names.contains(name)
                {
                    continue;
                }
                function
                    .captured_env
                    .borrow_mut()
                    .insert(name.to_string(), value.clone());
            }
        }
    }

    pub(crate) fn sync_scheduled_task_captures_for_binding(&mut self, name: &str, value: &Value) {
        if Self::is_internal_env_key(name) {
            return;
        }

        if self.listeners.capture_name_counts.contains_key(name) {
            for captured_env in self.listeners.captured_envs_for_name(name) {
                let captured_has_binding = captured_env.borrow().contains_key(name);
                if !captured_has_binding {
                    continue;
                }
                captured_env
                    .borrow_mut()
                    .insert(name.to_string(), value.clone());
            }
        }

        for task in &mut self.scheduler.task_queue {
            let task_has_binding = task.env.contains_key(name);
            if !task_has_binding {
                continue;
            }
            task.env.insert(name.to_string(), value.clone());

            let env_snapshot = task.env.to_map();
            for entry in env_snapshot.values() {
                let Value::Function(function) = entry else {
                    continue;
                };
                if function.global_scope
                    || function.local_bindings.contains(name)
                    || !function.captured_names.contains(name)
                {
                    continue;
                }
                function
                    .captured_env
                    .borrow_mut()
                    .insert(name.to_string(), value.clone());
            }
        }
    }

    pub(crate) fn eval_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        stacker::maybe_grow(
            Self::EVAL_EXPR_STACK_RED_ZONE,
            Self::EVAL_EXPR_STACK_SIZE,
            || self.eval_expr_impl(expr, env, event_param, event),
        )
    }

    fn eval_expr_impl(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if let Some(value) = self.eval_expr_core_date_intl(expr, env, event_param, event)? {
            return Ok(value);
        }
        if let Some(value) =
            self.eval_expr_regex_numbers_and_builtins(expr, env, event_param, event)?
        {
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
