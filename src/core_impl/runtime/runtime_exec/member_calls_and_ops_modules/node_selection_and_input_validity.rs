impl Harness {
    pub(crate) fn eval_node_member_call(
        &mut self,
        node: NodeId,
        member: &str,
        evaluated_args: &[Value],
        _event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "getAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                Ok(Some(Value::String(
                    self.dom.attr(node, &name).unwrap_or_default(),
                )))
            }
            "setAttribute" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "setAttribute requires exactly two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let value = evaluated_args[1].as_string();
                self.dom.set_attr(node, &name, &value)?;
                Ok(Some(Value::Undefined))
            }
            "hasAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "hasAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                Ok(Some(Value::Bool(self.dom.has_attr(node, &name)?)))
            }
            "hasAttributes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "hasAttributes takes no arguments".into(),
                    ));
                }
                let has_attributes = self
                    .dom
                    .element(node)
                    .map(|element| !element.attrs.is_empty())
                    .ok_or_else(|| {
                        Error::ScriptRuntime("hasAttributes target is not an element".into())
                    })?;
                Ok(Some(Value::Bool(has_attributes)))
            }
            "removeAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                self.dom.remove_attr(node, &name)?;
                Ok(Some(Value::Undefined))
            }
            "getAttributeNames" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNames takes no arguments".into(),
                    ));
                }
                let element = self.dom.element(node).ok_or_else(|| {
                    Error::ScriptRuntime("getAttributeNames target is not an element".into())
                })?;
                let mut names = element.attrs.keys().cloned().collect::<Vec<_>>();
                names.sort();
                Ok(Some(Self::new_array_value(
                    names.into_iter().map(Value::String).collect(),
                )))
            }
            "toggleAttribute" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "toggleAttribute requires one or two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let has = self.dom.has_attr(node, &name)?;
                let next = if evaluated_args.len() == 2 {
                    evaluated_args[1].truthy()
                } else {
                    !has
                };
                if next {
                    self.dom.set_attr(node, &name, "")?;
                } else {
                    self.dom.remove_attr(node, &name)?;
                }
                Ok(Some(Value::Bool(next)))
            }
            "matches" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "matches requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(Value::Bool(
                    self.dom.matches_selector(node, &selector)?,
                )))
            }
            "closest" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "closest requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom
                        .closest(node, &selector)?
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "querySelector" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelector requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom
                        .query_selector_from(&node, &selector)?
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "querySelectorAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelectorAll requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(Value::NodeList(
                    self.dom.query_selector_all_from(&node, &selector)?,
                )))
            }
            "getElementsByClassName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByClassName requires exactly one argument".into(),
                    ));
                }
                let classes = evaluated_args[0]
                    .as_string()
                    .split_whitespace()
                    .filter(|name| !name.is_empty())
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();
                if classes.is_empty() {
                    return Ok(Some(Value::NodeList(Vec::new())));
                }
                let selector = classes
                    .iter()
                    .map(|class_name| format!(".{class_name}"))
                    .collect::<String>();
                Ok(Some(Value::NodeList(
                    self.dom.query_selector_all_from(&node, &selector)?,
                )))
            }
            "getElementsByTagName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagName requires exactly one argument".into(),
                    ));
                }
                let tag_name = evaluated_args[0].as_string();
                if tag_name == "*" {
                    let mut nodes = Vec::new();
                    self.dom.collect_elements_descendants_dfs(node, &mut nodes);
                    return Ok(Some(Value::NodeList(nodes)));
                }
                Ok(Some(Value::NodeList(
                    self.dom.query_selector_all_from(&node, &tag_name)?,
                )))
            }
            "checkVisibility" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "checkVisibility supports at most one argument".into(),
                    ));
                }
                Ok(Some(Value::Bool(!self.dom.has_attr(node, "hidden")?)))
            }
            "checkValidity" | "reportValidity" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(format!("{member} takes no arguments")));
                }
                let validity = self.compute_input_validity(node)?;
                if !validity.valid {
                    let _ = self.dispatch_event(node, "invalid")?;
                }
                Ok(Some(Value::Bool(validity.valid)))
            }
            "setCustomValidity" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setCustomValidity requires exactly one argument".into(),
                    ));
                }
                self.dom
                    .set_custom_validity_message(node, &evaluated_args[0].as_string())?;
                Ok(Some(Value::Undefined))
            }
            "setSelectionRange" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "setSelectionRange requires two or three arguments".into(),
                    ));
                }
                self.set_node_selection_range(
                    node,
                    Self::value_to_i64(&evaluated_args[0]),
                    Self::value_to_i64(&evaluated_args[1]),
                    evaluated_args
                        .get(2)
                        .map(Value::as_string)
                        .unwrap_or_else(|| "none".to_string()),
                )?;
                Ok(Some(Value::Undefined))
            }
            "setRangeText" => {
                if !(evaluated_args.len() == 1
                    || evaluated_args.len() == 3
                    || evaluated_args.len() == 4)
                {
                    return Err(Error::ScriptRuntime(
                        "setRangeText supports one, three, or four arguments".into(),
                    ));
                }
                self.set_node_range_text(node, evaluated_args)?;
                Ok(Some(Value::Undefined))
            }
            "showPicker" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("showPicker takes no arguments".into()));
                }
                Ok(Some(Value::Undefined))
            }
            "stepUp" | "stepDown" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} supports at most one argument"
                    )));
                }
                let count = evaluated_args.first().map(Self::value_to_i64).unwrap_or(1);
                let direction = if member == "stepDown" { -1 } else { 1 };
                self.step_input_value(node, direction, count)?;
                Ok(Some(Value::Undefined))
            }
            "scrollIntoView" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "scrollIntoView takes no arguments".into(),
                    ));
                }
                Ok(Some(Value::Undefined))
            }
            "scroll" | "scrollTo" | "scrollBy" => {
                if !(evaluated_args.is_empty()
                    || evaluated_args.len() == 1
                    || evaluated_args.len() == 2)
                {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} supports zero, one, or two arguments"
                    )));
                }
                Ok(Some(Value::Undefined))
            }
            "select" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("select takes no arguments".into()));
                }
                if self.node_supports_text_selection(node) {
                    let len = self.dom.value(node)?.chars().count();
                    self.dom.set_selection_range(node, 0, len, "none")?;
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }

    fn normalized_input_type(&self, node: NodeId) -> String {
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

    fn node_supports_text_selection(&self, node: NodeId) -> bool {
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

    fn set_node_selection_range(
        &mut self,
        node: NodeId,
        start: i64,
        end: i64,
        direction: String,
    ) -> Result<()> {
        if !self.node_supports_text_selection(node) {
            return Ok(());
        }
        let start = start.max(0) as usize;
        let end = end.max(0) as usize;
        self.dom.set_selection_range(
            node,
            start,
            end,
            Self::normalize_selection_direction(direction.as_str()),
        )
    }

    fn shift_selection_index(index: usize, delta: i64) -> usize {
        if delta >= 0 {
            index.saturating_add(delta as usize)
        } else {
            index.saturating_sub(delta.unsigned_abs() as usize)
        }
    }

    fn set_node_range_text(&mut self, node: NodeId, args: &[Value]) -> Result<()> {
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
        self.dom
            .set_selection_range(node, selection_start, selection_end, "none")
    }

    fn parse_attr_f64(&self, node: NodeId, name: &str) -> Option<f64> {
        self.dom.attr(node, name).and_then(|raw| {
            let raw = raw.trim();
            if raw.is_empty() {
                None
            } else {
                raw.parse::<f64>().ok().filter(|value| value.is_finite())
            }
        })
    }

    fn parse_attr_i64(&self, node: NodeId, name: &str) -> Option<i64> {
        self.dom.attr(node, name).and_then(|raw| {
            let raw = raw.trim();
            if raw.is_empty() {
                None
            } else {
                raw.parse::<i64>().ok()
            }
        })
    }

    fn parse_number_value(raw: &str) -> Option<f64> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        raw.parse::<f64>().ok().filter(|value| value.is_finite())
    }

    fn format_number_for_input(value: f64) -> String {
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

    fn step_input_value(&mut self, node: NodeId, direction: i64, count: i64) -> Result<()> {
        if count == 0 {
            return Ok(());
        }
        let input_type = self.normalized_input_type(node);
        if !matches!(input_type.as_str(), "number" | "range") {
            return Ok(());
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

    fn is_radio_group_checked(&self, node: NodeId) -> bool {
        let name = self.dom.attr(node, "name").unwrap_or_default();
        if name.is_empty() {
            return self.dom.checked(node).unwrap_or(false);
        }
        let form = self.dom.find_ancestor_by_tag(node, "form");
        self.dom.all_element_nodes().into_iter().any(|candidate| {
            is_radio_input(&self.dom, candidate)
                && self.dom.attr(candidate, "name").unwrap_or_default() == name
                && self.dom.find_ancestor_by_tag(candidate, "form") == form
                && self.dom.checked(candidate).unwrap_or(false)
        })
    }

    fn is_simple_email(value: &str) -> bool {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return false;
        }
        let Some((local, domain)) = trimmed.split_once('@') else {
            return false;
        };
        !local.is_empty() && !domain.is_empty() && !domain.contains('@')
    }

    fn is_url_like(value: &str) -> bool {
        LocationParts::parse(value).is_some()
    }

    fn input_participates_in_constraint_validation(kind: &str) -> bool {
        !matches!(kind, "button" | "submit" | "reset" | "hidden")
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
        let multiple = self.dom.attr(node, "multiple").is_some();

        if required {
            validity.value_missing = if input_type == "checkbox" {
                !self.dom.checked(node)?
            } else if input_type == "radio" {
                !self.is_radio_group_checked(node)
            } else {
                value_is_empty
            };
        }

        if !value_is_empty {
            if input_type == "email" {
                validity.type_mismatch = if multiple {
                    let mut has_part = false;
                    let mut all_ok = true;
                    for part in value.split(',') {
                        let part = part.trim();
                        if part.is_empty() {
                            continue;
                        }
                        has_part = true;
                        if !Self::is_simple_email(part) {
                            all_ok = false;
                            break;
                        }
                    }
                    !has_part || !all_ok
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

            if matches!(input_type.as_str(), "number" | "range") {
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
