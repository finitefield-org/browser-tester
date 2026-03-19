use super::*;

impl Harness {
    pub(crate) fn try_eval_dom_read_anchor_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
    ) -> Result<Option<Value>> {
        let value = match prop {
            DomProp::AnchorAlt => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["alt"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "alt").unwrap_or_default())
                }
            }
            DomProp::AnchorAttributionSrc => {
                if self.node_explicit_own_property_overrides_dom_property(node, "attributionSrc") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "attributionSrc",
                    )? {
                        value
                    } else {
                        Value::String(self.dom.attr(node, "attributionsrc").unwrap_or_default())
                    }
                } else {
                    Value::String(self.dom.attr(node, "attributionsrc").unwrap_or_default())
                }
            }
            DomProp::AnchorDownload => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["download"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "download").unwrap_or_default())
                }
            }
            DomProp::AnchorHash => Value::String(self.anchor_hash_property_value(node)),
            DomProp::AnchorHost => Value::String(
                self.anchor_location_parts(node)
                    .map(|parts| parts.host())
                    .unwrap_or_default(),
            ),
            DomProp::AnchorHostname => Value::String(
                self.anchor_location_parts(node)
                    .map(|parts| parts.hostname)
                    .unwrap_or_default(),
            ),
            DomProp::AnchorHref => {
                if self.node_explicit_own_property_overrides_dom_property(node, "href") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "href",
                    )? {
                        value
                    } else {
                        Value::String(self.resolve_anchor_href(node))
                    }
                } else {
                    Value::String(self.resolve_anchor_href(node))
                }
            }
            DomProp::AnchorHreflang => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["hreflang"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "hreflang").unwrap_or_default())
                }
            }
            DomProp::AnchorInterestForElement => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("button"))
                {
                    self.dom
                        .attr(node, "interestfor")
                        .and_then(|raw| raw.split_whitespace().next().map(str::to_string))
                        .and_then(|id_ref| self.dom.by_id(&id_ref))
                        .map(Value::Node)
                        .unwrap_or(Value::Null)
                } else {
                    Value::String(self.dom.attr(node, "interestfor").unwrap_or_default())
                }
            }
            DomProp::AnchorOrigin => Value::String(
                self.anchor_location_parts(node)
                    .map(|parts| parts.origin())
                    .unwrap_or_default(),
            ),
            DomProp::AnchorPassword => Value::String(
                self.anchor_location_parts(node)
                    .map(|parts| parts.password)
                    .unwrap_or_default(),
            ),
            DomProp::AnchorPathname => Value::String(
                self.anchor_location_parts(node)
                    .map(|parts| {
                        if parts.has_authority {
                            parts.pathname
                        } else {
                            parts.opaque_path
                        }
                    })
                    .unwrap_or_default(),
            ),
            DomProp::AnchorPing => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["ping"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "ping").unwrap_or_default())
                }
            }
            DomProp::AnchorPort => Value::String(
                self.anchor_location_parts(node)
                    .map(|parts| parts.effective_port())
                    .unwrap_or_default(),
            ),
            DomProp::AnchorProtocol => Value::String(
                self.anchor_location_parts(node)
                    .map(|parts| parts.protocol())
                    .unwrap_or_else(|| ":".to_string()),
            ),
            DomProp::AnchorReferrerPolicy => {
                if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                    node,
                    &["referrerPolicy", "referrerpolicy"],
                )? {
                    value
                } else {
                    Value::String(self.dom.attr(node, "referrerpolicy").unwrap_or_default())
                }
            }
            DomProp::AnchorRel => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["rel"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "rel").unwrap_or_default())
                }
            }
            DomProp::AnchorRelList => Self::new_array_value(
                self.anchor_rel_tokens(node)
                    .into_iter()
                    .map(Value::String)
                    .collect::<Vec<_>>(),
            ),
            DomProp::AnchorRelListLength => {
                Value::Number(self.anchor_rel_tokens(node).len() as i64)
            }
            DomProp::AnchorSearch => Value::String(self.anchor_search_property_value(node)),
            DomProp::AnchorTarget => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["target"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "target").unwrap_or_default())
                }
            }
            DomProp::AnchorText => Value::String(self.dom.text_content(node)),
            DomProp::AnchorType => {
                if self.node_explicit_own_property_overrides_dom_property(node, "type") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "type",
                    )? {
                        value
                    } else {
                        Value::String(self.dom.attr(node, "type").unwrap_or_default())
                    }
                } else if self
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
                    Value::String(normalized)
                } else if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("input"))
                {
                    Value::String(self.normalized_input_type(node))
                } else if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("select"))
                {
                    Value::String(self.select_type_property_value(node))
                } else {
                    Value::String(self.dom.attr(node, "type").unwrap_or_default())
                }
            }
            DomProp::AnchorUsername => Value::String(
                self.anchor_location_parts(node)
                    .map(|parts| parts.username)
                    .unwrap_or_default(),
            ),
            DomProp::AnchorNoHref => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["noHref", "nohref"])?
                {
                    value
                } else {
                    Value::Bool(self.dom.attr(node, "nohref").is_some())
                }
            }
            DomProp::AnchorCharset => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["charset"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "charset").unwrap_or_default())
                }
            }
            DomProp::AnchorCoords => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["coords"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "coords").unwrap_or_default())
                }
            }
            DomProp::AnchorRev => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["rev"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "rev").unwrap_or_default())
                }
            }
            DomProp::AnchorShape => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["shape"])?
                {
                    value
                } else {
                    Value::String(self.dom.attr(node, "shape").unwrap_or_default())
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
