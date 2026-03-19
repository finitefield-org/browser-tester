use super::*;

impl Harness {
    pub(crate) fn eval_event_target_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let (is_event_target, is_match_media, shadowed) = {
            let entries = object.borrow();
            (
                Self::is_event_target_object(&entries),
                Self::is_match_media_object(&entries),
                Self::placeholder_backed_object_builtin_is_shadowed(&entries, member),
            )
        };
        if !is_event_target {
            return Ok(None);
        }
        if shadowed {
            return Ok(None);
        }

        let (normalized_member, listener_event_type, capture_mode) = match member {
            "addListener" if is_match_media => ("addEventListener", "change", None),
            "removeListener" if is_match_media => ("removeEventListener", "change", None),
            "addEventListener" | "removeEventListener" => (member, "", Some(())),
            _ => return Ok(None),
        };

        let (event_type, capture, callback_value) = if capture_mode.is_none() {
            if evaluated_args.len() != 1 {
                let label = if normalized_member == "addEventListener" {
                    "addListener"
                } else {
                    "removeListener"
                };
                return Err(Error::ScriptRuntime(format!(
                    "{label} requires exactly one callback argument"
                )));
            }
            (
                listener_event_type.to_string(),
                false,
                evaluated_args.first().cloned().unwrap_or(Value::Undefined),
            )
        } else {
            if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                return Err(Error::ScriptRuntime(format!(
                    "{normalized_member} requires two or three arguments"
                )));
            }
            (
                evaluated_args[0].as_string(),
                self.parse_listener_capture_arg(evaluated_args.get(2))?,
                evaluated_args[1].clone(),
            )
        };

        let node = self.event_target_listener_node_id(object);
        let result = match normalized_member {
            "addEventListener" => match callback_value {
                Value::Function(function) => {
                    self.listeners.add(
                        node,
                        event_type,
                        Listener {
                            capture,
                            is_event_handler_property: false,
                            is_arrow: function.is_arrow,
                            handler: function.handler.clone(),
                            function: Some(function.clone()),
                            captured_names: function.captured_names.clone(),
                            captured_env: function.captured_env.clone(),
                            captured_pending_function_decls: function
                                .captured_pending_function_decls
                                .clone(),
                        },
                    );
                    Value::Undefined
                }
                Value::Null | Value::Undefined => Value::Undefined,
                _ => {
                    return Err(Error::ScriptRuntime(
                        "addEventListener callback must be a function".into(),
                    ));
                }
            },
            "removeEventListener" => match callback_value {
                Value::Function(function) => {
                    let _ = self
                        .listeners
                        .remove(node, &event_type, capture, &function.handler);
                    Value::Undefined
                }
                Value::Null | Value::Undefined => Value::Undefined,
                _ => {
                    return Err(Error::ScriptRuntime(
                        "removeEventListener callback must be a function".into(),
                    ));
                }
            },
            _ => return Ok(None),
        };

        Ok(Some(result))
    }
}
