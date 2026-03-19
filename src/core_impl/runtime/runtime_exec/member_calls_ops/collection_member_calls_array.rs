use super::*;

impl Harness {
    pub(crate) fn eval_array_member_call(
        &mut self,
        values: &Rc<RefCell<ArrayValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Option<Value>> {
        {
            let values_ref = values.borrow();
            if Self::is_data_transfer_item_list_value(&values_ref)
                && matches!(member, "add" | "remove" | "clear")
            {
                let own_override = Self::object_get_entry(&values_ref.properties, member)
                    .is_some_and(|value| !Self::is_builtin_placeholder_value(&value));
                let builtin_deleted =
                    Self::is_builtin_object_property_deleted(&values_ref.properties, member);
                if own_override || builtin_deleted {
                    return Ok(None);
                }
            }
        }

        if member == "item" && Self::is_dom_rect_list_value(&values.borrow()) {
            if evaluated_args.len() > 1 {
                return Err(Error::ScriptRuntime(
                    "item requires zero or one argument".into(),
                ));
            }
            let index = evaluated_args
                .first()
                .map(Self::value_to_i64)
                .unwrap_or(0)
                .max(0) as usize;
            let value = values.borrow().get(index).cloned().unwrap_or(Value::Null);
            return Ok(Some(value));
        }

        if let Some(value) = self.try_eval_array_member_call_callbacks(
            values,
            member,
            evaluated_args,
            event,
            caller_env,
        )? {
            return Ok(Some(value));
        }

        if let Some(value) =
            self.try_eval_array_member_call_sequence(values, member, evaluated_args)?
        {
            return Ok(Some(value));
        }

        if let Some(value) =
            self.try_eval_array_member_call_clipboard_items(values, member, evaluated_args)?
        {
            return Ok(Some(value));
        }

        if let Some(value) =
            self.try_eval_array_member_call_mutations(values, member, evaluated_args, event)?
        {
            return Ok(Some(value));
        }

        Ok(None)
    }
}
