use super::*;

#[path = "map_collection_eval.rs"]
mod map_collection_eval;
#[path = "set_collection_eval.rs"]
mod set_collection_eval;

impl Harness {
    fn is_builtin_placeholder_callable(value: &Value) -> bool {
        matches!(value, Value::Function(function) if function.function_id == usize::MAX)
    }

    fn execute_named_member_with_receiver(
        &mut self,
        callee: &Value,
        target_value: &Value,
        member: &str,
        evaluated_args: &[Value],
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Value> {
        self.execute_callable_value_with_this_and_env(
            callee,
            evaluated_args,
            event,
            Some(env),
            Some(target_value.clone()),
        )
        .map_err(|err| match err {
            Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                Error::ScriptRuntime(format!("'{}' is not a function", member))
            }
            other => other,
        })
    }
}
