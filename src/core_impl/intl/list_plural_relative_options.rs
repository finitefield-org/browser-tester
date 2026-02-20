use super::*;

impl Harness {
    pub(crate) fn intl_list_separator_before_last(
        locale: &str,
        options: &IntlListOptions,
        list_len: usize,
    ) -> String {
        if options.list_type == "unit" {
            if options.style == "narrow" {
                return " ".to_string();
            }
            return ", ".to_string();
        }

        let family = Self::intl_locale_family(locale);
        let region = Self::intl_locale_region(locale);
        let oxford = family == "en" && region != Some("GB") && list_len > 2;

        match options.list_type.as_str() {
            "conjunction" => match family {
                "de" => " und ".to_string(),
                "en" => {
                    if oxford {
                        ", and ".to_string()
                    } else {
                        " and ".to_string()
                    }
                }
                _ => " and ".to_string(),
            },
            "disjunction" => match family {
                "de" => " oder ".to_string(),
                "en" => {
                    if oxford {
                        ", or ".to_string()
                    } else {
                        " or ".to_string()
                    }
                }
                _ => " or ".to_string(),
            },
            _ => ", ".to_string(),
        }
    }

    pub(crate) fn intl_format_list_to_parts(
        &self,
        locale: &str,
        options: &IntlListOptions,
        value: &Value,
    ) -> Result<Vec<IntlPart>> {
        let items = self.array_like_values_from_value(value)?;
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut parts = Vec::new();
        let len = items.len();
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                let literal = if options.list_type == "unit" && options.style == "narrow" {
                    " ".to_string()
                } else if index + 1 == len {
                    Self::intl_list_separator_before_last(locale, options, len)
                } else {
                    ", ".to_string()
                };
                parts.push(IntlPart {
                    part_type: "literal".to_string(),
                    value: literal,
                });
            }
            parts.push(IntlPart {
                part_type: "element".to_string(),
                value: item.as_string(),
            });
        }
        Ok(parts)
    }

    pub(crate) fn intl_format_list(
        &self,
        locale: &str,
        options: &IntlListOptions,
        value: &Value,
    ) -> Result<String> {
        Ok(self
            .intl_format_list_to_parts(locale, options, value)?
            .into_iter()
            .map(|part| part.value)
            .collect::<String>())
    }

    pub(crate) fn intl_list_resolved_options_value(
        &self,
        locale: String,
        options: &IntlListOptions,
    ) -> Value {
        Self::new_object_value(vec![
            ("locale".to_string(), Value::String(locale)),
            ("style".to_string(), Value::String(options.style.clone())),
            ("type".to_string(), Value::String(options.list_type.clone())),
        ])
    }

    pub(crate) fn intl_plural_rules_options_from_value(
        &self,
        options: Option<&Value>,
    ) -> Result<IntlPluralRulesOptions> {
        let mut rule_type = "cardinal".to_string();
        let Some(options) = options else {
            return Ok(IntlPluralRulesOptions { rule_type });
        };

        match options {
            Value::Undefined | Value::Null => {}
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(value) = Self::object_get_entry(&entries, "type") {
                    if !matches!(value, Value::Undefined) {
                        let parsed = value.as_string();
                        if !matches!(parsed.as_str(), "cardinal" | "ordinal") {
                            return Err(Error::ScriptRuntime(
                                "RangeError: invalid Intl.PluralRules type option".into(),
                            ));
                        }
                        rule_type = parsed;
                    }
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Intl.PluralRules options must be an object".into(),
                ));
            }
        }

        Ok(IntlPluralRulesOptions { rule_type })
    }

    pub(crate) fn intl_plural_rules_options_to_value(options: &IntlPluralRulesOptions) -> Value {
        Self::new_object_value(vec![(
            "type".to_string(),
            Value::String(options.rule_type.clone()),
        )])
    }

    pub(crate) fn intl_plural_rules_options_from_internal(
        entries: &[(String, Value)],
    ) -> IntlPluralRulesOptions {
        if let Some(Value::Object(options)) =
            Self::object_get_entry(entries, INTERNAL_INTL_OPTIONS_KEY)
        {
            let options = options.borrow();
            if let Some(Value::String(rule_type)) = Self::object_get_entry(&options, "type") {
                return IntlPluralRulesOptions { rule_type };
            }
        }
        IntlPluralRulesOptions {
            rule_type: "cardinal".to_string(),
        }
    }

    pub(crate) fn intl_plural_rules_categories(locale: &str, rule_type: &str) -> Vec<String> {
        let family = Self::intl_locale_family(locale);
        match (family, rule_type) {
            ("ar", "cardinal") => vec![
                "zero".to_string(),
                "one".to_string(),
                "two".to_string(),
                "few".to_string(),
                "many".to_string(),
                "other".to_string(),
            ],
            ("en", "ordinal") => vec![
                "one".to_string(),
                "two".to_string(),
                "few".to_string(),
                "other".to_string(),
            ],
            (_, "ordinal") => vec!["other".to_string()],
            _ => vec!["one".to_string(), "other".to_string()],
        }
    }

    pub(crate) fn intl_plural_rules_select_tag(
        locale: &str,
        rule_type: &str,
        number: f64,
    ) -> String {
        if !number.is_finite() {
            return "other".to_string();
        }
        let n = number.abs();
        let integer = n.fract() == 0.0;
        let i = if integer { n as i64 } else { 0 };
        let family = Self::intl_locale_family(locale);

        if rule_type == "ordinal" {
            if family == "en" {
                if !integer {
                    return "other".to_string();
                }
                let mod10 = i % 10;
                let mod100 = i % 100;
                if mod10 == 1 && mod100 != 11 {
                    return "one".to_string();
                }
                if mod10 == 2 && mod100 != 12 {
                    return "two".to_string();
                }
                if mod10 == 3 && mod100 != 13 {
                    return "few".to_string();
                }
            }
            return "other".to_string();
        }

        if family == "ar" {
            if !integer {
                return "other".to_string();
            }
            if i == 0 {
                return "zero".to_string();
            }
            if i == 1 {
                return "one".to_string();
            }
            if i == 2 {
                return "two".to_string();
            }
            let mod100 = i % 100;
            if (3..=10).contains(&mod100) {
                return "few".to_string();
            }
            if (11..=99).contains(&mod100) {
                return "many".to_string();
            }
            return "other".to_string();
        }

        if integer && i == 1 {
            "one".to_string()
        } else {
            "other".to_string()
        }
    }

    pub(crate) fn intl_plural_rules_select(
        &self,
        locale: &str,
        options: &IntlPluralRulesOptions,
        value: &Value,
    ) -> String {
        let number = Self::coerce_number_for_global(value);
        Self::intl_plural_rules_select_tag(locale, &options.rule_type, number)
    }

    pub(crate) fn intl_plural_rules_select_range(
        &self,
        locale: &str,
        options: &IntlPluralRulesOptions,
        start: &Value,
        end: &Value,
    ) -> String {
        let start_number = Self::coerce_number_for_global(start);
        let end_number = Self::coerce_number_for_global(end);
        if !start_number.is_finite() || !end_number.is_finite() {
            return "other".to_string();
        }
        if start_number == end_number {
            return Self::intl_plural_rules_select_tag(locale, &options.rule_type, start_number);
        }
        let start_tag =
            Self::intl_plural_rules_select_tag(locale, &options.rule_type, start_number);
        let end_tag = Self::intl_plural_rules_select_tag(locale, &options.rule_type, end_number);
        if start_tag == end_tag {
            start_tag
        } else {
            "other".to_string()
        }
    }

    pub(crate) fn intl_plural_rules_resolved_options_value(
        &self,
        locale: String,
        options: &IntlPluralRulesOptions,
    ) -> Value {
        let categories = Self::intl_plural_rules_categories(&locale, &options.rule_type);
        Self::new_object_value(vec![
            ("locale".to_string(), Value::String(locale)),
            ("type".to_string(), Value::String(options.rule_type.clone())),
            (
                "pluralCategories".to_string(),
                Self::new_array_value(
                    categories
                        .into_iter()
                        .map(Value::String)
                        .collect::<Vec<_>>(),
                ),
            ),
        ])
    }

    pub(crate) fn intl_relative_time_options_from_value(
        &self,
        options: Option<&Value>,
    ) -> Result<IntlRelativeTimeOptions> {
        let mut style = "long".to_string();
        let mut numeric = "always".to_string();
        let mut locale_matcher = "best fit".to_string();

        let Some(options) = options else {
            return Ok(IntlRelativeTimeOptions {
                style,
                numeric,
                locale_matcher,
            });
        };

        match options {
            Value::Undefined | Value::Null => {}
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(value) = Self::object_get_entry(&entries, "style") {
                    if !matches!(value, Value::Undefined) {
                        let parsed = value.as_string();
                        if !matches!(parsed.as_str(), "long" | "short" | "narrow") {
                            return Err(Error::ScriptRuntime(
                                "RangeError: invalid Intl.RelativeTimeFormat style option".into(),
                            ));
                        }
                        style = parsed;
                    }
                }
                if let Some(value) = Self::object_get_entry(&entries, "numeric") {
                    if !matches!(value, Value::Undefined) {
                        let parsed = value.as_string();
                        if !matches!(parsed.as_str(), "always" | "auto") {
                            return Err(Error::ScriptRuntime(
                                "RangeError: invalid Intl.RelativeTimeFormat numeric option".into(),
                            ));
                        }
                        numeric = parsed;
                    }
                }
                if let Some(value) = Self::object_get_entry(&entries, "localeMatcher") {
                    if !matches!(value, Value::Undefined) {
                        let parsed = value.as_string();
                        if !matches!(parsed.as_str(), "lookup" | "best fit") {
                            return Err(Error::ScriptRuntime(
                                "RangeError: invalid Intl.RelativeTimeFormat localeMatcher option"
                                    .into(),
                            ));
                        }
                        locale_matcher = parsed;
                    }
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Intl.RelativeTimeFormat options must be an object".into(),
                ));
            }
        }

        Ok(IntlRelativeTimeOptions {
            style,
            numeric,
            locale_matcher,
        })
    }

    pub(crate) fn intl_relative_time_options_to_value(options: &IntlRelativeTimeOptions) -> Value {
        Self::new_object_value(vec![
            ("style".to_string(), Value::String(options.style.clone())),
            (
                "numeric".to_string(),
                Value::String(options.numeric.clone()),
            ),
            (
                "localeMatcher".to_string(),
                Value::String(options.locale_matcher.clone()),
            ),
        ])
    }

    pub(crate) fn intl_relative_time_options_from_internal(
        entries: &[(String, Value)],
    ) -> IntlRelativeTimeOptions {
        if let Some(Value::Object(options)) =
            Self::object_get_entry(entries, INTERNAL_INTL_OPTIONS_KEY)
        {
            let options = options.borrow();
            let style = match Self::object_get_entry(&options, "style") {
                Some(Value::String(value)) => value,
                _ => "long".to_string(),
            };
            let numeric = match Self::object_get_entry(&options, "numeric") {
                Some(Value::String(value)) => value,
                _ => "always".to_string(),
            };
            let locale_matcher = match Self::object_get_entry(&options, "localeMatcher") {
                Some(Value::String(value)) => value,
                _ => "best fit".to_string(),
            };
            return IntlRelativeTimeOptions {
                style,
                numeric,
                locale_matcher,
            };
        }

        IntlRelativeTimeOptions {
            style: "long".to_string(),
            numeric: "always".to_string(),
            locale_matcher: "best fit".to_string(),
        }
    }
}
