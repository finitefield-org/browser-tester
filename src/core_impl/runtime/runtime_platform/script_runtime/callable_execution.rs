use super::*;

impl Harness {
    fn make_function_value_with_kind(
        &mut self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        is_async: bool,
        is_generator: bool,
        is_class_constructor: bool,
        class_super_constructor: Option<Value>,
        class_super_prototype: Option<Value>,
    ) -> Value {
        let local_bindings = Self::collect_function_scope_bindings(&handler);
        let scope_depth = Self::env_scope_depth(env);
        let captured_pending_function_decls = self.script_runtime.pending_function_decls.clone();
        let captured_env = if global_scope {
            Rc::new(RefCell::new(self.script_runtime.env.share()))
        } else {
            let captured_env = self.ensure_listener_capture_env();
            *captured_env.borrow_mut() = ScriptEnv::from_snapshot(env);
            captured_env
        };
        let captured_env_snapshot = captured_env.borrow();
        let mut captured_global_names = HashSet::new();
        for (name, value) in captured_env_snapshot.iter() {
            if Self::is_internal_env_key(name) || name == INTERNAL_RETURN_SLOT {
                continue;
            }
            if scope_depth == 0 {
                captured_global_names.insert(name.clone());
                continue;
            }
            let Some(global_value) = self.script_runtime.env.get(name) else {
                continue;
            };
            if global_scope || self.strict_equal(global_value, value) {
                captured_global_names.insert(name.clone());
            }
        }
        drop(captured_env_snapshot);
        Value::Function(Rc::new(FunctionValue {
            handler,
            captured_env,
            captured_pending_function_decls,
            captured_global_names,
            local_bindings,
            prototype_object: Rc::new(RefCell::new(ObjectValue::default())),
            global_scope,
            is_async,
            is_generator,
            is_class_constructor,
            class_super_constructor,
            class_super_prototype,
        }))
    }

    pub(crate) fn make_function_value(
        &mut self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        is_async: bool,
        is_generator: bool,
    ) -> Value {
        self.make_function_value_with_kind(
            handler,
            env,
            global_scope,
            is_async,
            is_generator,
            false,
            None,
            None,
        )
    }

    pub(crate) fn make_function_value_with_super(
        &mut self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        is_async: bool,
        is_generator: bool,
        class_super_constructor: Option<Value>,
        class_super_prototype: Option<Value>,
    ) -> Value {
        self.make_function_value_with_kind(
            handler,
            env,
            global_scope,
            is_async,
            is_generator,
            false,
            class_super_constructor,
            class_super_prototype,
        )
    }

    pub(crate) fn make_class_constructor_value_with_super(
        &mut self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        class_super_constructor: Option<Value>,
        class_super_prototype: Option<Value>,
    ) -> Value {
        self.make_function_value_with_kind(
            handler,
            env,
            global_scope,
            false,
            false,
            true,
            class_super_constructor,
            class_super_prototype,
        )
    }

    pub(crate) fn is_callable_value(&self, value: &Value) -> bool {
        matches!(
            value,
            Value::Function(_) | Value::PromiseCapability(_) | Value::StringConstructor
        ) || Self::callable_kind_from_value(value).is_some()
    }

    pub(crate) fn execute_callable_value(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
    ) -> Result<Value> {
        self.execute_callable_value_with_env(callable, args, event, None)
    }

    pub(crate) fn execute_constructor_value_with_env(
        &mut self,
        constructor: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        self.execute_constructor_value_with_this_and_env(constructor, args, event, caller_env, None)
    }

    pub(crate) fn execute_constructor_value_with_this_and_env(
        &mut self,
        constructor: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
        this_arg: Option<Value>,
    ) -> Result<Value> {
        match constructor {
            Value::Function(function) => {
                let instance = if let Some(instance) = this_arg {
                    if Self::is_primitive_value(&instance) {
                        return Err(Error::ScriptRuntime(
                            "constructor this value must be an object".into(),
                        ));
                    }
                    instance
                } else {
                    Self::new_object_value(vec![(
                        INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                        Value::Object(function.prototype_object.clone()),
                    )])
                };
                let result = self.execute_function_call(
                    function.as_ref(),
                    args,
                    event,
                    caller_env,
                    Some(instance.clone()),
                )?;
                if Self::is_primitive_value(&result) {
                    Ok(instance)
                } else {
                    Ok(result)
                }
            }
            other => {
                if self.is_callable_value(other) {
                    self.execute_callable_value_with_env(other, args, event, caller_env)
                } else {
                    Err(Error::ScriptRuntime("value is not a constructor".into()))
                }
            }
        }
    }

    pub(crate) fn execute_callable_value_with_env(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        self.execute_callable_value_with_this_and_env(callable, args, event, caller_env, None)
    }

    pub(crate) fn execute_callable_value_with_this_and_env(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
        this_arg: Option<Value>,
    ) -> Result<Value> {
        match callable {
            Value::Function(function) => {
                if function.is_class_constructor && this_arg.is_none() {
                    return Err(Error::ScriptRuntime(
                        "Class constructor cannot be invoked without 'new'".into(),
                    ));
                }
                self.execute_function_call(function.as_ref(), args, event, caller_env, this_arg)
            }
            Value::PromiseCapability(capability) => {
                self.invoke_promise_capability(capability, args)
            }
            Value::StringConstructor => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Ok(Value::String(value.as_string()))
            }
            Value::Object(_) => {
                let Some(kind) = Self::callable_kind_from_value(callable) else {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                };
                match kind {
                    "intl_collator_compare" => {
                        let (locale, case_first, sensitivity) =
                            self.resolve_intl_collator_options(callable)?;
                        let left = args
                            .first()
                            .cloned()
                            .unwrap_or(Value::Undefined)
                            .as_string();
                        let right = args.get(1).cloned().unwrap_or(Value::Undefined).as_string();
                        Ok(Value::Number(Self::intl_collator_compare_strings(
                            &left,
                            &right,
                            &locale,
                            &case_first,
                            &sensitivity,
                        )))
                    }
                    "intl_date_time_format" => {
                        let (locale, options) = self.resolve_intl_date_time_options(callable)?;
                        let timestamp_ms = args
                            .first()
                            .map(|value| self.coerce_date_timestamp_ms(value))
                            .unwrap_or(self.scheduler.now_ms);
                        Ok(Value::String(self.intl_format_date_time(
                            timestamp_ms,
                            &locale,
                            &options,
                        )))
                    }
                    "intl_duration_format" => {
                        let (locale, options) = self.resolve_intl_duration_options(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(
                            self.intl_format_duration(&locale, &options, &value)?,
                        ))
                    }
                    "intl_list_format" => {
                        let (locale, options) = self.resolve_intl_list_options(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(
                            self.intl_format_list(&locale, &options, &value)?,
                        ))
                    }
                    "intl_number_format" => {
                        let (_, locale) = self.resolve_intl_formatter(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(Self::intl_format_number_for_locale(
                            Self::coerce_number_for_global(&value),
                            &locale,
                        )))
                    }
                    "intl_segmenter_segments_iterator" => {
                        let Value::Object(entries) = callable else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let entries = entries.borrow();
                        let segments = Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENTS_KEY)
                            .ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "Intl.Segmenter iterator has invalid internal state".into(),
                                )
                            })?;
                        Ok(self.new_intl_segmenter_iterator_value(segments))
                    }
                    "intl_segmenter_iterator_next" => {
                        let Value::Object(entries) = callable else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let mut entries = entries.borrow_mut();
                        let segments = Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENTS_KEY)
                            .ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "Intl.Segmenter iterator has invalid internal state".into(),
                                )
                            })?;
                        let Value::Array(values) = segments else {
                            return Err(Error::ScriptRuntime(
                                "Intl.Segmenter iterator has invalid internal state".into(),
                            ));
                        };
                        let len = values.borrow().len();
                        let index =
                            match Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENT_INDEX_KEY)
                            {
                                Some(Value::Number(value)) if value >= 0 => value as usize,
                                _ => 0,
                            };
                        if index >= len {
                            return Ok(Self::new_object_value(vec![
                                ("value".to_string(), Value::Undefined),
                                ("done".to_string(), Value::Bool(true)),
                            ]));
                        }
                        let value = values
                            .borrow()
                            .get(index)
                            .cloned()
                            .unwrap_or(Value::Undefined);
                        Self::object_set_entry(
                            &mut entries,
                            INTERNAL_INTL_SEGMENT_INDEX_KEY.to_string(),
                            Value::Number((index + 1) as i64),
                        );
                        Ok(Self::new_object_value(vec![
                            ("value".to_string(), value),
                            ("done".to_string(), Value::Bool(false)),
                        ]))
                    }
                    "readable_stream_async_iterator" => {
                        let Value::Object(entries) = callable else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let entries = entries.borrow();
                        let chunks = match Self::object_get_entry(
                            &entries,
                            INTERNAL_ASYNC_ITERATOR_VALUES_KEY,
                        ) {
                            Some(Value::Array(values)) => values.borrow().clone(),
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "ReadableStream async iterator has invalid internal state"
                                        .into(),
                                ));
                            }
                        };
                        Ok(self.new_async_iterator_value(chunks))
                    }
                    "iterator_self" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Iterator[Symbol.iterator] does not take arguments".into(),
                            ));
                        }
                        let iterator = self.iterator_target_from_callable(callable)?;
                        Ok(Value::Object(iterator))
                    }
                    "async_generator_result_value" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Self::new_async_iterator_result_object(value, false))
                    }
                    "async_generator_result_done" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Self::new_async_iterator_result_object(value, true))
                    }
                    "async_iterator_next" => {
                        let iterator = self.async_iterator_target_from_callable(callable)?;
                        let is_async_generator = {
                            let entries = iterator.borrow();
                            Self::is_async_generator_object(&entries)
                        };
                        if !is_async_generator && !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "AsyncIterator.next does not take arguments".into(),
                            ));
                        }
                        let result = if let Some(value) =
                            self.async_iterator_next_value_from_object(&iterator)?
                        {
                            if is_async_generator {
                                return self
                                    .resolve_async_generator_iterator_result_promise(value, false);
                            }
                            Self::new_async_iterator_result_object(value, false)
                        } else {
                            Self::new_async_iterator_result_object(Value::Undefined, true)
                        };
                        let promise = self.new_pending_promise();
                        self.promise_resolve(&promise, result)?;
                        Ok(Value::Promise(promise))
                    }
                    "async_iterator_return" => {
                        let iterator = self.async_iterator_target_from_callable(callable)?;
                        let is_async_generator = {
                            let entries = iterator.borrow();
                            Self::is_async_generator_object(&entries)
                        };
                        if !is_async_generator {
                            return Err(Error::ScriptRuntime(
                                "AsyncIterator.return is not a function".into(),
                            ));
                        }
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        self.close_async_iterator_object(&iterator)?;
                        self.resolve_async_generator_iterator_result_promise(value, true)
                    }
                    "async_iterator_throw" => {
                        let iterator = self.async_iterator_target_from_callable(callable)?;
                        let is_async_generator = {
                            let entries = iterator.borrow();
                            Self::is_async_generator_object(&entries)
                        };
                        if !is_async_generator {
                            return Err(Error::ScriptRuntime(
                                "AsyncIterator.throw is not a function".into(),
                            ));
                        }
                        let reason = args.first().cloned().unwrap_or(Value::Undefined);
                        self.close_async_iterator_object(&iterator)?;
                        let promise = self.new_pending_promise();
                        self.promise_reject(&promise, reason);
                        Ok(Value::Promise(promise))
                    }
                    "async_iterator_self" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "AsyncIterator[Symbol.asyncIterator] does not take arguments"
                                    .into(),
                            ));
                        }
                        let iterator = self.async_iterator_target_from_callable(callable)?;
                        Ok(Value::Object(iterator))
                    }
                    "async_iterator_async_dispose" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "AsyncIterator[Symbol.asyncDispose] does not take arguments".into(),
                            ));
                        }
                        let iterator = self.async_iterator_target_from_callable(callable)?;
                        let return_value = {
                            let entries = iterator.borrow();
                            Self::object_get_entry(&entries, "return")
                        };
                        let dispose_result = if let Some(return_method) = return_value {
                            if !self.is_callable_value(&return_method) {
                                return Err(Error::ScriptRuntime(
                                    "AsyncIterator.return is not a function".into(),
                                ));
                            }
                            self.execute_callable_value(&return_method, &[], event)?
                        } else {
                            Value::Undefined
                        };
                        let promise = self.new_pending_promise();
                        self.promise_resolve(&promise, dispose_result)?;
                        Ok(Value::Promise(promise))
                    }
                    "async_generator_function_constructor" => {
                        self.build_async_generator_function_from_constructor_values(args)
                    }
                    "generator_function_constructor" => {
                        self.build_generator_function_from_constructor_values(args)
                    }
                    "boolean_constructor" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::Bool(value.truthy()))
                    }
                    _ => Err(Error::ScriptRuntime("callback is not a function".into())),
                }
            }
            _ => Err(Error::ScriptRuntime("callback is not a function".into())),
        }
    }

    pub(crate) fn invoke_promise_capability(
        &mut self,
        capability: &PromiseCapabilityFunction,
        args: &[Value],
    ) -> Result<Value> {
        let mut already_called = capability.already_called.borrow_mut();
        if *already_called {
            return Ok(Value::Undefined);
        }
        *already_called = true;
        drop(already_called);

        let value = args.first().cloned().unwrap_or(Value::Undefined);
        if capability.reject {
            self.promise_reject(&capability.promise, value);
            Ok(Value::Undefined)
        } else {
            self.promise_resolve(&capability.promise, value)?;
            Ok(Value::Undefined)
        }
    }

    pub(crate) fn is_primitive_value(value: &Value) -> bool {
        matches!(
            value,
            Value::String(_)
                | Value::Bool(_)
                | Value::Number(_)
                | Value::Float(_)
                | Value::BigInt(_)
                | Value::Null
                | Value::Undefined
                | Value::Symbol(_)
        )
    }

    pub(crate) fn bind_handler_params(
        &mut self,
        handler: &ScriptHandler,
        args: &[Value],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        for (index, param) in handler.params.iter().enumerate() {
            if param.is_rest {
                let rest = if index < args.len() {
                    args[index..].to_vec()
                } else {
                    Vec::new()
                };
                env.insert(param.name.clone(), Self::new_array_value(rest));
                self.set_const_binding(env, &param.name, false);
                continue;
            }

            let provided = args.get(index).cloned().unwrap_or(Value::Undefined);
            let value = if matches!(provided, Value::Undefined) {
                if let Some(default_expr) = &param.default {
                    self.eval_expr(default_expr, env, event_param, event)?
                } else {
                    Value::Undefined
                }
            } else {
                provided
            };
            env.insert(param.name.clone(), value);
            self.set_const_binding(env, &param.name, false);
        }
        Ok(())
    }

    pub(crate) fn execute_function_call(
        &mut self,
        function: &FunctionValue,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
        this_arg: Option<Value>,
    ) -> Result<Value> {
        let run = |this: &mut Self,
                   caller_env: Option<&HashMap<String, Value>>,
                   this_arg: Option<Value>|
         -> Result<Value> {
            let pending_scope_start =
                this.push_pending_function_decl_scopes(&function.captured_pending_function_decls);

            let result = this.with_isolated_loop_control_scope(|this| {
                (|| -> Result<Value> {
                    let captured_env_before_call = if function.global_scope {
                        HashMap::new()
                    } else {
                        function.captured_env.borrow().to_map()
                    };
                    let mut call_env = if function.global_scope {
                        this.script_runtime.env.to_map()
                    } else {
                        captured_env_before_call.clone()
                    };
                    call_env.remove(INTERNAL_RETURN_SLOT);
                    let scope_depth = Self::env_scope_depth(&call_env);
                    call_env.insert(
                        INTERNAL_SCOPE_DEPTH_KEY.to_string(),
                        Value::Number(scope_depth.saturating_add(1)),
                    );
                    call_env.insert("this".to_string(), this_arg.unwrap_or(Value::Undefined));
                    this.set_const_binding(&mut call_env, "this", false);
                    if let Some(super_constructor) = function.class_super_constructor.clone() {
                        call_env.insert(
                            INTERNAL_CLASS_SUPER_CONSTRUCTOR_KEY.to_string(),
                            super_constructor,
                        );
                    }
                    if let Some(super_prototype) = function.class_super_prototype.clone() {
                        call_env.insert(INTERNAL_CLASS_SUPER_PROTOTYPE_KEY.to_string(), super_prototype);
                    }
                    let mut global_sync_keys = HashSet::new();
                    let caller_view = caller_env;
                    for name in &function.captured_global_names {
                        if Self::is_internal_env_key(name) || function.local_bindings.contains(name)
                        {
                            continue;
                        }
                        global_sync_keys.insert(name.clone());
                        if let Some(global_value) = this.script_runtime.env.get(name).cloned() {
                            call_env.insert(name.clone(), global_value);
                        } else if let Some(value) =
                            caller_view.and_then(|env| env.get(name)).cloned()
                        {
                            call_env.insert(name.clone(), value);
                        }
                    }
                    for (name, global_value) in this.script_runtime.env.iter() {
                        if Self::is_internal_env_key(name)
                            || function.local_bindings.contains(name)
                            || call_env.contains_key(name)
                        {
                            continue;
                        }
                        call_env.insert(name.clone(), global_value.clone());
                        global_sync_keys.insert(name.clone());
                    }
                    if !global_sync_keys.is_empty() {
                        let mut sync_names = global_sync_keys.iter().cloned().collect::<Vec<_>>();
                        sync_names.sort();
                        call_env.insert(
                            INTERNAL_GLOBAL_SYNC_NAMES_KEY.to_string(),
                            Self::new_array_value(
                                sync_names.into_iter().map(Value::String).collect(),
                            ),
                        );
                    }
                    let mut global_values_before_call = HashMap::new();
                    for name in &global_sync_keys {
                        if let Some(value) = this.script_runtime.env.get(name).cloned() {
                            global_values_before_call.insert(name.clone(), value);
                        }
                    }
                    let mut call_event = event.clone();
                    let event_param = None;
                    this.bind_handler_params(
                        &function.handler,
                        args,
                        &mut call_env,
                        &event_param,
                        &call_event,
                    )?;
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
                    let flow = this.execute_stmts(
                        &function.handler.stmts,
                        &event_param,
                        &mut call_event,
                        &mut call_env,
                    );
                    if yield_collector.is_some() {
                        let _ = this.script_runtime.generator_yield_stack.pop();
                    }
                    let flow = match flow {
                        Ok(flow) => flow,
                        Err(Error::ScriptRuntime(msg))
                            if function.is_generator
                                && msg == INTERNAL_GENERATOR_YIELD_LIMIT_REACHED =>
                        {
                            ExecFlow::Continue
                        }
                        Err(err) => return Err(err),
                    };
                    let generator_yields = yield_collector
                        .as_ref()
                        .map(|values| values.borrow().clone())
                        .unwrap_or_default();
                    for name in &global_sync_keys {
                        if Self::is_internal_env_key(name) || function.local_bindings.contains(name)
                        {
                            continue;
                        }
                        let before = global_values_before_call.get(name);
                        let global_after = this.script_runtime.env.get(name).cloned();
                        let call_after = call_env.get(name).cloned();
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
                        }
                    }
                    if !function.global_scope {
                        let mut captured_env = function.captured_env.borrow_mut();
                        for name in captured_env_before_call.keys() {
                            if Self::is_internal_env_key(name)
                                || function.local_bindings.contains(name.as_str())
                            {
                                continue;
                            }
                            let before = captured_env_before_call.get(name);
                            let after = call_env.get(name);
                            let changed = match (before, after) {
                                (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                                (None, Some(_)) => true,
                                (Some(_), None) => true,
                                (None, None) => false,
                            };
                            if !changed {
                                continue;
                            }
                            if let Some(next) = after.cloned() {
                                captured_env.insert(name.clone(), next.clone());
                                this.queue_listener_capture_env_update(name.clone(), Some(next));
                            } else {
                                captured_env.remove(name);
                                this.queue_listener_capture_env_update(name.clone(), None);
                            }
                        }
                    }
                    if function.is_generator {
                        if function.is_async {
                            return Ok(this.new_async_generator_value(generator_yields));
                        }
                        return Ok(this.new_generator_value(generator_yields));
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

            this.restore_pending_function_decl_scopes(pending_scope_start);
            result
        };

        if function.is_async && !function.is_generator {
            let promise = self.new_pending_promise();
            match run(self, caller_env, this_arg.clone()) {
                Ok(value) => {
                    if let Err(err) = self.promise_resolve(&promise, value) {
                        self.promise_reject(&promise, Self::promise_error_reason(err));
                    }
                }
                Err(err) => self.promise_reject(&promise, Self::promise_error_reason(err)),
            }
            Ok(Value::Promise(promise))
        } else {
            run(self, caller_env, this_arg)
        }
    }
}
