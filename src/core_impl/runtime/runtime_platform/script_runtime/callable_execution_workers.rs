use super::*;

impl Harness {
    pub(crate) fn worker_target_from_callable(
        callable: &Value,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let Value::Object(entries) = callable else {
            return Err(Error::ScriptRuntime(
                "Worker callable has invalid internal state".into(),
            ));
        };
        let entries = entries.borrow();
        match Self::object_get_entry(&entries, INTERNAL_WORKER_TARGET_KEY) {
            Some(Value::Object(worker)) => Ok(worker),
            _ => Err(Error::ScriptRuntime(
                "Worker callable has invalid internal state".into(),
            )),
        }
    }

    pub(crate) fn worker_global_from_object(
        worker: &Rc<RefCell<ObjectValue>>,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let entries = worker.borrow();
        match Self::object_get_entry(&entries, INTERNAL_WORKER_GLOBAL_OBJECT_KEY) {
            Some(Value::Object(global)) => Ok(global),
            _ => Err(Error::ScriptRuntime(
                "Worker instance has invalid internal state".into(),
            )),
        }
    }

    pub(crate) fn worker_constructor_bindings(&mut self) -> Vec<(String, Value)> {
        let boolean_constructor = self
            .script_runtime
            .env
            .get("Boolean")
            .cloned()
            .unwrap_or_else(Self::new_boolean_constructor_callable);
        let number_constructor = self
            .script_runtime
            .env
            .get("Number")
            .cloned()
            .unwrap_or_else(Self::new_number_constructor_callable);
        let bigint_constructor = self
            .script_runtime
            .env
            .get("BigInt")
            .cloned()
            .unwrap_or_else(Self::new_bigint_constructor_callable);
        let object_constructor = self
            .script_runtime
            .env
            .get("Object")
            .cloned()
            .unwrap_or_else(Self::new_object_constructor_value);
        let reflect_object = self
            .script_runtime
            .env
            .get("Reflect")
            .cloned()
            .unwrap_or_else(|| self.new_reflect_object_value());
        let mut bindings = Self::shared_core_constructor_bindings(
            &Value::StringConstructor,
            &boolean_constructor,
            &number_constructor,
            &bigint_constructor,
            &Value::SymbolConstructor,
            &object_constructor,
            &reflect_object,
        );
        bindings.extend(self.function_family_constructor_bindings());
        if let Some(intl) = self.script_runtime.env.get("Intl").cloned() {
            bindings.push(("Intl".to_string(), intl));
        }
        bindings
    }

    pub(crate) fn worker_is_terminated_object(worker: &Rc<RefCell<ObjectValue>>) -> bool {
        let entries = worker.borrow();
        matches!(
            Self::object_get_entry(&entries, INTERNAL_WORKER_TERMINATED_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn worker_set_terminated_object(
        worker: &Rc<RefCell<ObjectValue>>,
        terminated: bool,
    ) {
        Self::object_set_entry(
            &mut worker.borrow_mut(),
            INTERNAL_WORKER_TERMINATED_KEY.to_string(),
            Value::Bool(terminated),
        );
    }

    pub(crate) fn resolve_worker_script_source(&self, script_url: &str) -> Result<String> {
        let url = script_url.trim();
        if url.is_empty() {
            return Err(Error::ScriptRuntime(
                "Worker constructor requires a non-empty script URL".into(),
            ));
        }
        if let Some(blob) = self.browser_apis.blob_url_objects.get(url) {
            return Ok(String::from_utf8_lossy(&blob.borrow().bytes).into_owned());
        }

        let resolved = Self::resolve_url_string(url, Some(&self.document_url))
            .unwrap_or_else(|| url.to_string());
        self.platform_mocks
            .fetch_mocks
            .get(&resolved)
            .or_else(|| self.platform_mocks.fetch_mocks.get(url))
            .map(|mock| mock.body.clone())
            .ok_or_else(|| {
                Error::ScriptRuntime(format!("Worker script source not found: {script_url}"))
            })
    }

    pub(crate) fn worker_function_id_from_source(source: &str) -> Option<usize> {
        fn parse_marker(value: &str) -> Option<usize> {
            let marker = value
                .strip_prefix("__bt_function_ref__(")?
                .strip_suffix(')')?;
            marker.trim().parse::<usize>().ok()
        }

        let trimmed = source.trim();
        if let Some(id) = parse_marker(trimmed) {
            return Some(id);
        }
        let wrapped = trimmed.strip_prefix('(')?.strip_suffix(")()")?;
        parse_marker(wrapped.trim())
    }

    pub(crate) fn execute_worker_stmts(
        &mut self,
        stmts: &[Stmt],
        worker: &Value,
        worker_global: &Value,
    ) -> Result<()> {
        let worker_post_message = Self::new_worker_context_post_message_callable(worker.clone());
        let mut worker_env = HashMap::new();
        worker_env.insert("self".to_string(), worker_global.clone());
        worker_env.insert("globalThis".to_string(), worker_global.clone());
        worker_env.insert("postMessage".to_string(), worker_post_message.clone());
        worker_env.insert("onmessage".to_string(), Value::Null);
        if let Value::Object(worker_global_entries) = worker_global {
            let entries = worker_global_entries.borrow();
            for (name, value) in &entries.entries {
                if name == INTERNAL_WORKER_OBJECT_KEY
                    || name == "self"
                    || name == "globalThis"
                    || name == "postMessage"
                    || name == "onmessage"
                {
                    continue;
                }
                worker_env.insert(name.clone(), value.clone());
            }
        }
        worker_env.insert(INTERNAL_SCOPE_DEPTH_KEY.to_string(), Value::Number(1));

        let mut worker_event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
        self.run_in_task_context(|inner| {
            inner
                .execute_stmts(stmts, &None, &mut worker_event, &mut worker_env)
                .map(|_| ())
        })?;

        if let Some(onmessage) = worker_env.get("onmessage").cloned() {
            if matches!(onmessage, Value::Null | Value::Undefined) {
                return Ok(());
            }
            let Value::Object(worker_global_entries) = worker_global else {
                return Err(Error::ScriptRuntime(
                    "Worker global has invalid internal state".into(),
                ));
            };
            Self::object_set_entry(
                &mut worker_global_entries.borrow_mut(),
                "onmessage".to_string(),
                onmessage,
            );
        }
        Ok(())
    }

    pub(crate) fn execute_worker_script_source(
        &mut self,
        source: &str,
        worker: &Value,
        worker_global: &Value,
    ) -> Result<()> {
        if let Some(function_id) = Self::worker_function_id_from_source(source) {
            let function = self
                .script_runtime
                .function_registry
                .get(&function_id)
                .cloned()
                .ok_or_else(|| {
                    Error::ScriptRuntime(format!(
                        "Worker script function reference is not available: {function_id}"
                    ))
                })?;
            return self.execute_worker_stmts(&function.handler.stmts, worker, worker_global);
        }

        let stmts = parse_block_statements(source)?;
        self.execute_worker_stmts(&stmts, worker, worker_global)
    }

    pub(crate) fn new_worker_instance_from_script_source(
        &mut self,
        source: &str,
    ) -> Result<Value> {
        let worker = Self::new_object_value(vec![
            (INTERNAL_WORKER_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_WORKER_TERMINATED_KEY.to_string(),
                Value::Bool(false),
            ),
            ("onmessage".to_string(), Value::Null),
        ]);

        let worker_global_entries = Rc::new(RefCell::new(ObjectValue::default()));
        let worker_global = Value::Object(worker_global_entries.clone());

        let worker_context_post_message =
            Self::new_worker_context_post_message_callable(worker.clone());
        {
            let mut entries = worker_global_entries.borrow_mut();
            Self::object_set_entry(
                &mut entries,
                INTERNAL_WORKER_OBJECT_KEY.to_string(),
                Value::Bool(true),
            );
            Self::object_set_entry(&mut entries, "self".to_string(), worker_global.clone());
            Self::object_set_entry(
                &mut entries,
                "globalThis".to_string(),
                worker_global.clone(),
            );
            Self::object_set_entry(
                &mut entries,
                "postMessage".to_string(),
                worker_context_post_message,
            );
            Self::object_set_entry(&mut entries, "onmessage".to_string(), Value::Null);
            for (name, value) in self.worker_constructor_bindings() {
                Self::object_set_entry(&mut entries, name, value);
            }
        }

        if let Value::Object(worker_entries) = &worker {
            let mut entries = worker_entries.borrow_mut();
            Self::object_set_entry(
                &mut entries,
                INTERNAL_WORKER_GLOBAL_OBJECT_KEY.to_string(),
                worker_global.clone(),
            );
            Self::object_set_entry(
                &mut entries,
                "postMessage".to_string(),
                Self::new_worker_main_post_message_callable(worker.clone()),
            );
            Self::object_set_entry(
                &mut entries,
                "terminate".to_string(),
                Self::new_worker_terminate_callable(worker.clone()),
            );
            let prototype = self
                .script_runtime
                .env
                .get("Worker")
                .cloned()
                .and_then(|constructor| self.constructor_prototype_from_value(&constructor))
                .or_else(|| {
                    self.script_runtime
                        .env
                        .get("Object")
                        .cloned()
                        .and_then(|constructor| self.constructor_prototype_from_value(&constructor))
                })
                .unwrap_or_else(|| Self::new_object_value(Vec::new()));
            Self::object_set_entry(
                &mut entries,
                INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                prototype,
            );
        }

        self.execute_worker_script_source(source, &worker, &worker_global)?;
        Ok(worker)
    }

    pub(crate) fn dispatch_worker_message_to_onmessage(
        &mut self,
        target: &Rc<RefCell<ObjectValue>>,
        target_this: Value,
        data: Value,
        event: &EventState,
    ) -> Result<()> {
        let handler = {
            let entries = target.borrow();
            Self::object_get_entry(&entries, "onmessage")
        };
        let Some(handler) = handler else {
            return Ok(());
        };
        if matches!(handler, Value::Null | Value::Undefined) {
            return Ok(());
        }
        if !self.is_callable_value(&handler) {
            return Err(Error::ScriptRuntime(
                "Worker.onmessage is not a function".into(),
            ));
        }
        let event_object = Self::new_object_value(vec![("data".to_string(), data)]);
        let _ = self.execute_callable_value_with_this_and_env(
            &handler,
            &[event_object],
            event,
            None,
            Some(target_this),
        )?;
        Ok(())
    }
}
