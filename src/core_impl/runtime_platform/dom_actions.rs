use super::*;

impl Harness {
    pub fn set_trace_stderr(&mut self, enabled: bool) {
        self.trace_state.to_stderr = enabled;
    }

    pub fn set_trace_events(&mut self, enabled: bool) {
        self.trace_state.events = enabled;
    }

    pub fn set_trace_timers(&mut self, enabled: bool) {
        self.trace_state.timers = enabled;
    }

    pub fn set_trace_log_limit(&mut self, max_entries: usize) -> Result<()> {
        if max_entries == 0 {
            return Err(Error::ScriptRuntime(
                "set_trace_log_limit requires at least 1 entry".into(),
            ));
        }
        self.trace_state.log_limit = max_entries;
        while self.trace_state.logs.len() > self.trace_state.log_limit {
            self.trace_state.logs.pop_front();
        }
        Ok(())
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.rng_state = if seed == 0 {
            0xA5A5_A5A5_A5A5_A5A5
        } else {
            seed
        };
    }

    pub fn set_fetch_mock(&mut self, url: &str, body: &str) {
        self.platform_mocks.fetch_mocks.insert(url.to_string(), body.to_string());
    }

    pub fn set_clipboard_text(&mut self, text: &str) {
        self.platform_mocks.clipboard_text = text.to_string();
    }

    pub fn clipboard_text(&self) -> String {
        self.platform_mocks.clipboard_text.clone()
    }

    pub fn set_location_mock_page(&mut self, url: &str, html: &str) {
        let normalized = self.resolve_location_target_url(url);
        self.location_history.location_mock_pages
            .insert(normalized, html.to_string());
    }

    pub fn clear_location_mock_pages(&mut self) {
        self.location_history.location_mock_pages.clear();
    }

    pub fn take_location_navigations(&mut self) -> Vec<LocationNavigation> {
        std::mem::take(&mut self.location_history.location_navigations)
    }

    pub fn location_reload_count(&self) -> usize {
        self.location_history.location_reload_count
    }

    pub fn clear_fetch_mocks(&mut self) {
        self.platform_mocks.fetch_mocks.clear();
    }

    pub fn take_fetch_calls(&mut self) -> Vec<String> {
        std::mem::take(&mut self.platform_mocks.fetch_calls)
    }

    pub fn set_match_media_mock(&mut self, query: &str, matches: bool) {
        self.platform_mocks.match_media_mocks.insert(query.to_string(), matches);
    }

    pub fn clear_match_media_mocks(&mut self) {
        self.platform_mocks.match_media_mocks.clear();
    }

    pub fn set_default_match_media_matches(&mut self, matches: bool) {
        self.platform_mocks.default_match_media_matches = matches;
    }

    pub fn take_match_media_calls(&mut self) -> Vec<String> {
        std::mem::take(&mut self.platform_mocks.match_media_calls)
    }

    pub fn enqueue_confirm_response(&mut self, accepted: bool) {
        self.platform_mocks.confirm_responses.push_back(accepted);
    }

    pub fn set_default_confirm_response(&mut self, accepted: bool) {
        self.platform_mocks.default_confirm_response = accepted;
    }

    pub fn enqueue_prompt_response(&mut self, value: Option<&str>) {
        self.platform_mocks.prompt_responses
            .push_back(value.map(std::string::ToString::to_string));
    }

    pub fn set_default_prompt_response(&mut self, value: Option<&str>) {
        self.platform_mocks.default_prompt_response = value.map(std::string::ToString::to_string);
    }

    pub fn take_alert_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.platform_mocks.alert_messages)
    }

    pub fn set_timer_step_limit(&mut self, max_steps: usize) -> Result<()> {
        if max_steps == 0 {
            return Err(Error::ScriptRuntime(
                "set_timer_step_limit requires at least 1 step".into(),
            ));
        }
        self.scheduler.timer_step_limit = max_steps;
        Ok(())
    }

    fn input_supports_required(kind: &str) -> bool {
        !matches!(
            kind,
            "hidden" | "range" | "color" | "button" | "submit" | "reset" | "image"
        )
    }

    fn is_labelable_control(&self, node: NodeId) -> bool {
        let Some(tag) = self.dom.tag_name(node) else {
            return false;
        };

        if tag.eq_ignore_ascii_case("input") {
            let input_type = self
                .dom
                .attr(node, "type")
                .unwrap_or_else(|| "text".to_string())
                .to_ascii_lowercase();
            return input_type != "hidden";
        }

        tag.eq_ignore_ascii_case("button")
            || tag.eq_ignore_ascii_case("select")
            || tag.eq_ignore_ascii_case("textarea")
    }

    fn resolve_label_control(&self, label: NodeId) -> Option<NodeId> {
        if !self
            .dom
            .tag_name(label)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("label"))
        {
            return None;
        }

        if let Some(target_id) = self.dom.attr(label, "for") {
            if let Some(target) = self.dom.by_id(&target_id) {
                if self.is_labelable_control(target) {
                    return Some(target);
                }
            }
        }

        let mut descendants = Vec::new();
        self.dom.collect_elements_descendants_dfs(label, &mut descendants);
        descendants
            .into_iter()
            .find(|candidate| self.is_labelable_control(*candidate))
    }

    fn resolve_details_for_summary_click(&self, target: NodeId) -> Option<NodeId> {
        let summary = if self
            .dom
            .tag_name(target)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("summary"))
        {
            Some(target)
        } else {
            self.dom.find_ancestor_by_tag(target, "summary")
        }?;

        let details = self.dom.parent(summary)?;
        if !self
            .dom
            .tag_name(details)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
        {
            return None;
        }

        let first_summary_child = self.dom.nodes[details.0].children.iter().copied().find(|node| {
            self.dom
                .tag_name(*node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("summary"))
        });
        if first_summary_child != Some(summary) {
            return None;
        }
        Some(details)
    }

    pub(crate) fn is_effectively_disabled(&self, node: NodeId) -> bool {
        if self.dom.disabled(node) {
            return true;
        }
        if !is_form_control(&self.dom, node) {
            return false;
        }

        let mut cursor = self.dom.parent(node);
        while let Some(parent) = cursor {
            if self
                .dom
                .tag_name(parent)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("fieldset"))
                && self.dom.disabled(parent)
            {
                return true;
            }
            cursor = self.dom.parent(parent);
        }

        false
    }

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
                    details,
                    "toggle",
                    env,
                    true,
                    false,
                    false,
                    None,
                    None,
                    None,
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

    fn request_form_submit_node_with_env(
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

    pub fn now_ms(&self) -> i64 {
        self.scheduler.now_ms
    }

    pub fn clear_timer(&mut self, timer_id: i64) -> bool {
        let existed = self.scheduler.running_timer_id == Some(timer_id)
            || self.scheduler.task_queue.iter().any(|task| task.id == timer_id);
        self.clear_timeout(timer_id);
        existed
    }

    pub fn clear_all_timers(&mut self) -> usize {
        let cleared = self.scheduler.task_queue.len();
        self.scheduler.task_queue.clear();
        if self.scheduler.running_timer_id.is_some() {
            self.scheduler.running_timer_canceled = true;
        }
        self.trace_timer_line(format!("[timer] clear_all cleared={cleared}"));
        cleared
    }

    pub fn pending_timers(&self) -> Vec<PendingTimer> {
        let mut timers = self
            .scheduler
            .task_queue
            .iter()
            .map(|task| PendingTimer {
                id: task.id,
                due_at: task.due_at,
                order: task.order,
                interval_ms: task.interval_ms,
            })
            .collect::<Vec<_>>();
        timers.sort_by_key(|timer| (timer.due_at, timer.order));
        timers
    }

    pub fn advance_time(&mut self, delta_ms: i64) -> Result<()> {
        if delta_ms < 0 {
            return Err(Error::ScriptRuntime(
                "advance_time requires non-negative milliseconds".into(),
            ));
        }
        let from = self.scheduler.now_ms;
        self.scheduler.now_ms = self.scheduler.now_ms.saturating_add(delta_ms);
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] advance delta_ms={} from={} to={} ran_due={}",
            delta_ms, from, self.scheduler.now_ms, ran
        ));
        Ok(())
    }

    pub fn advance_time_to(&mut self, target_ms: i64) -> Result<()> {
        if target_ms < self.scheduler.now_ms {
            return Err(Error::ScriptRuntime(format!(
                "advance_time_to requires target >= now_ms (target={target_ms}, now_ms={})",
                self.scheduler.now_ms
            )));
        }
        let from = self.scheduler.now_ms;
        self.scheduler.now_ms = target_ms;
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] advance_to from={} to={} ran_due={}",
            from, self.scheduler.now_ms, ran
        ));
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        let from = self.scheduler.now_ms;
        let ran = self.run_timer_queue(None, true)?;
        self.trace_timer_line(format!(
            "[timer] flush from={} to={} ran={}",
            from, self.scheduler.now_ms, ran
        ));
        Ok(())
    }

    pub fn run_next_timer(&mut self) -> Result<bool> {
        let Some(next_idx) = self.next_task_index(None) else {
            self.trace_timer_line("[timer] run_next none".into());
            return Ok(false);
        };

        let task = self.scheduler.task_queue.remove(next_idx);
        if task.due_at > self.scheduler.now_ms {
            self.scheduler.now_ms = task.due_at;
        }
        self.execute_timer_task(task)?;
        Ok(true)
    }

    pub fn run_next_due_timer(&mut self) -> Result<bool> {
        let Some(next_idx) = self.next_task_index(Some(self.scheduler.now_ms)) else {
            self.trace_timer_line("[timer] run_next_due none".into());
            return Ok(false);
        };

        let task = self.scheduler.task_queue.remove(next_idx);
        self.execute_timer_task(task)?;
        Ok(true)
    }

    pub fn run_due_timers(&mut self) -> Result<usize> {
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] run_due now_ms={} ran={}",
            self.scheduler.now_ms, ran
        ));
        Ok(ran)
    }

    pub(crate) fn run_due_timers_internal(&mut self) -> Result<usize> {
        self.run_timer_queue(Some(self.scheduler.now_ms), false)
    }

    pub(crate) fn run_timer_queue(
        &mut self,
        due_limit: Option<i64>,
        advance_clock: bool,
    ) -> Result<usize> {
        let mut steps = 0usize;
        while let Some(next_idx) = self.next_task_index(due_limit) {
            steps += 1;
            if steps > self.scheduler.timer_step_limit {
                return Err(self.timer_step_limit_error(
                    self.scheduler.timer_step_limit,
                    steps,
                    due_limit,
                ));
            }
            let task = self.scheduler.task_queue.remove(next_idx);
            if advance_clock && task.due_at > self.scheduler.now_ms {
                self.scheduler.now_ms = task.due_at;
            }
            self.execute_timer_task(task)?;
        }
        Ok(steps)
    }

    pub(crate) fn timer_step_limit_error(
        &self,
        max_steps: usize,
        steps: usize,
        due_limit: Option<i64>,
    ) -> Error {
        let due_limit_desc = due_limit
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".into());

        let next_task_desc = self
            .next_task_index(due_limit)
            .and_then(|idx| self.scheduler.task_queue.get(idx))
            .map(|task| {
                let interval_desc = task
                    .interval_ms
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".into());
                format!(
                    "id={},due_at={},order={},interval_ms={}",
                    task.id, task.due_at, task.order, interval_desc
                )
            })
            .unwrap_or_else(|| "none".into());

        Error::ScriptRuntime(format!(
            "flush exceeded max task steps (possible uncleared setInterval): limit={max_steps}, steps={steps}, now_ms={}, due_limit={}, pending_tasks={}, next_task={}",
            self.scheduler.now_ms,
            due_limit_desc,
            self.scheduler.task_queue.len(),
            next_task_desc
        ))
    }

    pub(crate) fn next_task_index(&self, due_limit: Option<i64>) -> Option<usize> {
        self.scheduler.task_queue
            .iter()
            .enumerate()
            .filter(|(_, task)| {
                if let Some(limit) = due_limit {
                    task.due_at <= limit
                } else {
                    true
                }
            })
            .min_by_key(|(_, task)| (task.due_at, task.order))
            .map(|(idx, _)| idx)
    }

    pub(crate) fn execute_timer_task(&mut self, task: ScheduledTask) -> Result<()> {
        stacker::grow(32 * 1024 * 1024, || self.execute_timer_task_impl(task))
    }

    fn execute_timer_task_impl(&mut self, mut task: ScheduledTask) -> Result<()> {
        let interval_desc = task
            .interval_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".into());
        self.trace_timer_line(format!(
            "[timer] run id={} due_at={} interval_ms={} now_ms={}",
            task.id, task.due_at, interval_desc, self.scheduler.now_ms
        ));

        self.scheduler.running_timer_id = Some(task.id);
        self.scheduler.running_timer_canceled = false;
        let mut event = EventState::new("timeout", self.dom.root, self.scheduler.now_ms);
        self.run_in_task_context(|this| {
            this.execute_timer_task_callback(
                &task.callback,
                &task.callback_args,
                &mut event,
                &mut task.env,
            )
            .map(|_| ())
        })?;
        let canceled = self.scheduler.running_timer_canceled;
        self.scheduler.running_timer_id = None;
        self.scheduler.running_timer_canceled = false;

        if let Some(interval_ms) = task.interval_ms {
            if !canceled {
                let delay_ms = interval_ms.max(0);
                let due_at = task.due_at.saturating_add(delay_ms);
                let order = self.scheduler.allocate_task_order();
                self.scheduler.task_queue.push(ScheduledTask {
                    id: task.id,
                    due_at,
                    order,
                    interval_ms: Some(delay_ms),
                    callback: task.callback,
                    callback_args: task.callback_args,
                    env: task.env,
                });
                self.trace_timer_line(format!(
                    "[timer] requeue id={} due_at={} interval_ms={}",
                    task.id, due_at, delay_ms
                ));
            }
        }

        Ok(())
    }

    pub fn assert_text(&self, selector: &str, expected: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.text_content(target);
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual,
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_value(&self, selector: &str, expected: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.value(target)?;
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual,
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_checked(&self, selector: &str, expected: bool) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.checked(target)?;
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual: actual.to_string(),
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_exists(&self, selector: &str) -> Result<()> {
        let _ = self.select_one(selector)?;
        Ok(())
    }

    pub fn dump_dom(&self, selector: &str) -> Result<String> {
        let target = self.select_one(selector)?;
        Ok(self.dom.dump_node(target))
    }

    pub(crate) fn select_one(&self, selector: &str) -> Result<NodeId> {
        self.dom
            .query_selector(selector)?
            .ok_or_else(|| Error::SelectorNotFound(selector.to_string()))
    }

    pub(crate) fn node_snippet(&self, node_id: NodeId) -> String {
        truncate_chars(&self.dom.dump_node(node_id), 200)
    }

    pub(crate) fn resolve_form_for_submit(&self, target: NodeId) -> Option<NodeId> {
        if self
            .dom
            .tag_name(target)
            .map(|t| t.eq_ignore_ascii_case("form"))
            .unwrap_or(false)
        {
            return Some(target);
        }
        self.dom.find_ancestor_by_tag(target, "form")
    }

    pub(crate) fn resolve_submit_form_target(&self, target: NodeId) -> Option<NodeId> {
        self.resolve_form_for_submit(target)
    }

    pub(crate) fn resolve_request_submitter_node(
        &self,
        submitter: Option<Value>,
    ) -> Result<Option<NodeId>> {
        match submitter {
            None | Some(Value::Undefined) | Some(Value::Null) => Ok(None),
            Some(Value::Node(node)) => Ok(Some(node)),
            Some(_) => Err(Error::ScriptRuntime(
                "requestSubmit submitter must be an element".into(),
            )),
        }
    }

    pub(crate) fn form_elements(&self, form: NodeId) -> Result<Vec<NodeId>> {
        let tag = self
            .dom
            .tag_name(form)
            .ok_or_else(|| Error::ScriptRuntime("elements target is not an element".into()))?;
        if !tag.eq_ignore_ascii_case("form") {
            return Err(Error::ScriptRuntime(format!(
                "{}.elements target is not a form",
                self.event_node_label(form)
            )));
        }

        let mut out = Vec::new();
        self.collect_form_controls(form, &mut out);
        Ok(out)
    }

    pub(crate) fn form_data_entries(&self, form: NodeId) -> Result<Vec<(String, String)>> {
        let mut out = Vec::new();
        for control in self.form_elements(form)? {
            if !self.is_successful_form_data_control(control)? {
                continue;
            }
            let name = self.dom.attr(control, "name").unwrap_or_default();
            let value = self.form_data_control_value(control)?;
            out.push((name, value));
        }
        Ok(out)
    }

    pub(crate) fn is_successful_form_data_control(&self, control: NodeId) -> Result<bool> {
        if self.is_effectively_disabled(control) {
            return Ok(false);
        }
        let name = self.dom.attr(control, "name").unwrap_or_default();
        if name.is_empty() {
            return Ok(false);
        }

        let tag = self
            .dom
            .tag_name(control)
            .ok_or_else(|| Error::ScriptRuntime("FormData target is not an element".into()))?;

        if tag.eq_ignore_ascii_case("button") {
            return Ok(false);
        }

        if tag.eq_ignore_ascii_case("input") {
            let kind = self
                .dom
                .attr(control, "type")
                .unwrap_or_default()
                .to_ascii_lowercase();
            if matches!(
                kind.as_str(),
                "button" | "submit" | "reset" | "file" | "image"
            ) {
                return Ok(false);
            }
            if kind == "checkbox" || kind == "radio" {
                return self.dom.checked(control);
            }
        }

        Ok(true)
    }

    pub(crate) fn form_data_control_value(&self, control: NodeId) -> Result<String> {
        self.dom.value(control)
    }

    pub(crate) fn form_is_valid_for_submit(&self, form: NodeId) -> Result<bool> {
        let controls = self.form_elements(form)?;
        for control in &controls {
            if !self.required_control_satisfied(*control, &controls)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(crate) fn required_control_satisfied(
        &self,
        control: NodeId,
        controls: &[NodeId],
    ) -> Result<bool> {
        if self.is_effectively_disabled(control) || !self.dom.required(control) {
            return Ok(true);
        }

        let tag = self
            .dom
            .tag_name(control)
            .ok_or_else(|| Error::ScriptRuntime("required target is not an element".into()))?;

        if tag.eq_ignore_ascii_case("input") {
            let kind = self
                .dom
                .attr(control, "type")
                .unwrap_or_else(|| "text".into())
                .to_ascii_lowercase();
            if !Self::input_supports_required(kind.as_str()) {
                return Ok(true);
            }
            if kind == "checkbox" {
                return self.dom.checked(control);
            }
            if kind == "radio" {
                if self.dom.checked(control)? {
                    return Ok(true);
                }
                let name = self.dom.attr(control, "name").unwrap_or_default();
                if name.is_empty() {
                    return Ok(false);
                }
                for candidate in controls {
                    if *candidate == control {
                        continue;
                    }
                    if !is_radio_input(&self.dom, *candidate) {
                        continue;
                    }
                    if self.dom.attr(*candidate, "name").unwrap_or_default() != name {
                        continue;
                    }
                    if self.dom.checked(*candidate)? {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            return Ok(!self.dom.value(control)?.is_empty());
        }

        if tag.eq_ignore_ascii_case("select") || tag.eq_ignore_ascii_case("textarea") {
            return Ok(!self.dom.value(control)?.is_empty());
        }

        Ok(true)
    }

    pub(crate) fn eval_form_data_source(
        &mut self,
        source: &FormDataSource,
        env: &HashMap<String, Value>,
    ) -> Result<Vec<(String, String)>> {
        match source {
            FormDataSource::NewForm(form) => {
                let form_node = self.resolve_dom_query_required_runtime(form, env)?;
                self.form_data_entries(form_node)
            }
            FormDataSource::Var(name) => match env.get(name) {
                Some(Value::FormData(entries)) => Ok(entries.clone()),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a FormData instance",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown FormData variable: {}",
                    name
                ))),
            },
        }
    }

    pub(crate) fn collect_form_controls(&self, node: NodeId, out: &mut Vec<NodeId>) {
        for child in &self.dom.nodes[node.0].children {
            if is_form_control(&self.dom, *child) {
                out.push(*child);
            }
            self.collect_form_controls(*child, out);
        }
    }

    pub(crate) fn dispatch_event(
        &mut self,
        target: NodeId,
        event_type: &str,
    ) -> Result<EventState> {
        self.with_script_env(|this, env| this.dispatch_event_with_env(target, event_type, env, true))
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
