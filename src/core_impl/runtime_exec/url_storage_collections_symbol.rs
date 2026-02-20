use super::*;

impl Harness {
    pub(crate) fn is_storage_method_name(name: &str) -> bool {
        matches!(name, "getItem" | "setItem" | "removeItem" | "clear" | "key")
    }

    pub(crate) fn storage_pairs_to_value(pairs: &[(String, String)]) -> Value {
        Self::new_array_value(
            pairs
                .iter()
                .map(|(name, value)| {
                    Self::new_array_value(vec![
                        Value::String(name.clone()),
                        Value::String(value.clone()),
                    ])
                })
                .collect::<Vec<_>>(),
        )
    }

    pub(crate) fn storage_pairs_from_object_entries(
        entries: &[(String, Value)],
    ) -> Vec<(String, String)> {
        let Some(Value::Array(list)) = Self::object_get_entry(entries, INTERNAL_STORAGE_ENTRIES_KEY)
        else {
            return Vec::new();
        };
        let snapshot = list.borrow().clone();
        let mut pairs = Vec::new();
        for item in snapshot {
            let Value::Array(pair) = item else {
                continue;
            };
            let pair = pair.borrow();
            if pair.is_empty() {
                continue;
            }
            let name = pair[0].as_string();
            let value = pair.get(1).map(Value::as_string).unwrap_or_default();
            pairs.push((name, value));
        }
        pairs
    }

    pub(crate) fn set_storage_pairs(
        entries: &mut impl ObjectEntryMut,
        pairs: &[(String, String)],
    ) {
        Self::object_set_entry(
            entries,
            INTERNAL_STORAGE_ENTRIES_KEY.to_string(),
            Self::storage_pairs_to_value(pairs),
        );
    }

    pub(crate) fn eval_storage_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "getItem" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Storage.getItem requires exactly one argument".into(),
                    ));
                }
                let key = args[0].as_string();
                let value = Self::storage_pairs_from_object_entries(&object.borrow())
                    .into_iter()
                    .find_map(|(name, value)| (name == key).then_some(value))
                    .map(Value::String)
                    .unwrap_or(Value::Null);
                Ok(Some(value))
            }
            "setItem" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Storage.setItem requires exactly two arguments".into(),
                    ));
                }
                let key = args[0].as_string();
                let value = args[1].as_string();
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::storage_pairs_from_object_entries(&entries);
                    if let Some((_, stored)) = pairs.iter_mut().find(|(name, _)| name == &key) {
                        *stored = value;
                    } else {
                        pairs.push((key, value));
                    }
                    Self::set_storage_pairs(&mut entries, &pairs);
                }
                Ok(Some(Value::Undefined))
            }
            "removeItem" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Storage.removeItem requires exactly one argument".into(),
                    ));
                }
                let key = args[0].as_string();
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::storage_pairs_from_object_entries(&entries);
                    pairs.retain(|(name, _)| name != &key);
                    Self::set_storage_pairs(&mut entries, &pairs);
                }
                Ok(Some(Value::Undefined))
            }
            "clear" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Storage.clear does not take arguments".into(),
                    ));
                }
                Self::set_storage_pairs(&mut object.borrow_mut(), &[]);
                Ok(Some(Value::Undefined))
            }
            "key" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Storage.key requires exactly one argument".into(),
                    ));
                }
                let Some(index) = self.value_as_index(&args[0]) else {
                    return Ok(Some(Value::Null));
                };
                let value = Self::storage_pairs_from_object_entries(&object.borrow())
                    .get(index)
                    .map(|(name, _)| Value::String(name.clone()))
                    .unwrap_or(Value::Null);
                Ok(Some(value))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn is_url_search_params_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_URL_SEARCH_PARAMS_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn url_search_params_pairs_to_value(pairs: &[(String, String)]) -> Value {
        Self::new_array_value(
            pairs
                .iter()
                .map(|(name, value)| {
                    Self::new_array_value(vec![
                        Value::String(name.clone()),
                        Value::String(value.clone()),
                    ])
                })
                .collect::<Vec<_>>(),
        )
    }

    pub(crate) fn url_search_params_pairs_from_object_entries(
        entries: &[(String, Value)],
    ) -> Vec<(String, String)> {
        let Some(Value::Array(list)) =
            Self::object_get_entry(entries, INTERNAL_URL_SEARCH_PARAMS_ENTRIES_KEY)
        else {
            return Vec::new();
        };
        let snapshot = list.borrow().clone();
        let mut pairs = Vec::new();
        for item in snapshot {
            let Value::Array(pair) = item else {
                continue;
            };
            let pair = pair.borrow();
            if pair.is_empty() {
                continue;
            }
            let name = pair[0].as_string();
            let value = pair.get(1).map(Value::as_string).unwrap_or_default();
            pairs.push((name, value));
        }
        pairs
    }

    pub(crate) fn set_url_search_params_pairs(
        entries: &mut impl ObjectEntryMut,
        pairs: &[(String, String)],
    ) {
        Self::object_set_entry(
            entries,
            INTERNAL_URL_SEARCH_PARAMS_ENTRIES_KEY.to_string(),
            Self::url_search_params_pairs_to_value(pairs),
        );
    }

    pub(crate) fn new_url_search_params_value(
        &self,
        pairs: Vec<(String, String)>,
        owner_id: Option<usize>,
    ) -> Value {
        let mut entries = vec![(
            INTERNAL_URL_SEARCH_PARAMS_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )];
        if let Some(owner_id) = owner_id {
            entries.push((
                INTERNAL_URL_SEARCH_PARAMS_OWNER_ID_KEY.to_string(),
                Value::Number(owner_id as i64),
            ));
        }
        Self::set_url_search_params_pairs(&mut entries, &pairs);
        Self::new_object_value(entries)
    }

    pub(crate) fn resolve_url_search_params_object_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        match env.get(target) {
            Some(Value::Object(entries)) => {
                if Self::is_url_search_params_object(&entries.borrow()) {
                    Ok(entries.clone())
                } else {
                    Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a URLSearchParams",
                        target
                    )))
                }
            }
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a URLSearchParams",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(crate) fn url_search_params_pairs_from_init_value(
        &self,
        init: &Value,
    ) -> Result<Vec<(String, String)>> {
        match init {
            Value::Undefined | Value::Null => Ok(Vec::new()),
            Value::String(text) => parse_url_search_params_pairs_from_query_string(text),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    Ok(Self::url_search_params_pairs_from_object_entries(&entries))
                } else {
                    let mut pairs = Vec::new();
                    for (name, value) in entries.iter() {
                        if Self::is_internal_object_key(name) {
                            continue;
                        }
                        pairs.push((name.clone(), value.as_string()));
                    }
                    Ok(pairs)
                }
            }
            Value::Array(_) | Value::Map(_) | Value::Set(_) | Value::TypedArray(_) => {
                let iterable = self.array_like_values_from_value(init)?;
                let mut pairs = Vec::new();
                for entry in iterable {
                    let pair = self.array_like_values_from_value(&entry).map_err(|_| {
                        Error::ScriptRuntime(
                            "URLSearchParams iterable values must be [name, value] pairs".into(),
                        )
                    })?;
                    if pair.len() < 2 {
                        return Err(Error::ScriptRuntime(
                            "URLSearchParams iterable values must be [name, value] pairs".into(),
                        ));
                    }
                    pairs.push((pair[0].as_string(), pair[1].as_string()));
                }
                Ok(pairs)
            }
            other => parse_url_search_params_pairs_from_query_string(&other.as_string()),
        }
    }

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
                self.browser_apis.blob_url_objects.insert(object_url.clone(), blob);
                Ok(Some(Value::String(object_url)))
            }
            "revokeObjectURL" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URL.revokeObjectURL requires exactly one argument".into(),
                    ));
                }
                self.browser_apis.blob_url_objects.remove(&args[0].as_string());
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

    pub(crate) fn eval_map_construct(
        &mut self,
        iterable: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Map constructor must be called with new".into(),
            ));
        }

        let map = Rc::new(RefCell::new(MapValue {
            entries: Vec::new(),
            properties: ObjectValue::default(),
        }));

        let Some(iterable) = iterable else {
            return Ok(Value::Map(map));
        };

        let iterable = self.eval_expr(iterable, env, event_param, event)?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Map(map));
        }

        match iterable {
            Value::Map(source) => {
                let source = source.borrow();
                map.borrow_mut().entries = source.entries.clone();
            }
            other => {
                let entries = self.array_like_values_from_value(&other)?;
                for entry in entries {
                    let pair = self.array_like_values_from_value(&entry).map_err(|_| {
                        Error::ScriptRuntime(
                            "Map constructor iterable values must be [key, value] pairs".into(),
                        )
                    })?;
                    if pair.len() < 2 {
                        return Err(Error::ScriptRuntime(
                            "Map constructor iterable values must be [key, value] pairs".into(),
                        ));
                    }
                    self.map_set_entry(&mut map.borrow_mut(), pair[0].clone(), pair[1].clone());
                }
            }
        }

        Ok(Value::Map(map))
    }

    pub(crate) fn eval_map_static_method(
        &mut self,
        method: MapStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            MapStaticMethod::GroupBy => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.groupBy requires exactly two arguments".into(),
                    ));
                }
                let iterable = self.eval_expr(&args[0], env, event_param, event)?;
                let callback = self.eval_expr(&args[1], env, event_param, event)?;
                let values = self.array_like_values_from_value(&iterable)?;
                let map = Rc::new(RefCell::new(MapValue {
                    entries: Vec::new(),
                    properties: ObjectValue::default(),
                }));
                for (index, item) in values.into_iter().enumerate() {
                    let group_key = self.execute_callback_value(
                        &callback,
                        &[item.clone(), Value::Number(index as i64)],
                        event,
                    )?;
                    let mut map_ref = map.borrow_mut();
                    if let Some(entry_index) = self.map_entry_index(&map_ref, &group_key) {
                        match &mut map_ref.entries[entry_index].1 {
                            Value::Array(group_values) => group_values.borrow_mut().push(item),
                            _ => {
                                map_ref.entries[entry_index].1 = Self::new_array_value(vec![item]);
                            }
                        }
                    } else {
                        map_ref
                            .entries
                            .push((group_key, Self::new_array_value(vec![item])));
                    }
                }
                Ok(Value::Map(map))
            }
        }
    }

    pub(crate) fn eval_map_method(
        &mut self,
        target: &str,
        method: MapInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let target_value = env
            .get(target)
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {}", target)))?;

        if let Value::Set(set) = target_value {
            let set = set.clone();
            return match method {
                MapInstanceMethod::Has => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.has requires exactly one argument".into(),
                        ));
                    }
                    let key = self.eval_expr(&args[0], env, event_param, event)?;
                    Ok(Value::Bool(
                        self.set_value_index(&set.borrow(), &key).is_some(),
                    ))
                }
                MapInstanceMethod::Delete => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.delete requires exactly one argument".into(),
                        ));
                    }
                    let key = self.eval_expr(&args[0], env, event_param, event)?;
                    let mut set_ref = set.borrow_mut();
                    if let Some(index) = self.set_value_index(&set_ref, &key) {
                        set_ref.values.remove(index);
                        Ok(Value::Bool(true))
                    } else {
                        Ok(Value::Bool(false))
                    }
                }
                MapInstanceMethod::Clear => {
                    if !args.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "Map.clear does not take arguments".into(),
                        ));
                    }
                    set.borrow_mut().values.clear();
                    Ok(Value::Undefined)
                }
                MapInstanceMethod::ForEach => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Map.forEach requires a callback and optional thisArg".into(),
                        ));
                    }
                    let callback = self.eval_expr(&args[0], env, event_param, event)?;
                    if args.len() == 2 {
                        let _ = self.eval_expr(&args[1], env, event_param, event)?;
                    }
                    let snapshot = set.borrow().values.clone();
                    for value in snapshot {
                        let _ = self.execute_callback_value(
                            &callback,
                            &[value.clone(), value, Value::Set(set.clone())],
                            event,
                        )?;
                    }
                    Ok(Value::Undefined)
                }
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a Map",
                    target
                ))),
            };
        }

        if let Value::FormData(entries) = target_value {
            let entries = entries.clone();
            return match method {
                MapInstanceMethod::Get => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.get requires exactly one argument".into(),
                        ));
                    }
                    let key = self
                        .eval_expr(&args[0], env, event_param, event)?
                        .as_string();
                    let value = entries
                        .iter()
                        .find_map(|(entry_name, value)| (entry_name == &key).then(|| value.clone()))
                        .unwrap_or_default();
                    Ok(Value::String(value))
                }
                MapInstanceMethod::Has => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.has requires exactly one argument".into(),
                        ));
                    }
                    let key = self
                        .eval_expr(&args[0], env, event_param, event)?
                        .as_string();
                    let has = entries.iter().any(|(entry_name, _)| entry_name == &key);
                    Ok(Value::Bool(has))
                }
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a Map",
                    target
                ))),
            };
        }

        if let Value::Object(entries) = target_value {
            let entries = entries.clone();
            if Self::is_storage_object(&entries.borrow()) {
                return match method {
                    MapInstanceMethod::Clear => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Storage.clear does not take arguments".into(),
                            ));
                        }
                        Self::set_storage_pairs(&mut entries.borrow_mut(), &[]);
                        Ok(Value::Undefined)
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a Map",
                        target
                    ))),
                };
            }
            if Self::is_url_search_params_object(&entries.borrow()) {
                return match method {
                    MapInstanceMethod::Get => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.get requires exactly one argument".into(),
                            ));
                        }
                        let name = self
                            .eval_expr(&args[0], env, event_param, event)?
                            .as_string();
                        let pairs =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        let value = pairs
                            .into_iter()
                            .find_map(|(entry_name, entry_value)| {
                                (entry_name == name).then_some(entry_value)
                            })
                            .map(Value::String)
                            .unwrap_or(Value::Null);
                        Ok(value)
                    }
                    MapInstanceMethod::Has => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.has requires exactly one argument".into(),
                            ));
                        }
                        let name = self
                            .eval_expr(&args[0], env, event_param, event)?
                            .as_string();
                        let pairs =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        Ok(Value::Bool(
                            pairs.into_iter().any(|(entry_name, _)| entry_name == name),
                        ))
                    }
                    MapInstanceMethod::Delete => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.delete requires exactly one argument".into(),
                            ));
                        }
                        let name = self
                            .eval_expr(&args[0], env, event_param, event)?
                            .as_string();
                        {
                            let mut object_ref = entries.borrow_mut();
                            let mut pairs =
                                Self::url_search_params_pairs_from_object_entries(&object_ref);
                            pairs.retain(|(entry_name, _)| entry_name != &name);
                            Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                        }
                        self.sync_url_search_params_owner(&entries);
                        Ok(Value::Undefined)
                    }
                    MapInstanceMethod::ForEach => {
                        if args.is_empty() || args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.forEach requires a callback and optional thisArg"
                                    .into(),
                            ));
                        }
                        let callback = self.eval_expr(&args[0], env, event_param, event)?;
                        if args.len() == 2 {
                            let _ = self.eval_expr(&args[1], env, event_param, event)?;
                        }
                        let snapshot =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        for (entry_name, entry_value) in snapshot {
                            let _ = self.execute_callback_value(
                                &callback,
                                &[
                                    Value::String(entry_value),
                                    Value::String(entry_name),
                                    Value::Object(entries.clone()),
                                ],
                                event,
                            )?;
                        }
                        Ok(Value::Undefined)
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a Map",
                        target
                    ))),
                };
            }
        }

        let Value::Map(map) = target_value else {
            if matches!(method, MapInstanceMethod::Get | MapInstanceMethod::Has) {
                return Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a FormData instance",
                    target
                )));
            }
            return Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a Map",
                target
            )));
        };
        let map = map.clone();
        match method {
            MapInstanceMethod::Get => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Map.get requires exactly one argument".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let map_ref = map.borrow();
                if let Some(index) = self.map_entry_index(&map_ref, &key) {
                    Ok(map_ref.entries[index].1.clone())
                } else {
                    Ok(Value::Undefined)
                }
            }
            MapInstanceMethod::Has => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Map.has requires exactly one argument".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let has = self.map_entry_index(&map.borrow(), &key).is_some();
                Ok(Value::Bool(has))
            }
            MapInstanceMethod::Delete => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Map.delete requires exactly one argument".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let mut map_ref = map.borrow_mut();
                if let Some(index) = self.map_entry_index(&map_ref, &key) {
                    map_ref.entries.remove(index);
                    Ok(Value::Bool(true))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            MapInstanceMethod::Clear => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Map.clear does not take arguments".into(),
                    ));
                }
                map.borrow_mut().entries.clear();
                Ok(Value::Undefined)
            }
            MapInstanceMethod::ForEach => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.forEach requires a callback and optional thisArg".into(),
                    ));
                }
                let callback = self.eval_expr(&args[0], env, event_param, event)?;
                if args.len() == 2 {
                    let _ = self.eval_expr(&args[1], env, event_param, event)?;
                }
                let snapshot = map.borrow().entries.clone();
                for (key, value) in snapshot {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[value, key, Value::Map(map.clone())],
                        event,
                    )?;
                }
                Ok(Value::Undefined)
            }
            MapInstanceMethod::GetOrInsert => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.getOrInsert requires exactly two arguments".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let default_value = self.eval_expr(&args[1], env, event_param, event)?;
                let mut map_ref = map.borrow_mut();
                if let Some(index) = self.map_entry_index(&map_ref, &key) {
                    Ok(map_ref.entries[index].1.clone())
                } else {
                    map_ref.entries.push((key, default_value.clone()));
                    Ok(default_value)
                }
            }
            MapInstanceMethod::GetOrInsertComputed => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.getOrInsertComputed requires exactly two arguments".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                {
                    let map_ref = map.borrow();
                    if let Some(index) = self.map_entry_index(&map_ref, &key) {
                        return Ok(map_ref.entries[index].1.clone());
                    }
                }
                let callback = self.eval_expr(&args[1], env, event_param, event)?;
                let computed =
                    self.execute_callback_value(&callback, std::slice::from_ref(&key), event)?;
                map.borrow_mut().entries.push((key, computed.clone()));
                Ok(computed)
            }
        }
    }

    pub(crate) fn eval_set_construct(
        &mut self,
        iterable: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Set constructor must be called with new".into(),
            ));
        }

        let set = Rc::new(RefCell::new(SetValue {
            values: Vec::new(),
            properties: ObjectValue::default(),
        }));

        let Some(iterable) = iterable else {
            return Ok(Value::Set(set));
        };

        let iterable = self.eval_expr(iterable, env, event_param, event)?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Set(set));
        }

        let values = self.array_like_values_from_value(&iterable)?;
        for value in values {
            self.set_add_value(&mut set.borrow_mut(), value);
        }
        Ok(Value::Set(set))
    }

    pub(crate) fn eval_set_method(
        &mut self,
        target: &str,
        method: SetInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let target_value = env
            .get(target)
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {}", target)))?;
        let Value::Set(set) = target_value else {
            return Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a Set",
                target
            )));
        };
        let set = set.clone();

        match method {
            SetInstanceMethod::Add => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.add requires exactly one argument".into(),
                    ));
                }
                let value = self.eval_expr(&args[0], env, event_param, event)?;
                self.set_add_value(&mut set.borrow_mut(), value);
                Ok(Value::Set(set))
            }
            SetInstanceMethod::Union => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.union requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let other_keys = self.set_like_keys_snapshot(&other)?;
                let mut out = SetValue {
                    values: set.borrow().values.clone(),
                    properties: ObjectValue::default(),
                };
                for key in other_keys {
                    self.set_add_value(&mut out, key);
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::Intersection => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.intersection requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = set.borrow().values.clone();
                let mut out = SetValue {
                    values: Vec::new(),
                    properties: ObjectValue::default(),
                };
                for value in snapshot {
                    if self.set_like_has_value(&other, &value)? {
                        self.set_add_value(&mut out, value);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::Difference => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.difference requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = set.borrow().values.clone();
                let mut out = SetValue {
                    values: Vec::new(),
                    properties: ObjectValue::default(),
                };
                for value in snapshot {
                    if !self.set_like_has_value(&other, &value)? {
                        self.set_add_value(&mut out, value);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::SymmetricDifference => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.symmetricDifference requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let other_keys = self.set_like_keys_snapshot(&other)?;
                let mut out = SetValue {
                    values: set.borrow().values.clone(),
                    properties: ObjectValue::default(),
                };
                for key in other_keys {
                    if let Some(index) = self.set_value_index(&out, &key) {
                        out.values.remove(index);
                    } else {
                        out.values.push(key);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::IsDisjointFrom => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.isDisjointFrom requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                for value in &set.borrow().values {
                    if self.set_like_has_value(&other, value)? {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            SetInstanceMethod::IsSubsetOf => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.isSubsetOf requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                for value in &set.borrow().values {
                    if !self.set_like_has_value(&other, value)? {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            SetInstanceMethod::IsSupersetOf => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.isSupersetOf requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                for value in self.set_like_keys_snapshot(&other)? {
                    if self.set_value_index(&set.borrow(), &value).is_none() {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
        }
    }

    pub(crate) fn new_symbol_value(
        &mut self,
        description: Option<String>,
        registry_key: Option<String>,
    ) -> Value {
        let id = self.symbol_runtime.allocate_symbol_id();
        let symbol = Rc::new(SymbolValue {
            id,
            description,
            registry_key,
        });
        self.symbol_runtime.symbols_by_id.insert(id, symbol.clone());
        Value::Symbol(symbol)
    }

    pub(crate) fn symbol_storage_key(id: usize) -> String {
        format!("{INTERNAL_SYMBOL_KEY_PREFIX}{id}")
    }

    pub(crate) fn symbol_id_from_storage_key(key: &str) -> Option<usize> {
        key.strip_prefix(INTERNAL_SYMBOL_KEY_PREFIX)
            .and_then(|value| value.parse::<usize>().ok())
    }

    pub(crate) fn is_symbol_storage_key(key: &str) -> bool {
        key.starts_with(INTERNAL_SYMBOL_KEY_PREFIX)
    }

    pub(crate) fn is_internal_object_key(key: &str) -> bool {
        Self::is_symbol_storage_key(key)
            || key == INTERNAL_SYMBOL_WRAPPER_KEY
            || key == INTERNAL_STRING_WRAPPER_VALUE_KEY
            || key.starts_with(INTERNAL_INTL_KEY_PREFIX)
            || key.starts_with(INTERNAL_CALLABLE_KEY_PREFIX)
            || key.starts_with(INTERNAL_URL_SEARCH_PARAMS_KEY_PREFIX)
            || key.starts_with(INTERNAL_STORAGE_KEY_PREFIX)
    }

    pub(crate) fn symbol_wrapper_id_from_object(entries: &[(String, Value)]) -> Option<usize> {
        let value = Self::object_get_entry(entries, INTERNAL_SYMBOL_WRAPPER_KEY)?;
        match value {
            Value::Number(value) if value >= 0 => Some(value as usize),
            _ => None,
        }
    }

    pub(crate) fn string_wrapper_value_from_object(entries: &[(String, Value)]) -> Option<String> {
        match Self::object_get_entry(entries, INTERNAL_STRING_WRAPPER_VALUE_KEY) {
            Some(Value::String(value)) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn symbol_id_from_property_key(&self, value: &Value) -> Option<usize> {
        match value {
            Value::Symbol(symbol) => Some(symbol.id),
            Value::Object(entries) => {
                let entries = entries.borrow();
                Self::symbol_wrapper_id_from_object(&entries)
            }
            _ => None,
        }
    }

    pub(crate) fn property_key_to_storage_key(&self, value: &Value) -> String {
        if let Some(symbol_id) = self.symbol_id_from_property_key(value) {
            Self::symbol_storage_key(symbol_id)
        } else {
            value.as_string()
        }
    }

    pub(crate) fn eval_symbol_construct(
        &mut self,
        description: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if called_with_new {
            return Err(Error::ScriptRuntime("Symbol is not a constructor".into()));
        }
        let description = if let Some(description) = description {
            let value = self.eval_expr(description, env, event_param, event)?;
            if matches!(value, Value::Undefined) {
                None
            } else {
                Some(value.as_string())
            }
        } else {
            None
        };
        Ok(self.new_symbol_value(description, None))
    }

    pub(crate) fn eval_symbol_static_method(
        &mut self,
        method: SymbolStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            SymbolStaticMethod::For => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Symbol.for requires exactly one argument".into(),
                    ));
                }
                let key = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                if let Some(symbol) = self.symbol_runtime.symbol_registry.get(&key) {
                    return Ok(Value::Symbol(symbol.clone()));
                }
                let symbol = match self.new_symbol_value(Some(key.clone()), Some(key.clone())) {
                    Value::Symbol(symbol) => symbol,
                    _ => unreachable!("new_symbol_value must create Symbol"),
                };
                self.symbol_runtime.symbol_registry.insert(key, symbol.clone());
                Ok(Value::Symbol(symbol))
            }
            SymbolStaticMethod::KeyFor => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Symbol.keyFor requires exactly one argument".into(),
                    ));
                }
                let symbol = self.eval_expr(&args[0], env, event_param, event)?;
                let Value::Symbol(symbol) = symbol else {
                    return Err(Error::ScriptRuntime(
                        "Symbol.keyFor argument must be a Symbol".into(),
                    ));
                };
                if let Some(key) = &symbol.registry_key {
                    Ok(Value::String(key.clone()))
                } else {
                    Ok(Value::Undefined)
                }
            }
        }
    }

    pub(crate) fn symbol_static_property_name(property: SymbolStaticProperty) -> &'static str {
        match property {
            SymbolStaticProperty::AsyncDispose => "Symbol.asyncDispose",
            SymbolStaticProperty::AsyncIterator => "Symbol.asyncIterator",
            SymbolStaticProperty::Dispose => "Symbol.dispose",
            SymbolStaticProperty::HasInstance => "Symbol.hasInstance",
            SymbolStaticProperty::IsConcatSpreadable => "Symbol.isConcatSpreadable",
            SymbolStaticProperty::Iterator => "Symbol.iterator",
            SymbolStaticProperty::Match => "Symbol.match",
            SymbolStaticProperty::MatchAll => "Symbol.matchAll",
            SymbolStaticProperty::Replace => "Symbol.replace",
            SymbolStaticProperty::Search => "Symbol.search",
            SymbolStaticProperty::Species => "Symbol.species",
            SymbolStaticProperty::Split => "Symbol.split",
            SymbolStaticProperty::ToPrimitive => "Symbol.toPrimitive",
            SymbolStaticProperty::ToStringTag => "Symbol.toStringTag",
            SymbolStaticProperty::Unscopables => "Symbol.unscopables",
        }
    }

    pub(crate) fn eval_symbol_static_property(&mut self, property: SymbolStaticProperty) -> Value {
        let name = Self::symbol_static_property_name(property).to_string();
        if let Some(symbol) = self.symbol_runtime.well_known_symbols.get(&name) {
            return Value::Symbol(symbol.clone());
        }
        let symbol = match self.new_symbol_value(Some(name.clone()), None) {
            Value::Symbol(symbol) => symbol,
            _ => unreachable!("new_symbol_value must create Symbol"),
        };
        self.symbol_runtime.well_known_symbols.insert(name, symbol.clone());
        Value::Symbol(symbol)
    }

    pub(crate) fn eval_string_static_method(
        &mut self,
        method: StringStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            StringStaticMethod::FromCharCode => {
                let mut units = Vec::with_capacity(args.len());
                for arg in args {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    let unit = (Self::value_to_i64(&value) as i128).rem_euclid(1 << 16) as u16;
                    units.push(unit);
                }
                Ok(Value::String(String::from_utf16_lossy(&units)))
            }
            StringStaticMethod::FromCodePoint => {
                let mut out = String::new();
                for arg in args {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    let n = Self::coerce_number_for_global(&value);
                    if !n.is_finite() || n.fract() != 0.0 || !(0.0..=0x10_FFFF as f64).contains(&n)
                    {
                        return Err(Error::ScriptRuntime(
                            "Invalid code point for String.fromCodePoint".into(),
                        ));
                    }
                    let cp = n as u32;
                    if (0xD800..=0xDFFF).contains(&cp) {
                        return Err(Error::ScriptRuntime(
                            "Invalid code point for String.fromCodePoint".into(),
                        ));
                    }
                    let ch = char::from_u32(cp).ok_or_else(|| {
                        Error::ScriptRuntime("Invalid code point for String.fromCodePoint".into())
                    })?;
                    out.push(ch);
                }
                Ok(Value::String(out))
            }
            StringStaticMethod::Raw => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "String.raw requires at least one argument".into(),
                    ));
                }
                let template = self.eval_expr(&args[0], env, event_param, event)?;
                let raw = match template {
                    Value::Object(entries) => {
                        let entries = entries.borrow();
                        Self::object_get_entry(&entries, "raw").unwrap_or(Value::Undefined)
                    }
                    other => other,
                };
                let raw_segments = self.array_like_values_from_value(&raw)?;
                let mut substitutions = Vec::with_capacity(args.len().saturating_sub(1));
                for arg in args.iter().skip(1) {
                    substitutions.push(self.eval_expr(arg, env, event_param, event)?.as_string());
                }
                if raw_segments.is_empty() {
                    return Ok(Value::String(String::new()));
                }
                let mut out = String::new();
                for (idx, segment) in raw_segments.iter().enumerate() {
                    out.push_str(&segment.as_string());
                    if let Some(substitution) = substitutions.get(idx) {
                        out.push_str(substitution);
                    }
                }
                Ok(Value::String(out))
            }
        }
    }

    pub(crate) fn eval_regexp_static_method(
        &mut self,
        method: RegExpStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            RegExpStaticMethod::Escape => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "RegExp.escape requires exactly one argument".into(),
                    ));
                }
                let value = self.eval_expr(&args[0], env, event_param, event)?;
                Ok(Value::String(regex_escape(&value.as_string()).into_owned()))
            }
        }
    }

    pub(crate) fn eval_string_match(&mut self, value: &str, pattern: Value) -> Result<Value> {
        let regex = if let Value::RegExp(regex) = pattern {
            regex
        } else {
            let compiled = Self::new_regex_from_values(&pattern, None)?;
            match compiled {
                Value::RegExp(regex) => regex,
                _ => unreachable!("RegExp constructor must return a RegExp"),
            }
        };

        let global = regex.borrow().global;
        if global {
            let compiled = regex.borrow().compiled.clone();
            let mut matches = Vec::new();
            for matched in compiled
                .find_all(value)
                .map_err(Self::map_regex_runtime_error)?
            {
                matches.push(Value::String(matched.as_str().to_string()));
            }
            regex.borrow_mut().last_index = 0;
            if matches.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(Self::new_array_value(matches))
            }
        } else {
            let Some(captures) = Self::regex_exec(&regex, value)? else {
                return Ok(Value::Null);
            };
            Ok(Self::new_array_value(
                captures.into_iter().map(Value::String).collect::<Vec<_>>(),
            ))
        }
    }

}
