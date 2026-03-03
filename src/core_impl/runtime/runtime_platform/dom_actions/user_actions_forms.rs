use super::*;

impl Harness {
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
            return stacker::grow(32 * 1024 * 1024, || {
                let previous_value = self.dom.value(target)?;
                self.dom.set_select_value(target, text)?;
                let next_value = self.dom.value(target)?;
                if next_value != previous_value {
                    self.dispatch_event(target, "input")?;
                    self.dispatch_event(target, "change")?;
                }
                Ok(())
            });
        }

        if tag != "input" && tag != "textarea" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input, textarea, or select".into(),
                actual: tag,
            });
        }

        stacker::grow(32 * 1024 * 1024, || {
            self.dom.set_value(target, text)?;
            self.dispatch_event(target, "input")?;
            Ok(())
        })
    }

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

        stacker::grow(32 * 1024 * 1024, || {
            let previous_value = self.dom.value(target)?;
            self.dom.set_select_value(target, value)?;
            let next_value = self.dom.value(target)?;
            if next_value != previous_value {
                self.dispatch_event(target, "input")?;
                self.dispatch_event(target, "change")?;
            }
            Ok(())
        })
    }

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
            self.dispatch_event_with_env(target, "input", env, true)?;
            self.dispatch_event_with_env(target, "change", env, true)?;
        } else {
            self.dispatch_event_with_env(target, "cancel", env, true)?;
        }
        Ok(())
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
                let next_open = !self.dom.has_attr(details, "open")?;
                let _ = self.set_details_open_state_with_env(details, next_open, env)?;
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
            let has_selected = self.dom.attr(option, "selected").is_some();
            if is_target {
                if !has_selected {
                    self.dom.set_attr(option, "selected", "true")?;
                }
                continue;
            }
            if !is_multiple && has_selected {
                self.dom.remove_attr(option, "selected")?;
            }
        }

        self.dom.sync_select_value(select_node)?;
        let next_value = self.dom.value(select_node)?;
        if next_value != previous_value {
            self.dispatch_event_with_env(select_node, "input", env, true)?;
            self.dispatch_event_with_env(select_node, "change", env, true)?;
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

    pub fn focus(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || self.focus_node(target))
    }

    pub fn blur(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || self.blur_node(target))
    }

    pub fn press_enter(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || {
            self.with_script_env_always(|this, env| this.press_enter_with_env(target, env))
        })
    }

    pub fn copy(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || {
            self.with_script_env_always(|this, env| this.copy_node_with_env(target, env))
        })
    }

    pub fn paste(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || {
            self.with_script_env_always(|this, env| this.paste_node_with_env(target, env))
        })
    }

    pub(crate) fn press_enter_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.is_effectively_disabled(target) {
            return Ok(());
        }

        self.focus_node_with_env(target, env)?;
        let keydown = self.dispatch_event_with_env(target, "keydown", env, true)?;
        if !keydown.default_prevented
            && self.dom.tag_name(target).is_some_and(|tag| {
                (tag.eq_ignore_ascii_case("a") && self.dom.attr(target, "href").is_some())
                    || tag.eq_ignore_ascii_case("button")
            })
        {
            self.click_node_with_env(target, env)?;
        }
        let _ = self.dispatch_event_with_env(target, "keyup", env, true)?;
        Ok(())
    }

    fn text_slice_by_char_range(text: &str, start: usize, end: usize) -> String {
        text.chars()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect()
    }

    fn selected_text_for_copy(&self, node: NodeId) -> Result<Option<String>> {
        if !self.node_supports_text_selection(node) {
            return Ok(None);
        }
        let value = self.dom.value(node)?;
        let text_len = value.chars().count();
        let start = self.dom.selection_start(node)?.min(text_len);
        let end = self.dom.selection_end(node)?.min(text_len);
        if end <= start {
            return Ok(None);
        }
        Ok(Some(Self::text_slice_by_char_range(&value, start, end)))
    }

    fn clipboard_plain_text_from_event(event: &EventState) -> Option<String> {
        let object = event.clipboard_data_object.as_ref()?;
        let entries = object.borrow();
        let store = match Self::object_get_entry(&entries, INTERNAL_CLIPBOARD_DATA_STORE_KEY) {
            Some(Value::Object(store)) => store,
            _ => return None,
        };
        Self::object_get_entry(&store.borrow(), "text/plain").map(|value| value.as_string())
    }

    pub(crate) fn copy_node_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.is_effectively_disabled(target) {
            return Ok(());
        }

        let outcome = self.dispatch_event_with_env(target, "copy", env, true)?;
        if outcome.default_prevented {
            if let Some(text) = Self::clipboard_plain_text_from_event(&outcome) {
                self.platform_mocks.clipboard_text = text;
            }
            return Ok(());
        }

        if let Some(selected) = self.selected_text_for_copy(target)? {
            self.platform_mocks.clipboard_text = selected;
        }
        Ok(())
    }

    fn paste_contenteditable_host(&self, node: NodeId) -> Option<NodeId> {
        let mut cursor = Some(node);
        while let Some(current) = cursor {
            let Some(raw) = self.dom.attr(current, "contenteditable") else {
                cursor = self.dom.parent(current);
                continue;
            };
            let normalized = raw.trim().to_ascii_lowercase();
            if normalized.is_empty() || normalized == "true" || normalized == "plaintext-only" {
                return Some(current);
            }
            if normalized == "false" {
                return None;
            }
            cursor = self.dom.parent(current);
        }
        None
    }

    pub(crate) fn paste_node_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.is_effectively_disabled(target) {
            return Ok(());
        }

        let outcome = self.dispatch_event_with_env(target, "paste", env, true)?;
        if outcome.default_prevented {
            return Ok(());
        }

        let pasted_text = outcome
            .clipboard_data
            .unwrap_or_else(|| self.platform_mocks.clipboard_text.clone());

        if self.node_supports_text_selection(target) {
            if self.dom.readonly(target) {
                return Ok(());
            }
            let before = self.dom.value(target)?;
            self.set_node_range_text(target, &[Value::String(pasted_text)])?;
            let after = self.dom.value(target)?;
            if after != before {
                self.dispatch_event_with_env(target, "input", env, true)?;
            }
            return Ok(());
        }

        if let Some(host) = self.paste_contenteditable_host(target) {
            let before = self.dom.text_content(host);
            let mut after = before.clone();
            after.push_str(&pasted_text);
            self.dom.set_text_content(host, &after)?;
            if after != before {
                self.dispatch_event_with_env(host, "input", env, true)?;
            }
        }

        Ok(())
    }

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
            self.maybe_close_dialog_for_form_submit_with_env(form_id, env)?;
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
        let skip_validation = self.dom.attr(form_id, "novalidate").is_some()
            || submitter.is_some_and(|node| self.dom.attr(node, "formnovalidate").is_some());

        if !skip_validation && !self.form_is_valid_for_submit(form_id)? {
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

            let is_file_input = self
                .dom
                .element(control)
                .map(is_file_input_element)
                .unwrap_or(false);
            if is_file_input {
                self.dom.set_value(control, "")?;
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
                    .map(|element| element.value.clone())
                    .unwrap_or_default();
                self.dom.set_value(control, &default_value)?;
                continue;
            }

            let default_value = self.dom.attr(control, "value").unwrap_or_default();
            self.dom.set_value(control, &default_value)?;
        }

        Ok(())
    }

    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()> {
        let target = self.resolve_dispatch_target(selector)?;
        self.with_script_env(|this, env| {
            stacker::grow(32 * 1024 * 1024, || {
                let _ = this.dispatch_event_with_env(target, event, env, false)?;
                Ok(())
            })
        })
    }

    pub fn dispatch_keyboard(
        &mut self,
        selector: &str,
        event: &str,
        init: KeyboardEventInit,
    ) -> Result<()> {
        let target = self.resolve_dispatch_target(selector)?;
        self.with_script_env(move |this, env| {
            stacker::grow(32 * 1024 * 1024, || {
                let mut dispatched =
                    EventState::new_untrusted(event, target, this.scheduler.now_ms);
                dispatched.key = Some(init.key.clone());
                dispatched.code = init.code.clone();
                dispatched.ctrl_key = init.ctrl_key;
                dispatched.meta_key = init.meta_key;
                dispatched.shift_key = init.shift_key;
                dispatched.alt_key = init.alt_key;
                dispatched.repeat = init.repeat;
                dispatched.is_composing = init.is_composing;
                let _ = this.dispatch_prepared_event_with_env(dispatched, env)?;
                Ok(())
            })
        })
    }

    fn resolve_dispatch_target(&self, selector: &str) -> Result<NodeId> {
        let selector = selector.trim();
        if matches!(selector, "window" | "document" | "window.document") {
            return Ok(self.dom.root);
        }
        self.select_one(selector)
    }
}
