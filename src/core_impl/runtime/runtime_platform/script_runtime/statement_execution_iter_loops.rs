use super::*;

impl Harness {
    pub(super) fn execute_for_in_stmt(
        &mut self,
        item_var: &str,
        iterable: &Expr,
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        self.with_loop_label_scope(|this| {
            let iterable = this.eval_expr(iterable, env, event_param, event)?;
            let items = match iterable {
                Value::NodeList(nodes) => (0..this.node_list_len(&nodes))
                    .map(|idx| Value::String(idx.to_string()))
                    .collect::<Vec<_>>(),
                Value::Array(values) => this
                    .collect_for_in_array_keys(&values)
                    .into_iter()
                    .map(Value::String)
                    .collect::<Vec<_>>(),
                Value::Object(entries) => this
                    .collect_for_in_object_chain_keys(&entries)
                    .into_iter()
                    .map(Value::String)
                    .collect::<Vec<_>>(),
                Value::Null | Value::Undefined => Vec::new(),
                _ => {
                    return Err(Error::ScriptRuntime(
                        "for...in iterable must be a NodeList, Array, or Object".into(),
                    ));
                }
            };

            let prev_item = env.get(item_var).cloned();
            for item_value in items {
                env.insert(item_var.to_string(), item_value.clone());
                this.sync_global_binding_if_needed(env, item_var, &item_value);
                match this.execute_stmts_with_pending_scope(body, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::ContinueLoop(label) => {
                        if this.loop_should_consume_continue(&label) {
                            continue;
                        }
                        return Ok(ExecFlow::ContinueLoop(label));
                    }
                    ExecFlow::Break(label) => {
                        if this.loop_should_consume_break(&label) {
                            break;
                        }
                        return Ok(ExecFlow::Break(label));
                    }
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                }
            }
            if let Some(prev) = prev_item {
                env.insert(item_var.to_string(), prev.clone());
                this.sync_global_binding_if_needed(env, item_var, &prev);
            } else {
                env.remove(item_var);
            }
            Ok(ExecFlow::Continue)
        })
    }

    pub(super) fn execute_for_of_stmt(
        &mut self,
        item_var: &str,
        iterable: &Expr,
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        self.with_loop_label_scope(|this| {
            enum ForOfSource {
                Values(Vec<Value>),
                InternalIterator(Rc<RefCell<ObjectValue>>),
                ProtocolIterator(Rc<RefCell<ObjectValue>>),
            }

            let iterable = this.eval_expr(iterable, env, event_param, event)?;
            let source = match iterable {
                Value::NodeList(nodes) => ForOfSource::Values(
                    this.node_list_snapshot(&nodes)
                        .into_iter()
                        .map(Value::Node)
                        .collect::<Vec<_>>(),
                ),
                Value::Array(values) => ForOfSource::Values(values.borrow().clone()),
                Value::String(text) => ForOfSource::Values(
                    text.chars()
                        .map(|ch| Value::String(ch.to_string()))
                        .collect::<Vec<_>>(),
                ),
                Value::TypedArray(values) => {
                    ForOfSource::Values(this.typed_array_snapshot(&values)?)
                }
                Value::Map(map) => ForOfSource::Values(this.map_entries_array(&map)),
                Value::Set(set) => ForOfSource::Values(set.borrow().values.clone()),
                Value::Object(entries) => {
                    if Self::is_iterator_object(&entries.borrow()) {
                        ForOfSource::InternalIterator(entries)
                    } else if Self::is_url_search_params_object(&entries.borrow()) {
                        ForOfSource::Values(
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow())
                                .into_iter()
                                .map(|(key, value)| {
                                    Self::new_array_value(vec![
                                        Value::String(key),
                                        Value::String(value),
                                    ])
                                })
                                .collect::<Vec<_>>(),
                        )
                    } else if let Some(iterator) =
                        this.for_of_symbol_iterator_factory_result(&entries, event)?
                    {
                        if Self::is_iterator_object(&iterator.borrow()) {
                            ForOfSource::InternalIterator(iterator)
                        } else {
                            ForOfSource::ProtocolIterator(iterator)
                        }
                    } else {
                        return Err(Error::ScriptRuntime(
                            "for...of iterable must be an Iterator, NodeList, Array, String, TypedArray, Map, Set, or URLSearchParams"
                                .into(),
                        ));
                    }
                }
                Value::Null | Value::Undefined => {
                    return Err(Error::ScriptRuntime(
                        "for...of iterable must be an Iterator, NodeList, Array, String, TypedArray, Map, Set, or URLSearchParams".into(),
                    ));
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "for...of iterable must be an Iterator, NodeList, Array, String, TypedArray, Map, Set, or URLSearchParams"
                            .into(),
                    ));
                }
            };

            let prev_item = env.get(item_var).cloned();
            let prev_local_bindings = env.get(INTERNAL_LOCAL_BINDINGS_KEY).cloned();
            let mut local_binding_names = prev_local_bindings
                .as_ref()
                .and_then(|value| match value {
                    Value::Array(bindings) => Some(
                        bindings
                            .borrow()
                            .iter()
                            .filter_map(|entry| match entry {
                                Value::String(name) => Some(name.clone()),
                                _ => None,
                            })
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                })
                .unwrap_or_default();
            if !local_binding_names.iter().any(|name| name == item_var) {
                local_binding_names.push(item_var.to_string());
            }
            local_binding_names.sort();
            local_binding_names.dedup();
            env.insert(
                INTERNAL_LOCAL_BINDINGS_KEY.to_string(),
                Self::new_array_value(local_binding_names.into_iter().map(Value::String).collect()),
            );
            let loop_result = (|| -> Result<ExecFlow> {
                match source {
                    ForOfSource::Values(items) => {
                        for item in items {
                            env.insert(item_var.to_string(), item.clone());
                            this.sync_global_binding_if_needed(env, item_var, &item);
                            let shared_env_frame_start = this.push_shared_listener_capture_env_frame(
                                Rc::new(RefCell::new(ScriptEnv::from_snapshot(env))),
                                true,
                            );
                            let flow = this.execute_stmts_with_pending_scope(
                                body,
                                event_param,
                                event,
                                env,
                                false,
                            );
                            this.restore_listener_capture_env_stack(shared_env_frame_start);
                            match flow? {
                                ExecFlow::Continue => {}
                                ExecFlow::ContinueLoop(label) => {
                                    if this.loop_should_consume_continue(&label) {
                                        continue;
                                    }
                                    return Ok(ExecFlow::ContinueLoop(label));
                                }
                                ExecFlow::Break(label) => {
                                    if this.loop_should_consume_break(&label) {
                                        break;
                                    }
                                    return Ok(ExecFlow::Break(label));
                                }
                                ExecFlow::Return => return Ok(ExecFlow::Return),
                            }
                        }
                        Ok(ExecFlow::Continue)
                    }
                    ForOfSource::InternalIterator(iterator) => {
                        loop {
                            let Some(item) = this.iterator_next_value_from_object(&iterator)? else {
                                break;
                            };
                            env.insert(item_var.to_string(), item.clone());
                            this.sync_global_binding_if_needed(env, item_var, &item);
                            let shared_env_frame_start = this.push_shared_listener_capture_env_frame(
                                Rc::new(RefCell::new(ScriptEnv::from_snapshot(env))),
                                true,
                            );
                            let flow = match this.execute_stmts_with_pending_scope(
                                body,
                                event_param,
                                event,
                                env,
                                false,
                            ) {
                                Ok(flow) => flow,
                                Err(err) => {
                                    this.restore_listener_capture_env_stack(shared_env_frame_start);
                                    this.for_of_internal_iterator_close_if_needed(&iterator, event)?;
                                    return Err(err);
                                }
                            };
                            this.restore_listener_capture_env_stack(shared_env_frame_start);
                            match flow {
                                ExecFlow::Continue => {}
                                ExecFlow::ContinueLoop(label) => {
                                    if this.loop_should_consume_continue(&label) {
                                        continue;
                                    }
                                    this.for_of_internal_iterator_close_if_needed(&iterator, event)?;
                                    return Ok(ExecFlow::ContinueLoop(label));
                                }
                                ExecFlow::Break(label) => {
                                    this.for_of_internal_iterator_close_if_needed(&iterator, event)?;
                                    if this.loop_should_consume_break(&label) {
                                        break;
                                    }
                                    return Ok(ExecFlow::Break(label));
                                }
                                ExecFlow::Return => {
                                    this.for_of_internal_iterator_close_if_needed(&iterator, event)?;
                                    return Ok(ExecFlow::Return);
                                }
                            }
                        }
                        Ok(ExecFlow::Continue)
                    }
                    ForOfSource::ProtocolIterator(iterator) => {
                        loop {
                            let Some(item) = this.for_of_protocol_iterator_next(&iterator, event)? else {
                                break;
                            };
                            env.insert(item_var.to_string(), item.clone());
                            this.sync_global_binding_if_needed(env, item_var, &item);
                            let shared_env_frame_start = this.push_shared_listener_capture_env_frame(
                                Rc::new(RefCell::new(ScriptEnv::from_snapshot(env))),
                                true,
                            );
                            let flow = match this.execute_stmts_with_pending_scope(
                                body,
                                event_param,
                                event,
                                env,
                                false,
                            ) {
                                Ok(flow) => flow,
                                Err(err) => {
                                    this.restore_listener_capture_env_stack(shared_env_frame_start);
                                    this.for_of_protocol_iterator_close(&iterator, event)?;
                                    return Err(err);
                                }
                            };
                            this.restore_listener_capture_env_stack(shared_env_frame_start);
                            match flow {
                                ExecFlow::Continue => {}
                                ExecFlow::ContinueLoop(label) => {
                                    if this.loop_should_consume_continue(&label) {
                                        continue;
                                    }
                                    this.for_of_protocol_iterator_close(&iterator, event)?;
                                    return Ok(ExecFlow::ContinueLoop(label));
                                }
                                ExecFlow::Break(label) => {
                                    this.for_of_protocol_iterator_close(&iterator, event)?;
                                    if this.loop_should_consume_break(&label) {
                                        break;
                                    }
                                    return Ok(ExecFlow::Break(label));
                                }
                                ExecFlow::Return => {
                                    this.for_of_protocol_iterator_close(&iterator, event)?;
                                    return Ok(ExecFlow::Return);
                                }
                            }
                        }
                        Ok(ExecFlow::Continue)
                    }
                }
            })();
            if let Some(prev) = prev_item {
                env.insert(item_var.to_string(), prev.clone());
                this.sync_global_binding_if_needed(env, item_var, &prev);
            } else {
                env.remove(item_var);
            }
            match prev_local_bindings {
                Some(value) => {
                    env.insert(INTERNAL_LOCAL_BINDINGS_KEY.to_string(), value);
                }
                None => {
                    env.remove(INTERNAL_LOCAL_BINDINGS_KEY);
                }
            }
            loop_result
        })
    }

    pub(super) fn execute_for_await_of_stmt(
        &mut self,
        item_var: &str,
        iterable: &Expr,
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        self.with_loop_label_scope(|this| {
            let iterable = this.eval_expr(iterable, env, event_param, event)?;
            let values = match iterable {
                Value::NodeList(nodes) => this
                    .node_list_snapshot(&nodes)
                    .into_iter()
                    .map(Value::Node)
                    .collect::<Vec<_>>(),
                Value::Array(values) => values.borrow().clone(),
                Value::Map(map) => this.map_entries_array(&map),
                Value::Set(set) => set.borrow().values.clone(),
                Value::Object(entries) => {
                    if Self::is_async_iterator_object(&entries.borrow()) {
                        let mut out = Vec::new();
                        while let Some(value) =
                            this.async_iterator_next_value_from_object(&entries)?
                        {
                            out.push(value);
                        }
                        out
                    } else {
                        let async_iterator_symbol =
                            this.eval_symbol_static_property(SymbolStaticProperty::AsyncIterator);
                        let async_iterator_key =
                            this.property_key_to_storage_key(&async_iterator_symbol);
                        let async_iterator_factory = {
                            let entries_ref = entries.borrow();
                            Self::object_get_entry(&entries_ref, async_iterator_key.as_str())
                        };

                        if let Some(factory) = async_iterator_factory {
                            if !this.is_callable_value(&factory) {
                                return Err(Error::ScriptRuntime(
                                    "for await...of async iterator factory is not callable".into(),
                                ));
                            }
                            let iterator_value = this.execute_callable_value(&factory, &[], event)?;
                            let Value::Object(async_iterator) = iterator_value else {
                                return Err(Error::ScriptRuntime(
                                    "for await...of async iterator factory must return an object"
                                        .into(),
                                ));
                            };
                            if !Self::is_async_iterator_object(&async_iterator.borrow()) {
                                return Err(Error::ScriptRuntime(
                                    "for await...of async iterator factory returned a non-async iterator"
                                        .into(),
                                ));
                            }
                            let mut out = Vec::new();
                            while let Some(value) =
                                this.async_iterator_next_value_from_object(&async_iterator)?
                            {
                                out.push(value);
                            }
                            out
                        } else if Self::is_iterator_object(&entries.borrow()) {
                            this.iterator_collect_remaining_values(&entries)?
                        } else if Self::is_url_search_params_object(&entries.borrow()) {
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
                                "for await...of iterable must be an AsyncIterator, Iterator, NodeList, Array, Map, Set, or URLSearchParams".into(),
                            ));
                        }
                    }
                }
                Value::Null | Value::Undefined => Vec::new(),
                _ => {
                    return Err(Error::ScriptRuntime(
                        "for await...of iterable must be an AsyncIterator, Iterator, NodeList, Array, Map, Set, or URLSearchParams".into(),
                    ));
                }
            };

            let prev_item = env.get(item_var).cloned();
            for value in values {
                let item = this.await_value_in_for_await(value)?;
                env.insert(item_var.to_string(), item.clone());
                this.sync_global_binding_if_needed(env, item_var, &item);
                match this.execute_stmts_with_pending_scope(body, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::ContinueLoop(label) => {
                        if this.loop_should_consume_continue(&label) {
                            continue;
                        }
                        return Ok(ExecFlow::ContinueLoop(label));
                    }
                    ExecFlow::Break(label) => {
                        if this.loop_should_consume_break(&label) {
                            break;
                        }
                        return Ok(ExecFlow::Break(label));
                    }
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                }
            }
            if let Some(prev) = prev_item {
                env.insert(item_var.to_string(), prev.clone());
                this.sync_global_binding_if_needed(env, item_var, &prev);
            } else {
                env.remove(item_var);
            }
            Ok(ExecFlow::Continue)
        })
    }
}
