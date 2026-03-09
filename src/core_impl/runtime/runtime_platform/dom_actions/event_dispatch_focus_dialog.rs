use super::*;

impl Harness {
    fn node_or_ancestor_has_hidden_attribute(&self, node: NodeId) -> bool {
        let mut cursor = Some(node);
        while let Some(current) = cursor {
            if self.dom.attr(current, "hidden").is_some() {
                return true;
            }
            cursor = self.dom.parent(current);
        }
        false
    }

    pub(crate) fn control_dispatches_change_on_blur(&self, node: NodeId) -> bool {
        let Some(tag) = self.dom.tag_name(node) else {
            return false;
        };
        if tag.eq_ignore_ascii_case("textarea") {
            return true;
        }
        if !tag.eq_ignore_ascii_case("input") {
            return false;
        }
        !matches!(
            self.normalized_input_type(node).as_str(),
            "hidden" | "checkbox" | "radio" | "file" | "submit" | "reset" | "button" | "image"
        )
    }

    pub(crate) fn sync_change_tracking_to_current_value(&mut self, node: NodeId) -> Result<()> {
        if !self.control_dispatches_change_on_blur(node) {
            self.dom_runtime.focused_form_control_values.remove(&node);
            self.dom_runtime.pending_form_control_change.remove(&node);
            return Ok(());
        }
        let current = self.dom.value(node)?;
        if let Some(snapshot) = self.dom_runtime.focused_form_control_values.get_mut(&node) {
            *snapshot = current;
        }
        self.dom_runtime.pending_form_control_change.remove(&node);
        Ok(())
    }

    pub(crate) fn note_user_committed_change_candidate(&mut self, node: NodeId) -> Result<()> {
        if !self.control_dispatches_change_on_blur(node) {
            return Ok(());
        }
        let current = self.dom.value(node)?;
        let baseline = self
            .dom_runtime
            .focused_form_control_values
            .get(&node)
            .cloned()
            .unwrap_or_else(|| current.clone());
        if current != baseline {
            self.dom_runtime.pending_form_control_change.insert(node);
        } else {
            self.dom_runtime.pending_form_control_change.remove(&node);
        }
        Ok(())
    }

    fn prime_focus_change_tracking(&mut self, node: NodeId) -> Result<()> {
        if !self.control_dispatches_change_on_blur(node) {
            self.dom_runtime.focused_form_control_values.remove(&node);
            self.dom_runtime.pending_form_control_change.remove(&node);
            return Ok(());
        }
        let current = self.dom.value(node)?;
        self.dom_runtime
            .focused_form_control_values
            .insert(node, current);
        self.dom_runtime.pending_form_control_change.remove(&node);
        Ok(())
    }

    fn maybe_dispatch_change_on_blur_with_env(
        &mut self,
        node: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let baseline = self.dom_runtime.focused_form_control_values.remove(&node);
        let was_pending = self.dom_runtime.pending_form_control_change.remove(&node);
        if !was_pending {
            return Ok(());
        }
        let Some(baseline) = baseline else {
            return Ok(());
        };
        if self.dom.value(node)? != baseline {
            let _ = self.dispatch_event_with_options(
                node, "change", env, true, true, false, None, None, None,
            )?;
        }
        Ok(())
    }

    fn untrusted_event_default_flags(event_type: &str) -> (bool, bool) {
        if event_type.eq_ignore_ascii_case("paste")
            || event_type.eq_ignore_ascii_case("copy")
            || event_type.eq_ignore_ascii_case("cut")
        {
            return (true, true);
        }
        (false, false)
    }

    fn prepare_clipboard_event_payload(
        &self,
        event_type: &str,
    ) -> Option<(String, Rc<RefCell<ObjectValue>>)> {
        if event_type.eq_ignore_ascii_case("paste") {
            let text = self.platform_mocks.clipboard_text.clone();
            let value = Self::new_clipboard_data_object_value(&text);
            if let Value::Object(object) = value {
                return Some((text, object));
            }
            return None;
        }
        if event_type.eq_ignore_ascii_case("copy") || event_type.eq_ignore_ascii_case("cut") {
            let value = Self::new_clipboard_data_object_value("");
            if let Value::Object(object) = value {
                return Some((String::new(), object));
            }
        }
        None
    }

    pub(crate) fn dispatch_event_with_env(
        &mut self,
        target: NodeId,
        event_type: &str,
        env: &mut HashMap<String, Value>,
        trusted: bool,
    ) -> Result<EventState> {
        let mut event = if trusted {
            EventState::new(event_type, target, self.scheduler.now_ms)
        } else {
            EventState::new_untrusted(event_type, target, self.scheduler.now_ms)
        };
        if !trusted {
            let (bubbles, cancelable) = Self::untrusted_event_default_flags(event_type);
            event.bubbles = bubbles;
            event.cancelable = cancelable;
        }
        if let Some((clipboard_text, clipboard_data_object)) =
            self.prepare_clipboard_event_payload(event_type)
        {
            event.clipboard_data = Some(clipboard_text);
            event.clipboard_data_object = Some(clipboard_data_object);
        }
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
        if let Some((clipboard_text, clipboard_data_object)) =
            self.prepare_clipboard_event_payload(event_type)
        {
            event.clipboard_data = Some(clipboard_text);
            event.clipboard_data_object = Some(clipboard_data_object);
        }
        event.bubbles = bubbles;
        event.cancelable = cancelable;
        event.state = state;
        event.old_state = old_state.map(str::to_string);
        event.new_state = new_state.map(str::to_string);
        self.dispatch_prepared_event_with_env(event, env)
    }

    pub(crate) fn dispatch_invalid_event(&mut self, target: NodeId) -> Result<EventState> {
        self.with_script_env(|this, env| this.dispatch_invalid_event_with_env(target, env, true))
    }

    pub(crate) fn dispatch_invalid_event_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
        trusted: bool,
    ) -> Result<EventState> {
        self.dispatch_event_with_options(
            target, "invalid", env, trusted, false, true, None, None, None,
        )
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
        if !self.dom.is_connected(node) {
            return Ok(());
        }

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

        if self.node_or_ancestor_has_hidden_attribute(node) {
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
        self.prime_focus_change_tracking(node)?;
        self.dispatch_event_with_options(node, "focus", env, true, false, false, None, None, None)?;
        self.dispatch_event_with_options(
            node, "focusin", env, true, true, false, None, None, None,
        )?;
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

        self.maybe_dispatch_change_on_blur_with_env(node, env)?;
        self.dom.set_active_element(None);
        self.dispatch_event_with_options(node, "blur", env, true, false, false, None, None, None)?;
        self.dispatch_event_with_options(
            node, "focusout", env, true, true, false, None, None, None,
        )?;
        Ok(())
    }

    pub(crate) fn scroll_into_view_node_with_env(
        &mut self,
        _node: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        self.dispatch_document_scroll_sequence_with_env(env, true)?;
        Ok(())
    }

    pub(crate) fn dispatch_document_scroll_with_env(
        &mut self,
        env: &mut HashMap<String, Value>,
    ) -> Result<EventState> {
        self.dispatch_event_with_options(
            self.dom.root,
            "scroll",
            env,
            true,
            false,
            false,
            None,
            None,
            None,
        )
    }

    pub(crate) fn dispatch_document_selectionchange_with_env(
        &mut self,
        env: &mut HashMap<String, Value>,
    ) -> Result<EventState> {
        self.dispatch_event_with_options(
            self.dom.root,
            "selectionchange",
            env,
            true,
            false,
            false,
            None,
            None,
            None,
        )
    }

    pub(crate) fn dispatch_document_dom_content_loaded_with_env(
        &mut self,
        env: &mut HashMap<String, Value>,
    ) -> Result<EventState> {
        self.dispatch_event_with_options(
            self.dom.root,
            "DOMContentLoaded",
            env,
            true,
            false,
            false,
            None,
            None,
            None,
        )
    }

    pub(crate) fn finalize_document_ready_state_with_dom_content_loaded(&mut self) -> Result<()> {
        self.dom_runtime.document_ready_state = "interactive".to_string();
        self.with_script_env_always(|this, env| {
            let _ = this.dispatch_document_dom_content_loaded_with_env(env)?;
            Ok(())
        })?;
        self.dom_runtime.document_ready_state = "complete".to_string();
        Ok(())
    }

    pub(crate) fn dispatch_document_scrollend_with_env(
        &mut self,
        env: &mut HashMap<String, Value>,
    ) -> Result<EventState> {
        self.dispatch_event_with_options(
            self.dom.root,
            "scrollend",
            env,
            true,
            false,
            false,
            None,
            None,
            None,
        )
    }

    pub(crate) fn dispatch_document_scroll_sequence_with_env(
        &mut self,
        env: &mut HashMap<String, Value>,
        position_changed: bool,
    ) -> Result<()> {
        let _ = self.dispatch_document_scroll_with_env(env)?;
        if position_changed {
            let _ = self.dispatch_document_scrollend_with_env(env)?;
        }
        Ok(())
    }

    pub(crate) fn dispatch_document_scroll_sequence(
        &mut self,
        position_changed: bool,
    ) -> Result<()> {
        self.with_script_env(|this, env| {
            this.dispatch_document_scroll_sequence_with_env(env, position_changed)
        })
    }

    pub(crate) fn dispatch_document_selectionchange(&mut self) -> Result<EventState> {
        self.with_script_env(|this, env| this.dispatch_document_selectionchange_with_env(env))
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
