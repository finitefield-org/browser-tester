use super::*;

impl Harness {
    fn array_destructure_binding_names(pattern: &ArrayDestructurePattern) -> Vec<String> {
        pattern
            .items
            .iter()
            .flatten()
            .map(|binding| binding.target.clone())
            .chain(pattern.rest.iter().cloned())
            .collect()
    }

    fn object_destructure_binding_names(pattern: &ObjectDestructurePattern) -> Vec<String> {
        pattern
            .bindings
            .iter()
            .map(|binding| binding.target.clone())
            .chain(pattern.rest.iter().cloned())
            .collect()
    }

    pub(crate) fn direct_decl_binding_kinds(stmt: &Stmt) -> Vec<(String, bool)> {
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
                pattern,
                decl_kind: Some(kind),
                ..
            } => {
                let is_lexical = !matches!(kind, VarDeclKind::Var);
                Self::array_destructure_binding_names(pattern)
                    .into_iter()
                    .map(|name| (name, is_lexical))
                    .collect()
            }
            Stmt::ObjectDestructureAssign {
                pattern,
                decl_kind: Some(kind),
                ..
            } => {
                let is_lexical = !matches!(kind, VarDeclKind::Var);
                Self::object_destructure_binding_names(pattern)
                    .into_iter()
                    .map(|target_name| (target_name, is_lexical))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    pub(crate) fn direct_tdz_binding_names(stmt: &Stmt) -> Vec<String> {
        match stmt {
            Stmt::VarDecl {
                name,
                kind: VarDeclKind::Let | VarDeclKind::Const,
                ..
            } => vec![name.clone()],
            Stmt::ClassDecl { name, .. } => vec![name.clone()],
            Stmt::ArrayDestructureAssign {
                pattern,
                decl_kind: Some(VarDeclKind::Let | VarDeclKind::Const),
                ..
            } => Self::array_destructure_binding_names(pattern),
            Stmt::ObjectDestructureAssign {
                pattern,
                decl_kind: Some(VarDeclKind::Let | VarDeclKind::Const),
                ..
            } => Self::object_destructure_binding_names(pattern),
            Stmt::ExportDecl { declaration, .. } => Self::direct_tdz_binding_names(declaration),
            _ => Vec::new(),
        }
    }

    pub(crate) fn collect_direct_tdz_binding_names(stmts: &[Stmt]) -> HashSet<String> {
        let mut out = HashSet::new();
        for stmt in stmts {
            out.extend(Self::direct_tdz_binding_names(stmt));
        }
        out
    }

    pub(crate) fn collect_direct_lexical_binding_names(stmts: &[Stmt]) -> HashSet<String> {
        let mut out = HashSet::new();
        for stmt in stmts {
            for (name, is_lexical) in Self::direct_decl_binding_kinds(stmt) {
                if is_lexical {
                    out.insert(name);
                }
            }
        }
        out
    }

    pub(crate) fn env_top_level_lexical_binding_names(
        env: &HashMap<String, Value>,
    ) -> HashSet<String> {
        match env.get(INTERNAL_TOP_LEVEL_LEXICAL_BINDINGS_KEY) {
            Some(Value::Array(bindings)) => bindings
                .borrow()
                .iter()
                .filter_map(|entry| match entry {
                    Value::String(name) => Some(name.clone()),
                    _ => None,
                })
                .collect(),
            _ => HashSet::new(),
        }
    }

    pub(crate) fn ensure_top_level_global_sync_names(
        &mut self,
        stmts: &[Stmt],
        env: &mut HashMap<String, Value>,
    ) {
        if Self::env_scope_depth(env) != 0 {
            return;
        }

        let lexical_bindings = Self::env_top_level_lexical_binding_names(env);
        let mut sync_names = HashSet::new();
        if let Some(Value::Array(existing)) = env.get(INTERNAL_GLOBAL_SYNC_NAMES_KEY) {
            for entry in existing.borrow().iter() {
                if let Value::String(name) = entry {
                    sync_names.insert(name.clone());
                }
            }
        }
        for name in self.script_runtime.env.keys() {
            sync_names.insert(name.clone());
        }
        for name in env.keys() {
            sync_names.insert(name.clone());
        }
        sync_names.extend(Self::collect_var_declared_names(stmts));
        sync_names.extend(Self::collect_function_decls(stmts).into_keys());

        let mut sync_names = sync_names
            .into_iter()
            .filter(|name| !Self::is_internal_env_key(name) && !lexical_bindings.contains(name))
            .collect::<Vec<_>>();
        sync_names.sort();
        if sync_names.is_empty() {
            env.remove(INTERNAL_GLOBAL_SYNC_NAMES_KEY);
            return;
        }
        env.insert(
            INTERNAL_GLOBAL_SYNC_NAMES_KEY.to_string(),
            Self::new_array_value(sync_names.into_iter().map(Value::String).collect()),
        );
    }

    pub(crate) fn collect_var_declared_names(stmts: &[Stmt]) -> HashSet<String> {
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
                pattern,
                decl_kind: Some(VarDeclKind::Var),
                ..
            } => {
                for name in Self::array_destructure_binding_names(pattern) {
                    out.insert(name.clone());
                }
            }
            Stmt::ObjectDestructureAssign {
                pattern,
                decl_kind: Some(VarDeclKind::Var),
                ..
            } => {
                for target in Self::object_destructure_binding_names(pattern) {
                    out.insert(target);
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

    pub(crate) fn hoist_var_declarations(
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
                pattern,
                decl_kind: Some(VarDeclKind::Let),
                ..
            } => Self::array_destructure_binding_names(pattern),
            Stmt::ObjectDestructureAssign {
                pattern,
                decl_kind: Some(VarDeclKind::Let),
                ..
            } => Self::object_destructure_binding_names(pattern),
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

    pub(crate) fn validate_const_redeclarations(stmts: &[Stmt]) -> Result<()> {
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

    pub(crate) fn push_tdz_scope_frame(&mut self, declared: HashSet<String>) {
        self.script_runtime.tdz_scope_stack.push(TdzScopeFrame {
            pending: declared.clone(),
            declared,
        });
    }

    pub(crate) fn pop_tdz_scope_frame(&mut self) {
        self.script_runtime.tdz_scope_stack.pop();
    }

    pub(crate) fn mark_tdz_initialized(
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

    pub(crate) fn collect_direct_block_lexical_bindings(
        &self,
        stmts: &[Stmt],
        env: &HashMap<String, Value>,
    ) -> Vec<(String, Option<Value>, bool)> {
        let mut seen = HashSet::new();
        let mut previous = Vec::new();
        for stmt in stmts {
            let names: Vec<String> = match stmt {
                Stmt::ImportDecl {
                    default_binding,
                    namespace_binding,
                    named_bindings,
                    ..
                } => {
                    let mut names = Vec::new();
                    if let Some(name) = default_binding {
                        names.push(name.clone());
                    }
                    if let Some(name) = namespace_binding {
                        names.push(name.clone());
                    }
                    names.extend(named_bindings.iter().map(|binding| binding.local.clone()));
                    names
                }
                Stmt::VarDecl { name, kind, .. } => {
                    if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                        vec![name.clone()]
                    } else {
                        Vec::new()
                    }
                }
                Stmt::ClassDecl { name, .. } => vec![name.clone()],
                Stmt::FunctionDecl { name, .. } => vec![name.clone()],
                Stmt::ExportDecl { declaration, .. } => match declaration.as_ref() {
                    Stmt::VarDecl { name, kind, .. } => {
                        if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                            vec![name.clone()]
                        } else {
                            Vec::new()
                        }
                    }
                    Stmt::ClassDecl { name, .. } => vec![name.clone()],
                    Stmt::FunctionDecl { name, .. } => vec![name.clone()],
                    Stmt::ArrayDestructureAssign {
                        pattern,
                        decl_kind: Some(kind),
                        ..
                    } => {
                        if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                            Self::array_destructure_binding_names(pattern)
                        } else {
                            Vec::new()
                        }
                    }
                    Stmt::ObjectDestructureAssign {
                        pattern,
                        decl_kind: Some(kind),
                        ..
                    } => {
                        if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                            Self::object_destructure_binding_names(pattern)
                        } else {
                            Vec::new()
                        }
                    }
                    _ => Vec::new(),
                },
                Stmt::ArrayDestructureAssign {
                    pattern,
                    decl_kind: Some(kind),
                    ..
                } => {
                    if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                        Self::array_destructure_binding_names(pattern)
                    } else {
                        Vec::new()
                    }
                }
                Stmt::ObjectDestructureAssign {
                    pattern,
                    decl_kind: Some(kind),
                    ..
                } => {
                    if matches!(kind, VarDeclKind::Let | VarDeclKind::Const) {
                        Self::object_destructure_binding_names(pattern)
                    } else {
                        Vec::new()
                    }
                }
                _ => Vec::new(),
            };
            for name in names {
                if seen.insert(name.clone()) {
                    previous.push((
                        name.clone(),
                        env.get(&name).cloned(),
                        self.is_const_binding(env, &name),
                    ));
                }
            }
        }
        previous
    }

    pub(crate) fn restore_block_lexical_bindings(
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
}
