use super::*;

impl Harness {
    fn parse_media_length_px(raw: &str) -> Option<f64> {
        let normalized = raw.trim().to_ascii_lowercase();
        if let Some(value) = normalized.strip_suffix("px") {
            return value.trim().parse::<f64>().ok();
        }
        normalized.parse::<f64>().ok()
    }

    fn viewport_dimension_value(&self, key: &str) -> Option<f64> {
        let window = self.dom_runtime.window_object.borrow();
        let value = Self::object_get_entry(&window, key)?;
        match value {
            Value::Number(value) => Some(value as f64),
            Value::Float(value) if value.is_finite() => Some(value),
            Value::String(value) => value.parse::<f64>().ok(),
            _ => None,
        }
    }

    fn viewport_dimensions(&self) -> (f64, f64) {
        let width = self
            .viewport_dimension_value("innerWidth")
            .unwrap_or(1024.0);
        let height = self
            .viewport_dimension_value("innerHeight")
            .unwrap_or(768.0);
        (width, height)
    }

    fn preferred_color_scheme(&self) -> String {
        let window = self.dom_runtime.window_object.borrow();
        let value = Self::object_get_entry(&window, "prefersColorScheme")
            .or_else(|| Self::object_get_entry(&window, "colorScheme"));
        match value {
            Some(Value::String(value)) => value.trim().to_ascii_lowercase(),
            _ => "light".to_string(),
        }
    }

    fn eval_media_dimension_comparison(&self, dimension: &str, operator: &str, rhs: &str) -> bool {
        let rhs = match Self::parse_media_length_px(rhs) {
            Some(value) => value,
            None => return false,
        };
        let (viewport_width, viewport_height) = self.viewport_dimensions();
        let lhs = match dimension.trim() {
            "width" => viewport_width,
            "height" => viewport_height,
            _ => return false,
        };
        match operator {
            "<=" => lhs <= rhs,
            ">=" => lhs >= rhs,
            "<" => lhs < rhs,
            ">" => lhs > rhs,
            "=" => (lhs - rhs).abs() < f64::EPSILON,
            _ => false,
        }
    }

    fn media_feature_matches(&self, feature: &str) -> bool {
        let feature = feature.trim();
        if feature.is_empty() {
            return false;
        }

        if let Some((name, value)) = feature.split_once(':') {
            let name = name.trim();
            let value = value.trim();
            return match name {
                "orientation" => {
                    let (viewport_width, viewport_height) = self.viewport_dimensions();
                    match value {
                        "portrait" => viewport_height > viewport_width,
                        "landscape" => viewport_width >= viewport_height,
                        _ => false,
                    }
                }
                "prefers-color-scheme" => self.preferred_color_scheme() == value,
                "min-width" => self.eval_media_dimension_comparison("width", ">=", value),
                "max-width" => self.eval_media_dimension_comparison("width", "<=", value),
                "width" => self.eval_media_dimension_comparison("width", "=", value),
                "min-height" => self.eval_media_dimension_comparison("height", ">=", value),
                "max-height" => self.eval_media_dimension_comparison("height", "<=", value),
                "height" => self.eval_media_dimension_comparison("height", "=", value),
                _ => false,
            };
        }

        for operator in ["<=", ">=", "<", ">", "="] {
            if let Some((lhs, rhs)) = feature.split_once(operator) {
                return self.eval_media_dimension_comparison(lhs.trim(), operator, rhs.trim());
            }
        }

        false
    }

    fn media_clause_matches(&self, clause: &str) -> bool {
        let clause = clause.trim();
        if clause.is_empty() {
            return true;
        }
        if clause == "all" || clause == "screen" || clause == "only screen" || clause == "only all"
        {
            return true;
        }
        if clause == "print" || clause == "only print" {
            return false;
        }

        let feature = clause
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
            .unwrap_or(clause)
            .trim();
        self.media_feature_matches(feature)
    }

    fn media_query_matches(&self, query: &str) -> bool {
        let query = query.trim().to_ascii_lowercase();
        if query.is_empty() {
            return true;
        }

        for clause in query.split(" and ") {
            if !self.media_clause_matches(clause) {
                return false;
            }
        }
        true
    }

    fn media_condition_matches(&self, media: &str) -> bool {
        let media = media.trim();
        if media.is_empty() {
            return true;
        }
        media
            .split(',')
            .map(str::trim)
            .any(|query| self.media_query_matches(query))
    }

    fn picture_source_type_supported(&self, source: NodeId) -> bool {
        let Some(raw_type) = self.dom.attr(source, "type") else {
            return true;
        };
        let normalized = raw_type.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return true;
        }
        matches!(
            normalized.as_str(),
            "image/apng"
                | "image/avif"
                | "image/bmp"
                | "image/gif"
                | "image/jpeg"
                | "image/jpg"
                | "image/png"
                | "image/svg+xml"
                | "image/webp"
        )
    }

    fn first_srcset_url(srcset: &str) -> Option<String> {
        for candidate in srcset.split(',') {
            let candidate = candidate.trim();
            if candidate.is_empty() {
                continue;
            }
            if let Some(url) = candidate.split_whitespace().next() {
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }
        None
    }

    fn picture_source_candidate_url(&self, source: NodeId) -> Option<String> {
        if let Some(srcset) = self.dom.attr(source, "srcset") {
            if let Some(url) = Self::first_srcset_url(&srcset) {
                return Some(url);
            }
        }
        self.dom
            .attr(source, "src")
            .filter(|value| !value.trim().is_empty())
    }

    fn resolve_picture_source_for_img(&self, img: NodeId) -> Option<String> {
        let parent = self.dom.parent(img)?;
        if !self
            .dom
            .tag_name(parent)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("picture"))
        {
            return None;
        }

        for child in self.dom.child_elements(parent) {
            if child == img {
                break;
            }
            if !self
                .dom
                .tag_name(child)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("source"))
            {
                continue;
            }

            if !self.picture_source_type_supported(child) {
                continue;
            }
            if let Some(media) = self.dom.attr(child, "media") {
                if !self.media_condition_matches(&media) {
                    continue;
                }
            }

            if let Some(url) = self.picture_source_candidate_url(child) {
                return Some(url);
            }
        }

        None
    }

    pub(crate) fn resolve_media_src(&self, node: NodeId) -> String {
        if self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("img"))
        {
            if let Some(source) = self.resolve_picture_source_for_img(node) {
                return self.resolve_document_target_url(&source);
            }
        }

        if let Some(raw) = self.dom.attr(node, "src") {
            return self.resolve_document_target_url(&raw);
        }

        if self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("img"))
        {
            if let Some(srcset) = self.dom.attr(node, "srcset") {
                if let Some(source) = Self::first_srcset_url(&srcset) {
                    return self.resolve_document_target_url(&source);
                }
            }
        }

        let source_attr = self
            .dom
            .child_elements(node)
            .into_iter()
            .find_map(|child| {
                if self
                    .dom
                    .tag_name(child)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("source"))
                {
                    self.dom.attr(child, "src").or_else(|| {
                        self.dom
                            .attr(child, "srcset")
                            .and_then(|v| Self::first_srcset_url(&v))
                    })
                } else {
                    None
                }
            })
            .unwrap_or_default();

        if source_attr.is_empty() {
            String::new()
        } else {
            self.resolve_document_target_url(&source_attr)
        }
    }

    pub(crate) fn anchor_rel_tokens(&self, node: NodeId) -> Vec<String> {
        self.dom
            .attr(node, "rel")
            .unwrap_or_default()
            .split_whitespace()
            .map(|token| token.to_string())
            .collect::<Vec<_>>()
    }

    pub(crate) fn resolve_anchor_href(&self, node: NodeId) -> String {
        let raw = self.dom.attr(node, "href").unwrap_or_default();
        self.resolve_document_target_url(&raw)
    }

    pub(crate) fn anchor_location_parts(&self, node: NodeId) -> LocationParts {
        let href = self.resolve_anchor_href(node);
        LocationParts::parse(&href).unwrap_or_else(|| self.current_location_parts())
    }

    pub(crate) fn set_anchor_url_property(
        &mut self,
        node: NodeId,
        key: &str,
        value: Value,
    ) -> Result<()> {
        match key {
            "href" => {
                self.dom.set_attr(node, "href", &value.as_string())?;
                return Ok(());
            }
            "origin" | "relList" => {
                return Err(Error::ScriptRuntime(format!("anchor.{key} is read-only")));
            }
            _ => {}
        }

        let mut parts = self.anchor_location_parts(node);
        match key {
            "protocol" => {
                let protocol = value.as_string();
                let protocol = protocol.trim_end_matches(':').to_ascii_lowercase();
                if !is_valid_url_scheme(&protocol) {
                    return Err(Error::ScriptRuntime(format!(
                        "invalid anchor.protocol value: {}",
                        value.as_string()
                    )));
                }
                parts.scheme = protocol;
            }
            "host" => {
                let host = value.as_string();
                let (hostname, port) = split_hostname_and_port(host.trim());
                parts.hostname = hostname;
                parts.port = port;
            }
            "hostname" => {
                parts.hostname = value.as_string();
            }
            "port" => {
                parts.port = value.as_string();
            }
            "pathname" => {
                let raw = value.as_string();
                if parts.has_authority {
                    let normalized_input = if raw.starts_with('/') {
                        raw
                    } else {
                        format!("/{raw}")
                    };
                    parts.pathname = normalize_pathname(&normalized_input);
                } else {
                    parts.opaque_path = raw;
                }
            }
            "search" => {
                parts.search = ensure_search_prefix(&value.as_string());
            }
            "hash" => {
                parts.hash = ensure_hash_prefix(&value.as_string());
            }
            "username" => {
                parts.username = value.as_string();
            }
            "password" => {
                parts.password = value.as_string();
            }
            _ => {
                return Err(Error::ScriptRuntime(format!(
                    "unsupported anchor URL property: {key}"
                )));
            }
        }

        self.dom.set_attr(node, "href", &parts.href())
    }

    pub fn enable_trace(&mut self, enabled: bool) {
        self.trace_state.enabled = enabled;
    }

    pub fn take_trace_logs(&mut self) -> Vec<String> {
        self.trace_state.logs.drain(..).collect()
    }
}
