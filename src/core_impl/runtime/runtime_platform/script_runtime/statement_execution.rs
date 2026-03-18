use super::*;

impl Harness {
    const EXEC_STMTS_STACK_RED_ZONE: usize = 64 * 1024;
    const EXEC_STMTS_STACK_SIZE: usize = 32 * 1024 * 1024;

    pub(crate) fn execute_stmts(
        &mut self,
        stmts: &[Stmt],
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        self.execute_stmts_with_pending_scope(stmts, event_param, event, env, true)
    }

    pub(crate) fn execute_stmts_with_pending_scope(
        &mut self,
        stmts: &[Stmt],
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
        inherit_outer_pending: bool,
    ) -> Result<ExecFlow> {
        stacker::maybe_grow(
            Self::EXEC_STMTS_STACK_RED_ZONE,
            Self::EXEC_STMTS_STACK_SIZE,
            || self.execute_stmts_impl(stmts, event_param, event, env, inherit_outer_pending),
        )
    }

    fn execute_stmts_impl(
        &mut self,
        stmts: &[Stmt],
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
        inherit_outer_pending: bool,
    ) -> Result<ExecFlow> {
        let saved_expression_env_overrides =
            std::mem::take(&mut self.script_runtime.expression_env_overrides);
        let previous_pending_scope_start = (!inherit_outer_pending).then(|| {
            env.insert(
                INTERNAL_PENDING_SCOPE_START_KEY.to_string(),
                Value::Number(self.script_runtime.listener_capture_env_stack.len() as i64),
            )
        });
        let pending = Self::collect_function_decls(stmts);
        let pending_scope_start = self.push_pending_function_decl_scope(pending);
        let scope_depth = Self::env_scope_depth(env);
        let prev_top_level_lexical_bindings =
            env.get(INTERNAL_TOP_LEVEL_LEXICAL_BINDINGS_KEY).cloned();
        if scope_depth == 0 {
            let mut lexical_binding_names = Self::env_top_level_lexical_binding_names(env)
                .into_iter()
                .collect::<Vec<_>>();
            lexical_binding_names.extend(Self::collect_direct_lexical_binding_names(stmts));
            lexical_binding_names.sort();
            lexical_binding_names.dedup();
            if !lexical_binding_names.is_empty() {
                env.insert(
                    INTERNAL_TOP_LEVEL_LEXICAL_BINDINGS_KEY.to_string(),
                    Self::new_array_value(
                        lexical_binding_names
                            .into_iter()
                            .map(Value::String)
                            .collect(),
                    ),
                );
            }
            self.ensure_top_level_global_sync_names(stmts, env);
        }
        self.script_runtime
            .listener_capture_env_stack
            .push(ListenerCaptureFrame::default());

        let result = (|| -> Result<ExecFlow> {
            Self::validate_const_redeclarations(stmts)?;
            self.bind_hoisted_import_decls(stmts, env)?;
            self.hoist_var_declarations(stmts, env);
            let mut pending_tdz_bindings = Self::collect_direct_tdz_binding_names(stmts);
            self.push_tdz_scope_frame(pending_tdz_bindings.clone());
            let mut initialized_var_bindings = HashSet::new();
            let flow_result = (|| -> Result<ExecFlow> {
                for stmt in stmts {
                    self.apply_expression_env_overrides_to_env(env);
                    self.apply_pending_listener_capture_env_updates(env);
                    self.sync_top_level_env_from_runtime(env);
                    self.sync_listener_capture_env_if_shared(env);
                    if let Some(flow) = self.try_execute_declaration_stmt(
                        stmt,
                        &mut pending_tdz_bindings,
                        &mut initialized_var_bindings,
                        env,
                        event_param,
                        event,
                    )? {
                        match flow {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                        continue;
                    }
                    if let Some(flow) = self.try_execute_assignment_stmt(
                        stmt,
                        &mut pending_tdz_bindings,
                        env,
                        event_param,
                        event,
                    )? {
                        match flow {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                        continue;
                    }
                    if let Some(flow) =
                        self.try_execute_control_flow_stmt(stmt, env, event_param, event)?
                    {
                        match flow {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                        continue;
                    }
                    if let Some(flow) = self.try_execute_loop_stmt(stmt, env, event_param, event)? {
                        match flow {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                        continue;
                    }
                    if let Some(flow) =
                        self.try_execute_dom_mutation_stmt(stmt, env, event_param, event)?
                    {
                        match flow {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                        continue;
                    }
                    if let Some(flow) =
                        self.try_execute_dom_assign_stmt(stmt, env, event_param, event)?
                    {
                        match flow {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                        continue;
                    }
                    match stmt {
                        Stmt::ImportDecl { .. }
                        | Stmt::VarDecl { .. }
                        | Stmt::FunctionDecl { .. }
                        | Stmt::ClassDecl { .. }
                        | Stmt::ExportDecl { .. }
                        | Stmt::ExportNamed { .. }
                        | Stmt::ExportDefaultExpr { .. }
                        | Stmt::VarAssign { .. }
                        | Stmt::PrivateAssign { .. }
                        | Stmt::VarUpdate { .. }
                        | Stmt::ArrayDestructureAssign { .. }
                        | Stmt::ObjectDestructureAssign { .. }
                        | Stmt::ObjectAssign { .. }
                        | Stmt::FormDataAppend { .. }
                        | Stmt::Block { .. }
                        | Stmt::Label { .. }
                        | Stmt::Switch { .. }
                        | Stmt::If { .. }
                        | Stmt::Try { .. }
                        | Stmt::Throw { .. }
                        | Stmt::Return { .. }
                        | Stmt::Empty
                        | Stmt::Debugger
                        | Stmt::Break { .. }
                        | Stmt::Continue { .. }
                        | Stmt::EventCall { .. }
                        | Stmt::ClassListForEach { .. }
                        | Stmt::ForEach { .. }
                        | Stmt::ArrayForEach { .. }
                        | Stmt::ArrayForEachExpr { .. }
                        | Stmt::For { .. }
                        | Stmt::ForIn { .. }
                        | Stmt::ForOf { .. }
                        | Stmt::ForAwaitOf { .. }
                        | Stmt::While { .. }
                        | Stmt::DoWhile { .. }
                        | Stmt::ClassListCall { .. }
                        | Stmt::DomSetAttribute { .. }
                        | Stmt::DomRemoveAttribute { .. }
                        | Stmt::NodeTreeMutation { .. }
                        | Stmt::InsertAdjacentElement { .. }
                        | Stmt::InsertAdjacentText { .. }
                        | Stmt::InsertAdjacentHTML { .. }
                        | Stmt::SetTimeout { .. }
                        | Stmt::SetInterval { .. }
                        | Stmt::QueueMicrotask { .. }
                        | Stmt::ClearTimeout { .. }
                        | Stmt::NodeRemove { .. }
                        | Stmt::ListenerMutation { .. }
                        | Stmt::DomMethodCall { .. }
                        | Stmt::DispatchEvent { .. }
                        | Stmt::DomAssign { .. } => unreachable!(),
                        Stmt::Expr(expr) => {
                            let _ = self.eval_expr(expr, env, event_param, event)?;
                        }
                    }
                }

                Ok(ExecFlow::Continue)
            })();

            self.apply_pending_listener_capture_env_updates(env);
            self.pop_tdz_scope_frame();
            flow_result
        })();

        self.script_runtime.listener_capture_env_stack.pop();
        self.restore_pending_function_decl_scopes(pending_scope_start);
        if scope_depth == 0 {
            match prev_top_level_lexical_bindings {
                Some(value) => {
                    env.insert(INTERNAL_TOP_LEVEL_LEXICAL_BINDINGS_KEY.to_string(), value);
                }
                None => {
                    env.remove(INTERNAL_TOP_LEVEL_LEXICAL_BINDINGS_KEY);
                }
            }
        }
        if let Some(previous_pending_scope_start) = previous_pending_scope_start {
            match previous_pending_scope_start {
                Some(value) => {
                    env.insert(INTERNAL_PENDING_SCOPE_START_KEY.to_string(), value);
                }
                None => {
                    env.remove(INTERNAL_PENDING_SCOPE_START_KEY);
                }
            }
        }
        self.script_runtime.expression_env_overrides = saved_expression_env_overrides;
        result
    }

    pub(crate) fn sync_top_level_env_from_runtime(&mut self, env: &mut HashMap<String, Value>) {
        if Self::env_scope_depth(env) != 0 {
            return;
        }

        let runtime_snapshot = self.script_runtime.env.to_map();
        let lexical_bindings = Self::env_top_level_lexical_binding_names(env);
        let Some(Value::Array(sync_names)) = env.get(INTERNAL_GLOBAL_SYNC_NAMES_KEY) else {
            return;
        };
        let sync_names = sync_names
            .borrow()
            .iter()
            .filter_map(|entry| match entry {
                Value::String(name) => Some(name.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        for name in sync_names {
            if Self::is_internal_env_key(&name) || lexical_bindings.contains(&name) {
                continue;
            }
            let Some(runtime_value) = runtime_snapshot.get(&name).cloned() else {
                continue;
            };
            let should_update = match env.get(&name) {
                Some(current) => !self.strict_equal(current, &runtime_value),
                None => true,
            };
            if should_update {
                env.insert(name, runtime_value);
            }
        }
    }
}
