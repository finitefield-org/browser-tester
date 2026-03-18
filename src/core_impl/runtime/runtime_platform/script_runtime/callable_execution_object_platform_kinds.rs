use super::*;

impl Harness {
    pub(crate) fn execute_object_callable_platform_kind(
        &mut self,
        kind: &str,
        callable: &Value,
        args: &[Value],
        this_arg: Option<&Value>,
    ) -> Result<Option<Value>> {
        let value = match kind {
            "async_generator_function_constructor" => {
                Some(self.build_async_generator_function_from_constructor_values(args)?)
            }
            "generator_function_constructor" => {
                Some(self.build_generator_function_from_constructor_values(args)?)
            }
            "function_constructor" => Some(self.build_function_from_constructor_values(args)?),
            "boolean_constructor" => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Some(Value::Bool(value.truthy()))
            }
            "number_constructor" => {
                let value = args.first().cloned().unwrap_or(Value::Number(0));
                Some(Self::number_value(
                    Self::coerce_number_for_number_constructor(&value),
                ))
            }
            "bigint_constructor" => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Some(Value::BigInt(Self::coerce_bigint_for_constructor(&value)?))
            }
            "object_constructor" => {
                if args.is_empty() || matches!(args[0], Value::Null | Value::Undefined) {
                    Some(Self::new_object_value(Vec::new()))
                } else {
                    Some(match &args[0] {
                        Value::Object(_)
                        | Value::Array(_)
                        | Value::Date(_)
                        | Value::Map(_)
                        | Value::WeakMap(_)
                        | Value::Set(_)
                        | Value::WeakSet(_)
                        | Value::Blob(_)
                        | Value::ArrayBuffer(_)
                        | Value::TypedArray(_)
                        | Value::Promise(_)
                        | Value::RegExp(_)
                        | Value::Function(_)
                        | Value::Node(_)
                        | Value::NodeList(_)
                        | Value::FormData(_)
                        | Value::StringConstructor
                        | Value::BlobConstructor
                        | Value::UrlConstructor
                        | Value::ArrayBufferConstructor
                        | Value::PromiseConstructor
                        | Value::MapConstructor
                        | Value::WeakMapConstructor
                        | Value::SetConstructor
                        | Value::WeakSetConstructor
                        | Value::UrlSearchParamsConstructor
                        | Value::SymbolConstructor
                        | Value::RegExpConstructor
                        | Value::TypedArrayConstructor(_)
                        | Value::PromiseCapability(_) => args[0].clone(),
                        _ => Self::box_primitive_value(args[0].clone()),
                    })
                }
            }
            "node_list_constructor"
            | "image_bitmap_constructor"
            | "text_track_constructor"
            | "text_track_list_constructor"
            | "time_ranges_constructor"
            | "storage_constructor"
            | "cookie_store_constructor"
            | "cache_storage_constructor"
            | "cache_constructor"
            | "radio_node_list_constructor"
            | "html_collection_constructor"
            | "html_form_controls_collection_constructor"
            | "html_options_collection_constructor" => {
                return Err(Error::ScriptRuntime("Illegal constructor".into()));
            }
            "event_target_constructor" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "EventTarget constructor does not take arguments".into(),
                    ));
                }
                Some(self.new_event_target_instance_from_constructor(callable, this_arg.cloned())?)
            }
            "event_constructor" => Some(self.new_event_object_from_constructor_args(
                "Event", args, false, false, false, false, false, false, false, false,
            )?),
            "custom_event_constructor" => Some(self.new_event_object_from_constructor_args(
                "CustomEvent",
                args,
                true,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
            )?),
            "mouse_event_constructor" => Some(self.new_event_object_from_constructor_args(
                "MouseEvent",
                args,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
            )?),
            "keyboard_event_constructor" => Some(self.new_event_object_from_constructor_args(
                "KeyboardEvent",
                args,
                false,
                true,
                false,
                false,
                false,
                false,
                false,
                false,
            )?),
            "wheel_event_constructor" => Some(self.new_event_object_from_constructor_args(
                "WheelEvent",
                args,
                false,
                false,
                true,
                false,
                false,
                false,
                false,
                false,
            )?),
            "navigate_event_constructor" => Some(self.new_event_object_from_constructor_args(
                "NavigateEvent",
                args,
                false,
                false,
                false,
                true,
                false,
                false,
                false,
                false,
            )?),
            "pointer_event_constructor" => Some(self.new_event_object_from_constructor_args(
                "PointerEvent",
                args,
                false,
                false,
                false,
                false,
                true,
                false,
                false,
                false,
            )?),
            "hash_change_event_constructor" => Some(self.new_event_object_from_constructor_args(
                "HashChangeEvent",
                args,
                false,
                false,
                false,
                false,
                false,
                true,
                false,
                false,
            )?),
            "error_event_constructor" => Some(self.new_event_object_from_constructor_args(
                "ErrorEvent",
                args,
                false,
                false,
                false,
                false,
                false,
                false,
                true,
                false,
            )?),
            "before_unload_event_constructor" => {
                Some(self.new_event_object_from_constructor_args(
                    "BeforeUnloadEvent",
                    args,
                    false,
                    false,
                    false,
                    false,
                    false,
                    false,
                    false,
                    true,
                )?)
            }
            "image_data_constructor" => Some(self.new_image_data_from_constructor_args(args)?),
            "dom_parser_constructor" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "DOMParser constructor does not take arguments".into(),
                    ));
                }
                Some(Self::new_dom_parser_instance_value())
            }
            "xml_serializer_constructor" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "XMLSerializer constructor does not take arguments".into(),
                    ));
                }
                Some(Self::new_xml_serializer_instance_value())
            }
            "document_constructor" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Document constructor does not take arguments".into(),
                    ));
                }
                Some(self.new_empty_parsed_document_value())
            }
            "document_parse_html" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Document.parseHTML requires exactly one argument".into(),
                    ));
                }
                Some(self.new_parsed_document_value_from_markup(
                    &args[0].as_string(),
                    true,
                    "text/html",
                )?)
            }
            "document_parse_html_unsafe" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Document.parseHTMLUnsafe requires exactly one argument".into(),
                    ));
                }
                Some(self.new_parsed_document_value_from_markup(
                    &args[0].as_string(),
                    false,
                    "text/html",
                )?)
            }
            "fetch_function" => Some(self.eval_fetch_call_from_values(args)?),
            "match_media_function" => Some(self.eval_match_media_call_from_values(args)?),
            "clipboard_item_constructor" => {
                Some(self.new_clipboard_item_value_from_constructor_args(args)?)
            }
            "clipboard_write" => Some(self.eval_clipboard_write_call(args)?),
            "request_constructor" => Some(self.new_fetch_request_value_from_call_args(args)?),
            "file_constructor" => {
                let mut instance = self.new_file_value_from_constructor_args(args)?;
                self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                Some(instance)
            }
            "headers_constructor" => Some(self.new_headers_value_from_call_args(args)?),
            "text_encoder_constructor" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TextEncoder constructor does not take arguments".into(),
                    ));
                }
                let mut instance = Self::new_text_encoder_instance_value();
                self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                Some(instance)
            }
            "text_decoder_constructor" => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TextDecoder constructor supports up to two arguments".into(),
                    ));
                }
                let encoding = match args.first() {
                    None | Some(Value::Undefined) => "utf-8",
                    Some(label) => Self::normalize_text_decoder_label(&label.as_string())
                        .ok_or_else(|| {
                            Error::ScriptRuntime(
                                "TextDecoder constructor received unsupported encoding label"
                                    .into(),
                            )
                        })?,
                };
                let (fatal, ignore_bom) = Self::text_decoder_options_from_value(args.get(1))?;
                let mut instance =
                    Self::new_text_decoder_instance_value(encoding, fatal, ignore_bom);
                self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                Some(instance)
            }
            "text_encoder_stream_constructor" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "TextEncoderStream constructor does not take arguments".into(),
                    ));
                }
                let readable = self.new_readable_stream_placeholder_value(Vec::new());
                let writable = Self::new_writable_stream_placeholder_value();
                let mut instance = Self::new_text_encoder_stream_instance_value(readable, writable);
                self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                Some(instance)
            }
            "text_decoder_stream_constructor" => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TextDecoderStream constructor supports up to two arguments".into(),
                    ));
                }
                let encoding = match args.first() {
                    None | Some(Value::Undefined) => "utf-8",
                    Some(label) => Self::normalize_text_decoder_label(&label.as_string())
                        .ok_or_else(|| {
                            Error::ScriptRuntime(
                                "TextDecoderStream constructor received unsupported encoding label"
                                    .into(),
                            )
                        })?,
                };
                let (fatal, ignore_bom) = Self::text_decoder_options_from_value(args.get(1))?;
                let readable = self.new_readable_stream_placeholder_value(Vec::new());
                let writable = Self::new_writable_stream_placeholder_value();
                let mut instance = Self::new_text_decoder_stream_instance_value(
                    encoding, fatal, ignore_bom, readable, writable,
                );
                self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                Some(instance)
            }
            "css_style_sheet_constructor" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "CSSStyleSheet constructor does not take arguments".into(),
                    ));
                }
                let mut instance = Self::new_css_style_sheet_instance_value(Value::Object(
                    self.dom_runtime.document_object.clone(),
                ));
                self.attach_constructor_prototype_to_instance(callable, &mut instance)?;
                Some(instance)
            }
            "text_encoder_get_encoding" => {
                Self::text_encoder_receiver_object(this_arg)?;
                Some(Value::String("utf-8".to_string()))
            }
            "text_encoder_encode" => {
                Self::text_encoder_receiver_object(this_arg)?;
                let input = args.first().map(Value::as_string).unwrap_or_default();
                Some(Self::new_uint8_typed_array_from_bytes(input.as_bytes()))
            }
            "text_encoder_encode_into" => {
                Self::text_encoder_receiver_object(this_arg)?;
                if args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "TextEncoder.encodeInto requires exactly two arguments".into(),
                    ));
                }
                let source = args[0].as_string();
                let Value::TypedArray(destination) = &args[1] else {
                    return Err(Error::ScriptRuntime(
                        "TextEncoder.encodeInto destination must be a Uint8Array".into(),
                    ));
                };
                Some(self.text_encoder_encode_into_value(&source, destination)?)
            }
            "text_decoder_get_encoding" => {
                let (encoding, _, _) = Self::text_decoder_state_from_receiver(this_arg)?;
                Some(Value::String(encoding))
            }
            "text_decoder_get_fatal" => {
                let (_, fatal, _) = Self::text_decoder_state_from_receiver(this_arg)?;
                Some(Value::Bool(fatal))
            }
            "text_decoder_get_ignore_bom" => {
                let (_, _, ignore_bom) = Self::text_decoder_state_from_receiver(this_arg)?;
                Some(Value::Bool(ignore_bom))
            }
            "text_decoder_decode" => {
                if args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "TextDecoder.decode supports up to two arguments".into(),
                    ));
                }
                let (encoding, fatal, ignore_bom) =
                    Self::text_decoder_state_from_receiver(this_arg)?;
                Self::validate_text_decoder_decode_options(args.get(1))?;
                let bytes = self.text_decoder_input_bytes(args.first())?;
                Some(Value::String(Self::decode_text_decoder_bytes(
                    &encoding, &bytes, fatal, ignore_bom,
                )?))
            }
            "text_encoder_stream_get_encoding" => {
                Self::text_encoder_stream_state_from_receiver(this_arg)?;
                Some(Value::String("utf-8".to_string()))
            }
            "text_encoder_stream_get_readable" => {
                let (readable, _) = Self::text_encoder_stream_state_from_receiver(this_arg)?;
                Some(readable)
            }
            "text_encoder_stream_get_writable" => {
                let (_, writable) = Self::text_encoder_stream_state_from_receiver(this_arg)?;
                Some(writable)
            }
            "text_decoder_stream_get_encoding" => {
                let (encoding, _, _, _, _) =
                    Self::text_decoder_stream_state_from_receiver(this_arg)?;
                Some(Value::String(encoding))
            }
            "text_decoder_stream_get_fatal" => {
                let (_, fatal, _, _, _) = Self::text_decoder_stream_state_from_receiver(this_arg)?;
                Some(Value::Bool(fatal))
            }
            "text_decoder_stream_get_ignore_bom" => {
                let (_, _, ignore_bom, _, _) =
                    Self::text_decoder_stream_state_from_receiver(this_arg)?;
                Some(Value::Bool(ignore_bom))
            }
            "text_decoder_stream_get_readable" => {
                let (_, _, _, readable, _) =
                    Self::text_decoder_stream_state_from_receiver(this_arg)?;
                Some(readable)
            }
            "text_decoder_stream_get_writable" => {
                let (_, _, _, _, writable) =
                    Self::text_decoder_stream_state_from_receiver(this_arg)?;
                Some(writable)
            }
            "css_style_sheet_replace_sync" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "CSSStyleSheet.replaceSync requires exactly one argument".into(),
                    ));
                }
                let sheet = Self::css_style_sheet_object_from_receiver(this_arg)?;
                let replacement = args[0].as_string();
                let rules = if replacement.trim().is_empty() {
                    Vec::new()
                } else {
                    vec![Value::String(replacement)]
                };
                Self::object_set_entry(
                    &mut sheet.borrow_mut(),
                    INTERNAL_CSS_STYLE_SHEET_RULES_KEY.to_string(),
                    Self::new_array_value(rules),
                );
                Some(Value::Undefined)
            }
            "css_style_sheet_insert_rule" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "CSSStyleSheet.insertRule requires one or two arguments".into(),
                    ));
                }
                let sheet = Self::css_style_sheet_object_from_receiver(this_arg)?;
                let rule = Value::as_string(&args[0]);
                let existing_rules = {
                    let sheet_ref = sheet.borrow();
                    match Self::object_get_entry(&sheet_ref, INTERNAL_CSS_STYLE_SHEET_RULES_KEY) {
                        Some(Value::Array(rules)) => rules,
                        _ => Rc::new(RefCell::new(ArrayValue::new(Vec::new()))),
                    }
                };
                let mut rules_ref = existing_rules.borrow_mut();
                let default_index = rules_ref.len();
                let index = if let Some(index_value) = args.get(1) {
                    let requested = Self::value_to_i64(index_value);
                    if requested < 0 || (requested as usize) > rules_ref.len() {
                        return Err(Error::ScriptRuntime(
                            "CSSStyleSheet.insertRule index out of range".into(),
                        ));
                    }
                    requested as usize
                } else {
                    default_index
                };
                rules_ref.insert(index, Value::String(rule));
                drop(rules_ref);
                Self::object_set_entry(
                    &mut sheet.borrow_mut(),
                    INTERNAL_CSS_STYLE_SHEET_RULES_KEY.to_string(),
                    Value::Array(existing_rules),
                );
                Some(Value::Number(index as i64))
            }
            "computed_style_get_property_value" => {
                if args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getPropertyValue requires exactly one argument".into(),
                    ));
                }
                let (node, pseudo) = Self::computed_style_state_from_receiver(this_arg)?;
                let property_name = args[0].as_string();
                let value =
                    self.computed_style_property_value(node, pseudo.as_deref(), &property_name)?;
                Some(Value::String(value))
            }
            "computed_style_item" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "item requires zero or one argument".into(),
                    ));
                }
                let _ = Self::computed_style_state_from_receiver(this_arg)?;
                Some(Value::String(String::new()))
            }
            "dom_rect_list_item" => {
                if args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "item requires zero or one argument".into(),
                    ));
                }
                let Some(Value::Array(values)) = this_arg else {
                    return Err(Error::ScriptRuntime(
                        "TypeError: incompatible receiver for DOMRectList.item".into(),
                    ));
                };
                let values = values.borrow();
                if !Self::is_dom_rect_list_value(&values) {
                    return Err(Error::ScriptRuntime(
                        "TypeError: incompatible receiver for DOMRectList.item".into(),
                    ));
                }
                let index = args
                    .first()
                    .map(|value| match value {
                        Value::Number(number) => *number,
                        Value::Float(number) if number.is_finite() => *number as i64,
                        Value::BigInt(number) => number.to_string().parse::<i64>().unwrap_or(0),
                        other => other.as_string().trim().parse::<i64>().unwrap_or(0),
                    })
                    .unwrap_or(0)
                    .max(0) as usize;
                Some(values.get(index).cloned().unwrap_or(Value::Null))
            }
            _ => None,
        };
        Ok(value)
    }
}
