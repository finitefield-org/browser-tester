use super::*;

impl Harness {
    pub(crate) fn intl_locale_prepend_unique(values: &mut Vec<String>, preferred: Option<&str>) {
        let Some(preferred) = preferred else {
            return;
        };
        if let Some(idx) = values.iter().position(|value| value == preferred) {
            if idx != 0 {
                let value = values.remove(idx);
                values.insert(0, value);
            }
            return;
        }
        values.insert(0, preferred.to_string());
    }

    pub(crate) fn intl_locale_get_calendars(&self, data: &IntlLocaleData) -> Vec<String> {
        let mut out = match data.language.as_str() {
            "ja" => vec!["gregory".to_string(), "japanese".to_string()],
            "ar" => vec!["gregory".to_string(), "islamic-umalqura".to_string()],
            _ => vec!["gregory".to_string()],
        };
        Self::intl_locale_prepend_unique(&mut out, data.calendar.as_deref());
        out
    }

    pub(crate) fn intl_locale_get_collations(&self, data: &IntlLocaleData) -> Vec<String> {
        let mut out = if data.language == "de" {
            vec![
                "default".to_string(),
                "phonebk".to_string(),
                "emoji".to_string(),
            ]
        } else {
            vec!["default".to_string(), "emoji".to_string()]
        };
        Self::intl_locale_prepend_unique(&mut out, data.collation.as_deref());
        out
    }

    pub(crate) fn intl_locale_get_hour_cycles(&self, data: &IntlLocaleData) -> Vec<String> {
        let mut out = if data.language == "en" || data.language == "ar" {
            vec!["h12".to_string(), "h23".to_string()]
        } else {
            vec!["h23".to_string(), "h12".to_string()]
        };
        Self::intl_locale_prepend_unique(&mut out, data.hour_cycle.as_deref());
        out
    }

    pub(crate) fn intl_locale_get_numbering_systems(&self, data: &IntlLocaleData) -> Vec<String> {
        let mut out = if data.language == "ar" {
            vec!["arab".to_string(), "latn".to_string()]
        } else {
            vec!["latn".to_string()]
        };
        Self::intl_locale_prepend_unique(&mut out, data.numbering_system.as_deref());
        out
    }

    pub(crate) fn intl_locale_get_text_info(&self, data: &IntlLocaleData) -> Value {
        let direction = if matches!(data.language.as_str(), "ar" | "he" | "fa" | "ur") {
            "rtl"
        } else {
            "ltr"
        };
        Self::new_object_value(vec![(
            "direction".to_string(),
            Value::String(direction.to_string()),
        )])
    }

    pub(crate) fn intl_locale_get_time_zones(&self, data: &IntlLocaleData) -> Vec<String> {
        match data.region.as_deref() {
            Some("US") => vec![
                "America/New_York".to_string(),
                "America/Los_Angeles".to_string(),
            ],
            Some("JP") => vec!["Asia/Tokyo".to_string()],
            Some("KR") => vec!["Asia/Seoul".to_string()],
            Some("GB") => vec!["Europe/London".to_string()],
            Some("DE") => vec!["Europe/Berlin".to_string()],
            Some("FR") => vec!["Europe/Paris".to_string()],
            Some("CN") => vec!["Asia/Shanghai".to_string()],
            Some("EG") => vec!["Africa/Cairo".to_string()],
            Some("IL") => vec!["Asia/Jerusalem".to_string()],
            _ => match data.language.as_str() {
                "ja" => vec!["Asia/Tokyo".to_string()],
                "ko" => vec!["Asia/Seoul".to_string()],
                "de" => vec!["Europe/Berlin".to_string()],
                "fr" => vec!["Europe/Paris".to_string()],
                _ => vec!["UTC".to_string()],
            },
        }
    }

    pub(crate) fn intl_locale_get_week_info(&self, data: &IntlLocaleData) -> Value {
        let (first_day, weekend, minimal_days) = match data.region.as_deref() {
            Some("US") => (7, vec![6, 7], 1),
            Some("EG") => (6, vec![5, 6], 1),
            _ => (1, vec![6, 7], 4),
        };
        Self::new_object_value(vec![
            ("firstDay".to_string(), Value::Number(first_day)),
            (
                "weekend".to_string(),
                Self::new_array_value(weekend.into_iter().map(Value::Number).collect::<Vec<_>>()),
            ),
            ("minimalDays".to_string(), Value::Number(minimal_days)),
        ])
    }

    pub(crate) fn intl_locale_likely_subtags(
        language: &str,
    ) -> (Option<&'static str>, Option<&'static str>) {
        match language {
            "en" => (Some("Latn"), Some("US")),
            "de" => (Some("Latn"), Some("DE")),
            "fr" => (Some("Latn"), Some("FR")),
            "ja" => (Some("Jpan"), Some("JP")),
            "ko" => (Some("Kore"), Some("KR")),
            "zh" => (Some("Hans"), Some("CN")),
            "ar" => (Some("Arab"), Some("EG")),
            "he" => (Some("Hebr"), Some("IL")),
            "pt" => (Some("Latn"), Some("BR")),
            "sv" => (Some("Latn"), Some("SE")),
            "id" => (Some("Latn"), Some("ID")),
            _ => (None, None),
        }
    }

    pub(crate) fn intl_locale_maximize_data(&self, data: &IntlLocaleData) -> IntlLocaleData {
        let mut out = data.clone();
        let (default_script, default_region) = Self::intl_locale_likely_subtags(&out.language);
        if out.script.is_none() {
            out.script = default_script.map(str::to_string);
        }
        if out.region.is_none() {
            out.region = default_region.map(str::to_string);
        }
        out
    }

    pub(crate) fn intl_locale_minimize_data(&self, data: &IntlLocaleData) -> IntlLocaleData {
        let mut out = data.clone();
        let (default_script, default_region) = Self::intl_locale_likely_subtags(&out.language);
        if out.script.as_deref() == default_script {
            out.script = None;
        }
        if out.region.as_deref() == default_region {
            out.region = None;
        }
        out
    }

    pub(crate) fn intl_display_names_options_from_value(
        &self,
        options: Option<&Value>,
    ) -> Result<IntlDisplayNamesOptions> {
        let Some(options) = options else {
            return Err(Error::ScriptRuntime(
                "TypeError: Intl.DisplayNames options with a type are required".into(),
            ));
        };

        let entries = match options {
            Value::Object(entries) => entries.borrow(),
            Value::Undefined | Value::Null => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Intl.DisplayNames options with a type are required".into(),
                ));
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Intl.DisplayNames options must be an object".into(),
                ));
            }
        };

        let string_option = |key: &str| -> Option<String> {
            match Self::object_get_entry(&entries, key) {
                Some(Value::Undefined) | None => None,
                Some(value) => Some(value.as_string()),
            }
        };

        let display_type = string_option("type").ok_or_else(|| {
            Error::ScriptRuntime("TypeError: Intl.DisplayNames requires a type option".into())
        })?;
        if !matches!(
            display_type.as_str(),
            "region" | "language" | "script" | "currency"
        ) {
            return Err(Error::ScriptRuntime(
                "RangeError: invalid Intl.DisplayNames type option".into(),
            ));
        }

        let style = string_option("style").unwrap_or_else(|| "long".to_string());
        if !matches!(style.as_str(), "narrow" | "short" | "long") {
            return Err(Error::ScriptRuntime(
                "RangeError: invalid Intl.DisplayNames style option".into(),
            ));
        }

        let fallback = string_option("fallback").unwrap_or_else(|| "code".to_string());
        if !matches!(fallback.as_str(), "code" | "none") {
            return Err(Error::ScriptRuntime(
                "RangeError: invalid Intl.DisplayNames fallback option".into(),
            ));
        }

        let language_display =
            string_option("languageDisplay").unwrap_or_else(|| "dialect".to_string());
        if !matches!(language_display.as_str(), "dialect" | "standard") {
            return Err(Error::ScriptRuntime(
                "RangeError: invalid Intl.DisplayNames languageDisplay option".into(),
            ));
        }

        Ok(IntlDisplayNamesOptions {
            style,
            display_type,
            fallback,
            language_display,
        })
    }

    pub(crate) fn intl_display_names_options_to_value(options: &IntlDisplayNamesOptions) -> Value {
        let mut entries = vec![
            ("style".to_string(), Value::String(options.style.clone())),
            (
                "type".to_string(),
                Value::String(options.display_type.clone()),
            ),
            (
                "fallback".to_string(),
                Value::String(options.fallback.clone()),
            ),
        ];
        if options.display_type == "language" {
            entries.push((
                "languageDisplay".to_string(),
                Value::String(options.language_display.clone()),
            ));
        }
        Self::new_object_value(entries)
    }

    pub(crate) fn intl_display_names_options_from_internal(
        entries: &[(String, Value)],
    ) -> IntlDisplayNamesOptions {
        if let Some(Value::Object(options)) =
            Self::object_get_entry(entries, INTERNAL_INTL_OPTIONS_KEY)
        {
            let options = options.borrow();
            let string_option = |key: &str| -> Option<String> {
                match Self::object_get_entry(&options, key) {
                    Some(Value::String(value)) => Some(value),
                    _ => None,
                }
            };
            return IntlDisplayNamesOptions {
                style: string_option("style").unwrap_or_else(|| "long".to_string()),
                display_type: string_option("type").unwrap_or_else(|| "region".to_string()),
                fallback: string_option("fallback").unwrap_or_else(|| "code".to_string()),
                language_display: string_option("languageDisplay")
                    .unwrap_or_else(|| "dialect".to_string()),
            };
        }
        IntlDisplayNamesOptions {
            style: "long".to_string(),
            display_type: "region".to_string(),
            fallback: "code".to_string(),
            language_display: "dialect".to_string(),
        }
    }

    pub(crate) fn intl_canonicalize_display_names_code(
        display_type: &str,
        code: &str,
    ) -> Result<String> {
        let code = code.trim();
        if code.is_empty() {
            return Err(Error::ScriptRuntime(
                "RangeError: invalid Intl.DisplayNames code".into(),
            ));
        }
        match display_type {
            "region" => {
                if code.len() == 2 && code.chars().all(|ch| ch.is_ascii_alphabetic()) {
                    Ok(code.to_ascii_uppercase())
                } else if code.len() == 3 && code.chars().all(|ch| ch.is_ascii_digit()) {
                    Ok(code.to_string())
                } else {
                    Err(Error::ScriptRuntime(
                        "RangeError: invalid region code for Intl.DisplayNames".into(),
                    ))
                }
            }
            "script" => {
                if code.len() == 4 && code.chars().all(|ch| ch.is_ascii_alphabetic()) {
                    let mut chars = code.chars();
                    let first = chars.next().unwrap_or_default().to_ascii_uppercase();
                    Ok(format!("{first}{}", chars.as_str().to_ascii_lowercase()))
                } else {
                    Err(Error::ScriptRuntime(
                        "RangeError: invalid script code for Intl.DisplayNames".into(),
                    ))
                }
            }
            "currency" => {
                if code.len() == 3 && code.chars().all(|ch| ch.is_ascii_alphabetic()) {
                    Ok(code.to_ascii_uppercase())
                } else {
                    Err(Error::ScriptRuntime(
                        "RangeError: invalid currency code for Intl.DisplayNames".into(),
                    ))
                }
            }
            "language" => Self::intl_canonicalize_locale(code),
            _ => Err(Error::ScriptRuntime(
                "RangeError: invalid Intl.DisplayNames type option".into(),
            )),
        }
    }
}
