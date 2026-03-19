use super::*;

impl Harness {
    pub(crate) fn match_media_matches_for_query(&self, query: &str) -> bool {
        self.platform_mocks
            .match_media_mocks
            .get(query)
            .copied()
            .unwrap_or(self.platform_mocks.default_match_media_matches)
    }

    pub(crate) fn new_match_media_query_list_value(query: &str) -> Value {
        let mut entries = vec![
            (INTERNAL_MATCH_MEDIA_OBJECT_KEY.into(), Value::Bool(true)),
            (
                INTERNAL_MATCH_MEDIA_QUERY_KEY.into(),
                Value::String(query.to_string()),
            ),
            (INTERNAL_EVENT_TARGET_OBJECT_KEY.into(), Value::Bool(true)),
            ("onchange".into(), Value::Null),
            (
                "addEventListener".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "removeEventListener".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "dispatchEvent".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "addListener".into(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "removeListener".into(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        Self::mark_object_properties_non_enumerable(
            &mut entries,
            &[
                "addEventListener",
                "removeEventListener",
                "dispatchEvent",
                "addListener",
                "removeListener",
            ],
        );
        Self::new_object_value(entries)
    }

    pub(crate) fn eval_match_media_call_with_query(&mut self, query: String) -> Value {
        self.platform_mocks.match_media_calls.push(query.clone());
        Self::new_match_media_query_list_value(&query)
    }

    pub(crate) fn eval_match_media_call_from_values(&mut self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(Error::ScriptRuntime(
                "matchMedia requires exactly one argument".into(),
            ));
        }
        Ok(self.eval_match_media_call_with_query(args[0].as_string()))
    }
}
