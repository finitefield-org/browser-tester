use super::*;

impl Harness {
    pub(crate) fn normalized_input_type(&self, node: NodeId) -> String {
        if !self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false)
        {
            return String::new();
        }
        let raw = self
            .dom
            .attr(node, "type")
            .unwrap_or_default()
            .to_ascii_lowercase();
        match raw.as_str() {
            "button" | "checkbox" | "color" | "date" | "datetime-local" | "email" | "file"
            | "hidden" | "image" | "month" | "number" | "password" | "radio" | "range"
            | "reset" | "search" | "submit" | "tel" | "text" | "time" | "url" | "week" => raw,
            _ => "text".to_string(),
        }
    }

    pub(crate) fn node_supports_text_selection(&self, node: NodeId) -> bool {
        if self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("textarea"))
            .unwrap_or(false)
        {
            return true;
        }
        if !self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("input"))
            .unwrap_or(false)
        {
            return false;
        }
        matches!(
            self.normalized_input_type(node).as_str(),
            "text" | "search" | "url" | "tel" | "email" | "password"
        )
    }

    pub(crate) fn normalize_selection_direction(direction: &str) -> &'static str {
        match direction {
            "forward" => "forward",
            "backward" => "backward",
            _ => "none",
        }
    }

    fn scroll_offset_from_object_arg(value: &Value, key: &str) -> Option<i64> {
        let Value::Object(entries) = value else {
            return None;
        };
        let entries = entries.borrow();
        Self::object_get_entry(&entries, key).map(|entry| Self::value_to_i64(&entry))
    }

    pub(crate) fn animate_option_entry(options: Option<&Value>, key: &str) -> Option<Value> {
        let options = options?;
        let Value::Object(entries) = options else {
            return None;
        };
        let entries = entries.borrow();
        Self::object_get_entry(&entries, key)
    }

    pub(crate) fn animate_id_from_options(options: Option<&Value>) -> String {
        match Self::animate_option_entry(options, "id") {
            Some(Value::Null) | Some(Value::Undefined) | None => String::new(),
            Some(value) => value.as_string(),
        }
    }

    pub(crate) fn get_animations_subtree_option(options: Option<&Value>) -> bool {
        let Some(Value::Object(entries)) = options else {
            return false;
        };
        let entries = entries.borrow();
        Self::object_get_entry(&entries, "subtree")
            .map(|value| value.truthy())
            .unwrap_or(false)
    }

    pub(crate) fn register_node_animation(&mut self, target: NodeId, animation: &Value) {
        let Value::Object(animation) = animation else {
            return;
        };
        self.dom_runtime.node_animations.push(NodeAnimationRecord {
            target,
            animation: animation.clone(),
        });
    }

    pub(crate) fn node_get_animations_value(&self, node: NodeId, subtree: bool) -> Value {
        let animations = self
            .dom_runtime
            .node_animations
            .iter()
            .filter(|record| {
                record.target == node || (subtree && self.dom.is_descendant_of(record.target, node))
            })
            .map(|record| Value::Object(record.animation.clone()))
            .collect::<Vec<_>>();
        Self::new_array_value(animations)
    }

    pub(crate) fn apply_document_scroll_operation(&mut self, method: &str, args: &[Value]) -> bool {
        let mut next_x = self.dom_runtime.document_scroll_x;
        let mut next_y = self.dom_runtime.document_scroll_y;

        match method {
            "scroll" | "scrollTo" => match args {
                [] => {}
                [single] => {
                    if matches!(single, Value::Object(_)) {
                        if let Some(left) = Self::scroll_offset_from_object_arg(single, "left") {
                            next_x = left;
                        }
                        if let Some(top) = Self::scroll_offset_from_object_arg(single, "top") {
                            next_y = top;
                        }
                    } else {
                        next_x = Self::value_to_i64(single);
                        next_y = 0;
                    }
                }
                [x, y] => {
                    next_x = Self::value_to_i64(x);
                    next_y = Self::value_to_i64(y);
                }
                _ => {}
            },
            "scrollBy" => {
                let mut delta_x = 0;
                let mut delta_y = 0;
                match args {
                    [] => {}
                    [single] => {
                        if matches!(single, Value::Object(_)) {
                            delta_x =
                                Self::scroll_offset_from_object_arg(single, "left").unwrap_or(0);
                            delta_y =
                                Self::scroll_offset_from_object_arg(single, "top").unwrap_or(0);
                        } else {
                            delta_x = Self::value_to_i64(single);
                        }
                    }
                    [x, y] => {
                        delta_x = Self::value_to_i64(x);
                        delta_y = Self::value_to_i64(y);
                    }
                    _ => {}
                }
                next_x = next_x.saturating_add(delta_x);
                next_y = next_y.saturating_add(delta_y);
            }
            _ => return true,
        }

        let changed = next_x != self.dom_runtime.document_scroll_x
            || next_y != self.dom_runtime.document_scroll_y;
        self.dom_runtime.document_scroll_x = next_x;
        self.dom_runtime.document_scroll_y = next_y;
        changed
    }

    pub(crate) fn set_node_selection_range(
        &mut self,
        node: NodeId,
        start: i64,
        end: i64,
        direction: String,
    ) -> Result<()> {
        if !self.node_supports_text_selection(node) {
            return Ok(());
        }
        let before_start = self.dom.selection_start(node)?;
        let before_end = self.dom.selection_end(node)?;
        let before_direction = self.dom.selection_direction(node)?;
        let start = start.max(0) as usize;
        let end = end.max(0) as usize;
        self.dom.set_selection_range(
            node,
            start,
            end,
            Self::normalize_selection_direction(direction.as_str()),
        )?;
        let after_start = self.dom.selection_start(node)?;
        let after_end = self.dom.selection_end(node)?;
        let after_direction = self.dom.selection_direction(node)?;
        if before_start != after_start
            || before_end != after_end
            || before_direction != after_direction
        {
            let _ = self.dispatch_document_selectionchange()?;
        }
        Ok(())
    }

    pub(crate) fn shift_selection_index(index: usize, delta: i64) -> usize {
        if delta >= 0 {
            index.saturating_add(delta as usize)
        } else {
            index.saturating_sub(delta.unsigned_abs() as usize)
        }
    }

    pub(crate) fn set_node_range_text(&mut self, node: NodeId, args: &[Value]) -> Result<()> {
        if !self.node_supports_text_selection(node) {
            return Ok(());
        }

        let replacement = args[0].as_string();
        let old_value = self.dom.value(node)?;
        let old_len = old_value.chars().count();
        let old_sel_start = self.dom.selection_start(node)?;
        let old_sel_end = self.dom.selection_end(node)?;

        let (mut start, mut end, mode) = match args.len() {
            1 => (old_sel_start, old_sel_end, "preserve".to_string()),
            3 => (
                Self::value_to_i64(&args[1]).max(0) as usize,
                Self::value_to_i64(&args[2]).max(0) as usize,
                "preserve".to_string(),
            ),
            4 => (
                Self::value_to_i64(&args[1]).max(0) as usize,
                Self::value_to_i64(&args[2]).max(0) as usize,
                args[3].as_string(),
            ),
            _ => {
                return Err(Error::ScriptRuntime(
                    "setRangeText supports one, three, or four arguments".into(),
                ));
            }
        };
        start = start.min(old_len);
        end = end.min(old_len);
        if end < start {
            end = start;
        }

        let start_byte = Self::char_index_to_byte(&old_value, start);
        let end_byte = Self::char_index_to_byte(&old_value, end);
        let mut next_value = String::new();
        next_value.push_str(&old_value[..start_byte]);
        next_value.push_str(&replacement);
        next_value.push_str(&old_value[end_byte..]);
        self.dom.set_value(node, &next_value)?;

        let replacement_len = replacement.chars().count();
        let replaced_len = end.saturating_sub(start);
        let delta = replacement_len as i64 - replaced_len as i64;
        let mode = mode.to_ascii_lowercase();
        let (selection_start, selection_end) = match mode.as_str() {
            "select" => (start, start + replacement_len),
            "start" => (start, start),
            "end" => {
                let caret = start + replacement_len;
                (caret, caret)
            }
            _ => {
                if old_sel_end <= start {
                    (old_sel_start, old_sel_end)
                } else if old_sel_start >= end {
                    (
                        Self::shift_selection_index(old_sel_start, delta),
                        Self::shift_selection_index(old_sel_end, delta),
                    )
                } else {
                    let caret = start + replacement_len;
                    (caret, caret)
                }
            }
        };
        self.set_node_selection_range(
            node,
            selection_start as i64,
            selection_end as i64,
            "none".to_string(),
        )
    }

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
