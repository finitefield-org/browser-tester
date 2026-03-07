use super::*;

impl Harness {
    fn eval_overlapping_map_method_from_values(
        &self,
        target_value: &Value,
        method: MapInstanceMethod,
        args: &[Value],
    ) -> Result<Value> {
        match target_value {
            Value::Map(map) => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(match method {
                        MapInstanceMethod::Has => "Map.has requires exactly one argument".into(),
                        MapInstanceMethod::Delete => {
                            "Map.delete requires exactly one argument".into()
                        }
                        _ => unreachable!(),
                    }));
                }
                let key = &args[0];
                match method {
                    MapInstanceMethod::Has => Ok(Value::Bool(
                        self.map_entry_index(&map.borrow(), key).is_some(),
                    )),
                    MapInstanceMethod::Delete => {
                        let mut map_ref = map.borrow_mut();
                        if let Some(index) = self.map_entry_index(&map_ref, key) {
                            map_ref.entries.remove(index);
                            Ok(Value::Bool(true))
                        } else {
                            Ok(Value::Bool(false))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Value::Set(set) => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(match method {
                        MapInstanceMethod::Has => "Map.has requires exactly one argument".into(),
                        MapInstanceMethod::Delete => {
                            "Map.delete requires exactly one argument".into()
                        }
                        _ => unreachable!(),
                    }));
                }
                let key = &args[0];
                match method {
                    MapInstanceMethod::Has => Ok(Value::Bool(
                        self.set_value_index(&set.borrow(), key).is_some(),
                    )),
                    MapInstanceMethod::Delete => {
                        let mut set_ref = set.borrow_mut();
                        if let Some(index) = self.set_value_index(&set_ref, key) {
                            set_ref.values.remove(index);
                            Ok(Value::Bool(true))
                        } else {
                            Ok(Value::Bool(false))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Value::WeakMap(weak_map) => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(match method {
                        MapInstanceMethod::Has => {
                            "WeakMap.has requires exactly one argument".into()
                        }
                        MapInstanceMethod::Delete => {
                            "WeakMap.delete requires exactly one argument".into()
                        }
                        _ => unreachable!(),
                    }));
                }
                let key = &args[0];
                if !Self::weak_map_accepts_key(key) {
                    return Ok(Value::Bool(false));
                }
                match method {
                    MapInstanceMethod::Has => Ok(Value::Bool(
                        self.weak_map_entry_index(&weak_map.borrow(), key).is_some(),
                    )),
                    MapInstanceMethod::Delete => {
                        let mut weak_map_ref = weak_map.borrow_mut();
                        if let Some(index) = self.weak_map_entry_index(&weak_map_ref, key) {
                            weak_map_ref.entries.remove(index);
                            Ok(Value::Bool(true))
                        } else {
                            Ok(Value::Bool(false))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Value::WeakSet(weak_set) => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(match method {
                        MapInstanceMethod::Has => {
                            "WeakSet.has requires exactly one argument".into()
                        }
                        MapInstanceMethod::Delete => {
                            "WeakSet.delete requires exactly one argument".into()
                        }
                        _ => unreachable!(),
                    }));
                }
                let key = &args[0];
                if !Self::weak_map_accepts_key(key) {
                    return Ok(Value::Bool(false));
                }
                match method {
                    MapInstanceMethod::Has => Ok(Value::Bool(
                        self.weak_set_value_index(&weak_set.borrow(), key).is_some(),
                    )),
                    MapInstanceMethod::Delete => {
                        let mut weak_set_ref = weak_set.borrow_mut();
                        if let Some(index) = self.weak_set_value_index(&weak_set_ref, key) {
                            weak_set_ref.values.remove(index);
                            Ok(Value::Bool(true))
                        } else {
                            Ok(Value::Bool(false))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
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

    pub(crate) fn eval_url_search_params_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "append" => {
                if args.len() < 2 {
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
                if args.is_empty() {
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
                if args.is_empty() {
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
                if args.is_empty() {
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
                if args.is_empty() {
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
                if args.len() < 2 {
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
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Self::new_array_value(
                    pairs
                        .into_iter()
                        .map(|(name, _)| Value::String(name))
                        .collect::<Vec<_>>(),
                )))
            }
            "values" => {
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Self::new_array_value(
                    pairs
                        .into_iter()
                        .map(|(_, value)| Value::String(value))
                        .collect::<Vec<_>>(),
                )))
            }
            "sort" => {
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
        if matches!(method, UrlSearchParamsInstanceMethod::Append)
            && matches!(env.get(target), Some(Value::Node(_)))
        {
            let node = match env.get(target) {
                Some(Value::Node(node)) => *node,
                _ => unreachable!(),
            };
            let evaluated_args = args
                .iter()
                .map(|arg| self.eval_expr(arg, env, event_param, event))
                .collect::<Result<Vec<_>>>()?;
            return self.eval_document_append_call(node, &evaluated_args);
        }

        if matches!(
            method,
            UrlSearchParamsInstanceMethod::Delete | UrlSearchParamsInstanceMethod::Has
        ) && matches!(
            env.get(target),
            Some(Value::Map(_) | Value::Set(_) | Value::WeakMap(_) | Value::WeakSet(_))
        ) {
            let target_value = env
                .get(target)
                .cloned()
                .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {}", target)))?;
            let evaluated_args = args
                .iter()
                .map(|arg| self.eval_expr(arg, env, event_param, event))
                .collect::<Result<Vec<_>>>()?;
            let map_method = match method {
                UrlSearchParamsInstanceMethod::Delete => MapInstanceMethod::Delete,
                UrlSearchParamsInstanceMethod::Has => MapInstanceMethod::Has,
                _ => unreachable!(),
            };
            return self.eval_overlapping_map_method_from_values(
                &target_value,
                map_method,
                &evaluated_args,
            );
        }

        if matches!(
            method,
            UrlSearchParamsInstanceMethod::GetAll
                | UrlSearchParamsInstanceMethod::Append
                | UrlSearchParamsInstanceMethod::Delete
                | UrlSearchParamsInstanceMethod::Has
                | UrlSearchParamsInstanceMethod::Set
        ) {
            match env.get(target) {
                Some(Value::FormData(entries)) => {
                    let evaluated_args = args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    return match method {
                        UrlSearchParamsInstanceMethod::GetAll => {
                            if evaluated_args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "FormData.getAll requires exactly one argument".into(),
                                ));
                            }
                            let name = evaluated_args[0].as_string();
                            let entries = entries.borrow();
                            Ok(Self::new_array_value(
                                entries
                                    .iter()
                                    .filter_map(|(entry_name, entry_value)| {
                                        (entry_name == &name)
                                            .then(|| Value::String(entry_value.clone()))
                                    })
                                    .collect::<Vec<_>>(),
                            ))
                        }
                        UrlSearchParamsInstanceMethod::Append => {
                            if evaluated_args.len() < 2 {
                                return Err(Error::ScriptRuntime(
                                    "FormData.append requires two or three arguments".into(),
                                ));
                            }
                            let name = evaluated_args[0].as_string();
                            let value = Self::form_data_append_string_value(
                                &evaluated_args[1],
                                evaluated_args.get(2),
                            );
                            entries.borrow_mut().push((name, value));
                            Ok(Value::Undefined)
                        }
                        UrlSearchParamsInstanceMethod::Delete => {
                            if evaluated_args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "FormData.delete requires exactly one argument".into(),
                                ));
                            }
                            let name = evaluated_args[0].as_string();
                            entries
                                .borrow_mut()
                                .retain(|(entry_name, _)| entry_name != &name);
                            Ok(Value::Undefined)
                        }
                        UrlSearchParamsInstanceMethod::Has => {
                            if evaluated_args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "FormData.has requires exactly one argument".into(),
                                ));
                            }
                            let name = evaluated_args[0].as_string();
                            let has = entries
                                .borrow()
                                .iter()
                                .any(|(entry_name, _)| entry_name == &name);
                            Ok(Value::Bool(has))
                        }
                        UrlSearchParamsInstanceMethod::Set => {
                            if evaluated_args.len() < 2 {
                                return Err(Error::ScriptRuntime(
                                    "FormData.set requires two or three arguments".into(),
                                ));
                            }
                            let name = evaluated_args[0].as_string();
                            let value = Self::form_data_append_string_value(
                                &evaluated_args[1],
                                evaluated_args.get(2),
                            );
                            let mut entries_ref = entries.borrow_mut();
                            if let Some(first_match) = entries_ref
                                .iter()
                                .position(|(entry_name, _)| entry_name == &name)
                            {
                                entries_ref[first_match].1 = value;
                                let mut index = entries_ref.len();
                                while index > 0 {
                                    index -= 1;
                                    if index != first_match && entries_ref[index].0 == name {
                                        entries_ref.remove(index);
                                    }
                                }
                            } else {
                                entries_ref.push((name, value));
                            }
                            Ok(Value::Undefined)
                        }
                        _ => unreachable!(),
                    };
                }
                Some(Value::Object(entries)) => {
                    if Self::is_cookie_store_object(&entries.borrow()) {
                        let mut evaluated_args = Vec::with_capacity(args.len());
                        for arg in args {
                            evaluated_args.push(self.eval_expr(arg, env, event_param, event)?);
                        }
                        let method_name = match method {
                            UrlSearchParamsInstanceMethod::GetAll => "getAll",
                            UrlSearchParamsInstanceMethod::Append => "append",
                            UrlSearchParamsInstanceMethod::Delete => "delete",
                            UrlSearchParamsInstanceMethod::Has => "has",
                            UrlSearchParamsInstanceMethod::Set
                            | UrlSearchParamsInstanceMethod::Sort => unreachable!(),
                        };
                        if let Some(value) = self.eval_cookie_store_member_call(
                            entries,
                            method_name,
                            &evaluated_args,
                        )? {
                            return Ok(value);
                        }
                    }
                    if !Self::is_url_search_params_object(&entries.borrow()) {
                        let mut evaluated_args = Vec::with_capacity(args.len());
                        for arg in args {
                            evaluated_args.push(self.eval_expr(arg, env, event_param, event)?);
                        }
                        let method_name = match method {
                            UrlSearchParamsInstanceMethod::GetAll => "getAll",
                            UrlSearchParamsInstanceMethod::Append => "append",
                            UrlSearchParamsInstanceMethod::Delete => "delete",
                            UrlSearchParamsInstanceMethod::Has => "has",
                            UrlSearchParamsInstanceMethod::Set => "set",
                            UrlSearchParamsInstanceMethod::Sort => unreachable!(),
                        };
                        if let Some(value) = self.eval_cache_storage_member_call(
                            entries,
                            method_name,
                            &evaluated_args,
                        )? {
                            return Ok(value);
                        }
                        if let Some(value) =
                            self.eval_cache_member_call(entries, method_name, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                        let message = if matches!(
                            method,
                            UrlSearchParamsInstanceMethod::Delete
                                | UrlSearchParamsInstanceMethod::Has
                        ) {
                            format!("variable '{}' is not a Map", target)
                        } else {
                            format!("variable '{}' is not a FormData instance", target)
                        };
                        return Err(Error::ScriptRuntime(message));
                    }
                }
                Some(_) => {
                    let message = if matches!(
                        method,
                        UrlSearchParamsInstanceMethod::Delete | UrlSearchParamsInstanceMethod::Has
                    ) {
                        format!("variable '{}' is not a Map", target)
                    } else {
                        format!("variable '{}' is not a FormData instance", target)
                    };
                    return Err(Error::ScriptRuntime(message));
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
        let evaluated_args = args
            .iter()
            .map(|arg| self.eval_expr(arg, env, event_param, event))
            .collect::<Result<Vec<_>>>()?;
        match method {
            UrlSearchParamsInstanceMethod::Append => {
                if evaluated_args.len() < 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.append requires exactly two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let value = evaluated_args[1].as_string();
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
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.delete requires one or two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let value = evaluated_args.get(1).map(Value::as_string);
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
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.getAll requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
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
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.has requires one or two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let value = evaluated_args.get(1).map(Value::as_string);
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
            UrlSearchParamsInstanceMethod::Set => {
                if evaluated_args.len() < 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.set requires exactly two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let value = evaluated_args[1].as_string();
                {
                    let mut object_ref = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&object_ref);
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
                    Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                }
                self.sync_url_search_params_owner(&object);
                Ok(Value::Undefined)
            }
            UrlSearchParamsInstanceMethod::Sort => {
                {
                    let mut object_ref = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&object_ref);
                    pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                    Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                }
                self.sync_url_search_params_owner(&object);
                Ok(Value::Undefined)
            }
        }
    }
}
