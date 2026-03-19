use super::*;

impl Harness {
    pub(crate) fn is_radio_group_checked(&self, node: NodeId) -> bool {
        let name = self.dom.attr(node, "name").unwrap_or_default();
        if name.is_empty() {
            return self.dom.checked(node).unwrap_or(false);
        }
        let form = self.dom.control_form_owner(node);
        self.dom.all_element_nodes().into_iter().any(|candidate| {
            is_radio_input(&self.dom, candidate)
                && self.dom.attr(candidate, "name").unwrap_or_default() == name
                && self.dom.control_form_owner(candidate) == form
                && self.dom.checked(candidate).unwrap_or(false)
        })
    }

    pub(crate) fn is_ascii_email_local_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                '.' | '!'
                    | '#'
                    | '$'
                    | '%'
                    | '&'
                    | '\''
                    | '*'
                    | '+'
                    | '/'
                    | '='
                    | '?'
                    | '^'
                    | '_'
                    | '`'
                    | '{'
                    | '|'
                    | '}'
                    | '~'
                    | '-'
            )
    }

    pub(crate) fn is_valid_email_domain_label(label: &str) -> bool {
        if label.is_empty() || label.len() > 63 {
            return false;
        }

        let mut chars = label.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !first.is_ascii_alphanumeric() {
            return false;
        }

        let mut last = first;
        for ch in chars {
            if !(ch.is_ascii_alphanumeric() || ch == '-') {
                return false;
            }
            last = ch;
        }

        last.is_ascii_alphanumeric()
    }

    pub(crate) fn is_valid_email_domain(domain: &str) -> bool {
        !domain.is_empty() && domain.split('.').all(Self::is_valid_email_domain_label)
    }

    pub(crate) fn is_simple_email(value: &str) -> bool {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return false;
        }
        let Some((local, domain)) = trimmed.split_once('@') else {
            return false;
        };
        if local.is_empty() || domain.is_empty() || domain.contains('@') {
            return false;
        }
        if !local.chars().all(Self::is_ascii_email_local_char) {
            return false;
        }
        Self::is_valid_email_domain(domain)
    }

    pub(crate) fn is_email_address_list(value: &str) -> bool {
        if value.trim().is_empty() {
            return true;
        }

        for part in value.split(',') {
            let part = part.trim();
            if part.is_empty() || !Self::is_simple_email(part) {
                return false;
            }
        }
        true
    }

    pub(crate) fn is_url_like(value: &str) -> bool {
        LocationParts::parse(value).is_some()
    }

    pub(crate) fn input_participates_in_constraint_validation(kind: &str) -> bool {
        !matches!(kind, "button" | "submit" | "reset" | "hidden" | "image")
    }

    pub(crate) fn compute_input_validity(&self, node: NodeId) -> Result<InputValidity> {
        let mut validity = InputValidity {
            valid: true,
            ..InputValidity::default()
        };

        if self.is_effectively_disabled(node) {
            return Ok(validity);
        }

        let Some(tag_name) = self.dom.tag_name(node) else {
            return Ok(validity);
        };
        if tag_name.eq_ignore_ascii_case("textarea") {
            let value = self.dom.value(node)?;
            let required = self.dom.required(node);
            let readonly = self.dom.readonly(node);

            if required && !readonly && value.is_empty() {
                validity.value_missing = true;
            }

            if !value.is_empty() {
                let value_len = value.chars().count() as i64;
                if let Some(min_len) = self.parse_attr_i64(node, "minlength") {
                    if min_len >= 0 && value_len < min_len {
                        validity.too_short = true;
                    }
                }
                if let Some(max_len) = self.parse_attr_i64(node, "maxlength") {
                    if max_len >= 0 && value_len > max_len {
                        validity.too_long = true;
                    }
                }
            }

            validity.custom_error = !self.dom.custom_validity_message(node)?.is_empty();
            validity.valid = !(validity.value_missing
                || validity.type_mismatch
                || validity.pattern_mismatch
                || validity.too_long
                || validity.too_short
                || validity.range_underflow
                || validity.range_overflow
                || validity.step_mismatch
                || validity.bad_input
                || validity.custom_error);
            return Ok(validity);
        }
        if tag_name.eq_ignore_ascii_case("select") {
            let value = self.dom.value(node)?;
            if self.dom.required(node) && value.is_empty() {
                validity.value_missing = true;
            }
            validity.custom_error = !self.dom.custom_validity_message(node)?.is_empty();
            validity.valid = !(validity.value_missing || validity.custom_error);
            return Ok(validity);
        }
        if tag_name.eq_ignore_ascii_case("button") {
            if !self.button_will_validate(node) {
                return Ok(validity);
            }
            let custom_error = !self.dom.custom_validity_message(node)?.is_empty();
            validity.custom_error = custom_error;
            validity.valid = !custom_error;
            return Ok(validity);
        }
        if !tag_name.eq_ignore_ascii_case("input") {
            let custom_error = !self.dom.custom_validity_message(node)?.is_empty();
            validity.custom_error = custom_error;
            validity.valid = !custom_error;
            return Ok(validity);
        }

        let input_type = self.normalized_input_type(node);
        if !Self::input_participates_in_constraint_validation(input_type.as_str()) {
            return Ok(validity);
        }
        let value = self.dom.value(node)?;
        let value_is_empty = value.is_empty();
        let required = self.dom.required(node);
        let readonly = self.dom.readonly(node);
        let multiple = self.dom.attr(node, "multiple").is_some();
        let email_multiple = input_type == "email" && multiple;
        let value_is_effectively_empty = if email_multiple {
            value.trim().is_empty()
        } else {
            value_is_empty
        };

        if required && !readonly && Self::input_supports_required(input_type.as_str()) {
            validity.value_missing = if input_type == "checkbox" {
                !self.dom.checked(node)?
            } else if input_type == "radio" {
                !self.is_radio_group_checked(node)
            } else if email_multiple {
                false
            } else {
                value_is_effectively_empty
            };
        }

        if !value_is_effectively_empty {
            if input_type == "email" {
                validity.type_mismatch = if email_multiple {
                    !Self::is_email_address_list(&value)
                } else {
                    !Self::is_simple_email(&value)
                };
            } else if input_type == "url" {
                validity.type_mismatch = !Self::is_url_like(&value);
            }

            if matches!(
                input_type.as_str(),
                "text" | "search" | "url" | "tel" | "email" | "password"
            ) {
                let value_len = value.chars().count() as i64;
                if let Some(min_len) = self.parse_attr_i64(node, "minlength") {
                    if min_len >= 0 && value_len < min_len {
                        validity.too_short = true;
                    }
                }
                if let Some(max_len) = self.parse_attr_i64(node, "maxlength") {
                    if max_len >= 0 && value_len > max_len {
                        validity.too_long = true;
                    }
                }

                if let Some(pattern) = self.dom.attr(node, "pattern") {
                    if !pattern.is_empty() {
                        let wrapped = format!("^(?:{})$", pattern);
                        if let Ok(regex) = Regex::new(&wrapped) {
                            if input_type == "email" && multiple {
                                for part in value.split(',') {
                                    let part = part.trim();
                                    if part.is_empty() {
                                        continue;
                                    }
                                    match regex.is_match(part) {
                                        Ok(true) => {}
                                        Ok(false) => {
                                            validity.pattern_mismatch = true;
                                            break;
                                        }
                                        Err(_) => {}
                                    }
                                }
                            } else if let Ok(false) = regex.is_match(&value) {
                                validity.pattern_mismatch = true;
                            }
                        }
                    }
                }
            }

            if input_type == "date" {
                match Self::parse_date_input_value_ms(&value) {
                    Some(date_value_ms) => {
                        if let Some(min) = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                        {
                            if date_value_ms < min {
                                validity.range_underflow = true;
                            }
                        }
                        if let Some(max) = self
                            .dom
                            .attr(node, "max")
                            .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                        {
                            if date_value_ms > max {
                                validity.range_overflow = true;
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        if !step_attr.eq_ignore_ascii_case("any") {
                            let step_days = step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(1.0);
                            let step_ms = step_days * 86_400_000.0;
                            let base = self
                                .dom
                                .attr(node, "min")
                                .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                                .or_else(|| {
                                    self.dom
                                        .attr(node, "value")
                                        .and_then(|raw| Self::parse_date_input_value_ms(&raw))
                                })
                                .unwrap_or(0) as f64;
                            let ratio = ((date_value_ms as f64) - base) / step_ms;
                            let nearest = ratio.round();
                            if (ratio - nearest).abs() > 1e-7 {
                                validity.step_mismatch = true;
                            }
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            } else if input_type == "datetime-local" {
                match Self::parse_datetime_local_input_value_ms(&value) {
                    Some(datetime_value_ms) => {
                        if let Some(min) = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                        {
                            if datetime_value_ms < min {
                                validity.range_underflow = true;
                            }
                        }
                        if let Some(max) = self
                            .dom
                            .attr(node, "max")
                            .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                        {
                            if datetime_value_ms > max {
                                validity.range_overflow = true;
                            }
                        }

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
                        let step_ms = step_seconds * 1_000.0;
                        let base = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                            .or_else(|| {
                                self.dom
                                    .attr(node, "value")
                                    .and_then(|raw| Self::parse_datetime_local_input_value_ms(&raw))
                            })
                            .unwrap_or(0) as f64;
                        let ratio = ((datetime_value_ms as f64) - base) / step_ms;
                        let nearest = ratio.round();
                        if (ratio - nearest).abs() > 1e-7 {
                            validity.step_mismatch = true;
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            } else if input_type == "time" {
                match Self::parse_time_input_value_ms(&value) {
                    Some(time_value_ms) => {
                        let min = self
                            .dom
                            .attr(node, "min")
                            .and_then(|raw| Self::parse_time_input_value_ms(&raw));
                        let max = self
                            .dom
                            .attr(node, "max")
                            .and_then(|raw| Self::parse_time_input_value_ms(&raw));
                        if let (Some(min), Some(max)) = (min, max) {
                            if min <= max {
                                if time_value_ms < min {
                                    validity.range_underflow = true;
                                }
                                if time_value_ms > max {
                                    validity.range_overflow = true;
                                }
                            } else {
                                let in_wrapped_range = time_value_ms >= min || time_value_ms <= max;
                                if !in_wrapped_range {
                                    validity.range_underflow = true;
                                    validity.range_overflow = true;
                                }
                            }
                        } else {
                            if let Some(min) = min {
                                if time_value_ms < min {
                                    validity.range_underflow = true;
                                }
                            }
                            if let Some(max) = max {
                                if time_value_ms > max {
                                    validity.range_overflow = true;
                                }
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        if !step_attr.eq_ignore_ascii_case("any") {
                            let step_seconds = step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(60.0);
                            let step_ms = step_seconds * 1_000.0;
                            let base = self
                                .dom
                                .attr(node, "min")
                                .and_then(|raw| Self::parse_time_input_value_ms(&raw))
                                .or_else(|| {
                                    self.dom
                                        .attr(node, "value")
                                        .and_then(|raw| Self::parse_time_input_value_ms(&raw))
                                })
                                .unwrap_or(0) as f64;
                            let ratio = ((time_value_ms as f64) - base) / step_ms;
                            let nearest = ratio.round();
                            if (ratio - nearest).abs() > 1e-7 {
                                validity.step_mismatch = true;
                            }
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            } else if matches!(input_type.as_str(), "number" | "range") {
                match Self::parse_number_value(&value) {
                    Some(numeric) => {
                        if let Some(min) = self.parse_attr_f64(node, "min") {
                            if numeric < min {
                                validity.range_underflow = true;
                            }
                        }
                        if let Some(max) = self.parse_attr_f64(node, "max") {
                            if numeric > max {
                                validity.range_overflow = true;
                            }
                        }

                        let step_attr = self.dom.attr(node, "step").unwrap_or_default();
                        if !step_attr.eq_ignore_ascii_case("any") {
                            let step = step_attr
                                .trim()
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite() && *value > 0.0)
                                .unwrap_or(1.0);
                            let base = self
                                .parse_attr_f64(node, "min")
                                .or_else(|| self.parse_attr_f64(node, "value"))
                                .unwrap_or(0.0);
                            let ratio = (numeric - base) / step;
                            let nearest = ratio.round();
                            if (ratio - nearest).abs() > 1e-7 {
                                validity.step_mismatch = true;
                            }
                        }
                    }
                    None => {
                        validity.bad_input = true;
                    }
                }
            }
        }

        validity.custom_error = !self.dom.custom_validity_message(node)?.is_empty();
        validity.valid = !(validity.value_missing
            || validity.type_mismatch
            || validity.pattern_mismatch
            || validity.too_long
            || validity.too_short
            || validity.range_underflow
            || validity.range_overflow
            || validity.step_mismatch
            || validity.bad_input
            || validity.custom_error);
        Ok(validity)
    }

    pub(crate) fn input_validity_to_value(validity: &InputValidity) -> Value {
        Self::new_object_value(vec![
            (
                "valueMissing".to_string(),
                Value::Bool(validity.value_missing),
            ),
            (
                "typeMismatch".to_string(),
                Value::Bool(validity.type_mismatch),
            ),
            (
                "patternMismatch".to_string(),
                Value::Bool(validity.pattern_mismatch),
            ),
            ("tooLong".to_string(), Value::Bool(validity.too_long)),
            ("tooShort".to_string(), Value::Bool(validity.too_short)),
            (
                "rangeUnderflow".to_string(),
                Value::Bool(validity.range_underflow),
            ),
            (
                "rangeOverflow".to_string(),
                Value::Bool(validity.range_overflow),
            ),
            (
                "stepMismatch".to_string(),
                Value::Bool(validity.step_mismatch),
            ),
            ("badInput".to_string(), Value::Bool(validity.bad_input)),
            (
                "customError".to_string(),
                Value::Bool(validity.custom_error),
            ),
            ("valid".to_string(), Value::Bool(validity.valid)),
        ])
    }
}
