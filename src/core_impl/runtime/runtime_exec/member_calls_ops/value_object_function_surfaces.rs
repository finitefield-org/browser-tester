use super::*;

impl Harness {
    pub(crate) fn new_function_call_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("function_call".to_string()),
        )])
    }

    pub(crate) fn new_function_apply_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("function_apply".to_string()),
        )])
    }

    pub(crate) fn new_function_bind_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("function_bind".to_string()),
        )])
    }

    pub(crate) fn new_function_to_string_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("function_to_string".to_string()),
        )])
    }

    pub(crate) fn new_string_static_from_char_code_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("string_static_from_char_code".to_string()),
        )])
    }

    pub(crate) fn new_string_static_from_code_point_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("string_static_from_code_point".to_string()),
        )])
    }

    pub(crate) fn new_string_static_raw_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("string_static_raw".to_string()),
        )])
    }

    pub(crate) fn cached_function_constructor_value(&mut self) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get("Function")
            .cloned()
        {
            if let Value::Object(constructor_entries) = &constructor {
                let prototype = {
                    let entries = constructor_entries.borrow();
                    Self::object_get_entry(&entries, "prototype")
                };
                if let Some(Value::Object(prototype)) = prototype {
                    Self::mark_existing_public_properties_non_enumerable(&prototype);
                    Self::mark_existing_public_properties_non_enumerable(constructor_entries);
                    Self::set_internal_prototype(
                        &prototype,
                        self.object_constructor_prototype_value(),
                    );
                    Self::set_internal_prototype(constructor_entries, Value::Object(prototype));
                }
            }
            return constructor;
        }

        let prototype = Rc::new(RefCell::new(ObjectValue::default()));
        let constructor = Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("function_constructor".to_string()),
            ),
            ("prototype".to_string(), Value::Object(prototype.clone())),
        ]);
        if let Value::Object(constructor_entries) = &constructor {
            Self::set_internal_prototype(constructor_entries, Value::Object(prototype.clone()));
        }

        {
            let mut prototype_entries = prototype.borrow_mut();
            Self::object_set_entry(
                &mut prototype_entries,
                "constructor".to_string(),
                constructor.clone(),
            );
            for method in ["call", "apply", "bind", "toString"] {
                Self::object_set_entry(
                    &mut prototype_entries,
                    method.to_string(),
                    self.cached_function_surface_method_value(method),
                );
            }
        }
        Self::mark_existing_public_properties_non_enumerable(&prototype);
        if let Value::Object(constructor_entries) = &constructor {
            Self::mark_existing_public_properties_non_enumerable(constructor_entries);
        }
        Self::set_internal_prototype(&prototype, self.object_constructor_prototype_value());

        self.script_runtime
            .builtin_constructor_prototypes
            .insert("Function".to_string(), prototype);
        self.script_runtime
            .constructor_static_methods
            .insert("Function".to_string(), constructor.clone());
        constructor
    }

    pub(crate) fn cached_function_constructor_prototype_value(&mut self) -> Value {
        if let Some(prototype) = self
            .script_runtime
            .builtin_constructor_prototypes
            .get("Function")
            .cloned()
        {
            return Value::Object(prototype);
        }
        let _ = self.cached_function_constructor_value();
        self.script_runtime
            .builtin_constructor_prototypes
            .get("Function")
            .cloned()
            .map(Value::Object)
            .unwrap_or_else(|| Self::new_object_value(Vec::new()))
    }

    pub(crate) fn function_family_constructor_bindings(&mut self) -> Vec<(String, Value)> {
        vec![
            (
                "Function".to_string(),
                self.cached_function_constructor_value(),
            ),
            (
                "GeneratorFunction".to_string(),
                self.new_generator_function_constructor_value(),
            ),
            (
                "AsyncGeneratorFunction".to_string(),
                self.new_async_generator_function_constructor_value(),
            ),
        ]
    }

    pub(crate) fn sync_function_prototype_object(&mut self, function: &Rc<FunctionValue>) {
        if function.is_arrow || function.is_method {
            return;
        }
        Self::mark_constructor_non_enumerable(&function.prototype_object);
        let mut prototype = function.prototype_object.borrow_mut();
        if Self::object_get_entry(&*prototype, INTERNAL_OBJECT_PROTOTYPE_KEY).is_none() {
            Self::object_set_entry(
                &mut *prototype,
                INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                self.object_constructor_prototype_value(),
            );
        }
        Self::object_set_entry(
            &mut *prototype,
            "constructor".to_string(),
            Value::Function(function.clone()),
        );
    }

    pub(crate) fn cached_function_surface_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("Function.prototype.{method}"), || {
            match method {
                "call" => Self::new_function_call_callable(),
                "apply" => Self::new_function_apply_callable(),
                "bind" => Self::new_function_bind_callable(),
                "toString" => Self::new_function_to_string_callable(),
                _ => Value::Undefined,
            }
        })
    }

    pub(crate) fn cached_string_static_method_value(&mut self, method: &str) -> Value {
        self.cached_constructor_static_method_value(&format!("String.{method}"), || match method {
            "fromCharCode" => Self::new_string_static_from_char_code_callable(),
            "fromCodePoint" => Self::new_string_static_from_code_point_callable(),
            "raw" => Self::new_string_static_raw_callable(),
            _ => Value::Undefined,
        })
    }

    pub(crate) fn set_function_public_name(&mut self, function: &Rc<FunctionValue>, name: &str) {
        let entries = self
            .script_runtime
            .function_public_properties
            .entry(function.function_id)
            .or_default();
        Self::object_set_entry(entries, "name".to_string(), Value::String(name.to_string()));
        Self::object_set_entry(
            entries,
            Self::object_non_enumerable_storage_key("name"),
            Value::Bool(true),
        );
        Self::object_set_entry(
            entries,
            Self::object_non_writable_storage_key("name"),
            Value::Bool(true),
        );
    }
}
