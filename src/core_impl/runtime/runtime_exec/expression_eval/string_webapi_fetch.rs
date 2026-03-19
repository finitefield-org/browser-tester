use super::*;

impl Harness {
    pub(crate) fn is_fetch_response_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_FETCH_RESPONSE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_fetch_request_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_FETCH_REQUEST_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_headers_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_HEADERS_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    fn fetch_type_error_value(reason: &str) -> Value {
        Value::String(format!("TypeError: {reason}"))
    }

    fn fetch_rejected_promise(&mut self, reason: &str) -> Result<Value> {
        let promise = self.new_pending_promise();
        self.promise_reject(&promise, Self::fetch_type_error_value(reason));
        Ok(Value::Promise(promise))
    }

    fn resolve_fetch_url(&self, input: &str) -> Result<String> {
        let base = self.document_base_url();
        let resolved = Self::resolve_url_string(input, Some(&base))
            .ok_or_else(|| Error::ScriptRuntime("TypeError: Invalid URL".into()))?;
        let parts = LocationParts::parse(&resolved)
            .ok_or_else(|| Error::ScriptRuntime("TypeError: Invalid URL".into()))?;
        if !parts.username.is_empty() || !parts.password.is_empty() {
            return Err(Error::ScriptRuntime(
                "TypeError: URL with credentials is not allowed".into(),
            ));
        }
        Ok(resolved)
    }

    pub(crate) fn fetch_request_input_and_url_from_value(
        &self,
        value: &Value,
    ) -> Result<(String, String)> {
        match value {
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_fetch_request_object(&entries) {
                    let input =
                        match Self::object_get_entry(&entries, INTERNAL_FETCH_REQUEST_INPUT_KEY) {
                            Some(Value::String(input)) => input,
                            _ => String::new(),
                        };
                    let url = match Self::object_get_entry(&entries, INTERNAL_FETCH_REQUEST_URL_KEY)
                    {
                        Some(Value::String(url)) => url,
                        _ => self.resolve_fetch_url(&input)?,
                    };
                    return Ok((input, url));
                }
                if Self::is_url_object(&entries) {
                    let href = Self::object_get_entry(&entries, "href")
                        .map(|value| value.as_string())
                        .unwrap_or_default();
                    let url = self.resolve_fetch_url(&href)?;
                    return Ok((href, url));
                }
                let input = value.as_string();
                let url = self.resolve_fetch_url(&input)?;
                Ok((input, url))
            }
            _ => {
                let input = value.as_string();
                let url = self.resolve_fetch_url(&input)?;
                Ok((input, url))
            }
        }
    }

    fn normalize_header_name(name: &str) -> String {
        name.trim().to_ascii_lowercase()
    }

    fn headers_pairs_from_value(&self, value: &Value) -> Result<Vec<(String, String)>> {
        match value {
            Value::Undefined | Value::Null => Ok(Vec::new()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_headers_object(&entries) {
                    let Some(Value::Object(header_entries)) =
                        Self::object_get_entry(&entries, INTERNAL_HEADERS_ENTRIES_KEY)
                    else {
                        return Ok(Vec::new());
                    };
                    let header_entries = header_entries.borrow();
                    let mut pairs = header_entries
                        .iter()
                        .filter(|(name, _)| !Self::is_internal_object_key(name))
                        .map(|(name, value)| (name.clone(), value.as_string()))
                        .collect::<Vec<_>>();
                    pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                    return Ok(pairs);
                }
                let mut pairs = entries
                    .iter()
                    .filter(|(name, _)| !Self::is_internal_object_key(name))
                    .map(|(name, value)| (Self::normalize_header_name(name), value.as_string()))
                    .filter(|(name, _)| !name.is_empty())
                    .collect::<Vec<_>>();
                pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                Ok(pairs)
            }
            _ => Err(Error::ScriptRuntime(
                "TypeError: RequestInit.headers must be an object or Headers".into(),
            )),
        }
    }

    fn fetch_options_from_value(&self, value: &Value) -> Result<(String, Vec<(String, String)>)> {
        match value {
            Value::Undefined | Value::Null => Ok(("GET".to_string(), Vec::new())),
            Value::Object(entries) => {
                let entries = entries.borrow();
                let method = Self::object_get_entry(&entries, "method")
                    .filter(|value| !matches!(value, Value::Undefined))
                    .map(|value| value.as_string().to_ascii_uppercase())
                    .filter(|method| !method.trim().is_empty())
                    .unwrap_or_else(|| "GET".to_string());
                let headers = Self::object_get_entry(&entries, "headers")
                    .map(|headers| self.headers_pairs_from_value(&headers))
                    .transpose()?
                    .unwrap_or_default();
                Ok((method, headers))
            }
            _ => Err(Error::ScriptRuntime(
                "TypeError: RequestInit must be an object".into(),
            )),
        }
    }

    pub(crate) fn new_headers_value_from_pairs(&self, pairs: &[(String, String)]) -> Value {
        let mut entries = ObjectValue::default();
        for (name, value) in pairs {
            if name.is_empty() {
                continue;
            }
            Self::object_set_entry(
                &mut entries,
                Self::normalize_header_name(name),
                Value::String(value.clone()),
            );
        }
        Self::new_object_value(vec![
            (INTERNAL_HEADERS_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_HEADERS_ENTRIES_KEY.to_string(),
                Value::Object(Rc::new(RefCell::new(entries))),
            ),
            ("set".to_string(), Self::new_builtin_placeholder_function()),
            ("get".to_string(), Self::new_builtin_placeholder_function()),
            ("has".to_string(), Self::new_builtin_placeholder_function()),
            (
                "append".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "delete".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ])
    }

    pub(crate) fn new_headers_value_from_call_args(&self, args: &[Value]) -> Result<Value> {
        if args.len() > 1 {
            return Err(Error::ScriptRuntime(
                "Headers constructor supports at most one argument".into(),
            ));
        }
        let pairs = args
            .first()
            .map(|value| self.headers_pairs_from_value(value))
            .transpose()?
            .unwrap_or_default();
        Ok(self.new_headers_value_from_pairs(&pairs))
    }

    pub(crate) fn new_fetch_request_value(
        &self,
        input: &str,
        url: &str,
        method: &str,
        headers: &[(String, String)],
    ) -> Value {
        let headers_value = self.new_headers_value_from_pairs(headers);
        Self::new_object_value(vec![
            (
                INTERNAL_FETCH_REQUEST_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_FETCH_REQUEST_INPUT_KEY.to_string(),
                Value::String(input.to_string()),
            ),
            (
                INTERNAL_FETCH_REQUEST_URL_KEY.to_string(),
                Value::String(url.to_string()),
            ),
            (
                INTERNAL_FETCH_REQUEST_METHOD_KEY.to_string(),
                Value::String(method.to_string()),
            ),
            ("url".to_string(), Value::String(url.to_string())),
            ("method".to_string(), Value::String(method.to_string())),
            ("headers".to_string(), headers_value),
            (
                "clone".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ])
    }

    pub(crate) fn new_fetch_request_value_from_call_args(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Err(Error::ScriptRuntime(
                "Request constructor requires at least one argument".into(),
            ));
        }
        if args.len() > 2 {
            return Err(Error::ScriptRuntime(
                "Request constructor supports one or two arguments".into(),
            ));
        }
        let (input, url) = self.fetch_request_input_and_url_from_value(&args[0])?;
        let (method, headers) = args
            .get(1)
            .map(|options| self.fetch_options_from_value(options))
            .transpose()?
            .unwrap_or_else(|| ("GET".to_string(), Vec::new()));
        Ok(self.new_fetch_request_value(&input, &url, &method, &headers))
    }

    pub(crate) fn new_fetch_response_value(
        &self,
        url: &str,
        status: i64,
        status_text: &str,
        body: &str,
    ) -> Value {
        let headers = self.new_headers_value_from_pairs(&[]);
        Self::new_object_value(vec![
            (
                INTERNAL_FETCH_RESPONSE_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_FETCH_RESPONSE_BODY_KEY.to_string(),
                Value::String(body.to_string()),
            ),
            (
                INTERNAL_FETCH_RESPONSE_STATUS_KEY.to_string(),
                Value::Number(status),
            ),
            (
                INTERNAL_FETCH_RESPONSE_STATUS_TEXT_KEY.to_string(),
                Value::String(status_text.to_string()),
            ),
            (
                INTERNAL_FETCH_RESPONSE_URL_KEY.to_string(),
                Value::String(url.to_string()),
            ),
            ("ok".to_string(), Value::Bool((200..=299).contains(&status))),
            ("status".to_string(), Value::Number(status)),
            (
                "statusText".to_string(),
                Value::String(status_text.to_string()),
            ),
            ("url".to_string(), Value::String(url.to_string())),
            ("headers".to_string(), headers),
            ("text".to_string(), Self::new_builtin_placeholder_function()),
            ("json".to_string(), Self::new_builtin_placeholder_function()),
            ("blob".to_string(), Self::new_builtin_placeholder_function()),
            (
                "arrayBuffer".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "clone".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ])
    }

    pub(crate) fn fetch_response_property_from_entries(
        &self,
        entries: &[(String, Value)],
        key: &str,
    ) -> Option<Value> {
        if !Self::is_fetch_response_object(entries) {
            return None;
        }
        match key {
            "ok" | "status" | "statusText" | "url" | "headers" | "text" | "json" | "blob"
            | "arrayBuffer" | "clone" => Self::object_get_entry(entries, key),
            _ => None,
        }
    }

    pub(crate) fn fetch_request_property_from_entries(
        &self,
        entries: &[(String, Value)],
        key: &str,
    ) -> Option<Value> {
        if !Self::is_fetch_request_object(entries) {
            return None;
        }
        match key {
            "url" | "method" | "headers" | "clone" => Self::object_get_entry(entries, key),
            _ => None,
        }
    }

    pub(crate) fn headers_property_from_entries(
        &self,
        entries: &[(String, Value)],
        key: &str,
    ) -> Option<Value> {
        if !Self::is_headers_object(entries) {
            return None;
        }
        match key {
            "set" | "get" | "has" | "append" | "delete" => Self::object_get_entry(entries, key),
            _ => None,
        }
    }

    pub(crate) fn eval_fetch_call(
        &mut self,
        request: &Expr,
        options: Option<&Expr>,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let request_value = self.eval_expr(request, env, event_param, event)?;
        let options_value = options
            .map(|expr| self.eval_expr(expr, env, event_param, event))
            .transpose()?;
        self.eval_fetch_call_impl(&request_value, options_value.as_ref())
    }

    fn eval_fetch_call_impl(
        &mut self,
        request_value: &Value,
        options_value: Option<&Value>,
    ) -> Result<Value> {
        let (input_key, request_url) =
            match self.fetch_request_input_and_url_from_value(request_value) {
                Ok(ok) => ok,
                Err(Error::ScriptRuntime(message)) => {
                    let reason = message.strip_prefix("TypeError: ").unwrap_or("Invalid URL");
                    return self.fetch_rejected_promise(reason);
                }
                Err(_) => return self.fetch_rejected_promise("Invalid URL"),
            };

        if let Some(options) = options_value {
            if self.fetch_options_from_value(options).is_err() {
                return self.fetch_rejected_promise("RequestInit is invalid");
            }
        }

        self.platform_mocks.fetch_calls.push(input_key.clone());
        let mock = self
            .platform_mocks
            .fetch_mocks
            .get(&input_key)
            .cloned()
            .or_else(|| self.platform_mocks.fetch_mocks.get(&request_url).cloned());
        let Some(mock) = mock else {
            return self.fetch_rejected_promise("Failed to fetch");
        };

        let response =
            self.new_fetch_response_value(&request_url, mock.status, &mock.status_text, &mock.body);
        let promise = self.new_pending_promise();
        self.promise_resolve(&promise, response)?;
        Ok(Value::Promise(promise))
    }

    pub(crate) fn eval_fetch_call_from_values(&mut self, args: &[Value]) -> Result<Value> {
        if args.is_empty() || args.len() > 2 {
            return Err(Error::ScriptRuntime(
                "fetch requires one or two arguments".into(),
            ));
        }
        self.eval_fetch_call_impl(&args[0], args.get(1))
    }

    pub(crate) fn eval_fetch_response_member_call(
        &mut self,
        response_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let (is_response, body, status, status_text, url) = {
            let entries = response_object.borrow();
            (
                Self::is_fetch_response_object(&entries),
                Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_BODY_KEY),
                Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_STATUS_KEY),
                Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_STATUS_TEXT_KEY),
                Self::object_get_entry(&entries, INTERNAL_FETCH_RESPONSE_URL_KEY),
            )
        };
        if !is_response {
            return Ok(None);
        }
        let body = body.map(|value| value.as_string()).unwrap_or_default();
        let status = status
            .map(|value| Self::value_to_i64(&value))
            .unwrap_or(200);
        let status_text = status_text
            .map(|value| value.as_string())
            .unwrap_or_default();
        let url = url.map(|value| value.as_string()).unwrap_or_default();

        match member {
            "text" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Response.text does not take arguments".into(),
                    ));
                }
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, Value::String(body))?;
                Ok(Some(Value::Promise(promise)))
            }
            "json" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Response.json does not take arguments".into(),
                    ));
                }
                let promise = self.new_pending_promise();
                match Self::parse_json_text(&body) {
                    Ok(value) => {
                        self.promise_resolve(&promise, value)?;
                    }
                    Err(_) => {
                        self.promise_reject(
                            &promise,
                            Value::String(
                                "SyntaxError: Response body is not valid JSON".to_string(),
                            ),
                        );
                    }
                }
                Ok(Some(Value::Promise(promise)))
            }
            "blob" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Response.blob does not take arguments".into(),
                    ));
                }
                let promise = self.new_pending_promise();
                self.promise_resolve(
                    &promise,
                    Self::new_blob_value(body.into_bytes(), String::new()),
                )?;
                Ok(Some(Value::Promise(promise)))
            }
            "arrayBuffer" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Response.arrayBuffer does not take arguments".into(),
                    ));
                }
                let promise = self.new_pending_promise();
                self.promise_resolve(
                    &promise,
                    Value::ArrayBuffer(Rc::new(RefCell::new(ArrayBufferValue {
                        bytes: body.into_bytes(),
                        max_byte_length: None,
                        detached: false,
                    }))),
                )?;
                Ok(Some(Value::Promise(promise)))
            }
            "clone" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Response.clone does not take arguments".into(),
                    ));
                }
                Ok(Some(self.new_fetch_response_value(
                    &url,
                    status,
                    &status_text,
                    &body,
                )))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_fetch_request_member_call(
        &mut self,
        request_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let is_request = {
            let entries = request_object.borrow();
            Self::is_fetch_request_object(&entries)
        };
        if !is_request {
            return Ok(None);
        }
        match member {
            "clone" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Request.clone does not take arguments".into(),
                    ));
                }
                let cloned = Value::Object(Rc::new(RefCell::new(request_object.borrow().clone())));
                Ok(Some(cloned))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_headers_member_call(
        &mut self,
        headers_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let header_entries = {
            let entries = headers_object.borrow();
            if !Self::is_headers_object(&entries) {
                return Ok(None);
            }
            match Self::object_get_entry(&entries, INTERNAL_HEADERS_ENTRIES_KEY) {
                Some(Value::Object(header_entries)) => header_entries,
                _ => {
                    return Err(Error::ScriptRuntime(
                        "headers object has invalid internal state".into(),
                    ));
                }
            }
        };

        match member {
            "set" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Headers.set requires exactly two arguments".into(),
                    ));
                }
                let name = Self::normalize_header_name(&args[0].as_string());
                let value = args[1].as_string();
                if !name.is_empty() {
                    Self::object_set_entry(
                        &mut header_entries.borrow_mut(),
                        name,
                        Value::String(value),
                    );
                }
                Ok(Some(Value::Undefined))
            }
            "append" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Headers.append requires exactly two arguments".into(),
                    ));
                }
                let name = Self::normalize_header_name(&args[0].as_string());
                let value = args[1].as_string();
                if !name.is_empty() {
                    let mut entries = header_entries.borrow_mut();
                    let existing = Self::object_get_entry(&entries, &name)
                        .map(|value| value.as_string())
                        .unwrap_or_default();
                    let next = if existing.is_empty() {
                        value
                    } else {
                        format!("{existing}, {value}")
                    };
                    Self::object_set_entry(&mut entries, name, Value::String(next));
                }
                Ok(Some(Value::Undefined))
            }
            "get" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Headers.get requires exactly one argument".into(),
                    ));
                }
                let name = Self::normalize_header_name(&args[0].as_string());
                let value = if name.is_empty() {
                    Value::Null
                } else {
                    Self::object_get_entry(&header_entries.borrow(), &name).unwrap_or(Value::Null)
                };
                Ok(Some(value))
            }
            "has" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Headers.has requires exactly one argument".into(),
                    ));
                }
                let name = Self::normalize_header_name(&args[0].as_string());
                let has = !name.is_empty()
                    && Self::object_get_entry(&header_entries.borrow(), &name).is_some();
                Ok(Some(Value::Bool(has)))
            }
            "delete" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Headers.delete requires exactly one argument".into(),
                    ));
                }
                let name = Self::normalize_header_name(&args[0].as_string());
                if !name.is_empty() {
                    let _ = header_entries.borrow_mut().delete_entry(&name);
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }
}
