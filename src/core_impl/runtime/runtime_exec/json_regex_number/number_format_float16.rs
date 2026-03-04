use super::*;

impl Harness {
    pub(crate) fn number_primitive_value(value: &Value) -> Option<f64> {
        match value {
            Value::Number(value) => Some(*value as f64),
            Value::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn number_value(value: f64) -> Value {
        if value == 0.0 && value.is_sign_negative() {
            return Value::Float(-0.0);
        }
        if value.is_finite()
            && value.fract() == 0.0
            && value >= i64::MIN as f64
            && value <= i64::MAX as f64
        {
            let integer = value as i64;
            if (integer as f64) == value {
                return Value::Number(integer);
            }
        }
        Value::Float(value)
    }

    pub(crate) fn coerce_number_for_number_constructor(value: &Value) -> f64 {
        match value {
            Value::Number(v) => *v as f64,
            Value::Float(v) => *v,
            Value::BigInt(v) => v.to_f64().unwrap_or_else(|| {
                if v.sign() == Sign::Minus {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                }
            }),
            Value::Bool(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            Value::String(v) => Self::parse_js_number_from_string(v),
            Value::Date(v) => *v.borrow() as f64,
            Value::Object(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::WeakMap(_)
            | Value::Set(_)
            | Value::WeakSet(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::WeakMapConstructor
            | Value::SetConstructor
            | Value::WeakSetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::Symbol(_)
            | Value::RegExp(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_) => f64::NAN,
            Value::Array(values) => {
                let rendered = Value::Array(values.clone()).as_string();
                Self::parse_js_number_from_string(&rendered)
            }
        }
    }

    pub(crate) fn parse_js_number_from_string(src: &str) -> f64 {
        let trimmed = src.trim();
        if trimmed.is_empty() {
            return 0.0;
        }
        if trimmed == "Infinity" || trimmed == "+Infinity" {
            return f64::INFINITY;
        }
        if trimmed == "-Infinity" {
            return f64::NEG_INFINITY;
        }

        if trimmed.starts_with('+') || trimmed.starts_with('-') {
            let rest = &trimmed[1..];
            if rest.starts_with("0x")
                || rest.starts_with("0X")
                || rest.starts_with("0o")
                || rest.starts_with("0O")
                || rest.starts_with("0b")
                || rest.starts_with("0B")
            {
                return f64::NAN;
            }
        }

        if let Some(digits) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            return Self::parse_prefixed_radix_to_f64(digits, 16);
        }
        if let Some(digits) = trimmed
            .strip_prefix("0o")
            .or_else(|| trimmed.strip_prefix("0O"))
        {
            return Self::parse_prefixed_radix_to_f64(digits, 8);
        }
        if let Some(digits) = trimmed
            .strip_prefix("0b")
            .or_else(|| trimmed.strip_prefix("0B"))
        {
            return Self::parse_prefixed_radix_to_f64(digits, 2);
        }

        trimmed.parse::<f64>().unwrap_or(f64::NAN)
    }

    pub(crate) fn parse_prefixed_radix_to_f64(src: &str, radix: u32) -> f64 {
        if src.is_empty() {
            return f64::NAN;
        }
        let mut out = 0.0f64;
        for ch in src.chars() {
            let Some(digit) = ch.to_digit(radix) else {
                return f64::NAN;
            };
            out = out * (radix as f64) + (digit as f64);
        }
        out
    }

    pub(crate) fn format_number_default(value: f64) -> String {
        if value.is_nan() {
            return "NaN".to_string();
        }
        if value == f64::INFINITY {
            return "Infinity".to_string();
        }
        if value == f64::NEG_INFINITY {
            return "-Infinity".to_string();
        }
        if value == 0.0 {
            return "0".to_string();
        }

        if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            let integer = value as i64;
            if (integer as f64) == value {
                return integer.to_string();
            }
        }

        let out = format!("{value}");
        Self::normalize_exponential_string(out, false)
    }

    pub(crate) fn number_to_exponential(value: f64, fraction_digits: Option<usize>) -> String {
        if !value.is_finite() {
            return Self::format_number_default(value);
        }

        let out = if let Some(fraction_digits) = fraction_digits {
            format!(
                "{:.*e}",
                fraction_digits,
                if value == 0.0 { 0.0 } else { value }
            )
        } else {
            format!("{:e}", if value == 0.0 { 0.0 } else { value })
        };
        Self::normalize_exponential_string(out, fraction_digits.is_none())
    }

    pub(crate) fn normalize_exponential_string(raw: String, trim_fraction_zeros: bool) -> String {
        let Some(exp_idx) = raw.find('e').or_else(|| raw.find('E')) else {
            return raw;
        };
        let mut mantissa = raw[..exp_idx].to_string();
        let exponent_src = &raw[exp_idx + 1..];

        if trim_fraction_zeros && mantissa.contains('.') {
            while mantissa.ends_with('0') {
                mantissa.pop();
            }
            if mantissa.ends_with('.') {
                mantissa.pop();
            }
        }

        let exponent = exponent_src.parse::<i32>().unwrap_or(0);
        format!("{mantissa}e{:+}", exponent)
    }

    pub(crate) fn number_to_fixed(value: f64, fraction_digits: usize) -> String {
        if !value.is_finite() {
            return Self::format_number_default(value);
        }
        format!(
            "{:.*}",
            fraction_digits,
            if value == 0.0 { 0.0 } else { value }
        )
    }

    pub(crate) fn number_to_precision(value: f64, precision: usize) -> String {
        if !value.is_finite() {
            return Self::format_number_default(value);
        }
        if value == 0.0 {
            if precision == 1 {
                return "0".to_string();
            }
            return format!("0.{}", "0".repeat(precision - 1));
        }

        let abs = value.abs();
        let exponent = abs.log10().floor() as i32;
        if exponent < -6 || exponent >= precision as i32 {
            return Self::number_to_exponential(value, Some(precision.saturating_sub(1)));
        }

        let fraction_digits = (precision as i32 - exponent - 1).max(0) as usize;
        format!(
            "{:.*}",
            fraction_digits,
            if value == 0.0 { 0.0 } else { value }
        )
    }

    pub(crate) fn number_to_string_radix(value: f64, radix: u32) -> String {
        if radix == 10 {
            return Self::format_number_default(value);
        }
        if !value.is_finite() {
            return Self::format_number_default(value);
        }
        if value == 0.0 {
            return "0".to_string();
        }

        let sign = if value < 0.0 { "-" } else { "" };
        let abs = value.abs();
        let int_part = abs.trunc();
        let mut int_digits = Vec::new();
        let mut n = int_part;
        let radix_f64 = radix as f64;

        while n >= 1.0 {
            let digit = (n % radix_f64).floor() as u32;
            int_digits.push(Self::radix_digit_char(digit));
            n = (n / radix_f64).floor();
        }
        if int_digits.is_empty() {
            int_digits.push('0');
        }
        int_digits.reverse();
        let int_str: String = int_digits.into_iter().collect();

        let mut frac = abs - int_part;
        if frac == 0.0 {
            return format!("{sign}{int_str}");
        }

        let mut frac_str = String::new();
        let mut digits = 0usize;
        while frac > 0.0 && digits < 16 {
            frac *= radix_f64;
            let digit = frac.floor() as u32;
            frac_str.push(Self::radix_digit_char(digit));
            frac -= digit as f64;
            digits += 1;
            if frac.abs() < f64::EPSILON {
                break;
            }
        }
        while frac_str.ends_with('0') {
            frac_str.pop();
        }

        if frac_str.is_empty() {
            format!("{sign}{int_str}")
        } else {
            format!("{sign}{int_str}.{frac_str}")
        }
    }

    pub(crate) fn resolve_number_to_locale_string_locale(
        &self,
        locale_arg: Option<&Value>,
    ) -> Result<String> {
        let requested = if let Some(locale_arg) = locale_arg {
            self.intl_collect_locales(locale_arg)?
        } else {
            Vec::new()
        };
        Ok(Self::intl_select_locale_for_formatter(
            IntlFormatterKind::NumberFormat,
            &requested,
        ))
    }

    pub(crate) fn parse_number_to_locale_string_fraction_digits(
        options_arg: Option<&Value>,
    ) -> Result<(Option<usize>, Option<usize>)> {
        let Some(options_arg) = options_arg else {
            return Ok((None, None));
        };
        match options_arg {
            Value::Undefined | Value::Null => Ok((None, None)),
            Value::Object(options) => {
                let options = options.borrow();
                let minimum = Self::parse_fraction_digits_option(
                    &options,
                    "minimumFractionDigits",
                    "minimumFractionDigits must be between 0 and 100",
                )?;
                let maximum = Self::parse_fraction_digits_option(
                    &options,
                    "maximumFractionDigits",
                    "maximumFractionDigits must be between 0 and 100",
                )?;
                if minimum.zip(maximum).is_some_and(|(min, max)| min > max) {
                    return Err(Error::ScriptRuntime(
                        "minimumFractionDigits cannot be greater than maximumFractionDigits".into(),
                    ));
                }
                Ok((minimum, maximum))
            }
            _ => Ok((None, None)),
        }
    }

    pub(crate) fn intl_number_format_options_from_value(
        &self,
        locale: &str,
        options_arg: Option<&Value>,
    ) -> Result<IntlNumberFormatOptions> {
        let mut options = IntlNumberFormatOptions {
            style: "decimal".to_string(),
            currency: None,
            unit: None,
            unit_display: "short".to_string(),
            numbering_system: Self::intl_locale_unicode_extension_value(locale, "nu")
                .unwrap_or_else(|| Self::intl_default_numbering_system_for_locale(locale)),
            minimum_fraction_digits: None,
            maximum_fraction_digits: None,
            maximum_significant_digits: None,
        };

        let Some(options_arg) = options_arg else {
            return Ok(options);
        };

        match options_arg {
            Value::Undefined | Value::Null => Ok(options),
            Value::Object(raw_options) => {
                let raw_options = raw_options.borrow();
                let string_option = |key: &str| -> Option<String> {
                    match Self::object_get_entry(&raw_options, key) {
                        Some(Value::Undefined) | None => None,
                        Some(value) => Some(value.as_string()),
                    }
                };

                if let Some(style) = string_option("style") {
                    if !matches!(style.as_str(), "decimal" | "currency" | "unit") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.NumberFormat style option".into(),
                        ));
                    }
                    options.style = style;
                }

                if let Some(currency) = string_option("currency") {
                    let normalized = currency.trim().to_ascii_uppercase();
                    if normalized.len() != 3
                        || !normalized.chars().all(|ch| ch.is_ascii_alphabetic())
                    {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.NumberFormat currency option".into(),
                        ));
                    }
                    options.currency = Some(normalized);
                }

                if let Some(unit) = string_option("unit") {
                    options.unit = Some(unit);
                }

                if let Some(unit_display) = string_option("unitDisplay") {
                    if !matches!(unit_display.as_str(), "short" | "long" | "narrow") {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.NumberFormat unitDisplay option".into(),
                        ));
                    }
                    options.unit_display = unit_display;
                }

                if let Some(numbering_system) = string_option("numberingSystem") {
                    if numbering_system.trim().is_empty() {
                        return Err(Error::ScriptRuntime(
                            "RangeError: invalid Intl.NumberFormat numberingSystem option".into(),
                        ));
                    }
                    options.numbering_system = numbering_system.to_ascii_lowercase();
                }

                let (minimum_fraction_digits, maximum_fraction_digits) =
                    Self::parse_number_to_locale_string_fraction_digits(Some(options_arg))?;
                options.minimum_fraction_digits = minimum_fraction_digits;
                options.maximum_fraction_digits = maximum_fraction_digits;

                options.maximum_significant_digits = Self::parse_significant_digits_option(
                    &raw_options,
                    "maximumSignificantDigits",
                    "maximumSignificantDigits must be between 1 and 21",
                )?;

                if options.style == "currency" {
                    if options.currency.is_none() {
                        return Err(Error::ScriptRuntime(
                            "TypeError: Intl.NumberFormat currency style requires a currency option"
                                .into(),
                        ));
                    }
                    let currency_digits = Self::intl_currency_default_fraction_digits(
                        options.currency.as_deref().unwrap_or("USD"),
                    );
                    if options.minimum_fraction_digits.is_none() {
                        options.minimum_fraction_digits = Some(currency_digits);
                    }
                    if options.maximum_fraction_digits.is_none() {
                        options.maximum_fraction_digits = Some(currency_digits);
                    }
                }

                if options.style == "unit" && options.unit.is_none() {
                    return Err(Error::ScriptRuntime(
                        "TypeError: Intl.NumberFormat unit style requires a unit option".into(),
                    ));
                }

                Ok(options)
            }
            _ => Err(Error::ScriptRuntime(
                "TypeError: Intl.NumberFormat options must be an object".into(),
            )),
        }
    }

    fn parse_fraction_digits_option(
        options: &ObjectValue,
        key: &str,
        out_of_range_message: &str,
    ) -> Result<Option<usize>> {
        let Some(value) = Self::object_get_entry(options, key) else {
            return Ok(None);
        };
        if matches!(value, Value::Undefined) {
            return Ok(None);
        }
        let digits = Self::value_to_i64(&value);
        if !(0..=100).contains(&digits) {
            return Err(Error::ScriptRuntime(out_of_range_message.into()));
        }
        Ok(Some(digits as usize))
    }

    fn parse_significant_digits_option(
        options: &ObjectValue,
        key: &str,
        out_of_range_message: &str,
    ) -> Result<Option<usize>> {
        let Some(value) = Self::object_get_entry(options, key) else {
            return Ok(None);
        };
        if matches!(value, Value::Undefined) {
            return Ok(None);
        }
        let digits = Self::value_to_i64(&value);
        if !(1..=21).contains(&digits) {
            return Err(Error::ScriptRuntime(out_of_range_message.into()));
        }
        Ok(Some(digits as usize))
    }

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

    fn intl_currency_default_fraction_digits(currency: &str) -> usize {
        match currency {
            "JPY" => 0,
            _ => 2,
        }
    }

    fn intl_currency_symbol(currency: &str) -> String {
        match currency {
            "EUR" => "€".to_string(),
            "JPY" => "￥".to_string(),
            "USD" => "$".to_string(),
            _ => currency.to_string(),
        }
    }

    fn intl_unit_label(locale: &str, unit: &str, unit_display: &str, value: f64) -> Result<String> {
        let singular = value.abs() == 1.0;
        let family = Self::intl_locale_family(locale);
        match unit {
            "kilometer-per-hour" => Ok(match unit_display {
                "long" => "kilometers per hour".to_string(),
                "short" | "narrow" => "km/h".to_string(),
                _ => {
                    return Err(Error::ScriptRuntime(
                        "RangeError: invalid Intl.NumberFormat unitDisplay option".into(),
                    ));
                }
            }),
            "liter" => Ok(match unit_display {
                "long" => {
                    if family == "en" && Self::intl_locale_region(locale) == Some("GB") {
                        if singular {
                            "litre".to_string()
                        } else {
                            "litres".to_string()
                        }
                    } else if singular {
                        "liter".to_string()
                    } else {
                        "liters".to_string()
                    }
                }
                "short" | "narrow" => "L".to_string(),
                _ => {
                    return Err(Error::ScriptRuntime(
                        "RangeError: invalid Intl.NumberFormat unitDisplay option".into(),
                    ));
                }
            }),
            _ => Err(Error::ScriptRuntime(
                "RangeError: invalid Intl.NumberFormat unit option".into(),
            )),
        }
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

    fn parse_prefixed_radix_to_bigint(src: &str, radix: u32) -> Option<JsBigInt> {
        if src.is_empty() {
            return None;
        }
        JsBigInt::parse_bytes(src.as_bytes(), radix)
    }

    fn parse_decimal_exponent_to_exact_decimal_string(src: &str) -> Option<String> {
        let bytes = src.as_bytes();
        let mut index = 0usize;

        let integer_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        let integer_digits = &src[integer_start..index];

        let mut fraction_digits = "";
        if index < bytes.len() && bytes[index] == b'.' {
            index += 1;
            let fraction_start = index;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            fraction_digits = &src[fraction_start..index];
        }

        if integer_digits.is_empty() && fraction_digits.is_empty() {
            return None;
        }

        let mut exponent = 0i64;
        if index < bytes.len() && (bytes[index] == b'e' || bytes[index] == b'E') {
            index += 1;
            if index >= bytes.len() {
                return None;
            }
            let mut exponent_negative = false;
            if bytes[index] == b'+' || bytes[index] == b'-' {
                exponent_negative = bytes[index] == b'-';
                index += 1;
            }
            if index >= bytes.len() || !bytes[index].is_ascii_digit() {
                return None;
            }
            let mut parsed = 0i64;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                parsed = parsed
                    .saturating_mul(10)
                    .saturating_add((bytes[index] - b'0') as i64);
                if parsed > 1_000_000 {
                    return None;
                }
                index += 1;
            }
            exponent = if exponent_negative { -parsed } else { parsed };
        }

        if index != bytes.len() {
            return None;
        }

        let mut digits = String::with_capacity(integer_digits.len() + fraction_digits.len());
        digits.push_str(integer_digits);
        digits.push_str(fraction_digits);
        let digits = digits.trim_start_matches('0').to_string();
        if digits.is_empty() {
            return Some("0".to_string());
        }

        let scale = exponent - fraction_digits.len() as i64;
        let mut rendered = if scale >= 0 {
            let zero_count = scale as usize;
            if zero_count > 1_000_000 {
                return None;
            }
            let mut out = digits;
            out.push_str(&"0".repeat(zero_count));
            out
        } else {
            let decimal_index = digits.len() as i64 + scale;
            if decimal_index > 0 {
                let decimal_index = decimal_index as usize;
                let mut out = String::with_capacity(digits.len() + 1);
                out.push_str(&digits[..decimal_index]);
                out.push('.');
                out.push_str(&digits[decimal_index..]);
                out
            } else {
                let leading_zeros = (-decimal_index) as usize;
                if leading_zeros > 1_000_000 {
                    return None;
                }
                let mut out = String::with_capacity(2 + leading_zeros + digits.len());
                out.push_str("0.");
                out.push_str(&"0".repeat(leading_zeros));
                out.push_str(&digits);
                out
            }
        };

        if let Some(dot) = rendered.find('.') {
            let mut end = rendered.len();
            while end > dot + 1 && rendered.as_bytes()[end - 1] == b'0' {
                end -= 1;
            }
            if end == dot + 1 {
                end -= 1;
            }
            rendered.truncate(end);
        }

        if rendered.is_empty() {
            Some("0".to_string())
        } else {
            Some(rendered)
        }
    }

    fn exact_numeric_string_from_value(value: &Value) -> Option<String> {
        match value {
            Value::BigInt(number) => Some(number.to_string()),
            Value::String(raw) => {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    return Some("0".to_string());
                }

                let mut body = trimmed;
                let mut negative = false;
                let mut had_sign = false;
                if let Some(stripped) = body.strip_prefix('+') {
                    body = stripped;
                    had_sign = true;
                } else if let Some(stripped) = body.strip_prefix('-') {
                    body = stripped;
                    negative = true;
                    had_sign = true;
                }

                if body.is_empty() {
                    return None;
                }

                if let Some(digits) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
                    if had_sign {
                        return None;
                    }
                    return Self::parse_prefixed_radix_to_bigint(digits, 16)
                        .map(|value| value.to_string());
                }
                if let Some(digits) = body.strip_prefix("0o").or_else(|| body.strip_prefix("0O")) {
                    if had_sign {
                        return None;
                    }
                    return Self::parse_prefixed_radix_to_bigint(digits, 8)
                        .map(|value| value.to_string());
                }
                if let Some(digits) = body.strip_prefix("0b").or_else(|| body.strip_prefix("0B")) {
                    if had_sign {
                        return None;
                    }
                    return Self::parse_prefixed_radix_to_bigint(digits, 2)
                        .map(|value| value.to_string());
                }

                let mut rendered = Self::parse_decimal_exponent_to_exact_decimal_string(body)?;
                if negative && rendered != "0" {
                    rendered = format!("-{rendered}");
                }
                Some(rendered)
            }
            _ => None,
        }
    }

    fn format_exact_decimal_string_with_fraction_constraints(
        rendered: &str,
        minimum_fraction_digits: Option<usize>,
        maximum_fraction_digits: Option<usize>,
    ) -> Option<String> {
        if rendered.contains('e') || rendered.contains('E') {
            return None;
        }

        let negative = rendered.starts_with('-');
        let unsigned = if negative { &rendered[1..] } else { rendered };

        let mut split = unsigned.splitn(2, '.');
        let mut integer = split.next().unwrap_or_default().to_string();
        let mut fraction = split.next().unwrap_or_default().to_string();
        if integer.is_empty() {
            integer.push('0');
        }

        if let Some(maximum_fraction_digits) = maximum_fraction_digits {
            if fraction.len() > maximum_fraction_digits {
                let should_round_up = fraction
                    .as_bytes()
                    .get(maximum_fraction_digits)
                    .is_some_and(|digit| *digit >= b'5');
                fraction.truncate(maximum_fraction_digits);

                if should_round_up {
                    let mut combined = String::with_capacity(integer.len() + fraction.len());
                    combined.push_str(&integer);
                    combined.push_str(&fraction);
                    if combined.is_empty() {
                        combined.push('0');
                    }
                    let mut magnitude = JsBigInt::parse_bytes(combined.as_bytes(), 10)?;
                    magnitude += 1;
                    let combined_rounded = magnitude.to_string();
                    if maximum_fraction_digits == 0 {
                        integer = combined_rounded;
                        fraction.clear();
                    } else if combined_rounded.len() <= maximum_fraction_digits {
                        integer = "0".to_string();
                        fraction = "0".repeat(maximum_fraction_digits - combined_rounded.len())
                            + &combined_rounded;
                    } else {
                        let split_index = combined_rounded.len() - maximum_fraction_digits;
                        integer = combined_rounded[..split_index].to_string();
                        fraction = combined_rounded[split_index..].to_string();
                    }
                }
            }

            let minimum_kept = minimum_fraction_digits
                .unwrap_or(0)
                .min(maximum_fraction_digits);
            while fraction.len() > minimum_kept && fraction.ends_with('0') {
                fraction.pop();
            }
        }

        if let Some(minimum_fraction_digits) = minimum_fraction_digits {
            if fraction.len() < minimum_fraction_digits {
                fraction.push_str(&"0".repeat(minimum_fraction_digits - fraction.len()));
            }
        }

        let mut out = if fraction.is_empty() {
            integer
        } else {
            format!("{integer}.{fraction}")
        };
        if negative && !Self::rendered_number_is_zero(&out) {
            out.insert(0, '-');
        }
        Some(out)
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

    fn rendered_number_is_zero(rendered: &str) -> bool {
        rendered
            .chars()
            .all(|ch| matches!(ch, '0' | '.' | ',' | '-'))
    }

    pub(crate) fn radix_digit_char(value: u32) -> char {
        if value < 10 {
            char::from(b'0' + value as u8)
        } else {
            char::from(b'a' + (value - 10) as u8)
        }
    }

    pub(crate) fn sum_precise(values: &[Value]) -> f64 {
        let mut sum = 0.0f64;
        let mut compensation = 0.0f64;
        for value in values {
            let value = Self::coerce_number_for_global(value);
            let adjusted = value - compensation;
            let next = sum + adjusted;
            compensation = (next - sum) - adjusted;
            sum = next;
        }
        sum
    }

    pub(crate) fn js_math_round(value: f64) -> f64 {
        if !value.is_finite() || value == 0.0 {
            return value;
        }
        if (-0.5..0.0).contains(&value) {
            return -0.0;
        }
        let floor = value.floor();
        let frac = value - floor;
        if frac < 0.5 { floor } else { floor + 1.0 }
    }

    pub(crate) fn js_math_sign(value: f64) -> f64 {
        if value.is_nan() {
            f64::NAN
        } else if value == 0.0 {
            value
        } else if value > 0.0 {
            1.0
        } else {
            -1.0
        }
    }

    pub(crate) fn to_i32_for_math(value: &Value) -> i32 {
        let numeric = Self::coerce_number_for_global(value);
        if !numeric.is_finite() {
            return 0;
        }
        let unsigned = numeric.trunc().rem_euclid(4_294_967_296.0);
        if unsigned >= 2_147_483_648.0 {
            (unsigned - 4_294_967_296.0) as i32
        } else {
            unsigned as i32
        }
    }

    pub(crate) fn to_u32_for_math(value: &Value) -> u32 {
        let numeric = Self::coerce_number_for_global(value);
        if !numeric.is_finite() {
            return 0;
        }
        numeric.trunc().rem_euclid(4_294_967_296.0) as u32
    }

    pub(crate) fn math_f16round(value: f64) -> f64 {
        let half = Self::f32_to_f16_bits(value as f32);
        Self::f16_bits_to_f32(half) as f64
    }

    pub(crate) fn f32_to_f16_bits(value: f32) -> u16 {
        let bits = value.to_bits();
        let sign = ((bits >> 16) & 0x8000) as u16;
        let exp = ((bits >> 23) & 0xff) as i32;
        let mant = bits & 0x007f_ffff;

        if exp == 0xff {
            if mant == 0 {
                return sign | 0x7c00;
            }
            return sign | 0x7e00;
        }

        let exp16 = exp - 127 + 15;
        if exp16 >= 0x1f {
            return sign | 0x7c00;
        }

        if exp16 <= 0 {
            if exp16 < -10 {
                return sign;
            }
            let mantissa = mant | 0x0080_0000;
            let shift = (14 - exp16) as u32;
            let mut half_mant = mantissa >> shift;
            let round_bit = 1u32 << (shift - 1);
            if (mantissa & round_bit) != 0
                && ((mantissa & (round_bit - 1)) != 0 || (half_mant & 1) != 0)
            {
                half_mant += 1;
            }
            return sign | (half_mant as u16);
        }

        let mut half_exp = (exp16 as u16) << 10;
        let mut half_mant = (mant >> 13) as u16;
        let round_bits = mant & 0x1fff;
        if round_bits > 0x1000 || (round_bits == 0x1000 && (half_mant & 1) != 0) {
            half_mant = half_mant.wrapping_add(1);
            if half_mant == 0x0400 {
                half_mant = 0;
                half_exp = half_exp.wrapping_add(0x0400);
                if half_exp >= 0x7c00 {
                    return sign | 0x7c00;
                }
            }
        }
        sign | half_exp | half_mant
    }

    pub(crate) fn f16_bits_to_f32(bits: u16) -> f32 {
        let sign = ((bits & 0x8000) as u32) << 16;
        let exp = ((bits >> 10) & 0x1f) as u32;
        let mant = (bits & 0x03ff) as u32;

        let out_bits = if exp == 0 {
            if mant == 0 {
                sign
            } else {
                let mut mantissa = mant;
                let mut exp_val = -14i32;
                while (mantissa & 0x0400) == 0 {
                    mantissa <<= 1;
                    exp_val -= 1;
                }
                mantissa &= 0x03ff;
                let exp32 = ((exp_val + 127) as u32) << 23;
                sign | exp32 | (mantissa << 13)
            }
        } else if exp == 0x1f {
            sign | 0x7f80_0000 | (mant << 13)
        } else {
            let exp32 = (((exp as i32) - 15 + 127) as u32) << 23;
            sign | exp32 | (mant << 13)
        };

        f32::from_bits(out_bits)
    }
}
