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
        self.platform_mocks
            .fetch_mocks
            .insert(url.to_string(), body.to_string());
    }

    pub fn set_clipboard_text(&mut self, text: &str) {
        self.platform_mocks.clipboard_text = text.to_string();
    }

    pub fn clipboard_text(&self) -> String {
        self.platform_mocks.clipboard_text.clone()
    }

    pub fn set_location_mock_page(&mut self, url: &str, html: &str) {
        let normalized = self.resolve_location_target_url(url);
        self.location_history
            .location_mock_pages
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
        self.platform_mocks
            .match_media_mocks
            .insert(query.to_string(), matches);
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
        self.platform_mocks
            .prompt_responses
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
        self.dom
            .collect_elements_descendants_dfs(label, &mut descendants);
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

        let first_summary_child = self.dom.nodes[details.0]
            .children
            .iter()
            .copied()
            .find(|node| {
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

}
