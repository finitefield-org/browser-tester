use super::*;

impl Harness {
    pub(crate) fn cookie_store_builtin_keys() -> &'static [&'static str] {
        &[
            "set",
            "get",
            "getAll",
            "delete",
            "addEventListener",
            "removeEventListener",
        ]
    }

    pub(crate) fn sync_cookie_store_object(&mut self) {
        let mut extras = Vec::new();
        {
            let entries = self.browser_apis.cookie_store_object.borrow();
            for (key, value) in entries.iter() {
                if Self::is_internal_object_key(key) {
                    continue;
                }
                if Self::cookie_store_builtin_keys()
                    .iter()
                    .any(|builtin| builtin == key)
                {
                    continue;
                }
                extras.push((key.clone(), value.clone()));
            }
        }

        let mut entries = vec![
            (
                INTERNAL_COOKIE_STORE_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("set".to_string(), Self::new_builtin_placeholder_function()),
            ("get".to_string(), Self::new_builtin_placeholder_function()),
            (
                "getAll".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "delete".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "addEventListener".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "removeEventListener".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        entries.extend(extras);
        *self.browser_apis.cookie_store_object.borrow_mut() = entries.into();
    }

    pub(crate) fn cookie_store_global_value(&self) -> Value {
        if self.window_is_secure_context() {
            Value::Object(self.browser_apis.cookie_store_object.clone())
        } else {
            Value::Undefined
        }
    }

    pub(crate) fn is_cookie_store_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_COOKIE_STORE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn normalize_cookie_path(path: &str) -> String {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return "/".to_string();
        }
        if trimmed.starts_with('/') {
            trimmed.to_string()
        } else {
            format!("/{trimmed}")
        }
    }

    fn normalize_cookie_domain(domain: &str) -> Option<String> {
        let trimmed = domain.trim().trim_start_matches('.').to_ascii_lowercase();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    fn current_cookie_request_path(&self) -> String {
        let parts = self.current_location_parts();
        if !parts.has_authority {
            return "/".to_string();
        }
        let pathname = if parts.pathname.is_empty() {
            "/"
        } else {
            parts.pathname.as_str()
        };
        Self::normalize_cookie_path(pathname)
    }

    fn cookie_path_matches_request_path(cookie_path: &str, request_path: &str) -> bool {
        if request_path == cookie_path {
            return true;
        }
        if !request_path.starts_with(cookie_path) {
            return false;
        }
        cookie_path.ends_with('/')
            || request_path
                .as_bytes()
                .get(cookie_path.len())
                .is_some_and(|ch| *ch == b'/')
    }

    fn cookie_domain_matches_current_host(&self, record: &CookieRecord) -> bool {
        let host = self.current_location_parts().hostname.to_ascii_lowercase();
        match &record.domain {
            Some(domain) => host == *domain || host.ends_with(&format!(".{domain}")),
            None => true,
        }
    }

    fn cookie_is_expired(record: &CookieRecord, now_ms: i64) -> bool {
        record.expires_ms.is_some_and(|expires| expires <= now_ms)
    }

    fn cookie_visible_to_document(&self, record: &CookieRecord) -> bool {
        if Self::cookie_is_expired(record, self.scheduler.now_ms) {
            return false;
        }
        if record.secure && !self.window_is_secure_context() {
            return false;
        }
        if !self.cookie_domain_matches_current_host(record) {
            return false;
        }
        let request_path = self.current_cookie_request_path();
        Self::cookie_path_matches_request_path(&record.path, &request_path)
    }

    pub(crate) fn prune_expired_cookies(&mut self) {
        let now_ms = self.scheduler.now_ms;
        self.browser_apis
            .cookies
            .retain(|record| !Self::cookie_is_expired(record, now_ms));
    }

    pub(crate) fn document_cookie_string(&mut self) -> String {
        self.prune_expired_cookies();
        self.browser_apis
            .cookies
            .iter()
            .filter(|record| self.cookie_visible_to_document(record))
            .map(|record| format!("{}={}", record.name, record.value))
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub(crate) fn sync_document_cookie_property(&mut self) {
        let cookie = self.document_cookie_string();
        Self::object_set_entry(
            &mut self.dom_runtime.document_object.borrow_mut(),
            "cookie".to_string(),
            Value::String(cookie),
        );
    }

    pub(crate) fn upsert_cookie_record(&mut self, mut record: CookieRecord) -> bool {
        self.prune_expired_cookies();
        let name = record.name.trim().to_string();
        if name.is_empty() {
            return false;
        }
        record.name = name;
        record.path = Self::normalize_cookie_path(&record.path);
        record.domain = record
            .domain
            .as_deref()
            .and_then(Self::normalize_cookie_domain);

        let key_matches = |candidate: &CookieRecord| {
            candidate.name == record.name
                && candidate.path == record.path
                && candidate.domain == record.domain
                && candidate.partitioned == record.partitioned
        };

        if record
            .expires_ms
            .is_some_and(|expires| expires <= self.scheduler.now_ms)
        {
            let before = self.browser_apis.cookies.len();
            self.browser_apis
                .cookies
                .retain(|candidate| !key_matches(candidate));
            return before != self.browser_apis.cookies.len();
        }

        if let Some(existing) = self
            .browser_apis
            .cookies
            .iter_mut()
            .find(|candidate| key_matches(candidate))
        {
            let changed = *existing != record;
            *existing = record;
            return changed;
        }

        self.browser_apis.cookies.push(record);
        true
    }

    pub(crate) fn cookie_matches_filter(
        &self,
        record: &CookieRecord,
        name: Option<&str>,
        path: Option<&str>,
        domain: Option<&str>,
        partitioned: Option<bool>,
    ) -> bool {
        if let Some(name) = name {
            if record.name != name {
                return false;
            }
        }
        if let Some(path) = path {
            if record.path != Self::normalize_cookie_path(path) {
                return false;
            }
        }
        if let Some(domain) = domain {
            let normalized = Self::normalize_cookie_domain(domain);
            if record.domain != normalized {
                return false;
            }
        }
        if let Some(partitioned) = partitioned {
            if record.partitioned != partitioned {
                return false;
            }
        }
        true
    }

    pub(crate) fn visible_cookie_records(
        &mut self,
        name: Option<&str>,
        path: Option<&str>,
        domain: Option<&str>,
        partitioned: Option<bool>,
    ) -> Vec<CookieRecord> {
        self.prune_expired_cookies();
        self.browser_apis
            .cookies
            .iter()
            .filter(|record| self.cookie_visible_to_document(record))
            .filter(|record| self.cookie_matches_filter(record, name, path, domain, partitioned))
            .cloned()
            .collect()
    }

    pub(crate) fn delete_visible_cookie_records(
        &mut self,
        name: Option<&str>,
        path: Option<&str>,
        domain: Option<&str>,
        partitioned: Option<bool>,
    ) -> Vec<CookieRecord> {
        self.prune_expired_cookies();
        let mut deleted = Vec::new();
        let mut kept = Vec::new();
        let existing = std::mem::take(&mut self.browser_apis.cookies);
        for record in existing {
            let should_delete = self.cookie_visible_to_document(&record)
                && self.cookie_matches_filter(&record, name, path, domain, partitioned);
            if should_delete {
                deleted.push(record);
            } else {
                kept.push(record);
            }
        }
        self.browser_apis.cookies = kept;
        deleted
    }

    pub(crate) fn set_cookie_from_document_assignment(&mut self, raw: &str) -> bool {
        let mut segments = raw.split(';');
        let Some(first) = segments.next() else {
            return false;
        };
        let Some((name_raw, value_raw)) = first.trim().split_once('=') else {
            return false;
        };
        let name = name_raw.trim().to_string();
        if name.is_empty() {
            return false;
        }

        let mut record = CookieRecord {
            name,
            value: value_raw.trim().to_string(),
            domain: None,
            path: "/".to_string(),
            expires_ms: None,
            secure: false,
            same_site: None,
            partitioned: false,
        };

        for segment in segments {
            let attr = segment.trim();
            if attr.is_empty() {
                continue;
            }

            if let Some((key, value)) = attr.split_once('=') {
                let key = key.trim().to_ascii_lowercase();
                let value = value.trim();
                match key.as_str() {
                    "path" => record.path = Self::normalize_cookie_path(value),
                    "domain" => record.domain = Self::normalize_cookie_domain(value),
                    "expires" => {
                        record.expires_ms = Self::parse_date_string_to_epoch_ms(value);
                    }
                    "max-age" => {
                        if let Ok(seconds) = value.parse::<i64>() {
                            record.expires_ms =
                                Some(self.scheduler.now_ms.saturating_add(seconds * 1_000));
                        }
                    }
                    "samesite" => {
                        if !value.is_empty() {
                            record.same_site = Some(value.to_string());
                        }
                    }
                    _ => {}
                }
            } else {
                match attr.to_ascii_lowercase().as_str() {
                    "secure" => record.secure = true,
                    "partitioned" => record.partitioned = true,
                    _ => {}
                }
            }
        }

        self.upsert_cookie_record(record)
    }

    pub(crate) fn cookie_record_to_value(record: &CookieRecord) -> Value {
        Self::new_object_value(vec![
            ("name".to_string(), Value::String(record.name.clone())),
            ("value".to_string(), Value::String(record.value.clone())),
            (
                "domain".to_string(),
                record
                    .domain
                    .clone()
                    .map(Value::String)
                    .unwrap_or(Value::Null),
            ),
            ("path".to_string(), Value::String(record.path.clone())),
            (
                "expires".to_string(),
                record.expires_ms.map(Value::Number).unwrap_or(Value::Null),
            ),
            ("secure".to_string(), Value::Bool(record.secure)),
            (
                "sameSite".to_string(),
                record
                    .same_site
                    .clone()
                    .map(Value::String)
                    .unwrap_or(Value::Null),
            ),
            ("partitioned".to_string(), Value::Bool(record.partitioned)),
        ])
    }

    pub(crate) fn add_cookie_store_change_listener(&mut self, callback: Value) -> Result<()> {
        if matches!(callback, Value::Null | Value::Undefined) {
            return Ok(());
        }
        if !self.is_callable_value(&callback) {
            return Err(Error::ScriptRuntime(
                "CookieStore change listener must be a function".into(),
            ));
        }
        let already = self
            .browser_apis
            .cookie_store_change_listeners
            .iter()
            .any(|existing| self.strict_equal(existing, &callback));
        if !already {
            self.browser_apis
                .cookie_store_change_listeners
                .push(callback);
        }
        Ok(())
    }

    pub(crate) fn remove_cookie_store_change_listener(&mut self, callback: &Value) {
        let listeners = std::mem::take(&mut self.browser_apis.cookie_store_change_listeners);
        let mut kept = Vec::new();
        for existing in listeners {
            if !self.strict_equal(&existing, callback) {
                kept.push(existing);
            }
        }
        self.browser_apis.cookie_store_change_listeners = kept;
    }

    pub(crate) fn dispatch_cookie_store_change_event(
        &mut self,
        changed: &[CookieRecord],
        deleted: &[CookieRecord],
    ) -> Result<()> {
        if changed.is_empty() && deleted.is_empty() {
            return Ok(());
        }

        let listeners = self.browser_apis.cookie_store_change_listeners.clone();
        if listeners.is_empty() {
            return Ok(());
        }

        let event_value = Self::new_object_value(vec![
            ("type".to_string(), Value::String("change".to_string())),
            (
                "changed".to_string(),
                Self::new_array_value(changed.iter().map(Self::cookie_record_to_value).collect()),
            ),
            (
                "deleted".to_string(),
                Self::new_array_value(deleted.iter().map(Self::cookie_record_to_value).collect()),
            ),
        ]);

        let event = EventState::new("change", self.dom.root, self.scheduler.now_ms);
        for listener in listeners {
            if !self.is_callable_value(&listener) {
                continue;
            }
            let _ =
                self.execute_callable_value(&listener, std::slice::from_ref(&event_value), &event)?;
        }
        Ok(())
    }
}
