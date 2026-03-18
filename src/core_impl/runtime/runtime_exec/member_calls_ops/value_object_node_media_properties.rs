use super::*;

impl Harness {
    pub(crate) fn node_media_property_value(&mut self, node: NodeId, key: &str) -> Result<Value> {
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
        let is_track = self.is_track_element(node);

        match key {
            "kind" => {
                if is_track {
                    Ok(Value::String(self.normalized_track_kind(node)))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "track" => {
                if is_track {
                    Ok(self.text_track_object_value(node))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "srclang" | "srcLang" => {
                if is_track {
                    Ok(Value::String(self.dom.attr(node, "srclang").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "label" => {
                if is_track {
                    Ok(Value::String(self.dom.attr(node, "label").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "default" => {
                if is_track {
                    Ok(Value::Bool(self.dom.attr(node, "default").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "readyState" => {
                if is_track || is_audio_or_video {
                    Ok(Value::Number(0))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "defaultMuted" => {
                if is_audio_or_video {
                    Ok(Value::Bool(self.dom.attr(node, "muted").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "autoplay" => {
                if is_audio_or_video {
                    Ok(Value::Bool(self.dom.attr(node, "autoplay").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "controls" => {
                if is_audio_or_video {
                    Ok(Value::Bool(self.dom.attr(node, "controls").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "loop" => {
                if is_audio_or_video {
                    Ok(Value::Bool(self.dom.attr(node, "loop").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "muted" => {
                if is_audio_or_video {
                    Ok(Value::Bool(self.dom.attr(node, "muted").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
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
            "media" => Ok(Value::String(self.dom.attr(node, "media").unwrap_or_default())),
            "playsInline" | "playsinline" => {
                Ok(Value::Bool(self.dom.attr(node, "playsinline").is_some()))
            }
            "paused" => {
                if is_audio_or_video {
                    Ok(self.media_boolean_state_value(node, INTERNAL_MEDIA_PAUSED_KEY, true))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "ended" => {
                if is_audio_or_video {
                    Ok(Value::Bool(false))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "seeking" => {
                if is_audio_or_video {
                    Ok(Value::Bool(false))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "networkState" => {
                if is_audio_or_video {
                    let state = if self.resolve_media_src(node).is_empty() {
                        0
                    } else {
                        1
                    };
                    Ok(Value::Number(state))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "currentTime" => {
                if is_audio_or_video {
                    Ok(self.media_numeric_state_value(node, INTERNAL_MEDIA_CURRENT_TIME_KEY, 0.0))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "volume" => {
                if is_audio_or_video {
                    Ok(self.media_numeric_state_value(node, INTERNAL_MEDIA_VOLUME_KEY, 1.0))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "duration" => {
                if is_audio_or_video {
                    Ok(self.media_numeric_state_value(node, INTERNAL_MEDIA_DURATION_KEY, f64::NAN))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "playbackRate" => {
                if is_audio_or_video {
                    Ok(self.media_numeric_state_value(
                        node,
                        INTERNAL_MEDIA_PLAYBACK_RATE_KEY,
                        1.0,
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "defaultPlaybackRate" => {
                if is_audio_or_video {
                    Ok(self.media_numeric_state_value(
                        node,
                        INTERNAL_MEDIA_DEFAULT_PLAYBACK_RATE_KEY,
                        1.0,
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "textTracks" => {
                if is_audio_or_video {
                    Ok(self.media_text_tracks_live_list_value(node))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "buffered" => {
                if is_audio_or_video {
                    Ok(self.media_time_ranges_live_value(node, "buffered"))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "seekable" => {
                if is_audio_or_video {
                    Ok(self.media_time_ranges_live_value(node, "seekable"))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "played" => {
                if is_audio_or_video {
                    Ok(self.media_time_ranges_live_value(node, "played"))
                } else {
                    Ok(Value::Undefined)
                }
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
                    Ok(Value::String(self.reflected_url_attribute_or_empty(node, "data")))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "srcdoc" | "srcDoc" => {
                if is_iframe {
                    Ok(Value::String(self.dom.attr(node, "srcdoc").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "preload" => Ok(Value::String(self.dom.attr(node, "preload").unwrap_or_default())),
            "sizes" => Ok(Value::String(self.dom.attr(node, "sizes").unwrap_or_default())),
            "srcset" | "srcSet" => Ok(Value::String(
                self.dom.attr(node, "srcset").unwrap_or_default(),
            )),
            "useMap" | "usemap" => {
                if is_img || is_object {
                    Ok(Value::String(self.dom.attr(node, "usemap").unwrap_or_default()))
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
