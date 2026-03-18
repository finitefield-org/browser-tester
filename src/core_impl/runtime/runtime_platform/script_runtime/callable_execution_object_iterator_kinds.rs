use super::*;

impl Harness {
    pub(crate) fn execute_object_callable_iterator_kind(
        &mut self,
        kind: &str,
        callable: &Value,
        args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match kind {
            "intl_segmenter_segments_iterator" => {
                let Value::Object(entries) = callable else {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                };
                let entries = entries.borrow();
                let segments = Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENTS_KEY)
                    .ok_or_else(|| {
                        Error::ScriptRuntime(
                            "Intl.Segmenter iterator has invalid internal state".into(),
                        )
                    })?;
                Some(self.new_intl_segmenter_iterator_value(segments))
            }
            "intl_segmenter_iterator_next" => {
                let Value::Object(entries) = callable else {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                };
                let mut entries = entries.borrow_mut();
                let segments = Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENTS_KEY)
                    .ok_or_else(|| {
                        Error::ScriptRuntime(
                            "Intl.Segmenter iterator has invalid internal state".into(),
                        )
                    })?;
                let Value::Array(values) = segments else {
                    return Err(Error::ScriptRuntime(
                        "Intl.Segmenter iterator has invalid internal state".into(),
                    ));
                };
                let len = values.borrow().len();
                let index = match Self::object_get_entry(&entries, INTERNAL_INTL_SEGMENT_INDEX_KEY)
                {
                    Some(Value::Number(value)) if value >= 0 => value as usize,
                    _ => 0,
                };
                if index >= len {
                    return Ok(Some(Self::new_object_value(vec![
                        ("value".to_string(), Value::Undefined),
                        ("done".to_string(), Value::Bool(true)),
                    ])));
                }
                let value = values
                    .borrow()
                    .get(index)
                    .cloned()
                    .unwrap_or(Value::Undefined);
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_INTL_SEGMENT_INDEX_KEY.to_string(),
                    Value::Number((index + 1) as i64),
                );
                Some(Self::new_object_value(vec![
                    ("value".to_string(), value),
                    ("done".to_string(), Value::Bool(false)),
                ]))
            }
            "readable_stream_async_iterator" => {
                let Value::Object(entries) = callable else {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                };
                let entries = entries.borrow();
                let chunks =
                    match Self::object_get_entry(&entries, INTERNAL_ASYNC_ITERATOR_VALUES_KEY) {
                        Some(Value::Array(values)) => values.borrow().clone(),
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "ReadableStream async iterator has invalid internal state".into(),
                            ));
                        }
                    };
                Some(self.new_async_iterator_value(chunks))
            }
            "named_node_map_iterator" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap[Symbol.iterator] does not take arguments".into(),
                    ));
                }
                let Value::Object(entries) = callable else {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                };
                let entries = entries.borrow();
                let Some(owner) = Self::named_node_map_owner_node(&entries) else {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap iterator has invalid internal state".into(),
                    ));
                };
                let values = self
                    .named_node_map_entries(owner)
                    .into_iter()
                    .map(|(name, value)| Self::new_attr_object_value(&name, &value, Some(owner)))
                    .collect::<Vec<_>>();
                Some(self.new_iterator_value(values))
            }
            "iterator_self" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Iterator[Symbol.iterator] does not take arguments".into(),
                    ));
                }
                let iterator = self.iterator_target_from_callable(callable)?;
                Some(Value::Object(iterator))
            }
            "async_generator_result_value" => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Some(Self::new_async_iterator_result_object(value, false))
            }
            "async_generator_result_done" => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Some(Self::new_async_iterator_result_object(value, true))
            }
            "async_iterator_next" => {
                let iterator = self.async_iterator_target_from_callable(callable)?;
                let is_async_generator = {
                    let entries = iterator.borrow();
                    Self::is_async_generator_object(&entries)
                };
                if !is_async_generator && !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "AsyncIterator.next does not take arguments".into(),
                    ));
                }
                let result =
                    if let Some(value) = self.async_iterator_next_value_from_object(&iterator)? {
                        if is_async_generator {
                            return self
                                .resolve_async_generator_iterator_result_promise(value, false)
                                .map(Some);
                        }
                        Self::new_async_iterator_result_object(value, false)
                    } else {
                        Self::new_async_iterator_result_object(Value::Undefined, true)
                    };
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, result)?;
                Some(Value::Promise(promise))
            }
            "async_iterator_return" => {
                let iterator = self.async_iterator_target_from_callable(callable)?;
                let is_async_generator = {
                    let entries = iterator.borrow();
                    Self::is_async_generator_object(&entries)
                };
                if !is_async_generator {
                    return Err(Error::ScriptRuntime(
                        "AsyncIterator.return is not a function".into(),
                    ));
                }
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                self.close_async_iterator_object(&iterator)?;
                return self
                    .resolve_async_generator_iterator_result_promise(value, true)
                    .map(Some);
            }
            "async_iterator_throw" => {
                let iterator = self.async_iterator_target_from_callable(callable)?;
                let is_async_generator = {
                    let entries = iterator.borrow();
                    Self::is_async_generator_object(&entries)
                };
                if !is_async_generator {
                    return Err(Error::ScriptRuntime(
                        "AsyncIterator.throw is not a function".into(),
                    ));
                }
                let reason = args.first().cloned().unwrap_or(Value::Undefined);
                self.close_async_iterator_object(&iterator)?;
                let promise = self.new_pending_promise();
                self.promise_reject(&promise, reason);
                Some(Value::Promise(promise))
            }
            "async_iterator_self" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "AsyncIterator[Symbol.asyncIterator] does not take arguments".into(),
                    ));
                }
                let iterator = self.async_iterator_target_from_callable(callable)?;
                Some(Value::Object(iterator))
            }
            "async_iterator_async_dispose" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "AsyncIterator[Symbol.asyncDispose] does not take arguments".into(),
                    ));
                }
                let iterator = self.async_iterator_target_from_callable(callable)?;
                let return_value = {
                    let entries = iterator.borrow();
                    Self::object_get_entry(&entries, "return")
                };
                let dispose_result = if let Some(return_method) = return_value {
                    if !self.is_callable_value(&return_method) {
                        return Err(Error::ScriptRuntime(
                            "AsyncIterator.return is not a function".into(),
                        ));
                    }
                    self.execute_callable_value(&return_method, &[], event)?
                } else {
                    Value::Undefined
                };
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, dispose_result)?;
                Some(Value::Promise(promise))
            }
            _ => None,
        };
        Ok(value)
    }
}
