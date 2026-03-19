use super::*;

impl Harness {
    fn eval_builtin_intl_locale_construct(
        &mut self,
        tag: &Expr,
        options: Option<&Expr>,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let tag = self.eval_expr(tag, env, event_param, event)?;
        let options = options
            .map(|value| self.eval_expr(value, env, event_param, event))
            .transpose()?;
        let data = self.intl_locale_data_from_input_value(&tag, options.as_ref())?;
        Ok(self.new_intl_locale_value(data))
    }

    fn eval_intl_locale_construct(
        &mut self,
        tag: &Expr,
        options: Option<&Expr>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let (intl_value, constructor) =
            self.current_intl_constructor_value("Locale", env, event_param, event)?;
        if Self::is_builtin_placeholder_value(&constructor) {
            return self.eval_builtin_intl_locale_construct(tag, options, env, event_param, event);
        }

        let mut args = Vec::with_capacity(2);
        args.push(self.eval_expr(tag, env, event_param, event)?);
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

    pub(crate) fn try_eval_core_intl_locale_static_exprs(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
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
                        let supported = Self::intl_supported_locales(
                            IntlFormatterKind::DateTimeFormat,
                            locales,
                        );
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
                        let supported = Self::intl_supported_locales(
                            IntlFormatterKind::DurationFormat,
                            locales,
                        );
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
                    IntlStaticMethod::NumberFormatSupportedLocalesOf => {
                        if args.is_empty() || args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "Intl.NumberFormat.supportedLocalesOf requires locales and optional options"
                                    .into(),
                            ));
                        }
                        let locales = self.eval_expr(&args[0], env, event_param, event)?;
                        let locales = self.intl_collect_locales(&locales)?;
                        let supported =
                            Self::intl_supported_locales(IntlFormatterKind::NumberFormat, locales);
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
                    called_with_new,
                } => self.eval_intl_locale_construct(
                    tag,
                    options.as_deref(),
                    *called_with_new,
                    env,
                    event_param,
                    event,
                ),
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
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
