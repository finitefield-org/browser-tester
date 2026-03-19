use super::*;

impl Harness {
    pub(crate) fn try_execute_dom_assign_media_prop(
        &mut self,
        node: NodeId,
        prop: &DomProp,
        value: &Value,
        event: &mut EventState,
    ) -> Result<bool> {
        match prop {
            DomProp::CanvasWidth => self.set_canvas_dimension_value(node, "width", value)?,
            DomProp::CanvasHeight => self.set_canvas_dimension_value(node, "height", value)?,
            DomProp::AudioSrc => {
                if self.node_explicit_own_property_overrides_dom_property(node, "src") {
                    self.set_node_assignment_property(node, "src", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "src", &value.as_string())?;
                }
            }
            DomProp::AudioAutoplay => {
                if self.node_explicit_own_property_overrides_dom_property(node, "autoplay") {
                    self.set_node_assignment_property(
                        node,
                        "autoplay",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.set_reflected_boolean_attribute(node, "autoplay", value.truthy())?;
                }
            }
            DomProp::AudioControls => {
                if self.node_explicit_own_property_overrides_dom_property(node, "controls") {
                    self.set_node_assignment_property(
                        node,
                        "controls",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.set_reflected_boolean_attribute(node, "controls", value.truthy())?;
                }
            }
            DomProp::AudioControlsList => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["controlsList", "controlslist"],
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
                        .set_attr(node, "controlslist", &value.as_string())?;
                }
            }
            DomProp::AudioCrossOrigin => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["crossOrigin", "crossorigin"],
                ) {
                    self.set_node_assignment_property(
                        node,
                        shadow_key,
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "crossorigin", &value.as_string())?;
                }
            }
            DomProp::AudioDisableRemotePlayback => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["disableRemotePlayback", "disableremoteplayback"],
                ) {
                    self.set_node_assignment_property(
                        node,
                        shadow_key,
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.set_reflected_boolean_attribute(
                        node,
                        "disableremoteplayback",
                        value.truthy(),
                    )?;
                }
            }
            DomProp::VideoDisablePictureInPicture => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["disablePictureInPicture", "disablepictureinpicture"],
                ) {
                    self.set_node_assignment_property(
                        node,
                        shadow_key,
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.set_reflected_boolean_attribute(
                        node,
                        "disablepictureinpicture",
                        value.truthy(),
                    )?;
                }
            }
            DomProp::AudioLoop => {
                if self.node_explicit_own_property_overrides_dom_property(node, "loop") {
                    self.set_node_assignment_property(node, "loop", value.clone(), event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "loop", value.truthy())?;
                }
            }
            DomProp::AudioMuted => {
                if self.node_explicit_own_property_overrides_dom_property(node, "muted") {
                    self.set_node_assignment_property(node, "muted", value.clone(), event, false)?;
                } else {
                    self.set_reflected_boolean_attribute(node, "muted", value.truthy())?;
                }
            }
            DomProp::AudioPreload => {
                if self.node_explicit_own_property_overrides_dom_property(node, "preload") {
                    self.set_node_assignment_property(
                        node,
                        "preload",
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.dom.set_attr(node, "preload", &value.as_string())?;
                }
            }
            DomProp::VideoPlaysInline => {
                if let Some(shadow_key) = self.node_explicit_own_dom_property_shadow_key(
                    node,
                    &["playsInline", "playsinline"],
                ) {
                    self.set_node_assignment_property(
                        node,
                        shadow_key,
                        value.clone(),
                        event,
                        false,
                    )?;
                } else {
                    self.set_reflected_boolean_attribute(node, "playsinline", value.truthy())?;
                }
            }
            DomProp::VideoPoster => {
                if self.node_explicit_own_property_overrides_dom_property(node, "poster") {
                    self.set_node_assignment_property(node, "poster", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "poster", &value.as_string())?;
                }
            }
            DomProp::Data => {
                if self.node_explicit_own_property_overrides_dom_property(node, "data") {
                    self.set_node_assignment_property(node, "data", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "data", &value.as_string())?;
                }
            }
            DomProp::SrcDoc => {
                if self.node_explicit_own_property_overrides_dom_property(node, "srcdoc") {
                    self.set_node_assignment_property(node, "srcdoc", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "srcdoc", &value.as_string())?;
                }
            }
            DomProp::UseMap => {
                if self.node_explicit_own_property_overrides_dom_property(node, "useMap") {
                    self.set_node_assignment_property(node, "useMap", value.clone(), event, false)?;
                } else {
                    self.dom.set_attr(node, "usemap", &value.as_string())?;
                }
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
