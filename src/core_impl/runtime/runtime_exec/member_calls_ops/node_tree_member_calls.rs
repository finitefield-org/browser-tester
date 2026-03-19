use super::*;

impl Harness {
    pub(crate) fn try_eval_node_tree_member_call(
        &mut self,
        node: NodeId,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        match member {
            "replaceWith" => Ok(Some(
                self.eval_node_replace_with_call(node, evaluated_args)?,
            )),
            "replaceChildren" => Ok(Some(
                self.eval_node_replace_children_call(node, evaluated_args)?,
            )),
            "append" => Ok(Some(self.eval_document_append_call(node, evaluated_args)?)),
            "prepend" => Ok(Some(self.eval_node_prepend_call(node, evaluated_args)?)),
            "after" => Ok(Some(self.eval_node_after_call(node, evaluated_args)?)),
            "before" => Ok(Some(self.eval_node_before_call(node, evaluated_args)?)),
            "insertAdjacentElement" => Ok(Some(
                self.eval_insert_adjacent_element_call(node, evaluated_args)?,
            )),
            "insertAdjacentHTML" => Ok(Some(
                self.eval_insert_adjacent_html_call(node, evaluated_args)?,
            )),
            "setHTMLUnsafe" => Ok(Some(self.eval_set_html_unsafe_call(node, evaluated_args)?)),
            "insertAdjacentText" => Ok(Some(
                self.eval_insert_adjacent_text_call(node, evaluated_args)?,
            )),
            "appendChild" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "appendChild requires exactly one node argument".into(),
                    ));
                }
                let child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "appendChild argument must be a Node".into(),
                        ));
                    }
                };
                self.dom.append_child(node, child)?;
                Ok(Some(Value::Node(child)))
            }
            "insertBefore" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "insertBefore requires exactly two arguments".into(),
                    ));
                }
                let child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "insertBefore first argument must be a Node".into(),
                        ));
                    }
                };
                match evaluated_args.get(1) {
                    Some(Value::Node(reference)) => {
                        self.dom.insert_before(node, child, *reference)?;
                    }
                    Some(Value::Null) | Some(Value::Undefined) => {
                        self.dom.append_child(node, child)?;
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "insertBefore second argument must be a Node or null".into(),
                        ));
                    }
                }
                Ok(Some(Value::Node(child)))
            }
            "removeChild" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "removeChild requires exactly one node argument".into(),
                    ));
                }
                let child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "removeChild argument must be a Node".into(),
                        ));
                    }
                };
                self.dom.remove_child(node, child)?;
                Ok(Some(Value::Node(child)))
            }
            "replaceChild" => {
                if evaluated_args.len() != 2 {
                    return Err(Error::ScriptRuntime(
                        "replaceChild requires exactly two node arguments".into(),
                    ));
                }
                let new_child = match evaluated_args.first() {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "replaceChild first argument must be a Node".into(),
                        ));
                    }
                };
                let old_child = match evaluated_args.get(1) {
                    Some(Value::Node(child)) => *child,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "replaceChild second argument must be a Node".into(),
                        ));
                    }
                };
                self.dom.replace_child(node, new_child, old_child)?;
                Ok(Some(Value::Node(old_child)))
            }
            "hasChildNodes" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "hasChildNodes takes no arguments".into(),
                    ));
                }
                Ok(Some(Value::Bool(
                    !self.dom.nodes[node.0].children.is_empty(),
                )))
            }
            "contains" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "contains requires exactly one argument".into(),
                    ));
                }
                let contains = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => false,
                    Some(Value::Node(other)) => {
                        *other == node || self.dom.is_descendant_of(*other, node)
                    }
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "contains argument must be a Node or null".into(),
                        ));
                    }
                };
                Ok(Some(Value::Bool(contains)))
            }
            "getRootNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "getRootNode supports at most one options argument".into(),
                    ));
                }
                Ok(Some(Value::Node(self.node_root(node))))
            }
            "compareDocumentPosition" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "compareDocumentPosition requires exactly one node argument".into(),
                    ));
                }
                let other = match evaluated_args.first() {
                    Some(Value::Node(other)) => *other,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "compareDocumentPosition argument must be a Node".into(),
                        ));
                    }
                };
                Ok(Some(Value::Number(
                    self.node_compare_document_position(node, other),
                )))
            }
            "isEqualNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "isEqualNode supports at most one argument".into(),
                    ));
                }
                let is_equal = match evaluated_args.first() {
                    None | Some(Value::Null) | Some(Value::Undefined) => false,
                    Some(Value::Node(other)) => self.nodes_are_equal(node, *other),
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "isEqualNode argument must be a Node or null".into(),
                        ));
                    }
                };
                Ok(Some(Value::Bool(is_equal)))
            }
            "isSameNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "isSameNode supports at most one argument".into(),
                    ));
                }
                let is_same = match evaluated_args.first() {
                    None | Some(Value::Null) | Some(Value::Undefined) => false,
                    Some(Value::Node(other)) => node == *other,
                    _ => {
                        return Err(Error::ScriptRuntime(
                            "isSameNode argument must be a Node or null".into(),
                        ));
                    }
                };
                Ok(Some(Value::Bool(is_same)))
            }
            "normalize" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("normalize takes no arguments".into()));
                }
                self.normalize_node_subtree(node)?;
                Ok(Some(Value::Undefined))
            }
            "isDefaultNamespace" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "isDefaultNamespace requires exactly one namespace argument".into(),
                    ));
                }
                let namespace = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => None,
                    Some(value) => Some(value.as_string()),
                    None => None,
                };
                Ok(Some(Value::Bool(
                    self.node_is_default_namespace(node, namespace.as_deref()),
                )))
            }
            "lookupPrefix" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "lookupPrefix requires exactly one namespace argument".into(),
                    ));
                }
                let namespace = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => None,
                    Some(value) => Some(value.as_string()),
                    None => None,
                };
                Ok(Some(
                    self.node_lookup_prefix(node, namespace.as_deref())
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                ))
            }
            "lookupNamespaceURI" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "lookupNamespaceURI requires exactly one prefix argument".into(),
                    ));
                }
                let prefix = match evaluated_args.first() {
                    Some(Value::Null) | Some(Value::Undefined) => None,
                    Some(value) => Some(value.as_string()),
                    None => None,
                };
                Ok(Some(
                    self.node_lookup_namespace_uri(node, prefix.as_deref())
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                ))
            }
            "cloneNode" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "cloneNode supports at most one argument".into(),
                    ));
                }
                let deep = evaluated_args.first().is_some_and(Value::truthy);
                let cloned = self.clone_dom_node(node, deep)?;
                Ok(Some(Value::Node(cloned)))
            }
            _ => Ok(None),
        }
    }
}
