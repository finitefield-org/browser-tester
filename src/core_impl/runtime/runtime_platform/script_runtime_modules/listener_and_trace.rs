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
