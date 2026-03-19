use super::json_object_array_descriptors::NormalizedOwnPropertyDescriptor;
use super::*;

impl Harness {
    fn define_property_on_object_entries(
        &mut self,
        entries: &mut ObjectValue,
        key: &str,
        descriptor: &Value,
    ) -> Result<()> {
        let current_descriptor = Self::own_property_descriptor_object_from_entries(entries, key);
        let normalized =
            self.normalize_property_descriptor(current_descriptor.as_ref(), key, descriptor)?;
        Self::apply_normalized_descriptor_to_object_entries(entries, key, normalized);
        Ok(())
    }

    pub(crate) fn object_define_property_value(
        &mut self,
        object: &Value,
        key: &str,
        descriptor: &Value,
    ) -> Result<Value> {
        if !Self::descriptor_is_object_like_value(descriptor) {
            return Err(Error::ScriptRuntime(
                "Object.defineProperty descriptor must be an object".into(),
            ));
        }

        match object {
            Value::Object(entries) => {
                let current_descriptor = {
                    let entries_ref = entries.borrow();
                    Self::string_wrapper_builtin_descriptor_value(&entries_ref, key)
                };
                if let Some(current_descriptor) = current_descriptor {
                    self.normalize_property_descriptor(Some(&current_descriptor), key, descriptor)?;
                    Self::delete_object_property_entries(&mut entries.borrow_mut(), key);
                    return Ok(object.clone());
                }
                self.define_property_on_object_entries(&mut entries.borrow_mut(), key, descriptor)?;
                Ok(object.clone())
            }
            Value::Array(array) => {
                let current_descriptor = self.array_own_property_descriptor_value(array, key);
                if key == "length" {
                    let normalized = self.normalize_property_descriptor(
                        current_descriptor.as_ref(),
                        key,
                        descriptor,
                    )?;
                    let NormalizedOwnPropertyDescriptor::Data {
                        value, writable, ..
                    } = normalized
                    else {
                        return Err(Self::redefine_property_error(key));
                    };
                    let mut values = array.borrow_mut();
                    Self::delete_object_property_entries(&mut values.properties, key);
                    let next = Self::value_to_i64(&value);
                    let next = if next <= 0 { 0usize } else { next as usize };
                    if next < values.len() {
                        values.truncate(next);
                    } else if next > values.len() {
                        values.resize(next, Value::Undefined);
                    }
                    Self::set_object_property_flags(
                        &mut values.properties,
                        key,
                        writable,
                        false,
                        false,
                    );
                    return Ok(object.clone());
                }
                if let Ok(index) = key.parse::<usize>() {
                    if !self.descriptor_is_accessor_descriptor(descriptor)? {
                        let normalized = self.normalize_property_descriptor(
                            current_descriptor.as_ref(),
                            key,
                            descriptor,
                        )?;
                        let NormalizedOwnPropertyDescriptor::Data {
                            value,
                            writable,
                            enumerable,
                            configurable,
                        } = normalized
                        else {
                            unreachable!();
                        };
                        let mut values = array.borrow_mut();
                        Self::delete_object_property_entries(&mut values.properties, key);
                        if index >= values.len() {
                            values.resize(index + 1, Value::Undefined);
                        }
                        values[index] = value;
                        Self::set_object_property_flags(
                            &mut values.properties,
                            key,
                            writable,
                            enumerable,
                            configurable,
                        );
                        drop(values);
                        Self::clear_array_hole(array, index);
                        return Ok(object.clone());
                    }
                    if current_descriptor.is_some() {
                        self.normalize_property_descriptor(
                            current_descriptor.as_ref(),
                            key,
                            descriptor,
                        )?;
                    }
                }
                self.define_property_on_object_entries(
                    &mut array.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            Value::Function(function) => {
                if Self::is_function_builtin_prototype_key(function, key) {
                    let current_descriptor = self
                        .function_own_property_descriptor_value(function, key)
                        .unwrap_or_else(|| {
                            Self::own_data_property_descriptor_with_attrs(
                                Value::Object(function.prototype_object.clone()),
                                true,
                                false,
                                false,
                            )
                        });
                    let normalized = self.normalize_property_descriptor(
                        Some(&current_descriptor),
                        key,
                        descriptor,
                    )?;
                    let NormalizedOwnPropertyDescriptor::Data {
                        value, writable, ..
                    } = normalized
                    else {
                        return Err(Self::redefine_property_error(key));
                    };

                    let entries = self
                        .script_runtime
                        .function_public_properties
                        .entry(function.function_id)
                        .or_default();
                    Self::set_function_builtin_prototype_property(entries, value, writable);
                    return Ok(object.clone());
                }

                let mut entries = self
                    .script_runtime
                    .function_public_properties
                    .remove(&function.function_id)
                    .unwrap_or_default();
                self.define_property_on_object_entries(&mut entries, key, descriptor)?;
                self.script_runtime
                    .function_public_properties
                    .insert(function.function_id, entries);
                Ok(object.clone())
            }
            Value::Node(node) => {
                let mut entries = ObjectValue::new(self.node_expando_entries(*node));
                self.define_property_on_object_entries(&mut entries, key, descriptor)?;
                self.replace_node_expando_entries(*node, entries.entries);
                Ok(object.clone())
            }
            Value::NodeList(nodes) => {
                let current_descriptor = {
                    let nodes_ref = nodes.borrow();
                    Self::own_property_descriptor_object_from_entries(&nodes_ref.properties, key)
                }
                .or_else(|| self.node_list_synthesized_descriptor_value(nodes, key));
                let normalized = self.normalize_property_descriptor(
                    current_descriptor.as_ref(),
                    key,
                    descriptor,
                )?;
                Self::apply_normalized_descriptor_to_object_entries(
                    &mut nodes.borrow_mut().properties,
                    key,
                    normalized,
                );
                Ok(object.clone())
            }
            Value::Map(map) => {
                self.define_property_on_object_entries(
                    &mut map.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            Value::WeakMap(map) => {
                self.define_property_on_object_entries(
                    &mut map.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            Value::Set(set) => {
                self.define_property_on_object_entries(
                    &mut set.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            Value::WeakSet(set) => {
                self.define_property_on_object_entries(
                    &mut set.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            Value::RegExp(regex) => {
                if key == "lastIndex" {
                    let current_descriptor = {
                        let regex_ref = regex.borrow();
                        self.regexp_builtin_descriptor_value(&regex_ref, key)
                            .unwrap_or_else(|| {
                                Self::own_data_property_descriptor_with_attrs(
                                    Value::Number(regex_ref.last_index as i64),
                                    true,
                                    false,
                                    false,
                                )
                            })
                    };
                    let normalized = self.normalize_property_descriptor(
                        Some(&current_descriptor),
                        key,
                        descriptor,
                    )?;
                    let NormalizedOwnPropertyDescriptor::Data {
                        value, writable, ..
                    } = normalized
                    else {
                        return Err(Self::redefine_property_error(key));
                    };
                    let mut regex_ref = regex.borrow_mut();
                    Self::delete_object_property_entries(&mut regex_ref.properties, key);
                    let next = Self::value_to_i64(&value);
                    regex_ref.last_index = if next <= 0 { 0 } else { next as usize };
                    Self::set_object_property_flags(
                        &mut regex_ref.properties,
                        key,
                        writable,
                        false,
                        false,
                    );
                    return Ok(object.clone());
                }
                self.define_property_on_object_entries(
                    &mut regex.borrow_mut().properties,
                    key,
                    descriptor,
                )?;
                Ok(object.clone())
            }
            _ => Err(Error::ScriptRuntime(
                "Object.defineProperty target must be an object".into(),
            )),
        }
    }
}
