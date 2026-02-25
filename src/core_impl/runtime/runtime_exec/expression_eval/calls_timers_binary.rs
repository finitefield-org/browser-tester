use super::*;

impl Harness {
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
                    let callee = if let Some(callee) = env.get(target).cloned() {
                        callee
                    } else if let Some(callee) = self.resolve_pending_function_decl(target, env) {
                        callee
                    } else {
                        return Err(Error::ScriptRuntime(format!("unknown variable: {target}")));
                    };
                    let evaluated_args =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    self.execute_callable_value_with_env(&callee, &evaluated_args, event, Some(env))
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
                    self.execute_callable_value_with_env(&callee, &evaluated_args, event, Some(env))
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
                            .execute_callable_value_with_this_and_env(
                                &callee,
                                &evaluated_args,
                                event,
                                Some(env),
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
                            .execute_callable_value_with_this_and_env(
                                &callee,
                                &evaluated_args,
                                event,
                                Some(env),
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
                                .execute_callable_value_with_env(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    Some(env),
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

                    if let Value::WeakMap(weak_map) = &receiver {
                        let weak_map_member_override = {
                            let weak_map_ref = weak_map.borrow();
                            Self::object_get_entry(&weak_map_ref.properties, member)
                        };
                        if let Some(callee) = weak_map_member_override {
                            return self
                                .execute_callable_value_with_env(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    Some(env),
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
                                .execute_callable_value_with_env(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    Some(env),
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
                                .execute_callable_value_with_env(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    Some(env),
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
                    self.execute_callable_value_with_this_and_env(
                        &callee,
                        &evaluated_args,
                        event,
                        Some(env),
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
                        self.object_property_from_value_with_receiver(
                            &receiver,
                            &key,
                            &this_value,
                        )
                    } else {
                        self.object_property_from_value(&receiver, &key)
                    }
                }
                Expr::Var(name) => {
                    if name == "super" {
                        return Self::super_prototype_from_env(env);
                    }
                    self.ensure_binding_initialized(env, name)?;
                    if let Some(value) = env.get(name).cloned() {
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
                        Ok(Value::NodeList(nodes))
                    } else {
                        let node = self.resolve_dom_query_required_runtime(target, env)?;
                        Ok(Value::Node(node))
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
                    let id =
                        self.schedule_timeout(handler.callback.clone(), delay, callback_args, env);
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
                    let id = self.schedule_interval(
                        handler.callback.clone(),
                        interval,
                        callback_args,
                        env,
                    );
                    Ok(Value::Number(id))
                }
                Expr::RequestAnimationFrame { callback } => {
                    const FRAME_DELAY_MS: i64 = 16;
                    let callback_args = vec![Value::Number(
                        self.scheduler.now_ms.saturating_add(FRAME_DELAY_MS),
                    )];
                    let id =
                        self.schedule_timeout(callback.clone(), FRAME_DELAY_MS, callback_args, env);
                    Ok(Value::Number(id))
                }
                Expr::QueueMicrotask { handler } => {
                    self.queue_microtask(handler.clone(), env);
                    Ok(Value::Null)
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
