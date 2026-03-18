use super::*;

/// Platform mock and artifact APIs for a [`Harness`].
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

    /// Remove all registered fetch mocks.
    pub fn clear_fetch_mocks(&mut self) {
        self.platform_mocks.fetch_mocks.clear();
    }

    /// Drain and return captured fetch call URLs.
    pub fn take_fetch_calls(&mut self) -> Vec<String> {
        std::mem::take(&mut self.platform_mocks.fetch_calls)
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

    /// Drain and return captured clipboard write artifacts.
    pub fn take_clipboard_writes(&mut self) -> Vec<ClipboardWriteArtifact> {
        std::mem::take(&mut self.browser_apis.clipboard_writes)
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

    /// Return the number of deterministic reloads performed through location APIs.
    pub fn location_reload_count(&self) -> usize {
        self.location_history.location_reload_count
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

    /// Drain and return captured download artifacts.
    pub fn take_downloads(&mut self) -> Vec<DownloadArtifact> {
        std::mem::take(&mut self.browser_apis.downloads)
    }
}
