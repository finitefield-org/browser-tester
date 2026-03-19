use super::*;

impl Harness {
    pub(crate) fn eval_event_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let (
            is_event_object,
            is_navigate_event_object,
            is_pointer_event_object,
            own_override,
            builtin_deleted,
        ) = {
            let entries = object.borrow();
            (
                Self::is_event_object(&entries),
                Self::is_navigate_event_object(&entries),
                Self::is_pointer_event_object(&entries),
                Self::object_get_entry(&entries, member)
                    .is_some_and(|value| !Self::is_builtin_placeholder_value(&value)),
                Self::is_builtin_object_property_deleted(&entries, member),
            )
        };
        if !is_event_object {
            return Ok(None);
        }
        if own_override || builtin_deleted {
            return Ok(None);
        }

        let value = match member {
            "preventDefault" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Event.preventDefault does not take arguments".into(),
                    ));
                }
                let cancelable = {
                    let entries = object.borrow();
                    Self::object_get_entry(&entries, "cancelable")
                        .is_some_and(|value| value.truthy())
                };
                if cancelable {
                    Self::object_set_entry(
                        &mut object.borrow_mut(),
                        "defaultPrevented".to_string(),
                        Value::Bool(true),
                    );
                }
                Value::Undefined
            }
            "stopPropagation" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Event.stopPropagation does not take arguments".into(),
                    ));
                }
                Self::object_set_entry(
                    &mut object.borrow_mut(),
                    INTERNAL_EVENT_STOP_PROPAGATION_KEY.to_string(),
                    Value::Bool(true),
                );
                Value::Undefined
            }
            "stopImmediatePropagation" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Event.stopImmediatePropagation does not take arguments".into(),
                    ));
                }
                let mut entries = object.borrow_mut();
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_EVENT_STOP_PROPAGATION_KEY.to_string(),
                    Value::Bool(true),
                );
                Self::object_set_entry(
                    &mut entries,
                    INTERNAL_EVENT_STOP_IMMEDIATE_PROPAGATION_KEY.to_string(),
                    Value::Bool(true),
                );
                Value::Undefined
            }
            "getModifierState" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "KeyboardEvent.getModifierState requires exactly one argument".into(),
                    ));
                }
                let modifier = evaluated_args[0].as_string();
                let entries = object.borrow();
                if !Self::is_keyboard_event_object(&entries) {
                    Value::Bool(false)
                } else {
                    Value::Bool(Self::event_modifier_state_from_entries(&entries, &modifier))
                }
            }
            "intercept" => {
                if !is_navigate_event_object {
                    return Ok(None);
                }
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "NavigateEvent.intercept supports at most one options argument".into(),
                    ));
                }

                let can_intercept = {
                    let entries = object.borrow();
                    Self::object_get_entry(&entries, "canIntercept")
                        .is_some_and(|value| value.truthy())
                };
                if !can_intercept {
                    return Err(Error::ScriptRuntime(
                        "InvalidStateError: Failed to execute 'intercept': navigation cannot be intercepted"
                            .into(),
                    ));
                }

                let mut handler = Value::Undefined;
                if let Some(options) = evaluated_args.first() {
                    match options {
                        Value::Null | Value::Undefined => {}
                        Value::Object(options_entries) => {
                            let options_entries = options_entries.borrow();
                            if let Some(value) = Self::object_get_entry(&options_entries, "handler")
                            {
                                if !matches!(value, Value::Null | Value::Undefined)
                                    && !self.is_callable_value(&value)
                                {
                                    return Err(Error::ScriptRuntime(
                                        "NavigateEvent.intercept handler must be callable".into(),
                                    ));
                                }
                                handler = value;
                            }
                        }
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "NavigateEvent.intercept options argument must be an object".into(),
                            ));
                        }
                    }
                }

                if !matches!(handler, Value::Null | Value::Undefined) {
                    let _ = self.execute_callback_value(&handler, &[], event)?;
                }

                Self::object_set_entry(
                    &mut object.borrow_mut(),
                    "\0\0bt_event:navigate:intercepted".to_string(),
                    Value::Bool(true),
                );
                Value::Undefined
            }
            "scroll" => {
                if !is_navigate_event_object {
                    return Ok(None);
                }
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "NavigateEvent.scroll does not take arguments".into(),
                    ));
                }
                Self::object_set_entry(
                    &mut object.borrow_mut(),
                    "\0\0bt_event:navigate:scroll_called".to_string(),
                    Value::Bool(true),
                );
                Value::Undefined
            }
            "getCoalescedEvents" => {
                if !is_pointer_event_object {
                    return Ok(None);
                }
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "PointerEvent.getCoalescedEvents does not take arguments".into(),
                    ));
                }
                Self::new_array_value(Vec::new())
            }
            "getPredictedEvents" => {
                if !is_pointer_event_object {
                    return Ok(None);
                }
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "PointerEvent.getPredictedEvents does not take arguments".into(),
                    ));
                }
                Self::new_array_value(Vec::new())
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    fn new_navigation_api_result_value(&mut self) -> Result<Value> {
        Ok(Self::new_object_value(vec![
            (
                "committed".to_string(),
                Value::Promise(self.promise_resolve_value_as_promise(Value::Undefined)?),
            ),
            (
                "finished".to_string(),
                Value::Promise(self.promise_resolve_value_as_promise(Value::Undefined)?),
            ),
        ]))
    }

    pub(crate) fn dispatch_navigation_simple_event(&mut self, event_type: &str) -> Result<()> {
        let _ = self.dispatch_event_target(
            self.location_history.navigation_object.clone(),
            Value::String(event_type.to_string()),
        )?;
        Ok(())
    }

    fn dispatch_navigation_navigate_event(
        &mut self,
        from_url: &str,
        destination_url: &str,
        navigation_type: &str,
    ) -> Result<()> {
        let payload = Self::new_object_value(vec![
            (INTERNAL_EVENT_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_NAVIGATE_EVENT_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("type".to_string(), Value::String("navigate".to_string())),
            ("bubbles".to_string(), Value::Bool(false)),
            ("cancelable".to_string(), Value::Bool(true)),
            ("canIntercept".to_string(), Value::Bool(true)),
            (
                "destination".to_string(),
                Self::new_object_value(vec![(
                    "url".to_string(),
                    Value::String(destination_url.to_string()),
                )]),
            ),
            ("downloadRequest".to_string(), Value::Null),
            ("formData".to_string(), Value::Null),
            (
                "hashChange".to_string(),
                Value::Bool(Self::is_hash_only_navigation(from_url, destination_url)),
            ),
            ("hasUAVisualTransition".to_string(), Value::Bool(false)),
            ("info".to_string(), Value::Undefined),
            (
                "navigationType".to_string(),
                Value::String(navigation_type.to_string()),
            ),
            (
                "signal".to_string(),
                Self::new_navigate_event_default_signal_value(),
            ),
            ("sourceElement".to_string(), Value::Null),
            ("userInitiated".to_string(), Value::Bool(false)),
            (
                "intercept".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "scroll".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ]);
        let _ =
            self.dispatch_event_target(self.location_history.navigation_object.clone(), payload)?;
        Ok(())
    }

    pub(crate) fn eval_navigation_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let is_navigation_object = {
            let entries = object.borrow();
            Self::is_navigation_object(&entries)
        };
        if !is_navigation_object {
            return Ok(None);
        }

        let result = match member {
            "entries" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "navigation.entries does not take arguments".into(),
                    ));
                }
                self.navigation_entries_value()
            }
            "back" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "navigation.back does not take arguments".into(),
                    ));
                }
                if !self.navigation_can_go_back() {
                    return Ok(Some(self.new_navigation_api_result_value()?));
                }

                let from = self.document_url.clone();
                let destination = self
                    .location_history
                    .history_entries
                    .get(self.location_history.history_index.saturating_sub(1))
                    .map(|entry| entry.url.clone())
                    .unwrap_or_else(|| self.document_url.clone());
                self.dispatch_navigation_navigate_event(&from, &destination, "traverse")?;
                if let Err(err) = self.history_go_with_env(-1) {
                    let _ = self.dispatch_navigation_simple_event("navigateerror");
                    return Err(err);
                }
                self.dispatch_navigation_simple_event("currententrychange")?;
                self.dispatch_navigation_simple_event("navigatesuccess")?;
                self.new_navigation_api_result_value()?
            }
            "forward" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "navigation.forward does not take arguments".into(),
                    ));
                }
                if !self.navigation_can_go_forward() {
                    return Ok(Some(self.new_navigation_api_result_value()?));
                }

                let from = self.document_url.clone();
                let destination = self
                    .location_history
                    .history_entries
                    .get(self.location_history.history_index.saturating_add(1))
                    .map(|entry| entry.url.clone())
                    .unwrap_or_else(|| self.document_url.clone());
                self.dispatch_navigation_navigate_event(&from, &destination, "traverse")?;
                if let Err(err) = self.history_go_with_env(1) {
                    let _ = self.dispatch_navigation_simple_event("navigateerror");
                    return Err(err);
                }
                self.dispatch_navigation_simple_event("currententrychange")?;
                self.dispatch_navigation_simple_event("navigatesuccess")?;
                self.new_navigation_api_result_value()?
            }
            "traverseTo" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "navigation.traverseTo requires exactly one key argument".into(),
                    ));
                }
                let key = evaluated_args[0].as_string();
                let Some(target_index) = self.navigation_find_entry_index_by_key(&key) else {
                    return Err(Error::ScriptRuntime(format!(
                        "NotFoundError: navigation entry key not found: {key}"
                    )));
                };

                if target_index == self.location_history.history_index {
                    return Ok(Some(self.new_navigation_api_result_value()?));
                }

                let from = self.document_url.clone();
                let destination = self
                    .location_history
                    .history_entries
                    .get(target_index)
                    .map(|entry| entry.url.clone())
                    .unwrap_or_else(|| self.document_url.clone());
                self.dispatch_navigation_navigate_event(&from, &destination, "traverse")?;
                let delta = target_index as i64 - self.location_history.history_index as i64;
                if let Err(err) = self.history_go_with_env(delta) {
                    let _ = self.dispatch_navigation_simple_event("navigateerror");
                    return Err(err);
                }
                self.dispatch_navigation_simple_event("currententrychange")?;
                self.dispatch_navigation_simple_event("navigatesuccess")?;
                self.new_navigation_api_result_value()?
            }
            "navigate" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "navigation.navigate requires one or two arguments".into(),
                    ));
                }

                let raw_url = evaluated_args[0].as_string();
                let destination = self.try_resolve_location_target_url(&raw_url)?;
                let mut state = Value::Null;
                let mut replace = false;

                if let Some(options) = evaluated_args.get(1) {
                    match options {
                        Value::Null | Value::Undefined => {}
                        Value::Object(options_entries) => {
                            let options_entries = options_entries.borrow();
                            if let Some(value) = Self::object_get_entry(&options_entries, "state") {
                                state = value;
                            }
                            if Self::object_get_entry(&options_entries, "history")
                                .map(|value| value.as_string().eq_ignore_ascii_case("replace"))
                                .unwrap_or(false)
                            {
                                replace = true;
                            }
                        }
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "navigation.navigate options argument must be an object".into(),
                            ));
                        }
                    }
                }

                let from = self.document_url.clone();
                self.dispatch_navigation_navigate_event(
                    &from,
                    &destination,
                    if replace { "replace" } else { "push" },
                )?;

                let state = Self::structured_clone_value(&state, &mut Vec::new(), &mut Vec::new())?;
                let hash_only_navigation = Self::is_hash_only_navigation(&from, &destination);
                let mock_html = if !hash_only_navigation {
                    self.location_history
                        .location_mock_pages
                        .get(&destination)
                        .cloned()
                } else {
                    None
                };
                let navigation_kind = if replace {
                    LocationNavigationKind::Replace
                } else {
                    LocationNavigationKind::Assign
                };

                if let Some(html) = mock_html {
                    if !self.prepare_for_cross_document_lifecycle_swap()? {
                        return self.new_navigation_api_result_value().map(Some);
                    }
                    self.document_url = destination.clone();
                    if replace {
                        self.history_replace_current_entry(&destination, state);
                    } else {
                        self.history_push_entry(&destination, state);
                    }
                    self.location_history
                        .location_navigations
                        .push(LocationNavigation {
                            kind: navigation_kind,
                            from: from.clone(),
                            to: destination.clone(),
                        });
                    self.replace_document_with_html(&html)?;
                    self.dispatch_window_lifecycle_event("pageshow")?;
                } else {
                    self.document_url = destination.clone();
                    if replace {
                        self.history_replace_current_entry(&destination, state);
                    } else {
                        self.history_push_entry(&destination, state);
                    }
                    self.sync_location_object();
                    self.sync_history_object();
                    self.sync_navigation_object();
                    self.sync_document_object();
                    self.sync_window_runtime_properties();
                    self.location_history
                        .location_navigations
                        .push(LocationNavigation {
                            kind: navigation_kind,
                            from: from.clone(),
                            to: destination.clone(),
                        });
                    if !hash_only_navigation {
                        let _ = self.load_location_mock_page_if_exists(&destination)?;
                    }
                }
                self.dispatch_navigation_simple_event("currententrychange")?;
                self.dispatch_navigation_simple_event("navigatesuccess")?;
                self.new_navigation_api_result_value()?
            }
            "reload" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "navigation.reload supports zero or one options argument".into(),
                    ));
                }
                let current_url = self.document_url.clone();
                self.dispatch_navigation_navigate_event(&current_url, &current_url, "reload")?;

                let mut state_override = None;
                if let Some(options) = evaluated_args.first() {
                    match options {
                        Value::Null | Value::Undefined => {}
                        Value::Object(options_entries) => {
                            let options_entries = options_entries.borrow();
                            if let Some(value) = Self::object_get_entry(&options_entries, "state") {
                                state_override = Some(value);
                            }
                        }
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "navigation.reload options argument must be an object".into(),
                            ));
                        }
                    }
                }

                let state_override = if let Some(state) = state_override {
                    Some(Self::structured_clone_value(
                        &state,
                        &mut Vec::new(),
                        &mut Vec::new(),
                    )?)
                } else {
                    None
                };

                if let Err(err) = self.reload_location_with_state_override(state_override) {
                    let _ = self.dispatch_navigation_simple_event("navigateerror");
                    return Err(err);
                }
                self.dispatch_navigation_simple_event("navigatesuccess")?;
                self.new_navigation_api_result_value()?
            }
            "updateCurrentEntry" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "navigation.updateCurrentEntry requires exactly one options argument"
                            .into(),
                    ));
                }

                let options_entries = match &evaluated_args[0] {
                    Value::Object(options_entries) => options_entries.borrow(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "navigation.updateCurrentEntry options argument must be an object"
                                .into(),
                        ));
                    }
                };
                let state =
                    Self::object_get_entry(&options_entries, "state").unwrap_or(Value::Undefined);
                let cloned =
                    Self::structured_clone_value(&state, &mut Vec::new(), &mut Vec::new())?;
                self.history_replace_current_entry(&self.document_url.clone(), cloned);
                self.sync_history_object();
                self.sync_navigation_object();
                self.sync_window_runtime_properties();
                self.dispatch_navigation_simple_event("currententrychange")?;
                Value::Undefined
            }
            _ => return Ok(None),
        };

        Ok(Some(result))
    }
}
