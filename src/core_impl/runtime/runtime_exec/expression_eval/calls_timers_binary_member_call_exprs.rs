use super::*;

impl Harness {
    pub(crate) fn try_eval_member_call_exprs(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Option<Value>> {
        let result = (|| -> Result<Value> {
            match expr {
                Expr::MemberCall {
                    target,
                    member,
                    args,
                    optional,
                    optional_call,
                } => {
                    if Self::is_super_target_expr(target) {
                        let super_prototype = Self::super_prototype_from_env(env)?;
                        let this_value = Self::super_this_from_env(env)?;
                        let evaluated_args =
                            self.eval_call_args_with_spread(args, env, event_param, event)?;
                        let callee = self.object_property_from_value_with_receiver(
                            &super_prototype,
                            member,
                            &this_value,
                        )?;
                        return self
                            .execute_callable_value_with_this_and_env_and_sync(
                                &callee,
                                &evaluated_args,
                                event,
                                env,
                                Some(this_value),
                            )
                            .map_err(|err| match err {
                                Error::ScriptRuntime(msg)
                                    if msg == "callback is not a function" =>
                                {
                                    Error::ScriptRuntime(format!("'{}' is not a function", member))
                                }
                                other => other,
                            });
                    }

                    let receiver = self.eval_expr(target, env, event_param, event)?;
                    if *optional && matches!(receiver, Value::Null | Value::Undefined) {
                        return Ok(Value::Undefined);
                    }
                    if *optional_call {
                        let callee =
                            self.object_property_from_value(&receiver, member)
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "value is not an object" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "member call target does not support property '{}'",
                                            member
                                        ))
                                    }
                                    other => other,
                                })?;
                        if matches!(callee, Value::Null | Value::Undefined) {
                            return Ok(Value::Undefined);
                        }
                        let evaluated_args =
                            self.eval_call_args_with_spread(args, env, event_param, event)?;
                        return self
                            .execute_callable_value_with_this_and_env_and_sync(
                                &callee,
                                &evaluated_args,
                                event,
                                env,
                                Some(receiver.clone()),
                            )
                            .map_err(|err| match err {
                                Error::ScriptRuntime(msg)
                                    if msg == "callback is not a function" =>
                                {
                                    Error::ScriptRuntime(format!("'{}' is not a function", member))
                                }
                                other => other,
                            });
                    }
                    let evaluated_args =
                        self.eval_call_args_with_spread(args, env, event_param, event)?;

                    if let Value::FormData(entries) = &receiver {
                        if let Some(value) = self.eval_form_data_member_call_from_values(
                            entries,
                            member,
                            &evaluated_args,
                            event,
                        )? {
                            return Ok(value);
                        }
                    }

                    if member == "dispatchEvent" {
                        if evaluated_args.len() != 1 {
                            return Err(Error::ScriptRuntime(
                                "dispatchEvent requires exactly one argument".into(),
                            ));
                        }
                        let event_payload = evaluated_args[0].clone();
                        if let Value::Node(node) = &receiver {
                            let dispatched =
                                self.dispatch_dom_event_payload(*node, event_payload)?;
                            return Ok(Value::Bool(!dispatched.default_prevented));
                        }
                        if let Value::Object(object) = &receiver {
                            let (is_document_object, is_event_target_object) = {
                                let entries = object.borrow();
                                (
                                    matches!(
                                        Self::object_get_entry(
                                            &entries,
                                            INTERNAL_DOCUMENT_OBJECT_KEY
                                        ),
                                        Some(Value::Bool(true))
                                    ),
                                    Self::is_event_target_object(&entries),
                                )
                            };
                            if is_document_object {
                                let dispatched =
                                    self.dispatch_dom_event_payload(self.dom.root, event_payload)?;
                                return Ok(Value::Bool(!dispatched.default_prevented));
                            }
                            if is_event_target_object {
                                let dispatched = self.dispatch_event_target_with_expr_env_sync(
                                    object.clone(),
                                    event_payload,
                                    env,
                                )?;
                                return Ok(Value::Bool(!dispatched.default_prevented));
                            }
                        }
                    }

                    if matches!(member.as_str(), "call" | "apply" | "bind")
                        && self.is_callable_value(&receiver)
                    {
                        return self.execute_function_prototype_member(
                            member,
                            &receiver,
                            &evaluated_args,
                            event,
                            Some(env),
                        );
                    }

                    if let Value::Array(values) = &receiver {
                        if let Some(value) = self.eval_array_member_call(
                            values,
                            member,
                            &evaluated_args,
                            event,
                            Some(env),
                        )? {
                            return Ok(value);
                        }
                    }

                    if let Value::String(text) = &receiver {
                        if let Some(value) =
                            self.eval_string_member_call(text, member, &evaluated_args, event)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::Date(value) = &receiver {
                        if let Some(value) =
                            self.eval_date_member_call(value, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::NodeList(nodes) = &receiver {
                        if let Some(value) =
                            self.eval_nodelist_member_call(nodes, member, &evaluated_args, event)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::Node(node) = &receiver {
                        if let Some(value) =
                            self.eval_node_member_call(*node, member, &evaluated_args, event)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::TypedArray(array) = &receiver {
                        if let Some(value) = self.eval_typed_array_member_call(
                            array,
                            member,
                            &evaluated_args,
                            event,
                            Some(env),
                        )? {
                            return Ok(value);
                        }
                    }

                    if let Value::Blob(blob) = &receiver {
                        if let Some(value) =
                            self.eval_blob_member_call(blob, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::Map(map) = &receiver {
                        let (map_member_override, has_explicit_prototype) = {
                            let map_ref = map.borrow();
                            (
                                Self::object_get_entry(&map_ref.properties, member),
                                Self::object_get_entry(
                                    &map_ref.properties,
                                    INTERNAL_OBJECT_PROTOTYPE_KEY,
                                )
                                .is_some(),
                            )
                        };
                        if let Some(callee) = map_member_override {
                            return self
                                .execute_callable_value_with_env_and_sync(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    env,
                                )
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "callback is not a function" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "'{}' is not a function",
                                            member
                                        ))
                                    }
                                    other => other,
                                });
                        }
                        if !has_explicit_prototype {
                            if let Some(value) = self.eval_map_member_call_from_values(
                                map,
                                member,
                                &evaluated_args,
                                event,
                            )? {
                                return Ok(value);
                            }
                        }
                    }

                    if let Value::Set(set) = &receiver {
                        let (set_member_override, has_explicit_prototype) = {
                            let set_ref = set.borrow();
                            (
                                Self::object_get_entry(&set_ref.properties, member),
                                Self::object_get_entry(
                                    &set_ref.properties,
                                    INTERNAL_OBJECT_PROTOTYPE_KEY,
                                )
                                .is_some(),
                            )
                        };
                        if let Some(callee) = set_member_override {
                            return self
                                .execute_callable_value_with_env_and_sync(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    env,
                                )
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "callback is not a function" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "'{}' is not a function",
                                            member
                                        ))
                                    }
                                    other => other,
                                });
                        }
                        if !has_explicit_prototype {
                            if let Some(value) = self.eval_set_member_call_from_values(
                                set,
                                member,
                                &evaluated_args,
                                event,
                            )? {
                                return Ok(value);
                            }
                        }
                    }

                    if let Value::WeakMap(weak_map) = &receiver {
                        let (weak_map_member_override, has_explicit_prototype) = {
                            let weak_map_ref = weak_map.borrow();
                            (
                                Self::object_get_entry(&weak_map_ref.properties, member),
                                Self::object_get_entry(
                                    &weak_map_ref.properties,
                                    INTERNAL_OBJECT_PROTOTYPE_KEY,
                                )
                                .is_some(),
                            )
                        };
                        if let Some(callee) = weak_map_member_override {
                            return self
                                .execute_callable_value_with_env_and_sync(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    env,
                                )
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "callback is not a function" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "'{}' is not a function",
                                            member
                                        ))
                                    }
                                    other => other,
                                });
                        }
                        if !has_explicit_prototype {
                            if let Some(value) = self.eval_weak_map_member_call_from_values(
                                weak_map,
                                member,
                                &evaluated_args,
                                event,
                            )? {
                                return Ok(value);
                            }
                        }
                    }

                    if let Value::WeakSet(weak_set) = &receiver {
                        let (weak_set_member_override, has_explicit_prototype) = {
                            let weak_set_ref = weak_set.borrow();
                            (
                                Self::object_get_entry(&weak_set_ref.properties, member),
                                Self::object_get_entry(
                                    &weak_set_ref.properties,
                                    INTERNAL_OBJECT_PROTOTYPE_KEY,
                                )
                                .is_some(),
                            )
                        };
                        if let Some(callee) = weak_set_member_override {
                            return self
                                .execute_callable_value_with_env_and_sync(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    env,
                                )
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "callback is not a function" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "'{}' is not a function",
                                            member
                                        ))
                                    }
                                    other => other,
                                });
                        }
                        if !has_explicit_prototype {
                            if let Some(value) = self.eval_weak_set_member_call_from_values(
                                weak_set,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                    }

                    if let Value::UrlConstructor = &receiver {
                        let url_constructor_override = {
                            let entries = self.browser_apis.url_constructor_properties.borrow();
                            Self::object_get_entry(&entries, member)
                        };
                        if let Some(callee) = url_constructor_override {
                            return self
                                .execute_callable_value_with_env_and_sync(
                                    &callee,
                                    &evaluated_args,
                                    event,
                                    env,
                                )
                                .map_err(|err| match err {
                                    Error::ScriptRuntime(msg)
                                        if msg == "callback is not a function" =>
                                    {
                                        Error::ScriptRuntime(format!(
                                            "'{}' is not a function",
                                            member
                                        ))
                                    }
                                    other => other,
                                });
                        }
                        if let Some(value) =
                            self.eval_url_static_member_call_from_values(member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                    }

                    if let Value::Object(object) = &receiver {
                        let is_object_constructor =
                            Self::callable_kind_from_value(&receiver) == Some("object_constructor");
                        if is_object_constructor && member == "assign" {
                            return self.eval_object_assign_static_call(&evaluated_args, event);
                        }
                        if let Some(value) =
                            self.eval_event_target_member_call(object, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                        if let Some(value) = self.eval_named_node_map_member_call(
                            object,
                            member,
                            &evaluated_args,
                            event,
                        )? {
                            return Ok(value);
                        }
                        if let Some(value) =
                            self.eval_event_member_call(object, member, &evaluated_args, event)?
                        {
                            return Ok(value);
                        }
                        if let Some(value) =
                            self.eval_navigation_member_call(object, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                        if let Some(value) =
                            self.eval_mock_file_member_call(object, member, &evaluated_args)?
                        {
                            return Ok(value);
                        }
                        if let Some(value) = self.eval_clipboard_data_member_call(
                            object,
                            member,
                            &evaluated_args,
                            event,
                        )? {
                            return Ok(value);
                        }
                        let is_fetch_response_object = {
                            let entries = object.borrow();
                            Self::is_fetch_response_object(&entries)
                        };
                        if is_fetch_response_object {
                            if let Some(value) = self.eval_fetch_response_member_call(
                                object,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_fetch_request_object = {
                            let entries = object.borrow();
                            Self::is_fetch_request_object(&entries)
                        };
                        if is_fetch_request_object {
                            if let Some(value) = self.eval_fetch_request_member_call(
                                object,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_headers_object = {
                            let entries = object.borrow();
                            Self::is_headers_object(&entries)
                        };
                        if is_headers_object {
                            if let Some(value) =
                                self.eval_headers_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_cookie_store_object = {
                            let entries = object.borrow();
                            Self::is_cookie_store_object(&entries)
                        };
                        if is_cookie_store_object {
                            if let Some(value) =
                                self.eval_cookie_store_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_cache_storage_object = {
                            let entries = object.borrow();
                            Self::is_cache_storage_object(&entries)
                        };
                        if is_cache_storage_object {
                            if let Some(value) = self.eval_cache_storage_member_call(
                                object,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_cache_object = {
                            let entries = object.borrow();
                            Self::is_cache_object(&entries)
                        };
                        if is_cache_object {
                            if let Some(value) =
                                self.eval_cache_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_import_meta_object = {
                            let entries = object.borrow();
                            matches!(
                                Self::object_get_entry(&entries, INTERNAL_IMPORT_META_OBJECT_KEY),
                                Some(Value::Bool(true))
                            )
                        };
                        if is_import_meta_object && member == "resolve" {
                            return self.eval_import_meta_resolve_call(&evaluated_args);
                        }
                        let is_dom_parser_object = {
                            let entries = object.borrow();
                            matches!(
                                Self::object_get_entry(&entries, INTERNAL_DOM_PARSER_OBJECT_KEY),
                                Some(Value::Bool(true))
                            )
                        };
                        if is_dom_parser_object {
                            if let Some(value) =
                                self.eval_dom_parser_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_xml_serializer_object = {
                            let entries = object.borrow();
                            matches!(
                                Self::object_get_entry(
                                    &entries,
                                    INTERNAL_XML_SERIALIZER_OBJECT_KEY
                                ),
                                Some(Value::Bool(true))
                            )
                        };
                        if is_xml_serializer_object {
                            if let Some(value) = self.eval_xml_serializer_member_call(
                                object,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_parsed_document_object = {
                            let entries = object.borrow();
                            matches!(
                                Self::object_get_entry(
                                    &entries,
                                    INTERNAL_PARSED_DOCUMENT_OBJECT_KEY
                                ),
                                Some(Value::Bool(true))
                            )
                        };
                        if is_parsed_document_object {
                            if let Some(value) = self.eval_parsed_document_member_call(
                                object,
                                member,
                                &evaluated_args,
                                event,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_tree_walker_object = {
                            let entries = object.borrow();
                            matches!(
                                Self::object_get_entry(&entries, INTERNAL_TREE_WALKER_OBJECT_KEY),
                                Some(Value::Bool(true))
                            )
                        };
                        if is_tree_walker_object {
                            if let Some(value) =
                                self.eval_tree_walker_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_range_object = {
                            let entries = object.borrow();
                            Self::is_range_object(&entries)
                        };
                        if is_range_object {
                            if let Some(value) =
                                self.eval_range_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_selection_object = {
                            let entries = object.borrow();
                            Self::is_selection_object(&entries)
                        };
                        if is_selection_object {
                            if let Some(value) =
                                self.eval_selection_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_iterator_constructor = {
                            let entries = object.borrow();
                            Self::is_iterator_constructor_object(&entries)
                        };
                        if is_iterator_constructor {
                            if let Some(value) = self.eval_iterator_constructor_member_call(
                                object,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_iterator = {
                            let entries = object.borrow();
                            Self::is_iterator_object(&entries)
                        };
                        if is_iterator {
                            if let Some(value) = self.eval_iterator_member_call(
                                object,
                                member,
                                &evaluated_args,
                                event,
                            )? {
                                return Ok(value);
                            }
                        }
                        if Self::is_canvas_2d_context_object(&object.borrow()) {
                            if let Some(value) = self.eval_canvas_2d_context_member_call(
                                object,
                                member,
                                &evaluated_args,
                            )? {
                                return Ok(value);
                            }
                        }
                        let is_document_object = {
                            let entries = object.borrow();
                            matches!(
                                Self::object_get_entry(&entries, INTERNAL_DOCUMENT_OBJECT_KEY),
                                Some(Value::Bool(true))
                            )
                        };
                        if is_document_object {
                            if let Some(value) =
                                self.eval_document_member_call(member, &evaluated_args, event)?
                            {
                                return Ok(value);
                            }
                        }
                        let is_window_object = {
                            let entries = object.borrow();
                            Self::is_window_object(&entries)
                        };
                        if is_window_object {
                            if let Some(value) =
                                self.eval_window_member_call(member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        if Self::is_url_object(&object.borrow()) {
                            if let Some(value) =
                                self.eval_url_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                        if Self::is_url_search_params_object(&object.borrow()) {
                            if let Some(value) = self.eval_url_search_params_member_call(
                                object,
                                member,
                                &evaluated_args,
                                event,
                            )? {
                                return Ok(value);
                            }
                        }
                        if Self::is_storage_object(&object.borrow()) {
                            if let Some(value) =
                                self.eval_storage_member_call(object, member, &evaluated_args)?
                            {
                                return Ok(value);
                            }
                        }
                    }

                    let callee = self.object_property_from_value(&receiver, member).map_err(
                        |err| match err {
                            Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                                Error::ScriptRuntime(format!(
                                    "member call target does not support property '{}'",
                                    member
                                ))
                            }
                            other => other,
                        },
                    )?;
                    self.execute_callable_value_with_this_and_env_and_sync(
                        &callee,
                        &evaluated_args,
                        event,
                        env,
                        Some(receiver.clone()),
                    )
                    .map_err(|err| match err {
                        Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                            Error::ScriptRuntime(format!("'{}' is not a function", member))
                        }
                        other => other,
                    })
                }
                _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
            }
        })();
        match result {
            Err(Error::ScriptRuntime(msg)) if msg == UNHANDLED_EXPR_CHUNK => Ok(None),
            other => other.map(Some),
        }
    }
}
