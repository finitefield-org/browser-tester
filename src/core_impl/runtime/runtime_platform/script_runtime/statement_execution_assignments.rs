use super::*;

impl Harness {
    fn enumerable_own_keys_for_object_destructure(value: &Value) -> Vec<String> {
        match value {
            Value::Object(entries) => {
                let entries = entries.borrow();
                entries
                    .iter()
                    .filter(|(key, _)| Self::is_enumerable_object_key(&*entries, key))
                    .map(|(key, _)| key.clone())
                    .collect()
            }
            Value::Array(values) => {
                let values = values.borrow();
                let mut keys = values
                    .iter()
                    .enumerate()
                    .filter_map(|(index, _)| {
                        (!Self::array_index_is_hole(&values, index)).then(|| index.to_string())
                    })
                    .collect::<Vec<_>>();
                keys.extend(
                    values
                        .properties
                        .iter()
                        .filter(|(key, _)| Self::is_enumerable_object_key(&values.properties, key))
                        .map(|(key, _)| key.clone()),
                );
                keys
            }
            Value::String(text) => text
                .chars()
                .enumerate()
                .map(|(index, _)| index.to_string())
                .collect(),
            _ => Vec::new(),
        }
    }

    fn ensure_binding_is_mutable(&self, env: &HashMap<String, Value>, name: &str) -> Result<()> {
        self.ensure_binding_initialized(env, name)?;
        if self.is_const_binding(env, name) {
            return Err(Error::ScriptRuntime(
                "Assignment to constant variable".into(),
            ));
        }
        Ok(())
    }

    fn try_append_string_binding_in_place(
        &mut self,
        env: &mut HashMap<String, Value>,
        name: &str,
        rhs: &Value,
    ) -> Result<bool> {
        let rhs = self.to_primitive_for_addition(rhs)?;
        if matches!(rhs, Value::Symbol(_)) {
            return Err(Error::ScriptRuntime(
                "Cannot convert a Symbol value to a string".into(),
            ));
        }
        let Some(Value::String(existing)) = env.get_mut(name) else {
            return Ok(false);
        };
        existing.push_str(&rhs.as_string());
        Ok(true)
    }

    fn finish_binding_assignment(
        &mut self,
        env: &mut HashMap<String, Value>,
        name: &str,
        next: &Value,
    ) {
        self.sync_arguments_after_param_write(env, name, next);
        self.sync_global_binding_if_needed(env, name, next);
        self.sync_scheduled_task_captures_for_binding_if_escaping(env, name, next);
    }

    fn finish_destructure_binding_assignment(
        &mut self,
        env: &mut HashMap<String, Value>,
        pending_tdz_bindings: &mut HashSet<String>,
        name: &str,
        next: &Value,
        decl_kind: Option<VarDeclKind>,
    ) {
        self.sync_arguments_after_param_write(env, name, next);
        if let Some(kind) = decl_kind {
            self.set_const_binding(env, name, matches!(kind, VarDeclKind::Const));
            if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                self.mark_tdz_initialized(pending_tdz_bindings, name);
            }
        }
        self.sync_global_binding_if_needed(env, name, next);
        self.sync_scheduled_task_captures_for_binding_if_escaping(env, name, next);
    }

    fn execute_var_assign_stmt(
        &mut self,
        name: &str,
        op: VarAssignOp,
        expr: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        self.ensure_binding_initialized(env, name)?;
        let previous = env
            .get(name)
            .cloned()
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;
        let should_assign = match op {
            VarAssignOp::LogicalAnd => previous.truthy(),
            VarAssignOp::LogicalOr => !previous.truthy(),
            VarAssignOp::Nullish => matches!(&previous, Value::Null | Value::Undefined),
            _ => true,
        };
        if !should_assign {
            return Ok(());
        }
        self.ensure_binding_is_mutable(env, name)?;

        let next = match op {
            VarAssignOp::Assign => self.eval_expr(expr, env, event_param, event)?,
            VarAssignOp::Add => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                if self.try_append_string_binding_in_place(env, name, &value)? {
                    let next = env
                        .get(name)
                        .cloned()
                        .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;
                    self.finish_binding_assignment(env, name, &next);
                    self.bind_timer_id_to_task_env(name, expr, &next);
                    return Ok(());
                }
                self.add_values(&previous, &value)?
            }
            VarAssignOp::Sub => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.eval_binary(&BinaryOp::Sub, &previous, &value)?
            }
            VarAssignOp::Mul => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.eval_binary(&BinaryOp::Mul, &previous, &value)?
            }
            VarAssignOp::Pow => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.eval_binary(&BinaryOp::Pow, &previous, &value)?
            }
            VarAssignOp::BitOr => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.eval_binary(&BinaryOp::BitOr, &previous, &value)?
            }
            VarAssignOp::BitXor => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.eval_binary(&BinaryOp::BitXor, &previous, &value)?
            }
            VarAssignOp::BitAnd => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.eval_binary(&BinaryOp::BitAnd, &previous, &value)?
            }
            VarAssignOp::ShiftLeft => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.eval_binary(&BinaryOp::ShiftLeft, &previous, &value)?
            }
            VarAssignOp::ShiftRight => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.eval_binary(&BinaryOp::ShiftRight, &previous, &value)?
            }
            VarAssignOp::UnsignedShiftRight => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.eval_binary(&BinaryOp::UnsignedShiftRight, &previous, &value)?
            }
            VarAssignOp::Div => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.eval_binary(&BinaryOp::Div, &previous, &value)?
            }
            VarAssignOp::Mod => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.eval_binary(&BinaryOp::Mod, &previous, &value)?
            }
            VarAssignOp::LogicalAnd => {
                if previous.truthy() {
                    self.eval_expr(expr, env, event_param, event)?
                } else {
                    previous.clone()
                }
            }
            VarAssignOp::LogicalOr => {
                if previous.truthy() {
                    previous.clone()
                } else {
                    self.eval_expr(expr, env, event_param, event)?
                }
            }
            VarAssignOp::Nullish => {
                if matches!(&previous, Value::Null | Value::Undefined) {
                    self.eval_expr(expr, env, event_param, event)?
                } else {
                    previous.clone()
                }
            }
        };
        env.insert(name.to_string(), next.clone());
        self.finish_binding_assignment(env, name, &next);
        self.bind_timer_id_to_task_env(name, expr, &next);
        Ok(())
    }

    fn execute_private_assign_stmt(
        &mut self,
        target: &Expr,
        member: &str,
        expr: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        let receiver = self.eval_expr(target, env, event_param, event)?;
        let value = self.eval_expr(expr, env, event_param, event)?;
        self.private_set_member(member, &receiver, value, env, event)
    }

    fn execute_var_update_stmt(
        &mut self,
        name: &str,
        delta: i8,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        self.ensure_binding_initialized(env, name)?;
        let previous = env
            .get(name)
            .cloned()
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;
        self.ensure_binding_is_mutable(env, name)?;
        let next = match previous {
            Value::BigInt(value) => Value::BigInt(value + JsBigInt::from(delta)),
            Value::Symbol(_) => {
                return Err(Error::ScriptRuntime(
                    "Cannot convert a Symbol value to a number".into(),
                ));
            }
            other => {
                let numeric = Self::coerce_number_for_global(&other);
                Self::number_value(numeric + f64::from(delta))
            }
        };
        env.insert(name.to_string(), next.clone());
        self.finish_binding_assignment(env, name, &next);
        Ok(())
    }

    fn execute_array_destructure_assign_stmt(
        &mut self,
        pattern: &ArrayDestructurePattern,
        expr: &Expr,
        decl_kind: Option<VarDeclKind>,
        pending_tdz_bindings: &mut HashSet<String>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        let value = self.eval_expr(expr, env, event_param, event)?;
        let values = self.array_like_values_from_value(&value)?;
        let is_declaration = decl_kind.is_some();
        for (index, binding) in pattern.items.iter().enumerate() {
            let Some(binding) = binding else {
                continue;
            };
            let target_name = &binding.target;
            if !is_declaration {
                self.ensure_binding_initialized(env, target_name)?;
            }
            if !is_declaration && env.contains_key(target_name) {
                self.ensure_binding_is_mutable(env, target_name)?;
            }
            let mut next = values.get(index).cloned().unwrap_or(Value::Undefined);
            if matches!(next, Value::Undefined) {
                if let Some(default) = &binding.default {
                    next = self.eval_expr(default, env, event_param, event)?;
                }
            }
            env.insert(target_name.clone(), next.clone());
            self.finish_destructure_binding_assignment(
                env,
                pending_tdz_bindings,
                target_name,
                &next,
                decl_kind,
            );
        }
        if let Some(rest_name) = &pattern.rest {
            if !is_declaration {
                self.ensure_binding_initialized(env, rest_name)?;
            }
            if !is_declaration && env.contains_key(rest_name) {
                self.ensure_binding_is_mutable(env, rest_name)?;
            }
            let next =
                Self::new_array_value(values.into_iter().skip(pattern.items.len()).collect());
            env.insert(rest_name.clone(), next.clone());
            self.finish_destructure_binding_assignment(
                env,
                pending_tdz_bindings,
                rest_name,
                &next,
                decl_kind,
            );
        }
        Ok(())
    }

    fn execute_object_destructure_assign_stmt(
        &mut self,
        pattern: &ObjectDestructurePattern,
        expr: &Expr,
        decl_kind: Option<VarDeclKind>,
        pending_tdz_bindings: &mut HashSet<String>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        let source = self.eval_expr(expr, env, event_param, event)?;
        if matches!(source, Value::Null | Value::Undefined) {
            return Err(Error::ScriptRuntime(
                "object destructuring source must be an object".into(),
            ));
        }
        let is_declaration = decl_kind.is_some();
        for binding in &pattern.bindings {
            let source_key = &binding.source;
            let target_name = &binding.target;
            if !is_declaration {
                self.ensure_binding_initialized(env, target_name)?;
            }
            if !is_declaration && env.contains_key(target_name) {
                self.ensure_binding_is_mutable(env, target_name)?;
            }
            let mut next = self.object_property_from_value(&source, source_key)?;
            if matches!(next, Value::Undefined) {
                if let Some(default) = &binding.default {
                    next = self.eval_expr(default, env, event_param, event)?;
                }
            }
            env.insert(target_name.clone(), next.clone());
            self.finish_destructure_binding_assignment(
                env,
                pending_tdz_bindings,
                target_name,
                &next,
                decl_kind,
            );
        }
        if let Some(rest_name) = &pattern.rest {
            if !is_declaration {
                self.ensure_binding_initialized(env, rest_name)?;
            }
            if !is_declaration && env.contains_key(rest_name) {
                self.ensure_binding_is_mutable(env, rest_name)?;
            }
            let excluded = pattern
                .bindings
                .iter()
                .map(|binding| binding.source.clone())
                .collect::<HashSet<_>>();
            let mut entries = Vec::new();
            for key in Self::enumerable_own_keys_for_object_destructure(&source) {
                if excluded.contains(&key) {
                    continue;
                }
                let value = self.object_property_from_value(&source, &key)?;
                entries.push((key, value));
            }
            let next = Self::new_object_value(entries);
            env.insert(rest_name.clone(), next.clone());
            self.finish_destructure_binding_assignment(
                env,
                pending_tdz_bindings,
                rest_name,
                &next,
                decl_kind,
            );
        }
        Ok(())
    }

    fn execute_form_data_append_stmt(
        &mut self,
        target_var: &str,
        name: &Expr,
        value: &Expr,
        filename: &Option<Expr>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        let name = self.eval_expr(name, env, event_param, event)?;
        let value = self.eval_expr(value, env, event_param, event)?;
        let filename = filename
            .as_ref()
            .map(|expr| self.eval_expr(expr, env, event_param, event))
            .transpose()?;
        let target_node = match env.get(target_var) {
            Some(Value::Node(node)) => Some(*node),
            Some(_) => None,
            None => {
                return Err(Error::ScriptRuntime(format!(
                    "unknown FormData variable: {}",
                    target_var
                )));
            }
        };

        if let Some(target_node) = target_node {
            let mut append_args = vec![name.clone(), value.clone()];
            if let Some(filename) = filename.clone() {
                append_args.push(filename);
            }
            self.eval_document_append_call(target_node, &append_args)?;
            return Ok(());
        }

        let name = name.as_string();
        let url_search_params_value = value.as_string();
        let form_data_value = Self::form_data_append_string_value(&value, filename.as_ref());
        let target = env.get_mut(target_var).ok_or_else(|| {
            Error::ScriptRuntime(format!("unknown FormData variable: {}", target_var))
        })?;
        match target {
            Value::FormData(entries) => {
                entries.borrow_mut().push((name, form_data_value));
            }
            Value::Object(entries) => {
                if !Self::is_url_search_params_object(&entries.borrow()) {
                    return Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a FormData instance",
                        target_var
                    )));
                }
                {
                    let mut object_ref = entries.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&object_ref);
                    pairs.push((name, url_search_params_value));
                    Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                }
                self.sync_url_search_params_owner(entries);
            }
            _ => {
                return Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a FormData instance",
                    target_var
                )));
            }
        }
        Ok(())
    }

    pub(crate) fn try_execute_assignment_stmt(
        &mut self,
        stmt: &Stmt,
        pending_tdz_bindings: &mut HashSet<String>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<Option<ExecFlow>> {
        match stmt {
            Stmt::VarAssign { name, op, expr } => {
                self.execute_var_assign_stmt(name, *op, expr, env, event_param, event)?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::PrivateAssign {
                target,
                member,
                expr,
            } => {
                self.execute_private_assign_stmt(target, member, expr, env, event_param, event)?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::VarUpdate { name, delta } => {
                self.execute_var_update_stmt(name, *delta, env)?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::ArrayDestructureAssign {
                pattern,
                expr,
                decl_kind,
            } => {
                self.execute_array_destructure_assign_stmt(
                    pattern,
                    expr,
                    *decl_kind,
                    pending_tdz_bindings,
                    env,
                    event_param,
                    event,
                )?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::ObjectDestructureAssign {
                pattern,
                expr,
                decl_kind,
            } => {
                self.execute_object_destructure_assign_stmt(
                    pattern,
                    expr,
                    *decl_kind,
                    pending_tdz_bindings,
                    env,
                    event_param,
                    event,
                )?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::ObjectAssign {
                target,
                path,
                op,
                expr,
            } => {
                self.execute_object_assignment_stmt(
                    target,
                    path,
                    *op,
                    expr,
                    env,
                    event_param,
                    event,
                )?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::FormDataAppend {
                target_var,
                name,
                value,
                filename,
            } => {
                self.execute_form_data_append_stmt(
                    target_var,
                    name,
                    value,
                    filename,
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
