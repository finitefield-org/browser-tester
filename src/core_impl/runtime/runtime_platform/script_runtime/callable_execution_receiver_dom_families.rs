use super::*;

impl Harness {
    pub(crate) fn execute_receiver_builtin_dom_family(
        &mut self,
        family: &str,
        member: &str,
        receiver: &Value,
        args: &[Value],
    ) -> Result<Option<Value>> {
        let result = match family {
            "worker" => {
                let Value::Object(worker) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                let normalized_options = match args.get(1) {
                    Some(Value::Array(values)) => Some(Self::new_object_value(vec![(
                        "transfer".to_string(),
                        Value::Array(values.clone()),
                    )])),
                    Some(other) => Some(other.clone()),
                    None => None,
                };
                Some(match member {
                    "postMessage" => {
                        if args.len() > 2 {
                            return Err(Error::ScriptRuntime(
                                "Worker.postMessage supports up to two arguments".into(),
                            ));
                        }
                        let data = args.first().cloned().unwrap_or(Value::Undefined);
                        if Self::worker_is_terminated_object(worker) {
                            return Ok(Some(Value::Undefined));
                        }
                        let data = Self::structured_clone_value_with_options(
                            &data,
                            normalized_options.as_ref(),
                        )?;
                        let worker_global = Self::worker_global_from_object(worker)?;
                        let worker_global_value = Value::Object(worker_global.clone());
                        self.queue_worker_message_microtask(
                            worker,
                            &worker_global,
                            worker_global_value,
                            data,
                        );
                        Value::Undefined
                    }
                    "terminate" => {
                        Self::worker_global_from_object(worker)?;
                        Self::worker_set_terminated_object(worker, true);
                        Value::Undefined
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Worker method: {member}"
                        )));
                    }
                })
            }
            "location" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_location_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(match member {
                    "assign" => {
                        let Some(url) = args.first() else {
                            return Err(Error::ScriptRuntime(
                                "location.assign requires exactly one argument".into(),
                            ));
                        };
                        self.navigate_location(&url.as_string(), LocationNavigationKind::Assign)?;
                        Value::Undefined
                    }
                    "reload" => {
                        self.reload_location()?;
                        Value::Undefined
                    }
                    "replace" => {
                        let Some(url) = args.first() else {
                            return Err(Error::ScriptRuntime(
                                "location.replace requires exactly one argument".into(),
                            ));
                        };
                        self.navigate_location(&url.as_string(), LocationNavigationKind::Replace)?;
                        Value::Undefined
                    }
                    "toString" => Value::String(self.document_url.clone()),
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Location method: {member}"
                        )));
                    }
                })
            }
            "image_bitmap" => {
                let object = Self::image_bitmap_receiver_object(Some(receiver))?;
                Some(match member {
                    "width_get" => {
                        let entries = object.borrow();
                        Self::object_get_entry(&entries, INTERNAL_IMAGE_BITMAP_WIDTH_KEY)
                            .unwrap_or(Value::Number(0))
                    }
                    "height_get" => {
                        let entries = object.borrow();
                        Self::object_get_entry(&entries, INTERNAL_IMAGE_BITMAP_HEIGHT_KEY)
                            .unwrap_or(Value::Number(0))
                    }
                    "close" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime("close takes no arguments".into()));
                        }
                        let mut entries = object.borrow_mut();
                        Self::object_set_entry(
                            &mut entries,
                            INTERNAL_IMAGE_BITMAP_WIDTH_KEY.to_string(),
                            Value::Number(0),
                        );
                        Self::object_set_entry(
                            &mut entries,
                            INTERNAL_IMAGE_BITMAP_HEIGHT_KEY.to_string(),
                            Value::Number(0),
                        );
                        Value::Undefined
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported ImageBitmap method: {member}"
                        )));
                    }
                })
            }
            "text_track" => {
                let (object, node) = Self::text_track_receiver_object_and_node(Some(receiver))?;
                Some(match member {
                    "id_get" => Value::String(self.dom.attr(node, "id").unwrap_or_default()),
                    "kind_get" => Value::String(self.normalized_track_kind(node)),
                    "label_get" => Value::String(self.dom.attr(node, "label").unwrap_or_default()),
                    "language_get" => {
                        Value::String(self.dom.attr(node, "srclang").unwrap_or_default())
                    }
                    "mode_get" => {
                        let entries = object.borrow();
                        Self::object_get_entry(&entries, INTERNAL_TEXT_TRACK_MODE_KEY)
                            .unwrap_or_else(|| Value::String("disabled".to_string()))
                    }
                    "mode_set" => {
                        let next_mode = args
                            .first()
                            .cloned()
                            .unwrap_or(Value::Undefined)
                            .as_string();
                        if matches!(next_mode.as_str(), "disabled" | "hidden" | "showing") {
                            Self::object_set_entry(
                                &mut object.borrow_mut(),
                                INTERNAL_TEXT_TRACK_MODE_KEY.to_string(),
                                Value::String(next_mode),
                            );
                        }
                        Value::Undefined
                    }
                    "cues_get" | "active_cues_get" => Value::Null,
                    "in_band_metadata_track_dispatch_type_get" => Value::String(String::new()),
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported TextTrack method: {member}"
                        )));
                    }
                })
            }
            "time_ranges" => {
                let (_object, owner, kind) =
                    Self::time_ranges_receiver_object_and_state(Some(receiver))?;
                let ranges = self.media_time_ranges_snapshot(owner, &kind);
                Some(match member {
                    "length_get" => Value::Number(ranges.len() as i64),
                    "start" | "end" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(format!(
                                "{member} requires exactly one index argument"
                            )));
                        }
                        let index = Self::value_to_i64(&args[0]);
                        if index < 0 || (index as usize) >= ranges.len() {
                            return Err(Error::ScriptRuntime(format!(
                                "TimeRanges.{member} index out of range"
                            )));
                        }
                        let (start, end) = ranges[index as usize];
                        Self::number_value(if member == "start" { start } else { end })
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported TimeRanges method: {member}"
                        )));
                    }
                })
            }
            "animation" => {
                let object = Self::animation_receiver_object(Some(receiver))?;
                Some(match member {
                    "cancel" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime("cancel takes no arguments".into()));
                        }
                        let mut entries = object.borrow_mut();
                        Self::object_set_entry(
                            &mut entries,
                            "playState".to_string(),
                            Value::String("idle".to_string()),
                        );
                        Self::object_set_entry(
                            &mut entries,
                            "currentTime".to_string(),
                            Value::Null,
                        );
                        Self::object_set_entry(&mut entries, "startTime".to_string(), Value::Null);
                        Self::object_set_entry(
                            &mut entries,
                            "pending".to_string(),
                            Value::Bool(false),
                        );
                        Value::Undefined
                    }
                    "finish" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime("finish takes no arguments".into()));
                        }
                        let mut entries = object.borrow_mut();
                        Self::object_set_entry(
                            &mut entries,
                            "playState".to_string(),
                            Value::String("finished".to_string()),
                        );
                        Self::object_set_entry(
                            &mut entries,
                            "pending".to_string(),
                            Value::Bool(false),
                        );
                        Value::Undefined
                    }
                    "pause" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime("pause takes no arguments".into()));
                        }
                        let mut entries = object.borrow_mut();
                        Self::object_set_entry(
                            &mut entries,
                            "playState".to_string(),
                            Value::String("paused".to_string()),
                        );
                        Self::object_set_entry(
                            &mut entries,
                            "pending".to_string(),
                            Value::Bool(false),
                        );
                        Value::Undefined
                    }
                    "play" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime("play takes no arguments".into()));
                        }
                        let mut entries = object.borrow_mut();
                        Self::object_set_entry(
                            &mut entries,
                            "playState".to_string(),
                            Value::String("running".to_string()),
                        );
                        if matches!(
                            Self::object_get_entry(&entries, "currentTime"),
                            Some(Value::Null)
                        ) {
                            Self::object_set_entry(
                                &mut entries,
                                "currentTime".to_string(),
                                Value::Number(0),
                            );
                        }
                        if matches!(
                            Self::object_get_entry(&entries, "startTime"),
                            Some(Value::Null)
                        ) {
                            Self::object_set_entry(
                                &mut entries,
                                "startTime".to_string(),
                                Value::Number(0),
                            );
                        }
                        Self::object_set_entry(
                            &mut entries,
                            "pending".to_string(),
                            Value::Bool(false),
                        );
                        Value::Undefined
                    }
                    "reverse" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime("reverse takes no arguments".into()));
                        }
                        let mut entries = object.borrow_mut();
                        Self::object_set_entry(
                            &mut entries,
                            "playState".to_string(),
                            Value::String("running".to_string()),
                        );
                        Self::object_set_entry(
                            &mut entries,
                            "pending".to_string(),
                            Value::Bool(false),
                        );
                        Value::Undefined
                    }
                    "updatePlaybackRate" => {
                        if args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "updatePlaybackRate requires exactly one argument".into(),
                            ));
                        }
                        let mut entries = object.borrow_mut();
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        let playback_rate = match value {
                            Value::Number(_) | Value::Float(_) => value,
                            _ => Self::number_value(
                                Self::coerce_number_for_number_constructor(&value),
                            ),
                        };
                        Self::object_set_entry(
                            &mut entries,
                            "playbackRate".to_string(),
                            playback_rate,
                        );
                        Value::Undefined
                    }
                    "commitStyles" | "persist" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(format!(
                                "{member} takes no arguments"
                            )));
                        }
                        Value::Undefined
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported Animation method: {member}"
                        )));
                    }
                })
            }
            "radio_node_list" => {
                let Value::NodeList(nodes) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::node_list_is_radio_node_list(nodes) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(match member {
                    "value_get" => Value::String(self.radio_node_list_value_string(nodes)?),
                    "value_set" => {
                        let next_value = args.first().cloned().unwrap_or(Value::Undefined);
                        self.set_radio_node_list_value(nodes, next_value.as_string().as_str())?;
                        Value::Undefined
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported RadioNodeList method: {member}"
                        )));
                    }
                })
            }
            "html_form" => {
                let Value::Node(node) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                let is_form = self
                    .dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("form"));
                if !is_form {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(match member {
                    "submit" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime("submit takes no arguments".into()));
                        }
                        self.with_script_env(|this, env| {
                            this.submit_form_with_env(*node, env)?;
                            Ok(Value::Undefined)
                        })?
                    }
                    "requestSubmit" => {
                        if args.len() > 1 {
                            return Err(Error::ScriptRuntime(
                                "requestSubmit takes at most one argument".into(),
                            ));
                        }
                        let submitter = args.first().cloned();
                        self.with_script_env(|this, env| {
                            this.request_submit_form_with_env(*node, submitter, env)?;
                            Ok(Value::Undefined)
                        })?
                    }
                    "reset" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime("reset takes no arguments".into()));
                        }
                        self.with_script_env(|this, env| {
                            this.reset_form_with_env(*node, env)?;
                            Ok(Value::Undefined)
                        })?
                    }
                    "checkValidity" | "reportValidity" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime(format!(
                                "{member} takes no arguments"
                            )));
                        }
                        self.with_script_env(|this, env| {
                            let valid = this.validate_form_submission_with_env(*node, env)?;
                            Ok(Value::Bool(valid))
                        })?
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported HTMLFormElement method: {member}"
                        )));
                    }
                })
            }
            "html_media" => {
                let Value::Node(node) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                let is_media = self.dom.tag_name(*node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("audio") || tag.eq_ignore_ascii_case("video")
                });
                if !is_media {
                    return Err(Self::incompatible_receiver_error(family));
                }
                let media_boolean_state = |this: &Self, key: &str, default: bool| match this
                    .dom_runtime
                    .node_expando_props
                    .get(&(*node, key.to_string()))
                {
                    Some(Value::Bool(value)) => *value,
                    Some(value) => value.truthy(),
                    None => default,
                };
                let media_numeric_state = |this: &Self, key: &str, default: f64| match this
                    .dom_runtime
                    .node_expando_props
                    .get(&(*node, key.to_string()))
                {
                    Some(Value::Number(value)) => *value as f64,
                    Some(Value::Float(value)) => *value,
                    Some(value) => Self::coerce_number_for_number_constructor(value),
                    None => default,
                };
                Some(match member {
                    "play" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime("play takes no arguments".into()));
                        }
                        let was_paused =
                            media_boolean_state(self, INTERNAL_MEDIA_PAUSED_KEY, true);
                        self.set_media_boolean_state_value(*node, INTERNAL_MEDIA_PAUSED_KEY, false);
                        if was_paused {
                            self.with_script_env(|this, env| {
                                let _ = this.dispatch_event_with_options(
                                    *node, "play", env, true, false, false, None, None, None,
                                )?;
                                let _ = this.dispatch_event_with_options(
                                    *node, "playing", env, true, false, false, None, None, None,
                                )?;
                                Ok(())
                            })?;
                        }
                        Value::Promise(
                            self.promise_resolve_value_as_promise(Value::Undefined)?,
                        )
                    }
                    "pause" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime("pause takes no arguments".into()));
                        }
                        let was_paused =
                            media_boolean_state(self, INTERNAL_MEDIA_PAUSED_KEY, true);
                        self.set_media_boolean_state_value(*node, INTERNAL_MEDIA_PAUSED_KEY, true);
                        if !was_paused {
                            self.with_script_env(|this, env| {
                                let _ = this.dispatch_event_with_options(
                                    *node, "pause", env, true, false, false, None, None, None,
                                )?;
                                Ok(())
                            })?;
                        }
                        Value::Undefined
                    }
                    "load" => {
                        if !args.is_empty() {
                            return Err(Error::ScriptRuntime("load takes no arguments".into()));
                        }
                        let had_current_src = !self.resolve_media_src(*node).is_empty();
                        let has_next_src = !self.resolve_media_src(*node).is_empty();
                        let had_current_time =
                            media_numeric_state(self, INTERNAL_MEDIA_CURRENT_TIME_KEY, 0.0) != 0.0;
                        self.set_media_boolean_state_value(*node, INTERNAL_MEDIA_PAUSED_KEY, true);
                        self.set_media_numeric_state_value(
                            *node,
                            INTERNAL_MEDIA_CURRENT_TIME_KEY,
                            &Value::Number(0),
                        );
                        self.with_script_env(|this, env| {
                            if had_current_src || had_current_time {
                                let _ = this.dispatch_event_with_options(
                                    *node, "emptied", env, true, false, false, None, None, None,
                                )?;
                            }
                            let _ = this.dispatch_event_with_options(
                                *node,
                                "loadstart",
                                env,
                                true,
                                false,
                                false,
                                None,
                                None,
                                None,
                            )?;
                            if has_next_src {
                                for event_type in [
                                    "durationchange",
                                    "loadedmetadata",
                                    "loadeddata",
                                    "canplay",
                                    "canplaythrough",
                                ] {
                                    let _ = this.dispatch_event_with_options(
                                        *node, event_type, env, true, false, false, None, None, None,
                                    )?;
                                }
                            }
                            Ok(())
                        })?;
                        Value::Undefined
                    }
                    "canPlayType" => {
                        let mime = args.first().map(Value::as_string).unwrap_or_default();
                        let normalized = mime.trim().to_ascii_lowercase();
                        let essence = normalized.split(';').next().unwrap_or("").trim();
                        let is_token = |part: &str| {
                            !part.is_empty()
                                && part.bytes().all(|byte| {
                                    byte.is_ascii_alphanumeric()
                                        || matches!(
                                            byte,
                                            b'!' | b'#'
                                                | b'$'
                                                | b'&'
                                                | b'^'
                                                | b'_'
                                                | b'.'
                                                | b'+'
                                                | b'-'
                                        )
                                })
                        };
                        let can_play = if essence.matches('/').count() == 1 {
                            if let Some((major, minor)) = essence.split_once('/') {
                                if matches!(major, "audio" | "video")
                                    && is_token(major)
                                    && is_token(minor)
                                {
                                    "maybe"
                                } else {
                                    ""
                                }
                            } else {
                                ""
                            }
                        } else {
                            ""
                        };
                        Value::String(can_play.to_string())
                    }
                    "fastSeek" => {
                        let Some(target_time) = args.first() else {
                            return Err(Error::ScriptRuntime(
                                "fastSeek requires an argument".into(),
                            ));
                        };
                        self.set_media_numeric_state_value(
                            *node,
                            INTERNAL_MEDIA_CURRENT_TIME_KEY,
                            target_time,
                        );
                        self.with_script_env(|this, env| {
                            let _ = this.dispatch_event_with_options(
                                *node, "seeking", env, true, false, false, None, None, None,
                            )?;
                            let _ = this.dispatch_event_with_options(
                                *node, "seeked", env, true, false, false, None, None, None,
                            )?;
                            Ok(())
                        })?;
                        Value::Undefined
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(format!(
                            "unsupported HTMLMediaElement method: {member}"
                        )));
                    }
                })
            }
            _ => None,
        };
        Ok(result)
    }
}
