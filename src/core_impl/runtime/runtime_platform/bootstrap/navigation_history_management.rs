use super::*;

impl Harness {
    fn set_document_visibility_state_with_event(&mut self, next_state: &str) -> Result<()> {
        let next_state = match next_state {
            "hidden" => "hidden",
            "visible" => "visible",
            _ => {
                return Err(Error::ScriptRuntime(format!(
                    "unsupported document visibilityState: {next_state}"
                )));
            }
        };
        if self.dom_runtime.document_visibility_state == next_state {
            return Ok(());
        }
        self.dom_runtime.document_visibility_state = next_state.to_string();
        self.with_script_env_always(|this, env| {
            let _ = this.dispatch_event_with_options(
                this.dom.root,
                "visibilitychange",
                env,
                true,
                false,
                false,
                None,
                None,
                None,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub(crate) fn first_base_attr(&self, attr_name: &str) -> Option<String> {
        let base_nodes = self.dom.query_selector_all("base").ok()?;
        base_nodes
            .into_iter()
            .find_map(|node| self.dom.attr(node, attr_name))
    }

    pub(crate) fn document_base_url(&self) -> String {
        let fallback = self.document_url.clone();
        let Some(raw_href) = self.first_base_attr("href") else {
            return fallback;
        };

        let href = raw_href.trim();
        if href.is_empty() {
            return fallback;
        }

        if let Some(parts) = LocationParts::parse(href) {
            if matches!(parts.scheme.as_str(), "data" | "javascript") {
                return fallback;
            }
            return parts.href();
        }

        let resolved = self.resolve_location_target_url(href);
        if let Some(parts) = LocationParts::parse(&resolved) {
            if matches!(parts.scheme.as_str(), "data" | "javascript") {
                return fallback;
            }
            return parts.href();
        }

        fallback
    }

    pub(crate) fn resolve_document_target_url(&self, input: &str) -> String {
        let input = input.trim();
        if let Some(parts) = LocationParts::parse(input) {
            return parts.href();
        }

        let base_url = self.document_base_url();
        if let Some(base_parts) = LocationParts::parse(&base_url) {
            return Self::resolve_url_against_base_parts(input, &base_parts);
        }

        self.resolve_location_target_url(input)
    }

    pub(crate) fn sanitize_base_target_value(target: &str) -> String {
        if target
            .chars()
            .any(|ch| matches!(ch, '\n' | '\r' | '\t' | '<'))
        {
            return "_blank".to_string();
        }
        target.to_string()
    }

    pub(crate) fn default_hyperlink_target(&self) -> String {
        self.first_base_attr("target")
            .map(|value| Self::sanitize_base_target_value(&value))
            .unwrap_or_default()
    }

    pub(crate) fn resolve_location_target_url(&self, input: &str) -> String {
        let input = input.trim();
        if input.is_empty() {
            return self.document_url.clone();
        }

        if let Some(parts) = LocationParts::parse(input) {
            return parts.href();
        }

        let base = self.current_location_parts();
        if input.starts_with("//") {
            return LocationParts::parse(&format!("{}{}", base.protocol(), input))
                .map(|parts| parts.href())
                .unwrap_or_else(|| input.to_string());
        }

        let mut next = base.clone();
        if input.starts_with('#') {
            next.hash = ensure_hash_prefix(input);
            return next.href();
        }

        if input.starts_with('?') {
            next.search = ensure_search_prefix(input);
            next.hash.clear();
            return next.href();
        }

        if input.starts_with('/') {
            if next.has_authority {
                next.pathname = normalize_pathname(input);
            } else {
                next.opaque_path = input.to_string();
            }
            next.search.clear();
            next.hash.clear();
            return next.href();
        }

        let mut relative = input;
        let mut next_search = String::new();
        let mut next_hash = String::new();
        if let Some(hash_pos) = relative.find('#') {
            next_hash = ensure_hash_prefix(&relative[hash_pos + 1..]);
            relative = &relative[..hash_pos];
        }
        if let Some(search_pos) = relative.find('?') {
            next_search = ensure_search_prefix(&relative[search_pos + 1..]);
            relative = &relative[..search_pos];
        }

        if next.has_authority {
            let base_dir = if let Some((prefix, _)) = next.pathname.rsplit_once('/') {
                if prefix.is_empty() {
                    "/".to_string()
                } else {
                    format!("{prefix}/")
                }
            } else {
                "/".to_string()
            };
            next.pathname = normalize_pathname(&format!("{base_dir}{relative}"));
        } else {
            next.opaque_path = relative.to_string();
        }
        next.search = next_search;
        next.hash = next_hash;
        next.href()
    }

    pub(crate) fn is_hash_only_navigation(from: &str, to: &str) -> bool {
        let Some(from_parts) = LocationParts::parse(from) else {
            return false;
        };
        let Some(to_parts) = LocationParts::parse(to) else {
            return false;
        };
        from_parts.scheme == to_parts.scheme
            && from_parts.has_authority == to_parts.has_authority
            && from_parts.username == to_parts.username
            && from_parts.password == to_parts.password
            && from_parts.hostname == to_parts.hostname
            && from_parts.port == to_parts.port
            && from_parts.pathname == to_parts.pathname
            && from_parts.opaque_path == to_parts.opaque_path
            && from_parts.search == to_parts.search
            && from_parts.hash != to_parts.hash
    }

    pub(crate) fn navigation_can_go_back(&self) -> bool {
        self.location_history.history_index > 0 && !self.location_history.history_entries.is_empty()
    }

    pub(crate) fn navigation_can_go_forward(&self) -> bool {
        self.location_history.history_index.saturating_add(1)
            < self.location_history.history_entries.len()
    }

    pub(crate) fn navigation_entry_value(&self, index: usize, entry: &HistoryEntry) -> Value {
        Self::new_object_value(vec![
            ("key".to_string(), Value::String(entry.key.clone())),
            ("url".to_string(), Value::String(entry.url.clone())),
            ("index".to_string(), Value::Number(index as i64)),
            ("sameDocument".to_string(), Value::Bool(true)),
            ("state".to_string(), entry.state.clone()),
        ])
    }

    pub(crate) fn navigation_current_entry_value(&self) -> Value {
        self.location_history
            .history_entries
            .get(self.location_history.history_index)
            .map(|entry| self.navigation_entry_value(self.location_history.history_index, entry))
            .unwrap_or(Value::Null)
    }

    pub(crate) fn navigation_entries_value(&self) -> Value {
        Self::new_array_value(
            self.location_history
                .history_entries
                .iter()
                .enumerate()
                .map(|(index, entry)| self.navigation_entry_value(index, entry))
                .collect(),
        )
    }

    pub(crate) fn navigation_find_entry_index_by_key(&self, key: &str) -> Option<usize> {
        self.location_history
            .history_entries
            .iter()
            .position(|entry| entry.key == key)
    }

    pub(crate) fn next_history_entry_key(&mut self) -> String {
        let key = format!("entry-{}", self.location_history.next_history_entry_key);
        self.location_history.next_history_entry_key = self
            .location_history
            .next_history_entry_key
            .saturating_add(1);
        key
    }

    fn dispatch_hash_change_event_with_urls(&mut self, old_url: &str, new_url: &str) -> Result<()> {
        let old_url = old_url.to_string();
        let new_url = new_url.to_string();
        self.with_script_env_always(|this, env| {
            let target_object = this.dom_runtime.window_object.clone();
            let target_node = this.event_target_listener_node_id(&target_object);
            let target_value = Value::Object(target_object);
            let mut event = EventState::new("hashchange", target_node, this.scheduler.now_ms);
            event.target_value = Some(target_value.clone());
            event.current_target_value = Some(target_value);
            event.bubbles = false;
            event.cancelable = false;
            event.hash_change_interface = true;
            event.hash_change_old_url = old_url.clone();
            event.hash_change_new_url = new_url.clone();
            event.event_phase = 2;
            event.current_target = target_node;
            this.invoke_listeners(target_node, &mut event, env, true)?;
            if !event.propagation_stopped {
                event.event_phase = 2;
                this.invoke_listeners(target_node, &mut event, env, false)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    pub(crate) fn navigate_location(
        &mut self,
        next_url: &str,
        kind: LocationNavigationKind,
    ) -> Result<()> {
        let from = self.document_url.clone();
        let to = self.resolve_location_target_url(next_url);
        self.document_url = to.clone();
        match kind {
            LocationNavigationKind::Replace => {
                self.history_replace_current_entry(&to, Value::Null);
            }
            LocationNavigationKind::Assign | LocationNavigationKind::HrefSet => {
                self.history_push_entry(&to, Value::Null);
            }
            LocationNavigationKind::Reload => {}
        }
        self.sync_location_object();
        self.sync_history_object();
        self.sync_navigation_object();
        self.sync_document_object();
        self.sync_window_runtime_properties();
        self.location_history
            .location_navigations
            .push(LocationNavigation {
                kind,
                from: from.clone(),
                to: to.clone(),
            });

        let hash_only_navigation = Self::is_hash_only_navigation(&from, &to);
        if !hash_only_navigation {
            let _ = self.load_location_mock_page_if_exists(&to)?;
        } else {
            self.dispatch_hash_change_event_with_urls(&from, &to)?;
        }
        Ok(())
    }

    pub(crate) fn reload_location(&mut self) -> Result<()> {
        self.location_history.location_reload_count += 1;
        let current = self.document_url.clone();
        self.location_history
            .location_navigations
            .push(LocationNavigation {
                kind: LocationNavigationKind::Reload,
                from: current.clone(),
                to: current.clone(),
            });
        self.sync_location_object();
        self.sync_history_object();
        self.sync_navigation_object();
        self.sync_document_object();
        self.sync_window_runtime_properties();
        let _ = self.load_location_mock_page_if_exists(&current)?;
        Ok(())
    }

    pub(crate) fn load_location_mock_page_if_exists(&mut self, url: &str) -> Result<bool> {
        let Some(html) = self.location_history.location_mock_pages.get(url).cloned() else {
            return Ok(false);
        };
        self.set_document_visibility_state_with_event("hidden")?;
        self.replace_document_with_html(&html)?;
        Ok(true)
    }

    pub(crate) fn history_push_entry(&mut self, url: &str, state: Value) {
        let next = self
            .location_history
            .history_index
            .saturating_add(1)
            .min(self.location_history.history_entries.len());
        self.location_history.history_entries.truncate(next);
        let key = self.next_history_entry_key();
        self.location_history.history_entries.push(HistoryEntry {
            key,
            url: url.to_string(),
            state,
        });
        self.location_history.history_index = self
            .location_history
            .history_entries
            .len()
            .saturating_sub(1);
    }

    pub(crate) fn history_replace_current_entry(&mut self, url: &str, state: Value) {
        if self.location_history.history_entries.is_empty() {
            let key = self.next_history_entry_key();
            self.location_history.history_entries.push(HistoryEntry {
                key,
                url: url.to_string(),
                state,
            });
            self.location_history.history_index = 0;
            return;
        }
        let index = self.location_history.history_index.min(
            self.location_history
                .history_entries
                .len()
                .saturating_sub(1),
        );
        let key = self.location_history.history_entries[index].key.clone();
        self.location_history.history_entries[index] = HistoryEntry {
            key,
            url: url.to_string(),
            state,
        };
        self.location_history.history_index = index;
    }

    pub(crate) fn history_push_state(
        &mut self,
        state: Value,
        url: Option<&str>,
        replace: bool,
    ) -> Result<()> {
        let cloned = Self::structured_clone_value(&state, &mut Vec::new(), &mut Vec::new())?;
        let target_url = url.unwrap_or(&self.document_url);
        let next_url = self.resolve_location_target_url(target_url);
        self.document_url = next_url.clone();

        if replace {
            self.history_replace_current_entry(&next_url, cloned);
        } else {
            self.history_push_entry(&next_url, cloned);
        }
        self.sync_location_object();
        self.sync_history_object();
        self.sync_navigation_object();
        self.sync_document_object();
        self.sync_window_runtime_properties();
        Ok(())
    }

    pub(crate) fn history_go_with_env(&mut self, delta: i64) -> Result<()> {
        if delta == 0 {
            self.reload_location()?;
            return Ok(());
        }

        let current = self.location_history.history_index as i64;
        let target = current.saturating_add(delta);
        if target < 0 || target >= self.location_history.history_entries.len() as i64 {
            return Ok(());
        }
        let target = target as usize;
        if target == self.location_history.history_index {
            return Ok(());
        }

        let from = self.document_url.clone();
        self.location_history.history_index = target;
        let entry = if let Some(entry) = self.location_history.history_entries.get(target).cloned()
        {
            entry
        } else {
            HistoryEntry {
                key: self.next_history_entry_key(),
                url: self.document_url.clone(),
                state: Value::Null,
            }
        };
        self.document_url = entry.url.clone();
        self.sync_location_object();
        self.sync_history_object();
        self.sync_navigation_object();
        self.sync_document_object();
        self.sync_window_runtime_properties();

        let hash_only_navigation = Self::is_hash_only_navigation(&from, &entry.url);
        if !hash_only_navigation {
            let _ = self.load_location_mock_page_if_exists(&entry.url)?;
        }

        self.with_script_env_always(|this, env| {
            let target_object = this.dom_runtime.window_object.clone();
            let target_node = this.event_target_listener_node_id(&target_object);
            let target_value = Value::Object(target_object);
            let mut event = EventState::new("popstate", target_node, this.scheduler.now_ms);
            event.target_value = Some(target_value.clone());
            event.current_target_value = Some(target_value);
            event.bubbles = false;
            event.cancelable = false;
            event.state = Some(entry.state.clone());
            event.event_phase = 2;
            event.current_target = target_node;
            this.invoke_listeners(target_node, &mut event, env, true)?;
            if !event.propagation_stopped {
                event.event_phase = 2;
                this.invoke_listeners(target_node, &mut event, env, false)?;
            }
            Ok(())
        })?;
        if hash_only_navigation {
            self.dispatch_hash_change_event_with_urls(&from, &entry.url)?;
        }
        Ok(())
    }
}
