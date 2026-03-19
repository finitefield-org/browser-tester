use super::*;

impl Harness {
    pub(super) fn execute_class_list_call_stmt(
        &mut self,
        target: &DomQuery,
        optional: bool,
        method: &ClassListMethod,
        class_names: &[Expr],
        force: &Option<Expr>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let node = if optional {
            if let DomQuery::Var(name) = target {
                if matches!(env.get(name), Some(Value::Null | Value::Undefined)) {
                    return Ok(ExecFlow::Continue);
                }
            }
            match self.resolve_dom_query_runtime(target, env)? {
                Some(node) => node,
                None => return Ok(ExecFlow::Continue),
            }
        } else {
            self.resolve_dom_query_required_runtime(target, env)?
        };
        match method {
            ClassListMethod::Add => {
                for class_name in class_names {
                    let class_name = self
                        .eval_expr(class_name, env, event_param, event)?
                        .as_string();
                    self.dom.class_add(node, &class_name)?;
                }
            }
            ClassListMethod::Remove => {
                for class_name in class_names {
                    let class_name = self
                        .eval_expr(class_name, env, event_param, event)?
                        .as_string();
                    self.dom.class_remove(node, &class_name)?;
                }
            }
            ClassListMethod::Toggle => {
                let class_name = class_names
                    .first()
                    .ok_or_else(|| Error::ScriptRuntime("toggle requires a class name".into()))?;
                let class_name = self
                    .eval_expr(class_name, env, event_param, event)?
                    .as_string();
                if let Some(force_expr) = force {
                    let force_value = self
                        .eval_expr(force_expr, env, event_param, event)?
                        .truthy();
                    if force_value {
                        self.dom.class_add(node, &class_name)?;
                    } else {
                        self.dom.class_remove(node, &class_name)?;
                    }
                } else {
                    let _ = self.dom.class_toggle(node, &class_name)?;
                }
            }
            ClassListMethod::Replace => {
                let old_class_name = class_names.first().ok_or_else(|| {
                    Error::ScriptRuntime("replace requires old and new class names".into())
                })?;
                let new_class_name = class_names.get(1).ok_or_else(|| {
                    Error::ScriptRuntime("replace requires old and new class names".into())
                })?;
                let old_class_name = self
                    .eval_expr(old_class_name, env, event_param, event)?
                    .as_string();
                let new_class_name = self
                    .eval_expr(new_class_name, env, event_param, event)?
                    .as_string();
                let _ = self
                    .dom
                    .class_replace(node, &old_class_name, &new_class_name)?;
            }
        }
        Ok(ExecFlow::Continue)
    }

    pub(super) fn execute_dom_set_attribute_stmt(
        &mut self,
        target: &DomQuery,
        name: &str,
        value: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let node = self.resolve_dom_query_required_runtime(target, env)?;
        let value = self.eval_expr(value, env, event_param, event)?;
        let normalized_name = name.to_ascii_lowercase();
        if !is_valid_create_attribute_name(&normalized_name) {
            return Err(Error::ScriptRuntime(
                "InvalidCharacterError: attribute name is not a valid XML name".into(),
            ));
        }
        if normalized_name == "open"
            && self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
        {
            let _ = self.set_details_open_state_with_env(node, true, env)?;
        } else {
            self.dom
                .set_attr(node, &normalized_name, &value.as_string())?;
        }
        Ok(ExecFlow::Continue)
    }

    pub(super) fn execute_dom_remove_attribute_stmt(
        &mut self,
        target: &DomQuery,
        name: &str,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
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
        Ok(ExecFlow::Continue)
    }

    pub(super) fn execute_node_tree_mutation_stmt(
        &mut self,
        target: &DomQuery,
        method: &NodeTreeMethod,
        child: &Expr,
        reference: &Option<Expr>,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
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
            NodeTreeMethod::AppendChild => self.dom.append_child(target_node, child)?,
            NodeTreeMethod::Before => {
                let Some(parent) = self.dom.parent(target_node) else {
                    return Ok(ExecFlow::Continue);
                };
                self.dom.insert_before(parent, child, target_node)?;
            }
            NodeTreeMethod::ReplaceWith => {
                self.dom.replace_with(target_node, child)?;
            }
            NodeTreeMethod::Prepend => self.dom.prepend_child(target_node, child)?,
            NodeTreeMethod::RemoveChild => self.dom.remove_child(target_node, child)?,
            NodeTreeMethod::InsertBefore => {
                let Some(reference) = reference else {
                    return Err(Error::ScriptRuntime(
                        "insertBefore requires reference node".into(),
                    ));
                };
                let reference = self.eval_expr(reference, env, event_param, event)?;
                let Value::Node(reference) = reference else {
                    return Err(Error::ScriptRuntime(
                        "insertBefore reference must be an element reference".into(),
                    ));
                };
                self.dom.insert_before(target_node, child, reference)?;
            }
        }
        Ok(ExecFlow::Continue)
    }

    pub(super) fn execute_insert_adjacent_element_stmt(
        &mut self,
        target: &DomQuery,
        position: &InsertAdjacentPosition,
        node: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let target_node = self.resolve_dom_query_required_runtime(target, env)?;
        let node = self.eval_expr(node, env, event_param, event)?;
        let Value::Node(node) = node else {
            return Err(Error::ScriptRuntime(
                "TypeError: Failed to execute 'insertAdjacentElement': parameter 2 is not of type 'Element'".into(),
            ));
        };
        let node_is_fragment = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("#document-fragment"));
        if self.dom.element(node).is_none() || node_is_fragment {
            return Err(Error::ScriptRuntime(
                "TypeError: Failed to execute 'insertAdjacentElement': parameter 2 is not of type 'Element'".into(),
            ));
        }

        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(target_node) else {
                return Ok(ExecFlow::Continue);
            };
            let parent_is_fragment = self
                .dom
                .tag_name(parent)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("#document-fragment"));
            if self.dom.element(parent).is_none() || parent_is_fragment {
                return Ok(ExecFlow::Continue);
            }
        }

        let _ = self.dom.insert_adjacent_node(target_node, *position, node);
        Ok(ExecFlow::Continue)
    }

    pub(super) fn execute_insert_adjacent_text_stmt(
        &mut self,
        target: &DomQuery,
        position: &InsertAdjacentPosition,
        text: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let target_node = self.resolve_dom_query_required_runtime(target, env)?;
        let text = self.eval_expr(text, env, event_param, event)?;
        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(target_node) else {
                return Ok(ExecFlow::Continue);
            };
            let parent_is_fragment = self
                .dom
                .tag_name(parent)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("#document-fragment"));
            if self.dom.element(parent).is_none() || parent_is_fragment {
                return Ok(ExecFlow::Continue);
            }
        }
        let text_node = self.dom.create_detached_text(text.as_string());
        self.dom
            .insert_adjacent_node(target_node, *position, text_node)?;
        Ok(ExecFlow::Continue)
    }

    pub(super) fn execute_insert_adjacent_html_stmt(
        &mut self,
        target: &DomQuery,
        position: &Expr,
        html: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<ExecFlow> {
        let target_node = self.resolve_dom_query_required_runtime(target, env)?;
        let position = self.eval_expr(position, env, event_param, event)?;
        let position_text = position.as_string();
        let position = resolve_insert_adjacent_position(&position_text).map_err(|_| {
            Error::ScriptRuntime(format!(
                "SyntaxError: unsupported insertAdjacentHTML position: {position_text}"
            ))
        })?;
        if matches!(
            position,
            InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
        ) {
            let Some(parent) = self.dom.parent(target_node) else {
                return Err(Error::ScriptRuntime(
                    "NoModificationAllowedError: Failed to execute 'insertAdjacentHTML' because the target has no parent element".into(),
                ));
            };
            let parent_is_fragment = self
                .dom
                .tag_name(parent)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("#document-fragment"));
            if self.dom.element(parent).is_none() || parent_is_fragment {
                return Err(Error::ScriptRuntime(
                    "NoModificationAllowedError: Failed to execute 'insertAdjacentHTML' on a node whose parent is not an Element".into(),
                ));
            }
        }
        let html = self.eval_expr(html, env, event_param, event)?;
        match self
            .dom
            .insert_adjacent_html(target_node, position, &html.as_string())
        {
            Ok(()) => {}
            Err(Error::ScriptParse(message)) => {
                return Err(Error::ScriptRuntime(format!("SyntaxError: {message}")));
            }
            Err(other) => return Err(other),
        }
        Ok(ExecFlow::Continue)
    }

    pub(super) fn execute_node_remove_stmt(
        &mut self,
        target: &DomQuery,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        let node = self.resolve_dom_query_required_runtime(target, env)?;
        if let Some(active) = self.dom.active_element() {
            if active == node || self.dom.is_descendant_of(active, node) {
                self.dom.set_active_element(None);
            }
        }
        if let Some(active_pseudo) = self.dom.active_pseudo_element() {
            if active_pseudo == node || self.dom.is_descendant_of(active_pseudo, node) {
                self.dom.set_active_pseudo_element(None);
            }
        }
        self.dom.remove_node(node)?;
        Ok(ExecFlow::Continue)
    }
}
