use super::*;

pub(crate) const INTERNAL_ASYNC_FUNCTION_SUSPENDED: &str = "\u{0}\u{0}bt_async_function_suspended";
pub(crate) const INTERNAL_ASYNC_AWAIT_VALUE_PARAM: &str = "__bt_async_await_value";

#[derive(Clone)]
pub(crate) enum TopLevelAwaitResumeKind {
    Ignore,
    Declare { name: String, kind: VarDeclKind },
    Assign { name: String },
    Return,
}

pub(crate) enum TopLevelAwaitOutcome {
    Resolved(Value),
    Pending(Rc<RefCell<PromiseValue>>),
}

impl Harness {
    pub(crate) fn isolate_execution_const_bindings(env: &mut HashMap<String, Value>) {
        let Some(Value::Object(bindings)) = env.get(INTERNAL_CONST_BINDINGS_KEY).cloned() else {
            return;
        };
        env.insert(
            INTERNAL_CONST_BINDINGS_KEY.to_string(),
            Value::Object(Rc::new(RefCell::new(bindings.borrow().clone()))),
        );
    }

    pub(crate) fn discard_pending_listener_updates_from_frames(
        &mut self,
        start: usize,
        local_bindings: &HashSet<String>,
    ) {
        if local_bindings.is_empty() || start >= self.script_runtime.listener_capture_env_stack.len()
        {
            return;
        }
        for frame in &mut self.script_runtime.listener_capture_env_stack[start..] {
            for name in local_bindings {
                frame.pending_env_updates.remove(name);
            }
        }
    }

    pub(crate) fn discard_event_sync_pending_updates_from_frames(&mut self, start: usize) {
        if start >= self.script_runtime.listener_capture_env_stack.len() {
            return;
        }
        for frame in &mut self.script_runtime.listener_capture_env_stack[start..] {
            frame.pending_env_updates.retain(|name, _| {
                Self::event_sync_pending_marker_name(name).is_none()
            });
        }
    }

    pub(crate) fn css_escape_identifier(input: &str) -> String {
        let chars: Vec<char> = input.chars().collect();
        let mut out = String::new();

        for (index, ch) in chars.iter().copied().enumerate() {
            let code = ch as u32;
            let is_digit = ch.is_ascii_digit();
            let is_letter = ch.is_ascii_alphabetic();
            let is_allowed = code >= 0x80 || is_digit || is_letter || matches!(ch, '-' | '_');

            if code == 0 {
                out.push('\u{fffd}');
                continue;
            }

            if (code <= 0x1f)
                || code == 0x7f
                || (index == 0 && is_digit)
                || (index == 1 && is_digit && chars.first() == Some(&'-'))
            {
                out.push('\\');
                out.push_str(&format!("{code:x} "));
                continue;
            }

            if index == 0 && ch == '-' && chars.len() == 1 {
                out.push_str("\\-");
                continue;
            }

            if is_allowed {
                out.push(ch);
            } else {
                out.push('\\');
                out.push(ch);
            }
        }

        out
    }

    pub(crate) fn has_simple_parameter_list(handler: &ScriptHandler) -> bool {
        handler.params.iter().all(|param| {
            !param.is_rest
                && param.default.is_none()
                && !param.name.starts_with("__bt_callback_arg_")
        })
    }

    pub(crate) fn first_suspendable_top_level_await(
        stmts: &[Stmt],
    ) -> Option<(usize, Expr, TopLevelAwaitResumeKind)> {
        stmts
            .iter()
            .enumerate()
            .find_map(|(index, stmt)| match stmt {
                Stmt::Expr(Expr::Await(inner)) => {
                    Some((index, (**inner).clone(), TopLevelAwaitResumeKind::Ignore))
                }
                Stmt::VarDecl { name, kind, expr } => match expr {
                    Expr::Await(inner) => Some((
                        index,
                        (**inner).clone(),
                        TopLevelAwaitResumeKind::Declare {
                            name: name.clone(),
                            kind: kind.clone(),
                        },
                    )),
                    _ => None,
                },
                Stmt::VarAssign { name, op, expr } => match (op, expr) {
                    (VarAssignOp::Assign, Expr::Await(inner)) => Some((
                        index,
                        (**inner).clone(),
                        TopLevelAwaitResumeKind::Assign { name: name.clone() },
                    )),
                    _ => None,
                },
                Stmt::Return {
                    value: Some(Expr::Await(inner)),
                } => Some((index, (**inner).clone(), TopLevelAwaitResumeKind::Return)),
                _ => None,
            })
    }

    pub(crate) fn eval_top_level_await_expr(
        &mut self,
        expr: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<TopLevelAwaitOutcome> {
        let value = self.eval_expr(expr, env, event_param, event)?;
        let promise = self.promise_resolve_value_as_promise(value)?;
        loop {
            let settled = {
                let promise_ref = promise.borrow();
                match &promise_ref.state {
                    PromiseState::Pending => None,
                    PromiseState::Fulfilled(value) => Some(Ok(value.clone())),
                    PromiseState::Rejected(reason) => Some(Err(reason.clone())),
                }
            };
            match settled {
                Some(Ok(value)) => return Ok(TopLevelAwaitOutcome::Resolved(value)),
                Some(Err(reason)) => return Err(Error::ScriptThrown(ThrownValue::new(reason))),
                None => {
                    if !self.scheduler.microtask_queue.is_empty() {
                        self.run_microtask_queue()?;
                        continue;
                    }
                    let ran_timers = self.run_due_timers_internal()?;
                    if ran_timers == 0 {
                        return Ok(TopLevelAwaitOutcome::Pending(promise));
                    }
                }
            }
        }
    }

    pub(crate) fn build_top_level_await_continuation_handler(
        resume: &TopLevelAwaitResumeKind,
        remaining: &[Stmt],
    ) -> ScriptHandler {
        let mut params = Vec::new();
        let mut stmts = Vec::new();

        match resume {
            TopLevelAwaitResumeKind::Ignore => {}
            TopLevelAwaitResumeKind::Declare { name, kind } => {
                params.push(FunctionParam {
                    name: INTERNAL_ASYNC_AWAIT_VALUE_PARAM.to_string(),
                    default: None,
                    is_rest: false,
                });
                stmts.push(Stmt::VarDecl {
                    name: name.clone(),
                    kind: kind.clone(),
                    expr: Expr::Var(INTERNAL_ASYNC_AWAIT_VALUE_PARAM.to_string()),
                });
            }
            TopLevelAwaitResumeKind::Assign { name } => {
                params.push(FunctionParam {
                    name: INTERNAL_ASYNC_AWAIT_VALUE_PARAM.to_string(),
                    default: None,
                    is_rest: false,
                });
                stmts.push(Stmt::VarAssign {
                    name: name.clone(),
                    op: VarAssignOp::Assign,
                    expr: Expr::Var(INTERNAL_ASYNC_AWAIT_VALUE_PARAM.to_string()),
                });
            }
            TopLevelAwaitResumeKind::Return => {
                params.push(FunctionParam {
                    name: INTERNAL_ASYNC_AWAIT_VALUE_PARAM.to_string(),
                    default: None,
                    is_rest: false,
                });
                stmts.push(Stmt::Return {
                    value: Some(Expr::Var(INTERNAL_ASYNC_AWAIT_VALUE_PARAM.to_string())),
                });
                return ScriptHandler { params, stmts };
            }
        }

        stmts.extend(remaining.iter().cloned());
        ScriptHandler { params, stmts }
    }

    pub(crate) fn selective_function_capture_snapshot(
        &mut self,
        env: &HashMap<String, Value>,
        capture_names: &HashSet<String>,
    ) -> HashMap<String, Value> {
        let declared_names = Self::env_local_or_lexical_binding_names(env);
        let mut captured_snapshot = env
            .iter()
            .filter(|(name, _)| Self::is_internal_env_key(name))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<HashMap<_, _>>();
        if let Some(return_slot) = env.get(INTERNAL_RETURN_SLOT).cloned() {
            captured_snapshot.insert(INTERNAL_RETURN_SLOT.to_string(), return_slot);
        }
        for name in capture_names {
            if let Some(value) = env.get(name).cloned() {
                captured_snapshot.insert(name.clone(), value);
            } else if declared_names.contains(name) && !self.has_pending_function_decl(name) {
                captured_snapshot.insert(name.clone(), Value::Undefined);
            }
        }
        self.project_pending_listener_capture_env_updates(&mut captured_snapshot);
        captured_snapshot.retain(|name, _| {
            Self::is_internal_env_key(name)
                || name == INTERNAL_RETURN_SLOT
                || capture_names.contains(name)
        });
        captured_snapshot
    }

    pub(crate) fn function_capture_snapshot(function: &FunctionValue) -> HashMap<String, Value> {
        function
            .captured_env
            .borrow()
            .iter()
            .filter(|(name, _)| {
                Self::is_internal_env_key(name)
                    || *name == INTERNAL_RETURN_SLOT
                    || function.is_class_constructor
                    || function.captured_names.contains(*name)
            })
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect()
    }

    pub(crate) fn expand_capture_names_with_pending_function_decls(
        &self,
        env: &HashMap<String, Value>,
        capture_names: &mut HashSet<String>,
    ) {
        let mut pending = capture_names.iter().cloned().collect::<Vec<_>>();
        while let Some(name) = pending.pop() {
            if let Some(Value::Function(function)) = env.get(&name) {
                for extra in &function.captured_names {
                    if capture_names.insert(extra.clone()) {
                        pending.push(extra.clone());
                    }
                }
                continue;
            }
            if env
                .get(&name)
                .is_some_and(|value| !matches!(value, Value::Undefined))
            {
                continue;
            }
            let Some((handler, _, _)) = self
                .script_runtime
                .pending_function_decls
                .iter()
                .rev()
                .find_map(|scope| scope.get(&name))
            else {
                continue;
            };
            for extra in Self::collect_function_capture_names(handler) {
                if capture_names.insert(extra.clone()) {
                    pending.push(extra);
                }
            }
        }
    }

    pub(crate) fn has_pending_function_decl(&self, name: &str) -> bool {
        self.script_runtime
            .pending_function_decls
            .iter()
            .rev()
            .any(|scope| scope.contains_key(name))
    }

    pub(crate) fn popup_window_receiver_object(
        value: Option<&Value>,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let Some(Value::Object(object)) = value else {
            return Err(Error::ScriptRuntime(
                "TypeError: popup window method called on incompatible receiver".into(),
            ));
        };
        let is_popup_window = {
            let entries = object.borrow();
            matches!(
                Self::object_get_entry(&entries, INTERNAL_POPUP_WINDOW_OBJECT_KEY),
                Some(Value::Bool(true))
            )
        };
        if !is_popup_window {
            return Err(Error::ScriptRuntime(
                "TypeError: popup window method called on incompatible receiver".into(),
            ));
        }
        Ok(object.clone())
    }

    pub(crate) fn popup_document_receiver_object(
        value: Option<&Value>,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let Some(Value::Object(object)) = value else {
            return Err(Error::ScriptRuntime(
                "TypeError: popup document method called on incompatible receiver".into(),
            ));
        };
        let is_popup_document = {
            let entries = object.borrow();
            matches!(
                Self::object_get_entry(&entries, INTERNAL_POPUP_DOCUMENT_OBJECT_KEY),
                Some(Value::Bool(true))
            )
        };
        if !is_popup_document {
            return Err(Error::ScriptRuntime(
                "TypeError: popup document method called on incompatible receiver".into(),
            ));
        }
        Ok(object.clone())
    }
}
