use super::*;

impl Harness {
    pub(crate) fn eval_document_member_call(
        &mut self,
        member: &str,
        evaluated_args: &[Value],
        _event: &EventState,
    ) -> Result<Option<Value>> {
        let shadowed = {
            let entries = self.dom_runtime.document_object.borrow();
            Self::placeholder_backed_object_builtin_is_shadowed(&entries, member)
        };
        if shadowed {
            return Ok(None);
        }

        match member {
            "getElementById" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementById requires exactly one argument".into(),
                    ));
                }
                let id = evaluated_args[0].as_string();
                Ok(Some(
                    self.dom.by_id(&id).map(Value::Node).unwrap_or(Value::Null),
                ))
            }
            "getElementsByClassName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByClassName requires exactly one argument".into(),
                    ));
                }
                let class_names = Self::class_names_from_argument(&evaluated_args[0]);
                Ok(Some(
                    self.class_names_live_list_value(self.dom.root, class_names),
                ))
            }
            "getElementsByName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(self.name_live_list_value(
                    self.dom.root,
                    evaluated_args[0].as_string(),
                )))
            }
            "getElementsByTagName" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagName requires exactly one argument".into(),
                    ));
                }
                Ok(Some(self.tag_name_live_list_value(
                    self.dom.root,
                    Self::tag_name_from_argument(&evaluated_args[0]),
                )))
            }
            "getElementsByTagNameNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "getElementsByTagNameNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string();
                Ok(Some(self.tag_name_ns_live_list_value(
                    self.dom.root,
                    namespace_uri,
                    local_name,
                )))
            }
            "createElement" => {
                if !(evaluated_args.len() == 1 || evaluated_args.len() == 2) {
                    return Err(Error::ScriptRuntime(
                        "createElement requires one or two arguments".into(),
                    ));
                }
                let tag_name = evaluated_args[0].as_string().to_ascii_lowercase();
                let node = self.dom.create_detached_element(tag_name);
                if let Some(is_value) =
                    Self::create_element_is_option_from_arg(evaluated_args.get(1))
                {
                    self.dom.set_attr(node, "is", &is_value)?;
                }
                Ok(Some(Value::Node(node)))
            }
            "createElementNS" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "createElementNS requires two or three arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let raw_tag_name = evaluated_args[1].as_string();
                let tag_name = if namespace_uri.as_deref() == Some("http://www.w3.org/1999/xhtml") {
                    raw_tag_name.to_ascii_lowercase()
                } else {
                    raw_tag_name
                };
                let node = self
                    .dom
                    .create_detached_element_with_namespace(tag_name, namespace_uri);
                if let Some(is_value) =
                    Self::create_element_is_option_from_arg(evaluated_args.get(2))
                {
                    self.dom.set_attr(node, "is", &is_value)?;
                }
                Ok(Some(Value::Node(node)))
            }
            "createTextNode" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "createTextNode requires exactly one argument".into(),
                    ));
                }
                let text = evaluated_args[0].as_string();
                let node = self.dom.create_detached_text(text);
                Ok(Some(Value::Node(node)))
            }
            "createAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "createAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if !is_valid_create_attribute_name(&name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                Ok(Some(Self::new_attr_object_value(&name, "", None)))
            }
            "createDocumentFragment" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "createDocumentFragment takes no arguments".into(),
                    ));
                }
                let node = self
                    .dom
                    .create_detached_element("#document-fragment".to_string());
                Ok(Some(Value::Node(node)))
            }
            "createRange" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "createRange takes no arguments".into(),
                    ));
                }
                Ok(Some(Self::new_range_object_value(self.dom.root)))
            }
            "getSelection" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getSelection takes no arguments".into(),
                    ));
                }
                Ok(Some(self.ensure_document_selection_object()))
            }
            "append" => Ok(Some(
                self.eval_document_append_call(self.dom.root, evaluated_args)?,
            )),
            "querySelector" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelector requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.eval_query_selector_value(self.dom.root, &selector)?,
                ))
            }
            "querySelectorAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelectorAll requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(
                    self.eval_query_selector_all_value(self.dom.root, &selector)?,
                ))
            }
            "createTreeWalker" => self.eval_create_tree_walker_call(evaluated_args),
            "addEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "addEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
                    Value::Function(function) => {
                        self.listeners.add(
                            self.dom.root,
                            event_type,
                            Listener {
                                capture,
                                is_event_handler_property: false,
                                is_arrow: function.is_arrow,
                                handler: function.handler.clone(),
                                function: Some(function.clone()),
                                captured_names: function.captured_names.clone(),
                                captured_env: function.captured_env.clone(),
                                captured_pending_function_decls: function
                                    .captured_pending_function_decls
                                    .clone(),
                            },
                        );
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "addEventListener callback must be a function".into(),
                    )),
                }
            }
            "removeEventListener" => {
                if !(evaluated_args.len() == 2 || evaluated_args.len() == 3) {
                    return Err(Error::ScriptRuntime(
                        "removeEventListener requires two or three arguments".into(),
                    ));
                }
                let event_type = evaluated_args[0].as_string();
                let capture = self.parse_listener_capture_arg(evaluated_args.get(2))?;
                match &evaluated_args[1] {
                    Value::Function(function) => {
                        let _ = self.listeners.remove(
                            self.dom.root,
                            &event_type,
                            capture,
                            &function.handler,
                        );
                        Ok(Some(Value::Undefined))
                    }
                    Value::Null | Value::Undefined => Ok(Some(Value::Undefined)),
                    _ => Err(Error::ScriptRuntime(
                        "removeEventListener callback must be a function".into(),
                    )),
                }
            }
            _ => Ok(None),
        }
    }
}
