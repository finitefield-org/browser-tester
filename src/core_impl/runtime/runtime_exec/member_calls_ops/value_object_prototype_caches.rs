use super::*;

impl Harness {
    pub(crate) fn cached_symbol_static_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("Symbol.{method}"), || {
            Self::new_symbol_static_method_callable(method)
        })
    }

    pub(crate) fn cached_regexp_static_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("RegExp.{method}"), || {
            Self::new_regexp_static_method_callable(method)
        })
    }

    pub(crate) fn cached_promise_static_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("Promise.{method}"), || {
            Self::new_promise_static_method_callable(method)
        })
    }

    pub(crate) fn cached_array_buffer_static_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("ArrayBuffer.{method}"), || {
            Self::new_array_buffer_static_method_callable(method)
        })
    }

    pub(crate) fn cached_typed_array_static_method_value(
        &mut self,
        kind: TypedArrayConstructorKind,
        method: &str,
    ) -> Value {
        let constructor_name = Self::typed_array_constructor_cache_key(&kind);
        self.cached_constructor_static_method_value(&format!("{constructor_name}.{method}"), || {
            Self::new_typed_array_static_method_callable(kind.clone(), method)
        })
    }

    fn typed_array_constructor_cache_key(kind: &TypedArrayConstructorKind) -> String {
        match kind {
            TypedArrayConstructorKind::Concrete(kind) => kind.name().to_string(),
            TypedArrayConstructorKind::Abstract => "TypedArray".to_string(),
        }
    }

    fn cached_builtin_constructor_prototype_value(
        &mut self,
        cache_key: &str,
        make_value: impl FnOnce(&mut Self) -> Value,
    ) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get(cache_key)
            .cloned()
        {
            return Value::Object(prototype);
        }
        let value = make_value(self);
        let Value::Object(prototype) = &value else {
            return value;
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert(cache_key.to_string(), prototype.clone());
        value
    }

    pub(crate) fn cached_string_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self.script_runtime.string_constructor_prototype.clone() {
            return Value::Object(prototype);
        }
        let Value::Object(prototype) = self.new_receiver_builtin_prototype_with_iterator_value(
            Value::StringConstructor,
            "string",
            &[
                "at",
                "charAt",
                "charCodeAt",
                "concat",
                "codePointAt",
                "endsWith",
                "includes",
                "indexOf",
                "lastIndexOf",
                "isWellFormed",
                "localeCompare",
                "match",
                "matchAll",
                "normalize",
                "padEnd",
                "padStart",
                "replace",
                "replaceAll",
                "repeat",
                "search",
                "slice",
                "split",
                "startsWith",
                "substring",
                "toLocaleLowerCase",
                "toLocaleUpperCase",
                "toLowerCase",
                "toString",
                "toUpperCase",
                "toWellFormed",
                "trim",
                "trimEnd",
                "trimStart",
                "valueOf",
            ],
            Some("iterator"),
        ) else {
            unreachable!("string constructor prototype must be an object");
        };
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime.string_constructor_prototype = Some(prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_symbol_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self.script_runtime.symbol_constructor_prototype.clone() {
            return Value::Object(prototype);
        }
        let Value::Object(prototype) = Self::new_receiver_builtin_prototype_value(
            Value::SymbolConstructor,
            "symbol",
            &["toString", "valueOf"],
        ) else {
            unreachable!("symbol constructor prototype must be an object");
        };
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime.symbol_constructor_prototype = Some(prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_typed_array_constructor_prototype_value(
        &mut self,
        kind: TypedArrayConstructorKind,
    ) -> Value {
        let cache_key = Self::typed_array_constructor_cache_key(&kind);
        if let Some(prototype) = self
            .script_runtime
            .typed_array_constructor_prototypes
            .get(&cache_key)
            .cloned()
        {
            return Value::Object(prototype);
        }
        let Value::Object(prototype) = self.new_receiver_builtin_prototype_with_iterator_value(
            Value::TypedArrayConstructor(kind.clone()),
            "typed_array",
            &[
                "at",
                "copyWithin",
                "entries",
                "join",
                "keys",
                "slice",
                "subarray",
                "values",
                "with",
            ],
            Some("values"),
        ) else {
            unreachable!("typed array constructor prototype must be an object");
        };
        let parent_prototype = match kind {
            TypedArrayConstructorKind::Concrete(_) => self
                .cached_typed_array_constructor_prototype_value(
                    TypedArrayConstructorKind::Abstract,
                ),
            TypedArrayConstructorKind::Abstract => self.object_constructor_prototype_value(),
        };
        Self::set_internal_prototype(&prototype, parent_prototype);
        self.script_runtime
            .typed_array_constructor_prototypes
            .insert(cache_key, prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_blob_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("Blob", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::BlobConstructor,
                "blob",
                &["arrayBuffer", "bytes", "slice", "stream", "text"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    pub(crate) fn cached_array_buffer_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("ArrayBuffer", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::ArrayBufferConstructor,
                "array_buffer",
                &["resize", "slice", "transfer", "transferToFixedLength"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    pub(crate) fn cached_promise_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("Promise", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::PromiseConstructor,
                "promise",
                &["then", "catch", "finally"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    pub(crate) fn cached_date_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("Date", |this| {
            let prototype = Self::new_object_value(
                [
                    "getTime",
                    "setTime",
                    "toISOString",
                    "toLocaleDateString",
                    "toString",
                    "valueOf",
                    "getUTCFullYear",
                    "getUTCMonth",
                    "getUTCDate",
                    "getUTCDay",
                    "getUTCHours",
                    "getUTCMinutes",
                    "getUTCSeconds",
                    "getUTCMilliseconds",
                    "getFullYear",
                    "getMonth",
                    "getDate",
                    "getHours",
                    "getMinutes",
                    "getSeconds",
                ]
                .into_iter()
                .map(|method| {
                    (
                        method.to_string(),
                        Self::new_receiver_builtin_callable("date", method),
                    )
                })
                .collect::<Vec<_>>(),
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            let to_string_tag_symbol =
                this.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
            let to_string_tag_key = this.property_key_to_storage_key(&to_string_tag_symbol);
            Self::object_set_entry(
                &mut entries.borrow_mut(),
                to_string_tag_key.clone(),
                Value::String("Date".to_string()),
            );
            Self::mark_existing_public_properties_non_enumerable(entries);
            Self::mark_property_non_enumerable(entries, &to_string_tag_key);
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    pub(crate) fn cached_regexp_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("RegExp", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::RegExpConstructor,
                "regexp",
                &["exec", "test", "toString"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            for (property, member) in [
                (SymbolStaticProperty::Match, "match"),
                (SymbolStaticProperty::MatchAll, "matchAll"),
                (SymbolStaticProperty::Replace, "replace"),
                (SymbolStaticProperty::Search, "search"),
                (SymbolStaticProperty::Split, "split"),
            ] {
                let symbol = this.eval_symbol_static_property(property);
                let key = this.property_key_to_storage_key(&symbol);
                Self::object_set_entry(
                    &mut entries.borrow_mut(),
                    key.clone(),
                    Self::new_receiver_builtin_callable("regexp", member),
                );
                Self::mark_property_non_enumerable(entries, &key);
            }
            for key in [
                "source",
                "flags",
                "global",
                "ignoreCase",
                "multiline",
                "dotAll",
                "sticky",
                "hasIndices",
                "unicode",
                "unicodeSets",
            ] {
                Self::object_set_entry(
                    &mut entries.borrow_mut(),
                    key.to_string(),
                    Value::Undefined,
                );
                Self::object_set_entry(
                    &mut entries.borrow_mut(),
                    Self::object_getter_storage_key(key),
                    Self::new_receiver_builtin_callable("regexp", key),
                );
                Self::mark_property_non_enumerable(entries, key);
            }
            let to_string_tag_symbol =
                this.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
            let to_string_tag_key = this.property_key_to_storage_key(&to_string_tag_symbol);
            Self::object_set_entry(
                &mut entries.borrow_mut(),
                to_string_tag_key.clone(),
                Value::String("RegExp".to_string()),
            );
            Self::object_set_entry(
                &mut entries.borrow_mut(),
                INTERNAL_REGEXP_PROTOTYPE_OBJECT_KEY.to_string(),
                Value::Bool(true),
            );
            Self::mark_property_non_enumerable(entries, &to_string_tag_key);
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    pub(crate) fn cached_map_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("Map", |this| {
            let prototype = this.new_receiver_builtin_prototype_with_iterator_value(
                Value::MapConstructor,
                "map",
                &[
                    "set",
                    "get",
                    "has",
                    "delete",
                    "clear",
                    "forEach",
                    "entries",
                    "keys",
                    "values",
                    "getOrInsert",
                    "getOrInsertComputed",
                ],
                Some("entries"),
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    pub(crate) fn cached_weak_map_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("WeakMap", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::WeakMapConstructor,
                "weak_map",
                &[
                    "set",
                    "get",
                    "has",
                    "delete",
                    "getOrInsert",
                    "getOrInsertComputed",
                ],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    pub(crate) fn cached_set_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("Set", |this| {
            let prototype = this.new_receiver_builtin_prototype_with_iterator_value(
                Value::SetConstructor,
                "set",
                &[
                    "add",
                    "has",
                    "delete",
                    "clear",
                    "forEach",
                    "entries",
                    "keys",
                    "values",
                    "union",
                    "intersection",
                    "difference",
                    "symmetricDifference",
                    "isDisjointFrom",
                    "isSubsetOf",
                    "isSupersetOf",
                ],
                Some("values"),
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    pub(crate) fn cached_weak_set_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("WeakSet", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::WeakSetConstructor,
                "weak_set",
                &["add", "has", "delete"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    pub(crate) fn cached_url_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("URL", |this| {
            let prototype = Self::new_receiver_builtin_prototype_value(
                Value::UrlConstructor,
                "url",
                &["toString", "toJSON"],
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }

    pub(crate) fn cached_url_search_params_constructor_prototype_value(&mut self) -> Value {
        self.cached_builtin_constructor_prototype_value("URLSearchParams", |this| {
            let prototype = this.new_receiver_builtin_prototype_with_iterator_value(
                Value::UrlSearchParamsConstructor,
                "url_search_params",
                &[
                    "append",
                    "delete",
                    "get",
                    "getAll",
                    "has",
                    "set",
                    "sort",
                    "forEach",
                    "entries",
                    "keys",
                    "values",
                    "toString",
                ],
                Some("entries"),
            );
            let Value::Object(entries) = &prototype else {
                return prototype;
            };
            Self::set_internal_prototype(entries, this.object_constructor_prototype_value());
            prototype
        })
    }
}
