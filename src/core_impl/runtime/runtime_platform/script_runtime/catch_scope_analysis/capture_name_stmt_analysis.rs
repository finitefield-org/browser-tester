use super::*;

impl Harness {
    pub(crate) fn collect_capture_names_from_stmts(stmts: &[Stmt], out: &mut HashSet<String>) {
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
            Stmt::ExportDecl { declaration, .. }
            | Stmt::Label {
                stmt: declaration, ..
            } => {
                Self::collect_capture_names_from_stmt(declaration, out);
            }
            Stmt::ExportDefaultExpr { expr } | Stmt::Throw { value: expr } | Stmt::Expr(expr) => {
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
            Stmt::ObjectAssign {
                target, path, expr, ..
            } => {
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
}
