use super::*;

impl Harness {
    pub(crate) fn current_intl_constructor_value(
        &mut self,
        constructor_name: &str,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<(Value, Value)> {
        let intl_value = self.eval_expr(&Expr::Var("Intl".to_string()), env, event_param, event)?;
        let constructor = self.object_property_from_value(&intl_value, constructor_name)?;
        Ok((intl_value, constructor))
    }

    pub(crate) fn eval_overridden_intl_constructor_call(
        &mut self,
        intl_value: &Value,
        constructor: &Value,
        args: &[Value],
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Value> {
        if called_with_new {
            self.execute_constructor_value_with_env(constructor, args, event, Some(env))
        } else {
            self.execute_callable_value_with_this_and_env(
                constructor,
                args,
                event,
                Some(env),
                Some(intl_value.clone()),
            )
        }
    }

    fn eval_builtin_intl_formatter_construct(
        &mut self,
        kind: IntlFormatterKind,
        locales: Option<&Expr>,
        options: Option<&Expr>,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let requested_locales = if let Some(locales) = locales {
            let value = self.eval_expr(locales, env, event_param, event)?;
            self.intl_collect_locales(&value)?
        } else {
            Vec::new()
        };
        let locale = Self::intl_select_locale_for_formatter(kind, &requested_locales);
        match kind {
            IntlFormatterKind::Collator => {
                let options = options
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                let (case_first, sensitivity, numeric) =
                    self.intl_collator_options_from_value(options.as_ref())?;
                Ok(self.new_intl_collator_value(locale, case_first, sensitivity, numeric))
            }
            IntlFormatterKind::DateTimeFormat => {
                let options = options
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                let options = self.intl_date_time_options_from_value(&locale, options.as_ref())?;
                Ok(self.new_intl_date_time_formatter_value(locale, options))
            }
            IntlFormatterKind::NumberFormat => {
                let options = options
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                let options =
                    self.intl_number_format_options_from_value(&locale, options.as_ref())?;
                Ok(self.new_intl_number_formatter_value(locale, options))
            }
            IntlFormatterKind::DisplayNames => {
                let options = options
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                let options = self.intl_display_names_options_from_value(options.as_ref())?;
                Ok(self.new_intl_display_names_value(locale, options))
            }
            IntlFormatterKind::DurationFormat => {
                let options = options
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                let options = self.intl_duration_options_from_value(options.as_ref())?;
                Ok(self.new_intl_duration_formatter_value(locale, options))
            }
            IntlFormatterKind::ListFormat => {
                let options = options
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                let options = self.intl_list_options_from_value(options.as_ref())?;
                Ok(self.new_intl_list_formatter_value(locale, options))
            }
            IntlFormatterKind::PluralRules => {
                let options = options
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                let options = self.intl_plural_rules_options_from_value(options.as_ref())?;
                Ok(self.new_intl_plural_rules_value(locale, options))
            }
            IntlFormatterKind::RelativeTimeFormat => {
                let options = options
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                let options =
                    self.intl_relative_time_options_from_value(&locale, options.as_ref())?;
                Ok(self.new_intl_relative_time_formatter_value(locale, options))
            }
            IntlFormatterKind::Segmenter => {
                let options = options
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                let options = self.intl_segmenter_options_from_value(options.as_ref())?;
                Ok(self.new_intl_segmenter_value(locale, options))
            }
        }
    }

    fn eval_intl_formatter_construct(
        &mut self,
        kind: IntlFormatterKind,
        locales: Option<&Expr>,
        options: Option<&Expr>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let (intl_value, constructor) =
            self.current_intl_constructor_value(kind.storage_name(), env, event_param, event)?;
        if Self::is_builtin_placeholder_value(&constructor) {
            return self.eval_builtin_intl_formatter_construct(
                kind,
                locales,
                options,
                env,
                event_param,
                event,
            );
        }

        let mut args = Vec::with_capacity(2);
        if let Some(locales) = locales {
            args.push(self.eval_expr(locales, env, event_param, event)?);
        }
        if let Some(options) = options {
            args.push(self.eval_expr(options, env, event_param, event)?);
        }

        self.eval_overridden_intl_constructor_call(
            &intl_value,
            &constructor,
            &args,
            called_with_new,
            env,
            event,
        )
    }

    pub(crate) fn try_eval_core_intl_exprs(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::IntlFormatterConstruct {
                    kind,
                    locales,
                    options,
                    called_with_new,
                } => self.eval_intl_formatter_construct(
                    *kind,
                    locales.as_deref(),
                    options.as_deref(),
                    *called_with_new,
                    env,
                    event_param,
                    event,
                ),
                Expr::IntlFormat { formatter, value } => {
                    let formatter = self.eval_expr(formatter, env, event_param, event)?;
                    let (kind, locale) = self.resolve_intl_formatter(&formatter)?;
                    let value = if let Some(value) = value {
                        self.eval_expr(value, env, event_param, event)?
                    } else {
                        Value::Undefined
                    };
                    match kind {
                        IntlFormatterKind::NumberFormat => {
                            let (locale, options) =
                                self.resolve_intl_number_format_options(&formatter)?;
                            Ok(Value::String(self.intl_format_number_value_with_options(
                                &value, &locale, &options,
                            )))
                        }
                        IntlFormatterKind::DateTimeFormat => {
                            let (_, options) = self.resolve_intl_date_time_options(&formatter)?;
                            let timestamp_ms = if matches!(value, Value::Undefined) {
                                self.scheduler.now_ms
                            } else {
                                self.coerce_date_timestamp_ms(&value)
                            };
                            Ok(Value::String(self.intl_format_date_time(
                                timestamp_ms,
                                &locale,
                                &options,
                            )))
                        }
                        IntlFormatterKind::DisplayNames => Err(Error::ScriptRuntime(
                            "Intl.DisplayNames does not support format()".into(),
                        )),
                        IntlFormatterKind::DurationFormat => {
                            let (_, options) = self.resolve_intl_duration_options(&formatter)?;
                            Ok(Value::String(
                                self.intl_format_duration(&locale, &options, &value)?,
                            ))
                        }
                        IntlFormatterKind::ListFormat => {
                            let (_, options) = self.resolve_intl_list_options(&formatter)?;
                            Ok(Value::String(
                                self.intl_format_list(&locale, &options, &value)?,
                            ))
                        }
                        IntlFormatterKind::PluralRules => Err(Error::ScriptRuntime(
                            "Intl.PluralRules does not support format()".into(),
                        )),
                        IntlFormatterKind::RelativeTimeFormat => Err(Error::ScriptRuntime(
                            "Intl.RelativeTimeFormat.format requires value and unit arguments"
                                .into(),
                        )),
                        IntlFormatterKind::Segmenter => Err(Error::ScriptRuntime(
                            "Intl.Segmenter does not support format()".into(),
                        )),
                        IntlFormatterKind::Collator => Err(Error::ScriptRuntime(
                            "Intl.Collator does not support format()".into(),
                        )),
                    }
                }
                Expr::IntlFormatGetter { formatter } => {
                    let formatter = self.eval_expr(formatter, env, event_param, event)?;
                    let (kind, locale) = self.resolve_intl_formatter(&formatter)?;
                    match kind {
                        IntlFormatterKind::DateTimeFormat => {
                            self.intl_bound_date_time_format_callable_from_receiver(&formatter)
                        }
                        IntlFormatterKind::NumberFormat => {
                            self.intl_bound_number_format_callable_from_receiver(&formatter)
                        }
                        IntlFormatterKind::DurationFormat => {
                            let (_, options) = self.resolve_intl_duration_options(&formatter)?;
                            Ok(self.new_intl_duration_format_callable(locale, options))
                        }
                        IntlFormatterKind::ListFormat => {
                            let (_, options) = self.resolve_intl_list_options(&formatter)?;
                            Ok(self.new_intl_list_format_callable(locale, options))
                        }
                        IntlFormatterKind::PluralRules => Err(Error::ScriptRuntime(
                            "Intl.PluralRules does not support format getter".into(),
                        )),
                        IntlFormatterKind::RelativeTimeFormat => Err(Error::ScriptRuntime(
                            "Intl.RelativeTimeFormat does not support format getter".into(),
                        )),
                        IntlFormatterKind::Segmenter => Err(Error::ScriptRuntime(
                            "Intl.Segmenter does not support format getter".into(),
                        )),
                        IntlFormatterKind::DisplayNames => Err(Error::ScriptRuntime(
                            "Intl.DisplayNames does not support format getter".into(),
                        )),
                        IntlFormatterKind::Collator => Err(Error::ScriptRuntime(
                            "Intl.Collator does not support format getter".into(),
                        )),
                    }
                }
                Expr::IntlCollatorCompare {
                    collator,
                    left,
                    right,
                } => {
                    let collator = self.eval_expr(collator, env, event_param, event)?;
                    let (locale, case_first, sensitivity, numeric) =
                        self.resolve_intl_collator_options(&collator)?;
                    let left = self.eval_expr(left, env, event_param, event)?.as_string();
                    let right = self.eval_expr(right, env, event_param, event)?.as_string();
                    Ok(Value::Number(Self::intl_collator_compare_strings(
                        &left,
                        &right,
                        &locale,
                        &case_first,
                        &sensitivity,
                        numeric,
                    )))
                }
                Expr::IntlCollatorCompareGetter { collator } => {
                    let collator = self.eval_expr(collator, env, event_param, event)?;
                    self.intl_bound_compare_callable_from_receiver(&collator)
                }
                Expr::IntlDateTimeFormatToParts { formatter, value } => {
                    let formatter = self.eval_expr(formatter, env, event_param, event)?;
                    let (kind, locale) = self.resolve_intl_formatter(&formatter)?;
                    match kind {
                        IntlFormatterKind::DateTimeFormat => {
                            let (_, options) = self.resolve_intl_date_time_options(&formatter)?;
                            let value = if let Some(value) = value {
                                self.eval_expr(value, env, event_param, event)?
                            } else {
                                Value::Undefined
                            };
                            let timestamp_ms = if matches!(value, Value::Undefined) {
                                self.scheduler.now_ms
                            } else {
                                self.coerce_intl_date_time_timestamp_ms(&value)?
                            };
                            let parts =
                                self.intl_format_date_time_to_parts(timestamp_ms, &locale, &options);
                            Ok(self.intl_date_time_parts_to_value(&parts, None))
                        }
                        IntlFormatterKind::DurationFormat => {
                            let (_, options) = self.resolve_intl_duration_options(&formatter)?;
                            let value = if let Some(value) = value {
                                self.eval_expr(value, env, event_param, event)?
                            } else {
                                Value::Undefined
                            };
                            let parts = self.intl_format_duration_to_parts(&locale, &options, &value)?;
                            Ok(self.intl_date_time_parts_to_value(&parts, None))
                        }
                        IntlFormatterKind::ListFormat => {
                            let (_, options) = self.resolve_intl_list_options(&formatter)?;
                            let value = if let Some(value) = value {
                                self.eval_expr(value, env, event_param, event)?
                            } else {
                                Value::Undefined
                            };
                            let parts = self.intl_format_list_to_parts(&locale, &options, &value)?;
                            Ok(self.intl_date_time_parts_to_value(&parts, None))
                        }
                        IntlFormatterKind::NumberFormat => {
                            let (_, options) = self.resolve_intl_number_format_options(&formatter)?;
                            let value = if let Some(value) = value {
                                self.eval_expr(value, env, event_param, event)?
                            } else {
                                Value::Undefined
                            };
                            let parts =
                                self.intl_number_format_value_to_parts(&value, &locale, &options);
                            Ok(self.intl_date_time_parts_to_value(&parts, None))
                        }
                        _ => Err(Error::ScriptRuntime(
                            "Intl formatter formatToParts requires an Intl.DateTimeFormat, Intl.DurationFormat, Intl.ListFormat, or Intl.NumberFormat instance"
                                .into(),
                        )),
                    }
                }
                Expr::IntlDateTimeFormatRange {
                    formatter,
                    start,
                    end,
                } => {
                    let formatter = self.eval_expr(formatter, env, event_param, event)?;
                    let (kind, locale) = self.resolve_intl_formatter(&formatter)?;
                    let start = self.eval_expr(start, env, event_param, event)?;
                    let end = self.eval_expr(end, env, event_param, event)?;
                    match kind {
                        IntlFormatterKind::DateTimeFormat => {
                            let (_, options) = self.resolve_intl_date_time_options(&formatter)?;
                            let start_ms = self.coerce_date_timestamp_ms(&start);
                            let end_ms = self.coerce_date_timestamp_ms(&end);
                            Ok(Value::String(self.intl_format_date_time_range(
                                start_ms, end_ms, &locale, &options,
                            )))
                        }
                        IntlFormatterKind::NumberFormat => {
                            let (_, options) = self.resolve_intl_number_format_options(&formatter)?;
                            let start = Self::coerce_number_for_global(&start);
                            let end = Self::coerce_number_for_global(&end);
                            Ok(Value::String(self.intl_format_number_range(
                                start, end, &locale, &options,
                            )))
                        }
                        _ => Err(Error::ScriptRuntime(
                            "Intl formatter formatRange requires an Intl.DateTimeFormat or Intl.NumberFormat instance"
                                .into(),
                        )),
                    }
                }
                Expr::IntlDateTimeFormatRangeToParts {
                    formatter,
                    start,
                    end,
                } => {
                    let formatter = self.eval_expr(formatter, env, event_param, event)?;
                    let (kind, locale) = self.resolve_intl_formatter(&formatter)?;
                    let start = self.eval_expr(start, env, event_param, event)?;
                    let end = self.eval_expr(end, env, event_param, event)?;
                    match kind {
                        IntlFormatterKind::DateTimeFormat => {
                            let (_, options) = self.resolve_intl_date_time_options(&formatter)?;
                            let start_ms = self.coerce_date_timestamp_ms(&start);
                            let end_ms = self.coerce_date_timestamp_ms(&end);
                            let (parts, sources) = self.intl_format_date_time_range_to_parts(
                                start_ms, end_ms, &locale, &options,
                            );
                            Ok(self.intl_date_time_parts_to_value(&parts, Some(&sources)))
                        }
                        IntlFormatterKind::NumberFormat => {
                            let (_, options) = self.resolve_intl_number_format_options(&formatter)?;
                            let start = Self::coerce_number_for_global(&start);
                            let end = Self::coerce_number_for_global(&end);
                            let (parts, sources) = self
                                .intl_format_number_range_to_parts(start, end, &locale, &options);
                            Ok(self.intl_date_time_parts_to_value(&parts, Some(&sources)))
                        }
                        _ => Err(Error::ScriptRuntime(
                            "Intl formatter formatRangeToParts requires an Intl.DateTimeFormat or Intl.NumberFormat instance"
                                .into(),
                        )),
                    }
                }
                Expr::IntlDateTimeResolvedOptions { formatter } => {
                    let formatter = self.eval_expr(formatter, env, event_param, event)?;
                    let (kind, locale) = self.resolve_intl_formatter(&formatter)?;
                    match kind {
                        IntlFormatterKind::DateTimeFormat => {
                            let (_, options) = self.resolve_intl_date_time_options(&formatter)?;
                            Ok(self.intl_date_time_resolved_options_value(locale, &options))
                        }
                        IntlFormatterKind::Collator => {
                            let (locale, case_first, sensitivity, numeric) =
                                self.resolve_intl_collator_options(&formatter)?;
                            Ok(Self::new_object_value(vec![
                                ("locale".into(), Value::String(locale)),
                                ("usage".into(), Value::String("sort".to_string())),
                                ("sensitivity".into(), Value::String(sensitivity)),
                                ("ignorePunctuation".into(), Value::Bool(false)),
                                ("collation".into(), Value::String("default".to_string())),
                                ("numeric".into(), Value::Bool(numeric)),
                                ("caseFirst".into(), Value::String(case_first)),
                            ]))
                        }
                        IntlFormatterKind::DisplayNames => {
                            let (_, options) =
                                self.resolve_intl_display_names_options(&formatter)?;
                            Ok(self.intl_display_names_resolved_options_value(locale, &options))
                        }
                        IntlFormatterKind::DurationFormat => {
                            let (_, options) = self.resolve_intl_duration_options(&formatter)?;
                            Ok(self.intl_duration_resolved_options_value(locale, &options))
                        }
                        IntlFormatterKind::ListFormat => {
                            let (_, options) = self.resolve_intl_list_options(&formatter)?;
                            Ok(self.intl_list_resolved_options_value(locale, &options))
                        }
                        IntlFormatterKind::PluralRules => {
                            let (_, options) =
                                self.resolve_intl_plural_rules_options(&formatter)?;
                            Ok(self.intl_plural_rules_resolved_options_value(locale, &options))
                        }
                        IntlFormatterKind::RelativeTimeFormat => {
                            let (_, options) =
                                self.resolve_intl_relative_time_options(&formatter)?;
                            Ok(self.intl_relative_time_resolved_options_value(locale, &options))
                        }
                        IntlFormatterKind::Segmenter => {
                            let (_, options) = self.resolve_intl_segmenter_options(&formatter)?;
                            Ok(self.intl_segmenter_resolved_options_value(locale, &options))
                        }
                        IntlFormatterKind::NumberFormat => {
                            let (_, options) =
                                self.resolve_intl_number_format_options(&formatter)?;
                            Ok(self.intl_number_resolved_options_value(locale, &options))
                        }
                    }
                }
                Expr::IntlDisplayNamesOf {
                    display_names,
                    code,
                } => {
                    let display_names = self.eval_expr(display_names, env, event_param, event)?;
                    let code = self.eval_expr(code, env, event_param, event)?.as_string();
                    let (locale, options) =
                        self.resolve_intl_display_names_options(&display_names)?;
                    self.intl_display_names_of(&locale, &options, &code)
                }
                Expr::IntlPluralRulesSelect {
                    plural_rules,
                    value,
                } => {
                    let plural_rules = self.eval_expr(plural_rules, env, event_param, event)?;
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let (locale, options) =
                        self.resolve_intl_plural_rules_options(&plural_rules)?;
                    Ok(Value::String(
                        self.intl_plural_rules_select(&locale, &options, &value),
                    ))
                }
                Expr::IntlPluralRulesSelectRange {
                    plural_rules,
                    start,
                    end,
                } => {
                    let plural_rules = self.eval_expr(plural_rules, env, event_param, event)?;
                    let start = self.eval_expr(start, env, event_param, event)?;
                    let end = self.eval_expr(end, env, event_param, event)?;
                    let (locale, options) =
                        self.resolve_intl_plural_rules_options(&plural_rules)?;
                    Ok(Value::String(self.intl_plural_rules_select_range(
                        &locale, &options, &start, &end,
                    )))
                }
                Expr::IntlRelativeTimeFormat {
                    formatter,
                    value,
                    unit,
                } => {
                    let formatter = self.eval_expr(formatter, env, event_param, event)?;
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let unit = self.eval_expr(unit, env, event_param, event)?;
                    match self.resolve_intl_relative_time_options(&formatter) {
                        Ok((locale, options)) => Ok(Value::String(
                            self.intl_format_relative_time(&locale, &options, &value, &unit)?,
                        )),
                        Err(Error::ScriptRuntime(message))
                            if message
                                == "Intl.RelativeTimeFormat method requires an Intl.RelativeTimeFormat instance" =>
                        {
                            let callee = self.object_property_from_value(&formatter, "format")?;
                            let result = self.execute_callable_value_with_this_and_env(
                                &callee,
                                &[value, unit],
                                event,
                                Some(env),
                                Some(formatter),
                            )?;
                            self.sync_listener_capture_env_if_shared(env);
                            Ok(result)
                        }
                        Err(other) => Err(other),
                    }
                }
                Expr::IntlRelativeTimeFormatToParts {
                    formatter,
                    value,
                    unit,
                } => {
                    let formatter = self.eval_expr(formatter, env, event_param, event)?;
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let unit = self.eval_expr(unit, env, event_param, event)?;
                    match self.resolve_intl_relative_time_options(&formatter) {
                        Ok((locale, options)) => {
                            let parts = self.intl_format_relative_time_to_parts(
                                &locale, &options, &value, &unit,
                            )?;
                            Ok(self.intl_relative_time_parts_to_value(&parts))
                        }
                        Err(Error::ScriptRuntime(message))
                            if message
                                == "Intl.RelativeTimeFormat method requires an Intl.RelativeTimeFormat instance" =>
                        {
                            let callee =
                                self.object_property_from_value(&formatter, "formatToParts")?;
                            let result = self.execute_callable_value_with_this_and_env(
                                &callee,
                                &[value, unit],
                                event,
                                Some(env),
                                Some(formatter),
                            )?;
                            self.sync_listener_capture_env_if_shared(env);
                            Ok(result)
                        }
                        Err(other) => Err(other),
                    }
                }
                Expr::IntlSegmenterSegment { segmenter, value } => {
                    let segmenter = self.eval_expr(segmenter, env, event_param, event)?;
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let (locale, options) = self.resolve_intl_segmenter_options(&segmenter)?;
                    let input = value.as_string();
                    let segments = self.intl_segment_input(&locale, &options, &input);
                    Ok(self.new_intl_segments_value(segments))
                }
                Expr::IntlConstruct { .. } => {
                    Err(Error::ScriptRuntime("Intl is not a constructor".into()))
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
