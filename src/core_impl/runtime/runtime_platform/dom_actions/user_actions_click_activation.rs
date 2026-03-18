use super::*;

#[derive(Debug, Clone)]
enum LegacyInputActivationState {
    Checkbox {
        checked: bool,
        checked_dirty: bool,
        indeterminate: bool,
    },
    RadioGroup {
        states: Vec<(NodeId, bool, bool)>,
    },
}

impl Harness {
    fn snapshot_radio_group_activation_state(&self, target: NodeId) -> Vec<(NodeId, bool, bool)> {
        let target_name = self.dom.attr(target, "name").unwrap_or_default();
        if target_name.is_empty() {
            return vec![(
                target,
                self.dom.checked(target).unwrap_or(false),
                self.dom
                    .element(target)
                    .map(|element| element.checked_dirty)
                    .unwrap_or(false),
            )];
        }
        let target_form = self.dom.control_form_owner(target);
        self.dom
            .all_element_nodes()
            .into_iter()
            .filter(|node| is_radio_input(&self.dom, *node))
            .filter(|node| self.dom.attr(*node, "name").unwrap_or_default() == target_name)
            .filter(|node| self.dom.control_form_owner(*node) == target_form)
            .map(|node| {
                (
                    node,
                    self.dom.checked(node).unwrap_or(false),
                    self.dom
                        .element(node)
                        .map(|element| element.checked_dirty)
                        .unwrap_or(false),
                )
            })
            .collect()
    }

    fn legacy_pre_activate_input_control(
        &mut self,
        target: NodeId,
    ) -> Result<Option<LegacyInputActivationState>> {
        if is_checkbox_input(&self.dom, target) {
            let snapshot =
                self.dom
                    .element(target)
                    .map(|element| LegacyInputActivationState::Checkbox {
                        checked: element.checked,
                        checked_dirty: element.checked_dirty,
                        indeterminate: element.indeterminate,
                    });
            if let Some(LegacyInputActivationState::Checkbox {
                checked,
                checked_dirty: _,
                indeterminate: _,
            }) = snapshot.as_ref()
            {
                self.dom.set_indeterminate(target, false)?;
                self.dom.set_checked_state(target, !*checked, Some(true))?;
            }
            return Ok(snapshot);
        }

        if is_radio_input(&self.dom, target) {
            let states = self.snapshot_radio_group_activation_state(target);
            if !self.dom.checked(target)? {
                self.dom.set_checked_state(target, true, Some(true))?;
            }
            return Ok(Some(LegacyInputActivationState::RadioGroup { states }));
        }

        Ok(None)
    }

    fn checkbox_state_changed_from_snapshot(
        &self,
        target: NodeId,
        checked: bool,
        indeterminate: bool,
    ) -> bool {
        self.dom.checked(target).unwrap_or(checked) != checked
            || self.dom.indeterminate(target).unwrap_or(indeterminate) != indeterminate
    }

    fn radio_group_state_changed_from_snapshot(&self, states: &[(NodeId, bool, bool)]) -> bool {
        states
            .iter()
            .any(|(node, checked, _)| self.dom.checked(*node).unwrap_or(*checked) != *checked)
    }

    /// Dispatch a trusted click-like path and apply default actions when allowed.
    pub fn click(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        self.click_node(target)
    }

    pub(crate) fn set_details_open_state_with_env(
        &mut self,
        details: NodeId,
        open: bool,
        env: &mut HashMap<String, Value>,
    ) -> Result<bool> {
        if !self
            .dom
            .tag_name(details)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
        {
            return Ok(false);
        }

        let was_open = self.dom.has_attr(details, "open")?;
        if was_open == open {
            return Ok(false);
        }

        let mut peers_to_close_toggle = Vec::new();
        if open {
            let group_name = self.dom.attr(details, "name").unwrap_or_default();
            if !group_name.is_empty() {
                for candidate in self.dom.query_selector_all("details")? {
                    if candidate == details {
                        continue;
                    }
                    if self.dom.attr(candidate, "name").as_deref() != Some(group_name.as_str()) {
                        continue;
                    }
                    if self.dom.has_attr(candidate, "open")? {
                        peers_to_close_toggle.push(candidate);
                    }
                }
            }
        }

        if open {
            self.dom.set_attr(details, "open", "true")?;
        } else {
            self.dom.remove_attr(details, "open")?;
        }

        let (old_state, new_state) = if open {
            ("closed", "open")
        } else {
            ("open", "closed")
        };
        let _ = self.dispatch_event_with_options(
            details,
            "toggle",
            env,
            true,
            false,
            false,
            None,
            Some(old_state),
            Some(new_state),
        )?;

        for peer in peers_to_close_toggle {
            if self.dom.has_attr(peer, "open")? {
                continue;
            }
            let _ = self.dispatch_event_with_options(
                peer,
                "toggle",
                env,
                true,
                false,
                false,
                None,
                Some("open"),
                Some("closed"),
            )?;
        }

        Ok(true)
    }

    fn activate_click_node_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
        trusted: bool,
    ) -> Result<()> {
        if self.is_effectively_disabled(target) {
            return Ok(());
        }

        self.dom.set_active_pseudo_element(Some(target));
        let result: Result<()> = (|| {
            let legacy_activation = self.legacy_pre_activate_input_control(target)?;
            let click_outcome = self.dispatch_event_with_options(
                target, "click", env, trusted, true, true, None, None, None,
            )?;
            if click_outcome.default_prevented {
                if let Some(state) = legacy_activation {
                    match state {
                        LegacyInputActivationState::Checkbox {
                            checked,
                            checked_dirty,
                            indeterminate,
                        } => {
                            if let Some(element) = self.dom.element_mut(target) {
                                element.checked = checked;
                                element.checked_dirty = checked_dirty;
                                element.indeterminate = indeterminate;
                            }
                        }
                        LegacyInputActivationState::RadioGroup { states } => {
                            for (node, checked, checked_dirty) in states {
                                if let Some(element) = self.dom.element_mut(node) {
                                    element.checked = checked;
                                    element.checked_dirty = checked_dirty;
                                }
                            }
                        }
                    }
                }
                return Ok(());
            }

            if let Some(control) = self.resolve_label_activation_control(target) {
                if control != target {
                    self.focus_node_with_env(control, env)?;
                    self.activate_click_node_with_env(control, env, trusted)?;
                    return Ok(());
                }
            }

            if let Some(details) = self.resolve_details_for_summary_click(target) {
                let next_open = !self.dom.has_attr(details, "open")?;
                let _ = self.set_details_open_state_with_env(details, next_open, env)?;
            }

            if let Some(state) = legacy_activation {
                match state {
                    LegacyInputActivationState::Checkbox {
                        checked,
                        checked_dirty: _,
                        indeterminate,
                    } => {
                        if self.checkbox_state_changed_from_snapshot(target, checked, indeterminate)
                        {
                            self.dispatch_form_control_input_with_env(target, env)?;
                            self.dispatch_form_control_change_with_env(target, env)?;
                        }
                    }
                    LegacyInputActivationState::RadioGroup { states } => {
                        if self.radio_group_state_changed_from_snapshot(&states) {
                            self.dispatch_form_control_input_with_env(target, env)?;
                            self.dispatch_form_control_change_with_env(target, env)?;
                        }
                    }
                }
            }

            self.apply_option_click_selection_with_env(target, env)?;

            if self.run_button_command_with_env(target, env)? {
                return Ok(());
            }

            if is_submit_control(&self.dom, target) {
                self.request_form_submit_with_env(target, Some(target), env)?;
            }
            if is_reset_control(&self.dom, target) {
                self.reset_form_with_env(target, env)?;
            }

            let captured_download = self.maybe_capture_anchor_download(target)?;
            if !captured_download {
                self.maybe_follow_anchor_hyperlink(target)?;
            }

            Ok(())
        })();
        self.dom.set_active_pseudo_element(None);
        result
    }

    pub(crate) fn click_node_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        self.activate_click_node_with_env(target, env, true)
    }

    fn apply_option_click_selection_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if !self
            .dom
            .tag_name(target)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("option"))
        {
            return Ok(());
        }

        let Some(select_node) = self.dom.find_ancestor_by_tag(target, "select") else {
            return Ok(());
        };
        if self.is_effectively_disabled(select_node) {
            return Ok(());
        }

        let previous_value = self.dom.value(select_node)?;
        let is_multiple = self.dom.attr(select_node, "multiple").is_some();
        let mut options = Vec::new();
        self.dom.collect_select_options(select_node, &mut options);

        for option in options {
            let is_target = option == target;
            let has_selected = self
                .dom
                .element(option)
                .map(|element| element.selected)
                .unwrap_or(false);
            if is_target {
                if !has_selected {
                    self.dom
                        .set_option_selected_state(option, true, Some(true))?;
                }
                continue;
            }
            if !is_multiple && has_selected {
                self.dom
                    .set_option_selected_state(option, false, Some(true))?;
            }
        }

        self.dom.sync_select_value(select_node)?;
        let next_value = self.dom.value(select_node)?;
        if next_value != previous_value {
            self.dispatch_form_control_input_with_env(select_node, env)?;
            self.dispatch_form_control_change_with_env(select_node, env)?;
        }
        Ok(())
    }

    pub(crate) fn run_button_command_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<bool> {
        if !self
            .dom
            .tag_name(target)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("button"))
        {
            return Ok(false);
        }

        let Some(command) = self.dom.attr(target, "command") else {
            return Ok(false);
        };
        let Some(command_for) = self.dom.attr(target, "commandfor") else {
            return Ok(false);
        };

        let Some(controlled) = self.dom.by_id(&command_for) else {
            return Ok(true);
        };
        let command = command.to_ascii_lowercase();
        let return_value = self.dom.attr(target, "value").map(Value::String);

        match command.as_str() {
            "show-modal" => {
                if self
                    .dom
                    .tag_name(controlled)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("dialog"))
                {
                    self.show_dialog_with_env(controlled, true, env)?;
                }
                Ok(true)
            }
            "close" => {
                if self
                    .dom
                    .tag_name(controlled)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("dialog"))
                {
                    self.close_dialog_with_env(controlled, return_value, env)?;
                }
                Ok(true)
            }
            "request-close" => {
                if self
                    .dom
                    .tag_name(controlled)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("dialog"))
                {
                    self.request_close_dialog_with_env(controlled, return_value, env)?;
                }
                Ok(true)
            }
            _ if command.starts_with("--") => Ok(true),
            _ => Ok(false),
        }
    }

    pub(crate) fn click_node(&mut self, target: NodeId) -> Result<()> {
        self.with_script_env_always(|this, env| {
            stacker::grow(32 * 1024 * 1024, || this.click_node_with_env(target, env))
        })
    }

    pub(crate) fn click_dom_method_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        // HTMLElement.click() must ignore re-entrant activation on the same node.
        if self.dom_runtime.click_in_progress.contains(&target) {
            return Ok(());
        }
        self.dom_runtime.click_in_progress.insert(target);
        let result = stacker::grow(32 * 1024 * 1024, || {
            self.activate_click_node_with_env(target, env, false)
        });
        self.dom_runtime.click_in_progress.remove(&target);
        result
    }

    pub(crate) fn click_dom_method(&mut self, target: NodeId) -> Result<()> {
        self.with_script_env_always(|this, env| this.click_dom_method_with_env(target, env))
    }
}
