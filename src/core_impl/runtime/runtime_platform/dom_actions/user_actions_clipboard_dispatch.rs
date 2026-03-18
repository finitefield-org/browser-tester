use super::*;

impl Harness {
    /// Move focus to the selected element.
    pub fn focus(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || self.focus_node(target))
    }

    /// Blur the selected element.
    pub fn blur(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || self.blur_node(target))
    }

    /// Simulate pressing Enter on the selected element.
    pub fn press_enter(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || {
            self.with_script_env_always(|this, env| this.press_enter_with_env(target, env))
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
        if !keydown.default_prevented {
            let activates_click = self.dom.tag_name(target).is_some_and(|tag| {
                (tag.eq_ignore_ascii_case("a") && self.dom.attr(target, "href").is_some())
                    || tag.eq_ignore_ascii_case("button")
            }) || is_submit_control(&self.dom, target);

            if activates_click {
                self.click_node_with_env(target, env)?;
            } else if self.supports_implicit_submit_on_enter(target) {
                if let Some(form_id) = self.resolve_form_for_submit(target) {
                    let submitter = self.default_submitter_for_form(form_id)?;
                    self.request_form_submit_node_with_env(form_id, submitter, env)?;
                }
            }
        }
        let _ = self.dispatch_event_with_env(target, "keyup", env, true)?;
        Ok(())
    }

    fn supports_implicit_submit_on_enter(&self, target: NodeId) -> bool {
        let Some(tag) = self.dom.tag_name(target) else {
            return false;
        };
        if !tag.eq_ignore_ascii_case("input") {
            return false;
        }

        matches!(
            self.normalized_input_type(target).as_str(),
            "text"
                | "search"
                | "url"
                | "tel"
                | "email"
                | "password"
                | "number"
                | "date"
                | "time"
                | "datetime-local"
        )
    }

    /// Simulate a trusted copy action from the selected element.
    pub fn copy(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || {
            self.with_script_env_always(|this, env| this.copy_node_with_env(target, env))
        })
    }

    /// Simulate a trusted paste action into the selected element.
    pub fn paste(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || {
            self.with_script_env_always(|this, env| this.paste_node_with_env(target, env))
        })
    }

    /// Simulate a trusted cut action from the selected element.
    pub fn cut(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        stacker::grow(32 * 1024 * 1024, || {
            self.with_script_env_always(|this, env| this.cut_node_with_env(target, env))
        })
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

    fn clipboard_plain_text_for_paste_default_action(event: &EventState) -> Option<String> {
        let object = event.clipboard_data_object.as_ref()?;
        let entries = object.borrow();
        if let Some(Value::Object(store)) =
            Self::object_get_entry(&entries, INTERNAL_CLIPBOARD_DATA_STORE_KEY)
        {
            if let Some(value) = Self::object_get_entry(&store.borrow(), "text/plain") {
                return Some(value.as_string());
            }
        }
        Self::object_get_entry(&entries, INTERNAL_CLIPBOARD_DATA_TEXT_KEY)
            .map(|value| value.as_string())
    }

    pub(crate) fn copy_node_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.is_effectively_disabled(target) {
            return Ok(());
        }

        self.focus_node_with_env(target, env)?;
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

    pub(crate) fn cut_node_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.is_effectively_disabled(target) {
            return Ok(());
        }

        self.focus_node_with_env(target, env)?;
        let selected = self.selected_text_for_copy(target)?;
        let outcome = self.dispatch_event_with_env(target, "cut", env, true)?;
        if outcome.default_prevented {
            if let Some(text) = Self::clipboard_plain_text_from_event(&outcome) {
                self.platform_mocks.clipboard_text = text;
            }
            return Ok(());
        }

        let Some(selected) = selected else {
            return Ok(());
        };
        self.platform_mocks.clipboard_text = selected;

        if !self.node_supports_text_selection(target) || self.dom.readonly(target) {
            return Ok(());
        }

        let before = self.dom.value(target)?;
        self.set_node_range_text(target, &[Value::String(String::new())])?;
        let after = self.dom.value(target)?;
        if after != before {
            self.dispatch_form_control_input_with_env(target, env)?;
            self.note_user_committed_change_candidate(target)?;
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

        self.focus_node_with_env(target, env)?;
        let outcome = self.dispatch_event_with_env(target, "paste", env, true)?;
        if outcome.default_prevented {
            return Ok(());
        }

        let pasted_text = Self::clipboard_plain_text_for_paste_default_action(&outcome)
            .or(outcome.clipboard_data)
            .unwrap_or_else(|| self.platform_mocks.clipboard_text.clone());

        if self.node_supports_text_selection(target) {
            if self.dom.readonly(target) {
                return Ok(());
            }
            let before = self.dom.value(target)?;
            self.set_node_range_text(target, &[Value::String(pasted_text)])?;
            let after = self.dom.value(target)?;
            if after != before {
                self.dispatch_form_control_input_with_env(target, env)?;
                self.note_user_committed_change_candidate(target)?;
            }
            return Ok(());
        }

        if let Some(host) = self.paste_contenteditable_host(target) {
            let before = self.dom.text_content(host);
            let mut after = before.clone();
            after.push_str(&pasted_text);
            self.dom.set_text_content(host, &after)?;
            if after != before {
                self.dispatch_form_control_input_with_env(host, env)?;
            }
        }

        Ok(())
    }

    /// Dispatch a named event on the selected element.
    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()> {
        if let Some(target_object) = self.resolve_dispatch_event_target_object(selector) {
            let event_payload = Value::String(event.to_string());
            return self.with_script_env(|this, env| {
                stacker::grow(32 * 1024 * 1024, || {
                    let _ = this.dispatch_event_target_with_env(
                        target_object.clone(),
                        event_payload.clone(),
                        env,
                    )?;
                    Ok(())
                })
            });
        }
        let target = self.resolve_dispatch_target(selector)?;
        self.with_script_env(|this, env| {
            stacker::grow(32 * 1024 * 1024, || {
                let _ = this.dispatch_event_with_env(target, event, env, false)?;
                Ok(())
            })
        })
    }

    /// Dispatch a keyboard event with the provided initialization values.
    pub fn dispatch_keyboard(
        &mut self,
        selector: &str,
        event: &str,
        init: KeyboardEventInit,
    ) -> Result<()> {
        if let Some(target_object) = self.resolve_dispatch_event_target_object(selector) {
            let event_payload = Self::new_object_value(vec![
                (INTERNAL_EVENT_OBJECT_KEY.to_string(), Value::Bool(true)),
                (
                    INTERNAL_KEYBOARD_EVENT_OBJECT_KEY.to_string(),
                    Value::Bool(true),
                ),
                ("type".to_string(), Value::String(event.to_string())),
                ("bubbles".to_string(), Value::Bool(false)),
                ("cancelable".to_string(), Value::Bool(false)),
                ("key".to_string(), Value::String(init.key.clone())),
                (
                    "code".to_string(),
                    Value::String(init.code.clone().unwrap_or_default()),
                ),
                ("location".to_string(), Value::Number(init.location)),
                ("ctrlKey".to_string(), Value::Bool(init.ctrl_key)),
                ("metaKey".to_string(), Value::Bool(init.meta_key)),
                ("shiftKey".to_string(), Value::Bool(init.shift_key)),
                ("altKey".to_string(), Value::Bool(init.alt_key)),
                ("repeat".to_string(), Value::Bool(init.repeat)),
                ("isComposing".to_string(), Value::Bool(init.is_composing)),
            ]);
            return self.with_script_env(move |this, env| {
                stacker::grow(32 * 1024 * 1024, || {
                    let _ = this.dispatch_event_target_with_env(
                        target_object.clone(),
                        event_payload.clone(),
                        env,
                    )?;
                    Ok(())
                })
            });
        }
        let target = self.resolve_dispatch_target(selector)?;
        self.with_script_env(move |this, env| {
            stacker::grow(32 * 1024 * 1024, || {
                let mut dispatched =
                    EventState::new_untrusted(event, target, this.scheduler.now_ms);
                dispatched.bubbles = true;
                dispatched.cancelable = true;
                dispatched.key = Some(init.key.clone());
                dispatched.code = init.code.clone();
                dispatched.location = init.location;
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

    fn resolve_dispatch_event_target_object(
        &self,
        selector: &str,
    ) -> Option<Rc<RefCell<ObjectValue>>> {
        let selector = selector.trim();
        if !matches!(
            selector,
            "window" | "self" | "top" | "parent" | "frames" | "globalThis"
        ) {
            return None;
        }
        match self.script_runtime.env.get("window") {
            Some(Value::Object(window)) => Some(window.clone()),
            _ => None,
        }
    }

    fn resolve_dispatch_target(&self, selector: &str) -> Result<NodeId> {
        let selector = selector.trim();
        if matches!(selector, "document" | "window.document") {
            return Ok(self.dom.root);
        }
        if selector == "window" {
            return Ok(self.dom.root);
        }
        self.select_one(selector)
    }
}
