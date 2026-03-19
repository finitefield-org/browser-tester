use super::*;

impl Harness {
    pub(crate) fn promise_error_reason(err: Error) -> Value {
        match err {
            Error::ScriptThrown(thrown) => thrown.into_value(),
            Error::ScriptRuntime(message) => Value::String(message),
            other => Value::String(format!("{other}")),
        }
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
