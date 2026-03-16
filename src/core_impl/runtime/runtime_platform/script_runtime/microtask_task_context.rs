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

    pub(crate) fn queue_worker_message_microtask(
        &mut self,
        worker: &Rc<RefCell<ObjectValue>>,
        target: &Rc<RefCell<ObjectValue>>,
        target_this: Value,
        data: Value,
    ) {
        self.scheduler
            .microtask_queue
            .push_back(ScheduledMicrotask::WorkerMessage {
                worker: worker.clone(),
                target: target.clone(),
                target_this,
                data,
            });
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
                    ScheduledMicrotask::WorkerMessage {
                        worker,
                        target,
                        target_this,
                        data,
                    } => {
                        this.run_worker_message_microtask(&worker, &target, target_this, data)?;
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
            vec![self.listener_event_argument(event)]
        } else {
            Vec::new()
        };
        self.with_callback_scope_depth(env, |this, callback_env| {
            this.with_isolated_loop_control_scope(|this| {
                this.project_pending_listener_capture_env_updates(callback_env);
                this.bind_handler_params(handler, &event_args, callback_env, &event_param, event)?;
                let flow = this.execute_stmts(&handler.stmts, &event_param, event, callback_env)?;
                Self::sync_event_argument_back_to_state(
                    event,
                    callback_env,
                    event_param.as_deref(),
                );
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

    pub(crate) fn listener_event_argument(&mut self, event: &mut EventState) -> Value {
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
                let value = self
                    .new_clipboard_data_object_value(event.clipboard_data.as_deref().unwrap_or(""));
                if let Value::Object(object) = &value {
                    event.clipboard_data_object = Some(object.clone());
                }
                value
            }
        } else {
            Value::Undefined
        };
        let data_transfer = if Self::event_exposes_data_transfer(&event.event_type) {
            if let Some(object) = &event.data_transfer_object {
                Value::Object(object.clone())
            } else {
                let value = self.new_data_transfer_object_value(&event.event_type);
                if let Value::Object(object) = &value {
                    event.data_transfer_object = Some(object.clone());
                }
                value
            }
        } else {
            Value::Undefined
        };
        let error_value = if event.error_event_interface {
            event.error_event_error.clone()
        } else if event.event_type.eq_ignore_ascii_case("error") {
            event.detail.as_ref().cloned().unwrap_or(Value::Undefined)
        } else {
            Value::Undefined
        };
        let error_message = if event.error_event_interface {
            Value::String(event.error_event_message.clone())
        } else {
            Value::Undefined
        };
        let error_filename = if event.error_event_interface {
            Value::String(event.error_event_filename.clone())
        } else {
            Value::Undefined
        };
        let error_lineno = if event.error_event_interface {
            Value::Number(event.error_event_lineno)
        } else {
            Value::Undefined
        };
        let error_colno = if event.error_event_interface {
            Value::Number(event.error_event_colno)
        } else {
            Value::Undefined
        };
        let hash_change_old_url = if event.hash_change_interface {
            Value::String(event.hash_change_old_url.clone())
        } else {
            Value::Undefined
        };
        let hash_change_new_url = if event.hash_change_interface {
            Value::String(event.hash_change_new_url.clone())
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
            ("message".to_string(), error_message),
            ("filename".to_string(), error_filename),
            ("lineno".to_string(), error_lineno),
            ("colno".to_string(), error_colno),
            ("oldURL".to_string(), hash_change_old_url),
            ("newURL".to_string(), hash_change_new_url),
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

        if event.event_type.eq_ignore_ascii_case("submit") || event.submitter.is_some() {
            entries.push((
                "submitter".to_string(),
                event.submitter.as_ref().cloned().unwrap_or(Value::Null),
            ));
        }

        if event.before_unload_interface || event.event_type.eq_ignore_ascii_case("beforeunload") {
            entries.push((
                INTERNAL_BEFORE_UNLOAD_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push((
                "returnValue".to_string(),
                Value::String(event.before_unload_return_value.clone()),
            ));
        }

        if event.error_event_interface {
            entries.push((
                INTERNAL_ERROR_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
        }

        if event.hash_change_interface {
            entries.push((
                INTERNAL_HASH_CHANGE_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
        }

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

        Self::mark_object_properties_non_enumerable(
            &mut entries,
            &[
                "preventDefault",
                "stopPropagation",
                "stopImmediatePropagation",
            ],
        );
        if matches!(
            event.event_type.to_ascii_lowercase().as_str(),
            "keydown" | "keyup" | "keypress"
        ) {
            Self::mark_object_properties_non_enumerable(&mut entries, &["getModifierState"]);
        }
        if Self::event_is_pointer_event(&event.event_type) {
            Self::mark_object_properties_non_enumerable(
                &mut entries,
                &["getCoalescedEvents", "getPredictedEvents"],
            );
        }
        if event.event_type.eq_ignore_ascii_case("navigate") {
            Self::mark_object_properties_non_enumerable(&mut entries, &["intercept", "scroll"]);
        }

        Self::new_object_value(entries)
    }

    pub(crate) fn sync_event_argument_back_to_state(
        event: &mut EventState,
        callback_env: &HashMap<String, Value>,
        event_param: Option<&str>,
    ) {
        let Some(event_param) = event_param else {
            return;
        };
        let Some(Value::Object(event_object)) = callback_env.get(event_param) else {
            return;
        };
        let entries = event_object.borrow();
        if !Self::is_event_object(&entries) {
            return;
        }

        if event.cancelable
            && Self::object_get_entry(&entries, "defaultPrevented").is_some_and(|v| v.truthy())
        {
            event.default_prevented = true;
        }
        if Self::object_get_entry(&entries, INTERNAL_EVENT_STOP_PROPAGATION_KEY)
            .is_some_and(|v| v.truthy())
        {
            event.propagation_stopped = true;
        }
        if Self::object_get_entry(&entries, INTERNAL_EVENT_STOP_IMMEDIATE_PROPAGATION_KEY)
            .is_some_and(|v| v.truthy())
        {
            event.immediate_propagation_stopped = true;
            event.propagation_stopped = true;
        }

        if Self::is_before_unload_event_object(&entries)
            || event.before_unload_interface
            || event.event_type.eq_ignore_ascii_case("beforeunload")
        {
            event.before_unload_interface = true;
            event.before_unload_return_value = Self::object_get_entry(&entries, "returnValue")
                .map(|value| value.as_string())
                .unwrap_or_default();
            if event.cancelable && !event.before_unload_return_value.is_empty() {
                event.default_prevented = true;
            }
        }
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
                self.apply_pending_listener_capture_env_updates(env);
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
                self.apply_expression_env_overrides_to_env(env);
                self.apply_pending_listener_capture_env_updates(env);
                Ok(())
            }
            TimerCallback::Inline(handler) => {
                let handler = handler.clone();
                let event_param = handler
                    .first_event_param()
                    .map(|event_param| event_param.to_string());
                self.with_callback_scope_depth(env, |this, callback_env| {
                    this.with_isolated_loop_control_scope(|this| {
                        this.apply_pending_listener_capture_env_updates(callback_env);
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
                this.apply_pending_listener_capture_env_updates(callback_env);
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

    fn run_worker_message_microtask(
        &mut self,
        worker: &Rc<RefCell<ObjectValue>>,
        target: &Rc<RefCell<ObjectValue>>,
        target_this: Value,
        data: Value,
    ) -> Result<()> {
        if Self::worker_is_terminated_object(worker) {
            return Ok(());
        }
        let event = EventState::new("microtask", self.dom.root, self.scheduler.now_ms);
        self.dispatch_worker_message_to_onmessage(target, target_this, data, &event)
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

    pub(crate) fn push_shared_listener_capture_env_frame(
        &mut self,
        shared_env: Rc<RefCell<ScriptEnv>>,
        shared_env_owned_by_scope: bool,
    ) -> usize {
        self.push_shared_listener_capture_env_frame_with_names(
            shared_env,
            shared_env_owned_by_scope,
            None,
        )
    }

    pub(crate) fn push_shared_listener_capture_env_frame_with_names(
        &mut self,
        shared_env: Rc<RefCell<ScriptEnv>>,
        shared_env_owned_by_scope: bool,
        tracked_names: Option<HashSet<String>>,
    ) -> usize {
        let start_len = self.script_runtime.listener_capture_env_stack.len();
        self.script_runtime
            .listener_capture_env_stack
            .push(ListenerCaptureFrame {
                shared_env: Some(shared_env),
                shared_env_owned_by_scope,
                tracked_names,
                ..ListenerCaptureFrame::default()
            });
        start_len
    }

    pub(crate) fn restore_listener_capture_env_stack(&mut self, start_len: usize) {
        if start_len >= self.script_runtime.listener_capture_env_stack.len() {
            return;
        }

        let mut propagated_updates = HashMap::new();
        for frame in &mut self.script_runtime.listener_capture_env_stack[start_len..] {
            let pending_updates = std::mem::take(&mut frame.pending_env_updates);
            if let Some(shared_env) = frame.shared_env.as_ref() {
                let mut shared_env = shared_env.borrow_mut();
                for (name, value) in &pending_updates {
                    if Self::is_internal_env_key(name) {
                        continue;
                    }
                    if let Some(value) = value {
                        shared_env.insert(name.clone(), value.clone());
                    } else {
                        shared_env.remove(name);
                    }
                }
            }
            propagated_updates.extend(pending_updates);
        }

        self.script_runtime
            .listener_capture_env_stack
            .truncate(start_len);

        if propagated_updates.is_empty() {
            return;
        }

        if let Some(parent) = self.script_runtime.listener_capture_env_stack.last_mut() {
            parent.pending_env_updates.extend(propagated_updates);
        }
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

    pub(crate) fn env_has_local_or_lexical_binding(
        env: &HashMap<String, Value>,
        name: &str,
    ) -> bool {
        if Self::env_has_local_binding(env, name) {
            return true;
        }
        Self::env_top_level_lexical_binding_names(env).contains(name)
    }

    pub(crate) fn env_local_or_lexical_binding_names(
        env: &HashMap<String, Value>,
    ) -> HashSet<String> {
        let mut names = match env.get(INTERNAL_LOCAL_BINDINGS_KEY) {
            Some(Value::Array(local_bindings)) => local_bindings
                .borrow()
                .iter()
                .filter_map(|entry| match entry {
                    Value::String(value) => Some(value.clone()),
                    _ => None,
                })
                .collect::<HashSet<_>>(),
            _ => HashSet::new(),
        };
        names.extend(Self::env_top_level_lexical_binding_names(env));
        names
    }

    pub(crate) fn env_has_local_binding(env: &HashMap<String, Value>, name: &str) -> bool {
        match env.get(INTERNAL_LOCAL_BINDINGS_KEY) {
            Some(Value::Array(local_bindings)) => local_bindings
                .borrow()
                .iter()
                .any(|entry| matches!(entry, Value::String(value) if value == name)),
            _ => false,
        }
    }

    pub(crate) fn env_should_sync_global_name(env: &HashMap<String, Value>, name: &str) -> bool {
        if Self::env_has_local_or_lexical_binding(env, name) {
            return false;
        }
        match env.get(INTERNAL_GLOBAL_SYNC_NAMES_KEY) {
            Some(Value::Array(names)) => names
                .borrow()
                .iter()
                .any(|entry| matches!(entry, Value::String(value) if value == name)),
            _ => false,
        }
    }

    pub(crate) fn env_has_explicit_binding(env: &HashMap<String, Value>, name: &str) -> bool {
        if Self::is_internal_env_key(name) || !env.contains_key(name) {
            return false;
        }
        !Self::env_should_sync_global_name(env, name)
    }

    pub(crate) fn apply_expression_env_overrides_to_env(
        &mut self,
        env: &mut HashMap<String, Value>,
    ) {
        let overrides = std::mem::take(&mut self.script_runtime.expression_env_overrides);
        for (name, value) in overrides {
            if Self::is_internal_env_key(&name) || Self::env_has_local_binding(env, &name) {
                continue;
            }
            if let Some(value) = value {
                env.insert(name, value);
            } else {
                env.remove(&name);
            }
        }
    }

    pub(crate) fn ensure_listener_capture_env(&mut self) -> Rc<RefCell<ScriptEnv>> {
        if let Some(frame) = self.script_runtime.listener_capture_env_stack.last_mut() {
            if frame.shared_env.is_none() {
                frame.shared_env = Some(Rc::new(RefCell::new(ScriptEnv::default())));
                frame.shared_env_owned_by_scope = true;
            }
            frame
                .shared_env
                .as_ref()
                .expect("shared env should exist after initialization")
                .clone()
        } else {
            Rc::new(RefCell::new(ScriptEnv::default()))
        }
    }

    pub(crate) fn queue_listener_capture_env_update_for_shared_env(
        &mut self,
        shared_env: &Rc<RefCell<ScriptEnv>>,
        name: String,
        value: Option<Value>,
    ) {
        if Self::is_internal_env_key(&name) {
            return;
        }
        for frame in self
            .script_runtime
            .listener_capture_env_stack
            .iter_mut()
            .rev()
        {
            let Some(frame_shared_env) = frame.shared_env.as_ref() else {
                continue;
            };
            if Rc::ptr_eq(frame_shared_env, shared_env) {
                frame.pending_env_updates.insert(name, value);
                return;
            }
        }
        if let Some(frame) = self.script_runtime.listener_capture_env_stack.last_mut() {
            frame.pending_env_updates.insert(name, value);
        }
    }

    pub(crate) fn pending_listener_capture_scope_start(env: &HashMap<String, Value>) -> usize {
        match env.get(INTERNAL_PENDING_SCOPE_START_KEY) {
            Some(Value::Number(start)) if *start >= 0 => *start as usize,
            _ => 0,
        }
    }

    fn listener_capture_pending_updates_snapshot_from(
        &self,
        start: usize,
    ) -> HashMap<String, Option<Value>> {
        let mut updates = HashMap::new();
        let start = start.min(self.script_runtime.listener_capture_env_stack.len());
        for frame in &self.script_runtime.listener_capture_env_stack[start..] {
            updates.extend(frame.pending_env_updates.clone());
        }
        updates
    }

    fn apply_listener_capture_pending_updates_map(
        &mut self,
        env: &mut HashMap<String, Value>,
        updates: HashMap<String, Option<Value>>,
        allow_local_bindings: bool,
    ) {
        if updates.is_empty() {
            return;
        }
        let restricted_names =
            (!allow_local_bindings).then(|| Self::env_local_or_lexical_binding_names(env));
        for (name, value) in updates {
            if Self::is_internal_env_key(&name) {
                continue;
            }
            if restricted_names
                .as_ref()
                .is_some_and(|names| names.contains(&name))
            {
                continue;
            }
            if let Some(value) = value {
                env.insert(name, value);
            } else {
                env.remove(&name);
            }
        }
    }

    pub(crate) fn project_pending_listener_capture_env_updates(
        &mut self,
        env: &mut HashMap<String, Value>,
    ) {
        if self.script_runtime.listener_capture_env_stack.is_empty() {
            return;
        }
        let updates = self.listener_capture_pending_updates_snapshot_from(0);
        let allow_local_bindings = self
            .script_runtime
            .listener_capture_env_stack
            .iter()
            .rev()
            .find_map(|frame| {
                frame
                    .shared_env
                    .as_ref()
                    .map(|_| frame.shared_env_owned_by_scope)
            })
            .unwrap_or(false);
        self.apply_listener_capture_pending_updates_map(env, updates, allow_local_bindings);
    }

    pub(crate) fn apply_pending_listener_capture_env_updates(
        &mut self,
        env: &mut HashMap<String, Value>,
    ) {
        if self.script_runtime.listener_capture_env_stack.is_empty() {
            return;
        }
        let drain_start = Self::pending_listener_capture_scope_start(env)
            .min(self.script_runtime.listener_capture_env_stack.len());
        let updates = self.listener_capture_pending_updates_snapshot_from(drain_start);
        if updates.is_empty() {
            return;
        }
        self.apply_listener_capture_pending_updates_map(env, updates, true);
        for frame in &mut self.script_runtime.listener_capture_env_stack[drain_start..] {
            frame.pending_env_updates.clear();
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

    pub(crate) fn sync_scheduled_task_captures_for_binding_if_escaping(
        &mut self,
        env: &HashMap<String, Value>,
        name: &str,
        value: &Value,
    ) {
        if Self::is_internal_env_key(name) {
            return;
        }

        let local_binding = Self::env_has_local_binding(env, name);
        let lexical_binding = Self::env_top_level_lexical_binding_names(env).contains(name);
        let local_scope_start = Self::pending_listener_capture_scope_start(env)
            .min(self.script_runtime.listener_capture_env_stack.len());
        let captured_in_current_function_scope = self
            .script_runtime
            .listener_capture_env_stack
            .iter()
            .enumerate()
            .rev()
            .any(|(frame_index, frame)| {
                if frame_index < local_scope_start || frame.shared_env_owned_by_scope {
                    return false;
                }
                frame.pending_env_updates.contains_key(name)
                    || frame
                        .tracked_names
                        .as_ref()
                        .is_some_and(|tracked_names| tracked_names.contains(name))
                    || frame
                        .shared_env
                        .as_ref()
                        .is_some_and(|shared_env| shared_env.borrow().contains_key(name))
            });
        let scope_local_binding = local_binding
            || (lexical_binding
                && Self::env_scope_depth(env) > 0
                && !captured_in_current_function_scope);
        let active_shared_env = self
            .script_runtime
            .listener_capture_env_stack
            .iter()
            .enumerate()
            .rev()
            .find_map(|(frame_index, frame)| {
                let shared_env = frame.shared_env.as_ref()?;
                let same_scope_owned_frame =
                    frame.shared_env_owned_by_scope && frame_index >= local_scope_start;
                let tracks_name = if scope_local_binding && !same_scope_owned_frame {
                    false
                } else {
                    same_scope_owned_frame
                        || frame.pending_env_updates.contains_key(name)
                        || frame
                            .tracked_names
                            .as_ref()
                            .is_some_and(|tracked_names| tracked_names.contains(name))
                        || frame
                            .shared_env
                            .as_ref()
                            .is_some_and(|shared_env| shared_env.borrow().contains_key(name))
                };
                tracks_name.then(|| (shared_env.clone(), frame.shared_env_owned_by_scope))
            });
        let shared_frame_tracks_name = active_shared_env.is_some();
        if scope_local_binding && !shared_frame_tracks_name {
            return;
        }

        if !shared_frame_tracks_name
            && !Self::env_should_sync_global_name(env, name)
            && !lexical_binding
            && !self.listeners.capture_name_counts.contains_key(name)
            && self.scheduler.task_queue.is_empty()
        {
            return;
        }

        if let Some((shared_env, shared_env_owned_by_scope)) = active_shared_env {
            if shared_env_owned_by_scope {
                shared_env
                    .borrow_mut()
                    .insert(name.to_string(), value.clone());
            }
            self.queue_listener_capture_env_update_for_shared_env(
                &shared_env,
                name.to_string(),
                Some(value.clone()),
            );
        }
        self.sync_function_captures_in_env_for_binding(env, name, value);
        if !scope_local_binding {
            self.sync_scheduled_task_captures_for_binding(name, value);
        }
    }

    fn sync_function_captures_in_env_for_binding(
        &mut self,
        env: &HashMap<String, Value>,
        name: &str,
        value: &Value,
    ) {
        let mut seen_function_ids = HashSet::new();
        for entry in env.values() {
            let Value::Function(function) = entry else {
                continue;
            };
            if !seen_function_ids.insert(function.function_id) {
                continue;
            }
            if function.global_scope || function.local_bindings.contains(name) {
                continue;
            }
            if !function.captured_names.contains(name) {
                continue;
            }
            function
                .captured_env
                .borrow_mut()
                .insert(name.to_string(), value.clone());
            self.queue_listener_capture_env_update_for_shared_env(
                &function.captured_env,
                name.to_string(),
                Some(value.clone()),
            );
        }
    }
}
