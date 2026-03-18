use super::*;

impl Harness {
    pub(crate) fn resolve_event_target_object_for_query(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<Option<Rc<RefCell<ObjectValue>>>> {
        let value = match target {
            DomQuery::Var(name) => env.get(name).cloned(),
            DomQuery::VarPath { base, path } => {
                self.resolve_dom_query_var_path_value(base, path, env)?
            }
            _ => None,
        };
        let Some(Value::Object(entries)) = value else {
            return Ok(None);
        };
        if Self::is_event_target_object(&entries.borrow()) {
            return Ok(Some(entries));
        }
        Ok(None)
    }

    pub(crate) fn event_target_listener_node_id(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
    ) -> NodeId {
        let object_id = Rc::as_ptr(object) as usize;
        if let Some(node_id) = self
            .script_runtime
            .event_target_listener_nodes
            .get(&object_id)
            .copied()
        {
            return node_id;
        }

        let slot = self.script_runtime.next_event_target_listener_slot;
        self.script_runtime.next_event_target_listener_slot = slot.saturating_add(1);
        let node_id = NodeId(usize::MAX.saturating_sub(slot));
        self.script_runtime
            .event_target_listener_nodes
            .insert(object_id, node_id);
        node_id
    }

    pub(crate) fn event_dispatch_payload_from_value(
        &self,
        value: &Value,
    ) -> Result<(
        String,
        Option<Value>,
        bool,
        bool,
        Option<Rc<RefCell<ObjectValue>>>,
    )> {
        if let Value::Object(entries) = value {
            let entries = entries.borrow();
            if Self::is_event_object(&entries) {
                let event_type = Self::object_get_entry(&entries, "type")
                    .unwrap_or(Value::Undefined)
                    .as_string();
                let detail = Self::object_get_entry(&entries, "detail");
                let bubbles =
                    Self::object_get_entry(&entries, "bubbles").is_some_and(|value| value.truthy());
                let cancelable = Self::object_get_entry(&entries, "cancelable")
                    .is_some_and(|value| value.truthy());
                let object = if let Value::Object(object) = value {
                    Some(object.clone())
                } else {
                    None
                };
                return Ok((event_type, detail, bubbles, cancelable, object));
            }
        }
        Ok((value.as_string(), None, false, false, None))
    }

    fn apply_keyboard_event_payload_fields(
        event: &mut EventState,
        event_payload_object: Option<&Rc<RefCell<ObjectValue>>>,
    ) {
        let Some(event_payload_object) = event_payload_object else {
            return;
        };
        let entries = event_payload_object.borrow();
        if !Self::is_keyboard_event_object(&entries) {
            return;
        }
        event.key = Some(
            Self::object_get_entry(&entries, "key")
                .map(|value| value.as_string())
                .unwrap_or_default(),
        );
        event.code = Some(
            Self::object_get_entry(&entries, "code")
                .map(|value| value.as_string())
                .unwrap_or_default(),
        );
        event.location = Self::value_to_i64(
            &Self::object_get_entry(&entries, "location").unwrap_or(Value::Number(0)),
        );
        event.ctrl_key =
            Self::object_get_entry(&entries, "ctrlKey").is_some_and(|value| value.truthy());
        event.meta_key =
            Self::object_get_entry(&entries, "metaKey").is_some_and(|value| value.truthy());
        event.shift_key =
            Self::object_get_entry(&entries, "shiftKey").is_some_and(|value| value.truthy());
        event.alt_key =
            Self::object_get_entry(&entries, "altKey").is_some_and(|value| value.truthy());
        event.repeat =
            Self::object_get_entry(&entries, "repeat").is_some_and(|value| value.truthy());
        event.is_composing =
            Self::object_get_entry(&entries, "isComposing").is_some_and(|value| value.truthy());
    }

    fn apply_wheel_event_payload_fields(
        event: &mut EventState,
        event_payload_object: Option<&Rc<RefCell<ObjectValue>>>,
    ) {
        let Some(event_payload_object) = event_payload_object else {
            return;
        };
        let entries = event_payload_object.borrow();
        if !Self::is_wheel_event_object(&entries) {
            return;
        }
        event.delta_x = Self::object_get_entry(&entries, "deltaX")
            .map(|value| Self::coerce_number_for_global(&value))
            .unwrap_or(0.0);
        event.delta_y = Self::object_get_entry(&entries, "deltaY")
            .map(|value| Self::coerce_number_for_global(&value))
            .unwrap_or(0.0);
        event.delta_z = Self::object_get_entry(&entries, "deltaZ")
            .map(|value| Self::coerce_number_for_global(&value))
            .unwrap_or(0.0);
        event.delta_mode = Self::value_to_i64(
            &Self::object_get_entry(&entries, "deltaMode").unwrap_or(Value::Number(0)),
        );
    }

    fn apply_pointer_event_payload_fields(
        event: &mut EventState,
        event_payload_object: Option<&Rc<RefCell<ObjectValue>>>,
    ) {
        let Some(event_payload_object) = event_payload_object else {
            return;
        };
        let entries = event_payload_object.borrow();
        if !Self::is_pointer_event_object(&entries) {
            return;
        }
        event.pointer_id = Self::value_to_i64(
            &Self::object_get_entry(&entries, "pointerId").unwrap_or(Value::Number(0)),
        );
        event.pointer_width = Self::object_get_entry(&entries, "width")
            .map(|value| Self::coerce_number_for_global(&value))
            .unwrap_or(1.0);
        event.pointer_height = Self::object_get_entry(&entries, "height")
            .map(|value| Self::coerce_number_for_global(&value))
            .unwrap_or(1.0);
        event.pointer_pressure = Self::object_get_entry(&entries, "pressure")
            .map(|value| Self::coerce_number_for_global(&value))
            .unwrap_or(0.0);
        event.pointer_tangential_pressure = Self::object_get_entry(&entries, "tangentialPressure")
            .map(|value| Self::coerce_number_for_global(&value))
            .unwrap_or(0.0);
        event.pointer_tilt_x = Self::value_to_i64(
            &Self::object_get_entry(&entries, "tiltX").unwrap_or(Value::Number(0)),
        );
        event.pointer_tilt_y = Self::value_to_i64(
            &Self::object_get_entry(&entries, "tiltY").unwrap_or(Value::Number(0)),
        );
        event.pointer_twist = Self::value_to_i64(
            &Self::object_get_entry(&entries, "twist").unwrap_or(Value::Number(0)),
        );
        event.pointer_type = Self::object_get_entry(&entries, "pointerType")
            .map(|value| value.as_string())
            .unwrap_or_default();
        event.pointer_is_primary =
            Self::object_get_entry(&entries, "isPrimary").is_some_and(|value| value.truthy());
        event.pointer_altitude_angle = Self::object_get_entry(&entries, "altitudeAngle")
            .map(|value| Self::coerce_number_for_global(&value))
            .unwrap_or(0.0);
        event.pointer_azimuth_angle = Self::object_get_entry(&entries, "azimuthAngle")
            .map(|value| Self::coerce_number_for_global(&value))
            .unwrap_or(0.0);
        event.pointer_persistent_device_id = Self::value_to_i64(
            &Self::object_get_entry(&entries, "persistentDeviceId").unwrap_or(Value::Number(0)),
        );
    }

    fn apply_navigate_event_payload_fields(
        event: &mut EventState,
        event_payload_object: Option<&Rc<RefCell<ObjectValue>>>,
    ) {
        let Some(event_payload_object) = event_payload_object else {
            return;
        };
        let entries = event_payload_object.borrow();
        if !Self::is_navigate_event_object(&entries) {
            return;
        }

        event.navigate_can_intercept =
            Self::object_get_entry(&entries, "canIntercept").is_some_and(|value| value.truthy());
        event.navigate_destination =
            Some(Self::object_get_entry(&entries, "destination").unwrap_or(Value::Null));
        event.navigate_download_request =
            Some(Self::object_get_entry(&entries, "downloadRequest").unwrap_or(Value::Null));
        event.navigate_form_data =
            Some(Self::object_get_entry(&entries, "formData").unwrap_or(Value::Null));
        event.navigate_hash_change =
            Self::object_get_entry(&entries, "hashChange").is_some_and(|value| value.truthy());
        event.navigate_has_ua_visual_transition =
            Self::object_get_entry(&entries, "hasUAVisualTransition")
                .is_some_and(|value| value.truthy());
        event.navigate_info =
            Some(Self::object_get_entry(&entries, "info").unwrap_or(Value::Undefined));
        event.navigate_navigation_type = Some(
            Self::object_get_entry(&entries, "navigationType")
                .unwrap_or(Value::String("push".to_string()))
                .as_string(),
        );
        event.navigate_signal =
            Some(Self::object_get_entry(&entries, "signal").unwrap_or(Value::Null));
        event.navigate_source_element =
            Some(Self::object_get_entry(&entries, "sourceElement").unwrap_or(Value::Null));
        event.navigate_user_initiated =
            Self::object_get_entry(&entries, "userInitiated").is_some_and(|value| value.truthy());
    }

    fn apply_before_unload_event_payload_fields(
        event: &mut EventState,
        event_payload_object: Option<&Rc<RefCell<ObjectValue>>>,
    ) {
        let Some(event_payload_object) = event_payload_object else {
            return;
        };
        let entries = event_payload_object.borrow();
        if !Self::is_before_unload_event_object(&entries) {
            return;
        }

        event.before_unload_interface = true;
        event.before_unload_return_value = Self::object_get_entry(&entries, "returnValue")
            .map(|value| value.as_string())
            .unwrap_or_default();
        if event.cancelable && !event.before_unload_return_value.is_empty() {
            event.default_prevented = true;
        }
    }

    fn apply_hash_change_event_payload_fields(
        event: &mut EventState,
        event_payload_object: Option<&Rc<RefCell<ObjectValue>>>,
    ) {
        let Some(event_payload_object) = event_payload_object else {
            return;
        };
        let entries = event_payload_object.borrow();
        if !Self::is_hash_change_event_object(&entries) {
            return;
        }

        event.hash_change_interface = true;
        event.hash_change_old_url = Self::object_get_entry(&entries, "oldURL")
            .map(|value| value.as_string())
            .unwrap_or_default();
        event.hash_change_new_url = Self::object_get_entry(&entries, "newURL")
            .map(|value| value.as_string())
            .unwrap_or_default();
    }

    fn apply_error_event_payload_fields(
        event: &mut EventState,
        event_payload_object: Option<&Rc<RefCell<ObjectValue>>>,
    ) {
        let Some(event_payload_object) = event_payload_object else {
            return;
        };
        let entries = event_payload_object.borrow();
        if !Self::is_error_event_object(&entries) {
            return;
        }

        event.error_event_interface = true;
        event.error_event_message = Self::object_get_entry(&entries, "message")
            .map(|value| value.as_string())
            .unwrap_or_default();
        event.error_event_filename = Self::object_get_entry(&entries, "filename")
            .map(|value| value.as_string())
            .unwrap_or_default();
        event.error_event_lineno = Self::value_to_i64(
            &Self::object_get_entry(&entries, "lineno").unwrap_or(Value::Number(0)),
        );
        event.error_event_colno = Self::value_to_i64(
            &Self::object_get_entry(&entries, "colno").unwrap_or(Value::Number(0)),
        );
        event.error_event_error = Self::object_get_entry(&entries, "error").unwrap_or(Value::Null);
    }

    fn apply_message_event_payload_fields(
        event: &mut EventState,
        event_payload_object: Option<&Rc<RefCell<ObjectValue>>>,
    ) {
        let Some(event_payload_object) = event_payload_object else {
            return;
        };
        if !event.event_type.eq_ignore_ascii_case("message") {
            return;
        }
        let entries = event_payload_object.borrow();
        event.message_data =
            Some(Self::object_get_entry(&entries, "data").unwrap_or(Value::Undefined));
        event.message_origin = Some(
            Self::object_get_entry(&entries, "origin")
                .map(|value| value.as_string())
                .unwrap_or_default(),
        );
        event.message_source =
            Some(Self::object_get_entry(&entries, "source").unwrap_or(Value::Null));
    }

    pub(crate) fn dispatch_event_target_with_env(
        &mut self,
        target_object: Rc<RefCell<ObjectValue>>,
        event_payload: Value,
        env: &mut HashMap<String, Value>,
    ) -> Result<EventState> {
        let (event_type, detail, bubbles, cancelable, event_payload_object) =
            self.event_dispatch_payload_from_value(&event_payload)?;
        if event_type.is_empty() {
            return Err(Error::ScriptRuntime(
                "InvalidStateError: dispatchEvent requires non-empty event type".into(),
            ));
        }

        let node_id = self.event_target_listener_node_id(&target_object);
        let target_value = Value::Object(target_object);
        let mut event = EventState::new_untrusted(&event_type, node_id, self.scheduler.now_ms);
        event.target_value = Some(target_value.clone());
        event.current_target_value = Some(target_value);
        event.detail = detail;
        event.bubbles = bubbles;
        event.cancelable = cancelable;
        Self::apply_keyboard_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_wheel_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_pointer_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_navigate_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_before_unload_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_hash_change_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_error_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_message_event_payload_fields(&mut event, event_payload_object.as_ref());

        event.event_phase = 2;
        event.current_target = node_id;
        self.invoke_listeners(node_id, &mut event, env, true)?;
        if event.propagation_stopped {
            return Ok(event);
        }
        event.event_phase = 2;
        self.invoke_listeners(node_id, &mut event, env, false)?;

        Ok(event)
    }

    pub(crate) fn dispatch_event_target(
        &mut self,
        target_object: Rc<RefCell<ObjectValue>>,
        event_payload: Value,
    ) -> Result<EventState> {
        self.with_script_env(|this, env| {
            this.dispatch_event_target_with_env(target_object.clone(), event_payload.clone(), env)
        })
    }

    pub(crate) fn dispatch_dom_event_payload_with_env(
        &mut self,
        target_node: NodeId,
        event_payload: Value,
        env: &mut HashMap<String, Value>,
    ) -> Result<EventState> {
        let (event_type, detail, bubbles, cancelable, event_payload_object) =
            self.event_dispatch_payload_from_value(&event_payload)?;
        if event_type.is_empty() {
            return Err(Error::ScriptRuntime(
                "InvalidStateError: dispatchEvent requires non-empty event type".into(),
            ));
        }

        let mut event = EventState::new_untrusted(&event_type, target_node, self.scheduler.now_ms);
        event.detail = detail;
        event.bubbles = bubbles;
        event.cancelable = cancelable;
        Self::apply_keyboard_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_wheel_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_pointer_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_navigate_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_before_unload_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_hash_change_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_error_event_payload_fields(&mut event, event_payload_object.as_ref());
        Self::apply_message_event_payload_fields(&mut event, event_payload_object.as_ref());
        self.dispatch_prepared_event_with_env(event, env)
    }

    pub(crate) fn dispatch_dom_event_payload(
        &mut self,
        target_node: NodeId,
        event_payload: Value,
    ) -> Result<EventState> {
        self.with_script_env(|this, env| {
            this.dispatch_dom_event_payload_with_env(target_node, event_payload.clone(), env)
        })
    }
}
