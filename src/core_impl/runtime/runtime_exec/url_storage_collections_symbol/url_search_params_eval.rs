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
                let Value::Blob(blob) = args[0].clone() else {
                    return Err(Error::ScriptRuntime(
                        "URL.createObjectURL requires a Blob argument".into(),
                    ));
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
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "toString" | "toJSON" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(format!(
                        "URL.{member} does not take arguments"
                    )));
                }
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

    pub(crate) fn eval_url_search_params_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "append" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.append requires exactly two arguments".into(),
                    ));
                }
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&entries);
                    pairs.push((args[0].as_string(), args[1].as_string()));
                    Self::set_url_search_params_pairs(&mut entries, &pairs);
                }
                self.sync_url_search_params_owner(object);
                Ok(Some(Value::Undefined))
            }
            "delete" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.delete requires one or two arguments".into(),
                    ));
                }
                let name = args[0].as_string();
                let value = args.get(1).map(Value::as_string);
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&entries);
                    pairs.retain(|(entry_name, entry_value)| {
                        if entry_name != &name {
                            return true;
                        }
                        if let Some(value) = value.as_ref() {
                            entry_value != value
                        } else {
                            false
                        }
                    });
                    Self::set_url_search_params_pairs(&mut entries, &pairs);
                }
                self.sync_url_search_params_owner(object);
                Ok(Some(Value::Undefined))
            }
            "get" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.get requires exactly one argument".into(),
                    ));
                }
                let name = args[0].as_string();
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                let value = pairs
                    .into_iter()
                    .find_map(|(entry_name, entry_value)| {
                        (entry_name == name).then_some(entry_value)
                    })
                    .map(Value::String)
                    .unwrap_or(Value::Null);
                Ok(Some(value))
            }
            "getAll" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.getAll requires exactly one argument".into(),
                    ));
                }
                let name = args[0].as_string();
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Self::new_array_value(
                    pairs
                        .into_iter()
                        .filter_map(|(entry_name, entry_value)| {
                            (entry_name == name).then(|| Value::String(entry_value))
                        })
                        .collect::<Vec<_>>(),
                )))
            }
            "has" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.has requires one or two arguments".into(),
                    ));
                }
                let name = args[0].as_string();
                let value = args.get(1).map(Value::as_string);
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                let has = pairs.into_iter().any(|(entry_name, entry_value)| {
                    if entry_name != name {
                        return false;
                    }
                    if let Some(value) = value.as_ref() {
                        &entry_value == value
                    } else {
                        true
                    }
                });
                Ok(Some(Value::Bool(has)))
            }
            "set" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.set requires exactly two arguments".into(),
                    ));
                }
                let name = args[0].as_string();
                let value = args[1].as_string();
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&entries);
                    if let Some(first_match) =
                        pairs.iter().position(|(entry_name, _)| entry_name == &name)
                    {
                        pairs[first_match].1 = value;
                        let mut index = pairs.len();
                        while index > 0 {
                            index -= 1;
                            if index != first_match && pairs[index].0 == name {
                                pairs.remove(index);
                            }
                        }
                    } else {
                        pairs.push((name, value));
                    }
                    Self::set_url_search_params_pairs(&mut entries, &pairs);
                }
                self.sync_url_search_params_owner(object);
                Ok(Some(Value::Undefined))
            }
            "entries" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.entries does not take arguments".into(),
                    ));
                }
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Self::new_array_value(
                    pairs
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>(),
                )))
            }
            "keys" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.keys does not take arguments".into(),
                    ));
                }
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Self::new_array_value(
                    pairs
                        .into_iter()
                        .map(|(name, _)| Value::String(name))
                        .collect::<Vec<_>>(),
                )))
            }
            "values" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.values does not take arguments".into(),
                    ));
                }
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Self::new_array_value(
                    pairs
                        .into_iter()
                        .map(|(_, value)| Value::String(value))
                        .collect::<Vec<_>>(),
                )))
            }
            "sort" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.sort does not take arguments".into(),
                    ));
                }
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&entries);
                    pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                    Self::set_url_search_params_pairs(&mut entries, &pairs);
                }
                self.sync_url_search_params_owner(object);
                Ok(Some(Value::Undefined))
            }
            "forEach" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.forEach requires a callback and optional thisArg".into(),
                    ));
                }
                let callback = args[0].clone();
                let snapshot = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                for (entry_name, entry_value) in snapshot {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[
                            Value::String(entry_value),
                            Value::String(entry_name),
                            Value::Object(object.clone()),
                        ],
                        event,
                    )?;
                }
                Ok(Some(Value::Undefined))
            }
            "toString" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.toString does not take arguments".into(),
                    ));
                }
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Value::String(serialize_url_search_params_pairs(
                    &pairs,
                ))))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_url_search_params_construct(
        &mut self,
        init: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "URLSearchParams constructor must be called with new".into(),
            ));
        }
        let init = init
            .as_ref()
            .map(|expr| self.eval_expr(expr, env, event_param, event))
            .transpose()?
            .unwrap_or(Value::Undefined);
        let pairs = self.url_search_params_pairs_from_init_value(&init)?;
        Ok(self.new_url_search_params_value(pairs, None))
    }

    pub(crate) fn eval_url_search_params_method(
        &mut self,
        target: &str,
        method: UrlSearchParamsInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if matches!(method, UrlSearchParamsInstanceMethod::GetAll) {
            match env.get(target) {
                Some(Value::FormData(entries)) => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "FormData.getAll requires exactly one argument".into(),
                        ));
                    }
                    let name = self
                        .eval_expr(&args[0], env, event_param, event)?
                        .as_string();
                    return Ok(Self::new_array_value(
                        entries
                            .iter()
                            .filter_map(|(entry_name, entry_value)| {
                                (entry_name == &name).then(|| Value::String(entry_value.clone()))
                            })
                            .collect::<Vec<_>>(),
                    ));
                }
                Some(Value::Object(entries)) => {
                    if !Self::is_url_search_params_object(&entries.borrow()) {
                        return Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not a FormData instance",
                            target
                        )));
                    }
                }
                Some(_) => {
                    return Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a FormData instance",
                        target
                    )));
                }
                None => {
                    return Err(Error::ScriptRuntime(format!(
                        "unknown FormData variable: {}",
                        target
                    )));
                }
            }
        }

        let object = self.resolve_url_search_params_object_from_env(env, target)?;
        match method {
            UrlSearchParamsInstanceMethod::Append => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.append requires exactly two arguments".into(),
                    ));
                }
                let name = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                let value = self
                    .eval_expr(&args[1], env, event_param, event)?
                    .as_string();
                {
                    let mut object_ref = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&object_ref);
                    pairs.push((name, value));
                    Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                }
                self.sync_url_search_params_owner(&object);
                Ok(Value::Undefined)
            }
            UrlSearchParamsInstanceMethod::Delete => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.delete requires one or two arguments".into(),
                    ));
                }
                let name = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                let value = if args.len() == 2 {
                    Some(
                        self.eval_expr(&args[1], env, event_param, event)?
                            .as_string(),
                    )
                } else {
                    None
                };
                {
                    let mut object_ref = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&object_ref);
                    pairs.retain(|(entry_name, entry_value)| {
                        if entry_name != &name {
                            return true;
                        }
                        if let Some(value) = value.as_ref() {
                            entry_value != value
                        } else {
                            false
                        }
                    });
                    Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                }
                self.sync_url_search_params_owner(&object);
                Ok(Value::Undefined)
            }
            UrlSearchParamsInstanceMethod::GetAll => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.getAll requires exactly one argument".into(),
                    ));
                }
                let name = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Self::new_array_value(
                    pairs
                        .into_iter()
                        .filter_map(|(entry_name, entry_value)| {
                            (entry_name == name).then(|| Value::String(entry_value))
                        })
                        .collect::<Vec<_>>(),
                ))
            }
            UrlSearchParamsInstanceMethod::Has => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.has requires one or two arguments".into(),
                    ));
                }
                let name = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                let value = if args.len() == 2 {
                    Some(
                        self.eval_expr(&args[1], env, event_param, event)?
                            .as_string(),
                    )
                } else {
                    None
                };
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                let has = pairs.into_iter().any(|(entry_name, entry_value)| {
                    if entry_name != name {
                        return false;
                    }
                    if let Some(value) = value.as_ref() {
                        &entry_value == value
                    } else {
                        true
                    }
                });
                Ok(Value::Bool(has))
            }
        }
    }
}
