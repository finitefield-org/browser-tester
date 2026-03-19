use super::*;

impl Harness {
    pub(crate) fn eval_window_member_call(
        &mut self,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let shadowed = {
            let entries = self.dom_runtime.window_object.borrow();
            Self::object_get_entry(&entries, member)
                .is_some_and(|value| !Self::is_builtin_placeholder_value(&value))
                || Self::is_builtin_object_property_deleted(&entries, member)
        };
        if shadowed {
            return Ok(None);
        }

        match member {
            "getSelection" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getSelection takes no arguments".into(),
                    ));
                }
                Ok(Some(self.ensure_document_selection_object()))
            }
            _ => Ok(None),
        }
    }
}
