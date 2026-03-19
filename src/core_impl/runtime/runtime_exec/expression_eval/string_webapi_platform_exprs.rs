use super::*;

impl Harness {
    pub(crate) fn try_eval_string_platform_exprs(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::StructuredClone { value, options } => {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let options = options
                        .as_ref()
                        .map(|options| self.eval_expr(options, env, event_param, event))
                        .transpose()?;
                    Self::structured_clone_value_with_options(&value, options.as_ref())
                }
                Expr::Fetch { request, options } => {
                    self.eval_fetch_call(request, options.as_deref(), env, event_param, event)
                }
                Expr::MatchMedia(query) => {
                    let query = self.eval_expr(query, env, event_param, event)?.as_string();
                    Ok(self.eval_match_media_call_with_query(query))
                }
                Expr::MatchMediaProp { query, prop } => {
                    let query = self.eval_expr(query, env, event_param, event)?.as_string();
                    self.platform_mocks.match_media_calls.push(query.clone());
                    let matches = self.match_media_matches_for_query(&query);
                    match prop {
                        MatchMediaProp::Matches => Ok(Value::Bool(matches)),
                        MatchMediaProp::Media => Ok(Value::String(query)),
                    }
                }
                Expr::Alert(message) => {
                    let message = self
                        .eval_expr(message, env, event_param, event)?
                        .as_string();
                    self.platform_mocks.alert_messages.push(message);
                    Ok(Value::Undefined)
                }
                Expr::Confirm(message) => {
                    let _ = self.eval_expr(message, env, event_param, event)?;
                    let accepted = self
                        .platform_mocks
                        .confirm_responses
                        .pop_front()
                        .unwrap_or(self.platform_mocks.default_confirm_response);
                    Ok(Value::Bool(accepted))
                }
                Expr::Prompt { message, default } => {
                    let _ = self.eval_expr(message, env, event_param, event)?;
                    let default_value = default
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| value.as_string());
                    let response = self
                        .platform_mocks
                        .prompt_responses
                        .pop_front()
                        .unwrap_or_else(|| {
                            self.platform_mocks
                                .default_prompt_response
                                .clone()
                                .or(default_value)
                        });
                    match response {
                        Some(value) => Ok(Value::String(value)),
                        None => Ok(Value::Null),
                    }
                }
                Expr::FunctionConstructor { args } => {
                    let args = args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    self.build_function_from_constructor_values(&args)
                }
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
