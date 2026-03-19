use super::*;

impl Harness {
    pub(crate) fn try_eval_call_like_exprs(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::FunctionCall { target, args } => {
                    if target == "super" {
                        let super_constructor = Self::super_constructor_from_env(env)?;
                        let this_value = Self::super_this_from_env(env)?;
                        let evaluated_args =
                            self.eval_call_args_with_spread(args, env, event_param, event)?;
                        let super_result = self.execute_constructor_value_with_this_and_env(
                            &super_constructor,
                            &evaluated_args,
                            event,
                            Some(env),
                            Some(this_value),
                        )?;
                        self.initialize_current_constructor_instance_fields(
                            env,
                            event_param,
                            event,
                        )?;
                        return Ok(super_result);
                    }
                    self.ensure_binding_initialized(env, target)?;
                    let callee = if let Some(callee) = env.get(target).cloned() {
                        callee
                    } else if let Some(pending) =
                        self.resolve_listener_capture_pending_value(target)
                    {
                        let Some(callee) = pending else {
                            return Err(Error::ScriptRuntime(format!(
                                "unknown variable: {target}"
                            )));
                        };
                        callee
                    } else if let Some(callee) = self.resolve_pending_function_decl(target, env) {
                        callee
                    } else {
                        return Err(Error::ScriptRuntime(format!("unknown variable: {target}")));
                    };
                    let evaluated_args =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    self.execute_callable_value_with_env_and_sync(
                        &callee,
                        &evaluated_args,
                        event,
                        env,
                    )
                    .map_err(|err| match err {
                        Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                            Error::ScriptRuntime(format!("'{target}' is not a function"))
                        }
                        other => other,
                    })
                }
                Expr::ImportCall { module, options } => {
                    Ok(self.eval_import_call(module, options, env, event_param, event))
                }
                Expr::Call {
                    target,
                    args,
                    optional,
                } => {
                    if let Expr::IndexGet {
                        target: index_target,
                        index,
                        optional: index_optional,
                    } = target.as_ref()
                    {
                        let Some((callee, this_arg)) = self.eval_index_get_call_target_and_this(
                            index_target,
                            index,
                            *index_optional,
                            *optional,
                            env,
                            event_param,
                            event,
                        )?
                        else {
                            return Ok(Value::Undefined);
                        };
                        let evaluated_args =
                            self.eval_call_args_with_spread(args, env, event_param, event)?;
                        return self
                            .execute_callable_value_with_this_and_env_and_sync(
                                &callee,
                                &evaluated_args,
                                event,
                                env,
                                Some(this_arg),
                            )
                            .map_err(|err| match err {
                                Error::ScriptRuntime(msg)
                                    if msg == "callback is not a function" =>
                                {
                                    Error::ScriptRuntime("call target is not a function".into())
                                }
                                other => other,
                            });
                    }
                    if let Expr::ArrayIndex { target, index } = target.as_ref() {
                        let Some((callee, this_arg)) = self.eval_array_index_call_target_and_this(
                            target,
                            index,
                            *optional,
                            env,
                            event_param,
                            event,
                        )?
                        else {
                            return Ok(Value::Undefined);
                        };
                        let evaluated_args =
                            self.eval_call_args_with_spread(args, env, event_param, event)?;
                        return self
                            .execute_callable_value_with_this_and_env_and_sync(
                                &callee,
                                &evaluated_args,
                                event,
                                env,
                                Some(this_arg),
                            )
                            .map_err(|err| match err {
                                Error::ScriptRuntime(msg)
                                    if msg == "callback is not a function" =>
                                {
                                    Error::ScriptRuntime("call target is not a function".into())
                                }
                                other => other,
                            });
                    }
                    let callee = self.eval_expr(target, env, event_param, event)?;
                    if *optional && matches!(callee, Value::Null | Value::Undefined) {
                        return Ok(Value::Undefined);
                    }
                    let evaluated_args =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    self.execute_callable_value_with_env_and_sync(
                        &callee,
                        &evaluated_args,
                        event,
                        env,
                    )
                    .map_err(|err| match err {
                        Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                            Error::ScriptRuntime("call target is not a function".into())
                        }
                        other => other,
                    })
                }
                Expr::PrivateMemberCall {
                    target,
                    member,
                    args,
                } => {
                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    let evaluated_args =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;
                    self.private_call_member(member, &receiver, &evaluated_args, env, event)
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
