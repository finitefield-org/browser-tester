use super::*;

impl Harness {
    fn dispatch_text_selectionchange_if_needed_with_env(
        &mut self,
        target: NodeId,
        before_start: usize,
        before_end: usize,
        before_direction: &str,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if !self.node_supports_text_selection(target) {
            return Ok(());
        }
        let after_start = self.dom.selection_start(target)?;
        let after_end = self.dom.selection_end(target)?;
        let after_direction = self.dom.selection_direction(target)?;
        if before_start != after_start
            || before_end != after_end
            || before_direction != after_direction
        {
            let _ = self.dispatch_document_selectionchange_with_env(env)?;
        }
        Ok(())
    }

    pub(crate) fn dispatch_form_control_input_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<EventState> {
        self.dispatch_event_with_options(target, "input", env, true, true, false, None, None, None)
    }

    pub(crate) fn dispatch_form_control_change_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<EventState> {
        self.dispatch_event_with_options(target, "change", env, true, true, false, None, None, None)
    }

    /// Replace the value of an input-like control and dispatch browser-like events.
    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        if self.is_effectively_disabled(target) {
            return Ok(());
        }
        let input_type = if self
            .dom
            .tag_name(target)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
        {
            self.dom
                .attr(target, "type")
                .unwrap_or_else(|| "text".to_string())
                .to_ascii_lowercase()
        } else {
            String::new()
        };
        if input_type == "hidden" || input_type == "image" {
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

        if tag == "select" {
            return self.with_script_env_always(|this, env| {
                stacker::grow(32 * 1024 * 1024, || {
                    let previous_value = this.dom.value(target)?;
                    this.dom.set_select_value(target, text)?;
                    let next_value = this.dom.value(target)?;
                    if next_value != previous_value {
                        this.dispatch_form_control_input_with_env(target, env)?;
                        this.dispatch_form_control_change_with_env(target, env)?;
                    }
                    Ok(())
                })
            });
        }

        if tag != "input" && tag != "textarea" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input, textarea, or select".into(),
                actual: tag,
            });
        }

        self.with_script_env_always(|this, env| {
            stacker::grow(32 * 1024 * 1024, || {
                this.focus_node_with_env(target, env)?;
                let before_selection = if this.node_supports_text_selection(target) {
                    Some((
                        this.dom.selection_start(target)?,
                        this.dom.selection_end(target)?,
                        this.dom.selection_direction(target)?,
                    ))
                } else {
                    None
                };
                this.dom.set_value(target, text)?;
                if let Some((before_start, before_end, before_direction)) = before_selection {
                    this.dispatch_text_selectionchange_if_needed_with_env(
                        target,
                        before_start,
                        before_end,
                        &before_direction,
                        env,
                    )?;
                }
                this.dispatch_form_control_input_with_env(target, env)?;
                this.note_user_committed_change_candidate(target)?;
                Ok(())
            })
        })
    }

    /// Select an option in a `<select>` by its value.
    pub fn set_select_value(&mut self, selector: &str, value: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        if self.is_effectively_disabled(target) {
            return Ok(());
        }
        let tag = self
            .dom
            .tag_name(target)
            .ok_or_else(|| Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "select".into(),
                actual: "non-element".into(),
            })?
            .to_ascii_lowercase();
        if tag != "select" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "select".into(),
                actual: tag,
            });
        }

        self.with_script_env_always(|this, env| {
            stacker::grow(32 * 1024 * 1024, || {
                let previous_value = this.dom.value(target)?;
                this.dom.set_select_value(target, value)?;
                let next_value = this.dom.value(target)?;
                if next_value != previous_value {
                    this.dispatch_form_control_input_with_env(target, env)?;
                    this.dispatch_form_control_change_with_env(target, env)?;
                }
                Ok(())
            })
        })
    }

    /// Seed deterministic file input state for `input[type="file"]`.
    pub fn set_input_files(&mut self, selector: &str, files: &[MockFile]) -> Result<()> {
        let target = self.select_one(selector)?;
        let files = files.to_vec();
        let selector = selector.to_string();
        stacker::grow(32 * 1024 * 1024, || {
            self.with_script_env_always(|this, env| {
                this.set_input_files_with_env(target, &selector, &files, env)
            })
        })
    }

    pub(crate) fn set_input_files_with_env(
        &mut self,
        target: NodeId,
        selector: &str,
        files: &[MockFile],
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.is_effectively_disabled(target) {
            return Ok(());
        }

        let tag = self
            .dom
            .tag_name(target)
            .ok_or_else(|| Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input[type=file]".into(),
                actual: "non-element".into(),
            })?
            .to_ascii_lowercase();
        if tag != "input" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input[type=file]".into(),
                actual: tag,
            });
        }

        let kind = self
            .dom
            .attr(target, "type")
            .unwrap_or_else(|| "text".into())
            .to_ascii_lowercase();
        if kind != "file" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input[type=file]".into(),
                actual: format!("input[type={kind}]"),
            });
        }

        let changed = self.dom.set_file_input_files(target, files)?;
        if changed {
            self.dispatch_form_control_input_with_env(target, env)?;
            self.dispatch_form_control_change_with_env(target, env)?;
        } else {
            self.dispatch_event_with_env(target, "cancel", env, true)?;
        }
        Ok(())
    }

    /// Set the checked state of a checkbox or radio-like control.
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

        self.with_script_env_always(|this, env| {
            stacker::grow(32 * 1024 * 1024, || {
                let current = this.dom.checked(target)?;
                if current != checked {
                    this.dom.set_checked(target, checked)?;
                    this.dispatch_form_control_input_with_env(target, env)?;
                    this.dispatch_form_control_change_with_env(target, env)?;
                }

                Ok(())
            })
        })
    }
}
