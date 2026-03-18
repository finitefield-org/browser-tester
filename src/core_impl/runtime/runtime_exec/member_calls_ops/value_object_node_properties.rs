use super::*;

impl Harness {
    pub(crate) fn object_property_from_node_value(
        &mut self,
        node: &NodeId,
        key: &str,
    ) -> Result<Value> {
        let is_select = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("select"))
            .unwrap_or(false);
        let is_button = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("button"))
            .unwrap_or(false);
        let is_form = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("form"))
            .unwrap_or(false);
        let is_media = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video"))
            .unwrap_or(false);
        let is_col_or_colgroup = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("col") || tag.eq_ignore_ascii_case("colgroup"))
            .unwrap_or(false);
        let is_table_cell = self
            .dom
            .tag_name(*node)
            .map(|tag| tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th"))
            .unwrap_or(false);

        if is_select {
            if let Ok(index) = key.parse::<usize>() {
                return Ok(self
                    .select_option_nodes(*node)
                    .get(index)
                    .copied()
                    .map(Value::Node)
                    .unwrap_or(Value::Undefined));
            }
        }

        if self.node_explicit_own_property_overrides_dom_property(*node, key) {
            let entries = self.node_expando_entries(*node);
            if let Some(value) =
                self.object_property_from_entries_with_getter(&Value::Node(*node), &entries, key)?
            {
                return Ok(value);
            }
        }

        if let Some(value) = self.node_receiver_builtin_method(*node, key) {
            return Ok(value);
        }

        match key {
            "nodeType" => Ok(Value::Number(self.node_type_number(*node))),
            "nodeName" => Ok(Value::String(self.node_name(*node))),
            "nodeValue" => Ok(self.node_value(*node)),
            "ownerDocument" => Ok(self
                .node_owner_document(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "parentNode" => Ok(self
                .dom
                .parent(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "parentElement" => Ok(self
                .node_parent_element(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "nextSibling" => Ok(self
                .node_next_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "previousSibling" => Ok(self
                .node_previous_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "isConnected" => Ok(Value::Bool(self.dom.is_connected(*node))),
            "childNodes" => Ok(self.child_nodes_live_list_value(*node)),
            "attributes" => {
                if self.dom.element(*node).is_some() {
                    Ok(self.named_node_map_live_value(*node))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "children" => Ok(self.child_elements_live_list_value(*node)),
            "childElementCount" => Ok(Value::Number(self.dom.child_element_count(*node) as i64)),
            "firstChild" => Ok(self.dom.nodes[node.0]
                .children
                .first()
                .copied()
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "lastChild" => Ok(self.dom.nodes[node.0]
                .children
                .last()
                .copied()
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "firstElementChild" => Ok(self
                .dom
                .first_element_child(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "lastElementChild" => Ok(self
                .dom
                .last_element_child(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "nextElementSibling" => Ok(self
                .dom
                .next_element_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "previousElementSibling" => Ok(self
                .dom
                .previous_element_sibling(*node)
                .map(Value::Node)
                .unwrap_or(Value::Null)),
            "shadowRoot" => Ok(self.shadow_root_property_value(*node)),
            "content"
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("template")) =>
            {
                self.template_content_fragment_value(*node)
            }
            "textContent" => Ok(self.node_text_content_value(*node)),
            "innerText" => Ok(Value::String(self.dom.text_content(*node))),
            "innerHTML" => Ok(Value::String(self.dom.inner_html(*node)?)),
            "outerHTML" => Ok(Value::String(self.dom.outer_html(*node)?)),
            "defaultValue"
            | "value"
            | "files"
            | "valueAsNumber"
            | "valueAsDate"
            | "defaultChecked"
            | "checked"
            | "defaultSelected"
            | "selected"
            | "disabled"
            | "required"
            | "multiple"
            | "readonly"
            | "readOnly"
            | "autocomplete"
            | "form"
            | "elements"
            | "action"
            | "method"
            | "enctype"
            | "encoding" => self.node_form_control_property_value(*node, key),
            "target" => {
                if is_form
                    || self.dom.tag_name(*node).is_some_and(|tag| {
                        tag.eq_ignore_ascii_case("a")
                            || tag.eq_ignore_ascii_case("area")
                            || tag.eq_ignore_ascii_case("base")
                    })
                {
                    Ok(Value::String(
                        self.dom.attr(*node, "target").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "acceptCharset"
            | "noValidate"
            | "command"
            | "commandForElement"
            | "formAction" => self.node_form_control_property_value(*node, key),
            "href" => Ok(Value::String(self.resolve_anchor_href(*node))),
            "download"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "download").unwrap_or_default(),
                ))
            }
            "hreflang"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("link")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "hreflang").unwrap_or_default(),
                ))
            }
            "ping"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "ping").unwrap_or_default(),
                ))
            }
            "referrerPolicy" | "referrerpolicy"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("img")
                        || tag.eq_ignore_ascii_case("link")
                        || tag.eq_ignore_ascii_case("script")
                        || tag.eq_ignore_ascii_case("iframe")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "referrerpolicy").unwrap_or_default(),
                ))
            }
            "rel"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("link")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "rel").unwrap_or_default(),
                ))
            }
            "alt"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("img")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "alt").unwrap_or_default(),
                ))
            }
            "charset"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "charset").unwrap_or_default(),
                ))
            }
            "coords"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "coords").unwrap_or_default(),
                ))
            }
            "rev"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "rev").unwrap_or_default(),
                ))
            }
            "shape"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::String(
                    self.dom.attr(*node, "shape").unwrap_or_default(),
                ))
            }
            "noHref" | "nohref"
                if self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area")
                }) =>
            {
                Ok(Value::Bool(self.dom.attr(*node, "nohref").is_some()))
            }
            "formEnctype"
            | "formMethod"
            | "formNoValidate"
            | "formTarget"
            | "labels" => self.node_form_control_property_value(*node, key),
            "id" => Ok(Value::String(
                self.dom.attr(*node, "id").unwrap_or_default(),
            )),
            "name" => Ok(Value::String(
                self.dom.attr(*node, "name").unwrap_or_default(),
            )),
            "interestForElement" | "popoverTargetAction" | "popoverTargetElement" => {
                self.node_form_control_property_value(*node, key)
            }
            "lang" => Ok(Value::String(
                self.dom.attr(*node, "lang").unwrap_or_default(),
            )),
            "dir" => Ok(Value::String(self.resolved_dir_for_node(*node))),
            "accessKey" | "accesskey" => Ok(Value::String(
                self.dom.attr(*node, "accesskey").unwrap_or_default(),
            )),
            "autocapitalize" => Ok(Value::String(
                self.dom.attr(*node, "autocapitalize").unwrap_or_default(),
            )),
            "autocorrect" => Ok(Value::String(
                self.dom.attr(*node, "autocorrect").unwrap_or_default(),
            )),
            "contentEditable" | "contenteditable" => Ok(Value::String(
                self.content_editable_property_value_for_node(*node),
            )),
            "draggable" => Ok(Value::Bool(self.draggable_property_value_for_node(*node))),
            "enterKeyHint" | "enterkeyhint" => Ok(Value::String(
                self.dom.attr(*node, "enterkeyhint").unwrap_or_default(),
            )),
            "inert" => Ok(Value::Bool(self.dom.has_attr(*node, "inert")?)),
            "inputMode" | "inputmode" => Ok(Value::String(
                self.dom.attr(*node, "inputmode").unwrap_or_default(),
            )),
            "nonce" => Ok(Value::String(
                self.dom.attr(*node, "nonce").unwrap_or_default(),
            )),
            "popover" => Ok(Value::String(
                self.dom.attr(*node, "popover").unwrap_or_default(),
            )),
            "spellcheck" => Ok(Value::Bool(self.spellcheck_property_value_for_node(*node))),
            "tabIndex" | "tabindex" => Ok(Value::Number(
                self.reflected_i64_attribute_or_default(*node, "tabindex", -1),
            )),
            "translate" => Ok(Value::Bool(self.translate_property_value_for_node(*node))),
            "cite" => Ok(Value::String(
                self.reflected_url_attribute_or_empty(*node, "cite"),
            )),
            "dateTime" | "datetime" => Ok(Value::String(
                self.dom.attr(*node, "datetime").unwrap_or_default(),
            )),
            "clear" => Ok(Value::String(
                self.dom.attr(*node, "clear").unwrap_or_default(),
            )),
            "align" => Ok(Value::String(
                self.dom.attr(*node, "align").unwrap_or_default(),
            )),
            "aLink" | "alink" => Ok(Value::String(
                self.dom.attr(*node, "alink").unwrap_or_default(),
            )),
            "background" => Ok(Value::String(
                self.dom.attr(*node, "background").unwrap_or_default(),
            )),
            "bgColor" | "bgcolor" => Ok(Value::String(
                self.dom.attr(*node, "bgcolor").unwrap_or_default(),
            )),
            "bottomMargin" | "bottommargin" => Ok(Value::String(
                self.dom.attr(*node, "bottommargin").unwrap_or_default(),
            )),
            "leftMargin" | "leftmargin" => Ok(Value::String(
                self.dom.attr(*node, "leftmargin").unwrap_or_default(),
            )),
            "link" => Ok(Value::String(
                self.dom.attr(*node, "link").unwrap_or_default(),
            )),
            "rightMargin" | "rightmargin" => Ok(Value::String(
                self.dom.attr(*node, "rightmargin").unwrap_or_default(),
            )),
            "text" => Ok(Value::String(
                if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
                {
                    self.dom.attr(*node, "text").unwrap_or_default()
                } else {
                    self.dom.text_content(*node)
                },
            )),
            "topMargin" | "topmargin" => Ok(Value::String(
                self.dom.attr(*node, "topmargin").unwrap_or_default(),
            )),
            "vLink" | "vlink" => Ok(Value::String(
                self.dom.attr(*node, "vlink").unwrap_or_default(),
            )),
            "title" => Ok(Value::String(
                self.dom.attr(*node, "title").unwrap_or_default(),
            )),
            "colSpan" | "colspan" if is_table_cell => {
                Ok(Value::Number(self.table_cell_col_span_value(*node)))
            }
            "rowSpan" | "rowspan" if is_table_cell => {
                Ok(Value::Number(self.table_cell_row_span_value(*node)))
            }
            "span" if is_col_or_colgroup => Ok(Value::Number(self.col_span_value(*node))),
            "type" => {
                if is_select {
                    Ok(Value::String(self.select_type_property_value(*node)))
                } else if self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("button"))
                {
                    let normalized = self
                        .dom
                        .attr(*node, "type")
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
                        self.dom.attr(*node, "type").unwrap_or_default(),
                    ))
                }
            }
            "kind"
            | "track"
            | "srclang"
            | "srcLang"
            | "label"
            | "default"
            | "readyState"
            | "defaultMuted"
            | "autoplay"
            | "controls"
            | "loop"
            | "muted"
            | "controlsList"
            | "controlslist"
            | "crossOrigin"
            | "crossorigin"
            | "disableRemotePlayback"
            | "disableremoteplayback"
            | "disablePictureInPicture"
            | "disablepictureinpicture"
            | "media"
            | "playsInline"
            | "playsinline"
            | "paused"
            | "ended"
            | "seeking"
            | "networkState"
            | "currentTime"
            | "volume"
            | "duration"
            | "playbackRate"
            | "defaultPlaybackRate"
            | "textTracks"
            | "buffered"
            | "seekable"
            | "played"
            | "currentSrc"
            | "currentsrc"
            | "complete"
            | "naturalWidth"
            | "naturalHeight"
            | "src"
            | "poster"
            | "attributionSrc"
            | "attributionsrc"
            | "data"
            | "srcdoc"
            | "srcDoc"
            | "preload"
            | "sizes"
            | "srcset"
            | "srcSet"
            | "useMap"
            | "usemap"
            | "width"
            | "height"
            | "mozOpaque"
            | "mozopaque"
            | "mozPrintCallback"
            | "mozprintcallback" => self.node_media_property_value(*node, key),
            "tagName" => Ok(Value::String(self.element_tag_name(*node))),
            "localName" => Ok(Value::String(
                self.dom
                    .tag_name(*node)
                    .map(|name| {
                        name.rsplit_once(':')
                            .map(|(_, local)| local)
                            .unwrap_or(name)
                            .to_ascii_lowercase()
                    })
                    .unwrap_or_default(),
            )),
            "namespaceURI" => Ok(self
                .dom
                .element(*node)
                .and_then(|element| element.namespace_uri.clone())
                .map(Value::String)
                .unwrap_or(Value::Null)),
            "prefix" => Ok(self
                .dom
                .tag_name(*node)
                .and_then(|name| name.split_once(':').map(|(prefix, _)| prefix))
                .map(|prefix| Value::String(prefix.to_string()))
                .unwrap_or(Value::Null)),
            "className" => Ok(Value::String(
                self.dom.attr(*node, "class").unwrap_or_default(),
            )),
            "classList" => Ok(self.class_list_live_value(*node)),
            "slot" => Ok(Value::String(
                self.dom.attr(*node, "slot").unwrap_or_default(),
            )),
            "role" => {
                let role = self.resolved_role_for_node(*node);
                if role.is_empty() {
                    Ok(Value::Null)
                } else if is_button {
                    Ok(Value::String("button".to_string()))
                } else {
                    Ok(Value::String(role))
                }
            }
            "baseURI" => Ok(Value::String(self.document_base_url())),
            "dataset" => Ok(self.dom_string_map_live_value(*node)),
            "open" => Ok(Value::Bool(self.dom.has_attr(*node, "open")?)),
            "closedBy" | "closedby" => Ok(Value::String(
                self.dom.attr(*node, "closedby").unwrap_or_default(),
            )),
            "htmlFor" => Ok(Value::String(
                self.dom.attr(*node, "for").unwrap_or_default(),
            )),
            "elementTiming" | "elementtiming" => Ok(Value::String(
                self.dom.attr(*node, "elementtiming").unwrap_or_default(),
            )),
            "options"
            | "selectedIndex"
            | "selectedOptions"
            | "size"
            | "min"
            | "max"
            | "step"
            | "maxLength"
            | "maxlength"
            | "minLength"
            | "minlength"
            | "rows"
            | "cols"
            | "validationMessage"
            | "validity"
            | "willValidate"
            | "length"
            | "captureStream"
            | "getContext"
            | "toDataURL"
            | "toBlob"
            | "transferControlToOffscreen" => {
                if matches!(
                    key,
                    "options"
                        | "selectedIndex"
                        | "selectedOptions"
                        | "size"
                        | "min"
                        | "max"
                        | "step"
                        | "maxLength"
                        | "maxlength"
                        | "minLength"
                        | "minlength"
                        | "rows"
                        | "cols"
                        | "validationMessage"
                        | "validity"
                        | "willValidate"
                        | "length"
                ) {
                    self.node_form_control_property_value(*node, key)
                } else {
                    self.node_media_property_value(*node, key)
                }
            }
            _ if key.starts_with("on") => {
                let is_body_window_alias = self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
                    && key
                        .strip_prefix("on")
                        .map(|event_type| event_type.to_ascii_lowercase())
                        .is_some_and(|event_type| {
                            Self::is_body_window_event_handler_alias(event_type.as_str())
                        });
                if is_body_window_alias {
                    Ok(
                        Self::object_get_entry(&self.dom_runtime.window_object.borrow(), key)
                            .unwrap_or(Value::Null),
                    )
                } else {
                    Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(*node, key.to_string()))
                        .cloned()
                        .unwrap_or(Value::Null))
                }
            }
            _ => Ok(self
                .dom_runtime
                .node_expando_props
                .get(&(*node, key.to_string()))
                .cloned()
                .or(if is_media {
                    self.html_media_builtin_property_value(*node, key)?
                } else {
                    None
                })
                .or(if is_form {
                    self.form_builtin_property_value(key)
                } else {
                    None
                })
                .or(if is_form {
                    self.form_named_property_value(*node, key)?
                } else {
                    None
                })
                .unwrap_or(Value::Undefined)),
        }
    }
}
