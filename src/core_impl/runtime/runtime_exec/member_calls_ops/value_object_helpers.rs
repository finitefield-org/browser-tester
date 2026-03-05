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
        if value < 0 {
            None
        } else {
            Some(value)
        }
    }

    pub(crate) fn parse_positive_int(raw: &str) -> Option<i64> {
        let value = raw.trim().parse::<i64>().ok()?;
        if value <= 0 {
            None
        } else {
            Some(value)
        }
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

    pub(crate) fn form_data_append_string_value(
        value: &Value,
        filename: Option<&Value>,
    ) -> String {
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
        if key == "Enter" {
            13
        } else {
            0
        }
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

    pub(crate) fn new_clipboard_data_object_value(text: &str) -> Value {
        let mut store = ObjectValue::default();
        let types = if text.is_empty() {
            Vec::new()
        } else {
            store.set_entry("text/plain".to_string(), Value::String(text.to_string()));
            vec![Value::String("text/plain".to_string())]
        };
        let store = Value::Object(Rc::new(RefCell::new(store)));
        Self::new_object_value(vec![
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
            ("types".to_string(), Self::new_array_value(types)),
        ])
    }

    pub(crate) fn new_data_transfer_object_value(event_type: &str) -> Value {
        let value = Self::new_clipboard_data_object_value("");
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
            Self::object_set_entry(
                &mut entries,
                "files".to_string(),
                Self::new_array_value(Vec::new()),
            );
            let items =
                Self::new_data_transfer_item_list_value(owner.clone(), event_type, Vec::new());
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
        }
        value
    }

    pub(crate) fn new_data_transfer_item_string_value(format: &str, data: &str) -> Value {
        Self::new_object_value(vec![
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
        ])
    }

    pub(crate) fn new_data_transfer_item_file_value(format: &str, file: Value) -> Value {
        Self::new_object_value(vec![
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
        ])
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
        }
        value
    }

    fn new_named_node_map_iterator_callable(owner: NodeId) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("named_node_map_iterator".to_string()),
            ),
            (
                INTERNAL_NAMED_NODE_MAP_OWNER_NODE_KEY.to_string(),
                Value::Node(owner),
            ),
        ])
    }

    pub(crate) fn new_named_node_map_value(&mut self, owner: NodeId) -> Value {
        let iterator_symbol = self.eval_symbol_static_property(SymbolStaticProperty::Iterator);
        let iterator_key = self.property_key_to_storage_key(&iterator_symbol);
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);

        Self::new_object_value(vec![
            (
                INTERNAL_NAMED_NODE_MAP_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_NAMED_NODE_MAP_OWNER_NODE_KEY.to_string(),
                Value::Node(owner),
            ),
            (
                iterator_key,
                Self::new_named_node_map_iterator_callable(owner),
            ),
            (to_string_tag_key, Value::String("NamedNodeMap".to_string())),
        ])
    }

    pub(crate) fn new_range_object_value(root: NodeId) -> Value {
        Self::new_object_value(vec![
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
        ])
    }

    pub(crate) fn new_selection_object_value(root: NodeId) -> Value {
        let range = Self::new_range_object_value(root);
        Self::new_object_value(vec![
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
        ])
    }

    pub(crate) fn new_animation_object_value(
        id: String,
        keyframes: Value,
        options: Value,
        timeline: Value,
        range_start: Value,
        range_end: Value,
    ) -> Value {
        Self::new_object_value(vec![
            (INTERNAL_ANIMATION_OBJECT_KEY.to_string(), Value::Bool(true)),
            ("id".to_string(), Value::String(id)),
            (
                "playState".to_string(),
                Value::String("running".to_string()),
            ),
            ("currentTime".to_string(), Value::Number(0)),
            ("startTime".to_string(), Value::Number(0)),
            ("pending".to_string(), Value::Bool(false)),
            ("timeline".to_string(), timeline),
            ("rangeStart".to_string(), range_start),
            ("rangeEnd".to_string(), range_end),
            ("keyframes".to_string(), keyframes),
            ("options".to_string(), options),
            (
                "cancel".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "finish".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "pause".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("play".to_string(), Self::new_builtin_placeholder_function()),
            (
                "reverse".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "updatePlaybackRate".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "commitStyles".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "persist".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "Symbol.toStringTag".to_string(),
                Value::String("Animation".to_string()),
            ),
        ])
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
                Self::new_builtin_placeholder_function(),
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

    pub(crate) fn delete_object_property_entries(entries: &mut ObjectValue, key: &str) -> bool {
        let mut deleted = entries.delete_entry(key);
        let getter_key = Self::object_getter_storage_key(key);
        deleted |= entries.delete_entry(&getter_key);
        let setter_key = Self::object_setter_storage_key(key);
        deleted |= entries.delete_entry(&setter_key);
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

    pub(crate) fn new_class_list_value(node: NodeId) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CLASS_LIST_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (INTERNAL_CLASS_LIST_NODE_KEY.to_string(), Value::Node(node)),
            (
                "add".to_string(),
                Self::new_class_list_method_callable("class_list_add"),
            ),
            (
                "remove".to_string(),
                Self::new_class_list_method_callable("class_list_remove"),
            ),
            (
                "toggle".to_string(),
                Self::new_class_list_method_callable("class_list_toggle"),
            ),
            (
                "contains".to_string(),
                Self::new_class_list_method_callable("class_list_contains"),
            ),
            (
                "replace".to_string(),
                Self::new_class_list_method_callable("class_list_replace"),
            ),
            (
                "item".to_string(),
                Self::new_class_list_method_callable("class_list_item"),
            ),
            (
                "forEach".to_string(),
                Self::new_class_list_method_callable("class_list_for_each"),
            ),
            (
                "toString".to_string(),
                Self::new_class_list_method_callable("class_list_to_string"),
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

    pub(crate) fn new_object_constructor_value() -> Value {
        let prototype = Self::new_object_value(Vec::new());
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("object_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype),
            ("assign".to_string(), Self::new_builtin_placeholder_function()),
        ])
    }

    pub(crate) fn new_event_target_constructor_value() -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("event_target_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
    }

    pub(crate) fn new_event_constructor_value() -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("event_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
    }

    pub(crate) fn new_custom_event_constructor_value() -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("custom_event_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
    }

    pub(crate) fn new_mouse_event_constructor_value() -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("mouse_event_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
    }

    pub(crate) fn new_keyboard_event_constructor_value() -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("keyboard_event_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
            ("DOM_KEY_LOCATION_STANDARD".to_string(), Value::Number(0x00)),
            ("DOM_KEY_LOCATION_LEFT".to_string(), Value::Number(0x01)),
            ("DOM_KEY_LOCATION_RIGHT".to_string(), Value::Number(0x02)),
            ("DOM_KEY_LOCATION_NUMPAD".to_string(), Value::Number(0x03)),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
    }

    pub(crate) fn new_wheel_event_constructor_value() -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("wheel_event_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
            ("DOM_DELTA_PIXEL".to_string(), Value::Number(0)),
            ("DOM_DELTA_LINE".to_string(), Value::Number(1)),
            ("DOM_DELTA_PAGE".to_string(), Value::Number(2)),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
    }

    pub(crate) fn new_navigate_event_constructor_value() -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("navigate_event_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
    }

    pub(crate) fn new_pointer_event_constructor_value() -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("pointer_event_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
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
        Self::new_object_value(vec![
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
        ])
    }

    pub(crate) fn new_fetch_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("fetch_function".to_string()),
        )])
    }

    pub(crate) fn new_window_close_callable_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("window_close_function".to_string()),
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

    pub(crate) fn new_request_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("request_constructor".to_string()),
        )])
    }

    pub(crate) fn new_file_constructor_value() -> Value {
        let prototype = Self::new_object_value(vec![]);
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("file_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
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

    pub(crate) fn new_worker_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("worker_constructor".to_string()),
        )])
    }

    pub(crate) fn new_data_transfer_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("data_transfer_constructor".to_string()),
        )])
    }

    pub(crate) fn new_option_constructor_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("option_constructor".to_string()),
        )])
    }

    pub(crate) fn new_text_encoder_constructor_value() -> Value {
        let prototype = Self::new_object_value(vec![]);
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("text_encoder_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
    }

    pub(crate) fn new_text_decoder_constructor_value() -> Value {
        let prototype = Self::new_object_value(vec![]);
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("text_decoder_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
    }

    pub(crate) fn new_text_encoder_stream_constructor_value() -> Value {
        let prototype = Self::new_object_value(vec![]);
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("text_encoder_stream_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
    }

    pub(crate) fn new_text_decoder_stream_constructor_value() -> Value {
        let prototype = Self::new_object_value(vec![]);
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("text_decoder_stream_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
    }

    pub(crate) fn new_css_style_sheet_constructor_value() -> Value {
        let prototype = Self::new_object_value(vec![]);
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("css_style_sheet_constructor".to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ]);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
        }
        constructor
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

    pub(crate) fn new_text_encoder_instance_value() -> Value {
        Self::new_object_value(vec![
            (
                Self::object_getter_storage_key("encoding"),
                Self::new_text_encoder_encoding_getter_callable(),
            ),
            (
                "encode".to_string(),
                Self::new_text_encoder_encode_callable(),
            ),
            (
                "encodeInto".to_string(),
                Self::new_text_encoder_encode_into_callable(),
            ),
        ])
    }

    pub(crate) fn new_text_decoder_instance_value(
        encoding: &str,
        fatal: bool,
        ignore_bom: bool,
    ) -> Value {
        Self::new_object_value(vec![
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
            (
                Self::object_getter_storage_key("encoding"),
                Self::new_text_decoder_encoding_getter_callable(),
            ),
            (
                Self::object_getter_storage_key("fatal"),
                Self::new_text_decoder_fatal_getter_callable(),
            ),
            (
                Self::object_getter_storage_key("ignoreBOM"),
                Self::new_text_decoder_ignore_bom_getter_callable(),
            ),
            (
                "decode".to_string(),
                Self::new_text_decoder_decode_callable(),
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
            (
                Self::object_getter_storage_key("encoding"),
                Self::new_text_encoder_stream_encoding_getter_callable(),
            ),
            (
                Self::object_getter_storage_key("readable"),
                Self::new_text_encoder_stream_readable_getter_callable(),
            ),
            (
                Self::object_getter_storage_key("writable"),
                Self::new_text_encoder_stream_writable_getter_callable(),
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
            (
                Self::object_getter_storage_key("encoding"),
                Self::new_text_decoder_stream_encoding_getter_callable(),
            ),
            (
                Self::object_getter_storage_key("fatal"),
                Self::new_text_decoder_stream_fatal_getter_callable(),
            ),
            (
                Self::object_getter_storage_key("ignoreBOM"),
                Self::new_text_decoder_stream_ignore_bom_getter_callable(),
            ),
            (
                Self::object_getter_storage_key("readable"),
                Self::new_text_decoder_stream_readable_getter_callable(),
            ),
            (
                Self::object_getter_storage_key("writable"),
                Self::new_text_decoder_stream_writable_getter_callable(),
            ),
        ])
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
        Self::new_object_value(vec![
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
        ])
    }

    pub(crate) fn is_computed_style_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_COMPUTED_STYLE_OBJECT_KEY),
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

        match key {
            "getPropertyValue" => Ok(Some(
                Self::object_get_entry(entries, "getPropertyValue").unwrap_or(Value::Undefined),
            )),
            "setProperty" | "removeProperty" | "item" => {
                Ok(Some(Self::new_builtin_placeholder_function()))
            }
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

    pub(crate) fn new_worker_main_post_message_callable(worker: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("worker_main_post_message".to_string()),
            ),
            (INTERNAL_WORKER_TARGET_KEY.to_string(), worker),
        ])
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

    pub(crate) fn new_worker_terminate_callable(worker: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("worker_terminate".to_string()),
            ),
            (INTERNAL_WORKER_TARGET_KEY.to_string(), worker),
        ])
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
                "object_constructor" => "object_constructor",
                "event_target_constructor" => "event_target_constructor",
                "event_constructor" => "event_constructor",
                "custom_event_constructor" => "custom_event_constructor",
                "mouse_event_constructor" => "mouse_event_constructor",
                "keyboard_event_constructor" => "keyboard_event_constructor",
                "wheel_event_constructor" => "wheel_event_constructor",
                "navigate_event_constructor" => "navigate_event_constructor",
                "pointer_event_constructor" => "pointer_event_constructor",
                "dom_parser_constructor" => "dom_parser_constructor",
                "document_constructor" => "document_constructor",
                "document_parse_html" => "document_parse_html",
                "document_parse_html_unsafe" => "document_parse_html_unsafe",
                "fetch_function" => "fetch_function",
                "window_close_function" => "window_close_function",
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
                "window_alert_function" => "window_alert_function",
                "window_confirm_function" => "window_confirm_function",
                "window_print_function" => "window_print_function",
                "window_report_error_function" => "window_report_error_function",
                "window_prompt_function" => "window_prompt_function",
                "request_constructor" => "request_constructor",
                "file_constructor" => "file_constructor",
                "clipboard_item_constructor" => "clipboard_item_constructor",
                "clipboard_write" => "clipboard_write",
                "headers_constructor" => "headers_constructor",
                "worker_constructor" => "worker_constructor",
                "data_transfer_constructor" => "data_transfer_constructor",
                "option_constructor" => "option_constructor",
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
                "class_list_to_string" => "class_list_to_string",
                "worker_main_post_message" => "worker_main_post_message",
                "worker_context_post_message" => "worker_context_post_message",
                "worker_terminate" => "worker_terminate",
                "global_decode_uri" => "global_decode_uri",
                "global_decode_uri_component" => "global_decode_uri_component",
                "global_atob" => "global_atob",
                "global_btoa" => "global_btoa",
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
                "function_call" => "function_call",
                "function_apply" => "function_apply",
                "function_bind" => "function_bind",
                "function_to_string" => "function_to_string",
                "bound_function" => "bound_function",
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
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
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

    fn function_own_property_value(
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
            "call" => Self::new_function_call_callable(),
            "apply" => Self::new_function_apply_callable(),
            "bind" => Self::new_function_bind_callable(),
            "toString" if include_to_string => Self::new_function_to_string_callable(),
            _ => Value::Undefined,
        }
    }

    fn object_property_from_string_value(text: &str, key: &str) -> Value {
        if key == "length" {
            Value::Number(text.chars().count() as i64)
        } else if key == "constructor" {
            Value::StringConstructor
        } else if let Ok(index) = key.parse::<usize>() {
            text.chars()
                .nth(index)
                .map(|ch| Value::String(ch.to_string()))
                .unwrap_or(Value::Undefined)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_array_value(values: &Rc<RefCell<ArrayValue>>, key: &str) -> Value {
        let values = values.borrow();
        if key == "length" {
            Value::Number(values.len() as i64)
        } else if let Ok(index) = key.parse::<usize>() {
            values.get(index).cloned().unwrap_or(Value::Undefined)
        } else if let Some(value) = Self::object_get_entry(&values.properties, key) {
            value
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_node_list_value(
        &self,
        nodes: &Rc<RefCell<NodeListValue>>,
        key: &str,
    ) -> Value {
        if key == "length" {
            Value::Number(self.node_list_len(nodes) as i64)
        } else if let Ok(index) = key.parse::<usize>() {
            self.node_list_get(nodes, index)
                .map(Value::Node)
                .unwrap_or(Value::Undefined)
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_typed_array_value(
        &self,
        values: &Rc<RefCell<TypedArrayValue>>,
        key: &str,
    ) -> Result<Value> {
        let snapshot = self.typed_array_snapshot(values)?;
        if key == "length" {
            Ok(Value::Number(snapshot.len() as i64))
        } else if let Ok(index) = key.parse::<usize>() {
            Ok(snapshot.get(index).cloned().unwrap_or(Value::Undefined))
        } else {
            Ok(Value::Undefined)
        }
    }

    fn object_property_from_promise_value(promise: &Rc<RefCell<PromiseValue>>, key: &str) -> Value {
        if key == "constructor" {
            return Value::PromiseConstructor;
        }
        let promise = promise.borrow();
        if key == "status" {
            let status = match &promise.state {
                PromiseState::Pending => "pending",
                PromiseState::Fulfilled(_) => "fulfilled",
                PromiseState::Rejected(_) => "rejected",
            };
            Value::String(status.to_string())
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_map_value(&self, map: &Rc<RefCell<MapValue>>, key: &str) -> Value {
        let map = map.borrow();
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        if key == "size" {
            Value::Number(map.entries.len() as i64)
        } else if key_is_to_string_tag {
            Value::String("Map".to_string())
        } else if key == "constructor" {
            Value::MapConstructor
        } else if let Some(value) = Self::object_get_entry(&map.properties, key) {
            value
        } else if Self::is_map_method_name(key) {
            Self::new_builtin_placeholder_function()
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_weak_map_value(
        &self,
        weak_map: &Rc<RefCell<WeakMapValue>>,
        key: &str,
    ) -> Value {
        let weak_map = weak_map.borrow();
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        if key_is_to_string_tag {
            Value::String("WeakMap".to_string())
        } else if key == "constructor" {
            Value::WeakMapConstructor
        } else if let Some(value) = Self::object_get_entry(&weak_map.properties, key) {
            value
        } else if Self::is_weak_map_method_name(key) {
            Self::new_builtin_placeholder_function()
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_weak_set_value(
        &self,
        weak_set: &Rc<RefCell<WeakSetValue>>,
        key: &str,
    ) -> Value {
        let weak_set = weak_set.borrow();
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        if key_is_to_string_tag {
            Value::String("WeakSet".to_string())
        } else if key == "constructor" {
            Value::WeakSetConstructor
        } else if let Some(value) = Self::object_get_entry(&weak_set.properties, key) {
            value
        } else if Self::is_weak_set_method_name(key) {
            Self::new_builtin_placeholder_function()
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_set_value(&self, set: &Rc<RefCell<SetValue>>, key: &str) -> Value {
        let set = set.borrow();
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        if key == "size" {
            Value::Number(set.values.len() as i64)
        } else if key_is_to_string_tag {
            Value::String("Set".to_string())
        } else if key == "constructor" {
            Value::SetConstructor
        } else {
            Self::object_get_entry(&set.properties, key).unwrap_or(Value::Undefined)
        }
    }

    fn object_property_from_blob_value(blob: &Rc<RefCell<BlobValue>>, key: &str) -> Value {
        let blob = blob.borrow();
        match key {
            "size" => Value::Number(blob.bytes.len() as i64),
            "type" => Value::String(blob.mime_type.clone()),
            "constructor" => Value::BlobConstructor,
            _ => Value::Undefined,
        }
    }

    fn object_property_from_array_buffer_value(key: &str) -> Value {
        if key == "constructor" {
            Value::ArrayBufferConstructor
        } else {
            Value::Undefined
        }
    }

    fn object_property_from_symbol_value(symbol: &Rc<SymbolValue>, key: &str) -> Value {
        match key {
            "description" => symbol
                .description
                .as_ref()
                .map(|value| Value::String(value.clone()))
                .unwrap_or(Value::Undefined),
            "constructor" => Value::SymbolConstructor,
            _ => Value::Undefined,
        }
    }

    fn object_property_from_regexp_value(regex: &Rc<RefCell<RegexValue>>, key: &str) -> Value {
        let regex = regex.borrow();
        match key {
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
        let is_input = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false);
        let is_button = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("button"))
            .unwrap_or(false);
        let is_form_associated_control = is_form_control(&self.dom, *node);
        let is_labelable_control = self.is_labelable_control(*node);
        let is_col_or_colgroup = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("col") || tag.eq_ignore_ascii_case("colgroup"))
            .unwrap_or(false);
        let select_options = || self.select_option_nodes(*node);

        if is_select {
            if let Ok(index) = key.parse::<usize>() {
                return Ok(select_options()
                    .get(index)
                    .copied()
                    .map(Value::Node)
                    .unwrap_or(Value::Undefined));
            }
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
            "command" => {
                if is_button {
                    Ok(Value::String(self.dom.attr(*node, "command").unwrap_or_default()))
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
                if is_button {
                    let form_action = self
                        .dom
                        .attr(*node, "formaction")
                        .map(|raw| self.resolve_document_target_url(&raw))
                        .unwrap_or_default();
                    Ok(Value::String(form_action))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formEnctype" => {
                if is_button {
                    Ok(Value::String(self.dom.attr(*node, "formenctype").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formMethod" => {
                if is_button {
                    Ok(Value::String(self.dom.attr(*node, "formmethod").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formNoValidate" => {
                if is_button {
                    Ok(Value::Bool(self.dom.attr(*node, "formnovalidate").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "formTarget" => {
                if is_button {
                    Ok(Value::String(self.dom.attr(*node, "formtarget").unwrap_or_default()))
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
                        self.dom.attr(*node, "popovertargetaction").unwrap_or_default(),
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
            "classList" => Ok(Self::new_class_list_value(*node)),
            "slot" => Ok(Value::String(
                self.dom.attr(*node, "slot").unwrap_or_default(),
            )),
            "role" => {
                if let Some(role) = self.dom.attr(*node, "role") {
                    Ok(Value::String(role))
                } else if is_button {
                    Ok(Value::String("button".to_string()))
                } else {
                    Ok(Value::Null)
                }
            }
            "baseURI" => Ok(Value::String(self.document_base_url())),
            "dataset" => Ok(Self::new_object_value(self.dataset_entries_for_node(*node))),
            "options" => {
                if !is_select {
                    return Ok(Value::Undefined);
                }
                Ok(Self::new_static_node_list_value(select_options()))
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
                Ok(Self::new_static_node_list_value(
                    self.select_selected_option_nodes(*node),
                ))
            }
            "size" => {
                if !is_select {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(self.select_size_property_value(*node)))
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
                if !is_select {
                    return Ok(Value::Undefined);
                }
                Ok(Value::Number(select_options().len() as i64))
            }
            "getContext" | "toDataURL" | "toBlob" | "transferControlToOffscreen" => {
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

        if Self::is_class_list_object(entries) {
            let Some(node) = (match Self::object_get_entry(entries, INTERNAL_CLASS_LIST_NODE_KEY) {
                Some(Value::Node(node)) => Some(node),
                _ => None,
            }) else {
                return Some(Value::Undefined);
            };
            let classes = class_tokens(self.dom.attr(node, "class").as_deref());
            let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
            if key == "length" {
                return Some(Value::Number(classes.len() as i64));
            }
            if key == "value" {
                return Some(Value::String(classes.join(" ")));
            }
            if key == "constructor" {
                return Some(Value::Undefined);
            }
            if key_is_to_string_tag {
                return Some(Value::String("DOMTokenList".to_string()));
            }
            if let Ok(index) = key.parse::<usize>() {
                return Some(
                    classes
                        .get(index)
                        .cloned()
                        .map(Value::String)
                        .unwrap_or(Value::Undefined),
                );
            }
            if let Some(value) = Self::object_get_entry(entries, key) {
                return Some(value);
            }
            return Some(Value::Undefined);
        }

        None
    }

    fn object_property_from_web_api_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Result<Option<Value>> {
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

    fn object_property_from_match_media_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
        key_is_to_string_tag: bool,
    ) -> Option<Value> {
        if !Self::is_match_media_object(entries) {
            return None;
        }
        let query = Self::object_get_entry(entries, INTERNAL_MATCH_MEDIA_QUERY_KEY)
            .or_else(|| Self::object_get_entry(entries, "media"))
            .map(|value| value.as_string())
            .unwrap_or_default();
        if key == "matches" {
            let matches = self
                .platform_mocks
                .match_media_mocks
                .get(&query)
                .copied()
                .unwrap_or(self.platform_mocks.default_match_media_matches);
            return Some(Value::Bool(matches));
        }
        if key == "media" {
            return Some(Value::String(query));
        }
        if key_is_to_string_tag {
            return Some(Value::String("MediaQueryList".to_string()));
        }
        None
    }

    fn object_property_from_named_node_map_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_named_node_map_object(entries) {
            return None;
        }
        let owner =
            Self::named_node_map_owner_node(entries).filter(|node| self.dom.element(*node).is_some());
        let attrs = owner
            .map(|owner_node| self.named_node_map_entries(owner_node))
            .unwrap_or_default();
        if key == "length" {
            return Some(Value::Number(attrs.len() as i64));
        }
        if let Ok(index) = key.parse::<usize>() {
            let value = attrs
                .get(index)
                .and_then(|(name, value)| {
                    owner.map(|owner_node| Self::new_attr_object_value(name, value, Some(owner_node)))
                })
                .unwrap_or(Value::Undefined);
            return Some(value);
        }
        if let Some(owner_node) = owner {
            if let Some((name, value)) = attrs.iter().find(|(name, _)| name == key) {
                return Some(Self::new_attr_object_value(name, value, Some(owner_node)));
            }
        }
        None
    }

    fn object_property_from_string_wrapper_entries(entries: &ObjectValue, key: &str) -> Option<Value> {
        let text = Self::string_wrapper_value_from_object(entries)?;
        if key == "length" {
            return Some(Value::Number(text.chars().count() as i64));
        }
        if key == "constructor" {
            return Some(Value::StringConstructor);
        }
        if let Ok(index) = key.parse::<usize>() {
            return Some(
                text.chars()
                    .nth(index)
                    .map(|ch| Value::String(ch.to_string()))
                    .unwrap_or(Value::Undefined),
            );
        }
        None
    }

    fn object_property_from_match_media_named_node_map_or_string_wrapper_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        let key_is_to_string_tag = self.is_to_string_tag_property_key(key);
        if let Some(value) =
            self.object_property_from_match_media_entries(entries, key, key_is_to_string_tag)
        {
            return Some(value);
        }
        if let Some(value) = self.object_property_from_named_node_map_entries(entries, key) {
            return Some(value);
        }
        Self::object_property_from_string_wrapper_entries(entries, key)
    }

    fn callable_method_value_for_key(key: &str) -> Option<Value> {
        let method = match key {
            "call" => Self::new_function_call_callable(),
            "apply" => Self::new_function_apply_callable(),
            "bind" => Self::new_function_bind_callable(),
            _ => return None,
        };
        Some(method)
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
            if let Some(function_method) = Self::callable_method_value_for_key(key) {
                return Some(Self::object_get_entry(entries, key).unwrap_or(function_method));
            }
        }
        if let Some(value) = self.object_property_from_generator_constructor_entries(entries, key) {
            return Some(value);
        }
        self.object_property_from_generator_to_string_tag_entries(entries, key)
    }

    fn object_property_from_url_search_params_entries(entries: &ObjectValue, key: &str) -> Option<Value> {
        if Self::is_url_search_params_object(entries) && key == "size" {
            let size = Self::url_search_params_pairs_from_object_entries(entries).len();
            return Some(Value::Number(size as i64));
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
            return Some(Self::new_builtin_placeholder_function());
        }
        if let Some((_, value)) = Self::storage_pairs_from_object_entries(entries)
            .into_iter()
            .find(|(name, _)| name == key)
        {
            return Some(Value::String(value));
        }
        Some(Value::Undefined)
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

    fn object_property_from_url_entries(entries: &ObjectValue, key: &str) -> Option<Value> {
        if Self::is_url_object(entries) && key == "constructor" {
            return Some(Value::UrlConstructor);
        }
        None
    }

    fn object_property_from_storage_document_and_url_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if let Some(value) = Self::object_property_from_url_search_params_entries(entries, key) {
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
        receiver: &Value,
        entries: &ObjectValue,
        key: &str,
    ) -> Result<Value> {
        if let Some(value) = self.object_property_from_entries_with_getter(receiver, entries, key)? {
            return Ok(value);
        }
        let mut prototype = Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY);
        while let Some(Value::Object(object)) = prototype {
            let object_ref = object.borrow();
            if let Some(value) =
                self.object_property_from_entries_with_getter(receiver, &object_ref, key)?
            {
                return Ok(value);
            }
            prototype = Self::object_get_entry(&object_ref, INTERNAL_OBJECT_PROTOTYPE_KEY);
        }
        Ok(Value::Undefined)
    }

    fn function_public_property_from_entries_with_receiver(
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

    fn object_property_from_object_value(
        &mut self,
        value: &Value,
        entries: &Rc<RefCell<ObjectValue>>,
        key: &str,
    ) -> Result<Value> {
        let entries = entries.borrow();
        if let Some(value) = self.object_property_from_attr_or_class_list_entries(&entries, key) {
            return Ok(value);
        }
        if let Some(value) = self.object_property_from_web_api_entries(&entries, key)? {
            return Ok(value);
        }
        if let Some(value) = self
            .object_property_from_match_media_named_node_map_or_string_wrapper_entries(
                &entries, key,
            )
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
        self.object_property_from_entries_via_prototype_chain(value, &entries, key)
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
        let own_value = self.function_own_property_value(function, key, true);
        if !matches!(own_value, Value::Undefined) {
            return Ok(own_value);
        }
        if let Some(inherited) =
            self.inherited_property_from_function_super_constructor(function, key, None)?
        {
            return Ok(inherited);
        }
        Ok(Value::Undefined)
    }

    fn object_property_from_object_value_with_receiver(
        &mut self,
        entries: &Rc<RefCell<ObjectValue>>,
        key: &str,
        receiver: &Value,
    ) -> Result<Value> {
        let entries = entries.borrow();
        self.object_property_from_entries_via_prototype_chain(receiver, &entries, key)
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
        let own_value = self.function_own_property_value(function, key, false);
        if !matches!(own_value, Value::Undefined) {
            return Ok(own_value);
        }
        if let Some(inherited) = self.inherited_property_from_function_super_constructor(
            function,
            key,
            Some(receiver),
        )? {
            return Ok(inherited);
        }
        Ok(Value::Undefined)
    }

    pub(crate) fn object_property_from_value(&mut self, value: &Value, key: &str) -> Result<Value> {
        match value {
            Value::Node(node) => self.object_property_from_node_value(node, key),
            Value::String(text) => Ok(Self::object_property_from_string_value(text, key)),
            Value::Array(values) => Ok(Self::object_property_from_array_value(values, key)),
            Value::NodeList(nodes) => Ok(self.object_property_from_node_list_value(nodes, key)),
            Value::TypedArray(values) => self.object_property_from_typed_array_value(values, key),
            Value::Object(entries) => self.object_property_from_object_value(value, entries, key),
            Value::Promise(promise) => Ok(Self::object_property_from_promise_value(promise, key)),
            Value::Map(map) => Ok(self.object_property_from_map_value(map, key)),
            Value::WeakMap(weak_map) => Ok(self.object_property_from_weak_map_value(weak_map, key)),
            Value::WeakSet(weak_set) => Ok(self.object_property_from_weak_set_value(weak_set, key)),
            Value::Set(set) => Ok(self.object_property_from_set_value(set, key)),
            Value::Blob(blob) => Ok(Self::object_property_from_blob_value(blob, key)),
            Value::ArrayBuffer(_) => Ok(Self::object_property_from_array_buffer_value(key)),
            Value::Symbol(symbol) => Ok(Self::object_property_from_symbol_value(symbol, key)),
            Value::RegExp(regex) => Ok(Self::object_property_from_regexp_value(regex, key)),
            Value::Function(function) => {
                self.object_property_from_function_value(value, function, key)
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
            Value::StringConstructor => {
                let value = match key {
                    "fromCharCode" => Self::new_string_static_from_char_code_callable(),
                    "fromCodePoint" => Self::new_string_static_from_code_point_callable(),
                    "raw" => Self::new_string_static_raw_callable(),
                    "call" => Self::new_function_call_callable(),
                    "apply" => Self::new_function_apply_callable(),
                    "bind" => Self::new_function_bind_callable(),
                    "toString" => Self::new_function_to_string_callable(),
                    _ => Value::Undefined,
                };
                Ok(value)
            }
            _ => Err(Error::ScriptRuntime("value is not an object".into())),
        }
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
