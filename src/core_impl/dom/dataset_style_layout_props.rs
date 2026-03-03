use super::*;

fn parse_css_pixel_value(raw: &str) -> Option<f64> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    let numeric = if let Some(stripped) = normalized.strip_suffix("px") {
        stripped.trim()
    } else {
        normalized.as_str()
    };
    numeric
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

fn parse_padding_vertical_shorthand(value: &str) -> Option<(f64, f64)> {
    let tokens = value
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return None;
    }

    let mut values = Vec::with_capacity(tokens.len());
    for token in tokens {
        values.push(parse_css_pixel_value(token)?);
    }

    let vertical = match values.len() {
        1 => (values[0], values[0]),
        2 => (values[0], values[0]),
        3 => (values[0], values[2]),
        _ => (values[0], values[2]),
    };
    Some(vertical)
}

fn parse_padding_horizontal_shorthand(value: &str) -> Option<(f64, f64)> {
    let tokens = value
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return None;
    }

    let mut values = Vec::with_capacity(tokens.len());
    for token in tokens {
        values.push(parse_css_pixel_value(token)?);
    }

    let horizontal = match values.len() {
        1 => (values[0], values[0]),
        2 => (values[1], values[1]),
        3 => (values[1], values[1]),
        _ => (values[3], values[1]),
    };
    Some(horizontal)
}

fn clamp_layout_px_to_i64(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    value.max(0.0).trunc() as i64
}

fn parse_css_border_width_token(token: &str) -> Option<f64> {
    let normalized = token.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    match normalized.as_str() {
        "thin" => Some(1.0),
        "medium" => Some(3.0),
        "thick" => Some(5.0),
        _ => parse_css_pixel_value(normalized.as_str()),
    }
}

fn parse_border_width_from_shorthand(value: &str) -> Option<f64> {
    value
        .split_whitespace()
        .find_map(parse_css_border_width_token)
}

#[derive(Clone, Copy)]
enum BorderSide {
    Top,
    Left,
}

fn parse_border_width_side_from_shorthand(value: &str, side: BorderSide) -> Option<f64> {
    let tokens = value
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return None;
    }

    let mut values = Vec::with_capacity(tokens.len());
    for token in tokens {
        values.push(parse_css_border_width_token(token)?);
    }

    let (top, left) = match values.len() {
        1 => (values[0], values[0]),
        2 => (values[0], values[1]),
        3 => (values[0], values[1]),
        _ => (values[0], values[3]),
    };

    Some(match side {
        BorderSide::Top => top,
        BorderSide::Left => left,
    })
}

impl Dom {
    pub(crate) fn dataset_get(&self, node_id: NodeId, key: &str) -> Result<String> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "dataset target is not an element".into(),
            ));
        }
        let name = dataset_key_to_attr_name(key);
        Ok(self.attr(node_id, &name).unwrap_or_default())
    }

    pub(crate) fn dataset_set(&mut self, node_id: NodeId, key: &str, value: &str) -> Result<()> {
        let name = dataset_key_to_attr_name(key);
        self.set_attr(node_id, &name, value)
    }

    pub(crate) fn style_get(&self, node_id: NodeId, key: &str) -> Result<String> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("style target is not an element".into()))?;
        let name = js_prop_to_css_name(key);
        let decls = parse_style_declarations(element.attrs.get("style").map(String::as_str));
        Ok(decls
            .iter()
            .find(|(prop, _)| prop == &name)
            .map(|(_, value)| value.clone())
            .unwrap_or_default())
    }

    pub(crate) fn style_set(&mut self, node_id: NodeId, key: &str, value: &str) -> Result<()> {
        let name = js_prop_to_css_name(key);
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("style target is not an element".into()))?;

        let mut decls = parse_style_declarations(element.attrs.get("style").map(String::as_str));
        if let Some(pos) = decls.iter().position(|(prop, _)| prop == &name) {
            if value.is_empty() {
                decls.remove(pos);
            } else {
                decls[pos].1 = value.to_string();
            }
        } else if !value.is_empty() {
            decls.push((name, value.to_string()));
        }

        if decls.is_empty() {
            // Keep an empty style attribute to match CSSStyleDeclaration behavior.
            element.attrs.insert("style".to_string(), String::new());
        } else {
            element
                .attrs
                .insert("style".to_string(), serialize_style_declarations(&decls));
        }

        Ok(())
    }

    pub(crate) fn offset_left(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetLeft target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn offset_top(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetTop target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn offset_width(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetWidth target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn offset_height(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetHeight target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn client_width(&self, node_id: NodeId) -> Result<i64> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("clientWidth target is not an element".into()))?;

        if !self.is_connected(node_id) {
            return Ok(0);
        }

        let style_decls = parse_style_declarations(element.attrs.get("style").map(String::as_str));
        let display = style_decls
            .iter()
            .find(|(name, _)| name == "display")
            .map(|(_, value)| value.trim().to_ascii_lowercase())
            .unwrap_or_default();
        if display == "none" || display == "inline" || element.attrs.contains_key("hidden") {
            return Ok(0);
        }

        let css_width = style_decls
            .iter()
            .find(|(name, _)| name == "width")
            .and_then(|(_, value)| parse_css_pixel_value(value))
            .unwrap_or(0.0);

        let mut padding_left = 0.0;
        let mut padding_right = 0.0;

        if let Some((_, value)) = style_decls.iter().find(|(name, _)| name == "padding") {
            if let Some((left, right)) = parse_padding_horizontal_shorthand(value) {
                padding_left = left;
                padding_right = right;
            }
        }
        if let Some((_, value)) = style_decls.iter().find(|(name, _)| name == "padding-left") {
            if let Some(parsed) = parse_css_pixel_value(value) {
                padding_left = parsed;
            }
        }
        if let Some((_, value)) = style_decls.iter().find(|(name, _)| name == "padding-right") {
            if let Some(parsed) = parse_css_pixel_value(value) {
                padding_right = parsed;
            }
        }

        Ok(clamp_layout_px_to_i64(css_width + padding_left + padding_right))
    }

    pub(crate) fn client_height(&self, node_id: NodeId) -> Result<i64> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("clientHeight target is not an element".into()))?;

        if !self.is_connected(node_id) {
            return Ok(0);
        }

        let style_decls = parse_style_declarations(element.attrs.get("style").map(String::as_str));
        let display_none = style_decls
            .iter()
            .find(|(name, _)| name == "display")
            .map(|(_, value)| value.trim().eq_ignore_ascii_case("none"))
            .unwrap_or(false);
        if display_none || element.attrs.contains_key("hidden") {
            return Ok(0);
        }

        let css_height = style_decls
            .iter()
            .find(|(name, _)| name == "height")
            .and_then(|(_, value)| parse_css_pixel_value(value))
            .unwrap_or(0.0);

        let mut padding_top = 0.0;
        let mut padding_bottom = 0.0;

        if let Some((_, value)) = style_decls.iter().find(|(name, _)| name == "padding") {
            if let Some((top, bottom)) = parse_padding_vertical_shorthand(value) {
                padding_top = top;
                padding_bottom = bottom;
            }
        }
        if let Some((_, value)) = style_decls.iter().find(|(name, _)| name == "padding-top") {
            if let Some(parsed) = parse_css_pixel_value(value) {
                padding_top = parsed;
            }
        }
        if let Some((_, value)) = style_decls
            .iter()
            .find(|(name, _)| name == "padding-bottom")
        {
            if let Some(parsed) = parse_css_pixel_value(value) {
                padding_bottom = parsed;
            }
        }

        Ok(clamp_layout_px_to_i64(
            css_height + padding_top + padding_bottom,
        ))
    }

    pub(crate) fn client_left(&self, node_id: NodeId) -> Result<i64> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("clientLeft target is not an element".into()))?;

        if !self.is_connected(node_id) {
            return Ok(0);
        }

        let style_decls = parse_style_declarations(element.attrs.get("style").map(String::as_str));
        let display = style_decls
            .iter()
            .find(|(name, _)| name == "display")
            .map(|(_, value)| value.trim().to_ascii_lowercase())
            .unwrap_or_default();
        if display == "none" || display == "inline" || element.attrs.contains_key("hidden") {
            return Ok(0);
        }

        let border_left_width = style_decls
            .iter()
            .find(|(name, _)| name == "border-left-width")
            .and_then(|(_, value)| parse_css_border_width_token(value))
            .or_else(|| {
                style_decls
                    .iter()
                    .find(|(name, _)| name == "border-left")
                    .and_then(|(_, value)| parse_border_width_from_shorthand(value))
            })
            .or_else(|| {
                style_decls
                    .iter()
                    .find(|(name, _)| name == "border-width")
                    .and_then(|(_, value)| {
                        parse_border_width_side_from_shorthand(value, BorderSide::Left)
                    })
            })
            .or_else(|| {
                style_decls
                    .iter()
                    .find(|(name, _)| name == "border")
                    .and_then(|(_, value)| parse_border_width_from_shorthand(value))
            })
            .unwrap_or(0.0);

        Ok(clamp_layout_px_to_i64(border_left_width))
    }

    pub(crate) fn client_top(&self, node_id: NodeId) -> Result<i64> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("clientTop target is not an element".into()))?;

        if !self.is_connected(node_id) {
            return Ok(0);
        }

        let style_decls = parse_style_declarations(element.attrs.get("style").map(String::as_str));
        let display = style_decls
            .iter()
            .find(|(name, _)| name == "display")
            .map(|(_, value)| value.trim().to_ascii_lowercase())
            .unwrap_or_default();
        if display == "none" || display == "inline" || element.attrs.contains_key("hidden") {
            return Ok(0);
        }

        let border_top_width = style_decls
            .iter()
            .find(|(name, _)| name == "border-top-width")
            .and_then(|(_, value)| parse_css_border_width_token(value))
            .or_else(|| {
                style_decls
                    .iter()
                    .find(|(name, _)| name == "border-top")
                    .and_then(|(_, value)| parse_border_width_from_shorthand(value))
            })
            .or_else(|| {
                style_decls
                    .iter()
                    .find(|(name, _)| name == "border-width")
                    .and_then(|(_, value)| {
                        parse_border_width_side_from_shorthand(value, BorderSide::Top)
                    })
            })
            .or_else(|| {
                style_decls
                    .iter()
                    .find(|(name, _)| name == "border")
                    .and_then(|(_, value)| parse_border_width_from_shorthand(value))
            })
            .unwrap_or(0.0);

        Ok(clamp_layout_px_to_i64(border_top_width))
    }

    pub(crate) fn scroll_width(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollWidth target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn scroll_height(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollHeight target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn scroll_left(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollLeft target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(crate) fn scroll_top(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollTop target is not an element".into(),
            ));
        }
        Ok(0)
    }
}
