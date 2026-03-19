use super::*;

impl Harness {
    pub(crate) fn dom_prop_non_node_fallback_path(prop: &DomProp) -> Option<Vec<&'static str>> {
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

    fn form_data_member_fallback_expr(
        source: &FormDataSource,
        member: &str,
        name: &str,
    ) -> Option<Expr> {
        match source {
            FormDataSource::Var(var_name) => Some(Expr::MemberCall {
                target: Box::new(Expr::Var(var_name.clone())),
                member: member.to_string(),
                args: vec![Expr::String(name.to_string())],
                optional: false,
                optional_call: false,
            }),
            FormDataSource::New { .. } => None,
        }
    }

    fn form_data_member_fallback_is_lookup_miss(err: &Error, member: &str) -> bool {
        match err {
            Error::ScriptRuntime(msg) => {
                msg == &format!("'{member}' is not a function")
                    || msg == &format!("member call target does not support property '{member}'")
            }
            _ => false,
        }
    }

    fn eval_form_data_member_expr_with_fallback(
        &mut self,
        source: &FormDataSource,
        member: &str,
        name: &str,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match self.eval_form_data_source(source, env) {
            Ok(entries) => match member {
                "get" => Ok(entries
                    .iter()
                    .find_map(|(entry_name, value)| {
                        (entry_name == name).then(|| Value::String(value.clone()))
                    })
                    .unwrap_or(Value::Null)),
                "has" => Ok(Value::Bool(
                    entries.iter().any(|(entry_name, _)| entry_name == name),
                )),
                "getAll" => Ok(Self::new_array_value(
                    entries
                        .iter()
                        .filter(|(entry_name, _)| entry_name == name)
                        .map(|(_, value)| Value::String(value.clone()))
                        .collect(),
                )),
                _ => Err(Error::ScriptRuntime(format!(
                    "unsupported FormData expression fallback: {member}"
                ))),
            },
            Err(form_data_err) => {
                let Some(fallback_expr) =
                    Self::form_data_member_fallback_expr(source, member, name)
                else {
                    return Err(form_data_err);
                };
                match self.eval_expr(&fallback_expr, env, event_param, event) {
                    Ok(value) => Ok(value),
                    Err(err) if Self::form_data_member_fallback_is_lookup_miss(&err, member) => {
                        Err(form_data_err)
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }

    fn eval_form_data_get_all_length_expr_with_fallback(
        &mut self,
        source: &FormDataSource,
        name: &str,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match self.eval_form_data_source(source, env) {
            Ok(entries) => Ok(Value::Number(
                entries
                    .iter()
                    .filter(|(entry_name, _)| entry_name == name)
                    .count() as i64,
            )),
            Err(form_data_err) => {
                let Some(fallback_expr) =
                    Self::form_data_member_fallback_expr(source, "getAll", name)
                else {
                    return Err(form_data_err);
                };
                match self.eval_expr(&fallback_expr, env, event_param, event) {
                    Ok(value) => self.object_property_from_value(&value, "length"),
                    Err(err) if Self::form_data_member_fallback_is_lookup_miss(&err, "getAll") => {
                        Err(form_data_err)
                    }
                    Err(err) => Err(err),
                }
            }
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
                            if self.node_explicit_own_property_overrides_dom_property(node, "value")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "value",
                                )? {
                                    Ok(value)
                                } else if self
                                    .dom
                                    .tag_name(node)
                                    .is_some_and(|tag| tag.eq_ignore_ascii_case("li"))
                                {
                                    Ok(Value::Number(self.li_value_property(node)))
                                } else {
                                    Ok(Value::String(self.dom.value(node)?))
                                }
                            } else if self
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
                        DomProp::Open => {
                            if self.node_explicit_own_property_overrides_dom_property(node, "open")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "open",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::Bool(self.dom.has_attr(node, "open")?))
                                }
                            } else {
                                Ok(Value::Bool(self.dom.has_attr(node, "open")?))
                            }
                        }
                        DomProp::ReturnValue => Ok(Value::String(self.dialog_return_value(node)?)),
                        DomProp::ClosedBy => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["closedBy", "closedby"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "closedby").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::Readonly => Ok(Value::Bool(self.dom.readonly(node))),
                        DomProp::Disabled => Ok(Value::Bool(self.dom.disabled(node))),
                        DomProp::Required => Ok(Value::Bool(self.dom.required(node))),
                        DomProp::NodeType => Ok(Value::Number(self.node_type_number(node))),
                        DomProp::TextContent => Ok(self.node_text_content_value(node)),
                        DomProp::InnerText => Ok(Value::String(self.dom.text_content(node))),
                        DomProp::InnerHtml => Ok(Value::String(self.dom.inner_html(node)?)),
                        DomProp::OuterHtml => Ok(Value::String(self.dom.outer_html(node)?)),
                        DomProp::ClassName => {
                            if self.node_explicit_own_property_overrides_dom_property(
                                node,
                                "className",
                            ) {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "className",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(
                                        self.dom.attr(node, "class").unwrap_or_default(),
                                    ))
                                }
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "class").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::ClassList => Ok(self.class_list_live_value(node)),
                        DomProp::ClassListLength => {
                            let list = self.class_list_live_value(node);
                            self.object_property_from_value_with_receiver(&list, "length", &list)
                        }
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
                            if self.node_explicit_own_property_overrides_dom_property(node, "id") {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "id",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(self.dom.attr(node, "id").unwrap_or_default()))
                                }
                            } else {
                                Ok(Value::String(self.dom.attr(node, "id").unwrap_or_default()))
                            }
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
                        DomProp::Slot => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["slot"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "slot").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::Role => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["role"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(self.resolved_role_for_node(node)))
                            }
                        }
                        DomProp::ElementTiming => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["elementTiming", "elementtiming"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "elementtiming").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::HtmlFor => {
                            if self
                                .node_explicit_own_property_overrides_dom_property(node, "htmlFor")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "htmlFor",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(
                                        self.dom.attr(node, "for").unwrap_or_default(),
                                    ))
                                }
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "for").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::Name => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["name"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "name").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::Action => {
                            if self
                                .dom
                                .tag_name(node)
                                .is_some_and(|tag| tag.eq_ignore_ascii_case("form"))
                            {
                                if self.node_has_explicit_own_property(node, "action") {
                                    let entries = self.node_expando_entries(node);
                                    if let Some(value) = self
                                        .object_property_from_entries_with_getter(
                                            &Value::Node(node),
                                            &entries,
                                            "action",
                                        )?
                                    {
                                        Ok(value)
                                    } else {
                                        Ok(Value::String(
                                            self.form_action_property_value_for_node(node),
                                        ))
                                    }
                                } else {
                                    Ok(Value::String(
                                        self.form_action_property_value_for_node(node),
                                    ))
                                }
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
                                if self.node_explicit_own_property_overrides_dom_property(
                                    node,
                                    "formAction",
                                ) {
                                    let entries = self.node_expando_entries(node);
                                    if let Some(value) = self
                                        .object_property_from_entries_with_getter(
                                            &Value::Node(node),
                                            &entries,
                                            "formAction",
                                        )?
                                    {
                                        Ok(value)
                                    } else {
                                        Ok(Value::String(
                                            self.submitter_form_action_property_value_for_node(
                                                node,
                                            ),
                                        ))
                                    }
                                } else {
                                    Ok(Value::String(
                                        self.submitter_form_action_property_value_for_node(node),
                                    ))
                                }
                            } else {
                                Ok(self
                                    .dom_runtime
                                    .node_expando_props
                                    .get(&(node, "formAction".to_string()))
                                    .cloned()
                                    .unwrap_or(Value::Undefined))
                            }
                        }
                        DomProp::Lang => {
                            if self.node_explicit_own_property_overrides_dom_property(node, "lang")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "lang",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(
                                        self.dom.attr(node, "lang").unwrap_or_default(),
                                    ))
                                }
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "lang").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::Dir => {
                            if self.node_explicit_own_property_overrides_dom_property(node, "dir") {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "dir",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(self.resolved_dir_for_node(node)))
                                }
                            } else {
                                Ok(Value::String(self.resolved_dir_for_node(node)))
                            }
                        }
                        DomProp::AccessKey => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["accessKey", "accesskey"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "accesskey").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AutoComplete => Ok(Value::String(
                            self.dom.attr(node, "autocomplete").unwrap_or_default(),
                        )),
                        DomProp::AutoCapitalize => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["autocapitalize"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "autocapitalize").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AutoCorrect => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["autocorrect"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "autocorrect").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::ContentEditable => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["contentEditable", "contenteditable"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.content_editable_property_value_for_node(node),
                                ))
                            }
                        }
                        DomProp::Draggable => {
                            if let Some(value) = self
                                .node_explicit_own_dom_property_shadow_value(node, &["draggable"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(self.draggable_property_value_for_node(node)))
                            }
                        }
                        DomProp::EnterKeyHint => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["enterKeyHint", "enterkeyhint"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "enterkeyhint").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::Inert => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["inert"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(self.dom.has_attr(node, "inert")?))
                            }
                        }
                        DomProp::InputMode => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["inputMode", "inputmode"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "inputmode").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::Nonce => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["nonce"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "nonce").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::Popover => {
                            if let Some(value) = self
                                .node_explicit_own_dom_property_shadow_value(node, &["popover"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "popover").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::Spellcheck => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["spellcheck"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(self.spellcheck_property_value_for_node(node)))
                            }
                        }
                        DomProp::TabIndex => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["tabIndex", "tabindex"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(
                                    self.reflected_i64_attribute_or_default(node, "tabindex", -1),
                                ))
                            }
                        }
                        DomProp::Translate => {
                            if let Some(value) = self
                                .node_explicit_own_dom_property_shadow_value(node, &["translate"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(self.translate_property_value_for_node(node)))
                            }
                        }
                        DomProp::Cite => {
                            if self.node_explicit_own_property_overrides_dom_property(node, "cite")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "cite",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(
                                        self.reflected_url_attribute_or_empty(node, "cite"),
                                    ))
                                }
                            } else {
                                Ok(Value::String(
                                    self.reflected_url_attribute_or_empty(node, "cite"),
                                ))
                            }
                        }
                        DomProp::DateTime => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["dateTime", "datetime"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "datetime").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::BrClear => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["clear"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "clear").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::CaptionAlign => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["align"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "align").unwrap_or_default(),
                                ))
                            }
                        }
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
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["clientWidth"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.client_width_property_value(node)?))
                            }
                        }
                        DomProp::ClientHeight => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["clientHeight"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.client_height_property_value(node)?))
                            }
                        }
                        DomProp::ClientLeft => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["clientLeft"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.dom.client_left(node)?))
                            }
                        }
                        DomProp::ClientTop => {
                            if let Some(value) = self
                                .node_explicit_own_dom_property_shadow_value(node, &["clientTop"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.dom.client_top(node)?))
                            }
                        }
                        DomProp::CurrentCssZoom => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["currentCSSZoom"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(1))
                            }
                        }
                        DomProp::Dataset(key) => {
                            let map = self.dom_string_map_live_value(node);
                            self.object_property_from_value_with_receiver(&map, key, &map)
                        }
                        DomProp::Style(prop) => Ok(Value::String(self.dom.style_get(node, prop)?)),
                        DomProp::OffsetWidth => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["offsetWidth"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.dom.offset_width(node)?))
                            }
                        }
                        DomProp::OffsetHeight => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["offsetHeight"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.dom.offset_height(node)?))
                            }
                        }
                        DomProp::OffsetLeft => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["offsetLeft"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.dom.offset_left(node)?))
                            }
                        }
                        DomProp::OffsetTop => {
                            if let Some(value) = self
                                .node_explicit_own_dom_property_shadow_value(node, &["offsetTop"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.dom.offset_top(node)?))
                            }
                        }
                        DomProp::ScrollWidth => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["scrollWidth"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.dom.scroll_width(node)?))
                            }
                        }
                        DomProp::ScrollHeight => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["scrollHeight"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.dom.scroll_height(node)?))
                            }
                        }
                        DomProp::ScrollLeft => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["scrollLeft"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.dom.scroll_left(node)?))
                            }
                        }
                        DomProp::ScrollTop => {
                            if let Some(value) = self
                                .node_explicit_own_dom_property_shadow_value(node, &["scrollTop"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::Number(self.dom.scroll_top(node)?))
                            }
                        }
                        DomProp::ScrollLeftMax => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["scrollLeftMax"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(0))
                            }
                        }
                        DomProp::ScrollTopMax => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["scrollTopMax"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Number(0))
                            }
                        }
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
                            Ok(Value::Object(self.dom_runtime.window_object.clone()))
                        }
                        DomProp::Hidden => {
                            if node == self.dom.root {
                                Ok(Value::Bool(
                                    self.dom_runtime.document_visibility_state == "hidden",
                                ))
                            } else if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["hidden"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(self.dom.attr(node, "hidden").is_some()))
                            }
                        }
                        DomProp::VisibilityState => Ok(Value::String(
                            self.dom_runtime.document_visibility_state.clone(),
                        )),
                        DomProp::Forms => Ok(self.document_forms_live_list_value()),
                        DomProp::Images => Ok(self.document_images_live_list_value()),
                        DomProp::Links => Ok(self.document_links_live_list_value()),
                        DomProp::Scripts => Ok(self.document_scripts_live_list_value()),
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
                        DomProp::FormsLength => {
                            let list = self.document_forms_live_list_value();
                            self.object_property_from_value_with_receiver(&list, "length", &list)
                        }
                        DomProp::ImagesLength => {
                            let list = self.document_images_live_list_value();
                            self.object_property_from_value_with_receiver(&list, "length", &list)
                        }
                        DomProp::LinksLength => {
                            let list = self.document_links_live_list_value();
                            self.object_property_from_value_with_receiver(&list, "length", &list)
                        }
                        DomProp::ScriptsLength => {
                            let list = self.document_scripts_live_list_value();
                            self.object_property_from_value_with_receiver(&list, "length", &list)
                        }
                        DomProp::ChildrenLength => {
                            let list = self.child_elements_live_list_value(node);
                            self.object_property_from_value_with_receiver(&list, "length", &list)
                        }
                        DomProp::AudioSrc => {
                            if self.node_explicit_own_property_overrides_dom_property(node, "src") {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "src",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(self.resolve_media_src(node)))
                                }
                            } else {
                                Ok(Value::String(self.resolve_media_src(node)))
                            }
                        }
                        DomProp::AudioAutoplay => {
                            if let Some(value) = self
                                .node_explicit_own_dom_property_shadow_value(node, &["autoplay"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(self.dom.has_attr(node, "autoplay")?))
                            }
                        }
                        DomProp::AudioControls => {
                            if let Some(value) = self
                                .node_explicit_own_dom_property_shadow_value(node, &["controls"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(self.dom.has_attr(node, "controls")?))
                            }
                        }
                        DomProp::AudioControlsList => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["controlsList", "controlslist"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "controlslist").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AudioCrossOrigin => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["crossOrigin", "crossorigin"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "crossorigin").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AudioDisableRemotePlayback => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["disableRemotePlayback", "disableremoteplayback"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(
                                    self.dom.has_attr(node, "disableremoteplayback")?,
                                ))
                            }
                        }
                        DomProp::VideoDisablePictureInPicture => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["disablePictureInPicture", "disablepictureinpicture"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(
                                    self.dom.has_attr(node, "disablepictureinpicture")?,
                                ))
                            }
                        }
                        DomProp::AudioLoop => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["loop"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(self.dom.has_attr(node, "loop")?))
                            }
                        }
                        DomProp::AudioMuted => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["muted"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(self.dom.has_attr(node, "muted")?))
                            }
                        }
                        DomProp::AudioPreload => {
                            if self
                                .node_explicit_own_property_overrides_dom_property(node, "preload")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "preload",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(
                                        self.dom.attr(node, "preload").unwrap_or_default(),
                                    ))
                                }
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "preload").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::VideoPlaysInline => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["playsInline", "playsinline"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(self.dom.has_attr(node, "playsinline")?))
                            }
                        }
                        DomProp::VideoPoster => {
                            if self
                                .node_explicit_own_property_overrides_dom_property(node, "poster")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "poster",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(
                                        self.reflected_url_attribute_or_empty(node, "poster"),
                                    ))
                                }
                            } else {
                                Ok(Value::String(
                                    self.reflected_url_attribute_or_empty(node, "poster"),
                                ))
                            }
                        }
                        DomProp::Data => {
                            if self.node_explicit_own_property_overrides_dom_property(node, "data")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "data",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(
                                        self.reflected_url_attribute_or_empty(node, "data"),
                                    ))
                                }
                            } else {
                                Ok(Value::String(
                                    self.reflected_url_attribute_or_empty(node, "data"),
                                ))
                            }
                        }
                        DomProp::SrcDoc => {
                            if self
                                .node_explicit_own_property_overrides_dom_property(node, "srcdoc")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "srcdoc",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(
                                        self.dom.attr(node, "srcdoc").unwrap_or_default(),
                                    ))
                                }
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "srcdoc").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::UseMap => {
                            if self
                                .node_explicit_own_property_overrides_dom_property(node, "useMap")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "useMap",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(
                                        self.dom.attr(node, "usemap").unwrap_or_default(),
                                    ))
                                }
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "usemap").unwrap_or_default(),
                                ))
                            }
                        }
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
                        DomProp::AnchorAlt => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["alt"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "alt").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AnchorAttributionSrc => {
                            if self.node_explicit_own_property_overrides_dom_property(
                                node,
                                "attributionSrc",
                            ) {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "attributionSrc",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(
                                        self.dom.attr(node, "attributionsrc").unwrap_or_default(),
                                    ))
                                }
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "attributionsrc").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AnchorDownload => {
                            if let Some(value) = self
                                .node_explicit_own_dom_property_shadow_value(node, &["download"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "download").unwrap_or_default(),
                                ))
                            }
                        }
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
                        DomProp::AnchorHref => {
                            if self.node_explicit_own_property_overrides_dom_property(node, "href")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "href",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(self.resolve_anchor_href(node)))
                                }
                            } else {
                                Ok(Value::String(self.resolve_anchor_href(node)))
                            }
                        }
                        DomProp::AnchorHreflang => {
                            if let Some(value) = self
                                .node_explicit_own_dom_property_shadow_value(node, &["hreflang"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "hreflang").unwrap_or_default(),
                                ))
                            }
                        }
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
                        DomProp::AnchorPing => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["ping"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "ping").unwrap_or_default(),
                                ))
                            }
                        }
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
                        DomProp::AnchorReferrerPolicy => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["referrerPolicy", "referrerpolicy"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "referrerpolicy").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AnchorRel => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["rel"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "rel").unwrap_or_default(),
                                ))
                            }
                        }
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
                        DomProp::AnchorTarget => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["target"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "target").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AnchorText => Ok(Value::String(self.dom.text_content(node))),
                        DomProp::AnchorType => {
                            if self.node_explicit_own_property_overrides_dom_property(node, "type")
                            {
                                let entries = self.node_expando_entries(node);
                                if let Some(value) = self.object_property_from_entries_with_getter(
                                    &Value::Node(node),
                                    &entries,
                                    "type",
                                )? {
                                    Ok(value)
                                } else {
                                    Ok(Value::String(
                                        self.dom.attr(node, "type").unwrap_or_default(),
                                    ))
                                }
                            } else {
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
                        }
                        DomProp::AnchorUsername => Ok(Value::String(
                            self.anchor_location_parts(node)
                                .map(|parts| parts.username)
                                .unwrap_or_default(),
                        )),
                        DomProp::AnchorNoHref => {
                            if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                                node,
                                &["noHref", "nohref"],
                            )? {
                                Ok(value)
                            } else {
                                Ok(Value::Bool(self.dom.attr(node, "nohref").is_some()))
                            }
                        }
                        DomProp::AnchorCharset => {
                            if let Some(value) = self
                                .node_explicit_own_dom_property_shadow_value(node, &["charset"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "charset").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AnchorCoords => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["coords"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "coords").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AnchorRev => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["rev"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "rev").unwrap_or_default(),
                                ))
                            }
                        }
                        DomProp::AnchorShape => {
                            if let Some(value) =
                                self.node_explicit_own_dom_property_shadow_value(node, &["shape"])?
                            {
                                Ok(value)
                            } else {
                                Ok(Value::String(
                                    self.dom.attr(node, "shape").unwrap_or_default(),
                                ))
                            }
                        }
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
                Expr::FormDataGet { source, name } => self
                    .eval_form_data_member_expr_with_fallback(
                        source,
                        "get",
                        name,
                        env,
                        event_param,
                        event,
                    ),
                Expr::FormDataHas { source, name } => self
                    .eval_form_data_member_expr_with_fallback(
                        source,
                        "has",
                        name,
                        env,
                        event_param,
                        event,
                    ),
                Expr::FormDataGetAll { source, name } => self
                    .eval_form_data_member_expr_with_fallback(
                        source,
                        "getAll",
                        name,
                        env,
                        event_param,
                        event,
                    ),
                Expr::FormDataGetAllLength { source, name } => self
                    .eval_form_data_get_all_length_expr_with_fallback(
                        source,
                        name,
                        env,
                        event_param,
                        event,
                    ),
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
        self.dom
            .active_element()
            .filter(|node| self.dom.is_connected(*node))
            .or_else(|| self.dom.body())
            .or_else(|| self.dom.document_element())
            .map(Value::Node)
            .unwrap_or(Value::Null)
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
}
