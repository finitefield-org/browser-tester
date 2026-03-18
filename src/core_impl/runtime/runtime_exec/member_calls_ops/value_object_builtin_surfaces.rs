use super::*;

impl Harness {
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

    pub(crate) fn node_receiver_builtin_method(&self, node: NodeId, key: &str) -> Option<Value> {
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

    fn parsed_document_receiver_builtin_method(key: &str) -> Option<Value> {
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

    fn dom_parser_receiver_builtin_method(key: &str) -> Option<Value> {
        if key == "parseFromString" {
            Some(Self::new_receiver_builtin_callable("dom_parser", key))
        } else {
            None
        }
    }

    fn xml_serializer_receiver_builtin_method(key: &str) -> Option<Value> {
        if key == "serializeToString" {
            Some(Self::new_receiver_builtin_callable("xml_serializer", key))
        } else {
            None
        }
    }

    fn tree_walker_receiver_builtin_method(key: &str) -> Option<Value> {
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
        if matches!(
            key,
            "addEventListener" | "removeEventListener" | "dispatchEvent"
        ) {
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
}
