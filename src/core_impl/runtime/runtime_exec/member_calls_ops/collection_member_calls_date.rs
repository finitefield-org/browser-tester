use super::*;

impl Harness {
    pub(crate) fn eval_date_member_call(
        &mut self,
        value: &Rc<RefCell<i64>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let result = match member {
            "getTime" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getTime does not take arguments".into(),
                    ));
                }
                Value::Number(*value.borrow())
            }
            "setTime" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setTime requires exactly one argument".into(),
                    ));
                }
                let timestamp_ms = Self::value_to_i64(&evaluated_args[0]);
                *value.borrow_mut() = timestamp_ms;
                Value::Number(timestamp_ms)
            }
            "toISOString" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "toISOString does not take arguments".into(),
                    ));
                }
                Value::String(Self::format_iso_8601_utc(*value.borrow()))
            }
            "toLocaleDateString" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "toLocaleDateString supports at most two arguments".into(),
                    ));
                }
                let requested_locales = evaluated_args
                    .first()
                    .map(|value| self.intl_collect_locales(value))
                    .transpose()?
                    .unwrap_or_default();
                let locale = Self::intl_select_locale_for_formatter(
                    IntlFormatterKind::DateTimeFormat,
                    &requested_locales,
                );
                let options =
                    self.intl_date_time_options_from_value(&locale, evaluated_args.get(1))?;
                Value::String(self.intl_format_date_time(*value.borrow(), &locale, &options))
            }
            "toString" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "toString does not take arguments".into(),
                    ));
                }
                Value::String(Self::format_iso_8601_utc(*value.borrow()))
            }
            "valueOf" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "valueOf does not take arguments".into(),
                    ));
                }
                Value::Number(*value.borrow())
            }
            "getUTCFullYear" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCFullYear does not take arguments".into(),
                    ));
                }
                let (year, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(year)
            }
            "getUTCMonth" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCMonth does not take arguments".into(),
                    ));
                }
                let (_, month, ..) = Self::date_components_utc(*value.borrow());
                Value::Number((month as i64) - 1)
            }
            "getUTCDate" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCDate does not take arguments".into(),
                    ));
                }
                let (_, _, day, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(day as i64)
            }
            "getUTCDay" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCDay does not take arguments".into(),
                    ));
                }
                let timestamp_ms = *value.borrow();
                let days = timestamp_ms.div_euclid(86_400_000);
                let weekday = ((days + 4).rem_euclid(7)) as i64;
                Value::Number(weekday)
            }
            "getUTCHours" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCHours does not take arguments".into(),
                    ));
                }
                let (_, _, _, hour, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(hour as i64)
            }
            "getUTCMinutes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCMinutes does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, minute, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(minute as i64)
            }
            "getUTCSeconds" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCSeconds does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, _, second, _) = Self::date_components_utc(*value.borrow());
                Value::Number(second as i64)
            }
            "getUTCMilliseconds" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getUTCMilliseconds does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, _, _, millisecond) = Self::date_components_utc(*value.borrow());
                Value::Number(millisecond as i64)
            }
            "getFullYear" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getFullYear does not take arguments".into(),
                    ));
                }
                let (year, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(year)
            }
            "getMonth" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getMonth does not take arguments".into(),
                    ));
                }
                let (_, month, ..) = Self::date_components_utc(*value.borrow());
                Value::Number((month as i64) - 1)
            }
            "getDate" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getDate does not take arguments".into(),
                    ));
                }
                let (_, _, day, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(day as i64)
            }
            "getHours" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getHours does not take arguments".into(),
                    ));
                }
                let (_, _, _, hour, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(hour as i64)
            }
            "getMinutes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getMinutes does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, minute, ..) = Self::date_components_utc(*value.borrow());
                Value::Number(minute as i64)
            }
            "getSeconds" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getSeconds does not take arguments".into(),
                    ));
                }
                let (_, _, _, _, _, second, _) = Self::date_components_utc(*value.borrow());
                Value::Number(second as i64)
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }
}
