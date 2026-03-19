use super::*;

impl Harness {
    pub(crate) fn try_eval_dom_navigation_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match expr {
            Expr::LocationMethodCall { method, url } => match method {
                LocationMethod::Assign => {
                    let Some(url_expr) = url else {
                        return Err(Error::ScriptRuntime(
                            "location.assign requires exactly one argument".into(),
                        ));
                    };
                    let url = self
                        .eval_expr(url_expr, env, event_param, event)?
                        .as_string();
                    self.navigate_location(&url, LocationNavigationKind::Assign)?;
                    Value::Undefined
                }
                LocationMethod::Reload => {
                    self.reload_location()?;
                    Value::Undefined
                }
                LocationMethod::Replace => {
                    let Some(url_expr) = url else {
                        return Err(Error::ScriptRuntime(
                            "location.replace requires exactly one argument".into(),
                        ));
                    };
                    let url = self
                        .eval_expr(url_expr, env, event_param, event)?
                        .as_string();
                    self.navigate_location(&url, LocationNavigationKind::Replace)?;
                    Value::Undefined
                }
                LocationMethod::ToString => Value::String(self.document_url.clone()),
            },
            Expr::HistoryMethodCall { method, args } => match method {
                HistoryMethod::Back => {
                    let _ = args;
                    self.history_go_with_env(-1)?;
                    Value::Undefined
                }
                HistoryMethod::Forward => {
                    let _ = args;
                    self.history_go_with_env(1)?;
                    Value::Undefined
                }
                HistoryMethod::Go => {
                    let delta = if let Some(delta) = args.first() {
                        let value = self.eval_expr(delta, env, event_param, event)?;
                        Self::value_to_i64(&value)
                    } else {
                        0
                    };
                    self.history_go_with_env(delta)?;
                    Value::Undefined
                }
                HistoryMethod::PushState => {
                    let state = self.eval_expr(&args[0], env, event_param, event)?;
                    let url = if args.len() >= 3 {
                        Some(
                            self.eval_expr(&args[2], env, event_param, event)?
                                .as_string(),
                        )
                    } else {
                        None
                    };
                    self.history_push_state(state, url.as_deref(), false)?;
                    Value::Undefined
                }
                HistoryMethod::ReplaceState => {
                    let state = self.eval_expr(&args[0], env, event_param, event)?;
                    let url = if args.len() >= 3 {
                        Some(
                            self.eval_expr(&args[2], env, event_param, event)?
                                .as_string(),
                        )
                    } else {
                        None
                    };
                    self.history_push_state(state, url.as_deref(), true)?;
                    Value::Undefined
                }
            },
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
