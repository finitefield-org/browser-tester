use super::*;

impl Harness {
    pub(crate) fn dispatch_event_target_with_expr_env_sync(
        &mut self,
        target_object: Rc<RefCell<ObjectValue>>,
        event_payload: Value,
        env: &HashMap<String, Value>,
    ) -> Result<EventState> {
        let mut dispatch_env = env.clone();
        self.project_pending_listener_capture_env_updates(&mut dispatch_env);
        let dispatched =
            self.dispatch_event_target_with_env(target_object, event_payload, &mut dispatch_env)?;

        let mut names = env
            .keys()
            .filter(|name| !Self::is_internal_env_key(name))
            .cloned()
            .collect::<HashSet<_>>();
        names.extend(
            dispatch_env
                .keys()
                .filter(|name| !Self::is_internal_env_key(name))
                .cloned(),
        );
        let mut changed = Vec::new();
        for name in names {
            let before = env.get(&name);
            let after = dispatch_env.get(&name);
            let changed_value = match (before, after) {
                (Some(prev), Some(next)) => !self.strict_equal(prev, next),
                (None, Some(_)) => true,
                (Some(_), None) => true,
                (None, None) => false,
            };
            if changed_value {
                changed.push((name, after.cloned()));
            }
        }
        for (name, value) in changed {
            self.queue_event_sync_pending_update(env, &name, value);
        }

        Ok(dispatched)
    }

    pub(crate) fn execute_callable_value_with_env_and_sync(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
        env: &HashMap<String, Value>,
    ) -> Result<Value> {
        self.sync_listener_capture_env_if_shared(env);
        let result = self.execute_callable_value_with_env(callable, args, event, Some(env))?;
        self.sync_listener_capture_env_if_shared(env);
        Ok(result)
    }

    pub(crate) fn execute_callable_value_with_this_and_env_and_sync(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
        env: &HashMap<String, Value>,
        this_arg: Option<Value>,
    ) -> Result<Value> {
        self.sync_listener_capture_env_if_shared(env);
        let result = self.execute_callable_value_with_this_and_env(
            callable,
            args,
            event,
            Some(env),
            this_arg,
        )?;
        self.sync_listener_capture_env_if_shared(env);
        Ok(result)
    }

    pub(crate) fn eval_index_get_call_target_and_this(
        &mut self,
        target: &Expr,
        index: &Expr,
        optional: bool,
        optional_call: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<(Value, Value)>> {
        let is_super = Self::is_super_target_expr(target);
        let receiver = if is_super {
            Self::super_prototype_from_env(env)?
        } else {
            self.eval_expr(target, env, event_param, event)?
        };
        if optional && matches!(receiver, Value::Null | Value::Undefined) {
            return Ok(None);
        }

        let index_value = self.eval_expr(index, env, event_param, event)?;
        let key = match index_value {
            Value::Number(value) => value.to_string(),
            Value::BigInt(value) => value.to_string(),
            Value::Float(value) if value.is_finite() && value.fract() == 0.0 => {
                format!("{value:.0}")
            }
            other => self.property_key_to_storage_key(&other),
        };

        let this_arg = if is_super {
            Self::super_this_from_env(env)?
        } else {
            receiver.clone()
        };
        let callee = if is_super {
            self.object_property_from_value_with_receiver(&receiver, &key, &this_arg)?
        } else {
            self.object_property_from_value(&receiver, &key)?
        };
        if optional_call && matches!(callee, Value::Null | Value::Undefined) {
            return Ok(None);
        }
        Ok(Some((callee, this_arg)))
    }

    pub(crate) fn eval_array_index_call_target_and_this(
        &mut self,
        target: &str,
        index: &Expr,
        optional_call: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<(Value, Value)>> {
        let index_value = self.eval_expr(index, env, event_param, event)?;
        let key = match &index_value {
            Value::Number(value) => value.to_string(),
            Value::BigInt(value) => value.to_string(),
            Value::Float(value) if value.is_finite() && value.fract() == 0.0 => {
                format!("{value:.0}")
            }
            other => self.property_key_to_storage_key(other),
        };

        let (receiver, this_arg) = if target == "super" {
            let this_value = Self::super_this_from_env(env)?;
            (Self::super_prototype_from_env(env)?, this_value)
        } else {
            let receiver = self
                .resolve_target_value_with_pending(env, target)
                .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {target}")))?;
            (receiver.clone(), receiver)
        };
        let callee = if target == "super" {
            self.object_property_from_value_with_receiver(&receiver, &key, &this_arg)?
        } else {
            self.object_property_from_value(&receiver, &key)?
        };
        if optional_call && matches!(callee, Value::Null | Value::Undefined) {
            return Ok(None);
        }
        Ok(Some((callee, this_arg)))
    }

    pub(crate) fn eval_form_data_member_call_from_values(
        &mut self,
        entries: &Rc<RefCell<Vec<(String, String)>>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match member {
            "append" => {
                if evaluated_args.len() < 2 {
                    return Err(Error::ScriptRuntime(
                        "FormData.append requires two or three arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let value =
                    Self::form_data_append_string_value(&evaluated_args[1], evaluated_args.get(2));
                entries.borrow_mut().push((name, value));
                Value::Undefined
            }
            "set" => {
                if evaluated_args.len() < 2 {
                    return Err(Error::ScriptRuntime(
                        "FormData.set requires two or three arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let value =
                    Self::form_data_append_string_value(&evaluated_args[1], evaluated_args.get(2));
                let mut entries_ref = entries.borrow_mut();
                if let Some(first_match) = entries_ref
                    .iter()
                    .position(|(entry_name, _)| entry_name == &name)
                {
                    entries_ref[first_match].1 = value;
                    let mut index = entries_ref.len();
                    while index > 0 {
                        index -= 1;
                        if index != first_match && entries_ref[index].0 == name {
                            entries_ref.remove(index);
                        }
                    }
                } else {
                    entries_ref.push((name, value));
                }
                Value::Undefined
            }
            "delete" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "FormData.delete requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                entries
                    .borrow_mut()
                    .retain(|(entry_name, _)| entry_name != &name);
                Value::Undefined
            }
            "entries" => {
                let snapshot = entries.borrow().clone();
                Self::new_array_value(
                    snapshot
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>(),
                )
            }
            "keys" => {
                let snapshot = entries.borrow().clone();
                Self::new_array_value(
                    snapshot
                        .into_iter()
                        .map(|(name, _)| Value::String(name))
                        .collect::<Vec<_>>(),
                )
            }
            "values" => {
                let snapshot = entries.borrow().clone();
                Self::new_array_value(
                    snapshot
                        .into_iter()
                        .map(|(_, value)| Value::String(value))
                        .collect::<Vec<_>>(),
                )
            }
            "get" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "FormData.get requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let entries = entries.borrow();
                entries
                    .iter()
                    .find_map(|(entry_name, value)| {
                        (entry_name == &name).then(|| Value::String(value.clone()))
                    })
                    .unwrap_or(Value::Null)
            }
            "getAll" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "FormData.getAll requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let snapshot = entries.borrow().clone();
                Self::new_array_value(
                    snapshot
                        .into_iter()
                        .filter_map(|(entry_name, value)| {
                            (entry_name == name).then(|| Value::String(value))
                        })
                        .collect::<Vec<_>>(),
                )
            }
            "has" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "FormData.has requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let has = entries
                    .borrow()
                    .iter()
                    .any(|(entry_name, _)| entry_name == &name);
                Value::Bool(has)
            }
            "forEach" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "FormData.forEach requires a callback and optional thisArg".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = entries.borrow().clone();
                for (name, value) in snapshot {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[
                            Value::String(value),
                            Value::String(name),
                            Value::FormData(entries.clone()),
                        ],
                        event,
                    )?;
                }
                Value::Undefined
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    pub(crate) fn resolve_listener_capture_pending_value_from(
        &self,
        start: usize,
        name: &str,
    ) -> Option<Option<Value>> {
        let start = start.min(self.script_runtime.listener_capture_env_stack.len());
        for frame in self.script_runtime.listener_capture_env_stack[start..]
            .iter()
            .rev()
        {
            if let Some(value) = frame.pending_env_updates.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    pub(crate) fn resolve_listener_capture_pending_value(
        &self,
        name: &str,
    ) -> Option<Option<Value>> {
        self.resolve_listener_capture_pending_value_from(0, name)
    }

    fn current_dynamic_import_referrer(&self) -> String {
        self.script_runtime
            .module_referrer_stack
            .last()
            .cloned()
            .unwrap_or_else(|| self.document_url.clone())
    }

    fn module_namespace_for_dynamic_import(
        &mut self,
        specifier: &str,
        attribute_type: Option<&str>,
        referrer: &str,
    ) -> Result<Value> {
        let cache_key = self.resolve_module_specifier_key(specifier, referrer);
        if let Some(cached) = self.script_runtime.module_namespace_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let exports = self.load_module_exports(specifier, attribute_type, referrer)?;
        let mut entries = exports.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(left, _), (right, _)| left.cmp(right));
        let namespace = Self::new_object_value(entries);
        self.script_runtime
            .module_namespace_cache
            .insert(cache_key, namespace.clone());
        Ok(namespace)
    }

    fn dynamic_import_attribute_type_from_options_value(
        &self,
        options: &Value,
    ) -> Result<Option<String>> {
        let Value::Object(options_entries) = options else {
            return Ok(None);
        };
        let with_value = {
            let options_entries = options_entries.borrow();
            Self::object_get_entry(&options_entries, "with")
        };
        let Some(with_value) = with_value else {
            return Ok(None);
        };
        if matches!(with_value, Value::Null | Value::Undefined) {
            return Ok(None);
        }
        let Value::Object(with_entries) = with_value else {
            return Err(Error::ScriptRuntime(
                "import() options.with must be an object".into(),
            ));
        };

        let with_entries = with_entries.borrow();
        let mut attribute_type = None;
        for (key, value) in with_entries.iter() {
            if key.starts_with('\0') {
                continue;
            }
            match key.as_str() {
                "type" => {
                    let value = value.as_string();
                    if value != "json" {
                        return Err(Error::ScriptRuntime(
                            "unsupported import attribute: type".into(),
                        ));
                    }
                    attribute_type = Some(value);
                }
                _ => {
                    return Err(Error::ScriptRuntime(format!(
                        "unsupported import attribute: {key}"
                    )));
                }
            }
        }

        Ok(attribute_type)
    }

    fn object_assign_is_copyable_key(key: &str) -> bool {
        Self::is_symbol_storage_key(key) || !Self::is_internal_object_key(key)
    }

    fn object_assign_accessor_property_key(key: &str) -> Option<&str> {
        key.strip_prefix(INTERNAL_OBJECT_GETTER_KEY_PREFIX)
            .or_else(|| key.strip_prefix(INTERNAL_OBJECT_SETTER_KEY_PREFIX))
            .or_else(|| key.strip_prefix(INTERNAL_OBJECT_UNDEFINED_GETTER_KEY_PREFIX))
            .or_else(|| key.strip_prefix(INTERNAL_OBJECT_UNDEFINED_SETTER_KEY_PREFIX))
    }

    fn object_assign_enumerable_keys(&mut self, value: &Value) -> Vec<String> {
        match value {
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(mut keys) = Self::string_wrapper_own_string_keys(&entries, true) {
                    keys.extend(
                        entries
                            .iter()
                            .filter(|(key, _)| {
                                Self::is_symbol_storage_key(key)
                                    && !Self::is_non_enumerable_object_key(&*entries, key)
                            })
                            .map(|(key, _)| key.clone()),
                    );
                    return keys;
                }
                if let Some(mut keys) = self.class_list_synthesized_keys(&entries, true) {
                    keys.extend(
                        entries
                            .iter()
                            .filter(|(key, _)| {
                                Self::is_symbol_storage_key(key)
                                    && !Self::is_non_enumerable_object_key(&*entries, key)
                            })
                            .map(|(key, _)| key.clone()),
                    );
                    return keys;
                }
                if let Some(mut keys) = self.named_node_map_synthesized_keys(&entries, true) {
                    keys.extend(
                        entries
                            .iter()
                            .filter(|(key, _)| {
                                Self::is_symbol_storage_key(key)
                                    && !Self::is_non_enumerable_object_key(&*entries, key)
                            })
                            .map(|(key, _)| key.clone()),
                    );
                    return keys;
                }
                if let Some(mut keys) = self.dom_string_map_synthesized_keys(&entries, true) {
                    keys.extend(
                        entries
                            .iter()
                            .filter(|(key, _)| {
                                Self::is_symbol_storage_key(key)
                                    && !Self::is_non_enumerable_object_key(&*entries, key)
                            })
                            .map(|(key, _)| key.clone()),
                    );
                    return keys;
                }
                let mut keys = entries
                    .iter()
                    .filter(|(key, _)| {
                        Self::object_assign_is_copyable_key(key)
                            && !Self::is_non_enumerable_object_key(&*entries, key)
                    })
                    .map(|(key, _)| key.clone())
                    .collect::<Vec<_>>();
                let mut seen = keys.iter().cloned().collect::<HashSet<_>>();
                for (key, _) in entries.iter() {
                    let Some(property_key) = Self::object_assign_accessor_property_key(key) else {
                        continue;
                    };
                    if Self::is_non_enumerable_object_key(&*entries, property_key) {
                        continue;
                    }
                    if seen.insert(property_key.to_string()) {
                        keys.push(property_key.to_string());
                    }
                }
                keys
            }
            Value::Array(values) => {
                let values = values.borrow();
                let mut keys = values
                    .iter()
                    .enumerate()
                    .filter_map(|(index, _)| {
                        (!Self::array_index_is_hole(&values, index)).then(|| index.to_string())
                    })
                    .collect::<Vec<_>>();
                keys.extend(
                    values
                        .properties
                        .iter()
                        .filter(|(key, _)| {
                            Self::object_assign_is_copyable_key(key)
                                && !Self::is_non_enumerable_object_key(&values.properties, key)
                        })
                        .map(|(key, _)| key.clone()),
                );
                let mut seen = keys.iter().cloned().collect::<HashSet<_>>();
                for (key, _) in values.properties.iter() {
                    let Some(property_key) = Self::object_assign_accessor_property_key(key) else {
                        continue;
                    };
                    if Self::is_non_enumerable_object_key(&values.properties, property_key) {
                        continue;
                    }
                    if seen.insert(property_key.to_string()) {
                        keys.push(property_key.to_string());
                    }
                }
                keys
            }
            Value::Node(node) => {
                let mut keys = self.node_expando_enumerable_string_keys(*node);
                keys.extend(
                    self.node_expando_enumerable_symbol_values(*node)
                        .into_iter()
                        .filter_map(|value| match value {
                            Value::Symbol(symbol) => Some(Self::symbol_storage_key(symbol.id)),
                            _ => None,
                        }),
                );
                keys
            }
            Value::NodeList(nodes) => {
                let mut keys = self.node_list_synthesized_keys(nodes, true);
                let nodes_ref = nodes.borrow();
                keys.extend(
                    nodes_ref
                        .properties
                        .iter()
                        .filter(|(key, _)| {
                            Self::is_symbol_storage_key(key)
                                && !Self::is_non_enumerable_object_key(&nodes_ref.properties, key)
                        })
                        .map(|(key, _)| key.clone()),
                );
                let mut seen = keys.iter().cloned().collect::<HashSet<_>>();
                for (key, _) in nodes_ref.properties.iter() {
                    let Some(property_key) = Self::object_assign_accessor_property_key(key) else {
                        continue;
                    };
                    if Self::is_non_enumerable_object_key(&nodes_ref.properties, property_key) {
                        continue;
                    }
                    if seen.insert(property_key.to_string()) {
                        keys.push(property_key.to_string());
                    }
                }
                keys
            }
            Value::String(text) => text
                .chars()
                .enumerate()
                .map(|(index, _)| index.to_string())
                .collect(),
            _ => Vec::new(),
        }
    }

    fn object_assign_target_to_object(target: Value) -> Result<Value> {
        match target {
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(
                "Cannot convert undefined or null to object".into(),
            )),
            Value::Object(_)
            | Value::Function(_)
            | Value::Array(_)
            | Value::Map(_)
            | Value::WeakMap(_)
            | Value::Set(_)
            | Value::WeakSet(_)
            | Value::RegExp(_)
            | Value::Node(_)
            | Value::UrlConstructor => Ok(target),
            primitive => Ok(Self::box_primitive_value(primitive)),
        }
    }

    fn object_assign_set_target_property(
        &mut self,
        target: &Value,
        key: &str,
        value: Value,
        event: &EventState,
    ) -> Result<()> {
        let key_value = if let Some(symbol_id) = Self::symbol_id_from_storage_key(key) {
            if let Some(symbol) = self.symbol_runtime.symbols_by_id.get(&symbol_id) {
                Value::Symbol(symbol.clone())
            } else {
                Value::String(key.to_string())
            }
        } else {
            Value::String(key.to_string())
        };
        let mut assign_env = HashMap::new();
        self.set_object_assignment_property(
            target,
            &key_value,
            value,
            "Object.assign target",
            &mut assign_env,
            event,
        )
        .map_err(|err| match err {
            Error::ScriptRuntime(msg)
                if msg
                    == "variable 'Object.assign target' is not an object (assignment target)" =>
            {
                Error::ScriptRuntime("Object.assign target must be an object".into())
            }
            other => other,
        })
    }

    pub(crate) fn eval_object_assign_static_call(
        &mut self,
        args: &[Value],
        event: &EventState,
    ) -> Result<Value> {
        if args.is_empty() {
            return Err(Error::ScriptRuntime(
                "Object.assign requires at least one argument".into(),
            ));
        }
        let target = Self::object_assign_target_to_object(args[0].clone())?;

        for source in args.iter().skip(1) {
            if matches!(source, Value::Null | Value::Undefined) {
                continue;
            }
            for key in self.object_assign_enumerable_keys(source) {
                let value = self.object_property_from_value(source, &key)?;
                self.object_assign_set_target_property(&target, &key, value, event)?;
            }
        }

        Ok(target)
    }

    pub(crate) fn eval_import_call(
        &mut self,
        module: &Expr,
        options: &Option<Box<Expr>>,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Value {
        let promise = self.new_pending_promise();
        let result = (|| -> Result<Value> {
            let specifier = self.eval_expr(module, env, event_param, event)?.as_string();
            let attribute_type = if let Some(options_expr) = options {
                let options_value = self.eval_expr(options_expr, env, event_param, event)?;
                self.dynamic_import_attribute_type_from_options_value(&options_value)?
            } else {
                None
            };

            let referrer = self.current_dynamic_import_referrer();
            self.module_namespace_for_dynamic_import(
                &specifier,
                attribute_type.as_deref(),
                &referrer,
            )
        })();

        match result {
            Ok(namespace) => {
                if let Err(err) = self.promise_resolve(&promise, namespace) {
                    self.promise_reject(&promise, Self::promise_error_reason(err));
                }
            }
            Err(err) => {
                self.promise_reject(&promise, Self::promise_error_reason(err));
            }
        }

        Value::Promise(promise)
    }

    fn current_import_meta_referrer(&self) -> Result<String> {
        self.script_runtime
            .module_referrer_stack
            .last()
            .cloned()
            .ok_or_else(|| {
                Error::ScriptRuntime("import.meta may only be used in module scripts".into())
            })
    }

    pub(crate) fn eval_import_meta_object(&self) -> Result<Value> {
        let referrer = self.current_import_meta_referrer()?;
        Ok(Self::new_object_value(vec![
            (
                INTERNAL_IMPORT_META_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("url".to_string(), Value::String(referrer)),
            (
                "resolve".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ]))
    }

    pub(crate) fn eval_import_meta_resolve_call(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(Error::ScriptRuntime(
                "import.meta.resolve requires exactly one argument".into(),
            ));
        }
        let referrer = self.current_import_meta_referrer()?;
        let specifier = args[0].as_string();
        let resolved =
            Self::resolve_url_string(&specifier, Some(&referrer)).unwrap_or_else(|| specifier);
        Ok(Value::String(resolved))
    }

    pub(crate) fn eval_new_target_value(&self, env: &HashMap<String, Value>) -> Result<Value> {
        env.get(INTERNAL_NEW_TARGET_KEY).cloned().ok_or_else(|| {
            Error::ScriptRuntime("new.target is only valid in function or class bodies".into())
        })
    }

    pub(crate) fn eval_expr_calls_timers_binary(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        if let Some(value) = self.try_eval_call_like_exprs(expr, env, event_param, event)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.try_eval_member_call_exprs(expr, env, event_param, event)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.try_eval_access_like_exprs(expr, env, event_param, event)? {
            return Ok(Some(value));
        }
        if let Some(value) =
            self.try_eval_scheduler_and_binary_exprs(expr, env, event_param, event)?
        {
            return Ok(Some(value));
        }
        Ok(None)
    }
}
