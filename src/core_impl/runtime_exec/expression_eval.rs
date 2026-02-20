impl Harness {
    pub(super) fn bind_timer_id_to_task_env(&mut self, name: &str, expr: &Expr, value: &Value) {
        if !matches!(
            expr,
            Expr::SetTimeout { .. } | Expr::SetInterval { .. } | Expr::RequestAnimationFrame { .. }
        ) {
            return;
        }
        let Value::Number(timer_id) = value else {
            return;
        };
        for task in self
            .scheduler
            .task_queue
            .iter_mut()
            .filter(|task| task.id == *timer_id)
        {
            task.env.insert(name.to_string(), value.clone());
        }
    }

    pub(super) fn eval_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
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
            Expr::DateNew { value } => {
                let timestamp_ms = if let Some(value) = value {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    self.coerce_date_timestamp_ms(&value)
                } else {
                    self.scheduler.now_ms
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
            Expr::IntlFormatterConstruct {
                kind,
                locales,
                options,
                called_with_new: _called_with_new,
            } => {
                let requested_locales = if let Some(locales) = locales {
                    let value = self.eval_expr(locales, env, event_param, event)?;
                    self.intl_collect_locales(&value)?
                } else {
                    Vec::new()
                };
                let locale = Self::intl_select_locale_for_formatter(*kind, &requested_locales);
                match kind {
                    IntlFormatterKind::Collator => {
                        let options = options
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?;
                        let (case_first, sensitivity) =
                            self.intl_collator_options_from_value(options.as_ref())?;
                        Ok(self.new_intl_collator_value(locale, case_first, sensitivity))
                    }
                    IntlFormatterKind::DateTimeFormat => {
                        let options = options
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?;
                        let options =
                            self.intl_date_time_options_from_value(&locale, options.as_ref())?;
                        Ok(self.new_intl_date_time_formatter_value(locale, options))
                    }
                    IntlFormatterKind::DisplayNames => {
                        let options = options
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?;
                        let options =
                            self.intl_display_names_options_from_value(options.as_ref())?;
                        Ok(self.new_intl_display_names_value(locale, options))
                    }
                    IntlFormatterKind::DurationFormat => {
                        let options = options
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?;
                        let options = self.intl_duration_options_from_value(options.as_ref())?;
                        Ok(self.new_intl_duration_formatter_value(locale, options))
                    }
                    IntlFormatterKind::ListFormat => {
                        let options = options
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?;
                        let options = self.intl_list_options_from_value(options.as_ref())?;
                        Ok(self.new_intl_list_formatter_value(locale, options))
                    }
                    IntlFormatterKind::PluralRules => {
                        let options = options
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?;
                        let options =
                            self.intl_plural_rules_options_from_value(options.as_ref())?;
                        Ok(self.new_intl_plural_rules_value(locale, options))
                    }
                    IntlFormatterKind::RelativeTimeFormat => {
                        let options = options
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?;
                        let options =
                            self.intl_relative_time_options_from_value(options.as_ref())?;
                        Ok(self.new_intl_relative_time_formatter_value(locale, options))
                    }
                    IntlFormatterKind::Segmenter => {
                        let options = options
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?;
                        let options = self.intl_segmenter_options_from_value(options.as_ref())?;
                        Ok(self.new_intl_segmenter_value(locale, options))
                    }
                    _ => Ok(self.new_intl_formatter_value(*kind, locale)),
                }
            }
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
                        Ok(Value::String(Self::intl_format_number_for_locale(
                            Self::coerce_number_for_global(&value),
                            &locale,
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
                        "Intl.RelativeTimeFormat.format requires value and unit arguments".into(),
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
                        let (_, options) = self.resolve_intl_date_time_options(&formatter)?;
                        Ok(self.new_intl_date_time_format_callable(locale, options))
                    }
                    IntlFormatterKind::NumberFormat => {
                        Ok(self.new_intl_number_format_callable(locale))
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
                let (locale, case_first, sensitivity) =
                    self.resolve_intl_collator_options(&collator)?;
                let left = self.eval_expr(left, env, event_param, event)?.as_string();
                let right = self.eval_expr(right, env, event_param, event)?.as_string();
                Ok(Value::Number(Self::intl_collator_compare_strings(
                    &left,
                    &right,
                    &locale,
                    &case_first,
                    &sensitivity,
                )))
            }
            Expr::IntlCollatorCompareGetter { collator } => {
                let collator = self.eval_expr(collator, env, event_param, event)?;
                let (locale, case_first, sensitivity) =
                    self.resolve_intl_collator_options(&collator)?;
                Ok(self.new_intl_collator_compare_callable(locale, case_first, sensitivity))
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
                            self.coerce_date_timestamp_ms(&value)
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
                    _ => Err(Error::ScriptRuntime(
                        "Intl formatter formatToParts requires an Intl.DateTimeFormat, Intl.DurationFormat, or Intl.ListFormat instance"
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
                if kind != IntlFormatterKind::DateTimeFormat {
                    return Err(Error::ScriptRuntime(
                        "Intl.DateTimeFormat.formatRange requires an Intl.DateTimeFormat instance"
                            .into(),
                    ));
                }
                let (_, options) = self.resolve_intl_date_time_options(&formatter)?;
                let start = self.eval_expr(start, env, event_param, event)?;
                let end = self.eval_expr(end, env, event_param, event)?;
                let start_ms = self.coerce_date_timestamp_ms(&start);
                let end_ms = self.coerce_date_timestamp_ms(&end);
                Ok(Value::String(self.intl_format_date_time_range(
                    start_ms, end_ms, &locale, &options,
                )))
            }
            Expr::IntlDateTimeFormatRangeToParts {
                formatter,
                start,
                end,
            } => {
                let formatter = self.eval_expr(formatter, env, event_param, event)?;
                let (kind, locale) = self.resolve_intl_formatter(&formatter)?;
                if kind != IntlFormatterKind::DateTimeFormat {
                    return Err(Error::ScriptRuntime(
                        "Intl.DateTimeFormat.formatRangeToParts requires an Intl.DateTimeFormat instance"
                            .into(),
                    ));
                }
                let (_, options) = self.resolve_intl_date_time_options(&formatter)?;
                let start = self.eval_expr(start, env, event_param, event)?;
                let end = self.eval_expr(end, env, event_param, event)?;
                let start_ms = self.coerce_date_timestamp_ms(&start);
                let end_ms = self.coerce_date_timestamp_ms(&end);
                let (parts, sources) =
                    self.intl_format_date_time_range_to_parts(start_ms, end_ms, &locale, &options);
                Ok(self.intl_date_time_parts_to_value(&parts, Some(&sources)))
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
                        let (locale, case_first, sensitivity) =
                            self.resolve_intl_collator_options(&formatter)?;
                        Ok(Self::new_object_value(vec![
                            ("locale".into(), Value::String(locale)),
                            ("usage".into(), Value::String("sort".to_string())),
                            ("sensitivity".into(), Value::String(sensitivity)),
                            ("ignorePunctuation".into(), Value::Bool(false)),
                            ("collation".into(), Value::String("default".to_string())),
                            ("numeric".into(), Value::Bool(false)),
                            ("caseFirst".into(), Value::String(case_first)),
                        ]))
                    }
                    IntlFormatterKind::DisplayNames => {
                        let (_, options) = self.resolve_intl_display_names_options(&formatter)?;
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
                        let (_, options) = self.resolve_intl_plural_rules_options(&formatter)?;
                        Ok(self.intl_plural_rules_resolved_options_value(locale, &options))
                    }
                    IntlFormatterKind::RelativeTimeFormat => {
                        let (_, options) = self.resolve_intl_relative_time_options(&formatter)?;
                        Ok(self.intl_relative_time_resolved_options_value(locale, &options))
                    }
                    IntlFormatterKind::Segmenter => {
                        let (_, options) = self.resolve_intl_segmenter_options(&formatter)?;
                        Ok(self.intl_segmenter_resolved_options_value(locale, &options))
                    }
                    IntlFormatterKind::NumberFormat => Err(Error::ScriptRuntime(
                        "Intl.NumberFormat.resolvedOptions is not implemented".into(),
                    )),
                }
            }
            Expr::IntlDisplayNamesOf {
                display_names,
                code,
            } => {
                let display_names = self.eval_expr(display_names, env, event_param, event)?;
                let code = self.eval_expr(code, env, event_param, event)?.as_string();
                let (locale, options) = self.resolve_intl_display_names_options(&display_names)?;
                self.intl_display_names_of(&locale, &options, &code)
            }
            Expr::IntlPluralRulesSelect {
                plural_rules,
                value,
            } => {
                let plural_rules = self.eval_expr(plural_rules, env, event_param, event)?;
                let value = self.eval_expr(value, env, event_param, event)?;
                let (locale, options) = self.resolve_intl_plural_rules_options(&plural_rules)?;
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
                let (locale, options) = self.resolve_intl_plural_rules_options(&plural_rules)?;
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
                let (locale, options) = self.resolve_intl_relative_time_options(&formatter)?;
                Ok(Value::String(self.intl_format_relative_time(
                    &locale, &options, &value, &unit,
                )?))
            }
            Expr::IntlRelativeTimeFormatToParts {
                formatter,
                value,
                unit,
            } => {
                let formatter = self.eval_expr(formatter, env, event_param, event)?;
                let value = self.eval_expr(value, env, event_param, event)?;
                let unit = self.eval_expr(unit, env, event_param, event)?;
                let (locale, options) = self.resolve_intl_relative_time_options(&formatter)?;
                let parts =
                    self.intl_format_relative_time_to_parts(&locale, &options, &value, &unit)?;
                Ok(self.intl_relative_time_parts_to_value(&parts))
            }
            Expr::IntlSegmenterSegment { segmenter, value } => {
                let segmenter = self.eval_expr(segmenter, env, event_param, event)?;
                let value = self.eval_expr(value, env, event_param, event)?;
                let (locale, options) = self.resolve_intl_segmenter_options(&segmenter)?;
                let input = value.as_string();
                let segments = self.intl_segment_input(&locale, &options, &input);
                Ok(self.new_intl_segments_value(segments))
            }
            Expr::IntlStaticMethod { method, args } => match method {
                IntlStaticMethod::CollatorSupportedLocalesOf => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Intl.Collator.supportedLocalesOf requires locales and optional options"
                                .into(),
                        ));
                    }
                    let locales = self.eval_expr(&args[0], env, event_param, event)?;
                    let locales = self.intl_collect_locales(&locales)?;
                    let supported =
                        Self::intl_supported_locales(IntlFormatterKind::Collator, locales);
                    Ok(Self::new_array_value(supported))
                }
                IntlStaticMethod::DateTimeFormatSupportedLocalesOf => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Intl.DateTimeFormat.supportedLocalesOf requires locales and optional options"
                                .into(),
                        ));
                    }
                    let locales = self.eval_expr(&args[0], env, event_param, event)?;
                    let locales = self.intl_collect_locales(&locales)?;
                    let supported =
                        Self::intl_supported_locales(IntlFormatterKind::DateTimeFormat, locales);
                    Ok(Self::new_array_value(supported))
                }
                IntlStaticMethod::DisplayNamesSupportedLocalesOf => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Intl.DisplayNames.supportedLocalesOf requires locales and optional options"
                                .into(),
                        ));
                    }
                    let locales = self.eval_expr(&args[0], env, event_param, event)?;
                    let locales = self.intl_collect_locales(&locales)?;
                    let supported =
                        Self::intl_supported_locales(IntlFormatterKind::DisplayNames, locales);
                    Ok(Self::new_array_value(supported))
                }
                IntlStaticMethod::DurationFormatSupportedLocalesOf => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Intl.DurationFormat.supportedLocalesOf requires locales and optional options"
                                .into(),
                        ));
                    }
                    let locales = self.eval_expr(&args[0], env, event_param, event)?;
                    let locales = self.intl_collect_locales(&locales)?;
                    let supported =
                        Self::intl_supported_locales(IntlFormatterKind::DurationFormat, locales);
                    Ok(Self::new_array_value(supported))
                }
                IntlStaticMethod::ListFormatSupportedLocalesOf => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Intl.ListFormat.supportedLocalesOf requires locales and optional options"
                                .into(),
                        ));
                    }
                    let locales = self.eval_expr(&args[0], env, event_param, event)?;
                    let locales = self.intl_collect_locales(&locales)?;
                    let supported =
                        Self::intl_supported_locales(IntlFormatterKind::ListFormat, locales);
                    Ok(Self::new_array_value(supported))
                }
                IntlStaticMethod::PluralRulesSupportedLocalesOf => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Intl.PluralRules.supportedLocalesOf requires locales and optional options"
                                .into(),
                        ));
                    }
                    let locales = self.eval_expr(&args[0], env, event_param, event)?;
                    let locales = self.intl_collect_locales(&locales)?;
                    let supported =
                        Self::intl_supported_locales(IntlFormatterKind::PluralRules, locales);
                    Ok(Self::new_array_value(supported))
                }
                IntlStaticMethod::RelativeTimeFormatSupportedLocalesOf => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Intl.RelativeTimeFormat.supportedLocalesOf requires locales and optional options"
                                .into(),
                        ));
                    }
                    let locales = self.eval_expr(&args[0], env, event_param, event)?;
                    let locales = self.intl_collect_locales(&locales)?;
                    let supported = Self::intl_supported_locales(
                        IntlFormatterKind::RelativeTimeFormat,
                        locales,
                    );
                    Ok(Self::new_array_value(supported))
                }
                IntlStaticMethod::SegmenterSupportedLocalesOf => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Intl.Segmenter.supportedLocalesOf requires locales and optional options"
                                .into(),
                        ));
                    }
                    let locales = self.eval_expr(&args[0], env, event_param, event)?;
                    let locales = self.intl_collect_locales(&locales)?;
                    let supported =
                        Self::intl_supported_locales(IntlFormatterKind::Segmenter, locales);
                    Ok(Self::new_array_value(supported))
                }
                IntlStaticMethod::GetCanonicalLocales => {
                    let locales = if let Some(locale_expr) = args.first() {
                        let value = self.eval_expr(locale_expr, env, event_param, event)?;
                        self.intl_collect_locales(&value)?
                    } else {
                        Vec::new()
                    };
                    Ok(Self::new_array_value(
                        locales.into_iter().map(Value::String).collect::<Vec<_>>(),
                    ))
                }
                IntlStaticMethod::SupportedValuesOf => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Intl.supportedValuesOf requires exactly one argument".into(),
                        ));
                    }
                    let key = self
                        .eval_expr(&args[0], env, event_param, event)?
                        .as_string();
                    let values = Self::intl_supported_values_of(&key)?;
                    Ok(Self::new_array_value(
                        values.into_iter().map(Value::String).collect::<Vec<_>>(),
                    ))
                }
            },
            Expr::IntlLocaleConstruct {
                tag,
                options,
                called_with_new: _called_with_new,
            } => {
                let tag = self.eval_expr(tag, env, event_param, event)?;
                let options = options
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                let data = self.intl_locale_data_from_input_value(&tag, options.as_ref())?;
                Ok(self.new_intl_locale_value(data))
            }
            Expr::IntlLocaleMethod { locale, method } => {
                let locale = self.eval_expr(locale, env, event_param, event)?;
                let data = self.resolve_intl_locale_data(&locale)?;
                match method {
                    IntlLocaleMethod::GetCalendars => Ok(Self::new_array_value(
                        self.intl_locale_get_calendars(&data)
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    )),
                    IntlLocaleMethod::GetCollations => Ok(Self::new_array_value(
                        self.intl_locale_get_collations(&data)
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    )),
                    IntlLocaleMethod::GetHourCycles => Ok(Self::new_array_value(
                        self.intl_locale_get_hour_cycles(&data)
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    )),
                    IntlLocaleMethod::GetNumberingSystems => Ok(Self::new_array_value(
                        self.intl_locale_get_numbering_systems(&data)
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    )),
                    IntlLocaleMethod::GetTextInfo => Ok(self.intl_locale_get_text_info(&data)),
                    IntlLocaleMethod::GetTimeZones => Ok(Self::new_array_value(
                        self.intl_locale_get_time_zones(&data)
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    )),
                    IntlLocaleMethod::GetWeekInfo => Ok(self.intl_locale_get_week_info(&data)),
                    IntlLocaleMethod::Maximize => {
                        Ok(self.new_intl_locale_value(self.intl_locale_maximize_data(&data)))
                    }
                    IntlLocaleMethod::Minimize => {
                        Ok(self.new_intl_locale_value(self.intl_locale_minimize_data(&data)))
                    }
                    IntlLocaleMethod::ToString => {
                        Ok(Value::String(Self::intl_locale_data_to_string(&data)))
                    }
                }
            }
            Expr::IntlConstruct { .. } => {
                Err(Error::ScriptRuntime("Intl is not a constructor".into()))
            }
            Expr::RegexLiteral { pattern, flags } => {
                Self::new_regex_value(pattern.clone(), flags.clone())
            }
            Expr::RegexNew { pattern, flags } => {
                let pattern = self.eval_expr(pattern, env, event_param, event)?;
                let flags = flags
                    .as_ref()
                    .map(|flags| self.eval_expr(flags, env, event_param, event))
                    .transpose()?;
                Self::new_regex_from_values(&pattern, flags.as_ref())
            }
            Expr::RegExpConstructor => Ok(Value::RegExpConstructor),
            Expr::RegExpStaticMethod { method, args } => {
                self.eval_regexp_static_method(*method, args, env, event_param, event)
            }
            Expr::RegexTest { regex, input } => {
                let regex = self.eval_expr(regex, env, event_param, event)?;
                let input = self.eval_expr(input, env, event_param, event)?.as_string();
                let regex = Self::resolve_regex_from_value(&regex)?;
                Ok(Value::Bool(Self::regex_test(&regex, &input)?))
            }
            Expr::RegexExec { regex, input } => {
                let regex = self.eval_expr(regex, env, event_param, event)?;
                let input = self.eval_expr(input, env, event_param, event)?.as_string();
                let regex = Self::resolve_regex_from_value(&regex)?;
                let Some(captures) = Self::regex_exec(&regex, &input)? else {
                    return Ok(Value::Null);
                };
                Ok(Self::new_array_value(
                    captures.into_iter().map(Value::String).collect::<Vec<_>>(),
                ))
            }
            Expr::RegexToString { regex } => {
                let value = self.eval_expr(regex, env, event_param, event)?;
                if let Ok(regex) = Self::resolve_regex_from_value(&value) {
                    let regex = regex.borrow();
                    Ok(Value::String(format!("/{}/{}", regex.source, regex.flags)))
                } else if let Ok(locale_data) = self.resolve_intl_locale_data(&value) {
                    Ok(Value::String(Self::intl_locale_data_to_string(
                        &locale_data,
                    )))
                } else {
                    Ok(Value::String(value.as_string()))
                }
            }
            Expr::MathConst(constant) => match constant {
                MathConst::E => Ok(Value::Float(std::f64::consts::E)),
                MathConst::Ln10 => Ok(Value::Float(std::f64::consts::LN_10)),
                MathConst::Ln2 => Ok(Value::Float(std::f64::consts::LN_2)),
                MathConst::Log10E => Ok(Value::Float(std::f64::consts::LOG10_E)),
                MathConst::Log2E => Ok(Value::Float(std::f64::consts::LOG2_E)),
                MathConst::Pi => Ok(Value::Float(std::f64::consts::PI)),
                MathConst::Sqrt1_2 => Ok(Value::Float(std::f64::consts::FRAC_1_SQRT_2)),
                MathConst::Sqrt2 => Ok(Value::Float(std::f64::consts::SQRT_2)),
                MathConst::ToStringTag => Ok(Value::String("Math".to_string())),
            },
            Expr::MathMethod { method, args } => {
                self.eval_math_method(*method, args, env, event_param, event)
            }
            Expr::StringConstruct {
                value,
                called_with_new,
            } => {
                let value = value
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .unwrap_or(Value::Undefined);
                let coerced = value.as_string();
                if *called_with_new {
                    Ok(Self::new_string_wrapper_value(coerced))
                } else {
                    Ok(Value::String(coerced))
                }
            }
            Expr::StringStaticMethod { method, args } => {
                self.eval_string_static_method(*method, args, env, event_param, event)
            }
            Expr::StringConstructor => Ok(Value::StringConstructor),
            Expr::NumberConstruct { value } => {
                let value = value
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .unwrap_or(Value::Number(0));
                Ok(Self::number_value(
                    Self::coerce_number_for_number_constructor(&value),
                ))
            }
            Expr::NumberConst(constant) => match constant {
                NumberConst::Epsilon => Ok(Value::Float(f64::EPSILON)),
                NumberConst::MaxSafeInteger => Ok(Value::Number(9_007_199_254_740_991)),
                NumberConst::MaxValue => Ok(Value::Float(f64::MAX)),
                NumberConst::MinSafeInteger => Ok(Value::Number(-9_007_199_254_740_991)),
                NumberConst::MinValue => Ok(Value::Float(f64::from_bits(1))),
                NumberConst::NaN => Ok(Value::Float(f64::NAN)),
                NumberConst::NegativeInfinity => Ok(Value::Float(f64::NEG_INFINITY)),
                NumberConst::PositiveInfinity => Ok(Value::Float(f64::INFINITY)),
            },
            Expr::NumberMethod { method, args } => {
                self.eval_number_method(*method, args, env, event_param, event)
            }
            Expr::NumberInstanceMethod {
                value,
                method,
                args,
            } => self.eval_number_instance_method(*method, value, args, env, event_param, event),
            Expr::BigIntConstruct {
                value,
                called_with_new,
            } => {
                if *called_with_new {
                    return Err(Error::ScriptRuntime("BigInt is not a constructor".into()));
                }
                let value = value
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .unwrap_or(Value::Undefined);
                Ok(Value::BigInt(Self::coerce_bigint_for_constructor(&value)?))
            }
            Expr::BigIntMethod { method, args } => {
                self.eval_bigint_method(*method, args, env, event_param, event)
            }
            Expr::BigIntInstanceMethod {
                value,
                method,
                args,
            } => self.eval_bigint_instance_method(*method, value, args, env, event_param, event),
            Expr::BlobConstruct {
                parts,
                options,
                called_with_new,
            } => {
                self.eval_blob_construct(parts, options, *called_with_new, env, event_param, event)
            }
            Expr::BlobConstructor => Ok(Value::BlobConstructor),
            Expr::UrlConstruct {
                input,
                base,
                called_with_new,
            } => self.eval_url_construct(input, base, *called_with_new, env, event_param, event),
            Expr::UrlConstructor => Ok(Value::UrlConstructor),
            Expr::UrlStaticMethod { method, args } => {
                self.eval_url_static_method(*method, args, env, event_param, event)
            }
            Expr::ArrayBufferConstruct {
                byte_length,
                options,
                called_with_new,
            } => self.eval_array_buffer_construct(
                byte_length,
                options,
                *called_with_new,
                env,
                event_param,
                event,
            ),
            Expr::ArrayBufferConstructor => Ok(Value::ArrayBufferConstructor),
            Expr::ArrayBufferIsView(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Bool(matches!(value, Value::TypedArray(_))))
            }
            Expr::ArrayBufferDetached(target) => {
                let buffer = self.resolve_array_buffer_from_env(env, target)?;
                Ok(Value::Bool(buffer.borrow().detached))
            }
            Expr::ArrayBufferMaxByteLength(target) => {
                let buffer = self.resolve_array_buffer_from_env(env, target)?;
                Ok(Value::Number(buffer.borrow().max_byte_length() as i64))
            }
            Expr::ArrayBufferResizable(target) => {
                let buffer = self.resolve_array_buffer_from_env(env, target)?;
                Ok(Value::Bool(buffer.borrow().resizable()))
            }
            Expr::ArrayBufferResize {
                target,
                new_byte_length,
            } => {
                let new_byte_length = self.eval_expr(new_byte_length, env, event_param, event)?;
                let new_byte_length = Self::value_to_i64(&new_byte_length);
                self.resize_array_buffer_in_env(env, target, new_byte_length)?;
                Ok(Value::Undefined)
            }
            Expr::ArrayBufferSlice { target, start, end } => {
                let buffer = self.resolve_array_buffer_from_env(env, target)?;
                Self::ensure_array_buffer_not_detached(&buffer, "slice")?;
                let source = buffer.borrow();
                let len = source.bytes.len();
                let start = if let Some(start) = start {
                    let start = self.eval_expr(start, env, event_param, event)?;
                    Self::normalize_slice_index(len, Self::value_to_i64(&start))
                } else {
                    0
                };
                let end = if let Some(end) = end {
                    let end = self.eval_expr(end, env, event_param, event)?;
                    Self::normalize_slice_index(len, Self::value_to_i64(&end))
                } else {
                    len
                };
                let end = end.max(start);
                let bytes = source.bytes[start..end].to_vec();
                Ok(Value::ArrayBuffer(Rc::new(RefCell::new(
                    ArrayBufferValue {
                        bytes,
                        max_byte_length: None,
                        detached: false,
                    },
                ))))
            }
            Expr::ArrayBufferTransfer {
                target,
                to_fixed_length,
            } => {
                let buffer = self.resolve_array_buffer_from_env(env, target)?;
                self.transfer_array_buffer(&buffer, *to_fixed_length)
            }
            Expr::TypedArrayConstructorRef(kind) => Ok(Value::TypedArrayConstructor(kind.clone())),
            Expr::TypedArrayConstruct {
                kind,
                args,
                called_with_new,
            } => self.eval_typed_array_construct(
                *kind,
                args,
                *called_with_new,
                env,
                event_param,
                event,
            ),
            Expr::TypedArrayConstructWithCallee {
                callee,
                args,
                called_with_new,
            } => self.eval_typed_array_construct_with_callee(
                callee,
                args,
                *called_with_new,
                env,
                event_param,
                event,
            ),
            Expr::PromiseConstruct {
                executor,
                called_with_new,
            } => self.eval_promise_construct(executor, *called_with_new, env, event_param, event),
            Expr::PromiseConstructor => Ok(Value::PromiseConstructor),
            Expr::PromiseStaticMethod { method, args } => {
                self.eval_promise_static_method(*method, args, env, event_param, event)
            }
            Expr::PromiseMethod {
                target,
                method,
                args,
            } => self.eval_promise_method(target, *method, args, env, event_param, event),
            Expr::MapConstruct {
                iterable,
                called_with_new,
            } => self.eval_map_construct(iterable, *called_with_new, env, event_param, event),
            Expr::MapConstructor => Ok(Value::MapConstructor),
            Expr::MapStaticMethod { method, args } => {
                self.eval_map_static_method(*method, args, env, event_param, event)
            }
            Expr::MapMethod {
                target,
                method,
                args,
            } => self.eval_map_method(target, *method, args, env, event_param, event),
            Expr::UrlSearchParamsConstruct {
                init,
                called_with_new,
            } => self.eval_url_search_params_construct(
                init,
                *called_with_new,
                env,
                event_param,
                event,
            ),
            Expr::UrlSearchParamsMethod {
                target,
                method,
                args,
            } => self.eval_url_search_params_method(target, *method, args, env, event_param, event),
            Expr::SetConstruct {
                iterable,
                called_with_new,
            } => self.eval_set_construct(iterable, *called_with_new, env, event_param, event),
            Expr::SetConstructor => Ok(Value::SetConstructor),
            Expr::SetMethod {
                target,
                method,
                args,
            } => self.eval_set_method(target, *method, args, env, event_param, event),
            Expr::SymbolConstruct {
                description,
                called_with_new,
            } => self.eval_symbol_construct(description, *called_with_new, env, event_param, event),
            Expr::SymbolConstructor => Ok(Value::SymbolConstructor),
            Expr::SymbolStaticMethod { method, args } => {
                self.eval_symbol_static_method(*method, args, env, event_param, event)
            }
            Expr::SymbolStaticProperty(property) => Ok(self.eval_symbol_static_property(*property)),
            Expr::TypedArrayStaticBytesPerElement(kind) => {
                Ok(Value::Number(kind.bytes_per_element() as i64))
            }
            Expr::TypedArrayStaticMethod { kind, method, args } => {
                self.eval_typed_array_static_method(*kind, *method, args, env, event_param, event)
            }
            Expr::TypedArrayByteLength(target) => match env.get(target) {
                Some(Value::TypedArray(array)) => {
                    Ok(Value::Number(array.borrow().observed_byte_length() as i64))
                }
                Some(Value::ArrayBuffer(buffer)) => {
                    Ok(Value::Number(buffer.borrow().byte_length() as i64))
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a TypedArray",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::TypedArrayByteOffset(target) => {
                let array = self.resolve_typed_array_from_env(env, target)?;
                let byte_offset = if array.borrow().observed_length() == 0
                    && array.borrow().byte_offset >= array.borrow().buffer.borrow().byte_length()
                {
                    0
                } else {
                    array.borrow().byte_offset
                };
                Ok(Value::Number(byte_offset as i64))
            }
            Expr::TypedArrayBuffer(target) => {
                let array = self.resolve_typed_array_from_env(env, target)?;
                Ok(Value::ArrayBuffer(array.borrow().buffer.clone()))
            }
            Expr::TypedArrayBytesPerElement(target) => {
                let array = self.resolve_typed_array_from_env(env, target)?;
                Ok(Value::Number(array.borrow().kind.bytes_per_element() as i64))
            }
            Expr::TypedArrayMethod {
                target,
                method,
                args,
            } => self.eval_typed_array_method(target, *method, args, env, event_param, event),
            Expr::EncodeUri(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(encode_uri_like(&value.as_string(), false)))
            }
            Expr::EncodeUriComponent(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(encode_uri_like(&value.as_string(), true)))
            }
            Expr::DecodeUri(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(decode_uri_like(&value.as_string(), false)?))
            }
            Expr::DecodeUriComponent(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(decode_uri_like(&value.as_string(), true)?))
            }
            Expr::Escape(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(js_escape(&value.as_string())))
            }
            Expr::Unescape(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(js_unescape(&value.as_string())))
            }
            Expr::IsNaN(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Bool(Self::coerce_number_for_global(&value).is_nan()))
            }
            Expr::IsFinite(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Bool(
                    Self::coerce_number_for_global(&value).is_finite(),
                ))
            }
            Expr::Atob(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(decode_base64_to_binary_string(
                    &value.as_string(),
                )?))
            }
            Expr::Btoa(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(encode_binary_string_to_base64(
                    &value.as_string(),
                )?))
            }
            Expr::ParseInt { value, radix } => {
                let value = self.eval_expr(value, env, event_param, event)?;
                let radix = radix
                    .as_ref()
                    .map(|expr| self.eval_expr(expr, env, event_param, event))
                    .transpose()?
                    .map(|radix| Self::value_to_i64(&radix));
                Ok(Value::Float(parse_js_parse_int(&value.as_string(), radix)))
            }
            Expr::ParseFloat(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Float(parse_js_parse_float(&value.as_string())))
            }
            Expr::JsonParse(value) => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                Self::parse_json_text(&value)
            }
            Expr::JsonStringify(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                match Self::json_stringify_top_level(&value)? {
                    Some(serialized) => Ok(Value::String(serialized)),
                    None => Ok(Value::Undefined),
                }
            }
            Expr::ObjectConstruct { value } => {
                let value = value
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .unwrap_or(Value::Undefined);
                match value {
                    Value::Null | Value::Undefined => Ok(Self::new_object_value(Vec::new())),
                    Value::Object(object) => Ok(Value::Object(object)),
                    Value::Array(array) => Ok(Value::Array(array)),
                    Value::Date(date) => Ok(Value::Date(date)),
                    Value::Map(map) => Ok(Value::Map(map)),
                    Value::Set(set) => Ok(Value::Set(set)),
                    Value::Blob(blob) => Ok(Value::Blob(blob)),
                    Value::ArrayBuffer(buffer) => Ok(Value::ArrayBuffer(buffer)),
                    Value::TypedArray(array) => Ok(Value::TypedArray(array)),
                    Value::Promise(promise) => Ok(Value::Promise(promise)),
                    Value::RegExp(regex) => Ok(Value::RegExp(regex)),
                    Value::Symbol(symbol) => Ok(Self::new_object_value(vec![(
                        INTERNAL_SYMBOL_WRAPPER_KEY.to_string(),
                        Value::Number(symbol.id as i64),
                    )])),
                    primitive => Ok(Self::new_object_value(vec![(
                        "value".into(),
                        Value::String(primitive.as_string()),
                    )])),
                }
            }
            Expr::ObjectLiteral(entries) => {
                let mut object_entries = Vec::with_capacity(entries.len());
                for entry in entries {
                    match entry {
                        ObjectLiteralEntry::Pair(key, value) => {
                            let value = self.eval_expr(value, env, event_param, event)?;
                            let key = match key {
                                ObjectLiteralKey::Static(key) => key.clone(),
                                ObjectLiteralKey::Computed(expr) => {
                                    let key = self.eval_expr(expr, env, event_param, event)?;
                                    self.property_key_to_storage_key(&key)
                                }
                            };
                            Self::object_set_entry(&mut object_entries, key, value);
                        }
                        ObjectLiteralEntry::Spread(expr) => {
                            let spread_value = self.eval_expr(expr, env, event_param, event)?;
                            match spread_value {
                                Value::Null | Value::Undefined => {}
                                Value::Object(entries) => {
                                    for (key, value) in entries.borrow().iter() {
                                        if Self::is_internal_object_key(key) {
                                            continue;
                                        }
                                        Self::object_set_entry(
                                            &mut object_entries,
                                            key.clone(),
                                            value.clone(),
                                        );
                                    }
                                }
                                Value::Array(values) => {
                                    for (index, value) in values.borrow().iter().enumerate() {
                                        Self::object_set_entry(
                                            &mut object_entries,
                                            index.to_string(),
                                            value.clone(),
                                        );
                                    }
                                }
                                Value::String(text) => {
                                    for (index, ch) in text.chars().enumerate() {
                                        Self::object_set_entry(
                                            &mut object_entries,
                                            index.to_string(),
                                            Value::String(ch.to_string()),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Self::new_object_value(object_entries))
            }
            Expr::ObjectGet { target, key } => match env.get(target) {
                Some(value) => {
                    self.object_property_from_value(value, key)
                        .map_err(|err| match err {
                            Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                                Error::ScriptRuntime(format!(
                                    "variable '{}' is not an object (key '{}')",
                                    target, key
                                ))
                            }
                            other => other,
                        })
                }
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::ObjectPathGet { target, path } => {
                let Some(mut value) = env.get(target).cloned() else {
                    return Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    )));
                };
                for key in path {
                    value = self.object_property_from_value(&value, key)?;
                }
                Ok(value)
            }
            Expr::ObjectGetOwnPropertySymbols(object) => {
                let object = self.eval_expr(object, env, event_param, event)?;
                match object {
                    Value::Object(entries) => {
                        let mut out = Vec::new();
                        for (key, _) in entries.borrow().iter() {
                            if let Some(symbol_id) = Self::symbol_id_from_storage_key(key) {
                                if let Some(symbol) = self.symbol_runtime.symbols_by_id.get(&symbol_id) {
                                    out.push(Value::Symbol(symbol.clone()));
                                }
                            }
                        }
                        Ok(Self::new_array_value(out))
                    }
                    _ => Err(Error::ScriptRuntime(
                        "Object.getOwnPropertySymbols argument must be an object".into(),
                    )),
                }
            }
            Expr::ObjectKeys(object) => {
                let object = self.eval_expr(object, env, event_param, event)?;
                match object {
                    Value::Object(entries) => {
                        let keys = entries
                            .borrow()
                            .iter()
                            .filter(|(key, _)| !Self::is_internal_object_key(key))
                            .map(|(key, _)| Value::String(key.clone()))
                            .collect::<Vec<_>>();
                        Ok(Self::new_array_value(keys))
                    }
                    _ => Err(Error::ScriptRuntime(
                        "Object.keys argument must be an object".into(),
                    )),
                }
            }
            Expr::ObjectValues(object) => {
                let object = self.eval_expr(object, env, event_param, event)?;
                match object {
                    Value::Object(entries) => {
                        let values = entries
                            .borrow()
                            .iter()
                            .filter(|(key, _)| !Self::is_internal_object_key(key))
                            .map(|(_, value)| value.clone())
                            .collect::<Vec<_>>();
                        Ok(Self::new_array_value(values))
                    }
                    _ => Err(Error::ScriptRuntime(
                        "Object.values argument must be an object".into(),
                    )),
                }
            }
            Expr::ObjectEntries(object) => {
                let object = self.eval_expr(object, env, event_param, event)?;
                match object {
                    Value::Object(entries) => {
                        let values = entries
                            .borrow()
                            .iter()
                            .filter(|(key, _)| !Self::is_internal_object_key(key))
                            .map(|(key, value)| {
                                Self::new_array_value(vec![
                                    Value::String(key.clone()),
                                    value.clone(),
                                ])
                            })
                            .collect::<Vec<_>>();
                        Ok(Self::new_array_value(values))
                    }
                    _ => Err(Error::ScriptRuntime(
                        "Object.entries argument must be an object".into(),
                    )),
                }
            }
            Expr::ObjectHasOwn { object, key } => {
                let object = self.eval_expr(object, env, event_param, event)?;
                let key = self.eval_expr(key, env, event_param, event)?;
                let key = self.property_key_to_storage_key(&key);
                match object {
                    Value::Object(entries) => Ok(Value::Bool(
                        Self::object_get_entry(&entries.borrow(), &key).is_some(),
                    )),
                    _ => Err(Error::ScriptRuntime(
                        "Object.hasOwn first argument must be an object".into(),
                    )),
                }
            }
            Expr::ObjectGetPrototypeOf(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                match value {
                    Value::TypedArrayConstructor(TypedArrayConstructorKind::Concrete(_)) => Ok(
                        Value::TypedArrayConstructor(TypedArrayConstructorKind::Abstract),
                    ),
                    Value::TypedArray(_) => Ok(Value::TypedArrayConstructor(
                        TypedArrayConstructorKind::Abstract,
                    )),
                    _ => Ok(Value::Object(Rc::new(RefCell::new(ObjectValue::default())))),
                }
            }
            Expr::ObjectFreeze(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                match value {
                    Value::TypedArray(array) => {
                        if array.borrow().observed_length() > 0 {
                            return Err(Error::ScriptRuntime(
                                "Cannot freeze array buffer views with elements".into(),
                            ));
                        }
                        Ok(Value::TypedArray(array))
                    }
                    other => Ok(other),
                }
            }
            Expr::ObjectHasOwnProperty { target, key } => {
                let key = self.eval_expr(key, env, event_param, event)?;
                let key = self.property_key_to_storage_key(&key);
                match env.get(target) {
                    Some(Value::Object(entries)) => Ok(Value::Bool(
                        Self::object_get_entry(&entries.borrow(), &key).is_some(),
                    )),
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an object",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                }
            }
            Expr::ArrayLiteral(values) => {
                let mut out = Vec::with_capacity(values.len());
                for value in values {
                    match value {
                        Expr::Spread(expr) => {
                            let spread_value = self.eval_expr(expr, env, event_param, event)?;
                            out.extend(self.spread_iterable_values_from_value(&spread_value)?);
                        }
                        _ => out.push(self.eval_expr(value, env, event_param, event)?),
                    }
                }
                Ok(Self::new_array_value(out))
            }
            Expr::ArrayIsArray(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Bool(matches!(value, Value::Array(_))))
            }
            Expr::ArrayFrom { source, map_fn } => {
                let source = self.eval_expr(source, env, event_param, event)?;
                let values = self.array_like_values_from_value(&source)?;
                if let Some(map_fn) = map_fn {
                    let callback = self.eval_expr(map_fn, env, event_param, event)?;
                    let mut mapped = Vec::with_capacity(values.len());
                    for (index, value) in values.into_iter().enumerate() {
                        mapped.push(self.execute_callback_value(
                            &callback,
                            &[value, Value::Number(index as i64)],
                            event,
                        )?);
                    }
                    return Ok(Self::new_array_value(mapped));
                }
                Ok(Self::new_array_value(values))
            }
            Expr::ArrayLength(target) => match env.get(target) {
                Some(Value::Array(values)) => Ok(Value::Number(values.borrow().len() as i64)),
                Some(Value::TypedArray(values)) => {
                    Ok(Value::Number(values.borrow().observed_length() as i64))
                }
                Some(Value::NodeList(nodes)) => Ok(Value::Number(nodes.len() as i64)),
                Some(Value::String(value)) => Ok(Value::Number(value.chars().count() as i64)),
                Some(Value::Object(entries)) => {
                    let entries = entries.borrow();
                    if Self::is_history_object(&entries) {
                        return Ok(Self::object_get_entry(&entries, "length")
                            .unwrap_or(Value::Number(self.location_history.history_entries.len() as i64)));
                    }
                    if Self::is_window_object(&entries) {
                        return Ok(
                            Self::object_get_entry(&entries, "length").unwrap_or(Value::Number(0))
                        );
                    }
                    if Self::is_storage_object(&entries) {
                        let len = Self::storage_pairs_from_object_entries(&entries).len();
                        return Ok(Value::Number(len as i64));
                    }
                    if let Some(value) = Self::string_wrapper_value_from_object(&entries) {
                        Ok(Value::Number(value.chars().count() as i64))
                    } else {
                        Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        )))
                    }
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::ArrayIndex { target, index } => {
                let index = self.eval_expr(index, env, event_param, event)?;
                match env.get(target) {
                    Some(Value::Object(entries)) => {
                        let entries_ref = entries.borrow();
                        if let Some(value) = Self::string_wrapper_value_from_object(&entries_ref) {
                            let Some(index) = self.value_as_index(&index) else {
                                return Ok(Value::Undefined);
                            };
                            return Ok(value
                                .chars()
                                .nth(index)
                                .map(|ch| Value::String(ch.to_string()))
                                .unwrap_or(Value::Undefined));
                        }
                        let key = self.property_key_to_storage_key(&index);
                        if Self::is_storage_object(&entries_ref) {
                            return Ok(Self::storage_pairs_from_object_entries(&entries_ref)
                                .into_iter()
                                .find_map(|(name, value)| {
                                    (name == key).then(|| Value::String(value))
                                })
                                .unwrap_or(Value::Undefined));
                        }
                        Ok(Self::object_get_entry(&entries_ref, &key).unwrap_or(Value::Undefined))
                    }
                    Some(Value::Array(values)) => {
                        let Some(index) = self.value_as_index(&index) else {
                            return Ok(Value::Undefined);
                        };
                        Ok(values
                            .borrow()
                            .get(index)
                            .cloned()
                            .unwrap_or(Value::Undefined))
                    }
                    Some(Value::TypedArray(values)) => {
                        let Some(index) = self.value_as_index(&index) else {
                            return Ok(Value::Undefined);
                        };
                        self.typed_array_get_index(values, index)
                    }
                    Some(Value::NodeList(nodes)) => {
                        let Some(index) = self.value_as_index(&index) else {
                            return Ok(Value::Undefined);
                        };
                        Ok(nodes
                            .get(index)
                            .copied()
                            .map(Value::Node)
                            .unwrap_or(Value::Undefined))
                    }
                    Some(Value::String(value)) => {
                        let Some(index) = self.value_as_index(&index) else {
                            return Ok(Value::Undefined);
                        };
                        Ok(value
                            .chars()
                            .nth(index)
                            .map(|ch| Value::String(ch.to_string()))
                            .unwrap_or(Value::Undefined))
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                }
            }
            Expr::ArrayPush { target, args } => {
                let values = self.resolve_array_from_env(env, target)?;
                let evaluated = self.eval_call_args_with_spread(args, env, event_param, event)?;
                let mut values = values.borrow_mut();
                values.extend(evaluated);
                Ok(Value::Number(values.len() as i64))
            }
            Expr::ArrayPop(target) => {
                let values = self.resolve_array_from_env(env, target)?;
                Ok(values.borrow_mut().pop().unwrap_or(Value::Undefined))
            }
            Expr::ArrayShift(target) => {
                let values = self.resolve_array_from_env(env, target)?;
                let mut values = values.borrow_mut();
                if values.is_empty() {
                    Ok(Value::Undefined)
                } else {
                    Ok(values.remove(0))
                }
            }
            Expr::ArrayUnshift { target, args } => {
                let values = self.resolve_array_from_env(env, target)?;
                let evaluated = self.eval_call_args_with_spread(args, env, event_param, event)?;
                let mut values = values.borrow_mut();
                for value in evaluated.into_iter().rev() {
                    values.insert(0, value);
                }
                Ok(Value::Number(values.len() as i64))
            }
            Expr::ArrayMap { target, callback } => match env.get(target) {
                Some(Value::Array(values)) => {
                    let input = values.borrow().clone();
                    let mut out = Vec::with_capacity(input.len());
                    for (idx, item) in input.into_iter().enumerate() {
                        let mapped = self.execute_array_callback(
                            callback,
                            &[
                                item,
                                Value::Number(idx as i64),
                                Value::Array(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        out.push(mapped);
                    }
                    Ok(Self::new_array_value(out))
                }
                Some(Value::TypedArray(values)) => {
                    let input = self.typed_array_snapshot(values)?;
                    let kind = values.borrow().kind;
                    let mut out = Vec::with_capacity(input.len());
                    for (idx, item) in input.into_iter().enumerate() {
                        let mapped = self.execute_array_callback(
                            callback,
                            &[
                                item,
                                Value::Number(idx as i64),
                                Value::TypedArray(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        out.push(mapped);
                    }
                    self.new_typed_array_from_values(kind, &out)
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::ArrayFilter { target, callback } => match env.get(target) {
                Some(Value::Array(values)) => {
                    let input = values.borrow().clone();
                    let mut out = Vec::new();
                    for (idx, item) in input.into_iter().enumerate() {
                        let keep = self.execute_array_callback(
                            callback,
                            &[
                                item.clone(),
                                Value::Number(idx as i64),
                                Value::Array(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        if keep.truthy() {
                            out.push(item);
                        }
                    }
                    Ok(Self::new_array_value(out))
                }
                Some(Value::TypedArray(values)) => {
                    let input = self.typed_array_snapshot(values)?;
                    let kind = values.borrow().kind;
                    let mut out = Vec::new();
                    for (idx, item) in input.into_iter().enumerate() {
                        let keep = self.execute_array_callback(
                            callback,
                            &[
                                item.clone(),
                                Value::Number(idx as i64),
                                Value::TypedArray(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        if keep.truthy() {
                            out.push(item);
                        }
                    }
                    self.new_typed_array_from_values(kind, &out)
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::ArrayReduce {
                target,
                callback,
                initial,
            } => match env.get(target) {
                Some(Value::Array(values)) => {
                    let input = values.borrow().clone();
                    let mut start_index = 0usize;
                    let mut acc = if let Some(initial) = initial {
                        self.eval_expr(initial, env, event_param, event)?
                    } else {
                        let Some(first) = input.first().cloned() else {
                            return Err(Error::ScriptRuntime(
                                "reduce of empty array with no initial value".into(),
                            ));
                        };
                        start_index = 1;
                        first
                    };
                    for (idx, item) in input.into_iter().enumerate().skip(start_index) {
                        acc = self.execute_array_callback(
                            callback,
                            &[
                                acc,
                                item,
                                Value::Number(idx as i64),
                                Value::Array(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                    }
                    Ok(acc)
                }
                Some(Value::TypedArray(values)) => {
                    let input = self.typed_array_snapshot(values)?;
                    let mut start_index = 0usize;
                    let mut acc = if let Some(initial) = initial {
                        self.eval_expr(initial, env, event_param, event)?
                    } else {
                        let Some(first) = input.first().cloned() else {
                            return Err(Error::ScriptRuntime(
                                "reduce of empty array with no initial value".into(),
                            ));
                        };
                        start_index = 1;
                        first
                    };
                    for (idx, item) in input.into_iter().enumerate().skip(start_index) {
                        acc = self.execute_array_callback(
                            callback,
                            &[
                                acc,
                                item,
                                Value::Number(idx as i64),
                                Value::TypedArray(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                    }
                    Ok(acc)
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::ArrayForEach { target, callback } => match env.get(target) {
                Some(Value::Array(values)) => {
                    let input = values.borrow().clone();
                    for (idx, item) in input.into_iter().enumerate() {
                        let _ = self.execute_array_callback(
                            callback,
                            &[
                                item,
                                Value::Number(idx as i64),
                                Value::Array(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                    }
                    Ok(Value::Undefined)
                }
                Some(Value::TypedArray(values)) => {
                    let input = self.typed_array_snapshot(values)?;
                    for (idx, item) in input.into_iter().enumerate() {
                        let _ = self.execute_array_callback(
                            callback,
                            &[
                                item,
                                Value::Number(idx as i64),
                                Value::TypedArray(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                    }
                    Ok(Value::Undefined)
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::ArrayFind { target, callback } => match env.get(target) {
                Some(Value::Array(values)) => {
                    let input = values.borrow().clone();
                    for (idx, item) in input.into_iter().enumerate() {
                        let matched = self.execute_array_callback(
                            callback,
                            &[
                                item.clone(),
                                Value::Number(idx as i64),
                                Value::Array(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        if matched.truthy() {
                            return Ok(item);
                        }
                    }
                    Ok(Value::Undefined)
                }
                Some(Value::TypedArray(values)) => {
                    let input = self.typed_array_snapshot(values)?;
                    for (idx, item) in input.into_iter().enumerate() {
                        let matched = self.execute_array_callback(
                            callback,
                            &[
                                item.clone(),
                                Value::Number(idx as i64),
                                Value::TypedArray(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        if matched.truthy() {
                            return Ok(item);
                        }
                    }
                    Ok(Value::Undefined)
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::ArrayFindIndex { target, callback } => match env.get(target) {
                Some(Value::Array(values)) => {
                    let input = values.borrow().clone();
                    for (idx, item) in input.into_iter().enumerate() {
                        let matched = self.execute_array_callback(
                            callback,
                            &[
                                item,
                                Value::Number(idx as i64),
                                Value::Array(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        if matched.truthy() {
                            return Ok(Value::Number(idx as i64));
                        }
                    }
                    Ok(Value::Number(-1))
                }
                Some(Value::TypedArray(values)) => {
                    let input = self.typed_array_snapshot(values)?;
                    for (idx, item) in input.into_iter().enumerate() {
                        let matched = self.execute_array_callback(
                            callback,
                            &[
                                item,
                                Value::Number(idx as i64),
                                Value::TypedArray(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        if matched.truthy() {
                            return Ok(Value::Number(idx as i64));
                        }
                    }
                    Ok(Value::Number(-1))
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::ArraySome { target, callback } => match env.get(target) {
                Some(Value::Array(values)) => {
                    let input = values.borrow().clone();
                    for (idx, item) in input.into_iter().enumerate() {
                        let matched = self.execute_array_callback(
                            callback,
                            &[
                                item,
                                Value::Number(idx as i64),
                                Value::Array(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        if matched.truthy() {
                            return Ok(Value::Bool(true));
                        }
                    }
                    Ok(Value::Bool(false))
                }
                Some(Value::TypedArray(values)) => {
                    let input = self.typed_array_snapshot(values)?;
                    for (idx, item) in input.into_iter().enumerate() {
                        let matched = self.execute_array_callback(
                            callback,
                            &[
                                item,
                                Value::Number(idx as i64),
                                Value::TypedArray(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        if matched.truthy() {
                            return Ok(Value::Bool(true));
                        }
                    }
                    Ok(Value::Bool(false))
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::ArrayEvery { target, callback } => match env.get(target) {
                Some(Value::Array(values)) => {
                    let input = values.borrow().clone();
                    for (idx, item) in input.into_iter().enumerate() {
                        let matched = self.execute_array_callback(
                            callback,
                            &[
                                item,
                                Value::Number(idx as i64),
                                Value::Array(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        if !matched.truthy() {
                            return Ok(Value::Bool(false));
                        }
                    }
                    Ok(Value::Bool(true))
                }
                Some(Value::TypedArray(values)) => {
                    let input = self.typed_array_snapshot(values)?;
                    for (idx, item) in input.into_iter().enumerate() {
                        let matched = self.execute_array_callback(
                            callback,
                            &[
                                item,
                                Value::Number(idx as i64),
                                Value::TypedArray(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                        if !matched.truthy() {
                            return Ok(Value::Bool(false));
                        }
                    }
                    Ok(Value::Bool(true))
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::ArrayIncludes {
                target,
                search,
                from_index,
            } => {
                let search = self.eval_expr(search, env, event_param, event)?;
                match env.get(target) {
                    Some(Value::Array(values)) => {
                        let values = values.borrow();
                        let len = values.len() as i64;
                        let mut start = from_index
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .unwrap_or(0);
                        if start < 0 {
                            start = (len + start).max(0);
                        }
                        let start = start.min(len) as usize;
                        for value in values.iter().skip(start) {
                            if self.strict_equal(value, &search) {
                                return Ok(Value::Bool(true));
                            }
                        }
                        Ok(Value::Bool(false))
                    }
                    Some(Value::TypedArray(values)) => {
                        let values_vec = self.typed_array_snapshot(values)?;
                        let len = values_vec.len() as i64;
                        let mut start = from_index
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .unwrap_or(0);
                        if start < 0 {
                            start = (len + start).max(0);
                        }
                        let start = start.min(len) as usize;
                        for value in values_vec.iter().skip(start) {
                            if self.strict_equal(value, &search) {
                                return Ok(Value::Bool(true));
                            }
                        }
                        Ok(Value::Bool(false))
                    }
                    Some(Value::String(value)) => {
                        let search = search.as_string();
                        let len = value.chars().count() as i64;
                        let mut start = from_index
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .unwrap_or(0);
                        if start < 0 {
                            start = (len + start).max(0);
                        }
                        let start = start.min(len) as usize;
                        let start_byte = Self::char_index_to_byte(value, start);
                        Ok(Value::Bool(value[start_byte..].contains(&search)))
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                }
            }
            Expr::ArraySlice { target, start, end } => match env.get(target) {
                Some(Value::Array(values)) => {
                    let values = values.borrow();
                    let len = values.len();
                    let start = start
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_slice_index(len, value))
                        .unwrap_or(0);
                    let end = end
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_slice_index(len, value))
                        .unwrap_or(len);
                    let end = end.max(start);
                    Ok(Self::new_array_value(values[start..end].to_vec()))
                }
                Some(Value::TypedArray(values)) => {
                    let snapshot = self.typed_array_snapshot(values)?;
                    let kind = values.borrow().kind;
                    let len = snapshot.len();
                    let start = start
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_slice_index(len, value))
                        .unwrap_or(0);
                    let end = end
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_slice_index(len, value))
                        .unwrap_or(len);
                    let end = end.max(start);
                    self.new_typed_array_from_values(kind, &snapshot[start..end])
                }
                Some(Value::ArrayBuffer(buffer)) => {
                    Self::ensure_array_buffer_not_detached(buffer, "slice")?;
                    let source = buffer.borrow();
                    let len = source.bytes.len();
                    let start = start
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_slice_index(len, value))
                        .unwrap_or(0);
                    let end = end
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_slice_index(len, value))
                        .unwrap_or(len);
                    let end = end.max(start);
                    Ok(Value::ArrayBuffer(Rc::new(RefCell::new(
                        ArrayBufferValue {
                            bytes: source.bytes[start..end].to_vec(),
                            max_byte_length: None,
                            detached: false,
                        },
                    ))))
                }
                Some(Value::Blob(blob)) => {
                    let source = blob.borrow();
                    let len = source.bytes.len();
                    let start = start
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_slice_index(len, value))
                        .unwrap_or(0);
                    let end = end
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_slice_index(len, value))
                        .unwrap_or(len);
                    let end = end.max(start);
                    Ok(Self::new_blob_value(
                        source.bytes[start..end].to_vec(),
                        String::new(),
                    ))
                }
                Some(Value::String(value)) => {
                    let len = value.chars().count();
                    let start = start
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_slice_index(len, value))
                        .unwrap_or(0);
                    let end = end
                        .as_ref()
                        .map(|value| self.eval_expr(value, env, event_param, event))
                        .transpose()?
                        .map(|value| Self::value_to_i64(&value))
                        .map(|value| Self::normalize_slice_index(len, value))
                        .unwrap_or(len);
                    let end = end.max(start);
                    Ok(Value::String(Self::substring_chars(value, start, end)))
                }
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                ))),
            },
            Expr::ArraySplice {
                target,
                start,
                delete_count,
                items,
            } => {
                let values = self.resolve_array_from_env(env, target)?;
                let start = self.eval_expr(start, env, event_param, event)?;
                let start = Self::value_to_i64(&start);
                let delete_count = delete_count
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value));
                let insert_items = self.eval_call_args_with_spread(items, env, event_param, event)?;

                let mut values = values.borrow_mut();
                let len = values.len();
                let start = Self::normalize_splice_start_index(len, start);
                let delete_count = delete_count
                    .unwrap_or((len.saturating_sub(start)) as i64)
                    .max(0) as usize;
                let delete_count = delete_count.min(len.saturating_sub(start));
                let removed = values
                    .drain(start..start + delete_count)
                    .collect::<Vec<_>>();
                for (offset, item) in insert_items.into_iter().enumerate() {
                    values.insert(start + offset, item);
                }
                Ok(Self::new_array_value(removed))
            }
            Expr::ArrayJoin { target, separator } => {
                let separator = separator
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| value.as_string())
                    .unwrap_or_else(|| ",".to_string());
                let values = match env.get(target) {
                    Some(Value::Array(values)) => values.borrow().clone(),
                    Some(Value::TypedArray(values)) => self.typed_array_snapshot(values)?,
                    Some(_) => {
                        return Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not an array",
                            target
                        )));
                    }
                    None => {
                        return Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        )));
                    }
                };
                let mut out = String::new();
                for (idx, value) in values.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(&separator);
                    }
                    if matches!(value, Value::Null | Value::Undefined) {
                        continue;
                    }
                    out.push_str(&value.as_string());
                }
                Ok(Value::String(out))
            }
            Expr::ArraySort { target, comparator } => {
                let comparator = comparator
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                if comparator
                    .as_ref()
                    .is_some_and(|value| !self.is_callable_value(value))
                {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }

                let values = self.resolve_array_from_env(env, target)?;
                let mut snapshot = values.borrow().clone();
                let len = snapshot.len();
                for i in 0..len {
                    let end = len.saturating_sub(i + 1);
                    for j in 0..end {
                        let should_swap = if let Some(comparator) = comparator.as_ref() {
                            let compared = self.execute_callable_value(
                                comparator,
                                &[snapshot[j].clone(), snapshot[j + 1].clone()],
                                event,
                            )?;
                            Self::coerce_number_for_global(&compared) > 0.0
                        } else {
                            snapshot[j].as_string() > snapshot[j + 1].as_string()
                        };
                        if should_swap {
                            snapshot.swap(j, j + 1);
                        }
                    }
                }
                *values.borrow_mut() = snapshot;
                Ok(Value::Array(values))
            }
            Expr::StringCharAt { value, index } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let len = value.chars().count();
                let index = index
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(0);
                if index < 0 || (index as usize) >= len {
                    Ok(Value::String(String::new()))
                } else {
                    Ok(value
                        .chars()
                        .nth(index as usize)
                        .map(|ch| Value::String(ch.to_string()))
                        .unwrap_or_else(|| Value::String(String::new())))
                }
            }
            Expr::StringCharCodeAt { value, index } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let len = value.chars().count();
                let index = index
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(0);
                if index < 0 || (index as usize) >= len {
                    Ok(Value::Float(f64::NAN))
                } else {
                    Ok(value
                        .chars()
                        .nth(index as usize)
                        .map(|ch| Value::Number(ch as i64))
                        .unwrap_or(Value::Float(f64::NAN)))
                }
            }
            Expr::StringCodePointAt { value, index } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let len = value.chars().count();
                let index = index
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(0);
                if index < 0 || (index as usize) >= len {
                    Ok(Value::Undefined)
                } else {
                    Ok(value
                        .chars()
                        .nth(index as usize)
                        .map(|ch| Value::Number(ch as i64))
                        .unwrap_or(Value::Undefined))
                }
            }
            Expr::StringAt { value, index } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let len = value.chars().count() as i64;
                let index = index
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(0);
                let index = if index < 0 { len + index } else { index };
                if index < 0 || index >= len {
                    Ok(Value::Undefined)
                } else {
                    Ok(value
                        .chars()
                        .nth(index as usize)
                        .map(|ch| Value::String(ch.to_string()))
                        .unwrap_or(Value::Undefined))
                }
            }
            Expr::StringConcat { value, args } => {
                let mut out = self.eval_expr(value, env, event_param, event)?.as_string();
                for arg in args {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    out.push_str(&value.as_string());
                }
                Ok(Value::String(out))
            }
            Expr::StringTrim { value, mode } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let value = match mode {
                    StringTrimMode::Both => value.trim().to_string(),
                    StringTrimMode::Start => value.trim_start().to_string(),
                    StringTrimMode::End => value.trim_end().to_string(),
                };
                Ok(Value::String(value))
            }
            Expr::StringToUpperCase(value) => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                Ok(Value::String(value.to_uppercase()))
            }
            Expr::StringToLowerCase(value) => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                Ok(Value::String(value.to_lowercase()))
            }
            Expr::StringIncludes {
                value,
                search,
                position,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let search = self.eval_expr(search, env, event_param, event)?;
                if matches!(search, Value::RegExp(_)) {
                    return Err(Error::ScriptRuntime(
                        "First argument to String.prototype.includes must not be a regular expression"
                            .into(),
                    ));
                }
                let search = search.as_string();
                let len = value.chars().count() as i64;
                let mut position = position
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(0);
                if position < 0 {
                    position = 0;
                }
                let position = position.min(len) as usize;
                let position_byte = Self::char_index_to_byte(&value, position);
                Ok(Value::Bool(value[position_byte..].contains(&search)))
            }
            Expr::StringStartsWith {
                value,
                search,
                position,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let search = self.eval_expr(search, env, event_param, event)?;
                if matches!(search, Value::RegExp(_)) {
                    return Err(Error::ScriptRuntime(
                        "First argument to String.prototype.startsWith must not be a regular expression"
                            .into(),
                    ));
                }
                let search = search.as_string();
                let len = value.chars().count() as i64;
                let mut position = position
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(0);
                if position < 0 {
                    position = 0;
                }
                let position = position.min(len) as usize;
                let position_byte = Self::char_index_to_byte(&value, position);
                Ok(Value::Bool(value[position_byte..].starts_with(&search)))
            }
            Expr::StringEndsWith {
                value,
                search,
                length,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let search = self.eval_expr(search, env, event_param, event)?;
                if matches!(search, Value::RegExp(_)) {
                    return Err(Error::ScriptRuntime(
                        "First argument to String.prototype.endsWith must not be a regular expression"
                            .into(),
                    ));
                }
                let search = search.as_string();
                let len = value.chars().count();
                let end = length
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .map(|value| {
                        if value < 0 {
                            0
                        } else {
                            (value as usize).min(len)
                        }
                    })
                    .unwrap_or(len);
                let hay = Self::substring_chars(&value, 0, end);
                Ok(Value::Bool(hay.ends_with(&search)))
            }
            Expr::StringSlice { value, start, end } => {
                let source = self.eval_expr(value, env, event_param, event)?;
                match source {
                    Value::Array(values) => {
                        let values_ref = values.borrow();
                        let len = values_ref.len();
                        let start = start
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(0);
                        let end = end
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(len);
                        let end = end.max(start);
                        Ok(Self::new_array_value(values_ref[start..end].to_vec()))
                    }
                    Value::TypedArray(values) => {
                        let snapshot = self.typed_array_snapshot(&values)?;
                        let kind = values.borrow().kind;
                        let len = snapshot.len();
                        let start = start
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(0);
                        let end = end
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(len);
                        let end = end.max(start);
                        self.new_typed_array_from_values(kind, &snapshot[start..end])
                    }
                    Value::ArrayBuffer(buffer) => {
                        Self::ensure_array_buffer_not_detached(&buffer, "slice")?;
                        let source = buffer.borrow();
                        let len = source.bytes.len();
                        let start = start
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(0);
                        let end = end
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(len);
                        let end = end.max(start);
                        Ok(Value::ArrayBuffer(Rc::new(RefCell::new(ArrayBufferValue {
                            bytes: source.bytes[start..end].to_vec(),
                            max_byte_length: None,
                            detached: false,
                        }))))
                    }
                    Value::Blob(blob) => {
                        let source = blob.borrow();
                        let len = source.bytes.len();
                        let start = start
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(0);
                        let end = end
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(len);
                        let end = end.max(start);
                        Ok(Self::new_blob_value(
                            source.bytes[start..end].to_vec(),
                            String::new(),
                        ))
                    }
                    other => {
                        let text = other.as_string();
                        let len = text.chars().count();
                        let start = start
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(0);
                        let end = end
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(len);
                        let end = end.max(start);
                        Ok(Value::String(Self::substring_chars(&text, start, end)))
                    }
                }
            }
            Expr::StringSubstring { value, start, end } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let len = value.chars().count();
                let start = start
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .map(|value| Self::normalize_substring_index(len, value))
                    .unwrap_or(0);
                let end = end
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .map(|value| Self::normalize_substring_index(len, value))
                    .unwrap_or(len);
                let (start, end) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                Ok(Value::String(Self::substring_chars(&value, start, end)))
            }
            Expr::StringMatch { value, pattern } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let pattern = self.eval_expr(pattern, env, event_param, event)?;
                self.eval_string_match(&value, pattern)
            }
            Expr::StringSplit {
                value,
                separator,
                limit,
            } => {
                let text = self.eval_expr(value, env, event_param, event)?.as_string();
                let separator = separator
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?;
                let limit = limit
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value));
                let parts = match separator {
                    None => Self::split_string(&text, None, limit),
                    Some(Value::RegExp(regex)) => {
                        Self::split_string_with_regex(&text, &regex, limit)?
                    }
                    Some(value) => Self::split_string(&text, Some(value.as_string()), limit),
                };
                Ok(Self::new_array_value(parts))
            }
            Expr::StringReplace { value, from, to } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let to = self.eval_expr(to, env, event_param, event)?.as_string();
                let from = self.eval_expr(from, env, event_param, event)?;
                let replaced = match from {
                    Value::RegExp(regex) => Self::replace_string_with_regex(&value, &regex, &to)?,
                    other => value.replacen(&other.as_string(), &to, 1),
                };
                Ok(Value::String(replaced))
            }
            Expr::StringReplaceAll { value, from, to } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let to = self.eval_expr(to, env, event_param, event)?.as_string();
                let from = self.eval_expr(from, env, event_param, event)?;
                let replaced = match from {
                    Value::RegExp(regex) => {
                        if !regex.borrow().global {
                            return Err(Error::ScriptRuntime(
                                "String.prototype.replaceAll called with a non-global RegExp argument"
                                    .into(),
                            ));
                        }
                        Self::replace_string_with_regex(&value, &regex, &to)?
                    }
                    other => {
                        let from = other.as_string();
                        if from.is_empty() {
                            let mut out = String::new();
                            for ch in value.chars() {
                                out.push_str(&to);
                                out.push(ch);
                            }
                            out.push_str(&to);
                            out
                        } else {
                            value.replace(&from, &to)
                        }
                    }
                };
                Ok(Value::String(replaced))
            }
            Expr::StringIndexOf {
                value,
                search,
                position,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let search = self.eval_expr(search, env, event_param, event)?.as_string();
                let len = value.chars().count() as i64;
                let mut position = position
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(0);
                if position < 0 {
                    position = 0;
                }
                let position = position.min(len) as usize;
                Ok(Value::Number(
                    Self::string_index_of(&value, &search, position)
                        .map(|value| value as i64)
                        .unwrap_or(-1),
                ))
            }
            Expr::StringLastIndexOf {
                value,
                search,
                position,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let search = self.eval_expr(search, env, event_param, event)?.as_string();
                let len = value.chars().count() as i64;
                let position = position
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(len);
                let position = if position < 0 { 0 } else { position.min(len) } as usize;
                let candidate = Self::substring_chars(&value, 0, position.saturating_add(1));
                let found = if search.is_empty() {
                    Some(position.min(candidate.chars().count()))
                } else {
                    candidate
                        .rfind(&search)
                        .map(|byte| candidate[..byte].chars().count())
                };
                Ok(Value::Number(found.map(|idx| idx as i64).unwrap_or(-1)))
            }
            Expr::StringSearch { value, pattern } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let pattern = self.eval_expr(pattern, env, event_param, event)?;
                let idx = match pattern {
                    Value::RegExp(regex) => {
                        let matched = regex
                            .borrow()
                            .compiled
                            .find(&value)
                            .map_err(Self::map_regex_runtime_error)?;
                        matched.map(|m| value[..m.start()].chars().count() as i64)
                    }
                    other => {
                        let search = other.as_string();
                        Self::string_index_of(&value, &search, 0).map(|idx| idx as i64)
                    }
                };
                Ok(Value::Number(idx.unwrap_or(-1)))
            }
            Expr::StringRepeat { value, count } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let count = self.eval_expr(count, env, event_param, event)?;
                let count = Self::value_to_i64(&count);
                if count < 0 {
                    return Err(Error::ScriptRuntime(
                        "Invalid count value for String.prototype.repeat".into(),
                    ));
                }
                let count = usize::try_from(count).map_err(|_| {
                    Error::ScriptRuntime("Invalid count value for String.prototype.repeat".into())
                })?;
                Ok(Value::String(value.repeat(count)))
            }
            Expr::StringPadStart {
                value,
                target_length,
                pad,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let target_length = self.eval_expr(target_length, env, event_param, event)?;
                let target_length = Self::value_to_i64(&target_length).max(0) as usize;
                let current_len = value.chars().count();
                if target_length <= current_len {
                    return Ok(Value::String(value));
                }
                let pad = pad
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| value.as_string())
                    .unwrap_or_else(|| " ".to_string());
                if pad.is_empty() {
                    return Ok(Value::String(value));
                }
                let mut filler = String::new();
                let needed = target_length - current_len;
                while filler.chars().count() < needed {
                    filler.push_str(&pad);
                }
                let filler = filler.chars().take(needed).collect::<String>();
                Ok(Value::String(format!("{filler}{value}")))
            }
            Expr::StringPadEnd {
                value,
                target_length,
                pad,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let target_length = self.eval_expr(target_length, env, event_param, event)?;
                let target_length = Self::value_to_i64(&target_length).max(0) as usize;
                let current_len = value.chars().count();
                if target_length <= current_len {
                    return Ok(Value::String(value));
                }
                let pad = pad
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| value.as_string())
                    .unwrap_or_else(|| " ".to_string());
                if pad.is_empty() {
                    return Ok(Value::String(value));
                }
                let mut filler = String::new();
                let needed = target_length - current_len;
                while filler.chars().count() < needed {
                    filler.push_str(&pad);
                }
                let filler = filler.chars().take(needed).collect::<String>();
                Ok(Value::String(format!("{value}{filler}")))
            }
            Expr::StringLocaleCompare {
                value,
                compare,
                locales,
                options,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let compare = self
                    .eval_expr(compare, env, event_param, event)?
                    .as_string();
                let locale = locales
                    .as_ref()
                    .map(|locales| self.eval_expr(locales, env, event_param, event))
                    .transpose()?
                    .map(|locales| self.intl_collect_locales(&locales))
                    .transpose()?
                    .and_then(|locales| locales.into_iter().next())
                    .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
                let mut case_first = "false".to_string();
                let mut sensitivity = "variant".to_string();
                if let Some(options) = options {
                    let options = self.eval_expr(options, env, event_param, event)?;
                    if let Value::Object(entries) = options {
                        let entries = entries.borrow();
                        if let Some(Value::String(value)) =
                            Self::object_get_entry(&entries, "caseFirst")
                        {
                            case_first = value;
                        }
                        if let Some(Value::String(value)) =
                            Self::object_get_entry(&entries, "sensitivity")
                        {
                            sensitivity = value;
                        }
                    }
                }
                Ok(Value::Number(Self::intl_collator_compare_strings(
                    &value,
                    &compare,
                    &locale,
                    &case_first,
                    &sensitivity,
                )))
            }
            Expr::StringIsWellFormed(value) => {
                let _ = self.eval_expr(value, env, event_param, event)?.as_string();
                Ok(Value::Bool(true))
            }
            Expr::StringToWellFormed(value) => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                Ok(Value::String(value))
            }
            Expr::StringValueOf(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                match value {
                    Value::Object(entries) => {
                        let entries_ref = entries.borrow();
                        if let Some(value) = Self::string_wrapper_value_from_object(&entries_ref) {
                            Ok(Value::String(value))
                        } else {
                            Ok(Value::Object(entries.clone()))
                        }
                    }
                    Value::String(value) => Ok(Value::String(value)),
                    other => Ok(other),
                }
            }
            Expr::StringToString(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                if let Value::Node(node) = &value {
                    if let Some(tag_name) = self.dom.tag_name(*node) {
                        if tag_name.eq_ignore_ascii_case("a") {
                            return Ok(Value::String(self.resolve_anchor_href(*node)));
                        }
                    }
                    return Ok(Value::String(Value::Node(*node).as_string()));
                }
                if let Value::Object(entries) = &value {
                    if Self::is_url_search_params_object(&entries.borrow()) {
                        let pairs =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        return Ok(Value::String(serialize_url_search_params_pairs(&pairs)));
                    }
                }
                Ok(Value::String(value.as_string()))
            }
            Expr::StructuredClone(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Self::structured_clone_value(&value, &mut Vec::new(), &mut Vec::new())
            }
            Expr::Fetch(request) => {
                let request = self
                    .eval_expr(request, env, event_param, event)?
                    .as_string();
                self.platform_mocks.fetch_calls.push(request.clone());
                let response = self.platform_mocks.fetch_mocks.get(&request).cloned().ok_or_else(|| {
                    Error::ScriptRuntime(format!("fetch mock not found for request: {request}"))
                })?;
                Ok(Value::String(response))
            }
            Expr::MatchMedia(query) => {
                let query = self.eval_expr(query, env, event_param, event)?.as_string();
                self.platform_mocks.match_media_calls.push(query.clone());
                let matches = self
                    .platform_mocks
                    .match_media_mocks
                    .get(&query)
                    .copied()
                    .unwrap_or(self.platform_mocks.default_match_media_matches);
                Ok(Self::new_object_value(vec![
                    ("matches".into(), Value::Bool(matches)),
                    ("media".into(), Value::String(query)),
                ]))
            }
            Expr::MatchMediaProp { query, prop } => {
                let query = self.eval_expr(query, env, event_param, event)?.as_string();
                self.platform_mocks.match_media_calls.push(query.clone());
                let matches = self
                    .platform_mocks
                    .match_media_mocks
                    .get(&query)
                    .copied()
                    .unwrap_or(self.platform_mocks.default_match_media_matches);
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
                    .unwrap_or_else(|| self.platform_mocks.default_prompt_response.clone().or(default_value));
                match response {
                    Some(value) => Ok(Value::String(value)),
                    None => Ok(Value::Null),
                }
            }
            Expr::FunctionConstructor { args } => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "new Function requires at least one argument".into(),
                    ));
                }

                let mut parts = Vec::with_capacity(args.len());
                for arg in args {
                    let part = self.eval_expr(arg, env, event_param, event)?.as_string();
                    parts.push(part);
                }

                let body_src = parts.last().cloned().ok_or_else(|| {
                    Error::ScriptRuntime("new Function requires body argument".into())
                })?;
                let mut params = Vec::new();
                for part in parts.iter().take(parts.len().saturating_sub(1)) {
                    let names = Self::parse_function_constructor_param_names(part)?;
                    params.extend(names.into_iter().map(|name| FunctionParam {
                        name,
                        default: None,
                        is_rest: false,
                    }));
                }

                let stmts = parse_block_statements(&body_src).map_err(|err| {
                    Error::ScriptRuntime(format!("new Function body parse failed: {err}"))
                })?;
                Ok(self.make_function_value(ScriptHandler { params, stmts }, env, true, false))
            }
            Expr::FunctionCall { target, args } => {
                let callee = if let Some(callee) = env.get(target).cloned() {
                    callee
                } else if let Some(callee) = self.resolve_pending_function_decl(target, env) {
                    callee
                } else {
                    return Err(Error::ScriptRuntime(format!("unknown variable: {target}")));
                };
                let evaluated_args = self.eval_call_args_with_spread(args, env, event_param, event)?;
                self.execute_callable_value_with_env(&callee, &evaluated_args, event, Some(env))
                    .map_err(|err| match err {
                        Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                            Error::ScriptRuntime(format!("'{target}' is not a function"))
                        }
                        other => other,
                    })
            }
            Expr::Call { target, args } => {
                let callee = self.eval_expr(target, env, event_param, event)?;
                let evaluated_args = self.eval_call_args_with_spread(args, env, event_param, event)?;
                self.execute_callable_value_with_env(&callee, &evaluated_args, event, Some(env))
                    .map_err(|err| match err {
                        Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                            Error::ScriptRuntime("call target is not a function".into())
                        }
                        other => other,
                    })
            }
            Expr::MemberCall {
                target,
                member,
                args,
                optional,
            } => {
                let receiver = self.eval_expr(target, env, event_param, event)?;
                if *optional && matches!(receiver, Value::Null | Value::Undefined) {
                    return Ok(Value::Undefined);
                }
                let evaluated_args = self.eval_call_args_with_spread(args, env, event_param, event)?;

                if let Value::Array(values) = &receiver {
                    if let Some(value) =
                        self.eval_array_member_call(values, member, &evaluated_args, event)?
                    {
                        return Ok(value);
                    }
                }

                if let Value::NodeList(nodes) = &receiver {
                    if let Some(value) =
                        self.eval_nodelist_member_call(nodes, member, &evaluated_args, event)?
                    {
                        return Ok(value);
                    }
                }

                if let Value::Node(node) = &receiver {
                    if let Some(value) =
                        self.eval_node_member_call(*node, member, &evaluated_args, event)?
                    {
                        return Ok(value);
                    }
                }

                if let Value::TypedArray(array) = &receiver {
                    if let Some(value) =
                        self.eval_typed_array_member_call(array, member, &evaluated_args)?
                    {
                        return Ok(value);
                    }
                }

                if let Value::Blob(blob) = &receiver {
                    if let Some(value) =
                        self.eval_blob_member_call(blob, member, &evaluated_args)?
                    {
                        return Ok(value);
                    }
                }

                if let Value::Map(map) = &receiver {
                    let map_member_override = {
                        let map_ref = map.borrow();
                        Self::object_get_entry(&map_ref.properties, member)
                    };
                    if let Some(callee) = map_member_override {
                        return self
                            .execute_callable_value_with_env(
                                &callee,
                                &evaluated_args,
                                event,
                                Some(env),
                            )
                            .map_err(|err| match err {
                                Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                                    Error::ScriptRuntime(format!("'{}' is not a function", member))
                                }
                                other => other,
                            });
                    }
                    if let Some(value) =
                        self.eval_map_member_call_from_values(map, member, &evaluated_args, event)?
                    {
                        return Ok(value);
                    }
                }

                if let Value::UrlConstructor = &receiver {
                    let url_constructor_override = {
                        let entries = self.browser_apis.url_constructor_properties.borrow();
                        Self::object_get_entry(&entries, member)
                    };
                    if let Some(callee) = url_constructor_override {
                        return self
                            .execute_callable_value_with_env(
                                &callee,
                                &evaluated_args,
                                event,
                                Some(env),
                            )
                            .map_err(|err| match err {
                                Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                                    Error::ScriptRuntime(format!("'{}' is not a function", member))
                                }
                                other => other,
                            });
                    }
                    if let Some(value) =
                        self.eval_url_static_member_call_from_values(member, &evaluated_args)?
                    {
                        return Ok(value);
                    }
                }

                if let Value::Object(object) = &receiver {
                    if Self::is_url_object(&object.borrow()) {
                        if let Some(value) =
                            self.eval_url_member_call(object, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                    }
                    if Self::is_url_search_params_object(&object.borrow()) {
                        if let Some(value) = self.eval_url_search_params_member_call(
                            object,
                            member,
                            &evaluated_args,
                            event,
                        )? {
                            return Ok(value);
                        }
                    }
                    if Self::is_storage_object(&object.borrow()) {
                        if let Some(value) =
                            self.eval_storage_member_call(object, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                    }
                }

                let callee = self
                    .object_property_from_value(&receiver, member)
                    .map_err(|err| match err {
                        Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                            Error::ScriptRuntime(format!(
                                "member call target does not support property '{}'",
                                member
                            ))
                        }
                        other => other,
                    })?;
                self.execute_callable_value_with_env(&callee, &evaluated_args, event, Some(env))
                    .map_err(|err| match err {
                        Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                            Error::ScriptRuntime(format!("'{}' is not a function", member))
                        }
                        other => other,
                    })
            }
            Expr::MemberGet {
                target,
                member,
                optional,
            } => {
                let receiver = self.eval_expr(target, env, event_param, event)?;
                if *optional && matches!(receiver, Value::Null | Value::Undefined) {
                    return Ok(Value::Undefined);
                }
                self.object_property_from_value(&receiver, member)
            }
            Expr::IndexGet {
                target,
                index,
                optional,
            } => {
                let receiver = self.eval_expr(target, env, event_param, event)?;
                if *optional && matches!(receiver, Value::Null | Value::Undefined) {
                    return Ok(Value::Undefined);
                }
                let index_value = self.eval_expr(index, env, event_param, event)?;
                let key = match index_value {
                    Value::Number(value) => value.to_string(),
                    Value::BigInt(value) => value.to_string(),
                    Value::Float(value) if value.is_finite() && value.fract() == 0.0 => {
                        format!("{:.0}", value)
                    }
                    other => other.as_string(),
                };
                self.object_property_from_value(&receiver, &key)
            }
            Expr::Var(name) => {
                if let Some(value) = env.get(name).cloned() {
                    Ok(value)
                } else if let Some(value) = self.resolve_pending_function_decl(name, env) {
                    Ok(value)
                } else {
                    Err(Error::ScriptRuntime(format!("unknown variable: {name}")))
                }
            }
            Expr::DomRef(target) => {
                let is_list_query = matches!(
                    target,
                    DomQuery::BySelectorAll { .. } | DomQuery::QuerySelectorAll { .. }
                );
                if is_list_query {
                    let nodes = self
                        .resolve_dom_query_list_runtime(target, env)?
                        .unwrap_or_default();
                    Ok(Value::NodeList(nodes))
                } else {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    Ok(Value::Node(node))
                }
            }
            Expr::CreateElement(tag_name) => {
                let node = self.dom.create_detached_element(tag_name.clone());
                Ok(Value::Node(node))
            }
            Expr::CreateTextNode(text) => {
                let node = self.dom.create_detached_text(text.clone());
                Ok(Value::Node(node))
            }
            Expr::Function { handler, is_async } => {
                Ok(self.make_function_value(handler.clone(), env, false, *is_async))
            }
            Expr::SetTimeout { handler, delay_ms } => {
                let delay = self.eval_expr(delay_ms, env, event_param, event)?;
                let delay = Self::value_to_i64(&delay);
                let callback_args = handler
                    .args
                    .iter()
                    .map(|arg| self.eval_expr(arg, env, event_param, event))
                    .collect::<Result<Vec<_>>>()?;
                let id = self.schedule_timeout(handler.callback.clone(), delay, callback_args, env);
                Ok(Value::Number(id))
            }
            Expr::SetInterval { handler, delay_ms } => {
                let interval = self.eval_expr(delay_ms, env, event_param, event)?;
                let interval = Self::value_to_i64(&interval);
                let callback_args = handler
                    .args
                    .iter()
                    .map(|arg| self.eval_expr(arg, env, event_param, event))
                    .collect::<Result<Vec<_>>>()?;
                let id =
                    self.schedule_interval(handler.callback.clone(), interval, callback_args, env);
                Ok(Value::Number(id))
            }
            Expr::RequestAnimationFrame { callback } => {
                const FRAME_DELAY_MS: i64 = 16;
                let callback_args = vec![Value::Number(self.scheduler.now_ms.saturating_add(FRAME_DELAY_MS))];
                let id =
                    self.schedule_timeout(callback.clone(), FRAME_DELAY_MS, callback_args, env);
                Ok(Value::Number(id))
            }
            Expr::QueueMicrotask { handler } => {
                self.queue_microtask(handler.clone(), env);
                Ok(Value::Null)
            }
            Expr::Binary { left, op, right } => match op {
                BinaryOp::And => {
                    let mut operands =
                        Self::collect_left_associative_binary_operands(expr, BinaryOp::And)
                            .into_iter();
                    let Some(first) = operands.next() else {
                        return Ok(Value::Undefined);
                    };
                    let mut current = self.eval_expr(first, env, event_param, event)?;
                    for operand in operands {
                        if !current.truthy() {
                            return Ok(current);
                        }
                        current = self.eval_expr(operand, env, event_param, event)?;
                    }
                    Ok(current)
                }
                BinaryOp::Or => {
                    let mut operands =
                        Self::collect_left_associative_binary_operands(expr, BinaryOp::Or)
                            .into_iter();
                    let Some(first) = operands.next() else {
                        return Ok(Value::Undefined);
                    };
                    let mut current = self.eval_expr(first, env, event_param, event)?;
                    for operand in operands {
                        if current.truthy() {
                            return Ok(current);
                        }
                        current = self.eval_expr(operand, env, event_param, event)?;
                    }
                    Ok(current)
                }
                BinaryOp::Nullish => {
                    let mut operands =
                        Self::collect_left_associative_binary_operands(expr, BinaryOp::Nullish)
                            .into_iter();
                    let Some(first) = operands.next() else {
                        return Ok(Value::Undefined);
                    };
                    let mut current = self.eval_expr(first, env, event_param, event)?;
                    for operand in operands {
                        if matches!(current, Value::Null | Value::Undefined) {
                            current = self.eval_expr(operand, env, event_param, event)?;
                        } else {
                            break;
                        }
                    }
                    Ok(current)
                }
                _ => {
                    let left = self.eval_expr(left, env, event_param, event)?;
                    let right = self.eval_expr(right, env, event_param, event)?;
                    self.eval_binary(op, &left, &right)
                }
            },
            Expr::DomRead { target, prop } => {
                let target_value = match target {
                    DomQuery::Var(name) => env.get(name).cloned(),
                    DomQuery::VarPath { base, path } => {
                        self.resolve_dom_query_var_path_value(base, path, env)?
                    }
                    _ => None,
                };
                if let Some(value) = target_value {
                    if !matches!(value, Value::Node(_) | Value::NodeList(_)) {
                        if let Some(key) = Self::object_key_from_dom_prop(prop) {
                            let variable_name = target.describe_call();
                            return self.object_property_from_named_value(
                                &variable_name,
                                &value,
                                key,
                            );
                        }
                    }
                }
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                match prop {
                    DomProp::Attributes => {
                        let element = self.dom.element(node).ok_or_else(|| {
                            Error::ScriptRuntime("attributes target is not an element".into())
                        })?;
                        let mut attrs = element
                            .attrs
                            .iter()
                            .map(|(name, value)| (name.clone(), Value::String(value.clone())))
                            .collect::<Vec<_>>();
                        attrs.sort_by(|(left, _), (right, _)| left.cmp(right));
                        attrs.insert(0, ("length".to_string(), Value::Number(attrs.len() as i64)));
                        Ok(Self::new_object_value(attrs))
                    }
                    DomProp::AssignedSlot => Ok(Value::Null),
                    DomProp::Value => Ok(Value::String(self.dom.value(node)?)),
                    DomProp::ValueLength => {
                        Ok(Value::Number(self.dom.value(node)?.chars().count() as i64))
                    }
                    DomProp::ValidationMessage => {
                        let validity = self.compute_input_validity(node)?;
                        if validity.custom_error {
                            Ok(Value::String(self.dom.custom_validity_message(node)?))
                        } else {
                            Ok(Value::String(String::new()))
                        }
                    }
                    DomProp::Validity => {
                        let validity = self.compute_input_validity(node)?;
                        Ok(Self::input_validity_to_value(&validity))
                    }
                    DomProp::ValidityValueMissing => {
                        Ok(Value::Bool(self.compute_input_validity(node)?.value_missing))
                    }
                    DomProp::ValidityTypeMismatch => {
                        Ok(Value::Bool(self.compute_input_validity(node)?.type_mismatch))
                    }
                    DomProp::ValidityPatternMismatch => Ok(Value::Bool(
                        self.compute_input_validity(node)?.pattern_mismatch,
                    )),
                    DomProp::ValidityTooLong => {
                        Ok(Value::Bool(self.compute_input_validity(node)?.too_long))
                    }
                    DomProp::ValidityTooShort => {
                        Ok(Value::Bool(self.compute_input_validity(node)?.too_short))
                    }
                    DomProp::ValidityRangeUnderflow => Ok(Value::Bool(
                        self.compute_input_validity(node)?.range_underflow,
                    )),
                    DomProp::ValidityRangeOverflow => Ok(Value::Bool(
                        self.compute_input_validity(node)?.range_overflow,
                    )),
                    DomProp::ValidityStepMismatch => Ok(Value::Bool(
                        self.compute_input_validity(node)?.step_mismatch,
                    )),
                    DomProp::ValidityBadInput => {
                        Ok(Value::Bool(self.compute_input_validity(node)?.bad_input))
                    }
                    DomProp::ValidityValid => {
                        Ok(Value::Bool(self.compute_input_validity(node)?.valid))
                    }
                    DomProp::ValidityCustomError => {
                        Ok(Value::Bool(self.compute_input_validity(node)?.custom_error))
                    }
                    DomProp::SelectionStart => Ok(Value::Number(
                        self.dom.selection_start(node).unwrap_or_default() as i64,
                    )),
                    DomProp::SelectionEnd => Ok(Value::Number(
                        self.dom.selection_end(node).unwrap_or_default() as i64,
                    )),
                    DomProp::SelectionDirection => Ok(Value::String(
                        self.dom
                            .selection_direction(node)
                            .unwrap_or_else(|_| "none".to_string()),
                    )),
                    DomProp::Checked => Ok(Value::Bool(self.dom.checked(node)?)),
                    DomProp::Indeterminate => Ok(Value::Bool(self.dom.indeterminate(node)?)),
                    DomProp::Open => Ok(Value::Bool(self.dom.has_attr(node, "open")?)),
                    DomProp::ReturnValue => Ok(Value::String(self.dialog_return_value(node)?)),
                    DomProp::ClosedBy => Ok(Value::String(
                        self.dom.attr(node, "closedby").unwrap_or_default(),
                    )),
                    DomProp::Readonly => Ok(Value::Bool(self.dom.readonly(node))),
                    DomProp::Disabled => Ok(Value::Bool(self.dom.disabled(node))),
                    DomProp::Required => Ok(Value::Bool(self.dom.required(node))),
                    DomProp::TextContent => Ok(Value::String(self.dom.text_content(node))),
                    DomProp::InnerText => Ok(Value::String(self.dom.text_content(node))),
                    DomProp::InnerHtml => Ok(Value::String(self.dom.inner_html(node)?)),
                    DomProp::OuterHtml => Ok(Value::String(self.dom.outer_html(node)?)),
                    DomProp::ClassName => Ok(Value::String(
                        self.dom.attr(node, "class").unwrap_or_default(),
                    )),
                    DomProp::ClassList => Ok(Self::new_array_value(
                        class_tokens(self.dom.attr(node, "class").as_deref())
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    )),
                    DomProp::ClassListLength => Ok(Value::Number(
                        class_tokens(self.dom.attr(node, "class").as_deref()).len() as i64,
                    )),
                    DomProp::Part => Ok(Self::new_array_value(
                        class_tokens(self.dom.attr(node, "part").as_deref())
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    )),
                    DomProp::PartLength => Ok(Value::Number(
                        class_tokens(self.dom.attr(node, "part").as_deref()).len() as i64,
                    )),
                    DomProp::Id => Ok(Value::String(self.dom.attr(node, "id").unwrap_or_default())),
                    DomProp::TagName => Ok(Value::String(
                        self.dom
                            .tag_name(node)
                            .map(|name| name.to_ascii_uppercase())
                            .unwrap_or_default(),
                    )),
                    DomProp::LocalName => Ok(Value::String(
                        self.dom
                            .tag_name(node)
                            .map(|name| name.to_ascii_lowercase())
                            .unwrap_or_default(),
                    )),
                    DomProp::NamespaceUri => {
                        if self.dom.element(node).is_some() {
                            Ok(Value::String("http://www.w3.org/1999/xhtml".to_string()))
                        } else {
                            Ok(Value::Null)
                        }
                    }
                    DomProp::Prefix => Ok(Value::Null),
                    DomProp::NextElementSibling => Ok(self
                        .dom
                        .next_element_sibling(node)
                        .map(Value::Node)
                        .unwrap_or(Value::Null)),
                    DomProp::PreviousElementSibling => Ok(self
                        .dom
                        .previous_element_sibling(node)
                        .map(Value::Node)
                        .unwrap_or(Value::Null)),
                    DomProp::Slot => Ok(Value::String(self.dom.attr(node, "slot").unwrap_or_default())),
                    DomProp::Role => Ok(Value::String(self.dom.attr(node, "role").unwrap_or_default())),
                    DomProp::ElementTiming => Ok(Value::String(
                        self.dom.attr(node, "elementtiming").unwrap_or_default(),
                    )),
                    DomProp::Name => Ok(Value::String(
                        self.dom.attr(node, "name").unwrap_or_default(),
                    )),
                    DomProp::Lang => Ok(Value::String(
                        self.dom.attr(node, "lang").unwrap_or_default(),
                    )),
                    DomProp::ClientWidth => Ok(Value::Number(self.dom.offset_width(node)?)),
                    DomProp::ClientHeight => Ok(Value::Number(self.dom.offset_height(node)?)),
                    DomProp::ClientLeft => Ok(Value::Number(self.dom.offset_left(node)?)),
                    DomProp::ClientTop => Ok(Value::Number(self.dom.offset_top(node)?)),
                    DomProp::CurrentCssZoom => Ok(Value::Number(1)),
                    DomProp::Dataset(key) => Ok(Value::String(self.dom.dataset_get(node, key)?)),
                    DomProp::Style(prop) => Ok(Value::String(self.dom.style_get(node, prop)?)),
                    DomProp::OffsetWidth => Ok(Value::Number(self.dom.offset_width(node)?)),
                    DomProp::OffsetHeight => Ok(Value::Number(self.dom.offset_height(node)?)),
                    DomProp::OffsetLeft => Ok(Value::Number(self.dom.offset_left(node)?)),
                    DomProp::OffsetTop => Ok(Value::Number(self.dom.offset_top(node)?)),
                    DomProp::ScrollWidth => Ok(Value::Number(self.dom.scroll_width(node)?)),
                    DomProp::ScrollHeight => Ok(Value::Number(self.dom.scroll_height(node)?)),
                    DomProp::ScrollLeft => Ok(Value::Number(self.dom.scroll_left(node)?)),
                    DomProp::ScrollTop => Ok(Value::Number(self.dom.scroll_top(node)?)),
                    DomProp::ScrollLeftMax => Ok(Value::Number(0)),
                    DomProp::ScrollTopMax => Ok(Value::Number(0)),
                    DomProp::ShadowRoot => Ok(Value::Null),
                    DomProp::ActiveElement => Ok(self
                        .dom
                        .active_element()
                        .map(Value::Node)
                        .unwrap_or(Value::Null)),
                    DomProp::CharacterSet => Ok(Value::String("UTF-8".to_string())),
                    DomProp::CompatMode => Ok(Value::String("CSS1Compat".to_string())),
                    DomProp::ContentType => Ok(Value::String("text/html".to_string())),
                    DomProp::ReadyState => Ok(Value::String("complete".to_string())),
                    DomProp::Referrer => Ok(Value::String(String::new())),
                    DomProp::Title => Ok(Value::String(self.dom.document_title())),
                    DomProp::Url | DomProp::DocumentUri => {
                        Ok(Value::String(self.document_url.clone()))
                    }
                    DomProp::Location => Ok(Value::Object(self.dom_runtime.location_object.clone())),
                    DomProp::LocationHref => Ok(Value::String(self.document_url.clone())),
                    DomProp::LocationProtocol => {
                        Ok(Value::String(self.current_location_parts().protocol()))
                    }
                    DomProp::LocationHost => {
                        Ok(Value::String(self.current_location_parts().host()))
                    }
                    DomProp::LocationHostname => {
                        Ok(Value::String(self.current_location_parts().hostname))
                    }
                    DomProp::LocationPort => Ok(Value::String(self.current_location_parts().port)),
                    DomProp::LocationPathname => {
                        let parts = self.current_location_parts();
                        Ok(Value::String(if parts.has_authority {
                            parts.pathname
                        } else {
                            parts.opaque_path
                        }))
                    }
                    DomProp::LocationSearch => {
                        Ok(Value::String(self.current_location_parts().search))
                    }
                    DomProp::LocationHash => Ok(Value::String(self.current_location_parts().hash)),
                    DomProp::LocationOrigin => {
                        Ok(Value::String(self.current_location_parts().origin()))
                    }
                    DomProp::LocationAncestorOrigins => Ok(Self::new_array_value(Vec::new())),
                    DomProp::History => Ok(Value::Object(self.location_history.history_object.clone())),
                    DomProp::HistoryLength => Ok(Value::Number(self.location_history.history_entries.len() as i64)),
                    DomProp::HistoryState => Ok(self.current_history_state()),
                    DomProp::HistoryScrollRestoration => {
                        Ok(Value::String(self.location_history.history_scroll_restoration.clone()))
                    }
                    DomProp::DefaultView => {
                        Ok(env.get("window").cloned().unwrap_or(Value::Undefined))
                    }
                    DomProp::Hidden => {
                        if node == self.dom.root {
                            Ok(Value::Bool(false))
                        } else {
                            Ok(Value::Bool(self.dom.attr(node, "hidden").is_some()))
                        }
                    }
                    DomProp::VisibilityState => Ok(Value::String("visible".to_string())),
                    DomProp::Forms => Ok(Value::NodeList(self.dom.query_selector_all("form")?)),
                    DomProp::Images => Ok(Value::NodeList(self.dom.query_selector_all("img")?)),
                    DomProp::Links => Ok(Value::NodeList(
                        self.dom.query_selector_all("a[href], area[href]")?,
                    )),
                    DomProp::Scripts => Ok(Value::NodeList(self.dom.query_selector_all("script")?)),
                    DomProp::Children => Ok(Value::NodeList(self.dom.child_elements(node))),
                    DomProp::ChildElementCount => {
                        Ok(Value::Number(self.dom.child_element_count(node) as i64))
                    }
                    DomProp::FirstElementChild => Ok(self
                        .dom
                        .first_element_child(node)
                        .map(Value::Node)
                        .unwrap_or(Value::Null)),
                    DomProp::LastElementChild => Ok(self
                        .dom
                        .last_element_child(node)
                        .map(Value::Node)
                        .unwrap_or(Value::Null)),
                    DomProp::CurrentScript => Ok(Value::Null),
                    DomProp::FormsLength => Ok(Value::Number(
                        self.dom.query_selector_all("form")?.len() as i64,
                    )),
                    DomProp::ImagesLength => Ok(Value::Number(
                        self.dom.query_selector_all("img")?.len() as i64,
                    )),
                    DomProp::LinksLength => Ok(Value::Number(
                        self.dom.query_selector_all("a[href], area[href]")?.len() as i64,
                    )),
                    DomProp::ScriptsLength => Ok(Value::Number(
                        self.dom.query_selector_all("script")?.len() as i64,
                    )),
                    DomProp::ChildrenLength => {
                        Ok(Value::Number(self.dom.child_element_count(node) as i64))
                    }
                    DomProp::AriaString(prop_name) => Ok(Value::String(
                        self.dom
                            .attr(node, &Self::aria_property_to_attr_name(prop_name))
                            .unwrap_or_default(),
                    )),
                    DomProp::AriaElementRefSingle(prop_name) => Ok(self
                        .resolve_aria_single_element_property(node, prop_name)
                        .map(Value::Node)
                        .unwrap_or(Value::Null)),
                    DomProp::AriaElementRefList(prop_name) => Ok(Value::NodeList(
                        self.resolve_aria_element_list_property(node, prop_name),
                    )),
                    DomProp::AnchorAttributionSrc => Ok(Value::String(
                        self.dom.attr(node, "attributionsrc").unwrap_or_default(),
                    )),
                    DomProp::AnchorDownload => Ok(Value::String(
                        self.dom.attr(node, "download").unwrap_or_default(),
                    )),
                    DomProp::AnchorHash => Ok(Value::String(self.anchor_location_parts(node).hash)),
                    DomProp::AnchorHost => {
                        Ok(Value::String(self.anchor_location_parts(node).host()))
                    }
                    DomProp::AnchorHostname => {
                        Ok(Value::String(self.anchor_location_parts(node).hostname))
                    }
                    DomProp::AnchorHref => Ok(Value::String(self.resolve_anchor_href(node))),
                    DomProp::AnchorHreflang => Ok(Value::String(
                        self.dom.attr(node, "hreflang").unwrap_or_default(),
                    )),
                    DomProp::AnchorInterestForElement => Ok(Value::String(
                        self.dom.attr(node, "interestfor").unwrap_or_default(),
                    )),
                    DomProp::AnchorOrigin => {
                        Ok(Value::String(self.anchor_location_parts(node).origin()))
                    }
                    DomProp::AnchorPassword => {
                        Ok(Value::String(self.anchor_location_parts(node).password))
                    }
                    DomProp::AnchorPathname => {
                        let parts = self.anchor_location_parts(node);
                        Ok(Value::String(if parts.has_authority {
                            parts.pathname
                        } else {
                            parts.opaque_path
                        }))
                    }
                    DomProp::AnchorPing => Ok(Value::String(
                        self.dom.attr(node, "ping").unwrap_or_default(),
                    )),
                    DomProp::AnchorPort => Ok(Value::String(self.anchor_location_parts(node).port)),
                    DomProp::AnchorProtocol => {
                        Ok(Value::String(self.anchor_location_parts(node).protocol()))
                    }
                    DomProp::AnchorReferrerPolicy => Ok(Value::String(
                        self.dom.attr(node, "referrerpolicy").unwrap_or_default(),
                    )),
                    DomProp::AnchorRel => Ok(Value::String(
                        self.dom.attr(node, "rel").unwrap_or_default(),
                    )),
                    DomProp::AnchorRelList => Ok(Self::new_array_value(
                        self.anchor_rel_tokens(node)
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    )),
                    DomProp::AnchorRelListLength => {
                        Ok(Value::Number(self.anchor_rel_tokens(node).len() as i64))
                    }
                    DomProp::AnchorSearch => {
                        Ok(Value::String(self.anchor_location_parts(node).search))
                    }
                    DomProp::AnchorTarget => Ok(Value::String(
                        self.dom.attr(node, "target").unwrap_or_default(),
                    )),
                    DomProp::AnchorText => Ok(Value::String(self.dom.text_content(node))),
                    DomProp::AnchorType => Ok(Value::String(
                        self.dom.attr(node, "type").unwrap_or_default(),
                    )),
                    DomProp::AnchorUsername => {
                        Ok(Value::String(self.anchor_location_parts(node).username))
                    }
                    DomProp::AnchorCharset => Ok(Value::String(
                        self.dom.attr(node, "charset").unwrap_or_default(),
                    )),
                    DomProp::AnchorCoords => Ok(Value::String(
                        self.dom.attr(node, "coords").unwrap_or_default(),
                    )),
                    DomProp::AnchorRev => Ok(Value::String(
                        self.dom.attr(node, "rev").unwrap_or_default(),
                    )),
                    DomProp::AnchorShape => Ok(Value::String(
                        self.dom.attr(node, "shape").unwrap_or_default(),
                    )),
                }
            }
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
                    Ok(Value::Undefined)
                }
                LocationMethod::Reload => {
                    self.reload_location()?;
                    Ok(Value::Undefined)
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
                    Ok(Value::Undefined)
                }
                LocationMethod::ToString => Ok(Value::String(self.document_url.clone())),
            },
            Expr::HistoryMethodCall { method, args } => match method {
                HistoryMethod::Back => {
                    let _ = args;
                    self.history_go_with_env(-1)?;
                    Ok(Value::Undefined)
                }
                HistoryMethod::Forward => {
                    let _ = args;
                    self.history_go_with_env(1)?;
                    Ok(Value::Undefined)
                }
                HistoryMethod::Go => {
                    let delta = if let Some(delta) = args.first() {
                        let value = self.eval_expr(delta, env, event_param, event)?;
                        Self::value_to_i64(&value)
                    } else {
                        0
                    };
                    self.history_go_with_env(delta)?;
                    Ok(Value::Undefined)
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
                    Ok(Value::Undefined)
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
                    Ok(Value::Undefined)
                }
            },
            Expr::ClipboardMethodCall { method, args } => match method {
                ClipboardMethod::ReadText => {
                    let _ = args;
                    let promise = self.new_pending_promise();
                    self.promise_resolve(&promise, Value::String(self.platform_mocks.clipboard_text.clone()))?;
                    Ok(Value::Promise(promise))
                }
                ClipboardMethod::WriteText => {
                    let text = self
                        .eval_expr(&args[0], env, event_param, event)?
                        .as_string();
                    self.platform_mocks.clipboard_text = text;
                    let promise = self.new_pending_promise();
                    self.promise_resolve(&promise, Value::Undefined)?;
                    Ok(Value::Promise(promise))
                }
            },
            Expr::DocumentHasFocus => Ok(Value::Bool(self.dom.active_element().is_some())),
            Expr::DomMatches { target, selector } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                let result = self.dom.matches_selector(node, selector)?;
                Ok(Value::Bool(result))
            }
            Expr::DomClosest { target, selector } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                let result = self.dom.closest(node, selector)?;
                Ok(result.map_or(Value::Null, Value::Node))
            }
            Expr::DomComputedStyleProperty { target, property } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Ok(Value::String(self.dom.style_get(node, property)?))
            }
            Expr::ClassListContains { target, class_name } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Ok(Value::Bool(self.dom.class_contains(node, class_name)?))
            }
            Expr::QuerySelectorAllLength { target } => {
                let len = self
                    .resolve_dom_query_list_runtime(target, env)?
                    .unwrap_or_default()
                    .len() as i64;
                Ok(Value::Number(len))
            }
            Expr::FormElementsLength { form } => {
                let form_node = self.resolve_dom_query_required_runtime(form, env)?;
                let len = self.form_elements(form_node)?.len() as i64;
                Ok(Value::Number(len))
            }
            Expr::FormDataNew { form } => {
                let form_node = self.resolve_dom_query_required_runtime(form, env)?;
                Ok(Value::FormData(self.form_data_entries(form_node)?))
            }
            Expr::FormDataGet { source, name } => {
                let entries = self.eval_form_data_source(source, env)?;
                let value = entries
                    .iter()
                    .find_map(|(entry_name, value)| (entry_name == name).then(|| value.clone()))
                    .unwrap_or_default();
                Ok(Value::String(value))
            }
            Expr::FormDataHas { source, name } => {
                let entries = self.eval_form_data_source(source, env)?;
                let has = entries.iter().any(|(entry_name, _)| entry_name == name);
                Ok(Value::Bool(has))
            }
            Expr::FormDataGetAll { source, name } => {
                let entries = self.eval_form_data_source(source, env)?;
                let values = entries
                    .iter()
                    .filter(|(entry_name, _)| entry_name == name)
                    .map(|(_, value)| Value::String(value.clone()))
                    .collect::<Vec<_>>();
                Ok(Self::new_array_value(values))
            }
            Expr::FormDataGetAllLength { source, name } => {
                let entries = self.eval_form_data_source(source, env)?;
                let len = entries
                    .iter()
                    .filter(|(entry_name, _)| entry_name == name)
                    .count() as i64;
                Ok(Value::Number(len))
            }
            Expr::DomGetAttribute { target, name } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Ok(Value::String(self.dom.attr(node, name).unwrap_or_default()))
            }
            Expr::DomHasAttribute { target, name } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Ok(Value::Bool(self.dom.has_attr(node, name)?))
            }
            Expr::EventProp { event_var, prop } => {
                if let Some(param) = event_param {
                    if param == event_var {
                        let value = match prop {
                            EventExprProp::Type => Value::String(event.event_type.clone()),
                            EventExprProp::Target => Value::Node(event.target),
                            EventExprProp::CurrentTarget => Value::Node(event.current_target),
                            EventExprProp::TargetName => Value::String(
                                self.dom.attr(event.target, "name").unwrap_or_default(),
                            ),
                            EventExprProp::CurrentTargetName => Value::String(
                                self.dom
                                    .attr(event.current_target, "name")
                                    .unwrap_or_default(),
                            ),
                            EventExprProp::DefaultPrevented => Value::Bool(event.default_prevented),
                            EventExprProp::IsTrusted => Value::Bool(event.is_trusted),
                            EventExprProp::Bubbles => Value::Bool(event.bubbles),
                            EventExprProp::Cancelable => Value::Bool(event.cancelable),
                            EventExprProp::TargetId => {
                                Value::String(self.dom.attr(event.target, "id").unwrap_or_default())
                            }
                            EventExprProp::CurrentTargetId => Value::String(
                                self.dom
                                    .attr(event.current_target, "id")
                                    .unwrap_or_default(),
                            ),
                            EventExprProp::EventPhase => Value::Number(event.event_phase as i64),
                            EventExprProp::TimeStamp => Value::Number(event.time_stamp_ms),
                            EventExprProp::State => {
                                event.state.as_ref().cloned().unwrap_or(Value::Undefined)
                            }
                            EventExprProp::OldState => event
                                .old_state
                                .as_ref()
                                .map(|value| Value::String(value.clone()))
                                .unwrap_or(Value::Undefined),
                            EventExprProp::NewState => event
                                .new_state
                                .as_ref()
                                .map(|value| Value::String(value.clone()))
                                .unwrap_or(Value::Undefined),
                        };
                        return Ok(value);
                    }
                }

                if let Some(value) = env.get(event_var) {
                    return self.eval_event_prop_fallback(event_var, value, *prop);
                }

                if event_param.is_none() {
                    return Err(Error::ScriptRuntime(format!(
                        "event variable '{}' is not available in this handler",
                        event_var
                    )));
                }
                Err(Error::ScriptRuntime(format!(
                    "unknown event variable: {}",
                    event_var
                )))
            }
            Expr::Neg(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                if matches!(value, Value::Symbol(_)) {
                    return Err(Error::ScriptRuntime(
                        "Cannot convert a Symbol value to a number".into(),
                    ));
                }
                match value {
                    Value::Number(v) => Ok(Value::Number(-v)),
                    Value::Float(v) => Ok(Value::Float(-v)),
                    Value::BigInt(v) => Ok(Value::BigInt(-v)),
                    other => Ok(Value::Float(-self.numeric_value(&other))),
                }
            }
            Expr::Pos(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                if matches!(value, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "unary plus is not supported for BigInt values".into(),
                    ));
                }
                if matches!(value, Value::Symbol(_)) {
                    return Err(Error::ScriptRuntime(
                        "Cannot convert a Symbol value to a number".into(),
                    ));
                }
                Ok(Value::Float(self.numeric_value(&value)))
            }
            Expr::BitNot(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                if matches!(value, Value::Symbol(_)) {
                    return Err(Error::ScriptRuntime(
                        "Cannot convert a Symbol value to a number".into(),
                    ));
                }
                if let Value::BigInt(v) = value {
                    return Ok(Value::BigInt(!v));
                }
                Ok(Value::Number((!self.to_i32_for_bitwise(&value)) as i64))
            }
            Expr::Not(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                Ok(Value::Bool(!value.truthy()))
            }
            Expr::Void(inner) => {
                self.eval_expr(inner, env, event_param, event)?;
                Ok(Value::Undefined)
            }
            Expr::Delete(inner) => match inner.as_ref() {
                Expr::Var(name) => Ok(Value::Bool(!env.contains_key(name))),
                _ => {
                    self.eval_expr(inner, env, event_param, event)?;
                    Ok(Value::Bool(true))
                }
            },
            Expr::TypeOf(inner) => {
                let js_type = match inner.as_ref() {
                    Expr::Var(name) => env.get(name).map_or("undefined", |value| match value {
                        Value::Null => "object",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) | Value::Float(_) => "number",
                        Value::BigInt(_) => "bigint",
                        Value::Symbol(_) => "symbol",
                        Value::Undefined => "undefined",
                        Value::String(_) => "string",
                        Value::StringConstructor => "function",
                        Value::TypedArrayConstructor(_)
                        | Value::BlobConstructor
                        | Value::UrlConstructor
                        | Value::ArrayBufferConstructor
                        | Value::PromiseConstructor
                        | Value::MapConstructor
                        | Value::SetConstructor
                        | Value::SymbolConstructor
                        | Value::RegExpConstructor
                        | Value::PromiseCapability(_) => "function",
                        Value::Function(_) => "function",
                        Value::Node(_)
                        | Value::NodeList(_)
                        | Value::FormData(_)
                        | Value::Array(_)
                        | Value::Object(_)
                        | Value::Map(_)
                        | Value::Set(_)
                        | Value::Blob(_)
                        | Value::Promise(_)
                        | Value::ArrayBuffer(_)
                        | Value::TypedArray(_)
                        | Value::RegExp(_)
                        | Value::Date(_) => "object",
                    }),
                    _ => {
                        let value = self.eval_expr(inner, env, event_param, event)?;
                        match value {
                            Value::Null => "object",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) | Value::Float(_) => "number",
                            Value::BigInt(_) => "bigint",
                            Value::Symbol(_) => "symbol",
                            Value::Undefined => "undefined",
                            Value::String(_) => "string",
                            Value::StringConstructor => "function",
                            Value::TypedArrayConstructor(_)
                            | Value::BlobConstructor
                            | Value::UrlConstructor
                            | Value::ArrayBufferConstructor
                            | Value::PromiseConstructor
                            | Value::MapConstructor
                            | Value::SetConstructor
                            | Value::SymbolConstructor
                            | Value::RegExpConstructor
                            | Value::PromiseCapability(_) => "function",
                            Value::Function(_) => "function",
                            Value::Node(_)
                            | Value::NodeList(_)
                            | Value::FormData(_)
                            | Value::Array(_)
                            | Value::Object(_)
                            | Value::Map(_)
                            | Value::Set(_)
                            | Value::Blob(_)
                            | Value::Promise(_)
                            | Value::ArrayBuffer(_)
                            | Value::TypedArray(_)
                            | Value::RegExp(_)
                            | Value::Date(_) => "object",
                        }
                    }
                };
                Ok(Value::String(js_type.to_string()))
            }
            Expr::Await(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                if let Value::Promise(promise) = value {
                    let settled = {
                        let promise = promise.borrow();
                        match &promise.state {
                            PromiseState::Pending => None,
                            PromiseState::Fulfilled(value) => {
                                Some(PromiseSettledValue::Fulfilled(value.clone()))
                            }
                            PromiseState::Rejected(reason) => {
                                Some(PromiseSettledValue::Rejected(reason.clone()))
                            }
                        }
                    };
                    match settled {
                        Some(PromiseSettledValue::Fulfilled(value)) => Ok(value),
                        Some(PromiseSettledValue::Rejected(reason)) => Err(Error::ScriptRuntime(
                            format!("await rejected Promise: {}", reason.as_string()),
                        )),
                        None => Ok(Value::Undefined),
                    }
                } else {
                    Ok(value)
                }
            }
            Expr::Yield(inner) => self.eval_expr(inner, env, event_param, event),
            Expr::YieldStar(inner) => self.eval_expr(inner, env, event_param, event),
            Expr::Comma(parts) => {
                let mut last = Value::Undefined;
                for part in parts {
                    last = self.eval_expr(part, env, event_param, event)?;
                }
                Ok(last)
            }
            Expr::Spread(_) => Err(Error::ScriptRuntime(
                "spread syntax is only supported in array literals, object literals, and call arguments".into(),
            )),
            Expr::Add(parts) => {
                if parts.is_empty() {
                    return Ok(Value::String(String::new()));
                }
                let mut iter = parts.iter();
                let first = iter
                    .next()
                    .ok_or_else(|| Error::ScriptRuntime("empty add expression".into()))?;
                let mut acc = self.eval_expr(first, env, event_param, event)?;
                for part in iter {
                    let rhs = self.eval_expr(part, env, event_param, event)?;
                    acc = self.add_values(&acc, &rhs)?;
                }
                Ok(acc)
            }
            Expr::Ternary {
                cond,
                on_true,
                on_false,
            } => {
                let cond = self.eval_expr(cond, env, event_param, event)?;
                if cond.truthy() {
                    self.eval_expr(on_true, env, event_param, event)
                } else {
                    self.eval_expr(on_false, env, event_param, event)
                }
            }
        }
    }

}
