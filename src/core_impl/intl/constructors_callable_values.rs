use super::*;

impl Harness {
    pub(crate) fn intl_display_names_lookup(
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

    pub(crate) fn intl_display_names_of(
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

    pub(crate) fn intl_display_names_resolved_options_value(
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

    pub(crate) fn intl_supported_values_of(key: &str) -> Result<Vec<String>> {
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

    pub(crate) fn new_builtin_placeholder_function() -> Value {
        Value::Function(Rc::new(FunctionValue {
            handler: ScriptHandler {
                params: Vec::new(),
                stmts: Vec::new(),
            },
            captured_env: Rc::new(RefCell::new(ScriptEnv::default())),
            captured_pending_function_decls: Vec::new(),
            captured_global_names: HashSet::new(),
            local_bindings: HashSet::new(),
            prototype_object: Rc::new(RefCell::new(ObjectValue::default())),
            global_scope: true,
            is_async: false,
            is_generator: false,
            is_arrow: false,
            is_method: false,
            is_class_constructor: false,
            class_super_constructor: None,
            class_super_prototype: None,
        }))
    }

    pub(crate) fn intl_constructor_value(&self, constructor_name: &str) -> Value {
        let Some(Value::Object(entries)) = self.script_runtime.env.get("Intl") else {
            return Self::new_builtin_placeholder_function();
        };
        Self::object_get_entry(&entries.borrow(), constructor_name)
            .unwrap_or_else(Self::new_builtin_placeholder_function)
    }

    pub(crate) fn intl_collator_options_from_value(
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

    pub(crate) fn new_intl_collator_compare_callable(
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

    pub(crate) fn new_intl_collator_value(
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

    pub(crate) fn new_intl_date_time_format_callable(
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

    pub(crate) fn new_intl_date_time_formatter_value(
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

    pub(crate) fn new_intl_display_names_value(
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

    pub(crate) fn new_intl_duration_format_callable(
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

    pub(crate) fn new_intl_duration_formatter_value(
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

    pub(crate) fn new_intl_list_format_callable(
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
}
