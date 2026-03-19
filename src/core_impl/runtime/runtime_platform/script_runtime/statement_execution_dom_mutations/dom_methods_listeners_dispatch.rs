use super::*;

impl Harness {
    fn dom_method_member_name(method: &DomMethod) -> &'static str {
        match method {
            DomMethod::Focus => "focus",
            DomMethod::Blur => "blur",
            DomMethod::Click => "click",
            DomMethod::ScrollIntoView => "scrollIntoView",
            DomMethod::Submit => "submit",
            DomMethod::RequestSubmit => "requestSubmit",
            DomMethod::Reset => "reset",
            DomMethod::Show => "show",
            DomMethod::ShowModal => "showModal",
            DomMethod::Close => "close",
            DomMethod::RequestClose => "requestClose",
        }
    }

    fn try_execute_non_dom_method_call_with_env(
        &mut self,
        target: &DomQuery,
        method: &DomMethod,
        arg_value: Option<Value>,
        env: &mut HashMap<String, Value>,
        event: &EventState,
    ) -> Result<bool> {
        let Some(receiver) = self.resolve_dom_query_value_runtime(target, env)? else {
            return Ok(false);
        };
        if matches!(receiver, Value::Node(_)) {
            return Ok(false);
        }

        let member = Self::dom_method_member_name(method);
        let callee = match self.object_property_from_value(&receiver, member) {
            Ok(callee) if self.is_callable_value(&callee) => callee,
            _ => return Ok(false),
        };

        let mut evaluated_args = Vec::new();
        if let Some(arg_value) = arg_value {
            evaluated_args.push(arg_value);
        }
        let _ = self.execute_callable_value_with_this_and_env(
            &callee,
            &evaluated_args,
            event,
            Some(env),
            Some(receiver),
        )?;
        self.sync_listener_capture_env_if_shared(env);
        Ok(true)
    }

    fn eval_dom_method_call_arg(
        &mut self,
        arg: &Option<Expr>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<Option<Value>> {
        arg.as_ref()
            .map(|expr| self.eval_expr(expr, env, event_param, event))
            .transpose()
    }

    fn try_execute_canvas_context_reset_with_env(
        &mut self,
        target: &DomQuery,
        method: &DomMethod,
        env: &HashMap<String, Value>,
    ) -> Result<bool> {
        if !matches!(method, DomMethod::Reset) {
            return Ok(false);
        }
        let DomQuery::Var(name) = target else {
            return Ok(false);
        };
        let Some(Value::Object(context_object)) = env.get(name) else {
            return Ok(false);
        };

        let is_canvas_context = {
            let entries = context_object.borrow();
            Self::is_canvas_2d_context_object(&entries)
        };
        if !is_canvas_context {
            return Ok(false);
        }

        let _ = self.eval_canvas_2d_context_member_call(context_object, "reset", &[])?;
        Ok(true)
    }

    fn resolve_dom_method_call_node_with_fallback(
        &mut self,
        target: &DomQuery,
        method: &DomMethod,
        arg_value: &Option<Value>,
        env: &mut HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Option<NodeId>> {
        match self.resolve_dom_query_required_runtime(target, env) {
            Ok(node) => Ok(Some(node)),
            Err(dom_resolution_error) => {
                if self.try_execute_non_dom_method_call_with_env(
                    target,
                    method,
                    arg_value.clone(),
                    env,
                    event,
                )? {
                    return Ok(None);
                }
                Err(dom_resolution_error)
            }
        }
    }

    fn execute_dom_method_on_node_with_env(
        &mut self,
        node: NodeId,
        method: &DomMethod,
        arg_value: Option<Value>,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        match method {
            DomMethod::Focus => self.focus_node_with_env(node, env)?,
            DomMethod::Blur => self.blur_node_with_env(node, env)?,
            DomMethod::Click => self.click_dom_method_with_env(node, env)?,
            DomMethod::Submit => self.submit_form_with_env(node, env)?,
            DomMethod::RequestSubmit => self.request_submit_form_with_env(node, arg_value, env)?,
            DomMethod::Reset => self.reset_form_with_env(node, env)?,
            DomMethod::ScrollIntoView => self.scroll_into_view_node_with_env(node, env)?,
            DomMethod::Show => self.show_dialog_with_env(node, false, env)?,
            DomMethod::ShowModal => self.show_dialog_with_env(node, true, env)?,
            DomMethod::Close => self.close_dialog_with_env(node, arg_value, env)?,
            DomMethod::RequestClose => self.request_close_dialog_with_env(node, arg_value, env)?,
        }
        Ok(())
    }

    pub(super) fn execute_dom_method_call_stmt_with_env(
        &mut self,
        target: &DomQuery,
        method: &DomMethod,
        arg: &Option<Expr>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        let arg_value = self.eval_dom_method_call_arg(arg, env, event_param, event)?;
        if self.try_execute_canvas_context_reset_with_env(target, method, env)? {
            return Ok(());
        }

        let Some(node) = self
            .resolve_dom_method_call_node_with_fallback(target, method, &arg_value, env, event)?
        else {
            return Ok(());
        };

        self.execute_dom_method_on_node_with_env(node, method, arg_value, env)
    }

    fn apply_listener_mutation_on_node_with_env(
        &mut self,
        node: NodeId,
        op: &ListenerRegistrationOp,
        event_type: &str,
        capture: bool,
        is_arrow: bool,
        handler: &ScriptHandler,
        env: &HashMap<String, Value>,
    ) {
        match op {
            ListenerRegistrationOp::Add => {
                let function = self.make_function_value(
                    handler.clone(),
                    env,
                    false,
                    false,
                    false,
                    is_arrow,
                    false,
                );
                let (function, captured_env, captured_pending_function_decls) = match function {
                    Value::Function(function) => (
                        Some(function.clone()),
                        function.captured_env.clone(),
                        function.captured_pending_function_decls.clone(),
                    ),
                    _ => {
                        let captured_env = self.ensure_listener_capture_env();
                        *captured_env.borrow_mut() = ScriptEnv::from_snapshot(env);
                        (
                            None,
                            captured_env,
                            self.script_runtime.pending_function_decls.clone(),
                        )
                    }
                };
                let captured_names = function
                    .as_ref()
                    .map(|function| function.captured_names.clone())
                    .unwrap_or_else(|| Self::collect_function_capture_names(handler));
                self.listeners.add(
                    node,
                    event_type.to_string(),
                    Listener {
                        capture,
                        is_event_handler_property: false,
                        is_arrow,
                        handler: handler.clone(),
                        function,
                        captured_names,
                        captured_env,
                        captured_pending_function_decls,
                    },
                );
            }
            ListenerRegistrationOp::Remove => {
                let _ = self.listeners.remove(node, event_type, capture, handler);
            }
        }
    }

    pub(super) fn execute_listener_mutation_stmt_with_env(
        &mut self,
        target: &DomQuery,
        op: &ListenerRegistrationOp,
        event_type_expr: &Expr,
        capture: bool,
        is_arrow: bool,
        handler: &ScriptHandler,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        let event_type = self
            .eval_expr(event_type_expr, env, event_param, event)?
            .as_string();
        if let Some(target_object) = self.resolve_event_target_object_for_query(target, env)? {
            let node = self.event_target_listener_node_id(&target_object);
            self.apply_listener_mutation_on_node_with_env(
                node,
                op,
                &event_type,
                capture,
                is_arrow,
                handler,
                env,
            );
            return Ok(());
        }

        let node = self.resolve_dom_query_required_runtime(target, env)?;
        self.apply_listener_mutation_on_node_with_env(
            node,
            op,
            &event_type,
            capture,
            is_arrow,
            handler,
            env,
        );
        Ok(())
    }

    pub(super) fn execute_dispatch_event_stmt_with_env(
        &mut self,
        target: &DomQuery,
        event_type_expr: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        let event_payload = self.eval_expr(event_type_expr, env, event_param, event)?;
        if let Some(target_object) = self.resolve_event_target_object_for_query(target, env)? {
            let _ = self.dispatch_event_target_with_env(target_object, event_payload, env)?;
            return Ok(());
        }

        let node = self.resolve_dom_query_required_runtime(target, env)?;
        let _ = self.dispatch_dom_event_payload_with_env(node, event_payload, env)?;
        Ok(())
    }
}
