use super::*;

impl Harness {
    pub(crate) fn new_generator_function_constructor_value(&self) -> Value {
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
            &mut generator_prototype.borrow_mut(),
            "constructor".to_string(),
            prototype_value.clone(),
        );
        Self::object_set_entry(
            &mut constructor.borrow_mut(),
            "prototype".to_string(),
            prototype_value,
        );
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
        Ok(self.make_function_value(
            ScriptHandler { params, stmts },
            &empty_env,
            true,
            false,
            true,
            false,
            false,
        ))
    }
}
