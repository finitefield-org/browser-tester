use super::*;

impl Harness {
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
        self.sync_document_object();
        self.sync_window_runtime_properties();
        self.location_history
            .location_navigations
            .push(LocationNavigation {
                kind,
                from,
                to: to.clone(),
            });

        if !Self::is_hash_only_navigation(
            &self
                .location_history
                .location_navigations
                .last()
                .map(|nav| nav.from.clone())
                .unwrap_or_default(),
            &to,
        ) {
            let _ = self.load_location_mock_page_if_exists(&to)?;
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
        self.sync_document_object();
        self.sync_window_runtime_properties();
        let _ = self.load_location_mock_page_if_exists(&current)?;
        Ok(())
    }

    pub(crate) fn load_location_mock_page_if_exists(&mut self, url: &str) -> Result<bool> {
        let Some(html) = self.location_history.location_mock_pages.get(url).cloned() else {
            return Ok(false);
        };
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
        self.location_history.history_entries.push(HistoryEntry {
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
            self.location_history.history_entries.push(HistoryEntry {
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
        self.location_history.history_entries[index] = HistoryEntry {
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
        let entry = self
            .location_history
            .history_entries
            .get(target)
            .cloned()
            .unwrap_or(HistoryEntry {
                url: self.document_url.clone(),
                state: Value::Null,
            });
        self.document_url = entry.url.clone();
        self.sync_location_object();
        self.sync_history_object();
        self.sync_document_object();
        self.sync_window_runtime_properties();

        if !Self::is_hash_only_navigation(&from, &entry.url) {
            let _ = self.load_location_mock_page_if_exists(&entry.url)?;
        }

        self.with_script_env_always(|this, env| {
            let _ = this.dispatch_event_with_options(
                this.dom.root,
                "popstate",
                env,
                true,
                false,
                false,
                Some(entry.state),
                None,
                None,
            )?;
            Ok(())
        })?;
        Ok(())
    }
}
