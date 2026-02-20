impl Harness {
    pub(crate) fn promise_error_reason(err: Error) -> Value {
        Value::String(format!("{err}"))
    }

    pub(crate) fn new_pending_promise(&mut self) -> Rc<RefCell<PromiseValue>> {
        let id = self.promise_runtime.allocate_promise_id();
        Rc::new(RefCell::new(PromiseValue {
            id,
            state: PromiseState::Pending,
            reactions: Vec::new(),
        }))
    }

    pub(crate) fn new_promise_capability_functions(
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

    pub(crate) fn promise_add_reaction(
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

    pub(crate) fn promise_fulfill(&mut self, promise: &Rc<RefCell<PromiseValue>>, value: Value) {
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

    pub(crate) fn promise_reject(&mut self, promise: &Rc<RefCell<PromiseValue>>, reason: Value) {
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

    pub(crate) fn promise_resolve(
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

    pub(crate) fn promise_resolve_value_as_promise(
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

    pub(crate) fn promise_then_internal(
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

    pub(crate) fn new_aggregate_error_value(reasons: Vec<Value>) -> Value {
        Self::new_object_value(vec![
            ("name".into(), Value::String("AggregateError".into())),
            (
                "message".into(),
                Value::String("All promises were rejected".into()),
            ),
            ("errors".into(), Self::new_array_value(reasons)),
        ])
    }

    pub(crate) fn run_promise_reaction_task(
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

}
