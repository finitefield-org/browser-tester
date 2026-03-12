use super::*;

impl Harness {
    pub(crate) fn resolve_intl_plural_rules_options(
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

    pub(crate) fn resolve_intl_relative_time_options(
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
        let options = Self::intl_relative_time_options_from_internal(&entries, &locale);
        Ok((locale, options))
    }

    pub(crate) fn resolve_intl_segmenter_options(
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

    pub(crate) fn resolve_intl_locale_data(&self, value: &Value) -> Result<IntlLocaleData> {
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

    pub(crate) fn resolve_intl_display_names_options(
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

    pub(crate) fn resolve_intl_collator_options(
        &self,
        value: &Value,
    ) -> Result<(String, String, String, bool)> {
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
        let numeric = Self::object_get_entry(&entries, INTERNAL_INTL_NUMERIC_KEY)
            .is_some_and(|value| value.truthy());
        Ok((locale, case_first, sensitivity, numeric))
    }

    pub(crate) fn intl_collator_compare_strings(
        left: &str,
        right: &str,
        locale: &str,
        case_first: &str,
        sensitivity: &str,
        numeric: bool,
    ) -> i64 {
        let case_priority = match case_first {
            "upper" => 0i32,
            _ => 1i32,
        };

        let left_chars = left.chars().collect::<Vec<_>>();
        let right_chars = right.chars().collect::<Vec<_>>();
        let mut left_index = 0usize;
        let mut right_index = 0usize;
        loop {
            if numeric
                && left_chars
                    .get(left_index)
                    .is_some_and(|ch| ch.to_digit(10).is_some())
                && right_chars
                    .get(right_index)
                    .is_some_and(|ch| ch.to_digit(10).is_some())
            {
                let left_start = left_index;
                while left_index < left_chars.len() && left_chars[left_index].to_digit(10).is_some()
                {
                    left_index += 1;
                }

                let right_start = right_index;
                while right_index < right_chars.len()
                    && right_chars[right_index].to_digit(10).is_some()
                {
                    right_index += 1;
                }

                let numeric_cmp = Self::intl_collator_compare_digit_runs(
                    &left_chars[left_start..left_index],
                    &right_chars[right_start..right_index],
                );
                if numeric_cmp != 0 {
                    return numeric_cmp;
                }
                continue;
            }

            match (left_chars.get(left_index), right_chars.get(right_index)) {
                (Some(left), Some(right)) => {
                    left_index += 1;
                    right_index += 1;
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

    fn intl_collator_compare_digit_runs(left: &[char], right: &[char]) -> i64 {
        let left_significant = left
            .iter()
            .position(|ch| ch.to_digit(10).unwrap_or(0) != 0)
            .unwrap_or(left.len());
        let right_significant = right
            .iter()
            .position(|ch| ch.to_digit(10).unwrap_or(0) != 0)
            .unwrap_or(right.len());

        let left_len = left.len().saturating_sub(left_significant);
        let right_len = right.len().saturating_sub(right_significant);
        if left_len != right_len {
            return if left_len < right_len { -1 } else { 1 };
        }

        for (left_digit, right_digit) in left[left_significant..]
            .iter()
            .zip(right[right_significant..].iter())
        {
            let left_digit = left_digit.to_digit(10).unwrap_or(0);
            let right_digit = right_digit.to_digit(10).unwrap_or(0);
            if left_digit != right_digit {
                return if left_digit < right_digit { -1 } else { 1 };
            }
        }

        0
    }

    pub(crate) fn intl_collator_char_key(
        ch: &char,
        locale: &str,
        case_priority: i32,
    ) -> (i32, i32, i32) {
        let lower = ch.to_lowercase().next().unwrap_or(*ch);
        let is_upper = ch.is_uppercase();

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
