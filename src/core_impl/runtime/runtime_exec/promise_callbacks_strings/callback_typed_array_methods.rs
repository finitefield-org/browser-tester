use super::*;

#[path = "callback_typed_array_foreign_receivers.rs"]
mod callback_typed_array_foreign_receivers;
#[path = "callback_typed_array_native_methods.rs"]
mod callback_typed_array_native_methods;

impl Harness {
    pub(crate) fn execute_callback_value(
        &mut self,
        callback: &Value,
        args: &[Value],
        event: &EventState,
    ) -> Result<Value> {
        self.execute_callable_value(callback, args, event)
    }

    pub(crate) fn execute_callback_value_with_env(
        &mut self,
        callback: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        if let Some(env) = caller_env {
            self.sync_listener_capture_env_if_shared(env);
            let result = self.execute_callable_value_with_env(callback, args, event, Some(env));
            self.sync_listener_capture_env_if_shared(env);
            result
        } else {
            self.execute_callback_value(callback, args, event)
        }
    }

    pub(crate) fn eval_typed_array_method(
        &mut self,
        target: &str,
        method: TypedArrayInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !matches!(env.get(target), Some(Value::TypedArray(_))) {
            let Some(target_value) = env.get(target) else {
                return Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                )));
            };

            return self.eval_typed_array_foreign_receiver_method(
                target,
                target_value,
                method,
                args,
                env,
                event_param,
                event,
            );
        }

        let array = self.resolve_typed_array_from_env(env, target)?;
        if array.borrow().buffer.borrow().detached {
            return Err(Error::ScriptRuntime(
                "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
            ));
        }
        let kind = array.borrow().kind;
        let len = array.borrow().observed_length();
        let this_value = Value::TypedArray(array.clone());

        self.eval_typed_array_native_method(
            array,
            kind,
            len,
            this_value,
            method,
            args,
            env,
            event_param,
            event,
        )
    }
}
