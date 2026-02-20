use super::*;

impl Harness {
    pub(crate) fn intl_canonicalize_locale(raw: &str) -> Result<String> {
        let raw = raw.trim();
        let invalid_language_tag =
            || Error::ScriptRuntime(format!("RangeError: invalid language tag: \"{raw}\""));
        if raw.is_empty() {
            return Err(invalid_language_tag());
        }

        let subtags = raw.split('-').collect::<Vec<_>>();
        if subtags.iter().any(|subtag| subtag.is_empty()) {
            return Err(invalid_language_tag());
        }

        let language = subtags[0];
        let language_len = language.len();
        if !(language_len == 2 || language_len == 3 || (5..=8).contains(&language_len))
            || !language.chars().all(|ch| ch.is_ascii_alphabetic())
        {
            return Err(invalid_language_tag());
        }

        let mut canonical = Vec::with_capacity(subtags.len());
        canonical.push(language.to_ascii_lowercase());

        let mut saw_script = false;
        let mut saw_region = false;
        let mut in_extension = false;
        let mut in_private_use = false;

        for subtag in subtags.into_iter().skip(1) {
            if !subtag.chars().all(|ch| ch.is_ascii_alphanumeric()) {
                return Err(invalid_language_tag());
            }

            if subtag.len() == 1 {
                let singleton = subtag.to_ascii_lowercase();
                in_private_use = singleton == "x";
                in_extension = !in_private_use;
                canonical.push(singleton);
                continue;
            }

            if in_private_use {
                if subtag.len() > 8 {
                    return Err(invalid_language_tag());
                }
                canonical.push(subtag.to_ascii_lowercase());
                continue;
            }

            if in_extension {
                if !(2..=8).contains(&subtag.len()) {
                    return Err(invalid_language_tag());
                }
                canonical.push(subtag.to_ascii_lowercase());
                continue;
            }

            if !saw_script && subtag.len() == 4 && subtag.chars().all(|ch| ch.is_ascii_alphabetic())
            {
                let mut chars = subtag.chars();
                let first = chars.next().unwrap_or_default().to_ascii_uppercase();
                canonical.push(format!("{first}{}", chars.as_str().to_ascii_lowercase()));
                saw_script = true;
                continue;
            }

            if !saw_region
                && ((subtag.len() == 2 && subtag.chars().all(|ch| ch.is_ascii_alphabetic()))
                    || (subtag.len() == 3 && subtag.chars().all(|ch| ch.is_ascii_digit())))
            {
                if subtag.len() == 2 {
                    canonical.push(subtag.to_ascii_uppercase());
                } else {
                    canonical.push(subtag.to_string());
                }
                saw_region = true;
                continue;
            }

            if (5..=8).contains(&subtag.len())
                || (subtag.len() == 4
                    && subtag
                        .as_bytes()
                        .first()
                        .is_some_and(|byte| byte.is_ascii_digit()))
            {
                canonical.push(subtag.to_ascii_lowercase());
                continue;
            }

            return Err(invalid_language_tag());
        }

        Ok(canonical.join("-"))
    }

    pub(crate) fn intl_locale_family(locale: &str) -> &str {
        locale.split('-').next().unwrap_or_default()
    }

    pub(crate) fn intl_locale_region(locale: &str) -> Option<&str> {
        for subtag in locale.split('-').skip(1) {
            if subtag.len() == 2 && subtag.chars().all(|ch| ch.is_ascii_alphabetic()) {
                return Some(subtag);
            }
        }
        None
    }

    pub(crate) fn intl_formatter_supports_locale(kind: IntlFormatterKind, locale: &str) -> bool {
        match kind {
            IntlFormatterKind::Collator => {
                matches!(Self::intl_locale_family(locale), "en" | "de" | "sv")
            }
            IntlFormatterKind::DateTimeFormat => matches!(
                Self::intl_locale_family(locale),
                "en" | "de" | "id" | "ko" | "ar" | "ja"
            ),
            IntlFormatterKind::DisplayNames => {
                matches!(
                    Self::intl_locale_family(locale),
                    "en" | "zh" | "ja" | "he" | "es" | "fr"
                )
            }
            IntlFormatterKind::DurationFormat => {
                matches!(Self::intl_locale_family(locale), "en" | "fr" | "pt")
            }
            IntlFormatterKind::ListFormat => {
                matches!(Self::intl_locale_family(locale), "en" | "de")
            }
            IntlFormatterKind::NumberFormat => {
                matches!(Self::intl_locale_family(locale), "en" | "de")
            }
            IntlFormatterKind::PluralRules => {
                matches!(Self::intl_locale_family(locale), "en" | "ar")
            }
            IntlFormatterKind::RelativeTimeFormat => {
                matches!(Self::intl_locale_family(locale), "en" | "es")
            }
            IntlFormatterKind::Segmenter => {
                matches!(Self::intl_locale_family(locale), "en" | "fr" | "ja")
            }
        }
    }

    pub(crate) fn intl_select_locale_for_formatter(
        kind: IntlFormatterKind,
        requested_locales: &[String],
    ) -> String {
        for locale in requested_locales {
            if Self::intl_formatter_supports_locale(kind, locale) {
                return locale.clone();
            }
        }
        DEFAULT_LOCALE.to_string()
    }

    pub(crate) fn intl_supported_locales(
        kind: IntlFormatterKind,
        locales: Vec<String>,
    ) -> Vec<Value> {
        locales
            .into_iter()
            .filter(|locale| Self::intl_formatter_supports_locale(kind, locale))
            .map(Value::String)
            .collect::<Vec<_>>()
    }

    pub(crate) fn intl_collect_locales(&self, locales: &Value) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();

        let mut push_locale = |raw: &str| -> Result<()> {
            let canonical = Self::intl_canonicalize_locale(raw)?;
            if seen.insert(canonical.clone()) {
                out.push(canonical);
            }
            Ok(())
        };

        match locales {
            Value::Undefined | Value::Null => {}
            Value::String(locale) => push_locale(locale)?,
            Value::Array(values) => {
                for locale in values.borrow().iter() {
                    match locale {
                        Value::String(locale) => push_locale(locale)?,
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "TypeError: locale identifier must be a string".into(),
                            ));
                        }
                    }
                }
            }
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(Value::String(locale)) = Self::object_get_entry(&entries, "baseName") {
                    push_locale(&locale)?;
                } else {
                    return Err(Error::ScriptRuntime(
                        "TypeError: locale identifier must be a string".into(),
                    ));
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: locale identifier must be a string".into(),
                ));
            }
        }

        Ok(out)
    }

    pub(crate) fn intl_format_number_for_locale(value: f64, locale: &str) -> String {
        if !value.is_finite() {
            return Self::format_number_default(value);
        }

        let family = Self::intl_locale_family(locale);
        let (group_sep, decimal_sep) = if family == "de" {
            ('.', ',')
        } else {
            (',', '.')
        };

        let mut rendered = Self::format_number_default(value.abs());
        if rendered.contains('e') {
            if decimal_sep != '.' {
                rendered = rendered.replacen('.', &decimal_sep.to_string(), 1);
            }
            if value.is_sign_negative() {
                return format!("-{rendered}");
            }
            return rendered;
        }

        let mut parts = rendered.splitn(2, '.');
        let integer = parts.next().unwrap_or_default();
        let fraction = parts.next();

        let mut grouped = String::new();
        for (index, ch) in integer.chars().rev().enumerate() {
            if index > 0 && index % 3 == 0 {
                grouped.push(group_sep);
            }
            grouped.push(ch);
        }
        let mut grouped_integer = grouped.chars().rev().collect::<String>();
        if grouped_integer.is_empty() {
            grouped_integer.push('0');
        }

        if let Some(fraction) = fraction {
            grouped_integer.push(decimal_sep);
            grouped_integer.push_str(fraction);
        }

        if value.is_sign_negative() && grouped_integer != "0" {
            format!("-{grouped_integer}")
        } else {
            grouped_integer
        }
    }

    pub(crate) fn intl_locale_unicode_extension_value(locale: &str, key: &str) -> Option<String> {
        let subtags = locale.split('-').collect::<Vec<_>>();
        let mut i = 0usize;
        while i < subtags.len() {
            if subtags[i] != "u" {
                i += 1;
                continue;
            }
            i += 1;
            while i < subtags.len() {
                let current = subtags[i];
                if current.len() == 1 {
                    break;
                }
                if current.len() != 2 {
                    i += 1;
                    continue;
                }
                let current_key = current.to_ascii_lowercase();
                i += 1;
                let start = i;
                while i < subtags.len() {
                    let next = subtags[i];
                    if next.len() == 1 || next.len() == 2 {
                        break;
                    }
                    i += 1;
                }
                if current_key == key {
                    if start == i {
                        return Some("true".to_string());
                    }
                    return Some(subtags[start..i].join("-").to_ascii_lowercase());
                }
            }
        }
        None
    }

    pub(crate) fn intl_default_numbering_system_for_locale(locale: &str) -> String {
        if Self::intl_locale_family(locale) == "ar" {
            "arab".to_string()
        } else {
            "latn".to_string()
        }
    }

    pub(crate) fn intl_date_time_options_from_value(
        &self,
        locale: &str,
        options: Option<&Value>,
    ) -> Result<IntlDateTimeOptions> {
        let mut out = IntlDateTimeOptions {
            calendar: Self::intl_locale_unicode_extension_value(locale, "ca")
                .unwrap_or_else(|| "gregory".to_string()),
            numbering_system: Self::intl_locale_unicode_extension_value(locale, "nu")
                .unwrap_or_else(|| Self::intl_default_numbering_system_for_locale(locale)),
            time_zone: "UTC".to_string(),
            date_style: None,
            time_style: None,
            weekday: None,
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
            fractional_second_digits: None,
            time_zone_name: None,
            hour12: None,
            day_period: None,
        };

        let Some(options) = options else {
            out.year = Some("numeric".to_string());
            out.month = Some("numeric".to_string());
            out.day = Some("numeric".to_string());
            return Ok(out);
        };

        match options {
            Value::Undefined | Value::Null => {
                out.year = Some("numeric".to_string());
                out.month = Some("numeric".to_string());
                out.day = Some("numeric".to_string());
                return Ok(out);
            }
            Value::Object(entries) => {
                let entries = entries.borrow();

                let string_option = |key: &str| -> Option<String> {
                    match Self::object_get_entry(&entries, key) {
                        Some(Value::Undefined) | None => None,
                        Some(value) => Some(value.as_string()),
                    }
                };

                if let Some(value) = string_option("calendar") {
                    out.calendar = value;
                }
                if let Some(value) = string_option("numberingSystem") {
                    out.numbering_system = value;
                }
                if let Some(value) = string_option("timeZone") {
                    let normalized = Self::intl_normalize_time_zone(&value).ok_or_else(|| {
                        Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat timeZone option".into(),
                        )
                    })?;
                    out.time_zone = normalized;
                }
                if let Some(value) = string_option("dateStyle") {
                    if !matches!(value.as_str(), "full" | "long" | "medium" | "short") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat dateStyle option".into(),
                        ));
                    }
                    out.date_style = Some(value);
                }
                if let Some(value) = string_option("timeStyle") {
                    if !matches!(value.as_str(), "full" | "long" | "medium" | "short") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat timeStyle option".into(),
                        ));
                    }
                    out.time_style = Some(value);
                }
                if let Some(value) = string_option("weekday") {
                    if !matches!(value.as_str(), "narrow" | "short" | "long") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat weekday option".into(),
                        ));
                    }
                    out.weekday = Some(value);
                }
                if let Some(value) = string_option("year") {
                    if !matches!(value.as_str(), "2-digit" | "numeric") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat year option".into(),
                        ));
                    }
                    out.year = Some(value);
                }
                if let Some(value) = string_option("month") {
                    if !matches!(
                        value.as_str(),
                        "2-digit" | "numeric" | "narrow" | "short" | "long"
                    ) {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat month option".into(),
                        ));
                    }
                    out.month = Some(value);
                }
                if let Some(value) = string_option("day") {
                    if !matches!(value.as_str(), "2-digit" | "numeric") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat day option".into(),
                        ));
                    }
                    out.day = Some(value);
                }
                if let Some(value) = string_option("hour") {
                    if !matches!(value.as_str(), "2-digit" | "numeric") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat hour option".into(),
                        ));
                    }
                    out.hour = Some(value);
                }
                if let Some(value) = string_option("minute") {
                    if !matches!(value.as_str(), "2-digit" | "numeric") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat minute option".into(),
                        ));
                    }
                    out.minute = Some(value);
                }
                if let Some(value) = string_option("second") {
                    if !matches!(value.as_str(), "2-digit" | "numeric") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat second option".into(),
                        ));
                    }
                    out.second = Some(value);
                }
                if let Some(value) = string_option("timeZoneName") {
                    if !matches!(value.as_str(), "short" | "long") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat timeZoneName option".into(),
                        ));
                    }
                    out.time_zone_name = Some(value);
                }
                if let Some(value) = string_option("dayPeriod") {
                    if !matches!(value.as_str(), "narrow" | "short" | "long") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.DateTimeFormat dayPeriod option".into(),
                        ));
                    }
                    out.day_period = Some(value);
                }
                if let Some(value) = Self::object_get_entry(&entries, "hour12") {
                    if !matches!(value, Value::Undefined) {
                        out.hour12 = Some(value.truthy());
                    }
                }
                if let Some(value) = Self::object_get_entry(&entries, "fractionalSecondDigits") {
                    if !matches!(value, Value::Undefined) {
                        let digits = Self::value_to_i64(&value);
                        if !(1..=3).contains(&digits) {
                            return Err(Error::ScriptRuntime(
                                "RangeError: invalid Intl.DateTimeFormat fractionalSecondDigits option"
                                    .into(),
                            ));
                        }
                        out.fractional_second_digits = Some(digits as u8);
                    }
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Intl.DateTimeFormat options must be an object".into(),
                ));
            }
        }

        let has_component_options = out.weekday.is_some()
            || out.year.is_some()
            || out.month.is_some()
            || out.day.is_some()
            || out.hour.is_some()
            || out.minute.is_some()
            || out.second.is_some()
            || out.fractional_second_digits.is_some()
            || out.time_zone_name.is_some()
            || out.day_period.is_some();

        if (out.date_style.is_some() || out.time_style.is_some()) && has_component_options {
            return Err(Error::ScriptRuntime(
                "TypeError: dateStyle/timeStyle cannot be combined with date-time component options"
                    .into(),
            ));
        }

        if !has_component_options && out.date_style.is_none() && out.time_style.is_none() {
            out.year = Some("numeric".to_string());
            out.month = Some("numeric".to_string());
            out.day = Some("numeric".to_string());
        }

        Ok(out)
    }

    pub(crate) fn intl_date_time_options_to_value(options: &IntlDateTimeOptions) -> Value {
        let mut entries = vec![
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

    pub(crate) fn intl_date_time_options_from_internal(
        entries: &[(String, Value)],
    ) -> IntlDateTimeOptions {
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
            let bool_option = |key: &str| -> Option<bool> {
                match Self::object_get_entry(&options, key) {
                    Some(Value::Bool(value)) => Some(value),
                    _ => None,
                }
            };
            let number_option = |key: &str| -> Option<u8> {
                match Self::object_get_entry(&options, key) {
                    Some(Value::Number(value)) if (1..=3).contains(&value) => Some(value as u8),
                    _ => None,
                }
            };
            return IntlDateTimeOptions {
                calendar: string_option("calendar").unwrap_or_else(|| "gregory".to_string()),
                numbering_system: string_option("numberingSystem")
                    .unwrap_or_else(|| "latn".to_string()),
                time_zone: string_option("timeZone").unwrap_or_else(|| "UTC".to_string()),
                date_style: string_option("dateStyle"),
                time_style: string_option("timeStyle"),
                weekday: string_option("weekday"),
                year: string_option("year"),
                month: string_option("month"),
                day: string_option("day"),
                hour: string_option("hour"),
                minute: string_option("minute"),
                second: string_option("second"),
                fractional_second_digits: number_option("fractionalSecondDigits"),
                time_zone_name: string_option("timeZoneName"),
                hour12: bool_option("hour12"),
                day_period: string_option("dayPeriod"),
            };
        }

        IntlDateTimeOptions {
            calendar: "gregory".to_string(),
            numbering_system: "latn".to_string(),
            time_zone: "UTC".to_string(),
            date_style: None,
            time_style: None,
            weekday: None,
            year: Some("numeric".to_string()),
            month: Some("numeric".to_string()),
            day: Some("numeric".to_string()),
            hour: None,
            minute: None,
            second: None,
            fractional_second_digits: None,
            time_zone_name: None,
            hour12: None,
            day_period: None,
        }
    }

    pub(crate) fn intl_normalize_time_zone(input: &str) -> Option<String> {
        let normalized = input.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "utc" | "etc/utc" | "gmt" => Some("UTC".to_string()),
            "australia/sydney" => Some("Australia/Sydney".to_string()),
            "america/los_angeles" => Some("America/Los_Angeles".to_string()),
            _ => None,
        }
    }

    pub(crate) fn intl_time_zone_offset_minutes(time_zone: &str, _timestamp_ms: i64) -> i64 {
        match time_zone {
            "Australia/Sydney" => 11 * 60,
            "America/Los_Angeles" => -8 * 60,
            _ => 0,
        }
    }
}
