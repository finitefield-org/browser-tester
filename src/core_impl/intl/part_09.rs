use super::*;

impl Harness {
    pub(crate) fn new_intl_list_formatter_value(
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

    pub(crate) fn new_intl_plural_rules_value(
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

    pub(crate) fn new_intl_relative_time_formatter_value(
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

    pub(crate) fn new_intl_segmenter_segments_iterator_callable(&self, segments: Value) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_CALLABLE_KIND_KEY.to_string(),
                Value::String("intl_segmenter_segments_iterator".to_string()),
            ),
            (INTERNAL_INTL_SEGMENTS_KEY.to_string(), segments),
        ])
    }

    pub(crate) fn new_intl_segmenter_iterator_next_callable(&self, segments: Value) -> Value {
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

    pub(crate) fn new_intl_segmenter_iterator_value(&self, segments: Value) -> Value {
        let next = self.new_intl_segmenter_iterator_next_callable(segments);
        Self::new_object_value(vec![("next".to_string(), next)])
    }

    pub(crate) fn new_intl_segments_value(&mut self, segments: Vec<Value>) -> Value {
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

    pub(crate) fn new_intl_segmenter_value(
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

    pub(crate) fn new_intl_locale_value(&self, data: IntlLocaleData) -> Value {
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

    pub(crate) fn new_intl_number_format_callable(&self, locale: String) -> Value {
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

    pub(crate) fn new_intl_formatter_value(
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

    pub(crate) fn resolve_intl_formatter(
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

    pub(crate) fn resolve_intl_date_time_options(
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

    pub(crate) fn resolve_intl_duration_options(
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

    pub(crate) fn resolve_intl_list_options(
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
}
