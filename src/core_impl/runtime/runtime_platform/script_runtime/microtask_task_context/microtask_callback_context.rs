use super::*;

impl Harness {
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

    pub(crate) fn run_script_microtask_handler(
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

    pub(crate) fn run_callable_microtask(&mut self, callback: &Value) -> Result<()> {
        let event = EventState::new("microtask", self.dom.root, self.scheduler.now_ms);
        let _ = self.execute_callable_value_with_env(callback, &[], &event, None)?;
        Ok(())
    }

    pub(crate) fn run_worker_message_microtask(
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
}
