use super::*;

impl Harness {
    pub(crate) fn error_to_catch_value(err: Error) -> std::result::Result<Value, Error> {
        match err {
            Error::ScriptThrown(value) => Ok(value.into_value()),
            Error::ScriptRuntime(message) => Ok(Value::String(message)),
            other => Err(other),
        }
    }

    pub(crate) fn bind_catch_binding(
        &self,
        binding: &CatchBinding,
        caught: &Value,
        env: &mut HashMap<String, Value>,
    ) -> Result<Vec<(String, Option<Value>)>> {
        let mut previous = Vec::new();
        let mut seen = HashSet::new();
        let mut remember = |name: &str, env: &HashMap<String, Value>| {
            if seen.insert(name.to_string()) {
                previous.push((name.to_string(), env.get(name).cloned()));
            }
        };

        match binding {
            CatchBinding::Identifier(name) => {
                remember(name, env);
                env.insert(name.clone(), caught.clone());
            }
            CatchBinding::ArrayPattern(pattern) => {
                let values = self.array_like_values_from_value(caught)?;
                for (index, name) in pattern.iter().enumerate() {
                    let Some(name) = name else {
                        continue;
                    };
                    remember(name, env);
                    let value = values.get(index).cloned().unwrap_or(Value::Undefined);
                    env.insert(name.clone(), value);
                }
            }
            CatchBinding::ObjectPattern(pattern) => {
                let Value::Object(entries) = caught else {
                    return Err(Error::ScriptRuntime(
                        "catch object binding requires an object value".into(),
                    ));
                };
                let entries = entries.borrow();
                for (source_key, target_name) in pattern {
                    remember(target_name, env);
                    let value =
                        Self::object_get_entry(&entries, source_key).unwrap_or(Value::Undefined);
                    env.insert(target_name.clone(), value);
                }
            }
        }

        Ok(previous)
    }

    pub(crate) fn restore_catch_binding(
        &self,
        previous: Vec<(String, Option<Value>)>,
        env: &mut HashMap<String, Value>,
    ) {
        for (name, value) in previous {
            if let Some(value) = value {
                env.insert(name, value);
            } else {
                env.remove(&name);
            }
        }
    }

    pub(crate) fn execute_catch_block(
        &mut self,
        catch_binding: &Option<CatchBinding>,
        catch_stmts: &[Stmt],
        caught: Value,
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        let previous = if let Some(binding) = catch_binding {
            self.bind_catch_binding(binding, &caught, env)?
        } else {
            Vec::new()
        };
        let result = self.execute_stmts(catch_stmts, event_param, event, env);
        self.restore_catch_binding(previous, env);
        result
    }

    pub(crate) fn parse_function_constructor_param_names(spec: &str) -> Result<Vec<String>> {
        let mut params = Vec::new();
        for raw in spec.split(',') {
            let raw = raw.trim();
            if raw.is_empty() {
                return Err(Error::ScriptRuntime(
                    "new Function parameter name cannot be empty".into(),
                ));
            }
            if !is_ident(raw) {
                return Err(Error::ScriptRuntime(format!(
                    "new Function parameter name is invalid: {raw}"
                )));
            }
            params.push(raw.to_string());
        }
        Ok(params)
    }

    pub(crate) fn collect_function_decls(stmts: &[Stmt]) -> HashMap<String, (ScriptHandler, bool)> {
        let mut out = HashMap::new();
        for stmt in stmts {
            if let Stmt::FunctionDecl {
                name,
                handler,
                is_async,
            } = stmt
            {
                out.insert(name.clone(), (handler.clone(), *is_async));
            }
        }
        out
    }

    pub(crate) fn collect_function_scope_bindings(handler: &ScriptHandler) -> HashSet<String> {
        let mut bindings = HashSet::new();
        for param in &handler.params {
            bindings.insert(param.name.clone());
        }
        Self::collect_scope_bindings_from_stmts(&handler.stmts, &mut bindings);
        bindings
    }

    pub(crate) fn collect_scope_bindings_from_stmts(stmts: &[Stmt], out: &mut HashSet<String>) {
        for stmt in stmts {
            Self::collect_scope_bindings_from_stmt(stmt, out);
        }
    }

    pub(crate) fn collect_scope_bindings_from_stmt(stmt: &Stmt, out: &mut HashSet<String>) {
        match stmt {
            Stmt::VarDecl { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::FunctionDecl { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::ForEach {
                item_var,
                index_var,
                body,
                ..
            }
            | Stmt::ClassListForEach {
                item_var,
                index_var,
                body,
                ..
            } => {
                out.insert(item_var.clone());
                if let Some(index_var) = index_var {
                    out.insert(index_var.clone());
                }
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::For { init, body, .. } => {
                if let Some(init) = init {
                    Self::collect_scope_bindings_from_stmt(init, out);
                }
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::ForIn { item_var, body, .. } | Stmt::ForOf { item_var, body, .. } => {
                out.insert(item_var.clone());
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::DoWhile { body, .. } | Stmt::While { body, .. } => {
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::Try {
                try_stmts,
                catch_binding,
                catch_stmts,
                finally_stmts,
            } => {
                Self::collect_scope_bindings_from_stmts(try_stmts, out);
                if let Some(catch_binding) = catch_binding {
                    Self::collect_scope_bindings_from_catch_binding(catch_binding, out);
                }
                if let Some(catch_stmts) = catch_stmts {
                    Self::collect_scope_bindings_from_stmts(catch_stmts, out);
                }
                if let Some(finally_stmts) = finally_stmts {
                    Self::collect_scope_bindings_from_stmts(finally_stmts, out);
                }
            }
            Stmt::If {
                then_stmts,
                else_stmts,
                ..
            } => {
                Self::collect_scope_bindings_from_stmts(then_stmts, out);
                Self::collect_scope_bindings_from_stmts(else_stmts, out);
            }
            _ => {}
        }
    }

    pub(crate) fn collect_scope_bindings_from_catch_binding(
        binding: &CatchBinding,
        out: &mut HashSet<String>,
    ) {
        match binding {
            CatchBinding::Identifier(name) => {
                out.insert(name.clone());
            }
            CatchBinding::ArrayPattern(pattern) => {
                for entry in pattern.iter().flatten() {
                    out.insert(entry.clone());
                }
            }
            CatchBinding::ObjectPattern(pattern) => {
                for (_, target) in pattern {
                    out.insert(target.clone());
                }
            }
        }
    }

    pub(crate) fn resolve_pending_function_decl(
        &mut self,
        name: &str,
        env: &HashMap<String, Value>,
    ) -> Option<Value> {
        let mut resolved = None;
        for scope in self.script_runtime.pending_function_decls.iter().rev() {
            let Some((handler, is_async)) = scope.get(name) else {
                continue;
            };
            resolved = Some((handler.clone(), *is_async));
            break;
        }
        let (handler, is_async) = resolved?;
        Some(self.make_function_value(handler, env, false, is_async, false))
    }

    pub(crate) fn sync_listener_capture_env_if_shared(&mut self, env: &HashMap<String, Value>) {
        let Some(frame) = self.script_runtime.listener_capture_env_stack.last() else {
            return;
        };
        let Some(shared_env) = frame.shared_env.as_ref() else {
            return;
        };
        if Rc::strong_count(shared_env) > 1 {
            *shared_env.borrow_mut() = ScriptEnv::from_snapshot(env);
        }
    }
}
