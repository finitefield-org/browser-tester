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

    pub(crate) fn queue_callable_microtask(&mut self, callback: Value) {
        self.scheduler
            .microtask_queue
            .push_back(ScheduledMicrotask::Callable { callback });
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
                    ScheduledMicrotask::Callable { callback } => {
                        this.run_callable_microtask(&callback)?;
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
            this.with_isolated_loop_control_scope(|this| {
                this.bind_handler_params(handler, &event_args, callback_env, &event_param, event)?;
                let flow = this.execute_stmts(&handler.stmts, &event_param, event, callback_env)?;
                callback_env.remove(INTERNAL_RETURN_SLOT);
                match flow {
                    ExecFlow::Continue => Ok(()),
                    ExecFlow::Break(label) => Err(Self::break_flow_error(&label)),
                    ExecFlow::ContinueLoop(label) => Err(Self::continue_flow_error(&label)),
                    ExecFlow::Return => Ok(()),
                }
            })
        })
    }

    fn listener_event_argument(event: &EventState) -> Value {
        let target = event
            .target_value
            .as_ref()
            .cloned()
            .unwrap_or(Value::Node(event.target));
        let current_target = event
            .current_target_value
            .as_ref()
            .cloned()
            .unwrap_or(Value::Node(event.current_target));
        let clipboard_data = if event.event_type.eq_ignore_ascii_case("paste")
            || event.event_type.eq_ignore_ascii_case("copy")
            || event.event_type.eq_ignore_ascii_case("cut")
        {
            if let Some(object) = &event.clipboard_data_object {
                Value::Object(object.clone())
            } else {
                Self::new_clipboard_data_object_value(event.clipboard_data.as_deref().unwrap_or(""))
            }
        } else {
            Value::Undefined
        };
        let data_transfer = if Self::event_exposes_data_transfer(&event.event_type) {
            Self::new_data_transfer_object_value(&event.event_type)
        } else {
            Value::Undefined
        };
        let error_value = if event.event_type.eq_ignore_ascii_case("error") {
            event.detail.as_ref().cloned().unwrap_or(Value::Undefined)
        } else {
            Value::Undefined
        };
        let mut entries = vec![
            (INTERNAL_EVENT_OBJECT_KEY.to_string(), Value::Bool(true)),
            ("type".to_string(), Value::String(event.event_type.clone())),
            ("target".to_string(), target),
            ("currentTarget".to_string(), current_target),
            ("clipboardData".to_string(), clipboard_data),
            ("dataTransfer".to_string(), data_transfer),
            (
                "defaultPrevented".to_string(),
                Value::Bool(event.default_prevented),
            ),
            ("isTrusted".to_string(), Value::Bool(event.is_trusted)),
            ("bubbles".to_string(), Value::Bool(event.bubbles)),
            ("cancelable".to_string(), Value::Bool(event.cancelable)),
            (
                "detail".to_string(),
                event.detail.as_ref().cloned().unwrap_or(Value::Undefined),
            ),
            ("error".to_string(), error_value),
            (
                "eventPhase".to_string(),
                Value::Number(event.event_phase as i64),
            ),
            ("timeStamp".to_string(), Value::Number(event.time_stamp_ms)),
            (
                "key".to_string(),
                event
                    .key
                    .as_ref()
                    .map(|value| Value::String(value.clone()))
                    .unwrap_or(Value::Undefined),
            ),
            (
                "code".to_string(),
                event
                    .code
                    .as_ref()
                    .map(|value| Value::String(value.clone()))
                    .unwrap_or(Value::Undefined),
            ),
            ("ctrlKey".to_string(), Value::Bool(event.ctrl_key)),
            ("metaKey".to_string(), Value::Bool(event.meta_key)),
            ("shiftKey".to_string(), Value::Bool(event.shift_key)),
            ("altKey".to_string(), Value::Bool(event.alt_key)),
            ("repeat".to_string(), Value::Bool(event.repeat)),
            ("isComposing".to_string(), Value::Bool(event.is_composing)),
            (
                "state".to_string(),
                event.state.as_ref().cloned().unwrap_or(Value::Undefined),
            ),
            (
                "oldState".to_string(),
                event
                    .old_state
                    .as_ref()
                    .map(|value| Value::String(value.clone()))
                    .unwrap_or(Value::Undefined),
            ),
            (
                "newState".to_string(),
                event
                    .new_state
                    .as_ref()
                    .map(|value| Value::String(value.clone()))
                    .unwrap_or(Value::Undefined),
            ),
            (
                "preventDefault".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "stopPropagation".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "stopImmediatePropagation".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];

        if matches!(
            event.event_type.to_ascii_lowercase().as_str(),
            "keydown" | "keyup" | "keypress"
        ) {
            let key = event.key.clone().unwrap_or_default();
            let key_code = Self::keyboard_key_code_for_key(&key);
            let char_code = Self::keyboard_char_code_for_event(&event.event_type, &key);
            entries.push((
                INTERNAL_KEYBOARD_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("location".to_string(), Value::Number(event.location)));
            entries.push(("keyCode".to_string(), Value::Number(key_code)));
            entries.push(("charCode".to_string(), Value::Number(char_code)));
            entries.push((
                "keyIdentifier".to_string(),
                Value::String(if key.is_empty() {
                    "Unidentified".to_string()
                } else {
                    key
                }),
            ));
            entries.push((
                "getModifierState".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
        }

        if event.event_type.eq_ignore_ascii_case("wheel") {
            entries.push((
                INTERNAL_WHEEL_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("deltaX".to_string(), Value::Float(event.delta_x)));
            entries.push(("deltaY".to_string(), Value::Float(event.delta_y)));
            entries.push(("deltaZ".to_string(), Value::Float(event.delta_z)));
            entries.push(("deltaMode".to_string(), Value::Number(event.delta_mode)));
        }

        if Self::event_is_pointer_event(&event.event_type) {
            entries.push((
                INTERNAL_POINTER_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("pointerId".to_string(), Value::Number(event.pointer_id)));
            entries.push(("width".to_string(), Value::Float(event.pointer_width)));
            entries.push(("height".to_string(), Value::Float(event.pointer_height)));
            entries.push(("pressure".to_string(), Value::Float(event.pointer_pressure)));
            entries.push((
                "tangentialPressure".to_string(),
                Value::Float(event.pointer_tangential_pressure),
            ));
            entries.push(("tiltX".to_string(), Value::Number(event.pointer_tilt_x)));
            entries.push(("tiltY".to_string(), Value::Number(event.pointer_tilt_y)));
            entries.push(("twist".to_string(), Value::Number(event.pointer_twist)));
            entries.push((
                "pointerType".to_string(),
                Value::String(event.pointer_type.clone()),
            ));
            entries.push((
                "isPrimary".to_string(),
                Value::Bool(event.pointer_is_primary),
            ));
            entries.push((
                "altitudeAngle".to_string(),
                Value::Float(event.pointer_altitude_angle),
            ));
            entries.push((
                "azimuthAngle".to_string(),
                Value::Float(event.pointer_azimuth_angle),
            ));
            entries.push((
                "persistentDeviceId".to_string(),
                Value::Number(event.pointer_persistent_device_id),
            ));
            entries.push((
                "getCoalescedEvents".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
            entries.push((
                "getPredictedEvents".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
        }

        if event.event_type.eq_ignore_ascii_case("navigate") {
            entries.push((
                INTERNAL_NAVIGATE_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push((
                "canIntercept".to_string(),
                Value::Bool(event.navigate_can_intercept),
            ));
            entries.push((
                "destination".to_string(),
                event.navigate_destination.clone().unwrap_or(Value::Null),
            ));
            entries.push((
                "downloadRequest".to_string(),
                event
                    .navigate_download_request
                    .clone()
                    .unwrap_or(Value::Null),
            ));
            entries.push((
                "formData".to_string(),
                event.navigate_form_data.clone().unwrap_or(Value::Null),
            ));
            entries.push((
                "hashChange".to_string(),
                Value::Bool(event.navigate_hash_change),
            ));
            entries.push((
                "hasUAVisualTransition".to_string(),
                Value::Bool(event.navigate_has_ua_visual_transition),
            ));
            entries.push((
                "info".to_string(),
                event.navigate_info.clone().unwrap_or(Value::Undefined),
            ));
            entries.push((
                "navigationType".to_string(),
                Value::String(
                    event
                        .navigate_navigation_type
                        .clone()
                        .unwrap_or_else(|| "push".to_string()),
                ),
            ));
            entries.push((
                "signal".to_string(),
                event
                    .navigate_signal
                    .clone()
                    .unwrap_or_else(Self::new_navigate_event_default_signal_value),
            ));
            entries.push((
                "sourceElement".to_string(),
                event.navigate_source_element.clone().unwrap_or(Value::Null),
            ));
            entries.push((
                "userInitiated".to_string(),
                Value::Bool(event.navigate_user_initiated),
            ));
            entries.push((
                "intercept".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
            entries.push((
                "scroll".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
        }

        if event.event_type.eq_ignore_ascii_case("message") {
            entries.push((
                "data".to_string(),
                event.message_data.clone().unwrap_or(Value::Undefined),
            ));
            entries.push((
                "origin".to_string(),
                Value::String(event.message_origin.clone().unwrap_or_default()),
            ));
            entries.push((
                "source".to_string(),
                event.message_source.clone().unwrap_or(Value::Null),
            ));
        }

        Self::new_object_value(entries)
    }

    fn event_exposes_data_transfer(event_type: &str) -> bool {
        matches!(
            event_type.to_ascii_lowercase().as_str(),
            "drag" | "dragstart" | "dragend" | "dragenter" | "dragover" | "dragleave" | "drop"
        )
    }

    fn event_is_pointer_event(event_type: &str) -> bool {
        matches!(
            event_type.to_ascii_lowercase().as_str(),
            "pointerover"
                | "pointerenter"
                | "pointerdown"
                | "pointermove"
                | "pointerrawupdate"
                | "pointerup"
                | "pointercancel"
                | "pointerout"
                | "pointerleave"
                | "gotpointercapture"
                | "lostpointercapture"
        )
    }

    pub(crate) fn execute_timer_task_callback(
        &mut self,
        callback: &TimerCallback,
        callback_args: &[Value],
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        match callback {
            TimerCallback::Reference(name) => {
                let callable = env
                    .get(name)
                    .cloned()
                    .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;
                if !self.is_callable_value(&callable) {
                    return Err(Error::ScriptRuntime(format!(
                        "timer callback '{name}' is not a function"
                    )));
                }
                let _ = self.execute_callable_value_with_env(
                    &callable,
                    callback_args,
                    event,
                    Some(env),
                )?;
                Ok(())
            }
            TimerCallback::Inline(handler) => {
                let handler = handler.clone();
                let event_param = handler
                    .first_event_param()
                    .map(|event_param| event_param.to_string());
                self.with_callback_scope_depth(env, |this, callback_env| {
                    this.with_isolated_loop_control_scope(|this| {
                        this.bind_handler_params(
                            &handler,
                            callback_args,
                            callback_env,
                            &event_param,
                            event,
                        )?;
                        let flow =
                            this.execute_stmts(&handler.stmts, &event_param, event, callback_env)?;
                        callback_env.remove(INTERNAL_RETURN_SLOT);
                        match flow {
                            ExecFlow::Continue => Ok(()),
                            ExecFlow::Break(label) => Err(Self::break_flow_error(&label)),
                            ExecFlow::ContinueLoop(label) => Err(Self::continue_flow_error(&label)),
                            ExecFlow::Return => Ok(()),
                        }
                    })
                })
            }
        }
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
            this.with_isolated_loop_control_scope(|this| {
                this.bind_handler_params(handler, &[], callback_env, &event_param, &event)?;
                let run =
                    this.execute_stmts(&handler.stmts, &event_param, &mut event, callback_env);
                run.map(|_| ())
            })
        })
    }

    fn run_callable_microtask(&mut self, callback: &Value) -> Result<()> {
        let event = EventState::new("microtask", self.dom.root, self.scheduler.now_ms);
        let _ = self.execute_callable_value_with_env(callback, &[], &event, None)?;
        Ok(())
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
        scope: HashMap<String, (ScriptHandler, bool, bool)>,
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
        scopes: &[Arc<HashMap<String, (ScriptHandler, bool, bool)>>],
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
