use super::*;

impl Harness {
    pub(crate) fn try_eval_node_element_attribute_member_call(
        &mut self,
        node: NodeId,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "attachShadow" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "attachShadow requires exactly one options argument".into(),
                    ));
                }
                let root = self.attach_shadow_root(node, &evaluated_args[0])?;
                Ok(Some(Value::Node(root)))
            }
            "getAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if name == "nonce" {
                    return Ok(Some(if self.dom.attr(node, "nonce").is_some() {
                        Value::String(String::new())
                    } else {
                        Value::Null
                    }));
                }
                Ok(Some(
                    self.dom
                        .attr(node, &name)
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                ))
            }
            "getAttributeNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string().to_ascii_lowercase();
                Ok(Some(self.get_attribute_ns_value(
                    node,
                    namespace_uri.as_deref(),
                    &local_name,
                )))
            }
            "getBoundingClientRect" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getBoundingClientRect takes no arguments".into(),
                    ));
                }
                Ok(Some(self.get_bounding_client_rect_value(node)?))
            }
            "getClientRects" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getClientRects takes no arguments".into(),
                    ));
                }
                Ok(Some(self.get_client_rects_value(node)?))
            }
            "getHTML" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "getHTML supports zero or one options argument".into(),
                    ));
                }
                Ok(Some(
                    self.element_get_html_value(node, evaluated_args.first())?,
                ))
            }
            "getAttributeNode" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNode requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                Ok(Some(
                    self.dom
                        .attr(node, &name)
                        .map(|value| Self::new_attr_object_value(&name, &value, Some(node)))
                        .unwrap_or(Value::Null),
                ))
            }
            "getAttributeNodeNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNodeNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string().to_ascii_lowercase();
                Ok(Some(self.get_attribute_node_ns_value(
                    node,
                    namespace_uri.as_deref(),
                    &local_name,
                )))
            }
            "setAttribute" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "setAttribute requires exactly two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if !is_valid_create_attribute_name(&name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                let value = evaluated_args[1].as_string();
                self.dom.set_attr(node, &name, &value)?;
                Ok(Some(Value::Undefined))
            }
            "setAttributeNS" => {
                if evaluated_args.len() != 3 {
                    return Err(Error::ScriptRuntime(
                        "setAttributeNS requires exactly three arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let qualified_name = evaluated_args[1].as_string().to_ascii_lowercase();
                if !is_valid_qualified_attribute_name(&qualified_name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                if namespace_uri.is_none() && qualified_name.contains(':') {
                    return Err(Error::ScriptRuntime(
                        "NamespaceError: prefix requires a namespace".into(),
                    ));
                }
                let value = evaluated_args[2].as_string();
                let local_name =
                    Self::local_name_from_qualified_name(&qualified_name).to_ascii_lowercase();
                let replaced = {
                    let Some(element) = self.dom.element(node) else {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNS target is not an element".into(),
                        ));
                    };
                    let mut matches = element
                        .attrs
                        .iter()
                        .filter_map(|(existing_name, _)| {
                            let existing_local_name =
                                Self::local_name_from_qualified_name(existing_name);
                            if !existing_local_name.eq_ignore_ascii_case(&local_name) {
                                return None;
                            }
                            let existing_namespace = self
                                .attribute_namespace_uri_for_qualified_name(node, existing_name);
                            let namespace_matches =
                                match (namespace_uri.as_deref(), existing_namespace.as_deref()) {
                                    (None, None) => true,
                                    (Some(expected), Some(actual)) => expected == actual,
                                    _ => false,
                                };
                            if !namespace_matches {
                                return None;
                            }
                            Some(existing_name.clone())
                        })
                        .collect::<Vec<_>>();
                    matches.sort();
                    matches.into_iter().next()
                };
                if let Some(replaced_name) = replaced {
                    self.dom.remove_attr(node, &replaced_name)?;
                }
                self.dom.set_attr(node, &qualified_name, &value)?;
                Ok(Some(Value::Undefined))
            }
            "setAttributeNode" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setAttributeNode requires exactly one argument".into(),
                    ));
                }
                let attr_object = match evaluated_args.first() {
                    Some(Value::Object(object)) => object.clone(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNode argument must be an Attr".into(),
                        ));
                    }
                };
                let (name, value): (String, String) = {
                    let entries = attr_object.borrow();
                    if !Self::is_attr_object(&entries) {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNode argument must be an Attr".into(),
                        ));
                    }
                    let name = Self::object_get_entry(&entries, "name")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    if !is_valid_create_attribute_name(&name) {
                        return Err(Error::ScriptRuntime(
                            "InvalidCharacterError: attribute name is not a valid XML name".into(),
                        ));
                    }
                    let value = Self::object_get_entry(&entries, "value")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default();
                    (name, value)
                };
                let replaced_value = self.dom.attr(node, &name);
                self.dom.set_attr(node, &name, &value)?;

                {
                    let mut entries = attr_object.borrow_mut();
                    Self::object_set_entry(
                        &mut entries,
                        "name".to_string(),
                        Value::String(name.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "value".to_string(),
                        Value::String(value.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "ownerElement".to_string(),
                        Value::Node(node),
                    );
                }

                Ok(Some(
                    replaced_value
                        .map(|old| Self::new_attr_object_value(&name, &old, None))
                        .unwrap_or(Value::Null),
                ))
            }
            "setAttributeNodeNS" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "setAttributeNodeNS requires exactly one argument".into(),
                    ));
                }
                let attr_object = match evaluated_args.first() {
                    Some(Value::Object(object)) => object.clone(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNodeNS argument must be an Attr".into(),
                        ));
                    }
                };
                let (name, value, owner_element): (String, String, Option<NodeId>) = {
                    let entries = attr_object.borrow();
                    if !Self::is_attr_object(&entries) {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNodeNS argument must be an Attr".into(),
                        ));
                    }
                    let name = Self::object_get_entry(&entries, "name")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    let value = Self::object_get_entry(&entries, "value")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default();
                    let owner_element = match Self::object_get_entry(&entries, "ownerElement") {
                        Some(Value::Node(owner)) => Some(owner),
                        _ => None,
                    };
                    (name, value, owner_element)
                };

                let namespace_uri = owner_element
                    .and_then(|owner| self.attribute_namespace_uri_for_qualified_name(owner, &name))
                    .or_else(|| self.attribute_namespace_uri_for_qualified_name(node, &name));
                let local_name = Self::local_name_from_qualified_name(&name).to_ascii_lowercase();

                let replaced = {
                    let Some(element) = self.dom.element(node) else {
                        return Err(Error::ScriptRuntime(
                            "setAttributeNodeNS target is not an element".into(),
                        ));
                    };
                    let mut matches = element
                        .attrs
                        .iter()
                        .filter_map(|(qualified_name, existing_value)| {
                            let candidate_local_name =
                                Self::local_name_from_qualified_name(qualified_name);
                            if !candidate_local_name.eq_ignore_ascii_case(&local_name) {
                                return None;
                            }
                            let candidate_namespace = self
                                .attribute_namespace_uri_for_qualified_name(node, qualified_name);
                            let namespace_matches =
                                match (namespace_uri.as_deref(), candidate_namespace.as_deref()) {
                                    (None, None) => true,
                                    (Some(expected), Some(actual)) => expected == actual,
                                    _ => false,
                                };
                            if !namespace_matches {
                                return None;
                            }
                            Some((qualified_name.clone(), existing_value.clone()))
                        })
                        .collect::<Vec<_>>();
                    matches.sort_by(|(left, _), (right, _)| left.cmp(right));
                    matches.into_iter().next()
                };

                if let Some((replaced_name, _)) = replaced.as_ref() {
                    self.dom.remove_attr(node, replaced_name)?;
                }
                self.dom.set_attr(node, &name, &value)?;

                {
                    let mut entries = attr_object.borrow_mut();
                    Self::object_set_entry(
                        &mut entries,
                        "name".to_string(),
                        Value::String(name.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "value".to_string(),
                        Value::String(value.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "ownerElement".to_string(),
                        Value::Node(node),
                    );
                }

                Ok(Some(
                    replaced
                        .map(|(old_name, old_value)| {
                            Self::new_attr_object_value(&old_name, &old_value, None)
                        })
                        .unwrap_or(Value::Null),
                ))
            }
            "removeAttributeNode" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeAttributeNode requires exactly one argument".into(),
                    ));
                }
                let attr_object = match evaluated_args.first() {
                    Some(Value::Object(object)) => object.clone(),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "removeAttributeNode argument must be an Attr".into(),
                        ));
                    }
                };
                let (name, owner_matches_node): (String, bool) = {
                    let entries = attr_object.borrow();
                    if !Self::is_attr_object(&entries) {
                        return Err(Error::ScriptRuntime(
                            "removeAttributeNode argument must be an Attr".into(),
                        ));
                    }
                    let name = Self::object_get_entry(&entries, "name")
                        .map(|entry| entry.as_string())
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    let owner_matches_node = matches!(Self::object_get_entry(&entries, "ownerElement"), Some(Value::Node(owner)) if owner == node);
                    (name, owner_matches_node)
                };

                let Some(current_value) = self.dom.attr(node, &name) else {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeAttributeNode': The attribute node was not found"
                            .into(),
                    ));
                };
                if !owner_matches_node {
                    return Err(Error::ScriptRuntime(
                        "NotFoundError: Failed to execute 'removeAttributeNode': The attribute node was not found"
                            .into(),
                    ));
                }
                self.dom.remove_attr(node, &name)?;
                {
                    let mut entries = attr_object.borrow_mut();
                    Self::object_set_entry(
                        &mut entries,
                        "name".to_string(),
                        Value::String(name.clone()),
                    );
                    Self::object_set_entry(
                        &mut entries,
                        "value".to_string(),
                        Value::String(current_value),
                    );
                    Self::object_set_entry(&mut entries, "ownerElement".to_string(), Value::Null);
                }
                Ok(Some(Value::Object(attr_object)))
            }
            "hasAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "hasAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                Ok(Some(Value::Bool(self.dom.has_attr(node, &name)?)))
            }
            "hasAttributeNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "hasAttributeNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string().to_ascii_lowercase();
                Ok(Some(Value::Bool(self.has_attribute_ns_value(
                    node,
                    namespace_uri.as_deref(),
                    &local_name,
                ))))
            }
            "hasAttributes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "hasAttributes takes no arguments".into(),
                    ));
                }
                let has_attributes = self
                    .dom
                    .element(node)
                    .map(|element| !element.attrs.is_empty())
                    .ok_or_else(|| {
                        Error::ScriptRuntime("hasAttributes target is not an element".into())
                    })?;
                Ok(Some(Value::Bool(has_attributes)))
            }
            "removeAttribute" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeAttribute requires exactly one argument".into(),
                    ));
                }
                let name = evaluated_args[0].as_string();
                self.dom.remove_attr(node, &name)?;
                Ok(Some(Value::Undefined))
            }
            "removeAttributeNS" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "removeAttributeNS requires exactly two arguments".into(),
                    ));
                }
                let namespace_uri =
                    Self::namespace_uri_from_create_element_ns_argument(&evaluated_args[0]);
                let local_name = evaluated_args[1].as_string().to_ascii_lowercase();
                self.remove_attribute_ns(node, namespace_uri.as_deref(), &local_name)?;
                Ok(Some(Value::Undefined))
            }
            "getAttributeNames" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "getAttributeNames takes no arguments".into(),
                    ));
                }
                let element = self.dom.element(node).ok_or_else(|| {
                    Error::ScriptRuntime("getAttributeNames target is not an element".into())
                })?;
                let mut names = element.attrs.keys().cloned().collect::<Vec<_>>();
                names.sort();
                Ok(Some(Self::new_array_value(
                    names.into_iter().map(Value::String).collect(),
                )))
            }
            "toggleAttribute" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "toggleAttribute requires one or two arguments".into(),
                    ));
                }
                let name = evaluated_args[0].as_string().to_ascii_lowercase();
                if !is_valid_create_attribute_name(&name) {
                    return Err(Error::ScriptRuntime(
                        "InvalidCharacterError: attribute name is not a valid XML name".into(),
                    ));
                }
                let has = self.dom.has_attr(node, &name)?;
                let next = if evaluated_args.len() == 2 {
                    evaluated_args[1].truthy()
                } else {
                    !has
                };
                if next {
                    self.dom.set_attr(node, &name, "")?;
                } else {
                    self.dom.remove_attr(node, &name)?;
                }
                Ok(Some(Value::Bool(next)))
            }
            "matches" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "matches requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_matches_selector_value(node, &selector)?))
            }
            "closest" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "closest requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_closest_selector_value(node, &selector)?))
            }
            "querySelector" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelector requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_query_selector_value(node, &selector)?))
            }
            "querySelectorAll" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "querySelectorAll requires exactly one selector argument".into(),
                    ));
                }
                let selector = evaluated_args[0].as_string();
                Ok(Some(self.eval_query_selector_all_value(node, &selector)?))
            }
            _ => Ok(None),
        }
    }
}
