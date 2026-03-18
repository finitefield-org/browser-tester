use super::*;

impl Harness {
    fn execute_dom_assign_stmt(
        &mut self,
        target: &DomQuery,
        prop: &DomProp,
        op: &VarAssignOp,
        expr: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        if matches!(
            op,
            VarAssignOp::LogicalAnd | VarAssignOp::LogicalOr | VarAssignOp::Nullish
        ) {
            let previous = self.eval_expr(
                &Expr::DomRead {
                    target: target.clone(),
                    prop: prop.clone(),
                },
                env,
                event_param,
                event,
            )?;
            let should_assign = match op {
                VarAssignOp::LogicalAnd => previous.truthy(),
                VarAssignOp::LogicalOr => !previous.truthy(),
                VarAssignOp::Nullish => {
                    matches!(&previous, Value::Null | Value::Undefined)
                }
                _ => true,
            };
            if !should_assign {
                return Ok(());
            }
        }

        let value = self.eval_expr(expr, env, event_param, event)?;
        if let Some(key) = Self::object_key_from_dom_prop(prop) {
            let receiver = match target {
                DomQuery::Var(name) => env.get(name).cloned(),
                DomQuery::VarPath { .. } | DomQuery::Index { .. } => {
                    self.resolve_dom_query_value_runtime(target, env)?
                }
                _ => None,
            };
            if let Some(receiver) = receiver {
                if !matches!(receiver, Value::Node(_) | Value::NodeList(_)) {
                    let assignment_target = match target {
                        DomQuery::Var(name) => name.clone(),
                        _ => target.describe_call(),
                    };
                    let mut assignment_value = value.clone();
                    let is_event_param_target = match target {
                        DomQuery::Var(name) => {
                            event_param.as_ref().is_some_and(|param| param == name)
                        }
                        _ => false,
                    };
                    if key == "returnValue"
                        && is_event_param_target
                        && (event.before_unload_interface
                            || event.event_type.eq_ignore_ascii_case("beforeunload"))
                    {
                        let return_value = assignment_value.as_string();
                        event.before_unload_interface = true;
                        event.before_unload_return_value = return_value.clone();
                        if event.cancelable && !return_value.is_empty() {
                            event.default_prevented = true;
                        }
                        assignment_value = Value::String(return_value);
                    }
                    self.set_object_assignment_property(
                        &receiver,
                        &Value::String(key.to_string()),
                        assignment_value,
                        &assignment_target,
                        env,
                        event,
                    )?;
                    return Ok(());
                }
            }
        }
        let node = self.resolve_dom_query_required_runtime(target, env)?;
        match prop {
            DomProp::TextContent => self.dom.set_text_content(node, &value.as_string())?,
            DomProp::InnerText => self.dom.set_text_content(node, &value.as_string())?,
            DomProp::InnerHtml => {
                let html = if matches!(value, Value::Null) {
                    String::new()
                } else {
                    value.as_string()
                };
                self.dom.set_inner_html(node, &html)?
            }
            DomProp::OuterHtml => {
                let html = if matches!(value, Value::Null) {
                    String::new()
                } else {
                    value.as_string()
                };
                self.dom.set_outer_html(node, &html)?
            }
            DomProp::Value => {
                if self.node_explicit_own_property_overrides_dom_property(node, "value") {
                    self.set_node_assignment_property(node, "value", value, event, false)?;
                } else if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("li"))
                {
                    let next = Self::value_to_i64(&value);
                    self.dom.set_attr(node, "value", &next.to_string())?;
                } else {
                    self.dom.set_value(node, &value.as_string())?;
                }
            }
            DomProp::ValueAsNumber => self.set_input_value_as_number(
                node,
                Self::coerce_number_for_number_constructor(&value),
            )?,
            DomProp::ValueAsDate => {
                let timestamp_ms = match value {
                    Value::Date(timestamp) => Some(*timestamp.borrow()),
                    Value::Null | Value::Undefined => None,
                    _ => None,
                };
                self.set_input_value_as_date_ms(node, timestamp_ms)?;
            }
            DomProp::SelectionStart => {
                let next_start = Self::value_to_i64(&value).max(0) as usize;
                let end = self.dom.selection_end(node).unwrap_or_default();
                self.set_node_selection_range(
                    node,
                    next_start as i64,
                    end as i64,
                    "none".to_string(),
                )?;
            }
            DomProp::SelectionEnd => {
                let start = self.dom.selection_start(node).unwrap_or_default();
                let next_end = Self::value_to_i64(&value).max(0) as usize;
                self.set_node_selection_range(
                    node,
                    start as i64,
                    next_end as i64,
                    "none".to_string(),
                )?;
            }
            DomProp::SelectionDirection => {
                let start = self.dom.selection_start(node).unwrap_or_default();
                let end = self.dom.selection_end(node).unwrap_or_default();
                let direction = value.as_string();
                let direction = Self::normalize_selection_direction(direction.as_str());
                self.set_node_selection_range(
                    node,
                    start as i64,
                    end as i64,
                    direction.to_string(),
                )?;
            }
            DomProp::Checked => self.dom.set_checked(node, value.truthy())?,
            DomProp::Indeterminate => self.dom.set_indeterminate(node, value.truthy())?,
            DomProp::Open => {
                if self.node_explicit_own_property_overrides_dom_property(node, "open") {
                    self.set_node_assignment_property(node, "open", value, event, false)?;
                } else if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
                {
                    let _ = self.set_details_open_state_with_env(node, value.truthy(), env)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "open", value.truthy())?;
                }
            }
            DomProp::ReturnValue => {
                self.set_dialog_return_value(node, value.as_string())?;
            }
            DomProp::ClosedBy => {
                if let Some(shadow_key) =
                    self.node_explicit_own_dom_property_shadow_key(node, &["closedBy", "closedby"])
                {
                    self.set_node_assignment_property(node, shadow_key, value, event, false)?;
                } else {
                    self.dom.set_attr(node, "closedby", &value.as_string())?
                }
            }
            DomProp::Readonly => {
                self.set_reflected_boolean_attribute(node, "readonly", value.truthy())?
            }
            DomProp::Required => {
                self.set_reflected_boolean_attribute(node, "required", value.truthy())?
            }
            DomProp::Disabled => {
                self.set_reflected_boolean_attribute(node, "disabled", value.truthy())?
            }
            DomProp::Hidden => {
                if node == self.dom.root {
                    let call = self.describe_dom_prop(prop);
                    return Err(Error::ScriptRuntime(format!("{call} is read-only")));
                }
                if self.node_explicit_own_property_overrides_dom_property(node, "hidden") {
                    self.set_node_assignment_property(node, "hidden", value, event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "hidden", value.truthy())?
                }
            }
            DomProp::ClassName => {
                if self.node_explicit_own_property_overrides_dom_property(node, "className") {
                    self.set_node_assignment_property(node, "className", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "class", &value.as_string())?
                }
            }
            DomProp::ClassList => self.dom.set_attr(node, "class", &value.as_string())?,
            DomProp::Part => self.dom.set_attr(node, "part", &value.as_string())?,
            DomProp::Id => {
                if self.node_explicit_own_property_overrides_dom_property(node, "id") {
                    self.set_node_assignment_property(node, "id", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "id", &value.as_string())?
                }
            }
            DomProp::Slot => {
                if self.node_explicit_own_property_overrides_dom_property(node, "slot") {
                    self.set_node_assignment_property(node, "slot", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "slot", &value.as_string())?
                }
            }
            DomProp::Role => {
                if self.node_explicit_own_property_overrides_dom_property(node, "role") {
                    self.set_node_assignment_property(node, "role", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "role", &value.as_string())?
                }
            }
            DomProp::ElementTiming => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["elementTiming", "elementtiming"],
                ) {
                    self.set_node_assignment_property(node, shadow_key, value, event, false)?;
                } else {
                    self.dom
                        .set_attr(node, "elementtiming", &value.as_string())?
                }
            }
            DomProp::HtmlFor => {
                if self.node_explicit_own_property_overrides_dom_property(node, "htmlFor") {
                    self.set_node_assignment_property(node, "htmlFor", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "for", &value.as_string())?
                }
            }
            DomProp::Name => {
                if self.node_explicit_own_property_overrides_dom_property(node, "name") {
                    self.set_node_assignment_property(node, "name", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "name", &value.as_string())?
                }
            }
            DomProp::Action => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("form"))
                {
                    self.dom.set_attr(node, "action", &value.as_string())?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "action".to_string()), value);
                }
            }
            DomProp::FormAction => {
                if self.node_explicit_own_property_overrides_dom_property(node, "formAction") {
                    self.set_node_assignment_property(node, "formAction", value, event, false)?;
                } else if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("button") || tag.eq_ignore_ascii_case("input")
                }) {
                    self.dom.set_attr(node, "formaction", &value.as_string())?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "formAction".to_string()), value);
                }
            }
            DomProp::Lang => {
                if self.node_explicit_own_property_overrides_dom_property(node, "lang") {
                    self.set_node_assignment_property(node, "lang", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "lang", &value.as_string())?
                }
            }
            DomProp::Dir => {
                if self.node_explicit_own_property_overrides_dom_property(node, "dir") {
                    self.set_node_assignment_property(node, "dir", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "dir", &value.as_string())?
                }
            }
            DomProp::AccessKey => {
                if self.node_explicit_own_property_overrides_dom_property(node, "accessKey") {
                    self.set_node_assignment_property(node, "accessKey", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "accesskey", &value.as_string())?
                }
            }
            DomProp::AutoComplete => self
                .dom
                .set_attr(node, "autocomplete", &value.as_string())?,
            DomProp::AutoCapitalize => {
                if self.node_explicit_own_property_overrides_dom_property(node, "autocapitalize") {
                    self.set_node_assignment_property(node, "autocapitalize", value, event, false)?;
                } else {
                    self.dom
                        .set_attr(node, "autocapitalize", &value.as_string())?
                }
            }
            DomProp::AutoCorrect => {
                if self.node_explicit_own_property_overrides_dom_property(node, "autocorrect") {
                    self.set_node_assignment_property(node, "autocorrect", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "autocorrect", &value.as_string())?
                }
            }
            DomProp::ContentEditable => {
                if self.node_explicit_own_property_overrides_dom_property(node, "contentEditable") {
                    self.set_node_assignment_property(
                        node,
                        "contentEditable",
                        value,
                        event,
                        false,
                    )?;
                } else {
                    self.set_content_editable_property_value(node, &value)?
                }
            }
            DomProp::Draggable => {
                if self.node_explicit_own_property_overrides_dom_property(node, "draggable") {
                    self.set_node_assignment_property(node, "draggable", value, event, false)?;
                } else {
                    self.set_reflected_keyword_boolean_attribute(
                        node,
                        "draggable",
                        value.truthy(),
                        "true",
                        "false",
                    )?
                }
            }
            DomProp::EnterKeyHint => {
                if self.node_explicit_own_property_overrides_dom_property(node, "enterKeyHint") {
                    self.set_node_assignment_property(node, "enterKeyHint", value, event, false)?;
                } else {
                    self.dom
                        .set_attr(node, "enterkeyhint", &value.as_string())?
                }
            }
            DomProp::Inert => {
                if self.node_explicit_own_property_overrides_dom_property(node, "inert") {
                    self.set_node_assignment_property(node, "inert", value, event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "inert", value.truthy())?
                }
            }
            DomProp::InputMode => {
                if self.node_explicit_own_property_overrides_dom_property(node, "inputMode") {
                    self.set_node_assignment_property(node, "inputMode", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "inputmode", &value.as_string())?
                }
            }
            DomProp::Nonce => {
                if self.node_explicit_own_property_overrides_dom_property(node, "nonce") {
                    self.set_node_assignment_property(node, "nonce", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "nonce", &value.as_string())?
                }
            }
            DomProp::Popover => {
                if self.node_explicit_own_property_overrides_dom_property(node, "popover") {
                    self.set_node_assignment_property(node, "popover", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "popover", &value.as_string())?
                }
            }
            DomProp::Spellcheck => {
                if self.node_explicit_own_property_overrides_dom_property(node, "spellcheck") {
                    self.set_node_assignment_property(node, "spellcheck", value, event, false)?;
                } else {
                    self.set_reflected_keyword_boolean_attribute(
                        node,
                        "spellcheck",
                        value.truthy(),
                        "true",
                        "false",
                    )?
                }
            }
            DomProp::TabIndex => {
                if self.node_explicit_own_property_overrides_dom_property(node, "tabIndex") {
                    self.set_node_assignment_property(node, "tabIndex", value, event, false)?;
                } else {
                    self.set_reflected_i64_attribute(node, "tabindex", &value)?
                }
            }
            DomProp::Translate => {
                if self.node_explicit_own_property_overrides_dom_property(node, "translate") {
                    self.set_node_assignment_property(node, "translate", value, event, false)?;
                } else {
                    self.set_reflected_keyword_boolean_attribute(
                        node,
                        "translate",
                        value.truthy(),
                        "yes",
                        "no",
                    )?
                }
            }
            DomProp::Cite => {
                if self.node_explicit_own_property_overrides_dom_property(node, "cite") {
                    self.set_node_assignment_property(node, "cite", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "cite", &value.as_string())?
                }
            }
            DomProp::DateTime => {
                if let Some(shadow_key) =
                    self.node_explicit_own_dom_property_shadow_key(node, &["dateTime", "datetime"])
                {
                    self.set_node_assignment_property(node, shadow_key, value, event, false)?;
                } else {
                    self.dom.set_attr(node, "datetime", &value.as_string())?
                }
            }
            DomProp::BrClear => {
                if self.node_explicit_own_property_overrides_dom_property(node, "clear") {
                    self.set_node_assignment_property(node, "clear", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "clear", &value.as_string())?
                }
            }
            DomProp::CaptionAlign => {
                if self.node_explicit_own_property_overrides_dom_property(node, "align") {
                    self.set_node_assignment_property(node, "align", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "align", &value.as_string())?
                }
            }
            DomProp::ColSpan => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("col") || tag.eq_ignore_ascii_case("colgroup")
                }) {
                    self.set_col_span_value(node, &value)?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "span".to_string()), value);
                }
            }
            DomProp::TableCellColSpan => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th")
                }) {
                    self.set_table_cell_col_span_value(node, &value)?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "colSpan".to_string()), value);
                }
            }
            DomProp::RowSpan => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th")
                }) {
                    self.set_table_cell_row_span_value(node, &value)?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "rowSpan".to_string()), value);
                }
            }
            DomProp::CanvasWidth => self.set_canvas_dimension_value(node, "width", &value)?,
            DomProp::CanvasHeight => self.set_canvas_dimension_value(node, "height", &value)?,
            DomProp::NodeEventHandler(event_name) => {
                let _ = self.set_node_event_handler_property(node, event_name, value.clone())?;
            }
            DomProp::BodyDeprecatedAttr(attr_name) => {
                self.dom.set_attr(node, attr_name, &value.as_string())?
            }
            DomProp::Title => self.dom.set_document_title(&value.as_string())?,
            DomProp::AdoptedStyleSheets => {
                self.set_document_adopted_style_sheets_property(value)?;
            }
            DomProp::AudioSrc => {
                if self.node_explicit_own_property_overrides_dom_property(node, "src") {
                    self.set_node_assignment_property(node, "src", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "src", &value.as_string())?
                }
            }
            DomProp::AudioAutoplay => {
                if self.node_explicit_own_property_overrides_dom_property(node, "autoplay") {
                    self.set_node_assignment_property(node, "autoplay", value, event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "autoplay", value.truthy())?
                }
            }
            DomProp::AudioControls => {
                if self.node_explicit_own_property_overrides_dom_property(node, "controls") {
                    self.set_node_assignment_property(node, "controls", value, event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "controls", value.truthy())?
                }
            }
            DomProp::AudioControlsList => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["controlsList", "controlslist"],
                ) {
                    self.set_node_assignment_property(node, shadow_key, value, event, false)?;
                } else {
                    self.dom
                        .set_attr(node, "controlslist", &value.as_string())?
                }
            }
            DomProp::AudioCrossOrigin => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["crossOrigin", "crossorigin"],
                ) {
                    self.set_node_assignment_property(node, shadow_key, value, event, false)?;
                } else {
                    self.dom.set_attr(node, "crossorigin", &value.as_string())?
                }
            }
            DomProp::AudioDisableRemotePlayback => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["disableRemotePlayback", "disableremoteplayback"],
                ) {
                    self.set_node_assignment_property(node, shadow_key, value, event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(
                        node,
                        "disableremoteplayback",
                        value.truthy(),
                    )?
                }
            }
            DomProp::VideoDisablePictureInPicture => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["disablePictureInPicture", "disablepictureinpicture"],
                ) {
                    self.set_node_assignment_property(node, shadow_key, value, event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(
                        node,
                        "disablepictureinpicture",
                        value.truthy(),
                    )?
                }
            }
            DomProp::AudioLoop => {
                if self.node_explicit_own_property_overrides_dom_property(node, "loop") {
                    self.set_node_assignment_property(node, "loop", value, event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "loop", value.truthy())?
                }
            }
            DomProp::AudioMuted => {
                if self.node_explicit_own_property_overrides_dom_property(node, "muted") {
                    self.set_node_assignment_property(node, "muted", value, event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "muted", value.truthy())?
                }
            }
            DomProp::AudioPreload => {
                if self.node_explicit_own_property_overrides_dom_property(node, "preload") {
                    self.set_node_assignment_property(node, "preload", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "preload", &value.as_string())?
                }
            }
            DomProp::VideoPlaysInline => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["playsInline", "playsinline"],
                ) {
                    self.set_node_assignment_property(node, shadow_key, value, event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "playsinline", value.truthy())?
                }
            }
            DomProp::VideoPoster => {
                if self.node_explicit_own_property_overrides_dom_property(node, "poster") {
                    self.set_node_assignment_property(node, "poster", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "poster", &value.as_string())?
                }
            }
            DomProp::Data => {
                if self.node_explicit_own_property_overrides_dom_property(node, "data") {
                    self.set_node_assignment_property(node, "data", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "data", &value.as_string())?
                }
            }
            DomProp::SrcDoc => {
                if self.node_explicit_own_property_overrides_dom_property(node, "srcdoc") {
                    self.set_node_assignment_property(node, "srcdoc", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "srcdoc", &value.as_string())?
                }
            }
            DomProp::UseMap => {
                if self.node_explicit_own_property_overrides_dom_property(node, "useMap") {
                    self.set_node_assignment_property(node, "useMap", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "usemap", &value.as_string())?
                }
            }
            DomProp::Location | DomProp::LocationHref => {
                self.navigate_location(&value.as_string(), LocationNavigationKind::HrefSet)?
            }
            DomProp::LocationProtocol => self.set_location_property("protocol", value.clone())?,
            DomProp::LocationHost => self.set_location_property("host", value.clone())?,
            DomProp::LocationHostname => self.set_location_property("hostname", value.clone())?,
            DomProp::LocationPort => self.set_location_property("port", value.clone())?,
            DomProp::LocationPathname => self.set_location_property("pathname", value.clone())?,
            DomProp::LocationSearch => self.set_location_property("search", value.clone())?,
            DomProp::LocationHash => self.set_location_property("hash", value.clone())?,
            DomProp::HistoryScrollRestoration => {
                self.set_history_property("scrollRestoration", value.clone())?
            }
            DomProp::AnchorAlt => {
                if self.node_explicit_own_property_overrides_dom_property(node, "alt") {
                    self.set_node_assignment_property(node, "alt", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "alt", &value.as_string())?
                }
            }
            DomProp::AnchorAttributionSrc => {
                if self.node_explicit_own_property_overrides_dom_property(node, "attributionSrc") {
                    self.set_node_assignment_property(node, "attributionSrc", value, event, false)?;
                } else {
                    self.dom
                        .set_attr(node, "attributionsrc", &value.as_string())?
                }
            }
            DomProp::AnchorDownload => {
                if self.node_explicit_own_property_overrides_dom_property(node, "download") {
                    self.set_node_assignment_property(node, "download", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "download", &value.as_string())?
                }
            }
            DomProp::AnchorHash => self.set_anchor_url_property(node, "hash", value.clone())?,
            DomProp::AnchorHost => self.set_anchor_url_property(node, "host", value.clone())?,
            DomProp::AnchorHostname => {
                self.set_anchor_url_property(node, "hostname", value.clone())?
            }
            DomProp::AnchorHref => {
                if self.node_explicit_own_property_overrides_dom_property(node, "href") {
                    self.set_node_assignment_property(node, "href", value, event, false)?;
                } else {
                    self.set_anchor_url_property(node, "href", value.clone())?
                }
            }
            DomProp::AnchorHreflang => {
                if self.node_explicit_own_property_overrides_dom_property(node, "hreflang") {
                    self.set_node_assignment_property(node, "hreflang", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "hreflang", &value.as_string())?
                }
            }
            DomProp::AnchorInterestForElement => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("button"))
                {
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
                        _ => {
                            self.dom.set_attr(node, "interestfor", &value.as_string())?;
                        }
                    }
                } else {
                    self.dom.set_attr(node, "interestfor", &value.as_string())?
                }
            }
            DomProp::AnchorPassword => {
                self.set_anchor_url_property(node, "password", value.clone())?
            }
            DomProp::AnchorPathname => {
                self.set_anchor_url_property(node, "pathname", value.clone())?
            }
            DomProp::AnchorPing => {
                if self.node_explicit_own_property_overrides_dom_property(node, "ping") {
                    self.set_node_assignment_property(node, "ping", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "ping", &value.as_string())?
                }
            }
            DomProp::AnchorPort => self.set_anchor_url_property(node, "port", value.clone())?,
            DomProp::AnchorProtocol => {
                self.set_anchor_url_property(node, "protocol", value.clone())?
            }
            DomProp::AnchorReferrerPolicy => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["referrerPolicy", "referrerpolicy"],
                ) {
                    self.set_node_assignment_property(node, shadow_key, value, event, false)?;
                } else {
                    self.dom
                        .set_attr(node, "referrerpolicy", &value.as_string())?
                }
            }
            DomProp::AnchorRel => {
                if self.node_explicit_own_property_overrides_dom_property(node, "rel") {
                    self.set_node_assignment_property(node, "rel", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "rel", &value.as_string())?
                }
            }
            DomProp::AnchorSearch => self.set_anchor_url_property(node, "search", value.clone())?,
            DomProp::AnchorTarget => {
                if self.node_explicit_own_property_overrides_dom_property(node, "target") {
                    self.set_node_assignment_property(node, "target", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "target", &value.as_string())?
                }
            }
            DomProp::AnchorText => self.dom.set_text_content(node, &value.as_string())?,
            DomProp::AnchorType => {
                if self.node_explicit_own_property_overrides_dom_property(node, "type") {
                    self.set_node_assignment_property(node, "type", value, event, false)?;
                } else if !self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("select"))
                {
                    self.dom.set_attr(node, "type", &value.as_string())?
                }
            }
            DomProp::AnchorUsername => {
                self.set_anchor_url_property(node, "username", value.clone())?
            }
            DomProp::AnchorNoHref => {
                if let Some(shadow_key) =
                    self.node_explicit_own_dom_property_shadow_key(node, &["noHref", "nohref"])
                {
                    self.set_node_assignment_property(node, shadow_key, value, event, false)?;
                } else if value.truthy() {
                    self.dom.set_attr(node, "nohref", "true")?;
                } else {
                    self.dom.remove_attr(node, "nohref")?;
                }
            }
            DomProp::AnchorCharset => {
                if self.node_explicit_own_property_overrides_dom_property(node, "charset") {
                    self.set_node_assignment_property(node, "charset", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "charset", &value.as_string())?
                }
            }
            DomProp::AnchorCoords => {
                if self.node_explicit_own_property_overrides_dom_property(node, "coords") {
                    self.set_node_assignment_property(node, "coords", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "coords", &value.as_string())?
                }
            }
            DomProp::AnchorRev => {
                if self.node_explicit_own_property_overrides_dom_property(node, "rev") {
                    self.set_node_assignment_property(node, "rev", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "rev", &value.as_string())?
                }
            }
            DomProp::AnchorShape => {
                if self.node_explicit_own_property_overrides_dom_property(node, "shape") {
                    self.set_node_assignment_property(node, "shape", value, event, false)?;
                } else {
                    self.dom.set_attr(node, "shape", &value.as_string())?
                }
            }
            DomProp::Size => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("select"))
                {
                    self.set_select_size_property_value(node, &value)?
                } else if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    self.set_input_size_property_value(node, &value)?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "size".to_string()), value);
                }
            }
            DomProp::Min => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    self.dom.set_attr(node, "min", &value.as_string())?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "min".to_string()), value);
                }
            }
            DomProp::Max => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    self.dom.set_attr(node, "max", &value.as_string())?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "max".to_string()), value);
                }
            }
            DomProp::Step => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    self.dom.set_attr(node, "step", &value.as_string())?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "step".to_string()), value);
                }
            }
            DomProp::MaxLength => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("input") || tag.eq_ignore_ascii_case("textarea")
                }) {
                    self.set_max_length_property_value(node, &value)?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "maxLength".to_string()), value);
                }
            }
            DomProp::MinLength => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("input") || tag.eq_ignore_ascii_case("textarea")
                }) {
                    self.set_min_length_property_value(node, &value)?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "minLength".to_string()), value);
                }
            }
            DomProp::Rows => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("textarea"))
                {
                    self.set_textarea_rows_property_value(node, &value)?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "rows".to_string()), value);
                }
            }
            DomProp::Cols => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("textarea"))
                {
                    self.set_textarea_cols_property_value(node, &value)?
                } else {
                    self.dom_runtime
                        .node_expando_props
                        .insert((node, "cols".to_string()), value);
                }
            }
            DomProp::AriaString(prop_name) => {
                let attr_name = Self::aria_property_to_attr_name(prop_name);
                self.dom.set_attr(node, &attr_name, &value.as_string())?
            }
            DomProp::Files => {
                let files = self.mock_files_from_input_assignment_value(&value)?;
                self.dom.set_file_input_files(node, &files)?;
            }
            DomProp::Attributes
            | DomProp::AssignedSlot
            | DomProp::FilesLength
            | DomProp::ValidationMessage
            | DomProp::Validity
            | DomProp::ValidityValueMissing
            | DomProp::ValidityTypeMismatch
            | DomProp::ValidityPatternMismatch
            | DomProp::ValidityTooLong
            | DomProp::ValidityTooShort
            | DomProp::ValidityRangeUnderflow
            | DomProp::ValidityRangeOverflow
            | DomProp::ValidityStepMismatch
            | DomProp::ValidityBadInput
            | DomProp::ValidityValid
            | DomProp::ValidityCustomError
            | DomProp::NodeType
            | DomProp::ClassListLength
            | DomProp::PartLength
            | DomProp::TagName
            | DomProp::LocalName
            | DomProp::NamespaceUri
            | DomProp::Prefix
            | DomProp::NextElementSibling
            | DomProp::PreviousElementSibling
            | DomProp::ClientWidth
            | DomProp::ClientHeight
            | DomProp::ClientLeft
            | DomProp::ClientTop
            | DomProp::CurrentCssZoom
            | DomProp::ScrollLeftMax
            | DomProp::ScrollTopMax
            | DomProp::ShadowRoot
            | DomProp::AriaElementRefSingle(_)
            | DomProp::AriaElementRefList(_)
            | DomProp::OffsetWidth
            | DomProp::ValueLength
            | DomProp::OffsetHeight
            | DomProp::OffsetLeft
            | DomProp::OffsetTop
            | DomProp::ScrollWidth
            | DomProp::ScrollHeight
            | DomProp::ScrollLeft
            | DomProp::ScrollTop
            | DomProp::ActiveElement
            | DomProp::ActiveViewTransition
            | DomProp::AdoptedStyleSheetsLength
            | DomProp::CharacterSet
            | DomProp::CompatMode
            | DomProp::ContentType
            | DomProp::ReadyState
            | DomProp::Referrer
            | DomProp::Url
            | DomProp::DocumentUri
            | DomProp::BaseUri
            | DomProp::LocationOrigin
            | DomProp::LocationAncestorOrigins
            | DomProp::History
            | DomProp::HistoryLength
            | DomProp::HistoryState
            | DomProp::DefaultView
            | DomProp::VisibilityState
            | DomProp::Forms
            | DomProp::Images
            | DomProp::Links
            | DomProp::Scripts
            | DomProp::Children
            | DomProp::ChildElementCount
            | DomProp::FirstElementChild
            | DomProp::LastElementChild
            | DomProp::CurrentScript
            | DomProp::FormsLength
            | DomProp::ImagesLength
            | DomProp::LinksLength
            | DomProp::ScriptsLength
            | DomProp::ChildrenLength
            | DomProp::AnchorOrigin
            | DomProp::AnchorRelList
            | DomProp::AnchorRelListLength => {
                let call = self.describe_dom_prop(prop);
                return Err(Error::ScriptRuntime(format!("{call} is read-only")));
            }
            DomProp::Dataset(key) => self.dom.dataset_set(node, key, &value.as_string())?,
            DomProp::Style(prop) => self.dom.style_set(node, prop, &value.as_string())?,
        }
        Ok(())
    }

    pub(crate) fn try_execute_dom_assign_stmt(
        &mut self,
        stmt: &Stmt,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<Option<ExecFlow>> {
        let Stmt::DomAssign {
            target,
            prop,
            op,
            expr,
        } = stmt
        else {
            return Ok(None);
        };

        self.execute_dom_assign_stmt(target, prop, op, expr, env, event_param, event)?;
        Ok(Some(ExecFlow::Continue))
    }
}
