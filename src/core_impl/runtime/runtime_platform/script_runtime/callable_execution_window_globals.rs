use super::*;

impl Harness {
    fn current_window_dimension(&self, key: &str, fallback: f64) -> i64 {
        let window = self.dom_runtime.window_object.borrow();
        let raw_value = Self::object_get_entry(&window, key);
        let parsed = match raw_value {
            Some(Value::Number(value)) => Some(value as f64),
            Some(Value::Float(value)) if value.is_finite() => Some(value),
            Some(Value::String(value)) => value.parse::<f64>().ok(),
            _ => None,
        }
        .unwrap_or(fallback);
        if !parsed.is_finite() {
            fallback as i64
        } else {
            parsed.max(0.0).trunc() as i64
        }
    }

    fn set_window_inner_outer_size(&mut self, width: i64, height: i64) {
        let mut window = self.dom_runtime.window_object.borrow_mut();
        Self::object_set_entry(&mut window, "innerWidth".to_string(), Value::Number(width));
        Self::object_set_entry(
            &mut window,
            "innerHeight".to_string(),
            Value::Number(height),
        );
        Self::object_set_entry(&mut window, "outerWidth".to_string(), Value::Number(width));
        Self::object_set_entry(
            &mut window,
            "outerHeight".to_string(),
            Value::Number(height),
        );
    }

    fn timer_callback_from_args(
        &mut self,
        kind: &str,
        args: &[Value],
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<(TimerCallback, HashMap<String, Value>)> {
        let mut timer_env = caller_env
            .cloned()
            .unwrap_or_else(|| self.script_runtime.env.to_map());
        let callback = match &args[0] {
            value if self.is_callable_value(value) => {
                let callback_name = format!(
                    "\u{0}\u{0}bt_{}_cb_{}",
                    kind,
                    self.script_runtime.allocate_function_id()
                );
                timer_env.insert(callback_name.clone(), value.clone());
                TimerCallback::Reference(callback_name)
            }
            Value::String(source) => {
                let stmts = parse_block_statements(source).map_err(|err| match err {
                    Error::ScriptParse(message) => {
                        Error::ScriptRuntime(format!("SyntaxError: {message}"))
                    }
                    other => other,
                })?;
                TimerCallback::Inline(ScriptHandler {
                    params: Vec::new(),
                    stmts,
                })
            }
            _ => {
                return Err(Error::ScriptRuntime(format!(
                    "TypeError: {kind} callback must be callable or a string"
                )));
            }
        };
        Ok((callback, timer_env))
    }

    pub(crate) fn execute_window_global_callable_kind(
        &mut self,
        kind: &str,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
        this_arg: Option<&Value>,
    ) -> Result<Option<Value>> {
        let value = match kind {
            "window_close_function" => {
                self.browser_apis.window_closed = true;
                self.sync_window_runtime_properties();
                Some(Value::Undefined)
            }
            "window_open_function" => {
                if args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "open supports zero to three arguments".into(),
                    ));
                }
                let url = self.window_open_target_url(args);
                let target = args.get(1).map(Value::as_string).unwrap_or_default();
                let features = args.get(2).map(Value::as_string).unwrap_or_default();
                Some(self.new_popup_window_value(&url, &target, &features))
            }
            "window_stop_function" | "window_focus_function" => Some(Value::Undefined),
            "window_scroll_function"
            | "window_scroll_by_function"
            | "window_scroll_to_function" => {
                if args.len() > 2 {
                    let name = match kind {
                        "window_scroll_function" => "scroll",
                        "window_scroll_by_function" => "scrollBy",
                        _ => "scrollTo",
                    };
                    return Err(Error::ScriptRuntime(format!(
                        "{name} supports zero, one, or two arguments"
                    )));
                }
                let method = match kind {
                    "window_scroll_function" => "scroll",
                    "window_scroll_by_function" => "scrollBy",
                    _ => "scrollTo",
                };
                let position_changed = self.apply_document_scroll_operation(method, args);
                self.sync_window_runtime_properties();
                self.dispatch_document_scroll_sequence(position_changed)?;
                Some(Value::Undefined)
            }
            "window_move_by_function" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "moveBy requires exactly two arguments".into(),
                    ));
                }
                let delta_x = Self::value_to_i64(&args[0]);
                let delta_y = Self::value_to_i64(&args[1]);
                self.browser_apis.window_screen_x =
                    self.browser_apis.window_screen_x.saturating_add(delta_x);
                self.browser_apis.window_screen_y =
                    self.browser_apis.window_screen_y.saturating_add(delta_y);
                self.sync_window_runtime_properties();
                Some(Value::Undefined)
            }
            "window_move_to_function" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "moveTo requires exactly two arguments".into(),
                    ));
                }
                self.browser_apis.window_screen_x = Self::value_to_i64(&args[0]);
                self.browser_apis.window_screen_y = Self::value_to_i64(&args[1]);
                self.sync_window_runtime_properties();
                Some(Value::Undefined)
            }
            "window_resize_by_function" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "resizeBy requires exactly two arguments".into(),
                    ));
                }
                let current_width = self.current_window_dimension("innerWidth", 1024.0);
                let current_height = self.current_window_dimension("innerHeight", 768.0);
                let next_width = current_width
                    .saturating_add(Self::value_to_i64(&args[0]))
                    .max(0);
                let next_height = current_height
                    .saturating_add(Self::value_to_i64(&args[1]))
                    .max(0);
                self.set_window_inner_outer_size(next_width, next_height);
                Some(Value::Undefined)
            }
            "window_resize_to_function" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "resizeTo requires exactly two arguments".into(),
                    ));
                }
                let next_width = Self::value_to_i64(&args[0]).max(0);
                let next_height = Self::value_to_i64(&args[1]).max(0);
                self.set_window_inner_outer_size(next_width, next_height);
                Some(Value::Undefined)
            }
            "window_post_message_function" => {
                if args.is_empty() || args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "postMessage requires one to three arguments".into(),
                    ));
                }

                let sender_origin = self.current_location_parts().origin();
                let target_origin =
                    self.window_post_message_target_origin_from_args(args, &sender_origin);
                let target_window = self.window_post_message_target_window(this_arg);
                let recipient_origin = self.current_location_parts().origin();
                if !Self::window_post_message_target_origin_matches(
                    &target_origin,
                    &recipient_origin,
                    &sender_origin,
                ) {
                    return Ok(Some(Value::Undefined));
                }

                let mut array_stack = Vec::new();
                let mut object_stack = Vec::new();
                let data =
                    Self::structured_clone_value(&args[0], &mut array_stack, &mut object_stack)?;
                let event_payload = Self::new_object_value(vec![
                    (INTERNAL_EVENT_OBJECT_KEY.to_string(), Value::Bool(true)),
                    ("type".to_string(), Value::String("message".to_string())),
                    ("data".to_string(), data),
                    ("origin".to_string(), Value::String(sender_origin)),
                    (
                        "source".to_string(),
                        Value::Object(self.dom_runtime.window_object.clone()),
                    ),
                ]);
                let _ = self.dispatch_event_target(target_window, event_payload)?;
                Some(Value::Undefined)
            }
            "window_get_computed_style_function" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "getComputedStyle requires one or two arguments".into(),
                    ));
                }
                let node = match &args[0] {
                    Value::Node(node) if self.dom.element(*node).is_some() => *node,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "TypeError: getComputedStyle target must be an Element".into(),
                        ));
                    }
                };
                let pseudo = Self::get_computed_style_pseudo_from_value(args.get(1))?;
                Some(Self::new_computed_style_object_value(node, pseudo))
            }
            "window_alert_function" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "alert requires zero or one argument".into(),
                    ));
                }
                let message = args.first().map(Value::as_string).unwrap_or_default();
                self.platform_mocks.alert_messages.push(message);
                Some(Value::Undefined)
            }
            "window_confirm_function" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "confirm requires zero or one argument".into(),
                    ));
                }
                if let Some(message) = args.first() {
                    let _ = message.as_string();
                }
                let accepted = self
                    .platform_mocks
                    .confirm_responses
                    .pop_front()
                    .unwrap_or(self.platform_mocks.default_confirm_response);
                Some(Value::Bool(accepted))
            }
            "window_prompt_function" => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "prompt requires zero to two arguments".into(),
                    ));
                }
                if let Some(message) = args.first() {
                    let _ = message.as_string();
                }
                let default_value = args.get(1).map(Value::as_string);
                let response = self
                    .platform_mocks
                    .prompt_responses
                    .pop_front()
                    .unwrap_or_else(|| {
                        self.platform_mocks
                            .default_prompt_response
                            .clone()
                            .or(default_value)
                    });
                Some(match response {
                    Some(value) => Value::String(value),
                    None => Value::Null,
                })
            }
            "popup_window_close_function" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime("close takes no arguments".into()));
                }
                let popup_window = Self::popup_window_receiver_object(this_arg)?;
                Self::object_set_entry(
                    &mut popup_window.borrow_mut(),
                    "closed".to_string(),
                    Value::Bool(true),
                );
                Some(Value::Undefined)
            }
            "popup_window_focus_function" => Some(Value::Undefined),
            "popup_window_print_function" | "window_print_function" => {
                self.platform_mocks.print_call_count =
                    self.platform_mocks.print_call_count.saturating_add(1);
                Some(Value::Undefined)
            }
            "popup_document_open_function" => {
                let popup_document = Self::popup_document_receiver_object(this_arg)?;
                {
                    let mut document_entries = popup_document.borrow_mut();
                    Self::object_set_entry(
                        &mut document_entries,
                        INTERNAL_POPUP_DOCUMENT_HTML_KEY.to_string(),
                        Value::String(String::new()),
                    );
                    Self::object_set_entry(
                        &mut document_entries,
                        "readyState".to_string(),
                        Value::String("loading".to_string()),
                    );
                }
                Some(this_arg.cloned().unwrap_or(Value::Undefined))
            }
            "popup_document_write_function" => {
                let popup_document = Self::popup_document_receiver_object(this_arg)?;
                let fragment = args.iter().map(Value::as_string).collect::<String>();
                let current_html = {
                    let document_entries = popup_document.borrow();
                    Self::object_get_entry(&document_entries, INTERNAL_POPUP_DOCUMENT_HTML_KEY)
                        .map(|value| value.as_string())
                        .unwrap_or_default()
                };
                Self::object_set_entry(
                    &mut popup_document.borrow_mut(),
                    INTERNAL_POPUP_DOCUMENT_HTML_KEY.to_string(),
                    Value::String(format!("{current_html}{fragment}")),
                );
                Some(Value::Undefined)
            }
            "popup_document_close_function" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime("close takes no arguments".into()));
                }
                let popup_document = Self::popup_document_receiver_object(this_arg)?;
                Self::object_set_entry(
                    &mut popup_document.borrow_mut(),
                    "readyState".to_string(),
                    Value::String("complete".to_string()),
                );
                Some(Value::Undefined)
            }
            "window_report_error_function" => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypeError: reportError requires one argument".into(),
                    ));
                }
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "reportError supports only one argument".into(),
                    ));
                }
                let throwable = args[0].clone();
                let event_payload = Self::new_object_value(vec![
                    (INTERNAL_EVENT_OBJECT_KEY.to_string(), Value::Bool(true)),
                    ("type".to_string(), Value::String("error".to_string())),
                    ("detail".to_string(), throwable),
                    ("bubbles".to_string(), Value::Bool(false)),
                    ("cancelable".to_string(), Value::Bool(true)),
                    ("defaultPrevented".to_string(), Value::Bool(false)),
                    ("isTrusted".to_string(), Value::Bool(false)),
                    ("eventPhase".to_string(), Value::Number(0)),
                    (
                        "timeStamp".to_string(),
                        Value::Number(self.scheduler.now_ms),
                    ),
                    ("target".to_string(), Value::Null),
                    ("currentTarget".to_string(), Value::Null),
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
                ]);
                let _ = self
                    .dispatch_event_target(self.dom_runtime.window_object.clone(), event_payload);
                Some(Value::Undefined)
            }
            "global_decode_uri" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "decodeURI requires exactly one argument".into(),
                    ));
                }
                Some(Value::String(decode_uri_like(&args[0].as_string(), false)?))
            }
            "global_decode_uri_component" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "decodeURIComponent requires exactly one argument".into(),
                    ));
                }
                Some(Value::String(decode_uri_like(&args[0].as_string(), true)?))
            }
            "global_atob" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "atob requires exactly one argument".into(),
                    ));
                }
                Some(Value::String(decode_base64_to_binary_string(
                    &args[0].as_string(),
                )?))
            }
            "global_btoa" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "btoa requires exactly one argument".into(),
                    ));
                }
                Some(Value::String(encode_binary_string_to_base64(
                    &args[0].as_string(),
                )?))
            }
            "global_structured_clone" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "structuredClone requires one or two arguments".into(),
                    ));
                }
                return Self::structured_clone_value_with_options(&args[0], args.get(1)).map(Some);
            }
            "global_css_escape" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "CSS.escape requires exactly one argument".into(),
                    ));
                }
                Some(Value::String(Self::css_escape_identifier(
                    &args[0].as_string(),
                )))
            }
            "global_request_animation_frame" => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "requestAnimationFrame requires at least one argument".into(),
                    ));
                }
                let callback = args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime(
                        "requestAnimationFrame callback must be callable".into(),
                    ));
                }
                let mut timer_env = caller_env
                    .cloned()
                    .unwrap_or_else(|| self.script_runtime.env.to_map());
                let callback_name = format!(
                    "\u{0}\u{0}bt_raf_cb_{}",
                    self.script_runtime.allocate_function_id()
                );
                timer_env.insert(callback_name.clone(), callback);
                let timer_id = self
                    .schedule_animation_frame(TimerCallback::Reference(callback_name), &timer_env);
                Some(Value::Number(timer_id))
            }
            "global_set_timeout" => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "setTimeout requires at least one argument".into(),
                    ));
                }
                let (callback, timer_env) =
                    self.timer_callback_from_args("setTimeout", args, caller_env)?;
                let delay = args.get(1).map(Self::value_to_i64).unwrap_or(0);
                let callback_args = args.iter().skip(2).cloned().collect::<Vec<_>>();
                let timer_id = self.schedule_timeout(callback, delay, callback_args, &timer_env);
                Some(Value::Number(timer_id))
            }
            "global_set_interval" => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "setInterval requires at least one argument".into(),
                    ));
                }
                let (callback, timer_env) =
                    self.timer_callback_from_args("setInterval", args, caller_env)?;
                let delay = args.get(1).map(Self::value_to_i64).unwrap_or(0);
                let callback_args = args.iter().skip(2).cloned().collect::<Vec<_>>();
                let timer_id = self.schedule_interval(callback, delay, callback_args, &timer_env);
                Some(Value::Number(timer_id))
            }
            "global_cancel_animation_frame" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "cancelAnimationFrame requires exactly one argument".into(),
                    ));
                }
                self.clear_timeout(Self::value_to_i64(&args[0]));
                Some(Value::Undefined)
            }
            "global_clear_interval" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "clearInterval requires exactly one argument".into(),
                    ));
                }
                self.clear_timeout(Self::value_to_i64(&args[0]));
                Some(Value::Undefined)
            }
            "global_clear_timeout" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "clearTimeout requires exactly one argument".into(),
                    ));
                }
                self.clear_timeout(Self::value_to_i64(&args[0]));
                Some(Value::Undefined)
            }
            "global_queue_microtask" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "queueMicrotask requires exactly one argument".into(),
                    ));
                }
                if !self.is_callable_value(&args[0]) {
                    return Err(Error::ScriptRuntime(
                        "queueMicrotask callback must be callable".into(),
                    ));
                }
                self.queue_callable_microtask(args[0].clone());
                Some(Value::Undefined)
            }
            _ => None,
        };
        let _ = event;
        Ok(value)
    }
}
