impl Harness {
    pub(crate) fn execute_array_callback(
        &mut self,
        callback: &ScriptHandler,
        args: &[Value],
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Value> {
        let mut callback_env = env.clone();
        callback_env.remove(INTERNAL_RETURN_SLOT);
        let mut callback_event = event.clone();
        let event_param = None;
        self.bind_handler_params(
            callback,
            args,
            &mut callback_env,
            &event_param,
            &callback_event,
        )?;
        match self.execute_stmts(
            &callback.stmts,
            &event_param,
            &mut callback_event,
            &mut callback_env,
        )? {
            ExecFlow::Continue | ExecFlow::Return => {}
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

        Ok(callback_env
            .remove(INTERNAL_RETURN_SLOT)
            .unwrap_or(Value::Undefined))
    }

    pub(crate) fn execute_array_callback_in_env(
        &mut self,
        callback: &ScriptHandler,
        args: &[Value],
        env: &mut HashMap<String, Value>,
        event: &EventState,
    ) -> Result<()> {
        let mut previous_values = Vec::with_capacity(callback.params.len());
        for param in &callback.params {
            previous_values.push((param.name.clone(), env.get(&param.name).cloned()));
        }

        let mut callback_event = event.clone();
        let event_param = None;
        self.bind_handler_params(callback, args, env, &event_param, &callback_event)?;
        let result = self.execute_stmts(&callback.stmts, &event_param, &mut callback_event, env);
        env.remove(INTERNAL_RETURN_SLOT);

        for (name, previous) in previous_values {
            if let Some(previous) = previous {
                env.insert(name, previous);
            } else {
                env.remove(&name);
            }
        }

        match result? {
            ExecFlow::Continue | ExecFlow::Return => Ok(()),
            ExecFlow::Break => Err(Error::ScriptRuntime(
                "break statement outside of loop".into(),
            )),
            ExecFlow::ContinueLoop => Err(Error::ScriptRuntime(
                "continue statement outside of loop".into(),
            )),
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
                let snapshot = nodes.clone();
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
                if Self::is_url_search_params_object(&entries.borrow()) {
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
