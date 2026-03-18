use super::*;

impl Harness {
    fn current_module_referrer(&self) -> String {
        self.script_runtime
            .module_referrer_stack
            .last()
            .cloned()
            .unwrap_or_else(|| self.document_url.clone())
    }

    pub(crate) fn bind_hoisted_import_decls(
        &mut self,
        stmts: &[Stmt],
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        for stmt in stmts {
            let Stmt::ImportDecl {
                specifier,
                default_binding,
                namespace_binding,
                named_bindings,
                attribute_type,
            } = stmt
            else {
                continue;
            };

            let referrer = self.current_module_referrer();
            let exports =
                self.load_module_exports(specifier, attribute_type.as_deref(), &referrer)?;

            if let Some(local) = default_binding {
                let value = exports.get("default").cloned().unwrap_or(Value::Undefined);
                env.insert(local.clone(), value);
                self.set_const_binding(env, local, true);
            }

            if let Some(local) = namespace_binding {
                let mut entries = exports
                    .iter()
                    .map(|(name, value)| (name.clone(), value.clone()))
                    .collect::<Vec<_>>();
                entries.sort_by(|(a, _), (b, _)| a.cmp(b));
                env.insert(local.clone(), Self::new_object_value(entries));
                self.set_const_binding(env, local, true);
            }

            for binding in named_bindings {
                let value = exports.get(&binding.imported).cloned().ok_or_else(|| {
                    Error::ScriptRuntime(format!(
                        "module '{}' does not provide an export named '{}'",
                        specifier, binding.imported
                    ))
                })?;
                env.insert(binding.local.clone(), value);
                self.set_const_binding(env, &binding.local, true);
            }
        }
        Ok(())
    }

    fn register_module_named_exports(&mut self, bindings: &[(String, String)]) {
        let Some(exports) = self.script_runtime.module_export_stack.last() else {
            return;
        };
        let mut exports = exports.borrow_mut();
        for (local, exported) in bindings {
            exports.insert(exported.clone(), ModuleExportBinding::Local(local.clone()));
        }
    }

    fn register_module_default_export_value(&mut self, value: Value) {
        let Some(exports) = self.script_runtime.module_export_stack.last() else {
            return;
        };
        exports
            .borrow_mut()
            .insert("default".to_string(), ModuleExportBinding::Value(value));
    }

    fn default_derived_class_constructor_handler() -> ScriptHandler {
        let args_name = "__bt_super_args".to_string();
        ScriptHandler {
            params: vec![FunctionParam {
                name: args_name.clone(),
                default: None,
                is_rest: true,
            }],
            stmts: vec![Stmt::Expr(Expr::FunctionCall {
                target: "super".to_string(),
                args: vec![Expr::Spread(Box::new(Expr::Var(args_name)))],
            })],
        }
    }

    fn is_valid_extends_constructor_candidate(value: &Value) -> bool {
        match value {
            Value::Function(function) => {
                !(function.is_generator || function.is_arrow || function.is_method)
            }
            Value::PromiseCapability(_) => false,
            Value::Null => true,
            other => Self::is_callable_kind_constructor(other),
        }
    }

    fn is_callable_kind_constructor(value: &Value) -> bool {
        matches!(
            value,
            Value::StringConstructor
                | Value::UrlConstructor
                | Value::ArrayBufferConstructor
                | Value::TypedArrayConstructor(_)
                | Value::BlobConstructor
                | Value::PromiseConstructor
                | Value::MapConstructor
                | Value::WeakMapConstructor
                | Value::SetConstructor
                | Value::WeakSetConstructor
                | Value::UrlSearchParamsConstructor
                | Value::RegExpConstructor
        ) || matches!(
            Self::callable_kind_from_value(value),
            Some("event_target_constructor" | "audio_constructor")
        )
    }

    fn execute_var_decl_stmt(
        &mut self,
        name: &str,
        expr: &Expr,
        kind: VarDeclKind,
        pending_tdz_bindings: &mut HashSet<String>,
        initialized_var_bindings: &mut HashSet<String>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        if matches!(kind, VarDeclKind::Var) && matches!(expr, Expr::Undefined) {
            if !env.contains_key(name) {
                env.insert(name.to_string(), Value::Undefined);
                self.set_const_binding(env, name, false);
                self.sync_global_binding_if_needed(env, name, &Value::Undefined);
                self.sync_scheduled_task_captures_for_binding_if_escaping(
                    env,
                    name,
                    &Value::Undefined,
                );
            }
            return Ok(());
        }

        let value = self.eval_expr(expr, env, event_param, event)?;
        env.insert(name.to_string(), value.clone());
        self.set_const_binding(env, name, matches!(kind, VarDeclKind::Const));
        if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
            self.mark_tdz_initialized(pending_tdz_bindings, name);
        }
        self.sync_global_binding_if_needed(env, name, &value);
        self.sync_scheduled_task_captures_for_binding_if_escaping(env, name, &value);
        self.bind_timer_id_to_task_env(name, expr, &value);
        if matches!(kind, VarDeclKind::Var) && !matches!(expr, Expr::Undefined) {
            initialized_var_bindings.insert(name.to_string());
        }
        Ok(())
    }

    fn execute_function_decl_stmt(
        &mut self,
        name: &str,
        handler: &ScriptHandler,
        is_async: bool,
        is_generator: bool,
        initialized_var_bindings: &HashSet<String>,
        env: &mut HashMap<String, Value>,
    ) {
        if initialized_var_bindings.contains(name) {
            return;
        }
        let function = self.make_function_value(
            handler.clone(),
            env,
            false,
            is_async,
            is_generator,
            false,
            false,
        );
        if let Value::Function(function_value) = &function {
            self.set_function_public_name(function_value, name);
            function_value
                .captured_env
                .borrow_mut()
                .insert(name.to_string(), function.clone());
        }
        env.insert(name.to_string(), function.clone());
        self.set_const_binding(env, name, false);
        self.sync_global_binding_if_needed(env, name, &function);
        self.sync_scheduled_task_captures_for_binding_if_escaping(env, name, &function);
    }

    fn execute_class_decl_stmt(
        &mut self,
        name: &str,
        super_class: &Option<Expr>,
        constructor: &Option<ScriptHandler>,
        fields: &[ClassFieldDecl],
        methods: &[ClassMethodDecl],
        static_initializers: &[ClassStaticInitializerDecl],
        pending_tdz_bindings: &mut HashSet<String>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        let (super_constructor, super_prototype) = if let Some(super_class_expr) = super_class {
            let evaluated_super = self.eval_expr(super_class_expr, env, event_param, event)?;
            if !Self::is_valid_extends_constructor_candidate(&evaluated_super) {
                return Err(Error::ScriptRuntime(
                    "class extends value is not a constructor".into(),
                ));
            }
            if matches!(evaluated_super, Value::Null) {
                (Some(evaluated_super), Some(Value::Null))
            } else {
                let super_prototype =
                    self.object_property_from_value(&evaluated_super, "prototype")?;
                if !matches!(super_prototype, Value::Object(_) | Value::Null) {
                    return Err(Error::ScriptRuntime(
                        "class extends value does not have a valid prototype".into(),
                    ));
                }
                (Some(evaluated_super), Some(super_prototype))
            }
        } else {
            (None, None)
        };

        let constructor_handler = if let Some(handler) = constructor.clone() {
            handler
        } else if super_constructor.is_some() {
            Self::default_derived_class_constructor_handler()
        } else {
            ScriptHandler {
                params: Vec::new(),
                stmts: Vec::new(),
            }
        };

        let class_constructor = self.make_class_constructor_value_with_super(
            constructor_handler,
            env,
            false,
            super_constructor.clone(),
            super_prototype.clone(),
        );
        let Value::Function(class_function) = &class_constructor else {
            return Err(Error::ScriptRuntime(
                "class constructor is not callable".into(),
            ));
        };
        self.set_function_public_name(class_function, name);
        let class_constructor_id = class_function.function_id;
        env.insert(name.to_string(), class_constructor.clone());
        self.set_const_binding(env, name, false);
        self.mark_tdz_initialized(pending_tdz_bindings, name);

        let mut private_bindings = HashMap::new();
        for method in methods.iter().filter(|method| method.is_private) {
            match method.kind {
                ClassMethodKind::Method => {
                    if private_bindings.contains_key(&method.name) {
                        return Err(Error::ScriptRuntime(format!(
                            "duplicate private identifier '#{}'",
                            method.name
                        )));
                    }
                    private_bindings.insert(
                        method.name.clone(),
                        PrivateBindingRuntime {
                            name: method.name.clone(),
                            slot_id: self.script_runtime.allocate_private_slot_id(),
                            is_static: method.is_static,
                            kind: PrivateBindingKind::Method,
                            has_getter: false,
                            has_setter: false,
                        },
                    );
                }
                ClassMethodKind::Getter | ClassMethodKind::Setter => {
                    if let Some(binding) = private_bindings.get_mut(&method.name) {
                        if binding.kind != PrivateBindingKind::Accessor
                            || binding.is_static != method.is_static
                        {
                            return Err(Error::ScriptRuntime(format!(
                                "duplicate private identifier '#{}'",
                                method.name
                            )));
                        }
                        if matches!(method.kind, ClassMethodKind::Getter) {
                            binding.has_getter = true;
                        } else {
                            binding.has_setter = true;
                        }
                    } else {
                        private_bindings.insert(
                            method.name.clone(),
                            PrivateBindingRuntime {
                                name: method.name.clone(),
                                slot_id: self.script_runtime.allocate_private_slot_id(),
                                is_static: method.is_static,
                                kind: PrivateBindingKind::Accessor,
                                has_getter: matches!(method.kind, ClassMethodKind::Getter),
                                has_setter: matches!(method.kind, ClassMethodKind::Setter),
                            },
                        );
                    }
                }
            }
        }
        for field in fields.iter().filter(|field| field.is_private) {
            if private_bindings.contains_key(&field.name) {
                return Err(Error::ScriptRuntime(format!(
                    "duplicate private identifier '#{}'",
                    field.name
                )));
            }
            private_bindings.insert(
                field.name.clone(),
                PrivateBindingRuntime {
                    name: field.name.clone(),
                    slot_id: self.script_runtime.allocate_private_slot_id(),
                    is_static: field.is_static,
                    kind: PrivateBindingKind::Field,
                    has_getter: false,
                    has_setter: false,
                },
            );
        }

        let mut class_function_ids = vec![class_constructor_id];
        let mut static_private_method_initializers = Vec::new();
        let mut instance_private_method_initializers = Vec::new();
        let mut static_field_initializers_by_index = HashMap::new();
        let mut instance_field_initializers = Vec::new();

        {
            let mut prototype = class_function.prototype_object.borrow_mut();
            if let Some(super_prototype) = super_prototype.clone() {
                Self::object_set_entry(
                    &mut *prototype,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    super_prototype,
                );
            }
            Self::object_set_entry(
                &mut *prototype,
                "constructor".to_string(),
                class_constructor.clone(),
            );
        }

        for method in methods {
            let method_super_prototype = if method.is_static {
                super_constructor.clone()
            } else {
                super_prototype.clone()
            };
            let method_value = self.make_function_value_with_super(
                method.handler.clone(),
                env,
                false,
                method.is_async,
                method.is_generator,
                false,
                true,
                super_constructor.clone(),
                method_super_prototype,
            );
            if let Value::Function(function) = &method_value {
                class_function_ids.push(function.function_id);
            }

            if method.is_private {
                let binding = private_bindings.get(&method.name).cloned().ok_or_else(|| {
                    Error::ScriptRuntime(format!(
                        "private identifier '#{}' is not declared",
                        method.name
                    ))
                })?;
                let mut initializer = PrivateInitializerRuntime {
                    binding,
                    initializer: None,
                    value: None,
                    setter_value: None,
                };
                match method.kind {
                    ClassMethodKind::Method | ClassMethodKind::Getter => {
                        initializer.value = Some(method_value);
                    }
                    ClassMethodKind::Setter => {
                        initializer.setter_value = Some(method_value);
                    }
                }
                if method.is_static {
                    static_private_method_initializers.push(initializer);
                } else {
                    instance_private_method_initializers.push(initializer);
                }
                continue;
            }

            if method.is_static {
                let properties = self
                    .script_runtime
                    .function_public_properties
                    .entry(class_constructor_id)
                    .or_default();
                match method.kind {
                    ClassMethodKind::Method => {
                        Self::object_set_entry(properties, method.name.clone(), method_value);
                    }
                    ClassMethodKind::Getter => {
                        let getter_key = Self::object_getter_storage_key(&method.name);
                        Self::object_set_entry(properties, getter_key, method_value);
                    }
                    ClassMethodKind::Setter => {
                        let setter_key = Self::object_setter_storage_key(&method.name);
                        Self::object_set_entry(properties, setter_key, method_value);
                    }
                }
                continue;
            }

            let mut prototype = class_function.prototype_object.borrow_mut();
            match method.kind {
                ClassMethodKind::Method => {
                    Self::object_set_entry(&mut *prototype, method.name.clone(), method_value);
                }
                ClassMethodKind::Getter => {
                    let getter_key = Self::object_getter_storage_key(&method.name);
                    Self::object_set_entry(&mut *prototype, getter_key, method_value);
                }
                ClassMethodKind::Setter => {
                    let setter_key = Self::object_setter_storage_key(&method.name);
                    Self::object_set_entry(&mut *prototype, setter_key, method_value);
                }
            }
        }

        for (field_index, field) in fields.iter().enumerate() {
            let initializer = if field.is_private {
                let binding = private_bindings.get(&field.name).cloned().ok_or_else(|| {
                    Error::ScriptRuntime(format!(
                        "private identifier '#{}' is not declared",
                        field.name
                    ))
                })?;
                ConstructorInstanceInitializerRuntime::Private(PrivateInitializerRuntime {
                    binding,
                    initializer: field.initializer.clone(),
                    value: None,
                    setter_value: None,
                })
            } else {
                let field_name = if let Some(name_expr) = field.computed_name.as_ref() {
                    let key_value = self.eval_expr(name_expr, env, event_param, event)?;
                    self.property_key_to_storage_key(&key_value)
                } else {
                    field.name.clone()
                };
                ConstructorInstanceInitializerRuntime::Public(PublicFieldInitializerRuntime {
                    name: field_name,
                    initializer: field.initializer.clone(),
                })
            };
            if field.is_static {
                static_field_initializers_by_index.insert(field_index, initializer);
            } else {
                instance_field_initializers.push(initializer);
            }
        }

        if !private_bindings.is_empty() {
            for function_id in class_function_ids {
                self.script_runtime
                    .function_private_bindings
                    .insert(function_id, private_bindings.clone());
            }
        }

        enum RuntimeStaticInitializer {
            Member(ConstructorInstanceInitializerRuntime),
            Block(ScriptHandler),
        }

        let mut static_runtime_initializers = Vec::new();
        static_runtime_initializers.extend(static_private_method_initializers.into_iter().map(
            |initializer| {
                RuntimeStaticInitializer::Member(ConstructorInstanceInitializerRuntime::Private(
                    initializer,
                ))
            },
        ));
        for entry in static_initializers {
            match entry {
                ClassStaticInitializerDecl::Field(field_index) => {
                    let initializer = static_field_initializers_by_index
                        .remove(field_index)
                        .ok_or_else(|| {
                            Error::ScriptRuntime("class static field initializer is missing".into())
                        })?;
                    static_runtime_initializers.push(RuntimeStaticInitializer::Member(initializer));
                }
                ClassStaticInitializerDecl::Block(handler) => {
                    static_runtime_initializers
                        .push(RuntimeStaticInitializer::Block(handler.clone()));
                }
            }
        }

        if !static_runtime_initializers.is_empty() {
            let outer_sync_names = env
                .keys()
                .filter(|key| {
                    let key = key.as_str();
                    !Self::is_internal_env_key(key) && key != "this" && key != name
                })
                .cloned()
                .collect::<Vec<_>>();
            let mut static_env = env.clone();
            static_env.insert("this".to_string(), class_constructor.clone());
            static_env.insert(name.to_string(), class_constructor.clone());
            static_env.insert(INTERNAL_NEW_TARGET_KEY.to_string(), Value::Undefined);
            if let Some(super_constructor) = super_constructor.clone() {
                static_env.insert(
                    INTERNAL_CLASS_SUPER_PROTOTYPE_KEY.to_string(),
                    super_constructor,
                );
            }
            self.script_runtime
                .private_binding_stack
                .push(private_bindings.clone());
            let static_result = (|| -> Result<()> {
                for initializer in &static_runtime_initializers {
                    match initializer {
                        RuntimeStaticInitializer::Member(initializer) => {
                            self.apply_constructor_instance_initializer_to_receiver(
                                initializer,
                                &class_constructor,
                                &static_env,
                                event_param,
                                event,
                            )?;
                        }
                        RuntimeStaticInitializer::Block(handler) => {
                            let mut block_env = static_env.clone();
                            let scope_depth = Self::env_scope_depth(&block_env).saturating_add(1);
                            block_env.insert(
                                INTERNAL_SCOPE_DEPTH_KEY.to_string(),
                                Value::Number(scope_depth),
                            );
                            let mut local_var_names =
                                Self::collect_var_declared_names(&handler.stmts)
                                    .into_iter()
                                    .collect::<Vec<_>>();
                            local_var_names.sort();
                            for local_name in &local_var_names {
                                block_env.insert(local_name.clone(), Value::Undefined);
                                self.set_const_binding(&mut block_env, local_name, false);
                            }
                            let mut local_declared_names =
                                local_var_names.into_iter().collect::<HashSet<_>>();
                            for stmt in &handler.stmts {
                                for (name, _) in Self::direct_decl_binding_kinds(stmt) {
                                    local_declared_names.insert(name);
                                }
                            }
                            if !local_declared_names.is_empty() {
                                let mut local_binding_names =
                                    local_declared_names.iter().cloned().collect::<Vec<_>>();
                                local_binding_names.sort();
                                block_env.insert(
                                    INTERNAL_LOCAL_BINDINGS_KEY.to_string(),
                                    Self::new_array_value(
                                        local_binding_names
                                            .into_iter()
                                            .map(Value::String)
                                            .collect(),
                                    ),
                                );
                            }
                            match self.execute_stmts_with_pending_scope(
                                &handler.stmts,
                                event_param,
                                event,
                                &mut block_env,
                                false,
                            )? {
                                ExecFlow::Continue => {}
                                ExecFlow::Break(_)
                                | ExecFlow::ContinueLoop(_)
                                | ExecFlow::Return => {
                                    return Err(Error::ScriptRuntime(
                                        "invalid control flow in class static initialization block"
                                            .into(),
                                    ));
                                }
                            }
                            for sync_name in &outer_sync_names {
                                if local_declared_names.contains(sync_name) {
                                    continue;
                                }
                                if let Some(next) = block_env.get(sync_name).cloned() {
                                    static_env.insert(sync_name.clone(), next.clone());
                                    env.insert(sync_name.clone(), next);
                                }
                            }
                        }
                    }
                }
                Ok(())
            })();
            self.script_runtime.private_binding_stack.pop();
            static_result?;
        }

        let mut instance_initializers = Vec::new();
        instance_initializers.extend(
            instance_private_method_initializers
                .into_iter()
                .map(ConstructorInstanceInitializerRuntime::Private),
        );
        instance_initializers.extend(instance_field_initializers);

        if instance_initializers.is_empty() {
            self.script_runtime
                .constructor_instance_initializers
                .remove(&class_constructor_id);
        } else {
            self.script_runtime
                .constructor_instance_initializers
                .insert(class_constructor_id, instance_initializers);
        }

        env.insert(name.to_string(), class_constructor);
        self.set_const_binding(env, name, false);
        let class_value = env
            .get(name)
            .cloned()
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;
        self.sync_global_binding_if_needed(env, name, &class_value);
        self.sync_scheduled_task_captures_for_binding_if_escaping(env, name, &class_value);
        Ok(())
    }

    fn execute_export_decl_stmt(
        &mut self,
        declaration: &Stmt,
        bindings: &[(String, String)],
        pending_tdz_bindings: &mut HashSet<String>,
        initialized_var_bindings: &mut HashSet<String>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let Some(flow) = self.try_execute_declaration_stmt(
            declaration,
            pending_tdz_bindings,
            initialized_var_bindings,
            env,
            event_param,
            event,
        )?
        else {
            return self.execute_stmts_with_pending_scope(
                std::slice::from_ref(declaration),
                event_param,
                event,
                env,
                false,
            );
        };
        if matches!(flow, ExecFlow::Continue) {
            for local in Self::direct_tdz_binding_names(declaration) {
                self.mark_tdz_initialized(pending_tdz_bindings, &local);
            }
            self.register_module_named_exports(bindings);
        }
        Ok(flow)
    }

    pub(crate) fn try_execute_declaration_stmt(
        &mut self,
        stmt: &Stmt,
        pending_tdz_bindings: &mut HashSet<String>,
        initialized_var_bindings: &mut HashSet<String>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<Option<ExecFlow>> {
        match stmt {
            Stmt::ImportDecl { .. } => Ok(Some(ExecFlow::Continue)),
            Stmt::VarDecl { name, expr, kind } => {
                self.execute_var_decl_stmt(
                    name,
                    expr,
                    *kind,
                    pending_tdz_bindings,
                    initialized_var_bindings,
                    env,
                    event_param,
                    event,
                )?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::FunctionDecl {
                name,
                handler,
                is_async,
                is_generator,
            } => {
                self.execute_function_decl_stmt(
                    name,
                    handler,
                    *is_async,
                    *is_generator,
                    initialized_var_bindings,
                    env,
                );
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::ClassDecl {
                name,
                super_class,
                constructor,
                fields,
                methods,
                static_initializers,
            } => {
                self.execute_class_decl_stmt(
                    name,
                    super_class,
                    constructor,
                    fields,
                    methods,
                    static_initializers,
                    pending_tdz_bindings,
                    env,
                    event_param,
                    event,
                )?;
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::ExportDecl {
                declaration,
                bindings,
            } => Ok(Some(self.execute_export_decl_stmt(
                declaration,
                bindings,
                pending_tdz_bindings,
                initialized_var_bindings,
                env,
                event_param,
                event,
            )?)),
            Stmt::ExportNamed { bindings } => {
                self.register_module_named_exports(bindings);
                Ok(Some(ExecFlow::Continue))
            }
            Stmt::ExportDefaultExpr { expr } => {
                let value = self.eval_expr(expr, env, event_param, event)?;
                self.register_module_default_export_value(value);
                Ok(Some(ExecFlow::Continue))
            }
            _ => Ok(None),
        }
    }
}
