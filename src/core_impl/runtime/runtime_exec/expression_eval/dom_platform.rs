use super::*;

impl Harness {
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
                    let target_value = match target {
                        DomQuery::Var(name) => env.get(name).cloned(),
                        DomQuery::VarPath { base, path } => {
                            self.resolve_dom_query_var_path_value(base, path, env)?
                        }
                        _ => None,
                    };
                    if let Some(value) = target_value {
                        if !matches!(value, Value::Node(_) | Value::NodeList(_)) {
                            if let Some(key) = Self::object_key_from_dom_prop(prop) {
                                let variable_name = target.describe_call();
                                return self.object_property_from_named_value(
                                    &variable_name,
                                    &value,
                                    key,
                                );
                            }
                        }
                    }
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match prop {
                        DomProp::Attributes => {
                            let element = self.dom.element(node).ok_or_else(|| {
                                Error::ScriptRuntime("attributes target is not an element".into())
                            })?;
                            let mut attrs = element
                                .attrs
                                .iter()
                                .map(|(name, value)| (name.clone(), Value::String(value.clone())))
                                .collect::<Vec<_>>();
                            attrs.sort_by(|(left, _), (right, _)| left.cmp(right));
                            attrs.insert(
                                0,
                                ("length".to_string(), Value::Number(attrs.len() as i64)),
                            );
                            Ok(Self::new_object_value(attrs))
                        }
                        DomProp::AssignedSlot => Ok(Value::Null),
                        DomProp::Value => Ok(Value::String(self.dom.value(node)?)),
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
                        DomProp::TextContent => Ok(Value::String(self.dom.text_content(node))),
                        DomProp::InnerText => Ok(Value::String(self.dom.text_content(node))),
                        DomProp::InnerHtml => Ok(Value::String(self.dom.inner_html(node)?)),
                        DomProp::OuterHtml => Ok(Value::String(self.dom.outer_html(node)?)),
                        DomProp::ClassName => Ok(Value::String(
                            self.dom.attr(node, "class").unwrap_or_default(),
                        )),
                        DomProp::ClassList => Ok(Self::new_array_value(
                            class_tokens(self.dom.attr(node, "class").as_deref())
                                .into_iter()
                                .map(Value::String)
                                .collect::<Vec<_>>(),
                        )),
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
                        DomProp::TagName => Ok(Value::String(
                            self.dom
                                .tag_name(node)
                                .map(|name| name.to_ascii_uppercase())
                                .unwrap_or_default(),
                        )),
                        DomProp::LocalName => Ok(Value::String(
                            self.dom
                                .tag_name(node)
                                .map(|name| name.to_ascii_lowercase())
                                .unwrap_or_default(),
                        )),
                        DomProp::NamespaceUri => {
                            if self.dom.element(node).is_some() {
                                Ok(Value::String("http://www.w3.org/1999/xhtml".to_string()))
                            } else {
                                Ok(Value::Null)
                            }
                        }
                        DomProp::Prefix => Ok(Value::Null),
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
                        DomProp::Role => Ok(Value::String(
                            self.dom.attr(node, "role").unwrap_or_default(),
                        )),
                        DomProp::ElementTiming => Ok(Value::String(
                            self.dom.attr(node, "elementtiming").unwrap_or_default(),
                        )),
                        DomProp::Name => Ok(Value::String(
                            self.dom.attr(node, "name").unwrap_or_default(),
                        )),
                        DomProp::Lang => Ok(Value::String(
                            self.dom.attr(node, "lang").unwrap_or_default(),
                        )),
                        DomProp::ClientWidth => Ok(Value::Number(self.dom.offset_width(node)?)),
                        DomProp::ClientHeight => Ok(Value::Number(self.dom.offset_height(node)?)),
                        DomProp::ClientLeft => Ok(Value::Number(self.dom.offset_left(node)?)),
                        DomProp::ClientTop => Ok(Value::Number(self.dom.offset_top(node)?)),
                        DomProp::CurrentCssZoom => Ok(Value::Number(1)),
                        DomProp::Dataset(key) => {
                            Ok(Value::String(self.dom.dataset_get(node, key)?))
                        }
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
                        DomProp::ShadowRoot => Ok(Value::Null),
                        DomProp::ActiveElement => Ok(self
                            .dom
                            .active_element()
                            .map(Value::Node)
                            .unwrap_or(Value::Null)),
                        DomProp::CharacterSet => Ok(Value::String("UTF-8".to_string())),
                        DomProp::CompatMode => Ok(Value::String("CSS1Compat".to_string())),
                        DomProp::ContentType => Ok(Value::String("text/html".to_string())),
                        DomProp::ReadyState => Ok(Value::String("complete".to_string())),
                        DomProp::Referrer => Ok(Value::String(String::new())),
                        DomProp::Title => Ok(Value::String(self.dom.document_title())),
                        DomProp::Url | DomProp::DocumentUri => {
                            Ok(Value::String(self.document_url.clone()))
                        }
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
                        DomProp::LocationPort => {
                            Ok(Value::String(self.current_location_parts().port))
                        }
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
                                Ok(Value::Bool(false))
                            } else {
                                Ok(Value::Bool(self.dom.attr(node, "hidden").is_some()))
                            }
                        }
                        DomProp::VisibilityState => Ok(Value::String("visible".to_string())),
                        DomProp::Forms => Ok(Value::NodeList(self.dom.query_selector_all("form")?)),
                        DomProp::Images => Ok(Value::NodeList(self.dom.query_selector_all("img")?)),
                        DomProp::Links => Ok(Value::NodeList(
                            self.dom.query_selector_all("a[href], area[href]")?,
                        )),
                        DomProp::Scripts => {
                            Ok(Value::NodeList(self.dom.query_selector_all("script")?))
                        }
                        DomProp::Children => Ok(Value::NodeList(self.dom.child_elements(node))),
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
                        DomProp::AriaString(prop_name) => Ok(Value::String(
                            self.dom
                                .attr(node, &Self::aria_property_to_attr_name(prop_name))
                                .unwrap_or_default(),
                        )),
                        DomProp::AriaElementRefSingle(prop_name) => Ok(self
                            .resolve_aria_single_element_property(node, prop_name)
                            .map(Value::Node)
                            .unwrap_or(Value::Null)),
                        DomProp::AriaElementRefList(prop_name) => Ok(Value::NodeList(
                            self.resolve_aria_element_list_property(node, prop_name),
                        )),
                        DomProp::AnchorAttributionSrc => Ok(Value::String(
                            self.dom.attr(node, "attributionsrc").unwrap_or_default(),
                        )),
                        DomProp::AnchorDownload => Ok(Value::String(
                            self.dom.attr(node, "download").unwrap_or_default(),
                        )),
                        DomProp::AnchorHash => {
                            Ok(Value::String(self.anchor_location_parts(node).hash))
                        }
                        DomProp::AnchorHost => {
                            Ok(Value::String(self.anchor_location_parts(node).host()))
                        }
                        DomProp::AnchorHostname => {
                            Ok(Value::String(self.anchor_location_parts(node).hostname))
                        }
                        DomProp::AnchorHref => Ok(Value::String(self.resolve_anchor_href(node))),
                        DomProp::AnchorHreflang => Ok(Value::String(
                            self.dom.attr(node, "hreflang").unwrap_or_default(),
                        )),
                        DomProp::AnchorInterestForElement => Ok(Value::String(
                            self.dom.attr(node, "interestfor").unwrap_or_default(),
                        )),
                        DomProp::AnchorOrigin => {
                            Ok(Value::String(self.anchor_location_parts(node).origin()))
                        }
                        DomProp::AnchorPassword => {
                            Ok(Value::String(self.anchor_location_parts(node).password))
                        }
                        DomProp::AnchorPathname => {
                            let parts = self.anchor_location_parts(node);
                            Ok(Value::String(if parts.has_authority {
                                parts.pathname
                            } else {
                                parts.opaque_path
                            }))
                        }
                        DomProp::AnchorPing => Ok(Value::String(
                            self.dom.attr(node, "ping").unwrap_or_default(),
                        )),
                        DomProp::AnchorPort => {
                            Ok(Value::String(self.anchor_location_parts(node).port))
                        }
                        DomProp::AnchorProtocol => {
                            Ok(Value::String(self.anchor_location_parts(node).protocol()))
                        }
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
                            Ok(Value::String(self.anchor_location_parts(node).search))
                        }
                        DomProp::AnchorTarget => Ok(Value::String(
                            self.dom.attr(node, "target").unwrap_or_default(),
                        )),
                        DomProp::AnchorText => Ok(Value::String(self.dom.text_content(node))),
                        DomProp::AnchorType => Ok(Value::String(
                            self.dom.attr(node, "type").unwrap_or_default(),
                        )),
                        DomProp::AnchorUsername => {
                            Ok(Value::String(self.anchor_location_parts(node).username))
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
                Expr::ClipboardMethodCall { method, args } => match method {
                    ClipboardMethod::ReadText => {
                        let _ = args;
                        let promise = self.new_pending_promise();
                        self.promise_resolve(
                            &promise,
                            Value::String(self.platform_mocks.clipboard_text.clone()),
                        )?;
                        Ok(Value::Promise(promise))
                    }
                    ClipboardMethod::WriteText => {
                        let text = self
                            .eval_expr(&args[0], env, event_param, event)?
                            .as_string();
                        self.platform_mocks.clipboard_text = text;
                        let promise = self.new_pending_promise();
                        self.promise_resolve(&promise, Value::Undefined)?;
                        Ok(Value::Promise(promise))
                    }
                },
                Expr::DocumentHasFocus => Ok(Value::Bool(self.dom.active_element().is_some())),
                Expr::DomMatches { target, selector } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let result = self.dom.matches_selector(node, selector)?;
                    Ok(Value::Bool(result))
                }
                Expr::DomClosest { target, selector } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let result = self.dom.closest(node, selector)?;
                    Ok(result.map_or(Value::Null, Value::Node))
                }
                Expr::DomComputedStyleProperty { target, property } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    Ok(Value::String(self.dom.style_get(node, property)?))
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
                Expr::FormDataNew { form } => {
                    let form_node = self.resolve_dom_query_required_runtime(form, env)?;
                    Ok(Value::FormData(self.form_data_entries(form_node)?))
                }
                Expr::FormDataGet { source, name } => {
                    let entries = self.eval_form_data_source(source, env)?;
                    let value = entries
                        .iter()
                        .find_map(|(entry_name, value)| (entry_name == name).then(|| value.clone()))
                        .unwrap_or_default();
                    Ok(Value::String(value))
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
                    Ok(self
                        .dom
                        .attr(node, name)
                        .map(Value::String)
                        .unwrap_or(Value::Null))
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
