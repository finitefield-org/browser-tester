use super::*;
use std::collections::HashSet;

impl Harness {
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
