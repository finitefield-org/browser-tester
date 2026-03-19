use super::*;

impl Harness {
    pub(crate) fn try_eval_core_date_exprs(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::String(value) => Ok(Value::String(value.clone())),
                Expr::Bool(value) => Ok(Value::Bool(*value)),
                Expr::Null => Ok(Value::Null),
                Expr::Undefined => Ok(Value::Undefined),
                Expr::Number(value) => Ok(Value::Number(*value)),
                Expr::Float(value) => Ok(Value::Float(*value)),
                Expr::BigInt(value) => Ok(Value::BigInt(value.clone())),
                Expr::DateNow => Ok(Value::Number(self.scheduler.now_ms)),
                Expr::PerformanceNow => Ok(Value::Float(self.scheduler.now_ms as f64)),
                Expr::DateNew { args } => {
                    let timestamp_ms = if args.is_empty() {
                        self.scheduler.now_ms
                    } else if args.len() == 1 {
                        let value = self.eval_expr(&args[0], env, event_param, event)?;
                        self.coerce_date_timestamp_ms(&value)
                    } else {
                        let mut values = Vec::with_capacity(args.len());
                        for arg in args {
                            let value = self.eval_expr(arg, env, event_param, event)?;
                            values.push(Self::value_to_i64(&value));
                        }

                        let mut year = values.first().copied().unwrap_or(0);
                        if (0..=99).contains(&year) {
                            year += 1900;
                        }
                        let month = values.get(1).copied().unwrap_or(0);
                        let day = values.get(2).copied().unwrap_or(1);
                        let hour = values.get(3).copied().unwrap_or(0);
                        let minute = values.get(4).copied().unwrap_or(0);
                        let second = values.get(5).copied().unwrap_or(0);
                        let millisecond = values.get(6).copied().unwrap_or(0);

                        Self::utc_timestamp_ms_from_components(
                            year,
                            month,
                            day,
                            hour,
                            minute,
                            second,
                            millisecond,
                        )
                    };
                    Ok(Self::new_date_value(timestamp_ms))
                }
                Expr::DateParse(value) => {
                    let value = self.eval_expr(value, env, event_param, event)?.as_string();
                    if let Some(timestamp_ms) = Self::parse_date_string_to_epoch_ms(&value) {
                        Ok(Value::Number(timestamp_ms))
                    } else {
                        Ok(Value::Float(f64::NAN))
                    }
                }
                Expr::DateUtc { args } => {
                    let mut values = Vec::with_capacity(args.len());
                    for arg in args {
                        let value = self.eval_expr(arg, env, event_param, event)?;
                        values.push(Self::value_to_i64(&value));
                    }

                    let mut year = values.first().copied().unwrap_or(0);
                    if (0..=99).contains(&year) {
                        year += 1900;
                    }
                    let month = values.get(1).copied().unwrap_or(0);
                    let day = values.get(2).copied().unwrap_or(1);
                    let hour = values.get(3).copied().unwrap_or(0);
                    let minute = values.get(4).copied().unwrap_or(0);
                    let second = values.get(5).copied().unwrap_or(0);
                    let millisecond = values.get(6).copied().unwrap_or(0);

                    Ok(Value::Number(Self::utc_timestamp_ms_from_components(
                        year,
                        month,
                        day,
                        hour,
                        minute,
                        second,
                        millisecond,
                    )))
                }
                Expr::DateGetTime(target) => {
                    let date = self.resolve_date_from_env(env, target)?;
                    Ok(Value::Number(*date.borrow()))
                }
                Expr::DateSetTime { target, value } => {
                    let date = self.resolve_date_from_env(env, target)?;
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let timestamp_ms = Self::value_to_i64(&value);
                    *date.borrow_mut() = timestamp_ms;
                    Ok(Value::Number(timestamp_ms))
                }
                Expr::DateToIsoString(target) => {
                    let date = self.resolve_date_from_env(env, target)?;
                    Ok(Value::String(Self::format_iso_8601_utc(*date.borrow())))
                }
                Expr::DateGetUTCFullYear(target) => {
                    let date = self.resolve_date_from_env(env, target)?;
                    let (year, ..) = Self::date_components_utc(*date.borrow());
                    Ok(Value::Number(year))
                }
                Expr::DateGetFullYear(target) => {
                    let date = self.resolve_date_from_env(env, target)?;
                    let (year, ..) = Self::date_components_utc(*date.borrow());
                    Ok(Value::Number(year))
                }
                Expr::DateGetMonth(target) => {
                    let date = self.resolve_date_from_env(env, target)?;
                    let (_, month, ..) = Self::date_components_utc(*date.borrow());
                    Ok(Value::Number((month as i64) - 1))
                }
                Expr::DateGetDate(target) => {
                    let date = self.resolve_date_from_env(env, target)?;
                    let (_, _, day, ..) = Self::date_components_utc(*date.borrow());
                    Ok(Value::Number(day as i64))
                }
                Expr::DateGetHours(target) => {
                    let date = self.resolve_date_from_env(env, target)?;
                    let (_, _, _, hour, ..) = Self::date_components_utc(*date.borrow());
                    Ok(Value::Number(hour as i64))
                }
                Expr::DateGetMinutes(target) => {
                    let date = self.resolve_date_from_env(env, target)?;
                    let (_, _, _, _, minute, ..) = Self::date_components_utc(*date.borrow());
                    Ok(Value::Number(minute as i64))
                }
                Expr::DateGetSeconds(target) => {
                    let date = self.resolve_date_from_env(env, target)?;
                    let (_, _, _, _, _, second, _) = Self::date_components_utc(*date.borrow());
                    Ok(Value::Number(second as i64))
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
