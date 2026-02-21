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
            | Value::Set(_)
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
            | Value::SetConstructor
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

    pub(crate) fn format_number_to_locale_string(
        value: f64,
        locale: &str,
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

        Self::format_preformatted_number_for_locale(&rendered, locale)
    }

    fn format_preformatted_number_for_locale(rendered: &str, locale: &str) -> String {
        let negative = rendered.starts_with('-');
        let unsigned = if negative { &rendered[1..] } else { rendered };

        let family = Self::intl_locale_family(locale);
        let (group_sep, decimal_sep) = if family == "de" {
            ('.', ',')
        } else {
            (',', '.')
        };

        if unsigned.contains('e') || unsigned.contains('E') {
            let mut out = unsigned.to_string();
            if decimal_sep != '.' {
                out = out.replacen('.', &decimal_sep.to_string(), 1);
            }
            if negative && !Self::rendered_number_is_zero(&out) {
                return format!("-{out}");
            }
            return out;
        }

        let mut parts = unsigned.splitn(2, '.');
        let integer = parts.next().unwrap_or_default();
        let fraction = parts.next();

        let mut grouped = String::new();
        for (index, ch) in integer.chars().rev().enumerate() {
            if index > 0 && index % 3 == 0 {
                grouped.push(group_sep);
            }
            grouped.push(ch);
        }
        let mut out = grouped.chars().rev().collect::<String>();
        if out.is_empty() {
            out.push('0');
        }

        if let Some(fraction) = fraction {
            if !fraction.is_empty() {
                out.push(decimal_sep);
                out.push_str(fraction);
            }
        }

        if negative && !Self::rendered_number_is_zero(&out) {
            format!("-{out}")
        } else {
            out
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
