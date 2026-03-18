use super::*;

impl Harness {
    pub(crate) fn execute_receiver_builtin_intl_family(
        &mut self,
        family: &str,
        member: &str,
        receiver: &Value,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let value = match family {
            "intl_collator" => {
                let (locale, case_first, sensitivity, numeric) = self
                    .resolve_intl_collator_options(receiver)
                    .map_err(|_| Self::incompatible_receiver_error(family))?;
                Some(match member {
                    "resolvedOptions" => Self::new_object_value(vec![
                        ("locale".into(), Value::String(locale)),
                        ("usage".into(), Value::String("sort".to_string())),
                        ("sensitivity".into(), Value::String(sensitivity)),
                        ("ignorePunctuation".into(), Value::Bool(false)),
                        ("collation".into(), Value::String("default".to_string())),
                        ("numeric".into(), Value::Bool(numeric)),
                        ("caseFirst".into(), Value::String(case_first)),
                    ]),
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Intl.Collator method: {member}"
                        )));
                    }
                })
            }
            "intl_date_time_format" => {
                let (locale, options) = self
                    .resolve_intl_date_time_options(receiver)
                    .map_err(|_| Self::incompatible_receiver_error(family))?;
                Some(match member {
                    "formatToParts" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        let timestamp_ms = if matches!(value, Value::Undefined) {
                            self.scheduler.now_ms
                        } else {
                            self.coerce_intl_date_time_timestamp_ms(&value)?
                        };
                        let parts =
                            self.intl_format_date_time_to_parts(timestamp_ms, &locale, &options);
                        self.intl_date_time_parts_to_value(&parts, None)
                    }
                    "formatRange" => {
                        let start = args.first().cloned().unwrap_or(Value::Undefined);
                        let end = args.get(1).cloned().unwrap_or(Value::Undefined);
                        let start_ms = self.coerce_date_timestamp_ms(&start);
                        let end_ms = self.coerce_date_timestamp_ms(&end);
                        Value::String(
                            self.intl_format_date_time_range(start_ms, end_ms, &locale, &options),
                        )
                    }
                    "formatRangeToParts" => {
                        let start = args.first().cloned().unwrap_or(Value::Undefined);
                        let end = args.get(1).cloned().unwrap_or(Value::Undefined);
                        let start_ms = self.coerce_date_timestamp_ms(&start);
                        let end_ms = self.coerce_date_timestamp_ms(&end);
                        let (parts, sources) = self.intl_format_date_time_range_to_parts(
                            start_ms, end_ms, &locale, &options,
                        );
                        self.intl_date_time_parts_to_value(&parts, Some(&sources))
                    }
                    "resolvedOptions" => {
                        self.intl_date_time_resolved_options_value(locale, &options)
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Intl.DateTimeFormat method: {member}"
                        )));
                    }
                })
            }
            "intl_display_names" => {
                let (locale, options) = self
                    .resolve_intl_display_names_options(receiver)
                    .map_err(|_| Self::incompatible_receiver_error(family))?;
                Some(match member {
                    "of" => {
                        let code = args
                            .first()
                            .cloned()
                            .unwrap_or(Value::Undefined)
                            .as_string();
                        return self
                            .intl_display_names_of(&locale, &options, &code)
                            .map(Some);
                    }
                    "resolvedOptions" => {
                        self.intl_display_names_resolved_options_value(locale, &options)
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Intl.DisplayNames method: {member}"
                        )));
                    }
                })
            }
            "intl_duration_format" => {
                let (locale, options) = self
                    .resolve_intl_duration_options(receiver)
                    .map_err(|_| Self::incompatible_receiver_error(family))?;
                Some(match member {
                    "formatToParts" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        let parts =
                            self.intl_format_duration_to_parts(&locale, &options, &value)?;
                        self.intl_date_time_parts_to_value(&parts, None)
                    }
                    "resolvedOptions" => {
                        self.intl_duration_resolved_options_value(locale, &options)
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Intl.DurationFormat method: {member}"
                        )));
                    }
                })
            }
            "intl_list_format" => {
                let (locale, options) = self
                    .resolve_intl_list_options(receiver)
                    .map_err(|_| Self::incompatible_receiver_error(family))?;
                Some(match member {
                    "formatToParts" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        let parts = self.intl_format_list_to_parts(&locale, &options, &value)?;
                        self.intl_date_time_parts_to_value(&parts, None)
                    }
                    "resolvedOptions" => self.intl_list_resolved_options_value(locale, &options),
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Intl.ListFormat method: {member}"
                        )));
                    }
                })
            }
            "intl_locale" => {
                let data = self
                    .resolve_intl_locale_data(receiver)
                    .map_err(|_| Self::incompatible_receiver_error(family))?;
                Some(match member {
                    "getCalendars" => Self::new_array_value(
                        self.intl_locale_get_calendars(&data)
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    ),
                    "getCollations" => Self::new_array_value(
                        self.intl_locale_get_collations(&data)
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    ),
                    "getHourCycles" => Self::new_array_value(
                        self.intl_locale_get_hour_cycles(&data)
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    ),
                    "getNumberingSystems" => Self::new_array_value(
                        self.intl_locale_get_numbering_systems(&data)
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    ),
                    "getTextInfo" => self.intl_locale_get_text_info(&data),
                    "getTimeZones" => Self::new_array_value(
                        self.intl_locale_get_time_zones(&data)
                            .into_iter()
                            .map(Value::String)
                            .collect::<Vec<_>>(),
                    ),
                    "getWeekInfo" => self.intl_locale_get_week_info(&data),
                    "maximize" => self.new_intl_locale_value(self.intl_locale_maximize_data(&data)),
                    "minimize" => self.new_intl_locale_value(self.intl_locale_minimize_data(&data)),
                    "toString" => Value::String(Self::intl_locale_data_to_string(&data)),
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Intl.Locale method: {member}"
                        )));
                    }
                })
            }
            "intl_number_format" => {
                let (locale, options) = self
                    .resolve_intl_number_format_options(receiver)
                    .map_err(|_| Self::incompatible_receiver_error(family))?;
                Some(match member {
                    "formatToParts" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        let parts =
                            self.intl_number_format_value_to_parts(&value, &locale, &options);
                        self.intl_date_time_parts_to_value(&parts, None)
                    }
                    "formatRange" => {
                        let start = args.first().cloned().unwrap_or(Value::Undefined);
                        let end = args.get(1).cloned().unwrap_or(Value::Undefined);
                        Value::String(self.intl_format_number_range(
                            Self::coerce_number_for_global(&start),
                            Self::coerce_number_for_global(&end),
                            &locale,
                            &options,
                        ))
                    }
                    "formatRangeToParts" => {
                        let start = args.first().cloned().unwrap_or(Value::Undefined);
                        let end = args.get(1).cloned().unwrap_or(Value::Undefined);
                        let (parts, sources) = self.intl_format_number_range_to_parts(
                            Self::coerce_number_for_global(&start),
                            Self::coerce_number_for_global(&end),
                            &locale,
                            &options,
                        );
                        self.intl_date_time_parts_to_value(&parts, Some(&sources))
                    }
                    "resolvedOptions" => self.intl_number_resolved_options_value(locale, &options),
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Intl.NumberFormat method: {member}"
                        )));
                    }
                })
            }
            "intl_plural_rules" => {
                let (locale, options) = self
                    .resolve_intl_plural_rules_options(receiver)
                    .map_err(|_| Self::incompatible_receiver_error(family))?;
                Some(match member {
                    "select" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        Value::String(self.intl_plural_rules_select(&locale, &options, &value))
                    }
                    "selectRange" => {
                        let start = args.first().cloned().unwrap_or(Value::Undefined);
                        let end = args.get(1).cloned().unwrap_or(Value::Undefined);
                        Value::String(
                            self.intl_plural_rules_select_range(&locale, &options, &start, &end),
                        )
                    }
                    "resolvedOptions" => {
                        self.intl_plural_rules_resolved_options_value(locale, &options)
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Intl.PluralRules method: {member}"
                        )));
                    }
                })
            }
            "intl_relative_time_format" => {
                let (locale, options) = self
                    .resolve_intl_relative_time_options(receiver)
                    .map_err(|_| Self::incompatible_receiver_error(family))?;
                Some(match member {
                    "format" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        let unit = args.get(1).cloned().unwrap_or(Value::Undefined);
                        Value::String(
                            self.intl_format_relative_time(&locale, &options, &value, &unit)?,
                        )
                    }
                    "formatToParts" => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        let unit = args.get(1).cloned().unwrap_or(Value::Undefined);
                        let parts = self
                            .intl_format_relative_time_to_parts(&locale, &options, &value, &unit)?;
                        self.intl_relative_time_parts_to_value(&parts)
                    }
                    "resolvedOptions" => {
                        self.intl_relative_time_resolved_options_value(locale, &options)
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Intl.RelativeTimeFormat method: {member}"
                        )));
                    }
                })
            }
            "intl_segmenter" => {
                let (locale, options) = self
                    .resolve_intl_segmenter_options(receiver)
                    .map_err(|_| Self::incompatible_receiver_error(family))?;
                Some(match member {
                    "segment" => {
                        let input = args
                            .first()
                            .cloned()
                            .unwrap_or(Value::Undefined)
                            .as_string();
                        let segments = self.intl_segment_input(&locale, &options, &input);
                        self.new_intl_segments_value(segments)
                    }
                    "resolvedOptions" => {
                        self.intl_segmenter_resolved_options_value(locale, &options)
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Intl.Segmenter method: {member}"
                        )));
                    }
                })
            }
            _ => None,
        };
        Ok(value)
    }
}
