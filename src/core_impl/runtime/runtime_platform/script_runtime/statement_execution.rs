use super::*;

impl Harness {
    const EXEC_STMTS_STACK_RED_ZONE: usize = 64 * 1024;
    const EXEC_STMTS_STACK_SIZE: usize = 32 * 1024 * 1024;

    pub(crate) fn execute_stmts(
        &mut self,
        stmts: &[Stmt],
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        stacker::maybe_grow(
            Self::EXEC_STMTS_STACK_RED_ZONE,
            Self::EXEC_STMTS_STACK_SIZE,
            || self.execute_stmts_impl(stmts, event_param, event, env),
        )
    }

    fn direct_decl_binding_kinds(stmt: &Stmt) -> Vec<(String, bool)> {
        match stmt {
            Stmt::ImportDecl {
                default_binding,
                namespace_binding,
                named_bindings,
                ..
            } => {
                let mut out = Vec::new();
                if let Some(name) = default_binding {
                    out.push((name.clone(), true));
                }
                if let Some(name) = namespace_binding {
                    out.push((name.clone(), true));
                }
                out.extend(
                    named_bindings
                        .iter()
                        .map(|binding| (binding.local.clone(), true)),
                );
                out
            }
            Stmt::VarDecl { name, kind, .. } => {
                vec![(name.clone(), !matches!(kind, VarDeclKind::Var))]
            }
            Stmt::FunctionDecl { name, .. } => vec![(name.clone(), false)],
            Stmt::ClassDecl { name, .. } => vec![(name.clone(), true)],
            Stmt::ExportDecl { declaration, .. } => Self::direct_decl_binding_kinds(declaration),
            Stmt::ArrayDestructureAssign {
                targets,
                decl_kind: Some(kind),
                ..
            } => {
                let is_lexical = !matches!(kind, VarDeclKind::Var);
                targets
                    .iter()
                    .flatten()
                    .cloned()
                    .map(|name| (name, is_lexical))
                    .collect()
            }
            Stmt::ObjectDestructureAssign {
                bindings,
                decl_kind: Some(kind),
                ..
            } => {
                let is_lexical = !matches!(kind, VarDeclKind::Var);
                bindings
                    .iter()
                    .map(|(_, target_name)| (target_name.clone(), is_lexical))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    fn direct_tdz_binding_names(stmt: &Stmt) -> Vec<String> {
        match stmt {
            Stmt::VarDecl {
                name,
                kind: VarDeclKind::Let | VarDeclKind::Const,
                ..
            } => vec![name.clone()],
            Stmt::ClassDecl { name, .. } => vec![name.clone()],
            Stmt::ArrayDestructureAssign {
                targets,
                decl_kind: Some(VarDeclKind::Let | VarDeclKind::Const),
                ..
            } => targets.iter().flatten().cloned().collect(),
            Stmt::ObjectDestructureAssign {
                bindings,
                decl_kind: Some(VarDeclKind::Let | VarDeclKind::Const),
                ..
            } => bindings.iter().map(|(_, target)| target.clone()).collect(),
            Stmt::ExportDecl { declaration, .. } => Self::direct_tdz_binding_names(declaration),
            _ => Vec::new(),
        }
    }

    fn collect_direct_tdz_binding_names(stmts: &[Stmt]) -> HashSet<String> {
        let mut out = HashSet::new();
        for stmt in stmts {
            out.extend(Self::direct_tdz_binding_names(stmt));
        }
        out
    }

    fn collect_var_declared_names(stmts: &[Stmt]) -> HashSet<String> {
        let mut out = HashSet::new();
        for stmt in stmts {
            Self::collect_var_declared_names_from_stmt(stmt, &mut out);
        }
        out
    }

    fn collect_var_declared_names_from_stmt(stmt: &Stmt, out: &mut HashSet<String>) {
        match stmt {
            Stmt::VarDecl {
                name,
                kind: VarDeclKind::Var,
                ..
            } => {
                out.insert(name.clone());
            }
            Stmt::ArrayDestructureAssign {
                targets,
                decl_kind: Some(VarDeclKind::Var),
                ..
            } => {
                for name in targets.iter().flatten() {
                    out.insert(name.clone());
                }
            }
            Stmt::ObjectDestructureAssign {
                bindings,
                decl_kind: Some(VarDeclKind::Var),
                ..
            } => {
                for (_, target) in bindings {
                    out.insert(target.clone());
                }
            }
            Stmt::ExportDecl { declaration, .. } => {
                Self::collect_var_declared_names_from_stmt(declaration, out);
            }
            Stmt::Label { stmt, .. } => {
                Self::collect_var_declared_names_from_stmt(stmt, out);
            }
            Stmt::Block { stmts } => {
                for stmt in stmts {
                    Self::collect_var_declared_names_from_stmt(stmt, out);
                }
            }
            Stmt::For {
                init, post, body, ..
            } => {
                for stmt in init {
                    Self::collect_var_declared_names_from_stmt(stmt, out);
                }
                for stmt in post {
                    Self::collect_var_declared_names_from_stmt(stmt, out);
                }
                for stmt in body {
                    Self::collect_var_declared_names_from_stmt(stmt, out);
                }
            }
            Stmt::ForEach { body, .. }
            | Stmt::ClassListForEach { body, .. }
            | Stmt::ForIn { body, .. }
            | Stmt::ForOf { body, .. }
            | Stmt::ForAwaitOf { body, .. }
            | Stmt::DoWhile { body, .. }
            | Stmt::While { body, .. } => {
                for stmt in body {
                    Self::collect_var_declared_names_from_stmt(stmt, out);
                }
            }
            Stmt::Switch { clauses, .. } => {
                for clause in clauses {
                    for stmt in &clause.stmts {
                        Self::collect_var_declared_names_from_stmt(stmt, out);
                    }
                }
            }
            Stmt::Try {
                try_stmts,
                catch_stmts,
                finally_stmts,
                ..
            } => {
                for stmt in try_stmts {
                    Self::collect_var_declared_names_from_stmt(stmt, out);
                }
                if let Some(catch_stmts) = catch_stmts {
                    for stmt in catch_stmts {
                        Self::collect_var_declared_names_from_stmt(stmt, out);
                    }
                }
                if let Some(finally_stmts) = finally_stmts {
                    for stmt in finally_stmts {
                        Self::collect_var_declared_names_from_stmt(stmt, out);
                    }
                }
            }
            Stmt::If {
                then_stmts,
                else_stmts,
                ..
            } => {
                for stmt in then_stmts {
                    Self::collect_var_declared_names_from_stmt(stmt, out);
                }
                for stmt in else_stmts {
                    Self::collect_var_declared_names_from_stmt(stmt, out);
                }
            }
            _ => {}
        }
    }

    fn hoist_var_declarations(
        &mut self,
        stmts: &[Stmt],
        env: &mut HashMap<String, Value>,
    ) {
        let mut names = Self::collect_var_declared_names(stmts)
            .into_iter()
            .collect::<Vec<_>>();
        names.sort();
        for name in names {
            if env.contains_key(&name) {
                continue;
            }
            env.insert(name.clone(), Value::Undefined);
            self.set_const_binding(env, &name, false);
            self.sync_global_binding_if_needed(env, &name, &Value::Undefined);
        }
    }

    fn direct_let_decl_names(stmt: &Stmt) -> Vec<String> {
        match stmt {
            Stmt::VarDecl {
                name,
                kind: VarDeclKind::Let,
                ..
            } => vec![name.clone()],
            Stmt::ArrayDestructureAssign {
                targets,
                decl_kind: Some(VarDeclKind::Let),
                ..
            } => targets.iter().flatten().cloned().collect(),
            Stmt::ObjectDestructureAssign {
                bindings,
                decl_kind: Some(VarDeclKind::Let),
                ..
            } => bindings.iter().map(|(_, target)| target.clone()).collect(),
            Stmt::ExportDecl { declaration, .. } => Self::direct_let_decl_names(declaration),
            _ => Vec::new(),
        }
    }

    pub(crate) fn ensure_no_direct_let_redeclarations(
        &self,
        stmts: &[Stmt],
        occupied_names: &HashSet<String>,
    ) -> Result<()> {
        for stmt in stmts {
            for name in Self::direct_let_decl_names(stmt) {
                if occupied_names.contains(&name) {
                    return Err(Error::ScriptRuntime(format!(
                        "Identifier '{name}' has already been declared"
                    )));
                }
            }
        }
        Ok(())
    }

    fn validate_const_redeclarations(stmts: &[Stmt]) -> Result<()> {
        let mut lexical = HashSet::new();
        let mut var_like = HashSet::new();
        for stmt in stmts {
            for (name, is_lexical) in Self::direct_decl_binding_kinds(stmt) {
                if is_lexical {
                    if lexical.contains(&name) || var_like.contains(&name) {
                        return Err(Error::ScriptRuntime(format!(
                            "Identifier '{name}' has already been declared"
                        )));
                    }
                    lexical.insert(name);
                } else {
                    if lexical.contains(&name) {
                        return Err(Error::ScriptRuntime(format!(
                            "Identifier '{name}' has already been declared"
                        )));
                    }
                    var_like.insert(name);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn is_const_binding(&self, env: &HashMap<String, Value>, name: &str) -> bool {
        let Some(Value::Object(bindings)) = env.get(INTERNAL_CONST_BINDINGS_KEY) else {
            return false;
        };
        matches!(
            Self::object_get_entry(&bindings.borrow(), name),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn set_const_binding(
        &self,
        env: &mut HashMap<String, Value>,
        name: &str,
        is_const: bool,
    ) {
        if Self::is_internal_env_key(name) {
            return;
        }
        let bindings = match env.get(INTERNAL_CONST_BINDINGS_KEY) {
            Some(Value::Object(bindings)) => bindings.clone(),
            _ => {
                let entries = Rc::new(RefCell::new(ObjectValue::default()));
                env.insert(
                    INTERNAL_CONST_BINDINGS_KEY.to_string(),
                    Value::Object(entries.clone()),
                );
                entries
            }
        };
        Self::object_set_entry(
            &mut bindings.borrow_mut(),
            name.to_string(),
            Value::Bool(is_const),
        );
    }

    fn push_tdz_scope_frame(&mut self, declared: HashSet<String>) {
        self.script_runtime.tdz_scope_stack.push(TdzScopeFrame {
            pending: declared.clone(),
            declared,
        });
    }

    fn pop_tdz_scope_frame(&mut self) {
        self.script_runtime.tdz_scope_stack.pop();
    }

    fn mark_tdz_initialized(
        &mut self,
        pending_tdz_bindings: &mut HashSet<String>,
        name: &str,
    ) {
        if pending_tdz_bindings.remove(name) {
            if let Some(frame) = self.script_runtime.tdz_scope_stack.last_mut() {
                frame.pending.remove(name);
            }
        }
    }

    pub(crate) fn is_binding_in_tdz(&self, _env: &HashMap<String, Value>, name: &str) -> bool {
        for frame in self.script_runtime.tdz_scope_stack.iter().rev() {
            if frame.declared.contains(name) {
                return frame.pending.contains(name);
            }
        }
        false
    }

    pub(crate) fn ensure_binding_initialized(
        &self,
        env: &HashMap<String, Value>,
        name: &str,
    ) -> Result<()> {
        if self.is_binding_in_tdz(env, name) {
            return Err(Error::ScriptRuntime(format!(
                "Cannot access '{name}' before initialization"
            )));
        }
        Ok(())
    }

    fn ensure_binding_is_mutable(&self, env: &HashMap<String, Value>, name: &str) -> Result<()> {
        self.ensure_binding_initialized(env, name)?;
        if self.is_const_binding(env, name) {
            return Err(Error::ScriptRuntime(
                "Assignment to constant variable".into(),
            ));
        }
        Ok(())
    }

    fn collect_direct_block_lexical_bindings(
        &self,
        stmts: &[Stmt],
        env: &HashMap<String, Value>,
    ) -> Vec<(String, Option<Value>, bool)> {
        let mut seen = HashSet::new();
        let mut previous = Vec::new();
        for stmt in stmts {
            let names: Vec<&str> = match stmt {
                Stmt::ImportDecl {
                    default_binding,
                    namespace_binding,
                    named_bindings,
                    ..
                } => {
                    let mut names = Vec::new();
                    if let Some(name) = default_binding {
                        names.push(name.as_str());
                    }
                    if let Some(name) = namespace_binding {
                        names.push(name.as_str());
                    }
                    names.extend(named_bindings.iter().map(|binding| binding.local.as_str()));
                    names
                }
                Stmt::VarDecl { name, kind, .. } => {
                    if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                        vec![name.as_str()]
                    } else {
                        Vec::new()
                    }
                }
                Stmt::ClassDecl { name, .. } => vec![name.as_str()],
                Stmt::FunctionDecl { name, .. } => vec![name.as_str()],
                Stmt::ExportDecl { declaration, .. } => match declaration.as_ref() {
                    Stmt::VarDecl { name, kind, .. } => {
                        if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                            vec![name.as_str()]
                        } else {
                            Vec::new()
                        }
                    }
                    Stmt::ClassDecl { name, .. } => vec![name.as_str()],
                    Stmt::FunctionDecl { name, .. } => vec![name.as_str()],
                    Stmt::ArrayDestructureAssign {
                        targets,
                        decl_kind: Some(kind),
                        ..
                    } => {
                        if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                            targets.iter().flatten().map(String::as_str).collect()
                        } else {
                            Vec::new()
                        }
                    }
                    Stmt::ObjectDestructureAssign {
                        bindings,
                        decl_kind: Some(kind),
                        ..
                    } => {
                        if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                            bindings.iter().map(|(_, target)| target.as_str()).collect()
                        } else {
                            Vec::new()
                        }
                    }
                    _ => Vec::new(),
                },
                Stmt::ArrayDestructureAssign {
                    targets,
                    decl_kind: Some(kind),
                    ..
                } => {
                    if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                        targets.iter().flatten().map(String::as_str).collect()
                    } else {
                        Vec::new()
                    }
                }
                Stmt::ObjectDestructureAssign {
                    bindings,
                    decl_kind: Some(kind),
                    ..
                } => {
                    if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                        bindings.iter().map(|(_, target)| target.as_str()).collect()
                    } else {
                        Vec::new()
                    }
                }
                _ => Vec::new(),
            };
            for name in names {
                if seen.insert(name.to_string()) {
                    previous.push((
                        name.to_string(),
                        env.get(name).cloned(),
                        self.is_const_binding(env, name),
                    ));
                }
            }
        }
        previous
    }

    fn restore_block_lexical_bindings(
        &mut self,
        previous: Vec<(String, Option<Value>, bool)>,
        env: &mut HashMap<String, Value>,
    ) {
        for (name, value, was_const) in previous {
            if let Some(value) = value {
                env.insert(name.clone(), value.clone());
                self.sync_global_binding_if_needed(env, &name, &value);
            } else {
                env.remove(&name);
            }
            self.set_const_binding(env, &name, was_const);
        }
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

    fn is_iteration_stmt(stmt: &Stmt) -> bool {
        matches!(
            stmt,
            Stmt::For { .. }
                | Stmt::ForIn { .. }
                | Stmt::ForOf { .. }
                | Stmt::ForAwaitOf { .. }
                | Stmt::While { .. }
                | Stmt::DoWhile { .. }
        )
    }

    fn await_value_in_for_await(&self, value: Value) -> Result<Value> {
        let Value::Promise(promise) = value else {
            return Ok(value);
        };
        let settled = {
            let promise = promise.borrow();
            match &promise.state {
                PromiseState::Pending => None,
                PromiseState::Fulfilled(value) => Some(Ok(value.clone())),
                PromiseState::Rejected(reason) => Some(Err(reason.clone())),
            }
        };
        match settled {
            Some(Ok(value)) => Ok(value),
            Some(Err(reason)) => Err(Error::ScriptRuntime(format!(
                "await rejected Promise: {}",
                reason.as_string()
            ))),
            None => Ok(Value::Undefined),
        }
    }

    fn for_in_integer_key(key: &str) -> Option<u64> {
        if key.is_empty() || !key.as_bytes().iter().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let value = key.parse::<u64>().ok()?;
        if value.to_string() == key {
            Some(value)
        } else {
            None
        }
    }

    fn ordered_for_in_own_string_keys(entries: &ObjectValue) -> Vec<String> {
        let mut integer_keys: Vec<(u64, String)> = Vec::new();
        let mut string_keys: Vec<String> = Vec::new();
        for (key, _) in entries.iter() {
            if Self::is_internal_object_key(key) {
                continue;
            }
            if let Some(index) = Self::for_in_integer_key(key) {
                integer_keys.push((index, key.clone()));
            } else {
                string_keys.push(key.clone());
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = Vec::with_capacity(integer_keys.len() + string_keys.len());
        out.extend(integer_keys.into_iter().map(|(_, key)| key));
        out.extend(string_keys);
        out
    }

    fn collect_for_in_object_chain_keys(&self, object: &Rc<RefCell<ObjectValue>>) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut out = Vec::new();
        let mut current = Some(object.clone());
        while let Some(target) = current {
            let (keys, next) = {
                let entries = target.borrow();
                let keys = Self::ordered_for_in_own_string_keys(&entries);
                let next = match Self::object_get_entry(&entries, INTERNAL_OBJECT_PROTOTYPE_KEY) {
                    Some(Value::Object(next)) => Some(next),
                    _ => None,
                };
                (keys, next)
            };
            for key in keys {
                if visited.insert(key.clone()) {
                    out.push(key);
                }
            }
            current = next;
        }
        out
    }

    fn collect_for_in_array_keys(&self, array: &Rc<RefCell<ArrayValue>>) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut out = Vec::new();
        let (length, own_props, prototype) = {
            let array = array.borrow();
            let own_props = Self::ordered_for_in_own_string_keys(&array.properties);
            let prototype =
                match Self::object_get_entry(&array.properties, INTERNAL_OBJECT_PROTOTYPE_KEY) {
                    Some(Value::Object(next)) => Some(next),
                    _ => None,
                };
            (array.elements.len(), own_props, prototype)
        };
        for index in 0..length {
            let key = index.to_string();
            if visited.insert(key.clone()) {
                out.push(key);
            }
        }
        for key in own_props {
            if visited.insert(key.clone()) {
                out.push(key);
            }
        }
        let mut current = prototype;
        while let Some(target) = current {
            let (keys, next) = {
                let entries = target.borrow();
                let keys = Self::ordered_for_in_own_string_keys(&entries);
                let next = match Self::object_get_entry(&entries, INTERNAL_OBJECT_PROTOTYPE_KEY) {
                    Some(Value::Object(next)) => Some(next),
                    _ => None,
                };
                (keys, next)
            };
            for key in keys {
                if visited.insert(key.clone()) {
                    out.push(key);
                }
            }
            current = next;
        }
        out
    }

    fn for_of_symbol_iterator_factory_result(
        &mut self,
        iterable: &Rc<RefCell<ObjectValue>>,
        event: &EventState,
    ) -> Result<Option<Rc<RefCell<ObjectValue>>>> {
        let iterator_symbol = self.eval_symbol_static_property(SymbolStaticProperty::Iterator);
        let iterator_key = self.property_key_to_storage_key(&iterator_symbol);
        let iterable_value = Value::Object(iterable.clone());
        let iterator_factory = self.object_property_from_value(&iterable_value, &iterator_key)?;
        if matches!(iterator_factory, Value::Undefined | Value::Null) {
            return Ok(None);
        }
        if !self.is_callable_value(&iterator_factory) {
            return Err(Error::ScriptRuntime(
                "for...of iterator factory is not callable".into(),
            ));
        }
        let iterator_value = self.execute_callable_value_with_this_and_env(
            &iterator_factory,
            &[],
            event,
            None,
            Some(iterable_value),
        )?;
        let Value::Object(iterator) = iterator_value else {
            return Err(Error::ScriptRuntime(
                "for...of iterator factory must return an object".into(),
            ));
        };
        Ok(Some(iterator))
    }

    fn for_of_protocol_iterator_next(
        &mut self,
        iterator: &Rc<RefCell<ObjectValue>>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let iterator_value = Value::Object(iterator.clone());
        let next_method = self.object_property_from_value(&iterator_value, "next")?;
        if !self.is_callable_value(&next_method) {
            return Err(Error::ScriptRuntime(
                "for...of iterator next is not callable".into(),
            ));
        }
        let result = self.execute_callable_value_with_this_and_env(
            &next_method,
            &[],
            event,
            None,
            Some(iterator_value),
        )?;
        let Value::Object(result_obj) = result else {
            return Err(Error::ScriptRuntime(
                "for...of iterator.next must return an object".into(),
            ));
        };
        let result_value = Value::Object(result_obj.clone());
        let done = self
            .object_property_from_value(&result_value, "done")?
            .truthy();
        if done {
            return Ok(None);
        }
        let value = self.object_property_from_value(&result_value, "value")?;
        Ok(Some(value))
    }

    fn for_of_protocol_iterator_close(
        &mut self,
        iterator: &Rc<RefCell<ObjectValue>>,
        event: &EventState,
    ) -> Result<()> {
        let iterator_value = Value::Object(iterator.clone());
        let return_method = self.object_property_from_value(&iterator_value, "return")?;
        if matches!(return_method, Value::Undefined | Value::Null) {
            return Ok(());
        }
        if !self.is_callable_value(&return_method) {
            return Err(Error::ScriptRuntime(
                "for...of iterator.return is not callable".into(),
            ));
        }
        let _ = self.execute_callable_value_with_this_and_env(
            &return_method,
            &[],
            event,
            None,
            Some(iterator_value),
        )?;
        Ok(())
    }

    fn for_of_internal_iterator_close_if_needed(
        &mut self,
        iterator: &Rc<RefCell<ObjectValue>>,
        event: &EventState,
    ) -> Result<()> {
        let _ = self.eval_iterator_member_call(iterator, "return", &[], event)?;
        Ok(())
    }

    fn take_pending_loop_labels(&mut self) -> Vec<String> {
        self.script_runtime
            .pending_loop_labels
            .pop()
            .unwrap_or_default()
    }

    fn push_loop_label_scope(&mut self, labels: Vec<String>) {
        self.script_runtime
            .loop_label_stack
            .push(labels.into_iter().collect());
    }

    fn pop_loop_label_scope(&mut self) {
        self.script_runtime.loop_label_stack.pop();
    }

    fn current_loop_has_label(&self, label: &str) -> bool {
        self.script_runtime
            .loop_label_stack
            .last()
            .is_some_and(|labels| labels.contains(label))
    }

    fn loop_should_consume_break(&self, label: &Option<String>) -> bool {
        match label {
            None => true,
            Some(label) => self.current_loop_has_label(label),
        }
    }

    fn loop_should_consume_continue(&self, label: &Option<String>) -> bool {
        match label {
            None => true,
            Some(label) => self.current_loop_has_label(label),
        }
    }

    pub(crate) fn break_flow_error(label: &Option<String>) -> Error {
        if let Some(label) = label {
            Error::ScriptRuntime(format!("label not found: {label}"))
        } else {
            Error::ScriptRuntime("break statement outside of loop".into())
        }
    }

    pub(crate) fn continue_flow_error(label: &Option<String>) -> Error {
        if let Some(label) = label {
            Error::ScriptRuntime(format!("label not found: {label}"))
        } else {
            Error::ScriptRuntime("continue statement outside of loop".into())
        }
    }

    pub(crate) fn with_isolated_loop_control_scope<T>(
        &mut self,
        run: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let previous_pending = std::mem::take(&mut self.script_runtime.pending_loop_labels);
        let previous_labels = std::mem::take(&mut self.script_runtime.loop_label_stack);
        let result = run(self);
        self.script_runtime.pending_loop_labels = previous_pending;
        self.script_runtime.loop_label_stack = previous_labels;
        result
    }

    fn current_module_referrer(&self) -> String {
        self.script_runtime
            .module_referrer_stack
            .last()
            .cloned()
            .unwrap_or_else(|| self.document_url.clone())
    }

    fn bind_hoisted_import_decls(
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

    fn execute_stmts_impl(
        &mut self,
        stmts: &[Stmt],
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        let pending = Self::collect_function_decls(stmts);
        let pending_scope_start = self.push_pending_function_decl_scope(pending);
        self.script_runtime
            .listener_capture_env_stack
            .push(ListenerCaptureFrame::default());

        let result = (|| -> Result<ExecFlow> {
            Self::validate_const_redeclarations(stmts)?;
            self.bind_hoisted_import_decls(stmts, env)?;
            self.hoist_var_declarations(stmts, env);
            let mut pending_tdz_bindings = Self::collect_direct_tdz_binding_names(stmts);
            self.push_tdz_scope_frame(pending_tdz_bindings.clone());
            let mut initialized_var_bindings = HashSet::new();
            let flow_result = (|| -> Result<ExecFlow> {
                for stmt in stmts {
                    self.apply_pending_listener_capture_env_updates(env);
                    self.sync_top_level_env_from_runtime(env);
                    self.sync_listener_capture_env_if_shared(env);
                    match stmt {
                    Stmt::ImportDecl { .. } => {}
                    Stmt::VarDecl { name, expr, kind } => {
                        if matches!(kind, VarDeclKind::Var) && matches!(expr, Expr::Undefined) {
                            if !env.contains_key(name) {
                                env.insert(name.clone(), Value::Undefined);
                                self.set_const_binding(env, name, false);
                                self.sync_global_binding_if_needed(env, name, &Value::Undefined);
                            }
                            continue;
                        }
                        let value = self.eval_expr(expr, env, event_param, event)?;
                        env.insert(name.clone(), value.clone());
                        self.set_const_binding(env, name, matches!(kind, VarDeclKind::Const));
                        if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                            self.mark_tdz_initialized(&mut pending_tdz_bindings, name);
                        }
                        self.bind_timer_id_to_task_env(name, expr, &value);
                        if matches!(kind, VarDeclKind::Var) && !matches!(expr, Expr::Undefined) {
                            initialized_var_bindings.insert(name.clone());
                        }
                    }
                    Stmt::FunctionDecl {
                        name,
                        handler,
                        is_async,
                        is_generator,
                    } => {
                        // Keep prior initialized `var` binding values in place.
                        if initialized_var_bindings.contains(name) {
                            continue;
                        }
                        let function = self.make_function_value(
                            handler.clone(),
                            env,
                            false,
                            *is_async,
                            *is_generator,
                            false,
                            false,
                        );
                        env.insert(name.clone(), function);
                        self.set_const_binding(env, name, false);
                    }
                    Stmt::ClassDecl {
                        name,
                        super_class,
                        constructor,
                        methods,
                    } => {
                        let (super_constructor, super_prototype) = if let Some(super_class_expr) =
                            super_class
                        {
                            let evaluated_super =
                                self.eval_expr(super_class_expr, env, event_param, event)?;
                            if !self.is_callable_value(&evaluated_super) {
                                return Err(Error::ScriptRuntime(
                                    "class extends value is not a constructor".into(),
                                ));
                            }
                            let super_prototype =
                                self.object_property_from_value(&evaluated_super, "prototype")?;
                            let Value::Object(super_prototype) = super_prototype else {
                                return Err(Error::ScriptRuntime(
                                    "class extends value does not have a valid prototype".into(),
                                ));
                            };
                            (Some(evaluated_super), Some(super_prototype))
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
                            super_prototype.clone().map(Value::Object),
                        );
                        let Value::Function(class_function) = &class_constructor else {
                            return Err(Error::ScriptRuntime(
                                "class constructor is not callable".into(),
                            ));
                        };

                        {
                            let mut prototype = class_function.prototype_object.borrow_mut();
                            if let Some(super_prototype) = super_prototype.clone() {
                                Self::object_set_entry(
                                    &mut *prototype,
                                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                                    Value::Object(super_prototype),
                                );
                            }
                            Self::object_set_entry(
                                &mut *prototype,
                                "constructor".to_string(),
                                class_constructor.clone(),
                            );
                            for method in methods {
                                match method.kind {
                                    ClassMethodKind::Method => {
                                        let method_value = self.make_function_value_with_super(
                                            method.handler.clone(),
                                            env,
                                            false,
                                            method.is_async,
                                            method.is_generator,
                                            false,
                                            true,
                                            super_constructor.clone(),
                                            super_prototype.clone().map(Value::Object),
                                        );
                                        Self::object_set_entry(
                                            &mut *prototype,
                                            method.name.clone(),
                                            method_value,
                                        );
                                    }
                                    ClassMethodKind::Getter => {
                                        let getter_value = self.make_function_value_with_super(
                                            method.handler.clone(),
                                            env,
                                            false,
                                            false,
                                            false,
                                            false,
                                            true,
                                            super_constructor.clone(),
                                            super_prototype.clone().map(Value::Object),
                                        );
                                        let getter_key =
                                            Self::object_getter_storage_key(&method.name);
                                        Self::object_set_entry(
                                            &mut *prototype,
                                            getter_key,
                                            getter_value,
                                        );
                                    }
                                    ClassMethodKind::Setter => {
                                        let setter_value = self.make_function_value_with_super(
                                            method.handler.clone(),
                                            env,
                                            false,
                                            false,
                                            false,
                                            false,
                                            true,
                                            super_constructor.clone(),
                                            super_prototype.clone().map(Value::Object),
                                        );
                                        let setter_key =
                                            Self::object_setter_storage_key(&method.name);
                                        Self::object_set_entry(
                                            &mut *prototype,
                                            setter_key,
                                            setter_value,
                                        );
                                    }
                                }
                            }
                        }

                        env.insert(name.clone(), class_constructor);
                        self.set_const_binding(env, name, false);
                        self.mark_tdz_initialized(&mut pending_tdz_bindings, name);
                    }
                    Stmt::ExportDecl {
                        declaration,
                        bindings,
                    } => {
                        match self.execute_stmts(
                            std::slice::from_ref(declaration.as_ref()),
                            event_param,
                            event,
                            env,
                        )? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                        for local in Self::direct_tdz_binding_names(declaration) {
                            self.mark_tdz_initialized(&mut pending_tdz_bindings, &local);
                        }
                        self.register_module_named_exports(bindings);
                    }
                    Stmt::ExportNamed { bindings } => {
                        self.register_module_named_exports(bindings);
                    }
                    Stmt::ExportDefaultExpr { expr } => {
                        let value = self.eval_expr(expr, env, event_param, event)?;
                        self.register_module_default_export_value(value);
                    }
                    Stmt::Block { stmts } => {
                        let previous = self.collect_direct_block_lexical_bindings(stmts, env);
                        let flow = self.execute_stmts(stmts, event_param, event, env);
                        self.restore_block_lexical_bindings(previous, env);
                        match flow? {
                            ExecFlow::Continue => {}
                            ExecFlow::Break(label) => return Ok(ExecFlow::Break(label)),
                            ExecFlow::ContinueLoop(label) => {
                                return Ok(ExecFlow::ContinueLoop(label));
                            }
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }
                    Stmt::Label { name, stmt } => {
                        let mut labels = vec![name.clone()];
                        let mut target = stmt.as_ref();
                        while let Stmt::Label { name, stmt } = target {
                            labels.push(name.clone());
                            target = stmt.as_ref();
                        }

                        if Self::is_iteration_stmt(target) {
                            self.script_runtime.pending_loop_labels.push(labels);
                            match self.execute_stmts(
                                std::slice::from_ref(target),
                                event_param,
                                event,
                                env,
                            )? {
                                ExecFlow::Continue => {}
                                flow => return Ok(flow),
                            }
                        } else {
                            match self.execute_stmts(
                                std::slice::from_ref(target),
                                event_param,
                                event,
                                env,
                            )? {
                                ExecFlow::Continue => {}
                                ExecFlow::Break(Some(label))
                                    if labels.iter().any(|candidate| candidate == &label) => {}
                                ExecFlow::ContinueLoop(Some(label))
                                    if labels.iter().any(|candidate| candidate == &label) =>
                                {
                                    return Err(Error::ScriptRuntime(format!(
                                        "continue statement: '{label}' does not denote an iteration statement"
                                    )));
                                }
                                flow => return Ok(flow),
                            }
                        }
                    }
                    Stmt::VarAssign { name, op, expr } => {
                        self.ensure_binding_initialized(env, name)?;
                        let previous = env.get(name).cloned().ok_or_else(|| {
                            Error::ScriptRuntime(format!("unknown variable: {name}"))
                        })?;
                        self.ensure_binding_is_mutable(env, name)?;

                        let next = match op {
                            VarAssignOp::Assign => self.eval_expr(expr, env, event_param, event)?,
                            VarAssignOp::Add => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.add_values(&previous, &value)?
                            }
                            VarAssignOp::Sub => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.eval_binary(&BinaryOp::Sub, &previous, &value)?
                            }
                            VarAssignOp::Mul => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.eval_binary(&BinaryOp::Mul, &previous, &value)?
                            }
                            VarAssignOp::Pow => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.eval_binary(&BinaryOp::Pow, &previous, &value)?
                            }
                            VarAssignOp::BitOr => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.eval_binary(&BinaryOp::BitOr, &previous, &value)?
                            }
                            VarAssignOp::BitXor => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.eval_binary(&BinaryOp::BitXor, &previous, &value)?
                            }
                            VarAssignOp::BitAnd => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.eval_binary(&BinaryOp::BitAnd, &previous, &value)?
                            }
                            VarAssignOp::ShiftLeft => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.eval_binary(&BinaryOp::ShiftLeft, &previous, &value)?
                            }
                            VarAssignOp::ShiftRight => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.eval_binary(&BinaryOp::ShiftRight, &previous, &value)?
                            }
                            VarAssignOp::UnsignedShiftRight => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.eval_binary(&BinaryOp::UnsignedShiftRight, &previous, &value)?
                            }
                            VarAssignOp::Div => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.eval_binary(&BinaryOp::Div, &previous, &value)?
                            }
                            VarAssignOp::Mod => {
                                let value = self.eval_expr(expr, env, event_param, event)?;
                                self.eval_binary(&BinaryOp::Mod, &previous, &value)?
                            }
                            VarAssignOp::LogicalAnd => {
                                if previous.truthy() {
                                    self.eval_expr(expr, env, event_param, event)?
                                } else {
                                    previous.clone()
                                }
                            }
                            VarAssignOp::LogicalOr => {
                                if previous.truthy() {
                                    previous.clone()
                                } else {
                                    self.eval_expr(expr, env, event_param, event)?
                                }
                            }
                            VarAssignOp::Nullish => {
                                if matches!(&previous, Value::Null | Value::Undefined) {
                                    self.eval_expr(expr, env, event_param, event)?
                                } else {
                                    previous.clone()
                                }
                            }
                        };
                        env.insert(name.clone(), next.clone());
                        self.sync_arguments_after_param_write(env, name, &next);
                        self.sync_global_binding_if_needed(env, name, &next);
                        self.bind_timer_id_to_task_env(name, expr, &next);
                    }
                    Stmt::VarUpdate { name, delta } => {
                        self.ensure_binding_initialized(env, name)?;
                        let previous = env.get(name).cloned().ok_or_else(|| {
                            Error::ScriptRuntime(format!("unknown variable: {name}"))
                        })?;
                        self.ensure_binding_is_mutable(env, name)?;
                        let next = match previous {
                            Value::Number(value) => {
                                if let Some(sum) = value.checked_add(i64::from(*delta)) {
                                    Value::Number(sum)
                                } else {
                                    Value::Float((value as f64) + f64::from(*delta))
                                }
                            }
                            Value::Float(value) => Value::Float(value + f64::from(*delta)),
                            Value::BigInt(value) => Value::BigInt(value + JsBigInt::from(*delta)),
                            _ => {
                                return Err(Error::ScriptRuntime(format!(
                                    "cannot apply update operator to '{}'",
                                    name
                                )));
                            }
                        };
                        env.insert(name.clone(), next.clone());
                        self.sync_arguments_after_param_write(env, name, &next);
                        self.sync_global_binding_if_needed(env, name, &next);
                    }
                    Stmt::ArrayDestructureAssign {
                        targets,
                        expr,
                        decl_kind,
                    } => {
                        let value = self.eval_expr(expr, env, event_param, event)?;
                        let values = self.array_like_values_from_value(&value)?;
                        let is_declaration = decl_kind.is_some();
                        let is_const_decl = matches!(decl_kind, Some(VarDeclKind::Const));
                        for (index, target_name) in targets.iter().enumerate() {
                            let Some(target_name) = target_name else {
                                continue;
                            };
                            if !is_declaration {
                                self.ensure_binding_initialized(env, target_name)?;
                            }
                            if !is_declaration && env.contains_key(target_name) {
                                self.ensure_binding_is_mutable(env, target_name)?;
                            }
                            let next = values.get(index).cloned().unwrap_or(Value::Undefined);
                            env.insert(target_name.clone(), next.clone());
                            self.sync_arguments_after_param_write(env, target_name, &next);
                            if is_declaration {
                                self.set_const_binding(env, target_name, is_const_decl);
                                if decl_kind
                                    .is_some_and(|kind| matches!(kind, VarDeclKind::Let | VarDeclKind::Const))
                                {
                                    self.mark_tdz_initialized(
                                        &mut pending_tdz_bindings,
                                        target_name,
                                    );
                                }
                            }
                            self.sync_global_binding_if_needed(env, target_name, &next);
                        }
                    }
                    Stmt::ObjectDestructureAssign {
                        bindings,
                        expr,
                        decl_kind,
                    } => {
                        let value = self.eval_expr(expr, env, event_param, event)?;
                        let Value::Object(entries) = value else {
                            return Err(Error::ScriptRuntime(
                                "object destructuring source must be an object".into(),
                            ));
                        };
                        let entries = entries.borrow();
                        let is_declaration = decl_kind.is_some();
                        let is_const_decl = matches!(decl_kind, Some(VarDeclKind::Const));
                        for (source_key, target_name) in bindings {
                            if !is_declaration {
                                self.ensure_binding_initialized(env, target_name)?;
                            }
                            if !is_declaration && env.contains_key(target_name) {
                                self.ensure_binding_is_mutable(env, target_name)?;
                            }
                            let next = Self::object_get_entry(&entries, source_key)
                                .unwrap_or(Value::Undefined);
                            env.insert(target_name.clone(), next.clone());
                            self.sync_arguments_after_param_write(env, target_name, &next);
                            if is_declaration {
                                self.set_const_binding(env, target_name, is_const_decl);
                                if decl_kind
                                    .is_some_and(|kind| matches!(kind, VarDeclKind::Let | VarDeclKind::Const))
                                {
                                    self.mark_tdz_initialized(
                                        &mut pending_tdz_bindings,
                                        target_name,
                                    );
                                }
                            }
                            self.sync_global_binding_if_needed(env, target_name, &next);
                        }
                    }
                    Stmt::ObjectAssign { target, path, expr } => {
                        self.execute_object_assignment_stmt(
                            target,
                            path,
                            expr,
                            env,
                            event_param,
                            event,
                        )?;
                    }
                    Stmt::FormDataAppend {
                        target_var,
                        name,
                        value,
                    } => {
                        let name = self.eval_expr(name, env, event_param, event)?;
                        let value = self.eval_expr(value, env, event_param, event)?;
                        let name = name.as_string();
                        let value = value.as_string();
                        let target = env.get_mut(target_var).ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unknown FormData variable: {}",
                                target_var
                            ))
                        })?;
                        match target {
                            Value::FormData(entries) => {
                                entries.push((name, value));
                            }
                            Value::Object(entries) => {
                                if !Self::is_url_search_params_object(&entries.borrow()) {
                                    return Err(Error::ScriptRuntime(format!(
                                        "variable '{}' is not a FormData instance",
                                        target_var
                                    )));
                                }
                                {
                                    let mut object_ref = entries.borrow_mut();
                                    let mut pairs =
                                        Self::url_search_params_pairs_from_object_entries(
                                            &object_ref,
                                        );
                                    pairs.push((name, value));
                                    Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                                }
                                self.sync_url_search_params_owner(entries);
                            }
                            _ => {
                                return Err(Error::ScriptRuntime(format!(
                                    "variable '{}' is not a FormData instance",
                                    target_var
                                )));
                            }
                        }
                    }
                    Stmt::DomAssign { target, prop, expr } => {
                        let value = self.eval_expr(expr, env, event_param, event)?;
                        if let DomQuery::Var(name) = target {
                            if let Some(Value::Object(entries)) = env.get(name) {
                                if let Some(key) = Self::object_key_from_dom_prop(prop) {
                                    Self::object_set_entry(
                                        &mut entries.borrow_mut(),
                                        key.to_string(),
                                        value,
                                    );
                                    continue;
                                }
                            }
                        }
                        let node = self.resolve_dom_query_required_runtime(target, env)?;
                        match prop {
                            DomProp::TextContent => {
                                self.dom.set_text_content(node, &value.as_string())?
                            }
                            DomProp::InnerText => {
                                self.dom.set_text_content(node, &value.as_string())?
                            }
                            DomProp::InnerHtml => {
                                self.dom.set_inner_html(node, &value.as_string())?
                            }
                            DomProp::OuterHtml => {
                                self.dom.set_outer_html(node, &value.as_string())?
                            }
                            DomProp::Value => {
                                if self
                                    .dom
                                    .tag_name(node)
                                    .is_some_and(|tag| tag.eq_ignore_ascii_case("li"))
                                {
                                    let next = Self::value_to_i64(&value);
                                    self.dom.set_attr(node, "value", &next.to_string())?;
                                } else {
                                    self.dom.set_value(node, &value.as_string())?;
                                }
                            }
                            DomProp::ValueAsNumber => self.set_input_value_as_number(
                                node,
                                Self::coerce_number_for_number_constructor(&value),
                            )?,
                            DomProp::ValueAsDate => {
                                let timestamp_ms = match value {
                                    Value::Date(timestamp) => Some(*timestamp.borrow()),
                                    Value::Null | Value::Undefined => None,
                                    _ => None,
                                };
                                self.set_input_value_as_date_ms(node, timestamp_ms)?;
                            }
                            DomProp::SelectionStart => {
                                let next_start = Self::value_to_i64(&value).max(0) as usize;
                                let end = self.dom.selection_end(node).unwrap_or_default();
                                self.dom
                                    .set_selection_range(node, next_start, end, "none")?;
                            }
                            DomProp::SelectionEnd => {
                                let start = self.dom.selection_start(node).unwrap_or_default();
                                let next_end = Self::value_to_i64(&value).max(0) as usize;
                                self.dom
                                    .set_selection_range(node, start, next_end, "none")?;
                            }
                            DomProp::SelectionDirection => {
                                let start = self.dom.selection_start(node).unwrap_or_default();
                                let end = self.dom.selection_end(node).unwrap_or_default();
                                let direction = value.as_string();
                                let direction =
                                    Self::normalize_selection_direction(direction.as_str());
                                self.dom.set_selection_range(node, start, end, direction)?;
                            }
                            DomProp::Checked => self.dom.set_checked(node, value.truthy())?,
                            DomProp::Indeterminate => {
                                self.dom.set_indeterminate(node, value.truthy())?
                            }
                            DomProp::Open => {
                                if self
                                    .dom
                                    .tag_name(node)
                                    .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
                                {
                                    let _ = self.set_details_open_state_with_env(
                                        node,
                                        value.truthy(),
                                        env,
                                    )?;
                                } else {
                                    if value.truthy() {
                                        self.dom.set_attr(node, "open", "true")?;
                                    } else {
                                        self.dom.remove_attr(node, "open")?;
                                    }
                                }
                            }
                            DomProp::ReturnValue => {
                                self.set_dialog_return_value(node, value.as_string())?;
                            }
                            DomProp::ClosedBy => {
                                self.dom.set_attr(node, "closedby", &value.as_string())?
                            }
                            DomProp::Readonly => {
                                if value.truthy() {
                                    self.dom.set_attr(node, "readonly", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "readonly")?;
                                }
                            }
                            DomProp::Required => {
                                if value.truthy() {
                                    self.dom.set_attr(node, "required", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "required")?;
                                }
                            }
                            DomProp::Disabled => {
                                if value.truthy() {
                                    self.dom.set_attr(node, "disabled", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "disabled")?;
                                }
                            }
                            DomProp::Hidden => {
                                if node == self.dom.root {
                                    let call = self.describe_dom_prop(prop);
                                    return Err(Error::ScriptRuntime(format!(
                                        "{call} is read-only"
                                    )));
                                }
                                if value.truthy() {
                                    self.dom.set_attr(node, "hidden", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "hidden")?;
                                }
                            }
                            DomProp::ClassName => {
                                self.dom.set_attr(node, "class", &value.as_string())?
                            }
                            DomProp::Id => self.dom.set_attr(node, "id", &value.as_string())?,
                            DomProp::Slot => self.dom.set_attr(node, "slot", &value.as_string())?,
                            DomProp::Role => self.dom.set_attr(node, "role", &value.as_string())?,
                            DomProp::ElementTiming => {
                                self.dom
                                    .set_attr(node, "elementtiming", &value.as_string())?
                            }
                            DomProp::HtmlFor => {
                                self.dom.set_attr(node, "for", &value.as_string())?
                            }
                            DomProp::Name => self.dom.set_attr(node, "name", &value.as_string())?,
                            DomProp::Lang => self.dom.set_attr(node, "lang", &value.as_string())?,
                            DomProp::Dir => self.dom.set_attr(node, "dir", &value.as_string())?,
                            DomProp::AccessKey => {
                                self.dom.set_attr(node, "accesskey", &value.as_string())?
                            }
                            DomProp::AutoCapitalize => {
                                self.dom
                                    .set_attr(node, "autocapitalize", &value.as_string())?
                            }
                            DomProp::AutoCorrect => {
                                self.dom.set_attr(node, "autocorrect", &value.as_string())?
                            }
                            DomProp::ContentEditable => {
                                self.dom
                                    .set_attr(node, "contenteditable", &value.as_string())?
                            }
                            DomProp::Draggable => self.dom.set_attr(
                                node,
                                "draggable",
                                if value.truthy() { "true" } else { "false" },
                            )?,
                            DomProp::EnterKeyHint => {
                                self.dom
                                    .set_attr(node, "enterkeyhint", &value.as_string())?
                            }
                            DomProp::Inert => {
                                if value.truthy() {
                                    self.dom.set_attr(node, "inert", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "inert")?;
                                }
                            }
                            DomProp::InputMode => {
                                self.dom.set_attr(node, "inputmode", &value.as_string())?
                            }
                            DomProp::Nonce => {
                                self.dom.set_attr(node, "nonce", &value.as_string())?
                            }
                            DomProp::Popover => {
                                self.dom.set_attr(node, "popover", &value.as_string())?
                            }
                            DomProp::Spellcheck => self.dom.set_attr(
                                node,
                                "spellcheck",
                                if value.truthy() { "true" } else { "false" },
                            )?,
                            DomProp::TabIndex => self.dom.set_attr(
                                node,
                                "tabindex",
                                &Self::value_to_i64(&value).to_string(),
                            )?,
                            DomProp::Translate => self.dom.set_attr(
                                node,
                                "translate",
                                if value.truthy() { "yes" } else { "no" },
                            )?,
                            DomProp::Cite => self.dom.set_attr(node, "cite", &value.as_string())?,
                            DomProp::DateTime => {
                                self.dom.set_attr(node, "datetime", &value.as_string())?
                            }
                            DomProp::BrClear => {
                                self.dom.set_attr(node, "clear", &value.as_string())?
                            }
                            DomProp::CaptionAlign => {
                                self.dom.set_attr(node, "align", &value.as_string())?
                            }
                            DomProp::ColSpan => {
                                if self.dom.tag_name(node).is_some_and(|tag| {
                                    tag.eq_ignore_ascii_case("col")
                                        || tag.eq_ignore_ascii_case("colgroup")
                                }) {
                                    self.set_col_span_value(node, &value)?
                                } else {
                                    self.dom_runtime
                                        .node_expando_props
                                        .insert((node, "span".to_string()), value);
                                }
                            }
                            DomProp::CanvasWidth => {
                                self.set_canvas_dimension_value(node, "width", &value)?
                            }
                            DomProp::CanvasHeight => {
                                self.set_canvas_dimension_value(node, "height", &value)?
                            }
                            DomProp::NodeEventHandler(event_name) => {
                                let _ = self.set_node_event_handler_property(
                                    node,
                                    event_name,
                                    value.clone(),
                                )?;
                            }
                            DomProp::BodyDeprecatedAttr(attr_name) => {
                                self.dom.set_attr(node, attr_name, &value.as_string())?
                            }
                            DomProp::Title => self.dom.set_document_title(&value.as_string())?,
                            DomProp::AudioSrc => {
                                self.dom.set_attr(node, "src", &value.as_string())?
                            }
                            DomProp::AudioAutoplay => {
                                if value.truthy() {
                                    self.dom.set_attr(node, "autoplay", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "autoplay")?;
                                }
                            }
                            DomProp::AudioControls => {
                                if value.truthy() {
                                    self.dom.set_attr(node, "controls", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "controls")?;
                                }
                            }
                            DomProp::AudioControlsList => {
                                self.dom
                                    .set_attr(node, "controlslist", &value.as_string())?
                            }
                            DomProp::AudioCrossOrigin => {
                                self.dom.set_attr(node, "crossorigin", &value.as_string())?
                            }
                            DomProp::AudioDisableRemotePlayback => {
                                if value.truthy() {
                                    self.dom.set_attr(node, "disableremoteplayback", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "disableremoteplayback")?;
                                }
                            }
                            DomProp::VideoDisablePictureInPicture => {
                                if value.truthy() {
                                    self.dom.set_attr(node, "disablepictureinpicture", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "disablepictureinpicture")?;
                                }
                            }
                            DomProp::AudioLoop => {
                                if value.truthy() {
                                    self.dom.set_attr(node, "loop", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "loop")?;
                                }
                            }
                            DomProp::AudioMuted => {
                                if value.truthy() {
                                    self.dom.set_attr(node, "muted", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "muted")?;
                                }
                            }
                            DomProp::AudioPreload => {
                                self.dom.set_attr(node, "preload", &value.as_string())?
                            }
                            DomProp::VideoPlaysInline => {
                                if value.truthy() {
                                    self.dom.set_attr(node, "playsinline", "true")?;
                                } else {
                                    self.dom.remove_attr(node, "playsinline")?;
                                }
                            }
                            DomProp::VideoPoster => {
                                self.dom.set_attr(node, "poster", &value.as_string())?
                            }
                            DomProp::Location | DomProp::LocationHref => self.navigate_location(
                                &value.as_string(),
                                LocationNavigationKind::HrefSet,
                            )?,
                            DomProp::LocationProtocol => {
                                self.set_location_property("protocol", value.clone())?
                            }
                            DomProp::LocationHost => {
                                self.set_location_property("host", value.clone())?
                            }
                            DomProp::LocationHostname => {
                                self.set_location_property("hostname", value.clone())?
                            }
                            DomProp::LocationPort => {
                                self.set_location_property("port", value.clone())?
                            }
                            DomProp::LocationPathname => {
                                self.set_location_property("pathname", value.clone())?
                            }
                            DomProp::LocationSearch => {
                                self.set_location_property("search", value.clone())?
                            }
                            DomProp::LocationHash => {
                                self.set_location_property("hash", value.clone())?
                            }
                            DomProp::HistoryScrollRestoration => {
                                self.set_history_property("scrollRestoration", value.clone())?
                            }
                            DomProp::AnchorAttributionSrc => {
                                self.dom
                                    .set_attr(node, "attributionsrc", &value.as_string())?
                            }
                            DomProp::AnchorDownload => {
                                self.dom.set_attr(node, "download", &value.as_string())?
                            }
                            DomProp::AnchorHash => {
                                self.set_anchor_url_property(node, "hash", value.clone())?
                            }
                            DomProp::AnchorHost => {
                                self.set_anchor_url_property(node, "host", value.clone())?
                            }
                            DomProp::AnchorHostname => {
                                self.set_anchor_url_property(node, "hostname", value.clone())?
                            }
                            DomProp::AnchorHref => {
                                self.set_anchor_url_property(node, "href", value.clone())?
                            }
                            DomProp::AnchorHreflang => {
                                self.dom.set_attr(node, "hreflang", &value.as_string())?
                            }
                            DomProp::AnchorInterestForElement => {
                                self.dom.set_attr(node, "interestfor", &value.as_string())?
                            }
                            DomProp::AnchorPassword => {
                                self.set_anchor_url_property(node, "password", value.clone())?
                            }
                            DomProp::AnchorPathname => {
                                self.set_anchor_url_property(node, "pathname", value.clone())?
                            }
                            DomProp::AnchorPing => {
                                self.dom.set_attr(node, "ping", &value.as_string())?
                            }
                            DomProp::AnchorPort => {
                                self.set_anchor_url_property(node, "port", value.clone())?
                            }
                            DomProp::AnchorProtocol => {
                                self.set_anchor_url_property(node, "protocol", value.clone())?
                            }
                            DomProp::AnchorReferrerPolicy => {
                                self.dom
                                    .set_attr(node, "referrerpolicy", &value.as_string())?
                            }
                            DomProp::AnchorRel => {
                                self.dom.set_attr(node, "rel", &value.as_string())?
                            }
                            DomProp::AnchorSearch => {
                                self.set_anchor_url_property(node, "search", value.clone())?
                            }
                            DomProp::AnchorTarget => {
                                self.dom.set_attr(node, "target", &value.as_string())?
                            }
                            DomProp::AnchorText => {
                                self.dom.set_text_content(node, &value.as_string())?
                            }
                            DomProp::AnchorType => {
                                self.dom.set_attr(node, "type", &value.as_string())?
                            }
                            DomProp::AnchorUsername => {
                                self.set_anchor_url_property(node, "username", value.clone())?
                            }
                            DomProp::AnchorCharset => {
                                self.dom.set_attr(node, "charset", &value.as_string())?
                            }
                            DomProp::AnchorCoords => {
                                self.dom.set_attr(node, "coords", &value.as_string())?
                            }
                            DomProp::AnchorRev => {
                                self.dom.set_attr(node, "rev", &value.as_string())?
                            }
                            DomProp::AnchorShape => {
                                self.dom.set_attr(node, "shape", &value.as_string())?
                            }
                            DomProp::AriaString(prop_name) => {
                                let attr_name = Self::aria_property_to_attr_name(prop_name);
                                self.dom.set_attr(node, &attr_name, &value.as_string())?
                            }
                            DomProp::Attributes
                            | DomProp::AssignedSlot
                            | DomProp::Files
                            | DomProp::FilesLength
                            | DomProp::ValidationMessage
                            | DomProp::Validity
                            | DomProp::ValidityValueMissing
                            | DomProp::ValidityTypeMismatch
                            | DomProp::ValidityPatternMismatch
                            | DomProp::ValidityTooLong
                            | DomProp::ValidityTooShort
                            | DomProp::ValidityRangeUnderflow
                            | DomProp::ValidityRangeOverflow
                            | DomProp::ValidityStepMismatch
                            | DomProp::ValidityBadInput
                            | DomProp::ValidityValid
                            | DomProp::ValidityCustomError
                            | DomProp::ClassList
                            | DomProp::ClassListLength
                            | DomProp::Part
                            | DomProp::PartLength
                            | DomProp::TagName
                            | DomProp::LocalName
                            | DomProp::NamespaceUri
                            | DomProp::Prefix
                            | DomProp::NextElementSibling
                            | DomProp::PreviousElementSibling
                            | DomProp::ClientWidth
                            | DomProp::ClientHeight
                            | DomProp::ClientLeft
                            | DomProp::ClientTop
                            | DomProp::CurrentCssZoom
                            | DomProp::ScrollLeftMax
                            | DomProp::ScrollTopMax
                            | DomProp::ShadowRoot
                            | DomProp::AriaElementRefSingle(_)
                            | DomProp::AriaElementRefList(_)
                            | DomProp::OffsetWidth
                            | DomProp::ValueLength
                            | DomProp::OffsetHeight
                            | DomProp::OffsetLeft
                            | DomProp::OffsetTop
                            | DomProp::ScrollWidth
                            | DomProp::ScrollHeight
                            | DomProp::ScrollLeft
                            | DomProp::ScrollTop
                            | DomProp::ActiveElement
                            | DomProp::CharacterSet
                            | DomProp::CompatMode
                            | DomProp::ContentType
                            | DomProp::ReadyState
                            | DomProp::Referrer
                            | DomProp::Url
                            | DomProp::DocumentUri
                            | DomProp::BaseUri
                            | DomProp::LocationOrigin
                            | DomProp::LocationAncestorOrigins
                            | DomProp::History
                            | DomProp::HistoryLength
                            | DomProp::HistoryState
                            | DomProp::DefaultView
                            | DomProp::VisibilityState
                            | DomProp::Forms
                            | DomProp::Images
                            | DomProp::Links
                            | DomProp::Scripts
                            | DomProp::Children
                            | DomProp::ChildElementCount
                            | DomProp::FirstElementChild
                            | DomProp::LastElementChild
                            | DomProp::CurrentScript
                            | DomProp::FormsLength
                            | DomProp::ImagesLength
                            | DomProp::LinksLength
                            | DomProp::ScriptsLength
                            | DomProp::ChildrenLength
                            | DomProp::AnchorOrigin
                            | DomProp::AnchorRelList
                            | DomProp::AnchorRelListLength => {
                                let call = self.describe_dom_prop(prop);
                                return Err(Error::ScriptRuntime(format!("{call} is read-only")));
                            }
                            DomProp::Dataset(key) => {
                                self.dom.dataset_set(node, key, &value.as_string())?
                            }
                            DomProp::Style(prop) => {
                                self.dom.style_set(node, prop, &value.as_string())?
                            }
                        }
                    }
                    Stmt::ClassListCall {
                        target,
                        method,
                        class_names,
                        force,
                    } => {
                        let node = self.resolve_dom_query_required_runtime(target, env)?;
                        match method {
                            ClassListMethod::Add => {
                                for class_name in class_names {
                                    self.dom.class_add(node, class_name)?;
                                }
                            }
                            ClassListMethod::Remove => {
                                for class_name in class_names {
                                    self.dom.class_remove(node, class_name)?;
                                }
                            }
                            ClassListMethod::Toggle => {
                                let class_name = class_names.first().ok_or_else(|| {
                                    Error::ScriptRuntime("toggle requires a class name".into())
                                })?;
                                if let Some(force_expr) = force {
                                    let force_value = self
                                        .eval_expr(force_expr, env, event_param, event)?
                                        .truthy();
                                    if force_value {
                                        self.dom.class_add(node, class_name)?;
                                    } else {
                                        self.dom.class_remove(node, class_name)?;
                                    }
                                } else {
                                    let _ = self.dom.class_toggle(node, class_name)?;
                                }
                            }
                        }
                    }
                    Stmt::ClassListForEach {
                        target,
                        item_var,
                        index_var,
                        body,
                    } => {
                        let node = self.resolve_dom_query_required_runtime(target, env)?;
                        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
                        let prev_item = env.get(item_var).cloned();
                        let prev_item_const = self.is_const_binding(env, item_var);
                        let prev_index = index_var.as_ref().and_then(|v| env.get(v).cloned());
                        let prev_index_const = index_var
                            .as_ref()
                            .is_some_and(|name| self.is_const_binding(env, name));
                        self.set_const_binding(env, item_var, false);
                        if let Some(index_var) = index_var {
                            self.set_const_binding(env, index_var, false);
                        }

                        for (idx, class_name) in classes.iter().enumerate() {
                            let item_value = Value::String(class_name.clone());
                            env.insert(item_var.clone(), item_value.clone());
                            self.sync_global_binding_if_needed(env, item_var, &item_value);
                            if let Some(index_var) = index_var {
                                let index_value = Value::Number(idx as i64);
                                env.insert(index_var.clone(), index_value.clone());
                                self.sync_global_binding_if_needed(env, index_var, &index_value);
                            }
                            match self.execute_stmts(body, event_param, event, env)? {
                                ExecFlow::Continue => {}
                                ExecFlow::Break(None) => break,
                                ExecFlow::Break(label) => return Ok(ExecFlow::Break(label)),
                                ExecFlow::ContinueLoop(None) => continue,
                                ExecFlow::ContinueLoop(label) => {
                                    return Ok(ExecFlow::ContinueLoop(label));
                                }
                                ExecFlow::Return => return Ok(ExecFlow::Return),
                            }
                        }

                        if let Some(prev) = prev_item {
                            env.insert(item_var.clone(), prev.clone());
                            self.sync_global_binding_if_needed(env, item_var, &prev);
                        } else {
                            env.remove(item_var);
                        }
                        self.set_const_binding(env, item_var, prev_item_const);
                        if let Some(index_var) = index_var {
                            if let Some(prev) = prev_index {
                                env.insert(index_var.clone(), prev.clone());
                                self.sync_global_binding_if_needed(env, index_var, &prev);
                            } else {
                                env.remove(index_var);
                            }
                            self.set_const_binding(env, index_var, prev_index_const);
                        }
                    }
                    Stmt::DomSetAttribute {
                        target,
                        name,
                        value,
                    } => {
                        let node = self.resolve_dom_query_required_runtime(target, env)?;
                        let value = self.eval_expr(value, env, event_param, event)?;
                        if name.eq_ignore_ascii_case("open")
                            && self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
                        {
                            let _ = self.set_details_open_state_with_env(node, true, env)?;
                        } else {
                            self.dom.set_attr(node, name, &value.as_string())?;
                        }
                    }
                    Stmt::DomRemoveAttribute { target, name } => {
                        let node = self.resolve_dom_query_required_runtime(target, env)?;
                        if name.eq_ignore_ascii_case("open")
                            && self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
                        {
                            let _ = self.set_details_open_state_with_env(node, false, env)?;
                        } else {
                            self.dom.remove_attr(node, name)?;
                        }
                    }
                    Stmt::NodeTreeMutation {
                        target,
                        method,
                        child,
                        reference,
                    } => {
                        let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                        let child = self.eval_expr(child, env, event_param, event)?;
                        let Value::Node(child) = child else {
                            return Err(Error::ScriptRuntime(
                            "before/after/replaceWith/append/appendChild/prepend/removeChild/insertBefore argument must be an element reference".into(),
                        ));
                        };
                        match method {
                            NodeTreeMethod::After => self.dom.insert_after(target_node, child)?,
                            NodeTreeMethod::Append => self.dom.append_child(target_node, child)?,
                            NodeTreeMethod::AppendChild => {
                                self.dom.append_child(target_node, child)?
                            }
                            NodeTreeMethod::Before => {
                                let Some(parent) = self.dom.parent(target_node) else {
                                    continue;
                                };
                                self.dom.insert_before(parent, child, target_node)?;
                            }
                            NodeTreeMethod::ReplaceWith => {
                                self.dom.replace_with(target_node, child)?;
                            }
                            NodeTreeMethod::Prepend => {
                                self.dom.prepend_child(target_node, child)?
                            }
                            NodeTreeMethod::RemoveChild => {
                                self.dom.remove_child(target_node, child)?
                            }
                            NodeTreeMethod::InsertBefore => {
                                let Some(reference) = reference else {
                                    return Err(Error::ScriptRuntime(
                                        "insertBefore requires reference node".into(),
                                    ));
                                };
                                let reference =
                                    self.eval_expr(reference, env, event_param, event)?;
                                let Value::Node(reference) = reference else {
                                    return Err(Error::ScriptRuntime(
                                        "insertBefore reference must be an element reference"
                                            .into(),
                                    ));
                                };
                                self.dom.insert_before(target_node, child, reference)?;
                            }
                        }
                    }
                    Stmt::InsertAdjacentElement {
                        target,
                        position,
                        node,
                    } => {
                        let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                        let node = self.eval_expr(node, env, event_param, event)?;
                        let Value::Node(node) = node else {
                            return Err(Error::ScriptRuntime(
                            "insertAdjacentElement second argument must be an element reference"
                                .into(),
                        ));
                        };
                        self.dom
                            .insert_adjacent_node(target_node, *position, node)?;
                    }
                    Stmt::InsertAdjacentText {
                        target,
                        position,
                        text,
                    } => {
                        let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                        let text = self.eval_expr(text, env, event_param, event)?;
                        if matches!(
                            position,
                            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
                        ) && self.dom.parent(target_node).is_none()
                        {
                            continue;
                        }
                        let text_node = self.dom.create_detached_text(text.as_string());
                        self.dom
                            .insert_adjacent_node(target_node, *position, text_node)?;
                    }
                    Stmt::InsertAdjacentHTML {
                        target,
                        position,
                        html,
                    } => {
                        let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                        let position = self.eval_expr(position, env, event_param, event)?;
                        let position = resolve_insert_adjacent_position(&position.as_string())?;
                        let html = self.eval_expr(html, env, event_param, event)?;
                        self.dom
                            .insert_adjacent_html(target_node, position, &html.as_string())?;
                    }
                    Stmt::SetTimeout { handler, delay_ms } => {
                        let delay = self.eval_expr(delay_ms, env, event_param, event)?;
                        let delay = Self::value_to_i64(&delay);
                        let callback_args = handler
                            .args
                            .iter()
                            .map(|arg| self.eval_expr(arg, env, event_param, event))
                            .collect::<Result<Vec<_>>>()?;
                        let _ = self.schedule_timeout(
                            handler.callback.clone(),
                            delay,
                            callback_args,
                            env,
                        );
                    }
                    Stmt::SetInterval { handler, delay_ms } => {
                        let interval = self.eval_expr(delay_ms, env, event_param, event)?;
                        let interval = Self::value_to_i64(&interval);
                        let callback_args = handler
                            .args
                            .iter()
                            .map(|arg| self.eval_expr(arg, env, event_param, event))
                            .collect::<Result<Vec<_>>>()?;
                        let _ = self.schedule_interval(
                            handler.callback.clone(),
                            interval,
                            callback_args,
                            env,
                        );
                    }
                    Stmt::QueueMicrotask { handler } => {
                        self.queue_microtask(handler.clone(), env);
                    }
                    Stmt::ClearTimeout { timer_id } => {
                        let timer_id = self.eval_expr(timer_id, env, event_param, event)?;
                        let timer_id = Self::value_to_i64(&timer_id);
                        self.clear_timeout(timer_id);
                    }
                    Stmt::NodeRemove { target } => {
                        let node = self.resolve_dom_query_required_runtime(target, env)?;
                        if let Some(active) = self.dom.active_element() {
                            if active == node || self.dom.is_descendant_of(active, node) {
                                self.dom.set_active_element(None);
                            }
                        }
                        if let Some(active_pseudo) = self.dom.active_pseudo_element() {
                            if active_pseudo == node
                                || self.dom.is_descendant_of(active_pseudo, node)
                            {
                                self.dom.set_active_pseudo_element(None);
                            }
                        }
                        self.dom.remove_node(node)?;
                    }
                    Stmt::ForEach {
                        target,
                        selector,
                        item_var,
                        index_var,
                        body,
                    } => {
                        let items = if let Some(target) = target {
                            match self.resolve_dom_query_runtime(target, env)? {
                                Some(target_node) => {
                                    self.dom.query_selector_all_from(&target_node, selector)?
                                }
                                None => Vec::new(),
                            }
                        } else {
                            self.dom.query_selector_all(selector)?
                        };
                        let prev_item = env.get(item_var).cloned();
                        let prev_item_const = self.is_const_binding(env, item_var);
                        let prev_index = index_var.as_ref().and_then(|v| env.get(v).cloned());
                        let prev_index_const = index_var
                            .as_ref()
                            .is_some_and(|name| self.is_const_binding(env, name));
                        self.set_const_binding(env, item_var, false);
                        if let Some(index_var) = index_var {
                            self.set_const_binding(env, index_var, false);
                        }

                        for (idx, node) in items.iter().enumerate() {
                            let item_value = Value::Node(*node);
                            env.insert(item_var.clone(), item_value.clone());
                            self.sync_global_binding_if_needed(env, item_var, &item_value);
                            if let Some(index_var) = index_var {
                                let index_value = Value::Number(idx as i64);
                                env.insert(index_var.clone(), index_value.clone());
                                self.sync_global_binding_if_needed(env, index_var, &index_value);
                            }
                            match self.execute_stmts(body, event_param, event, env)? {
                                ExecFlow::Continue => {}
                                ExecFlow::Break(None) => break,
                                ExecFlow::Break(label) => return Ok(ExecFlow::Break(label)),
                                ExecFlow::ContinueLoop(None) => continue,
                                ExecFlow::ContinueLoop(label) => {
                                    return Ok(ExecFlow::ContinueLoop(label));
                                }
                                ExecFlow::Return => return Ok(ExecFlow::Return),
                            }
                        }

                        if let Some(prev) = prev_item {
                            env.insert(item_var.clone(), prev.clone());
                            self.sync_global_binding_if_needed(env, item_var, &prev);
                        } else {
                            env.remove(item_var);
                        }
                        self.set_const_binding(env, item_var, prev_item_const);
                        if let Some(index_var) = index_var {
                            if let Some(prev) = prev_index {
                                env.insert(index_var.clone(), prev.clone());
                                self.sync_global_binding_if_needed(env, index_var, &prev);
                            } else {
                                env.remove(index_var);
                            }
                            self.set_const_binding(env, index_var, prev_index_const);
                        }
                    }
                    Stmt::ArrayForEach { target, callback } => {
                        let target_value = env.get(target).cloned().ok_or_else(|| {
                            Error::ScriptRuntime(format!("unknown variable: {target}"))
                        })?;
                        self.execute_array_like_foreach_in_env(
                            target_value,
                            callback,
                            env,
                            event,
                            target,
                        )?;
                    }
                    Stmt::ArrayForEachExpr { target, callback } => {
                        let target_value = self.eval_expr(target, env, event_param, event)?;
                        self.execute_array_like_foreach_in_env(
                            target_value,
                            callback,
                            env,
                            event,
                            "<expression>",
                        )?;
                    }
                    Stmt::For {
                        init,
                        cond,
                        post,
                        body,
                    } => {
                        let previous_init_lexical =
                            self.collect_direct_block_lexical_bindings(init, env);
                        let loop_labels = self.take_pending_loop_labels();
                        self.push_loop_label_scope(loop_labels);
                        let for_result = (|| -> Result<ExecFlow> {
                            if !init.is_empty() {
                                match self.execute_stmts(init, event_param, event, env)? {
                                    ExecFlow::Continue => {}
                                    ExecFlow::Return => return Ok(ExecFlow::Return),
                                    ExecFlow::Break(label) => {
                                        return Err(Self::break_flow_error(&label));
                                    }
                                    ExecFlow::ContinueLoop(label) => {
                                        return Err(Self::continue_flow_error(&label));
                                    }
                                }
                            }

                            loop {
                                let should_run = if let Some(cond) = cond {
                                    self.eval_expr(cond, env, event_param, event)?.truthy()
                                } else {
                                    true
                                };
                                if !should_run {
                                    break;
                                }

                                match self.execute_stmts(body, event_param, event, env)? {
                                    ExecFlow::Continue => {}
                                    ExecFlow::ContinueLoop(label) => {
                                        if self.loop_should_consume_continue(&label) {
                                            if !post.is_empty() {
                                                match self.execute_stmts(
                                                    post,
                                                    event_param,
                                                    event,
                                                    env,
                                                )? {
                                                    ExecFlow::Continue => {}
                                                    ExecFlow::Return => {
                                                        return Ok(ExecFlow::Return);
                                                    }
                                                    ExecFlow::Break(_)
                                                    | ExecFlow::ContinueLoop(_) => {
                                                        return Err(Error::ScriptRuntime(
                                                            "invalid loop control in post expression"
                                                                .into(),
                                                        ));
                                                    }
                                                }
                                            }
                                            continue;
                                        }
                                        return Ok(ExecFlow::ContinueLoop(label));
                                    }
                                    ExecFlow::Break(label) => {
                                        if self.loop_should_consume_break(&label) {
                                            break;
                                        }
                                        return Ok(ExecFlow::Break(label));
                                    }
                                    ExecFlow::Return => return Ok(ExecFlow::Return),
                                }
                                if !post.is_empty() {
                                    match self.execute_stmts(post, event_param, event, env)? {
                                        ExecFlow::Continue => {}
                                        ExecFlow::Return => return Ok(ExecFlow::Return),
                                        ExecFlow::Break(_) | ExecFlow::ContinueLoop(_) => {
                                            return Err(Error::ScriptRuntime(
                                                "invalid loop control in post expression".into(),
                                            ));
                                        }
                                    }
                                }
                            }
                            Ok(ExecFlow::Continue)
                        })();
                        self.restore_block_lexical_bindings(previous_init_lexical, env);
                        self.pop_loop_label_scope();
                        match for_result? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                    Stmt::ForIn {
                        item_var,
                        iterable,
                        body,
                    } => {
                        let loop_labels = self.take_pending_loop_labels();
                        self.push_loop_label_scope(loop_labels);
                        let for_in_result = (|| -> Result<ExecFlow> {
                            let iterable = self.eval_expr(iterable, env, event_param, event)?;
                            let items = match iterable {
                                Value::NodeList(nodes) => (0..nodes.len())
                                    .map(|idx| Value::String(idx.to_string()))
                                    .collect::<Vec<_>>(),
                                Value::Array(values) => self
                                    .collect_for_in_array_keys(&values)
                                    .into_iter()
                                    .map(Value::String)
                                    .collect::<Vec<_>>(),
                                Value::Object(entries) => self
                                    .collect_for_in_object_chain_keys(&entries)
                                    .into_iter()
                                    .map(Value::String)
                                    .collect::<Vec<_>>(),
                                Value::Null | Value::Undefined => Vec::new(),
                                _ => {
                                    return Err(Error::ScriptRuntime(
                                        "for...in iterable must be a NodeList, Array, or Object"
                                            .into(),
                                    ));
                                }
                            };

                            let prev_item = env.get(item_var).cloned();
                            for item_value in items {
                                env.insert(item_var.clone(), item_value.clone());
                                self.sync_global_binding_if_needed(env, item_var, &item_value);
                                match self.execute_stmts(body, event_param, event, env)? {
                                    ExecFlow::Continue => {}
                                    ExecFlow::ContinueLoop(label) => {
                                        if self.loop_should_consume_continue(&label) {
                                            continue;
                                        }
                                        return Ok(ExecFlow::ContinueLoop(label));
                                    }
                                    ExecFlow::Break(label) => {
                                        if self.loop_should_consume_break(&label) {
                                            break;
                                        }
                                        return Ok(ExecFlow::Break(label));
                                    }
                                    ExecFlow::Return => return Ok(ExecFlow::Return),
                                }
                            }
                            if let Some(prev) = prev_item {
                                env.insert(item_var.clone(), prev.clone());
                                self.sync_global_binding_if_needed(env, item_var, &prev);
                            } else {
                                env.remove(item_var);
                            }
                            Ok(ExecFlow::Continue)
                        })();
                        self.pop_loop_label_scope();
                        match for_in_result? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                    Stmt::ForOf {
                        item_var,
                        iterable,
                        body,
                    } => {
                        let loop_labels = self.take_pending_loop_labels();
                        self.push_loop_label_scope(loop_labels);
                        let for_of_result = (|| -> Result<ExecFlow> {
                            enum ForOfSource {
                                Values(Vec<Value>),
                                InternalIterator(Rc<RefCell<ObjectValue>>),
                                ProtocolIterator(Rc<RefCell<ObjectValue>>),
                            }

                            let iterable = self.eval_expr(iterable, env, event_param, event)?;
                            let source = match iterable {
                                Value::NodeList(nodes) => ForOfSource::Values(
                                    nodes.into_iter().map(Value::Node).collect::<Vec<_>>(),
                                ),
                                Value::Array(values) => {
                                    ForOfSource::Values(values.borrow().clone())
                                }
                                Value::String(text) => ForOfSource::Values(
                                    text.chars()
                                        .map(|ch| Value::String(ch.to_string()))
                                        .collect::<Vec<_>>(),
                                ),
                                Value::TypedArray(values) => {
                                    ForOfSource::Values(self.typed_array_snapshot(&values)?)
                                }
                                Value::Map(map) => {
                                    ForOfSource::Values(self.map_entries_array(&map))
                                }
                                Value::Set(set) => ForOfSource::Values(set.borrow().values.clone()),
                                Value::Object(entries) => {
                                    if Self::is_iterator_object(&entries.borrow()) {
                                        ForOfSource::InternalIterator(entries)
                                    } else if Self::is_url_search_params_object(&entries.borrow()) {
                                        ForOfSource::Values(
                                            Self::url_search_params_pairs_from_object_entries(
                                                &entries.borrow(),
                                            )
                                            .into_iter()
                                            .map(|(key, value)| {
                                                Self::new_array_value(vec![
                                                    Value::String(key),
                                                    Value::String(value),
                                                ])
                                            })
                                            .collect::<Vec<_>>(),
                                        )
                                    } else if let Some(iterator) =
                                        self.for_of_symbol_iterator_factory_result(&entries, event)?
                                    {
                                        if Self::is_iterator_object(&iterator.borrow()) {
                                            ForOfSource::InternalIterator(iterator)
                                        } else {
                                            ForOfSource::ProtocolIterator(iterator)
                                        }
                                    } else {
                                        return Err(Error::ScriptRuntime(
                                        "for...of iterable must be an Iterator, NodeList, Array, String, TypedArray, Map, Set, or URLSearchParams"
                                            .into(),
                                    ));
                                    }
                                }
                                Value::Null | Value::Undefined => {
                                    return Err(Error::ScriptRuntime(
                                        "for...of iterable must be an Iterator, NodeList, Array, String, TypedArray, Map, Set, or URLSearchParams".into(),
                                    ));
                                }
                                _ => {
                                    return Err(Error::ScriptRuntime(
                                    "for...of iterable must be an Iterator, NodeList, Array, String, TypedArray, Map, Set, or URLSearchParams"
                                        .into(),
                                ));
                                }
                            };

                            let prev_item = env.get(item_var).cloned();
                            let loop_result = (|| -> Result<ExecFlow> {
                                match source {
                                    ForOfSource::Values(items) => {
                                        for item in items {
                                            env.insert(item_var.clone(), item.clone());
                                            self.sync_global_binding_if_needed(
                                                env, item_var, &item,
                                            );
                                            match self.execute_stmts(
                                                body,
                                                event_param,
                                                event,
                                                env,
                                            )? {
                                                ExecFlow::Continue => {}
                                                ExecFlow::ContinueLoop(label) => {
                                                    if self.loop_should_consume_continue(&label) {
                                                        continue;
                                                    }
                                                    return Ok(ExecFlow::ContinueLoop(label));
                                                }
                                                ExecFlow::Break(label) => {
                                                    if self.loop_should_consume_break(&label) {
                                                        break;
                                                    }
                                                    return Ok(ExecFlow::Break(label));
                                                }
                                                ExecFlow::Return => return Ok(ExecFlow::Return),
                                            }
                                        }
                                        Ok(ExecFlow::Continue)
                                    }
                                    ForOfSource::InternalIterator(iterator) => {
                                        loop {
                                            let Some(item) =
                                                self.iterator_next_value_from_object(&iterator)?
                                            else {
                                                break;
                                            };
                                            env.insert(item_var.clone(), item.clone());
                                            self.sync_global_binding_if_needed(
                                                env, item_var, &item,
                                            );
                                            let flow = match self.execute_stmts(
                                                body,
                                                event_param,
                                                event,
                                                env,
                                            ) {
                                                Ok(flow) => flow,
                                                Err(err) => {
                                                    self.for_of_internal_iterator_close_if_needed(
                                                        &iterator, event,
                                                    )?;
                                                    return Err(err);
                                                }
                                            };
                                            match flow {
                                                ExecFlow::Continue => {}
                                                ExecFlow::ContinueLoop(label) => {
                                                    if self.loop_should_consume_continue(&label) {
                                                        continue;
                                                    }
                                                    self.for_of_internal_iterator_close_if_needed(
                                                        &iterator, event,
                                                    )?;
                                                    return Ok(ExecFlow::ContinueLoop(label));
                                                }
                                                ExecFlow::Break(label) => {
                                                    self.for_of_internal_iterator_close_if_needed(
                                                        &iterator, event,
                                                    )?;
                                                    if self.loop_should_consume_break(&label) {
                                                        break;
                                                    }
                                                    return Ok(ExecFlow::Break(label));
                                                }
                                                ExecFlow::Return => {
                                                    self.for_of_internal_iterator_close_if_needed(
                                                        &iterator, event,
                                                    )?;
                                                    return Ok(ExecFlow::Return);
                                                }
                                            }
                                        }
                                        Ok(ExecFlow::Continue)
                                    }
                                    ForOfSource::ProtocolIterator(iterator) => {
                                        loop {
                                            let Some(item) = self
                                                .for_of_protocol_iterator_next(&iterator, event)?
                                            else {
                                                break;
                                            };
                                            env.insert(item_var.clone(), item.clone());
                                            self.sync_global_binding_if_needed(
                                                env, item_var, &item,
                                            );
                                            let flow = match self.execute_stmts(
                                                body,
                                                event_param,
                                                event,
                                                env,
                                            ) {
                                                Ok(flow) => flow,
                                                Err(err) => {
                                                    self.for_of_protocol_iterator_close(
                                                        &iterator, event,
                                                    )?;
                                                    return Err(err);
                                                }
                                            };
                                            match flow {
                                                ExecFlow::Continue => {}
                                                ExecFlow::ContinueLoop(label) => {
                                                    if self.loop_should_consume_continue(&label) {
                                                        continue;
                                                    }
                                                    self.for_of_protocol_iterator_close(
                                                        &iterator, event,
                                                    )?;
                                                    return Ok(ExecFlow::ContinueLoop(label));
                                                }
                                                ExecFlow::Break(label) => {
                                                    self.for_of_protocol_iterator_close(
                                                        &iterator, event,
                                                    )?;
                                                    if self.loop_should_consume_break(&label) {
                                                        break;
                                                    }
                                                    return Ok(ExecFlow::Break(label));
                                                }
                                                ExecFlow::Return => {
                                                    self.for_of_protocol_iterator_close(
                                                        &iterator, event,
                                                    )?;
                                                    return Ok(ExecFlow::Return);
                                                }
                                            }
                                        }
                                        Ok(ExecFlow::Continue)
                                    }
                                }
                            })();
                            if let Some(prev) = prev_item {
                                env.insert(item_var.clone(), prev.clone());
                                self.sync_global_binding_if_needed(env, item_var, &prev);
                            } else {
                                env.remove(item_var);
                            }
                            loop_result
                        })();
                        self.pop_loop_label_scope();
                        match for_of_result? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                    Stmt::ForAwaitOf {
                        item_var,
                        iterable,
                        body,
                    } => {
                        let loop_labels = self.take_pending_loop_labels();
                        self.push_loop_label_scope(loop_labels);
                        let for_await_result = (|| -> Result<ExecFlow> {
                            let iterable = self.eval_expr(iterable, env, event_param, event)?;
                            let values = match iterable {
                                Value::NodeList(nodes) => {
                                    nodes.into_iter().map(Value::Node).collect::<Vec<_>>()
                                }
                                Value::Array(values) => values.borrow().clone(),
                                Value::Map(map) => self.map_entries_array(&map),
                                Value::Set(set) => set.borrow().values.clone(),
                                Value::Object(entries) => {
                                    if Self::is_async_iterator_object(&entries.borrow()) {
                                        let mut out = Vec::new();
                                        while let Some(value) =
                                            self.async_iterator_next_value_from_object(&entries)?
                                        {
                                            out.push(value);
                                        }
                                        out
                                    } else {
                                        let async_iterator_symbol = self
                                            .eval_symbol_static_property(
                                                SymbolStaticProperty::AsyncIterator,
                                            );
                                        let async_iterator_key = self
                                            .property_key_to_storage_key(&async_iterator_symbol);
                                        let async_iterator_factory = {
                                            let entries_ref = entries.borrow();
                                            Self::object_get_entry(
                                                &entries_ref,
                                                async_iterator_key.as_str(),
                                            )
                                        };

                                        if let Some(factory) = async_iterator_factory {
                                            if !self.is_callable_value(&factory) {
                                                return Err(Error::ScriptRuntime(
                                                    "for await...of async iterator factory is not callable"
                                                        .into(),
                                                ));
                                            }
                                            let iterator_value =
                                                self.execute_callable_value(&factory, &[], event)?;
                                            let Value::Object(async_iterator) = iterator_value
                                            else {
                                                return Err(Error::ScriptRuntime(
                                                    "for await...of async iterator factory must return an object"
                                                        .into(),
                                                ));
                                            };
                                            if !Self::is_async_iterator_object(
                                                &async_iterator.borrow(),
                                            ) {
                                                return Err(Error::ScriptRuntime(
                                                    "for await...of async iterator factory returned a non-async iterator"
                                                        .into(),
                                                ));
                                            }
                                            let mut out = Vec::new();
                                            while let Some(value) = self
                                                .async_iterator_next_value_from_object(
                                                    &async_iterator,
                                                )?
                                            {
                                                out.push(value);
                                            }
                                            out
                                        } else if Self::is_iterator_object(&entries.borrow()) {
                                            self.iterator_collect_remaining_values(&entries)?
                                        } else if Self::is_url_search_params_object(
                                            &entries.borrow(),
                                        ) {
                                            Self::url_search_params_pairs_from_object_entries(
                                                &entries.borrow(),
                                            )
                                            .into_iter()
                                            .map(|(key, value)| {
                                                Self::new_array_value(vec![
                                                    Value::String(key),
                                                    Value::String(value),
                                                ])
                                            })
                                            .collect::<Vec<_>>()
                                        } else {
                                            return Err(Error::ScriptRuntime(
                                                "for await...of iterable must be an AsyncIterator, Iterator, NodeList, Array, Map, Set, or URLSearchParams".into(),
                                            ));
                                        }
                                    }
                                }
                                Value::Null | Value::Undefined => Vec::new(),
                                _ => {
                                    return Err(Error::ScriptRuntime(
                                        "for await...of iterable must be an AsyncIterator, Iterator, NodeList, Array, Map, Set, or URLSearchParams".into(),
                                    ));
                                }
                            };

                            let prev_item = env.get(item_var).cloned();
                            for value in values {
                                let item = self.await_value_in_for_await(value)?;
                                env.insert(item_var.clone(), item.clone());
                                self.sync_global_binding_if_needed(env, item_var, &item);
                                match self.execute_stmts(body, event_param, event, env)? {
                                    ExecFlow::Continue => {}
                                    ExecFlow::ContinueLoop(label) => {
                                        if self.loop_should_consume_continue(&label) {
                                            continue;
                                        }
                                        return Ok(ExecFlow::ContinueLoop(label));
                                    }
                                    ExecFlow::Break(label) => {
                                        if self.loop_should_consume_break(&label) {
                                            break;
                                        }
                                        return Ok(ExecFlow::Break(label));
                                    }
                                    ExecFlow::Return => return Ok(ExecFlow::Return),
                                }
                            }
                            if let Some(prev) = prev_item {
                                env.insert(item_var.clone(), prev.clone());
                                self.sync_global_binding_if_needed(env, item_var, &prev);
                            } else {
                                env.remove(item_var);
                            }
                            Ok(ExecFlow::Continue)
                        })();
                        self.pop_loop_label_scope();
                        match for_await_result? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                    Stmt::While { cond, body } => {
                        let loop_labels = self.take_pending_loop_labels();
                        self.push_loop_label_scope(loop_labels);
                        let while_result = (|| -> Result<ExecFlow> {
                            while self.eval_expr(cond, env, event_param, event)?.truthy() {
                                match self.execute_stmts(body, event_param, event, env)? {
                                    ExecFlow::Continue => {}
                                    ExecFlow::ContinueLoop(label) => {
                                        if self.loop_should_consume_continue(&label) {
                                            continue;
                                        }
                                        return Ok(ExecFlow::ContinueLoop(label));
                                    }
                                    ExecFlow::Break(label) => {
                                        if self.loop_should_consume_break(&label) {
                                            break;
                                        }
                                        return Ok(ExecFlow::Break(label));
                                    }
                                    ExecFlow::Return => return Ok(ExecFlow::Return),
                                }
                            }
                            Ok(ExecFlow::Continue)
                        })();
                        self.pop_loop_label_scope();
                        match while_result? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                    Stmt::DoWhile { cond, body } => {
                        let loop_labels = self.take_pending_loop_labels();
                        self.push_loop_label_scope(loop_labels);
                        let do_while_result = (|| -> Result<ExecFlow> {
                            loop {
                                match self.execute_stmts(body, event_param, event, env)? {
                                    ExecFlow::Continue => {}
                                    ExecFlow::ContinueLoop(label) => {
                                        if self.loop_should_consume_continue(&label) {
                                        } else {
                                            return Ok(ExecFlow::ContinueLoop(label));
                                        }
                                    }
                                    ExecFlow::Break(label) => {
                                        if self.loop_should_consume_break(&label) {
                                            break;
                                        }
                                        return Ok(ExecFlow::Break(label));
                                    }
                                    ExecFlow::Return => return Ok(ExecFlow::Return),
                                }
                                if !self.eval_expr(cond, env, event_param, event)?.truthy() {
                                    break;
                                }
                            }
                            Ok(ExecFlow::Continue)
                        })();
                        self.pop_loop_label_scope();
                        match do_while_result? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                    Stmt::Switch { expr, clauses } => {
                        let switch_value = self.eval_expr(expr, env, event_param, event)?;
                        let all_clause_stmts = clauses
                            .iter()
                            .flat_map(|clause| clause.stmts.iter().cloned())
                            .collect::<Vec<_>>();
                        Self::validate_const_redeclarations(&all_clause_stmts)?;
                        let previous = self.collect_direct_block_lexical_bindings(&all_clause_stmts, env);

                        let switch_result = (|| -> Result<ExecFlow> {
                            let pending_switch_tdz_bindings =
                                Self::collect_direct_tdz_binding_names(&all_clause_stmts);
                            self.push_tdz_scope_frame(pending_switch_tdz_bindings);

                            let mut default_index = None;
                            let mut matched_index = None;

                            for (index, clause) in clauses.iter().enumerate() {
                                if let Some(test) = &clause.test {
                                    let case_value =
                                        self.eval_expr(test, env, event_param, event)?;
                                    if self.strict_equal(&switch_value, &case_value) {
                                        matched_index = Some(index);
                                        break;
                                    }
                                } else if default_index.is_none() {
                                    default_index = Some(index);
                                }
                            }

                            if let Some(start_index) = matched_index.or(default_index) {
                                let mut selected_stmts = Vec::new();
                                for clause in clauses.iter().skip(start_index) {
                                    selected_stmts.extend(clause.stmts.iter().cloned());
                                }
                                match self.execute_stmts(&selected_stmts, event_param, event, env)? {
                                    ExecFlow::Continue => {}
                                    ExecFlow::Break(label) => {
                                        if label.is_none() {
                                        } else {
                                            return Ok(ExecFlow::Break(label));
                                        }
                                    }
                                    flow => return Ok(flow),
                                }
                            }

                            Ok(ExecFlow::Continue)
                        })();

                        self.pop_tdz_scope_frame();
                        self.restore_block_lexical_bindings(previous, env);

                        match switch_result? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                    Stmt::If {
                        cond,
                        then_stmts,
                        else_stmts,
                    } => {
                        let cond = self.eval_expr(cond, env, event_param, event)?;
                        if cond.truthy() {
                            match self.execute_stmts(then_stmts, event_param, event, env)? {
                                ExecFlow::Continue => {}
                                flow => return Ok(flow),
                            }
                        } else {
                            match self.execute_stmts(else_stmts, event_param, event, env)? {
                                ExecFlow::Continue => {}
                                flow => return Ok(flow),
                            }
                        }
                    }
                    Stmt::Try {
                        try_stmts,
                        catch_binding,
                        catch_stmts,
                        finally_stmts,
                    } => {
                        let mut completion = self.execute_stmts(try_stmts, event_param, event, env);

                        if let Err(err) = completion {
                            if let Some(catch_stmts) = catch_stmts {
                                let caught = Self::error_to_catch_value(err)?;
                                completion = self.execute_catch_block(
                                    catch_binding,
                                    catch_stmts,
                                    caught,
                                    event_param,
                                    event,
                                    env,
                                );
                            } else {
                                completion = Err(err);
                            }
                        }

                        if let Some(finally_stmts) = finally_stmts {
                            match self.execute_stmts(finally_stmts, event_param, event, env) {
                                Ok(ExecFlow::Continue) => {}
                                Ok(flow) => return Ok(flow),
                                Err(err) => return Err(err),
                            }
                        }

                        match completion {
                            Ok(ExecFlow::Continue) => {}
                            Ok(flow) => return Ok(flow),
                            Err(err) => return Err(err),
                        }
                    }
                    Stmt::Throw { value } => {
                        let thrown = self.eval_expr(value, env, event_param, event)?;
                        return Err(Error::ScriptThrown(ThrownValue::new(thrown)));
                    }
                    Stmt::Return { value } => {
                        let return_value = if let Some(value) = value {
                            self.eval_expr(value, env, event_param, event)?
                        } else {
                            Value::Undefined
                        };
                        env.insert(INTERNAL_RETURN_SLOT.to_string(), return_value);
                        return Ok(ExecFlow::Return);
                    }
                    Stmt::Empty => {}
                    Stmt::Debugger => {}
                    Stmt::Break { label } => {
                        return Ok(ExecFlow::Break(label.clone()));
                    }
                    Stmt::Continue { label } => {
                        return Ok(ExecFlow::ContinueLoop(label.clone()));
                    }
                    Stmt::EventCall { event_var, method } => {
                        if let Some(param) = event_param {
                            if param == event_var {
                                match method {
                                    EventMethod::PreventDefault => {
                                        if event.cancelable {
                                            event.default_prevented = true;
                                        }
                                    }
                                    EventMethod::StopPropagation => {
                                        event.propagation_stopped = true;
                                    }
                                    EventMethod::StopImmediatePropagation => {
                                        event.immediate_propagation_stopped = true;
                                        event.propagation_stopped = true;
                                    }
                                }
                            }
                        }
                    }
                    Stmt::ListenerMutation {
                        target,
                        op,
                        event_type,
                        capture,
                        handler,
                    } => {
                        let node = self.resolve_dom_query_required_runtime(target, env)?;
                        match op {
                            ListenerRegistrationOp::Add => {
                                let captured_env = self.ensure_listener_capture_env();
                                *captured_env.borrow_mut() = ScriptEnv::from_snapshot(env);
                                self.listeners.add(
                                    node,
                                    event_type.clone(),
                                    Listener {
                                        capture: *capture,
                                        handler: handler.clone(),
                                        captured_env,
                                        captured_pending_function_decls: self
                                            .script_runtime
                                            .pending_function_decls
                                            .clone(),
                                    },
                                );
                            }
                            ListenerRegistrationOp::Remove => {
                                let _ = self.listeners.remove(node, event_type, *capture, handler);
                            }
                        }
                    }
                    Stmt::DomMethodCall {
                        target,
                        method,
                        arg,
                    } => {
                        let node = self.resolve_dom_query_required_runtime(target, env)?;
                        let arg_value = arg
                            .as_ref()
                            .map(|expr| self.eval_expr(expr, env, event_param, event))
                            .transpose()?;
                        match method {
                            DomMethod::Focus => self.focus_node_with_env(node, env)?,
                            DomMethod::Blur => self.blur_node_with_env(node, env)?,
                            DomMethod::Click => self.click_node_with_env(node, env)?,
                            DomMethod::Submit => self.submit_form_with_env(node, env)?,
                            DomMethod::RequestSubmit => {
                                self.request_submit_form_with_env(node, arg_value, env)?
                            }
                            DomMethod::Reset => self.reset_form_with_env(node, env)?,
                            DomMethod::ScrollIntoView => {
                                self.scroll_into_view_node_with_env(node, env)?
                            }
                            DomMethod::Show => self.show_dialog_with_env(node, false, env)?,
                            DomMethod::ShowModal => self.show_dialog_with_env(node, true, env)?,
                            DomMethod::Close => self.close_dialog_with_env(node, arg_value, env)?,
                            DomMethod::RequestClose => {
                                self.request_close_dialog_with_env(node, arg_value, env)?
                            }
                        }
                    }
                    Stmt::DispatchEvent { target, event_type } => {
                        let node = self.resolve_dom_query_required_runtime(target, env)?;
                        let event_name = self
                            .eval_expr(event_type, env, event_param, event)?
                            .as_string();
                        if event_name.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "dispatchEvent requires non-empty event type".into(),
                            ));
                        }
                        let _ = self.dispatch_event_with_env(node, &event_name, env, false)?;
                    }
                    Stmt::Expr(expr) => {
                        let _ = self.eval_expr(expr, env, event_param, event)?;
                    }
                }
                }

                self.apply_pending_listener_capture_env_updates(env);
                Ok(ExecFlow::Continue)
            })();

            self.pop_tdz_scope_frame();
            flow_result
        })();

        self.script_runtime.listener_capture_env_stack.pop();
        self.restore_pending_function_decl_scopes(pending_scope_start);
        result
    }

    pub(crate) fn sync_top_level_env_from_runtime(&mut self, env: &mut HashMap<String, Value>) {
        if Self::env_scope_depth(env) != 0 {
            return;
        }

        let runtime_snapshot = self.script_runtime.env.to_map();
        for (name, runtime_value) in runtime_snapshot {
            if Self::is_internal_env_key(&name) {
                continue;
            }
            let should_update = match env.get(&name) {
                Some(current) => !self.strict_equal(current, &runtime_value),
                None => true,
            };
            if should_update {
                env.insert(name, runtime_value);
            }
        }
    }
}
