use super::*;

impl Harness {
    pub(crate) fn resolve_media_src(&self, node: NodeId) -> String {
        if let Some(raw) = self.dom.attr(node, "src") {
            return self.resolve_document_target_url(&raw);
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
                    self.dom.attr(child, "src")
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
