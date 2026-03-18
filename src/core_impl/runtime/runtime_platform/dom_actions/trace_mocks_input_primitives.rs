use super::*;
use crate::core_dom_utils::{decode_base64_to_binary_string, decode_uri_like};

/// Determinism, mock, and trace APIs for a [`Harness`].
impl Harness {
    pub(crate) fn default_fetch_status_text(status: i64) -> String {
        match status {
            100 => "Continue",
            101 => "Switching Protocols",
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            203 => "Non-Authoritative Information",
            204 => "No Content",
            205 => "Reset Content",
            206 => "Partial Content",
            300 => "Multiple Choices",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            307 => "Temporary Redirect",
            308 => "Permanent Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            408 => "Request Timeout",
            409 => "Conflict",
            410 => "Gone",
            413 => "Payload Too Large",
            415 => "Unsupported Media Type",
            418 => "I'm a teapot",
            422 => "Unprocessable Content",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            _ => "",
        }
        .to_string()
    }

    /// Enable or disable trace log emission to stderr.
    pub fn set_trace_stderr(&mut self, enabled: bool) {
        self.trace_state.to_stderr = enabled;
    }

    /// Enable or disable event trace collection.
    pub fn set_trace_events(&mut self, enabled: bool) {
        self.trace_state.events = enabled;
    }

    /// Enable or disable timer trace collection.
    pub fn set_trace_timers(&mut self, enabled: bool) {
        self.trace_state.timers = enabled;
    }

    /// Set the maximum number of retained trace log entries.
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

    /// Seed the deterministic random number generator.
    pub fn set_random_seed(&mut self, seed: u64) {
        self.rng_state = if seed == 0 {
            0xA5A5_A5A5_A5A5_A5A5
        } else {
            seed
        };
    }

    /// Mock a successful text response for `fetch(url)`.
    pub fn set_fetch_mock(&mut self, url: &str, body: &str) {
        self.platform_mocks.fetch_mocks.insert(
            url.to_string(),
            FetchMockResponse {
                status: 200,
                status_text: "OK".to_string(),
                body: body.to_string(),
            },
        );
    }

    /// Mock a `fetch(url)` response with an explicit status code and body.
    pub fn set_fetch_mock_response(&mut self, url: &str, status: i64, body: &str) {
        self.platform_mocks.fetch_mocks.insert(
            url.to_string(),
            FetchMockResponse {
                status,
                status_text: Self::default_fetch_status_text(status),
                body: body.to_string(),
            },
        );
    }

    /// Seed deterministic clipboard text for subsequent reads or user actions.
    pub fn set_clipboard_text(&mut self, text: &str) {
        self.platform_mocks.clipboard_text = text.to_string();
    }

    /// Return the currently seeded clipboard text.
    pub fn clipboard_text(&self) -> String {
        self.platform_mocks.clipboard_text.clone()
    }

    /// Inject a deterministic clipboard read rejection by error name.
    pub fn set_clipboard_read_error(&mut self, error: Option<&str>) {
        self.platform_mocks.clipboard_read_error = error.map(std::string::ToString::to_string);
    }

    /// Inject a deterministic clipboard write rejection by error name.
    pub fn set_clipboard_write_error(&mut self, error: Option<&str>) {
        self.platform_mocks.clipboard_write_error = error.map(std::string::ToString::to_string);
    }

    /// Clear injected clipboard read and write errors.
    pub fn clear_clipboard_errors(&mut self) {
        self.platform_mocks.clipboard_read_error = None;
        self.platform_mocks.clipboard_write_error = None;
    }

    /// Register deterministic HTML to load when navigating to `url`.
    pub fn set_location_mock_page(&mut self, url: &str, html: &str) {
        let normalized = self.resolve_location_target_url(url);
        self.location_history
            .location_mock_pages
            .insert(normalized, html.to_string());
    }

    /// Remove all registered location mock pages.
    pub fn clear_location_mock_pages(&mut self) {
        self.location_history.location_mock_pages.clear();
    }

    /// Drain and return captured location navigation records.
    pub fn take_location_navigations(&mut self) -> Vec<LocationNavigation> {
        std::mem::take(&mut self.location_history.location_navigations)
    }

    /// Drain and return captured download artifacts.
    pub fn take_downloads(&mut self) -> Vec<DownloadArtifact> {
        std::mem::take(&mut self.browser_apis.downloads)
    }

    /// Drain and return captured clipboard write artifacts.
    pub fn take_clipboard_writes(&mut self) -> Vec<ClipboardWriteArtifact> {
        std::mem::take(&mut self.browser_apis.clipboard_writes)
    }

    /// Return the number of deterministic reloads performed through location APIs.
    pub fn location_reload_count(&self) -> usize {
        self.location_history.location_reload_count
    }

    /// Remove all registered fetch mocks.
    pub fn clear_fetch_mocks(&mut self) {
        self.platform_mocks.fetch_mocks.clear();
    }

    /// Drain and return captured fetch call URLs.
    pub fn take_fetch_calls(&mut self) -> Vec<String> {
        std::mem::take(&mut self.platform_mocks.fetch_calls)
    }

    /// Override `matchMedia(query).matches` for a specific query.
    pub fn set_match_media_mock(&mut self, query: &str, matches: bool) {
        self.platform_mocks
            .match_media_mocks
            .insert(query.to_string(), matches);
    }

    /// Remove all query-specific `matchMedia` overrides.
    pub fn clear_match_media_mocks(&mut self) {
        self.platform_mocks.match_media_mocks.clear();
    }

    /// Set the fallback `matchMedia(...).matches` value when no query-specific mock exists.
    pub fn set_default_match_media_matches(&mut self, matches: bool) {
        self.platform_mocks.default_match_media_matches = matches;
    }

    /// Drain and return captured `matchMedia` query strings.
    pub fn take_match_media_calls(&mut self) -> Vec<String> {
        std::mem::take(&mut self.platform_mocks.match_media_calls)
    }

    /// Queue one deterministic `confirm()` response.
    pub fn enqueue_confirm_response(&mut self, accepted: bool) {
        self.platform_mocks.confirm_responses.push_back(accepted);
    }

    /// Set the default `confirm()` response when the queue is empty.
    pub fn set_default_confirm_response(&mut self, accepted: bool) {
        self.platform_mocks.default_confirm_response = accepted;
    }

    /// Queue one deterministic `prompt()` response.
    pub fn enqueue_prompt_response(&mut self, value: Option<&str>) {
        self.platform_mocks
            .prompt_responses
            .push_back(value.map(std::string::ToString::to_string));
    }

    /// Set the default `prompt()` response when the queue is empty.
    pub fn set_default_prompt_response(&mut self, value: Option<&str>) {
        self.platform_mocks.default_prompt_response = value.map(std::string::ToString::to_string);
    }

    /// Drain and return captured `alert()` messages.
    pub fn take_alert_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.platform_mocks.alert_messages)
    }

    /// Drain and return the captured `window.print()` call count.
    pub fn take_print_call_count(&mut self) -> usize {
        std::mem::take(&mut self.platform_mocks.print_call_count)
    }

    /// Set the scheduler safety limit used by timer-draining APIs.
    pub fn set_timer_step_limit(&mut self, max_steps: usize) -> Result<()> {
        if max_steps == 0 {
            return Err(Error::ScriptRuntime(
                "set_timer_step_limit requires at least 1 step".into(),
            ));
        }
        self.scheduler.timer_step_limit = max_steps;
        Ok(())
    }

    pub(crate) fn input_supports_required(kind: &str) -> bool {
        !matches!(
            kind,
            "hidden" | "range" | "color" | "button" | "submit" | "reset" | "image"
        )
    }

    pub(crate) fn is_labelable_control(&self, node: NodeId) -> bool {
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
            || tag.eq_ignore_ascii_case("output")
    }

    pub(crate) fn resolve_label_control(&self, label: NodeId) -> Option<NodeId> {
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

    pub(crate) fn resolve_label_activation_control(&self, target: NodeId) -> Option<NodeId> {
        if self.is_labelable_control(target) {
            return None;
        }

        let label = if self
            .dom
            .tag_name(target)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("label"))
        {
            target
        } else {
            self.dom.find_ancestor_by_tag(target, "label")?
        };

        let mut cursor = self.dom.parent(target);
        while let Some(node) = cursor {
            if node == label {
                break;
            }
            if self.is_labelable_control(node) {
                return None;
            }
            cursor = self.dom.parent(node);
        }

        self.resolve_label_control(label)
            .filter(|control| *control != target)
    }

    pub(crate) fn resolve_details_for_summary_click(&self, target: NodeId) -> Option<NodeId> {
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

    pub(crate) fn is_hyperlink_element(&self, node: NodeId) -> bool {
        self.dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area"))
    }

    fn parse_data_url_download_artifact(href: &str) -> Result<Option<(Option<String>, Vec<u8>)>> {
        let Some((scheme, rest)) = href.split_once(':') else {
            return Ok(None);
        };
        if !scheme.eq_ignore_ascii_case("data") {
            return Ok(None);
        }

        let Some((meta, payload)) = rest.split_once(',') else {
            return Ok(None);
        };
        let is_base64 = meta
            .split(';')
            .skip(1)
            .any(|part| part.trim().eq_ignore_ascii_case("base64"));
        let media_type = meta
            .split(';')
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_ascii_lowercase())
            .or_else(|| Some("text/plain".to_string()));
        let bytes = if is_base64 {
            decode_base64_to_binary_string(payload)?
                .chars()
                .map(|ch| ch as u32 as u8)
                .collect()
        } else {
            decode_uri_like(payload, true)?.into_bytes()
        };
        Ok(Some((media_type, bytes)))
    }

    pub(crate) fn maybe_capture_anchor_download(&mut self, target: NodeId) -> Result<bool> {
        if !self.is_hyperlink_element(target) {
            return Ok(false);
        }
        let Some(filename) = self.dom.attr(target, "download") else {
            return Ok(false);
        };

        let href = self.resolve_anchor_href(target);
        let (mime_type, bytes) = if let Some(blob) =
            self.browser_apis.blob_url_objects.get(&href).cloned()
        {
            let blob = blob.borrow();
            let mime_type = if blob.mime_type.is_empty() {
                None
            } else {
                Some(blob.mime_type.clone())
            };
            (mime_type, blob.bytes.clone())
        } else if let Some((mime_type, bytes)) = Self::parse_data_url_download_artifact(&href)? {
            (mime_type, bytes)
        } else {
            return Ok(false);
        };

        self.browser_apis.downloads.push(DownloadArtifact {
            filename: if filename.is_empty() {
                None
            } else {
                Some(filename)
            },
            mime_type,
            bytes,
        });
        Ok(true)
    }

    pub(crate) fn maybe_follow_anchor_hyperlink(&mut self, target: NodeId) -> Result<()> {
        if !self.is_hyperlink_element(target) {
            return Ok(());
        }
        if self
            .dom
            .tag_name(target)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("area"))
            && self.dom.attr(target, "nohref").is_some()
        {
            return Ok(());
        }
        if self.dom.attr(target, "href").is_none() {
            return Ok(());
        }
        // If download is present, current-context navigation is suppressed.
        if self.dom.attr(target, "download").is_some() {
            return Ok(());
        }

        let target_attr = self
            .dom
            .attr(target, "target")
            .unwrap_or_else(|| self.default_hyperlink_target())
            .to_ascii_lowercase();
        if !matches!(
            target_attr.as_str(),
            "" | "_self" | "_parent" | "_top" | "_unfencedtop"
        ) {
            return Ok(());
        }

        let href = self.resolve_anchor_href(target);
        if href
            .split_once(':')
            .is_some_and(|(scheme, _)| scheme.eq_ignore_ascii_case("javascript"))
        {
            return Ok(());
        }
        if self.try_resolve_location_target_url(&href).is_err() {
            return Ok(());
        }
        self.navigate_location(&href, LocationNavigationKind::Assign)
    }

    pub(crate) fn is_within_first_legend_of_fieldset(
        &self,
        node: NodeId,
        fieldset: NodeId,
    ) -> bool {
        let first_legend = self.dom.child_elements(fieldset).into_iter().find(|child| {
            self.dom
                .tag_name(*child)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("legend"))
        });
        let Some(first_legend) = first_legend else {
            return false;
        };

        node == first_legend || self.dom.is_descendant_of(node, first_legend)
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
                if self.is_within_first_legend_of_fieldset(node, parent) {
                    cursor = self.dom.parent(parent);
                    continue;
                }
                return true;
            }
            cursor = self.dom.parent(parent);
        }

        false
    }
}
