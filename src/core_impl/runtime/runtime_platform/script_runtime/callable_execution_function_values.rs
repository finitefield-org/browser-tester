use super::*;

impl Harness {
    pub(crate) fn make_function_value_with_kind(
        &mut self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        is_async: bool,
        is_generator: bool,
        is_arrow: bool,
        is_method: bool,
        is_class_constructor: bool,
        class_super_constructor: Option<Value>,
        class_super_prototype: Option<Value>,
    ) -> Value {
        let local_bindings = Self::collect_function_scope_bindings(&handler);
        let scope_depth = Self::env_scope_depth(env);
        let captured_pending_function_decls = self.script_runtime.pending_function_decls.clone();
        let mut captured_names = if global_scope {
            HashSet::new()
        } else if is_class_constructor {
            env.keys()
                .filter(|name| {
                    !Self::is_internal_env_key(name) && name.as_str() != INTERNAL_RETURN_SLOT
                })
                .cloned()
                .collect()
        } else {
            let mut capture_names = Self::collect_function_capture_names(&handler);
            self.expand_capture_names_with_pending_function_decls(env, &mut capture_names);
            capture_names
        };
        if !is_arrow {
            captured_names.remove("this");
            captured_names.remove("arguments");
        }
        let mut captured_snapshot = if global_scope {
            HashMap::new()
        } else if is_class_constructor {
            let mut captured_snapshot = env.clone();
            self.project_pending_listener_capture_env_updates(&mut captured_snapshot);
            captured_snapshot
        } else {
            self.selective_function_capture_snapshot(env, &captured_names)
        };
        let mut env_local_bindings = match env.get(INTERNAL_LOCAL_BINDINGS_KEY) {
            Some(Value::Array(bindings)) => bindings
                .borrow()
                .iter()
                .filter_map(|entry| match entry {
                    Value::String(name) => Some(name.clone()),
                    _ => None,
                })
                .collect::<HashSet<_>>(),
            _ => HashSet::new(),
        };
        env_local_bindings.extend(Self::env_top_level_lexical_binding_names(env));
        let mut captured_global_names = HashSet::new();
        for (name, value) in &captured_snapshot {
            if Self::is_internal_env_key(name) || name == INTERNAL_RETURN_SLOT {
                continue;
            }
            if env_local_bindings.contains(name) {
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
        if !global_scope {
            if !is_arrow {
                captured_snapshot.remove("this");
                captured_snapshot.remove("arguments");
            }
            for name in &local_bindings {
                captured_snapshot.remove(name);
            }
            for name in &captured_global_names {
                captured_snapshot.remove(name);
            }
            captured_names.retain(|name| {
                !local_bindings.contains(name) && !captured_global_names.contains(name)
            });
        }
        let missing_capture_names = captured_names
            .iter()
            .filter(|name| {
                !captured_snapshot.contains_key(*name) && !self.has_pending_function_decl(name)
            })
            .cloned()
            .collect::<Vec<_>>();
        let captured_env = if global_scope {
            Rc::new(RefCell::new(self.script_runtime.env.share()))
        } else if captured_names.is_empty() {
            Rc::new(RefCell::new(if captured_snapshot.is_empty() {
                ScriptEnv::default()
            } else {
                ScriptEnv::from_snapshot(&captured_snapshot)
            }))
        } else {
            let captured_env = self.ensure_listener_capture_env();
            {
                let mut shared_env = captured_env.borrow_mut();
                for (name, value) in captured_snapshot {
                    shared_env.insert(name, value);
                }
            }
            for name in missing_capture_names {
                self.queue_listener_capture_env_update_for_shared_env(&captured_env, name, None);
            }
            captured_env
        };
        let function_id = self.script_runtime.allocate_function_id();
        if !self.script_runtime.private_binding_stack.is_empty() {
            let mut captured_private_bindings = HashMap::new();
            for bindings in &self.script_runtime.private_binding_stack {
                for (name, binding) in bindings {
                    captured_private_bindings.insert(name.clone(), binding.clone());
                }
            }
            self.script_runtime
                .function_private_bindings
                .insert(function_id, captured_private_bindings);
        }
        let function = Rc::new(FunctionValue {
            function_id,
            handler,
            expression_name: None,
            captured_env,
            captured_pending_function_decls,
            captured_global_names,
            captured_names,
            local_bindings,
            prototype_object: Rc::new(RefCell::new(ObjectValue::default())),
            global_scope,
            is_async,
            is_generator,
            is_arrow,
            is_method,
            is_class_constructor,
            class_super_constructor,
            class_super_prototype,
        });
        self.sync_function_prototype_object(&function);
        self.script_runtime
            .function_registry
            .insert(function_id, function.clone());
        Value::Function(function)
    }

    pub(crate) fn make_function_value(
        &mut self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        is_async: bool,
        is_generator: bool,
        is_arrow: bool,
        is_method: bool,
    ) -> Value {
        self.make_function_value_with_kind(
            handler,
            env,
            global_scope,
            is_async,
            is_generator,
            is_arrow,
            is_method,
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
        is_arrow: bool,
        is_method: bool,
        class_super_constructor: Option<Value>,
        class_super_prototype: Option<Value>,
    ) -> Value {
        self.make_function_value_with_kind(
            handler,
            env,
            global_scope,
            is_async,
            is_generator,
            is_arrow,
            is_method,
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
            Value::Function(_)
                | Value::PromiseCapability(_)
                | Value::StringConstructor
                | Value::RegExpConstructor
                | Value::TypedArrayConstructor(_)
                | Value::BlobConstructor
                | Value::UrlConstructor
                | Value::ArrayBufferConstructor
                | Value::PromiseConstructor
                | Value::MapConstructor
                | Value::WeakMapConstructor
                | Value::SetConstructor
                | Value::WeakSetConstructor
                | Value::UrlSearchParamsConstructor
                | Value::SymbolConstructor
        ) || Self::callable_kind_from_value(value).is_some()
    }
}
