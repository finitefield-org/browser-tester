use super::callable_execution_runtime_helpers::{
    INTERNAL_ASYNC_FUNCTION_SUSPENDED, TopLevelAwaitOutcome, TopLevelAwaitResumeKind,
};
use super::*;

impl Harness {
    pub(crate) fn window_open_target_url(&self, args: &[Value]) -> String {
        let requested = args.first().map(Value::as_string).unwrap_or_default();
        if requested.trim().is_empty() {
            "about:blank".to_string()
        } else {
            self.resolve_document_target_url(&requested)
        }
    }

    fn window_open_disables_opener(features: &str) -> bool {
        features.split(',').any(|raw_feature| {
            let feature = raw_feature.trim();
            if feature.is_empty() {
                return false;
            }
            let mut parts = feature.splitn(2, '=');
            let name = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
            let value = parts.next().map(|value| value.trim().to_ascii_lowercase());
            match name.as_str() {
                "noopener" | "noreferrer" => {
                    !matches!(value.as_deref(), Some("0") | Some("false") | Some("no"))
                }
                _ => false,
            }
        })
    }

    pub(crate) fn new_popup_window_value(&self, url: &str, target: &str, features: &str) -> Value {
        let popup_window = Rc::new(RefCell::new(ObjectValue::default()));
        let popup_document = Rc::new(RefCell::new(ObjectValue::default()));
        let popup_window_value = Value::Object(popup_window.clone());
        let popup_document_value = Value::Object(popup_document.clone());
        let opener = if Self::window_open_disables_opener(features) {
            Value::Null
        } else {
            Value::Object(self.dom_runtime.window_object.clone())
        };
        let popup_location =
            Self::new_object_value(vec![("href".to_string(), Value::String(url.to_string()))]);

        {
            let mut document_entries = popup_document.borrow_mut();
            Self::object_set_entry(
                &mut document_entries,
                INTERNAL_POPUP_DOCUMENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            );
            Self::object_set_entry(
                &mut document_entries,
                INTERNAL_POPUP_DOCUMENT_HTML_KEY.to_string(),
                Value::String(String::new()),
            );
            Self::object_set_entry(
                &mut document_entries,
                "defaultView".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut document_entries,
                "URL".to_string(),
                Value::String(url.to_string()),
            );
            Self::object_set_entry(
                &mut document_entries,
                "baseURI".to_string(),
                Value::String(url.to_string()),
            );
            Self::object_set_entry(
                &mut document_entries,
                "readyState".to_string(),
                Value::String("complete".to_string()),
            );
            Self::object_set_entry(
                &mut document_entries,
                "open".to_string(),
                Self::new_popup_document_open_callable_value(),
            );
            Self::object_set_entry(
                &mut document_entries,
                "write".to_string(),
                Self::new_popup_document_write_callable_value(),
            );
            Self::object_set_entry(
                &mut document_entries,
                "close".to_string(),
                Self::new_popup_document_close_callable_value(),
            );
        }

        {
            let mut window_entries = popup_window.borrow_mut();
            Self::object_set_entry(
                &mut window_entries,
                INTERNAL_POPUP_WINDOW_OBJECT_KEY.to_string(),
                Value::Bool(true),
            );
            Self::object_set_entry(
                &mut window_entries,
                "window".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "globalThis".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "self".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "top".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "parent".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "frames".to_string(),
                popup_window_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "closed".to_string(),
                Value::Bool(false),
            );
            Self::object_set_entry(
                &mut window_entries,
                "name".to_string(),
                Value::String(target.to_string()),
            );
            Self::object_set_entry(&mut window_entries, "opener".to_string(), opener);
            Self::object_set_entry(&mut window_entries, "location".to_string(), popup_location);
            Self::object_set_entry(
                &mut window_entries,
                "document".to_string(),
                popup_document_value.clone(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "close".to_string(),
                Self::new_popup_window_close_callable_value(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "focus".to_string(),
                Self::new_popup_window_focus_callable_value(),
            );
            Self::object_set_entry(
                &mut window_entries,
                "print".to_string(),
                Self::new_popup_window_print_callable_value(),
            );
        }

        popup_window_value
    }

    fn execute_receiver_builtin_callable(
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
                    "receiver_builtin_method" => {
                        self.execute_receiver_builtin_callable(
                            callable,
                            args,
                            event,
                            this_arg,
                            caller_env,
                        )
                    }
                    "intl_collator_get_compare" => {
                        self.intl_bound_compare_callable_from_receiver(
                            this_arg.as_ref().ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "Intl.Collator.compare requires an Intl.Collator instance"
                                        .into(),
                                )
                            })?,
                        )
                    }
                    "intl_date_time_format_get_format" => {
                        self.intl_bound_date_time_format_callable_from_receiver(
                            this_arg.as_ref().ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "Intl.DateTimeFormat method requires an Intl.DateTimeFormat instance"
                                        .into(),
                                )
                            })?,
                        )
                    }
                    "intl_number_format_get_format" => {
                        self.intl_bound_number_format_callable_from_receiver(
                            this_arg.as_ref().ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "Intl.NumberFormat method requires an Intl.NumberFormat instance"
                                        .into(),
                                )
                            })?,
                        )
                    }
                    "intl_collator_compare" => {
                        let (locale, case_first, sensitivity, numeric) =
                            self.resolve_intl_collator_options(callable)?;
                        let left = args
                            .first()
                            .cloned()
                            .unwrap_or(Value::Undefined)
                            .as_string();
                        let right = args.get(1).cloned().unwrap_or(Value::Undefined).as_string();
                        Ok(Value::Number(Self::intl_collator_compare_strings(
                            &left,
                            &right,
                            &locale,
                            &case_first,
                            &sensitivity,
                            numeric,
                        )))
                    }
                    "intl_date_time_format" => {
                        let (locale, options) = self.resolve_intl_date_time_options(callable)?;
                        let timestamp_ms = args
                            .first()
                            .map(|value| self.coerce_date_timestamp_ms(value))
                            .unwrap_or(self.scheduler.now_ms);
                        Ok(Value::String(self.intl_format_date_time(
                            timestamp_ms,
                            &locale,
                            &options,
                        )))
                    }
                    "intl_duration_format" => {
                        let (locale, options) = self.resolve_intl_duration_options(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(
                            self.intl_format_duration(&locale, &options, &value)?,
                        ))
                    }
                    "intl_list_format" => {
                        let (locale, options) = self.resolve_intl_list_options(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(
                            self.intl_format_list(&locale, &options, &value)?,
                        ))
                    }
                    "intl_number_format" => {
                        let (locale, options) =
                            self.resolve_intl_number_format_options(callable)?;
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::String(self.intl_format_number_value_with_options(
                            &value, &locale, &options,
                        )))
                    }
                    "intl_segmenter_segments_iterator" => {
                        let Value::Object(entries) = callable else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let entries = entries.borrow();
                        let segments = Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENTS_KEY)
                            .ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "Intl.Segmenter iterator has invalid internal state".into(),
                                )
                            })?;
                        Ok(self.new_intl_segmenter_iterator_value(segments))
                    }
                    "intl_segmenter_iterator_next" => {
                        let Value::Object(entries) = callable else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let mut entries = entries.borrow_mut();
                        let segments = Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENTS_KEY)
                            .ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "Intl.Segmenter iterator has invalid internal state".into(),
                                )
                            })?;
                        let Value::Array(values) = segments else {
                            return Err(Error::ScriptRuntime(
                                "Intl.Segmenter iterator has invalid internal state".into(),
                            ));
                        };
                        let len = values.borrow().len();
                        let index =
                            match Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENT_INDEX_KEY)
                            {
                                Some(Value::Number(value)) if value >= 0 => value as usize,
                                _ => 0,
                            };
                        if index >= len {
                            return Ok(Self::new_object_value(vec![
                                ("value".to_string(), Value::Undefined),
                                ("done".to_string(), Value::Bool(true)),
                            ]));
                        }
                        let value = values
                            .borrow()
                            .get(index)
                            .cloned()
                            .unwrap_or(Value::Undefined);
                        Self::object_set_entry(
                            &mut entries,
                            INTERNAL_INTL_SEGMENT_INDEX_KEY.to_string(),
                            Value::Number((index + 1) as i64),
                        );
                        Ok(Self::new_object_value(vec![
                            ("value".to_string(), value),
                            ("done".to_string(), Value::Bool(false)),
                        ]))
                    }
                    "readable_stream_async_iterator" => {
                        let Value::Object(entries) = callable else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let entries = entries.borrow();
                        let chunks = match Self::object_get_entry(
                            &entries,
                            INTERNAL_ASYNC_ITERATOR_VALUES_KEY,
                        ) {
                            Some(Value::Array(values)) => values.borrow().clone(),
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "ReadableStream async iterator has invalid internal state"
                                        .into(),
                                ));
                            }
                        };
                        Ok(self.new_async_iterator_value(chunks))
                    }
                    "named_node_map_iterator" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "NamedNodeMap[Symbol.iterator] does not take arguments".into(),
                            ));
                        }
                        let Value::Object(entries) = callable else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let entries = entries.borrow();
                        let Some(owner) = Self::named_node_map_owner_node(&entries) else {
                            return Err(Error::ScriptRuntime(
                                "NamedNodeMap iterator has invalid internal state".into(),
                            ));
                        };
                        let values = self
                            .named_node_map_entries(owner)
                            .into_iter()
                            .map(|(name, value)| {
                                Self::new_attr_object_value(&name, &value, Some(owner))
                            })
                            .collect::<Vec<_>>();
                        Ok(self.new_iterator_value(values))
                    }
                    "iterator_self" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Iterator[Symbol.iterator] does not take arguments".into(),
                            ));
                        }
                        let iterator = self.iterator_target_from_callable(callable)?;
                        Ok(Value::Object(iterator))
                    }
                    "async_generator_result_value" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Self::new_async_iterator_result_object(value, false))
                    }
                    "async_generator_result_done" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Self::new_async_iterator_result_object(value, true))
                    }
                    "async_iterator_next" => {
                        let iterator = self.async_iterator_target_from_callable(callable)?;
                        let is_async_generator = {
                            let entries = iterator.borrow();
                            Self::is_async_generator_object(&entries)
                        };
                        if !is_async_generator && !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "AsyncIterator.next does not take arguments".into(),
                            ));
                        }
                        let result = if let Some(value) =
                            self.async_iterator_next_value_from_object(&iterator)?
                        {
                            if is_async_generator {
                                return self
                                    .resolve_async_generator_iterator_result_promise(value, false);
                            }
                            Self::new_async_iterator_result_object(value, false)
                        } else {
                            Self::new_async_iterator_result_object(Value::Undefined, true)
                        };
                        let promise = self.new_pending_promise();
                        self.promise_resolve(&promise, result)?;
                        Ok(Value::Promise(promise))
                    }
                    "async_iterator_return" => {
                        let iterator = self.async_iterator_target_from_callable(callable)?;
                        let is_async_generator = {
                            let entries = iterator.borrow();
                            Self::is_async_generator_object(&entries)
                        };
                        if !is_async_generator {
                            return Err(Error::ScriptRuntime(
                                "AsyncIterator.return is not a function".into(),
                            ));
                        }
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        self.close_async_iterator_object(&iterator)?;
                        self.resolve_async_generator_iterator_result_promise(value, true)
                    }
                    "async_iterator_throw" => {
                        let iterator = self.async_iterator_target_from_callable(callable)?;
                        let is_async_generator = {
                            let entries = iterator.borrow();
                            Self::is_async_generator_object(&entries)
                        };
                        if !is_async_generator {
                            return Err(Error::ScriptRuntime(
                                "AsyncIterator.throw is not a function".into(),
                            ));
                        }
                        let reason = args.first().cloned().unwrap_or(Value::Undefined);
                        self.close_async_iterator_object(&iterator)?;
                        let promise = self.new_pending_promise();
                        self.promise_reject(&promise, reason);
                        Ok(Value::Promise(promise))
                    }
                    "async_iterator_self" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "AsyncIterator[Symbol.asyncIterator] does not take arguments"
                                    .into(),
                            ));
                        }
                        let iterator = self.async_iterator_target_from_callable(callable)?;
                        Ok(Value::Object(iterator))
                    }
                    "async_iterator_async_dispose" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "AsyncIterator[Symbol.asyncDispose] does not take arguments".into(),
                            ));
                        }
                        let iterator = self.async_iterator_target_from_callable(callable)?;
                        let return_value = {
                            let entries = iterator.borrow();
                            Self::object_get_entry(&entries, "return")
                        };
                        let dispose_result = if let Some(return_method) = return_value {
                            if !self.is_callable_value(&return_method) {
                                return Err(Error::ScriptRuntime(
                                    "AsyncIterator.return is not a function".into(),
                                ));
                            }
                            self.execute_callable_value(&return_method, &[], event)?
                        } else {
                            Value::Undefined
                        };
                        let promise = self.new_pending_promise();
                        self.promise_resolve(&promise, dispose_result)?;
                        Ok(Value::Promise(promise))
                    }
                    "async_generator_function_constructor" => {
                        self.build_async_generator_function_from_constructor_values(args)
                    }
                    "generator_function_constructor" => {
                        self.build_generator_function_from_constructor_values(args)
                    }
                    "function_constructor" => self.build_function_from_constructor_values(args),
                    "boolean_constructor" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::Bool(value.truthy()))
                    }
                    "number_constructor" => {
                        let value = args.first().cloned().unwrap_or(Value::Number(0));
                        Ok(Self::number_value(
                            Self::coerce_number_for_number_constructor(&value),
                        ))
                    }
                    "bigint_constructor" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Ok(Value::BigInt(Self::coerce_bigint_for_constructor(&value)?))
                    }
                    "object_constructor" => {
                        if args.is_empty() || matches!(args[0], Value::Null | Value::Undefined) {
                            Ok(Self::new_object_value(Vec::new()))
                        } else {
                            Ok(match &args[0] {
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
                            })
                        }
                    }
                    "node_list_constructor"
                    | "image_bitmap_constructor"
                    | "text_track_constructor"
                    | "text_track_list_constructor"
                    | "time_ranges_constructor"
                    | "storage_constructor"
                    | "cookie_store_constructor"
                    | "cache_storage_constructor"
                    | "cache_constructor"
                    | "radio_node_list_constructor"
                    | "html_collection_constructor"
                    | "html_form_controls_collection_constructor"
                    | "html_options_collection_constructor" => {
                        Err(Error::ScriptRuntime("Illegal constructor".into()))
                    }
                    "event_target_constructor" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "EventTarget constructor does not take arguments".into(),
                            ));
                        }
                        self.new_event_target_instance_from_constructor(callable, this_arg)
                    }
                    "event_constructor" => self.new_event_object_from_constructor_args(
                        "Event", args, false, false, false, false, false, false, false, false,
                    ),
                    "custom_event_constructor" => self.new_event_object_from_constructor_args(
                        "CustomEvent",
                        args,
                        true,
                        false,
                        false,
                        false,
                        false,
                        false,
                        false,
                        false,
                    ),
                    "mouse_event_constructor" => self.new_event_object_from_constructor_args(
                        "MouseEvent",
                        args,
                        false,
                        false,
                        false,
                        false,
                        false,
                        false,
                        false,
                        false,
                    ),
                    "keyboard_event_constructor" => self.new_event_object_from_constructor_args(
                        "KeyboardEvent",
                        args,
                        false,
                        true,
                        false,
                        false,
                        false,
                        false,
                        false,
                        false,
                    ),
                    "wheel_event_constructor" => self.new_event_object_from_constructor_args(
                        "WheelEvent",
                        args,
                        false,
                        false,
                        true,
                        false,
                        false,
                        false,
                        false,
                        false,
                    ),
                    "navigate_event_constructor" => self.new_event_object_from_constructor_args(
                        "NavigateEvent",
                        args,
                        false,
                        false,
                        false,
                        true,
                        false,
                        false,
                        false,
                        false,
                    ),
                    "pointer_event_constructor" => self.new_event_object_from_constructor_args(
                        "PointerEvent",
                        args,
                        false,
                        false,
                        false,
                        false,
                        true,
                        false,
                        false,
                        false,
                    ),
                    "hash_change_event_constructor" => self.new_event_object_from_constructor_args(
                        "HashChangeEvent",
                        args,
                        false,
                        false,
                        false,
                        false,
                        false,
                        true,
                        false,
                        false,
                    ),
                    "error_event_constructor" => self.new_event_object_from_constructor_args(
                        "ErrorEvent",
                        args,
                        false,
                        false,
                        false,
                        false,
                        false,
                        false,
                        true,
                        false,
                    ),
                    "before_unload_event_constructor" => self
                        .new_event_object_from_constructor_args(
                            "BeforeUnloadEvent",
                            args,
                            false,
                            false,
                            false,
                            false,
                            false,
                            false,
                            false,
                            true,
                        ),
                    "image_data_constructor" => self.new_image_data_from_constructor_args(args),
                    "dom_parser_constructor" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "DOMParser constructor does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_dom_parser_instance_value())
                    }
                    "xml_serializer_constructor" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "XMLSerializer constructor does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_xml_serializer_instance_value())
                    }
                    "document_constructor" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Document constructor does not take arguments".into(),
                            ));
                        }
                        Ok(self.new_empty_parsed_document_value())
                    }
                    "document_parse_html" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "Document.parseHTML requires exactly one argument".into(),
                            ));
                        }
                        self.new_parsed_document_value_from_markup(
                            &args[0].as_string(),
                            true,
                            "text/html",
                        )
                    }
                    "document_parse_html_unsafe" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "Document.parseHTMLUnsafe requires exactly one argument".into(),
                            ));
                        }
                        self.new_parsed_document_value_from_markup(
                            &args[0].as_string(),
                            false,
                            "text/html",
                        )
                    }
                    "fetch_function" => self.eval_fetch_call_from_values(args),
                    "match_media_function" => self.eval_match_media_call_from_values(args),
                    "clipboard_item_constructor" => {
                        self.new_clipboard_item_value_from_constructor_args(args)
                    }
                    "clipboard_write" => self.eval_clipboard_write_call(args),
                    "request_constructor" => self.new_fetch_request_value_from_call_args(args),
                    "file_constructor" => {
                        let mut instance = self.new_file_value_from_constructor_args(args)?;
                        self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                        Ok(instance)
                    }
                    "headers_constructor" => self.new_headers_value_from_call_args(args),
                    "text_encoder_constructor" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "TextEncoder constructor does not take arguments".into(),
                            ));
                        }
                        let mut instance = Self::new_text_encoder_instance_value();
                        self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                        Ok(instance)
                    }
                    "text_decoder_constructor" => {
                        if args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "TextDecoder constructor supports up to two arguments".into(),
                            ));
                        }
                        let encoding = match args.first() {
                            None | Some(Value::Undefined) => "utf-8",
                            Some(label) => Self::normalize_text_decoder_label(&label.as_string())
                                .ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "TextDecoder constructor received unsupported encoding label"
                                        .into(),
                                )
                            })?,
                        };
                        let (fatal, ignore_bom) =
                            Self::text_decoder_options_from_value(args.get(1))?;
                        let mut instance =
                            Self::new_text_decoder_instance_value(encoding, fatal, ignore_bom);
                        self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                        Ok(instance)
                    }
                    "text_encoder_stream_constructor" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "TextEncoderStream constructor does not take arguments".into(),
                            ));
                        }
                        let readable = self.new_readable_stream_placeholder_value(Vec::new());
                        let writable = Self::new_writable_stream_placeholder_value();
                        let mut instance =
                            Self::new_text_encoder_stream_instance_value(readable, writable);
                        self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                        Ok(instance)
                    }
                    "text_decoder_stream_constructor" => {
                        if args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "TextDecoderStream constructor supports up to two arguments".into(),
                            ));
                        }
                        let encoding = match args.first() {
                            None | Some(Value::Undefined) => "utf-8",
                            Some(label) => Self::normalize_text_decoder_label(&label.as_string())
                                .ok_or_else(|| {
                                    Error::ScriptRuntime(
                                        "TextDecoderStream constructor received unsupported encoding label"
                                            .into(),
                                    )
                                })?,
                        };
                        let (fatal, ignore_bom) =
                            Self::text_decoder_options_from_value(args.get(1))?;
                        let readable = self.new_readable_stream_placeholder_value(Vec::new());
                        let writable = Self::new_writable_stream_placeholder_value();
                        let mut instance = Self::new_text_decoder_stream_instance_value(
                            encoding, fatal, ignore_bom, readable, writable,
                        );
                        self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                        Ok(instance)
                    }
                    "css_style_sheet_constructor" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "CSSStyleSheet constructor does not take arguments".into(),
                            ));
                        }
                        let mut instance = Self::new_css_style_sheet_instance_value(Value::Object(
                            self.dom_runtime.document_object.clone(),
                        ));
                        self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                        Ok(instance)
                    }
                    "text_encoder_get_encoding" => {
                        Self::text_encoder_receiver_object(this_arg.as_ref())?;
                        Ok(Value::String("utf-8".to_string()))
                    }
                    "text_encoder_encode" => {
                        Self::text_encoder_receiver_object(this_arg.as_ref())?;
                        let input = args.first().map(Value::as_string).unwrap_or_default();
                        Ok(Self::new_uint8_typed_array_from_bytes(input.as_bytes()))
                    }
                    "text_encoder_encode_into" => {
                        Self::text_encoder_receiver_object(this_arg.as_ref())?;
                        if args.len() != 2 {
                            return Err(Error::ScriptRuntime(
                                "TextEncoder.encodeInto requires exactly two arguments".into(),
                            ));
                        }
                        let source = args[0].as_string();
                        let Value::TypedArray(destination) = &args[1] else {
                            return Err(Error::ScriptRuntime(
                                "TextEncoder.encodeInto destination must be a Uint8Array".into(),
                            ));
                        };
                        self.text_encoder_encode_into_value(&source, destination)
                    }
                    "text_decoder_get_encoding" => {
                        let (encoding, _, _) =
                            Self::text_decoder_state_from_receiver(this_arg.as_ref())?;
                        Ok(Value::String(encoding))
                    }
                    "text_decoder_get_fatal" => {
                        let (_, fatal, _) =
                            Self::text_decoder_state_from_receiver(this_arg.as_ref())?;
                        Ok(Value::Bool(fatal))
                    }
                    "text_decoder_get_ignore_bom" => {
                        let (_, _, ignore_bom) =
                            Self::text_decoder_state_from_receiver(this_arg.as_ref())?;
                        Ok(Value::Bool(ignore_bom))
                    }
                    "text_decoder_decode" => {
                        if args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "TextDecoder.decode supports up to two arguments".into(),
                            ));
                        }
                        let (encoding, fatal, ignore_bom) =
                            Self::text_decoder_state_from_receiver(this_arg.as_ref())?;
                        Self::validate_text_decoder_decode_options(args.get(1))?;
                        let bytes = self.text_decoder_input_bytes(args.first())?;
                        Ok(Value::String(Self::decode_text_decoder_bytes(
                            &encoding, &bytes, fatal, ignore_bom,
                        )?))
                    }
                    "text_encoder_stream_get_encoding" => {
                        Self::text_encoder_stream_state_from_receiver(this_arg.as_ref())?;
                        Ok(Value::String("utf-8".to_string()))
                    }
                    "text_encoder_stream_get_readable" => {
                        let (readable, _) =
                            Self::text_encoder_stream_state_from_receiver(this_arg.as_ref())?;
                        Ok(readable)
                    }
                    "text_encoder_stream_get_writable" => {
                        let (_, writable) =
                            Self::text_encoder_stream_state_from_receiver(this_arg.as_ref())?;
                        Ok(writable)
                    }
                    "text_decoder_stream_get_encoding" => {
                        let (encoding, _, _, _, _) =
                            Self::text_decoder_stream_state_from_receiver(this_arg.as_ref())?;
                        Ok(Value::String(encoding))
                    }
                    "text_decoder_stream_get_fatal" => {
                        let (_, fatal, _, _, _) =
                            Self::text_decoder_stream_state_from_receiver(this_arg.as_ref())?;
                        Ok(Value::Bool(fatal))
                    }
                    "text_decoder_stream_get_ignore_bom" => {
                        let (_, _, ignore_bom, _, _) =
                            Self::text_decoder_stream_state_from_receiver(this_arg.as_ref())?;
                        Ok(Value::Bool(ignore_bom))
                    }
                    "text_decoder_stream_get_readable" => {
                        let (_, _, _, readable, _) =
                            Self::text_decoder_stream_state_from_receiver(this_arg.as_ref())?;
                        Ok(readable)
                    }
                    "text_decoder_stream_get_writable" => {
                        let (_, _, _, _, writable) =
                            Self::text_decoder_stream_state_from_receiver(this_arg.as_ref())?;
                        Ok(writable)
                    }
                    "css_style_sheet_replace_sync" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "CSSStyleSheet.replaceSync requires exactly one argument".into(),
                            ));
                        }
                        let sheet = Self::css_style_sheet_object_from_receiver(this_arg.as_ref())?;
                        let replacement = args[0].as_string();
                        let rules = if replacement.trim().is_empty() {
                            Vec::new()
                        } else {
                            vec![Value::String(replacement)]
                        };
                        Self::object_set_entry(
                            &mut sheet.borrow_mut(),
                            INTERNAL_CSS_STYLE_SHEET_RULES_KEY.to_string(),
                            Self::new_array_value(rules),
                        );
                        Ok(Value::Undefined)
                    }
                    "css_style_sheet_insert_rule" => {
                        if args.is_empty() || args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "CSSStyleSheet.insertRule requires one or two arguments".into(),
                            ));
                        }
                        let sheet = Self::css_style_sheet_object_from_receiver(this_arg.as_ref())?;
                        let rule = Value::as_string(&args[0]);
                        let existing_rules = {
                            let sheet_ref = sheet.borrow();
                            match Self::object_get_entry(
                                &sheet_ref,
                                INTERNAL_CSS_STYLE_SHEET_RULES_KEY,
                            ) {
                                Some(Value::Array(rules)) => rules,
                                _ => Rc::new(RefCell::new(ArrayValue::new(Vec::new()))),
                            }
                        };
                        let mut rules_ref = existing_rules.borrow_mut();
                        let default_index = rules_ref.len();
                        let index = if let Some(index_value) = args.get(1) {
                            let requested = Self::value_to_i64(index_value);
                            if requested < 0 || (requested as usize) > rules_ref.len() {
                                return Err(Error::ScriptRuntime(
                                    "CSSStyleSheet.insertRule index out of range".into(),
                                ));
                            }
                            requested as usize
                        } else {
                            default_index
                        };
                        rules_ref.insert(index, Value::String(rule));
                        drop(rules_ref);
                        Self::object_set_entry(
                            &mut sheet.borrow_mut(),
                            INTERNAL_CSS_STYLE_SHEET_RULES_KEY.to_string(),
                            Value::Array(existing_rules),
                        );
                        Ok(Value::Number(index as i64))
                    }
                    "computed_style_get_property_value" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "getPropertyValue requires exactly one argument".into(),
                            ));
                        }
                        let (node, pseudo) =
                            Self::computed_style_state_from_receiver(this_arg.as_ref())?;
                        let property_name = args[0].as_string();
                        let value = self.computed_style_property_value(
                            node,
                            pseudo.as_deref(),
                            &property_name,
                        )?;
                        Ok(Value::String(value))
                    }
                    "computed_style_item" => {
                        if args.len() > 1 {
                            return Err(Error::ScriptRuntime(
                                "item requires zero or one argument".into(),
                            ));
                        }
                        let _ = Self::computed_style_state_from_receiver(this_arg.as_ref())?;
                        Ok(Value::String(String::new()))
                    }
                    "dom_rect_list_item" => {
                        if args.len() > 1 {
                            return Err(Error::ScriptRuntime(
                                "item requires zero or one argument".into(),
                            ));
                        }
                        let Value::Array(values) = this_arg
                            .as_ref()
                            .ok_or_else(|| {
                                Error::ScriptRuntime(
                                    "TypeError: incompatible receiver for DOMRectList.item".into(),
                                )
                            })?
                        else {
                            return Err(Error::ScriptRuntime(
                                "TypeError: incompatible receiver for DOMRectList.item".into(),
                            ));
                        };
                        let values = values.borrow();
                        if !Self::is_dom_rect_list_value(&values) {
                            return Err(Error::ScriptRuntime(
                                "TypeError: incompatible receiver for DOMRectList.item".into(),
                            ));
                        }
                        let index = args
                            .first()
                            .map(|value| match value {
                                Value::Number(number) => *number,
                                Value::Float(number) if number.is_finite() => *number as i64,
                                Value::BigInt(number) => {
                                    number.to_string().parse::<i64>().unwrap_or(0)
                                }
                                other => other.as_string().trim().parse::<i64>().unwrap_or(0),
                            })
                            .unwrap_or(0)
                            .max(0) as usize;
                        Ok(values.get(index).cloned().unwrap_or(Value::Null))
                    }
                    "class_list_add" => {
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        for class_name in args {
                            self.dom.class_add(node, &class_name.as_string())?;
                        }
                        Ok(Value::Undefined)
                    }
                    "class_list_remove" => {
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        for class_name in args {
                            self.dom.class_remove(node, &class_name.as_string())?;
                        }
                        Ok(Value::Undefined)
                    }
                    "class_list_toggle" => {
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        let Some(class_name) = args.first() else {
                            return Err(Error::ScriptRuntime(
                                "DOMTokenList.toggle requires at least one argument".into(),
                            ));
                        };
                        let class_name = class_name.as_string();
                        let toggled = if let Some(force) = args.get(1) {
                            if force.truthy() {
                                self.dom.class_add(node, &class_name)?;
                                true
                            } else {
                                self.dom.class_remove(node, &class_name)?;
                                false
                            }
                        } else {
                            self.dom.class_toggle(node, &class_name)?
                        };
                        Ok(Value::Bool(toggled))
                    }
                    "class_list_contains" => {
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        let Some(class_name) = args.first() else {
                            return Ok(Value::Bool(false));
                        };
                        Ok(Value::Bool(
                            self.dom.class_contains(node, &class_name.as_string())?,
                        ))
                    }
                    "class_list_replace" => {
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        let Some(old_class_name) = args.first() else {
                            return Ok(Value::Bool(false));
                        };
                        let Some(new_class_name) = args.get(1) else {
                            return Ok(Value::Bool(false));
                        };
                        Ok(Value::Bool(self.dom.class_replace(
                            node,
                            &old_class_name.as_string(),
                            &new_class_name.as_string(),
                        )?))
                    }
                    "class_list_item" => {
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        let index = args.first().map(Self::value_to_i64).unwrap_or(0);
                        if index < 0 {
                            return Ok(Value::Null);
                        }
                        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
                        Ok(classes
                            .get(index as usize)
                            .cloned()
                            .map(Value::String)
                            .unwrap_or(Value::Null))
                    }
                    "class_list_for_each" => {
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        if args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "DOMTokenList.forEach requires a callback".into(),
                            ));
                        }
                        let callback = args[0].clone();
                        if !self.is_callable_value(&callback) {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        }
                        let this_value = args.get(1).cloned().unwrap_or(Value::Undefined);
                        let class_list_object = this_arg.clone().unwrap_or(Value::Undefined);
                        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
                        for (index, class_name) in classes.iter().enumerate() {
                            let callback_args = [
                                Value::String(class_name.clone()),
                                Value::Number(index as i64),
                                class_list_object.clone(),
                            ];
                            let _ = self.execute_callable_value_with_this_and_env(
                                &callback,
                                &callback_args,
                                event,
                                caller_env,
                                Some(this_value.clone()),
                            )?;
                        }
                        Ok(Value::Undefined)
                    }
                    "class_list_keys" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "DOMTokenList.keys does not take arguments".into(),
                            ));
                        }
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
                        Ok(self.new_iterator_value(
                            (0..classes.len()).map(|index| Value::Number(index as i64)).collect(),
                        ))
                    }
                    "class_list_values" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "DOMTokenList.values does not take arguments".into(),
                            ));
                        }
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        Ok(self.new_iterator_value(
                            class_tokens(self.dom.attr(node, "class").as_deref())
                                .into_iter()
                                .map(Value::String)
                                .collect(),
                        ))
                    }
                    "class_list_entries" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "DOMTokenList.entries does not take arguments".into(),
                            ));
                        }
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        Ok(self.new_iterator_value(
                            class_tokens(self.dom.attr(node, "class").as_deref())
                                .into_iter()
                                .enumerate()
                                .map(|(index, class_name)| {
                                    Self::new_array_value(vec![
                                        Value::Number(index as i64),
                                        Value::String(class_name),
                                    ])
                                })
                                .collect(),
                        ))
                    }
                    "class_list_to_string" => {
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        Ok(Value::String(
                            class_tokens(self.dom.attr(node, "class").as_deref()).join(" "),
                        ))
                    }
                    "named_node_map_item" => {
                        let (object, _owner) =
                            Self::named_node_map_receiver_object_and_owner(this_arg.as_ref())?;
                        self.eval_named_node_map_member_call(&object, "item", args, event)?
                            .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))
                    }
                    "named_node_map_get_named_item" => {
                        let (object, _owner) =
                            Self::named_node_map_receiver_object_and_owner(this_arg.as_ref())?;
                        self.eval_named_node_map_member_call(
                            &object,
                            "getNamedItem",
                            args,
                            event,
                        )?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))
                    }
                    "named_node_map_set_named_item" => {
                        let (object, _owner) =
                            Self::named_node_map_receiver_object_and_owner(this_arg.as_ref())?;
                        self.eval_named_node_map_member_call(
                            &object,
                            "setNamedItem",
                            args,
                            event,
                        )?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))
                    }
                    "named_node_map_remove_named_item" => {
                        let (object, _owner) =
                            Self::named_node_map_receiver_object_and_owner(this_arg.as_ref())?;
                        self.eval_named_node_map_member_call(
                            &object,
                            "removeNamedItem",
                            args,
                            event,
                        )?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))
                    }
                    "named_node_map_get_named_item_ns" => {
                        let (object, _owner) =
                            Self::named_node_map_receiver_object_and_owner(this_arg.as_ref())?;
                        self.eval_named_node_map_member_call(
                            &object,
                            "getNamedItemNS",
                            args,
                            event,
                        )?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))
                    }
                    "named_node_map_set_named_item_ns" => {
                        let (object, _owner) =
                            Self::named_node_map_receiver_object_and_owner(this_arg.as_ref())?;
                        self.eval_named_node_map_member_call(
                            &object,
                            "setNamedItemNS",
                            args,
                            event,
                        )?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))
                    }
                    "named_node_map_remove_named_item_ns" => {
                        let (object, _owner) =
                            Self::named_node_map_receiver_object_and_owner(this_arg.as_ref())?;
                        self.eval_named_node_map_member_call(
                            &object,
                            "removeNamedItemNS",
                            args,
                            event,
                        )?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))
                    }
                    "named_node_map_for_each" => {
                        if args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "NamedNodeMap.forEach requires a callback".into(),
                            ));
                        }
                        let callback = args[0].clone();
                        if !self.is_callable_value(&callback) {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        }
                        let (object, owner) =
                            Self::named_node_map_receiver_object_and_owner(this_arg.as_ref())?;
                        let this_value = args.get(1).cloned().unwrap_or(Value::Undefined);
                        let attrs = self.named_node_map_entries(owner);
                        for (index, (name, value)) in attrs.iter().enumerate() {
                            let callback_args = [
                                Self::new_attr_object_value(name, value, Some(owner)),
                                Value::Number(index as i64),
                                Value::Object(object.clone()),
                            ];
                            let _ = self.execute_callable_value_with_this_and_env(
                                &callback,
                                &callback_args,
                                event,
                                caller_env,
                                Some(this_value.clone()),
                            )?;
                        }
                        Ok(Value::Undefined)
                    }
                    "named_node_map_keys" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "NamedNodeMap.keys does not take arguments".into(),
                            ));
                        }
                        let (_object, owner) =
                            Self::named_node_map_receiver_object_and_owner(this_arg.as_ref())?;
                        Ok(self.new_iterator_value(
                            (0..self.named_node_map_entries(owner).len())
                                .map(|index| Value::Number(index as i64))
                                .collect(),
                        ))
                    }
                    "named_node_map_values" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "NamedNodeMap.values does not take arguments".into(),
                            ));
                        }
                        let (_object, owner) =
                            Self::named_node_map_receiver_object_and_owner(this_arg.as_ref())?;
                        Ok(self.new_iterator_value(
                            self.named_node_map_entries(owner)
                                .into_iter()
                                .map(|(name, value)| {
                                    Self::new_attr_object_value(&name, &value, Some(owner))
                                })
                                .collect(),
                        ))
                    }
                    "named_node_map_entries" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "NamedNodeMap.entries does not take arguments".into(),
                            ));
                        }
                        let (_object, owner) =
                            Self::named_node_map_receiver_object_and_owner(this_arg.as_ref())?;
                        Ok(self.new_iterator_value(
                            self.named_node_map_entries(owner)
                                .into_iter()
                                .enumerate()
                                .map(|(index, (name, value))| {
                                    Self::new_array_value(vec![
                                        Value::Number(index as i64),
                                        Self::new_attr_object_value(&name, &value, Some(owner)),
                                    ])
                                })
                                .collect(),
                        ))
                    }
                    "worker_constructor" => {
                        if args.is_empty() || args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "Worker constructor requires one or two arguments".into(),
                            ));
                        }
                        let source = self.resolve_worker_script_source(&args[0].as_string())?;
                        self.new_worker_instance_from_script_source(&source)
                    }
                    "data_transfer_constructor" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "DataTransfer constructor does not take arguments".into(),
                            ));
                        }
                        Ok(self.new_data_transfer_object_value("dragstart"))
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

                        Ok(Value::Node(option))
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
                        Ok(Value::Node(audio))
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
                            return Ok(Value::Undefined);
                        }
                        let data = Self::structured_clone_value_with_options(
                            &data,
                            normalized_options.as_ref(),
                        )?;
                        let worker_value = Value::Object(worker.clone());
                        self.queue_worker_message_microtask(
                            &worker,
                            &worker,
                            worker_value,
                            data,
                        );
                        Ok(Value::Undefined)
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
                            return Ok(Value::Undefined);
                        }
                        let data = Self::structured_clone_value_with_options(
                            &data,
                            normalized_options.as_ref(),
                        )?;
                        let worker_global = Self::worker_global_from_object(&worker)?;
                        let worker_global_value = Value::Object(worker_global.clone());
                        self.queue_worker_message_microtask(
                            &worker,
                            &worker_global,
                            worker_global_value,
                            data,
                        );
                        Ok(Value::Undefined)
                    }
                    "worker_terminate" => {
                        let worker = Self::worker_target_from_callable(callable)?;
                        Self::worker_set_terminated_object(&worker, true);
                        Ok(Value::Undefined)
                    }
                    "string_static_from_char_code" => self.eval_string_static_method_from_values(
                        StringStaticMethod::FromCharCode,
                        args,
                    ),
                    "string_static_from_code_point" => self.eval_string_static_method_from_values(
                        StringStaticMethod::FromCodePoint,
                        args,
                    ),
                    "string_static_raw" => {
                        self.eval_string_static_method_from_values(StringStaticMethod::Raw, args)
                    }
                    "object_static_method" => match Self::static_method_name(callable)?.as_str() {
                        "create" => {
                            if args.is_empty() || args.len() > 2 {
                                return Err(Error::ScriptRuntime(
                                    "Object.create requires one or two arguments".into(),
                                ));
                            }
                            self.object_create_value(&args[0], args.get(1))
                        }
                        "assign" => self.eval_object_assign_static_call(args, event),
                        "getOwnPropertyDescriptor" => {
                            if args.len() != 2 {
                                return Err(Error::ScriptRuntime(
                                    "Object.getOwnPropertyDescriptor requires exactly two arguments"
                                        .into(),
                                ));
                            }
                            let key = self.property_key_to_storage_key(&args[1]);
                            self.object_get_own_property_descriptor_value(&args[0], &key)
                        }
                        "defineProperty" => {
                            if args.len() != 3 {
                                return Err(Error::ScriptRuntime(
                                    "Object.defineProperty requires exactly three arguments"
                                        .into(),
                                ));
                            }
                            let key = self.property_key_to_storage_key(&args[1]);
                            self.object_define_property_value(&args[0], &key, &args[2])
                        }
                        "getOwnPropertyNames" => {
                            if args.len() != 1 {
                                return Err(Error::ScriptRuntime(
                                    "Object.getOwnPropertyNames requires exactly one argument"
                                        .into(),
                                ));
                            }
                            self.object_get_own_property_names_value(&args[0])
                        }
                        "getOwnPropertySymbols" => {
                            if args.len() != 1 {
                                return Err(Error::ScriptRuntime(
                                    "Object.getOwnPropertySymbols requires exactly one argument"
                                        .into(),
                                ));
                            }
                            self.object_get_own_property_symbols_value(&args[0])
                        }
                        "keys" => {
                            if args.len() != 1 {
                                return Err(Error::ScriptRuntime(
                                    "Object.keys requires exactly one argument".into(),
                                ));
                            }
                            self.object_keys_value(&args[0])
                        }
                        "values" => {
                            if args.len() != 1 {
                                return Err(Error::ScriptRuntime(
                                    "Object.values requires exactly one argument".into(),
                                ));
                            }
                            self.object_values_value(&args[0])
                        }
                        "entries" => {
                            if args.len() != 1 {
                                return Err(Error::ScriptRuntime(
                                    "Object.entries requires exactly one argument".into(),
                                ));
                            }
                            self.object_entries_value(&args[0])
                        }
                        "fromEntries" => {
                            if args.len() != 1 {
                                return Err(Error::ScriptRuntime(
                                    "Object.fromEntries requires exactly one argument".into(),
                                ));
                            }
                            self.object_from_entries_value(&args[0])
                        }
                        "hasOwn" => {
                            if args.len() != 2 {
                                return Err(Error::ScriptRuntime(
                                    "Object.hasOwn requires exactly two arguments".into(),
                                ));
                            }
                            let key = self.property_key_to_storage_key(&args[1]);
                            self.object_has_own_value(&args[0], &key)
                        }
                        "getPrototypeOf" => {
                            if args.len() != 1 {
                                return Err(Error::ScriptRuntime(
                                    "Object.getPrototypeOf requires exactly one argument".into(),
                                ));
                            }
                            self.object_get_prototype_of_value(&args[0])
                        }
                        "setPrototypeOf" => {
                            if args.len() != 2 {
                                return Err(Error::ScriptRuntime(
                                    "Object.setPrototypeOf requires exactly two arguments".into(),
                                ));
                            }
                            self.object_set_prototype_of_value(&args[0], &args[1])
                        }
                        "freeze" => {
                            if args.len() != 1 {
                                return Err(Error::ScriptRuntime(
                                    "Object.freeze requires exactly one argument".into(),
                                ));
                            }
                            self.object_freeze_value(&args[0])
                        }
                        _ => Err(Error::ScriptRuntime("callback is not a function".into())),
                    },
                    "number_static_method" => {
                        let method = match Self::static_method_name(callable)?.as_str() {
                            "isFinite" => NumberMethod::IsFinite,
                            "isInteger" => NumberMethod::IsInteger,
                            "isNaN" => NumberMethod::IsNaN,
                            "isSafeInteger" => NumberMethod::IsSafeInteger,
                            "parseFloat" => NumberMethod::ParseFloat,
                            "parseInt" => NumberMethod::ParseInt,
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "callback is not a function".into(),
                                ));
                            }
                        };
                        self.eval_number_method_from_values(method, args)
                    }
                    "bigint_static_method" => {
                        let method = match Self::static_method_name(callable)?.as_str() {
                            "asIntN" => BigIntMethod::AsIntN,
                            "asUintN" => BigIntMethod::AsUintN,
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "callback is not a function".into(),
                                ));
                            }
                        };
                        self.eval_bigint_method_from_values(method, args)
                    }
                    "regexp_static_method" => {
                        let method = match Self::static_method_name(callable)?.as_str() {
                            "escape" => RegExpStaticMethod::Escape,
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "callback is not a function".into(),
                                ));
                            }
                        };
                        self.eval_regexp_static_method_from_values(method, args)
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
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "callback is not a function".into(),
                                ));
                            }
                        };
                        self.eval_promise_static_method_from_values(method, args, event)
                    }
                    "array_buffer_static_method" => {
                        match Self::static_method_name(callable)?.as_str() {
                            "isView" => {
                                if args.len() != 1 {
                                    return Err(Error::ScriptRuntime(
                                        "ArrayBuffer.isView requires exactly one argument".into(),
                                    ));
                                }
                                Ok(Value::Bool(matches!(
                                    args.first(),
                                    Some(Value::TypedArray(_))
                                )))
                            }
                            _ => Err(Error::ScriptRuntime("callback is not a function".into())),
                        }
                    }
                    "symbol_static_method" => {
                        let method = match Self::static_method_name(callable)?.as_str() {
                            "for" => SymbolStaticMethod::For,
                            "keyFor" => SymbolStaticMethod::KeyFor,
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "callback is not a function".into(),
                                ));
                            }
                        };
                        self.eval_symbol_static_method_from_values(method, args)
                    }
                    "typed_array_static_method" => {
                        let (kind, method_name) =
                            Self::typed_array_static_method_components(callable)?;
                        let TypedArrayConstructorKind::Concrete(kind) = kind else {
                            return Err(Error::ScriptRuntime("callback is not a function".into()));
                        };
                        let method = match method_name.as_str() {
                            "from" => TypedArrayStaticMethod::From,
                            "of" => TypedArrayStaticMethod::Of,
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "callback is not a function".into(),
                                ));
                            }
                        };
                        self.eval_typed_array_static_method_from_values(kind, method, args)
                    }
                    "reflect_static_method" => match Self::static_method_name(callable)?.as_str() {
                        "set" => {
                            if args.len() != 3 && args.len() != 4 {
                                return Err(Error::ScriptRuntime(
                                    "Reflect.set requires three or four arguments".into(),
                                ));
                            }
                            let receiver = args.get(3).cloned().unwrap_or_else(|| args[0].clone());
                            let key = self.property_key_to_storage_key(&args[1]);
                            Ok(Value::Bool(self.reflect_set_object_property_value(
                                &args[0],
                                &key,
                                args[2].clone(),
                                &receiver,
                                event,
                            )?))
                        }
                        "ownKeys" => {
                            if args.len() != 1 {
                                return Err(Error::ScriptRuntime(
                                    "Reflect.ownKeys requires exactly one argument".into(),
                                ));
                            }
                            self.reflect_own_keys_value(&args[0])
                        }
                        _ => Err(Error::ScriptRuntime("callback is not a function".into())),
                    },
                    "create_image_bitmap" => self.eval_create_image_bitmap_call(args),
                    _ => Err(Error::ScriptRuntime("callback is not a function".into())),
                }
            }
            _ => Err(Error::ScriptRuntime("callback is not a function".into())),
        }
    }

    pub(crate) fn invoke_promise_capability(
        &mut self,
        capability: &PromiseCapabilityFunction,
        args: &[Value],
    ) -> Result<Value> {
        let mut already_called = capability.already_called.borrow_mut();
        if *already_called {
            return Ok(Value::Undefined);
        }
        *already_called = true;
        drop(already_called);

        let value = args.first().cloned().unwrap_or(Value::Undefined);
        if capability.reject {
            self.promise_reject(&capability.promise, value);
            Ok(Value::Undefined)
        } else {
            self.promise_resolve(&capability.promise, value)?;
            Ok(Value::Undefined)
        }
    }

    pub(crate) fn is_primitive_value(value: &Value) -> bool {
        matches!(
            value,
            Value::String(_)
                | Value::Bool(_)
                | Value::Number(_)
                | Value::Float(_)
                | Value::BigInt(_)
                | Value::Null
                | Value::Undefined
                | Value::Symbol(_)
        )
    }

    pub(crate) fn apply_constructor_instance_initializers_by_id(
        &mut self,
        constructor_id: usize,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        let Some(initializers) = self
            .script_runtime
            .constructor_instance_initializers
            .get(&constructor_id)
            .cloned()
        else {
            return Ok(());
        };

        let this_value = env.get("this").cloned().unwrap_or(Value::Undefined);
        for initializer in &initializers {
            self.apply_constructor_instance_initializer_to_receiver(
                initializer,
                &this_value,
                env,
                event_param,
                event,
            )?;
        }
        Ok(())
    }

    pub(crate) fn initialize_current_constructor_instance_fields(
        &mut self,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        let Some(constructor_id) = self.script_runtime.constructor_call_stack.last().copied()
        else {
            return Ok(());
        };
        let Some(already_initialized) = self
            .script_runtime
            .constructor_instance_initialized_stack
            .last()
            .copied()
        else {
            return Ok(());
        };
        if already_initialized {
            return Err(Error::ScriptRuntime(
                "super() has already been called for this constructor".into(),
            ));
        }
        self.apply_constructor_instance_initializers_by_id(
            constructor_id,
            env,
            event_param,
            event,
        )?;
        if let Some(last) = self
            .script_runtime
            .constructor_instance_initialized_stack
            .last_mut()
        {
            *last = true;
        }
        Ok(())
    }

    pub(crate) fn bind_handler_params(
        &mut self,
        handler: &ScriptHandler,
        args: &[Value],
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        for (index, param) in handler.params.iter().enumerate() {
            if param.is_rest {
                let rest = if index < args.len() {
                    args[index..].to_vec()
                } else {
                    Vec::new()
                };
                env.insert(param.name.clone(), Self::new_array_value(rest));
                self.set_const_binding(env, &param.name, false);
                continue;
            }

            let provided = args.get(index).cloned().unwrap_or(Value::Undefined);
            let value = if matches!(provided, Value::Undefined) {
                if let Some(default_expr) = &param.default {
                    self.eval_expr(default_expr, env, event_param, event)?
                } else {
                    Value::Undefined
                }
            } else {
                provided
            };
            env.insert(param.name.clone(), value);
            self.set_const_binding(env, &param.name, false);
        }
        Ok(())
    }

    pub(crate) fn execute_function_call(
        &mut self,
        function: Rc<FunctionValue>,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
        this_arg: Option<Value>,
        new_target: Option<Value>,
        sync_event_to: Option<&mut EventState>,
    ) -> Result<Value> {
        let run = |this: &mut Self,
                   caller_env: Option<&HashMap<String, Value>>,
                   this_arg: Option<Value>,
                   new_target: Option<Value>,
                   sync_event_to: Option<&mut EventState>|
         -> Result<Value> {
            let pending_scope_start =
                this.push_pending_function_decl_scopes(&function.captured_pending_function_decls);

            let private_bindings = this
                .script_runtime
                .function_private_bindings
                .get(&function.function_id)
                .cloned();
            if let Some(bindings) = private_bindings.clone() {
                this.script_runtime.private_binding_stack.push(bindings);
            }

            let is_constructor_call = function.is_class_constructor;
            if is_constructor_call {
                this.script_runtime
                    .constructor_call_stack
                    .push(function.function_id);
                let initialized = function.class_super_constructor.is_none();
                this.script_runtime
                    .constructor_instance_initialized_stack
                    .push(initialized);
            }

            let listener_capture_scope_start = this.script_runtime.listener_capture_env_stack.len();
            let captured_env_seed =
                (!function.global_scope).then(|| Self::function_capture_snapshot(&function));
            let shared_env_frame_start = (!function.global_scope).then(|| {
                this.push_shared_listener_capture_env_frame_with_names(
                    function.captured_env.clone(),
                    false,
                    Some(function.captured_names.clone()),
                )
            });

            let result = this.with_isolated_loop_control_scope(|this| {
                (|| -> Result<Value> {
                    let captured_env_before_call = captured_env_seed.clone();
                    let mut call_env = if function.global_scope {
                        this.script_runtime.env.to_map()
                    } else {
                        captured_env_before_call
                            .as_ref()
                            .cloned()
                            .unwrap_or_default()
                    };
                    Self::isolate_execution_const_bindings(&mut call_env);
                    for name in &function.local_bindings {
                        call_env.remove(name);
                    }
                    call_env.remove(INTERNAL_RETURN_SLOT);
                    call_env.remove(INTERNAL_LOCAL_BINDINGS_KEY);
                    let mut global_sync_keys = HashSet::new();
                    let caller_view = caller_env;
                    let caller_scope_start =
                        caller_view.map(Self::pending_listener_capture_scope_start);
                    for name in &function.captured_names {
                        if Self::is_internal_env_key(&name)
                            || function.local_bindings.contains(name.as_str())
                            || matches!(name.as_str(), "this" | "arguments")
                            || call_env.contains_key(name)
                        {
                            continue;
                        }
                        if let Some(caller_scope_start) = caller_scope_start {
                            if let Some(pending) = this.resolve_listener_capture_pending_value_from(
                                caller_scope_start,
                                name,
                            ) {
                                if let Some(value) = pending {
                                    call_env.insert(name.clone(), value);
                                } else {
                                    call_env.remove(name);
                                }
                                continue;
                            }
                        }
                        if let Some(value) = caller_view.and_then(|env| env.get(name)).cloned() {
                            call_env.insert(name.clone(), value);
                            continue;
                        }
                        if let Some(value) = this.resolve_runtime_global_identifier(name) {
                            call_env.insert(name.clone(), value);
                            global_sync_keys.insert(name.clone());
                        }
                    }
                    if let Some(caller_view) = caller_view {
                        let lexical_names = Self::env_top_level_lexical_binding_names(&call_env);
                        for name in lexical_names {
                            if Self::is_internal_env_key(&name)
                                || function.local_bindings.contains(name.as_str())
                                || function.captured_names.contains(&name)
                                || matches!(name.as_str(), "this" | "arguments")
                                || Self::env_has_local_binding(caller_view, &name)
                            {
                                continue;
                            }
                            if let Some(value) = caller_view.get(&name).cloned() {
                                call_env.insert(name, value);
                                continue;
                            }
                            if let Some(caller_scope_start) = caller_scope_start {
                                if let Some(pending) = this
                                    .resolve_listener_capture_pending_value_from(
                                        caller_scope_start,
                                        &name,
                                    )
                                {
                                    if let Some(value) = pending {
                                        call_env.insert(name, value);
                                    } else {
                                        call_env.remove(&name);
                                    }
                                    continue;
                                }
                            }
                        }
                    }
                    let scope_depth = Self::env_scope_depth(&call_env);
                    call_env.insert(
                        INTERNAL_SCOPE_DEPTH_KEY.to_string(),
                        Value::Number(scope_depth.saturating_add(1)),
                    );
                    call_env.insert(
                        INTERNAL_PENDING_SCOPE_START_KEY.to_string(),
                        Value::Number(listener_capture_scope_start as i64),
                    );
                    if function.is_arrow {
                        if !call_env.contains_key("this") {
                            call_env.insert("this".to_string(), Value::Undefined);
                            this.set_const_binding(&mut call_env, "this", false);
                        }
                    } else {
                        call_env.insert("this".to_string(), this_arg.unwrap_or(Value::Undefined));
                        this.set_const_binding(&mut call_env, "this", false);
                        call_env.insert(
                            INTERNAL_NEW_TARGET_KEY.to_string(),
                            new_target.unwrap_or(Value::Undefined),
                        );
                        let arguments_value = Self::new_array_value(args.to_vec());
                        if let Value::Array(arguments) = &arguments_value {
                            Self::object_set_entry(
                                &mut arguments.borrow_mut().properties,
                                "callee".to_string(),
                                Value::Function(function.clone()),
                            );
                        }
                        call_env.insert("arguments".to_string(), arguments_value);
                        this.set_const_binding(&mut call_env, "arguments", false);
                        if Self::has_simple_parameter_list(&function.handler) {
                            let mut bindings = Vec::with_capacity(args.len());
                            for index in 0..args.len() {
                                let binding = function
                                    .handler
                                    .params
                                    .get(index)
                                    .map(|param| Value::String(param.name.clone()))
                                    .unwrap_or(Value::Undefined);
                                bindings.push(binding);
                            }
                            call_env.insert(
                                INTERNAL_ARGUMENTS_PARAM_BINDINGS_KEY.to_string(),
                                Self::new_array_value(bindings),
                            );
                        }
                    }
                    if let Some(expression_name) = function.expression_name.as_ref() {
                        call_env.insert(expression_name.clone(), Value::Function(function.clone()));
                        this.set_const_binding(&mut call_env, expression_name, true);
                    }
                    if !function.local_bindings.is_empty() {
                        let mut local_bindings =
                            function.local_bindings.iter().cloned().collect::<Vec<_>>();
                        local_bindings.sort();
                        call_env.insert(
                            INTERNAL_LOCAL_BINDINGS_KEY.to_string(),
                            Self::new_array_value(
                                local_bindings.into_iter().map(Value::String).collect(),
                            ),
                        );
                    }
                    if let Some(super_constructor) = function.class_super_constructor.clone() {
                        call_env.insert(
                            INTERNAL_CLASS_SUPER_CONSTRUCTOR_KEY.to_string(),
                            super_constructor,
                        );
                    }
                    if let Some(super_prototype) = function.class_super_prototype.clone() {
                        call_env.insert(
                            INTERNAL_CLASS_SUPER_PROTOTYPE_KEY.to_string(),
                            super_prototype,
                        );
                    } else if function.is_method {
                        let inferred_super = match call_env.get("this").cloned() {
                            Some(Value::Object(object)) => Self::object_get_entry(
                                &object.borrow(),
                                INTERNAL_OBJECT_PROTOTYPE_KEY,
                            ),
                            Some(Value::Function(function_value)) => {
                                function_value.class_super_constructor.clone()
                            }
                            _ => None,
                        };
                        if let Some(super_prototype) = inferred_super {
                            call_env.insert(
                                INTERNAL_CLASS_SUPER_PROTOTYPE_KEY.to_string(),
                                super_prototype,
                            );
                        }
                    }
                    for name in &function.captured_global_names {
                        if Self::is_internal_env_key(&name)
                            || function.local_bindings.contains(name)
                            || name == "this"
                            || name == "arguments"
                        {
                            continue;
                        }
                        global_sync_keys.insert(name.clone());
                        if let Some(global_value) = this.resolve_runtime_global_identifier(name) {
                            call_env.insert(name.clone(), global_value);
                        } else if !call_env.contains_key(name) {
                            if let Some(value) = caller_view.and_then(|env| env.get(name)).cloned()
                            {
                                call_env.insert(name.clone(), value);
                            }
                        }
                    }
                    if !global_sync_keys.is_empty() {
                        let mut sync_names = global_sync_keys.iter().cloned().collect::<Vec<_>>();
                        sync_names.sort();
                        call_env.insert(
                            INTERNAL_GLOBAL_SYNC_NAMES_KEY.to_string(),
                            Self::new_array_value(
                                sync_names.into_iter().map(Value::String).collect(),
                            ),
                        );
                    }
                    let mut global_values_before_call = HashMap::new();
                    for name in &global_sync_keys {
                        if let Some(value) = this.script_runtime.env.get(name).cloned() {
                            global_values_before_call.insert(name.clone(), value);
                        }
                    }
                    let mut call_event = event.clone();
                    let event_param = sync_event_to
                        .as_ref()
                        .and_then(|_| function.handler.first_event_param())
                        .map(str::to_string);
                    this.script_runtime
                        .listener_capture_env_stack
                        .push(ListenerCaptureFrame {
                            ..ListenerCaptureFrame::default()
                        });
                    let bind_result = (|| -> Result<()> {
                        this.project_pending_listener_capture_env_updates(&mut call_env);
                        this.bind_handler_params(
                            &function.handler,
                            args,
                            &mut call_env,
                            &event_param,
                            &call_event,
                        )?;
                        Ok(())
                    })();
                    this.script_runtime.listener_capture_env_stack.pop();
                    bind_result?;
                    if function.is_class_constructor && function.class_super_constructor.is_none() {
                        this.apply_constructor_instance_initializers_by_id(
                            function.function_id,
                            &call_env,
                            &event_param,
                            &call_event,
                        )?;
                    }
                    let param_names = function
                        .handler
                        .params
                        .iter()
                        .map(|param| param.name.clone())
                        .collect::<HashSet<_>>();
                    this.ensure_no_direct_let_redeclarations(
                        &function.handler.stmts,
                        &param_names,
                    )?;
                    let yield_collector = if function.is_generator {
                        Some(Rc::new(RefCell::new(Vec::new())))
                    } else {
                        None
                    };
                    if let Some(yields) = &yield_collector {
                        this.script_runtime
                            .generator_yield_stack
                            .push(yields.clone());
                    }
                    let mut non_tdz_shadowed =
                        Self::collect_var_declared_names(&function.handler.stmts);
                    non_tdz_shadowed.extend(
                        function
                            .handler
                            .params
                            .iter()
                            .map(|param| param.name.clone()),
                    );
                    non_tdz_shadowed
                        .extend(Self::collect_function_decls(&function.handler.stmts).into_keys());
                    if let Some(expression_name) = function.expression_name.as_ref() {
                        non_tdz_shadowed.insert(expression_name.clone());
                    }
                    if let Some(caller_view) = caller_view {
                        non_tdz_shadowed.extend(
                            Self::env_local_or_lexical_binding_names(caller_view)
                                .into_iter()
                                .filter(|name| {
                                    !function.captured_names.contains(name)
                                        && !matches!(name.as_str(), "this" | "arguments")
                                }),
                        );
                    }

                    let pushed_non_tdz_scope = !non_tdz_shadowed.is_empty();
                    if pushed_non_tdz_scope {
                        this.script_runtime.tdz_scope_stack.push(TdzScopeFrame {
                            declared: non_tdz_shadowed,
                            pending: HashSet::new(),
                        });
                    }
                    let current_scope_pending_updates_before = this
                        .listener_capture_pending_updates_snapshot_from(
                            Self::pending_listener_capture_scope_start(&call_env),
                        );
                    let mut pending_async_suspend = None;
                    let flow_result = if function.is_async && !function.is_generator {
                        if let Some((await_index, await_expr, resume_kind)) =
                            Self::first_suspendable_top_level_await(&function.handler.stmts)
                        {
                            let prefix_flow = this.execute_stmts_with_pending_scope(
                                &function.handler.stmts[..await_index],
                                &event_param,
                                &mut call_event,
                                &mut call_env,
                                false,
                            )?;
                            Ok(match prefix_flow {
                                ExecFlow::Continue => match this.eval_top_level_await_expr(
                                    &await_expr,
                                    &mut call_env,
                                    &event_param,
                                    &call_event,
                                )? {
                                    TopLevelAwaitOutcome::Resolved(awaited_value) => {
                                        match &resume_kind {
                                            TopLevelAwaitResumeKind::Ignore => {}
                                            TopLevelAwaitResumeKind::Declare { name, kind } => {
                                                call_env.insert(name.clone(), awaited_value);
                                                this.set_const_binding(
                                                    &mut call_env,
                                                    &name,
                                                    matches!(kind, VarDeclKind::Const),
                                                );
                                            }
                                            TopLevelAwaitResumeKind::Assign { name } => {
                                                call_env.insert(name.clone(), awaited_value);
                                            }
                                            TopLevelAwaitResumeKind::Return => {
                                                call_env.insert(
                                                    INTERNAL_RETURN_SLOT.to_string(),
                                                    awaited_value,
                                                );
                                            }
                                        };
                                        if matches!(resume_kind, TopLevelAwaitResumeKind::Return) {
                                            ExecFlow::Return
                                        } else {
                                            this.execute_stmts_with_pending_scope(
                                                &function.handler.stmts[await_index + 1..],
                                                &event_param,
                                                &mut call_event,
                                                &mut call_env,
                                                false,
                                            )?
                                        }
                                    }
                                    TopLevelAwaitOutcome::Pending(awaited_promise) => {
                                        let continuation_handler =
                                            Self::build_top_level_await_continuation_handler(
                                                &resume_kind,
                                                &function.handler.stmts[await_index + 1..],
                                            );
                                        let continuation = this.make_function_value_with_kind(
                                            continuation_handler,
                                            &call_env,
                                            false,
                                            true,
                                            false,
                                            true,
                                            false,
                                            false,
                                            None,
                                            None,
                                        );
                                        pending_async_suspend = Some(PendingAsyncFunctionSuspend {
                                            awaited_promise,
                                            continuation,
                                        });
                                        ExecFlow::Continue
                                    }
                                },
                                other => other,
                            })
                        } else {
                            this.execute_stmts_with_pending_scope(
                                &function.handler.stmts,
                                &event_param,
                                &mut call_event,
                                &mut call_env,
                                false,
                            )
                        }
                    } else {
                        this.execute_stmts_with_pending_scope(
                            &function.handler.stmts,
                            &event_param,
                            &mut call_event,
                            &mut call_env,
                            false,
                        )
                    };
                    if pushed_non_tdz_scope {
                        this.script_runtime.tdz_scope_stack.pop();
                    }
                    if yield_collector.is_some() {
                        let _ = this.script_runtime.generator_yield_stack.pop();
                    }
                    let mut deferred_error = None;
                    let flow = match flow_result {
                        Ok(flow) => flow,
                        Err(Error::ScriptRuntime(msg))
                            if function.is_generator
                                && msg == INTERNAL_GENERATOR_YIELD_LIMIT_REACHED =>
                        {
                            ExecFlow::Continue
                        }
                        Err(err) => {
                            deferred_error = Some(err);
                            ExecFlow::Continue
                        }
                    };
                    let current_scope_pending_updates_after = this
                        .listener_capture_pending_updates_snapshot_from(
                            Self::pending_listener_capture_scope_start(&call_env),
                        );
                    this.apply_pending_listener_capture_env_updates(&mut call_env);
                    let generator_yields = yield_collector
                        .as_ref()
                        .map(|values| values.borrow().clone())
                        .unwrap_or_default();
                    let generator_return_value = if matches!(flow, ExecFlow::Return) {
                        call_env
                            .get(INTERNAL_RETURN_SLOT)
                            .cloned()
                            .unwrap_or(Value::Undefined)
                    } else {
                        Value::Undefined
                    };
                    if let Some(event_state) = sync_event_to {
                        Self::sync_event_argument_back_to_state(
                            event_state,
                            &call_env,
                            event_param.as_deref(),
                        );
                    }
                    let caller_has_explicit_binding = |name: &str| {
                        caller_view.is_some_and(|env| Self::env_has_explicit_binding(env, name))
                    };
                    let effective_call_binding = |this: &Self, name: &str| {
                        let current_scope_pending = this
                            .resolve_listener_capture_pending_value_from(
                                Self::pending_listener_capture_scope_start(&call_env),
                                name,
                            );
                        if let Some(pending) = current_scope_pending {
                            return pending;
                        }
                        if let Some(value) = call_env.get(name).cloned() {
                            return Some(value);
                        }
                        if let Some(pending) = this.resolve_listener_capture_pending_value(name) {
                            return pending;
                        }
                        None
                    };
                    for name in &global_sync_keys {
                        if Self::is_internal_env_key(name)
                            || function.local_bindings.contains(name)
                            || name == "this"
                            || name == "arguments"
                        {
                            continue;
                        }
                        let before = global_values_before_call.get(name);
                        let global_after = this.script_runtime.env.get(name).cloned();
                        let call_after = effective_call_binding(this, name);
                        let global_changed = match (before, global_after.as_ref()) {
                            (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                            (None, Some(_)) => true,
                            (Some(_), None) => true,
                            (None, None) => false,
                        };
                        let call_changed = match (before, call_after.as_ref()) {
                            (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                            (None, Some(_)) => true,
                            (Some(_), None) => true,
                            (None, None) => false,
                        };
                        if global_changed && !call_changed {
                            continue;
                        }
                        if let Some(next) = call_after {
                            this.script_runtime.env.insert(name.clone(), next);
                            if caller_has_explicit_binding(name) {
                                this.script_runtime.expression_env_overrides.remove(name);
                            } else if caller_view.is_some() {
                                this.script_runtime.expression_env_overrides.insert(
                                    name.clone(),
                                    Some(this.script_runtime.env[name].clone()),
                                );
                            }
                            if let Some(value) = this.script_runtime.env.get(name).cloned() {
                                this.sync_scheduled_task_captures_for_binding(name, &value);
                            }
                        }
                    }
                    let mut scheduled_capture_updates = Vec::new();
                    if !function.global_scope {
                        let captured_env_after_call = Self::function_capture_snapshot(&function);
                        let mut captured_env = function.captured_env.borrow_mut();
                        let captured_env_before_call = captured_env_before_call
                            .as_ref()
                            .expect("non-global functions always snapshot their capture env");
                        for name in &function.captured_names {
                            if matches!(name.as_str(), "this" | "arguments") {
                                continue;
                            }
                            let before = captured_env_before_call.get(name);
                            let call_after_from_env = effective_call_binding(this, name);
                            let call_after_from_shared = captured_env_after_call.get(name).cloned();
                            let call_after = match (
                                before,
                                call_after_from_env.as_ref(),
                                call_after_from_shared.as_ref(),
                            ) {
                                (Some(prev), Some(env_next), Some(shared_next))
                                    if this.strict_equal(prev, env_next)
                                        && !this.strict_equal(prev, shared_next) =>
                                {
                                    Some(shared_next.clone())
                                }
                                (Some(_), None, Some(shared_next)) => Some(shared_next.clone()),
                                _ => call_after_from_env.or(call_after_from_shared),
                            };
                            let after = call_after.as_ref();
                            let changed = match (before, after) {
                                (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                                (None, Some(_)) => true,
                                (Some(_), None) => true,
                                (None, None) => false,
                            };
                            if !changed {
                                continue;
                            }
                            if let Some(next) = call_after {
                                captured_env.insert(name.clone(), next.clone());
                                if caller_has_explicit_binding(name) {
                                    if let Some(parent_index) = shared_env_frame_start
                                        .and_then(|start| start.checked_sub(1))
                                    {
                                        if let Some(parent_frame) = this
                                            .script_runtime
                                            .listener_capture_env_stack
                                            .get_mut(parent_index)
                                        {
                                            parent_frame
                                                .pending_env_updates
                                                .insert(name.clone(), Some(next.clone()));
                                        }
                                    }
                                }
                                if caller_view.is_some() {
                                    this.script_runtime
                                        .expression_env_overrides
                                        .insert(name.clone(), Some(next.clone()));
                                }
                                this.queue_listener_capture_env_update_for_shared_env(
                                    &function.captured_env,
                                    name.clone(),
                                    Some(next.clone()),
                                );
                                scheduled_capture_updates.push((name.clone(), next));
                            } else {
                                captured_env.remove(name);
                                if caller_has_explicit_binding(name) {
                                    if let Some(parent_index) = shared_env_frame_start
                                        .and_then(|start| start.checked_sub(1))
                                    {
                                        if let Some(parent_frame) = this
                                            .script_runtime
                                            .listener_capture_env_stack
                                            .get_mut(parent_index)
                                        {
                                            parent_frame
                                                .pending_env_updates
                                                .insert(name.clone(), None);
                                        }
                                    }
                                }
                                if caller_view.is_some() {
                                    this.script_runtime
                                        .expression_env_overrides
                                        .insert(name.clone(), None);
                                }
                                this.queue_listener_capture_env_update_for_shared_env(
                                    &function.captured_env,
                                    name.clone(),
                                    None,
                                );
                            }
                        }
                    }
                    let mut caller_visible_names = current_scope_pending_updates_before
                        .keys()
                        .chain(current_scope_pending_updates_after.keys())
                        .filter_map(|name| Self::event_sync_pending_marker_name(name))
                        .map(str::to_string)
                        .collect::<Vec<_>>();
                    caller_visible_names.sort();
                    caller_visible_names.dedup();
                    for name in caller_visible_names {
                        let marker_key = Self::event_sync_pending_marker_key(&name);
                        let pending_before = current_scope_pending_updates_before.get(&marker_key);
                        let pending_after = current_scope_pending_updates_after.get(&marker_key);
                        let pending_changed = match (pending_before, pending_after) {
                            (Some(Some(prev)), Some(Some(next))) => !this.strict_equal(prev, next),
                            (Some(None), Some(None)) | (None, None) => false,
                            (Some(_), Some(_)) | (Some(_), None) | (None, Some(_)) => true,
                        };
                        if !pending_changed
                            || Self::is_internal_env_key(&name)
                            || function.local_bindings.contains(&name)
                            || matches!(name.as_str(), "this" | "arguments")
                            || global_sync_keys.contains(&name)
                            || function.captured_names.contains(&name)
                            || !caller_has_explicit_binding(&name)
                        {
                            continue;
                        }
                        let before = caller_view.and_then(|env| env.get(&name));
                        let call_after = pending_after
                            .cloned()
                            .unwrap_or_else(|| effective_call_binding(this, &name));
                        let after = call_after.as_ref();
                        let changed = match (before, after) {
                            (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                            (None, Some(_)) => true,
                            (Some(_), None) => true,
                            (None, None) => false,
                        };
                        if !changed {
                            continue;
                        }
                        if let Some(parent_index) =
                            shared_env_frame_start.and_then(|start| start.checked_sub(1))
                        {
                            if let Some(parent_frame) = this
                                .script_runtime
                                .listener_capture_env_stack
                                .get_mut(parent_index)
                            {
                                parent_frame
                                    .pending_env_updates
                                    .insert(name.clone(), call_after.clone());
                            }
                        }
                        if caller_view.is_some() {
                            this.script_runtime
                                .expression_env_overrides
                                .insert(name.clone(), call_after.clone());
                        }
                        if let Some(value) = call_after {
                            this.sync_scheduled_task_captures_for_binding(&name, &value);
                        }
                    }
                    for (name, value) in scheduled_capture_updates {
                        this.sync_scheduled_task_captures_for_binding(&name, &value);
                    }
                    if let Some(err) = deferred_error {
                        return Err(err);
                    }
                    if let Some(suspend) = pending_async_suspend {
                        this.script_runtime.pending_async_function_suspend = Some(suspend);
                        return Err(Error::ScriptRuntime(
                            INTERNAL_ASYNC_FUNCTION_SUSPENDED.into(),
                        ));
                    }
                    if function.is_generator {
                        if function.is_async {
                            return Ok(this.new_async_generator_value(generator_yields));
                        }
                        return Ok(
                            this.new_generator_value(generator_yields, generator_return_value)
                        );
                    }
                    match flow {
                        ExecFlow::Continue => Ok(Value::Undefined),
                        ExecFlow::Break(label) => Err(Self::break_flow_error(&label)),
                        ExecFlow::ContinueLoop(label) => Err(Self::continue_flow_error(&label)),
                        ExecFlow::Return => Ok(call_env
                            .remove(INTERNAL_RETURN_SLOT)
                            .unwrap_or(Value::Undefined)),
                    }
                })()
            });
            if let Some(start) = shared_env_frame_start {
                this.discard_event_sync_pending_updates_from_frames(start);
                this.discard_pending_listener_updates_from_frames(start, &function.local_bindings);
                this.restore_listener_capture_env_stack(start);
            }

            if private_bindings.is_some() {
                this.script_runtime.private_binding_stack.pop();
            }
            if is_constructor_call {
                this.script_runtime.constructor_call_stack.pop();
                this.script_runtime
                    .constructor_instance_initialized_stack
                    .pop();
            }
            this.restore_pending_function_decl_scopes(pending_scope_start);
            result
        };

        if function.is_async && !function.is_generator {
            let promise = self.new_pending_promise();
            match run(
                self,
                caller_env,
                this_arg.clone(),
                new_target.clone(),
                sync_event_to,
            ) {
                Ok(value) => {
                    if let Err(err) = self.promise_resolve(&promise, value) {
                        self.promise_reject(&promise, Self::promise_error_reason(err));
                    }
                }
                Err(Error::ScriptRuntime(msg)) if msg == INTERNAL_ASYNC_FUNCTION_SUSPENDED => {
                    if let Some(suspend) = self.script_runtime.pending_async_function_suspend.take()
                    {
                        self.promise_add_reaction(
                            &suspend.awaited_promise,
                            PromiseReactionKind::Then {
                                on_fulfilled: Some(suspend.continuation),
                                on_rejected: None,
                                result: promise.clone(),
                            },
                        );
                    } else {
                        self.promise_reject(
                            &promise,
                            Value::String("async function suspended without continuation".into()),
                        );
                    }
                }
                Err(err) => self.promise_reject(&promise, Self::promise_error_reason(err)),
            }
            Ok(Value::Promise(promise))
        } else {
            run(self, caller_env, this_arg, new_target, sync_event_to)
        }
    }
}
