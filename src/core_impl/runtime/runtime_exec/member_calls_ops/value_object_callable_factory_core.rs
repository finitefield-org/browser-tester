use super::*;

impl Harness {
    pub(crate) fn new_receiver_builtin_constructor_object(
        callable_kind: Option<&str>,
        family: &str,
        methods: &[&str],
    ) -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let mut constructor_entries = Vec::new();
        if let Some(kind) = callable_kind {
            constructor_entries.push((
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String(kind.to_string()),
            ));
        }
        constructor_entries.push(("prototype".to_string(), prototype.clone()));
        let constructor = Self::new_object_value(constructor_entries);
        if let Value::Object(prototype_entries) = &prototype {
            let mut prototype_entries = prototype_entries.borrow_mut();
            Self::object_set_entry(
                &mut prototype_entries,
                "constructor".to_string(),
                constructor.clone(),
            );
            for method in methods {
                Self::object_set_entry(
                    &mut prototype_entries,
                    (*method).to_string(),
                    Self::new_receiver_builtin_callable(family, method),
                );
            }
        }
        if let Value::Object(prototype_entries) = &prototype {
            Self::mark_existing_public_properties_non_enumerable(prototype_entries);
        }
        if let Value::Object(constructor_entries) = &constructor {
            Self::mark_property_non_enumerable(constructor_entries, "prototype");
        }
        constructor
    }

    pub(crate) fn new_object_backed_constructor_with_prototype(
        callable_kind: &str,
        extra_public_entries: Vec<(String, Value)>,
    ) -> Value {
        let prototype = Self::new_object_value(Vec::new());
        let mut constructor_entries = vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String(callable_kind.to_string()),
            ),
            ("prototype".to_string(), prototype.clone()),
        ];
        constructor_entries.extend(extra_public_entries);
        let constructor = Self::new_object_value(constructor_entries);
        if let Value::Object(prototype_entries) = &prototype {
            Self::object_set_entry(
                &mut prototype_entries.borrow_mut(),
                "constructor".to_string(),
                constructor.clone(),
            );
            Self::mark_existing_public_properties_non_enumerable(prototype_entries);
        }
        if let Value::Object(constructor_entries) = &constructor {
            Self::mark_existing_public_properties_non_enumerable(constructor_entries);
        }
        constructor
    }

    pub(crate) fn new_bound_function_callable(
        target: Value,
        bound_this: Value,
        bound_args: Vec<Value>,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("bound_function".to_string()),
            ),
            (INTERNAL_BOUND_CALLABLE_TARGET_KEY.to_string(), target),
            (INTERNAL_BOUND_CALLABLE_THIS_KEY.to_string(), bound_this),
            (
                INTERNAL_BOUND_CALLABLE_ARGS_KEY.to_string(),
                Self::new_array_value(bound_args),
            ),
            ("call".to_string(), Self::new_function_call_callable()),
            ("apply".to_string(), Self::new_function_apply_callable()),
            ("bind".to_string(), Self::new_function_bind_callable()),
        ])
    }

    pub(crate) fn new_receiver_builtin_callable(family: &str, member: &str) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("receiver_builtin_method".to_string()),
            ),
            (
                "__bt_receiver_builtin_family".to_string(),
                Value::String(family.to_string()),
            ),
            (
                "__bt_receiver_builtin_member".to_string(),
                Value::String(member.to_string()),
            ),
        ])
    }

    pub(crate) fn new_receiver_builtin_prototype_value(
        constructor: Value,
        family: &str,
        methods: &[&str],
    ) -> Value {
        let mut entries = vec![("constructor".to_string(), constructor)];
        for method in methods {
            entries.push((
                (*method).to_string(),
                Self::new_receiver_builtin_callable(family, method),
            ));
        }
        let prototype = Self::new_object_value(entries);
        if let Value::Object(entries) = &prototype {
            Self::mark_existing_public_properties_non_enumerable(entries);
        }
        prototype
    }

    pub(crate) fn new_receiver_builtin_prototype_with_iterator_value(
        &mut self,
        constructor: Value,
        family: &str,
        methods: &[&str],
        iterator_member: Option<&str>,
    ) -> Value {
        let prototype = Self::new_receiver_builtin_prototype_value(constructor, family, methods);
        let Some(iterator_member) = iterator_member else {
            return prototype;
        };
        let Value::Object(entries) = &prototype else {
            return prototype;
        };
        let iterator_symbol = self.eval_symbol_static_property(SymbolStaticProperty::Iterator);
        let iterator_key = self.property_key_to_storage_key(&iterator_symbol);
        Self::object_set_entry(
            &mut entries.borrow_mut(),
            iterator_key,
            Self::new_receiver_builtin_callable(family, iterator_member),
        );
        prototype
    }
}
