use super::*;

impl Harness {
    pub(crate) fn eval_url_construct(
        &mut self,
        input: &Option<Box<Expr>>,
        base: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "URL constructor must be called with new".into(),
            ));
        }

        let input = input
            .as_ref()
            .map(|expr| self.eval_expr(expr, env, event_param, event))
            .transpose()?
            .unwrap_or(Value::Undefined)
            .as_string();
        let base = base
            .as_ref()
            .map(|expr| self.eval_expr(expr, env, event_param, event))
            .transpose()?
            .map(|value| value.as_string());

        let href = Self::resolve_url_string(&input, base.as_deref())
            .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
        self.new_url_value_from_href(&href)
    }

    pub(crate) fn eval_url_static_method(
        &mut self,
        method: UrlStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let member = match method {
            UrlStaticMethod::CanParse => "canParse",
            UrlStaticMethod::Parse => "parse",
            UrlStaticMethod::CreateObjectUrl => "createObjectURL",
            UrlStaticMethod::RevokeObjectUrl => "revokeObjectURL",
        };
        let evaluated_args = args
            .iter()
            .map(|arg| self.eval_expr(arg, env, event_param, event))
            .collect::<Result<Vec<_>>>()?;
        let url_constructor_override = {
            let entries = self.browser_apis.url_constructor_properties.borrow();
            Self::object_get_entry(&entries, member)
        };
        if let Some(callee) = url_constructor_override {
            return self
                .execute_callable_value_with_env(&callee, &evaluated_args, event, Some(env))
                .map_err(|err| match err {
                    Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                        Error::ScriptRuntime(format!("URL.{member} is not a function"))
                    }
                    other => other,
                });
        }
        self.eval_url_static_member_call_from_values(member, &evaluated_args)?
            .ok_or_else(|| Error::ScriptRuntime(format!("unsupported URL static method: {member}")))
    }

    pub(crate) fn eval_url_static_member_call_from_values(
        &mut self,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "canParse" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URL.canParse requires a URL argument and optional base".into(),
                    ));
                }
                let input = args[0].as_string();
                let base = args.get(1).map(Value::as_string);
                Ok(Some(Value::Bool(
                    Self::resolve_url_string(&input, base.as_deref()).is_some(),
                )))
            }
            "parse" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URL.parse requires a URL argument and optional base".into(),
                    ));
                }
                let input = args[0].as_string();
                let base = args.get(1).map(Value::as_string);
                if let Some(href) = Self::resolve_url_string(&input, base.as_deref()) {
                    Ok(Some(self.new_url_value_from_href(&href)?))
                } else {
                    Ok(Some(Value::Null))
                }
            }
            "createObjectURL" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URL.createObjectURL requires exactly one argument".into(),
                    ));
                }
                let blob = match args[0].clone() {
                    Value::Blob(blob) => blob,
                    Value::Object(entries) => {
                        let entries = entries.borrow();
                        if !Self::is_mock_file_object(&entries) {
                            return Err(Error::ScriptRuntime(
                                "URL.createObjectURL requires a Blob argument".into(),
                            ));
                        }
                        match Self::object_get_entry(&entries, INTERNAL_MOCK_FILE_BLOB_KEY) {
                            Some(Value::Blob(blob)) => blob,
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "URL.createObjectURL requires a Blob argument".into(),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "URL.createObjectURL requires a Blob argument".into(),
                        ));
                    }
                };
                let object_url = self.browser_apis.allocate_blob_url();
                self.browser_apis
                    .blob_url_objects
                    .insert(object_url.clone(), blob);
                Ok(Some(Value::String(object_url)))
            }
            "revokeObjectURL" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URL.revokeObjectURL requires exactly one argument".into(),
                    ));
                }
                self.browser_apis
                    .blob_url_objects
                    .remove(&args[0].as_string());
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_url_member_call(
        &self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        _args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "toString" | "toJSON" => {
                let href = {
                    let entries = object.borrow();
                    Self::object_get_entry(&entries, "href")
                        .map(|value| value.as_string())
                        .unwrap_or_default()
                };
                Ok(Some(Value::String(href)))
            }
            _ => Ok(None),
        }
    }
}
