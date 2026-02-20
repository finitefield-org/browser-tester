impl Harness {
    pub(crate) fn is_url_search_params_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_URL_SEARCH_PARAMS_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn url_search_params_pairs_to_value(pairs: &[(String, String)]) -> Value {
        Self::new_array_value(
            pairs
                .iter()
                .map(|(name, value)| {
                    Self::new_array_value(vec![
                        Value::String(name.clone()),
                        Value::String(value.clone()),
                    ])
                })
                .collect::<Vec<_>>(),
        )
    }

    pub(crate) fn url_search_params_pairs_from_object_entries(
        entries: &[(String, Value)],
    ) -> Vec<(String, String)> {
        let Some(Value::Array(list)) =
            Self::object_get_entry(entries, INTERNAL_URL_SEARCH_PARAMS_ENTRIES_KEY)
        else {
            return Vec::new();
        };
        let snapshot = list.borrow().clone();
        let mut pairs = Vec::new();
        for item in snapshot {
            let Value::Array(pair) = item else {
                continue;
            };
            let pair = pair.borrow();
            if pair.is_empty() {
                continue;
            }
            let name = pair[0].as_string();
            let value = pair.get(1).map(Value::as_string).unwrap_or_default();
            pairs.push((name, value));
        }
        pairs
    }

    pub(crate) fn set_url_search_params_pairs(
        entries: &mut impl ObjectEntryMut,
        pairs: &[(String, String)],
    ) {
        Self::object_set_entry(
            entries,
            INTERNAL_URL_SEARCH_PARAMS_ENTRIES_KEY.to_string(),
            Self::url_search_params_pairs_to_value(pairs),
        );
    }

    pub(crate) fn new_url_search_params_value(
        &self,
        pairs: Vec<(String, String)>,
        owner_id: Option<usize>,
    ) -> Value {
        let mut entries = vec![(
            INTERNAL_URL_SEARCH_PARAMS_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )];
        if let Some(owner_id) = owner_id {
            entries.push((
                INTERNAL_URL_SEARCH_PARAMS_OWNER_ID_KEY.to_string(),
                Value::Number(owner_id as i64),
            ));
        }
        Self::set_url_search_params_pairs(&mut entries, &pairs);
        Self::new_object_value(entries)
    }

    pub(crate) fn resolve_url_search_params_object_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<ObjectValue>>> {
        match env.get(target) {
            Some(Value::Object(entries)) => {
                if Self::is_url_search_params_object(&entries.borrow()) {
                    Ok(entries.clone())
                } else {
                    Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a URLSearchParams",
                        target
                    )))
                }
            }
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a URLSearchParams",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(crate) fn url_search_params_pairs_from_init_value(
        &self,
        init: &Value,
    ) -> Result<Vec<(String, String)>> {
        match init {
            Value::Undefined | Value::Null => Ok(Vec::new()),
            Value::String(text) => parse_url_search_params_pairs_from_query_string(text),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    Ok(Self::url_search_params_pairs_from_object_entries(&entries))
                } else {
                    let mut pairs = Vec::new();
                    for (name, value) in entries.iter() {
                        if Self::is_internal_object_key(name) {
                            continue;
                        }
                        pairs.push((name.clone(), value.as_string()));
                    }
                    Ok(pairs)
                }
            }
            Value::Array(_) | Value::Map(_) | Value::Set(_) | Value::TypedArray(_) => {
                let iterable = self.array_like_values_from_value(init)?;
                let mut pairs = Vec::new();
                for entry in iterable {
                    let pair = self.array_like_values_from_value(&entry).map_err(|_| {
                        Error::ScriptRuntime(
                            "URLSearchParams iterable values must be [name, value] pairs".into(),
                        )
                    })?;
                    if pair.len() < 2 {
                        return Err(Error::ScriptRuntime(
                            "URLSearchParams iterable values must be [name, value] pairs".into(),
                        ));
                    }
                    pairs.push((pair[0].as_string(), pair[1].as_string()));
                }
                Ok(pairs)
            }
            other => parse_url_search_params_pairs_from_query_string(&other.as_string()),
        }
    }

}
