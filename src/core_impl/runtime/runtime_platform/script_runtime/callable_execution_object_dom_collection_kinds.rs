use super::*;

impl Harness {
    pub(crate) fn execute_object_callable_dom_collection_kind(
        &mut self,
        kind: &str,
        args: &[Value],
        event: &EventState,
        caller_env: Option<&HashMap<String, Value>>,
        this_arg: Option<&Value>,
    ) -> Result<Option<Value>> {
        let value = match kind {
            "class_list_add" => {
                let node = Self::class_list_node_from_receiver(this_arg)?;
                for class_name in args {
                    self.dom.class_add(node, &class_name.as_string())?;
                }
                Some(Value::Undefined)
            }
            "class_list_remove" => {
                let node = Self::class_list_node_from_receiver(this_arg)?;
                for class_name in args {
                    self.dom.class_remove(node, &class_name.as_string())?;
                }
                Some(Value::Undefined)
            }
            "class_list_toggle" => {
                let node = Self::class_list_node_from_receiver(this_arg)?;
                let Some(class_name) = args.first() else {
                    return Err(Error::ScriptRuntime(
                        "DOMTokenList.toggle requires at least one argument".into(),
                    ));
                };
                let class_name = class_name.as_string();
                let toggled = if let Some(force) = args.get(1) {
                    if force.truthy() {
                        self.dom.class_add(node, &class_name)?;
                        true
                    } else {
                        self.dom.class_remove(node, &class_name)?;
                        false
                    }
                } else {
                    self.dom.class_toggle(node, &class_name)?
                };
                Some(Value::Bool(toggled))
            }
            "class_list_contains" => {
                let node = Self::class_list_node_from_receiver(this_arg)?;
                let Some(class_name) = args.first() else {
                    return Ok(Some(Value::Bool(false)));
                };
                Some(Value::Bool(
                    self.dom.class_contains(node, &class_name.as_string())?,
                ))
            }
            "class_list_replace" => {
                let node = Self::class_list_node_from_receiver(this_arg)?;
                let Some(old_class_name) = args.first() else {
                    return Ok(Some(Value::Bool(false)));
                };
                let Some(new_class_name) = args.get(1) else {
                    return Ok(Some(Value::Bool(false)));
                };
                Some(Value::Bool(self.dom.class_replace(
                    node,
                    &old_class_name.as_string(),
                    &new_class_name.as_string(),
                )?))
            }
            "class_list_item" => {
                let node = Self::class_list_node_from_receiver(this_arg)?;
                let index = args.first().map(Self::value_to_i64).unwrap_or(0);
                if index < 0 {
                    return Ok(Some(Value::Null));
                }
                let classes = class_tokens(self.dom.attr(node, "class").as_deref());
                Some(
                    classes
                        .get(index as usize)
                        .cloned()
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                )
            }
            "class_list_for_each" => {
                let node = Self::class_list_node_from_receiver(this_arg)?;
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "DOMTokenList.forEach requires a callback".into(),
                    ));
                }
                let callback = args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let this_value = args.get(1).cloned().unwrap_or(Value::Undefined);
                let class_list_object = this_arg.cloned().unwrap_or(Value::Undefined);
                let classes = class_tokens(self.dom.attr(node, "class").as_deref());
                for (index, class_name) in classes.iter().enumerate() {
                    let callback_args = [
                        Value::String(class_name.clone()),
                        Value::Number(index as i64),
                        class_list_object.clone(),
                    ];
                    let _ = self.execute_callable_value_with_this_and_env(
                        &callback,
                        &callback_args,
                        event,
                        caller_env,
                        Some(this_value.clone()),
                    )?;
                }
                Some(Value::Undefined)
            }
            "class_list_keys" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "DOMTokenList.keys does not take arguments".into(),
                    ));
                }
                let node = Self::class_list_node_from_receiver(this_arg)?;
                let classes = class_tokens(self.dom.attr(node, "class").as_deref());
                Some(
                    self.new_iterator_value(
                        (0..classes.len())
                            .map(|index| Value::Number(index as i64))
                            .collect(),
                    ),
                )
            }
            "class_list_values" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "DOMTokenList.values does not take arguments".into(),
                    ));
                }
                let node = Self::class_list_node_from_receiver(this_arg)?;
                Some(
                    self.new_iterator_value(
                        class_tokens(self.dom.attr(node, "class").as_deref())
                            .into_iter()
                            .map(Value::String)
                            .collect(),
                    ),
                )
            }
            "class_list_entries" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "DOMTokenList.entries does not take arguments".into(),
                    ));
                }
                let node = Self::class_list_node_from_receiver(this_arg)?;
                Some(
                    self.new_iterator_value(
                        class_tokens(self.dom.attr(node, "class").as_deref())
                            .into_iter()
                            .enumerate()
                            .map(|(index, class_name)| {
                                Self::new_array_value(vec![
                                    Value::Number(index as i64),
                                    Value::String(class_name),
                                ])
                            })
                            .collect(),
                    ),
                )
            }
            "class_list_to_string" => {
                let node = Self::class_list_node_from_receiver(this_arg)?;
                Some(Value::String(
                    class_tokens(self.dom.attr(node, "class").as_deref()).join(" "),
                ))
            }
            "named_node_map_item" => {
                let (object, _owner) = Self::named_node_map_receiver_object_and_owner(this_arg)?;
                Some(
                    self.eval_named_node_map_member_call(&object, "item", args, event)?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))?,
                )
            }
            "named_node_map_get_named_item" => {
                let (object, _owner) = Self::named_node_map_receiver_object_and_owner(this_arg)?;
                Some(
                    self.eval_named_node_map_member_call(&object, "getNamedItem", args, event)?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))?,
                )
            }
            "named_node_map_set_named_item" => {
                let (object, _owner) = Self::named_node_map_receiver_object_and_owner(this_arg)?;
                Some(
                    self.eval_named_node_map_member_call(&object, "setNamedItem", args, event)?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))?,
                )
            }
            "named_node_map_remove_named_item" => {
                let (object, _owner) = Self::named_node_map_receiver_object_and_owner(this_arg)?;
                Some(
                    self.eval_named_node_map_member_call(&object, "removeNamedItem", args, event)?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))?,
                )
            }
            "named_node_map_get_named_item_ns" => {
                let (object, _owner) = Self::named_node_map_receiver_object_and_owner(this_arg)?;
                Some(
                    self.eval_named_node_map_member_call(&object, "getNamedItemNS", args, event)?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))?,
                )
            }
            "named_node_map_set_named_item_ns" => {
                let (object, _owner) = Self::named_node_map_receiver_object_and_owner(this_arg)?;
                Some(
                    self.eval_named_node_map_member_call(&object, "setNamedItemNS", args, event)?
                        .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))?,
                )
            }
            "named_node_map_remove_named_item_ns" => {
                let (object, _owner) = Self::named_node_map_receiver_object_and_owner(this_arg)?;
                Some(
                    self.eval_named_node_map_member_call(
                        &object,
                        "removeNamedItemNS",
                        args,
                        event,
                    )?
                    .ok_or_else(|| Self::incompatible_receiver_error("named_node_map"))?,
                )
            }
            "named_node_map_for_each" => {
                if args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.forEach requires a callback".into(),
                    ));
                }
                let callback = args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let (object, owner) = Self::named_node_map_receiver_object_and_owner(this_arg)?;
                let this_value = args.get(1).cloned().unwrap_or(Value::Undefined);
                let attrs = self.named_node_map_entries(owner);
                for (index, (name, value)) in attrs.iter().enumerate() {
                    let callback_args = [
                        Self::new_attr_object_value(name, value, Some(owner)),
                        Value::Number(index as i64),
                        Value::Object(object.clone()),
                    ];
                    let _ = self.execute_callable_value_with_this_and_env(
                        &callback,
                        &callback_args,
                        event,
                        caller_env,
                        Some(this_value.clone()),
                    )?;
                }
                Some(Value::Undefined)
            }
            "named_node_map_keys" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.keys does not take arguments".into(),
                    ));
                }
                let (_object, owner) = Self::named_node_map_receiver_object_and_owner(this_arg)?;
                Some(
                    self.new_iterator_value(
                        (0..self.named_node_map_entries(owner).len())
                            .map(|index| Value::Number(index as i64))
                            .collect(),
                    ),
                )
            }
            "named_node_map_values" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.values does not take arguments".into(),
                    ));
                }
                let (_object, owner) = Self::named_node_map_receiver_object_and_owner(this_arg)?;
                Some(
                    self.new_iterator_value(
                        self.named_node_map_entries(owner)
                            .into_iter()
                            .map(|(name, value)| {
                                Self::new_attr_object_value(&name, &value, Some(owner))
                            })
                            .collect(),
                    ),
                )
            }
            "named_node_map_entries" => {
                if !args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.entries does not take arguments".into(),
                    ));
                }
                let (_object, owner) = Self::named_node_map_receiver_object_and_owner(this_arg)?;
                Some(
                    self.new_iterator_value(
                        self.named_node_map_entries(owner)
                            .into_iter()
                            .enumerate()
                            .map(|(index, (name, value))| {
                                Self::new_array_value(vec![
                                    Value::Number(index as i64),
                                    Self::new_attr_object_value(&name, &value, Some(owner)),
                                ])
                            })
                            .collect(),
                    ),
                )
            }
            _ => None,
        };
        Ok(value)
    }
}
