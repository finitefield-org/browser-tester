impl Harness {
    pub(super) fn promise_error_reason(err: Error) -> Value {
        Value::String(format!("{err}"))
    }

    pub(super) fn new_pending_promise(&mut self) -> Rc<RefCell<PromiseValue>> {
        let id = self.promise_runtime.allocate_promise_id();
        Rc::new(RefCell::new(PromiseValue {
            id,
            state: PromiseState::Pending,
            reactions: Vec::new(),
        }))
    }

    pub(super) fn new_promise_capability_functions(
        &self,
        promise: Rc<RefCell<PromiseValue>>,
    ) -> (Value, Value) {
        let already_called = Rc::new(RefCell::new(false));
        let resolve = Value::PromiseCapability(Rc::new(PromiseCapabilityFunction {
            promise: promise.clone(),
            reject: false,
            already_called: already_called.clone(),
        }));
        let reject = Value::PromiseCapability(Rc::new(PromiseCapabilityFunction {
            promise,
            reject: true,
            already_called,
        }));
        (resolve, reject)
    }

    pub(super) fn promise_add_reaction(
        &mut self,
        promise: &Rc<RefCell<PromiseValue>>,
        kind: PromiseReactionKind,
    ) {
        let settled = {
            let mut promise_ref = promise.borrow_mut();
            match &promise_ref.state {
                PromiseState::Pending => {
                    promise_ref.reactions.push(PromiseReaction { kind });
                    return;
                }
                PromiseState::Fulfilled(value) => PromiseSettledValue::Fulfilled(value.clone()),
                PromiseState::Rejected(reason) => PromiseSettledValue::Rejected(reason.clone()),
            }
        };
        self.queue_promise_reaction_microtask(kind, settled);
    }

    pub(super) fn promise_fulfill(&mut self, promise: &Rc<RefCell<PromiseValue>>, value: Value) {
        let reactions = {
            let mut promise_ref = promise.borrow_mut();
            if !matches!(promise_ref.state, PromiseState::Pending) {
                return;
            }
            promise_ref.state = PromiseState::Fulfilled(value.clone());
            std::mem::take(&mut promise_ref.reactions)
        };
        for reaction in reactions {
            self.queue_promise_reaction_microtask(
                reaction.kind,
                PromiseSettledValue::Fulfilled(value.clone()),
            );
        }
    }

    pub(super) fn promise_reject(&mut self, promise: &Rc<RefCell<PromiseValue>>, reason: Value) {
        let reactions = {
            let mut promise_ref = promise.borrow_mut();
            if !matches!(promise_ref.state, PromiseState::Pending) {
                return;
            }
            promise_ref.state = PromiseState::Rejected(reason.clone());
            std::mem::take(&mut promise_ref.reactions)
        };
        for reaction in reactions {
            self.queue_promise_reaction_microtask(
                reaction.kind,
                PromiseSettledValue::Rejected(reason.clone()),
            );
        }
    }

    pub(super) fn promise_resolve(
        &mut self,
        promise: &Rc<RefCell<PromiseValue>>,
        value: Value,
    ) -> Result<()> {
        if !matches!(promise.borrow().state, PromiseState::Pending) {
            return Ok(());
        }

        if let Value::Promise(other) = &value {
            if Rc::ptr_eq(other, promise) {
                self.promise_reject(
                    promise,
                    Value::String("TypeError: Cannot resolve promise with itself".into()),
                );
                return Ok(());
            }

            let settled = {
                let other_ref = other.borrow();
                match &other_ref.state {
                    PromiseState::Pending => None,
                    PromiseState::Fulfilled(value) => {
                        Some(PromiseSettledValue::Fulfilled(value.clone()))
                    }
                    PromiseState::Rejected(reason) => {
                        Some(PromiseSettledValue::Rejected(reason.clone()))
                    }
                }
            };

            if let Some(settled) = settled {
                match settled {
                    PromiseSettledValue::Fulfilled(value) => self.promise_fulfill(promise, value),
                    PromiseSettledValue::Rejected(reason) => self.promise_reject(promise, reason),
                }
            } else {
                self.promise_add_reaction(
                    other,
                    PromiseReactionKind::ResolveTo {
                        target: promise.clone(),
                    },
                );
            }
            return Ok(());
        }

        if let Value::Object(entries) = &value {
            let then = {
                let entries = entries.borrow();
                Self::object_get_entry(&entries, "then")
            };

            if let Some(then) = then {
                if self.is_callable_value(&then) {
                    let (resolve, reject) = self.new_promise_capability_functions(promise.clone());
                    let event = EventState::new("microtask", self.dom.root, self.scheduler.now_ms);
                    match self.execute_callable_value(&then, &[resolve, reject], &event) {
                        Ok(_) => {}
                        Err(err) => self.promise_reject(promise, Self::promise_error_reason(err)),
                    }
                    return Ok(());
                }
            }
        }

        self.promise_fulfill(promise, value);
        Ok(())
    }

    pub(super) fn promise_resolve_value_as_promise(
        &mut self,
        value: Value,
    ) -> Result<Rc<RefCell<PromiseValue>>> {
        if let Value::Promise(promise) = value {
            return Ok(promise);
        }
        let promise = self.new_pending_promise();
        self.promise_resolve(&promise, value)?;
        Ok(promise)
    }

    pub(super) fn promise_then_internal(
        &mut self,
        promise: &Rc<RefCell<PromiseValue>>,
        on_fulfilled: Option<Value>,
        on_rejected: Option<Value>,
    ) -> Rc<RefCell<PromiseValue>> {
        let result = self.new_pending_promise();
        self.promise_add_reaction(
            promise,
            PromiseReactionKind::Then {
                on_fulfilled,
                on_rejected,
                result: result.clone(),
            },
        );
        result
    }

    pub(super) fn eval_promise_construct(
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

    pub(super) fn eval_promise_static_method(
        &mut self,
        method: PromiseStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            PromiseStaticMethod::Resolve => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.resolve supports zero or one argument".into(),
                    ));
                }
                let value = if let Some(value) = args.first() {
                    self.eval_expr(value, env, event_param, event)?
                } else {
                    Value::Undefined
                };
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
                let reason = if let Some(reason) = args.first() {
                    self.eval_expr(reason, env, event_param, event)?
                } else {
                    Value::Undefined
                };
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
                let iterable = self.eval_expr(&args[0], env, event_param, event)?;
                self.eval_promise_all(iterable)
            }
            PromiseStaticMethod::AllSettled => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.allSettled requires exactly one argument".into(),
                    ));
                }
                let iterable = self.eval_expr(&args[0], env, event_param, event)?;
                self.eval_promise_all_settled(iterable)
            }
            PromiseStaticMethod::Any => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.any requires exactly one argument".into(),
                    ));
                }
                let iterable = self.eval_expr(&args[0], env, event_param, event)?;
                self.eval_promise_any(iterable)
            }
            PromiseStaticMethod::Race => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.race requires exactly one argument".into(),
                    ));
                }
                let iterable = self.eval_expr(&args[0], env, event_param, event)?;
                self.eval_promise_race(iterable)
            }
            PromiseStaticMethod::Try => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Promise.try requires at least one argument".into(),
                    ));
                }
                let callback = self.eval_expr(&args[0], env, event_param, event)?;
                let mut callback_args = Vec::with_capacity(args.len().saturating_sub(1));
                for arg in args.iter().skip(1) {
                    callback_args.push(self.eval_expr(arg, env, event_param, event)?);
                }
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

    pub(super) fn eval_promise_method(
        &mut self,
        target: &Expr,
        method: PromiseInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let target = self.eval_expr(target, env, event_param, event)?;
        let Value::Promise(promise) = target else {
            return Err(Error::ScriptRuntime(
                "Promise instance method target must be a Promise".into(),
            ));
        };

        match method {
            PromiseInstanceMethod::Then => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Promise.then supports up to two arguments".into(),
                    ));
                }
                let on_fulfilled = if let Some(arg) = args.first() {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    if self.is_callable_value(&value) {
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let on_rejected = if args.len() >= 2 {
                    let value = self.eval_expr(&args[1], env, event_param, event)?;
                    if self.is_callable_value(&value) {
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Value::Promise(self.promise_then_internal(
                    &promise,
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
                let on_rejected = if let Some(arg) = args.first() {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    if self.is_callable_value(&value) {
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Value::Promise(self.promise_then_internal(
                    &promise,
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
                let callback = if let Some(arg) = args.first() {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    if self.is_callable_value(&value) {
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let result = self.new_pending_promise();
                self.promise_add_reaction(
                    &promise,
                    PromiseReactionKind::Finally {
                        callback,
                        result: result.clone(),
                    },
                );
                Ok(Value::Promise(result))
            }
        }
    }

    pub(super) fn eval_promise_all(&mut self, iterable: Value) -> Result<Value> {
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

    pub(super) fn eval_promise_all_settled(&mut self, iterable: Value) -> Result<Value> {
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

    pub(super) fn eval_promise_any(&mut self, iterable: Value) -> Result<Value> {
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

    pub(super) fn eval_promise_race(&mut self, iterable: Value) -> Result<Value> {
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

    pub(super) fn new_aggregate_error_value(reasons: Vec<Value>) -> Value {
        Self::new_object_value(vec![
            ("name".into(), Value::String("AggregateError".into())),
            (
                "message".into(),
                Value::String("All promises were rejected".into()),
            ),
            ("errors".into(), Self::new_array_value(reasons)),
        ])
    }

    pub(super) fn run_promise_reaction_task(
        &mut self,
        reaction: PromiseReactionKind,
        settled: PromiseSettledValue,
    ) -> Result<()> {
        let event = EventState::new("microtask", self.dom.root, self.scheduler.now_ms);
        match reaction {
            PromiseReactionKind::Then {
                on_fulfilled,
                on_rejected,
                result,
            } => match settled {
                PromiseSettledValue::Fulfilled(value) => {
                    if let Some(callback) = on_fulfilled {
                        match self.execute_callable_value(
                            &callback,
                            std::slice::from_ref(&value),
                            &event,
                        ) {
                            Ok(next) => self.promise_resolve(&result, next)?,
                            Err(err) => {
                                self.promise_reject(&result, Self::promise_error_reason(err))
                            }
                        }
                    } else {
                        self.promise_fulfill(&result, value);
                    }
                }
                PromiseSettledValue::Rejected(reason) => {
                    if let Some(callback) = on_rejected {
                        match self.execute_callable_value(
                            &callback,
                            std::slice::from_ref(&reason),
                            &event,
                        ) {
                            Ok(next) => self.promise_resolve(&result, next)?,
                            Err(err) => {
                                self.promise_reject(&result, Self::promise_error_reason(err))
                            }
                        }
                    } else {
                        self.promise_reject(&result, reason);
                    }
                }
            },
            PromiseReactionKind::Finally { callback, result } => {
                if let Some(callback) = callback {
                    match self.execute_callable_value(&callback, &[], &event) {
                        Ok(next) => {
                            let continuation = self.promise_resolve_value_as_promise(next)?;
                            self.promise_add_reaction(
                                &continuation,
                                PromiseReactionKind::FinallyContinuation {
                                    original: settled,
                                    result,
                                },
                            );
                        }
                        Err(err) => self.promise_reject(&result, Self::promise_error_reason(err)),
                    }
                } else {
                    match settled {
                        PromiseSettledValue::Fulfilled(value) => {
                            self.promise_fulfill(&result, value)
                        }
                        PromiseSettledValue::Rejected(reason) => {
                            self.promise_reject(&result, reason)
                        }
                    }
                }
            }
            PromiseReactionKind::FinallyContinuation { original, result } => match settled {
                PromiseSettledValue::Fulfilled(_) => match original {
                    PromiseSettledValue::Fulfilled(value) => self.promise_fulfill(&result, value),
                    PromiseSettledValue::Rejected(reason) => self.promise_reject(&result, reason),
                },
                PromiseSettledValue::Rejected(reason) => self.promise_reject(&result, reason),
            },
            PromiseReactionKind::ResolveTo { target } => match settled {
                PromiseSettledValue::Fulfilled(value) => self.promise_resolve(&target, value)?,
                PromiseSettledValue::Rejected(reason) => self.promise_reject(&target, reason),
            },
            PromiseReactionKind::All { state, index } => {
                let mut state_ref = state.borrow_mut();
                if state_ref.settled {
                    return Ok(());
                }
                match settled {
                    PromiseSettledValue::Fulfilled(value) => {
                        if state_ref.values[index].is_none() {
                            state_ref.values[index] = Some(value);
                            state_ref.remaining = state_ref.remaining.saturating_sub(1);
                        }
                        if state_ref.remaining == 0 {
                            state_ref.settled = true;
                            let result = state_ref.result.clone();
                            let values = state_ref
                                .values
                                .iter()
                                .map(|value| value.clone().unwrap_or(Value::Undefined))
                                .collect::<Vec<_>>();
                            drop(state_ref);
                            self.promise_fulfill(&result, Self::new_array_value(values));
                        }
                    }
                    PromiseSettledValue::Rejected(reason) => {
                        state_ref.settled = true;
                        let result = state_ref.result.clone();
                        drop(state_ref);
                        self.promise_reject(&result, reason);
                    }
                }
            }
            PromiseReactionKind::AllSettled { state, index } => {
                let mut state_ref = state.borrow_mut();
                if state_ref.remaining == 0 {
                    return Ok(());
                }
                if state_ref.values[index].is_none() {
                    let entry = match settled {
                        PromiseSettledValue::Fulfilled(value) => Self::new_object_value(vec![
                            ("status".into(), Value::String("fulfilled".into())),
                            ("value".into(), value),
                        ]),
                        PromiseSettledValue::Rejected(reason) => Self::new_object_value(vec![
                            ("status".into(), Value::String("rejected".into())),
                            ("reason".into(), reason),
                        ]),
                    };
                    state_ref.values[index] = Some(entry);
                    state_ref.remaining = state_ref.remaining.saturating_sub(1);
                }
                if state_ref.remaining == 0 {
                    let result = state_ref.result.clone();
                    let values = state_ref
                        .values
                        .iter()
                        .map(|value| value.clone().unwrap_or(Value::Undefined))
                        .collect::<Vec<_>>();
                    drop(state_ref);
                    self.promise_fulfill(&result, Self::new_array_value(values));
                }
            }
            PromiseReactionKind::Any { state, index } => {
                let mut state_ref = state.borrow_mut();
                if state_ref.settled {
                    return Ok(());
                }
                match settled {
                    PromiseSettledValue::Fulfilled(value) => {
                        state_ref.settled = true;
                        let result = state_ref.result.clone();
                        drop(state_ref);
                        self.promise_fulfill(&result, value);
                    }
                    PromiseSettledValue::Rejected(reason) => {
                        if state_ref.reasons[index].is_none() {
                            state_ref.reasons[index] = Some(reason);
                            state_ref.remaining = state_ref.remaining.saturating_sub(1);
                        }
                        if state_ref.remaining == 0 {
                            state_ref.settled = true;
                            let result = state_ref.result.clone();
                            let reasons = state_ref
                                .reasons
                                .iter()
                                .map(|reason| reason.clone().unwrap_or(Value::Undefined))
                                .collect::<Vec<_>>();
                            drop(state_ref);
                            self.promise_reject(&result, Self::new_aggregate_error_value(reasons));
                        }
                    }
                }
            }
            PromiseReactionKind::Race { state } => {
                let mut state_ref = state.borrow_mut();
                if state_ref.settled {
                    return Ok(());
                }
                state_ref.settled = true;
                let result = state_ref.result.clone();
                drop(state_ref);
                match settled {
                    PromiseSettledValue::Fulfilled(value) => self.promise_fulfill(&result, value),
                    PromiseSettledValue::Rejected(reason) => self.promise_reject(&result, reason),
                }
            }
        }
        Ok(())
    }

    pub(super) fn execute_callback_value(
        &mut self,
        callback: &Value,
        args: &[Value],
        event: &EventState,
    ) -> Result<Value> {
        self.execute_callable_value(callback, args, event)
    }

    pub(super) fn eval_typed_array_method(
        &mut self,
        target: &str,
        method: TypedArrayInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !matches!(env.get(target), Some(Value::TypedArray(_))) {
            let Some(target_value) = env.get(target) else {
                return Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                )));
            };

            if let Value::Map(map) = target_value {
                return match method {
                    TypedArrayInstanceMethod::Set => {
                        if args.len() != 2 {
                            return Err(Error::ScriptRuntime(
                                "Map.set requires exactly two arguments".into(),
                            ));
                        }
                        let key = self.eval_expr(&args[0], env, event_param, event)?;
                        let value = self.eval_expr(&args[1], env, event_param, event)?;
                        self.map_set_entry(&mut map.borrow_mut(), key, value);
                        Ok(Value::Map(map.clone()))
                    }
                    TypedArrayInstanceMethod::Entries => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Map.entries does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(self.map_entries_array(map)))
                    }
                    TypedArrayInstanceMethod::Keys => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Map.keys does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(
                            map.borrow()
                                .entries
                                .iter()
                                .map(|(key, _)| key.clone())
                                .collect(),
                        ))
                    }
                    TypedArrayInstanceMethod::Values => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Map.values does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(
                            map.borrow()
                                .entries
                                .iter()
                                .map(|(_, value)| value.clone())
                                .collect(),
                        ))
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a TypedArray",
                        target
                    ))),
                };
            }

            if let Value::Set(set) = target_value {
                return match method {
                    TypedArrayInstanceMethod::Entries => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Set.entries does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(self.set_entries_array(set)))
                    }
                    TypedArrayInstanceMethod::Keys | TypedArrayInstanceMethod::Values => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Set.keys/values does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(self.set_values_array(set)))
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a TypedArray",
                        target
                    ))),
                };
            }

            if let Value::Object(entries) = target_value {
                if Self::is_url_search_params_object(&entries.borrow()) {
                    return match method {
                        TypedArrayInstanceMethod::Set => {
                            if args.len() != 2 {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.set requires exactly two arguments".into(),
                                ));
                            }
                            let name = self
                                .eval_expr(&args[0], env, event_param, event)?
                                .as_string();
                            let value = self
                                .eval_expr(&args[1], env, event_param, event)?
                                .as_string();
                            {
                                let mut object_ref = entries.borrow_mut();
                                let mut pairs =
                                    Self::url_search_params_pairs_from_object_entries(&object_ref);
                                if let Some(first_match) =
                                    pairs.iter().position(|(entry_name, _)| entry_name == &name)
                                {
                                    pairs[first_match].1 = value;
                                    let mut index = pairs.len();
                                    while index > 0 {
                                        index -= 1;
                                        if index != first_match && pairs[index].0 == name {
                                            pairs.remove(index);
                                        }
                                    }
                                } else {
                                    pairs.push((name, value));
                                }
                                Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                            }
                            self.sync_url_search_params_owner(entries);
                            Ok(Value::Undefined)
                        }
                        TypedArrayInstanceMethod::Entries => {
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.entries does not take arguments".into(),
                                ));
                            }
                            let pairs = Self::url_search_params_pairs_from_object_entries(
                                &entries.borrow(),
                            );
                            Ok(Self::new_array_value(
                                pairs
                                    .into_iter()
                                    .map(|(name, value)| {
                                        Self::new_array_value(vec![
                                            Value::String(name),
                                            Value::String(value),
                                        ])
                                    })
                                    .collect::<Vec<_>>(),
                            ))
                        }
                        TypedArrayInstanceMethod::Keys => {
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.keys does not take arguments".into(),
                                ));
                            }
                            let pairs = Self::url_search_params_pairs_from_object_entries(
                                &entries.borrow(),
                            );
                            Ok(Self::new_array_value(
                                pairs
                                    .into_iter()
                                    .map(|(name, _)| Value::String(name))
                                    .collect::<Vec<_>>(),
                            ))
                        }
                        TypedArrayInstanceMethod::Values => {
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.values does not take arguments".into(),
                                ));
                            }
                            let pairs = Self::url_search_params_pairs_from_object_entries(
                                &entries.borrow(),
                            );
                            Ok(Self::new_array_value(
                                pairs
                                    .into_iter()
                                    .map(|(_, value)| Value::String(value))
                                    .collect::<Vec<_>>(),
                            ))
                        }
                        TypedArrayInstanceMethod::Sort => {
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.sort does not take arguments".into(),
                                ));
                            }
                            {
                                let mut object_ref = entries.borrow_mut();
                                let mut pairs =
                                    Self::url_search_params_pairs_from_object_entries(&object_ref);
                                pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                                Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                            }
                            self.sync_url_search_params_owner(entries);
                            Ok(Value::Undefined)
                        }
                        _ => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not a TypedArray",
                            target
                        ))),
                    };
                }
            }

            if matches!(method, TypedArrayInstanceMethod::At) {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "at supports zero or one argument".into(),
                    ));
                }
                let index = if let Some(index) = args.first() {
                    Self::value_to_i64(&self.eval_expr(index, env, event_param, event)?)
                } else {
                    0
                };

                return match target_value {
                    Value::String(value) => {
                        let len = value.chars().count() as i64;
                        let index = if index < 0 { len + index } else { index };
                        if index < 0 || index >= len {
                            Ok(Value::Undefined)
                        } else {
                            Ok(value
                                .chars()
                                .nth(index as usize)
                                .map(|ch| Value::String(ch.to_string()))
                                .unwrap_or(Value::Undefined))
                        }
                    }
                    Value::Object(entries) => {
                        let entries = entries.borrow();
                        if let Some(value) = Self::string_wrapper_value_from_object(&entries) {
                            let len = value.chars().count() as i64;
                            let index = if index < 0 { len + index } else { index };
                            if index < 0 || index >= len {
                                Ok(Value::Undefined)
                            } else {
                                Ok(value
                                    .chars()
                                    .nth(index as usize)
                                    .map(|ch| Value::String(ch.to_string()))
                                    .unwrap_or(Value::Undefined))
                            }
                        } else {
                            Err(Error::ScriptRuntime(format!(
                                "variable '{}' is not a TypedArray",
                                target
                            )))
                        }
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a TypedArray",
                        target
                    ))),
                };
            }

            if matches!(
                method,
                TypedArrayInstanceMethod::IndexOf | TypedArrayInstanceMethod::LastIndexOf
            ) {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "indexOf requires one or two arguments".into(),
                    ));
                }
                let search = self.eval_expr(&args[0], env, event_param, event)?;

                return match target_value {
                    Value::String(value) => {
                        let len = value.chars().count() as i64;
                        if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                            let mut start = if args.len() == 2 {
                                Self::value_to_i64(&self.eval_expr(
                                    &args[1],
                                    env,
                                    event_param,
                                    event,
                                )?)
                            } else {
                                0
                            };
                            if start < 0 {
                                start = 0;
                            }
                            if start > len {
                                start = len;
                            }
                            let index =
                                Self::string_index_of(value, &search.as_string(), start as usize)
                                    .map(|idx| idx as i64)
                                    .unwrap_or(-1);
                            Ok(Value::Number(index))
                        } else {
                            let mut from = if args.len() == 2 {
                                Self::value_to_i64(&self.eval_expr(
                                    &args[1],
                                    env,
                                    event_param,
                                    event,
                                )?)
                            } else {
                                len
                            };
                            if from < 0 {
                                from = 0;
                            }
                            if from > len {
                                from = len;
                            }
                            let from = from as usize;
                            let search = search.as_string();
                            if search.is_empty() {
                                return Ok(Value::Number(from as i64));
                            }
                            for idx in (0..=from).rev() {
                                let byte_idx = Self::char_index_to_byte(value, idx);
                                if value[byte_idx..].starts_with(&search) {
                                    return Ok(Value::Number(idx as i64));
                                }
                            }
                            Ok(Value::Number(-1))
                        }
                    }
                    Value::Array(values) => {
                        let from = if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                            let len = values.borrow().len() as i64;
                            let mut from = if args.len() == 2 {
                                Self::value_to_i64(&self.eval_expr(
                                    &args[1],
                                    env,
                                    event_param,
                                    event,
                                )?)
                            } else {
                                0
                            };
                            if from < 0 {
                                from = (len + from).max(0);
                            }
                            if from > len {
                                from = len;
                            }
                            from
                        } else {
                            let len = values.borrow().len() as i64;
                            let from = if args.len() == 2 {
                                Self::value_to_i64(&self.eval_expr(
                                    &args[1],
                                    env,
                                    event_param,
                                    event,
                                )?)
                            } else {
                                len - 1
                            };
                            if from < 0 {
                                (len + from).max(-1)
                            } else {
                                from.min(len - 1)
                            }
                        };

                        let values = values.borrow();
                        if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                            for (index, value) in values.iter().enumerate().skip(from as usize) {
                                if self.strict_equal(value, &search) {
                                    return Ok(Value::Number(index as i64));
                                }
                            }
                            Ok(Value::Number(-1))
                        } else {
                            if from < 0 {
                                return Ok(Value::Number(-1));
                            }
                            for index in (0..=from as usize).rev() {
                                if self.strict_equal(&values[index], &search) {
                                    return Ok(Value::Number(index as i64));
                                }
                            }
                            Ok(Value::Number(-1))
                        }
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a TypedArray",
                        target
                    ))),
                };
            }

            return Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a TypedArray",
                target
            )));
        }

        let array = self.resolve_typed_array_from_env(env, target)?;
        if array.borrow().buffer.borrow().detached {
            return Err(Error::ScriptRuntime(
                "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
            ));
        }
        let kind = array.borrow().kind;
        let len = array.borrow().observed_length();
        let this_value = Value::TypedArray(array.clone());

        match method {
            TypedArrayInstanceMethod::At => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.at requires exactly one argument".into(),
                    ));
                }
                let index = self.eval_expr(&args[0], env, event_param, event)?;
                let mut index = Self::value_to_i64(&index);
                let len_i64 = len as i64;
                if index < 0 {
                    index += len_i64;
                }
                if index < 0 || index >= len_i64 {
                    return Ok(Value::Undefined);
                }
                self.typed_array_get_index(&array, index as usize)
            }
            TypedArrayInstanceMethod::CopyWithin => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.copyWithin requires 2 or 3 arguments".into(),
                    ));
                }
                let target_index =
                    Self::value_to_i64(&self.eval_expr(&args[0], env, event_param, event)?);
                let start_index =
                    Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?);
                let end_index = if args.len() == 3 {
                    Self::value_to_i64(&self.eval_expr(&args[2], env, event_param, event)?)
                } else {
                    len as i64
                };
                let target_index = Self::normalize_slice_index(len, target_index);
                let start_index = Self::normalize_slice_index(len, start_index);
                let end_index = Self::normalize_slice_index(len, end_index);
                let end_index = end_index.max(start_index);
                let count = end_index
                    .saturating_sub(start_index)
                    .min(len.saturating_sub(target_index));
                let snapshot = self.typed_array_snapshot(&array)?;
                for offset in 0..count {
                    self.typed_array_set_index(
                        &array,
                        target_index + offset,
                        snapshot[start_index + offset].clone(),
                    )?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::Entries => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.entries does not take arguments".into(),
                    ));
                }
                let snapshot = self.typed_array_snapshot(&array)?;
                let out = snapshot
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| {
                        Self::new_array_value(vec![Value::Number(index as i64), value])
                    })
                    .collect::<Vec<_>>();
                Ok(Self::new_array_value(out))
            }
            TypedArrayInstanceMethod::Fill => {
                if args.is_empty() || args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.fill requires 1 to 3 arguments".into(),
                    ));
                }
                let value = self.eval_expr(&args[0], env, event_param, event)?;
                let start = if args.len() >= 2 {
                    Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                } else {
                    0
                };
                let end = if args.len() == 3 {
                    Self::value_to_i64(&self.eval_expr(&args[2], env, event_param, event)?)
                } else {
                    len as i64
                };
                let start = Self::normalize_slice_index(len, start);
                let end = Self::normalize_slice_index(len, end).max(start);
                for index in start..end {
                    self.typed_array_set_index(&array, index, value.clone())?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::FindIndex
            | TypedArrayInstanceMethod::FindLast
            | TypedArrayInstanceMethod::FindLastIndex => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray find callback methods require exactly one argument".into(),
                    ));
                }
                let callback = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = self.typed_array_snapshot(&array)?;
                let iter: Box<dyn Iterator<Item = (usize, Value)>> = match method {
                    TypedArrayInstanceMethod::FindLast
                    | TypedArrayInstanceMethod::FindLastIndex => {
                        Box::new(snapshot.into_iter().enumerate().rev())
                    }
                    _ => Box::new(snapshot.into_iter().enumerate()),
                };
                for (index, value) in iter {
                    let matched = self.execute_callback_value(
                        &callback,
                        &[
                            value.clone(),
                            Value::Number(index as i64),
                            this_value.clone(),
                        ],
                        event,
                    )?;
                    if matched.truthy() {
                        return if matches!(
                            method,
                            TypedArrayInstanceMethod::FindLastIndex
                                | TypedArrayInstanceMethod::FindIndex
                        ) {
                            Ok(Value::Number(index as i64))
                        } else {
                            Ok(value)
                        };
                    }
                }
                if matches!(
                    method,
                    TypedArrayInstanceMethod::FindLastIndex | TypedArrayInstanceMethod::FindIndex
                ) {
                    Ok(Value::Number(-1))
                } else {
                    Ok(Value::Undefined)
                }
            }
            TypedArrayInstanceMethod::IndexOf | TypedArrayInstanceMethod::LastIndexOf => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray indexOf methods require one or two arguments".into(),
                    ));
                }
                let search = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = self.typed_array_snapshot(&array)?;
                if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                    let from = if args.len() == 2 {
                        Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                    } else {
                        0
                    };
                    let mut from = if from < 0 {
                        (len as i64 + from).max(0)
                    } else {
                        from
                    };
                    if from > len as i64 {
                        from = len as i64;
                    }
                    for (index, value) in snapshot.iter().enumerate().skip(from as usize) {
                        if self.strict_equal(value, &search) {
                            return Ok(Value::Number(index as i64));
                        }
                    }
                    Ok(Value::Number(-1))
                } else {
                    let from = if args.len() == 2 {
                        Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                    } else {
                        (len as i64) - 1
                    };
                    let from = if from < 0 {
                        (len as i64 + from).max(-1)
                    } else {
                        from.min((len as i64) - 1)
                    };
                    if from < 0 {
                        return Ok(Value::Number(-1));
                    }
                    for index in (0..=from as usize).rev() {
                        if self.strict_equal(&snapshot[index], &search) {
                            return Ok(Value::Number(index as i64));
                        }
                    }
                    Ok(Value::Number(-1))
                }
            }
            TypedArrayInstanceMethod::Keys => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.keys does not take arguments".into(),
                    ));
                }
                Ok(Self::new_array_value(
                    (0..len).map(|index| Value::Number(index as i64)).collect(),
                ))
            }
            TypedArrayInstanceMethod::ReduceRight => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.reduceRight requires callback and optional initial value"
                            .into(),
                    ));
                }
                let callback = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = self.typed_array_snapshot(&array)?;
                let mut iter = snapshot.into_iter().enumerate().rev();
                let mut acc = if args.len() == 2 {
                    self.eval_expr(&args[1], env, event_param, event)?
                } else {
                    let Some((_, first)) = iter.next() else {
                        return Err(Error::ScriptRuntime(
                            "reduce of empty array with no initial value".into(),
                        ));
                    };
                    first
                };
                for (index, value) in iter {
                    acc = self.execute_callback_value(
                        &callback,
                        &[acc, value, Value::Number(index as i64), this_value.clone()],
                        event,
                    )?;
                }
                Ok(acc)
            }
            TypedArrayInstanceMethod::Reverse => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.reverse does not take arguments".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                snapshot.reverse();
                for (index, value) in snapshot.into_iter().enumerate() {
                    self.typed_array_set_index(&array, index, value)?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::Set => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.set requires source and optional offset".into(),
                    ));
                }
                let source = self.eval_expr(&args[0], env, event_param, event)?;
                let source_values = self.array_like_values_from_value(&source)?;
                let offset = if args.len() == 2 {
                    Self::to_non_negative_usize(
                        &self.eval_expr(&args[1], env, event_param, event)?,
                        "TypedArray.set offset",
                    )?
                } else {
                    0
                };
                if offset > len || source_values.len() > len.saturating_sub(offset) {
                    return Err(Error::ScriptRuntime(
                        "source array is too large for target TypedArray".into(),
                    ));
                }
                for (index, value) in source_values.into_iter().enumerate() {
                    self.typed_array_set_index(&array, offset + index, value)?;
                }
                Ok(Value::Undefined)
            }
            TypedArrayInstanceMethod::Sort => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.sort supports at most one argument".into(),
                    ));
                }
                if args.len() == 1 {
                    return Err(Error::ScriptRuntime(
                        "custom comparator for TypedArray.sort is not supported".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                if kind.is_bigint() {
                    snapshot.sort_by(|left, right| {
                        let left = match left {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        let right = match right {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        left.cmp(&right)
                    });
                } else {
                    snapshot.sort_by(|left, right| {
                        let left = Self::coerce_number_for_global(left);
                        let right = Self::coerce_number_for_global(right);
                        match (left.is_nan(), right.is_nan()) {
                            (true, true) => std::cmp::Ordering::Equal,
                            (true, false) => std::cmp::Ordering::Greater,
                            (false, true) => std::cmp::Ordering::Less,
                            (false, false) => left
                                .partial_cmp(&right)
                                .unwrap_or(std::cmp::Ordering::Equal),
                        }
                    });
                }
                for (index, value) in snapshot.into_iter().enumerate() {
                    self.typed_array_set_index(&array, index, value)?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::Subarray => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.subarray supports at most two arguments".into(),
                    ));
                }
                let begin = if !args.is_empty() {
                    Self::value_to_i64(&self.eval_expr(&args[0], env, event_param, event)?)
                } else {
                    0
                };
                let end = if args.len() == 2 {
                    Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                } else {
                    len as i64
                };
                let begin = Self::normalize_slice_index(len, begin);
                let end = Self::normalize_slice_index(len, end).max(begin);
                let byte_offset = array
                    .borrow()
                    .byte_offset
                    .saturating_add(begin.saturating_mul(kind.bytes_per_element()));
                self.new_typed_array_view(
                    kind,
                    array.borrow().buffer.clone(),
                    byte_offset,
                    Some(end.saturating_sub(begin)),
                )
            }
            TypedArrayInstanceMethod::ToReversed => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.toReversed does not take arguments".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                snapshot.reverse();
                self.new_typed_array_from_values(kind, &snapshot)
            }
            TypedArrayInstanceMethod::ToSorted => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.toSorted supports at most one argument".into(),
                    ));
                }
                if args.len() == 1 {
                    return Err(Error::ScriptRuntime(
                        "custom comparator for TypedArray.toSorted is not supported".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                if kind.is_bigint() {
                    snapshot.sort_by(|left, right| {
                        let left = match left {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        let right = match right {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        left.cmp(&right)
                    });
                } else {
                    snapshot.sort_by(|left, right| {
                        let left = Self::coerce_number_for_global(left);
                        let right = Self::coerce_number_for_global(right);
                        match (left.is_nan(), right.is_nan()) {
                            (true, true) => std::cmp::Ordering::Equal,
                            (true, false) => std::cmp::Ordering::Greater,
                            (false, true) => std::cmp::Ordering::Less,
                            (false, false) => left
                                .partial_cmp(&right)
                                .unwrap_or(std::cmp::Ordering::Equal),
                        }
                    });
                }
                self.new_typed_array_from_values(kind, &snapshot)
            }
            TypedArrayInstanceMethod::Values => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.values does not take arguments".into(),
                    ));
                }
                Ok(Self::new_array_value(self.typed_array_snapshot(&array)?))
            }
            TypedArrayInstanceMethod::With => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.with requires exactly two arguments".into(),
                    ));
                }
                let index =
                    Self::value_to_i64(&self.eval_expr(&args[0], env, event_param, event)?);
                let value = self.eval_expr(&args[1], env, event_param, event)?;
                let index = if index < 0 {
                    (len as i64) + index
                } else {
                    index
                };
                if index < 0 || index >= len as i64 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.with index out of range".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                snapshot[index as usize] = value;
                self.new_typed_array_from_values(kind, &snapshot)
            }
        }
    }

    pub(super) fn execute_array_callback(
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

    pub(super) fn execute_array_callback_in_env(
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

    pub(super) fn execute_array_like_foreach_in_env(
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
                        &[item, Value::Number(idx as i64), Value::Array(values.clone())],
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
                    let snapshot = Self::url_search_params_pairs_from_object_entries(&entries.borrow());
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

    pub(super) fn normalize_slice_index(len: usize, index: i64) -> usize {
        if index < 0 {
            len.saturating_sub(index.unsigned_abs() as usize)
        } else {
            (index as usize).min(len)
        }
    }

    pub(super) fn normalize_splice_start_index(len: usize, start: i64) -> usize {
        if start < 0 {
            len.saturating_sub(start.unsigned_abs() as usize)
        } else {
            (start as usize).min(len)
        }
    }

    pub(super) fn normalize_substring_index(len: usize, index: i64) -> usize {
        if index < 0 {
            0
        } else {
            (index as usize).min(len)
        }
    }

    pub(super) fn char_index_to_byte(value: &str, char_index: usize) -> usize {
        value
            .char_indices()
            .nth(char_index)
            .map(|(idx, _)| idx)
            .unwrap_or(value.len())
    }

    pub(super) fn substring_chars(value: &str, start: usize, end: usize) -> String {
        if start >= end {
            return String::new();
        }
        value.chars().skip(start).take(end - start).collect()
    }

    pub(super) fn split_string(
        value: &str,
        separator: Option<String>,
        limit: Option<i64>,
    ) -> Vec<Value> {
        let mut parts = match separator {
            None => vec![Value::String(value.to_string())],
            Some(separator) => {
                if separator.is_empty() {
                    value
                        .chars()
                        .map(|ch| Value::String(ch.to_string()))
                        .collect::<Vec<_>>()
                } else {
                    value
                        .split(&separator)
                        .map(|part| Value::String(part.to_string()))
                        .collect::<Vec<_>>()
                }
            }
        };

        if let Some(limit) = limit {
            if limit == 0 {
                parts.clear();
            } else if limit > 0 {
                parts.truncate(limit as usize);
            }
        }

        parts
    }

    pub(super) fn split_string_with_regex(
        value: &str,
        regex: &Rc<RefCell<RegexValue>>,
        limit: Option<i64>,
    ) -> Result<Vec<Value>> {
        let compiled = regex.borrow().compiled.clone();
        let mut parts = Vec::new();
        for part in compiled
            .split_all(value)
            .map_err(Self::map_regex_runtime_error)?
        {
            parts.push(Value::String(part));
        }
        if let Some(limit) = limit {
            if limit == 0 {
                parts.clear();
            } else if limit > 0 {
                parts.truncate(limit as usize);
            }
        }
        Ok(parts)
    }

    pub(super) fn expand_regex_replacement(template: &str, captures: &Captures) -> String {
        let chars = template.chars().collect::<Vec<_>>();
        let mut i = 0usize;
        let mut out = String::new();
        while i < chars.len() {
            if chars[i] != '$' {
                out.push(chars[i]);
                i += 1;
                continue;
            }
            if i + 1 >= chars.len() {
                out.push('$');
                i += 1;
                continue;
            }
            let next = chars[i + 1];
            match next {
                '$' => {
                    out.push('$');
                    i += 2;
                }
                '&' => {
                    if let Some(full) = captures.get(0) {
                        out.push_str(full.as_str());
                    }
                    i += 2;
                }
                '0'..='9' => {
                    let mut idx = (next as u8 - b'0') as usize;
                    let mut consumed = 2usize;
                    if i + 2 < chars.len() && chars[i + 2].is_ascii_digit() {
                        let candidate = idx * 10 + (chars[i + 2] as u8 - b'0') as usize;
                        if captures.get(candidate).is_some() {
                            idx = candidate;
                            consumed = 3;
                        }
                    }
                    if idx > 0 {
                        if let Some(group) = captures.get(idx) {
                            out.push_str(group.as_str());
                        }
                    } else {
                        out.push('$');
                        out.push('0');
                    }
                    i += consumed;
                }
                _ => {
                    out.push('$');
                    out.push(next);
                    i += 2;
                }
            }
        }
        out
    }

    pub(super) fn replace_string_with_regex(
        value: &str,
        regex: &Rc<RefCell<RegexValue>>,
        replacement: &str,
    ) -> Result<String> {
        let (compiled, global) = {
            let regex = regex.borrow();
            (regex.compiled.clone(), regex.global)
        };

        if global {
            let mut out = String::new();
            let mut last_end = 0usize;
            for captures in compiled
                .captures_all(value)
                .map_err(Self::map_regex_runtime_error)?
            {
                let Some(full) = captures.get(0) else {
                    continue;
                };
                out.push_str(&value[last_end..full.start()]);
                out.push_str(&Self::expand_regex_replacement(replacement, &captures));
                last_end = full.end();
            }
            out.push_str(&value[last_end..]);
            Ok(out)
        } else if let Some(captures) = compiled
            .captures(value)
            .map_err(Self::map_regex_runtime_error)?
        {
            if let Some(full) = captures.get(0) {
                let mut out = String::new();
                out.push_str(&value[..full.start()]);
                out.push_str(&Self::expand_regex_replacement(replacement, &captures));
                out.push_str(&value[full.end()..]);
                Ok(out)
            } else {
                Ok(value.to_string())
            }
        } else {
            Ok(value.to_string())
        }
    }

    pub(super) fn string_index_of(
        value: &str,
        search: &str,
        start_char_idx: usize,
    ) -> Option<usize> {
        let start_byte = Self::char_index_to_byte(value, start_char_idx);
        let pos = value.get(start_byte..)?.find(search)?;
        Some(value[..start_byte + pos].chars().count())
    }

    pub(super) fn parse_date_string_to_epoch_ms(src: &str) -> Option<i64> {
        let src = src.trim();
        if src.is_empty() {
            return None;
        }

        let bytes = src.as_bytes();
        let mut i = 0usize;

        let mut sign = 1i64;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            if bytes[i] == b'-' {
                sign = -1;
            }
            i += 1;
        }

        let year_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i <= year_start || (i - year_start) < 4 {
            return None;
        }
        let year = sign * src.get(year_start..i)?.parse::<i64>().ok()?;

        if i >= bytes.len() || bytes[i] != b'-' {
            return None;
        }
        i += 1;
        let month = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
        if i >= bytes.len() || bytes[i] != b'-' {
            return None;
        }
        i += 1;
        let day = Self::parse_fixed_digits_i64(src, &mut i, 2)?;

        let month = u32::try_from(month).ok()?;
        if !(1..=12).contains(&month) {
            return None;
        }
        let day = u32::try_from(day).ok()?;
        if day == 0 || day > Self::days_in_month(year, month) {
            return None;
        }

        let mut hour = 0i64;
        let mut minute = 0i64;
        let mut second = 0i64;
        let mut millisecond = 0i64;
        let mut offset_minutes = 0i64;

        if i < bytes.len() {
            if bytes[i] != b'T' && bytes[i] != b' ' {
                return None;
            }
            i += 1;

            hour = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
            if i >= bytes.len() || bytes[i] != b':' {
                return None;
            }
            i += 1;
            minute = Self::parse_fixed_digits_i64(src, &mut i, 2)?;

            if i < bytes.len() && bytes[i] == b':' {
                i += 1;
                second = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
            }

            if i < bytes.len() && bytes[i] == b'.' {
                i += 1;
                let frac_start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i == frac_start {
                    return None;
                }

                let frac = src.get(frac_start..i)?;
                let mut parsed = 0i64;
                let mut digits = 0usize;
                for ch in frac.chars().take(3) {
                    parsed = parsed * 10 + i64::from(ch.to_digit(10)?);
                    digits += 1;
                }
                while digits < 3 {
                    parsed *= 10;
                    digits += 1;
                }
                millisecond = parsed;
            }

            if i < bytes.len() {
                match bytes[i] {
                    b'Z' | b'z' => {
                        i += 1;
                    }
                    b'+' | b'-' => {
                        let tz_sign = if bytes[i] == b'+' { 1 } else { -1 };
                        i += 1;
                        let tz_hour = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
                        let tz_minute = if i < bytes.len() && bytes[i] == b':' {
                            i += 1;
                            Self::parse_fixed_digits_i64(src, &mut i, 2)?
                        } else {
                            Self::parse_fixed_digits_i64(src, &mut i, 2)?
                        };
                        if tz_hour > 23 || tz_minute > 59 {
                            return None;
                        }
                        offset_minutes = tz_sign * (tz_hour * 60 + tz_minute);
                    }
                    _ => return None,
                }
            }
        }

        if i != bytes.len() {
            return None;
        }
        if hour > 23 || minute > 59 || second > 59 {
            return None;
        }

        let timestamp_ms = Self::utc_timestamp_ms_from_components(
            year,
            i64::from(month) - 1,
            i64::from(day),
            hour,
            minute,
            second,
            millisecond,
        );
        Some(timestamp_ms - offset_minutes * 60_000)
    }

    pub(super) fn parse_fixed_digits_i64(src: &str, i: &mut usize, width: usize) -> Option<i64> {
        let end = i.checked_add(width)?;
        let segment = src.get(*i..end)?;
        if !segment.as_bytes().iter().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *i = end;
        segment.parse::<i64>().ok()
    }

    pub(super) fn format_iso_8601_utc(timestamp_ms: i64) -> String {
        let (year, month, day, hour, minute, second, millisecond) =
            Self::date_components_utc(timestamp_ms);
        let year_str = if (0..=9999).contains(&year) {
            format!("{year:04}")
        } else if year < 0 {
            format!("-{:06}", -(year as i128))
        } else {
            format!("+{:06}", year)
        };
        format!(
            "{year_str}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millisecond:03}Z"
        )
    }

    pub(super) fn date_components_utc(timestamp_ms: i64) -> (i64, u32, u32, u32, u32, u32, u32) {
        let days = timestamp_ms.div_euclid(86_400_000);
        let rem = timestamp_ms.rem_euclid(86_400_000);
        let hour = (rem / 3_600_000) as u32;
        let minute = ((rem % 3_600_000) / 60_000) as u32;
        let second = ((rem % 60_000) / 1_000) as u32;
        let millisecond = (rem % 1_000) as u32;
        let (year, month, day) = Self::civil_from_days(days);
        (year, month, day, hour, minute, second, millisecond)
    }

    pub(super) fn utc_timestamp_ms_from_components(
        year: i64,
        month_zero_based: i64,
        day: i64,
        hour: i64,
        minute: i64,
        second: i64,
        millisecond: i64,
    ) -> i64 {
        let (norm_year, norm_month) = Self::normalize_year_month(year, month_zero_based);
        let mut days = Self::days_from_civil(norm_year, norm_month, 1) + (day - 1);
        let mut time_ms = ((hour * 60 + minute) * 60 + second) * 1_000 + millisecond;
        days += time_ms.div_euclid(86_400_000);
        time_ms = time_ms.rem_euclid(86_400_000);

        let out = (days as i128) * 86_400_000i128 + (time_ms as i128);
        out.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
    }

    pub(super) fn normalize_year_month(year: i64, month_zero_based: i64) -> (i64, u32) {
        let total_month = year.saturating_mul(12).saturating_add(month_zero_based);
        let norm_year = total_month.div_euclid(12);
        let norm_month = total_month.rem_euclid(12) as u32 + 1;
        (norm_year, norm_month)
    }

    pub(super) fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
        let adjusted_year = year - if month <= 2 { 1 } else { 0 };
        let era = adjusted_year.div_euclid(400);
        let yoe = adjusted_year - era * 400;
        let month = i64::from(month);
        let day = i64::from(day);
        let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    pub(super) fn civil_from_days(days: i64) -> (i64, u32, u32) {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096).div_euclid(365);
        let mut year = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2).div_euclid(153);
        let day = (doy - (153 * mp + 2).div_euclid(5) + 1) as u32;
        let month = (mp + if mp < 10 { 3 } else { -9 }) as u32;
        if month <= 2 {
            year += 1;
        }
        (year, month, day)
    }

    pub(super) fn days_in_month(year: i64, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    pub(super) fn is_leap_year(year: i64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    pub(super) fn numeric_value(&self, value: &Value) -> f64 {
        match value {
            Value::Number(v) => *v as f64,
            Value::Float(v) => *v,
            Value::BigInt(v) => v.to_f64().unwrap_or_else(|| {
                if v.sign() == Sign::Minus {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                }
            }),
            Value::Date(v) => *v.borrow() as f64,
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            _ => value.as_string().parse::<f64>().unwrap_or(0.0),
        }
    }

    pub(super) fn coerce_number_for_global(value: &Value) -> f64 {
        match value {
            Value::Number(v) => *v as f64,
            Value::Float(v) => *v,
            Value::BigInt(v) => v.to_f64().unwrap_or_else(|| {
                if v.sign() == Sign::Minus {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                }
            }),
            Value::Bool(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            Value::String(v) => Self::parse_js_number_from_string(v),
            Value::Date(v) => *v.borrow() as f64,
            Value::Object(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Symbol(_)
            | Value::RegExp(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_) => f64::NAN,
            Value::Array(values) => {
                let rendered = Value::Array(values.clone()).as_string();
                Self::parse_js_number_from_string(&rendered)
            }
        }
    }

    pub(super) fn to_i32_for_bitwise(&self, value: &Value) -> i32 {
        let numeric = self.numeric_value(value);
        if !numeric.is_finite() {
            return 0;
        }
        let unsigned = numeric.trunc().rem_euclid(4_294_967_296.0);
        if unsigned >= 2_147_483_648.0 {
            (unsigned - 4_294_967_296.0) as i32
        } else {
            unsigned as i32
        }
    }

    pub(super) fn to_u32_for_bitwise(&self, value: &Value) -> u32 {
        let numeric = self.numeric_value(value);
        if !numeric.is_finite() {
            return 0;
        }
        numeric.trunc().rem_euclid(4_294_967_296.0) as u32
    }

    pub(super) fn resolve_dom_query_var_path_value(
        &self,
        base: &str,
        path: &[String],
        env: &HashMap<String, Value>,
    ) -> Result<Option<Value>> {
        let Some(mut value) = env.get(base).cloned() else {
            return Err(Error::ScriptRuntime(format!(
                "unknown element variable: {}",
                base
            )));
        };

        for key in path {
            let next = match self.object_property_from_value(&value, key) {
                Ok(next) => next,
                Err(_) => return Ok(None),
            };
            if matches!(next, Value::Null | Value::Undefined) {
                return Ok(None);
            }
            value = next;
        }

        Ok(Some(value))
    }

    pub(super) fn resolve_dom_query_list_static(
        &mut self,
        target: &DomQuery,
    ) -> Result<Option<Vec<NodeId>>> {
        match target {
            DomQuery::BySelectorAll { selector } => {
                Ok(Some(self.dom.query_selector_all(selector)?))
            }
            DomQuery::QuerySelectorAll { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                Ok(Some(
                    self.dom.query_selector_all_from(&target_node, selector)?,
                ))
            }
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_static(target)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, None)?;
                Ok(list.get(index).copied().map(|node| vec![node]))
            }
            DomQuery::BySelectorAllIndex { selector, index } => {
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                Ok(self
                    .dom
                    .query_selector_all(selector)?
                    .get(index)
                    .copied()
                    .map(|node| vec![node]))
            }
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                let list = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(list.get(index).copied().map(|node| vec![node]))
            }
            DomQuery::Var(_) | DomQuery::VarPath { .. } => Err(Error::ScriptRuntime(
                "element variable cannot be resolved in static context".into(),
            )),
            _ => Ok(None),
        }
    }

    pub(super) fn resolve_dom_query_list_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<Option<Vec<NodeId>>> {
        match target {
            DomQuery::Var(name) => match env.get(name) {
                Some(Value::NodeList(nodes)) => Ok(Some(nodes.clone())),
                Some(Value::Node(_)) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a node list",
                    name
                ))),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a node list",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown element variable: {}",
                    name
                ))),
            },
            DomQuery::VarPath { base, path } => {
                let Some(value) = self.resolve_dom_query_var_path_value(base, path, env)? else {
                    return Ok(None);
                };
                match value {
                    Value::NodeList(nodes) => Ok(Some(nodes)),
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a node list",
                        target.describe_call()
                    ))),
                }
            }
            DomQuery::QuerySelectorAll {
                target: query_target,
                selector,
            } => {
                let Some(target_node) = self.resolve_dom_query_runtime(query_target, env)? else {
                    return Ok(None);
                };
                Ok(Some(
                    self.dom.query_selector_all_from(&target_node, selector)?,
                ))
            }
            DomQuery::QuerySelectorAllIndex {
                target: query_target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_runtime(query_target, env)? else {
                    return Ok(None);
                };
                let all = self.dom.query_selector_all_from(&target_node, selector)?;
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                Ok(all.get(index).copied().map(|node| vec![node]))
            }
            _ => self.resolve_dom_query_list_static(target),
        }
    }

    pub(super) fn resolve_dom_query_static(&mut self, target: &DomQuery) -> Result<Option<NodeId>> {
        match target {
            DomQuery::DocumentRoot => Ok(Some(self.dom.root)),
            DomQuery::DocumentBody => Ok(self.dom.body()),
            DomQuery::DocumentHead => Ok(self.dom.head()),
            DomQuery::DocumentElement => Ok(self.dom.document_element()),
            DomQuery::ById(id) => Ok(self.dom.by_id(id)),
            DomQuery::BySelector(selector) => self.dom.query_selector(selector),
            DomQuery::BySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::BySelectorAllIndex { selector, index } => {
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                let all = self.dom.query_selector_all(selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_static(target)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, None)?;
                Ok(list.get(index).copied())
            }
            DomQuery::QuerySelector { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                self.dom.query_selector_from(&target_node, selector)
            }
            DomQuery::QuerySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                let all = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::FormElementsIndex { form, index } => {
                let Some(form_node) = self.resolve_dom_query_static(form)? else {
                    return Ok(None);
                };
                let all = self.form_elements(form_node)?;
                let index = self.resolve_runtime_dom_index(index, None)?;
                Ok(all.get(index).copied())
            }
            DomQuery::Var(_) | DomQuery::VarPath { .. } => Err(Error::ScriptRuntime(
                "element variable cannot be resolved in static context".into(),
            )),
        }
    }

    pub(super) fn resolve_dom_query_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<Option<NodeId>> {
        match target {
            DomQuery::DocumentRoot => Ok(Some(self.dom.root)),
            DomQuery::DocumentBody => Ok(self.dom.body()),
            DomQuery::DocumentHead => Ok(self.dom.head()),
            DomQuery::DocumentElement => Ok(self.dom.document_element()),
            DomQuery::Var(name) => match env.get(name) {
                Some(Value::Node(node)) => Ok(Some(*node)),
                Some(Value::NodeList(_)) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a single element",
                    name
                ))),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a single element",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown element variable: {}",
                    name
                ))),
            },
            DomQuery::VarPath { base, path } => {
                let Some(value) = self.resolve_dom_query_var_path_value(base, path, env)? else {
                    return Ok(None);
                };
                match value {
                    Value::Node(node) => Ok(Some(node)),
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a single element",
                        target.describe_call()
                    ))),
                }
            }
            DomQuery::BySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::QuerySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_runtime(target, env)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                Ok(list.get(index).copied())
            }
            DomQuery::QuerySelector { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_runtime(target, env)? else {
                    return Ok(None);
                };
                self.dom.query_selector_from(&target_node, selector)
            }
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_runtime(target, env)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                let all = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::FormElementsIndex { form, index } => {
                let Some(form_node) = self.resolve_dom_query_runtime(form, env)? else {
                    return Ok(None);
                };
                let all = self.form_elements(form_node)?;
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                Ok(all.get(index).copied())
            }
            _ => self.resolve_dom_query_static(target),
        }
    }

    pub(super) fn resolve_dom_query_required_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<NodeId> {
        self.resolve_dom_query_runtime(target, env)?.ok_or_else(|| {
            Error::ScriptRuntime(format!("{} returned null", target.describe_call()))
        })
    }

    pub(super) fn resolve_runtime_dom_index(
        &mut self,
        index: &DomIndex,
        env: Option<&HashMap<String, Value>>,
    ) -> Result<usize> {
        match index {
            DomIndex::Static(index) => Ok(*index),
            DomIndex::Dynamic(expr_src) => {
                let expr = parse_expr(expr_src)?;
                let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                let value = self.eval_expr(
                    &expr,
                    env.ok_or_else(|| {
                        Error::ScriptRuntime("dynamic index requires runtime context".into())
                    })?,
                    &None,
                    &event,
                )?;
                self.value_as_index(&value).ok_or_else(|| {
                    Error::ScriptRuntime(format!("invalid index expression: {expr_src}"))
                })
            }
        }
    }

    pub(super) fn describe_dom_prop(&self, prop: &DomProp) -> String {
        match prop {
            DomProp::Attributes => "attributes".into(),
            DomProp::AssignedSlot => "assignedSlot".into(),
            DomProp::Value => "value".into(),
            DomProp::ValueLength => "value.length".into(),
            DomProp::ValidationMessage => "validationMessage".into(),
            DomProp::Validity => "validity".into(),
            DomProp::ValidityValueMissing => "validity.valueMissing".into(),
            DomProp::ValidityTypeMismatch => "validity.typeMismatch".into(),
            DomProp::ValidityPatternMismatch => "validity.patternMismatch".into(),
            DomProp::ValidityTooLong => "validity.tooLong".into(),
            DomProp::ValidityTooShort => "validity.tooShort".into(),
            DomProp::ValidityRangeUnderflow => "validity.rangeUnderflow".into(),
            DomProp::ValidityRangeOverflow => "validity.rangeOverflow".into(),
            DomProp::ValidityStepMismatch => "validity.stepMismatch".into(),
            DomProp::ValidityBadInput => "validity.badInput".into(),
            DomProp::ValidityValid => "validity.valid".into(),
            DomProp::ValidityCustomError => "validity.customError".into(),
            DomProp::SelectionStart => "selectionStart".into(),
            DomProp::SelectionEnd => "selectionEnd".into(),
            DomProp::SelectionDirection => "selectionDirection".into(),
            DomProp::Checked => "checked".into(),
            DomProp::Indeterminate => "indeterminate".into(),
            DomProp::Open => "open".into(),
            DomProp::ReturnValue => "returnValue".into(),
            DomProp::ClosedBy => "closedBy".into(),
            DomProp::Readonly => "readonly".into(),
            DomProp::Required => "required".into(),
            DomProp::Disabled => "disabled".into(),
            DomProp::TextContent => "textContent".into(),
            DomProp::InnerText => "innerText".into(),
            DomProp::InnerHtml => "innerHTML".into(),
            DomProp::OuterHtml => "outerHTML".into(),
            DomProp::ClassName => "className".into(),
            DomProp::ClassList => "classList".into(),
            DomProp::ClassListLength => "classList.length".into(),
            DomProp::Part => "part".into(),
            DomProp::PartLength => "part.length".into(),
            DomProp::Id => "id".into(),
            DomProp::TagName => "tagName".into(),
            DomProp::LocalName => "localName".into(),
            DomProp::NamespaceUri => "namespaceURI".into(),
            DomProp::Prefix => "prefix".into(),
            DomProp::NextElementSibling => "nextElementSibling".into(),
            DomProp::PreviousElementSibling => "previousElementSibling".into(),
            DomProp::Slot => "slot".into(),
            DomProp::Role => "role".into(),
            DomProp::ElementTiming => "elementTiming".into(),
            DomProp::Name => "name".into(),
            DomProp::Lang => "lang".into(),
            DomProp::ClientWidth => "clientWidth".into(),
            DomProp::ClientHeight => "clientHeight".into(),
            DomProp::ClientLeft => "clientLeft".into(),
            DomProp::ClientTop => "clientTop".into(),
            DomProp::CurrentCssZoom => "currentCSSZoom".into(),
            DomProp::OffsetWidth => "offsetWidth".into(),
            DomProp::OffsetHeight => "offsetHeight".into(),
            DomProp::OffsetLeft => "offsetLeft".into(),
            DomProp::OffsetTop => "offsetTop".into(),
            DomProp::ScrollWidth => "scrollWidth".into(),
            DomProp::ScrollHeight => "scrollHeight".into(),
            DomProp::ScrollLeft => "scrollLeft".into(),
            DomProp::ScrollTop => "scrollTop".into(),
            DomProp::ScrollLeftMax => "scrollLeftMax".into(),
            DomProp::ScrollTopMax => "scrollTopMax".into(),
            DomProp::ShadowRoot => "shadowRoot".into(),
            DomProp::Dataset(_) => "dataset".into(),
            DomProp::Style(_) => "style".into(),
            DomProp::AriaString(prop_name) => prop_name.clone(),
            DomProp::AriaElementRefSingle(prop_name) => prop_name.clone(),
            DomProp::AriaElementRefList(prop_name) => prop_name.clone(),
            DomProp::ActiveElement => "activeElement".into(),
            DomProp::CharacterSet => "characterSet".into(),
            DomProp::CompatMode => "compatMode".into(),
            DomProp::ContentType => "contentType".into(),
            DomProp::ReadyState => "readyState".into(),
            DomProp::Referrer => "referrer".into(),
            DomProp::Title => "title".into(),
            DomProp::Url => "URL".into(),
            DomProp::DocumentUri => "documentURI".into(),
            DomProp::Location => "location".into(),
            DomProp::LocationHref => "location.href".into(),
            DomProp::LocationProtocol => "location.protocol".into(),
            DomProp::LocationHost => "location.host".into(),
            DomProp::LocationHostname => "location.hostname".into(),
            DomProp::LocationPort => "location.port".into(),
            DomProp::LocationPathname => "location.pathname".into(),
            DomProp::LocationSearch => "location.search".into(),
            DomProp::LocationHash => "location.hash".into(),
            DomProp::LocationOrigin => "location.origin".into(),
            DomProp::LocationAncestorOrigins => "location.ancestorOrigins".into(),
            DomProp::History => "history".into(),
            DomProp::HistoryLength => "history.length".into(),
            DomProp::HistoryState => "history.state".into(),
            DomProp::HistoryScrollRestoration => "history.scrollRestoration".into(),
            DomProp::DefaultView => "defaultView".into(),
            DomProp::Hidden => "hidden".into(),
            DomProp::VisibilityState => "visibilityState".into(),
            DomProp::Forms => "forms".into(),
            DomProp::Images => "images".into(),
            DomProp::Links => "links".into(),
            DomProp::Scripts => "scripts".into(),
            DomProp::Children => "children".into(),
            DomProp::ChildElementCount => "childElementCount".into(),
            DomProp::FirstElementChild => "firstElementChild".into(),
            DomProp::LastElementChild => "lastElementChild".into(),
            DomProp::CurrentScript => "currentScript".into(),
            DomProp::FormsLength => "forms.length".into(),
            DomProp::ImagesLength => "images.length".into(),
            DomProp::LinksLength => "links.length".into(),
            DomProp::ScriptsLength => "scripts.length".into(),
            DomProp::ChildrenLength => "children.length".into(),
            DomProp::AnchorAttributionSrc => "attributionSrc".into(),
            DomProp::AnchorDownload => "download".into(),
            DomProp::AnchorHash => "hash".into(),
            DomProp::AnchorHost => "host".into(),
            DomProp::AnchorHostname => "hostname".into(),
            DomProp::AnchorHref => "href".into(),
            DomProp::AnchorHreflang => "hreflang".into(),
            DomProp::AnchorInterestForElement => "interestForElement".into(),
            DomProp::AnchorOrigin => "origin".into(),
            DomProp::AnchorPassword => "password".into(),
            DomProp::AnchorPathname => "pathname".into(),
            DomProp::AnchorPing => "ping".into(),
            DomProp::AnchorPort => "port".into(),
            DomProp::AnchorProtocol => "protocol".into(),
            DomProp::AnchorReferrerPolicy => "referrerPolicy".into(),
            DomProp::AnchorRel => "rel".into(),
            DomProp::AnchorRelList => "relList".into(),
            DomProp::AnchorRelListLength => "relList.length".into(),
            DomProp::AnchorSearch => "search".into(),
            DomProp::AnchorTarget => "target".into(),
            DomProp::AnchorText => "text".into(),
            DomProp::AnchorType => "type".into(),
            DomProp::AnchorUsername => "username".into(),
            DomProp::AnchorCharset => "charset".into(),
            DomProp::AnchorCoords => "coords".into(),
            DomProp::AnchorRev => "rev".into(),
            DomProp::AnchorShape => "shape".into(),
        }
    }

    pub(super) fn event_node_label(&self, node: NodeId) -> String {
        if let Some(id) = self.dom.attr(node, "id") {
            if !id.is_empty() {
                return id;
            }
        }
        self.dom
            .tag_name(node)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("node-{}", node.0))
    }

    pub(super) fn trace_node_label(&self, node: NodeId) -> String {
        if let Some(id) = self.dom.attr(node, "id") {
            if !id.is_empty() {
                return format!("#{id}");
            }
        }
        self.dom
            .tag_name(node)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("node-{}", node.0))
    }

    pub(super) fn value_to_i64(value: &Value) -> i64 {
        match value {
            Value::Number(v) => *v,
            Value::Float(v) => *v as i64,
            Value::BigInt(v) => v.to_i64().unwrap_or_else(|| {
                if v.sign() == Sign::Minus {
                    i64::MIN
                } else {
                    i64::MAX
                }
            }),
            Value::Bool(v) => {
                if *v {
                    1
                } else {
                    0
                }
            }
            Value::String(v) => v
                .parse::<i64>()
                .ok()
                .or_else(|| v.parse::<f64>().ok().map(|n| n as i64))
                .unwrap_or(0),
            Value::Array(values) => Value::Array(values.clone())
                .as_string()
                .parse::<i64>()
                .ok()
                .or_else(|| {
                    Value::Array(values.clone())
                        .as_string()
                        .parse::<f64>()
                        .ok()
                        .map(|n| n as i64)
                })
                .unwrap_or(0),
            Value::Date(value) => *value.borrow(),
            Value::Object(_) => 0,
            Value::Promise(_) => 0,
            Value::Map(_) => 0,
            Value::Set(_) => 0,
            Value::Blob(_) => 0,
            Value::ArrayBuffer(_) => 0,
            Value::TypedArray(_) => 0,
            Value::StringConstructor => 0,
            Value::TypedArrayConstructor(_) => 0,
            Value::BlobConstructor => 0,
            Value::UrlConstructor => 0,
            Value::ArrayBufferConstructor => 0,
            Value::PromiseConstructor => 0,
            Value::MapConstructor => 0,
            Value::SetConstructor => 0,
            Value::SymbolConstructor => 0,
            Value::RegExpConstructor => 0,
            Value::PromiseCapability(_) => 0,
            Value::Symbol(_) => 0,
            Value::RegExp(_) => 0,
            Value::Node(_) => 0,
            Value::NodeList(_) => 0,
            Value::FormData(_) => 0,
            Value::Function(_) => 0,
            Value::Null => 0,
            Value::Undefined => 0,
        }
    }

    pub(super) fn next_random_f64(&mut self) -> f64 {
        // xorshift64*: simple deterministic PRNG for test runtime.
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = if x == 0 { 0xA5A5_A5A5_A5A5_A5A5 } else { x };
        let out = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        // Convert top 53 bits to [0.0, 1.0).
        let mantissa = out >> 11;
        (mantissa as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    pub(super) fn schedule_timeout(
        &mut self,
        callback: TimerCallback,
        delay_ms: i64,
        callback_args: Vec<Value>,
        env: &HashMap<String, Value>,
    ) -> i64 {
        let delay_ms = delay_ms.max(0);
        let due_at = self.scheduler.now_ms.saturating_add(delay_ms);
        let id = self.scheduler.allocate_timer_id();
        let order = self.scheduler.allocate_task_order();
        self.scheduler.task_queue.push(ScheduledTask {
            id,
            due_at,
            order,
            interval_ms: None,
            callback,
            callback_args,
            env: ScriptEnv::from_snapshot(env),
        });
        self.trace_timer_line(format!(
            "[timer] schedule timeout id={} due_at={} delay_ms={}",
            id, due_at, delay_ms
        ));
        id
    }

    pub(super) fn schedule_interval(
        &mut self,
        callback: TimerCallback,
        interval_ms: i64,
        callback_args: Vec<Value>,
        env: &HashMap<String, Value>,
    ) -> i64 {
        let interval_ms = interval_ms.max(0);
        let due_at = self.scheduler.now_ms.saturating_add(interval_ms);
        let id = self.scheduler.allocate_timer_id();
        let order = self.scheduler.allocate_task_order();
        self.scheduler.task_queue.push(ScheduledTask {
            id,
            due_at,
            order,
            interval_ms: Some(interval_ms),
            callback,
            callback_args,
            env: ScriptEnv::from_snapshot(env),
        });
        self.trace_timer_line(format!(
            "[timer] schedule interval id={} due_at={} interval_ms={}",
            id, due_at, interval_ms
        ));
        id
    }

    pub(super) fn clear_timeout(&mut self, id: i64) {
        let before = self.scheduler.task_queue.len();
        self.scheduler.task_queue.retain(|task| task.id != id);
        let removed = before.saturating_sub(self.scheduler.task_queue.len());
        let mut running_canceled = false;
        if self.scheduler.running_timer_id == Some(id) {
            self.scheduler.running_timer_canceled = true;
            running_canceled = true;
        }
        self.trace_timer_line(format!(
            "[timer] clear id={} removed={} running_canceled={}",
            id, removed, running_canceled
        ));
    }

    pub(super) fn compile_and_register_script(&mut self, script: &str) -> Result<()> {
        let stmts = parse_block_statements(script)?;
        self.with_script_env(|this, env| {
            let mut event = EventState::new("script", this.dom.root, this.scheduler.now_ms);
            this.run_in_task_context(|inner| {
                inner
                    .execute_stmts(&stmts, &None, &mut event, env)
                    .map(|_| ())
            })
        })?;

        Ok(())
    }
}
