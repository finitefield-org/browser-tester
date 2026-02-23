use super::*;

impl Harness {
    fn direct_decl_names_for_catch_conflict(stmt: &Stmt) -> Vec<(String, bool)> {
        match stmt {
            Stmt::ImportDecl {
                default_binding,
                namespace_binding,
                named_bindings,
                ..
            } => {
                let mut out = Vec::new();
                if let Some(name) = default_binding {
                    out.push((name.clone(), false));
                }
                if let Some(name) = namespace_binding {
                    out.push((name.clone(), false));
                }
                out.extend(
                    named_bindings
                        .iter()
                        .map(|binding| (binding.local.clone(), false)),
                );
                out
            }
            Stmt::VarDecl { name, kind, .. } => {
                vec![(name.clone(), matches!(kind, VarDeclKind::Var))]
            }
            Stmt::FunctionDecl { name, .. } => vec![(name.clone(), false)],
            Stmt::ClassDecl { name, .. } => vec![(name.clone(), false)],
            Stmt::ExportDecl { declaration, .. } => {
                Self::direct_decl_names_for_catch_conflict(declaration)
            }
            Stmt::ArrayDestructureAssign {
                targets,
                decl_kind: Some(kind),
                ..
            } => {
                let is_var = matches!(kind, VarDeclKind::Var);
                targets
                    .iter()
                    .flatten()
                    .cloned()
                    .map(|name| (name, is_var))
                    .collect()
            }
            Stmt::ObjectDestructureAssign {
                bindings,
                decl_kind: Some(kind),
                ..
            } => {
                let is_var = matches!(kind, VarDeclKind::Var);
                bindings
                    .iter()
                    .map(|(_, target)| (target.clone(), is_var))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    fn catch_binding_names(binding: &CatchBinding) -> HashSet<String> {
        match binding {
            CatchBinding::Identifier(name) => HashSet::from([name.clone()]),
            CatchBinding::ArrayPattern(pattern) => pattern.iter().flatten().cloned().collect(),
            CatchBinding::ObjectPattern(pattern) => {
                pattern.iter().map(|(_, target)| target.clone()).collect()
            }
        }
    }

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
    ) -> Result<Vec<(String, Option<Value>, bool)>> {
        let mut previous = Vec::new();
        let mut seen = HashSet::new();
        let mut remember = |name: &str, env: &HashMap<String, Value>, is_const: bool| {
            if seen.insert(name.to_string()) {
                previous.push((name.to_string(), env.get(name).cloned(), is_const));
            }
        };

        match binding {
            CatchBinding::Identifier(name) => {
                remember(name, env, self.is_const_binding(env, name));
                env.insert(name.clone(), caught.clone());
                self.set_const_binding(env, name, false);
            }
            CatchBinding::ArrayPattern(pattern) => {
                let values = self.array_like_values_from_value(caught)?;
                for (index, name) in pattern.iter().enumerate() {
                    let Some(name) = name else {
                        continue;
                    };
                    remember(name, env, self.is_const_binding(env, name));
                    let value = values.get(index).cloned().unwrap_or(Value::Undefined);
                    env.insert(name.clone(), value);
                    self.set_const_binding(env, name, false);
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
                    remember(target_name, env, self.is_const_binding(env, target_name));
                    let value =
                        Self::object_get_entry(&entries, source_key).unwrap_or(Value::Undefined);
                    env.insert(target_name.clone(), value);
                    self.set_const_binding(env, target_name, false);
                }
            }
        }

        Ok(previous)
    }

    pub(crate) fn restore_catch_binding(
        &self,
        previous: Vec<(String, Option<Value>, bool)>,
        env: &mut HashMap<String, Value>,
    ) {
        for (name, value, was_const) in previous {
            if let Some(value) = value {
                env.insert(name.clone(), value);
            } else {
                env.remove(&name);
            }
            self.set_const_binding(env, &name, was_const);
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
        if let Some(binding) = catch_binding {
            let direct_scope_stmts = match catch_stmts {
                [Stmt::Block { stmts }] => stmts.as_slice(),
                _ => catch_stmts,
            };
            let occupied_names = Self::catch_binding_names(binding);
            let allow_var_shadow_for_simple_identifier =
                matches!(binding, CatchBinding::Identifier(_));
            let simple_identifier_name = match binding {
                CatchBinding::Identifier(name) => Some(name.as_str()),
                _ => None,
            };
            for stmt in direct_scope_stmts {
                for (name, is_var_decl) in Self::direct_decl_names_for_catch_conflict(stmt) {
                    if allow_var_shadow_for_simple_identifier
                        && is_var_decl
                        && Some(name.as_str()) == simple_identifier_name
                    {
                        continue;
                    }
                    if occupied_names.contains(&name) {
                        return Err(Error::ScriptRuntime(format!(
                            "Identifier '{name}' has already been declared"
                        )));
                    }
                }
            }
        }
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

    pub(crate) fn collect_function_decls(
        stmts: &[Stmt],
    ) -> HashMap<String, (ScriptHandler, bool, bool)> {
        let mut out = HashMap::new();
        for stmt in stmts {
            match stmt {
                Stmt::FunctionDecl {
                    name,
                    handler,
                    is_async,
                    is_generator,
                } => {
                    out.insert(name.clone(), (handler.clone(), *is_async, *is_generator));
                }
                Stmt::ExportDecl { declaration, .. } => {
                    if let Stmt::FunctionDecl {
                        name,
                        handler,
                        is_async,
                        is_generator,
                    } = declaration.as_ref()
                    {
                        out.insert(name.clone(), (handler.clone(), *is_async, *is_generator));
                    }
                }
                _ => {}
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
            Stmt::ImportDecl {
                default_binding,
                namespace_binding,
                named_bindings,
                ..
            } => {
                if let Some(name) = default_binding {
                    out.insert(name.clone());
                }
                if let Some(name) = namespace_binding {
                    out.insert(name.clone());
                }
                for binding in named_bindings {
                    out.insert(binding.local.clone());
                }
            }
            Stmt::ArrayDestructureAssign {
                targets,
                decl_kind: Some(_),
                ..
            } => {
                for name in targets.iter().flatten() {
                    out.insert(name.clone());
                }
            }
            Stmt::ObjectDestructureAssign {
                bindings,
                decl_kind: Some(_),
                ..
            } => {
                for (_, target) in bindings {
                    out.insert(target.clone());
                }
            }
            Stmt::FunctionDecl { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::ClassDecl { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::ExportDecl { declaration, .. } => {
                Self::collect_scope_bindings_from_stmt(declaration, out);
            }
            Stmt::Label { stmt, .. } => {
                Self::collect_scope_bindings_from_stmt(stmt, out);
            }
            Stmt::Block { stmts } => {
                Self::collect_scope_bindings_from_stmts(stmts, out);
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
                for init in init {
                    Self::collect_scope_bindings_from_stmt(init, out);
                }
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::ForIn { item_var, body, .. }
            | Stmt::ForOf { item_var, body, .. }
            | Stmt::ForAwaitOf { item_var, body, .. } => {
                out.insert(item_var.clone());
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::DoWhile { body, .. } | Stmt::While { body, .. } => {
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::Switch { clauses, .. } => {
                for clause in clauses {
                    Self::collect_scope_bindings_from_stmts(&clause.stmts, out);
                }
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
            let Some((handler, is_async, is_generator)) = scope.get(name) else {
                continue;
            };
            resolved = Some((handler.clone(), *is_async, *is_generator));
            break;
        }
        let (handler, is_async, is_generator) = resolved?;
        Some(self.make_function_value(handler, env, false, is_async, is_generator, false, false))
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
