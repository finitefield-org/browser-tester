use super::*;

#[derive(Debug, Default)]
struct CookieQuery {
    name: Option<String>,
    path: Option<String>,
    domain: Option<String>,
    partitioned: Option<bool>,
}

impl Harness {
    fn cookie_store_resolved_promise(&mut self, value: Value) -> Result<Value> {
        let promise = self.new_pending_promise();
        self.promise_resolve(&promise, value)?;
        Ok(Value::Promise(promise))
    }

    fn cookie_query_from_value(&self, value: Option<&Value>) -> Result<CookieQuery> {
        let Some(value) = value else {
            return Ok(CookieQuery::default());
        };
        match value {
            Value::Undefined | Value::Null => Ok(CookieQuery::default()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                let mut query = CookieQuery::default();
                if let Some(name) = Self::object_get_entry(&entries, "name")
                    .filter(|value| !matches!(value, Value::Undefined | Value::Null))
                {
                    query.name = Some(name.as_string());
                }
                if let Some(path) = Self::object_get_entry(&entries, "path")
                    .filter(|value| !matches!(value, Value::Undefined | Value::Null))
                {
                    query.path = Some(path.as_string());
                }
                if let Some(domain) = Self::object_get_entry(&entries, "domain")
                    .filter(|value| !matches!(value, Value::Undefined | Value::Null))
                {
                    query.domain = Some(domain.as_string());
                }
                if let Some(partitioned) = Self::object_get_entry(&entries, "partitioned")
                    .filter(|value| !matches!(value, Value::Undefined | Value::Null))
                {
                    query.partitioned = Some(partitioned.truthy());
                }
                Ok(query)
            }
            other => Ok(CookieQuery {
                name: Some(other.as_string()),
                ..CookieQuery::default()
            }),
        }
    }

    fn cookie_record_from_set_args(&self, args: &[Value]) -> Result<CookieRecord> {
        match args.len() {
            1 => {
                let Value::Object(entries) = &args[0] else {
                    return Err(Error::ScriptRuntime(
                        "CookieStore.set single argument must be an options object".into(),
                    ));
                };
                let entries = entries.borrow();

                let name = Self::object_get_entry(&entries, "name")
                    .map(|value| value.as_string())
                    .unwrap_or_default();
                if name.trim().is_empty() {
                    return Err(Error::ScriptRuntime(
                        "CookieStore.set options.name is required".into(),
                    ));
                }

                let value = Self::object_get_entry(&entries, "value")
                    .map(|value| value.as_string())
                    .unwrap_or_default();
                let domain = Self::object_get_entry(&entries, "domain")
                    .filter(|value| !matches!(value, Value::Undefined | Value::Null))
                    .map(|value| value.as_string());
                let path = Self::object_get_entry(&entries, "path")
                    .filter(|value| !matches!(value, Value::Undefined | Value::Null))
                    .map(|value| value.as_string())
                    .unwrap_or_else(|| "/".to_string());
                let expires_ms = Self::object_get_entry(&entries, "expires")
                    .filter(|value| !matches!(value, Value::Undefined | Value::Null))
                    .map(|value| Self::value_to_i64(&value));
                let secure = Self::object_get_entry(&entries, "secure")
                    .map(|value| value.truthy())
                    .unwrap_or(false);
                let same_site = Self::object_get_entry(&entries, "sameSite")
                    .filter(|value| !matches!(value, Value::Undefined | Value::Null))
                    .map(|value| value.as_string());
                let partitioned = Self::object_get_entry(&entries, "partitioned")
                    .map(|value| value.truthy())
                    .unwrap_or(false);

                Ok(CookieRecord {
                    name,
                    value,
                    domain,
                    path,
                    expires_ms,
                    secure,
                    same_site,
                    partitioned,
                })
            }
            2 => Ok(CookieRecord {
                name: args[0].as_string(),
                value: args[1].as_string(),
                domain: None,
                path: "/".to_string(),
                expires_ms: None,
                secure: false,
                same_site: None,
                partitioned: false,
            }),
            _ => Err(Error::ScriptRuntime(
                "CookieStore.set requires one or two arguments".into(),
            )),
        }
    }

    pub(crate) fn eval_cookie_store_member_call(
        &mut self,
        cookie_store_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let is_cookie_store = {
            let entries = cookie_store_object.borrow();
            Self::is_cookie_store_object(&entries)
        };
        if !is_cookie_store {
            return Ok(None);
        }

        match member {
            "set" => {
                let record = self.cookie_record_from_set_args(args)?;
                let expires_in_past = record
                    .expires_ms
                    .is_some_and(|expires| expires <= self.scheduler.now_ms);
                let changed = self.upsert_cookie_record(record.clone());
                if changed {
                    if expires_in_past {
                        self.dispatch_cookie_store_change_event(&[], &[record.clone()])?;
                    } else {
                        self.dispatch_cookie_store_change_event(&[record.clone()], &[])?;
                    }
                }
                self.sync_document_cookie_property();
                Ok(Some(self.cookie_store_resolved_promise(Value::Undefined)?))
            }
            "get" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "CookieStore.get supports at most one argument".into(),
                    ));
                }
                let query = self.cookie_query_from_value(args.first())?;
                if query
                    .name
                    .as_deref()
                    .is_some_and(|name| name.trim().is_empty())
                {
                    return Ok(Some(self.cookie_store_resolved_promise(Value::Null)?));
                }
                let cookie = self
                    .visible_cookie_records(
                        query.name.as_deref(),
                        query.path.as_deref(),
                        query.domain.as_deref(),
                        query.partitioned,
                    )
                    .into_iter()
                    .next();
                self.sync_document_cookie_property();
                Ok(Some(
                    self.cookie_store_resolved_promise(
                        cookie
                            .as_ref()
                            .map(Self::cookie_record_to_value)
                            .unwrap_or(Value::Null),
                    )?,
                ))
            }
            "getAll" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "CookieStore.getAll supports at most one argument".into(),
                    ));
                }
                let query = self.cookie_query_from_value(args.first())?;
                let cookies = self.visible_cookie_records(
                    query.name.as_deref(),
                    query.path.as_deref(),
                    query.domain.as_deref(),
                    query.partitioned,
                );
                self.sync_document_cookie_property();
                Ok(Some(
                    self.cookie_store_resolved_promise(Self::new_array_value(
                        cookies
                            .iter()
                            .map(Self::cookie_record_to_value)
                            .collect::<Vec<_>>(),
                    ))?,
                ))
            }
            "delete" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "CookieStore.delete supports at most one argument".into(),
                    ));
                }
                let query = self.cookie_query_from_value(args.first())?;
                let deleted = self.delete_visible_cookie_records(
                    query.name.as_deref(),
                    query.path.as_deref(),
                    query.domain.as_deref(),
                    query.partitioned,
                );
                if !deleted.is_empty() {
                    self.dispatch_cookie_store_change_event(&[], &deleted)?;
                }
                self.sync_document_cookie_property();
                Ok(Some(self.cookie_store_resolved_promise(Value::Undefined)?))
            }
            "addEventListener" => {
                if args.is_empty() || args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "CookieStore.addEventListener requires two or three arguments".into(),
                    ));
                }
                if args[0].as_string() == "change" {
                    let callback = args.get(1).cloned().unwrap_or(Value::Undefined);
                    self.add_cookie_store_change_listener(callback)?;
                }
                Ok(Some(Value::Undefined))
            }
            "removeEventListener" => {
                if args.is_empty() || args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "CookieStore.removeEventListener requires two or three arguments".into(),
                    ));
                }
                if args[0].as_string() == "change" {
                    let callback = args.get(1).cloned().unwrap_or(Value::Undefined);
                    self.remove_cookie_store_change_listener(&callback);
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }
}
