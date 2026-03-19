use super::*;

impl Harness {
    pub(crate) fn try_eval_access_like_exprs(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::MemberGet {
                    target,
                    member,
                    optional,
                } => {
                    if Self::is_super_target_expr(target) {
                        let super_prototype = Self::super_prototype_from_env(env)?;
                        let this_value = Self::super_this_from_env(env)?;
                        return self.object_property_from_value_with_receiver(
                            &super_prototype,
                            member,
                            &this_value,
                        );
                    }
                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    if *optional && matches!(receiver, Value::Null | Value::Undefined) {
                        return Ok(Value::Undefined);
                    }
                    self.object_property_from_value(&receiver, member)
                }
                Expr::PrivateMemberGet { target, member } => {
                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    self.private_get_member(member, &receiver, env, event)
                }
                Expr::PrivateIn { member, target } => {
                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    Ok(Value::Bool(self.private_has_member(member, &receiver)?))
                }
                Expr::IndexGet {
                    target,
                    index,
                    optional,
                } => {
                    let is_super = Self::is_super_target_expr(target);
                    let receiver = if is_super {
                        Self::super_prototype_from_env(env)?
                    } else {
                        self.eval_expr(target, env, event_param, event)?
                    };
                    if *optional && matches!(receiver, Value::Null | Value::Undefined) {
                        return Ok(Value::Undefined);
                    }
                    let index_value = self.eval_expr(index, env, event_param, event)?;
                    let key = match index_value {
                        Value::Number(value) => value.to_string(),
                        Value::BigInt(value) => value.to_string(),
                        Value::Float(value) if value.is_finite() && value.fract() == 0.0 => {
                            format!("{:.0}", value)
                        }
                        other => self.property_key_to_storage_key(&other),
                    };
                    if is_super {
                        let this_value = Self::super_this_from_env(env)?;
                        self.object_property_from_value_with_receiver(&receiver, &key, &this_value)
                    } else {
                        self.object_property_from_value(&receiver, &key)
                    }
                }
                Expr::Var(name) => {
                    if name == "super" {
                        return Self::super_prototype_from_env(env);
                    }
                    self.ensure_binding_initialized(env, name)?;
                    let current_scope_pending = self.resolve_listener_capture_pending_value_from(
                        Self::pending_listener_capture_scope_start(env),
                        name,
                    );
                    if Self::env_has_local_or_lexical_binding(env, name) {
                        if let Some(pending) = current_scope_pending.clone() {
                            if let Some(value) = pending {
                                return Ok(value);
                            }
                            return Err(Error::ScriptRuntime(format!("unknown variable: {name}")));
                        }
                        if let Some(value) = env.get(name).cloned() {
                            return Ok(value);
                        }
                    }
                    if let Some(value) = self
                        .script_runtime
                        .expression_env_overrides
                        .get(name)
                        .cloned()
                        .flatten()
                    {
                        Ok(value)
                    } else if let Some(Some(value)) =
                        self.resolve_listener_capture_pending_value(name)
                    {
                        Ok(value)
                    } else if let Some(value) = env.get(name).cloned() {
                        Ok(value)
                    } else if let Some(pending) = current_scope_pending {
                        if let Some(value) = pending {
                            Ok(value)
                        } else {
                            Err(Error::ScriptRuntime(format!("unknown variable: {name}")))
                        }
                    } else if let Some(value) = self.resolve_pending_function_decl(name, env) {
                        Ok(value)
                    } else if let Some(value) = self.resolve_runtime_global_identifier(name) {
                        Ok(value)
                    } else if self.resolve_listener_capture_pending_value(name).is_some() {
                        Err(Error::ScriptRuntime(format!("unknown variable: {name}")))
                    } else {
                        Err(Error::ScriptRuntime(format!("unknown variable: {name}")))
                    }
                }
                Expr::ImportMeta => self.eval_import_meta_object(),
                Expr::NewTarget => self.eval_new_target_value(env),
                Expr::DomRef(target) => {
                    let is_list_query = matches!(
                        target,
                        DomQuery::BySelectorAll { .. } | DomQuery::QuerySelectorAll { .. }
                    );
                    if is_list_query {
                        let nodes = self
                            .resolve_dom_query_list_runtime(target, env)?
                            .unwrap_or_default();
                        Ok(Self::new_static_node_list_value(nodes))
                    } else if matches!(target, DomQuery::ById(_)) {
                        Ok(self
                            .resolve_dom_query_runtime(target, env)?
                            .map(Value::Node)
                            .unwrap_or(Value::Null))
                    } else {
                        Ok(self
                            .resolve_dom_query_runtime(target, env)?
                            .map(Value::Node)
                            .unwrap_or(Value::Null))
                    }
                }
                Expr::CreateElement(tag_name) => {
                    let node = self.dom.create_detached_element(tag_name.clone());
                    Ok(Value::Node(node))
                }
                Expr::CreateTextNode(text) => {
                    let node = self.dom.create_detached_text(text.clone());
                    Ok(Value::Node(node))
                }
                Expr::Function {
                    handler,
                    name,
                    is_async,
                    is_generator,
                    is_arrow,
                    is_method,
                } => {
                    let value = self.make_function_value(
                        handler.clone(),
                        env,
                        false,
                        *is_async,
                        *is_generator,
                        *is_arrow,
                        *is_method,
                    );
                    let Value::Function(function) = value else {
                        return Ok(value);
                    };
                    if let Some(expression_name) = name {
                        let mut named = function.as_ref().clone();
                        named.expression_name = Some(expression_name.clone());
                        named.local_bindings.insert(expression_name.clone());
                        named.captured_names.remove(expression_name);
                        named.captured_global_names.remove(expression_name);
                        let named = Rc::new(named);
                        self.sync_function_prototype_object(&named);
                        Ok(Value::Function(named))
                    } else {
                        Ok(Value::Function(function))
                    }
                }
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
