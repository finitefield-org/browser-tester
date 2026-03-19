use super::*;

impl Harness {
    pub(crate) fn try_eval_dom_read_media_embed_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
    ) -> Result<Option<Value>> {
        let value = match prop {
            DomProp::AudioSrc => {
                if self.node_explicit_own_property_overrides_dom_property(node, "src") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "src",
                    )? {
                        value
                    } else {
                        Value::String(self.resolve_media_src(node))
                    }
                } else {
                    Value::String(self.resolve_media_src(node))
                }
            }
            DomProp::AudioAutoplay => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["autoplay"])?
                {
                    value
                } else {
                    Value::Bool(self.dom.has_attr(node, "autoplay")?)
                }
            }
            DomProp::AudioControls => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["controls"])?
                {
                    value
                } else {
                    Value::Bool(self.dom.has_attr(node, "controls")?)
                }
            }
            DomProp::AudioControlsList => {
                if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                    node,
                    &["controlsList", "controlslist"],
                )? {
                    value
                } else {
                    Value::String(self.dom.attr(node, "controlslist").unwrap_or_default())
                }
            }
            DomProp::AudioCrossOrigin => {
                if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                    node,
                    &["crossOrigin", "crossorigin"],
                )? {
                    value
                } else {
                    Value::String(self.dom.attr(node, "crossorigin").unwrap_or_default())
                }
            }
            DomProp::AudioDisableRemotePlayback => {
                if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                    node,
                    &["disableRemotePlayback", "disableremoteplayback"],
                )? {
                    value
                } else {
                    Value::Bool(self.dom.has_attr(node, "disableremoteplayback")?)
                }
            }
            DomProp::VideoDisablePictureInPicture => {
                if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                    node,
                    &["disablePictureInPicture", "disablepictureinpicture"],
                )? {
                    value
                } else {
                    Value::Bool(self.dom.has_attr(node, "disablepictureinpicture")?)
                }
            }
            DomProp::AudioLoop => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["loop"])?
                {
                    value
                } else {
                    Value::Bool(self.dom.has_attr(node, "loop")?)
                }
            }
            DomProp::AudioMuted => {
                if let Some(value) =
                    self.node_explicit_own_dom_property_shadow_value(node, &["muted"])?
                {
                    value
                } else {
                    Value::Bool(self.dom.has_attr(node, "muted")?)
                }
            }
            DomProp::AudioPreload => {
                if self.node_explicit_own_property_overrides_dom_property(node, "preload") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "preload",
                    )? {
                        value
                    } else {
                        Value::String(self.dom.attr(node, "preload").unwrap_or_default())
                    }
                } else {
                    Value::String(self.dom.attr(node, "preload").unwrap_or_default())
                }
            }
            DomProp::VideoPlaysInline => {
                if let Some(value) = self.node_explicit_own_dom_property_shadow_value(
                    node,
                    &["playsInline", "playsinline"],
                )? {
                    value
                } else {
                    Value::Bool(self.dom.has_attr(node, "playsinline")?)
                }
            }
            DomProp::VideoPoster => {
                if self.node_explicit_own_property_overrides_dom_property(node, "poster") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "poster",
                    )? {
                        value
                    } else {
                        Value::String(self.reflected_url_attribute_or_empty(node, "poster"))
                    }
                } else {
                    Value::String(self.reflected_url_attribute_or_empty(node, "poster"))
                }
            }
            DomProp::Data => {
                if self.node_explicit_own_property_overrides_dom_property(node, "data") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "data",
                    )? {
                        value
                    } else {
                        Value::String(self.reflected_url_attribute_or_empty(node, "data"))
                    }
                } else {
                    Value::String(self.reflected_url_attribute_or_empty(node, "data"))
                }
            }
            DomProp::SrcDoc => {
                if self.node_explicit_own_property_overrides_dom_property(node, "srcdoc") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "srcdoc",
                    )? {
                        value
                    } else {
                        Value::String(self.dom.attr(node, "srcdoc").unwrap_or_default())
                    }
                } else {
                    Value::String(self.dom.attr(node, "srcdoc").unwrap_or_default())
                }
            }
            DomProp::UseMap => {
                if self.node_explicit_own_property_overrides_dom_property(node, "useMap") {
                    let entries = self.node_expando_entries(node);
                    if let Some(value) = self.object_property_from_entries_with_getter(
                        &Value::Node(node),
                        &entries,
                        "useMap",
                    )? {
                        value
                    } else {
                        Value::String(self.dom.attr(node, "usemap").unwrap_or_default())
                    }
                } else {
                    Value::String(self.dom.attr(node, "usemap").unwrap_or_default())
                }
            }
            DomProp::AriaString(prop_name) => Value::String(
                self.dom
                    .attr(node, &Self::aria_property_to_attr_name(prop_name))
                    .unwrap_or_default(),
            ),
            DomProp::AriaElementRefSingle(prop_name) => self
                .resolve_aria_single_element_property(node, prop_name)
                .map(Value::Node)
                .unwrap_or(Value::Null),
            DomProp::AriaElementRefList(prop_name) => Self::new_static_node_list_value(
                self.resolve_aria_element_list_property(node, prop_name),
            ),
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
