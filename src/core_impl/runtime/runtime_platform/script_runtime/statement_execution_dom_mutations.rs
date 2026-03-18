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

    fn execute_dom_method_call_stmt_with_env(
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

    fn execute_listener_mutation_stmt_with_env(
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

    fn execute_dispatch_event_stmt_with_env(
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

    fn execute_class_list_call_stmt(
        &mut self,
        target: &DomQuery,
        optional: bool,
        method: &ClassListMethod,
        class_names: &[Expr],
        force: &Option<Expr>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let node = if optional {
            if let DomQuery::Var(name) = target {
                if matches!(env.get(name), Some(Value::Null | Value::Undefined)) {
                    return Ok(ExecFlow::Continue);
                }
            }
            match self.resolve_dom_query_runtime(target, env)? {
                Some(node) => node,
                None => return Ok(ExecFlow::Continue),
            }
        } else {
            self.resolve_dom_query_required_runtime(target, env)?
        };
        match method {
            ClassListMethod::Add => {
                for class_name in class_names {
                    let class_name = self
                        .eval_expr(class_name, env, event_param, event)?
                        .as_string();
                    self.dom.class_add(node, &class_name)?;
                }
            }
            ClassListMethod::Remove => {
                for class_name in class_names {
                    let class_name = self
                        .eval_expr(class_name, env, event_param, event)?
                        .as_string();
                    self.dom.class_remove(node, &class_name)?;
                }
            }
            ClassListMethod::Toggle => {
                let class_name = class_names
                    .first()
                    .ok_or_else(|| Error::ScriptRuntime("toggle requires a class name".into()))?;
                let class_name = self
                    .eval_expr(class_name, env, event_param, event)?
                    .as_string();
                if let Some(force_expr) = force {
                    let force_value = self
                        .eval_expr(force_expr, env, event_param, event)?
                        .truthy();
                    if force_value {
                        self.dom.class_add(node, &class_name)?;
                    } else {
                        self.dom.class_remove(node, &class_name)?;
                    }
                } else {
                    let _ = self.dom.class_toggle(node, &class_name)?;
                }
            }
            ClassListMethod::Replace => {
                let old_class_name = class_names.first().ok_or_else(|| {
                    Error::ScriptRuntime("replace requires old and new class names".into())
                })?;
                let new_class_name = class_names.get(1).ok_or_else(|| {
                    Error::ScriptRuntime("replace requires old and new class names".into())
                })?;
                let old_class_name = self
                    .eval_expr(old_class_name, env, event_param, event)?
                    .as_string();
                let new_class_name = self
                    .eval_expr(new_class_name, env, event_param, event)?
                    .as_string();
                let _ = self
                    .dom
                    .class_replace(node, &old_class_name, &new_class_name)?;
            }
        }
        Ok(ExecFlow::Continue)
    }

    fn execute_dom_set_attribute_stmt(
        &mut self,
        target: &DomQuery,
        name: &str,
        value: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let node = self.resolve_dom_query_required_runtime(target, env)?;
        let value = self.eval_expr(value, env, event_param, event)?;
        let normalized_name = name.to_ascii_lowercase();
        if !is_valid_create_attribute_name(&normalized_name) {
            return Err(Error::ScriptRuntime(
                "InvalidCharacterError: attribute name is not a valid XML name".into(),
            ));
        }
        if normalized_name == "open"
            && self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
        {
            let _ = self.set_details_open_state_with_env(node, true, env)?;
        } else {
            self.dom
                .set_attr(node, &normalized_name, &value.as_string())?;
        }
        Ok(ExecFlow::Continue)
    }

    fn execute_dom_remove_attribute_stmt(
        &mut self,
        target: &DomQuery,
        name: &str,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        let node = self.resolve_dom_query_required_runtime(target, env)?;
        if name.eq_ignore_ascii_case("open")
            && self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
        {
            let _ = self.set_details_open_state_with_env(node, false, env)?;
        } else {
            self.dom.remove_attr(node, name)?;
        }
        Ok(ExecFlow::Continue)
    }

    fn execute_node_tree_mutation_stmt(
        &mut self,
        target: &DomQuery,
        method: &NodeTreeMethod,
        child: &Expr,
        reference: &Option<Expr>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let target_node = self.resolve_dom_query_required_runtime(target, env)?;
        let child = self.eval_expr(child, env, event_param, event)?;
        let Value::Node(child) = child else {
            return Err(Error::ScriptRuntime(
                "before/after/replaceWith/append/appendChild/prepend/removeChild/insertBefore argument must be an element reference".into(),
            ));
        };
        match method {
            NodeTreeMethod::After => self.dom.insert_after(target_node, child)?,
            NodeTreeMethod::Append => self.dom.append_child(target_node, child)?,
            NodeTreeMethod::AppendChild => self.dom.append_child(target_node, child)?,
            NodeTreeMethod::Before => {
                let Some(parent) = self.dom.parent(target_node) else {
                    return Ok(ExecFlow::Continue);
                };
                self.dom.insert_before(parent, child, target_node)?;
            }
            NodeTreeMethod::ReplaceWith => {
                self.dom.replace_with(target_node, child)?;
            }
            NodeTreeMethod::Prepend => self.dom.prepend_child(target_node, child)?,
            NodeTreeMethod::RemoveChild => self.dom.remove_child(target_node, child)?,
            NodeTreeMethod::InsertBefore => {
                let Some(reference) = reference else {
                    return Err(Error::ScriptRuntime(
                        "insertBefore requires reference node".into(),
                    ));
                };
                let reference = self.eval_expr(reference, env, event_param, event)?;
                let Value::Node(reference) = reference else {
                    return Err(Error::ScriptRuntime(
                        "insertBefore reference must be an element reference".into(),
                    ));
                };
                self.dom.insert_before(target_node, child, reference)?;
            }
        }
        Ok(ExecFlow::Continue)
    }

    fn execute_insert_adjacent_element_stmt(
        &mut self,
        target: &DomQuery,
        position: &InsertAdjacentPosition,
        node: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let target_node = self.resolve_dom_query_required_runtime(target, env)?;
        let node = self.eval_expr(node, env, event_param, event)?;
        let Value::Node(node) = node else {
            return Err(Error::ScriptRuntime(
                "TypeError: Failed to execute 'insertAdjacentElement': parameter 2 is not of type 'Element'".into(),
            ));
        };
        let node_is_fragment = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("#document-fragment"));
        if self.dom.element(node).is_none() || node_is_fragment {
            return Err(Error::ScriptRuntime(
                "TypeError: Failed to execute 'insertAdjacentElement': parameter 2 is not of type 'Element'".into(),
            ));
        }

        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(target_node) else {
                return Ok(ExecFlow::Continue);
            };
            let parent_is_fragment = self
                .dom
                .tag_name(parent)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("#document-fragment"));
            if self.dom.element(parent).is_none() || parent_is_fragment {
                return Ok(ExecFlow::Continue);
            }
        }

        let _ = self.dom.insert_adjacent_node(target_node, *position, node);
        Ok(ExecFlow::Continue)
    }

    fn execute_insert_adjacent_text_stmt(
        &mut self,
        target: &DomQuery,
        position: &InsertAdjacentPosition,
        text: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let target_node = self.resolve_dom_query_required_runtime(target, env)?;
        let text = self.eval_expr(text, env, event_param, event)?;
        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(target_node) else {
                return Ok(ExecFlow::Continue);
            };
            let parent_is_fragment = self
                .dom
                .tag_name(parent)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("#document-fragment"));
            if self.dom.element(parent).is_none() || parent_is_fragment {
                return Ok(ExecFlow::Continue);
            }
        }
        let text_node = self.dom.create_detached_text(text.as_string());
        self.dom
            .insert_adjacent_node(target_node, *position, text_node)?;
        Ok(ExecFlow::Continue)
    }

    fn execute_insert_adjacent_html_stmt(
        &mut self,
        target: &DomQuery,
        position: &Expr,
        html: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let target_node = self.resolve_dom_query_required_runtime(target, env)?;
        let position = self.eval_expr(position, env, event_param, event)?;
        let position_text = position.as_string();
        let position = resolve_insert_adjacent_position(&position_text).map_err(|_| {
            Error::ScriptRuntime(format!(
                "SyntaxError: unsupported insertAdjacentHTML position: {position_text}"
            ))
        })?;
        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(target_node) else {
                return Err(Error::ScriptRuntime(
                    "NoModificationAllowedError: Failed to execute 'insertAdjacentHTML' because the target has no parent element".into(),
                ));
            };
            let parent_is_fragment = self
                .dom
                .tag_name(parent)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("#document-fragment"));
            if self.dom.element(parent).is_none() || parent_is_fragment {
                return Err(Error::ScriptRuntime(
                    "NoModificationAllowedError: Failed to execute 'insertAdjacentHTML' on a node whose parent is not an Element".into(),
                ));
            }
        }
        let html = self.eval_expr(html, env, event_param, event)?;
        match self
            .dom
            .insert_adjacent_html(target_node, position, &html.as_string())
        {
            Ok(()) => {}
            Err(Error::ScriptParse(message)) => {
                return Err(Error::ScriptRuntime(format!("SyntaxError: {message}")));
            }
            Err(other) => return Err(other),
        }
        Ok(ExecFlow::Continue)
    }

    fn execute_set_timeout_stmt(
        &mut self,
        handler: &TimerInvocation,
        delay_ms: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let delay = self.eval_expr(delay_ms, env, event_param, event)?;
        let delay = Self::value_to_i64(&delay);
        let callback_args = handler
            .args
            .iter()
            .map(|arg| self.eval_expr(arg, env, event_param, event))
            .collect::<Result<Vec<_>>>()?;
        let (callback, timer_env) =
            self.materialize_timer_callback_for_schedule(&handler.callback, env, "timeout");
        let _ = self.schedule_timeout(callback, delay, callback_args, &timer_env);
        Ok(ExecFlow::Continue)
    }

    fn execute_set_interval_stmt(
        &mut self,
        handler: &TimerInvocation,
        delay_ms: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let interval = self.eval_expr(delay_ms, env, event_param, event)?;
        let interval = Self::value_to_i64(&interval);
        let callback_args = handler
            .args
            .iter()
            .map(|arg| self.eval_expr(arg, env, event_param, event))
            .collect::<Result<Vec<_>>>()?;
        let (callback, timer_env) =
            self.materialize_timer_callback_for_schedule(&handler.callback, env, "interval");
        let _ = self.schedule_interval(callback, interval, callback_args, &timer_env);
        Ok(ExecFlow::Continue)
    }

    fn execute_clear_timeout_stmt(
        &mut self,
        timer_id: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let timer_id = self.eval_expr(timer_id, env, event_param, event)?;
        let timer_id = Self::value_to_i64(&timer_id);
        self.clear_timeout(timer_id);
        Ok(ExecFlow::Continue)
    }

    fn execute_node_remove_stmt(
        &mut self,
        target: &DomQuery,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        let node = self.resolve_dom_query_required_runtime(target, env)?;
        if let Some(active) = self.dom.active_element() {
            if active == node || self.dom.is_descendant_of(active, node) {
                self.dom.set_active_element(None);
            }
        }
        if let Some(active_pseudo) = self.dom.active_pseudo_element() {
            if active_pseudo == node || self.dom.is_descendant_of(active_pseudo, node) {
                self.dom.set_active_pseudo_element(None);
            }
        }
        self.dom.remove_node(node)?;
        Ok(ExecFlow::Continue)
    }

    pub(crate) fn try_execute_dom_mutation_stmt(
        &mut self,
        stmt: &Stmt,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<Option<ExecFlow>> {
        match stmt {
            Stmt::ClassListCall {
                target,
                optional,
                method,
                class_names,
                force,
            } => Ok(Some(self.execute_class_list_call_stmt(
                target,
                *optional,
                method,
                class_names,
                force,
                env,
                event_param,
                event,
            )?)),
            Stmt::DomSetAttribute {
                target,
                name,
                value,
            } => Ok(Some(self.execute_dom_set_attribute_stmt(
                target,
                name,
                value,
                env,
                event_param,
                event,
            )?)),
            Stmt::DomRemoveAttribute { target, name } => Ok(Some(
                self.execute_dom_remove_attribute_stmt(target, name, env)?,
            )),
            Stmt::NodeTreeMutation {
                target,
                method,
                child,
                reference,
            } => Ok(Some(self.execute_node_tree_mutation_stmt(
                target,
                method,
                child,
                reference,
                env,
                event_param,
                event,
            )?)),
            Stmt::InsertAdjacentElement {
                target,
                position,
                node,
            } => Ok(Some(self.execute_insert_adjacent_element_stmt(
                target,
                position,
                node,
                env,
                event_param,
                event,
            )?)),
            Stmt::InsertAdjacentText {
                target,
                position,
                text,
            } => Ok(Some(self.execute_insert_adjacent_text_stmt(
                target,
                position,
                text,
                env,
                event_param,
                event,
            )?)),
            Stmt::InsertAdjacentHTML {
                target,
                position,
                html,
            } => Ok(Some(self.execute_insert_adjacent_html_stmt(
                target,
                position,
                html,
                env,
                event_param,
                event,
            )?)),
            Stmt::SetTimeout { handler, delay_ms } => Ok(Some(self.execute_set_timeout_stmt(
                handler,
                delay_ms,
                env,
                event_param,
                event,
            )?)),
            Stmt::SetInterval { handler, delay_ms } => Ok(Some(self.execute_set_interval_stmt(
                handler,
                delay_ms,
                env,
                event_param,
                event,
            )?)),
            Stmt::QueueMicrotask { handler } => {
                self.queue_microtask(handler.clone(), env);
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::ClearTimeout { timer_id } => Ok(Some(self.execute_clear_timeout_stmt(
                timer_id,
                env,
                event_param,
                event,
            )?)),
            Stmt::NodeRemove { target } => Ok(Some(self.execute_node_remove_stmt(target, env)?)),
            Stmt::ListenerMutation {
                target,
                op,
                event_type,
                capture,
                is_arrow,
                handler,
            } => {
                self.execute_listener_mutation_stmt_with_env(
                    target,
                    op,
                    event_type,
                    *capture,
                    *is_arrow,
                    handler,
                    env,
                    event_param,
                    event,
                )?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::DomMethodCall {
                target,
                method,
                arg,
            } => {
                self.execute_dom_method_call_stmt_with_env(
                    target,
                    method,
                    arg,
                    env,
                    event_param,
                    event,
                )?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::DispatchEvent { target, event_type } => {
                self.execute_dispatch_event_stmt_with_env(
                    target,
                    event_type,
                    env,
                    event_param,
                    event,
                )?;
                Ok(Some(ExecFlow::Continue))
            }
            _ => Ok(None),
        }
    }
}
