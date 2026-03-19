use super::super::*;

impl Harness {
    pub(crate) fn resolve_number_to_locale_string_locale(
        &self,
        locale_arg: Option<&Value>,
    ) -> Result<String> {
        let requested = if let Some(locale_arg) = locale_arg {
            self.intl_collect_locales(locale_arg)?
        } else {
            Vec::new()
        };
        Ok(Self::intl_select_locale_for_formatter(
            IntlFormatterKind::NumberFormat,
            &requested,
        ))
    }

    pub(crate) fn parse_number_to_locale_string_fraction_digits(
        options_arg: Option<&Value>,
    ) -> Result<(Option<usize>, Option<usize>)> {
        let Some(options_arg) = options_arg else {
            return Ok((None, None));
        };
        match options_arg {
            Value::Undefined | Value::Null => Ok((None, None)),
            Value::Object(options) => {
                let options = options.borrow();
                let minimum = Self::parse_fraction_digits_option(
                    &options,
                    "minimumFractionDigits",
                    "minimumFractionDigits must be between 0 and 100",
                )?;
                let maximum = Self::parse_fraction_digits_option(
                    &options,
                    "maximumFractionDigits",
                    "maximumFractionDigits must be between 0 and 100",
                )?;
                if minimum.zip(maximum).is_some_and(|(min, max)| min > max) {
                    return Err(Error::ScriptRuntime(
                        "minimumFractionDigits cannot be greater than maximumFractionDigits".into(),
                    ));
                }
                Ok((minimum, maximum))
            }
            _ => Ok((None, None)),
        }
    }

    pub(crate) fn intl_number_format_options_from_value(
        &self,
        locale: &str,
        options_arg: Option<&Value>,
    ) -> Result<IntlNumberFormatOptions> {
        let mut options = IntlNumberFormatOptions {
            style: "decimal".to_string(),
            currency: None,
            unit: None,
            unit_display: "short".to_string(),
            numbering_system: Self::intl_locale_unicode_extension_value(locale, "nu")
                .unwrap_or_else(|| Self::intl_default_numbering_system_for_locale(locale)),
            minimum_fraction_digits: None,
            maximum_fraction_digits: None,
            maximum_significant_digits: None,
        };

        let Some(options_arg) = options_arg else {
            return Ok(options);
        };

        match options_arg {
            Value::Undefined | Value::Null => Ok(options),
            Value::Object(raw_options) => {
                let raw_options = raw_options.borrow();
                let string_option = |key: &str| -> Option<String> {
                    match Self::object_get_entry(&raw_options, key) {
                        Some(Value::Undefined) | None => None,
                        Some(value) => Some(value.as_string()),
                    }
                };

                if let Some(style) = string_option("style") {
                    if !matches!(style.as_str(), "decimal" | "currency" | "unit") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.NumberFormat style option".into(),
                        ));
                    }
                    options.style = style;
                }

                if let Some(currency) = string_option("currency") {
                    let normalized = currency.trim().to_ascii_uppercase();
                    if normalized.len() != 3
                        || !normalized.chars().all(|ch| ch.is_ascii_alphabetic())
                    {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.NumberFormat currency option".into(),
                        ));
                    }
                    options.currency = Some(normalized);
                }

                if let Some(unit) = string_option("unit") {
                    options.unit = Some(unit);
                }

                if let Some(unit_display) = string_option("unitDisplay") {
                    if !matches!(unit_display.as_str(), "short" | "long" | "narrow") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.NumberFormat unitDisplay option".into(),
                        ));
                    }
                    options.unit_display = unit_display;
                }

                if let Some(numbering_system) = string_option("numberingSystem") {
                    if numbering_system.trim().is_empty() {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.NumberFormat numberingSystem option".into(),
                        ));
                    }
                    options.numbering_system = numbering_system.to_ascii_lowercase();
                }

                let (minimum_fraction_digits, maximum_fraction_digits) =
                    Self::parse_number_to_locale_string_fraction_digits(Some(options_arg))?;
                options.minimum_fraction_digits = minimum_fraction_digits;
                options.maximum_fraction_digits = maximum_fraction_digits;

                options.maximum_significant_digits = Self::parse_significant_digits_option(
                    &raw_options,
                    "maximumSignificantDigits",
                    "maximumSignificantDigits must be between 1 and 21",
                )?;

                if options.style == "currency" {
                    if options.currency.is_none() {
                        return Err(Error::ScriptRuntime(
                            "TypeError: Intl.NumberFormat currency style requires a currency option"
                                .into(),
                        ));
                    }
                    let currency_digits = Self::intl_currency_default_fraction_digits(
                        options.currency.as_deref().unwrap_or("USD"),
                    );
                    if options.minimum_fraction_digits.is_none() {
                        options.minimum_fraction_digits = Some(currency_digits);
                    }
                    if options.maximum_fraction_digits.is_none() {
                        options.maximum_fraction_digits = Some(currency_digits);
                    }
                }

                if options.style == "unit" && options.unit.is_none() {
                    return Err(Error::ScriptRuntime(
                        "TypeError: Intl.NumberFormat unit style requires a unit option".into(),
                    ));
                }

                Ok(options)
            }
            _ => Err(Error::ScriptRuntime(
                "TypeError: Intl.NumberFormat options must be an object".into(),
            )),
        }
    }

    fn parse_fraction_digits_option(
        options: &ObjectValue,
        key: &str,
        out_of_range_message: &str,
    ) -> Result<Option<usize>> {
        let Some(value) = Self::object_get_entry(options, key) else {
            return Ok(None);
        };
        if matches!(value, Value::Undefined) {
            return Ok(None);
        }
        let digits = Self::value_to_i64(&value);
        if !(0..=100).contains(&digits) {
            return Err(Error::ScriptRuntime(out_of_range_message.into()));
        }
        Ok(Some(digits as usize))
    }

    fn parse_significant_digits_option(
        options: &ObjectValue,
        key: &str,
        out_of_range_message: &str,
    ) -> Result<Option<usize>> {
        let Some(value) = Self::object_get_entry(options, key) else {
            return Ok(None);
        };
        if matches!(value, Value::Undefined) {
            return Ok(None);
        }
        let digits = Self::value_to_i64(&value);
        if !(1..=21).contains(&digits) {
            return Err(Error::ScriptRuntime(out_of_range_message.into()));
        }
        Ok(Some(digits as usize))
    }

    pub(crate) fn intl_currency_default_fraction_digits(currency: &str) -> usize {
        match currency {
            "JPY" => 0,
            _ => 2,
        }
    }

    pub(crate) fn intl_currency_symbol(currency: &str) -> String {
        match currency {
            "EUR" => "€".to_string(),
            "JPY" => "￥".to_string(),
            "USD" => "$".to_string(),
            _ => currency.to_string(),
        }
    }

    pub(crate) fn intl_unit_label(
        locale: &str,
        unit: &str,
        unit_display: &str,
        value: f64,
    ) -> Result<String> {
        let singular = value.abs() == 1.0;
        let family = Self::intl_locale_family(locale);
        match unit {
            "kilometer-per-hour" => Ok(match unit_display {
                "long" => "kilometers per hour".to_string(),
                "short" | "narrow" => "km/h".to_string(),
                _ => {
                    return Err(Error::ScriptRuntime(
                        "RangeError: invalid Intl.NumberFormat unitDisplay option".into(),
                    ));
                }
            }),
            "liter" => Ok(match unit_display {
                "long" => {
                    if family == "en" && Self::intl_locale_region(locale) == Some("GB") {
                        if singular {
                            "litre".to_string()
                        } else {
                            "litres".to_string()
                        }
                    } else if singular {
                        "liter".to_string()
                    } else {
                        "liters".to_string()
                    }
                }
                "short" | "narrow" => "L".to_string(),
                _ => {
                    return Err(Error::ScriptRuntime(
                        "RangeError: invalid Intl.NumberFormat unitDisplay option".into(),
                    ));
                }
            }),
            _ => Err(Error::ScriptRuntime(
                "RangeError: invalid Intl.NumberFormat unit option".into(),
            )),
        }
    }
}
