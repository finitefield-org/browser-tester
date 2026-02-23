use super::*;

impl Harness {
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

    fn super_prototype_from_env(env: &HashMap<String, Value>) -> Result<Value> {
        env.get(INTERNAL_CLASS_SUPER_PROTOTYPE_KEY)
            .cloned()
            .ok_or_else(|| {
                Error::ScriptRuntime("super property access is only valid in a class method".into())
            })
    }

    fn super_this_from_env(env: &HashMap<String, Value>) -> Result<Value> {
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
                        return self.execute_constructor_value_with_this_and_env(
                            &super_constructor,
                            &evaluated_args,
                            event,
                            Some(env),
                            Some(this_value),
                        );
                    }
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
                Expr::Call { target, args } => {
                    let callee = self.eval_expr(target, env, event_param, event)?;
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
                } => {
                    if Self::is_super_target_expr(target) {
                        let super_prototype = Self::super_prototype_from_env(env)?;
                        let this_value = Self::super_this_from_env(env)?;
                        let evaluated_args =
                            self.eval_call_args_with_spread(args, env, event_param, event)?;
                        let callee = self.object_property_from_value(&super_prototype, member)?;
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
                    let evaluated_args =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;

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
                Expr::MemberGet {
                    target,
                    member,
                    optional,
                } => {
                    if Self::is_super_target_expr(target) {
                        let super_prototype = Self::super_prototype_from_env(env)?;
                        return self.object_property_from_value(&super_prototype, member);
                    }
                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    if *optional && matches!(receiver, Value::Null | Value::Undefined) {
                        return Ok(Value::Undefined);
                    }
                    self.object_property_from_value(&receiver, member)
                }
                Expr::IndexGet {
                    target,
                    index,
                    optional,
                } => {
                    let receiver = if Self::is_super_target_expr(target) {
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
                    self.object_property_from_value(&receiver, &key)
                }
                Expr::Var(name) => {
                    if let Some(value) = env.get(name).cloned() {
                        Ok(value)
                    } else if let Some(value) = self.resolve_pending_function_decl(name, env) {
                        Ok(value)
                    } else {
                        Err(Error::ScriptRuntime(format!("unknown variable: {name}")))
                    }
                }
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
                    is_async,
                    is_generator,
                } => Ok(self.make_function_value(
                    handler.clone(),
                    env,
                    false,
                    *is_async,
                    *is_generator,
                )),

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
