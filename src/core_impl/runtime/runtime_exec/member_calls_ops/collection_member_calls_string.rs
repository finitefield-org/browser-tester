use super::*;

impl Harness {
    pub(crate) fn eval_string_member_call(
        &mut self,
        text: &str,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        if let Some(value) = self.try_eval_string_basic_member_call(text, member, evaluated_args)? {
            return Ok(Some(value));
        }
        if let Some(value) =
            self.try_eval_string_search_member_call(text, member, evaluated_args, event)?
        {
            return Ok(Some(value));
        }
        self.try_eval_string_transform_member_call(text, member, evaluated_args, event)
    }
}
