use super::*;
use std::collections::HashSet;

impl Harness {
    pub(crate) fn callable_object_surface_descriptor_value(
        &mut self,
        object: &Value,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if Self::callable_kind_from_value(object).is_none() {
            return None;
        }
        if Self::object_get_entry(entries, key).is_some()
            || Self::has_object_accessor_property(entries, key)
            || Self::is_builtin_object_property_deleted(entries, key)
        {
            return None;
        }
        let value = self.callable_own_surface_value(object, key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            value, false, false, true,
        ))
    }

    pub(crate) fn placeholder_backed_object_builtin_descriptor_value(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        let stored = Self::object_get_entry(entries, key)?;
        if !Self::is_builtin_placeholder_value(&stored) {
            return None;
        }
        let value = Self::placeholder_backed_object_builtin_property_value(entries, key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            value,
            Self::is_writable_object_key(entries, key),
            Self::is_enumerable_object_key(entries, key),
            Self::is_configurable_object_key(entries, key),
        ))
    }

    pub(crate) fn placeholder_backed_array_builtin_descriptor_value(
        &mut self,
        array: &ArrayValue,
        key: &str,
    ) -> Option<Value> {
        let stored = Self::object_get_entry(&array.properties, key)?;
        if !Self::is_builtin_placeholder_value(&stored) {
            return None;
        }
        let value = Self::placeholder_backed_array_builtin_property_value(array, key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            value,
            Self::is_writable_object_key(&array.properties, key),
            Self::is_enumerable_object_key(&array.properties, key),
            Self::is_configurable_object_key(&array.properties, key),
        ))
    }

    pub(crate) fn function_own_property_descriptor_value(
        &mut self,
        function: &Rc<FunctionValue>,
        key: &str,
    ) -> Option<Value> {
        if let Some(entries) = self
            .script_runtime
            .function_public_properties
            .get(&function.function_id)
        {
            if let Some(descriptor) =
                Self::own_property_descriptor_object_from_entries(entries, key)
            {
                return Some(descriptor);
            }
            if Self::is_builtin_object_property_deleted(entries, key) {
                return None;
            }
        }

        match key {
            "name" | "length" => {
                return self
                    .function_builtin_own_property_value(function, key)
                    .map(|value| {
                        Self::own_data_property_descriptor_with_attrs(value, false, false, true)
                    });
            }
            "prototype" if !function.is_arrow && !function.is_method => {
                return self
                    .function_builtin_own_property_value(function, key)
                    .map(|value| {
                        Self::own_data_property_descriptor_with_attrs(value, true, false, false)
                    });
            }
            _ => {}
        }

        None
    }

    pub(crate) fn array_own_property_descriptor_value(
        &mut self,
        array: &Rc<RefCell<ArrayValue>>,
        key: &str,
    ) -> Option<Value> {
        let array_ref = array.borrow();
        if let Some(descriptor) =
            self.placeholder_backed_array_builtin_descriptor_value(&array_ref, key)
        {
            return Some(descriptor);
        }
        if let Some(descriptor) =
            Self::own_property_descriptor_object_from_entries(&array_ref.properties, key)
        {
            return Some(descriptor);
        }
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(array_ref.len() as i64),
                Self::is_writable_object_key(&array_ref.properties, "length"),
                false,
                false,
            ));
        }
        if let Ok(index) = key.parse::<usize>() {
            if index < array_ref.len() && !Self::array_index_is_hole(&array_ref, index) {
                return Some(Self::own_data_property_descriptor_with_attrs(
                    array_ref[index].clone(),
                    Self::array_index_is_writable(&array_ref, index),
                    Self::array_index_is_enumerable(&array_ref, index),
                    Self::array_index_is_configurable(&array_ref, index),
                ));
            }
            return None;
        }
        None
    }

    pub(crate) fn collection_own_property_descriptor_value(
        &mut self,
        entries: &ObjectValue,
        builtin_descriptor: Option<Value>,
        key: &str,
    ) -> Option<Value> {
        if let Some(descriptor) = Self::own_property_descriptor_object_from_entries(entries, key) {
            return Some(descriptor);
        }
        if Self::is_builtin_object_property_deleted(entries, key) {
            return None;
        }
        builtin_descriptor
    }

    pub(crate) fn dom_string_map_synthesized_keys(
        &self,
        entries: &ObjectValue,
        enumerable_only: bool,
    ) -> Option<Vec<String>> {
        if !Self::is_dom_string_map_object(entries) {
            return None;
        }
        let node = Self::dom_string_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let mut keys = self
            .dataset_entries_for_node(node)
            .into_iter()
            .map(|(key, _)| key)
            .filter(|key| Self::own_property_descriptor_object_from_entries(entries, key).is_none())
            .collect::<Vec<_>>();
        keys.extend(if enumerable_only {
            Self::ordered_enumerable_string_keys(entries)
        } else {
            Self::ordered_visible_string_keys(entries)
        });
        Some(keys)
    }

    pub(crate) fn dom_string_map_synthesized_descriptor_value(
        &self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_dom_string_map_object(entries)
            || Self::own_property_descriptor_object_from_entries(entries, key).is_some()
        {
            return None;
        }
        let node = Self::dom_string_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let attr_name = dataset_key_to_attr_name(key);
        let value = self.dom.attr(node, &attr_name)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            Value::String(value),
            true,
            true,
            true,
        ))
    }

    pub(crate) fn class_list_synthesized_keys(
        &self,
        entries: &ObjectValue,
        enumerable_only: bool,
    ) -> Option<Vec<String>> {
        if !Self::is_class_list_object(entries) {
            return None;
        }
        let node = match Self::object_get_entry(entries, INTERNAL_CLASS_LIST_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }?;
        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
        let mut integer_keys = classes
            .iter()
            .enumerate()
            .map(|(index, _)| (index as u64, index.to_string()))
            .collect::<Vec<_>>();
        let property_keys = if enumerable_only {
            Self::ordered_enumerable_string_keys(entries)
        } else {
            Self::ordered_visible_string_keys(entries)
        };
        for key in &property_keys {
            if let Some(index) = Self::own_property_integer_key(key)
                && !integer_keys.iter().any(|(existing, _)| *existing == index)
            {
                integer_keys.push((index, key.clone()));
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = integer_keys
            .into_iter()
            .map(|(_, key)| key)
            .collect::<Vec<_>>();
        if !enumerable_only {
            out.push("length".to_string());
            out.push("value".to_string());
        }
        out.extend(property_keys.into_iter().filter(|key| {
            Self::own_property_integer_key(key).is_none()
                && (enumerable_only || (key != "length" && key != "value"))
        }));
        Some(out)
    }

    pub(crate) fn class_list_synthesized_descriptor_value(
        &self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_class_list_object(entries)
            || Self::own_property_descriptor_object_from_entries(entries, key).is_some()
        {
            return None;
        }
        let node = match Self::object_get_entry(entries, INTERNAL_CLASS_LIST_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }?;
        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(classes.len() as i64),
                true,
                false,
                true,
            ));
        }
        if key == "value" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::String(classes.join(" ")),
                true,
                false,
                true,
            ));
        }
        let index = Self::own_property_integer_key(key)? as usize;
        let class_name = classes.get(index)?.clone();
        Some(Self::own_data_property_descriptor_with_attrs(
            Value::String(class_name),
            true,
            true,
            true,
        ))
    }

    pub(crate) fn named_node_map_synthesized_keys(
        &mut self,
        entries: &ObjectValue,
        enumerable_only: bool,
    ) -> Option<Vec<String>> {
        if !Self::is_named_node_map_object(entries) {
            return None;
        }
        let node = Self::named_node_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let attrs = self.named_node_map_entries(node);
        let mut integer_keys = attrs
            .iter()
            .enumerate()
            .map(|(index, _)| (index as u64, index.to_string()))
            .collect::<Vec<_>>();
        let property_keys = if enumerable_only {
            Self::ordered_enumerable_string_keys(entries)
        } else {
            Self::ordered_visible_string_keys(entries)
        };
        for key in &property_keys {
            if let Some(index) = Self::own_property_integer_key(key)
                && !integer_keys.iter().any(|(existing, _)| *existing == index)
            {
                integer_keys.push((index, key.clone()));
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = integer_keys
            .into_iter()
            .map(|(_, key)| key)
            .collect::<Vec<_>>();
        if !enumerable_only {
            out.push("length".to_string());
        }
        out.extend(attrs.iter().map(|(name, _)| name.clone()).filter(|key| {
            !property_keys.iter().any(|existing| existing == key)
                && self.named_node_map_named_property_is_visible(entries, key)
        }));
        out.extend(property_keys.into_iter().filter(|key| {
            Self::own_property_integer_key(key).is_none() && (enumerable_only || key != "length")
        }));
        Some(out)
    }

    pub(crate) fn named_node_map_synthesized_descriptor_value(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_named_node_map_object(entries)
            || Self::own_property_descriptor_object_from_entries(entries, key).is_some()
        {
            return None;
        }
        let node = Self::named_node_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let attrs = self.named_node_map_entries(node);
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(attrs.len() as i64),
                true,
                false,
                true,
            ));
        }
        if let Some(index) = Self::own_property_integer_key(key) {
            let (name, value) = attrs.get(index as usize)?;
            return Some(Self::own_data_property_descriptor_with_attrs(
                Self::new_attr_object_value(name, value, Some(node)),
                true,
                true,
                true,
            ));
        }
        if !self.named_node_map_named_property_is_visible(entries, key) {
            return None;
        }
        let (name, value) = attrs.iter().find(|(name, _)| name == key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            Self::new_attr_object_value(name, value, Some(node)),
            true,
            true,
            true,
        ))
    }

    pub(crate) fn node_list_synthesized_keys(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        enumerable_only: bool,
    ) -> Vec<String> {
        let snapshot = self.node_list_snapshot(nodes);
        let mut integer_keys = snapshot
            .iter()
            .enumerate()
            .map(|(index, _)| (index as u64, index.to_string()))
            .collect::<Vec<_>>();
        let property_keys = {
            let nodes_ref = nodes.borrow();
            if enumerable_only {
                Self::ordered_enumerable_string_keys(&nodes_ref.properties)
            } else {
                Self::ordered_visible_string_keys(&nodes_ref.properties)
            }
        };
        for key in &property_keys {
            if let Some(index) = Self::own_property_integer_key(key)
                && !integer_keys.iter().any(|(existing, _)| *existing == index)
            {
                integer_keys.push((index, key.clone()));
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = integer_keys
            .into_iter()
            .map(|(_, key)| key)
            .collect::<Vec<_>>();
        let named_keys = self
            .html_collection_named_entries(nodes)
            .into_iter()
            .map(|(name, _)| name)
            .filter(|key| {
                !property_keys.iter().any(|existing| existing == key)
                    && self.html_collection_named_property_is_visible(nodes, key)
            })
            .collect::<Vec<_>>();
        if !enumerable_only {
            out.push("length".to_string());
        }
        out.extend(named_keys);
        out.extend(property_keys.into_iter().filter(|key| {
            Self::own_property_integer_key(key).is_none() && (enumerable_only || key != "length")
        }));
        out
    }

    pub(crate) fn node_list_synthesized_descriptor_value(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        key: &str,
    ) -> Option<Value> {
        {
            let nodes_ref = nodes.borrow();
            if Self::own_property_descriptor_object_from_entries(&nodes_ref.properties, key)
                .is_some()
            {
                return None;
            }
        }
        let snapshot = self.node_list_snapshot(nodes);
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(snapshot.len() as i64),
                false,
                false,
                true,
            ));
        }
        if let Some(index) = Self::own_property_integer_key(key) {
            if let Some(node) = snapshot.get(index as usize).copied() {
                return Some(Self::own_data_property_descriptor_with_attrs(
                    self.node_list_item_value(nodes, node),
                    false,
                    true,
                    true,
                ));
            }
        }
        self.html_collection_named_property_value(nodes, key)
            .map(|value| Self::own_data_property_descriptor_with_attrs(value, false, true, true))
    }

    pub(crate) fn node_expando_entries(&self, node: NodeId) -> Vec<(String, Value)> {
        let mut entries = self
            .dom_runtime
            .node_expando_props
            .iter()
            .filter(|((owner, _), _)| *owner == node)
            .map(|((_, key), value)| (key.clone(), value.clone()))
            .collect::<Vec<_>>();
        entries.sort_by(|(left, _), (right, _)| left.cmp(right));
        entries
    }

    pub(crate) fn replace_node_expando_entries(
        &mut self,
        node: NodeId,
        entries: Vec<(String, Value)>,
    ) {
        self.dom_runtime
            .node_expando_props
            .retain(|(owner, _), _| *owner != node);
        for (key, value) in entries {
            self.dom_runtime
                .node_expando_props
                .insert((node, key), value);
        }
    }

    pub(crate) fn node_has_explicit_own_property(&self, node: NodeId, key: &str) -> bool {
        let entries = self.node_expando_entries(node);
        Self::object_get_entry(&entries, key).is_some()
            || Self::has_object_accessor_property(&entries, key)
    }

    pub(crate) fn node_expando_enumerable_string_keys(&self, node: NodeId) -> Vec<String> {
        let entries = ObjectValue::new(self.node_expando_entries(node));
        Self::ordered_enumerable_string_keys(&entries)
    }

    pub(crate) fn node_expando_string_keys(&self, node: NodeId) -> Vec<String> {
        let entries = ObjectValue::new(self.node_expando_entries(node));
        Self::ordered_visible_string_keys(&entries)
    }

    pub(crate) fn node_expando_enumerable_symbol_values(&self, node: NodeId) -> Vec<Value> {
        let entries = self.node_expando_entries(node);
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for (key, _) in &entries {
            if !Self::is_symbol_storage_key(key) || !Self::is_enumerable_object_key(&entries, key) {
                continue;
            }
            if let Some(symbol_id) = Self::symbol_id_from_storage_key(key) {
                if !seen.insert(symbol_id) {
                    continue;
                }
                if let Some(symbol) = self.symbol_runtime.symbols_by_id.get(&symbol_id) {
                    out.push(Value::Symbol(symbol.clone()));
                }
            }
        }
        out
    }

    pub(crate) fn node_expando_symbol_values(&self, node: NodeId) -> Vec<Value> {
        let entries = self.node_expando_entries(node);
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for (key, _) in &entries {
            if let Some(symbol_id) = Self::symbol_id_from_storage_key(key) {
                if !seen.insert(symbol_id) {
                    continue;
                }
                if let Some(symbol) = self.symbol_runtime.symbols_by_id.get(&symbol_id) {
                    out.push(Value::Symbol(symbol.clone()));
                }
            }
        }
        out
    }

    pub(crate) fn html_form_own_string_keys(&mut self, form: NodeId) -> Result<Vec<String>> {
        let expando_keys = self.node_expando_string_keys(form);
        let expando_set = expando_keys.iter().cloned().collect::<HashSet<_>>();
        let mut out = expando_keys;

        for key in Self::html_form_builtin_own_string_keys() {
            if !expando_set.contains(key) {
                out.push(key.to_string());
            }
        }

        for key in self.html_form_named_property_keys(form)? {
            if !expando_set.contains(&key) && !out.iter().any(|existing| existing == &key) {
                out.push(key);
            }
        }

        Ok(out)
    }

    pub(crate) fn html_media_own_string_keys(&mut self, media: NodeId) -> Vec<String> {
        let expando_keys = self.node_expando_string_keys(media);
        let expando_set = expando_keys.iter().cloned().collect::<HashSet<_>>();
        let mut out = expando_keys;

        for key in Self::html_media_builtin_own_string_keys() {
            if !expando_set.contains(key) {
                out.push(key.to_string());
            }
        }

        out
    }

    pub(crate) fn node_own_property_descriptor_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Option<Value>> {
        let expando_entries = self.node_expando_entries(node);
        if let Some(descriptor) =
            Self::own_property_descriptor_object_from_entries(&expando_entries, key)
        {
            return Ok(Some(descriptor));
        }

        let is_form = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("form"));
        if is_form {
            if let Some(value) = self.html_form_builtin_property_value(node, key)? {
                return Ok(Some(Self::own_data_property_descriptor_with_attrs(
                    value, false, false, true,
                )));
            }

            return Ok(self.form_named_property_value(node, key)?.map(|value| {
                Self::own_data_property_descriptor_with_attrs(value, false, false, true)
            }));
        }

        let is_media = self.dom.tag_name(node).is_some_and(|tag| {
            tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
        });
        if !is_media {
            return Ok(None);
        }

        if let Some(value) = self.html_media_builtin_property_value(node, key)? {
            return Ok(Some(Self::own_data_property_descriptor_with_attrs(
                value, false, false, true,
            )));
        }

        Ok(None)
    }

    pub(crate) fn object_like_enumerable_keys(&mut self, object: &Value) -> Result<Vec<String>> {
        match object {
            Value::Object(entries) => {
                let entries = entries.borrow();
                Ok(Self::string_wrapper_own_string_keys(&entries, true)
                    .or_else(|| self.class_list_synthesized_keys(&entries, true))
                    .or_else(|| self.named_node_map_synthesized_keys(&entries, true))
                    .or_else(|| self.dom_string_map_synthesized_keys(&entries, true))
                    .unwrap_or_else(|| Self::ordered_enumerable_string_keys(&entries)))
            }
            Value::Array(array) => {
                let array_ref = array.borrow();
                let mut keys = array_ref
                    .iter()
                    .enumerate()
                    .filter_map(|(index, _)| {
                        (!Self::array_index_is_hole(&array_ref, index)
                            && Self::array_index_is_enumerable(&array_ref, index))
                        .then(|| index.to_string())
                    })
                    .collect::<Vec<_>>();
                keys.extend(Self::ordered_enumerable_string_keys(&array_ref.properties));
                Ok(keys)
            }
            Value::Node(node) => Ok(self.node_expando_enumerable_string_keys(*node)),
            Value::NodeList(nodes) => Ok(self.node_list_synthesized_keys(nodes, true)),
            Value::Function(function) => Ok(self
                .script_runtime
                .function_public_properties
                .get(&function.function_id)
                .map(Self::ordered_enumerable_string_keys)
                .unwrap_or_default()),
            Value::Map(map) => Ok(Self::ordered_enumerable_string_keys(
                &map.borrow().properties,
            )),
            Value::WeakMap(map) => Ok(Self::ordered_enumerable_string_keys(
                &map.borrow().properties,
            )),
            Value::Set(set) => Ok(Self::ordered_enumerable_string_keys(
                &set.borrow().properties,
            )),
            Value::WeakSet(set) => Ok(Self::ordered_enumerable_string_keys(
                &set.borrow().properties,
            )),
            Value::RegExp(regex) => Ok(Self::ordered_enumerable_string_keys(
                &regex.borrow().properties,
            )),
            _ => Err(Error::ScriptRuntime(
                "Object.keys argument must be an object".into(),
            )),
        }
    }

    pub(crate) fn object_like_own_string_keys(&mut self, object: &Value) -> Result<Vec<String>> {
        match object {
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(keys) = Self::string_wrapper_own_string_keys(&entries, false) {
                    return Ok(keys);
                }
                if let Some(keys) = self.class_list_synthesized_keys(&entries, false) {
                    return Ok(keys);
                }
                if let Some(keys) = self.named_node_map_synthesized_keys(&entries, false) {
                    return Ok(keys);
                }
                if let Some(keys) = self.dom_string_map_synthesized_keys(&entries, false) {
                    return Ok(keys);
                }
                let (integer_keys, string_keys) = Self::ordered_visible_string_keys_split(&entries);
                Ok(if Self::callable_kind_from_value(object).is_some() {
                    let builtin_keys =
                        Self::visible_builtin_string_keys(&entries, ["length", "name"]);
                    Self::merge_builtin_string_keys(integer_keys, string_keys, &builtin_keys)
                } else {
                    Self::ordered_visible_string_keys(&entries)
                })
            }
            Value::Node(node) => {
                let is_form = self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("form"));
                if is_form {
                    self.html_form_own_string_keys(*node)
                } else if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) {
                    Ok(self.html_media_own_string_keys(*node))
                } else {
                    Ok(self.node_expando_string_keys(*node))
                }
            }
            Value::Array(array) => {
                let array_ref = array.borrow();
                let mut integer_keys: Vec<(u64, String)> = array_ref
                    .iter()
                    .enumerate()
                    .filter_map(|(index, _)| {
                        (!Self::array_index_is_hole(&array_ref, index))
                            .then(|| (index as u64, index.to_string()))
                    })
                    .collect();
                let (property_integer_keys, property_string_keys) =
                    Self::ordered_visible_string_keys_split(&array_ref.properties);
                for key in property_integer_keys {
                    if let Some(index) = Self::own_property_integer_key(&key) {
                        if !integer_keys.iter().any(|(existing, _)| *existing == index) {
                            integer_keys.push((index, key));
                        }
                    }
                }
                integer_keys.sort_by_key(|(index, _)| *index);
                let mut out = integer_keys
                    .into_iter()
                    .map(|(_, key)| key)
                    .collect::<Vec<_>>();
                out.push("length".to_string());
                out.extend(
                    property_string_keys
                        .into_iter()
                        .filter(|key| key != "length"),
                );
                Ok(out)
            }
            Value::NodeList(nodes) => Ok(self.node_list_synthesized_keys(nodes, false)),
            Value::Function(function) => {
                let (integer_keys, string_keys, builtin_keys) = self
                    .script_runtime
                    .function_public_properties
                    .get(&function.function_id)
                    .map(|entries| {
                        let (integer_keys, string_keys) =
                            Self::ordered_visible_string_keys_split(entries);
                        (
                            integer_keys,
                            string_keys,
                            Self::visible_builtin_string_keys(
                                entries,
                                Self::function_builtin_own_string_keys(function),
                            ),
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            Vec::new(),
                            Vec::new(),
                            Self::function_builtin_own_string_keys(function),
                        )
                    });
                Ok(Self::merge_builtin_string_keys(
                    integer_keys,
                    string_keys,
                    &builtin_keys,
                ))
            }
            Value::Map(map) => {
                let map = map.borrow();
                let (integer_keys, string_keys) =
                    Self::ordered_visible_string_keys_split(&map.properties);
                let builtin_keys = Self::visible_builtin_string_keys(&map.properties, ["size"]);
                Ok(Self::merge_builtin_string_keys(
                    integer_keys,
                    string_keys,
                    &builtin_keys,
                ))
            }
            Value::WeakMap(map) => Ok(Self::ordered_visible_string_keys(&map.borrow().properties)),
            Value::Set(set) => {
                let set = set.borrow();
                let (integer_keys, string_keys) =
                    Self::ordered_visible_string_keys_split(&set.properties);
                let builtin_keys = Self::visible_builtin_string_keys(&set.properties, ["size"]);
                Ok(Self::merge_builtin_string_keys(
                    integer_keys,
                    string_keys,
                    &builtin_keys,
                ))
            }
            Value::WeakSet(set) => Ok(Self::ordered_visible_string_keys(&set.borrow().properties)),
            Value::RegExp(regex) => {
                let regex = regex.borrow();
                let (integer_keys, string_keys) =
                    Self::ordered_visible_string_keys_split(&regex.properties);
                let builtin_keys = Self::visible_builtin_string_keys(
                    &regex.properties,
                    Self::regexp_builtin_own_string_keys(),
                );
                Ok(Self::merge_builtin_string_keys(
                    integer_keys,
                    string_keys,
                    &builtin_keys,
                ))
            }
            _ => Err(Error::ScriptRuntime(
                "Object.getOwnPropertyNames argument must be an object".into(),
            )),
        }
    }

    pub(crate) fn object_like_own_symbol_values(&self, object: &Value) -> Result<Vec<Value>> {
        match object {
            Value::Object(entries) => Ok(self.collection_property_symbol_values(&entries.borrow())),
            Value::Array(array) => {
                Ok(self.collection_property_symbol_values(&array.borrow().properties))
            }
            Value::Node(node) => Ok(self.node_expando_symbol_values(*node)),
            Value::NodeList(nodes) => {
                Ok(self.collection_property_symbol_values(&nodes.borrow().properties))
            }
            Value::Function(function) => Ok(self
                .script_runtime
                .function_public_properties
                .get(&function.function_id)
                .map(|entries| self.collection_property_symbol_values(entries))
                .unwrap_or_default()),
            Value::Map(map) => Ok(self.collection_property_symbol_values(&map.borrow().properties)),
            Value::WeakMap(map) => {
                Ok(self.collection_property_symbol_values(&map.borrow().properties))
            }
            Value::Set(set) => Ok(self.collection_property_symbol_values(&set.borrow().properties)),
            Value::WeakSet(set) => {
                Ok(self.collection_property_symbol_values(&set.borrow().properties))
            }
            Value::RegExp(regex) => {
                Ok(self.collection_property_symbol_values(&regex.borrow().properties))
            }
            _ => Err(Error::ScriptRuntime(
                "Reflect.ownKeys target must be an object".into(),
            )),
        }
    }
}
