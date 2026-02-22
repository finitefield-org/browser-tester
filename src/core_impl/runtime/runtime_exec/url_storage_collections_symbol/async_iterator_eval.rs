use super::*;

impl Harness {
    pub(crate) fn async_generator_prototype_method_entries() -> Vec<(String, Value)> {
        vec![
            ("next".to_string(), Self::new_builtin_placeholder_function()),
            (
                "return".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "throw".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ]
    }

    pub(crate) fn new_readable_stream_async_iterator_callable(&self, chunks: Vec<Value>) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("readable_stream_async_iterator".to_string()),
            ),
            (
                INTERNAL_ASYNC_ITERATOR_VALUES_KEY.to_string(),
                Self::new_array_value(chunks),
            ),
        ])
    }

    pub(crate) fn new_async_iterator_next_callable(&self, iterator: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("async_iterator_next".to_string()),
            ),
            (INTERNAL_ASYNC_ITERATOR_TARGET_KEY.to_string(), iterator),
        ])
    }

    pub(crate) fn new_async_iterator_return_callable(&self, iterator: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("async_iterator_return".to_string()),
            ),
            (INTERNAL_ASYNC_ITERATOR_TARGET_KEY.to_string(), iterator),
        ])
    }

    pub(crate) fn new_async_iterator_throw_callable(&self, iterator: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("async_iterator_throw".to_string()),
            ),
            (INTERNAL_ASYNC_ITERATOR_TARGET_KEY.to_string(), iterator),
        ])
    }

    pub(crate) fn new_async_iterator_self_callable(&self, iterator: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("async_iterator_self".to_string()),
            ),
            (INTERNAL_ASYNC_ITERATOR_TARGET_KEY.to_string(), iterator),
        ])
    }

    pub(crate) fn new_async_iterator_async_dispose_callable(&self, iterator: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("async_iterator_async_dispose".to_string()),
            ),
            (INTERNAL_ASYNC_ITERATOR_TARGET_KEY.to_string(), iterator),
        ])
    }

    pub(crate) fn new_async_generator_result_value_callable(&self) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("async_generator_result_value".to_string()),
        )])
    }

    pub(crate) fn new_async_generator_result_done_callable(&self) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("async_generator_result_done".to_string()),
        )])
    }

    pub(crate) fn new_async_iterator_result_object(value: Value, done: bool) -> Value {
        Self::new_object_value(vec![
            ("value".to_string(), value),
            ("done".to_string(), Value::Bool(done)),
        ])
    }

    pub(crate) fn resolve_async_generator_iterator_result_promise(
        &mut self,
        value: Value,
        done: bool,
    ) -> Result<Value> {
        let awaited = self.promise_resolve_value_as_promise(value)?;
        let mapper = if done {
            self.new_async_generator_result_done_callable()
        } else {
            self.new_async_generator_result_value_callable()
        };
        Ok(Value::Promise(self.promise_then_internal(
            &awaited,
            Some(mapper),
            None,
        )))
    }

    pub(crate) fn new_async_iterator_value(&mut self, values: Vec<Value>) -> Value {
        let iterator = Rc::new(RefCell::new(ObjectValue::new(vec![
            (
                INTERNAL_ASYNC_ITERATOR_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_ASYNC_ITERATOR_VALUES_KEY.to_string(),
                Self::new_array_value(values),
            ),
            (
                INTERNAL_ASYNC_ITERATOR_INDEX_KEY.to_string(),
                Value::Number(0),
            ),
        ])));
        let iterator_value = Value::Object(iterator.clone());
        let next = self.new_async_iterator_next_callable(iterator_value.clone());
        let self_factory = self.new_async_iterator_self_callable(iterator_value.clone());
        let async_dispose = self.new_async_iterator_async_dispose_callable(iterator_value);

        let async_iterator_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::AsyncIterator);
        let async_iterator_key = self.property_key_to_storage_key(&async_iterator_symbol);
        let async_dispose_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::AsyncDispose);
        let async_dispose_key = self.property_key_to_storage_key(&async_dispose_symbol);

        let mut entries = iterator.borrow_mut();
        Self::object_set_entry(&mut entries, "next".to_string(), next);
        Self::object_set_entry(&mut entries, async_iterator_key, self_factory);
        Self::object_set_entry(&mut entries, async_dispose_key, async_dispose);
        drop(entries);

        Value::Object(iterator)
    }

    pub(crate) fn new_async_generator_value(&mut self, values: Vec<Value>) -> Value {
        let iterator = Rc::new(RefCell::new(ObjectValue::new(vec![
            (
                INTERNAL_ASYNC_ITERATOR_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_ASYNC_GENERATOR_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_ASYNC_ITERATOR_VALUES_KEY.to_string(),
                Self::new_array_value(values),
            ),
            (
                INTERNAL_ASYNC_ITERATOR_INDEX_KEY.to_string(),
                Value::Number(0),
            ),
        ])));
        let iterator_value = Value::Object(iterator.clone());
        let next = self.new_async_iterator_next_callable(iterator_value.clone());
        let return_fn = self.new_async_iterator_return_callable(iterator_value.clone());
        let throw_fn = self.new_async_iterator_throw_callable(iterator_value.clone());
        let self_factory = self.new_async_iterator_self_callable(iterator_value.clone());
        let async_dispose = self.new_async_iterator_async_dispose_callable(iterator_value);

        let async_iterator_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::AsyncIterator);
        let async_iterator_key = self.property_key_to_storage_key(&async_iterator_symbol);
        let async_dispose_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::AsyncDispose);
        let async_dispose_key = self.property_key_to_storage_key(&async_dispose_symbol);
        let to_string_tag_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::ToStringTag);
        let to_string_tag_key = self.property_key_to_storage_key(&to_string_tag_symbol);

        let mut entries = iterator.borrow_mut();
        Self::object_set_entry(&mut entries, "next".to_string(), next);
        Self::object_set_entry(&mut entries, "return".to_string(), return_fn);
        Self::object_set_entry(&mut entries, "throw".to_string(), throw_fn);
        Self::object_set_entry(&mut entries, async_iterator_key, self_factory);
        Self::object_set_entry(&mut entries, async_dispose_key, async_dispose);
        Self::object_set_entry(
            &mut entries,
            to_string_tag_key,
            Value::String("AsyncGenerator".to_string()),
        );
        drop(entries);

        Value::Object(iterator)
    }

    pub(crate) fn is_async_iterator_object(entries: &(impl ObjectEntryLookup + ?Sized)) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_ASYNC_ITERATOR_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_async_generator_object(entries: &(impl ObjectEntryLookup + ?Sized)) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_ASYNC_GENERATOR_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_async_generator_prototype_object(
        entries: &(impl ObjectEntryLookup + ?Sized),
    ) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_ASYNC_GENERATOR_PROTOTYPE_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn async_iterator_target_from_callable(
        &self,
        callable: &Value,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        let Value::Object(entries) = callable else {
            return Err(Error::ScriptRuntime("callback is not a function".into()));
        };
        let entries = entries.borrow();
        let Some(Value::Object(target)) =
            Self::object_get_entry(&entries, INTERNAL_ASYNC_ITERATOR_TARGET_KEY)
        else {
            return Err(Error::ScriptRuntime(
                "AsyncIterator callable has invalid internal state".into(),
            ));
        };
        if !Self::is_async_iterator_object(&target.borrow()) {
            return Err(Error::ScriptRuntime(
                "AsyncIterator callable has invalid internal state".into(),
            ));
        }
        Ok(target)
    }

    pub(crate) fn close_async_iterator_object(
        &self,
        iterator: &Rc<RefCell<ObjectValue>>,
    ) -> Result<()> {
        let mut entries = iterator.borrow_mut();
        if !Self::is_async_iterator_object(&entries) {
            return Err(Error::ScriptRuntime("value is not an AsyncIterator".into()));
        }
        let values = match Self::object_get_entry(&entries, INTERNAL_ASYNC_ITERATOR_VALUES_KEY) {
            Some(Value::Array(values)) => values,
            _ => {
                return Err(Error::ScriptRuntime(
                    "AsyncIterator has invalid internal state".into(),
                ));
            }
        };
        let len = values.borrow().len();
        Self::object_set_entry(
            &mut entries,
            INTERNAL_ASYNC_ITERATOR_INDEX_KEY.to_string(),
            Value::Number(len as i64),
        );
        Ok(())
    }

    pub(crate) fn async_iterator_next_value_from_object(
        &self,
        iterator: &Rc<RefCell<ObjectValue>>,
    ) -> Result<Option<Value>> {
        let mut entries = iterator.borrow_mut();
        if !Self::is_async_iterator_object(&entries) {
            return Err(Error::ScriptRuntime("value is not an AsyncIterator".into()));
        }
        let values = match Self::object_get_entry(&entries, INTERNAL_ASYNC_ITERATOR_VALUES_KEY) {
            Some(Value::Array(values)) => values,
            _ => {
                return Err(Error::ScriptRuntime(
                    "AsyncIterator has invalid internal state".into(),
                ));
            }
        };
        let index = match Self::object_get_entry(&entries, INTERNAL_ASYNC_ITERATOR_INDEX_KEY) {
            Some(Value::Number(value)) if value >= 0 => value as usize,
            _ => 0,
        };
        let value = {
            let values = values.borrow();
            if index >= values.len() {
                return Ok(None);
            }
            values.get(index).cloned().unwrap_or(Value::Undefined)
        };
        Self::object_set_entry(
            &mut entries,
            INTERNAL_ASYNC_ITERATOR_INDEX_KEY.to_string(),
            Value::Number((index + 1) as i64),
        );
        Ok(Some(value))
    }
}
