use super::*;

impl Harness {
    pub(crate) fn intl_relative_time_normalize_unit(unit: &str) -> Option<String> {
        let unit = unit.trim().to_ascii_lowercase();
        let canonical = match unit.as_str() {
            "year" | "years" => "year",
            "quarter" | "quarters" => "quarter",
            "month" | "months" => "month",
            "week" | "weeks" => "week",
            "day" | "days" => "day",
            "hour" | "hours" => "hour",
            "minute" | "minutes" => "minute",
            "second" | "seconds" => "second",
            _ => return None,
        };
        Some(canonical.to_string())
    }

    pub(crate) fn intl_relative_time_auto_literal(
        locale: &str,
        unit: &str,
        value: f64,
    ) -> Option<String> {
        if unit != "day" || !value.is_finite() || value.fract() != 0.0 {
            return None;
        }

        let day = value as i64;
        match Self::intl_locale_family(locale) {
            "es" => match day {
                -2 => Some("anteayer".to_string()),
                -1 => Some("ayer".to_string()),
                0 => Some("hoy".to_string()),
                1 => Some("mañana".to_string()),
                2 => Some("pasado mañana".to_string()),
                _ => None,
            },
            _ => match day {
                -1 => Some("yesterday".to_string()),
                0 => Some("today".to_string()),
                1 => Some("tomorrow".to_string()),
                _ => None,
            },
        }
    }

    pub(crate) fn intl_relative_time_unit_label(
        locale: &str,
        style: &str,
        unit: &str,
        value: f64,
    ) -> String {
        let singular = value.abs() == 1.0;
        match Self::intl_locale_family(locale) {
            "es" => match style {
                "short" => match unit {
                    "year" => "a.".to_string(),
                    "quarter" => "trim.".to_string(),
                    "month" => "mes".to_string(),
                    "week" => "sem.".to_string(),
                    "day" => "d.".to_string(),
                    "hour" => "h".to_string(),
                    "minute" => "min".to_string(),
                    "second" => "s".to_string(),
                    _ => unit.to_string(),
                },
                "narrow" => match unit {
                    "year" => "a".to_string(),
                    "quarter" => "trim".to_string(),
                    "month" => "m".to_string(),
                    "week" => "sem".to_string(),
                    "day" => "d".to_string(),
                    "hour" => "h".to_string(),
                    "minute" => "min".to_string(),
                    "second" => "s".to_string(),
                    _ => unit.to_string(),
                },
                _ => match unit {
                    "year" => {
                        if singular {
                            "año".to_string()
                        } else {
                            "años".to_string()
                        }
                    }
                    "quarter" => {
                        if singular {
                            "trimestre".to_string()
                        } else {
                            "trimestres".to_string()
                        }
                    }
                    "month" => {
                        if singular {
                            "mes".to_string()
                        } else {
                            "meses".to_string()
                        }
                    }
                    "week" => {
                        if singular {
                            "semana".to_string()
                        } else {
                            "semanas".to_string()
                        }
                    }
                    "day" => {
                        if singular {
                            "día".to_string()
                        } else {
                            "días".to_string()
                        }
                    }
                    "hour" => {
                        if singular {
                            "hora".to_string()
                        } else {
                            "horas".to_string()
                        }
                    }
                    "minute" => {
                        if singular {
                            "minuto".to_string()
                        } else {
                            "minutos".to_string()
                        }
                    }
                    "second" => {
                        if singular {
                            "segundo".to_string()
                        } else {
                            "segundos".to_string()
                        }
                    }
                    _ => unit.to_string(),
                },
            },
            _ => match style {
                "short" => match unit {
                    "year" => {
                        if singular {
                            "yr.".to_string()
                        } else {
                            "yrs.".to_string()
                        }
                    }
                    "quarter" => {
                        if singular {
                            "qtr.".to_string()
                        } else {
                            "qtrs.".to_string()
                        }
                    }
                    "month" => {
                        if singular {
                            "mo.".to_string()
                        } else {
                            "mos.".to_string()
                        }
                    }
                    "week" => {
                        if singular {
                            "wk.".to_string()
                        } else {
                            "wks.".to_string()
                        }
                    }
                    "day" => {
                        if singular {
                            "day".to_string()
                        } else {
                            "days".to_string()
                        }
                    }
                    "hour" => {
                        if singular {
                            "hr.".to_string()
                        } else {
                            "hrs.".to_string()
                        }
                    }
                    "minute" => "min.".to_string(),
                    "second" => "sec.".to_string(),
                    _ => unit.to_string(),
                },
                "narrow" => match unit {
                    "year" => "y".to_string(),
                    "quarter" => "q".to_string(),
                    "month" => "mo".to_string(),
                    "week" => "w".to_string(),
                    "day" => "d".to_string(),
                    "hour" => "h".to_string(),
                    "minute" => "m".to_string(),
                    "second" => "s".to_string(),
                    _ => unit.to_string(),
                },
                _ => match unit {
                    "year" => {
                        if singular {
                            "year".to_string()
                        } else {
                            "years".to_string()
                        }
                    }
                    "quarter" => {
                        if singular {
                            "quarter".to_string()
                        } else {
                            "quarters".to_string()
                        }
                    }
                    "month" => {
                        if singular {
                            "month".to_string()
                        } else {
                            "months".to_string()
                        }
                    }
                    "week" => {
                        if singular {
                            "week".to_string()
                        } else {
                            "weeks".to_string()
                        }
                    }
                    "day" => {
                        if singular {
                            "day".to_string()
                        } else {
                            "days".to_string()
                        }
                    }
                    "hour" => {
                        if singular {
                            "hour".to_string()
                        } else {
                            "hours".to_string()
                        }
                    }
                    "minute" => {
                        if singular {
                            "minute".to_string()
                        } else {
                            "minutes".to_string()
                        }
                    }
                    "second" => {
                        if singular {
                            "second".to_string()
                        } else {
                            "seconds".to_string()
                        }
                    }
                    _ => unit.to_string(),
                },
            },
        }
    }

    pub(crate) fn intl_relative_time_parts(
        locale: &str,
        options: &IntlRelativeTimeOptions,
        value: f64,
        unit: &str,
    ) -> Vec<IntlRelativeTimePart> {
        if options.numeric == "auto" {
            if let Some(literal) = Self::intl_relative_time_auto_literal(locale, unit, value) {
                return vec![IntlRelativeTimePart {
                    part_type: "literal".to_string(),
                    value: literal,
                    unit: None,
                }];
            }
        }

        let unit_label = Self::intl_relative_time_unit_label(locale, &options.style, unit, value);
        let numeric = IntlRelativeTimePart {
            part_type: "integer".to_string(),
            value: Self::format_number_default(value.abs()),
            unit: Some(unit.to_string()),
        };

        let family = Self::intl_locale_family(locale);
        if value < 0.0 {
            if family == "es" {
                return vec![
                    IntlRelativeTimePart {
                        part_type: "literal".to_string(),
                        value: "hace ".to_string(),
                        unit: None,
                    },
                    numeric,
                    IntlRelativeTimePart {
                        part_type: "literal".to_string(),
                        value: format!(" {unit_label}"),
                        unit: None,
                    },
                ];
            }

            return vec![
                numeric,
                IntlRelativeTimePart {
                    part_type: "literal".to_string(),
                    value: format!(" {unit_label} ago"),
                    unit: None,
                },
            ];
        }

        if family == "es" {
            return vec![
                IntlRelativeTimePart {
                    part_type: "literal".to_string(),
                    value: "dentro de ".to_string(),
                    unit: None,
                },
                numeric,
                IntlRelativeTimePart {
                    part_type: "literal".to_string(),
                    value: format!(" {unit_label}"),
                    unit: None,
                },
            ];
        }

        vec![
            IntlRelativeTimePart {
                part_type: "literal".to_string(),
                value: "in ".to_string(),
                unit: None,
            },
            numeric,
            IntlRelativeTimePart {
                part_type: "literal".to_string(),
                value: format!(" {unit_label}"),
                unit: None,
            },
        ]
    }

    pub(crate) fn intl_format_relative_time(
        &self,
        locale: &str,
        options: &IntlRelativeTimeOptions,
        value: &Value,
        unit: &Value,
    ) -> Result<String> {
        let parts = self.intl_format_relative_time_to_parts(locale, options, value, unit)?;
        Ok(parts.into_iter().map(|part| part.value).collect::<String>())
    }

    pub(crate) fn intl_format_relative_time_to_parts(
        &self,
        locale: &str,
        options: &IntlRelativeTimeOptions,
        value: &Value,
        unit: &Value,
    ) -> Result<Vec<IntlRelativeTimePart>> {
        let numeric_value = Self::coerce_number_for_global(value);
        let unit_raw = unit.as_string();
        let unit = Self::intl_relative_time_normalize_unit(&unit_raw).ok_or_else(|| {
            Error::ScriptRuntime("RangeError: invalid Intl.RelativeTimeFormat unit argument".into())
        })?;
        Ok(Self::intl_relative_time_parts(
            locale,
            options,
            numeric_value,
            &unit,
        ))
    }

    pub(crate) fn intl_relative_time_parts_to_value(
        &self,
        parts: &[IntlRelativeTimePart],
    ) -> Value {
        let mut out = Vec::with_capacity(parts.len());
        for part in parts {
            let mut entries = vec![
                ("type".to_string(), Value::String(part.part_type.clone())),
                ("value".to_string(), Value::String(part.value.clone())),
            ];
            if let Some(unit) = &part.unit {
                entries.push(("unit".to_string(), Value::String(unit.clone())));
            }
            out.push(Self::new_object_value(entries));
        }
        Self::new_array_value(out)
    }

    pub(crate) fn intl_relative_time_resolved_options_value(
        &self,
        locale: String,
        options: &IntlRelativeTimeOptions,
    ) -> Value {
        Self::new_object_value(vec![
            ("locale".to_string(), Value::String(locale)),
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

    pub(crate) fn intl_segmenter_options_from_value(
        &self,
        options: Option<&Value>,
    ) -> Result<IntlSegmenterOptions> {
        let mut granularity = "grapheme".to_string();
        let mut locale_matcher = "best fit".to_string();
        let Some(options) = options else {
            return Ok(IntlSegmenterOptions {
                granularity,
                locale_matcher,
            });
        };

        match options {
            Value::Undefined | Value::Null => {}
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(value) = Self::object_get_entry(&entries, "granularity") {
                    if !matches!(value, Value::Undefined) {
                        let parsed = value.as_string();
                        if !matches!(parsed.as_str(), "grapheme" | "word" | "sentence") {
                            return Err(Error::ScriptRuntime(
                                "RangeError: invalid Intl.Segmenter granularity option".into(),
                            ));
                        }
                        granularity = parsed;
                    }
                }
                if let Some(value) = Self::object_get_entry(&entries, "localeMatcher") {
                    if !matches!(value, Value::Undefined) {
                        let parsed = value.as_string();
                        if !matches!(parsed.as_str(), "lookup" | "best fit") {
                            return Err(Error::ScriptRuntime(
                                "RangeError: invalid Intl.Segmenter localeMatcher option".into(),
                            ));
                        }
                        locale_matcher = parsed;
                    }
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Intl.Segmenter options must be an object".into(),
                ));
            }
        }

        Ok(IntlSegmenterOptions {
            granularity,
            locale_matcher,
        })
    }

    pub(crate) fn intl_segmenter_options_to_value(options: &IntlSegmenterOptions) -> Value {
        Self::new_object_value(vec![
            (
                "granularity".to_string(),
                Value::String(options.granularity.clone()),
            ),
            (
                "localeMatcher".to_string(),
                Value::String(options.locale_matcher.clone()),
            ),
        ])
    }

    pub(crate) fn intl_segmenter_options_from_internal(
        entries: &[(String, Value)],
    ) -> IntlSegmenterOptions {
        if let Some(Value::Object(options)) =
            Self::object_get_entry(entries, INTERNAL_INTL_OPTIONS_KEY)
        {
            let options = options.borrow();
            let granularity = match Self::object_get_entry(&options, "granularity") {
                Some(Value::String(value)) => value,
                _ => "grapheme".to_string(),
            };
            let locale_matcher = match Self::object_get_entry(&options, "localeMatcher") {
                Some(Value::String(value)) => value,
                _ => "best fit".to_string(),
            };
            return IntlSegmenterOptions {
                granularity,
                locale_matcher,
            };
        }
        IntlSegmenterOptions {
            granularity: "grapheme".to_string(),
            locale_matcher: "best fit".to_string(),
        }
    }

    pub(crate) fn intl_segmenter_is_japanese_char(ch: char) -> bool {
        matches!(
            ch as u32,
            0x3040..=0x309F | 0x30A0..=0x30FF | 0x4E00..=0x9FFF | 0xFF66..=0xFF9D
        )
    }

    pub(crate) fn intl_segmenter_is_sentence_terminal(ch: char) -> bool {
        matches!(ch, '.' | '!' | '?' | '。' | '！' | '？')
    }

    pub(crate) fn intl_segmenter_make_segment_value(
        segment: String,
        index: usize,
        input: &str,
        is_word_like: Option<bool>,
    ) -> Value {
        let mut entries = vec![
            ("segment".to_string(), Value::String(segment)),
            ("index".to_string(), Value::Number(index as i64)),
            ("input".to_string(), Value::String(input.to_string())),
        ];
        if let Some(is_word_like) = is_word_like {
            entries.push(("isWordLike".to_string(), Value::Bool(is_word_like)));
        }
        Self::new_object_value(entries)
    }

    pub(crate) fn intl_segment_graphemes(&self, input: &str) -> Vec<Value> {
        let mut out = Vec::new();
        for (index, ch) in input.chars().enumerate() {
            out.push(Self::intl_segmenter_make_segment_value(
                ch.to_string(),
                index,
                input,
                None,
            ));
        }
        out
    }
}
