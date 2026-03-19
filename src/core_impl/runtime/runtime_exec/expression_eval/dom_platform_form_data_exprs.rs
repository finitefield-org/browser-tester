use super::*;

impl Harness {
    fn form_data_member_fallback_expr(
        source: &FormDataSource,
        member: &str,
        name: &str,
    ) -> Option<Expr> {
        match source {
            FormDataSource::Var(var_name) => Some(Expr::MemberCall {
                target: Box::new(Expr::Var(var_name.clone())),
                member: member.to_string(),
                args: vec![Expr::String(name.to_string())],
                optional: false,
                optional_call: false,
            }),
            FormDataSource::New { .. } => None,
        }
    }

    fn form_data_member_fallback_is_lookup_miss(err: &Error, member: &str) -> bool {
        match err {
            Error::ScriptRuntime(msg) => {
                msg == &format!("'{member}' is not a function")
                    || msg == &format!("member call target does not support property '{member}'")
            }
            _ => false,
        }
    }

    fn eval_form_data_member_expr_with_fallback(
        &mut self,
        source: &FormDataSource,
        member: &str,
        name: &str,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match self.eval_form_data_source(source, env) {
            Ok(entries) => match member {
                "get" => Ok(entries
                    .iter()
                    .find_map(|(entry_name, value)| {
                        (entry_name == name).then(|| Value::String(value.clone()))
                    })
                    .unwrap_or(Value::Null)),
                "has" => Ok(Value::Bool(
                    entries.iter().any(|(entry_name, _)| entry_name == name),
                )),
                "getAll" => Ok(Self::new_array_value(
                    entries
                        .iter()
                        .filter(|(entry_name, _)| entry_name == name)
                        .map(|(_, value)| Value::String(value.clone()))
                        .collect(),
                )),
                _ => Err(Error::ScriptRuntime(format!(
                    "unsupported FormData expression fallback: {member}"
                ))),
            },
            Err(form_data_err) => {
                let Some(fallback_expr) =
                    Self::form_data_member_fallback_expr(source, member, name)
                else {
                    return Err(form_data_err);
                };
                match self.eval_expr(&fallback_expr, env, event_param, event) {
                    Ok(value) => Ok(value),
                    Err(err) if Self::form_data_member_fallback_is_lookup_miss(&err, member) => {
                        Err(form_data_err)
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }

    fn eval_form_data_get_all_length_expr_with_fallback(
        &mut self,
        source: &FormDataSource,
        name: &str,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match self.eval_form_data_source(source, env) {
            Ok(entries) => Ok(Value::Number(
                entries
                    .iter()
                    .filter(|(entry_name, _)| entry_name == name)
                    .count() as i64,
            )),
            Err(form_data_err) => {
                let Some(fallback_expr) =
                    Self::form_data_member_fallback_expr(source, "getAll", name)
                else {
                    return Err(form_data_err);
                };
                match self.eval_expr(&fallback_expr, env, event_param, event) {
                    Ok(value) => self.object_property_from_value(&value, "length"),
                    Err(err) if Self::form_data_member_fallback_is_lookup_miss(&err, "getAll") => {
                        Err(form_data_err)
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }

    pub(crate) fn try_eval_dom_form_data_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match expr {
            Expr::FormDataNew { form, submitter } => Value::FormData(Rc::new(RefCell::new(
                self.eval_form_data_constructor_entries(form.as_ref(), submitter.as_ref(), env)?,
            ))),
            Expr::FormDataGet { source, name } => {
                return self
                    .eval_form_data_member_expr_with_fallback(
                        source,
                        "get",
                        name,
                        env,
                        event_param,
                        event,
                    )
                    .map(Some);
            }
            Expr::FormDataHas { source, name } => {
                return self
                    .eval_form_data_member_expr_with_fallback(
                        source,
                        "has",
                        name,
                        env,
                        event_param,
                        event,
                    )
                    .map(Some);
            }
            Expr::FormDataGetAll { source, name } => {
                return self
                    .eval_form_data_member_expr_with_fallback(
                        source,
                        "getAll",
                        name,
                        env,
                        event_param,
                        event,
                    )
                    .map(Some);
            }
            Expr::FormDataGetAllLength { source, name } => {
                return self
                    .eval_form_data_get_all_length_expr_with_fallback(
                        source,
                        name,
                        env,
                        event_param,
                        event,
                    )
                    .map(Some);
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
