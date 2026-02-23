use super::*;

impl Harness {
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
                Ok(Value::Float(self.numeric_value(&value)))
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
            Expr::Delete(inner) => match inner.as_ref() {
                Expr::Var(name) => Ok(Value::Bool(!env.contains_key(name))),
                _ => {
                    self.eval_expr(inner, env, event_param, event)?;
                    Ok(Value::Bool(true))
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
                if let Value::Promise(promise) = value {
                    let settled = {
                        let promise = promise.borrow();
                        match &promise.state {
                            PromiseState::Pending => None,
                            PromiseState::Fulfilled(value) => {
                                Some(PromiseSettledValue::Fulfilled(value.clone()))
                            }
                            PromiseState::Rejected(reason) => {
                                Some(PromiseSettledValue::Rejected(reason.clone()))
                            }
                        }
                    };
                    match settled {
                        Some(PromiseSettledValue::Fulfilled(value)) => Ok(value),
                        Some(PromiseSettledValue::Rejected(reason)) => Err(Error::ScriptRuntime(
                            format!("await rejected Promise: {}", reason.as_string()),
                        )),
                        None => Ok(Value::Undefined),
                    }
                } else {
                    Ok(value)
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
                if let Some(yields) = self.script_runtime.generator_yield_stack.last() {
                    let values = self
                        .array_like_values_from_value(&value)
                        .unwrap_or_else(|_| vec![value.clone()]);
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
                Ok(value)
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
