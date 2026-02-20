use super::*;

impl Harness {
    pub(super) fn intl_canonicalize_locale(raw: &str) -> Result<String> {
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

    pub(super) fn intl_locale_family(locale: &str) -> &str {
        locale.split('-').next().unwrap_or_default()
    }

    pub(super) fn intl_locale_region(locale: &str) -> Option<&str> {
        for subtag in locale.split('-').skip(1) {
            if subtag.len() == 2 && subtag.chars().all(|ch| ch.is_ascii_alphabetic()) {
                return Some(subtag);
            }
        }
        None
    }

    pub(super) fn intl_formatter_supports_locale(kind: IntlFormatterKind, locale: &str) -> bool {
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

    pub(super) fn intl_select_locale_for_formatter(
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

    pub(super) fn intl_supported_locales(
        kind: IntlFormatterKind,
        locales: Vec<String>,
    ) -> Vec<Value> {
        locales
            .into_iter()
            .filter(|locale| Self::intl_formatter_supports_locale(kind, locale))
            .map(Value::String)
            .collect::<Vec<_>>()
    }

    pub(super) fn intl_collect_locales(&self, locales: &Value) -> Result<Vec<String>> {
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

    pub(super) fn intl_format_number_for_locale(value: f64, locale: &str) -> String {
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

    pub(super) fn intl_locale_unicode_extension_value(locale: &str, key: &str) -> Option<String> {
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

    pub(super) fn intl_default_numbering_system_for_locale(locale: &str) -> String {
        if Self::intl_locale_family(locale) == "ar" {
            "arab".to_string()
        } else {
            "latn".to_string()
        }
    }

    pub(super) fn intl_date_time_options_from_value(
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

    pub(super) fn intl_date_time_options_to_value(options: &IntlDateTimeOptions) -> Value {
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

    pub(super) fn intl_date_time_options_from_internal(
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

    pub(super) fn intl_normalize_time_zone(input: &str) -> Option<String> {
        let normalized = input.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "utc" | "etc/utc" | "gmt" => Some("UTC".to_string()),
            "australia/sydney" => Some("Australia/Sydney".to_string()),
            "america/los_angeles" => Some("America/Los_Angeles".to_string()),
            _ => None,
        }
    }

    pub(super) fn intl_time_zone_offset_minutes(time_zone: &str, _timestamp_ms: i64) -> i64 {
        match time_zone {
            "Australia/Sydney" => 11 * 60,
            "America/Los_Angeles" => -8 * 60,
            _ => 0,
        }
    }

    pub(super) fn intl_date_time_components(
        timestamp_ms: i64,
        time_zone: &str,
    ) -> IntlDateTimeComponents {
        let offset_minutes = Self::intl_time_zone_offset_minutes(time_zone, timestamp_ms);
        let adjusted = timestamp_ms.saturating_add(offset_minutes.saturating_mul(60_000));
        let (year, month, day, hour, minute, second, millisecond) =
            Self::date_components_utc(adjusted);
        let days = adjusted.div_euclid(86_400_000);
        let weekday = ((days + 4).rem_euclid(7)) as u32;
        IntlDateTimeComponents {
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            weekday,
            offset_minutes,
        }
    }

    pub(super) fn intl_default_hour12(locale: &str) -> bool {
        let family = Self::intl_locale_family(locale);
        if family != "en" {
            return false;
        }
        !matches!(Self::intl_locale_region(locale), Some("GB"))
    }

    pub(super) fn intl_month_name(locale: &str, month: u32, width: &str) -> String {
        let idx = month.saturating_sub(1) as usize;
        let family = Self::intl_locale_family(locale);
        let value = match (family, width) {
            ("de", "long") => [
                "Januar",
                "Februar",
                "Maerz",
                "April",
                "Mai",
                "Juni",
                "Juli",
                "August",
                "September",
                "Oktober",
                "November",
                "Dezember",
            ],
            ("de", "short") => [
                "Jan", "Feb", "Maer", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez",
            ],
            ("de", _) => ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"],
            (_, "long") => [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ],
            (_, "short") => [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ],
            _ => ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"],
        };
        value.get(idx).copied().unwrap_or_default().to_string()
    }

    pub(super) fn intl_weekday_name(locale: &str, weekday: u32, width: &str) -> String {
        let idx = weekday as usize;
        let family = Self::intl_locale_family(locale);
        let value = match (family, width) {
            ("de", "long") => [
                "Sonntag",
                "Montag",
                "Dienstag",
                "Mittwoch",
                "Donnerstag",
                "Freitag",
                "Samstag",
            ],
            ("de", "short") => ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"],
            ("de", _) => ["S", "M", "D", "M", "D", "F", "S"],
            (_, "long") => [
                "Sunday",
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
            ],
            (_, "short") => ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
            _ => ["S", "M", "T", "W", "T", "F", "S"],
        };
        value.get(idx).copied().unwrap_or_default().to_string()
    }

    pub(super) fn intl_apply_numbering_system(text: &str, numbering_system: &str) -> String {
        if numbering_system != "arab" {
            return text.to_string();
        }
        let mut out = String::with_capacity(text.len());
        for ch in text.chars() {
            let mapped = match ch {
                '0' => '٠',
                '1' => '١',
                '2' => '٢',
                '3' => '٣',
                '4' => '٤',
                '5' => '٥',
                '6' => '٦',
                '7' => '٧',
                '8' => '٨',
                '9' => '٩',
                _ => ch,
            };
            out.push(mapped);
        }
        out
    }

    pub(super) fn intl_format_date_component_year(
        locale: &str,
        options: &IntlDateTimeOptions,
        components: &IntlDateTimeComponents,
    ) -> String {
        let mut year = components.year;
        if options.calendar == "japanese" && Self::intl_locale_family(locale) == "ja" {
            year = (components.year - 1988).max(1);
        }
        match options.year.as_deref() {
            Some("2-digit") => format!("{:02}", year.rem_euclid(100)),
            _ => year.to_string(),
        }
    }

    pub(super) fn intl_format_date_component_month(
        locale: &str,
        options: &IntlDateTimeOptions,
        components: &IntlDateTimeComponents,
    ) -> String {
        match options.month.as_deref() {
            Some("2-digit") => format!("{:02}", components.month),
            Some("long") | Some("short") | Some("narrow") => Self::intl_month_name(
                locale,
                components.month,
                options.month.as_deref().unwrap_or("long"),
            ),
            _ => components.month.to_string(),
        }
    }

    pub(super) fn intl_format_date_component_day(
        options: &IntlDateTimeOptions,
        components: &IntlDateTimeComponents,
    ) -> String {
        match options.day.as_deref() {
            Some("2-digit") => format!("{:02}", components.day),
            _ => components.day.to_string(),
        }
    }

    pub(super) fn intl_append_date_parts(
        &self,
        parts: &mut Vec<IntlPart>,
        locale: &str,
        options: &IntlDateTimeOptions,
        components: &IntlDateTimeComponents,
    ) {
        if let Some(weekday_width) = options.weekday.as_deref() {
            parts.push(IntlPart {
                part_type: "weekday".to_string(),
                value: Self::intl_weekday_name(locale, components.weekday, weekday_width),
            });
            parts.push(IntlPart {
                part_type: "literal".to_string(),
                value: ", ".to_string(),
            });
        }

        let year = options
            .year
            .as_ref()
            .map(|_| Self::intl_format_date_component_year(locale, options, components));
        let month = options
            .month
            .as_ref()
            .map(|_| Self::intl_format_date_component_month(locale, options, components));
        let day = options
            .day
            .as_ref()
            .map(|_| Self::intl_format_date_component_day(options, components));

        let family = Self::intl_locale_family(locale);
        if family == "ko" {
            if let Some(year) = year {
                parts.push(IntlPart {
                    part_type: "year".to_string(),
                    value: year,
                });
                parts.push(IntlPart {
                    part_type: "literal".to_string(),
                    value: ". ".to_string(),
                });
            }
            if let Some(month) = month {
                parts.push(IntlPart {
                    part_type: "month".to_string(),
                    value: month,
                });
                parts.push(IntlPart {
                    part_type: "literal".to_string(),
                    value: ". ".to_string(),
                });
            }
            if let Some(day) = day {
                parts.push(IntlPart {
                    part_type: "day".to_string(),
                    value: day,
                });
                parts.push(IntlPart {
                    part_type: "literal".to_string(),
                    value: ".".to_string(),
                });
            }
            return;
        }

        let order = if family == "de" || family == "id" {
            ["day", "month", "year"]
        } else if family == "ja" {
            ["year", "month", "day"]
        } else if family == "en" && Self::intl_locale_region(locale) == Some("US") {
            ["month", "day", "year"]
        } else if family == "en" {
            ["day", "month", "year"]
        } else if family == "ar" {
            ["day", "month", "year"]
        } else {
            ["month", "day", "year"]
        };

        let separator = if family == "de" {
            "."
        } else if family == "en"
            && month
                .as_ref()
                .is_some_and(|m| m.chars().all(|ch| ch.is_ascii_alphabetic()))
        {
            " "
        } else {
            "/"
        };

        let mut first = true;
        for key in order {
            let value = match key {
                "year" => year.as_ref(),
                "month" => month.as_ref(),
                "day" => day.as_ref(),
                _ => None,
            };
            let Some(value) = value else {
                continue;
            };
            if !first {
                parts.push(IntlPart {
                    part_type: "literal".to_string(),
                    value: separator.to_string(),
                });
            }
            first = false;
            parts.push(IntlPart {
                part_type: key.to_string(),
                value: value.clone(),
            });
        }
    }

    pub(super) fn intl_day_period_label(hour: u32, width: &str) -> String {
        let base = if hour < 6 {
            "at night"
        } else if hour < 12 {
            "in the morning"
        } else if hour < 18 {
            "in the afternoon"
        } else {
            "at night"
        };
        match width {
            "narrow" => match hour {
                0..=11 => "am".to_string(),
                _ => "pm".to_string(),
            },
            "short" | "long" => base.to_string(),
            _ => base.to_string(),
        }
    }

    pub(super) fn intl_time_zone_name(
        locale: &str,
        time_zone: &str,
        offset_minutes: i64,
        style: &str,
    ) -> String {
        if time_zone == "UTC" {
            return "GMT".to_string();
        }
        if time_zone == "Australia/Sydney"
            && style == "short"
            && Self::intl_locale_region(locale) == Some("AU")
        {
            return "AEDT".to_string();
        }
        let sign = if offset_minutes >= 0 { '+' } else { '-' };
        let total = offset_minutes.abs();
        let hour = total / 60;
        let minute = total % 60;
        if minute == 0 {
            format!("GMT{sign}{hour}")
        } else {
            format!("GMT{sign}{hour}:{minute:02}")
        }
    }

    pub(super) fn intl_append_time_parts(
        &self,
        parts: &mut Vec<IntlPart>,
        locale: &str,
        options: &IntlDateTimeOptions,
        components: &IntlDateTimeComponents,
    ) {
        let has_hour = options.hour.is_some();
        let has_minute = options.minute.is_some();
        let has_second = options.second.is_some();
        if !has_hour && !has_minute && !has_second && options.time_zone_name.is_none() {
            return;
        }

        let hour12 = options
            .hour12
            .unwrap_or_else(|| Self::intl_default_hour12(locale));
        if has_hour {
            let raw_hour = components.hour;
            let hour_display = if hour12 {
                let mut h = raw_hour % 12;
                if h == 0 {
                    h = 12;
                }
                h
            } else {
                raw_hour
            };
            let hour_text = match options.hour.as_deref() {
                Some("2-digit") => format!("{hour_display:02}"),
                _ => hour_display.to_string(),
            };
            parts.push(IntlPart {
                part_type: "hour".to_string(),
                value: hour_text,
            });
        }

        if has_minute {
            if has_hour {
                parts.push(IntlPart {
                    part_type: "literal".to_string(),
                    value: ":".to_string(),
                });
            }
            let minute_text = match options.minute.as_deref() {
                Some("2-digit") => format!("{:02}", components.minute),
                _ => components.minute.to_string(),
            };
            parts.push(IntlPart {
                part_type: "minute".to_string(),
                value: minute_text,
            });
        }

        if has_second {
            if has_hour || has_minute {
                parts.push(IntlPart {
                    part_type: "literal".to_string(),
                    value: ":".to_string(),
                });
            }
            let second_text = match options.second.as_deref() {
                Some("2-digit") => format!("{:02}", components.second),
                _ => components.second.to_string(),
            };
            parts.push(IntlPart {
                part_type: "second".to_string(),
                value: second_text,
            });
        }

        if let Some(digits) = options.fractional_second_digits {
            let divisor = match digits {
                1 => 100,
                2 => 10,
                _ => 1,
            };
            parts.push(IntlPart {
                part_type: "literal".to_string(),
                value: ".".to_string(),
            });
            parts.push(IntlPart {
                part_type: "fractionalSecond".to_string(),
                value: format!(
                    "{:0width$}",
                    components.millisecond / divisor,
                    width = digits as usize
                ),
            });
        }

        if let Some(width) = options.day_period.as_deref() {
            parts.push(IntlPart {
                part_type: "literal".to_string(),
                value: " ".to_string(),
            });
            parts.push(IntlPart {
                part_type: "dayPeriod".to_string(),
                value: Self::intl_day_period_label(components.hour, width),
            });
        } else if has_hour && hour12 {
            parts.push(IntlPart {
                part_type: "literal".to_string(),
                value: " ".to_string(),
            });
            let lower = Self::intl_locale_region(locale) == Some("AU");
            let value = if components.hour < 12 { "AM" } else { "PM" };
            parts.push(IntlPart {
                part_type: "dayPeriod".to_string(),
                value: if lower {
                    value.to_ascii_lowercase()
                } else {
                    value.to_string()
                },
            });
        }

        if let Some(style) = options.time_zone_name.as_deref() {
            parts.push(IntlPart {
                part_type: "literal".to_string(),
                value: " ".to_string(),
            });
            parts.push(IntlPart {
                part_type: "timeZoneName".to_string(),
                value: Self::intl_time_zone_name(
                    locale,
                    &options.time_zone,
                    components.offset_minutes,
                    style,
                ),
            });
        }
    }

    pub(super) fn intl_expand_date_time_styles(
        locale: &str,
        options: &IntlDateTimeOptions,
    ) -> IntlDateTimeOptions {
        let mut expanded = options.clone();
        if let Some(date_style) = options.date_style.as_deref() {
            match date_style {
                "full" => {
                    expanded.weekday = Some("long".to_string());
                    expanded.year = Some("numeric".to_string());
                    expanded.month = Some("long".to_string());
                    expanded.day = Some("numeric".to_string());
                }
                "long" => {
                    expanded.year = Some("numeric".to_string());
                    expanded.month = Some("long".to_string());
                    expanded.day = Some("numeric".to_string());
                }
                "medium" => {
                    expanded.year = Some("numeric".to_string());
                    expanded.month = Some("short".to_string());
                    expanded.day = Some("numeric".to_string());
                }
                "short" => {
                    expanded.year = Some("2-digit".to_string());
                    expanded.month = Some("numeric".to_string());
                    expanded.day = Some("numeric".to_string());
                }
                _ => {}
            }
        }
        if let Some(time_style) = options.time_style.as_deref() {
            match time_style {
                "full" => {
                    expanded.hour = Some("numeric".to_string());
                    expanded.minute = Some("2-digit".to_string());
                    expanded.second = Some("2-digit".to_string());
                    expanded.time_zone_name = Some("long".to_string());
                }
                "long" => {
                    expanded.hour = Some("numeric".to_string());
                    expanded.minute = Some("2-digit".to_string());
                    expanded.second = Some("2-digit".to_string());
                    expanded.time_zone_name = Some("short".to_string());
                }
                "medium" => {
                    expanded.hour = Some("numeric".to_string());
                    expanded.minute = Some("2-digit".to_string());
                    expanded.second = Some("2-digit".to_string());
                }
                "short" => {
                    expanded.hour = Some("numeric".to_string());
                    expanded.minute = Some("2-digit".to_string());
                }
                _ => {}
            }
        }
        if options.date_style.is_some()
            && options.time_style.is_some()
            && Self::intl_locale_family(locale) == "en"
        {
            expanded.day_period = None;
        }
        expanded
    }

    pub(super) fn intl_format_date_time_to_parts(
        &self,
        timestamp_ms: i64,
        locale: &str,
        options: &IntlDateTimeOptions,
    ) -> Vec<IntlPart> {
        let options = Self::intl_expand_date_time_styles(locale, options);
        let components = Self::intl_date_time_components(timestamp_ms, &options.time_zone);
        let mut parts = Vec::new();

        let has_date = options.year.is_some() || options.month.is_some() || options.day.is_some();
        let has_time = options.hour.is_some()
            || options.minute.is_some()
            || options.second.is_some()
            || options.time_zone_name.is_some()
            || options.day_period.is_some()
            || options.fractional_second_digits.is_some();

        if has_date {
            self.intl_append_date_parts(&mut parts, locale, &options, &components);
        }
        if has_time {
            if has_date {
                let separator = if options.date_style.is_some() && options.time_style.is_some() {
                    if Self::intl_locale_family(locale) == "en" {
                        " at "
                    } else {
                        ", "
                    }
                } else {
                    ", "
                };
                parts.push(IntlPart {
                    part_type: "literal".to_string(),
                    value: separator.to_string(),
                });
            }
            self.intl_append_time_parts(&mut parts, locale, &options, &components);
        }

        for part in &mut parts {
            part.value = Self::intl_apply_numbering_system(&part.value, &options.numbering_system);
        }

        parts
    }

    pub(super) fn intl_format_date_time(
        &self,
        timestamp_ms: i64,
        locale: &str,
        options: &IntlDateTimeOptions,
    ) -> String {
        self.intl_format_date_time_to_parts(timestamp_ms, locale, options)
            .into_iter()
            .map(|part| part.value)
            .collect::<String>()
    }

    pub(super) fn intl_format_date_time_range(
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

    pub(super) fn intl_format_date_time_range_to_parts(
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

    pub(super) fn intl_date_time_parts_to_value(
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

    pub(super) fn intl_date_time_resolved_options_value(
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

    pub(super) fn intl_duration_options_from_value(
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

    pub(super) fn intl_duration_options_to_value(options: &IntlDurationOptions) -> Value {
        Self::new_object_value(vec![(
            "style".to_string(),
            Value::String(options.style.clone()),
        )])
    }

    pub(super) fn intl_duration_options_from_internal(
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

    pub(super) fn intl_duration_conjunction(locale: &str) -> &'static str {
        match Self::intl_locale_family(locale) {
            "fr" => " et ",
            "pt" => " e ",
            _ => " and ",
        }
    }

    pub(super) fn intl_duration_unit_label(
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

    pub(super) fn intl_format_duration_to_parts(
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

    pub(super) fn intl_format_duration(
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

    pub(super) fn intl_duration_resolved_options_value(
        &self,
        locale: String,
        options: &IntlDurationOptions,
    ) -> Value {
        Self::new_object_value(vec![
            ("locale".to_string(), Value::String(locale)),
            ("style".to_string(), Value::String(options.style.clone())),
        ])
    }

    pub(super) fn intl_list_options_from_value(
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

    pub(super) fn intl_list_options_to_value(options: &IntlListOptions) -> Value {
        Self::new_object_value(vec![
            ("style".to_string(), Value::String(options.style.clone())),
            ("type".to_string(), Value::String(options.list_type.clone())),
        ])
    }

    pub(super) fn intl_list_options_from_internal(entries: &[(String, Value)]) -> IntlListOptions {
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

    pub(super) fn intl_list_separator_before_last(
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

    pub(super) fn intl_format_list_to_parts(
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

    pub(super) fn intl_format_list(
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

    pub(super) fn intl_list_resolved_options_value(
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

    pub(super) fn intl_plural_rules_options_from_value(
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

    pub(super) fn intl_plural_rules_options_to_value(options: &IntlPluralRulesOptions) -> Value {
        Self::new_object_value(vec![(
            "type".to_string(),
            Value::String(options.rule_type.clone()),
        )])
    }

    pub(super) fn intl_plural_rules_options_from_internal(
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

    pub(super) fn intl_plural_rules_categories(locale: &str, rule_type: &str) -> Vec<String> {
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

    pub(super) fn intl_plural_rules_select_tag(
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

    pub(super) fn intl_plural_rules_select(
        &self,
        locale: &str,
        options: &IntlPluralRulesOptions,
        value: &Value,
    ) -> String {
        let number = Self::coerce_number_for_global(value);
        Self::intl_plural_rules_select_tag(locale, &options.rule_type, number)
    }

    pub(super) fn intl_plural_rules_select_range(
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

    pub(super) fn intl_plural_rules_resolved_options_value(
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

    pub(super) fn intl_relative_time_options_from_value(
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

    pub(super) fn intl_relative_time_options_to_value(options: &IntlRelativeTimeOptions) -> Value {
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

    pub(super) fn intl_relative_time_options_from_internal(
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

    pub(super) fn intl_relative_time_normalize_unit(unit: &str) -> Option<String> {
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

    pub(super) fn intl_relative_time_auto_literal(
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

    pub(super) fn intl_relative_time_unit_label(
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

    pub(super) fn intl_relative_time_parts(
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

    pub(super) fn intl_format_relative_time(
        &self,
        locale: &str,
        options: &IntlRelativeTimeOptions,
        value: &Value,
        unit: &Value,
    ) -> Result<String> {
        let parts = self.intl_format_relative_time_to_parts(locale, options, value, unit)?;
        Ok(parts.into_iter().map(|part| part.value).collect::<String>())
    }

    pub(super) fn intl_format_relative_time_to_parts(
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

    pub(super) fn intl_relative_time_parts_to_value(
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

    pub(super) fn intl_relative_time_resolved_options_value(
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

    pub(super) fn intl_segmenter_options_from_value(
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

    pub(super) fn intl_segmenter_options_to_value(options: &IntlSegmenterOptions) -> Value {
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

    pub(super) fn intl_segmenter_options_from_internal(
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

    pub(super) fn intl_segmenter_is_japanese_char(ch: char) -> bool {
        matches!(
            ch as u32,
            0x3040..=0x309F | 0x30A0..=0x30FF | 0x4E00..=0x9FFF | 0xFF66..=0xFF9D
        )
    }

    pub(super) fn intl_segmenter_is_sentence_terminal(ch: char) -> bool {
        matches!(ch, '.' | '!' | '?' | '。' | '！' | '？')
    }

    pub(super) fn intl_segmenter_make_segment_value(
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

    pub(super) fn intl_segment_graphemes(&self, input: &str) -> Vec<Value> {
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

    pub(super) fn intl_segment_words(&self, locale: &str, input: &str) -> Vec<Value> {
        if input.is_empty() {
            return Vec::new();
        }

        let family = Self::intl_locale_family(locale);
        let chars = input.chars().collect::<Vec<_>>();
        let mut out = Vec::new();
        let mut idx = 0usize;

        while idx < chars.len() {
            let ch = chars[idx];
            if ch.is_whitespace() {
                let start = idx;
                idx += 1;
                while idx < chars.len() && chars[idx].is_whitespace() {
                    idx += 1;
                }
                let segment = chars[start..idx].iter().collect::<String>();
                out.push(Self::intl_segmenter_make_segment_value(
                    segment,
                    start,
                    input,
                    Some(false),
                ));
                continue;
            }

            if family == "ja" && Self::intl_segmenter_is_japanese_char(ch) {
                let start = idx;
                if matches!(ch, 'は' | 'が' | 'を' | 'に' | 'で' | 'と') {
                    idx += 1;
                } else {
                    idx += 1;
                    while idx < chars.len() {
                        let next = chars[idx];
                        if !Self::intl_segmenter_is_japanese_char(next)
                            || matches!(next, 'は' | 'が' | 'を' | 'に' | 'で' | 'と')
                        {
                            break;
                        }
                        idx += 1;
                    }
                }
                let segment = chars[start..idx].iter().collect::<String>();
                out.push(Self::intl_segmenter_make_segment_value(
                    segment,
                    start,
                    input,
                    Some(true),
                ));
                continue;
            }

            if ch.is_alphanumeric() || ch == '\'' {
                let start = idx;
                idx += 1;
                while idx < chars.len() {
                    let next = chars[idx];
                    if !(next.is_alphanumeric() || next == '\'') {
                        break;
                    }
                    idx += 1;
                }
                let segment = chars[start..idx].iter().collect::<String>();
                out.push(Self::intl_segmenter_make_segment_value(
                    segment,
                    start,
                    input,
                    Some(true),
                ));
                continue;
            }

            let start = idx;
            idx += 1;
            out.push(Self::intl_segmenter_make_segment_value(
                ch.to_string(),
                start,
                input,
                Some(false),
            ));
        }

        out
    }

    pub(super) fn intl_segment_sentences(&self, input: &str) -> Vec<Value> {
        if input.is_empty() {
            return Vec::new();
        }

        let chars = input.chars().collect::<Vec<_>>();
        let mut out = Vec::new();
        let mut start = 0usize;
        let mut idx = 0usize;
        while idx < chars.len() {
            let ch = chars[idx];
            idx += 1;
            if Self::intl_segmenter_is_sentence_terminal(ch) {
                let segment = chars[start..idx].iter().collect::<String>();
                out.push(Self::intl_segmenter_make_segment_value(
                    segment, start, input, None,
                ));
                start = idx;
            }
        }
        if start < chars.len() {
            let segment = chars[start..].iter().collect::<String>();
            out.push(Self::intl_segmenter_make_segment_value(
                segment, start, input, None,
            ));
        }
        out
    }

    pub(super) fn intl_segment_input(
        &self,
        locale: &str,
        options: &IntlSegmenterOptions,
        input: &str,
    ) -> Vec<Value> {
        match options.granularity.as_str() {
            "word" => self.intl_segment_words(locale, input),
            "sentence" => self.intl_segment_sentences(input),
            _ => self.intl_segment_graphemes(input),
        }
    }

    pub(super) fn intl_segmenter_resolved_options_value(
        &self,
        locale: String,
        options: &IntlSegmenterOptions,
    ) -> Value {
        Self::new_object_value(vec![
            ("locale".to_string(), Value::String(locale)),
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

    pub(super) fn intl_locale_normalize_language(value: &str) -> Option<String> {
        let value = value.trim();
        let len = value.len();
        if !(len == 2 || len == 3 || (5..=8).contains(&len)) {
            return None;
        }
        if !value.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return None;
        }
        Some(value.to_ascii_lowercase())
    }

    pub(super) fn intl_locale_normalize_script(value: &str) -> Option<String> {
        let value = value.trim();
        if value.len() != 4 || !value.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return None;
        }
        let mut chars = value.chars();
        let first = chars.next().unwrap_or_default().to_ascii_uppercase();
        Some(format!("{first}{}", chars.as_str().to_ascii_lowercase()))
    }

    pub(super) fn intl_locale_normalize_region(value: &str) -> Option<String> {
        let value = value.trim();
        if value.len() == 2 && value.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return Some(value.to_ascii_uppercase());
        }
        if value.len() == 3 && value.chars().all(|ch| ch.is_ascii_digit()) {
            return Some(value.to_string());
        }
        None
    }

    pub(super) fn intl_locale_normalize_unicode_type(value: &str) -> Option<String> {
        let value = value.trim();
        if value.is_empty() {
            return None;
        }
        let parts = value.split('-').collect::<Vec<_>>();
        if parts.is_empty() {
            return None;
        }
        if parts.iter().any(|part| {
            part.len() < 3 || part.len() > 8 || !part.chars().all(|ch| ch.is_ascii_alphanumeric())
        }) {
            return None;
        }
        Some(parts.join("-").to_ascii_lowercase())
    }

    pub(super) fn intl_locale_options_from_value(
        &self,
        options: Option<&Value>,
    ) -> Result<IntlLocaleOptions> {
        let mut out = IntlLocaleOptions {
            language: None,
            script: None,
            region: None,
            calendar: None,
            case_first: None,
            collation: None,
            hour_cycle: None,
            numbering_system: None,
            numeric: None,
        };

        let Some(options) = options else {
            return Ok(out);
        };

        match options {
            Value::Undefined | Value::Null => return Ok(out),
            Value::Object(entries) => {
                let entries = entries.borrow();
                let string_option = |key: &str| -> Option<String> {
                    match Self::object_get_entry(&entries, key) {
                        Some(Value::Undefined) | None => None,
                        Some(value) => Some(value.as_string()),
                    }
                };

                if let Some(value) = string_option("language") {
                    out.language =
                        Some(Self::intl_locale_normalize_language(&value).ok_or_else(|| {
                            Error::ScriptRuntime(
                                "RangeError: invalid Intl.Locale language option".into(),
                            )
                        })?);
                }
                if let Some(value) = string_option("script") {
                    out.script =
                        Some(Self::intl_locale_normalize_script(&value).ok_or_else(|| {
                            Error::ScriptRuntime(
                                "RangeError: invalid Intl.Locale script option".into(),
                            )
                        })?);
                }
                if let Some(value) = string_option("region") {
                    out.region =
                        Some(Self::intl_locale_normalize_region(&value).ok_or_else(|| {
                            Error::ScriptRuntime(
                                "RangeError: invalid Intl.Locale region option".into(),
                            )
                        })?);
                }
                if let Some(value) = string_option("calendar") {
                    out.calendar = Some(
                        Self::intl_locale_normalize_unicode_type(&value).ok_or_else(|| {
                            Error::ScriptRuntime(
                                "RangeError: invalid Intl.Locale calendar option".into(),
                            )
                        })?,
                    );
                }
                if let Some(value) = string_option("caseFirst") {
                    let value = value.to_ascii_lowercase();
                    if !matches!(value.as_str(), "upper" | "lower" | "false") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.Locale caseFirst option".into(),
                        ));
                    }
                    out.case_first = Some(value);
                }
                if let Some(value) = string_option("collation") {
                    out.collation = Some(
                        Self::intl_locale_normalize_unicode_type(&value).ok_or_else(|| {
                            Error::ScriptRuntime(
                                "RangeError: invalid Intl.Locale collation option".into(),
                            )
                        })?,
                    );
                }
                if let Some(value) = string_option("hourCycle") {
                    let value = value.to_ascii_lowercase();
                    if !matches!(value.as_str(), "h11" | "h12" | "h23" | "h24") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.Locale hourCycle option".into(),
                        ));
                    }
                    out.hour_cycle = Some(value);
                }
                if let Some(value) = string_option("numberingSystem") {
                    out.numbering_system = Some(
                        Self::intl_locale_normalize_unicode_type(&value).ok_or_else(|| {
                            Error::ScriptRuntime(
                                "RangeError: invalid Intl.Locale numberingSystem option".into(),
                            )
                        })?,
                    );
                }
                if let Some(value) = Self::object_get_entry(&entries, "numeric") {
                    if !matches!(value, Value::Undefined) {
                        out.numeric = Some(value.truthy());
                    }
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Intl.Locale options must be an object".into(),
                ));
            }
        }

        Ok(out)
    }

    pub(super) fn intl_locale_data_from_canonical_tag(canonical: &str) -> IntlLocaleData {
        let subtags = canonical.split('-').collect::<Vec<_>>();
        let language = subtags.first().copied().unwrap_or_default().to_string();
        let mut script = None;
        let mut region = None;
        let mut variants = Vec::new();
        let mut calendar = None;
        let mut case_first = None;
        let mut collation = None;
        let mut hour_cycle = None;
        let mut numbering_system = None;
        let mut numeric = None;

        let mut idx = 1usize;
        while idx < subtags.len() {
            let subtag = subtags[idx];
            if subtag.len() == 1 {
                break;
            }
            if script.is_none()
                && subtag.len() == 4
                && subtag.chars().all(|ch| ch.is_ascii_alphabetic())
            {
                script = Some(subtag.to_string());
                idx += 1;
                continue;
            }
            if region.is_none()
                && ((subtag.len() == 2 && subtag.chars().all(|ch| ch.is_ascii_alphabetic()))
                    || (subtag.len() == 3 && subtag.chars().all(|ch| ch.is_ascii_digit())))
            {
                region = Some(subtag.to_string());
                idx += 1;
                continue;
            }
            variants.push(subtag.to_string());
            idx += 1;
        }

        while idx < subtags.len() {
            let singleton = subtags[idx].to_ascii_lowercase();
            idx += 1;
            let start = idx;
            while idx < subtags.len() && subtags[idx].len() != 1 {
                idx += 1;
            }
            if singleton != "u" {
                continue;
            }

            let mut key_index = start;
            while key_index < idx {
                let key = subtags[key_index].to_ascii_lowercase();
                key_index += 1;
                if key.len() != 2 {
                    continue;
                }

                let value_start = key_index;
                while key_index < idx && subtags[key_index].len() > 2 {
                    key_index += 1;
                }
                let value = if value_start == key_index {
                    "true".to_string()
                } else {
                    subtags[value_start..key_index]
                        .join("-")
                        .to_ascii_lowercase()
                };
                match key.as_str() {
                    "ca" => calendar = Some(value),
                    "kf" => case_first = Some(value),
                    "co" => collation = Some(value),
                    "hc" => hour_cycle = Some(value),
                    "nu" => numbering_system = Some(value),
                    "kn" => numeric = Some(value != "false"),
                    _ => {}
                }
            }
        }

        IntlLocaleData {
            language,
            script,
            region,
            variants,
            calendar,
            case_first,
            collation,
            hour_cycle,
            numbering_system,
            numeric,
        }
    }

    pub(super) fn intl_locale_data_base_name(data: &IntlLocaleData) -> String {
        let mut out = vec![data.language.clone()];
        if let Some(script) = &data.script {
            out.push(script.clone());
        }
        if let Some(region) = &data.region {
            out.push(region.clone());
        }
        out.extend(data.variants.iter().cloned());
        out.join("-")
    }

    pub(super) fn intl_locale_data_to_string(data: &IntlLocaleData) -> String {
        let mut out = vec![Self::intl_locale_data_base_name(data)];
        let mut extension = Vec::new();

        if let Some(value) = &data.calendar {
            extension.push("ca".to_string());
            extension.push(value.clone());
        }
        if let Some(value) = &data.collation {
            extension.push("co".to_string());
            extension.push(value.clone());
        }
        if let Some(value) = &data.hour_cycle {
            extension.push("hc".to_string());
            extension.push(value.clone());
        }
        if let Some(value) = &data.case_first {
            extension.push("kf".to_string());
            extension.push(value.clone());
        }
        if let Some(value) = data.numeric {
            extension.push("kn".to_string());
            if !value {
                extension.push("false".to_string());
            }
        }
        if let Some(value) = &data.numbering_system {
            extension.push("nu".to_string());
            extension.push(value.clone());
        }

        if !extension.is_empty() {
            out.push("u".to_string());
            out.extend(extension);
        }
        out.join("-")
    }

    pub(super) fn intl_locale_data_to_internal_value(data: &IntlLocaleData) -> Value {
        Self::new_object_value(vec![
            ("language".to_string(), Value::String(data.language.clone())),
            (
                "script".to_string(),
                data.script
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "region".to_string(),
                data.region
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "variants".to_string(),
                Self::new_array_value(
                    data.variants
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect::<Vec<_>>(),
                ),
            ),
            (
                "calendar".to_string(),
                data.calendar
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "caseFirst".to_string(),
                data.case_first
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "collation".to_string(),
                data.collation
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "hourCycle".to_string(),
                data.hour_cycle
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "numberingSystem".to_string(),
                data.numbering_system
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "numeric".to_string(),
                data.numeric.map_or(Value::Undefined, Value::Bool),
            ),
        ])
    }

    pub(super) fn intl_locale_data_from_internal_value(value: &Value) -> Option<IntlLocaleData> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        let language = match Self::object_get_entry(&entries, "language") {
            Some(Value::String(value)) => value,
            _ => return None,
        };
        let script = match Self::object_get_entry(&entries, "script") {
            Some(Value::String(value)) => Some(value),
            _ => None,
        };
        let region = match Self::object_get_entry(&entries, "region") {
            Some(Value::String(value)) => Some(value),
            _ => None,
        };
        let variants = match Self::object_get_entry(&entries, "variants") {
            Some(Value::Array(values)) => values
                .borrow()
                .iter()
                .filter_map(|value| match value {
                    Value::String(value) => Some(value.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        let calendar = match Self::object_get_entry(&entries, "calendar") {
            Some(Value::String(value)) => Some(value),
            _ => None,
        };
        let case_first = match Self::object_get_entry(&entries, "caseFirst") {
            Some(Value::String(value)) => Some(value),
            _ => None,
        };
        let collation = match Self::object_get_entry(&entries, "collation") {
            Some(Value::String(value)) => Some(value),
            _ => None,
        };
        let hour_cycle = match Self::object_get_entry(&entries, "hourCycle") {
            Some(Value::String(value)) => Some(value),
            _ => None,
        };
        let numbering_system = match Self::object_get_entry(&entries, "numberingSystem") {
            Some(Value::String(value)) => Some(value),
            _ => None,
        };
        let numeric = match Self::object_get_entry(&entries, "numeric") {
            Some(Value::Bool(value)) => Some(value),
            _ => None,
        };
        Some(IntlLocaleData {
            language,
            script,
            region,
            variants,
            calendar,
            case_first,
            collation,
            hour_cycle,
            numbering_system,
            numeric,
        })
    }

    pub(super) fn intl_locale_data_from_input_value(
        &self,
        tag: &Value,
        options: Option<&Value>,
    ) -> Result<IntlLocaleData> {
        let raw_tag = match tag {
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(value) = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_DATA_KEY)
                {
                    if let Some(data) = Self::intl_locale_data_from_internal_value(&value) {
                        Self::intl_locale_data_to_string(&data)
                    } else {
                        tag.as_string()
                    }
                } else if let Some(Value::String(base_name)) =
                    Self::object_get_entry(&entries, "baseName")
                {
                    base_name
                } else {
                    tag.as_string()
                }
            }
            _ => tag.as_string(),
        };

        let canonical = Self::intl_canonicalize_locale(&raw_tag)?;
        let mut data = Self::intl_locale_data_from_canonical_tag(&canonical);
        let options = self.intl_locale_options_from_value(options)?;

        if let Some(value) = options.language {
            data.language = value;
        }
        if let Some(value) = options.script {
            data.script = Some(value);
        }
        if let Some(value) = options.region {
            data.region = Some(value);
        }
        if let Some(value) = options.calendar {
            data.calendar = Some(value);
        }
        if let Some(value) = options.case_first {
            data.case_first = Some(value);
        }
        if let Some(value) = options.collation {
            data.collation = Some(value);
        }
        if let Some(value) = options.hour_cycle {
            data.hour_cycle = Some(value);
        }
        if let Some(value) = options.numbering_system {
            data.numbering_system = Some(value);
        }
        if let Some(value) = options.numeric {
            data.numeric = Some(value);
        }

        Ok(data)
    }

    pub(super) fn intl_locale_prepend_unique(values: &mut Vec<String>, preferred: Option<&str>) {
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

    pub(super) fn intl_locale_get_calendars(&self, data: &IntlLocaleData) -> Vec<String> {
        let mut out = match data.language.as_str() {
            "ja" => vec!["gregory".to_string(), "japanese".to_string()],
            "ar" => vec!["gregory".to_string(), "islamic-umalqura".to_string()],
            _ => vec!["gregory".to_string()],
        };
        Self::intl_locale_prepend_unique(&mut out, data.calendar.as_deref());
        out
    }

    pub(super) fn intl_locale_get_collations(&self, data: &IntlLocaleData) -> Vec<String> {
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

    pub(super) fn intl_locale_get_hour_cycles(&self, data: &IntlLocaleData) -> Vec<String> {
        let mut out = if data.language == "en" || data.language == "ar" {
            vec!["h12".to_string(), "h23".to_string()]
        } else {
            vec!["h23".to_string(), "h12".to_string()]
        };
        Self::intl_locale_prepend_unique(&mut out, data.hour_cycle.as_deref());
        out
    }

    pub(super) fn intl_locale_get_numbering_systems(&self, data: &IntlLocaleData) -> Vec<String> {
        let mut out = if data.language == "ar" {
            vec!["arab".to_string(), "latn".to_string()]
        } else {
            vec!["latn".to_string()]
        };
        Self::intl_locale_prepend_unique(&mut out, data.numbering_system.as_deref());
        out
    }

    pub(super) fn intl_locale_get_text_info(&self, data: &IntlLocaleData) -> Value {
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

    pub(super) fn intl_locale_get_time_zones(&self, data: &IntlLocaleData) -> Vec<String> {
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

    pub(super) fn intl_locale_get_week_info(&self, data: &IntlLocaleData) -> Value {
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

    pub(super) fn intl_locale_likely_subtags(
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

    pub(super) fn intl_locale_maximize_data(&self, data: &IntlLocaleData) -> IntlLocaleData {
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

    pub(super) fn intl_locale_minimize_data(&self, data: &IntlLocaleData) -> IntlLocaleData {
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

    pub(super) fn intl_display_names_options_from_value(
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

    pub(super) fn intl_display_names_options_to_value(options: &IntlDisplayNamesOptions) -> Value {
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

    pub(super) fn intl_display_names_options_from_internal(
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

    pub(super) fn intl_canonicalize_display_names_code(
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

    pub(super) fn intl_display_names_lookup(
        locale: &str,
        options: &IntlDisplayNamesOptions,
        code: &str,
    ) -> Option<String> {
        let family = Self::intl_locale_family(locale);
        match options.display_type.as_str() {
            "region" => match family {
                "zh" => match code {
                    "419" => Some("拉丁美洲".to_string()),
                    "BZ" => Some("貝里斯".to_string()),
                    "US" => Some("美國".to_string()),
                    "BA" => Some("波士尼亞與赫塞哥維納".to_string()),
                    "MM" => Some("緬甸".to_string()),
                    _ => None,
                },
                "ja" => match code {
                    "419" => Some("ラテンアメリカ".to_string()),
                    "BZ" => Some("ベリーズ".to_string()),
                    "US" => Some("アメリカ合衆国".to_string()),
                    "BA" => Some("ボスニア・ヘルツェゴビナ".to_string()),
                    "MM" => Some("ミャンマー".to_string()),
                    _ => None,
                },
                "he" => match code {
                    "419" => Some("אמריקה הלטינית".to_string()),
                    "BZ" => Some("בליז".to_string()),
                    "US" => Some("ארצות הברית".to_string()),
                    "BA" => Some("בוסניה והרצגובינה".to_string()),
                    "MM" => Some("מיאנמר (בורמה)".to_string()),
                    _ => None,
                },
                "es" => match code {
                    "419" => Some("Latinoamérica".to_string()),
                    "BZ" => Some("Belice".to_string()),
                    "US" => Some("Estados Unidos".to_string()),
                    "BA" => Some("Bosnia y Herzegovina".to_string()),
                    "MM" => Some("Myanmar (Birmania)".to_string()),
                    _ => None,
                },
                "fr" => match code {
                    "419" => Some("Amérique latine".to_string()),
                    "BZ" => Some("Belize".to_string()),
                    "US" => Some("États-Unis".to_string()),
                    "BA" => Some("Bosnie-Herzégovine".to_string()),
                    "MM" => Some("Myanmar (Birmanie)".to_string()),
                    _ => None,
                },
                _ => match code {
                    "419" => Some("Latin America".to_string()),
                    "BZ" => Some("Belize".to_string()),
                    "US" => Some("United States".to_string()),
                    "BA" => Some("Bosnia & Herzegovina".to_string()),
                    "MM" => Some("Myanmar (Burma)".to_string()),
                    _ => None,
                },
            },
            "language" => match family {
                "zh" => match code {
                    "fr" => Some("法文".to_string()),
                    "de" => Some("德文".to_string()),
                    "zh" => Some("中文".to_string()),
                    "fr-CA" => Some("加拿大法文".to_string()),
                    "zh-Hant" => Some("繁體中文".to_string()),
                    "en-US" => Some("美式英文".to_string()),
                    "zh-TW" => Some("中文（台灣）".to_string()),
                    _ => None,
                },
                "ja" => match code {
                    "fr" => Some("フランス語".to_string()),
                    "de" => Some("ドイツ語".to_string()),
                    "zh" => Some("中国語".to_string()),
                    "fr-CA" => Some("カナダのフランス語".to_string()),
                    "zh-Hant" => Some("繁体字中国語".to_string()),
                    "en-US" => Some("アメリカ英語".to_string()),
                    "zh-TW" => Some("中国語（台湾）".to_string()),
                    _ => None,
                },
                "he" => match code {
                    "fr" => Some("צרפתית".to_string()),
                    "de" => Some("גרמנית".to_string()),
                    "zh" => Some("סינית".to_string()),
                    "fr-CA" => Some("צרפתית קנדית".to_string()),
                    "zh-Hant" => Some("סינית מסורתית".to_string()),
                    "en-US" => Some("אנגלית אמריקאית".to_string()),
                    "zh-TW" => Some("סינית (טייוואן)".to_string()),
                    _ => None,
                },
                "es" => match code {
                    "fr" => Some("francés".to_string()),
                    "de" => Some("alemán".to_string()),
                    "zh" => Some("chino".to_string()),
                    "fr-CA" => Some("francés canadiense".to_string()),
                    "zh-Hant" => Some("chino tradicional".to_string()),
                    "en-US" => Some("inglés estadounidense".to_string()),
                    "zh-TW" => Some("chino (Taiwán)".to_string()),
                    _ => None,
                },
                "fr" => match code {
                    "fr" => Some("français".to_string()),
                    "de" => Some("allemand".to_string()),
                    "zh" => Some("chinois".to_string()),
                    "fr-CA" => Some("français canadien".to_string()),
                    "zh-Hant" => Some("chinois traditionnel".to_string()),
                    "en-US" => Some("anglais américain".to_string()),
                    "zh-TW" => Some("chinois (Taïwan)".to_string()),
                    _ => None,
                },
                _ => match code {
                    "fr" => Some("French".to_string()),
                    "de" => Some("German".to_string()),
                    "zh" => Some("Chinese".to_string()),
                    "fr-CA" => Some("Canadian French".to_string()),
                    "zh-Hant" => Some("Traditional Chinese".to_string()),
                    "en-US" => Some("American English".to_string()),
                    "zh-TW" => Some("Chinese (Taiwan)".to_string()),
                    _ => None,
                },
            },
            "script" => match family {
                "zh" => match code {
                    "Latn" => Some("拉丁文".to_string()),
                    "Arab" => Some("阿拉伯文".to_string()),
                    "Kana" => Some("片假名".to_string()),
                    _ => None,
                },
                "ja" => match code {
                    "Latn" => Some("ラテン文字".to_string()),
                    "Arab" => Some("アラビア文字".to_string()),
                    "Kana" => Some("片仮名".to_string()),
                    _ => None,
                },
                "he" => match code {
                    "Latn" => Some("לטיני".to_string()),
                    "Arab" => Some("ערבי".to_string()),
                    "Kana" => Some("קטקאנה".to_string()),
                    _ => None,
                },
                "es" => match code {
                    "Latn" => Some("latín".to_string()),
                    "Arab" => Some("árabe".to_string()),
                    "Kana" => Some("katakana".to_string()),
                    _ => None,
                },
                "fr" => match code {
                    "Latn" => Some("latin".to_string()),
                    "Arab" => Some("arabe".to_string()),
                    "Kana" => Some("katakana".to_string()),
                    _ => None,
                },
                _ => match code {
                    "Latn" => Some("Latin".to_string()),
                    "Arab" => Some("Arabic".to_string()),
                    "Kana" => Some("Katakana".to_string()),
                    _ => None,
                },
            },
            "currency" => match family {
                "zh" => match code {
                    "USD" => Some("美元".to_string()),
                    "EUR" => Some("歐元".to_string()),
                    "TWD" => Some("新台幣".to_string()),
                    "CNY" => Some("人民幣".to_string()),
                    _ => None,
                },
                "ja" => match code {
                    "USD" => Some("米ドル".to_string()),
                    "EUR" => Some("ユーロ".to_string()),
                    "TWD" => Some("新台湾ドル".to_string()),
                    "CNY" => Some("中国人民元".to_string()),
                    _ => None,
                },
                "he" => match code {
                    "USD" => Some("דולר אמריקאי".to_string()),
                    "EUR" => Some("אירו".to_string()),
                    "TWD" => Some("דולר טאיוואני חדש".to_string()),
                    "CNY" => Some("יואן סיני".to_string()),
                    _ => None,
                },
                "es" => match code {
                    "USD" => Some("dólar estadounidense".to_string()),
                    "EUR" => Some("euro".to_string()),
                    "TWD" => Some("nuevo dólar taiwanés".to_string()),
                    "CNY" => Some("yuan chino".to_string()),
                    _ => None,
                },
                "fr" => match code {
                    "USD" => Some("dollar des États-Unis".to_string()),
                    "EUR" => Some("euro".to_string()),
                    "TWD" => Some("nouveau dollar taïwanais".to_string()),
                    "CNY" => Some("yuan renminbi chinois".to_string()),
                    _ => None,
                },
                _ => match code {
                    "USD" => Some("US Dollar".to_string()),
                    "EUR" => Some("Euro".to_string()),
                    "TWD" => Some("New Taiwan Dollar".to_string()),
                    "CNY" => Some("Chinese Yuan".to_string()),
                    _ => None,
                },
            },
            _ => None,
        }
    }

    pub(super) fn intl_display_names_of(
        &self,
        locale: &str,
        options: &IntlDisplayNamesOptions,
        code: &str,
    ) -> Result<Value> {
        let canonical_code =
            Self::intl_canonicalize_display_names_code(&options.display_type, code)?;
        if let Some(name) = Self::intl_display_names_lookup(locale, options, &canonical_code) {
            return Ok(Value::String(name));
        }
        if options.fallback == "none" {
            Ok(Value::Undefined)
        } else {
            Ok(Value::String(canonical_code))
        }
    }

    pub(super) fn intl_display_names_resolved_options_value(
        &self,
        locale: String,
        options: &IntlDisplayNamesOptions,
    ) -> Value {
        let mut entries = vec![
            ("locale".to_string(), Value::String(locale)),
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

    pub(super) fn intl_supported_values_of(key: &str) -> Result<Vec<String>> {
        let key = key.trim();
        let mut values = match key.to_ascii_lowercase().as_str() {
            "calendar" => vec!["gregory", "islamic-umalqura", "japanese"],
            "collation" => vec!["default", "emoji", "phonebk"],
            "currency" => vec!["EUR", "JPY", "USD"],
            "numberingsystem" => vec!["arab", "latn", "thai"],
            "timezone" => vec![
                "America/Los_Angeles",
                "America/New_York",
                "Asia/Kolkata",
                "UTC",
            ],
            "unit" => vec![
                "day", "hour", "meter", "minute", "month", "second", "week", "year",
            ],
            _ => {
                return Err(Error::ScriptRuntime(format!(
                    "RangeError: invalid key: \"{key}\""
                )));
            }
        };
        let mut values = values
            .drain(..)
            .map(str::to_string)
            .collect::<Vec<String>>();
        values.sort();
        values.dedup();
        Ok(values)
    }

    pub(super) fn new_builtin_placeholder_function() -> Value {
        Value::Function(Rc::new(FunctionValue {
            handler: ScriptHandler {
                params: Vec::new(),
                stmts: Vec::new(),
            },
            captured_env: Rc::new(RefCell::new(ScriptEnv::default())),
            captured_pending_function_decls: Vec::new(),
            captured_global_names: HashSet::new(),
            local_bindings: HashSet::new(),
            global_scope: true,
            is_async: false,
        }))
    }

    pub(super) fn intl_constructor_value(&self, constructor_name: &str) -> Value {
        let Some(Value::Object(entries)) = self.script_runtime.env.get("Intl") else {
            return Self::new_builtin_placeholder_function();
        };
        Self::object_get_entry(&entries.borrow(), constructor_name)
            .unwrap_or_else(Self::new_builtin_placeholder_function)
    }

    pub(super) fn intl_collator_options_from_value(
        &self,
        options: Option<&Value>,
    ) -> Result<(String, String)> {
        let mut case_first = "false".to_string();
        let mut sensitivity = "variant".to_string();
        let Some(options) = options else {
            return Ok((case_first, sensitivity));
        };

        match options {
            Value::Undefined | Value::Null => {}
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(value) = Self::object_get_entry(&entries, "caseFirst") {
                    if !matches!(value, Value::Undefined) {
                        let parsed = value.as_string();
                        if !matches!(parsed.as_str(), "upper" | "lower" | "false") {
                            return Err(Error::ScriptRuntime(
                                "RangeError: invalid Intl.Collator caseFirst option".into(),
                            ));
                        }
                        case_first = parsed;
                    }
                }
                if let Some(value) = Self::object_get_entry(&entries, "sensitivity") {
                    if !matches!(value, Value::Undefined) {
                        let parsed = value.as_string();
                        if !matches!(parsed.as_str(), "base" | "accent" | "case" | "variant") {
                            return Err(Error::ScriptRuntime(
                                "RangeError: invalid Intl.Collator sensitivity option".into(),
                            ));
                        }
                        sensitivity = parsed;
                    }
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "TypeError: Intl.Collator options must be an object".into(),
                ));
            }
        }

        Ok((case_first, sensitivity))
    }

    pub(super) fn new_intl_collator_compare_callable(
        &self,
        locale: String,
        case_first: String,
        sensitivity: String,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("intl_collator_compare".to_string()),
            ),
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::Collator.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_CASE_FIRST_KEY.to_string(),
                Value::String(case_first),
            ),
            (
                INTERNAL_INTL_SENSITIVITY_KEY.to_string(),
                Value::String(sensitivity),
            ),
        ])
    }

    pub(super) fn new_intl_collator_value(
        &self,
        locale: String,
        case_first: String,
        sensitivity: String,
    ) -> Value {
        let compare = self.new_intl_collator_compare_callable(
            locale.clone(),
            case_first.clone(),
            sensitivity.clone(),
        );
        Self::new_object_value(vec![
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::Collator.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_CASE_FIRST_KEY.to_string(),
                Value::String(case_first),
            ),
            (
                INTERNAL_INTL_SENSITIVITY_KEY.to_string(),
                Value::String(sensitivity),
            ),
            ("compare".to_string(), compare),
            (
                "constructor".to_string(),
                self.intl_constructor_value("Collator"),
            ),
        ])
    }

    pub(super) fn new_intl_date_time_format_callable(
        &self,
        locale: String,
        options: IntlDateTimeOptions,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("intl_date_time_format".to_string()),
            ),
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::DateTimeFormat.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_OPTIONS_KEY.to_string(),
                Self::intl_date_time_options_to_value(&options),
            ),
        ])
    }

    pub(super) fn new_intl_date_time_formatter_value(
        &self,
        locale: String,
        options: IntlDateTimeOptions,
    ) -> Value {
        let format = self.new_intl_date_time_format_callable(locale.clone(), options.clone());
        Self::new_object_value(vec![
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::DateTimeFormat.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_OPTIONS_KEY.to_string(),
                Self::intl_date_time_options_to_value(&options),
            ),
            ("format".to_string(), format),
            (
                "constructor".to_string(),
                self.intl_constructor_value("DateTimeFormat"),
            ),
        ])
    }

    pub(super) fn new_intl_display_names_value(
        &self,
        locale: String,
        options: IntlDisplayNamesOptions,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::DisplayNames.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_OPTIONS_KEY.to_string(),
                Self::intl_display_names_options_to_value(&options),
            ),
            (
                "constructor".to_string(),
                self.intl_constructor_value("DisplayNames"),
            ),
        ])
    }

    pub(super) fn new_intl_duration_format_callable(
        &self,
        locale: String,
        options: IntlDurationOptions,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("intl_duration_format".to_string()),
            ),
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::DurationFormat.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_OPTIONS_KEY.to_string(),
                Self::intl_duration_options_to_value(&options),
            ),
        ])
    }

    pub(super) fn new_intl_duration_formatter_value(
        &self,
        locale: String,
        options: IntlDurationOptions,
    ) -> Value {
        let format = self.new_intl_duration_format_callable(locale.clone(), options.clone());
        Self::new_object_value(vec![
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::DurationFormat.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_OPTIONS_KEY.to_string(),
                Self::intl_duration_options_to_value(&options),
            ),
            ("format".to_string(), format),
            (
                "constructor".to_string(),
                self.intl_constructor_value("DurationFormat"),
            ),
        ])
    }

    pub(super) fn new_intl_list_format_callable(
        &self,
        locale: String,
        options: IntlListOptions,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("intl_list_format".to_string()),
            ),
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::ListFormat.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_OPTIONS_KEY.to_string(),
                Self::intl_list_options_to_value(&options),
            ),
        ])
    }

    pub(super) fn new_intl_list_formatter_value(
        &self,
        locale: String,
        options: IntlListOptions,
    ) -> Value {
        let format = self.new_intl_list_format_callable(locale.clone(), options.clone());
        Self::new_object_value(vec![
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::ListFormat.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_OPTIONS_KEY.to_string(),
                Self::intl_list_options_to_value(&options),
            ),
            ("format".to_string(), format),
            (
                "constructor".to_string(),
                self.intl_constructor_value("ListFormat"),
            ),
        ])
    }

    pub(super) fn new_intl_plural_rules_value(
        &self,
        locale: String,
        options: IntlPluralRulesOptions,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::PluralRules.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_OPTIONS_KEY.to_string(),
                Self::intl_plural_rules_options_to_value(&options),
            ),
            (
                "constructor".to_string(),
                self.intl_constructor_value("PluralRules"),
            ),
        ])
    }

    pub(super) fn new_intl_relative_time_formatter_value(
        &self,
        locale: String,
        options: IntlRelativeTimeOptions,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(
                    IntlFormatterKind::RelativeTimeFormat
                        .storage_name()
                        .to_string(),
                ),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_OPTIONS_KEY.to_string(),
                Self::intl_relative_time_options_to_value(&options),
            ),
            (
                "constructor".to_string(),
                self.intl_constructor_value("RelativeTimeFormat"),
            ),
        ])
    }

    pub(super) fn new_intl_segmenter_segments_iterator_callable(&self, segments: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("intl_segmenter_segments_iterator".to_string()),
            ),
            (INTERNAL_INTL_SEGMENTS_KEY.to_string(), segments),
        ])
    }

    pub(super) fn new_intl_segmenter_iterator_next_callable(&self, segments: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("intl_segmenter_iterator_next".to_string()),
            ),
            (INTERNAL_INTL_SEGMENTS_KEY.to_string(), segments),
            (
                INTERNAL_INTL_SEGMENT_INDEX_KEY.to_string(),
                Value::Number(0),
            ),
        ])
    }

    pub(super) fn new_intl_segmenter_iterator_value(&self, segments: Value) -> Value {
        let next = self.new_intl_segmenter_iterator_next_callable(segments);
        Self::new_object_value(vec![("next".to_string(), next)])
    }

    pub(super) fn new_intl_segments_value(&mut self, segments: Vec<Value>) -> Value {
        let segments_array = Self::new_array_value(segments.clone());
        let iterator = self.new_intl_segmenter_segments_iterator_callable(segments_array);
        let iterator_symbol = self.eval_symbol_static_property(SymbolStaticProperty::Iterator);
        let iterator_key = self.property_key_to_storage_key(&iterator_symbol);

        let mut entries = Vec::with_capacity(segments.len() + 2);
        entries.push(("length".to_string(), Value::Number(segments.len() as i64)));
        for (index, segment) in segments.into_iter().enumerate() {
            entries.push((index.to_string(), segment));
        }
        entries.push((iterator_key, iterator));
        Self::new_object_value(entries)
    }

    pub(super) fn new_intl_segmenter_value(
        &self,
        locale: String,
        options: IntlSegmenterOptions,
    ) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::Segmenter.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
            (
                INTERNAL_INTL_OPTIONS_KEY.to_string(),
                Self::intl_segmenter_options_to_value(&options),
            ),
            (
                "constructor".to_string(),
                self.intl_constructor_value("Segmenter"),
            ),
        ])
    }

    pub(super) fn new_intl_locale_value(&self, data: IntlLocaleData) -> Value {
        let base_name = Self::intl_locale_data_base_name(&data);
        Self::new_object_value(vec![
            (
                INTERNAL_INTL_LOCALE_DATA_KEY.to_string(),
                Self::intl_locale_data_to_internal_value(&data),
            ),
            ("baseName".to_string(), Value::String(base_name)),
            ("language".to_string(), Value::String(data.language.clone())),
            (
                "script".to_string(),
                data.script
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "region".to_string(),
                data.region
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "variants".to_string(),
                Self::new_array_value(
                    data.variants
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect::<Vec<_>>(),
                ),
            ),
            (
                "calendar".to_string(),
                data.calendar
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "caseFirst".to_string(),
                data.case_first
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "collation".to_string(),
                data.collation
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "hourCycle".to_string(),
                data.hour_cycle
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "numberingSystem".to_string(),
                data.numbering_system
                    .as_ref()
                    .map_or(Value::Undefined, |value| Value::String(value.clone())),
            ),
            (
                "numeric".to_string(),
                data.numeric.map_or(Value::Undefined, Value::Bool),
            ),
            (
                "constructor".to_string(),
                self.intl_constructor_value("Locale"),
            ),
        ])
    }

    pub(super) fn new_intl_number_format_callable(&self, locale: String) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("intl_number_format".to_string()),
            ),
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(IntlFormatterKind::NumberFormat.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
        ])
    }

    pub(super) fn new_intl_formatter_value(
        &self,
        kind: IntlFormatterKind,
        locale: String,
    ) -> Value {
        let mut entries = vec![
            (
                INTERNAL_INTL_KIND_KEY.to_string(),
                Value::String(kind.storage_name().to_string()),
            ),
            (INTERNAL_INTL_LOCALE_KEY.to_string(), Value::String(locale)),
        ];
        if kind == IntlFormatterKind::NumberFormat {
            let locale = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_KEY)
                .and_then(|value| match value {
                    Value::String(value) => Some(value),
                    _ => None,
                })
                .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
            entries.push((
                "format".to_string(),
                self.new_intl_number_format_callable(locale),
            ));
            entries.push((
                "constructor".to_string(),
                self.intl_constructor_value("NumberFormat"),
            ));
        }
        Self::new_object_value(entries)
    }

    pub(super) fn resolve_intl_formatter(
        &self,
        value: &Value,
    ) -> Result<(IntlFormatterKind, String)> {
        let Value::Object(entries) = value else {
            return Err(Error::ScriptRuntime(
                "Intl formatter format requires an Intl formatter instance".into(),
            ));
        };
        let entries = entries.borrow();
        let kind = Self::object_get_entry(&entries, INTERNAL_INTL_KIND_KEY)
            .and_then(|value| match value {
                Value::String(value) => IntlFormatterKind::from_storage_name(&value),
                _ => None,
            })
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "Intl formatter format requires an Intl formatter instance".into(),
                )
            })?;
        let locale = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_KEY)
            .and_then(|value| match value {
                Value::String(value) => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
        Ok((kind, locale))
    }

    pub(super) fn resolve_intl_date_time_options(
        &self,
        value: &Value,
    ) -> Result<(String, IntlDateTimeOptions)> {
        let Value::Object(entries) = value else {
            return Err(Error::ScriptRuntime(
                "Intl.DateTimeFormat method requires an Intl.DateTimeFormat instance".into(),
            ));
        };
        let entries = entries.borrow();
        let kind = Self::object_get_entry(&entries, INTERNAL_INTL_KIND_KEY)
            .and_then(|value| match value {
                Value::String(value) => IntlFormatterKind::from_storage_name(&value),
                _ => None,
            })
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "Intl.DateTimeFormat method requires an Intl.DateTimeFormat instance".into(),
                )
            })?;
        if kind != IntlFormatterKind::DateTimeFormat {
            return Err(Error::ScriptRuntime(
                "Intl.DateTimeFormat method requires an Intl.DateTimeFormat instance".into(),
            ));
        }
        let locale = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_KEY)
            .and_then(|value| match value {
                Value::String(value) => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
        let options = Self::intl_date_time_options_from_internal(&entries);
        Ok((locale, options))
    }

    pub(super) fn resolve_intl_duration_options(
        &self,
        value: &Value,
    ) -> Result<(String, IntlDurationOptions)> {
        let Value::Object(entries) = value else {
            return Err(Error::ScriptRuntime(
                "Intl.DurationFormat method requires an Intl.DurationFormat instance".into(),
            ));
        };
        let entries = entries.borrow();
        let kind = Self::object_get_entry(&entries, INTERNAL_INTL_KIND_KEY)
            .and_then(|value| match value {
                Value::String(value) => IntlFormatterKind::from_storage_name(&value),
                _ => None,
            })
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "Intl.DurationFormat method requires an Intl.DurationFormat instance".into(),
                )
            })?;
        if kind != IntlFormatterKind::DurationFormat {
            return Err(Error::ScriptRuntime(
                "Intl.DurationFormat method requires an Intl.DurationFormat instance".into(),
            ));
        }
        let locale = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_KEY)
            .and_then(|value| match value {
                Value::String(value) => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
        let options = Self::intl_duration_options_from_internal(&entries);
        Ok((locale, options))
    }

    pub(super) fn resolve_intl_list_options(
        &self,
        value: &Value,
    ) -> Result<(String, IntlListOptions)> {
        let Value::Object(entries) = value else {
            return Err(Error::ScriptRuntime(
                "Intl.ListFormat method requires an Intl.ListFormat instance".into(),
            ));
        };
        let entries = entries.borrow();
        let kind = Self::object_get_entry(&entries, INTERNAL_INTL_KIND_KEY)
            .and_then(|value| match value {
                Value::String(value) => IntlFormatterKind::from_storage_name(&value),
                _ => None,
            })
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "Intl.ListFormat method requires an Intl.ListFormat instance".into(),
                )
            })?;
        if kind != IntlFormatterKind::ListFormat {
            return Err(Error::ScriptRuntime(
                "Intl.ListFormat method requires an Intl.ListFormat instance".into(),
            ));
        }
        let locale = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_KEY)
            .and_then(|value| match value {
                Value::String(value) => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
        let options = Self::intl_list_options_from_internal(&entries);
        Ok((locale, options))
    }

    pub(super) fn resolve_intl_plural_rules_options(
        &self,
        value: &Value,
    ) -> Result<(String, IntlPluralRulesOptions)> {
        let Value::Object(entries) = value else {
            return Err(Error::ScriptRuntime(
                "Intl.PluralRules method requires an Intl.PluralRules instance".into(),
            ));
        };
        let entries = entries.borrow();
        let kind = Self::object_get_entry(&entries, INTERNAL_INTL_KIND_KEY)
            .and_then(|value| match value {
                Value::String(value) => IntlFormatterKind::from_storage_name(&value),
                _ => None,
            })
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "Intl.PluralRules method requires an Intl.PluralRules instance".into(),
                )
            })?;
        if kind != IntlFormatterKind::PluralRules {
            return Err(Error::ScriptRuntime(
                "Intl.PluralRules method requires an Intl.PluralRules instance".into(),
            ));
        }
        let locale = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_KEY)
            .and_then(|value| match value {
                Value::String(value) => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
        let options = Self::intl_plural_rules_options_from_internal(&entries);
        Ok((locale, options))
    }

    pub(super) fn resolve_intl_relative_time_options(
        &self,
        value: &Value,
    ) -> Result<(String, IntlRelativeTimeOptions)> {
        let Value::Object(entries) = value else {
            return Err(Error::ScriptRuntime(
                "Intl.RelativeTimeFormat method requires an Intl.RelativeTimeFormat instance"
                    .into(),
            ));
        };
        let entries = entries.borrow();
        let kind = Self::object_get_entry(&entries, INTERNAL_INTL_KIND_KEY)
            .and_then(|value| match value {
                Value::String(value) => IntlFormatterKind::from_storage_name(&value),
                _ => None,
            })
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "Intl.RelativeTimeFormat method requires an Intl.RelativeTimeFormat instance"
                        .into(),
                )
            })?;
        if kind != IntlFormatterKind::RelativeTimeFormat {
            return Err(Error::ScriptRuntime(
                "Intl.RelativeTimeFormat method requires an Intl.RelativeTimeFormat instance"
                    .into(),
            ));
        }
        let locale = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_KEY)
            .and_then(|value| match value {
                Value::String(value) => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
        let options = Self::intl_relative_time_options_from_internal(&entries);
        Ok((locale, options))
    }

    pub(super) fn resolve_intl_segmenter_options(
        &self,
        value: &Value,
    ) -> Result<(String, IntlSegmenterOptions)> {
        let Value::Object(entries) = value else {
            return Err(Error::ScriptRuntime(
                "Intl.Segmenter method requires an Intl.Segmenter instance".into(),
            ));
        };
        let entries = entries.borrow();
        let kind = Self::object_get_entry(&entries, INTERNAL_INTL_KIND_KEY)
            .and_then(|value| match value {
                Value::String(value) => IntlFormatterKind::from_storage_name(&value),
                _ => None,
            })
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "Intl.Segmenter method requires an Intl.Segmenter instance".into(),
                )
            })?;
        if kind != IntlFormatterKind::Segmenter {
            return Err(Error::ScriptRuntime(
                "Intl.Segmenter method requires an Intl.Segmenter instance".into(),
            ));
        }
        let locale = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_KEY)
            .and_then(|value| match value {
                Value::String(value) => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
        let options = Self::intl_segmenter_options_from_internal(&entries);
        Ok((locale, options))
    }

    pub(super) fn resolve_intl_locale_data(&self, value: &Value) -> Result<IntlLocaleData> {
        let Value::Object(entries) = value else {
            return Err(Error::ScriptRuntime(
                "Intl.Locale method requires an Intl.Locale instance".into(),
            ));
        };
        let entries = entries.borrow();
        let data_value = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_DATA_KEY)
            .ok_or_else(|| {
                Error::ScriptRuntime("Intl.Locale method requires an Intl.Locale instance".into())
            })?;
        Self::intl_locale_data_from_internal_value(&data_value).ok_or_else(|| {
            Error::ScriptRuntime("Intl.Locale method requires an Intl.Locale instance".into())
        })
    }

    pub(super) fn resolve_intl_display_names_options(
        &self,
        value: &Value,
    ) -> Result<(String, IntlDisplayNamesOptions)> {
        let Value::Object(entries) = value else {
            return Err(Error::ScriptRuntime(
                "Intl.DisplayNames method requires an Intl.DisplayNames instance".into(),
            ));
        };
        let entries = entries.borrow();
        let kind = Self::object_get_entry(&entries, INTERNAL_INTL_KIND_KEY)
            .and_then(|value| match value {
                Value::String(value) => IntlFormatterKind::from_storage_name(&value),
                _ => None,
            })
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "Intl.DisplayNames method requires an Intl.DisplayNames instance".into(),
                )
            })?;
        if kind != IntlFormatterKind::DisplayNames {
            return Err(Error::ScriptRuntime(
                "Intl.DisplayNames method requires an Intl.DisplayNames instance".into(),
            ));
        }
        let locale = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_KEY)
            .and_then(|value| match value {
                Value::String(value) => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
        let options = Self::intl_display_names_options_from_internal(&entries);
        Ok((locale, options))
    }

    pub(super) fn resolve_intl_collator_options(
        &self,
        value: &Value,
    ) -> Result<(String, String, String)> {
        let Value::Object(entries) = value else {
            return Err(Error::ScriptRuntime(
                "Intl.Collator.compare requires an Intl.Collator instance".into(),
            ));
        };
        let entries = entries.borrow();
        let kind = Self::object_get_entry(&entries, INTERNAL_INTL_KIND_KEY)
            .and_then(|value| match value {
                Value::String(value) => IntlFormatterKind::from_storage_name(&value),
                _ => None,
            })
            .ok_or_else(|| {
                Error::ScriptRuntime(
                    "Intl.Collator.compare requires an Intl.Collator instance".into(),
                )
            })?;
        if kind != IntlFormatterKind::Collator {
            return Err(Error::ScriptRuntime(
                "Intl.Collator.compare requires an Intl.Collator instance".into(),
            ));
        }
        let locale = Self::object_get_entry(&entries, INTERNAL_INTL_LOCALE_KEY)
            .and_then(|value| match value {
                Value::String(value) => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
        let case_first = Self::object_get_entry(&entries, INTERNAL_INTL_CASE_FIRST_KEY)
            .and_then(|value| match value {
                Value::String(value) => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| "false".to_string());
        let sensitivity = Self::object_get_entry(&entries, INTERNAL_INTL_SENSITIVITY_KEY)
            .and_then(|value| match value {
                Value::String(value) => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| "variant".to_string());
        Ok((locale, case_first, sensitivity))
    }

    pub(super) fn intl_collator_compare_strings(
        left: &str,
        right: &str,
        locale: &str,
        case_first: &str,
        sensitivity: &str,
    ) -> i64 {
        let case_priority = match case_first {
            "upper" => 0i32,
            _ => 1i32,
        };

        let mut left_chars = left.chars();
        let mut right_chars = right.chars();
        loop {
            match (left_chars.next(), right_chars.next()) {
                (Some(left), Some(right)) => {
                    let (lp, ls, lc) = Self::intl_collator_char_key(left, locale, case_priority);
                    let (rp, rs, rc) = Self::intl_collator_char_key(right, locale, case_priority);

                    if lp != rp {
                        return if lp < rp { -1 } else { 1 };
                    }
                    if matches!(sensitivity, "accent" | "variant") && ls != rs {
                        return if ls < rs { -1 } else { 1 };
                    }
                    if matches!(sensitivity, "case" | "variant") && lc != rc {
                        return if lc < rc { -1 } else { 1 };
                    }
                }
                (Some(_), None) => return 1,
                (None, Some(_)) => return -1,
                (None, None) => return 0,
            }
        }
    }

    pub(super) fn intl_collator_char_key(
        ch: char,
        locale: &str,
        case_priority: i32,
    ) -> (i32, i32, i32) {
        let lower = ch.to_ascii_lowercase();
        let is_upper = ch.is_ascii_uppercase();

        let (primary, secondary) = if Self::intl_locale_family(locale) == "sv" {
            match lower {
                'a'..='z' => ((lower as u32 - 'a' as u32 + 1) as i32, 0),
                'å' => (27, 0),
                'ä' => (28, 0),
                'ö' => (29, 0),
                _ => (1000 + lower as i32, 0),
            }
        } else {
            match lower {
                'a'..='z' => ((lower as u32 - 'a' as u32 + 1) as i32, 0),
                'ä' => (1, 1),
                'ö' => (15, 1),
                'ü' => (21, 1),
                'ß' => (19, 1),
                _ => (1000 + lower as i32, 0),
            }
        };

        let case_rank = if case_priority == 0 {
            if is_upper { 0 } else { 1 }
        } else if is_upper {
            1
        } else {
            0
        };
        (primary, secondary, case_rank)
    }
}
