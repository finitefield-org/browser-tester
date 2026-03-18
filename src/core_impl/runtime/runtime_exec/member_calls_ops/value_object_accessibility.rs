use super::*;

impl Harness {
    pub(crate) fn resolved_dir_for_node(&self, node: NodeId) -> String {
        if let Some(explicit) = self.dom.attr(node, "dir") {
            return explicit;
        }
        if self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("bdi"))
        {
            return "auto".to_string();
        }
        String::new()
    }

    fn resolved_static_role_for_tag(tag: &str) -> Option<&'static str> {
        match tag {
            "address" => Some("group"),
            "aside" => Some("complementary"),
            "article" => Some("article"),
            "blockquote" => Some("blockquote"),
            "body" | "b" | "bdi" | "bdo" | "data" | "div" | "i" | "pre" | "q" | "samp"
            | "small" | "u" => Some("generic"),
            "button" => Some("button"),
            "caption" => Some("caption"),
            "code" => Some("code"),
            "datalist" => Some("listbox"),
            "details" | "fieldset" | "hgroup" | "optgroup" => Some("group"),
            "dialog" => Some("dialog"),
            "del" | "s" => Some("deletion"),
            "dfn" => Some("term"),
            "em" => Some("emphasis"),
            "figure" => Some("figure"),
            "form" => Some("form"),
            "hr" => Some("separator"),
            "html" => Some("document"),
            "ins" => Some("insertion"),
            "main" => Some("main"),
            "ol" | "menu" | "ul" => Some("list"),
            "meter" => Some("meter"),
            "nav" => Some("navigation"),
            "option" => Some("option"),
            "output" => Some("status"),
            "p" => Some("paragraph"),
            "progress" => Some("progressbar"),
            "strong" => Some("strong"),
            "sub" => Some("subscript"),
            "sup" => Some("superscript"),
            "table" => Some("table"),
            "tbody" | "tfoot" | "thead" => Some("rowgroup"),
            "tr" => Some("row"),
            "textarea" => Some("textbox"),
            "time" => Some("time"),
            "search" => Some("search"),
            _ => None,
        }
    }

    fn is_heading_tag(tag: &str) -> bool {
        matches!(tag.as_bytes(), [b'h' | b'H', b'1'..=b'6'])
    }

    pub(crate) fn resolved_role_for_node(&self, node: NodeId) -> String {
        if let Some(explicit) = self.dom.attr(node, "role") {
            return explicit;
        }
        let Some(tag) = self.dom.tag_name(node) else {
            return String::new();
        };
        let normalized_tag = tag.to_ascii_lowercase();

        match normalized_tag.as_str() {
            "header" => return self.resolved_header_role(node),
            "input" => return self.resolved_input_role(node),
            "footer" => return self.resolved_footer_role(node),
            "img" => {
                if self.dom.attr(node, "alt").is_some_and(|alt| alt.is_empty()) {
                    return "presentation".to_string();
                }
                return "img".to_string();
            }
            "li" => return self.resolved_list_item_role(node),
            "th" => return self.resolved_table_header_role(node),
            "td" => return self.resolved_table_data_cell_role(node),
            "select" => return self.resolved_select_role(node),
            "section" => return self.resolved_section_role(node),
            "a" | "area" | "link" if self.dom.attr(node, "href").is_some() => {
                return "link".to_string();
            }
            _ => {}
        }

        if Self::is_heading_tag(normalized_tag.as_str()) {
            return "heading".to_string();
        }

        Self::resolved_static_role_for_tag(normalized_tag.as_str())
            .unwrap_or_default()
            .to_string()
    }

    pub(crate) fn footer_has_scoped_ancestor(&self, node: NodeId) -> bool {
        let mut cursor = self.dom.parent(node);
        while let Some(parent) = cursor {
            if self.dom.tag_name(parent).is_some_and(|tag| {
                tag.eq_ignore_ascii_case("article")
                    || tag.eq_ignore_ascii_case("aside")
                    || tag.eq_ignore_ascii_case("main")
                    || tag.eq_ignore_ascii_case("nav")
                    || tag.eq_ignore_ascii_case("section")
            }) {
                return true;
            }

            if self.dom.attr(parent, "role").is_some_and(|role| {
                let normalized = role.trim().to_ascii_lowercase();
                matches!(
                    normalized.as_str(),
                    "article" | "complementary" | "main" | "navigation" | "region"
                )
            }) {
                return true;
            }

            cursor = self.dom.parent(parent);
        }
        false
    }

    pub(crate) fn resolved_footer_role(&self, node: NodeId) -> String {
        if self.footer_has_scoped_ancestor(node) {
            "generic".to_string()
        } else {
            "contentinfo".to_string()
        }
    }

    pub(crate) fn resolved_header_role(&self, node: NodeId) -> String {
        if self.footer_has_scoped_ancestor(node) {
            "generic".to_string()
        } else {
            "banner".to_string()
        }
    }

    pub(crate) fn has_accessible_name_for_landmark(&self, node: NodeId) -> bool {
        if self
            .dom
            .attr(node, "aria-label")
            .is_some_and(|value| !value.trim().is_empty())
        {
            return true;
        }

        let Some(raw_ids) = self.dom.attr(node, "aria-labelledby") else {
            return false;
        };

        raw_ids.split_whitespace().any(|id_ref| {
            self.dom
                .by_id(id_ref)
                .is_some_and(|label_node| !self.dom.text_content(label_node).trim().is_empty())
        })
    }

    pub(crate) fn resolved_section_role(&self, node: NodeId) -> String {
        if self.has_accessible_name_for_landmark(node) {
            "region".to_string()
        } else {
            "generic".to_string()
        }
    }

    pub(crate) fn resolved_input_role(&self, node: NodeId) -> String {
        let input_type = self
            .dom
            .attr(node, "type")
            .unwrap_or_else(|| "text".to_string())
            .trim()
            .to_ascii_lowercase();
        let has_list = self.dom.attr(node, "list").is_some();

        match input_type.as_str() {
            "button" | "image" | "reset" | "submit" => "button".to_string(),
            "checkbox" => "checkbox".to_string(),
            "number" => "spinbutton".to_string(),
            "radio" => "radio".to_string(),
            "range" => "slider".to_string(),
            "search" => {
                if has_list {
                    "combobox".to_string()
                } else {
                    "searchbox".to_string()
                }
            }
            "color" | "date" | "datetime-local" | "file" | "hidden" | "month" | "password"
            | "time" | "week" => String::new(),
            "email" | "tel" | "text" | "url" => {
                if has_list {
                    "combobox".to_string()
                } else {
                    "textbox".to_string()
                }
            }
            _ => {
                if has_list {
                    "combobox".to_string()
                } else {
                    "textbox".to_string()
                }
            }
        }
    }

    pub(crate) fn resolved_list_item_role(&self, node: NodeId) -> String {
        let Some(parent) = self.dom.parent(node) else {
            return String::new();
        };
        if self.dom.tag_name(parent).is_some_and(|tag| {
            tag.eq_ignore_ascii_case("ol")
                || tag.eq_ignore_ascii_case("ul")
                || tag.eq_ignore_ascii_case("menu")
        }) {
            "listitem".to_string()
        } else {
            String::new()
        }
    }

    pub(crate) fn resolved_select_role(&self, node: NodeId) -> String {
        let multiple = self.dom.attr(node, "multiple").is_some();
        let size_is_listbox = self
            .dom
            .attr(node, "size")
            .and_then(|raw| Self::parse_non_negative_int(&raw))
            .is_some_and(|size| size > 1);
        if !multiple && !size_is_listbox {
            "combobox".to_string()
        } else {
            "listbox".to_string()
        }
    }

    pub(crate) fn resolved_table_data_cell_role(&self, node: NodeId) -> String {
        let mut cursor = self.dom.parent(node);
        let mut has_table_ancestor = false;

        while let Some(parent) = cursor {
            if self
                .dom
                .attr(parent, "role")
                .is_some_and(|role| role.trim().eq_ignore_ascii_case("grid"))
            {
                return "gridcell".to_string();
            }

            if self
                .dom
                .tag_name(parent)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("table"))
            {
                has_table_ancestor = true;
            }

            cursor = self.dom.parent(parent);
        }

        if has_table_ancestor {
            "cell".to_string()
        } else {
            String::new()
        }
    }

    pub(crate) fn resolved_table_header_role(&self, node: NodeId) -> String {
        if let Some(scope) = self.dom.attr(node, "scope") {
            let scope = scope.trim().to_ascii_lowercase();
            if matches!(scope.as_str(), "row" | "rowgroup") {
                return "rowheader".to_string();
            }
            if matches!(scope.as_str(), "col" | "colgroup") {
                return "columnheader".to_string();
            }
        }

        let Some(parent) = self.dom.parent(node) else {
            return "columnheader".to_string();
        };
        if !self
            .dom
            .tag_name(parent)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("tr"))
        {
            return "columnheader".to_string();
        }

        let has_data_cell_sibling = self.dom.nodes[parent.0].children.iter().any(|child| {
            self.dom
                .tag_name(*child)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("td"))
        });

        if has_data_cell_sibling {
            "rowheader".to_string()
        } else {
            "columnheader".to_string()
        }
    }
}
