use super::*;

impl Harness {
    pub(crate) fn try_execute_dom_assign_navigation_anchor_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
        value: &Value,
        event: &mut EventState,
    ) -> Result<bool> {
        match prop {
            DomProp::Title => self.dom.set_document_title(&value.as_string())?,
            DomProp::AdoptedStyleSheets => {
                self.set_document_adopted_style_sheets_property(value.clone())?;
            }
            DomProp::Location | DomProp::LocationHref => {
                self.navigate_location(&value.as_string(), LocationNavigationKind::HrefSet)?;
            }
            DomProp::LocationProtocol => self.set_location_property("protocol", value.clone())?,
            DomProp::LocationHost => self.set_location_property("host", value.clone())?,
            DomProp::LocationHostname => self.set_location_property("hostname", value.clone())?,
            DomProp::LocationPort => self.set_location_property("port", value.clone())?,
            DomProp::LocationPathname => self.set_location_property("pathname", value.clone())?,
            DomProp::LocationSearch => self.set_location_property("search", value.clone())?,
            DomProp::LocationHash => self.set_location_property("hash", value.clone())?,
            DomProp::HistoryScrollRestoration => {
                self.set_history_property("scrollRestoration", value.clone())?;
            }
            DomProp::AnchorAlt => {
                if self.node_explicit_own_property_overrides_dom_property(node, "alt") {
                    self.set_node_assignment_property(node, "alt", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "alt", &value.as_string())?;
                }
            }
            DomProp::AnchorAttributionSrc => {
                if self.node_explicit_own_property_overrides_dom_property(node, "attributionSrc") {
                    self.set_node_assignment_property(
                        node,
                        "attributionSrc",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom
                        .set_attr(node, "attributionsrc", &value.as_string())?;
                }
            }
            DomProp::AnchorDownload => {
                if self.node_explicit_own_property_overrides_dom_property(node, "download") {
                    self.set_node_assignment_property(
                        node,
                        "download",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "download", &value.as_string())?;
                }
            }
            DomProp::AnchorHash => self.set_anchor_url_property(node, "hash", value.clone())?,
            DomProp::AnchorHost => self.set_anchor_url_property(node, "host", value.clone())?,
            DomProp::AnchorHostname => {
                self.set_anchor_url_property(node, "hostname", value.clone())?;
            }
            DomProp::AnchorHref => {
                if self.node_explicit_own_property_overrides_dom_property(node, "href") {
                    self.set_node_assignment_property(node, "href", value.clone(), event, false)?;
                } else {
                    self.set_anchor_url_property(node, "href", value.clone())?;
                }
            }
            DomProp::AnchorHreflang => {
                if self.node_explicit_own_property_overrides_dom_property(node, "hreflang") {
                    self.set_node_assignment_property(
                        node,
                        "hreflang",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "hreflang", &value.as_string())?;
                }
            }
            DomProp::AnchorInterestForElement => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("button"))
                {
                    match value {
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
                    self.dom.set_attr(node, "interestfor", &value.as_string())?;
                }
            }
            DomProp::AnchorPassword => {
                self.set_anchor_url_property(node, "password", value.clone())?;
            }
            DomProp::AnchorPathname => {
                self.set_anchor_url_property(node, "pathname", value.clone())?;
            }
            DomProp::AnchorPing => {
                if self.node_explicit_own_property_overrides_dom_property(node, "ping") {
                    self.set_node_assignment_property(node, "ping", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "ping", &value.as_string())?;
                }
            }
            DomProp::AnchorPort => self.set_anchor_url_property(node, "port", value.clone())?,
            DomProp::AnchorProtocol => {
                self.set_anchor_url_property(node, "protocol", value.clone())?;
            }
            DomProp::AnchorReferrerPolicy => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["referrerPolicy", "referrerpolicy"],
                ) {
                    self.set_node_assignment_property(
                        node,
                        shadow_key,
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom
                        .set_attr(node, "referrerpolicy", &value.as_string())?;
                }
            }
            DomProp::AnchorRel => {
                if self.node_explicit_own_property_overrides_dom_property(node, "rel") {
                    self.set_node_assignment_property(node, "rel", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "rel", &value.as_string())?;
                }
            }
            DomProp::AnchorSearch => self.set_anchor_url_property(node, "search", value.clone())?,
            DomProp::AnchorTarget => {
                if self.node_explicit_own_property_overrides_dom_property(node, "target") {
                    self.set_node_assignment_property(node, "target", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "target", &value.as_string())?;
                }
            }
            DomProp::AnchorText => self.dom.set_text_content(node, &value.as_string())?,
            DomProp::AnchorType => {
                if self.node_explicit_own_property_overrides_dom_property(node, "type") {
                    self.set_node_assignment_property(node, "type", value.clone(), event, false)?;
                } else if !self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("select"))
                {
                    self.dom.set_attr(node, "type", &value.as_string())?;
                }
            }
            DomProp::AnchorUsername => {
                self.set_anchor_url_property(node, "username", value.clone())?;
            }
            DomProp::AnchorNoHref => {
                if let Some(shadow_key) =
                    self.node_explicit_own_dom_property_shadow_key(node, &["noHref", "nohref"])
                {
                    self.set_node_assignment_property(
                        node,
                        shadow_key,
                        value.clone(),
                        event,
                        false,
                    )?;
                } else if value.truthy() {
                    self.dom.set_attr(node, "nohref", "true")?;
                } else {
                    self.dom.remove_attr(node, "nohref")?;
                }
            }
            DomProp::AnchorCharset => {
                if self.node_explicit_own_property_overrides_dom_property(node, "charset") {
                    self.set_node_assignment_property(
                        node,
                        "charset",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "charset", &value.as_string())?;
                }
            }
            DomProp::AnchorCoords => {
                if self.node_explicit_own_property_overrides_dom_property(node, "coords") {
                    self.set_node_assignment_property(node, "coords", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "coords", &value.as_string())?;
                }
            }
            DomProp::AnchorRev => {
                if self.node_explicit_own_property_overrides_dom_property(node, "rev") {
                    self.set_node_assignment_property(node, "rev", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "rev", &value.as_string())?;
                }
            }
            DomProp::AnchorShape => {
                if self.node_explicit_own_property_overrides_dom_property(node, "shape") {
                    self.set_node_assignment_property(node, "shape", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "shape", &value.as_string())?;
                }
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
