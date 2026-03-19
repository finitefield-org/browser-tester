use super::*;

impl Harness {
    pub(crate) fn invoke_promise_capability(
        &mut self,
        capability: &PromiseCapabilityFunction,
        args: &[Value],
    ) -> Result<Value> {
        let mut already_called = capability.already_called.borrow_mut();
        if *already_called {
            return Ok(Value::Undefined);
        }
        *already_called = true;
        drop(already_called);

        let value = args.first().cloned().unwrap_or(Value::Undefined);
        if capability.reject {
            self.promise_reject(&capability.promise, value);
            Ok(Value::Undefined)
        } else {
            self.promise_resolve(&capability.promise, value)?;
            Ok(Value::Undefined)
        }
    }

    pub(crate) fn is_primitive_value(value: &Value) -> bool {
        matches!(
            value,
            Value::String(_)
                | Value::Bool(_)
                | Value::Number(_)
                | Value::Float(_)
                | Value::BigInt(_)
                | Value::Null
                | Value::Undefined
                | Value::Symbol(_)
        )
    }

    pub(crate) fn apply_constructor_instance_initializers_by_id(
        &mut self,
        constructor_id: usize,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        let Some(initializers) = self
            .script_runtime
            .constructor_instance_initializers
            .get(&constructor_id)
            .cloned()
        else {
            return Ok(());
        };

        let this_value = env.get("this").cloned().unwrap_or(Value::Undefined);
        for initializer in &initializers {
            self.apply_constructor_instance_initializer_to_receiver(
                initializer,
                &this_value,
                env,
                event_param,
                event,
            )?;
        }
        Ok(())
    }

    pub(crate) fn initialize_current_constructor_instance_fields(
        &mut self,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        let Some(constructor_id) = self.script_runtime.constructor_call_stack.last().copied()
        else {
            return Ok(());
        };
        let Some(already_initialized) = self
            .script_runtime
            .constructor_instance_initialized_stack
            .last()
            .copied()
        else {
            return Ok(());
        };
        if already_initialized {
            return Err(Error::ScriptRuntime(
                "super() has already been called for this constructor".into(),
            ));
        }
        self.apply_constructor_instance_initializers_by_id(
            constructor_id,
            env,
            event_param,
            event,
        )?;
        if let Some(last) = self
            .script_runtime
            .constructor_instance_initialized_stack
            .last_mut()
        {
            *last = true;
        }
        Ok(())
    }

    pub(crate) fn bind_handler_params(
        &mut self,
        handler: &ScriptHandler,
        args: &[Value],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        for (index, param) in handler.params.iter().enumerate() {
            if param.is_rest {
                let rest = if index < args.len() {
                    args[index..].to_vec()
                } else {
                    Vec::new()
                };
                env.insert(param.name.clone(), Self::new_array_value(rest));
                self.set_const_binding(env, &param.name, false);
                continue;
            }

            let provided = args.get(index).cloned().unwrap_or(Value::Undefined);
            let value = if matches!(provided, Value::Undefined) {
                if let Some(default_expr) = &param.default {
                    self.eval_expr(default_expr, env, event_param, event)?
                } else {
                    Value::Undefined
                }
            } else {
                provided
            };
            env.insert(param.name.clone(), value);
            self.set_const_binding(env, &param.name, false);
        }
        Ok(())
    }
}
