use super::*;

impl Harness {
    pub(crate) fn execute_receiver_builtin_callable(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
        this_arg: Option<Value>,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        let (family, member) = Self::receiver_builtin_callable_components(callable)?;
        let receiver = this_arg.ok_or_else(|| Self::incompatible_receiver_error(&family))?;
        if let Some(value) =
            self.execute_receiver_builtin_dom_family(&family, &member, &receiver, args)?
        {
            return Ok(value);
        }
        if let Some(value) =
            self.execute_receiver_builtin_intl_family(&family, &member, &receiver, args)?
        {
            return Ok(value);
        }
        if let Some(value) = self.execute_receiver_builtin_webapi_family(
            &family, &member, &receiver, args, event, caller_env,
        )? {
            return Ok(value);
        }
        match family.as_str() {
            "array" => {
                let Value::Array(values) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_array_member_call(&values, &member, args, event, caller_env)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported Array method: {member}"))
                    })
            }
            "date" => {
                let Value::Date(value) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_date_member_call(&value, &member, args)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported Date method: {member}"))
                    })
            }
            "map" => {
                let Value::Map(map) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_map_member_call_from_values(&map, &member, args, event)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported Map method: {member}"))
                    })
            }
            "weak_map" => {
                let Value::WeakMap(weak_map) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_weak_map_member_call_from_values(&weak_map, &member, args, event)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported WeakMap method: {member}"))
                    })
            }
            "set" => {
                let Value::Set(set) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_set_member_call_from_values(&set, &member, args, event)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported Set method: {member}"))
                    })
            }
            "weak_set" => {
                let Value::WeakSet(weak_set) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_weak_set_member_call_from_values(&weak_set, &member, args)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported WeakSet method: {member}"))
                    })
            }
            "string" => {
                let text = match member.as_str() {
                    "toString" | "valueOf" => match receiver {
                        Value::String(text) => text,
                        Value::Object(object) => {
                            let entries = object.borrow();
                            Self::string_wrapper_value_from_object(&entries)
                                .ok_or_else(|| Self::incompatible_receiver_error(&family))?
                        }
                        _ => return Err(Self::incompatible_receiver_error(&family)),
                    },
                    _ => self.coerce_string_method_receiver(&receiver)?,
                };
                match member.as_str() {
                    "toString" | "valueOf" => Ok(Value::String(text)),
                    _ => self
                        .eval_string_member_call(&text, &member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported String method: {member}"))
                        }),
                }
            }
            "node" => {
                let Value::Node(node) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_node_member_call(node, &member, args, event)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported Node method: {member}"))
                    })
            }
            "node_list" | "html_collection" => {
                let Value::NodeList(nodes) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                if family == "html_collection" && !Self::node_list_is_html_collection(&nodes) {
                    return Err(Self::incompatible_receiver_error(&family));
                }
                if family == "node_list" && Self::node_list_is_html_collection(&nodes) {
                    return Err(Self::incompatible_receiver_error(&family));
                }
                self.eval_nodelist_member_call(&nodes, &member, args, event)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!(
                            "unsupported {} method: {member}",
                            if family == "html_collection" {
                                "HTMLCollection"
                            } else {
                                "NodeList"
                            }
                        ))
                    })
            }
            "typed_array" => {
                let Value::TypedArray(array) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_typed_array_member_call(&array, &member, args, event, caller_env)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported TypedArray method: {member}"))
                    })
            }
            "boolean" => {
                let value = match receiver {
                    Value::Bool(value) => value,
                    Value::Object(object) => {
                        let entries = object.borrow();
                        Self::boolean_wrapper_value_from_object(&entries)
                            .ok_or_else(|| Self::incompatible_receiver_error(&family))?
                    }
                    _ => return Err(Self::incompatible_receiver_error(&family)),
                };
                match member.as_str() {
                    "toString" => Ok(Value::String(if value {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    })),
                    "valueOf" => Ok(Value::Bool(value)),
                    _ => Err(Error::ScriptRuntime(format!(
                        "unsupported Boolean method: {member}"
                    ))),
                }
            }
            "number" => {
                match receiver {
                    Value::Number(_) | Value::Float(_) => {}
                    Value::Object(ref object) => {
                        let entries = object.borrow();
                        Self::number_wrapper_value_from_object(&entries)
                            .ok_or_else(|| Self::incompatible_receiver_error(&family))?;
                    }
                    _ => return Err(Self::incompatible_receiver_error(&family)),
                }
                let method = match member.as_str() {
                    "toExponential" => NumberInstanceMethod::ToExponential,
                    "toFixed" => NumberInstanceMethod::ToFixed,
                    "toLocaleString" => NumberInstanceMethod::ToLocaleString,
                    "toPrecision" => NumberInstanceMethod::ToPrecision,
                    "toString" => NumberInstanceMethod::ToString,
                    "valueOf" => NumberInstanceMethod::ValueOf,
                    _ => Err(Error::ScriptRuntime(format!(
                        "unsupported Number method: {member}"
                    )))?,
                };
                self.eval_number_instance_method_from_values(method, &receiver, args)
            }
            "bigint" => {
                let value = match receiver {
                    Value::BigInt(value) => value,
                    Value::Object(object) => {
                        let entries = object.borrow();
                        Self::bigint_wrapper_value_from_object(&entries)
                            .ok_or_else(|| Self::incompatible_receiver_error(&family))?
                    }
                    _ => return Err(Self::incompatible_receiver_error(&family)),
                };
                match member.as_str() {
                    "toLocaleString" => Ok(Value::String(value.to_string())),
                    "toString" => {
                        let radix = if let Some(arg) = args.first() {
                            let radix = Self::value_to_i64(arg);
                            if !(2..=36).contains(&radix) {
                                return Err(Error::ScriptRuntime(
                                    "toString radix must be between 2 and 36".into(),
                                ));
                            }
                            radix as u32
                        } else {
                            10
                        };
                        Ok(Value::String(value.to_str_radix(radix)))
                    }
                    "valueOf" => Ok(Value::BigInt(value)),
                    _ => Err(Error::ScriptRuntime(format!(
                        "unsupported BigInt method: {member}"
                    ))),
                }
            }
            "symbol" => {
                let symbol = match receiver {
                    Value::Symbol(symbol) => symbol,
                    Value::Object(object) => {
                        let entries = object.borrow();
                        let symbol_id = Self::symbol_wrapper_id_from_object(&entries)
                            .ok_or_else(|| Self::incompatible_receiver_error(&family))?;
                        self.symbol_runtime
                            .symbols_by_id
                            .get(&symbol_id)
                            .cloned()
                            .ok_or_else(|| Self::incompatible_receiver_error(&family))?
                    }
                    _ => {
                        return Err(Self::incompatible_receiver_error(&family));
                    }
                };
                match member.as_str() {
                    "toString" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Symbol.toString does not take arguments".into(),
                            ));
                        }
                        Ok(Value::String(Value::Symbol(symbol.clone()).as_string()))
                    }
                    "valueOf" => Ok(Value::Symbol(symbol)),
                    _ => Err(Error::ScriptRuntime(format!(
                        "unsupported Symbol method: {member}"
                    ))),
                }
            }
            "regexp" => {
                if let Value::RegExp(regex) = receiver {
                    if let Some(value) =
                        Self::regexp_instance_property_value(&regex.borrow(), &member)
                    {
                        return Ok(value);
                    }
                    return self
                        .eval_regexp_member_call_from_values(&regex, &member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported RegExp method: {member}"))
                        });
                }
                let Value::Object(entries) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                if !Self::is_regexp_prototype_object(&entries.borrow()) {
                    return Err(Self::incompatible_receiver_error(&family));
                }
                if let Some(value) = Self::regexp_default_property_value(&member) {
                    return Ok(value);
                }
                match member.as_str() {
                    "toString" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "RegExp.toString does not take arguments".into(),
                            ));
                        }
                        Ok(Value::String("/(?:)/".to_string()))
                    }
                    _ => Err(Self::incompatible_receiver_error(&family)),
                }
            }
            "object" => match member.as_str() {
                "hasOwnProperty" => {
                    let key = args.first().cloned().unwrap_or(Value::Undefined);
                    self.object_prototype_has_own_property_value(&receiver, &key)
                }
                "isPrototypeOf" => {
                    let value = args.first().cloned().unwrap_or(Value::Undefined);
                    self.object_prototype_is_prototype_of_value(&receiver, &value)
                }
                "propertyIsEnumerable" => {
                    let key = args.first().cloned().unwrap_or(Value::Undefined);
                    self.object_prototype_property_is_enumerable_value(&receiver, &key)
                }
                "toString" => self.object_prototype_to_string_value(&receiver),
                "valueOf" => self.object_prototype_value_of_value(&receiver),
                _ => Err(Error::ScriptRuntime(format!(
                    "unsupported Object method: {member}"
                ))),
            },
            _ => Err(Error::ScriptRuntime(
                "builtin method has invalid internal state".into(),
            )),
        }
    }
}
