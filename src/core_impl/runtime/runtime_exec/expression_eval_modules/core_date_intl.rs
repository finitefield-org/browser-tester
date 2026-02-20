impl Harness {
    fn eval_expr_core_date_intl(
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
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
