use super::*;

impl Harness {
    pub(crate) fn to_non_negative_usize(value: &Value, label: &str) -> Result<usize> {
        let n = Self::value_to_i64(value);
        if n < 0 {
            return Err(Error::ScriptRuntime(format!(
                "{label} must be a non-negative integer"
            )));
        }
        usize::try_from(n).map_err(|_| Error::ScriptRuntime(format!("{label} is too large")))
    }

    pub(crate) fn eval_call_args_with_spread(
        &mut self,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Vec<Value>> {
        let mut evaluated = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                Expr::Spread(inner) => {
                    let spread_value = self.eval_expr(inner, env, event_param, event)?;
                    evaluated.extend(self.spread_iterable_values_from_value(&spread_value)?);
                }
                _ => evaluated.push(self.eval_expr(arg, env, event_param, event)?),
            }
        }
        Ok(evaluated)
    }

    pub(crate) fn spread_iterable_values_from_value(&self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Array(values) => Ok(values.borrow().clone()),
            Value::TypedArray(values) => self.typed_array_snapshot(values),
            Value::Map(map) => {
                let map = map.borrow();
                Ok(map
                    .entries
                    .iter()
                    .map(|(key, value)| Self::new_array_value(vec![key.clone(), value.clone()]))
                    .collect::<Vec<_>>())
            }
            Value::Set(set) => Ok(set.borrow().values.clone()),
            Value::String(text) => Ok(text
                .chars()
                .map(|ch| Value::String(ch.to_string()))
                .collect::<Vec<_>>()),
            Value::NodeList(nodes) => Ok(self
                .node_list_snapshot(nodes)
                .into_iter()
                .map(Value::Node)
                .collect()),
            Value::Object(entries) => {
                let is_iterator = {
                    let entries_ref = entries.borrow();
                    Self::is_iterator_object(&entries_ref)
                };
                if is_iterator {
                    return self.iterator_collect_remaining_values(entries);
                }
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    return Ok(Self::url_search_params_pairs_from_object_entries(&entries)
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>());
                }
                Err(Error::ScriptRuntime("spread source is not iterable".into()))
            }
            _ => Err(Error::ScriptRuntime("spread source is not iterable".into())),
        }
    }

    pub(crate) fn array_like_values_from_value(&self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Array(values) => Ok(values.borrow().clone()),
            Value::TypedArray(values) => self.typed_array_snapshot(values),
            Value::Map(map) => {
                let map = map.borrow();
                Ok(map
                    .entries
                    .iter()
                    .map(|(key, value)| Self::new_array_value(vec![key.clone(), value.clone()]))
                    .collect::<Vec<_>>())
            }
            Value::Set(set) => Ok(set.borrow().values.clone()),
            Value::String(text) => Ok(text
                .chars()
                .map(|ch| Value::String(ch.to_string()))
                .collect::<Vec<_>>()),
            Value::NodeList(nodes) => Ok(self
                .node_list_snapshot(nodes)
                .into_iter()
                .map(Value::Node)
                .collect()),
            Value::Object(entries) => {
                let is_iterator = {
                    let entries_ref = entries.borrow();
                    Self::is_iterator_object(&entries_ref)
                };
                if is_iterator {
                    return self.iterator_collect_remaining_values(entries);
                }
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    return Ok(Self::url_search_params_pairs_from_object_entries(&entries)
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>());
                }
                let length_value =
                    Self::object_get_entry(&entries, "length").unwrap_or(Value::Number(0));
                let length = Self::to_non_negative_usize(&length_value, "array-like length")?;
                let mut out = Vec::with_capacity(length);
                for index in 0..length {
                    let key = index.to_string();
                    out.push(Self::object_get_entry(&entries, &key).unwrap_or(Value::Undefined));
                }
                Ok(out)
            }
            _ => Err(Error::ScriptRuntime(
                "expected an array-like or iterable source".into(),
            )),
        }
    }

    pub(crate) fn array_like_values_from_value_with_live_properties(
        &mut self,
        value: &Value,
    ) -> Result<Vec<Value>> {
        match value {
            Value::Object(entries) => {
                let is_iterator = {
                    let entries_ref = entries.borrow();
                    Self::is_iterator_object(&entries_ref)
                };
                if is_iterator {
                    return self.iterator_collect_remaining_values(entries);
                }
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    return Ok(Self::url_search_params_pairs_from_object_entries(&entries)
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>());
                }
                drop(entries);
                let length_value = self.object_property_from_value(value, "length")?;
                let length = Self::to_non_negative_usize(&length_value, "array-like length")?;
                let mut out = Vec::with_capacity(length);
                for index in 0..length {
                    out.push(self.object_property_from_value(value, &index.to_string())?);
                }
                Ok(out)
            }
            _ => self.array_like_values_from_value(value),
        }
    }
}
