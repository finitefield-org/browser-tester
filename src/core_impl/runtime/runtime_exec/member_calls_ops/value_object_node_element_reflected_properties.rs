use super::*;

impl Harness {
    pub(crate) fn is_node_element_reflected_property_key(key: &str) -> bool {
        matches!(
            key,
            "id" | "name"
                | "lang"
                | "dir"
                | "accessKey"
                | "accesskey"
                | "autocapitalize"
                | "autocorrect"
                | "contentEditable"
                | "contenteditable"
                | "draggable"
                | "enterKeyHint"
                | "enterkeyhint"
                | "inert"
                | "inputMode"
                | "inputmode"
                | "nonce"
                | "popover"
                | "spellcheck"
                | "tabIndex"
                | "tabindex"
                | "translate"
                | "cite"
                | "dateTime"
                | "datetime"
                | "clear"
                | "align"
                | "aLink"
                | "alink"
                | "background"
                | "bgColor"
                | "bgcolor"
                | "bottomMargin"
                | "bottommargin"
                | "leftMargin"
                | "leftmargin"
                | "link"
                | "rightMargin"
                | "rightmargin"
                | "text"
                | "topMargin"
                | "topmargin"
                | "vLink"
                | "vlink"
                | "title"
                | "colSpan"
                | "colspan"
                | "rowSpan"
                | "rowspan"
                | "span"
                | "type"
                | "open"
                | "closedBy"
                | "closedby"
                | "htmlFor"
                | "elementTiming"
                | "elementtiming"
        )
    }

    pub(crate) fn node_element_reflected_property_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Value> {
        let is_select = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("select"))
            .unwrap_or(false);
        let is_button = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("button"))
            .unwrap_or(false);
        let is_col_or_colgroup = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("col") || tag.eq_ignore_ascii_case("colgroup"))
            .unwrap_or(false);
        let is_table_cell = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th"))
            .unwrap_or(false);

        match key {
            "id" => Ok(Value::String(self.dom.attr(node, "id").unwrap_or_default())),
            "name" => Ok(Value::String(
                self.dom.attr(node, "name").unwrap_or_default(),
            )),
            "lang" => Ok(Value::String(
                self.dom.attr(node, "lang").unwrap_or_default(),
            )),
            "dir" => Ok(Value::String(self.resolved_dir_for_node(node))),
            "accessKey" | "accesskey" => Ok(Value::String(
                self.dom.attr(node, "accesskey").unwrap_or_default(),
            )),
            "autocapitalize" => Ok(Value::String(
                self.dom.attr(node, "autocapitalize").unwrap_or_default(),
            )),
            "autocorrect" => Ok(Value::String(
                self.dom.attr(node, "autocorrect").unwrap_or_default(),
            )),
            "contentEditable" | "contenteditable" => Ok(Value::String(
                self.content_editable_property_value_for_node(node),
            )),
            "draggable" => Ok(Value::Bool(self.draggable_property_value_for_node(node))),
            "enterKeyHint" | "enterkeyhint" => Ok(Value::String(
                self.dom.attr(node, "enterkeyhint").unwrap_or_default(),
            )),
            "inert" => Ok(Value::Bool(self.dom.has_attr(node, "inert")?)),
            "inputMode" | "inputmode" => Ok(Value::String(
                self.dom.attr(node, "inputmode").unwrap_or_default(),
            )),
            "nonce" => Ok(Value::String(
                self.dom.attr(node, "nonce").unwrap_or_default(),
            )),
            "popover" => Ok(Value::String(
                self.dom.attr(node, "popover").unwrap_or_default(),
            )),
            "spellcheck" => Ok(Value::Bool(self.spellcheck_property_value_for_node(node))),
            "tabIndex" | "tabindex" => Ok(Value::Number(
                self.reflected_i64_attribute_or_default(node, "tabindex", -1),
            )),
            "translate" => Ok(Value::Bool(self.translate_property_value_for_node(node))),
            "cite" => Ok(Value::String(
                self.reflected_url_attribute_or_empty(node, "cite"),
            )),
            "dateTime" | "datetime" => Ok(Value::String(
                self.dom.attr(node, "datetime").unwrap_or_default(),
            )),
            "clear" => Ok(Value::String(
                self.dom.attr(node, "clear").unwrap_or_default(),
            )),
            "align" => Ok(Value::String(
                self.dom.attr(node, "align").unwrap_or_default(),
            )),
            "aLink" | "alink" => Ok(Value::String(
                self.dom.attr(node, "alink").unwrap_or_default(),
            )),
            "background" => Ok(Value::String(
                self.dom.attr(node, "background").unwrap_or_default(),
            )),
            "bgColor" | "bgcolor" => Ok(Value::String(
                self.dom.attr(node, "bgcolor").unwrap_or_default(),
            )),
            "bottomMargin" | "bottommargin" => Ok(Value::String(
                self.dom.attr(node, "bottommargin").unwrap_or_default(),
            )),
            "leftMargin" | "leftmargin" => Ok(Value::String(
                self.dom.attr(node, "leftmargin").unwrap_or_default(),
            )),
            "link" => Ok(Value::String(
                self.dom.attr(node, "link").unwrap_or_default(),
            )),
            "rightMargin" | "rightmargin" => Ok(Value::String(
                self.dom.attr(node, "rightmargin").unwrap_or_default(),
            )),
            "text" => Ok(Value::String(
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
                {
                    self.dom.attr(node, "text").unwrap_or_default()
                } else {
                    self.dom.text_content(node)
                },
            )),
            "topMargin" | "topmargin" => Ok(Value::String(
                self.dom.attr(node, "topmargin").unwrap_or_default(),
            )),
            "vLink" | "vlink" => Ok(Value::String(
                self.dom.attr(node, "vlink").unwrap_or_default(),
            )),
            "title" => Ok(Value::String(
                self.dom.attr(node, "title").unwrap_or_default(),
            )),
            "colSpan" | "colspan" => {
                if is_table_cell {
                    Ok(Value::Number(self.table_cell_col_span_value(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "rowSpan" | "rowspan" => {
                if is_table_cell {
                    Ok(Value::Number(self.table_cell_row_span_value(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "span" => {
                if is_col_or_colgroup {
                    Ok(Value::Number(self.col_span_value(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "type" => {
                if is_select {
                    Ok(Value::String(self.select_type_property_value(node)))
                } else if is_button {
                    let normalized = self
                        .dom
                        .attr(node, "type")
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                        .map(|value| {
                            if value.eq_ignore_ascii_case("reset") {
                                "reset".to_string()
                            } else if value.eq_ignore_ascii_case("button") {
                                "button".to_string()
                            } else {
                                "submit".to_string()
                            }
                        })
                        .unwrap_or_else(|| "submit".to_string());
                    Ok(Value::String(normalized))
                } else {
                    Ok(Value::String(
                        self.dom.attr(node, "type").unwrap_or_default(),
                    ))
                }
            }
            "open" => Ok(Value::Bool(self.dom.has_attr(node, "open")?)),
            "closedBy" | "closedby" => Ok(Value::String(
                self.dom.attr(node, "closedby").unwrap_or_default(),
            )),
            "htmlFor" => Ok(Value::String(
                self.dom.attr(node, "for").unwrap_or_default(),
            )),
            "elementTiming" | "elementtiming" => Ok(Value::String(
                self.dom.attr(node, "elementtiming").unwrap_or_default(),
            )),
            _ => Ok(Value::Undefined),
        }
    }
}
