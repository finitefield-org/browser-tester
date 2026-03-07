use super::*;

impl Harness {
    fn execute_callable_value_with_env_and_sync(
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

    fn execute_callable_value_with_this_and_env_and_sync(
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

    pub(crate) fn resolve_listener_capture_pending_value(
        &self,
        name: &str,
    ) -> Option<Option<Value>> {
        for frame in self.script_runtime.listener_capture_env_stack.iter().rev() {
            if let Some(value) = frame.pending_env_updates.get(name) {
                return Some(value.clone());
            }
        }
        None
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

    fn object_assign_enumerable_keys(value: &Value) -> Vec<String> {
        match value {
            Value::Object(entries) => entries
                .borrow()
                .iter()
                .filter(|(key, _)| Self::object_assign_is_copyable_key(key))
                .map(|(key, _)| key.clone())
                .collect(),
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
                        .filter(|(key, _)| Self::object_assign_is_copyable_key(key))
                        .map(|(key, _)| key.clone()),
                );
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
            Value::String(text) => Ok(Self::new_string_wrapper_value(text)),
            Value::Symbol(symbol) => Ok(Self::new_object_value(vec![(
                INTERNAL_SYMBOL_WRAPPER_KEY.to_string(),
                Value::Number(symbol.id as i64),
            )])),
            primitive => Ok(Self::new_object_value(vec![(
                "value".to_string(),
                Value::String(primitive.as_string()),
            )])),
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

    fn eval_object_assign_static_call(
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
            for key in Self::object_assign_enumerable_keys(source) {
                let value = self.object_property_from_value(source, &key)?;
                self.object_assign_set_target_property(&target, &key, value, event)?;
            }
        }

        Ok(target)
    }

    fn eval_import_call(
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

    fn eval_import_meta_object(&self) -> Result<Value> {
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

    fn eval_import_meta_resolve_call(&self, args: &[Value]) -> Result<Value> {
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

    fn eval_new_target_value(&self, env: &HashMap<String, Value>) -> Result<Value> {
        env.get(INTERNAL_NEW_TARGET_KEY).cloned().ok_or_else(|| {
            Error::ScriptRuntime("new.target is only valid in function or class bodies".into())
        })
    }

    fn is_super_target_expr(expr: &Expr) -> bool {
        matches!(expr, Expr::Var(name) if name == "super")
    }

    fn super_constructor_from_env(env: &HashMap<String, Value>) -> Result<Value> {
        env.get(INTERNAL_CLASS_SUPER_CONSTRUCTOR_KEY)
            .cloned()
            .ok_or_else(|| {
                Error::ScriptRuntime("super() is only valid in a derived class constructor".into())
            })
    }

    pub(crate) fn super_prototype_from_env(env: &HashMap<String, Value>) -> Result<Value> {
        env.get(INTERNAL_CLASS_SUPER_PROTOTYPE_KEY)
            .cloned()
            .ok_or_else(|| {
                Error::ScriptRuntime("super property access is only valid in a class method".into())
            })
    }

    pub(crate) fn super_this_from_env(env: &HashMap<String, Value>) -> Result<Value> {
        match env.get("this").cloned().unwrap_or(Value::Undefined) {
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(
                "super requires an initialized this value".into(),
            )),
            value => Ok(value),
        }
    }

    pub(crate) fn eval_expr_calls_timers_binary(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::FunctionCall { target, args } => {
                    if target == "super" {
                        let super_constructor = Self::super_constructor_from_env(env)?;
                        let this_value = Self::super_this_from_env(env)?;
                        let evaluated_args =
                            self.eval_call_args_with_spread(args, env, event_param, event)?;
                        let super_result = self.execute_constructor_value_with_this_and_env(
                            &super_constructor,
                            &evaluated_args,
                            event,
                            Some(env),
                            Some(this_value),
                        )?;
                        self.initialize_current_constructor_instance_fields(
                            env,
                            event_param,
                            event,
                        )?;
                        return Ok(super_result);
                    }
                    self.ensure_binding_initialized(env, target)?;
                    let callee = if let Some(pending) =
                        self.resolve_listener_capture_pending_value(target)
                    {
                        let Some(callee) = pending else {
                            return Err(Error::ScriptRuntime(format!(
                                "unknown variable: {target}"
                            )));
                        };
                        callee
                    } else if let Some(callee) = env.get(target).cloned() {
                        callee
                    } else if let Some(callee) = self.resolve_pending_function_decl(target, env) {
                        callee
                    } else {
                        return Err(Error::ScriptRuntime(format!("unknown variable: {target}")));
                    };
                    let evaluated_args =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    self.execute_callable_value_with_env_and_sync(
                        &callee,
                        &evaluated_args,
                        event,
                        env,
                    )
                    .map_err(|err| match err {
                        Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                            Error::ScriptRuntime(format!("'{target}' is not a function"))
                        }
                        other => other,
                    })
                }
                Expr::ImportCall { module, options } => {
                    Ok(self.eval_import_call(module, options, env, event_param, event))
                }
                Expr::Call {
                    target,
                    args,
                    optional,
                } => {
                    let callee = self.eval_expr(target, env, event_param, event)?;
                    if *optional && matches!(callee, Value::Null | Value::Undefined) {
                        return Ok(Value::Undefined);
                    }
                    let evaluated_args =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    self.execute_callable_value_with_env_and_sync(
                        &callee,
                        &evaluated_args,
                        event,
                        env,
                    )
                    .map_err(|err| match err {
                        Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                            Error::ScriptRuntime("call target is not a function".into())
                        }
                        other => other,
                    })
                }
                Expr::MemberCall {
                    target,
                    member,
                    args,
                    optional,
                    optional_call,
                } => {
                    if Self::is_super_target_expr(target) {
                        let super_prototype = Self::super_prototype_from_env(env)?;
                        let this_value = Self::super_this_from_env(env)?;
                        let evaluated_args =
                            self.eval_call_args_with_spread(args, env, event_param, event)?;
                        let callee = self.object_property_from_value_with_receiver(
                            &super_prototype,
                            member,
                            &this_value,
                        )?;
                        return self
                            .execute_callable_value_with_this_and_env_and_sync(
                                &callee,
                                &evaluated_args,
                                event,
                                env,
                                Some(this_value),
                            )
                            .map_err(|err| match err {
                                Error::ScriptRuntime(msg)
                                    if msg == "callback is not a function" =>
                                {
                                    Error::ScriptRuntime(format!("'{}' is not a function", member))
                                }
                                other => other,
                            });
                    }

                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    if *optional && matches!(receiver, Value::Null | Value::Undefined) {
                        return Ok(Value::Undefined);
                    }
                    if *optional_call {
                        let callee =
                            self.object_property_from_value(&receiver, member)
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "value is not an object" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "member call target does not support property '{}'",
                                            member
                                        ))
                                    }
                                    other => other,
                                })?;
                        if matches!(callee, Value::Null | Value::Undefined) {
                            return Ok(Value::Undefined);
                        }
                        let evaluated_args =
                            self.eval_call_args_with_spread(args, env, event_param, event)?;
                        return self
                            .execute_callable_value_with_this_and_env_and_sync(
                                &callee,
                                &evaluated_args,
                                event,
                                env,
                                Some(receiver.clone()),
                            )
                            .map_err(|err| match err {
                                Error::ScriptRuntime(msg)
                                    if msg == "callback is not a function" =>
                                {
                                    Error::ScriptRuntime(format!("'{}' is not a function", member))
                                }
                                other => other,
                            });
                    }
                    let evaluated_args =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;

                    if let Value::FormData(entries) = &receiver {
                        if let Some(value) = self.eval_form_data_member_call_from_values(
                            entries,
                            member,
                            &evaluated_args,
                            event,
                        )? {
                            return Ok(value);
                        }
                    }

                    if member == "dispatchEvent" {
                        if evaluated_args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "dispatchEvent requires exactly one argument".into(),
                            ));
                        }
                        let event_payload = evaluated_args[0].clone();
                        if let Value::Node(node) = &receiver {
                            let dispatched =
                                self.dispatch_dom_event_payload(*node, event_payload)?;
                            return Ok(Value::Bool(!dispatched.default_prevented));
                        }
                        if let Value::Object(object) = &receiver {
                            let (is_document_object, is_event_target_object) = {
                                let entries = object.borrow();
                                (
                                    matches!(
                                        Self::object_get_entry(
                                            &entries,
                                            INTERNAL_DOCUMENT_OBJECT_KEY
                                        ),
                                        Some(Value::Bool(true))
                                    ),
                                    Self::is_event_target_object(&entries),
                                )
                            };
                            if is_document_object {
                                let dispatched =
                                    self.dispatch_dom_event_payload(self.dom.root, event_payload)?;
                                return Ok(Value::Bool(!dispatched.default_prevented));
                            }
                            if is_event_target_object {
                                let dispatched =
                                    self.dispatch_event_target(object.clone(), event_payload)?;
                                return Ok(Value::Bool(!dispatched.default_prevented));
                            }
                        }
                    }

                    if matches!(member.as_str(), "call" | "apply" | "bind")
                        && self.is_callable_value(&receiver)
                    {
                        return self.execute_function_prototype_member(
                            member,
                            &receiver,
                            &evaluated_args,
                            event,
                            Some(env),
                        );
                    }

                    if let Value::Array(values) = &receiver {
                        if let Some(value) =
                            self.eval_array_member_call(values, member, &evaluated_args, event)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::String(text) = &receiver {
                        if let Some(value) =
                            self.eval_string_member_call(text, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::Date(value) = &receiver {
                        if let Some(value) =
                            self.eval_date_member_call(value, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::NodeList(nodes) = &receiver {
                        if let Some(value) =
                            self.eval_nodelist_member_call(nodes, member, &evaluated_args, event)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::Node(node) = &receiver {
                        if let Some(value) =
                            self.eval_node_member_call(*node, member, &evaluated_args, event)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::TypedArray(array) = &receiver {
                        if let Some(value) =
                            self.eval_typed_array_member_call(array, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::Blob(blob) = &receiver {
                        if let Some(value) =
                            self.eval_blob_member_call(blob, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::Map(map) = &receiver {
                        let map_member_override = {
                            let map_ref = map.borrow();
                            Self::object_get_entry(&map_ref.properties, member)
                        };
                        if let Some(callee) = map_member_override {
                            return self
                                .execute_callable_value_with_env_and_sync(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    env,
                                )
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "callback is not a function" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "'{}' is not a function",
                                            member
                                        ))
                                    }
                                    other => other,
                                });
                        }
                        if let Some(value) = self.eval_map_member_call_from_values(
                            map,
                            member,
                            &evaluated_args,
                            event,
                        )? {
                            return Ok(value);
                        }
                    }

                    if let Value::Set(set) = &receiver {
                        let set_member_override = {
                            let set_ref = set.borrow();
                            Self::object_get_entry(&set_ref.properties, member)
                        };
                        if let Some(callee) = set_member_override {
                            return self
                                .execute_callable_value_with_env_and_sync(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    env,
                                )
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "callback is not a function" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "'{}' is not a function",
                                            member
                                        ))
                                    }
                                    other => other,
                                });
                        }
                        if let Some(value) = self.eval_set_member_call_from_values(
                            set,
                            member,
                            &evaluated_args,
                            event,
                        )? {
                            return Ok(value);
                        }
                    }

                    if let Value::WeakMap(weak_map) = &receiver {
                        let weak_map_member_override = {
                            let weak_map_ref = weak_map.borrow();
                            Self::object_get_entry(&weak_map_ref.properties, member)
                        };
                        if let Some(callee) = weak_map_member_override {
                            return self
                                .execute_callable_value_with_env_and_sync(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    env,
                                )
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "callback is not a function" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "'{}' is not a function",
                                            member
                                        ))
                                    }
                                    other => other,
                                });
                        }
                        if let Some(value) = self.eval_weak_map_member_call_from_values(
                            weak_map,
                            member,
                            &evaluated_args,
                            event,
                        )? {
                            return Ok(value);
                        }
                    }

                    if let Value::WeakSet(weak_set) = &receiver {
                        let weak_set_member_override = {
                            let weak_set_ref = weak_set.borrow();
                            Self::object_get_entry(&weak_set_ref.properties, member)
                        };
                        if let Some(callee) = weak_set_member_override {
                            return self
                                .execute_callable_value_with_env_and_sync(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    env,
                                )
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "callback is not a function" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "'{}' is not a function",
                                            member
                                        ))
                                    }
                                    other => other,
                                });
                        }
                        if let Some(value) = self.eval_weak_set_member_call_from_values(
                            weak_set,
                            member,
                            &evaluated_args,
                        )? {
                            return Ok(value);
                        }
                    }

                    if let Value::UrlConstructor = &receiver {
                        let url_constructor_override = {
                            let entries = self.browser_apis.url_constructor_properties.borrow();
                            Self::object_get_entry(&entries, member)
                        };
                        if let Some(callee) = url_constructor_override {
                            return self
                                .execute_callable_value_with_env_and_sync(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    env,
                                )
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "callback is not a function" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "'{}' is not a function",
                                            member
                                        ))
                                    }
                                    other => other,
                                });
                        }
                        if let Some(value) =
                            self.eval_url_static_member_call_from_values(member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::Object(object) = &receiver {
                        let is_object_constructor =
                            Self::callable_kind_from_value(&receiver) == Some("object_constructor");
                        if is_object_constructor && member == "assign" {
                            return self.eval_object_assign_static_call(&evaluated_args, event);
                        }
                        if let Some(value) =
                            self.eval_event_target_member_call(object, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                        if let Some(value) = self.eval_named_node_map_member_call(
                            object,
                            member,
                            &evaluated_args,
                            event,
                        )? {
                            return Ok(value);
                        }
                        if let Some(value) =
                            self.eval_event_member_call(object, member, &evaluated_args, event)?
                        {
                            return Ok(value);
                        }
                        if let Some(value) =
                            self.eval_navigation_member_call(object, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                        if let Some(value) =
                            self.eval_mock_file_member_call(object, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                        if let Some(value) = self.eval_clipboard_data_member_call(
                            object,
                            member,
                            &evaluated_args,
                            event,
                        )? {
                            return Ok(value);
                        }
                        let is_fetch_response_object = {
                            let entries = object.borrow();
                            Self::is_fetch_response_object(&entries)
                        };
                        if is_fetch_response_object {
                            if let Some(value) = self.eval_fetch_response_member_call(
                                object,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_fetch_request_object = {
                            let entries = object.borrow();
                            Self::is_fetch_request_object(&entries)
                        };
                        if is_fetch_request_object {
                            if let Some(value) = self.eval_fetch_request_member_call(
                                object,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_headers_object = {
                            let entries = object.borrow();
                            Self::is_headers_object(&entries)
                        };
                        if is_headers_object {
                            if let Some(value) =
                                self.eval_headers_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_cookie_store_object = {
                            let entries = object.borrow();
                            Self::is_cookie_store_object(&entries)
                        };
                        if is_cookie_store_object {
                            if let Some(value) =
                                self.eval_cookie_store_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_cache_storage_object = {
                            let entries = object.borrow();
                            Self::is_cache_storage_object(&entries)
                        };
                        if is_cache_storage_object {
                            if let Some(value) = self.eval_cache_storage_member_call(
                                object,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_cache_object = {
                            let entries = object.borrow();
                            Self::is_cache_object(&entries)
                        };
                        if is_cache_object {
                            if let Some(value) =
                                self.eval_cache_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_import_meta_object = {
                            let entries = object.borrow();
                            matches!(
                                Self::object_get_entry(&entries, INTERNAL_IMPORT_META_OBJECT_KEY),
                                Some(Value::Bool(true))
                            )
                        };
                        if is_import_meta_object && member == "resolve" {
                            return self.eval_import_meta_resolve_call(&evaluated_args);
                        }
                        let is_dom_parser_object = {
                            let entries = object.borrow();
                            matches!(
                                Self::object_get_entry(&entries, INTERNAL_DOM_PARSER_OBJECT_KEY),
                                Some(Value::Bool(true))
                            )
                        };
                        if is_dom_parser_object {
                            if let Some(value) =
                                self.eval_dom_parser_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_parsed_document_object = {
                            let entries = object.borrow();
                            matches!(
                                Self::object_get_entry(
                                    &entries,
                                    INTERNAL_PARSED_DOCUMENT_OBJECT_KEY
                                ),
                                Some(Value::Bool(true))
                            )
                        };
                        if is_parsed_document_object {
                            if let Some(value) = self.eval_parsed_document_member_call(
                                object,
                                member,
                                &evaluated_args,
                                event,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_tree_walker_object = {
                            let entries = object.borrow();
                            matches!(
                                Self::object_get_entry(&entries, INTERNAL_TREE_WALKER_OBJECT_KEY),
                                Some(Value::Bool(true))
                            )
                        };
                        if is_tree_walker_object {
                            if let Some(value) =
                                self.eval_tree_walker_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_range_object = {
                            let entries = object.borrow();
                            Self::is_range_object(&entries)
                        };
                        if is_range_object {
                            if let Some(value) =
                                self.eval_range_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_selection_object = {
                            let entries = object.borrow();
                            Self::is_selection_object(&entries)
                        };
                        if is_selection_object {
                            if let Some(value) =
                                self.eval_selection_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }

                        let is_iterator_constructor = {
                            let entries = object.borrow();
                            Self::is_iterator_constructor_object(&entries)
                        };
                        if is_iterator_constructor {
                            if let Some(value) = self.eval_iterator_constructor_member_call(
                                object,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_iterator = {
                            let entries = object.borrow();
                            Self::is_iterator_object(&entries)
                        };
                        if is_iterator {
                            if let Some(value) = self.eval_iterator_member_call(
                                object,
                                member,
                                &evaluated_args,
                                event,
                            )? {
                                return Ok(value);
                            }
                        }
                        if Self::is_canvas_2d_context_object(&object.borrow()) {
                            if let Some(value) = self.eval_canvas_2d_context_member_call(
                                object,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_document_object = {
                            let entries = object.borrow();
                            matches!(
                                Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                                Some(Value::Bool(true))
                            )
                        };
                        if is_document_object {
                            if let Some(value) =
                                self.eval_document_member_call(member, &evaluated_args, event)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_window_object = {
                            let entries = object.borrow();
                            Self::is_window_object(&entries)
                        };
                        if is_window_object {
                            if let Some(value) =
                                self.eval_window_member_call(member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        if Self::is_url_object(&object.borrow()) {
                            if let Some(value) =
                                self.eval_url_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        if Self::is_url_search_params_object(&object.borrow()) {
                            if let Some(value) = self.eval_url_search_params_member_call(
                                object,
                                member,
                                &evaluated_args,
                                event,
                            )? {
                                return Ok(value);
                            }
                        }
                        if Self::is_storage_object(&object.borrow()) {
                            if let Some(value) =
                                self.eval_storage_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                    }

                    let callee = self.object_property_from_value(&receiver, member).map_err(
                        |err| match err {
                            Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                                Error::ScriptRuntime(format!(
                                    "member call target does not support property '{}'",
                                    member
                                ))
                            }
                            other => other,
                        },
                    )?;
                    self.execute_callable_value_with_this_and_env_and_sync(
                        &callee,
                        &evaluated_args,
                        event,
                        env,
                        Some(receiver.clone()),
                    )
                    .map_err(|err| match err {
                        Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                            Error::ScriptRuntime(format!("'{}' is not a function", member))
                        }
                        other => other,
                    })
                }
                Expr::PrivateMemberCall {
                    target,
                    member,
                    args,
                } => {
                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    let evaluated_args =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    self.private_call_member(member, &receiver, &evaluated_args, env, event)
                }
                Expr::MemberGet {
                    target,
                    member,
                    optional,
                } => {
                    if Self::is_super_target_expr(target) {
                        let super_prototype = Self::super_prototype_from_env(env)?;
                        let this_value = Self::super_this_from_env(env)?;
                        return self.object_property_from_value_with_receiver(
                            &super_prototype,
                            member,
                            &this_value,
                        );
                    }
                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    if *optional && matches!(receiver, Value::Null | Value::Undefined) {
                        return Ok(Value::Undefined);
                    }
                    self.object_property_from_value(&receiver, member)
                }
                Expr::PrivateMemberGet { target, member } => {
                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    self.private_get_member(member, &receiver, env, event)
                }
                Expr::PrivateIn { member, target } => {
                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    Ok(Value::Bool(self.private_has_member(member, &receiver)?))
                }
                Expr::IndexGet {
                    target,
                    index,
                    optional,
                } => {
                    let is_super = Self::is_super_target_expr(target);
                    let receiver = if is_super {
                        Self::super_prototype_from_env(env)?
                    } else {
                        self.eval_expr(target, env, event_param, event)?
                    };
                    if *optional && matches!(receiver, Value::Null | Value::Undefined) {
                        return Ok(Value::Undefined);
                    }
                    let index_value = self.eval_expr(index, env, event_param, event)?;
                    let key = match index_value {
                        Value::Number(value) => value.to_string(),
                        Value::BigInt(value) => value.to_string(),
                        Value::Float(value) if value.is_finite() && value.fract() == 0.0 => {
                            format!("{:.0}", value)
                        }
                        other => self.property_key_to_storage_key(&other),
                    };
                    if is_super {
                        let this_value = Self::super_this_from_env(env)?;
                        self.object_property_from_value_with_receiver(&receiver, &key, &this_value)
                    } else {
                        self.object_property_from_value(&receiver, &key)
                    }
                }
                Expr::Var(name) => {
                    if name == "super" {
                        return Self::super_prototype_from_env(env);
                    }
                    self.ensure_binding_initialized(env, name)?;
                    if let Some(pending) = self.resolve_listener_capture_pending_value(name) {
                        if let Some(value) = pending {
                            Ok(value)
                        } else {
                            Err(Error::ScriptRuntime(format!("unknown variable: {name}")))
                        }
                    } else if let Some(value) = env.get(name).cloned() {
                        Ok(value)
                    } else if let Some(value) = self.resolve_pending_function_decl(name, env) {
                        Ok(value)
                    } else {
                        Err(Error::ScriptRuntime(format!("unknown variable: {name}")))
                    }
                }
                Expr::ImportMeta => self.eval_import_meta_object(),
                Expr::NewTarget => self.eval_new_target_value(env),
                Expr::DomRef(target) => {
                    let is_list_query = matches!(
                        target,
                        DomQuery::BySelectorAll { .. } | DomQuery::QuerySelectorAll { .. }
                    );
                    if is_list_query {
                        let nodes = self
                            .resolve_dom_query_list_runtime(target, env)?
                            .unwrap_or_default();
                        Ok(Self::new_static_node_list_value(nodes))
                    } else if matches!(target, DomQuery::ById(_)) {
                        Ok(self
                            .resolve_dom_query_runtime(target, env)?
                            .map(Value::Node)
                            .unwrap_or(Value::Null))
                    } else {
                        Ok(self
                            .resolve_dom_query_runtime(target, env)?
                            .map(Value::Node)
                            .unwrap_or(Value::Null))
                    }
                }
                Expr::CreateElement(tag_name) => {
                    let node = self.dom.create_detached_element(tag_name.clone());
                    Ok(Value::Node(node))
                }
                Expr::CreateTextNode(text) => {
                    let node = self.dom.create_detached_text(text.clone());
                    Ok(Value::Node(node))
                }
                Expr::Function {
                    handler,
                    name,
                    is_async,
                    is_generator,
                    is_arrow,
                    is_method,
                } => {
                    let value = self.make_function_value(
                        handler.clone(),
                        env,
                        false,
                        *is_async,
                        *is_generator,
                        *is_arrow,
                        *is_method,
                    );
                    let Value::Function(function) = value else {
                        return Ok(value);
                    };
                    if let Some(expression_name) = name {
                        let mut named = function.as_ref().clone();
                        named.expression_name = Some(expression_name.clone());
                        named.local_bindings.insert(expression_name.clone());
                        Ok(Value::Function(Rc::new(named)))
                    } else {
                        Ok(Value::Function(function))
                    }
                }

                Expr::SetTimeout { handler, delay_ms } => {
                    let delay = self.eval_expr(delay_ms, env, event_param, event)?;
                    let delay = Self::value_to_i64(&delay);
                    let callback_args = handler
                        .args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    let (callback, timer_env) = self.materialize_timer_callback_for_schedule(
                        &handler.callback,
                        env,
                        "timeout",
                    );
                    let id = self.schedule_timeout(callback, delay, callback_args, &timer_env);
                    Ok(Value::Number(id))
                }
                Expr::SetInterval { handler, delay_ms } => {
                    let interval = self.eval_expr(delay_ms, env, event_param, event)?;
                    let interval = Self::value_to_i64(&interval);
                    let callback_args = handler
                        .args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    let (callback, timer_env) = self.materialize_timer_callback_for_schedule(
                        &handler.callback,
                        env,
                        "interval",
                    );
                    let id = self.schedule_interval(callback, interval, callback_args, &timer_env);
                    Ok(Value::Number(id))
                }
                Expr::RequestAnimationFrame { callback } => {
                    let (callback, timer_env) =
                        self.materialize_timer_callback_for_schedule(callback, env, "raf");
                    let id = self.schedule_animation_frame(callback, &timer_env);
                    Ok(Value::Number(id))
                }
                Expr::QueueMicrotask { handler } => {
                    self.queue_microtask(handler.clone(), env);
                    Ok(Value::Undefined)
                }
                Expr::Binary { left, op, right } => match op {
                    BinaryOp::And => {
                        let mut operands =
                            Self::collect_left_associative_binary_operands(expr, BinaryOp::And)
                                .into_iter();
                        let Some(first) = operands.next() else {
                            return Ok(Value::Undefined);
                        };
                        let mut current = self.eval_expr(first, env, event_param, event)?;
                        for operand in operands {
                            if !current.truthy() {
                                return Ok(current);
                            }
                            current = self.eval_expr(operand, env, event_param, event)?;
                        }
                        Ok(current)
                    }
                    BinaryOp::Or => {
                        let mut operands =
                            Self::collect_left_associative_binary_operands(expr, BinaryOp::Or)
                                .into_iter();
                        let Some(first) = operands.next() else {
                            return Ok(Value::Undefined);
                        };
                        let mut current = self.eval_expr(first, env, event_param, event)?;
                        for operand in operands {
                            if current.truthy() {
                                return Ok(current);
                            }
                            current = self.eval_expr(operand, env, event_param, event)?;
                        }
                        Ok(current)
                    }
                    BinaryOp::Nullish => {
                        let mut operands =
                            Self::collect_left_associative_binary_operands(expr, BinaryOp::Nullish)
                                .into_iter();
                        let Some(first) = operands.next() else {
                            return Ok(Value::Undefined);
                        };
                        let mut current = self.eval_expr(first, env, event_param, event)?;
                        for operand in operands {
                            if matches!(current, Value::Null | Value::Undefined) {
                                current = self.eval_expr(operand, env, event_param, event)?;
                            } else {
                                break;
                            }
                        }
                        Ok(current)
                    }
                    _ => {
                        let left = self.eval_expr(left, env, event_param, event)?;
                        let right = self.eval_expr(right, env, event_param, event)?;
                        self.eval_binary(op, &left, &right)
                    }
                },
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
