use super::*;
use std::collections::HashSet;

impl Harness {
    pub(crate) fn resolved_dir_for_node(&self, node: NodeId) -> String {
        if let Some(explicit) = self.dom.attr(node, "dir") {
            return explicit;
        }
        if self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("bdi"))
        {
            return "auto".to_string();
        }
        String::new()
    }

    fn resolved_static_role_for_tag(tag: &str) -> Option<&'static str> {
        match tag {
            "address" => Some("group"),
            "aside" => Some("complementary"),
            "article" => Some("article"),
            "blockquote" => Some("blockquote"),
            "body" | "b" | "bdi" | "bdo" | "data" | "div" | "i" | "pre" | "q" | "samp"
            | "small" | "u" => Some("generic"),
            "button" => Some("button"),
            "caption" => Some("caption"),
            "code" => Some("code"),
            "datalist" => Some("listbox"),
            "details" | "fieldset" | "hgroup" | "optgroup" => Some("group"),
            "dialog" => Some("dialog"),
            "del" | "s" => Some("deletion"),
            "dfn" => Some("term"),
            "em" => Some("emphasis"),
            "figure" => Some("figure"),
            "form" => Some("form"),
            "hr" => Some("separator"),
            "html" => Some("document"),
            "ins" => Some("insertion"),
            "main" => Some("main"),
            "ol" | "menu" | "ul" => Some("list"),
            "meter" => Some("meter"),
            "nav" => Some("navigation"),
            "option" => Some("option"),
            "output" => Some("status"),
            "p" => Some("paragraph"),
            "progress" => Some("progressbar"),
            "strong" => Some("strong"),
            "sub" => Some("subscript"),
            "sup" => Some("superscript"),
            "table" => Some("table"),
            "tbody" | "tfoot" | "thead" => Some("rowgroup"),
            "tr" => Some("row"),
            "textarea" => Some("textbox"),
            "time" => Some("time"),
            "search" => Some("search"),
            _ => None,
        }
    }

    fn is_heading_tag(tag: &str) -> bool {
        matches!(tag.as_bytes(), [b'h' | b'H', b'1'..=b'6'])
    }

    pub(crate) fn resolved_role_for_node(&self, node: NodeId) -> String {
        if let Some(explicit) = self.dom.attr(node, "role") {
            return explicit;
        }
        let Some(tag) = self.dom.tag_name(node) else {
            return String::new();
        };
        let normalized_tag = tag.to_ascii_lowercase();

        match normalized_tag.as_str() {
            "header" => return self.resolved_header_role(node),
            "input" => return self.resolved_input_role(node),
            "footer" => return self.resolved_footer_role(node),
            "img" => {
                if self.dom.attr(node, "alt").is_some_and(|alt| alt.is_empty()) {
                    return "presentation".to_string();
                }
                return "img".to_string();
            }
            "li" => return self.resolved_list_item_role(node),
            "th" => return self.resolved_table_header_role(node),
            "td" => return self.resolved_table_data_cell_role(node),
            "select" => return self.resolved_select_role(node),
            "section" => return self.resolved_section_role(node),
            "a" | "area" | "link" if self.dom.attr(node, "href").is_some() => {
                return "link".to_string();
            }
            _ => {}
        }

        if Self::is_heading_tag(normalized_tag.as_str()) {
            return "heading".to_string();
        }

        Self::resolved_static_role_for_tag(normalized_tag.as_str())
            .unwrap_or_default()
            .to_string()
    }

    pub(crate) fn footer_has_scoped_ancestor(&self, node: NodeId) -> bool {
        let mut cursor = self.dom.parent(node);
        while let Some(parent) = cursor {
            if self.dom.tag_name(parent).is_some_and(|tag| {
                tag.eq_ignore_ascii_case("article")
                    || tag.eq_ignore_ascii_case("aside")
                    || tag.eq_ignore_ascii_case("main")
                    || tag.eq_ignore_ascii_case("nav")
                    || tag.eq_ignore_ascii_case("section")
            }) {
                return true;
            }

            if self.dom.attr(parent, "role").is_some_and(|role| {
                let normalized = role.trim().to_ascii_lowercase();
                matches!(
                    normalized.as_str(),
                    "article" | "complementary" | "main" | "navigation" | "region"
                )
            }) {
                return true;
            }

            cursor = self.dom.parent(parent);
        }
        false
    }

    pub(crate) fn resolved_footer_role(&self, node: NodeId) -> String {
        if self.footer_has_scoped_ancestor(node) {
            "generic".to_string()
        } else {
            "contentinfo".to_string()
        }
    }

    pub(crate) fn resolved_header_role(&self, node: NodeId) -> String {
        if self.footer_has_scoped_ancestor(node) {
            "generic".to_string()
        } else {
            "banner".to_string()
        }
    }

    pub(crate) fn has_accessible_name_for_landmark(&self, node: NodeId) -> bool {
        if self
            .dom
            .attr(node, "aria-label")
            .is_some_and(|value| !value.trim().is_empty())
        {
            return true;
        }

        let Some(raw_ids) = self.dom.attr(node, "aria-labelledby") else {
            return false;
        };

        raw_ids.split_whitespace().any(|id_ref| {
            self.dom
                .by_id(id_ref)
                .is_some_and(|label_node| !self.dom.text_content(label_node).trim().is_empty())
        })
    }

    pub(crate) fn resolved_section_role(&self, node: NodeId) -> String {
        if self.has_accessible_name_for_landmark(node) {
            "region".to_string()
        } else {
            "generic".to_string()
        }
    }

    pub(crate) fn resolved_input_role(&self, node: NodeId) -> String {
        let input_type = self
            .dom
            .attr(node, "type")
            .unwrap_or_else(|| "text".to_string())
            .trim()
            .to_ascii_lowercase();
        let has_list = self.dom.attr(node, "list").is_some();

        match input_type.as_str() {
            "button" | "image" | "reset" | "submit" => "button".to_string(),
            "checkbox" => "checkbox".to_string(),
            "number" => "spinbutton".to_string(),
            "radio" => "radio".to_string(),
            "range" => "slider".to_string(),
            "search" => {
                if has_list {
                    "combobox".to_string()
                } else {
                    "searchbox".to_string()
                }
            }
            "color" | "date" | "datetime-local" | "file" | "hidden" | "month" | "password"
            | "time" | "week" => String::new(),
            "email" | "tel" | "text" | "url" => {
                if has_list {
                    "combobox".to_string()
                } else {
                    "textbox".to_string()
                }
            }
            _ => {
                if has_list {
                    "combobox".to_string()
                } else {
                    "textbox".to_string()
                }
            }
        }
    }

    pub(crate) fn resolved_list_item_role(&self, node: NodeId) -> String {
        let Some(parent) = self.dom.parent(node) else {
            return String::new();
        };
        if self.dom.tag_name(parent).is_some_and(|tag| {
            tag.eq_ignore_ascii_case("ol")
                || tag.eq_ignore_ascii_case("ul")
                || tag.eq_ignore_ascii_case("menu")
        }) {
            "listitem".to_string()
        } else {
            String::new()
        }
    }

    pub(crate) fn resolved_select_role(&self, node: NodeId) -> String {
        let multiple = self.dom.attr(node, "multiple").is_some();
        let size_is_listbox = self
            .dom
            .attr(node, "size")
            .and_then(|raw| Self::parse_non_negative_int(&raw))
            .is_some_and(|size| size > 1);
        if !multiple && !size_is_listbox {
            "combobox".to_string()
        } else {
            "listbox".to_string()
        }
    }

    pub(crate) fn resolved_table_data_cell_role(&self, node: NodeId) -> String {
        let mut cursor = self.dom.parent(node);
        let mut has_table_ancestor = false;

        while let Some(parent) = cursor {
            if self
                .dom
                .attr(parent, "role")
                .is_some_and(|role| role.trim().eq_ignore_ascii_case("grid"))
            {
                return "gridcell".to_string();
            }

            if self
                .dom
                .tag_name(parent)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("table"))
            {
                has_table_ancestor = true;
            }

            cursor = self.dom.parent(parent);
        }

        if has_table_ancestor {
            "cell".to_string()
        } else {
            String::new()
        }
    }

    pub(crate) fn resolved_table_header_role(&self, node: NodeId) -> String {
        if let Some(scope) = self.dom.attr(node, "scope") {
            let scope = scope.trim().to_ascii_lowercase();
            if matches!(scope.as_str(), "row" | "rowgroup") {
                return "rowheader".to_string();
            }
            if matches!(scope.as_str(), "col" | "colgroup") {
                return "columnheader".to_string();
            }
        }

        let Some(parent) = self.dom.parent(node) else {
            return "columnheader".to_string();
        };
        if !self
            .dom
            .tag_name(parent)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("tr"))
        {
            return "columnheader".to_string();
        }

        let has_data_cell_sibling = self.dom.nodes[parent.0].children.iter().any(|child| {
            self.dom
                .tag_name(*child)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("td"))
        });

        if has_data_cell_sibling {
            "rowheader".to_string()
        } else {
            "columnheader".to_string()
        }
    }

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

    pub(crate) fn is_canvas_2d_context_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_CANVAS_2D_CONTEXT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_event_target_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_EVENT_TARGET_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_match_media_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_MATCH_MEDIA_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_hash_change_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_HASH_CHANGE_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_error_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_ERROR_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_before_unload_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_BEFORE_UNLOAD_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_keyboard_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_KEYBOARD_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_wheel_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_WHEEL_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_navigate_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_NAVIGATE_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_pointer_event_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_POINTER_EVENT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_attr_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_ATTR_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_named_node_map_object(entries: &(impl ObjectEntryLookup + ?Sized)) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_NAMED_NODE_MAP_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn named_node_map_owner_node(
        entries: &(impl ObjectEntryLookup + ?Sized),
    ) -> Option<NodeId> {
        match Self::object_get_entry(entries, INTERNAL_NAMED_NODE_MAP_OWNER_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }
    }

    pub(crate) fn named_node_map_entries(&self, owner: NodeId) -> Vec<(String, String)> {
        let Some(element) = self.dom.element(owner) else {
            return Vec::new();
        };
        let mut attrs = element
            .attrs
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<Vec<_>>();
        attrs.sort_by(|(left, _), (right, _)| left.cmp(right));
        attrs
    }

    fn is_named_node_map_builtin_property_name(key: &str) -> bool {
        matches!(
            key,
            "length"
                | "item"
                | "getNamedItem"
                | "setNamedItem"
                | "removeNamedItem"
                | "getNamedItemNS"
                | "setNamedItemNS"
                | "removeNamedItemNS"
                | "forEach"
                | "keys"
                | "values"
                | "entries"
        )
    }

    pub(crate) fn named_node_map_named_property_is_visible(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> bool {
        if Self::is_named_node_map_builtin_property_name(key) {
            return false;
        }

        let mut prototype = Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
            .or_else(|| Some(self.object_constructor_prototype_value()));
        while let Some(current) = prototype {
            match current {
                Value::Null | Value::Undefined => break,
                Value::Object(object) => {
                    let object_value = Value::Object(object.clone());
                    if self
                        .object_has_own_value(&object_value, key)
                        .is_ok_and(|value| value.truthy())
                    {
                        return false;
                    }
                    prototype = self.value_internal_prototype_value(&object_value);
                }
                _ => break,
            }
        }
        true
    }

    fn is_html_collection_builtin_property_name(key: &str) -> bool {
        matches!(
            key,
            "length" | "item" | "namedItem" | "forEach" | "keys" | "values" | "entries"
        )
    }

    pub(crate) fn node_list_is_html_collection(nodes: &Rc<RefCell<NodeListValue>>) -> bool {
        nodes.borrow().kind.is_html_collection_family()
    }

    pub(crate) fn node_list_is_radio_node_list(nodes: &Rc<RefCell<NodeListValue>>) -> bool {
        matches!(nodes.borrow().kind, NodeListKind::RadioNodeList)
    }

    fn node_list_display_name(nodes: &Rc<RefCell<NodeListValue>>) -> &'static str {
        nodes.borrow().kind.display_name()
    }

    pub(crate) fn node_list_item_value(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        node: NodeId,
    ) -> Value {
        if matches!(nodes.borrow().kind, NodeListKind::TextTrackList) {
            return self.text_track_object_value(node);
        }
        Value::Node(node)
    }

    pub(crate) fn html_collection_named_entries(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
    ) -> Vec<(String, NodeId)> {
        if !Self::node_list_is_html_collection(nodes) {
            return Vec::new();
        }

        let mut supported = Vec::new();
        let mut seen = HashSet::new();
        for node in self.node_list_snapshot(nodes) {
            let Some(_) = self.dom.element(node) else {
                continue;
            };
            for candidate in [self.dom.attr(node, "id"), self.dom.attr(node, "name")] {
                let Some(candidate) = candidate.filter(|candidate| !candidate.is_empty()) else {
                    continue;
                };
                if seen.insert(candidate.clone()) {
                    supported.push((candidate, node));
                }
            }
        }
        supported
    }

    pub(crate) fn html_collection_named_property_is_visible(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        key: &str,
    ) -> bool {
        if !Self::node_list_is_html_collection(nodes)
            || key.is_empty()
            || Self::own_property_integer_key(key).is_some()
            || Self::is_html_collection_builtin_property_name(key)
        {
            return false;
        }

        let collection = Value::NodeList(nodes.clone());
        let mut prototype = self.value_internal_prototype_value(&collection);
        while let Some(current) = prototype {
            match current {
                Value::Null | Value::Undefined => break,
                _ => {
                    if self
                        .object_has_own_value(&current, key)
                        .is_ok_and(|value| value.truthy())
                    {
                        return false;
                    }
                    prototype = self.value_internal_prototype_value(&current);
                }
            }
        }
        true
    }

    pub(crate) fn html_collection_named_property_value(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        key: &str,
    ) -> Option<Value> {
        if !self.html_collection_named_property_is_visible(nodes, key) {
            return None;
        }
        let owner_form = {
            let nodes_ref = nodes.borrow();
            match nodes_ref.live_source {
                Some(LiveNodeListSource::FormElements { form }) => Some(form),
                _ => None,
            }
        };
        if let Some(form) = owner_form {
            return self
                .form_controls_named_item_value(form, key)
                .ok()
                .flatten();
        }
        self.html_collection_named_entries(nodes)
            .into_iter()
            .find(|(name, _)| name == key)
            .map(|(_, node)| Value::Node(node))
    }

    pub(crate) fn form_controls_named_item_value(
        &mut self,
        form: NodeId,
        key: &str,
    ) -> Result<Option<Value>> {
        let matches = self.form_controls_named_matches(form, key)?;
        Ok(match matches.len() {
            0 => None,
            1 => Some(Value::Node(matches[0])),
            _ => Some(self.form_named_group_live_list_value(form, key)?),
        })
    }

    pub(crate) fn is_html_form_hidden_named_property_name(key: &str) -> bool {
        matches!(
            key,
            "elements"
                | "length"
                | "name"
                | "action"
                | "submit"
                | "requestSubmit"
                | "reset"
                | "checkValidity"
                | "reportValidity"
                | "method"
                | "enctype"
                | "encoding"
                | "target"
                | "noValidate"
                | "acceptCharset"
                | "rel"
        )
    }

    pub(crate) fn form_named_property_value(
        &mut self,
        form: NodeId,
        key: &str,
    ) -> Result<Option<Value>> {
        if key.is_empty() || Self::is_html_form_hidden_named_property_name(key) {
            return Ok(None);
        }
        self.form_controls_named_item_value(form, key)
    }

    pub(crate) fn html_form_builtin_own_string_keys() -> [&'static str; 7] {
        [
            "elements",
            "length",
            "submit",
            "requestSubmit",
            "reset",
            "checkValidity",
            "reportValidity",
        ]
    }

    fn node_cached_receiver_builtin_callable(
        &mut self,
        node: NodeId,
        cache_key: &str,
        family: &str,
        member: &str,
    ) -> Value {
        if let Some(value) = self
            .dom_runtime
            .node_expando_props
            .get(&(node, cache_key.to_string()))
            .cloned()
        {
            return value;
        }
        let value = Self::new_receiver_builtin_callable(family, member);
        self.dom_runtime
            .node_expando_props
            .insert((node, cache_key.to_string()), value.clone());
        value
    }

    pub(crate) fn form_builtin_property_value(&self, key: &str) -> Option<Value> {
        match key {
            "submit" | "requestSubmit" | "reset" | "checkValidity" | "reportValidity" => {
                Some(Self::new_receiver_builtin_callable("html_form", key))
            }
            _ => None,
        }
    }

    pub(crate) fn html_media_builtin_own_string_keys() -> [&'static str; 5] {
        ["play", "pause", "load", "canPlayType", "fastSeek"]
    }

    pub(crate) fn html_media_builtin_property_value(
        &mut self,
        media: NodeId,
        key: &str,
    ) -> Result<Option<Value>> {
        match key {
            "play" => Ok(Some(self.node_cached_receiver_builtin_callable(
                media,
                INTERNAL_MEDIA_PLAY_CALLABLE_KEY,
                "html_media",
                "play",
            ))),
            "pause" => Ok(Some(self.node_cached_receiver_builtin_callable(
                media,
                INTERNAL_MEDIA_PAUSE_CALLABLE_KEY,
                "html_media",
                "pause",
            ))),
            "load" => Ok(Some(self.node_cached_receiver_builtin_callable(
                media,
                INTERNAL_MEDIA_LOAD_CALLABLE_KEY,
                "html_media",
                "load",
            ))),
            "canPlayType" => Ok(Some(self.node_cached_receiver_builtin_callable(
                media,
                INTERNAL_MEDIA_CAN_PLAY_TYPE_CALLABLE_KEY,
                "html_media",
                "canPlayType",
            ))),
            "fastSeek" => Ok(Some(self.node_cached_receiver_builtin_callable(
                media,
                INTERNAL_MEDIA_FAST_SEEK_CALLABLE_KEY,
                "html_media",
                "fastSeek",
            ))),
            _ => Ok(None),
        }
    }

    pub(crate) fn html_form_builtin_property_value(
        &mut self,
        form: NodeId,
        key: &str,
    ) -> Result<Option<Value>> {
        match key {
            "elements" => self.form_elements_live_list_value(form).map(Some),
            "length" => Ok(Some(Value::Number(self.form_elements(form)?.len() as i64))),
            _ => Ok(self.form_builtin_property_value(key)),
        }
    }

    pub(crate) fn html_form_named_property_keys(&mut self, form: NodeId) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for control in self.form_elements(form)? {
            let id = self.dom.attr(control, "id").unwrap_or_default();
            if !id.is_empty()
                && !Self::is_html_form_hidden_named_property_name(&id)
                && seen.insert(id.clone())
            {
                out.push(id);
            }

            let name = self.dom.attr(control, "name").unwrap_or_default();
            if !name.is_empty()
                && !Self::is_html_form_hidden_named_property_name(&name)
                && seen.insert(name.clone())
            {
                out.push(name);
            }
        }
        Ok(out)
    }

    pub(crate) fn node_explicit_own_property_overrides_dom_property(
        &self,
        node: NodeId,
        key: &str,
    ) -> bool {
        if !self.node_has_explicit_own_property(node, key) {
            return false;
        }
        if self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("form"))
        {
            return true;
        }
        matches!(
            key,
            "id" | "className"
                | "lang"
                | "dir"
                | "accessKey"
                | "accesskey"
                | "autocapitalize"
                | "autocorrect"
                | "contentEditable"
                | "contenteditable"
                | "draggable"
                | "enterKeyHint"
                | "enterkeyhint"
                | "hidden"
                | "inert"
                | "inputMode"
                | "inputmode"
                | "nonce"
                | "popover"
                | "spellcheck"
                | "tabIndex"
                | "tabindex"
                | "title"
                | "translate"
                | "append"
                | "prepend"
                | "replaceChildren"
                | "before"
                | "after"
                | "replaceWith"
                | "remove"
                | "appendChild"
                | "insertBefore"
                | "removeChild"
                | "replaceChild"
                | "hasChildNodes"
                | "contains"
                | "getRootNode"
                | "compareDocumentPosition"
                | "isEqualNode"
                | "isSameNode"
                | "normalize"
                | "isDefaultNamespace"
                | "lookupPrefix"
                | "lookupNamespaceURI"
                | "cloneNode"
                | "querySelector"
                | "querySelectorAll"
                | "getAttributeNames"
                | "toggleAttribute"
                | "matches"
                | "closest"
                | "insertAdjacentElement"
                | "insertAdjacentHTML"
                | "insertAdjacentText"
                | "setHTMLUnsafe"
                | "controlsList"
                | "controlslist"
                | "crossOrigin"
                | "crossorigin"
                | "disableRemotePlayback"
                | "disableremoteplayback"
                | "disablePictureInPicture"
                | "disablepictureinpicture"
                | "playsInline"
                | "playsinline"
                | "clientWidth"
                | "clientHeight"
                | "clientLeft"
                | "clientTop"
                | "currentCSSZoom"
                | "offsetWidth"
                | "offsetHeight"
                | "offsetLeft"
                | "offsetTop"
                | "scrollWidth"
                | "scrollHeight"
                | "scrollLeft"
                | "scrollTop"
                | "scrollLeftMax"
                | "scrollTopMax"
                | "paused"
                | "ended"
                | "seeking"
                | "networkState"
                | "readyState"
                | "defaultMuted"
                | "currentTime"
                | "volume"
                | "duration"
                | "playbackRate"
                | "defaultPlaybackRate"
                | "play"
                | "pause"
                | "load"
                | "canPlayType"
                | "fastSeek"
                | "textTracks"
                | "buffered"
                | "seekable"
                | "played"
                | "value"
                | "open"
                | "closedBy"
                | "closedby"
                | "htmlFor"
                | "slot"
                | "role"
                | "elementTiming"
                | "elementtiming"
                | "name"
                | "cite"
                | "dateTime"
                | "datetime"
                | "clear"
                | "align"
                | "href"
                | "src"
                | "currentSrc"
                | "currentsrc"
                | "autoplay"
                | "controls"
                | "loop"
                | "muted"
                | "alt"
                | "download"
                | "hreflang"
                | "ping"
                | "referrerPolicy"
                | "referrerpolicy"
                | "rel"
                | "target"
                | "noHref"
                | "nohref"
                | "charset"
                | "coords"
                | "rev"
                | "shape"
                | "media"
                | "type"
                | "kind"
                | "label"
                | "srclang"
                | "srcLang"
                | "track"
                | "default"
                | "poster"
                | "preload"
                | "formAction"
                | "attributionSrc"
                | "attributionsrc"
                | "sizes"
                | "srcset"
                | "srcSet"
                | "data"
                | "srcdoc"
                | "srcDoc"
                | "useMap"
                | "usemap"
        )
    }

    pub(crate) fn node_explicit_own_dom_property_shadow_key<'a>(
        &self,
        node: NodeId,
        keys: &[&'a str],
    ) -> Option<&'a str> {
        keys.iter()
            .copied()
            .find(|key| self.node_explicit_own_property_overrides_dom_property(node, key))
    }

    pub(crate) fn node_explicit_own_dom_property_shadow_value(
        &mut self,
        node: NodeId,
        keys: &[&str],
    ) -> Result<Option<Value>> {
        let Some(_) = self.node_explicit_own_dom_property_shadow_key(node, keys) else {
            return Ok(None);
        };
        let entries = self.node_expando_entries(node);
        for key in keys {
            if let Some(value) =
                self.object_property_from_entries_with_getter(&Value::Node(node), &entries, key)?
            {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    fn media_numeric_state_value(&self, node: NodeId, key: &str, default: f64) -> Value {
        self.dom_runtime
            .node_expando_props
            .get(&(node, key.to_string()))
            .cloned()
            .unwrap_or_else(|| Self::number_value(default))
    }

    fn media_boolean_state_value(&self, node: NodeId, key: &str, default: bool) -> Value {
        match self
            .dom_runtime
            .node_expando_props
            .get(&(node, key.to_string()))
        {
            Some(Value::Bool(value)) => Value::Bool(*value),
            Some(value) => Value::Bool(value.truthy()),
            None => Value::Bool(default),
        }
    }

    fn media_numeric_state_number(&self, node: NodeId, key: &str, default: f64) -> f64 {
        match self
            .dom_runtime
            .node_expando_props
            .get(&(node, key.to_string()))
        {
            Some(Value::Number(value)) => *value as f64,
            Some(Value::Float(value)) => *value,
            Some(value) => Self::coerce_number_for_number_constructor(value),
            None => default,
        }
    }

    pub(crate) fn set_media_numeric_state_value(&mut self, node: NodeId, key: &str, value: &Value) {
        let next = Self::coerce_number_for_number_constructor(value);
        self.dom_runtime
            .node_expando_props
            .insert((node, key.to_string()), Self::number_value(next));
    }

    pub(crate) fn set_media_boolean_state_value(&mut self, node: NodeId, key: &str, next: bool) {
        self.dom_runtime
            .node_expando_props
            .insert((node, key.to_string()), Value::Bool(next));
    }

    pub(crate) fn media_time_ranges_snapshot(&self, media: NodeId, kind: &str) -> Vec<(f64, f64)> {
        let has_src = !self.resolve_media_src(media).is_empty();
        if !has_src {
            return Vec::new();
        }

        let current_time = self
            .media_numeric_state_number(media, INTERNAL_MEDIA_CURRENT_TIME_KEY, 0.0)
            .max(0.0);
        match kind {
            "buffered" | "seekable" => vec![(0.0, current_time)],
            "played" if current_time > 0.0 => vec![(0.0, current_time)],
            _ => Vec::new(),
        }
    }

    fn image_has_resolved_source(&self, node: NodeId) -> bool {
        !self.resolve_media_src(node).is_empty()
    }

    fn image_natural_dimension_value(&self, node: NodeId) -> i64 {
        if self.image_has_resolved_source(node) {
            1
        } else {
            0
        }
    }

    fn radio_node_list_value_string_from_nodes(&self, nodes: &[NodeId]) -> Result<String> {
        for node in nodes {
            if is_radio_input(&self.dom, *node) && self.dom.checked(*node)? {
                return Ok(self.dom.value(*node)?);
            }
        }
        Ok(String::new())
    }

    pub(crate) fn radio_node_list_value_string(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
    ) -> Result<String> {
        let snapshot = self.node_list_snapshot(nodes);
        self.radio_node_list_value_string_from_nodes(&snapshot)
    }

    pub(crate) fn set_radio_node_list_value(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        next_value: &str,
    ) -> Result<()> {
        let snapshot = self.node_list_snapshot(nodes);
        for node in snapshot {
            if is_radio_input(&self.dom, node) && self.dom.value(node)? == next_value {
                self.dom.set_checked(node, true)?;
                break;
            }
        }
        Ok(())
    }

    pub(crate) fn is_range_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_RANGE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_selection_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_SELECTION_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_clipboard_data_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_CLIPBOARD_DATA_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_data_transfer_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_DATA_TRANSFER_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_data_transfer_item_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_DATA_TRANSFER_ITEM_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_data_transfer_item_list_value(array: &ArrayValue) -> bool {
        matches!(
            Self::object_get_entry(
                &array.properties,
                INTERNAL_DATA_TRANSFER_ITEM_LIST_OBJECT_KEY
            ),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_dom_rect_list_value(array: &ArrayValue) -> bool {
        matches!(
            Self::object_get_entry(&array.properties, INTERNAL_DOM_RECT_LIST_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_clipboard_item_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_CLIPBOARD_ITEM_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_mock_file_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_MOCK_FILE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn form_data_append_string_value(value: &Value, filename: Option<&Value>) -> String {
        match value {
            Value::Blob(_) => filename
                .map(Value::as_string)
                .unwrap_or_else(|| "blob".to_string()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_mock_file_object(&entries) {
                    return filename
                        .map(Value::as_string)
                        .or_else(|| Self::object_get_entry(&entries, "name").map(|v| v.as_string()))
                        .unwrap_or_else(|| "blob".to_string());
                }
                value.as_string()
            }
            _ => value.as_string(),
        }
    }

    pub(crate) fn is_class_list_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_CLASS_LIST_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_time_ranges_object(entries: &(impl ObjectEntryLookup + ?Sized)) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_TIME_RANGES_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_image_bitmap_object(entries: &(impl ObjectEntryLookup + ?Sized)) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_IMAGE_BITMAP_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_text_track_object(entries: &(impl ObjectEntryLookup + ?Sized)) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_TEXT_TRACK_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn text_track_owner_node(
        entries: &(impl ObjectEntryLookup + ?Sized),
    ) -> Option<NodeId> {
        match Self::object_get_entry(entries, INTERNAL_TEXT_TRACK_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }
    }

    pub(crate) fn time_ranges_owner_node(
        entries: &(impl ObjectEntryLookup + ?Sized),
    ) -> Option<NodeId> {
        match Self::object_get_entry(entries, INTERNAL_TIME_RANGES_MEDIA_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }
    }

    pub(crate) fn time_ranges_kind(entries: &(impl ObjectEntryLookup + ?Sized)) -> Option<String> {
        match Self::object_get_entry(entries, INTERNAL_TIME_RANGES_KIND_KEY) {
            Some(Value::String(kind)) => Some(kind),
            _ => None,
        }
    }

    pub(crate) fn is_dom_string_map_object(entries: &(impl ObjectEntryLookup + ?Sized)) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_DOM_STRING_MAP_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn dom_string_map_owner_node(
        entries: &(impl ObjectEntryLookup + ?Sized),
    ) -> Option<NodeId> {
        match Self::object_get_entry(entries, INTERNAL_DOM_STRING_MAP_OWNER_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }
    }

    pub(crate) fn keyboard_key_code_for_key(key: &str) -> i64 {
        if let Some(ch) = key.chars().next().filter(|_| key.chars().count() == 1) {
            return ch as i64;
        }
        match key {
            "Backspace" => 8,
            "Tab" => 9,
            "Enter" => 13,
            "Shift" => 16,
            "Control" => 17,
            "Alt" => 18,
            "Pause" => 19,
            "CapsLock" => 20,
            "Escape" => 27,
            " " => 32,
            "PageUp" => 33,
            "PageDown" => 34,
            "End" => 35,
            "Home" => 36,
            "ArrowLeft" => 37,
            "ArrowUp" => 38,
            "ArrowRight" => 39,
            "ArrowDown" => 40,
            "Insert" => 45,
            "Delete" => 46,
            "Meta" => 91,
            "ContextMenu" => 93,
            "NumLock" => 144,
            "ScrollLock" => 145,
            "F1" => 112,
            "F2" => 113,
            "F3" => 114,
            "F4" => 115,
            "F5" => 116,
            "F6" => 117,
            "F7" => 118,
            "F8" => 119,
            "F9" => 120,
            "F10" => 121,
            "F11" => 122,
            "F12" => 123,
            _ => 0,
        }
    }

    pub(crate) fn keyboard_char_code_for_event(event_type: &str, key: &str) -> i64 {
        if !event_type.eq_ignore_ascii_case("keypress") {
            return 0;
        }
        if let Some(ch) = key.chars().next().filter(|_| key.chars().count() == 1) {
            return ch as i64;
        }
        if key == "Enter" { 13 } else { 0 }
    }

    pub(crate) fn event_modifier_state_from_entries(
        entries: &(impl ObjectEntryLookup + ?Sized),
        modifier: &str,
    ) -> bool {
        let normalized = modifier.trim();
        match normalized {
            "Alt" | "alt" => {
                Self::object_get_entry(entries, "altKey").is_some_and(|value| value.truthy())
            }
            "Control" | "control" | "Ctrl" | "ctrl" => {
                Self::object_get_entry(entries, "ctrlKey").is_some_and(|value| value.truthy())
            }
            "Meta" | "meta" => {
                Self::object_get_entry(entries, "metaKey").is_some_and(|value| value.truthy())
            }
            "Shift" | "shift" => {
                Self::object_get_entry(entries, "shiftKey").is_some_and(|value| value.truthy())
            }
            "AltGraph" | "altgraph" => {
                Self::object_get_entry(entries, "altKey").is_some_and(|value| value.truthy())
                    && Self::object_get_entry(entries, "ctrlKey")
                        .is_some_and(|value| value.truthy())
            }
            _ => false,
        }
    }

    pub(crate) fn new_attr_object_value(name: &str, value: &str, owner: Option<NodeId>) -> Value {
        Self::new_object_value(vec![
            (INTERNAL_ATTR_OBJECT_KEY.to_string(), Value::Bool(true)),
            ("name".to_string(), Value::String(name.to_string())),
            ("value".to_string(), Value::String(value.to_string())),
            (
                "ownerElement".to_string(),
                owner.map(Value::Node).unwrap_or(Value::Null),
            ),
        ])
    }

    pub(crate) fn new_clipboard_data_object_value(&mut self, text: &str) -> Value {
        let mut store = ObjectValue::default();
        let types = if text.is_empty() {
            Vec::new()
        } else {
            store.set_entry("text/plain".to_string(), Value::String(text.to_string()));
            vec![Value::String("text/plain".to_string())]
        };
        let store = Value::Object(Rc::new(RefCell::new(store)));
        let types_array = Self::new_array_value(types);
        let mut entries = vec![
            (
                INTERNAL_CLIPBOARD_DATA_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_CLIPBOARD_DATA_TEXT_KEY.to_string(),
                Value::String(text.to_string()),
            ),
            (INTERNAL_CLIPBOARD_DATA_STORE_KEY.to_string(), store),
            (
                "getData".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "setData".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "clearData".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                INTERNAL_CLIPBOARD_DATA_TYPES_KEY.to_string(),
                types_array.clone(),
            ),
            ("types".to_string(), types_array),
        ];
        Self::mark_object_properties_non_enumerable(
            &mut entries,
            &["getData", "setData", "clearData"],
        );
        let value = Self::new_object_value(entries);
        if let Value::Object(object) = &value {
            let prototype = self
                .constructor_prototype_from_env("DataTransfer")
                .unwrap_or_else(|| self.object_constructor_prototype_value());
            Self::set_internal_prototype(object, prototype);
        }
        value
    }

    pub(crate) fn new_data_transfer_object_value(&mut self, event_type: &str) -> Value {
        let value = self.new_clipboard_data_object_value("");
        if let Value::Object(owner) = &value {
            let mut entries = owner.borrow_mut();
            Self::object_set_entry(
                &mut entries,
                INTERNAL_DATA_TRANSFER_OBJECT_KEY.to_string(),
                Value::Bool(true),
            );
            Self::object_set_entry(
                &mut entries,
                INTERNAL_DATA_TRANSFER_EVENT_TYPE_KEY.to_string(),
                Value::String(event_type.to_ascii_lowercase()),
            );
            Self::object_set_entry(
                &mut entries,
                "dropEffect".to_string(),
                Value::String("none".to_string()),
            );
            Self::object_set_entry(
                &mut entries,
                "effectAllowed".to_string(),
                Value::String("all".to_string()),
            );
            let files = Self::new_array_value(Vec::new());
            Self::object_set_entry(
                &mut entries,
                INTERNAL_DATA_TRANSFER_FILES_KEY.to_string(),
                files.clone(),
            );
            Self::object_set_entry(&mut entries, "files".to_string(), files);
            let items =
                Self::new_data_transfer_item_list_value(owner.clone(), event_type, Vec::new());
            Self::object_set_entry(
                &mut entries,
                INTERNAL_DATA_TRANSFER_ITEMS_KEY.to_string(),
                items.clone(),
            );
            Self::object_set_entry(&mut entries, "items".to_string(), items);
            Self::object_set_entry(
                &mut entries,
                "setDragImage".to_string(),
                Self::new_builtin_placeholder_function(),
            );
            Self::object_set_entry(
                &mut entries,
                "addElement".to_string(),
                Self::new_builtin_placeholder_function(),
            );
            Self::mark_object_properties_non_enumerable(
                &mut entries,
                &[
                    "getData",
                    "setData",
                    "clearData",
                    "setDragImage",
                    "addElement",
                ],
            );
        }
        value
    }

    pub(crate) fn new_data_transfer_item_string_value(format: &str, data: &str) -> Value {
        let mut entries = vec![
            (
                INTERNAL_DATA_TRANSFER_ITEM_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_DATA_TRANSFER_ITEM_KIND_KEY.to_string(),
                Value::String("string".to_string()),
            ),
            (
                INTERNAL_DATA_TRANSFER_ITEM_TYPE_KEY.to_string(),
                Value::String(format.to_string()),
            ),
            (
                INTERNAL_DATA_TRANSFER_ITEM_DATA_KEY.to_string(),
                Value::String(data.to_string()),
            ),
            ("kind".to_string(), Value::String("string".to_string())),
            ("type".to_string(), Value::String(format.to_string())),
            (
                "getAsFile".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getAsFileSystemHandle".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getAsString".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "webkitGetAsEntry".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        Self::mark_object_properties_non_enumerable(
            &mut entries,
            &[
                "getAsFile",
                "getAsFileSystemHandle",
                "getAsString",
                "webkitGetAsEntry",
            ],
        );
        Self::new_object_value(entries)
    }

    pub(crate) fn new_data_transfer_item_file_value(format: &str, file: Value) -> Value {
        let mut entries = vec![
            (
                INTERNAL_DATA_TRANSFER_ITEM_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_DATA_TRANSFER_ITEM_KIND_KEY.to_string(),
                Value::String("file".to_string()),
            ),
            (
                INTERNAL_DATA_TRANSFER_ITEM_TYPE_KEY.to_string(),
                Value::String(format.to_string()),
            ),
            (INTERNAL_DATA_TRANSFER_ITEM_DATA_KEY.to_string(), file),
            ("kind".to_string(), Value::String("file".to_string())),
            ("type".to_string(), Value::String(format.to_string())),
            (
                "getAsFile".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getAsFileSystemHandle".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getAsString".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "webkitGetAsEntry".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        Self::mark_object_properties_non_enumerable(
            &mut entries,
            &[
                "getAsFile",
                "getAsFileSystemHandle",
                "getAsString",
                "webkitGetAsEntry",
            ],
        );
        Self::new_object_value(entries)
    }

    pub(crate) fn new_data_transfer_item_list_value(
        owner: Rc<RefCell<ObjectValue>>,
        event_type: &str,
        items: Vec<Value>,
    ) -> Value {
        let value = Self::new_array_value(items);
        if let Value::Array(list) = &value {
            Self::set_array_property(
                list,
                INTERNAL_DATA_TRANSFER_ITEM_LIST_OBJECT_KEY.to_string(),
                Value::Bool(true),
            );
            Self::set_array_property(
                list,
                INTERNAL_DATA_TRANSFER_ITEM_LIST_OWNER_KEY.to_string(),
                Value::Object(owner),
            );
            Self::set_array_property(
                list,
                INTERNAL_DATA_TRANSFER_ITEM_LIST_EVENT_TYPE_KEY.to_string(),
                Value::String(event_type.to_ascii_lowercase()),
            );
            Self::set_array_property(
                list,
                "add".to_string(),
                Self::new_builtin_placeholder_function(),
            );
            Self::set_array_property(
                list,
                "remove".to_string(),
                Self::new_builtin_placeholder_function(),
            );
            Self::set_array_property(
                list,
                "clear".to_string(),
                Self::new_builtin_placeholder_function(),
            );
            Self::mark_object_properties_non_enumerable(
                &mut list.borrow_mut().properties,
                &["add", "remove", "clear"],
            );
        }
        value
    }

    pub(crate) fn new_named_node_map_value(&mut self, owner: NodeId) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_NAMED_NODE_MAP_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_NAMED_NODE_MAP_OWNER_NODE_KEY.to_string(),
                Value::Node(owner),
            ),
        ])
    }

    pub(crate) fn new_range_object_value(root: NodeId) -> Value {
        let mut entries = vec![
            (INTERNAL_RANGE_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_RANGE_START_CONTAINER_KEY.to_string(),
                Value::Node(root),
            ),
            (
                INTERNAL_RANGE_START_OFFSET_KEY.to_string(),
                Value::Number(0),
            ),
            (
                INTERNAL_RANGE_END_CONTAINER_KEY.to_string(),
                Value::Node(root),
            ),
            (INTERNAL_RANGE_END_OFFSET_KEY.to_string(), Value::Number(0)),
            ("startContainer".to_string(), Value::Node(root)),
            ("startOffset".to_string(), Value::Number(0)),
            ("endContainer".to_string(), Value::Node(root)),
            ("endOffset".to_string(), Value::Number(0)),
            (
                "setStart".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "setEnd".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        Self::mark_object_properties_non_enumerable(&mut entries, &["setStart", "setEnd"]);
        Self::new_object_value(entries)
    }

    pub(crate) fn new_selection_object_value(root: NodeId) -> Value {
        let range = Self::new_range_object_value(root);
        let mut entries = vec![
            (INTERNAL_SELECTION_OBJECT_KEY.to_string(), Value::Bool(true)),
            (INTERNAL_SELECTION_RANGE_KEY.to_string(), range),
            ("anchorNode".to_string(), Value::Null),
            ("anchorOffset".to_string(), Value::Number(0)),
            ("focusNode".to_string(), Value::Null),
            ("focusOffset".to_string(), Value::Number(0)),
            ("isCollapsed".to_string(), Value::Bool(true)),
            ("rangeCount".to_string(), Value::Number(0)),
            ("type".to_string(), Value::String("None".to_string())),
            ("direction".to_string(), Value::String("none".to_string())),
            (
                "addRange".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "collapse".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "collapseToEnd".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "collapseToStart".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "containsNode".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "deleteFromDocument".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "empty".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "extend".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getComposedRanges".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getRangeAt".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "modify".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "removeAllRanges".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "removeRange".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "selectAllChildren".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "setBaseAndExtent".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "setPosition".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "toString".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        Self::mark_object_properties_non_enumerable(
            &mut entries,
            &[
                "addRange",
                "collapse",
                "collapseToEnd",
                "collapseToStart",
                "containsNode",
                "deleteFromDocument",
                "empty",
                "extend",
                "getComposedRanges",
                "getRangeAt",
                "modify",
                "removeAllRanges",
                "removeRange",
                "selectAllChildren",
                "setBaseAndExtent",
                "setPosition",
                "toString",
            ],
        );
        Self::new_object_value(entries)
    }

    pub(crate) fn new_animation_object_value(
        id: String,
        keyframes: Value,
        options: Value,
        timeline: Value,
        range_start: Value,
        range_end: Value,
    ) -> Value {
        let mut entries = vec![
            (INTERNAL_ANIMATION_OBJECT_KEY.to_string(), Value::Bool(true)),
            ("id".to_string(), Value::String(id)),
            (
                "playState".to_string(),
                Value::String("running".to_string()),
            ),
            ("currentTime".to_string(), Value::Number(0)),
            ("startTime".to_string(), Value::Number(0)),
            ("pending".to_string(), Value::Bool(false)),
            ("playbackRate".to_string(), Value::Number(1)),
            ("timeline".to_string(), timeline),
            ("rangeStart".to_string(), range_start),
            ("rangeEnd".to_string(), range_end),
            ("keyframes".to_string(), keyframes),
            ("options".to_string(), options),
            (
                "cancel".to_string(),
                Self::new_receiver_builtin_callable("animation", "cancel"),
            ),
            (
                "finish".to_string(),
                Self::new_receiver_builtin_callable("animation", "finish"),
            ),
            (
                "pause".to_string(),
                Self::new_receiver_builtin_callable("animation", "pause"),
            ),
            (
                "play".to_string(),
                Self::new_receiver_builtin_callable("animation", "play"),
            ),
            (
                "reverse".to_string(),
                Self::new_receiver_builtin_callable("animation", "reverse"),
            ),
            (
                "updatePlaybackRate".to_string(),
                Self::new_receiver_builtin_callable("animation", "updatePlaybackRate"),
            ),
            (
                "commitStyles".to_string(),
                Self::new_receiver_builtin_callable("animation", "commitStyles"),
            ),
            (
                "persist".to_string(),
                Self::new_receiver_builtin_callable("animation", "persist"),
            ),
            (
                "Symbol.toStringTag".to_string(),
                Value::String("Animation".to_string()),
            ),
        ];
        Self::mark_object_properties_non_enumerable(
            &mut entries,
            &[
                "cancel",
                "finish",
                "pause",
                "play",
                "reverse",
                "updatePlaybackRate",
                "commitStyles",
                "persist",
                "Symbol.toStringTag",
            ],
        );
        Self::new_object_value(entries)
    }

    pub(crate) fn create_element_is_option_from_arg(arg: Option<&Value>) -> Option<String> {
        let arg = arg?;
        match arg {
            Value::Undefined | Value::Null => None,
            // Legacy compatibility: allow passing a string as the custom element name.
            Value::String(value) => Some(value.clone()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                match Self::object_get_entry(&entries, "is") {
                    Some(Value::Undefined) | Some(Value::Null) | None => None,
                    Some(value) => Some(value.as_string()),
                }
            }
            _ => None,
        }
    }

    pub(crate) fn canvas_dimension_default(name: &str) -> i64 {
        match name {
            "width" => 300,
            "height" => 150,
            _ => 0,
        }
    }

    pub(crate) fn canvas_dimension_value(&self, node: NodeId, name: &str) -> i64 {
        let default = if self.dom.tag_name(node).is_some_and(|tag| {
            tag.eq_ignore_ascii_case("canvas") || tag.eq_ignore_ascii_case("iframe")
        }) {
            Self::canvas_dimension_default(name)
        } else {
            0
        };
        self.dom
            .attr(node, name)
            .and_then(|raw| Self::parse_non_negative_int(&raw))
            .unwrap_or(default)
    }

    pub(crate) fn set_canvas_dimension_value(
        &mut self,
        node: NodeId,
        name: &str,
        value: &Value,
    ) -> Result<()> {
        let next = match value {
            Value::Number(number) => *number,
            Value::Float(number) if number.is_finite() => *number as i64,
            Value::BigInt(number) => number.to_string().parse::<i64>().unwrap_or(0),
            other => other.as_string().trim().parse::<i64>().unwrap_or(0),
        };
        let next = next.max(0);
        self.dom.set_attr(node, name, &next.to_string())
    }

    pub(crate) fn new_canvas_2d_context_value(&self, canvas_node: NodeId, alpha: bool) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CANVAS_2D_CONTEXT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (INTERNAL_CANVAS_2D_ALPHA_KEY.to_string(), Value::Bool(alpha)),
            (
                INTERNAL_CANVAS_2D_LINE_DASH_KEY.to_string(),
                Self::new_array_value(Vec::new()),
            ),
            (
                INTERNAL_CANVAS_2D_TRANSFORM_KEY.to_string(),
                Self::new_array_value(vec![
                    Value::Number(1),
                    Value::Number(0),
                    Value::Number(0),
                    Value::Number(1),
                    Value::Number(0),
                    Value::Number(0),
                ]),
            ),
            ("canvas".to_string(), Value::Node(canvas_node)),
            (
                "fillStyle".to_string(),
                Value::String("#000000".to_string()),
            ),
            (
                "strokeStyle".to_string(),
                Value::String("#000000".to_string()),
            ),
            ("lineWidth".to_string(), Value::Number(1)),
            ("lineCap".to_string(), Value::String("butt".to_string())),
            ("lineJoin".to_string(), Value::String("miter".to_string())),
            ("miterLimit".to_string(), Value::Number(10)),
            ("lineDashOffset".to_string(), Value::Number(0)),
            (
                "font".to_string(),
                Value::String("10px sans-serif".to_string()),
            ),
            ("textAlign".to_string(), Value::String("start".to_string())),
            (
                "textBaseline".to_string(),
                Value::String("alphabetic".to_string()),
            ),
            (
                "direction".to_string(),
                Value::String("inherit".to_string()),
            ),
            (
                "letterSpacing".to_string(),
                Value::String("0px".to_string()),
            ),
            ("fontKerning".to_string(), Value::String("auto".to_string())),
            (
                "fontStretch".to_string(),
                Value::String("normal".to_string()),
            ),
            (
                "fontVariantCaps".to_string(),
                Value::String("normal".to_string()),
            ),
            (
                "textRendering".to_string(),
                Value::String("auto".to_string()),
            ),
            ("wordSpacing".to_string(), Value::String("0px".to_string())),
            ("lang".to_string(), Value::String("inherit".to_string())),
            ("shadowBlur".to_string(), Value::Number(0)),
            (
                "shadowColor".to_string(),
                Value::String("rgba(0, 0, 0, 0)".to_string()),
            ),
            ("shadowOffsetX".to_string(), Value::Number(0)),
            ("shadowOffsetY".to_string(), Value::Number(0)),
            ("globalAlpha".to_string(), Value::Number(1)),
            (
                "globalCompositeOperation".to_string(),
                Value::String("source-over".to_string()),
            ),
            ("imageSmoothingEnabled".to_string(), Value::Bool(true)),
            (
                "imageSmoothingQuality".to_string(),
                Value::String("low".to_string()),
            ),
            ("filter".to_string(), Value::String("none".to_string())),
            (
                "clearRect".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "fillRect".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "strokeRect".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "fillText".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "strokeText".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "measureText".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "beginPath".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "closePath".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "moveTo".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "lineTo".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "bezierCurveTo".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "quadraticCurveTo".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("arc".to_string(), Self::new_builtin_placeholder_function()),
            (
                "arcTo".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "ellipse".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("rect".to_string(), Self::new_builtin_placeholder_function()),
            (
                "roundRect".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("fill".to_string(), Self::new_builtin_placeholder_function()),
            (
                "stroke".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "drawFocusIfNeeded".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("clip".to_string(), Self::new_builtin_placeholder_function()),
            (
                "isPointInPath".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "isPointInStroke".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "setLineDash".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getLineDash".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createConicGradient".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createLinearGradient".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createRadialGradient".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createPattern".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "drawImage".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "createImageData".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getImageData".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "putImageData".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getTransform".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "rotate".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "scale".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "translate".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "transform".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "setTransform".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "resetTransform".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("save".to_string(), Self::new_builtin_placeholder_function()),
            (
                "restore".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "reset".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "getContextAttributes".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "isContextLost".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "toString".to_string(),
                Self::new_receiver_builtin_callable("canvas_2d_context", "toString"),
            ),
            (
                "Symbol.toStringTag".to_string(),
                Value::String("CanvasRenderingContext2D".to_string()),
            ),
        ])
    }

    pub(crate) fn new_array_value(values: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(ArrayValue::new(values))))
    }

    pub(crate) fn set_array_property(array: &Rc<RefCell<ArrayValue>>, key: String, value: Value) {
        Self::object_set_entry(&mut array.borrow_mut().properties, key, value);
    }

    pub(crate) fn array_hole_storage_key(index: usize) -> String {
        format!("{INTERNAL_ARRAY_HOLE_KEY_PREFIX}{index}")
    }

    pub(crate) fn array_index_is_hole(array: &ArrayValue, index: usize) -> bool {
        let hole_key = Self::array_hole_storage_key(index);
        Self::object_get_entry(&array.properties, &hole_key).is_some()
    }

    pub(crate) fn clear_array_hole(array: &Rc<RefCell<ArrayValue>>, index: usize) {
        let hole_key = Self::array_hole_storage_key(index);
        array.borrow_mut().properties.delete_entry(&hole_key);
    }

    pub(crate) fn mark_array_hole(array: &Rc<RefCell<ArrayValue>>, index: usize) {
        let hole_key = Self::array_hole_storage_key(index);
        Self::object_set_entry(
            &mut array.borrow_mut().properties,
            hole_key,
            Value::Bool(true),
        );
    }

    pub(crate) fn delete_object_getter_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let getter_key = Self::object_getter_storage_key(key);
        let mut deleted = entries.delete_entry(&getter_key);
        let undefined_getter_key = Self::object_undefined_getter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_getter_key);
        deleted
    }

    pub(crate) fn delete_object_setter_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let setter_key = Self::object_setter_storage_key(key);
        let mut deleted = entries.delete_entry(&setter_key);
        let undefined_setter_key = Self::object_undefined_setter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_setter_key);
        deleted
    }

    pub(crate) fn delete_object_property_auxiliary_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let mut deleted = Self::delete_object_getter_entries(entries, key);
        deleted |= Self::delete_object_setter_entries(entries, key);
        let non_enumerable_key = Self::object_non_enumerable_storage_key(key);
        deleted |= entries.delete_entry(&non_enumerable_key);
        let non_writable_key = Self::object_non_writable_storage_key(key);
        deleted |= entries.delete_entry(&non_writable_key);
        let non_configurable_key = Self::object_non_configurable_storage_key(key);
        deleted |= entries.delete_entry(&non_configurable_key);
        deleted
    }

    pub(crate) fn delete_object_property_entries(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) -> bool {
        let mut deleted = entries.delete_entry(key);
        let getter_key = Self::object_getter_storage_key(key);
        deleted |= entries.delete_entry(&getter_key);
        let setter_key = Self::object_setter_storage_key(key);
        deleted |= entries.delete_entry(&setter_key);
        let undefined_getter_key = Self::object_undefined_getter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_getter_key);
        let undefined_setter_key = Self::object_undefined_setter_storage_key(key);
        deleted |= entries.delete_entry(&undefined_setter_key);
        let non_enumerable_key = Self::object_non_enumerable_storage_key(key);
        deleted |= entries.delete_entry(&non_enumerable_key);
        let non_writable_key = Self::object_non_writable_storage_key(key);
        deleted |= entries.delete_entry(&non_writable_key);
        let non_configurable_key = Self::object_non_configurable_storage_key(key);
        deleted |= entries.delete_entry(&non_configurable_key);
        deleted
    }

    pub(crate) fn new_object_value(entries: Vec<(String, Value)>) -> Value {
        Value::Object(Rc::new(RefCell::new(ObjectValue::new(entries))))
    }

    pub(crate) fn mock_file_to_value(file: &MockFile) -> Value {
        let file_blob = Self::new_blob_value(file.bytes.clone(), file.mime_type.clone());
        Self::new_object_value(vec![
            (INTERNAL_MOCK_FILE_OBJECT_KEY.to_string(), Value::Bool(true)),
            (INTERNAL_MOCK_FILE_BLOB_KEY.to_string(), file_blob),
            ("name".to_string(), Value::String(file.name.clone())),
            (
                "lastModified".to_string(),
                Value::Number(file.last_modified),
            ),
            ("size".to_string(), Value::Number(file.size.max(0))),
            ("type".to_string(), Value::String(file.mime_type.clone())),
            (
                "webkitRelativePath".to_string(),
                Value::String(file.webkit_relative_path.clone()),
            ),
            (
                "arrayBuffer".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("text".to_string(), Self::new_builtin_placeholder_function()),
            (
                "bytes".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "stream".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ])
    }

    fn input_files_type_error() -> Error {
        Error::ScriptRuntime(
            "TypeError: Failed to set the 'files' property on 'HTMLInputElement': The provided value is not of type 'FileList'."
                .into(),
        )
    }

    fn mock_file_from_input_assignment_value(&self, value: &Value) -> Result<MockFile> {
        let Value::Object(entries) = value else {
            return Err(Self::input_files_type_error());
        };
        let entries = entries.borrow();
        if !Self::is_mock_file_object(&entries) {
            return Err(Self::input_files_type_error());
        }

        let (bytes, blob_mime_type) =
            match Self::object_get_entry(&entries, INTERNAL_MOCK_FILE_BLOB_KEY) {
                Some(Value::Blob(blob)) => {
                    let blob = blob.borrow();
                    (blob.bytes.clone(), blob.mime_type.clone())
                }
                _ => (Vec::new(), String::new()),
            };

        let explicit_mime_type = Self::object_get_entry(&entries, "type")
            .map(|value| Self::normalize_blob_type(&value.as_string()))
            .unwrap_or_default();
        let mime_type = if explicit_mime_type.is_empty() {
            blob_mime_type
        } else {
            explicit_mime_type
        };
        let size = Self::object_get_entry(&entries, "size")
            .map(|value| Self::value_to_i64(&value).max(0))
            .unwrap_or(bytes.len() as i64);
        let file = MockFile {
            name: Self::object_get_entry(&entries, "name")
                .map(|value| value.as_string())
                .unwrap_or_default(),
            size,
            mime_type,
            last_modified: Self::object_get_entry(&entries, "lastModified")
                .map(|value| Self::value_to_i64(&value))
                .unwrap_or(0),
            webkit_relative_path: Self::object_get_entry(&entries, "webkitRelativePath")
                .map(|value| value.as_string())
                .unwrap_or_default(),
            bytes,
        };
        Ok(normalize_mock_file(&file))
    }

    pub(crate) fn mock_files_from_input_assignment_value(
        &self,
        value: &Value,
    ) -> Result<Vec<MockFile>> {
        if matches!(value, Value::Null | Value::Undefined) {
            return Ok(Vec::new());
        }

        let file_values = match value {
            Value::Array(values) => values.borrow().clone(),
            Value::Object(entries) => {
                let (is_mock_file, is_iterator, has_length) = {
                    let entries_ref = entries.borrow();
                    (
                        Self::is_mock_file_object(&entries_ref),
                        Self::is_iterator_object(&entries_ref),
                        Self::object_get_entry(&entries_ref, "length").is_some(),
                    )
                };
                if is_mock_file || (!is_iterator && !has_length) {
                    return Err(Self::input_files_type_error());
                }
                self.array_like_values_from_value(value)
                    .map_err(|_| Self::input_files_type_error())?
            }
            _ => self
                .array_like_values_from_value(value)
                .map_err(|_| Self::input_files_type_error())?,
        };

        let mut files = Vec::with_capacity(file_values.len());
        for file_value in file_values {
            files.push(self.mock_file_from_input_assignment_value(&file_value)?);
        }
        Ok(files)
    }

    fn new_class_list_method_callable(kind: &str) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String(kind.to_string()),
        )])
    }

    fn new_named_node_map_method_callable(kind: &str) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String(kind.to_string()),
        )])
    }

    pub(crate) fn new_dom_string_map_value(&self, node: NodeId) -> Value {
        let entries = vec![
            (
                INTERNAL_DOM_STRING_MAP_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_DOM_STRING_MAP_OWNER_NODE_KEY.to_string(),
                Value::Node(node),
            ),
        ];
        Self::new_object_value(entries)
    }

    pub(crate) fn new_class_list_value(node: NodeId) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CLASS_LIST_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (INTERNAL_CLASS_LIST_NODE_KEY.to_string(), Value::Node(node)),
        ])
    }

    pub(crate) fn new_image_bitmap_value(&mut self, width: i64, height: i64) -> Value {
        let object = Self::new_object_value(vec![
            (
                INTERNAL_IMAGE_BITMAP_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_IMAGE_BITMAP_WIDTH_KEY.to_string(),
                Value::Number(width.max(0)),
            ),
            (
                INTERNAL_IMAGE_BITMAP_HEIGHT_KEY.to_string(),
                Value::Number(height.max(0)),
            ),
        ]);
        if let Value::Object(entries) = &object {
            Self::set_internal_prototype(
                entries,
                self.cached_image_bitmap_constructor_prototype_value(),
            );
        }
        object
    }

    pub(crate) fn new_time_ranges_value(&mut self, media: NodeId, kind: &str) -> Value {
        let object = Self::new_object_value(vec![
            (
                INTERNAL_TIME_RANGES_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_TIME_RANGES_MEDIA_NODE_KEY.to_string(),
                Value::Node(media),
            ),
            (
                INTERNAL_TIME_RANGES_KIND_KEY.to_string(),
                Value::String(kind.to_string()),
            ),
        ]);
        if let Value::Object(entries) = &object {
            Self::set_internal_prototype(
                entries,
                self.cached_time_ranges_constructor_prototype_value(),
            );
        }
        object
    }

    pub(crate) fn text_track_object_value(&mut self, node: NodeId) -> Value {
        let existing = self.dom_runtime.live_text_track_objects.get(&node).cloned();
        let object = existing.unwrap_or_else(|| {
            let object = Rc::new(RefCell::new(ObjectValue::new(vec![
                (
                    INTERNAL_TEXT_TRACK_OBJECT_KEY.to_string(),
                    Value::Bool(true),
                ),
                (INTERNAL_TEXT_TRACK_NODE_KEY.to_string(), Value::Node(node)),
                (
                    INTERNAL_TEXT_TRACK_MODE_KEY.to_string(),
                    Value::String("disabled".to_string()),
                ),
            ])));
            Self::set_internal_prototype(
                &object,
                self.cached_text_track_constructor_prototype_value(),
            );
            self.dom_runtime
                .live_text_track_objects
                .insert(node, object.clone());
            object
        });
        Value::Object(object)
    }

    pub(crate) fn input_files_value(&self, node: NodeId) -> Result<Value> {
        let element = self
            .dom
            .element(node)
            .ok_or_else(|| Error::ScriptRuntime("files target is not an element".into()))?;
        if !is_file_input_element(element) {
            return Ok(Value::Null);
        }
        let files = self.dom.files(node)?;
        Ok(Self::new_array_value(
            files.iter().map(Self::mock_file_to_value).collect(),
        ))
    }

    fn new_receiver_builtin_constructor_object(
        callable_kind: Option<&str>,
        family: &str,
        methods: &[&str],
    ) -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let mut constructor_entries = Vec::new();
        if let Some(kind) = callable_kind {
            constructor_entries.push((
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String(kind.to_string()),
            ));
        }
        constructor_entries.push(("prototype".to_string(), prototype.clone()));
        let constructor = Self::new_object_value(constructor_entries);
        if let Value::Object(prototype_entries) = &prototype {
            let mut prototype_entries = prototype_entries.borrow_mut();
            Self::object_set_entry(
                &mut prototype_entries,
                "constructor".to_string(),
                constructor.clone(),
            );
            for method in methods {
                Self::object_set_entry(
                    &mut prototype_entries,
                    (*method).to_string(),
                    Self::new_receiver_builtin_callable(family, method),
                );
            }
        }
        if let Value::Object(prototype_entries) = &prototype {
            Self::mark_existing_public_properties_non_enumerable(prototype_entries);
        }
        if let Value::Object(constructor_entries) = &constructor {
            Self::mark_property_non_enumerable(constructor_entries, "prototype");
        }
        constructor
    }

    fn new_object_backed_constructor_with_prototype(
        callable_kind: &str,
        extra_public_entries: Vec<(String, Value)>,
    ) -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let mut constructor_entries = vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String(callable_kind.to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ];
        constructor_entries.extend(extra_public_entries);
        let constructor = Self::new_object_value(constructor_entries);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
            Self::mark_existing_public_properties_non_enumerable(prototype_entries);
        }
        if let Value::Object(constructor_entries) = &constructor {
            Self::mark_existing_public_properties_non_enumerable(constructor_entries);
        }
        constructor
    }

    pub(crate) fn shared_core_constructor_bindings(
        string_constructor: &Value,
        boolean_constructor: &Value,
        number_constructor: &Value,
        bigint_constructor: &Value,
        symbol_constructor: &Value,
        object_constructor: &Value,
        reflect_object: &Value,
    ) -> Vec<(String, Value)> {
        let object_prototype = match object_constructor {
            Value::Object(entries) => {
                let entries = entries.borrow();
                Self::object_get_entry(&entries, "prototype")
            }
            _ => None,
        };
        if let Some(object_prototype) = object_prototype {
            for constructor in [boolean_constructor, number_constructor, bigint_constructor] {
                let Value::Object(entries) = constructor else {
                    continue;
                };
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype_entries)) = prototype {
                    Self::set_internal_prototype(&prototype_entries, object_prototype.clone());
                }
            }
        }

        let mut bindings = vec![
            ("String".to_string(), string_constructor.clone()),
            ("Boolean".to_string(), boolean_constructor.clone()),
            ("Number".to_string(), number_constructor.clone()),
            ("BigInt".to_string(), bigint_constructor.clone()),
            ("Symbol".to_string(), symbol_constructor.clone()),
            ("RegExp".to_string(), Value::RegExpConstructor),
            ("Object".to_string(), object_constructor.clone()),
            ("Reflect".to_string(), reflect_object.clone()),
            ("Blob".to_string(), Value::BlobConstructor),
            ("URL".to_string(), Value::UrlConstructor),
            (
                "URLSearchParams".to_string(),
                Value::UrlSearchParamsConstructor,
            ),
            ("ArrayBuffer".to_string(), Value::ArrayBufferConstructor),
            ("Promise".to_string(), Value::PromiseConstructor),
            ("Map".to_string(), Value::MapConstructor),
            ("WeakMap".to_string(), Value::WeakMapConstructor),
            ("Set".to_string(), Value::SetConstructor),
            ("WeakSet".to_string(), Value::WeakSetConstructor),
        ];
        for kind in TypedArrayKind::concrete_kinds() {
            bindings.push((
                kind.name().to_string(),
                Value::TypedArrayConstructor(TypedArrayConstructorKind::Concrete(*kind)),
            ));
        }
        bindings
    }

    pub(crate) fn new_boolean_constructor_callable() -> Value {
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("boolean_constructor"),
            "boolean",
            &["toString", "valueOf"],
        );
        if let Value::Object(entries) = &constructor {
            Self::mark_existing_public_properties_non_enumerable(entries);
        }
        constructor
    }

    pub(crate) fn new_number_constructor_callable() -> Value {
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("number_constructor"),
            "number",
            &[
                "toExponential",
                "toFixed",
                "toLocaleString",
                "toPrecision",
                "toString",
                "valueOf",
            ],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let mut entries = constructor_entries.borrow_mut();
        for (key, value) in [
            (
                "isFinite",
                Self::new_number_static_method_callable("isFinite"),
            ),
            (
                "isInteger",
                Self::new_number_static_method_callable("isInteger"),
            ),
            ("isNaN", Self::new_number_static_method_callable("isNaN")),
            (
                "isSafeInteger",
                Self::new_number_static_method_callable("isSafeInteger"),
            ),
            (
                "parseFloat",
                Self::new_number_static_method_callable("parseFloat"),
            ),
            (
                "parseInt",
                Self::new_number_static_method_callable("parseInt"),
            ),
        ] {
            Self::object_set_entry(&mut entries, key.to_string(), value);
        }
        for (key, value) in [
            ("EPSILON", Value::Float(f64::EPSILON)),
            ("MAX_SAFE_INTEGER", Value::Number(9_007_199_254_740_991)),
            ("MAX_VALUE", Value::Float(f64::MAX)),
            ("MIN_SAFE_INTEGER", Value::Number(-9_007_199_254_740_991)),
            ("MIN_VALUE", Value::Float(f64::from_bits(1))),
            ("NaN", Value::Float(f64::NAN)),
            ("NEGATIVE_INFINITY", Value::Float(f64::NEG_INFINITY)),
            ("POSITIVE_INFINITY", Value::Float(f64::INFINITY)),
        ] {
            Self::object_set_entry(&mut entries, key.to_string(), value);
        }
        drop(entries);
        Self::mark_existing_public_properties_non_enumerable(constructor_entries);
        constructor
    }

    pub(crate) fn new_bigint_constructor_callable() -> Value {
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("bigint_constructor"),
            "bigint",
            &["toLocaleString", "toString", "valueOf"],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let mut entries = constructor_entries.borrow_mut();
        for (key, value) in [
            ("asIntN", Self::new_bigint_static_method_callable("asIntN")),
            (
                "asUintN",
                Self::new_bigint_static_method_callable("asUintN"),
            ),
        ] {
            Self::object_set_entry(&mut entries, key.to_string(), value);
        }
        drop(entries);
        Self::mark_existing_public_properties_non_enumerable(constructor_entries);
        constructor
    }

    pub(crate) fn new_object_constructor_value() -> Value {
        let prototype = Self::new_object_value(vec![
            (
                "toString".to_string(),
                Self::new_receiver_builtin_callable("object", "toString"),
            ),
            (
                "valueOf".to_string(),
                Self::new_receiver_builtin_callable("object", "valueOf"),
            ),
            (
                "hasOwnProperty".to_string(),
                Self::new_receiver_builtin_callable("object", "hasOwnProperty"),
            ),
            (
                "isPrototypeOf".to_string(),
                Self::new_receiver_builtin_callable("object", "isPrototypeOf"),
            ),
            (
                "propertyIsEnumerable".to_string(),
                Self::new_receiver_builtin_callable("object", "propertyIsEnumerable"),
            ),
        ]);
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("object_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
            (
                "create".to_string(),
                Self::new_object_static_method_callable("create"),
            ),
            (
                "assign".to_string(),
                Self::new_object_static_method_callable("assign"),
            ),
            (
                "getOwnPropertyDescriptor".to_string(),
                Self::new_object_static_method_callable("getOwnPropertyDescriptor"),
            ),
            (
                "defineProperty".to_string(),
                Self::new_object_static_method_callable("defineProperty"),
            ),
            (
                "getOwnPropertyNames".to_string(),
                Self::new_object_static_method_callable("getOwnPropertyNames"),
            ),
            (
                "getOwnPropertySymbols".to_string(),
                Self::new_object_static_method_callable("getOwnPropertySymbols"),
            ),
            (
                "keys".to_string(),
                Self::new_object_static_method_callable("keys"),
            ),
            (
                "values".to_string(),
                Self::new_object_static_method_callable("values"),
            ),
            (
                "entries".to_string(),
                Self::new_object_static_method_callable("entries"),
            ),
            (
                "fromEntries".to_string(),
                Self::new_object_static_method_callable("fromEntries"),
            ),
            (
                "hasOwn".to_string(),
                Self::new_object_static_method_callable("hasOwn"),
            ),
            (
                "getPrototypeOf".to_string(),
                Self::new_object_static_method_callable("getPrototypeOf"),
            ),
            (
                "setPrototypeOf".to_string(),
                Self::new_object_static_method_callable("setPrototypeOf"),
            ),
            (
                "freeze".to_string(),
                Self::new_object_static_method_callable("freeze"),
            ),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            let mut prototype_entries = prototype_entries.borrow_mut();
            Self::object_set_entry(
                &mut prototype_entries,
                "constructor".to_string(),
                constructor.clone(),
            );
            Self::object_set_entry(
                &mut prototype_entries,
                INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                Value::Null,
            );
        }
        if let Value::Object(prototype_entries) = &prototype {
            Self::mark_existing_public_properties_non_enumerable(prototype_entries);
        }
        if let Value::Object(constructor_entries) = &constructor {
            Self::mark_existing_public_properties_non_enumerable(constructor_entries);
        }
        constructor
    }

    pub(crate) fn new_reflect_object_value(&mut self) -> Value {
        let to_string_tag = self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag);
        let reflect = Self::new_object_value(vec![
            (
                "set".to_string(),
                Self::new_reflect_static_method_callable("set"),
            ),
            (
                "ownKeys".to_string(),
                Self::new_reflect_static_method_callable("ownKeys"),
            ),
            (to_string_tag_key, Value::String("Reflect".to_string())),
        ]);
        if let Value::Object(entries) = &reflect {
            Self::mark_existing_public_properties_non_enumerable(entries);
        }
        reflect
    }

    pub(crate) fn new_event_target_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("event_target_constructor", vec![])
    }

    pub(crate) fn new_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("event_constructor", vec![])
    }

    pub(crate) fn new_custom_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("custom_event_constructor", vec![])
    }

    pub(crate) fn new_mouse_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("mouse_event_constructor", vec![])
    }

    pub(crate) fn new_keyboard_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype(
            "keyboard_event_constructor",
            vec![
                ("DOM_KEY_LOCATION_STANDARD".to_string(), Value::Number(0x00)),
                ("DOM_KEY_LOCATION_LEFT".to_string(), Value::Number(0x01)),
                ("DOM_KEY_LOCATION_RIGHT".to_string(), Value::Number(0x02)),
                ("DOM_KEY_LOCATION_NUMPAD".to_string(), Value::Number(0x03)),
            ],
        )
    }

    pub(crate) fn new_wheel_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype(
            "wheel_event_constructor",
            vec![
                ("DOM_DELTA_PIXEL".to_string(), Value::Number(0)),
                ("DOM_DELTA_LINE".to_string(), Value::Number(1)),
                ("DOM_DELTA_PAGE".to_string(), Value::Number(2)),
            ],
        )
    }

    pub(crate) fn new_navigate_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("navigate_event_constructor", vec![])
    }

    pub(crate) fn new_pointer_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("pointer_event_constructor", vec![])
    }

    pub(crate) fn new_error_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("error_event_constructor", vec![])
    }

    pub(crate) fn new_hash_change_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("hash_change_event_constructor", vec![])
    }

    pub(crate) fn new_before_unload_event_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype(
            "before_unload_event_constructor",
            vec![],
        )
    }

    pub(crate) fn new_image_data_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("image_data_constructor", vec![])
    }

    pub(crate) fn new_navigate_event_default_signal_value() -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_EVENT_TARGET_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("aborted".to_string(), Value::Bool(false)),
            ("onabort".to_string(), Value::Null),
            (
                "addEventListener".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "removeEventListener".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "dispatchEvent".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ])
    }

    pub(crate) fn new_dom_parser_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("dom_parser_constructor".to_string()),
        )])
    }

    pub(crate) fn new_xml_serializer_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("xml_serializer_constructor".to_string()),
        )])
    }

    pub(crate) fn new_document_parse_html_callable(sanitize: bool) -> Value {
        let kind = if sanitize {
            "document_parse_html"
        } else {
            "document_parse_html_unsafe"
        };
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String(kind.to_string()),
        )])
    }

    pub(crate) fn new_document_constructor_value() -> Value {
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("document_constructor".to_string()),
            ),
            (
                "parseHTML".to_string(),
                Self::new_document_parse_html_callable(true),
            ),
            (
                "parseHTMLUnsafe".to_string(),
                Self::new_document_parse_html_callable(false),
            ),
        ]);
        if let Value::Object(entries) = &constructor {
            Self::mark_existing_public_properties_non_enumerable(entries);
        }
        constructor
    }

    pub(crate) fn new_fetch_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("fetch_function".to_string()),
        )])
    }

    pub(crate) fn new_match_media_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("match_media_function".to_string()),
        )])
    }

    pub(crate) fn new_window_close_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_close_function".to_string()),
        )])
    }

    pub(crate) fn new_window_open_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_open_function".to_string()),
        )])
    }

    pub(crate) fn new_window_stop_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_stop_function".to_string()),
        )])
    }

    pub(crate) fn new_window_focus_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_focus_function".to_string()),
        )])
    }

    pub(crate) fn new_window_scroll_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_scroll_function".to_string()),
        )])
    }

    pub(crate) fn new_window_scroll_by_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_scroll_by_function".to_string()),
        )])
    }

    pub(crate) fn new_window_scroll_to_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_scroll_to_function".to_string()),
        )])
    }

    pub(crate) fn new_window_move_by_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_move_by_function".to_string()),
        )])
    }

    pub(crate) fn new_window_move_to_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_move_to_function".to_string()),
        )])
    }

    pub(crate) fn new_window_resize_by_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_resize_by_function".to_string()),
        )])
    }

    pub(crate) fn new_window_resize_to_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_resize_to_function".to_string()),
        )])
    }

    pub(crate) fn new_window_post_message_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_post_message_function".to_string()),
        )])
    }

    pub(crate) fn new_window_get_computed_style_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_get_computed_style_function".to_string()),
        )])
    }

    pub(crate) fn new_window_alert_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_alert_function".to_string()),
        )])
    }

    pub(crate) fn new_window_confirm_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_confirm_function".to_string()),
        )])
    }

    pub(crate) fn new_window_print_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_print_function".to_string()),
        )])
    }

    pub(crate) fn new_window_report_error_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_report_error_function".to_string()),
        )])
    }

    pub(crate) fn new_window_prompt_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_prompt_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_window_close_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_window_close_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_window_focus_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_window_focus_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_window_print_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_window_print_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_document_open_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_document_open_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_document_write_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_document_write_function".to_string()),
        )])
    }

    pub(crate) fn new_popup_document_close_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("popup_document_close_function".to_string()),
        )])
    }

    pub(crate) fn new_request_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("request_constructor".to_string()),
        )])
    }

    pub(crate) fn new_file_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("file_constructor", vec![])
    }

    pub(crate) fn new_clipboard_item_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("clipboard_item_constructor".to_string()),
        )])
    }

    pub(crate) fn new_clipboard_write_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("clipboard_write".to_string()),
        )])
    }

    pub(crate) fn new_headers_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("headers_constructor".to_string()),
        )])
    }

    pub(crate) fn new_worker_constructor_value(&mut self) -> Value {
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("worker_constructor"),
            "worker",
            &["postMessage", "terminate"],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String("Worker".to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        constructor
    }

    pub(crate) fn new_data_transfer_constructor_value(&mut self) -> Value {
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("data_transfer_constructor"),
            "data_transfer",
            &[
                "getData",
                "setData",
                "clearData",
                "setDragImage",
                "addElement",
            ],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String("DataTransfer".to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        constructor
    }

    pub(crate) fn new_option_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("option_constructor".to_string()),
        )])
    }

    pub(crate) fn new_audio_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("audio_constructor".to_string()),
        )])
    }

    pub(crate) fn new_css_style_sheet_constructor_value() -> Value {
        Self::new_object_backed_constructor_with_prototype("css_style_sheet_constructor", vec![])
    }

    pub(crate) fn new_text_encoder_encode_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_encode".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_encode_into_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_encode_into".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_encoding_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_get_encoding".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_decode_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_decode".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_encoding_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_get_encoding".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_fatal_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_get_fatal".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_ignore_bom_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_get_ignore_bom".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_stream_encoding_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_stream_get_encoding".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_stream_readable_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_stream_get_readable".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_stream_writable_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_encoder_stream_get_writable".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_stream_encoding_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_stream_get_encoding".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_stream_fatal_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_stream_get_fatal".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_stream_ignore_bom_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_stream_get_ignore_bom".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_stream_readable_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_stream_get_readable".to_string()),
        )])
    }

    pub(crate) fn new_text_decoder_stream_writable_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("text_decoder_stream_get_writable".to_string()),
        )])
    }

    pub(crate) fn new_css_style_sheet_replace_sync_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("css_style_sheet_replace_sync".to_string()),
        )])
    }

    pub(crate) fn new_css_style_sheet_insert_rule_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("css_style_sheet_insert_rule".to_string()),
        )])
    }

    pub(crate) fn new_computed_style_get_property_value_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("computed_style_get_property_value".to_string()),
        )])
    }

    pub(crate) fn new_computed_style_item_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("computed_style_item".to_string()),
        )])
    }

    pub(crate) fn new_dom_rect_list_item_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("dom_rect_list_item".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_instance_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_TEXT_ENCODER_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )])
    }

    pub(crate) fn new_text_decoder_instance_value(
        encoding: &str,
        fatal: bool,
        ignore_bom: bool,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_TEXT_DECODER_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_TEXT_DECODER_ENCODING_KEY.to_string(),
                Value::String(encoding.to_string()),
            ),
            (
                INTERNAL_TEXT_DECODER_FATAL_KEY.to_string(),
                Value::Bool(fatal),
            ),
            (
                INTERNAL_TEXT_DECODER_IGNORE_BOM_KEY.to_string(),
                Value::Bool(ignore_bom),
            ),
        ])
    }

    pub(crate) fn new_text_encoder_stream_instance_value(
        readable: Value,
        writable: Value,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_TEXT_ENCODER_STREAM_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_TEXT_ENCODER_STREAM_READABLE_KEY.to_string(),
                readable,
            ),
            (
                INTERNAL_TEXT_ENCODER_STREAM_WRITABLE_KEY.to_string(),
                writable,
            ),
        ])
    }

    pub(crate) fn new_text_decoder_stream_instance_value(
        encoding: &str,
        fatal: bool,
        ignore_bom: bool,
        readable: Value,
        writable: Value,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_TEXT_DECODER_STREAM_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_TEXT_DECODER_STREAM_ENCODING_KEY.to_string(),
                Value::String(encoding.to_string()),
            ),
            (
                INTERNAL_TEXT_DECODER_STREAM_FATAL_KEY.to_string(),
                Value::Bool(fatal),
            ),
            (
                INTERNAL_TEXT_DECODER_STREAM_IGNORE_BOM_KEY.to_string(),
                Value::Bool(ignore_bom),
            ),
            (
                INTERNAL_TEXT_DECODER_STREAM_READABLE_KEY.to_string(),
                readable,
            ),
            (
                INTERNAL_TEXT_DECODER_STREAM_WRITABLE_KEY.to_string(),
                writable,
            ),
        ])
    }

    fn install_text_encoder_prototype_surface(&mut self, prototype: &Rc<RefCell<ObjectValue>>) {
        let has_encoding_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "encoding")
        };
        if !has_encoding_accessor {
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                Self::object_getter_storage_key("encoding"),
                Self::new_text_encoder_encoding_getter_callable(),
            );
            Self::mark_property_non_enumerable(prototype, "encoding");
        }
        for (name, callable) in [
            ("encode", Self::new_text_encoder_encode_callable()),
            ("encodeInto", Self::new_text_encoder_encode_into_callable()),
        ] {
            Self::object_set_entry(&mut prototype.borrow_mut(), name.to_string(), callable);
            Self::mark_property_non_enumerable(prototype, name);
        }
    }

    fn install_text_decoder_prototype_surface(&mut self, prototype: &Rc<RefCell<ObjectValue>>) {
        let has_encoding_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "encoding")
        };
        if !has_encoding_accessor {
            for (property, getter) in [
                (
                    "encoding",
                    Self::new_text_decoder_encoding_getter_callable(),
                ),
                ("fatal", Self::new_text_decoder_fatal_getter_callable()),
                (
                    "ignoreBOM",
                    Self::new_text_decoder_ignore_bom_getter_callable(),
                ),
            ] {
                Self::object_set_entry(
                    &mut prototype.borrow_mut(),
                    Self::object_getter_storage_key(property),
                    getter,
                );
                Self::mark_property_non_enumerable(prototype, property);
            }
        }
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            "decode".to_string(),
            Self::new_text_decoder_decode_callable(),
        );
        Self::mark_property_non_enumerable(prototype, "decode");
    }

    fn install_text_encoder_stream_prototype_surface(
        &mut self,
        prototype: &Rc<RefCell<ObjectValue>>,
    ) {
        let has_encoding_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "encoding")
        };
        if has_encoding_accessor {
            return;
        }
        for (property, getter) in [
            (
                "encoding",
                Self::new_text_encoder_stream_encoding_getter_callable(),
            ),
            (
                "readable",
                Self::new_text_encoder_stream_readable_getter_callable(),
            ),
            (
                "writable",
                Self::new_text_encoder_stream_writable_getter_callable(),
            ),
        ] {
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                Self::object_getter_storage_key(property),
                getter,
            );
            Self::mark_property_non_enumerable(prototype, property);
        }
    }

    fn install_text_decoder_stream_prototype_surface(
        &mut self,
        prototype: &Rc<RefCell<ObjectValue>>,
    ) {
        let has_encoding_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "encoding")
        };
        if has_encoding_accessor {
            return;
        }
        for (property, getter) in [
            (
                "encoding",
                Self::new_text_decoder_stream_encoding_getter_callable(),
            ),
            (
                "fatal",
                Self::new_text_decoder_stream_fatal_getter_callable(),
            ),
            (
                "ignoreBOM",
                Self::new_text_decoder_stream_ignore_bom_getter_callable(),
            ),
            (
                "readable",
                Self::new_text_decoder_stream_readable_getter_callable(),
            ),
            (
                "writable",
                Self::new_text_decoder_stream_writable_getter_callable(),
            ),
        ] {
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                Self::object_getter_storage_key(property),
                getter,
            );
            Self::mark_property_non_enumerable(prototype, property);
        }
    }

    fn cached_text_codec_constructor_value(
        &mut self,
        name: &str,
        callable_kind: &str,
        tag: &str,
        installer: fn(&mut Self, &Rc<RefCell<ObjectValue>>),
    ) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get(name)
            .cloned()
        {
            if let Value::Object(entries) = &constructor {
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    installer(self, &prototype);
                    Self::set_internal_prototype(
                        &prototype,
                        self.object_constructor_prototype_value(),
                    );
                }
            }
            return constructor;
        }
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor = Self::new_object_backed_constructor_with_prototype(callable_kind, vec![]);
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        installer(self, &prototype);
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String(tag.to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime
            .constructor_static_methods
            .insert(name.to_string(), constructor.clone());
        constructor
    }

    pub(crate) fn cached_text_encoder_constructor_value(&mut self) -> Value {
        self.cached_text_codec_constructor_value(
            "TextEncoder",
            "text_encoder_constructor",
            "TextEncoder",
            Self::install_text_encoder_prototype_surface,
        )
    }

    pub(crate) fn cached_text_decoder_constructor_value(&mut self) -> Value {
        self.cached_text_codec_constructor_value(
            "TextDecoder",
            "text_decoder_constructor",
            "TextDecoder",
            Self::install_text_decoder_prototype_surface,
        )
    }

    pub(crate) fn cached_text_encoder_stream_constructor_value(&mut self) -> Value {
        self.cached_text_codec_constructor_value(
            "TextEncoderStream",
            "text_encoder_stream_constructor",
            "TextEncoderStream",
            Self::install_text_encoder_stream_prototype_surface,
        )
    }

    pub(crate) fn cached_text_decoder_stream_constructor_value(&mut self) -> Value {
        self.cached_text_codec_constructor_value(
            "TextDecoderStream",
            "text_decoder_stream_constructor",
            "TextDecoderStream",
            Self::install_text_decoder_stream_prototype_surface,
        )
    }

    pub(crate) fn new_css_style_sheet_instance_value(owner_document: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CSS_STYLE_SHEET_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_CSS_STYLE_SHEET_OWNER_DOCUMENT_KEY.to_string(),
                owner_document,
            ),
            (
                INTERNAL_CSS_STYLE_SHEET_RULES_KEY.to_string(),
                Self::new_array_value(Vec::new()),
            ),
            (
                "replaceSync".to_string(),
                Self::new_css_style_sheet_replace_sync_callable(),
            ),
            (
                "insertRule".to_string(),
                Self::new_css_style_sheet_insert_rule_callable(),
            ),
        ])
    }

    pub(crate) fn is_css_style_sheet_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_CSS_STYLE_SHEET_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn css_style_sheet_owner_document(
        entries: &[(String, Value)],
    ) -> Option<Rc<RefCell<ObjectValue>>> {
        match Self::object_get_entry(entries, INTERNAL_CSS_STYLE_SHEET_OWNER_DOCUMENT_KEY) {
            Some(Value::Object(document)) => Some(document),
            _ => None,
        }
    }

    pub(crate) fn is_css_style_sheet_for_document(
        &self,
        value: &Value,
        document_object: &Rc<RefCell<ObjectValue>>,
    ) -> bool {
        let Value::Object(entries) = value else {
            return false;
        };
        let entries = entries.borrow();
        if !Self::is_css_style_sheet_object(&entries) {
            return false;
        }
        let Some(owner_document) = Self::css_style_sheet_owner_document(&entries) else {
            return false;
        };
        Rc::ptr_eq(&owner_document, document_object)
    }

    pub(crate) fn new_adopted_style_sheets_array_value(owner_document: Value) -> Value {
        let array = Self::new_array_value(Vec::new());
        if let Value::Array(values) = &array {
            let mut values_ref = values.borrow_mut();
            Self::object_set_entry(
                &mut values_ref.properties,
                INTERNAL_ADOPTED_STYLE_SHEETS_ARRAY_KEY.to_string(),
                Value::Bool(true),
            );
            Self::object_set_entry(
                &mut values_ref.properties,
                INTERNAL_ADOPTED_STYLE_SHEETS_OWNER_DOCUMENT_KEY.to_string(),
                owner_document,
            );
        }
        array
    }

    pub(crate) fn mark_as_adopted_style_sheets_array(
        &self,
        values: &Rc<RefCell<ArrayValue>>,
        owner_document: Value,
    ) {
        let mut values_ref = values.borrow_mut();
        Self::object_set_entry(
            &mut values_ref.properties,
            INTERNAL_ADOPTED_STYLE_SHEETS_ARRAY_KEY.to_string(),
            Value::Bool(true),
        );
        Self::object_set_entry(
            &mut values_ref.properties,
            INTERNAL_ADOPTED_STYLE_SHEETS_OWNER_DOCUMENT_KEY.to_string(),
            owner_document,
        );
    }

    pub(crate) fn adopted_style_sheets_owner_document(
        values: &ArrayValue,
    ) -> Option<Rc<RefCell<ObjectValue>>> {
        let is_adopted_array = matches!(
            Self::object_get_entry(&values.properties, INTERNAL_ADOPTED_STYLE_SHEETS_ARRAY_KEY),
            Some(Value::Bool(true))
        );
        if !is_adopted_array {
            return None;
        }
        match Self::object_get_entry(
            &values.properties,
            INTERNAL_ADOPTED_STYLE_SHEETS_OWNER_DOCUMENT_KEY,
        ) {
            Some(Value::Object(document)) => Some(document),
            _ => None,
        }
    }

    pub(crate) fn adopted_style_sheets_not_allowed_error() -> Error {
        Error::ScriptRuntime(
            "NotAllowedError: adoptedStyleSheets items must be CSSStyleSheet instances created in the same document".into(),
        )
    }

    pub(crate) fn ensure_document_adopted_style_sheets_property(&mut self) -> Value {
        if let Some(existing) = Self::object_get_entry(
            &self.dom_runtime.document_object.borrow(),
            "adoptedStyleSheets",
        ) {
            return existing;
        }
        let value = Self::new_adopted_style_sheets_array_value(Value::Object(
            self.dom_runtime.document_object.clone(),
        ));
        Self::object_set_entry(
            &mut self.dom_runtime.document_object.borrow_mut(),
            "adoptedStyleSheets".to_string(),
            value.clone(),
        );
        value
    }

    pub(crate) fn set_document_adopted_style_sheets_property(
        &mut self,
        value: Value,
    ) -> Result<()> {
        let Value::Array(values) = value else {
            return Err(Self::adopted_style_sheets_not_allowed_error());
        };
        let owner_document = self.dom_runtime.document_object.clone();
        for item in values.borrow().iter() {
            if !self.is_css_style_sheet_for_document(item, &owner_document) {
                return Err(Self::adopted_style_sheets_not_allowed_error());
            }
        }
        self.mark_as_adopted_style_sheets_array(
            &values,
            Value::Object(self.dom_runtime.document_object.clone()),
        );
        Self::object_set_entry(
            &mut self.dom_runtime.document_object.borrow_mut(),
            "adoptedStyleSheets".to_string(),
            Value::Array(values),
        );
        Ok(())
    }

    pub(crate) fn new_computed_style_object_value(node: NodeId, pseudo: Option<String>) -> Value {
        let value = Self::new_object_value(vec![
            (
                INTERNAL_COMPUTED_STYLE_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_COMPUTED_STYLE_TARGET_NODE_KEY.to_string(),
                Value::Node(node),
            ),
            (
                INTERNAL_COMPUTED_STYLE_PSEUDO_KEY.to_string(),
                pseudo.map(Value::String).unwrap_or(Value::Null),
            ),
            (
                "getPropertyValue".to_string(),
                Self::new_computed_style_get_property_value_callable(),
            ),
            ("item".to_string(), Self::new_computed_style_item_callable()),
        ]);
        let Value::Object(entries) = &value else {
            return value;
        };
        let mut entries = entries.borrow_mut();
        Self::mark_object_properties_non_enumerable(&mut *entries, &["getPropertyValue", "item"]);
        drop(entries);
        value
    }

    pub(crate) fn new_dom_rect_value(
        left: i64,
        top: i64,
        right: i64,
        bottom: i64,
        width: i64,
        height: i64,
    ) -> Value {
        let value = Self::new_object_value(vec![
            (INTERNAL_DOM_RECT_OBJECT_KEY.to_string(), Value::Bool(true)),
            ("x".to_string(), Value::Number(left)),
            ("y".to_string(), Value::Number(top)),
            ("left".to_string(), Value::Number(left)),
            ("top".to_string(), Value::Number(top)),
            ("right".to_string(), Value::Number(right)),
            ("bottom".to_string(), Value::Number(bottom)),
            ("width".to_string(), Value::Number(width)),
            ("height".to_string(), Value::Number(height)),
        ]);
        let Value::Object(entries) = &value else {
            return value;
        };
        let mut entries = entries.borrow_mut();
        for key in [
            "x", "y", "left", "top", "right", "bottom", "width", "height",
        ] {
            Self::object_set_entry(
                &mut *entries,
                Self::object_non_enumerable_storage_key(key),
                Value::Bool(true),
            );
            Self::object_set_entry(
                &mut *entries,
                Self::object_non_writable_storage_key(key),
                Value::Bool(true),
            );
            Self::object_set_entry(
                &mut *entries,
                Self::object_non_configurable_storage_key(key),
                Value::Bool(true),
            );
        }
        drop(entries);
        value
    }

    pub(crate) fn new_dom_rect_list_value(values: Vec<Value>) -> Value {
        let value = Self::new_array_value(values);
        let Value::Array(values) = &value else {
            return value;
        };
        let mut values = values.borrow_mut();
        Self::object_set_entry(
            &mut values.properties,
            INTERNAL_DOM_RECT_LIST_OBJECT_KEY.to_string(),
            Value::Bool(true),
        );
        Self::object_set_entry(
            &mut values.properties,
            "item".to_string(),
            Self::new_dom_rect_list_item_callable(),
        );
        Self::object_set_entry(
            &mut values.properties,
            Self::object_non_enumerable_storage_key("item"),
            Value::Bool(true),
        );
        drop(values);
        value
    }

    pub(crate) fn is_computed_style_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_COMPUTED_STYLE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_dom_rect_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_DOM_RECT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn computed_style_target_node(entries: &[(String, Value)]) -> Option<NodeId> {
        match Self::object_get_entry(entries, INTERNAL_COMPUTED_STYLE_TARGET_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }
    }

    pub(crate) fn computed_style_pseudo(entries: &[(String, Value)]) -> Option<String> {
        match Self::object_get_entry(entries, INTERNAL_COMPUTED_STYLE_PSEUDO_KEY) {
            Some(Value::String(pseudo)) => Some(pseudo),
            _ => None,
        }
    }

    fn computed_style_rule_value_from_style_nodes(
        &self,
        node: NodeId,
        pseudo: Option<&str>,
        property_name: &str,
    ) -> Option<String> {
        let mut resolved = None;
        for index in 0..self.dom.nodes.len() {
            let node_id = NodeId(index);
            let is_style_tag = self
                .dom
                .tag_name(node_id)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("style"));
            if !is_style_tag {
                continue;
            }
            let css_source = self.dom.text_content(node_id);
            for (selector_text, declarations_text) in Self::parse_css_rule_blocks(&css_source) {
                for selector in selector_text.split(',').map(str::trim) {
                    if selector.is_empty() {
                        continue;
                    }
                    let (base_selector, selector_pseudo) =
                        Self::split_selector_and_pseudo(selector);

                    let pseudo_matches = match (pseudo, selector_pseudo.as_deref()) {
                        (None, None) => true,
                        (Some(expected), Some(actual)) => actual.eq_ignore_ascii_case(expected),
                        _ => false,
                    };
                    if !pseudo_matches {
                        continue;
                    }

                    let selector_matches = if base_selector.is_empty() || base_selector == "*" {
                        true
                    } else {
                        matches!(
                            self.eval_matches_selector_value(node, base_selector),
                            Ok(Value::Bool(true))
                        )
                    };
                    if !selector_matches {
                        continue;
                    }

                    for (name, value) in parse_style_declarations(Some(declarations_text)) {
                        if name == property_name {
                            resolved = Some(value);
                        }
                    }
                }
            }
        }
        resolved
    }

    fn split_selector_and_pseudo(selector: &str) -> (&str, Option<String>) {
        let normalized = selector.trim();
        let Some(pseudo_pos) = normalized.find("::") else {
            return (normalized, None);
        };
        let base = normalized[..pseudo_pos].trim_end();
        let pseudo = normalized[pseudo_pos..].trim();
        (base, Some(pseudo.to_string()))
    }

    fn parse_css_rule_blocks(css_source: &str) -> Vec<(&str, &str)> {
        let bytes = css_source.as_bytes();
        let mut blocks = Vec::new();
        let mut cursor = 0usize;
        let mut selector_start = 0usize;
        while cursor < bytes.len() {
            if bytes[cursor] != b'{' {
                cursor += 1;
                continue;
            }
            let selector_end = cursor;
            cursor += 1;
            let declarations_start = cursor;
            let mut depth = 1usize;
            while cursor < bytes.len() && depth > 0 {
                match bytes[cursor] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                cursor += 1;
            }
            if depth != 0 || cursor == 0 {
                break;
            }
            let declarations_end = cursor.saturating_sub(1);
            let selector = css_source[selector_start..selector_end].trim();
            let declarations = css_source[declarations_start..declarations_end].trim();
            if !selector.is_empty() && !declarations.is_empty() {
                blocks.push((selector, declarations));
            }
            selector_start = cursor;
        }
        blocks
    }

    pub(crate) fn computed_style_property_value(
        &self,
        node: NodeId,
        pseudo: Option<&str>,
        property_name: &str,
    ) -> Result<String> {
        if self.dom.element(node).is_none() {
            return Err(Error::ScriptRuntime(
                "TypeError: getComputedStyle target must be an Element".into(),
            ));
        }
        let css_property = js_prop_to_css_name(property_name.trim());

        if pseudo.is_none() {
            let inline = self.dom.style_get(node, &css_property)?;
            if !inline.is_empty() {
                return Ok(inline);
            }
        }

        if let Some(from_rules) =
            self.computed_style_rule_value_from_style_nodes(node, pseudo, &css_property)
        {
            return Ok(from_rules);
        }

        Ok(String::new())
    }

    pub(crate) fn computed_style_object_property_from_entries(
        &self,
        entries: &[(String, Value)],
        key: &str,
    ) -> Result<Option<Value>> {
        if !Self::is_computed_style_object(entries) {
            return Ok(None);
        }

        if self.is_to_string_tag_property_key(key) {
            return Ok(Some(Value::String("CSSStyleDeclaration".to_string())));
        }

        match key {
            "getPropertyValue" | "item" => Ok(Some(
                Self::object_get_entry(entries, key).unwrap_or(Value::Undefined),
            )),
            "setProperty" | "removeProperty" => Ok(Some(Self::new_builtin_placeholder_function())),
            "cssText" => Ok(Some(Value::String(String::new()))),
            "length" => Ok(Some(Value::Number(0))),
            "parentRule" => Ok(Some(Value::Null)),
            "constructor" => Ok(Some(Value::Undefined)),
            _ => {
                let reserved = matches!(
                    key,
                    "__proto__"
                        | "toString"
                        | "valueOf"
                        | "hasOwnProperty"
                        | "isPrototypeOf"
                        | "propertyIsEnumerable"
                );
                if reserved {
                    return Ok(None);
                }
                let Some(node) = Self::computed_style_target_node(entries) else {
                    return Ok(Some(Value::Undefined));
                };
                let pseudo = Self::computed_style_pseudo(entries);
                let value = self.computed_style_property_value(node, pseudo.as_deref(), key)?;
                Ok(Some(Value::String(value)))
            }
        }
    }

    pub(crate) fn new_worker_context_post_message_callable(worker: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("worker_context_post_message".to_string()),
            ),
            (INTERNAL_WORKER_TARGET_KEY.to_string(), worker),
        ])
    }

    pub(crate) fn new_worker_main_post_message_callable(worker: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("worker_main_post_message".to_string()),
            ),
            (INTERNAL_WORKER_TARGET_KEY.to_string(), worker),
        ])
    }

    pub(crate) fn new_worker_terminate_callable(worker: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("worker_terminate".to_string()),
            ),
            (INTERNAL_WORKER_TARGET_KEY.to_string(), worker),
        ])
    }

    pub(crate) fn new_intl_collator_compare_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("intl_collator_get_compare".to_string()),
        )])
    }

    pub(crate) fn new_intl_date_time_format_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("intl_date_time_format_get_format".to_string()),
        )])
    }

    pub(crate) fn new_intl_number_format_getter_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("intl_number_format_get_format".to_string()),
        )])
    }

    pub(crate) fn new_global_decode_uri_callable(component: bool) -> Value {
        let kind = if component {
            "global_decode_uri_component"
        } else {
            "global_decode_uri"
        };
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String(kind.to_string()),
        )])
    }

    pub(crate) fn new_global_atob_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_atob".to_string()),
        )])
    }

    pub(crate) fn new_global_btoa_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_btoa".to_string()),
        )])
    }

    pub(crate) fn new_global_structured_clone_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_structured_clone".to_string()),
        )])
    }

    pub(crate) fn new_global_css_escape_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_css_escape".to_string()),
        )])
    }

    pub(crate) fn new_global_request_animation_frame_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_request_animation_frame".to_string()),
        )])
    }

    pub(crate) fn new_global_set_timeout_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_set_timeout".to_string()),
        )])
    }

    pub(crate) fn new_global_set_interval_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_set_interval".to_string()),
        )])
    }

    pub(crate) fn new_global_cancel_animation_frame_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_cancel_animation_frame".to_string()),
        )])
    }

    pub(crate) fn new_global_clear_interval_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_clear_interval".to_string()),
        )])
    }

    pub(crate) fn new_global_clear_timeout_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_clear_timeout".to_string()),
        )])
    }

    pub(crate) fn new_global_queue_microtask_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("global_queue_microtask".to_string()),
        )])
    }

    pub(crate) fn new_create_image_bitmap_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("create_image_bitmap".to_string()),
        )])
    }

    pub(crate) fn new_dom_parser_instance_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_DOM_PARSER_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )])
    }

    pub(crate) fn new_xml_serializer_instance_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_XML_SERIALIZER_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )])
    }

    pub(crate) fn new_function_call_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("function_call".to_string()),
        )])
    }

    pub(crate) fn new_function_apply_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("function_apply".to_string()),
        )])
    }

    pub(crate) fn new_function_bind_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("function_bind".to_string()),
        )])
    }

    pub(crate) fn new_function_to_string_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("function_to_string".to_string()),
        )])
    }

    pub(crate) fn new_string_static_from_char_code_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("string_static_from_char_code".to_string()),
        )])
    }

    pub(crate) fn new_string_static_from_code_point_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("string_static_from_code_point".to_string()),
        )])
    }

    pub(crate) fn new_string_static_raw_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("string_static_raw".to_string()),
        )])
    }

    fn new_number_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("number_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    fn new_object_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("object_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    fn new_reflect_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("reflect_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    fn new_bigint_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("bigint_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    fn new_regexp_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("regexp_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    fn new_promise_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("promise_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    fn new_array_buffer_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("array_buffer_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    fn new_symbol_static_method_callable(method: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("symbol_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    fn new_typed_array_static_method_callable(
        kind: TypedArrayConstructorKind,
        method: &str,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("typed_array_static_method".to_string()),
            ),
            (
                INTERNAL_STATIC_TYPED_ARRAY_KIND_KEY.to_string(),
                Value::TypedArrayConstructor(kind),
            ),
            (
                INTERNAL_STATIC_METHOD_NAME_KEY.to_string(),
                Value::String(method.to_string()),
            ),
        ])
    }

    pub(crate) fn new_bound_function_callable(
        target: Value,
        bound_this: Value,
        bound_args: Vec<Value>,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("bound_function".to_string()),
            ),
            (INTERNAL_BOUND_CALLABLE_TARGET_KEY.to_string(), target),
            (INTERNAL_BOUND_CALLABLE_THIS_KEY.to_string(), bound_this),
            (
                INTERNAL_BOUND_CALLABLE_ARGS_KEY.to_string(),
                Self::new_array_value(bound_args),
            ),
            ("call".to_string(), Self::new_function_call_callable()),
            ("apply".to_string(), Self::new_function_apply_callable()),
            ("bind".to_string(), Self::new_function_bind_callable()),
        ])
    }

    pub(crate) fn new_receiver_builtin_callable(family: &str, member: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("receiver_builtin_method".to_string()),
            ),
            (
                "__bt_receiver_builtin_family".to_string(),
                Value::String(family.to_string()),
            ),
            (
                "__bt_receiver_builtin_member".to_string(),
                Value::String(member.to_string()),
            ),
        ])
    }

    fn document_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(
            key,
            "createElement"
                | "createElementNS"
                | "createTextNode"
                | "createAttribute"
                | "createDocumentFragment"
                | "createRange"
                | "getSelection"
                | "append"
                | "getElementById"
                | "getElementsByClassName"
                | "getElementsByName"
                | "getElementsByTagName"
                | "getElementsByTagNameNS"
                | "querySelector"
                | "querySelectorAll"
                | "createTreeWalker"
                | "addEventListener"
                | "removeEventListener"
        ) {
            Some(Self::new_receiver_builtin_callable("document", key))
        } else {
            None
        }
    }

    fn node_receiver_builtin_method(&self, node: NodeId, key: &str) -> Option<Value> {
        let node_type = self.node_type_number(node);
        let is_parent_node = matches!(node_type, 1 | 9 | 11);
        let is_child_node = matches!(node_type, 1 | 3 | 8 | 10);
        let is_element = node_type == 1;

        if matches!(
            key,
            "appendChild"
                | "insertBefore"
                | "removeChild"
                | "replaceChild"
                | "hasChildNodes"
                | "contains"
                | "getRootNode"
                | "compareDocumentPosition"
                | "isEqualNode"
                | "isSameNode"
                | "normalize"
                | "isDefaultNamespace"
                | "lookupPrefix"
                | "lookupNamespaceURI"
                | "cloneNode"
        ) {
            return Some(Self::new_receiver_builtin_callable("node", key));
        }

        if is_parent_node
            && matches!(
                key,
                "append" | "prepend" | "replaceChildren" | "querySelector" | "querySelectorAll"
            )
        {
            return Some(Self::new_receiver_builtin_callable("node", key));
        }

        if is_child_node && matches!(key, "before" | "after" | "replaceWith" | "remove") {
            return Some(Self::new_receiver_builtin_callable("node", key));
        }

        if is_element
            && matches!(
                key,
                "getAttributeNames"
                    | "toggleAttribute"
                    | "matches"
                    | "closest"
                    | "insertAdjacentElement"
                    | "insertAdjacentHTML"
                    | "insertAdjacentText"
                    | "setHTMLUnsafe"
            )
        {
            return Some(Self::new_receiver_builtin_callable("node", key));
        }

        None
    }

    pub(crate) fn parsed_document_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(
            key,
            "createTreeWalker"
                | "querySelector"
                | "querySelectorAll"
                | "getElementById"
                | "getElementsByClassName"
                | "getElementsByName"
                | "getElementsByTagName"
                | "createElement"
                | "createElementNS"
                | "createTextNode"
                | "createAttribute"
                | "createDocumentFragment"
                | "createRange"
                | "append"
        ) {
            Some(Self::new_receiver_builtin_callable("parsed_document", key))
        } else {
            None
        }
    }

    pub(crate) fn dom_parser_receiver_builtin_method(key: &str) -> Option<Value> {
        if key == "parseFromString" {
            Some(Self::new_receiver_builtin_callable("dom_parser", key))
        } else {
            None
        }
    }

    pub(crate) fn xml_serializer_receiver_builtin_method(key: &str) -> Option<Value> {
        if key == "serializeToString" {
            Some(Self::new_receiver_builtin_callable("xml_serializer", key))
        } else {
            None
        }
    }

    pub(crate) fn tree_walker_receiver_builtin_method(key: &str) -> Option<Value> {
        if key == "nextNode" {
            Some(Self::new_receiver_builtin_callable("tree_walker", key))
        } else {
            None
        }
    }

    fn range_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(key, "setStart" | "setEnd") {
            Some(Self::new_receiver_builtin_callable("range", key))
        } else {
            None
        }
    }

    fn selection_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(
            key,
            "addRange"
                | "collapse"
                | "collapseToEnd"
                | "collapseToStart"
                | "containsNode"
                | "deleteFromDocument"
                | "empty"
                | "extend"
                | "getComposedRanges"
                | "getRangeAt"
                | "modify"
                | "removeAllRanges"
                | "removeRange"
                | "selectAllChildren"
                | "setBaseAndExtent"
                | "setPosition"
                | "toString"
        ) {
            Some(Self::new_receiver_builtin_callable("selection", key))
        } else {
            None
        }
    }

    fn event_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(
            key,
            "preventDefault" | "stopPropagation" | "stopImmediatePropagation"
        ) {
            Some(Self::new_receiver_builtin_callable("event", key))
        } else {
            None
        }
    }

    fn keyboard_event_receiver_builtin_method(key: &str) -> Option<Value> {
        if key == "getModifierState" {
            Some(Self::new_receiver_builtin_callable("keyboard_event", key))
        } else {
            None
        }
    }

    fn pointer_event_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(key, "getCoalescedEvents" | "getPredictedEvents") {
            Some(Self::new_receiver_builtin_callable("pointer_event", key))
        } else {
            None
        }
    }

    fn event_target_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(key, "addEventListener" | "removeEventListener" | "dispatchEvent") {
            Some(Self::new_receiver_builtin_callable("event_target", key))
        } else {
            None
        }
    }

    fn navigate_event_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(key, "intercept" | "scroll") {
            Some(Self::new_receiver_builtin_callable("navigate_event", key))
        } else {
            None
        }
    }

    fn data_transfer_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(
            key,
            "getData" | "setData" | "clearData" | "setDragImage" | "addElement"
        ) {
            Some(Self::new_receiver_builtin_callable("data_transfer", key))
        } else {
            None
        }
    }

    fn data_transfer_item_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(
            key,
            "getAsFile" | "getAsFileSystemHandle" | "getAsString" | "webkitGetAsEntry"
        ) {
            Some(Self::new_receiver_builtin_callable(
                "data_transfer_item",
                key,
            ))
        } else {
            None
        }
    }

    fn data_transfer_item_list_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(key, "add" | "remove" | "clear") {
            Some(Self::new_receiver_builtin_callable(
                "data_transfer_item_list",
                key,
            ))
        } else {
            None
        }
    }

    fn placeholder_backed_object_receiver_builtin_method(
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if Self::is_event_object(entries)
            && let Some(value) = Self::event_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_keyboard_event_object(entries)
            && let Some(value) = Self::keyboard_event_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_pointer_event_object(entries)
            && let Some(value) = Self::pointer_event_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_navigate_event_object(entries)
            && let Some(value) = Self::navigate_event_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if (Self::is_data_transfer_object(entries) || Self::is_clipboard_data_object(entries))
            && let Some(value) = Self::data_transfer_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_data_transfer_item_object(entries)
            && let Some(value) = Self::data_transfer_item_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_document_object(entries)
            && let Some(value) = Self::document_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_PARSED_DOCUMENT_OBJECT_KEY),
            Some(Value::Bool(true))
        ) && let Some(value) = Self::parsed_document_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_DOM_PARSER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) && let Some(value) = Self::dom_parser_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_XML_SERIALIZER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) && let Some(value) = Self::xml_serializer_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_TREE_WALKER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) && let Some(value) = Self::tree_walker_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_range_object(entries)
            && let Some(value) = Self::range_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_selection_object(entries)
            && let Some(value) = Self::selection_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_event_target_object(entries)
            && let Some(value) = Self::event_target_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_match_media_object(entries)
            && let Some(value) = Self::match_media_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_cookie_store_object(entries)
            && let Some(value) = Self::cookie_store_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_cache_storage_object(entries)
            && let Some(value) = Self::cache_storage_receiver_builtin_method(key)
        {
            return Some(value);
        }
        if Self::is_cache_object(entries)
            && let Some(value) = Self::cache_receiver_builtin_method(key)
        {
            return Some(value);
        }
        None
    }

    pub(crate) fn placeholder_backed_object_builtin_property_value(
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if let Some(value) = Self::object_get_entry(entries, key)
            && !Self::is_builtin_placeholder_value(&value)
        {
            return Some(value);
        }
        let builtin = Self::placeholder_backed_object_receiver_builtin_method(entries, key)?;
        if Self::is_builtin_object_property_deleted(entries, key) {
            return None;
        }
        Some(builtin)
    }

    pub(crate) fn placeholder_backed_object_builtin_surface_exists(
        entries: &ObjectValue,
        key: &str,
    ) -> bool {
        Self::placeholder_backed_object_receiver_builtin_method(entries, key).is_some()
    }

    pub(crate) fn placeholder_backed_object_builtin_is_shadowed(
        entries: &ObjectValue,
        key: &str,
    ) -> bool {
        Self::placeholder_backed_object_builtin_surface_exists(entries, key)
            && (Self::object_get_entry(entries, key)
                .is_some_and(|value| !Self::is_builtin_placeholder_value(&value))
                || Self::is_builtin_object_property_deleted(entries, key))
    }

    pub(crate) fn placeholder_backed_array_builtin_property_value(
        values: &ArrayValue,
        key: &str,
    ) -> Option<Value> {
        if let Some(value) = Self::object_get_entry(&values.properties, key)
            && !Self::is_builtin_placeholder_value(&value)
        {
            return Some(value);
        }
        if Self::is_builtin_object_property_deleted(&values.properties, key) {
            return None;
        }
        if Self::is_data_transfer_item_list_value(values) {
            return Self::data_transfer_item_list_receiver_builtin_method(key);
        }
        if Self::is_dom_rect_list_value(values) {
            return Self::dom_rect_list_receiver_builtin_method(key);
        }
        None
    }

    pub(crate) fn placeholder_backed_array_builtin_surface_exists(
        values: &ArrayValue,
        key: &str,
    ) -> bool {
        (Self::data_transfer_item_list_receiver_builtin_method(key).is_some()
            && Self::is_data_transfer_item_list_value(values))
            || (Self::dom_rect_list_receiver_builtin_method(key).is_some()
                && Self::is_dom_rect_list_value(values))
    }

    fn dom_rect_list_receiver_builtin_method(key: &str) -> Option<Value> {
        if key == "item" {
            Some(Self::new_dom_rect_list_item_callable())
        } else {
            None
        }
    }

    fn match_media_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(
            key,
            "addEventListener"
                | "removeEventListener"
                | "dispatchEvent"
                | "addListener"
                | "removeListener"
        ) {
            Some(Self::new_receiver_builtin_callable("match_media", key))
        } else {
            None
        }
    }

    fn cookie_store_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(
            key,
            "set" | "get" | "getAll" | "delete" | "addEventListener" | "removeEventListener"
        ) {
            Some(Self::new_receiver_builtin_callable("cookie_store", key))
        } else {
            None
        }
    }

    fn cache_storage_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(key, "open" | "match" | "has" | "delete" | "keys") {
            Some(Self::new_receiver_builtin_callable("cache_storage", key))
        } else {
            None
        }
    }

    fn cache_receiver_builtin_method(key: &str) -> Option<Value> {
        if matches!(key, "match" | "put" | "delete" | "keys" | "add" | "addAll") {
            Some(Self::new_receiver_builtin_callable("cache", key))
        } else {
            None
        }
    }

    pub(crate) fn is_builtin_placeholder_value(value: &Value) -> bool {
        matches!(value, Value::Function(function) if function.function_id == usize::MAX)
    }

    fn new_receiver_builtin_prototype_value(
        constructor: Value,
        family: &str,
        methods: &[&str],
    ) -> Value {
        let mut entries = vec![("constructor".to_string(), constructor)];
        for method in methods {
            entries.push((
                (*method).to_string(),
                Self::new_receiver_builtin_callable(family, method),
            ));
        }
        let prototype = Self::new_object_value(entries);
        if let Value::Object(entries) = &prototype {
            Self::mark_existing_public_properties_non_enumerable(entries);
        }
        prototype
    }

    fn new_receiver_builtin_prototype_with_iterator_value(
        &mut self,
        constructor: Value,
        family: &str,
        methods: &[&str],
        iterator_member: Option<&str>,
    ) -> Value {
        let prototype = Self::new_receiver_builtin_prototype_value(constructor, family, methods);
        let Some(iterator_member) = iterator_member else {
            return prototype;
        };
        let Value::Object(entries) = &prototype else {
            return prototype;
        };
        let iterator_symbol = self.eval_symbol_static_property(SymbolStaticProperty::Iterator);
        let iterator_key = self.property_key_to_storage_key(&iterator_symbol);
        Self::object_set_entry(
            &mut entries.borrow_mut(),
            iterator_key,
            Self::new_receiver_builtin_callable(family, iterator_member),
        );
        prototype
    }

    pub(crate) fn set_internal_prototype(entries: &Rc<RefCell<ObjectValue>>, prototype: Value) {
        Self::object_set_entry(
            &mut entries.borrow_mut(),
            INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
            prototype,
        );
    }

    pub(crate) fn mark_property_non_enumerable(
        entries: &Rc<RefCell<ObjectValue>>,
        property_key: &str,
    ) {
        Self::object_set_entry(
            &mut entries.borrow_mut(),
            Self::object_non_enumerable_storage_key(property_key),
            Value::Bool(true),
        );
    }

    pub(crate) fn mark_existing_public_properties_non_enumerable(
        entries: &Rc<RefCell<ObjectValue>>,
    ) {
        let keys = entries
            .borrow()
            .iter()
            .filter(|(key, _)| !Self::is_internal_object_key(key))
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        for key in keys {
            Self::mark_property_non_enumerable(entries, &key);
        }
    }

    pub(crate) fn mark_constructor_non_enumerable(entries: &Rc<RefCell<ObjectValue>>) {
        Self::mark_property_non_enumerable(entries, "constructor");
        Self::object_set_entry(
            &mut entries.borrow_mut(),
            INTERNAL_NON_ENUMERABLE_CONSTRUCTOR_KEY.to_string(),
            Value::Bool(true),
        );
    }

    pub(crate) fn constructor_prototype_from_value(
        &mut self,
        constructor: &Value,
    ) -> Option<Value> {
        match self.object_property_from_value(constructor, "prototype") {
            Ok(Value::Object(prototype)) => Some(Value::Object(prototype)),
            _ => None,
        }
    }

    fn constructor_prototype_from_env(&mut self, name: &str) -> Option<Value> {
        let constructor = self.script_runtime.env.get(name).cloned()?;
        self.constructor_prototype_from_value(&constructor)
    }

    fn object_constructor_prototype_value(&mut self) -> Value {
        self.constructor_prototype_from_env("Object")
            .unwrap_or_else(|| Self::new_object_value(Vec::new()))
    }

    pub(crate) fn cached_function_constructor_value(&mut self) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get("Function")
            .cloned()
        {
            if let Value::Object(constructor_entries) = &constructor {
                let prototype = {
                    let entries = constructor_entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    Self::mark_existing_public_properties_non_enumerable(&prototype);
                    Self::mark_existing_public_properties_non_enumerable(constructor_entries);
                    Self::set_internal_prototype(
                        &prototype,
                        self.object_constructor_prototype_value(),
                    );
                    Self::set_internal_prototype(constructor_entries, Value::Object(prototype));
                }
            }
            return constructor;
        }

        let prototype = Rc::new(RefCell::new(ObjectValue::default()));
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("function_constructor".to_string()),
            ),
            ("prototype".to_string(), Value::Object(prototype.clone())),
        ]);
        if let Value::Object(constructor_entries) = &constructor {
            Self::set_internal_prototype(constructor_entries, Value::Object(prototype.clone()));
        }

        {
            let mut prototype_entries = prototype.borrow_mut();
            Self::object_set_entry(
                &mut prototype_entries,
                "constructor".to_string(),
                constructor.clone(),
            );
            for method in ["call", "apply", "bind", "toString"] {
                Self::object_set_entry(
                    &mut prototype_entries,
                    method.to_string(),
                    self.cached_function_surface_method_value(method),
                );
            }
        }
        Self::mark_existing_public_properties_non_enumerable(&prototype);
        if let Value::Object(constructor_entries) = &constructor {
            Self::mark_existing_public_properties_non_enumerable(constructor_entries);
        }
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());

        self.script_runtime
            .builtin_constructor_prototypes
            .insert("Function".to_string(), prototype);
        self.script_runtime
            .constructor_static_methods
            .insert("Function".to_string(), constructor.clone());
        constructor
    }

    pub(crate) fn cached_function_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("Function")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let _ = self.cached_function_constructor_value();
        self.script_runtime
            .builtin_constructor_prototypes
            .get("Function")
            .cloned()
            .map(Value::Object)
            .unwrap_or_else(|| Self::new_object_value(Vec::new()))
    }

    pub(crate) fn function_family_constructor_bindings(&mut self) -> Vec<(String, Value)> {
        vec![
            (
                "Function".to_string(),
                self.cached_function_constructor_value(),
            ),
            (
                "GeneratorFunction".to_string(),
                self.new_generator_function_constructor_value(),
            ),
            (
                "AsyncGeneratorFunction".to_string(),
                self.new_async_generator_function_constructor_value(),
            ),
        ]
    }

    pub(crate) fn sync_function_prototype_object(&mut self, function: &Rc<FunctionValue>) {
        if function.is_arrow || function.is_method {
            return;
        }
        Self::mark_constructor_non_enumerable(&function.prototype_object);
        let mut prototype = function.prototype_object.borrow_mut();
        if Self::object_get_entry(&*prototype, INTERNAL_OBJECT_PROTOTYPE_KEY).is_none() {
            Self::object_set_entry(
                &mut *prototype,
                INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                self.object_constructor_prototype_value(),
            );
        }
        Self::object_set_entry(
            &mut *prototype,
            "constructor".to_string(),
            Value::Function(function.clone()),
        );
    }

    fn typed_array_constructor_cache_key(kind: &TypedArrayConstructorKind) -> String {
        match kind {
            TypedArrayConstructorKind::Concrete(kind) => kind.name().to_string(),
            TypedArrayConstructorKind::Abstract => "TypedArray".to_string(),
        }
    }

    fn cached_constructor_static_method_value(
        &mut self,
        cache_key: &str,
        make_value: impl FnOnce() -> Value,
    ) -> Value {
        if let Some(value) = self
            .script_runtime
            .constructor_static_methods
            .get(cache_key)
            .cloned()
        {
            return value;
        }
        let value = make_value();
        self.script_runtime
            .constructor_static_methods
            .insert(cache_key.to_string(), value.clone());
        value
    }

    fn cached_function_surface_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("Function.prototype.{method}"), || {
            match method {
                "call" => Self::new_function_call_callable(),
                "apply" => Self::new_function_apply_callable(),
                "bind" => Self::new_function_bind_callable(),
                "toString" => Self::new_function_to_string_callable(),
                _ => Value::Undefined,
            }
        })
    }

    fn cached_string_static_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("String.{method}"), || match method {
            "fromCharCode" => Self::new_string_static_from_char_code_callable(),
            "fromCodePoint" => Self::new_string_static_from_code_point_callable(),
            "raw" => Self::new_string_static_raw_callable(),
            _ => Value::Undefined,
        })
    }

    fn cached_collection_like_constructor_value(
        &mut self,
        name: &str,
        callable_kind: &str,
        family: &str,
        methods: &[&str],
        prototype_parent: Option<Value>,
    ) -> Value {
        let iterator_symbol = self.eval_symbol_static_property(SymbolStaticProperty::Iterator);
        let iterator_key = self.property_key_to_storage_key(&iterator_symbol);
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let prototype_parent =
            prototype_parent.unwrap_or_else(|| self.object_constructor_prototype_value());
        self.cached_constructor_static_method_value(name, || {
            let constructor =
                Self::new_receiver_builtin_constructor_object(Some(callable_kind), family, methods);
            let Value::Object(constructor_entries) = &constructor else {
                return constructor;
            };
            let prototype = {
                let entries = constructor_entries.borrow();
                Self::object_get_entry(&entries, "prototype")
            };
            let Some(Value::Object(prototype)) = prototype else {
                return constructor;
            };
            if !methods.is_empty() {
                Self::object_set_entry(
                    &mut prototype.borrow_mut(),
                    iterator_key.clone(),
                    Self::new_receiver_builtin_callable(family, "values"),
                );
            }
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                to_string_tag_key.clone(),
                Value::String(name.to_string()),
            );
            Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
            Self::set_internal_prototype(&prototype, prototype_parent.clone());
            constructor
        })
    }

    pub(crate) fn cached_node_list_constructor_value(&mut self) -> Value {
        self.cached_collection_like_constructor_value(
            "NodeList",
            "node_list_constructor",
            "node_list",
            &["item", "forEach", "entries", "keys", "values"],
            None,
        )
    }

    fn cached_node_list_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("NodeList")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let constructor = self.cached_node_list_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("NodeList".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_text_track_list_constructor_value(&mut self) -> Value {
        let parent = self.cached_node_list_constructor_prototype_value();
        self.cached_collection_like_constructor_value(
            "TextTrackList",
            "text_track_list_constructor",
            "node_list",
            &[],
            Some(parent),
        )
    }

    fn cached_text_track_list_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("TextTrackList")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let constructor = self.cached_text_track_list_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("TextTrackList".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_text_track_constructor_value(&mut self) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get("TextTrack")
            .cloned()
        {
            if let Value::Object(entries) = &constructor {
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    self.install_text_track_prototype_accessors(&prototype);
                    Self::set_internal_prototype(
                        &prototype,
                        self.object_constructor_prototype_value(),
                    );
                }
            }
            return constructor;
        }
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("text_track_constructor"),
            "text_track",
            &[],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        self.install_text_track_prototype_accessors(&prototype);
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String("TextTrack".to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime
            .constructor_static_methods
            .insert("TextTrack".to_string(), constructor.clone());
        constructor
    }

    fn install_text_track_prototype_accessors(&mut self, prototype: &Rc<RefCell<ObjectValue>>) {
        let has_mode_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "mode")
        };
        if has_mode_accessor {
            return;
        }
        for (property, getter) in [
            ("id", "id_get"),
            ("kind", "kind_get"),
            ("label", "label_get"),
            ("language", "language_get"),
            ("cues", "cues_get"),
            ("activeCues", "active_cues_get"),
            (
                "inBandMetadataTrackDispatchType",
                "in_band_metadata_track_dispatch_type_get",
            ),
        ] {
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                Self::object_getter_storage_key(property),
                Self::new_receiver_builtin_callable("text_track", getter),
            );
            Self::mark_property_non_enumerable(prototype, property);
        }
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            Self::object_getter_storage_key("mode"),
            Self::new_receiver_builtin_callable("text_track", "mode_get"),
        );
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            Self::object_setter_storage_key("mode"),
            Self::new_receiver_builtin_callable("text_track", "mode_set"),
        );
        Self::mark_property_non_enumerable(prototype, "mode");
    }

    fn cached_text_track_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("TextTrack")
            .cloned()
        {
            self.install_text_track_prototype_accessors(&prototype);
            Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
            return Value::Object(prototype);
        }
        let constructor = self.cached_text_track_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.install_text_track_prototype_accessors(&prototype);
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("TextTrack".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_image_bitmap_constructor_value(&mut self) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get("ImageBitmap")
            .cloned()
        {
            if let Value::Object(entries) = &constructor {
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    self.install_image_bitmap_prototype_accessors(&prototype);
                    Self::set_internal_prototype(
                        &prototype,
                        self.object_constructor_prototype_value(),
                    );
                }
            }
            return constructor;
        }
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("image_bitmap_constructor"),
            "image_bitmap",
            &["close"],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        self.install_image_bitmap_prototype_accessors(&prototype);
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String("ImageBitmap".to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime
            .constructor_static_methods
            .insert("ImageBitmap".to_string(), constructor.clone());
        constructor
    }

    fn install_image_bitmap_prototype_accessors(&mut self, prototype: &Rc<RefCell<ObjectValue>>) {
        let has_width_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "width")
        };
        if has_width_accessor {
            return;
        }
        for property in ["width", "height"] {
            Self::object_set_entry(
                &mut prototype.borrow_mut(),
                Self::object_getter_storage_key(property),
                Self::new_receiver_builtin_callable(
                    "image_bitmap",
                    if property == "width" {
                        "width_get"
                    } else {
                        "height_get"
                    },
                ),
            );
            Self::mark_property_non_enumerable(prototype, property);
        }
    }

    fn cached_image_bitmap_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("ImageBitmap")
            .cloned()
        {
            self.install_image_bitmap_prototype_accessors(&prototype);
            Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
            return Value::Object(prototype);
        }
        let constructor = self.cached_image_bitmap_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.install_image_bitmap_prototype_accessors(&prototype);
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("ImageBitmap".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_time_ranges_constructor_value(&mut self) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get("TimeRanges")
            .cloned()
        {
            if let Value::Object(entries) = &constructor {
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    self.install_time_ranges_prototype_accessors(&prototype);
                    Self::set_internal_prototype(
                        &prototype,
                        self.object_constructor_prototype_value(),
                    );
                }
            }
            return constructor;
        }
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor = Self::new_receiver_builtin_constructor_object(
            Some("time_ranges_constructor"),
            "time_ranges",
            &["start", "end"],
        );
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        self.install_time_ranges_prototype_accessors(&prototype);
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String("TimeRanges".to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime
            .constructor_static_methods
            .insert("TimeRanges".to_string(), constructor.clone());
        constructor
    }

    fn install_time_ranges_prototype_accessors(&mut self, prototype: &Rc<RefCell<ObjectValue>>) {
        let has_length_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "length")
        };
        if has_length_accessor {
            return;
        }
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            Self::object_getter_storage_key("length"),
            Self::new_receiver_builtin_callable("time_ranges", "length_get"),
        );
        Self::mark_property_non_enumerable(prototype, "length");
    }

    fn cached_time_ranges_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("TimeRanges")
            .cloned()
        {
            self.install_time_ranges_prototype_accessors(&prototype);
            Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
            return Value::Object(prototype);
        }
        let constructor = self.cached_time_ranges_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.install_time_ranges_prototype_accessors(&prototype);
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("TimeRanges".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    fn cached_placeholder_backed_interface_constructor_value(
        &mut self,
        interface_name: &str,
        callable_kind: &str,
        to_string_tag: &str,
    ) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get(interface_name)
            .cloned()
        {
            if let Value::Object(entries) = &constructor {
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    Self::set_internal_prototype(
                        &prototype,
                        self.object_constructor_prototype_value(),
                    );
                }
            }
            return constructor;
        }

        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor =
            Self::new_receiver_builtin_constructor_object(Some(callable_kind), callable_kind, &[]);
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String(to_string_tag.to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime
            .constructor_static_methods
            .insert(interface_name.to_string(), constructor.clone());
        constructor
    }

    fn cached_placeholder_backed_interface_constructor_prototype_value(
        &mut self,
        interface_name: &str,
        callable_kind: &str,
        to_string_tag: &str,
    ) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get(interface_name)
            .cloned()
        {
            Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
            return Value::Object(prototype);
        }
        let constructor = self.cached_placeholder_backed_interface_constructor_value(
            interface_name,
            callable_kind,
            to_string_tag,
        );
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert(interface_name.to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_storage_constructor_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_value(
            "Storage",
            "storage_constructor",
            "Storage",
        )
    }

    pub(crate) fn cached_cookie_store_constructor_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_value(
            "CookieStore",
            "cookie_store_constructor",
            "CookieStore",
        )
    }

    pub(crate) fn cached_cache_storage_constructor_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_value(
            "CacheStorage",
            "cache_storage_constructor",
            "CacheStorage",
        )
    }

    pub(crate) fn cached_cache_constructor_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_value(
            "Cache",
            "cache_constructor",
            "Cache",
        )
    }

    pub(crate) fn cached_storage_constructor_prototype_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_prototype_value(
            "Storage",
            "storage_constructor",
            "Storage",
        )
    }

    pub(crate) fn cached_cookie_store_constructor_prototype_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_prototype_value(
            "CookieStore",
            "cookie_store_constructor",
            "CookieStore",
        )
    }

    pub(crate) fn cached_cache_storage_constructor_prototype_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_prototype_value(
            "CacheStorage",
            "cache_storage_constructor",
            "CacheStorage",
        )
    }

    pub(crate) fn cached_cache_constructor_prototype_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_prototype_value(
            "Cache",
            "cache_constructor",
            "Cache",
        )
    }

    pub(crate) fn cached_html_collection_constructor_value(&mut self) -> Value {
        self.cached_collection_like_constructor_value(
            "HTMLCollection",
            "html_collection_constructor",
            "html_collection",
            &["item", "namedItem", "forEach", "entries", "keys", "values"],
            None,
        )
    }

    fn cached_html_collection_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("HTMLCollection")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let constructor = self.cached_html_collection_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("HTMLCollection".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_radio_node_list_constructor_value(&mut self) -> Value {
        let parent = self.cached_node_list_constructor_prototype_value();
        let constructor = self.cached_collection_like_constructor_value(
            "RadioNodeList",
            "radio_node_list_constructor",
            "node_list",
            &["item", "forEach", "entries", "keys", "values"],
            Some(parent),
        );
        if let Value::Object(entries) = &constructor {
            let prototype = {
                let entries = entries.borrow();
                Self::object_get_entry(&entries, "prototype")
            };
            if let Some(Value::Object(prototype)) = prototype {
                self.install_radio_node_list_prototype_accessors(&prototype);
            }
        }
        constructor
    }

    fn install_radio_node_list_prototype_accessors(
        &mut self,
        prototype: &Rc<RefCell<ObjectValue>>,
    ) {
        let has_value_accessor = {
            let prototype_ref = prototype.borrow();
            Self::has_object_accessor_property(&*prototype_ref, "value")
        };
        if has_value_accessor {
            return;
        }
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            Self::object_getter_storage_key("value"),
            Self::new_receiver_builtin_callable("radio_node_list", "value_get"),
        );
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            Self::object_setter_storage_key("value"),
            Self::new_receiver_builtin_callable("radio_node_list", "value_set"),
        );
        Self::mark_property_non_enumerable(prototype, "value");
    }

    fn cached_radio_node_list_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("RadioNodeList")
            .cloned()
        {
            self.install_radio_node_list_prototype_accessors(&prototype);
            return Value::Object(prototype);
        }
        let constructor = self.cached_radio_node_list_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.install_radio_node_list_prototype_accessors(&prototype);
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("RadioNodeList".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_html_form_controls_collection_constructor_value(&mut self) -> Value {
        let parent = self.cached_html_collection_constructor_prototype_value();
        self.cached_collection_like_constructor_value(
            "HTMLFormControlsCollection",
            "html_form_controls_collection_constructor",
            "html_collection",
            &[],
            Some(parent),
        )
    }

    fn cached_html_form_controls_collection_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("HTMLFormControlsCollection")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let constructor = self.cached_html_form_controls_collection_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("HTMLFormControlsCollection".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_html_options_collection_constructor_value(&mut self) -> Value {
        let parent = self.cached_html_collection_constructor_prototype_value();
        self.cached_collection_like_constructor_value(
            "HTMLOptionsCollection",
            "html_options_collection_constructor",
            "html_collection",
            &[],
            Some(parent),
        )
    }

    fn cached_html_options_collection_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("HTMLOptionsCollection")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let constructor = self.cached_html_options_collection_constructor_value();
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("HTMLOptionsCollection".to_string(), prototype.clone());
        Value::Object(prototype)
    }

    fn cached_symbol_static_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("Symbol.{method}"), || {
            Self::new_symbol_static_method_callable(method)
        })
    }

    fn cached_regexp_static_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("RegExp.{method}"), || {
            Self::new_regexp_static_method_callable(method)
        })
    }

    fn cached_promise_static_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("Promise.{method}"), || {
            Self::new_promise_static_method_callable(method)
        })
    }

    fn cached_array_buffer_static_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("ArrayBuffer.{method}"), || {
            Self::new_array_buffer_static_method_callable(method)
        })
    }

    fn cached_typed_array_static_method_value(
        &mut self,
        kind: TypedArrayConstructorKind,
        method: &str,
    ) -> Value {
        let constructor_name = Self::typed_array_constructor_cache_key(&kind);
        self.cached_constructor_static_method_value(&format!("{constructor_name}.{method}"), || {
            Self::new_typed_array_static_method_callable(kind.clone(), method)
        })
    }

    fn cached_builtin_constructor_prototype_value(
        &mut self,
        cache_key: &str,
        make_value: impl FnOnce(&mut Self) -> Value,
    ) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get(cache_key)
            .cloned()
        {
            return Value::Object(prototype);
        }
        let value = make_value(self);
        let Value::Object(prototype) = &value else {
            return value;
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert(cache_key.to_string(), prototype.clone());
        value
    }

    fn cached_string_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self.script_runtime.string_constructor_prototype.clone() {
            return Value::Object(prototype);
        }
        let Value::Object(prototype) = self.new_receiver_builtin_prototype_with_iterator_value(
            Value::StringConstructor,
            "string",
            &[
                "at",
                "charAt",
                "charCodeAt",
                "concat",
                "codePointAt",
                "endsWith",
                "includes",
                "indexOf",
                "lastIndexOf",
                "isWellFormed",
                "localeCompare",
                "match",
                "matchAll",
                "normalize",
                "padEnd",
                "padStart",
                "replace",
                "replaceAll",
                "repeat",
                "search",
                "slice",
                "split",
                "startsWith",
                "substring",
                "toLocaleLowerCase",
                "toLocaleUpperCase",
                "toLowerCase",
                "toString",
                "toUpperCase",
                "toWellFormed",
                "trim",
                "trimEnd",
                "trimStart",
                "valueOf",
            ],
            Some("iterator"),
        ) else {
            unreachable!("string constructor prototype must be an object");
        };
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime.string_constructor_prototype = Some(prototype.clone());
        Value::Object(prototype)
    }

    fn cached_symbol_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self.script_runtime.symbol_constructor_prototype.clone() {
            return Value::Object(prototype);
        }
        let Value::Object(prototype) = Self::new_receiver_builtin_prototype_value(
            Value::SymbolConstructor,
            "symbol",
            &["toString", "valueOf"],
        ) else {
            unreachable!("symbol constructor prototype must be an object");
        };
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime.symbol_constructor_prototype = Some(prototype.clone());
        Value::Object(prototype)
    }

    fn cached_typed_array_constructor_prototype_value(
        &mut self,
        kind: TypedArrayConstructorKind,
    ) -> Value {
        let cache_key = Self::typed_array_constructor_cache_key(&kind);
        if let Some(prototype) = self
            .script_runtime
            .typed_array_constructor_prototypes
            .get(&cache_key)
            .cloned()
        {
            return Value::Object(prototype);
        }
        let Value::Object(prototype) = self.new_receiver_builtin_prototype_with_iterator_value(
            Value::TypedArrayConstructor(kind.clone()),
            "typed_array",
            &[
                "at",
                "copyWithin",
                "entries",
                "join",
                "keys",
                "slice",
                "subarray",
                "values",
                "with",
            ],
            Some("values"),
        ) else {
            unreachable!("typed array constructor prototype must be an object");
        };
        let parent_prototype = match kind {
            TypedArrayConstructorKind::Concrete(_) => self
                .cached_typed_array_constructor_prototype_value(
                    TypedArrayConstructorKind::Abstract,
                ),
            TypedArrayConstructorKind::Abstract => self.object_constructor_prototype_value(),
        };
        Self::set_internal_prototype(&prototype, parent_prototype);
        self.script_runtime
            .typed_array_constructor_prototypes
            .insert(cache_key, prototype.clone());
        Value::Object(prototype)
    }

    fn cached_blob_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("Blob", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::BlobConstructor,
                "blob",
                &["arrayBuffer", "bytes", "slice", "stream", "text"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    fn cached_array_buffer_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("ArrayBuffer", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::ArrayBufferConstructor,
                "array_buffer",
                &["resize", "slice", "transfer", "transferToFixedLength"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    fn cached_promise_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("Promise", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::PromiseConstructor,
                "promise",
                &["then", "catch", "finally"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    fn cached_date_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("Date", |this| {
            let prototype = Self::new_object_value(
                [
                    "getTime",
                    "setTime",
                    "toISOString",
                    "toLocaleDateString",
                    "toString",
                    "valueOf",
                    "getUTCFullYear",
                    "getUTCMonth",
                    "getUTCDate",
                    "getUTCDay",
                    "getUTCHours",
                    "getUTCMinutes",
                    "getUTCSeconds",
                    "getUTCMilliseconds",
                    "getFullYear",
                    "getMonth",
                    "getDate",
                    "getHours",
                    "getMinutes",
                    "getSeconds",
                ]
                .into_iter()
                .map(|method| {
                    (
                        method.to_string(),
                        Self::new_receiver_builtin_callable("date", method),
                    )
                })
                .collect::<Vec<_>>(),
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            let to_string_tag_symbol =
                this.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
            let to_string_tag_key = this.property_key_to_storage_key(&to_string_tag_symbol);
            Self::object_set_entry(
                &mut entries.borrow_mut(),
                to_string_tag_key.clone(),
                Value::String("Date".to_string()),
            );
            Self::mark_existing_public_properties_non_enumerable(entries);
            Self::mark_property_non_enumerable(entries, &to_string_tag_key);
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    fn cached_regexp_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("RegExp", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::RegExpConstructor,
                "regexp",
                &["exec", "test", "toString"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            for (property, member) in [
                (SymbolStaticProperty::Match, "match"),
                (SymbolStaticProperty::MatchAll, "matchAll"),
                (SymbolStaticProperty::Replace, "replace"),
                (SymbolStaticProperty::Search, "search"),
                (SymbolStaticProperty::Split, "split"),
            ] {
                let symbol = this.eval_symbol_static_property(property);
                let key = this.property_key_to_storage_key(&symbol);
                Self::object_set_entry(
                    &mut entries.borrow_mut(),
                    key.clone(),
                    Self::new_receiver_builtin_callable("regexp", member),
                );
                Self::mark_property_non_enumerable(entries, &key);
            }
            for key in [
                "source",
                "flags",
                "global",
                "ignoreCase",
                "multiline",
                "dotAll",
                "sticky",
                "hasIndices",
                "unicode",
                "unicodeSets",
            ] {
                Self::object_set_entry(
                    &mut entries.borrow_mut(),
                    key.to_string(),
                    Value::Undefined,
                );
                Self::object_set_entry(
                    &mut entries.borrow_mut(),
                    Self::object_getter_storage_key(key),
                    Self::new_receiver_builtin_callable("regexp", key),
                );
                Self::mark_property_non_enumerable(entries, key);
            }
            let to_string_tag_symbol =
                this.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
            let to_string_tag_key = this.property_key_to_storage_key(&to_string_tag_symbol);
            Self::object_set_entry(
                &mut entries.borrow_mut(),
                to_string_tag_key.clone(),
                Value::String("RegExp".to_string()),
            );
            Self::object_set_entry(
                &mut entries.borrow_mut(),
                INTERNAL_REGEXP_PROTOTYPE_OBJECT_KEY.to_string(),
                Value::Bool(true),
            );
            Self::mark_property_non_enumerable(entries, &to_string_tag_key);
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    fn cached_map_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("Map", |this| {
            let prototype = this.new_receiver_builtin_prototype_with_iterator_value(
                Value::MapConstructor,
                "map",
                &[
                    "set",
                    "get",
                    "has",
                    "delete",
                    "clear",
                    "forEach",
                    "entries",
                    "keys",
                    "values",
                    "getOrInsert",
                    "getOrInsertComputed",
                ],
                Some("entries"),
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    fn cached_weak_map_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("WeakMap", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::WeakMapConstructor,
                "weak_map",
                &[
                    "set",
                    "get",
                    "has",
                    "delete",
                    "getOrInsert",
                    "getOrInsertComputed",
                ],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    fn cached_set_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("Set", |this| {
            let prototype = this.new_receiver_builtin_prototype_with_iterator_value(
                Value::SetConstructor,
                "set",
                &[
                    "add",
                    "has",
                    "delete",
                    "clear",
                    "forEach",
                    "entries",
                    "keys",
                    "values",
                    "union",
                    "intersection",
                    "difference",
                    "symmetricDifference",
                    "isDisjointFrom",
                    "isSubsetOf",
                    "isSupersetOf",
                ],
                Some("values"),
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    fn cached_weak_set_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("WeakSet", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::WeakSetConstructor,
                "weak_set",
                &["add", "has", "delete"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    fn cached_url_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("URL", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::UrlConstructor,
                "url",
                &["toString", "toJSON"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    fn cached_url_search_params_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("URLSearchParams", |this| {
            let prototype = this.new_receiver_builtin_prototype_with_iterator_value(
                Value::UrlSearchParamsConstructor,
                "url_search_params",
                &[
                    "append", "delete", "get", "getAll", "has", "set", "sort", "forEach",
                    "entries", "keys", "values", "toString",
                ],
                Some("entries"),
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    fn function_length(function: &Rc<FunctionValue>) -> i64 {
        let mut length = 0_i64;
        for param in &function.handler.params {
            if param.is_rest || param.default.is_some() {
                break;
            }
            length += 1;
        }
        length
    }

    fn function_display_name(&self, function: &Rc<FunctionValue>) -> String {
        self.script_runtime
            .function_public_properties
            .get(&function.function_id)
            .and_then(|entries| Self::object_get_entry(entries, "name"))
            .map(|value| value.as_string())
            .unwrap_or_else(|| function.expression_name.clone().unwrap_or_default())
    }

    pub(crate) fn set_function_public_name(&mut self, function: &Rc<FunctionValue>, name: &str) {
        let entries = self
            .script_runtime
            .function_public_properties
            .entry(function.function_id)
            .or_default();
        Self::object_set_entry(entries, "name".to_string(), Value::String(name.to_string()));
        Self::object_set_entry(
            entries,
            Self::object_non_enumerable_storage_key("name"),
            Value::Bool(true),
        );
        Self::object_set_entry(
            entries,
            Self::object_non_writable_storage_key("name"),
            Value::Bool(true),
        );
    }

    fn object_backed_callable_name_and_length(kind: &str) -> Option<(&'static str, i64)> {
        match kind {
            "generator_function_constructor" => Some(("GeneratorFunction", 1)),
            "async_generator_function_constructor" => Some(("AsyncGeneratorFunction", 1)),
            "boolean_constructor" => Some(("Boolean", 1)),
            "number_constructor" => Some(("Number", 1)),
            "bigint_constructor" => Some(("BigInt", 1)),
            "object_constructor" => Some(("Object", 1)),
            "function_constructor" => Some(("Function", 1)),
            "node_list_constructor" => Some(("NodeList", 0)),
            "image_bitmap_constructor" => Some(("ImageBitmap", 0)),
            "text_track_constructor" => Some(("TextTrack", 0)),
            "text_track_list_constructor" => Some(("TextTrackList", 0)),
            "time_ranges_constructor" => Some(("TimeRanges", 0)),
            "storage_constructor" => Some(("Storage", 0)),
            "cookie_store_constructor" => Some(("CookieStore", 0)),
            "cache_storage_constructor" => Some(("CacheStorage", 0)),
            "cache_constructor" => Some(("Cache", 0)),
            "radio_node_list_constructor" => Some(("RadioNodeList", 0)),
            "html_collection_constructor" => Some(("HTMLCollection", 0)),
            "html_form_controls_collection_constructor" => Some(("HTMLFormControlsCollection", 0)),
            "html_options_collection_constructor" => Some(("HTMLOptionsCollection", 0)),
            "function_call" => Some(("call", 1)),
            "function_apply" => Some(("apply", 2)),
            "function_bind" => Some(("bind", 1)),
            "function_to_string" => Some(("toString", 0)),
            "event_target_constructor" => Some(("EventTarget", 0)),
            "event_constructor" => Some(("Event", 1)),
            "custom_event_constructor" => Some(("CustomEvent", 1)),
            "mouse_event_constructor" => Some(("MouseEvent", 1)),
            "keyboard_event_constructor" => Some(("KeyboardEvent", 1)),
            "wheel_event_constructor" => Some(("WheelEvent", 1)),
            "navigate_event_constructor" => Some(("NavigateEvent", 1)),
            "pointer_event_constructor" => Some(("PointerEvent", 1)),
            "error_event_constructor" => Some(("ErrorEvent", 1)),
            "hash_change_event_constructor" => Some(("HashChangeEvent", 1)),
            "before_unload_event_constructor" => Some(("BeforeUnloadEvent", 1)),
            "image_data_constructor" => Some(("ImageData", 2)),
            "dom_parser_constructor" => Some(("DOMParser", 0)),
            "xml_serializer_constructor" => Some(("XMLSerializer", 0)),
            "document_constructor" => Some(("Document", 0)),
            "document_parse_html" => Some(("parseHTML", 1)),
            "document_parse_html_unsafe" => Some(("parseHTMLUnsafe", 1)),
            "fetch_function" => Some(("fetch", 1)),
            "match_media_function" => Some(("matchMedia", 1)),
            "window_close_function" => Some(("close", 0)),
            "window_open_function" => Some(("open", 0)),
            "window_stop_function" => Some(("stop", 0)),
            "window_focus_function" => Some(("focus", 0)),
            "window_scroll_function" => Some(("scroll", 0)),
            "window_scroll_by_function" => Some(("scrollBy", 0)),
            "window_scroll_to_function" => Some(("scrollTo", 0)),
            "window_move_by_function" => Some(("moveBy", 2)),
            "window_move_to_function" => Some(("moveTo", 2)),
            "window_resize_by_function" => Some(("resizeBy", 2)),
            "window_resize_to_function" => Some(("resizeTo", 2)),
            "window_post_message_function" => Some(("postMessage", 1)),
            "window_get_computed_style_function" => Some(("getComputedStyle", 1)),
            "computed_style_item" => Some(("item", 1)),
            "dom_rect_list_item" => Some(("item", 1)),
            "window_alert_function" => Some(("alert", 0)),
            "window_confirm_function" => Some(("confirm", 0)),
            "window_print_function" => Some(("print", 0)),
            "window_report_error_function" => Some(("reportError", 1)),
            "window_prompt_function" => Some(("prompt", 0)),
            "popup_window_close_function" => Some(("close", 0)),
            "popup_window_focus_function" => Some(("focus", 0)),
            "popup_window_print_function" => Some(("print", 0)),
            "popup_document_open_function" => Some(("open", 0)),
            "popup_document_write_function" => Some(("write", 0)),
            "popup_document_close_function" => Some(("close", 0)),
            "global_css_escape" => Some(("escape", 1)),
            "intl_collator_compare" => Some(("compare", 2)),
            "intl_date_time_format" => Some(("format", 1)),
            "intl_duration_format" => Some(("format", 1)),
            "intl_list_format" => Some(("format", 1)),
            "intl_number_format" => Some(("format", 1)),
            "clipboard_item_constructor" => Some(("ClipboardItem", 1)),
            "clipboard_write" => Some(("write", 1)),
            "request_constructor" => Some(("Request", 1)),
            "file_constructor" => Some(("File", 2)),
            "headers_constructor" => Some(("Headers", 0)),
            "worker_constructor" => Some(("Worker", 1)),
            "data_transfer_constructor" => Some(("DataTransfer", 0)),
            "option_constructor" => Some(("Option", 0)),
            "audio_constructor" => Some(("Audio", 0)),
            "text_encoder_constructor" => Some(("TextEncoder", 0)),
            "text_decoder_constructor" => Some(("TextDecoder", 0)),
            "text_encoder_stream_constructor" => Some(("TextEncoderStream", 0)),
            "text_decoder_stream_constructor" => Some(("TextDecoderStream", 0)),
            "css_style_sheet_constructor" => Some(("CSSStyleSheet", 0)),
            "text_encoder_get_encoding" => Some(("encoding", 0)),
            "text_encoder_encode" => Some(("encode", 0)),
            "text_encoder_encode_into" => Some(("encodeInto", 2)),
            "text_decoder_get_encoding" => Some(("encoding", 0)),
            "text_decoder_get_fatal" => Some(("fatal", 0)),
            "text_decoder_get_ignore_bom" => Some(("ignoreBOM", 0)),
            "text_decoder_decode" => Some(("decode", 0)),
            "text_encoder_stream_get_encoding" => Some(("encoding", 0)),
            "text_encoder_stream_get_readable" => Some(("readable", 0)),
            "text_encoder_stream_get_writable" => Some(("writable", 0)),
            "text_decoder_stream_get_encoding" => Some(("encoding", 0)),
            "text_decoder_stream_get_fatal" => Some(("fatal", 0)),
            "text_decoder_stream_get_ignore_bom" => Some(("ignoreBOM", 0)),
            "text_decoder_stream_get_readable" => Some(("readable", 0)),
            "text_decoder_stream_get_writable" => Some(("writable", 0)),
            "class_list_add" => Some(("add", 1)),
            "class_list_remove" => Some(("remove", 1)),
            "class_list_toggle" => Some(("toggle", 1)),
            "class_list_contains" => Some(("contains", 1)),
            "class_list_replace" => Some(("replace", 2)),
            "class_list_item" => Some(("item", 1)),
            "class_list_for_each" => Some(("forEach", 1)),
            "class_list_keys" => Some(("keys", 0)),
            "class_list_values" => Some(("values", 0)),
            "class_list_entries" => Some(("entries", 0)),
            "class_list_to_string" => Some(("toString", 0)),
            "named_node_map_item" => Some(("item", 1)),
            "named_node_map_get_named_item" => Some(("getNamedItem", 1)),
            "named_node_map_set_named_item" => Some(("setNamedItem", 1)),
            "named_node_map_remove_named_item" => Some(("removeNamedItem", 1)),
            "named_node_map_get_named_item_ns" => Some(("getNamedItemNS", 2)),
            "named_node_map_set_named_item_ns" => Some(("setNamedItemNS", 1)),
            "named_node_map_remove_named_item_ns" => Some(("removeNamedItemNS", 2)),
            "named_node_map_for_each" => Some(("forEach", 1)),
            "named_node_map_keys" => Some(("keys", 0)),
            "named_node_map_values" => Some(("values", 0)),
            "named_node_map_entries" => Some(("entries", 0)),
            _ => None,
        }
    }

    fn static_object_method_name_and_length(value: &Value) -> Option<(String, i64)> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        let method = match Self::object_get_entry(&entries, INTERNAL_STATIC_METHOD_NAME_KEY) {
            Some(Value::String(method)) => method,
            _ => return None,
        };
        let length = match method.as_str() {
            "create" => 2,
            "assign" => 2,
            "getOwnPropertyDescriptor" => 2,
            "defineProperty" => 3,
            "getOwnPropertyNames" => 1,
            "getOwnPropertySymbols" => 1,
            "keys" => 1,
            "values" => 1,
            "entries" => 1,
            "hasOwn" => 2,
            "getPrototypeOf" => 1,
            "setPrototypeOf" => 2,
            "freeze" => 1,
            _ => return None,
        };
        Some((method, length))
    }

    fn static_reflect_method_name_and_length(value: &Value) -> Option<(String, i64)> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        let method = match Self::object_get_entry(&entries, INTERNAL_STATIC_METHOD_NAME_KEY) {
            Some(Value::String(method)) => method,
            _ => return None,
        };
        let length = match method.as_str() {
            "set" => 3,
            "ownKeys" => 1,
            _ => return None,
        };
        Some((method, length))
    }

    fn receiver_builtin_callable_name_and_length(value: &Value) -> Option<(String, i64)> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        let family = match Self::object_get_entry(&entries, "__bt_receiver_builtin_family") {
            Some(Value::String(family)) => family,
            _ => return None,
        };
        let member = match Self::object_get_entry(&entries, "__bt_receiver_builtin_member") {
            Some(Value::String(member)) => member,
            _ => return None,
        };
        let (name, length) = match (family.as_str(), member.as_str()) {
            ("worker", "postMessage") => ("postMessage", 1),
            ("worker", "terminate") => ("terminate", 0),
            ("boolean", "toString") => ("toString", 0),
            ("boolean", "valueOf") => ("valueOf", 0),
            ("number", "toExponential") => ("toExponential", 1),
            ("number", "toFixed") => ("toFixed", 1),
            ("number", "toLocaleString") => ("toLocaleString", 0),
            ("number", "toPrecision") => ("toPrecision", 1),
            ("number", "toString") => ("toString", 1),
            ("number", "valueOf") => ("valueOf", 0),
            ("bigint", "toLocaleString") => ("toLocaleString", 0),
            ("bigint", "toString") => ("toString", 1),
            ("bigint", "valueOf") => ("valueOf", 0),
            ("symbol", "toString") => ("toString", 0),
            ("symbol", "valueOf") => ("valueOf", 0),
            ("string", "at") => ("at", 1),
            ("string", "charAt") => ("charAt", 1),
            ("string", "charCodeAt") => ("charCodeAt", 1),
            ("string", "concat") => ("concat", 1),
            ("string", "codePointAt") => ("codePointAt", 1),
            ("string", "endsWith") => ("endsWith", 1),
            ("string", "includes") => ("includes", 1),
            ("string", "indexOf") => ("indexOf", 1),
            ("string", "isWellFormed") => ("isWellFormed", 0),
            ("string", "lastIndexOf") => ("lastIndexOf", 1),
            ("string", "localeCompare") => ("localeCompare", 1),
            ("string", "match") => ("match", 1),
            ("string", "matchAll") => ("matchAll", 1),
            ("string", "normalize") => ("normalize", 0),
            ("string", "padEnd") => ("padEnd", 1),
            ("string", "padStart") => ("padStart", 1),
            ("string", "replace") => ("replace", 2),
            ("string", "replaceAll") => ("replaceAll", 2),
            ("string", "repeat") => ("repeat", 1),
            ("string", "search") => ("search", 1),
            ("string", "slice") => ("slice", 2),
            ("string", "split") => ("split", 2),
            ("string", "startsWith") => ("startsWith", 1),
            ("string", "substring") => ("substring", 2),
            ("string", "toLocaleLowerCase") => ("toLocaleLowerCase", 0),
            ("string", "toLocaleUpperCase") => ("toLocaleUpperCase", 0),
            ("string", "toLowerCase") => ("toLowerCase", 0),
            ("string", "toString") => ("toString", 0),
            ("string", "toUpperCase") => ("toUpperCase", 0),
            ("string", "toWellFormed") => ("toWellFormed", 0),
            ("string", "trim") => ("trim", 0),
            ("string", "trimEnd") => ("trimEnd", 0),
            ("string", "trimStart") => ("trimStart", 0),
            ("string", "valueOf") => ("valueOf", 0),
            ("node", "append") => ("append", 0),
            ("node", "prepend") => ("prepend", 0),
            ("node", "replaceChildren") => ("replaceChildren", 0),
            ("node", "before") => ("before", 0),
            ("node", "after") => ("after", 0),
            ("node", "replaceWith") => ("replaceWith", 0),
            ("node", "remove") => ("remove", 0),
            ("node", "appendChild") => ("appendChild", 1),
            ("node", "insertBefore") => ("insertBefore", 2),
            ("node", "removeChild") => ("removeChild", 1),
            ("node", "replaceChild") => ("replaceChild", 2),
            ("node", "hasChildNodes") => ("hasChildNodes", 0),
            ("node", "contains") => ("contains", 1),
            ("node", "getRootNode") => ("getRootNode", 0),
            ("node", "compareDocumentPosition") => ("compareDocumentPosition", 1),
            ("node", "isEqualNode") => ("isEqualNode", 1),
            ("node", "isSameNode") => ("isSameNode", 1),
            ("node", "normalize") => ("normalize", 0),
            ("node", "isDefaultNamespace") => ("isDefaultNamespace", 1),
            ("node", "lookupPrefix") => ("lookupPrefix", 1),
            ("node", "lookupNamespaceURI") => ("lookupNamespaceURI", 1),
            ("node", "cloneNode") => ("cloneNode", 0),
            ("node", "querySelector") => ("querySelector", 1),
            ("node", "querySelectorAll") => ("querySelectorAll", 1),
            ("node", "getAttributeNames") => ("getAttributeNames", 0),
            ("node", "toggleAttribute") => ("toggleAttribute", 1),
            ("node", "matches") => ("matches", 1),
            ("node", "closest") => ("closest", 1),
            ("node", "insertAdjacentElement") => ("insertAdjacentElement", 2),
            ("node", "insertAdjacentHTML") => ("insertAdjacentHTML", 2),
            ("node", "insertAdjacentText") => ("insertAdjacentText", 2),
            ("node", "setHTMLUnsafe") => ("setHTMLUnsafe", 1),
            ("node_list", "item") => ("item", 1),
            ("node_list", "namedItem") => ("namedItem", 1),
            ("node_list", "forEach") => ("forEach", 1),
            ("node_list", "entries") => ("entries", 0),
            ("node_list", "keys") => ("keys", 0),
            ("node_list", "values") => ("values", 0),
            ("image_bitmap", "width_get") => ("get width", 0),
            ("image_bitmap", "height_get") => ("get height", 0),
            ("image_bitmap", "close") => ("close", 0),
            ("text_track", "id_get") => ("get id", 0),
            ("text_track", "kind_get") => ("get kind", 0),
            ("text_track", "label_get") => ("get label", 0),
            ("text_track", "language_get") => ("get language", 0),
            ("text_track", "mode_get") => ("get mode", 0),
            ("text_track", "mode_set") => ("set mode", 1),
            ("text_track", "cues_get") => ("get cues", 0),
            ("text_track", "active_cues_get") => ("get activeCues", 0),
            ("text_track", "in_band_metadata_track_dispatch_type_get") => {
                ("get inBandMetadataTrackDispatchType", 0)
            }
            ("time_ranges", "length_get") => ("get length", 0),
            ("time_ranges", "start") => ("start", 1),
            ("time_ranges", "end") => ("end", 1),
            ("animation", "cancel") => ("cancel", 0),
            ("animation", "finish") => ("finish", 0),
            ("animation", "pause") => ("pause", 0),
            ("animation", "play") => ("play", 0),
            ("animation", "reverse") => ("reverse", 0),
            ("animation", "updatePlaybackRate") => ("updatePlaybackRate", 1),
            ("animation", "commitStyles") => ("commitStyles", 0),
            ("animation", "persist") => ("persist", 0),
            ("radio_node_list", "value_get") => ("get value", 0),
            ("radio_node_list", "value_set") => ("set value", 1),
            ("html_form", "submit") => ("submit", 0),
            ("html_form", "requestSubmit") => ("requestSubmit", 1),
            ("html_form", "reset") => ("reset", 0),
            ("html_form", "checkValidity") => ("checkValidity", 0),
            ("html_form", "reportValidity") => ("reportValidity", 0),
            ("html_media", "play") => ("play", 0),
            ("html_media", "pause") => ("pause", 0),
            ("html_media", "load") => ("load", 0),
            ("html_media", "canPlayType") => ("canPlayType", 1),
            ("html_media", "fastSeek") => ("fastSeek", 1),
            ("html_collection", "item") => ("item", 1),
            ("html_collection", "namedItem") => ("namedItem", 1),
            ("html_collection", "forEach") => ("forEach", 1),
            ("html_collection", "entries") => ("entries", 0),
            ("html_collection", "keys") => ("keys", 0),
            ("html_collection", "values") => ("values", 0),
            ("date", "getTime") => ("getTime", 0),
            ("date", "setTime") => ("setTime", 1),
            ("date", "toISOString") => ("toISOString", 0),
            ("date", "toLocaleDateString") => ("toLocaleDateString", 0),
            ("date", "toString") => ("toString", 0),
            ("date", "valueOf") => ("valueOf", 0),
            ("date", "getUTCFullYear") => ("getUTCFullYear", 0),
            ("date", "getUTCMonth") => ("getUTCMonth", 0),
            ("date", "getUTCDate") => ("getUTCDate", 0),
            ("date", "getUTCDay") => ("getUTCDay", 0),
            ("date", "getUTCHours") => ("getUTCHours", 0),
            ("date", "getUTCMinutes") => ("getUTCMinutes", 0),
            ("date", "getUTCSeconds") => ("getUTCSeconds", 0),
            ("date", "getUTCMilliseconds") => ("getUTCMilliseconds", 0),
            ("date", "getFullYear") => ("getFullYear", 0),
            ("date", "getMonth") => ("getMonth", 0),
            ("date", "getDate") => ("getDate", 0),
            ("date", "getHours") => ("getHours", 0),
            ("date", "getMinutes") => ("getMinutes", 0),
            ("date", "getSeconds") => ("getSeconds", 0),
            ("regexp", "source") => ("get source", 0),
            ("regexp", "flags") => ("get flags", 0),
            ("regexp", "global") => ("get global", 0),
            ("regexp", "ignoreCase") => ("get ignoreCase", 0),
            ("regexp", "multiline") => ("get multiline", 0),
            ("regexp", "dotAll") => ("get dotAll", 0),
            ("regexp", "sticky") => ("get sticky", 0),
            ("regexp", "hasIndices") => ("get hasIndices", 0),
            ("regexp", "unicode") => ("get unicode", 0),
            ("regexp", "unicodeSets") => ("get unicodeSets", 0),
            ("regexp", "exec") => ("exec", 1),
            ("regexp", "test") => ("test", 1),
            ("regexp", "toString") => ("toString", 0),
            ("regexp", "match") => ("[Symbol.match]", 1),
            ("regexp", "matchAll") => ("[Symbol.matchAll]", 1),
            ("regexp", "replace") => ("[Symbol.replace]", 2),
            ("regexp", "search") => ("[Symbol.search]", 1),
            ("regexp", "split") => ("[Symbol.split]", 2),
            ("intl_collator", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_date_time_format", "formatToParts") => ("formatToParts", 0),
            ("intl_date_time_format", "formatRange") => ("formatRange", 2),
            ("intl_date_time_format", "formatRangeToParts") => ("formatRangeToParts", 2),
            ("intl_date_time_format", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_display_names", "of") => ("of", 1),
            ("intl_display_names", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_duration_format", "formatToParts") => ("formatToParts", 1),
            ("intl_duration_format", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_list_format", "formatToParts") => ("formatToParts", 1),
            ("intl_list_format", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_locale", "getCalendars") => ("getCalendars", 0),
            ("intl_locale", "getCollations") => ("getCollations", 0),
            ("intl_locale", "getHourCycles") => ("getHourCycles", 0),
            ("intl_locale", "getNumberingSystems") => ("getNumberingSystems", 0),
            ("intl_locale", "getTextInfo") => ("getTextInfo", 0),
            ("intl_locale", "getTimeZones") => ("getTimeZones", 0),
            ("intl_locale", "getWeekInfo") => ("getWeekInfo", 0),
            ("intl_locale", "maximize") => ("maximize", 0),
            ("intl_locale", "minimize") => ("minimize", 0),
            ("intl_locale", "toString") => ("toString", 0),
            ("intl_number_format", "formatToParts") => ("formatToParts", 1),
            ("intl_number_format", "formatRange") => ("formatRange", 2),
            ("intl_number_format", "formatRangeToParts") => ("formatRangeToParts", 2),
            ("intl_number_format", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_plural_rules", "select") => ("select", 1),
            ("intl_plural_rules", "selectRange") => ("selectRange", 2),
            ("intl_plural_rules", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_relative_time_format", "format") => ("format", 2),
            ("intl_relative_time_format", "formatToParts") => ("formatToParts", 2),
            ("intl_relative_time_format", "resolvedOptions") => ("resolvedOptions", 0),
            ("intl_segmenter", "segment") => ("segment", 1),
            ("intl_segmenter", "resolvedOptions") => ("resolvedOptions", 0),
            ("object", "hasOwnProperty") => ("hasOwnProperty", 1),
            ("object", "isPrototypeOf") => ("isPrototypeOf", 1),
            ("object", "propertyIsEnumerable") => ("propertyIsEnumerable", 1),
            ("object", "toString") => ("toString", 0),
            ("object", "valueOf") => ("valueOf", 0),
            ("document", "createElement") => ("createElement", 1),
            ("document", "createElementNS") => ("createElementNS", 2),
            ("document", "createTextNode") => ("createTextNode", 1),
            ("document", "createAttribute") => ("createAttribute", 1),
            ("document", "createDocumentFragment") => ("createDocumentFragment", 0),
            ("document", "createRange") => ("createRange", 0),
            ("document", "getSelection") => ("getSelection", 0),
            ("document", "append") => ("append", 0),
            ("document", "getElementById") => ("getElementById", 1),
            ("document", "getElementsByClassName") => ("getElementsByClassName", 1),
            ("document", "getElementsByName") => ("getElementsByName", 1),
            ("document", "getElementsByTagName") => ("getElementsByTagName", 1),
            ("document", "getElementsByTagNameNS") => ("getElementsByTagNameNS", 2),
            ("document", "querySelector") => ("querySelector", 1),
            ("document", "querySelectorAll") => ("querySelectorAll", 1),
            ("document", "createTreeWalker") => ("createTreeWalker", 1),
            ("document", "addEventListener") => ("addEventListener", 2),
            ("document", "removeEventListener") => ("removeEventListener", 2),
            ("parsed_document", "createTreeWalker") => ("createTreeWalker", 1),
            ("parsed_document", "querySelector") => ("querySelector", 1),
            ("parsed_document", "querySelectorAll") => ("querySelectorAll", 1),
            ("parsed_document", "getElementById") => ("getElementById", 1),
            ("parsed_document", "getElementsByClassName") => ("getElementsByClassName", 1),
            ("parsed_document", "getElementsByName") => ("getElementsByName", 1),
            ("parsed_document", "getElementsByTagName") => ("getElementsByTagName", 1),
            ("parsed_document", "createElement") => ("createElement", 1),
            ("parsed_document", "createElementNS") => ("createElementNS", 2),
            ("parsed_document", "createTextNode") => ("createTextNode", 1),
            ("parsed_document", "createAttribute") => ("createAttribute", 1),
            ("parsed_document", "createDocumentFragment") => ("createDocumentFragment", 0),
            ("parsed_document", "createRange") => ("createRange", 0),
            ("parsed_document", "append") => ("append", 0),
            ("dom_parser", "parseFromString") => ("parseFromString", 2),
            ("xml_serializer", "serializeToString") => ("serializeToString", 1),
            ("tree_walker", "nextNode") => ("nextNode", 0),
            ("range", "setStart") => ("setStart", 2),
            ("range", "setEnd") => ("setEnd", 2),
            ("selection", "addRange") => ("addRange", 1),
            ("selection", "collapse") => ("collapse", 1),
            ("selection", "collapseToEnd") => ("collapseToEnd", 0),
            ("selection", "collapseToStart") => ("collapseToStart", 0),
            ("selection", "containsNode") => ("containsNode", 1),
            ("selection", "deleteFromDocument") => ("deleteFromDocument", 0),
            ("selection", "empty") => ("empty", 0),
            ("selection", "extend") => ("extend", 2),
            ("selection", "getComposedRanges") => ("getComposedRanges", 0),
            ("selection", "getRangeAt") => ("getRangeAt", 1),
            ("selection", "modify") => ("modify", 3),
            ("selection", "removeAllRanges") => ("removeAllRanges", 0),
            ("selection", "removeRange") => ("removeRange", 1),
            ("selection", "selectAllChildren") => ("selectAllChildren", 1),
            ("selection", "setBaseAndExtent") => ("setBaseAndExtent", 4),
            ("selection", "setPosition") => ("setPosition", 1),
            ("selection", "toString") => ("toString", 0),
            ("event_target", "addEventListener") => ("addEventListener", 2),
            ("event_target", "removeEventListener") => ("removeEventListener", 2),
            ("event_target", "dispatchEvent") => ("dispatchEvent", 1),
            ("event", "preventDefault") => ("preventDefault", 0),
            ("event", "stopPropagation") => ("stopPropagation", 0),
            ("event", "stopImmediatePropagation") => ("stopImmediatePropagation", 0),
            ("keyboard_event", "getModifierState") => ("getModifierState", 1),
            ("pointer_event", "getCoalescedEvents") => ("getCoalescedEvents", 0),
            ("pointer_event", "getPredictedEvents") => ("getPredictedEvents", 0),
            ("navigate_event", "intercept") => ("intercept", 1),
            ("navigate_event", "scroll") => ("scroll", 0),
            ("data_transfer", "getData") => ("getData", 1),
            ("data_transfer", "setData") => ("setData", 2),
            ("data_transfer", "clearData") => ("clearData", 0),
            ("data_transfer", "setDragImage") => ("setDragImage", 3),
            ("data_transfer", "addElement") => ("addElement", 1),
            ("data_transfer_item", "getAsFile") => ("getAsFile", 0),
            ("data_transfer_item", "getAsFileSystemHandle") => ("getAsFileSystemHandle", 0),
            ("data_transfer_item", "getAsString") => ("getAsString", 1),
            ("data_transfer_item", "webkitGetAsEntry") => ("webkitGetAsEntry", 0),
            ("data_transfer_item_list", "add") => ("add", 1),
            ("data_transfer_item_list", "remove") => ("remove", 1),
            ("data_transfer_item_list", "clear") => ("clear", 0),
            ("match_media", "addEventListener") => ("addEventListener", 2),
            ("match_media", "removeEventListener") => ("removeEventListener", 2),
            ("match_media", "dispatchEvent") => ("dispatchEvent", 1),
            ("match_media", "addListener") => ("addListener", 1),
            ("match_media", "removeListener") => ("removeListener", 1),
            ("cookie_store", "set") => ("set", 1),
            ("cookie_store", "get") => ("get", 1),
            ("cookie_store", "getAll") => ("getAll", 1),
            ("cookie_store", "delete") => ("delete", 1),
            ("cookie_store", "addEventListener") => ("addEventListener", 2),
            ("cookie_store", "removeEventListener") => ("removeEventListener", 2),
            ("cache_storage", "open") => ("open", 1),
            ("cache_storage", "match") => ("match", 1),
            ("cache_storage", "has") => ("has", 1),
            ("cache_storage", "delete") => ("delete", 1),
            ("cache_storage", "keys") => ("keys", 0),
            ("cache", "match") => ("match", 1),
            ("cache", "put") => ("put", 2),
            ("cache", "delete") => ("delete", 1),
            ("cache", "keys") => ("keys", 0),
            ("cache", "add") => ("add", 1),
            ("cache", "addAll") => ("addAll", 1),
            ("canvas_2d_context", "toString") => ("toString", 0),
            _ => return None,
        };
        Some((name.to_string(), length))
    }

    fn object_to_string_tag_property(&mut self, value: &Value) -> Result<Option<String>> {
        let symbol = self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let key = self.property_key_to_storage_key(&symbol);
        match self.object_property_from_value(value, &key)? {
            Value::String(tag) if !tag.is_empty() => Ok(Some(tag)),
            _ => Ok(None),
        }
    }

    fn object_prototype_to_string_tag(&mut self, value: &Value) -> Result<String> {
        let tag = match value {
            Value::Null => "Null".to_string(),
            Value::Undefined => "Undefined".to_string(),
            Value::Bool(_) => "Boolean".to_string(),
            Value::Number(_) | Value::Float(_) => "Number".to_string(),
            Value::BigInt(_) => "BigInt".to_string(),
            Value::String(_) => "String".to_string(),
            Value::Symbol(_) => "Symbol".to_string(),
            Value::Array(values) => {
                if Self::is_dom_rect_list_value(&values.borrow()) {
                    "DOMRectList".to_string()
                } else {
                    "Array".to_string()
                }
            }
            Value::Promise(_) => "Promise".to_string(),
            Value::Map(_) => "Map".to_string(),
            Value::WeakMap(_) => "WeakMap".to_string(),
            Value::Set(_) => "Set".to_string(),
            Value::WeakSet(_) => "WeakSet".to_string(),
            Value::Blob(_) => "Blob".to_string(),
            Value::ArrayBuffer(_) => "ArrayBuffer".to_string(),
            Value::TypedArray(values) => values.borrow().kind.name().to_string(),
            Value::RegExp(_) => "RegExp".to_string(),
            Value::Date(_) => "Date".to_string(),
            Value::NodeList(nodes) => Self::node_list_display_name(nodes).to_string(),
            Value::FormData(_) => "FormData".to_string(),
            Value::Function(_) => "Function".to_string(),
            Value::StringConstructor
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::WeakMapConstructor
            | Value::SetConstructor
            | Value::WeakSetConstructor
            | Value::UrlSearchParamsConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::TypedArrayConstructor(_)
            | Value::PromiseCapability(_) => "Function".to_string(),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::string_wrapper_value_from_object(&entries).is_some() {
                    "String".to_string()
                } else if Self::boolean_wrapper_value_from_object(&entries).is_some() {
                    "Boolean".to_string()
                } else if Self::number_wrapper_value_from_object(&entries).is_some() {
                    "Number".to_string()
                } else if Self::bigint_wrapper_value_from_object(&entries).is_some() {
                    "BigInt".to_string()
                } else if Self::symbol_wrapper_id_from_object(&entries).is_some() {
                    "Symbol".to_string()
                } else if Self::callable_kind_from_value(value).is_some() {
                    "Function".to_string()
                } else if let Some(tag) = self.object_to_string_tag_property(value)? {
                    tag
                } else if Self::is_url_object(&entries) {
                    "URL".to_string()
                } else if Self::is_location_object(&entries) {
                    "Location".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "Document".to_string()
                } else if Self::is_range_object(&entries) {
                    "Range".to_string()
                } else if Self::is_selection_object(&entries) {
                    "Selection".to_string()
                } else if Self::is_match_media_object(&entries) {
                    "MediaQueryList".to_string()
                } else if Self::is_named_node_map_object(&entries) {
                    "NamedNodeMap".to_string()
                } else if Self::is_attr_object(&entries) {
                    "Attr".to_string()
                } else if Self::is_canvas_2d_context_object(&entries) {
                    "CanvasRenderingContext2D".to_string()
                } else if Self::is_class_list_object(&entries) {
                    "DOMTokenList".to_string()
                } else if Self::is_dom_rect_object(&entries) {
                    "DOMRect".to_string()
                } else if Self::is_image_bitmap_object(&entries) {
                    "ImageBitmap".to_string()
                } else if Self::is_text_track_object(&entries) {
                    "TextTrack".to_string()
                } else if Self::is_dom_string_map_object(&entries) {
                    "DOMStringMap".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOM_PARSER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "DOMParser".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_XML_SERIALIZER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "XMLSerializer".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_TREE_WALKER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "TreeWalker".to_string()
                } else if Self::is_css_style_sheet_object(&entries) {
                    "CSSStyleSheet".to_string()
                } else if Self::is_computed_style_object(&entries) {
                    "CSSStyleDeclaration".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_READABLE_STREAM_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "ReadableStream".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_WRITABLE_STREAM_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "WritableStream".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_TEXT_ENCODER_STREAM_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "TextEncoderStream".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_TEXT_DECODER_STREAM_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "TextDecoderStream".to_string()
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_ANIMATION_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    "Animation".to_string()
                } else if Self::is_event_object(&entries) {
                    "Event".to_string()
                } else if Self::is_hash_change_event_object(&entries) {
                    "HashChangeEvent".to_string()
                } else if Self::is_error_event_object(&entries) {
                    "ErrorEvent".to_string()
                } else if Self::is_before_unload_event_object(&entries) {
                    "BeforeUnloadEvent".to_string()
                } else if Self::is_keyboard_event_object(&entries) {
                    "KeyboardEvent".to_string()
                } else if Self::is_wheel_event_object(&entries) {
                    "WheelEvent".to_string()
                } else if Self::is_navigate_event_object(&entries) {
                    "NavigateEvent".to_string()
                } else if Self::is_pointer_event_object(&entries) {
                    "PointerEvent".to_string()
                } else {
                    "Object".to_string()
                }
            }
            Value::Node(_) => "Object".to_string(),
        };
        Ok(tag)
    }

    pub(crate) fn object_prototype_to_string_value(&mut self, value: &Value) -> Result<Value> {
        Ok(Value::String(format!(
            "[object {}]",
            self.object_prototype_to_string_tag(value)?
        )))
    }

    pub(crate) fn object_prototype_value_of_value(&mut self, value: &Value) -> Result<Value> {
        match value {
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(
                "Object.valueOf called on null or undefined".into(),
            )),
            _ => Ok(value.clone()),
        }
    }

    fn object_prototype_reflection_target(
        &mut self,
        value: &Value,
        method_name: &str,
    ) -> Result<Value> {
        match value {
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(format!(
                "Object.{method_name} called on null or undefined"
            ))),
            Value::String(_)
            | Value::Bool(_)
            | Value::Number(_)
            | Value::Float(_)
            | Value::BigInt(_)
            | Value::Symbol(_) => Ok(Self::box_primitive_value(value.clone())),
            _ => Ok(value.clone()),
        }
    }

    pub(crate) fn object_prototype_has_own_property_value(
        &mut self,
        value: &Value,
        key: &Value,
    ) -> Result<Value> {
        let target = self.object_prototype_reflection_target(value, "hasOwnProperty")?;
        let key = self.property_key_to_storage_key(key);
        self.object_has_own_value(&target, &key)
    }

    pub(crate) fn object_prototype_property_is_enumerable_value(
        &mut self,
        value: &Value,
        key: &Value,
    ) -> Result<Value> {
        let target = self.object_prototype_reflection_target(value, "propertyIsEnumerable")?;
        let key = self.property_key_to_storage_key(key);
        let descriptor = self.object_get_own_property_descriptor_value(&target, &key)?;
        let Value::Object(_) = descriptor else {
            return Ok(Value::Bool(false));
        };
        Ok(Value::Bool(
            self.object_property_from_value(&descriptor, "enumerable")?
                .truthy(),
        ))
    }

    pub(crate) fn object_prototype_is_prototype_of_value(
        &mut self,
        prototype: &Value,
        value: &Value,
    ) -> Result<Value> {
        if matches!(prototype, Value::Null | Value::Undefined) {
            return Err(Error::ScriptRuntime(
                "Object.isPrototypeOf called on null or undefined".into(),
            ));
        }
        if Self::is_primitive_value(value) {
            return Ok(Value::Bool(false));
        }
        let mut current = self.value_internal_prototype_value(value);
        let mut hops = 0usize;
        while let Some(next) = current {
            if self.strict_equal(prototype, &next) {
                return Ok(Value::Bool(true));
            }
            hops += 1;
            if hops > 256 {
                break;
            }
            current = self.value_internal_prototype_value(&next);
        }
        Ok(Value::Bool(false))
    }

    fn callable_name_and_length(&mut self, value: &Value) -> Option<(String, i64)> {
        match value {
            Value::Function(function) => Some((
                self.function_display_name(function),
                Self::function_length(function),
            )),
            Value::StringConstructor => Some(("String".to_string(), 1)),
            Value::RegExpConstructor => Some(("RegExp".to_string(), 2)),
            Value::BlobConstructor => Some(("Blob".to_string(), 0)),
            Value::UrlConstructor => Some(("URL".to_string(), 1)),
            Value::ArrayBufferConstructor => Some(("ArrayBuffer".to_string(), 1)),
            Value::PromiseConstructor => Some(("Promise".to_string(), 1)),
            Value::MapConstructor => Some(("Map".to_string(), 0)),
            Value::WeakMapConstructor => Some(("WeakMap".to_string(), 0)),
            Value::SetConstructor => Some(("Set".to_string(), 0)),
            Value::WeakSetConstructor => Some(("WeakSet".to_string(), 0)),
            Value::UrlSearchParamsConstructor => Some(("URLSearchParams".to_string(), 0)),
            Value::SymbolConstructor => Some(("Symbol".to_string(), 0)),
            Value::TypedArrayConstructor(kind) => Some((
                match kind {
                    TypedArrayConstructorKind::Concrete(kind) => kind.name(),
                    TypedArrayConstructorKind::Abstract => "TypedArray",
                }
                .to_string(),
                3,
            )),
            Value::Object(_) => match Self::callable_kind_from_value(value) {
                Some("bound_function") => {
                    let (target, _bound_this, bound_args) =
                        Self::bound_callable_components(value).ok()?;
                    let (target_name, target_length) = self.callable_name_and_length(&target)?;
                    let bound_name = format!("bound {target_name}");
                    let bound_length = target_length.saturating_sub(bound_args.len() as i64).max(0);
                    Some((bound_name, bound_length))
                }
                Some("receiver_builtin_method") => {
                    Self::receiver_builtin_callable_name_and_length(value)
                }
                Some("object_static_method") => Self::static_object_method_name_and_length(value),
                Some("reflect_static_method") => Self::static_reflect_method_name_and_length(value),
                Some(kind) => Self::object_backed_callable_name_and_length(kind)
                    .map(|(name, length)| (name.to_string(), length)),
                None => None,
            },
            _ => None,
        }
    }

    pub(crate) fn callable_source_text(&mut self, value: &Value) -> Option<String> {
        match value {
            Value::Function(function) if function.function_id != usize::MAX => {
                return Some(format!("__bt_function_ref__({})", function.function_id));
            }
            _ if !self.is_callable_value(value) => return None,
            Value::Object(_)
                if matches!(
                    Self::callable_kind_from_value(value),
                    Some("bound_function")
                ) =>
            {
                return Some("function () { [native code] }".to_string());
            }
            _ => {}
        }

        let name = self
            .callable_name_and_length(value)
            .map(|(name, _)| name)
            .unwrap_or_default();
        if name.is_empty() {
            Some("function () { [native code] }".to_string())
        } else {
            Some(format!("function {name}() {{ [native code] }}"))
        }
    }

    fn coerce_object_like_to_string_via_primitive_methods(
        &mut self,
        value: &Value,
        allow_symbol: bool,
    ) -> Result<String> {
        let mut saw_callable = false;
        for method_name in ["toString", "valueOf"] {
            let method = self.object_property_from_value(value, method_name)?;
            if !self.is_callable_value(&method) {
                continue;
            }
            saw_callable = true;
            let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
            let coerced = self.execute_callable_value_with_this_and_env(
                &method,
                &[],
                &event,
                None,
                Some(value.clone()),
            )?;
            if Self::is_primitive_value(&coerced) {
                if matches!(coerced, Value::Symbol(_)) {
                    if !allow_symbol {
                        return Err(Error::ScriptRuntime(
                            "Cannot convert a Symbol value to a string".into(),
                        ));
                    }
                }
                return Ok(self.coerce_to_string_for_string_context(&coerced));
            }
        }
        if saw_callable {
            return Err(Error::ScriptRuntime(
                "Cannot convert object to primitive value".into(),
            ));
        }
        Ok(self.coerce_to_string_for_string_context(value))
    }

    pub(crate) fn coerce_to_string_for_tostring(&mut self, value: &Value) -> Result<String> {
        match value {
            Value::Symbol(_) => Err(Error::ScriptRuntime(
                "Cannot convert a Symbol value to a string".into(),
            )),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(wrapped) = Self::string_wrapper_value_from_object(&entries) {
                    return Ok(wrapped);
                }
                if Self::symbol_wrapper_id_from_object(&entries).is_some() {
                    return Err(Error::ScriptRuntime(
                        "Cannot convert a Symbol value to a string".into(),
                    ));
                }
                drop(entries);
                self.coerce_object_like_to_string_via_primitive_methods(value, false)
            }
            Value::Date(_) => self.coerce_object_like_to_string_via_primitive_methods(value, false),
            _ => Ok(self.coerce_to_string_for_string_context(value)),
        }
    }

    pub(crate) fn coerce_to_string_for_string_constructor(
        &mut self,
        value: &Value,
    ) -> Result<String> {
        match value {
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(wrapped) = Self::string_wrapper_value_from_object(&entries) {
                    return Ok(wrapped);
                }
                drop(entries);
                self.coerce_object_like_to_string_via_primitive_methods(value, true)
            }
            Value::Date(_) => self.coerce_object_like_to_string_via_primitive_methods(value, true),
            _ => Ok(self.coerce_to_string_for_string_context(value)),
        }
    }

    pub(crate) fn coerce_to_string_for_string_context(&mut self, value: &Value) -> String {
        self.callable_source_text(value)
            .unwrap_or_else(|| value.as_string())
    }

    pub(crate) fn callable_function_surface_value(
        &mut self,
        value: &Value,
        key: &str,
    ) -> Option<Value> {
        match key {
            "call" | "apply" | "bind" | "toString" => {
                return Some(self.cached_function_surface_method_value(key));
            }
            "name" => {
                let (name, _) = self.callable_name_and_length(value)?;
                return Some(Value::String(name));
            }
            "length" => {
                let (_, length) = self.callable_name_and_length(value)?;
                return Some(Value::Number(length));
            }
            _ => {}
        }
        None
    }

    pub(crate) fn variant_callable_public_storage_key(value: &Value) -> Option<String> {
        match value {
            Value::StringConstructor => Some("String".to_string()),
            Value::SymbolConstructor => Some("Symbol".to_string()),
            Value::MapConstructor => Some("Map".to_string()),
            Value::WeakMapConstructor => Some("WeakMap".to_string()),
            Value::SetConstructor => Some("Set".to_string()),
            Value::WeakSetConstructor => Some("WeakSet".to_string()),
            Value::PromiseConstructor => Some("Promise".to_string()),
            Value::BlobConstructor => Some("Blob".to_string()),
            Value::ArrayBufferConstructor => Some("ArrayBuffer".to_string()),
            Value::RegExpConstructor => Some("RegExp".to_string()),
            Value::UrlSearchParamsConstructor => Some("URLSearchParams".to_string()),
            Value::TypedArrayConstructor(kind) => Some(format!(
                "TypedArrayConstructor:{}",
                match kind {
                    TypedArrayConstructorKind::Concrete(kind) => kind.name(),
                    TypedArrayConstructorKind::Abstract => "TypedArray",
                }
            )),
            _ => None,
        }
    }

    pub(crate) fn variant_callable_internal_prototype_value(&self, value: &Value) -> Option<Value> {
        let storage_key = Self::variant_callable_public_storage_key(value)?;
        let entries = self
            .script_runtime
            .variant_callable_public_properties
            .get(&storage_key)?;
        Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
    }

    pub(crate) fn new_string_wrapper_value(value: String) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
            Value::String(value),
        )])
    }

    pub(crate) fn new_boolean_wrapper_value(value: bool) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_BOOLEAN_WRAPPER_VALUE_KEY.to_string(),
            Value::Bool(value),
        )])
    }

    pub(crate) fn new_number_wrapper_value(value: Value) -> Value {
        Self::new_object_value(vec![(INTERNAL_NUMBER_WRAPPER_VALUE_KEY.to_string(), value)])
    }

    pub(crate) fn new_bigint_wrapper_value(value: JsBigInt) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_BIGINT_WRAPPER_VALUE_KEY.to_string(),
            Value::BigInt(value),
        )])
    }

    pub(crate) fn new_symbol_wrapper_value(symbol_id: usize) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_SYMBOL_WRAPPER_KEY.to_string(),
            Value::Number(symbol_id as i64),
        )])
    }

    pub(crate) fn box_primitive_value(value: Value) -> Value {
        match value {
            Value::String(text) => Self::new_string_wrapper_value(text),
            Value::Bool(value) => Self::new_boolean_wrapper_value(value),
            Value::Number(value) => Self::new_number_wrapper_value(Value::Number(value)),
            Value::Float(value) => Self::new_number_wrapper_value(Value::Float(value)),
            Value::BigInt(value) => Self::new_bigint_wrapper_value(value),
            Value::Symbol(symbol) => Self::new_symbol_wrapper_value(symbol.id),
            other => other,
        }
    }

    pub(crate) fn object_set_entry(entries: &mut impl ObjectEntryMut, key: String, value: Value) {
        entries.set_entry(key, value);
    }

    pub(crate) fn object_get_entry(
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> Option<Value> {
        entries.get_entry(key)
    }

    pub(crate) fn object_getter_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_OBJECT_GETTER_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_setter_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_OBJECT_SETTER_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_undefined_getter_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_OBJECT_UNDEFINED_GETTER_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_undefined_setter_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_OBJECT_UNDEFINED_SETTER_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_non_enumerable_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_NON_ENUMERABLE_PROPERTY_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn mark_object_properties_non_enumerable(
        entries: &mut impl ObjectEntryMut,
        keys: &[&str],
    ) {
        for key in keys {
            Self::object_set_entry(
                entries,
                Self::object_non_enumerable_storage_key(key),
                Value::Bool(true),
            );
        }
    }

    pub(crate) fn object_non_writable_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_NON_WRITABLE_PROPERTY_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_non_configurable_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_NON_CONFIGURABLE_PROPERTY_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_deleted_builtin_storage_key(property_key: &str) -> String {
        format!("{INTERNAL_DELETED_BUILTIN_PROPERTY_KEY_PREFIX}{property_key}")
    }

    pub(crate) fn object_getter_from_entries(
        entries: &(impl ObjectEntryLookup + ?Sized),
        property_key: &str,
    ) -> Option<Value> {
        let getter_key = Self::object_getter_storage_key(property_key);
        Self::object_get_entry(entries, &getter_key)
    }

    pub(crate) fn object_setter_from_entries(
        entries: &(impl ObjectEntryLookup + ?Sized),
        property_key: &str,
    ) -> Option<Value> {
        let setter_key = Self::object_setter_storage_key(property_key);
        Self::object_get_entry(entries, &setter_key)
    }

    pub(crate) fn has_object_getter_property(
        entries: &(impl ObjectEntryLookup + ?Sized),
        property_key: &str,
    ) -> bool {
        Self::object_getter_from_entries(entries, property_key).is_some()
            || matches!(
                Self::object_get_entry(
                    entries,
                    &Self::object_undefined_getter_storage_key(property_key),
                ),
                Some(Value::Bool(true))
            )
    }

    pub(crate) fn has_object_setter_property(
        entries: &(impl ObjectEntryLookup + ?Sized),
        property_key: &str,
    ) -> bool {
        Self::object_setter_from_entries(entries, property_key).is_some()
            || matches!(
                Self::object_get_entry(
                    entries,
                    &Self::object_undefined_setter_storage_key(property_key),
                ),
                Some(Value::Bool(true))
            )
    }

    pub(crate) fn has_object_accessor_property(
        entries: &(impl ObjectEntryLookup + ?Sized),
        property_key: &str,
    ) -> bool {
        Self::has_object_getter_property(entries, property_key)
            || Self::has_object_setter_property(entries, property_key)
    }

    pub(crate) fn is_writable_object_key(
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> bool {
        !matches!(
            Self::object_get_entry(entries, &Self::object_non_writable_storage_key(key)),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_configurable_object_key(
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> bool {
        !matches!(
            Self::object_get_entry(entries, &Self::object_non_configurable_storage_key(key)),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn mark_builtin_object_property_deleted(
        entries: &mut impl ObjectEntryMut,
        key: &str,
    ) {
        Self::object_set_entry(
            entries,
            Self::object_deleted_builtin_storage_key(key),
            Value::Bool(true),
        );
    }

    pub(crate) fn is_builtin_object_property_deleted(
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> bool {
        matches!(
            Self::object_get_entry(entries, &Self::object_deleted_builtin_storage_key(key)),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_callable_own_surface_key(key: &str) -> bool {
        matches!(key, "name" | "length")
    }

    pub(crate) fn deleted_callable_surface_fallback_value(key: &str) -> Option<Value> {
        match key {
            "name" => Some(Value::String(String::new())),
            "length" => Some(Value::Number(0)),
            _ => None,
        }
    }

    pub(crate) fn is_function_builtin_prototype_key(function: &FunctionValue, key: &str) -> bool {
        key == "prototype" && !function.is_arrow && !function.is_method
    }

    pub(crate) fn set_function_builtin_prototype_property(
        entries: &mut ObjectValue,
        value: Value,
        writable: bool,
    ) {
        Self::delete_object_property_entries(entries, "prototype");
        Self::object_set_entry(entries, "prototype".to_string(), value);
        Self::object_set_entry(
            entries,
            Self::object_non_enumerable_storage_key("prototype"),
            Value::Bool(true),
        );
        Self::object_set_entry(
            entries,
            Self::object_non_configurable_storage_key("prototype"),
            Value::Bool(true),
        );
        if !writable {
            Self::object_set_entry(
                entries,
                Self::object_non_writable_storage_key("prototype"),
                Value::Bool(true),
            );
        }
    }

    pub(crate) fn is_regexp_prototype_object(entries: &(impl ObjectEntryLookup + ?Sized)) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_REGEXP_PROTOTYPE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn regexp_default_property_value(key: &str) -> Option<Value> {
        match key {
            "source" => Some(Value::String("(?:)".to_string())),
            "flags" => Some(Value::String(String::new())),
            "global" | "ignoreCase" | "multiline" | "dotAll" | "sticky" | "hasIndices"
            | "unicode" | "unicodeSets" => Some(Value::Bool(false)),
            "lastIndex" => Some(Value::Number(0)),
            _ => None,
        }
    }

    pub(crate) fn regexp_instance_property_value(regex: &RegexValue, key: &str) -> Option<Value> {
        match key {
            "source" => Some(Value::String(regex.source.clone())),
            "flags" => Some(Value::String(regex.flags.clone())),
            "global" => Some(Value::Bool(regex.global)),
            "ignoreCase" => Some(Value::Bool(regex.ignore_case)),
            "multiline" => Some(Value::Bool(regex.multiline)),
            "dotAll" => Some(Value::Bool(regex.dot_all)),
            "sticky" => Some(Value::Bool(regex.sticky)),
            "hasIndices" => Some(Value::Bool(regex.has_indices)),
            "unicode" => Some(Value::Bool(regex.unicode)),
            "unicodeSets" => Some(Value::Bool(regex.unicode_sets)),
            "lastIndex" => Some(Value::Number(regex.last_index as i64)),
            _ => None,
        }
    }

    pub(crate) fn is_regexp_builtin_own_key(key: &str) -> bool {
        matches!(
            key,
            "source"
                | "flags"
                | "global"
                | "ignoreCase"
                | "multiline"
                | "dotAll"
                | "sticky"
                | "hasIndices"
                | "unicode"
                | "unicodeSets"
                | "lastIndex"
        )
    }

    fn invoke_object_getter(&mut self, getter: &Value, receiver: &Value) -> Result<Value> {
        if !self.is_callable_value(getter) {
            return Err(Error::ScriptRuntime("object getter is not callable".into()));
        }
        let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
        self.execute_callable_value_with_this_and_env(
            getter,
            &[],
            &event,
            None,
            Some(receiver.clone()),
        )
    }

    pub(crate) fn object_property_from_entries_with_getter(
        &mut self,
        receiver: &Value,
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> Result<Option<Value>> {
        if let Some(getter) = Self::object_getter_from_entries(entries, key) {
            return Ok(Some(self.invoke_object_getter(&getter, receiver)?));
        }
        if Self::has_object_accessor_property(entries, key) {
            return Ok(Some(Value::Undefined));
        }
        Ok(Self::object_get_entry(entries, key))
    }

    pub(crate) fn callable_kind_from_value(value: &Value) -> Option<&str> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        match Self::object_get_entry(&entries, INTERNAL_CALLABLE_KIND_KEY) {
            Some(Value::String(kind)) => Some(match kind.as_str() {
                "intl_collator_compare" => "intl_collator_compare",
                "intl_date_time_format" => "intl_date_time_format",
                "intl_duration_format" => "intl_duration_format",
                "intl_list_format" => "intl_list_format",
                "intl_number_format" => "intl_number_format",
                "intl_segmenter_segments_iterator" => "intl_segmenter_segments_iterator",
                "intl_segmenter_iterator_next" => "intl_segmenter_iterator_next",
                "readable_stream_async_iterator" => "readable_stream_async_iterator",
                "named_node_map_iterator" => "named_node_map_iterator",
                "iterator_self" => "iterator_self",
                "async_iterator_next" => "async_iterator_next",
                "async_iterator_return" => "async_iterator_return",
                "async_iterator_throw" => "async_iterator_throw",
                "async_iterator_self" => "async_iterator_self",
                "async_iterator_async_dispose" => "async_iterator_async_dispose",
                "async_generator_result_value" => "async_generator_result_value",
                "async_generator_result_done" => "async_generator_result_done",
                "async_generator_function_constructor" => "async_generator_function_constructor",
                "generator_function_constructor" => "generator_function_constructor",
                "boolean_constructor" => "boolean_constructor",
                "number_constructor" => "number_constructor",
                "bigint_constructor" => "bigint_constructor",
                "object_constructor" => "object_constructor",
                "object_static_method" => "object_static_method",
                "function_constructor" => "function_constructor",
                "node_list_constructor" => "node_list_constructor",
                "image_bitmap_constructor" => "image_bitmap_constructor",
                "text_track_constructor" => "text_track_constructor",
                "text_track_list_constructor" => "text_track_list_constructor",
                "time_ranges_constructor" => "time_ranges_constructor",
                "storage_constructor" => "storage_constructor",
                "cookie_store_constructor" => "cookie_store_constructor",
                "cache_storage_constructor" => "cache_storage_constructor",
                "cache_constructor" => "cache_constructor",
                "radio_node_list_constructor" => "radio_node_list_constructor",
                "html_collection_constructor" => "html_collection_constructor",
                "html_form_controls_collection_constructor" => {
                    "html_form_controls_collection_constructor"
                }
                "html_options_collection_constructor" => "html_options_collection_constructor",
                "event_target_constructor" => "event_target_constructor",
                "event_constructor" => "event_constructor",
                "custom_event_constructor" => "custom_event_constructor",
                "mouse_event_constructor" => "mouse_event_constructor",
                "keyboard_event_constructor" => "keyboard_event_constructor",
                "wheel_event_constructor" => "wheel_event_constructor",
                "navigate_event_constructor" => "navigate_event_constructor",
                "pointer_event_constructor" => "pointer_event_constructor",
                "error_event_constructor" => "error_event_constructor",
                "hash_change_event_constructor" => "hash_change_event_constructor",
                "before_unload_event_constructor" => "before_unload_event_constructor",
                "image_data_constructor" => "image_data_constructor",
                "dom_parser_constructor" => "dom_parser_constructor",
                "xml_serializer_constructor" => "xml_serializer_constructor",
                "document_constructor" => "document_constructor",
                "document_parse_html" => "document_parse_html",
                "document_parse_html_unsafe" => "document_parse_html_unsafe",
                "fetch_function" => "fetch_function",
                "match_media_function" => "match_media_function",
                "window_close_function" => "window_close_function",
                "window_open_function" => "window_open_function",
                "window_stop_function" => "window_stop_function",
                "window_focus_function" => "window_focus_function",
                "window_scroll_function" => "window_scroll_function",
                "window_scroll_by_function" => "window_scroll_by_function",
                "window_scroll_to_function" => "window_scroll_to_function",
                "window_move_by_function" => "window_move_by_function",
                "window_move_to_function" => "window_move_to_function",
                "window_resize_by_function" => "window_resize_by_function",
                "window_resize_to_function" => "window_resize_to_function",
                "window_post_message_function" => "window_post_message_function",
                "window_get_computed_style_function" => "window_get_computed_style_function",
                "computed_style_item" => "computed_style_item",
                "dom_rect_list_item" => "dom_rect_list_item",
                "window_alert_function" => "window_alert_function",
                "window_confirm_function" => "window_confirm_function",
                "window_print_function" => "window_print_function",
                "window_report_error_function" => "window_report_error_function",
                "window_prompt_function" => "window_prompt_function",
                "popup_window_close_function" => "popup_window_close_function",
                "popup_window_focus_function" => "popup_window_focus_function",
                "popup_window_print_function" => "popup_window_print_function",
                "popup_document_open_function" => "popup_document_open_function",
                "popup_document_write_function" => "popup_document_write_function",
                "popup_document_close_function" => "popup_document_close_function",
                "request_constructor" => "request_constructor",
                "file_constructor" => "file_constructor",
                "clipboard_item_constructor" => "clipboard_item_constructor",
                "clipboard_write" => "clipboard_write",
                "headers_constructor" => "headers_constructor",
                "worker_constructor" => "worker_constructor",
                "data_transfer_constructor" => "data_transfer_constructor",
                "option_constructor" => "option_constructor",
                "audio_constructor" => "audio_constructor",
                "text_encoder_constructor" => "text_encoder_constructor",
                "text_decoder_constructor" => "text_decoder_constructor",
                "text_encoder_stream_constructor" => "text_encoder_stream_constructor",
                "text_decoder_stream_constructor" => "text_decoder_stream_constructor",
                "text_encoder_get_encoding" => "text_encoder_get_encoding",
                "text_encoder_encode" => "text_encoder_encode",
                "text_encoder_encode_into" => "text_encoder_encode_into",
                "text_decoder_get_encoding" => "text_decoder_get_encoding",
                "text_decoder_get_fatal" => "text_decoder_get_fatal",
                "text_decoder_get_ignore_bom" => "text_decoder_get_ignore_bom",
                "text_decoder_decode" => "text_decoder_decode",
                "text_encoder_stream_get_encoding" => "text_encoder_stream_get_encoding",
                "text_encoder_stream_get_readable" => "text_encoder_stream_get_readable",
                "text_encoder_stream_get_writable" => "text_encoder_stream_get_writable",
                "text_decoder_stream_get_encoding" => "text_decoder_stream_get_encoding",
                "text_decoder_stream_get_fatal" => "text_decoder_stream_get_fatal",
                "text_decoder_stream_get_ignore_bom" => "text_decoder_stream_get_ignore_bom",
                "text_decoder_stream_get_readable" => "text_decoder_stream_get_readable",
                "text_decoder_stream_get_writable" => "text_decoder_stream_get_writable",
                "css_style_sheet_constructor" => "css_style_sheet_constructor",
                "css_style_sheet_replace_sync" => "css_style_sheet_replace_sync",
                "css_style_sheet_insert_rule" => "css_style_sheet_insert_rule",
                "computed_style_get_property_value" => "computed_style_get_property_value",
                "class_list_add" => "class_list_add",
                "class_list_remove" => "class_list_remove",
                "class_list_toggle" => "class_list_toggle",
                "class_list_contains" => "class_list_contains",
                "class_list_replace" => "class_list_replace",
                "class_list_item" => "class_list_item",
                "class_list_for_each" => "class_list_for_each",
                "class_list_keys" => "class_list_keys",
                "class_list_values" => "class_list_values",
                "class_list_entries" => "class_list_entries",
                "class_list_to_string" => "class_list_to_string",
                "named_node_map_item" => "named_node_map_item",
                "named_node_map_get_named_item" => "named_node_map_get_named_item",
                "named_node_map_set_named_item" => "named_node_map_set_named_item",
                "named_node_map_remove_named_item" => "named_node_map_remove_named_item",
                "named_node_map_get_named_item_ns" => "named_node_map_get_named_item_ns",
                "named_node_map_set_named_item_ns" => "named_node_map_set_named_item_ns",
                "named_node_map_remove_named_item_ns" => "named_node_map_remove_named_item_ns",
                "named_node_map_for_each" => "named_node_map_for_each",
                "named_node_map_keys" => "named_node_map_keys",
                "named_node_map_values" => "named_node_map_values",
                "named_node_map_entries" => "named_node_map_entries",
                "worker_main_post_message" => "worker_main_post_message",
                "worker_context_post_message" => "worker_context_post_message",
                "worker_terminate" => "worker_terminate",
                "intl_collator_get_compare" => "intl_collator_get_compare",
                "intl_date_time_format_get_format" => "intl_date_time_format_get_format",
                "intl_number_format_get_format" => "intl_number_format_get_format",
                "global_decode_uri" => "global_decode_uri",
                "global_decode_uri_component" => "global_decode_uri_component",
                "global_atob" => "global_atob",
                "global_btoa" => "global_btoa",
                "global_css_escape" => "global_css_escape",
                "global_structured_clone" => "global_structured_clone",
                "global_request_animation_frame" => "global_request_animation_frame",
                "global_set_timeout" => "global_set_timeout",
                "global_set_interval" => "global_set_interval",
                "global_cancel_animation_frame" => "global_cancel_animation_frame",
                "global_clear_interval" => "global_clear_interval",
                "global_clear_timeout" => "global_clear_timeout",
                "global_queue_microtask" => "global_queue_microtask",
                "create_image_bitmap" => "create_image_bitmap",
                "string_static_from_char_code" => "string_static_from_char_code",
                "string_static_from_code_point" => "string_static_from_code_point",
                "string_static_raw" => "string_static_raw",
                "number_static_method" => "number_static_method",
                "bigint_static_method" => "bigint_static_method",
                "regexp_static_method" => "regexp_static_method",
                "promise_static_method" => "promise_static_method",
                "array_buffer_static_method" => "array_buffer_static_method",
                "symbol_static_method" => "symbol_static_method",
                "typed_array_static_method" => "typed_array_static_method",
                "reflect_static_method" => "reflect_static_method",
                "function_call" => "function_call",
                "function_apply" => "function_apply",
                "function_bind" => "function_bind",
                "function_to_string" => "function_to_string",
                "bound_function" => "bound_function",
                "receiver_builtin_method" => "receiver_builtin_method",
                _ => return None,
            }),
            _ => None,
        }
    }

    pub(crate) fn data_attr_name_to_dataset_key(attr_name: &str) -> Option<String> {
        let raw = attr_name.strip_prefix("data-")?;
        if raw.is_empty() {
            return None;
        }
        let normalized = raw.to_ascii_lowercase();
        let chars = normalized.chars().collect::<Vec<_>>();
        let mut out = String::with_capacity(chars.len());
        let mut index = 0usize;
        while index < chars.len() {
            let ch = chars[index];
            if ch == '-' {
                if let Some(next) = chars.get(index + 1).copied() {
                    if next.is_ascii_lowercase() {
                        out.push(next.to_ascii_uppercase());
                        index += 2;
                        continue;
                    }
                }
                out.push(ch);
            } else {
                out.push(ch);
            }
            index += 1;
        }
        if out.is_empty() { None } else { Some(out) }
    }

    pub(crate) fn dataset_entries_for_node(&self, node: NodeId) -> Vec<(String, Value)> {
        let Some(element) = self.dom.element(node) else {
            return Vec::new();
        };
        let mut entries = element
            .attrs
            .iter()
            .filter_map(|(attr_name, attr_value)| {
                Self::data_attr_name_to_dataset_key(attr_name)
                    .map(|key| (key, Value::String(attr_value.clone())))
            })
            .collect::<Vec<_>>();
        entries.sort_by(|(left, _), (right, _)| left.cmp(right));
        entries
    }

    fn is_to_string_tag_property_key(&self, key: &str) -> bool {
        Self::symbol_id_from_storage_key(key)
            .and_then(|symbol_id| self.symbol_runtime.symbols_by_id.get(&symbol_id))
            .and_then(|symbol| symbol.description.as_deref())
            .is_some_and(|description| description == "Symbol.toStringTag")
            || key == "Symbol.toStringTag"
    }

    fn is_iterator_property_key(&self, key: &str) -> bool {
        Self::symbol_id_from_storage_key(key)
            .and_then(|symbol_id| self.symbol_runtime.symbols_by_id.get(&symbol_id))
            .and_then(|symbol| symbol.description.as_deref())
            .is_some_and(|description| description == "Symbol.iterator")
            || key == "Symbol.iterator"
    }

    fn is_string_method_name(name: &str) -> bool {
        matches!(
            name,
            "concat"
                | "endsWith"
                | "includes"
                | "normalize"
                | "slice"
                | "split"
                | "startsWith"
                | "substring"
        )
    }

    fn is_array_method_name(name: &str) -> bool {
        matches!(
            name,
            "forEach"
                | "map"
                | "flat"
                | "flatMap"
                | "filter"
                | "reduce"
                | "find"
                | "findIndex"
                | "some"
                | "every"
                | "values"
                | "keys"
                | "entries"
                | "fill"
                | "includes"
                | "slice"
                | "join"
                | "concat"
                | "add"
                | "remove"
                | "clear"
                | "push"
                | "pop"
                | "shift"
                | "unshift"
                | "splice"
                | "sort"
                | "reverse"
        )
    }

    fn is_class_list_method_name(name: &str) -> bool {
        matches!(
            name,
            "add"
                | "remove"
                | "toggle"
                | "contains"
                | "replace"
                | "item"
                | "forEach"
                | "keys"
                | "values"
                | "entries"
                | "toString"
        )
    }

    fn is_named_node_map_method_name(name: &str) -> bool {
        matches!(
            name,
            "item"
                | "getNamedItem"
                | "setNamedItem"
                | "removeNamedItem"
                | "getNamedItemNS"
                | "setNamedItemNS"
                | "removeNamedItemNS"
                | "forEach"
                | "keys"
                | "values"
                | "entries"
        )
    }

    fn is_typed_array_method_name(name: &str) -> bool {
        matches!(
            name,
            "at" | "copyWithin"
                | "entries"
                | "join"
                | "keys"
                | "slice"
                | "subarray"
                | "values"
                | "with"
        )
    }

    pub(crate) fn function_own_property_value(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
        include_to_string: bool,
    ) -> Value {
        match key {
            "constructor" => {
                if function.is_generator {
                    if function.is_async {
                        self.new_async_generator_function_constructor_value()
                    } else {
                        self.new_generator_function_constructor_value()
                    }
                } else {
                    Value::Undefined
                }
            }
            "prototype" => {
                if function.is_arrow || function.is_method {
                    Value::Undefined
                } else {
                    Value::Object(function.prototype_object.clone())
                }
            }
            "length" => {
                let mut length = 0_i64;
                for param in &function.handler.params {
                    if param.is_rest || param.default.is_some() {
                        break;
                    }
                    length += 1;
                }
                Value::Number(length)
            }
            "name" => Value::String(self.function_display_name(function)),
            "call" | "apply" | "bind" => self.cached_function_surface_method_value(key),
            "toString" if include_to_string => self.cached_function_surface_method_value(key),
            _ => Value::Undefined,
        }
    }

    fn object_property_from_string_value(&self, text: &str, key: &str) -> Value {
        if key == "length" {
            Value::Number(Self::string_char_len(text) as i64)
        } else if key == "constructor" {
            Value::StringConstructor
        } else if self.is_iterator_property_key(key) {
            Self::new_receiver_builtin_callable("string", "iterator")
        } else if matches!(key, "toString" | "valueOf") || Self::is_string_method_name(key) {
            Self::new_receiver_builtin_callable("string", key)
        } else if let Ok(index) = key.parse::<usize>() {
            Self::string_char_at(text, index)
                .map(|ch| Value::String(ch.to_string()))
                .unwrap_or(Value::Undefined)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_bool_value(&self, key: &str) -> Value {
        if key == "constructor" {
            self.script_runtime
                .env
                .get("Boolean")
                .cloned()
                .unwrap_or_else(Self::new_boolean_constructor_callable)
        } else if matches!(key, "toString" | "valueOf") {
            Self::new_receiver_builtin_callable("boolean", key)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_number_value(&self, key: &str) -> Value {
        if key == "constructor" {
            self.script_runtime
                .env
                .get("Number")
                .cloned()
                .unwrap_or_else(Self::new_number_constructor_callable)
        } else if matches!(
            key,
            "toExponential" | "toFixed" | "toLocaleString" | "toPrecision" | "toString" | "valueOf"
        ) {
            Self::new_receiver_builtin_callable("number", key)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_bigint_value(&self, key: &str) -> Value {
        if key == "constructor" {
            self.script_runtime
                .env
                .get("BigInt")
                .cloned()
                .unwrap_or_else(Self::new_bigint_constructor_callable)
        } else if matches!(key, "toLocaleString" | "toString" | "valueOf") {
            Self::new_receiver_builtin_callable("bigint", key)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_array_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        values: &Rc<RefCell<ArrayValue>>,
        key: &str,
    ) -> Result<Value> {
        let values = values.borrow();
        if Self::is_dom_rect_list_value(&values) && self.is_to_string_tag_property_key(key) {
            return Ok(Value::String("DOMRectList".to_string()));
        }
        if key == "length" {
            return Ok(Value::Number(values.len() as i64));
        }
        let has_placeholder_builtin =
            Self::placeholder_backed_array_builtin_surface_exists(&values, key);
        if has_placeholder_builtin {
            if let Some(value) = Self::placeholder_backed_array_builtin_property_value(&values, key)
            {
                return Ok(value);
            }
            return Ok(Value::Undefined);
        }
        let has_explicit_prototype =
            Self::object_get_entry(&values.properties, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
        if let Ok(index) = key.parse::<usize>() {
            if index < values.len() && !Self::array_index_is_hole(&values, index) {
                return Ok(values[index].clone());
            }
            drop(values);
            if has_explicit_prototype {
                return Ok(self
                    .inherited_property_from_value_prototype_chain_with_receiver(
                        owner, receiver, key,
                    )?
                    .unwrap_or(Value::Undefined));
            }
            return Ok(Value::Undefined);
        }
        if let Some(value) = Self::object_get_entry(&values.properties, key) {
            return Ok(value);
        }
        if !has_explicit_prototype {
            if self.is_iterator_property_key(key) {
                return Ok(Self::new_receiver_builtin_callable("array", "values"));
            }
            if Self::is_array_method_name(key) {
                return Ok(Self::new_receiver_builtin_callable("array", key));
            }
            return Ok(Value::Undefined);
        }
        drop(values);
        Ok(self
            .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
            .unwrap_or(Value::Undefined))
    }

    fn object_property_from_node_list_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        nodes: &Rc<RefCell<NodeListValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_override = {
            let nodes_ref = nodes.borrow();
            self.object_property_from_entries_with_getter(receiver, &nodes_ref.properties, key)?
        };
        if let Some(value) = own_override {
            return Ok(value);
        }
        let has_explicit_prototype = {
            let nodes_ref = nodes.borrow();
            Self::object_get_entry(&nodes_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some()
        };
        if key == "length" {
            return Ok(Value::Number(self.node_list_len(nodes) as i64));
        }
        if let Ok(index) = key.parse::<usize>() {
            if let Some(node) = self.node_list_get(nodes, index) {
                return Ok(self.node_list_item_value(nodes, node));
            }
            if has_explicit_prototype {
                return Ok(self
                    .inherited_property_from_value_prototype_chain_with_receiver(
                        owner, receiver, key,
                    )?
                    .unwrap_or(Value::Undefined));
            }
            return Ok(Value::Undefined);
        }
        if let Some(value) = self.html_collection_named_property_value(nodes, key) {
            return Ok(value);
        }
        Ok(self
            .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
            .unwrap_or(Value::Undefined))
    }

    fn object_property_from_typed_array_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        values: &Rc<RefCell<TypedArrayValue>>,
        key: &str,
    ) -> Result<Value> {
        let value_ref = values.borrow();
        let has_explicit_prototype =
            Self::object_get_entry(&value_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
        let kind = value_ref.kind;
        drop(value_ref);
        if key == "length" {
            return Ok(Value::Number(values.borrow().observed_length() as i64));
        }
        if let Ok(index) = key.parse::<usize>() {
            let snapshot = self.typed_array_snapshot(values)?;
            if let Some(value) = snapshot.get(index) {
                return Ok(value.clone());
            }
            if has_explicit_prototype {
                return Ok(self
                    .inherited_property_from_value_prototype_chain_with_receiver(
                        owner, receiver, key,
                    )?
                    .unwrap_or(Value::Undefined));
            }
            return Ok(Value::Undefined);
        }
        if let Some(value) = Self::object_get_entry(&values.borrow().properties, key) {
            return Ok(value);
        }
        if !has_explicit_prototype {
            match key {
                "constructor" => {
                    return Ok(Value::TypedArrayConstructor(
                        TypedArrayConstructorKind::Concrete(kind),
                    ));
                }
                "byteLength" => {
                    return Ok(Value::Number(values.borrow().observed_byte_length() as i64));
                }
                "byteOffset" => {
                    let value_ref = values.borrow();
                    let byte_offset = if value_ref.observed_length() == 0
                        && value_ref.byte_offset >= value_ref.buffer.borrow().byte_length()
                    {
                        0
                    } else {
                        value_ref.byte_offset
                    };
                    return Ok(Value::Number(byte_offset as i64));
                }
                "buffer" => {
                    return Ok(Value::ArrayBuffer(values.borrow().buffer.clone()));
                }
                "BYTES_PER_ELEMENT" => {
                    return Ok(Value::Number(kind.bytes_per_element() as i64));
                }
                _ => {}
            }
            if self.is_iterator_property_key(key) {
                return Ok(Self::new_receiver_builtin_callable("typed_array", "values"));
            }
            if Self::is_typed_array_method_name(key) {
                return Ok(Self::new_receiver_builtin_callable("typed_array", key));
            }
            return Ok(Value::Undefined);
        }
        Ok(self
            .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
            .unwrap_or(Value::Undefined))
    }

    fn object_property_from_promise_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        promise: &Rc<RefCell<PromiseValue>>,
        key: &str,
    ) -> Result<Value> {
        if !self.strict_equal(owner, receiver) {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }
        if key == "constructor" {
            return Ok(Value::PromiseConstructor);
        }
        if matches!(key, "then" | "catch" | "finally") {
            return Ok(Self::new_receiver_builtin_callable("promise", key));
        }
        let promise = promise.borrow();
        if key == "status" {
            let status = match &promise.state {
                PromiseState::Pending => "pending",
                PromiseState::Fulfilled(_) => "fulfilled",
                PromiseState::Rejected(_) => "rejected",
            };
            Ok(Value::String(status.to_string()))
        } else {
            Ok(Value::Undefined)
        }
    }

    fn object_property_from_map_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        map: &Rc<RefCell<MapValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_value = {
            let map_ref = map.borrow();
            self.object_property_from_entries_with_getter(receiver, &map_ref.properties, key)?
        };
        if let Some(value) = own_value {
            return Ok(value);
        }
        let use_prototype_chain = {
            let map_ref = map.borrow();
            !self.strict_equal(owner, receiver)
                || Self::object_get_entry(&map_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .is_some()
        };
        if use_prototype_chain {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }

        let map = map.borrow();
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        Ok(if key == "size" {
            Value::Number(map.entries.len() as i64)
        } else if key_is_to_string_tag {
            Value::String("Map".to_string())
        } else if key == "constructor" {
            Value::MapConstructor
        } else if self.is_iterator_property_key(key) {
            Self::new_receiver_builtin_callable("map", "entries")
        } else if Self::is_map_method_name(key) {
            Self::new_receiver_builtin_callable("map", key)
        } else {
            Value::Undefined
        })
    }

    fn object_property_from_weak_map_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        weak_map: &Rc<RefCell<WeakMapValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_value = {
            let weak_map_ref = weak_map.borrow();
            self.object_property_from_entries_with_getter(receiver, &weak_map_ref.properties, key)?
        };
        if let Some(value) = own_value {
            return Ok(value);
        }
        let use_prototype_chain = {
            let weak_map_ref = weak_map.borrow();
            !self.strict_equal(owner, receiver)
                || Self::object_get_entry(&weak_map_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .is_some()
        };
        if use_prototype_chain {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }

        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        Ok(if key_is_to_string_tag {
            Value::String("WeakMap".to_string())
        } else if key == "constructor" {
            Value::WeakMapConstructor
        } else if Self::is_weak_map_method_name(key) {
            Self::new_receiver_builtin_callable("weak_map", key)
        } else {
            Value::Undefined
        })
    }

    fn object_property_from_weak_set_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        weak_set: &Rc<RefCell<WeakSetValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_value = {
            let weak_set_ref = weak_set.borrow();
            self.object_property_from_entries_with_getter(receiver, &weak_set_ref.properties, key)?
        };
        if let Some(value) = own_value {
            return Ok(value);
        }
        let use_prototype_chain = {
            let weak_set_ref = weak_set.borrow();
            !self.strict_equal(owner, receiver)
                || Self::object_get_entry(&weak_set_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .is_some()
        };
        if use_prototype_chain {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }

        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        Ok(if key_is_to_string_tag {
            Value::String("WeakSet".to_string())
        } else if key == "constructor" {
            Value::WeakSetConstructor
        } else if Self::is_weak_set_method_name(key) {
            Self::new_receiver_builtin_callable("weak_set", key)
        } else {
            Value::Undefined
        })
    }

    fn object_property_from_set_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        set: &Rc<RefCell<SetValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_value = {
            let set_ref = set.borrow();
            self.object_property_from_entries_with_getter(receiver, &set_ref.properties, key)?
        };
        if let Some(value) = own_value {
            return Ok(value);
        }
        let use_prototype_chain = {
            let set_ref = set.borrow();
            !self.strict_equal(owner, receiver)
                || Self::object_get_entry(&set_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .is_some()
        };
        if use_prototype_chain {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }

        let set = set.borrow();
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        Ok(if key == "size" {
            Value::Number(set.values.len() as i64)
        } else if key_is_to_string_tag {
            Value::String("Set".to_string())
        } else if key == "constructor" {
            Value::SetConstructor
        } else if self.is_iterator_property_key(key) {
            Self::new_receiver_builtin_callable("set", "values")
        } else if Self::is_set_method_name(key) {
            Self::new_receiver_builtin_callable("set", key)
        } else {
            Value::Undefined
        })
    }

    fn object_property_from_form_data_value(
        &self,
        _entries: &Rc<RefCell<Vec<(String, String)>>>,
        key: &str,
    ) -> Value {
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        if key_is_to_string_tag {
            Value::String("FormData".to_string())
        } else if self.is_iterator_property_key(key) {
            Self::new_receiver_builtin_callable("form_data", "entries")
        } else if matches!(
            key,
            "append" | "set" | "delete" | "entries" | "keys" | "values" | "get" | "getAll" | "has"
        ) {
            Self::new_receiver_builtin_callable("form_data", key)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_blob_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        blob: &Rc<RefCell<BlobValue>>,
        key: &str,
    ) -> Result<Value> {
        if !self.strict_equal(owner, receiver) {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }
        let blob = blob.borrow();
        Ok(match key {
            "size" => Value::Number(blob.bytes.len() as i64),
            "type" => Value::String(blob.mime_type.clone()),
            "constructor" => Value::BlobConstructor,
            "arrayBuffer" | "bytes" | "slice" | "stream" | "text" => {
                Self::new_receiver_builtin_callable("blob", key)
            }
            _ => Value::Undefined,
        })
    }

    fn object_property_from_array_buffer_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        buffer: &Rc<RefCell<ArrayBufferValue>>,
        key: &str,
    ) -> Result<Value> {
        if !self.strict_equal(owner, receiver) {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }
        let buffer = buffer.borrow();
        Ok(match key {
            "byteLength" => Value::Number(buffer.byte_length() as i64),
            "detached" => Value::Bool(buffer.detached),
            "maxByteLength" => Value::Number(buffer.max_byte_length() as i64),
            "resizable" => Value::Bool(buffer.resizable()),
            "constructor" => Value::ArrayBufferConstructor,
            "resize" | "slice" | "transfer" | "transferToFixedLength" => {
                Self::new_receiver_builtin_callable("array_buffer", key)
            }
            _ => Value::Undefined,
        })
    }

    fn object_property_from_symbol_value(symbol: &Rc<SymbolValue>, key: &str) -> Value {
        match key {
            "description" => symbol
                .description
                .as_ref()
                .map(|value| Value::String(value.clone()))
                .unwrap_or(Value::Undefined),
            "constructor" => Value::SymbolConstructor,
            "toString" | "valueOf" => Self::new_receiver_builtin_callable("symbol", key),
            _ => Value::Undefined,
        }
    }

    fn object_property_from_regexp_value(
        &mut self,
        owner: &Value,
        receiver: &Value,
        regex: &Rc<RefCell<RegexValue>>,
        key: &str,
    ) -> Result<Value> {
        let own_value = {
            let regex_ref = regex.borrow();
            self.object_property_from_entries_with_getter(receiver, &regex_ref.properties, key)?
        };
        if let Some(value) = own_value {
            return Ok(value);
        }
        let use_prototype_chain = {
            let regex_ref = regex.borrow();
            !self.strict_equal(owner, receiver)
                || Self::object_get_entry(&regex_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .is_some()
        };
        if use_prototype_chain {
            return Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined));
        }

        let regex = regex.borrow();
        if key == "lastIndex" {
            Ok(Value::Number(regex.last_index as i64))
        } else {
            Ok(self
                .inherited_property_from_value_prototype_chain_with_receiver(owner, receiver, key)?
                .unwrap_or(Value::Undefined))
        }
    }

    fn object_property_from_node_value(&mut self, node: &NodeId, key: &str) -> Result<Value> {
        let is_canvas = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("canvas"))
            .unwrap_or(false);
        let is_select = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("select"))
            .unwrap_or(false);
        let is_datalist = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("datalist"))
            .unwrap_or(false);
        let is_input = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false);
        let is_option = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("option"))
            .unwrap_or(false);
        let is_textarea = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("textarea"))
            .unwrap_or(false);
        let is_output = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("output"))
            .unwrap_or(false);
        let is_button = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("button"))
            .unwrap_or(false);
        let is_form = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("form"))
            .unwrap_or(false);
        let is_media = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video"))
            .unwrap_or(false);
        let is_form_associated_control = is_form_control(&self.dom, *node);
        let is_labelable_control = self.is_labelable_control(*node);
        let is_col_or_colgroup = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("col") || tag.eq_ignore_ascii_case("colgroup"))
            .unwrap_or(false);
        let is_table_cell = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th"))
            .unwrap_or(false);

        if is_select {
            if let Ok(index) = key.parse::<usize>() {
                return Ok(self
                    .select_option_nodes(*node)
                    .get(index)
                    .copied()
                    .map(Value::Node)
                    .unwrap_or(Value::Undefined));
            }
        }

        if self.node_explicit_own_property_overrides_dom_property(*node, key) {
            let entries = self.node_expando_entries(*node);
            if let Some(value) =
                self.object_property_from_entries_with_getter(&Value::Node(*node), &entries, key)?
            {
                return Ok(value);
            }
        }

        if let Some(value) = self.node_receiver_builtin_method(*node, key) {
            return Ok(value);
        }

        match key {
            "nodeType" => Ok(Value::Number(self.node_type_number(*node))),
            "nodeName" => Ok(Value::String(self.node_name(*node))),
            "nodeValue" => Ok(self.node_value(*node)),
            "ownerDocument" => Ok(self
                .node_owner_document(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "parentNode" => Ok(self
                .dom
                .parent(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "parentElement" => Ok(self
                .node_parent_element(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "nextSibling" => Ok(self
                .node_next_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "previousSibling" => Ok(self
                .node_previous_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "isConnected" => Ok(Value::Bool(self.dom.is_connected(*node))),
            "childNodes" => Ok(self.child_nodes_live_list_value(*node)),
            "attributes" => {
                if self.dom.element(*node).is_some() {
                    Ok(self.named_node_map_live_value(*node))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "children" => Ok(self.child_elements_live_list_value(*node)),
            "childElementCount" => Ok(Value::Number(self.dom.child_element_count(*node) as i64)),
            "firstChild" => Ok(self.dom.nodes[node.0]
                .children
                .first()
                .copied()
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "lastChild" => Ok(self.dom.nodes[node.0]
                .children
                .last()
                .copied()
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "firstElementChild" => Ok(self
                .dom
                .first_element_child(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "lastElementChild" => Ok(self
                .dom
                .last_element_child(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "nextElementSibling" => Ok(self
                .dom
                .next_element_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "previousElementSibling" => Ok(self
                .dom
                .previous_element_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "shadowRoot" => Ok(self.shadow_root_property_value(*node)),
            "content"
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("template")) =>
            {
                self.template_content_fragment_value(*node)
            }
            "textContent" => Ok(self.node_text_content_value(*node)),
            "innerText" => Ok(Value::String(self.dom.text_content(*node))),
            "innerHTML" => Ok(Value::String(self.dom.inner_html(*node)?)),
            "outerHTML" => Ok(Value::String(self.dom.outer_html(*node)?)),
            "defaultValue" => {
                if is_input || is_textarea || is_output {
                    Ok(Value::String(
                        self.dom
                            .element(*node)
                            .map(|element| element.default_value.clone())
                            .unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "value" => Ok(Value::String(self.dom.value(*node)?)),
            "files" => self.input_files_value(*node),
            "valueAsNumber" => Ok(Self::number_value(self.input_value_as_number(*node)?)),
            "valueAsDate" => Ok(self
                .input_value_as_date_ms(*node)?
                .map(Self::new_date_value)
                .unwrap_or(Value::Null)),
            "defaultChecked" => {
                if is_input {
                    Ok(Value::Bool(
                        self.dom
                            .element(*node)
                            .map(|element| element.default_checked)
                            .unwrap_or(false),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "checked" => Ok(Value::Bool(self.dom.checked(*node)?)),
            "defaultSelected" => {
                if is_option {
                    Ok(Value::Bool(
                        self.dom
                            .element(*node)
                            .map(|element| element.default_selected)
                            .unwrap_or(false),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "selected" => {
                if is_option {
                    Ok(Value::Bool(
                        self.dom
                            .element(*node)
                            .map(|element| element.selected)
                            .unwrap_or(false),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "disabled" => Ok(Value::Bool(self.dom.disabled(*node))),
            "required" => Ok(Value::Bool(self.dom.required(*node))),
            "multiple" => {
                if is_select || is_input {
                    Ok(Value::Bool(self.dom.attr(*node, "multiple").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "readonly" | "readOnly" => Ok(Value::Bool(self.dom.readonly(*node))),
            "autocomplete" => Ok(Value::String(
                self.dom.attr(*node, "autocomplete").unwrap_or_default(),
            )),
            "form" => {
                if is_form_associated_control {
                    Ok(self
                        .resolve_form_for_submit(*node)
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "elements" => {
                if is_form {
                    self.form_elements_live_list_value(*node)
                } else {
                    Ok(Value::Undefined)
                }
            }
            "action" => {
                if is_form {
                    Ok(Value::String(
                        self.form_action_property_value_for_node(*node),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "method" => {
                if is_form {
                    Ok(Value::String(
                        self.dom.attr(*node, "method").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "enctype" | "encoding" => {
                if is_form {
                    Ok(Value::String(
                        self.dom.attr(*node, "enctype").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "target" => {
                if is_form
                    || self.dom.tag_name(*node).is_some_and(|tag| {
                        tag.eq_ignore_ascii_case("a")
                            || tag.eq_ignore_ascii_case("area")
                            || tag.eq_ignore_ascii_case("base")
                    })
                {
                    Ok(Value::String(
                        self.dom.attr(*node, "target").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "acceptCharset" => {
                if is_form {
                    Ok(Value::String(
                        self.dom.attr(*node, "accept-charset").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "noValidate" => {
                if is_form {
                    Ok(Value::Bool(self.dom.attr(*node, "novalidate").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "command" => {
                if is_button {
                    Ok(Value::String(
                        self.dom.attr(*node, "command").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "commandForElement" => {
                if is_button {
                    Ok(self
                        .dom
                        .attr(*node, "commandfor")
                        .and_then(|raw| raw.split_whitespace().next().map(str::to_string))
                        .and_then(|id_ref| self.dom.by_id(&id_ref))
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formAction" => {
                if is_button || is_input {
                    Ok(Value::String(
                        self.submitter_form_action_property_value_for_node(*node),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "href" => Ok(Value::String(self.resolve_anchor_href(*node))),
            "download"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "download").unwrap_or_default(),
                ))
            }
            "hreflang"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("link")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "hreflang").unwrap_or_default(),
                ))
            }
            "ping"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "ping").unwrap_or_default(),
                ))
            }
            "referrerPolicy" | "referrerpolicy"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("img")
                        || tag.eq_ignore_ascii_case("link")
                        || tag.eq_ignore_ascii_case("script")
                        || tag.eq_ignore_ascii_case("iframe")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "referrerpolicy").unwrap_or_default(),
                ))
            }
            "rel"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("link")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "rel").unwrap_or_default(),
                ))
            }
            "alt"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("img")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "alt").unwrap_or_default(),
                ))
            }
            "charset"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "charset").unwrap_or_default(),
                ))
            }
            "coords"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "coords").unwrap_or_default(),
                ))
            }
            "rev"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "rev").unwrap_or_default(),
                ))
            }
            "shape"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "shape").unwrap_or_default(),
                ))
            }
            "noHref" | "nohref"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "nohref").is_some()))
            }
            "formEnctype" => {
                if is_button {
                    Ok(Value::String(
                        self.dom.attr(*node, "formenctype").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formMethod" => {
                if is_button {
                    Ok(Value::String(
                        self.dom.attr(*node, "formmethod").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formNoValidate" => {
                if is_button {
                    Ok(Value::Bool(
                        self.dom.attr(*node, "formnovalidate").is_some(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formTarget" => {
                if is_button {
                    Ok(Value::String(
                        self.dom.attr(*node, "formtarget").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "labels" => {
                if is_labelable_control {
                    Ok(Self::new_static_node_list_value(
                        self.labels_for_control_node(*node),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "id" => Ok(Value::String(
                self.dom.attr(*node, "id").unwrap_or_default(),
            )),
            "name" => Ok(Value::String(
                self.dom.attr(*node, "name").unwrap_or_default(),
            )),
            "interestForElement" => {
                if is_button {
                    Ok(self
                        .dom
                        .attr(*node, "interestfor")
                        .and_then(|raw| raw.split_whitespace().next().map(str::to_string))
                        .and_then(|id_ref| self.dom.by_id(&id_ref))
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "popoverTargetAction" => {
                if is_button {
                    Ok(Value::String(
                        self.dom
                            .attr(*node, "popovertargetaction")
                            .unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "popoverTargetElement" => {
                if is_button {
                    Ok(self
                        .dom
                        .attr(*node, "popovertarget")
                        .and_then(|raw| raw.split_whitespace().next().map(str::to_string))
                        .and_then(|id_ref| self.dom.by_id(&id_ref))
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "lang" => Ok(Value::String(
                self.dom.attr(*node, "lang").unwrap_or_default(),
            )),
            "dir" => Ok(Value::String(self.resolved_dir_for_node(*node))),
            "accessKey" | "accesskey" => Ok(Value::String(
                self.dom.attr(*node, "accesskey").unwrap_or_default(),
            )),
            "autocapitalize" => Ok(Value::String(
                self.dom.attr(*node, "autocapitalize").unwrap_or_default(),
            )),
            "autocorrect" => Ok(Value::String(
                self.dom.attr(*node, "autocorrect").unwrap_or_default(),
            )),
            "contentEditable" | "contenteditable" => Ok(Value::String(
                self.content_editable_property_value_for_node(*node),
            )),
            "draggable" => Ok(Value::Bool(self.draggable_property_value_for_node(*node))),
            "enterKeyHint" | "enterkeyhint" => Ok(Value::String(
                self.dom.attr(*node, "enterkeyhint").unwrap_or_default(),
            )),
            "inert" => Ok(Value::Bool(self.dom.has_attr(*node, "inert")?)),
            "inputMode" | "inputmode" => Ok(Value::String(
                self.dom.attr(*node, "inputmode").unwrap_or_default(),
            )),
            "nonce" => Ok(Value::String(
                self.dom.attr(*node, "nonce").unwrap_or_default(),
            )),
            "popover" => Ok(Value::String(
                self.dom.attr(*node, "popover").unwrap_or_default(),
            )),
            "spellcheck" => Ok(Value::Bool(self.spellcheck_property_value_for_node(*node))),
            "tabIndex" | "tabindex" => Ok(Value::Number(
                self.reflected_i64_attribute_or_default(*node, "tabindex", -1),
            )),
            "translate" => Ok(Value::Bool(self.translate_property_value_for_node(*node))),
            "cite" => Ok(Value::String(
                self.reflected_url_attribute_or_empty(*node, "cite"),
            )),
            "dateTime" | "datetime" => Ok(Value::String(
                self.dom.attr(*node, "datetime").unwrap_or_default(),
            )),
            "clear" => Ok(Value::String(
                self.dom.attr(*node, "clear").unwrap_or_default(),
            )),
            "align" => Ok(Value::String(
                self.dom.attr(*node, "align").unwrap_or_default(),
            )),
            "aLink" | "alink" => Ok(Value::String(
                self.dom.attr(*node, "alink").unwrap_or_default(),
            )),
            "background" => Ok(Value::String(
                self.dom.attr(*node, "background").unwrap_or_default(),
            )),
            "bgColor" | "bgcolor" => Ok(Value::String(
                self.dom.attr(*node, "bgcolor").unwrap_or_default(),
            )),
            "bottomMargin" | "bottommargin" => Ok(Value::String(
                self.dom.attr(*node, "bottommargin").unwrap_or_default(),
            )),
            "leftMargin" | "leftmargin" => Ok(Value::String(
                self.dom.attr(*node, "leftmargin").unwrap_or_default(),
            )),
            "link" => Ok(Value::String(
                self.dom.attr(*node, "link").unwrap_or_default(),
            )),
            "rightMargin" | "rightmargin" => Ok(Value::String(
                self.dom.attr(*node, "rightmargin").unwrap_or_default(),
            )),
            "text" => Ok(Value::String(
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
                {
                    self.dom.attr(*node, "text").unwrap_or_default()
                } else {
                    self.dom.text_content(*node)
                },
            )),
            "topMargin" | "topmargin" => Ok(Value::String(
                self.dom.attr(*node, "topmargin").unwrap_or_default(),
            )),
            "vLink" | "vlink" => Ok(Value::String(
                self.dom.attr(*node, "vlink").unwrap_or_default(),
            )),
            "title" => Ok(Value::String(
                self.dom.attr(*node, "title").unwrap_or_default(),
            )),
            "colSpan" | "colspan" if is_table_cell => {
                Ok(Value::Number(self.table_cell_col_span_value(*node)))
            }
            "rowSpan" | "rowspan" if is_table_cell => {
                Ok(Value::Number(self.table_cell_row_span_value(*node)))
            }
            "span" if is_col_or_colgroup => Ok(Value::Number(self.col_span_value(*node))),
            "type" => {
                if is_select {
                    Ok(Value::String(self.select_type_property_value(*node)))
                } else if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("button"))
                {
                    let normalized = self
                        .dom
                        .attr(*node, "type")
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                        .map(|value| {
                            if value.eq_ignore_ascii_case("reset") {
                                "reset".to_string()
                            } else if value.eq_ignore_ascii_case("button") {
                                "button".to_string()
                            } else {
                                "submit".to_string()
                            }
                        })
                        .unwrap_or_else(|| "submit".to_string());
                    Ok(Value::String(normalized))
                } else {
                    Ok(Value::String(
                        self.dom.attr(*node, "type").unwrap_or_default(),
                    ))
                }
            }
            "kind" if self.is_track_element(*node) => {
                Ok(Value::String(self.normalized_track_kind(*node)))
            }
            "track" if self.is_track_element(*node) => Ok(self.text_track_object_value(*node)),
            "srclang" | "srcLang" if self.is_track_element(*node) => Ok(Value::String(
                self.dom.attr(*node, "srclang").unwrap_or_default(),
            )),
            "label" if self.is_track_element(*node) => Ok(Value::String(
                self.dom.attr(*node, "label").unwrap_or_default(),
            )),
            "default" if self.is_track_element(*node) => {
                Ok(Value::Bool(self.dom.attr(*node, "default").is_some()))
            }
            "readyState" if self.is_track_element(*node) => Ok(Value::Number(0)),
            "defaultMuted"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "muted").is_some()))
            }
            "autoplay"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "autoplay").is_some()))
            }
            "controls"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "controls").is_some()))
            }
            "loop"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "loop").is_some()))
            }
            "muted"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "muted").is_some()))
            }
            "controlsList" | "controlslist" => Ok(Value::String(
                self.dom.attr(*node, "controlslist").unwrap_or_default(),
            )),
            "crossOrigin" | "crossorigin" => Ok(Value::String(
                self.dom.attr(*node, "crossorigin").unwrap_or_default(),
            )),
            "disableRemotePlayback" | "disableremoteplayback" => Ok(Value::Bool(
                self.dom.attr(*node, "disableremoteplayback").is_some(),
            )),
            "disablePictureInPicture" | "disablepictureinpicture" => Ok(Value::Bool(
                self.dom.attr(*node, "disablepictureinpicture").is_some(),
            )),
            "media" => Ok(Value::String(
                self.dom.attr(*node, "media").unwrap_or_default(),
            )),
            "playsInline" | "playsinline" => {
                Ok(Value::Bool(self.dom.attr(*node, "playsinline").is_some()))
            }
            "paused"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_boolean_state_value(*node, INTERNAL_MEDIA_PAUSED_KEY, true))
            }
            "ended"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(false))
            }
            "seeking"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Bool(false))
            }
            "networkState"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                let state = if self.resolve_media_src(*node).is_empty() {
                    0
                } else {
                    1
                };
                Ok(Value::Number(state))
            }
            "readyState"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::Number(0))
            }
            "currentTime"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_numeric_state_value(*node, INTERNAL_MEDIA_CURRENT_TIME_KEY, 0.0))
            }
            "volume"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_numeric_state_value(*node, INTERNAL_MEDIA_VOLUME_KEY, 1.0))
            }
            "duration"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_numeric_state_value(*node, INTERNAL_MEDIA_DURATION_KEY, f64::NAN))
            }
            "playbackRate"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_numeric_state_value(*node, INTERNAL_MEDIA_PLAYBACK_RATE_KEY, 1.0))
            }
            "defaultPlaybackRate"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_numeric_state_value(
                    *node,
                    INTERNAL_MEDIA_DEFAULT_PLAYBACK_RATE_KEY,
                    1.0,
                ))
            }
            "textTracks"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_text_tracks_live_list_value(*node))
            }
            "buffered"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_time_ranges_live_value(*node, "buffered"))
            }
            "seekable"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_time_ranges_live_value(*node, "seekable"))
            }
            "played"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(self.media_time_ranges_live_value(*node, "played"))
            }
            "currentSrc" | "currentsrc"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("img")
                        || tag.eq_ignore_ascii_case("audio")
                        || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                Ok(Value::String(self.resolve_media_src(*node)))
            }
            "complete"
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("img")) =>
            {
                Ok(Value::Bool(true))
            }
            "naturalWidth"
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("img")) =>
            {
                Ok(Value::Number(self.image_natural_dimension_value(*node)))
            }
            "naturalHeight"
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("img")) =>
            {
                Ok(Value::Number(self.image_natural_dimension_value(*node)))
            }
            "src" => Ok(Value::String(self.resolve_media_src(*node))),
            "poster" => Ok(Value::String(
                self.reflected_url_attribute_or_empty(*node, "poster"),
            )),
            "attributionSrc" | "attributionsrc" => Ok(Value::String(
                self.dom.attr(*node, "attributionsrc").unwrap_or_default(),
            )),
            "data" => {
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("object"))
                {
                    Ok(Value::String(
                        self.reflected_url_attribute_or_empty(*node, "data"),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "srcdoc" | "srcDoc" => {
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("iframe"))
                {
                    Ok(Value::String(
                        self.dom.attr(*node, "srcdoc").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "preload" => Ok(Value::String(
                self.dom.attr(*node, "preload").unwrap_or_default(),
            )),
            "sizes" => Ok(Value::String(
                self.dom.attr(*node, "sizes").unwrap_or_default(),
            )),
            "srcset" | "srcSet" => Ok(Value::String(
                self.dom.attr(*node, "srcset").unwrap_or_default(),
            )),
            "useMap" | "usemap" => {
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("img") || tag.eq_ignore_ascii_case("object")
                }) {
                    Ok(Value::String(
                        self.dom.attr(*node, "usemap").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "width" => Ok(Value::Number(self.canvas_dimension_value(*node, "width"))),
            "height" => Ok(Value::Number(self.canvas_dimension_value(*node, "height"))),
            "mozOpaque" | "mozopaque" => {
                if is_canvas {
                    Ok(Value::Bool(self.dom.attr(*node, "moz-opaque").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "mozPrintCallback" | "mozprintcallback" => {
                if is_canvas {
                    Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(*node, key.to_string()))
                        .cloned()
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "tagName" => Ok(Value::String(self.element_tag_name(*node))),
            "localName" => Ok(Value::String(
                self.dom
                    .tag_name(*node)
                    .map(|name| {
                        name.rsplit_once(':')
                            .map(|(_, local)| local)
                            .unwrap_or(name)
                            .to_ascii_lowercase()
                    })
                    .unwrap_or_default(),
            )),
            "namespaceURI" => Ok(self
                .dom
                .element(*node)
                .and_then(|element| element.namespace_uri.clone())
                .map(Value::String)
                .unwrap_or(Value::Null)),
            "prefix" => Ok(self
                .dom
                .tag_name(*node)
                .and_then(|name| name.split_once(':').map(|(prefix, _)| prefix))
                .map(|prefix| Value::String(prefix.to_string()))
                .unwrap_or(Value::Null)),
            "className" => Ok(Value::String(
                self.dom.attr(*node, "class").unwrap_or_default(),
            )),
            "classList" => Ok(self.class_list_live_value(*node)),
            "slot" => Ok(Value::String(
                self.dom.attr(*node, "slot").unwrap_or_default(),
            )),
            "role" => {
                let role = self.resolved_role_for_node(*node);
                if role.is_empty() {
                    Ok(Value::Null)
                } else if is_button {
                    Ok(Value::String("button".to_string()))
                } else {
                    Ok(Value::String(role))
                }
            }
            "baseURI" => Ok(Value::String(self.document_base_url())),
            "dataset" => Ok(self.dom_string_map_live_value(*node)),
            "open" => Ok(Value::Bool(self.dom.has_attr(*node, "open")?)),
            "closedBy" | "closedby" => Ok(Value::String(
                self.dom.attr(*node, "closedby").unwrap_or_default(),
            )),
            "htmlFor" => Ok(Value::String(
                self.dom.attr(*node, "for").unwrap_or_default(),
            )),
            "elementTiming" | "elementtiming" => Ok(Value::String(
                self.dom.attr(*node, "elementtiming").unwrap_or_default(),
            )),
            "options" => {
                if is_select {
                    return Ok(self.select_options_live_list_value(*node));
                }
                if is_datalist {
                    return Ok(self.datalist_options_live_list_value(*node));
                }
                Ok(Value::Undefined)
            }
            "selectedIndex" => {
                if !is_select {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(self.select_selected_index_value(*node)))
            }
            "selectedOptions" => {
                if !is_select {
                    return Ok(Value::Undefined);
                }
                Ok(self.selected_options_live_list_value(*node))
            }
            "size" => {
                if is_select {
                    return Ok(Value::Number(self.select_size_property_value(*node)));
                }
                if is_input {
                    return Ok(Value::Number(
                        self.input_size_property_value_for_node(*node),
                    ));
                }
                Ok(Value::Undefined)
            }
            "min" | "max" | "step" => {
                if !is_input {
                    return Ok(Value::Undefined);
                }
                Ok(Value::String(self.dom.attr(*node, key).unwrap_or_default()))
            }
            "maxLength" | "maxlength" => {
                if !(is_input || is_textarea) {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(
                    self.max_length_property_value_for_node(*node),
                ))
            }
            "minLength" | "minlength" => {
                if !(is_input || is_textarea) {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(
                    self.min_length_property_value_for_node(*node),
                ))
            }
            "rows" => {
                if !is_textarea {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(
                    self.textarea_rows_property_value_for_node(*node),
                ))
            }
            "cols" => {
                if !is_textarea {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(
                    self.textarea_cols_property_value_for_node(*node),
                ))
            }
            "validationMessage" => {
                let validity = self.compute_input_validity(*node)?;
                if validity.custom_error {
                    Ok(Value::String(self.dom.custom_validity_message(*node)?))
                } else {
                    Ok(Value::String(String::new()))
                }
            }
            "validity" => {
                let validity = self.compute_input_validity(*node)?;
                Ok(Self::input_validity_to_value(&validity))
            }
            "willValidate" => {
                let will_validate = if is_select {
                    self.select_will_validate(*node)
                } else if is_button {
                    self.button_will_validate(*node)
                } else if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("textarea"))
                {
                    !self.is_effectively_disabled(*node)
                } else if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    Self::input_participates_in_constraint_validation(
                        self.normalized_input_type(*node).as_str(),
                    ) && !self.is_effectively_disabled(*node)
                } else {
                    false
                };
                Ok(Value::Bool(will_validate))
            }
            "length" => {
                if is_form {
                    return Ok(Value::Number(self.form_elements(*node)?.len() as i64));
                }
                if !is_select {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(self.select_option_nodes(*node).len() as i64))
            }
            "captureStream"
            | "getContext"
            | "toDataURL"
            | "toBlob"
            | "transferControlToOffscreen" => {
                if !is_canvas {
                    return Ok(Value::Undefined);
                }
                Ok(self
                    .dom_runtime
                    .node_expando_props
                    .get(&(*node, key.to_string()))
                    .cloned()
                    .unwrap_or_else(Self::new_builtin_placeholder_function))
            }
            _ if key.starts_with("on") => {
                let is_body_window_alias = self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
                    && key
                        .strip_prefix("on")
                        .map(|event_type| event_type.to_ascii_lowercase())
                        .is_some_and(|event_type| {
                            Self::is_body_window_event_handler_alias(event_type.as_str())
                        });
                if is_body_window_alias {
                    Ok(
                        Self::object_get_entry(&self.dom_runtime.window_object.borrow(), key)
                            .unwrap_or(Value::Null),
                    )
                } else {
                    Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(*node, key.to_string()))
                        .cloned()
                        .unwrap_or(Value::Null))
                }
            }
            _ => Ok(self
                .dom_runtime
                .node_expando_props
                .get(&(*node, key.to_string()))
                .cloned()
                .or(if is_media {
                    self.html_media_builtin_property_value(*node, key)?
                } else {
                    None
                })
                .or(if is_form {
                    self.form_builtin_property_value(key)
                } else {
                    None
                })
                .or(if is_form {
                    self.form_named_property_value(*node, key)?
                } else {
                    None
                })
                .unwrap_or(Value::Undefined)),
        }
    }

    fn object_property_from_attr_or_class_list_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if Self::is_attr_object(entries) {
            let value = match key {
                "ownerElement" => {
                    Self::object_get_entry(entries, "ownerElement").unwrap_or(Value::Null)
                }
                "name" => Self::object_get_entry(entries, "name")
                    .unwrap_or_else(|| Value::String(String::new())),
                "value" => Self::object_get_entry(entries, "value")
                    .unwrap_or_else(|| Value::String(String::new())),
                "nodeType" => Value::Number(2),
                "nodeName" => Self::object_get_entry(entries, "name")
                    .unwrap_or_else(|| Value::String(String::new())),
                "nodeValue" => Self::object_get_entry(entries, "value")
                    .unwrap_or_else(|| Value::String(String::new())),
                "parentNode" | "parentElement" | "previousSibling" | "nextSibling" => Value::Null,
                _ => Value::Undefined,
            };
            if !matches!(value, Value::Undefined) {
                return Some(value);
            }
        }

        if Self::is_dom_string_map_object(entries) {
            let Some(node) = Self::dom_string_map_owner_node(entries) else {
                return None;
            };
            if self.dom.element(node).is_none() {
                return None;
            }
            if Self::is_symbol_storage_key(key) {
                return Some(Self::object_get_entry(entries, key).unwrap_or(Value::Undefined));
            }
            if self.is_to_string_tag_property_key(key) {
                return Some(Value::String("DOMStringMap".to_string()));
            }
            let attr_name = dataset_key_to_attr_name(key);
            return self.dom.attr(node, &attr_name).map(Value::String);
        }

        if Self::is_class_list_object(entries) {
            let Some(node) = (match Self::object_get_entry(entries, INTERNAL_CLASS_LIST_NODE_KEY) {
                Some(Value::Node(node)) => Some(node),
                _ => None,
            }) else {
                return None;
            };
            let classes = class_tokens(self.dom.attr(node, "class").as_deref());
            let has_explicit_prototype =
                Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
            let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
            if key == "length" {
                return Some(Value::Number(classes.len() as i64));
            }
            if key == "value" {
                return Some(Value::String(classes.join(" ")));
            }
            if key_is_to_string_tag {
                return (!has_explicit_prototype)
                    .then_some(Value::String("DOMTokenList".to_string()));
            }
            if !has_explicit_prototype {
                if let Some(kind) = match key {
                    "add" => Some("class_list_add"),
                    "remove" => Some("class_list_remove"),
                    "toggle" => Some("class_list_toggle"),
                    "contains" => Some("class_list_contains"),
                    "replace" => Some("class_list_replace"),
                    "item" => Some("class_list_item"),
                    "forEach" => Some("class_list_for_each"),
                    "keys" => Some("class_list_keys"),
                    "values" => Some("class_list_values"),
                    "entries" => Some("class_list_entries"),
                    "toString" => Some("class_list_to_string"),
                    _ if self.is_iterator_property_key(key) => Some("class_list_values"),
                    _ => None,
                } {
                    return Some(Self::new_class_list_method_callable(kind));
                }
            }
            if let Ok(index) = key.parse::<usize>() {
                return classes.get(index).cloned().map(Value::String);
            }
            if let Some(value) = Self::object_get_entry(entries, key) {
                return Some(value);
            }
            return None;
        }

        None
    }

    fn object_property_from_web_api_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Result<Option<Value>> {
        if Self::is_event_target_object(entries)
            && let Some(value) = Self::placeholder_backed_object_builtin_property_value(entries, key)
        {
            return Ok(Some(value));
        }
        if let Some(value) = Self::object_property_from_event_entries(entries, key) {
            return Ok(Some(value));
        }
        if let Some(value) = Self::object_property_from_data_transfer_entries(entries, key) {
            return Ok(Some(value));
        }
        if let Some(value) = Self::object_property_from_range_or_selection_entries(entries, key) {
            return Ok(Some(value));
        }
        if let Some(value) = Self::object_property_from_cookie_store_or_cache_entries(entries, key)
        {
            return Ok(Some(value));
        }
        if let Some(value) = self.computed_style_object_property_from_entries(entries, key)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.fetch_response_property_from_entries(entries, key) {
            return Ok(Some(value));
        }
        if let Some(value) = self.fetch_request_property_from_entries(entries, key) {
            return Ok(Some(value));
        }
        if let Some(value) = self.headers_property_from_entries(entries, key) {
            return Ok(Some(value));
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_DOM_PARSER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            if let Some(value) = self.dom_parser_object_property(entries, key) {
                return Ok(Some(value));
            }
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_XML_SERIALIZER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            if let Some(value) = self.xml_serializer_object_property(entries, key) {
                return Ok(Some(value));
            }
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_PARSED_DOCUMENT_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            if let Some(value) = self.parsed_document_property_from_entries(entries, key)? {
                return Ok(Some(value));
            }
        }
        if matches!(
            Self::object_get_entry(entries, INTERNAL_TREE_WALKER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            if let Some(value) = self.tree_walker_property_from_entries(entries, key)? {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    fn object_property_from_event_entries(entries: &ObjectValue, key: &str) -> Option<Value> {
        if Self::is_event_object(entries)
            || Self::is_keyboard_event_object(entries)
            || Self::is_pointer_event_object(entries)
            || Self::is_navigate_event_object(entries)
        {
            return Self::placeholder_backed_object_builtin_property_value(entries, key);
        }
        None
    }

    fn object_property_from_data_transfer_entries(
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if Self::is_data_transfer_object(entries)
            || Self::is_clipboard_data_object(entries)
            || Self::is_data_transfer_item_object(entries)
        {
            return Self::placeholder_backed_object_builtin_property_value(entries, key);
        }
        None
    }

    fn object_property_from_match_media_entries(
        &mut self,
        receiver: &Value,
        entries: &ObjectValue,
        key: &str,
        key_is_to_string_tag: bool,
    ) -> Result<Option<Value>> {
        if !Self::is_match_media_object(entries) {
            return Ok(None);
        }
        if matches!(key, "matches" | "media")
            && let Some(value) =
                self.object_property_from_entries_with_getter(receiver, entries, key)?
        {
            return Ok(Some(value));
        }
        let query = Self::object_get_entry(entries, INTERNAL_MATCH_MEDIA_QUERY_KEY)
            .map(|value| value.as_string())
            .unwrap_or_default();
        if key == "matches" {
            let matches = self
                .platform_mocks
                .match_media_mocks
                .get(&query)
                .copied()
                .unwrap_or(self.platform_mocks.default_match_media_matches);
            return Ok(Some(Value::Bool(matches)));
        }
        if key == "media" {
            return Ok(Some(Value::String(query)));
        }
        if let Some(value) = Self::placeholder_backed_object_builtin_property_value(entries, key) {
            return Ok(Some(value));
        }
        if key_is_to_string_tag {
            return Ok(Some(Value::String("MediaQueryList".to_string())));
        }
        Ok(None)
    }

    fn object_property_from_named_node_map_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_named_node_map_object(entries) {
            return None;
        }
        let has_explicit_prototype =
            Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
        if self.is_to_string_tag_property_key(key) {
            return (!has_explicit_prototype).then_some(Value::String("NamedNodeMap".to_string()));
        }
        if !has_explicit_prototype {
            if let Some(kind) = match key {
                "item" => Some("named_node_map_item"),
                "getNamedItem" => Some("named_node_map_get_named_item"),
                "setNamedItem" => Some("named_node_map_set_named_item"),
                "removeNamedItem" => Some("named_node_map_remove_named_item"),
                "getNamedItemNS" => Some("named_node_map_get_named_item_ns"),
                "setNamedItemNS" => Some("named_node_map_set_named_item_ns"),
                "removeNamedItemNS" => Some("named_node_map_remove_named_item_ns"),
                "forEach" => Some("named_node_map_for_each"),
                "keys" => Some("named_node_map_keys"),
                "values" => Some("named_node_map_values"),
                "entries" => Some("named_node_map_entries"),
                _ if self.is_iterator_property_key(key) => Some("named_node_map_values"),
                _ => None,
            } {
                return Some(Self::new_named_node_map_method_callable(kind));
            }
        }
        let owner = Self::named_node_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some());
        let attrs = owner
            .map(|owner_node| self.named_node_map_entries(owner_node))
            .unwrap_or_default();
        if key == "length" {
            return Some(Value::Number(attrs.len() as i64));
        }
        if let Ok(index) = key.parse::<usize>() {
            return attrs.get(index).and_then(|(name, value)| {
                owner.map(|owner_node| Self::new_attr_object_value(name, value, Some(owner_node)))
            });
        }
        if !self.named_node_map_named_property_is_visible(entries, key) {
            return None;
        }
        if let Some(owner_node) = owner {
            if let Some((name, value)) = attrs.iter().find(|(name, _)| name == key) {
                return Some(Self::new_attr_object_value(name, value, Some(owner_node)));
            }
        }
        None
    }

    fn object_property_from_string_wrapper_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        let text = Self::string_wrapper_value_from_object(entries)?;
        if key == "length" {
            return Some(Value::Number(text.chars().count() as i64));
        }
        let is_url_like = Self::is_url_object(entries) || Self::is_location_object(entries);
        if key == "constructor" && !is_url_like {
            return Some(Value::StringConstructor);
        }
        if !is_url_like {
            if self.is_iterator_property_key(key) {
                return Some(Self::new_receiver_builtin_callable("string", "iterator"));
            }
            if matches!(key, "toString" | "valueOf") || Self::is_string_method_name(key) {
                return Some(Self::new_receiver_builtin_callable("string", key));
            }
        }
        if let Ok(index) = key.parse::<usize>() {
            return text
                .chars()
                .nth(index)
                .map(|ch| Value::String(ch.to_string()));
        }
        None
    }

    fn object_property_from_match_media_named_node_map_or_string_wrapper_entries(
        &mut self,
        receiver: &Value,
        entries: &ObjectValue,
        key: &str,
    ) -> Result<Option<Value>> {
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        if let Some(value) = self.object_property_from_match_media_entries(
            receiver,
            entries,
            key,
            key_is_to_string_tag,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) = self.object_property_from_named_node_map_entries(entries, key) {
            return Ok(Some(value));
        }
        Ok(self.object_property_from_string_wrapper_entries(entries, key))
    }

    fn generator_constructor_prototype_value(&mut self, is_async: bool) -> Option<Value> {
        let constructor = if is_async {
            self.new_async_generator_function_constructor_value()
        } else {
            self.new_generator_function_constructor_value()
        };
        let Value::Object(constructor_entries) = constructor else {
            return None;
        };
        let constructor_entries = constructor_entries.borrow();
        Self::object_get_entry(&constructor_entries, "prototype")
    }

    fn object_property_from_generator_constructor_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if key != "constructor" {
            return None;
        }
        if Self::is_generator_object(entries) {
            return self.generator_constructor_prototype_value(false);
        }
        if Self::is_async_generator_object(entries) {
            return self.generator_constructor_prototype_value(true);
        }
        None
    }

    fn looks_like_iterator_prototype_entries(entries: &ObjectValue, is_async: bool) -> bool {
        let constructor_matches = matches!(
            Self::object_get_entry(entries, "constructor"),
            Some(Value::Object(constructor)) if {
                let constructor = constructor.borrow();
                if is_async {
                    Self::is_async_generator_function_prototype_object(&constructor)
                } else {
                    Self::is_generator_function_prototype_object(&constructor)
                }
            }
        );
        constructor_matches
            && Self::object_get_entry(entries, "next").is_some()
            && Self::object_get_entry(entries, "return").is_some()
            && Self::object_get_entry(entries, "throw").is_some()
    }

    fn object_property_from_generator_to_string_tag_entries(
        &self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !self.is_to_string_tag_property_key(key) {
            return None;
        }
        if Self::is_generator_function_prototype_object(entries) {
            return Some(Value::String("GeneratorFunction".to_string()));
        }
        if Self::is_generator_object(entries)
            || Self::is_generator_prototype_object(entries)
            || Self::looks_like_iterator_prototype_entries(entries, false)
        {
            return Some(Value::String("Generator".to_string()));
        }
        if Self::is_async_generator_function_prototype_object(entries) {
            return Some(Value::String("AsyncGeneratorFunction".to_string()));
        }
        if Self::is_async_generator_object(entries)
            || Self::is_async_generator_prototype_object(entries)
            || Self::looks_like_iterator_prototype_entries(entries, true)
        {
            return Some(Value::String("AsyncGenerator".to_string()));
        }
        None
    }

    fn object_property_from_callable_and_generator_entries(
        &mut self,
        value: &Value,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if Self::callable_kind_from_value(value).is_some() {
            if Self::is_callable_own_surface_key(key)
                && Self::is_builtin_object_property_deleted(entries, key)
            {
                return Self::deleted_callable_surface_fallback_value(key);
            }
            if let Some(surface_value) = self.callable_function_surface_value(value, key) {
                return Some(Self::object_get_entry(entries, key).unwrap_or(surface_value));
            }
        }
        if let Some(value) = self.object_property_from_generator_constructor_entries(entries, key) {
            return Some(value);
        }
        self.object_property_from_generator_to_string_tag_entries(entries, key)
    }

    fn object_property_from_url_search_params_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_url_search_params_object(entries) {
            return None;
        }
        if key == "size" {
            let size = Self::url_search_params_pairs_from_object_entries(entries).len();
            return Some(Value::Number(size as i64));
        }
        if self.is_iterator_property_key(key) {
            return Some(Self::new_receiver_builtin_callable(
                "url_search_params",
                "entries",
            ));
        }
        if matches!(
            key,
            "append"
                | "delete"
                | "get"
                | "getAll"
                | "has"
                | "set"
                | "sort"
                | "forEach"
                | "entries"
                | "keys"
                | "values"
                | "toString"
        ) {
            return Some(Self::new_receiver_builtin_callable(
                "url_search_params",
                key,
            ));
        }
        None
    }

    fn object_property_from_storage_entries(entries: &ObjectValue, key: &str) -> Option<Value> {
        if !Self::is_storage_object(entries) {
            return None;
        }
        if key == "length" {
            let len = Self::storage_pairs_from_object_entries(entries).len();
            return Some(Value::Number(len as i64));
        }
        if let Some(value) = Self::object_get_entry(entries, key) {
            return Some(value);
        }
        if Self::is_storage_method_name(key) {
            return Some(Self::new_receiver_builtin_callable("storage", key));
        }
        if let Some((_, value)) = Self::storage_pairs_from_object_entries(entries)
            .into_iter()
            .find(|(name, _)| name == key)
        {
            return Some(Value::String(value));
        }
        None
    }

    fn object_property_from_document_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        let is_document_object = matches!(
            Self::object_get_entry(entries, INTERNAL_DOCUMENT_OBJECT_KEY),
            Some(Value::Bool(true))
        );
        if !is_document_object {
            return None;
        }
        if let Some(value) = Self::placeholder_backed_object_builtin_property_value(entries, key) {
            return Some(value);
        }
        let value = match key {
            "nodeType" => Value::Number(self.node_type_number(self.dom.root)),
            "textContent" => self.node_text_content_value(self.dom.root),
            "body" => self.dom.body().map(Value::Node).unwrap_or(Value::Null),
            "head" => self.dom.head().map(Value::Node).unwrap_or(Value::Null),
            "documentElement" => self
                .dom
                .document_element()
                .map(Value::Node)
                .unwrap_or(Value::Null),
            "forms" => self.document_forms_live_list_value(),
            "images" => self.document_images_live_list_value(),
            "links" => self.document_links_live_list_value(),
            "scripts" => self.document_scripts_live_list_value(),
            "readyState" => Value::String(self.dom_runtime.document_ready_state.clone()),
            "cookie" => Value::String(self.document_cookie_string()),
            "hidden" => Value::Bool(self.dom_runtime.document_visibility_state == "hidden"),
            "visibilityState" => Value::String(self.dom_runtime.document_visibility_state.clone()),
            "adoptedStyleSheets" => self.ensure_document_adopted_style_sheets_property(),
            _ if key.starts_with("on") => self
                .dom_runtime
                .node_expando_props
                .get(&(self.dom.root, key.to_string()))
                .cloned()
                .unwrap_or(Value::Null),
            _ => Value::Undefined,
        };
        if matches!(value, Value::Undefined) {
            None
        } else {
            Some(value)
        }
    }

    fn object_property_from_range_or_selection_entries(
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        Self::placeholder_backed_object_builtin_property_value(entries, key)
    }

    fn object_property_from_cookie_store_or_cache_entries(
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        Self::placeholder_backed_object_builtin_property_value(entries, key)
    }

    fn object_property_from_url_entries(entries: &ObjectValue, key: &str) -> Option<Value> {
        if !Self::is_url_object(entries) {
            return None;
        }
        if key == "constructor" {
            return Some(Value::UrlConstructor);
        }
        if matches!(key, "toString" | "toJSON") {
            return Some(Self::new_receiver_builtin_callable("url", key));
        }
        None
    }

    fn object_property_from_storage_document_and_url_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if let Some(value) = self.object_property_from_url_search_params_entries(entries, key) {
            return Some(value);
        }
        if let Some(value) = Self::object_property_from_storage_entries(entries, key) {
            return Some(value);
        }
        if let Some(value) = self.object_property_from_document_entries(entries, key) {
            return Some(value);
        }
        Self::object_property_from_url_entries(entries, key)
    }

    fn object_property_from_entries_via_prototype_chain(
        &mut self,
        owner: &Value,
        receiver: &Value,
        entries: &ObjectValue,
        key: &str,
    ) -> Result<Value> {
        if let Some(value) =
            self.object_property_from_entries_with_getter(receiver, entries, key)?
        {
            return Ok(value);
        }
        let mut prototype = Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
            .or_else(|| self.value_internal_prototype_value(owner));
        while let Some(current) = prototype {
            if matches!(current, Value::Null | Value::Undefined) {
                break;
            }
            let value = self.object_property_from_value_with_receiver(&current, key, receiver)?;
            if !matches!(value, Value::Undefined) {
                return Ok(value);
            }
            prototype = self.value_internal_prototype_value(&current);
        }
        Ok(Value::Undefined)
    }

    pub(crate) fn value_internal_prototype_value(&mut self, value: &Value) -> Option<Value> {
        if let Some(value) = self.variant_callable_internal_prototype_value(value) {
            return Some(value);
        }
        match value {
            Value::Object(entries) => {
                let entries_ref = entries.borrow();
                if let Some(value) =
                    Self::object_get_entry(&entries_ref, INTERNAL_OBJECT_PROTOTYPE_KEY)
                {
                    return Some(value);
                }
                if Self::is_url_object(&entries_ref) {
                    return Some(self.cached_url_constructor_prototype_value());
                }
                if Self::is_url_search_params_object(&entries_ref) {
                    return Some(self.cached_url_search_params_constructor_prototype_value());
                }
                if Self::string_wrapper_value_from_object(&entries_ref).is_some() {
                    return Some(self.cached_string_constructor_prototype_value());
                }
                if Self::boolean_wrapper_value_from_object(&entries_ref).is_some() {
                    return self.constructor_prototype_from_env("Boolean");
                }
                if Self::number_wrapper_value_from_object(&entries_ref).is_some() {
                    return self.constructor_prototype_from_env("Number");
                }
                if Self::bigint_wrapper_value_from_object(&entries_ref).is_some() {
                    return self.constructor_prototype_from_env("BigInt");
                }
                if Self::symbol_wrapper_id_from_object(&entries_ref).is_some() {
                    return Some(self.cached_symbol_constructor_prototype_value());
                }
                if Self::callable_kind_from_value(value).is_some() {
                    return Some(self.cached_function_constructor_prototype_value());
                }
                Some(self.object_constructor_prototype_value())
            }
            Value::Array(values) => {
                Self::object_get_entry(&values.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
            }
            Value::Map(map) => Some(
                Self::object_get_entry(&map.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| self.cached_map_constructor_prototype_value()),
            ),
            Value::WeakMap(map) => Some(
                Self::object_get_entry(&map.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| self.cached_weak_map_constructor_prototype_value()),
            ),
            Value::Set(set) => Some(
                Self::object_get_entry(&set.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| self.cached_set_constructor_prototype_value()),
            ),
            Value::WeakSet(set) => Some(
                Self::object_get_entry(&set.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| self.cached_weak_set_constructor_prototype_value()),
            ),
            Value::RegExp(regex) => Some(
                Self::object_get_entry(&regex.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| self.cached_regexp_constructor_prototype_value()),
            ),
            Value::Date(_) => Some(self.cached_date_prototype_value()),
            Value::Promise(_) => Some(self.cached_promise_constructor_prototype_value()),
            Value::TypedArray(values) => {
                let (explicit, kind) = {
                    let values_ref = values.borrow();
                    (
                        Self::object_get_entry(
                            &values_ref.properties,
                            INTERNAL_OBJECT_PROTOTYPE_KEY,
                        ),
                        values_ref.kind,
                    )
                };
                Some(explicit.unwrap_or_else(|| {
                    self.cached_typed_array_constructor_prototype_value(
                        TypedArrayConstructorKind::Concrete(kind),
                    )
                }))
            }
            Value::Blob(_) => Some(self.cached_blob_constructor_prototype_value()),
            Value::ArrayBuffer(_) => Some(self.cached_array_buffer_constructor_prototype_value()),
            Value::NodeList(nodes) => Some(
                Self::object_get_entry(&nodes.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                    .unwrap_or_else(|| match nodes.borrow().kind {
                        NodeListKind::NodeList => {
                            self.cached_node_list_constructor_prototype_value()
                        }
                        NodeListKind::TextTrackList => {
                            self.cached_text_track_list_constructor_prototype_value()
                        }
                        NodeListKind::RadioNodeList => {
                            self.cached_radio_node_list_constructor_prototype_value()
                        }
                        NodeListKind::HtmlCollection => {
                            self.cached_html_collection_constructor_prototype_value()
                        }
                        NodeListKind::HtmlFormControlsCollection => {
                            self.cached_html_form_controls_collection_constructor_prototype_value()
                        }
                        NodeListKind::HtmlOptionsCollection => {
                            self.cached_html_options_collection_constructor_prototype_value()
                        }
                    }),
            ),
            Value::String(_) => Some(self.cached_string_constructor_prototype_value()),
            Value::Bool(_) => self.constructor_prototype_from_env("Boolean"),
            Value::Number(_) | Value::Float(_) => self.constructor_prototype_from_env("Number"),
            Value::BigInt(_) => self.constructor_prototype_from_env("BigInt"),
            Value::Symbol(_) => Some(self.cached_symbol_constructor_prototype_value()),
            Value::UrlConstructor => {
                let explicit = {
                    let entries = self.browser_apis.url_constructor_properties.borrow();
                    Self::object_get_entry(&entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
                };
                Some(explicit.unwrap_or_else(|| self.cached_function_constructor_prototype_value()))
            }
            Value::Function(function) => {
                if let Some(entries) = self
                    .script_runtime
                    .function_public_properties
                    .get(&function.function_id)
                    && let Some(value) =
                        Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
                {
                    return Some(value);
                }
                if function.is_generator {
                    Some(
                        self.generator_constructor_prototype_value(function.is_async)
                            .unwrap_or_else(|| self.cached_function_constructor_prototype_value()),
                    )
                } else {
                    Some(self.cached_function_constructor_prototype_value())
                }
            }
            _ if self.is_callable_value(value) => {
                Some(self.cached_function_constructor_prototype_value())
            }
            _ => None,
        }
    }

    pub(crate) fn function_public_property_from_entries_with_receiver(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
        receiver: &Value,
    ) -> Result<Option<Value>> {
        let Some(entries) = self
            .script_runtime
            .function_public_properties
            .get(&function.function_id)
            .cloned()
        else {
            return Ok(None);
        };
        self.object_property_from_entries_with_getter(receiver, &entries, key)
    }

    fn inherited_property_from_function_super_constructor(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
        receiver: Option<&Value>,
    ) -> Result<Option<Value>> {
        let Some(super_constructor) = function.class_super_constructor.clone() else {
            return Ok(None);
        };
        if matches!(super_constructor, Value::Null) {
            return Ok(None);
        }
        let inherited = if let Some(receiver) = receiver {
            self.object_property_from_value_with_receiver(&super_constructor, key, receiver)?
        } else {
            self.object_property_from_value(&super_constructor, key)?
        };
        if matches!(inherited, Value::Undefined) {
            Ok(None)
        } else {
            Ok(Some(inherited))
        }
    }

    fn inherited_property_from_value_prototype_chain_with_receiver(
        &mut self,
        owner: &Value,
        receiver: &Value,
        key: &str,
    ) -> Result<Option<Value>> {
        let mut prototype = self.value_internal_prototype_value(owner);
        while let Some(current) = prototype {
            if matches!(current, Value::Null | Value::Undefined) {
                break;
            }
            let value = self.object_property_from_value_with_receiver(&current, key, receiver)?;
            if !matches!(value, Value::Undefined) {
                return Ok(Some(value));
            }
            prototype = self.value_internal_prototype_value(&current);
        }
        Ok(None)
    }

    fn inherited_property_from_value_prototype_chain(
        &mut self,
        receiver: &Value,
        key: &str,
    ) -> Result<Option<Value>> {
        self.inherited_property_from_value_prototype_chain_with_receiver(receiver, receiver, key)
    }

    fn callable_value_property_or_inherited(
        &mut self,
        receiver: &Value,
        key: &str,
        own_value: Value,
    ) -> Result<Value> {
        if !matches!(own_value, Value::Undefined) {
            return Ok(own_value);
        }
        Ok(self
            .inherited_property_from_value_prototype_chain(receiver, key)?
            .unwrap_or(Value::Undefined))
    }

    fn object_property_from_object_value(
        &mut self,
        value: &Value,
        entries: &Rc<RefCell<ObjectValue>>,
        key: &str,
    ) -> Result<Value> {
        let entries = entries.borrow();
        if (Self::is_dom_string_map_object(&entries)
            || Self::is_class_list_object(&entries)
            || Self::is_named_node_map_object(&entries))
            && let Some(value) =
                self.object_property_from_entries_with_getter(value, &entries, key)?
        {
            return Ok(value);
        }
        if let Some(value) = self.object_property_from_attr_or_class_list_entries(&entries, key) {
            return Ok(value);
        }
        if let Some(value) = self.object_property_from_web_api_entries(&entries, key)? {
            return Ok(value);
        }
        if let Some(value) = self
            .object_property_from_match_media_named_node_map_or_string_wrapper_entries(
                value, &entries, key,
            )?
        {
            return Ok(value);
        }
        if let Some(value) =
            self.object_property_from_callable_and_generator_entries(value, &entries, key)
        {
            return Ok(value);
        }
        if let Some(value) =
            self.object_property_from_storage_document_and_url_entries(&entries, key)
        {
            return Ok(value);
        }
        self.object_property_from_entries_via_prototype_chain(value, value, &entries, key)
    }

    fn object_property_from_function_value(
        &mut self,
        value: &Value,
        function: &Rc<FunctionValue>,
        key: &str,
    ) -> Result<Value> {
        if let Some(custom_value) =
            self.function_public_property_from_entries_with_receiver(function, key, value)?
        {
            return Ok(custom_value);
        }
        let own_value = if self
            .script_runtime
            .function_public_properties
            .get(&function.function_id)
            .is_some_and(|entries| Self::is_builtin_object_property_deleted(entries, key))
        {
            Self::deleted_callable_surface_fallback_value(key).unwrap_or(Value::Undefined)
        } else {
            self.function_own_property_value(function, key, true)
        };
        if !matches!(own_value, Value::Undefined) {
            return Ok(own_value);
        }
        if let Some(inherited) =
            self.inherited_property_from_function_super_constructor(function, key, None)?
        {
            return Ok(inherited);
        }
        Ok(self
            .inherited_property_from_value_prototype_chain(value, key)?
            .unwrap_or(Value::Undefined))
    }

    fn object_property_from_object_value_with_receiver(
        &mut self,
        entries: &Rc<RefCell<ObjectValue>>,
        key: &str,
        receiver: &Value,
    ) -> Result<Value> {
        let owner = Value::Object(entries.clone());
        let entries = entries.borrow();
        if (Self::is_dom_string_map_object(&entries)
            || Self::is_class_list_object(&entries)
            || Self::is_named_node_map_object(&entries))
            && let Some(value) =
                self.object_property_from_entries_with_getter(receiver, &entries, key)?
        {
            return Ok(value);
        }
        if let Some(value) = self.object_property_from_attr_or_class_list_entries(&entries, key) {
            return Ok(value);
        }
        if let Some(value) = self.object_property_from_web_api_entries(&entries, key)?
            && self.is_callable_value(&value)
            && !Self::is_builtin_placeholder_value(&value)
        {
            return Ok(value);
        }
        if let Some(value) = self
            .object_property_from_match_media_named_node_map_or_string_wrapper_entries(
                receiver, &entries, key,
            )?
        {
            return Ok(value);
        }
        if let Some(value) =
            self.object_property_from_storage_document_and_url_entries(&entries, key)
            && self.is_callable_value(&value)
            && !Self::is_builtin_placeholder_value(&value)
        {
            return Ok(value);
        }
        self.object_property_from_entries_via_prototype_chain(&owner, receiver, &entries, key)
    }

    fn object_property_from_function_value_with_receiver(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
        receiver: &Value,
    ) -> Result<Value> {
        if let Some(custom_value) =
            self.function_public_property_from_entries_with_receiver(function, key, receiver)?
        {
            return Ok(custom_value);
        }
        let own_value = if self
            .script_runtime
            .function_public_properties
            .get(&function.function_id)
            .is_some_and(|entries| Self::is_builtin_object_property_deleted(entries, key))
        {
            Self::deleted_callable_surface_fallback_value(key).unwrap_or(Value::Undefined)
        } else {
            self.function_own_property_value(function, key, false)
        };
        if !matches!(own_value, Value::Undefined) {
            return Ok(own_value);
        }
        if let Some(inherited) =
            self.inherited_property_from_function_super_constructor(function, key, Some(receiver))?
        {
            return Ok(inherited);
        }
        let owner = Value::Function(function.clone());
        Ok(self
            .inherited_property_from_value_prototype_chain_with_receiver(&owner, receiver, key)?
            .unwrap_or(Value::Undefined))
    }

    pub(crate) fn object_property_from_value(&mut self, value: &Value, key: &str) -> Result<Value> {
        match value {
            Value::Node(node) => self.object_property_from_node_value(node, key),
            Value::String(text) => Ok(self.object_property_from_string_value(text, key)),
            Value::Bool(_) => Ok(self.object_property_from_bool_value(key)),
            Value::Number(_) | Value::Float(_) => Ok(self.object_property_from_number_value(key)),
            Value::BigInt(_) => Ok(self.object_property_from_bigint_value(key)),
            Value::Array(values) => {
                self.object_property_from_array_value(value, value, values, key)
            }
            Value::NodeList(nodes) => {
                self.object_property_from_node_list_value(value, value, nodes, key)
            }
            Value::TypedArray(values) => {
                self.object_property_from_typed_array_value(value, value, values, key)
            }
            Value::Object(entries) => self.object_property_from_object_value(value, entries, key),
            Value::Promise(promise) => {
                self.object_property_from_promise_value(value, value, promise, key)
            }
            Value::Map(map) => self.object_property_from_map_value(value, value, map, key),
            Value::WeakMap(weak_map) => {
                self.object_property_from_weak_map_value(value, value, weak_map, key)
            }
            Value::WeakSet(weak_set) => {
                self.object_property_from_weak_set_value(value, value, weak_set, key)
            }
            Value::Set(set) => self.object_property_from_set_value(value, value, set, key),
            Value::FormData(entries) => Ok(self.object_property_from_form_data_value(entries, key)),
            Value::Blob(blob) => self.object_property_from_blob_value(value, value, blob, key),
            Value::ArrayBuffer(buffer) => {
                self.object_property_from_array_buffer_value(value, value, buffer, key)
            }
            Value::Symbol(symbol) => Ok(Self::object_property_from_symbol_value(symbol, key)),
            Value::RegExp(regex) => {
                self.object_property_from_regexp_value(value, value, regex, key)
            }
            Value::Date(_) => Ok(self
                .inherited_property_from_value_prototype_chain(value, key)?
                .unwrap_or(Value::Undefined)),
            Value::Function(function) => {
                self.object_property_from_function_value(value, function, key)
            }
            Value::MapConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_map_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::WeakMapConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_weak_map_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::SetConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_set_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::WeakSetConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_weak_set_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::TypedArrayConstructor(kind) => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => {
                        self.cached_typed_array_constructor_prototype_value(kind.clone())
                    }
                    "from" if matches!(kind, TypedArrayConstructorKind::Concrete(_)) => {
                        self.cached_typed_array_static_method_value(kind.clone(), "from")
                    }
                    "of" if matches!(kind, TypedArrayConstructorKind::Concrete(_)) => {
                        self.cached_typed_array_static_method_value(kind.clone(), "of")
                    }
                    "BYTES_PER_ELEMENT" => match kind {
                        TypedArrayConstructorKind::Concrete(kind) => {
                            Value::Number(kind.bytes_per_element() as i64)
                        }
                        TypedArrayConstructorKind::Abstract => Value::Undefined,
                    },
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::BlobConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_blob_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::RegExpConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_regexp_constructor_prototype_value(),
                    "escape" => self.cached_regexp_static_method_value("escape"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::UrlConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = if key == "prototype" {
                    self.cached_url_constructor_prototype_value()
                } else if let Some(value) = Self::object_get_entry(
                    &self.browser_apis.url_constructor_properties.borrow(),
                    key,
                ) {
                    value
                } else if Self::is_url_static_method_name(key) {
                    Self::new_builtin_placeholder_function()
                } else {
                    Value::Undefined
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::ArrayBufferConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_array_buffer_constructor_prototype_value(),
                    "isView" => self.cached_array_buffer_static_method_value("isView"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::PromiseConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_promise_constructor_prototype_value(),
                    "resolve" => self.cached_promise_static_method_value("resolve"),
                    "reject" => self.cached_promise_static_method_value("reject"),
                    "all" => self.cached_promise_static_method_value("all"),
                    "allSettled" => self.cached_promise_static_method_value("allSettled"),
                    "any" => self.cached_promise_static_method_value("any"),
                    "race" => self.cached_promise_static_method_value("race"),
                    "try" => self.cached_promise_static_method_value("try"),
                    "withResolvers" => self.cached_promise_static_method_value("withResolvers"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::UrlSearchParamsConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_url_search_params_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::StringConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_string_constructor_prototype_value(),
                    "fromCharCode" => self.cached_string_static_method_value("fromCharCode"),
                    "fromCodePoint" => self.cached_string_static_method_value("fromCodePoint"),
                    "raw" => self.cached_string_static_method_value("raw"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::SymbolConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Ok(value);
                }
                let own_value = match key {
                    "prototype" => self.cached_symbol_constructor_prototype_value(),
                    "for" => self.cached_symbol_static_method_value("for"),
                    "keyFor" => self.cached_symbol_static_method_value("keyFor"),
                    "asyncDispose" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::AsyncDispose)
                    }
                    "asyncIterator" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::AsyncIterator)
                    }
                    "dispose" => self.eval_symbol_static_property(SymbolStaticProperty::Dispose),
                    "hasInstance" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::HasInstance)
                    }
                    "isConcatSpreadable" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::IsConcatSpreadable)
                    }
                    "iterator" => self.eval_symbol_static_property(SymbolStaticProperty::Iterator),
                    "match" => self.eval_symbol_static_property(SymbolStaticProperty::Match),
                    "matchAll" => self.eval_symbol_static_property(SymbolStaticProperty::MatchAll),
                    "replace" => self.eval_symbol_static_property(SymbolStaticProperty::Replace),
                    "search" => self.eval_symbol_static_property(SymbolStaticProperty::Search),
                    "species" => self.eval_symbol_static_property(SymbolStaticProperty::Species),
                    "split" => self.eval_symbol_static_property(SymbolStaticProperty::Split),
                    "toPrimitive" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::ToPrimitive)
                    }
                    "toStringTag" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag)
                    }
                    "unscopables" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::Unscopables)
                    }
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            _ => Err(Error::ScriptRuntime("value is not an object".into())),
        }
    }

    pub(crate) fn object_synthesized_own_property_exists(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> bool {
        if Self::is_class_list_object(entries) {
            let has_explicit_prototype =
                Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
            if key == "length" || key == "value" {
                return true;
            }
            if Self::is_class_list_method_name(key) || self.is_iterator_property_key(key) {
                return !has_explicit_prototype;
            }
        }
        if Self::is_named_node_map_object(entries) {
            let has_explicit_prototype =
                Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY).is_some();
            if key == "length" {
                return true;
            }
            if Self::is_named_node_map_method_name(key) || self.is_iterator_property_key(key) {
                return !has_explicit_prototype;
            }
        }
        self.object_property_from_attr_or_class_list_entries(entries, key)
            .is_some()
            || self
                .object_property_from_named_node_map_entries(entries, key)
                .is_some()
    }

    pub(crate) fn object_property_from_value_with_receiver(
        &mut self,
        value: &Value,
        key: &str,
        receiver: &Value,
    ) -> Result<Value> {
        match value {
            Value::Object(entries) => {
                self.object_property_from_object_value_with_receiver(entries, key, receiver)
            }
            Value::Function(function) => {
                self.object_property_from_function_value_with_receiver(function, key, receiver)
            }
            Value::Array(values) => {
                self.object_property_from_array_value(value, receiver, values, key)
            }
            Value::NodeList(nodes) => {
                self.object_property_from_node_list_value(value, receiver, nodes, key)
            }
            Value::TypedArray(values) => {
                self.object_property_from_typed_array_value(value, receiver, values, key)
            }
            Value::Promise(promise) => {
                self.object_property_from_promise_value(value, receiver, promise, key)
            }
            Value::Map(map) => self.object_property_from_map_value(value, receiver, map, key),
            Value::WeakMap(weak_map) => {
                self.object_property_from_weak_map_value(value, receiver, weak_map, key)
            }
            Value::WeakSet(weak_set) => {
                self.object_property_from_weak_set_value(value, receiver, weak_set, key)
            }
            Value::Set(set) => self.object_property_from_set_value(value, receiver, set, key),
            Value::Blob(blob) => self.object_property_from_blob_value(value, receiver, blob, key),
            Value::ArrayBuffer(buffer) => {
                self.object_property_from_array_buffer_value(value, receiver, buffer, key)
            }
            Value::RegExp(regex) => {
                self.object_property_from_regexp_value(value, receiver, regex, key)
            }
            _ => self.object_property_from_value(value, key),
        }
    }

    pub(crate) fn object_property_from_named_value(
        &mut self,
        variable_name: &str,
        value: &Value,
        key: &str,
    ) -> Result<Value> {
        self.object_property_from_value(value, key)
            .map_err(|err| match err {
                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                    Error::ScriptRuntime(format!(
                        "variable '{}' is not an object (key '{}')",
                        variable_name, key
                    ))
                }
                other => other,
            })
    }
}
