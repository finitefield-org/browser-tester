use super::*;

impl Harness {
    pub(crate) fn intl_segment_words(&self, locale: &str, input: &str) -> Vec<Value> {
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

    pub(crate) fn intl_segment_sentences(&self, input: &str) -> Vec<Value> {
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

    pub(crate) fn intl_segment_input(
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

    pub(crate) fn intl_segmenter_resolved_options_value(
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

    pub(crate) fn intl_locale_normalize_language(value: &str) -> Option<String> {
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

    pub(crate) fn intl_locale_normalize_script(value: &str) -> Option<String> {
        let value = value.trim();
        if value.len() != 4 || !value.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return None;
        }
        let mut chars = value.chars();
        let first = chars.next().unwrap_or_default().to_ascii_uppercase();
        Some(format!("{first}{}", chars.as_str().to_ascii_lowercase()))
    }

    pub(crate) fn intl_locale_normalize_region(value: &str) -> Option<String> {
        let value = value.trim();
        if value.len() == 2 && value.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return Some(value.to_ascii_uppercase());
        }
        if value.len() == 3 && value.chars().all(|ch| ch.is_ascii_digit()) {
            return Some(value.to_string());
        }
        None
    }

    pub(crate) fn intl_locale_normalize_unicode_type(value: &str) -> Option<String> {
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

    pub(crate) fn intl_locale_options_from_value(
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

    pub(crate) fn intl_locale_data_from_canonical_tag(canonical: &str) -> IntlLocaleData {
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

    pub(crate) fn intl_locale_data_base_name(data: &IntlLocaleData) -> String {
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

    pub(crate) fn intl_locale_data_to_string(data: &IntlLocaleData) -> String {
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

    pub(crate) fn intl_locale_data_to_internal_value(data: &IntlLocaleData) -> Value {
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

    pub(crate) fn intl_locale_data_from_internal_value(value: &Value) -> Option<IntlLocaleData> {
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

    pub(crate) fn intl_locale_data_from_input_value(
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
}
