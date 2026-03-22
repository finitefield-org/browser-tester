use std::collections::BTreeMap;

use crate::syntax::{AssignTarget, Expr, Program, Statement};
use crate::{
    CollectionEntryHandle, CollectionIteratorHandle, ElementHandle, HostBindings,
    HtmlCollectionNamedItem, HtmlCollectionScope, HtmlCollectionTarget, ListenerTarget,
    MimeTypeArrayState, NodeHandle, NodeListTarget, RadioNodeListTarget, Result, ScriptError,
    ScriptValue as Value, StorageTarget, StringListState, StyleSheetListTarget, StyleSheetTarget,
};

pub(crate) fn eval_program<H: HostBindings>(program: &Program, host: &mut H) -> Result<()> {
    eval_program_with_bindings(program, host, BTreeMap::new())
}

pub(crate) fn eval_program_with_bindings<H: HostBindings>(
    program: &Program,
    host: &mut H,
    mut env: BTreeMap<String, Value>,
) -> Result<()> {
    for statement in &program.statements {
        eval_statement(statement, &mut env, host)?;
    }

    Ok(())
}

fn as_string(value: &Value) -> String {
    match value {
        Value::Undefined => "undefined".to_string(),
        Value::Null => "null".to_string(),
        Value::Boolean(value) => value.to_string(),
        Value::Number(value) => {
            if value.fract() == 0.0 {
                (*value as i64).to_string()
            } else {
                value.to_string()
            }
        }
        Value::String(value) => value.clone(),
        Value::Element(_) => "[object Element]".to_string(),
        Value::ClassList(_) => "[object DOMTokenList]".to_string(),
        Value::Dataset(_) => "[object DOMStringMap]".to_string(),
        Value::TemplateContent(_) => "[object DocumentFragment]".to_string(),
        Value::HtmlCollection(_) => "[object HTMLCollection]".to_string(),
        Value::StyleSheetList(_) => "[object StyleSheetList]".to_string(),
        Value::StyleSheet(_) => "[object CSSStyleSheet]".to_string(),
        Value::Node(_) => "[object Node]".to_string(),
        Value::NodeList(_) => "[object NodeList]".to_string(),
        Value::RadioNodeList(_) => "[object RadioNodeList]".to_string(),
        Value::Storage(_) => "[object Storage]".to_string(),
        Value::MediaQueryList(_) => "[object MediaQueryList]".to_string(),
        Value::StringList(_) => "[object DOMStringList]".to_string(),
        Value::MimeTypeArray(_) => "[object MimeTypeArray]".to_string(),
        Value::Navigator => "[object Navigator]".to_string(),
        Value::History => "[object History]".to_string(),
        Value::Screen => "[object Screen]".to_string(),
        Value::ScreenOrientation(_) => "[object ScreenOrientation]".to_string(),
        Value::CollectionIterator(_) => "[object Iterator]".to_string(),
        Value::IteratorResult(_) => "[object IteratorResult]".to_string(),
        Value::CollectionEntry(_) => "[object IteratorEntry]".to_string(),
        Value::Document => "[object Document]".to_string(),
        Value::Window => "[object Window]".to_string(),
        Value::Event(_) => "[object Event]".to_string(),
        Value::Function(_) => "[function]".to_string(),
    }
}

fn content_editable_reflection(value: Option<&str>) -> &'static str {
    match value.map(|value| value.trim().to_ascii_lowercase()) {
        Some(value) if value.is_empty() || value == "true" => "true",
        Some(value) if value == "false" => "false",
        Some(value) if value == "plaintext-only" => "plaintext-only",
        _ => "inherit",
    }
}

fn value_for_listener_target(target: ListenerTarget) -> Value {
    match target {
        ListenerTarget::Window => Value::Window,
        ListenerTarget::Document => Value::Document,
        ListenerTarget::Element(element) => Value::Element(element),
    }
}

fn value_for_parent_node<H: HostBindings>(node: NodeHandle, host: &mut H) -> Result<Value> {
    let Some(parent) = host.node_parent(node)? else {
        return Ok(Value::Null);
    };

    Ok(match host.node_type(parent)? {
        9 => Value::Document,
        1 => Value::Element(ElementHandle::new(parent.raw())),
        _ => Value::Node(parent),
    })
}

fn value_for_parent_element<H: HostBindings>(node: NodeHandle, host: &mut H) -> Result<Value> {
    let Some(parent) = host.node_parent(node)? else {
        return Ok(Value::Null);
    };

    Ok(match host.node_type(parent)? {
        1 => Value::Element(ElementHandle::new(parent.raw())),
        _ => Value::Null,
    })
}

fn value_for_is_connected<H: HostBindings>(node: NodeHandle, host: &mut H) -> Result<Value> {
    let mut current = node;

    loop {
        let Some(parent) = host.node_parent(current)? else {
            return Ok(Value::Boolean(false));
        };

        if host.node_type(parent)? == 9 {
            return Ok(Value::Boolean(true));
        }

        current = parent;
    }
}

fn value_for_first_element_child(children: Vec<ElementHandle>) -> Value {
    children
        .into_iter()
        .next()
        .map(Value::Element)
        .unwrap_or(Value::Null)
}

fn value_for_last_element_child(children: Vec<ElementHandle>) -> Value {
    children
        .into_iter()
        .last()
        .map(Value::Element)
        .unwrap_or(Value::Null)
}

fn value_for_first_child<H: HostBindings>(
    scope: HtmlCollectionScope,
    host: &mut H,
) -> Result<Value> {
    Ok(
        match host.node_child_nodes_items(scope)?.into_iter().next() {
            Some(node) => value_for_node_handle(node, host)?,
            None => Value::Null,
        },
    )
}

fn value_for_last_child<H: HostBindings>(
    scope: HtmlCollectionScope,
    host: &mut H,
) -> Result<Value> {
    Ok(
        match host.node_child_nodes_items(scope)?.into_iter().last() {
            Some(node) => value_for_node_handle(node, host)?,
            None => Value::Null,
        },
    )
}

fn value_for_adjacent_sibling<H: HostBindings>(
    node: NodeHandle,
    next: bool,
    host: &mut H,
) -> Result<Value> {
    let Some(parent) = host.node_parent(node)? else {
        return Ok(Value::Null);
    };

    let scope = match host.node_type(parent)? {
        9 => HtmlCollectionScope::Document,
        1 => HtmlCollectionScope::Element(ElementHandle::new(parent.raw())),
        _ => HtmlCollectionScope::Node(parent),
    };
    let children = host.node_child_nodes_items(scope)?;
    let Some(index) = children
        .iter()
        .position(|candidate| candidate.raw() == node.raw())
    else {
        return Ok(Value::Null);
    };
    let sibling = if next {
        children.get(index + 1).copied()
    } else {
        index
            .checked_sub(1)
            .and_then(|index| children.get(index).copied())
    };

    Ok(match sibling {
        Some(node) => value_for_node_handle(node, host)?,
        None => Value::Null,
    })
}

fn value_for_adjacent_element_sibling<H: HostBindings>(
    node: NodeHandle,
    next: bool,
    host: &mut H,
) -> Result<Value> {
    let Some(parent) = host.node_parent(node)? else {
        return Ok(Value::Null);
    };

    let scope = match host.node_type(parent)? {
        9 => HtmlCollectionScope::Document,
        1 => HtmlCollectionScope::Element(ElementHandle::new(parent.raw())),
        _ => HtmlCollectionScope::Node(parent),
    };
    let children = host.node_child_nodes_items(scope)?;
    let Some(index) = children
        .iter()
        .position(|candidate| candidate.raw() == node.raw())
    else {
        return Ok(Value::Null);
    };

    if next {
        for candidate in children.iter().skip(index + 1) {
            if host.node_type(*candidate)? == 1 {
                return value_for_node_handle(*candidate, host);
            }
        }
    } else {
        for candidate in children.iter().take(index).rev() {
            if host.node_type(*candidate)? == 1 {
                return value_for_node_handle(*candidate, host);
            }
        }
    }

    Ok(Value::Null)
}

fn value_for_child_element_count(children: Vec<ElementHandle>) -> Value {
    Value::Number(children.len() as f64)
}

fn value_for_node_handle<H: HostBindings>(node: NodeHandle, host: &mut H) -> Result<Value> {
    Ok(match host.node_type(node)? {
        9 => Value::Document,
        1 => Value::Element(ElementHandle::new(node.raw())),
        _ => Value::Node(node),
    })
}

fn node_clone<H: HostBindings>(
    node: NodeHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 1 {
        return Err(ScriptError::new("cloneNode() expects at most one argument"));
    }

    let deep = match args.first() {
        Some(expr) => is_truthy(&eval_expr(expr, env, host)?),
        None => false,
    };
    let cloned = host.node_clone(node, deep)?;
    value_for_node_handle(cloned, host)
}

fn document_import_node<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(ScriptError::new(
            "document.importNode() expects one or two arguments",
        ));
    }

    let node = eval_expr(&args[0], env, host)?;
    let deep = match args.get(1) {
        Some(expr) => is_truthy(&eval_expr(expr, env, host)?),
        None => false,
    };

    match node {
        Value::Element(element) => {
            let cloned = host.node_clone(NodeHandle::new(element.raw()), deep)?;
            value_for_node_handle(cloned, host)
        }
        Value::Node(node) => {
            let cloned = host.node_clone(node, deep)?;
            value_for_node_handle(cloned, host)
        }
        Value::TemplateContent(element) => {
            let cloned = host.node_clone(NodeHandle::new(element.raw()), deep)?;
            if host.node_type(cloned)? != 1 {
                return Err(ScriptError::new(
                    "document.importNode() expected a cloned <template> element",
                ));
            }
            Ok(Value::TemplateContent(ElementHandle::new(cloned.raw())))
        }
        _ => Err(ScriptError::new(
            "document.importNode() expects a node or DocumentFragment argument",
        )),
    }
}

fn node_replace_with<H: HostBindings>(
    node: NodeHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_mutation_children(args, env, host, "replaceWith")?;
    host.node_replace_with(node, children)?;
    Ok(Value::Undefined)
}

fn node_remove<H: HostBindings>(
    node: NodeHandle,
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("remove() expects no arguments"));
    }

    host.node_replace_with(node, Vec::new())?;
    Ok(Value::Undefined)
}

fn node_before<H: HostBindings>(
    node: NodeHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_mutation_children(args, env, host, "before")?;
    host.node_before(node, children)?;
    Ok(Value::Undefined)
}

fn node_after<H: HostBindings>(
    node: NodeHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_mutation_children(args, env, host, "after")?;
    host.node_after(node, children)?;
    Ok(Value::Undefined)
}

fn node_normalize<H: HostBindings>(
    node: NodeHandle,
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("normalize() expects no arguments"));
    }

    host.node_normalize(node)?;
    Ok(Value::Undefined)
}

fn document_contains<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [node_expr] = args else {
        return Err(ScriptError::new("contains() expects exactly one argument"));
    };
    let node = eval_expr(node_expr, env, host)?;
    Ok(Value::Boolean(match node {
        Value::Null | Value::Undefined => false,
        Value::Document => true,
        Value::Element(element) => host.document_contains(NodeHandle::new(element.raw()))?,
        Value::Node(node) => host.document_contains(node)?,
        Value::TemplateContent(_) => false,
        _ => {
            return Err(ScriptError::new(
                "contains() expects a node or null reference",
            ));
        }
    }))
}

fn node_contains<H: HostBindings>(
    node: NodeHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [other_expr] = args else {
        return Err(ScriptError::new("contains() expects exactly one argument"));
    };
    let other = eval_expr(other_expr, env, host)?;
    Ok(Value::Boolean(match other {
        Value::Null | Value::Undefined => false,
        Value::Element(element) => host.node_contains(node, NodeHandle::new(element.raw()))?,
        Value::Node(other) => host.node_contains(node, other)?,
        Value::Document | Value::TemplateContent(_) => false,
        _ => {
            return Err(ScriptError::new(
                "contains() expects a node or null reference",
            ));
        }
    }))
}

fn compare_document_position<H: HostBindings>(
    node: NodeHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [other_expr] = args else {
        return Err(ScriptError::new(
            "compareDocumentPosition() expects exactly one argument",
        ));
    };
    let other = eval_expr(other_expr, env, host)?;
    let other = match other {
        Value::Document => NodeHandle::new(0),
        Value::Element(element) => NodeHandle::new(element.raw()),
        Value::Node(other) => other,
        _ => {
            return Err(ScriptError::new(
                "compareDocumentPosition() expects a node argument",
            ));
        }
    };
    Ok(Value::Number(
        host.node_compare_document_position(node, other)? as f64,
    ))
}

fn same_node<H: HostBindings>(
    object: Value,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [other_expr] = args else {
        return Err(ScriptError::new(
            "isSameNode() expects exactly one argument",
        ));
    };
    let other = eval_expr(other_expr, env, host)?;
    Ok(Value::Boolean(match object {
        Value::Document => match other {
            Value::Null | Value::Undefined => false,
            Value::Document => true,
            Value::Element(_) | Value::Node(_) | Value::TemplateContent(_) => false,
            _ => {
                return Err(ScriptError::new(
                    "isSameNode() expects a node or null reference",
                ));
            }
        },
        Value::Element(element) => match other {
            Value::Null | Value::Undefined => false,
            Value::Document => false,
            Value::Element(other) => element.raw() == other.raw(),
            Value::Node(other) => NodeHandle::new(element.raw()) == other,
            Value::TemplateContent(_) => false,
            _ => {
                return Err(ScriptError::new(
                    "isSameNode() expects a node or null reference",
                ));
            }
        },
        Value::Node(node) => match other {
            Value::Null | Value::Undefined => false,
            Value::Document => false,
            Value::Element(other) => node == NodeHandle::new(other.raw()),
            Value::Node(other) => node == other,
            Value::TemplateContent(_) => false,
            _ => {
                return Err(ScriptError::new(
                    "isSameNode() expects a node or null reference",
                ));
            }
        },
        Value::TemplateContent(fragment) => match other {
            Value::Null | Value::Undefined => false,
            Value::Document | Value::Element(_) | Value::Node(_) => false,
            Value::TemplateContent(other) => fragment.raw() == other.raw(),
            _ => {
                return Err(ScriptError::new(
                    "isSameNode() expects a node or null reference",
                ));
            }
        },
        _ => {
            return Err(ScriptError::new(
                "isSameNode() can only be called on node values",
            ));
        }
    }))
}

fn equal_node<H: HostBindings>(
    object: Value,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [other_expr] = args else {
        return Err(ScriptError::new(
            "isEqualNode() expects exactly one argument",
        ));
    };
    let other = eval_expr(other_expr, env, host)?;
    Ok(Value::Boolean(match object {
        Value::Document => match other {
            Value::Null | Value::Undefined => false,
            Value::Document => true,
            Value::Element(element) => {
                host.node_is_equal_node(NodeHandle::new(0), NodeHandle::new(element.raw()))?
            }
            Value::Node(node) => host.node_is_equal_node(NodeHandle::new(0), node)?,
            Value::TemplateContent(_) => false,
            _ => {
                return Err(ScriptError::new(
                    "isEqualNode() expects a node or null reference",
                ));
            }
        },
        Value::Element(element) => match other {
            Value::Null | Value::Undefined => false,
            Value::Document => {
                host.node_is_equal_node(NodeHandle::new(element.raw()), NodeHandle::new(0))?
            }
            Value::Element(other) => host
                .node_is_equal_node(NodeHandle::new(element.raw()), NodeHandle::new(other.raw()))?,
            Value::Node(other) => host.node_is_equal_node(NodeHandle::new(element.raw()), other)?,
            Value::TemplateContent(_) => false,
            _ => {
                return Err(ScriptError::new(
                    "isEqualNode() expects a node or null reference",
                ));
            }
        },
        Value::Node(node) => match other {
            Value::Null | Value::Undefined => false,
            Value::Document => host.node_is_equal_node(node, NodeHandle::new(0))?,
            Value::Element(other) => host.node_is_equal_node(node, NodeHandle::new(other.raw()))?,
            Value::Node(other) => host.node_is_equal_node(node, other)?,
            Value::TemplateContent(_) => false,
            _ => {
                return Err(ScriptError::new(
                    "isEqualNode() expects a node or null reference",
                ));
            }
        },
        Value::TemplateContent(fragment) => match other {
            Value::Null | Value::Undefined => false,
            Value::TemplateContent(other) => {
                host.template_content_is_equal_node(fragment, other)?
            }
            Value::Document | Value::Element(_) | Value::Node(_) => false,
            _ => {
                return Err(ScriptError::new(
                    "isEqualNode() expects a node or null reference",
                ));
            }
        },
        _ => {
            return Err(ScriptError::new(
                "isEqualNode() can only be called on node values",
            ));
        }
    }))
}

fn eval_statement<H: HostBindings>(
    statement: &Statement,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<()> {
    match statement {
        Statement::VariableDeclaration { name, value } => {
            let value = eval_expr(value, env, host)?;
            env.insert(name.clone(), value);
            Ok(())
        }
        Statement::Assignment { target, value } => {
            let value = eval_expr(value, env, host)?;
            eval_assignment(target, value, env, host)
        }
        Statement::Expression(expr) => {
            let _ = eval_expr(expr, env, host)?;
            Ok(())
        }
    }
}

fn eval_assignment<H: HostBindings>(
    target: &AssignTarget,
    value: Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<()> {
    match target {
        AssignTarget::Property { object, property } => {
            if let Some(result) =
                try_eval_location_url_assignment(object, property, &value, env, host)?
            {
                return result;
            }
            let object = eval_expr(object, env, host)?;
            match (object, property.as_str()) {
                (Value::Element(element), "textContent") => {
                    host.element_set_text_content(element, &as_string(&value))
                }
                (Value::Element(element), "innerHTML") => {
                    host.element_set_inner_html(element, &as_string(&value))
                }
                (Value::Element(element), "outerHTML") => {
                    host.element_set_outer_html(element, &as_string(&value))
                }
                (Value::Element(_element), "insertAdjacentHTML") => Err(ScriptError::new(
                    "insertAdjacentHTML() is a method, not an assignment target",
                )),
                (Value::TemplateContent(element), "innerHTML") => {
                    host.element_set_inner_html(element, &as_string(&value))
                }
                (Value::TemplateContent(element), "textContent") => {
                    host.element_set_text_content(element, &as_string(&value))
                }
                (Value::Element(element), "value") => {
                    host.element_set_value(element, &as_string(&value))
                }
                (Value::Element(element), "checked") => {
                    host.element_set_checked(element, is_truthy(&value))
                }
                (Value::Element(element), "className") => {
                    host.element_set_attribute(element, "class", &as_string(&value))
                }
                (Value::Element(element), "contentEditable") => {
                    let value = as_string(&value);
                    let normalized = value.trim().to_ascii_lowercase();
                    match normalized.as_str() {
                        "inherit" => {
                            host.element_remove_attribute(element, "contenteditable")?;
                            Ok(())
                        }
                        "true" => host.element_set_attribute(element, "contenteditable", "true"),
                        "false" => host.element_set_attribute(element, "contenteditable", "false"),
                        "plaintext-only" => {
                            host.element_set_attribute(element, "contenteditable", "plaintext-only")
                        }
                        _ => Err(ScriptError::new(format!(
                            "unsupported contentEditable value: {value}"
                        ))),
                    }
                }
                (Value::Dataset(element), property) => {
                    let attribute_name = dataset_attribute_name(property)?;
                    host.element_set_attribute(element, &attribute_name, &as_string(&value))
                }
                (Value::TemplateContent(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on template content value"
                ))),
                (Value::Element(_), _) => Err(ScriptError::new(format!(
                    "unsupported assignment target on element: {property}"
                ))),
                (Value::ClassList(_), property) => Err(ScriptError::new(format!(
                    "unsupported assignment target on class list value: {property}"
                ))),
                (Value::NodeList(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on node list value"
                ))),
                (Value::RadioNodeList(target), "value") => {
                    host.radio_node_list_set_value(&target, &as_string(&value))?;
                    Ok(())
                }
                (Value::RadioNodeList(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on radio node list value"
                ))),
                (Value::HtmlCollection(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on html collection value"
                ))),
                (Value::StyleSheetList(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on style sheet list value"
                ))),
                (Value::StyleSheet(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on style sheet value"
                ))),
                (Value::Node(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on node value"
                ))),
                (Value::CollectionEntry(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on iterator entry value"
                ))),
                (Value::Screen, property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on screen value"
                ))),
                (Value::ScreenOrientation(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on screen orientation value"
                ))),
                (Value::CollectionIterator(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on iterator value"
                ))),
                (Value::IteratorResult(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on iterator result value"
                ))),
                (Value::Document, "title") => {
                    host.document_set_title(&as_string(&value))?;
                    Ok(())
                }
                (Value::Document, "designMode") => {
                    host.document_set_design_mode(&as_string(&value))?;
                    Ok(())
                }
                (Value::Document, "dir") => {
                    host.document_set_dir(&as_string(&value))?;
                    Ok(())
                }
                (Value::Document, "location") => {
                    host.document_set_location(&as_string(&value))?;
                    Ok(())
                }
                (Value::Document, "cookie") => {
                    host.document_set_cookie(&as_string(&value))?;
                    Ok(())
                }
                (Value::Window, "title") => {
                    host.document_set_title(&as_string(&value))?;
                    Ok(())
                }
                (Value::Window, "name") => {
                    host.set_window_name(&as_string(&value))?;
                    Ok(())
                }
                (Value::Window, "location") => {
                    host.document_set_location(&as_string(&value))?;
                    Ok(())
                }
                (Value::Storage(target), property) if storage_property_is_reserved(property) => {
                    Err(ScriptError::new(format!(
                        "cannot assign to `{property}` on storage value"
                    )))
                }
                (Value::Storage(target), property) => {
                    host.storage_set_item(target.clone(), property, &as_string(&value))?;
                    Ok(())
                }
                (Value::MediaQueryList(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on media query list value"
                ))),
                (Value::StringList(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on string list value"
                ))),
                (Value::MimeTypeArray(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on mime type array value"
                ))),
                (Value::Navigator, property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on navigator value"
                ))),
                (Value::History, "scrollRestoration") => {
                    host.set_window_history_scroll_restoration(&as_string(&value))?;
                    Ok(())
                }
                (Value::History, property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on history value"
                ))),
                (Value::Document, property) | (Value::Window, property) => Err(ScriptError::new(
                    format!("unsupported assignment target: {property}"),
                )),
                (Value::String(_), property) => Err(ScriptError::new(format!(
                    "unsupported assignment target on string value: {property}"
                ))),
                (Value::Number(_), property) => Err(ScriptError::new(format!(
                    "unsupported assignment target on number value: {property}"
                ))),
                (Value::Boolean(_), property) => Err(ScriptError::new(format!(
                    "unsupported assignment target on boolean value: {property}"
                ))),
                (Value::Undefined, property) | (Value::Null, property) => Err(ScriptError::new(
                    format!("cannot assign to `{property}` on nullish value"),
                )),
                (Value::Function(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on function value"
                ))),
                (Value::Event(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on event value"
                ))),
            }
        }
    }
}

fn eval_expr<H: HostBindings>(
    expr: &Expr,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    match expr {
        Expr::Identifier(name) => eval_identifier(name, env),
        Expr::String(value) => Ok(Value::String(value.clone())),
        Expr::Number(value) => {
            let parsed = value
                .parse::<f64>()
                .map_err(|_| ScriptError::new(format!("invalid number literal: {value}")))?;
            Ok(Value::Number(parsed))
        }
        Expr::Boolean(value) => Ok(Value::Boolean(*value)),
        Expr::Null => Ok(Value::Null),
        Expr::Undefined => Ok(Value::Undefined),
        Expr::Member { object, property } => eval_member(object, property, env, host),
        Expr::Call { callee, args } => eval_call(callee, args, env, host),
        Expr::BinaryAdd { left, right } => {
            let left = eval_expr(left, env, host)?;
            let right = eval_expr(right, env, host)?;
            Ok(eval_add(left, right))
        }
        Expr::UnaryNeg(expr) => {
            let value = eval_expr(expr, env, host)?;
            match value {
                Value::Number(number) => Ok(Value::Number(-number)),
                _ => Err(ScriptError::new("unary - expects a number")),
            }
        }
        Expr::ArrowFunction(function) => Ok(Value::Function(function.clone())),
    }
}

fn eval_identifier(name: &str, env: &BTreeMap<String, Value>) -> Result<Value> {
    if let Some(value) = env.get(name) {
        return Ok(value.clone());
    }

    match name {
        "document" => Ok(Value::Document),
        "window" => Ok(Value::Window),
        "undefined" => Ok(Value::Undefined),
        "null" => Ok(Value::Null),
        "true" => Ok(Value::Boolean(true)),
        "false" => Ok(Value::Boolean(false)),
        other => Err(ScriptError::new(format!("unknown variable: {other}"))),
    }
}

fn eval_member<H: HostBindings>(
    object: &Expr,
    property: &str,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if let Some(value) = try_eval_location_url_access(object, property, env, host)? {
        return Ok(value);
    }
    let object = eval_expr(object, env, host)?;
    match object {
        Value::Document if property == "forms" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "form".to_string(),
            }))
        }
        Value::Document if property == "all" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "*".to_string(),
            }))
        }
        Value::Document if property == "images" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "img".to_string(),
            }))
        }
        Value::Document if property == "scripts" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "script".to_string(),
            }))
        }
        Value::Document if property == "styleSheets" => {
            Ok(Value::StyleSheetList(StyleSheetListTarget::Document))
        }
        Value::Document if property == "documentElement" => {
            Ok(match host.document_document_element()? {
                Some(element) => Value::Element(element),
                None => Value::Null,
            })
        }
        Value::Document if property == "isConnected" => Ok(Value::Boolean(true)),
        Value::Document if property == "ownerDocument" => Ok(Value::Null),
        Value::Document if property == "parentNode" => Ok(Value::Null),
        Value::Document if property == "title" => Ok(Value::String(host.document_title()?)),
        Value::Document if property == "location" => Ok(Value::String(host.document_location()?)),
        Value::Document if property == "URL" => Ok(Value::String(host.document_url()?)),
        Value::Document if property == "documentURI" => {
            Ok(Value::String(host.document_document_uri()?))
        }
        Value::Document if property == "baseURI" => Ok(Value::String(host.document_base_uri()?)),
        Value::Document if property == "origin" => Ok(Value::String(host.document_origin()?)),
        Value::Document if property == "referrer" => Ok(Value::String(host.document_referrer()?)),
        Value::Document if property == "cookie" => Ok(Value::String(host.document_cookie()?)),
        Value::Document if property == "currentScript" => {
            Ok(match host.document_current_script()? {
                Some(element) => Value::Element(element),
                None => Value::Null,
            })
        }
        Value::Document if property == "readyState" => {
            Ok(Value::String(host.document_ready_state()?))
        }
        Value::Document if property == "compatMode" => {
            Ok(Value::String(host.document_compat_mode()?))
        }
        Value::Document if property == "characterSet" || property == "charset" => {
            Ok(Value::String(host.document_character_set()?))
        }
        Value::Document if property == "contentType" => {
            Ok(Value::String(host.document_content_type()?))
        }
        Value::Document if property == "designMode" => {
            Ok(Value::String(host.document_design_mode()?))
        }
        Value::Document if property == "dir" => Ok(Value::String(host.document_dir()?)),
        Value::Document if property == "head" => Ok(match host.document_head()? {
            Some(element) => Value::Element(element),
            None => Value::Null,
        }),
        Value::Document if property == "body" => Ok(match host.document_body()? {
            Some(element) => Value::Element(element),
            None => Value::Null,
        }),
        Value::Document if property == "scrollingElement" => {
            Ok(match host.document_scrolling_element()? {
                Some(element) => Value::Element(element),
                None => Value::Null,
            })
        }
        Value::Document if property == "firstChild" => {
            value_for_first_child(HtmlCollectionScope::Document, host)
        }
        Value::Document if property == "lastChild" => {
            value_for_last_child(HtmlCollectionScope::Document, host)
        }
        Value::Document if property == "nextSibling" || property == "previousSibling" => {
            Ok(Value::Null)
        }
        Value::Document
            if property == "nextElementSibling" || property == "previousElementSibling" =>
        {
            Ok(Value::Null)
        }
        Value::Document if property == "activeElement" => {
            Ok(match host.document_active_element()? {
                Some(element) => Value::Element(element),
                None => Value::Null,
            })
        }
        Value::Document if property == "childNodes" => Ok(Value::NodeList(
            NodeListTarget::ChildNodes(HtmlCollectionScope::Document),
        )),
        Value::Document if property == "firstElementChild" => Ok(value_for_first_element_child(
            host.html_collection_document_children_items()?,
        )),
        Value::Document if property == "lastElementChild" => Ok(value_for_last_element_child(
            host.html_collection_document_children_items()?,
        )),
        Value::Document if property == "childElementCount" => Ok(value_for_child_element_count(
            host.html_collection_document_children_items()?,
        )),
        Value::Document if property == "links" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::DocumentLinks))
        }
        Value::Document if property == "anchors" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::DocumentAnchors))
        }
        Value::Document if property == "applets" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "applet".to_string(),
            }))
        }
        Value::Document if property == "children" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::DocumentChildren,
        )),
        Value::Document if property == "embeds" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "embed".to_string(),
            }))
        }
        Value::Document if property == "plugins" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "embed".to_string(),
            }))
        }
        Value::Window if property == "document" => Ok(Value::Document),
        Value::Window if property == "self" => Ok(Value::Window),
        Value::Window if property == "window" => Ok(Value::Window),
        Value::Window if property == "parent" => Ok(Value::Window),
        Value::Window if property == "top" => Ok(Value::Window),
        Value::Window if property == "closed" => Ok(Value::Boolean(false)),
        Value::Window if property == "frameElement" => Ok(Value::Null),
        Value::Window if property == "opener" => Ok(Value::Null),
        Value::Window if property == "frames" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::WindowFrames))
        }
        Value::Window if property == "length" => Ok(Value::Number(
            host.html_collection_window_frames_items()?.len() as f64,
        )),
        Value::Document if property == "defaultView" => Ok(Value::Window),
        Value::Document if property == "visibilityState" => {
            Ok(Value::String(host.document_visibility_state()?))
        }
        Value::Document if property == "hidden" => Ok(Value::Boolean(host.document_hidden()?)),
        Value::Window if property == "children" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::DocumentChildren,
        )),
        Value::Window if property == "name" => Ok(Value::String(host.window_name()?)),
        Value::Window if property == "title" => Ok(Value::String(host.document_title()?)),
        Value::Window if property == "location" => Ok(Value::String(host.document_location()?)),
        Value::Window if property == "origin" => Ok(Value::String(host.document_origin()?)),
        Value::Document if property == "domain" => Ok(Value::String(host.document_domain()?)),
        Value::Window if property == "localStorage" => Ok(Value::Storage(StorageTarget::Local)),
        Value::Window if property == "sessionStorage" => Ok(Value::Storage(StorageTarget::Session)),
        Value::Window if property == "navigator" => Ok(Value::Navigator),
        Value::Window if property == "history" => Ok(Value::History),
        Value::Window if property == "scrollX" => Ok(Value::Number(host.window_scroll_x()? as f64)),
        Value::Window if property == "scrollY" => Ok(Value::Number(host.window_scroll_y()? as f64)),
        Value::Window if property == "pageXOffset" => {
            Ok(Value::Number(host.window_page_x_offset()? as f64))
        }
        Value::Window if property == "pageYOffset" => {
            Ok(Value::Number(host.window_page_y_offset()? as f64))
        }
        Value::Window if property == "devicePixelRatio" => {
            Ok(Value::Number(host.window_device_pixel_ratio()?))
        }
        Value::Window if property == "innerWidth" => {
            Ok(Value::Number(host.window_inner_width()? as f64))
        }
        Value::Window if property == "innerHeight" => {
            Ok(Value::Number(host.window_inner_height()? as f64))
        }
        Value::Window if property == "outerWidth" => {
            Ok(Value::Number(host.window_outer_width()? as f64))
        }
        Value::Window if property == "outerHeight" => {
            Ok(Value::Number(host.window_outer_height()? as f64))
        }
        Value::Window if property == "screenX" => Ok(Value::Number(host.window_screen_x()? as f64)),
        Value::Window if property == "screenY" => Ok(Value::Number(host.window_screen_y()? as f64)),
        Value::Window if property == "screenLeft" => {
            Ok(Value::Number(host.window_screen_left()? as f64))
        }
        Value::Window if property == "screenTop" => {
            Ok(Value::Number(host.window_screen_top()? as f64))
        }
        Value::Window if property == "screen" => Ok(Value::Screen),
        Value::Screen if property == "width" => {
            Ok(Value::Number(host.window_screen_width()? as f64))
        }
        Value::Screen if property == "height" => {
            Ok(Value::Number(host.window_screen_height()? as f64))
        }
        Value::Screen if property == "availWidth" => {
            Ok(Value::Number(host.window_screen_avail_width()? as f64))
        }
        Value::Screen if property == "availHeight" => {
            Ok(Value::Number(host.window_screen_avail_height()? as f64))
        }
        Value::Screen if property == "availLeft" => {
            Ok(Value::Number(host.window_screen_avail_left()? as f64))
        }
        Value::Screen if property == "availTop" => {
            Ok(Value::Number(host.window_screen_avail_top()? as f64))
        }
        Value::Screen if property == "colorDepth" => {
            Ok(Value::Number(host.window_screen_color_depth()? as f64))
        }
        Value::Screen if property == "pixelDepth" => {
            Ok(Value::Number(host.window_screen_pixel_depth()? as f64))
        }
        Value::Screen if property == "orientation" => {
            Ok(Value::ScreenOrientation(host.window_screen_orientation()?))
        }
        Value::ScreenOrientation(orientation) if property == "type" => {
            Ok(Value::String(orientation.orientation_type().to_string()))
        }
        Value::ScreenOrientation(orientation) if property == "angle" => {
            Ok(Value::Number(orientation.angle() as f64))
        }
        Value::MediaQueryList(list) if property == "matches" => Ok(Value::Boolean(list.matches())),
        Value::MediaQueryList(list) if property == "media" => {
            Ok(Value::String(list.media().to_string()))
        }
        Value::Navigator if property == "userAgent" => {
            Ok(Value::String(host.window_navigator_user_agent()?))
        }
        Value::Navigator if property == "appCodeName" => {
            Ok(Value::String(host.window_navigator_app_code_name()?))
        }
        Value::Navigator if property == "appName" => {
            Ok(Value::String(host.window_navigator_app_name()?))
        }
        Value::Navigator if property == "appVersion" => {
            Ok(Value::String(host.window_navigator_app_version()?))
        }
        Value::Navigator if property == "product" => {
            Ok(Value::String(host.window_navigator_product()?))
        }
        Value::Navigator if property == "productSub" => {
            Ok(Value::String(host.window_navigator_product_sub()?))
        }
        Value::Navigator if property == "platform" => {
            Ok(Value::String(host.window_navigator_platform()?))
        }
        Value::Navigator if property == "language" => {
            Ok(Value::String(host.window_navigator_language()?))
        }
        Value::Navigator if property == "oscpu" => {
            Ok(Value::String(host.window_navigator_oscpu()?))
        }
        Value::Navigator if property == "userLanguage" => {
            Ok(Value::String(host.window_navigator_user_language()?))
        }
        Value::Navigator if property == "browserLanguage" => {
            Ok(Value::String(host.window_navigator_browser_language()?))
        }
        Value::Navigator if property == "systemLanguage" => {
            Ok(Value::String(host.window_navigator_system_language()?))
        }
        Value::Navigator if property == "languages" => Ok(Value::StringList(StringListState::new(
            host.window_navigator_languages()?,
        ))),
        Value::Navigator if property == "mimeTypes" => Ok(Value::MimeTypeArray(
            MimeTypeArrayState::new(host.window_navigator_mime_types()?),
        )),
        Value::Navigator if property == "cookieEnabled" => {
            Ok(Value::Boolean(host.window_navigator_cookie_enabled()?))
        }
        Value::Navigator if property == "onLine" => {
            Ok(Value::Boolean(host.window_navigator_on_line()?))
        }
        Value::Navigator if property == "webdriver" => {
            Ok(Value::Boolean(host.window_navigator_webdriver()?))
        }
        Value::Navigator if property == "vendor" => {
            Ok(Value::String(host.window_navigator_vendor()?))
        }
        Value::Navigator if property == "vendorSub" => {
            Ok(Value::String(host.window_navigator_vendor_sub()?))
        }
        Value::Navigator if property == "pdfViewerEnabled" => {
            Ok(Value::Boolean(host.window_navigator_pdf_viewer_enabled()?))
        }
        Value::Navigator if property == "doNotTrack" => {
            Ok(Value::String(host.window_navigator_do_not_track()?))
        }
        Value::Navigator if property == "javaEnabled" => {
            Ok(Value::Boolean(host.window_navigator_java_enabled()?))
        }
        Value::Navigator if property == "plugins" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "embed".to_string(),
            }))
        }
        Value::Navigator if property == "hardwareConcurrency" => Ok(Value::Number(
            host.window_navigator_hardware_concurrency()? as f64,
        )),
        Value::Navigator if property == "maxTouchPoints" => Ok(Value::Number(
            host.window_navigator_max_touch_points()? as f64,
        )),
        Value::History if property == "length" => {
            Ok(Value::Number(host.window_history_length()? as f64))
        }
        Value::History if property == "state" => match host.window_history_state()? {
            Some(value) => Ok(Value::String(value)),
            None => Ok(Value::Null),
        },
        Value::History if property == "scrollRestoration" => {
            Ok(Value::String(host.window_history_scroll_restoration()?))
        }
        Value::Element(element) if property == "textContent" => {
            Ok(Value::String(host.element_text_content(element)?))
        }
        Value::Element(element) if property == "innerHTML" => {
            Ok(Value::String(host.element_inner_html(element)?))
        }
        Value::Element(element) if property == "outerHTML" => {
            Ok(Value::String(host.element_outer_html(element)?))
        }
        Value::Element(_) if property == "ownerDocument" => Ok(Value::Document),
        Value::Element(element) if property == "baseURI" => {
            Ok(Value::String(host.element_base_uri(element)?))
        }
        Value::Element(element) if property == "origin" => {
            Ok(Value::String(host.element_origin(element)?))
        }
        Value::Element(element) if property == "value" => {
            Ok(Value::String(host.element_value(element)?))
        }
        Value::Element(element) if property == "length" => match host.element_tag_name(element)? {
            tag if tag == "form" => Ok(Value::Number(
                host.html_collection_form_elements_items(element)?.len() as f64,
            )),
            tag if tag == "select" => Ok(Value::Number(
                host.html_collection_select_options_items(element)?.len() as f64,
            )),
            _ => Err(unsupported_member_access(property, "element")),
        },
        Value::Element(element) if property == "checked" => {
            Ok(Value::Boolean(host.element_checked(element)?))
        }
        Value::Element(element) if property == "className" => Ok(Value::String(
            host.element_get_attribute(element, "class")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "contentEditable" => {
            let value = host.element_get_attribute(element, "contenteditable")?;
            Ok(Value::String(
                content_editable_reflection(value.as_deref()).to_string(),
            ))
        }
        Value::Element(element) if property == "isConnected" => {
            value_for_is_connected(NodeHandle::new(element.raw()), host)
        }
        Value::Element(element) if property == "isContentEditable" => {
            let value = host.element_is_content_editable(element)?;
            Ok(Value::Boolean(value))
        }
        Value::Element(element) if property == "content" => {
            if host.element_tag_name(element)? == "template" {
                Ok(Value::TemplateContent(element))
            } else {
                Err(ScriptError::new(
                    "template.content is only supported on <template> elements",
                ))
            }
        }
        Value::Element(element) if property == "classList" => Ok(Value::ClassList(element)),
        Value::Element(element) if property == "dataset" => Ok(Value::Dataset(element)),
        Value::Element(element) if property == "children" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::Children(element),
        )),
        Value::Element(element) if property == "firstElementChild" => Ok(
            value_for_first_element_child(host.element_children(element)?),
        ),
        Value::Element(element) if property == "lastElementChild" => Ok(
            value_for_last_element_child(host.element_children(element)?),
        ),
        Value::Element(element) if property == "childElementCount" => Ok(
            value_for_child_element_count(host.element_children(element)?),
        ),
        Value::Element(element) if property == "childNodes" => Ok(Value::NodeList(
            NodeListTarget::ChildNodes(HtmlCollectionScope::Element(element)),
        )),
        Value::Element(element) if property == "firstChild" => {
            value_for_first_child(HtmlCollectionScope::Element(element), host)
        }
        Value::Element(element) if property == "lastChild" => {
            value_for_last_child(HtmlCollectionScope::Element(element), host)
        }
        Value::Element(element) if property == "nextSibling" => {
            value_for_adjacent_sibling(NodeHandle::new(element.raw()), true, host)
        }
        Value::Element(element) if property == "previousSibling" => {
            value_for_adjacent_sibling(NodeHandle::new(element.raw()), false, host)
        }
        Value::Element(element) if property == "nextElementSibling" => {
            value_for_adjacent_element_sibling(NodeHandle::new(element.raw()), true, host)
        }
        Value::Element(element) if property == "previousElementSibling" => {
            value_for_adjacent_element_sibling(NodeHandle::new(element.raw()), false, host)
        }
        Value::Element(element) if property == "labels" => {
            Ok(Value::NodeList(NodeListTarget::Labels(element)))
        }
        Value::Element(element) if property == "parentNode" => {
            value_for_parent_node(NodeHandle::new(element.raw()), host)
        }
        Value::Element(element) if property == "parentElement" => {
            value_for_parent_element(NodeHandle::new(element.raw()), host)
        }
        Value::Element(element) if property == "rows" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::TableRows(element),
        )),
        Value::Element(element) if property == "cells" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::RowCells(element),
        )),
        Value::Element(element) if property == "elements" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::FormElements(element),
        )),
        Value::Element(element) if property == "options" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::SelectOptions(element),
        )),
        Value::Element(element) if property == "selectedOptions" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::SelectSelectedOptions(element),
        )),
        Value::Element(element) if property == "areas" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::MapAreas(element),
        )),
        Value::Element(element) if property == "tBodies" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::TableTBodies(element),
        )),
        Value::Event(event) if property == "type" => Ok(Value::String(event.event_type())),
        Value::Event(event) if property == "target" => {
            Ok(value_for_listener_target(event.target()))
        }
        Value::Event(event) if property == "currentTarget" => Ok(event
            .current_target()
            .map(value_for_listener_target)
            .unwrap_or(Value::Undefined)),
        Value::Event(event) if property == "defaultPrevented" => {
            Ok(Value::Boolean(event.default_prevented()))
        }
        Value::Event(event) if property == "cancelable" => Ok(Value::Boolean(event.cancelable())),
        Value::Event(event) if property == "bubbles" => Ok(Value::Boolean(event.bubbles())),
        Value::Event(event) if property == "eventPhase" => {
            Ok(Value::Number(event.event_phase() as u8 as f64))
        }
        Value::Node(node) if property == "childNodes" => Ok(Value::NodeList(
            NodeListTarget::ChildNodes(HtmlCollectionScope::Node(node)),
        )),
        Value::Node(node) if property == "firstChild" => {
            value_for_first_child(HtmlCollectionScope::Node(node), host)
        }
        Value::Node(node) if property == "lastChild" => {
            value_for_last_child(HtmlCollectionScope::Node(node), host)
        }
        Value::Node(node) if property == "nextSibling" => {
            value_for_adjacent_sibling(node, true, host)
        }
        Value::Node(node) if property == "previousSibling" => {
            value_for_adjacent_sibling(node, false, host)
        }
        Value::Node(node) if property == "nextElementSibling" => {
            value_for_adjacent_element_sibling(node, true, host)
        }
        Value::Node(node) if property == "previousElementSibling" => {
            value_for_adjacent_element_sibling(node, false, host)
        }
        Value::Node(node) if property == "textContent" => {
            Ok(Value::String(host.node_text_content(node)?))
        }
        Value::Node(node) if property == "isConnected" => value_for_is_connected(node, host),
        Value::Node(node) if property == "parentNode" => value_for_parent_node(node, host),
        Value::Node(node) if property == "parentElement" => value_for_parent_element(node, host),
        Value::Node(_) if property == "ownerDocument" => Ok(Value::Document),
        Value::Node(node) if property == "nodeType" => {
            Ok(Value::Number(host.node_type(node)? as f64))
        }
        Value::Node(node) if property == "nodeName" => Ok(Value::String(host.node_name(node)?)),
        Value::HtmlCollection(collection) if property == "length" => {
            let length = html_collection_items(&collection, host)?.len();
            Ok(Value::Number(length as f64))
        }
        Value::HtmlCollection(_collection) if html_collection_property_is_reserved(property) => {
            Err(unsupported_member_access(property, "html collection"))
        }
        Value::HtmlCollection(collection) => Ok(
            match html_collection_named_item_handle(&collection, property, host)? {
                Some(HtmlCollectionNamedItem::Element(handle)) => Value::Element(handle),
                Some(HtmlCollectionNamedItem::RadioNodeList(target)) => {
                    Value::RadioNodeList(target)
                }
                None => Value::Undefined,
            },
        ),
        Value::StyleSheetList(target) if property == "length" => {
            let length = style_sheet_list_items(&target, host)?.len();
            Ok(Value::Number(length as f64))
        }
        Value::IteratorResult(result) if property == "value" => {
            Ok(result.value().unwrap_or(Value::Undefined))
        }
        Value::IteratorResult(result) if property == "done" => Ok(Value::Boolean(result.done())),
        Value::CollectionEntry(entry) if property == "index" => {
            Ok(Value::Number(entry.index() as f64))
        }
        Value::CollectionEntry(entry) if property == "value" => Ok(entry.value()),
        Value::ClassList(element) if property == "length" => {
            let length = class_list_tokens(element, host)?.len();
            Ok(Value::Number(length as f64))
        }
        Value::NodeList(target) if property == "length" => {
            let length = node_list_items(&target, host)?.len();
            Ok(Value::Number(length as f64))
        }
        Value::RadioNodeList(target) if property == "length" => {
            let length = radio_node_list_items(&target, host)?.len();
            Ok(Value::Number(length as f64))
        }
        Value::RadioNodeList(target) if property == "value" => {
            Ok(Value::String(radio_node_list_value(&target, host)?))
        }
        Value::StringList(list) if property == "length" => Ok(Value::Number(list.length() as f64)),
        Value::MimeTypeArray(list) if property == "length" => {
            Ok(Value::Number(list.length() as f64))
        }
        Value::Navigator => Err(unsupported_member_access(property, "navigator")),
        Value::History => Err(unsupported_member_access(property, "history")),
        Value::Screen => Err(unsupported_member_access(property, "screen")),
        Value::Element(_) => Err(unsupported_member_access(property, "element")),
        Value::ClassList(_) => Err(unsupported_member_access(property, "class list")),
        Value::Dataset(element) => {
            let attribute_name = dataset_attribute_name(property)?;
            Ok(
                match host.element_get_attribute(element, &attribute_name)? {
                    Some(value) => Value::String(value),
                    None => Value::Undefined,
                },
            )
        }
        Value::TemplateContent(element) if property == "childNodes" => Ok(Value::NodeList(
            NodeListTarget::ChildNodes(HtmlCollectionScope::Element(element)),
        )),
        Value::TemplateContent(element) if property == "children" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::Children(element),
        )),
        Value::TemplateContent(_) if property == "isConnected" => Ok(Value::Boolean(false)),
        Value::TemplateContent(_) if property == "nodeType" => Ok(Value::Number(11.0)),
        Value::TemplateContent(_) if property == "nodeName" => {
            Ok(Value::String("#document-fragment".to_string()))
        }
        Value::TemplateContent(_) if property == "parentNode" => Ok(Value::Null),
        Value::TemplateContent(_) if property == "nextSibling" => Ok(Value::Null),
        Value::TemplateContent(_) if property == "previousSibling" => Ok(Value::Null),
        Value::TemplateContent(_) if property == "nextElementSibling" => Ok(Value::Null),
        Value::TemplateContent(_) if property == "previousElementSibling" => Ok(Value::Null),
        Value::TemplateContent(element) if property == "firstChild" => {
            value_for_first_child(HtmlCollectionScope::Element(element), host)
        }
        Value::TemplateContent(element) if property == "lastChild" => {
            value_for_last_child(HtmlCollectionScope::Element(element), host)
        }
        Value::TemplateContent(element) if property == "firstElementChild" => Ok(
            value_for_first_element_child(host.element_children(element)?),
        ),
        Value::TemplateContent(element) if property == "lastElementChild" => Ok(
            value_for_last_element_child(host.element_children(element)?),
        ),
        Value::TemplateContent(element) if property == "childElementCount" => Ok(
            value_for_child_element_count(host.element_children(element)?),
        ),
        Value::TemplateContent(element) if property == "textContent" => {
            Ok(Value::String(host.element_text_content(element)?))
        }
        Value::TemplateContent(element) if property == "innerHTML" => {
            Ok(Value::String(host.element_inner_html(element)?))
        }
        Value::TemplateContent(_) if property == "ownerDocument" => Ok(Value::Document),
        Value::Document => Err(unsupported_member_access(property, "document")),
        Value::Window => Err(unsupported_member_access(property, "window")),
        Value::String(_) => Err(unsupported_member_access(property, "string")),
        Value::Number(_) => Err(unsupported_member_access(property, "number")),
        Value::Boolean(_) => Err(unsupported_member_access(property, "boolean")),
        Value::Null | Value::Undefined => Err(unsupported_member_access(property, "nullish")),
        Value::Event(_) => Err(unsupported_member_access(property, "event")),
        Value::StyleSheetList(_) => Err(unsupported_member_access(property, "style sheet list")),
        Value::StyleSheet(_) => Err(unsupported_member_access(property, "style sheet")),
        Value::Node(_) => Err(unsupported_member_access(property, "node")),
        Value::NodeList(_) => Err(unsupported_member_access(property, "node list")),
        Value::RadioNodeList(_) => Err(unsupported_member_access(property, "radio node list")),
        Value::MediaQueryList(_) => Err(unsupported_member_access(property, "media query list")),
        Value::StringList(_) => Err(unsupported_member_access(property, "string list")),
        Value::MimeTypeArray(_) => Err(unsupported_member_access(property, "mime type array")),
        Value::ScreenOrientation(_) => {
            Err(unsupported_member_access(property, "screen orientation"))
        }
        Value::Storage(target) if property == "length" => {
            Ok(Value::Number(host.storage_length(target)? as f64))
        }
        Value::Storage(target) if storage_property_is_reserved(property) => {
            Err(unsupported_member_access(property, "storage"))
        }
        Value::Storage(target) => Ok(match host.storage_get_item(target.clone(), property)? {
            Some(value) => Value::String(value),
            None => Value::Undefined,
        }),
        Value::CollectionIterator(_) => Err(unsupported_member_access(property, "iterator")),
        Value::IteratorResult(_) => Err(unsupported_member_access(property, "iterator result")),
        Value::CollectionEntry(_) => Err(unsupported_member_access(property, "iterator entry")),
        Value::Function(_) => Err(unsupported_member_access(property, "function")),
        Value::TemplateContent(_) => Err(unsupported_member_access(property, "template content")),
    }
}

fn eval_call<H: HostBindings>(
    callee: &Expr,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    match callee {
        Expr::Identifier(name) if name == "String" => {
            let value = match args.len() {
                0 => Value::Undefined,
                1 => eval_expr(&args[0], env, host)?,
                _ => return Err(ScriptError::new("String() accepts at most one argument")),
            };
            Ok(Value::String(as_string(&value)))
        }
        Expr::Identifier(name) if name == "Boolean" => {
            let value = match args.len() {
                0 => Value::Undefined,
                1 => eval_expr(&args[0], env, host)?,
                _ => return Err(ScriptError::new("Boolean() accepts at most one argument")),
            };
            Ok(Value::Boolean(is_truthy(&value)))
        }
        Expr::Identifier(_) => Err(ScriptError::new("invalid call target")),
        Expr::Member { object, property } => {
            if let Some(value) = try_eval_location_method_call(object, property, args, env, host)? {
                return Ok(value);
            }
            let object_value = eval_expr(object, env, host)?;
            eval_method_call(object_value, property, args, env, host)
        }
        Expr::ArrowFunction(_) => Err(ScriptError::new("arrow functions are not callable")),
        Expr::String(_)
        | Expr::Number(_)
        | Expr::Boolean(_)
        | Expr::Null
        | Expr::Undefined
        | Expr::UnaryNeg(_) => Err(ScriptError::new("invalid call target")),
        Expr::Call { .. } | Expr::BinaryAdd { .. } => {
            Err(ScriptError::new("invalid nested call target"))
        }
    }
}

fn try_eval_location_method_call<H: HostBindings>(
    object: &Expr,
    method: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Option<Value>> {
    let Expr::Member {
        object: location_owner,
        property: location_property,
    } = object
    else {
        return Ok(None);
    };

    if location_property != "location" {
        return Ok(None);
    }

    let location_owner = eval_expr(location_owner, env, host)?;
    if !matches!(location_owner, Value::Document | Value::Window) {
        return Ok(None);
    }

    match method {
        "toString" => {
            if !args.is_empty() {
                return Err(ScriptError::new("location.toString() expects no arguments"));
            }
            Ok(Some(Value::String(host.document_location()?)))
        }
        "valueOf" => {
            if !args.is_empty() {
                return Err(ScriptError::new("location.valueOf() expects no arguments"));
            }
            Ok(Some(Value::String(host.document_location()?)))
        }
        "assign" => {
            if args.len() != 1 {
                return Err(ScriptError::new(
                    "location.assign() expects exactly one argument",
                ));
            }
            let url = as_string(&eval_expr(&args[0], env, host)?);
            host.document_location_assign(&url)?;
            Ok(Some(Value::Undefined))
        }
        "replace" => {
            if args.len() != 1 {
                return Err(ScriptError::new(
                    "location.replace() expects exactly one argument",
                ));
            }
            let url = as_string(&eval_expr(&args[0], env, host)?);
            host.document_location_replace(&url)?;
            Ok(Some(Value::Undefined))
        }
        "reload" => {
            if args.len() > 1 {
                return Err(ScriptError::new(
                    "location.reload() expects at most one argument",
                ));
            }
            if let Some(expr) = args.first() {
                let _ = eval_expr(expr, env, host)?;
            }
            host.document_location_reload()?;
            Ok(Some(Value::Undefined))
        }
        _ => Ok(None),
    }
}

fn try_eval_location_url_access<H: HostBindings>(
    object: &Expr,
    property: &str,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Option<Value>> {
    if property != "href"
        && property != "hash"
        && property != "pathname"
        && property != "search"
        && property != "origin"
        && property != "protocol"
        && property != "host"
        && property != "hostname"
        && property != "port"
        && property != "username"
        && property != "password"
    {
        return Ok(None);
    }

    let Expr::Member {
        object: location_owner,
        property: location_property,
    } = object
    else {
        return Ok(None);
    };

    if location_property != "location" {
        return Ok(None);
    }

    let location_owner = eval_expr(location_owner, env, host)?;
    if !matches!(location_owner, Value::Document | Value::Window) {
        return Ok(None);
    }

    let location = host.document_location()?;
    Ok(Some(Value::String(match property {
        "href" => location,
        "protocol" => location_protocol(&location),
        "host" => location_host(&location),
        "hostname" => location_hostname(&location),
        "port" => location_port(&location),
        "username" => location_username(&location),
        "password" => location_password(&location),
        "hash" => location
            .split_once('#')
            .map(|(_, fragment)| format!("#{fragment}"))
            .unwrap_or_default(),
        "pathname" => location_pathname(&location),
        "search" => location_search(&location),
        "origin" => host.document_origin()?,
        _ => unreachable!(),
    })))
}

fn try_eval_location_url_assignment<H: HostBindings>(
    object: &Expr,
    property: &str,
    value: &Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Option<Result<()>>> {
    if property != "href"
        && property != "hash"
        && property != "pathname"
        && property != "search"
        && property != "protocol"
        && property != "host"
        && property != "hostname"
        && property != "port"
        && property != "username"
        && property != "password"
    {
        return Ok(None);
    }

    let Expr::Member {
        object: location_owner,
        property: location_property,
    } = object
    else {
        return Ok(None);
    };

    if location_property != "location" {
        return Ok(None);
    }

    let location_owner = eval_expr(location_owner, env, host)?;
    if !matches!(location_owner, Value::Document | Value::Window) {
        return Ok(None);
    }

    let current_url = host.document_location()?;
    let next_url = match property {
        "href" => as_string(value),
        "hash" => location_with_hash(&current_url, &as_string(value)),
        "pathname" => location_with_pathname(&current_url, &as_string(value)),
        "search" => location_with_search(&current_url, &as_string(value)),
        "protocol" => location_with_protocol(&current_url, &as_string(value))?,
        "host" => location_with_host(&current_url, &as_string(value))?,
        "hostname" => location_with_hostname(&current_url, &as_string(value))?,
        "port" => location_with_port(&current_url, &as_string(value))?,
        "username" => location_with_username(&current_url, &as_string(value))?,
        "password" => location_with_password(&current_url, &as_string(value))?,
        _ => unreachable!(),
    };
    host.document_set_location(&next_url)?;
    Ok(Some(Ok(())))
}

fn location_pathname(current_url: &str) -> String {
    let Some((path_start, path_end)) = location_path_bounds(current_url) else {
        return "/".to_string();
    };

    let path = &current_url[path_start..path_end];
    if path.is_empty() {
        "/".to_string()
    } else {
        path.to_string()
    }
}

fn location_protocol(current_url: &str) -> String {
    current_url
        .split_once(':')
        .map(|(scheme, _)| format!("{}:", scheme.to_ascii_lowercase()))
        .unwrap_or_default()
}

fn location_authority_bounds(url: &str) -> Option<(usize, usize)> {
    let (_, rest) = url.split_once(':')?;
    let after_slashes = rest.strip_prefix("//")?;
    let authority_end = after_slashes
        .find(['/', '?', '#'])
        .unwrap_or(after_slashes.len());
    let authority_start = url.len() - after_slashes.len();
    Some((authority_start, authority_start + authority_end))
}

fn location_authority(url: &str) -> Option<&str> {
    let (authority_start, authority_end) = location_authority_bounds(url)?;
    Some(&url[authority_start..authority_end])
}

#[derive(Clone, Debug)]
struct LocationAuthorityParts {
    username: String,
    password: String,
    host: String,
    hostname: String,
    port: String,
}

fn location_authority_parts(authority: &str) -> Option<LocationAuthorityParts> {
    let (userinfo, authority) = authority
        .rsplit_once('@')
        .map_or((None, authority), |(userinfo, host)| (Some(userinfo), host));
    let (username, password) = userinfo
        .map(|userinfo| userinfo.split_once(':').unwrap_or((userinfo, "")))
        .unwrap_or(("", ""));

    if let Some(rest) = authority.strip_prefix('[') {
        let end_bracket = rest.find(']')?;
        let hostname = rest[..end_bracket].to_ascii_lowercase();
        let port = rest[end_bracket + 1..].strip_prefix(':').unwrap_or("");
        let host = if port.is_empty() {
            format!("[{hostname}]")
        } else {
            format!("[{hostname}]:{port}")
        };
        return Some(LocationAuthorityParts {
            username: username.to_string(),
            password: password.to_string(),
            host,
            hostname,
            port: port.to_string(),
        });
    }

    let (hostname, port) = authority.split_once(':').unwrap_or((authority, ""));
    let hostname = hostname.to_ascii_lowercase();
    let host = if port.is_empty() {
        hostname.clone()
    } else {
        format!("{hostname}:{port}")
    };
    Some(LocationAuthorityParts {
        username: username.to_string(),
        password: password.to_string(),
        host,
        hostname,
        port: port.to_string(),
    })
}

fn location_authority_string(username: &str, password: &str, host: &str, port: &str) -> String {
    let mut authority = String::new();
    if !username.is_empty() || !password.is_empty() {
        authority.push_str(username);
        if !password.is_empty() || username.is_empty() {
            authority.push(':');
            authority.push_str(password);
        }
        authority.push('@');
    }

    authority.push_str(host);
    if !port.is_empty() {
        authority.push(':');
        authority.push_str(port);
    }
    authority
}

fn location_host(current_url: &str) -> String {
    location_authority(current_url)
        .and_then(location_authority_parts)
        .map(|parts| parts.host)
        .unwrap_or_default()
}

fn location_hostname(current_url: &str) -> String {
    location_authority(current_url)
        .and_then(location_authority_parts)
        .map(|parts| parts.hostname)
        .unwrap_or_default()
}

fn location_port(current_url: &str) -> String {
    location_authority(current_url)
        .and_then(location_authority_parts)
        .map(|parts| parts.port)
        .unwrap_or_default()
}

fn location_username(current_url: &str) -> String {
    location_authority(current_url)
        .and_then(location_authority_parts)
        .map(|parts| parts.username)
        .unwrap_or_default()
}

fn location_password(current_url: &str) -> String {
    location_authority(current_url)
        .and_then(location_authority_parts)
        .map(|parts| parts.password)
        .unwrap_or_default()
}

fn location_with_hash(current_url: &str, hash: &str) -> String {
    let (_path_start, path_end) =
        location_path_bounds(current_url).unwrap_or((current_url.len(), current_url.len()));
    let mut next_url = String::with_capacity(current_url.len() + hash.len());
    next_url.push_str(&current_url[..path_end]);

    let normalized = hash.trim();
    if normalized.is_empty() {
        next_url.truncate(path_end);
        return next_url;
    }

    next_url.push('#');
    if let Some(fragment) = normalized.strip_prefix('#') {
        next_url.push_str(fragment);
    } else {
        next_url.push_str(normalized);
    }

    next_url
}

fn normalize_location_hostname_for_authority(hostname: &str) -> String {
    let trimmed = hostname.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(trimmed)
        .to_ascii_lowercase();
    if trimmed.starts_with('[') && trimmed.ends_with(']') || inner.contains(':') {
        format!("[{inner}]")
    } else {
        inner
    }
}

fn location_with_protocol(current_url: &str, protocol: &str) -> Result<String> {
    let normalized = protocol.trim().trim_end_matches(':');
    if normalized.is_empty() {
        return Err(ScriptError::new(format!(
            "unsupported location.protocol value: {protocol}"
        )));
    }

    let Some((_scheme, rest)) = current_url.split_once(':') else {
        return Err(ScriptError::new(format!(
            "unsupported location.protocol value: {protocol}"
        )));
    };

    let mut next_url = String::with_capacity(normalized.len() + 1 + rest.len());
    next_url.push_str(normalized);
    next_url.push(':');
    next_url.push_str(rest);
    Ok(next_url)
}

fn location_with_authority(current_url: &str, authority: &str, property: &str) -> Result<String> {
    let normalized = authority.trim();
    if normalized.is_empty() {
        return Err(ScriptError::new(format!(
            "unsupported location.{property} value: {authority}"
        )));
    }

    let Some((authority_start, authority_end)) = location_authority_bounds(current_url) else {
        return Err(ScriptError::new(format!(
            "unsupported location.{property} value: {authority}"
        )));
    };

    let mut next_url = String::with_capacity(
        current_url.len() - (authority_end - authority_start) + normalized.len(),
    );
    next_url.push_str(&current_url[..authority_start]);
    next_url.push_str(normalized);
    next_url.push_str(&current_url[authority_end..]);
    Ok(next_url)
}

fn location_with_host(current_url: &str, host: &str) -> Result<String> {
    let normalized = host.trim();
    if normalized.is_empty() {
        return Err(ScriptError::new(format!(
            "unsupported location.host value: {host}"
        )));
    }

    let Some(current_authority) = location_authority(current_url) else {
        return Err(ScriptError::new(format!(
            "unsupported location.host value: {host}"
        )));
    };
    let Some(current_parts) = location_authority_parts(current_authority) else {
        return Err(ScriptError::new(format!(
            "unsupported location.host value: {host}"
        )));
    };
    let Some(next_parts) = location_authority_parts(normalized) else {
        return Err(ScriptError::new(format!(
            "unsupported location.host value: {host}"
        )));
    };

    let next_authority = location_authority_string(
        &current_parts.username,
        &current_parts.password,
        &next_parts.host,
        "",
    );
    location_with_authority(current_url, &next_authority, "host")
}

fn location_with_hostname(current_url: &str, hostname: &str) -> Result<String> {
    let normalized = normalize_location_hostname_for_authority(hostname);
    if normalized.is_empty() {
        return Err(ScriptError::new(format!(
            "unsupported location.hostname value: {hostname}"
        )));
    }

    let Some(authority) = location_authority(current_url) else {
        return Err(ScriptError::new(format!(
            "unsupported location.hostname value: {hostname}"
        )));
    };
    let Some(parts) = location_authority_parts(authority) else {
        return Err(ScriptError::new(format!(
            "unsupported location.hostname value: {hostname}"
        )));
    };

    let next_authority =
        location_authority_string(&parts.username, &parts.password, &normalized, &parts.port);
    location_with_authority(current_url, &next_authority, "hostname")
}

fn location_with_port(current_url: &str, port: &str) -> Result<String> {
    let normalized = port.trim();
    if !normalized.is_empty() && !normalized.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(ScriptError::new(format!(
            "unsupported location.port value: {port}"
        )));
    }

    let Some(authority) = location_authority(current_url) else {
        return Err(ScriptError::new(format!(
            "unsupported location.port value: {port}"
        )));
    };
    let Some(parts) = location_authority_parts(authority) else {
        return Err(ScriptError::new(format!(
            "unsupported location.port value: {port}"
        )));
    };

    let host = normalize_location_hostname_for_authority(&parts.hostname);
    let next_authority =
        location_authority_string(&parts.username, &parts.password, &host, normalized);
    location_with_authority(current_url, &next_authority, "port")
}

fn location_with_username(current_url: &str, username: &str) -> Result<String> {
    let normalized = username.trim();
    let Some(authority) = location_authority(current_url) else {
        return Err(ScriptError::new(format!(
            "unsupported location.username value: {username}"
        )));
    };
    let Some(parts) = location_authority_parts(authority) else {
        return Err(ScriptError::new(format!(
            "unsupported location.username value: {username}"
        )));
    };

    let next_authority = location_authority_string(normalized, &parts.password, &parts.host, "");
    location_with_authority(current_url, &next_authority, "username")
}

fn location_with_password(current_url: &str, password: &str) -> Result<String> {
    let normalized = password.trim();
    let Some(authority) = location_authority(current_url) else {
        return Err(ScriptError::new(format!(
            "unsupported location.password value: {password}"
        )));
    };
    let Some(parts) = location_authority_parts(authority) else {
        return Err(ScriptError::new(format!(
            "unsupported location.password value: {password}"
        )));
    };

    let next_authority = location_authority_string(&parts.username, normalized, &parts.host, "");
    location_with_authority(current_url, &next_authority, "password")
}

fn location_search_bounds(url: &str) -> Option<(usize, usize)> {
    let (_, path_end) = location_path_bounds(url)?;
    if !url[path_end..].starts_with('?') {
        return None;
    }

    let search_start = path_end + 1;
    let search_end = url[search_start..]
        .find('#')
        .map(|offset| search_start + offset)
        .unwrap_or(url.len());

    Some((search_start, search_end))
}

fn location_search(current_url: &str) -> String {
    let Some((search_start, search_end)) = location_search_bounds(current_url) else {
        return String::new();
    };

    let search = &current_url[search_start..search_end];
    if search.is_empty() {
        "?".to_string()
    } else {
        format!("?{search}")
    }
}

fn location_with_pathname(current_url: &str, pathname: &str) -> String {
    let (path_start, path_end) =
        location_path_bounds(current_url).unwrap_or((current_url.len(), current_url.len()));
    let mut next_url = String::with_capacity(current_url.len() + pathname.len() + 1);
    next_url.push_str(&current_url[..path_start]);

    if pathname.is_empty() {
        next_url.push('/');
    } else if pathname.starts_with('/') {
        next_url.push_str(pathname);
    } else {
        next_url.push('/');
        next_url.push_str(pathname);
    }

    next_url.push_str(&current_url[path_end..]);
    next_url
}

fn location_with_search(current_url: &str, search: &str) -> String {
    let (_path_start, path_end) =
        location_path_bounds(current_url).unwrap_or((current_url.len(), current_url.len()));
    let hash_start = current_url[path_end..]
        .find('#')
        .map(|offset| path_end + offset)
        .unwrap_or(current_url.len());
    let mut next_url = String::with_capacity(current_url.len() + search.len() + 1);
    next_url.push_str(&current_url[..path_end]);

    let normalized = search.trim();
    if normalized.is_empty() {
        next_url.push_str(&current_url[hash_start..]);
        return next_url;
    }

    next_url.push('?');
    if let Some(fragment) = normalized.strip_prefix('?') {
        next_url.push_str(fragment);
    } else {
        next_url.push_str(normalized);
    }

    next_url.push_str(&current_url[hash_start..]);
    next_url
}

fn location_path_bounds(url: &str) -> Option<(usize, usize)> {
    let scheme_end = url.find(':')?;
    let mut path_start = scheme_end + 1;

    if url[path_start..].starts_with("//") {
        path_start += 2;
        let authority_end = url[path_start..]
            .find(['/', '?', '#'])
            .unwrap_or_else(|| url.len().saturating_sub(path_start));
        path_start += authority_end;
    }

    let path_end = url[path_start..]
        .find(['?', '#'])
        .map(|offset| path_start + offset)
        .unwrap_or(url.len());

    Some((path_start, path_end))
}

fn eval_method_call<H: HostBindings>(
    object: Value,
    method: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    match object {
        Value::Document => match method {
            "getElementById" => {
                let [id_expr] = args else {
                    return Err(ScriptError::new(
                        "document.getElementById() expects exactly one argument",
                    ));
                };
                let id = as_string(&eval_expr(id_expr, env, host)?);
                let Some(element) = host.document_get_element_by_id(&id)? else {
                    return Err(ScriptError::new(format!(
                        "document.getElementById(\"{id}\") returned no element"
                    )));
                };
                Ok(Value::Element(element))
            }
            "createElement" => {
                let [tag_expr] = args else {
                    return Err(ScriptError::new(
                        "document.createElement() expects exactly one argument",
                    ));
                };
                let tag_name = as_string(&eval_expr(tag_expr, env, host)?);
                Ok(Value::Element(host.document_create_element(&tag_name)?))
            }
            "createTextNode" => {
                let [text_expr] = args else {
                    return Err(ScriptError::new(
                        "document.createTextNode() expects exactly one argument",
                    ));
                };
                let text = as_string(&eval_expr(text_expr, env, host)?);
                Ok(Value::Node(host.document_create_text_node(&text)?))
            }
            "createComment" => {
                let [text_expr] = args else {
                    return Err(ScriptError::new(
                        "document.createComment() expects exactly one argument",
                    ));
                };
                let text = as_string(&eval_expr(text_expr, env, host)?);
                Ok(Value::Node(host.document_create_comment(&text)?))
            }
            "createDocumentFragment" => {
                if !args.is_empty() {
                    return Err(ScriptError::new(
                        "document.createDocumentFragment() expects no arguments",
                    ));
                }
                Ok(Value::TemplateContent(
                    host.document_create_element("template")?,
                ))
            }
            "importNode" => document_import_node(args, env, host),
            "normalize" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("normalize() expects no arguments"));
                }
                host.document_normalize()?;
                Ok(Value::Undefined)
            }
            "contains" => document_contains(args, env, host),
            "isSameNode" => same_node(Value::Document, args, env, host),
            "isEqualNode" => equal_node(Value::Document, args, env, host),
            "compareDocumentPosition" => {
                compare_document_position(NodeHandle::new(0), args, env, host)
            }
            "hasChildNodes" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("hasChildNodes() expects no arguments"));
                }
                Ok(Value::Boolean(host.document_has_child_nodes()?))
            }
            "hasFocus" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("document.hasFocus() expects no arguments"));
                }
                Ok(Value::Boolean(host.document_has_focus()?))
            }
            "querySelector" => query_selector(QuerySelectorTarget::Document, args, env, host),
            "querySelectorAll" => {
                query_selector_all(QuerySelectorTarget::Document, args, env, host)
            }
            "getElementsByTagName" => {
                get_elements_by_tag_name(HtmlCollectionScope::Document, args, env, host)
            }
            "getElementsByTagNameNS" => {
                get_elements_by_tag_name_ns(HtmlCollectionScope::Document, args, env, host)
            }
            "getElementsByClassName" => {
                get_elements_by_class_name(HtmlCollectionScope::Document, args, env, host)
            }
            "getElementsByName" => get_elements_by_name(args, env, host),
            "addEventListener" => register_listener(ListenerTarget::Document, args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported Document method: {other}"
            ))),
        },
        Value::Window => match method {
            "alert" => {
                if args.len() > 1 {
                    return Err(ScriptError::new("alert() expects at most one argument"));
                }
                let message = if let Some(expr) = args.first() {
                    as_string(&eval_expr(expr, env, host)?)
                } else {
                    as_string(&Value::Undefined)
                };
                host.window_alert(&message)?;
                Ok(Value::Undefined)
            }
            "confirm" => {
                if args.len() > 1 {
                    return Err(ScriptError::new("confirm() expects at most one argument"));
                }
                let message = if let Some(expr) = args.first() {
                    as_string(&eval_expr(expr, env, host)?)
                } else {
                    as_string(&Value::Undefined)
                };
                Ok(Value::Boolean(host.window_confirm(&message)?))
            }
            "prompt" => {
                if args.len() > 2 {
                    return Err(ScriptError::new("prompt() expects at most two arguments"));
                }
                let message = if let Some(expr) = args.first() {
                    as_string(&eval_expr(expr, env, host)?)
                } else {
                    as_string(&Value::Undefined)
                };
                let default_text = match args.get(1) {
                    Some(expr) => Some(as_string(&eval_expr(expr, env, host)?)),
                    None => None,
                };
                match host.window_prompt(&message, default_text.as_deref())? {
                    Some(value) => Ok(Value::String(value)),
                    None => Ok(Value::Null),
                }
            }
            "open" => {
                if args.len() > 3 {
                    return Err(ScriptError::new("open() expects at most three arguments"));
                }
                let url = if let Some(expr) = args.first() {
                    Some(as_string(&eval_expr(expr, env, host)?))
                } else {
                    None
                };
                let target = if let Some(expr) = args.get(1) {
                    Some(as_string(&eval_expr(expr, env, host)?))
                } else {
                    None
                };
                let features = if let Some(expr) = args.get(2) {
                    Some(as_string(&eval_expr(expr, env, host)?))
                } else {
                    None
                };
                host.window_open(url.as_deref(), target.as_deref(), features.as_deref())?;
                Ok(Value::Undefined)
            }
            "close" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("close() expects no arguments"));
                }
                host.window_close()?;
                Ok(Value::Undefined)
            }
            "print" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("print() expects no arguments"));
                }
                host.window_print()?;
                Ok(Value::Undefined)
            }
            "scrollTo" => window_scroll_to(args, env, host),
            "scrollBy" => window_scroll_by(args, env, host),
            "matchMedia" => {
                let [query_expr] = args else {
                    return Err(ScriptError::new(
                        "matchMedia() expects exactly one argument",
                    ));
                };
                let query = as_string(&eval_expr(query_expr, env, host)?);
                Ok(Value::MediaQueryList(host.match_media(&query)?))
            }
            "addEventListener" => register_listener(ListenerTarget::Window, args, env, host),
            "document" => Ok(Value::Document),
            other => Err(ScriptError::new(format!(
                "unsupported Window method: {other}"
            ))),
        },
        Value::Element(element) => match method {
            "getAttribute" => element_get_attribute(element, args, env, host),
            "setAttribute" => element_set_attribute(element, args, env, host),
            "removeAttribute" => element_remove_attribute(element, args, env, host),
            "hasAttribute" => element_has_attribute(element, args, env, host),
            "toggleAttribute" => element_toggle_attribute(element, args, env, host),
            "isSameNode" => same_node(Value::Element(element), args, env, host),
            "isEqualNode" => equal_node(Value::Element(element), args, env, host),
            "compareDocumentPosition" => {
                compare_document_position(NodeHandle::new(element.raw()), args, env, host)
            }
            "cloneNode" => node_clone(NodeHandle::new(element.raw()), args, env, host),
            "replaceWith" => node_replace_with(NodeHandle::new(element.raw()), args, env, host),
            "contains" => node_contains(NodeHandle::new(element.raw()), args, env, host),
            "normalize" => node_normalize(NodeHandle::new(element.raw()), args, env, host),
            "hasChildNodes" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("hasChildNodes() expects no arguments"));
                }
                Ok(Value::Boolean(
                    host.node_has_child_nodes(NodeHandle::new(element.raw()))?,
                ))
            }
            "appendChild" => element_append_child(element, args, env, host),
            "insertBefore" => element_insert_before(element, args, env, host),
            "replaceChild" => element_replace_child(element, args, env, host),
            "replaceChildren" => element_replace_children(element, args, env, host),
            "append" => element_append(element, args, env, host),
            "prepend" => element_prepend(element, args, env, host),
            "before" => element_before(element, args, env, host),
            "after" => element_after(element, args, env, host),
            "insertAdjacentHTML" => element_insert_adjacent_html(element, args, env, host),
            "insertAdjacentElement" => element_insert_adjacent_element(element, args, env, host),
            "insertAdjacentText" => element_insert_adjacent_text(element, args, env, host),
            "remove" => element_remove(element, args, env, host),
            "querySelector" => {
                query_selector(QuerySelectorTarget::Element(element), args, env, host)
            }
            "querySelectorAll" => {
                query_selector_all(QuerySelectorTarget::Element(element), args, env, host)
            }
            "getElementsByTagName" => {
                get_elements_by_tag_name(HtmlCollectionScope::Element(element), args, env, host)
            }
            "getElementsByTagNameNS" => {
                get_elements_by_tag_name_ns(HtmlCollectionScope::Element(element), args, env, host)
            }
            "getElementsByClassName" => {
                get_elements_by_class_name(HtmlCollectionScope::Element(element), args, env, host)
            }
            "matches" => element_matches(element, args, env, host),
            "closest" => element_closest(element, args, env, host),
            "addEventListener" => {
                register_listener(ListenerTarget::Element(element), args, env, host)
            }
            other => Err(ScriptError::new(format!(
                "unsupported Element method: {other}"
            ))),
        },
        Value::Node(node) => match method {
            "isSameNode" => same_node(Value::Node(node), args, env, host),
            "isEqualNode" => equal_node(Value::Node(node), args, env, host),
            "cloneNode" => node_clone(node, args, env, host),
            "compareDocumentPosition" => compare_document_position(node, args, env, host),
            "before" => node_before(node, args, env, host),
            "after" => node_after(node, args, env, host),
            "replaceWith" => node_replace_with(node, args, env, host),
            "remove" => node_remove(node, args, env, host),
            "normalize" => node_normalize(node, args, env, host),
            "contains" => node_contains(node, args, env, host),
            "hasChildNodes" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("hasChildNodes() expects no arguments"));
                }
                Ok(Value::Boolean(host.node_has_child_nodes(node)?))
            }
            other => Err(ScriptError::new(format!(
                "unsupported Node method: {other}"
            ))),
        },
        Value::TemplateContent(element) => match method {
            "isSameNode" => same_node(Value::TemplateContent(element), args, env, host),
            "isEqualNode" => equal_node(Value::TemplateContent(element), args, env, host),
            "cloneNode" => {
                if args.len() > 1 {
                    return Err(ScriptError::new("cloneNode() expects at most one argument"));
                }

                let deep = match args.first() {
                    Some(expr) => is_truthy(&eval_expr(expr, env, host)?),
                    None => false,
                };
                let cloned = host.node_clone(NodeHandle::new(element.raw()), deep)?;
                if host.node_type(cloned)? != 1 {
                    return Err(ScriptError::new(
                        "template.content.cloneNode() expected a cloned <template> element",
                    ));
                }
                Ok(Value::TemplateContent(ElementHandle::new(cloned.raw())))
            }
            "contains" => node_contains(NodeHandle::new(element.raw()), args, env, host),
            "remove" => node_remove(NodeHandle::new(element.raw()), args, env, host),
            "normalize" => node_normalize(NodeHandle::new(element.raw()), args, env, host),
            "hasChildNodes" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("hasChildNodes() expects no arguments"));
                }
                Ok(Value::Boolean(
                    host.node_has_child_nodes(NodeHandle::new(element.raw()))?,
                ))
            }
            "appendChild" => element_append_child(element, args, env, host),
            "insertBefore" => element_insert_before(element, args, env, host),
            "replaceChild" => element_replace_child(element, args, env, host),
            "replaceChildren" => element_replace_children(element, args, env, host),
            "append" => element_append(element, args, env, host),
            "prepend" => element_prepend(element, args, env, host),
            "getElementById" => {
                let [id_expr] = args else {
                    return Err(ScriptError::new(
                        "getElementById() expects exactly one argument",
                    ));
                };
                let id = as_string(&eval_expr(id_expr, env, host)?);
                let selector = format!("#{}", css_escape_ident(&id));
                Ok(match host.element_query_selector(element, &selector)? {
                    Some(element) => Value::Element(element),
                    None => Value::Null,
                })
            }
            "querySelector" => query_selector(
                QuerySelectorTarget::TemplateContent(element),
                args,
                env,
                host,
            ),
            "querySelectorAll" => query_selector_all(
                QuerySelectorTarget::TemplateContent(element),
                args,
                env,
                host,
            ),
            other => Err(ScriptError::new(format!(
                "unsupported DocumentFragment method: {other}"
            ))),
        },
        Value::Event(event) => match method {
            "preventDefault" => {
                event.prevent_default();
                Ok(Value::Undefined)
            }
            "stopPropagation" => {
                event.stop_propagation();
                Ok(Value::Undefined)
            }
            "stopImmediatePropagation" => {
                event.stop_immediate_propagation();
                Ok(Value::Undefined)
            }
            other => Err(ScriptError::new(format!(
                "unsupported Event method: {other}"
            ))),
        },
        Value::ScreenOrientation(_) => Err(ScriptError::new(format!(
            "unsupported ScreenOrientation method: {method}"
        ))),
        Value::HtmlCollection(collection) => match method {
            "item" => html_collection_item(&collection, args, env, host),
            "namedItem" => html_collection_named_item(&collection, args, env, host),
            "forEach" => html_collection_for_each(&collection, args, env, host),
            "keys" => html_collection_keys(&collection, host),
            "values" => html_collection_values(&collection, host),
            "entries" => html_collection_entries(&collection, host),
            "add" => html_collection_select_options_add(&collection, args, env, host),
            "remove" => html_collection_select_options_remove(&collection, args, env, host),
            "toString" => collection_to_string("HTMLCollection", args),
            other => Err(ScriptError::new(format!(
                "unsupported HTMLCollection method: {other}"
            ))),
        },
        Value::StyleSheetList(target) => match method {
            "item" => style_sheet_list_item(&target, args, env, host),
            "namedItem" => style_sheet_list_named_item(&target, args, env, host),
            "keys" => style_sheet_list_keys(&target, host),
            "values" => style_sheet_list_values(&target, host),
            "entries" => style_sheet_list_entries(&target, host),
            "toString" => collection_to_string("StyleSheetList", args),
            other => Err(ScriptError::new(format!(
                "unsupported StyleSheetList method: {other}"
            ))),
        },
        Value::StyleSheet(_) => Err(ScriptError::new(format!(
            "cannot call `{method}` on a style sheet value"
        ))),
        Value::Navigator => match method {
            "javaEnabled" => {
                if !args.is_empty() {
                    return Err(ScriptError::new(
                        "window.navigator.javaEnabled() expects no arguments",
                    ));
                }
                Ok(Value::Boolean(host.window_navigator_java_enabled()?))
            }
            other => Err(ScriptError::new(format!(
                "cannot call `{other}` on a navigator value"
            ))),
        },
        Value::History => match method {
            "pushState" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(ScriptError::new(
                        "history.pushState() expects 2 or 3 arguments",
                    ));
                }
                let state = eval_expr(&args[0], env, host)?;
                let _ = eval_expr(&args[1], env, host)?;
                let url = match args.get(2) {
                    Some(expr) => Some(as_string(&eval_expr(expr, env, host)?)),
                    None => None,
                };
                let state = history_state_from_value(&state);
                host.window_history_push_state(state.as_deref(), url.as_deref())?;
                Ok(Value::Undefined)
            }
            "replaceState" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(ScriptError::new(
                        "history.replaceState() expects 2 or 3 arguments",
                    ));
                }
                let state = eval_expr(&args[0], env, host)?;
                let _ = eval_expr(&args[1], env, host)?;
                let url = match args.get(2) {
                    Some(expr) => Some(as_string(&eval_expr(expr, env, host)?)),
                    None => None,
                };
                let state = history_state_from_value(&state);
                host.window_history_replace_state(state.as_deref(), url.as_deref())?;
                Ok(Value::Undefined)
            }
            "back" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("history.back() expects no arguments"));
                }
                host.window_history_back()?;
                Ok(Value::Undefined)
            }
            "forward" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("history.forward() expects no arguments"));
                }
                host.window_history_forward()?;
                Ok(Value::Undefined)
            }
            "go" => {
                if args.len() > 1 {
                    return Err(ScriptError::new(
                        "history.go() expects at most one argument",
                    ));
                }
                let delta = match args.first() {
                    Some(expr) => history_delta_from_value(&eval_expr(expr, env, host)?)?,
                    None => 0,
                };
                host.window_history_go(delta)?;
                Ok(Value::Undefined)
            }
            other => Err(ScriptError::new(format!(
                "cannot call `{other}` on a history value"
            ))),
        },
        Value::Screen => Err(ScriptError::new(format!(
            "cannot call `{method}` on a screen value"
        ))),
        Value::ClassList(element) => match method {
            "contains" => class_list_contains(element, args, env, host),
            "add" => class_list_add(element, args, env, host),
            "remove" => class_list_remove(element, args, env, host),
            "toggle" => class_list_toggle(element, args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported class list method: {other}"
            ))),
        },
        Value::Dataset(_) => Err(ScriptError::new(format!(
            "cannot call `{method}` on a dataset value"
        ))),
        Value::NodeList(target) => match method {
            "item" => node_list_item(&target, args, env, host),
            "forEach" => node_list_for_each(&target, args, env, host),
            "keys" => node_list_keys(&target, host),
            "values" => node_list_values(&target, host),
            "entries" => node_list_entries(&target, host),
            "toString" => collection_to_string("NodeList", args),
            other => Err(ScriptError::new(format!(
                "unsupported NodeList method: {other}"
            ))),
        },
        Value::RadioNodeList(target) => match method {
            "item" => radio_node_list_item(&target, args, env, host),
            "forEach" => radio_node_list_for_each(&target, args, env, host),
            "keys" => radio_node_list_keys(&target, host),
            "values" => radio_node_list_values(&target, host),
            "entries" => radio_node_list_entries(&target, host),
            "toString" => collection_to_string("RadioNodeList", args),
            other => Err(ScriptError::new(format!(
                "unsupported RadioNodeList method: {other}"
            ))),
        },
        Value::Storage(target) => match method {
            "getItem" => storage_get_item(&target, args, env, host),
            "setItem" => storage_set_item(&target, args, env, host),
            "removeItem" => storage_remove_item(&target, args, env, host),
            "clear" => storage_clear(&target, args, env, host),
            "key" => storage_key(&target, args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported Storage method: {other}"
            ))),
        },
        Value::MediaQueryList(_) => Err(ScriptError::new(format!(
            "cannot call `{method}` on a media query list value"
        ))),
        Value::StringList(list) => match method {
            "item" => string_list_item(&list, args, env, host),
            "contains" => string_list_contains(&list, args, env, host),
            "keys" => Ok(string_list_keys(&list)),
            "values" => Ok(string_list_values(&list)),
            "entries" => Ok(string_list_entries(&list)),
            "toString" => string_list_to_string(args),
            other => Err(ScriptError::new(format!(
                "unsupported string list method: {other}"
            ))),
        },
        Value::MimeTypeArray(list) => match method {
            "item" => mime_type_array_item(&list, args, env, host),
            "namedItem" => mime_type_array_named_item(&list, args, env, host),
            "keys" => Ok(mime_type_array_keys(&list)),
            "values" => Ok(mime_type_array_values(&list)),
            "entries" => Ok(mime_type_array_entries(&list)),
            "toString" => collection_to_string("MimeTypeArray", args),
            other => Err(ScriptError::new(format!(
                "unsupported mime type array method: {other}"
            ))),
        },
        Value::CollectionIterator(iterator) => match method {
            "next" => collection_iterator_next(&iterator),
            other => Err(ScriptError::new(format!(
                "unsupported iterator method: {other}"
            ))),
        },
        Value::CollectionEntry(_) => Err(ScriptError::new(format!(
            "cannot call `{method}` on an iterator entry value"
        ))),
        Value::IteratorResult(_) => Err(ScriptError::new(format!(
            "cannot call `{method}` on an iterator result value"
        ))),
        Value::String(_) => Err(ScriptError::new(format!(
            "unsupported method call on string value: {method}"
        ))),
        Value::Number(_) => Err(ScriptError::new(format!(
            "unsupported method call on number value: {method}"
        ))),
        Value::Boolean(_) => Err(ScriptError::new(format!(
            "unsupported method call on boolean value: {method}"
        ))),
        Value::Null | Value::Undefined => Err(ScriptError::new(format!(
            "cannot call `{method}` on a nullish value"
        ))),
        Value::Function(_) => Err(ScriptError::new(format!(
            "cannot call `{method}` on a function value"
        ))),
    }
}

#[derive(Clone, Copy, Debug)]
enum QuerySelectorTarget {
    Document,
    Element(crate::ElementHandle),
    TemplateContent(crate::ElementHandle),
}

#[derive(Clone, Copy, Debug)]
enum MutationArgument {
    Node(NodeHandle),
    Fragment(crate::ElementHandle),
}

fn register_listener<H: HostBindings>(
    target: ListenerTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if !(2..=3).contains(&args.len()) {
        return Err(ScriptError::new(
            "addEventListener() expects two or three arguments",
        ));
    }

    let event = as_string(&eval_expr(&args[0], env, host)?);
    let handler = match eval_expr(&args[1], env, host)? {
        Value::Function(function) => function,
        _ => {
            return Err(ScriptError::new(
                "addEventListener() requires an arrow function callback",
            ));
        }
    };
    let capture = match args.get(2) {
        Some(capture_expr) => is_truthy(&eval_expr(capture_expr, env, host)?),
        None => false,
    };
    host.register_event_listener_with_capture(target, &event, capture, handler)?;
    Ok(Value::Undefined)
}

fn query_selector<H: HostBindings>(
    target: QuerySelectorTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [selector_expr] = args else {
        return Err(ScriptError::new(
            "querySelector() expects exactly one argument",
        ));
    };

    let selector = as_string(&eval_expr(selector_expr, env, host)?);
    let match_handle = match target {
        QuerySelectorTarget::Document => host.document_query_selector(&selector)?,
        QuerySelectorTarget::Element(element) => host.element_query_selector(element, &selector)?,
        QuerySelectorTarget::TemplateContent(element) => {
            host.element_query_selector(element, &selector)?
        }
    };

    Ok(match_handle.map(Value::Element).unwrap_or(Value::Null))
}

fn query_selector_all<H: HostBindings>(
    target: QuerySelectorTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [selector_expr] = args else {
        return Err(ScriptError::new(
            "querySelectorAll() expects exactly one argument",
        ));
    };

    let selector = as_string(&eval_expr(selector_expr, env, host)?);
    let matches = match target {
        QuerySelectorTarget::Document => host.document_query_selector_all(&selector)?,
        QuerySelectorTarget::Element(element) => {
            host.element_query_selector_all(element, &selector)?
        }
        QuerySelectorTarget::TemplateContent(element) => {
            host.element_query_selector_all(element, &selector)?
        }
    };

    Ok(Value::NodeList(NodeListTarget::Snapshot(matches)))
}

fn css_escape_ident(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars().peekable();
    let mut index = 0;

    while let Some(ch) = chars.next() {
        let safe = ch.is_ascii_alphanumeric()
            || ch == '_'
            || ch == '-'
            || (!ch.is_ascii() && !ch.is_whitespace() && !ch.is_control());
        let needs_escape = if index == 0 {
            ch.is_ascii_digit()
                || (ch == '-'
                    && chars
                        .peek()
                        .copied()
                        .is_some_and(|next| next.is_ascii_digit()))
        } else {
            !safe
        };

        if needs_escape {
            output.push('\\');
            output.push_str(&format!("{:x} ", ch as u32));
        } else {
            output.push(ch);
        }

        index += 1;
    }

    output
}

fn element_get_attribute<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [name_expr] = args else {
        return Err(ScriptError::new(
            "getAttribute() expects exactly one argument",
        ));
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    let value = host.element_get_attribute(element, &name)?;
    Ok(value.map(Value::String).unwrap_or(Value::Null))
}

fn element_set_attribute<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [name_expr, value_expr] = args else {
        return Err(ScriptError::new(
            "setAttribute() expects exactly two arguments",
        ));
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    let value = as_string(&eval_expr(value_expr, env, host)?);
    host.element_set_attribute(element, &name, &value)?;
    Ok(Value::Undefined)
}

fn element_remove_attribute<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [name_expr] = args else {
        return Err(ScriptError::new(
            "removeAttribute() expects exactly one argument",
        ));
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    host.element_remove_attribute(element, &name)?;
    Ok(Value::Undefined)
}

fn element_has_attribute<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [name_expr] = args else {
        return Err(ScriptError::new(
            "hasAttribute() expects exactly one argument",
        ));
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    Ok(Value::Boolean(host.element_has_attribute(element, &name)?))
}

fn element_toggle_attribute<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (name_expr, force_expr) = match args {
        [name_expr] => (name_expr, None),
        [name_expr, force_expr] => (name_expr, Some(force_expr)),
        _ => {
            return Err(ScriptError::new(
                "toggleAttribute() expects one or two arguments",
            ));
        }
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    let force = match force_expr {
        Some(expr) => Some(is_truthy(&eval_expr(expr, env, host)?)),
        None => None,
    };
    Ok(Value::Boolean(
        host.element_toggle_attribute(element, &name, force)?,
    ))
}

fn element_append_child<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [child_expr] = args else {
        return Err(ScriptError::new(
            "appendChild() expects exactly one argument",
        ));
    };

    match eval_mutation_argument(child_expr, env, host, "appendChild")? {
        MutationArgument::Node(child) => {
            host.element_append_child(element, child)?;
            value_for_node_handle(child, host)
        }
        MutationArgument::Fragment(fragment) => {
            let children = fragment_child_nodes(fragment, host)?;
            host.element_append(element, children)?;
            Ok(Value::TemplateContent(fragment))
        }
    }
}

fn element_insert_before<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [child_expr, reference_expr] = args else {
        return Err(ScriptError::new(
            "insertBefore() expects exactly two arguments",
        ));
    };

    let child = eval_mutation_argument(child_expr, env, host, "insertBefore")?;
    let reference = eval_optional_node_handle(reference_expr, env, host, "insertBefore")?;
    match child {
        MutationArgument::Node(child) => {
            host.element_insert_before(element, child, reference)?;
            value_for_node_handle(child, host)
        }
        MutationArgument::Fragment(fragment) => {
            let children = fragment_child_nodes(fragment, host)?;
            if let Some(reference) = reference {
                let parent = NodeHandle::new(element.raw());
                if host.node_parent(reference)? != Some(parent) {
                    return Err(ScriptError::new(
                        "insertBefore() expects the reference node to belong to the parent",
                    ));
                }
                for child in &children {
                    if host.node_contains(*child, parent)? {
                        return Err(ScriptError::new(
                            "insertBefore() cannot insert a node into its descendant",
                        ));
                    }
                }
                for child in children {
                    host.element_insert_before(element, child, Some(reference))?;
                }
            } else {
                host.element_append(element, children)?;
            }
            Ok(Value::TemplateContent(fragment))
        }
    }
}

fn element_replace_child<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [new_child_expr, old_child_expr] = args else {
        return Err(ScriptError::new(
            "replaceChild() expects exactly two arguments",
        ));
    };

    let new_child = eval_mutation_argument(new_child_expr, env, host, "replaceChild")?;
    let old_child = eval_node_handle(old_child_expr, env, host, "replaceChild")?;
    match new_child {
        MutationArgument::Node(new_child) => {
            host.element_replace_child(element, new_child, old_child)?;
            value_for_node_handle(old_child, host)
        }
        MutationArgument::Fragment(fragment) => {
            let parent = NodeHandle::new(element.raw());
            if host.node_parent(old_child)? != Some(parent) {
                return Err(ScriptError::new(
                    "replaceChild() expects the old child to belong to the parent",
                ));
            }
            let children = fragment_child_nodes(fragment, host)?;
            host.node_replace_with(old_child, children)?;
            value_for_node_handle(old_child, host)
        }
    }
}

fn element_replace_children<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_mutation_children(args, env, host, "replaceChildren")?;
    host.element_replace_children(element, children)?;
    Ok(Value::Undefined)
}

fn element_append<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_mutation_children(args, env, host, "append")?;
    host.element_append(element, children)?;
    Ok(Value::Undefined)
}

fn element_prepend<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_mutation_children(args, env, host, "prepend")?;
    host.element_prepend(element, children)?;
    Ok(Value::Undefined)
}

fn element_before<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_mutation_children(args, env, host, "before")?;
    host.element_before(element, children)?;
    Ok(Value::Undefined)
}

fn element_after<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_mutation_children(args, env, host, "after")?;
    host.element_after(element, children)?;
    Ok(Value::Undefined)
}

fn element_insert_adjacent_html<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [position_expr, html_expr] = args else {
        return Err(ScriptError::new(
            "insertAdjacentHTML() expects exactly two arguments",
        ));
    };

    let position = as_string(&eval_expr(position_expr, env, host)?);
    let html = as_string(&eval_expr(html_expr, env, host)?);
    host.element_insert_adjacent_html(element, &position, &html)?;
    Ok(Value::Undefined)
}

fn insert_adjacent_node<H: HostBindings>(
    element: crate::ElementHandle,
    position: &str,
    child: NodeHandle,
    method: &str,
    host: &mut H,
) -> Result<()> {
    match position {
        "beforebegin" => {
            let Some(_parent) = host.node_parent(NodeHandle::new(element.raw()))? else {
                return Err(ScriptError::new(format!(
                    "node {:?} has no parent for {method}(beforebegin)",
                    NodeHandle::new(element.raw())
                )));
            };
            host.element_before(element, vec![child])?;
        }
        "afterbegin" => {
            let tag_name = host.element_tag_name(element)?;
            if is_void_element(tag_name.as_str()) {
                return Err(ScriptError::new(format!(
                    "{method} is not supported on void elements like <{}>",
                    tag_name
                )));
            }
            host.element_prepend(element, vec![child])?;
        }
        "beforeend" => {
            let tag_name = host.element_tag_name(element)?;
            if is_void_element(tag_name.as_str()) {
                return Err(ScriptError::new(format!(
                    "{method} is not supported on void elements like <{}>",
                    tag_name
                )));
            }
            host.element_append(element, vec![child])?;
        }
        "afterend" => {
            let Some(_parent) = host.node_parent(NodeHandle::new(element.raw()))? else {
                return Err(ScriptError::new(format!(
                    "node {:?} has no parent for {method}(afterend)",
                    NodeHandle::new(element.raw())
                )));
            };
            host.element_after(element, vec![child])?;
        }
        _ => {
            return Err(ScriptError::new(format!(
                "unsupported {method} position `{position}`"
            )));
        }
    }

    Ok(())
}

fn is_void_element(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn element_insert_adjacent_element<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [position_expr, element_expr] = args else {
        return Err(ScriptError::new(
            "insertAdjacentElement() expects exactly two arguments",
        ));
    };

    let position = as_string(&eval_expr(position_expr, env, host)?);
    let inserted = eval_element_handle(element_expr, env, host, "insertAdjacentElement")?;
    insert_adjacent_node(
        element,
        &position,
        NodeHandle::new(inserted.raw()),
        "insertAdjacentElement",
        host,
    )?;
    Ok(Value::Element(inserted))
}

fn element_insert_adjacent_text<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [position_expr, text_expr] = args else {
        return Err(ScriptError::new(
            "insertAdjacentText() expects exactly two arguments",
        ));
    };

    let position = as_string(&eval_expr(position_expr, env, host)?);
    let text = as_string(&eval_expr(text_expr, env, host)?);
    match position.as_str() {
        "beforebegin" => {
            if host.node_parent(NodeHandle::new(element.raw()))?.is_none() {
                return Err(ScriptError::new(format!(
                    "node {:?} has no parent for insertAdjacentText(beforebegin)",
                    NodeHandle::new(element.raw())
                )));
            }
        }
        "afterbegin" | "beforeend" => {
            let tag_name = host.element_tag_name(element)?;
            if is_void_element(tag_name.as_str()) {
                return Err(ScriptError::new(format!(
                    "insertAdjacentText is not supported on void elements like <{}>",
                    tag_name
                )));
            }
        }
        "afterend" => {
            if host.node_parent(NodeHandle::new(element.raw()))?.is_none() {
                return Err(ScriptError::new(format!(
                    "node {:?} has no parent for insertAdjacentText(afterend)",
                    NodeHandle::new(element.raw())
                )));
            }
        }
        _ => {
            return Err(ScriptError::new(format!(
                "unsupported insertAdjacentText position `{position}`"
            )));
        }
    }

    let child = host.document_create_text_node(&text)?;
    insert_adjacent_node(element, &position, child, "insertAdjacentText", host)?;
    Ok(Value::Undefined)
}

fn element_remove<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("remove() expects no arguments"));
    }

    host.element_remove(element)?;
    Ok(Value::Undefined)
}

fn eval_node_handle<H: HostBindings>(
    expr: &Expr,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<NodeHandle> {
    match eval_expr(expr, env, host)? {
        Value::Element(element) => Ok(NodeHandle::new(element.raw())),
        Value::Node(node) => Ok(node),
        _ => Err(ScriptError::new(format!(
            "{method}() expects node arguments"
        ))),
    }
}

fn eval_mutation_argument<H: HostBindings>(
    expr: &Expr,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<MutationArgument> {
    Ok(match eval_expr(expr, env, host)? {
        Value::Element(element) => MutationArgument::Node(NodeHandle::new(element.raw())),
        Value::Node(node) => MutationArgument::Node(node),
        Value::TemplateContent(element) => MutationArgument::Fragment(element),
        _ => {
            return Err(ScriptError::new(format!(
                "{method}() expects node or DocumentFragment arguments"
            )));
        }
    })
}

fn eval_element_handle<H: HostBindings>(
    expr: &Expr,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<ElementHandle> {
    match eval_expr(expr, env, host)? {
        Value::Element(element) => Ok(element),
        _ => Err(ScriptError::new(format!(
            "{method}() expects element arguments"
        ))),
    }
}

fn eval_optional_node_handle<H: HostBindings>(
    expr: &Expr,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<Option<NodeHandle>> {
    let value = eval_expr(expr, env, host)?;
    match value {
        Value::Element(element) => Ok(Some(NodeHandle::new(element.raw()))),
        Value::Node(node) => Ok(Some(node)),
        Value::Null | Value::Undefined => Ok(None),
        _ => Err(ScriptError::new(format!(
            "{method}() expects a node or null reference"
        ))),
    }
}

fn fragment_child_nodes<H: HostBindings>(
    fragment: crate::ElementHandle,
    host: &mut H,
) -> Result<Vec<NodeHandle>> {
    host.node_child_nodes_items(HtmlCollectionScope::Element(fragment))
}

fn eval_mutation_children<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<Vec<NodeHandle>> {
    let mut children = Vec::new();

    for expr in args {
        match eval_mutation_argument(expr, env, host, method)? {
            MutationArgument::Node(node) => children.push(node),
            MutationArgument::Fragment(fragment) => {
                children.extend(fragment_child_nodes(fragment, host)?);
            }
        }
    }

    Ok(children)
}

fn get_elements_by_tag_name<H: HostBindings>(
    scope: HtmlCollectionScope,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [tag_expr] = args else {
        return Err(ScriptError::new(
            "getElementsByTagName() expects exactly one argument",
        ));
    };

    let tag_name = as_string(&eval_expr(tag_expr, env, host)?);
    Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagName {
        scope,
        tag_name,
    }))
}

fn get_elements_by_tag_name_ns<H: HostBindings>(
    scope: HtmlCollectionScope,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [namespace_expr, local_name_expr] = args else {
        return Err(ScriptError::new(
            "getElementsByTagNameNS() expects exactly two arguments",
        ));
    };

    let namespace_uri = as_string(&eval_expr(namespace_expr, env, host)?);
    let local_name = as_string(&eval_expr(local_name_expr, env, host)?);
    Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagNameNs {
        scope,
        namespace_uri,
        local_name,
    }))
}

fn get_elements_by_class_name<H: HostBindings>(
    scope: HtmlCollectionScope,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [class_expr] = args else {
        return Err(ScriptError::new(
            "getElementsByClassName() expects exactly one argument",
        ));
    };

    let class_names = as_string(&eval_expr(class_expr, env, host)?);
    Ok(Value::HtmlCollection(HtmlCollectionTarget::ByClassName {
        scope,
        class_names,
    }))
}

fn get_elements_by_name<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [name_expr] = args else {
        return Err(ScriptError::new(
            "getElementsByName() expects exactly one argument",
        ));
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    Ok(Value::NodeList(NodeListTarget::ByName(name)))
}

fn element_matches<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [selector_expr] = args else {
        return Err(ScriptError::new("matches() expects exactly one argument"));
    };

    let selector = as_string(&eval_expr(selector_expr, env, host)?);
    Ok(Value::Boolean(host.element_matches(element, &selector)?))
}

fn element_closest<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [selector_expr] = args else {
        return Err(ScriptError::new("closest() expects exactly one argument"));
    };

    let selector = as_string(&eval_expr(selector_expr, env, host)?);
    let match_handle = host.element_closest(element, &selector)?;

    Ok(match_handle.map(Value::Element).unwrap_or(Value::Null))
}

#[derive(Clone, Debug, PartialEq)]
enum NodeListItem {
    Element(crate::ElementHandle),
    Node(NodeHandle),
}

impl NodeListItem {
    fn into_value(self) -> Value {
        match self {
            NodeListItem::Element(handle) => Value::Element(handle),
            NodeListItem::Node(handle) => Value::Node(handle),
        }
    }
}

fn node_list_item<H: HostBindings>(
    target: &NodeListTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [index_expr] = args else {
        return Err(ScriptError::new(
            "NodeList.item() expects exactly one argument",
        ));
    };

    let index_value = eval_expr(index_expr, env, host)?;
    let Some(index) = index_from_value(&index_value) else {
        return Ok(Value::Null);
    };

    Ok(node_list_items(target, host)?
        .get(index)
        .cloned()
        .map(NodeListItem::into_value)
        .unwrap_or(Value::Null))
}

fn radio_node_list_item<H: HostBindings>(
    target: &RadioNodeListTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [index_expr] = args else {
        return Err(ScriptError::new(
            "RadioNodeList.item() expects exactly one argument",
        ));
    };

    let index_value = eval_expr(index_expr, env, host)?;
    let Some(index) = index_from_value(&index_value) else {
        return Ok(Value::Null);
    };

    Ok(radio_node_list_items(target, host)?
        .get(index)
        .copied()
        .map(Value::Element)
        .unwrap_or(Value::Null))
}

fn html_collection_item<H: HostBindings>(
    collection: &HtmlCollectionTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [index_expr] = args else {
        return Err(ScriptError::new(
            "HTMLCollection.item() expects exactly one argument",
        ));
    };

    let index_value = eval_expr(index_expr, env, host)?;
    let Some(index) = index_from_value(&index_value) else {
        return Ok(Value::Null);
    };

    let items = html_collection_items(collection, host)?;

    Ok(items
        .get(index)
        .copied()
        .map(Value::Element)
        .unwrap_or(Value::Null))
}

fn html_collection_named_item<H: HostBindings>(
    collection: &HtmlCollectionTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [name_expr] = args else {
        return Err(ScriptError::new(
            "HTMLCollection.namedItem() expects exactly one argument",
        ));
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    Ok(
        match html_collection_named_item_handle(collection, &name, host)? {
            Some(HtmlCollectionNamedItem::Element(handle)) => Value::Element(handle),
            Some(HtmlCollectionNamedItem::RadioNodeList(target)) => Value::RadioNodeList(target),
            None => Value::Null,
        },
    )
}

fn html_collection_select_options_add<H: HostBindings>(
    collection: &HtmlCollectionTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [option_expr] = args else {
        return Err(ScriptError::new(
            "select.options.add() expects exactly one argument",
        ));
    };

    let option = eval_element_handle(option_expr, env, host, "select.options.add")?;
    match collection {
        HtmlCollectionTarget::SelectOptions(element) => {
            host.html_collection_select_options_add(*element, option)?;
            Ok(Value::Undefined)
        }
        _ => Err(ScriptError::new(
            "add() is only supported on select.options in this workspace",
        )),
    }
}

fn html_collection_select_options_remove<H: HostBindings>(
    collection: &HtmlCollectionTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [index_expr] = args else {
        return Err(ScriptError::new(
            "select.options.remove() expects exactly one argument",
        ));
    };

    let index_value = eval_expr(index_expr, env, host)?;
    let Some(index) = index_from_value(&index_value) else {
        return Ok(Value::Undefined);
    };

    match collection {
        HtmlCollectionTarget::SelectOptions(element) => {
            host.html_collection_select_options_remove(*element, index)?;
            Ok(Value::Undefined)
        }
        _ => Err(ScriptError::new(
            "remove() is only supported on select.options in this workspace",
        )),
    }
}

fn html_collection_for_each<H: HostBindings>(
    collection: &HtmlCollectionTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new(
                "HTMLCollection.forEach() expects one or two arguments",
            ));
        }
    };

    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => {
            return Err(ScriptError::new(
                "HTMLCollection.forEach() requires an arrow function callback",
            ));
        }
    };
    if let Some(this_arg_expr) = this_arg_expr {
        let _ = eval_expr(this_arg_expr, env, host)?;
    }

    let items = html_collection_items(collection, host)?
        .into_iter()
        .map(|handle| Value::Element(handle))
        .collect();
    let collection_value = Value::HtmlCollection(collection.clone());
    for_each_over_items(&callback, items, collection_value, env, host)
}

fn html_collection_keys<H: HostBindings>(
    collection: &HtmlCollectionTarget,
    host: &mut H,
) -> Result<Value> {
    let items = html_collection_items(collection, host)?;
    Ok(collection_iterator(
        (0..items.len())
            .map(|index| Value::Number(index as f64))
            .collect(),
    ))
}

fn html_collection_values<H: HostBindings>(
    collection: &HtmlCollectionTarget,
    host: &mut H,
) -> Result<Value> {
    let items = html_collection_items(collection, host)?;
    Ok(collection_iterator(
        items.into_iter().map(Value::Element).collect(),
    ))
}

fn html_collection_entries<H: HostBindings>(
    collection: &HtmlCollectionTarget,
    host: &mut H,
) -> Result<Value> {
    let items = html_collection_items(collection, host)?;
    Ok(collection_entries(
        items.into_iter().map(Value::Element).collect(),
    ))
}

fn style_sheet_list_item<H: HostBindings>(
    target: &StyleSheetListTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [index_expr] = args else {
        return Err(ScriptError::new(
            "StyleSheetList.item() expects exactly one argument",
        ));
    };

    let index_value = eval_expr(index_expr, env, host)?;
    let Some(index) = index_from_value(&index_value) else {
        return Ok(Value::Null);
    };

    Ok(style_sheet_list_items(target, host)?
        .get(index)
        .copied()
        .map(|handle| Value::StyleSheet(StyleSheetTarget::OwnerNode(handle)))
        .unwrap_or(Value::Null))
}

fn style_sheet_list_named_item<H: HostBindings>(
    target: &StyleSheetListTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [name_expr] = args else {
        return Err(ScriptError::new(
            "StyleSheetList.namedItem() expects exactly one argument",
        ));
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    Ok(
        match style_sheet_list_named_item_handle(target, &name, host)? {
            Some(handle) => Value::StyleSheet(StyleSheetTarget::OwnerNode(handle)),
            None => Value::Null,
        },
    )
}

fn style_sheet_list_keys<H: HostBindings>(
    target: &StyleSheetListTarget,
    host: &mut H,
) -> Result<Value> {
    let items = style_sheet_list_items(target, host)?;
    Ok(collection_iterator(
        (0..items.len())
            .map(|index| Value::Number(index as f64))
            .collect(),
    ))
}

fn style_sheet_list_values<H: HostBindings>(
    target: &StyleSheetListTarget,
    host: &mut H,
) -> Result<Value> {
    let items = style_sheet_list_items(target, host)?;
    Ok(collection_iterator(
        items
            .into_iter()
            .map(|handle| Value::StyleSheet(StyleSheetTarget::OwnerNode(handle)))
            .collect(),
    ))
}

fn style_sheet_list_entries<H: HostBindings>(
    target: &StyleSheetListTarget,
    host: &mut H,
) -> Result<Value> {
    let items = style_sheet_list_items(target, host)?;
    Ok(collection_entries(
        items
            .into_iter()
            .map(|handle| Value::StyleSheet(StyleSheetTarget::OwnerNode(handle)))
            .collect(),
    ))
}

fn html_collection_items<H: HostBindings>(
    collection: &HtmlCollectionTarget,
    host: &mut H,
) -> Result<Vec<crate::ElementHandle>> {
    match collection {
        HtmlCollectionTarget::Children(element) => host.element_children(*element),
        HtmlCollectionTarget::ByTagName { .. } => {
            host.html_collection_tag_name_items(collection.clone())
        }
        HtmlCollectionTarget::ByTagNameNs { .. } => {
            host.html_collection_tag_name_ns_items(collection.clone())
        }
        HtmlCollectionTarget::ByClassName { .. } => {
            host.html_collection_class_name_items(collection.clone())
        }
        HtmlCollectionTarget::FormElements(element) => {
            host.html_collection_form_elements_items(*element)
        }
        HtmlCollectionTarget::SelectOptions(element) => {
            host.html_collection_select_options_items(*element)
        }
        HtmlCollectionTarget::SelectSelectedOptions(element) => {
            host.html_collection_select_selected_options_items(*element)
        }
        HtmlCollectionTarget::DocumentLinks => host.html_collection_document_links_items(),
        HtmlCollectionTarget::DocumentAnchors => host.html_collection_document_anchors_items(),
        HtmlCollectionTarget::DocumentChildren => host.html_collection_document_children_items(),
        HtmlCollectionTarget::WindowFrames => host.html_collection_window_frames_items(),
        HtmlCollectionTarget::MapAreas(element) => host.html_collection_map_areas_items(*element),
        HtmlCollectionTarget::TableTBodies(element) => {
            host.html_collection_table_bodies_items(*element)
        }
        HtmlCollectionTarget::TableRows(element) => host.html_collection_table_rows_items(*element),
        HtmlCollectionTarget::RowCells(element) => host.html_collection_row_cells_items(*element),
    }
}

fn style_sheet_list_items<H: HostBindings>(
    target: &StyleSheetListTarget,
    host: &mut H,
) -> Result<Vec<crate::ElementHandle>> {
    match target {
        StyleSheetListTarget::Document => host.document_style_sheets_items(),
    }
}

fn style_sheet_list_named_item_handle<H: HostBindings>(
    target: &StyleSheetListTarget,
    name: &str,
    host: &mut H,
) -> Result<Option<crate::ElementHandle>> {
    match target {
        StyleSheetListTarget::Document => host.document_style_sheets_named_item(name),
    }
}

fn html_collection_named_item_handle<H: HostBindings>(
    collection: &HtmlCollectionTarget,
    name: &str,
    host: &mut H,
) -> Result<Option<HtmlCollectionNamedItem>> {
    match collection {
        HtmlCollectionTarget::Children(element) => host
            .html_collection_named_item(*element, name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::ByTagName { .. } => host
            .html_collection_tag_name_named_item(collection.clone(), name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::ByTagNameNs { .. } => host
            .html_collection_tag_name_ns_named_item(collection.clone(), name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::ByClassName { .. } => host
            .html_collection_class_name_named_item(collection.clone(), name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::FormElements(element) => {
            let items = host.html_collection_form_elements_named_items(*element, name)?;
            Ok(match items.len() {
                0 => None,
                1 => Some(HtmlCollectionNamedItem::Element(items[0])),
                _ => Some(HtmlCollectionNamedItem::RadioNodeList(
                    RadioNodeListTarget::FormElements {
                        element: *element,
                        name: name.to_string(),
                    },
                )),
            })
        }
        HtmlCollectionTarget::SelectOptions(element) => host
            .html_collection_select_options_named_item(*element, name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::SelectSelectedOptions(element) => host
            .html_collection_select_selected_options_named_item(*element, name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::DocumentLinks => host
            .html_collection_document_links_named_item(name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::DocumentAnchors => host
            .html_collection_document_anchors_named_item(name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::DocumentChildren => host
            .html_collection_document_children_named_item(name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::WindowFrames => host
            .html_collection_window_frames_named_item(name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::MapAreas(element) => host
            .html_collection_map_areas_named_item(*element, name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::TableTBodies(element) => host
            .html_collection_table_bodies_named_item(*element, name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::TableRows(element) => host
            .html_collection_table_rows_named_item(*element, name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
        HtmlCollectionTarget::RowCells(element) => host
            .html_collection_row_cells_named_item(*element, name)
            .map(|value| value.map(HtmlCollectionNamedItem::Element)),
    }
}

fn node_list_for_each<H: HostBindings>(
    target: &NodeListTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new(
                "NodeList.forEach() expects one or two arguments",
            ));
        }
    };

    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => {
            return Err(ScriptError::new(
                "NodeList.forEach() requires an arrow function callback",
            ));
        }
    };
    if let Some(this_arg_expr) = this_arg_expr {
        let _ = eval_expr(this_arg_expr, env, host)?;
    }

    let items = node_list_items(target, host)?
        .into_iter()
        .map(NodeListItem::into_value)
        .collect();
    let collection_value = Value::NodeList(target.clone());
    for_each_over_items(&callback, items, collection_value, env, host)
}

fn radio_node_list_for_each<H: HostBindings>(
    target: &RadioNodeListTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new(
                "RadioNodeList.forEach() expects one or two arguments",
            ));
        }
    };

    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => {
            return Err(ScriptError::new(
                "RadioNodeList.forEach() requires an arrow function callback",
            ));
        }
    };
    if let Some(this_arg_expr) = this_arg_expr {
        let _ = eval_expr(this_arg_expr, env, host)?;
    }

    let items = radio_node_list_items(target, host)?
        .into_iter()
        .map(|handle| Value::Element(handle))
        .collect();
    let collection_value = Value::RadioNodeList(target.clone());
    for_each_over_items(&callback, items, collection_value, env, host)
}

fn node_list_keys<H: HostBindings>(target: &NodeListTarget, host: &mut H) -> Result<Value> {
    let items = node_list_items(target, host)?;
    Ok(collection_iterator(
        (0..items.len())
            .map(|index| Value::Number(index as f64))
            .collect(),
    ))
}

fn node_list_values<H: HostBindings>(target: &NodeListTarget, host: &mut H) -> Result<Value> {
    let items = node_list_items(target, host)?;
    Ok(collection_iterator(
        items.into_iter().map(NodeListItem::into_value).collect(),
    ))
}

fn node_list_entries<H: HostBindings>(target: &NodeListTarget, host: &mut H) -> Result<Value> {
    let items = node_list_items(target, host)?;
    Ok(collection_entries(
        items.into_iter().map(NodeListItem::into_value).collect(),
    ))
}

fn radio_node_list_keys<H: HostBindings>(
    target: &RadioNodeListTarget,
    host: &mut H,
) -> Result<Value> {
    let items = radio_node_list_items(target, host)?;
    Ok(collection_iterator(
        (0..items.len())
            .map(|index| Value::Number(index as f64))
            .collect(),
    ))
}

fn radio_node_list_values<H: HostBindings>(
    target: &RadioNodeListTarget,
    host: &mut H,
) -> Result<Value> {
    let items = radio_node_list_items(target, host)?;
    Ok(collection_iterator(
        items.into_iter().map(Value::Element).collect(),
    ))
}

fn radio_node_list_entries<H: HostBindings>(
    target: &RadioNodeListTarget,
    host: &mut H,
) -> Result<Value> {
    let items = radio_node_list_items(target, host)?;
    Ok(collection_entries(
        items.into_iter().map(Value::Element).collect(),
    ))
}

fn string_list_item<H: HostBindings>(
    list: &StringListState,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [index_expr] = args else {
        return Err(ScriptError::new(
            "navigator.languages.item() expects exactly one argument",
        ));
    };

    let index_value = eval_expr(index_expr, env, host)?;
    let Some(index) = index_from_value(&index_value) else {
        return Ok(Value::Null);
    };

    Ok(list
        .item(index)
        .map(|value| Value::String(value.to_string()))
        .unwrap_or(Value::Null))
}

fn string_list_contains<H: HostBindings>(
    list: &StringListState,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [value_expr] = args else {
        return Err(ScriptError::new(
            "navigator.languages.contains() expects exactly one argument",
        ));
    };

    let value = as_string(&eval_expr(value_expr, env, host)?);
    Ok(Value::Boolean(list.contains(&value)))
}

fn string_list_to_string(args: &[Expr]) -> Result<Value> {
    let [] = args else {
        return Err(ScriptError::new(
            "navigator.languages.toString() expects no arguments",
        ));
    };

    Ok(Value::String("[object DOMStringList]".to_string()))
}

fn string_list_keys(list: &StringListState) -> Value {
    collection_iterator(
        (0..list.length())
            .map(|index| Value::Number(index as f64))
            .collect(),
    )
}

fn string_list_values(list: &StringListState) -> Value {
    collection_iterator(list.items().iter().cloned().map(Value::String).collect())
}

fn string_list_entries(list: &StringListState) -> Value {
    collection_entries(list.items().iter().cloned().map(Value::String).collect())
}

fn collection_to_string(tag: &'static str, args: &[Expr]) -> Result<Value> {
    let [] = args else {
        return Err(ScriptError::new(format!(
            "{tag}.toString() expects no arguments"
        )));
    };

    Ok(Value::String(format!("[object {tag}]")))
}

fn mime_type_array_item<H: HostBindings>(
    list: &MimeTypeArrayState,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [index_expr] = args else {
        return Err(ScriptError::new(
            "navigator.mimeTypes.item() expects exactly one argument",
        ));
    };

    let index_value = eval_expr(index_expr, env, host)?;
    let Some(index) = index_from_value(&index_value) else {
        return Ok(Value::Null);
    };

    Ok(list
        .item(index)
        .map(|value| Value::String(value.to_string()))
        .unwrap_or(Value::Null))
}

fn mime_type_array_named_item<H: HostBindings>(
    list: &MimeTypeArrayState,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [name_expr] = args else {
        return Err(ScriptError::new(
            "navigator.mimeTypes.namedItem() expects exactly one argument",
        ));
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    Ok(list
        .named_item(&name)
        .map(|value| Value::String(value.to_string()))
        .unwrap_or(Value::Null))
}

fn mime_type_array_keys(list: &MimeTypeArrayState) -> Value {
    collection_iterator(
        (0..list.length())
            .map(|index| Value::Number(index as f64))
            .collect(),
    )
}

fn mime_type_array_values(list: &MimeTypeArrayState) -> Value {
    collection_iterator(list.items().iter().cloned().map(Value::String).collect())
}

fn mime_type_array_entries(list: &MimeTypeArrayState) -> Value {
    collection_entries(list.items().iter().cloned().map(Value::String).collect())
}

fn node_list_items<H: HostBindings>(
    target: &NodeListTarget,
    host: &mut H,
) -> Result<Vec<NodeListItem>> {
    match target {
        NodeListTarget::Snapshot(nodes) => {
            Ok(nodes.iter().copied().map(NodeListItem::Element).collect())
        }
        NodeListTarget::ByName(name) => Ok(host
            .document_get_elements_by_name(name)?
            .into_iter()
            .map(NodeListItem::Element)
            .collect()),
        NodeListTarget::Labels(element) => Ok(host
            .element_labels(*element)?
            .into_iter()
            .map(NodeListItem::Element)
            .collect()),
        NodeListTarget::ChildNodes(scope) => Ok(host
            .node_child_nodes_items(scope.clone())?
            .into_iter()
            .map(NodeListItem::Node)
            .collect()),
    }
}

fn radio_node_list_items<H: HostBindings>(
    target: &RadioNodeListTarget,
    host: &mut H,
) -> Result<Vec<crate::ElementHandle>> {
    match target {
        RadioNodeListTarget::FormElements { element, name } => {
            host.html_collection_form_elements_named_items(*element, name)
        }
    }
}

fn radio_node_list_value<H: HostBindings>(
    target: &RadioNodeListTarget,
    host: &mut H,
) -> Result<String> {
    let items = radio_node_list_items(target, host)?;
    for item in items {
        if !host.element_checked(item)? {
            continue;
        }
        return host.element_value(item);
    }

    Ok(String::new())
}

fn for_each_over_items<H: HostBindings>(
    callback: &crate::ScriptFunction,
    items: Vec<Value>,
    collection_value: Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let program = crate::parser::parse_program(&callback.body_source)?;

    for (index, item) in items.into_iter().enumerate() {
        let mut bindings = env.clone();
        for (param_index, param) in callback.params.iter().enumerate() {
            let value = match param_index {
                0 => item.clone(),
                1 => Value::Number(index as f64),
                2 => collection_value.clone(),
                _ => Value::Undefined,
            };
            bindings.insert(param.clone(), value);
        }
        eval_program_with_bindings(&program, host, bindings)?;
    }

    Ok(Value::Undefined)
}

fn collection_iterator_next(iterator: &CollectionIteratorHandle) -> Result<Value> {
    Ok(Value::IteratorResult(Box::new(iterator.next_result())))
}

fn collection_iterator(items: Vec<Value>) -> Value {
    Value::CollectionIterator(CollectionIteratorHandle::new(items))
}

fn collection_entries(items: Vec<Value>) -> Value {
    collection_iterator(
        items
            .into_iter()
            .enumerate()
            .map(|(index, value)| Value::CollectionEntry(CollectionEntryHandle::new(index, value)))
            .collect(),
    )
}

fn storage_get_item<H: HostBindings>(
    target: &StorageTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [key_expr] = args else {
        return Err(ScriptError::new("getItem() expects exactly one argument"));
    };

    let key = as_string(&eval_expr(key_expr, env, host)?);
    Ok(match host.storage_get_item(target.clone(), &key)? {
        Some(value) => Value::String(value),
        None => Value::Null,
    })
}

fn storage_set_item<H: HostBindings>(
    target: &StorageTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [key_expr, value_expr] = args else {
        return Err(ScriptError::new("setItem() expects exactly two arguments"));
    };

    let key = as_string(&eval_expr(key_expr, env, host)?);
    let value = as_string(&eval_expr(value_expr, env, host)?);
    host.storage_set_item(target.clone(), &key, &value)?;
    Ok(Value::Undefined)
}

fn storage_remove_item<H: HostBindings>(
    target: &StorageTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [key_expr] = args else {
        return Err(ScriptError::new(
            "removeItem() expects exactly one argument",
        ));
    };

    let key = as_string(&eval_expr(key_expr, env, host)?);
    host.storage_remove_item(target.clone(), &key)?;
    Ok(Value::Undefined)
}

fn storage_clear<H: HostBindings>(
    target: &StorageTarget,
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("clear() expects no arguments"));
    }

    host.storage_clear(target.clone())?;
    Ok(Value::Undefined)
}

fn storage_key<H: HostBindings>(
    target: &StorageTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [index_expr] = args else {
        return Err(ScriptError::new("key() expects exactly one argument"));
    };

    let index = index_from_value(&eval_expr(index_expr, env, host)?)
        .ok_or_else(|| ScriptError::new("key() expects a non-negative integer argument"))?;
    Ok(match host.storage_key(target.clone(), index)? {
        Some(value) => Value::String(value),
        None => Value::Null,
    })
}

fn storage_property_is_reserved(property: &str) -> bool {
    matches!(
        property,
        "length" | "getItem" | "setItem" | "removeItem" | "clear" | "key"
    )
}

fn html_collection_property_is_reserved(property: &str) -> bool {
    matches!(
        property,
        "item" | "namedItem" | "forEach" | "keys" | "values" | "entries" | "toString"
    )
}

fn window_scroll_to<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 2 {
        return Err(ScriptError::new("scrollTo() expects at most two arguments"));
    }

    let x = if let Some(expr) = args.first() {
        scroll_coordinate(&eval_expr(expr, env, host)?, "scrollTo")?
    } else {
        0
    };
    let y = if let Some(expr) = args.get(1) {
        scroll_coordinate(&eval_expr(expr, env, host)?, "scrollTo")?
    } else {
        0
    };

    host.window_scroll_to(x, y)?;
    Ok(Value::Undefined)
}

fn window_scroll_by<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 2 {
        return Err(ScriptError::new("scrollBy() expects at most two arguments"));
    }

    let x = if let Some(expr) = args.first() {
        scroll_coordinate(&eval_expr(expr, env, host)?, "scrollBy")?
    } else {
        0
    };
    let y = if let Some(expr) = args.get(1) {
        scroll_coordinate(&eval_expr(expr, env, host)?, "scrollBy")?
    } else {
        0
    };

    host.window_scroll_by(x, y)?;
    Ok(Value::Undefined)
}

fn eval_add(left: Value, right: Value) -> Value {
    match (left, right) {
        (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs + rhs),
        (left, right) => Value::String(format!("{}{}", as_string(&left), as_string(&right))),
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Undefined | Value::Null => false,
        Value::Boolean(value) => *value,
        Value::Number(value) => *value != 0.0,
        Value::String(value) => !value.is_empty(),
        Value::Element(_)
        | Value::ClassList(_)
        | Value::Dataset(_)
        | Value::Navigator
        | Value::History
        | Value::HtmlCollection(_)
        | Value::StyleSheetList(_)
        | Value::StyleSheet(_)
        | Value::Node(_)
        | Value::NodeList(_)
        | Value::RadioNodeList(_)
        | Value::Storage(_)
        | Value::MediaQueryList(_)
        | Value::StringList(_)
        | Value::MimeTypeArray(_)
        | Value::TemplateContent(_)
        | Value::Screen
        | Value::CollectionIterator(_)
        | Value::IteratorResult(_)
        | Value::CollectionEntry(_)
        | Value::Document
        | Value::Window
        | Value::Function(_)
        | Value::Event(_)
        | Value::ScreenOrientation(_) => true,
    }
}

fn index_from_value(value: &Value) -> Option<usize> {
    match value {
        Value::Number(number) if number.is_finite() && *number >= 0.0 && number.fract() == 0.0 => {
            Some(*number as usize)
        }
        Value::String(value) => value.parse::<usize>().ok(),
        _ => None,
    }
}

fn scroll_coordinate(value: &Value, method: &str) -> Result<i64> {
    match value {
        Value::Number(number)
            if number.is_finite()
                && number.fract() == 0.0
                && *number >= i64::MIN as f64
                && *number <= i64::MAX as f64 =>
        {
            Ok(*number as i64)
        }
        Value::String(value) => value
            .parse::<i64>()
            .map_err(|_| ScriptError::new(format!("{method}() expects integer coordinates"))),
        _ => Err(ScriptError::new(format!(
            "{method}() expects integer coordinates"
        ))),
    }
}

fn history_delta_from_value(value: &Value) -> Result<i64> {
    match value {
        Value::Number(number)
            if number.is_finite()
                && number.fract() == 0.0
                && *number >= i64::MIN as f64
                && *number <= i64::MAX as f64 =>
        {
            Ok(*number as i64)
        }
        Value::String(value) => value
            .parse::<i64>()
            .map_err(|_| ScriptError::new("history.go() expects an integer delta")),
        _ => Err(ScriptError::new("history.go() expects an integer delta")),
    }
}

fn history_state_from_value(value: &Value) -> Option<String> {
    match value {
        Value::Undefined | Value::Null => None,
        _ => Some(as_string(value)),
    }
}

fn class_list_tokens<H: HostBindings>(
    element: crate::ElementHandle,
    host: &mut H,
) -> Result<Vec<String>> {
    let class_value = host
        .element_get_attribute(element, "class")?
        .unwrap_or_default();
    Ok(normalize_class_list_tokens(
        class_value.split_ascii_whitespace().map(str::to_string),
    ))
}

fn normalize_class_list_tokens<I>(tokens: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut unique = Vec::new();
    for token in tokens {
        if !unique.iter().any(|candidate| candidate == &token) {
            unique.push(token);
        }
    }
    unique
}

fn validate_class_list_token(token: &str) -> Result<String> {
    let trimmed = token.trim();
    if trimmed.is_empty() || trimmed != token || trimmed.chars().any(char::is_whitespace) {
        return Err(ScriptError::new(
            "classList token must be a non-empty string without whitespace",
        ));
    }
    Ok(trimmed.to_string())
}

fn write_class_list_tokens<H: HostBindings>(
    element: crate::ElementHandle,
    tokens: &[String],
    host: &mut H,
) -> Result<()> {
    host.element_set_attribute(element, "class", &tokens.join(" "))
}

fn class_list_contains<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [token_expr] = args else {
        return Err(ScriptError::new(
            "classList.contains() expects exactly one argument",
        ));
    };

    let token = validate_class_list_token(&as_string(&eval_expr(token_expr, env, host)?))?;
    let tokens = class_list_tokens(element, host)?;
    Ok(Value::Boolean(
        tokens.iter().any(|candidate| candidate == &token),
    ))
}

fn class_list_add<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.is_empty() {
        return Err(ScriptError::new(
            "classList.add() expects at least one argument",
        ));
    }

    let mut tokens = class_list_tokens(element, host)?;
    let mut changed = false;
    for expr in args {
        let token = validate_class_list_token(&as_string(&eval_expr(expr, env, host)?))?;
        if !tokens.iter().any(|candidate| candidate == &token) {
            tokens.push(token);
            changed = true;
        }
    }
    if changed {
        write_class_list_tokens(element, &tokens, host)?;
    }
    Ok(Value::Undefined)
}

fn class_list_remove<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.is_empty() {
        return Err(ScriptError::new(
            "classList.remove() expects at least one argument",
        ));
    }

    let mut tokens = class_list_tokens(element, host)?;
    let original_len = tokens.len();
    for expr in args {
        let token = validate_class_list_token(&as_string(&eval_expr(expr, env, host)?))?;
        tokens.retain(|candidate| candidate != &token);
    }
    if tokens.len() != original_len {
        write_class_list_tokens(element, &tokens, host)?;
    }
    Ok(Value::Undefined)
}

fn class_list_toggle<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (token_expr, force_expr) = match args {
        [token_expr] => (token_expr, None),
        [token_expr, force_expr] => (token_expr, Some(force_expr)),
        _ => {
            return Err(ScriptError::new(
                "classList.toggle() expects one or two arguments",
            ));
        }
    };

    let token = validate_class_list_token(&as_string(&eval_expr(token_expr, env, host)?))?;
    let force = match force_expr {
        Some(expr) => Some(is_truthy(&eval_expr(expr, env, host)?)),
        None => None,
    };

    let mut tokens = class_list_tokens(element, host)?;
    let present = tokens.iter().any(|candidate| candidate == &token);
    let now_present = match force {
        Some(true) => {
            if !present {
                tokens.push(token);
                write_class_list_tokens(element, &tokens, host)?;
            }
            true
        }
        Some(false) => {
            if present {
                tokens.retain(|candidate| candidate != &token);
                write_class_list_tokens(element, &tokens, host)?;
            }
            false
        }
        None => {
            if present {
                tokens.retain(|candidate| candidate != &token);
                write_class_list_tokens(element, &tokens, host)?;
                false
            } else {
                tokens.push(token);
                write_class_list_tokens(element, &tokens, host)?;
                true
            }
        }
    };

    Ok(Value::Boolean(now_present))
}

fn dataset_attribute_name(property: &str) -> Result<String> {
    let trimmed = property.trim();
    if trimmed.is_empty() {
        return Err(ScriptError::new("dataset property name must not be empty"));
    }

    let mut attribute = String::from("data-");
    for ch in trimmed.chars() {
        match ch {
            'A'..='Z' => {
                attribute.push('-');
                attribute.push(ch.to_ascii_lowercase());
            }
            'a'..='z' | '0'..='9' | '_' | '$' => attribute.push(ch),
            _ => {
                return Err(ScriptError::new(format!(
                    "unsupported dataset property name: {property}"
                )));
            }
        }
    }

    Ok(attribute)
}

fn unsupported_member_access(property: &str, kind: &str) -> ScriptError {
    ScriptError::new(format!(
        "unsupported member access: `{property}` on {kind} value"
    ))
}
