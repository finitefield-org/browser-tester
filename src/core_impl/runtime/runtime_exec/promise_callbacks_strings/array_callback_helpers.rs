use super::*;

impl Harness {
    fn callback_non_tdz_shadowed_names(callback: &ScriptHandler) -> HashSet<String> {
        let mut names = Self::collect_var_declared_names(&callback.stmts);
        names.extend(callback.params.iter().map(|param| param.name.clone()));
        names.extend(Self::collect_function_decls(&callback.stmts).into_keys());
        names
    }

    fn callback_local_bindings(callback: &ScriptHandler) -> HashSet<String> {
        Self::collect_function_scope_bindings(callback)
    }

    fn project_callback_pending_updates_to_env(
        &mut self,
        env: &mut HashMap<String, Value>,
        preserve_before: Option<&HashMap<String, Value>>,
        local_bindings: &HashSet<String>,
    ) {
        if self.script_runtime.listener_capture_env_stack.is_empty() {
            return;
        }

        let start = Self::pending_listener_capture_scope_start(env)
            .min(self.script_runtime.listener_capture_env_stack.len());
        let mut updates = HashMap::new();
        for frame in &self.script_runtime.listener_capture_env_stack[start..] {
            updates.extend(frame.pending_env_updates.clone());
        }
        for (name, value) in updates {
            if Self::is_internal_env_key(&name) {
                continue;
            }
            if local_bindings.contains(name.as_str()) {
                continue;
            }
            if let Some(before) = preserve_before {
                let changed_since_before = match (before.get(&name), env.get(&name)) {
                    (Some(prev), Some(current)) => !self.strict_equal(prev, current),
                    (None, Some(_)) | (Some(_), None) => true,
                    (None, None) => false,
                };
                if changed_since_before {
                    continue;
                }
            }
            if let Some(value) = value {
                env.insert(name, value);
            } else {
                env.remove(&name);
            }
        }
    }

    fn discard_callback_local_pending_updates(&mut self, local_bindings: &HashSet<String>) {
        let Some(frame) = self.script_runtime.listener_capture_env_stack.last_mut() else {
            return;
        };
        for name in local_bindings {
            frame.pending_env_updates.remove(name);
        }
    }

    fn sync_callback_env_back_to_outer(
        &mut self,
        before: &HashMap<String, Value>,
        after: &HashMap<String, Value>,
        outer_env: &mut HashMap<String, Value>,
        local_bindings: &HashSet<String>,
    ) {
        let mut names = before
            .keys()
            .chain(after.keys())
            .filter(|name| !Self::is_internal_env_key(name))
            .cloned()
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();

        for name in names {
            if local_bindings.contains(name.as_str()) {
                continue;
            }

            let before_value = before.get(&name);
            let after_value = after.get(&name);
            let changed = match (before_value, after_value) {
                (Some(prev), Some(next)) => !self.strict_equal(prev, next),
                (None, Some(_)) | (Some(_), None) => true,
                (None, None) => false,
            };
            if !changed {
                continue;
            }

            match after_value.cloned() {
                Some(next) => {
                    outer_env.insert(name.clone(), next.clone());
                    self.sync_arguments_after_param_write(outer_env, &name, &next);
                    self.sync_global_binding_if_needed(outer_env, &name, &next);
                    self.sync_scheduled_task_captures_for_binding_if_escaping(
                        outer_env, &name, &next,
                    );
                }
                None => {
                    outer_env.remove(&name);
                    self.sync_global_binding_if_needed(outer_env, &name, &Value::Undefined);
                    self.sync_scheduled_task_captures_for_binding_if_escaping(
                        outer_env,
                        &name,
                        &Value::Undefined,
                    );
                }
            }
        }
    }

    fn queue_callback_env_updates_to_outer_scope(
        &mut self,
        before: &HashMap<String, Value>,
        after: &HashMap<String, Value>,
        outer_env: &HashMap<String, Value>,
        local_bindings: &HashSet<String>,
    ) {
        let mut names = before
            .keys()
            .chain(after.keys())
            .filter(|name| !Self::is_internal_env_key(name))
            .cloned()
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();

        for name in names {
            if local_bindings.contains(name.as_str()) {
                continue;
            }

            let before_value = before.get(&name);
            let after_value = after.get(&name);
            let changed = match (before_value, after_value) {
                (Some(prev), Some(next)) => !self.strict_equal(prev, next),
                (None, Some(_)) | (Some(_), None) => true,
                (None, None) => false,
            };
            if !changed {
                continue;
            }

            if let Some(frame) = self.script_runtime.listener_capture_env_stack.last_mut() {
                frame
                    .pending_env_updates
                    .insert(name.clone(), after_value.cloned());
            } else {
                self.script_runtime
                    .expression_env_overrides
                    .insert(name.clone(), after_value.cloned());
            }

            match after_value.cloned() {
                Some(next) => {
                    self.sync_global_binding_if_needed(outer_env, &name, &next);
                    self.sync_scheduled_task_captures_for_binding_if_escaping(
                        outer_env, &name, &next,
                    );
                }
                None => {
                    self.sync_global_binding_if_needed(outer_env, &name, &Value::Undefined);
                    self.sync_scheduled_task_captures_for_binding_if_escaping(
                        outer_env,
                        &name,
                        &Value::Undefined,
                    );
                }
            }
        }
    }

    pub(crate) fn execute_array_callback(
        &mut self,
        callback: &ScriptHandler,
        args: &[Value],
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Value> {
        let local_bindings = Self::callback_local_bindings(callback);
        let mut callback_env = env.clone();
        self.project_callback_pending_updates_to_env(&mut callback_env, None, &local_bindings);
        let callback_before = callback_env.clone();
        callback_env.remove(INTERNAL_RETURN_SLOT);
        if !local_bindings.is_empty() {
            let mut local_binding_names = local_bindings.iter().cloned().collect::<Vec<_>>();
            local_binding_names.sort();
            callback_env.insert(
                INTERNAL_LOCAL_BINDINGS_KEY.to_string(),
                Self::new_array_value(local_binding_names.into_iter().map(Value::String).collect()),
            );
        }
        let mut callback_event = event.clone();
        let event_param = None;
        let previous_pending_scope_start = callback_env.insert(
            INTERNAL_PENDING_SCOPE_START_KEY.to_string(),
            Value::Number(self.script_runtime.listener_capture_env_stack.len() as i64),
        );
        let return_value = self.with_isolated_loop_control_scope(|this| {
            this.bind_handler_params(
                callback,
                args,
                &mut callback_env,
                &event_param,
                &callback_event,
            )?;
            let non_tdz_shadowed = Self::callback_non_tdz_shadowed_names(callback);
            let pushed_non_tdz_scope = !non_tdz_shadowed.is_empty();
            if pushed_non_tdz_scope {
                this.script_runtime.tdz_scope_stack.push(TdzScopeFrame {
                    declared: non_tdz_shadowed,
                    pending: HashSet::new(),
                });
            }
            let shared_env_frame_start = this.push_shared_listener_capture_env_frame(
                Rc::new(RefCell::new(ScriptEnv::from_snapshot(&callback_env))),
                true,
            );
            let flow = this.execute_stmts(
                &callback.stmts,
                &event_param,
                &mut callback_event,
                &mut callback_env,
            );
            let return_value = flow
                .as_ref()
                .ok()
                .filter(|flow| matches!(flow, ExecFlow::Return))
                .and_then(|_| callback_env.get(INTERNAL_RETURN_SLOT).cloned())
                .unwrap_or(Value::Undefined);
            this.restore_listener_capture_env_stack(shared_env_frame_start);
            this.discard_callback_local_pending_updates(&local_bindings);
            if pushed_non_tdz_scope {
                this.script_runtime.tdz_scope_stack.pop();
            }
            match flow? {
                ExecFlow::Continue | ExecFlow::Return => {}
                ExecFlow::Break(label) => return Err(Self::break_flow_error(&label)),
                ExecFlow::ContinueLoop(label) => return Err(Self::continue_flow_error(&label)),
            }
            Ok(return_value)
        })?;
        match previous_pending_scope_start {
            Some(value) => {
                callback_env.insert(INTERNAL_PENDING_SCOPE_START_KEY.to_string(), value);
            }
            None => {
                callback_env.remove(INTERNAL_PENDING_SCOPE_START_KEY);
            }
        }
        self.project_callback_pending_updates_to_env(
            &mut callback_env,
            Some(&callback_before),
            &local_bindings,
        );
        self.queue_callback_env_updates_to_outer_scope(
            &callback_before,
            &callback_env,
            env,
            &local_bindings,
        );
        callback_env.remove(INTERNAL_RETURN_SLOT);
        Ok(return_value)
    }

    pub(crate) fn execute_array_callback_in_env(
        &mut self,
        callback: &ScriptHandler,
        args: &[Value],
        env: &mut HashMap<String, Value>,
        event: &EventState,
    ) -> Result<()> {
        let local_bindings = Self::callback_local_bindings(callback);
        self.project_callback_pending_updates_to_env(env, None, &local_bindings);
        let callback_before = env.clone();
        let mut callback_env = env.clone();
        callback_env.remove(INTERNAL_RETURN_SLOT);
        if !local_bindings.is_empty() {
            let mut local_binding_names = local_bindings.iter().cloned().collect::<Vec<_>>();
            local_binding_names.sort();
            callback_env.insert(
                INTERNAL_LOCAL_BINDINGS_KEY.to_string(),
                Self::new_array_value(local_binding_names.into_iter().map(Value::String).collect()),
            );
        }
        let mut callback_event = event.clone();
        let event_param = None;
        let previous_pending_scope_start = callback_env.insert(
            INTERNAL_PENDING_SCOPE_START_KEY.to_string(),
            Value::Number(self.script_runtime.listener_capture_env_stack.len() as i64),
        );
        let result = self.with_isolated_loop_control_scope(|this| {
            this.bind_handler_params(
                callback,
                args,
                &mut callback_env,
                &event_param,
                &callback_event,
            )?;
            let non_tdz_shadowed = Self::callback_non_tdz_shadowed_names(callback);
            let pushed_non_tdz_scope = !non_tdz_shadowed.is_empty();
            if pushed_non_tdz_scope {
                this.script_runtime.tdz_scope_stack.push(TdzScopeFrame {
                    declared: non_tdz_shadowed,
                    pending: HashSet::new(),
                });
            }
            let shared_env_frame_start = this.push_shared_listener_capture_env_frame(
                Rc::new(RefCell::new(ScriptEnv::from_snapshot(&callback_env))),
                true,
            );
            let flow = this.execute_stmts(
                &callback.stmts,
                &event_param,
                &mut callback_event,
                &mut callback_env,
            );
            this.restore_listener_capture_env_stack(shared_env_frame_start);
            this.discard_callback_local_pending_updates(&local_bindings);
            if pushed_non_tdz_scope {
                this.script_runtime.tdz_scope_stack.pop();
            }
            flow
        });
        match previous_pending_scope_start {
            Some(value) => {
                callback_env.insert(INTERNAL_PENDING_SCOPE_START_KEY.to_string(), value);
            }
            None => {
                callback_env.remove(INTERNAL_PENDING_SCOPE_START_KEY);
            }
        }
        self.project_callback_pending_updates_to_env(
            &mut callback_env,
            Some(&callback_before),
            &local_bindings,
        );
        callback_env.remove(INTERNAL_RETURN_SLOT);
        self.sync_callback_env_back_to_outer(&callback_before, &callback_env, env, &local_bindings);

        match result? {
            ExecFlow::Continue | ExecFlow::Return => Ok(()),
            ExecFlow::Break(label) => Err(Self::break_flow_error(&label)),
            ExecFlow::ContinueLoop(label) => Err(Self::continue_flow_error(&label)),
        }
    }

    pub(crate) fn execute_array_like_foreach_in_env(
        &mut self,
        target_value: Value,
        callback: &ScriptHandler,
        env: &mut HashMap<String, Value>,
        event: &EventState,
        target_label: &str,
    ) -> Result<()> {
        match target_value {
            Value::NodeList(nodes) => {
                let snapshot = self.node_list_snapshot(&nodes);
                for (idx, node) in snapshot.into_iter().enumerate() {
                    self.execute_array_callback_in_env(
                        callback,
                        &[
                            Value::Node(node),
                            Value::Number(idx as i64),
                            Value::NodeList(nodes.clone()),
                        ],
                        env,
                        event,
                    )?;
                }
            }
            Value::Array(values) => {
                let input = values.borrow().clone();
                for (idx, item) in input.into_iter().enumerate() {
                    self.execute_array_callback_in_env(
                        callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        env,
                        event,
                    )?;
                }
            }
            Value::TypedArray(values) => {
                let input = self.typed_array_snapshot(&values)?;
                for (idx, item) in input.into_iter().enumerate() {
                    self.execute_array_callback_in_env(
                        callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::TypedArray(values.clone()),
                        ],
                        env,
                        event,
                    )?;
                }
            }
            Value::Map(map) => {
                let snapshot = map.borrow().entries.clone();
                for (key, value) in snapshot {
                    self.execute_array_callback_in_env(
                        callback,
                        &[value, key, Value::Map(map.clone())],
                        env,
                        event,
                    )?;
                }
            }
            Value::Set(set) => {
                let snapshot = set.borrow().values.clone();
                for value in snapshot {
                    self.execute_array_callback_in_env(
                        callback,
                        &[value.clone(), value, Value::Set(set.clone())],
                        env,
                        event,
                    )?;
                }
            }
            Value::Object(entries) => {
                if Self::is_iterator_object(&entries.borrow()) {
                    let snapshot = self.iterator_collect_remaining_values(&entries)?;
                    for (idx, value) in snapshot.into_iter().enumerate() {
                        self.execute_array_callback_in_env(
                            callback,
                            &[
                                value,
                                Value::Number(idx as i64),
                                Value::Object(entries.clone()),
                            ],
                            env,
                            event,
                        )?;
                    }
                } else if Self::is_url_search_params_object(&entries.borrow()) {
                    let snapshot =
                        Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                    for (key, value) in snapshot {
                        self.execute_array_callback_in_env(
                            callback,
                            &[
                                Value::String(value),
                                Value::String(key),
                                Value::Object(entries.clone()),
                            ],
                            env,
                            event,
                        )?;
                    }
                } else {
                    return Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target_label
                    )));
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target_label
                )));
            }
        }
        Ok(())
    }
}
