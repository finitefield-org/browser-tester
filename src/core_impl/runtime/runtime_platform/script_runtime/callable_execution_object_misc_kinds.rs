use super::*;

impl Harness {
    pub(crate) fn execute_object_callable_worker_or_static_kind(
        &mut self,
        kind: &str,
        callable: &Value,
        args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match kind {
            "worker_constructor" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Worker constructor requires one or two arguments".into(),
                    ));
                }
                let source = self.resolve_worker_script_source(&args[0].as_string())?;
                Some(self.new_worker_instance_from_script_source(&source)?)
            }
            "data_transfer_constructor" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "DataTransfer constructor does not take arguments".into(),
                    ));
                }
                Some(self.new_data_transfer_object_value("dragstart"))
            }
            "option_constructor" => {
                if args.len() > 4 {
                    return Err(Error::ScriptRuntime(
                        "Option constructor supports up to four arguments".into(),
                    ));
                }
                let text = if args.is_empty() {
                    String::new()
                } else {
                    args[0].as_string()
                };
                let option = self.dom.create_detached_element("option".to_string());
                self.dom.set_text_content(option, &text)?;

                if args.len() >= 2 {
                    self.dom.set_value(option, &args[1].as_string())?;
                }

                let default_selected = args.get(2).is_some_and(Value::truthy);
                let selected = args.get(3).is_some_and(Value::truthy);
                if default_selected {
                    self.dom.set_attr(option, "selected", "true")?;
                }
                if selected {
                    self.dom
                        .set_option_selected_state(option, true, Some(true))?;
                }

                Some(Value::Node(option))
            }
            "audio_constructor" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "Audio constructor supports up to one argument".into(),
                    ));
                }
                let audio = self.dom.create_detached_element("audio".to_string());
                if let Some(src) = args.first() {
                    self.dom.set_attr(audio, "src", &src.as_string())?;
                }
                Some(Value::Node(audio))
            }
            "worker_context_post_message" => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "WorkerGlobalScope.postMessage supports up to two arguments".into(),
                    ));
                }
                let data = args.first().cloned().unwrap_or(Value::Undefined);
                let normalized_options = match args.get(1) {
                    Some(Value::Array(values)) => Some(Self::new_object_value(vec![(
                        "transfer".to_string(),
                        Value::Array(values.clone()),
                    )])),
                    Some(other) => Some(other.clone()),
                    None => None,
                };
                let worker = Self::worker_target_from_callable(callable)?;
                if Self::worker_is_terminated_object(&worker) {
                    return Ok(Some(Value::Undefined));
                }
                let data =
                    Self::structured_clone_value_with_options(&data, normalized_options.as_ref())?;
                let worker_value = Value::Object(worker.clone());
                self.queue_worker_message_microtask(&worker, &worker, worker_value, data);
                Some(Value::Undefined)
            }
            "worker_main_post_message" => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Worker.postMessage supports up to two arguments".into(),
                    ));
                }
                let data = args.first().cloned().unwrap_or(Value::Undefined);
                let normalized_options = match args.get(1) {
                    Some(Value::Array(values)) => Some(Self::new_object_value(vec![(
                        "transfer".to_string(),
                        Value::Array(values.clone()),
                    )])),
                    Some(other) => Some(other.clone()),
                    None => None,
                };
                let worker = Self::worker_target_from_callable(callable)?;
                if Self::worker_is_terminated_object(&worker) {
                    return Ok(Some(Value::Undefined));
                }
                let data =
                    Self::structured_clone_value_with_options(&data, normalized_options.as_ref())?;
                let worker_global = Self::worker_global_from_object(&worker)?;
                let worker_global_value = Value::Object(worker_global.clone());
                self.queue_worker_message_microtask(
                    &worker,
                    &worker_global,
                    worker_global_value,
                    data,
                );
                Some(Value::Undefined)
            }
            "worker_terminate" => {
                let worker = Self::worker_target_from_callable(callable)?;
                Self::worker_set_terminated_object(&worker, true);
                Some(Value::Undefined)
            }
            "string_static_from_char_code" => Some(
                self.eval_string_static_method_from_values(StringStaticMethod::FromCharCode, args)?,
            ),
            "string_static_from_code_point" => {
                Some(self.eval_string_static_method_from_values(
                    StringStaticMethod::FromCodePoint,
                    args,
                )?)
            }
            "string_static_raw" => {
                Some(self.eval_string_static_method_from_values(StringStaticMethod::Raw, args)?)
            }
            "object_static_method" => Some(match Self::static_method_name(callable)?.as_str() {
                "create" => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Object.create requires one or two arguments".into(),
                        ));
                    }
                    self.object_create_value(&args[0], args.get(1))?
                }
                "assign" => self.eval_object_assign_static_call(args, event)?,
                "getOwnPropertyDescriptor" => {
                    if args.len() != 2 {
                        return Err(Error::ScriptRuntime(
                            "Object.getOwnPropertyDescriptor requires exactly two arguments".into(),
                        ));
                    }
                    let key = self.property_key_to_storage_key(&args[1]);
                    self.object_get_own_property_descriptor_value(&args[0], &key)?
                }
                "defineProperty" => {
                    if args.len() != 3 {
                        return Err(Error::ScriptRuntime(
                            "Object.defineProperty requires exactly three arguments".into(),
                        ));
                    }
                    let key = self.property_key_to_storage_key(&args[1]);
                    self.object_define_property_value(&args[0], &key, &args[2])?
                }
                "getOwnPropertyNames" => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Object.getOwnPropertyNames requires exactly one argument".into(),
                        ));
                    }
                    self.object_get_own_property_names_value(&args[0])?
                }
                "getOwnPropertySymbols" => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Object.getOwnPropertySymbols requires exactly one argument".into(),
                        ));
                    }
                    self.object_get_own_property_symbols_value(&args[0])?
                }
                "keys" => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Object.keys requires exactly one argument".into(),
                        ));
                    }
                    self.object_keys_value(&args[0])?
                }
                "values" => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Object.values requires exactly one argument".into(),
                        ));
                    }
                    self.object_values_value(&args[0])?
                }
                "entries" => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Object.entries requires exactly one argument".into(),
                        ));
                    }
                    self.object_entries_value(&args[0])?
                }
                "fromEntries" => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Object.fromEntries requires exactly one argument".into(),
                        ));
                    }
                    self.object_from_entries_value(&args[0])?
                }
                "hasOwn" => {
                    if args.len() != 2 {
                        return Err(Error::ScriptRuntime(
                            "Object.hasOwn requires exactly two arguments".into(),
                        ));
                    }
                    let key = self.property_key_to_storage_key(&args[1]);
                    self.object_has_own_value(&args[0], &key)?
                }
                "getPrototypeOf" => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Object.getPrototypeOf requires exactly one argument".into(),
                        ));
                    }
                    self.object_get_prototype_of_value(&args[0])?
                }
                "setPrototypeOf" => {
                    if args.len() != 2 {
                        return Err(Error::ScriptRuntime(
                            "Object.setPrototypeOf requires exactly two arguments".into(),
                        ));
                    }
                    self.object_set_prototype_of_value(&args[0], &args[1])?
                }
                "freeze" => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Object.freeze requires exactly one argument".into(),
                        ));
                    }
                    self.object_freeze_value(&args[0])?
                }
                _ => return Err(Error::ScriptRuntime("callback is not a function".into())),
            }),
            "number_static_method" => {
                let method = match Self::static_method_name(callable)?.as_str() {
                    "isFinite" => NumberMethod::IsFinite,
                    "isInteger" => NumberMethod::IsInteger,
                    "isNaN" => NumberMethod::IsNaN,
                    "isSafeInteger" => NumberMethod::IsSafeInteger,
                    "parseFloat" => NumberMethod::ParseFloat,
                    "parseInt" => NumberMethod::ParseInt,
                    _ => return Err(Error::ScriptRuntime("callback is not a function".into())),
                };
                Some(self.eval_number_method_from_values(method, args)?)
            }
            "bigint_static_method" => {
                let method = match Self::static_method_name(callable)?.as_str() {
                    "asIntN" => BigIntMethod::AsIntN,
                    "asUintN" => BigIntMethod::AsUintN,
                    _ => return Err(Error::ScriptRuntime("callback is not a function".into())),
                };
                Some(self.eval_bigint_method_from_values(method, args)?)
            }
            "regexp_static_method" => {
                let method = match Self::static_method_name(callable)?.as_str() {
                    "escape" => RegExpStaticMethod::Escape,
                    _ => return Err(Error::ScriptRuntime("callback is not a function".into())),
                };
                Some(self.eval_regexp_static_method_from_values(method, args)?)
            }
            "promise_static_method" => {
                let method = match Self::static_method_name(callable)?.as_str() {
                    "resolve" => PromiseStaticMethod::Resolve,
                    "reject" => PromiseStaticMethod::Reject,
                    "all" => PromiseStaticMethod::All,
                    "allSettled" => PromiseStaticMethod::AllSettled,
                    "any" => PromiseStaticMethod::Any,
                    "race" => PromiseStaticMethod::Race,
                    "try" => PromiseStaticMethod::Try,
                    "withResolvers" => PromiseStaticMethod::WithResolvers,
                    _ => return Err(Error::ScriptRuntime("callback is not a function".into())),
                };
                Some(self.eval_promise_static_method_from_values(method, args, event)?)
            }
            "array_buffer_static_method" => {
                Some(match Self::static_method_name(callable)?.as_str() {
                    "isView" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "ArrayBuffer.isView requires exactly one argument".into(),
                            ));
                        }
                        Value::Bool(matches!(args.first(), Some(Value::TypedArray(_))))
                    }
                    _ => return Err(Error::ScriptRuntime("callback is not a function".into())),
                })
            }
            "symbol_static_method" => {
                let method = match Self::static_method_name(callable)?.as_str() {
                    "for" => SymbolStaticMethod::For,
                    "keyFor" => SymbolStaticMethod::KeyFor,
                    _ => return Err(Error::ScriptRuntime("callback is not a function".into())),
                };
                Some(self.eval_symbol_static_method_from_values(method, args)?)
            }
            "typed_array_static_method" => {
                let (kind, method_name) = Self::typed_array_static_method_components(callable)?;
                let TypedArrayConstructorKind::Concrete(kind) = kind else {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                };
                let method = match method_name.as_str() {
                    "from" => TypedArrayStaticMethod::From,
                    "of" => TypedArrayStaticMethod::Of,
                    _ => return Err(Error::ScriptRuntime("callback is not a function".into())),
                };
                Some(self.eval_typed_array_static_method_from_values(kind, method, args)?)
            }
            "reflect_static_method" => Some(match Self::static_method_name(callable)?.as_str() {
                "set" => {
                    if args.len() != 3 && args.len() != 4 {
                        return Err(Error::ScriptRuntime(
                            "Reflect.set requires three or four arguments".into(),
                        ));
                    }
                    let receiver = args.get(3).cloned().unwrap_or_else(|| args[0].clone());
                    let key = self.property_key_to_storage_key(&args[1]);
                    Value::Bool(self.reflect_set_object_property_value(
                        &args[0],
                        &key,
                        args[2].clone(),
                        &receiver,
                        event,
                    )?)
                }
                "ownKeys" => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Reflect.ownKeys requires exactly one argument".into(),
                        ));
                    }
                    self.reflect_own_keys_value(&args[0])?
                }
                _ => return Err(Error::ScriptRuntime("callback is not a function".into())),
            }),
            "create_image_bitmap" => Some(self.eval_create_image_bitmap_call(args)?),
            _ => None,
        };
        Ok(value)
    }
}
