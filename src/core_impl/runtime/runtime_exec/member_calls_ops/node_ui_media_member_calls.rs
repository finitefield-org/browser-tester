use super::*;

impl Harness {
    pub(crate) fn try_eval_node_ui_media_member_call(
        &mut self,
        node: NodeId,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "add" => {
                if !self.is_select_element(node) {
                    return Ok(None);
                }
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "add on HTMLSelectElement requires one or two arguments".into(),
                    ));
                }
                let option = match evaluated_args.first() {
                    Some(Value::Node(option)) => *option,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "TypeError: Failed to execute 'add' on 'HTMLSelectElement': parameter 1 is not of type 'HTMLElement'"
                                .into(),
                        ));
                    }
                };
                let option_tag = self.dom.tag_name(option).unwrap_or_default();
                if !option_tag.eq_ignore_ascii_case("option")
                    && !option_tag.eq_ignore_ascii_case("optgroup")
                {
                    return Err(Error::ScriptRuntime(
                        "TypeError: Failed to execute 'add' on 'HTMLSelectElement': parameter 1 is not of type 'HTMLElement'"
                            .into(),
                    ));
                }

                let before = match evaluated_args.get(1) {
                    None | Some(Value::Undefined) | Some(Value::Null) => None,
                    Some(Value::Node(candidate)) if self.dom.parent(*candidate) == Some(node) => {
                        Some(*candidate)
                    }
                    Some(value) => self
                        .value_as_index(value)
                        .and_then(|index| self.select_option_nodes(node).get(index).copied()),
                };

                if let Some(before) = before {
                    self.dom.insert_before(node, option, before)?;
                } else {
                    self.dom.append_child(node, option)?;
                }
                self.dom.sync_select_value(node)?;
                Ok(Some(Value::Undefined))
            }
            "item" => {
                if !self.is_select_element(node) {
                    return Ok(None);
                }
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "item on HTMLSelectElement requires exactly one index argument".into(),
                    ));
                }
                let index = Self::value_to_i64(&evaluated_args[0]);
                if index < 0 {
                    return Ok(Some(Value::Null));
                }
                Ok(Some(
                    self.select_option_nodes(node)
                        .get(index as usize)
                        .copied()
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "namedItem" => {
                if !self.is_select_element(node) {
                    return Ok(None);
                }
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "namedItem on HTMLSelectElement requires exactly one name argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                Ok(Some(
                    self.select_named_item(node, &name)
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "remove" => {
                if self.is_select_element(node) {
                    match evaluated_args.len() {
                        0 => {}
                        1 => {
                            let index = Self::value_to_i64(&evaluated_args[0]);
                            if index >= 0 {
                                if let Some(option) =
                                    self.select_option_nodes(node).get(index as usize).copied()
                                {
                                    self.dom.remove_node(option)?;
                                }
                                self.dom.sync_select_value(node)?;
                            }
                            return Ok(Some(Value::Undefined));
                        }
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "remove on HTMLSelectElement supports at most one index argument"
                                    .into(),
                            ));
                        }
                    }
                } else if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("remove takes no arguments".into()));
                }
                if evaluated_args.is_empty() {
                    if let Some(active) = self.dom.active_element() {
                        if active == node || self.dom.is_descendant_of(active, node) {
                            self.dom.set_active_element(None);
                        }
                    }
                    if let Some(active_pseudo) = self.dom.active_pseudo_element() {
                        if active_pseudo == node || self.dom.is_descendant_of(active_pseudo, node) {
                            self.dom.set_active_pseudo_element(None);
                        }
                    }
                    self.dom.remove_node(node)?;
                }
                Ok(Some(Value::Undefined))
            }
            "focus" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("focus takes no arguments".into()));
                }
                self.focus_node(node)?;
                Ok(Some(Value::Undefined))
            }
            "blur" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("blur takes no arguments".into()));
                }
                self.blur_node(node)?;
                Ok(Some(Value::Undefined))
            }
            "setPointerCapture" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setPointerCapture requires exactly one pointerId argument".into(),
                    ));
                }
                let pointer_id = Self::value_to_i64(&evaluated_args[0]);
                if pointer_id <= 0 {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'setPointerCapture': No active pointer with the given id"
                            .into(),
                    ));
                }
                self.dom_runtime
                    .pointer_capture_targets
                    .insert(pointer_id, node);
                Ok(Some(Value::Undefined))
            }
            "hasPointerCapture" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "hasPointerCapture requires exactly one pointerId argument".into(),
                    ));
                }
                let pointer_id = Self::value_to_i64(&evaluated_args[0]);
                let has_capture = self
                    .dom_runtime
                    .pointer_capture_targets
                    .get(&pointer_id)
                    .is_some_and(|captured_node| *captured_node == node);
                Ok(Some(Value::Bool(has_capture)))
            }
            "releasePointerCapture" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "releasePointerCapture requires exactly one pointerId argument".into(),
                    ));
                }
                let pointer_id = Self::value_to_i64(&evaluated_args[0]);
                let Some(captured_node) = self
                    .dom_runtime
                    .pointer_capture_targets
                    .get(&pointer_id)
                    .copied()
                else {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'releasePointerCapture': No active pointer with the given id"
                            .into(),
                    ));
                };
                if captured_node == node {
                    self.dom_runtime.pointer_capture_targets.remove(&pointer_id);
                }
                Ok(Some(Value::Undefined))
            }
            "captureStream" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "captureStream supports at most one argument".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                let frame_rate = evaluated_args
                    .first()
                    .map(|value| Self::number_value(Self::value_to_i64(value) as f64))
                    .unwrap_or(Value::Undefined);
                Ok(Some(Self::new_object_value(vec![
                    (
                        INTERNAL_CANVAS_KEY_PREFIX.to_string(),
                        Value::String("canvas_capture_stream".to_string()),
                    ),
                    ("active".to_string(), Value::Bool(true)),
                    ("canvas".to_string(), Value::Node(node)),
                    ("frameRate".to_string(), frame_rate),
                ])))
            }
            "getContext" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "getContext requires one or two arguments".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                let transferred_key =
                    INTERNAL_CANVAS_TRANSFERRED_TO_OFFSCREEN_NODE_EXPANDO_KEY.to_string();
                let transferred_to_offscreen = self
                    .dom_runtime
                    .node_expando_props
                    .get(&(node, transferred_key))
                    .is_some_and(|value| value.truthy());
                if transferred_to_offscreen {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Failed to execute 'getContext': canvas has transferred control to offscreen"
                            .into(),
                    ));
                }
                let context_kind = evaluated_args[0].as_string().to_ascii_lowercase();
                let is_known_context = matches!(
                    context_kind.as_str(),
                    "2d" | "webgl" | "experimental-webgl" | "webgl2" | "webgpu" | "bitmaprenderer"
                );
                if let Some(Value::String(existing_mode)) =
                    self.dom_runtime.node_expando_props.get(&(
                        node,
                        INTERNAL_CANVAS_CONTEXT_MODE_NODE_EXPANDO_KEY.to_string(),
                    ))
                {
                    if existing_mode != &context_kind {
                        return Ok(Some(Value::Null));
                    }
                }
                if context_kind != "2d" {
                    return Ok(Some(Value::Null));
                }
                let key = INTERNAL_CANVAS_2D_CONTEXT_NODE_EXPANDO_KEY.to_string();
                if let Some(existing) = self
                    .dom_runtime
                    .node_expando_props
                    .get(&(node, key.clone()))
                {
                    return Ok(Some(existing.clone()));
                }
                let alpha = evaluated_args
                    .get(1)
                    .map(Self::canvas_2d_alpha_from_options)
                    .unwrap_or(true);
                let context = self.new_canvas_2d_context_value(node, alpha);
                self.dom_runtime
                    .node_expando_props
                    .insert((node, key), context.clone());
                if is_known_context {
                    self.dom_runtime.node_expando_props.insert(
                        (
                            node,
                            INTERNAL_CANVAS_CONTEXT_MODE_NODE_EXPANDO_KEY.to_string(),
                        ),
                        Value::String(context_kind),
                    );
                }
                Ok(Some(context))
            }
            "transferControlToOffscreen" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "transferControlToOffscreen takes no arguments".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                if self.dom_runtime.node_expando_props.contains_key(&(
                    node,
                    INTERNAL_CANVAS_CONTEXT_MODE_NODE_EXPANDO_KEY.to_string(),
                )) {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Failed to execute 'transferControlToOffscreen': canvas has an existing rendering context"
                            .into(),
                    ));
                }
                if self.dom_runtime.node_expando_props.contains_key(&(
                    node,
                    INTERNAL_CANVAS_TRANSFERRED_TO_OFFSCREEN_NODE_EXPANDO_KEY.to_string(),
                )) {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Failed to execute 'transferControlToOffscreen': canvas has already transferred control to offscreen"
                            .into(),
                    ));
                }
                self.dom_runtime.node_expando_props.insert(
                    (
                        node,
                        INTERNAL_CANVAS_TRANSFERRED_TO_OFFSCREEN_NODE_EXPANDO_KEY.to_string(),
                    ),
                    Value::Bool(true),
                );
                Ok(Some(Self::new_object_value(vec![
                    (
                        INTERNAL_CANVAS_KEY_PREFIX.to_string(),
                        Value::String("offscreen_canvas".to_string()),
                    ),
                    (
                        "width".to_string(),
                        Value::Number(self.canvas_dimension_value(node, "width")),
                    ),
                    (
                        "height".to_string(),
                        Value::Number(self.canvas_dimension_value(node, "height")),
                    ),
                ])))
            }
            "toDataURL" => {
                if evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "toDataURL supports at most two arguments".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                let mime = evaluated_args
                    .first()
                    .map(Value::as_string)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "image/png".to_string());
                let mime = if mime.eq_ignore_ascii_case("image/png")
                    || mime.eq_ignore_ascii_case("image/jpeg")
                    || mime.eq_ignore_ascii_case("image/webp")
                {
                    mime.to_ascii_lowercase()
                } else {
                    "image/png".to_string()
                };
                let payload = match mime.as_str() {
                    "image/jpeg" => "/9j/4AAQSkZJRgABAQAAAQABAAD/2w==",
                    "image/webp" => "UklGRhIAAABXRUJQVlA4TA0AAAAvAAAAAA==",
                    _ => {
                        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR4nGNgYAAAAAMAASsJTYQAAAAASUVORK5CYII="
                    }
                };
                Ok(Some(Value::String(format!("data:{mime};base64,{payload}"))))
            }
            "toBlob" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 3 {
                    return Err(Error::ScriptRuntime(
                        "toBlob requires one to three arguments".into(),
                    ));
                }
                let is_canvas = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("canvas"));
                if !is_canvas {
                    return Ok(None);
                }
                let callback = evaluated_args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime(
                        "toBlob callback must be callable".into(),
                    ));
                }
                let mime = evaluated_args
                    .get(1)
                    .map(Value::as_string)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "image/png".to_string());
                let mime = if mime.eq_ignore_ascii_case("image/png")
                    || mime.eq_ignore_ascii_case("image/jpeg")
                    || mime.eq_ignore_ascii_case("image/webp")
                {
                    mime.to_ascii_lowercase()
                } else {
                    "image/png".to_string()
                };
                let bytes = match mime.as_str() {
                    "image/jpeg" => vec![0xFF, 0xD8, 0xFF, 0xD9],
                    "image/webp" => b"RIFFWEBP".to_vec(),
                    _ => vec![0x89, b'P', b'N', b'G'],
                };
                let blob = Self::new_blob_value(bytes, mime);
                self.execute_callback_value(&callback, &[blob], event)?;
                Ok(Some(Value::Undefined))
            }
            "getElementsByClassName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByClassName requires exactly one argument".into(),
                    ));
                }
                let class_names = Self::class_names_from_argument(&evaluated_args[0]);
                Ok(Some(self.class_names_live_list_value(node, class_names)))
            }
            "getElementsByTagName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(self.tag_name_live_list_value(
                    node,
                    Self::tag_name_from_argument(&evaluated_args[0]),
                )))
            }
            "getElementsByTagNameNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagNameNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string();
                Ok(Some(self.tag_name_ns_live_list_value(
                    node,
                    namespace_uri,
                    local_name,
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
                let is_form = self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("form"));
                if is_form {
                    return Ok(Some(Value::Bool(self.validate_form_submission(node)?)));
                }
                let validity = self.compute_input_validity(node)?;
                if !validity.valid {
                    let _ = self.dispatch_invalid_event(node)?;
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
            "getAnimations" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "getAnimations supports zero or one options argument".into(),
                    ));
                }
                let subtree = Self::get_animations_subtree_option(evaluated_args.first());
                Ok(Some(self.node_get_animations_value(node, subtree)))
            }
            "animate" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "animate requires one or two arguments".into(),
                    ));
                }
                let options_arg = evaluated_args.get(1);
                let id = Self::animate_id_from_options(options_arg);
                let timeline =
                    Self::animate_option_entry(options_arg, "timeline").unwrap_or(Value::Null);
                let range_start = Self::animate_option_entry(options_arg, "rangeStart")
                    .unwrap_or(Value::String("normal".to_string()));
                let range_end = Self::animate_option_entry(options_arg, "rangeEnd")
                    .unwrap_or(Value::String("normal".to_string()));
                let keyframes = evaluated_args[0].clone();
                let options = options_arg.cloned().unwrap_or(Value::Undefined);
                let animation = Self::new_animation_object_value(
                    id,
                    keyframes,
                    options,
                    timeline,
                    range_start,
                    range_end,
                );
                self.register_node_animation(node, &animation);
                Ok(Some(animation))
            }
            "scrollIntoView" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "scrollIntoView supports zero or one argument".into(),
                    ));
                }
                self.dispatch_document_scroll_sequence(true)?;
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
                let position_changed = self.apply_document_scroll_operation(member, evaluated_args);
                self.sync_window_runtime_properties();
                self.dispatch_document_scroll_sequence(position_changed)?;
                Ok(Some(Value::Undefined))
            }
            "select" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("select takes no arguments".into()));
                }
                if self.node_supports_text_selection(node) {
                    self.focus_node(node)?;
                    let len = self.dom.value(node)?.chars().count();
                    self.set_node_selection_range(node, 0, len as i64, "none".to_string())?;
                }
                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }
}
