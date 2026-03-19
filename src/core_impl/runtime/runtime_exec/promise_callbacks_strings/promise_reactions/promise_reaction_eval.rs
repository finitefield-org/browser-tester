use super::*;

impl Harness {
    pub(crate) fn eval_promise_construct(
        &mut self,
        executor: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Promise constructor must be called with new".into(),
            ));
        }
        let Some(executor) = executor else {
            return Err(Error::ScriptRuntime(
                "Promise constructor requires exactly one executor".into(),
            ));
        };
        let executor = self.eval_expr(executor, env, event_param, event)?;
        if !self.is_callable_value(&executor) {
            return Err(Error::ScriptRuntime(
                "Promise constructor executor must be a function".into(),
            ));
        }

        let promise = self.new_pending_promise();
        let (resolve, reject) = self.new_promise_capability_functions(promise.clone());
        if let Err(err) = self.execute_callable_value(&executor, &[resolve, reject], event) {
            self.promise_reject(&promise, Self::promise_error_reason(err));
        }
        Ok(Value::Promise(promise))
    }

    pub(crate) fn eval_promise_static_method(
        &mut self,
        method: PromiseStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }
        self.eval_promise_static_method_from_values(method, &values, event)
    }

    pub(crate) fn eval_promise_static_method_from_values(
        &mut self,
        method: PromiseStaticMethod,
        args: &[Value],
        event: &EventState,
    ) -> Result<Value> {
        match method {
            PromiseStaticMethod::Resolve => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.resolve supports zero or one argument".into(),
                    ));
                }
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                if let Value::Promise(promise) = value {
                    return Ok(Value::Promise(promise));
                }
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, value)?;
                Ok(Value::Promise(promise))
            }
            PromiseStaticMethod::Reject => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.reject supports zero or one argument".into(),
                    ));
                }
                let reason = args.first().cloned().unwrap_or(Value::Undefined);
                let promise = self.new_pending_promise();
                self.promise_reject(&promise, reason);
                Ok(Value::Promise(promise))
            }
            PromiseStaticMethod::All => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.all requires exactly one argument".into(),
                    ));
                }
                let iterable = args[0].clone();
                self.eval_promise_all(iterable)
            }
            PromiseStaticMethod::AllSettled => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.allSettled requires exactly one argument".into(),
                    ));
                }
                let iterable = args[0].clone();
                self.eval_promise_all_settled(iterable)
            }
            PromiseStaticMethod::Any => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.any requires exactly one argument".into(),
                    ));
                }
                let iterable = args[0].clone();
                self.eval_promise_any(iterable)
            }
            PromiseStaticMethod::Race => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.race requires exactly one argument".into(),
                    ));
                }
                let iterable = args[0].clone();
                self.eval_promise_race(iterable)
            }
            PromiseStaticMethod::Try => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Promise.try requires at least one argument".into(),
                    ));
                }
                let callback = args[0].clone();
                let callback_args = args.get(1..).unwrap_or(&[]).to_vec();
                let promise = self.new_pending_promise();
                match self.execute_callable_value(&callback, &callback_args, event) {
                    Ok(value) => {
                        self.promise_resolve(&promise, value)?;
                    }
                    Err(err) => {
                        self.promise_reject(&promise, Self::promise_error_reason(err));
                    }
                }
                Ok(Value::Promise(promise))
            }
            PromiseStaticMethod::WithResolvers => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Promise.withResolvers does not take arguments".into(),
                    ));
                }
                let promise = self.new_pending_promise();
                let (resolve, reject) = self.new_promise_capability_functions(promise.clone());
                Ok(Self::new_object_value(vec![
                    ("promise".into(), Value::Promise(promise)),
                    ("resolve".into(), resolve),
                    ("reject".into(), reject),
                ]))
            }
        }
    }

    pub(crate) fn eval_promise_method(
        &mut self,
        target: &Expr,
        method: PromiseInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let target = self.eval_expr(target, env, event_param, event)?;
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }
        let member = match method {
            PromiseInstanceMethod::Then => "then",
            PromiseInstanceMethod::Catch => "catch",
            PromiseInstanceMethod::Finally => "finally",
        };
        let Value::Promise(promise) = target.clone() else {
            let callee = self.object_property_from_value(&target, member)?;
            return self
                .execute_callable_value_with_this_and_env(
                    &callee,
                    &values,
                    event,
                    Some(env),
                    Some(target),
                )
                .map_err(|err| match err {
                    Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                        Error::ScriptRuntime(format!("'{}' is not a function", member))
                    }
                    other => other,
                });
        };
        self.eval_promise_instance_method_from_values(&promise, method, &values)
    }

    pub(crate) fn eval_promise_instance_method_from_values(
        &mut self,
        promise: &Rc<RefCell<PromiseValue>>,
        method: PromiseInstanceMethod,
        args: &[Value],
    ) -> Result<Value> {
        match method {
            PromiseInstanceMethod::Then => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Promise.then supports up to two arguments".into(),
                    ));
                }
                let on_fulfilled = if let Some(value) = args.first() {
                    if self.is_callable_value(value) {
                        Some(value.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };
                let on_rejected = if args.len() >= 2 {
                    if self.is_callable_value(&args[1]) {
                        Some(args[1].clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Value::Promise(self.promise_then_internal(
                    promise,
                    on_fulfilled,
                    on_rejected,
                )))
            }
            PromiseInstanceMethod::Catch => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.catch supports at most one argument".into(),
                    ));
                }
                let on_rejected = if let Some(value) = args.first() {
                    if self.is_callable_value(value) {
                        Some(value.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Value::Promise(self.promise_then_internal(
                    promise,
                    None,
                    on_rejected,
                )))
            }
            PromiseInstanceMethod::Finally => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.finally supports at most one argument".into(),
                    ));
                }
                let callback = if let Some(value) = args.first() {
                    if self.is_callable_value(value) {
                        Some(value.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };
                let result = self.new_pending_promise();
                self.promise_add_reaction(
                    promise,
                    PromiseReactionKind::Finally {
                        callback,
                        result: result.clone(),
                    },
                );
                Ok(Value::Promise(result))
            }
        }
    }

    pub(crate) fn eval_promise_member_call_from_values(
        &mut self,
        promise: &Rc<RefCell<PromiseValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let method = match member {
            "then" => PromiseInstanceMethod::Then,
            "catch" => PromiseInstanceMethod::Catch,
            "finally" => PromiseInstanceMethod::Finally,
            _ => return Ok(None),
        };
        self.eval_promise_instance_method_from_values(promise, method, args)
            .map(Some)
    }

    pub(crate) fn eval_promise_all(&mut self, iterable: Value) -> Result<Value> {
        let values = self.array_like_values_from_value(&iterable)?;
        let result = self.new_pending_promise();
        if values.is_empty() {
            self.promise_fulfill(&result, Self::new_array_value(Vec::new()));
            return Ok(Value::Promise(result));
        }

        let state = Rc::new(RefCell::new(PromiseAllState {
            result: result.clone(),
            remaining: values.len(),
            values: vec![None; values.len()],
            settled: false,
        }));

        for (index, value) in values.into_iter().enumerate() {
            let promise = self.promise_resolve_value_as_promise(value)?;
            self.promise_add_reaction(
                &promise,
                PromiseReactionKind::All {
                    state: state.clone(),
                    index,
                },
            );
        }

        Ok(Value::Promise(result))
    }

    pub(crate) fn eval_promise_all_settled(&mut self, iterable: Value) -> Result<Value> {
        let values = self.array_like_values_from_value(&iterable)?;
        let result = self.new_pending_promise();
        if values.is_empty() {
            self.promise_fulfill(&result, Self::new_array_value(Vec::new()));
            return Ok(Value::Promise(result));
        }

        let state = Rc::new(RefCell::new(PromiseAllSettledState {
            result: result.clone(),
            remaining: values.len(),
            values: vec![None; values.len()],
        }));

        for (index, value) in values.into_iter().enumerate() {
            let promise = self.promise_resolve_value_as_promise(value)?;
            self.promise_add_reaction(
                &promise,
                PromiseReactionKind::AllSettled {
                    state: state.clone(),
                    index,
                },
            );
        }

        Ok(Value::Promise(result))
    }

    pub(crate) fn eval_promise_any(&mut self, iterable: Value) -> Result<Value> {
        let values = self.array_like_values_from_value(&iterable)?;
        let result = self.new_pending_promise();
        if values.is_empty() {
            self.promise_reject(&result, Self::new_aggregate_error_value(Vec::new()));
            return Ok(Value::Promise(result));
        }

        let state = Rc::new(RefCell::new(PromiseAnyState {
            result: result.clone(),
            remaining: values.len(),
            reasons: vec![None; values.len()],
            settled: false,
        }));

        for (index, value) in values.into_iter().enumerate() {
            let promise = self.promise_resolve_value_as_promise(value)?;
            self.promise_add_reaction(
                &promise,
                PromiseReactionKind::Any {
                    state: state.clone(),
                    index,
                },
            );
        }

        Ok(Value::Promise(result))
    }

    pub(crate) fn eval_promise_race(&mut self, iterable: Value) -> Result<Value> {
        let values = self.array_like_values_from_value(&iterable)?;
        let result = self.new_pending_promise();
        if values.is_empty() {
            return Ok(Value::Promise(result));
        }

        let state = Rc::new(RefCell::new(PromiseRaceState {
            result: result.clone(),
            settled: false,
        }));

        for value in values {
            let promise = self.promise_resolve_value_as_promise(value)?;
            self.promise_add_reaction(
                &promise,
                PromiseReactionKind::Race {
                    state: state.clone(),
                },
            );
        }

        Ok(Value::Promise(result))
    }
}
