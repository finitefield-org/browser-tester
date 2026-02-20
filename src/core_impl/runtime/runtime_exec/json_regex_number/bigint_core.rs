use super::*;

impl Harness {
    pub(crate) fn bigint_as_uint_n(bits: usize, value: &JsBigInt) -> JsBigInt {
        if bits == 0 {
            return JsBigInt::zero();
        }
        let modulo = JsBigInt::one() << bits;
        let mut out = value % &modulo;
        if out.sign() == Sign::Minus {
            out += &modulo;
        }
        out
    }

    pub(crate) fn bigint_as_int_n(bits: usize, value: &JsBigInt) -> JsBigInt {
        if bits == 0 {
            return JsBigInt::zero();
        }
        let modulo = JsBigInt::one() << bits;
        let threshold = JsBigInt::one() << (bits - 1);
        let unsigned = Self::bigint_as_uint_n(bits, value);
        if unsigned >= threshold {
            unsigned - modulo
        } else {
            unsigned
        }
    }

    pub(crate) fn coerce_bigint_for_constructor(value: &Value) -> Result<JsBigInt> {
        match value {
            Value::BigInt(value) => Ok(value.clone()),
            Value::Bool(value) => Ok(if *value {
                JsBigInt::one()
            } else {
                JsBigInt::zero()
            }),
            Value::Number(value) => Ok(JsBigInt::from(*value)),
            Value::Float(value) => {
                if value.is_finite() && value.fract() == 0.0 {
                    Ok(JsBigInt::from(*value as i64))
                } else {
                    Err(Error::ScriptRuntime(
                        "cannot convert Number value to BigInt".into(),
                    ))
                }
            }
            Value::String(value) => Self::parse_js_bigint_from_string(value),
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(
                "cannot convert null or undefined to BigInt".into(),
            )),
            Value::Date(value) => Ok(JsBigInt::from(*value.borrow())),
            Value::Array(values) => {
                let rendered = Value::Array(values.clone()).as_string();
                Self::parse_js_bigint_from_string(&rendered)
            }
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
            | Value::Function(_) => Err(Error::ScriptRuntime(
                "cannot convert object value to BigInt".into(),
            )),
        }
    }

    pub(crate) fn coerce_bigint_for_builtin_op(value: &Value) -> Result<JsBigInt> {
        match value {
            Value::BigInt(value) => Ok(value.clone()),
            Value::Bool(value) => Ok(if *value {
                JsBigInt::one()
            } else {
                JsBigInt::zero()
            }),
            Value::String(value) => Self::parse_js_bigint_from_string(value),
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(
                "cannot convert null or undefined to BigInt".into(),
            )),
            Value::Number(_)
            | Value::Float(_)
            | Value::Date(_)
            | Value::Object(_)
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
            | Value::Function(_)
            | Value::Array(_) => Err(Error::ScriptRuntime(
                "cannot convert value to BigInt".into(),
            )),
        }
    }

    pub(crate) fn parse_js_bigint_from_string(src: &str) -> Result<JsBigInt> {
        let trimmed = src.trim();
        if trimmed.is_empty() {
            return Ok(JsBigInt::zero());
        }

        if let Some(rest) = trimmed.strip_prefix('+') {
            return Self::parse_signed_decimal_bigint(rest, false);
        }
        if let Some(rest) = trimmed.strip_prefix('-') {
            return Self::parse_signed_decimal_bigint(rest, true);
        }

        if let Some(rest) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            return Self::parse_prefixed_bigint(rest, 16, trimmed);
        }
        if let Some(rest) = trimmed
            .strip_prefix("0o")
            .or_else(|| trimmed.strip_prefix("0O"))
        {
            return Self::parse_prefixed_bigint(rest, 8, trimmed);
        }
        if let Some(rest) = trimmed
            .strip_prefix("0b")
            .or_else(|| trimmed.strip_prefix("0B"))
        {
            return Self::parse_prefixed_bigint(rest, 2, trimmed);
        }

        Self::parse_signed_decimal_bigint(trimmed, false)
    }

    pub(crate) fn parse_prefixed_bigint(src: &str, radix: u32, original: &str) -> Result<JsBigInt> {
        if src.is_empty() {
            return Err(Error::ScriptRuntime(format!(
                "cannot convert {} to a BigInt",
                original
            )));
        }
        JsBigInt::parse_bytes(src.as_bytes(), radix)
            .ok_or_else(|| Error::ScriptRuntime(format!("cannot convert {} to a BigInt", original)))
    }

    pub(crate) fn parse_signed_decimal_bigint(src: &str, negative: bool) -> Result<JsBigInt> {
        let original = format!("{}{}", if negative { "-" } else { "" }, src);
        if src.is_empty() || !src.as_bytes().iter().all(u8::is_ascii_digit) {
            return Err(Error::ScriptRuntime(format!(
                "cannot convert {} to a BigInt",
                original
            )));
        }
        let mut value = JsBigInt::parse_bytes(src.as_bytes(), 10).ok_or_else(|| {
            Error::ScriptRuntime(format!("cannot convert {} to a BigInt", original))
        })?;
        if negative {
            value = -value;
        }
        Ok(value)
    }

    pub(crate) fn bigint_shift_left(value: &JsBigInt, shift: &JsBigInt) -> Result<JsBigInt> {
        if shift.sign() == Sign::Minus {
            let magnitude = (-shift)
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value >> magnitude)
        } else {
            let magnitude = shift
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value << magnitude)
        }
    }

    pub(crate) fn bigint_shift_right(value: &JsBigInt, shift: &JsBigInt) -> Result<JsBigInt> {
        if shift.sign() == Sign::Minus {
            let magnitude = (-shift)
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value << magnitude)
        } else {
            let magnitude = shift
                .to_usize()
                .ok_or_else(|| Error::ScriptRuntime("BigInt shift count is too large".into()))?;
            Ok(value >> magnitude)
        }
    }
}
