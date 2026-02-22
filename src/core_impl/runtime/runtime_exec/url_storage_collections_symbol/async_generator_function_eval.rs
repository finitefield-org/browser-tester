use super::*;

impl Harness {
    pub(crate) fn new_async_generator_function_constructor_value(&self) -> Value {
        let constructor = Rc::new(RefCell::new(ObjectValue::new(vec![
            (
                INTERNAL_ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("async_generator_function_constructor".to_string()),
            ),
        ])));
        let constructor_value = Value::Object(constructor.clone());
        let async_generator_prototype = Self::new_object_value(vec![(
            "next".to_string(),
            Self::new_builtin_placeholder_function(),
        )]);
        let prototype = Self::new_object_value(vec![
            (
                INTERNAL_ASYNC_GENERATOR_FUNCTION_PROTOTYPE_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("constructor".to_string(), constructor_value.clone()),
            ("prototype".to_string(), async_generator_prototype),
        ]);
        Self::object_set_entry(
            &mut constructor.borrow_mut(),
            "prototype".to_string(),
            prototype,
        );
        constructor_value
    }

    pub(crate) fn is_async_generator_function_prototype_object(
        entries: &(impl ObjectEntryLookup + ?Sized),
    ) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_ASYNC_GENERATOR_FUNCTION_PROTOTYPE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn build_async_generator_function_from_constructor_values(
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
            Error::ScriptRuntime(format!("AsyncGeneratorFunction body parse failed: {err}"))
        })?;
        let empty_env = HashMap::new();
        Ok(self.make_function_value(
            ScriptHandler { params, stmts },
            &empty_env,
            true,
            true,
            true,
        ))
    }
}
