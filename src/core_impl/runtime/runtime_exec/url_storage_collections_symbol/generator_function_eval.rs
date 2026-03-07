use super::*;

impl Harness {
    pub(crate) fn new_generator_function_constructor_value(&mut self) -> Value {
        if let Some(constructor) = self
            .script_runtime
            .constructor_static_methods
            .get("GeneratorFunction")
            .cloned()
        {
            return constructor;
        }

        let constructor = Rc::new(RefCell::new(ObjectValue::new(vec![
            (
                INTERNAL_GENERATOR_FUNCTION_CONSTRUCTOR_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("generator_function_constructor".to_string()),
            ),
        ])));
        let constructor_value = Value::Object(constructor.clone());
        let generator_prototype = Rc::new(RefCell::new(ObjectValue::new(vec![(
            INTERNAL_GENERATOR_PROTOTYPE_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )])));
        {
            let mut generator_prototype_entries = generator_prototype.borrow_mut();
            for (key, value) in Self::generator_prototype_method_entries() {
                Self::object_set_entry(&mut generator_prototype_entries, key, value);
            }
        }
        let generator_prototype_value = Value::Object(generator_prototype.clone());
        let prototype = Rc::new(RefCell::new(ObjectValue::new(vec![
            (
                INTERNAL_GENERATOR_FUNCTION_PROTOTYPE_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("constructor".to_string(), constructor_value.clone()),
            ("prototype".to_string(), generator_prototype_value),
        ])));
        let prototype_value = Value::Object(prototype.clone());
        Self::object_set_entry(
            &mut constructor.borrow_mut(),
            INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
            self.cached_function_constructor_prototype_value(),
        );
        Self::object_set_entry(
            &mut generator_prototype.borrow_mut(),
            "constructor".to_string(),
            prototype_value.clone(),
        );
        Self::mark_existing_public_properties_non_enumerable(&generator_prototype);
        Self::object_set_entry(
            &mut prototype.borrow_mut(),
            INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
            self.cached_function_constructor_prototype_value(),
        );
        Self::mark_existing_public_properties_non_enumerable(&prototype);
        Self::object_set_entry(
            &mut constructor.borrow_mut(),
            "prototype".to_string(),
            prototype_value,
        );
        Self::mark_existing_public_properties_non_enumerable(&constructor);
        self.script_runtime
            .builtin_constructor_prototypes
            .insert("GeneratorFunction".to_string(), prototype);
        self.script_runtime
            .constructor_static_methods
            .insert("GeneratorFunction".to_string(), constructor_value.clone());
        constructor_value
    }

    pub(crate) fn is_generator_function_prototype_object(
        entries: &(impl ObjectEntryLookup + ?Sized),
    ) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_GENERATOR_FUNCTION_PROTOTYPE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn build_generator_function_from_constructor_values(
        &mut self,
        args: &[Value],
    ) -> Result<Value> {
        let mut parts = Vec::with_capacity(args.len());
        for arg in args {
            parts.push(arg.as_string());
        }
        let body_src = parts.last().cloned().unwrap_or_default();
        let mut params = Vec::new();
        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            let names = Self::parse_function_constructor_param_names(part)?;
            params.extend(names.into_iter().map(|name| FunctionParam {
                name,
                default: None,
                is_rest: false,
            }));
        }

        let stmts = parse_block_statements(&body_src).map_err(|err| {
            Error::ScriptRuntime(format!("GeneratorFunction body parse failed: {err}"))
        })?;
        let empty_env = HashMap::new();
        let value = self.make_function_value(
            ScriptHandler { params, stmts },
            &empty_env,
            true,
            false,
            true,
            false,
            false,
        );
        if let Value::Function(function) = &value {
            self.set_function_public_name(function, "anonymous");
        }
        Ok(value)
    }
}
