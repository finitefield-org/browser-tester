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
}
