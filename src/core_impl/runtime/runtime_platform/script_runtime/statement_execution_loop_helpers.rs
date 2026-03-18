use super::*;

impl Harness {
    pub(crate) fn is_iteration_stmt(stmt: &Stmt) -> bool {
        matches!(
            stmt,
            Stmt::For { .. }
                | Stmt::ForIn { .. }
                | Stmt::ForOf { .. }
                | Stmt::ForAwaitOf { .. }
                | Stmt::While { .. }
                | Stmt::DoWhile { .. }
        )
    }

    pub(crate) fn await_value_in_for_await(&mut self, value: Value) -> Result<Value> {
        let promise = self.promise_resolve_value_as_promise(value)?;
        loop {
            let settled = {
                let promise = promise.borrow();
                match &promise.state {
                    PromiseState::Pending => None,
                    PromiseState::Fulfilled(value) => Some(Ok(value.clone())),
                    PromiseState::Rejected(reason) => Some(Err(reason.clone())),
                }
            };
            match settled {
                Some(Ok(value)) => return Ok(value),
                Some(Err(reason)) => return Err(Error::ScriptThrown(ThrownValue::new(reason))),
                None => {
                    if !self.scheduler.microtask_queue.is_empty() {
                        self.run_microtask_queue()?;
                        continue;
                    }
                    let ran_timers = self.run_due_timers_internal()?;
                    if ran_timers == 0 {
                        return Ok(Value::Undefined);
                    }
                }
            }
        }
    }

    fn for_in_integer_key(key: &str) -> Option<u64> {
        if key.is_empty() || !key.as_bytes().iter().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let value = key.parse::<u64>().ok()?;
        if value.to_string() == key {
            Some(value)
        } else {
            None
        }
    }

    fn ordered_for_in_own_string_keys(entries: &ObjectValue) -> Vec<String> {
        let mut integer_keys: Vec<(u64, String)> = Vec::new();
        let mut string_keys: Vec<String> = Vec::new();
        for (key, _) in entries.iter() {
            if !Self::is_enumerable_object_key(entries, key) {
                continue;
            }
            if let Some(index) = Self::for_in_integer_key(key) {
                integer_keys.push((index, key.clone()));
            } else {
                string_keys.push(key.clone());
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = Vec::with_capacity(integer_keys.len() + string_keys.len());
        out.extend(integer_keys.into_iter().map(|(_, key)| key));
        out.extend(string_keys);
        out
    }

    pub(crate) fn collect_for_in_object_chain_keys(
        &self,
        object: &Rc<RefCell<ObjectValue>>,
    ) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut out = Vec::new();
        let mut current = Some(object.clone());
        while let Some(target) = current {
            let (keys, next) = {
                let entries = target.borrow();
                let keys = Self::ordered_for_in_own_string_keys(&entries);
                let next = match Self::object_get_entry(&entries, INTERNAL_OBJECT_PROTOTYPE_KEY) {
                    Some(Value::Object(next)) => Some(next),
                    _ => None,
                };
                (keys, next)
            };
            for key in keys {
                if visited.insert(key.clone()) {
                    out.push(key);
                }
            }
            current = next;
        }
        out
    }

    pub(crate) fn collect_for_in_array_keys(&self, array: &Rc<RefCell<ArrayValue>>) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut out = Vec::new();
        let (length, own_props, prototype) = {
            let array = array.borrow();
            let own_props = Self::ordered_for_in_own_string_keys(&array.properties);
            let prototype =
                match Self::object_get_entry(&array.properties, INTERNAL_OBJECT_PROTOTYPE_KEY) {
                    Some(Value::Object(next)) => Some(next),
                    _ => None,
                };
            (array.elements.len(), own_props, prototype)
        };
        for index in 0..length {
            let key = index.to_string();
            if visited.insert(key.clone()) {
                out.push(key);
            }
        }
        for key in own_props {
            if visited.insert(key.clone()) {
                out.push(key);
            }
        }
        let mut current = prototype;
        while let Some(target) = current {
            let (keys, next) = {
                let entries = target.borrow();
                let keys = Self::ordered_for_in_own_string_keys(&entries);
                let next = match Self::object_get_entry(&entries, INTERNAL_OBJECT_PROTOTYPE_KEY) {
                    Some(Value::Object(next)) => Some(next),
                    _ => None,
                };
                (keys, next)
            };
            for key in keys {
                if visited.insert(key.clone()) {
                    out.push(key);
                }
            }
            current = next;
        }
        out
    }

    pub(crate) fn for_of_symbol_iterator_factory_result(
        &mut self,
        iterable: &Rc<RefCell<ObjectValue>>,
        event: &EventState,
    ) -> Result<Option<Rc<RefCell<ObjectValue>>>> {
        let iterator_symbol = self.eval_symbol_static_property(SymbolStaticProperty::Iterator);
        let iterator_key = self.property_key_to_storage_key(&iterator_symbol);
        let iterable_value = Value::Object(iterable.clone());
        let iterator_factory = self.object_property_from_value(&iterable_value, &iterator_key)?;
        if matches!(iterator_factory, Value::Undefined | Value::Null) {
            return Ok(None);
        }
        if !self.is_callable_value(&iterator_factory) {
            return Err(Error::ScriptRuntime(
                "for...of iterator factory is not callable".into(),
            ));
        }
        let iterator_value = self.execute_callable_value_with_this_and_env(
            &iterator_factory,
            &[],
            event,
            None,
            Some(iterable_value),
        )?;
        let Value::Object(iterator) = iterator_value else {
            return Err(Error::ScriptRuntime(
                "for...of iterator factory must return an object".into(),
            ));
        };
        Ok(Some(iterator))
    }

    pub(crate) fn for_of_protocol_iterator_next(
        &mut self,
        iterator: &Rc<RefCell<ObjectValue>>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let iterator_value = Value::Object(iterator.clone());
        let next_method = self.object_property_from_value(&iterator_value, "next")?;
        if !self.is_callable_value(&next_method) {
            return Err(Error::ScriptRuntime(
                "for...of iterator next is not callable".into(),
            ));
        }
        let result = self.execute_callable_value_with_this_and_env(
            &next_method,
            &[],
            event,
            None,
            Some(iterator_value),
        )?;
        let Value::Object(result_obj) = result else {
            return Err(Error::ScriptRuntime(
                "for...of iterator.next must return an object".into(),
            ));
        };
        let result_value = Value::Object(result_obj.clone());
        let done = self
            .object_property_from_value(&result_value, "done")?
            .truthy();
        if done {
            return Ok(None);
        }
        let value = self.object_property_from_value(&result_value, "value")?;
        Ok(Some(value))
    }

    pub(crate) fn for_of_protocol_iterator_close(
        &mut self,
        iterator: &Rc<RefCell<ObjectValue>>,
        event: &EventState,
    ) -> Result<()> {
        let iterator_value = Value::Object(iterator.clone());
        let return_method = self.object_property_from_value(&iterator_value, "return")?;
        if matches!(return_method, Value::Undefined | Value::Null) {
            return Ok(());
        }
        if !self.is_callable_value(&return_method) {
            return Err(Error::ScriptRuntime(
                "for...of iterator.return is not callable".into(),
            ));
        }
        let _ = self.execute_callable_value_with_this_and_env(
            &return_method,
            &[],
            event,
            None,
            Some(iterator_value),
        )?;
        Ok(())
    }

    pub(crate) fn for_of_internal_iterator_close_if_needed(
        &mut self,
        iterator: &Rc<RefCell<ObjectValue>>,
        event: &EventState,
    ) -> Result<()> {
        let _ = self.eval_iterator_member_call(iterator, "return", &[], event)?;
        Ok(())
    }

    pub(crate) fn take_pending_loop_labels(&mut self) -> Vec<String> {
        self.script_runtime
            .pending_loop_labels
            .pop()
            .unwrap_or_default()
    }

    pub(crate) fn push_loop_label_scope(&mut self, labels: Vec<String>) {
        self.script_runtime
            .loop_label_stack
            .push(labels.into_iter().collect());
    }

    pub(crate) fn pop_loop_label_scope(&mut self) {
        self.script_runtime.loop_label_stack.pop();
    }

    fn current_loop_has_label(&self, label: &str) -> bool {
        self.script_runtime
            .loop_label_stack
            .last()
            .is_some_and(|labels| labels.contains(label))
    }

    pub(crate) fn loop_should_consume_break(&self, label: &Option<String>) -> bool {
        match label {
            None => true,
            Some(label) => self.current_loop_has_label(label),
        }
    }

    pub(crate) fn loop_should_consume_continue(&self, label: &Option<String>) -> bool {
        match label {
            None => true,
            Some(label) => self.current_loop_has_label(label),
        }
    }

    pub(crate) fn break_flow_error(label: &Option<String>) -> Error {
        if let Some(label) = label {
            Error::ScriptRuntime(format!("label not found: {label}"))
        } else {
            Error::ScriptRuntime("break statement outside of loop".into())
        }
    }

    pub(crate) fn continue_flow_error(label: &Option<String>) -> Error {
        if let Some(label) = label {
            Error::ScriptRuntime(format!("label not found: {label}"))
        } else {
            Error::ScriptRuntime("continue statement outside of loop".into())
        }
    }

    pub(crate) fn with_isolated_loop_control_scope<T>(
        &mut self,
        run: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let previous_pending = std::mem::take(&mut self.script_runtime.pending_loop_labels);
        let previous_labels = std::mem::take(&mut self.script_runtime.loop_label_stack);
        let result = run(self);
        self.script_runtime.pending_loop_labels = previous_pending;
        self.script_runtime.loop_label_stack = previous_labels;
        result
    }
}
