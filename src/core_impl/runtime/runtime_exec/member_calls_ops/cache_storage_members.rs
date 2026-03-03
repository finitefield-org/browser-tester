use super::*;

impl Harness {
    fn cache_storage_resolved_promise(&mut self, value: Value) -> Result<Value> {
        let promise = self.new_pending_promise();
        self.promise_resolve(&promise, value)?;
        Ok(Value::Promise(promise))
    }

    fn cache_storage_rejected_promise(&mut self, reason: Value) -> Value {
        let promise = self.new_pending_promise();
        self.promise_reject(&promise, reason);
        Value::Promise(promise)
    }

    fn cache_storage_security_error_promise(&mut self) -> Value {
        self.cache_storage_rejected_promise(Value::String(
            "SecurityError: CacheStorage is not available in insecure context".to_string(),
        ))
    }

    fn cache_request_url_from_value(&self, value: &Value) -> Result<String> {
        let (_, url) = self.fetch_request_input_and_url_from_value(value)?;
        Ok(url)
    }

    fn cache_response_entry_from_value(
        &self,
        request_url: String,
        response: &Value,
    ) -> Result<CacheEntryRecord> {
        let Value::Object(entries) = response else {
            return Err(Error::ScriptRuntime(
                "Cache.put requires a Response object".into(),
            ));
        };
        let entries = entries.borrow();
        if !Self::is_fetch_response_object(&entries) {
            return Err(Error::ScriptRuntime(
                "Cache.put requires a Response object".into(),
            ));
        }

        let response_body = Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_BODY_KEY)
            .map(|value| value.as_string())
            .unwrap_or_default();
        let response_status = Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_STATUS_KEY)
            .map(|value| Self::value_to_i64(&value))
            .unwrap_or(200);
        let response_status_text =
            Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_STATUS_TEXT_KEY)
                .map(|value| value.as_string())
                .unwrap_or_default();
        let response_url = Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_URL_KEY)
            .map(|value| value.as_string())
            .unwrap_or_else(|| request_url.clone());

        Ok(CacheEntryRecord {
            request_url,
            response_url,
            response_status,
            response_status_text,
            response_body,
        })
    }

    fn cache_response_value_from_entry(&self, entry: &CacheEntryRecord) -> Value {
        self.new_fetch_response_value(
            &entry.response_url,
            entry.response_status,
            &entry.response_status_text,
            &entry.response_body,
        )
    }

    fn cache_storage_option_cache_name(&self, value: Option<&Value>) -> Option<String> {
        let Value::Object(options) = value? else {
            return None;
        };
        let options = options.borrow();
        let cache_name = Self::object_get_entry(&options, "cacheName")?;
        if matches!(cache_name, Value::Undefined | Value::Null) {
            return None;
        }
        Some(cache_name.as_string())
    }

    fn cache_entries_for_name_mut(&mut self, cache_name: &str) -> &mut Vec<CacheEntryRecord> {
        self.browser_apis
            .cache_entries_by_name
            .entry(cache_name.to_string())
            .or_default()
    }

    fn cache_entries_for_name(&self, cache_name: &str) -> Vec<CacheEntryRecord> {
        self.browser_apis
            .cache_entries_by_name
            .get(cache_name)
            .cloned()
            .unwrap_or_default()
    }

    fn cache_add_from_value(
        &mut self,
        cache_name: &str,
        request_value: &Value,
    ) -> Result<Option<Value>> {
        let fetch_result = self.eval_fetch_call_from_values(std::slice::from_ref(request_value))?;
        let Value::Promise(fetch_promise) = fetch_result else {
            return Ok(Some(self.cache_storage_rejected_promise(Value::String(
                "TypeError: Failed to fetch".to_string(),
            ))));
        };

        let settled = {
            let promise_ref = fetch_promise.borrow();
            match &promise_ref.state {
                PromiseState::Pending => None,
                PromiseState::Fulfilled(value) => Some(Ok(value.clone())),
                PromiseState::Rejected(reason) => Some(Err(reason.clone())),
            }
        };

        let Some(settled) = settled else {
            return Ok(Some(self.cache_storage_rejected_promise(Value::String(
                "TypeError: Failed to fetch".to_string(),
            ))));
        };

        let response_value = match settled {
            Ok(value) => value,
            Err(reason) => return Ok(Some(self.cache_storage_rejected_promise(reason))),
        };

        let request_url = self.cache_request_url_from_value(request_value)?;
        let entry = self.cache_response_entry_from_value(request_url, &response_value)?;
        let entries = self.cache_entries_for_name_mut(cache_name);
        if let Some(existing) = entries
            .iter_mut()
            .find(|existing| existing.request_url == entry.request_url)
        {
            *existing = entry;
        } else {
            entries.push(entry);
        }

        Ok(Some(self.cache_storage_resolved_promise(Value::Undefined)?))
    }

    pub(crate) fn eval_cache_storage_member_call(
        &mut self,
        cache_storage_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let is_cache_storage = {
            let entries = cache_storage_object.borrow();
            Self::is_cache_storage_object(&entries)
        };
        if !is_cache_storage {
            return Ok(None);
        }

        if !self.window_is_secure_context() {
            return Ok(Some(self.cache_storage_security_error_promise()));
        }

        match member {
            "open" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "CacheStorage.open requires exactly one argument".into(),
                    ));
                }
                let cache_name = args[0].as_string();
                let cache_object = self.ensure_cache_object(&cache_name);
                Ok(Some(self.cache_storage_resolved_promise(Value::Object(
                    cache_object,
                ))?))
            }
            "has" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "CacheStorage.has requires exactly one argument".into(),
                    ));
                }
                let cache_name = args[0].as_string();
                let has = self
                    .browser_apis
                    .caches_by_name
                    .contains_key(cache_name.as_str());
                Ok(Some(self.cache_storage_resolved_promise(Value::Bool(has))?))
            }
            "delete" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "CacheStorage.delete requires exactly one argument".into(),
                    ));
                }
                let cache_name = args[0].as_string();
                let deleted = self.remove_named_cache(&cache_name);
                Ok(Some(
                    self.cache_storage_resolved_promise(Value::Bool(deleted))?,
                ))
            }
            "keys" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "CacheStorage.keys does not take arguments".into(),
                    ));
                }
                let names = self
                    .cache_storage_names_snapshot()
                    .into_iter()
                    .map(Value::String)
                    .collect::<Vec<_>>();
                Ok(Some(self.cache_storage_resolved_promise(
                    Self::new_array_value(names),
                )?))
            }
            "match" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "CacheStorage.match requires one or two arguments".into(),
                    ));
                }
                let request_url = self.cache_request_url_from_value(&args[0])?;
                let cache_name_filter = self.cache_storage_option_cache_name(args.get(1));

                let names = self.cache_storage_names_snapshot();
                let mut matched = None;
                for cache_name in names {
                    if let Some(filter) = cache_name_filter.as_deref() {
                        if cache_name != filter {
                            continue;
                        }
                    }
                    let entries = self.cache_entries_for_name(&cache_name);
                    if let Some(entry) = entries
                        .into_iter()
                        .find(|entry| entry.request_url == request_url)
                    {
                        matched = Some(entry);
                        break;
                    }
                }

                let result = matched
                    .as_ref()
                    .map(|entry| self.cache_response_value_from_entry(entry))
                    .unwrap_or(Value::Undefined);
                Ok(Some(self.cache_storage_resolved_promise(result)?))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_cache_member_call(
        &mut self,
        cache_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let (is_cache, cache_name) = {
            let entries = cache_object.borrow();
            (
                Self::is_cache_object(&entries),
                Self::cache_name_from_object_entries(&entries),
            )
        };
        if !is_cache {
            return Ok(None);
        }
        let Some(cache_name) = cache_name else {
            return Ok(None);
        };

        if !self.window_is_secure_context() {
            return Ok(Some(self.cache_storage_security_error_promise()));
        }

        match member {
            "match" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Cache.match requires one or two arguments".into(),
                    ));
                }
                let request_url = self.cache_request_url_from_value(&args[0])?;
                let entry = self
                    .cache_entries_for_name(&cache_name)
                    .into_iter()
                    .find(|entry| entry.request_url == request_url);
                let result = entry
                    .as_ref()
                    .map(|entry| self.cache_response_value_from_entry(entry))
                    .unwrap_or(Value::Undefined);
                Ok(Some(self.cache_storage_resolved_promise(result)?))
            }
            "put" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Cache.put requires exactly two arguments".into(),
                    ));
                }
                let request_url = self.cache_request_url_from_value(&args[0])?;
                let entry = self.cache_response_entry_from_value(request_url, &args[1])?;
                let entries = self.cache_entries_for_name_mut(&cache_name);
                if let Some(existing) = entries
                    .iter_mut()
                    .find(|existing| existing.request_url == entry.request_url)
                {
                    *existing = entry;
                } else {
                    entries.push(entry);
                }
                Ok(Some(self.cache_storage_resolved_promise(Value::Undefined)?))
            }
            "delete" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Cache.delete requires one or two arguments".into(),
                    ));
                }
                let request_url = self.cache_request_url_from_value(&args[0])?;
                let entries = self.cache_entries_for_name_mut(&cache_name);
                let before = entries.len();
                entries.retain(|entry| entry.request_url != request_url);
                let deleted = before != entries.len();
                Ok(Some(
                    self.cache_storage_resolved_promise(Value::Bool(deleted))?,
                ))
            }
            "keys" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Cache.keys does not take arguments".into(),
                    ));
                }
                let keys = self
                    .cache_entries_for_name(&cache_name)
                    .into_iter()
                    .map(|entry| {
                        self.new_fetch_request_value(
                            &entry.request_url,
                            &entry.request_url,
                            "GET",
                            &[],
                        )
                    })
                    .collect::<Vec<_>>();
                Ok(Some(self.cache_storage_resolved_promise(
                    Self::new_array_value(keys),
                )?))
            }
            "add" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Cache.add requires exactly one argument".into(),
                    ));
                }
                self.cache_add_from_value(&cache_name, &args[0])
            }
            "addAll" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Cache.addAll requires exactly one argument".into(),
                    ));
                }
                let requests = self.array_like_values_from_value(&args[0])?;
                for request in requests {
                    let Some(result) = self.cache_add_from_value(&cache_name, &request)? else {
                        continue;
                    };
                    let Value::Promise(promise) = result else {
                        continue;
                    };
                    let rejected_reason = {
                        let promise_ref = promise.borrow();
                        match &promise_ref.state {
                            PromiseState::Rejected(reason) => Some(reason.clone()),
                            _ => None,
                        }
                    };
                    if let Some(reason) = rejected_reason {
                        return Ok(Some(self.cache_storage_rejected_promise(reason)));
                    }
                }
                Ok(Some(self.cache_storage_resolved_promise(Value::Undefined)?))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_cache_storage_typed_array_method_dispatch(
        &mut self,
        target_value: &Value,
        method: TypedArrayInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let member = match method {
            TypedArrayInstanceMethod::Keys => "keys",
            _ => return Ok(None),
        };
        let Value::Object(object) = target_value else {
            return Ok(None);
        };
        let mut evaluated_args = Vec::with_capacity(args.len());
        for arg in args {
            evaluated_args.push(self.eval_expr(arg, env, event_param, event)?);
        }
        if let Some(value) = self.eval_cache_storage_member_call(object, member, &evaluated_args)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.eval_cache_member_call(object, member, &evaluated_args)? {
            return Ok(Some(value));
        }
        Ok(None)
    }

    pub(crate) fn eval_cache_storage_map_method_dispatch(
        &mut self,
        target_value: &Value,
        method: MapInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let member = match method {
            MapInstanceMethod::Has => "has",
            MapInstanceMethod::Delete => "delete",
            _ => return Ok(None),
        };
        let Value::Object(object) = target_value else {
            return Ok(None);
        };
        let mut evaluated_args = Vec::with_capacity(args.len());
        for arg in args {
            evaluated_args.push(self.eval_expr(arg, env, event_param, event)?);
        }
        if let Some(value) = self.eval_cache_storage_member_call(object, member, &evaluated_args)? {
            return Ok(Some(value));
        }
        if member == "delete" {
            if let Some(value) = self.eval_cache_member_call(object, member, &evaluated_args)? {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    pub(crate) fn eval_cache_storage_set_method_dispatch(
        &mut self,
        target_value: &Value,
        method: SetInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let member = match method {
            SetInstanceMethod::Add => "add",
            _ => return Ok(None),
        };
        let Value::Object(object) = target_value else {
            return Ok(None);
        };
        let mut evaluated_args = Vec::with_capacity(args.len());
        for arg in args {
            evaluated_args.push(self.eval_expr(arg, env, event_param, event)?);
        }
        if let Some(value) = self.eval_cache_member_call(object, member, &evaluated_args)? {
            return Ok(Some(value));
        }
        Ok(None)
    }
}
