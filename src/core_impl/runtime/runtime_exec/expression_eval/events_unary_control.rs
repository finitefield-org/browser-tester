use super::*;

impl Harness {
    fn live_event_object_from_env(
        env: &HashMap<String, Value>,
        event_var: &str,
    ) -> Option<Rc<RefCell<ObjectValue>>> {
        let Value::Object(entries) = env.get(event_var)?.clone() else {
            return None;
        };
        if !Self::is_event_object(&entries.borrow()) {
            return None;
        }
        Some(entries)
    }

    fn live_event_property_from_env(
        &mut self,
        env: &HashMap<String, Value>,
        event_var: &str,
        key: &str,
    ) -> Result<Option<Value>> {
        let Some(entries) = Self::live_event_object_from_env(env, event_var) else {
            return Ok(None);
        };
        self.object_property_from_value(&Value::Object(entries), key)
            .map(Some)
    }

    fn live_event_nested_property_from_env(
        &mut self,
        env: &HashMap<String, Value>,
        event_var: &str,
        key: &str,
        nested: &str,
    ) -> Result<Option<Value>> {
        let Some(value) = self.live_event_property_from_env(env, event_var, key)? else {
            return Ok(None);
        };
        self.object_property_from_value(&value, nested).map(Some)
    }

    fn delete_property_from_value(&mut self, value: &Value, key: &str) -> Result<bool> {
        match value {
            Value::Null | Value::Undefined => {
                Err(Error::ScriptRuntime("value is not an object".into()))
            }
            Value::Object(entries) => {
                let (is_string_wrapper_builtin, placeholder_builtin_surface, owner) = {
                    let entries_ref = entries.borrow();
                    (
                        Self::string_wrapper_builtin_has_own_property(&entries_ref, key),
                        Self::placeholder_backed_object_builtin_surface_exists(&entries_ref, key),
                        if !Self::is_symbol_storage_key(key)
                            && Self::is_dom_string_map_object(&entries_ref)
                        {
                            Self::dom_string_map_owner_node(&entries_ref)
                        } else {
                            None
                        },
                    )
                };
                if is_string_wrapper_builtin {
                    return Ok(false);
                }
                if let Some(owner) = owner {
                    self.dom.dataset_delete(owner, key)?;
                }
                if !Self::is_configurable_object_key(&*entries.borrow(), key) {
                    return Ok(false);
                }
                let delete_builtin_surface =
                    Self::is_callable_own_surface_key(key) || placeholder_builtin_surface;
                let mut entries = entries.borrow_mut();
                Self::delete_object_property_entries(&mut entries, key);
                if delete_builtin_surface {
                    Self::mark_builtin_object_property_deleted(&mut entries, key);
                }
                Ok(true)
            }
            Value::Array(array) => {
                if key == "length" {
                    return Ok(false);
                }
                let placeholder_builtin_surface = {
                    let array_ref = array.borrow();
                    Self::placeholder_backed_array_builtin_surface_exists(&array_ref, key)
                };
                if let Ok(index) = key.parse::<usize>() {
                    if !Self::is_configurable_object_key(&array.borrow().properties, key) {
                        return Ok(false);
                    }
                    let has_index = {
                        let mut values = array.borrow_mut();
                        let has_index = index < values.len();
                        if has_index {
                            values[index] = Value::Undefined;
                        }
                        has_index
                    };
                    if has_index {
                        Self::mark_array_hole(array, index);
                    }
                    return Ok(true);
                }
                if !Self::is_configurable_object_key(&array.borrow().properties, key) {
                    return Ok(false);
                }
                let mut array = array.borrow_mut();
                Self::delete_object_property_entries(&mut array.properties, key);
                if placeholder_builtin_surface {
                    Self::mark_builtin_object_property_deleted(&mut array.properties, key);
                }
                Ok(true)
            }
            Value::Map(map) => {
                if !Self::is_configurable_object_key(&map.borrow().properties, key) {
                    return Ok(false);
                }
                let delete_builtin_surface = key == "size";
                let mut map = map.borrow_mut();
                Self::delete_object_property_entries(&mut map.properties, key);
                if delete_builtin_surface {
                    Self::mark_builtin_object_property_deleted(&mut map.properties, key);
                }
                Ok(true)
            }
            Value::WeakMap(weak_map) => {
                if !Self::is_configurable_object_key(&weak_map.borrow().properties, key) {
                    return Ok(false);
                }
                Self::delete_object_property_entries(&mut weak_map.borrow_mut().properties, key);
                Ok(true)
            }
            Value::Set(set) => {
                if !Self::is_configurable_object_key(&set.borrow().properties, key) {
                    return Ok(false);
                }
                let delete_builtin_surface = key == "size";
                let mut set = set.borrow_mut();
                Self::delete_object_property_entries(&mut set.properties, key);
                if delete_builtin_surface {
                    Self::mark_builtin_object_property_deleted(&mut set.properties, key);
                }
                Ok(true)
            }
            Value::WeakSet(weak_set) => {
                if !Self::is_configurable_object_key(&weak_set.borrow().properties, key) {
                    return Ok(false);
                }
                Self::delete_object_property_entries(&mut weak_set.borrow_mut().properties, key);
                Ok(true)
            }
            Value::RegExp(regex) => {
                if key == "lastIndex" {
                    return Ok(false);
                }
                let has_own_surface = {
                    let regex_ref = regex.borrow();
                    Self::object_get_entry(&regex_ref.properties, key).is_some()
                        || Self::has_object_accessor_property(&regex_ref.properties, key)
                };
                if !has_own_surface {
                    return Ok(true);
                }
                if !Self::is_configurable_object_key(&regex.borrow().properties, key) {
                    return Ok(false);
                }
                let mut regex = regex.borrow_mut();
                Self::delete_object_property_entries(&mut regex.properties, key);
                Ok(true)
            }
            Value::Function(function) => {
                let delete_builtin_surface = Self::is_callable_own_surface_key(key);
                if Self::is_function_builtin_prototype_key(function, key) {
                    return Ok(false);
                }
                if let Some(entries) = self
                    .script_runtime
                    .function_public_properties
                    .get_mut(&function.function_id)
                {
                    if !Self::is_configurable_object_key(entries, key) {
                        return Ok(false);
                    }
                    Self::delete_object_property_entries(entries, key);
                    if delete_builtin_surface {
                        Self::mark_builtin_object_property_deleted(entries, key);
                    }
                    return Ok(true);
                }
                if delete_builtin_surface {
                    let entries = self
                        .script_runtime
                        .function_public_properties
                        .entry(function.function_id)
                        .or_default();
                    Self::mark_builtin_object_property_deleted(entries, key);
                    return Ok(true);
                }
                Ok(true)
            }
            Value::Node(node) => {
                self.dom_runtime
                    .node_expando_props
                    .remove(&(*node, key.to_string()));
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    fn delete_member_expr_property(
        &mut self,
        receiver: &Value,
        key: String,
        optional: bool,
    ) -> Result<Value> {
        if optional && matches!(receiver, Value::Null | Value::Undefined) {
            return Ok(Value::Bool(true));
        }
        let deleted = self.delete_property_from_value(receiver, &key)?;
        Ok(Value::Bool(deleted))
    }

    fn delete_event_prop_from_value(
        &mut self,
        event_var: &str,
        value: &Value,
        prop: EventExprProp,
    ) -> Result<Value> {
        let deleted = match prop {
            EventExprProp::Type => self.delete_property_from_value(value, "type")?,
            EventExprProp::Target => self.delete_property_from_value(value, "target")?,
            EventExprProp::CurrentTarget => {
                self.delete_property_from_value(value, "currentTarget")?
            }
            EventExprProp::DefaultPrevented => {
                self.delete_property_from_value(value, "defaultPrevented")?
            }
            EventExprProp::IsTrusted => self.delete_property_from_value(value, "isTrusted")?,
            EventExprProp::Bubbles => self.delete_property_from_value(value, "bubbles")?,
            EventExprProp::Cancelable => self.delete_property_from_value(value, "cancelable")?,
            EventExprProp::EventPhase => self.delete_property_from_value(value, "eventPhase")?,
            EventExprProp::TimeStamp => self.delete_property_from_value(value, "timeStamp")?,
            EventExprProp::State => self.delete_property_from_value(value, "state")?,
            EventExprProp::OldState => self.delete_property_from_value(value, "oldState")?,
            EventExprProp::NewState => self.delete_property_from_value(value, "newState")?,
            EventExprProp::TargetName => {
                let target = self.object_property_from_named_value(event_var, value, "target")?;
                self.delete_property_from_value(&target, "name")?
            }
            EventExprProp::CurrentTargetName => {
                let current_target =
                    self.object_property_from_named_value(event_var, value, "currentTarget")?;
                self.delete_property_from_value(&current_target, "name")?
            }
            EventExprProp::TargetId => {
                let target = self.object_property_from_named_value(event_var, value, "target")?;
                self.delete_property_from_value(&target, "id")?
            }
            EventExprProp::CurrentTargetId => {
                let current_target =
                    self.object_property_from_named_value(event_var, value, "currentTarget")?;
                self.delete_property_from_value(&current_target, "id")?
            }
        };
        Ok(Value::Bool(deleted))
    }

    fn delete_non_node_dom_prop_fallback(
        &mut self,
        target: &DomQuery,
        prop: &DomProp,
        env: &HashMap<String, Value>,
    ) -> Result<Option<Value>> {
        let Some(value) = self.resolve_dom_query_value_runtime(target, env)? else {
            return Ok(None);
        };
        if matches!(value, Value::Node(_) | Value::NodeList(_)) {
            return Ok(None);
        }
        let Some(path) = Self::dom_prop_non_node_fallback_path(prop) else {
            return Ok(None);
        };
        if path.is_empty() {
            return Ok(Some(Value::Bool(true)));
        }

        let variable_name = target.describe_call();
        let mut current = value;
        for key in path.iter().take(path.len().saturating_sub(1)) {
            current = self.object_property_from_named_value(&variable_name, &current, key)?;
        }
        let final_key = path
            .last()
            .ok_or_else(|| Error::ScriptRuntime("dom fallback path cannot be empty".into()))?;
        Ok(Some(Value::Bool(
            self.delete_property_from_value(&current, final_key)?,
        )))
    }

    pub(crate) fn eval_expr_events_unary_control(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
            Expr::EventProp { event_var, prop } => {
                if let Some(param) = event_param {
                        if param == event_var {
                            let value = match prop {
                            EventExprProp::Type => self
                                .live_event_property_from_env(env, event_var, "type")?
                                .unwrap_or(Value::String(event.event_type.clone())),
                            EventExprProp::Target => self
                                .live_event_property_from_env(env, event_var, "target")?
                                .unwrap_or_else(|| {
                                    event
                                        .target_value
                                        .as_ref()
                                        .cloned()
                                        .unwrap_or(Value::Node(event.target))
                                }),
                            EventExprProp::CurrentTarget => self
                                .live_event_property_from_env(env, event_var, "currentTarget")?
                                .unwrap_or_else(|| {
                                    event
                                        .current_target_value
                                        .as_ref()
                                        .cloned()
                                        .unwrap_or(Value::Node(event.current_target))
                                }),
                            EventExprProp::TargetName => {
                                if let Some(value) = self
                                    .live_event_nested_property_from_env(
                                        env,
                                        event_var,
                                        "target",
                                        "name",
                                    )?
                                {
                                    value
                                } else if let Some(value) = event.target_value.as_ref() {
                                    self.object_property_from_value(value, "name")
                                        .unwrap_or(Value::Undefined)
                                } else {
                                    Value::String(self.dom.attr(event.target, "name").unwrap_or_default())
                                }
                            }
                            EventExprProp::CurrentTargetName => {
                                if let Some(value) = self
                                    .live_event_nested_property_from_env(
                                        env,
                                        event_var,
                                        "currentTarget",
                                        "name",
                                    )?
                                {
                                    value
                                } else if let Some(value) = event.current_target_value.as_ref() {
                                    self.object_property_from_value(value, "name")
                                        .unwrap_or(Value::Undefined)
                                } else {
                                    Value::String(
                                        self.dom
                                            .attr(event.current_target, "name")
                                            .unwrap_or_default(),
                                    )
                                }
                            }
                            EventExprProp::DefaultPrevented => self
                                .live_event_property_from_env(env, event_var, "defaultPrevented")?
                                .unwrap_or(Value::Bool(event.default_prevented)),
                            EventExprProp::IsTrusted => self
                                .live_event_property_from_env(env, event_var, "isTrusted")?
                                .unwrap_or(Value::Bool(event.is_trusted)),
                            EventExprProp::Bubbles => self
                                .live_event_property_from_env(env, event_var, "bubbles")?
                                .unwrap_or(Value::Bool(event.bubbles)),
                            EventExprProp::Cancelable => self
                                .live_event_property_from_env(env, event_var, "cancelable")?
                                .unwrap_or(Value::Bool(event.cancelable)),
                            EventExprProp::TargetId => {
                                if let Some(value) = self
                                    .live_event_nested_property_from_env(env, event_var, "target", "id")?
                                {
                                    value
                                } else if let Some(value) = event.target_value.as_ref() {
                                    self.object_property_from_value(value, "id")
                                        .unwrap_or(Value::Undefined)
                                } else {
                                    Value::String(self.dom.attr(event.target, "id").unwrap_or_default())
                                }
                            }
                            EventExprProp::CurrentTargetId => {
                                if let Some(value) = self.live_event_nested_property_from_env(
                                    env,
                                    event_var,
                                    "currentTarget",
                                    "id",
                                )? {
                                    value
                                } else if let Some(value) = event.current_target_value.as_ref() {
                                    self.object_property_from_value(value, "id")
                                        .unwrap_or(Value::Undefined)
                                } else {
                                    Value::String(
                                        self.dom
                                            .attr(event.current_target, "id")
                                            .unwrap_or_default(),
                                    )
                                }
                            }
                            EventExprProp::EventPhase => self
                                .live_event_property_from_env(env, event_var, "eventPhase")?
                                .unwrap_or(Value::Number(event.event_phase as i64)),
                            EventExprProp::TimeStamp => self
                                .live_event_property_from_env(env, event_var, "timeStamp")?
                                .unwrap_or(Value::Number(event.time_stamp_ms)),
                            EventExprProp::State => self
                                .live_event_property_from_env(env, event_var, "state")?
                                .unwrap_or_else(|| {
                                    event.state.as_ref().cloned().unwrap_or(Value::Undefined)
                                }),
                            EventExprProp::OldState => self
                                .live_event_property_from_env(env, event_var, "oldState")?
                                .unwrap_or_else(|| {
                                    event
                                        .old_state
                                        .as_ref()
                                        .map(|value| Value::String(value.clone()))
                                        .unwrap_or(Value::Undefined)
                                }),
                            EventExprProp::NewState => self
                                .live_event_property_from_env(env, event_var, "newState")?
                                .unwrap_or_else(|| {
                                    event
                                        .new_state
                                        .as_ref()
                                        .map(|value| Value::String(value.clone()))
                                        .unwrap_or(Value::Undefined)
                                }),
                        };
                        return Ok(value);
                    }
                }

                if let Some(value) = env.get(event_var) {
                    return self.eval_event_prop_fallback(event_var, value, *prop);
                }

                if event_param.is_none() {
                    return Err(Error::ScriptRuntime(format!(
                        "event variable '{}' is not available in this handler",
                        event_var
                    )));
                }
                Err(Error::ScriptRuntime(format!(
                    "unknown event variable: {}",
                    event_var
                )))
            }
            Expr::Neg(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                if matches!(value, Value::Symbol(_)) {
                    return Err(Error::ScriptRuntime(
                        "Cannot convert a Symbol value to a number".into(),
                    ));
                }
                match value {
                    Value::Number(v) => Ok(Value::Number(-v)),
                    Value::Float(v) => Ok(Value::Float(-v)),
                    Value::BigInt(v) => Ok(Value::BigInt(-v)),
                    other => Ok(Value::Float(-self.numeric_value(&other))),
                }
            }
            Expr::Pos(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                if matches!(value, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "unary plus is not supported for BigInt values".into(),
                    ));
                }
                if matches!(value, Value::Symbol(_)) {
                    return Err(Error::ScriptRuntime(
                        "Cannot convert a Symbol value to a number".into(),
                    ));
                }
                Ok(Self::number_value(Self::coerce_number_for_global(&value)))
            }
            Expr::BitNot(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                if matches!(value, Value::Symbol(_)) {
                    return Err(Error::ScriptRuntime(
                        "Cannot convert a Symbol value to a number".into(),
                    ));
                }
                if let Value::BigInt(v) = value {
                    return Ok(Value::BigInt(!v));
                }
                Ok(Value::Number((!self.to_i32_for_bitwise(&value)) as i64))
            }
            Expr::Not(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                Ok(Value::Bool(!value.truthy()))
            }
            Expr::Void(inner) => {
                self.eval_expr(inner, env, event_param, event)?;
                Ok(Value::Undefined)
            }
            Expr::Delete(inner) => {
                match inner.as_ref() {
                Expr::Var(name) => Ok(Value::Bool(!env.contains_key(name))),
                Expr::ObjectGet { target, key } => {
                    if target == "super" {
                        return Err(Error::ScriptRuntime(
                            "Cannot delete super property".into(),
                        ));
                    }
                    let value = env.get(target).cloned().ok_or_else(|| {
                        Error::ScriptRuntime(format!("unknown variable: {}", target))
                    })?;
                    let deleted = self.delete_property_from_value(&value, key)?;
                    Ok(Value::Bool(deleted))
                }
                Expr::ArrayIndex { target, index } => {
                    if target == "super" {
                        return Err(Error::ScriptRuntime(
                            "Cannot delete super property".into(),
                        ));
                    }
                    let value = env.get(target).cloned().ok_or_else(|| {
                        Error::ScriptRuntime(format!("unknown variable: {}", target))
                    })?;
                    let index_value = self.eval_expr(index, env, event_param, event)?;
                    let key = self.property_key_to_storage_key(&index_value);
                    let deleted = self.delete_property_from_value(&value, &key)?;
                    Ok(Value::Bool(deleted))
                }
                Expr::ArrayLength(target) => {
                    if target == "super" {
                        return Err(Error::ScriptRuntime(
                            "Cannot delete super property".into(),
                        ));
                    }
                    let value = env.get(target).cloned().ok_or_else(|| {
                        Error::ScriptRuntime(format!("unknown variable: {}", target))
                    })?;
                    let deleted = self.delete_property_from_value(&value, "length")?;
                    Ok(Value::Bool(deleted))
                }
                Expr::ObjectPathGet { target, path } => {
                    if target == "super" {
                        return Err(Error::ScriptRuntime(
                            "Cannot delete super property".into(),
                        ));
                    }
                    let Some(mut receiver) = env.get(target).cloned() else {
                        return Err(Error::ScriptRuntime(format!("unknown variable: {}", target)));
                    };
                    if path.is_empty() {
                        return Ok(Value::Bool(true));
                    }
                    for key in path.iter().take(path.len().saturating_sub(1)) {
                        receiver = self.object_property_from_value(&receiver, key)?;
                    }
                    let final_key = path
                        .last()
                        .ok_or_else(|| Error::ScriptRuntime("object path cannot be empty".into()))?;
                    let deleted = self.delete_property_from_value(&receiver, final_key)?;
                    Ok(Value::Bool(deleted))
                }
                Expr::EventProp { event_var, prop } => {
                    if let Some(value) = env.get(event_var) {
                        self.delete_event_prop_from_value(event_var, value, *prop)
                    } else if event_param.is_none() {
                        Err(Error::ScriptRuntime(format!(
                            "event variable '{}' is not available in this handler",
                            event_var
                        )))
                    } else {
                        Err(Error::ScriptRuntime(format!(
                            "unknown event variable: {}",
                            event_var
                        )))
                    }
                }
                Expr::DomRead { target, prop } => {
                    if let Some(value) = self.delete_non_node_dom_prop_fallback(target, prop, env)? {
                        Ok(value)
                    } else {
                        self.eval_expr(inner, env, event_param, event)?;
                        Ok(Value::Bool(true))
                    }
                }
                Expr::MemberGet {
                    target,
                    member,
                    optional,
                } => {
                    if matches!(target.as_ref(), Expr::Var(name) if name == "super") {
                        return Err(Error::ScriptRuntime(
                            "Cannot delete super property".into(),
                        ));
                    }
                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    self.delete_member_expr_property(&receiver, member.clone(), *optional)
                }
                Expr::IndexGet {
                    target,
                    index,
                    optional,
                } => {
                    if matches!(target.as_ref(), Expr::Var(name) if name == "super") {
                        return Err(Error::ScriptRuntime(
                            "Cannot delete super property".into(),
                        ));
                    }
                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    let index_value = self.eval_expr(index, env, event_param, event)?;
                    let key = match index_value {
                        Value::Number(value) => value.to_string(),
                        Value::BigInt(value) => value.to_string(),
                        Value::Float(value) if value.is_finite() && value.fract() == 0.0 => {
                            format!("{:.0}", value)
                        }
                        other => self.property_key_to_storage_key(&other),
                    };
                    self.delete_member_expr_property(&receiver, key, *optional)
                }
                _ => {
                    self.eval_expr(inner, env, event_param, event)?;
                    Ok(Value::Bool(true))
                }
            }
            },
            Expr::TypeOf(inner) => {
                let js_type = match inner.as_ref() {
                    Expr::Var(name) => {
                        self.ensure_binding_initialized(env, name)?;
                        self.resolve_listener_capture_pending_value(name)
                            .flatten()
                            .or_else(|| env.get(name).cloned())
                            .or_else(|| self.resolve_pending_function_decl(name, env))
                            .as_ref()
                            .map_or("undefined", |value| match value {
                            Value::Null => "object",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) | Value::Float(_) => "number",
                            Value::BigInt(_) => "bigint",
                            Value::Symbol(_) => "symbol",
                            Value::Undefined => "undefined",
                            Value::String(_) => "string",
                            Value::StringConstructor => "function",
                            Value::TypedArrayConstructor(_)
                            | Value::BlobConstructor
                            | Value::UrlConstructor
                            | Value::ArrayBufferConstructor
                            | Value::PromiseConstructor
                            | Value::MapConstructor
                            | Value::WeakMapConstructor
                            | Value::SetConstructor
                            | Value::WeakSetConstructor
                            | Value::UrlSearchParamsConstructor
                            | Value::SymbolConstructor
                            | Value::RegExpConstructor
                            | Value::PromiseCapability(_) => "function",
                            Value::Function(_) => "function",
                            Value::Node(_)
                            | Value::NodeList(_)
                            | Value::FormData(_)
                            | Value::Array(_)
                            | Value::Map(_)
                            | Value::WeakMap(_)
                            | Value::Set(_)
                            | Value::WeakSet(_)
                            | Value::Blob(_)
                            | Value::Promise(_)
                            | Value::ArrayBuffer(_)
                            | Value::TypedArray(_)
                            | Value::RegExp(_)
                            | Value::Date(_) => "object",
                            Value::Object(_) => {
                                if self.is_callable_value(&value) {
                                    "function"
                                } else {
                                    "object"
                                }
                            }
                        })
                    }
                    _ => {
                        let value = self.eval_expr(inner, env, event_param, event)?;
                        match value {
                            Value::Null => "object",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) | Value::Float(_) => "number",
                            Value::BigInt(_) => "bigint",
                            Value::Symbol(_) => "symbol",
                            Value::Undefined => "undefined",
                            Value::String(_) => "string",
                            Value::StringConstructor => "function",
                            Value::TypedArrayConstructor(_)
                            | Value::BlobConstructor
                            | Value::UrlConstructor
                            | Value::ArrayBufferConstructor
                            | Value::PromiseConstructor
                            | Value::MapConstructor
                            | Value::WeakMapConstructor
                            | Value::SetConstructor
                            | Value::WeakSetConstructor
                            | Value::UrlSearchParamsConstructor
                            | Value::SymbolConstructor
                            | Value::RegExpConstructor
                            | Value::PromiseCapability(_) => "function",
                            Value::Function(_) => "function",
                            Value::Node(_)
                            | Value::NodeList(_)
                            | Value::FormData(_)
                            | Value::Array(_)
                            | Value::Map(_)
                            | Value::WeakMap(_)
                            | Value::Set(_)
                            | Value::WeakSet(_)
                            | Value::Blob(_)
                            | Value::Promise(_)
                            | Value::ArrayBuffer(_)
                            | Value::TypedArray(_)
                            | Value::RegExp(_)
                            | Value::Date(_) => "object",
                            Value::Object(_) => {
                                if self.is_callable_value(&value) {
                                    "function"
                                } else {
                                    "object"
                                }
                            }
                        }
                    }
                };
                Ok(Value::String(js_type.to_string()))
            }
            Expr::Await(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                let promise = self.promise_resolve_value_as_promise(value)?;
                loop {
                    let settled = {
                        let promise = promise.borrow();
                        match &promise.state {
                            PromiseState::Pending => None,
                            PromiseState::Fulfilled(value) => Some(Ok(value.clone())),
                            PromiseState::Rejected(reason) => Some(Err(reason.clone())),
                        }
                    };
                    match settled {
                        Some(Ok(value)) => return Ok(value),
                        Some(Err(reason)) => {
                            return Err(Error::ScriptThrown(ThrownValue::new(reason)));
                        }
                        None => {
                            if !self.scheduler.microtask_queue.is_empty() {
                                self.run_microtask_queue()?;
                                continue;
                            }
                            let ran_timers = self.run_due_timers_internal()?;
                            if ran_timers == 0 {
                                return Ok(Value::Undefined);
                            }
                        }
                    }
                }
            }
            Expr::Yield(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                if let Some(yields) = self.script_runtime.generator_yield_stack.last() {
                    let mut yields = yields.borrow_mut();
                    yields.push(value.clone());
                    if yields.len() >= GENERATOR_MAX_BUFFERED_YIELDS {
                        return Err(Error::ScriptRuntime(
                            INTERNAL_GENERATOR_YIELD_LIMIT_REACHED.into(),
                        ));
                    }
                }
                Ok(value)
            }
            Expr::YieldStar(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                let values = self.array_like_values_from_value(&value)?;
                if let Some(yields) = self.script_runtime.generator_yield_stack.last() {
                    let mut yields = yields.borrow_mut();
                    for item in values {
                        yields.push(item);
                        if yields.len() >= GENERATOR_MAX_BUFFERED_YIELDS {
                            return Err(Error::ScriptRuntime(
                                INTERNAL_GENERATOR_YIELD_LIMIT_REACHED.into(),
                            ));
                        }
                    }
                }
                let completion = match &value {
                    Value::Object(entries) => {
                        let entries = entries.borrow();
                        if Self::is_iterator_object(&entries) {
                            Self::object_get_entry(&entries, INTERNAL_ITERATOR_RETURN_VALUE_KEY)
                                .unwrap_or(Value::Undefined)
                        } else {
                            Value::Undefined
                        }
                    }
                    _ => Value::Undefined,
                };
                Ok(completion)
            }
            Expr::Comma(parts) => {
                let mut last = Value::Undefined;
                for part in parts {
                    last = self.eval_expr(part, env, event_param, event)?;
                }
                Ok(last)
            }
            Expr::Spread(_) => Err(Error::ScriptRuntime(
                "spread syntax is only supported in array literals, object literals, and call arguments".into(),
            )),
            Expr::Add(parts) => {
                if parts.is_empty() {
                    return Ok(Value::String(String::new()));
                }
                let mut iter = parts.iter();
                let first = iter
                    .next()
                    .ok_or_else(|| Error::ScriptRuntime("empty add expression".into()))?;
                let mut acc = self.eval_expr(first, env, event_param, event)?;
                for part in iter {
                    let rhs = self.eval_expr(part, env, event_param, event)?;
                    acc = self.add_values(&acc, &rhs)?;
                }
                Ok(acc)
            }
            Expr::Ternary {
                cond,
                on_true,
                on_false,
            } => {
                let cond = self.eval_expr(cond, env, event_param, event)?;
                if cond.truthy() {
                    self.eval_expr(on_true, env, event_param, event)
                } else {
                    self.eval_expr(on_false, env, event_param, event)
                }
            }
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
