use super::*;

const IMAGE_DATA_MAX_DEFAULT_ELEMENTS: usize = 1_000_000;

impl Harness {
    fn has_simple_parameter_list(handler: &ScriptHandler) -> bool {
        handler.params.iter().all(|param| {
            !param.is_rest
                && param.default.is_none()
                && !param.name.starts_with("__bt_callback_arg_")
        })
    }

    fn make_function_value_with_kind(
        &mut self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        is_async: bool,
        is_generator: bool,
        is_arrow: bool,
        is_method: bool,
        is_class_constructor: bool,
        class_super_constructor: Option<Value>,
        class_super_prototype: Option<Value>,
    ) -> Value {
        let local_bindings = Self::collect_function_scope_bindings(&handler);
        let scope_depth = Self::env_scope_depth(env);
        let captured_pending_function_decls = self.script_runtime.pending_function_decls.clone();
        let captured_env = if global_scope {
            Rc::new(RefCell::new(self.script_runtime.env.share()))
        } else {
            let captured_env = self.ensure_listener_capture_env();
            *captured_env.borrow_mut() = ScriptEnv::from_snapshot(env);
            captured_env
        };
        let captured_env_snapshot = captured_env.borrow();
        let mut captured_global_names = HashSet::new();
        for (name, value) in captured_env_snapshot.iter() {
            if Self::is_internal_env_key(name) || name == INTERNAL_RETURN_SLOT {
                continue;
            }
            if scope_depth == 0 {
                captured_global_names.insert(name.clone());
                continue;
            }
            let Some(global_value) = self.script_runtime.env.get(name) else {
                continue;
            };
            if global_scope || self.strict_equal(global_value, value) {
                captured_global_names.insert(name.clone());
            }
        }
        drop(captured_env_snapshot);
        let function_id = self.script_runtime.allocate_function_id();
        if !self.script_runtime.private_binding_stack.is_empty() {
            let mut captured_private_bindings = HashMap::new();
            for bindings in &self.script_runtime.private_binding_stack {
                for (name, binding) in bindings {
                    captured_private_bindings.insert(name.clone(), binding.clone());
                }
            }
            self.script_runtime
                .function_private_bindings
                .insert(function_id, captured_private_bindings);
        }
        let function = Rc::new(FunctionValue {
            function_id,
            handler,
            expression_name: None,
            captured_env,
            captured_pending_function_decls,
            captured_global_names,
            local_bindings,
            prototype_object: Rc::new(RefCell::new(ObjectValue::default())),
            global_scope,
            is_async,
            is_generator,
            is_arrow,
            is_method,
            is_class_constructor,
            class_super_constructor,
            class_super_prototype,
        });
        self.script_runtime
            .function_registry
            .insert(function_id, function.clone());
        Value::Function(function)
    }

    pub(crate) fn make_function_value(
        &mut self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        is_async: bool,
        is_generator: bool,
        is_arrow: bool,
        is_method: bool,
    ) -> Value {
        self.make_function_value_with_kind(
            handler,
            env,
            global_scope,
            is_async,
            is_generator,
            is_arrow,
            is_method,
            false,
            None,
            None,
        )
    }

    pub(crate) fn make_function_value_with_super(
        &mut self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        is_async: bool,
        is_generator: bool,
        is_arrow: bool,
        is_method: bool,
        class_super_constructor: Option<Value>,
        class_super_prototype: Option<Value>,
    ) -> Value {
        self.make_function_value_with_kind(
            handler,
            env,
            global_scope,
            is_async,
            is_generator,
            is_arrow,
            is_method,
            false,
            class_super_constructor,
            class_super_prototype,
        )
    }

    pub(crate) fn make_class_constructor_value_with_super(
        &mut self,
        handler: ScriptHandler,
        env: &HashMap<String, Value>,
        global_scope: bool,
        class_super_constructor: Option<Value>,
        class_super_prototype: Option<Value>,
    ) -> Value {
        self.make_function_value_with_kind(
            handler,
            env,
            global_scope,
            false,
            false,
            false,
            false,
            true,
            class_super_constructor,
            class_super_prototype,
        )
    }

    pub(crate) fn is_callable_value(&self, value: &Value) -> bool {
        matches!(
            value,
            Value::Function(_) | Value::PromiseCapability(_) | Value::StringConstructor
        ) || Self::callable_kind_from_value(value).is_some()
    }

    fn callable_receiver_from_this_arg(
        &self,
        this_arg: Option<Value>,
        method: &str,
    ) -> Result<Value> {
        let Some(target) = this_arg else {
            return Err(Error::ScriptRuntime(format!(
                "Function.prototype.{method} called on non-callable value"
            )));
        };
        if !self.is_callable_value(&target) {
            return Err(Error::ScriptRuntime(format!(
                "Function.prototype.{method} called on non-callable value"
            )));
        }
        Ok(target)
    }

    fn receiver_builtin_callable_components(callable: &Value) -> Result<(String, String)> {
        let Value::Object(entries) = callable else {
            return Err(Error::ScriptRuntime(
                "builtin method has invalid internal state".into(),
            ));
        };
        let entries = entries.borrow();
        let family = match Self::object_get_entry(&entries, "__bt_receiver_builtin_family") {
            Some(Value::String(family)) => family,
            _ => {
                return Err(Error::ScriptRuntime(
                    "builtin method has invalid internal state".into(),
                ));
            }
        };
        let member = match Self::object_get_entry(&entries, "__bt_receiver_builtin_member") {
            Some(Value::String(member)) => member,
            _ => {
                return Err(Error::ScriptRuntime(
                    "builtin method has invalid internal state".into(),
                ));
            }
        };
        Ok((family, member))
    }

    fn static_method_name(callable: &Value) -> Result<String> {
        let Value::Object(entries) = callable else {
            return Err(Error::ScriptRuntime(
                "builtin method has invalid internal state".into(),
            ));
        };
        let entries = entries.borrow();
        match Self::object_get_entry(&entries, INTERNAL_STATIC_METHOD_NAME_KEY) {
            Some(Value::String(method)) => Ok(method),
            _ => Err(Error::ScriptRuntime(
                "builtin method has invalid internal state".into(),
            )),
        }
    }

    fn typed_array_static_method_components(
        callable: &Value,
    ) -> Result<(TypedArrayConstructorKind, String)> {
        let Value::Object(entries) = callable else {
            return Err(Error::ScriptRuntime(
                "builtin method has invalid internal state".into(),
            ));
        };
        let entries = entries.borrow();
        let kind = match Self::object_get_entry(&entries, INTERNAL_STATIC_TYPED_ARRAY_KIND_KEY) {
            Some(Value::TypedArrayConstructor(kind)) => kind,
            _ => {
                return Err(Error::ScriptRuntime(
                    "builtin method has invalid internal state".into(),
                ));
            }
        };
        let method = match Self::object_get_entry(&entries, INTERNAL_STATIC_METHOD_NAME_KEY) {
            Some(Value::String(method)) => method,
            _ => {
                return Err(Error::ScriptRuntime(
                    "builtin method has invalid internal state".into(),
                ));
            }
        };
        Ok((kind, method))
    }

    fn incompatible_receiver_error(family: &str) -> Error {
        let label = match family {
            "array" => "Array",
            "map" => "Map",
            "node_list" => "NodeList",
            "weak_map" => "WeakMap",
            "set" => "Set",
            "weak_set" => "WeakSet",
            "location" => "Location",
            "string" => "String",
            "typed_array" => "TypedArray",
            "boolean" => "Boolean",
            "number" => "Number",
            "bigint" => "BigInt",
            "symbol" => "Symbol",
            "url" => "URL",
            "url_search_params" => "URLSearchParams",
            "storage" => "Storage",
            "form_data" => "FormData",
            _ => "builtin method",
        };
        Error::ScriptRuntime(format!("{label} method called on incompatible receiver"))
    }

    fn execute_receiver_builtin_callable(
        &mut self,
        callable: &Value,
        args: &[Value],
        event: &EventState,
        this_arg: Option<Value>,
    ) -> Result<Value> {
        let (family, member) = Self::receiver_builtin_callable_components(callable)?;
        let receiver = this_arg.ok_or_else(|| Self::incompatible_receiver_error(&family))?;
        match family.as_str() {
            "array" => {
                let Value::Array(values) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_array_member_call(&values, &member, args, event)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported Array method: {member}"))
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
            "location" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                if !Self::is_location_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(&family));
                }
                match member.as_str() {
                    "assign" => {
                        let Some(url) = args.first() else {
                            return Err(Error::ScriptRuntime(
                                "location.assign requires exactly one argument".into(),
                            ));
                        };
                        self.navigate_location(&url.as_string(), LocationNavigationKind::Assign)?;
                        Ok(Value::Undefined)
                    }
                    "reload" => {
                        self.reload_location()?;
                        Ok(Value::Undefined)
                    }
                    "replace" => {
                        let Some(url) = args.first() else {
                            return Err(Error::ScriptRuntime(
                                "location.replace requires exactly one argument".into(),
                            ));
                        };
                        self.navigate_location(&url.as_string(), LocationNavigationKind::Replace)?;
                        Ok(Value::Undefined)
                    }
                    "toString" => Ok(Value::String(self.document_url.clone())),
                    _ => Err(Error::ScriptRuntime(format!(
                        "unsupported Location method: {member}"
                    ))),
                }
            }
            "string" => {
                let text = match receiver {
                    Value::String(text) => text,
                    Value::Object(object) => {
                        let entries = object.borrow();
                        if Self::is_url_object(&entries) || Self::is_location_object(&entries) {
                            return Err(Self::incompatible_receiver_error(&family));
                        }
                        Self::string_wrapper_value_from_object(&entries)
                            .ok_or_else(|| Self::incompatible_receiver_error(&family))?
                    }
                    _ => return Err(Self::incompatible_receiver_error(&family)),
                };
                match member.as_str() {
                    "toString" | "valueOf" => Ok(Value::String(text)),
                    _ => self
                        .eval_string_member_call(&text, &member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported String method: {member}"))
                        }),
                }
            }
            "node_list" => {
                let Value::NodeList(nodes) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_nodelist_member_call(&nodes, &member, args, event)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported NodeList method: {member}"))
                    })
            }
            "typed_array" => {
                let Value::TypedArray(array) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_typed_array_member_call(&array, &member, args)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported TypedArray method: {member}"))
                    })
            }
            "boolean" => {
                let Value::Bool(value) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
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
                if !matches!(receiver, Value::Number(_) | Value::Float(_)) {
                    return Err(Self::incompatible_receiver_error(&family));
                }
                match member.as_str() {
                    "toLocaleString" => {
                        let numeric = Self::coerce_number_for_number_constructor(&receiver);
                        let locale = self.resolve_number_to_locale_string_locale(args.first())?;
                        let options =
                            self.intl_number_format_options_from_value(&locale, args.get(1))?;
                        Ok(Value::String(self.intl_format_number_with_options(
                            numeric, &locale, &options,
                        )))
                    }
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
                        let numeric = Self::coerce_number_for_number_constructor(&receiver);
                        Ok(Value::String(Self::number_to_string_radix(numeric, radix)))
                    }
                    "valueOf" => Ok(receiver),
                    _ => Err(Error::ScriptRuntime(format!(
                        "unsupported Number method: {member}"
                    ))),
                }
            }
            "bigint" => {
                let Value::BigInt(value) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
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
                let Value::Symbol(symbol) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
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
            "url" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                if !Self::is_url_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(&family));
                }
                self.eval_url_member_call(&object, &member, args)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported URL method: {member}"))
                    })
            }
            "url_search_params" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                if !Self::is_url_search_params_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(&family));
                }
                self.eval_url_search_params_member_call(&object, &member, args, event)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!(
                            "unsupported URLSearchParams method: {member}"
                        ))
                    })
            }
            "storage" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                if !Self::is_storage_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(&family));
                }
                self.eval_storage_member_call(&object, &member, args)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported Storage method: {member}"))
                    })
            }
            "form_data" => {
                let Value::FormData(entries) = receiver else {
                    return Err(Self::incompatible_receiver_error(&family));
                };
                self.eval_form_data_member_call_from_values(&entries, &member, args, event)?
                    .ok_or_else(|| {
                        Error::ScriptRuntime(format!("unsupported FormData method: {member}"))
                    })
            }
            _ => Err(Error::ScriptRuntime(
                "builtin method has invalid internal state".into(),
            )),
        }
    }

    fn worker_target_from_callable(callable: &Value) -> Result<Rc<RefCell<ObjectValue>>> {
        let Value::Object(entries) = callable else {
            return Err(Error::ScriptRuntime(
                "Worker callable has invalid internal state".into(),
            ));
        };
        let entries = entries.borrow();
        match Self::object_get_entry(&entries, INTERNAL_WORKER_TARGET_KEY) {
            Some(Value::Object(worker)) => Ok(worker),
            _ => Err(Error::ScriptRuntime(
                "Worker callable has invalid internal state".into(),
            )),
        }
    }

    fn worker_global_from_object(
        worker: &Rc<RefCell<ObjectValue>>,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let entries = worker.borrow();
        match Self::object_get_entry(&entries, INTERNAL_WORKER_GLOBAL_OBJECT_KEY) {
            Some(Value::Object(global)) => Ok(global),
            _ => Err(Error::ScriptRuntime(
                "Worker instance has invalid internal state".into(),
            )),
        }
    }

    fn worker_is_terminated_object(worker: &Rc<RefCell<ObjectValue>>) -> bool {
        let entries = worker.borrow();
        matches!(
            Self::object_get_entry(&entries, INTERNAL_WORKER_TERMINATED_KEY),
            Some(Value::Bool(true))
        )
    }

    fn worker_set_terminated_object(worker: &Rc<RefCell<ObjectValue>>, terminated: bool) {
        Self::object_set_entry(
            &mut worker.borrow_mut(),
            INTERNAL_WORKER_TERMINATED_KEY.to_string(),
            Value::Bool(terminated),
        );
    }

    fn text_encoder_encode_into_value(
        &mut self,
        source: &str,
        destination: &Rc<RefCell<TypedArrayValue>>,
    ) -> Result<Value> {
        if destination.borrow().kind != TypedArrayKind::Uint8 {
            return Err(Error::ScriptRuntime(
                "TextEncoder.encodeInto destination must be a Uint8Array".into(),
            ));
        }

        let capacity = destination.borrow().observed_length();
        let mut read_utf16_units = 0usize;
        let mut written_bytes = 0usize;

        for ch in source.chars() {
            let mut encoded = [0u8; 4];
            let encoded = ch.encode_utf8(&mut encoded).as_bytes();
            if written_bytes.saturating_add(encoded.len()) > capacity {
                break;
            }

            for byte in encoded {
                self.typed_array_set_index(
                    destination,
                    written_bytes,
                    Value::Number(i64::from(*byte)),
                )?;
                written_bytes = written_bytes.saturating_add(1);
            }
            read_utf16_units = read_utf16_units.saturating_add(ch.len_utf16());
        }

        Ok(Self::new_object_value(vec![
            ("read".to_string(), Value::Number(read_utf16_units as i64)),
            ("written".to_string(), Value::Number(written_bytes as i64)),
        ]))
    }

    fn text_encoder_stream_state_from_receiver(receiver: Option<&Value>) -> Result<(Value, Value)> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "TextEncoderStream getter called on incompatible receiver".into(),
            ));
        };
        let entries = entries.borrow();
        let is_text_encoder_stream = matches!(
            Self::object_get_entry(&entries, INTERNAL_TEXT_ENCODER_STREAM_OBJECT_KEY),
            Some(Value::Bool(true))
        );
        if !is_text_encoder_stream {
            return Err(Error::ScriptRuntime(
                "TextEncoderStream getter called on incompatible receiver".into(),
            ));
        }
        let readable = Self::object_get_entry(&entries, INTERNAL_TEXT_ENCODER_STREAM_READABLE_KEY)
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "TextEncoderStream getter called on incompatible receiver".into(),
                )
            })?;
        let writable = Self::object_get_entry(&entries, INTERNAL_TEXT_ENCODER_STREAM_WRITABLE_KEY)
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "TextEncoderStream getter called on incompatible receiver".into(),
                )
            })?;
        Ok((readable, writable))
    }

    fn text_decoder_stream_state_from_receiver(
        receiver: Option<&Value>,
    ) -> Result<(String, bool, bool, Value, Value)> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "TextDecoderStream getter called on incompatible receiver".into(),
            ));
        };
        let entries = entries.borrow();
        let is_text_decoder_stream = matches!(
            Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_OBJECT_KEY),
            Some(Value::Bool(true))
        );
        if !is_text_decoder_stream {
            return Err(Error::ScriptRuntime(
                "TextDecoderStream getter called on incompatible receiver".into(),
            ));
        }
        let encoding =
            match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_ENCODING_KEY) {
                Some(Value::String(value)) => value,
                _ => {
                    return Err(Error::ScriptRuntime(
                        "TextDecoderStream getter called on incompatible receiver".into(),
                    ));
                }
            };
        let fatal = match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_FATAL_KEY) {
            Some(Value::Bool(value)) => value,
            _ => false,
        };
        let ignore_bom =
            match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_IGNORE_BOM_KEY) {
                Some(Value::Bool(value)) => value,
                _ => false,
            };
        let readable = Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_READABLE_KEY)
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "TextDecoderStream getter called on incompatible receiver".into(),
                )
            })?;
        let writable = Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_WRITABLE_KEY)
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "TextDecoderStream getter called on incompatible receiver".into(),
                )
            })?;
        Ok((encoding, fatal, ignore_bom, readable, writable))
    }

    fn attach_constructor_prototype_to_instance(
        &mut self,
        constructor: &Value,
        instance: &mut Value,
    ) -> Result<()> {
        let Value::Object(instance_entries) = instance else {
            return Ok(());
        };
        let prototype = self.object_property_from_value(constructor, "prototype")?;
        let Value::Object(prototype_entries) = prototype else {
            return Ok(());
        };
        let mut instance_entries = instance_entries.borrow_mut();
        if Self::object_get_entry(&instance_entries, INTERNAL_OBJECT_PROTOTYPE_KEY).is_none() {
            Self::object_set_entry(
                &mut instance_entries,
                INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                Value::Object(prototype_entries),
            );
        }
        Ok(())
    }

    fn normalize_text_decoder_label(raw: &str) -> Option<&'static str> {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "utf-8" | "utf8" | "unicode-1-1-utf-8" => Some("utf-8"),
            "windows-1251" | "cp1251" | "x-cp1251" => Some("windows-1251"),
            _ => None,
        }
    }

    fn text_decoder_state_from_receiver(receiver: Option<&Value>) -> Result<(String, bool, bool)> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "TextDecoder method called on incompatible receiver".into(),
            ));
        };
        let entries = entries.borrow();
        let encoding = match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_ENCODING_KEY) {
            Some(Value::String(encoding)) => encoding,
            _ => {
                return Err(Error::ScriptRuntime(
                    "TextDecoder method called on incompatible receiver".into(),
                ));
            }
        };
        let fatal = match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_FATAL_KEY) {
            Some(Value::Bool(fatal)) => fatal,
            _ => false,
        };
        let ignore_bom =
            match Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_IGNORE_BOM_KEY) {
                Some(Value::Bool(ignore_bom)) => ignore_bom,
                _ => false,
            };
        Ok((encoding, fatal, ignore_bom))
    }

    fn css_style_sheet_object_from_receiver(
        receiver: Option<&Value>,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "CSSStyleSheet method called on incompatible receiver".into(),
            ));
        };
        let is_css_style_sheet = {
            let entries_ref = entries.borrow();
            Self::is_css_style_sheet_object(&entries_ref)
        };
        if !is_css_style_sheet {
            return Err(Error::ScriptRuntime(
                "CSSStyleSheet method called on incompatible receiver".into(),
            ));
        }
        Ok(entries.clone())
    }

    fn computed_style_state_from_receiver(
        receiver: Option<&Value>,
    ) -> Result<(NodeId, Option<String>)> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "getPropertyValue called on incompatible receiver".into(),
            ));
        };
        let entries = entries.borrow();
        if !Self::is_computed_style_object(&entries) {
            return Err(Error::ScriptRuntime(
                "getPropertyValue called on incompatible receiver".into(),
            ));
        }
        let Some(node) = Self::computed_style_target_node(&entries) else {
            return Err(Error::ScriptRuntime(
                "getPropertyValue called on incompatible receiver".into(),
            ));
        };
        let pseudo = Self::computed_style_pseudo(&entries);
        Ok((node, pseudo))
    }

    fn get_computed_style_pseudo_from_value(value: Option<&Value>) -> Result<Option<String>> {
        let Some(value) = value else {
            return Ok(None);
        };
        match value {
            Value::Null | Value::Undefined => Ok(None),
            Value::String(raw) => {
                let pseudo = raw.trim();
                if !Self::is_valid_get_computed_style_pseudo_selector(pseudo) {
                    return Err(Error::ScriptRuntime(
                        "TypeError: pseudoElt must be a valid pseudo-element selector and not ::part() or ::slotted()".into(),
                    ));
                }
                Ok(Some(pseudo.to_string()))
            }
            _ => Err(Error::ScriptRuntime(
                "TypeError: pseudoElt must be a valid pseudo-element selector and not ::part() or ::slotted()".into(),
            )),
        }
    }

    fn is_valid_get_computed_style_pseudo_selector(pseudo: &str) -> bool {
        if pseudo.is_empty() {
            return false;
        }
        let lowered = pseudo.to_ascii_lowercase();
        if lowered.starts_with("::part(") || lowered.starts_with("::slotted(") {
            return false;
        }
        let Some(rest) = lowered.strip_prefix("::") else {
            return false;
        };
        if rest.is_empty() {
            return false;
        }
        let (name, maybe_args) = if let Some(paren_idx) = rest.find('(') {
            let Some(stripped) = rest.strip_suffix(')') else {
                return false;
            };
            if paren_idx + 1 > stripped.len() {
                return false;
            }
            (&stripped[..paren_idx], Some(&stripped[paren_idx + 1..]))
        } else {
            (rest, None)
        };
        if name.is_empty()
            || !name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        {
            return false;
        }
        if let Some(args) = maybe_args {
            if args.contains('(') || args.contains(')') {
                return false;
            }
        }
        true
    }

    fn text_decoder_options_from_value(options: Option<&Value>) -> Result<(bool, bool)> {
        let Some(options) = options else {
            return Ok((false, false));
        };
        match options {
            Value::Undefined | Value::Null => Ok((false, false)),
            Value::Object(entries) => {
                let entries = entries.borrow();
                let fatal =
                    Self::object_get_entry(&entries, "fatal").is_some_and(|value| value.truthy());
                let ignore_bom = Self::object_get_entry(&entries, "ignoreBOM")
                    .is_some_and(|value| value.truthy());
                Ok((fatal, ignore_bom))
            }
            _ => Err(Error::ScriptRuntime(
                "TextDecoder constructor options must be an object".into(),
            )),
        }
    }

    fn validate_text_decoder_decode_options(options: Option<&Value>) -> Result<()> {
        let Some(options) = options else {
            return Ok(());
        };
        match options {
            Value::Undefined | Value::Null => Ok(()),
            Value::Object(_) => Ok(()),
            _ => Err(Error::ScriptRuntime(
                "TextDecoder.decode options must be an object".into(),
            )),
        }
    }

    fn window_post_message_target_window(
        &self,
        this_arg: Option<&Value>,
    ) -> Rc<RefCell<ObjectValue>> {
        let Some(Value::Object(target)) = this_arg else {
            return self.dom_runtime.window_object.clone();
        };
        if Self::is_window_object(&target.borrow()) {
            target.clone()
        } else {
            self.dom_runtime.window_object.clone()
        }
    }

    fn window_post_message_target_origin_from_args(
        &self,
        args: &[Value],
        fallback_origin: &str,
    ) -> String {
        let Some(second) = args.get(1) else {
            return fallback_origin.to_string();
        };

        if matches!(second, Value::Object(_) | Value::Null) {
            if let Value::Object(entries) = second {
                let entries = entries.borrow();
                return match Self::object_get_entry(&entries, "targetOrigin") {
                    Some(Value::Null | Value::Undefined) | None => fallback_origin.to_string(),
                    Some(value) => value.as_string(),
                };
            }
            return fallback_origin.to_string();
        }

        second.as_string()
    }

    fn window_post_message_target_origin_matches(
        target_origin: &str,
        recipient_origin: &str,
        sender_origin: &str,
    ) -> bool {
        if target_origin == "*" {
            return true;
        }
        if target_origin == "/" {
            return sender_origin == recipient_origin;
        }
        target_origin == recipient_origin
    }

    fn class_list_node_from_receiver(receiver: Option<&Value>) -> Result<NodeId> {
        let Some(Value::Object(entries)) = receiver else {
            return Err(Error::ScriptRuntime(
                "DOMTokenList method called on incompatible receiver".into(),
            ));
        };
        let entries = entries.borrow();
        if !Self::is_class_list_object(&entries) {
            return Err(Error::ScriptRuntime(
                "DOMTokenList method called on incompatible receiver".into(),
            ));
        }
        match Self::object_get_entry(&entries, INTERNAL_CLASS_LIST_NODE_KEY) {
            Some(Value::Node(node)) => Ok(node),
            _ => Err(Error::ScriptRuntime(
                "DOMTokenList method called on incompatible receiver".into(),
            )),
        }
    }

    fn text_decoder_input_bytes(&self, input: Option<&Value>) -> Result<Vec<u8>> {
        let Some(input) = input else {
            return Ok(Vec::new());
        };
        match input {
            Value::Undefined => Ok(Vec::new()),
            Value::TypedArray(array) => Ok(self.typed_array_raw_bytes(array)),
            Value::ArrayBuffer(buffer) => Ok(buffer.borrow().bytes.clone()),
            _ => Err(Error::ScriptRuntime(
                "TextDecoder.decode input must be an ArrayBuffer or typed array".into(),
            )),
        }
    }

    fn decode_utf8_bytes(bytes: &[u8], fatal: bool, ignore_bom: bool) -> Result<String> {
        let mut text = if fatal {
            std::str::from_utf8(bytes)
                .map_err(|_| Error::ScriptRuntime("TextDecoder.decode invalid UTF-8 input".into()))?
                .to_string()
        } else {
            String::from_utf8_lossy(bytes).into_owned()
        };
        if !ignore_bom && text.starts_with('\u{FEFF}') {
            text.remove(0);
        }
        Ok(text)
    }

    fn decode_windows_1251_bytes(bytes: &[u8], ignore_bom: bool) -> String {
        let mut out = String::with_capacity(bytes.len());
        for byte in bytes {
            let ch = match *byte {
                0x00..=0x7F => char::from(*byte),
                0x80 => '\u{0402}',
                0x81 => '\u{0403}',
                0x82 => '\u{201A}',
                0x83 => '\u{0453}',
                0x84 => '\u{201E}',
                0x85 => '\u{2026}',
                0x86 => '\u{2020}',
                0x87 => '\u{2021}',
                0x88 => '\u{20AC}',
                0x89 => '\u{2030}',
                0x8A => '\u{0409}',
                0x8B => '\u{2039}',
                0x8C => '\u{040A}',
                0x8D => '\u{040C}',
                0x8E => '\u{040B}',
                0x8F => '\u{040F}',
                0x90 => '\u{0452}',
                0x91 => '\u{2018}',
                0x92 => '\u{2019}',
                0x93 => '\u{201C}',
                0x94 => '\u{201D}',
                0x95 => '\u{2022}',
                0x96 => '\u{2013}',
                0x97 => '\u{2014}',
                0x98 => '\u{0098}',
                0x99 => '\u{2122}',
                0x9A => '\u{0459}',
                0x9B => '\u{203A}',
                0x9C => '\u{045A}',
                0x9D => '\u{045C}',
                0x9E => '\u{045B}',
                0x9F => '\u{045F}',
                0xA0 => '\u{00A0}',
                0xA1 => '\u{040E}',
                0xA2 => '\u{045E}',
                0xA3 => '\u{0408}',
                0xA4 => '\u{00A4}',
                0xA5 => '\u{0490}',
                0xA6 => '\u{00A6}',
                0xA7 => '\u{00A7}',
                0xA8 => '\u{0401}',
                0xA9 => '\u{00A9}',
                0xAA => '\u{0404}',
                0xAB => '\u{00AB}',
                0xAC => '\u{00AC}',
                0xAD => '\u{00AD}',
                0xAE => '\u{00AE}',
                0xAF => '\u{0407}',
                0xB0 => '\u{00B0}',
                0xB1 => '\u{00B1}',
                0xB2 => '\u{0406}',
                0xB3 => '\u{0456}',
                0xB4 => '\u{0491}',
                0xB5 => '\u{00B5}',
                0xB6 => '\u{00B6}',
                0xB7 => '\u{00B7}',
                0xB8 => '\u{0451}',
                0xB9 => '\u{2116}',
                0xBA => '\u{0454}',
                0xBB => '\u{00BB}',
                0xBC => '\u{0458}',
                0xBD => '\u{0405}',
                0xBE => '\u{0455}',
                0xBF => '\u{0457}',
                0xC0..=0xDF => {
                    char::from_u32(u32::from(*byte) - 0xC0 + 0x0410).unwrap_or('\u{FFFD}')
                }
                0xE0..=0xFF => {
                    char::from_u32(u32::from(*byte) - 0xE0 + 0x0430).unwrap_or('\u{FFFD}')
                }
            };
            out.push(ch);
        }
        if !ignore_bom && out.starts_with('\u{FEFF}') {
            out.remove(0);
        }
        out
    }

    fn decode_text_decoder_bytes(
        encoding: &str,
        bytes: &[u8],
        fatal: bool,
        ignore_bom: bool,
    ) -> Result<String> {
        match encoding {
            "utf-8" => Self::decode_utf8_bytes(bytes, fatal, ignore_bom),
            "windows-1251" => Ok(Self::decode_windows_1251_bytes(bytes, ignore_bom)),
            _ => Err(Error::ScriptRuntime(format!(
                "TextDecoder.decode unsupported encoding: {encoding}"
            ))),
        }
    }

    fn resolve_worker_script_source(&self, script_url: &str) -> Result<String> {
        let url = script_url.trim();
        if url.is_empty() {
            return Err(Error::ScriptRuntime(
                "Worker constructor requires a non-empty script URL".into(),
            ));
        }
        if let Some(blob) = self.browser_apis.blob_url_objects.get(url) {
            return Ok(String::from_utf8_lossy(&blob.borrow().bytes).into_owned());
        }

        let resolved = Self::resolve_url_string(url, Some(&self.document_url))
            .unwrap_or_else(|| url.to_string());
        self.platform_mocks
            .fetch_mocks
            .get(&resolved)
            .or_else(|| self.platform_mocks.fetch_mocks.get(url))
            .map(|mock| mock.body.clone())
            .ok_or_else(|| {
                Error::ScriptRuntime(format!("Worker script source not found: {script_url}"))
            })
    }

    fn function_to_string_reference(function_id: usize) -> String {
        format!("__bt_function_ref__({function_id})")
    }

    fn worker_function_id_from_source(source: &str) -> Option<usize> {
        fn parse_marker(value: &str) -> Option<usize> {
            let marker = value
                .strip_prefix("__bt_function_ref__(")?
                .strip_suffix(')')?;
            marker.trim().parse::<usize>().ok()
        }

        let trimmed = source.trim();
        if let Some(id) = parse_marker(trimmed) {
            return Some(id);
        }
        let wrapped = trimmed.strip_prefix('(')?.strip_suffix(")()")?;
        parse_marker(wrapped.trim())
    }

    fn execute_worker_stmts(
        &mut self,
        stmts: &[Stmt],
        worker: &Value,
        worker_global: &Value,
    ) -> Result<()> {
        let worker_post_message = Self::new_worker_context_post_message_callable(worker.clone());
        let mut worker_env = HashMap::new();
        worker_env.insert("self".to_string(), worker_global.clone());
        worker_env.insert("globalThis".to_string(), worker_global.clone());
        worker_env.insert("postMessage".to_string(), worker_post_message.clone());
        worker_env.insert("onmessage".to_string(), Value::Null);
        worker_env.insert(INTERNAL_SCOPE_DEPTH_KEY.to_string(), Value::Number(1));

        let mut worker_event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
        self.run_in_task_context(|inner| {
            inner
                .execute_stmts(stmts, &None, &mut worker_event, &mut worker_env)
                .map(|_| ())
        })?;

        if let Some(onmessage) = worker_env.get("onmessage").cloned() {
            if matches!(onmessage, Value::Null | Value::Undefined) {
                return Ok(());
            }
            let Value::Object(worker_global_entries) = worker_global else {
                return Err(Error::ScriptRuntime(
                    "Worker global has invalid internal state".into(),
                ));
            };
            Self::object_set_entry(
                &mut worker_global_entries.borrow_mut(),
                "onmessage".to_string(),
                onmessage,
            );
        }
        Ok(())
    }

    fn execute_worker_script_source(
        &mut self,
        source: &str,
        worker: &Value,
        worker_global: &Value,
    ) -> Result<()> {
        if let Some(function_id) = Self::worker_function_id_from_source(source) {
            let function = self
                .script_runtime
                .function_registry
                .get(&function_id)
                .cloned()
                .ok_or_else(|| {
                    Error::ScriptRuntime(format!(
                        "Worker script function reference is not available: {function_id}"
                    ))
                })?;
            return self.execute_worker_stmts(&function.handler.stmts, worker, worker_global);
        }

        let stmts = parse_block_statements(source)?;
        self.execute_worker_stmts(&stmts, worker, worker_global)
    }

    fn new_worker_instance_from_script_source(&mut self, source: &str) -> Result<Value> {
        let worker = Self::new_object_value(vec![
            (INTERNAL_WORKER_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_WORKER_TERMINATED_KEY.to_string(),
                Value::Bool(false),
            ),
            ("onmessage".to_string(), Value::Null),
        ]);

        let worker_global_entries = Rc::new(RefCell::new(ObjectValue::default()));
        let worker_global = Value::Object(worker_global_entries.clone());

        let worker_context_post_message =
            Self::new_worker_context_post_message_callable(worker.clone());
        {
            let mut entries = worker_global_entries.borrow_mut();
            Self::object_set_entry(
                &mut entries,
                INTERNAL_WORKER_OBJECT_KEY.to_string(),
                Value::Bool(true),
            );
            Self::object_set_entry(&mut entries, "self".to_string(), worker_global.clone());
            Self::object_set_entry(
                &mut entries,
                "globalThis".to_string(),
                worker_global.clone(),
            );
            Self::object_set_entry(
                &mut entries,
                "postMessage".to_string(),
                worker_context_post_message,
            );
            Self::object_set_entry(&mut entries, "onmessage".to_string(), Value::Null);
        }

        let worker_post_message = Self::new_worker_main_post_message_callable(worker.clone());
        let worker_terminate = Self::new_worker_terminate_callable(worker.clone());
        if let Value::Object(worker_entries) = &worker {
            let mut entries = worker_entries.borrow_mut();
            Self::object_set_entry(
                &mut entries,
                INTERNAL_WORKER_GLOBAL_OBJECT_KEY.to_string(),
                worker_global.clone(),
            );
            Self::object_set_entry(&mut entries, "postMessage".to_string(), worker_post_message);
            Self::object_set_entry(&mut entries, "terminate".to_string(), worker_terminate);
        }

        self.execute_worker_script_source(source, &worker, &worker_global)?;
        Ok(worker)
    }

    fn dispatch_worker_message_to_onmessage(
        &mut self,
        target: &Rc<RefCell<ObjectValue>>,
        target_this: Value,
        data: Value,
        event: &EventState,
    ) -> Result<()> {
        let handler = {
            let entries = target.borrow();
            Self::object_get_entry(&entries, "onmessage")
        };
        let Some(handler) = handler else {
            return Ok(());
        };
        if matches!(handler, Value::Null | Value::Undefined) {
            return Ok(());
        }
        if !self.is_callable_value(&handler) {
            return Err(Error::ScriptRuntime(
                "Worker.onmessage is not a function".into(),
            ));
        }
        let event_object = Self::new_object_value(vec![("data".to_string(), data)]);
        let _ = self.execute_callable_value_with_this_and_env(
            &handler,
            &[event_object],
            event,
            None,
            Some(target_this),
        )?;
        Ok(())
    }

    fn new_event_target_instance_from_constructor(
        &mut self,
        constructor: &Value,
        this_arg: Option<Value>,
    ) -> Result<Value> {
        if let Some(this_value) = this_arg {
            if Self::is_primitive_value(&this_value) {
                return Err(Error::ScriptRuntime(
                    "constructor this value must be an object".into(),
                ));
            }
            if let Value::Object(entries) = &this_value {
                Self::object_set_entry(
                    &mut entries.borrow_mut(),
                    INTERNAL_EVENT_TARGET_OBJECT_KEY.to_string(),
                    Value::Bool(true),
                );
            }
            return Ok(this_value);
        }

        let mut entries = vec![(
            INTERNAL_EVENT_TARGET_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )];
        if let Value::Object(constructor_entries) = constructor {
            let constructor_entries = constructor_entries.borrow();
            if let Some(prototype) = Self::object_get_entry(&constructor_entries, "prototype") {
                if matches!(prototype, Value::Object(_) | Value::Null) {
                    entries.push((INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(), prototype));
                }
            }
        }
        Ok(Self::new_object_value(entries))
    }

    fn image_data_expected_length(width: usize, height: usize) -> Result<usize> {
        width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| Error::ScriptRuntime("ImageData dimensions are too large".into()))
    }

    fn image_data_kind_for_pixel_format(pixel_format: &str) -> Option<TypedArrayKind> {
        match pixel_format {
            "rgba-unorm8" => Some(TypedArrayKind::Uint8Clamped),
            "rgba-float16" => Some(TypedArrayKind::Float16),
            _ => None,
        }
    }

    fn image_data_default_pixel_format_for_kind(kind: TypedArrayKind) -> Option<&'static str> {
        match kind {
            TypedArrayKind::Uint8Clamped => Some("rgba-unorm8"),
            TypedArrayKind::Float16 => Some("rgba-float16"),
            _ => None,
        }
    }

    fn image_data_settings_from_value(options: Option<&Value>) -> Result<(String, Option<String>)> {
        let Some(options) = options else {
            return Ok(("srgb".to_string(), None));
        };
        match options {
            Value::Null | Value::Undefined => Ok(("srgb".to_string(), None)),
            Value::Object(entries) => {
                let entries = entries.borrow();
                let color_space = Self::object_get_entry(&entries, "colorSpace")
                    .map(|value| value.as_string())
                    .unwrap_or_else(|| "srgb".to_string());
                if color_space != "srgb" && color_space != "display-p3" {
                    return Err(Error::ScriptRuntime(
                        "ImageData colorSpace must be \"srgb\" or \"display-p3\"".into(),
                    ));
                }
                let pixel_format =
                    Self::object_get_entry(&entries, "pixelFormat").map(|value| value.as_string());
                if let Some(pixel_format) = &pixel_format {
                    if Self::image_data_kind_for_pixel_format(pixel_format).is_none() {
                        return Err(Error::ScriptRuntime(
                            "ImageData pixelFormat must be \"rgba-unorm8\" or \"rgba-float16\""
                                .into(),
                        ));
                    }
                }
                Ok((color_space, pixel_format))
            }
            _ => Err(Error::ScriptRuntime(
                "ImageData constructor settings argument must be an object".into(),
            )),
        }
    }

    fn image_data_constructor_dimensions_require_positive(
        width: usize,
        height: usize,
    ) -> Result<()> {
        if width == 0 || height == 0 {
            return Err(Error::ScriptRuntime(
                "ImageData width and height must be greater than 0".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn new_image_data_value(
        &mut self,
        width: usize,
        height: usize,
        kind: TypedArrayKind,
        data_override: Option<Value>,
        color_space: &str,
        pixel_format: &str,
    ) -> Result<Value> {
        let width = i64::try_from(width)
            .map_err(|_| Error::ScriptRuntime("ImageData width is too large".into()))?;
        let height = i64::try_from(height)
            .map_err(|_| Error::ScriptRuntime("ImageData height is too large".into()))?;
        let data = if let Some(data) = data_override {
            data
        } else {
            let requested_len = Self::image_data_expected_length(width as usize, height as usize)?;
            let default_len = requested_len.min(IMAGE_DATA_MAX_DEFAULT_ELEMENTS);
            self.new_typed_array_with_length(kind, default_len)?
        };
        Ok(Self::new_object_value(vec![
            ("width".to_string(), Value::Number(width)),
            ("height".to_string(), Value::Number(height)),
            ("data".to_string(), data),
            (
                "colorSpace".to_string(),
                Value::String(color_space.to_string()),
            ),
            (
                "pixelFormat".to_string(),
                Value::String(pixel_format.to_string()),
            ),
        ]))
    }

    fn new_image_data_from_constructor_args(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() < 2 || args.len() > 4 {
            return Err(Error::ScriptRuntime(
                "ImageData constructor supports two to four arguments".into(),
            ));
        }

        match &args[0] {
            Value::TypedArray(array) => {
                let input_kind = array.borrow().kind;
                if !matches!(
                    input_kind,
                    TypedArrayKind::Uint8Clamped | TypedArrayKind::Float16
                ) {
                    return Err(Error::ScriptRuntime(
                        "ImageData data argument must be a Uint8ClampedArray or Float16Array"
                            .into(),
                    ));
                }

                let width = Self::to_non_negative_usize(&args[1], "ImageData width")?;
                if width == 0 {
                    return Err(Error::ScriptRuntime(
                        "ImageData width and height must be greater than 0".into(),
                    ));
                }

                let (raw_height, settings_value) = match args.len() {
                    2 => (None, None),
                    3 => match args[2] {
                        Value::Object(_) | Value::Null | Value::Undefined => (None, Some(&args[2])),
                        _ => (
                            Some(Self::to_non_negative_usize(&args[2], "ImageData height")?),
                            None,
                        ),
                    },
                    4 => (
                        Some(Self::to_non_negative_usize(&args[2], "ImageData height")?),
                        Some(&args[3]),
                    ),
                    _ => unreachable!(),
                };

                if raw_height == Some(0) {
                    return Err(Error::ScriptRuntime(
                        "ImageData width and height must be greater than 0".into(),
                    ));
                }

                let (color_space, settings_pixel_format) =
                    Self::image_data_settings_from_value(settings_value)?;
                let default_pixel_format =
                    Self::image_data_default_pixel_format_for_kind(input_kind).ok_or_else(
                        || Error::ScriptRuntime("unsupported ImageData typed array kind".into()),
                    )?;
                let pixel_format =
                    settings_pixel_format.unwrap_or_else(|| default_pixel_format.to_string());
                let pixel_format_kind = Self::image_data_kind_for_pixel_format(&pixel_format)
                    .ok_or_else(|| {
                        Error::ScriptRuntime(
                            "ImageData pixelFormat must be \"rgba-unorm8\" or \"rgba-float16\""
                                .into(),
                        )
                    })?;
                if pixel_format_kind != input_kind {
                    return Err(Error::ScriptRuntime(
                        "ImageData pixelFormat does not match data typed array kind".into(),
                    ));
                }

                let data_len = array.borrow().observed_length();
                let height = if let Some(height) = raw_height {
                    let expected = Self::image_data_expected_length(width, height)?;
                    if expected != data_len {
                        return Err(Error::ScriptRuntime(
                            "ImageData data length does not match width and height".into(),
                        ));
                    }
                    height
                } else {
                    let row_stride = width.checked_mul(4).ok_or_else(|| {
                        Error::ScriptRuntime("ImageData dimensions are too large".into())
                    })?;
                    if row_stride == 0 || data_len % row_stride != 0 {
                        return Err(Error::ScriptRuntime(
                            "ImageData data length is not compatible with the given width".into(),
                        ));
                    }
                    let resolved_height = data_len / row_stride;
                    if resolved_height == 0 {
                        return Err(Error::ScriptRuntime(
                            "ImageData width and height must be greater than 0".into(),
                        ));
                    }
                    resolved_height
                };

                Self::image_data_constructor_dimensions_require_positive(width, height)?;

                let data_values = self.typed_array_snapshot(array)?;
                let data_copy = self.new_typed_array_from_values(input_kind, &data_values)?;
                self.new_image_data_value(
                    width,
                    height,
                    input_kind,
                    Some(data_copy),
                    &color_space,
                    &pixel_format,
                )
            }
            _ => {
                if args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "ImageData(width, height) constructor supports up to three arguments"
                            .into(),
                    ));
                }
                let width = Self::to_non_negative_usize(&args[0], "ImageData width")?;
                let height = Self::to_non_negative_usize(&args[1], "ImageData height")?;
                Self::image_data_constructor_dimensions_require_positive(width, height)?;
                let (color_space, settings_pixel_format) =
                    Self::image_data_settings_from_value(args.get(2))?;
                let pixel_format =
                    settings_pixel_format.unwrap_or_else(|| "rgba-unorm8".to_string());
                let kind =
                    Self::image_data_kind_for_pixel_format(&pixel_format).ok_or_else(|| {
                        Error::ScriptRuntime(
                            "ImageData pixelFormat must be \"rgba-unorm8\" or \"rgba-float16\""
                                .into(),
                        )
                    })?;
                self.new_image_data_value(width, height, kind, None, &color_space, &pixel_format)
            }
        }
    }

    fn new_event_object_from_constructor_args(
        &mut self,
        constructor_name: &str,
        args: &[Value],
        include_detail: bool,
        include_keyboard_fields: bool,
        include_wheel_fields: bool,
        include_navigate_fields: bool,
        include_pointer_fields: bool,
        include_hash_change_fields: bool,
        include_error_fields: bool,
        include_before_unload_fields: bool,
    ) -> Result<Value> {
        if args.is_empty() || args.len() > 2 {
            return Err(Error::ScriptRuntime(format!(
                "{constructor_name} constructor supports one or two arguments"
            )));
        }
        let event_type = args[0].as_string();
        if event_type.is_empty() {
            return Err(Error::ScriptRuntime(format!(
                "{constructor_name} constructor requires a non-empty event type"
            )));
        }

        let mut bubbles = false;
        let mut cancelable = false;
        let mut detail = if include_detail {
            Some(Value::Null)
        } else {
            None
        };
        let mut key = String::new();
        let mut code = String::new();
        let mut location = 0i64;
        let mut ctrl_key = false;
        let mut meta_key = false;
        let mut shift_key = false;
        let mut alt_key = false;
        let mut repeat = false;
        let mut is_composing = false;
        let mut delta_x = 0.0f64;
        let mut delta_y = 0.0f64;
        let mut delta_z = 0.0f64;
        let mut delta_mode = 0i64;
        let mut pointer_id = 0i64;
        let mut pointer_width = 1.0f64;
        let mut pointer_height = 1.0f64;
        let mut pointer_pressure = 0.0f64;
        let mut pointer_tangential_pressure = 0.0f64;
        let mut pointer_tilt_x = 0i64;
        let mut pointer_tilt_y = 0i64;
        let mut pointer_twist = 0i64;
        let mut pointer_type = String::new();
        let mut pointer_is_primary = false;
        let mut pointer_altitude_angle = 0.0f64;
        let mut pointer_azimuth_angle = 0.0f64;
        let mut pointer_persistent_device_id = 0i64;
        let mut can_intercept = false;
        let mut destination = Value::Null;
        let mut download_request = Value::Null;
        let mut form_data = Value::Null;
        let mut hash_change = false;
        let mut has_ua_visual_transition = false;
        let mut info = Value::Undefined;
        let mut navigation_type = "push".to_string();
        let mut signal = Self::new_navigate_event_default_signal_value();
        let mut source_element = Value::Null;
        let mut user_initiated = false;
        let mut hash_change_old_url = String::new();
        let mut hash_change_new_url = String::new();
        let mut error_message = String::new();
        let mut error_filename = String::new();
        let mut error_lineno = 0i64;
        let mut error_colno = 0i64;
        let mut error_value = Value::Null;
        let mut before_unload_return_value = String::new();
        if let Some(options) = args.get(1) {
            match options {
                Value::Null | Value::Undefined => {}
                Value::Object(entries) => {
                    let entries = entries.borrow();
                    bubbles = Self::object_get_entry(&entries, "bubbles")
                        .is_some_and(|value| value.truthy());
                    cancelable = Self::object_get_entry(&entries, "cancelable")
                        .is_some_and(|value| value.truthy());
                    if include_detail {
                        detail =
                            Some(Self::object_get_entry(&entries, "detail").unwrap_or(Value::Null));
                    }
                    if include_keyboard_fields {
                        key = Self::object_get_entry(&entries, "key")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        code = Self::object_get_entry(&entries, "code")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        location = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "location")
                                .unwrap_or(Value::Number(0)),
                        );
                        ctrl_key = Self::object_get_entry(&entries, "ctrlKey")
                            .is_some_and(|value| value.truthy());
                        meta_key = Self::object_get_entry(&entries, "metaKey")
                            .is_some_and(|value| value.truthy());
                        shift_key = Self::object_get_entry(&entries, "shiftKey")
                            .is_some_and(|value| value.truthy());
                        alt_key = Self::object_get_entry(&entries, "altKey")
                            .is_some_and(|value| value.truthy());
                        repeat = Self::object_get_entry(&entries, "repeat")
                            .is_some_and(|value| value.truthy());
                        is_composing = Self::object_get_entry(&entries, "isComposing")
                            .is_some_and(|value| value.truthy());
                    }
                    if include_wheel_fields {
                        delta_x = Self::object_get_entry(&entries, "deltaX")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        delta_y = Self::object_get_entry(&entries, "deltaY")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        delta_z = Self::object_get_entry(&entries, "deltaZ")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        delta_mode = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "deltaMode")
                                .unwrap_or(Value::Number(0)),
                        );
                    }
                    if include_pointer_fields {
                        pointer_id = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "pointerId")
                                .unwrap_or(Value::Number(0)),
                        );
                        pointer_width = Self::object_get_entry(&entries, "width")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(1.0);
                        pointer_height = Self::object_get_entry(&entries, "height")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(1.0);
                        pointer_pressure = Self::object_get_entry(&entries, "pressure")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        pointer_tangential_pressure =
                            Self::object_get_entry(&entries, "tangentialPressure")
                                .map(|value| Self::coerce_number_for_global(&value))
                                .unwrap_or(0.0);
                        pointer_tilt_x = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "tiltX").unwrap_or(Value::Number(0)),
                        );
                        pointer_tilt_y = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "tiltY").unwrap_or(Value::Number(0)),
                        );
                        pointer_twist = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "twist").unwrap_or(Value::Number(0)),
                        );
                        pointer_type = Self::object_get_entry(&entries, "pointerType")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        pointer_is_primary = Self::object_get_entry(&entries, "isPrimary")
                            .is_some_and(|value| value.truthy());
                        pointer_altitude_angle = Self::object_get_entry(&entries, "altitudeAngle")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        pointer_azimuth_angle = Self::object_get_entry(&entries, "azimuthAngle")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        pointer_persistent_device_id = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "persistentDeviceId")
                                .unwrap_or(Value::Number(0)),
                        );
                    }
                    if include_navigate_fields {
                        can_intercept = Self::object_get_entry(&entries, "canIntercept")
                            .is_some_and(|value| value.truthy());
                        destination =
                            Self::object_get_entry(&entries, "destination").unwrap_or(Value::Null);
                        download_request = Self::object_get_entry(&entries, "downloadRequest")
                            .unwrap_or(Value::Null);
                        form_data =
                            Self::object_get_entry(&entries, "formData").unwrap_or(Value::Null);
                        hash_change = Self::object_get_entry(&entries, "hashChange")
                            .is_some_and(|value| value.truthy());
                        has_ua_visual_transition =
                            Self::object_get_entry(&entries, "hasUAVisualTransition")
                                .is_some_and(|value| value.truthy());
                        info = Self::object_get_entry(&entries, "info").unwrap_or(Value::Undefined);
                        if let Some(value) = Self::object_get_entry(&entries, "navigationType") {
                            navigation_type = value.as_string();
                        }
                        if let Some(value) = Self::object_get_entry(&entries, "signal") {
                            signal = value;
                        }
                        source_element = Self::object_get_entry(&entries, "sourceElement")
                            .unwrap_or(Value::Null);
                        user_initiated = Self::object_get_entry(&entries, "userInitiated")
                            .is_some_and(|value| value.truthy());
                    }
                    if include_before_unload_fields {
                        before_unload_return_value =
                            Self::object_get_entry(&entries, "returnValue")
                                .map(|value| value.as_string())
                                .unwrap_or_default();
                    }
                    if include_hash_change_fields {
                        hash_change_old_url = Self::object_get_entry(&entries, "oldURL")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        hash_change_new_url = Self::object_get_entry(&entries, "newURL")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                    }
                    if include_error_fields {
                        error_message = Self::object_get_entry(&entries, "message")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        error_filename = Self::object_get_entry(&entries, "filename")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        error_lineno = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "lineno").unwrap_or(Value::Number(0)),
                        );
                        error_colno = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "colno").unwrap_or(Value::Number(0)),
                        );
                        error_value =
                            Self::object_get_entry(&entries, "error").unwrap_or(Value::Null);
                    }
                }
                _ => {
                    return Err(Error::ScriptRuntime(format!(
                        "{constructor_name} constructor options argument must be an object"
                    )));
                }
            }
        }

        let default_prevented =
            cancelable && include_before_unload_fields && !before_unload_return_value.is_empty();
        let event_type_value = event_type.clone();
        let mut entries = vec![
            (INTERNAL_EVENT_OBJECT_KEY.to_string(), Value::Bool(true)),
            ("type".to_string(), Value::String(event_type)),
            ("bubbles".to_string(), Value::Bool(bubbles)),
            ("cancelable".to_string(), Value::Bool(cancelable)),
            (
                "defaultPrevented".to_string(),
                Value::Bool(default_prevented),
            ),
            ("isTrusted".to_string(), Value::Bool(false)),
            ("eventPhase".to_string(), Value::Number(0)),
            (
                "timeStamp".to_string(),
                Value::Number(self.scheduler.now_ms),
            ),
            ("target".to_string(), Value::Null),
            ("currentTarget".to_string(), Value::Null),
            (
                "preventDefault".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "stopPropagation".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "stopImmediatePropagation".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        if let Some(detail) = detail {
            entries.push(("detail".to_string(), detail));
        }
        if include_keyboard_fields {
            let key_code = Self::keyboard_key_code_for_key(&key);
            let char_code = Self::keyboard_char_code_for_event(&event_type_value, &key);
            entries.push((
                INTERNAL_KEYBOARD_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("key".to_string(), Value::String(key.clone())));
            entries.push(("code".to_string(), Value::String(code)));
            entries.push(("location".to_string(), Value::Number(location)));
            entries.push(("ctrlKey".to_string(), Value::Bool(ctrl_key)));
            entries.push(("metaKey".to_string(), Value::Bool(meta_key)));
            entries.push(("shiftKey".to_string(), Value::Bool(shift_key)));
            entries.push(("altKey".to_string(), Value::Bool(alt_key)));
            entries.push(("repeat".to_string(), Value::Bool(repeat)));
            entries.push(("isComposing".to_string(), Value::Bool(is_composing)));
            entries.push(("keyCode".to_string(), Value::Number(key_code)));
            entries.push(("charCode".to_string(), Value::Number(char_code)));
            entries.push((
                "keyIdentifier".to_string(),
                Value::String(if key.is_empty() {
                    "Unidentified".to_string()
                } else {
                    key
                }),
            ));
            entries.push((
                "getModifierState".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
        }
        if include_wheel_fields {
            entries.push((
                INTERNAL_WHEEL_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("deltaX".to_string(), Value::Float(delta_x)));
            entries.push(("deltaY".to_string(), Value::Float(delta_y)));
            entries.push(("deltaZ".to_string(), Value::Float(delta_z)));
            entries.push(("deltaMode".to_string(), Value::Number(delta_mode)));
        }
        if include_pointer_fields {
            entries.push((
                INTERNAL_POINTER_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("pointerId".to_string(), Value::Number(pointer_id)));
            entries.push(("width".to_string(), Value::Float(pointer_width)));
            entries.push(("height".to_string(), Value::Float(pointer_height)));
            entries.push(("pressure".to_string(), Value::Float(pointer_pressure)));
            entries.push((
                "tangentialPressure".to_string(),
                Value::Float(pointer_tangential_pressure),
            ));
            entries.push(("tiltX".to_string(), Value::Number(pointer_tilt_x)));
            entries.push(("tiltY".to_string(), Value::Number(pointer_tilt_y)));
            entries.push(("twist".to_string(), Value::Number(pointer_twist)));
            entries.push(("pointerType".to_string(), Value::String(pointer_type)));
            entries.push(("isPrimary".to_string(), Value::Bool(pointer_is_primary)));
            entries.push((
                "altitudeAngle".to_string(),
                Value::Float(pointer_altitude_angle),
            ));
            entries.push((
                "azimuthAngle".to_string(),
                Value::Float(pointer_azimuth_angle),
            ));
            entries.push((
                "persistentDeviceId".to_string(),
                Value::Number(pointer_persistent_device_id),
            ));
            entries.push((
                "getCoalescedEvents".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
            entries.push((
                "getPredictedEvents".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
        }
        if include_navigate_fields {
            entries.push((
                INTERNAL_NAVIGATE_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("canIntercept".to_string(), Value::Bool(can_intercept)));
            entries.push(("destination".to_string(), destination));
            entries.push(("downloadRequest".to_string(), download_request));
            entries.push(("formData".to_string(), form_data));
            entries.push(("hashChange".to_string(), Value::Bool(hash_change)));
            entries.push((
                "hasUAVisualTransition".to_string(),
                Value::Bool(has_ua_visual_transition),
            ));
            entries.push(("info".to_string(), info));
            entries.push(("navigationType".to_string(), Value::String(navigation_type)));
            entries.push(("signal".to_string(), signal));
            entries.push(("sourceElement".to_string(), source_element));
            entries.push(("userInitiated".to_string(), Value::Bool(user_initiated)));
            entries.push((
                "intercept".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
            entries.push((
                "scroll".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
        }
        if include_before_unload_fields {
            entries.push((
                INTERNAL_BEFORE_UNLOAD_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push((
                "returnValue".to_string(),
                Value::String(before_unload_return_value),
            ));
        }
        if include_hash_change_fields {
            entries.push((
                INTERNAL_HASH_CHANGE_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("oldURL".to_string(), Value::String(hash_change_old_url)));
            entries.push(("newURL".to_string(), Value::String(hash_change_new_url)));
        }
        if include_error_fields {
            entries.push((
                INTERNAL_ERROR_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("message".to_string(), Value::String(error_message)));
            entries.push(("filename".to_string(), Value::String(error_filename)));
            entries.push(("lineno".to_string(), Value::Number(error_lineno)));
            entries.push(("colno".to_string(), Value::Number(error_colno)));
            entries.push(("error".to_string(), error_value));
        }
        Ok(Self::new_object_value(entries))
    }

    pub(crate) fn execute_function_prototype_member(
        &mut self,
        member: &str,
        receiver: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        if !self.is_callable_value(receiver) {
            return Err(Error::ScriptRuntime(format!(
                "Function.prototype.{member} called on non-callable value"
            )));
        }
        match member {
            "call" => {
                let call_this = args.first().cloned().unwrap_or(Value::Undefined);
                let call_args = args.get(1..).unwrap_or(&[]);
                self.execute_callable_value_with_this_and_env(
                    receiver,
                    call_args,
                    event,
                    caller_env,
                    Some(call_this),
                )
            }
            "apply" => {
                let call_this = args.first().cloned().unwrap_or(Value::Undefined);
                let call_args = if let Some(args_value) = args.get(1) {
                    self.apply_arguments_from_value(args_value)?
                } else {
                    Vec::new()
                };
                self.execute_callable_value_with_this_and_env(
                    receiver,
                    &call_args,
                    event,
                    caller_env,
                    Some(call_this),
                )
            }
            "bind" => {
                let bound_this = args.first().cloned().unwrap_or(Value::Undefined);
                let bound_args = args.get(1..).unwrap_or(&[]).to_vec();
                Ok(Self::new_bound_function_callable(
                    receiver.clone(),
                    bound_this,
                    bound_args,
                ))
            }
            "toString" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Function.prototype.toString does not take arguments".into(),
                    ));
                }
                match receiver {
                    Value::Function(function) => Ok(Value::String(
                        Self::function_to_string_reference(function.function_id),
                    )),
                    _ => Ok(Value::String("function () { [native code] }".to_string())),
                }
            }
            _ => Err(Error::ScriptRuntime(format!(
                "unsupported Function.prototype method: {member}"
            ))),
        }
    }

    fn apply_arguments_from_value(&mut self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Undefined | Value::Null => Ok(Vec::new()),
            Value::Array(values) => Ok(values.borrow().clone()),
            Value::NodeList(nodes) => Ok(self
                .node_list_snapshot(nodes)
                .into_iter()
                .map(Value::Node)
                .collect()),
            Value::TypedArray(array) => self.typed_array_snapshot(array),
            Value::Object(_) | Value::Function(_) => {
                let length = Self::value_to_i64(&self.object_property_from_value(value, "length")?);
                let length = length.max(0) as usize;
                let mut out = Vec::with_capacity(length);
                for index in 0..length {
                    out.push(self.object_property_from_value(value, &index.to_string())?);
                }
                Ok(out)
            }
            _ => Err(Error::ScriptRuntime(
                "Function.prototype.apply requires array-like arguments".into(),
            )),
        }
    }

    fn bound_callable_components(callable: &Value) -> Result<(Value, Value, Vec<Value>)> {
        let Value::Object(entries) = callable else {
            return Err(Error::ScriptRuntime(
                "bound function has invalid internal state".into(),
            ));
        };
        let entries = entries.borrow();
        let target = Self::object_get_entry(&entries, INTERNAL_BOUND_CALLABLE_TARGET_KEY)
            .ok_or_else(|| Error::ScriptRuntime("bound function has invalid target".into()))?;
        let bound_this = Self::object_get_entry(&entries, INTERNAL_BOUND_CALLABLE_THIS_KEY)
            .unwrap_or(Value::Undefined);
        let bound_args = match Self::object_get_entry(&entries, INTERNAL_BOUND_CALLABLE_ARGS_KEY) {
            Some(Value::Array(values)) => values.borrow().clone(),
            Some(Value::Undefined) | None => Vec::new(),
            _ => {
                return Err(Error::ScriptRuntime(
                    "bound function has invalid bound arguments".into(),
                ));
            }
        };
        Ok((target, bound_this, bound_args))
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
                )
            }
            Value::PromiseCapability(capability) => {
                self.invoke_promise_capability(capability, args)
            }
            Value::StringConstructor => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Ok(Value::String(value.as_string()))
            }
            Value::Object(_) => {
                let Some(kind) = Self::callable_kind_from_value(callable) else {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                };
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
                        self.execute_receiver_builtin_callable(callable, args, event, this_arg)
                    }
                    "intl_collator_compare" => {
                        let (locale, case_first, sensitivity) =
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
                            Ok(args[0].clone())
                        }
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
                        self.new_parsed_document_value_from_markup(&args[0].as_string(), true)
                    }
                    "document_parse_html_unsafe" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "Document.parseHTMLUnsafe requires exactly one argument".into(),
                            ));
                        }
                        self.new_parsed_document_value_from_markup(&args[0].as_string(), false)
                    }
                    "fetch_function" => self.eval_fetch_call_from_values(args),
                    "window_close_function" => {
                        self.browser_apis.window_closed = true;
                        self.sync_window_runtime_properties();
                        Ok(Value::Undefined)
                    }
                    "window_stop_function" => Ok(Value::Undefined),
                    "window_focus_function" => Ok(Value::Undefined),
                    "window_scroll_function" => {
                        if args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "scroll supports zero, one, or two arguments".into(),
                            ));
                        }
                        let position_changed = self.apply_document_scroll_operation("scroll", args);
                        self.dispatch_document_scroll_sequence(position_changed)?;
                        Ok(Value::Undefined)
                    }
                    "window_scroll_by_function" => {
                        if args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "scrollBy supports zero, one, or two arguments".into(),
                            ));
                        }
                        let position_changed =
                            self.apply_document_scroll_operation("scrollBy", args);
                        self.dispatch_document_scroll_sequence(position_changed)?;
                        Ok(Value::Undefined)
                    }
                    "window_scroll_to_function" => {
                        if args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "scrollTo supports zero, one, or two arguments".into(),
                            ));
                        }
                        let position_changed =
                            self.apply_document_scroll_operation("scrollTo", args);
                        self.dispatch_document_scroll_sequence(position_changed)?;
                        Ok(Value::Undefined)
                    }
                    "window_move_by_function" => {
                        if args.len() != 2 {
                            return Err(Error::ScriptRuntime(
                                "moveBy requires exactly two arguments".into(),
                            ));
                        }
                        let delta_x = Self::value_to_i64(&args[0]);
                        let delta_y = Self::value_to_i64(&args[1]);
                        self.browser_apis.window_screen_x =
                            self.browser_apis.window_screen_x.saturating_add(delta_x);
                        self.browser_apis.window_screen_y =
                            self.browser_apis.window_screen_y.saturating_add(delta_y);
                        self.sync_window_runtime_properties();
                        Ok(Value::Undefined)
                    }
                    "window_move_to_function" => {
                        if args.len() != 2 {
                            return Err(Error::ScriptRuntime(
                                "moveTo requires exactly two arguments".into(),
                            ));
                        }
                        let next_x = Self::value_to_i64(&args[0]);
                        let next_y = Self::value_to_i64(&args[1]);
                        self.browser_apis.window_screen_x = next_x;
                        self.browser_apis.window_screen_y = next_y;
                        self.sync_window_runtime_properties();
                        Ok(Value::Undefined)
                    }
                    "window_resize_by_function" => {
                        if args.len() != 2 {
                            return Err(Error::ScriptRuntime(
                                "resizeBy requires exactly two arguments".into(),
                            ));
                        }

                        let current_width = {
                            let window = self.dom_runtime.window_object.borrow();
                            let raw_value = Self::object_get_entry(&window, "innerWidth");
                            let parsed = match raw_value {
                                Some(Value::Number(value)) => Some(value as f64),
                                Some(Value::Float(value)) if value.is_finite() => Some(value),
                                Some(Value::String(value)) => value.parse::<f64>().ok(),
                                _ => None,
                            }
                            .unwrap_or(1024.0);
                            if !parsed.is_finite() {
                                1024i64
                            } else {
                                parsed.max(0.0).trunc() as i64
                            }
                        };
                        let current_height = {
                            let window = self.dom_runtime.window_object.borrow();
                            let raw_value = Self::object_get_entry(&window, "innerHeight");
                            let parsed = match raw_value {
                                Some(Value::Number(value)) => Some(value as f64),
                                Some(Value::Float(value)) if value.is_finite() => Some(value),
                                Some(Value::String(value)) => value.parse::<f64>().ok(),
                                _ => None,
                            }
                            .unwrap_or(768.0);
                            if !parsed.is_finite() {
                                768i64
                            } else {
                                parsed.max(0.0).trunc() as i64
                            }
                        };

                        let delta_x = Self::value_to_i64(&args[0]);
                        let delta_y = Self::value_to_i64(&args[1]);
                        let next_width = current_width.saturating_add(delta_x).max(0);
                        let next_height = current_height.saturating_add(delta_y).max(0);

                        {
                            let mut window = self.dom_runtime.window_object.borrow_mut();
                            Self::object_set_entry(
                                &mut window,
                                "innerWidth".to_string(),
                                Value::Number(next_width),
                            );
                            Self::object_set_entry(
                                &mut window,
                                "innerHeight".to_string(),
                                Value::Number(next_height),
                            );
                            Self::object_set_entry(
                                &mut window,
                                "outerWidth".to_string(),
                                Value::Number(next_width),
                            );
                            Self::object_set_entry(
                                &mut window,
                                "outerHeight".to_string(),
                                Value::Number(next_height),
                            );
                        }

                        Ok(Value::Undefined)
                    }
                    "window_resize_to_function" => {
                        if args.len() != 2 {
                            return Err(Error::ScriptRuntime(
                                "resizeTo requires exactly two arguments".into(),
                            ));
                        }

                        let next_width = Self::value_to_i64(&args[0]).max(0);
                        let next_height = Self::value_to_i64(&args[1]).max(0);
                        {
                            let mut window = self.dom_runtime.window_object.borrow_mut();
                            Self::object_set_entry(
                                &mut window,
                                "innerWidth".to_string(),
                                Value::Number(next_width),
                            );
                            Self::object_set_entry(
                                &mut window,
                                "innerHeight".to_string(),
                                Value::Number(next_height),
                            );
                            Self::object_set_entry(
                                &mut window,
                                "outerWidth".to_string(),
                                Value::Number(next_width),
                            );
                            Self::object_set_entry(
                                &mut window,
                                "outerHeight".to_string(),
                                Value::Number(next_height),
                            );
                        }
                        Ok(Value::Undefined)
                    }
                    "window_post_message_function" => {
                        if args.is_empty() || args.len() > 3 {
                            return Err(Error::ScriptRuntime(
                                "postMessage requires one to three arguments".into(),
                            ));
                        }

                        let sender_origin = self.current_location_parts().origin();
                        let target_origin =
                            self.window_post_message_target_origin_from_args(args, &sender_origin);
                        let target_window =
                            self.window_post_message_target_window(this_arg.as_ref());
                        let recipient_origin = self.current_location_parts().origin();
                        if !Self::window_post_message_target_origin_matches(
                            &target_origin,
                            &recipient_origin,
                            &sender_origin,
                        ) {
                            return Ok(Value::Undefined);
                        }

                        let mut array_stack = Vec::new();
                        let mut object_stack = Vec::new();
                        let data = Self::structured_clone_value(
                            &args[0],
                            &mut array_stack,
                            &mut object_stack,
                        )?;
                        let event_payload = Self::new_object_value(vec![
                            (INTERNAL_EVENT_OBJECT_KEY.to_string(), Value::Bool(true)),
                            ("type".to_string(), Value::String("message".to_string())),
                            ("data".to_string(), data),
                            ("origin".to_string(), Value::String(sender_origin)),
                            (
                                "source".to_string(),
                                Value::Object(self.dom_runtime.window_object.clone()),
                            ),
                        ]);
                        let _ = self.dispatch_event_target(target_window, event_payload)?;
                        Ok(Value::Undefined)
                    }
                    "window_get_computed_style_function" => {
                        if args.is_empty() || args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "getComputedStyle requires one or two arguments".into(),
                            ));
                        }
                        let node = match &args[0] {
                            Value::Node(node) if self.dom.element(*node).is_some() => *node,
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "TypeError: getComputedStyle target must be an Element".into(),
                                ));
                            }
                        };
                        let pseudo = Self::get_computed_style_pseudo_from_value(args.get(1))?;
                        Ok(Self::new_computed_style_object_value(node, pseudo))
                    }
                    "window_alert_function" => {
                        if args.len() > 1 {
                            return Err(Error::ScriptRuntime(
                                "alert requires zero or one argument".into(),
                            ));
                        }
                        let message = args.first().map(Value::as_string).unwrap_or_default();
                        self.platform_mocks.alert_messages.push(message);
                        Ok(Value::Undefined)
                    }
                    "window_confirm_function" => {
                        if args.len() > 1 {
                            return Err(Error::ScriptRuntime(
                                "confirm requires zero or one argument".into(),
                            ));
                        }
                        if let Some(message) = args.first() {
                            let _ = message.as_string();
                        }
                        let accepted = self
                            .platform_mocks
                            .confirm_responses
                            .pop_front()
                            .unwrap_or(self.platform_mocks.default_confirm_response);
                        Ok(Value::Bool(accepted))
                    }
                    "window_prompt_function" => {
                        if args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "prompt requires zero to two arguments".into(),
                            ));
                        }
                        if let Some(message) = args.first() {
                            let _ = message.as_string();
                        }
                        let default_value = args.get(1).map(Value::as_string);
                        let response = self
                            .platform_mocks
                            .prompt_responses
                            .pop_front()
                            .unwrap_or_else(|| {
                                self.platform_mocks
                                    .default_prompt_response
                                    .clone()
                                    .or(default_value)
                            });
                        match response {
                            Some(value) => Ok(Value::String(value)),
                            None => Ok(Value::Null),
                        }
                    }
                    "window_print_function" => {
                        self.platform_mocks.print_call_count =
                            self.platform_mocks.print_call_count.saturating_add(1);
                        Ok(Value::Undefined)
                    }
                    "window_report_error_function" => {
                        if args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "TypeError: reportError requires one argument".into(),
                            ));
                        }
                        if args.len() > 1 {
                            return Err(Error::ScriptRuntime(
                                "reportError supports only one argument".into(),
                            ));
                        }
                        let throwable = args[0].clone();
                        let event_payload = Self::new_object_value(vec![
                            (INTERNAL_EVENT_OBJECT_KEY.to_string(), Value::Bool(true)),
                            ("type".to_string(), Value::String("error".to_string())),
                            ("detail".to_string(), throwable),
                            ("bubbles".to_string(), Value::Bool(false)),
                            ("cancelable".to_string(), Value::Bool(true)),
                            ("defaultPrevented".to_string(), Value::Bool(false)),
                            ("isTrusted".to_string(), Value::Bool(false)),
                            ("eventPhase".to_string(), Value::Number(0)),
                            (
                                "timeStamp".to_string(),
                                Value::Number(self.scheduler.now_ms),
                            ),
                            ("target".to_string(), Value::Null),
                            ("currentTarget".to_string(), Value::Null),
                            (
                                "preventDefault".to_string(),
                                Self::new_builtin_placeholder_function(),
                            ),
                            (
                                "stopPropagation".to_string(),
                                Self::new_builtin_placeholder_function(),
                            ),
                            (
                                "stopImmediatePropagation".to_string(),
                                Self::new_builtin_placeholder_function(),
                            ),
                        ]);
                        let _ = self.dispatch_event_target(
                            self.dom_runtime.window_object.clone(),
                            event_payload,
                        );
                        Ok(Value::Undefined)
                    }
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
                    "text_encoder_get_encoding" => Ok(Value::String("utf-8".to_string())),
                    "text_encoder_encode" => {
                        let input = args.first().map(Value::as_string).unwrap_or_default();
                        Ok(Self::new_uint8_typed_array_from_bytes(input.as_bytes()))
                    }
                    "text_encoder_encode_into" => {
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
                    "class_list_to_string" => {
                        let node = Self::class_list_node_from_receiver(this_arg.as_ref())?;
                        Ok(Value::String(
                            class_tokens(self.dom.attr(node, "class").as_deref()).join(" "),
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
                        Ok(Self::new_data_transfer_object_value("dragstart"))
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
                        if default_selected || selected {
                            self.dom.set_attr(option, "selected", "true")?;
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
                    "worker_main_post_message" => {
                        if args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "Worker.postMessage supports up to two arguments".into(),
                            ));
                        }
                        let data = args.first().cloned().unwrap_or(Value::Undefined);
                        let worker = Self::worker_target_from_callable(callable)?;
                        if Self::worker_is_terminated_object(&worker) {
                            return Ok(Value::Undefined);
                        }
                        let worker_global = Self::worker_global_from_object(&worker)?;
                        let worker_global_value = Value::Object(worker_global.clone());
                        self.dispatch_worker_message_to_onmessage(
                            &worker_global,
                            worker_global_value,
                            data,
                            event,
                        )?;
                        Ok(Value::Undefined)
                    }
                    "worker_context_post_message" => {
                        if args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "WorkerGlobalScope.postMessage supports up to two arguments".into(),
                            ));
                        }
                        let data = args.first().cloned().unwrap_or(Value::Undefined);
                        let worker = Self::worker_target_from_callable(callable)?;
                        if Self::worker_is_terminated_object(&worker) {
                            return Ok(Value::Undefined);
                        }
                        let worker_value = Value::Object(worker.clone());
                        self.dispatch_worker_message_to_onmessage(
                            &worker,
                            worker_value,
                            data,
                            event,
                        )?;
                        Ok(Value::Undefined)
                    }
                    "worker_terminate" => {
                        let worker = Self::worker_target_from_callable(callable)?;
                        Self::worker_set_terminated_object(&worker, true);
                        Ok(Value::Undefined)
                    }
                    "global_decode_uri" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "decodeURI requires exactly one argument".into(),
                            ));
                        }
                        Ok(Value::String(decode_uri_like(&args[0].as_string(), false)?))
                    }
                    "global_decode_uri_component" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "decodeURIComponent requires exactly one argument".into(),
                            ));
                        }
                        Ok(Value::String(decode_uri_like(&args[0].as_string(), true)?))
                    }
                    "global_atob" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "atob requires exactly one argument".into(),
                            ));
                        }
                        Ok(Value::String(decode_base64_to_binary_string(
                            &args[0].as_string(),
                        )?))
                    }
                    "global_btoa" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "btoa requires exactly one argument".into(),
                            ));
                        }
                        Ok(Value::String(encode_binary_string_to_base64(
                            &args[0].as_string(),
                        )?))
                    }
                    "global_structured_clone" => {
                        if args.is_empty() || args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "structuredClone requires one or two arguments".into(),
                            ));
                        }
                        Self::structured_clone_value_with_options(&args[0], args.get(1))
                    }
                    "global_request_animation_frame" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "requestAnimationFrame requires exactly one argument".into(),
                            ));
                        }
                        let callback = args[0].clone();
                        if !self.is_callable_value(&callback) {
                            return Err(Error::ScriptRuntime(
                                "requestAnimationFrame callback must be callable".into(),
                            ));
                        }
                        let mut timer_env = caller_env
                            .cloned()
                            .unwrap_or_else(|| self.script_runtime.env.to_map());
                        let callback_name = format!(
                            "\u{0}\u{0}bt_raf_cb_{}",
                            self.script_runtime.allocate_function_id()
                        );
                        timer_env.insert(callback_name.clone(), callback);
                        let timer_id = self.schedule_animation_frame(
                            TimerCallback::Reference(callback_name),
                            &timer_env,
                        );
                        Ok(Value::Number(timer_id))
                    }
                    "global_set_timeout" => {
                        if args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "setTimeout requires at least one argument".into(),
                            ));
                        }

                        let mut timer_env = caller_env
                            .cloned()
                            .unwrap_or_else(|| self.script_runtime.env.to_map());
                        let callback = match &args[0] {
                            value if self.is_callable_value(value) => {
                                let callback_name = format!(
                                    "\u{0}\u{0}bt_timeout_cb_{}",
                                    self.script_runtime.allocate_function_id()
                                );
                                timer_env.insert(callback_name.clone(), value.clone());
                                TimerCallback::Reference(callback_name)
                            }
                            Value::String(source) => {
                                let stmts =
                                    parse_block_statements(source).map_err(|err| match err {
                                        Error::ScriptParse(message) => {
                                            Error::ScriptRuntime(format!("SyntaxError: {message}"))
                                        }
                                        other => other,
                                    })?;
                                TimerCallback::Inline(ScriptHandler {
                                    params: Vec::new(),
                                    stmts,
                                })
                            }
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "TypeError: setTimeout callback must be callable or a string"
                                        .into(),
                                ));
                            }
                        };

                        let delay = args.get(1).map(Self::value_to_i64).unwrap_or(0);
                        let callback_args = args.iter().skip(2).cloned().collect::<Vec<_>>();
                        let timer_id =
                            self.schedule_timeout(callback, delay, callback_args, &timer_env);
                        Ok(Value::Number(timer_id))
                    }
                    "global_set_interval" => {
                        if args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "setInterval requires at least one argument".into(),
                            ));
                        }

                        let mut timer_env = caller_env
                            .cloned()
                            .unwrap_or_else(|| self.script_runtime.env.to_map());
                        let callback = match &args[0] {
                            value if self.is_callable_value(value) => {
                                let callback_name = format!(
                                    "\u{0}\u{0}bt_interval_cb_{}",
                                    self.script_runtime.allocate_function_id()
                                );
                                timer_env.insert(callback_name.clone(), value.clone());
                                TimerCallback::Reference(callback_name)
                            }
                            Value::String(source) => {
                                let stmts =
                                    parse_block_statements(source).map_err(|err| match err {
                                        Error::ScriptParse(message) => {
                                            Error::ScriptRuntime(format!("SyntaxError: {message}"))
                                        }
                                        other => other,
                                    })?;
                                TimerCallback::Inline(ScriptHandler {
                                    params: Vec::new(),
                                    stmts,
                                })
                            }
                            _ => {
                                return Err(Error::ScriptRuntime(
                                    "TypeError: setInterval callback must be callable or a string"
                                        .into(),
                                ));
                            }
                        };

                        let delay = args.get(1).map(Self::value_to_i64).unwrap_or(0);
                        let callback_args = args.iter().skip(2).cloned().collect::<Vec<_>>();
                        let timer_id =
                            self.schedule_interval(callback, delay, callback_args, &timer_env);
                        Ok(Value::Number(timer_id))
                    }
                    "global_cancel_animation_frame" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "cancelAnimationFrame requires exactly one argument".into(),
                            ));
                        }
                        let timer_id = Self::value_to_i64(&args[0]);
                        self.clear_timeout(timer_id);
                        Ok(Value::Undefined)
                    }
                    "global_clear_interval" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "clearInterval requires exactly one argument".into(),
                            ));
                        }
                        let timer_id = Self::value_to_i64(&args[0]);
                        self.clear_timeout(timer_id);
                        Ok(Value::Undefined)
                    }
                    "global_clear_timeout" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "clearTimeout requires exactly one argument".into(),
                            ));
                        }
                        let timer_id = Self::value_to_i64(&args[0]);
                        self.clear_timeout(timer_id);
                        Ok(Value::Undefined)
                    }
                    "global_queue_microtask" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "queueMicrotask requires exactly one argument".into(),
                            ));
                        }
                        if !self.is_callable_value(&args[0]) {
                            return Err(Error::ScriptRuntime(
                                "queueMicrotask callback must be callable".into(),
                            ));
                        }
                        self.queue_callable_microtask(args[0].clone());
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
                    "create_image_bitmap" => self.eval_create_image_bitmap_call(args),
                    _ => Err(Error::ScriptRuntime("callback is not a function".into())),
                }
            }
            _ => Err(Error::ScriptRuntime("callback is not a function".into())),
        }
    }

    fn eval_create_image_bitmap_call(&mut self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Err(Error::ScriptRuntime(
                "createImageBitmap requires at least one argument".into(),
            ));
        }

        let promise = self.new_pending_promise();
        match self.create_image_bitmap_dimensions_from_args(args) {
            Ok((width, height)) => {
                self.promise_resolve(
                    &promise,
                    Self::new_object_value(vec![
                        ("width".to_string(), Value::Number(width)),
                        ("height".to_string(), Value::Number(height)),
                        (
                            "close".to_string(),
                            Self::new_builtin_placeholder_function(),
                        ),
                    ]),
                )?;
            }
            Err(err) => {
                self.promise_reject(&promise, Value::String(err));
            }
        }
        Ok(Value::Promise(promise))
    }

    fn create_image_bitmap_dimensions_from_args(
        &self,
        args: &[Value],
    ) -> std::result::Result<(i64, i64), String> {
        let (source_width, source_height) =
            self.create_image_bitmap_dimensions_from_value(&args[0])?;
        let mut width = source_width;
        let mut height = source_height;

        let options = match args.len() {
            1 => None,
            2 => Some(&args[1]),
            5 => {
                let crop_width = Self::value_to_i64(&args[3]).abs();
                let crop_height = Self::value_to_i64(&args[4]).abs();
                if crop_width == 0 || crop_height == 0 {
                    return Err("createImageBitmap crop width/height must be non-zero".to_string());
                }
                width = crop_width;
                height = crop_height;
                None
            }
            6 => {
                let crop_width = Self::value_to_i64(&args[3]).abs();
                let crop_height = Self::value_to_i64(&args[4]).abs();
                if crop_width == 0 || crop_height == 0 {
                    return Err("createImageBitmap crop width/height must be non-zero".to_string());
                }
                width = crop_width;
                height = crop_height;
                Some(&args[5])
            }
            _ => {
                return Err("createImageBitmap supports 1, 2, 5, or 6 arguments".to_string());
            }
        };

        let (resize_width, resize_height) =
            self.create_image_bitmap_resize_from_options(options)?;
        if let Some(resize_width) = resize_width {
            width = resize_width;
        }
        if let Some(resize_height) = resize_height {
            height = resize_height;
        }

        Ok((width.max(1), height.max(1)))
    }

    fn create_image_bitmap_resize_from_options(
        &self,
        options: Option<&Value>,
    ) -> std::result::Result<(Option<i64>, Option<i64>), String> {
        let Some(options) = options else {
            return Ok((None, None));
        };

        match options {
            Value::Null | Value::Undefined => Ok((None, None)),
            Value::Object(entries) => {
                let entries = entries.borrow();
                let resize_width = match Self::object_get_entry(&entries, "resizeWidth") {
                    Some(Value::Null | Value::Undefined) | None => None,
                    Some(value) => {
                        let width = Self::value_to_i64(&value);
                        if width <= 0 {
                            return Err("createImageBitmap resizeWidth must be a positive integer"
                                .to_string());
                        }
                        Some(width)
                    }
                };
                let resize_height = match Self::object_get_entry(&entries, "resizeHeight") {
                    Some(Value::Null | Value::Undefined) | None => None,
                    Some(value) => {
                        let height = Self::value_to_i64(&value);
                        if height <= 0 {
                            return Err(
                                "createImageBitmap resizeHeight must be a positive integer"
                                    .to_string(),
                            );
                        }
                        Some(height)
                    }
                };
                Ok((resize_width, resize_height))
            }
            _ => Err("createImageBitmap options must be an object".to_string()),
        }
    }

    fn create_image_bitmap_dimensions_from_value(
        &self,
        source: &Value,
    ) -> std::result::Result<(i64, i64), String> {
        let (bytes, mime_type, logical_size) = match source {
            Value::Blob(blob) => {
                let blob = blob.borrow();
                (
                    blob.bytes.clone(),
                    blob.mime_type.clone(),
                    blob.bytes.len() as i64,
                )
            }
            Value::Object(entries) => {
                let entries = entries.borrow();
                let width = Self::object_get_entry(&entries, "width")
                    .map(|value| Self::value_to_i64(&value));
                let height = Self::object_get_entry(&entries, "height")
                    .map(|value| Self::value_to_i64(&value));
                if let (Some(width), Some(height)) = (width, height) {
                    if width > 0 && height > 0 {
                        return Ok((width, height));
                    }
                }

                if !Self::is_mock_file_object(&entries) {
                    return Err(
                        "createImageBitmap requires an image Blob or File source".to_string()
                    );
                }

                let blob = match Self::object_get_entry(&entries, INTERNAL_MOCK_FILE_BLOB_KEY) {
                    Some(Value::Blob(blob)) => blob,
                    _ => {
                        return Err(
                            "createImageBitmap could not access mock file bytes".to_string()
                        );
                    }
                };
                let (bytes, mime_type) = {
                    let blob = blob.borrow();
                    (blob.bytes.clone(), blob.mime_type.clone())
                };
                let logical_size = Self::object_get_entry(&entries, "size")
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(bytes.len() as i64);
                (bytes, mime_type, logical_size)
            }
            Value::Node(node) => {
                let tag = self
                    .dom
                    .tag_name(*node)
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                let dimensions = match tag.as_str() {
                    "canvas" => {
                        let width = self
                            .dom
                            .attr(*node, "width")
                            .and_then(|value| value.parse::<i64>().ok())
                            .unwrap_or(300);
                        let height = self
                            .dom
                            .attr(*node, "height")
                            .and_then(|value| value.parse::<i64>().ok())
                            .unwrap_or(150);
                        Some((width, height))
                    }
                    _ => None,
                };
                let Some((width, height)) = dimensions else {
                    return Err(
                        "createImageBitmap requires an image Blob or File source".to_string()
                    );
                };
                if width <= 0 || height <= 0 {
                    return Err("createImageBitmap could not decode image source".to_string());
                }
                return Ok((width, height));
            }
            _ => {
                return Err("createImageBitmap requires an image Blob or File source".to_string());
            }
        };

        let mime = mime_type.to_ascii_lowercase();
        let (width, height) = Self::decode_image_dimensions(&bytes)
            .or_else(|| {
                if mime.starts_with("image/") && (logical_size > 0 || !bytes.is_empty()) {
                    Some((1, 1))
                } else {
                    None
                }
            })
            .ok_or_else(|| "createImageBitmap could not decode image source".to_string())?;

        Ok((width.max(1), height.max(1)))
    }

    fn decode_image_dimensions(bytes: &[u8]) -> Option<(i64, i64)> {
        Self::decode_png_dimensions(bytes)
            .or_else(|| Self::decode_gif_dimensions(bytes))
            .or_else(|| Self::decode_jpeg_dimensions(bytes))
    }

    fn decode_png_dimensions(bytes: &[u8]) -> Option<(i64, i64)> {
        const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
        if bytes.len() < 24 || bytes[0..8] != PNG_SIGNATURE {
            return None;
        }
        if &bytes[12..16] != b"IHDR" {
            return None;
        }
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        if width == 0 || height == 0 {
            return None;
        }
        Some((width as i64, height as i64))
    }

    fn decode_gif_dimensions(bytes: &[u8]) -> Option<(i64, i64)> {
        if bytes.len() < 10 {
            return None;
        }
        if &bytes[0..6] != b"GIF87a" && &bytes[0..6] != b"GIF89a" {
            return None;
        }
        let width = u16::from_le_bytes([bytes[6], bytes[7]]);
        let height = u16::from_le_bytes([bytes[8], bytes[9]]);
        if width == 0 || height == 0 {
            return None;
        }
        Some((width as i64, height as i64))
    }

    fn decode_jpeg_dimensions(bytes: &[u8]) -> Option<(i64, i64)> {
        if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
            return None;
        }

        let mut offset = 2usize;
        while offset + 1 < bytes.len() {
            if bytes[offset] != 0xFF {
                offset += 1;
                continue;
            }
            while offset < bytes.len() && bytes[offset] == 0xFF {
                offset += 1;
            }
            if offset >= bytes.len() {
                break;
            }
            let marker = bytes[offset];
            offset += 1;

            if marker == 0xD9 || marker == 0xDA {
                break;
            }
            if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
                continue;
            }

            if offset + 1 >= bytes.len() {
                break;
            }
            let segment_len = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
            offset += 2;
            if segment_len < 2 || offset + segment_len - 2 > bytes.len() {
                break;
            }

            let is_sof = matches!(
                marker,
                0xC0 | 0xC1
                    | 0xC2
                    | 0xC3
                    | 0xC5
                    | 0xC6
                    | 0xC7
                    | 0xC9
                    | 0xCA
                    | 0xCB
                    | 0xCD
                    | 0xCE
                    | 0xCF
            );
            if is_sof && segment_len >= 7 {
                let height = u16::from_be_bytes([bytes[offset + 1], bytes[offset + 2]]);
                let width = u16::from_be_bytes([bytes[offset + 3], bytes[offset + 4]]);
                if width > 0 && height > 0 {
                    return Some((width as i64, height as i64));
                }
                return None;
            }

            offset += segment_len - 2;
        }

        None
    }

    fn new_clipboard_item_value_from_constructor_args(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Err(Error::ScriptRuntime(
                "ClipboardItem constructor requires at least one argument".into(),
            ));
        }
        if args.len() > 2 {
            return Err(Error::ScriptRuntime(
                "ClipboardItem constructor supports up to two arguments".into(),
            ));
        }

        let Value::Object(entries) = &args[0] else {
            return Err(Error::ScriptRuntime(
                "ClipboardItem constructor requires a data object".into(),
            ));
        };
        let entries = entries.borrow();
        let mut instance_entries = vec![
            (
                INTERNAL_CLIPBOARD_ITEM_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                "presentationStyle".to_string(),
                Value::String("unspecified".to_string()),
            ),
        ];
        let mut types = Vec::new();

        for (mime_type, payload) in entries.iter() {
            if Self::is_internal_object_key(mime_type) {
                continue;
            }
            let mime_type = mime_type.to_ascii_lowercase();
            let blob = Self::clipboard_payload_to_blob(payload, &mime_type)?;
            instance_entries.push((mime_type.clone(), Value::Blob(blob)));
            types.push(Value::String(mime_type));
        }

        if types.is_empty() {
            return Err(Error::ScriptRuntime(
                "ClipboardItem constructor requires at least one clipboard type".into(),
            ));
        }
        instance_entries.push(("types".to_string(), Self::new_array_value(types)));
        Ok(Self::new_object_value(instance_entries))
    }

    fn eval_clipboard_write_call(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(Error::ScriptRuntime(
                "navigator.clipboard.write requires exactly one argument".into(),
            ));
        }

        let promise = self.new_pending_promise();
        if let Some(reason) = self.platform_mocks.clipboard_write_error.clone() {
            self.promise_reject(&promise, Value::String(reason));
            return Ok(Value::Promise(promise));
        }

        let mut write_artifact = ClipboardWriteArtifact {
            payloads: Vec::new(),
        };
        let Value::Array(items) = &args[0] else {
            self.promise_reject(
                &promise,
                Value::String(
                    "navigator.clipboard.write requires an array of ClipboardItem".into(),
                ),
            );
            return Ok(Value::Promise(promise));
        };

        for item in items.borrow().iter() {
            let payloads = self.clipboard_payloads_from_item_value(item)?;
            write_artifact.payloads.extend(payloads);
        }

        self.browser_apis.clipboard_writes.push(write_artifact);
        self.promise_resolve(&promise, Value::Undefined)?;
        Ok(Value::Promise(promise))
    }

    fn clipboard_payloads_from_item_value(
        &self,
        item: &Value,
    ) -> Result<Vec<ClipboardPayloadArtifact>> {
        let Value::Object(entries) = item else {
            return Err(Error::ScriptRuntime(
                "Clipboard.write items must be objects".into(),
            ));
        };
        let entries = entries.borrow();

        let types = if Self::is_clipboard_item_object(&entries) {
            match Self::object_get_entry(&entries, "types") {
                Some(Value::Array(types)) => types
                    .borrow()
                    .iter()
                    .map(Value::as_string)
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            }
        } else {
            entries
                .iter()
                .filter_map(|(key, _)| (!Self::is_internal_object_key(key)).then_some(key.clone()))
                .collect::<Vec<_>>()
        };

        let mut payloads = Vec::new();
        for mime_type in types {
            let Some(payload) = Self::object_get_entry(&entries, &mime_type) else {
                continue;
            };
            let blob = Self::clipboard_payload_to_blob(&payload, &mime_type)?;
            let blob = blob.borrow();
            payloads.push(ClipboardPayloadArtifact {
                mime_type: mime_type.clone(),
                bytes: blob.bytes.clone(),
            });
        }

        if payloads.is_empty() {
            return Err(Error::ScriptRuntime(
                "ClipboardItem must provide at least one payload".into(),
            ));
        }
        Ok(payloads)
    }

    fn clipboard_payload_to_blob(
        payload: &Value,
        mime_type_hint: &str,
    ) -> Result<Rc<RefCell<BlobValue>>> {
        match payload {
            Value::Blob(blob) => Ok(blob.clone()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if !Self::is_mock_file_object(&entries) {
                    return Err(Error::ScriptRuntime(
                        "ClipboardItem payload must be a Blob or mock file".into(),
                    ));
                }
                let blob = match Self::object_get_entry(&entries, INTERNAL_MOCK_FILE_BLOB_KEY) {
                    Some(Value::Blob(blob)) => blob,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "ClipboardItem payload has invalid mock file blob".into(),
                        ));
                    }
                };
                Ok(blob)
            }
            Value::String(text) => Ok(Rc::new(RefCell::new(BlobValue {
                bytes: text.as_bytes().to_vec(),
                mime_type: mime_type_hint.to_string(),
            }))),
            _ => Err(Error::ScriptRuntime(
                "ClipboardItem payload must be a Blob or string".into(),
            )),
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
    ) -> Result<Value> {
        let run = |this: &mut Self,
                   caller_env: Option<&HashMap<String, Value>>,
                   this_arg: Option<Value>,
                   new_target: Option<Value>|
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

            let result = this.with_isolated_loop_control_scope(|this| {
                (|| -> Result<Value> {
                    let captured_env_before_call = if function.global_scope {
                        HashMap::new()
                    } else {
                        function.captured_env.borrow().to_map()
                    };
                    let mut call_env = if function.global_scope {
                        this.script_runtime.env.to_map()
                    } else {
                        captured_env_before_call.clone()
                    };
                    call_env.remove(INTERNAL_RETURN_SLOT);
                    let scope_depth = Self::env_scope_depth(&call_env);
                    call_env.insert(
                        INTERNAL_SCOPE_DEPTH_KEY.to_string(),
                        Value::Number(scope_depth.saturating_add(1)),
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
                    let mut global_sync_keys = HashSet::new();
                    let caller_view = caller_env;
                    for name in &function.captured_global_names {
                        if Self::is_internal_env_key(name)
                            || function.local_bindings.contains(name)
                            || name == "this"
                            || name == "arguments"
                        {
                            continue;
                        }
                        global_sync_keys.insert(name.clone());
                        if let Some(global_value) = this.script_runtime.env.get(name).cloned() {
                            if function.global_scope || !call_env.contains_key(name) {
                                call_env.insert(name.clone(), global_value);
                            }
                        } else if !call_env.contains_key(name) {
                            if let Some(value) = caller_view.and_then(|env| env.get(name)).cloned()
                            {
                                call_env.insert(name.clone(), value);
                            }
                        }
                    }
                    for (name, global_value) in this.script_runtime.env.iter() {
                        if Self::is_internal_env_key(name)
                            || function.local_bindings.contains(name)
                            || name == "this"
                            || name == "arguments"
                            || call_env.contains_key(name)
                        {
                            continue;
                        }
                        call_env.insert(name.clone(), global_value.clone());
                        global_sync_keys.insert(name.clone());
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
                    let event_param = None;
                    this.script_runtime
                        .listener_capture_env_stack
                        .push(ListenerCaptureFrame {
                            inherit_outer_pending: false,
                            ..ListenerCaptureFrame::default()
                        });
                    let bind_result = (|| -> Result<()> {
                        this.bind_handler_params(
                            &function.handler,
                            args,
                            &mut call_env,
                            &event_param,
                            &call_event,
                        )?;
                        this.apply_pending_listener_capture_env_updates(&mut call_env);
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
                    let mut body_env = call_env.clone();
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

                    let pushed_non_tdz_scope = !non_tdz_shadowed.is_empty();
                    if pushed_non_tdz_scope {
                        this.script_runtime.tdz_scope_stack.push(TdzScopeFrame {
                            declared: non_tdz_shadowed,
                            pending: HashSet::new(),
                        });
                    }
                    let flow = this.execute_stmts_with_pending_scope(
                        &function.handler.stmts,
                        &event_param,
                        &mut call_event,
                        &mut body_env,
                        false,
                    );
                    if pushed_non_tdz_scope {
                        this.script_runtime.tdz_scope_stack.pop();
                    }
                    if yield_collector.is_some() {
                        let _ = this.script_runtime.generator_yield_stack.pop();
                    }
                    let flow = match flow {
                        Ok(flow) => flow,
                        Err(Error::ScriptRuntime(msg))
                            if function.is_generator
                                && msg == INTERNAL_GENERATOR_YIELD_LIMIT_REACHED =>
                        {
                            ExecFlow::Continue
                        }
                        Err(err) => return Err(err),
                    };
                    let generator_yields = yield_collector
                        .as_ref()
                        .map(|values| values.borrow().clone())
                        .unwrap_or_default();
                    let generator_return_value = if matches!(flow, ExecFlow::Return) {
                        body_env
                            .get(INTERNAL_RETURN_SLOT)
                            .cloned()
                            .unwrap_or(Value::Undefined)
                    } else {
                        Value::Undefined
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
                        let call_after = body_env.get(name).cloned();
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
                        }
                    }
                    if !function.global_scope {
                        let mut captured_env = function.captured_env.borrow_mut();
                        for name in captured_env_before_call.keys() {
                            if Self::is_internal_env_key(name)
                                || function.local_bindings.contains(name.as_str())
                                || name == "this"
                                || name == "arguments"
                            {
                                continue;
                            }
                            let before = captured_env_before_call.get(name);
                            let after = body_env.get(name);
                            let changed = match (before, after) {
                                (Some(prev), Some(next)) => !this.strict_equal(prev, next),
                                (None, Some(_)) => true,
                                (Some(_), None) => true,
                                (None, None) => false,
                            };
                            if !changed {
                                continue;
                            }
                            if let Some(next) = after.cloned() {
                                captured_env.insert(name.clone(), next.clone());
                                this.queue_listener_capture_env_update_for_shared_env(
                                    &function.captured_env,
                                    name.clone(),
                                    Some(next),
                                );
                            } else {
                                captured_env.remove(name);
                                this.queue_listener_capture_env_update_for_shared_env(
                                    &function.captured_env,
                                    name.clone(),
                                    None,
                                );
                            }
                        }
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
                        ExecFlow::Return => Ok(body_env
                            .remove(INTERNAL_RETURN_SLOT)
                            .unwrap_or(Value::Undefined)),
                    }
                })()
            });

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
            match run(self, caller_env, this_arg.clone(), new_target.clone()) {
                Ok(value) => {
                    if let Err(err) = self.promise_resolve(&promise, value) {
                        self.promise_reject(&promise, Self::promise_error_reason(err));
                    }
                }
                Err(err) => self.promise_reject(&promise, Self::promise_error_reason(err)),
            }
            Ok(Value::Promise(promise))
        } else {
            run(self, caller_env, this_arg, new_target)
        }
    }
}
