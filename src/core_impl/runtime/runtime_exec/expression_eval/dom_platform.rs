use super::*;

impl Harness {
    pub(crate) fn dom_prop_non_node_fallback_path(prop: &DomProp) -> Option<Vec<&'static str>> {
        match prop {
            DomProp::ValueLength => Some(vec!["value", "length"]),
            DomProp::FilesLength => Some(vec!["files", "length"]),
            DomProp::ClassListLength => Some(vec!["classList", "length"]),
            DomProp::PartLength => Some(vec!["part", "length"]),
            DomProp::AdoptedStyleSheetsLength => Some(vec!["adoptedStyleSheets", "length"]),
            DomProp::HistoryLength => Some(vec!["history", "length"]),
            DomProp::FormsLength => Some(vec!["forms", "length"]),
            DomProp::ImagesLength => Some(vec!["images", "length"]),
            DomProp::LinksLength => Some(vec!["links", "length"]),
            DomProp::ScriptsLength => Some(vec!["scripts", "length"]),
            DomProp::ChildrenLength => Some(vec!["children", "length"]),
            DomProp::AnchorRelListLength => Some(vec!["relList", "length"]),
            _ => Self::object_key_from_dom_prop(prop).map(|key| vec![key]),
        }
    }

    pub(crate) fn eval_expr_dom_and_platform(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::DomRead { target, prop } => {
                    let target_value = self.resolve_dom_query_value_runtime(target, env)?;
                    if let Some(value) = target_value {
                        if !matches!(value, Value::Node(_) | Value::NodeList(_)) {
                            if let Some(path) = Self::dom_prop_non_node_fallback_path(prop) {
                                let variable_name = target.describe_call();
                                let mut current = value;
                                for key in path {
                                    current = self.object_property_from_named_value(
                                        &variable_name,
                                        &current,
                                        key,
                                    )?;
                                }
                                return Ok(current);
                            }
                        }
                    }
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    if let Some(value) = self.try_eval_dom_read_form_prop(node, prop)? {
                        return Ok(value);
                    }
                    if let Some(value) = self.try_eval_dom_read_element_prop(node, prop)? {
                        return Ok(value);
                    }
                    if let Some(value) = self.try_eval_dom_read_reflected_prop(node, prop)? {
                        return Ok(value);
                    }
                    if let Some(value) = self.try_eval_dom_read_layout_prop(node, prop)? {
                        return Ok(value);
                    }
                    if let Some(value) = self.try_eval_dom_read_document_prop(node, prop)? {
                        return Ok(value);
                    }
                    if let Some(value) = self.try_eval_dom_read_media_embed_prop(node, prop)? {
                        return Ok(value);
                    }
                    if let Some(value) = self.try_eval_dom_read_anchor_prop(node, prop)? {
                        return Ok(value);
                    }
                    if let Some(value) = self.try_eval_dom_read_dimension_prop(node, prop)? {
                        return Ok(value);
                    }
                    Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into()))
                }
                _ => {
                    if let Some(value) =
                        self.try_eval_dom_navigation_expr(expr, env, event_param, event)?
                    {
                        return Ok(value);
                    }
                    if let Some(value) =
                        self.try_eval_dom_query_expr(expr, env, event_param, event)?
                    {
                        return Ok(value);
                    }
                    if let Some(value) =
                        self.try_eval_dom_form_data_expr(expr, env, event_param, event)?
                    {
                        return Ok(value);
                    }
                    Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into()))
                }
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
