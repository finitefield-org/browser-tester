use super::*;

impl Harness {
    pub(crate) fn resolve_target_value_with_pending(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Option<Value> {
        self.resolve_listener_capture_pending_value(target)
            .flatten()
            .or_else(|| env.get(target).cloned())
            .or_else(|| self.resolve_runtime_global_identifier(target))
    }

    pub(crate) fn resolve_runtime_global_identifier(&self, name: &str) -> Option<Value> {
        self.script_runtime.env.get(name).cloned().or_else(|| {
            if Self::is_internal_env_key(name) {
                return None;
            }
            let window = self.dom_runtime.window_object.borrow();
            Self::object_get_entry(&window, name)
        })
    }

    pub(crate) fn eval_expr_json_object_array(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = match expr {
            Expr::JsonParse(value) => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                Self::parse_json_text(&value)
            }
            Expr::JsonStringify {
                value,
                replacer,
                space,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?;
                let _evaluated_replacer = replacer
                    .as_ref()
                    .map(|replacer| self.eval_expr(replacer, env, event_param, event))
                    .transpose()?;
                let evaluated_space = space
                    .as_ref()
                    .map(|space| self.eval_expr(space, env, event_param, event))
                    .transpose()?;
                match Self::json_stringify_top_level(&value, evaluated_space.as_ref())? {
                    Some(serialized) => Ok(Value::String(serialized)),
                    None => Ok(Value::Undefined),
                }
            }
            _ => match self.try_eval_object_expr(expr, env, event_param, event) {
                Ok(result) => Ok(result),
                Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => {
                    self.try_eval_array_expr(expr, env, event_param, event)
                }
                Err(err) => Err(err),
            },
        };
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }

    pub(crate) fn array_constructor_length_from_value(value: &Value) -> Result<Option<usize>> {
        match value {
            Value::Number(value) => {
                if *value < 0 {
                    return Err(Error::ScriptRuntime("invalid array length".into()));
                }
                Ok(Some(usize::try_from(*value).map_err(|_| {
                    Error::ScriptRuntime("invalid array length".into())
                })?))
            }
            Value::Float(value) => {
                if !value.is_finite() || *value < 0.0 || value.fract() != 0.0 {
                    return Err(Error::ScriptRuntime("invalid array length".into()));
                }
                if *value > usize::MAX as f64 {
                    return Err(Error::ScriptRuntime("invalid array length".into()));
                }
                Ok(Some(*value as usize))
            }
            _ => Ok(None),
        }
    }
}
