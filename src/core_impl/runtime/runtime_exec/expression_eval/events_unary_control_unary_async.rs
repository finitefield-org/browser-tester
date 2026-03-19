use super::*;

impl Harness {
    fn typeof_name_for_value(&self, value: &Value) -> &'static str {
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
                if self.is_callable_value(value) {
                    "function"
                } else {
                    "object"
                }
            }
        }
    }

    pub(crate) fn try_eval_unary_and_async_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
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
                Expr::TypeOf(inner) => {
                    let js_type = match inner.as_ref() {
                        Expr::Var(name) => {
                            self.ensure_binding_initialized(env, name)?;
                            self.resolve_listener_capture_pending_value(name)
                                .flatten()
                                .or_else(|| env.get(name).cloned())
                                .or_else(|| self.resolve_pending_function_decl(name, env))
                                .or_else(|| self.resolve_runtime_global_identifier(name))
                                .as_ref()
                                .map_or("undefined", |value| self.typeof_name_for_value(value))
                        }
                        _ => {
                            let value = self.eval_expr(inner, env, event_param, event)?;
                            self.typeof_name_for_value(&value)
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
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
