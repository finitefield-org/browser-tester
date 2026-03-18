use super::*;

impl Harness {
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

    pub(crate) fn object_property_from_attr_or_class_list_entries(
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

    pub(crate) fn object_property_from_web_api_entries(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Result<Option<Value>> {
        if Self::is_event_target_object(entries)
            && let Some(value) =
                Self::placeholder_backed_object_builtin_property_value(entries, key)
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

    pub(crate) fn object_property_from_match_media_named_node_map_or_string_wrapper_entries(
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
}
