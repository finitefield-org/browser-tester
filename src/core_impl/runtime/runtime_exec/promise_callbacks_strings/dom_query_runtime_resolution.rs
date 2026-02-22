use super::*;

impl Harness {
    fn node_list_from_value(value: &Value) -> Option<Vec<NodeId>> {
        match value {
            Value::NodeList(nodes) => Some(nodes.clone()),
            Value::Array(values) => {
                let values = values.borrow();
                let mut nodes = Vec::with_capacity(values.len());
                for value in values.iter() {
                    let Value::Node(node) = value else {
                        return None;
                    };
                    nodes.push(*node);
                }
                Some(nodes)
            }
            _ => None,
        }
    }

    fn form_elements_named_item(&self, controls: &[NodeId], key: &str) -> Option<NodeId> {
        if let Ok(index) = key.parse::<usize>() {
            return controls.get(index).copied();
        }

        controls
            .iter()
            .copied()
            .find(|node| self.dom.attr(*node, "id").is_some_and(|id| id == key))
            .or_else(|| {
                controls
                    .iter()
                    .copied()
                    .find(|node| self.dom.attr(*node, "name").is_some_and(|name| name == key))
            })
    }

    fn resolve_form_elements_index_static(
        &mut self,
        controls: &[NodeId],
        index: &DomIndex,
    ) -> Result<Option<NodeId>> {
        match index {
            DomIndex::Static(index) => Ok(controls.get(*index).copied()),
            DomIndex::Dynamic(expr_src) => {
                let expr = parse_expr(expr_src)?;
                match expr {
                    Expr::String(name) => Ok(self.form_elements_named_item(controls, &name)),
                    _ => Err(Error::ScriptRuntime(
                        "dynamic index in static context".into(),
                    )),
                }
            }
        }
    }

    fn resolve_form_elements_index_runtime(
        &mut self,
        controls: &[NodeId],
        index: &DomIndex,
        env: &HashMap<String, Value>,
    ) -> Result<Option<NodeId>> {
        match index {
            DomIndex::Static(index) => Ok(controls.get(*index).copied()),
            DomIndex::Dynamic(expr_src) => {
                let expr = parse_expr(expr_src)?;
                let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                let value = self.eval_expr(&expr, env, &None, &event)?;
                if let Some(index) = self.value_as_index(&value) {
                    return Ok(controls.get(index).copied());
                }
                Ok(self.form_elements_named_item(controls, &value.as_string()))
            }
        }
    }

    pub(crate) fn resolve_dom_query_var_path_value(
        &self,
        base: &str,
        path: &[String],
        env: &HashMap<String, Value>,
    ) -> Result<Option<Value>> {
        let Some(mut value) = env.get(base).cloned() else {
            return Err(Error::ScriptRuntime(format!(
                "unknown element variable: {}",
                base
            )));
        };

        for key in path {
            let next = match self.object_property_from_value(&value, key) {
                Ok(next) => next,
                Err(_) => return Ok(None),
            };
            if matches!(next, Value::Null | Value::Undefined) {
                return Ok(None);
            }
            value = next;
        }

        Ok(Some(value))
    }

    pub(crate) fn resolve_dom_query_list_static(
        &mut self,
        target: &DomQuery,
    ) -> Result<Option<Vec<NodeId>>> {
        match target {
            DomQuery::BySelectorAll { selector } => {
                Ok(Some(self.dom.query_selector_all(selector)?))
            }
            DomQuery::QuerySelectorAll { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                Ok(Some(
                    self.dom.query_selector_all_from(&target_node, selector)?,
                ))
            }
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_static(target)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, None)?;
                Ok(list.get(index).copied().map(|node| vec![node]))
            }
            DomQuery::BySelectorAllIndex { selector, index } => {
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                Ok(self
                    .dom
                    .query_selector_all(selector)?
                    .get(index)
                    .copied()
                    .map(|node| vec![node]))
            }
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                let list = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(list.get(index).copied().map(|node| vec![node]))
            }
            DomQuery::Var(_) | DomQuery::VarPath { .. } => Err(Error::ScriptRuntime(
                "element variable cannot be resolved in static context".into(),
            )),
            _ => Ok(None),
        }
    }

    pub(crate) fn resolve_dom_query_list_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<Option<Vec<NodeId>>> {
        match target {
            DomQuery::Var(name) => match env.get(name) {
                Some(value) => {
                    if let Some(nodes) = Self::node_list_from_value(value) {
                        Ok(Some(nodes))
                    } else {
                        Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not a node list",
                            name
                        )))
                    }
                }
                None => Err(Error::ScriptRuntime(format!(
                    "unknown element variable: {}",
                    name
                ))),
            },
            DomQuery::VarPath { base, path } => {
                let Some(value) = self.resolve_dom_query_var_path_value(base, path, env)? else {
                    return Ok(None);
                };
                if let Some(nodes) = Self::node_list_from_value(&value) {
                    Ok(Some(nodes))
                } else {
                    Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a node list",
                        target.describe_call()
                    )))
                }
            }
            DomQuery::QuerySelectorAll {
                target: query_target,
                selector,
            } => {
                let Some(target_node) = self.resolve_dom_query_runtime(query_target, env)? else {
                    return Ok(None);
                };
                Ok(Some(
                    self.dom.query_selector_all_from(&target_node, selector)?,
                ))
            }
            DomQuery::QuerySelectorAllIndex {
                target: query_target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_runtime(query_target, env)? else {
                    return Ok(None);
                };
                let all = self.dom.query_selector_all_from(&target_node, selector)?;
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                Ok(all.get(index).copied().map(|node| vec![node]))
            }
            _ => self.resolve_dom_query_list_static(target),
        }
    }

    pub(crate) fn resolve_dom_query_static(&mut self, target: &DomQuery) -> Result<Option<NodeId>> {
        match target {
            DomQuery::DocumentRoot => Ok(Some(self.dom.root)),
            DomQuery::DocumentBody => Ok(Some(self.dom.ensure_document_body_element()?)),
            DomQuery::DocumentHead => Ok(self.dom.head()),
            DomQuery::DocumentElement => Ok(self.dom.document_element()),
            DomQuery::ById(id) => Ok(self.dom.by_id(id)),
            DomQuery::BySelector(selector) => self.dom.query_selector(selector),
            DomQuery::BySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::BySelectorAllIndex { selector, index } => {
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                let all = self.dom.query_selector_all(selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_static(target)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, None)?;
                Ok(list.get(index).copied())
            }
            DomQuery::QuerySelector { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                self.dom.query_selector_from(&target_node, selector)
            }
            DomQuery::QuerySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                let index = index.static_index().ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index in static context".into())
                })?;
                let all = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::FormElementsIndex { form, index } => {
                let Some(form_node) = self.resolve_dom_query_static(form)? else {
                    return Ok(None);
                };
                let all = self.form_elements(form_node)?;
                self.resolve_form_elements_index_static(&all, index)
            }
            DomQuery::Var(_) | DomQuery::VarPath { .. } => Err(Error::ScriptRuntime(
                "element variable cannot be resolved in static context".into(),
            )),
        }
    }

    pub(crate) fn resolve_dom_query_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<Option<NodeId>> {
        match target {
            DomQuery::DocumentRoot => Ok(Some(self.dom.root)),
            DomQuery::DocumentBody => Ok(Some(self.dom.ensure_document_body_element()?)),
            DomQuery::DocumentHead => Ok(self.dom.head()),
            DomQuery::DocumentElement => Ok(self.dom.document_element()),
            DomQuery::Var(name) => match env.get(name) {
                Some(Value::Node(node)) => Ok(Some(*node)),
                Some(Value::NodeList(_)) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a single element",
                    name
                ))),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a single element",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown element variable: {}",
                    name
                ))),
            },
            DomQuery::VarPath { base, path } => {
                let Some(value) = self.resolve_dom_query_var_path_value(base, path, env)? else {
                    return Ok(None);
                };
                match value {
                    Value::Node(node) => Ok(Some(node)),
                    _ => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not a single element",
                        target.describe_call()
                    ))),
                }
            }
            DomQuery::BySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::QuerySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_runtime(target, env)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                Ok(list.get(index).copied())
            }
            DomQuery::QuerySelector { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_runtime(target, env)? else {
                    return Ok(None);
                };
                self.dom.query_selector_from(&target_node, selector)
            }
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_runtime(target, env)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                let all = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::FormElementsIndex { form, index } => {
                let Some(form_node) = self.resolve_dom_query_runtime(form, env)? else {
                    return Ok(None);
                };
                let all = self.form_elements(form_node)?;
                self.resolve_form_elements_index_runtime(&all, index, env)
            }
            _ => self.resolve_dom_query_static(target),
        }
    }

    pub(crate) fn resolve_dom_query_required_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<NodeId> {
        self.resolve_dom_query_runtime(target, env)?.ok_or_else(|| {
            Error::ScriptRuntime(format!("{} returned null", target.describe_call()))
        })
    }

    pub(crate) fn resolve_runtime_dom_index(
        &mut self,
        index: &DomIndex,
        env: Option<&HashMap<String, Value>>,
    ) -> Result<usize> {
        match index {
            DomIndex::Static(index) => Ok(*index),
            DomIndex::Dynamic(expr_src) => {
                let expr = parse_expr(expr_src)?;
                let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
                let value = self.eval_expr(
                    &expr,
                    env.ok_or_else(|| {
                        Error::ScriptRuntime("dynamic index requires runtime context".into())
                    })?,
                    &None,
                    &event,
                )?;
                self.value_as_index(&value).ok_or_else(|| {
                    Error::ScriptRuntime(format!("invalid index expression: {expr_src}"))
                })
            }
        }
    }

    pub(crate) fn describe_dom_prop(&self, prop: &DomProp) -> String {
        match prop {
            DomProp::Attributes => "attributes".into(),
            DomProp::AssignedSlot => "assignedSlot".into(),
            DomProp::Value => "value".into(),
            DomProp::Files => "files".into(),
            DomProp::FilesLength => "files.length".into(),
            DomProp::ValueAsNumber => "valueAsNumber".into(),
            DomProp::ValueAsDate => "valueAsDate".into(),
            DomProp::ValueLength => "value.length".into(),
            DomProp::ValidationMessage => "validationMessage".into(),
            DomProp::Validity => "validity".into(),
            DomProp::ValidityValueMissing => "validity.valueMissing".into(),
            DomProp::ValidityTypeMismatch => "validity.typeMismatch".into(),
            DomProp::ValidityPatternMismatch => "validity.patternMismatch".into(),
            DomProp::ValidityTooLong => "validity.tooLong".into(),
            DomProp::ValidityTooShort => "validity.tooShort".into(),
            DomProp::ValidityRangeUnderflow => "validity.rangeUnderflow".into(),
            DomProp::ValidityRangeOverflow => "validity.rangeOverflow".into(),
            DomProp::ValidityStepMismatch => "validity.stepMismatch".into(),
            DomProp::ValidityBadInput => "validity.badInput".into(),
            DomProp::ValidityValid => "validity.valid".into(),
            DomProp::ValidityCustomError => "validity.customError".into(),
            DomProp::SelectionStart => "selectionStart".into(),
            DomProp::SelectionEnd => "selectionEnd".into(),
            DomProp::SelectionDirection => "selectionDirection".into(),
            DomProp::Checked => "checked".into(),
            DomProp::Indeterminate => "indeterminate".into(),
            DomProp::Open => "open".into(),
            DomProp::ReturnValue => "returnValue".into(),
            DomProp::ClosedBy => "closedBy".into(),
            DomProp::Readonly => "readonly".into(),
            DomProp::Required => "required".into(),
            DomProp::Disabled => "disabled".into(),
            DomProp::TextContent => "textContent".into(),
            DomProp::InnerText => "innerText".into(),
            DomProp::InnerHtml => "innerHTML".into(),
            DomProp::OuterHtml => "outerHTML".into(),
            DomProp::ClassName => "className".into(),
            DomProp::ClassList => "classList".into(),
            DomProp::ClassListLength => "classList.length".into(),
            DomProp::Part => "part".into(),
            DomProp::PartLength => "part.length".into(),
            DomProp::Id => "id".into(),
            DomProp::TagName => "tagName".into(),
            DomProp::LocalName => "localName".into(),
            DomProp::NamespaceUri => "namespaceURI".into(),
            DomProp::Prefix => "prefix".into(),
            DomProp::NextElementSibling => "nextElementSibling".into(),
            DomProp::PreviousElementSibling => "previousElementSibling".into(),
            DomProp::Slot => "slot".into(),
            DomProp::Role => "role".into(),
            DomProp::ElementTiming => "elementTiming".into(),
            DomProp::HtmlFor => "htmlFor".into(),
            DomProp::Name => "name".into(),
            DomProp::Lang => "lang".into(),
            DomProp::Dir => "dir".into(),
            DomProp::Cite => "cite".into(),
            DomProp::DateTime => "dateTime".into(),
            DomProp::BrClear => "clear".into(),
            DomProp::CaptionAlign => "align".into(),
            DomProp::ColSpan => "span".into(),
            DomProp::CanvasWidth => "width".into(),
            DomProp::CanvasHeight => "height".into(),
            DomProp::NodeEventHandler(event_name) => event_name.clone(),
            DomProp::BodyDeprecatedAttr(attr_name) => attr_name.clone(),
            DomProp::ClientWidth => "clientWidth".into(),
            DomProp::ClientHeight => "clientHeight".into(),
            DomProp::ClientLeft => "clientLeft".into(),
            DomProp::ClientTop => "clientTop".into(),
            DomProp::CurrentCssZoom => "currentCSSZoom".into(),
            DomProp::OffsetWidth => "offsetWidth".into(),
            DomProp::OffsetHeight => "offsetHeight".into(),
            DomProp::OffsetLeft => "offsetLeft".into(),
            DomProp::OffsetTop => "offsetTop".into(),
            DomProp::ScrollWidth => "scrollWidth".into(),
            DomProp::ScrollHeight => "scrollHeight".into(),
            DomProp::ScrollLeft => "scrollLeft".into(),
            DomProp::ScrollTop => "scrollTop".into(),
            DomProp::ScrollLeftMax => "scrollLeftMax".into(),
            DomProp::ScrollTopMax => "scrollTopMax".into(),
            DomProp::ShadowRoot => "shadowRoot".into(),
            DomProp::Dataset(_) => "dataset".into(),
            DomProp::Style(_) => "style".into(),
            DomProp::AriaString(prop_name) => prop_name.clone(),
            DomProp::AriaElementRefSingle(prop_name) => prop_name.clone(),
            DomProp::AriaElementRefList(prop_name) => prop_name.clone(),
            DomProp::ActiveElement => "activeElement".into(),
            DomProp::CharacterSet => "characterSet".into(),
            DomProp::CompatMode => "compatMode".into(),
            DomProp::ContentType => "contentType".into(),
            DomProp::ReadyState => "readyState".into(),
            DomProp::Referrer => "referrer".into(),
            DomProp::Title => "title".into(),
            DomProp::BaseUri => "baseURI".into(),
            DomProp::AudioSrc => "src".into(),
            DomProp::AudioAutoplay => "autoplay".into(),
            DomProp::AudioControls => "controls".into(),
            DomProp::AudioControlsList => "controlsList".into(),
            DomProp::AudioCrossOrigin => "crossOrigin".into(),
            DomProp::AudioDisableRemotePlayback => "disableRemotePlayback".into(),
            DomProp::AudioLoop => "loop".into(),
            DomProp::AudioMuted => "muted".into(),
            DomProp::AudioPreload => "preload".into(),
            DomProp::Url => "URL".into(),
            DomProp::DocumentUri => "documentURI".into(),
            DomProp::Location => "location".into(),
            DomProp::LocationHref => "location.href".into(),
            DomProp::LocationProtocol => "location.protocol".into(),
            DomProp::LocationHost => "location.host".into(),
            DomProp::LocationHostname => "location.hostname".into(),
            DomProp::LocationPort => "location.port".into(),
            DomProp::LocationPathname => "location.pathname".into(),
            DomProp::LocationSearch => "location.search".into(),
            DomProp::LocationHash => "location.hash".into(),
            DomProp::LocationOrigin => "location.origin".into(),
            DomProp::LocationAncestorOrigins => "location.ancestorOrigins".into(),
            DomProp::History => "history".into(),
            DomProp::HistoryLength => "history.length".into(),
            DomProp::HistoryState => "history.state".into(),
            DomProp::HistoryScrollRestoration => "history.scrollRestoration".into(),
            DomProp::DefaultView => "defaultView".into(),
            DomProp::Hidden => "hidden".into(),
            DomProp::VisibilityState => "visibilityState".into(),
            DomProp::Forms => "forms".into(),
            DomProp::Images => "images".into(),
            DomProp::Links => "links".into(),
            DomProp::Scripts => "scripts".into(),
            DomProp::Children => "children".into(),
            DomProp::ChildElementCount => "childElementCount".into(),
            DomProp::FirstElementChild => "firstElementChild".into(),
            DomProp::LastElementChild => "lastElementChild".into(),
            DomProp::CurrentScript => "currentScript".into(),
            DomProp::FormsLength => "forms.length".into(),
            DomProp::ImagesLength => "images.length".into(),
            DomProp::LinksLength => "links.length".into(),
            DomProp::ScriptsLength => "scripts.length".into(),
            DomProp::ChildrenLength => "children.length".into(),
            DomProp::AnchorAttributionSrc => "attributionSrc".into(),
            DomProp::AnchorDownload => "download".into(),
            DomProp::AnchorHash => "hash".into(),
            DomProp::AnchorHost => "host".into(),
            DomProp::AnchorHostname => "hostname".into(),
            DomProp::AnchorHref => "href".into(),
            DomProp::AnchorHreflang => "hreflang".into(),
            DomProp::AnchorInterestForElement => "interestForElement".into(),
            DomProp::AnchorOrigin => "origin".into(),
            DomProp::AnchorPassword => "password".into(),
            DomProp::AnchorPathname => "pathname".into(),
            DomProp::AnchorPing => "ping".into(),
            DomProp::AnchorPort => "port".into(),
            DomProp::AnchorProtocol => "protocol".into(),
            DomProp::AnchorReferrerPolicy => "referrerPolicy".into(),
            DomProp::AnchorRel => "rel".into(),
            DomProp::AnchorRelList => "relList".into(),
            DomProp::AnchorRelListLength => "relList.length".into(),
            DomProp::AnchorSearch => "search".into(),
            DomProp::AnchorTarget => "target".into(),
            DomProp::AnchorText => "text".into(),
            DomProp::AnchorType => "type".into(),
            DomProp::AnchorUsername => "username".into(),
            DomProp::AnchorCharset => "charset".into(),
            DomProp::AnchorCoords => "coords".into(),
            DomProp::AnchorRev => "rev".into(),
            DomProp::AnchorShape => "shape".into(),
        }
    }

    pub(crate) fn event_node_label(&self, node: NodeId) -> String {
        if let Some(id) = self.dom.attr(node, "id") {
            if !id.is_empty() {
                return id;
            }
        }
        self.dom
            .tag_name(node)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("node-{}", node.0))
    }

    pub(crate) fn trace_node_label(&self, node: NodeId) -> String {
        if let Some(id) = self.dom.attr(node, "id") {
            if !id.is_empty() {
                return format!("#{id}");
            }
        }
        self.dom
            .tag_name(node)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("node-{}", node.0))
    }
}
