use super::*;

#[path = "capture_name_expr_analysis.rs"]
mod capture_name_expr_analysis;
#[path = "capture_name_stmt_analysis.rs"]
mod capture_name_stmt_analysis;

impl Harness {
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
}
