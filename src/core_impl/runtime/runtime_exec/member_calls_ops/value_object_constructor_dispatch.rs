use super::*;

impl Harness {
    pub(crate) fn object_property_from_constructor_value(
        &mut self,
        value: &Value,
        key: &str,
    ) -> Option<Result<Value>> {
        let result = match value {
            Value::MapConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => self.cached_map_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::WeakMapConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => self.cached_weak_map_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::SetConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => self.cached_set_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::WeakSetConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => self.cached_weak_set_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::TypedArrayConstructor(kind) => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => {
                        self.cached_typed_array_constructor_prototype_value(kind.clone())
                    }
                    "from" if matches!(kind, TypedArrayConstructorKind::Concrete(_)) => {
                        self.cached_typed_array_static_method_value(kind.clone(), "from")
                    }
                    "of" if matches!(kind, TypedArrayConstructorKind::Concrete(_)) => {
                        self.cached_typed_array_static_method_value(kind.clone(), "of")
                    }
                    "BYTES_PER_ELEMENT" => match kind {
                        TypedArrayConstructorKind::Concrete(kind) => {
                            Value::Number(kind.bytes_per_element() as i64)
                        }
                        TypedArrayConstructorKind::Abstract => Value::Undefined,
                    },
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::BlobConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => self.cached_blob_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::RegExpConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => self.cached_regexp_constructor_prototype_value(),
                    "escape" => self.cached_regexp_static_method_value("escape"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::UrlConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = if key == "prototype" {
                    self.cached_url_constructor_prototype_value()
                } else if let Some(value) = Self::object_get_entry(
                    &self.browser_apis.url_constructor_properties.borrow(),
                    key,
                ) {
                    value
                } else if Self::is_url_static_method_name(key) {
                    Self::new_builtin_placeholder_function()
                } else {
                    Value::Undefined
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::ArrayBufferConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => self.cached_array_buffer_constructor_prototype_value(),
                    "isView" => self.cached_array_buffer_static_method_value("isView"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::PromiseConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => self.cached_promise_constructor_prototype_value(),
                    "resolve" => self.cached_promise_static_method_value("resolve"),
                    "reject" => self.cached_promise_static_method_value("reject"),
                    "all" => self.cached_promise_static_method_value("all"),
                    "allSettled" => self.cached_promise_static_method_value("allSettled"),
                    "any" => self.cached_promise_static_method_value("any"),
                    "race" => self.cached_promise_static_method_value("race"),
                    "try" => self.cached_promise_static_method_value("try"),
                    "withResolvers" => self.cached_promise_static_method_value("withResolvers"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::UrlSearchParamsConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => self.cached_url_search_params_constructor_prototype_value(),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::StringConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => self.cached_string_constructor_prototype_value(),
                    "fromCharCode" => self.cached_string_static_method_value("fromCharCode"),
                    "fromCodePoint" => self.cached_string_static_method_value("fromCodePoint"),
                    "raw" => self.cached_string_static_method_value("raw"),
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            Value::SymbolConstructor => {
                if let Some(value) = self.callable_function_surface_value(value, key) {
                    return Some(Ok(value));
                }
                let own_value = match key {
                    "prototype" => self.cached_symbol_constructor_prototype_value(),
                    "for" => self.cached_symbol_static_method_value("for"),
                    "keyFor" => self.cached_symbol_static_method_value("keyFor"),
                    "asyncDispose" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::AsyncDispose)
                    }
                    "asyncIterator" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::AsyncIterator)
                    }
                    "dispose" => self.eval_symbol_static_property(SymbolStaticProperty::Dispose),
                    "hasInstance" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::HasInstance)
                    }
                    "isConcatSpreadable" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::IsConcatSpreadable)
                    }
                    "iterator" => self.eval_symbol_static_property(SymbolStaticProperty::Iterator),
                    "match" => self.eval_symbol_static_property(SymbolStaticProperty::Match),
                    "matchAll" => self.eval_symbol_static_property(SymbolStaticProperty::MatchAll),
                    "replace" => self.eval_symbol_static_property(SymbolStaticProperty::Replace),
                    "search" => self.eval_symbol_static_property(SymbolStaticProperty::Search),
                    "species" => self.eval_symbol_static_property(SymbolStaticProperty::Species),
                    "split" => self.eval_symbol_static_property(SymbolStaticProperty::Split),
                    "toPrimitive" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::ToPrimitive)
                    }
                    "toStringTag" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag)
                    }
                    "unscopables" => {
                        self.eval_symbol_static_property(SymbolStaticProperty::Unscopables)
                    }
                    _ => Value::Undefined,
                };
                self.callable_value_property_or_inherited(value, key, own_value)
            }
            _ => return None,
        };
        Some(result)
    }
}
