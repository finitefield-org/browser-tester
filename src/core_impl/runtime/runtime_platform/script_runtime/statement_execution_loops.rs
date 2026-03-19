use super::*;

#[path = "statement_execution_basic_loops.rs"]
mod statement_execution_basic_loops;
#[path = "statement_execution_dom_foreach_loops.rs"]
mod statement_execution_dom_foreach_loops;
#[path = "statement_execution_iter_loops.rs"]
mod statement_execution_iter_loops;

impl Harness {
    fn with_loop_label_scope<T>(&mut self, run: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        let loop_labels = self.take_pending_loop_labels();
        self.push_loop_label_scope(loop_labels);
        let result = run(self);
        self.pop_loop_label_scope();
        result
    }

    pub(crate) fn try_execute_loop_stmt(
        &mut self,
        stmt: &Stmt,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<Option<ExecFlow>> {
        match stmt {
            Stmt::ClassListForEach {
                target,
                optional,
                item_var,
                index_var,
                body,
            } => Ok(Some(self.execute_class_list_for_each_stmt(
                target,
                *optional,
                item_var,
                index_var,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::ForEach {
                target,
                selector,
                item_var,
                index_var,
                body,
            } => Ok(Some(self.execute_query_selector_for_each_stmt(
                target,
                selector,
                item_var,
                index_var,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::ArrayForEach { target, callback } => {
                let target_value = env
                    .get(target)
                    .cloned()
                    .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {target}")))?;
                self.execute_array_like_foreach_in_env(target_value, callback, env, event, target)?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::ArrayForEachExpr { target, callback } => {
                let target_value = self.eval_expr(target, env, event_param, event)?;
                self.execute_array_like_foreach_in_env(
                    target_value,
                    callback,
                    env,
                    event,
                    "<expression>",
                )?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::For {
                init,
                cond,
                post,
                body,
            } => Ok(Some(self.execute_for_stmt(
                init,
                cond,
                post,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::ForIn {
                item_var,
                iterable,
                body,
            } => Ok(Some(self.execute_for_in_stmt(
                item_var,
                iterable,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::ForOf {
                item_var,
                iterable,
                body,
            } => Ok(Some(self.execute_for_of_stmt(
                item_var,
                iterable,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::ForAwaitOf {
                item_var,
                iterable,
                body,
            } => Ok(Some(self.execute_for_await_of_stmt(
                item_var,
                iterable,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::While { cond, body } => Ok(Some(self.execute_while_stmt(
                cond,
                body,
                env,
                event_param,
                event,
            )?)),
            Stmt::DoWhile { cond, body } => Ok(Some(self.execute_do_while_stmt(
                cond,
                body,
                env,
                event_param,
                event,
            )?)),
            _ => Ok(None),
        }
    }
}
