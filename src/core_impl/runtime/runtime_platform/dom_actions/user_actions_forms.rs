use super::*;

/// Form submission APIs for driving a [`Harness`] with selectors.
impl Harness {
    /// Submit a form through a user-like submission path.
    pub fn submit(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || {
            self.with_script_env(|this, env| this.request_form_submit_with_env(target, None, env))
        })
    }

    pub(crate) fn submit_form_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        // form.submit() bypasses validation and submit event dispatch.
        if let Some(form_id) = self.resolve_submit_form_target(target) {
            self.maybe_close_dialog_for_form_submit_with_env(form_id, None, env)?;
        }

        Ok(())
    }

    pub(crate) fn request_form_submit_with_env(
        &mut self,
        target: NodeId,
        submitter: Option<NodeId>,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let Some(form_id) = self.resolve_submit_form_target(target) else {
            return Ok(());
        };
        self.request_form_submit_node_with_env(form_id, submitter, env)
    }

    pub(crate) fn request_submit_form_with_env(
        &mut self,
        target: NodeId,
        submitter: Option<Value>,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let Some(form_id) = self.resolve_submit_form_target(target) else {
            return Ok(());
        };
        let submitter = self.resolve_request_submitter_node(submitter)?;
        if let Some(submitter_node) = submitter {
            if !is_submit_control(&self.dom, submitter_node) {
                return Err(Error::ScriptRuntime(
                    "requestSubmit submitter must be a submit control".into(),
                ));
            }
            if self.resolve_form_for_submit(submitter_node) != Some(form_id) {
                return Err(Error::ScriptRuntime(
                    "requestSubmit submitter must belong to the target form".into(),
                ));
            }
        }
        self.request_form_submit_node_with_env(form_id, submitter, env)
    }

    pub(crate) fn request_form_submit_node_with_env(
        &mut self,
        form_id: NodeId,
        submitter: Option<NodeId>,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let skip_validation = self.form_submission_skips_validation(form_id, submitter);

        if !skip_validation && !self.validate_form_submission_with_env(form_id, env)? {
            return Ok(());
        }

        let mut submit_event = EventState::new("submit", form_id, self.scheduler.now_ms);
        submit_event.submitter = submitter.map(Value::Node);
        let submit_outcome = self.dispatch_prepared_event_with_env(submit_event, env)?;
        if !submit_outcome.default_prevented {
            self.maybe_close_dialog_for_form_submit_with_env(form_id, submitter, env)?;
        }
        Ok(())
    }

    pub(crate) fn maybe_close_dialog_for_form_submit_with_env(
        &mut self,
        form: NodeId,
        submitter: Option<NodeId>,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let method = self.effective_form_submit_method(form, submitter);
        if !method.eq_ignore_ascii_case("dialog") {
            return Ok(());
        }
        let Some(dialog) = self.dom.find_ancestor_by_tag(form, "dialog") else {
            return Ok(());
        };
        if self.dialog_return_value(dialog)?.is_empty() {
            self.set_dialog_return_value(
                dialog,
                self.dialog_submitter_return_value(form, submitter),
            )?;
        }
        let _ = self.transition_dialog_open_state_with_env(dialog, false, true, env)?;
        Ok(())
    }

    pub(crate) fn reset_form_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let Some(form_id) = self.resolve_form_for_submit(target) else {
            return Ok(());
        };

        let outcome = self.dispatch_event_with_env(form_id, "reset", env, true)?;
        if outcome.default_prevented {
            return Ok(());
        }

        let controls = self.form_elements(form_id)?;
        for control in controls {
            if is_checkbox_input(&self.dom, control) || is_radio_input(&self.dom, control) {
                let default_checked = self
                    .dom
                    .element(control)
                    .map(|element| element.default_checked)
                    .unwrap_or(false);
                self.dom
                    .set_checked_state(control, default_checked, Some(false))?;
                self.dom.set_indeterminate(control, false)?;
                self.sync_change_tracking_to_current_value(control)?;
                continue;
            }

            if self
                .dom
                .tag_name(control)
                .map(|tag| tag.eq_ignore_ascii_case("select"))
                .unwrap_or(false)
            {
                let mut options = Vec::new();
                self.dom.collect_select_options(control, &mut options);
                for option in options {
                    let default_selected = self
                        .dom
                        .element(option)
                        .map(|element| element.default_selected)
                        .unwrap_or(false);
                    self.dom
                        .set_option_selected_state(option, default_selected, Some(false))?;
                }
                self.dom.sync_select_value(control)?;
                self.sync_change_tracking_to_current_value(control)?;
                continue;
            }

            let is_file_input = self
                .dom
                .element(control)
                .map(is_file_input_element)
                .unwrap_or(false);
            if is_file_input {
                self.dom.set_current_value_state(
                    control,
                    normalize_file_input_value(""),
                    Some(false),
                )?;
                if let Some(element) = self.dom.element_mut(control) {
                    element.files.clear();
                }
                self.sync_change_tracking_to_current_value(control)?;
                continue;
            }

            if self
                .dom
                .tag_name(control)
                .map(|tag| tag.eq_ignore_ascii_case("output"))
                .unwrap_or(false)
            {
                let default_value = self
                    .dom
                    .element(control)
                    .map(|element| element.default_value.clone())
                    .unwrap_or_default();
                self.dom.set_text_content(control, &default_value)?;
                if let Some(element) = self.dom.element_mut(control) {
                    element.value = default_value;
                    element.dirty_value = false;
                }
                self.sync_change_tracking_to_current_value(control)?;
                continue;
            }

            let default_value = self
                .dom
                .element(control)
                .map(|element| element.default_value.clone())
                .unwrap_or_default();
            self.dom
                .set_current_value_state(control, default_value, Some(false))?;
            self.sync_change_tracking_to_current_value(control)?;
        }

        Ok(())
    }
}
