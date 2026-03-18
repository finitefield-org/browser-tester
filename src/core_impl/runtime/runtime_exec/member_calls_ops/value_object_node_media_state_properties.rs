use super::*;

impl Harness {
    pub(crate) fn is_node_media_state_property_key(key: &str) -> bool {
        matches!(
            key,
            "kind"
                | "track"
                | "srclang"
                | "srcLang"
                | "label"
                | "default"
                | "readyState"
                | "defaultMuted"
                | "autoplay"
                | "controls"
                | "loop"
                | "muted"
                | "paused"
                | "ended"
                | "seeking"
                | "networkState"
                | "currentTime"
                | "volume"
                | "duration"
                | "playbackRate"
                | "defaultPlaybackRate"
                | "textTracks"
                | "buffered"
                | "seekable"
                | "played"
        )
    }

    pub(crate) fn node_media_state_property_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Value> {
        let is_audio_or_video = self
            .dom
            .tag_name(node)
            .map(|tag| tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video"))
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
            _ => Ok(Value::Undefined),
        }
    }
}
