use super::*;

mod dom_methods_listeners_dispatch;
mod dom_timer_mutations;
mod dom_tree_attr_mutations;

impl Harness {
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
