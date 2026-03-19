use super::*;

impl Harness {
    pub(crate) fn parse_attr_f64(&self, node: NodeId, name: &str) -> Option<f64> {
        self.dom.attr(node, name).and_then(|raw| {
            let raw = raw.trim();
            if raw.is_empty() {
                None
            } else {
                raw.parse::<f64>().ok().filter(|value| value.is_finite())
            }
        })
    }

    pub(crate) fn parse_attr_i64(&self, node: NodeId, name: &str) -> Option<i64> {
        self.dom.attr(node, name).and_then(|raw| {
            let raw = raw.trim();
            if raw.is_empty() {
                None
            } else {
                raw.parse::<i64>().ok()
            }
        })
    }

    pub(crate) fn parse_number_value(raw: &str) -> Option<f64> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        raw.parse::<f64>().ok().filter(|value| value.is_finite())
    }

    pub(crate) fn parse_date_input_value_ms(raw: &str) -> Option<i64> {
        let (year, month, day) = parse_date_input_components(raw)?;
        Some(Self::utc_timestamp_ms_from_components(
            year,
            i64::from(month) - 1,
            i64::from(day),
            0,
            0,
            0,
            0,
        ))
    }

    pub(crate) fn format_date_input_from_timestamp_ms(timestamp_ms: i64) -> String {
        let (year, month, day, ..) = Self::date_components_utc(timestamp_ms);
        if !(0..=9999).contains(&year) {
            return String::new();
        }
        format!("{year:04}-{month:02}-{day:02}")
    }

    pub(crate) fn parse_datetime_local_input_value_ms(raw: &str) -> Option<i64> {
        let (year, month, day, hour, minute, second, millisecond) =
            parse_datetime_local_input_components(raw)?;
        Some(Self::utc_timestamp_ms_from_components(
            year,
            i64::from(month) - 1,
            i64::from(day),
            i64::from(hour),
            i64::from(minute),
            i64::from(second),
            i64::from(millisecond),
        ))
    }

    fn format_time_precision_suffix(second: u32, millisecond: u32) -> String {
        if second == 0 && millisecond == 0 {
            String::new()
        } else if millisecond == 0 {
            format!(":{second:02}")
        } else {
            let mut fraction = format!("{millisecond:03}");
            while fraction.ends_with('0') {
                fraction.pop();
            }
            format!(":{second:02}.{fraction}")
        }
    }

    pub(crate) fn format_datetime_local_input_from_timestamp_ms(timestamp_ms: i64) -> String {
        let (year, month, day, hour, minute, second, millisecond) =
            Self::date_components_utc(timestamp_ms);
        if !(0..=9999).contains(&year) {
            return String::new();
        }
        let suffix = Self::format_time_precision_suffix(second, millisecond);
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}{suffix}")
    }

    pub(crate) fn parse_time_input_value_ms(raw: &str) -> Option<i64> {
        let (hour, minute, second, millisecond) = parse_time_input_components(raw)?;
        let total_seconds = i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second);
        Some(total_seconds * 1_000 + i64::from(millisecond))
    }

    pub(crate) fn format_time_input_from_timestamp_ms(timestamp_ms: i64) -> String {
        let day_ms = 86_400_000i64;
        let wrapped = timestamp_ms.rem_euclid(day_ms);
        let total_seconds = wrapped / 1_000;
        let hour = total_seconds / 3_600;
        let minute = (total_seconds % 3_600) / 60;
        let second = (total_seconds % 60) as u32;
        let millisecond = wrapped.rem_euclid(1_000) as u32;
        if second == 0 && millisecond == 0 {
            format!("{hour:02}:{minute:02}")
        } else {
            let suffix = Self::format_time_precision_suffix(second, millisecond);
            format!("{hour:02}:{minute:02}{suffix}")
        }
    }

    pub(crate) fn format_number_for_input(value: f64) -> String {
        if value.fract().abs() < 1e-9 {
            format!("{:.0}", value)
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

    pub(crate) fn step_input_value(
        &mut self,
        node: NodeId,
        direction: i64,
        count: i64,
    ) -> Result<()> {
        if count == 0 {
            return Ok(());
        }
        let input_type = self.normalized_input_type(node);
        if !matches!(
            input_type.as_str(),
            "number" | "range" | "date" | "datetime-local" | "time"
        ) {
            return Ok(());
        }

        if input_type == "time" {
            let step_attr = self.dom.attr(node, "step").unwrap_or_default();
            let step_seconds = if step_attr.eq_ignore_ascii_case("any") {
                60.0
            } else {
                step_attr
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
                    .unwrap_or(60.0)
            };
            let step_ms = ((step_seconds * 1_000.0).round() as i64).max(1);
            let min = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_time_input_value_ms(&raw));
            let max = self
                .dom
                .attr(node, "max")
                .and_then(|raw| Self::parse_time_input_value_ms(&raw));
            let base = min
                .or_else(|| {
                    self.dom
                        .attr(node, "value")
                        .and_then(|raw| Self::parse_time_input_value_ms(&raw))
                })
                .unwrap_or(0);
            let current = Self::parse_time_input_value_ms(&self.dom.value(node)?).unwrap_or(base);
            let delta = (direction as i128)
                .saturating_mul(count as i128)
                .saturating_mul(step_ms as i128);
            let day_ms = 86_400_000i64;
            let mut next = (((current as i128) + delta)
                .clamp(i128::from(i64::MIN), i128::from(i64::MAX))
                as i64)
                .rem_euclid(day_ms);

            if let (Some(min), Some(max)) = (min, max) {
                if min <= max {
                    if next < min {
                        next = min;
                    }
                    if next > max {
                        next = max;
                    }
                } else {
                    let in_wrapped_range = next >= min || next <= max;
                    if !in_wrapped_range {
                        next = if direction >= 0 { min } else { max };
                    }
                }
            } else {
                if let Some(min) = min {
                    if next < min {
                        next = min;
                    }
                }
                if let Some(max) = max {
                    if next > max {
                        next = max;
                    }
                }
            }

            let next_value = Self::format_time_input_from_timestamp_ms(next);
            return self.dom.set_value(node, &next_value);
        }

        if input_type == "date" {
            let step_attr = self.dom.attr(node, "step").unwrap_or_default();
            let step_days = if step_attr.eq_ignore_ascii_case("any") {
                1.0
            } else {
                step_attr
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
                    .unwrap_or(1.0)
            };
            let step_ms = ((step_days * 86_400_000.0).round() as i64).max(1);
            let base = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                .or_else(|| {
                    self.dom
                        .attr(node, "value")
                        .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                })
                .unwrap_or(0);
            let current = Self::parse_date_input_value_ms(&self.dom.value(node)?).unwrap_or(base);
            let delta = (direction as i128)
                .saturating_mul(count as i128)
                .saturating_mul(step_ms as i128);
            let mut next = ((current as i128) + delta)
                .clamp(i128::from(i64::MIN), i128::from(i64::MAX))
                as i64;
            if let Some(min) = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
            {
                if next < min {
                    next = min;
                }
            }
            if let Some(max) = self
                .dom
                .attr(node, "max")
                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
            {
                if next > max {
                    next = max;
                }
            }
            let next_value = Self::format_date_input_from_timestamp_ms(next);
            return self.dom.set_value(node, &next_value);
        }

        if input_type == "datetime-local" {
            let step_attr = self.dom.attr(node, "step").unwrap_or_default();
            let step_seconds = if step_attr.eq_ignore_ascii_case("any") {
                60.0
            } else {
                step_attr
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
                    .unwrap_or(60.0)
            };
            let step_ms = ((step_seconds * 1_000.0).round() as i64).max(1);
            let base = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                .or_else(|| {
                    self.dom
                        .attr(node, "value")
                        .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                })
                .unwrap_or(0);
            let current =
                Self::parse_datetime_local_input_value_ms(&self.dom.value(node)?).unwrap_or(base);
            let delta = (direction as i128)
                .saturating_mul(count as i128)
                .saturating_mul(step_ms as i128);
            let mut next = ((current as i128) + delta)
                .clamp(i128::from(i64::MIN), i128::from(i64::MAX))
                as i64;
            if let Some(min) = self
                .dom
                .attr(node, "min")
                .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
            {
                if next < min {
                    next = min;
                }
            }
            if let Some(max) = self
                .dom
                .attr(node, "max")
                .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
            {
                if next > max {
                    next = max;
                }
            }
            let next_value = Self::format_datetime_local_input_from_timestamp_ms(next);
            return self.dom.set_value(node, &next_value);
        }

        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
        let step = if step_attr.eq_ignore_ascii_case("any") {
            1.0
        } else {
            step_attr
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
                .unwrap_or(1.0)
        };
        let base = self
            .parse_attr_f64(node, "min")
            .or_else(|| self.parse_attr_f64(node, "value"))
            .unwrap_or(0.0);
        let current = Self::parse_number_value(&self.dom.value(node)?).unwrap_or(base);
        let mut next = current + (direction as f64) * (count as f64) * step;
        if let Some(min) = self.parse_attr_f64(node, "min") {
            if next < min {
                next = min;
            }
        }
        if let Some(max) = self.parse_attr_f64(node, "max") {
            if next > max {
                next = max;
            }
        }
        self.dom
            .set_value(node, &Self::format_number_for_input(next))
    }

    pub(crate) fn input_value_as_number(&self, node: NodeId) -> Result<f64> {
        let input_type = self.normalized_input_type(node);
        let value = self.dom.value(node)?;
        let number = match input_type.as_str() {
            "number" | "range" => Self::parse_number_value(&value).unwrap_or(f64::NAN),
            "date" => Self::parse_date_input_value_ms(&value)
                .map(|timestamp| timestamp as f64)
                .unwrap_or(f64::NAN),
            "datetime-local" => Self::parse_datetime_local_input_value_ms(&value)
                .map(|timestamp| timestamp as f64)
                .unwrap_or(f64::NAN),
            "time" => Self::parse_time_input_value_ms(&value)
                .map(|timestamp| timestamp as f64)
                .unwrap_or(f64::NAN),
            _ => f64::NAN,
        };
        Ok(number)
    }

    pub(crate) fn set_input_value_as_number(&mut self, node: NodeId, number: f64) -> Result<()> {
        let input_type = self.normalized_input_type(node);
        if input_type == "date" {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            let timestamp_ms = number as i64;
            let formatted = Self::format_date_input_from_timestamp_ms(timestamp_ms);
            return self.dom.set_value(node, &formatted);
        }
        if input_type == "datetime-local" {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            let timestamp_ms = number as i64;
            let formatted = Self::format_datetime_local_input_from_timestamp_ms(timestamp_ms);
            return self.dom.set_value(node, &formatted);
        }
        if input_type == "time" {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            let timestamp_ms = number as i64;
            let formatted = Self::format_time_input_from_timestamp_ms(timestamp_ms);
            return self.dom.set_value(node, &formatted);
        }
        if matches!(input_type.as_str(), "number" | "range") {
            if !number.is_finite() {
                return self.dom.set_value(node, "");
            }
            return self
                .dom
                .set_value(node, &Self::format_number_for_input(number));
        }
        self.dom.set_value(node, "")
    }

    pub(crate) fn input_value_as_date_ms(&self, node: NodeId) -> Result<Option<i64>> {
        let input_type = self.normalized_input_type(node);
        if input_type == "date" {
            return Ok(Self::parse_date_input_value_ms(&self.dom.value(node)?));
        }
        if input_type == "datetime-local" {
            return Ok(Self::parse_datetime_local_input_value_ms(
                &self.dom.value(node)?,
            ));
        }
        if input_type == "time" {
            return Ok(Self::parse_time_input_value_ms(&self.dom.value(node)?));
        }
        if !matches!(input_type.as_str(), "date" | "datetime-local" | "time") {
            return Ok(None);
        }
        Ok(None)
    }

    pub(crate) fn set_input_value_as_date_ms(
        &mut self,
        node: NodeId,
        timestamp_ms: Option<i64>,
    ) -> Result<()> {
        let input_type = self.normalized_input_type(node);
        if !matches!(input_type.as_str(), "date" | "datetime-local" | "time") {
            return self.dom.set_value(node, "");
        }

        let Some(timestamp_ms) = timestamp_ms else {
            return self.dom.set_value(node, "");
        };
        let formatted = if input_type == "date" {
            Self::format_date_input_from_timestamp_ms(timestamp_ms)
        } else if input_type == "time" {
            Self::format_time_input_from_timestamp_ms(timestamp_ms)
        } else {
            Self::format_datetime_local_input_from_timestamp_ms(timestamp_ms)
        };
        self.dom.set_value(node, &formatted)
    }
}
