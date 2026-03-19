use super::*;

impl Harness {
    pub(crate) fn eval_clipboard_method_call(
        &mut self,
        method: &ClipboardMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let (method_name, evaluated_args) = match method {
            ClipboardMethod::ReadText => ("readText", Vec::new()),
            ClipboardMethod::WriteText => (
                "writeText",
                vec![self.eval_expr(&args[0], env, event_param, event)?],
            ),
        };

        if let Some((receiver, callee)) =
            self.resolve_clipboard_method_override(env, method_name)?
        {
            return self
                .execute_callable_value_with_this_and_env(
                    &callee,
                    &evaluated_args,
                    event,
                    Some(env),
                    Some(receiver),
                )
                .map_err(|err| match err {
                    Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                        Error::ScriptRuntime(format!("'{}' is not a function", method_name))
                    }
                    other => other,
                });
        }

        match method {
            ClipboardMethod::ReadText => {
                let promise = self.new_pending_promise();
                if let Some(reason) = self.platform_mocks.clipboard_read_error.clone() {
                    self.promise_reject(&promise, Value::String(reason));
                } else {
                    self.promise_resolve(
                        &promise,
                        Value::String(self.platform_mocks.clipboard_text.clone()),
                    )?;
                }
                Ok(Value::Promise(promise))
            }
            ClipboardMethod::WriteText => {
                let promise = self.new_pending_promise();
                if let Some(reason) = self.platform_mocks.clipboard_write_error.clone() {
                    self.promise_reject(&promise, Value::String(reason));
                } else {
                    self.platform_mocks.clipboard_text = evaluated_args[0].as_string();
                    self.promise_resolve(&promise, Value::Undefined)?;
                }
                Ok(Value::Promise(promise))
            }
        }
    }

    fn resolve_clipboard_method_override(
        &mut self,
        env: &HashMap<String, Value>,
        method_name: &str,
    ) -> Result<Option<(Value, Value)>> {
        let navigator = if let Some(value) = env.get("navigator") {
            Some(value.clone())
        } else {
            self.script_runtime.env.get("navigator").cloned()
        };
        let Some(navigator) = navigator else {
            return Ok(None);
        };

        let clipboard = self
            .object_property_from_value(&navigator, "clipboard")
            .map_err(|err| match err {
                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                    Error::ScriptRuntime(
                        "member call target does not support property 'clipboard'".into(),
                    )
                }
                other => other,
            })?;

        let use_builtin = if let Value::Object(entries) = &clipboard {
            let entries = entries.borrow();
            let is_builtin_clipboard = matches!(
                Self::object_get_entry(&entries, INTERNAL_CLIPBOARD_OBJECT_KEY),
                Some(Value::Bool(true))
            );
            if !is_builtin_clipboard {
                false
            } else {
                let default_key = match method_name {
                    "readText" => INTERNAL_CLIPBOARD_READ_TEXT_DEFAULT_KEY,
                    "writeText" => INTERNAL_CLIPBOARD_WRITE_TEXT_DEFAULT_KEY,
                    _ => return Ok(None),
                };
                let current =
                    Self::object_get_entry(&entries, method_name).unwrap_or(Value::Undefined);
                Self::object_get_entry(&entries, default_key)
                    .as_ref()
                    .is_some_and(|default_value| self.strict_equal(&current, default_value))
            }
        } else {
            false
        };

        if use_builtin {
            return Ok(None);
        }

        let callee = self
            .object_property_from_value(&clipboard, method_name)
            .map_err(|err| match err {
                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                    Error::ScriptRuntime(format!(
                        "member call target does not support property '{}'",
                        method_name
                    ))
                }
                other => other,
            })?;

        Ok(Some((clipboard, callee)))
    }
}
