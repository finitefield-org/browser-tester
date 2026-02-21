use super::*;

impl Harness {
    pub(crate) fn queue_microtask(&mut self, handler: ScriptHandler, env: &HashMap<String, Value>) {
        self.scheduler
            .microtask_queue
            .push_back(ScheduledMicrotask::Script {
                handler,
                env: ScriptEnv::from_snapshot(env),
            });
    }

    pub(crate) fn queue_promise_reaction_microtask(
        &mut self,
        reaction: PromiseReactionKind,
        settled: PromiseSettledValue,
    ) {
        self.scheduler
            .microtask_queue
            .push_back(ScheduledMicrotask::Promise { reaction, settled });
    }

    pub(crate) fn run_microtask_queue(&mut self) -> Result<usize> {
        self.with_task_depth(|this| {
            let mut steps = 0usize;
            loop {
                let Some(task) = this.scheduler.microtask_queue.pop_front() else {
                    return Ok(steps);
                };
                steps += 1;
                if steps > this.scheduler.timer_step_limit {
                    return Err(this.timer_step_limit_error(
                        this.scheduler.timer_step_limit,
                        steps,
                        Some(this.scheduler.now_ms),
                    ));
                }

                match task {
                    ScheduledMicrotask::Script { handler, mut env } => {
                        this.run_script_microtask_handler(&handler, &mut env)?;
                    }
                    ScheduledMicrotask::Promise { reaction, settled } => {
                        this.run_promise_reaction_task(reaction, settled)?;
                    }
                }
            }
        })
    }

    fn with_task_depth<T>(&mut self, run: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.scheduler.task_depth += 1;
        let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(self)));
        self.scheduler.task_depth = self.scheduler.task_depth.saturating_sub(1);
        match run_result {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    pub(crate) fn run_in_task_context<T>(
        &mut self,
        mut run: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let result = self.with_task_depth(|this| run(this));
        let should_flush_microtasks = self.scheduler.task_depth == 0;
        match result {
            Ok(value) => {
                if should_flush_microtasks {
                    self.run_microtask_queue()?;
                }
                Ok(value)
            }
            Err(err) => Err(err),
        }
    }

    pub(crate) fn execute_handler(
        &mut self,
        handler: &ScriptHandler,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let event_param = handler
            .first_event_param()
            .map(|event_param| event_param.to_string());
        let event_args = if event_param.is_some() {
            vec![Self::listener_event_argument(event)]
        } else {
            Vec::new()
        };
        self.with_callback_scope_depth(env, |this, callback_env| {
            this.bind_handler_params(handler, &event_args, callback_env, &event_param, event)?;
            let flow = this.execute_stmts(&handler.stmts, &event_param, event, callback_env)?;
            callback_env.remove(INTERNAL_RETURN_SLOT);
            match flow {
                ExecFlow::Continue => Ok(()),
                ExecFlow::Break => Err(Error::ScriptRuntime(
                    "break statement outside of loop".into(),
                )),
                ExecFlow::ContinueLoop => Err(Error::ScriptRuntime(
                    "continue statement outside of loop".into(),
                )),
                ExecFlow::Return => Ok(()),
            }
        })
    }

    fn listener_event_argument(event: &EventState) -> Value {
        Self::new_object_value(vec![
            ("type".to_string(), Value::String(event.event_type.clone())),
            ("target".to_string(), Value::Node(event.target)),
            ("currentTarget".to_string(), Value::Node(event.current_target)),
            (
                "defaultPrevented".to_string(),
                Value::Bool(event.default_prevented),
            ),
            ("isTrusted".to_string(), Value::Bool(event.is_trusted)),
            ("bubbles".to_string(), Value::Bool(event.bubbles)),
            ("cancelable".to_string(), Value::Bool(event.cancelable)),
            ("eventPhase".to_string(), Value::Number(event.event_phase as i64)),
            ("timeStamp".to_string(), Value::Number(event.time_stamp_ms)),
            (
                "state".to_string(),
                event.state.as_ref().cloned().unwrap_or(Value::Undefined),
            ),
            (
                "oldState".to_string(),
                event.old_state
                    .as_ref()
                    .map(|value| Value::String(value.clone()))
                    .unwrap_or(Value::Undefined),
            ),
            (
                "newState".to_string(),
                event.new_state
                    .as_ref()
                    .map(|value| Value::String(value.clone()))
                    .unwrap_or(Value::Undefined),
            ),
        ])
    }

    pub(crate) fn execute_timer_task_callback(
        &mut self,
        callback: &TimerCallback,
        callback_args: &[Value],
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let handler = match callback {
            TimerCallback::Inline(handler) => handler.clone(),
            TimerCallback::Reference(name) => {
                let value = env
                    .get(name)
                    .cloned()
                    .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;
                let Value::Function(function) = value else {
                    return Err(Error::ScriptRuntime(format!(
                        "timer callback '{name}' is not a function"
                    )));
                };
                function.handler.clone()
            }
        };
        let event_param = handler
            .first_event_param()
            .map(|event_param| event_param.to_string());
        self.with_callback_scope_depth(env, |this, callback_env| {
            this.bind_handler_params(&handler, callback_args, callback_env, &event_param, event)?;
            let flow = this.execute_stmts(&handler.stmts, &event_param, event, callback_env)?;
            callback_env.remove(INTERNAL_RETURN_SLOT);
            match flow {
                ExecFlow::Continue => Ok(()),
                ExecFlow::Break => Err(Error::ScriptRuntime(
                    "break statement outside of loop".into(),
                )),
                ExecFlow::ContinueLoop => Err(Error::ScriptRuntime(
                    "continue statement outside of loop".into(),
                )),
                ExecFlow::Return => Ok(()),
            }
        })
    }

    fn run_script_microtask_handler(
        &mut self,
        handler: &ScriptHandler,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let mut event = EventState::new("microtask", self.dom.root, self.scheduler.now_ms);
        let event_param = handler
            .first_event_param()
            .map(|event_param| event_param.to_string());
        self.with_callback_scope_depth(env, |this, callback_env| {
            this.bind_handler_params(handler, &[], callback_env, &event_param, &event)?;
            let run = this.execute_stmts(&handler.stmts, &event_param, &mut event, callback_env);
            run.map(|_| ())
        })
    }

    pub(crate) fn with_callback_scope_depth<T>(
        &mut self,
        env: &mut HashMap<String, Value>,
        run: impl FnOnce(&mut Self, &mut HashMap<String, Value>) -> Result<T>,
    ) -> Result<T> {
        let previous = env.get(INTERNAL_SCOPE_DEPTH_KEY).cloned();
        let next_depth = Self::env_scope_depth(env).saturating_add(1);
        env.insert(
            INTERNAL_SCOPE_DEPTH_KEY.to_string(),
            Value::Number(next_depth),
        );
        let result = run(self, env);
        match previous {
            Some(value) => {
                env.insert(INTERNAL_SCOPE_DEPTH_KEY.to_string(), value);
            }
            None => {
                env.remove(INTERNAL_SCOPE_DEPTH_KEY);
            }
        }
        result
    }

    pub(crate) fn is_internal_env_key(name: &str) -> bool {
        name.starts_with("\u{0}\u{0}bt_")
    }

    pub(crate) fn env_scope_depth(env: &HashMap<String, Value>) -> i64 {
        match env.get(INTERNAL_SCOPE_DEPTH_KEY) {
            Some(Value::Number(depth)) if *depth >= 0 => *depth,
            _ => 0,
        }
    }

    pub(crate) fn env_should_sync_global_name(env: &HashMap<String, Value>, name: &str) -> bool {
        match env.get(INTERNAL_GLOBAL_SYNC_NAMES_KEY) {
            Some(Value::Array(names)) => names
                .borrow()
                .iter()
                .any(|entry| matches!(entry, Value::String(value) if value == name)),
            _ => false,
        }
    }

    pub(crate) fn ensure_listener_capture_env(&mut self) -> Rc<RefCell<ScriptEnv>> {
        if let Some(frame) = self.script_runtime.listener_capture_env_stack.last_mut() {
            frame
                .shared_env
                .get_or_insert_with(|| Rc::new(RefCell::new(ScriptEnv::default())))
                .clone()
        } else {
            Rc::new(RefCell::new(ScriptEnv::default()))
        }
    }

    pub(crate) fn queue_listener_capture_env_update(&mut self, name: String, value: Option<Value>) {
        if Self::is_internal_env_key(&name) {
            return;
        }
        if let Some(frame) = self.script_runtime.listener_capture_env_stack.last_mut() {
            frame.pending_env_updates.insert(name, value);
        }
    }

    pub(crate) fn apply_pending_listener_capture_env_updates(
        &mut self,
        env: &mut HashMap<String, Value>,
    ) {
        let Some(frame) = self.script_runtime.listener_capture_env_stack.last_mut() else {
            return;
        };
        if frame.pending_env_updates.is_empty() {
            return;
        }
        let updates = std::mem::take(&mut frame.pending_env_updates);
        for (name, value) in updates {
            if Self::is_internal_env_key(&name) {
                continue;
            }
            if let Some(value) = value {
                env.insert(name, value);
            } else {
                env.remove(&name);
            }
        }
    }

    pub(crate) fn push_pending_function_decl_scope(
        &mut self,
        scope: HashMap<String, (ScriptHandler, bool)>,
    ) -> usize {
        let start_len = self.script_runtime.pending_function_decls.len();
        if !scope.is_empty() {
            self.script_runtime
                .pending_function_decls
                .push(Arc::new(scope));
        }
        start_len
    }

    pub(crate) fn push_pending_function_decl_scopes(
        &mut self,
        scopes: &[Arc<HashMap<String, (ScriptHandler, bool)>>],
    ) -> usize {
        let start_len = self.script_runtime.pending_function_decls.len();
        self.script_runtime
            .pending_function_decls
            .extend(scopes.iter().cloned());
        start_len
    }

    pub(crate) fn restore_pending_function_decl_scopes(&mut self, start_len: usize) {
        self.script_runtime
            .pending_function_decls
            .truncate(start_len);
    }

    pub(crate) fn sync_global_binding_if_needed(
        &mut self,
        env: &HashMap<String, Value>,
        name: &str,
        value: &Value,
    ) {
        if Self::env_should_sync_global_name(env, name) {
            self.script_runtime
                .env
                .insert(name.to_string(), value.clone());
        }
    }
}
