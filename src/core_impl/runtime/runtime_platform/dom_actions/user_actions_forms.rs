use super::*;

impl Harness {
    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        if self.is_effectively_disabled(target) {
            return Ok(());
        }
        if self.dom.readonly(target) {
            return Ok(());
        }

        let tag = self
            .dom
            .tag_name(target)
            .ok_or_else(|| Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input or textarea".into(),
                actual: "non-element".into(),
            })?
            .to_ascii_lowercase();

        if tag != "input" && tag != "textarea" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input or textarea".into(),
                actual: tag,
            });
        }

        stacker::grow(32 * 1024 * 1024, || {
            self.dom.set_value(target, text)?;
            self.dispatch_event(target, "input")?;
            Ok(())
        })
    }

    pub fn set_checked(&mut self, selector: &str, checked: bool) -> Result<()> {
        let target = self.select_one(selector)?;
        if self.is_effectively_disabled(target) {
            return Ok(());
        }
        let tag = self
            .dom
            .tag_name(target)
            .unwrap_or_default()
            .to_ascii_lowercase();
        if tag != "input" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input[type=checkbox|radio]".into(),
                actual: tag,
            });
        }

        let kind = self
            .dom
            .attr(target, "type")
            .unwrap_or_else(|| "text".into())
            .to_ascii_lowercase();
        if kind != "checkbox" && kind != "radio" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input[type=checkbox|radio]".into(),
                actual: format!("input[type={kind}]"),
            });
        }

        stacker::grow(32 * 1024 * 1024, || {
            let current = self.dom.checked(target)?;
            if current != checked {
                self.dom.set_checked(target, checked)?;
                self.dispatch_event(target, "input")?;
                self.dispatch_event(target, "change")?;
            }

            Ok(())
        })
    }

    pub fn click(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        self.click_node(target)
    }

    pub(crate) fn click_node_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.is_effectively_disabled(target) {
            return Ok(());
        }

        self.dom.set_active_pseudo_element(Some(target));
        let result: Result<()> = (|| {
            let click_outcome = self.dispatch_event_with_env(target, "click", env, true)?;
            if click_outcome.default_prevented {
                return Ok(());
            }

            if let Some(control) = self.resolve_label_control(target) {
                if control != target {
                    self.click_node_with_env(control, env)?;
                    return Ok(());
                }
            }

            if let Some(details) = self.resolve_details_for_summary_click(target) {
                if self.dom.has_attr(details, "open")? {
                    self.dom.remove_attr(details, "open")?;
                } else {
                    self.dom.set_attr(details, "open", "true")?;
                }
                let _ = self.dispatch_event_with_options(
                    details, "toggle", env, true, false, false, None, None, None,
                )?;
            }

            if is_checkbox_input(&self.dom, target) {
                let current = self.dom.checked(target)?;
                self.dom.set_indeterminate(target, false)?;
                self.dom.set_checked(target, !current)?;
                self.dispatch_event_with_env(target, "input", env, true)?;
                self.dispatch_event_with_env(target, "change", env, true)?;
            }

            if is_radio_input(&self.dom, target) {
                let current = self.dom.checked(target)?;
                if !current {
                    self.dom.set_checked(target, true)?;
                    self.dispatch_event_with_env(target, "input", env, true)?;
                    self.dispatch_event_with_env(target, "change", env, true)?;
                }
            }

            if is_submit_control(&self.dom, target) {
                self.request_form_submit_with_env(target, env)?;
            }

            Ok(())
        })();
        self.dom.set_active_pseudo_element(None);
        result
    }

    pub(crate) fn click_node(&mut self, target: NodeId) -> Result<()> {
        self.with_script_env_always(|this, env| {
            stacker::grow(32 * 1024 * 1024, || this.click_node_with_env(target, env))
        })
    }

    pub fn focus(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || self.focus_node(target))
    }

    pub fn blur(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || self.blur_node(target))
    }

    pub fn submit(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || {
            self.with_script_env(|this, env| this.request_form_submit_with_env(target, env))
        })
    }

    pub(crate) fn submit_form_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        // form.submit() bypasses validation and submit event dispatch.
        if let Some(form_id) = self.resolve_submit_form_target(target) {
            self.maybe_close_dialog_for_form_submit_with_env(form_id, env)?;
        }

        Ok(())
    }

    pub(crate) fn request_form_submit_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let Some(form_id) = self.resolve_submit_form_target(target) else {
            return Ok(());
        };
        self.request_form_submit_node_with_env(form_id, env)
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
        self.request_form_submit_node_with_env(form_id, env)
    }

    pub(crate) fn request_form_submit_node_with_env(
        &mut self,
        form_id: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if !self.form_is_valid_for_submit(form_id)? {
            return Ok(());
        }

        let submit_outcome = self.dispatch_event_with_env(form_id, "submit", env, true)?;
        if !submit_outcome.default_prevented {
            self.maybe_close_dialog_for_form_submit_with_env(form_id, env)?;
        }
        Ok(())
    }

    pub(crate) fn maybe_close_dialog_for_form_submit_with_env(
        &mut self,
        form: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let Some(method) = self.dom.attr(form, "method") else {
            return Ok(());
        };
        if !method.eq_ignore_ascii_case("dialog") {
            return Ok(());
        }
        let Some(dialog) = self.dom.find_ancestor_by_tag(form, "dialog") else {
            return Ok(());
        };
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
                let default_checked = self.dom.attr(control, "checked").is_some();
                self.dom.set_checked(control, default_checked)?;
                self.dom.set_indeterminate(control, false)?;
                continue;
            }

            if self
                .dom
                .tag_name(control)
                .map(|tag| tag.eq_ignore_ascii_case("select"))
                .unwrap_or(false)
            {
                self.dom.sync_select_value(control)?;
                continue;
            }

            let default_value = self.dom.attr(control, "value").unwrap_or_default();
            self.dom.set_value(control, &default_value)?;
        }

        Ok(())
    }

    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        self.with_script_env(|this, env| {
            stacker::grow(32 * 1024 * 1024, || {
                let _ = this.dispatch_event_with_env(target, event, env, false)?;
                Ok(())
            })
        })
    }
}
