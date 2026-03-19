use super::*;

impl Harness {
    pub(crate) fn eval_node_member_call(
        &mut self,
        node: NodeId,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "addEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "addEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
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
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "addEventListener callback must be a function".into(),
                    )),
                }
            }
            "removeEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "removeEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
                    Value::Function(function) => {
                        let _ =
                            self.listeners
                                .remove(node, &event_type, capture, &function.handler);
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "removeEventListener callback must be a function".into(),
                    )),
                }
            }
            "click" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("click does not take arguments".into()));
                }
                self.click_dom_method(node)?;
                Ok(Some(Value::Undefined))
            }
            _ => {
                if let Some(value) =
                    self.try_eval_node_element_attribute_member_call(node, member, evaluated_args)?
                {
                    return Ok(Some(value));
                }
                if let Some(value) =
                    self.try_eval_node_tree_member_call(node, member, evaluated_args)?
                {
                    return Ok(Some(value));
                }
                self.try_eval_node_ui_media_member_call(node, member, evaluated_args, event)
            }
        }
    }
}
