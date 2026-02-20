impl Harness {
    pub(super) fn invoke_listeners(
        &mut self,
        node_id: NodeId,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
        capture: bool,
    ) -> Result<()> {
        let listeners = self.listeners.get(node_id, &event.event_type, capture);
        for listener in listeners {
            let mut listener_env = env.clone();
            let captured_env_snapshot = listener.captured_env.borrow().to_map();
            let captured_keys = captured_env_snapshot
                .keys()
                .filter(|name| !Self::is_internal_env_key(name))
                .cloned()
                .collect::<Vec<_>>();
            for (name, value) in &captured_env_snapshot {
                if Self::is_internal_env_key(name) {
                    continue;
                }
                if !listener_env.contains_key(name) {
                    listener_env.insert(name.clone(), value.clone());
                }
            }
            let current_keys = env.keys().cloned().collect::<Vec<_>>();
            let mut script_env_before = HashMap::new();
            for key in &current_keys {
                if let Some(value) = self.script_runtime.env.get(key).cloned() {
                    script_env_before.insert(key.clone(), value);
                }
            }
            if self.trace_state.enabled {
                let phase = if capture { "capture" } else { "bubble" };
                let target_label = self.trace_node_label(event.target);
                let current_label = self.trace_node_label(event.current_target);
                self.trace_event_line(format!(
                    "[event] {} target={} current={} phase={} default_prevented={}",
                    event.event_type, target_label, current_label, phase, event.default_prevented
                ));
            }
            let pending_scope_start =
                self.push_pending_function_decl_scopes(&listener.captured_pending_function_decls);
            let call_result = self.execute_handler(&listener.handler, event, &mut listener_env);
            self.restore_pending_function_decl_scopes(pending_scope_start);
            {
                let mut captured_env = listener.captured_env.borrow_mut();
                for key in &captured_keys {
                    let before = captured_env_snapshot.get(key);
                    let after = listener_env.get(key);
                    let changed = match (before, after) {
                        (Some(prev), Some(next)) => !self.strict_equal(prev, next),
                        (None, Some(_)) => true,
                        (Some(_), None) => true,
                        (None, None) => false,
                    };
                    if !changed {
                        continue;
                    }
                    if let Some(value) = after.cloned() {
                        captured_env.insert(key.clone(), value);
                    } else {
                        captured_env.remove(key);
                    }
                }
            }
            for key in current_keys {
                let listener_value = listener_env.get(&key).cloned();
                let before = script_env_before.get(&key);
                let after = self.script_runtime.env.get(&key).cloned();
                let script_changed = match (before, after.as_ref()) {
                    (Some(prev), Some(next)) => !self.strict_equal(prev, next),
                    (None, Some(_)) => true,
                    _ => false,
                };
                if script_changed {
                    if let Some(value) = after {
                        env.insert(key, value);
                    } else if let Some(value) = listener_value {
                        env.insert(key, value);
                    }
                } else if let Some(value) = listener_value {
                    env.insert(key, value);
                }
            }
            if let Err(err) = call_result {
                return Err(err);
            }
            if event.immediate_propagation_stopped {
                break;
            }
        }
        Ok(())
    }

    pub(super) fn trace_event_done(&mut self, event: &EventState, outcome: &str) {
        let target_label = self.trace_node_label(event.target);
        let current_label = self.trace_node_label(event.current_target);
        self.trace_event_line(format!(
            "[event] done {} target={} current={} outcome={} default_prevented={} propagation_stopped={} immediate_stopped={}",
            event.event_type,
            target_label,
            current_label,
            outcome,
            event.default_prevented,
            event.propagation_stopped,
            event.immediate_propagation_stopped
        ));
    }

    pub(super) fn trace_event_line(&mut self, line: String) {
        if self.trace_state.enabled && self.trace_state.events {
            self.trace_line(line);
        }
    }

    pub(super) fn trace_timer_line(&mut self, line: String) {
        if self.trace_state.enabled && self.trace_state.timers {
            self.trace_line(line);
        }
    }

    pub(super) fn trace_line(&mut self, line: String) {
        if self.trace_state.enabled {
            if self.trace_state.to_stderr {
                eprintln!("{line}");
            }
            if self.trace_state.logs.len() >= self.trace_state.log_limit {
                self.trace_state.logs.pop_front();
            }
            self.trace_state.logs.push_back(line);
        }
    }

    pub(super) fn queue_microtask(&mut self, handler: ScriptHandler, env: &HashMap<String, Value>) {
        self.scheduler.microtask_queue.push_back(ScheduledMicrotask::Script {
            handler,
            env: ScriptEnv::from_snapshot(env),
        });
    }

    pub(super) fn queue_promise_reaction_microtask(
        &mut self,
        reaction: PromiseReactionKind,
        settled: PromiseSettledValue,
    ) {
        self.scheduler.microtask_queue
            .push_back(ScheduledMicrotask::Promise { reaction, settled });
    }

    pub(super) fn run_microtask_queue(&mut self) -> Result<usize> {
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
                        let mut event =
                            EventState::new("microtask", this.dom.root, this.scheduler.now_ms);
                        let event_param = handler
                            .first_event_param()
                            .map(|event_param| event_param.to_string());
                        this.bind_handler_params(&handler, &[], &mut env, &event_param, &event)?;
                        let run = this.execute_stmts(
                            &handler.stmts,
                            &event_param,
                            &mut event,
                            &mut env,
                        );
                        let run = run.map(|_| ());
                        if let Err(err) = run {
                            return Err(err);
                        }
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

    pub(super) fn run_in_task_context<T>(
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

    pub(super) fn execute_handler(
        &mut self,
        handler: &ScriptHandler,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let event_param = handler
            .first_event_param()
            .map(|event_param| event_param.to_string());
        let event_args = if event_param.is_some() {
            vec![Self::new_object_value(Vec::new())]
        } else {
            Vec::new()
        };
        self.bind_handler_params(handler, &event_args, env, &event_param, event)?;
        let flow = self.execute_stmts(&handler.stmts, &event_param, event, env)?;
        env.remove(INTERNAL_RETURN_SLOT);
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
    }

    pub(super) fn execute_timer_task_callback(
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
        self.bind_handler_params(&handler, callback_args, env, &event_param, event)?;
        let flow = self.execute_stmts(&handler.stmts, &event_param, event, env)?;
        env.remove(INTERNAL_RETURN_SLOT);
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
    }

    fn is_internal_env_key(name: &str) -> bool {
        name.starts_with("\u{0}\u{0}bt_")
    }

    fn env_scope_depth(env: &HashMap<String, Value>) -> i64 {
        match env.get(INTERNAL_SCOPE_DEPTH_KEY) {
            Some(Value::Number(depth)) if *depth >= 0 => *depth,
            _ => 0,
        }
    }

    fn env_should_sync_global_name(env: &HashMap<String, Value>, name: &str) -> bool {
        match env.get(INTERNAL_GLOBAL_SYNC_NAMES_KEY) {
            Some(Value::Array(names)) => names
                .borrow()
                .iter()
                .any(|entry| matches!(entry, Value::String(value) if value == name)),
            _ => false,
        }
    }

    fn ensure_listener_capture_env(&mut self) -> Rc<RefCell<ScriptEnv>> {
        if let Some(frame) = self.script_runtime.listener_capture_env_stack.last_mut() {
            frame
                .shared_env
                .get_or_insert_with(|| Rc::new(RefCell::new(ScriptEnv::default())))
                .clone()
        } else {
            Rc::new(RefCell::new(ScriptEnv::default()))
        }
    }

    fn push_pending_function_decl_scope(
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

    fn push_pending_function_decl_scopes(
        &mut self,
        scopes: &[Arc<HashMap<String, (ScriptHandler, bool)>>],
    ) -> usize {
        let start_len = self.script_runtime.pending_function_decls.len();
        self.script_runtime
            .pending_function_decls
            .extend(scopes.iter().cloned());
        start_len
    }

    fn restore_pending_function_decl_scopes(&mut self, start_len: usize) {
        self.script_runtime.pending_function_decls.truncate(start_len);
    }

    pub(super) fn sync_global_binding_if_needed(
        &mut self,
        env: &HashMap<String, Value>,
        name: &str,
        value: &Value,
    ) {
        if Self::env_should_sync_global_name(env, name) {
            self.script_runtime.env.insert(name.to_string(), value.clone());
        }
    }

    pub(super) fn make_function_value(
        &mut self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        is_async: bool,
    ) -> Value {
        let local_bindings = Self::collect_function_scope_bindings(&handler);
        let scope_depth = Self::env_scope_depth(env);
        let captured_pending_function_decls = self.script_runtime.pending_function_decls.clone();
        let captured_env = if global_scope {
            Rc::new(RefCell::new(self.script_runtime.env.share()))
        } else {
            let captured_env = self.ensure_listener_capture_env();
            *captured_env.borrow_mut() = ScriptEnv::from_snapshot(env);
            captured_env
        };
        let captured_env_snapshot = captured_env.borrow();
        let mut captured_global_names = HashSet::new();
        for (name, value) in captured_env_snapshot.iter() {
            if Self::is_internal_env_key(name) || name == INTERNAL_RETURN_SLOT {
                continue;
            }
            if scope_depth == 0 {
                captured_global_names.insert(name.clone());
                continue;
            }
            let Some(global_value) = self.script_runtime.env.get(name) else {
                continue;
            };
            if global_scope || self.strict_equal(global_value, value) {
                captured_global_names.insert(name.clone());
            }
        }
        drop(captured_env_snapshot);
        Value::Function(Rc::new(FunctionValue {
            handler,
            captured_env,
            captured_pending_function_decls,
            captured_global_names,
            local_bindings,
            global_scope,
            is_async,
        }))
    }

    pub(super) fn is_callable_value(&self, value: &Value) -> bool {
        matches!(
            value,
            Value::Function(_) | Value::PromiseCapability(_) | Value::StringConstructor
        ) || Self::callable_kind_from_value(value).is_some()
    }

    pub(super) fn execute_callable_value(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
    ) -> Result<Value> {
        self.execute_callable_value_with_env(callable, args, event, None)
    }

    pub(super) fn execute_callable_value_with_env(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        match callable {
            Value::Function(function) => {
                self.execute_function_call(function.as_ref(), args, event, caller_env)
            }
            Value::PromiseCapability(capability) => {
                self.invoke_promise_capability(capability, args)
            }
            Value::StringConstructor => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Ok(Value::String(value.as_string()))
            }
            Value::Object(_) => {
                let Some(kind) = Self::callable_kind_from_value(callable) else {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                };
                match kind {
                    "intl_collator_compare" => {
                        let (locale, case_first, sensitivity) =
                            self.resolve_intl_collator_options(callable)?;
                        let left = args
                            .first()
                            .cloned()
                            .unwrap_or(Value::Undefined)
                            .as_string();
                        let right = args.get(1).cloned().unwrap_or(Value::Undefined).as_string();
                        Ok(Value::Number(Self::intl_collator_compare_strings(
                            &left,
                            &right,
                            &locale,
                            &case_first,
                            &sensitivity,
                        )))
                    }
                    "intl_date_time_format" => {
                        let (locale, options) = self.resolve_intl_date_time_options(callable)?;
                        let timestamp_ms = args
                            .first()
                            .map(|value| self.coerce_date_timestamp_ms(value))
                            .unwrap_or(self.scheduler.now_ms);
                        Ok(Value::String(self.intl_format_date_time(
                            timestamp_ms,
                            &locale,
                            &options,
                        )))
                    }
                    "intl_duration_format" => {
                        let (locale, options) = self.resolve_intl_duration_options(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(
                            self.intl_format_duration(&locale, &options, &value)?,
                        ))
                    }
                    "intl_list_format" => {
                        let (locale, options) = self.resolve_intl_list_options(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(
                            self.intl_format_list(&locale, &options, &value)?,
                        ))
                    }
                    "intl_number_format" => {
                        let (_, locale) = self.resolve_intl_formatter(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(Self::intl_format_number_for_locale(
                            Self::coerce_number_for_global(&value),
                            &locale,
                        )))
                    }
                    "intl_segmenter_segments_iterator" => {
                        let Value::Object(entries) = callable else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let entries = entries.borrow();
                        let segments = Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENTS_KEY)
                            .ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "Intl.Segmenter iterator has invalid internal state".into(),
                                )
                            })?;
                        Ok(self.new_intl_segmenter_iterator_value(segments))
                    }
                    "intl_segmenter_iterator_next" => {
                        let Value::Object(entries) = callable else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let mut entries = entries.borrow_mut();
                        let segments = Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENTS_KEY)
                            .ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "Intl.Segmenter iterator has invalid internal state".into(),
                                )
                            })?;
                        let Value::Array(values) = segments else {
                            return Err(Error::ScriptRuntime(
                                "Intl.Segmenter iterator has invalid internal state".into(),
                            ));
                        };
                        let len = values.borrow().len();
                        let index =
                            match Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENT_INDEX_KEY)
                            {
                                Some(Value::Number(value)) if value >= 0 => value as usize,
                                _ => 0,
                            };
                        if index >= len {
                            return Ok(Self::new_object_value(vec![
                                ("value".to_string(), Value::Undefined),
                                ("done".to_string(), Value::Bool(true)),
                            ]));
                        }
                        let value = values
                            .borrow()
                            .get(index)
                            .cloned()
                            .unwrap_or(Value::Undefined);
                        Self::object_set_entry(
                            &mut entries,
                            INTERNAL_INTL_SEGMENT_INDEX_KEY.to_string(),
                            Value::Number((index + 1) as i64),
                        );
                        Ok(Self::new_object_value(vec![
                            ("value".to_string(), value),
                            ("done".to_string(), Value::Bool(false)),
                        ]))
                    }
                    "boolean_constructor" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::Bool(value.truthy()))
                    }
                    _ => Err(Error::ScriptRuntime("callback is not a function".into())),
                }
            }
            _ => Err(Error::ScriptRuntime("callback is not a function".into())),
        }
    }

    pub(super) fn invoke_promise_capability(
        &mut self,
        capability: &PromiseCapabilityFunction,
        args: &[Value],
    ) -> Result<Value> {
        let mut already_called = capability.already_called.borrow_mut();
        if *already_called {
            return Ok(Value::Undefined);
        }
        *already_called = true;
        drop(already_called);

        let value = args.first().cloned().unwrap_or(Value::Undefined);
        if capability.reject {
            self.promise_reject(&capability.promise, value);
            Ok(Value::Undefined)
        } else {
            self.promise_resolve(&capability.promise, value)?;
            Ok(Value::Undefined)
        }
    }

    pub(super) fn bind_handler_params(
        &mut self,
        handler: &ScriptHandler,
        args: &[Value],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        for (index, param) in handler.params.iter().enumerate() {
            if param.is_rest {
                let rest = if index < args.len() {
                    args[index..].to_vec()
                } else {
                    Vec::new()
                };
                env.insert(param.name.clone(), Self::new_array_value(rest));
                continue;
            }

            let provided = args.get(index).cloned().unwrap_or(Value::Undefined);
            let value = if matches!(provided, Value::Undefined) {
                if let Some(default_expr) = &param.default {
                    self.eval_expr(default_expr, env, event_param, event)?
                } else {
                    Value::Undefined
                }
            } else {
                provided
            };
            env.insert(param.name.clone(), value);
        }
        Ok(())
    }

    pub(super) fn execute_function_call(
        &mut self,
        function: &FunctionValue,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        let run =
            |this: &mut Self, caller_env: Option<&HashMap<String, Value>>| -> Result<Value> {
                let pending_scope_start =
                    this.push_pending_function_decl_scopes(&function.captured_pending_function_decls);

                let result = (|| -> Result<Value> {
                    let captured_env_before_call = if function.global_scope {
                        HashMap::new()
                    } else {
                        function.captured_env.borrow().to_map()
                    };
                    let mut call_env = if function.global_scope {
                        this.script_runtime.env.to_map()
                    } else {
                        captured_env_before_call.clone()
                    };
                    call_env.remove(INTERNAL_RETURN_SLOT);
                    let scope_depth = Self::env_scope_depth(&call_env);
                    call_env.insert(
                        INTERNAL_SCOPE_DEPTH_KEY.to_string(),
                        Value::Number(scope_depth.saturating_add(1)),
                    );
                    let mut global_sync_keys = HashSet::new();
                    let caller_view = caller_env;
                    for name in &function.captured_global_names {
                        if Self::is_internal_env_key(name) || function.local_bindings.contains(name) {
                            continue;
                        }
                        global_sync_keys.insert(name.clone());
                        if let Some(global_value) = this.script_runtime.env.get(name).cloned() {
                            call_env.insert(name.clone(), global_value);
                        } else if let Some(value) = caller_view.and_then(|env| env.get(name)).cloned() {
                            call_env.insert(name.clone(), value);
                        }
                    }
                    for (name, global_value) in this.script_runtime.env.iter() {
                        if Self::is_internal_env_key(name)
                            || function.local_bindings.contains(name)
                            || call_env.contains_key(name)
                        {
                            continue;
                        }
                        call_env.insert(name.clone(), global_value.clone());
                        global_sync_keys.insert(name.clone());
                    }
                    if !global_sync_keys.is_empty() {
                        let mut sync_names = global_sync_keys.iter().cloned().collect::<Vec<_>>();
                        sync_names.sort();
                        call_env.insert(
                            INTERNAL_GLOBAL_SYNC_NAMES_KEY.to_string(),
                            Self::new_array_value(sync_names.into_iter().map(Value::String).collect()),
                        );
                    }
                    let mut global_values_before_call = HashMap::new();
                    for name in &global_sync_keys {
                        if let Some(value) = this.script_runtime.env.get(name).cloned() {
                            global_values_before_call.insert(name.clone(), value);
                        }
                    }
                    let mut call_event = event.clone();
                    let event_param = None;
                    this.bind_handler_params(
                        &function.handler,
                        args,
                        &mut call_env,
                        &event_param,
                        &call_event,
                    )?;
                    let flow = this.execute_stmts(
                        &function.handler.stmts,
                        &event_param,
                        &mut call_event,
                        &mut call_env,
                    )?;
                    for name in &global_sync_keys {
                        if Self::is_internal_env_key(name) || function.local_bindings.contains(name) {
                            continue;
                        }
                        let before = global_values_before_call.get(name);
                        let global_after = this.script_runtime.env.get(name).cloned();
                        let call_after = call_env.get(name).cloned();
                        let global_changed = match (before, global_after.as_ref()) {
                            (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                            (None, Some(_)) => true,
                            (Some(_), None) => true,
                            (None, None) => false,
                        };
                        let call_changed = match (before, call_after.as_ref()) {
                            (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                            (None, Some(_)) => true,
                            (Some(_), None) => true,
                            (None, None) => false,
                        };
                        if global_changed && !call_changed {
                            continue;
                        }
                        if let Some(next) = call_after {
                            this.script_runtime.env.insert(name.clone(), next);
                        }
                    }
                    if !function.global_scope {
                        let mut captured_env = function.captured_env.borrow_mut();
                        for name in captured_env_before_call.keys() {
                            if Self::is_internal_env_key(name)
                                || function.local_bindings.contains(name.as_str())
                            {
                                continue;
                            }
                            let before = captured_env_before_call.get(name);
                            let after = call_env.get(name);
                            let changed = match (before, after) {
                                (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                                (None, Some(_)) => true,
                                (Some(_), None) => true,
                                (None, None) => false,
                            };
                            if !changed {
                                continue;
                            }
                            if let Some(next) = after.cloned() {
                                captured_env.insert(name.clone(), next);
                            } else {
                                captured_env.remove(name);
                            }
                        }
                    }
                    match flow {
                        ExecFlow::Continue => Ok(Value::Undefined),
                        ExecFlow::Break => Err(Error::ScriptRuntime(
                            "break statement outside of loop".into(),
                        )),
                        ExecFlow::ContinueLoop => Err(Error::ScriptRuntime(
                            "continue statement outside of loop".into(),
                        )),
                        ExecFlow::Return => Ok(call_env
                            .remove(INTERNAL_RETURN_SLOT)
                            .unwrap_or(Value::Undefined)),
                    }
                })();

                this.restore_pending_function_decl_scopes(pending_scope_start);
                result
            };

        if function.is_async {
            let promise = self.new_pending_promise();
            match run(self, caller_env) {
                Ok(value) => {
                    if let Err(err) = self.promise_resolve(&promise, value) {
                        self.promise_reject(&promise, Self::promise_error_reason(err));
                    }
                }
                Err(err) => self.promise_reject(&promise, Self::promise_error_reason(err)),
            }
            Ok(Value::Promise(promise))
        } else {
            run(self, caller_env)
        }
    }

    pub(super) fn error_to_catch_value(err: Error) -> std::result::Result<Value, Error> {
        match err {
            Error::ScriptThrown(value) => Ok(value.into_value()),
            Error::ScriptRuntime(message) => Ok(Value::String(message)),
            other => Err(other),
        }
    }

    pub(super) fn bind_catch_binding(
        &self,
        binding: &CatchBinding,
        caught: &Value,
        env: &mut HashMap<String, Value>,
    ) -> Result<Vec<(String, Option<Value>)>> {
        let mut previous = Vec::new();
        let mut seen = HashSet::new();
        let mut remember = |name: &str, env: &HashMap<String, Value>| {
            if seen.insert(name.to_string()) {
                previous.push((name.to_string(), env.get(name).cloned()));
            }
        };

        match binding {
            CatchBinding::Identifier(name) => {
                remember(name, env);
                env.insert(name.clone(), caught.clone());
            }
            CatchBinding::ArrayPattern(pattern) => {
                let values = self.array_like_values_from_value(caught)?;
                for (index, name) in pattern.iter().enumerate() {
                    let Some(name) = name else {
                        continue;
                    };
                    remember(name, env);
                    let value = values.get(index).cloned().unwrap_or(Value::Undefined);
                    env.insert(name.clone(), value);
                }
            }
            CatchBinding::ObjectPattern(pattern) => {
                let Value::Object(entries) = caught else {
                    return Err(Error::ScriptRuntime(
                        "catch object binding requires an object value".into(),
                    ));
                };
                let entries = entries.borrow();
                for (source_key, target_name) in pattern {
                    remember(target_name, env);
                    let value =
                        Self::object_get_entry(&entries, source_key).unwrap_or(Value::Undefined);
                    env.insert(target_name.clone(), value);
                }
            }
        }

        Ok(previous)
    }

    pub(super) fn restore_catch_binding(
        &self,
        previous: Vec<(String, Option<Value>)>,
        env: &mut HashMap<String, Value>,
    ) {
        for (name, value) in previous {
            if let Some(value) = value {
                env.insert(name, value);
            } else {
                env.remove(&name);
            }
        }
    }

    pub(super) fn execute_catch_block(
        &mut self,
        catch_binding: &Option<CatchBinding>,
        catch_stmts: &[Stmt],
        caught: Value,
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        let previous = if let Some(binding) = catch_binding {
            self.bind_catch_binding(binding, &caught, env)?
        } else {
            Vec::new()
        };
        let result = self.execute_stmts(catch_stmts, event_param, event, env);
        self.restore_catch_binding(previous, env);
        result
    }

    pub(super) fn parse_function_constructor_param_names(spec: &str) -> Result<Vec<String>> {
        let mut params = Vec::new();
        for raw in spec.split(',') {
            let raw = raw.trim();
            if raw.is_empty() {
                return Err(Error::ScriptRuntime(
                    "new Function parameter name cannot be empty".into(),
                ));
            }
            if !is_ident(raw) {
                return Err(Error::ScriptRuntime(format!(
                    "new Function parameter name is invalid: {raw}"
                )));
            }
            params.push(raw.to_string());
        }
        Ok(params)
    }

    fn collect_function_decls(stmts: &[Stmt]) -> HashMap<String, (ScriptHandler, bool)> {
        let mut out = HashMap::new();
        for stmt in stmts {
            if let Stmt::FunctionDecl {
                name,
                handler,
                is_async,
            } = stmt
            {
                out.insert(name.clone(), (handler.clone(), *is_async));
            }
        }
        out
    }

    fn collect_function_scope_bindings(handler: &ScriptHandler) -> HashSet<String> {
        let mut bindings = HashSet::new();
        for param in &handler.params {
            bindings.insert(param.name.clone());
        }
        Self::collect_scope_bindings_from_stmts(&handler.stmts, &mut bindings);
        bindings
    }

    fn collect_scope_bindings_from_stmts(stmts: &[Stmt], out: &mut HashSet<String>) {
        for stmt in stmts {
            Self::collect_scope_bindings_from_stmt(stmt, out);
        }
    }

    fn collect_scope_bindings_from_stmt(stmt: &Stmt, out: &mut HashSet<String>) {
        match stmt {
            Stmt::VarDecl { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::FunctionDecl { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::ForEach {
                item_var,
                index_var,
                body,
                ..
            }
            | Stmt::ClassListForEach {
                item_var,
                index_var,
                body,
                ..
            } => {
                out.insert(item_var.clone());
                if let Some(index_var) = index_var {
                    out.insert(index_var.clone());
                }
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::For { init, body, .. } => {
                if let Some(init) = init {
                    Self::collect_scope_bindings_from_stmt(init, out);
                }
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::ForIn { item_var, body, .. } | Stmt::ForOf { item_var, body, .. } => {
                out.insert(item_var.clone());
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::DoWhile { body, .. } | Stmt::While { body, .. } => {
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::Try {
                try_stmts,
                catch_binding,
                catch_stmts,
                finally_stmts,
            } => {
                Self::collect_scope_bindings_from_stmts(try_stmts, out);
                if let Some(catch_binding) = catch_binding {
                    Self::collect_scope_bindings_from_catch_binding(catch_binding, out);
                }
                if let Some(catch_stmts) = catch_stmts {
                    Self::collect_scope_bindings_from_stmts(catch_stmts, out);
                }
                if let Some(finally_stmts) = finally_stmts {
                    Self::collect_scope_bindings_from_stmts(finally_stmts, out);
                }
            }
            Stmt::If {
                then_stmts,
                else_stmts,
                ..
            } => {
                Self::collect_scope_bindings_from_stmts(then_stmts, out);
                Self::collect_scope_bindings_from_stmts(else_stmts, out);
            }
            _ => {}
        }
    }

    fn collect_scope_bindings_from_catch_binding(
        binding: &CatchBinding,
        out: &mut HashSet<String>,
    ) {
        match binding {
            CatchBinding::Identifier(name) => {
                out.insert(name.clone());
            }
            CatchBinding::ArrayPattern(pattern) => {
                for entry in pattern.iter().flatten() {
                    out.insert(entry.clone());
                }
            }
            CatchBinding::ObjectPattern(pattern) => {
                for (_, target) in pattern {
                    out.insert(target.clone());
                }
            }
        }
    }

    pub(super) fn resolve_pending_function_decl(
        &mut self,
        name: &str,
        env: &HashMap<String, Value>,
    ) -> Option<Value> {
        let mut resolved = None;
        for scope in self.script_runtime.pending_function_decls.iter().rev() {
            let Some((handler, is_async)) = scope.get(name) else {
                continue;
            };
            resolved = Some((handler.clone(), *is_async));
            break;
        }
        let (handler, is_async) = resolved?;
        Some(self.make_function_value(handler, env, false, is_async))
    }

    fn sync_listener_capture_env_if_shared(&mut self, env: &HashMap<String, Value>) {
        let Some(frame) = self.script_runtime.listener_capture_env_stack.last() else {
            return;
        };
        let Some(shared_env) = frame.shared_env.as_ref() else {
            return;
        };
        if Rc::strong_count(shared_env) > 1 {
            *shared_env.borrow_mut() = ScriptEnv::from_snapshot(env);
        }
    }

    pub(super) fn execute_stmts(
        &mut self,
        stmts: &[Stmt],
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        let pending = Self::collect_function_decls(stmts);
        let pending_scope_start = self.push_pending_function_decl_scope(pending);
        self.script_runtime
            .listener_capture_env_stack
            .push(ListenerCaptureFrame::default());

        let result = (|| -> Result<ExecFlow> {
            for stmt in stmts {
                self.sync_listener_capture_env_if_shared(env);
                match stmt {
                Stmt::VarDecl { name, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    env.insert(name.clone(), value.clone());
                    self.bind_timer_id_to_task_env(name, expr, &value);
                }
                Stmt::FunctionDecl {
                    name,
                    handler,
                    is_async,
                } => {
                    let function = self.make_function_value(handler.clone(), env, false, *is_async);
                    env.insert(name.clone(), function);
                }
                Stmt::VarAssign { name, op, expr } => {
                    let previous = env
                        .get(name)
                        .cloned()
                        .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;

                    let next = match op {
                        VarAssignOp::Assign => self.eval_expr(expr, env, event_param, event)?,
                        VarAssignOp::Add => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.add_values(&previous, &value)?
                        }
                        VarAssignOp::Sub => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::Sub, &previous, &value)?
                        }
                        VarAssignOp::Mul => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::Mul, &previous, &value)?
                        }
                        VarAssignOp::Pow => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::Pow, &previous, &value)?
                        }
                        VarAssignOp::BitOr => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::BitOr, &previous, &value)?
                        }
                        VarAssignOp::BitXor => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::BitXor, &previous, &value)?
                        }
                        VarAssignOp::BitAnd => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::BitAnd, &previous, &value)?
                        }
                        VarAssignOp::ShiftLeft => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::ShiftLeft, &previous, &value)?
                        }
                        VarAssignOp::ShiftRight => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::ShiftRight, &previous, &value)?
                        }
                        VarAssignOp::UnsignedShiftRight => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::UnsignedShiftRight, &previous, &value)?
                        }
                        VarAssignOp::Div => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::Div, &previous, &value)?
                        }
                        VarAssignOp::Mod => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            self.eval_binary(&BinaryOp::Mod, &previous, &value)?
                        }
                        VarAssignOp::LogicalAnd => {
                            if previous.truthy() {
                                self.eval_expr(expr, env, event_param, event)?
                            } else {
                                previous.clone()
                            }
                        }
                        VarAssignOp::LogicalOr => {
                            if previous.truthy() {
                                previous.clone()
                            } else {
                                self.eval_expr(expr, env, event_param, event)?
                            }
                        }
                        VarAssignOp::Nullish => {
                            if matches!(&previous, Value::Null | Value::Undefined) {
                                self.eval_expr(expr, env, event_param, event)?
                            } else {
                                previous.clone()
                            }
                        }
                    };
                    env.insert(name.clone(), next.clone());
                    self.sync_global_binding_if_needed(env, name, &next);
                    self.bind_timer_id_to_task_env(name, expr, &next);
                }
                Stmt::VarUpdate { name, delta } => {
                    let previous = env
                        .get(name)
                        .cloned()
                        .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;
                    let next = match previous {
                        Value::Number(value) => {
                            if let Some(sum) = value.checked_add(i64::from(*delta)) {
                                Value::Number(sum)
                            } else {
                                Value::Float((value as f64) + f64::from(*delta))
                            }
                        }
                        Value::Float(value) => Value::Float(value + f64::from(*delta)),
                        Value::BigInt(value) => Value::BigInt(value + JsBigInt::from(*delta)),
                        _ => {
                            return Err(Error::ScriptRuntime(format!(
                                "cannot apply update operator to '{}'",
                                name
                            )));
                        }
                    };
                    env.insert(name.clone(), next.clone());
                    self.sync_global_binding_if_needed(env, name, &next);
                }
                Stmt::ArrayDestructureAssign { targets, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    let values = self.array_like_values_from_value(&value)?;
                    for (index, target_name) in targets.iter().enumerate() {
                        let Some(target_name) = target_name else {
                            continue;
                        };
                        let next = values.get(index).cloned().unwrap_or(Value::Undefined);
                        env.insert(target_name.clone(), next.clone());
                        self.sync_global_binding_if_needed(env, target_name, &next);
                    }
                }
                Stmt::ObjectDestructureAssign { bindings, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    let Value::Object(entries) = value else {
                        return Err(Error::ScriptRuntime(
                            "object destructuring source must be an object".into(),
                        ));
                    };
                    let entries = entries.borrow();
                    for (source_key, target_name) in bindings {
                        let next = Self::object_get_entry(&entries, source_key)
                            .unwrap_or(Value::Undefined);
                        env.insert(target_name.clone(), next.clone());
                        self.sync_global_binding_if_needed(env, target_name, &next);
                    }
                }
                Stmt::ObjectAssign { target, path, expr } => {
                    self.execute_object_assignment_stmt(
                        target,
                        path,
                        expr,
                        env,
                        event_param,
                        event,
                    )?;
                }
                Stmt::FormDataAppend {
                    target_var,
                    name,
                    value,
                } => {
                    let name = self.eval_expr(name, env, event_param, event)?;
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let name = name.as_string();
                    let value = value.as_string();
                    let target = env.get_mut(target_var).ok_or_else(|| {
                        Error::ScriptRuntime(format!("unknown FormData variable: {}", target_var))
                    })?;
                    match target {
                        Value::FormData(entries) => {
                            entries.push((name, value));
                        }
                        Value::Object(entries) => {
                            if !Self::is_url_search_params_object(&entries.borrow()) {
                                return Err(Error::ScriptRuntime(format!(
                                    "variable '{}' is not a FormData instance",
                                    target_var
                                )));
                            }
                            {
                                let mut object_ref = entries.borrow_mut();
                                let mut pairs =
                                    Self::url_search_params_pairs_from_object_entries(&object_ref);
                                pairs.push((name, value));
                                Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                            }
                            self.sync_url_search_params_owner(entries);
                        }
                        _ => {
                            return Err(Error::ScriptRuntime(format!(
                                "variable '{}' is not a FormData instance",
                                target_var
                            )));
                        }
                    }
                }
                Stmt::DomAssign { target, prop, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    if let DomQuery::Var(name) = target {
                        if let Some(Value::Object(entries)) = env.get(name) {
                            if let Some(key) = Self::object_key_from_dom_prop(prop) {
                                Self::object_set_entry(
                                    &mut entries.borrow_mut(),
                                    key.to_string(),
                                    value,
                                );
                                continue;
                            }
                        }
                    }
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match prop {
                        DomProp::TextContent => {
                            self.dom.set_text_content(node, &value.as_string())?
                        }
                        DomProp::InnerText => {
                            self.dom.set_text_content(node, &value.as_string())?
                        }
                        DomProp::InnerHtml => self.dom.set_inner_html(node, &value.as_string())?,
                        DomProp::OuterHtml => self.dom.set_outer_html(node, &value.as_string())?,
                        DomProp::Value => self.dom.set_value(node, &value.as_string())?,
                        DomProp::SelectionStart => {
                            let next_start = Self::value_to_i64(&value).max(0) as usize;
                            let end = self.dom.selection_end(node).unwrap_or_default();
                            self.dom
                                .set_selection_range(node, next_start, end, "none")?;
                        }
                        DomProp::SelectionEnd => {
                            let start = self.dom.selection_start(node).unwrap_or_default();
                            let next_end = Self::value_to_i64(&value).max(0) as usize;
                            self.dom
                                .set_selection_range(node, start, next_end, "none")?;
                        }
                        DomProp::SelectionDirection => {
                            let start = self.dom.selection_start(node).unwrap_or_default();
                            let end = self.dom.selection_end(node).unwrap_or_default();
                            let direction = value.as_string();
                            let direction =
                                Self::normalize_selection_direction(direction.as_str());
                            self.dom.set_selection_range(node, start, end, direction)?;
                        }
                        DomProp::Checked => self.dom.set_checked(node, value.truthy())?,
                        DomProp::Indeterminate => {
                            self.dom.set_indeterminate(node, value.truthy())?
                        }
                        DomProp::Open => {
                            if value.truthy() {
                                self.dom.set_attr(node, "open", "true")?;
                            } else {
                                self.dom.remove_attr(node, "open")?;
                            }
                        }
                        DomProp::ReturnValue => {
                            self.set_dialog_return_value(node, value.as_string())?;
                        }
                        DomProp::ClosedBy => {
                            self.dom.set_attr(node, "closedby", &value.as_string())?
                        }
                        DomProp::Readonly => {
                            if value.truthy() {
                                self.dom.set_attr(node, "readonly", "true")?;
                            } else {
                                self.dom.remove_attr(node, "readonly")?;
                            }
                        }
                        DomProp::Required => {
                            if value.truthy() {
                                self.dom.set_attr(node, "required", "true")?;
                            } else {
                                self.dom.remove_attr(node, "required")?;
                            }
                        }
                        DomProp::Disabled => {
                            if value.truthy() {
                                self.dom.set_attr(node, "disabled", "true")?;
                            } else {
                                self.dom.remove_attr(node, "disabled")?;
                            }
                        }
                        DomProp::Hidden => {
                            if node == self.dom.root {
                                let call = self.describe_dom_prop(prop);
                                return Err(Error::ScriptRuntime(format!("{call} is read-only")));
                            }
                            if value.truthy() {
                                self.dom.set_attr(node, "hidden", "true")?;
                            } else {
                                self.dom.remove_attr(node, "hidden")?;
                            }
                        }
                        DomProp::ClassName => {
                            self.dom.set_attr(node, "class", &value.as_string())?
                        }
                        DomProp::Id => self.dom.set_attr(node, "id", &value.as_string())?,
                        DomProp::Slot => self.dom.set_attr(node, "slot", &value.as_string())?,
                        DomProp::Role => self.dom.set_attr(node, "role", &value.as_string())?,
                        DomProp::ElementTiming => {
                            self.dom.set_attr(node, "elementtiming", &value.as_string())?
                        }
                        DomProp::Name => self.dom.set_attr(node, "name", &value.as_string())?,
                        DomProp::Lang => self.dom.set_attr(node, "lang", &value.as_string())?,
                        DomProp::Title => self.dom.set_document_title(&value.as_string())?,
                        DomProp::Location | DomProp::LocationHref => self.navigate_location(
                            &value.as_string(),
                            LocationNavigationKind::HrefSet,
                        )?,
                        DomProp::LocationProtocol => {
                            self.set_location_property("protocol", value.clone())?
                        }
                        DomProp::LocationHost => {
                            self.set_location_property("host", value.clone())?
                        }
                        DomProp::LocationHostname => {
                            self.set_location_property("hostname", value.clone())?
                        }
                        DomProp::LocationPort => {
                            self.set_location_property("port", value.clone())?
                        }
                        DomProp::LocationPathname => {
                            self.set_location_property("pathname", value.clone())?
                        }
                        DomProp::LocationSearch => {
                            self.set_location_property("search", value.clone())?
                        }
                        DomProp::LocationHash => {
                            self.set_location_property("hash", value.clone())?
                        }
                        DomProp::HistoryScrollRestoration => {
                            self.set_history_property("scrollRestoration", value.clone())?
                        }
                        DomProp::AnchorAttributionSrc => {
                            self.dom
                                .set_attr(node, "attributionsrc", &value.as_string())?
                        }
                        DomProp::AnchorDownload => {
                            self.dom.set_attr(node, "download", &value.as_string())?
                        }
                        DomProp::AnchorHash => {
                            self.set_anchor_url_property(node, "hash", value.clone())?
                        }
                        DomProp::AnchorHost => {
                            self.set_anchor_url_property(node, "host", value.clone())?
                        }
                        DomProp::AnchorHostname => {
                            self.set_anchor_url_property(node, "hostname", value.clone())?
                        }
                        DomProp::AnchorHref => {
                            self.set_anchor_url_property(node, "href", value.clone())?
                        }
                        DomProp::AnchorHreflang => {
                            self.dom.set_attr(node, "hreflang", &value.as_string())?
                        }
                        DomProp::AnchorInterestForElement => {
                            self.dom.set_attr(node, "interestfor", &value.as_string())?
                        }
                        DomProp::AnchorPassword => {
                            self.set_anchor_url_property(node, "password", value.clone())?
                        }
                        DomProp::AnchorPathname => {
                            self.set_anchor_url_property(node, "pathname", value.clone())?
                        }
                        DomProp::AnchorPing => {
                            self.dom.set_attr(node, "ping", &value.as_string())?
                        }
                        DomProp::AnchorPort => {
                            self.set_anchor_url_property(node, "port", value.clone())?
                        }
                        DomProp::AnchorProtocol => {
                            self.set_anchor_url_property(node, "protocol", value.clone())?
                        }
                        DomProp::AnchorReferrerPolicy => {
                            self.dom
                                .set_attr(node, "referrerpolicy", &value.as_string())?
                        }
                        DomProp::AnchorRel => self.dom.set_attr(node, "rel", &value.as_string())?,
                        DomProp::AnchorSearch => {
                            self.set_anchor_url_property(node, "search", value.clone())?
                        }
                        DomProp::AnchorTarget => {
                            self.dom.set_attr(node, "target", &value.as_string())?
                        }
                        DomProp::AnchorText => {
                            self.dom.set_text_content(node, &value.as_string())?
                        }
                        DomProp::AnchorType => {
                            self.dom.set_attr(node, "type", &value.as_string())?
                        }
                        DomProp::AnchorUsername => {
                            self.set_anchor_url_property(node, "username", value.clone())?
                        }
                        DomProp::AnchorCharset => {
                            self.dom.set_attr(node, "charset", &value.as_string())?
                        }
                        DomProp::AnchorCoords => {
                            self.dom.set_attr(node, "coords", &value.as_string())?
                        }
                        DomProp::AnchorRev => self.dom.set_attr(node, "rev", &value.as_string())?,
                        DomProp::AnchorShape => {
                            self.dom.set_attr(node, "shape", &value.as_string())?
                        }
                        DomProp::AriaString(prop_name) => {
                            let attr_name = Self::aria_property_to_attr_name(prop_name);
                            self.dom.set_attr(node, &attr_name, &value.as_string())?
                        }
                        DomProp::Attributes
                        | DomProp::AssignedSlot
                        | DomProp::ValidationMessage
                        | DomProp::Validity
                        | DomProp::ValidityValueMissing
                        | DomProp::ValidityTypeMismatch
                        | DomProp::ValidityPatternMismatch
                        | DomProp::ValidityTooLong
                        | DomProp::ValidityTooShort
                        | DomProp::ValidityRangeUnderflow
                        | DomProp::ValidityRangeOverflow
                        | DomProp::ValidityStepMismatch
                        | DomProp::ValidityBadInput
                        | DomProp::ValidityValid
                        | DomProp::ValidityCustomError
                        | DomProp::ClassList
                        | DomProp::ClassListLength
                        | DomProp::Part
                        | DomProp::PartLength
                        | DomProp::TagName
                        | DomProp::LocalName
                        | DomProp::NamespaceUri
                        | DomProp::Prefix
                        | DomProp::NextElementSibling
                        | DomProp::PreviousElementSibling
                        | DomProp::ClientWidth
                        | DomProp::ClientHeight
                        | DomProp::ClientLeft
                        | DomProp::ClientTop
                        | DomProp::CurrentCssZoom
                        | DomProp::ScrollLeftMax
                        | DomProp::ScrollTopMax
                        | DomProp::ShadowRoot
                        | DomProp::AriaElementRefSingle(_)
                        | DomProp::AriaElementRefList(_)
                        | DomProp::OffsetWidth
                        | DomProp::ValueLength
                        | DomProp::OffsetHeight
                        | DomProp::OffsetLeft
                        | DomProp::OffsetTop
                        | DomProp::ScrollWidth
                        | DomProp::ScrollHeight
                        | DomProp::ScrollLeft
                        | DomProp::ScrollTop
                        | DomProp::ActiveElement
                        | DomProp::CharacterSet
                        | DomProp::CompatMode
                        | DomProp::ContentType
                        | DomProp::ReadyState
                        | DomProp::Referrer
                        | DomProp::Url
                        | DomProp::DocumentUri
                        | DomProp::LocationOrigin
                        | DomProp::LocationAncestorOrigins
                        | DomProp::History
                        | DomProp::HistoryLength
                        | DomProp::HistoryState
                        | DomProp::DefaultView
                        | DomProp::VisibilityState
                        | DomProp::Forms
                        | DomProp::Images
                        | DomProp::Links
                        | DomProp::Scripts
                        | DomProp::Children
                        | DomProp::ChildElementCount
                        | DomProp::FirstElementChild
                        | DomProp::LastElementChild
                        | DomProp::CurrentScript
                        | DomProp::FormsLength
                        | DomProp::ImagesLength
                        | DomProp::LinksLength
                        | DomProp::ScriptsLength
                        | DomProp::ChildrenLength
                        | DomProp::AnchorOrigin
                        | DomProp::AnchorRelList
                        | DomProp::AnchorRelListLength => {
                            let call = self.describe_dom_prop(prop);
                            return Err(Error::ScriptRuntime(format!("{call} is read-only")));
                        }
                        DomProp::Dataset(key) => {
                            self.dom.dataset_set(node, key, &value.as_string())?
                        }
                        DomProp::Style(prop) => {
                            self.dom.style_set(node, prop, &value.as_string())?
                        }
                    }
                }
                Stmt::ClassListCall {
                    target,
                    method,
                    class_names,
                    force,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match method {
                        ClassListMethod::Add => {
                            for class_name in class_names {
                                self.dom.class_add(node, class_name)?;
                            }
                        }
                        ClassListMethod::Remove => {
                            for class_name in class_names {
                                self.dom.class_remove(node, class_name)?;
                            }
                        }
                        ClassListMethod::Toggle => {
                            let class_name = class_names.first().ok_or_else(|| {
                                Error::ScriptRuntime("toggle requires a class name".into())
                            })?;
                            if let Some(force_expr) = force {
                                let force_value = self
                                    .eval_expr(force_expr, env, event_param, event)?
                                    .truthy();
                                if force_value {
                                    self.dom.class_add(node, class_name)?;
                                } else {
                                    self.dom.class_remove(node, class_name)?;
                                }
                            } else {
                                let _ = self.dom.class_toggle(node, class_name)?;
                            }
                        }
                    }
                }
                Stmt::ClassListForEach {
                    target,
                    item_var,
                    index_var,
                    body,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let classes = class_tokens(self.dom.attr(node, "class").as_deref());
                    let prev_item = env.get(item_var).cloned();
                    let prev_index = index_var.as_ref().and_then(|v| env.get(v).cloned());

                    for (idx, class_name) in classes.iter().enumerate() {
                        let item_value = Value::String(class_name.clone());
                        env.insert(item_var.clone(), item_value.clone());
                        self.sync_global_binding_if_needed(env, item_var, &item_value);
                        if let Some(index_var) = index_var {
                            let index_value = Value::Number(idx as i64);
                            env.insert(index_var.clone(), index_value.clone());
                            self.sync_global_binding_if_needed(env, index_var, &index_value);
                        }
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::Break => break,
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }

                    if let Some(prev) = prev_item {
                        env.insert(item_var.clone(), prev.clone());
                        self.sync_global_binding_if_needed(env, item_var, &prev);
                    } else {
                        env.remove(item_var);
                    }
                    if let Some(index_var) = index_var {
                        if let Some(prev) = prev_index {
                            env.insert(index_var.clone(), prev.clone());
                            self.sync_global_binding_if_needed(env, index_var, &prev);
                        } else {
                            env.remove(index_var);
                        }
                    }
                }
                Stmt::DomSetAttribute {
                    target,
                    name,
                    value,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let value = self.eval_expr(value, env, event_param, event)?;
                    self.dom.set_attr(node, name, &value.as_string())?;
                }
                Stmt::DomRemoveAttribute { target, name } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    self.dom.remove_attr(node, name)?;
                }
                Stmt::NodeTreeMutation {
                    target,
                    method,
                    child,
                    reference,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let child = self.eval_expr(child, env, event_param, event)?;
                    let Value::Node(child) = child else {
                        return Err(Error::ScriptRuntime(
                            "before/after/replaceWith/append/appendChild/prepend/removeChild/insertBefore argument must be an element reference".into(),
                        ));
                    };
                    match method {
                        NodeTreeMethod::After => self.dom.insert_after(target_node, child)?,
                        NodeTreeMethod::Append => self.dom.append_child(target_node, child)?,
                        NodeTreeMethod::AppendChild => self.dom.append_child(target_node, child)?,
                        NodeTreeMethod::Before => {
                            let Some(parent) = self.dom.parent(target_node) else {
                                continue;
                            };
                            self.dom.insert_before(parent, child, target_node)?;
                        }
                        NodeTreeMethod::ReplaceWith => {
                            self.dom.replace_with(target_node, child)?;
                        }
                        NodeTreeMethod::Prepend => self.dom.prepend_child(target_node, child)?,
                        NodeTreeMethod::RemoveChild => self.dom.remove_child(target_node, child)?,
                        NodeTreeMethod::InsertBefore => {
                            let Some(reference) = reference else {
                                return Err(Error::ScriptRuntime(
                                    "insertBefore requires reference node".into(),
                                ));
                            };
                            let reference = self.eval_expr(reference, env, event_param, event)?;
                            let Value::Node(reference) = reference else {
                                return Err(Error::ScriptRuntime(
                                    "insertBefore reference must be an element reference".into(),
                                ));
                            };
                            self.dom.insert_before(target_node, child, reference)?;
                        }
                    }
                }
                Stmt::InsertAdjacentElement {
                    target,
                    position,
                    node,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let node = self.eval_expr(node, env, event_param, event)?;
                    let Value::Node(node) = node else {
                        return Err(Error::ScriptRuntime(
                            "insertAdjacentElement second argument must be an element reference"
                                .into(),
                        ));
                    };
                    self.dom
                        .insert_adjacent_node(target_node, *position, node)?;
                }
                Stmt::InsertAdjacentText {
                    target,
                    position,
                    text,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let text = self.eval_expr(text, env, event_param, event)?;
                    if matches!(
                        position,
                        InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
                    ) && self.dom.parent(target_node).is_none()
                    {
                        continue;
                    }
                    let text_node = self.dom.create_detached_text(text.as_string());
                    self.dom
                        .insert_adjacent_node(target_node, *position, text_node)?;
                }
                Stmt::InsertAdjacentHTML {
                    target,
                    position,
                    html,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let position = self.eval_expr(position, env, event_param, event)?;
                    let position = resolve_insert_adjacent_position(&position.as_string())?;
                    let html = self.eval_expr(html, env, event_param, event)?;
                    self.dom
                        .insert_adjacent_html(target_node, position, &html.as_string())?;
                }
                Stmt::SetTimeout { handler, delay_ms } => {
                    let delay = self.eval_expr(delay_ms, env, event_param, event)?;
                    let delay = Self::value_to_i64(&delay);
                    let callback_args = handler
                        .args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    let _ =
                        self.schedule_timeout(handler.callback.clone(), delay, callback_args, env);
                }
                Stmt::SetInterval { handler, delay_ms } => {
                    let interval = self.eval_expr(delay_ms, env, event_param, event)?;
                    let interval = Self::value_to_i64(&interval);
                    let callback_args = handler
                        .args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    let _ = self.schedule_interval(
                        handler.callback.clone(),
                        interval,
                        callback_args,
                        env,
                    );
                }
                Stmt::QueueMicrotask { handler } => {
                    self.queue_microtask(handler.clone(), env);
                }
                Stmt::ClearTimeout { timer_id } => {
                    let timer_id = self.eval_expr(timer_id, env, event_param, event)?;
                    let timer_id = Self::value_to_i64(&timer_id);
                    self.clear_timeout(timer_id);
                }
                Stmt::NodeRemove { target } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    if let Some(active) = self.dom.active_element() {
                        if active == node || self.dom.is_descendant_of(active, node) {
                            self.dom.set_active_element(None);
                        }
                    }
                    if let Some(active_pseudo) = self.dom.active_pseudo_element() {
                        if active_pseudo == node || self.dom.is_descendant_of(active_pseudo, node) {
                            self.dom.set_active_pseudo_element(None);
                        }
                    }
                    self.dom.remove_node(node)?;
                }
                Stmt::ForEach {
                    target,
                    selector,
                    item_var,
                    index_var,
                    body,
                } => {
                    let items = if let Some(target) = target {
                        match self.resolve_dom_query_runtime(target, env)? {
                            Some(target_node) => {
                                self.dom.query_selector_all_from(&target_node, selector)?
                            }
                            None => Vec::new(),
                        }
                    } else {
                        self.dom.query_selector_all(selector)?
                    };
                    let prev_item = env.get(item_var).cloned();
                    let prev_index = index_var.as_ref().and_then(|v| env.get(v).cloned());

                    for (idx, node) in items.iter().enumerate() {
                        let item_value = Value::Node(*node);
                        env.insert(item_var.clone(), item_value.clone());
                        self.sync_global_binding_if_needed(env, item_var, &item_value);
                        if let Some(index_var) = index_var {
                            let index_value = Value::Number(idx as i64);
                            env.insert(index_var.clone(), index_value.clone());
                            self.sync_global_binding_if_needed(env, index_var, &index_value);
                        }
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::Break => break,
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }

                    if let Some(prev) = prev_item {
                        env.insert(item_var.clone(), prev.clone());
                        self.sync_global_binding_if_needed(env, item_var, &prev);
                    } else {
                        env.remove(item_var);
                    }
                    if let Some(index_var) = index_var {
                        if let Some(prev) = prev_index {
                            env.insert(index_var.clone(), prev.clone());
                            self.sync_global_binding_if_needed(env, index_var, &prev);
                        } else {
                            env.remove(index_var);
                        }
                    }
                }
                Stmt::ArrayForEach { target, callback } => {
                    let target_value = env
                        .get(target)
                        .cloned()
                        .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {target}")))?;
                    self.execute_array_like_foreach_in_env(
                        target_value,
                        callback,
                        env,
                        event,
                        target,
                    )?;
                }
                Stmt::ArrayForEachExpr { target, callback } => {
                    let target_value = self.eval_expr(target, env, event_param, event)?;
                    self.execute_array_like_foreach_in_env(
                        target_value,
                        callback,
                        env,
                        event,
                        "<expression>",
                    )?;
                }
                Stmt::For {
                    init,
                    cond,
                    post,
                    body,
                } => {
                    if let Some(init) = init.as_deref() {
                        match self.execute_stmts(
                            std::slice::from_ref(init),
                            event_param,
                            event,
                            env,
                        )? {
                            ExecFlow::Continue => {}
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                            ExecFlow::Break => {
                                return Err(Error::ScriptRuntime(
                                    "break statement outside of loop".into(),
                                ));
                            }
                            ExecFlow::ContinueLoop => {
                                return Err(Error::ScriptRuntime(
                                    "continue statement outside of loop".into(),
                                ));
                            }
                        }
                    }

                    loop {
                        let should_run = if let Some(cond) = cond {
                            self.eval_expr(cond, env, event_param, event)?.truthy()
                        } else {
                            true
                        };
                        if !should_run {
                            break;
                        }

                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => {
                                if let Some(post) = post.as_deref() {
                                    match self.execute_stmts(
                                        std::slice::from_ref(post),
                                        event_param,
                                        event,
                                        env,
                                    )? {
                                        ExecFlow::Continue => {}
                                        ExecFlow::Return => return Ok(ExecFlow::Return),
                                        ExecFlow::Break | ExecFlow::ContinueLoop => {
                                            return Err(Error::ScriptRuntime(
                                                "invalid loop control in post expression".into(),
                                            ));
                                        }
                                    }
                                }
                                continue;
                            }
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                        if let Some(post) = post.as_deref() {
                            match self.execute_stmts(
                                std::slice::from_ref(post),
                                event_param,
                                event,
                                env,
                            )? {
                                ExecFlow::Continue => {}
                                ExecFlow::Return => return Ok(ExecFlow::Return),
                                ExecFlow::Break | ExecFlow::ContinueLoop => {
                                    return Err(Error::ScriptRuntime(
                                        "invalid loop control in post expression".into(),
                                    ));
                                }
                            }
                        }
                    }
                }
                Stmt::ForIn {
                    item_var,
                    iterable,
                    body,
                } => {
                    let iterable = self.eval_expr(iterable, env, event_param, event)?;
                    let items = match iterable {
                        Value::NodeList(nodes) => (0..nodes.len()).collect::<Vec<_>>(),
                        Value::Array(values) => {
                            let values = values.borrow();
                            (0..values.len()).collect::<Vec<_>>()
                        }
                        Value::Null | Value::Undefined => Vec::new(),
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "for...in iterable must be a NodeList or Array".into(),
                            ));
                        }
                    };

                    let prev_item = env.get(item_var).cloned();
                    for idx in items {
                        let item_value = Value::Number(idx as i64);
                        env.insert(item_var.clone(), item_value.clone());
                        self.sync_global_binding_if_needed(env, item_var, &item_value);
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }
                    if let Some(prev) = prev_item {
                        env.insert(item_var.clone(), prev.clone());
                        self.sync_global_binding_if_needed(env, item_var, &prev);
                    } else {
                        env.remove(item_var);
                    }
                }
                Stmt::ForOf {
                    item_var,
                    iterable,
                    body,
                } => {
                    let iterable = self.eval_expr(iterable, env, event_param, event)?;
                    let nodes = match iterable {
                        Value::NodeList(nodes) => {
                            nodes.into_iter().map(Value::Node).collect::<Vec<_>>()
                        }
                        Value::Array(values) => values.borrow().clone(),
                        Value::Map(map) => self.map_entries_array(&map),
                        Value::Set(set) => set.borrow().values.clone(),
                        Value::Object(entries) => {
                            if Self::is_url_search_params_object(&entries.borrow()) {
                                Self::url_search_params_pairs_from_object_entries(&entries.borrow())
                                    .into_iter()
                                    .map(|(key, value)| {
                                        Self::new_array_value(vec![
                                            Value::String(key),
                                            Value::String(value),
                                        ])
                                    })
                                    .collect::<Vec<_>>()
                            } else {
                                return Err(Error::ScriptRuntime(
                                    "for...of iterable must be a NodeList, Array, Map, Set, or URLSearchParams"
                                        .into(),
                                ));
                            }
                        }
                        Value::Null | Value::Undefined => Vec::new(),
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "for...of iterable must be a NodeList, Array, Map, Set, or URLSearchParams"
                                    .into(),
                            ));
                        }
                    };

                    let prev_item = env.get(item_var).cloned();
                    for item in nodes {
                        env.insert(item_var.clone(), item.clone());
                        self.sync_global_binding_if_needed(env, item_var, &item);
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }
                    if let Some(prev) = prev_item {
                        env.insert(item_var.clone(), prev.clone());
                        self.sync_global_binding_if_needed(env, item_var, &prev);
                    } else {
                        env.remove(item_var);
                    }
                }
                Stmt::While { cond, body } => {
                    while self.eval_expr(cond, env, event_param, event)?.truthy() {
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }
                }
                Stmt::DoWhile { cond, body } => loop {
                    match self.execute_stmts(body, event_param, event, env)? {
                        ExecFlow::Continue => {}
                        ExecFlow::ContinueLoop => {}
                        ExecFlow::Break => break,
                        ExecFlow::Return => return Ok(ExecFlow::Return),
                    }
                    if !self.eval_expr(cond, env, event_param, event)?.truthy() {
                        break;
                    }
                },
                Stmt::If {
                    cond,
                    then_stmts,
                    else_stmts,
                } => {
                    let cond = self.eval_expr(cond, env, event_param, event)?;
                    if cond.truthy() {
                        match self.execute_stmts(then_stmts, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    } else {
                        match self.execute_stmts(else_stmts, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                }
                Stmt::Try {
                    try_stmts,
                    catch_binding,
                    catch_stmts,
                    finally_stmts,
                } => {
                    let mut completion = self.execute_stmts(try_stmts, event_param, event, env);

                    if let Err(err) = completion {
                        if let Some(catch_stmts) = catch_stmts {
                            let caught = Self::error_to_catch_value(err)?;
                            completion = self.execute_catch_block(
                                catch_binding,
                                catch_stmts,
                                caught,
                                event_param,
                                event,
                                env,
                            );
                        } else {
                            completion = Err(err);
                        }
                    }

                    if let Some(finally_stmts) = finally_stmts {
                        match self.execute_stmts(finally_stmts, event_param, event, env) {
                            Ok(ExecFlow::Continue) => {}
                            Ok(flow) => return Ok(flow),
                            Err(err) => return Err(err),
                        }
                    }

                    match completion {
                        Ok(ExecFlow::Continue) => {}
                        Ok(flow) => return Ok(flow),
                        Err(err) => return Err(err),
                    }
                }
                Stmt::Throw { value } => {
                    let thrown = self.eval_expr(value, env, event_param, event)?;
                    return Err(Error::ScriptThrown(ThrownValue::new(thrown)));
                }
                Stmt::Return { value } => {
                    let return_value = if let Some(value) = value {
                        self.eval_expr(value, env, event_param, event)?
                    } else {
                        Value::Undefined
                    };
                    env.insert(INTERNAL_RETURN_SLOT.to_string(), return_value);
                    return Ok(ExecFlow::Return);
                }
                Stmt::Break => {
                    return Ok(ExecFlow::Break);
                }
                Stmt::Continue => {
                    return Ok(ExecFlow::ContinueLoop);
                }
                Stmt::EventCall { event_var, method } => {
                    if let Some(param) = event_param {
                        if param == event_var {
                            match method {
                                EventMethod::PreventDefault => {
                                    if event.cancelable {
                                        event.default_prevented = true;
                                    }
                                }
                                EventMethod::StopPropagation => {
                                    event.propagation_stopped = true;
                                }
                                EventMethod::StopImmediatePropagation => {
                                    event.immediate_propagation_stopped = true;
                                    event.propagation_stopped = true;
                                }
                            }
                        }
                    }
                }
                Stmt::ListenerMutation {
                    target,
                    op,
                    event_type,
                    capture,
                    handler,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match op {
                        ListenerRegistrationOp::Add => {
                            let captured_env = self.ensure_listener_capture_env();
                            *captured_env.borrow_mut() = ScriptEnv::from_snapshot(env);
                            self.listeners.add(
                                node,
                                event_type.clone(),
                                Listener {
                                    capture: *capture,
                                    handler: handler.clone(),
                                    captured_env,
                                    captured_pending_function_decls: self
                                        .script_runtime
                                        .pending_function_decls
                                        .clone(),
                                },
                            );
                        }
                        ListenerRegistrationOp::Remove => {
                            let _ = self.listeners.remove(node, event_type, *capture, handler);
                        }
                    }
                }
                Stmt::DomMethodCall {
                    target,
                    method,
                    arg,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let arg_value = arg
                        .as_ref()
                        .map(|expr| self.eval_expr(expr, env, event_param, event))
                        .transpose()?;
                    match method {
                        DomMethod::Focus => self.focus_node_with_env(node, env)?,
                        DomMethod::Blur => self.blur_node_with_env(node, env)?,
                        DomMethod::Click => self.click_node_with_env(node, env)?,
                        DomMethod::Submit => self.submit_form_with_env(node, env)?,
                        DomMethod::RequestSubmit => {
                            self.request_submit_form_with_env(node, arg_value, env)?
                        }
                        DomMethod::Reset => self.reset_form_with_env(node, env)?,
                        DomMethod::ScrollIntoView => {
                            self.scroll_into_view_node_with_env(node, env)?
                        }
                        DomMethod::Show => self.show_dialog_with_env(node, false, env)?,
                        DomMethod::ShowModal => self.show_dialog_with_env(node, true, env)?,
                        DomMethod::Close => self.close_dialog_with_env(node, arg_value, env)?,
                        DomMethod::RequestClose => {
                            self.request_close_dialog_with_env(node, arg_value, env)?
                        }
                    }
                }
                Stmt::DispatchEvent { target, event_type } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let event_name = self
                        .eval_expr(event_type, env, event_param, event)?
                        .as_string();
                    if event_name.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "dispatchEvent requires non-empty event type".into(),
                        ));
                    }
                    let _ = self.dispatch_event_with_env(node, &event_name, env, false)?;
                }
                Stmt::Expr(expr) => {
                    let _ = self.eval_expr(expr, env, event_param, event)?;
                }
                }
            }

            Ok(ExecFlow::Continue)
        })();

        self.script_runtime.listener_capture_env_stack.pop();
        self.restore_pending_function_decl_scopes(pending_scope_start);
        result
    }

}
