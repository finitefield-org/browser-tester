use super::*;

impl Harness {
    pub(crate) fn set_node_event_handler_property(
        &mut self,
        node: NodeId,
        key: &str,
        value: Value,
    ) -> Result<bool> {
        let Some(raw_event_type) = key.strip_prefix("on") else {
            return Ok(false);
        };
        if raw_event_type.is_empty() {
            return Ok(false);
        }

        let event_type = raw_event_type.to_ascii_lowercase();
        let is_body_window_alias = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
            && Self::is_body_window_event_handler_alias(event_type.as_str());
        if is_body_window_alias {
            if let Some(previous_handler) = self
                .dom_runtime
                .node_event_handler_props
                .remove(&(node, event_type.clone()))
            {
                let _ = self.listeners.remove_event_handler_property(
                    node,
                    &event_type,
                    &previous_handler,
                );
            }
            let _ = self.set_event_target_event_handler_property(
                &self.dom_runtime.window_object.clone(),
                key,
                value.clone(),
            )?;
            self.dom_runtime.node_expando_props.insert(
                (node, key.to_string()),
                if matches!(value, Value::Function(_)) {
                    value
                } else {
                    Value::Null
                },
            );
            return Ok(true);
        }
        let previous_handler = self
            .dom_runtime
            .node_event_handler_props
            .remove(&(node, event_type.clone()));

        if let Value::Function(function) = value {
            let handler = function.handler.clone();
            let listener = Listener {
                capture: false,
                is_event_handler_property: true,
                is_arrow: function.is_arrow,
                handler: handler.clone(),
                function: Some(function.clone()),
                captured_names: function.captured_names.clone(),
                captured_env: function.captured_env.clone(),
                captured_pending_function_decls: function.captured_pending_function_decls.clone(),
            };

            let replaced = previous_handler.as_ref().is_some_and(|previous| {
                self.listeners.replace_event_handler_property(
                    node,
                    &event_type,
                    previous,
                    listener.clone(),
                )
            });
            if !replaced {
                if let Some(previous_handler) = previous_handler.as_ref() {
                    let _ = self.listeners.remove_event_handler_property(
                        node,
                        &event_type,
                        previous_handler,
                    );
                }
                self.listeners.add(node, event_type.clone(), listener);
            }

            self.dom_runtime
                .node_event_handler_props
                .insert((node, event_type), handler);
            self.dom_runtime
                .node_expando_props
                .insert((node, key.to_string()), Value::Function(function));
        } else {
            if let Some(previous_handler) = previous_handler {
                let _ = self.listeners.remove_event_handler_property(
                    node,
                    &event_type,
                    &previous_handler,
                );
            }
            self.dom_runtime
                .node_expando_props
                .insert((node, key.to_string()), Value::Null);
        }
        Ok(true)
    }

    pub(crate) fn is_body_window_event_handler_alias(event_type: &str) -> bool {
        matches!(
            event_type,
            "afterprint"
                | "beforeprint"
                | "beforeunload"
                | "blur"
                | "error"
                | "focus"
                | "gamepadconnected"
                | "gamepaddisconnected"
                | "hashchange"
                | "languagechange"
                | "load"
                | "message"
                | "messageerror"
                | "offline"
                | "online"
                | "pagehide"
                | "pageshow"
                | "popstate"
                | "rejectionhandled"
                | "resize"
                | "scroll"
                | "storage"
                | "unhandledrejection"
                | "unload"
        )
    }

    pub(crate) fn set_event_target_event_handler_property(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        key: &str,
        value: Value,
    ) -> Result<bool> {
        let Some(raw_event_type) = key.strip_prefix("on") else {
            return Ok(false);
        };
        if raw_event_type.is_empty() {
            return Ok(false);
        }
        let is_event_target = {
            let entries = object.borrow();
            Self::is_event_target_object(&entries)
        };
        if !is_event_target {
            return Ok(false);
        }

        let event_type = raw_event_type.to_ascii_lowercase();
        let node = self.event_target_listener_node_id(object);
        let previous_handler = {
            let entries = object.borrow();
            match Self::object_get_entry(&entries, key) {
                Some(Value::Function(function)) => Some(function.handler.clone()),
                _ => None,
            }
        };

        if let Value::Function(function) = value {
            let listener = Listener {
                capture: false,
                is_event_handler_property: true,
                is_arrow: function.is_arrow,
                handler: function.handler.clone(),
                function: Some(function.clone()),
                captured_names: function.captured_names.clone(),
                captured_env: function.captured_env.clone(),
                captured_pending_function_decls: function.captured_pending_function_decls.clone(),
            };
            let replaced = previous_handler.as_ref().is_some_and(|previous| {
                self.listeners.replace_event_handler_property(
                    node,
                    &event_type,
                    previous,
                    listener.clone(),
                )
            });
            if !replaced {
                if let Some(previous_handler) = previous_handler.as_ref() {
                    let _ = self.listeners.remove_event_handler_property(
                        node,
                        &event_type,
                        previous_handler,
                    );
                }
                self.listeners.add(node, event_type, listener);
            }
            Self::object_set_entry(
                &mut object.borrow_mut(),
                key.to_string(),
                Value::Function(function),
            );
        } else {
            if let Some(previous_handler) = previous_handler {
                let _ = self.listeners.remove_event_handler_property(
                    node,
                    &event_type,
                    &previous_handler,
                );
            }
            Self::object_set_entry(&mut object.borrow_mut(), key.to_string(), Value::Null);
        }
        Ok(true)
    }
}
