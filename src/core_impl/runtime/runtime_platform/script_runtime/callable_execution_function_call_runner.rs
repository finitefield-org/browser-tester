use super::*;

impl Harness {
    fn run_function_call_body(
        &mut self,
        function: Rc<FunctionValue>,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
        this_arg: Option<Value>,
        new_target: Option<Value>,
        sync_event_to: Option<&mut EventState>,
    ) -> Result<Value> {
        let pending_scope_start =
            self.push_pending_function_decl_scopes(&function.captured_pending_function_decls);

        let private_bindings = self
            .script_runtime
            .function_private_bindings
            .get(&function.function_id)
            .cloned();
        if let Some(bindings) = private_bindings.clone() {
            self.script_runtime.private_binding_stack.push(bindings);
        }

        let is_constructor_call = function.is_class_constructor;
        if is_constructor_call {
            self.script_runtime
                .constructor_call_stack
                .push(function.function_id);
            let initialized = function.class_super_constructor.is_none();
            self.script_runtime
                .constructor_instance_initialized_stack
                .push(initialized);
        }

        let listener_capture_scope_start = self.script_runtime.listener_capture_env_stack.len();
        let captured_env_seed =
            (!function.global_scope).then(|| Self::function_capture_snapshot(&function));
        let shared_env_frame_start = (!function.global_scope).then(|| {
            self.push_shared_listener_capture_env_frame_with_names(
                function.captured_env.clone(),
                false,
                Some(function.captured_names.clone()),
            )
        });

        let result = self.with_isolated_loop_control_scope(|this| {
            (|| -> Result<Value> {
                let captured_env_before_call = captured_env_seed.clone();
                let mut call_env = if function.global_scope {
                    this.script_runtime.env.to_map()
                } else {
                    captured_env_before_call
                        .as_ref()
                        .cloned()
                        .unwrap_or_default()
                };
                Self::isolate_execution_const_bindings(&mut call_env);
                for name in &function.local_bindings {
                    call_env.remove(name);
                }
                call_env.remove(INTERNAL_RETURN_SLOT);
                call_env.remove(INTERNAL_LOCAL_BINDINGS_KEY);
                let mut global_sync_keys = HashSet::new();
                let caller_view = caller_env;
                let caller_scope_start =
                    caller_view.map(Self::pending_listener_capture_scope_start);
                for name in &function.captured_names {
                    if Self::is_internal_env_key(&name)
                        || function.local_bindings.contains(name.as_str())
                        || matches!(name.as_str(), "this" | "arguments")
                        || call_env.contains_key(name)
                    {
                        continue;
                    }
                    if let Some(caller_scope_start) = caller_scope_start {
                        if let Some(pending) = this
                            .resolve_listener_capture_pending_value_from(caller_scope_start, name)
                        {
                            if let Some(value) = pending {
                                call_env.insert(name.clone(), value);
                            } else {
                                call_env.remove(name);
                            }
                            continue;
                        }
                    }
                    if let Some(value) = caller_view.and_then(|env| env.get(name)).cloned() {
                        call_env.insert(name.clone(), value);
                        continue;
                    }
                    if let Some(value) = this.resolve_runtime_global_identifier(name) {
                        call_env.insert(name.clone(), value);
                        global_sync_keys.insert(name.clone());
                    }
                }
                if let Some(caller_view) = caller_view {
                    let lexical_names = Self::env_top_level_lexical_binding_names(&call_env);
                    for name in lexical_names {
                        if Self::is_internal_env_key(&name)
                            || function.local_bindings.contains(name.as_str())
                            || function.captured_names.contains(&name)
                            || matches!(name.as_str(), "this" | "arguments")
                            || Self::env_has_local_binding(caller_view, &name)
                        {
                            continue;
                        }
                        if let Some(value) = caller_view.get(&name).cloned() {
                            call_env.insert(name, value);
                            continue;
                        }
                        if let Some(caller_scope_start) = caller_scope_start {
                            if let Some(pending) = this.resolve_listener_capture_pending_value_from(
                                caller_scope_start,
                                &name,
                            ) {
                                if let Some(value) = pending {
                                    call_env.insert(name, value);
                                } else {
                                    call_env.remove(&name);
                                }
                                continue;
                            }
                        }
                    }
                }
                let scope_depth = Self::env_scope_depth(&call_env);
                call_env.insert(
                    INTERNAL_SCOPE_DEPTH_KEY.to_string(),
                    Value::Number(scope_depth.saturating_add(1)),
                );
                call_env.insert(
                    INTERNAL_PENDING_SCOPE_START_KEY.to_string(),
                    Value::Number(listener_capture_scope_start as i64),
                );
                if function.is_arrow {
                    if !call_env.contains_key("this") {
                        call_env.insert("this".to_string(), Value::Undefined);
                        this.set_const_binding(&mut call_env, "this", false);
                    }
                } else {
                    call_env.insert("this".to_string(), this_arg.unwrap_or(Value::Undefined));
                    this.set_const_binding(&mut call_env, "this", false);
                    call_env.insert(
                        INTERNAL_NEW_TARGET_KEY.to_string(),
                        new_target.unwrap_or(Value::Undefined),
                    );
                    let arguments_value = Self::new_array_value(args.to_vec());
                    if let Value::Array(arguments) = &arguments_value {
                        Self::object_set_entry(
                            &mut arguments.borrow_mut().properties,
                            "callee".to_string(),
                            Value::Function(function.clone()),
                        );
                    }
                    call_env.insert("arguments".to_string(), arguments_value);
                    this.set_const_binding(&mut call_env, "arguments", false);
                    if Self::has_simple_parameter_list(&function.handler) {
                        let mut bindings = Vec::with_capacity(args.len());
                        for index in 0..args.len() {
                            let binding = function
                                .handler
                                .params
                                .get(index)
                                .map(|param| Value::String(param.name.clone()))
                                .unwrap_or(Value::Undefined);
                            bindings.push(binding);
                        }
                        call_env.insert(
                            INTERNAL_ARGUMENTS_PARAM_BINDINGS_KEY.to_string(),
                            Self::new_array_value(bindings),
                        );
                    }
                }
                if let Some(expression_name) = function.expression_name.as_ref() {
                    call_env.insert(expression_name.clone(), Value::Function(function.clone()));
                    this.set_const_binding(&mut call_env, expression_name, true);
                }
                if !function.local_bindings.is_empty() {
                    let mut local_bindings =
                        function.local_bindings.iter().cloned().collect::<Vec<_>>();
                    local_bindings.sort();
                    call_env.insert(
                        INTERNAL_LOCAL_BINDINGS_KEY.to_string(),
                        Self::new_array_value(
                            local_bindings.into_iter().map(Value::String).collect(),
                        ),
                    );
                }
                if let Some(super_constructor) = function.class_super_constructor.clone() {
                    call_env.insert(
                        INTERNAL_CLASS_SUPER_CONSTRUCTOR_KEY.to_string(),
                        super_constructor,
                    );
                }
                if let Some(super_prototype) = function.class_super_prototype.clone() {
                    call_env.insert(
                        INTERNAL_CLASS_SUPER_PROTOTYPE_KEY.to_string(),
                        super_prototype,
                    );
                } else if function.is_method {
                    let inferred_super = match call_env.get("this").cloned() {
                        Some(Value::Object(object)) => {
                            Self::object_get_entry(&object.borrow(), INTERNAL_OBJECT_PROTOTYPE_KEY)
                        }
                        Some(Value::Function(function_value)) => {
                            function_value.class_super_constructor.clone()
                        }
                        _ => None,
                    };
                    if let Some(super_prototype) = inferred_super {
                        call_env.insert(
                            INTERNAL_CLASS_SUPER_PROTOTYPE_KEY.to_string(),
                            super_prototype,
                        );
                    }
                }
                for name in &function.captured_global_names {
                    if Self::is_internal_env_key(&name)
                        || function.local_bindings.contains(name)
                        || name == "this"
                        || name == "arguments"
                    {
                        continue;
                    }
                    global_sync_keys.insert(name.clone());
                    if let Some(global_value) = this.resolve_runtime_global_identifier(name) {
                        call_env.insert(name.clone(), global_value);
                    } else if !call_env.contains_key(name) {
                        if let Some(value) = caller_view.and_then(|env| env.get(name)).cloned() {
                            call_env.insert(name.clone(), value);
                        }
                    }
                }
                if !global_sync_keys.is_empty() {
                    let mut sync_names = global_sync_keys.iter().cloned().collect::<Vec<_>>();
                    sync_names.sort();
                    call_env.insert(
                        INTERNAL_GLOBAL_SYNC_NAMES_KEY.to_string(),
                        Self::new_array_value(sync_names.into_iter().map(Value::String).collect()),
                    );
                }
                let mut global_values_before_call = HashMap::new();
                for name in &global_sync_keys {
                    if let Some(value) = this.script_runtime.env.get(name).cloned() {
                        global_values_before_call.insert(name.clone(), value);
                    }
                }
                let mut call_event = event.clone();
                let event_param = sync_event_to
                    .as_ref()
                    .and_then(|_| function.handler.first_event_param())
                    .map(str::to_string);
                this.script_runtime
                    .listener_capture_env_stack
                    .push(ListenerCaptureFrame {
                        ..ListenerCaptureFrame::default()
                    });
                let bind_result = (|| -> Result<()> {
                    this.project_pending_listener_capture_env_updates(&mut call_env);
                    this.bind_handler_params(
                        &function.handler,
                        args,
                        &mut call_env,
                        &event_param,
                        &call_event,
                    )?;
                    Ok(())
                })();
                this.script_runtime.listener_capture_env_stack.pop();
                bind_result?;
                if function.is_class_constructor && function.class_super_constructor.is_none() {
                    this.apply_constructor_instance_initializers_by_id(
                        function.function_id,
                        &call_env,
                        &event_param,
                        &call_event,
                    )?;
                }
                let param_names = function
                    .handler
                    .params
                    .iter()
                    .map(|param| param.name.clone())
                    .collect::<HashSet<_>>();
                this.ensure_no_direct_let_redeclarations(&function.handler.stmts, &param_names)?;
                let yield_collector = if function.is_generator {
                    Some(Rc::new(RefCell::new(Vec::new())))
                } else {
                    None
                };
                if let Some(yields) = &yield_collector {
                    this.script_runtime
                        .generator_yield_stack
                        .push(yields.clone());
                }
                let mut non_tdz_shadowed =
                    Self::collect_var_declared_names(&function.handler.stmts);
                non_tdz_shadowed.extend(
                    function
                        .handler
                        .params
                        .iter()
                        .map(|param| param.name.clone()),
                );
                non_tdz_shadowed
                    .extend(Self::collect_function_decls(&function.handler.stmts).into_keys());
                if let Some(expression_name) = function.expression_name.as_ref() {
                    non_tdz_shadowed.insert(expression_name.clone());
                }
                if let Some(caller_view) = caller_view {
                    non_tdz_shadowed.extend(
                        Self::env_local_or_lexical_binding_names(caller_view)
                            .into_iter()
                            .filter(|name| {
                                !function.captured_names.contains(name)
                                    && !matches!(name.as_str(), "this" | "arguments")
                            }),
                    );
                }

                let pushed_non_tdz_scope = !non_tdz_shadowed.is_empty();
                if pushed_non_tdz_scope {
                    this.script_runtime.tdz_scope_stack.push(TdzScopeFrame {
                        declared: non_tdz_shadowed,
                        pending: HashSet::new(),
                    });
                }
                let current_scope_pending_updates_before = this
                    .listener_capture_pending_updates_snapshot_from(
                        Self::pending_listener_capture_scope_start(&call_env),
                    );
                let mut pending_async_suspend = None;
                let flow_result = if function.is_async && !function.is_generator {
                    if let Some((await_index, await_expr, resume_kind)) =
                        Self::first_suspendable_top_level_await(&function.handler.stmts)
                    {
                        let prefix_flow = this.execute_stmts_with_pending_scope(
                            &function.handler.stmts[..await_index],
                            &event_param,
                            &mut call_event,
                            &mut call_env,
                            false,
                        )?;
                        Ok(match prefix_flow {
                            ExecFlow::Continue => match this.eval_top_level_await_expr(
                                &await_expr,
                                &mut call_env,
                                &event_param,
                                &call_event,
                            )? {
                                TopLevelAwaitOutcome::Resolved(awaited_value) => {
                                    match &resume_kind {
                                        TopLevelAwaitResumeKind::Ignore => {}
                                        TopLevelAwaitResumeKind::Declare { name, kind } => {
                                            call_env.insert(name.clone(), awaited_value);
                                            this.set_const_binding(
                                                &mut call_env,
                                                name,
                                                matches!(kind, VarDeclKind::Const),
                                            );
                                        }
                                        TopLevelAwaitResumeKind::Assign { name } => {
                                            call_env.insert(name.clone(), awaited_value);
                                        }
                                        TopLevelAwaitResumeKind::Return => {
                                            call_env.insert(
                                                INTERNAL_RETURN_SLOT.to_string(),
                                                awaited_value,
                                            );
                                        }
                                    };
                                    if matches!(resume_kind, TopLevelAwaitResumeKind::Return) {
                                        ExecFlow::Return
                                    } else {
                                        this.execute_stmts_with_pending_scope(
                                            &function.handler.stmts[await_index + 1..],
                                            &event_param,
                                            &mut call_event,
                                            &mut call_env,
                                            false,
                                        )?
                                    }
                                }
                                TopLevelAwaitOutcome::Pending(awaited_promise) => {
                                    let continuation_handler =
                                        Self::build_top_level_await_continuation_handler(
                                            &resume_kind,
                                            &function.handler.stmts[await_index + 1..],
                                        );
                                    let continuation = this.make_function_value_with_kind(
                                        continuation_handler,
                                        &call_env,
                                        false,
                                        true,
                                        false,
                                        true,
                                        false,
                                        false,
                                        None,
                                        None,
                                    );
                                    pending_async_suspend = Some(PendingAsyncFunctionSuspend {
                                        awaited_promise,
                                        continuation,
                                    });
                                    ExecFlow::Continue
                                }
                            },
                            other => other,
                        })
                    } else {
                        this.execute_stmts_with_pending_scope(
                            &function.handler.stmts,
                            &event_param,
                            &mut call_event,
                            &mut call_env,
                            false,
                        )
                    }
                } else {
                    this.execute_stmts_with_pending_scope(
                        &function.handler.stmts,
                        &event_param,
                        &mut call_event,
                        &mut call_env,
                        false,
                    )
                };
                if pushed_non_tdz_scope {
                    this.script_runtime.tdz_scope_stack.pop();
                }
                if yield_collector.is_some() {
                    let _ = this.script_runtime.generator_yield_stack.pop();
                }
                let mut deferred_error = None;
                let flow = match flow_result {
                    Ok(flow) => flow,
                    Err(Error::ScriptRuntime(msg))
                        if function.is_generator
                            && msg == INTERNAL_GENERATOR_YIELD_LIMIT_REACHED =>
                    {
                        ExecFlow::Continue
                    }
                    Err(err) => {
                        deferred_error = Some(err);
                        ExecFlow::Continue
                    }
                };
                let current_scope_pending_updates_after = this
                    .listener_capture_pending_updates_snapshot_from(
                        Self::pending_listener_capture_scope_start(&call_env),
                    );
                this.apply_pending_listener_capture_env_updates(&mut call_env);
                let generator_yields = yield_collector
                    .as_ref()
                    .map(|values| values.borrow().clone())
                    .unwrap_or_default();
                let generator_return_value = if matches!(flow, ExecFlow::Return) {
                    call_env
                        .get(INTERNAL_RETURN_SLOT)
                        .cloned()
                        .unwrap_or(Value::Undefined)
                } else {
                    Value::Undefined
                };
                if let Some(event_state) = sync_event_to {
                    Self::sync_event_argument_back_to_state(
                        event_state,
                        &call_env,
                        event_param.as_deref(),
                    );
                }
                let caller_has_explicit_binding = |name: &str| {
                    caller_view.is_some_and(|env| Self::env_has_explicit_binding(env, name))
                };
                let effective_call_binding = |this: &Self, name: &str| {
                    let current_scope_pending = this.resolve_listener_capture_pending_value_from(
                        Self::pending_listener_capture_scope_start(&call_env),
                        name,
                    );
                    if let Some(pending) = current_scope_pending {
                        return pending;
                    }
                    if let Some(value) = call_env.get(name).cloned() {
                        return Some(value);
                    }
                    if let Some(pending) = this.resolve_listener_capture_pending_value(name) {
                        return pending;
                    }
                    None
                };
                for name in &global_sync_keys {
                    if Self::is_internal_env_key(name)
                        || function.local_bindings.contains(name)
                        || name == "this"
                        || name == "arguments"
                    {
                        continue;
                    }
                    let before = global_values_before_call.get(name);
                    let global_after = this.script_runtime.env.get(name).cloned();
                    let call_after = effective_call_binding(this, name);
                    let global_changed = match (before, global_after.as_ref()) {
                        (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                        (None, Some(_)) => true,
                        (Some(_), None) => true,
                        (None, None) => false,
                    };
                    let call_changed = match (before, call_after.as_ref()) {
                        (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                        (None, Some(_)) => true,
                        (Some(_), None) => true,
                        (None, None) => false,
                    };
                    if global_changed && !call_changed {
                        continue;
                    }
                    if let Some(next) = call_after {
                        this.script_runtime.env.insert(name.clone(), next);
                        if caller_has_explicit_binding(name) {
                            this.script_runtime.expression_env_overrides.remove(name);
                        } else if caller_view.is_some() {
                            this.script_runtime
                                .expression_env_overrides
                                .insert(name.clone(), Some(this.script_runtime.env[name].clone()));
                        }
                        if let Some(value) = this.script_runtime.env.get(name).cloned() {
                            this.sync_scheduled_task_captures_for_binding(name, &value);
                        }
                    }
                }
                let mut scheduled_capture_updates = Vec::new();
                if !function.global_scope {
                    let captured_env_after_call = Self::function_capture_snapshot(&function);
                    let mut captured_env = function.captured_env.borrow_mut();
                    let captured_env_before_call = captured_env_before_call
                        .as_ref()
                        .expect("non-global functions always snapshot their capture env");
                    for name in &function.captured_names {
                        if matches!(name.as_str(), "this" | "arguments") {
                            continue;
                        }
                        let before = captured_env_before_call.get(name);
                        let call_after_from_env = effective_call_binding(this, name);
                        let call_after_from_shared = captured_env_after_call.get(name).cloned();
                        let call_after = match (
                            before,
                            call_after_from_env.as_ref(),
                            call_after_from_shared.as_ref(),
                        ) {
                            (Some(prev), Some(env_next), Some(shared_next))
                                if this.strict_equal(prev, env_next)
                                    && !this.strict_equal(prev, shared_next) =>
                            {
                                Some(shared_next.clone())
                            }
                            (Some(_), None, Some(shared_next)) => Some(shared_next.clone()),
                            _ => call_after_from_env.or(call_after_from_shared),
                        };
                        let after = call_after.as_ref();
                        let changed = match (before, after) {
                            (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                            (None, Some(_)) => true,
                            (Some(_), None) => true,
                            (None, None) => false,
                        };
                        if !changed {
                            continue;
                        }
                        if let Some(next) = call_after {
                            captured_env.insert(name.clone(), next.clone());
                            if caller_has_explicit_binding(name) {
                                if let Some(parent_index) =
                                    shared_env_frame_start.and_then(|start| start.checked_sub(1))
                                {
                                    if let Some(parent_frame) = this
                                        .script_runtime
                                        .listener_capture_env_stack
                                        .get_mut(parent_index)
                                    {
                                        parent_frame
                                            .pending_env_updates
                                            .insert(name.clone(), Some(next.clone()));
                                    }
                                }
                            }
                            if caller_view.is_some() {
                                this.script_runtime
                                    .expression_env_overrides
                                    .insert(name.clone(), Some(next.clone()));
                            }
                            this.queue_listener_capture_env_update_for_shared_env(
                                &function.captured_env,
                                name.clone(),
                                Some(next.clone()),
                            );
                            scheduled_capture_updates.push((name.clone(), next));
                        } else {
                            captured_env.remove(name);
                            if caller_has_explicit_binding(name) {
                                if let Some(parent_index) =
                                    shared_env_frame_start.and_then(|start| start.checked_sub(1))
                                {
                                    if let Some(parent_frame) = this
                                        .script_runtime
                                        .listener_capture_env_stack
                                        .get_mut(parent_index)
                                    {
                                        parent_frame.pending_env_updates.insert(name.clone(), None);
                                    }
                                }
                            }
                            if caller_view.is_some() {
                                this.script_runtime
                                    .expression_env_overrides
                                    .insert(name.clone(), None);
                            }
                            this.queue_listener_capture_env_update_for_shared_env(
                                &function.captured_env,
                                name.clone(),
                                None,
                            );
                        }
                    }
                }
                let mut caller_visible_names = current_scope_pending_updates_before
                    .keys()
                    .chain(current_scope_pending_updates_after.keys())
                    .filter_map(|name| Self::event_sync_pending_marker_name(name))
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                caller_visible_names.sort();
                caller_visible_names.dedup();
                for name in caller_visible_names {
                    let marker_key = Self::event_sync_pending_marker_key(&name);
                    let pending_before = current_scope_pending_updates_before.get(&marker_key);
                    let pending_after = current_scope_pending_updates_after.get(&marker_key);
                    let pending_changed = match (pending_before, pending_after) {
                        (Some(Some(prev)), Some(Some(next))) => !this.strict_equal(prev, next),
                        (Some(None), Some(None)) | (None, None) => false,
                        (Some(_), Some(_)) | (Some(_), None) | (None, Some(_)) => true,
                    };
                    if !pending_changed
                        || Self::is_internal_env_key(&name)
                        || function.local_bindings.contains(&name)
                        || matches!(name.as_str(), "this" | "arguments")
                        || global_sync_keys.contains(&name)
                        || function.captured_names.contains(&name)
                        || !caller_has_explicit_binding(&name)
                    {
                        continue;
                    }
                    let before = caller_view.and_then(|env| env.get(&name));
                    let call_after = pending_after
                        .cloned()
                        .unwrap_or_else(|| effective_call_binding(this, &name));
                    let after = call_after.as_ref();
                    let changed = match (before, after) {
                        (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                        (None, Some(_)) => true,
                        (Some(_), None) => true,
                        (None, None) => false,
                    };
                    if !changed {
                        continue;
                    }
                    if let Some(parent_index) =
                        shared_env_frame_start.and_then(|start| start.checked_sub(1))
                    {
                        if let Some(parent_frame) = this
                            .script_runtime
                            .listener_capture_env_stack
                            .get_mut(parent_index)
                        {
                            parent_frame
                                .pending_env_updates
                                .insert(name.clone(), call_after.clone());
                        }
                    }
                    if caller_view.is_some() {
                        this.script_runtime
                            .expression_env_overrides
                            .insert(name.clone(), call_after.clone());
                    }
                    if let Some(value) = call_after {
                        this.sync_scheduled_task_captures_for_binding(&name, &value);
                    }
                }
                for (name, value) in scheduled_capture_updates {
                    this.sync_scheduled_task_captures_for_binding(&name, &value);
                }
                if let Some(err) = deferred_error {
                    return Err(err);
                }
                if let Some(suspend) = pending_async_suspend {
                    this.script_runtime.pending_async_function_suspend = Some(suspend);
                    return Err(Error::ScriptRuntime(
                        INTERNAL_ASYNC_FUNCTION_SUSPENDED.into(),
                    ));
                }
                if function.is_generator {
                    if function.is_async {
                        return Ok(this.new_async_generator_value(generator_yields));
                    }
                    return Ok(this.new_generator_value(generator_yields, generator_return_value));
                }
                match flow {
                    ExecFlow::Continue => Ok(Value::Undefined),
                    ExecFlow::Break(label) => Err(Self::break_flow_error(&label)),
                    ExecFlow::ContinueLoop(label) => Err(Self::continue_flow_error(&label)),
                    ExecFlow::Return => Ok(call_env
                        .remove(INTERNAL_RETURN_SLOT)
                        .unwrap_or(Value::Undefined)),
                }
            })()
        });
        if let Some(start) = shared_env_frame_start {
            self.discard_event_sync_pending_updates_from_frames(start);
            self.discard_pending_listener_updates_from_frames(start, &function.local_bindings);
            self.restore_listener_capture_env_stack(start);
        }

        if private_bindings.is_some() {
            self.script_runtime.private_binding_stack.pop();
        }
        if is_constructor_call {
            self.script_runtime.constructor_call_stack.pop();
            self.script_runtime
                .constructor_instance_initialized_stack
                .pop();
        }
        self.restore_pending_function_decl_scopes(pending_scope_start);
        result
    }

    pub(crate) fn execute_function_call(
        &mut self,
        function: Rc<FunctionValue>,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
        this_arg: Option<Value>,
        new_target: Option<Value>,
        sync_event_to: Option<&mut EventState>,
    ) -> Result<Value> {
        if function.is_async && !function.is_generator {
            let promise = self.new_pending_promise();
            match self.run_function_call_body(
                function.clone(),
                args,
                event,
                caller_env,
                this_arg.clone(),
                new_target.clone(),
                sync_event_to,
            ) {
                Ok(value) => {
                    if let Err(err) = self.promise_resolve(&promise, value) {
                        self.promise_reject(&promise, Self::promise_error_reason(err));
                    }
                }
                Err(Error::ScriptRuntime(msg)) if msg == INTERNAL_ASYNC_FUNCTION_SUSPENDED => {
                    if let Some(suspend) = self.script_runtime.pending_async_function_suspend.take()
                    {
                        self.promise_add_reaction(
                            &suspend.awaited_promise,
                            PromiseReactionKind::Then {
                                on_fulfilled: Some(suspend.continuation),
                                on_rejected: None,
                                result: promise.clone(),
                            },
                        );
                    } else {
                        self.promise_reject(
                            &promise,
                            Value::String("async function suspended without continuation".into()),
                        );
                    }
                }
                Err(err) => self.promise_reject(&promise, Self::promise_error_reason(err)),
            }
            Ok(Value::Promise(promise))
        } else {
            self.run_function_call_body(
                function,
                args,
                event,
                caller_env,
                this_arg,
                new_target,
                sync_event_to,
            )
        }
    }
}
