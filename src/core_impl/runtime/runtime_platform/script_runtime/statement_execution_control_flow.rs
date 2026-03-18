use super::*;

impl Harness {
    fn execute_block_stmt(
        &mut self,
        stmts: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let previous = self.collect_direct_block_lexical_bindings(stmts, env);
        let flow = self.execute_stmts_with_pending_scope(stmts, event_param, event, env, false);
        self.restore_block_lexical_bindings(previous, env);
        flow
    }

    fn execute_label_stmt(
        &mut self,
        name: &str,
        stmt: &Stmt,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let mut labels = vec![name.to_string()];
        let mut target = stmt;
        while let Stmt::Label { name, stmt } = target {
            labels.push(name.clone());
            target = stmt.as_ref();
        }

        if Self::is_iteration_stmt(target) {
            self.script_runtime.pending_loop_labels.push(labels);
            self.execute_stmts_with_pending_scope(
                std::slice::from_ref(target),
                event_param,
                event,
                env,
                false,
            )
        } else {
            match self.execute_stmts_with_pending_scope(
                std::slice::from_ref(target),
                event_param,
                event,
                env,
                false,
            )? {
                ExecFlow::Continue => Ok(ExecFlow::Continue),
                ExecFlow::Break(Some(label))
                    if labels.iter().any(|candidate| candidate == &label) =>
                {
                    Ok(ExecFlow::Continue)
                }
                ExecFlow::ContinueLoop(Some(label))
                    if labels.iter().any(|candidate| candidate == &label) =>
                {
                    Err(Error::ScriptRuntime(format!(
                        "continue statement: '{label}' does not denote an iteration statement"
                    )))
                }
                flow => Ok(flow),
            }
        }
    }

    fn execute_switch_stmt(
        &mut self,
        expr: &Expr,
        clauses: &[SwitchClause],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let switch_value = self.eval_expr(expr, env, event_param, event)?;
        let all_clause_stmts = clauses
            .iter()
            .flat_map(|clause| clause.stmts.iter().cloned())
            .collect::<Vec<_>>();
        Self::validate_const_redeclarations(&all_clause_stmts)?;
        let previous = self.collect_direct_block_lexical_bindings(&all_clause_stmts, env);

        let switch_result = (|| -> Result<ExecFlow> {
            let pending_switch_tdz_bindings =
                Self::collect_direct_tdz_binding_names(&all_clause_stmts);
            self.push_tdz_scope_frame(pending_switch_tdz_bindings);

            let mut default_index = None;
            let mut matched_index = None;

            for (index, clause) in clauses.iter().enumerate() {
                if let Some(test) = &clause.test {
                    let case_value = self.eval_expr(test, env, event_param, event)?;
                    if self.strict_equal(&switch_value, &case_value) {
                        matched_index = Some(index);
                        break;
                    }
                } else if default_index.is_none() {
                    default_index = Some(index);
                }
            }

            if let Some(start_index) = matched_index.or(default_index) {
                let mut selected_stmts = Vec::new();
                for clause in clauses.iter().skip(start_index) {
                    selected_stmts.extend(clause.stmts.iter().cloned());
                }
                match self.execute_stmts_with_pending_scope(
                    &selected_stmts,
                    event_param,
                    event,
                    env,
                    false,
                )? {
                    ExecFlow::Continue => {}
                    ExecFlow::Break(label) => {
                        if label.is_some() {
                            return Ok(ExecFlow::Break(label));
                        }
                    }
                    flow => return Ok(flow),
                }
            }

            Ok(ExecFlow::Continue)
        })();

        self.pop_tdz_scope_frame();
        self.restore_block_lexical_bindings(previous, env);
        switch_result
    }

    fn execute_if_stmt(
        &mut self,
        cond: &Expr,
        then_stmts: &[Stmt],
        else_stmts: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let cond = self.eval_expr(cond, env, event_param, event)?;
        if cond.truthy() {
            self.execute_stmts_with_pending_scope(then_stmts, event_param, event, env, false)
        } else {
            self.execute_stmts_with_pending_scope(else_stmts, event_param, event, env, false)
        }
    }

    fn execute_try_stmt(
        &mut self,
        try_stmts: &[Stmt],
        catch_binding: &Option<CatchBinding>,
        catch_stmts: &Option<Vec<Stmt>>,
        finally_stmts: &Option<Vec<Stmt>>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let mut completion =
            self.execute_stmts_with_pending_scope(try_stmts, event_param, event, env, false);

        if let Err(err) = completion {
            if let Some(catch_stmts) = catch_stmts {
                let caught = Self::error_to_catch_value(err)?;
                completion = self.execute_catch_block(
                    catch_binding,
                    catch_stmts,
                    caught,
                    event_param,
                    event,
                    env,
                );
            } else {
                completion = Err(err);
            }
        }

        let try_return_slot = if matches!(completion, Ok(ExecFlow::Return)) {
            env.get(INTERNAL_RETURN_SLOT).cloned()
        } else {
            None
        };

        if let Some(finally_stmts) = finally_stmts {
            match self.execute_stmts_with_pending_scope(
                finally_stmts,
                event_param,
                event,
                env,
                false,
            ) {
                Ok(ExecFlow::Continue) => {}
                Ok(flow) => return Ok(flow),
                Err(err) => return Err(err),
            }
        }

        if matches!(completion, Ok(ExecFlow::Return)) {
            if let Some(value) = try_return_slot {
                env.insert(INTERNAL_RETURN_SLOT.to_string(), value);
            }
        }

        completion
    }

    fn execute_event_call_stmt(
        &mut self,
        event_var: &str,
        method: EventMethod,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) {
        if let Some(param) = event_param {
            if param == event_var {
                if let Some(Value::Object(event_object)) = env.get(event_var) {
                    let mut entries = event_object.borrow_mut();
                    match method {
                        EventMethod::PreventDefault => {
                            if event.cancelable {
                                Self::object_set_entry(
                                    &mut entries,
                                    "defaultPrevented".to_string(),
                                    Value::Bool(true),
                                );
                            }
                        }
                        EventMethod::StopPropagation => {
                            Self::object_set_entry(
                                &mut entries,
                                INTERNAL_EVENT_STOP_PROPAGATION_KEY.to_string(),
                                Value::Bool(true),
                            );
                        }
                        EventMethod::StopImmediatePropagation => {
                            Self::object_set_entry(
                                &mut entries,
                                INTERNAL_EVENT_STOP_PROPAGATION_KEY.to_string(),
                                Value::Bool(true),
                            );
                            Self::object_set_entry(
                                &mut entries,
                                INTERNAL_EVENT_STOP_IMMEDIATE_PROPAGATION_KEY.to_string(),
                                Value::Bool(true),
                            );
                        }
                    }
                }
                match method {
                    EventMethod::PreventDefault => {
                        if event.cancelable {
                            event.default_prevented = true;
                        }
                    }
                    EventMethod::StopPropagation => {
                        event.propagation_stopped = true;
                    }
                    EventMethod::StopImmediatePropagation => {
                        event.immediate_propagation_stopped = true;
                        event.propagation_stopped = true;
                    }
                }
            }
        }
    }

    pub(crate) fn try_execute_control_flow_stmt(
        &mut self,
        stmt: &Stmt,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<Option<ExecFlow>> {
        match stmt {
            Stmt::Block { stmts } => Ok(Some(self.execute_block_stmt(
                stmts,
                env,
                event_param,
                event,
            )?)),
            Stmt::Label { name, stmt } => Ok(Some(self.execute_label_stmt(
                name,
                stmt,
                env,
                event_param,
                event,
            )?)),
            Stmt::Switch { expr, clauses } => Ok(Some(self.execute_switch_stmt(
                expr,
                clauses,
                env,
                event_param,
                event,
            )?)),
            Stmt::If {
                cond,
                then_stmts,
                else_stmts,
            } => Ok(Some(self.execute_if_stmt(
                cond,
                then_stmts,
                else_stmts,
                env,
                event_param,
                event,
            )?)),
            Stmt::Try {
                try_stmts,
                catch_binding,
                catch_stmts,
                finally_stmts,
            } => Ok(Some(self.execute_try_stmt(
                try_stmts,
                catch_binding,
                catch_stmts,
                finally_stmts,
                env,
                event_param,
                event,
            )?)),
            Stmt::Throw { value } => {
                let thrown = self.eval_expr(value, env, event_param, event)?;
                Err(Error::ScriptThrown(ThrownValue::new(thrown)))
            }
            Stmt::Return { value } => {
                let return_value = if let Some(value) = value {
                    self.eval_expr(value, env, event_param, event)?
                } else {
                    Value::Undefined
                };
                env.insert(INTERNAL_RETURN_SLOT.to_string(), return_value);
                Ok(Some(ExecFlow::Return))
            }
            Stmt::Empty | Stmt::Debugger => Ok(Some(ExecFlow::Continue)),
            Stmt::Break { label } => Ok(Some(ExecFlow::Break(label.clone()))),
            Stmt::Continue { label } => Ok(Some(ExecFlow::ContinueLoop(label.clone()))),
            Stmt::EventCall { event_var, method } => {
                self.execute_event_call_stmt(event_var, *method, env, event_param, event);
                Ok(Some(ExecFlow::Continue))
            }
            _ => Ok(None),
        }
    }
}
