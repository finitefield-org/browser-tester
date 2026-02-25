use super::*;

impl Harness {
    fn delete_property_from_value(&mut self, value: &Value, key: &str) -> Result<bool> {
        match value {
            Value::Null | Value::Undefined => {
                Err(Error::ScriptRuntime("value is not an object".into()))
            }
            Value::Object(entries) => {
                Self::delete_object_property_entries(&mut entries.borrow_mut(), key);
                Ok(true)
            }
            Value::Array(array) => {
                if let Ok(index) = key.parse::<usize>() {
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
                Self::delete_object_property_entries(&mut array.borrow_mut().properties, key);
                Ok(true)
            }
            Value::Map(map) => {
                Self::delete_object_property_entries(&mut map.borrow_mut().properties, key);
                Ok(true)
            }
            Value::WeakMap(weak_map) => {
                Self::delete_object_property_entries(&mut weak_map.borrow_mut().properties, key);
                Ok(true)
            }
            Value::Set(set) => {
                Self::delete_object_property_entries(&mut set.borrow_mut().properties, key);
                Ok(true)
            }
            Value::WeakSet(weak_set) => {
                Self::delete_object_property_entries(&mut weak_set.borrow_mut().properties, key);
                Ok(true)
            }
            Value::RegExp(regex) => {
                Self::delete_object_property_entries(&mut regex.borrow_mut().properties, key);
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
                            EventExprProp::Type => Value::String(event.event_type.clone()),
                            EventExprProp::Target => Value::Node(event.target),
                            EventExprProp::CurrentTarget => Value::Node(event.current_target),
                            EventExprProp::TargetName => Value::String(
                                self.dom.attr(event.target, "name").unwrap_or_default(),
                            ),
                            EventExprProp::CurrentTargetName => Value::String(
                                self.dom
                                    .attr(event.current_target, "name")
                                    .unwrap_or_default(),
                            ),
                            EventExprProp::DefaultPrevented => Value::Bool(event.default_prevented),
                            EventExprProp::IsTrusted => Value::Bool(event.is_trusted),
                            EventExprProp::Bubbles => Value::Bool(event.bubbles),
                            EventExprProp::Cancelable => Value::Bool(event.cancelable),
                            EventExprProp::TargetId => {
                                Value::String(self.dom.attr(event.target, "id").unwrap_or_default())
                            }
                            EventExprProp::CurrentTargetId => Value::String(
                                self.dom
                                    .attr(event.current_target, "id")
                                    .unwrap_or_default(),
                            ),
                            EventExprProp::EventPhase => Value::Number(event.event_phase as i64),
                            EventExprProp::TimeStamp => Value::Number(event.time_stamp_ms),
                            EventExprProp::State => {
                                event.state.as_ref().cloned().unwrap_or(Value::Undefined)
                            }
                            EventExprProp::OldState => event
                                .old_state
                                .as_ref()
                                .map(|value| Value::String(value.clone()))
                                .unwrap_or(Value::Undefined),
                            EventExprProp::NewState => event
                                .new_state
                                .as_ref()
                                .map(|value| Value::String(value.clone()))
                                .unwrap_or(Value::Undefined),
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
                        env.get(name)
                            .cloned()
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
                            | Value::SymbolConstructor
                            | Value::RegExpConstructor
                            | Value::PromiseCapability(_) => "function",
                            Value::Function(_) => "function",
                            Value::Node(_)
                            | Value::NodeList(_)
                            | Value::FormData(_)
                            | Value::Array(_)
                            | Value::Object(_)
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
                            | Value::SymbolConstructor
                            | Value::RegExpConstructor
                            | Value::PromiseCapability(_) => "function",
                            Value::Function(_) => "function",
                            Value::Node(_)
                            | Value::NodeList(_)
                            | Value::FormData(_)
                            | Value::Array(_)
                            | Value::Object(_)
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
