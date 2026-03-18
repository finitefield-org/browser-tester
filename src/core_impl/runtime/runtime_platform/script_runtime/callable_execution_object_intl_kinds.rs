use super::*;

impl Harness {
    pub(crate) fn execute_object_callable_intl_kind(
        &mut self,
        kind: &str,
        callable: &Value,
        args: &[Value],
        this_arg: Option<&Value>,
    ) -> Result<Option<Value>> {
        let value = match kind {
            "intl_collator_get_compare" => Some(self.intl_bound_compare_callable_from_receiver(
                this_arg.ok_or_else(|| {
                    Error::ScriptRuntime(
                        "Intl.Collator.compare requires an Intl.Collator instance".into(),
                    )
                })?,
            )?),
            "intl_date_time_format_get_format" => {
                Some(self.intl_bound_date_time_format_callable_from_receiver(
                    this_arg.ok_or_else(|| {
                        Error::ScriptRuntime(
                            "Intl.DateTimeFormat method requires an Intl.DateTimeFormat instance"
                                .into(),
                        )
                    })?,
                )?)
            }
            "intl_number_format_get_format" => Some(
                self.intl_bound_number_format_callable_from_receiver(this_arg.ok_or_else(
                    || {
                        Error::ScriptRuntime(
                            "Intl.NumberFormat method requires an Intl.NumberFormat instance"
                                .into(),
                        )
                    },
                )?)?,
            ),
            "intl_collator_compare" => {
                let (locale, case_first, sensitivity, numeric) =
                    self.resolve_intl_collator_options(callable)?;
                let left = args
                    .first()
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .as_string();
                let right = args.get(1).cloned().unwrap_or(Value::Undefined).as_string();
                Some(Value::Number(Self::intl_collator_compare_strings(
                    &left,
                    &right,
                    &locale,
                    &case_first,
                    &sensitivity,
                    numeric,
                )))
            }
            "intl_date_time_format" => {
                let (locale, options) = self.resolve_intl_date_time_options(callable)?;
                let timestamp_ms = args
                    .first()
                    .map(|value| self.coerce_date_timestamp_ms(value))
                    .unwrap_or(self.scheduler.now_ms);
                Some(Value::String(self.intl_format_date_time(
                    timestamp_ms,
                    &locale,
                    &options,
                )))
            }
            "intl_duration_format" => {
                let (locale, options) = self.resolve_intl_duration_options(callable)?;
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Some(Value::String(
                    self.intl_format_duration(&locale, &options, &value)?,
                ))
            }
            "intl_list_format" => {
                let (locale, options) = self.resolve_intl_list_options(callable)?;
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Some(Value::String(
                    self.intl_format_list(&locale, &options, &value)?,
                ))
            }
            "intl_number_format" => {
                let (locale, options) = self.resolve_intl_number_format_options(callable)?;
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Some(Value::String(self.intl_format_number_value_with_options(
                    &value, &locale, &options,
                )))
            }
            _ => None,
        };
        Ok(value)
    }
}
