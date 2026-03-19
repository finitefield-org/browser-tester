use super::*;

impl Harness {
    pub(crate) fn eval_nodelist_member_call(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        match member {
            "item" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "item requires exactly one index argument".into(),
                    ));
                }
                let index = Self::value_to_i64(&evaluated_args[0]);
                if index < 0 {
                    return Ok(Some(Value::Null));
                }
                Ok(Some(
                    self.node_list_get(nodes, index as usize)
                        .map(|node| self.node_list_item_value(nodes, node))
                        .unwrap_or(Value::Null),
                ))
            }
            "namedItem" => {
                if !Self::node_list_is_html_collection(nodes) {
                    return Ok(None);
                }
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "namedItem requires exactly one name argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                let owner_form = {
                    let nodes_ref = nodes.borrow();
                    match nodes_ref.live_source {
                        Some(LiveNodeListSource::FormElements { form }) => Some(form),
                        _ => None,
                    }
                };
                if let Some(form) = owner_form {
                    return Ok(Some(
                        self.form_controls_named_item_value(form, name.as_str())?
                            .unwrap_or(Value::Null),
                    ));
                }
                Ok(Some(
                    self.html_collection_named_entries(nodes)
                        .into_iter()
                        .find(|(candidate, _)| candidate == &name)
                        .map(|(_, node)| Value::Node(node))
                        .unwrap_or(Value::Null),
                ))
            }
            "forEach" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "forEach requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                let snapshot = self.node_list_snapshot(nodes);
                for (idx, node) in snapshot.iter().copied().enumerate() {
                    let item_value = self.node_list_item_value(nodes, node);
                    let _ = self.execute_callback_value(
                        &callback,
                        &[
                            item_value,
                            Value::Number(idx as i64),
                            Value::NodeList(nodes.clone()),
                        ],
                        event,
                    )?;
                }
                Ok(Some(Value::Undefined))
            }
            "entries" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "entries does not take arguments".into(),
                    ));
                }
                let snapshot = self.node_list_snapshot(nodes);
                let mut iterator_values = Vec::with_capacity(snapshot.len());
                for (index, node) in snapshot.iter().copied().enumerate() {
                    iterator_values.push(Self::new_array_value(vec![
                        Value::Number(index as i64),
                        self.node_list_item_value(nodes, node),
                    ]));
                }
                Ok(Some(self.new_iterator_value(iterator_values)))
            }
            "keys" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("keys does not take arguments".into()));
                }
                let len = self.node_list_len(nodes);
                Ok(Some(self.new_iterator_value(
                    (0..len).map(|index| Value::Number(index as i64)).collect(),
                )))
            }
            "values" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "values does not take arguments".into(),
                    ));
                }
                let mut iterator_values = Vec::new();
                for node in self.node_list_snapshot(nodes) {
                    iterator_values.push(self.node_list_item_value(nodes, node));
                }
                Ok(Some(self.new_iterator_value(iterator_values)))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn eval_named_node_map_member_call(
        &mut self,
        object: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        let owner = {
            let entries = object.borrow();
            if !Self::is_named_node_map_object(&entries) {
                return Ok(None);
            }
            Self::named_node_map_owner_node(&entries)
                .filter(|node| self.dom.element(*node).is_some())
        };

        let value = match member {
            "item" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.item requires exactly one index argument".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Ok(Some(Value::Null));
                };
                let index = Self::value_to_i64(&evaluated_args[0]);
                if index < 0 {
                    Value::Null
                } else {
                    self.named_node_map_entries(owner)
                        .get(index as usize)
                        .map(|(name, value)| Self::new_attr_object_value(name, value, Some(owner)))
                        .unwrap_or(Value::Null)
                }
            }
            "getNamedItem" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.getNamedItem requires exactly one argument".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Ok(Some(Value::Null));
                };
                self.eval_node_member_call(owner, "getAttributeNode", evaluated_args, event)?
                    .unwrap_or(Value::Null)
            }
            "setNamedItem" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.setNamedItem requires exactly one argument".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Err(Error::ScriptRuntime(
                        "setNamedItem target is not an element".into(),
                    ));
                };
                self.eval_node_member_call(owner, "setAttributeNode", evaluated_args, event)?
                    .unwrap_or(Value::Null)
            }
            "removeNamedItem" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.removeNamedItem requires exactly one argument".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeNamedItem': The attribute was not found"
                            .into(),
                    ));
                };
                let attr = self
                    .eval_node_member_call(owner, "getAttributeNode", evaluated_args, event)?
                    .unwrap_or(Value::Null);
                if matches!(attr, Value::Null) {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeNamedItem': The attribute was not found"
                            .into(),
                    ));
                }
                let _ =
                    self.eval_node_member_call(owner, "removeAttribute", evaluated_args, event)?;
                match attr {
                    Value::Object(attr_object) => {
                        let (name, value) = {
                            let entries = attr_object.borrow();
                            (
                                Self::object_get_entry(&entries, "name")
                                    .map(|entry| entry.as_string())
                                    .unwrap_or_default(),
                                Self::object_get_entry(&entries, "value")
                                    .map(|entry| entry.as_string())
                                    .unwrap_or_default(),
                            )
                        };
                        Self::new_attr_object_value(&name, &value, None)
                    }
                    _ => Value::Null,
                }
            }
            "getNamedItemNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.getNamedItemNS requires exactly two arguments".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Ok(Some(Value::Null));
                };
                self.eval_node_member_call(owner, "getAttributeNodeNS", evaluated_args, event)?
                    .unwrap_or(Value::Null)
            }
            "setNamedItemNS" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.setNamedItemNS requires exactly one argument".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Err(Error::ScriptRuntime(
                        "setNamedItemNS target is not an element".into(),
                    ));
                };
                self.eval_node_member_call(owner, "setAttributeNodeNS", evaluated_args, event)?
                    .unwrap_or(Value::Null)
            }
            "removeNamedItemNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "NamedNodeMap.removeNamedItemNS requires exactly two arguments".into(),
                    ));
                }
                let Some(owner) = owner else {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeNamedItemNS': The attribute was not found"
                            .into(),
                    ));
                };
                let attr = self
                    .eval_node_member_call(owner, "getAttributeNodeNS", evaluated_args, event)?
                    .unwrap_or(Value::Null);
                if matches!(attr, Value::Null) {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeNamedItemNS': The attribute was not found"
                            .into(),
                    ));
                }
                let _ =
                    self.eval_node_member_call(owner, "removeAttributeNS", evaluated_args, event)?;
                match attr {
                    Value::Object(attr_object) => {
                        let (name, value) = {
                            let entries = attr_object.borrow();
                            (
                                Self::object_get_entry(&entries, "name")
                                    .map(|entry| entry.as_string())
                                    .unwrap_or_default(),
                                Self::object_get_entry(&entries, "value")
                                    .map(|entry| entry.as_string())
                                    .unwrap_or_default(),
                            )
                        };
                        Self::new_attr_object_value(&name, &value, None)
                    }
                    _ => Value::Null,
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
