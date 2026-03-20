use std::collections::BTreeMap;

use crate::syntax::{AssignTarget, Expr, Program, Statement};
use crate::{
    HostBindings, HtmlCollectionScope, HtmlCollectionTarget, ListenerTarget, NodeListTarget,
    Result, ScriptError, ScriptValue as Value,
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
        Value::HtmlCollection(_) => "[object HTMLCollection]".to_string(),
        Value::NodeList(_) => "[object NodeList]".to_string(),
        Value::Document => "[object Document]".to_string(),
        Value::Window => "[object Window]".to_string(),
        Value::Event(_) => "[object Event]".to_string(),
        Value::Function(_) => "[function]".to_string(),
    }
}

fn value_for_listener_target(target: ListenerTarget) -> Value {
    match target {
        ListenerTarget::Window => Value::Window,
        ListenerTarget::Document => Value::Document,
        ListenerTarget::Element(element) => Value::Element(element),
    }
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
            let object = eval_expr(object, env, host)?;
            match (object, property.as_str()) {
                (Value::Element(element), "textContent") => {
                    host.element_set_text_content(element, &as_string(&value))
                }
                (Value::Element(element), "value") => {
                    host.element_set_value(element, &as_string(&value))
                }
                (Value::Element(element), "checked") => {
                    host.element_set_checked(element, is_truthy(&value))
                }
                (Value::Element(_), _) => Err(ScriptError::new(format!(
                    "unsupported assignment target on element: {property}"
                ))),
                (Value::NodeList(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on node list value"
                ))),
                (Value::HtmlCollection(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on html collection value"
                ))),
                (Value::Document, "title") => Err(ScriptError::phase_not_ready("document.title")),
                (Value::Window, "title") => Err(ScriptError::phase_not_ready("window.title")),
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
    let object = eval_expr(object, env, host)?;
    match object {
        Value::Document if property == "forms" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "form".to_string(),
            }))
        }
        Value::Document if property == "images" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "img".to_string(),
            }))
        }
        Value::Document if property == "links" => {
            Ok(Value::HtmlCollection(HtmlCollectionTarget::DocumentLinks))
        }
        Value::Window if property == "document" => Ok(Value::Document),
        Value::Document if property == "defaultView" => Ok(Value::Window),
        Value::Element(element) if property == "textContent" => {
            Ok(Value::String(host.element_text_content(element)?))
        }
        Value::Element(element) if property == "value" => {
            Ok(Value::String(host.element_value(element)?))
        }
        Value::Element(element) if property == "checked" => {
            Ok(Value::Boolean(host.element_checked(element)?))
        }
        Value::Element(element) if property == "children" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::Children(element),
        )),
        Value::Element(element) if property == "elements" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::FormElements(element),
        )),
        Value::Element(element) if property == "options" => Ok(Value::HtmlCollection(
            HtmlCollectionTarget::SelectOptions(element),
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
        Value::HtmlCollection(collection) if property == "length" => {
            let length = html_collection_items(&collection, host)?.len();
            Ok(Value::Number(length as f64))
        }
        Value::NodeList(target) if property == "length" => {
            let length = node_list_items(&target, host)?.len();
            Ok(Value::Number(length as f64))
        }
        Value::Element(_) => Err(unsupported_member_access(property, "element")),
        Value::Document => Err(unsupported_member_access(property, "document")),
        Value::Window => Err(unsupported_member_access(property, "window")),
        Value::String(_) => Err(unsupported_member_access(property, "string")),
        Value::Number(_) => Err(unsupported_member_access(property, "number")),
        Value::Boolean(_) => Err(unsupported_member_access(property, "boolean")),
        Value::Null | Value::Undefined => Err(unsupported_member_access(property, "nullish")),
        Value::Event(_) => Err(unsupported_member_access(property, "event")),
        Value::HtmlCollection(_) => Err(unsupported_member_access(property, "html collection")),
        Value::NodeList(_) => Err(unsupported_member_access(property, "node list")),
        Value::Function(_) => Err(unsupported_member_access(property, "function")),
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
            let object_value = eval_expr(object, env, host)?;
            eval_method_call(object_value, property, args, env, host)
        }
        Expr::ArrowFunction(_) => Err(ScriptError::new("arrow functions are not callable")),
        Expr::String(_) | Expr::Number(_) | Expr::Boolean(_) | Expr::Null | Expr::Undefined => {
            Err(ScriptError::new("invalid call target"))
        }
        Expr::Call { .. } | Expr::BinaryAdd { .. } => {
            Err(ScriptError::new("invalid nested call target"))
        }
    }
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
            "querySelector" => query_selector(QuerySelectorTarget::Document, args, env, host),
            "querySelectorAll" => {
                query_selector_all(QuerySelectorTarget::Document, args, env, host)
            }
            "getElementsByTagName" => {
                get_elements_by_tag_name(HtmlCollectionScope::Document, args, env, host)
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
            "addEventListener" => register_listener(ListenerTarget::Window, args, env, host),
            "document" => Ok(Value::Document),
            other => Err(ScriptError::new(format!(
                "unsupported Window method: {other}"
            ))),
        },
        Value::Element(element) => match method {
            "querySelector" => {
                query_selector(QuerySelectorTarget::Element(element), args, env, host)
            }
            "querySelectorAll" => {
                query_selector_all(QuerySelectorTarget::Element(element), args, env, host)
            }
            "getElementsByTagName" => {
                get_elements_by_tag_name(HtmlCollectionScope::Element(element), args, env, host)
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
        Value::HtmlCollection(collection) => match method {
            "item" => html_collection_item(&collection, args, env, host),
            "namedItem" => html_collection_named_item(&collection, args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported HTMLCollection method: {other}"
            ))),
        },
        Value::NodeList(target) => match method {
            "item" => node_list_item(&target, args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported NodeList method: {other}"
            ))),
        },
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
    };

    Ok(Value::NodeList(NodeListTarget::Snapshot(matches)))
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
    let match_handle = html_collection_named_item_handle(collection, &name, host)?;

    Ok(match_handle.map(Value::Element).unwrap_or(Value::Null))
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
        HtmlCollectionTarget::ByClassName { .. } => {
            host.html_collection_class_name_items(collection.clone())
        }
        HtmlCollectionTarget::FormElements(element) => {
            host.html_collection_form_elements_items(*element)
        }
        HtmlCollectionTarget::SelectOptions(element) => {
            host.html_collection_select_options_items(*element)
        }
        HtmlCollectionTarget::DocumentLinks => host.html_collection_document_links_items(),
    }
}

fn html_collection_named_item_handle<H: HostBindings>(
    collection: &HtmlCollectionTarget,
    name: &str,
    host: &mut H,
) -> Result<Option<crate::ElementHandle>> {
    match collection {
        HtmlCollectionTarget::Children(element) => host.html_collection_named_item(*element, name),
        HtmlCollectionTarget::ByTagName { .. } => {
            host.html_collection_tag_name_named_item(collection.clone(), name)
        }
        HtmlCollectionTarget::ByClassName { .. } => {
            host.html_collection_class_name_named_item(collection.clone(), name)
        }
        HtmlCollectionTarget::FormElements(element) => {
            host.html_collection_form_elements_named_item(*element, name)
        }
        HtmlCollectionTarget::SelectOptions(element) => {
            host.html_collection_select_options_named_item(*element, name)
        }
        HtmlCollectionTarget::DocumentLinks => host.html_collection_document_links_named_item(name),
    }
}

fn node_list_items<H: HostBindings>(
    target: &NodeListTarget,
    host: &mut H,
) -> Result<Vec<crate::ElementHandle>> {
    match target {
        NodeListTarget::Snapshot(nodes) => Ok(nodes.clone()),
        NodeListTarget::ByName(name) => host.document_get_elements_by_name(name),
    }
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
        | Value::HtmlCollection(_)
        | Value::NodeList(_)
        | Value::Document
        | Value::Window
        | Value::Function(_)
        | Value::Event(_) => true,
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

fn unsupported_member_access(property: &str, kind: &str) -> ScriptError {
    ScriptError::new(format!(
        "unsupported member access: `{property}` on {kind} value"
    ))
}
