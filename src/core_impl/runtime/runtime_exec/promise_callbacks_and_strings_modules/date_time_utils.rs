impl Harness {
    pub(crate) fn parse_date_string_to_epoch_ms(src: &str) -> Option<i64> {
        let src = src.trim();
        if src.is_empty() {
            return None;
        }

        let bytes = src.as_bytes();
        let mut i = 0usize;

        let mut sign = 1i64;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            if bytes[i] == b'-' {
                sign = -1;
            }
            i += 1;
        }

        let year_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i <= year_start || (i - year_start) < 4 {
            return None;
        }
        let year = sign * src.get(year_start..i)?.parse::<i64>().ok()?;

        if i >= bytes.len() || bytes[i] != b'-' {
            return None;
        }
        i += 1;
        let month = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
        if i >= bytes.len() || bytes[i] != b'-' {
            return None;
        }
        i += 1;
        let day = Self::parse_fixed_digits_i64(src, &mut i, 2)?;

        let month = u32::try_from(month).ok()?;
        if !(1..=12).contains(&month) {
            return None;
        }
        let day = u32::try_from(day).ok()?;
        if day == 0 || day > Self::days_in_month(year, month) {
            return None;
        }

        let mut hour = 0i64;
        let mut minute = 0i64;
        let mut second = 0i64;
        let mut millisecond = 0i64;
        let mut offset_minutes = 0i64;

        if i < bytes.len() {
            if bytes[i] != b'T' && bytes[i] != b' ' {
                return None;
            }
            i += 1;

            hour = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
            if i >= bytes.len() || bytes[i] != b':' {
                return None;
            }
            i += 1;
            minute = Self::parse_fixed_digits_i64(src, &mut i, 2)?;

            if i < bytes.len() && bytes[i] == b':' {
                i += 1;
                second = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
            }

            if i < bytes.len() && bytes[i] == b'.' {
                i += 1;
                let frac_start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i == frac_start {
                    return None;
                }

                let frac = src.get(frac_start..i)?;
                let mut parsed = 0i64;
                let mut digits = 0usize;
                for ch in frac.chars().take(3) {
                    parsed = parsed * 10 + i64::from(ch.to_digit(10)?);
                    digits += 1;
                }
                while digits < 3 {
                    parsed *= 10;
                    digits += 1;
                }
                millisecond = parsed;
            }

            if i < bytes.len() {
                match bytes[i] {
                    b'Z' | b'z' => {
                        i += 1;
                    }
                    b'+' | b'-' => {
                        let tz_sign = if bytes[i] == b'+' { 1 } else { -1 };
                        i += 1;
                        let tz_hour = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
                        let tz_minute = if i < bytes.len() && bytes[i] == b':' {
                            i += 1;
                            Self::parse_fixed_digits_i64(src, &mut i, 2)?
                        } else {
                            Self::parse_fixed_digits_i64(src, &mut i, 2)?
                        };
                        if tz_hour > 23 || tz_minute > 59 {
                            return None;
                        }
                        offset_minutes = tz_sign * (tz_hour * 60 + tz_minute);
                    }
                    _ => return None,
                }
            }
        }

        if i != bytes.len() {
            return None;
        }
        if hour > 23 || minute > 59 || second > 59 {
            return None;
        }

        let timestamp_ms = Self::utc_timestamp_ms_from_components(
            year,
            i64::from(month) - 1,
            i64::from(day),
            hour,
            minute,
            second,
            millisecond,
        );
        Some(timestamp_ms - offset_minutes * 60_000)
    }

    pub(crate) fn parse_fixed_digits_i64(src: &str, i: &mut usize, width: usize) -> Option<i64> {
        let end = i.checked_add(width)?;
        let segment = src.get(*i..end)?;
        if !segment.as_bytes().iter().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *i = end;
        segment.parse::<i64>().ok()
    }

    pub(crate) fn format_iso_8601_utc(timestamp_ms: i64) -> String {
        let (year, month, day, hour, minute, second, millisecond) =
            Self::date_components_utc(timestamp_ms);
        let year_str = if (0..=9999).contains(&year) {
            format!("{year:04}")
        } else if year < 0 {
            format!("-{:06}", -(year as i128))
        } else {
            format!("+{:06}", year)
        };
        format!(
            "{year_str}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millisecond:03}Z"
        )
    }

    pub(crate) fn date_components_utc(timestamp_ms: i64) -> (i64, u32, u32, u32, u32, u32, u32) {
        let days = timestamp_ms.div_euclid(86_400_000);
        let rem = timestamp_ms.rem_euclid(86_400_000);
        let hour = (rem / 3_600_000) as u32;
        let minute = ((rem % 3_600_000) / 60_000) as u32;
        let second = ((rem % 60_000) / 1_000) as u32;
        let millisecond = (rem % 1_000) as u32;
        let (year, month, day) = Self::civil_from_days(days);
        (year, month, day, hour, minute, second, millisecond)
    }

    pub(crate) fn utc_timestamp_ms_from_components(
        year: i64,
        month_zero_based: i64,
        day: i64,
        hour: i64,
        minute: i64,
        second: i64,
        millisecond: i64,
    ) -> i64 {
        let (norm_year, norm_month) = Self::normalize_year_month(year, month_zero_based);
        let mut days = Self::days_from_civil(norm_year, norm_month, 1) + (day - 1);
        let mut time_ms = ((hour * 60 + minute) * 60 + second) * 1_000 + millisecond;
        days += time_ms.div_euclid(86_400_000);
        time_ms = time_ms.rem_euclid(86_400_000);

        let out = (days as i128) * 86_400_000i128 + (time_ms as i128);
        out.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
    }

    pub(crate) fn normalize_year_month(year: i64, month_zero_based: i64) -> (i64, u32) {
        let total_month = year.saturating_mul(12).saturating_add(month_zero_based);
        let norm_year = total_month.div_euclid(12);
        let norm_month = total_month.rem_euclid(12) as u32 + 1;
        (norm_year, norm_month)
    }

    pub(crate) fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
        let adjusted_year = year - if month <= 2 { 1 } else { 0 };
        let era = adjusted_year.div_euclid(400);
        let yoe = adjusted_year - era * 400;
        let month = i64::from(month);
        let day = i64::from(day);
        let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    pub(crate) fn civil_from_days(days: i64) -> (i64, u32, u32) {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096).div_euclid(365);
        let mut year = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2).div_euclid(153);
        let day = (doy - (153 * mp + 2).div_euclid(5) + 1) as u32;
        let month = (mp + if mp < 10 { 3 } else { -9 }) as u32;
        if month <= 2 {
            year += 1;
        }
        (year, month, day)
    }

    pub(crate) fn days_in_month(year: i64, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    pub(crate) fn is_leap_year(year: i64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

}
