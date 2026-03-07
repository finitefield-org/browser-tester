use super::*;

impl Harness {
    fn dom_prop_non_node_fallback_path(prop: &DomProp) -> Option<Vec<&'static str>> {
        match prop {
            DomProp::ValueLength => Some(vec!["value", "length"]),
            DomProp::FilesLength => Some(vec!["files", "length"]),
            DomProp::ClassListLength => Some(vec!["classList", "length"]),
            DomProp::PartLength => Some(vec!["part", "length"]),
            DomProp::AdoptedStyleSheetsLength => Some(vec!["adoptedStyleSheets", "length"]),
            DomProp::HistoryLength => Some(vec!["history", "length"]),
            DomProp::FormsLength => Some(vec!["forms", "length"]),
            DomProp::ImagesLength => Some(vec!["images", "length"]),
            DomProp::LinksLength => Some(vec!["links", "length"]),
            DomProp::ScriptsLength => Some(vec!["scripts", "length"]),
            DomProp::ChildrenLength => Some(vec!["children", "length"]),
            DomProp::AnchorRelListLength => Some(vec!["relList", "length"]),
            _ => Self::object_key_from_dom_prop(prop).map(|key| vec![key]),
        }
    }

    pub(crate) fn eval_expr_dom_and_platform(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::DomRead { target, prop } => {
                    let target_value = self.resolve_dom_query_value_runtime(target, env)?;
                    if let Some(value) = target_value {
                        if !matches!(value, Value::Node(_) | Value::NodeList(_)) {
                            if let Some(path) = Self::dom_prop_non_node_fallback_path(prop) {
                                let variable_name = target.describe_call();
                                let mut current = value;
                                for key in path {
                                    current = self.object_property_from_named_value(
                                        &variable_name,
                                        &current,
                                        key,
                                    )?;
                                }
                                return Ok(current);
                            }
                        }
                    }
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match prop {
                        DomProp::Attributes => {
                            self.dom.element(node).ok_or_else(|| {
                                Error::ScriptRuntime("attributes target is not an element".into())
                            })?;
                            Ok(self.named_node_map_live_value(node))
                        }
                        DomProp::AssignedSlot => Ok(Value::Null),
                        DomProp::Value => {
                            if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("li"))
                            {
                                Ok(Value::Number(self.li_value_property(node)))
                            } else {
                                Ok(Value::String(self.dom.value(node)?))
                            }
                        }
                        DomProp::Files => self.input_files_value(node),
                        DomProp::FilesLength => match self.input_files_value(node)? {
                            Value::Array(values) => Ok(Value::Number(values.borrow().len() as i64)),
                            Value::Null => Ok(Value::Number(0)),
                            _ => Ok(Value::Number(0)),
                        },
                        DomProp::ValueAsNumber => {
                            Ok(Self::number_value(self.input_value_as_number(node)?))
                        }
                        DomProp::ValueAsDate => Ok(self
                            .input_value_as_date_ms(node)?
                            .map(Self::new_date_value)
                            .unwrap_or(Value::Null)),
                        DomProp::ValueLength => {
                            Ok(Value::Number(self.dom.value(node)?.chars().count() as i64))
                        }
                        DomProp::ValidationMessage => {
                            let validity = self.compute_input_validity(node)?;
                            if validity.custom_error {
                                Ok(Value::String(self.dom.custom_validity_message(node)?))
                            } else {
                                Ok(Value::String(String::new()))
                            }
                        }
                        DomProp::Validity => {
                            let validity = self.compute_input_validity(node)?;
                            Ok(Self::input_validity_to_value(&validity))
                        }
                        DomProp::ValidityValueMissing => Ok(Value::Bool(
                            self.compute_input_validity(node)?.value_missing,
                        )),
                        DomProp::ValidityTypeMismatch => Ok(Value::Bool(
                            self.compute_input_validity(node)?.type_mismatch,
                        )),
                        DomProp::ValidityPatternMismatch => Ok(Value::Bool(
                            self.compute_input_validity(node)?.pattern_mismatch,
                        )),
                        DomProp::ValidityTooLong => {
                            Ok(Value::Bool(self.compute_input_validity(node)?.too_long))
                        }
                        DomProp::ValidityTooShort => {
                            Ok(Value::Bool(self.compute_input_validity(node)?.too_short))
                        }
                        DomProp::ValidityRangeUnderflow => Ok(Value::Bool(
                            self.compute_input_validity(node)?.range_underflow,
                        )),
                        DomProp::ValidityRangeOverflow => Ok(Value::Bool(
                            self.compute_input_validity(node)?.range_overflow,
                        )),
                        DomProp::ValidityStepMismatch => Ok(Value::Bool(
                            self.compute_input_validity(node)?.step_mismatch,
                        )),
                        DomProp::ValidityBadInput => {
                            Ok(Value::Bool(self.compute_input_validity(node)?.bad_input))
                        }
                        DomProp::ValidityValid => {
                            Ok(Value::Bool(self.compute_input_validity(node)?.valid))
                        }
                        DomProp::ValidityCustomError => {
                            Ok(Value::Bool(self.compute_input_validity(node)?.custom_error))
                        }
                        DomProp::SelectionStart => Ok(Value::Number(
                            self.dom.selection_start(node).unwrap_or_default() as i64,
                        )),
                        DomProp::SelectionEnd => Ok(Value::Number(
                            self.dom.selection_end(node).unwrap_or_default() as i64,
                        )),
                        DomProp::SelectionDirection => Ok(Value::String(
                            self.dom
                                .selection_direction(node)
                                .unwrap_or_else(|_| "none".to_string()),
                        )),
                        DomProp::Checked => Ok(Value::Bool(self.dom.checked(node)?)),
                        DomProp::Indeterminate => Ok(Value::Bool(self.dom.indeterminate(node)?)),
                        DomProp::Open => Ok(Value::Bool(self.dom.has_attr(node, "open")?)),
                        DomProp::ReturnValue => Ok(Value::String(self.dialog_return_value(node)?)),
                        DomProp::ClosedBy => Ok(Value::String(
                            self.dom.attr(node, "closedby").unwrap_or_default(),
                        )),
                        DomProp::Readonly => Ok(Value::Bool(self.dom.readonly(node))),
                        DomProp::Disabled => Ok(Value::Bool(self.dom.disabled(node))),
                        DomProp::Required => Ok(Value::Bool(self.dom.required(node))),
                        DomProp::NodeType => Ok(Value::Number(self.node_type_number(node))),
                        DomProp::TextContent => Ok(self.node_text_content_value(node)),
                        DomProp::InnerText => Ok(Value::String(self.dom.text_content(node))),
                        DomProp::InnerHtml => Ok(Value::String(self.dom.inner_html(node)?)),
                        DomProp::OuterHtml => Ok(Value::String(self.dom.outer_html(node)?)),
                        DomProp::ClassName => Ok(Value::String(
                            self.dom.attr(node, "class").unwrap_or_default(),
                        )),
                        DomProp::ClassList => Ok(Self::new_class_list_value(node)),
                        DomProp::ClassListLength => Ok(Value::Number(
                            class_tokens(self.dom.attr(node, "class").as_deref()).len() as i64,
                        )),
                        DomProp::Part => Ok(Self::new_array_value(
                            class_tokens(self.dom.attr(node, "part").as_deref())
                                .into_iter()
                                .map(Value::String)
                                .collect::<Vec<_>>(),
                        )),
                        DomProp::PartLength => Ok(Value::Number(
                            class_tokens(self.dom.attr(node, "part").as_deref()).len() as i64,
                        )),
                        DomProp::Id => {
                            Ok(Value::String(self.dom.attr(node, "id").unwrap_or_default()))
                        }
                        DomProp::TagName => Ok(Value::String(self.element_tag_name(node))),
                        DomProp::LocalName => Ok(Value::String(
                            self.dom
                                .tag_name(node)
                                .map(|name| {
                                    name.rsplit_once(':')
                                        .map(|(_, local)| local)
                                        .unwrap_or(name)
                                        .to_ascii_lowercase()
                                })
                                .unwrap_or_default(),
                        )),
                        DomProp::NamespaceUri => Ok(self
                            .dom
                            .element(node)
                            .and_then(|element| element.namespace_uri.clone())
                            .map(Value::String)
                            .unwrap_or(Value::Null)),
                        DomProp::Prefix => Ok(self
                            .dom
                            .tag_name(node)
                            .and_then(|name| name.split_once(':').map(|(prefix, _)| prefix))
                            .map(|prefix| Value::String(prefix.to_string()))
                            .unwrap_or(Value::Null)),
                        DomProp::NextElementSibling => Ok(self
                            .dom
                            .next_element_sibling(node)
                            .map(Value::Node)
                            .unwrap_or(Value::Null)),
                        DomProp::PreviousElementSibling => Ok(self
                            .dom
                            .previous_element_sibling(node)
                            .map(Value::Node)
                            .unwrap_or(Value::Null)),
                        DomProp::Slot => Ok(Value::String(
                            self.dom.attr(node, "slot").unwrap_or_default(),
                        )),
                        DomProp::Role => Ok(Value::String(self.resolved_role_for_node(node))),
                        DomProp::ElementTiming => Ok(Value::String(
                            self.dom.attr(node, "elementtiming").unwrap_or_default(),
                        )),
                        DomProp::HtmlFor => Ok(Value::String(
                            self.dom.attr(node, "for").unwrap_or_default(),
                        )),
                        DomProp::Name => Ok(Value::String(
                            self.dom.attr(node, "name").unwrap_or_default(),
                        )),
                        DomProp::Action => {
                            if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("form"))
                            {
                                Ok(Value::String(
                                    self.form_action_property_value_for_node(node),
                                ))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "action".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::FormAction => {
                            if self.dom.tag_name(node).is_some_and(|tag| {
                                tag.eq_ignore_ascii_case("button")
                                    || tag.eq_ignore_ascii_case("input")
                            }) {
                                Ok(Value::String(
                                    self.submitter_form_action_property_value_for_node(node),
                                ))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "formAction".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::Lang => Ok(Value::String(
                            self.dom.attr(node, "lang").unwrap_or_default(),
                        )),
                        DomProp::Dir => Ok(Value::String(self.resolved_dir_for_node(node))),
                        DomProp::AccessKey => Ok(Value::String(
                            self.dom.attr(node, "accesskey").unwrap_or_default(),
                        )),
                        DomProp::AutoComplete => Ok(Value::String(
                            self.dom.attr(node, "autocomplete").unwrap_or_default(),
                        )),
                        DomProp::AutoCapitalize => Ok(Value::String(
                            self.dom.attr(node, "autocapitalize").unwrap_or_default(),
                        )),
                        DomProp::AutoCorrect => Ok(Value::String(
                            self.dom.attr(node, "autocorrect").unwrap_or_default(),
                        )),
                        DomProp::ContentEditable => Ok(Value::String(
                            self.content_editable_property_value_for_node(node),
                        )),
                        DomProp::Draggable => {
                            Ok(Value::Bool(self.draggable_property_value_for_node(node)))
                        }
                        DomProp::EnterKeyHint => Ok(Value::String(
                            self.dom.attr(node, "enterkeyhint").unwrap_or_default(),
                        )),
                        DomProp::Inert => Ok(Value::Bool(self.dom.has_attr(node, "inert")?)),
                        DomProp::InputMode => Ok(Value::String(
                            self.dom.attr(node, "inputmode").unwrap_or_default(),
                        )),
                        DomProp::Nonce => Ok(Value::String(
                            self.dom.attr(node, "nonce").unwrap_or_default(),
                        )),
                        DomProp::Popover => Ok(Value::String(
                            self.dom.attr(node, "popover").unwrap_or_default(),
                        )),
                        DomProp::Spellcheck => {
                            Ok(Value::Bool(self.spellcheck_property_value_for_node(node)))
                        }
                        DomProp::TabIndex => Ok(Value::Number(
                            self.reflected_i64_attribute_or_default(node, "tabindex", -1),
                        )),
                        DomProp::Translate => {
                            Ok(Value::Bool(self.translate_property_value_for_node(node)))
                        }
                        DomProp::Cite => Ok(Value::String(
                            self.reflected_url_attribute_or_empty(node, "cite"),
                        )),
                        DomProp::DateTime => Ok(Value::String(
                            self.dom.attr(node, "datetime").unwrap_or_default(),
                        )),
                        DomProp::BrClear => Ok(Value::String(
                            self.dom.attr(node, "clear").unwrap_or_default(),
                        )),
                        DomProp::CaptionAlign => Ok(Value::String(
                            self.dom.attr(node, "align").unwrap_or_default(),
                        )),
                        DomProp::ColSpan => {
                            if self.dom.tag_name(node).is_some_and(|tag| {
                                tag.eq_ignore_ascii_case("col")
                                    || tag.eq_ignore_ascii_case("colgroup")
                            }) {
                                Ok(Value::Number(self.col_span_value(node)))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "span".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::TableCellColSpan => {
                            if self.dom.tag_name(node).is_some_and(|tag| {
                                tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th")
                            }) {
                                Ok(Value::Number(self.table_cell_col_span_value(node)))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "colSpan".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::RowSpan => {
                            if self.dom.tag_name(node).is_some_and(|tag| {
                                tag.eq_ignore_ascii_case("td") || tag.eq_ignore_ascii_case("th")
                            }) {
                                Ok(Value::Number(self.table_cell_row_span_value(node)))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "rowSpan".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::CanvasWidth => {
                            Ok(Value::Number(self.canvas_dimension_value(node, "width")))
                        }
                        DomProp::CanvasHeight => {
                            Ok(Value::Number(self.canvas_dimension_value(node, "height")))
                        }
                        DomProp::NodeEventHandler(event_name) => {
                            let is_body_window_alias = self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
                                && event_name
                                    .strip_prefix("on")
                                    .is_some_and(Self::is_body_window_event_handler_alias);
                            if is_body_window_alias {
                                Ok(Self::object_get_entry(
                                    &self.dom_runtime.window_object.borrow(),
                                    event_name,
                                )
                                .unwrap_or(Value::Null))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, event_name.clone()))
                                    .cloned()
                                    .unwrap_or(Value::Null))
                            }
                        }
                        DomProp::BodyDeprecatedAttr(attr_name) => Ok(Value::String(
                            self.dom.attr(node, attr_name).unwrap_or_default(),
                        )),
                        DomProp::ClientWidth => {
                            Ok(Value::Number(self.client_width_property_value(node)?))
                        }
                        DomProp::ClientHeight => {
                            Ok(Value::Number(self.client_height_property_value(node)?))
                        }
                        DomProp::ClientLeft => Ok(Value::Number(self.dom.client_left(node)?)),
                        DomProp::ClientTop => Ok(Value::Number(self.dom.client_top(node)?)),
                        DomProp::CurrentCssZoom => Ok(Value::Number(1)),
                        DomProp::Dataset(key) => Ok(self
                            .dom
                            .dataset_get(node, key)?
                            .map(Value::String)
                            .unwrap_or(Value::Undefined)),
                        DomProp::Style(prop) => Ok(Value::String(self.dom.style_get(node, prop)?)),
                        DomProp::OffsetWidth => Ok(Value::Number(self.dom.offset_width(node)?)),
                        DomProp::OffsetHeight => Ok(Value::Number(self.dom.offset_height(node)?)),
                        DomProp::OffsetLeft => Ok(Value::Number(self.dom.offset_left(node)?)),
                        DomProp::OffsetTop => Ok(Value::Number(self.dom.offset_top(node)?)),
                        DomProp::ScrollWidth => Ok(Value::Number(self.dom.scroll_width(node)?)),
                        DomProp::ScrollHeight => Ok(Value::Number(self.dom.scroll_height(node)?)),
                        DomProp::ScrollLeft => Ok(Value::Number(self.dom.scroll_left(node)?)),
                        DomProp::ScrollTop => Ok(Value::Number(self.dom.scroll_top(node)?)),
                        DomProp::ScrollLeftMax => Ok(Value::Number(0)),
                        DomProp::ScrollTopMax => Ok(Value::Number(0)),
                        DomProp::ShadowRoot => Ok(self.shadow_root_property_value(node)),
                        DomProp::ActiveElement => Ok(self.document_active_element_property_value()),
                        DomProp::ActiveViewTransition => Ok(Value::Null),
                        DomProp::AdoptedStyleSheets => {
                            Ok(self.ensure_document_adopted_style_sheets_property())
                        }
                        DomProp::AdoptedStyleSheetsLength => {
                            let adopted = self.ensure_document_adopted_style_sheets_property();
                            let len = match adopted {
                                Value::Array(values) => values.borrow().len() as i64,
                                _ => 0,
                            };
                            Ok(Value::Number(len))
                        }
                        DomProp::CharacterSet => Ok(Value::String("UTF-8".to_string())),
                        DomProp::CompatMode => Ok(Value::String("CSS1Compat".to_string())),
                        DomProp::ContentType => Ok(Value::String("text/html".to_string())),
                        DomProp::ReadyState => {
                            Ok(Value::String(self.dom_runtime.document_ready_state.clone()))
                        }
                        DomProp::Referrer => Ok(Value::String(String::new())),
                        DomProp::Title => Ok(Value::String(self.dom.document_title())),
                        DomProp::Url | DomProp::DocumentUri => {
                            Ok(Value::String(self.document_url.clone()))
                        }
                        DomProp::BaseUri => Ok(Value::String(self.document_base_url())),
                        DomProp::Location => {
                            Ok(Value::Object(self.dom_runtime.location_object.clone()))
                        }
                        DomProp::LocationHref => Ok(Value::String(self.document_url.clone())),
                        DomProp::LocationProtocol => {
                            Ok(Value::String(self.current_location_parts().protocol()))
                        }
                        DomProp::LocationHost => {
                            Ok(Value::String(self.current_location_parts().host()))
                        }
                        DomProp::LocationHostname => {
                            Ok(Value::String(self.current_location_parts().hostname))
                        }
                        DomProp::LocationPort => Ok(Value::String(
                            self.current_location_parts().effective_port(),
                        )),
                        DomProp::LocationPathname => {
                            let parts = self.current_location_parts();
                            Ok(Value::String(if parts.has_authority {
                                parts.pathname
                            } else {
                                parts.opaque_path
                            }))
                        }
                        DomProp::LocationSearch => {
                            Ok(Value::String(self.current_location_parts().search))
                        }
                        DomProp::LocationHash => {
                            Ok(Value::String(self.current_location_parts().hash))
                        }
                        DomProp::LocationOrigin => {
                            Ok(Value::String(self.current_location_parts().origin()))
                        }
                        DomProp::LocationAncestorOrigins => Ok(Self::new_array_value(Vec::new())),
                        DomProp::History => {
                            Ok(Value::Object(self.location_history.history_object.clone()))
                        }
                        DomProp::HistoryLength => Ok(Value::Number(
                            self.location_history.history_entries.len() as i64,
                        )),
                        DomProp::HistoryState => Ok(self.current_history_state()),
                        DomProp::HistoryScrollRestoration => Ok(Value::String(
                            self.location_history.history_scroll_restoration.clone(),
                        )),
                        DomProp::DefaultView => {
                            Ok(env.get("window").cloned().unwrap_or(Value::Undefined))
                        }
                        DomProp::Hidden => {
                            if node == self.dom.root {
                                Ok(Value::Bool(
                                    self.dom_runtime.document_visibility_state == "hidden",
                                ))
                            } else {
                                Ok(Value::Bool(self.dom.attr(node, "hidden").is_some()))
                            }
                        }
                        DomProp::VisibilityState => Ok(Value::String(
                            self.dom_runtime.document_visibility_state.clone(),
                        )),
                        DomProp::Forms => Ok(Self::new_static_node_list_value(
                            self.dom.query_selector_all("form")?,
                        )),
                        DomProp::Images => Ok(Self::new_static_node_list_value(
                            self.dom.query_selector_all("img")?,
                        )),
                        DomProp::Links => Ok(Self::new_static_node_list_value(
                            self.dom.query_selector_all("a[href], area[href]")?,
                        )),
                        DomProp::Scripts => Ok(Self::new_static_node_list_value(
                            self.dom.query_selector_all("script")?,
                        )),
                        DomProp::Children => Ok(self.child_elements_live_list_value(node)),
                        DomProp::ChildElementCount => {
                            Ok(Value::Number(self.dom.child_element_count(node) as i64))
                        }
                        DomProp::FirstElementChild => Ok(self
                            .dom
                            .first_element_child(node)
                            .map(Value::Node)
                            .unwrap_or(Value::Null)),
                        DomProp::LastElementChild => Ok(self
                            .dom
                            .last_element_child(node)
                            .map(Value::Node)
                            .unwrap_or(Value::Null)),
                        DomProp::CurrentScript => Ok(Value::Null),
                        DomProp::FormsLength => Ok(Value::Number(
                            self.dom.query_selector_all("form")?.len() as i64,
                        )),
                        DomProp::ImagesLength => Ok(Value::Number(
                            self.dom.query_selector_all("img")?.len() as i64,
                        )),
                        DomProp::LinksLength => Ok(Value::Number(
                            self.dom.query_selector_all("a[href], area[href]")?.len() as i64,
                        )),
                        DomProp::ScriptsLength => Ok(Value::Number(
                            self.dom.query_selector_all("script")?.len() as i64,
                        )),
                        DomProp::ChildrenLength => {
                            Ok(Value::Number(self.dom.child_element_count(node) as i64))
                        }
                        DomProp::AudioSrc => Ok(Value::String(self.resolve_media_src(node))),
                        DomProp::AudioAutoplay => {
                            Ok(Value::Bool(self.dom.has_attr(node, "autoplay")?))
                        }
                        DomProp::AudioControls => {
                            Ok(Value::Bool(self.dom.has_attr(node, "controls")?))
                        }
                        DomProp::AudioControlsList => Ok(Value::String(
                            self.dom.attr(node, "controlslist").unwrap_or_default(),
                        )),
                        DomProp::AudioCrossOrigin => Ok(Value::String(
                            self.dom.attr(node, "crossorigin").unwrap_or_default(),
                        )),
                        DomProp::AudioDisableRemotePlayback => Ok(Value::Bool(
                            self.dom.has_attr(node, "disableremoteplayback")?,
                        )),
                        DomProp::VideoDisablePictureInPicture => Ok(Value::Bool(
                            self.dom.has_attr(node, "disablepictureinpicture")?,
                        )),
                        DomProp::AudioLoop => Ok(Value::Bool(self.dom.has_attr(node, "loop")?)),
                        DomProp::AudioMuted => Ok(Value::Bool(self.dom.has_attr(node, "muted")?)),
                        DomProp::AudioPreload => Ok(Value::String(
                            self.dom.attr(node, "preload").unwrap_or_default(),
                        )),
                        DomProp::VideoPlaysInline => {
                            Ok(Value::Bool(self.dom.has_attr(node, "playsinline")?))
                        }
                        DomProp::VideoPoster => Ok(Value::String(
                            self.reflected_url_attribute_or_empty(node, "poster"),
                        )),
                        DomProp::AriaString(prop_name) => Ok(Value::String(
                            self.dom
                                .attr(node, &Self::aria_property_to_attr_name(prop_name))
                                .unwrap_or_default(),
                        )),
                        DomProp::AriaElementRefSingle(prop_name) => Ok(self
                            .resolve_aria_single_element_property(node, prop_name)
                            .map(Value::Node)
                            .unwrap_or(Value::Null)),
                        DomProp::AriaElementRefList(prop_name) => {
                            Ok(Self::new_static_node_list_value(
                                self.resolve_aria_element_list_property(node, prop_name),
                            ))
                        }
                        DomProp::AnchorAlt => Ok(Value::String(
                            self.dom.attr(node, "alt").unwrap_or_default(),
                        )),
                        DomProp::AnchorAttributionSrc => Ok(Value::String(
                            self.dom.attr(node, "attributionsrc").unwrap_or_default(),
                        )),
                        DomProp::AnchorDownload => Ok(Value::String(
                            self.dom.attr(node, "download").unwrap_or_default(),
                        )),
                        DomProp::AnchorHash => {
                            Ok(Value::String(self.anchor_hash_property_value(node)))
                        }
                        DomProp::AnchorHost => Ok(Value::String(
                            self.anchor_location_parts(node)
                                .map(|parts| parts.host())
                                .unwrap_or_default(),
                        )),
                        DomProp::AnchorHostname => Ok(Value::String(
                            self.anchor_location_parts(node)
                                .map(|parts| parts.hostname)
                                .unwrap_or_default(),
                        )),
                        DomProp::AnchorHref => Ok(Value::String(self.resolve_anchor_href(node))),
                        DomProp::AnchorHreflang => Ok(Value::String(
                            self.dom.attr(node, "hreflang").unwrap_or_default(),
                        )),
                        DomProp::AnchorInterestForElement => {
                            if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("button"))
                            {
                                Ok(self
                                    .dom
                                    .attr(node, "interestfor")
                                    .and_then(|raw| {
                                        raw.split_whitespace().next().map(str::to_string)
                                    })
                                    .and_then(|id_ref| self.dom.by_id(&id_ref))
                                    .map(Value::Node)
                                    .unwrap_or(Value::Null))
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "interestfor").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AnchorOrigin => Ok(Value::String(
                            self.anchor_location_parts(node)
                                .map(|parts| parts.origin())
                                .unwrap_or_default(),
                        )),
                        DomProp::AnchorPassword => Ok(Value::String(
                            self.anchor_location_parts(node)
                                .map(|parts| parts.password)
                                .unwrap_or_default(),
                        )),
                        DomProp::AnchorPathname => Ok(Value::String(
                            self.anchor_location_parts(node)
                                .map(|parts| {
                                    if parts.has_authority {
                                        parts.pathname
                                    } else {
                                        parts.opaque_path
                                    }
                                })
                                .unwrap_or_default(),
                        )),
                        DomProp::AnchorPing => Ok(Value::String(
                            self.dom.attr(node, "ping").unwrap_or_default(),
                        )),
                        DomProp::AnchorPort => Ok(Value::String(
                            self.anchor_location_parts(node)
                                .map(|parts| parts.effective_port())
                                .unwrap_or_default(),
                        )),
                        DomProp::AnchorProtocol => Ok(Value::String(
                            self.anchor_location_parts(node)
                                .map(|parts| parts.protocol())
                                .unwrap_or_else(|| ":".to_string()),
                        )),
                        DomProp::AnchorReferrerPolicy => Ok(Value::String(
                            self.dom.attr(node, "referrerpolicy").unwrap_or_default(),
                        )),
                        DomProp::AnchorRel => Ok(Value::String(
                            self.dom.attr(node, "rel").unwrap_or_default(),
                        )),
                        DomProp::AnchorRelList => Ok(Self::new_array_value(
                            self.anchor_rel_tokens(node)
                                .into_iter()
                                .map(Value::String)
                                .collect::<Vec<_>>(),
                        )),
                        DomProp::AnchorRelListLength => {
                            Ok(Value::Number(self.anchor_rel_tokens(node).len() as i64))
                        }
                        DomProp::AnchorSearch => {
                            Ok(Value::String(self.anchor_search_property_value(node)))
                        }
                        DomProp::AnchorTarget => Ok(Value::String(
                            self.dom.attr(node, "target").unwrap_or_default(),
                        )),
                        DomProp::AnchorText => Ok(Value::String(self.dom.text_content(node))),
                        DomProp::AnchorType => {
                            if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("button"))
                            {
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
                            } else if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                            {
                                Ok(Value::String(self.normalized_input_type(node)))
                            } else if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("select"))
                            {
                                Ok(Value::String(self.select_type_property_value(node)))
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "type").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AnchorUsername => Ok(Value::String(
                            self.anchor_location_parts(node)
                                .map(|parts| parts.username)
                                .unwrap_or_default(),
                        )),
                        DomProp::AnchorNoHref => {
                            Ok(Value::Bool(self.dom.attr(node, "nohref").is_some()))
                        }
                        DomProp::AnchorCharset => Ok(Value::String(
                            self.dom.attr(node, "charset").unwrap_or_default(),
                        )),
                        DomProp::AnchorCoords => Ok(Value::String(
                            self.dom.attr(node, "coords").unwrap_or_default(),
                        )),
                        DomProp::AnchorRev => Ok(Value::String(
                            self.dom.attr(node, "rev").unwrap_or_default(),
                        )),
                        DomProp::AnchorShape => Ok(Value::String(
                            self.dom.attr(node, "shape").unwrap_or_default(),
                        )),
                        DomProp::Size => {
                            if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("select"))
                            {
                                Ok(Value::Number(self.select_size_property_value(node)))
                            } else if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                            {
                                Ok(Value::Number(self.input_size_property_value_for_node(node)))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "size".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::Min => {
                            if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                            {
                                Ok(Value::String(
                                    self.dom.attr(node, "min").unwrap_or_default(),
                                ))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "min".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::Max => {
                            if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                            {
                                Ok(Value::String(
                                    self.dom.attr(node, "max").unwrap_or_default(),
                                ))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "max".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::Step => {
                            if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                            {
                                Ok(Value::String(
                                    self.dom.attr(node, "step").unwrap_or_default(),
                                ))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "step".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::MaxLength => {
                            if self.dom.tag_name(node).is_some_and(|tag| {
                                tag.eq_ignore_ascii_case("input")
                                    || tag.eq_ignore_ascii_case("textarea")
                            }) {
                                Ok(Value::Number(self.max_length_property_value_for_node(node)))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "maxLength".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::MinLength => {
                            if self.dom.tag_name(node).is_some_and(|tag| {
                                tag.eq_ignore_ascii_case("input")
                                    || tag.eq_ignore_ascii_case("textarea")
                            }) {
                                Ok(Value::Number(self.min_length_property_value_for_node(node)))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "minLength".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::Rows => {
                            if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("textarea"))
                            {
                                Ok(Value::Number(
                                    self.textarea_rows_property_value_for_node(node),
                                ))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "rows".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::Cols => {
                            if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("textarea"))
                            {
                                Ok(Value::Number(
                                    self.textarea_cols_property_value_for_node(node),
                                ))
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "cols".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                    }
                }
                Expr::LocationMethodCall { method, url } => match method {
                    LocationMethod::Assign => {
                        let Some(url_expr) = url else {
                            return Err(Error::ScriptRuntime(
                                "location.assign requires exactly one argument".into(),
                            ));
                        };
                        let url = self
                            .eval_expr(url_expr, env, event_param, event)?
                            .as_string();
                        self.navigate_location(&url, LocationNavigationKind::Assign)?;
                        Ok(Value::Undefined)
                    }
                    LocationMethod::Reload => {
                        self.reload_location()?;
                        Ok(Value::Undefined)
                    }
                    LocationMethod::Replace => {
                        let Some(url_expr) = url else {
                            return Err(Error::ScriptRuntime(
                                "location.replace requires exactly one argument".into(),
                            ));
                        };
                        let url = self
                            .eval_expr(url_expr, env, event_param, event)?
                            .as_string();
                        self.navigate_location(&url, LocationNavigationKind::Replace)?;
                        Ok(Value::Undefined)
                    }
                    LocationMethod::ToString => Ok(Value::String(self.document_url.clone())),
                },
                Expr::HistoryMethodCall { method, args } => match method {
                    HistoryMethod::Back => {
                        let _ = args;
                        self.history_go_with_env(-1)?;
                        Ok(Value::Undefined)
                    }
                    HistoryMethod::Forward => {
                        let _ = args;
                        self.history_go_with_env(1)?;
                        Ok(Value::Undefined)
                    }
                    HistoryMethod::Go => {
                        let delta = if let Some(delta) = args.first() {
                            let value = self.eval_expr(delta, env, event_param, event)?;
                            Self::value_to_i64(&value)
                        } else {
                            0
                        };
                        self.history_go_with_env(delta)?;
                        Ok(Value::Undefined)
                    }
                    HistoryMethod::PushState => {
                        let state = self.eval_expr(&args[0], env, event_param, event)?;
                        let url = if args.len() >= 3 {
                            Some(
                                self.eval_expr(&args[2], env, event_param, event)?
                                    .as_string(),
                            )
                        } else {
                            None
                        };
                        self.history_push_state(state, url.as_deref(), false)?;
                        Ok(Value::Undefined)
                    }
                    HistoryMethod::ReplaceState => {
                        let state = self.eval_expr(&args[0], env, event_param, event)?;
                        let url = if args.len() >= 3 {
                            Some(
                                self.eval_expr(&args[2], env, event_param, event)?
                                    .as_string(),
                            )
                        } else {
                            None
                        };
                        self.history_push_state(state, url.as_deref(), true)?;
                        Ok(Value::Undefined)
                    }
                },
                Expr::ClipboardMethodCall { method, args } => {
                    self.eval_clipboard_method_call(method, args, env, event_param, event)
                }
                Expr::DocumentHasFocus => Ok(Value::Bool(self.dom.active_element().is_some())),
                Expr::DomMatches { target, selector } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    self.eval_matches_selector_value(node, selector)
                }
                Expr::DomClosest { target, selector } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    self.eval_closest_selector_value(node, selector)
                }
                Expr::DomComputedStyleProperty { target, property } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    Ok(Value::String(
                        self.computed_style_property_value(node, None, property)?,
                    ))
                }
                Expr::ClassListContains { target, class_name } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    Ok(Value::Bool(self.dom.class_contains(node, class_name)?))
                }
                Expr::QuerySelectorAllLength { target } => {
                    let len = self
                        .resolve_dom_query_list_runtime(target, env)?
                        .unwrap_or_default()
                        .len() as i64;
                    Ok(Value::Number(len))
                }
                Expr::FormElementsLength { form } => {
                    let form_node = self.resolve_dom_query_required_runtime(form, env)?;
                    let len = self.form_elements(form_node)?.len() as i64;
                    Ok(Value::Number(len))
                }
                Expr::FormDataNew { form, submitter } => Ok(Value::FormData(Rc::new(
                    RefCell::new(self.eval_form_data_constructor_entries(
                        form.as_ref(),
                        submitter.as_ref(),
                        env,
                    )?),
                ))),
                Expr::FormDataGet { source, name } => {
                    let entries = self.eval_form_data_source(source, env)?;
                    Ok(entries
                        .iter()
                        .find_map(|(entry_name, value)| {
                            (entry_name == name).then(|| Value::String(value.clone()))
                        })
                        .unwrap_or(Value::Null))
                }
                Expr::FormDataHas { source, name } => {
                    let entries = self.eval_form_data_source(source, env)?;
                    let has = entries.iter().any(|(entry_name, _)| entry_name == name);
                    Ok(Value::Bool(has))
                }
                Expr::FormDataGetAll { source, name } => {
                    let entries = self.eval_form_data_source(source, env)?;
                    let values = entries
                        .iter()
                        .filter(|(entry_name, _)| entry_name == name)
                        .map(|(_, value)| Value::String(value.clone()))
                        .collect::<Vec<_>>();
                    Ok(Self::new_array_value(values))
                }
                Expr::FormDataGetAllLength { source, name } => {
                    let entries = self.eval_form_data_source(source, env)?;
                    let len = entries
                        .iter()
                        .filter(|(entry_name, _)| entry_name == name)
                        .count() as i64;
                    Ok(Value::Number(len))
                }
                Expr::DomGetAttribute { target, name } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let name = name.to_ascii_lowercase();
                    if name == "nonce" {
                        Ok(if self.dom.attr(node, "nonce").is_some() {
                            Value::String(String::new())
                        } else {
                            Value::Null
                        })
                    } else {
                        Ok(self
                            .dom
                            .attr(node, &name)
                            .map(Value::String)
                            .unwrap_or(Value::Null))
                    }
                }
                Expr::DomHasAttribute { target, name } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    Ok(Value::Bool(self.dom.has_attr(node, name)?))
                }
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}

impl Harness {
    fn eval_clipboard_method_call(
        &mut self,
        method: &ClipboardMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let (method_name, evaluated_args) = match method {
            ClipboardMethod::ReadText => ("readText", Vec::new()),
            ClipboardMethod::WriteText => (
                "writeText",
                vec![self.eval_expr(&args[0], env, event_param, event)?],
            ),
        };

        if let Some((receiver, callee)) =
            self.resolve_clipboard_method_override(env, method_name)?
        {
            return self
                .execute_callable_value_with_this_and_env(
                    &callee,
                    &evaluated_args,
                    event,
                    Some(env),
                    Some(receiver),
                )
                .map_err(|err| match err {
                    Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                        Error::ScriptRuntime(format!("'{}' is not a function", method_name))
                    }
                    other => other,
                });
        }

        match method {
            ClipboardMethod::ReadText => {
                let promise = self.new_pending_promise();
                if let Some(reason) = self.platform_mocks.clipboard_read_error.clone() {
                    self.promise_reject(&promise, Value::String(reason));
                } else {
                    self.promise_resolve(
                        &promise,
                        Value::String(self.platform_mocks.clipboard_text.clone()),
                    )?;
                }
                Ok(Value::Promise(promise))
            }
            ClipboardMethod::WriteText => {
                let promise = self.new_pending_promise();
                if let Some(reason) = self.platform_mocks.clipboard_write_error.clone() {
                    self.promise_reject(&promise, Value::String(reason));
                } else {
                    self.platform_mocks.clipboard_text = evaluated_args[0].as_string();
                    self.promise_resolve(&promise, Value::Undefined)?;
                }
                Ok(Value::Promise(promise))
            }
        }
    }

    fn resolve_clipboard_method_override(
        &mut self,
        env: &HashMap<String, Value>,
        method_name: &str,
    ) -> Result<Option<(Value, Value)>> {
        let navigator = if let Some(value) = env.get("navigator") {
            Some(value.clone())
        } else {
            self.script_runtime.env.get("navigator").cloned()
        };
        let Some(navigator) = navigator else {
            return Ok(None);
        };

        let clipboard = self
            .object_property_from_value(&navigator, "clipboard")
            .map_err(|err| match err {
                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                    Error::ScriptRuntime(
                        "member call target does not support property 'clipboard'".into(),
                    )
                }
                other => other,
            })?;

        let use_builtin = if let Value::Object(entries) = &clipboard {
            let entries = entries.borrow();
            let is_builtin_clipboard = matches!(
                Self::object_get_entry(&entries, INTERNAL_CLIPBOARD_OBJECT_KEY),
                Some(Value::Bool(true))
            );
            if !is_builtin_clipboard {
                false
            } else {
                let default_key = match method_name {
                    "readText" => INTERNAL_CLIPBOARD_READ_TEXT_DEFAULT_KEY,
                    "writeText" => INTERNAL_CLIPBOARD_WRITE_TEXT_DEFAULT_KEY,
                    _ => return Ok(None),
                };
                let current =
                    Self::object_get_entry(&entries, method_name).unwrap_or(Value::Undefined);
                Self::object_get_entry(&entries, default_key)
                    .as_ref()
                    .is_some_and(|default_value| self.strict_equal(&current, default_value))
            }
        } else {
            false
        };

        if use_builtin {
            return Ok(None);
        }

        let callee = self
            .object_property_from_value(&clipboard, method_name)
            .map_err(|err| match err {
                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                    Error::ScriptRuntime(format!(
                        "member call target does not support property '{}'",
                        method_name
                    ))
                }
                other => other,
            })?;

        Ok(Some((clipboard, callee)))
    }

    fn viewport_inner_height_value(&self) -> i64 {
        const DEFAULT_INNER_HEIGHT: f64 = 768.0;
        let window = self.dom_runtime.window_object.borrow();
        let raw_value = Self::object_get_entry(&window, "innerHeight");
        let parsed = match raw_value {
            Some(Value::Number(value)) => Some(value as f64),
            Some(Value::Float(value)) if value.is_finite() => Some(value),
            Some(Value::String(value)) => value.parse::<f64>().ok(),
            _ => None,
        }
        .unwrap_or(DEFAULT_INNER_HEIGHT);
        if !parsed.is_finite() {
            return DEFAULT_INNER_HEIGHT as i64;
        }
        parsed.max(0.0).trunc() as i64
    }

    fn viewport_inner_width_value(&self) -> i64 {
        const DEFAULT_INNER_WIDTH: f64 = 1024.0;
        let window = self.dom_runtime.window_object.borrow();
        let raw_value = Self::object_get_entry(&window, "innerWidth");
        let parsed = match raw_value {
            Some(Value::Number(value)) => Some(value as f64),
            Some(Value::Float(value)) if value.is_finite() => Some(value),
            Some(Value::String(value)) => value.parse::<f64>().ok(),
            _ => None,
        }
        .unwrap_or(DEFAULT_INNER_WIDTH);
        if !parsed.is_finite() {
            return DEFAULT_INNER_WIDTH as i64;
        }
        parsed.max(0.0).trunc() as i64
    }

    fn client_width_property_value(&self, node: NodeId) -> Result<i64> {
        let is_document_html_element = self.dom.document_element() == Some(node)
            && self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("html"));
        if is_document_html_element {
            return Ok(self.viewport_inner_width_value());
        }
        self.dom.client_width(node)
    }

    fn client_height_property_value(&self, node: NodeId) -> Result<i64> {
        let is_document_html_element = self.dom.document_element() == Some(node)
            && self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("html"));
        if is_document_html_element {
            return Ok(self.viewport_inner_height_value());
        }
        self.dom.client_height(node)
    }

    fn document_active_element_property_value(&self) -> Value {
        if let Some(active) = self.dom.active_element() {
            if self.dom.is_connected(active) {
                return Value::Node(active);
            }
        }
        if let Some(body) = self.dom.body() {
            return Value::Node(body);
        }
        if let Some(document_element) = self.dom.document_element() {
            return Value::Node(document_element);
        }
        Value::Null
    }

    pub(crate) fn node_type_number(&self, node: NodeId) -> i64 {
        match &self.dom.nodes[node.0].node_type {
            NodeType::Document => 9,
            NodeType::Text(_) => 3,
            NodeType::Element(element)
                if element.tag_name.eq_ignore_ascii_case("#document-fragment") =>
            {
                11
            }
            NodeType::Element(_) => 1,
        }
    }

    pub(crate) fn new_static_node_list_value(nodes: Vec<NodeId>) -> Value {
        Value::NodeList(Rc::new(RefCell::new(NodeListValue::static_list(nodes))))
    }

    pub(crate) fn class_names_from_argument(value: &Value) -> Vec<String> {
        value
            .as_string()
            .split_whitespace()
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    pub(crate) fn class_names_live_list_value(
        &self,
        root: NodeId,
        class_names: Vec<String>,
    ) -> Value {
        let nodes = self
            .dom
            .get_elements_by_class_names_from(&root, &class_names);
        Value::NodeList(Rc::new(RefCell::new(
            NodeListValue::live_descendants_by_class_names(root, class_names, nodes),
        )))
    }

    pub(crate) fn name_live_list_value(&self, root: NodeId, name: String) -> Value {
        let nodes = self.dom.get_elements_by_name_from(&root, &name);
        Value::NodeList(Rc::new(RefCell::new(
            NodeListValue::live_descendants_by_name(root, name, nodes),
        )))
    }

    pub(crate) fn tag_name_from_argument(value: &Value) -> String {
        let raw = value.as_string();
        if raw == "*" {
            "*".to_string()
        } else {
            raw.to_ascii_lowercase()
        }
    }

    pub(crate) fn namespace_uri_from_create_element_ns_argument(value: &Value) -> Option<String> {
        if matches!(value, Value::Null) {
            return None;
        }
        let raw = value.as_string();
        if raw.is_empty() { None } else { Some(raw) }
    }

    pub(crate) fn tag_name_live_list_value(&self, root: NodeId, tag_name: String) -> Value {
        let nodes = self.dom.get_elements_by_tag_name_from(&root, &tag_name);
        Value::NodeList(Rc::new(RefCell::new(
            NodeListValue::live_descendants_by_tag_name(root, tag_name, nodes),
        )))
    }

    pub(crate) fn tag_name_ns_live_list_value(
        &self,
        root: NodeId,
        namespace_uri: Option<String>,
        local_name: String,
    ) -> Value {
        let nodes =
            self.dom
                .get_elements_by_tag_name_ns_from(&root, namespace_uri.as_deref(), &local_name);
        Value::NodeList(Rc::new(RefCell::new(
            NodeListValue::live_descendants_by_tag_name_ns(root, namespace_uri, local_name, nodes),
        )))
    }

    pub(crate) fn child_nodes_live_list_value(&mut self, parent: NodeId) -> Value {
        let existing = self
            .dom_runtime
            .live_child_nodes_lists
            .get(&parent)
            .cloned();
        let list = existing.unwrap_or_else(|| {
            let list = Rc::new(RefCell::new(NodeListValue::live_child_nodes(
                parent,
                self.dom.nodes[parent.0].children.clone(),
            )));
            self.dom_runtime
                .live_child_nodes_lists
                .insert(parent, list.clone());
            list
        });
        self.refresh_node_list(&list);
        Value::NodeList(list)
    }

    pub(crate) fn child_elements_live_list_value(&mut self, parent: NodeId) -> Value {
        let existing = self.dom_runtime.live_children_lists.get(&parent).cloned();
        let list = existing.unwrap_or_else(|| {
            let list = Rc::new(RefCell::new(NodeListValue::live_child_elements(
                parent,
                self.dom.child_elements(parent),
            )));
            self.dom_runtime
                .live_children_lists
                .insert(parent, list.clone());
            list
        });
        self.refresh_node_list(&list);
        Value::NodeList(list)
    }

    pub(crate) fn named_node_map_live_value(&mut self, owner: NodeId) -> Value {
        let existing = self.dom_runtime.live_named_node_maps.get(&owner).cloned();
        let map = existing.unwrap_or_else(|| {
            let named_node_map = self.new_named_node_map_value(owner);
            let Value::Object(object) = named_node_map else {
                unreachable!("new_named_node_map_value must return an object");
            };
            self.dom_runtime
                .live_named_node_maps
                .insert(owner, object.clone());
            object
        });
        Value::Object(map)
    }

    pub(crate) fn refresh_node_list(&self, list: &Rc<RefCell<NodeListValue>>) {
        let source = list.borrow().live_source.clone();
        let Some(source) = source else {
            return;
        };

        let nodes = match source {
            LiveNodeListSource::ChildNodes { parent } => {
                if self.dom.is_valid_node(parent) {
                    self.dom.nodes[parent.0].children.clone()
                } else {
                    Vec::new()
                }
            }
            LiveNodeListSource::ChildElements { parent } => {
                if self.dom.is_valid_node(parent) {
                    self.dom.child_elements(parent)
                } else {
                    Vec::new()
                }
            }
            LiveNodeListSource::DescendantsByClassNames { root, class_names } => {
                if !self.dom.is_valid_node(root) || class_names.is_empty() {
                    Vec::new()
                } else {
                    self.dom
                        .get_elements_by_class_names_from(&root, &class_names)
                }
            }
            LiveNodeListSource::DescendantsByName { root, name } => {
                if !self.dom.is_valid_node(root) {
                    Vec::new()
                } else {
                    self.dom.get_elements_by_name_from(&root, &name)
                }
            }
            LiveNodeListSource::DescendantsByTagName { root, tag_name } => {
                if !self.dom.is_valid_node(root) {
                    Vec::new()
                } else {
                    self.dom.get_elements_by_tag_name_from(&root, &tag_name)
                }
            }
            LiveNodeListSource::DescendantsByTagNameNs {
                root,
                namespace_uri,
                local_name,
            } => {
                if !self.dom.is_valid_node(root) {
                    Vec::new()
                } else {
                    self.dom.get_elements_by_tag_name_ns_from(
                        &root,
                        namespace_uri.as_deref(),
                        &local_name,
                    )
                }
            }
        };
        list.borrow_mut().nodes = nodes;
    }

    pub(crate) fn node_list_snapshot(&self, list: &Rc<RefCell<NodeListValue>>) -> Vec<NodeId> {
        self.refresh_node_list(list);
        list.borrow().nodes.clone()
    }

    pub(crate) fn node_list_len(&self, list: &Rc<RefCell<NodeListValue>>) -> usize {
        self.refresh_node_list(list);
        list.borrow().nodes.len()
    }

    pub(crate) fn node_list_get(
        &self,
        list: &Rc<RefCell<NodeListValue>>,
        index: usize,
    ) -> Option<NodeId> {
        self.refresh_node_list(list);
        list.borrow().nodes.get(index).copied()
    }

    pub(crate) fn node_name(&self, node: NodeId) -> String {
        match &self.dom.nodes[node.0].node_type {
            NodeType::Document => "#document".to_string(),
            NodeType::Text(_) => "#text".to_string(),
            NodeType::Element(element)
                if element.tag_name.eq_ignore_ascii_case("#document-fragment") =>
            {
                "#document-fragment".to_string()
            }
            NodeType::Element(_) => self.element_tag_name(node),
        }
    }

    pub(crate) fn element_tag_name(&self, node: NodeId) -> String {
        let Some(element) = self.dom.element(node) else {
            return String::new();
        };
        if element.namespace_uri.as_deref() == Some("http://www.w3.org/1999/xhtml") {
            element.tag_name.to_ascii_uppercase()
        } else {
            element.tag_name.clone()
        }
    }

    pub(crate) fn node_value(&self, node: NodeId) -> Value {
        match &self.dom.nodes[node.0].node_type {
            NodeType::Text(text) => Value::String(text.clone()),
            _ => Value::Null,
        }
    }

    pub(crate) fn node_text_content_value(&self, node: NodeId) -> Value {
        if matches!(self.dom.nodes[node.0].node_type, NodeType::Document) {
            Value::Null
        } else {
            Value::String(self.dom.text_content(node))
        }
    }

    pub(crate) fn node_root(&self, node: NodeId) -> NodeId {
        let mut current = node;
        while let Some(parent) = self.dom.parent(current) {
            current = parent;
        }
        current
    }

    pub(crate) fn node_owner_document(&self, node: NodeId) -> Option<NodeId> {
        if matches!(self.dom.nodes[node.0].node_type, NodeType::Document) {
            return None;
        }
        let root = self.node_root(node);
        if matches!(self.dom.nodes[root.0].node_type, NodeType::Document) {
            Some(root)
        } else {
            Some(self.dom.root)
        }
    }

    pub(crate) fn node_parent_element(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.dom.parent(node)?;
        match &self.dom.nodes[parent.0].node_type {
            NodeType::Element(element)
                if !element.tag_name.eq_ignore_ascii_case("#document-fragment") =>
            {
                Some(parent)
            }
            _ => None,
        }
    }

    pub(crate) fn node_previous_sibling(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.dom.parent(node)?;
        let siblings = &self.dom.nodes[parent.0].children;
        let position = siblings.iter().position(|sibling| *sibling == node)?;
        position.checked_sub(1).map(|index| siblings[index])
    }

    pub(crate) fn node_next_sibling(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.dom.parent(node)?;
        let siblings = &self.dom.nodes[parent.0].children;
        let position = siblings.iter().position(|sibling| *sibling == node)?;
        siblings.get(position + 1).copied()
    }

    fn node_document_order_index(&self, root: NodeId, target: NodeId) -> Option<usize> {
        let mut stack = vec![root];
        let mut index = 0usize;
        while let Some(current) = stack.pop() {
            if current == target {
                return Some(index);
            }
            index += 1;
            for child in self.dom.nodes[current.0].children.iter().rev() {
                stack.push(*child);
            }
        }
        None
    }

    pub(crate) fn node_compare_document_position(&self, left: NodeId, right: NodeId) -> i64 {
        const DOCUMENT_POSITION_DISCONNECTED: i64 = 0x01;
        const DOCUMENT_POSITION_PRECEDING: i64 = 0x02;
        const DOCUMENT_POSITION_FOLLOWING: i64 = 0x04;
        const DOCUMENT_POSITION_CONTAINS: i64 = 0x08;
        const DOCUMENT_POSITION_CONTAINED_BY: i64 = 0x10;
        const DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: i64 = 0x20;

        if left == right {
            return 0;
        }

        let left_root = self.node_root(left);
        let right_root = self.node_root(right);
        if left_root != right_root {
            let disconnected_order = if left.0 < right.0 {
                DOCUMENT_POSITION_FOLLOWING
            } else {
                DOCUMENT_POSITION_PRECEDING
            };
            return DOCUMENT_POSITION_DISCONNECTED
                | DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC
                | disconnected_order;
        }

        if self.dom.is_descendant_of(right, left) {
            return DOCUMENT_POSITION_CONTAINED_BY | DOCUMENT_POSITION_FOLLOWING;
        }
        if self.dom.is_descendant_of(left, right) {
            return DOCUMENT_POSITION_CONTAINS | DOCUMENT_POSITION_PRECEDING;
        }

        let left_index = self.node_document_order_index(left_root, left).unwrap_or(0);
        let right_index = self
            .node_document_order_index(left_root, right)
            .unwrap_or(0);
        if left_index < right_index {
            DOCUMENT_POSITION_FOLLOWING
        } else {
            DOCUMENT_POSITION_PRECEDING
        }
    }

    pub(crate) fn nodes_are_equal(&self, left: NodeId, right: NodeId) -> bool {
        let left_node = &self.dom.nodes[left.0];
        let right_node = &self.dom.nodes[right.0];
        let metadata_equal = match (&left_node.node_type, &right_node.node_type) {
            (NodeType::Document, NodeType::Document) => true,
            (NodeType::Text(left_text), NodeType::Text(right_text)) => left_text == right_text,
            (NodeType::Element(left_element), NodeType::Element(right_element)) => {
                left_element
                    .tag_name
                    .eq_ignore_ascii_case(&right_element.tag_name)
                    && left_element.attrs == right_element.attrs
                    && left_element.value == right_element.value
                    && left_element.files == right_element.files
                    && left_element.checked == right_element.checked
                    && left_element.indeterminate == right_element.indeterminate
                    && left_element.disabled == right_element.disabled
                    && left_element.readonly == right_element.readonly
                    && left_element.required == right_element.required
                    && left_element.custom_validity_message == right_element.custom_validity_message
                    && left_element.selection_start == right_element.selection_start
                    && left_element.selection_end == right_element.selection_end
                    && left_element.selection_direction == right_element.selection_direction
            }
            _ => false,
        };
        if !metadata_equal {
            return false;
        }
        if left_node.children.len() != right_node.children.len() {
            return false;
        }
        left_node
            .children
            .iter()
            .zip(right_node.children.iter())
            .all(|(left_child, right_child)| self.nodes_are_equal(*left_child, *right_child))
    }

    pub(crate) fn normalize_node_subtree(&mut self, node: NodeId) -> Result<()> {
        let direct_children = self.dom.nodes[node.0].children.clone();
        for child in direct_children {
            if self.dom.parent(child) == Some(node) {
                self.normalize_node_subtree(child)?;
            }
        }

        let mut index = 0usize;
        while index < self.dom.nodes[node.0].children.len() {
            let current = self.dom.nodes[node.0].children[index];
            let Some(mut merged_text) = (match &self.dom.nodes[current.0].node_type {
                NodeType::Text(text) => Some(text.clone()),
                _ => None,
            }) else {
                index += 1;
                continue;
            };

            loop {
                let Some(next) = self.dom.nodes[node.0].children.get(index + 1).copied() else {
                    break;
                };
                let Some(next_text) = (match &self.dom.nodes[next.0].node_type {
                    NodeType::Text(text) => Some(text.clone()),
                    _ => None,
                }) else {
                    break;
                };
                merged_text.push_str(&next_text);
                self.dom.remove_child(node, next)?;
            }

            if let NodeType::Text(text) = &mut self.dom.nodes[current.0].node_type {
                *text = merged_text.clone();
            }
            if merged_text.is_empty() {
                self.dom.remove_child(node, current)?;
                continue;
            }
            index += 1;
        }

        Ok(())
    }

    pub(crate) fn node_lookup_namespace_uri(
        &self,
        node: NodeId,
        prefix: Option<&str>,
    ) -> Option<String> {
        let element = self.dom.element(node)?;
        let normalized_prefix = prefix.unwrap_or_default();
        if normalized_prefix.is_empty() {
            return element.namespace_uri.clone();
        }
        element
            .tag_name
            .split_once(':')
            .filter(|(node_prefix, _)| *node_prefix == normalized_prefix)
            .and_then(|_| element.namespace_uri.clone())
    }

    pub(crate) fn node_lookup_prefix(
        &self,
        node: NodeId,
        namespace_uri: Option<&str>,
    ) -> Option<String> {
        let element = self.dom.element(node)?;
        let Some(namespace_uri) = namespace_uri else {
            return None;
        };
        if element.namespace_uri.as_deref() != Some(namespace_uri) {
            return None;
        }
        element
            .tag_name
            .split_once(':')
            .map(|(prefix, _)| prefix.to_string())
    }

    pub(crate) fn node_is_default_namespace(
        &self,
        node: NodeId,
        namespace_uri: Option<&str>,
    ) -> bool {
        let default_namespace = self.node_lookup_namespace_uri(node, None);
        match (namespace_uri, default_namespace.as_deref()) {
            (None, None) => true,
            (Some(namespace_uri), Some(default_namespace)) => namespace_uri == default_namespace,
            _ => false,
        }
    }

    pub(crate) fn clone_dom_node(&mut self, node: NodeId, deep: bool) -> Result<NodeId> {
        let source = self.dom.clone();
        let cloned = self
            .dom
            .create_node(None, source.nodes[node.0].node_type.clone());
        if deep {
            let children = source.nodes[node.0].children.clone();
            for child in children {
                let _ = self
                    .dom
                    .clone_subtree_from_dom(&source, child, Some(cloned), false)?;
            }
        }
        Ok(cloned)
    }

    pub(crate) fn template_content_fragment_value(
        &mut self,
        template_node: NodeId,
    ) -> Result<Value> {
        let source = self.dom.clone();
        let fragment = self
            .dom
            .create_detached_element("#document-fragment".to_string());
        let children = source.nodes[template_node.0].children.clone();
        for child in children {
            let _ = self
                .dom
                .clone_subtree_from_dom(&source, child, Some(fragment), false)?;
        }
        Ok(Value::Node(fragment))
    }

    fn parsed_document_root_from_entries(entries: &[(String, Value)]) -> Option<NodeId> {
        match Self::object_get_entry(entries, INTERNAL_PARSED_DOCUMENT_ROOT_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }
    }

    pub(crate) fn parsed_document_value_from_root(&mut self, root: NodeId) -> Value {
        Self::new_object_value(vec![
            (
                INTERNAL_PARSED_DOCUMENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_PARSED_DOCUMENT_ROOT_NODE_KEY.to_string(),
                Value::Node(root),
            ),
        ])
    }

    pub(crate) fn new_empty_parsed_document_value(&mut self) -> Value {
        let root = self.dom.create_node(None, NodeType::Document);
        self.parsed_document_value_from_root(root)
    }

    pub(crate) fn new_parsed_document_value_from_markup(
        &mut self,
        markup: &str,
        sanitize: bool,
    ) -> Result<Value> {
        let ParseOutput { dom: parsed, .. } = parse_html(markup)?;
        let parsed_root = self.dom.create_node(None, NodeType::Document);
        let children = parsed.nodes[parsed.root.0].children.clone();
        for child in children {
            let _ = self
                .dom
                .clone_subtree_from_dom(&parsed, child, Some(parsed_root), sanitize)?;
        }
        Ok(self.parsed_document_value_from_root(parsed_root))
    }

    fn parsed_document_document_element(&self, root: NodeId) -> Option<NodeId> {
        self.dom.nodes[root.0]
            .children
            .iter()
            .find(|child| {
                self.dom
                    .tag_name(**child)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("html"))
            })
            .copied()
            .or_else(|| {
                self.dom.nodes[root.0]
                    .children
                    .iter()
                    .find(|child| self.dom.element(**child).is_some())
                    .copied()
            })
    }

    fn parsed_document_body(&self, root: NodeId) -> Option<NodeId> {
        let doc_element = self.parsed_document_document_element(root)?;
        self.dom.nodes[doc_element.0]
            .children
            .iter()
            .find(|child| {
                self.dom
                    .tag_name(**child)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("body"))
            })
            .copied()
            .or_else(|| self.dom.query_selector_from(&root, "body").ok().flatten())
            .or(Some(doc_element))
    }

    fn parsed_document_head(&self, root: NodeId) -> Option<NodeId> {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if self
                .dom
                .tag_name(node)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("head"))
            {
                return Some(node);
            }
            for child in self.dom.nodes[node.0].children.iter().rev() {
                stack.push(*child);
            }
        }
        None
    }

    fn find_descendant_by_id(&self, root: NodeId, id: &str) -> Option<NodeId> {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if self.dom.attr(node, "id").is_some_and(|value| value == id) {
                return Some(node);
            }
            for child in self.dom.nodes[node.0].children.iter().rev() {
                stack.push(*child);
            }
        }
        None
    }

    pub(crate) fn parsed_document_property_from_entries(
        &mut self,
        entries: &[(String, Value)],
        key: &str,
    ) -> Result<Option<Value>> {
        let Some(root) = Self::parsed_document_root_from_entries(entries) else {
            return Ok(None);
        };
        Ok(match key {
            "body" => Some(
                self.parsed_document_body(root)
                    .map(Value::Node)
                    .unwrap_or(Value::Null),
            ),
            "head" => Some(
                self.parsed_document_head(root)
                    .map(Value::Node)
                    .unwrap_or(Value::Null),
            ),
            "documentElement" => Some(
                self.parsed_document_document_element(root)
                    .map(Value::Node)
                    .unwrap_or(Value::Null),
            ),
            "contentType" => Some(Value::String("text/html".to_string())),
            "URL" | "documentURI" => Some(Value::String("about:blank".to_string())),
            "createTreeWalker"
            | "querySelector"
            | "querySelectorAll"
            | "getElementById"
            | "getElementsByClassName"
            | "getElementsByName"
            | "getElementsByTagName"
            | "createElement"
            | "createElementNS"
            | "createTextNode"
            | "createAttribute"
            | "createDocumentFragment"
            | "createRange"
            | "append" => Some(Self::new_builtin_placeholder_function()),
            _ => None,
        })
    }

    pub(crate) fn dom_parser_object_property(
        &self,
        entries: &[(String, Value)],
        key: &str,
    ) -> Option<Value> {
        if !matches!(
            Self::object_get_entry(entries, INTERNAL_DOM_PARSER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            return None;
        }
        match key {
            "parseFromString" => Some(Self::new_builtin_placeholder_function()),
            _ => None,
        }
    }

    fn tree_walker_mask_for_node(&self, node: NodeId) -> u32 {
        match self.node_type_number(node) {
            1 => 0x1,
            3 => 0x4,
            8 => 0x80,
            9 => 0x100,
            11 => 0x400,
            _ => 0,
        }
    }

    fn tree_walker_accepts(what_to_show: i64, node_mask: u32) -> bool {
        if what_to_show == -1 || what_to_show == 4_294_967_295 {
            return true;
        }
        ((what_to_show as u32) & node_mask) != 0
    }

    fn collect_tree_walker_traversal(&self, root: NodeId, out: &mut Vec<NodeId>) {
        out.push(root);
        for child in &self.dom.nodes[root.0].children {
            self.collect_tree_walker_traversal(*child, out);
        }
    }

    fn tree_walker_current_node_from_entries(&self, entries: &[(String, Value)]) -> Value {
        let traversal =
            match Self::object_get_entry(entries, INTERNAL_TREE_WALKER_TRAVERSAL_NODES_KEY) {
                Some(Value::Array(nodes)) => nodes,
                _ => return Value::Null,
            };
        let nodes = traversal.borrow();
        let index = match Self::object_get_entry(entries, INTERNAL_TREE_WALKER_INDEX_KEY) {
            Some(Value::Number(index)) if index >= 0 => index as usize,
            _ => 0,
        };
        nodes.get(index).cloned().unwrap_or(Value::Null)
    }

    pub(crate) fn tree_walker_property_from_entries(
        &mut self,
        entries: &[(String, Value)],
        key: &str,
    ) -> Result<Option<Value>> {
        if !matches!(
            Self::object_get_entry(entries, INTERNAL_TREE_WALKER_OBJECT_KEY),
            Some(Value::Bool(true))
        ) {
            return Ok(None);
        }
        Ok(match key {
            "currentNode" => Some(self.tree_walker_current_node_from_entries(entries)),
            "nextNode" => Some(Self::new_builtin_placeholder_function()),
            "root" => {
                let traversal =
                    Self::object_get_entry(entries, INTERNAL_TREE_WALKER_TRAVERSAL_NODES_KEY);
                match traversal {
                    Some(Value::Array(nodes)) => nodes.borrow().first().cloned(),
                    _ => Some(Value::Null),
                }
            }
            "whatToShow" => Self::object_get_entry(entries, INTERNAL_TREE_WALKER_WHAT_TO_SHOW_KEY),
            _ => None,
        })
    }

    pub(crate) fn eval_dom_parser_member_call(
        &mut self,
        parser_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let is_parser = {
            let entries = parser_object.borrow();
            matches!(
                Self::object_get_entry(&entries, INTERNAL_DOM_PARSER_OBJECT_KEY),
                Some(Value::Bool(true))
            )
        };
        if !is_parser {
            return Ok(None);
        }

        match member {
            "parseFromString" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "DOMParser.parseFromString requires exactly two arguments".into(),
                    ));
                }
                let markup = evaluated_args[0].as_string();
                let mime_type = evaluated_args[1].as_string().to_ascii_lowercase();
                if mime_type.trim() != "text/html" {
                    return Err(Error::ScriptRuntime(
                        "DOMParser.parseFromString supports only 'text/html'".into(),
                    ));
                }

                Ok(Some(
                    self.new_parsed_document_value_from_markup(&markup, false)?,
                ))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_parsed_document_member_call(
        &mut self,
        document_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
        _event: &EventState,
    ) -> Result<Option<Value>> {
        let root = {
            let entries = document_object.borrow();
            if !matches!(
                Self::object_get_entry(&entries, INTERNAL_PARSED_DOCUMENT_OBJECT_KEY),
                Some(Value::Bool(true))
            ) {
                return Ok(None);
            }
            let Some(root) = Self::parsed_document_root_from_entries(&entries) else {
                return Ok(None);
            };
            root
        };

        match member {
            "append" => Ok(Some(self.eval_document_append_call(root, evaluated_args)?)),
            "getElementById" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementById requires exactly one argument".into(),
                    ));
                }
                let id = evaluated_args[0].as_string();
                Ok(Some(
                    self.find_descendant_by_id(root, &id)
                        .map(Value::Node)
                        .unwrap_or(Value::Null),
                ))
            }
            "getElementsByClassName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByClassName requires exactly one argument".into(),
                    ));
                }
                let class_names = Self::class_names_from_argument(&evaluated_args[0]);
                Ok(Some(self.class_names_live_list_value(root, class_names)))
            }
            "getElementsByName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(
                    self.name_live_list_value(root, evaluated_args[0].as_string()),
                ))
            }
            "getElementsByTagName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(self.tag_name_live_list_value(
                    root,
                    Self::tag_name_from_argument(&evaluated_args[0]),
                )))
            }
            "querySelector" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelector requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_query_selector_value(root, &selector)?))
            }
            "querySelectorAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelectorAll requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_query_selector_all_value(root, &selector)?))
            }
            "createTreeWalker" => self.eval_create_tree_walker_call(evaluated_args),
            "createElement" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "createElement requires one or two arguments".into(),
                    ));
                }
                let tag_name = evaluated_args[0].as_string().to_ascii_lowercase();
                let node = self.dom.create_detached_element(tag_name);
                if let Some(is_value) =
                    Self::create_element_is_option_from_arg(evaluated_args.get(1))
                {
                    self.dom.set_attr(node, "is", &is_value)?;
                }
                Ok(Some(Value::Node(node)))
            }
            "createElementNS" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "createElementNS requires two or three arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let raw_tag_name = evaluated_args[1].as_string();
                let tag_name = if namespace_uri.as_deref() == Some("http://www.w3.org/1999/xhtml") {
                    raw_tag_name.to_ascii_lowercase()
                } else {
                    raw_tag_name
                };
                let node = self
                    .dom
                    .create_detached_element_with_namespace(tag_name, namespace_uri);
                if let Some(is_value) =
                    Self::create_element_is_option_from_arg(evaluated_args.get(2))
                {
                    self.dom.set_attr(node, "is", &is_value)?;
                }
                Ok(Some(Value::Node(node)))
            }
            "createTextNode" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "createTextNode requires exactly one argument".into(),
                    ));
                }
                let text = evaluated_args[0].as_string();
                let node = self.dom.create_detached_text(text);
                Ok(Some(Value::Node(node)))
            }
            "createAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "createAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if !is_valid_create_attribute_name(&name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                Ok(Some(Self::new_attr_object_value(&name, "", None)))
            }
            "createDocumentFragment" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "createDocumentFragment takes no arguments".into(),
                    ));
                }
                let node = self
                    .dom
                    .create_detached_element("#document-fragment".to_string());
                Ok(Some(Value::Node(node)))
            }
            "createRange" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "createRange takes no arguments".into(),
                    ));
                }
                Ok(Some(Self::new_range_object_value(root)))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_tree_walker_member_call(
        &mut self,
        walker_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let is_tree_walker = {
            let entries = walker_object.borrow();
            matches!(
                Self::object_get_entry(&entries, INTERNAL_TREE_WALKER_OBJECT_KEY),
                Some(Value::Bool(true))
            )
        };
        if !is_tree_walker {
            return Ok(None);
        }

        match member {
            "nextNode" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("nextNode takes no arguments".into()));
                }
                let (traversal, current_index, what_to_show) = {
                    let entries = walker_object.borrow();
                    let traversal = match Self::object_get_entry(
                        &entries,
                        INTERNAL_TREE_WALKER_TRAVERSAL_NODES_KEY,
                    ) {
                        Some(Value::Array(nodes)) => nodes,
                        _ => return Ok(Some(Value::Null)),
                    };
                    let current_index =
                        match Self::object_get_entry(&entries, INTERNAL_TREE_WALKER_INDEX_KEY) {
                            Some(Value::Number(index)) if index >= 0 => index as usize,
                            _ => 0,
                        };
                    let what_to_show = match Self::object_get_entry(
                        &entries,
                        INTERNAL_TREE_WALKER_WHAT_TO_SHOW_KEY,
                    ) {
                        Some(Value::Number(mask)) => mask,
                        _ => 4_294_967_295,
                    };
                    (traversal, current_index, what_to_show)
                };

                let nodes = traversal.borrow();
                for index in (current_index + 1)..nodes.len() {
                    let Value::Node(node) = &nodes[index] else {
                        continue;
                    };
                    if !Self::tree_walker_accepts(
                        what_to_show,
                        self.tree_walker_mask_for_node(*node),
                    ) {
                        continue;
                    }
                    Self::object_set_entry(
                        &mut walker_object.borrow_mut(),
                        INTERNAL_TREE_WALKER_INDEX_KEY.to_string(),
                        Value::Number(index as i64),
                    );
                    return Ok(Some(Value::Node(*node)));
                }
                Ok(Some(Value::Null))
            }
            _ => Ok(None),
        }
    }

    fn range_boundary_node_from_value(&self, value: &Value) -> Result<NodeId> {
        match value {
            Value::Node(node) if self.dom.is_valid_node(*node) => Ok(*node),
            Value::Object(entries) => {
                let entries = entries.borrow();
                if matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    Ok(self.dom.root)
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_PARSED_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    Self::parsed_document_root_from_entries(&entries).ok_or_else(|| {
                        Error::ScriptRuntime("Range boundary container must be a Node".into())
                    })
                } else {
                    Err(Error::ScriptRuntime(
                        "Range boundary container must be a Node".into(),
                    ))
                }
            }
            _ => Err(Error::ScriptRuntime(
                "Range boundary container must be a Node".into(),
            )),
        }
    }

    pub(crate) fn eval_range_member_call(
        &mut self,
        range_object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let is_range = {
            let entries = range_object.borrow();
            Self::is_range_object(&entries)
        };
        if !is_range {
            return Ok(None);
        }

        match member {
            "setStart" | "setEnd" => {
                if evaluated_args.len() != 2 {
                    let message = if member == "setStart" {
                        "setStart requires exactly two arguments"
                    } else {
                        "setEnd requires exactly two arguments"
                    };
                    return Err(Error::ScriptRuntime(message.into()));
                }

                let container = self.range_boundary_node_from_value(&evaluated_args[0])?;
                let offset = Self::value_to_i64(&evaluated_args[1]);
                if offset < 0 {
                    return Err(Error::ScriptRuntime(
                        "IndexSizeError: offset must be non-negative".into(),
                    ));
                }

                let (internal_container_key, internal_offset_key, container_key, offset_key) =
                    if member == "setStart" {
                        (
                            INTERNAL_RANGE_START_CONTAINER_KEY,
                            INTERNAL_RANGE_START_OFFSET_KEY,
                            "startContainer",
                            "startOffset",
                        )
                    } else {
                        (
                            INTERNAL_RANGE_END_CONTAINER_KEY,
                            INTERNAL_RANGE_END_OFFSET_KEY,
                            "endContainer",
                            "endOffset",
                        )
                    };
                let mut entries = range_object.borrow_mut();
                Self::object_set_entry(
                    &mut entries,
                    internal_container_key.to_string(),
                    Value::Node(container),
                );
                Self::object_set_entry(
                    &mut entries,
                    internal_offset_key.to_string(),
                    Value::Number(offset),
                );
                Self::object_set_entry(
                    &mut entries,
                    container_key.to_string(),
                    Value::Node(container),
                );
                Self::object_set_entry(&mut entries, offset_key.to_string(), Value::Number(offset));

                Ok(Some(Value::Undefined))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_create_tree_walker_call(
        &mut self,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        if evaluated_args.is_empty() {
            return Err(Error::ScriptRuntime(
                "createTreeWalker requires at least one root argument".into(),
            ));
        }

        let root = match &evaluated_args[0] {
            Value::Node(node) => *node,
            Value::Object(entries) => {
                let entries = entries.borrow();
                if matches!(
                    Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    self.dom.root
                } else if matches!(
                    Self::object_get_entry(&entries, INTERNAL_PARSED_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    Self::parsed_document_root_from_entries(&entries).unwrap_or(self.dom.root)
                } else {
                    return Err(Error::ScriptRuntime(
                        "createTreeWalker root must be a Node".into(),
                    ));
                }
            }
            _ => {
                return Err(Error::ScriptRuntime(
                    "createTreeWalker root must be a Node".into(),
                ));
            }
        };

        let what_to_show = evaluated_args
            .get(1)
            .map(Self::value_to_i64)
            .unwrap_or(4_294_967_295);

        let mut traversal = Vec::new();
        self.collect_tree_walker_traversal(root, &mut traversal);
        let traversal_values = traversal.into_iter().map(Value::Node).collect::<Vec<_>>();

        Ok(Some(Self::new_object_value(vec![
            (
                INTERNAL_TREE_WALKER_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            (
                INTERNAL_TREE_WALKER_TRAVERSAL_NODES_KEY.to_string(),
                Self::new_array_value(traversal_values),
            ),
            (INTERNAL_TREE_WALKER_INDEX_KEY.to_string(), Value::Number(0)),
            (
                INTERNAL_TREE_WALKER_WHAT_TO_SHOW_KEY.to_string(),
                Value::Number(what_to_show),
            ),
        ])))
    }
}
