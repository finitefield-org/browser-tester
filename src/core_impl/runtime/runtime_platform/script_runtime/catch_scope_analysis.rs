use super::*;

impl Harness {
    fn direct_decl_names_for_catch_conflict(stmt: &Stmt) -> Vec<(String, bool)> {
        match stmt {
            Stmt::ImportDecl {
                default_binding,
                namespace_binding,
                named_bindings,
                ..
            } => {
                let mut out = Vec::new();
                if let Some(name) = default_binding {
                    out.push((name.clone(), false));
                }
                if let Some(name) = namespace_binding {
                    out.push((name.clone(), false));
                }
                out.extend(
                    named_bindings
                        .iter()
                        .map(|binding| (binding.local.clone(), false)),
                );
                out
            }
            Stmt::VarDecl { name, kind, .. } => {
                vec![(name.clone(), matches!(kind, VarDeclKind::Var))]
            }
            Stmt::FunctionDecl { name, .. } => vec![(name.clone(), false)],
            Stmt::ClassDecl { name, .. } => vec![(name.clone(), false)],
            Stmt::ExportDecl { declaration, .. } => {
                Self::direct_decl_names_for_catch_conflict(declaration)
            }
            Stmt::ArrayDestructureAssign {
                pattern,
                decl_kind: Some(kind),
                ..
            } => {
                let is_var = matches!(kind, VarDeclKind::Var);
                pattern
                    .items
                    .iter()
                    .flatten()
                    .map(|binding| (binding.target.clone(), is_var))
                    .chain(pattern.rest.iter().cloned().map(|name| (name, is_var)))
                    .collect::<Vec<_>>()
            }
            Stmt::ObjectDestructureAssign {
                pattern,
                decl_kind: Some(kind),
                ..
            } => {
                let is_var = matches!(kind, VarDeclKind::Var);
                pattern
                    .bindings
                    .iter()
                    .map(|binding| (binding.target.clone(), is_var))
                    .chain(pattern.rest.iter().cloned().map(|name| (name, is_var)))
                    .collect::<Vec<_>>()
            }
            _ => Vec::new(),
        }
    }

    fn catch_binding_names(binding: &CatchBinding) -> HashSet<String> {
        match binding {
            CatchBinding::Identifier(name) => HashSet::from([name.clone()]),
            CatchBinding::ArrayPattern(pattern) => pattern.iter().flatten().cloned().collect(),
            CatchBinding::ObjectPattern(pattern) => {
                pattern.iter().map(|(_, target)| target.clone()).collect()
            }
        }
    }

    pub(crate) fn error_to_catch_value(err: Error) -> std::result::Result<Value, Error> {
        match err {
            Error::ScriptThrown(value) => Ok(value.into_value()),
            Error::ScriptRuntime(message) => Ok(Value::String(message)),
            other => Err(other),
        }
    }

    pub(crate) fn bind_catch_binding(
        &self,
        binding: &CatchBinding,
        caught: &Value,
        env: &mut HashMap<String, Value>,
    ) -> Result<Vec<(String, Option<Value>, bool)>> {
        let mut previous = Vec::new();
        let mut seen = HashSet::new();
        let mut remember = |name: &str, env: &HashMap<String, Value>, is_const: bool| {
            if seen.insert(name.to_string()) {
                previous.push((name.to_string(), env.get(name).cloned(), is_const));
            }
        };

        match binding {
            CatchBinding::Identifier(name) => {
                remember(name, env, self.is_const_binding(env, name));
                env.insert(name.clone(), caught.clone());
                self.set_const_binding(env, name, false);
            }
            CatchBinding::ArrayPattern(pattern) => {
                let values = self.array_like_values_from_value(caught)?;
                for (index, name) in pattern.iter().enumerate() {
                    let Some(name) = name else {
                        continue;
                    };
                    remember(name, env, self.is_const_binding(env, name));
                    let value = values.get(index).cloned().unwrap_or(Value::Undefined);
                    env.insert(name.clone(), value);
                    self.set_const_binding(env, name, false);
                }
            }
            CatchBinding::ObjectPattern(pattern) => {
                let Value::Object(entries) = caught else {
                    return Err(Error::ScriptRuntime(
                        "catch object binding requires an object value".into(),
                    ));
                };
                let entries = entries.borrow();
                for (source_key, target_name) in pattern {
                    remember(target_name, env, self.is_const_binding(env, target_name));
                    let value =
                        Self::object_get_entry(&entries, source_key).unwrap_or(Value::Undefined);
                    env.insert(target_name.clone(), value);
                    self.set_const_binding(env, target_name, false);
                }
            }
        }

        Ok(previous)
    }

    pub(crate) fn restore_catch_binding(
        &self,
        previous: Vec<(String, Option<Value>, bool)>,
        env: &mut HashMap<String, Value>,
    ) {
        for (name, value, was_const) in previous {
            if let Some(value) = value {
                env.insert(name.clone(), value);
            } else {
                env.remove(&name);
            }
            self.set_const_binding(env, &name, was_const);
        }
    }

    pub(crate) fn execute_catch_block(
        &mut self,
        catch_binding: &Option<CatchBinding>,
        catch_stmts: &[Stmt],
        caught: Value,
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        let previous = if let Some(binding) = catch_binding {
            self.bind_catch_binding(binding, &caught, env)?
        } else {
            Vec::new()
        };
        if let Some(binding) = catch_binding {
            let direct_scope_stmts = match catch_stmts {
                [Stmt::Block { stmts }] => stmts.as_slice(),
                _ => catch_stmts,
            };
            let occupied_names = Self::catch_binding_names(binding);
            let allow_var_shadow_for_simple_identifier =
                matches!(binding, CatchBinding::Identifier(_));
            let simple_identifier_name = match binding {
                CatchBinding::Identifier(name) => Some(name.as_str()),
                _ => None,
            };
            for stmt in direct_scope_stmts {
                for (name, is_var_decl) in Self::direct_decl_names_for_catch_conflict(stmt) {
                    if allow_var_shadow_for_simple_identifier
                        && is_var_decl
                        && Some(name.as_str()) == simple_identifier_name
                    {
                        continue;
                    }
                    if occupied_names.contains(&name) {
                        return Err(Error::ScriptRuntime(format!(
                            "Identifier '{name}' has already been declared"
                        )));
                    }
                }
            }
        }
        let result = self.execute_stmts(catch_stmts, event_param, event, env);
        self.restore_catch_binding(previous, env);
        result
    }

    pub(crate) fn parse_function_constructor_param_names(spec: &str) -> Result<Vec<String>> {
        let mut params = Vec::new();
        for raw in spec.split(',') {
            let raw = raw.trim();
            if raw.is_empty() {
                return Err(Error::ScriptRuntime(
                    "new Function parameter name cannot be empty".into(),
                ));
            }
            if !is_ident(raw) {
                return Err(Error::ScriptRuntime(format!(
                    "new Function parameter name is invalid: {raw}"
                )));
            }
            params.push(raw.to_string());
        }
        Ok(params)
    }

    pub(crate) fn collect_function_decls(
        stmts: &[Stmt],
    ) -> HashMap<String, (ScriptHandler, bool, bool)> {
        let mut out = HashMap::new();
        for stmt in stmts {
            match stmt {
                Stmt::FunctionDecl {
                    name,
                    handler,
                    is_async,
                    is_generator,
                } => {
                    out.insert(name.clone(), (handler.clone(), *is_async, *is_generator));
                }
                Stmt::ExportDecl { declaration, .. } => {
                    if let Stmt::FunctionDecl {
                        name,
                        handler,
                        is_async,
                        is_generator,
                    } = declaration.as_ref()
                    {
                        out.insert(name.clone(), (handler.clone(), *is_async, *is_generator));
                    }
                }
                _ => {}
            }
        }
        out
    }

    pub(crate) fn collect_function_scope_bindings(handler: &ScriptHandler) -> HashSet<String> {
        let mut bindings = HashSet::new();
        for param in &handler.params {
            bindings.insert(param.name.clone());
        }
        Self::collect_scope_bindings_from_stmts(&handler.stmts, &mut bindings);
        bindings
    }

    pub(crate) fn collect_scope_bindings_from_stmts(stmts: &[Stmt], out: &mut HashSet<String>) {
        for stmt in stmts {
            Self::collect_scope_bindings_from_stmt(stmt, out);
        }
    }

    pub(crate) fn collect_scope_bindings_from_stmt(stmt: &Stmt, out: &mut HashSet<String>) {
        match stmt {
            Stmt::VarDecl { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::ImportDecl {
                default_binding,
                namespace_binding,
                named_bindings,
                ..
            } => {
                if let Some(name) = default_binding {
                    out.insert(name.clone());
                }
                if let Some(name) = namespace_binding {
                    out.insert(name.clone());
                }
                for binding in named_bindings {
                    out.insert(binding.local.clone());
                }
            }
            Stmt::ArrayDestructureAssign {
                pattern,
                decl_kind: Some(_),
                ..
            } => {
                for binding in pattern.items.iter().flatten() {
                    out.insert(binding.target.clone());
                }
                if let Some(rest) = &pattern.rest {
                    out.insert(rest.clone());
                }
            }
            Stmt::ObjectDestructureAssign {
                pattern,
                decl_kind: Some(_),
                ..
            } => {
                for binding in &pattern.bindings {
                    out.insert(binding.target.clone());
                }
                if let Some(rest) = &pattern.rest {
                    out.insert(rest.clone());
                }
            }
            Stmt::FunctionDecl { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::ClassDecl { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::ExportDecl { declaration, .. } => {
                Self::collect_scope_bindings_from_stmt(declaration, out);
            }
            Stmt::Label { stmt, .. } => {
                Self::collect_scope_bindings_from_stmt(stmt, out);
            }
            Stmt::Block { stmts } => {
                Self::collect_scope_bindings_from_stmts(stmts, out);
            }
            Stmt::ForEach {
                item_var,
                index_var,
                body,
                ..
            }
            | Stmt::ClassListForEach {
                item_var,
                index_var,
                body,
                ..
            } => {
                out.insert(item_var.clone());
                if let Some(index_var) = index_var {
                    out.insert(index_var.clone());
                }
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::For { init, body, .. } => {
                for init in init {
                    Self::collect_scope_bindings_from_stmt(init, out);
                }
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::ForIn { item_var, body, .. }
            | Stmt::ForOf { item_var, body, .. }
            | Stmt::ForAwaitOf { item_var, body, .. } => {
                out.insert(item_var.clone());
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::DoWhile { body, .. } | Stmt::While { body, .. } => {
                Self::collect_scope_bindings_from_stmts(body, out);
            }
            Stmt::Switch { clauses, .. } => {
                for clause in clauses {
                    Self::collect_scope_bindings_from_stmts(&clause.stmts, out);
                }
            }
            Stmt::Try {
                try_stmts,
                catch_binding,
                catch_stmts,
                finally_stmts,
            } => {
                Self::collect_scope_bindings_from_stmts(try_stmts, out);
                if let Some(catch_binding) = catch_binding {
                    Self::collect_scope_bindings_from_catch_binding(catch_binding, out);
                }
                if let Some(catch_stmts) = catch_stmts {
                    Self::collect_scope_bindings_from_stmts(catch_stmts, out);
                }
                if let Some(finally_stmts) = finally_stmts {
                    Self::collect_scope_bindings_from_stmts(finally_stmts, out);
                }
            }
            Stmt::If {
                then_stmts,
                else_stmts,
                ..
            } => {
                Self::collect_scope_bindings_from_stmts(then_stmts, out);
                Self::collect_scope_bindings_from_stmts(else_stmts, out);
            }
            _ => {}
        }
    }

    pub(crate) fn collect_scope_bindings_from_catch_binding(
        binding: &CatchBinding,
        out: &mut HashSet<String>,
    ) {
        match binding {
            CatchBinding::Identifier(name) => {
                out.insert(name.clone());
            }
            CatchBinding::ArrayPattern(pattern) => {
                for entry in pattern.iter().flatten() {
                    out.insert(entry.clone());
                }
            }
            CatchBinding::ObjectPattern(pattern) => {
                for (_, target) in pattern {
                    out.insert(target.clone());
                }
            }
        }
    }

    pub(crate) fn collect_function_capture_names(handler: &ScriptHandler) -> HashSet<String> {
        let local_bindings = Self::collect_function_scope_bindings(handler);
        let mut names = HashSet::new();
        for param in &handler.params {
            if let Some(default) = param.default.as_ref() {
                Self::collect_capture_names_from_expr(default, &mut names);
            }
        }
        Self::collect_capture_names_from_stmts(&handler.stmts, &mut names);
        names.retain(|name| !Self::is_internal_env_key(name) && !local_bindings.contains(name));
        names
    }

    fn collect_capture_name(name: &str, out: &mut HashSet<String>) {
        if !Self::is_internal_env_key(name) {
            out.insert(name.to_string());
        }
    }

    fn collect_nested_handler_capture_names(handler: &ScriptHandler, out: &mut HashSet<String>) {
        out.extend(Self::collect_function_capture_names(handler));
    }

    fn collect_capture_names_from_stmts(stmts: &[Stmt], out: &mut HashSet<String>) {
        for stmt in stmts {
            Self::collect_capture_names_from_stmt(stmt, out);
        }
    }

    fn collect_capture_names_from_stmt(stmt: &Stmt, out: &mut HashSet<String>) {
        match stmt {
            Stmt::ImportDecl { .. } | Stmt::ExportNamed { .. } | Stmt::Empty | Stmt::Debugger => {}
            Stmt::VarDecl { expr, .. } => {
                Self::collect_capture_names_from_expr(expr, out);
            }
            Stmt::ExportDecl { declaration, .. } | Stmt::Label { stmt: declaration, .. } => {
                Self::collect_capture_names_from_stmt(declaration, out);
            }
            Stmt::ExportDefaultExpr { expr }
            | Stmt::Throw { value: expr }
            | Stmt::Expr(expr) => {
                Self::collect_capture_names_from_expr(expr, out);
            }
            Stmt::FunctionDecl { handler, .. } => {
                Self::collect_nested_handler_capture_names(handler, out);
            }
            Stmt::ClassDecl {
                super_class,
                constructor,
                fields,
                methods,
                static_initializers,
                ..
            } => {
                if let Some(super_class) = super_class {
                    Self::collect_capture_names_from_expr(super_class, out);
                }
                if let Some(constructor) = constructor {
                    Self::collect_nested_handler_capture_names(constructor, out);
                }
                for field in fields {
                    if let Some(computed_name) = field.computed_name.as_ref() {
                        Self::collect_capture_names_from_expr(computed_name, out);
                    }
                    if let Some(initializer) = field.initializer.as_ref() {
                        Self::collect_capture_names_from_expr(initializer, out);
                    }
                }
                for method in methods {
                    Self::collect_nested_handler_capture_names(&method.handler, out);
                }
                for initializer in static_initializers {
                    if let ClassStaticInitializerDecl::Block(handler) = initializer {
                        Self::collect_nested_handler_capture_names(handler, out);
                    }
                }
            }
            Stmt::PrivateAssign { target, expr, .. } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_capture_names_from_expr(expr, out);
            }
            Stmt::Block { stmts } => {
                Self::collect_capture_names_from_stmts(stmts, out);
            }
            Stmt::VarAssign { name, expr, .. } => {
                Self::collect_capture_name(name, out);
                Self::collect_capture_names_from_expr(expr, out);
            }
            Stmt::VarUpdate { name, .. } => {
                Self::collect_capture_name(name, out);
            }
            Stmt::ArrayDestructureAssign { pattern, expr, .. } => {
                Self::collect_capture_names_from_expr(expr, out);
                for binding in pattern.items.iter().flatten() {
                    Self::collect_capture_name(&binding.target, out);
                    if let Some(default) = binding.default.as_ref() {
                        Self::collect_capture_names_from_expr(default, out);
                    }
                }
                if let Some(rest) = pattern.rest.as_ref() {
                    Self::collect_capture_name(rest, out);
                }
            }
            Stmt::ObjectDestructureAssign { pattern, expr, .. } => {
                Self::collect_capture_names_from_expr(expr, out);
                for binding in &pattern.bindings {
                    Self::collect_capture_name(&binding.target, out);
                    if let Some(default) = binding.default.as_ref() {
                        Self::collect_capture_names_from_expr(default, out);
                    }
                }
                if let Some(rest) = pattern.rest.as_ref() {
                    Self::collect_capture_name(rest, out);
                }
            }
            Stmt::ObjectAssign { target, path, expr, .. } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_exprs(path, out);
                Self::collect_capture_names_from_expr(expr, out);
            }
            Stmt::FormDataAppend {
                target_var,
                name,
                value,
                filename,
            } => {
                Self::collect_capture_name(target_var, out);
                Self::collect_capture_names_from_expr(name, out);
                Self::collect_capture_names_from_expr(value, out);
                if let Some(filename) = filename.as_ref() {
                    Self::collect_capture_names_from_expr(filename, out);
                }
            }
            Stmt::DomAssign { target, expr, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_expr(expr, out);
            }
            Stmt::ClassListCall {
                target,
                class_names,
                force,
                ..
            } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_exprs(class_names, out);
                if let Some(force) = force.as_ref() {
                    Self::collect_capture_names_from_expr(force, out);
                }
            }
            Stmt::ClassListForEach { target, body, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_stmts(body, out);
            }
            Stmt::DomSetAttribute { target, value, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_expr(value, out);
            }
            Stmt::DomRemoveAttribute { target, .. } | Stmt::NodeRemove { target } => {
                Self::collect_capture_names_from_dom_query(target, out);
            }
            Stmt::NodeTreeMutation {
                target,
                child,
                reference,
                ..
            } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_expr(child, out);
                if let Some(reference) = reference.as_ref() {
                    Self::collect_capture_names_from_expr(reference, out);
                }
            }
            Stmt::InsertAdjacentElement { target, node, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_expr(node, out);
            }
            Stmt::InsertAdjacentText { target, text, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_expr(text, out);
            }
            Stmt::InsertAdjacentHTML {
                target,
                position,
                html,
            } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_expr(position, out);
                Self::collect_capture_names_from_expr(html, out);
            }
            Stmt::SetTimeout { handler, delay_ms } | Stmt::SetInterval { handler, delay_ms } => {
                Self::collect_capture_names_from_timer_invocation(handler, out);
                Self::collect_capture_names_from_expr(delay_ms, out);
            }
            Stmt::QueueMicrotask { handler } => {
                Self::collect_nested_handler_capture_names(handler, out);
            }
            Stmt::ClearTimeout { timer_id } => {
                Self::collect_capture_names_from_expr(timer_id, out);
            }
            Stmt::ForEach { target, body, .. } => {
                if let Some(target) = target.as_ref() {
                    Self::collect_capture_names_from_dom_query(target, out);
                }
                Self::collect_capture_names_from_stmts(body, out);
            }
            Stmt::ArrayForEach { target, callback } => {
                Self::collect_capture_name(target, out);
                Self::collect_nested_handler_capture_names(callback, out);
            }
            Stmt::ArrayForEachExpr { target, callback } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_nested_handler_capture_names(callback, out);
            }
            Stmt::For {
                init,
                cond,
                post,
                body,
            } => {
                Self::collect_capture_names_from_stmts(init, out);
                if let Some(cond) = cond.as_ref() {
                    Self::collect_capture_names_from_expr(cond, out);
                }
                Self::collect_capture_names_from_stmts(post, out);
                Self::collect_capture_names_from_stmts(body, out);
            }
            Stmt::ForIn { iterable, body, .. }
            | Stmt::ForOf { iterable, body, .. }
            | Stmt::ForAwaitOf { iterable, body, .. } => {
                Self::collect_capture_names_from_expr(iterable, out);
                Self::collect_capture_names_from_stmts(body, out);
            }
            Stmt::DoWhile { cond, body } | Stmt::While { cond, body } => {
                Self::collect_capture_names_from_expr(cond, out);
                Self::collect_capture_names_from_stmts(body, out);
            }
            Stmt::Switch { expr, clauses } => {
                Self::collect_capture_names_from_expr(expr, out);
                for clause in clauses {
                    if let Some(test) = clause.test.as_ref() {
                        Self::collect_capture_names_from_expr(test, out);
                    }
                    Self::collect_capture_names_from_stmts(&clause.stmts, out);
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
            Stmt::Try {
                try_stmts,
                catch_stmts,
                finally_stmts,
                ..
            } => {
                Self::collect_capture_names_from_stmts(try_stmts, out);
                if let Some(catch_stmts) = catch_stmts.as_ref() {
                    Self::collect_capture_names_from_stmts(catch_stmts, out);
                }
                if let Some(finally_stmts) = finally_stmts.as_ref() {
                    Self::collect_capture_names_from_stmts(finally_stmts, out);
                }
            }
            Stmt::Return { value } => {
                if let Some(value) = value.as_ref() {
                    Self::collect_capture_names_from_expr(value, out);
                }
            }
            Stmt::If {
                cond,
                then_stmts,
                else_stmts,
            } => {
                Self::collect_capture_names_from_expr(cond, out);
                Self::collect_capture_names_from_stmts(then_stmts, out);
                Self::collect_capture_names_from_stmts(else_stmts, out);
            }
            Stmt::EventCall { event_var, .. } => {
                Self::collect_capture_name(event_var, out);
            }
            Stmt::ListenerMutation {
                target,
                event_type,
                handler,
                ..
            } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_expr(event_type, out);
                Self::collect_nested_handler_capture_names(handler, out);
            }
            Stmt::DispatchEvent { target, event_type } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_expr(event_type, out);
            }
            Stmt::DomMethodCall { target, arg, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
                if let Some(arg) = arg.as_ref() {
                    Self::collect_capture_names_from_expr(arg, out);
                }
            }
        }
    }

    fn collect_capture_names_from_exprs(exprs: &[Expr], out: &mut HashSet<String>) {
        for expr in exprs {
            Self::collect_capture_names_from_expr(expr, out);
        }
    }

    fn collect_capture_names_from_expr(expr: &Expr, out: &mut HashSet<String>) {
        match expr {
            Expr::String(_)
            | Expr::Bool(_)
            | Expr::Null
            | Expr::Undefined
            | Expr::Number(_)
            | Expr::Float(_)
            | Expr::BigInt(_)
            | Expr::DateNow
            | Expr::PerformanceNow
            | Expr::RegexLiteral { .. }
            | Expr::RegExpConstructor
            | Expr::MathConst(_)
            | Expr::StringConstructor
            | Expr::NumberConst(_)
            | Expr::BlobConstructor
            | Expr::UrlConstructor
            | Expr::ArrayBufferConstructor
            | Expr::TypedArrayConstructorRef(_)
            | Expr::PromiseConstructor
            | Expr::MapConstructor
            | Expr::WeakMapConstructor
            | Expr::UrlSearchParamsConstructor
            | Expr::SetConstructor
            | Expr::WeakSetConstructor
            | Expr::SymbolConstructor
            | Expr::SymbolStaticProperty(_)
            | Expr::TypedArrayStaticBytesPerElement(_)
            | Expr::ImportMeta
            | Expr::NewTarget
            | Expr::CreateElement(_)
            | Expr::CreateTextNode(_)
            | Expr::DocumentHasFocus => {}
            Expr::DateNew { args }
            | Expr::DateUtc { args }
            | Expr::IntlStaticMethod { args, .. }
            | Expr::IntlConstruct { args }
            | Expr::RegExpStaticMethod { args, .. }
            | Expr::MathMethod { args, .. }
            | Expr::StringStaticMethod { args, .. }
            | Expr::NumberMethod { args, .. }
            | Expr::BigIntMethod { args, .. }
            | Expr::UrlStaticMethod { args, .. }
            | Expr::PromiseStaticMethod { args, .. }
            | Expr::MapStaticMethod { args, .. }
            | Expr::SymbolStaticMethod { args, .. }
            | Expr::TypedArrayStaticMethod { args, .. }
            | Expr::FunctionConstructor { args }
            | Expr::HistoryMethodCall { args, .. }
            | Expr::ClipboardMethodCall { args, .. }
            | Expr::ArrayLiteral(args)
            | Expr::ArrayConstruct { args, .. }
            | Expr::Comma(args)
            | Expr::Add(args) => {
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::DateParse(inner)
            | Expr::ArrayBufferIsView(inner)
            | Expr::EncodeUri(inner)
            | Expr::EncodeUriComponent(inner)
            | Expr::DecodeUri(inner)
            | Expr::DecodeUriComponent(inner)
            | Expr::Escape(inner)
            | Expr::Unescape(inner)
            | Expr::IsNaN(inner)
            | Expr::IsFinite(inner)
            | Expr::Atob(inner)
            | Expr::Btoa(inner)
            | Expr::ParseFloat(inner)
            | Expr::JsonParse(inner)
            | Expr::ObjectGetOwnPropertyNames(inner)
            | Expr::ObjectGetOwnPropertySymbols(inner)
            | Expr::ObjectKeys(inner)
            | Expr::ObjectValues(inner)
            | Expr::ObjectEntries(inner)
            | Expr::ObjectGetPrototypeOf(inner)
            | Expr::ObjectFreeze(inner)
            | Expr::ReflectOwnKeys(inner)
            | Expr::ArrayIsArray(inner)
            | Expr::StringToUpperCase(inner)
            | Expr::StringToLowerCase(inner)
            | Expr::StringIsWellFormed(inner)
            | Expr::StringToWellFormed(inner)
            | Expr::StringValueOf(inner)
            | Expr::StringToString(inner)
            | Expr::MatchMedia(inner)
            | Expr::Alert(inner)
            | Expr::Confirm(inner)
            | Expr::Neg(inner)
            | Expr::Pos(inner)
            | Expr::BitNot(inner)
            | Expr::Not(inner)
            | Expr::Void(inner)
            | Expr::Delete(inner)
            | Expr::TypeOf(inner)
            | Expr::Await(inner)
            | Expr::Yield(inner)
            | Expr::YieldStar(inner)
            | Expr::Spread(inner) => {
                Self::collect_capture_names_from_expr(inner, out);
            }
            Expr::DateGetTime(target)
            | Expr::DateToIsoString(target)
            | Expr::DateGetUTCFullYear(target)
            | Expr::DateGetFullYear(target)
            | Expr::DateGetMonth(target)
            | Expr::DateGetDate(target)
            | Expr::DateGetHours(target)
            | Expr::DateGetMinutes(target)
            | Expr::DateGetSeconds(target)
            | Expr::ArrayBufferDetached(target)
            | Expr::ArrayBufferMaxByteLength(target)
            | Expr::ArrayBufferResizable(target)
            | Expr::MapMethod { target, .. }
            | Expr::UrlSearchParamsMethod { target, .. }
            | Expr::SetMethod { target, .. }
            | Expr::TypedArrayByteLength(target)
            | Expr::TypedArrayByteOffset(target)
            | Expr::TypedArrayBuffer(target)
            | Expr::TypedArrayBytesPerElement(target)
            | Expr::ArrayLength(target)
            | Expr::ArrayPop(target)
            | Expr::ArrayShift(target)
            | Expr::ObjectGet { target, .. }
            | Expr::ObjectPathGet { target, .. }
            | Expr::ObjectHasOwnProperty { target, .. }
            | Expr::EventProp {
                event_var: target, ..
            }
            | Expr::Var(target) => {
                Self::collect_capture_name(target, out);
            }
            Expr::FunctionCall { target, args } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::DateSetTime { target, value } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_expr(value, out);
            }
            Expr::IntlFormatterConstruct {
                locales, options, ..
            } => {
                if let Some(locales) = locales.as_ref() {
                    Self::collect_capture_names_from_expr(locales, out);
                }
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::IntlFormat { formatter, value }
            | Expr::IntlDateTimeFormatToParts { formatter, value } => {
                Self::collect_capture_names_from_expr(formatter, out);
                if let Some(value) = value.as_ref() {
                    Self::collect_capture_names_from_expr(value, out);
                }
            }
            Expr::IntlFormatGetter { formatter }
            | Expr::IntlCollatorCompareGetter { collator: formatter }
            | Expr::IntlDateTimeResolvedOptions { formatter }
            | Expr::RegexToString { regex: formatter } => {
                Self::collect_capture_names_from_expr(formatter, out);
            }
            Expr::IntlCollatorCompare {
                collator,
                left,
                right,
            } => {
                Self::collect_capture_names_from_expr(collator, out);
                Self::collect_capture_names_from_expr(left, out);
                Self::collect_capture_names_from_expr(right, out);
            }
            Expr::IntlDateTimeFormatRange {
                formatter,
                start,
                end,
            }
            | Expr::IntlDateTimeFormatRangeToParts {
                formatter,
                start,
                end,
            } => {
                Self::collect_capture_names_from_expr(formatter, out);
                Self::collect_capture_names_from_expr(start, out);
                Self::collect_capture_names_from_expr(end, out);
            }
            Expr::IntlDisplayNamesOf {
                display_names,
                code,
            } => {
                Self::collect_capture_names_from_expr(display_names, out);
                Self::collect_capture_names_from_expr(code, out);
            }
            Expr::IntlPluralRulesSelect {
                plural_rules,
                value,
            } => {
                Self::collect_capture_names_from_expr(plural_rules, out);
                Self::collect_capture_names_from_expr(value, out);
            }
            Expr::IntlPluralRulesSelectRange {
                plural_rules,
                start,
                end,
            } => {
                Self::collect_capture_names_from_expr(plural_rules, out);
                Self::collect_capture_names_from_expr(start, out);
                Self::collect_capture_names_from_expr(end, out);
            }
            Expr::IntlRelativeTimeFormat {
                formatter,
                value,
                unit,
            }
            | Expr::IntlRelativeTimeFormatToParts {
                formatter,
                value,
                unit,
            } => {
                Self::collect_capture_names_from_expr(formatter, out);
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(unit, out);
            }
            Expr::IntlSegmenterSegment { segmenter, value } => {
                Self::collect_capture_names_from_expr(segmenter, out);
                Self::collect_capture_names_from_expr(value, out);
            }
            Expr::IntlLocaleConstruct { tag, options, .. } => {
                Self::collect_capture_names_from_expr(tag, out);
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::IntlLocaleMethod { locale, .. } => {
                Self::collect_capture_names_from_expr(locale, out);
            }
            Expr::RegexNew { pattern, flags } => {
                Self::collect_capture_names_from_expr(pattern, out);
                if let Some(flags) = flags.as_ref() {
                    Self::collect_capture_names_from_expr(flags, out);
                }
            }
            Expr::RegexTest { regex, input } | Expr::RegexExec { regex, input } => {
                Self::collect_capture_names_from_expr(regex, out);
                Self::collect_capture_names_from_expr(input, out);
            }
            Expr::StringConstruct { value, .. }
            | Expr::BooleanConstruct { value, .. }
            | Expr::NumberConstruct { value, .. }
            | Expr::BigIntConstruct { value, .. }
            | Expr::ObjectConstruct { value } => {
                if let Some(value) = value.as_ref() {
                    Self::collect_capture_names_from_expr(value, out);
                }
            }
            Expr::NumberInstanceMethod { value, args, .. }
            | Expr::BigIntInstanceMethod { value, args, .. } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::BlobConstruct { parts, options, .. } => {
                if let Some(parts) = parts.as_ref() {
                    Self::collect_capture_names_from_expr(parts, out);
                }
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::UrlConstruct { input, base, .. } => {
                if let Some(input) = input.as_ref() {
                    Self::collect_capture_names_from_expr(input, out);
                }
                if let Some(base) = base.as_ref() {
                    Self::collect_capture_names_from_expr(base, out);
                }
            }
            Expr::ArrayBufferConstruct {
                byte_length,
                options,
                ..
            } => {
                if let Some(byte_length) = byte_length.as_ref() {
                    Self::collect_capture_names_from_expr(byte_length, out);
                }
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::ArrayBufferResize {
                target,
                new_byte_length,
            } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_expr(new_byte_length, out);
            }
            Expr::ArrayBufferSlice { target, start, end } => {
                Self::collect_capture_name(target, out);
                if let Some(start) = start.as_ref() {
                    Self::collect_capture_names_from_expr(start, out);
                }
                if let Some(end) = end.as_ref() {
                    Self::collect_capture_names_from_expr(end, out);
                }
            }
            Expr::ArrayBufferTransfer { target, .. } => {
                Self::collect_capture_name(target, out);
            }
            Expr::TypedArrayConstruct { args, .. } => {
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::TypedArrayConstructWithCallee { callee, args, .. } => {
                Self::collect_capture_names_from_expr(callee, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::PromiseConstruct { executor, .. }
            | Expr::MapConstruct {
                iterable: executor, ..
            }
            | Expr::WeakMapConstruct {
                iterable: executor, ..
            }
            | Expr::UrlSearchParamsConstruct { init: executor, .. }
            | Expr::SetConstruct {
                iterable: executor, ..
            }
            | Expr::WeakSetConstruct {
                iterable: executor, ..
            }
            | Expr::SymbolConstruct {
                description: executor,
                ..
            } => {
                if let Some(executor) = executor.as_ref() {
                    Self::collect_capture_names_from_expr(executor, out);
                }
            }
            Expr::PromiseMethod { target, args, .. } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::TypedArrayMethod { target, args, .. } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::ParseInt { value, radix } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(radix) = radix.as_ref() {
                    Self::collect_capture_names_from_expr(radix, out);
                }
            }
            Expr::JsonStringify {
                value,
                replacer,
                space,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(replacer) = replacer.as_ref() {
                    Self::collect_capture_names_from_expr(replacer, out);
                }
                if let Some(space) = space.as_ref() {
                    Self::collect_capture_names_from_expr(space, out);
                }
            }
            Expr::ObjectLiteral(entries) => {
                for entry in entries {
                    Self::collect_capture_names_from_object_literal_entry(entry, out);
                }
            }
            Expr::ObjectGetOwnPropertyDescriptor {
                object,
                key,
            }
            | Expr::ObjectHasOwn { object, key } => {
                Self::collect_capture_names_from_expr(object, out);
                Self::collect_capture_names_from_expr(key, out);
            }
            Expr::ObjectDefineProperty {
                object,
                key,
                descriptor,
            } => {
                Self::collect_capture_names_from_expr(object, out);
                Self::collect_capture_names_from_expr(key, out);
                Self::collect_capture_names_from_expr(descriptor, out);
            }
            Expr::ReflectSet {
                target,
                key,
                value,
                receiver,
            } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_capture_names_from_expr(key, out);
                Self::collect_capture_names_from_expr(value, out);
                if let Some(receiver) = receiver.as_ref() {
                    Self::collect_capture_names_from_expr(receiver, out);
                }
            }
            Expr::ArrayFrom { source, map_fn } => {
                Self::collect_capture_names_from_expr(source, out);
                if let Some(map_fn) = map_fn.as_ref() {
                    Self::collect_capture_names_from_expr(map_fn, out);
                }
            }
            Expr::ArrayIndex { target, index } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_expr(index, out);
            }
            Expr::ArrayPush { target, args } | Expr::ArrayUnshift { target, args } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::ArrayMap { target, callback }
            | Expr::ArrayFilter { target, callback }
            | Expr::ArrayForEach { target, callback }
            | Expr::ArrayFind { target, callback }
            | Expr::ArrayFindIndex { target, callback }
            | Expr::ArraySome { target, callback }
            | Expr::ArrayEvery { target, callback } => {
                Self::collect_capture_name(target, out);
                Self::collect_nested_handler_capture_names(callback, out);
            }
            Expr::ArrayReduce {
                target,
                callback,
                initial,
            } => {
                Self::collect_capture_name(target, out);
                Self::collect_nested_handler_capture_names(callback, out);
                if let Some(initial) = initial.as_ref() {
                    Self::collect_capture_names_from_expr(initial, out);
                }
            }
            Expr::ArraySlice { target, start, end } => {
                Self::collect_capture_name(target, out);
                if let Some(start) = start.as_ref() {
                    Self::collect_capture_names_from_expr(start, out);
                }
                if let Some(end) = end.as_ref() {
                    Self::collect_capture_names_from_expr(end, out);
                }
            }
            Expr::ArraySplice {
                target,
                start,
                delete_count,
                items,
            } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_expr(start, out);
                if let Some(delete_count) = delete_count.as_ref() {
                    Self::collect_capture_names_from_expr(delete_count, out);
                }
                Self::collect_capture_names_from_exprs(items, out);
            }
            Expr::ArrayJoin { target, separator } => {
                Self::collect_capture_name(target, out);
                if let Some(separator) = separator.as_ref() {
                    Self::collect_capture_names_from_expr(separator, out);
                }
            }
            Expr::ArraySort { target, comparator } => {
                Self::collect_capture_name(target, out);
                if let Some(comparator) = comparator.as_ref() {
                    Self::collect_capture_names_from_expr(comparator, out);
                }
            }
            Expr::StringTrim { value, .. } => {
                Self::collect_capture_names_from_expr(value, out);
            }
            Expr::StringIncludes {
                value,
                search,
                position,
            }
            | Expr::StringStartsWith {
                value,
                search,
                position,
            }
            | Expr::StringIndexOf {
                value,
                search,
                position,
            }
            | Expr::StringLastIndexOf {
                value,
                search,
                position,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(search, out);
                if let Some(position) = position.as_ref() {
                    Self::collect_capture_names_from_expr(position, out);
                }
            }
            Expr::StringEndsWith {
                value,
                search,
                length,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(search, out);
                if let Some(length) = length.as_ref() {
                    Self::collect_capture_names_from_expr(length, out);
                }
            }
            Expr::StringSlice { value, start, end }
            | Expr::StringSubstring { value, start, end } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(start) = start.as_ref() {
                    Self::collect_capture_names_from_expr(start, out);
                }
                if let Some(end) = end.as_ref() {
                    Self::collect_capture_names_from_expr(end, out);
                }
            }
            Expr::StringMatch { value, pattern }
            | Expr::StringSearch { value, pattern } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(pattern, out);
            }
            Expr::StringSplit {
                value,
                separator,
                limit,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(separator) = separator.as_ref() {
                    Self::collect_capture_names_from_expr(separator, out);
                }
                if let Some(limit) = limit.as_ref() {
                    Self::collect_capture_names_from_expr(limit, out);
                }
            }
            Expr::StringReplace { value, from, to }
            | Expr::StringReplaceAll { value, from, to } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(from, out);
                Self::collect_capture_names_from_expr(to, out);
            }
            Expr::StringCharAt { value, index }
            | Expr::StringCharCodeAt { value, index }
            | Expr::StringCodePointAt { value, index }
            | Expr::StringAt { value, index } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(index) = index.as_ref() {
                    Self::collect_capture_names_from_expr(index, out);
                }
            }
            Expr::StringConcat { value, args } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::StringRepeat { value, count } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(count, out);
            }
            Expr::StringPadStart {
                value,
                target_length,
                pad,
            }
            | Expr::StringPadEnd {
                value,
                target_length,
                pad,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(target_length, out);
                if let Some(pad) = pad.as_ref() {
                    Self::collect_capture_names_from_expr(pad, out);
                }
            }
            Expr::StringLocaleCompare {
                value,
                compare,
                locales,
                options,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(compare, out);
                if let Some(locales) = locales.as_ref() {
                    Self::collect_capture_names_from_expr(locales, out);
                }
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::StructuredClone { value, options } | Expr::Fetch { request: value, options } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::MatchMediaProp { query, .. } => {
                Self::collect_capture_names_from_expr(query, out);
            }
            Expr::Prompt { message, default } => {
                Self::collect_capture_names_from_expr(message, out);
                if let Some(default) = default.as_ref() {
                    Self::collect_capture_names_from_expr(default, out);
                }
            }
            Expr::ImportCall { module, options } => {
                Self::collect_capture_names_from_expr(module, out);
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::Call { target, args, .. } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::MemberCall { target, args, .. }
            | Expr::PrivateMemberCall { target, args, .. } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::MemberGet { target, .. }
            | Expr::PrivateMemberGet { target, .. }
            | Expr::PrivateIn { target, .. } => {
                Self::collect_capture_names_from_expr(target, out);
            }
            Expr::IndexGet { target, index, .. } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_capture_names_from_expr(index, out);
            }
            Expr::DomRef(query)
            | Expr::QuerySelectorAllLength { target: query }
            | Expr::FormElementsLength { form: query }
            | Expr::DomGetAttribute { target: query, .. }
            | Expr::DomHasAttribute { target: query, .. } => {
                Self::collect_capture_names_from_dom_query(query, out);
            }
            Expr::SetTimeout { handler, delay_ms } | Expr::SetInterval { handler, delay_ms } => {
                Self::collect_capture_names_from_timer_invocation(handler, out);
                Self::collect_capture_names_from_expr(delay_ms, out);
            }
            Expr::RequestAnimationFrame { callback } => {
                Self::collect_capture_names_from_timer_callback(callback, out);
            }
            Expr::Function { handler, .. } | Expr::QueueMicrotask { handler } => {
                Self::collect_nested_handler_capture_names(handler, out);
            }
            Expr::Binary { left, right, .. } => {
                Self::collect_capture_names_from_expr(left, out);
                Self::collect_capture_names_from_expr(right, out);
            }
            Expr::DomRead { target, .. }
            | Expr::DomMatches { target, .. }
            | Expr::DomClosest { target, .. }
            | Expr::DomComputedStyleProperty { target, .. }
            | Expr::ClassListContains { target, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
            }
            Expr::LocationMethodCall { url, .. } => {
                if let Some(url) = url.as_ref() {
                    Self::collect_capture_names_from_expr(url, out);
                }
            }
            Expr::FormDataNew { form, submitter } => {
                if let Some(form) = form.as_ref() {
                    Self::collect_capture_names_from_dom_query(form, out);
                }
                if let Some(submitter) = submitter.as_ref() {
                    Self::collect_capture_names_from_dom_query(submitter, out);
                }
            }
            Expr::FormDataGet { source, .. }
            | Expr::FormDataHas { source, .. }
            | Expr::FormDataGetAll { source, .. }
            | Expr::FormDataGetAllLength { source, .. } => {
                Self::collect_capture_names_from_form_data_source(source, out);
            }
            Expr::Ternary {
                cond,
                on_true,
                on_false,
            } => {
                Self::collect_capture_names_from_expr(cond, out);
                Self::collect_capture_names_from_expr(on_true, out);
                Self::collect_capture_names_from_expr(on_false, out);
            }
        }
    }

    fn collect_capture_names_from_object_literal_key(
        key: &ObjectLiteralKey,
        out: &mut HashSet<String>,
    ) {
        if let ObjectLiteralKey::Computed(expr) = key {
            Self::collect_capture_names_from_expr(expr, out);
        }
    }

    fn collect_capture_names_from_object_literal_entry(
        entry: &ObjectLiteralEntry,
        out: &mut HashSet<String>,
    ) {
        match entry {
            ObjectLiteralEntry::Pair(key, value) => {
                Self::collect_capture_names_from_object_literal_key(key, out);
                Self::collect_capture_names_from_expr(value, out);
            }
            ObjectLiteralEntry::ProtoSetter(value) | ObjectLiteralEntry::Spread(value) => {
                Self::collect_capture_names_from_expr(value, out);
            }
            ObjectLiteralEntry::Getter(key, handler)
            | ObjectLiteralEntry::Setter(key, handler) => {
                Self::collect_capture_names_from_object_literal_key(key, out);
                Self::collect_nested_handler_capture_names(handler, out);
            }
        }
    }

    fn collect_capture_names_from_dom_query(query: &DomQuery, out: &mut HashSet<String>) {
        match query {
            DomQuery::DocumentRoot
            | DomQuery::DocumentBody
            | DomQuery::DocumentHead
            | DomQuery::DocumentElement
            | DomQuery::ActiveElement
            | DomQuery::ById(_)
            | DomQuery::BySelector(_)
            | DomQuery::BySelectorAll { .. } => {}
            DomQuery::BySelectorAllIndex { index, .. } => {
                Self::collect_capture_names_from_dom_index(index, out);
            }
            DomQuery::QuerySelector { target, .. }
            | DomQuery::QuerySelectorAll { target, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
            }
            DomQuery::Index { target, index }
            | DomQuery::QuerySelectorAllIndex { target, index, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_dom_index(index, out);
            }
            DomQuery::FormElementsIndex { form, index } => {
                Self::collect_capture_names_from_dom_query(form, out);
                Self::collect_capture_names_from_dom_index(index, out);
            }
            DomQuery::Var(name) => {
                Self::collect_capture_name(name, out);
            }
            DomQuery::VarPath { base, .. } => {
                Self::collect_capture_name(base, out);
            }
        }
    }

    fn collect_capture_names_from_dom_index(index: &DomIndex, out: &mut HashSet<String>) {
        let DomIndex::Dynamic(raw) = index else {
            return;
        };
        if let Ok(expr) = crate::core_impl::parser::api::parse_expr(raw) {
            Self::collect_capture_names_from_expr(&expr, out);
            return;
        }
        Self::collect_capture_name(raw, out);
    }

    fn collect_capture_names_from_form_data_source(
        source: &FormDataSource,
        out: &mut HashSet<String>,
    ) {
        match source {
            FormDataSource::New { form, submitter } => {
                if let Some(form) = form.as_ref() {
                    Self::collect_capture_names_from_dom_query(form, out);
                }
                if let Some(submitter) = submitter.as_ref() {
                    Self::collect_capture_names_from_dom_query(submitter, out);
                }
            }
            FormDataSource::Var(name) => {
                Self::collect_capture_name(name, out);
            }
        }
    }

    fn collect_capture_names_from_timer_invocation(
        invocation: &TimerInvocation,
        out: &mut HashSet<String>,
    ) {
        Self::collect_capture_names_from_timer_callback(&invocation.callback, out);
        Self::collect_capture_names_from_exprs(&invocation.args, out);
    }

    fn collect_capture_names_from_timer_callback(
        callback: &TimerCallback,
        out: &mut HashSet<String>,
    ) {
        match callback {
            TimerCallback::Inline(handler) => {
                Self::collect_nested_handler_capture_names(handler, out);
            }
            TimerCallback::Reference(name) => {
                Self::collect_capture_name(name, out);
            }
        }
    }

    pub(crate) fn resolve_pending_function_decl(
        &mut self,
        name: &str,
        env: &HashMap<String, Value>,
    ) -> Option<Value> {
        let mut resolved = None;
        for scope in self.script_runtime.pending_function_decls.iter().rev() {
            let Some((handler, is_async, is_generator)) = scope.get(name) else {
                continue;
            };
            resolved = Some((handler.clone(), *is_async, *is_generator));
            break;
        }
        let (handler, is_async, is_generator) = resolved?;
        Some(self.make_function_value(handler, env, false, is_async, is_generator, false, false))
    }

    pub(crate) fn sync_listener_capture_env_if_shared(&mut self, env: &HashMap<String, Value>) {
        let Some(frame_index) = self
            .script_runtime
            .listener_capture_env_stack
            .iter()
            .rev()
            .position(|frame| frame.shared_env.is_some())
        else {
            return;
        };
        let frame_index = self.script_runtime.listener_capture_env_stack.len() - 1 - frame_index;
        let frame = &self.script_runtime.listener_capture_env_stack[frame_index];
        let Some(shared_env) = frame.shared_env.as_ref() else {
            return;
        };
        if frame.shared_env_owned_by_scope {
            if frame.pending_env_updates.is_empty() {
                return;
            }
            let mut shared_env = shared_env.borrow_mut();
            for (name, next) in &frame.pending_env_updates {
                if Self::is_internal_env_key(name) {
                    continue;
                }
                match next {
                    Some(value) => {
                        shared_env.insert(name.clone(), value.clone());
                    }
                    None => {
                        shared_env.remove(name);
                    }
                }
            }
            return;
        }
        let allow_local_bindings = frame.shared_env_owned_by_scope;
        let restricted_names =
            (!allow_local_bindings).then(|| Self::env_local_or_lexical_binding_names(env));
        let is_restricted_name = |name: &str| {
            restricted_names
                .as_ref()
                .is_some_and(|names| names.contains(name))
        };
        let shared_snapshot = shared_env.borrow();
        let shared_keys = shared_snapshot
            .keys()
            .filter(|name| !Self::is_internal_env_key(name))
            .cloned()
            .collect::<Vec<_>>();
        let mut changed_entries = Vec::new();
        let mut removed_entries = Vec::new();

        for name in &shared_keys {
            let previous = shared_snapshot
                .get(name)
                .expect("shared key should still exist in snapshot");
            match frame.pending_env_updates.get(name) {
                Some(Some(next)) => {
                    if !self.strict_equal(previous, next) {
                        changed_entries.push((name.clone(), next.clone()));
                    }
                }
                Some(None) => removed_entries.push(name.clone()),
                None if is_restricted_name(name) => {}
                None => match env.get(name) {
                    Some(next) if !self.strict_equal(previous, next) => {
                        changed_entries.push((name.clone(), next.clone()));
                    }
                    None => removed_entries.push(name.clone()),
                    _ => {}
                },
            }
        }

        let mut added_entries = Vec::new();
        for (name, next) in &frame.pending_env_updates {
            let Some(next) = next else {
                continue;
            };
            if Self::is_internal_env_key(name)
                || is_restricted_name(name)
                || shared_snapshot.contains_key(name)
            {
                continue;
            }
            added_entries.push((name.clone(), next.clone()));
        }
        for (name, next) in env {
            if Self::is_internal_env_key(name)
                || is_restricted_name(name)
                || shared_snapshot.contains_key(name)
                || frame.pending_env_updates.contains_key(name)
            {
                continue;
            }
            added_entries.push((name.clone(), next.clone()));
        }
        drop(shared_snapshot);

        if changed_entries.is_empty() && removed_entries.is_empty() && added_entries.is_empty() {
            return;
        }

        let mut shared_env = shared_env.borrow_mut();
        for (name, value) in changed_entries {
            shared_env.insert(name, value);
        }
        for name in removed_entries {
            shared_env.remove(&name);
        }
        for (name, value) in added_entries {
            shared_env.insert(name, value);
        }
    }
}
