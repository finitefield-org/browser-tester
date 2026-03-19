use super::*;

impl Harness {
    pub(crate) fn try_eval_array_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let result = match expr {
            Expr::ArrayConstruct { args, .. } => {
                let evaluated = self.eval_call_args_with_spread(args, env, event_param, event)?;
                if evaluated.is_empty() {
                    return Ok(Self::new_array_value(Vec::new()));
                }
                if evaluated.len() == 1 {
                    let first = &evaluated[0];
                    if let Some(length) = Self::array_constructor_length_from_value(first)? {
                        let mut out = Vec::new();
                        out.resize(length, Value::Undefined);
                        return Ok(Self::new_array_value(out));
                    }
                    return Ok(Self::new_array_value(vec![first.clone()]));
                }
                Ok(Self::new_array_value(evaluated))
            }
            Expr::ArrayLiteral(values) => {
                let mut out = Vec::with_capacity(values.len());
                for value in values {
                    match value {
                        Expr::Spread(expr) => {
                            let spread_value = self.eval_expr(expr, env, event_param, event)?;
                            out.extend(self.spread_iterable_values_from_value(&spread_value)?);
                        }
                        _ => out.push(self.eval_expr(value, env, event_param, event)?),
                    }
                }
                Ok(Self::new_array_value(out))
            }
            Expr::ArrayIsArray(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Bool(matches!(value, Value::Array(_))))
            }
            Expr::ArrayFrom { source, map_fn } => {
                let source = self.eval_expr(source, env, event_param, event)?;
                let values = self.array_like_values_from_value_with_live_properties(&source)?;
                if let Some(map_fn) = map_fn {
                    let callback = self.eval_expr(map_fn, env, event_param, event)?;
                    let mut mapped = Vec::with_capacity(values.len());
                    for (index, value) in values.into_iter().enumerate() {
                        mapped.push(self.execute_callback_value(
                            &callback,
                            &[value, Value::Number(index as i64)],
                            event,
                        )?);
                    }
                    return Ok(Self::new_array_value(mapped));
                }
                Ok(Self::new_array_value(values))
            }
            Expr::ArrayLength(target) => {
                match self.resolve_target_value_with_pending(env, target) {
                    Some(Value::Array(values)) => Ok(Value::Number(values.borrow().len() as i64)),
                    Some(Value::TypedArray(values)) => {
                        Ok(Value::Number(values.borrow().observed_length() as i64))
                    }
                    Some(Value::NodeList(nodes)) => {
                        let receiver = Value::NodeList(nodes.clone());
                        let own_override = {
                            let nodes_ref = nodes.borrow();
                            self.object_property_from_entries_with_getter(
                                &receiver,
                                &nodes_ref.properties,
                                "length",
                            )?
                        };
                        if let Some(value) = own_override {
                            Ok(value)
                        } else {
                            Ok(Value::Number(self.node_list_len(&nodes) as i64))
                        }
                    }
                    Some(Value::String(value)) => Ok(Value::Number(value.chars().count() as i64)),
                    Some(Value::Function(function)) => {
                        let function_value = Value::Function(function.clone());
                        if let Some(custom) = self
                            .function_public_property_from_entries_with_receiver(
                                &function,
                                "length",
                                &function_value,
                            )?
                        {
                            return Ok(custom);
                        }
                        if self
                            .script_runtime
                            .function_public_properties
                            .get(&function.function_id)
                            .is_some_and(|entries| {
                                Self::is_builtin_object_property_deleted(entries, "length")
                            })
                        {
                            return Ok(Value::Number(0));
                        }
                        let mut length = 0_i64;
                        for param in &function.handler.params {
                            if param.is_rest || param.default.is_some() {
                                break;
                            }
                            length += 1;
                        }
                        Ok(Value::Number(length))
                    }
                    Some(Value::Object(entries)) => {
                        let object = Value::Object(entries.clone());
                        let entries = entries.borrow();
                        if Self::is_history_object(&entries) {
                            return Ok(Self::object_get_entry(&entries, "length").unwrap_or(
                                Value::Number(self.location_history.history_entries.len() as i64),
                            ));
                        }
                        if Self::is_window_object(&entries) {
                            return Ok(Self::object_get_entry(&entries, "length")
                                .unwrap_or(Value::Number(0)));
                        }
                        if Self::is_storage_object(&entries) {
                            let len = Self::storage_pairs_from_object_entries(&entries).len();
                            return Ok(Value::Number(len as i64));
                        }
                        if let Some(value) = Self::string_wrapper_value_from_object(&entries) {
                            return Ok(Value::Number(value.chars().count() as i64));
                        }
                        drop(entries);
                        self.object_property_from_value(&object, "length")
                    }
                    Some(other) => self.object_property_from_value(&other, "length"),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                }
            }
            Expr::ArrayIndex { target, index } => {
                let index = self.eval_expr(index, env, event_param, event)?;
                let key = match &index {
                    Value::Number(value) => value.to_string(),
                    Value::BigInt(value) => value.to_string(),
                    Value::Float(value) if value.is_finite() && value.fract() == 0.0 => {
                        format!("{value:.0}")
                    }
                    other => self.property_key_to_storage_key(other),
                };
                if target == "super" {
                    let super_prototype = Self::super_prototype_from_env(env)?;
                    let this_value = Self::super_this_from_env(env)?;
                    return self.object_property_from_value_with_receiver(
                        &super_prototype,
                        &key,
                        &this_value,
                    );
                }
                match self.resolve_target_value_with_pending(env, target) {
                    Some(value) => self.object_property_from_value(&value, &key),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                }
            }
            Expr::ArrayPush { target, args } => {
                let values = self.resolve_array_from_env(env, target)?;
                let evaluated = self.eval_call_args_with_spread(args, env, event_param, event)?;
                let mut values = values.borrow_mut();
                values.extend(evaluated);
                Ok(Value::Number(values.len() as i64))
            }
            Expr::ArrayPop(target) => {
                let values = self.resolve_array_from_env(env, target)?;
                Ok(values.borrow_mut().pop().unwrap_or(Value::Undefined))
            }
            Expr::ArrayShift(target) => {
                let values = self.resolve_array_from_env(env, target)?;
                let mut values = values.borrow_mut();
                if values.is_empty() {
                    Ok(Value::Undefined)
                } else {
                    Ok(values.remove(0))
                }
            }
            Expr::ArrayUnshift { target, args } => {
                let values = self.resolve_array_from_env(env, target)?;
                let evaluated = self.eval_call_args_with_spread(args, env, event_param, event)?;
                let mut values = values.borrow_mut();
                for value in evaluated.into_iter().rev() {
                    values.insert(0, value);
                }
                Ok(Value::Number(values.len() as i64))
            }
            _ => match self.try_eval_array_higher_order_expr(expr, env, event_param, event) {
                Ok(result) => Ok(result),
                Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => {
                    self.try_eval_array_sequence_expr(expr, env, event_param, event)
                }
                Err(err) => Err(err),
            },
        }?;
        Ok(result)
    }
}
