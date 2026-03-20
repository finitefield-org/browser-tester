use std::collections::BTreeMap;

use crate::syntax::{AssignTarget, Expr, Program, Statement};
use crate::{
    CollectionIteratorHandle, HostBindings, HtmlCollectionScope, HtmlCollectionTarget,
    ListenerTarget, NodeListTarget, Result, ScriptError, ScriptValue as Value,
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
        Value::HtmlCollection(_) => "[object HTMLCollection]".to_string(),
        Value::NodeList(_) => "[object NodeList]".to_string(),
        Value::CollectionIterator(_) => "[object Iterator]".to_string(),
        Value::IteratorResult(_) => "[object IteratorResult]".to_string(),
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
                (Value::Element(element), "innerHTML") => {
                    host.element_set_inner_html(element, &as_string(&value))
                }
                (Value::Element(element), "outerHTML") => {
                    host.element_set_outer_html(element, &as_string(&value))
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
                (Value::Dataset(element), property) => {
                    let attribute_name = dataset_attribute_name(property)?;
                    host.element_set_attribute(element, &attribute_name, &as_string(&value))
                }
                (Value::Element(_), _) => Err(ScriptError::new(format!(
                    "unsupported assignment target on element: {property}"
                ))),
                (Value::ClassList(_), property) => Err(ScriptError::new(format!(
                    "unsupported assignment target on class list value: {property}"
                ))),
                (Value::NodeList(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on node list value"
                ))),
                (Value::HtmlCollection(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on html collection value"
                ))),
                (Value::CollectionIterator(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on iterator value"
                ))),
                (Value::IteratorResult(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on iterator result value"
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
        Value::Window if property == "document" => Ok(Value::Document),
        Value::Document if property == "defaultView" => Ok(Value::Window),
        Value::Element(element) if property == "textContent" => {
            Ok(Value::String(host.element_text_content(element)?))
        }
        Value::Element(element) if property == "innerHTML" => {
            Ok(Value::String(host.element_inner_html(element)?))
        }
        Value::Element(element) if property == "outerHTML" => {
            Ok(Value::String(host.element_outer_html(element)?))
        }
        Value::Element(element) if property == "value" => {
            Ok(Value::String(host.element_value(element)?))
        }
        Value::Element(element) if property == "checked" => {
            Ok(Value::Boolean(host.element_checked(element)?))
        }
        Value::Element(element) if property == "className" => Ok(Value::String(
            host.element_get_attribute(element, "class")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "classList" => Ok(Value::ClassList(element)),
        Value::Element(element) if property == "dataset" => Ok(Value::Dataset(element)),
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
        Value::IteratorResult(result) if property == "value" => {
            Ok(result.value().unwrap_or(Value::Undefined))
        }
        Value::IteratorResult(result) if property == "done" => Ok(Value::Boolean(result.done())),
        Value::ClassList(element) if property == "length" => {
            let length = class_list_tokens(element, host)?.len();
            Ok(Value::Number(length as f64))
        }
        Value::NodeList(target) if property == "length" => {
            let length = node_list_items(&target, host)?.len();
            Ok(Value::Number(length as f64))
        }
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
        Value::Document => Err(unsupported_member_access(property, "document")),
        Value::Window => Err(unsupported_member_access(property, "window")),
        Value::String(_) => Err(unsupported_member_access(property, "string")),
        Value::Number(_) => Err(unsupported_member_access(property, "number")),
        Value::Boolean(_) => Err(unsupported_member_access(property, "boolean")),
        Value::Null | Value::Undefined => Err(unsupported_member_access(property, "nullish")),
        Value::Event(_) => Err(unsupported_member_access(property, "event")),
        Value::HtmlCollection(_) => Err(unsupported_member_access(property, "html collection")),
        Value::NodeList(_) => Err(unsupported_member_access(property, "node list")),
        Value::CollectionIterator(_) => Err(unsupported_member_access(property, "iterator")),
        Value::IteratorResult(_) => Err(unsupported_member_access(property, "iterator result")),
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
            "appendChild" => element_append_child(element, args, env, host),
            "insertBefore" => element_insert_before(element, args, env, host),
            "replaceChild" => element_replace_child(element, args, env, host),
            "replaceChildren" => element_replace_children(element, args, env, host),
            "append" => element_append(element, args, env, host),
            "prepend" => element_prepend(element, args, env, host),
            "before" => element_before(element, args, env, host),
            "after" => element_after(element, args, env, host),
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
            "forEach" => html_collection_for_each(&collection, args, env, host),
            "keys" => html_collection_keys(&collection, host),
            "values" => html_collection_values(&collection, host),
            other => Err(ScriptError::new(format!(
                "unsupported HTMLCollection method: {other}"
            ))),
        },
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
            other => Err(ScriptError::new(format!(
                "unsupported NodeList method: {other}"
            ))),
        },
        Value::CollectionIterator(iterator) => match method {
            "next" => collection_iterator_next(&iterator),
            other => Err(ScriptError::new(format!(
                "unsupported iterator method: {other}"
            ))),
        },
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

    let child = eval_element_handle(child_expr, env, host, "appendChild")?;
    host.element_append_child(element, child)?;
    Ok(Value::Element(child))
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

    let child = eval_element_handle(child_expr, env, host, "insertBefore")?;
    let reference = eval_optional_element_handle(reference_expr, env, host, "insertBefore")?;
    host.element_insert_before(element, child, reference)?;
    Ok(Value::Element(child))
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

    let new_child = eval_element_handle(new_child_expr, env, host, "replaceChild")?;
    let old_child = eval_element_handle(old_child_expr, env, host, "replaceChild")?;
    host.element_replace_child(element, new_child, old_child)?;
    Ok(Value::Element(old_child))
}

fn element_replace_children<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_element_arguments(args, env, host, "replaceChildren")?;
    host.element_replace_children(element, children)?;
    Ok(Value::Undefined)
}

fn element_append<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_element_arguments(args, env, host, "append")?;
    host.element_append(element, children)?;
    Ok(Value::Undefined)
}

fn element_prepend<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_element_arguments(args, env, host, "prepend")?;
    host.element_prepend(element, children)?;
    Ok(Value::Undefined)
}

fn element_before<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_element_arguments(args, env, host, "before")?;
    host.element_before(element, children)?;
    Ok(Value::Undefined)
}

fn element_after<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let children = eval_element_arguments(args, env, host, "after")?;
    host.element_after(element, children)?;
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

fn eval_element_handle<H: HostBindings>(
    expr: &Expr,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<crate::ElementHandle> {
    match eval_expr(expr, env, host)? {
        Value::Element(element) => Ok(element),
        _ => Err(ScriptError::new(format!(
            "{method}() expects element arguments"
        ))),
    }
}

fn eval_optional_element_handle<H: HostBindings>(
    expr: &Expr,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<Option<crate::ElementHandle>> {
    let value = eval_expr(expr, env, host)?;
    match value {
        Value::Element(element) => Ok(Some(element)),
        Value::Null | Value::Undefined => Ok(None),
        _ => Err(ScriptError::new(format!(
            "{method}() expects an element or null reference"
        ))),
    }
}

fn eval_element_arguments<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<Vec<crate::ElementHandle>> {
    let mut children = Vec::new();
    for expr in args {
        children.push(eval_element_handle(expr, env, host, method)?);
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

    let items = html_collection_items(collection, host)?;
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
        HtmlCollectionTarget::DocumentLinks => host.html_collection_document_links_items(),
        HtmlCollectionTarget::DocumentAnchors => host.html_collection_document_anchors_items(),
        HtmlCollectionTarget::DocumentChildren => host.html_collection_document_children_items(),
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
        HtmlCollectionTarget::ByTagNameNs { .. } => {
            host.html_collection_tag_name_ns_named_item(collection.clone(), name)
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
        HtmlCollectionTarget::DocumentAnchors => {
            host.html_collection_document_anchors_named_item(name)
        }
        HtmlCollectionTarget::DocumentChildren => {
            host.html_collection_document_children_named_item(name)
        }
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

    let items = node_list_items(target, host)?;
    let collection_value = Value::NodeList(target.clone());
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
        items.into_iter().map(Value::Element).collect(),
    ))
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

fn for_each_over_items<H: HostBindings>(
    callback: &crate::ScriptFunction,
    items: Vec<crate::ElementHandle>,
    collection_value: Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let program = crate::parser::parse_program(&callback.body_source)?;

    for (index, item) in items.into_iter().enumerate() {
        let mut bindings = env.clone();
        for (param_index, param) in callback.params.iter().enumerate() {
            let value = match param_index {
                0 => Value::Element(item),
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
        | Value::HtmlCollection(_)
        | Value::NodeList(_)
        | Value::CollectionIterator(_)
        | Value::IteratorResult(_)
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
