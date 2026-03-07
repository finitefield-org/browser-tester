use super::*;

impl Harness {
    pub(crate) fn is_url_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_URL_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_url_static_method_name(name: &str) -> bool {
        matches!(
            name,
            "canParse" | "parse" | "createObjectURL" | "revokeObjectURL"
        )
    }

    pub(crate) fn normalize_url_parts_for_serialization(parts: &mut LocationParts) {
        parts.scheme = parts.scheme.to_ascii_lowercase();
        parts.normalize_special_port();
        if parts.has_authority {
            parts.hostname = parts.hostname.to_ascii_lowercase();
            parts.username = encode_url_component_preserving_percent(
                &parts.username,
                UrlPercentEncodeSet::UserInfo,
            );
            parts.password = encode_url_component_preserving_percent(
                &parts.password,
                UrlPercentEncodeSet::UserInfo,
            );
            let path = if parts.pathname.is_empty() {
                "/".to_string()
            } else if parts.pathname.starts_with('/') {
                parts.pathname.clone()
            } else {
                format!("/{}", parts.pathname)
            };
            let path = if is_special_url_scheme(&parts.scheme) {
                path.replace('\\', "/")
            } else {
                path
            };
            parts.pathname = normalize_pathname(&path);
            parts.pathname =
                encode_url_component_preserving_percent(&parts.pathname, UrlPercentEncodeSet::Path);
        } else {
            parts.opaque_path = encode_url_component_preserving_percent(
                &parts.opaque_path,
                UrlPercentEncodeSet::OpaquePath,
            );
        }

        if !parts.search.is_empty() {
            let body = parts
                .search
                .strip_prefix('?')
                .unwrap_or(parts.search.as_str());
            let encode_set = if is_special_url_scheme(&parts.scheme) {
                UrlPercentEncodeSet::SpecialQuery
            } else {
                UrlPercentEncodeSet::Query
            };
            parts.search = format!(
                "?{}",
                encode_url_component_preserving_percent(body, encode_set)
            );
        }
        if !parts.hash.is_empty() {
            let body = parts.hash.strip_prefix('#').unwrap_or(parts.hash.as_str());
            parts.hash = format!(
                "#{}",
                encode_url_component_preserving_percent(body, UrlPercentEncodeSet::Fragment)
            );
        }
    }

    pub(crate) fn resolve_url_against_base_parts(input: &str, base: &LocationParts) -> String {
        let input = input.trim();
        if input.is_empty() {
            return base.href();
        }

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

    pub(crate) fn resolve_url_string(input: &str, base: Option<&str>) -> Option<String> {
        let input = input.trim();
        if let Some(mut absolute) = LocationParts::parse(input) {
            Self::normalize_url_parts_for_serialization(&mut absolute);
            return Some(absolute.href());
        }
        if looks_like_absolute_url(input) {
            return None;
        }

        let base = base?;
        let mut base_parts = LocationParts::parse(base)?;
        Self::normalize_url_parts_for_serialization(&mut base_parts);
        let resolved = Self::resolve_url_against_base_parts(input, &base_parts);
        let mut resolved_parts = LocationParts::parse(&resolved)?;
        Self::normalize_url_parts_for_serialization(&mut resolved_parts);
        Some(resolved_parts.href())
    }

    pub(crate) fn sync_url_object_entries_from_parts(
        &self,
        entries: &mut (impl ObjectEntryLookup + ObjectEntryMut),
        parts: &LocationParts,
    ) {
        let href = parts.href();
        Self::object_set_entry(
            entries,
            INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
            Value::String(href.clone()),
        );
        Self::object_set_entry(entries, "href".to_string(), Value::String(href));
        Self::object_set_entry(
            entries,
            "protocol".to_string(),
            Value::String(parts.protocol()),
        );
        Self::object_set_entry(entries, "host".to_string(), Value::String(parts.host()));
        Self::object_set_entry(
            entries,
            "hostname".to_string(),
            Value::String(parts.hostname.clone()),
        );
        Self::object_set_entry(
            entries,
            "port".to_string(),
            Value::String(parts.effective_port()),
        );
        Self::object_set_entry(
            entries,
            "pathname".to_string(),
            Value::String(if parts.has_authority {
                parts.pathname.clone()
            } else {
                parts.opaque_path.clone()
            }),
        );
        Self::object_set_entry(
            entries,
            "search".to_string(),
            Value::String(parts.search.clone()),
        );
        Self::object_set_entry(
            entries,
            "hash".to_string(),
            Value::String(parts.hash.clone()),
        );
        Self::object_set_entry(
            entries,
            "username".to_string(),
            Value::String(parts.username.clone()),
        );
        Self::object_set_entry(
            entries,
            "password".to_string(),
            Value::String(parts.password.clone()),
        );
        Self::object_set_entry(entries, "origin".to_string(), Value::String(parts.origin()));
        Self::object_set_entry(entries, "constructor".to_string(), Value::UrlConstructor);
        Self::object_set_entry(
            entries,
            "toString".to_string(),
            Self::new_receiver_builtin_callable("url", "toString"),
        );
        Self::object_set_entry(
            entries,
            "toJSON".to_string(),
            Self::new_receiver_builtin_callable("url", "toJSON"),
        );

        let owner_id = match Self::object_get_entry(entries, INTERNAL_URL_OBJECT_ID_KEY) {
            Some(Value::Number(id)) if id >= 0 => usize::try_from(id).ok(),
            _ => None,
        };
        let pairs = parse_url_search_params_pairs_from_query_string(&parts.search);
        if let Some(Value::Object(search_params_object)) =
            Self::object_get_entry(entries, "searchParams")
        {
            let mut search_params_entries = search_params_object.borrow_mut();
            Self::set_url_search_params_pairs(&mut search_params_entries, &pairs);
            if let Some(owner_id) = owner_id {
                Self::object_set_entry(
                    &mut search_params_entries,
                    INTERNAL_URL_SEARCH_PARAMS_OWNER_ID_KEY.to_string(),
                    Value::Number(owner_id as i64),
                );
            }
        } else {
            Self::object_set_entry(
                entries,
                "searchParams".to_string(),
                self.new_url_search_params_value(pairs, owner_id),
            );
        }
    }

    pub(crate) fn new_url_value_from_href(&mut self, href: &str) -> Result<Value> {
        let mut parts =
            LocationParts::parse(href).ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
        Self::normalize_url_parts_for_serialization(&mut parts);
        let id = self.browser_apis.allocate_url_object_id();

        let mut entries = vec![
            (INTERNAL_URL_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_URL_OBJECT_ID_KEY.to_string(),
                Value::Number(id as i64),
            ),
        ];
        self.sync_url_object_entries_from_parts(&mut entries, &parts);
        let object = Rc::new(RefCell::new(ObjectValue::new(entries)));
        self.browser_apis.url_objects.insert(id, object.clone());
        Ok(Value::Object(object))
    }

    pub(crate) fn set_url_object_property(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        key: &str,
        value: Value,
    ) -> Result<()> {
        if matches!(key, "origin" | "searchParams") {
            return Err(Error::ScriptRuntime(format!("URL.{key} is read-only")));
        }

        let current_href = {
            let entries = object.borrow();
            Self::object_get_entry(&entries, "href")
                .map(|value| value.as_string())
                .unwrap_or_default()
        };
        let mut parts = LocationParts::parse(&current_href)
            .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
        match key {
            "href" => {
                let href = Self::resolve_url_string(&value.as_string(), None)
                    .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
                parts = LocationParts::parse(&href)
                    .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
            }
            "protocol" => {
                let protocol = value.as_string();
                let protocol = protocol.trim_end_matches(':').to_ascii_lowercase();
                if !is_valid_url_scheme(&protocol) {
                    return Err(Error::ScriptRuntime(format!(
                        "invalid URL.protocol value: {}",
                        value.as_string()
                    )));
                }
                if !parts.apply_protocol_setter(&protocol) {
                    return Ok(());
                }
            }
            "host" => {
                if !parts.apply_host_setter(&value.as_string()) {
                    return Ok(());
                }
            }
            "hostname" => {
                if !parts.apply_hostname_setter(&value.as_string()) {
                    return Ok(());
                }
            }
            "port" => {
                if !parts.apply_port_setter(&value.as_string()) {
                    return Ok(());
                }
            }
            "pathname" => {
                if !parts.has_authority {
                    return Ok(());
                }
                let raw = value.as_string();
                let normalized_input = if raw.starts_with('/') {
                    raw
                } else {
                    format!("/{raw}")
                };
                parts.pathname = normalize_pathname(&normalized_input);
            }
            "search" => {
                parts.search = ensure_search_prefix(&value.as_string());
            }
            "hash" => {
                parts.hash = ensure_hash_prefix(&value.as_string());
            }
            "username" => {
                if !parts.has_authority || parts.scheme.eq_ignore_ascii_case("file") {
                    return Ok(());
                }
                parts.username = value.as_string();
            }
            "password" => {
                if !parts.has_authority || parts.scheme.eq_ignore_ascii_case("file") {
                    return Ok(());
                }
                parts.password = value.as_string();
            }
            _ => {
                Self::object_set_entry(&mut object.borrow_mut(), key.to_string(), value);
                return Ok(());
            }
        }

        Self::normalize_url_parts_for_serialization(&mut parts);
        self.sync_url_object_entries_from_parts(&mut object.borrow_mut(), &parts);
        Ok(())
    }
}
