use super::super::*;

impl Harness {
    pub(crate) fn format_number_to_locale_string(
        value: f64,
        locale: &str,
        minimum_fraction_digits: Option<usize>,
        maximum_fraction_digits: Option<usize>,
    ) -> String {
        let rendered = Self::format_number_with_fraction_constraints(
            value,
            minimum_fraction_digits,
            maximum_fraction_digits,
        );
        Self::format_preformatted_number_for_locale(&rendered, locale, None)
    }

    fn format_number_with_fraction_constraints(
        value: f64,
        minimum_fraction_digits: Option<usize>,
        maximum_fraction_digits: Option<usize>,
    ) -> String {
        if !value.is_finite() {
            return Self::format_number_default(value);
        }

        let mut rendered = if let Some(maximum_fraction_digits) = maximum_fraction_digits {
            let mut rounded = Self::number_to_fixed(value, maximum_fraction_digits);
            if let Some(dot_index) = rounded.find('.') {
                let minimum_kept = minimum_fraction_digits
                    .unwrap_or(0)
                    .min(maximum_fraction_digits);
                let mut fraction_len = rounded.len().saturating_sub(dot_index + 1);
                while fraction_len > minimum_kept && rounded.ends_with('0') {
                    rounded.pop();
                    fraction_len -= 1;
                }
                if fraction_len == 0 && rounded.ends_with('.') {
                    rounded.pop();
                }
            }
            rounded
        } else {
            Self::format_number_default(value)
        };

        if let Some(minimum_fraction_digits) = minimum_fraction_digits {
            if !rendered.contains('e') && !rendered.contains('E') {
                let existing = if let Some(dot_index) = rendered.find('.') {
                    rendered.len().saturating_sub(dot_index + 1)
                } else {
                    rendered.push('.');
                    0
                };
                if minimum_fraction_digits > existing {
                    rendered.push_str(&"0".repeat(minimum_fraction_digits - existing));
                }
            }
        }

        rendered
    }

    fn round_to_max_significant_digits(value: f64, maximum_significant_digits: usize) -> f64 {
        if !value.is_finite() || value == 0.0 {
            return value;
        }
        let abs = value.abs();
        let exponent = abs.log10().floor() as i32;
        let scale = maximum_significant_digits as i32 - exponent - 1;
        if scale >= 0 {
            let factor = 10f64.powi(scale);
            (value * factor).round() / factor
        } else {
            let factor = 10f64.powi(-scale);
            (value / factor).round() * factor
        }
    }

    fn intl_apply_number_style(
        &self,
        numeric: String,
        locale: &str,
        options: &IntlNumberFormatOptions,
        numeric_hint: f64,
    ) -> String {
        match options.style.as_str() {
            "currency" => {
                let currency = options.currency.as_deref().unwrap_or("USD");
                let symbol = Self::intl_currency_symbol(currency);
                let family = Self::intl_locale_family(locale);
                if matches!(family, "de" | "id" | "pt") {
                    format!("{numeric} {symbol}")
                } else {
                    format!("{symbol}{numeric}")
                }
            }
            "unit" => {
                let unit = options.unit.as_deref().unwrap_or_default();
                let label =
                    Self::intl_unit_label(locale, unit, &options.unit_display, numeric_hint)
                        .unwrap_or_else(|_| unit.to_string());
                format!("{numeric} {label}")
            }
            _ => numeric,
        }
    }

    fn intl_apply_number_parts_style(
        &self,
        mut parts: Vec<IntlPart>,
        locale: &str,
        options: &IntlNumberFormatOptions,
        numeric_hint: f64,
    ) -> Vec<IntlPart> {
        match options.style.as_str() {
            "currency" => {
                let symbol =
                    Self::intl_currency_symbol(options.currency.as_deref().unwrap_or("USD"));
                let family = Self::intl_locale_family(locale);
                if matches!(family, "de" | "id" | "pt") {
                    parts.push(IntlPart {
                        part_type: "literal".to_string(),
                        value: " ".to_string(),
                    });
                    parts.push(IntlPart {
                        part_type: "currency".to_string(),
                        value: symbol,
                    });
                } else {
                    let mut prefixed = Vec::with_capacity(parts.len() + 1);
                    prefixed.push(IntlPart {
                        part_type: "currency".to_string(),
                        value: symbol,
                    });
                    prefixed.extend(parts);
                    parts = prefixed;
                }
            }
            "unit" => {
                let unit = options.unit.as_deref().unwrap_or_default();
                let label =
                    Self::intl_unit_label(locale, unit, &options.unit_display, numeric_hint)
                        .unwrap_or_else(|_| unit.to_string());
                parts.push(IntlPart {
                    part_type: "literal".to_string(),
                    value: " ".to_string(),
                });
                parts.push(IntlPart {
                    part_type: "unit".to_string(),
                    value: label,
                });
            }
            _ => {}
        }
        parts
    }

    pub(crate) fn intl_format_number_value_with_options(
        &self,
        value: &Value,
        locale: &str,
        options: &IntlNumberFormatOptions,
    ) -> String {
        if options.maximum_significant_digits.is_none() {
            if let Some(exact) = Self::exact_numeric_string_from_value(value) {
                if let Some(constrained) =
                    Self::format_exact_decimal_string_with_fraction_constraints(
                        &exact,
                        options.minimum_fraction_digits,
                        options.maximum_fraction_digits,
                    )
                {
                    let numeric = Self::format_preformatted_number_for_locale(
                        &constrained,
                        locale,
                        Some(&options.numbering_system),
                    );
                    let numeric_hint = Self::coerce_number_for_global(value);
                    return self.intl_apply_number_style(numeric, locale, options, numeric_hint);
                }
            }
        }

        let number = Self::coerce_number_for_global(value);
        self.intl_format_number_with_options(number, locale, options)
    }

    pub(crate) fn intl_number_format_value_to_parts(
        &self,
        value: &Value,
        locale: &str,
        options: &IntlNumberFormatOptions,
    ) -> Vec<IntlPart> {
        if options.maximum_significant_digits.is_none() {
            if let Some(exact) = Self::exact_numeric_string_from_value(value) {
                if let Some(constrained) =
                    Self::format_exact_decimal_string_with_fraction_constraints(
                        &exact,
                        options.minimum_fraction_digits,
                        options.maximum_fraction_digits,
                    )
                {
                    let numeric = Self::format_preformatted_number_for_locale(
                        &constrained,
                        locale,
                        Some(&options.numbering_system),
                    );
                    let parts = Self::intl_number_numeric_parts(&numeric, locale);
                    let numeric_hint = Self::coerce_number_for_global(value);
                    return self.intl_apply_number_parts_style(
                        parts,
                        locale,
                        options,
                        numeric_hint,
                    );
                }
            }
        }

        let number = Self::coerce_number_for_global(value);
        self.intl_number_format_to_parts(number, locale, options)
    }

    pub(crate) fn intl_format_number_with_options(
        &self,
        value: f64,
        locale: &str,
        options: &IntlNumberFormatOptions,
    ) -> String {
        let default_numbering_system = Self::intl_locale_unicode_extension_value(locale, "nu")
            .unwrap_or_else(|| Self::intl_default_numbering_system_for_locale(locale));
        if options.style == "decimal"
            && options.maximum_significant_digits.is_none()
            && options.numbering_system == default_numbering_system
        {
            if options.minimum_fraction_digits.is_none()
                && options.maximum_fraction_digits.is_none()
            {
                return Self::intl_format_number_for_locale(value, locale);
            }
            return Self::format_number_to_locale_string(
                value,
                locale,
                options.minimum_fraction_digits,
                options.maximum_fraction_digits,
            );
        }

        let numeric = if let Some(maximum_significant_digits) = options.maximum_significant_digits {
            let rounded = Self::round_to_max_significant_digits(value, maximum_significant_digits);
            let rendered = Self::format_number_with_fraction_constraints(
                rounded,
                options.minimum_fraction_digits,
                options.maximum_fraction_digits,
            );
            Self::format_preformatted_number_for_locale(
                &rendered,
                locale,
                Some(&options.numbering_system),
            )
        } else {
            let rendered = Self::format_number_with_fraction_constraints(
                value,
                options.minimum_fraction_digits,
                options.maximum_fraction_digits,
            );
            Self::format_preformatted_number_for_locale(
                &rendered,
                locale,
                Some(&options.numbering_system),
            )
        };
        self.intl_apply_number_style(numeric, locale, options, value)
    }

    pub(crate) fn intl_number_format_to_parts(
        &self,
        value: f64,
        locale: &str,
        options: &IntlNumberFormatOptions,
    ) -> Vec<IntlPart> {
        let numeric = if let Some(maximum_significant_digits) = options.maximum_significant_digits {
            let rounded = Self::round_to_max_significant_digits(value, maximum_significant_digits);
            let rendered = Self::format_number_with_fraction_constraints(
                rounded,
                options.minimum_fraction_digits,
                options.maximum_fraction_digits,
            );
            Self::format_preformatted_number_for_locale(
                &rendered,
                locale,
                Some(&options.numbering_system),
            )
        } else {
            let rendered = Self::format_number_with_fraction_constraints(
                value,
                options.minimum_fraction_digits,
                options.maximum_fraction_digits,
            );
            Self::format_preformatted_number_for_locale(
                &rendered,
                locale,
                Some(&options.numbering_system),
            )
        };

        let parts = Self::intl_number_numeric_parts(&numeric, locale);
        self.intl_apply_number_parts_style(parts, locale, options, value)
    }

    fn intl_number_numeric_parts(numeric: &str, locale: &str) -> Vec<IntlPart> {
        if numeric == "NaN" {
            return vec![IntlPart {
                part_type: "nan".to_string(),
                value: "NaN".to_string(),
            }];
        }
        if numeric == "Infinity" {
            return vec![IntlPart {
                part_type: "infinity".to_string(),
                value: "Infinity".to_string(),
            }];
        }
        if numeric == "-Infinity" {
            return vec![
                IntlPart {
                    part_type: "minusSign".to_string(),
                    value: "-".to_string(),
                },
                IntlPart {
                    part_type: "infinity".to_string(),
                    value: "Infinity".to_string(),
                },
            ];
        }

        let family = Self::intl_locale_family(locale);
        let (group_sep, decimal_sep) = if matches!(family, "de" | "id" | "pt") {
            ('.', ',')
        } else if family == "ar" {
            ('٬', '٫')
        } else {
            (',', '.')
        };

        let mut out = Vec::new();
        let mut chars = numeric.chars().peekable();
        if chars.peek().is_some_and(|ch| *ch == '-') {
            out.push(IntlPart {
                part_type: "minusSign".to_string(),
                value: "-".to_string(),
            });
            chars.next();
        }

        let mut current = String::new();
        let mut in_fraction = false;
        for ch in chars {
            if ch == group_sep {
                if !current.is_empty() {
                    out.push(IntlPart {
                        part_type: if in_fraction {
                            "fraction".to_string()
                        } else {
                            "integer".to_string()
                        },
                        value: current.clone(),
                    });
                    current.clear();
                }
                out.push(IntlPart {
                    part_type: "group".to_string(),
                    value: ch.to_string(),
                });
                continue;
            }
            if ch == decimal_sep {
                if !current.is_empty() {
                    out.push(IntlPart {
                        part_type: "integer".to_string(),
                        value: current.clone(),
                    });
                    current.clear();
                }
                out.push(IntlPart {
                    part_type: "decimal".to_string(),
                    value: ch.to_string(),
                });
                in_fraction = true;
                continue;
            }
            current.push(ch);
        }
        if !current.is_empty() {
            out.push(IntlPart {
                part_type: if in_fraction {
                    "fraction".to_string()
                } else {
                    "integer".to_string()
                },
                value: current,
            });
        }
        out
    }

    pub(crate) fn intl_format_number_range(
        &self,
        start: f64,
        end: f64,
        locale: &str,
        options: &IntlNumberFormatOptions,
    ) -> String {
        let start = self.intl_format_number_with_options(start, locale, options);
        let end = self.intl_format_number_with_options(end, locale, options);
        if start == end {
            start
        } else {
            format!("{start} - {end}")
        }
    }

    pub(crate) fn intl_format_number_range_to_parts(
        &self,
        start: f64,
        end: f64,
        locale: &str,
        options: &IntlNumberFormatOptions,
    ) -> (Vec<IntlPart>, Vec<String>) {
        let start = self.intl_number_format_to_parts(start, locale, options);
        let end = self.intl_number_format_to_parts(end, locale, options);
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

    pub(crate) fn format_preformatted_number_for_locale(
        rendered: &str,
        locale: &str,
        numbering_system_override: Option<&str>,
    ) -> String {
        let negative = rendered.starts_with('-');
        let unsigned = if negative { &rendered[1..] } else { rendered };

        let family = Self::intl_locale_family(locale);
        let region = Self::intl_locale_region(locale);
        let (group_sep, decimal_sep) = if matches!(family, "de" | "id" | "pt") {
            ('.', ',')
        } else if family == "ar" {
            ('٬', '٫')
        } else {
            (',', '.')
        };
        let use_indian_grouping = matches!(region, Some("IN"));
        let numbering_system = numbering_system_override
            .map(|value| value.to_string())
            .or_else(|| Self::intl_locale_unicode_extension_value(locale, "nu"))
            .unwrap_or_else(|| Self::intl_default_numbering_system_for_locale(locale));

        if unsigned.contains('e') || unsigned.contains('E') {
            let mut out = unsigned.to_string();
            if decimal_sep != '.' {
                out = out.replacen('.', &decimal_sep.to_string(), 1);
            }
            out = Self::apply_numbering_system_digits(&out, &numbering_system);
            if negative && !Self::rendered_number_is_zero(&out) {
                return format!("-{out}");
            }
            return out;
        }

        let mut parts = unsigned.splitn(2, '.');
        let integer = parts.next().unwrap_or_default();
        let fraction = parts.next();

        let mut out = if use_indian_grouping {
            Self::group_integer_indian(integer, group_sep)
        } else {
            Self::group_integer_standard(integer, group_sep)
        };
        if out.is_empty() {
            out.push('0');
        }

        if let Some(fraction) = fraction {
            if !fraction.is_empty() {
                out.push(decimal_sep);
                out.push_str(fraction);
            }
        }
        out = Self::apply_numbering_system_digits(&out, &numbering_system);

        if negative && !Self::rendered_number_is_zero(&out) {
            format!("-{out}")
        } else {
            out
        }
    }

    fn group_integer_standard(integer: &str, group_sep: char) -> String {
        let mut grouped = String::new();
        for (index, ch) in integer.chars().rev().enumerate() {
            if index > 0 && index % 3 == 0 {
                grouped.push(group_sep);
            }
            grouped.push(ch);
        }
        grouped.chars().rev().collect::<String>()
    }

    fn group_integer_indian(integer: &str, group_sep: char) -> String {
        if integer.len() <= 3 {
            return integer.to_string();
        }
        let split = integer.len() - 3;
        let (head, tail) = integer.split_at(split);
        let mut grouped_head_rev = String::new();
        for (index, ch) in head.chars().rev().enumerate() {
            if index > 0 && index % 2 == 0 {
                grouped_head_rev.push(group_sep);
            }
            grouped_head_rev.push(ch);
        }
        let mut out = grouped_head_rev.chars().rev().collect::<String>();
        out.push(group_sep);
        out.push_str(tail);
        out
    }

    fn apply_numbering_system_digits(text: &str, numbering_system: &str) -> String {
        match numbering_system {
            "arab" => text
                .chars()
                .map(|ch| match ch {
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
                    other => other,
                })
                .collect(),
            "hanidec" => text
                .chars()
                .map(|ch| match ch {
                    '0' => '〇',
                    '1' => '一',
                    '2' => '二',
                    '3' => '三',
                    '4' => '四',
                    '5' => '五',
                    '6' => '六',
                    '7' => '七',
                    '8' => '八',
                    '9' => '九',
                    other => other,
                })
                .collect(),
            _ => text.to_string(),
        }
    }
}
