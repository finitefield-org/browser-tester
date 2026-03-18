use super::*;

impl Harness {
    pub(crate) fn build_function_from_constructor_values(
        &mut self,
        args: &[Value],
    ) -> Result<Value> {
        if args.is_empty() {
            return Err(Error::ScriptRuntime(
                "new Function requires at least one argument".into(),
            ));
        }

        let mut parts = Vec::with_capacity(args.len());
        for arg in args {
            parts.push(arg.as_string());
        }

        let body_src = parts
            .last()
            .cloned()
            .ok_or_else(|| Error::ScriptRuntime("new Function requires body argument".into()))?;
        let mut params = Vec::new();
        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            let names = Self::parse_function_constructor_param_names(part)?;
            params.extend(names.into_iter().map(|name| FunctionParam {
                name,
                default: None,
                is_rest: false,
            }));
        }

        let stmts = parse_block_statements(&body_src).map_err(|err| {
            Error::ScriptRuntime(format!("new Function body parse failed: {err}"))
        })?;
        let empty_env = HashMap::new();
        let value = self.make_function_value(
            ScriptHandler { params, stmts },
            &empty_env,
            true,
            false,
            false,
            false,
            false,
        );
        if let Value::Function(function) = &value {
            self.set_function_public_name(function, "anonymous");
        }
        Ok(value)
    }

    pub(crate) fn execute_callable_value(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
    ) -> Result<Value> {
        self.execute_callable_value_with_env(callable, args, event, None)
    }

    pub(crate) fn execute_constructor_value_with_env(
        &mut self,
        constructor: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        self.execute_constructor_value_with_this_and_env(constructor, args, event, caller_env, None)
    }

    pub(crate) fn execute_constructor_value_with_this_and_env(
        &mut self,
        constructor: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
        this_arg: Option<Value>,
    ) -> Result<Value> {
        match constructor {
            Value::RegExpConstructor => self.construct_regexp_from_values(args),
            Value::TypedArrayConstructor(TypedArrayConstructorKind::Concrete(kind)) => {
                self.construct_typed_array_from_values(*kind, args)
            }
            Value::TypedArrayConstructor(TypedArrayConstructorKind::Abstract) => Err(
                Error::ScriptRuntime("Abstract class TypedArray not directly constructable".into()),
            ),
            Value::StringConstructor => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Ok(Self::new_string_wrapper_value(
                    self.coerce_to_string_for_string_constructor(&value)?,
                ))
            }
            Value::BlobConstructor => self.construct_blob_from_values(args),
            Value::UrlConstructor => self.construct_url_from_values(args),
            Value::ArrayBufferConstructor => self.construct_array_buffer_from_values(args),
            Value::PromiseConstructor => self.construct_promise_from_values(args, event),
            Value::MapConstructor => self.construct_map_from_values(args),
            Value::WeakMapConstructor => self.construct_weak_map_from_values(args),
            Value::SetConstructor => self.construct_set_from_values(args),
            Value::WeakSetConstructor => self.construct_weak_set_from_values(args),
            Value::UrlSearchParamsConstructor => self.construct_url_search_params_from_values(args),
            Value::SymbolConstructor => {
                Err(Error::ScriptRuntime("Symbol is not a constructor".into()))
            }
            Value::Function(function) => {
                if function.is_generator || function.is_arrow || function.is_method {
                    return Err(Error::ScriptRuntime("value is not a constructor".into()));
                }
                let effective_new_target = if this_arg.is_some() {
                    caller_env
                        .and_then(|env| env.get(INTERNAL_NEW_TARGET_KEY).cloned())
                        .unwrap_or_else(|| constructor.clone())
                } else {
                    constructor.clone()
                };
                let is_derived_class_constructor =
                    function.is_class_constructor && function.class_super_constructor.is_some();
                let constructor_prototype =
                    match self.object_property_from_value(constructor, "prototype")? {
                        Value::Object(prototype) => Value::Object(prototype),
                        _ => Value::Object(function.prototype_object.clone()),
                    };
                let instance = if let Some(instance) = this_arg {
                    if Self::is_primitive_value(&instance) {
                        return Err(Error::ScriptRuntime(
                            "constructor this value must be an object".into(),
                        ));
                    }
                    instance
                } else {
                    Self::new_object_value(vec![(
                        INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                        constructor_prototype,
                    )])
                };
                let result = self.execute_function_call(
                    function.clone(),
                    args,
                    event,
                    caller_env,
                    Some(instance.clone()),
                    Some(effective_new_target),
                    None,
                )?;
                if Self::is_primitive_value(&result) {
                    if is_derived_class_constructor && !matches!(result, Value::Undefined) {
                        return Err(Error::ScriptRuntime(
                            "Derived constructors may only return object or undefined".into(),
                        ));
                    }
                    Ok(instance)
                } else {
                    Ok(result)
                }
            }
            other => {
                if matches!(
                    Self::callable_kind_from_value(other),
                    Some("bound_function")
                ) {
                    let (target, _bound_this, mut bound_args) =
                        Self::bound_callable_components(other)?;
                    bound_args.extend_from_slice(args);
                    return self.execute_constructor_value_with_env(
                        &target,
                        &bound_args,
                        event,
                        caller_env,
                    );
                }
                match Self::callable_kind_from_value(other) {
                    Some("boolean_constructor") => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        return Ok(Self::new_boolean_wrapper_value(value.truthy()));
                    }
                    Some("number_constructor") => {
                        let value = args.first().cloned().unwrap_or(Value::Number(0));
                        return Ok(Self::new_number_wrapper_value(Self::number_value(
                            Self::coerce_number_for_number_constructor(&value),
                        )));
                    }
                    Some("bigint_constructor") => {
                        return Err(Error::ScriptRuntime("BigInt is not a constructor".into()));
                    }
                    Some("object_constructor") => {
                        if args.is_empty() || matches!(args[0], Value::Null | Value::Undefined) {
                            return Ok(Self::new_object_value(Vec::new()));
                        }
                        return Ok(match &args[0] {
                            Value::Object(_)
                            | Value::Array(_)
                            | Value::Date(_)
                            | Value::Map(_)
                            | Value::WeakMap(_)
                            | Value::Set(_)
                            | Value::WeakSet(_)
                            | Value::Blob(_)
                            | Value::ArrayBuffer(_)
                            | Value::TypedArray(_)
                            | Value::Promise(_)
                            | Value::RegExp(_)
                            | Value::Function(_)
                            | Value::Node(_)
                            | Value::NodeList(_)
                            | Value::FormData(_)
                            | Value::StringConstructor
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
                            | Value::TypedArrayConstructor(_)
                            | Value::PromiseCapability(_) => args[0].clone(),
                            _ => Self::box_primitive_value(args[0].clone()),
                        });
                    }
                    _ => {}
                }
                if self.is_callable_value(other) {
                    self.execute_callable_value_with_this_and_env(
                        other, args, event, caller_env, this_arg,
                    )
                } else {
                    Err(Error::ScriptRuntime("value is not a constructor".into()))
                }
            }
        }
    }

    pub(crate) fn execute_callable_value_with_env(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        self.execute_callable_value_with_this_and_env(callable, args, event, caller_env, None)
    }

    pub(crate) fn execute_callable_value_with_this_and_env(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
        this_arg: Option<Value>,
    ) -> Result<Value> {
        match callable {
            Value::Function(function) => {
                if function.is_class_constructor {
                    return Err(Error::ScriptRuntime(
                        "Class constructor cannot be invoked without 'new'".into(),
                    ));
                }
                self.execute_function_call(
                    function.clone(),
                    args,
                    event,
                    caller_env,
                    this_arg,
                    None,
                    None,
                )
            }
            Value::PromiseCapability(capability) => {
                self.invoke_promise_capability(capability, args)
            }
            Value::StringConstructor => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Ok(Value::String(
                    self.coerce_to_string_for_string_constructor(&value)?,
                ))
            }
            Value::RegExpConstructor => self.construct_regexp_from_values(args),
            Value::TypedArrayConstructor(kind) => match kind {
                TypedArrayConstructorKind::Concrete(kind) => Err(Error::ScriptRuntime(format!(
                    "{} constructor must be called with new",
                    kind.name()
                ))),
                TypedArrayConstructorKind::Abstract => Err(Error::ScriptRuntime(
                    "Abstract class TypedArray not directly constructable".into(),
                )),
            },
            Value::BlobConstructor => Err(Error::ScriptRuntime(
                "Blob constructor must be called with new".into(),
            )),
            Value::UrlConstructor => Err(Error::ScriptRuntime(
                "URL constructor must be called with new".into(),
            )),
            Value::ArrayBufferConstructor => Err(Error::ScriptRuntime(
                "ArrayBuffer constructor must be called with new".into(),
            )),
            Value::PromiseConstructor => Err(Error::ScriptRuntime(
                "Promise constructor must be called with new".into(),
            )),
            Value::MapConstructor => Err(Error::ScriptRuntime(
                "Map constructor must be called with new".into(),
            )),
            Value::WeakMapConstructor => Err(Error::ScriptRuntime(
                "WeakMap constructor must be called with new".into(),
            )),
            Value::SetConstructor => Err(Error::ScriptRuntime(
                "Set constructor must be called with new".into(),
            )),
            Value::WeakSetConstructor => Err(Error::ScriptRuntime(
                "WeakSet constructor must be called with new".into(),
            )),
            Value::UrlSearchParamsConstructor => Err(Error::ScriptRuntime(
                "URLSearchParams constructor must be called with new".into(),
            )),
            Value::SymbolConstructor => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "Symbol supports zero or one argument".into(),
                    ));
                }
                let description = args.first().cloned().unwrap_or(Value::Undefined);
                let description = if matches!(description, Value::Undefined) {
                    None
                } else {
                    Some(description.as_string())
                };
                Ok(self.new_symbol_value(description, None))
            }
            Value::Object(_) => {
                let Some(kind) = Self::callable_kind_from_value(callable) else {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                };
                if let Some(value) = self.execute_window_global_callable_kind(
                    kind,
                    args,
                    event,
                    caller_env,
                    this_arg.as_ref(),
                )? {
                    return Ok(value);
                }
                if let Some(value) =
                    self.execute_object_callable_worker_or_static_kind(kind, callable, args, event)?
                {
                    return Ok(value);
                }
                if let Some(value) =
                    self.execute_object_callable_intl_kind(kind, callable, args, this_arg.as_ref())?
                {
                    return Ok(value);
                }
                if let Some(value) =
                    self.execute_object_callable_iterator_kind(kind, callable, args, event)?
                {
                    return Ok(value);
                }
                if let Some(value) = self.execute_object_callable_platform_kind(
                    kind,
                    callable,
                    args,
                    this_arg.as_ref(),
                )? {
                    return Ok(value);
                }
                if let Some(value) = self.execute_object_callable_dom_collection_kind(
                    kind,
                    args,
                    event,
                    caller_env,
                    this_arg.as_ref(),
                )? {
                    return Ok(value);
                }
                match kind {
                    "function_call" => {
                        let target = self.callable_receiver_from_this_arg(this_arg, "call")?;
                        self.execute_function_prototype_member(
                            "call", &target, args, event, caller_env,
                        )
                    }
                    "function_apply" => {
                        let target = self.callable_receiver_from_this_arg(this_arg, "apply")?;
                        self.execute_function_prototype_member(
                            "apply", &target, args, event, caller_env,
                        )
                    }
                    "function_bind" => {
                        let target = self.callable_receiver_from_this_arg(this_arg, "bind")?;
                        self.execute_function_prototype_member(
                            "bind", &target, args, event, caller_env,
                        )
                    }
                    "function_to_string" => {
                        let target = self.callable_receiver_from_this_arg(this_arg, "toString")?;
                        self.execute_function_prototype_member(
                            "toString", &target, args, event, caller_env,
                        )
                    }
                    "bound_function" => {
                        let (target, bound_this, mut bound_args) =
                            Self::bound_callable_components(callable)?;
                        bound_args.extend_from_slice(args);
                        self.execute_callable_value_with_this_and_env(
                            &target,
                            &bound_args,
                            event,
                            caller_env,
                            Some(bound_this),
                        )
                    }
                    "receiver_builtin_method" => self.execute_receiver_builtin_callable(
                        callable, args, event, this_arg, caller_env,
                    ),
                    _ => Err(Error::ScriptRuntime("callback is not a function".into())),
                }
            }
            _ => Err(Error::ScriptRuntime("callback is not a function".into())),
        }
    }
}
