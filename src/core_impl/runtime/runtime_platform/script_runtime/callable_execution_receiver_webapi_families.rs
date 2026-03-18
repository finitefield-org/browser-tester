use super::*;

impl Harness {
    pub(crate) fn execute_receiver_builtin_webapi_family(
        &mut self,
        family: &str,
        member: &str,
        receiver: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Option<Value>> {
        let value = match family {
            "document" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !matches!(
                    Self::object_get_entry(&object.borrow(), INTERNAL_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_document_member_call(member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported Document method: {member}"))
                        })?,
                )
            }
            "parsed_document" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !matches!(
                    Self::object_get_entry(&object.borrow(), INTERNAL_PARSED_DOCUMENT_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_parsed_document_member_call(object, member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported Document method: {member}"))
                        })?,
                )
            }
            "dom_parser" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !matches!(
                    Self::object_get_entry(&object.borrow(), INTERNAL_DOM_PARSER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_dom_parser_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported DOMParser method: {member}"))
                        })?,
                )
            }
            "xml_serializer" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !matches!(
                    Self::object_get_entry(&object.borrow(), INTERNAL_XML_SERIALIZER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_xml_serializer_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported XMLSerializer method: {member}"
                            ))
                        })?,
                )
            }
            "tree_walker" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !matches!(
                    Self::object_get_entry(&object.borrow(), INTERNAL_TREE_WALKER_OBJECT_KEY),
                    Some(Value::Bool(true))
                ) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_tree_walker_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported TreeWalker method: {member}"))
                        })?,
                )
            }
            "range" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_range_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_range_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported Range method: {member}"))
                        })?,
                )
            }
            "selection" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_selection_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_selection_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported Selection method: {member}"))
                        })?,
                )
            }
            "event" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_event_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_event_member_call(object, member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported Event method: {member}"))
                        })?,
                )
            }
            "keyboard_event" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_keyboard_event_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_event_member_call(object, member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported KeyboardEvent method: {member}"
                            ))
                        })?,
                )
            }
            "pointer_event" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_pointer_event_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_event_member_call(object, member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported PointerEvent method: {member}"
                            ))
                        })?,
                )
            }
            "navigate_event" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_navigate_event_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_event_member_call(object, member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported NavigateEvent method: {member}"
                            ))
                        })?,
                )
            }
            "data_transfer" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                {
                    let entries = object.borrow();
                    if !Self::is_data_transfer_object(&entries)
                        && !Self::is_clipboard_data_object(&entries)
                    {
                        return Err(Self::incompatible_receiver_error(family));
                    }
                }
                Some(
                    self.eval_clipboard_data_member_call(object, member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported DataTransfer method: {member}"
                            ))
                        })?,
                )
            }
            "data_transfer_item" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_data_transfer_item_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_clipboard_data_member_call(object, member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported DataTransferItem method: {member}"
                            ))
                        })?,
                )
            }
            "data_transfer_item_list" => {
                let Value::Array(values) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_data_transfer_item_list_value(&values.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_array_member_call(values, member, args, event, caller_env)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported DataTransferItemList method: {member}"
                            ))
                        })?,
                )
            }
            "match_media" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_match_media_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(if member == "dispatchEvent" {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "dispatchEvent requires exactly one argument".into(),
                        ));
                    }
                    let dispatched = if let Some(caller_env) = caller_env {
                        self.dispatch_event_target_with_expr_env_sync(
                            object.clone(),
                            args[0].clone(),
                            caller_env,
                        )?
                    } else {
                        self.dispatch_event_target(object.clone(), args[0].clone())?
                    };
                    Value::Bool(!dispatched.default_prevented)
                } else {
                    self.eval_event_target_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported MediaQueryList method: {member}"
                            ))
                        })?
                })
            }
            "event_target" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_event_target_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(if member == "dispatchEvent" {
                    if args.len() != 1 {
                        return Err(Error::ScriptRuntime(
                            "dispatchEvent requires exactly one argument".into(),
                        ));
                    }
                    let dispatched = if let Some(caller_env) = caller_env {
                        self.dispatch_event_target_with_expr_env_sync(
                            object.clone(),
                            args[0].clone(),
                            caller_env,
                        )?
                    } else {
                        self.dispatch_event_target(object.clone(), args[0].clone())?
                    };
                    Value::Bool(!dispatched.default_prevented)
                } else {
                    self.eval_event_target_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported EventTarget method: {member}"
                            ))
                        })?
                })
            }
            "cookie_store" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_cookie_store_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_cookie_store_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported CookieStore method: {member}"
                            ))
                        })?,
                )
            }
            "cache_storage" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_cache_storage_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_cache_storage_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported CacheStorage method: {member}"
                            ))
                        })?,
                )
            }
            "cache" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_cache_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_cache_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported Cache method: {member}"))
                        })?,
                )
            }
            "blob" => {
                let Value::Blob(blob) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                Some(
                    self.eval_blob_member_call(blob, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported Blob method: {member}"))
                        })?,
                )
            }
            "array_buffer" => {
                let Value::ArrayBuffer(buffer) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                Some(
                    self.eval_array_buffer_member_call_from_values(buffer, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported ArrayBuffer method: {member}"
                            ))
                        })?,
                )
            }
            "promise" => {
                let Value::Promise(promise) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                Some(
                    self.eval_promise_member_call_from_values(promise, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported Promise method: {member}"))
                        })?,
                )
            }
            "url" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_url_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_url_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported URL method: {member}"))
                        })?,
                )
            }
            "url_search_params" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_url_search_params_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_url_search_params_member_call(object, member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported URLSearchParams method: {member}"
                            ))
                        })?,
                )
            }
            "storage" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_storage_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_storage_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported Storage method: {member}"))
                        })?,
                )
            }
            "form_data" => {
                let Value::FormData(entries) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                Some(
                    self.eval_form_data_member_call_from_values(entries, member, args, event)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!("unsupported FormData method: {member}"))
                        })?,
                )
            }
            "canvas_2d_context" => {
                let Value::Object(object) = receiver else {
                    return Err(Self::incompatible_receiver_error(family));
                };
                if !Self::is_canvas_2d_context_object(&object.borrow()) {
                    return Err(Self::incompatible_receiver_error(family));
                }
                Some(
                    self.eval_canvas_2d_context_member_call(object, member, args)?
                        .ok_or_else(|| {
                            Error::ScriptRuntime(format!(
                                "unsupported CanvasRenderingContext2D method: {member}"
                            ))
                        })?,
                )
            }
            _ => None,
        };
        Ok(value)
    }
}
