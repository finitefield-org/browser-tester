use super::*;

impl Harness {
    pub(crate) fn new_event_object_from_constructor_args(
        &mut self,
        constructor_name: &str,
        args: &[Value],
        include_detail: bool,
        include_keyboard_fields: bool,
        include_wheel_fields: bool,
        include_navigate_fields: bool,
        include_pointer_fields: bool,
        include_hash_change_fields: bool,
        include_error_fields: bool,
        include_before_unload_fields: bool,
    ) -> Result<Value> {
        if args.is_empty() || args.len() > 2 {
            return Err(Error::ScriptRuntime(format!(
                "{constructor_name} constructor supports one or two arguments"
            )));
        }
        let event_type = args[0].as_string();
        if event_type.is_empty() {
            return Err(Error::ScriptRuntime(format!(
                "{constructor_name} constructor requires a non-empty event type"
            )));
        }

        let mut bubbles = false;
        let mut cancelable = false;
        let mut detail = if include_detail {
            Some(Value::Null)
        } else {
            None
        };
        let mut key = String::new();
        let mut code = String::new();
        let mut location = 0i64;
        let mut ctrl_key = false;
        let mut meta_key = false;
        let mut shift_key = false;
        let mut alt_key = false;
        let mut repeat = false;
        let mut is_composing = false;
        let mut delta_x = 0.0f64;
        let mut delta_y = 0.0f64;
        let mut delta_z = 0.0f64;
        let mut delta_mode = 0i64;
        let mut pointer_id = 0i64;
        let mut pointer_width = 1.0f64;
        let mut pointer_height = 1.0f64;
        let mut pointer_pressure = 0.0f64;
        let mut pointer_tangential_pressure = 0.0f64;
        let mut pointer_tilt_x = 0i64;
        let mut pointer_tilt_y = 0i64;
        let mut pointer_twist = 0i64;
        let mut pointer_type = String::new();
        let mut pointer_is_primary = false;
        let mut pointer_altitude_angle = 0.0f64;
        let mut pointer_azimuth_angle = 0.0f64;
        let mut pointer_persistent_device_id = 0i64;
        let mut can_intercept = false;
        let mut destination = Value::Null;
        let mut download_request = Value::Null;
        let mut form_data = Value::Null;
        let mut hash_change = false;
        let mut has_ua_visual_transition = false;
        let mut info = Value::Undefined;
        let mut navigation_type = "push".to_string();
        let mut signal = Self::new_navigate_event_default_signal_value();
        let mut source_element = Value::Null;
        let mut user_initiated = false;
        let mut hash_change_old_url = String::new();
        let mut hash_change_new_url = String::new();
        let mut error_message = String::new();
        let mut error_filename = String::new();
        let mut error_lineno = 0i64;
        let mut error_colno = 0i64;
        let mut error_value = Value::Null;
        let mut before_unload_return_value = String::new();
        if let Some(options) = args.get(1) {
            match options {
                Value::Null | Value::Undefined => {}
                Value::Object(entries) => {
                    let entries = entries.borrow();
                    bubbles = Self::object_get_entry(&entries, "bubbles")
                        .is_some_and(|value| value.truthy());
                    cancelable = Self::object_get_entry(&entries, "cancelable")
                        .is_some_and(|value| value.truthy());
                    if include_detail {
                        detail =
                            Some(Self::object_get_entry(&entries, "detail").unwrap_or(Value::Null));
                    }
                    if include_keyboard_fields {
                        key = Self::object_get_entry(&entries, "key")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        code = Self::object_get_entry(&entries, "code")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        location = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "location")
                                .unwrap_or(Value::Number(0)),
                        );
                        ctrl_key = Self::object_get_entry(&entries, "ctrlKey")
                            .is_some_and(|value| value.truthy());
                        meta_key = Self::object_get_entry(&entries, "metaKey")
                            .is_some_and(|value| value.truthy());
                        shift_key = Self::object_get_entry(&entries, "shiftKey")
                            .is_some_and(|value| value.truthy());
                        alt_key = Self::object_get_entry(&entries, "altKey")
                            .is_some_and(|value| value.truthy());
                        repeat = Self::object_get_entry(&entries, "repeat")
                            .is_some_and(|value| value.truthy());
                        is_composing = Self::object_get_entry(&entries, "isComposing")
                            .is_some_and(|value| value.truthy());
                    }
                    if include_wheel_fields {
                        delta_x = Self::object_get_entry(&entries, "deltaX")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        delta_y = Self::object_get_entry(&entries, "deltaY")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        delta_z = Self::object_get_entry(&entries, "deltaZ")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        delta_mode = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "deltaMode")
                                .unwrap_or(Value::Number(0)),
                        );
                    }
                    if include_pointer_fields {
                        pointer_id = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "pointerId")
                                .unwrap_or(Value::Number(0)),
                        );
                        pointer_width = Self::object_get_entry(&entries, "width")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(1.0);
                        pointer_height = Self::object_get_entry(&entries, "height")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(1.0);
                        pointer_pressure = Self::object_get_entry(&entries, "pressure")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        pointer_tangential_pressure =
                            Self::object_get_entry(&entries, "tangentialPressure")
                                .map(|value| Self::coerce_number_for_global(&value))
                                .unwrap_or(0.0);
                        pointer_tilt_x = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "tiltX").unwrap_or(Value::Number(0)),
                        );
                        pointer_tilt_y = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "tiltY").unwrap_or(Value::Number(0)),
                        );
                        pointer_twist = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "twist").unwrap_or(Value::Number(0)),
                        );
                        pointer_type = Self::object_get_entry(&entries, "pointerType")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        pointer_is_primary = Self::object_get_entry(&entries, "isPrimary")
                            .is_some_and(|value| value.truthy());
                        pointer_altitude_angle = Self::object_get_entry(&entries, "altitudeAngle")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        pointer_azimuth_angle = Self::object_get_entry(&entries, "azimuthAngle")
                            .map(|value| Self::coerce_number_for_global(&value))
                            .unwrap_or(0.0);
                        pointer_persistent_device_id = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "persistentDeviceId")
                                .unwrap_or(Value::Number(0)),
                        );
                    }
                    if include_navigate_fields {
                        can_intercept = Self::object_get_entry(&entries, "canIntercept")
                            .is_some_and(|value| value.truthy());
                        destination =
                            Self::object_get_entry(&entries, "destination").unwrap_or(Value::Null);
                        download_request = Self::object_get_entry(&entries, "downloadRequest")
                            .unwrap_or(Value::Null);
                        form_data =
                            Self::object_get_entry(&entries, "formData").unwrap_or(Value::Null);
                        hash_change = Self::object_get_entry(&entries, "hashChange")
                            .is_some_and(|value| value.truthy());
                        has_ua_visual_transition =
                            Self::object_get_entry(&entries, "hasUAVisualTransition")
                                .is_some_and(|value| value.truthy());
                        info = Self::object_get_entry(&entries, "info").unwrap_or(Value::Undefined);
                        if let Some(value) = Self::object_get_entry(&entries, "navigationType") {
                            navigation_type = value.as_string();
                        }
                        if let Some(value) = Self::object_get_entry(&entries, "signal") {
                            signal = value;
                        }
                        source_element = Self::object_get_entry(&entries, "sourceElement")
                            .unwrap_or(Value::Null);
                        user_initiated = Self::object_get_entry(&entries, "userInitiated")
                            .is_some_and(|value| value.truthy());
                    }
                    if include_before_unload_fields {
                        before_unload_return_value =
                            Self::object_get_entry(&entries, "returnValue")
                                .map(|value| value.as_string())
                                .unwrap_or_default();
                    }
                    if include_hash_change_fields {
                        hash_change_old_url = Self::object_get_entry(&entries, "oldURL")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        hash_change_new_url = Self::object_get_entry(&entries, "newURL")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                    }
                    if include_error_fields {
                        error_message = Self::object_get_entry(&entries, "message")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        error_filename = Self::object_get_entry(&entries, "filename")
                            .map(|value| value.as_string())
                            .unwrap_or_default();
                        error_lineno = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "lineno").unwrap_or(Value::Number(0)),
                        );
                        error_colno = Self::value_to_i64(
                            &Self::object_get_entry(&entries, "colno").unwrap_or(Value::Number(0)),
                        );
                        error_value =
                            Self::object_get_entry(&entries, "error").unwrap_or(Value::Null);
                    }
                }
                _ => {
                    return Err(Error::ScriptRuntime(format!(
                        "{constructor_name} constructor options argument must be an object"
                    )));
                }
            }
        }

        let default_prevented =
            cancelable && include_before_unload_fields && !before_unload_return_value.is_empty();
        let event_type_value = event_type.clone();
        let mut entries = vec![
            (INTERNAL_EVENT_OBJECT_KEY.to_string(), Value::Bool(true)),
            ("type".to_string(), Value::String(event_type)),
            ("bubbles".to_string(), Value::Bool(bubbles)),
            ("cancelable".to_string(), Value::Bool(cancelable)),
            (
                "defaultPrevented".to_string(),
                Value::Bool(default_prevented),
            ),
            ("isTrusted".to_string(), Value::Bool(false)),
            ("eventPhase".to_string(), Value::Number(0)),
            (
                "timeStamp".to_string(),
                Value::Number(self.scheduler.now_ms),
            ),
            ("target".to_string(), Value::Null),
            ("currentTarget".to_string(), Value::Null),
            (
                "preventDefault".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "stopPropagation".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "stopImmediatePropagation".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ];
        if let Some(detail) = detail {
            entries.push(("detail".to_string(), detail));
        }
        if include_keyboard_fields {
            let key_code = Self::keyboard_key_code_for_key(&key);
            let char_code = Self::keyboard_char_code_for_event(&event_type_value, &key);
            entries.push((
                INTERNAL_KEYBOARD_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("key".to_string(), Value::String(key.clone())));
            entries.push(("code".to_string(), Value::String(code)));
            entries.push(("location".to_string(), Value::Number(location)));
            entries.push(("ctrlKey".to_string(), Value::Bool(ctrl_key)));
            entries.push(("metaKey".to_string(), Value::Bool(meta_key)));
            entries.push(("shiftKey".to_string(), Value::Bool(shift_key)));
            entries.push(("altKey".to_string(), Value::Bool(alt_key)));
            entries.push(("repeat".to_string(), Value::Bool(repeat)));
            entries.push(("isComposing".to_string(), Value::Bool(is_composing)));
            entries.push(("keyCode".to_string(), Value::Number(key_code)));
            entries.push(("charCode".to_string(), Value::Number(char_code)));
            entries.push((
                "keyIdentifier".to_string(),
                Value::String(if key.is_empty() {
                    "Unidentified".to_string()
                } else {
                    key
                }),
            ));
            entries.push((
                "getModifierState".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
        }
        if include_wheel_fields {
            entries.push((
                INTERNAL_WHEEL_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("deltaX".to_string(), Value::Float(delta_x)));
            entries.push(("deltaY".to_string(), Value::Float(delta_y)));
            entries.push(("deltaZ".to_string(), Value::Float(delta_z)));
            entries.push(("deltaMode".to_string(), Value::Number(delta_mode)));
        }
        if include_pointer_fields {
            entries.push((
                INTERNAL_POINTER_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("pointerId".to_string(), Value::Number(pointer_id)));
            entries.push(("width".to_string(), Value::Float(pointer_width)));
            entries.push(("height".to_string(), Value::Float(pointer_height)));
            entries.push(("pressure".to_string(), Value::Float(pointer_pressure)));
            entries.push((
                "tangentialPressure".to_string(),
                Value::Float(pointer_tangential_pressure),
            ));
            entries.push(("tiltX".to_string(), Value::Number(pointer_tilt_x)));
            entries.push(("tiltY".to_string(), Value::Number(pointer_tilt_y)));
            entries.push(("twist".to_string(), Value::Number(pointer_twist)));
            entries.push(("pointerType".to_string(), Value::String(pointer_type)));
            entries.push(("isPrimary".to_string(), Value::Bool(pointer_is_primary)));
            entries.push((
                "altitudeAngle".to_string(),
                Value::Float(pointer_altitude_angle),
            ));
            entries.push((
                "azimuthAngle".to_string(),
                Value::Float(pointer_azimuth_angle),
            ));
            entries.push((
                "persistentDeviceId".to_string(),
                Value::Number(pointer_persistent_device_id),
            ));
            entries.push((
                "getCoalescedEvents".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
            entries.push((
                "getPredictedEvents".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
        }
        if include_navigate_fields {
            entries.push((
                INTERNAL_NAVIGATE_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("canIntercept".to_string(), Value::Bool(can_intercept)));
            entries.push(("destination".to_string(), destination));
            entries.push(("downloadRequest".to_string(), download_request));
            entries.push(("formData".to_string(), form_data));
            entries.push(("hashChange".to_string(), Value::Bool(hash_change)));
            entries.push((
                "hasUAVisualTransition".to_string(),
                Value::Bool(has_ua_visual_transition),
            ));
            entries.push(("info".to_string(), info));
            entries.push(("navigationType".to_string(), Value::String(navigation_type)));
            entries.push(("signal".to_string(), signal));
            entries.push(("sourceElement".to_string(), source_element));
            entries.push(("userInitiated".to_string(), Value::Bool(user_initiated)));
            entries.push((
                "intercept".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
            entries.push((
                "scroll".to_string(),
                Self::new_builtin_placeholder_function(),
            ));
        }
        if include_before_unload_fields {
            entries.push((
                INTERNAL_BEFORE_UNLOAD_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push((
                "returnValue".to_string(),
                Value::String(before_unload_return_value),
            ));
        }
        if include_hash_change_fields {
            entries.push((
                INTERNAL_HASH_CHANGE_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("oldURL".to_string(), Value::String(hash_change_old_url)));
            entries.push(("newURL".to_string(), Value::String(hash_change_new_url)));
        }
        if include_error_fields {
            entries.push((
                INTERNAL_ERROR_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ));
            entries.push(("message".to_string(), Value::String(error_message)));
            entries.push(("filename".to_string(), Value::String(error_filename)));
            entries.push(("lineno".to_string(), Value::Number(error_lineno)));
            entries.push(("colno".to_string(), Value::Number(error_colno)));
            entries.push(("error".to_string(), error_value));
        }
        Self::mark_object_properties_non_enumerable(
            &mut entries,
            &[
                "preventDefault",
                "stopPropagation",
                "stopImmediatePropagation",
            ],
        );
        if include_keyboard_fields {
            Self::mark_object_properties_non_enumerable(&mut entries, &["getModifierState"]);
        }
        if include_pointer_fields {
            Self::mark_object_properties_non_enumerable(
                &mut entries,
                &["getCoalescedEvents", "getPredictedEvents"],
            );
        }
        if include_navigate_fields {
            Self::mark_object_properties_non_enumerable(&mut entries, &["intercept", "scroll"]);
        }
        Ok(Self::new_object_value(entries))
    }
}
