use super::*;

impl Harness {
    pub(crate) fn li_value_property(&self, node: NodeId) -> i64 {
        self.dom
            .attr(node, "value")
            .and_then(|raw| raw.trim().parse::<i64>().ok())
            .unwrap_or(0)
    }

    pub(crate) fn is_track_element(&self, node: NodeId) -> bool {
        self.dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("track"))
    }

    pub(crate) fn normalized_track_kind(&self, node: NodeId) -> String {
        let Some(raw) = self.dom.attr(node, "kind") else {
            return "subtitles".to_string();
        };
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "subtitles" | "captions" | "descriptions" | "chapters" | "metadata" => normalized,
            _ => "metadata".to_string(),
        }
    }

    pub(crate) fn parse_non_negative_int(raw: &str) -> Option<i64> {
        let value = raw.trim().parse::<i64>().ok()?;
        if value < 0 { None } else { Some(value) }
    }

    pub(crate) fn parse_positive_int(raw: &str) -> Option<i64> {
        let value = raw.trim().parse::<i64>().ok()?;
        if value <= 0 { None } else { Some(value) }
    }

    pub(crate) fn reflected_i64_attribute_or_default(
        &self,
        node: NodeId,
        attr_name: &str,
        default: i64,
    ) -> i64 {
        self.dom
            .attr(node, attr_name)
            .and_then(|raw| raw.trim().parse::<i64>().ok())
            .unwrap_or(default)
    }

    pub(crate) fn set_reflected_i64_attribute(
        &mut self,
        node: NodeId,
        attr_name: &str,
        value: &Value,
    ) -> Result<()> {
        self.dom
            .set_attr(node, attr_name, &Self::value_to_i64(value).to_string())
    }

    pub(crate) fn set_reflected_keyword_boolean_attribute(
        &mut self,
        node: NodeId,
        attr_name: &str,
        enabled: bool,
        true_keyword: &str,
        false_keyword: &str,
    ) -> Result<()> {
        self.dom.set_attr(
            node,
            attr_name,
            if enabled { true_keyword } else { false_keyword },
        )
    }

    pub(crate) fn set_reflected_boolean_attribute(
        &mut self,
        node: NodeId,
        attr_name: &str,
        enabled: bool,
    ) -> Result<()> {
        if enabled {
            self.dom.set_attr(node, attr_name, "true")
        } else {
            self.dom.remove_attr(node, attr_name)
        }
    }

    fn normalize_content_editable_keyword(raw: &str) -> Option<&'static str> {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("true") {
            Some("true")
        } else if trimmed.eq_ignore_ascii_case("false") {
            Some("false")
        } else if trimmed.eq_ignore_ascii_case("plaintext-only") {
            Some("plaintext-only")
        } else if trimmed.eq_ignore_ascii_case("inherit") {
            Some("inherit")
        } else {
            None
        }
    }

    pub(crate) fn content_editable_property_value_for_node(&self, node: NodeId) -> String {
        let Some(raw) = self.dom.attr(node, "contenteditable") else {
            return "inherit".to_string();
        };
        Self::normalize_content_editable_keyword(&raw)
            .unwrap_or("inherit")
            .to_string()
    }

    pub(crate) fn set_content_editable_property_value(
        &mut self,
        node: NodeId,
        value: &Value,
    ) -> Result<()> {
        let raw = value.as_string();
        let Some(normalized) = Self::normalize_content_editable_keyword(&raw) else {
            return Err(Error::ScriptRuntime(
                "SyntaxError: Failed to set 'contentEditable': The value provided is not one of 'true', 'false', 'plaintext-only', or 'inherit'"
                    .into(),
            ));
        };
        self.dom.set_attr(node, "contenteditable", normalized)
    }

    fn normalized_boolean_keyword_state(
        raw: Option<&str>,
        true_keywords: &[&str],
        false_keywords: &[&str],
        default_state: bool,
    ) -> bool {
        let Some(raw) = raw else {
            return default_state;
        };
        let normalized = raw.trim().to_ascii_lowercase();
        if true_keywords
            .iter()
            .any(|keyword| normalized == keyword.to_ascii_lowercase())
        {
            return true;
        }
        if false_keywords
            .iter()
            .any(|keyword| normalized == keyword.to_ascii_lowercase())
        {
            return false;
        }
        default_state
    }

    fn default_draggable_property_state_for_node(&self, node: NodeId) -> bool {
        self.dom.tag_name(node).is_some_and(|tag| {
            (tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area"))
                && self.dom.attr(node, "href").is_some()
                || tag.eq_ignore_ascii_case("img")
        })
    }

    pub(crate) fn draggable_property_value_for_node(&self, node: NodeId) -> bool {
        Self::normalized_boolean_keyword_state(
            self.dom.attr(node, "draggable").as_deref(),
            &["true"],
            &["false"],
            self.default_draggable_property_state_for_node(node),
        )
    }

    pub(crate) fn spellcheck_property_value_for_node(&self, node: NodeId) -> bool {
        let default = self.dom.tag_name(node).is_some_and(|tag| {
            tag.eq_ignore_ascii_case("textarea")
                || tag.eq_ignore_ascii_case("input")
                || self
                    .dom
                    .attr(node, "contenteditable")
                    .is_some_and(|value| !value.eq_ignore_ascii_case("false"))
        });
        Self::normalized_boolean_keyword_state(
            self.dom.attr(node, "spellcheck").as_deref(),
            &["", "true"],
            &["false"],
            default,
        )
    }

    pub(crate) fn translate_property_value_for_node(&self, node: NodeId) -> bool {
        Self::normalized_boolean_keyword_state(
            self.dom.attr(node, "translate").as_deref(),
            &["", "yes"],
            &["no"],
            true,
        )
    }

    pub(crate) fn reflected_url_attribute_or_empty(&self, node: NodeId, attr_name: &str) -> String {
        self.dom
            .attr(node, attr_name)
            .map(|raw| self.resolve_document_target_url(&raw))
            .unwrap_or_default()
    }

    pub(crate) fn submitter_form_action_property_value_for_node(&self, node: NodeId) -> String {
        if let Some(raw) = self.dom.attr(node, "formaction") {
            return self.resolve_document_target_url(&raw);
        }
        if let Some(form_owner) = self.resolve_form_for_submit(node) {
            return self.form_action_property_value_for_node(form_owner);
        }
        self.document_url.clone()
    }

    pub(crate) fn form_action_property_value_for_node(&self, node: NodeId) -> String {
        self.dom
            .attr(node, "action")
            .map(|raw| self.resolve_document_target_url(&raw))
            .unwrap_or_else(|| self.document_url.clone())
    }

    fn reflected_span_assignment_number(value: &Value, default: i64) -> i64 {
        match value {
            Value::Number(number) => *number,
            Value::Float(number) if number.is_finite() => *number as i64,
            Value::BigInt(number) => number.to_string().parse::<i64>().unwrap_or(default),
            other => other.as_string().trim().parse::<i64>().unwrap_or(default),
        }
    }

    pub(crate) fn col_span_value(&self, node: NodeId) -> i64 {
        self.dom
            .attr(node, "span")
            .and_then(|raw| Self::parse_positive_int(&raw))
            .map(|span| span.min(1000))
            .unwrap_or(1)
    }

    pub(crate) fn set_col_span_value(&mut self, node: NodeId, value: &Value) -> Result<()> {
        let next = Self::reflected_span_assignment_number(value, 1).clamp(1, 1000);
        self.dom.set_attr(node, "span", &next.to_string())
    }

    pub(crate) fn table_cell_col_span_value(&self, node: NodeId) -> i64 {
        self.dom
            .attr(node, "colspan")
            .and_then(|raw| Self::parse_positive_int(&raw))
            .map(|span| span.min(1000))
            .unwrap_or(1)
    }

    pub(crate) fn set_table_cell_col_span_value(
        &mut self,
        node: NodeId,
        value: &Value,
    ) -> Result<()> {
        let next = Self::reflected_span_assignment_number(value, 1).clamp(1, 1000);
        self.dom.set_attr(node, "colspan", &next.to_string())
    }

    pub(crate) fn table_cell_row_span_value(&self, node: NodeId) -> i64 {
        self.dom
            .attr(node, "rowspan")
            .and_then(|raw| Self::parse_non_negative_int(&raw))
            .map(|span| if span == 0 { 0 } else { span.clamp(1, 65534) })
            .unwrap_or(1)
    }

    pub(crate) fn set_table_cell_row_span_value(
        &mut self,
        node: NodeId,
        value: &Value,
    ) -> Result<()> {
        let next = Self::reflected_span_assignment_number(value, 1);
        let next = if next == 0 { 0 } else { next.clamp(1, 65534) };
        self.dom.set_attr(node, "rowspan", &next.to_string())
    }

    pub(crate) fn input_size_property_value_for_node(&self, node: NodeId) -> i64 {
        self.dom
            .attr(node, "size")
            .and_then(|raw| Self::parse_non_negative_int(&raw))
            .filter(|size| *size > 0)
            .unwrap_or(20)
    }

    pub(crate) fn set_input_size_property_value(
        &mut self,
        node: NodeId,
        value: &Value,
    ) -> Result<()> {
        let next = Self::value_to_i64(value).max(1);
        self.dom.set_attr(node, "size", &next.to_string())
    }

    pub(crate) fn textarea_rows_property_value_for_node(&self, node: NodeId) -> i64 {
        self.dom
            .attr(node, "rows")
            .and_then(|raw| Self::parse_non_negative_int(&raw))
            .filter(|rows| *rows > 0)
            .map(|rows| rows.min(2_147_483_647))
            .unwrap_or(2)
    }

    pub(crate) fn set_textarea_rows_property_value(
        &mut self,
        node: NodeId,
        value: &Value,
    ) -> Result<()> {
        let next = Self::value_to_i64(value).clamp(1, 2_147_483_647);
        self.dom.set_attr(node, "rows", &next.to_string())
    }

    pub(crate) fn textarea_cols_property_value_for_node(&self, node: NodeId) -> i64 {
        self.dom
            .attr(node, "cols")
            .and_then(|raw| Self::parse_non_negative_int(&raw))
            .filter(|cols| *cols > 0)
            .map(|cols| cols.min(2_147_483_647))
            .unwrap_or(20)
    }

    pub(crate) fn set_textarea_cols_property_value(
        &mut self,
        node: NodeId,
        value: &Value,
    ) -> Result<()> {
        let next = Self::value_to_i64(value).clamp(1, 2_147_483_647);
        self.dom.set_attr(node, "cols", &next.to_string())
    }

    pub(crate) fn min_length_property_value_for_node(&self, node: NodeId) -> i64 {
        self.dom
            .attr(node, "minlength")
            .and_then(|raw| Self::parse_non_negative_int(&raw))
            .unwrap_or(-1)
    }

    pub(crate) fn set_min_length_property_value(
        &mut self,
        node: NodeId,
        value: &Value,
    ) -> Result<()> {
        let next = Self::value_to_i64(value).max(-1);
        self.dom.set_attr(node, "minlength", &next.to_string())
    }

    pub(crate) fn max_length_property_value_for_node(&self, node: NodeId) -> i64 {
        self.dom
            .attr(node, "maxlength")
            .and_then(|raw| Self::parse_non_negative_int(&raw))
            .unwrap_or(-1)
    }

    pub(crate) fn set_max_length_property_value(
        &mut self,
        node: NodeId,
        value: &Value,
    ) -> Result<()> {
        let next = Self::value_to_i64(value).max(-1);
        self.dom.set_attr(node, "maxlength", &next.to_string())
    }
}
