use super::*;

impl Harness {
    pub(super) fn execute_class_list_for_each_stmt(
        &mut self,
        target: &DomQuery,
        optional: bool,
        item_var: &str,
        index_var: &Option<String>,
        body: &[Stmt],
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
        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
        let prev_item = env.get(item_var).cloned();
        let prev_item_const = self.is_const_binding(env, item_var);
        let prev_index = index_var.as_ref().and_then(|var| env.get(var).cloned());
        let prev_index_const = index_var
            .as_ref()
            .is_some_and(|name| self.is_const_binding(env, name));
        let prev_local_bindings = env.get(INTERNAL_LOCAL_BINDINGS_KEY).cloned();
        let mut local_binding_names = prev_local_bindings
            .as_ref()
            .and_then(|value| match value {
                Value::Array(bindings) => Some(
                    bindings
                        .borrow()
                        .iter()
                        .filter_map(|entry| match entry {
                            Value::String(name) => Some(name.clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        if !local_binding_names.iter().any(|name| name == item_var) {
            local_binding_names.push(item_var.to_string());
        }
        if let Some(index_var) = index_var {
            if !local_binding_names.iter().any(|name| name == index_var) {
                local_binding_names.push(index_var.clone());
            }
        }
        local_binding_names.sort();
        local_binding_names.dedup();
        env.insert(
            INTERNAL_LOCAL_BINDINGS_KEY.to_string(),
            Self::new_array_value(local_binding_names.into_iter().map(Value::String).collect()),
        );
        self.set_const_binding(env, item_var, false);
        if let Some(index_var) = index_var {
            self.set_const_binding(env, index_var, false);
        }

        let loop_result = (|| -> Result<ExecFlow> {
            for (idx, class_name) in classes.iter().enumerate() {
                let item_value = Value::String(class_name.clone());
                env.insert(item_var.to_string(), item_value.clone());
                self.sync_global_binding_if_needed(env, item_var, &item_value);
                if let Some(index_var) = index_var {
                    let index_value = Value::Number(idx as i64);
                    env.insert(index_var.clone(), index_value.clone());
                    self.sync_global_binding_if_needed(env, index_var, &index_value);
                }
                match self.execute_stmts_with_pending_scope(body, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::Break(None) => break,
                    ExecFlow::Break(label) => return Ok(ExecFlow::Break(label)),
                    ExecFlow::ContinueLoop(None) => continue,
                    ExecFlow::ContinueLoop(label) => return Ok(ExecFlow::ContinueLoop(label)),
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                }
            }
            Ok(ExecFlow::Continue)
        })();

        if let Some(prev) = prev_item {
            env.insert(item_var.to_string(), prev.clone());
            self.sync_global_binding_if_needed(env, item_var, &prev);
        } else {
            env.remove(item_var);
        }
        self.set_const_binding(env, item_var, prev_item_const);
        if let Some(index_var) = index_var {
            if let Some(prev) = prev_index {
                env.insert(index_var.clone(), prev.clone());
                self.sync_global_binding_if_needed(env, index_var, &prev);
            } else {
                env.remove(index_var);
            }
            self.set_const_binding(env, index_var, prev_index_const);
        }
        match prev_local_bindings {
            Some(value) => {
                env.insert(INTERNAL_LOCAL_BINDINGS_KEY.to_string(), value);
            }
            None => {
                env.remove(INTERNAL_LOCAL_BINDINGS_KEY);
            }
        }
        loop_result
    }

    pub(super) fn execute_query_selector_for_each_stmt(
        &mut self,
        target: &Option<DomQuery>,
        selector: &str,
        item_var: &str,
        index_var: &Option<String>,
        body: &[Stmt],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let items = if let Some(target) = target {
            match self.resolve_dom_query_runtime(target, env)? {
                Some(target_node) => self.dom.query_selector_all_from(&target_node, selector)?,
                None => Vec::new(),
            }
        } else {
            self.dom.query_selector_all(selector)?
        };
        let prev_item = env.get(item_var).cloned();
        let prev_item_const = self.is_const_binding(env, item_var);
        let prev_index = index_var.as_ref().and_then(|var| env.get(var).cloned());
        let prev_index_const = index_var
            .as_ref()
            .is_some_and(|name| self.is_const_binding(env, name));
        let prev_local_bindings = env.get(INTERNAL_LOCAL_BINDINGS_KEY).cloned();
        let mut local_binding_names = prev_local_bindings
            .as_ref()
            .and_then(|value| match value {
                Value::Array(bindings) => Some(
                    bindings
                        .borrow()
                        .iter()
                        .filter_map(|entry| match entry {
                            Value::String(name) => Some(name.clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        if !local_binding_names.iter().any(|name| name == item_var) {
            local_binding_names.push(item_var.to_string());
        }
        if let Some(index_var) = index_var {
            if !local_binding_names.iter().any(|name| name == index_var) {
                local_binding_names.push(index_var.clone());
            }
        }
        local_binding_names.sort();
        local_binding_names.dedup();
        env.insert(
            INTERNAL_LOCAL_BINDINGS_KEY.to_string(),
            Self::new_array_value(local_binding_names.into_iter().map(Value::String).collect()),
        );
        self.set_const_binding(env, item_var, false);
        if let Some(index_var) = index_var {
            self.set_const_binding(env, index_var, false);
        }

        let loop_result = (|| -> Result<ExecFlow> {
            for (idx, node) in items.iter().enumerate() {
                let item_value = Value::Node(*node);
                env.insert(item_var.to_string(), item_value.clone());
                self.sync_global_binding_if_needed(env, item_var, &item_value);
                if let Some(index_var) = index_var {
                    let index_value = Value::Number(idx as i64);
                    env.insert(index_var.clone(), index_value.clone());
                    self.sync_global_binding_if_needed(env, index_var, &index_value);
                }
                match self.execute_stmts_with_pending_scope(body, event_param, event, env, false)? {
                    ExecFlow::Continue => {}
                    ExecFlow::Break(None) => break,
                    ExecFlow::Break(label) => return Ok(ExecFlow::Break(label)),
                    ExecFlow::ContinueLoop(None) => continue,
                    ExecFlow::ContinueLoop(label) => return Ok(ExecFlow::ContinueLoop(label)),
                    ExecFlow::Return => return Ok(ExecFlow::Return),
                }
            }
            Ok(ExecFlow::Continue)
        })();

        if let Some(prev) = prev_item {
            env.insert(item_var.to_string(), prev.clone());
            self.sync_global_binding_if_needed(env, item_var, &prev);
        } else {
            env.remove(item_var);
        }
        self.set_const_binding(env, item_var, prev_item_const);
        if let Some(index_var) = index_var {
            if let Some(prev) = prev_index {
                env.insert(index_var.clone(), prev.clone());
                self.sync_global_binding_if_needed(env, index_var, &prev);
            } else {
                env.remove(index_var);
            }
            self.set_const_binding(env, index_var, prev_index_const);
        }
        match prev_local_bindings {
            Some(value) => {
                env.insert(INTERNAL_LOCAL_BINDINGS_KEY.to_string(), value);
            }
            None => {
                env.remove(INTERNAL_LOCAL_BINDINGS_KEY);
            }
        }
        loop_result
    }
}
