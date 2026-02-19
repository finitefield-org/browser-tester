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
            Expr::DateNow => Ok(Value::Number(self.now_ms)),
            Expr::PerformanceNow => Ok(Value::Float(self.now_ms as f64)),
            Expr::DateNew { value } => {
                let timestamp_ms = if let Some(value) = value {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    self.coerce_date_timestamp_ms(&value)
                } else {
                    self.now_ms
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
                            self.now_ms
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
                            self.now_ms
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
                                    "variable '{}' is not an object",
                                    target
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
                                if let Some(symbol) = self.symbols_by_id.get(&symbol_id) {
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
                    _ => Ok(Value::Object(Rc::new(RefCell::new(Vec::new())))),
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
                            .unwrap_or(Value::Number(self.history_entries.len() as i64)));
                    }
                    if Self::is_window_object(&entries) {
                        return Ok(
                            Self::object_get_entry(&entries, "length").unwrap_or(Value::Number(0))
                        );
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
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
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
                Ok(Value::String(Self::substring_chars(&value, start, end)))
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
                        Self::split_string_with_regex(&text, &regex, limit)
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
                    Value::RegExp(regex) => Self::replace_string_with_regex(&value, &regex, &to),
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
                        Self::replace_string_with_regex(&value, &regex, &to)
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
                    Value::RegExp(regex) => regex
                        .borrow()
                        .compiled
                        .find(&value)
                        .map(|m| value[..m.start()].chars().count() as i64),
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
                self.fetch_calls.push(request.clone());
                let response = self.fetch_mocks.get(&request).cloned().ok_or_else(|| {
                    Error::ScriptRuntime(format!("fetch mock not found for request: {request}"))
                })?;
                Ok(Value::String(response))
            }
            Expr::MatchMedia(query) => {
                let query = self.eval_expr(query, env, event_param, event)?.as_string();
                self.match_media_calls.push(query.clone());
                let matches = self
                    .match_media_mocks
                    .get(&query)
                    .copied()
                    .unwrap_or(self.default_match_media_matches);
                Ok(Self::new_object_value(vec![
                    ("matches".into(), Value::Bool(matches)),
                    ("media".into(), Value::String(query)),
                ]))
            }
            Expr::MatchMediaProp { query, prop } => {
                let query = self.eval_expr(query, env, event_param, event)?.as_string();
                self.match_media_calls.push(query.clone());
                let matches = self
                    .match_media_mocks
                    .get(&query)
                    .copied()
                    .unwrap_or(self.default_match_media_matches);
                match prop {
                    MatchMediaProp::Matches => Ok(Value::Bool(matches)),
                    MatchMediaProp::Media => Ok(Value::String(query)),
                }
            }
            Expr::Alert(message) => {
                let message = self
                    .eval_expr(message, env, event_param, event)?
                    .as_string();
                self.alert_messages.push(message);
                Ok(Value::Undefined)
            }
            Expr::Confirm(message) => {
                let _ = self.eval_expr(message, env, event_param, event)?;
                let accepted = self
                    .confirm_responses
                    .pop_front()
                    .unwrap_or(self.default_confirm_response);
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
                    .prompt_responses
                    .pop_front()
                    .unwrap_or_else(|| self.default_prompt_response.clone().or(default_value));
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
            } => {
                let receiver = self.eval_expr(target, env, event_param, event)?;
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

                if let Value::UrlConstructor = &receiver {
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
                let callback_args = vec![Value::Number(self.now_ms.saturating_add(FRAME_DELAY_MS))];
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
                    DomProp::Checked => Ok(Value::Bool(self.dom.checked(node)?)),
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
                    DomProp::Location => Ok(Value::Object(self.location_object.clone())),
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
                    DomProp::History => Ok(Value::Object(self.history_object.clone())),
                    DomProp::HistoryLength => Ok(Value::Number(self.history_entries.len() as i64)),
                    DomProp::HistoryState => Ok(self.current_history_state()),
                    DomProp::HistoryScrollRestoration => {
                        Ok(Value::String(self.history_scroll_restoration.clone()))
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
                    self.promise_resolve(&promise, Value::String(self.clipboard_text.clone()))?;
                    Ok(Value::Promise(promise))
                }
                ClipboardMethod::WriteText => {
                    let text = self
                        .eval_expr(&args[0], env, event_param, event)?
                        .as_string();
                    self.clipboard_text = text;
                    let promise = self.new_pending_promise();
                    self.promise_resolve(&promise, Value::Undefined)?;
                    Ok(Value::Promise(promise))
                }
            },
            Expr::DocumentHasFocus => Ok(Value::Bool(self.active_element.is_some())),
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

    fn eval_array_member_call(
        &mut self,
        values: &Rc<RefCell<Vec<Value>>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let value = match member {
            "forEach" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "forEach requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                }
                Value::Undefined
            }
            "map" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "map requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut out = Vec::with_capacity(snapshot.len());
                for (idx, item) in snapshot.into_iter().enumerate() {
                    out.push(self.execute_callback_value(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?);
                }
                Self::new_array_value(out)
            }
            "filter" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "filter requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut out = Vec::new();
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let keep = self.execute_callback_value(
                        &callback,
                        &[
                            item.clone(),
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                    if keep.truthy() {
                        out.push(item);
                    }
                }
                Self::new_array_value(out)
            }
            "reduce" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "reduce requires callback and optional initial value".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut start_index = 0usize;
                let mut acc = if let Some(initial) = evaluated_args.get(1) {
                    initial.clone()
                } else {
                    let Some(first) = snapshot.first().cloned() else {
                        return Err(Error::ScriptRuntime(
                            "reduce of empty array with no initial value".into(),
                        ));
                    };
                    start_index = 1;
                    first
                };
                for (idx, item) in snapshot.into_iter().enumerate().skip(start_index) {
                    acc = self.execute_callback_value(
                        &callback,
                        &[
                            acc,
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                }
                acc
            }
            "find" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "find requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut found = Value::Undefined;
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let matched = self.execute_callback_value(
                        &callback,
                        &[
                            item.clone(),
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                    if matched.truthy() {
                        found = item;
                        break;
                    }
                }
                found
            }
            "some" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "some requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut matched = false;
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let keep = self.execute_callback_value(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                    if keep.truthy() {
                        matched = true;
                        break;
                    }
                }
                Value::Bool(matched)
            }
            "every" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "every requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = values.borrow().clone();
                let mut all = true;
                for (idx, item) in snapshot.into_iter().enumerate() {
                    let keep = self.execute_callback_value(
                        &callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        event,
                    )?;
                    if !keep.truthy() {
                        all = false;
                        break;
                    }
                }
                Value::Bool(all)
            }
            "includes" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "includes requires one or two arguments".into(),
                    ));
                }
                let search = evaluated_args[0].clone();
                let values_ref = values.borrow();
                let len = values_ref.len() as i64;
                let mut start = evaluated_args.get(1).map(Self::value_to_i64).unwrap_or(0);
                if start < 0 {
                    start = (len + start).max(0);
                }
                let start = start.min(len) as usize;
                let mut found = false;
                for value in values_ref.iter().skip(start) {
                    if self.strict_equal(value, &search) {
                        found = true;
                        break;
                    }
                }
                Value::Bool(found)
            }
            "slice" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "slice supports up to two arguments".into(),
                    ));
                }
                let values_ref = values.borrow();
                let len = values_ref.len();
                let start = evaluated_args
                    .first()
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(0);
                let end = evaluated_args
                    .get(1)
                    .map(Self::value_to_i64)
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(len);
                let end = end.max(start);
                Self::new_array_value(values_ref[start..end].to_vec())
            }
            "join" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "join supports zero or one separator argument".into(),
                    ));
                }
                let separator = evaluated_args
                    .first()
                    .map(Value::as_string)
                    .unwrap_or_else(|| ",".to_string());
                let values_ref = values.borrow();
                let mut out = String::new();
                for (idx, value) in values_ref.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(&separator);
                    }
                    if matches!(value, Value::Null | Value::Undefined) {
                        continue;
                    }
                    out.push_str(&value.as_string());
                }
                Value::String(out)
            }
            "push" => {
                let mut values_ref = values.borrow_mut();
                values_ref.extend(evaluated_args.iter().cloned());
                Value::Number(values_ref.len() as i64)
            }
            "pop" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("pop does not take arguments".into()));
                }
                values.borrow_mut().pop().unwrap_or(Value::Undefined)
            }
            "shift" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("shift does not take arguments".into()));
                }
                let mut values_ref = values.borrow_mut();
                if values_ref.is_empty() {
                    Value::Undefined
                } else {
                    values_ref.remove(0)
                }
            }
            "unshift" => {
                let mut values_ref = values.borrow_mut();
                for value in evaluated_args.iter().cloned().rev() {
                    values_ref.insert(0, value);
                }
                Value::Number(values_ref.len() as i64)
            }
            "splice" => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "splice requires at least a start index".into(),
                    ));
                }
                let start = Self::value_to_i64(&evaluated_args[0]);
                let delete_count = evaluated_args.get(1).map(Self::value_to_i64);
                let mut values_ref = values.borrow_mut();
                let len = values_ref.len();
                let start = Self::normalize_splice_start_index(len, start);
                let delete_count = delete_count
                    .unwrap_or((len.saturating_sub(start)) as i64)
                    .max(0) as usize;
                let delete_count = delete_count.min(len.saturating_sub(start));
                let removed = values_ref
                    .drain(start..start + delete_count)
                    .collect::<Vec<_>>();
                for (offset, item) in evaluated_args.iter().skip(2).cloned().enumerate() {
                    values_ref.insert(start + offset, item);
                }
                Self::new_array_value(removed)
            }
            "sort" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "sort supports zero or one comparator argument".into(),
                    ));
                }
                if evaluated_args
                    .first()
                    .is_some_and(|value| !self.is_callable_value(value))
                {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let comparator = evaluated_args.first().cloned();
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
                Value::Array(values.clone())
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    fn eval_nodelist_member_call(
        &mut self,
        nodes: &[NodeId],
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "forEach" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "forEach requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = nodes.to_vec();
                for (idx, node) in snapshot.iter().copied().enumerate() {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[
                            Value::Node(node),
                            Value::Number(idx as i64),
                            Value::NodeList(snapshot.clone()),
                        ],
                        event,
                    )?;
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }

    fn eval_node_member_call(
        &mut self,
        node: NodeId,
        member: &str,
        evaluated_args: &[Value],
        _event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "getAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                Ok(Some(Value::String(
                    self.dom.attr(node, &name).unwrap_or_default(),
                )))
            }
            "setAttribute" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "setAttribute requires exactly two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let value = evaluated_args[1].as_string();
                self.dom.set_attr(node, &name, &value)?;
                Ok(Some(Value::Undefined))
            }
            "hasAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "hasAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                Ok(Some(Value::Bool(self.dom.has_attr(node, &name)?)))
            }
            "hasAttributes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "hasAttributes takes no arguments".into(),
                    ));
                }
                let has_attributes = self
                    .dom
                    .element(node)
                    .map(|element| !element.attrs.is_empty())
                    .ok_or_else(|| {
                        Error::ScriptRuntime("hasAttributes target is not an element".into())
                    })?;
                Ok(Some(Value::Bool(has_attributes)))
            }
            "removeAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                self.dom.remove_attr(node, &name)?;
                Ok(Some(Value::Undefined))
            }
            "getAttributeNames" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNames takes no arguments".into(),
                    ));
                }
                let element = self.dom.element(node).ok_or_else(|| {
                    Error::ScriptRuntime("getAttributeNames target is not an element".into())
                })?;
                let mut names = element.attrs.keys().cloned().collect::<Vec<_>>();
                names.sort();
                Ok(Some(Self::new_array_value(
                    names.into_iter().map(Value::String).collect(),
                )))
            }
            "toggleAttribute" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "toggleAttribute requires one or two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let has = self.dom.has_attr(node, &name)?;
                let next = if evaluated_args.len() == 2 {
                    evaluated_args[1].truthy()
                } else {
                    !has
                };
                if next {
                    self.dom.set_attr(node, &name, "")?;
                } else {
                    self.dom.remove_attr(node, &name)?;
                }
                Ok(Some(Value::Bool(next)))
            }
            "matches" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "matches requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(Value::Bool(self.dom.matches_selector(node, &selector)?)))
            }
            "closest" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "closest requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom
                        .closest(node, &selector)?
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "querySelector" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelector requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom
                        .query_selector_from(&node, &selector)?
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "querySelectorAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelectorAll requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(Value::NodeList(
                    self.dom.query_selector_all_from(&node, &selector)?,
                )))
            }
            "getElementsByClassName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByClassName requires exactly one argument".into(),
                    ));
                }
                let classes = evaluated_args[0]
                    .as_string()
                    .split_whitespace()
                    .filter(|name| !name.is_empty())
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();
                if classes.is_empty() {
                    return Ok(Some(Value::NodeList(Vec::new())));
                }
                let selector = classes
                    .iter()
                    .map(|class_name| format!(".{class_name}"))
                    .collect::<String>();
                Ok(Some(Value::NodeList(
                    self.dom.query_selector_all_from(&node, &selector)?,
                )))
            }
            "getElementsByTagName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagName requires exactly one argument".into(),
                    ));
                }
                let tag_name = evaluated_args[0].as_string();
                if tag_name == "*" {
                    let mut nodes = Vec::new();
                    self.dom.collect_elements_descendants_dfs(node, &mut nodes);
                    return Ok(Some(Value::NodeList(nodes)));
                }
                Ok(Some(Value::NodeList(
                    self.dom.query_selector_all_from(&node, &tag_name)?,
                )))
            }
            "checkVisibility" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "checkVisibility supports at most one argument".into(),
                    ));
                }
                Ok(Some(Value::Bool(!self.dom.has_attr(node, "hidden")?)))
            }
            "scrollIntoView" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "scrollIntoView takes no arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "scroll" | "scrollTo" | "scrollBy" => {
                if !(evaluated_args.is_empty()
                    || evaluated_args.len() == 1
                    || evaluated_args.len() == 2)
                {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} supports zero, one, or two arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn collect_left_associative_binary_operands<'a>(
        expr: &'a Expr,
        op: BinaryOp,
    ) -> Vec<&'a Expr> {
        let mut right_operands = Vec::new();
        let mut cursor = expr;
        loop {
            match cursor {
                Expr::Binary {
                    left,
                    op: inner_op,
                    right,
                } if *inner_op == op => {
                    right_operands.push(right.as_ref());
                    cursor = left.as_ref();
                }
                _ => break,
            }
        }

        let mut out = Vec::with_capacity(right_operands.len() + 1);
        out.push(cursor);
        while let Some(operand) = right_operands.pop() {
            out.push(operand);
        }
        out
    }

    pub(super) fn eval_binary(&self, op: &BinaryOp, left: &Value, right: &Value) -> Result<Value> {
        if matches!(left, Value::Symbol(_)) || matches!(right, Value::Symbol(_)) {
            if matches!(
                op,
                BinaryOp::BitOr
                    | BinaryOp::BitXor
                    | BinaryOp::BitAnd
                    | BinaryOp::ShiftLeft
                    | BinaryOp::ShiftRight
                    | BinaryOp::UnsignedShiftRight
                    | BinaryOp::Pow
                    | BinaryOp::Lt
                    | BinaryOp::Gt
                    | BinaryOp::Le
                    | BinaryOp::Ge
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Mod
                    | BinaryOp::Div
            ) {
                return Err(Error::ScriptRuntime(
                    "Cannot convert a Symbol value to a number".into(),
                ));
            }
        }
        let out = match op {
            BinaryOp::Or => {
                if left.truthy() {
                    left.clone()
                } else {
                    right.clone()
                }
            }
            BinaryOp::And => {
                if left.truthy() {
                    right.clone()
                } else {
                    left.clone()
                }
            }
            BinaryOp::Nullish => {
                if matches!(left, Value::Null | Value::Undefined) {
                    right.clone()
                } else {
                    left.clone()
                }
            }
            BinaryOp::Eq => Value::Bool(self.loose_equal(left, right)),
            BinaryOp::Ne => Value::Bool(!self.loose_equal(left, right)),
            BinaryOp::StrictEq => Value::Bool(self.strict_equal(left, right)),
            BinaryOp::StrictNe => Value::Bool(!self.strict_equal(left, right)),
            BinaryOp::In => Value::Bool(self.value_in(left, right)),
            BinaryOp::InstanceOf => Value::Bool(self.value_instance_of(left, right)),
            BinaryOp::BitOr => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l | r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                Value::Number(i64::from(
                    self.to_i32_for_bitwise(left) | self.to_i32_for_bitwise(right),
                ))
            }
            BinaryOp::BitXor => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l ^ r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                Value::Number(i64::from(
                    self.to_i32_for_bitwise(left) ^ self.to_i32_for_bitwise(right),
                ))
            }
            BinaryOp::BitAnd => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l & r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                Value::Number(i64::from(
                    self.to_i32_for_bitwise(left) & self.to_i32_for_bitwise(right),
                ))
            }
            BinaryOp::ShiftLeft => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(Self::bigint_shift_left(l, r)?));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_i32_for_bitwise(left) << shift))
            }
            BinaryOp::ShiftRight => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(Self::bigint_shift_right(l, r)?));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_i32_for_bitwise(left) >> shift))
            }
            BinaryOp::UnsignedShiftRight => {
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "BigInt values do not support unsigned right shift".into(),
                    ));
                }
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_u32_for_bitwise(left) >> shift))
            }
            BinaryOp::Pow => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    if r.sign() == Sign::Minus {
                        return Err(Error::ScriptRuntime(
                            "BigInt exponent must be non-negative".into(),
                        ));
                    }
                    let exp = r.to_u32().ok_or_else(|| {
                        Error::ScriptRuntime("BigInt exponent is too large".into())
                    })?;
                    return Ok(Value::BigInt(l.pow(exp)));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                Value::Float(self.numeric_value(left).powf(self.numeric_value(right)))
            }
            BinaryOp::Lt => Value::Bool(self.compare(left, right, |l, r| l < r)),
            BinaryOp::Gt => Value::Bool(self.compare(left, right, |l, r| l > r)),
            BinaryOp::Le => Value::Bool(self.compare(left, right, |l, r| l <= r)),
            BinaryOp::Ge => Value::Bool(self.compare(left, right, |l, r| l >= r)),
            BinaryOp::Sub => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l - r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                Value::Float(self.numeric_value(left) - self.numeric_value(right))
            }
            BinaryOp::Mul => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l * r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                Value::Float(self.numeric_value(left) * self.numeric_value(right))
            }
            BinaryOp::Mod => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    if r.is_zero() {
                        return Err(Error::ScriptRuntime("modulo by zero".into()));
                    }
                    return Ok(Value::BigInt(l % r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                let rhs = self.numeric_value(right);
                if rhs == 0.0 {
                    return Err(Error::ScriptRuntime("modulo by zero".into()));
                }
                Value::Float(self.numeric_value(left) % rhs)
            }
            BinaryOp::Div => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    if r.is_zero() {
                        return Err(Error::ScriptRuntime("division by zero".into()));
                    }
                    return Ok(Value::BigInt(l / r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                let rhs = self.numeric_value(right);
                if rhs == 0.0 {
                    return Err(Error::ScriptRuntime("division by zero".into()));
                }
                Value::Float(self.numeric_value(left) / rhs)
            }
        };
        Ok(out)
    }

    pub(super) fn loose_equal(&self, left: &Value, right: &Value) -> bool {
        if self.strict_equal(left, right) {
            return true;
        }

        match (left, right) {
            (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null) => true,
            (Value::BigInt(l), Value::String(r)) => {
                Self::parse_js_bigint_from_string(r).is_ok_and(|parsed| parsed == *l)
            }
            (Value::String(l), Value::BigInt(r)) => {
                Self::parse_js_bigint_from_string(l).is_ok_and(|parsed| parsed == *r)
            }
            (Value::BigInt(_), Value::Number(_) | Value::Float(_))
            | (Value::Number(_) | Value::Float(_), Value::BigInt(_)) => {
                Self::number_bigint_loose_equal(left, right)
            }
            (Value::Number(_) | Value::Float(_), Value::String(_))
            | (Value::String(_), Value::Number(_) | Value::Float(_)) => {
                Self::coerce_number_for_global(left) == Self::coerce_number_for_global(right)
            }
            (Value::Bool(_), _) => {
                let coerced = Value::Float(Self::coerce_number_for_global(left));
                self.loose_equal(&coerced, right)
            }
            (_, Value::Bool(_)) => {
                let coerced = Value::Float(Self::coerce_number_for_global(right));
                self.loose_equal(left, &coerced)
            }
            _ if Self::is_loose_primitive(left) && Self::is_loose_object(right) => {
                let prim = self.to_primitive_for_loose(right);
                self.loose_equal(left, &prim)
            }
            _ if Self::is_loose_object(left) && Self::is_loose_primitive(right) => {
                let prim = self.to_primitive_for_loose(left);
                self.loose_equal(&prim, right)
            }
            _ => false,
        }
    }

    pub(super) fn is_loose_primitive(value: &Value) -> bool {
        matches!(
            value,
            Value::String(_)
                | Value::Bool(_)
                | Value::Number(_)
                | Value::Float(_)
                | Value::BigInt(_)
                | Value::Symbol(_)
                | Value::Null
                | Value::Undefined
        )
    }

    pub(super) fn is_loose_object(value: &Value) -> bool {
        matches!(
            value,
            Value::Array(_)
                | Value::Object(_)
                | Value::Promise(_)
                | Value::Map(_)
                | Value::Set(_)
                | Value::Blob(_)
                | Value::ArrayBuffer(_)
                | Value::TypedArray(_)
                | Value::StringConstructor
                | Value::TypedArrayConstructor(_)
                | Value::BlobConstructor
                | Value::UrlConstructor
                | Value::ArrayBufferConstructor
                | Value::PromiseConstructor
                | Value::MapConstructor
                | Value::SetConstructor
                | Value::SymbolConstructor
                | Value::RegExpConstructor
                | Value::PromiseCapability(_)
                | Value::RegExp(_)
                | Value::Date(_)
                | Value::Node(_)
                | Value::NodeList(_)
                | Value::FormData(_)
                | Value::Function(_)
        )
    }

    pub(super) fn to_primitive_for_loose(&self, value: &Value) -> Value {
        match value {
            Value::Object(entries) => {
                if let Some(wrapped) = Self::string_wrapper_value_from_object(&entries.borrow()) {
                    return Value::String(wrapped);
                }
                if let Some(id) = Self::symbol_wrapper_id_from_object(&entries.borrow()) {
                    if let Some(symbol) = self.symbols_by_id.get(&id) {
                        return Value::Symbol(symbol.clone());
                    }
                }
                Value::String(value.as_string())
            }
            Value::Array(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::RegExp(_)
            | Value::Date(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_) => Value::String(value.as_string()),
            _ => value.clone(),
        }
    }

    pub(super) fn value_in(&self, left: &Value, right: &Value) -> bool {
        match right {
            Value::NodeList(nodes) => self
                .value_as_index(left)
                .is_some_and(|index| index < nodes.len()),
            Value::Array(values) => self
                .value_as_index(left)
                .is_some_and(|index| index < values.borrow().len()),
            Value::TypedArray(values) => self
                .value_as_index(left)
                .is_some_and(|index| index < values.borrow().observed_length()),
            Value::Object(entries) => {
                let key = self.property_key_to_storage_key(left);
                entries.borrow().iter().any(|(name, _)| name == &key)
            }
            Value::FormData(entries) => {
                let key = left.as_string();
                entries.iter().any(|(name, _)| name == &key)
            }
            _ => false,
        }
    }

    pub(super) fn value_instance_of(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Node(left), Value::Node(right)) => left == right,
            (Value::Node(left), Value::NodeList(nodes)) => nodes.contains(left),
            (Value::Array(left), Value::Array(right)) => Rc::ptr_eq(left, right),
            (Value::Map(left), Value::Map(right)) => Rc::ptr_eq(left, right),
            (Value::Set(left), Value::Set(right)) => Rc::ptr_eq(left, right),
            (Value::Promise(left), Value::Promise(right)) => Rc::ptr_eq(left, right),
            (Value::TypedArray(left), Value::TypedArray(right)) => Rc::ptr_eq(left, right),
            (Value::Blob(left), Value::Blob(right)) => Rc::ptr_eq(left, right),
            (Value::ArrayBuffer(left), Value::ArrayBuffer(right)) => Rc::ptr_eq(left, right),
            (Value::Object(left), Value::Object(right)) => Rc::ptr_eq(left, right),
            (Value::RegExp(left), Value::RegExp(right)) => Rc::ptr_eq(left, right),
            (Value::Symbol(left), Value::Symbol(right)) => left.id == right.id,
            (Value::Date(left), Value::Date(right)) => Rc::ptr_eq(left, right),
            (Value::FormData(left), Value::FormData(right)) => left == right,
            (Value::Blob(_), Value::BlobConstructor) => true,
            (Value::Object(left), Value::UrlConstructor) => Self::is_url_object(&left.borrow()),
            (Value::Object(left), Value::StringConstructor) => {
                Self::string_wrapper_value_from_object(&left.borrow()).is_some()
            }
            _ => false,
        }
    }

    pub(super) fn value_as_index(&self, value: &Value) -> Option<usize> {
        match value {
            Value::Number(v) => usize::try_from(*v).ok(),
            Value::Float(v) => {
                if !v.is_finite() || v.fract() != 0.0 || *v < 0.0 {
                    None
                } else {
                    usize::try_from(*v as i64).ok()
                }
            }
            Value::BigInt(v) => v.to_usize(),
            Value::String(s) => {
                if let Ok(int) = s.parse::<i64>() {
                    usize::try_from(int).ok()
                } else if let Ok(float) = s.parse::<f64>() {
                    if float.fract() == 0.0 && float >= 0.0 {
                        usize::try_from(float as i64).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(super) fn strict_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::Float(l), Value::Float(r)) => l == r,
            (Value::Number(l), Value::Float(r)) => (*l as f64) == *r,
            (Value::Float(l), Value::Number(r)) => *l == (*r as f64),
            (Value::BigInt(l), Value::BigInt(r)) => l == r,
            (Value::Symbol(l), Value::Symbol(r)) => l.id == r.id,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Node(l), Value::Node(r)) => l == r,
            (Value::Array(l), Value::Array(r)) => Rc::ptr_eq(l, r),
            (Value::Map(l), Value::Map(r)) => Rc::ptr_eq(l, r),
            (Value::Set(l), Value::Set(r)) => Rc::ptr_eq(l, r),
            (Value::Promise(l), Value::Promise(r)) => Rc::ptr_eq(l, r),
            (Value::TypedArray(l), Value::TypedArray(r)) => Rc::ptr_eq(l, r),
            (Value::Blob(l), Value::Blob(r)) => Rc::ptr_eq(l, r),
            (Value::ArrayBuffer(l), Value::ArrayBuffer(r)) => Rc::ptr_eq(l, r),
            (Value::StringConstructor, Value::StringConstructor) => true,
            (Value::TypedArrayConstructor(l), Value::TypedArrayConstructor(r)) => l == r,
            (Value::BlobConstructor, Value::BlobConstructor) => true,
            (Value::UrlConstructor, Value::UrlConstructor) => true,
            (Value::ArrayBufferConstructor, Value::ArrayBufferConstructor) => true,
            (Value::PromiseConstructor, Value::PromiseConstructor) => true,
            (Value::MapConstructor, Value::MapConstructor) => true,
            (Value::SetConstructor, Value::SetConstructor) => true,
            (Value::SymbolConstructor, Value::SymbolConstructor) => true,
            (Value::RegExpConstructor, Value::RegExpConstructor) => true,
            (Value::PromiseCapability(l), Value::PromiseCapability(r)) => Rc::ptr_eq(l, r),
            (Value::Object(l), Value::Object(r)) => Rc::ptr_eq(l, r),
            (Value::RegExp(l), Value::RegExp(r)) => Rc::ptr_eq(l, r),
            (Value::Date(l), Value::Date(r)) => Rc::ptr_eq(l, r),
            (Value::Function(l), Value::Function(r)) => Rc::ptr_eq(l, r),
            (Value::FormData(l), Value::FormData(r)) => l == r,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Undefined) => true,
            _ => false,
        }
    }

    pub(super) fn compare<F>(&self, left: &Value, right: &Value, op: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Value::String(l), Value::String(r)) => {
                let ordering = l.cmp(r);
                let cmp = if ordering.is_lt() {
                    -1.0
                } else if ordering.is_gt() {
                    1.0
                } else {
                    0.0
                };
                return op(cmp, 0.0);
            }
            (Value::BigInt(l), Value::BigInt(r)) => {
                return op(
                    l.to_f64().unwrap_or_else(|| {
                        if l.sign() == Sign::Minus {
                            f64::NEG_INFINITY
                        } else {
                            f64::INFINITY
                        }
                    }),
                    r.to_f64().unwrap_or_else(|| {
                        if r.sign() == Sign::Minus {
                            f64::NEG_INFINITY
                        } else {
                            f64::INFINITY
                        }
                    }),
                );
            }
            (Value::BigInt(l), Value::Number(_) | Value::Float(_)) => {
                let r = self.numeric_value(right);
                if r.is_nan() {
                    return false;
                }
                if let Some(rb) = Self::f64_to_bigint_if_integral(r) {
                    return op(
                        l.to_f64().unwrap_or_else(|| {
                            if l.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                        rb.to_f64().unwrap_or_else(|| {
                            if rb.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                    );
                }
                return op(
                    l.to_f64().unwrap_or_else(|| {
                        if l.sign() == Sign::Minus {
                            f64::NEG_INFINITY
                        } else {
                            f64::INFINITY
                        }
                    }),
                    r,
                );
            }
            (Value::Number(_) | Value::Float(_), Value::BigInt(r)) => {
                let l = self.numeric_value(left);
                if l.is_nan() {
                    return false;
                }
                if let Some(lb) = Self::f64_to_bigint_if_integral(l) {
                    return op(
                        lb.to_f64().unwrap_or_else(|| {
                            if lb.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                        r.to_f64().unwrap_or_else(|| {
                            if r.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                    );
                }
                return op(
                    l,
                    r.to_f64().unwrap_or_else(|| {
                        if r.sign() == Sign::Minus {
                            f64::NEG_INFINITY
                        } else {
                            f64::INFINITY
                        }
                    }),
                );
            }
            _ => {}
        }
        let l = self.numeric_value(left);
        let r = self.numeric_value(right);
        op(l, r)
    }

    pub(super) fn number_bigint_loose_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::BigInt(l), Value::Number(r)) => *l == JsBigInt::from(*r),
            (Value::BigInt(l), Value::Float(r)) => {
                Self::f64_to_bigint_if_integral(*r).is_some_and(|rb| rb == *l)
            }
            (Value::Number(l), Value::BigInt(r)) => JsBigInt::from(*l) == *r,
            (Value::Float(l), Value::BigInt(r)) => {
                Self::f64_to_bigint_if_integral(*l).is_some_and(|lb| lb == *r)
            }
            _ => false,
        }
    }

    pub(super) fn f64_to_bigint_if_integral(value: f64) -> Option<JsBigInt> {
        if !value.is_finite() || value.fract() != 0.0 {
            return None;
        }
        if value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            return Some(JsBigInt::from(value as i64));
        }
        let rendered = format!("{value:.0}");
        JsBigInt::parse_bytes(rendered.as_bytes(), 10)
    }

    pub(super) fn add_values(&self, left: &Value, right: &Value) -> Result<Value> {
        if matches!(left, Value::Symbol(_)) || matches!(right, Value::Symbol(_)) {
            return Err(Error::ScriptRuntime(
                "Cannot convert a Symbol value to a string".into(),
            ));
        }
        if matches!(left, Value::String(_)) || matches!(right, Value::String(_)) {
            return Ok(Value::String(format!(
                "{}{}",
                left.as_string(),
                right.as_string()
            )));
        }

        if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
            return match (left, right) {
                (Value::BigInt(l), Value::BigInt(r)) => Ok(Value::BigInt(l + r)),
                _ => Err(Error::ScriptRuntime(
                    "cannot mix BigInt and other types in addition".into(),
                )),
            };
        }

        match (left, right) {
            (Value::Number(l), Value::Number(r)) => {
                if let Some(sum) = l.checked_add(*r) {
                    Ok(Value::Number(sum))
                } else {
                    Ok(Value::Float((*l as f64) + (*r as f64)))
                }
            }
            _ => Ok(Value::Float(
                self.numeric_value(left) + self.numeric_value(right),
            )),
        }
    }

    pub(super) fn new_array_value(values: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(values)))
    }

    pub(super) fn new_object_value(entries: Vec<(String, Value)>) -> Value {
        Value::Object(Rc::new(RefCell::new(entries)))
    }

    pub(super) fn new_boolean_constructor_callable() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_CALLABLE_KIND_KEY.to_string(),
            Value::String("boolean_constructor".to_string()),
        )])
    }

    pub(super) fn new_string_wrapper_value(value: String) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
            Value::String(value),
        )])
    }

    pub(super) fn object_set_entry(entries: &mut Vec<(String, Value)>, key: String, value: Value) {
        if let Some((_, existing)) = entries.iter_mut().find(|(name, _)| name == &key) {
            *existing = value;
        } else {
            entries.push((key, value));
        }
    }

    pub(super) fn object_get_entry(entries: &[(String, Value)], key: &str) -> Option<Value> {
        entries
            .iter()
            .find_map(|(name, value)| (name == key).then(|| value.clone()))
    }

    pub(super) fn callable_kind_from_value(value: &Value) -> Option<&str> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        match Self::object_get_entry(&entries, INTERNAL_CALLABLE_KIND_KEY) {
            Some(Value::String(kind)) => Some(match kind.as_str() {
                "intl_collator_compare" => "intl_collator_compare",
                "intl_date_time_format" => "intl_date_time_format",
                "intl_duration_format" => "intl_duration_format",
                "intl_list_format" => "intl_list_format",
                "intl_number_format" => "intl_number_format",
                "intl_segmenter_segments_iterator" => "intl_segmenter_segments_iterator",
                "intl_segmenter_iterator_next" => "intl_segmenter_iterator_next",
                "boolean_constructor" => "boolean_constructor",
                _ => return None,
            }),
            _ => None,
        }
    }

    pub(super) fn object_property_from_value(&self, value: &Value, key: &str) -> Result<Value> {
        match value {
            Value::String(text) => {
                if key == "length" {
                    Ok(Value::Number(text.chars().count() as i64))
                } else if key == "constructor" {
                    Ok(Value::StringConstructor)
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(text
                        .chars()
                        .nth(index)
                        .map(|ch| Value::String(ch.to_string()))
                        .unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Array(values) => {
                let values = values.borrow();
                if key == "length" {
                    Ok(Value::Number(values.len() as i64))
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(values.get(index).cloned().unwrap_or(Value::Undefined))
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(text) = Self::string_wrapper_value_from_object(&entries) {
                    if key == "length" {
                        return Ok(Value::Number(text.chars().count() as i64));
                    }
                    if key == "constructor" {
                        return Ok(Value::StringConstructor);
                    }
                    if let Ok(index) = key.parse::<usize>() {
                        return Ok(text
                            .chars()
                            .nth(index)
                            .map(|ch| Value::String(ch.to_string()))
                            .unwrap_or(Value::Undefined));
                    }
                }
                if Self::is_url_search_params_object(&entries) {
                    if key == "size" {
                        let size =
                            Self::url_search_params_pairs_from_object_entries(&entries).len();
                        return Ok(Value::Number(size as i64));
                    }
                }
                if Self::is_url_object(&entries) && key == "constructor" {
                    return Ok(Value::UrlConstructor);
                }
                Ok(Self::object_get_entry(&entries, key).unwrap_or(Value::Undefined))
            }
            Value::Promise(promise) => {
                if key == "constructor" {
                    Ok(Value::PromiseConstructor)
                } else {
                    let promise = promise.borrow();
                    if key == "status" {
                        let status = match &promise.state {
                            PromiseState::Pending => "pending",
                            PromiseState::Fulfilled(_) => "fulfilled",
                            PromiseState::Rejected(_) => "rejected",
                        };
                        Ok(Value::String(status.to_string()))
                    } else {
                        Ok(Value::Undefined)
                    }
                }
            }
            Value::Map(map) => {
                let map = map.borrow();
                if key == "size" {
                    Ok(Value::Number(map.entries.len() as i64))
                } else if key == "constructor" {
                    Ok(Value::MapConstructor)
                } else {
                    Ok(Self::object_get_entry(&map.properties, key).unwrap_or(Value::Undefined))
                }
            }
            Value::Set(set) => {
                let set = set.borrow();
                if key == "size" {
                    Ok(Value::Number(set.values.len() as i64))
                } else if key == "constructor" {
                    Ok(Value::SetConstructor)
                } else {
                    Ok(Self::object_get_entry(&set.properties, key).unwrap_or(Value::Undefined))
                }
            }
            Value::Blob(blob) => {
                let blob = blob.borrow();
                match key {
                    "size" => Ok(Value::Number(blob.bytes.len() as i64)),
                    "type" => Ok(Value::String(blob.mime_type.clone())),
                    "constructor" => Ok(Value::BlobConstructor),
                    _ => Ok(Value::Undefined),
                }
            }
            Value::ArrayBuffer(_) => {
                if key == "constructor" {
                    Ok(Value::ArrayBufferConstructor)
                } else {
                    Ok(Value::Undefined)
                }
            }
            Value::Symbol(symbol) => {
                let value = match key {
                    "description" => symbol
                        .description
                        .as_ref()
                        .map(|value| Value::String(value.clone()))
                        .unwrap_or(Value::Undefined),
                    "constructor" => Value::SymbolConstructor,
                    _ => Value::Undefined,
                };
                Ok(value)
            }
            Value::RegExp(regex) => {
                let regex = regex.borrow();
                let value = match key {
                    "source" => Value::String(regex.source.clone()),
                    "flags" => Value::String(regex.flags.clone()),
                    "global" => Value::Bool(regex.global),
                    "ignoreCase" => Value::Bool(regex.ignore_case),
                    "multiline" => Value::Bool(regex.multiline),
                    "dotAll" => Value::Bool(regex.dot_all),
                    "sticky" => Value::Bool(regex.sticky),
                    "hasIndices" => Value::Bool(regex.has_indices),
                    "unicode" => Value::Bool(regex.unicode),
                    "unicodeSets" => Value::Bool(false),
                    "lastIndex" => Value::Number(regex.last_index as i64),
                    "constructor" => Value::RegExpConstructor,
                    _ => Self::object_get_entry(&regex.properties, key).unwrap_or(Value::Undefined),
                };
                Ok(value)
            }
            Value::StringConstructor => Ok(Value::Undefined),
            _ => Err(Error::ScriptRuntime("value is not an object".into())),
        }
    }

    pub(super) fn object_property_from_named_value(
        &self,
        variable_name: &str,
        value: &Value,
        key: &str,
    ) -> Result<Value> {
        self.object_property_from_value(value, key)
            .map_err(|err| match err {
                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                    Error::ScriptRuntime(format!("variable '{}' is not an object", variable_name))
                }
                other => other,
            })
    }

    pub(super) fn eval_event_prop_fallback(
        &self,
        event_var: &str,
        value: &Value,
        prop: EventExprProp,
    ) -> Result<Value> {
        let read =
            |value: &Value, key: &str| self.object_property_from_named_value(event_var, value, key);
        match prop {
            EventExprProp::Type => read(value, "type"),
            EventExprProp::Target => read(value, "target"),
            EventExprProp::CurrentTarget => read(value, "currentTarget"),
            EventExprProp::TargetName => {
                let target = read(value, "target")?;
                read(&target, "name")
            }
            EventExprProp::CurrentTargetName => {
                let target = read(value, "currentTarget")?;
                read(&target, "name")
            }
            EventExprProp::DefaultPrevented => read(value, "defaultPrevented"),
            EventExprProp::IsTrusted => read(value, "isTrusted"),
            EventExprProp::Bubbles => read(value, "bubbles"),
            EventExprProp::Cancelable => read(value, "cancelable"),
            EventExprProp::TargetId => {
                let target = read(value, "target")?;
                read(&target, "id")
            }
            EventExprProp::CurrentTargetId => {
                let target = read(value, "currentTarget")?;
                read(&target, "id")
            }
            EventExprProp::EventPhase => read(value, "eventPhase"),
            EventExprProp::TimeStamp => read(value, "timeStamp"),
            EventExprProp::State => read(value, "state"),
            EventExprProp::OldState => read(value, "oldState"),
            EventExprProp::NewState => read(value, "newState"),
        }
    }

    pub(super) fn aria_property_to_attr_name(prop_name: &str) -> String {
        if !prop_name.starts_with("aria") || prop_name.len() <= 4 {
            return prop_name.to_ascii_lowercase();
        }
        format!("aria-{}", prop_name[4..].to_ascii_lowercase())
    }

    pub(super) fn aria_element_ref_attr_name(prop_name: &str) -> Option<&'static str> {
        match prop_name {
            "ariaActiveDescendantElement" => Some("aria-activedescendant"),
            "ariaControlsElements" => Some("aria-controls"),
            "ariaDescribedByElements" => Some("aria-describedby"),
            "ariaDetailsElements" => Some("aria-details"),
            "ariaErrorMessageElements" => Some("aria-errormessage"),
            "ariaFlowToElements" => Some("aria-flowto"),
            "ariaLabelledByElements" => Some("aria-labelledby"),
            "ariaOwnsElements" => Some("aria-owns"),
            _ => None,
        }
    }

    pub(super) fn resolve_aria_single_element_property(
        &self,
        node: NodeId,
        prop_name: &str,
    ) -> Option<NodeId> {
        let attr_name = Self::aria_element_ref_attr_name(prop_name)?;
        let raw = self.dom.attr(node, attr_name)?;
        let id_ref = raw.split_whitespace().next()?;
        self.dom.by_id(id_ref)
    }

    pub(super) fn resolve_aria_element_list_property(
        &self,
        node: NodeId,
        prop_name: &str,
    ) -> Vec<NodeId> {
        let Some(attr_name) = Self::aria_element_ref_attr_name(prop_name) else {
            return Vec::new();
        };
        let Some(raw) = self.dom.attr(node, attr_name) else {
            return Vec::new();
        };
        raw.split_whitespace()
            .filter_map(|id_ref| self.dom.by_id(id_ref))
            .collect()
    }

    pub(super) fn object_key_from_dom_prop(prop: &DomProp) -> Option<&'static str> {
        match prop {
            DomProp::Attributes => Some("attributes"),
            DomProp::AssignedSlot => Some("assignedSlot"),
            DomProp::Value => Some("value"),
            DomProp::Checked => Some("checked"),
            DomProp::Open => Some("open"),
            DomProp::ReturnValue => Some("returnValue"),
            DomProp::ClosedBy => Some("closedBy"),
            DomProp::Readonly => Some("readOnly"),
            DomProp::Required => Some("required"),
            DomProp::Disabled => Some("disabled"),
            DomProp::TextContent => Some("textContent"),
            DomProp::InnerText => Some("innerText"),
            DomProp::InnerHtml => Some("innerHTML"),
            DomProp::OuterHtml => Some("outerHTML"),
            DomProp::ClassName => Some("className"),
            DomProp::ClassList => Some("classList"),
            DomProp::Part => Some("part"),
            DomProp::Id => Some("id"),
            DomProp::TagName => Some("tagName"),
            DomProp::LocalName => Some("localName"),
            DomProp::NamespaceUri => Some("namespaceURI"),
            DomProp::Prefix => Some("prefix"),
            DomProp::NextElementSibling => Some("nextElementSibling"),
            DomProp::PreviousElementSibling => Some("previousElementSibling"),
            DomProp::Slot => Some("slot"),
            DomProp::Role => Some("role"),
            DomProp::ElementTiming => Some("elementTiming"),
            DomProp::Name => Some("name"),
            DomProp::Lang => Some("lang"),
            DomProp::ClientWidth => Some("clientWidth"),
            DomProp::ClientHeight => Some("clientHeight"),
            DomProp::ClientLeft => Some("clientLeft"),
            DomProp::ClientTop => Some("clientTop"),
            DomProp::CurrentCssZoom => Some("currentCSSZoom"),
            DomProp::OffsetWidth => Some("offsetWidth"),
            DomProp::OffsetHeight => Some("offsetHeight"),
            DomProp::OffsetLeft => Some("offsetLeft"),
            DomProp::OffsetTop => Some("offsetTop"),
            DomProp::ScrollWidth => Some("scrollWidth"),
            DomProp::ScrollHeight => Some("scrollHeight"),
            DomProp::ScrollLeft => Some("scrollLeft"),
            DomProp::ScrollTop => Some("scrollTop"),
            DomProp::ScrollLeftMax => Some("scrollLeftMax"),
            DomProp::ScrollTopMax => Some("scrollTopMax"),
            DomProp::ShadowRoot => Some("shadowRoot"),
            DomProp::Children => Some("children"),
            DomProp::ChildElementCount => Some("childElementCount"),
            DomProp::FirstElementChild => Some("firstElementChild"),
            DomProp::LastElementChild => Some("lastElementChild"),
            DomProp::Title => Some("title"),
            DomProp::AnchorAttributionSrc => Some("attributionSrc"),
            DomProp::AnchorDownload => Some("download"),
            DomProp::AnchorHash => Some("hash"),
            DomProp::AnchorHost => Some("host"),
            DomProp::AnchorHostname => Some("hostname"),
            DomProp::AnchorHref => Some("href"),
            DomProp::AnchorHreflang => Some("hreflang"),
            DomProp::AnchorInterestForElement => Some("interestForElement"),
            DomProp::AnchorOrigin => Some("origin"),
            DomProp::AnchorPassword => Some("password"),
            DomProp::AnchorPathname => Some("pathname"),
            DomProp::AnchorPing => Some("ping"),
            DomProp::AnchorPort => Some("port"),
            DomProp::AnchorProtocol => Some("protocol"),
            DomProp::AnchorReferrerPolicy => Some("referrerPolicy"),
            DomProp::AnchorRel => Some("rel"),
            DomProp::AnchorRelList => Some("relList"),
            DomProp::AnchorSearch => Some("search"),
            DomProp::AnchorTarget => Some("target"),
            DomProp::AnchorText => Some("text"),
            DomProp::AnchorType => Some("type"),
            DomProp::AnchorUsername => Some("username"),
            DomProp::AnchorCharset => Some("charset"),
            DomProp::AnchorCoords => Some("coords"),
            DomProp::AnchorRev => Some("rev"),
            DomProp::AnchorShape => Some("shape"),
            DomProp::Dataset(_)
            | DomProp::Style(_)
            | DomProp::ClassListLength
            | DomProp::PartLength
            | DomProp::AriaString(_)
            | DomProp::AriaElementRefSingle(_)
            | DomProp::AriaElementRefList(_)
            | DomProp::ValueLength
            | DomProp::ActiveElement
            | DomProp::CharacterSet
            | DomProp::CompatMode
            | DomProp::ContentType
            | DomProp::ReadyState
            | DomProp::Referrer
            | DomProp::Url
            | DomProp::DocumentUri
            | DomProp::Location
            | DomProp::LocationHref
            | DomProp::LocationProtocol
            | DomProp::LocationHost
            | DomProp::LocationHostname
            | DomProp::LocationPort
            | DomProp::LocationPathname
            | DomProp::LocationSearch
            | DomProp::LocationHash
            | DomProp::LocationOrigin
            | DomProp::LocationAncestorOrigins
            | DomProp::History
            | DomProp::HistoryLength
            | DomProp::HistoryState
            | DomProp::HistoryScrollRestoration
            | DomProp::DefaultView
            | DomProp::Hidden
            | DomProp::VisibilityState
            | DomProp::Forms
            | DomProp::Images
            | DomProp::Links
            | DomProp::Scripts
            | DomProp::CurrentScript
            | DomProp::FormsLength
            | DomProp::ImagesLength
            | DomProp::LinksLength
            | DomProp::ScriptsLength
            | DomProp::ChildrenLength
            | DomProp::AnchorRelListLength => None,
        }
    }

    pub(super) fn parse_json_text(src: &str) -> Result<Value> {
        let bytes = src.as_bytes();
        let mut i = 0usize;
        Self::json_skip_ws(bytes, &mut i);
        let value = Self::parse_json_value(src, bytes, &mut i)?;
        Self::json_skip_ws(bytes, &mut i);
        if i != bytes.len() {
            return Err(Error::ScriptRuntime(
                "JSON.parse invalid JSON: trailing characters".into(),
            ));
        }
        Ok(value)
    }

    pub(super) fn parse_json_value(src: &str, bytes: &[u8], i: &mut usize) -> Result<Value> {
        Self::json_skip_ws(bytes, i);
        let Some(&b) = bytes.get(*i) else {
            return Err(Error::ScriptRuntime(
                "JSON.parse invalid JSON: unexpected end of input".into(),
            ));
        };

        match b {
            b'{' => Self::parse_json_object(src, bytes, i),
            b'[' => Self::parse_json_array(src, bytes, i),
            b'"' => Ok(Value::String(Self::parse_json_string(src, bytes, i)?)),
            b't' => {
                if Self::json_consume_ascii(bytes, i, "true") {
                    Ok(Value::Bool(true))
                } else {
                    Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: unexpected token".into(),
                    ))
                }
            }
            b'f' => {
                if Self::json_consume_ascii(bytes, i, "false") {
                    Ok(Value::Bool(false))
                } else {
                    Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: unexpected token".into(),
                    ))
                }
            }
            b'n' => {
                if Self::json_consume_ascii(bytes, i, "null") {
                    Ok(Value::Null)
                } else {
                    Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: unexpected token".into(),
                    ))
                }
            }
            b'-' | b'0'..=b'9' => Self::parse_json_number(src, bytes, i),
            _ => Err(Error::ScriptRuntime(
                "JSON.parse invalid JSON: unexpected token".into(),
            )),
        }
    }

    pub(super) fn parse_json_object(src: &str, bytes: &[u8], i: &mut usize) -> Result<Value> {
        *i += 1; // consume '{'
        Self::json_skip_ws(bytes, i);
        let mut entries = Vec::new();

        if bytes.get(*i) == Some(&b'}') {
            *i += 1;
            return Ok(Self::new_object_value(entries));
        }

        loop {
            Self::json_skip_ws(bytes, i);
            if bytes.get(*i) != Some(&b'"') {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: object key must be string".into(),
                ));
            }
            let key = Self::parse_json_string(src, bytes, i)?;
            Self::json_skip_ws(bytes, i);
            if bytes.get(*i) != Some(&b':') {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: expected ':' after object key".into(),
                ));
            }
            *i += 1;
            let value = Self::parse_json_value(src, bytes, i)?;
            Self::object_set_entry(&mut entries, key, value);
            Self::json_skip_ws(bytes, i);

            match bytes.get(*i) {
                Some(b',') => {
                    *i += 1;
                }
                Some(b'}') => {
                    *i += 1;
                    break;
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: expected ',' or '}'".into(),
                    ));
                }
            }
        }

        Ok(Self::new_object_value(entries))
    }

    pub(super) fn parse_json_array(src: &str, bytes: &[u8], i: &mut usize) -> Result<Value> {
        *i += 1; // consume '['
        Self::json_skip_ws(bytes, i);
        let mut items = Vec::new();

        if bytes.get(*i) == Some(&b']') {
            *i += 1;
            return Ok(Self::new_array_value(items));
        }

        loop {
            let item = Self::parse_json_value(src, bytes, i)?;
            items.push(item);
            Self::json_skip_ws(bytes, i);
            match bytes.get(*i) {
                Some(b',') => {
                    *i += 1;
                }
                Some(b']') => {
                    *i += 1;
                    break;
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: expected ',' or ']'".into(),
                    ));
                }
            }
        }

        Ok(Self::new_array_value(items))
    }

    pub(super) fn parse_json_string(src: &str, bytes: &[u8], i: &mut usize) -> Result<String> {
        if bytes.get(*i) != Some(&b'"') {
            return Err(Error::ScriptRuntime(
                "JSON.parse invalid JSON: expected string".into(),
            ));
        }
        *i += 1;
        let mut out = String::new();

        while *i < bytes.len() {
            let b = bytes[*i];
            if b == b'"' {
                *i += 1;
                return Ok(out);
            }
            if b < 0x20 {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: unescaped control character in string".into(),
                ));
            }
            if b == b'\\' {
                *i += 1;
                let Some(&esc) = bytes.get(*i) else {
                    return Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: unterminated escape sequence".into(),
                    ));
                };
                match esc {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{0008}'),
                    b'f' => out.push('\u{000C}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => {
                        *i += 1;
                        let first = Self::parse_json_hex4(src, i)?;
                        if (0xD800..=0xDBFF).contains(&first) {
                            let Some(b'\\') = bytes.get(*i).copied() else {
                                return Err(Error::ScriptRuntime(
                                    "JSON.parse invalid JSON: invalid unicode surrogate pair"
                                        .into(),
                                ));
                            };
                            *i += 1;
                            let Some(b'u') = bytes.get(*i).copied() else {
                                return Err(Error::ScriptRuntime(
                                    "JSON.parse invalid JSON: invalid unicode surrogate pair"
                                        .into(),
                                ));
                            };
                            *i += 1;
                            let second = Self::parse_json_hex4(src, i)?;
                            if !(0xDC00..=0xDFFF).contains(&second) {
                                return Err(Error::ScriptRuntime(
                                    "JSON.parse invalid JSON: invalid unicode surrogate pair"
                                        .into(),
                                ));
                            }
                            let codepoint = 0x10000
                                + (((first as u32 - 0xD800) << 10) | (second as u32 - 0xDC00));
                            let Some(ch) = char::from_u32(codepoint) else {
                                return Err(Error::ScriptRuntime(
                                    "JSON.parse invalid JSON: invalid unicode escape".into(),
                                ));
                            };
                            out.push(ch);
                            continue;
                        } else if (0xDC00..=0xDFFF).contains(&first) {
                            return Err(Error::ScriptRuntime(
                                "JSON.parse invalid JSON: invalid unicode surrogate pair".into(),
                            ));
                        } else {
                            let Some(ch) = char::from_u32(first as u32) else {
                                return Err(Error::ScriptRuntime(
                                    "JSON.parse invalid JSON: invalid unicode escape".into(),
                                ));
                            };
                            out.push(ch);
                            continue;
                        }
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "JSON.parse invalid JSON: invalid escape sequence".into(),
                        ));
                    }
                }
                *i += 1;
                continue;
            }

            if b.is_ascii() {
                out.push(b as char);
                *i += 1;
            } else {
                let rest = src.get(*i..).ok_or_else(|| {
                    Error::ScriptRuntime("JSON.parse invalid JSON: invalid utf-8".into())
                })?;
                let Some(ch) = rest.chars().next() else {
                    return Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: invalid utf-8".into(),
                    ));
                };
                out.push(ch);
                *i += ch.len_utf8();
            }
        }

        Err(Error::ScriptRuntime(
            "JSON.parse invalid JSON: unterminated string".into(),
        ))
    }

    pub(super) fn parse_json_hex4(src: &str, i: &mut usize) -> Result<u16> {
        let end = i.saturating_add(4);
        let segment = src.get(*i..end).ok_or_else(|| {
            Error::ScriptRuntime("JSON.parse invalid JSON: invalid unicode escape".into())
        })?;
        if !segment.as_bytes().iter().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::ScriptRuntime(
                "JSON.parse invalid JSON: invalid unicode escape".into(),
            ));
        }
        *i = end;
        u16::from_str_radix(segment, 16).map_err(|_| {
            Error::ScriptRuntime("JSON.parse invalid JSON: invalid unicode escape".into())
        })
    }

    pub(super) fn parse_json_number(src: &str, bytes: &[u8], i: &mut usize) -> Result<Value> {
        let start = *i;

        if bytes.get(*i) == Some(&b'-') {
            *i += 1;
        }

        match bytes.get(*i).copied() {
            Some(b'0') => {
                *i += 1;
                if bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                    return Err(Error::ScriptRuntime(
                        "JSON.parse invalid JSON: invalid number".into(),
                    ));
                }
            }
            Some(b'1'..=b'9') => {
                *i += 1;
                while bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                    *i += 1;
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: invalid number".into(),
                ));
            }
        }

        if bytes.get(*i) == Some(&b'.') {
            *i += 1;
            if !bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: invalid number".into(),
                ));
            }
            while bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                *i += 1;
            }
        }

        if bytes.get(*i).is_some_and(|b| *b == b'e' || *b == b'E') {
            *i += 1;
            if bytes.get(*i).is_some_and(|b| *b == b'+' || *b == b'-') {
                *i += 1;
            }
            if !bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                return Err(Error::ScriptRuntime(
                    "JSON.parse invalid JSON: invalid number".into(),
                ));
            }
            while bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                *i += 1;
            }
        }

        let token = src.get(start..*i).ok_or_else(|| {
            Error::ScriptRuntime("JSON.parse invalid JSON: invalid number".into())
        })?;
        if !token.contains('.') && !token.contains('e') && !token.contains('E') {
            if let Ok(n) = token.parse::<i64>() {
                return Ok(Value::Number(n));
            }
        }
        let n = token
            .parse::<f64>()
            .map_err(|_| Error::ScriptRuntime("JSON.parse invalid JSON: invalid number".into()))?;
        Ok(Value::Float(n))
    }

    pub(super) fn json_skip_ws(bytes: &[u8], i: &mut usize) {
        while bytes.get(*i).is_some_and(|b| b.is_ascii_whitespace()) {
            *i += 1;
        }
    }

    pub(super) fn json_consume_ascii(bytes: &[u8], i: &mut usize, token: &str) -> bool {
        let token_bytes = token.as_bytes();
        let end = i.saturating_add(token_bytes.len());
        if end <= bytes.len() && &bytes[*i..end] == token_bytes {
            *i = end;
            true
        } else {
            false
        }
    }

    pub(super) fn json_stringify_top_level(value: &Value) -> Result<Option<String>> {
        let mut array_stack = Vec::new();
        let mut object_stack = Vec::new();
        Self::json_stringify_value(value, &mut array_stack, &mut object_stack)
    }

    pub(super) fn json_stringify_value(
        value: &Value,
        array_stack: &mut Vec<usize>,
        object_stack: &mut Vec<usize>,
    ) -> Result<Option<String>> {
        match value {
            Value::String(v) => Ok(Some(format!("\"{}\"", Self::json_escape_string(v)))),
            Value::Bool(v) => Ok(Some(if *v { "true".into() } else { "false".into() })),
            Value::Number(v) => Ok(Some(v.to_string())),
            Value::Float(v) => {
                if v.is_finite() {
                    Ok(Some(format_float(*v)))
                } else {
                    Ok(Some("null".into()))
                }
            }
            Value::BigInt(_) => Err(Error::ScriptRuntime(
                "JSON.stringify does not support BigInt values".into(),
            )),
            Value::Null => Ok(Some("null".into())),
            Value::Undefined
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::Symbol(_)
            | Value::PromiseCapability(_)
            | Value::Function(_) => Ok(None),
            Value::RegExp(_) => Ok(Some("{}".to_string())),
            Value::Date(v) => Ok(Some(format!(
                "\"{}\"",
                Self::json_escape_string(&Self::format_iso_8601_utc(*v.borrow()))
            ))),
            Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_) => Ok(Some("{}".to_string())),
            Value::Array(values) => {
                let ptr = Rc::as_ptr(values) as usize;
                if array_stack.contains(&ptr) {
                    return Err(Error::ScriptRuntime(
                        "JSON.stringify circular structure".into(),
                    ));
                }
                array_stack.push(ptr);

                let items = values.borrow();
                let mut out = String::from("[");
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        out.push(',');
                    }
                    let serialized = Self::json_stringify_value(item, array_stack, object_stack)?
                        .unwrap_or_else(|| "null".to_string());
                    out.push_str(&serialized);
                }
                out.push(']');

                array_stack.pop();
                Ok(Some(out))
            }
            Value::Object(entries) => {
                let ptr = Rc::as_ptr(entries) as usize;
                if object_stack.contains(&ptr) {
                    return Err(Error::ScriptRuntime(
                        "JSON.stringify circular structure".into(),
                    ));
                }
                object_stack.push(ptr);

                let entries = entries.borrow();
                let mut out = String::from("{");
                let mut wrote = false;
                for (key, value) in entries.iter() {
                    if Self::is_internal_object_key(key) {
                        continue;
                    }
                    let Some(serialized) =
                        Self::json_stringify_value(value, array_stack, object_stack)?
                    else {
                        continue;
                    };
                    if wrote {
                        out.push(',');
                    }
                    wrote = true;
                    out.push('"');
                    out.push_str(&Self::json_escape_string(key));
                    out.push_str("\":");
                    out.push_str(&serialized);
                }
                out.push('}');

                object_stack.pop();
                Ok(Some(out))
            }
        }
    }

    pub(super) fn json_escape_string(src: &str) -> String {
        let mut out = String::new();
        for ch in src.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\u{0008}' => out.push_str("\\b"),
                '\u{000C}' => out.push_str("\\f"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c <= '\u{001F}' => {
                    out.push_str(&format!("\\u{:04X}", c as u32));
                }
                c => out.push(c),
            }
        }
        out
    }

    pub(super) fn structured_clone_value(
        value: &Value,
        array_stack: &mut Vec<usize>,
        object_stack: &mut Vec<usize>,
    ) -> Result<Value> {
        match value {
            Value::String(v) => Ok(Value::String(v.clone())),
            Value::Bool(v) => Ok(Value::Bool(*v)),
            Value::Number(v) => Ok(Value::Number(*v)),
            Value::Float(v) => Ok(Value::Float(*v)),
            Value::BigInt(v) => Ok(Value::BigInt(v.clone())),
            Value::Null => Ok(Value::Null),
            Value::Undefined => Ok(Value::Undefined),
            Value::Date(v) => Ok(Value::Date(Rc::new(RefCell::new(*v.borrow())))),
            Value::RegExp(v) => {
                let v = v.borrow();
                let cloned = Self::new_regex_value(v.source.clone(), v.flags.clone())?;
                let Value::RegExp(cloned_regex) = &cloned else {
                    unreachable!("RegExp clone must produce RegExp value");
                };
                {
                    let mut cloned_regex = cloned_regex.borrow_mut();
                    cloned_regex.last_index = v.last_index;
                    cloned_regex.properties = v.properties.clone();
                }
                Ok(cloned)
            }
            Value::ArrayBuffer(buffer) => {
                let buffer = buffer.borrow();
                Ok(Value::ArrayBuffer(Rc::new(RefCell::new(
                    ArrayBufferValue {
                        bytes: buffer.bytes.clone(),
                        max_byte_length: buffer.max_byte_length,
                        detached: buffer.detached,
                    },
                ))))
            }
            Value::TypedArray(array) => {
                let array = array.borrow();
                let buffer = array.buffer.borrow();
                let cloned_buffer = Rc::new(RefCell::new(ArrayBufferValue {
                    bytes: buffer.bytes.clone(),
                    max_byte_length: buffer.max_byte_length,
                    detached: buffer.detached,
                }));
                Ok(Value::TypedArray(Rc::new(RefCell::new(TypedArrayValue {
                    kind: array.kind,
                    buffer: cloned_buffer,
                    byte_offset: array.byte_offset,
                    fixed_length: array.fixed_length,
                }))))
            }
            Value::Blob(blob) => {
                let blob = blob.borrow();
                Ok(Self::new_blob_value(
                    blob.bytes.clone(),
                    blob.mime_type.clone(),
                ))
            }
            Value::Map(map) => {
                let map = map.borrow();
                Ok(Value::Map(Rc::new(RefCell::new(MapValue {
                    entries: map.entries.clone(),
                    properties: map.properties.clone(),
                }))))
            }
            Value::Set(set) => {
                let set = set.borrow();
                Ok(Value::Set(Rc::new(RefCell::new(SetValue {
                    values: set.values.clone(),
                    properties: set.properties.clone(),
                }))))
            }
            Value::Array(values) => {
                let ptr = Rc::as_ptr(values) as usize;
                if array_stack.contains(&ptr) {
                    return Err(Error::ScriptRuntime(
                        "structuredClone does not support circular values".into(),
                    ));
                }
                array_stack.push(ptr);

                let items = values.borrow();
                let mut cloned = Vec::with_capacity(items.len());
                for item in items.iter() {
                    cloned.push(Self::structured_clone_value(
                        item,
                        array_stack,
                        object_stack,
                    )?);
                }
                array_stack.pop();

                Ok(Self::new_array_value(cloned))
            }
            Value::Object(entries) => {
                let ptr = Rc::as_ptr(entries) as usize;
                if object_stack.contains(&ptr) {
                    return Err(Error::ScriptRuntime(
                        "structuredClone does not support circular values".into(),
                    ));
                }
                object_stack.push(ptr);

                let entries = entries.borrow();
                let mut cloned = Vec::with_capacity(entries.len());
                for (key, value) in entries.iter() {
                    let value = Self::structured_clone_value(value, array_stack, object_stack)?;
                    cloned.push((key.clone(), value));
                }
                object_stack.pop();

                Ok(Self::new_object_value(cloned))
            }
            Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Promise(_)
            | Value::Symbol(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Function(_) => Err(Error::ScriptRuntime(
                "structuredClone value is not cloneable".into(),
            )),
        }
    }

    pub(super) fn analyze_regex_flags(flags: &str) -> std::result::Result<RegexFlags, String> {
        let mut info = RegexFlags {
            global: false,
            ignore_case: false,
            multiline: false,
            dot_all: false,
            sticky: false,
            has_indices: false,
            unicode: false,
        };
        let mut seen = HashSet::new();
        for ch in flags.chars() {
            if !seen.insert(ch) {
                return Err(format!("invalid regular expression flags: {flags}"));
            }
            match ch {
                'g' => info.global = true,
                'i' => info.ignore_case = true,
                'm' => info.multiline = true,
                's' => info.dot_all = true,
                'y' => info.sticky = true,
                'd' => info.has_indices = true,
                'u' => info.unicode = true,
                'v' => {
                    return Err("invalid regular expression flags: v flag is not supported".into());
                }
                _ => return Err(format!("invalid regular expression flags: {flags}")),
            }
        }
        Ok(info)
    }

    pub(super) fn compile_regex(
        pattern: &str,
        info: RegexFlags,
    ) -> std::result::Result<Regex, regex::Error> {
        let mut builder = RegexBuilder::new(pattern);
        builder.case_insensitive(info.ignore_case);
        builder.multi_line(info.multiline);
        builder.dot_matches_new_line(info.dot_all);
        builder.build()
    }

    pub(super) fn new_regex_value(pattern: String, flags: String) -> Result<Value> {
        let info = Self::analyze_regex_flags(&flags).map_err(Error::ScriptRuntime)?;
        let compiled = Self::compile_regex(&pattern, info).map_err(|err| {
            Error::ScriptRuntime(format!(
                "invalid regular expression: /{pattern}/{flags}: {err}"
            ))
        })?;
        Ok(Value::RegExp(Rc::new(RefCell::new(RegexValue {
            source: pattern,
            flags,
            global: info.global,
            ignore_case: info.ignore_case,
            multiline: info.multiline,
            dot_all: info.dot_all,
            sticky: info.sticky,
            has_indices: info.has_indices,
            unicode: info.unicode,
            compiled,
            last_index: 0,
            properties: Vec::new(),
        }))))
    }

    pub(super) fn new_regex_from_values(pattern: &Value, flags: Option<&Value>) -> Result<Value> {
        let pattern_text = match pattern {
            Value::RegExp(value) => value.borrow().source.clone(),
            _ => pattern.as_string(),
        };
        let flags_text = if let Some(flags) = flags {
            flags.as_string()
        } else if let Value::RegExp(value) = pattern {
            value.borrow().flags.clone()
        } else {
            String::new()
        };
        Self::new_regex_value(pattern_text, flags_text)
    }

    pub(super) fn resolve_regex_from_value(value: &Value) -> Result<Rc<RefCell<RegexValue>>> {
        match value {
            Value::RegExp(regex) => Ok(regex.clone()),
            _ => Err(Error::ScriptRuntime("value is not a RegExp".into())),
        }
    }

    pub(super) fn regex_test(regex: &Rc<RefCell<RegexValue>>, input: &str) -> Result<bool> {
        Ok(Self::regex_exec_internal(regex, input)?.is_some())
    }

    pub(super) fn regex_exec(
        regex: &Rc<RefCell<RegexValue>>,
        input: &str,
    ) -> Result<Option<Vec<String>>> {
        Self::regex_exec_internal(regex, input)
    }

    pub(super) fn regex_exec_internal(
        regex: &Rc<RefCell<RegexValue>>,
        input: &str,
    ) -> Result<Option<Vec<String>>> {
        let mut regex = regex.borrow_mut();
        let start = if regex.global || regex.sticky {
            regex.last_index
        } else {
            0
        };
        if start > input.len() {
            regex.last_index = 0;
            return Ok(None);
        }

        let captures = regex.compiled.captures_at(input, start);

        let Some(captures) = captures else {
            if regex.global || regex.sticky {
                regex.last_index = 0;
            }
            return Ok(None);
        };

        let Some(full_match) = captures.get(0) else {
            if regex.global || regex.sticky {
                regex.last_index = 0;
            }
            return Ok(None);
        };

        if regex.sticky && full_match.start() != start {
            regex.last_index = 0;
            return Ok(None);
        }

        if regex.global || regex.sticky {
            regex.last_index = full_match.end();
        }

        let mut out = Vec::with_capacity(captures.len());
        for idx in 0..captures.len() {
            out.push(
                captures
                    .get(idx)
                    .map(|capture| capture.as_str().to_string())
                    .unwrap_or_default(),
            );
        }
        Ok(Some(out))
    }

    pub(super) fn eval_math_method(
        &mut self,
        method: MathMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }

        let single = |values: &[Value]| Self::coerce_number_for_global(&values[0]);

        match method {
            MathMethod::Abs => Ok(Value::Float(single(&values).abs())),
            MathMethod::Acos => Ok(Value::Float(single(&values).acos())),
            MathMethod::Acosh => Ok(Value::Float(single(&values).acosh())),
            MathMethod::Asin => Ok(Value::Float(single(&values).asin())),
            MathMethod::Asinh => Ok(Value::Float(single(&values).asinh())),
            MathMethod::Atan => Ok(Value::Float(single(&values).atan())),
            MathMethod::Atan2 => Ok(Value::Float(
                Self::coerce_number_for_global(&values[0])
                    .atan2(Self::coerce_number_for_global(&values[1])),
            )),
            MathMethod::Atanh => Ok(Value::Float(single(&values).atanh())),
            MathMethod::Cbrt => Ok(Value::Float(single(&values).cbrt())),
            MathMethod::Ceil => Ok(Value::Float(single(&values).ceil())),
            MathMethod::Clz32 => Ok(Value::Number(i64::from(
                Self::to_u32_for_math(&values[0]).leading_zeros(),
            ))),
            MathMethod::Cos => Ok(Value::Float(single(&values).cos())),
            MathMethod::Cosh => Ok(Value::Float(single(&values).cosh())),
            MathMethod::Exp => Ok(Value::Float(single(&values).exp())),
            MathMethod::Expm1 => Ok(Value::Float(single(&values).exp_m1())),
            MathMethod::Floor => Ok(Value::Float(single(&values).floor())),
            MathMethod::F16Round => Ok(Value::Float(Self::math_f16round(single(&values)))),
            MathMethod::FRound => Ok(Value::Float((single(&values) as f32) as f64)),
            MathMethod::Hypot => {
                let mut sum = 0.0f64;
                for value in values {
                    let value = Self::coerce_number_for_global(&value);
                    sum += value * value;
                }
                Ok(Value::Float(sum.sqrt()))
            }
            MathMethod::Imul => {
                let left = Self::to_i32_for_math(&values[0]);
                let right = Self::to_i32_for_math(&values[1]);
                Ok(Value::Number(i64::from(left.wrapping_mul(right))))
            }
            MathMethod::Log => Ok(Value::Float(single(&values).ln())),
            MathMethod::Log10 => Ok(Value::Float(single(&values).log10())),
            MathMethod::Log1p => Ok(Value::Float(single(&values).ln_1p())),
            MathMethod::Log2 => Ok(Value::Float(single(&values).log2())),
            MathMethod::Max => {
                let mut out = f64::NEG_INFINITY;
                for value in values {
                    out = out.max(Self::coerce_number_for_global(&value));
                }
                Ok(Value::Float(out))
            }
            MathMethod::Min => {
                let mut out = f64::INFINITY;
                for value in values {
                    out = out.min(Self::coerce_number_for_global(&value));
                }
                Ok(Value::Float(out))
            }
            MathMethod::Pow => Ok(Value::Float(
                Self::coerce_number_for_global(&values[0])
                    .powf(Self::coerce_number_for_global(&values[1])),
            )),
            MathMethod::Random => Ok(Value::Float(self.next_random_f64())),
            MathMethod::Round => Ok(Value::Float(Self::js_math_round(single(&values)))),
            MathMethod::Sign => Ok(Value::Float(Self::js_math_sign(single(&values)))),
            MathMethod::Sin => Ok(Value::Float(single(&values).sin())),
            MathMethod::Sinh => Ok(Value::Float(single(&values).sinh())),
            MathMethod::Sqrt => Ok(Value::Float(single(&values).sqrt())),
            MathMethod::SumPrecise => match &values[0] {
                Value::Array(values) => Ok(Value::Float(Self::sum_precise(&values.borrow()))),
                _ => Err(Error::ScriptRuntime(
                    "Math.sumPrecise argument must be an array".into(),
                )),
            },
            MathMethod::Tan => Ok(Value::Float(single(&values).tan())),
            MathMethod::Tanh => Ok(Value::Float(single(&values).tanh())),
            MathMethod::Trunc => Ok(Value::Float(single(&values).trunc())),
        }
    }

    pub(super) fn eval_number_method(
        &mut self,
        method: NumberMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }

        match method {
            NumberMethod::IsFinite => Ok(Value::Bool(
                Self::number_primitive_value(&values[0]).is_some_and(f64::is_finite),
            )),
            NumberMethod::IsInteger => Ok(Value::Bool(
                Self::number_primitive_value(&values[0])
                    .is_some_and(|value| value.is_finite() && value.fract() == 0.0),
            )),
            NumberMethod::IsNaN => Ok(Value::Bool(matches!(
                values[0],
                Value::Float(value) if value.is_nan()
            ))),
            NumberMethod::IsSafeInteger => Ok(Value::Bool(
                Self::number_primitive_value(&values[0]).is_some_and(|value| {
                    value.is_finite()
                        && value.fract() == 0.0
                        && value.abs() <= 9_007_199_254_740_991.0
                }),
            )),
            NumberMethod::ParseFloat => {
                Ok(Value::Float(parse_js_parse_float(&values[0].as_string())))
            }
            NumberMethod::ParseInt => {
                let radix = if values.len() == 2 {
                    Some(Self::value_to_i64(&values[1]))
                } else {
                    None
                };
                Ok(Value::Float(parse_js_parse_int(
                    &values[0].as_string(),
                    radix,
                )))
            }
        }
    }

    pub(super) fn eval_number_instance_method(
        &mut self,
        method: NumberInstanceMethod,
        value: &Expr,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let value = self.eval_expr(value, env, event_param, event)?;
        let mut args_value = Vec::with_capacity(args.len());
        for arg in args {
            args_value.push(self.eval_expr(arg, env, event_param, event)?);
        }

        if let Value::BigInt(bigint) = &value {
            return match method {
                NumberInstanceMethod::ToLocaleString => Ok(Value::String(bigint.to_string())),
                NumberInstanceMethod::ToString => {
                    let radix = if let Some(arg) = args_value.first() {
                        let radix = Self::value_to_i64(arg);
                        if !(2..=36).contains(&radix) {
                            return Err(Error::ScriptRuntime(
                                "toString radix must be between 2 and 36".into(),
                            ));
                        }
                        radix as u32
                    } else {
                        10
                    };
                    Ok(Value::String(bigint.to_str_radix(radix)))
                }
                NumberInstanceMethod::ValueOf => Ok(Value::BigInt(bigint.clone())),
                NumberInstanceMethod::ToExponential
                | NumberInstanceMethod::ToFixed
                | NumberInstanceMethod::ToPrecision => Err(Error::ScriptRuntime(
                    "number formatting methods are not supported for BigInt values".into(),
                )),
            };
        }

        if let Value::Symbol(symbol) = &value {
            return match method {
                NumberInstanceMethod::ValueOf => Ok(Value::Symbol(symbol.clone())),
                NumberInstanceMethod::ToString | NumberInstanceMethod::ToLocaleString => {
                    if !args_value.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "Symbol.toString does not take arguments".into(),
                        ));
                    }
                    Ok(Value::String(Value::Symbol(symbol.clone()).as_string()))
                }
                NumberInstanceMethod::ToExponential
                | NumberInstanceMethod::ToFixed
                | NumberInstanceMethod::ToPrecision => Err(Error::ScriptRuntime(
                    "Cannot convert a Symbol value to a number".into(),
                )),
            };
        }

        let numeric = Self::coerce_number_for_number_constructor(&value);

        match method {
            NumberInstanceMethod::ToExponential => {
                let fraction_digits = if let Some(arg) = args_value.first() {
                    let fraction_digits = Self::value_to_i64(arg);
                    if !(0..=100).contains(&fraction_digits) {
                        return Err(Error::ScriptRuntime(
                            "toExponential fractionDigits must be between 0 and 100".into(),
                        ));
                    }
                    Some(fraction_digits as usize)
                } else {
                    None
                };
                Ok(Value::String(Self::number_to_exponential(
                    numeric,
                    fraction_digits,
                )))
            }
            NumberInstanceMethod::ToFixed => {
                let fraction_digits = if let Some(arg) = args_value.first() {
                    let fraction_digits = Self::value_to_i64(arg);
                    if !(0..=100).contains(&fraction_digits) {
                        return Err(Error::ScriptRuntime(
                            "toFixed fractionDigits must be between 0 and 100".into(),
                        ));
                    }
                    fraction_digits as usize
                } else {
                    0
                };
                Ok(Value::String(Self::number_to_fixed(
                    numeric,
                    fraction_digits,
                )))
            }
            NumberInstanceMethod::ToLocaleString => {
                Ok(Value::String(Self::format_number_default(numeric)))
            }
            NumberInstanceMethod::ToPrecision => {
                if let Some(arg) = args_value.first() {
                    let precision = Self::value_to_i64(arg);
                    if !(1..=100).contains(&precision) {
                        return Err(Error::ScriptRuntime(
                            "toPrecision precision must be between 1 and 100".into(),
                        ));
                    }
                    Ok(Value::String(Self::number_to_precision(
                        numeric,
                        precision as usize,
                    )))
                } else {
                    Ok(Value::String(Self::format_number_default(numeric)))
                }
            }
            NumberInstanceMethod::ToString => {
                let radix = if let Some(arg) = args_value.first() {
                    let radix = Self::value_to_i64(arg);
                    if !(2..=36).contains(&radix) {
                        return Err(Error::ScriptRuntime(
                            "toString radix must be between 2 and 36".into(),
                        ));
                    }
                    radix as u32
                } else {
                    10
                };
                Ok(Value::String(Self::number_to_string_radix(numeric, radix)))
            }
            NumberInstanceMethod::ValueOf => Ok(Self::number_value(numeric)),
        }
    }

    pub(super) fn eval_bigint_method(
        &mut self,
        method: BigIntMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }
        let bits_i64 = Self::value_to_i64(&values[0]);
        if bits_i64 < 0 {
            return Err(Error::ScriptRuntime(
                "BigInt bit width must be a non-negative integer".into(),
            ));
        }
        let bits = usize::try_from(bits_i64)
            .map_err(|_| Error::ScriptRuntime("BigInt bit width is too large".into()))?;
        let value = Self::coerce_bigint_for_builtin_op(&values[1])?;
        let out = match method {
            BigIntMethod::AsIntN => Self::bigint_as_int_n(bits, &value),
            BigIntMethod::AsUintN => Self::bigint_as_uint_n(bits, &value),
        };
        Ok(Value::BigInt(out))
    }

    pub(super) fn eval_bigint_instance_method(
        &mut self,
        method: BigIntInstanceMethod,
        value: &Expr,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let value = self.eval_expr(value, env, event_param, event)?;
        let value = Self::coerce_bigint_for_builtin_op(&value)?;
        let mut args_value = Vec::with_capacity(args.len());
        for arg in args {
            args_value.push(self.eval_expr(arg, env, event_param, event)?);
        }

        match method {
            BigIntInstanceMethod::ToLocaleString => Ok(Value::String(value.to_string())),
            BigIntInstanceMethod::ToString => {
                let radix = if let Some(arg) = args_value.first() {
                    let radix = Self::value_to_i64(arg);
                    if !(2..=36).contains(&radix) {
                        return Err(Error::ScriptRuntime(
                            "toString radix must be between 2 and 36".into(),
                        ));
                    }
                    radix as u32
                } else {
                    10
                };
                Ok(Value::String(value.to_str_radix(radix)))
            }
            BigIntInstanceMethod::ValueOf => Ok(Value::BigInt(value)),
        }
    }

    pub(super) fn bigint_as_uint_n(bits: usize, value: &JsBigInt) -> JsBigInt {
        if bits == 0 {
            return JsBigInt::zero();
        }
        let modulo = JsBigInt::one() << bits;
        let mut out = value % &modulo;
        if out.sign() == Sign::Minus {
            out += &modulo;
        }
        out
    }

    pub(super) fn bigint_as_int_n(bits: usize, value: &JsBigInt) -> JsBigInt {
        if bits == 0 {
            return JsBigInt::zero();
        }
        let modulo = JsBigInt::one() << bits;
        let threshold = JsBigInt::one() << (bits - 1);
        let unsigned = Self::bigint_as_uint_n(bits, value);
        if unsigned >= threshold {
            unsigned - modulo
        } else {
            unsigned
        }
    }

    pub(super) fn coerce_bigint_for_constructor(value: &Value) -> Result<JsBigInt> {
        match value {
            Value::BigInt(value) => Ok(value.clone()),
            Value::Bool(value) => Ok(if *value {
                JsBigInt::one()
            } else {
                JsBigInt::zero()
            }),
            Value::Number(value) => Ok(JsBigInt::from(*value)),
            Value::Float(value) => {
                if value.is_finite() && value.fract() == 0.0 {
                    Ok(JsBigInt::from(*value as i64))
                } else {
                    Err(Error::ScriptRuntime(
                        "cannot convert Number value to BigInt".into(),
                    ))
                }
            }
            Value::String(value) => Self::parse_js_bigint_from_string(value),
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(
                "cannot convert null or undefined to BigInt".into(),
            )),
            Value::Date(value) => Ok(JsBigInt::from(*value.borrow())),
            Value::Array(values) => {
                let rendered = Value::Array(values.clone()).as_string();
                Self::parse_js_bigint_from_string(&rendered)
            }
            Value::Object(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Symbol(_)
            | Value::RegExp(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_) => Err(Error::ScriptRuntime(
                "cannot convert object value to BigInt".into(),
            )),
        }
    }

    pub(super) fn coerce_bigint_for_builtin_op(value: &Value) -> Result<JsBigInt> {
        match value {
            Value::BigInt(value) => Ok(value.clone()),
            Value::Bool(value) => Ok(if *value {
                JsBigInt::one()
            } else {
                JsBigInt::zero()
            }),
            Value::String(value) => Self::parse_js_bigint_from_string(value),
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(
                "cannot convert null or undefined to BigInt".into(),
            )),
            Value::Number(_)
            | Value::Float(_)
            | Value::Date(_)
            | Value::Object(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Symbol(_)
            | Value::RegExp(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_)
            | Value::Array(_) => Err(Error::ScriptRuntime(
                "cannot convert value to BigInt".into(),
            )),
        }
    }

    pub(super) fn parse_js_bigint_from_string(src: &str) -> Result<JsBigInt> {
        let trimmed = src.trim();
        if trimmed.is_empty() {
            return Ok(JsBigInt::zero());
        }

        if let Some(rest) = trimmed.strip_prefix('+') {
            return Self::parse_signed_decimal_bigint(rest, false);
        }
        if let Some(rest) = trimmed.strip_prefix('-') {
            return Self::parse_signed_decimal_bigint(rest, true);
        }

        if let Some(rest) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            return Self::parse_prefixed_bigint(rest, 16, trimmed);
        }
        if let Some(rest) = trimmed
            .strip_prefix("0o")
            .or_else(|| trimmed.strip_prefix("0O"))
        {
            return Self::parse_prefixed_bigint(rest, 8, trimmed);
        }
        if let Some(rest) = trimmed
            .strip_prefix("0b")
            .or_else(|| trimmed.strip_prefix("0B"))
        {
            return Self::parse_prefixed_bigint(rest, 2, trimmed);
        }

        Self::parse_signed_decimal_bigint(trimmed, false)
    }

    pub(super) fn parse_prefixed_bigint(src: &str, radix: u32, original: &str) -> Result<JsBigInt> {
        if src.is_empty() {
            return Err(Error::ScriptRuntime(format!(
                "cannot convert {} to a BigInt",
                original
            )));
        }
        JsBigInt::parse_bytes(src.as_bytes(), radix)
            .ok_or_else(|| Error::ScriptRuntime(format!("cannot convert {} to a BigInt", original)))
    }

    pub(super) fn parse_signed_decimal_bigint(src: &str, negative: bool) -> Result<JsBigInt> {
        let original = format!("{}{}", if negative { "-" } else { "" }, src);
        if src.is_empty() || !src.as_bytes().iter().all(u8::is_ascii_digit) {
            return Err(Error::ScriptRuntime(format!(
                "cannot convert {} to a BigInt",
                original
            )));
        }
        let mut value = JsBigInt::parse_bytes(src.as_bytes(), 10).ok_or_else(|| {
            Error::ScriptRuntime(format!("cannot convert {} to a BigInt", original))
        })?;
        if negative {
            value = -value;
        }
        Ok(value)
    }

    pub(super) fn bigint_shift_left(value: &JsBigInt, shift: &JsBigInt) -> Result<JsBigInt> {
        if shift.sign() == Sign::Minus {
            let magnitude = (-shift)
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value >> magnitude)
        } else {
            let magnitude = shift
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value << magnitude)
        }
    }

    pub(super) fn bigint_shift_right(value: &JsBigInt, shift: &JsBigInt) -> Result<JsBigInt> {
        if shift.sign() == Sign::Minus {
            let magnitude = (-shift)
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value << magnitude)
        } else {
            let magnitude = shift
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value >> magnitude)
        }
    }

    pub(super) fn number_primitive_value(value: &Value) -> Option<f64> {
        match value {
            Value::Number(value) => Some(*value as f64),
            Value::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub(super) fn number_value(value: f64) -> Value {
        if value == 0.0 && value.is_sign_negative() {
            return Value::Float(-0.0);
        }
        if value.is_finite()
            && value.fract() == 0.0
            && value >= i64::MIN as f64
            && value <= i64::MAX as f64
        {
            let integer = value as i64;
            if (integer as f64) == value {
                return Value::Number(integer);
            }
        }
        Value::Float(value)
    }

    pub(super) fn coerce_number_for_number_constructor(value: &Value) -> f64 {
        match value {
            Value::Number(v) => *v as f64,
            Value::Float(v) => *v,
            Value::BigInt(v) => v.to_f64().unwrap_or_else(|| {
                if v.sign() == Sign::Minus {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                }
            }),
            Value::Bool(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            Value::String(v) => Self::parse_js_number_from_string(v),
            Value::Date(v) => *v.borrow() as f64,
            Value::Object(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Symbol(_)
            | Value::RegExp(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_) => f64::NAN,
            Value::Array(values) => {
                let rendered = Value::Array(values.clone()).as_string();
                Self::parse_js_number_from_string(&rendered)
            }
        }
    }

    pub(super) fn parse_js_number_from_string(src: &str) -> f64 {
        let trimmed = src.trim();
        if trimmed.is_empty() {
            return 0.0;
        }
        if trimmed == "Infinity" || trimmed == "+Infinity" {
            return f64::INFINITY;
        }
        if trimmed == "-Infinity" {
            return f64::NEG_INFINITY;
        }

        if trimmed.starts_with('+') || trimmed.starts_with('-') {
            let rest = &trimmed[1..];
            if rest.starts_with("0x")
                || rest.starts_with("0X")
                || rest.starts_with("0o")
                || rest.starts_with("0O")
                || rest.starts_with("0b")
                || rest.starts_with("0B")
            {
                return f64::NAN;
            }
        }

        if let Some(digits) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            return Self::parse_prefixed_radix_to_f64(digits, 16);
        }
        if let Some(digits) = trimmed
            .strip_prefix("0o")
            .or_else(|| trimmed.strip_prefix("0O"))
        {
            return Self::parse_prefixed_radix_to_f64(digits, 8);
        }
        if let Some(digits) = trimmed
            .strip_prefix("0b")
            .or_else(|| trimmed.strip_prefix("0B"))
        {
            return Self::parse_prefixed_radix_to_f64(digits, 2);
        }

        trimmed.parse::<f64>().unwrap_or(f64::NAN)
    }

    pub(super) fn parse_prefixed_radix_to_f64(src: &str, radix: u32) -> f64 {
        if src.is_empty() {
            return f64::NAN;
        }
        let mut out = 0.0f64;
        for ch in src.chars() {
            let Some(digit) = ch.to_digit(radix) else {
                return f64::NAN;
            };
            out = out * (radix as f64) + (digit as f64);
        }
        out
    }

    pub(super) fn format_number_default(value: f64) -> String {
        if value.is_nan() {
            return "NaN".to_string();
        }
        if value == f64::INFINITY {
            return "Infinity".to_string();
        }
        if value == f64::NEG_INFINITY {
            return "-Infinity".to_string();
        }
        if value == 0.0 {
            return "0".to_string();
        }

        if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            let integer = value as i64;
            if (integer as f64) == value {
                return integer.to_string();
            }
        }

        let out = format!("{value}");
        Self::normalize_exponential_string(out, false)
    }

    pub(super) fn number_to_exponential(value: f64, fraction_digits: Option<usize>) -> String {
        if !value.is_finite() {
            return Self::format_number_default(value);
        }

        let out = if let Some(fraction_digits) = fraction_digits {
            format!(
                "{:.*e}",
                fraction_digits,
                if value == 0.0 { 0.0 } else { value }
            )
        } else {
            format!("{:e}", if value == 0.0 { 0.0 } else { value })
        };
        Self::normalize_exponential_string(out, fraction_digits.is_none())
    }

    pub(super) fn normalize_exponential_string(raw: String, trim_fraction_zeros: bool) -> String {
        let Some(exp_idx) = raw.find('e').or_else(|| raw.find('E')) else {
            return raw;
        };
        let mut mantissa = raw[..exp_idx].to_string();
        let exponent_src = &raw[exp_idx + 1..];

        if trim_fraction_zeros && mantissa.contains('.') {
            while mantissa.ends_with('0') {
                mantissa.pop();
            }
            if mantissa.ends_with('.') {
                mantissa.pop();
            }
        }

        let exponent = exponent_src.parse::<i32>().unwrap_or(0);
        format!("{mantissa}e{:+}", exponent)
    }

    pub(super) fn number_to_fixed(value: f64, fraction_digits: usize) -> String {
        if !value.is_finite() {
            return Self::format_number_default(value);
        }
        format!(
            "{:.*}",
            fraction_digits,
            if value == 0.0 { 0.0 } else { value }
        )
    }

    pub(super) fn number_to_precision(value: f64, precision: usize) -> String {
        if !value.is_finite() {
            return Self::format_number_default(value);
        }
        if value == 0.0 {
            if precision == 1 {
                return "0".to_string();
            }
            return format!("0.{}", "0".repeat(precision - 1));
        }

        let abs = value.abs();
        let exponent = abs.log10().floor() as i32;
        if exponent < -6 || exponent >= precision as i32 {
            return Self::number_to_exponential(value, Some(precision.saturating_sub(1)));
        }

        let fraction_digits = (precision as i32 - exponent - 1).max(0) as usize;
        format!(
            "{:.*}",
            fraction_digits,
            if value == 0.0 { 0.0 } else { value }
        )
    }

    pub(super) fn number_to_string_radix(value: f64, radix: u32) -> String {
        if radix == 10 {
            return Self::format_number_default(value);
        }
        if !value.is_finite() {
            return Self::format_number_default(value);
        }
        if value == 0.0 {
            return "0".to_string();
        }

        let sign = if value < 0.0 { "-" } else { "" };
        let abs = value.abs();
        let int_part = abs.trunc();
        let mut int_digits = Vec::new();
        let mut n = int_part;
        let radix_f64 = radix as f64;

        while n >= 1.0 {
            let digit = (n % radix_f64).floor() as u32;
            int_digits.push(Self::radix_digit_char(digit));
            n = (n / radix_f64).floor();
        }
        if int_digits.is_empty() {
            int_digits.push('0');
        }
        int_digits.reverse();
        let int_str: String = int_digits.into_iter().collect();

        let mut frac = abs - int_part;
        if frac == 0.0 {
            return format!("{sign}{int_str}");
        }

        let mut frac_str = String::new();
        let mut digits = 0usize;
        while frac > 0.0 && digits < 16 {
            frac *= radix_f64;
            let digit = frac.floor() as u32;
            frac_str.push(Self::radix_digit_char(digit));
            frac -= digit as f64;
            digits += 1;
            if frac.abs() < f64::EPSILON {
                break;
            }
        }
        while frac_str.ends_with('0') {
            frac_str.pop();
        }

        if frac_str.is_empty() {
            format!("{sign}{int_str}")
        } else {
            format!("{sign}{int_str}.{frac_str}")
        }
    }

    pub(super) fn radix_digit_char(value: u32) -> char {
        if value < 10 {
            char::from(b'0' + value as u8)
        } else {
            char::from(b'a' + (value - 10) as u8)
        }
    }

    pub(super) fn sum_precise(values: &[Value]) -> f64 {
        let mut sum = 0.0f64;
        let mut compensation = 0.0f64;
        for value in values {
            let value = Self::coerce_number_for_global(value);
            let adjusted = value - compensation;
            let next = sum + adjusted;
            compensation = (next - sum) - adjusted;
            sum = next;
        }
        sum
    }

    pub(super) fn js_math_round(value: f64) -> f64 {
        if !value.is_finite() || value == 0.0 {
            return value;
        }
        if (-0.5..0.0).contains(&value) {
            return -0.0;
        }
        let floor = value.floor();
        let frac = value - floor;
        if frac < 0.5 { floor } else { floor + 1.0 }
    }

    pub(super) fn js_math_sign(value: f64) -> f64 {
        if value.is_nan() {
            f64::NAN
        } else if value == 0.0 {
            value
        } else if value > 0.0 {
            1.0
        } else {
            -1.0
        }
    }

    pub(super) fn to_i32_for_math(value: &Value) -> i32 {
        let numeric = Self::coerce_number_for_global(value);
        if !numeric.is_finite() {
            return 0;
        }
        let unsigned = numeric.trunc().rem_euclid(4_294_967_296.0);
        if unsigned >= 2_147_483_648.0 {
            (unsigned - 4_294_967_296.0) as i32
        } else {
            unsigned as i32
        }
    }

    pub(super) fn to_u32_for_math(value: &Value) -> u32 {
        let numeric = Self::coerce_number_for_global(value);
        if !numeric.is_finite() {
            return 0;
        }
        numeric.trunc().rem_euclid(4_294_967_296.0) as u32
    }

    pub(super) fn math_f16round(value: f64) -> f64 {
        let half = Self::f32_to_f16_bits(value as f32);
        Self::f16_bits_to_f32(half) as f64
    }

    pub(super) fn f32_to_f16_bits(value: f32) -> u16 {
        let bits = value.to_bits();
        let sign = ((bits >> 16) & 0x8000) as u16;
        let exp = ((bits >> 23) & 0xff) as i32;
        let mant = bits & 0x007f_ffff;

        if exp == 0xff {
            if mant == 0 {
                return sign | 0x7c00;
            }
            return sign | 0x7e00;
        }

        let exp16 = exp - 127 + 15;
        if exp16 >= 0x1f {
            return sign | 0x7c00;
        }

        if exp16 <= 0 {
            if exp16 < -10 {
                return sign;
            }
            let mantissa = mant | 0x0080_0000;
            let shift = (14 - exp16) as u32;
            let mut half_mant = mantissa >> shift;
            let round_bit = 1u32 << (shift - 1);
            if (mantissa & round_bit) != 0
                && ((mantissa & (round_bit - 1)) != 0 || (half_mant & 1) != 0)
            {
                half_mant += 1;
            }
            return sign | (half_mant as u16);
        }

        let mut half_exp = (exp16 as u16) << 10;
        let mut half_mant = (mant >> 13) as u16;
        let round_bits = mant & 0x1fff;
        if round_bits > 0x1000 || (round_bits == 0x1000 && (half_mant & 1) != 0) {
            half_mant = half_mant.wrapping_add(1);
            if half_mant == 0x0400 {
                half_mant = 0;
                half_exp = half_exp.wrapping_add(0x0400);
                if half_exp >= 0x7c00 {
                    return sign | 0x7c00;
                }
            }
        }
        sign | half_exp | half_mant
    }

    pub(super) fn f16_bits_to_f32(bits: u16) -> f32 {
        let sign = ((bits & 0x8000) as u32) << 16;
        let exp = ((bits >> 10) & 0x1f) as u32;
        let mant = (bits & 0x03ff) as u32;

        let out_bits = if exp == 0 {
            if mant == 0 {
                sign
            } else {
                let mut mantissa = mant;
                let mut exp_val = -14i32;
                while (mantissa & 0x0400) == 0 {
                    mantissa <<= 1;
                    exp_val -= 1;
                }
                mantissa &= 0x03ff;
                let exp32 = ((exp_val + 127) as u32) << 23;
                sign | exp32 | (mantissa << 13)
            }
        } else if exp == 0x1f {
            sign | 0x7f80_0000 | (mant << 13)
        } else {
            let exp32 = (((exp as i32) - 15 + 127) as u32) << 23;
            sign | exp32 | (mant << 13)
        };

        f32::from_bits(out_bits)
    }

    pub(super) fn new_date_value(timestamp_ms: i64) -> Value {
        Value::Date(Rc::new(RefCell::new(timestamp_ms)))
    }

    pub(super) fn resolve_date_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<i64>>> {
        match env.get(target) {
            Some(Value::Date(value)) => Ok(value.clone()),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a Date",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(super) fn coerce_date_timestamp_ms(&self, value: &Value) -> i64 {
        match value {
            Value::Date(value) => *value.borrow(),
            Value::String(value) => Self::parse_date_string_to_epoch_ms(value).unwrap_or(0),
            _ => Self::value_to_i64(value),
        }
    }

    pub(super) fn resolve_array_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<Vec<Value>>>> {
        match env.get(target) {
            Some(Value::Array(values)) => Ok(values.clone()),
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

    pub(super) fn resolve_array_buffer_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<ArrayBufferValue>>> {
        match env.get(target) {
            Some(Value::ArrayBuffer(buffer)) => Ok(buffer.clone()),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not an ArrayBuffer",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(super) fn resolve_typed_array_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<TypedArrayValue>>> {
        match env.get(target) {
            Some(Value::TypedArray(array)) => Ok(array.clone()),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a TypedArray",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(super) fn normalize_blob_type(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.as_bytes().iter().all(|b| (0x20..=0x7e).contains(b)) {
            trimmed.to_ascii_lowercase()
        } else {
            String::new()
        }
    }

    pub(super) fn new_blob_value(bytes: Vec<u8>, mime_type: String) -> Value {
        Value::Blob(Rc::new(RefCell::new(BlobValue { bytes, mime_type })))
    }

    pub(super) fn new_readable_stream_placeholder_value() -> Value {
        Self::new_object_value(vec![(
            INTERNAL_READABLE_STREAM_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )])
    }

    pub(super) fn new_uint8_typed_array_from_bytes(bytes: &[u8]) -> Value {
        let buffer = Rc::new(RefCell::new(ArrayBufferValue {
            bytes: bytes.to_vec(),
            max_byte_length: None,
            detached: false,
        }));
        Value::TypedArray(Rc::new(RefCell::new(TypedArrayValue {
            kind: TypedArrayKind::Uint8,
            buffer,
            byte_offset: 0,
            fixed_length: Some(bytes.len()),
        })))
    }

    pub(super) fn typed_array_raw_bytes(&self, array: &Rc<RefCell<TypedArrayValue>>) -> Vec<u8> {
        let (buffer, byte_offset, byte_length) = {
            let array = array.borrow();
            (
                array.buffer.clone(),
                array.byte_offset,
                array.observed_byte_length(),
            )
        };
        if byte_length == 0 {
            return Vec::new();
        }
        let buffer = buffer.borrow();
        let start = byte_offset.min(buffer.byte_length());
        let end = start.saturating_add(byte_length).min(buffer.byte_length());
        if end <= start {
            Vec::new()
        } else {
            buffer.bytes[start..end].to_vec()
        }
    }

    pub(super) fn blob_part_bytes(&self, part: &Value) -> Vec<u8> {
        match part {
            Value::Blob(blob) => blob.borrow().bytes.clone(),
            Value::ArrayBuffer(buffer) => buffer.borrow().bytes.clone(),
            Value::TypedArray(array) => self.typed_array_raw_bytes(array),
            Value::String(text) => text.as_bytes().to_vec(),
            other => other.as_string().into_bytes(),
        }
    }

    pub(super) fn eval_blob_construct(
        &mut self,
        parts: &Option<Box<Expr>>,
        options: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Blob constructor must be called with new".into(),
            ));
        }

        let mut bytes = Vec::new();
        if let Some(parts) = parts {
            let parts_value = self.eval_expr(parts, env, event_param, event)?;
            if !matches!(parts_value, Value::Undefined | Value::Null) {
                let items = self
                    .array_like_values_from_value(&parts_value)
                    .map_err(|_| {
                        Error::ScriptRuntime(
                            "Blob constructor first argument must be an array-like or iterable"
                                .into(),
                        )
                    })?;
                for item in items {
                    bytes.extend(self.blob_part_bytes(&item));
                }
            }
        }

        let mut mime_type = String::new();
        if let Some(options) = options {
            let options = self.eval_expr(options, env, event_param, event)?;
            match options {
                Value::Undefined | Value::Null => {}
                Value::Object(entries) => {
                    let entries = entries.borrow();
                    if let Some(value) = Self::object_get_entry(&entries, "type") {
                        mime_type = Self::normalize_blob_type(&value.as_string());
                    }
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "Blob options must be an object".into(),
                    ));
                }
            }
        }

        Ok(Self::new_blob_value(bytes, mime_type))
    }

    pub(super) fn eval_blob_member_call(
        &mut self,
        blob: &Rc<RefCell<BlobValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "text" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.text does not take arguments".into(),
                    ));
                }
                let text = String::from_utf8_lossy(&blob.borrow().bytes).to_string();
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, Value::String(text))?;
                Ok(Some(Value::Promise(promise)))
            }
            "arrayBuffer" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.arrayBuffer does not take arguments".into(),
                    ));
                }
                let bytes = blob.borrow().bytes.clone();
                let promise = self.new_pending_promise();
                self.promise_resolve(
                    &promise,
                    Value::ArrayBuffer(Rc::new(RefCell::new(ArrayBufferValue {
                        bytes,
                        max_byte_length: None,
                        detached: false,
                    }))),
                )?;
                Ok(Some(Value::Promise(promise)))
            }
            "bytes" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.bytes does not take arguments".into(),
                    ));
                }
                let bytes = blob.borrow().bytes.clone();
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, Self::new_uint8_typed_array_from_bytes(&bytes))?;
                Ok(Some(Value::Promise(promise)))
            }
            "stream" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Blob.stream does not take arguments".into(),
                    ));
                }
                Ok(Some(Self::new_readable_stream_placeholder_value()))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn eval_typed_array_member_call(
        &mut self,
        array: &Rc<RefCell<TypedArrayValue>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "join" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.join supports at most one argument".into(),
                    ));
                }
                if array.borrow().buffer.borrow().detached {
                    return Err(Error::ScriptRuntime(
                        "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
                    ));
                }
                let separator = if let Some(first) = args.first() {
                    if matches!(first, Value::Undefined) {
                        ",".to_string()
                    } else {
                        first.as_string()
                    }
                } else {
                    ",".to_string()
                };
                let joined = self
                    .typed_array_snapshot(array)?
                    .into_iter()
                    .map(|value| value.as_string())
                    .collect::<Vec<_>>()
                    .join(&separator);
                Ok(Some(Value::String(joined)))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn to_non_negative_usize(value: &Value, label: &str) -> Result<usize> {
        let n = Self::value_to_i64(value);
        if n < 0 {
            return Err(Error::ScriptRuntime(format!(
                "{label} must be a non-negative integer"
            )));
        }
        usize::try_from(n).map_err(|_| Error::ScriptRuntime(format!("{label} is too large")))
    }

    pub(super) fn eval_call_args_with_spread(
        &mut self,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Vec<Value>> {
        let mut evaluated = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                Expr::Spread(inner) => {
                    let spread_value = self.eval_expr(inner, env, event_param, event)?;
                    evaluated.extend(self.spread_iterable_values_from_value(&spread_value)?);
                }
                _ => evaluated.push(self.eval_expr(arg, env, event_param, event)?),
            }
        }
        Ok(evaluated)
    }

    pub(super) fn spread_iterable_values_from_value(&self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Array(values) => Ok(values.borrow().clone()),
            Value::TypedArray(values) => self.typed_array_snapshot(values),
            Value::Map(map) => {
                let map = map.borrow();
                Ok(map
                    .entries
                    .iter()
                    .map(|(key, value)| Self::new_array_value(vec![key.clone(), value.clone()]))
                    .collect::<Vec<_>>())
            }
            Value::Set(set) => Ok(set.borrow().values.clone()),
            Value::String(text) => Ok(text
                .chars()
                .map(|ch| Value::String(ch.to_string()))
                .collect::<Vec<_>>()),
            Value::NodeList(nodes) => Ok(nodes.iter().copied().map(Value::Node).collect()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    return Ok(Self::url_search_params_pairs_from_object_entries(&entries)
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>());
                }
                Err(Error::ScriptRuntime("spread source is not iterable".into()))
            }
            _ => Err(Error::ScriptRuntime("spread source is not iterable".into())),
        }
    }

    pub(super) fn array_like_values_from_value(&self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Array(values) => Ok(values.borrow().clone()),
            Value::TypedArray(values) => self.typed_array_snapshot(values),
            Value::Map(map) => {
                let map = map.borrow();
                Ok(map
                    .entries
                    .iter()
                    .map(|(key, value)| Self::new_array_value(vec![key.clone(), value.clone()]))
                    .collect::<Vec<_>>())
            }
            Value::Set(set) => Ok(set.borrow().values.clone()),
            Value::String(text) => Ok(text
                .chars()
                .map(|ch| Value::String(ch.to_string()))
                .collect::<Vec<_>>()),
            Value::NodeList(nodes) => Ok(nodes.iter().copied().map(Value::Node).collect()),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    return Ok(Self::url_search_params_pairs_from_object_entries(&entries)
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>());
                }
                let length_value =
                    Self::object_get_entry(&entries, "length").unwrap_or(Value::Number(0));
                let length = Self::to_non_negative_usize(&length_value, "array-like length")?;
                let mut out = Vec::with_capacity(length);
                for index in 0..length {
                    let key = index.to_string();
                    out.push(Self::object_get_entry(&entries, &key).unwrap_or(Value::Undefined));
                }
                Ok(out)
            }
            _ => Err(Error::ScriptRuntime(
                "expected an array-like or iterable source".into(),
            )),
        }
    }

    pub(super) fn new_array_buffer_value(
        byte_length: usize,
        max_byte_length: Option<usize>,
    ) -> Value {
        Value::ArrayBuffer(Rc::new(RefCell::new(ArrayBufferValue {
            bytes: vec![0; byte_length],
            max_byte_length,
            detached: false,
        })))
    }

    pub(super) fn new_typed_array_with_length(
        &mut self,
        kind: TypedArrayKind,
        length: usize,
    ) -> Result<Value> {
        let byte_length = length.saturating_mul(kind.bytes_per_element());
        let buffer = Rc::new(RefCell::new(ArrayBufferValue {
            bytes: vec![0; byte_length],
            max_byte_length: None,
            detached: false,
        }));
        Ok(Value::TypedArray(Rc::new(RefCell::new(TypedArrayValue {
            kind,
            buffer,
            byte_offset: 0,
            fixed_length: Some(length),
        }))))
    }

    pub(super) fn new_typed_array_view(
        &self,
        kind: TypedArrayKind,
        buffer: Rc<RefCell<ArrayBufferValue>>,
        byte_offset: usize,
        length: Option<usize>,
    ) -> Result<Value> {
        let bytes_per_element = kind.bytes_per_element();
        if byte_offset % bytes_per_element != 0 {
            return Err(Error::ScriptRuntime(format!(
                "start offset of {} should be a multiple of {}",
                kind.name(),
                bytes_per_element
            )));
        }

        let buffer_len = buffer.borrow().byte_length();
        if byte_offset > buffer_len {
            return Err(Error::ScriptRuntime(
                "typed array view bounds are outside the buffer".into(),
            ));
        }

        if let Some(length) = length {
            let required = byte_offset.saturating_add(length.saturating_mul(bytes_per_element));
            if required > buffer_len {
                return Err(Error::ScriptRuntime(
                    "typed array view bounds are outside the buffer".into(),
                ));
            }
        } else {
            let remaining = buffer_len.saturating_sub(byte_offset);
            if remaining % bytes_per_element != 0 {
                return Err(Error::ScriptRuntime(format!(
                    "byte length of {} should be a multiple of {}",
                    kind.name(),
                    bytes_per_element
                )));
            }
        }

        Ok(Value::TypedArray(Rc::new(RefCell::new(TypedArrayValue {
            kind,
            buffer,
            byte_offset,
            fixed_length: length,
        }))))
    }

    pub(super) fn new_typed_array_from_values(
        &mut self,
        kind: TypedArrayKind,
        values: &[Value],
    ) -> Result<Value> {
        let array = self.new_typed_array_with_length(kind, values.len())?;
        let Value::TypedArray(array) = array else {
            unreachable!();
        };
        for (index, value) in values.iter().enumerate() {
            self.typed_array_set_index(&array, index, value.clone())?;
        }
        Ok(Value::TypedArray(array))
    }

    pub(super) fn typed_array_snapshot(
        &self,
        array: &Rc<RefCell<TypedArrayValue>>,
    ) -> Result<Vec<Value>> {
        let length = array.borrow().observed_length();
        let mut out = Vec::with_capacity(length);
        for index in 0..length {
            out.push(self.typed_array_get_index(array, index)?);
        }
        Ok(out)
    }

    pub(super) fn typed_array_get_index(
        &self,
        array: &Rc<RefCell<TypedArrayValue>>,
        index: usize,
    ) -> Result<Value> {
        let (kind, buffer, byte_offset, length) = {
            let array = array.borrow();
            (
                array.kind,
                array.buffer.clone(),
                array.byte_offset,
                array.observed_length(),
            )
        };
        if index >= length {
            return Ok(Value::Undefined);
        }

        let bytes_per_element = kind.bytes_per_element();
        let start = byte_offset.saturating_add(index.saturating_mul(bytes_per_element));
        let buffer = buffer.borrow();
        if start.saturating_add(bytes_per_element) > buffer.byte_length() {
            return Ok(Value::Undefined);
        }
        let bytes = &buffer.bytes[start..start + bytes_per_element];
        let value = match kind {
            TypedArrayKind::Int8 => Value::Number(i64::from(i8::from_le_bytes([bytes[0]]))),
            TypedArrayKind::Uint8 | TypedArrayKind::Uint8Clamped => {
                Value::Number(i64::from(u8::from_le_bytes([bytes[0]])))
            }
            TypedArrayKind::Int16 => {
                Value::Number(i64::from(i16::from_le_bytes([bytes[0], bytes[1]])))
            }
            TypedArrayKind::Uint16 => {
                Value::Number(i64::from(u16::from_le_bytes([bytes[0], bytes[1]])))
            }
            TypedArrayKind::Int32 => Value::Number(i64::from(i32::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
            ]))),
            TypedArrayKind::Uint32 => Value::Number(i64::from(u32::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
            ]))),
            TypedArrayKind::Float16 => {
                let bits = u16::from_le_bytes([bytes[0], bytes[1]]);
                Self::number_value(Self::f16_bits_to_f32(bits) as f64)
            }
            TypedArrayKind::Float32 => Self::number_value(f64::from(f32::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
            ]))),
            TypedArrayKind::Float64 => Self::number_value(f64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ])),
            TypedArrayKind::BigInt64 => Value::BigInt(JsBigInt::from(i64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))),
            TypedArrayKind::BigUint64 => Value::BigInt(JsBigInt::from(u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))),
        };
        Ok(value)
    }

    pub(super) fn typed_array_number_to_i128(value: f64) -> i128 {
        if !value.is_finite() {
            return 0;
        }
        let value = value.trunc();
        if value >= i128::MAX as f64 {
            i128::MAX
        } else if value <= i128::MIN as f64 {
            i128::MIN
        } else {
            value as i128
        }
    }

    pub(super) fn typed_array_round_half_even(value: f64) -> f64 {
        let floor = value.floor();
        let frac = value - floor;
        if frac < 0.5 {
            floor
        } else if frac > 0.5 {
            floor + 1.0
        } else if (floor as i64) % 2 == 0 {
            floor
        } else {
            floor + 1.0
        }
    }

    pub(super) fn typed_array_bytes_for_value(
        kind: TypedArrayKind,
        value: &Value,
    ) -> Result<Vec<u8>> {
        if kind.is_bigint() {
            let Value::BigInt(value) = value else {
                return Err(Error::ScriptRuntime(
                    "Cannot convert number to BigInt typed array element".into(),
                ));
            };
            let modulus = JsBigInt::one() << 64usize;
            let mut unsigned = value % &modulus;
            if unsigned.sign() == Sign::Minus {
                unsigned += &modulus;
            }
            return match kind {
                TypedArrayKind::BigInt64 => {
                    let cutoff = JsBigInt::one() << 63usize;
                    let signed = if unsigned >= cutoff {
                        unsigned - &modulus
                    } else {
                        unsigned
                    };
                    let value = signed.to_i64().unwrap_or(0);
                    Ok(value.to_le_bytes().to_vec())
                }
                TypedArrayKind::BigUint64 => {
                    let value = unsigned.to_u64().unwrap_or(0);
                    Ok(value.to_le_bytes().to_vec())
                }
                _ => unreachable!(),
            };
        }

        if matches!(value, Value::BigInt(_)) {
            return Err(Error::ScriptRuntime(
                "Cannot convert a BigInt value to a number".into(),
            ));
        }

        let number = Self::coerce_number_for_global(value);
        let bytes = match kind {
            TypedArrayKind::Int8 => {
                let modulus = 1i128 << 8;
                let mut out = Self::typed_array_number_to_i128(number).rem_euclid(modulus);
                if out >= (1i128 << 7) {
                    out -= modulus;
                }
                (out as i8).to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint8 => {
                let out = Self::typed_array_number_to_i128(number).rem_euclid(1i128 << 8);
                (out as u8).to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint8Clamped => {
                let clamped = if number.is_nan() {
                    0.0
                } else {
                    number.clamp(0.0, 255.0)
                };
                let rounded = Self::typed_array_round_half_even(clamped);
                (rounded as u8).to_le_bytes().to_vec()
            }
            TypedArrayKind::Int16 => {
                let modulus = 1i128 << 16;
                let mut out = Self::typed_array_number_to_i128(number).rem_euclid(modulus);
                if out >= (1i128 << 15) {
                    out -= modulus;
                }
                (out as i16).to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint16 => {
                let out = Self::typed_array_number_to_i128(number).rem_euclid(1i128 << 16);
                (out as u16).to_le_bytes().to_vec()
            }
            TypedArrayKind::Int32 => {
                let modulus = 1i128 << 32;
                let mut out = Self::typed_array_number_to_i128(number).rem_euclid(modulus);
                if out >= (1i128 << 31) {
                    out -= modulus;
                }
                (out as i32).to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint32 => {
                let out = Self::typed_array_number_to_i128(number).rem_euclid(1i128 << 32);
                (out as u32).to_le_bytes().to_vec()
            }
            TypedArrayKind::Float16 => {
                let rounded = Self::math_f16round(number);
                let bits = Self::f32_to_f16_bits(rounded as f32);
                bits.to_le_bytes().to_vec()
            }
            TypedArrayKind::Float32 => (number as f32).to_le_bytes().to_vec(),
            TypedArrayKind::Float64 => number.to_le_bytes().to_vec(),
            TypedArrayKind::BigInt64 | TypedArrayKind::BigUint64 => unreachable!(),
        };
        Ok(bytes)
    }

    pub(super) fn typed_array_set_index(
        &mut self,
        array: &Rc<RefCell<TypedArrayValue>>,
        index: usize,
        value: Value,
    ) -> Result<()> {
        let (kind, buffer, byte_offset, length) = {
            let array = array.borrow();
            (
                array.kind,
                array.buffer.clone(),
                array.byte_offset,
                array.observed_length(),
            )
        };
        if index >= length {
            return Ok(());
        }
        let bytes_per_element = kind.bytes_per_element();
        let start = byte_offset.saturating_add(index.saturating_mul(bytes_per_element));
        let bytes = Self::typed_array_bytes_for_value(kind, &value)?;
        if bytes.len() != bytes_per_element {
            return Err(Error::ScriptRuntime(
                "typed array element size mismatch".into(),
            ));
        }
        let mut buffer = buffer.borrow_mut();
        if start.saturating_add(bytes_per_element) > buffer.byte_length() {
            return Ok(());
        }
        buffer.bytes[start..start + bytes_per_element].copy_from_slice(&bytes);
        Ok(())
    }

    pub(super) fn eval_array_buffer_construct(
        &mut self,
        byte_length: &Option<Box<Expr>>,
        options: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "ArrayBuffer constructor must be called with new".into(),
            ));
        }
        let byte_length = if let Some(byte_length) = byte_length {
            let value = self.eval_expr(byte_length, env, event_param, event)?;
            Self::to_non_negative_usize(&value, "ArrayBuffer byteLength")?
        } else {
            0
        };
        let max_byte_length = if let Some(options) = options {
            let options = self.eval_expr(options, env, event_param, event)?;
            match options {
                Value::Undefined | Value::Null => None,
                Value::Object(entries) => {
                    let entries = entries.borrow();
                    if let Some(value) = Self::object_get_entry(&entries, "maxByteLength") {
                        Some(Self::to_non_negative_usize(
                            &value,
                            "ArrayBuffer maxByteLength",
                        )?)
                    } else {
                        None
                    }
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "ArrayBuffer options must be an object".into(),
                    ));
                }
            }
        } else {
            None
        };
        if max_byte_length.is_some_and(|max| byte_length > max) {
            return Err(Error::ScriptRuntime(
                "ArrayBuffer byteLength exceeds maxByteLength".into(),
            ));
        }
        Ok(Self::new_array_buffer_value(byte_length, max_byte_length))
    }

    pub(super) fn resize_array_buffer(
        &mut self,
        buffer: &Rc<RefCell<ArrayBufferValue>>,
        new_byte_length: i64,
    ) -> Result<()> {
        Self::ensure_array_buffer_not_detached(buffer, "resize")?;
        if new_byte_length < 0 {
            return Err(Error::ScriptRuntime(
                "ArrayBuffer resize length must be non-negative".into(),
            ));
        }
        let new_byte_length = usize::try_from(new_byte_length)
            .map_err(|_| Error::ScriptRuntime("ArrayBuffer resize length is too large".into()))?;
        let max_byte_length = buffer.borrow().max_byte_length;
        let Some(max_byte_length) = max_byte_length else {
            return Err(Error::ScriptRuntime("ArrayBuffer is not resizable".into()));
        };
        if new_byte_length > max_byte_length {
            return Err(Error::ScriptRuntime(
                "ArrayBuffer resize exceeds maxByteLength".into(),
            ));
        }
        buffer.borrow_mut().bytes.resize(new_byte_length, 0);
        Ok(())
    }

    pub(super) fn ensure_array_buffer_not_detached(
        buffer: &Rc<RefCell<ArrayBufferValue>>,
        method: &str,
    ) -> Result<()> {
        if buffer.borrow().detached {
            return Err(Error::ScriptRuntime(format!(
                "Cannot perform ArrayBuffer.prototype.{method} on a detached ArrayBuffer"
            )));
        }
        Ok(())
    }

    pub(super) fn transfer_array_buffer(
        &mut self,
        buffer: &Rc<RefCell<ArrayBufferValue>>,
        to_fixed_length: bool,
    ) -> Result<Value> {
        Self::ensure_array_buffer_not_detached(
            buffer,
            if to_fixed_length {
                "transferToFixedLength"
            } else {
                "transfer"
            },
        )?;
        let mut source = buffer.borrow_mut();
        let bytes = source.bytes.clone();
        let max_byte_length = if to_fixed_length {
            None
        } else {
            source.max_byte_length
        };
        source.bytes.clear();
        source.max_byte_length = None;
        source.detached = true;
        drop(source);
        Ok(Value::ArrayBuffer(Rc::new(RefCell::new(
            ArrayBufferValue {
                bytes,
                max_byte_length,
                detached: false,
            },
        ))))
    }

    pub(super) fn resize_array_buffer_in_env(
        &mut self,
        env: &HashMap<String, Value>,
        target: &str,
        new_byte_length: i64,
    ) -> Result<()> {
        let buffer = self.resolve_array_buffer_from_env(env, target)?;
        self.resize_array_buffer(&buffer, new_byte_length)
    }

    pub(super) fn eval_typed_array_construct(
        &mut self,
        kind: TypedArrayKind,
        args: &[Expr],
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(format!(
                "{} constructor must be called with new",
                kind.name()
            )));
        }
        if args.len() > 3 {
            return Err(Error::ScriptRuntime(format!(
                "{} supports up to three arguments",
                kind.name()
            )));
        }

        if args.is_empty() {
            return self.new_typed_array_with_length(kind, 0);
        }

        let first = self.eval_expr(&args[0], env, event_param, event)?;
        match (&first, args.len()) {
            (Value::ArrayBuffer(buffer), 1) => {
                self.new_typed_array_view(kind, buffer.clone(), 0, None)
            }
            (Value::TypedArray(source), 1) => {
                let source_kind = source.borrow().kind;
                if kind.is_bigint() != source_kind.is_bigint() {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and Number typed arrays".into(),
                    ));
                }
                let values = self.typed_array_snapshot(source)?;
                self.new_typed_array_from_values(kind, &values)
            }
            (Value::Array(_), 1) | (Value::Object(_), 1) | (Value::String(_), 1) => {
                let values = self.array_like_values_from_value(&first)?;
                self.new_typed_array_from_values(kind, &values)
            }
            (Value::ArrayBuffer(buffer), _) => {
                let byte_offset = if args.len() >= 2 {
                    let offset = self.eval_expr(&args[1], env, event_param, event)?;
                    Self::to_non_negative_usize(&offset, "typed array byteOffset")?
                } else {
                    0
                };
                let length = if args.len() == 3 {
                    let length = self.eval_expr(&args[2], env, event_param, event)?;
                    if matches!(length, Value::Undefined) {
                        None
                    } else {
                        Some(Self::to_non_negative_usize(&length, "typed array length")?)
                    }
                } else {
                    None
                };
                self.new_typed_array_view(kind, buffer.clone(), byte_offset, length)
            }
            _ => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "typed array buffer view requires an ArrayBuffer first argument".into(),
                    ));
                }
                let length = Self::to_non_negative_usize(&first, "typed array length")?;
                self.new_typed_array_with_length(kind, length)
            }
        }
    }

    pub(super) fn eval_typed_array_construct_with_callee(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "constructor must be called with new".into(),
            ));
        }
        let callee = self.eval_expr(callee, env, event_param, event)?;
        match callee {
            Value::TypedArrayConstructor(TypedArrayConstructorKind::Concrete(kind)) => {
                self.eval_typed_array_construct(kind, args, true, env, event_param, event)
            }
            Value::TypedArrayConstructor(TypedArrayConstructorKind::Abstract) => Err(
                Error::ScriptRuntime("Abstract class TypedArray not directly constructable".into()),
            ),
            _ => Err(Error::ScriptRuntime("value is not a constructor".into())),
        }
    }

    pub(super) fn eval_typed_array_static_method(
        &mut self,
        kind: TypedArrayKind,
        method: TypedArrayStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            TypedArrayStaticMethod::From => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(format!(
                        "{}.from requires exactly one argument",
                        kind.name()
                    )));
                }
                let source = self.eval_expr(&args[0], env, event_param, event)?;
                if let Value::TypedArray(source_array) = &source {
                    if kind.is_bigint() != source_array.borrow().kind.is_bigint() {
                        return Err(Error::ScriptRuntime(
                            "cannot mix BigInt and Number typed arrays".into(),
                        ));
                    }
                }
                let values = self.array_like_values_from_value(&source)?;
                self.new_typed_array_from_values(kind, &values)
            }
            TypedArrayStaticMethod::Of => {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.eval_expr(arg, env, event_param, event)?);
                }
                self.new_typed_array_from_values(kind, &values)
            }
        }
    }

    pub(super) fn same_value_zero(&self, left: &Value, right: &Value) -> bool {
        if let (Some(left_num), Some(right_num)) = (
            Self::number_primitive_value(left),
            Self::number_primitive_value(right),
        ) {
            if left_num.is_nan() && right_num.is_nan() {
                return true;
            }
        }
        self.strict_equal(left, right)
    }

    pub(super) fn map_entry_index(&self, map: &MapValue, key: &Value) -> Option<usize> {
        map.entries
            .iter()
            .position(|(existing_key, _)| self.same_value_zero(existing_key, key))
    }

    pub(super) fn map_set_entry(&self, map: &mut MapValue, key: Value, value: Value) {
        if let Some(index) = self.map_entry_index(map, &key) {
            map.entries[index].1 = value;
        } else {
            map.entries.push((key, value));
        }
    }

    pub(super) fn map_entries_array(&self, map: &Rc<RefCell<MapValue>>) -> Vec<Value> {
        map.borrow()
            .entries
            .iter()
            .map(|(key, value)| Self::new_array_value(vec![key.clone(), value.clone()]))
            .collect::<Vec<_>>()
    }

    pub(super) fn set_value_index(&self, set: &SetValue, value: &Value) -> Option<usize> {
        set.values
            .iter()
            .position(|existing_value| self.same_value_zero(existing_value, value))
    }

    pub(super) fn set_add_value(&self, set: &mut SetValue, value: Value) {
        if self.set_value_index(set, &value).is_none() {
            set.values.push(value);
        }
    }

    pub(super) fn set_values_array(&self, set: &Rc<RefCell<SetValue>>) -> Vec<Value> {
        set.borrow().values.clone()
    }

    pub(super) fn set_entries_array(&self, set: &Rc<RefCell<SetValue>>) -> Vec<Value> {
        set.borrow()
            .values
            .iter()
            .map(|value| Self::new_array_value(vec![value.clone(), value.clone()]))
            .collect::<Vec<_>>()
    }

    pub(super) fn set_like_keys_snapshot(&self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Set(set) => Ok(set.borrow().values.clone()),
            Value::Map(map) => Ok(map
                .borrow()
                .entries
                .iter()
                .map(|(key, _)| key.clone())
                .collect::<Vec<_>>()),
            _ => Err(Error::ScriptRuntime(
                "Set composition argument must be set-like (Set or Map)".into(),
            )),
        }
    }

    pub(super) fn set_like_has_value(&self, value: &Value, candidate: &Value) -> Result<bool> {
        match value {
            Value::Set(set) => Ok(self.set_value_index(&set.borrow(), candidate).is_some()),
            Value::Map(map) => Ok(self.map_entry_index(&map.borrow(), candidate).is_some()),
            _ => Err(Error::ScriptRuntime(
                "Set composition argument must be set-like (Set or Map)".into(),
            )),
        }
    }

    pub(super) fn is_url_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_URL_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(super) fn url_object_id_from_entries(entries: &[(String, Value)]) -> Option<usize> {
        match Self::object_get_entry(entries, INTERNAL_URL_OBJECT_ID_KEY) {
            Some(Value::Number(id)) if id > 0 => usize::try_from(id).ok(),
            _ => None,
        }
    }

    pub(super) fn normalize_url_parts_for_serialization(parts: &mut LocationParts) {
        parts.scheme = parts.scheme.to_ascii_lowercase();
        if parts.has_authority {
            parts.hostname = parts.hostname.to_ascii_lowercase();
            let path = if parts.pathname.is_empty() {
                "/".to_string()
            } else if parts.pathname.starts_with('/') {
                parts.pathname.clone()
            } else {
                format!("/{}", parts.pathname)
            };
            parts.pathname = normalize_pathname(&path);
            parts.pathname = encode_uri_like_preserving_percent(&parts.pathname, false);
        } else {
            parts.opaque_path = encode_uri_like_preserving_percent(&parts.opaque_path, false);
        }

        if !parts.search.is_empty() {
            let body = parts
                .search
                .strip_prefix('?')
                .unwrap_or(parts.search.as_str());
            parts.search = format!("?{}", encode_uri_like_preserving_percent(body, false));
        }
        if !parts.hash.is_empty() {
            let body = parts.hash.strip_prefix('#').unwrap_or(parts.hash.as_str());
            parts.hash = format!("#{}", encode_uri_like_preserving_percent(body, false));
        }
    }

    pub(super) fn resolve_url_against_base_parts(input: &str, base: &LocationParts) -> String {
        let input = input.trim();
        if input.is_empty() {
            return base.href();
        }

        if input.starts_with("//") {
            return LocationParts::parse(&format!("{}{}", base.protocol(), input))
                .map(|parts| parts.href())
                .unwrap_or_else(|| input.to_string());
        }

        let mut next = base.clone();
        if input.starts_with('#') {
            next.hash = ensure_hash_prefix(input);
            return next.href();
        }

        if input.starts_with('?') {
            next.search = ensure_search_prefix(input);
            next.hash.clear();
            return next.href();
        }

        if input.starts_with('/') {
            if next.has_authority {
                next.pathname = normalize_pathname(input);
            } else {
                next.opaque_path = input.to_string();
            }
            next.search.clear();
            next.hash.clear();
            return next.href();
        }

        let mut relative = input;
        let mut next_search = String::new();
        let mut next_hash = String::new();
        if let Some(hash_pos) = relative.find('#') {
            next_hash = ensure_hash_prefix(&relative[hash_pos + 1..]);
            relative = &relative[..hash_pos];
        }
        if let Some(search_pos) = relative.find('?') {
            next_search = ensure_search_prefix(&relative[search_pos + 1..]);
            relative = &relative[..search_pos];
        }

        if next.has_authority {
            let base_dir = if let Some((prefix, _)) = next.pathname.rsplit_once('/') {
                if prefix.is_empty() {
                    "/".to_string()
                } else {
                    format!("{prefix}/")
                }
            } else {
                "/".to_string()
            };
            next.pathname = normalize_pathname(&format!("{base_dir}{relative}"));
        } else {
            next.opaque_path = relative.to_string();
        }
        next.search = next_search;
        next.hash = next_hash;
        next.href()
    }

    pub(super) fn resolve_url_string(input: &str, base: Option<&str>) -> Option<String> {
        let input = input.trim();
        if let Some(mut absolute) = LocationParts::parse(input) {
            Self::normalize_url_parts_for_serialization(&mut absolute);
            return Some(absolute.href());
        }

        let base = base?;
        let mut base_parts = LocationParts::parse(base)?;
        Self::normalize_url_parts_for_serialization(&mut base_parts);
        let resolved = Self::resolve_url_against_base_parts(input, &base_parts);
        let mut resolved_parts = LocationParts::parse(&resolved)?;
        Self::normalize_url_parts_for_serialization(&mut resolved_parts);
        Some(resolved_parts.href())
    }

    pub(super) fn sync_url_object_entries_from_parts(
        &self,
        entries: &mut Vec<(String, Value)>,
        parts: &LocationParts,
    ) {
        let href = parts.href();
        Self::object_set_entry(
            entries,
            INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
            Value::String(href.clone()),
        );
        Self::object_set_entry(entries, "href".to_string(), Value::String(href));
        Self::object_set_entry(
            entries,
            "protocol".to_string(),
            Value::String(parts.protocol()),
        );
        Self::object_set_entry(entries, "host".to_string(), Value::String(parts.host()));
        Self::object_set_entry(
            entries,
            "hostname".to_string(),
            Value::String(parts.hostname.clone()),
        );
        Self::object_set_entry(
            entries,
            "port".to_string(),
            Value::String(parts.port.clone()),
        );
        Self::object_set_entry(
            entries,
            "pathname".to_string(),
            Value::String(if parts.has_authority {
                parts.pathname.clone()
            } else {
                parts.opaque_path.clone()
            }),
        );
        Self::object_set_entry(
            entries,
            "search".to_string(),
            Value::String(parts.search.clone()),
        );
        Self::object_set_entry(
            entries,
            "hash".to_string(),
            Value::String(parts.hash.clone()),
        );
        Self::object_set_entry(
            entries,
            "username".to_string(),
            Value::String(parts.username.clone()),
        );
        Self::object_set_entry(
            entries,
            "password".to_string(),
            Value::String(parts.password.clone()),
        );
        Self::object_set_entry(entries, "origin".to_string(), Value::String(parts.origin()));

        let owner_id = Self::url_object_id_from_entries(entries);
        let pairs =
            parse_url_search_params_pairs_from_query_string(&parts.search).unwrap_or_default();
        if let Some(Value::Object(search_params_object)) =
            Self::object_get_entry(entries, "searchParams")
        {
            let mut search_params_entries = search_params_object.borrow_mut();
            Self::set_url_search_params_pairs(&mut search_params_entries, &pairs);
            if let Some(owner_id) = owner_id {
                Self::object_set_entry(
                    &mut search_params_entries,
                    INTERNAL_URL_SEARCH_PARAMS_OWNER_ID_KEY.to_string(),
                    Value::Number(owner_id as i64),
                );
            }
        } else {
            Self::object_set_entry(
                entries,
                "searchParams".to_string(),
                self.new_url_search_params_value(pairs, owner_id),
            );
        }
    }

    pub(super) fn new_url_value_from_href(&mut self, href: &str) -> Result<Value> {
        let mut parts =
            LocationParts::parse(href).ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
        Self::normalize_url_parts_for_serialization(&mut parts);
        let id = self.next_url_object_id;
        self.next_url_object_id = self.next_url_object_id.saturating_add(1);

        let mut entries = vec![
            (INTERNAL_URL_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_URL_OBJECT_ID_KEY.to_string(),
                Value::Number(id as i64),
            ),
        ];
        self.sync_url_object_entries_from_parts(&mut entries, &parts);
        let object = Rc::new(RefCell::new(entries));
        self.url_objects.insert(id, object.clone());
        Ok(Value::Object(object))
    }

    pub(super) fn set_url_object_property(
        &mut self,
        object: &Rc<RefCell<Vec<(String, Value)>>>,
        key: &str,
        value: Value,
    ) -> Result<()> {
        if matches!(key, "origin" | "searchParams") {
            return Err(Error::ScriptRuntime(format!("URL.{key} is read-only")));
        }

        let current_href = {
            let entries = object.borrow();
            Self::object_get_entry(&entries, "href")
                .map(|value| value.as_string())
                .unwrap_or_default()
        };
        let mut parts = LocationParts::parse(&current_href)
            .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
        match key {
            "href" => {
                let href = Self::resolve_url_string(&value.as_string(), None)
                    .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
                parts = LocationParts::parse(&href)
                    .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
            }
            "protocol" => {
                let protocol = value.as_string();
                let protocol = protocol.trim_end_matches(':').to_ascii_lowercase();
                if !is_valid_url_scheme(&protocol) {
                    return Err(Error::ScriptRuntime(format!(
                        "invalid URL.protocol value: {}",
                        value.as_string()
                    )));
                }
                parts.scheme = protocol;
            }
            "host" => {
                let host = value.as_string();
                let (hostname, port) = split_hostname_and_port(host.trim());
                parts.hostname = hostname;
                parts.port = port;
            }
            "hostname" => {
                parts.hostname = value.as_string();
            }
            "port" => {
                parts.port = value.as_string();
            }
            "pathname" => {
                let raw = value.as_string();
                if parts.has_authority {
                    let normalized_input = if raw.starts_with('/') {
                        raw
                    } else {
                        format!("/{raw}")
                    };
                    parts.pathname = normalize_pathname(&normalized_input);
                } else {
                    parts.opaque_path = raw;
                }
            }
            "search" => {
                parts.search = ensure_search_prefix(&value.as_string());
            }
            "hash" => {
                parts.hash = ensure_hash_prefix(&value.as_string());
            }
            "username" => {
                parts.username = value.as_string();
            }
            "password" => {
                parts.password = value.as_string();
            }
            _ => {
                Self::object_set_entry(&mut object.borrow_mut(), key.to_string(), value);
                return Ok(());
            }
        }

        Self::normalize_url_parts_for_serialization(&mut parts);
        self.sync_url_object_entries_from_parts(&mut object.borrow_mut(), &parts);
        Ok(())
    }

    pub(super) fn sync_url_search_params_owner(
        &mut self,
        object: &Rc<RefCell<Vec<(String, Value)>>>,
    ) {
        let (owner_id, pairs) = {
            let entries = object.borrow();
            let owner_id =
                match Self::object_get_entry(&entries, INTERNAL_URL_SEARCH_PARAMS_OWNER_ID_KEY) {
                    Some(Value::Number(id)) if id > 0 => usize::try_from(id).ok(),
                    _ => None,
                };
            let pairs = Self::url_search_params_pairs_from_object_entries(&entries);
            (owner_id, pairs)
        };
        let Some(owner_id) = owner_id else {
            return;
        };
        let Some(url_object) = self.url_objects.get(&owner_id).cloned() else {
            return;
        };

        let current_href = {
            let entries = url_object.borrow();
            Self::object_get_entry(&entries, "href")
                .map(|value| value.as_string())
                .unwrap_or_default()
        };
        let Some(mut parts) = LocationParts::parse(&current_href) else {
            return;
        };

        let serialized = serialize_url_search_params_pairs(&pairs);
        parts.search = if serialized.is_empty() {
            String::new()
        } else {
            format!("?{serialized}")
        };
        Self::normalize_url_parts_for_serialization(&mut parts);
        self.sync_url_object_entries_from_parts(&mut url_object.borrow_mut(), &parts);
    }

    pub(super) fn is_url_search_params_object(entries: &[(String, Value)]) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_URL_SEARCH_PARAMS_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(super) fn url_search_params_pairs_to_value(pairs: &[(String, String)]) -> Value {
        Self::new_array_value(
            pairs
                .iter()
                .map(|(name, value)| {
                    Self::new_array_value(vec![
                        Value::String(name.clone()),
                        Value::String(value.clone()),
                    ])
                })
                .collect::<Vec<_>>(),
        )
    }

    pub(super) fn url_search_params_pairs_from_object_entries(
        entries: &[(String, Value)],
    ) -> Vec<(String, String)> {
        let Some(Value::Array(list)) =
            Self::object_get_entry(entries, INTERNAL_URL_SEARCH_PARAMS_ENTRIES_KEY)
        else {
            return Vec::new();
        };
        let snapshot = list.borrow().clone();
        let mut pairs = Vec::new();
        for item in snapshot {
            let Value::Array(pair) = item else {
                continue;
            };
            let pair = pair.borrow();
            if pair.is_empty() {
                continue;
            }
            let name = pair[0].as_string();
            let value = pair.get(1).map(Value::as_string).unwrap_or_default();
            pairs.push((name, value));
        }
        pairs
    }

    pub(super) fn set_url_search_params_pairs(
        entries: &mut Vec<(String, Value)>,
        pairs: &[(String, String)],
    ) {
        Self::object_set_entry(
            entries,
            INTERNAL_URL_SEARCH_PARAMS_ENTRIES_KEY.to_string(),
            Self::url_search_params_pairs_to_value(pairs),
        );
    }

    pub(super) fn new_url_search_params_value(
        &self,
        pairs: Vec<(String, String)>,
        owner_id: Option<usize>,
    ) -> Value {
        let mut entries = vec![(
            INTERNAL_URL_SEARCH_PARAMS_OBJECT_KEY.to_string(),
            Value::Bool(true),
        )];
        if let Some(owner_id) = owner_id {
            entries.push((
                INTERNAL_URL_SEARCH_PARAMS_OWNER_ID_KEY.to_string(),
                Value::Number(owner_id as i64),
            ));
        }
        Self::set_url_search_params_pairs(&mut entries, &pairs);
        Self::new_object_value(entries)
    }

    pub(super) fn resolve_url_search_params_object_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<Vec<(String, Value)>>>> {
        match env.get(target) {
            Some(Value::Object(entries)) => {
                if Self::is_url_search_params_object(&entries.borrow()) {
                    Ok(entries.clone())
                } else {
                    Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a URLSearchParams",
                        target
                    )))
                }
            }
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a URLSearchParams",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    pub(super) fn url_search_params_pairs_from_init_value(
        &self,
        init: &Value,
    ) -> Result<Vec<(String, String)>> {
        match init {
            Value::Undefined | Value::Null => Ok(Vec::new()),
            Value::String(text) => parse_url_search_params_pairs_from_query_string(text),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if Self::is_url_search_params_object(&entries) {
                    Ok(Self::url_search_params_pairs_from_object_entries(&entries))
                } else {
                    let mut pairs = Vec::new();
                    for (name, value) in entries.iter() {
                        if Self::is_internal_object_key(name) {
                            continue;
                        }
                        pairs.push((name.clone(), value.as_string()));
                    }
                    Ok(pairs)
                }
            }
            Value::Array(_) | Value::Map(_) | Value::Set(_) | Value::TypedArray(_) => {
                let iterable = self.array_like_values_from_value(init)?;
                let mut pairs = Vec::new();
                for entry in iterable {
                    let pair = self.array_like_values_from_value(&entry).map_err(|_| {
                        Error::ScriptRuntime(
                            "URLSearchParams iterable values must be [name, value] pairs".into(),
                        )
                    })?;
                    if pair.len() < 2 {
                        return Err(Error::ScriptRuntime(
                            "URLSearchParams iterable values must be [name, value] pairs".into(),
                        ));
                    }
                    pairs.push((pair[0].as_string(), pair[1].as_string()));
                }
                Ok(pairs)
            }
            other => parse_url_search_params_pairs_from_query_string(&other.as_string()),
        }
    }

    pub(super) fn eval_url_construct(
        &mut self,
        input: &Option<Box<Expr>>,
        base: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "URL constructor must be called with new".into(),
            ));
        }

        let input = input
            .as_ref()
            .map(|expr| self.eval_expr(expr, env, event_param, event))
            .transpose()?
            .unwrap_or(Value::Undefined)
            .as_string();
        let base = base
            .as_ref()
            .map(|expr| self.eval_expr(expr, env, event_param, event))
            .transpose()?
            .map(|value| value.as_string());

        let href = Self::resolve_url_string(&input, base.as_deref())
            .ok_or_else(|| Error::ScriptRuntime("Invalid URL".into()))?;
        self.new_url_value_from_href(&href)
    }

    pub(super) fn eval_url_static_method(
        &mut self,
        method: UrlStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            UrlStaticMethod::CanParse => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URL.canParse requires a URL argument and optional base".into(),
                    ));
                }
                let input = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                let base = if args.len() == 2 {
                    Some(
                        self.eval_expr(&args[1], env, event_param, event)?
                            .as_string(),
                    )
                } else {
                    None
                };
                Ok(Value::Bool(
                    Self::resolve_url_string(&input, base.as_deref()).is_some(),
                ))
            }
            UrlStaticMethod::Parse => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URL.parse requires a URL argument and optional base".into(),
                    ));
                }
                let input = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                let base = if args.len() == 2 {
                    Some(
                        self.eval_expr(&args[1], env, event_param, event)?
                            .as_string(),
                    )
                } else {
                    None
                };
                if let Some(href) = Self::resolve_url_string(&input, base.as_deref()) {
                    self.new_url_value_from_href(&href)
                } else {
                    Ok(Value::Null)
                }
            }
            UrlStaticMethod::CreateObjectUrl => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URL.createObjectURL requires exactly one argument".into(),
                    ));
                }
                let value = self.eval_expr(&args[0], env, event_param, event)?;
                let Value::Blob(blob) = value else {
                    return Err(Error::ScriptRuntime(
                        "URL.createObjectURL requires a Blob argument".into(),
                    ));
                };
                let object_url = format!("blob:bt-{}", self.next_blob_url_id);
                self.next_blob_url_id = self.next_blob_url_id.saturating_add(1);
                self.blob_url_objects.insert(object_url.clone(), blob);
                Ok(Value::String(object_url))
            }
            UrlStaticMethod::RevokeObjectUrl => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URL.revokeObjectURL requires exactly one argument".into(),
                    ));
                }
                let object_url = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                self.blob_url_objects.remove(&object_url);
                Ok(Value::Undefined)
            }
        }
    }

    pub(super) fn eval_url_static_member_call_from_values(
        &mut self,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "canParse" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URL.canParse requires a URL argument and optional base".into(),
                    ));
                }
                let input = args[0].as_string();
                let base = args.get(1).map(Value::as_string);
                Ok(Some(Value::Bool(
                    Self::resolve_url_string(&input, base.as_deref()).is_some(),
                )))
            }
            "parse" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URL.parse requires a URL argument and optional base".into(),
                    ));
                }
                let input = args[0].as_string();
                let base = args.get(1).map(Value::as_string);
                if let Some(href) = Self::resolve_url_string(&input, base.as_deref()) {
                    Ok(Some(self.new_url_value_from_href(&href)?))
                } else {
                    Ok(Some(Value::Null))
                }
            }
            "createObjectURL" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URL.createObjectURL requires exactly one argument".into(),
                    ));
                }
                let Value::Blob(blob) = args[0].clone() else {
                    return Err(Error::ScriptRuntime(
                        "URL.createObjectURL requires a Blob argument".into(),
                    ));
                };
                let object_url = format!("blob:bt-{}", self.next_blob_url_id);
                self.next_blob_url_id = self.next_blob_url_id.saturating_add(1);
                self.blob_url_objects.insert(object_url.clone(), blob);
                Ok(Some(Value::String(object_url)))
            }
            "revokeObjectURL" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URL.revokeObjectURL requires exactly one argument".into(),
                    ));
                }
                self.blob_url_objects.remove(&args[0].as_string());
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn eval_url_member_call(
        &self,
        object: &Rc<RefCell<Vec<(String, Value)>>>,
        member: &str,
        args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "toString" | "toJSON" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(format!(
                        "URL.{member} does not take arguments"
                    )));
                }
                let href = {
                    let entries = object.borrow();
                    Self::object_get_entry(&entries, "href")
                        .map(|value| value.as_string())
                        .unwrap_or_default()
                };
                Ok(Some(Value::String(href)))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn eval_url_search_params_member_call(
        &mut self,
        object: &Rc<RefCell<Vec<(String, Value)>>>,
        member: &str,
        args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "append" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.append requires exactly two arguments".into(),
                    ));
                }
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&entries);
                    pairs.push((args[0].as_string(), args[1].as_string()));
                    Self::set_url_search_params_pairs(&mut entries, &pairs);
                }
                self.sync_url_search_params_owner(object);
                Ok(Some(Value::Undefined))
            }
            "delete" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.delete requires one or two arguments".into(),
                    ));
                }
                let name = args[0].as_string();
                let value = args.get(1).map(Value::as_string);
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&entries);
                    pairs.retain(|(entry_name, entry_value)| {
                        if entry_name != &name {
                            return true;
                        }
                        if let Some(value) = value.as_ref() {
                            entry_value != value
                        } else {
                            false
                        }
                    });
                    Self::set_url_search_params_pairs(&mut entries, &pairs);
                }
                self.sync_url_search_params_owner(object);
                Ok(Some(Value::Undefined))
            }
            "get" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.get requires exactly one argument".into(),
                    ));
                }
                let name = args[0].as_string();
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                let value = pairs
                    .into_iter()
                    .find_map(|(entry_name, entry_value)| {
                        (entry_name == name).then_some(entry_value)
                    })
                    .map(Value::String)
                    .unwrap_or(Value::Null);
                Ok(Some(value))
            }
            "getAll" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.getAll requires exactly one argument".into(),
                    ));
                }
                let name = args[0].as_string();
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Self::new_array_value(
                    pairs
                        .into_iter()
                        .filter_map(|(entry_name, entry_value)| {
                            (entry_name == name).then(|| Value::String(entry_value))
                        })
                        .collect::<Vec<_>>(),
                )))
            }
            "has" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.has requires one or two arguments".into(),
                    ));
                }
                let name = args[0].as_string();
                let value = args.get(1).map(Value::as_string);
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                let has = pairs.into_iter().any(|(entry_name, entry_value)| {
                    if entry_name != name {
                        return false;
                    }
                    if let Some(value) = value.as_ref() {
                        &entry_value == value
                    } else {
                        true
                    }
                });
                Ok(Some(Value::Bool(has)))
            }
            "set" => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.set requires exactly two arguments".into(),
                    ));
                }
                let name = args[0].as_string();
                let value = args[1].as_string();
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&entries);
                    if let Some(first_match) =
                        pairs.iter().position(|(entry_name, _)| entry_name == &name)
                    {
                        pairs[first_match].1 = value;
                        let mut index = pairs.len();
                        while index > 0 {
                            index -= 1;
                            if index != first_match && pairs[index].0 == name {
                                pairs.remove(index);
                            }
                        }
                    } else {
                        pairs.push((name, value));
                    }
                    Self::set_url_search_params_pairs(&mut entries, &pairs);
                }
                self.sync_url_search_params_owner(object);
                Ok(Some(Value::Undefined))
            }
            "entries" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.entries does not take arguments".into(),
                    ));
                }
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Self::new_array_value(
                    pairs
                        .into_iter()
                        .map(|(name, value)| {
                            Self::new_array_value(vec![Value::String(name), Value::String(value)])
                        })
                        .collect::<Vec<_>>(),
                )))
            }
            "keys" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.keys does not take arguments".into(),
                    ));
                }
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Self::new_array_value(
                    pairs
                        .into_iter()
                        .map(|(name, _)| Value::String(name))
                        .collect::<Vec<_>>(),
                )))
            }
            "values" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.values does not take arguments".into(),
                    ));
                }
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Self::new_array_value(
                    pairs
                        .into_iter()
                        .map(|(_, value)| Value::String(value))
                        .collect::<Vec<_>>(),
                )))
            }
            "sort" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.sort does not take arguments".into(),
                    ));
                }
                {
                    let mut entries = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&entries);
                    pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                    Self::set_url_search_params_pairs(&mut entries, &pairs);
                }
                self.sync_url_search_params_owner(object);
                Ok(Some(Value::Undefined))
            }
            "forEach" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.forEach requires a callback and optional thisArg".into(),
                    ));
                }
                let callback = args[0].clone();
                let snapshot = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                for (entry_name, entry_value) in snapshot {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[
                            Value::String(entry_value),
                            Value::String(entry_name),
                            Value::Object(object.clone()),
                        ],
                        event,
                    )?;
                }
                Ok(Some(Value::Undefined))
            }
            "toString" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.toString does not take arguments".into(),
                    ));
                }
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Some(Value::String(serialize_url_search_params_pairs(
                    &pairs,
                ))))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn eval_url_search_params_construct(
        &mut self,
        init: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "URLSearchParams constructor must be called with new".into(),
            ));
        }
        let init = init
            .as_ref()
            .map(|expr| self.eval_expr(expr, env, event_param, event))
            .transpose()?
            .unwrap_or(Value::Undefined);
        let pairs = self.url_search_params_pairs_from_init_value(&init)?;
        Ok(self.new_url_search_params_value(pairs, None))
    }

    pub(super) fn eval_url_search_params_method(
        &mut self,
        target: &str,
        method: UrlSearchParamsInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if matches!(method, UrlSearchParamsInstanceMethod::GetAll) {
            match env.get(target) {
                Some(Value::FormData(entries)) => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "FormData.getAll requires exactly one argument".into(),
                        ));
                    }
                    let name = self
                        .eval_expr(&args[0], env, event_param, event)?
                        .as_string();
                    return Ok(Self::new_array_value(
                        entries
                            .iter()
                            .filter_map(|(entry_name, entry_value)| {
                                (entry_name == &name).then(|| Value::String(entry_value.clone()))
                            })
                            .collect::<Vec<_>>(),
                    ));
                }
                Some(Value::Object(entries)) => {
                    if !Self::is_url_search_params_object(&entries.borrow()) {
                        return Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not a FormData instance",
                            target
                        )));
                    }
                }
                Some(_) => {
                    return Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a FormData instance",
                        target
                    )));
                }
                None => {
                    return Err(Error::ScriptRuntime(format!(
                        "unknown FormData variable: {}",
                        target
                    )));
                }
            }
        }

        let object = self.resolve_url_search_params_object_from_env(env, target)?;
        match method {
            UrlSearchParamsInstanceMethod::Append => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.append requires exactly two arguments".into(),
                    ));
                }
                let name = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                let value = self
                    .eval_expr(&args[1], env, event_param, event)?
                    .as_string();
                {
                    let mut object_ref = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&object_ref);
                    pairs.push((name, value));
                    Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                }
                self.sync_url_search_params_owner(&object);
                Ok(Value::Undefined)
            }
            UrlSearchParamsInstanceMethod::Delete => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.delete requires one or two arguments".into(),
                    ));
                }
                let name = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                let value = if args.len() == 2 {
                    Some(
                        self.eval_expr(&args[1], env, event_param, event)?
                            .as_string(),
                    )
                } else {
                    None
                };
                {
                    let mut object_ref = object.borrow_mut();
                    let mut pairs = Self::url_search_params_pairs_from_object_entries(&object_ref);
                    pairs.retain(|(entry_name, entry_value)| {
                        if entry_name != &name {
                            return true;
                        }
                        if let Some(value) = value.as_ref() {
                            entry_value != value
                        } else {
                            false
                        }
                    });
                    Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                }
                self.sync_url_search_params_owner(&object);
                Ok(Value::Undefined)
            }
            UrlSearchParamsInstanceMethod::GetAll => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.getAll requires exactly one argument".into(),
                    ));
                }
                let name = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                Ok(Self::new_array_value(
                    pairs
                        .into_iter()
                        .filter_map(|(entry_name, entry_value)| {
                            (entry_name == name).then(|| Value::String(entry_value))
                        })
                        .collect::<Vec<_>>(),
                ))
            }
            UrlSearchParamsInstanceMethod::Has => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "URLSearchParams.has requires one or two arguments".into(),
                    ));
                }
                let name = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                let value = if args.len() == 2 {
                    Some(
                        self.eval_expr(&args[1], env, event_param, event)?
                            .as_string(),
                    )
                } else {
                    None
                };
                let pairs = Self::url_search_params_pairs_from_object_entries(&object.borrow());
                let has = pairs.into_iter().any(|(entry_name, entry_value)| {
                    if entry_name != name {
                        return false;
                    }
                    if let Some(value) = value.as_ref() {
                        &entry_value == value
                    } else {
                        true
                    }
                });
                Ok(Value::Bool(has))
            }
        }
    }

    pub(super) fn eval_map_construct(
        &mut self,
        iterable: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Map constructor must be called with new".into(),
            ));
        }

        let map = Rc::new(RefCell::new(MapValue {
            entries: Vec::new(),
            properties: Vec::new(),
        }));

        let Some(iterable) = iterable else {
            return Ok(Value::Map(map));
        };

        let iterable = self.eval_expr(iterable, env, event_param, event)?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Map(map));
        }

        match iterable {
            Value::Map(source) => {
                let source = source.borrow();
                map.borrow_mut().entries = source.entries.clone();
            }
            other => {
                let entries = self.array_like_values_from_value(&other)?;
                for entry in entries {
                    let pair = self.array_like_values_from_value(&entry).map_err(|_| {
                        Error::ScriptRuntime(
                            "Map constructor iterable values must be [key, value] pairs".into(),
                        )
                    })?;
                    if pair.len() < 2 {
                        return Err(Error::ScriptRuntime(
                            "Map constructor iterable values must be [key, value] pairs".into(),
                        ));
                    }
                    self.map_set_entry(&mut map.borrow_mut(), pair[0].clone(), pair[1].clone());
                }
            }
        }

        Ok(Value::Map(map))
    }

    pub(super) fn eval_map_static_method(
        &mut self,
        method: MapStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            MapStaticMethod::GroupBy => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.groupBy requires exactly two arguments".into(),
                    ));
                }
                let iterable = self.eval_expr(&args[0], env, event_param, event)?;
                let callback = self.eval_expr(&args[1], env, event_param, event)?;
                let values = self.array_like_values_from_value(&iterable)?;
                let map = Rc::new(RefCell::new(MapValue {
                    entries: Vec::new(),
                    properties: Vec::new(),
                }));
                for (index, item) in values.into_iter().enumerate() {
                    let group_key = self.execute_callback_value(
                        &callback,
                        &[item.clone(), Value::Number(index as i64)],
                        event,
                    )?;
                    let mut map_ref = map.borrow_mut();
                    if let Some(entry_index) = self.map_entry_index(&map_ref, &group_key) {
                        match &mut map_ref.entries[entry_index].1 {
                            Value::Array(group_values) => group_values.borrow_mut().push(item),
                            _ => {
                                map_ref.entries[entry_index].1 = Self::new_array_value(vec![item]);
                            }
                        }
                    } else {
                        map_ref
                            .entries
                            .push((group_key, Self::new_array_value(vec![item])));
                    }
                }
                Ok(Value::Map(map))
            }
        }
    }

    pub(super) fn eval_map_method(
        &mut self,
        target: &str,
        method: MapInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let target_value = env
            .get(target)
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {}", target)))?;

        if let Value::Set(set) = target_value {
            let set = set.clone();
            return match method {
                MapInstanceMethod::Has => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.has requires exactly one argument".into(),
                        ));
                    }
                    let key = self.eval_expr(&args[0], env, event_param, event)?;
                    Ok(Value::Bool(
                        self.set_value_index(&set.borrow(), &key).is_some(),
                    ))
                }
                MapInstanceMethod::Delete => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.delete requires exactly one argument".into(),
                        ));
                    }
                    let key = self.eval_expr(&args[0], env, event_param, event)?;
                    let mut set_ref = set.borrow_mut();
                    if let Some(index) = self.set_value_index(&set_ref, &key) {
                        set_ref.values.remove(index);
                        Ok(Value::Bool(true))
                    } else {
                        Ok(Value::Bool(false))
                    }
                }
                MapInstanceMethod::Clear => {
                    if !args.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "Map.clear does not take arguments".into(),
                        ));
                    }
                    set.borrow_mut().values.clear();
                    Ok(Value::Undefined)
                }
                MapInstanceMethod::ForEach => {
                    if args.is_empty() || args.len() > 2 {
                        return Err(Error::ScriptRuntime(
                            "Map.forEach requires a callback and optional thisArg".into(),
                        ));
                    }
                    let callback = self.eval_expr(&args[0], env, event_param, event)?;
                    if args.len() == 2 {
                        let _ = self.eval_expr(&args[1], env, event_param, event)?;
                    }
                    let snapshot = set.borrow().values.clone();
                    for value in snapshot {
                        let _ = self.execute_callback_value(
                            &callback,
                            &[value.clone(), value, Value::Set(set.clone())],
                            event,
                        )?;
                    }
                    Ok(Value::Undefined)
                }
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a Map",
                    target
                ))),
            };
        }

        if let Value::FormData(entries) = target_value {
            let entries = entries.clone();
            return match method {
                MapInstanceMethod::Get => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.get requires exactly one argument".into(),
                        ));
                    }
                    let key = self
                        .eval_expr(&args[0], env, event_param, event)?
                        .as_string();
                    let value = entries
                        .iter()
                        .find_map(|(entry_name, value)| (entry_name == &key).then(|| value.clone()))
                        .unwrap_or_default();
                    Ok(Value::String(value))
                }
                MapInstanceMethod::Has => {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "Map.has requires exactly one argument".into(),
                        ));
                    }
                    let key = self
                        .eval_expr(&args[0], env, event_param, event)?
                        .as_string();
                    let has = entries.iter().any(|(entry_name, _)| entry_name == &key);
                    Ok(Value::Bool(has))
                }
                _ => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a Map",
                    target
                ))),
            };
        }

        if let Value::Object(entries) = target_value {
            let entries = entries.clone();
            if Self::is_url_search_params_object(&entries.borrow()) {
                return match method {
                    MapInstanceMethod::Get => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.get requires exactly one argument".into(),
                            ));
                        }
                        let name = self
                            .eval_expr(&args[0], env, event_param, event)?
                            .as_string();
                        let pairs =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        let value = pairs
                            .into_iter()
                            .find_map(|(entry_name, entry_value)| {
                                (entry_name == name).then_some(entry_value)
                            })
                            .map(Value::String)
                            .unwrap_or(Value::Null);
                        Ok(value)
                    }
                    MapInstanceMethod::Has => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.has requires exactly one argument".into(),
                            ));
                        }
                        let name = self
                            .eval_expr(&args[0], env, event_param, event)?
                            .as_string();
                        let pairs =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        Ok(Value::Bool(
                            pairs.into_iter().any(|(entry_name, _)| entry_name == name),
                        ))
                    }
                    MapInstanceMethod::Delete => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.delete requires exactly one argument".into(),
                            ));
                        }
                        let name = self
                            .eval_expr(&args[0], env, event_param, event)?
                            .as_string();
                        {
                            let mut object_ref = entries.borrow_mut();
                            let mut pairs =
                                Self::url_search_params_pairs_from_object_entries(&object_ref);
                            pairs.retain(|(entry_name, _)| entry_name != &name);
                            Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                        }
                        self.sync_url_search_params_owner(&entries);
                        Ok(Value::Undefined)
                    }
                    MapInstanceMethod::ForEach => {
                        if args.is_empty() || args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "URLSearchParams.forEach requires a callback and optional thisArg"
                                    .into(),
                            ));
                        }
                        let callback = self.eval_expr(&args[0], env, event_param, event)?;
                        if args.len() == 2 {
                            let _ = self.eval_expr(&args[1], env, event_param, event)?;
                        }
                        let snapshot =
                            Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                        for (entry_name, entry_value) in snapshot {
                            let _ = self.execute_callback_value(
                                &callback,
                                &[
                                    Value::String(entry_value),
                                    Value::String(entry_name),
                                    Value::Object(entries.clone()),
                                ],
                                event,
                            )?;
                        }
                        Ok(Value::Undefined)
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a Map",
                        target
                    ))),
                };
            }
        }

        let Value::Map(map) = target_value else {
            if matches!(method, MapInstanceMethod::Get | MapInstanceMethod::Has) {
                return Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a FormData instance",
                    target
                )));
            }
            return Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a Map",
                target
            )));
        };
        let map = map.clone();
        match method {
            MapInstanceMethod::Get => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Map.get requires exactly one argument".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let map_ref = map.borrow();
                if let Some(index) = self.map_entry_index(&map_ref, &key) {
                    Ok(map_ref.entries[index].1.clone())
                } else {
                    Ok(Value::Undefined)
                }
            }
            MapInstanceMethod::Has => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Map.has requires exactly one argument".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let has = self.map_entry_index(&map.borrow(), &key).is_some();
                Ok(Value::Bool(has))
            }
            MapInstanceMethod::Delete => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Map.delete requires exactly one argument".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let mut map_ref = map.borrow_mut();
                if let Some(index) = self.map_entry_index(&map_ref, &key) {
                    map_ref.entries.remove(index);
                    Ok(Value::Bool(true))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            MapInstanceMethod::Clear => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Map.clear does not take arguments".into(),
                    ));
                }
                map.borrow_mut().entries.clear();
                Ok(Value::Undefined)
            }
            MapInstanceMethod::ForEach => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.forEach requires a callback and optional thisArg".into(),
                    ));
                }
                let callback = self.eval_expr(&args[0], env, event_param, event)?;
                if args.len() == 2 {
                    let _ = self.eval_expr(&args[1], env, event_param, event)?;
                }
                let snapshot = map.borrow().entries.clone();
                for (key, value) in snapshot {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[value, key, Value::Map(map.clone())],
                        event,
                    )?;
                }
                Ok(Value::Undefined)
            }
            MapInstanceMethod::GetOrInsert => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.getOrInsert requires exactly two arguments".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                let default_value = self.eval_expr(&args[1], env, event_param, event)?;
                let mut map_ref = map.borrow_mut();
                if let Some(index) = self.map_entry_index(&map_ref, &key) {
                    Ok(map_ref.entries[index].1.clone())
                } else {
                    map_ref.entries.push((key, default_value.clone()));
                    Ok(default_value)
                }
            }
            MapInstanceMethod::GetOrInsertComputed => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "Map.getOrInsertComputed requires exactly two arguments".into(),
                    ));
                }
                let key = self.eval_expr(&args[0], env, event_param, event)?;
                {
                    let map_ref = map.borrow();
                    if let Some(index) = self.map_entry_index(&map_ref, &key) {
                        return Ok(map_ref.entries[index].1.clone());
                    }
                }
                let callback = self.eval_expr(&args[1], env, event_param, event)?;
                let computed =
                    self.execute_callback_value(&callback, std::slice::from_ref(&key), event)?;
                map.borrow_mut().entries.push((key, computed.clone()));
                Ok(computed)
            }
        }
    }

    pub(super) fn eval_set_construct(
        &mut self,
        iterable: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Set constructor must be called with new".into(),
            ));
        }

        let set = Rc::new(RefCell::new(SetValue {
            values: Vec::new(),
            properties: Vec::new(),
        }));

        let Some(iterable) = iterable else {
            return Ok(Value::Set(set));
        };

        let iterable = self.eval_expr(iterable, env, event_param, event)?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Set(set));
        }

        let values = self.array_like_values_from_value(&iterable)?;
        for value in values {
            self.set_add_value(&mut set.borrow_mut(), value);
        }
        Ok(Value::Set(set))
    }

    pub(super) fn eval_set_method(
        &mut self,
        target: &str,
        method: SetInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let target_value = env
            .get(target)
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {}", target)))?;
        let Value::Set(set) = target_value else {
            return Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a Set",
                target
            )));
        };
        let set = set.clone();

        match method {
            SetInstanceMethod::Add => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.add requires exactly one argument".into(),
                    ));
                }
                let value = self.eval_expr(&args[0], env, event_param, event)?;
                self.set_add_value(&mut set.borrow_mut(), value);
                Ok(Value::Set(set))
            }
            SetInstanceMethod::Union => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.union requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let other_keys = self.set_like_keys_snapshot(&other)?;
                let mut out = SetValue {
                    values: set.borrow().values.clone(),
                    properties: Vec::new(),
                };
                for key in other_keys {
                    self.set_add_value(&mut out, key);
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::Intersection => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.intersection requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = set.borrow().values.clone();
                let mut out = SetValue {
                    values: Vec::new(),
                    properties: Vec::new(),
                };
                for value in snapshot {
                    if self.set_like_has_value(&other, &value)? {
                        self.set_add_value(&mut out, value);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::Difference => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.difference requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = set.borrow().values.clone();
                let mut out = SetValue {
                    values: Vec::new(),
                    properties: Vec::new(),
                };
                for value in snapshot {
                    if !self.set_like_has_value(&other, &value)? {
                        self.set_add_value(&mut out, value);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::SymmetricDifference => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.symmetricDifference requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                let other_keys = self.set_like_keys_snapshot(&other)?;
                let mut out = SetValue {
                    values: set.borrow().values.clone(),
                    properties: Vec::new(),
                };
                for key in other_keys {
                    if let Some(index) = self.set_value_index(&out, &key) {
                        out.values.remove(index);
                    } else {
                        out.values.push(key);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::IsDisjointFrom => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.isDisjointFrom requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                for value in &set.borrow().values {
                    if self.set_like_has_value(&other, value)? {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            SetInstanceMethod::IsSubsetOf => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.isSubsetOf requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                for value in &set.borrow().values {
                    if !self.set_like_has_value(&other, value)? {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            SetInstanceMethod::IsSupersetOf => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Set.isSupersetOf requires exactly one argument".into(),
                    ));
                }
                let other = self.eval_expr(&args[0], env, event_param, event)?;
                for value in self.set_like_keys_snapshot(&other)? {
                    if self.set_value_index(&set.borrow(), &value).is_none() {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
        }
    }

    pub(super) fn new_symbol_value(
        &mut self,
        description: Option<String>,
        registry_key: Option<String>,
    ) -> Value {
        let id = self.next_symbol_id;
        self.next_symbol_id = self.next_symbol_id.saturating_add(1);
        let symbol = Rc::new(SymbolValue {
            id,
            description,
            registry_key,
        });
        self.symbols_by_id.insert(id, symbol.clone());
        Value::Symbol(symbol)
    }

    pub(super) fn symbol_storage_key(id: usize) -> String {
        format!("{INTERNAL_SYMBOL_KEY_PREFIX}{id}")
    }

    pub(super) fn symbol_id_from_storage_key(key: &str) -> Option<usize> {
        key.strip_prefix(INTERNAL_SYMBOL_KEY_PREFIX)
            .and_then(|value| value.parse::<usize>().ok())
    }

    pub(super) fn is_symbol_storage_key(key: &str) -> bool {
        key.starts_with(INTERNAL_SYMBOL_KEY_PREFIX)
    }

    pub(super) fn is_internal_object_key(key: &str) -> bool {
        Self::is_symbol_storage_key(key)
            || key == INTERNAL_SYMBOL_WRAPPER_KEY
            || key == INTERNAL_STRING_WRAPPER_VALUE_KEY
            || key.starts_with(INTERNAL_INTL_KEY_PREFIX)
            || key.starts_with(INTERNAL_CALLABLE_KEY_PREFIX)
            || key.starts_with(INTERNAL_URL_SEARCH_PARAMS_KEY_PREFIX)
    }

    pub(super) fn symbol_wrapper_id_from_object(entries: &[(String, Value)]) -> Option<usize> {
        let value = Self::object_get_entry(entries, INTERNAL_SYMBOL_WRAPPER_KEY)?;
        match value {
            Value::Number(value) if value >= 0 => Some(value as usize),
            _ => None,
        }
    }

    pub(super) fn string_wrapper_value_from_object(entries: &[(String, Value)]) -> Option<String> {
        match Self::object_get_entry(entries, INTERNAL_STRING_WRAPPER_VALUE_KEY) {
            Some(Value::String(value)) => Some(value),
            _ => None,
        }
    }

    pub(super) fn symbol_id_from_property_key(&self, value: &Value) -> Option<usize> {
        match value {
            Value::Symbol(symbol) => Some(symbol.id),
            Value::Object(entries) => {
                let entries = entries.borrow();
                Self::symbol_wrapper_id_from_object(&entries)
            }
            _ => None,
        }
    }

    pub(super) fn property_key_to_storage_key(&self, value: &Value) -> String {
        if let Some(symbol_id) = self.symbol_id_from_property_key(value) {
            Self::symbol_storage_key(symbol_id)
        } else {
            value.as_string()
        }
    }

    pub(super) fn eval_symbol_construct(
        &mut self,
        description: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if called_with_new {
            return Err(Error::ScriptRuntime("Symbol is not a constructor".into()));
        }
        let description = if let Some(description) = description {
            let value = self.eval_expr(description, env, event_param, event)?;
            if matches!(value, Value::Undefined) {
                None
            } else {
                Some(value.as_string())
            }
        } else {
            None
        };
        Ok(self.new_symbol_value(description, None))
    }

    pub(super) fn eval_symbol_static_method(
        &mut self,
        method: SymbolStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            SymbolStaticMethod::For => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Symbol.for requires exactly one argument".into(),
                    ));
                }
                let key = self
                    .eval_expr(&args[0], env, event_param, event)?
                    .as_string();
                if let Some(symbol) = self.symbol_registry.get(&key) {
                    return Ok(Value::Symbol(symbol.clone()));
                }
                let symbol = match self.new_symbol_value(Some(key.clone()), Some(key.clone())) {
                    Value::Symbol(symbol) => symbol,
                    _ => unreachable!("new_symbol_value must create Symbol"),
                };
                self.symbol_registry.insert(key, symbol.clone());
                Ok(Value::Symbol(symbol))
            }
            SymbolStaticMethod::KeyFor => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Symbol.keyFor requires exactly one argument".into(),
                    ));
                }
                let symbol = self.eval_expr(&args[0], env, event_param, event)?;
                let Value::Symbol(symbol) = symbol else {
                    return Err(Error::ScriptRuntime(
                        "Symbol.keyFor argument must be a Symbol".into(),
                    ));
                };
                if let Some(key) = &symbol.registry_key {
                    Ok(Value::String(key.clone()))
                } else {
                    Ok(Value::Undefined)
                }
            }
        }
    }

    pub(super) fn symbol_static_property_name(property: SymbolStaticProperty) -> &'static str {
        match property {
            SymbolStaticProperty::AsyncDispose => "Symbol.asyncDispose",
            SymbolStaticProperty::AsyncIterator => "Symbol.asyncIterator",
            SymbolStaticProperty::Dispose => "Symbol.dispose",
            SymbolStaticProperty::HasInstance => "Symbol.hasInstance",
            SymbolStaticProperty::IsConcatSpreadable => "Symbol.isConcatSpreadable",
            SymbolStaticProperty::Iterator => "Symbol.iterator",
            SymbolStaticProperty::Match => "Symbol.match",
            SymbolStaticProperty::MatchAll => "Symbol.matchAll",
            SymbolStaticProperty::Replace => "Symbol.replace",
            SymbolStaticProperty::Search => "Symbol.search",
            SymbolStaticProperty::Species => "Symbol.species",
            SymbolStaticProperty::Split => "Symbol.split",
            SymbolStaticProperty::ToPrimitive => "Symbol.toPrimitive",
            SymbolStaticProperty::ToStringTag => "Symbol.toStringTag",
            SymbolStaticProperty::Unscopables => "Symbol.unscopables",
        }
    }

    pub(super) fn eval_symbol_static_property(&mut self, property: SymbolStaticProperty) -> Value {
        let name = Self::symbol_static_property_name(property).to_string();
        if let Some(symbol) = self.well_known_symbols.get(&name) {
            return Value::Symbol(symbol.clone());
        }
        let symbol = match self.new_symbol_value(Some(name.clone()), None) {
            Value::Symbol(symbol) => symbol,
            _ => unreachable!("new_symbol_value must create Symbol"),
        };
        self.well_known_symbols.insert(name, symbol.clone());
        Value::Symbol(symbol)
    }

    pub(super) fn eval_string_static_method(
        &mut self,
        method: StringStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            StringStaticMethod::FromCharCode => {
                let mut units = Vec::with_capacity(args.len());
                for arg in args {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    let unit = (Self::value_to_i64(&value) as i128).rem_euclid(1 << 16) as u16;
                    units.push(unit);
                }
                Ok(Value::String(String::from_utf16_lossy(&units)))
            }
            StringStaticMethod::FromCodePoint => {
                let mut out = String::new();
                for arg in args {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    let n = Self::coerce_number_for_global(&value);
                    if !n.is_finite() || n.fract() != 0.0 || !(0.0..=0x10_FFFF as f64).contains(&n)
                    {
                        return Err(Error::ScriptRuntime(
                            "Invalid code point for String.fromCodePoint".into(),
                        ));
                    }
                    let cp = n as u32;
                    if (0xD800..=0xDFFF).contains(&cp) {
                        return Err(Error::ScriptRuntime(
                            "Invalid code point for String.fromCodePoint".into(),
                        ));
                    }
                    let ch = char::from_u32(cp).ok_or_else(|| {
                        Error::ScriptRuntime("Invalid code point for String.fromCodePoint".into())
                    })?;
                    out.push(ch);
                }
                Ok(Value::String(out))
            }
            StringStaticMethod::Raw => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "String.raw requires at least one argument".into(),
                    ));
                }
                let template = self.eval_expr(&args[0], env, event_param, event)?;
                let raw = match template {
                    Value::Object(entries) => {
                        let entries = entries.borrow();
                        Self::object_get_entry(&entries, "raw").unwrap_or(Value::Undefined)
                    }
                    other => other,
                };
                let raw_segments = self.array_like_values_from_value(&raw)?;
                let mut substitutions = Vec::with_capacity(args.len().saturating_sub(1));
                for arg in args.iter().skip(1) {
                    substitutions.push(self.eval_expr(arg, env, event_param, event)?.as_string());
                }
                if raw_segments.is_empty() {
                    return Ok(Value::String(String::new()));
                }
                let mut out = String::new();
                for (idx, segment) in raw_segments.iter().enumerate() {
                    out.push_str(&segment.as_string());
                    if let Some(substitution) = substitutions.get(idx) {
                        out.push_str(substitution);
                    }
                }
                Ok(Value::String(out))
            }
        }
    }

    pub(super) fn eval_regexp_static_method(
        &mut self,
        method: RegExpStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            RegExpStaticMethod::Escape => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "RegExp.escape requires exactly one argument".into(),
                    ));
                }
                let value = self.eval_expr(&args[0], env, event_param, event)?;
                Ok(Value::String(regex::escape(&value.as_string())))
            }
        }
    }

    pub(super) fn eval_string_match(&mut self, value: &str, pattern: Value) -> Result<Value> {
        let regex = if let Value::RegExp(regex) = pattern {
            regex
        } else {
            let compiled = Self::new_regex_from_values(&pattern, None)?;
            match compiled {
                Value::RegExp(regex) => regex,
                _ => unreachable!("RegExp constructor must return a RegExp"),
            }
        };

        let global = regex.borrow().global;
        if global {
            let compiled = regex.borrow().compiled.clone();
            let matches = compiled
                .find_iter(value)
                .map(|m| Value::String(m.as_str().to_string()))
                .collect::<Vec<_>>();
            regex.borrow_mut().last_index = 0;
            if matches.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(Self::new_array_value(matches))
            }
        } else {
            let Some(captures) = Self::regex_exec(&regex, value)? else {
                return Ok(Value::Null);
            };
            Ok(Self::new_array_value(
                captures.into_iter().map(Value::String).collect::<Vec<_>>(),
            ))
        }
    }

    pub(super) fn promise_error_reason(err: Error) -> Value {
        Value::String(format!("{err}"))
    }

    pub(super) fn new_pending_promise(&mut self) -> Rc<RefCell<PromiseValue>> {
        let id = self.next_promise_id;
        self.next_promise_id = self.next_promise_id.saturating_add(1);
        Rc::new(RefCell::new(PromiseValue {
            id,
            state: PromiseState::Pending,
            reactions: Vec::new(),
        }))
    }

    pub(super) fn new_promise_capability_functions(
        &self,
        promise: Rc<RefCell<PromiseValue>>,
    ) -> (Value, Value) {
        let already_called = Rc::new(RefCell::new(false));
        let resolve = Value::PromiseCapability(Rc::new(PromiseCapabilityFunction {
            promise: promise.clone(),
            reject: false,
            already_called: already_called.clone(),
        }));
        let reject = Value::PromiseCapability(Rc::new(PromiseCapabilityFunction {
            promise,
            reject: true,
            already_called,
        }));
        (resolve, reject)
    }

    pub(super) fn promise_add_reaction(
        &mut self,
        promise: &Rc<RefCell<PromiseValue>>,
        kind: PromiseReactionKind,
    ) {
        let settled = {
            let mut promise_ref = promise.borrow_mut();
            match &promise_ref.state {
                PromiseState::Pending => {
                    promise_ref.reactions.push(PromiseReaction { kind });
                    return;
                }
                PromiseState::Fulfilled(value) => PromiseSettledValue::Fulfilled(value.clone()),
                PromiseState::Rejected(reason) => PromiseSettledValue::Rejected(reason.clone()),
            }
        };
        self.queue_promise_reaction_microtask(kind, settled);
    }

    pub(super) fn promise_fulfill(&mut self, promise: &Rc<RefCell<PromiseValue>>, value: Value) {
        let reactions = {
            let mut promise_ref = promise.borrow_mut();
            if !matches!(promise_ref.state, PromiseState::Pending) {
                return;
            }
            promise_ref.state = PromiseState::Fulfilled(value.clone());
            std::mem::take(&mut promise_ref.reactions)
        };
        for reaction in reactions {
            self.queue_promise_reaction_microtask(
                reaction.kind,
                PromiseSettledValue::Fulfilled(value.clone()),
            );
        }
    }

    pub(super) fn promise_reject(&mut self, promise: &Rc<RefCell<PromiseValue>>, reason: Value) {
        let reactions = {
            let mut promise_ref = promise.borrow_mut();
            if !matches!(promise_ref.state, PromiseState::Pending) {
                return;
            }
            promise_ref.state = PromiseState::Rejected(reason.clone());
            std::mem::take(&mut promise_ref.reactions)
        };
        for reaction in reactions {
            self.queue_promise_reaction_microtask(
                reaction.kind,
                PromiseSettledValue::Rejected(reason.clone()),
            );
        }
    }

    pub(super) fn promise_resolve(
        &mut self,
        promise: &Rc<RefCell<PromiseValue>>,
        value: Value,
    ) -> Result<()> {
        if !matches!(promise.borrow().state, PromiseState::Pending) {
            return Ok(());
        }

        if let Value::Promise(other) = &value {
            if Rc::ptr_eq(other, promise) {
                self.promise_reject(
                    promise,
                    Value::String("TypeError: Cannot resolve promise with itself".into()),
                );
                return Ok(());
            }

            let settled = {
                let other_ref = other.borrow();
                match &other_ref.state {
                    PromiseState::Pending => None,
                    PromiseState::Fulfilled(value) => {
                        Some(PromiseSettledValue::Fulfilled(value.clone()))
                    }
                    PromiseState::Rejected(reason) => {
                        Some(PromiseSettledValue::Rejected(reason.clone()))
                    }
                }
            };

            if let Some(settled) = settled {
                match settled {
                    PromiseSettledValue::Fulfilled(value) => self.promise_fulfill(promise, value),
                    PromiseSettledValue::Rejected(reason) => self.promise_reject(promise, reason),
                }
            } else {
                self.promise_add_reaction(
                    other,
                    PromiseReactionKind::ResolveTo {
                        target: promise.clone(),
                    },
                );
            }
            return Ok(());
        }

        if let Value::Object(entries) = &value {
            let then = {
                let entries = entries.borrow();
                Self::object_get_entry(&entries, "then")
            };

            if let Some(then) = then {
                if self.is_callable_value(&then) {
                    let (resolve, reject) = self.new_promise_capability_functions(promise.clone());
                    let event = EventState::new("microtask", self.dom.root, self.now_ms);
                    match self.execute_callable_value(&then, &[resolve, reject], &event) {
                        Ok(_) => {}
                        Err(err) => self.promise_reject(promise, Self::promise_error_reason(err)),
                    }
                    return Ok(());
                }
            }
        }

        self.promise_fulfill(promise, value);
        Ok(())
    }

    pub(super) fn promise_resolve_value_as_promise(
        &mut self,
        value: Value,
    ) -> Result<Rc<RefCell<PromiseValue>>> {
        if let Value::Promise(promise) = value {
            return Ok(promise);
        }
        let promise = self.new_pending_promise();
        self.promise_resolve(&promise, value)?;
        Ok(promise)
    }

    pub(super) fn promise_then_internal(
        &mut self,
        promise: &Rc<RefCell<PromiseValue>>,
        on_fulfilled: Option<Value>,
        on_rejected: Option<Value>,
    ) -> Rc<RefCell<PromiseValue>> {
        let result = self.new_pending_promise();
        self.promise_add_reaction(
            promise,
            PromiseReactionKind::Then {
                on_fulfilled,
                on_rejected,
                result: result.clone(),
            },
        );
        result
    }

    pub(super) fn eval_promise_construct(
        &mut self,
        executor: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Promise constructor must be called with new".into(),
            ));
        }
        let Some(executor) = executor else {
            return Err(Error::ScriptRuntime(
                "Promise constructor requires exactly one executor".into(),
            ));
        };
        let executor = self.eval_expr(executor, env, event_param, event)?;
        if !self.is_callable_value(&executor) {
            return Err(Error::ScriptRuntime(
                "Promise constructor executor must be a function".into(),
            ));
        }

        let promise = self.new_pending_promise();
        let (resolve, reject) = self.new_promise_capability_functions(promise.clone());
        if let Err(err) = self.execute_callable_value(&executor, &[resolve, reject], event) {
            self.promise_reject(&promise, Self::promise_error_reason(err));
        }
        Ok(Value::Promise(promise))
    }

    pub(super) fn eval_promise_static_method(
        &mut self,
        method: PromiseStaticMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match method {
            PromiseStaticMethod::Resolve => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.resolve supports zero or one argument".into(),
                    ));
                }
                let value = if let Some(value) = args.first() {
                    self.eval_expr(value, env, event_param, event)?
                } else {
                    Value::Undefined
                };
                if let Value::Promise(promise) = value {
                    return Ok(Value::Promise(promise));
                }
                let promise = self.new_pending_promise();
                self.promise_resolve(&promise, value)?;
                Ok(Value::Promise(promise))
            }
            PromiseStaticMethod::Reject => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.reject supports zero or one argument".into(),
                    ));
                }
                let reason = if let Some(reason) = args.first() {
                    self.eval_expr(reason, env, event_param, event)?
                } else {
                    Value::Undefined
                };
                let promise = self.new_pending_promise();
                self.promise_reject(&promise, reason);
                Ok(Value::Promise(promise))
            }
            PromiseStaticMethod::All => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.all requires exactly one argument".into(),
                    ));
                }
                let iterable = self.eval_expr(&args[0], env, event_param, event)?;
                self.eval_promise_all(iterable)
            }
            PromiseStaticMethod::AllSettled => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.allSettled requires exactly one argument".into(),
                    ));
                }
                let iterable = self.eval_expr(&args[0], env, event_param, event)?;
                self.eval_promise_all_settled(iterable)
            }
            PromiseStaticMethod::Any => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.any requires exactly one argument".into(),
                    ));
                }
                let iterable = self.eval_expr(&args[0], env, event_param, event)?;
                self.eval_promise_any(iterable)
            }
            PromiseStaticMethod::Race => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.race requires exactly one argument".into(),
                    ));
                }
                let iterable = self.eval_expr(&args[0], env, event_param, event)?;
                self.eval_promise_race(iterable)
            }
            PromiseStaticMethod::Try => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Promise.try requires at least one argument".into(),
                    ));
                }
                let callback = self.eval_expr(&args[0], env, event_param, event)?;
                let mut callback_args = Vec::with_capacity(args.len().saturating_sub(1));
                for arg in args.iter().skip(1) {
                    callback_args.push(self.eval_expr(arg, env, event_param, event)?);
                }
                let promise = self.new_pending_promise();
                match self.execute_callable_value(&callback, &callback_args, event) {
                    Ok(value) => {
                        self.promise_resolve(&promise, value)?;
                    }
                    Err(err) => {
                        self.promise_reject(&promise, Self::promise_error_reason(err));
                    }
                }
                Ok(Value::Promise(promise))
            }
            PromiseStaticMethod::WithResolvers => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Promise.withResolvers does not take arguments".into(),
                    ));
                }
                let promise = self.new_pending_promise();
                let (resolve, reject) = self.new_promise_capability_functions(promise.clone());
                Ok(Self::new_object_value(vec![
                    ("promise".into(), Value::Promise(promise)),
                    ("resolve".into(), resolve),
                    ("reject".into(), reject),
                ]))
            }
        }
    }

    pub(super) fn eval_promise_method(
        &mut self,
        target: &Expr,
        method: PromiseInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let target = self.eval_expr(target, env, event_param, event)?;
        let Value::Promise(promise) = target else {
            return Err(Error::ScriptRuntime(
                "Promise instance method target must be a Promise".into(),
            ));
        };

        match method {
            PromiseInstanceMethod::Then => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Promise.then supports up to two arguments".into(),
                    ));
                }
                let on_fulfilled = if let Some(arg) = args.first() {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    if self.is_callable_value(&value) {
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let on_rejected = if args.len() >= 2 {
                    let value = self.eval_expr(&args[1], env, event_param, event)?;
                    if self.is_callable_value(&value) {
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Value::Promise(self.promise_then_internal(
                    &promise,
                    on_fulfilled,
                    on_rejected,
                )))
            }
            PromiseInstanceMethod::Catch => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.catch supports at most one argument".into(),
                    ));
                }
                let on_rejected = if let Some(arg) = args.first() {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    if self.is_callable_value(&value) {
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Value::Promise(self.promise_then_internal(
                    &promise,
                    None,
                    on_rejected,
                )))
            }
            PromiseInstanceMethod::Finally => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "Promise.finally supports at most one argument".into(),
                    ));
                }
                let callback = if let Some(arg) = args.first() {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    if self.is_callable_value(&value) {
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let result = self.new_pending_promise();
                self.promise_add_reaction(
                    &promise,
                    PromiseReactionKind::Finally {
                        callback,
                        result: result.clone(),
                    },
                );
                Ok(Value::Promise(result))
            }
        }
    }

    pub(super) fn eval_promise_all(&mut self, iterable: Value) -> Result<Value> {
        let values = self.array_like_values_from_value(&iterable)?;
        let result = self.new_pending_promise();
        if values.is_empty() {
            self.promise_fulfill(&result, Self::new_array_value(Vec::new()));
            return Ok(Value::Promise(result));
        }

        let state = Rc::new(RefCell::new(PromiseAllState {
            result: result.clone(),
            remaining: values.len(),
            values: vec![None; values.len()],
            settled: false,
        }));

        for (index, value) in values.into_iter().enumerate() {
            let promise = self.promise_resolve_value_as_promise(value)?;
            self.promise_add_reaction(
                &promise,
                PromiseReactionKind::All {
                    state: state.clone(),
                    index,
                },
            );
        }

        Ok(Value::Promise(result))
    }

    pub(super) fn eval_promise_all_settled(&mut self, iterable: Value) -> Result<Value> {
        let values = self.array_like_values_from_value(&iterable)?;
        let result = self.new_pending_promise();
        if values.is_empty() {
            self.promise_fulfill(&result, Self::new_array_value(Vec::new()));
            return Ok(Value::Promise(result));
        }

        let state = Rc::new(RefCell::new(PromiseAllSettledState {
            result: result.clone(),
            remaining: values.len(),
            values: vec![None; values.len()],
        }));

        for (index, value) in values.into_iter().enumerate() {
            let promise = self.promise_resolve_value_as_promise(value)?;
            self.promise_add_reaction(
                &promise,
                PromiseReactionKind::AllSettled {
                    state: state.clone(),
                    index,
                },
            );
        }

        Ok(Value::Promise(result))
    }

    pub(super) fn eval_promise_any(&mut self, iterable: Value) -> Result<Value> {
        let values = self.array_like_values_from_value(&iterable)?;
        let result = self.new_pending_promise();
        if values.is_empty() {
            self.promise_reject(&result, Self::new_aggregate_error_value(Vec::new()));
            return Ok(Value::Promise(result));
        }

        let state = Rc::new(RefCell::new(PromiseAnyState {
            result: result.clone(),
            remaining: values.len(),
            reasons: vec![None; values.len()],
            settled: false,
        }));

        for (index, value) in values.into_iter().enumerate() {
            let promise = self.promise_resolve_value_as_promise(value)?;
            self.promise_add_reaction(
                &promise,
                PromiseReactionKind::Any {
                    state: state.clone(),
                    index,
                },
            );
        }

        Ok(Value::Promise(result))
    }

    pub(super) fn eval_promise_race(&mut self, iterable: Value) -> Result<Value> {
        let values = self.array_like_values_from_value(&iterable)?;
        let result = self.new_pending_promise();
        if values.is_empty() {
            return Ok(Value::Promise(result));
        }

        let state = Rc::new(RefCell::new(PromiseRaceState {
            result: result.clone(),
            settled: false,
        }));

        for value in values {
            let promise = self.promise_resolve_value_as_promise(value)?;
            self.promise_add_reaction(
                &promise,
                PromiseReactionKind::Race {
                    state: state.clone(),
                },
            );
        }

        Ok(Value::Promise(result))
    }

    pub(super) fn new_aggregate_error_value(reasons: Vec<Value>) -> Value {
        Self::new_object_value(vec![
            ("name".into(), Value::String("AggregateError".into())),
            (
                "message".into(),
                Value::String("All promises were rejected".into()),
            ),
            ("errors".into(), Self::new_array_value(reasons)),
        ])
    }

    pub(super) fn run_promise_reaction_task(
        &mut self,
        reaction: PromiseReactionKind,
        settled: PromiseSettledValue,
    ) -> Result<()> {
        let event = EventState::new("microtask", self.dom.root, self.now_ms);
        match reaction {
            PromiseReactionKind::Then {
                on_fulfilled,
                on_rejected,
                result,
            } => match settled {
                PromiseSettledValue::Fulfilled(value) => {
                    if let Some(callback) = on_fulfilled {
                        match self.execute_callable_value(
                            &callback,
                            std::slice::from_ref(&value),
                            &event,
                        ) {
                            Ok(next) => self.promise_resolve(&result, next)?,
                            Err(err) => {
                                self.promise_reject(&result, Self::promise_error_reason(err))
                            }
                        }
                    } else {
                        self.promise_fulfill(&result, value);
                    }
                }
                PromiseSettledValue::Rejected(reason) => {
                    if let Some(callback) = on_rejected {
                        match self.execute_callable_value(
                            &callback,
                            std::slice::from_ref(&reason),
                            &event,
                        ) {
                            Ok(next) => self.promise_resolve(&result, next)?,
                            Err(err) => {
                                self.promise_reject(&result, Self::promise_error_reason(err))
                            }
                        }
                    } else {
                        self.promise_reject(&result, reason);
                    }
                }
            },
            PromiseReactionKind::Finally { callback, result } => {
                if let Some(callback) = callback {
                    match self.execute_callable_value(&callback, &[], &event) {
                        Ok(next) => {
                            let continuation = self.promise_resolve_value_as_promise(next)?;
                            self.promise_add_reaction(
                                &continuation,
                                PromiseReactionKind::FinallyContinuation {
                                    original: settled,
                                    result,
                                },
                            );
                        }
                        Err(err) => self.promise_reject(&result, Self::promise_error_reason(err)),
                    }
                } else {
                    match settled {
                        PromiseSettledValue::Fulfilled(value) => {
                            self.promise_fulfill(&result, value)
                        }
                        PromiseSettledValue::Rejected(reason) => {
                            self.promise_reject(&result, reason)
                        }
                    }
                }
            }
            PromiseReactionKind::FinallyContinuation { original, result } => match settled {
                PromiseSettledValue::Fulfilled(_) => match original {
                    PromiseSettledValue::Fulfilled(value) => self.promise_fulfill(&result, value),
                    PromiseSettledValue::Rejected(reason) => self.promise_reject(&result, reason),
                },
                PromiseSettledValue::Rejected(reason) => self.promise_reject(&result, reason),
            },
            PromiseReactionKind::ResolveTo { target } => match settled {
                PromiseSettledValue::Fulfilled(value) => self.promise_resolve(&target, value)?,
                PromiseSettledValue::Rejected(reason) => self.promise_reject(&target, reason),
            },
            PromiseReactionKind::All { state, index } => {
                let mut state_ref = state.borrow_mut();
                if state_ref.settled {
                    return Ok(());
                }
                match settled {
                    PromiseSettledValue::Fulfilled(value) => {
                        if state_ref.values[index].is_none() {
                            state_ref.values[index] = Some(value);
                            state_ref.remaining = state_ref.remaining.saturating_sub(1);
                        }
                        if state_ref.remaining == 0 {
                            state_ref.settled = true;
                            let result = state_ref.result.clone();
                            let values = state_ref
                                .values
                                .iter()
                                .map(|value| value.clone().unwrap_or(Value::Undefined))
                                .collect::<Vec<_>>();
                            drop(state_ref);
                            self.promise_fulfill(&result, Self::new_array_value(values));
                        }
                    }
                    PromiseSettledValue::Rejected(reason) => {
                        state_ref.settled = true;
                        let result = state_ref.result.clone();
                        drop(state_ref);
                        self.promise_reject(&result, reason);
                    }
                }
            }
            PromiseReactionKind::AllSettled { state, index } => {
                let mut state_ref = state.borrow_mut();
                if state_ref.remaining == 0 {
                    return Ok(());
                }
                if state_ref.values[index].is_none() {
                    let entry = match settled {
                        PromiseSettledValue::Fulfilled(value) => Self::new_object_value(vec![
                            ("status".into(), Value::String("fulfilled".into())),
                            ("value".into(), value),
                        ]),
                        PromiseSettledValue::Rejected(reason) => Self::new_object_value(vec![
                            ("status".into(), Value::String("rejected".into())),
                            ("reason".into(), reason),
                        ]),
                    };
                    state_ref.values[index] = Some(entry);
                    state_ref.remaining = state_ref.remaining.saturating_sub(1);
                }
                if state_ref.remaining == 0 {
                    let result = state_ref.result.clone();
                    let values = state_ref
                        .values
                        .iter()
                        .map(|value| value.clone().unwrap_or(Value::Undefined))
                        .collect::<Vec<_>>();
                    drop(state_ref);
                    self.promise_fulfill(&result, Self::new_array_value(values));
                }
            }
            PromiseReactionKind::Any { state, index } => {
                let mut state_ref = state.borrow_mut();
                if state_ref.settled {
                    return Ok(());
                }
                match settled {
                    PromiseSettledValue::Fulfilled(value) => {
                        state_ref.settled = true;
                        let result = state_ref.result.clone();
                        drop(state_ref);
                        self.promise_fulfill(&result, value);
                    }
                    PromiseSettledValue::Rejected(reason) => {
                        if state_ref.reasons[index].is_none() {
                            state_ref.reasons[index] = Some(reason);
                            state_ref.remaining = state_ref.remaining.saturating_sub(1);
                        }
                        if state_ref.remaining == 0 {
                            state_ref.settled = true;
                            let result = state_ref.result.clone();
                            let reasons = state_ref
                                .reasons
                                .iter()
                                .map(|reason| reason.clone().unwrap_or(Value::Undefined))
                                .collect::<Vec<_>>();
                            drop(state_ref);
                            self.promise_reject(&result, Self::new_aggregate_error_value(reasons));
                        }
                    }
                }
            }
            PromiseReactionKind::Race { state } => {
                let mut state_ref = state.borrow_mut();
                if state_ref.settled {
                    return Ok(());
                }
                state_ref.settled = true;
                let result = state_ref.result.clone();
                drop(state_ref);
                match settled {
                    PromiseSettledValue::Fulfilled(value) => self.promise_fulfill(&result, value),
                    PromiseSettledValue::Rejected(reason) => self.promise_reject(&result, reason),
                }
            }
        }
        Ok(())
    }

    pub(super) fn execute_callback_value(
        &mut self,
        callback: &Value,
        args: &[Value],
        event: &EventState,
    ) -> Result<Value> {
        self.execute_callable_value(callback, args, event)
    }

    pub(super) fn eval_typed_array_method(
        &mut self,
        target: &str,
        method: TypedArrayInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !matches!(env.get(target), Some(Value::TypedArray(_))) {
            let Some(target_value) = env.get(target) else {
                return Err(Error::ScriptRuntime(format!(
                    "unknown variable: {}",
                    target
                )));
            };

            if let Value::Map(map) = target_value {
                return match method {
                    TypedArrayInstanceMethod::Set => {
                        if args.len() != 2 {
                            return Err(Error::ScriptRuntime(
                                "Map.set requires exactly two arguments".into(),
                            ));
                        }
                        let key = self.eval_expr(&args[0], env, event_param, event)?;
                        let value = self.eval_expr(&args[1], env, event_param, event)?;
                        self.map_set_entry(&mut map.borrow_mut(), key, value);
                        Ok(Value::Map(map.clone()))
                    }
                    TypedArrayInstanceMethod::Entries => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Map.entries does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(self.map_entries_array(map)))
                    }
                    TypedArrayInstanceMethod::Keys => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Map.keys does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(
                            map.borrow()
                                .entries
                                .iter()
                                .map(|(key, _)| key.clone())
                                .collect(),
                        ))
                    }
                    TypedArrayInstanceMethod::Values => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Map.values does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(
                            map.borrow()
                                .entries
                                .iter()
                                .map(|(_, value)| value.clone())
                                .collect(),
                        ))
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a TypedArray",
                        target
                    ))),
                };
            }

            if let Value::Set(set) = target_value {
                return match method {
                    TypedArrayInstanceMethod::Entries => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Set.entries does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(self.set_entries_array(set)))
                    }
                    TypedArrayInstanceMethod::Keys | TypedArrayInstanceMethod::Values => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(
                                "Set.keys/values does not take arguments".into(),
                            ));
                        }
                        Ok(Self::new_array_value(self.set_values_array(set)))
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a TypedArray",
                        target
                    ))),
                };
            }

            if let Value::Object(entries) = target_value {
                if Self::is_url_search_params_object(&entries.borrow()) {
                    return match method {
                        TypedArrayInstanceMethod::Set => {
                            if args.len() != 2 {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.set requires exactly two arguments".into(),
                                ));
                            }
                            let name = self
                                .eval_expr(&args[0], env, event_param, event)?
                                .as_string();
                            let value = self
                                .eval_expr(&args[1], env, event_param, event)?
                                .as_string();
                            {
                                let mut object_ref = entries.borrow_mut();
                                let mut pairs =
                                    Self::url_search_params_pairs_from_object_entries(&object_ref);
                                if let Some(first_match) =
                                    pairs.iter().position(|(entry_name, _)| entry_name == &name)
                                {
                                    pairs[first_match].1 = value;
                                    let mut index = pairs.len();
                                    while index > 0 {
                                        index -= 1;
                                        if index != first_match && pairs[index].0 == name {
                                            pairs.remove(index);
                                        }
                                    }
                                } else {
                                    pairs.push((name, value));
                                }
                                Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                            }
                            self.sync_url_search_params_owner(entries);
                            Ok(Value::Undefined)
                        }
                        TypedArrayInstanceMethod::Entries => {
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.entries does not take arguments".into(),
                                ));
                            }
                            let pairs = Self::url_search_params_pairs_from_object_entries(
                                &entries.borrow(),
                            );
                            Ok(Self::new_array_value(
                                pairs
                                    .into_iter()
                                    .map(|(name, value)| {
                                        Self::new_array_value(vec![
                                            Value::String(name),
                                            Value::String(value),
                                        ])
                                    })
                                    .collect::<Vec<_>>(),
                            ))
                        }
                        TypedArrayInstanceMethod::Keys => {
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.keys does not take arguments".into(),
                                ));
                            }
                            let pairs = Self::url_search_params_pairs_from_object_entries(
                                &entries.borrow(),
                            );
                            Ok(Self::new_array_value(
                                pairs
                                    .into_iter()
                                    .map(|(name, _)| Value::String(name))
                                    .collect::<Vec<_>>(),
                            ))
                        }
                        TypedArrayInstanceMethod::Values => {
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.values does not take arguments".into(),
                                ));
                            }
                            let pairs = Self::url_search_params_pairs_from_object_entries(
                                &entries.borrow(),
                            );
                            Ok(Self::new_array_value(
                                pairs
                                    .into_iter()
                                    .map(|(_, value)| Value::String(value))
                                    .collect::<Vec<_>>(),
                            ))
                        }
                        TypedArrayInstanceMethod::Sort => {
                            if !args.is_empty() {
                                return Err(Error::ScriptRuntime(
                                    "URLSearchParams.sort does not take arguments".into(),
                                ));
                            }
                            {
                                let mut object_ref = entries.borrow_mut();
                                let mut pairs =
                                    Self::url_search_params_pairs_from_object_entries(&object_ref);
                                pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
                                Self::set_url_search_params_pairs(&mut object_ref, &pairs);
                            }
                            self.sync_url_search_params_owner(entries);
                            Ok(Value::Undefined)
                        }
                        _ => Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not a TypedArray",
                            target
                        ))),
                    };
                }
            }

            if matches!(method, TypedArrayInstanceMethod::At) {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "at supports zero or one argument".into(),
                    ));
                }
                let index = if let Some(index) = args.first() {
                    Self::value_to_i64(&self.eval_expr(index, env, event_param, event)?)
                } else {
                    0
                };

                return match target_value {
                    Value::String(value) => {
                        let len = value.chars().count() as i64;
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
                    Value::Object(entries) => {
                        let entries = entries.borrow();
                        if let Some(value) = Self::string_wrapper_value_from_object(&entries) {
                            let len = value.chars().count() as i64;
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
                        } else {
                            Err(Error::ScriptRuntime(format!(
                                "variable '{}' is not a TypedArray",
                                target
                            )))
                        }
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a TypedArray",
                        target
                    ))),
                };
            }

            if matches!(
                method,
                TypedArrayInstanceMethod::IndexOf | TypedArrayInstanceMethod::LastIndexOf
            ) {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "indexOf requires one or two arguments".into(),
                    ));
                }
                let search = self.eval_expr(&args[0], env, event_param, event)?;

                return match target_value {
                    Value::String(value) => {
                        let len = value.chars().count() as i64;
                        if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                            let mut start = if args.len() == 2 {
                                Self::value_to_i64(&self.eval_expr(
                                    &args[1],
                                    env,
                                    event_param,
                                    event,
                                )?)
                            } else {
                                0
                            };
                            if start < 0 {
                                start = 0;
                            }
                            if start > len {
                                start = len;
                            }
                            let index =
                                Self::string_index_of(value, &search.as_string(), start as usize)
                                    .map(|idx| idx as i64)
                                    .unwrap_or(-1);
                            Ok(Value::Number(index))
                        } else {
                            let mut from = if args.len() == 2 {
                                Self::value_to_i64(&self.eval_expr(
                                    &args[1],
                                    env,
                                    event_param,
                                    event,
                                )?)
                            } else {
                                len
                            };
                            if from < 0 {
                                from = 0;
                            }
                            if from > len {
                                from = len;
                            }
                            let from = from as usize;
                            let search = search.as_string();
                            if search.is_empty() {
                                return Ok(Value::Number(from as i64));
                            }
                            for idx in (0..=from).rev() {
                                let byte_idx = Self::char_index_to_byte(value, idx);
                                if value[byte_idx..].starts_with(&search) {
                                    return Ok(Value::Number(idx as i64));
                                }
                            }
                            Ok(Value::Number(-1))
                        }
                    }
                    Value::Array(values) => {
                        let from = if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                            let len = values.borrow().len() as i64;
                            let mut from = if args.len() == 2 {
                                Self::value_to_i64(&self.eval_expr(
                                    &args[1],
                                    env,
                                    event_param,
                                    event,
                                )?)
                            } else {
                                0
                            };
                            if from < 0 {
                                from = (len + from).max(0);
                            }
                            if from > len {
                                from = len;
                            }
                            from
                        } else {
                            let len = values.borrow().len() as i64;
                            let from = if args.len() == 2 {
                                Self::value_to_i64(&self.eval_expr(
                                    &args[1],
                                    env,
                                    event_param,
                                    event,
                                )?)
                            } else {
                                len - 1
                            };
                            if from < 0 {
                                (len + from).max(-1)
                            } else {
                                from.min(len - 1)
                            }
                        };

                        let values = values.borrow();
                        if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                            for (index, value) in values.iter().enumerate().skip(from as usize) {
                                if self.strict_equal(value, &search) {
                                    return Ok(Value::Number(index as i64));
                                }
                            }
                            Ok(Value::Number(-1))
                        } else {
                            if from < 0 {
                                return Ok(Value::Number(-1));
                            }
                            for index in (0..=from as usize).rev() {
                                if self.strict_equal(&values[index], &search) {
                                    return Ok(Value::Number(index as i64));
                                }
                            }
                            Ok(Value::Number(-1))
                        }
                    }
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a TypedArray",
                        target
                    ))),
                };
            }

            return Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a TypedArray",
                target
            )));
        }

        let array = self.resolve_typed_array_from_env(env, target)?;
        if array.borrow().buffer.borrow().detached {
            return Err(Error::ScriptRuntime(
                "Cannot perform TypedArray method on a detached ArrayBuffer".into(),
            ));
        }
        let kind = array.borrow().kind;
        let len = array.borrow().observed_length();
        let this_value = Value::TypedArray(array.clone());

        match method {
            TypedArrayInstanceMethod::At => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.at requires exactly one argument".into(),
                    ));
                }
                let index = self.eval_expr(&args[0], env, event_param, event)?;
                let mut index = Self::value_to_i64(&index);
                let len_i64 = len as i64;
                if index < 0 {
                    index += len_i64;
                }
                if index < 0 || index >= len_i64 {
                    return Ok(Value::Undefined);
                }
                self.typed_array_get_index(&array, index as usize)
            }
            TypedArrayInstanceMethod::CopyWithin => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.copyWithin requires 2 or 3 arguments".into(),
                    ));
                }
                let target_index =
                    Self::value_to_i64(&self.eval_expr(&args[0], env, event_param, event)?);
                let start_index =
                    Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?);
                let end_index = if args.len() == 3 {
                    Self::value_to_i64(&self.eval_expr(&args[2], env, event_param, event)?)
                } else {
                    len as i64
                };
                let target_index = Self::normalize_slice_index(len, target_index);
                let start_index = Self::normalize_slice_index(len, start_index);
                let end_index = Self::normalize_slice_index(len, end_index);
                let end_index = end_index.max(start_index);
                let count = end_index
                    .saturating_sub(start_index)
                    .min(len.saturating_sub(target_index));
                let snapshot = self.typed_array_snapshot(&array)?;
                for offset in 0..count {
                    self.typed_array_set_index(
                        &array,
                        target_index + offset,
                        snapshot[start_index + offset].clone(),
                    )?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::Entries => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.entries does not take arguments".into(),
                    ));
                }
                let snapshot = self.typed_array_snapshot(&array)?;
                let out = snapshot
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| {
                        Self::new_array_value(vec![Value::Number(index as i64), value])
                    })
                    .collect::<Vec<_>>();
                Ok(Self::new_array_value(out))
            }
            TypedArrayInstanceMethod::Fill => {
                if args.is_empty() || args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.fill requires 1 to 3 arguments".into(),
                    ));
                }
                let value = self.eval_expr(&args[0], env, event_param, event)?;
                let start = if args.len() >= 2 {
                    Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                } else {
                    0
                };
                let end = if args.len() == 3 {
                    Self::value_to_i64(&self.eval_expr(&args[2], env, event_param, event)?)
                } else {
                    len as i64
                };
                let start = Self::normalize_slice_index(len, start);
                let end = Self::normalize_slice_index(len, end).max(start);
                for index in start..end {
                    self.typed_array_set_index(&array, index, value.clone())?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::FindIndex
            | TypedArrayInstanceMethod::FindLast
            | TypedArrayInstanceMethod::FindLastIndex => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray find callback methods require exactly one argument".into(),
                    ));
                }
                let callback = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = self.typed_array_snapshot(&array)?;
                let iter: Box<dyn Iterator<Item = (usize, Value)>> = match method {
                    TypedArrayInstanceMethod::FindLast
                    | TypedArrayInstanceMethod::FindLastIndex => {
                        Box::new(snapshot.into_iter().enumerate().rev())
                    }
                    _ => Box::new(snapshot.into_iter().enumerate()),
                };
                for (index, value) in iter {
                    let matched = self.execute_callback_value(
                        &callback,
                        &[
                            value.clone(),
                            Value::Number(index as i64),
                            this_value.clone(),
                        ],
                        event,
                    )?;
                    if matched.truthy() {
                        return if matches!(
                            method,
                            TypedArrayInstanceMethod::FindLastIndex
                                | TypedArrayInstanceMethod::FindIndex
                        ) {
                            Ok(Value::Number(index as i64))
                        } else {
                            Ok(value)
                        };
                    }
                }
                if matches!(
                    method,
                    TypedArrayInstanceMethod::FindLastIndex | TypedArrayInstanceMethod::FindIndex
                ) {
                    Ok(Value::Number(-1))
                } else {
                    Ok(Value::Undefined)
                }
            }
            TypedArrayInstanceMethod::IndexOf | TypedArrayInstanceMethod::LastIndexOf => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray indexOf methods require one or two arguments".into(),
                    ));
                }
                let search = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = self.typed_array_snapshot(&array)?;
                if matches!(method, TypedArrayInstanceMethod::IndexOf) {
                    let from = if args.len() == 2 {
                        Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                    } else {
                        0
                    };
                    let mut from = if from < 0 {
                        (len as i64 + from).max(0)
                    } else {
                        from
                    };
                    if from > len as i64 {
                        from = len as i64;
                    }
                    for (index, value) in snapshot.iter().enumerate().skip(from as usize) {
                        if self.strict_equal(value, &search) {
                            return Ok(Value::Number(index as i64));
                        }
                    }
                    Ok(Value::Number(-1))
                } else {
                    let from = if args.len() == 2 {
                        Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                    } else {
                        (len as i64) - 1
                    };
                    let from = if from < 0 {
                        (len as i64 + from).max(-1)
                    } else {
                        from.min((len as i64) - 1)
                    };
                    if from < 0 {
                        return Ok(Value::Number(-1));
                    }
                    for index in (0..=from as usize).rev() {
                        if self.strict_equal(&snapshot[index], &search) {
                            return Ok(Value::Number(index as i64));
                        }
                    }
                    Ok(Value::Number(-1))
                }
            }
            TypedArrayInstanceMethod::Keys => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.keys does not take arguments".into(),
                    ));
                }
                Ok(Self::new_array_value(
                    (0..len).map(|index| Value::Number(index as i64)).collect(),
                ))
            }
            TypedArrayInstanceMethod::ReduceRight => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.reduceRight requires callback and optional initial value"
                            .into(),
                    ));
                }
                let callback = self.eval_expr(&args[0], env, event_param, event)?;
                let snapshot = self.typed_array_snapshot(&array)?;
                let mut iter = snapshot.into_iter().enumerate().rev();
                let mut acc = if args.len() == 2 {
                    self.eval_expr(&args[1], env, event_param, event)?
                } else {
                    let Some((_, first)) = iter.next() else {
                        return Err(Error::ScriptRuntime(
                            "reduce of empty array with no initial value".into(),
                        ));
                    };
                    first
                };
                for (index, value) in iter {
                    acc = self.execute_callback_value(
                        &callback,
                        &[acc, value, Value::Number(index as i64), this_value.clone()],
                        event,
                    )?;
                }
                Ok(acc)
            }
            TypedArrayInstanceMethod::Reverse => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.reverse does not take arguments".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                snapshot.reverse();
                for (index, value) in snapshot.into_iter().enumerate() {
                    self.typed_array_set_index(&array, index, value)?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::Set => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.set requires source and optional offset".into(),
                    ));
                }
                let source = self.eval_expr(&args[0], env, event_param, event)?;
                let source_values = self.array_like_values_from_value(&source)?;
                let offset = if args.len() == 2 {
                    Self::to_non_negative_usize(
                        &self.eval_expr(&args[1], env, event_param, event)?,
                        "TypedArray.set offset",
                    )?
                } else {
                    0
                };
                if offset > len || source_values.len() > len.saturating_sub(offset) {
                    return Err(Error::ScriptRuntime(
                        "source array is too large for target TypedArray".into(),
                    ));
                }
                for (index, value) in source_values.into_iter().enumerate() {
                    self.typed_array_set_index(&array, offset + index, value)?;
                }
                Ok(Value::Undefined)
            }
            TypedArrayInstanceMethod::Sort => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.sort supports at most one argument".into(),
                    ));
                }
                if args.len() == 1 {
                    return Err(Error::ScriptRuntime(
                        "custom comparator for TypedArray.sort is not supported".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                if kind.is_bigint() {
                    snapshot.sort_by(|left, right| {
                        let left = match left {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        let right = match right {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        left.cmp(&right)
                    });
                } else {
                    snapshot.sort_by(|left, right| {
                        let left = Self::coerce_number_for_global(left);
                        let right = Self::coerce_number_for_global(right);
                        match (left.is_nan(), right.is_nan()) {
                            (true, true) => std::cmp::Ordering::Equal,
                            (true, false) => std::cmp::Ordering::Greater,
                            (false, true) => std::cmp::Ordering::Less,
                            (false, false) => left
                                .partial_cmp(&right)
                                .unwrap_or(std::cmp::Ordering::Equal),
                        }
                    });
                }
                for (index, value) in snapshot.into_iter().enumerate() {
                    self.typed_array_set_index(&array, index, value)?;
                }
                Ok(this_value)
            }
            TypedArrayInstanceMethod::Subarray => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.subarray supports at most two arguments".into(),
                    ));
                }
                let begin = if !args.is_empty() {
                    Self::value_to_i64(&self.eval_expr(&args[0], env, event_param, event)?)
                } else {
                    0
                };
                let end = if args.len() == 2 {
                    Self::value_to_i64(&self.eval_expr(&args[1], env, event_param, event)?)
                } else {
                    len as i64
                };
                let begin = Self::normalize_slice_index(len, begin);
                let end = Self::normalize_slice_index(len, end).max(begin);
                let byte_offset = array
                    .borrow()
                    .byte_offset
                    .saturating_add(begin.saturating_mul(kind.bytes_per_element()));
                self.new_typed_array_view(
                    kind,
                    array.borrow().buffer.clone(),
                    byte_offset,
                    Some(end.saturating_sub(begin)),
                )
            }
            TypedArrayInstanceMethod::ToReversed => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.toReversed does not take arguments".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                snapshot.reverse();
                self.new_typed_array_from_values(kind, &snapshot)
            }
            TypedArrayInstanceMethod::ToSorted => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.toSorted supports at most one argument".into(),
                    ));
                }
                if args.len() == 1 {
                    return Err(Error::ScriptRuntime(
                        "custom comparator for TypedArray.toSorted is not supported".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                if kind.is_bigint() {
                    snapshot.sort_by(|left, right| {
                        let left = match left {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        let right = match right {
                            Value::BigInt(value) => value.clone(),
                            _ => JsBigInt::zero(),
                        };
                        left.cmp(&right)
                    });
                } else {
                    snapshot.sort_by(|left, right| {
                        let left = Self::coerce_number_for_global(left);
                        let right = Self::coerce_number_for_global(right);
                        match (left.is_nan(), right.is_nan()) {
                            (true, true) => std::cmp::Ordering::Equal,
                            (true, false) => std::cmp::Ordering::Greater,
                            (false, true) => std::cmp::Ordering::Less,
                            (false, false) => left
                                .partial_cmp(&right)
                                .unwrap_or(std::cmp::Ordering::Equal),
                        }
                    });
                }
                self.new_typed_array_from_values(kind, &snapshot)
            }
            TypedArrayInstanceMethod::Values => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.values does not take arguments".into(),
                    ));
                }
                Ok(Self::new_array_value(self.typed_array_snapshot(&array)?))
            }
            TypedArrayInstanceMethod::With => {
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.with requires exactly two arguments".into(),
                    ));
                }
                let index =
                    Self::value_to_i64(&self.eval_expr(&args[0], env, event_param, event)?);
                let value = self.eval_expr(&args[1], env, event_param, event)?;
                let index = if index < 0 {
                    (len as i64) + index
                } else {
                    index
                };
                if index < 0 || index >= len as i64 {
                    return Err(Error::ScriptRuntime(
                        "TypedArray.with index out of range".into(),
                    ));
                }
                let mut snapshot = self.typed_array_snapshot(&array)?;
                snapshot[index as usize] = value;
                self.new_typed_array_from_values(kind, &snapshot)
            }
        }
    }

    pub(super) fn execute_array_callback(
        &mut self,
        callback: &ScriptHandler,
        args: &[Value],
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Value> {
        let mut callback_env = env.clone();
        callback_env.remove(INTERNAL_RETURN_SLOT);
        let mut callback_event = event.clone();
        let event_param = None;
        self.bind_handler_params(
            callback,
            args,
            &mut callback_env,
            &event_param,
            &callback_event,
        )?;
        match self.execute_stmts(
            &callback.stmts,
            &event_param,
            &mut callback_event,
            &mut callback_env,
        )? {
            ExecFlow::Continue | ExecFlow::Return => {}
            ExecFlow::Break => {
                return Err(Error::ScriptRuntime(
                    "break statement outside of loop".into(),
                ));
            }
            ExecFlow::ContinueLoop => {
                return Err(Error::ScriptRuntime(
                    "continue statement outside of loop".into(),
                ));
            }
        }

        Ok(callback_env
            .remove(INTERNAL_RETURN_SLOT)
            .unwrap_or(Value::Undefined))
    }

    pub(super) fn execute_array_callback_in_env(
        &mut self,
        callback: &ScriptHandler,
        args: &[Value],
        env: &mut HashMap<String, Value>,
        event: &EventState,
    ) -> Result<()> {
        let mut previous_values = Vec::with_capacity(callback.params.len());
        for param in &callback.params {
            previous_values.push((param.name.clone(), env.get(&param.name).cloned()));
        }

        let mut callback_event = event.clone();
        let event_param = None;
        self.bind_handler_params(callback, args, env, &event_param, &callback_event)?;
        let result = self.execute_stmts(&callback.stmts, &event_param, &mut callback_event, env);
        env.remove(INTERNAL_RETURN_SLOT);

        for (name, previous) in previous_values {
            if let Some(previous) = previous {
                env.insert(name, previous);
            } else {
                env.remove(&name);
            }
        }

        match result? {
            ExecFlow::Continue | ExecFlow::Return => Ok(()),
            ExecFlow::Break => Err(Error::ScriptRuntime(
                "break statement outside of loop".into(),
            )),
            ExecFlow::ContinueLoop => Err(Error::ScriptRuntime(
                "continue statement outside of loop".into(),
            )),
        }
    }

    pub(super) fn execute_array_like_foreach_in_env(
        &mut self,
        target_value: Value,
        callback: &ScriptHandler,
        env: &mut HashMap<String, Value>,
        event: &EventState,
        target_label: &str,
    ) -> Result<()> {
        match target_value {
            Value::NodeList(nodes) => {
                let snapshot = nodes.clone();
                for (idx, node) in snapshot.into_iter().enumerate() {
                    self.execute_array_callback_in_env(
                        callback,
                        &[
                            Value::Node(node),
                            Value::Number(idx as i64),
                            Value::NodeList(nodes.clone()),
                        ],
                        env,
                        event,
                    )?;
                }
            }
            Value::Array(values) => {
                let input = values.borrow().clone();
                for (idx, item) in input.into_iter().enumerate() {
                    self.execute_array_callback_in_env(
                        callback,
                        &[item, Value::Number(idx as i64), Value::Array(values.clone())],
                        env,
                        event,
                    )?;
                }
            }
            Value::Map(map) => {
                let snapshot = map.borrow().entries.clone();
                for (key, value) in snapshot {
                    self.execute_array_callback_in_env(
                        callback,
                        &[value, key, Value::Map(map.clone())],
                        env,
                        event,
                    )?;
                }
            }
            Value::Set(set) => {
                let snapshot = set.borrow().values.clone();
                for value in snapshot {
                    self.execute_array_callback_in_env(
                        callback,
                        &[value.clone(), value, Value::Set(set.clone())],
                        env,
                        event,
                    )?;
                }
            }
            Value::Object(entries) => {
                if Self::is_url_search_params_object(&entries.borrow()) {
                    let snapshot = Self::url_search_params_pairs_from_object_entries(&entries.borrow());
                    for (key, value) in snapshot {
                        self.execute_array_callback_in_env(
                            callback,
                            &[
                                Value::String(value),
                                Value::String(key),
                                Value::Object(entries.clone()),
                            ],
                            env,
                            event,
                        )?;
                    }
                } else {
                    return Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target_label
                    )));
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an array",
                    target_label
                )));
            }
        }
        Ok(())
    }

    pub(super) fn normalize_slice_index(len: usize, index: i64) -> usize {
        if index < 0 {
            len.saturating_sub(index.unsigned_abs() as usize)
        } else {
            (index as usize).min(len)
        }
    }

    pub(super) fn normalize_splice_start_index(len: usize, start: i64) -> usize {
        if start < 0 {
            len.saturating_sub(start.unsigned_abs() as usize)
        } else {
            (start as usize).min(len)
        }
    }

    pub(super) fn normalize_substring_index(len: usize, index: i64) -> usize {
        if index < 0 {
            0
        } else {
            (index as usize).min(len)
        }
    }

    pub(super) fn char_index_to_byte(value: &str, char_index: usize) -> usize {
        value
            .char_indices()
            .nth(char_index)
            .map(|(idx, _)| idx)
            .unwrap_or(value.len())
    }

    pub(super) fn substring_chars(value: &str, start: usize, end: usize) -> String {
        if start >= end {
            return String::new();
        }
        value.chars().skip(start).take(end - start).collect()
    }

    pub(super) fn split_string(
        value: &str,
        separator: Option<String>,
        limit: Option<i64>,
    ) -> Vec<Value> {
        let mut parts = match separator {
            None => vec![Value::String(value.to_string())],
            Some(separator) => {
                if separator.is_empty() {
                    value
                        .chars()
                        .map(|ch| Value::String(ch.to_string()))
                        .collect::<Vec<_>>()
                } else {
                    value
                        .split(&separator)
                        .map(|part| Value::String(part.to_string()))
                        .collect::<Vec<_>>()
                }
            }
        };

        if let Some(limit) = limit {
            if limit == 0 {
                parts.clear();
            } else if limit > 0 {
                parts.truncate(limit as usize);
            }
        }

        parts
    }

    pub(super) fn split_string_with_regex(
        value: &str,
        regex: &Rc<RefCell<RegexValue>>,
        limit: Option<i64>,
    ) -> Vec<Value> {
        let compiled = regex.borrow().compiled.clone();
        let mut parts = compiled
            .split(value)
            .map(|part| Value::String(part.to_string()))
            .collect::<Vec<_>>();
        if let Some(limit) = limit {
            if limit == 0 {
                parts.clear();
            } else if limit > 0 {
                parts.truncate(limit as usize);
            }
        }
        parts
    }

    pub(super) fn expand_regex_replacement(template: &str, captures: &Captures<'_>) -> String {
        let chars = template.chars().collect::<Vec<_>>();
        let mut i = 0usize;
        let mut out = String::new();
        while i < chars.len() {
            if chars[i] != '$' {
                out.push(chars[i]);
                i += 1;
                continue;
            }
            if i + 1 >= chars.len() {
                out.push('$');
                i += 1;
                continue;
            }
            let next = chars[i + 1];
            match next {
                '$' => {
                    out.push('$');
                    i += 2;
                }
                '&' => {
                    if let Some(full) = captures.get(0) {
                        out.push_str(full.as_str());
                    }
                    i += 2;
                }
                '0'..='9' => {
                    let mut idx = (next as u8 - b'0') as usize;
                    let mut consumed = 2usize;
                    if i + 2 < chars.len() && chars[i + 2].is_ascii_digit() {
                        let candidate = idx * 10 + (chars[i + 2] as u8 - b'0') as usize;
                        if captures.get(candidate).is_some() {
                            idx = candidate;
                            consumed = 3;
                        }
                    }
                    if idx > 0 {
                        if let Some(group) = captures.get(idx) {
                            out.push_str(group.as_str());
                        }
                    } else {
                        out.push('$');
                        out.push('0');
                    }
                    i += consumed;
                }
                _ => {
                    out.push('$');
                    out.push(next);
                    i += 2;
                }
            }
        }
        out
    }

    pub(super) fn replace_string_with_regex(
        value: &str,
        regex: &Rc<RefCell<RegexValue>>,
        replacement: &str,
    ) -> String {
        let (compiled, global) = {
            let regex = regex.borrow();
            (regex.compiled.clone(), regex.global)
        };

        if global {
            let mut out = String::new();
            let mut last_end = 0usize;
            for captures in compiled.captures_iter(value) {
                let Some(full) = captures.get(0) else {
                    continue;
                };
                out.push_str(&value[last_end..full.start()]);
                out.push_str(&Self::expand_regex_replacement(replacement, &captures));
                last_end = full.end();
            }
            out.push_str(&value[last_end..]);
            out
        } else if let Some(captures) = compiled.captures(value) {
            if let Some(full) = captures.get(0) {
                let mut out = String::new();
                out.push_str(&value[..full.start()]);
                out.push_str(&Self::expand_regex_replacement(replacement, &captures));
                out.push_str(&value[full.end()..]);
                out
            } else {
                value.to_string()
            }
        } else {
            value.to_string()
        }
    }

    pub(super) fn string_index_of(
        value: &str,
        search: &str,
        start_char_idx: usize,
    ) -> Option<usize> {
        let start_byte = Self::char_index_to_byte(value, start_char_idx);
        let pos = value.get(start_byte..)?.find(search)?;
        Some(value[..start_byte + pos].chars().count())
    }

    pub(super) fn parse_date_string_to_epoch_ms(src: &str) -> Option<i64> {
        let src = src.trim();
        if src.is_empty() {
            return None;
        }

        let bytes = src.as_bytes();
        let mut i = 0usize;

        let mut sign = 1i64;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            if bytes[i] == b'-' {
                sign = -1;
            }
            i += 1;
        }

        let year_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i <= year_start || (i - year_start) < 4 {
            return None;
        }
        let year = sign * src.get(year_start..i)?.parse::<i64>().ok()?;

        if i >= bytes.len() || bytes[i] != b'-' {
            return None;
        }
        i += 1;
        let month = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
        if i >= bytes.len() || bytes[i] != b'-' {
            return None;
        }
        i += 1;
        let day = Self::parse_fixed_digits_i64(src, &mut i, 2)?;

        let month = u32::try_from(month).ok()?;
        if !(1..=12).contains(&month) {
            return None;
        }
        let day = u32::try_from(day).ok()?;
        if day == 0 || day > Self::days_in_month(year, month) {
            return None;
        }

        let mut hour = 0i64;
        let mut minute = 0i64;
        let mut second = 0i64;
        let mut millisecond = 0i64;
        let mut offset_minutes = 0i64;

        if i < bytes.len() {
            if bytes[i] != b'T' && bytes[i] != b' ' {
                return None;
            }
            i += 1;

            hour = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
            if i >= bytes.len() || bytes[i] != b':' {
                return None;
            }
            i += 1;
            minute = Self::parse_fixed_digits_i64(src, &mut i, 2)?;

            if i < bytes.len() && bytes[i] == b':' {
                i += 1;
                second = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
            }

            if i < bytes.len() && bytes[i] == b'.' {
                i += 1;
                let frac_start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i == frac_start {
                    return None;
                }

                let frac = src.get(frac_start..i)?;
                let mut parsed = 0i64;
                let mut digits = 0usize;
                for ch in frac.chars().take(3) {
                    parsed = parsed * 10 + i64::from(ch.to_digit(10)?);
                    digits += 1;
                }
                while digits < 3 {
                    parsed *= 10;
                    digits += 1;
                }
                millisecond = parsed;
            }

            if i < bytes.len() {
                match bytes[i] {
                    b'Z' | b'z' => {
                        i += 1;
                    }
                    b'+' | b'-' => {
                        let tz_sign = if bytes[i] == b'+' { 1 } else { -1 };
                        i += 1;
                        let tz_hour = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
                        let tz_minute = if i < bytes.len() && bytes[i] == b':' {
                            i += 1;
                            Self::parse_fixed_digits_i64(src, &mut i, 2)?
                        } else {
                            Self::parse_fixed_digits_i64(src, &mut i, 2)?
                        };
                        if tz_hour > 23 || tz_minute > 59 {
                            return None;
                        }
                        offset_minutes = tz_sign * (tz_hour * 60 + tz_minute);
                    }
                    _ => return None,
                }
            }
        }

        if i != bytes.len() {
            return None;
        }
        if hour > 23 || minute > 59 || second > 59 {
            return None;
        }

        let timestamp_ms = Self::utc_timestamp_ms_from_components(
            year,
            i64::from(month) - 1,
            i64::from(day),
            hour,
            minute,
            second,
            millisecond,
        );
        Some(timestamp_ms - offset_minutes * 60_000)
    }

    pub(super) fn parse_fixed_digits_i64(src: &str, i: &mut usize, width: usize) -> Option<i64> {
        let end = i.checked_add(width)?;
        let segment = src.get(*i..end)?;
        if !segment.as_bytes().iter().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *i = end;
        segment.parse::<i64>().ok()
    }

    pub(super) fn format_iso_8601_utc(timestamp_ms: i64) -> String {
        let (year, month, day, hour, minute, second, millisecond) =
            Self::date_components_utc(timestamp_ms);
        let year_str = if (0..=9999).contains(&year) {
            format!("{year:04}")
        } else if year < 0 {
            format!("-{:06}", -(year as i128))
        } else {
            format!("+{:06}", year)
        };
        format!(
            "{year_str}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millisecond:03}Z"
        )
    }

    pub(super) fn date_components_utc(timestamp_ms: i64) -> (i64, u32, u32, u32, u32, u32, u32) {
        let days = timestamp_ms.div_euclid(86_400_000);
        let rem = timestamp_ms.rem_euclid(86_400_000);
        let hour = (rem / 3_600_000) as u32;
        let minute = ((rem % 3_600_000) / 60_000) as u32;
        let second = ((rem % 60_000) / 1_000) as u32;
        let millisecond = (rem % 1_000) as u32;
        let (year, month, day) = Self::civil_from_days(days);
        (year, month, day, hour, minute, second, millisecond)
    }

    pub(super) fn utc_timestamp_ms_from_components(
        year: i64,
        month_zero_based: i64,
        day: i64,
        hour: i64,
        minute: i64,
        second: i64,
        millisecond: i64,
    ) -> i64 {
        let (norm_year, norm_month) = Self::normalize_year_month(year, month_zero_based);
        let mut days = Self::days_from_civil(norm_year, norm_month, 1) + (day - 1);
        let mut time_ms = ((hour * 60 + minute) * 60 + second) * 1_000 + millisecond;
        days += time_ms.div_euclid(86_400_000);
        time_ms = time_ms.rem_euclid(86_400_000);

        let out = (days as i128) * 86_400_000i128 + (time_ms as i128);
        out.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
    }

    pub(super) fn normalize_year_month(year: i64, month_zero_based: i64) -> (i64, u32) {
        let total_month = year.saturating_mul(12).saturating_add(month_zero_based);
        let norm_year = total_month.div_euclid(12);
        let norm_month = total_month.rem_euclid(12) as u32 + 1;
        (norm_year, norm_month)
    }

    pub(super) fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
        let adjusted_year = year - if month <= 2 { 1 } else { 0 };
        let era = adjusted_year.div_euclid(400);
        let yoe = adjusted_year - era * 400;
        let month = i64::from(month);
        let day = i64::from(day);
        let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    pub(super) fn civil_from_days(days: i64) -> (i64, u32, u32) {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096).div_euclid(365);
        let mut year = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2).div_euclid(153);
        let day = (doy - (153 * mp + 2).div_euclid(5) + 1) as u32;
        let month = (mp + if mp < 10 { 3 } else { -9 }) as u32;
        if month <= 2 {
            year += 1;
        }
        (year, month, day)
    }

    pub(super) fn days_in_month(year: i64, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    pub(super) fn is_leap_year(year: i64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    pub(super) fn numeric_value(&self, value: &Value) -> f64 {
        match value {
            Value::Number(v) => *v as f64,
            Value::Float(v) => *v,
            Value::BigInt(v) => v.to_f64().unwrap_or_else(|| {
                if v.sign() == Sign::Minus {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                }
            }),
            Value::Date(v) => *v.borrow() as f64,
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            _ => value.as_string().parse::<f64>().unwrap_or(0.0),
        }
    }

    pub(super) fn coerce_number_for_global(value: &Value) -> f64 {
        match value {
            Value::Number(v) => *v as f64,
            Value::Float(v) => *v,
            Value::BigInt(v) => v.to_f64().unwrap_or_else(|| {
                if v.sign() == Sign::Minus {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                }
            }),
            Value::Bool(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            Value::String(v) => Self::parse_js_number_from_string(v),
            Value::Date(v) => *v.borrow() as f64,
            Value::Object(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::Set(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::SetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Symbol(_)
            | Value::RegExp(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_) => f64::NAN,
            Value::Array(values) => {
                let rendered = Value::Array(values.clone()).as_string();
                Self::parse_js_number_from_string(&rendered)
            }
        }
    }

    pub(super) fn to_i32_for_bitwise(&self, value: &Value) -> i32 {
        let numeric = self.numeric_value(value);
        if !numeric.is_finite() {
            return 0;
        }
        let unsigned = numeric.trunc().rem_euclid(4_294_967_296.0);
        if unsigned >= 2_147_483_648.0 {
            (unsigned - 4_294_967_296.0) as i32
        } else {
            unsigned as i32
        }
    }

    pub(super) fn to_u32_for_bitwise(&self, value: &Value) -> u32 {
        let numeric = self.numeric_value(value);
        if !numeric.is_finite() {
            return 0;
        }
        numeric.trunc().rem_euclid(4_294_967_296.0) as u32
    }

    pub(super) fn resolve_dom_query_var_path_value(
        &self,
        base: &str,
        path: &[String],
        env: &HashMap<String, Value>,
    ) -> Result<Option<Value>> {
        let Some(mut value) = env.get(base).cloned() else {
            return Err(Error::ScriptRuntime(format!(
                "unknown element variable: {}",
                base
            )));
        };

        for key in path {
            let next = match self.object_property_from_value(&value, key) {
                Ok(next) => next,
                Err(_) => return Ok(None),
            };
            if matches!(next, Value::Null | Value::Undefined) {
                return Ok(None);
            }
            value = next;
        }

        Ok(Some(value))
    }

    pub(super) fn resolve_dom_query_list_static(
        &mut self,
        target: &DomQuery,
    ) -> Result<Option<Vec<NodeId>>> {
        match target {
            DomQuery::BySelectorAll { selector } => {
                Ok(Some(self.dom.query_selector_all(selector)?))
            }
            DomQuery::QuerySelectorAll { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                Ok(Some(
                    self.dom.query_selector_all_from(&target_node, selector)?,
                ))
            }
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_static(target)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, None)?;
                Ok(list.get(index).copied().map(|node| vec![node]))
            }
            DomQuery::BySelectorAllIndex { selector, index } => {
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                Ok(self
                    .dom
                    .query_selector_all(selector)?
                    .get(index)
                    .copied()
                    .map(|node| vec![node]))
            }
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                let list = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(list.get(index).copied().map(|node| vec![node]))
            }
            DomQuery::Var(_) | DomQuery::VarPath { .. } => Err(Error::ScriptRuntime(
                "element variable cannot be resolved in static context".into(),
            )),
            _ => Ok(None),
        }
    }

    pub(super) fn resolve_dom_query_list_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<Option<Vec<NodeId>>> {
        match target {
            DomQuery::Var(name) => match env.get(name) {
                Some(Value::NodeList(nodes)) => Ok(Some(nodes.clone())),
                Some(Value::Node(_)) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a node list",
                    name
                ))),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a node list",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown element variable: {}",
                    name
                ))),
            },
            DomQuery::VarPath { base, path } => {
                let Some(value) = self.resolve_dom_query_var_path_value(base, path, env)? else {
                    return Ok(None);
                };
                match value {
                    Value::NodeList(nodes) => Ok(Some(nodes)),
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a node list",
                        target.describe_call()
                    ))),
                }
            }
            DomQuery::QuerySelectorAll {
                target: query_target,
                selector,
            } => {
                let Some(target_node) = self.resolve_dom_query_runtime(query_target, env)? else {
                    return Ok(None);
                };
                Ok(Some(
                    self.dom.query_selector_all_from(&target_node, selector)?,
                ))
            }
            DomQuery::QuerySelectorAllIndex {
                target: query_target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_runtime(query_target, env)? else {
                    return Ok(None);
                };
                let all = self.dom.query_selector_all_from(&target_node, selector)?;
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                Ok(all.get(index).copied().map(|node| vec![node]))
            }
            _ => self.resolve_dom_query_list_static(target),
        }
    }

    pub(super) fn resolve_dom_query_static(&mut self, target: &DomQuery) -> Result<Option<NodeId>> {
        match target {
            DomQuery::DocumentRoot => Ok(Some(self.dom.root)),
            DomQuery::DocumentBody => Ok(self.dom.body()),
            DomQuery::DocumentHead => Ok(self.dom.head()),
            DomQuery::DocumentElement => Ok(self.dom.document_element()),
            DomQuery::ById(id) => Ok(self.dom.by_id(id)),
            DomQuery::BySelector(selector) => self.dom.query_selector(selector),
            DomQuery::BySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::BySelectorAllIndex { selector, index } => {
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                let all = self.dom.query_selector_all(selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_static(target)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, None)?;
                Ok(list.get(index).copied())
            }
            DomQuery::QuerySelector { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                self.dom.query_selector_from(&target_node, selector)
            }
            DomQuery::QuerySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                let all = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::FormElementsIndex { form, index } => {
                let Some(form_node) = self.resolve_dom_query_static(form)? else {
                    return Ok(None);
                };
                let all = self.form_elements(form_node)?;
                let index = self.resolve_runtime_dom_index(index, None)?;
                Ok(all.get(index).copied())
            }
            DomQuery::Var(_) | DomQuery::VarPath { .. } => Err(Error::ScriptRuntime(
                "element variable cannot be resolved in static context".into(),
            )),
        }
    }

    pub(super) fn resolve_dom_query_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<Option<NodeId>> {
        match target {
            DomQuery::DocumentRoot => Ok(Some(self.dom.root)),
            DomQuery::DocumentBody => Ok(self.dom.body()),
            DomQuery::DocumentHead => Ok(self.dom.head()),
            DomQuery::DocumentElement => Ok(self.dom.document_element()),
            DomQuery::Var(name) => match env.get(name) {
                Some(Value::Node(node)) => Ok(Some(*node)),
                Some(Value::NodeList(_)) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a single element",
                    name
                ))),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a single element",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown element variable: {}",
                    name
                ))),
            },
            DomQuery::VarPath { base, path } => {
                let Some(value) = self.resolve_dom_query_var_path_value(base, path, env)? else {
                    return Ok(None);
                };
                match value {
                    Value::Node(node) => Ok(Some(node)),
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a single element",
                        target.describe_call()
                    ))),
                }
            }
            DomQuery::BySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::QuerySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_runtime(target, env)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                Ok(list.get(index).copied())
            }
            DomQuery::QuerySelector { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_runtime(target, env)? else {
                    return Ok(None);
                };
                self.dom.query_selector_from(&target_node, selector)
            }
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_runtime(target, env)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                let all = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::FormElementsIndex { form, index } => {
                let Some(form_node) = self.resolve_dom_query_runtime(form, env)? else {
                    return Ok(None);
                };
                let all = self.form_elements(form_node)?;
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                Ok(all.get(index).copied())
            }
            _ => self.resolve_dom_query_static(target),
        }
    }

    pub(super) fn resolve_dom_query_required_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<NodeId> {
        self.resolve_dom_query_runtime(target, env)?.ok_or_else(|| {
            Error::ScriptRuntime(format!("{} returned null", target.describe_call()))
        })
    }

    pub(super) fn resolve_runtime_dom_index(
        &mut self,
        index: &DomIndex,
        env: Option<&HashMap<String, Value>>,
    ) -> Result<usize> {
        match index {
            DomIndex::Static(index) => Ok(*index),
            DomIndex::Dynamic(expr_src) => {
                let expr = parse_expr(expr_src)?;
                let event = EventState::new("script", self.dom.root, self.now_ms);
                let value = self.eval_expr(
                    &expr,
                    env.ok_or_else(|| {
                        Error::ScriptRuntime("dynamic index requires runtime context".into())
                    })?,
                    &None,
                    &event,
                )?;
                self.value_as_index(&value).ok_or_else(|| {
                    Error::ScriptRuntime(format!("invalid index expression: {expr_src}"))
                })
            }
        }
    }

    pub(super) fn describe_dom_prop(&self, prop: &DomProp) -> String {
        match prop {
            DomProp::Attributes => "attributes".into(),
            DomProp::AssignedSlot => "assignedSlot".into(),
            DomProp::Value => "value".into(),
            DomProp::ValueLength => "value.length".into(),
            DomProp::Checked => "checked".into(),
            DomProp::Open => "open".into(),
            DomProp::ReturnValue => "returnValue".into(),
            DomProp::ClosedBy => "closedBy".into(),
            DomProp::Readonly => "readonly".into(),
            DomProp::Required => "required".into(),
            DomProp::Disabled => "disabled".into(),
            DomProp::TextContent => "textContent".into(),
            DomProp::InnerText => "innerText".into(),
            DomProp::InnerHtml => "innerHTML".into(),
            DomProp::OuterHtml => "outerHTML".into(),
            DomProp::ClassName => "className".into(),
            DomProp::ClassList => "classList".into(),
            DomProp::ClassListLength => "classList.length".into(),
            DomProp::Part => "part".into(),
            DomProp::PartLength => "part.length".into(),
            DomProp::Id => "id".into(),
            DomProp::TagName => "tagName".into(),
            DomProp::LocalName => "localName".into(),
            DomProp::NamespaceUri => "namespaceURI".into(),
            DomProp::Prefix => "prefix".into(),
            DomProp::NextElementSibling => "nextElementSibling".into(),
            DomProp::PreviousElementSibling => "previousElementSibling".into(),
            DomProp::Slot => "slot".into(),
            DomProp::Role => "role".into(),
            DomProp::ElementTiming => "elementTiming".into(),
            DomProp::Name => "name".into(),
            DomProp::Lang => "lang".into(),
            DomProp::ClientWidth => "clientWidth".into(),
            DomProp::ClientHeight => "clientHeight".into(),
            DomProp::ClientLeft => "clientLeft".into(),
            DomProp::ClientTop => "clientTop".into(),
            DomProp::CurrentCssZoom => "currentCSSZoom".into(),
            DomProp::OffsetWidth => "offsetWidth".into(),
            DomProp::OffsetHeight => "offsetHeight".into(),
            DomProp::OffsetLeft => "offsetLeft".into(),
            DomProp::OffsetTop => "offsetTop".into(),
            DomProp::ScrollWidth => "scrollWidth".into(),
            DomProp::ScrollHeight => "scrollHeight".into(),
            DomProp::ScrollLeft => "scrollLeft".into(),
            DomProp::ScrollTop => "scrollTop".into(),
            DomProp::ScrollLeftMax => "scrollLeftMax".into(),
            DomProp::ScrollTopMax => "scrollTopMax".into(),
            DomProp::ShadowRoot => "shadowRoot".into(),
            DomProp::Dataset(_) => "dataset".into(),
            DomProp::Style(_) => "style".into(),
            DomProp::AriaString(prop_name) => prop_name.clone(),
            DomProp::AriaElementRefSingle(prop_name) => prop_name.clone(),
            DomProp::AriaElementRefList(prop_name) => prop_name.clone(),
            DomProp::ActiveElement => "activeElement".into(),
            DomProp::CharacterSet => "characterSet".into(),
            DomProp::CompatMode => "compatMode".into(),
            DomProp::ContentType => "contentType".into(),
            DomProp::ReadyState => "readyState".into(),
            DomProp::Referrer => "referrer".into(),
            DomProp::Title => "title".into(),
            DomProp::Url => "URL".into(),
            DomProp::DocumentUri => "documentURI".into(),
            DomProp::Location => "location".into(),
            DomProp::LocationHref => "location.href".into(),
            DomProp::LocationProtocol => "location.protocol".into(),
            DomProp::LocationHost => "location.host".into(),
            DomProp::LocationHostname => "location.hostname".into(),
            DomProp::LocationPort => "location.port".into(),
            DomProp::LocationPathname => "location.pathname".into(),
            DomProp::LocationSearch => "location.search".into(),
            DomProp::LocationHash => "location.hash".into(),
            DomProp::LocationOrigin => "location.origin".into(),
            DomProp::LocationAncestorOrigins => "location.ancestorOrigins".into(),
            DomProp::History => "history".into(),
            DomProp::HistoryLength => "history.length".into(),
            DomProp::HistoryState => "history.state".into(),
            DomProp::HistoryScrollRestoration => "history.scrollRestoration".into(),
            DomProp::DefaultView => "defaultView".into(),
            DomProp::Hidden => "hidden".into(),
            DomProp::VisibilityState => "visibilityState".into(),
            DomProp::Forms => "forms".into(),
            DomProp::Images => "images".into(),
            DomProp::Links => "links".into(),
            DomProp::Scripts => "scripts".into(),
            DomProp::Children => "children".into(),
            DomProp::ChildElementCount => "childElementCount".into(),
            DomProp::FirstElementChild => "firstElementChild".into(),
            DomProp::LastElementChild => "lastElementChild".into(),
            DomProp::CurrentScript => "currentScript".into(),
            DomProp::FormsLength => "forms.length".into(),
            DomProp::ImagesLength => "images.length".into(),
            DomProp::LinksLength => "links.length".into(),
            DomProp::ScriptsLength => "scripts.length".into(),
            DomProp::ChildrenLength => "children.length".into(),
            DomProp::AnchorAttributionSrc => "attributionSrc".into(),
            DomProp::AnchorDownload => "download".into(),
            DomProp::AnchorHash => "hash".into(),
            DomProp::AnchorHost => "host".into(),
            DomProp::AnchorHostname => "hostname".into(),
            DomProp::AnchorHref => "href".into(),
            DomProp::AnchorHreflang => "hreflang".into(),
            DomProp::AnchorInterestForElement => "interestForElement".into(),
            DomProp::AnchorOrigin => "origin".into(),
            DomProp::AnchorPassword => "password".into(),
            DomProp::AnchorPathname => "pathname".into(),
            DomProp::AnchorPing => "ping".into(),
            DomProp::AnchorPort => "port".into(),
            DomProp::AnchorProtocol => "protocol".into(),
            DomProp::AnchorReferrerPolicy => "referrerPolicy".into(),
            DomProp::AnchorRel => "rel".into(),
            DomProp::AnchorRelList => "relList".into(),
            DomProp::AnchorRelListLength => "relList.length".into(),
            DomProp::AnchorSearch => "search".into(),
            DomProp::AnchorTarget => "target".into(),
            DomProp::AnchorText => "text".into(),
            DomProp::AnchorType => "type".into(),
            DomProp::AnchorUsername => "username".into(),
            DomProp::AnchorCharset => "charset".into(),
            DomProp::AnchorCoords => "coords".into(),
            DomProp::AnchorRev => "rev".into(),
            DomProp::AnchorShape => "shape".into(),
        }
    }

    pub(super) fn event_node_label(&self, node: NodeId) -> String {
        if let Some(id) = self.dom.attr(node, "id") {
            if !id.is_empty() {
                return id;
            }
        }
        self.dom
            .tag_name(node)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("node-{}", node.0))
    }

    pub(super) fn trace_node_label(&self, node: NodeId) -> String {
        if let Some(id) = self.dom.attr(node, "id") {
            if !id.is_empty() {
                return format!("#{id}");
            }
        }
        self.dom
            .tag_name(node)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("node-{}", node.0))
    }

    pub(super) fn value_to_i64(value: &Value) -> i64 {
        match value {
            Value::Number(v) => *v,
            Value::Float(v) => *v as i64,
            Value::BigInt(v) => v.to_i64().unwrap_or_else(|| {
                if v.sign() == Sign::Minus {
                    i64::MIN
                } else {
                    i64::MAX
                }
            }),
            Value::Bool(v) => {
                if *v {
                    1
                } else {
                    0
                }
            }
            Value::String(v) => v
                .parse::<i64>()
                .ok()
                .or_else(|| v.parse::<f64>().ok().map(|n| n as i64))
                .unwrap_or(0),
            Value::Array(values) => Value::Array(values.clone())
                .as_string()
                .parse::<i64>()
                .ok()
                .or_else(|| {
                    Value::Array(values.clone())
                        .as_string()
                        .parse::<f64>()
                        .ok()
                        .map(|n| n as i64)
                })
                .unwrap_or(0),
            Value::Date(value) => *value.borrow(),
            Value::Object(_) => 0,
            Value::Promise(_) => 0,
            Value::Map(_) => 0,
            Value::Set(_) => 0,
            Value::Blob(_) => 0,
            Value::ArrayBuffer(_) => 0,
            Value::TypedArray(_) => 0,
            Value::StringConstructor => 0,
            Value::TypedArrayConstructor(_) => 0,
            Value::BlobConstructor => 0,
            Value::UrlConstructor => 0,
            Value::ArrayBufferConstructor => 0,
            Value::PromiseConstructor => 0,
            Value::MapConstructor => 0,
            Value::SetConstructor => 0,
            Value::SymbolConstructor => 0,
            Value::RegExpConstructor => 0,
            Value::PromiseCapability(_) => 0,
            Value::Symbol(_) => 0,
            Value::RegExp(_) => 0,
            Value::Node(_) => 0,
            Value::NodeList(_) => 0,
            Value::FormData(_) => 0,
            Value::Function(_) => 0,
            Value::Null => 0,
            Value::Undefined => 0,
        }
    }

    pub(super) fn next_random_f64(&mut self) -> f64 {
        // xorshift64*: simple deterministic PRNG for test runtime.
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = if x == 0 { 0xA5A5_A5A5_A5A5_A5A5 } else { x };
        let out = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        // Convert top 53 bits to [0.0, 1.0).
        let mantissa = out >> 11;
        (mantissa as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    pub(super) fn schedule_timeout(
        &mut self,
        callback: TimerCallback,
        delay_ms: i64,
        callback_args: Vec<Value>,
        env: &HashMap<String, Value>,
    ) -> i64 {
        let delay_ms = delay_ms.max(0);
        let due_at = self.now_ms + delay_ms;
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        let order = self.next_task_order;
        self.next_task_order += 1;
        self.task_queue.push(ScheduledTask {
            id,
            due_at,
            order,
            interval_ms: None,
            callback,
            callback_args,
            env: env.clone(),
        });
        self.trace_timer_line(format!(
            "[timer] schedule timeout id={} due_at={} delay_ms={}",
            id, due_at, delay_ms
        ));
        id
    }

    pub(super) fn schedule_interval(
        &mut self,
        callback: TimerCallback,
        interval_ms: i64,
        callback_args: Vec<Value>,
        env: &HashMap<String, Value>,
    ) -> i64 {
        let interval_ms = interval_ms.max(0);
        let due_at = self.now_ms + interval_ms;
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        let order = self.next_task_order;
        self.next_task_order += 1;
        self.task_queue.push(ScheduledTask {
            id,
            due_at,
            order,
            interval_ms: Some(interval_ms),
            callback,
            callback_args,
            env: env.clone(),
        });
        self.trace_timer_line(format!(
            "[timer] schedule interval id={} due_at={} interval_ms={}",
            id, due_at, interval_ms
        ));
        id
    }

    pub(super) fn clear_timeout(&mut self, id: i64) {
        let before = self.task_queue.len();
        self.task_queue.retain(|task| task.id != id);
        let removed = before.saturating_sub(self.task_queue.len());
        let mut running_canceled = false;
        if self.running_timer_id == Some(id) {
            self.running_timer_canceled = true;
            running_canceled = true;
        }
        self.trace_timer_line(format!(
            "[timer] clear id={} removed={} running_canceled={}",
            id, removed, running_canceled
        ));
    }

    pub(super) fn compile_and_register_script(&mut self, script: &str) -> Result<()> {
        let stmts = parse_block_statements(script)?;
        let mut event = EventState::new("script", self.dom.root, self.now_ms);
        let mut env = self.script_env.clone();
        self.run_in_task_context(|this| {
            this.execute_stmts(&stmts, &None, &mut event, &mut env)
                .map(|_| ())
        })?;
        self.script_env = env;

        Ok(())
    }
}
