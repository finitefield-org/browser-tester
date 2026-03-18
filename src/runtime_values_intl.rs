#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntlFormatterKind {
    Collator,
    DateTimeFormat,
    DisplayNames,
    DurationFormat,
    ListFormat,
    NumberFormat,
    PluralRules,
    RelativeTimeFormat,
    Segmenter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntlStaticMethod {
    CollatorSupportedLocalesOf,
    DateTimeFormatSupportedLocalesOf,
    DisplayNamesSupportedLocalesOf,
    DurationFormatSupportedLocalesOf,
    ListFormatSupportedLocalesOf,
    NumberFormatSupportedLocalesOf,
    PluralRulesSupportedLocalesOf,
    RelativeTimeFormatSupportedLocalesOf,
    SegmenterSupportedLocalesOf,
    GetCanonicalLocales,
    SupportedValuesOf,
}

impl IntlFormatterKind {
    pub(crate) fn storage_name(self) -> &'static str {
        match self {
            Self::Collator => "Collator",
            Self::DateTimeFormat => "DateTimeFormat",
            Self::DisplayNames => "DisplayNames",
            Self::DurationFormat => "DurationFormat",
            Self::ListFormat => "ListFormat",
            Self::NumberFormat => "NumberFormat",
            Self::PluralRules => "PluralRules",
            Self::RelativeTimeFormat => "RelativeTimeFormat",
            Self::Segmenter => "Segmenter",
        }
    }

    pub(crate) fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "Collator" => Some(Self::Collator),
            "DateTimeFormat" => Some(Self::DateTimeFormat),
            "DisplayNames" => Some(Self::DisplayNames),
            "DurationFormat" => Some(Self::DurationFormat),
            "ListFormat" => Some(Self::ListFormat),
            "NumberFormat" => Some(Self::NumberFormat),
            "PluralRules" => Some(Self::PluralRules),
            "RelativeTimeFormat" => Some(Self::RelativeTimeFormat),
            "Segmenter" => Some(Self::Segmenter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IntlDateTimeOptions {
    pub(crate) calendar: String,
    pub(crate) numbering_system: String,
    pub(crate) time_zone: String,
    pub(crate) date_style: Option<String>,
    pub(crate) time_style: Option<String>,
    pub(crate) weekday: Option<String>,
    pub(crate) year: Option<String>,
    pub(crate) month: Option<String>,
    pub(crate) day: Option<String>,
    pub(crate) hour: Option<String>,
    pub(crate) minute: Option<String>,
    pub(crate) second: Option<String>,
    pub(crate) fractional_second_digits: Option<u8>,
    pub(crate) time_zone_name: Option<String>,
    pub(crate) hour12: Option<bool>,
    pub(crate) day_period: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IntlDateTimeComponents {
    pub(crate) year: i64,
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) hour: u32,
    pub(crate) minute: u32,
    pub(crate) second: u32,
    pub(crate) millisecond: u32,
    pub(crate) weekday: u32,
    pub(crate) offset_minutes: i64,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlPart {
    pub(crate) part_type: String,
    pub(crate) value: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlRelativeTimePart {
    pub(crate) part_type: String,
    pub(crate) value: String,
    pub(crate) unit: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlDisplayNamesOptions {
    pub(crate) style: String,
    pub(crate) display_type: String,
    pub(crate) fallback: String,
    pub(crate) language_display: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlDurationOptions {
    pub(crate) style: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlNumberFormatOptions {
    pub(crate) style: String,
    pub(crate) currency: Option<String>,
    pub(crate) unit: Option<String>,
    pub(crate) unit_display: String,
    pub(crate) numbering_system: String,
    pub(crate) minimum_fraction_digits: Option<usize>,
    pub(crate) maximum_fraction_digits: Option<usize>,
    pub(crate) maximum_significant_digits: Option<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlListOptions {
    pub(crate) style: String,
    pub(crate) list_type: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlPluralRulesOptions {
    pub(crate) rule_type: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlRelativeTimeOptions {
    pub(crate) style: String,
    pub(crate) numeric: String,
    pub(crate) numbering_system: String,
    pub(crate) locale_matcher: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlSegmenterOptions {
    pub(crate) granularity: String,
    pub(crate) locale_matcher: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlLocaleOptions {
    pub(crate) language: Option<String>,
    pub(crate) script: Option<String>,
    pub(crate) region: Option<String>,
    pub(crate) calendar: Option<String>,
    pub(crate) case_first: Option<String>,
    pub(crate) collation: Option<String>,
    pub(crate) hour_cycle: Option<String>,
    pub(crate) numbering_system: Option<String>,
    pub(crate) numeric: Option<bool>,
}

#[derive(Debug, Clone)]
pub(crate) struct IntlLocaleData {
    pub(crate) language: String,
    pub(crate) script: Option<String>,
    pub(crate) region: Option<String>,
    pub(crate) variants: Vec<String>,
    pub(crate) calendar: Option<String>,
    pub(crate) case_first: Option<String>,
    pub(crate) collation: Option<String>,
    pub(crate) hour_cycle: Option<String>,
    pub(crate) numbering_system: Option<String>,
    pub(crate) numeric: Option<bool>,
}
