use super::*;

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

    pub(crate) fn resolved_role_for_node(&self, node: NodeId) -> String {
        if let Some(explicit) = self.dom.attr(node, "role") {
            return explicit;
        }
        if let Some(tag) = self.dom.tag_name(node) {
            if tag.eq_ignore_ascii_case("address") {
                return "group".to_string();
            }
            if tag.eq_ignore_ascii_case("aside") {
                return "complementary".to_string();
            }
            if tag.eq_ignore_ascii_case("article") {
                return "article".to_string();
            }
            if tag.eq_ignore_ascii_case("blockquote") {
                return "blockquote".to_string();
            }
            if tag.eq_ignore_ascii_case("body") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("button") {
                return "button".to_string();
            }
            if tag.eq_ignore_ascii_case("caption") {
                return "caption".to_string();
            }
            if tag.eq_ignore_ascii_case("code") {
                return "code".to_string();
            }
            if tag.eq_ignore_ascii_case("datalist") {
                return "listbox".to_string();
            }
            if tag.eq_ignore_ascii_case("details") {
                return "group".to_string();
            }
            if tag.eq_ignore_ascii_case("div") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("dialog") {
                return "dialog".to_string();
            }
            if tag.eq_ignore_ascii_case("del") {
                return "deletion".to_string();
            }
            if tag.eq_ignore_ascii_case("dfn") {
                return "term".to_string();
            }
            if tag.eq_ignore_ascii_case("em") {
                return "emphasis".to_string();
            }
            if tag.eq_ignore_ascii_case("fieldset") {
                return "group".to_string();
            }
            if tag.eq_ignore_ascii_case("figure") {
                return "figure".to_string();
            }
            if tag.eq_ignore_ascii_case("form") {
                return "form".to_string();
            }
            if tag.eq_ignore_ascii_case("header") {
                return self.resolved_header_role(node);
            }
            if tag.eq_ignore_ascii_case("hgroup") {
                return "group".to_string();
            }
            if tag.eq_ignore_ascii_case("hr") {
                return "separator".to_string();
            }
            if tag.eq_ignore_ascii_case("html") {
                return "document".to_string();
            }
            if tag.eq_ignore_ascii_case("input") {
                return self.resolved_input_role(node);
            }
            if tag.len() == 2 {
                let mut chars = tag.chars();
                if let (Some(prefix), Some(level), None) =
                    (chars.next(), chars.next(), chars.next())
                {
                    if (prefix == 'h' || prefix == 'H') && ('1'..='6').contains(&level) {
                        return "heading".to_string();
                    }
                }
            }
            if tag.eq_ignore_ascii_case("footer") {
                return self.resolved_footer_role(node);
            }
            if tag.eq_ignore_ascii_case("b") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("bdi") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("bdo") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("data") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("i") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("img") {
                if self.dom.attr(node, "alt").is_some_and(|alt| alt.is_empty()) {
                    return "presentation".to_string();
                }
                return "img".to_string();
            }
            if tag.eq_ignore_ascii_case("ins") {
                return "insertion".to_string();
            }
            if tag.eq_ignore_ascii_case("li") {
                return self.resolved_list_item_role(node);
            }
            if tag.eq_ignore_ascii_case("main") {
                return "main".to_string();
            }
            if tag.eq_ignore_ascii_case("ol") {
                return "list".to_string();
            }
            if tag.eq_ignore_ascii_case("menu") {
                return "list".to_string();
            }
            if tag.eq_ignore_ascii_case("ul") {
                return "list".to_string();
            }
            if tag.eq_ignore_ascii_case("meter") {
                return "meter".to_string();
            }
            if tag.eq_ignore_ascii_case("nav") {
                return "navigation".to_string();
            }
            if tag.eq_ignore_ascii_case("optgroup") {
                return "group".to_string();
            }
            if tag.eq_ignore_ascii_case("option") {
                return "option".to_string();
            }
            if tag.eq_ignore_ascii_case("output") {
                return "status".to_string();
            }
            if tag.eq_ignore_ascii_case("p") {
                return "paragraph".to_string();
            }
            if tag.eq_ignore_ascii_case("pre") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("progress") {
                return "progressbar".to_string();
            }
            if tag.eq_ignore_ascii_case("q") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("s") {
                return "deletion".to_string();
            }
            if tag.eq_ignore_ascii_case("samp") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("small") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("strong") {
                return "strong".to_string();
            }
            if tag.eq_ignore_ascii_case("sub") {
                return "subscript".to_string();
            }
            if tag.eq_ignore_ascii_case("sup") {
                return "superscript".to_string();
            }
            if tag.eq_ignore_ascii_case("table") {
                return "table".to_string();
            }
            if tag.eq_ignore_ascii_case("tbody") {
                return "rowgroup".to_string();
            }
            if tag.eq_ignore_ascii_case("tfoot") {
                return "rowgroup".to_string();
            }
            if tag.eq_ignore_ascii_case("thead") {
                return "rowgroup".to_string();
            }
            if tag.eq_ignore_ascii_case("tr") {
                return "row".to_string();
            }
            if tag.eq_ignore_ascii_case("th") {
                return self.resolved_table_header_role(node);
            }
            if tag.eq_ignore_ascii_case("td") {
                return self.resolved_table_data_cell_role(node);
            }
            if tag.eq_ignore_ascii_case("textarea") {
                return "textbox".to_string();
            }
            if tag.eq_ignore_ascii_case("time") {
                return "time".to_string();
            }
            if tag.eq_ignore_ascii_case("u") {
                return "generic".to_string();
            }
            if tag.eq_ignore_ascii_case("select") {
                return self.resolved_select_role(node);
            }
            if tag.eq_ignore_ascii_case("section") {
                return self.resolved_section_role(node);
            }
            if tag.eq_ignore_ascii_case("search") {
                return "search".to_string();
            }
            if (tag.eq_ignore_ascii_case("a")
                || tag.eq_ignore_ascii_case("area")
                || tag.eq_ignore_ascii_case("link"))
                && self.dom.attr(node, "href").is_some()
            {
                return "link".to_string();
            }
        }
        String::new()
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

    pub(crate) fn col_span_value(&self, node: NodeId) -> i64 {
        self.dom
            .attr(node, "span")
            .and_then(|raw| Self::parse_positive_int(&raw))
            .unwrap_or(1)
    }

    pub(crate) fn set_col_span_value(&mut self, node: NodeId, value: &Value) -> Result<()> {
        let next = match value {
            Value::Number(number) => *number,
            Value::Float(number) if number.is_finite() => *number as i64,
            Value::BigInt(number) => number.to_string().parse::<i64>().unwrap_or(1),
            other => other.as_string().trim().parse::<i64>().unwrap_or(1),
        };
        let next = if next <= 0 { 1 } else { next };
        self.dom.set_attr(node, "span", &next.to_string())
    }

    pub(crate) fn is_canvas_2d_context_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_CANVAS_2D_CONTEXT_OBJECT_KEY),
            Some(Value::Bool(true))
        )
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

    pub(crate) fn new_canvas_2d_context_value(&self, alpha: bool) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CANVAS_2D_CONTEXT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (INTERNAL_CANVAS_2D_ALPHA_KEY.to_string(), Value::Bool(alpha)),
            (
                "fillStyle".to_string(),
                Value::String("#000000".to_string()),
            ),
            (
                "strokeStyle".to_string(),
                Value::String("#000000".to_string()),
            ),
            ("lineWidth".to_string(), Value::Number(1)),
        ])
    }

    pub(crate) fn new_array_value(values: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(ArrayValue::new(values))))
    }

    pub(crate) fn set_array_property(array: &Rc<RefCell<ArrayValue>>, key: String, value: Value) {
        Self::object_set_entry(&mut array.borrow_mut().properties, key, value);
    }

    pub(crate) fn new_object_value(entries: Vec<(String, Value)>) -> Value {
        Value::Object(Rc::new(RefCell::new(ObjectValue::new(entries))))
    }

    pub(crate) fn mock_file_to_value(file: &MockFile) -> Value {
        Self::new_object_value(vec![
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
        ])
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

    pub(crate) fn new_boolean_constructor_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("boolean_constructor".to_string()),
        )])
    }

    pub(crate) fn new_string_wrapper_value(value: String) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
            Value::String(value),
        )])
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

    fn object_property_from_entries_with_getter(
        &mut self,
        receiver: &Value,
        entries: &(impl ObjectEntryLookup + ?Sized),
        key: &str,
    ) -> Result<Option<Value>> {
        if let Some(getter) = Self::object_getter_from_entries(entries, key) {
            return Ok(Some(self.invoke_object_getter(&getter, receiver)?));
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
        let mut out = String::new();
        let mut uppercase_next = false;
        for ch in raw.chars() {
            if ch == '-' {
                uppercase_next = true;
                continue;
            }
            if uppercase_next {
                out.push(ch.to_ascii_uppercase());
                uppercase_next = false;
            } else {
                out.push(ch);
            }
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

    pub(crate) fn object_property_from_value(&mut self, value: &Value, key: &str) -> Result<Value> {
        match value {
            Value::Node(node) => {
                let is_select = self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("select"))
                    .unwrap_or(false);
                let is_col_or_colgroup = self
                    .dom
                    .tag_name(*node)
                    .map(|tag| {
                        tag.eq_ignore_ascii_case("col") || tag.eq_ignore_ascii_case("colgroup")
                    })
                    .unwrap_or(false);
                let select_options = || {
                    let mut options = Vec::new();
                    self.dom.collect_select_options(*node, &mut options);
                    options
                };

                match key {
                    "textContent" | "innerText" => Ok(Value::String(self.dom.text_content(*node))),
                    "innerHTML" => Ok(Value::String(self.dom.inner_html(*node)?)),
                    "outerHTML" => Ok(Value::String(self.dom.outer_html(*node)?)),
                    "value" => Ok(Value::String(self.dom.value(*node)?)),
                    "files" => self.input_files_value(*node),
                    "valueAsNumber" => Ok(Self::number_value(self.input_value_as_number(*node)?)),
                    "valueAsDate" => Ok(self
                        .input_value_as_date_ms(*node)?
                        .map(Self::new_date_value)
                        .unwrap_or(Value::Null)),
                    "checked" => Ok(Value::Bool(self.dom.checked(*node)?)),
                    "disabled" => Ok(Value::Bool(self.dom.disabled(*node))),
                    "required" => Ok(Value::Bool(self.dom.required(*node))),
                    "readonly" | "readOnly" => Ok(Value::Bool(self.dom.readonly(*node))),
                    "id" => Ok(Value::String(
                        self.dom.attr(*node, "id").unwrap_or_default(),
                    )),
                    "name" => Ok(Value::String(
                        self.dom.attr(*node, "name").unwrap_or_default(),
                    )),
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
                        self.dom
                            .attr(*node, "contenteditable")
                            .unwrap_or_else(|| "inherit".to_string()),
                    )),
                    "draggable" => Ok(Value::Bool(
                        self.dom
                            .attr(*node, "draggable")
                            .is_some_and(|value| value.eq_ignore_ascii_case("true")),
                    )),
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
                    "spellcheck" => Ok(Value::Bool(
                        self.dom
                            .attr(*node, "spellcheck")
                            .is_some_and(|value| !value.eq_ignore_ascii_case("false")),
                    )),
                    "tabIndex" | "tabindex" => Ok(Value::Number(
                        self.dom
                            .attr(*node, "tabindex")
                            .and_then(|raw| raw.trim().parse::<i64>().ok())
                            .unwrap_or(-1),
                    )),
                    "translate" => Ok(Value::Bool(
                        !self
                            .dom
                            .attr(*node, "translate")
                            .is_some_and(|value| value.eq_ignore_ascii_case("no")),
                    )),
                    "cite" => Ok(Value::String(
                        self.dom.attr(*node, "cite").unwrap_or_default(),
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
                    "span" if is_col_or_colgroup => Ok(Value::Number(self.col_span_value(*node))),
                    "type" => {
                        if self
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
                    "srclang" | "srcLang" if self.is_track_element(*node) => Ok(Value::String(
                        self.dom.attr(*node, "srclang").unwrap_or_default(),
                    )),
                    "label" if self.is_track_element(*node) => Ok(Value::String(
                        self.dom.attr(*node, "label").unwrap_or_default(),
                    )),
                    "default" if self.is_track_element(*node) => {
                        Ok(Value::Bool(self.dom.attr(*node, "default").is_some()))
                    }
                    "disablePictureInPicture" | "disablepictureinpicture" => Ok(Value::Bool(
                        self.dom.attr(*node, "disablepictureinpicture").is_some(),
                    )),
                    "media" => Ok(Value::String(
                        self.dom.attr(*node, "media").unwrap_or_default(),
                    )),
                    "playsInline" | "playsinline" => {
                        Ok(Value::Bool(self.dom.attr(*node, "playsinline").is_some()))
                    }
                    "poster" => Ok(Value::String(
                        self.dom
                            .attr(*node, "poster")
                            .map(|raw| self.resolve_document_target_url(&raw))
                            .unwrap_or_default(),
                    )),
                    "sizes" => Ok(Value::String(
                        self.dom.attr(*node, "sizes").unwrap_or_default(),
                    )),
                    "srcset" | "srcSet" => Ok(Value::String(
                        self.dom.attr(*node, "srcset").unwrap_or_default(),
                    )),
                    "width" => Ok(Value::Number(self.canvas_dimension_value(*node, "width"))),
                    "height" => Ok(Value::Number(self.canvas_dimension_value(*node, "height"))),
                    "tagName" => Ok(Value::String(
                        self.dom
                            .tag_name(*node)
                            .unwrap_or_default()
                            .to_ascii_uppercase(),
                    )),
                    "className" => Ok(Value::String(
                        self.dom.attr(*node, "class").unwrap_or_default(),
                    )),
                    "role" => Ok(Value::String(self.resolved_role_for_node(*node))),
                    "baseURI" => Ok(Value::String(self.document_base_url())),
                    "dataset" => Ok(Self::new_object_value(self.dataset_entries_for_node(*node))),
                    "options" => {
                        if !is_select {
                            return Ok(Value::Undefined);
                        }
                        Ok(Value::NodeList(select_options()))
                    }
                    "selectedIndex" => {
                        if !is_select {
                            return Ok(Value::Undefined);
                        }
                        let options = select_options();
                        if options.is_empty() {
                            return Ok(Value::Number(-1));
                        }
                        let selected = options
                            .iter()
                            .position(|option| self.dom.attr(*option, "selected").is_some())
                            .unwrap_or(0);
                        Ok(Value::Number(selected as i64))
                    }
                    "length" => {
                        if !is_select {
                            return Ok(Value::Undefined);
                        }
                        Ok(Value::Number(select_options().len() as i64))
                    }
                    _ if key.starts_with("on") => Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(*node, key.to_string()))
                        .cloned()
                        .unwrap_or(Value::Null)),
                    _ => Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(*node, key.to_string()))
                        .cloned()
                        .unwrap_or(Value::Undefined)),
                }
            }
            Value::String(text) => {
                if key == "length" {
                    Ok(Value::Number(text.chars().count() as i64))
                } else if key == "constructor" {
                    Ok(Value::StringConstructor)
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(text
                        .chars()
                        .nth(index)
                        .map(|ch| Value::String(ch.to_string()))
                        .unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Array(values) => {
                let values = values.borrow();
                if key == "length" {
                    Ok(Value::Number(values.len() as i64))
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(values.get(index).cloned().unwrap_or(Value::Undefined))
                } else if let Some(value) = Self::object_get_entry(&values.properties, key) {
                    Ok(value)
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::NodeList(nodes) => {
                if key == "length" {
                    Ok(Value::Number(nodes.len() as i64))
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(nodes
                        .get(index)
                        .copied()
                        .map(Value::Node)
                        .unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::TypedArray(values) => {
                let snapshot = self.typed_array_snapshot(values)?;
                if key == "length" {
                    Ok(Value::Number(snapshot.len() as i64))
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(snapshot.get(index).cloned().unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Object(entries) => {
                let entries = entries.borrow();
                let key_is_to_string_tag = Self::symbol_id_from_storage_key(key)
                    .and_then(|symbol_id| self.symbol_runtime.symbols_by_id.get(&symbol_id))
                    .and_then(|symbol| symbol.description.as_deref())
                    .is_some_and(|description| description == "Symbol.toStringTag")
                    || key == "Symbol.toStringTag";
                if let Some(text) = Self::string_wrapper_value_from_object(&entries) {
                    if key == "length" {
                        return Ok(Value::Number(text.chars().count() as i64));
                    }
                    if key == "constructor" {
                        return Ok(Value::StringConstructor);
                    }
                    if let Ok(index) = key.parse::<usize>() {
                        return Ok(text
                            .chars()
                            .nth(index)
                            .map(|ch| Value::String(ch.to_string()))
                            .unwrap_or(Value::Undefined));
                    }
                }
                if Self::is_generator_object(&entries) && key == "constructor" {
                    let constructor = self.new_generator_function_constructor_value();
                    if let Value::Object(constructor_entries) = constructor {
                        let constructor_entries = constructor_entries.borrow();
                        if let Some(value) =
                            Self::object_get_entry(&constructor_entries, "prototype")
                        {
                            return Ok(value);
                        }
                    }
                }
                if Self::is_async_generator_object(&entries) && key == "constructor" {
                    let constructor = self.new_async_generator_function_constructor_value();
                    if let Value::Object(constructor_entries) = constructor {
                        let constructor_entries = constructor_entries.borrow();
                        if let Some(value) =
                            Self::object_get_entry(&constructor_entries, "prototype")
                        {
                            return Ok(value);
                        }
                    }
                }
                if Self::is_generator_function_prototype_object(&entries) && key_is_to_string_tag {
                    return Ok(Value::String("GeneratorFunction".to_string()));
                }
                if Self::is_generator_object(&entries)
                    || Self::is_generator_prototype_object(&entries)
                {
                    if key_is_to_string_tag {
                        return Ok(Value::String("Generator".to_string()));
                    }
                }
                if key_is_to_string_tag {
                    let looks_like_generator_prototype = matches!(
                        Self::object_get_entry(&entries, "constructor"),
                        Some(Value::Object(constructor))
                            if Self::is_generator_function_prototype_object(&constructor.borrow())
                    ) && Self::object_get_entry(
                        &entries, "next",
                    )
                    .is_some()
                        && Self::object_get_entry(&entries, "return").is_some()
                        && Self::object_get_entry(&entries, "throw").is_some();
                    if looks_like_generator_prototype {
                        return Ok(Value::String("Generator".to_string()));
                    }
                }
                if Self::is_async_generator_function_prototype_object(&entries)
                    && key_is_to_string_tag
                {
                    return Ok(Value::String("AsyncGeneratorFunction".to_string()));
                }
                if Self::is_async_generator_object(&entries)
                    || Self::is_async_generator_prototype_object(&entries)
                {
                    if key_is_to_string_tag {
                        return Ok(Value::String("AsyncGenerator".to_string()));
                    }
                }
                if key_is_to_string_tag {
                    let looks_like_async_generator_prototype =
                        matches!(
                            Self::object_get_entry(&entries, "constructor"),
                            Some(Value::Object(constructor))
                                if Self::is_async_generator_function_prototype_object(
                                    &constructor.borrow()
                                )
                        ) && Self::object_get_entry(&entries, "next").is_some()
                            && Self::object_get_entry(&entries, "return").is_some()
                            && Self::object_get_entry(&entries, "throw").is_some();
                    if looks_like_async_generator_prototype {
                        return Ok(Value::String("AsyncGenerator".to_string()));
                    }
                }
                if Self::is_url_search_params_object(&entries) {
                    if key == "size" {
                        let size =
                            Self::url_search_params_pairs_from_object_entries(&entries).len();
                        return Ok(Value::Number(size as i64));
                    }
                }
                if Self::is_storage_object(&entries) {
                    if key == "length" {
                        let len = Self::storage_pairs_from_object_entries(&entries).len();
                        return Ok(Value::Number(len as i64));
                    }
                    if let Some(value) = Self::object_get_entry(&entries, key) {
                        return Ok(value);
                    }
                    if Self::is_storage_method_name(key) {
                        return Ok(Self::new_builtin_placeholder_function());
                    }
                    if let Some((_, value)) = Self::storage_pairs_from_object_entries(&entries)
                        .into_iter()
                        .find(|(name, _)| name == key)
                    {
                        return Ok(Value::String(value));
                    }
                    return Ok(Value::Undefined);
                }
                let is_document_object = matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                );
                if is_document_object {
                    let value = match key {
                        "body" => self.dom.body().map(Value::Node).unwrap_or(Value::Null),
                        "head" => self.dom.head().map(Value::Node).unwrap_or(Value::Null),
                        "documentElement" => self
                            .dom
                            .document_element()
                            .map(Value::Node)
                            .unwrap_or(Value::Null),
                        _ => Value::Undefined,
                    };
                    if !matches!(value, Value::Undefined) {
                        return Ok(value);
                    }
                }
                if Self::is_url_object(&entries) && key == "constructor" {
                    return Ok(Value::UrlConstructor);
                }
                if let Some(value) = self.object_property_from_entries_with_getter(value, &entries, key)? {
                    return Ok(value);
                }

                let mut prototype = Self::object_get_entry(&entries, INTERNAL_OBJECT_PROTOTYPE_KEY);
                drop(entries);

                while let Some(Value::Object(object)) = prototype {
                    let object_ref = object.borrow();
                    if let Some(value) =
                        self.object_property_from_entries_with_getter(value, &object_ref, key)?
                    {
                        return Ok(value);
                    }
                    prototype = Self::object_get_entry(&object_ref, INTERNAL_OBJECT_PROTOTYPE_KEY);
                }
                Ok(Value::Undefined)
            }
            Value::Promise(promise) => {
                if key == "constructor" {
                    Ok(Value::PromiseConstructor)
                } else {
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
            }
            Value::Map(map) => {
                let map = map.borrow();
                let key_is_to_string_tag = Self::symbol_id_from_storage_key(key)
                    .and_then(|symbol_id| self.symbol_runtime.symbols_by_id.get(&symbol_id))
                    .and_then(|symbol| symbol.description.as_deref())
                    .is_some_and(|description| description == "Symbol.toStringTag")
                    || key == "Symbol.toStringTag";
                if key == "size" {
                    Ok(Value::Number(map.entries.len() as i64))
                } else if key_is_to_string_tag {
                    Ok(Value::String("Map".to_string()))
                } else if key == "constructor" {
                    Ok(Value::MapConstructor)
                } else if let Some(value) = Self::object_get_entry(&map.properties, key) {
                    Ok(value)
                } else if Self::is_map_method_name(key) {
                    Ok(Self::new_builtin_placeholder_function())
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::WeakMap(weak_map) => {
                let weak_map = weak_map.borrow();
                let key_is_to_string_tag = Self::symbol_id_from_storage_key(key)
                    .and_then(|symbol_id| self.symbol_runtime.symbols_by_id.get(&symbol_id))
                    .and_then(|symbol| symbol.description.as_deref())
                    .is_some_and(|description| description == "Symbol.toStringTag")
                    || key == "Symbol.toStringTag";
                if key_is_to_string_tag {
                    Ok(Value::String("WeakMap".to_string()))
                } else if key == "constructor" {
                    Ok(Value::WeakMapConstructor)
                } else if let Some(value) = Self::object_get_entry(&weak_map.properties, key) {
                    Ok(value)
                } else if Self::is_weak_map_method_name(key) {
                    Ok(Self::new_builtin_placeholder_function())
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::WeakSet(weak_set) => {
                let weak_set = weak_set.borrow();
                let key_is_to_string_tag = Self::symbol_id_from_storage_key(key)
                    .and_then(|symbol_id| self.symbol_runtime.symbols_by_id.get(&symbol_id))
                    .and_then(|symbol| symbol.description.as_deref())
                    .is_some_and(|description| description == "Symbol.toStringTag")
                    || key == "Symbol.toStringTag";
                if key_is_to_string_tag {
                    Ok(Value::String("WeakSet".to_string()))
                } else if key == "constructor" {
                    Ok(Value::WeakSetConstructor)
                } else if let Some(value) = Self::object_get_entry(&weak_set.properties, key) {
                    Ok(value)
                } else if Self::is_weak_set_method_name(key) {
                    Ok(Self::new_builtin_placeholder_function())
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Set(set) => {
                let set = set.borrow();
                let key_is_to_string_tag = Self::symbol_id_from_storage_key(key)
                    .and_then(|symbol_id| self.symbol_runtime.symbols_by_id.get(&symbol_id))
                    .and_then(|symbol| symbol.description.as_deref())
                    .is_some_and(|description| description == "Symbol.toStringTag")
                    || key == "Symbol.toStringTag";
                if key == "size" {
                    Ok(Value::Number(set.values.len() as i64))
                } else if key_is_to_string_tag {
                    Ok(Value::String("Set".to_string()))
                } else if key == "constructor" {
                    Ok(Value::SetConstructor)
                } else {
                    Ok(Self::object_get_entry(&set.properties, key).unwrap_or(Value::Undefined))
                }
            }
            Value::Blob(blob) => {
                let blob = blob.borrow();
                match key {
                    "size" => Ok(Value::Number(blob.bytes.len() as i64)),
                    "type" => Ok(Value::String(blob.mime_type.clone())),
                    "constructor" => Ok(Value::BlobConstructor),
                    _ => Ok(Value::Undefined),
                }
            }
            Value::ArrayBuffer(_) => {
                if key == "constructor" {
                    Ok(Value::ArrayBufferConstructor)
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Symbol(symbol) => {
                let value = match key {
                    "description" => symbol
                        .description
                        .as_ref()
                        .map(|value| Value::String(value.clone()))
                        .unwrap_or(Value::Undefined),
                    "constructor" => Value::SymbolConstructor,
                    _ => Value::Undefined,
                };
                Ok(value)
            }
            Value::RegExp(regex) => {
                let regex = regex.borrow();
                let value = match key {
                    "source" => Value::String(regex.source.clone()),
                    "flags" => Value::String(regex.flags.clone()),
                    "global" => Value::Bool(regex.global),
                    "ignoreCase" => Value::Bool(regex.ignore_case),
                    "multiline" => Value::Bool(regex.multiline),
                    "dotAll" => Value::Bool(regex.dot_all),
                    "sticky" => Value::Bool(regex.sticky),
                    "hasIndices" => Value::Bool(regex.has_indices),
                    "unicode" => Value::Bool(regex.unicode),
                    "unicodeSets" => Value::Bool(regex.unicode_sets),
                    "lastIndex" => Value::Number(regex.last_index as i64),
                    "constructor" => Value::RegExpConstructor,
                    _ => Self::object_get_entry(&regex.properties, key).unwrap_or(Value::Undefined),
                };
                Ok(value)
            }
            Value::Function(function) => {
                if let Some(entries) = self
                    .script_runtime
                    .function_public_properties
                    .get(&function.function_id)
                    .cloned()
                {
                    if let Some(custom_value) =
                        self.object_property_from_entries_with_getter(value, &entries, key)?
                    {
                        return Ok(custom_value);
                    }
                }
                let own_value = match key {
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
                    _ => Value::Undefined,
                };
                if !matches!(own_value, Value::Undefined) {
                    return Ok(own_value);
                }
                if let Some(super_constructor) = function.class_super_constructor.clone() {
                    if !matches!(super_constructor, Value::Null) {
                        let inherited = self.object_property_from_value(&super_constructor, key)?;
                        if !matches!(inherited, Value::Undefined) {
                            return Ok(inherited);
                        }
                    }
                }
                Ok(Value::Undefined)
            }
            Value::UrlConstructor => {
                if let Some(value) = Self::object_get_entry(
                    &self.browser_apis.url_constructor_properties.borrow(),
                    key,
                ) {
                    return Ok(value);
                }
                if Self::is_url_static_method_name(key) {
                    return Ok(Self::new_builtin_placeholder_function());
                }
                Ok(Value::Undefined)
            }
            Value::StringConstructor => Ok(Value::Undefined),
            _ => Err(Error::ScriptRuntime("value is not an object".into())),
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
