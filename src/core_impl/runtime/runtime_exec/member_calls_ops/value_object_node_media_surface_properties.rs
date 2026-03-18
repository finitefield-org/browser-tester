use super::*;

impl Harness {
    pub(crate) fn is_node_media_surface_property_key(key: &str) -> bool {
        matches!(
            key,
            "controlsList"
                | "controlslist"
                | "crossOrigin"
                | "crossorigin"
                | "disableRemotePlayback"
                | "disableremoteplayback"
                | "disablePictureInPicture"
                | "disablepictureinpicture"
                | "media"
                | "playsInline"
                | "playsinline"
                | "currentSrc"
                | "currentsrc"
                | "complete"
                | "naturalWidth"
                | "naturalHeight"
                | "src"
                | "poster"
                | "attributionSrc"
                | "attributionsrc"
                | "data"
                | "srcdoc"
                | "srcDoc"
                | "preload"
                | "sizes"
                | "srcset"
                | "srcSet"
                | "useMap"
                | "usemap"
                | "width"
                | "height"
                | "mozOpaque"
                | "mozopaque"
                | "mozPrintCallback"
                | "mozprintcallback"
                | "captureStream"
                | "getContext"
                | "toDataURL"
                | "toBlob"
                | "transferControlToOffscreen"
        )
    }

    pub(crate) fn node_media_surface_property_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Value> {
        let is_canvas = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("canvas"))
            .unwrap_or(false);
        let is_audio_or_video = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video"))
            .unwrap_or(false);
        let is_img = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("img"))
            .unwrap_or(false);
        let is_object = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("object"))
            .unwrap_or(false);
        let is_iframe = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("iframe"))
            .unwrap_or(false);

        match key {
            "controlsList" | "controlslist" => Ok(Value::String(
                self.dom.attr(node, "controlslist").unwrap_or_default(),
            )),
            "crossOrigin" | "crossorigin" => Ok(Value::String(
                self.dom.attr(node, "crossorigin").unwrap_or_default(),
            )),
            "disableRemotePlayback" | "disableremoteplayback" => Ok(Value::Bool(
                self.dom.attr(node, "disableremoteplayback").is_some(),
            )),
            "disablePictureInPicture" | "disablepictureinpicture" => Ok(Value::Bool(
                self.dom.attr(node, "disablepictureinpicture").is_some(),
            )),
            "media" => Ok(Value::String(
                self.dom.attr(node, "media").unwrap_or_default(),
            )),
            "playsInline" | "playsinline" => {
                Ok(Value::Bool(self.dom.attr(node, "playsinline").is_some()))
            }
            "currentSrc" | "currentsrc" => {
                if is_img || is_audio_or_video {
                    Ok(Value::String(self.resolve_media_src(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "complete" => {
                if is_img {
                    Ok(Value::Bool(true))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "naturalWidth" | "naturalHeight" => {
                if is_img {
                    Ok(Value::Number(self.image_natural_dimension_value(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "src" => Ok(Value::String(self.resolve_media_src(node))),
            "poster" => Ok(Value::String(
                self.reflected_url_attribute_or_empty(node, "poster"),
            )),
            "attributionSrc" | "attributionsrc" => Ok(Value::String(
                self.dom.attr(node, "attributionsrc").unwrap_or_default(),
            )),
            "data" => {
                if is_object {
                    Ok(Value::String(
                        self.reflected_url_attribute_or_empty(node, "data"),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "srcdoc" | "srcDoc" => {
                if is_iframe {
                    Ok(Value::String(
                        self.dom.attr(node, "srcdoc").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "preload" => Ok(Value::String(
                self.dom.attr(node, "preload").unwrap_or_default(),
            )),
            "sizes" => Ok(Value::String(
                self.dom.attr(node, "sizes").unwrap_or_default(),
            )),
            "srcset" | "srcSet" => Ok(Value::String(
                self.dom.attr(node, "srcset").unwrap_or_default(),
            )),
            "useMap" | "usemap" => {
                if is_img || is_object {
                    Ok(Value::String(
                        self.dom.attr(node, "usemap").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "width" => Ok(Value::Number(self.canvas_dimension_value(node, "width"))),
            "height" => Ok(Value::Number(self.canvas_dimension_value(node, "height"))),
            "mozOpaque" | "mozopaque" => {
                if is_canvas {
                    Ok(Value::Bool(self.dom.attr(node, "moz-opaque").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "mozPrintCallback" | "mozprintcallback" => {
                if is_canvas {
                    Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(node, key.to_string()))
                        .cloned()
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "captureStream"
            | "getContext"
            | "toDataURL"
            | "toBlob"
            | "transferControlToOffscreen" => {
                if is_canvas {
                    Ok(self
                        .dom_runtime
                        .node_expando_props
                        .get(&(node, key.to_string()))
                        .cloned()
                        .unwrap_or_else(Self::new_builtin_placeholder_function))
                } else {
                    Ok(Value::Undefined)
                }
            }
            _ => Ok(Value::Undefined),
        }
    }
}
