use super::*;

impl Harness {
    pub(super) fn execute_for_stmt(
        &mut self,
        init: &[Stmt],
        cond: &Option<Expr>,
        post: &[Stmt],
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let previous_init_lexical = self.collect_direct_block_lexical_bindings(init, env);
        let result = self.with_loop_label_scope(|this| {
            if !init.is_empty() {
                match this.execute_stmts_with_pending_scope(init, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                    ExecFlow::Break(label) => return Err(Self::break_flow_error(&label)),
                    ExecFlow::ContinueLoop(label) => {
                        return Err(Self::continue_flow_error(&label));
                    }
                }
            }

            loop {
                let should_run = if let Some(cond) = cond {
                    this.eval_expr(cond, env, event_param, event)?.truthy()
                } else {
                    true
                };
                if !should_run {
                    break;
                }

                match this.execute_stmts_with_pending_scope(body, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::ContinueLoop(label) => {
                        if this.loop_should_consume_continue(&label) {
                            if !post.is_empty() {
                                match this.execute_stmts_with_pending_scope(
                                    post,
                                    event_param,
                                    event,
                                    env,
                                    false,
                                )? {
                                    ExecFlow::Continue => {}
                                    ExecFlow::Return => return Ok(ExecFlow::Return),
                                    ExecFlow::Break(_) | ExecFlow::ContinueLoop(_) => {
                                        return Err(Error::ScriptRuntime(
                                            "invalid loop control in post expression".into(),
                                        ));
                                    }
                                }
                            }
                            continue;
                        }
                        return Ok(ExecFlow::ContinueLoop(label));
                    }
                    ExecFlow::Break(label) => {
                        if this.loop_should_consume_break(&label) {
                            break;
                        }
                        return Ok(ExecFlow::Break(label));
                    }
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                }
                if !post.is_empty() {
                    match this.execute_stmts_with_pending_scope(
                        post,
                        event_param,
                        event,
                        env,
                        false,
                    )? {
                        ExecFlow::Continue => {}
                        ExecFlow::Return => return Ok(ExecFlow::Return),
                        ExecFlow::Break(_) | ExecFlow::ContinueLoop(_) => {
                            return Err(Error::ScriptRuntime(
                                "invalid loop control in post expression".into(),
                            ));
                        }
                    }
                }
            }
            Ok(ExecFlow::Continue)
        });
        self.restore_block_lexical_bindings(previous_init_lexical, env);
        result
    }

    pub(super) fn execute_while_stmt(
        &mut self,
        cond: &Expr,
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        self.with_loop_label_scope(|this| {
            while this.eval_expr(cond, env, event_param, event)?.truthy() {
                match this.execute_stmts_with_pending_scope(body, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::ContinueLoop(label) => {
                        if this.loop_should_consume_continue(&label) {
                            continue;
                        }
                        return Ok(ExecFlow::ContinueLoop(label));
                    }
                    ExecFlow::Break(label) => {
                        if this.loop_should_consume_break(&label) {
                            break;
                        }
                        return Ok(ExecFlow::Break(label));
                    }
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                }
            }
            Ok(ExecFlow::Continue)
        })
    }

    pub(super) fn execute_do_while_stmt(
        &mut self,
        cond: &Expr,
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        self.with_loop_label_scope(|this| {
            loop {
                match this.execute_stmts_with_pending_scope(body, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::ContinueLoop(label) => {
                        if !this.loop_should_consume_continue(&label) {
                            return Ok(ExecFlow::ContinueLoop(label));
                        }
                    }
                    ExecFlow::Break(label) => {
                        if this.loop_should_consume_break(&label) {
                            break;
                        }
                        return Ok(ExecFlow::Break(label));
                    }
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                }
                if !this.eval_expr(cond, env, event_param, event)?.truthy() {
                    break;
                }
            }
            Ok(ExecFlow::Continue)
        })
    }
}
