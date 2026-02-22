use super::*;

impl Harness {
    pub fn assert_text(&self, selector: &str, expected: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.text_content(target);
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual,
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_value(&self, selector: &str, expected: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.value(target)?;
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual,
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_checked(&self, selector: &str, expected: bool) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.checked(target)?;
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual: actual.to_string(),
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_exists(&self, selector: &str) -> Result<()> {
        let _ = self.select_one(selector)?;
        Ok(())
    }

    pub fn dump_dom(&self, selector: &str) -> Result<String> {
        let target = self.select_one(selector)?;
        Ok(self.dom.dump_node(target))
    }

    pub(crate) fn select_one(&self, selector: &str) -> Result<NodeId> {
        self.dom
            .query_selector(selector)?
            .ok_or_else(|| Error::SelectorNotFound(selector.to_string()))
    }

    pub(crate) fn node_snippet(&self, node_id: NodeId) -> String {
        truncate_chars(&self.dom.dump_node(node_id), 200)
    }

    pub(crate) fn resolve_form_for_submit(&self, target: NodeId) -> Option<NodeId> {
        if self
            .dom
            .tag_name(target)
            .map(|t| t.eq_ignore_ascii_case("form"))
            .unwrap_or(false)
        {
            return Some(target);
        }
        if let Some(form_id) = self.dom.attr(target, "form") {
            let owner = self.dom.by_id(&form_id)?;
            if self
                .dom
                .tag_name(owner)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("form"))
            {
                return Some(owner);
            }
            return None;
        }
        self.dom.find_ancestor_by_tag(target, "form")
    }

    pub(crate) fn resolve_submit_form_target(&self, target: NodeId) -> Option<NodeId> {
        self.resolve_form_for_submit(target)
    }

    pub(crate) fn resolve_request_submitter_node(
        &self,
        submitter: Option<Value>,
    ) -> Result<Option<NodeId>> {
        match submitter {
            None | Some(Value::Undefined) | Some(Value::Null) => Ok(None),
            Some(Value::Node(node)) => Ok(Some(node)),
            Some(_) => Err(Error::ScriptRuntime(
                "requestSubmit submitter must be an element".into(),
            )),
        }
    }

    pub(crate) fn form_elements(&self, form: NodeId) -> Result<Vec<NodeId>> {
        let tag = self
            .dom
            .tag_name(form)
            .ok_or_else(|| Error::ScriptRuntime("elements target is not an element".into()))?;
        if !tag.eq_ignore_ascii_case("form") {
            return Err(Error::ScriptRuntime(format!(
                "{}.elements target is not a form",
                self.event_node_label(form)
            )));
        }

        let mut out = Vec::new();
        for node in self.dom.all_element_nodes() {
            if !is_form_control(&self.dom, node) {
                continue;
            }
            if self.resolve_form_for_submit(node) == Some(form) {
                out.push(node);
            }
        }
        Ok(out)
    }

    pub(crate) fn form_data_entries(&self, form: NodeId) -> Result<Vec<(String, String)>> {
        let mut out = Vec::new();
        for control in self.form_elements(form)? {
            if !self.is_successful_form_data_control(control)? {
                continue;
            }
            let name = self.dom.attr(control, "name").unwrap_or_default();
            let value = if self
                .dom
                .tag_name(control)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                && self
                    .dom
                    .attr(control, "type")
                    .unwrap_or_else(|| "text".to_string())
                    .eq_ignore_ascii_case("hidden")
                && name == "_charset_"
            {
                "UTF-8".to_string()
            } else {
                self.form_data_control_value(control)?
            };
            out.push((name, value));
        }
        Ok(out)
    }

    pub(crate) fn is_successful_form_data_control(&self, control: NodeId) -> Result<bool> {
        if self.is_effectively_disabled(control) {
            return Ok(false);
        }
        let name = self.dom.attr(control, "name").unwrap_or_default();
        if name.is_empty() {
            return Ok(false);
        }

        let tag = self
            .dom
            .tag_name(control)
            .ok_or_else(|| Error::ScriptRuntime("FormData target is not an element".into()))?;

        if tag.eq_ignore_ascii_case("button") {
            return Ok(false);
        }
        if tag.eq_ignore_ascii_case("output") {
            return Ok(false);
        }

        if tag.eq_ignore_ascii_case("input") {
            let kind = self
                .dom
                .attr(control, "type")
                .unwrap_or_default()
                .to_ascii_lowercase();
            if matches!(
                kind.as_str(),
                "button" | "submit" | "reset" | "file" | "image"
            ) {
                return Ok(false);
            }
            if kind == "checkbox" || kind == "radio" {
                return self.dom.checked(control);
            }
        }

        Ok(true)
    }

    pub(crate) fn form_data_control_value(&self, control: NodeId) -> Result<String> {
        self.dom.value(control)
    }

    pub(crate) fn form_is_valid_for_submit(&self, form: NodeId) -> Result<bool> {
        let controls = self.form_elements(form)?;
        for control in &controls {
            if !self.required_control_satisfied(*control, &controls)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(crate) fn required_control_satisfied(
        &self,
        control: NodeId,
        controls: &[NodeId],
    ) -> Result<bool> {
        if self.is_effectively_disabled(control) || !self.dom.required(control) {
            return Ok(true);
        }

        let tag = self
            .dom
            .tag_name(control)
            .ok_or_else(|| Error::ScriptRuntime("required target is not an element".into()))?;

        if tag.eq_ignore_ascii_case("input") {
            let kind = self
                .dom
                .attr(control, "type")
                .unwrap_or_else(|| "text".into())
                .to_ascii_lowercase();
            if !Self::input_supports_required(kind.as_str()) {
                return Ok(true);
            }
            if kind == "checkbox" {
                return self.dom.checked(control);
            }
            if kind == "radio" {
                if self.dom.checked(control)? {
                    return Ok(true);
                }
                let name = self.dom.attr(control, "name").unwrap_or_default();
                if name.is_empty() {
                    return Ok(false);
                }
                for candidate in controls {
                    if *candidate == control {
                        continue;
                    }
                    if !is_radio_input(&self.dom, *candidate) {
                        continue;
                    }
                    if self.dom.attr(*candidate, "name").unwrap_or_default() != name {
                        continue;
                    }
                    if self.dom.checked(*candidate)? {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            return Ok(!self.dom.value(control)?.is_empty());
        }

        if tag.eq_ignore_ascii_case("select") || tag.eq_ignore_ascii_case("textarea") {
            return Ok(!self.dom.value(control)?.is_empty());
        }

        Ok(true)
    }

    pub(crate) fn eval_form_data_source(
        &mut self,
        source: &FormDataSource,
        env: &HashMap<String, Value>,
    ) -> Result<Vec<(String, String)>> {
        match source {
            FormDataSource::NewForm(form) => {
                let form_node = self.resolve_dom_query_required_runtime(form, env)?;
                self.form_data_entries(form_node)
            }
            FormDataSource::Var(name) => match env.get(name) {
                Some(Value::FormData(entries)) => Ok(entries.clone()),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a FormData instance",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown FormData variable: {}",
                    name
                ))),
            },
        }
    }
}
