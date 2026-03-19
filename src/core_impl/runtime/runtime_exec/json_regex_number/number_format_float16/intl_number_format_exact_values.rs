use super::super::*;

impl Harness {
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

    pub(crate) fn exact_numeric_string_from_value(value: &Value) -> Option<String> {
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

    pub(crate) fn format_exact_decimal_string_with_fraction_constraints(
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

    pub(crate) fn rendered_number_is_zero(rendered: &str) -> bool {
        rendered
            .chars()
            .all(|ch| matches!(ch, '0' | '.' | ',' | '-'))
    }
}
