use super::*;

impl Harness {
    pub(crate) fn arguments_param_indexes_for_name(
        env: &HashMap<String, Value>,
        name: &str,
    ) -> Vec<usize> {
        let Some(Value::Array(bindings)) = env.get(INTERNAL_ARGUMENTS_PARAM_BINDINGS_KEY) else {
            return Vec::new();
        };
        bindings
            .borrow()
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                matches!(entry, Value::String(param) if param == name).then_some(index)
            })
            .collect()
    }

    pub(crate) fn sync_arguments_after_param_write(
        &mut self,
        env: &mut HashMap<String, Value>,
        name: &str,
        value: &Value,
    ) {
        let indexes = Self::arguments_param_indexes_for_name(env, name);
        if indexes.is_empty() {
            return;
        }
        let Some(Value::Array(arguments)) = env.get("arguments").cloned() else {
            return;
        };
        let mut args_ref = arguments.borrow_mut();
        for index in indexes {
            if index < args_ref.len() {
                args_ref[index] = value.clone();
            }
        }
    }
}
