use super::*;

impl Harness {
    pub(crate) fn loose_equal(&self, left: &Value, right: &Value) -> bool {
        if self.strict_equal(left, right) {
            return true;
        }

        match (left, right) {
            (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null) => true,
            (Value::BigInt(l), Value::String(r)) => {
                Self::parse_js_bigint_from_string(r).is_ok_and(|parsed| parsed == *l)
            }
            (Value::String(l), Value::BigInt(r)) => {
                Self::parse_js_bigint_from_string(l).is_ok_and(|parsed| parsed == *r)
            }
            (Value::BigInt(_), Value::Number(_) | Value::Float(_))
            | (Value::Number(_) | Value::Float(_), Value::BigInt(_)) => {
                Self::number_bigint_loose_equal(left, right)
            }
            (Value::Number(_) | Value::Float(_), Value::String(_))
            | (Value::String(_), Value::Number(_) | Value::Float(_)) => {
                Self::coerce_number_for_global(left) == Self::coerce_number_for_global(right)
            }
            (Value::Bool(_), _) => {
                let coerced = Value::Float(Self::coerce_number_for_global(left));
                self.loose_equal(&coerced, right)
            }
            (_, Value::Bool(_)) => {
                let coerced = Value::Float(Self::coerce_number_for_global(right));
                self.loose_equal(left, &coerced)
            }
            _ if Self::is_loose_primitive(left) && Self::is_loose_object(right) => {
                let prim = self.to_primitive_for_loose(right);
                self.loose_equal(left, &prim)
            }
            _ if Self::is_loose_object(left) && Self::is_loose_primitive(right) => {
                let prim = self.to_primitive_for_loose(left);
                self.loose_equal(&prim, right)
            }
            _ => false,
        }
    }

    pub(crate) fn is_loose_primitive(value: &Value) -> bool {
        matches!(
            value,
            Value::String(_)
                | Value::Bool(_)
                | Value::Number(_)
                | Value::Float(_)
                | Value::BigInt(_)
                | Value::Symbol(_)
                | Value::Null
                | Value::Undefined
        )
    }

    pub(crate) fn is_loose_object(value: &Value) -> bool {
        matches!(
            value,
            Value::Array(_)
                | Value::Object(_)
                | Value::Promise(_)
                | Value::Map(_)
                | Value::WeakMap(_)
                | Value::Set(_)
                | Value::WeakSet(_)
                | Value::Blob(_)
                | Value::ArrayBuffer(_)
                | Value::TypedArray(_)
                | Value::StringConstructor
                | Value::TypedArrayConstructor(_)
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
                | Value::PromiseCapability(_)
                | Value::RegExp(_)
                | Value::Date(_)
                | Value::Node(_)
                | Value::NodeList(_)
                | Value::FormData(_)
                | Value::Function(_)
        )
    }

    pub(crate) fn to_primitive_for_loose(&self, value: &Value) -> Value {
        match value {
            Value::Object(entries) => {
                if let Some(wrapped) = Self::string_wrapper_value_from_object(&entries.borrow()) {
                    return Value::String(wrapped);
                }
                if let Some(id) = Self::symbol_wrapper_id_from_object(&entries.borrow()) {
                    if let Some(symbol) = self.symbol_runtime.symbols_by_id.get(&id) {
                        return Value::Symbol(symbol.clone());
                    }
                }
                Value::String(value.as_string())
            }
            Value::Array(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::WeakMap(_)
            | Value::Set(_)
            | Value::WeakSet(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::WeakMapConstructor
            | Value::SetConstructor
            | Value::WeakSetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::RegExp(_)
            | Value::Date(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_) => Value::String(value.as_string()),
            _ => value.clone(),
        }
    }

    fn object_has_property_in_chain(
        &mut self,
        value: &Value,
        entries: &ObjectValue,
        key: &str,
    ) -> bool {
        if Self::object_get_entry(entries, key).is_some()
            || Self::has_object_accessor_property(entries, key)
            || self.object_synthesized_own_property_exists(entries, key)
            || Self::string_wrapper_builtin_has_own_property(entries, key)
            || (self.callable_own_surface_value(value, key).is_some()
                && !Self::is_builtin_object_property_deleted(entries, key))
        {
            return true;
        }

        let mut prototype = Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
            .or_else(|| self.value_internal_prototype_value(value));
        let mut hops = 0usize;
        while let Some(current) = prototype {
            if matches!(current, Value::Null | Value::Undefined) {
                break;
            }
            match &current {
                Value::Object(object) => {
                    let object_value = Value::Object(object.clone());
                    let object_ref = object.borrow();
                    if Self::object_get_entry(&object_ref, key).is_some()
                        || Self::has_object_accessor_property(&object_ref, key)
                        || self.object_synthesized_own_property_exists(&object_ref, key)
                        || Self::string_wrapper_builtin_has_own_property(&object_ref, key)
                        || (self
                            .callable_own_surface_value(&object_value, key)
                            .is_some()
                            && !Self::is_builtin_object_property_deleted(&object_ref, key))
                    {
                        return true;
                    }
                }
                _ => {
                    if self
                        .object_property_from_value(&current, key)
                        .is_ok_and(|value| !matches!(value, Value::Undefined))
                    {
                        return true;
                    }
                }
            }
            hops += 1;
            if hops > 256 {
                break;
            }
            prototype = self.value_internal_prototype_value(&current);
        }
        false
    }

    pub(crate) fn value_in(&mut self, left: &Value, right: &Value) -> Result<bool> {
        if Self::is_primitive_value(right) {
            return Err(Error::ScriptRuntime(
                "right-hand side of in must be an object".into(),
            ));
        }

        let key = self.property_key_to_storage_key(left);
        let has_property = match right {
            Value::NodeList(nodes) => {
                let has_own_surface = {
                    let nodes_ref = nodes.borrow();
                    Self::object_get_entry(&nodes_ref.properties, &key).is_some()
                        || Self::has_object_accessor_property(&nodes_ref.properties, &key)
                };
                if key == "length" || has_own_surface {
                    true
                } else {
                    let has_own_index = self
                        .value_as_index(left)
                        .is_some_and(|index| index < self.node_list_len(nodes));
                    let has_named_property = self
                        .html_collection_named_property_value(nodes, &key)
                        .is_some();
                    has_own_index
                        || has_named_property
                        || self.object_has_property_in_chain(
                            right,
                            &nodes.borrow().properties,
                            &key,
                        )
                }
            }
            Value::Array(values) => {
                let values = values.borrow();
                if key == "length" || Self::object_get_entry(&values.properties, &key).is_some() {
                    true
                } else {
                    let has_own_index = self.value_as_index(left).is_some_and(|index| {
                        index < values.len() && !Self::array_index_is_hole(&values, index)
                    });
                    has_own_index
                        || Self::object_get_entry(&values.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                            .is_some_and(|_| {
                                self.object_has_property_in_chain(right, &values.properties, &key)
                            })
                }
            }
            Value::TypedArray(values) => {
                if key == "length" {
                    true
                } else {
                    let has_own_index = self
                        .value_as_index(left)
                        .is_some_and(|index| index < values.borrow().observed_length());
                    has_own_index
                        || Self::object_get_entry(
                            &values.borrow().properties,
                            INTERNAL_OBJECT_PROTOTYPE_KEY,
                        )
                        .is_some_and(|_| {
                            self.object_has_property_in_chain(
                                right,
                                &values.borrow().properties,
                                &key,
                            )
                        })
                }
            }
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(text) = Self::string_wrapper_value_from_object(&entries) {
                    if key == "length" || key == "constructor" {
                        return Ok(true);
                    }
                    if key
                        .parse::<usize>()
                        .ok()
                        .is_some_and(|index| text.chars().nth(index).is_some())
                    {
                        return Ok(true);
                    }
                }
                self.object_has_property_in_chain(right, &entries, &key)
            }
            Value::FormData(entries) => entries.borrow().iter().any(|(name, _)| name == &key),
            _ => false,
        };

        Ok(has_property)
    }

    pub(crate) fn value_as_index(&self, value: &Value) -> Option<usize> {
        match value {
            Value::Number(v) => usize::try_from(*v).ok(),
            Value::Float(v) => {
                if !v.is_finite() || v.fract() != 0.0 || *v < 0.0 {
                    None
                } else {
                    usize::try_from(*v as i64).ok()
                }
            }
            Value::BigInt(v) => v.to_usize(),
            Value::String(s) => {
                if let Ok(int) = s.parse::<i64>() {
                    usize::try_from(int).ok()
                } else if let Ok(float) = s.parse::<f64>() {
                    if float.fract() == 0.0 && float >= 0.0 {
                        usize::try_from(float as i64).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
