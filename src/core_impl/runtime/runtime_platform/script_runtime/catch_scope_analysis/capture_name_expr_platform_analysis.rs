use super::*;

impl Harness {
    pub(crate) fn collect_capture_names_from_platform_expr(
        expr: &Expr,
        out: &mut HashSet<String>,
    ) -> bool {
        match expr {
            Expr::StructuredClone { value, options }
            | Expr::Fetch {
                request: value,
                options,
            } => {
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
            _ => return false,
        }
        true
    }
}
