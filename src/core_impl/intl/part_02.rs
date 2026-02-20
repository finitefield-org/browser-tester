use super::*;

impl Harness {
    pub(crate) fn intl_date_time_components(
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

    pub(crate) fn intl_default_hour12(locale: &str) -> bool {
        let family = Self::intl_locale_family(locale);
        if family != "en" {
            return false;
        }
        !matches!(Self::intl_locale_region(locale), Some("GB"))
    }

    pub(crate) fn intl_month_name(locale: &str, month: u32, width: &str) -> String {
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

    pub(crate) fn intl_weekday_name(locale: &str, weekday: u32, width: &str) -> String {
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

    pub(crate) fn intl_apply_numbering_system(text: &str, numbering_system: &str) -> String {
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

    pub(crate) fn intl_format_date_component_year(
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

    pub(crate) fn intl_format_date_component_month(
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

    pub(crate) fn intl_format_date_component_day(
        options: &IntlDateTimeOptions,
        components: &IntlDateTimeComponents,
    ) -> String {
        match options.day.as_deref() {
            Some("2-digit") => format!("{:02}", components.day),
            _ => components.day.to_string(),
        }
    }

    pub(crate) fn intl_append_date_parts(
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

    pub(crate) fn intl_day_period_label(hour: u32, width: &str) -> String {
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

    pub(crate) fn intl_time_zone_name(
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

    pub(crate) fn intl_append_time_parts(
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

    pub(crate) fn intl_expand_date_time_styles(
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

    pub(crate) fn intl_format_date_time_to_parts(
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

    pub(crate) fn intl_format_date_time(
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
}
