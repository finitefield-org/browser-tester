use super::*;

impl Harness {
    pub(crate) fn object_get_own_property_names_value(&mut self, object: &Value) -> Result<Value> {
        Ok(Self::new_array_value(
            self.object_like_own_string_keys(object)?
                .into_iter()
                .map(Value::String)
                .collect(),
        ))
    }

    pub(crate) fn reflect_own_keys_value(&mut self, object: &Value) -> Result<Value> {
        let mut keys = self
            .object_like_own_string_keys(object)?
            .into_iter()
            .map(Value::String)
            .collect::<Vec<_>>();
        keys.extend(self.object_like_own_symbol_values(object)?);
        Ok(Self::new_array_value(keys))
    }

    pub(crate) fn object_get_own_property_descriptor_value(
        &mut self,
        object: &Value,
        key: &str,
    ) -> Result<Value> {
        match object {
            Value::Object(entries) => Ok({
                let entries = entries.borrow();
                Self::string_wrapper_builtin_descriptor_value(&entries, key)
                    .or_else(|| self.class_list_synthesized_descriptor_value(&entries, key))
                    .or_else(|| self.named_node_map_synthesized_descriptor_value(&entries, key))
                    .or_else(|| {
                        self.placeholder_backed_object_builtin_descriptor_value(&entries, key)
                    })
                    .or_else(|| Self::own_property_descriptor_object_from_entries(&*entries, key))
                    .or_else(|| self.dom_string_map_synthesized_descriptor_value(&entries, key))
                    .or_else(|| {
                        self.callable_object_surface_descriptor_value(object, &entries, key)
                    })
                    .unwrap_or(Value::Undefined)
            }),
            Value::Array(array) => Ok(self
                .array_own_property_descriptor_value(array, key)
                .unwrap_or(Value::Undefined)),
            Value::Node(node) => Ok(self
                .node_own_property_descriptor_value(*node, key)?
                .unwrap_or(Value::Undefined)),
            Value::NodeList(nodes) => {
                let own = {
                    let nodes_ref = nodes.borrow();
                    Self::own_property_descriptor_object_from_entries(&nodes_ref.properties, key)
                };
                Ok(own
                    .or_else(|| self.node_list_synthesized_descriptor_value(nodes, key))
                    .unwrap_or(Value::Undefined))
            }
            Value::Function(function) => Ok(self
                .function_own_property_descriptor_value(function, key)
                .unwrap_or(Value::Undefined)),
            Value::Map(map) => Ok({
                let map = map.borrow();
                self.collection_own_property_descriptor_value(
                    &map.properties,
                    (key == "size").then(|| {
                        Self::own_data_property_descriptor_with_attrs(
                            Value::Number(map.entries.len() as i64),
                            false,
                            false,
                            true,
                        )
                    }),
                    key,
                )
                .unwrap_or(Value::Undefined)
            }),
            Value::WeakMap(map) => Ok({
                let map = map.borrow();
                self.collection_own_property_descriptor_value(&map.properties, None, key)
                    .unwrap_or(Value::Undefined)
            }),
            Value::Set(set) => Ok({
                let set = set.borrow();
                self.collection_own_property_descriptor_value(
                    &set.properties,
                    (key == "size").then(|| {
                        Self::own_data_property_descriptor_with_attrs(
                            Value::Number(set.values.len() as i64),
                            false,
                            false,
                            true,
                        )
                    }),
                    key,
                )
                .unwrap_or(Value::Undefined)
            }),
            Value::WeakSet(set) => Ok({
                let set = set.borrow();
                self.collection_own_property_descriptor_value(&set.properties, None, key)
                    .unwrap_or(Value::Undefined)
            }),
            Value::RegExp(regex) => Ok({
                let regex = regex.borrow();
                self.collection_own_property_descriptor_value(
                    &regex.properties,
                    self.regexp_builtin_descriptor_value(&regex, key),
                    key,
                )
                .unwrap_or(Value::Undefined)
            }),
            _ => Err(Error::ScriptRuntime(
                "Object.getOwnPropertyDescriptor argument must be an object".into(),
            )),
        }
    }

    pub(crate) fn object_get_own_property_symbols_value(
        &mut self,
        object: &Value,
    ) -> Result<Value> {
        match self.object_like_own_symbol_values(object) {
            Ok(symbols) => Ok(Self::new_array_value(symbols)),
            Err(_) => Err(Error::ScriptRuntime(
                "Object.getOwnPropertySymbols argument must be an object".into(),
            )),
        }
    }

    pub(crate) fn object_keys_value(&mut self, object: &Value) -> Result<Value> {
        let keys = self
            .object_like_enumerable_keys(object)?
            .into_iter()
            .map(Value::String)
            .collect::<Vec<_>>();
        Ok(Self::new_array_value(keys))
    }

    pub(crate) fn object_values_value(&mut self, object: &Value) -> Result<Value> {
        let keys = self.object_like_enumerable_keys(object)?;
        let mut values = Vec::with_capacity(keys.len());
        for key in keys {
            values.push(self.object_property_from_value(object, &key)?);
        }
        Ok(Self::new_array_value(values))
    }

    pub(crate) fn object_entries_value(&mut self, object: &Value) -> Result<Value> {
        let keys = self.object_like_enumerable_keys(object)?;
        let mut values = Vec::with_capacity(keys.len());
        for key in keys {
            let value = self.object_property_from_value(object, &key)?;
            values.push(Self::new_array_value(vec![Value::String(key), value]));
        }
        Ok(Self::new_array_value(values))
    }

    pub(crate) fn object_has_own_value(&mut self, object: &Value, key: &str) -> Result<Value> {
        match object {
            Value::Object(entries) => {
                let entries = entries.borrow();
                Ok(Value::Bool(
                    Self::object_get_entry(&*entries, key).is_some()
                        || Self::has_object_accessor_property(&*entries, key)
                        || self
                            .class_list_synthesized_descriptor_value(&entries, key)
                            .is_some()
                        || self
                            .named_node_map_synthesized_descriptor_value(&entries, key)
                            .is_some()
                        || self
                            .dom_string_map_synthesized_descriptor_value(&entries, key)
                            .is_some()
                        || Self::string_wrapper_builtin_has_own_property(&entries, key)
                        || (self.callable_own_surface_value(object, key).is_some()
                            && !Self::is_builtin_object_property_deleted(&*entries, key)),
                ))
            }
            Value::Array(array) => {
                let array_ref = array.borrow();
                let has = if key == "length" {
                    true
                } else if let Ok(index) = key.parse::<usize>() {
                    index < array_ref.len() && !Self::array_index_is_hole(&array_ref, index)
                } else {
                    Self::object_get_entry(&array_ref.properties, key).is_some()
                        || Self::has_object_accessor_property(&array_ref.properties, key)
                };
                Ok(Value::Bool(has))
            }
            Value::Node(node) => {
                if self.node_has_explicit_own_property(*node, key) {
                    return Ok(Value::Bool(true));
                }
                let is_form = self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("form"));
                if !is_form {
                    let is_media = self.dom.tag_name(*node).is_some_and(|tag| {
                        tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                    });
                    if !is_media {
                        return Ok(Value::Bool(false));
                    }
                    return Ok(Value::Bool(
                        self.html_media_builtin_property_value(*node, key)?
                            .is_some(),
                    ));
                }
                Ok(Value::Bool(
                    self.html_form_builtin_property_value(*node, key)?.is_some()
                        || self.form_named_property_value(*node, key)?.is_some(),
                ))
            }
            Value::NodeList(nodes) => {
                let snapshot = self.node_list_snapshot(nodes);
                let has_own_surface = {
                    let nodes_ref = nodes.borrow();
                    Self::object_get_entry(&nodes_ref.properties, key).is_some()
                        || Self::has_object_accessor_property(&nodes_ref.properties, key)
                };
                Ok(Value::Bool(
                    key == "length"
                        || key
                            .parse::<usize>()
                            .ok()
                            .is_some_and(|index| index < snapshot.len())
                        || self
                            .html_collection_named_property_value(nodes, key)
                            .is_some()
                        || has_own_surface,
                ))
            }
            Value::Function(function) => Ok(Value::Bool(
                self.script_runtime
                    .function_public_properties
                    .get(&function.function_id)
                    .is_some_and(|entries| {
                        Self::object_get_entry(entries, key).is_some()
                            || Self::has_object_accessor_property(entries, key)
                    })
                    || self
                        .function_builtin_own_property_value(function, key)
                        .is_some()
                        && !self
                            .script_runtime
                            .function_public_properties
                            .get(&function.function_id)
                            .is_some_and(|entries| {
                                Self::is_builtin_object_property_deleted(entries, key)
                            }),
            )),
            Value::Map(map) => {
                let map = map.borrow();
                Ok(Value::Bool(
                    (key == "size"
                        && !Self::is_builtin_object_property_deleted(&map.properties, key))
                        || Self::object_get_entry(&map.properties, key).is_some()
                        || Self::has_object_accessor_property(&map.properties, key),
                ))
            }
            Value::WeakMap(map) => {
                let map = map.borrow();
                Ok(Value::Bool(
                    Self::object_get_entry(&map.properties, key).is_some()
                        || Self::has_object_accessor_property(&map.properties, key),
                ))
            }
            Value::Set(set) => {
                let set = set.borrow();
                Ok(Value::Bool(
                    (key == "size"
                        && !Self::is_builtin_object_property_deleted(&set.properties, key))
                        || Self::object_get_entry(&set.properties, key).is_some()
                        || Self::has_object_accessor_property(&set.properties, key),
                ))
            }
            Value::WeakSet(set) => {
                let set = set.borrow();
                Ok(Value::Bool(
                    Self::object_get_entry(&set.properties, key).is_some()
                        || Self::has_object_accessor_property(&set.properties, key),
                ))
            }
            Value::RegExp(regex) => {
                let regex = regex.borrow();
                Ok(Value::Bool(
                    key == "lastIndex"
                        || Self::object_get_entry(&regex.properties, key).is_some()
                        || Self::has_object_accessor_property(&regex.properties, key),
                ))
            }
            _ => Err(Error::ScriptRuntime(
                "Object.hasOwn first argument must be an object".into(),
            )),
        }
    }
}
