use super::*;

impl Harness {
    pub(crate) fn dispatch_event(
        &mut self,
        target: NodeId,
        event_type: &str,
    ) -> Result<EventState> {
        self.with_script_env(|this, env| {
            this.dispatch_event_with_env(target, event_type, env, true)
        })
    }

    pub(crate) fn dispatch_event_with_env(
        &mut self,
        target: NodeId,
        event_type: &str,
        env: &mut HashMap<String, Value>,
        trusted: bool,
    ) -> Result<EventState> {
        let event = if trusted {
            EventState::new(event_type, target, self.scheduler.now_ms)
        } else {
            EventState::new_untrusted(event_type, target, self.scheduler.now_ms)
        };
        self.dispatch_prepared_event_with_env(event, env)
    }

    pub(crate) fn dispatch_event_with_options(
        &mut self,
        target: NodeId,
        event_type: &str,
        env: &mut HashMap<String, Value>,
        trusted: bool,
        bubbles: bool,
        cancelable: bool,
        state: Option<Value>,
        old_state: Option<&str>,
        new_state: Option<&str>,
    ) -> Result<EventState> {
        let mut event = if trusted {
            EventState::new(event_type, target, self.scheduler.now_ms)
        } else {
            EventState::new_untrusted(event_type, target, self.scheduler.now_ms)
        };
        event.bubbles = bubbles;
        event.cancelable = cancelable;
        event.state = state;
        event.old_state = old_state.map(str::to_string);
        event.new_state = new_state.map(str::to_string);
        self.dispatch_prepared_event_with_env(event, env)
    }

    pub(crate) fn dispatch_prepared_event_with_env(
        &mut self,
        mut event: EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<EventState> {
        let target = event.target;
        self.run_in_task_context(|this| {
            let mut path = Vec::new();
            let mut cursor = Some(target);
            while let Some(node) = cursor {
                path.push(node);
                cursor = this.dom.parent(node);
            }
            path.reverse();

            if path.is_empty() {
                this.trace_event_done(&event, "empty_path");
                return Ok(());
            }

            // Capture phase.
            if path.len() >= 2 {
                for node in &path[..path.len() - 1] {
                    event.event_phase = 1;
                    event.current_target = *node;
                    this.invoke_listeners(*node, &mut event, env, true)?;
                    if event.propagation_stopped {
                        this.trace_event_done(&event, "propagation_stopped");
                        return Ok(());
                    }
                }
            }

            // Target phase: capture listeners first.
            event.event_phase = 2;
            event.current_target = target;
            this.invoke_listeners(target, &mut event, env, true)?;
            if event.propagation_stopped {
                this.trace_event_done(&event, "propagation_stopped");
                return Ok(());
            }

            // Target phase: bubble listeners.
            event.event_phase = 2;
            this.invoke_listeners(target, &mut event, env, false)?;
            if event.propagation_stopped {
                this.trace_event_done(&event, "propagation_stopped");
                return Ok(());
            }

            // Bubble phase.
            if event.bubbles && path.len() >= 2 {
                for node in path[..path.len() - 1].iter().rev() {
                    event.event_phase = 3;
                    event.current_target = *node;
                    this.invoke_listeners(*node, &mut event, env, false)?;
                    if event.propagation_stopped {
                        this.trace_event_done(&event, "propagation_stopped");
                        return Ok(());
                    }
                }
            }

            this.trace_event_done(&event, "completed");
            Ok(())
        })?;
        Ok(event)
    }

    pub(crate) fn focus_node(&mut self, node: NodeId) -> Result<()> {
        self.with_script_env(|this, env| this.focus_node_with_env(node, env))
    }

    pub(crate) fn focus_node_with_env(
        &mut self,
        node: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let is_hidden_input = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
            && self
                .dom
                .attr(node, "type")
                .unwrap_or_else(|| "text".to_string())
                .eq_ignore_ascii_case("hidden");
        if is_hidden_input {
            return Ok(());
        }

        if self.is_effectively_disabled(node) {
            return Ok(());
        }

        if self.dom.active_element() == Some(node) {
            return Ok(());
        }

        if let Some(current) = self.dom.active_element() {
            self.blur_node_with_env(current, env)?;
        }

        self.dom.set_active_element(Some(node));
        self.dispatch_event_with_env(node, "focusin", env, true)?;
        self.dispatch_event_with_env(node, "focus", env, true)?;
        Ok(())
    }

    pub(crate) fn blur_node(&mut self, node: NodeId) -> Result<()> {
        self.with_script_env(|this, env| this.blur_node_with_env(node, env))
    }

    pub(crate) fn blur_node_with_env(
        &mut self,
        node: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.dom.active_element() != Some(node) {
            return Ok(());
        }

        self.dispatch_event_with_env(node, "focusout", env, true)?;
        self.dispatch_event_with_env(node, "blur", env, true)?;
        self.dom.set_active_element(None);
        Ok(())
    }

    pub(crate) fn scroll_into_view_node_with_env(
        &mut self,
        _node: NodeId,
        _env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        Ok(())
    }

    pub(crate) fn ensure_dialog_target(&self, node: NodeId, operation: &str) -> Result<()> {
        let tag = self
            .dom
            .tag_name(node)
            .ok_or_else(|| Error::ScriptRuntime(format!("{operation} target is not an element")))?;
        if tag.eq_ignore_ascii_case("dialog") {
            return Ok(());
        }
        Err(Error::ScriptRuntime(format!(
            "{operation} target is not a <dialog> element"
        )))
    }

    pub(crate) fn dialog_return_value(&self, dialog: NodeId) -> Result<String> {
        self.ensure_dialog_target(dialog, "returnValue")?;
        Ok(self
            .dom_runtime
            .dialog_return_values
            .get(&dialog)
            .cloned()
            .unwrap_or_default())
    }

    pub(crate) fn set_dialog_return_value(&mut self, dialog: NodeId, value: String) -> Result<()> {
        self.ensure_dialog_target(dialog, "returnValue")?;
        self.dom_runtime.dialog_return_values.insert(dialog, value);
        Ok(())
    }

    pub(crate) fn show_dialog_with_env(
        &mut self,
        dialog: NodeId,
        _modal: bool,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        self.ensure_dialog_target(dialog, "show/showModal")?;
        let _ = self.transition_dialog_open_state_with_env(dialog, true, false, env)?;
        Ok(())
    }

    pub(crate) fn close_dialog_with_env(
        &mut self,
        dialog: NodeId,
        return_value: Option<Value>,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        self.ensure_dialog_target(dialog, "close()")?;
        if let Some(return_value) = return_value {
            self.set_dialog_return_value(dialog, return_value.as_string())?;
        }
        let _ = self.transition_dialog_open_state_with_env(dialog, false, true, env)?;
        Ok(())
    }

    pub(crate) fn request_close_dialog_with_env(
        &mut self,
        dialog: NodeId,
        return_value: Option<Value>,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        self.ensure_dialog_target(dialog, "requestClose()")?;
        if let Some(return_value) = return_value {
            self.set_dialog_return_value(dialog, return_value.as_string())?;
        }
        if !self.dom.has_attr(dialog, "open")? {
            return Ok(());
        }
        let cancel_event = self.dispatch_event_with_options(
            dialog, "cancel", env, true, false, true, None, None, None,
        )?;
        if cancel_event.default_prevented {
            return Ok(());
        }
        let _ = self.transition_dialog_open_state_with_env(dialog, false, true, env)?;
        Ok(())
    }

    pub(crate) fn transition_dialog_open_state_with_env(
        &mut self,
        dialog: NodeId,
        open: bool,
        fire_close_event: bool,
        env: &mut HashMap<String, Value>,
    ) -> Result<bool> {
        let was_open = self.dom.has_attr(dialog, "open")?;
        if was_open == open {
            return Ok(false);
        }

        let (old_state, new_state) = if open {
            ("closed", "open")
        } else {
            ("open", "closed")
        };
        let beforetoggle = self.dispatch_event_with_options(
            dialog,
            "beforetoggle",
            env,
            true,
            false,
            true,
            None,
            Some(old_state),
            Some(new_state),
        )?;
        if beforetoggle.default_prevented {
            return Ok(false);
        }

        if open {
            self.dom.set_attr(dialog, "open", "true")?;
        } else {
            self.dom.remove_attr(dialog, "open")?;
        }

        let _ = self.dispatch_event_with_options(
            dialog,
            "toggle",
            env,
            true,
            false,
            false,
            None,
            Some(old_state),
            Some(new_state),
        )?;

        if !open && fire_close_event {
            let _ = self.dispatch_event_with_options(
                dialog, "close", env, true, false, false, None, None, None,
            )?;
        }

        Ok(true)
    }
}
