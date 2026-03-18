use super::*;

impl Harness {
    fn cached_placeholder_backed_interface_constructor_value(
        &mut self,
        interface_name: &str,
        callable_kind: &str,
        to_string_tag: &str,
    ) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get(interface_name)
            .cloned()
        {
            if let Value::Object(entries) = &constructor {
                let prototype = {
                    let entries = entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    Self::set_internal_prototype(
                        &prototype,
                        self.object_constructor_prototype_value(),
                    );
                }
            }
            return constructor;
        }

        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);
        let constructor =
            Self::new_receiver_builtin_constructor_object(Some(callable_kind), callable_kind, &[]);
        let Value::Object(constructor_entries) = &constructor else {
            return constructor;
        };
        let prototype = {
            let entries = constructor_entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return constructor;
        };
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            to_string_tag_key.clone(),
            Value::String(to_string_tag.to_string()),
        );
        Self::mark_property_non_enumerable(&prototype, &to_string_tag_key);
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
        self.script_runtime
            .constructor_static_methods
            .insert(interface_name.to_string(), constructor.clone());
        constructor
    }

    fn cached_placeholder_backed_interface_constructor_prototype_value(
        &mut self,
        interface_name: &str,
        callable_kind: &str,
        to_string_tag: &str,
    ) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get(interface_name)
            .cloned()
        {
            Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());
            return Value::Object(prototype);
        }
        let constructor = self.cached_placeholder_backed_interface_constructor_value(
            interface_name,
            callable_kind,
            to_string_tag,
        );
        let Value::Object(entries) = constructor else {
            return Self::new_object_value(Vec::new());
        };
        let prototype = {
            let entries = entries.borrow();
            Self::object_get_entry(&entries, "prototype")
        };
        let Some(Value::Object(prototype)) = prototype else {
            return Self::new_object_value(Vec::new());
        };
        self.script_runtime
            .builtin_constructor_prototypes
            .insert(interface_name.to_string(), prototype.clone());
        Value::Object(prototype)
    }

    pub(crate) fn cached_storage_constructor_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_value(
            "Storage",
            "storage_constructor",
            "Storage",
        )
    }

    pub(crate) fn cached_cookie_store_constructor_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_value(
            "CookieStore",
            "cookie_store_constructor",
            "CookieStore",
        )
    }

    pub(crate) fn cached_cache_storage_constructor_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_value(
            "CacheStorage",
            "cache_storage_constructor",
            "CacheStorage",
        )
    }

    pub(crate) fn cached_cache_constructor_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_value(
            "Cache",
            "cache_constructor",
            "Cache",
        )
    }

    pub(crate) fn cached_storage_constructor_prototype_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_prototype_value(
            "Storage",
            "storage_constructor",
            "Storage",
        )
    }

    pub(crate) fn cached_cookie_store_constructor_prototype_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_prototype_value(
            "CookieStore",
            "cookie_store_constructor",
            "CookieStore",
        )
    }

    pub(crate) fn cached_cache_storage_constructor_prototype_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_prototype_value(
            "CacheStorage",
            "cache_storage_constructor",
            "CacheStorage",
        )
    }

    pub(crate) fn cached_cache_constructor_prototype_value(&mut self) -> Value {
        self.cached_placeholder_backed_interface_constructor_prototype_value(
            "Cache",
            "cache_constructor",
            "Cache",
        )
    }
}
