use super::*;

impl Harness {
    pub(crate) fn callable_receiver_from_this_arg(
        &self,
        this_arg: Option<Value>,
        method: &str,
    ) -> Result<Value> {
        let Some(target) = this_arg else {
            return Err(Error::ScriptRuntime(format!(
                "Function.prototype.{method} called on non-callable value"
            )));
        };
        if !self.is_callable_value(&target) {
            return Err(Error::ScriptRuntime(format!(
                "Function.prototype.{method} called on non-callable value"
            )));
        }
        Ok(target)
    }

    pub(crate) fn receiver_builtin_callable_components(
        callable: &Value,
    ) -> Result<(String, String)> {
        let Value::Object(entries) = callable else {
            return Err(Error::ScriptRuntime(
                "builtin method has invalid internal state".into(),
            ));
        };
        let entries = entries.borrow();
        let family = match Self::object_get_entry(&entries, "__bt_receiver_builtin_family") {
            Some(Value::String(family)) => family,
            _ => {
                return Err(Error::ScriptRuntime(
                    "builtin method has invalid internal state".into(),
                ));
            }
        };
        let member = match Self::object_get_entry(&entries, "__bt_receiver_builtin_member") {
            Some(Value::String(member)) => member,
            _ => {
                return Err(Error::ScriptRuntime(
                    "builtin method has invalid internal state".into(),
                ));
            }
        };
        Ok((family, member))
    }

    pub(crate) fn static_method_name(callable: &Value) -> Result<String> {
        let Value::Object(entries) = callable else {
            return Err(Error::ScriptRuntime(
                "builtin method has invalid internal state".into(),
            ));
        };
        let entries = entries.borrow();
        match Self::object_get_entry(&entries, INTERNAL_STATIC_METHOD_NAME_KEY) {
            Some(Value::String(method)) => Ok(method),
            _ => Err(Error::ScriptRuntime(
                "builtin method has invalid internal state".into(),
            )),
        }
    }

    pub(crate) fn typed_array_static_method_components(
        callable: &Value,
    ) -> Result<(TypedArrayConstructorKind, String)> {
        let Value::Object(entries) = callable else {
            return Err(Error::ScriptRuntime(
                "builtin method has invalid internal state".into(),
            ));
        };
        let entries = entries.borrow();
        let kind = match Self::object_get_entry(&entries, INTERNAL_STATIC_TYPED_ARRAY_KIND_KEY) {
            Some(Value::TypedArrayConstructor(kind)) => kind,
            _ => {
                return Err(Error::ScriptRuntime(
                    "builtin method has invalid internal state".into(),
                ));
            }
        };
        let method = match Self::object_get_entry(&entries, INTERNAL_STATIC_METHOD_NAME_KEY) {
            Some(Value::String(method)) => method,
            _ => {
                return Err(Error::ScriptRuntime(
                    "builtin method has invalid internal state".into(),
                ));
            }
        };
        Ok((kind, method))
    }

    pub(crate) fn incompatible_receiver_error(family: &str) -> Error {
        let label = match family {
            "array" => "Array",
            "date" => "Date",
            "map" => "Map",
            "worker" => "Worker",
            "node_list" => "NodeList",
            "image_bitmap" => "ImageBitmap",
            "text_track" => "TextTrack",
            "time_ranges" => "TimeRanges",
            "animation" => "Animation",
            "radio_node_list" => "RadioNodeList",
            "html_form" => "HTMLFormElement",
            "html_media" => "HTMLMediaElement",
            "html_collection" => "HTMLCollection",
            "weak_map" => "WeakMap",
            "set" => "Set",
            "weak_set" => "WeakSet",
            "location" => "Location",
            "string" => "String",
            "typed_array" => "TypedArray",
            "boolean" => "Boolean",
            "number" => "Number",
            "bigint" => "BigInt",
            "symbol" => "Symbol",
            "regexp" => "RegExp",
            "intl_collator" => "Intl.Collator",
            "intl_date_time_format" => "Intl.DateTimeFormat",
            "intl_display_names" => "Intl.DisplayNames",
            "intl_duration_format" => "Intl.DurationFormat",
            "intl_list_format" => "Intl.ListFormat",
            "intl_locale" => "Intl.Locale",
            "intl_number_format" => "Intl.NumberFormat",
            "intl_plural_rules" => "Intl.PluralRules",
            "intl_relative_time_format" => "Intl.RelativeTimeFormat",
            "intl_segmenter" => "Intl.Segmenter",
            "object" => "Object",
            "document" => "Document",
            "parsed_document" => "Document",
            "dom_parser" => "DOMParser",
            "xml_serializer" => "XMLSerializer",
            "tree_walker" => "TreeWalker",
            "range" => "Range",
            "selection" => "Selection",
            "event" => "Event",
            "keyboard_event" => "KeyboardEvent",
            "pointer_event" => "PointerEvent",
            "navigate_event" => "NavigateEvent",
            "data_transfer" => "DataTransfer",
            "data_transfer_item" => "DataTransferItem",
            "data_transfer_item_list" => "DataTransferItemList",
            "match_media" => "MediaQueryList",
            "cookie_store" => "CookieStore",
            "cache_storage" => "CacheStorage",
            "cache" => "Cache",
            "node" => "Node",
            "named_node_map" => "NamedNodeMap",
            "url" => "URL",
            "url_search_params" => "URLSearchParams",
            "storage" => "Storage",
            "form_data" => "FormData",
            "blob" => "Blob",
            "array_buffer" => "ArrayBuffer",
            "promise" => "Promise",
            "canvas_2d_context" => "CanvasRenderingContext2D",
            _ => "builtin method",
        };
        Error::ScriptRuntime(format!("{label} method called on incompatible receiver"))
    }

    pub(crate) fn coerce_string_method_receiver(&mut self, receiver: &Value) -> Result<String> {
        match receiver {
            Value::Null | Value::Undefined => Err(Self::incompatible_receiver_error("string")),
            Value::Symbol(_) => Err(Error::ScriptRuntime(
                "Cannot convert a Symbol value to a string".into(),
            )),
            Value::Object(object) => {
                let entries = object.borrow();
                if let Some(value) = Self::string_wrapper_value_from_object(&entries) {
                    return Ok(value);
                }
                if Self::symbol_wrapper_id_from_object(&entries).is_some() {
                    return Err(Error::ScriptRuntime(
                        "Cannot convert a Symbol value to a string".into(),
                    ));
                }
                drop(entries);
                self.coerce_to_string_for_tostring(receiver)
            }
            _ => self.coerce_to_string_for_tostring(receiver),
        }
    }

    pub(crate) fn execute_function_prototype_member(
        &mut self,
        member: &str,
        receiver: &Value,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
    ) -> Result<Value> {
        if !self.is_callable_value(receiver) {
            return Err(Error::ScriptRuntime(format!(
                "Function.prototype.{member} called on non-callable value"
            )));
        }
        match member {
            "call" => {
                let call_this = args.first().cloned().unwrap_or(Value::Undefined);
                let call_args = args.get(1..).unwrap_or(&[]);
                self.execute_callable_value_with_this_and_env(
                    receiver,
                    call_args,
                    event,
                    caller_env,
                    Some(call_this),
                )
            }
            "apply" => {
                let call_this = args.first().cloned().unwrap_or(Value::Undefined);
                let call_args = if let Some(args_value) = args.get(1) {
                    self.apply_arguments_from_value(args_value)?
                } else {
                    Vec::new()
                };
                self.execute_callable_value_with_this_and_env(
                    receiver,
                    &call_args,
                    event,
                    caller_env,
                    Some(call_this),
                )
            }
            "bind" => {
                let bound_this = args.first().cloned().unwrap_or(Value::Undefined);
                let bound_args = args.get(1..).unwrap_or(&[]).to_vec();
                Ok(Self::new_bound_function_callable(
                    receiver.clone(),
                    bound_this,
                    bound_args,
                ))
            }
            "toString" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Function.prototype.toString does not take arguments".into(),
                    ));
                }
                Ok(Value::String(
                    self.callable_source_text(receiver)
                        .unwrap_or_else(|| "function () { [native code] }".to_string()),
                ))
            }
            _ => Err(Error::ScriptRuntime(format!(
                "unsupported Function.prototype method: {member}"
            ))),
        }
    }

    fn apply_arguments_from_value(&mut self, value: &Value) -> Result<Vec<Value>> {
        match value {
            Value::Undefined | Value::Null => Ok(Vec::new()),
            Value::Array(values) => Ok(values.borrow().clone()),
            Value::NodeList(nodes) => Ok(self
                .node_list_snapshot(nodes)
                .into_iter()
                .map(Value::Node)
                .collect()),
            Value::TypedArray(array) => self.typed_array_snapshot(array),
            Value::Object(_) | Value::Function(_) => {
                let length = Self::value_to_i64(&self.object_property_from_value(value, "length")?);
                let length = length.max(0) as usize;
                let mut out = Vec::with_capacity(length);
                for index in 0..length {
                    out.push(self.object_property_from_value(value, &index.to_string())?);
                }
                Ok(out)
            }
            _ => Err(Error::ScriptRuntime(
                "Function.prototype.apply requires array-like arguments".into(),
            )),
        }
    }

    pub(crate) fn bound_callable_components(
        callable: &Value,
    ) -> Result<(Value, Value, Vec<Value>)> {
        let Value::Object(entries) = callable else {
            return Err(Error::ScriptRuntime(
                "bound function has invalid internal state".into(),
            ));
        };
        let entries = entries.borrow();
        let target = Self::object_get_entry(&entries, INTERNAL_BOUND_CALLABLE_TARGET_KEY)
            .ok_or_else(|| Error::ScriptRuntime("bound function has invalid target".into()))?;
        let bound_this = Self::object_get_entry(&entries, INTERNAL_BOUND_CALLABLE_THIS_KEY)
            .unwrap_or(Value::Undefined);
        let bound_args = match Self::object_get_entry(&entries, INTERNAL_BOUND_CALLABLE_ARGS_KEY) {
            Some(Value::Array(values)) => values.borrow().clone(),
            Some(Value::Undefined) | None => Vec::new(),
            _ => {
                return Err(Error::ScriptRuntime(
                    "bound function has invalid bound arguments".into(),
                ));
            }
        };
        Ok((target, bound_this, bound_args))
    }
}
