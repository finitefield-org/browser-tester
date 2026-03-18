use super::*;

impl Harness {
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
}
