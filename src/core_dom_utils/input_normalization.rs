use super::*;

pub(crate) fn normalize_file_input_name(name: &str) -> String {
    let trimmed = name.trim();
    let basename = trimmed
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(trimmed)
        .to_string();
    if basename.is_empty() {
        "unnamed".to_string()
    } else {
        basename
    }
}

pub(crate) fn normalize_mock_file(file: &MockFile) -> MockFile {
    let mut normalized_bytes = file.bytes.clone();
    if normalized_bytes.is_empty() && file.size > 0 {
        if let Ok(target_len) = usize::try_from(file.size) {
            normalized_bytes.resize(target_len, 0);
        }
    }
    let normalized_size = normalized_bytes.len() as i64;
    MockFile {
        name: normalize_file_input_name(&file.name),
        size: normalized_size,
        mime_type: file.mime_type.clone(),
        last_modified: file.last_modified,
        webkit_relative_path: file.webkit_relative_path.clone(),
        bytes: normalized_bytes,
    }
}

pub(crate) fn file_input_value_from_files(files: &[MockFile]) -> String {
    let Some(first) = files.first() else {
        return String::new();
    };
    format!("{FILE_INPUT_FAKEPATH_PREFIX}{}", first.name)
}

fn normalize_hex_color(value: &str) -> Option<String> {
    if !value.starts_with('#') {
        return None;
    }

    let hex = &value[1..];
    let len = hex.len();
    if !matches!(len, 3 | 4 | 6 | 8) {
        return None;
    }
    if !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }

    let mut out = String::from("#");
    if matches!(len, 3 | 4) {
        for ch in hex.chars() {
            let ch = ch.to_ascii_lowercase();
            out.push(ch);
            out.push(ch);
        }
    } else {
        out.push_str(&hex.to_ascii_lowercase());
    }
    Some(out)
}

fn is_css_color_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }

    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
}

fn is_css_color_function_name(name: &str) -> bool {
    matches!(
        name,
        "rgb"
            | "rgba"
            | "hsl"
            | "hsla"
            | "hwb"
            | "lab"
            | "lch"
            | "oklab"
            | "oklch"
            | "color"
            | "color-mix"
            | "device-cmyk"
    )
}

fn is_css_color_function(value: &str) -> bool {
    let Some(open_index) = value.find('(') else {
        return false;
    };
    if open_index == 0 || !value.ends_with(')') {
        return false;
    }

    let name = value[..open_index].trim().to_ascii_lowercase();
    if !is_css_color_function_name(name.as_str()) {
        return false;
    }

    let args = &value[open_index + 1..value.len() - 1];
    if args.trim().is_empty() {
        return false;
    }

    let mut nested_depth = 0usize;
    for ch in args.chars() {
        match ch {
            '(' => nested_depth += 1,
            ')' => {
                if nested_depth == 0 {
                    return false;
                }
                nested_depth -= 1;
            }
            _ => {}
        }
    }

    nested_depth == 0
}

pub(crate) fn normalize_color_input_value(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return DEFAULT_COLOR_INPUT_VALUE.to_string();
    }

    if let Some(normalized_hex) = normalize_hex_color(trimmed) {
        return normalized_hex;
    }

    if is_css_color_function(trimmed) || is_css_color_identifier(trimmed) {
        return trimmed.to_string();
    }

    DEFAULT_COLOR_INPUT_VALUE.to_string()
}

fn parse_fixed_digits_u32(src: &str, start: usize, width: usize) -> Option<u32> {
    let end = start.checked_add(width)?;
    let part = src.get(start..end)?;
    if !part.as_bytes().iter().all(|b| b.is_ascii_digit()) {
        return None;
    }
    part.parse::<u32>().ok()
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

pub(crate) fn parse_date_input_components(value: &str) -> Option<(i64, u32, u32)> {
    let value = value.trim();
    if value.len() != 10 {
        return None;
    }

    let bytes = value.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }

    let year = parse_fixed_digits_u32(value, 0, 4)? as i64;
    let month = parse_fixed_digits_u32(value, 5, 2)?;
    let day = parse_fixed_digits_u32(value, 8, 2)?;

    if !(1..=12).contains(&month) {
        return None;
    }
    if day == 0 || day > days_in_month(year, month) {
        return None;
    }

    Some((year, month, day))
}

pub(crate) fn normalize_date_input_value(value: &str) -> String {
    let Some((year, month, day)) = parse_date_input_components(value) else {
        return String::new();
    };
    format!("{year:04}-{month:02}-{day:02}")
}

pub(crate) fn parse_datetime_local_input_components(
    value: &str,
) -> Option<(i64, u32, u32, u32, u32, u32, u32)> {
    let value = value.trim();
    if value.len() < 16 {
        return None;
    }

    let bytes = value.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' || bytes[10] != b'T' || bytes[13] != b':' {
        return None;
    }

    let year = parse_fixed_digits_u32(value, 0, 4)? as i64;
    let month = parse_fixed_digits_u32(value, 5, 2)?;
    let day = parse_fixed_digits_u32(value, 8, 2)?;
    let hour = parse_fixed_digits_u32(value, 11, 2)?;
    let minute = parse_fixed_digits_u32(value, 14, 2)?;
    let mut second = 0;
    let mut millisecond = 0;

    match value.len() {
        16 => {}
        19 => {
            if bytes.get(16) != Some(&b':') {
                return None;
            }
            second = parse_fixed_digits_u32(value, 17, 2)?;
        }
        21..=23 => {
            if bytes.get(16) != Some(&b':') || bytes.get(19) != Some(&b'.') {
                return None;
            }
            second = parse_fixed_digits_u32(value, 17, 2)?;
            let fraction = value.get(20..)?;
            if fraction.is_empty()
                || fraction.len() > 3
                || !fraction.as_bytes().iter().all(|b| b.is_ascii_digit())
            {
                return None;
            }
            millisecond = fraction.parse::<u32>().ok()?;
            for _ in 0..(3 - fraction.len()) {
                millisecond *= 10;
            }
        }
        _ => return None,
    }

    if !(1..=12).contains(&month) {
        return None;
    }
    if day == 0 || day > days_in_month(year, month) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }

    Some((year, month, day, hour, minute, second, millisecond))
}

pub(crate) fn normalize_datetime_local_input_value(value: &str) -> String {
    let Some((year, month, day, hour, minute, second, millisecond)) =
        parse_datetime_local_input_components(value)
    else {
        return String::new();
    };
    let seconds = if second == 0 && millisecond == 0 {
        String::new()
    } else if millisecond == 0 {
        format!(":{second:02}")
    } else {
        let mut fraction = format!("{millisecond:03}");
        while fraction.ends_with('0') {
            fraction.pop();
        }
        format!(":{second:02}.{fraction}")
    };
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}{seconds}")
}

pub(crate) fn parse_time_input_components(value: &str) -> Option<(u32, u32, u32, u32)> {
    let value = value.trim();
    let has_seconds = match value.len() {
        5 => false,
        8 | 10..=12 => true,
        _ => return None,
    };

    let bytes = value.as_bytes();
    if bytes[2] != b':' {
        return None;
    }
    if has_seconds && bytes.get(5) != Some(&b':') {
        return None;
    }

    let hour = parse_fixed_digits_u32(value, 0, 2)?;
    let minute = parse_fixed_digits_u32(value, 3, 2)?;
    let mut second = 0;
    let mut millisecond = 0;

    match value.len() {
        5 => {}
        8 => {
            second = parse_fixed_digits_u32(value, 6, 2)?;
        }
        10..=12 => {
            if bytes.get(8) != Some(&b'.') {
                return None;
            }
            second = parse_fixed_digits_u32(value, 6, 2)?;
            let fraction = value.get(9..)?;
            if fraction.is_empty()
                || fraction.len() > 3
                || !fraction.as_bytes().iter().all(|b| b.is_ascii_digit())
            {
                return None;
            }
            millisecond = fraction.parse::<u32>().ok()?;
            for _ in 0..(3 - fraction.len()) {
                millisecond *= 10;
            }
        }
        _ => return None,
    }

    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }

    Some((hour, minute, second, millisecond))
}

pub(crate) fn normalize_time_input_value(value: &str) -> String {
    let Some((hour, minute, second, millisecond)) = parse_time_input_components(value) else {
        return String::new();
    };
    if second == 0 && millisecond == 0 {
        format!("{hour:02}:{minute:02}")
    } else if millisecond == 0 {
        format!("{hour:02}:{minute:02}:{second:02}")
    } else {
        let mut fraction = format!("{millisecond:03}");
        while fraction.ends_with('0') {
            fraction.pop();
        }
        format!("{hour:02}:{minute:02}:{second:02}.{fraction}")
    }
}

pub(crate) fn normalize_file_input_value(_value: &str) -> String {
    String::new()
}

pub(crate) fn normalize_number_input_value(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    match value.parse::<f64>() {
        Ok(parsed) if parsed.is_finite() => value.to_string(),
        _ => String::new(),
    }
}

fn parse_finite_decimal(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    value
        .parse::<f64>()
        .ok()
        .filter(|parsed| parsed.is_finite())
}

fn parse_optional_finite_decimal(value: Option<&str>) -> Option<f64> {
    value.and_then(parse_finite_decimal)
}

fn format_decimal_for_input(value: f64) -> String {
    let value = if value.abs() < 1e-12 { 0.0 } else { value };
    if value.fract().abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        let mut out = value.to_string();
        if out.contains('.') {
            while out.ends_with('0') {
                out.pop();
            }
            if out.ends_with('.') {
                out.pop();
            }
        }
        out
    }
}

fn snap_to_step(value: f64, base: f64, step: f64) -> f64 {
    if !value.is_finite() || !base.is_finite() || !step.is_finite() || step <= 0.0 {
        return value;
    }
    let ratio = (value - base) / step;
    if !ratio.is_finite() {
        return value;
    }
    let lower = ratio.floor();
    let upper = ratio.ceil();
    let lower_value = base + lower * step;
    let upper_value = base + upper * step;
    let lower_diff = (value - lower_value).abs();
    let upper_diff = (upper_value - value).abs();
    if (lower_diff - upper_diff).abs() <= 1e-9 {
        upper_value
    } else if lower_diff < upper_diff {
        lower_value
    } else {
        upper_value
    }
}

pub(crate) fn normalize_range_input_value(
    value: &str,
    min_attr: Option<&str>,
    max_attr: Option<&str>,
    step_attr: Option<&str>,
    value_attr: Option<&str>,
) -> String {
    let min = parse_optional_finite_decimal(min_attr).unwrap_or(DEFAULT_RANGE_INPUT_MIN);
    let max = parse_optional_finite_decimal(max_attr).unwrap_or(DEFAULT_RANGE_INPUT_MAX);
    let default_value = if max < min {
        min
    } else {
        min + (max - min) / 2.0
    };

    let parsed_value = parse_finite_decimal(value);
    let mut numeric = parsed_value.unwrap_or(default_value);
    if max < min {
        numeric = min;
    } else {
        numeric = numeric.clamp(min, max);
    }

    let step_is_any = step_attr
        .map(|raw| raw.trim().eq_ignore_ascii_case("any"))
        .unwrap_or(false);
    if !step_is_any && max >= min && parsed_value.is_some() {
        let step = parse_optional_finite_decimal(step_attr)
            .filter(|parsed| *parsed > 0.0)
            .unwrap_or(1.0);
        let base = parse_optional_finite_decimal(min_attr)
            .or_else(|| parse_optional_finite_decimal(value_attr))
            .unwrap_or(0.0);
        numeric = snap_to_step(numeric, base, step);
        numeric = numeric.clamp(min, max);
    }

    format_decimal_for_input(numeric)
}

pub(crate) fn normalize_password_input_value(value: &str) -> String {
    value
        .chars()
        .filter(|ch| *ch != '\n' && *ch != '\r')
        .collect()
}

pub(crate) fn normalize_image_input_value(_value: &str) -> String {
    String::new()
}
