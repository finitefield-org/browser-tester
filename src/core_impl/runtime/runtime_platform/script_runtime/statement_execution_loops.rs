use super::*;

impl Harness {
    fn with_loop_label_scope<T>(&mut self, run: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        let loop_labels = self.take_pending_loop_labels();
        self.push_loop_label_scope(loop_labels);
        let result = run(self);
        self.pop_loop_label_scope();
        result
    }

    fn execute_class_list_for_each_stmt(
        &mut self,
        target: &DomQuery,
        optional: bool,
        item_var: &str,
        index_var: &Option<String>,
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let node = if optional {
            if let DomQuery::Var(name) = target {
                if matches!(env.get(name), Some(Value::Null | Value::Undefined)) {
                    return Ok(ExecFlow::Continue);
                }
            }
            match self.resolve_dom_query_runtime(target, env)? {
                Some(node) => node,
                None => return Ok(ExecFlow::Continue),
            }
        } else {
            self.resolve_dom_query_required_runtime(target, env)?
        };
        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
        let prev_item = env.get(item_var).cloned();
        let prev_item_const = self.is_const_binding(env, item_var);
        let prev_index = index_var.as_ref().and_then(|var| env.get(var).cloned());
        let prev_index_const = index_var
            .as_ref()
            .is_some_and(|name| self.is_const_binding(env, name));
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
        if let Some(index_var) = index_var {
            if !local_binding_names.iter().any(|name| name == index_var) {
                local_binding_names.push(index_var.clone());
            }
        }
        local_binding_names.sort();
        local_binding_names.dedup();
        env.insert(
            INTERNAL_LOCAL_BINDINGS_KEY.to_string(),
            Self::new_array_value(local_binding_names.into_iter().map(Value::String).collect()),
        );
        self.set_const_binding(env, item_var, false);
        if let Some(index_var) = index_var {
            self.set_const_binding(env, index_var, false);
        }

        let loop_result = (|| -> Result<ExecFlow> {
            for (idx, class_name) in classes.iter().enumerate() {
                let item_value = Value::String(class_name.clone());
                env.insert(item_var.to_string(), item_value.clone());
                self.sync_global_binding_if_needed(env, item_var, &item_value);
                if let Some(index_var) = index_var {
                    let index_value = Value::Number(idx as i64);
                    env.insert(index_var.clone(), index_value.clone());
                    self.sync_global_binding_if_needed(env, index_var, &index_value);
                }
                match self.execute_stmts_with_pending_scope(body, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::Break(None) => break,
                    ExecFlow::Break(label) => return Ok(ExecFlow::Break(label)),
                    ExecFlow::ContinueLoop(None) => continue,
                    ExecFlow::ContinueLoop(label) => return Ok(ExecFlow::ContinueLoop(label)),
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                }
            }
            Ok(ExecFlow::Continue)
        })();

        if let Some(prev) = prev_item {
            env.insert(item_var.to_string(), prev.clone());
            self.sync_global_binding_if_needed(env, item_var, &prev);
        } else {
            env.remove(item_var);
        }
        self.set_const_binding(env, item_var, prev_item_const);
        if let Some(index_var) = index_var {
            if let Some(prev) = prev_index {
                env.insert(index_var.clone(), prev.clone());
                self.sync_global_binding_if_needed(env, index_var, &prev);
            } else {
                env.remove(index_var);
            }
            self.set_const_binding(env, index_var, prev_index_const);
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
    }

    fn execute_query_selector_for_each_stmt(
        &mut self,
        target: &Option<DomQuery>,
        selector: &str,
        item_var: &str,
        index_var: &Option<String>,
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let items = if let Some(target) = target {
            match self.resolve_dom_query_runtime(target, env)? {
                Some(target_node) => self.dom.query_selector_all_from(&target_node, selector)?,
                None => Vec::new(),
            }
        } else {
            self.dom.query_selector_all(selector)?
        };
        let prev_item = env.get(item_var).cloned();
        let prev_item_const = self.is_const_binding(env, item_var);
        let prev_index = index_var.as_ref().and_then(|var| env.get(var).cloned());
        let prev_index_const = index_var
            .as_ref()
            .is_some_and(|name| self.is_const_binding(env, name));
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
        if let Some(index_var) = index_var {
            if !local_binding_names.iter().any(|name| name == index_var) {
                local_binding_names.push(index_var.clone());
            }
        }
        local_binding_names.sort();
        local_binding_names.dedup();
        env.insert(
            INTERNAL_LOCAL_BINDINGS_KEY.to_string(),
            Self::new_array_value(local_binding_names.into_iter().map(Value::String).collect()),
        );
        self.set_const_binding(env, item_var, false);
        if let Some(index_var) = index_var {
            self.set_const_binding(env, index_var, false);
        }

        let loop_result = (|| -> Result<ExecFlow> {
            for (idx, node) in items.iter().enumerate() {
                let item_value = Value::Node(*node);
                env.insert(item_var.to_string(), item_value.clone());
                self.sync_global_binding_if_needed(env, item_var, &item_value);
                if let Some(index_var) = index_var {
                    let index_value = Value::Number(idx as i64);
                    env.insert(index_var.clone(), index_value.clone());
                    self.sync_global_binding_if_needed(env, index_var, &index_value);
                }
                match self.execute_stmts_with_pending_scope(body, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::Break(None) => break,
                    ExecFlow::Break(label) => return Ok(ExecFlow::Break(label)),
                    ExecFlow::ContinueLoop(None) => continue,
                    ExecFlow::ContinueLoop(label) => return Ok(ExecFlow::ContinueLoop(label)),
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                }
            }
            Ok(ExecFlow::Continue)
        })();

        if let Some(prev) = prev_item {
            env.insert(item_var.to_string(), prev.clone());
            self.sync_global_binding_if_needed(env, item_var, &prev);
        } else {
            env.remove(item_var);
        }
        self.set_const_binding(env, item_var, prev_item_const);
        if let Some(index_var) = index_var {
            if let Some(prev) = prev_index {
                env.insert(index_var.clone(), prev.clone());
                self.sync_global_binding_if_needed(env, index_var, &prev);
            } else {
                env.remove(index_var);
            }
            self.set_const_binding(env, index_var, prev_index_const);
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
    }

    fn execute_for_stmt(
        &mut self,
        init: &[Stmt],
        cond: &Option<Expr>,
        post: &[Stmt],
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let previous_init_lexical = self.collect_direct_block_lexical_bindings(init, env);
        let result = self.with_loop_label_scope(|this| {
            if !init.is_empty() {
                match this.execute_stmts_with_pending_scope(init, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                    ExecFlow::Break(label) => return Err(Self::break_flow_error(&label)),
                    ExecFlow::ContinueLoop(label) => {
                        return Err(Self::continue_flow_error(&label));
                    }
                }
            }

            loop {
                let should_run = if let Some(cond) = cond {
                    this.eval_expr(cond, env, event_param, event)?.truthy()
                } else {
                    true
                };
                if !should_run {
                    break;
                }

                match this.execute_stmts_with_pending_scope(body, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::ContinueLoop(label) => {
                        if this.loop_should_consume_continue(&label) {
                            if !post.is_empty() {
                                match this.execute_stmts_with_pending_scope(
                                    post,
                                    event_param,
                                    event,
                                    env,
                                    false,
                                )? {
                                    ExecFlow::Continue => {}
                                    ExecFlow::Return => return Ok(ExecFlow::Return),
                                    ExecFlow::Break(_) | ExecFlow::ContinueLoop(_) => {
                                        return Err(Error::ScriptRuntime(
                                            "invalid loop control in post expression".into(),
                                        ));
                                    }
                                }
                            }
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
                if !post.is_empty() {
                    match this.execute_stmts_with_pending_scope(
                        post,
                        event_param,
                        event,
                        env,
                        false,
                    )? {
                        ExecFlow::Continue => {}
                        ExecFlow::Return => return Ok(ExecFlow::Return),
                        ExecFlow::Break(_) | ExecFlow::ContinueLoop(_) => {
                            return Err(Error::ScriptRuntime(
                                "invalid loop control in post expression".into(),
                            ));
                        }
                    }
                }
            }
            Ok(ExecFlow::Continue)
        });
        self.restore_block_lexical_bindings(previous_init_lexical, env);
        result
    }

    fn execute_for_in_stmt(
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

    fn execute_for_of_stmt(
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
                Value::TypedArray(values) => ForOfSource::Values(this.typed_array_snapshot(&values)?),
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
                            let flow =
                                this.execute_stmts_with_pending_scope(body, event_param, event, env, false);
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

    fn execute_for_await_of_stmt(
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
                        while let Some(value) = this.async_iterator_next_value_from_object(&entries)? {
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

    fn execute_while_stmt(
        &mut self,
        cond: &Expr,
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        self.with_loop_label_scope(|this| {
            while this.eval_expr(cond, env, event_param, event)?.truthy() {
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
            Ok(ExecFlow::Continue)
        })
    }

    fn execute_do_while_stmt(
        &mut self,
        cond: &Expr,
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        self.with_loop_label_scope(|this| {
            loop {
                match this.execute_stmts_with_pending_scope(body, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::ContinueLoop(label) => {
                        if !this.loop_should_consume_continue(&label) {
                            return Ok(ExecFlow::ContinueLoop(label));
                        }
                    }
                    ExecFlow::Break(label) => {
                        if this.loop_should_consume_break(&label) {
                            break;
                        }
                        return Ok(ExecFlow::Break(label));
                    }
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                }
                if !this.eval_expr(cond, env, event_param, event)?.truthy() {
                    break;
                }
            }
            Ok(ExecFlow::Continue)
        })
    }

    pub(crate) fn try_execute_loop_stmt(
        &mut self,
        stmt: &Stmt,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<Option<ExecFlow>> {
        match stmt {
            Stmt::ClassListForEach {
                target,
                optional,
                item_var,
                index_var,
                body,
            } => Ok(Some(self.execute_class_list_for_each_stmt(
                target,
                *optional,
                item_var,
                index_var,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::ForEach {
                target,
                selector,
                item_var,
                index_var,
                body,
            } => Ok(Some(self.execute_query_selector_for_each_stmt(
                target,
                selector,
                item_var,
                index_var,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::ArrayForEach { target, callback } => {
                let target_value = env
                    .get(target)
                    .cloned()
                    .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {target}")))?;
                self.execute_array_like_foreach_in_env(target_value, callback, env, event, target)?;
                Ok(Some(ExecFlow::Continue))
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
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::For {
                init,
                cond,
                post,
                body,
            } => Ok(Some(self.execute_for_stmt(
                init,
                cond,
                post,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::ForIn {
                item_var,
                iterable,
                body,
            } => Ok(Some(self.execute_for_in_stmt(
                item_var,
                iterable,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::ForOf {
                item_var,
                iterable,
                body,
            } => Ok(Some(self.execute_for_of_stmt(
                item_var,
                iterable,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::ForAwaitOf {
                item_var,
                iterable,
                body,
            } => Ok(Some(self.execute_for_await_of_stmt(
                item_var,
                iterable,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::While { cond, body } => Ok(Some(self.execute_while_stmt(
                cond,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::DoWhile { cond, body } => Ok(Some(self.execute_do_while_stmt(
                cond,
                body,
                env,
                event_param,
                event,
            )?)),
            _ => Ok(None),
        }
    }
}
