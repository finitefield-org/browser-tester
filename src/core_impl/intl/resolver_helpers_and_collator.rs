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
        let options = Self::intl_relative_time_options_from_internal(&entries);
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

    pub(crate) fn intl_collator_compare_strings(
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

    pub(crate) fn intl_collator_char_key(
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
