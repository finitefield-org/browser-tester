use super::*;

impl Harness {
    pub(crate) fn intl_format_date_time_range(
        &self,
        start_ms: i64,
        end_ms: i64,
        locale: &str,
        options: &IntlDateTimeOptions,
    ) -> String {
        let start = self.intl_format_date_time(start_ms, locale, options);
        let end = self.intl_format_date_time(end_ms, locale, options);
        if start == end {
            start
        } else {
            format!("{start} - {end}")
        }
    }

    pub(crate) fn intl_format_date_time_range_to_parts(
        &self,
        start_ms: i64,
        end_ms: i64,
        locale: &str,
        options: &IntlDateTimeOptions,
    ) -> (Vec<IntlPart>, Vec<String>) {
        let start = self.intl_format_date_time_to_parts(start_ms, locale, options);
        let end = self.intl_format_date_time_to_parts(end_ms, locale, options);
        if start
            .iter()
            .map(|part| part.value.as_str())
            .collect::<String>()
            == end
                .iter()
                .map(|part| part.value.as_str())
                .collect::<String>()
        {
            let sources = vec!["shared".to_string(); start.len()];
            return (start, sources);
        }

        let mut parts = Vec::new();
        let mut sources = Vec::new();
        for part in start {
            parts.push(part);
            sources.push("startRange".to_string());
        }
        parts.push(IntlPart {
            part_type: "literal".to_string(),
            value: " - ".to_string(),
        });
        sources.push("shared".to_string());
        for part in end {
            parts.push(part);
            sources.push("endRange".to_string());
        }
        (parts, sources)
    }

    pub(crate) fn intl_date_time_parts_to_value(
        &self,
        parts: &[IntlPart],
        sources: Option<&[String]>,
    ) -> Value {
        let mut out = Vec::with_capacity(parts.len());
        for (idx, part) in parts.iter().enumerate() {
            let mut entries = vec![
                ("type".to_string(), Value::String(part.part_type.clone())),
                ("value".to_string(), Value::String(part.value.clone())),
            ];
            if let Some(sources) = sources {
                if let Some(source) = sources.get(idx) {
                    entries.push(("source".to_string(), Value::String(source.clone())));
                }
            }
            out.push(Self::new_object_value(entries));
        }
        Self::new_array_value(out)
    }

    pub(crate) fn intl_date_time_resolved_options_value(
        &self,
        locale: String,
        options: &IntlDateTimeOptions,
    ) -> Value {
        let mut entries = vec![
            ("locale".to_string(), Value::String(locale)),
            (
                "calendar".to_string(),
                Value::String(options.calendar.clone()),
            ),
            (
                "numberingSystem".to_string(),
                Value::String(options.numbering_system.clone()),
            ),
            (
                "timeZone".to_string(),
                Value::String(options.time_zone.clone()),
            ),
        ];
        if let Some(value) = &options.date_style {
            entries.push(("dateStyle".to_string(), Value::String(value.clone())));
        }
        if let Some(value) = &options.time_style {
            entries.push(("timeStyle".to_string(), Value::String(value.clone())));
        }
        if let Some(value) = &options.weekday {
            entries.push(("weekday".to_string(), Value::String(value.clone())));
        }
        if let Some(value) = &options.year {
            entries.push(("year".to_string(), Value::String(value.clone())));
        }
        if let Some(value) = &options.month {
            entries.push(("month".to_string(), Value::String(value.clone())));
        }
        if let Some(value) = &options.day {
            entries.push(("day".to_string(), Value::String(value.clone())));
        }
        if let Some(value) = &options.hour {
            entries.push(("hour".to_string(), Value::String(value.clone())));
        }
        if let Some(value) = &options.minute {
            entries.push(("minute".to_string(), Value::String(value.clone())));
        }
        if let Some(value) = &options.second {
            entries.push(("second".to_string(), Value::String(value.clone())));
        }
        if let Some(value) = options.fractional_second_digits {
            entries.push((
                "fractionalSecondDigits".to_string(),
                Value::Number(value as i64),
            ));
        }
        if let Some(value) = &options.time_zone_name {
            entries.push(("timeZoneName".to_string(), Value::String(value.clone())));
        }
        if let Some(value) = options.hour12 {
            entries.push(("hour12".to_string(), Value::Bool(value)));
        }
        if let Some(value) = &options.day_period {
            entries.push(("dayPeriod".to_string(), Value::String(value.clone())));
        }
        Self::new_object_value(entries)
    }

    pub(crate) fn intl_duration_options_from_value(
        &self,
        options: Option<&Value>,
    ) -> Result<IntlDurationOptions> {
        let mut style = "short".to_string();
        let Some(options) = options else {
            return Ok(IntlDurationOptions { style });
        };

        match options {
            Value::Undefined | Value::Null => {}
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(value) = Self::object_get_entry(&entries, "style") {
                    if !matches!(value, Value::Undefined) {
                        let parsed = value.as_string();
                        if !matches!(parsed.as_str(), "long" | "short" | "narrow" | "digital") {
                            return Err(Error::ScriptRuntime(
                                "RangeError: invalid Intl.DurationFormat style option".into(),
                            ));
                        }
                        style = parsed;
                    }
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Intl.DurationFormat options must be an object".into(),
                ));
            }
        }

        Ok(IntlDurationOptions { style })
    }

    pub(crate) fn intl_duration_options_to_value(options: &IntlDurationOptions) -> Value {
        Self::new_object_value(vec![(
            "style".to_string(),
            Value::String(options.style.clone()),
        )])
    }

    pub(crate) fn intl_duration_options_from_internal(
        entries: &[(String, Value)],
    ) -> IntlDurationOptions {
        if let Some(Value::Object(options)) =
            Self::object_get_entry(entries, INTERNAL_INTL_OPTIONS_KEY)
        {
            let options = options.borrow();
            if let Some(Value::String(style)) = Self::object_get_entry(&options, "style") {
                return IntlDurationOptions { style };
            }
        }
        IntlDurationOptions {
            style: "short".to_string(),
        }
    }

    pub(crate) fn intl_duration_conjunction(locale: &str) -> &'static str {
        match Self::intl_locale_family(locale) {
            "fr" => " et ",
            "pt" => " e ",
            _ => " and ",
        }
    }

    pub(crate) fn intl_duration_unit_label(
        locale: &str,
        style: &str,
        unit: &str,
        value: i64,
    ) -> String {
        let style = if style == "digital" { "short" } else { style };
        let singular = matches!(value, -1 | 1);
        let family = Self::intl_locale_family(locale);

        match style {
            "narrow" => match unit {
                "year" => "y".to_string(),
                "month" => "mo".to_string(),
                "week" => "w".to_string(),
                "day" => "d".to_string(),
                "hour" => "h".to_string(),
                "minute" => "min".to_string(),
                "second" => "s".to_string(),
                "millisecond" => "ms".to_string(),
                "microsecond" => "us".to_string(),
                "nanosecond" => "ns".to_string(),
                _ => unit.to_string(),
            },
            "short" => {
                if family == "en" {
                    match unit {
                        "year" => "yr".to_string(),
                        "month" => "mo".to_string(),
                        "week" => "wk".to_string(),
                        "day" => "day".to_string(),
                        "hour" => "hr".to_string(),
                        "minute" => "min".to_string(),
                        "second" => "sec".to_string(),
                        "millisecond" => "ms".to_string(),
                        "microsecond" => "us".to_string(),
                        "nanosecond" => "ns".to_string(),
                        _ => unit.to_string(),
                    }
                } else {
                    match unit {
                        "year" => "a".to_string(),
                        "month" => "mo".to_string(),
                        "week" => "sem".to_string(),
                        "day" => "d".to_string(),
                        "hour" => "h".to_string(),
                        "minute" => "min".to_string(),
                        "second" => "s".to_string(),
                        "millisecond" => "ms".to_string(),
                        "microsecond" => "us".to_string(),
                        "nanosecond" => "ns".to_string(),
                        _ => unit.to_string(),
                    }
                }
            }
            _ => match family {
                "fr" => match unit {
                    "year" => {
                        if singular {
                            "an".to_string()
                        } else {
                            "ans".to_string()
                        }
                    }
                    "month" => "mois".to_string(),
                    "week" => {
                        if singular {
                            "semaine".to_string()
                        } else {
                            "semaines".to_string()
                        }
                    }
                    "day" => {
                        if singular {
                            "jour".to_string()
                        } else {
                            "jours".to_string()
                        }
                    }
                    "hour" => {
                        if singular {
                            "heure".to_string()
                        } else {
                            "heures".to_string()
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
                            "seconde".to_string()
                        } else {
                            "secondes".to_string()
                        }
                    }
                    "millisecond" => {
                        if singular {
                            "milliseconde".to_string()
                        } else {
                            "millisecondes".to_string()
                        }
                    }
                    "microsecond" => {
                        if singular {
                            "microseconde".to_string()
                        } else {
                            "microsecondes".to_string()
                        }
                    }
                    "nanosecond" => {
                        if singular {
                            "nanoseconde".to_string()
                        } else {
                            "nanosecondes".to_string()
                        }
                    }
                    _ => unit.to_string(),
                },
                "pt" => match unit {
                    "year" => {
                        if singular {
                            "ano".to_string()
                        } else {
                            "anos".to_string()
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
                            "dia".to_string()
                        } else {
                            "dias".to_string()
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
                    "millisecond" => {
                        if singular {
                            "milissegundo".to_string()
                        } else {
                            "milissegundos".to_string()
                        }
                    }
                    "microsecond" => {
                        if singular {
                            "microssegundo".to_string()
                        } else {
                            "microssegundos".to_string()
                        }
                    }
                    "nanosecond" => {
                        if singular {
                            "nanossegundo".to_string()
                        } else {
                            "nanossegundos".to_string()
                        }
                    }
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
                    "millisecond" => {
                        if singular {
                            "millisecond".to_string()
                        } else {
                            "milliseconds".to_string()
                        }
                    }
                    "microsecond" => {
                        if singular {
                            "microsecond".to_string()
                        } else {
                            "microseconds".to_string()
                        }
                    }
                    "nanosecond" => {
                        if singular {
                            "nanosecond".to_string()
                        } else {
                            "nanoseconds".to_string()
                        }
                    }
                    _ => unit.to_string(),
                },
            },
        }
    }

    pub(crate) fn intl_format_duration_to_parts(
        &self,
        locale: &str,
        options: &IntlDurationOptions,
        value: &Value,
    ) -> Result<Vec<IntlPart>> {
        let Value::Object(entries) = value else {
            return Err(Error::ScriptRuntime(
                "TypeError: Intl.DurationFormat input must be an object".into(),
            ));
        };
        let entries = entries.borrow();

        let units = [
            ("years", "year"),
            ("months", "month"),
            ("weeks", "week"),
            ("days", "day"),
            ("hours", "hour"),
            ("minutes", "minute"),
            ("seconds", "second"),
            ("milliseconds", "millisecond"),
            ("microseconds", "microsecond"),
            ("nanoseconds", "nanosecond"),
        ];

        let mut segments: Vec<(String, String)> = Vec::new();
        for (input_key, unit_name) in units {
            let Some(raw) = Self::object_get_entry(&entries, input_key) else {
                continue;
            };
            if matches!(raw, Value::Undefined) {
                continue;
            }
            let numeric = Self::value_to_i64(&raw);
            if numeric == 0 {
                continue;
            }
            let unit_label =
                Self::intl_duration_unit_label(locale, &options.style, unit_name, numeric);
            segments.push((unit_name.to_string(), format!("{numeric} {unit_label}")));
        }

        if segments.is_empty() {
            let unit_label = Self::intl_duration_unit_label(locale, &options.style, "second", 0);
            segments.push(("second".to_string(), format!("0 {unit_label}")));
        }

        let mut parts = Vec::new();
        for (index, (unit, text)) in segments.iter().enumerate() {
            if index > 0 {
                let literal = if matches!(options.style.as_str(), "narrow" | "digital") {
                    " ".to_string()
                } else if index + 1 == segments.len() {
                    Self::intl_duration_conjunction(locale).to_string()
                } else {
                    ", ".to_string()
                };
                parts.push(IntlPart {
                    part_type: "literal".to_string(),
                    value: literal,
                });
            }
            parts.push(IntlPart {
                part_type: unit.clone(),
                value: text.clone(),
            });
        }
        Ok(parts)
    }

    pub(crate) fn intl_format_duration(
        &self,
        locale: &str,
        options: &IntlDurationOptions,
        value: &Value,
    ) -> Result<String> {
        Ok(self
            .intl_format_duration_to_parts(locale, options, value)?
            .into_iter()
            .map(|part| part.value)
            .collect::<String>())
    }

    pub(crate) fn intl_duration_resolved_options_value(
        &self,
        locale: String,
        options: &IntlDurationOptions,
    ) -> Value {
        Self::new_object_value(vec![
            ("locale".to_string(), Value::String(locale)),
            ("style".to_string(), Value::String(options.style.clone())),
        ])
    }

    pub(crate) fn intl_list_options_from_value(
        &self,
        options: Option<&Value>,
    ) -> Result<IntlListOptions> {
        let mut style = "long".to_string();
        let mut list_type = "conjunction".to_string();
        let Some(options) = options else {
            return Ok(IntlListOptions { style, list_type });
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
                                "RangeError: invalid Intl.ListFormat style option".into(),
                            ));
                        }
                        style = parsed;
                    }
                }
                if let Some(value) = Self::object_get_entry(&entries, "type") {
                    if !matches!(value, Value::Undefined) {
                        let parsed = value.as_string();
                        if !matches!(parsed.as_str(), "conjunction" | "disjunction" | "unit") {
                            return Err(Error::ScriptRuntime(
                                "RangeError: invalid Intl.ListFormat type option".into(),
                            ));
                        }
                        list_type = parsed;
                    }
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Intl.ListFormat options must be an object".into(),
                ));
            }
        }

        Ok(IntlListOptions { style, list_type })
    }

    pub(crate) fn intl_list_options_to_value(options: &IntlListOptions) -> Value {
        Self::new_object_value(vec![
            ("style".to_string(), Value::String(options.style.clone())),
            ("type".to_string(), Value::String(options.list_type.clone())),
        ])
    }

    pub(crate) fn intl_list_options_from_internal(entries: &[(String, Value)]) -> IntlListOptions {
        if let Some(Value::Object(options)) =
            Self::object_get_entry(entries, INTERNAL_INTL_OPTIONS_KEY)
        {
            let options = options.borrow();
            let style = match Self::object_get_entry(&options, "style") {
                Some(Value::String(value)) => value,
                _ => "long".to_string(),
            };
            let list_type = match Self::object_get_entry(&options, "type") {
                Some(Value::String(value)) => value,
                _ => "conjunction".to_string(),
            };
            return IntlListOptions { style, list_type };
        }
        IntlListOptions {
            style: "long".to_string(),
            list_type: "conjunction".to_string(),
        }
    }
}
