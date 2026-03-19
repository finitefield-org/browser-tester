use super::*;

impl Harness {
    pub(crate) fn try_eval_dom_query_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match expr {
            Expr::ClipboardMethodCall { method, args } => {
                return self
                    .eval_clipboard_method_call(method, args, env, event_param, event)
                    .map(Some);
            }
            Expr::DocumentHasFocus => Value::Bool(self.dom.active_element().is_some()),
            Expr::DomMatches { target, selector } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                return self.eval_matches_selector_value(node, selector).map(Some);
            }
            Expr::DomClosest { target, selector } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                return self.eval_closest_selector_value(node, selector).map(Some);
            }
            Expr::DomComputedStyleProperty { target, property } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Value::String(self.computed_style_property_value(node, None, property)?)
            }
            Expr::ClassListContains { target, class_name } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Value::Bool(self.dom.class_contains(node, class_name)?)
            }
            Expr::QuerySelectorAllLength { target } => {
                let len = self
                    .resolve_dom_query_list_runtime(target, env)?
                    .unwrap_or_default()
                    .len() as i64;
                Value::Number(len)
            }
            Expr::FormElementsLength { form } => {
                let form_node = self.resolve_dom_query_required_runtime(form, env)?;
                Value::Number(self.form_elements(form_node)?.len() as i64)
            }
            Expr::DomGetAttribute { target, name } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                let name = name.to_ascii_lowercase();
                if name == "nonce" {
                    if self.dom.attr(node, "nonce").is_some() {
                        Value::String(String::new())
                    } else {
                        Value::Null
                    }
                } else {
                    self.dom
                        .attr(node, &name)
                        .map(Value::String)
                        .unwrap_or(Value::Null)
                }
            }
            Expr::DomHasAttribute { target, name } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Value::Bool(self.dom.has_attr(node, name)?)
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
