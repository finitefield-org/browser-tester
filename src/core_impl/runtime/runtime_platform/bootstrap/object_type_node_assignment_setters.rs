use super::*;

impl Harness {
    pub(crate) fn set_node_assignment_property(
        &mut self,
        node: NodeId,
        key: &str,
        value: Value,
        event: &EventState,
        reflect_set: bool,
    ) -> Result<()> {
        if self.set_node_event_handler_property(node, key, value.clone())? {
            return Ok(());
        }

        if key == "text"
            && self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
        {
            self.dom.set_attr(node, "text", &value.as_string())?;
            return Ok(());
        }

        let is_select = self.is_select_element(node);
        let is_input = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("input"));
        let is_option = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("option"));
        let is_textarea = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("textarea"));
        let is_button = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("button"));
        let is_form = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("form"));
        let is_table_cell = self
            .dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th"));

        if self.node_explicit_own_property_overrides_dom_property(node, key) {
            let mut entries = self.node_expando_entries(node);
            let own_setter = Self::object_setter_from_entries(&entries, key);
            let own_has_accessor = Self::has_object_accessor_property(&entries, key);
            let own_data = Self::object_get_entry(&entries, key).is_some();
            if let Some(setter) = own_setter {
                if !self.is_callable_value(&setter) {
                    return Err(Error::ScriptRuntime("object setter is not callable".into()));
                }
                self.execute_callable_value_with_this_and_env(
                    &setter,
                    &[value],
                    event,
                    None,
                    Some(Value::Node(node)),
                )?;
                return Ok(());
            }
            if own_has_accessor {
                if reflect_set {
                    return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                }
                return Ok(());
            }
            if own_data && !Self::is_writable_object_key(&entries, key) {
                if reflect_set {
                    return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                }
                return Ok(());
            }
            if own_data {
                Self::object_set_entry(&mut entries, key.to_string(), value);
                self.replace_node_expando_entries(node, entries);
                return Ok(());
            }
        }

        if is_select && let Ok(index) = key.parse::<usize>() {
            match value {
                Value::Null | Value::Undefined => {
                    if let Some(option) = self.select_option_nodes(node).get(index).copied() {
                        self.dom.remove_node(option)?;
                    }
                    self.dom.sync_select_value(node)?;
                    return Ok(());
                }
                Value::Node(option) => {
                    let option_tag = self.dom.tag_name(option).unwrap_or_default();
                    if !option_tag.eq_ignore_ascii_case("option")
                        && !option_tag.eq_ignore_ascii_case("optgroup")
                    {
                        return Err(Error::ScriptRuntime(
                            "TypeError: Failed to set an indexed property on 'HTMLSelectElement': value is not an HTMLOptionElement or HTMLOptGroupElement".into(),
                        ));
                    }

                    let options = self.select_option_nodes(node);
                    if let Some(existing) = options.get(index).copied() {
                        self.dom.replace_child(node, option, existing)?;
                    } else {
                        self.dom.append_child(node, option)?;
                    }
                    self.dom.sync_select_value(node)?;
                    return Ok(());
                }
                _ => {
                    return Err(Error::ScriptRuntime(
                        "TypeError: Failed to set an indexed property on 'HTMLSelectElement': value is not an HTMLOptionElement or HTMLOptGroupElement".into(),
                    ));
                }
            }
        }

        match key {
            "textContent" | "innerText" | "text" => {
                self.dom.set_text_content(node, &value.as_string())?
            }
            "nodeValue" => {
                if let NodeType::Text(text) = &mut self.dom.nodes[node.0].node_type {
                    *text = value.as_string();
                }
            }
            "innerHTML" => {
                let html = if matches!(value, Value::Null) {
                    String::new()
                } else {
                    value.as_string()
                };
                self.dom.set_inner_html(node, &html)?
            }
            "outerHTML" => {
                let html = if matches!(value, Value::Null) {
                    String::new()
                } else {
                    value.as_string()
                };
                self.dom.set_outer_html(node, &html)?
            }
            "defaultValue" if is_input => self.dom.set_attr(node, "value", &value.as_string())?,
            "defaultValue" if is_textarea => {
                self.dom
                    .set_textarea_default_value_state(node, value.as_string())?;
            }
            "value" => self.dom.set_value(node, &value.as_string())?,
            "selectedIndex" if is_select => {
                self.set_select_selected_index(node, Self::value_to_i64(&value))?
            }
            "length" if is_select => {
                let next_len = Self::value_to_i64(&value).max(0) as usize;
                self.set_select_length(node, next_len)?;
            }
            "files" => {
                let files = self.mock_files_from_input_assignment_value(&value)?;
                self.dom.set_file_input_files(node, &files)?;
            }
            "defaultChecked" if is_input => {
                self.set_reflected_boolean_attribute(node, "checked", value.truthy())?;
            }
            "checked" => self.dom.set_checked(node, value.truthy())?,
            "defaultSelected" if is_option => {
                self.set_reflected_boolean_attribute(node, "selected", value.truthy())?;
            }
            "selected" if is_option => self
                .dom
                .set_option_selected_property(node, value.truthy())?,
            "indeterminate" => self.dom.set_indeterminate(node, value.truthy())?,
            "open" => {
                self.set_reflected_boolean_attribute(node, "open", value.truthy())?;
            }
            "returnValue" => {
                self.set_dialog_return_value(node, value.as_string())?;
            }
            "closedBy" | "closedby" => self.dom.set_attr(node, "closedby", &value.as_string())?,
            "htmlFor" | "htmlfor" => self.dom.set_attr(node, "for", &value.as_string())?,
            "readOnly" | "readonly" => {
                self.set_reflected_boolean_attribute(node, "readonly", value.truthy())?;
            }
            "required" => {
                self.set_reflected_boolean_attribute(node, "required", value.truthy())?;
            }
            "multiple" if is_select || is_input => {
                self.set_reflected_boolean_attribute(node, "multiple", value.truthy())?;
            }
            "disabled" => {
                self.set_reflected_boolean_attribute(node, "disabled", value.truthy())?;
            }
            "hidden" => {
                if node == self.dom.root {
                    return Err(Error::ScriptRuntime("hidden is read-only".into()));
                }
                self.set_reflected_boolean_attribute(node, "hidden", value.truthy())?;
            }
            "className" | "classList" => self.dom.set_attr(node, "class", &value.as_string())?,
            "part" => self.dom.set_attr(node, "part", &value.as_string())?,
            "id" => self.dom.set_attr(node, "id", &value.as_string())?,
            "slot" => self.dom.set_attr(node, "slot", &value.as_string())?,
            "shadowRoot" => return Err(Error::ScriptRuntime("shadowRoot is read-only".into())),
            "role" => self.dom.set_attr(node, "role", &value.as_string())?,
            "elementTiming" => self
                .dom
                .set_attr(node, "elementtiming", &value.as_string())?,
            "name" => self.dom.set_attr(node, "name", &value.as_string())?,
            "action" if is_form => self.dom.set_attr(node, "action", &value.as_string())?,
            "method" if is_form => self.dom.set_attr(node, "method", &value.as_string())?,
            "enctype" | "encoding" if is_form => {
                self.dom.set_attr(node, "enctype", &value.as_string())?;
            }
            "target" if is_form => self.dom.set_attr(node, "target", &value.as_string())?,
            "acceptCharset" if is_form => {
                self.dom
                    .set_attr(node, "accept-charset", &value.as_string())?;
            }
            "noValidate" if is_form => {
                self.set_reflected_boolean_attribute(node, "novalidate", value.truthy())?;
            }
            "command" => {
                if is_button {
                    self.dom.set_attr(node, "command", &value.as_string())?;
                }
            }
            "commandForElement" => {
                if is_button {
                    match &value {
                        Value::Null | Value::Undefined => {
                            self.dom.remove_attr(node, "commandfor")?;
                        }
                        Value::Node(target) => {
                            let target_id = self.dom.attr(*target, "id").unwrap_or_default();
                            if target_id.is_empty() {
                                self.dom.remove_attr(node, "commandfor")?;
                            } else {
                                self.dom.set_attr(node, "commandfor", &target_id)?;
                            }
                        }
                        _ => self.dom.set_attr(node, "commandfor", &value.as_string())?,
                    }
                }
            }
            "formAction" => {
                if is_button || is_input {
                    self.dom.set_attr(node, "formaction", &value.as_string())?;
                }
            }
            "formEnctype" => {
                if is_button {
                    self.dom.set_attr(node, "formenctype", &value.as_string())?;
                }
            }
            "formMethod" => {
                if is_button {
                    self.dom.set_attr(node, "formmethod", &value.as_string())?;
                }
            }
            "formNoValidate" => {
                if is_button {
                    self.set_reflected_boolean_attribute(node, "formnovalidate", value.truthy())?;
                }
            }
            "formTarget" => {
                if is_button {
                    self.dom.set_attr(node, "formtarget", &value.as_string())?;
                }
            }
            "lang" => self.dom.set_attr(node, "lang", &value.as_string())?,
            "dir" => self.dom.set_attr(node, "dir", &value.as_string())?,
            "accessKey" | "accesskey" => {
                self.dom.set_attr(node, "accesskey", &value.as_string())?
            }
            "autocapitalize" => self
                .dom
                .set_attr(node, "autocapitalize", &value.as_string())?,
            "autocorrect" => self.dom.set_attr(node, "autocorrect", &value.as_string())?,
            "autocomplete" => self
                .dom
                .set_attr(node, "autocomplete", &value.as_string())?,
            "contentEditable" | "contenteditable" => {
                self.set_content_editable_property_value(node, &value)?
            }
            "draggable" => self.set_reflected_keyword_boolean_attribute(
                node,
                "draggable",
                value.truthy(),
                "true",
                "false",
            )?,
            "enterKeyHint" | "enterkeyhint" => {
                self.dom
                    .set_attr(node, "enterkeyhint", &value.as_string())?
            }
            "inert" => {
                self.set_reflected_boolean_attribute(node, "inert", value.truthy())?;
            }
            "inputMode" | "inputmode" => {
                self.dom.set_attr(node, "inputmode", &value.as_string())?
            }
            "nonce" => self.dom.set_attr(node, "nonce", &value.as_string())?,
            "popover" => self.dom.set_attr(node, "popover", &value.as_string())?,
            "spellcheck" => self.set_reflected_keyword_boolean_attribute(
                node,
                "spellcheck",
                value.truthy(),
                "true",
                "false",
            )?,
            "tabIndex" | "tabindex" => {
                self.set_reflected_i64_attribute(node, "tabindex", &value)?
            }
            "translate" => self.set_reflected_keyword_boolean_attribute(
                node,
                "translate",
                value.truthy(),
                "yes",
                "no",
            )?,
            "cite" => self.dom.set_attr(node, "cite", &value.as_string())?,
            "dateTime" | "datetime" => self.dom.set_attr(node, "datetime", &value.as_string())?,
            "clear" => self.dom.set_attr(node, "clear", &value.as_string())?,
            "align" => self.dom.set_attr(node, "align", &value.as_string())?,
            "aLink" | "alink" => self.dom.set_attr(node, "alink", &value.as_string())?,
            "background" => self.dom.set_attr(node, "background", &value.as_string())?,
            "bgColor" | "bgcolor" => self.dom.set_attr(node, "bgcolor", &value.as_string())?,
            "bottomMargin" | "bottommargin" => {
                self.dom
                    .set_attr(node, "bottommargin", &value.as_string())?
            }
            "leftMargin" | "leftmargin" => {
                self.dom.set_attr(node, "leftmargin", &value.as_string())?
            }
            "link" => self.dom.set_attr(node, "link", &value.as_string())?,
            "rightMargin" | "rightmargin" => {
                self.dom.set_attr(node, "rightmargin", &value.as_string())?
            }
            "topMargin" | "topmargin" => {
                self.dom.set_attr(node, "topmargin", &value.as_string())?
            }
            "vLink" | "vlink" => self.dom.set_attr(node, "vlink", &value.as_string())?,
            "title" => self.dom.set_attr(node, "title", &value.as_string())?,
            "alt" => self.dom.set_attr(node, "alt", &value.as_string())?,
            "colSpan" | "colspan" if is_table_cell => {
                self.set_table_cell_col_span_value(node, &value)?
            }
            "rowSpan" | "rowspan" if is_table_cell => {
                self.set_table_cell_row_span_value(node, &value)?
            }
            "span"
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("col") || tag.eq_ignore_ascii_case("colgroup")
                }) =>
            {
                self.set_col_span_value(node, &value)?
            }
            "src" => self.dom.set_attr(node, "src", &value.as_string())?,
            "autoplay" => {
                self.set_reflected_boolean_attribute(node, "autoplay", value.truthy())?;
            }
            "controls" => {
                self.set_reflected_boolean_attribute(node, "controls", value.truthy())?;
            }
            "controlsList" | "controlslist" => {
                self.dom
                    .set_attr(node, "controlslist", &value.as_string())?
            }
            "crossOrigin" | "crossorigin" => {
                self.dom.set_attr(node, "crossorigin", &value.as_string())?
            }
            "disableRemotePlayback" | "disableremoteplayback" => {
                self.set_reflected_boolean_attribute(
                    node,
                    "disableremoteplayback",
                    value.truthy(),
                )?;
            }
            "disablePictureInPicture" | "disablepictureinpicture" => {
                self.set_reflected_boolean_attribute(
                    node,
                    "disablepictureinpicture",
                    value.truthy(),
                )?;
            }
            "loop" => {
                self.set_reflected_boolean_attribute(node, "loop", value.truthy())?;
            }
            "defaultMuted" => {
                self.set_reflected_boolean_attribute(node, "muted", value.truthy())?;
            }
            "muted" => {
                self.set_reflected_boolean_attribute(node, "muted", value.truthy())?;
            }
            "currentTime"
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                self.set_media_numeric_state_value(node, INTERNAL_MEDIA_CURRENT_TIME_KEY, &value);
            }
            "volume"
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                self.set_media_numeric_state_value(node, INTERNAL_MEDIA_VOLUME_KEY, &value);
            }
            "playbackRate"
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                self.set_media_numeric_state_value(node, INTERNAL_MEDIA_PLAYBACK_RATE_KEY, &value);
                self.with_script_env(|this, env| {
                    let _ = this.dispatch_event_with_options(
                        node,
                        "ratechange",
                        env,
                        true,
                        false,
                        false,
                        None,
                        None,
                        None,
                    )?;
                    Ok(())
                })?;
            }
            "defaultPlaybackRate"
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                self.set_media_numeric_state_value(
                    node,
                    INTERNAL_MEDIA_DEFAULT_PLAYBACK_RATE_KEY,
                    &value,
                );
            }
            "duration" | "paused" | "ended" | "seeking" | "networkState" | "readyState"
            | "textTracks" | "buffered" | "seekable" | "played"
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                }) =>
            {
                if reflect_set {
                    return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                }
            }
            "preload" => self.dom.set_attr(node, "preload", &value.as_string())?,
            "playsInline" | "playsinline" => {
                self.set_reflected_boolean_attribute(node, "playsinline", value.truthy())?;
            }
            "poster" => self.dom.set_attr(node, "poster", &value.as_string())?,
            "attributionSrc" | "attributionsrc" => {
                self.dom
                    .set_attr(node, "attributionsrc", &value.as_string())?
            }
            "data" => self.dom.set_attr(node, "data", &value.as_string())?,
            "srcdoc" | "srcDoc" => self.dom.set_attr(node, "srcdoc", &value.as_string())?,
            "download" => self.dom.set_attr(node, "download", &value.as_string())?,
            "hash" => self.set_anchor_url_property(node, "hash", value.clone())?,
            "host" => self.set_anchor_url_property(node, "host", value.clone())?,
            "hostname" => self.set_anchor_url_property(node, "hostname", value.clone())?,
            "href" => self.set_anchor_url_property(node, "href", value.clone())?,
            "hreflang" => self.dom.set_attr(node, "hreflang", &value.as_string())?,
            "interestForElement" => {
                if is_button {
                    match &value {
                        Value::Null | Value::Undefined => {
                            self.dom.remove_attr(node, "interestfor")?;
                        }
                        Value::Node(target) => {
                            let target_id = self.dom.attr(*target, "id").unwrap_or_default();
                            if target_id.is_empty() {
                                self.dom.remove_attr(node, "interestfor")?;
                            } else {
                                self.dom.set_attr(node, "interestfor", &target_id)?;
                            }
                        }
                        _ => self.dom.set_attr(node, "interestfor", &value.as_string())?,
                    }
                } else {
                    self.dom.set_attr(node, "interestfor", &value.as_string())?;
                }
            }
            "popoverTargetAction" => {
                if is_button {
                    self.dom
                        .set_attr(node, "popovertargetaction", &value.as_string())?;
                }
            }
            "popoverTargetElement" => {
                if is_button {
                    match &value {
                        Value::Null | Value::Undefined => {
                            self.dom.remove_attr(node, "popovertarget")?;
                        }
                        Value::Node(target) => {
                            let target_id = self.dom.attr(*target, "id").unwrap_or_default();
                            if target_id.is_empty() {
                                self.dom.remove_attr(node, "popovertarget")?;
                            } else {
                                self.dom.set_attr(node, "popovertarget", &target_id)?;
                            }
                        }
                        _ => self
                            .dom
                            .set_attr(node, "popovertarget", &value.as_string())?,
                    }
                }
            }
            "password" => self.set_anchor_url_property(node, "password", value.clone())?,
            "pathname" => self.set_anchor_url_property(node, "pathname", value.clone())?,
            "ping" => self.dom.set_attr(node, "ping", &value.as_string())?,
            "port" => self.set_anchor_url_property(node, "port", value.clone())?,
            "protocol" => self.set_anchor_url_property(node, "protocol", value.clone())?,
            "referrerPolicy" => self
                .dom
                .set_attr(node, "referrerpolicy", &value.as_string())?,
            "rel" => self.dom.set_attr(node, "rel", &value.as_string())?,
            "search" => self.set_anchor_url_property(node, "search", value.clone())?,
            "target" => self.dom.set_attr(node, "target", &value.as_string())?,
            "size" if is_select => self.set_select_size_property_value(node, &value)?,
            "size" if is_input => self.set_input_size_property_value(node, &value)?,
            "min" | "max" | "step" if is_input => {
                self.dom.set_attr(node, key, &value.as_string())?
            }
            "maxLength" | "maxlength" if is_input || is_textarea => {
                self.set_max_length_property_value(node, &value)?
            }
            "minLength" | "minlength" if is_input || is_textarea => {
                self.set_min_length_property_value(node, &value)?
            }
            "rows" if is_textarea => self.set_textarea_rows_property_value(node, &value)?,
            "cols" if is_textarea => self.set_textarea_cols_property_value(node, &value)?,
            "type" if is_select => {}
            "type" => self.dom.set_attr(node, "type", &value.as_string())?,
            "mozOpaque" | "mozopaque"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas")) =>
            {
                self.set_reflected_boolean_attribute(node, "moz-opaque", value.truthy())?;
            }
            "mozPrintCallback" | "mozprintcallback"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas")) =>
            {
                self.dom_runtime.node_expando_props.insert(
                    (node, key.to_string()),
                    if matches!(value, Value::Function(_)) {
                        value
                    } else {
                        Value::Null
                    },
                );
            }
            "kind"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("track")) =>
            {
                self.dom.set_attr(node, "kind", &value.as_string())?
            }
            "track"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("track")) =>
            {
                if reflect_set {
                    return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                }
            }
            "noHref" | "nohref" => {
                self.set_reflected_boolean_attribute(node, "nohref", value.truthy())?;
            }
            "srclang" | "srcLang"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("track")) =>
            {
                self.dom.set_attr(node, "srclang", &value.as_string())?
            }
            "label"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("track")) =>
            {
                self.dom.set_attr(node, "label", &value.as_string())?
            }
            "default"
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("track")) =>
            {
                self.set_reflected_boolean_attribute(node, "default", value.truthy())?;
            }
            "media" => self.dom.set_attr(node, "media", &value.as_string())?,
            "sizes" => self.dom.set_attr(node, "sizes", &value.as_string())?,
            "srcset" | "srcSet" => self.dom.set_attr(node, "srcset", &value.as_string())?,
            "useMap" | "usemap" => self.dom.set_attr(node, "usemap", &value.as_string())?,
            "width" => self.set_canvas_dimension_value(node, "width", &value)?,
            "height" => self.set_canvas_dimension_value(node, "height", &value)?,
            "username" => self.set_anchor_url_property(node, "username", value.clone())?,
            "charset" => self.dom.set_attr(node, "charset", &value.as_string())?,
            "coords" => self.dom.set_attr(node, "coords", &value.as_string())?,
            "rev" => self.dom.set_attr(node, "rev", &value.as_string())?,
            "shape" => self.dom.set_attr(node, "shape", &value.as_string())?,
            _ => {
                self.dom_runtime
                    .node_expando_props
                    .insert((node, key.to_string()), value);
            }
        }
        Ok(())
    }
}
