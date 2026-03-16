use super::*;

impl Harness {
    pub(crate) fn invoke_listeners(
        &mut self,
        node_id: NodeId,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
        capture: bool,
    ) -> Result<()> {
        let listeners = self.listeners.get(node_id, &event.event_type, capture);
        for listener in listeners {
            let mut listener_env = env.clone();
            self.project_pending_listener_capture_env_updates(&mut listener_env);
            if !listener.is_arrow {
                let this_value = event
                    .current_target_value
                    .as_ref()
                    .cloned()
                    .unwrap_or(Value::Node(event.current_target));
                listener_env.insert("this".to_string(), this_value);
                self.set_const_binding(&mut listener_env, "this", false);
            }
            let captured_env_snapshot = listener.captured_env.borrow().to_map();
            let captured_keys = listener.captured_names.iter().cloned().collect::<Vec<_>>();
            for name in &captured_keys {
                if let Some(value) = captured_env_snapshot.get(name) {
                    if !listener_env.contains_key(name) {
                        listener_env.insert(name.clone(), value.clone());
                    }
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
            let used_function_dispatch = listener.function.is_some();
            let call_result = if let Some(function) = listener.function.as_ref() {
                {
                    let mut captured_env = listener.captured_env.borrow_mut();
                    for (name, value) in &listener_env {
                        if Self::is_internal_env_key(name)
                            || function.local_bindings.contains(name.as_str())
                            || Self::env_has_local_binding(&captured_env_snapshot, name)
                            || function.captured_global_names.contains(name.as_str())
                            || matches!(name.as_str(), "this" | "arguments")
                            || !function.captured_names.contains(name)
                        {
                            continue;
                        }
                        captured_env.insert(name.clone(), value.clone());
                    }
                }
                let event_param = function.handler.first_event_param();
                let event_args = if event_param.is_some() {
                    vec![self.listener_event_argument(event)]
                } else {
                    Vec::new()
                };
                let this_value = event
                    .current_target_value
                    .as_ref()
                    .cloned()
                    .unwrap_or(Value::Node(event.current_target));
                let event_snapshot = event.clone();
                self.execute_function_call(
                    function.clone(),
                    &event_args,
                    &event_snapshot,
                    Some(&listener_env),
                    Some(this_value),
                    None,
                    Some(event),
                )
                .map(|_| ())
            } else {
                let pending_scope_start = self
                    .push_pending_function_decl_scopes(&listener.captured_pending_function_decls);
                let shared_env_frame_start = self
                    .push_shared_listener_capture_env_frame_with_names(
                        listener.captured_env.clone(),
                        false,
                        Some(listener.captured_names.clone()),
                    );
                let result = self.execute_handler(&listener.handler, event, &mut listener_env);
                self.restore_listener_capture_env_stack(shared_env_frame_start);
                self.restore_pending_function_decl_scopes(pending_scope_start);
                result
            };
            if !used_function_dispatch {
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
            if used_function_dispatch {
                self.apply_expression_env_overrides_to_env(env);
                self.apply_pending_listener_capture_env_updates(env);
                let captured_env_after = listener.captured_env.borrow().to_map();
                for key in &captured_keys {
                    let before = captured_env_snapshot.get(key);
                    let after = captured_env_after.get(key);
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
                        env.insert(key.clone(), value);
                    } else {
                        env.remove(key);
                    }
                }
                listener_env = env.clone();
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

    pub(crate) fn trace_event_done(&mut self, event: &EventState, outcome: &str) {
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

    pub(crate) fn trace_event_line(&mut self, line: String) {
        if self.trace_state.enabled && self.trace_state.events {
            self.trace_line(line);
        }
    }

    pub(crate) fn trace_timer_line(&mut self, line: String) {
        if self.trace_state.enabled && self.trace_state.timers {
            self.trace_line(line);
        }
    }

    pub(crate) fn trace_line(&mut self, line: String) {
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
}
