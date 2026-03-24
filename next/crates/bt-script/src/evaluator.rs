use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{
    DateTime, Datelike, Duration, Local, LocalResult, NaiveDate, Offset, TimeZone, Timelike, Utc,
};
use chrono_tz::Tz;
use fancy_regex::{Regex as FancyRegex, RegexBuilder};
use unicode_normalization::UnicodeNormalization;
use url::Url;

use crate::syntax::{
    ArrayElement, AssignTarget, AssignmentOperator, ComparisonOperator, Expr, ObjectProperty,
    ObjectPropertyName, Program, Statement,
};
use crate::{
    ArrayHandle, AttributeHandle, CollectionEntryHandle, CollectionIteratorHandle, DateValue,
    ElementHandle, HostBindings, HtmlCollectionNamedItem, HtmlCollectionScope,
    HtmlCollectionTarget, IntlCollatorValue, IntlDateTimeFormatValue, IntlNumberFormatValue,
    ListenerTarget, MapHandle, MapKey, MediaQueryListState, MimeTypeArrayState, NodeHandle,
    NodeListTarget, PropertyKey, PropertyValue, RadioNodeListTarget, Result, ScriptError,
    ScriptValue as Value, StorageTarget, StringListState, StyleSheetListTarget, StyleSheetTarget,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum EvalControl {
    Continue,
    Return(Value),
    Break,
    ContinueLoop,
}

pub(crate) fn eval_program<H: HostBindings>(program: &Program, host: &mut H) -> Result<()> {
    let mut env = BTreeMap::new();
    match eval_program_with_bindings(program, host, &mut env)? {
        EvalControl::Continue => Ok(()),
        EvalControl::Return(_) => Err(ScriptError::new("return outside function")),
        EvalControl::Break => Err(ScriptError::new("break outside loop")),
        EvalControl::ContinueLoop => Err(ScriptError::new("continue outside loop")),
    }
}

pub(crate) fn eval_program_with_bindings<H: HostBindings>(
    program: &Program,
    host: &mut H,
    env: &mut BTreeMap<String, Value>,
) -> Result<EvalControl> {
    for statement in &program.statements {
        match eval_statement(statement, env, host)? {
            EvalControl::Continue => {}
            other => return Ok(other),
        }
    }

    Ok(EvalControl::Continue)
}

fn as_string(value: &Value) -> String {
    match value {
        Value::Undefined => "undefined".to_string(),
        Value::Null => "null".to_string(),
        Value::Boolean(value) => value.to_string(),
        Value::Number(value) => {
            if value.is_nan() {
                "NaN".to_string()
            } else if value.is_infinite() {
                if value.is_sign_negative() {
                    "-Infinity".to_string()
                } else {
                    "Infinity".to_string()
                }
            } else if value.fract() == 0.0 {
                (*value as i64).to_string()
            } else {
                value.to_string()
            }
        }
        Value::String(value) => value.clone(),
        Value::Object(_) | Value::ObjectNamespace => "[object Object]".to_string(),
        Value::Array(array) => array_to_string(array, ",", None),
        Value::Map(_) => "[object Map]".to_string(),
        Value::Symbol(symbol) => match symbol.description() {
            Some(description) => format!("Symbol({description})"),
            None => "Symbol()".to_string(),
        },
        Value::RegExp(value) => {
            let mut out = String::from("/");
            out.push_str(value.pattern());
            out.push('/');
            out.push_str(value.flags());
            out
        }
        Value::Date(value) => match value.epoch_ms {
            Some(epoch_ms) => {
                date_to_iso_string(epoch_ms).unwrap_or_else(|| "Invalid Date".to_string())
            }
            None => "Invalid Date".to_string(),
        },
        Value::ArrayNamespace => "[function Array]".to_string(),
        Value::IntlNamespace => "[object Intl]".to_string(),
        Value::IntlNumberFormat(_) => "[object Intl.NumberFormat]".to_string(),
        Value::IntlDateTimeFormat(_) => "[object Intl.DateTimeFormat]".to_string(),
        Value::IntlCollator(_) => "[object Intl.Collator]".to_string(),
        Value::Element(_) => "[object Element]".to_string(),
        Value::Attribute(_) => "[object Attr]".to_string(),
        Value::ClassList(_) => "[object DOMTokenList]".to_string(),
        Value::Dataset(_) => "[object DOMStringMap]".to_string(),
        Value::TemplateContent(_) => "[object DocumentFragment]".to_string(),
        Value::NamedNodeMap(_) => "[object NamedNodeMap]".to_string(),
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
        Value::Clipboard => "[object Clipboard]".to_string(),
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

fn error_object_from_message<H: HostBindings>(
    message: impl Into<String>,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let object = crate::ObjectHandle::new();
    let message = message.into();
    object_set_property_value(
        &object,
        property_key_from_string("message"),
        Value::String(message.clone()),
        env,
        host,
    )?;
    object_set_property_value(
        &object,
        property_key_from_string("name"),
        Value::String("Error".to_string()),
        env,
        host,
    )?;
    Ok(Value::Object(object))
}

fn exception_message_from_value<H: HostBindings>(
    value: &Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<String> {
    if let Value::Object(_) = value
        && let Some(message) =
            property_value_on_value(value, &property_key_from_string("message"), env, host)?
    {
        return Ok(as_string(&message));
    }

    Ok(as_string(value))
}

fn content_editable_reflection(value: Option<&str>) -> &'static str {
    match value.map(|value| value.trim().to_ascii_lowercase()) {
        Some(value) if value.is_empty() || value == "true" => "true",
        Some(value) if value == "false" => "false",
        Some(value) if value == "plaintext-only" => "plaintext-only",
        _ => "inherit",
    }
}

fn input_type_reflection(value: Option<&str>) -> String {
    let value = value.map(|value| value.trim().to_ascii_lowercase());
    match value.as_deref() {
        Some("hidden")
        | Some("text")
        | Some("search")
        | Some("tel")
        | Some("url")
        | Some("email")
        | Some("password")
        | Some("date")
        | Some("month")
        | Some("week")
        | Some("time")
        | Some("datetime-local")
        | Some("number")
        | Some("range")
        | Some("color")
        | Some("checkbox")
        | Some("radio")
        | Some("file")
        | Some("submit")
        | Some("image")
        | Some("reset")
        | Some("button") => value.unwrap(),
        _ => "text".to_string(),
    }
}

fn button_type_reflection(value: Option<&str>) -> String {
    match value.map(|value| value.trim().to_ascii_lowercase()) {
        Some(value) if matches!(value.as_str(), "submit" | "reset" | "button") => value,
        _ => "submit".to_string(),
    }
}

fn form_method_reflection(value: Option<&str>) -> String {
    match value.map(|value| value.trim().to_ascii_lowercase()) {
        Some(value) if matches!(value.as_str(), "get" | "post" | "dialog") => value,
        _ => "get".to_string(),
    }
}

fn form_enctype_reflection(value: Option<&str>) -> String {
    match value.map(|value| value.trim().to_ascii_lowercase()) {
        Some(value)
            if matches!(
                value.as_str(),
                "application/x-www-form-urlencoded" | "multipart/form-data" | "text/plain"
            ) =>
        {
            value
        }
        _ => "application/x-www-form-urlencoded".to_string(),
    }
}

fn form_target_reflection(value: Option<&str>) -> String {
    value.unwrap_or_default().to_string()
}

fn textarea_wrap_reflection(value: Option<&str>) -> String {
    match value.map(|value| value.trim().to_ascii_lowercase()) {
        Some(value) if matches!(value.as_str(), "soft" | "hard" | "off") => value,
        _ => "soft".to_string(),
    }
}

fn resolve_url_reflection(value: Option<&str>, base_uri: &str) -> String {
    let Some(value) = value.map(str::trim) else {
        return base_uri.to_string();
    };
    if value.is_empty() {
        return base_uri.to_string();
    }

    if let Ok(url) = Url::parse(value) {
        return url.to_string();
    }

    Url::parse(base_uri)
        .and_then(|base| base.join(value))
        .map(|url| url.to_string())
        .unwrap_or_else(|_| value.to_string())
}

fn associated_form_for_submit_control<H: HostBindings>(
    element: ElementHandle,
    host: &mut H,
) -> Result<Option<ElementHandle>> {
    match host.element_tag_name(element)?.as_str() {
        "input" | "button" => {}
        _ => return Err(unsupported_member_access("formAction", "element")),
    }

    let mut current = NodeHandle::new(element.raw());
    while let Some(parent) = host.node_parent(current)? {
        if host.node_type(parent)? == 1 {
            let parent_element = ElementHandle::new(parent.raw());
            if host.element_tag_name(parent_element)? == "form" {
                return Ok(Some(parent_element));
            }
        }

        current = parent;
    }

    Ok(None)
}

fn form_action_reflection<H: HostBindings>(element: ElementHandle, host: &mut H) -> Result<String> {
    let base_uri = host.document_base_uri()?;
    Ok(resolve_url_reflection(
        host.element_get_attribute(element, "action")?.as_deref(),
        &base_uri,
    ))
}

fn form_action_override_reflection<H: HostBindings>(
    element: ElementHandle,
    host: &mut H,
) -> Result<String> {
    let base_uri = host.document_base_uri()?;
    let override_value = host.element_get_attribute(element, "formaction")?;
    if override_value.is_some() {
        return Ok(resolve_url_reflection(override_value.as_deref(), &base_uri));
    }

    if let Some(form) = associated_form_for_submit_control(element, host)? {
        return Ok(resolve_url_reflection(
            host.element_get_attribute(form, "action")?.as_deref(),
            &base_uri,
        ));
    }

    Ok(base_uri)
}

fn translate_reflection<H: HostBindings>(element: ElementHandle, host: &mut H) -> Result<bool> {
    let mut current = Some(NodeHandle::new(element.raw()));

    while let Some(node) = current {
        if host.node_type(node)? == 1 {
            let current_element = ElementHandle::new(node.raw());
            if let Some(value) = host.element_get_attribute(current_element, "translate")? {
                return Ok(value.trim().to_ascii_lowercase() != "no");
            }
        }

        current = host.node_parent(node)?;
    }

    Ok(true)
}

fn spellcheck_reflection<H: HostBindings>(element: ElementHandle, host: &mut H) -> Result<bool> {
    let mut current = Some(NodeHandle::new(element.raw()));

    while let Some(node) = current {
        if host.node_type(node)? == 1 {
            let current_element = ElementHandle::new(node.raw());
            if let Some(value) = host.element_get_attribute(current_element, "spellcheck")? {
                let value = value.trim().to_ascii_lowercase();
                if value == "false" {
                    return Ok(false);
                }
                if value == "true" || value.is_empty() {
                    return Ok(true);
                }

                return Ok(true);
            }
        }

        current = host.node_parent(node)?;
    }

    Ok(true)
}

fn element_is_natively_focusable<H: HostBindings>(
    element: ElementHandle,
    host: &mut H,
) -> Result<bool> {
    if host.element_get_attribute(element, "disabled")?.is_some() {
        return Ok(false);
    }

    if host
        .element_get_attribute(element, "contenteditable")?
        .is_some()
    {
        return Ok(true);
    }

    match host.element_tag_name(element)?.as_str() {
        "button" | "select" | "textarea" | "summary" | "iframe" | "object" | "embed" => Ok(true),
        "input" => Ok(!matches!(
            host.element_get_attribute(element, "type")?
                .as_deref()
                .map(|value| value.trim().to_ascii_lowercase()),
            Some(value) if value == "hidden"
        )),
        "a" | "area" => Ok(host.element_get_attribute(element, "href")?.is_some()),
        _ => Ok(false),
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

fn selected_index_from_value(value: &Value) -> Result<i64> {
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
            .map_err(|_| ScriptError::new("selectedIndex expects an integer")),
        _ => Err(ScriptError::new("selectedIndex expects an integer")),
    }
}

fn integer_from_value(value: &Value, property: &str) -> Result<i64> {
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
            .map_err(|_| ScriptError::new(format!("{property} expects an integer"))),
        _ => Err(ScriptError::new(format!("{property} expects an integer"))),
    }
}

fn option_index_for_element<H: HostBindings>(option: ElementHandle, host: &mut H) -> Result<i64> {
    if host.element_tag_name(option)? != "option" {
        return Err(unsupported_member_access("index", "element"));
    }

    let mut current = NodeHandle::new(option.raw());
    let select = loop {
        let Some(parent) = host.node_parent(current)? else {
            return Ok(-1);
        };

        if host.node_type(parent)? == 1 {
            let parent_element = ElementHandle::new(parent.raw());
            if host.element_tag_name(parent_element)? == "select" {
                break parent_element;
            }
        }

        current = parent;
    };

    let options = host.html_collection_select_options_items(select)?;
    Ok(options
        .iter()
        .position(|candidate| candidate.raw() == option.raw())
        .map(|index| index as i64)
        .unwrap_or(-1))
}

fn form_owner_for_element<H: HostBindings>(element: ElementHandle, host: &mut H) -> Result<Value> {
    match host.element_tag_name(element)?.as_str() {
        "input" | "button" | "select" | "textarea" | "option" | "fieldset" | "output"
        | "object" | "embed" => {}
        _ => return Err(unsupported_member_access("form", "element")),
    }

    let mut current = NodeHandle::new(element.raw());
    while let Some(parent) = host.node_parent(current)? {
        if host.node_type(parent)? == 1 {
            let parent_element = ElementHandle::new(parent.raw());
            if host.element_tag_name(parent_element)? == "form" {
                return Ok(Value::Element(parent_element));
            }
        }

        current = parent;
    }

    Ok(Value::Null)
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

fn document_write<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let mut html = String::new();
    for expr in args {
        html.push_str(&as_string(&eval_expr(expr, env, host)?));
    }

    host.document_write(&html)?;
    Ok(Value::Undefined)
}

fn document_writeln<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let mut html = String::new();
    for expr in args {
        html.push_str(&as_string(&eval_expr(expr, env, host)?));
    }

    html.push('\n');
    host.document_writeln(&html)?;
    Ok(Value::Undefined)
}

fn document_open<H: HostBindings>(
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("document.open() expects no arguments"));
    }

    host.document_open()?;
    Ok(Value::Document)
}

fn document_close<H: HostBindings>(
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("document.close() expects no arguments"));
    }

    host.document_close()?;
    Ok(Value::Document)
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

fn node_remove_child<H: HostBindings>(
    parent: NodeHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [child_expr] = args else {
        return Err(ScriptError::new(
            "removeChild() expects exactly one argument",
        ));
    };

    let child = eval_node_handle(child_expr, env, host, "removeChild")?;
    if host.node_parent(child)? != Some(parent) {
        return Err(ScriptError::new(
            "removeChild() expects the child to belong to the parent",
        ));
    }

    host.node_replace_with(child, Vec::new())?;
    value_for_node_handle(child, host)
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
) -> Result<EvalControl> {
    match statement {
        Statement::VariableDeclaration { name, value } => {
            let value = eval_expr(value, env, host)?;
            env.insert(name.clone(), value);
            Ok(EvalControl::Continue)
        }
        Statement::FunctionDeclaration { name, function } => {
            let function = function.clone().with_captured_bindings(env.clone());
            env.insert(name.clone(), Value::Function(function));
            Ok(EvalControl::Continue)
        }
        Statement::Return(value) => Ok(EvalControl::Return(match value {
            Some(expr) => eval_expr(expr, env, host)?,
            None => Value::Undefined,
        })),
        Statement::Break => Ok(EvalControl::Break),
        Statement::Continue => Ok(EvalControl::ContinueLoop),
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            if is_truthy(&eval_expr(condition, env, host)?) {
                eval_statements(then_branch, env, host)
            } else if let Some(else_branch) = else_branch {
                eval_statements(else_branch, env, host)
            } else {
                Ok(EvalControl::Continue)
            }
        }
        Statement::While { condition, body } => {
            loop {
                if !is_truthy(&eval_expr(condition, env, host)?) {
                    break;
                }

                match eval_statements(body, env, host)? {
                    EvalControl::Continue | EvalControl::ContinueLoop => {}
                    EvalControl::Break => break,
                    EvalControl::Return(value) => return Ok(EvalControl::Return(value)),
                }
            }

            Ok(EvalControl::Continue)
        }
        Statement::For {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(init) = init {
                match eval_statement(init, env, host)? {
                    EvalControl::Continue => {}
                    EvalControl::Return(value) => return Ok(EvalControl::Return(value)),
                    EvalControl::Break => return Err(ScriptError::new("break outside loop")),
                    EvalControl::ContinueLoop => {
                        return Err(ScriptError::new("continue outside loop"));
                    }
                }
            }

            loop {
                if let Some(condition) = condition {
                    if !is_truthy(&eval_expr(condition, env, host)?) {
                        break;
                    }
                }

                match eval_statements(body, env, host)? {
                    EvalControl::Continue | EvalControl::ContinueLoop => {}
                    EvalControl::Break => break,
                    EvalControl::Return(value) => return Ok(EvalControl::Return(value)),
                }

                if let Some(update) = update {
                    let _ = eval_expr(update, env, host)?;
                }
            }

            Ok(EvalControl::Continue)
        }
        Statement::ForIn {
            binding,
            iterable,
            body,
        } => {
            let source = eval_expr(iterable, env, host)?;
            let keys = match source {
                Value::Object(object) => object_own_string_keys(&object),
                Value::Array(array) => array_own_property_values(&array, env, host)?
                    .into_iter()
                    .filter_map(|(key, _)| property_key_to_string(&key).map(str::to_string))
                    .collect(),
                _ => return Err(ScriptError::new("for-in expects an object")),
            };

            for key in keys {
                env.insert(binding.clone(), Value::String(key));
                match eval_statements(body, env, host)? {
                    EvalControl::Continue | EvalControl::ContinueLoop => {}
                    EvalControl::Break => break,
                    EvalControl::Return(value) => return Ok(EvalControl::Return(value)),
                }
            }

            Ok(EvalControl::Continue)
        }
        Statement::ForOf {
            binding,
            iterable,
            body,
        } => {
            let source = eval_expr(iterable, env, host)?;
            let items = array_from_value(source, env, host)?;

            for item in items {
                env.insert(binding.clone(), item);
                match eval_statements(body, env, host)? {
                    EvalControl::Continue | EvalControl::ContinueLoop => {}
                    EvalControl::Break => break,
                    EvalControl::Return(value) => return Ok(EvalControl::Return(value)),
                }
            }

            Ok(EvalControl::Continue)
        }
        Statement::TryCatch {
            try_body,
            catch_binding,
            catch_body,
        } => match eval_statements(try_body, env, host) {
            Ok(control) => Ok(control),
            Err(error) => {
                let catch_value = error_object_from_message(error.message(), env, host)?;
                let previous = env.insert(catch_binding.clone(), catch_value);
                let result = eval_statements(catch_body, env, host);
                match previous {
                    Some(value) => {
                        env.insert(catch_binding.clone(), value);
                    }
                    None => {
                        env.remove(catch_binding);
                    }
                }
                result
            }
        },
        Statement::Throw(expr) => {
            let value = eval_expr(expr, env, host)?;
            let message = exception_message_from_value(&value, env, host)?;
            Err(ScriptError::new(message))
        }
        Statement::Assignment { target, value } => {
            let value = eval_expr(value, env, host)?;
            eval_assign(target, value, env, host)?;
            Ok(EvalControl::Continue)
        }
        Statement::Expression(expr) => {
            let _ = eval_expr(expr, env, host)?;
            Ok(EvalControl::Continue)
        }
    }
}

fn eval_statements<H: HostBindings>(
    statements: &[Statement],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<EvalControl> {
    for statement in statements {
        match eval_statement(statement, env, host)? {
            EvalControl::Continue => {}
            other => return Ok(other),
        }
    }

    Ok(EvalControl::Continue)
}

fn eval_assignment_target_value<H: HostBindings>(
    target: &AssignTarget,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    match target {
        AssignTarget::Identifier(name) => Ok(env.get(name).cloned().unwrap_or(Value::Undefined)),
        AssignTarget::Property { object, property } => eval_expr(
            &Expr::Member {
                object: object.clone(),
                property: property.clone(),
            },
            env,
            host,
        ),
        AssignTarget::ComputedProperty { object, property } => eval_expr(
            &Expr::ComputedMember {
                object: object.clone(),
                property: property.clone(),
            },
            env,
            host,
        ),
    }
}

fn eval_assign<H: HostBindings>(
    target: &AssignTarget,
    value: Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<()> {
    match target {
        AssignTarget::Identifier(name) => {
            env.insert(name.clone(), value);
            Ok(())
        }
        AssignTarget::ComputedProperty { object, property } => {
            let object = eval_expr(object, env, host)?;
            let property = eval_expr(property, env, host)?;
            let key = property_key_from_expr_value(&property);
            if set_property_value_on_value(&object, key.clone(), value.clone(), env, host)? {
                return Ok(());
            }

            match object {
                Value::Window => {
                    if let Some(name) = property_key_to_string(&key) {
                        env.insert(name.to_string(), value);
                        Ok(())
                    } else {
                        Err(ScriptError::new(
                            "window computed property assignment expects a string key",
                        ))
                    }
                }
                _ => Err(ScriptError::new("unsupported computed assignment target")),
            }
        }
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
                (Value::Element(element), "defaultValue") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" => host.element_set_attribute(element, "value", &as_string(&value)),
                        "textarea" => host.element_set_text_content(element, &as_string(&value)),
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "checked") => {
                    host.element_set_checked(element, is_truthy(&value))
                }
                (Value::Element(element), "defaultChecked") => {
                    if host.element_tag_name(element)? != "input" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    match host.element_get_attribute(element, "type")?.as_deref() {
                        Some("checkbox") | Some("radio") => {
                            host.element_set_checked(element, is_truthy(&value))
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "indeterminate") => {
                    if host.element_tag_name(element)? != "input" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    match host.element_get_attribute(element, "type")?.as_deref() {
                        Some("checkbox") => {
                            host.element_set_indeterminate(element, is_truthy(&value))
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "selected") => {
                    if host.element_tag_name(element)? != "option" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    if is_truthy(&value) {
                        host.element_set_attribute(element, "selected", "")
                    } else {
                        host.element_remove_attribute(element, "selected")?;
                        Ok(())
                    }
                }
                (Value::Element(element), "defaultSelected") => {
                    if host.element_tag_name(element)? != "option" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    if is_truthy(&value) {
                        host.element_set_attribute(element, "selected", "")
                    } else {
                        host.element_remove_attribute(element, "selected")?;
                        Ok(())
                    }
                }
                (Value::Element(element), "disabled") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "textarea" | "button" | "select" | "option" | "optgroup"
                        | "fieldset" => {
                            if is_truthy(&value) {
                                host.element_set_attribute(element, "disabled", "")
                            } else {
                                host.element_remove_attribute(element, "disabled")?;
                                Ok(())
                            }
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "label") => {
                    match host.element_tag_name(element)?.as_str() {
                        "option" | "optgroup" => {
                            host.element_set_attribute(element, "label", &as_string(&value))
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "text") => {
                    if host.element_tag_name(element)? != "option" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    host.element_set_text_content(element, &as_string(&value))
                }
                (Value::Element(element), "selectedIndex") => {
                    if host.element_tag_name(element)? != "select" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    let selected_index = selected_index_from_value(&value)?;
                    if selected_index < -1 {
                        return Err(ScriptError::new(
                            "selectedIndex expects an integer greater than or equal to -1",
                        ));
                    }

                    let options = host.html_collection_select_options_items(element)?;
                    for (index, option) in options.into_iter().enumerate() {
                        if index as i64 == selected_index {
                            host.element_set_attribute(option, "selected", "")?;
                        } else if host.element_get_attribute(option, "selected")?.is_some() {
                            host.element_remove_attribute(option, "selected")?;
                        }
                    }
                    Ok(())
                }
                (Value::Element(element), "multiple") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "select" => {
                            if is_truthy(&value) {
                                host.element_set_attribute(element, "multiple", "")
                            } else {
                                host.element_remove_attribute(element, "multiple")?;
                                Ok(())
                            }
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "readOnly") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "textarea" => {
                            if is_truthy(&value) {
                                host.element_set_attribute(element, "readonly", "")
                            } else {
                                host.element_remove_attribute(element, "readonly")?;
                                Ok(())
                            }
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "autofocus") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "textarea" | "button" | "select" => {
                            if is_truthy(&value) {
                                host.element_set_attribute(element, "autofocus", "")
                            } else {
                                host.element_remove_attribute(element, "autofocus")?;
                                Ok(())
                            }
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "autocomplete") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "textarea" => {
                            host.element_set_attribute(element, "autocomplete", &as_string(&value))
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "accept") => {
                    if host.element_tag_name(element)? != "input" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    host.element_set_attribute(element, "accept", &as_string(&value))
                }
                (Value::Element(element), "minLength") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "textarea" => {
                            let Some(length) = index_from_value(&value) else {
                                return Err(ScriptError::new(
                                    "minLength expects a non-negative integer",
                                ));
                            };

                            host.element_set_attribute(element, "minlength", &length.to_string())
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "maxLength") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "textarea" => {
                            let Some(length) = index_from_value(&value) else {
                                return Err(ScriptError::new(
                                    "maxLength expects a non-negative integer",
                                ));
                            };

                            host.element_set_attribute(element, "maxlength", &length.to_string())
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "pattern") => {
                    if host.element_tag_name(element)? != "input" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    host.element_set_attribute(element, "pattern", &as_string(&value))
                }
                (Value::Element(element), "min") => {
                    if host.element_tag_name(element)? != "input" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    host.element_set_attribute(element, "min", &as_string(&value))
                }
                (Value::Element(element), "max") => {
                    if host.element_tag_name(element)? != "input" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    host.element_set_attribute(element, "max", &as_string(&value))
                }
                (Value::Element(element), "placeholder") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "textarea" => {
                            host.element_set_attribute(element, "placeholder", &as_string(&value))
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "required") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "textarea" | "select" => {
                            if is_truthy(&value) {
                                host.element_set_attribute(element, "required", "")
                            } else {
                                host.element_remove_attribute(element, "required")?;
                                Ok(())
                            }
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "noValidate") => {
                    if host.element_tag_name(element)? != "form" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    if is_truthy(&value) {
                        host.element_set_attribute(element, "novalidate", "")
                    } else {
                        host.element_remove_attribute(element, "novalidate")?;
                        Ok(())
                    }
                }
                (Value::Element(element), "method") => {
                    if host.element_tag_name(element)? != "form" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    host.element_set_attribute(
                        element,
                        "method",
                        &as_string(&value).trim().to_ascii_lowercase(),
                    )
                }
                (Value::Element(element), "enctype") => {
                    if host.element_tag_name(element)? != "form" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    host.element_set_attribute(
                        element,
                        "enctype",
                        &as_string(&value).trim().to_ascii_lowercase(),
                    )
                }
                (Value::Element(element), "target") => {
                    if host.element_tag_name(element)? != "form" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    host.element_set_attribute(element, "target", &as_string(&value))
                }
                (Value::Element(element), "action") => {
                    if host.element_tag_name(element)? != "form" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    host.element_set_attribute(element, "action", &as_string(&value))
                }
                (Value::Element(element), "formNoValidate") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "button" => {
                            if is_truthy(&value) {
                                host.element_set_attribute(element, "formnovalidate", "")
                            } else {
                                host.element_remove_attribute(element, "formnovalidate")?;
                                Ok(())
                            }
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "formMethod") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "button" => host.element_set_attribute(
                            element,
                            "formmethod",
                            &as_string(&value).trim().to_ascii_lowercase(),
                        ),
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "formEnctype") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "button" => host.element_set_attribute(
                            element,
                            "formenctype",
                            &as_string(&value).trim().to_ascii_lowercase(),
                        ),
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "formTarget") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "button" => {
                            host.element_set_attribute(element, "formtarget", &as_string(&value))
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "formAction") => {
                    match host.element_tag_name(element)?.as_str() {
                        "input" | "button" => {
                            host.element_set_attribute(element, "formaction", &as_string(&value))
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "size") => {
                    match host.element_tag_name(element)?.as_str() {
                        "select" | "input" => {
                            let Some(size) = index_from_value(&value) else {
                                return Err(ScriptError::new(
                                    "size expects a non-negative integer",
                                ));
                            };

                            host.element_set_attribute(element, "size", &size.to_string())
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "rows") => {
                    match host.element_tag_name(element)?.as_str() {
                        "textarea" => {
                            let Some(rows) = index_from_value(&value) else {
                                return Err(ScriptError::new(
                                    "rows expects a non-negative integer",
                                ));
                            };

                            host.element_set_attribute(element, "rows", &rows.to_string())
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "cols") => {
                    match host.element_tag_name(element)?.as_str() {
                        "textarea" => {
                            let Some(cols) = index_from_value(&value) else {
                                return Err(ScriptError::new(
                                    "cols expects a non-negative integer",
                                ));
                            };

                            host.element_set_attribute(element, "cols", &cols.to_string())
                        }
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "wrap") => {
                    match host.element_tag_name(element)?.as_str() {
                        "textarea" => host.element_set_attribute(
                            element,
                            "wrap",
                            &as_string(&value).trim().to_ascii_lowercase(),
                        ),
                        _ => Err(unsupported_member_access(property, "element")),
                    }
                }
                (Value::Element(element), "step") => {
                    if host.element_tag_name(element)? != "input" {
                        return Err(unsupported_member_access(property, "element"));
                    }

                    host.element_set_attribute(element, "step", &as_string(&value))
                }
                (Value::Element(element), "className") => {
                    host.element_set_attribute(element, "class", &as_string(&value))
                }
                (Value::ClassList(element), "value") => {
                    host.element_set_attribute(element, "class", &as_string(&value))
                }
                (Value::Element(element), "id") => {
                    host.element_set_attribute(element, "id", &as_string(&value))
                }
                (Value::Element(element), "name") => {
                    host.element_set_attribute(element, "name", &as_string(&value))
                }
                (Value::Element(element), "title") => {
                    host.element_set_attribute(element, "title", &as_string(&value))
                }
                (Value::Element(element), "role") => {
                    host.element_set_attribute(element, "role", &as_string(&value))
                }
                (Value::Element(element), "ariaLabel") => {
                    host.element_set_attribute(element, "aria-label", &as_string(&value))
                }
                (Value::Element(element), "ariaDescription") => {
                    host.element_set_attribute(element, "aria-description", &as_string(&value))
                }
                (Value::Element(element), "ariaRoleDescription") => {
                    host.element_set_attribute(element, "aria-roledescription", &as_string(&value))
                }
                (Value::Element(element), "ariaHidden") => {
                    host.element_set_attribute(element, "aria-hidden", &as_string(&value))
                }
                (Value::Element(element), "tabIndex") => {
                    let tab_index = integer_from_value(&value, "tabIndex")?;
                    host.element_set_attribute(element, "tabindex", &tab_index.to_string())
                }
                (Value::Element(element), "type") => match host.element_tag_name(element)?.as_str()
                {
                    "input" | "button" => host.element_set_attribute(
                        element,
                        "type",
                        &as_string(&value).trim().to_ascii_lowercase(),
                    ),
                    _ => Err(ScriptError::new(format!(
                        "unsupported assignment target on element: {property}"
                    ))),
                },
                (Value::Element(element), "accessKey") => {
                    host.element_set_attribute(element, "accesskey", &as_string(&value))
                }
                (Value::Element(element), "slot") => {
                    host.element_set_attribute(element, "slot", &as_string(&value))
                }
                (Value::Element(element), "autocapitalize") => {
                    host.element_set_attribute(element, "autocapitalize", &as_string(&value))
                }
                (Value::Element(element), "spellcheck") => {
                    if is_truthy(&value) {
                        host.element_set_attribute(element, "spellcheck", "true")
                    } else {
                        host.element_set_attribute(element, "spellcheck", "false")
                    }
                }
                (Value::Element(element), "translate") => {
                    if is_truthy(&value) {
                        host.element_set_attribute(element, "translate", "yes")
                    } else {
                        host.element_set_attribute(element, "translate", "no")
                    }
                }
                (Value::Element(element), "inputMode") => {
                    host.element_set_attribute(element, "inputmode", &as_string(&value))
                }
                (Value::Element(element), "hidden") => {
                    if is_truthy(&value) {
                        host.element_set_attribute(element, "hidden", "")
                    } else {
                        host.element_remove_attribute(element, "hidden")?;
                        Ok(())
                    }
                }
                (Value::Element(element), "dir") => {
                    host.element_set_attribute(element, "dir", &as_string(&value))
                }
                (Value::Element(element), "lang") => {
                    host.element_set_attribute(element, "lang", &as_string(&value))
                }
                (Value::RegExp(_), "lastIndex") => Ok(()),
                (Value::RegExp(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on regexp value"
                ))),
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
                (Value::Attribute(attr), property)
                    if matches!(property, "value" | "nodeValue" | "data" | "textContent") =>
                {
                    attribute_set_value(attr, &as_string(&value), host)
                }
                (Value::Attribute(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on attr value"
                ))),
                (Value::TemplateContent(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on template content value"
                ))),
                (Value::Element(_), _) => Err(ScriptError::new(format!(
                    "unsupported assignment target on element: {property}"
                ))),
                (Value::NamedNodeMap(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on named node map value"
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
                (Value::Date(_), property) => Err(unsupported_member_access(property, "date")),
                (Value::IntlNumberFormat(_), property) => {
                    Err(unsupported_member_access(property, "intl number format"))
                }
                (Value::IntlDateTimeFormat(_), property) => {
                    Err(unsupported_member_access(property, "intl date time format"))
                }
                (Value::IntlCollator(_), property) => {
                    Err(unsupported_member_access(property, "intl collator"))
                }
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
                (Value::Window, property) if window_property_is_reserved(property) => {
                    Err(ScriptError::new(format!(
                        "unsupported assignment target on window: {property}"
                    )))
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
                (Value::Clipboard, property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on clipboard value"
                ))),
                (Value::History, "scrollRestoration") => {
                    host.set_window_history_scroll_restoration(&as_string(&value))?;
                    Ok(())
                }
                (Value::History, property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on history value"
                ))),
                (Value::Object(object), property) => {
                    let key = property_key_from_string(property);
                    set_property_value_on_value(
                        &Value::Object(object),
                        key,
                        value.clone(),
                        env,
                        host,
                    )?;
                    Ok(())
                }
                (Value::Array(array), property) => {
                    let key = property_key_from_string(property);
                    set_property_value_on_value(
                        &Value::Array(array),
                        key,
                        value.clone(),
                        env,
                        host,
                    )?;
                    Ok(())
                }
                (Value::Map(map), property) => {
                    let key = property_key_from_string(property);
                    set_property_value_on_value(&Value::Map(map), key, value.clone(), env, host)?;
                    Ok(())
                }
                (Value::ObjectNamespace, property)
                | (Value::ArrayNamespace, property)
                | (Value::IntlNamespace, property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on namespace value"
                ))),
                (Value::Document, property) => Err(ScriptError::new(format!(
                    "unsupported assignment target: {property}"
                ))),
                (Value::Window, property) => {
                    env.insert(property.to_string(), value);
                    Ok(())
                }
                (Value::Symbol(_), property) => Err(ScriptError::new(format!(
                    "cannot assign to `{property}` on symbol value"
                ))),
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
        Expr::RegexLiteral { pattern, flags } => Ok(Value::RegExp(crate::RegExpValue::new(
            pattern.clone(),
            flags.clone(),
        ))),
        Expr::Boolean(value) => Ok(Value::Boolean(*value)),
        Expr::Null => Ok(Value::Null),
        Expr::Undefined => Ok(Value::Undefined),
        Expr::Member { object, property } => eval_member(object, property, env, host),
        Expr::OptionalMember { object, property } => {
            eval_optional_member(object, property, env, host)
        }
        Expr::OptionalMemberCall {
            object,
            property,
            args,
        } => eval_optional_member_call(object, property, args, env, host),
        Expr::ComputedMember { object, property } => {
            eval_computed_member(object, property, env, host)
        }
        Expr::Call { callee, args } => eval_call(callee, args, env, host),
        Expr::ArrayLiteral(elements) => eval_array_literal(elements, env, host),
        Expr::ObjectLiteral(properties) => eval_object_literal(properties, env, host),
        Expr::New { callee, args } => eval_new(callee, args, env, host),
        Expr::Assignment {
            target,
            value,
            operator,
        } => {
            let right = eval_expr(value, env, host)?;
            let result = match operator {
                AssignmentOperator::Assign => right.clone(),
                AssignmentOperator::AddAssign => {
                    let left = eval_assignment_target_value(target, env, host)?;
                    eval_add(left, right.clone())
                }
            };
            eval_assign(target, result.clone(), env, host)?;
            Ok(result)
        }
        Expr::BinaryAdd { left, right } => {
            let left = eval_expr(left, env, host)?;
            let right = eval_expr(right, env, host)?;
            Ok(eval_add(left, right))
        }
        Expr::BinarySub { left, right } => {
            let left = eval_number(left, env, host, "operator -")?;
            let right = eval_number(right, env, host, "operator -")?;
            Ok(Value::Number(left - right))
        }
        Expr::BinaryMul { left, right } => {
            let left = eval_number(left, env, host, "operator *")?;
            let right = eval_number(right, env, host, "operator *")?;
            Ok(Value::Number(left * right))
        }
        Expr::BinaryDiv { left, right } => {
            let left = eval_number(left, env, host, "operator /")?;
            let right = eval_number(right, env, host, "operator /")?;
            Ok(Value::Number(left / right))
        }
        Expr::BinaryRem { left, right } => {
            let left = eval_number(left, env, host, "operator %")?;
            let right = eval_number(right, env, host, "operator %")?;
            Ok(Value::Number(left % right))
        }
        Expr::LogicalAnd { left, right } => {
            let left = eval_expr(left, env, host)?;
            if is_truthy(&left) {
                eval_expr(right, env, host)
            } else {
                Ok(left)
            }
        }
        Expr::LogicalOr { left, right } => {
            let left = eval_expr(left, env, host)?;
            if is_truthy(&left) {
                Ok(left)
            } else {
                eval_expr(right, env, host)
            }
        }
        Expr::NullishCoalesce { left, right } => {
            let left = eval_expr(left, env, host)?;
            if matches!(left, Value::Null | Value::Undefined) {
                eval_expr(right, env, host)
            } else {
                Ok(left)
            }
        }
        Expr::Equality {
            left,
            right,
            negated,
            strict,
        } => {
            let left = eval_expr(left, env, host)?;
            let right = eval_expr(right, env, host)?;
            let equal = eval_equality(&left, &right, *strict);
            Ok(Value::Boolean(if *negated { !equal } else { equal }))
        }
        Expr::Comparison {
            left,
            right,
            operator,
        } => Ok(Value::Boolean(match operator {
            ComparisonOperator::InstanceOf => is_instance_of_constructor(
                &eval_expr(left, env, host)?,
                &eval_expr(right, env, host)?,
                host,
            )?,
            ComparisonOperator::LessThan
            | ComparisonOperator::LessThanOrEqual
            | ComparisonOperator::GreaterThan
            | ComparisonOperator::GreaterThanOrEqual => {
                let left = eval_number(left, env, host, "comparison")?;
                let right = eval_number(right, env, host, "comparison")?;
                match operator {
                    ComparisonOperator::LessThan => left < right,
                    ComparisonOperator::LessThanOrEqual => left <= right,
                    ComparisonOperator::GreaterThan => left > right,
                    ComparisonOperator::GreaterThanOrEqual => left >= right,
                    ComparisonOperator::InstanceOf => unreachable!(),
                }
            }
        })),
        Expr::UnaryNeg(expr) => {
            let value = eval_expr(expr, env, host)?;
            match value {
                Value::Number(number) => Ok(Value::Number(-number)),
                _ => Err(ScriptError::new("unary - expects a number")),
            }
        }
        Expr::UnaryNot(expr) => Ok(Value::Boolean(!is_truthy(&eval_expr(expr, env, host)?))),
        Expr::TypeOf(expr) => Ok(Value::String(
            eval_typeof(&eval_expr(expr, env, host)?).to_string(),
        )),
        Expr::Void(expr) => {
            let _ = eval_expr(expr, env, host)?;
            Ok(Value::Undefined)
        }
        Expr::Conditional {
            condition,
            consequent,
            alternate,
        } => {
            if is_truthy(&eval_expr(condition, env, host)?) {
                eval_expr(consequent, env, host)
            } else {
                eval_expr(alternate, env, host)
            }
        }
        Expr::ArrowFunction(function) | Expr::FunctionExpression(function) => Ok(Value::Function(
            function.clone().with_captured_bindings(env.clone()),
        )),
        Expr::Spread(_) => Err(ScriptError::new(
            "spread syntax is only valid in arrays, objects, and call arguments",
        )),
        Expr::Update {
            target,
            increment,
            prefix,
        } => {
            let current = eval_assignment_target_value(target, env, host)?;
            let current_number = match current.clone() {
                Value::Number(number) => number,
                Value::String(value) => value
                    .parse::<f64>()
                    .map_err(|_| ScriptError::new("update operator expects a number"))?,
                _ => {
                    return Err(ScriptError::new("update operator expects a number"));
                }
            };
            let next = if *increment {
                Value::Number(current_number + 1.0)
            } else {
                Value::Number(current_number - 1.0)
            };
            eval_assign(target, next.clone(), env, host)?;
            if *prefix { Ok(next) } else { Ok(current) }
        }
    }
}

fn eval_identifier(name: &str, env: &BTreeMap<String, Value>) -> Result<Value> {
    if let Some(value) = env.get(name) {
        return Ok(value.clone());
    }

    match name {
        "document" => Ok(Value::Document),
        "window" => Ok(Value::Window),
        "alert"
        | "confirm"
        | "prompt"
        | "open"
        | "close"
        | "print"
        | "scrollTo"
        | "scrollBy"
        | "matchMedia"
        | "requestAnimationFrame"
        | "cancelAnimationFrame"
        | "setTimeout"
        | "setInterval"
        | "clearTimeout"
        | "clearInterval"
        | "addEventListener"
        | "removeEventListener"
        | "dispatchEvent" => Ok(native_method_function(Value::Window, name)),
        "localStorage" => Ok(Value::Storage(StorageTarget::Local)),
        "sessionStorage" => Ok(Value::Storage(StorageTarget::Session)),
        "Object" => Ok(Value::ObjectNamespace),
        "Array" | "Uint8Array" => Ok(Value::ArrayNamespace),
        "String" => Ok(Value::ObjectNamespace),
        "Number" => Ok(Value::ObjectNamespace),
        "Date" => Ok(Value::ObjectNamespace),
        "Boolean" => Ok(native_global_function("Boolean")),
        "decodeURI" => Ok(native_global_function("decodeURI")),
        "decodeURIComponent" => Ok(native_global_function("decodeURIComponent")),
        "encodeURI" => Ok(native_global_function("encodeURI")),
        "encodeURIComponent" => Ok(native_global_function("encodeURIComponent")),
        "Node" => Ok(html_constructor_function("Node")),
        "Element" => Ok(html_constructor_function("Element")),
        "HTMLElement" => Ok(html_constructor_function("HTMLElement")),
        "HTMLButtonElement" => Ok(html_constructor_function("HTMLButtonElement")),
        "HTMLSelectElement" => Ok(html_constructor_function("HTMLSelectElement")),
        "HTMLInputElement" => Ok(html_constructor_function("HTMLInputElement")),
        "HTMLTextAreaElement" => Ok(html_constructor_function("HTMLTextAreaElement")),
        "HTMLFormElement" => Ok(html_constructor_function("HTMLFormElement")),
        "HTMLOptionElement" => Ok(html_constructor_function("HTMLOptionElement")),
        "HTMLOptGroupElement" => Ok(html_constructor_function("HTMLOptGroupElement")),
        "HTMLFieldSetElement" => Ok(html_constructor_function("HTMLFieldSetElement")),
        "HTMLLabelElement" => Ok(html_constructor_function("HTMLLabelElement")),
        "HTMLImageElement" => Ok(html_constructor_function("HTMLImageElement")),
        "HTMLAnchorElement" => Ok(html_constructor_function("HTMLAnchorElement")),
        "HTMLAreaElement" => Ok(html_constructor_function("HTMLAreaElement")),
        "HTMLMapElement" => Ok(html_constructor_function("HTMLMapElement")),
        "HTMLTableElement" => Ok(html_constructor_function("HTMLTableElement")),
        "HTMLTableSectionElement" => Ok(html_constructor_function("HTMLTableSectionElement")),
        "HTMLTableRowElement" => Ok(html_constructor_function("HTMLTableRowElement")),
        "HTMLTableCellElement" => Ok(html_constructor_function("HTMLTableCellElement")),
        "HTMLUListElement" => Ok(html_constructor_function("HTMLUListElement")),
        "HTMLOListElement" => Ok(html_constructor_function("HTMLOListElement")),
        "HTMLLIElement" => Ok(html_constructor_function("HTMLLIElement")),
        "HTMLObjectElement" => Ok(html_constructor_function("HTMLObjectElement")),
        "HTMLEmbedElement" => Ok(html_constructor_function("HTMLEmbedElement")),
        "HTMLLegendElement" => Ok(html_constructor_function("HTMLLegendElement")),
        "HTMLDListElement" => Ok(html_constructor_function("HTMLDListElement")),
        "HTMLScriptElement" => Ok(html_constructor_function("HTMLScriptElement")),
        "HTMLStyleElement" => Ok(html_constructor_function("HTMLStyleElement")),
        "Math" => Ok(Value::IntlNamespace),
        "CSS" => Ok(Value::IntlNamespace),
        "Intl" => Ok(Value::IntlNamespace),
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
        Value::Document if property == "namespaceURI" => Ok(Value::Null),
        Value::Document if property == "title" => Ok(Value::String(host.document_title()?)),
        Value::Document if property == "location" => Ok(Value::String(host.document_location()?)),
        Value::Document if property == "URL" => Ok(Value::String(host.document_url()?)),
        Value::Document if property == "documentURI" => {
            Ok(Value::String(host.document_document_uri()?))
        }
        Value::Document if property == "baseURI" => Ok(Value::String(host.document_base_uri()?)),
        Value::Document if property == "origin" => Ok(Value::String(host.document_origin()?)),
        Value::Document
            if matches!(
                property,
                "createElement"
                    | "createElementNS"
                    | "createTextNode"
                    | "createComment"
                    | "createAttribute"
                    | "createAttributeNS"
                    | "createDocumentFragment"
                    | "importNode"
                    | "normalize"
                    | "removeChild"
                    | "open"
                    | "close"
                    | "write"
                    | "writeln"
                    | "contains"
                    | "isSameNode"
                    | "isEqualNode"
                    | "compareDocumentPosition"
                    | "hasChildNodes"
                    | "hasFocus"
                    | "querySelector"
                    | "querySelectorAll"
                    | "getElementById"
                    | "getElementsByName"
            ) =>
        {
            Ok(native_method_function(Value::Document, property))
        }
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
            Ok(Value::HtmlCollection(HtmlCollectionTarget::DocumentPlugins))
        }
        Value::Window if property == "document" => Ok(Value::Document),
        Value::Window if property == "Node" => Ok(html_constructor_function("Node")),
        Value::Window if property == "Element" => Ok(html_constructor_function("Element")),
        Value::Window if property == "HTMLElement" => Ok(html_constructor_function("HTMLElement")),
        Value::Window if property == "HTMLButtonElement" => {
            Ok(html_constructor_function("HTMLButtonElement"))
        }
        Value::Window if property == "HTMLSelectElement" => {
            Ok(html_constructor_function("HTMLSelectElement"))
        }
        Value::Window if property == "HTMLInputElement" => {
            Ok(html_constructor_function("HTMLInputElement"))
        }
        Value::Window if property == "HTMLTextAreaElement" => {
            Ok(html_constructor_function("HTMLTextAreaElement"))
        }
        Value::Window if property == "HTMLFormElement" => {
            Ok(html_constructor_function("HTMLFormElement"))
        }
        Value::Window if property == "HTMLOptionElement" => {
            Ok(html_constructor_function("HTMLOptionElement"))
        }
        Value::Window if property == "HTMLOptGroupElement" => {
            Ok(html_constructor_function("HTMLOptGroupElement"))
        }
        Value::Window if property == "HTMLFieldSetElement" => {
            Ok(html_constructor_function("HTMLFieldSetElement"))
        }
        Value::Window if property == "HTMLLabelElement" => {
            Ok(html_constructor_function("HTMLLabelElement"))
        }
        Value::Window if property == "HTMLImageElement" => {
            Ok(html_constructor_function("HTMLImageElement"))
        }
        Value::Window if property == "HTMLAnchorElement" => {
            Ok(html_constructor_function("HTMLAnchorElement"))
        }
        Value::Window if property == "HTMLAreaElement" => {
            Ok(html_constructor_function("HTMLAreaElement"))
        }
        Value::Window if property == "HTMLMapElement" => {
            Ok(html_constructor_function("HTMLMapElement"))
        }
        Value::Window if property == "HTMLTableElement" => {
            Ok(html_constructor_function("HTMLTableElement"))
        }
        Value::Window if property == "HTMLTableSectionElement" => {
            Ok(html_constructor_function("HTMLTableSectionElement"))
        }
        Value::Window if property == "HTMLTableRowElement" => {
            Ok(html_constructor_function("HTMLTableRowElement"))
        }
        Value::Window if property == "HTMLTableCellElement" => {
            Ok(html_constructor_function("HTMLTableCellElement"))
        }
        Value::Window if property == "HTMLUListElement" => {
            Ok(html_constructor_function("HTMLUListElement"))
        }
        Value::Window if property == "HTMLOListElement" => {
            Ok(html_constructor_function("HTMLOListElement"))
        }
        Value::Window if property == "HTMLLIElement" => {
            Ok(html_constructor_function("HTMLLIElement"))
        }
        Value::Window if property == "HTMLObjectElement" => {
            Ok(html_constructor_function("HTMLObjectElement"))
        }
        Value::Window if property == "HTMLEmbedElement" => {
            Ok(html_constructor_function("HTMLEmbedElement"))
        }
        Value::Window if property == "HTMLLegendElement" => {
            Ok(html_constructor_function("HTMLLegendElement"))
        }
        Value::Window if property == "HTMLDListElement" => {
            Ok(html_constructor_function("HTMLDListElement"))
        }
        Value::Window if property == "HTMLScriptElement" => {
            Ok(html_constructor_function("HTMLScriptElement"))
        }
        Value::Window if property == "HTMLStyleElement" => {
            Ok(html_constructor_function("HTMLStyleElement"))
        }
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
        Value::Window if env.contains_key(property) => {
            Ok(env.get(property).cloned().unwrap_or(Value::Undefined))
        }
        Value::Window
            if matches!(
                property,
                "matchMedia"
                    | "requestAnimationFrame"
                    | "cancelAnimationFrame"
                    | "setTimeout"
                    | "setInterval"
                    | "clearTimeout"
                    | "clearInterval"
                    | "open"
                    | "close"
                    | "print"
                    | "scrollTo"
                    | "scrollBy"
                    | "addEventListener"
                    | "removeEventListener"
                    | "dispatchEvent"
            ) =>
        {
            Ok(native_method_function(Value::Window, property))
        }
        Value::Window if property == "URL" => Ok(Value::ObjectNamespace),
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
        Value::Navigator if property == "clipboard" => Ok(Value::Clipboard),
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
            Ok(Value::HtmlCollection(HtmlCollectionTarget::DocumentPlugins))
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
        Value::History
            if matches!(
                property,
                "pushState" | "replaceState" | "back" | "forward" | "go"
            ) =>
        {
            Ok(native_method_function(Value::History, property))
        }
        Value::RegExp(value) if matches!(property, "test" | "exec" | "toString" | "valueOf") => Ok(
            native_method_function(Value::RegExp(value.clone()), property),
        ),
        Value::RegExp(value) if property == "source" => {
            Ok(Value::String(value.pattern().to_string()))
        }
        Value::RegExp(value) if property == "flags" => Ok(Value::String(value.flags().to_string())),
        Value::RegExp(value) if property == "global" => Ok(Value::Boolean(value.is_global())),
        Value::RegExp(value) if property == "ignoreCase" => {
            Ok(Value::Boolean(value.is_ignore_case()))
        }
        Value::RegExp(value) if property == "multiline" => Ok(Value::Boolean(value.is_multiline())),
        Value::RegExp(value) if property == "dotAll" => Ok(Value::Boolean(value.is_dot_all())),
        Value::RegExp(value) if property == "unicode" => Ok(Value::Boolean(value.is_unicode())),
        Value::RegExp(value) if property == "sticky" => Ok(Value::Boolean(value.is_sticky())),
        Value::RegExp(_) if property == "lastIndex" => Ok(Value::Number(0.0)),
        Value::String(value) if property == "length" => {
            Ok(Value::Number(value.chars().count() as f64))
        }
        Value::String(value)
            if matches!(
                property,
                "trim"
                    | "toString"
                    | "valueOf"
                    | "split"
                    | "replace"
                    | "replaceAll"
                    | "match"
                    | "search"
                    | "toLowerCase"
                    | "toUpperCase"
                    | "includes"
                    | "startsWith"
                    | "endsWith"
                    | "indexOf"
                    | "lastIndexOf"
                    | "slice"
                    | "substring"
                    | "charAt"
                    | "charCodeAt"
                    | "concat"
                    | "repeat"
                    | "normalize"
            ) =>
        {
            Ok(native_method_function(
                Value::String(value.clone()),
                property,
            ))
        }
        Value::Symbol(symbol) if property == "description" => Ok(Value::String(
            symbol.description().unwrap_or_default().to_string(),
        )),
        Value::Symbol(_) => Err(unsupported_member_access(property, "symbol")),
        Value::Element(element) if property == "textContent" => {
            Ok(Value::String(host.element_text_content(element)?))
        }
        Value::Element(element) if property == "innerHTML" => {
            Ok(Value::String(host.element_inner_html(element)?))
        }
        Value::Element(element) if property == "outerHTML" => {
            Ok(Value::String(host.element_outer_html(element)?))
        }
        Value::Element(element) if property == "tagName" => {
            Ok(Value::String(host.element_tag_name(element)?))
        }
        Value::Element(element) if property == "localName" => {
            Ok(Value::String(host.element_tag_name(element)?))
        }
        Value::Element(element) if property == "namespaceURI" => Ok(
            match host.node_namespace_uri(NodeHandle::new(element.raw()))? {
                Some(value) => Value::String(value),
                None => Value::Null,
            },
        ),
        Value::Element(element) if property == "id" => Ok(Value::String(
            host.element_get_attribute(element, "id")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "name" => Ok(Value::String(
            host.element_get_attribute(element, "name")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "title" => Ok(Value::String(
            host.element_get_attribute(element, "title")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "role" => Ok(Value::String(
            host.element_get_attribute(element, "role")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "ariaLabel" => Ok(Value::String(
            host.element_get_attribute(element, "aria-label")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "ariaDescription" => Ok(Value::String(
            host.element_get_attribute(element, "aria-description")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "ariaRoleDescription" => Ok(Value::String(
            host.element_get_attribute(element, "aria-roledescription")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "ariaHidden" => Ok(Value::String(
            host.element_get_attribute(element, "aria-hidden")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "tabIndex" => {
            let default = if element_is_natively_focusable(element, host)? {
                0
            } else {
                -1
            };
            let value = host.element_get_attribute(element, "tabindex")?;
            let tab_index = value
                .as_deref()
                .and_then(|value| value.trim().parse::<i64>().ok())
                .unwrap_or(default);
            Ok(Value::Number(tab_index as f64))
        }
        Value::Element(element) if property == "accessKey" => Ok(Value::String(
            host.element_get_attribute(element, "accesskey")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "slot" => Ok(Value::String(
            host.element_get_attribute(element, "slot")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "autocapitalize" => Ok(Value::String(
            host.element_get_attribute(element, "autocapitalize")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "inputMode" => Ok(Value::String(
            host.element_get_attribute(element, "inputmode")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "spellcheck" => {
            Ok(Value::Boolean(spellcheck_reflection(element, host)?))
        }
        Value::Element(element) if property == "translate" => {
            Ok(Value::Boolean(translate_reflection(element, host)?))
        }
        Value::Element(element) if property == "hidden" => Ok(Value::Boolean(
            host.element_get_attribute(element, "hidden")?.is_some(),
        )),
        Value::Element(element) if property == "dir" => Ok(Value::String(
            host.element_get_attribute(element, "dir")?
                .unwrap_or_default(),
        )),
        Value::Element(element) if property == "lang" => Ok(Value::String(
            host.element_get_attribute(element, "lang")?
                .unwrap_or_default(),
        )),
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
        Value::Element(element) if property == "defaultValue" => {
            match host.element_tag_name(element)?.as_str() {
                "input" => Ok(Value::String(
                    host.element_get_attribute(element, "value")?
                        .unwrap_or_default(),
                )),
                "textarea" => Ok(Value::String(host.element_text_content(element)?)),
                _ => Err(unsupported_member_access(property, "element")),
            }
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
        Value::Element(element) if property == "defaultChecked" => {
            if host.element_tag_name(element)? != "input" {
                return Err(unsupported_member_access(property, "element"));
            }

            match host.element_get_attribute(element, "type")?.as_deref() {
                Some("checkbox") | Some("radio") => {
                    Ok(Value::Boolean(host.element_checked(element)?))
                }
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "indeterminate" => {
            if host.element_tag_name(element)? != "input" {
                return Err(unsupported_member_access(property, "element"));
            }

            match host.element_get_attribute(element, "type")?.as_deref() {
                Some("checkbox") => Ok(Value::Boolean(host.element_indeterminate(element)?)),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "selected" => {
            if host.element_tag_name(element)? != "option" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::Boolean(
                host.element_get_attribute(element, "selected")?.is_some(),
            ))
        }
        Value::Element(element) if property == "defaultSelected" => {
            if host.element_tag_name(element)? != "option" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::Boolean(
                host.element_get_attribute(element, "selected")?.is_some(),
            ))
        }
        Value::Element(element) if property == "selectedIndex" => {
            if host.element_tag_name(element)? != "select" {
                return Err(unsupported_member_access(property, "element"));
            }

            let options = host.html_collection_select_options_items(element)?;
            let index = options.iter().enumerate().find_map(|(index, option)| {
                match host.element_get_attribute(*option, "selected") {
                    Ok(Some(_)) => Some(Ok(index as i64)),
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                }
            });

            Ok(Value::Number(match index {
                Some(Ok(index)) => index as f64,
                Some(Err(error)) => return Err(error),
                None => -1.0,
            }))
        }
        Value::Element(element) if property == "multiple" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "select" => Ok(Value::Boolean(
                    host.element_get_attribute(element, "multiple")?.is_some(),
                )),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "accept" => {
            if host.element_tag_name(element)? != "input" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::String(
                host.element_get_attribute(element, "accept")?
                    .unwrap_or_default(),
            ))
        }
        Value::Element(element) if property == "readOnly" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "textarea" => Ok(Value::Boolean(
                    host.element_get_attribute(element, "readonly")?.is_some(),
                )),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "autofocus" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "textarea" | "button" | "select" => Ok(Value::Boolean(
                    host.element_get_attribute(element, "autofocus")?.is_some(),
                )),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "autocomplete" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "textarea" => Ok(Value::String(
                    host.element_get_attribute(element, "autocomplete")?
                        .unwrap_or_default(),
                )),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "minLength" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "textarea" => Ok(Value::Number(
                    host.element_get_attribute(element, "minlength")?
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or(0) as f64,
                )),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "maxLength" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "textarea" => Ok(Value::Number(
                    host.element_get_attribute(element, "maxlength")?
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or(0) as f64,
                )),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "pattern" => {
            if host.element_tag_name(element)? != "input" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::String(
                host.element_get_attribute(element, "pattern")?
                    .unwrap_or_default(),
            ))
        }
        Value::Element(element) if property == "min" => {
            if host.element_tag_name(element)? != "input" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::String(
                host.element_get_attribute(element, "min")?
                    .unwrap_or_default(),
            ))
        }
        Value::Element(element) if property == "max" => {
            if host.element_tag_name(element)? != "input" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::String(
                host.element_get_attribute(element, "max")?
                    .unwrap_or_default(),
            ))
        }
        Value::Element(element) if property == "step" => {
            if host.element_tag_name(element)? != "input" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::String(
                host.element_get_attribute(element, "step")?
                    .unwrap_or_default(),
            ))
        }
        Value::Element(element) if property == "placeholder" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "textarea" => Ok(Value::String(
                    host.element_get_attribute(element, "placeholder")?
                        .unwrap_or_default(),
                )),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "required" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "textarea" | "select" => Ok(Value::Boolean(
                    host.element_get_attribute(element, "required")?.is_some(),
                )),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "noValidate" => {
            if host.element_tag_name(element)? != "form" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::Boolean(
                host.element_get_attribute(element, "novalidate")?.is_some(),
            ))
        }
        Value::Element(element) if property == "method" => {
            if host.element_tag_name(element)? != "form" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::String(form_method_reflection(
                host.element_get_attribute(element, "method")?.as_deref(),
            )))
        }
        Value::Element(element) if property == "enctype" => {
            if host.element_tag_name(element)? != "form" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::String(form_enctype_reflection(
                host.element_get_attribute(element, "enctype")?.as_deref(),
            )))
        }
        Value::Element(element) if property == "target" => {
            if host.element_tag_name(element)? != "form" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::String(form_target_reflection(
                host.element_get_attribute(element, "target")?.as_deref(),
            )))
        }
        Value::Element(element) if property == "action" => {
            if host.element_tag_name(element)? != "form" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::String(form_action_reflection(element, host)?))
        }
        Value::Element(element) if property == "formNoValidate" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "button" => Ok(Value::Boolean(
                    host.element_get_attribute(element, "formnovalidate")?
                        .is_some(),
                )),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "formMethod" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "button" => Ok(Value::String(form_method_reflection(
                    host.element_get_attribute(element, "formmethod")?
                        .as_deref(),
                ))),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "formEnctype" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "button" => Ok(Value::String(form_enctype_reflection(
                    host.element_get_attribute(element, "formenctype")?
                        .as_deref(),
                ))),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "formTarget" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "button" => Ok(Value::String(form_target_reflection(
                    host.element_get_attribute(element, "formtarget")?
                        .as_deref(),
                ))),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "formAction" => {
            match host.element_tag_name(element)?.as_str() {
                "input" | "button" => Ok(Value::String(form_action_override_reflection(
                    element, host,
                )?)),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "size" => {
            match host.element_tag_name(element)?.as_str() {
                "select" => Ok(Value::Number(
                    host.element_get_attribute(element, "size")?
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or(0) as f64,
                )),
                "input" => Ok(Value::Number(
                    host.element_get_attribute(element, "size")?
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or(20) as f64,
                )),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "type" => {
            match host.element_tag_name(element)?.as_str() {
                "select" => Ok(Value::String(
                    if host.element_get_attribute(element, "multiple")?.is_some() {
                        "select-multiple".to_string()
                    } else {
                        "select-one".to_string()
                    },
                )),
                "input" => Ok(Value::String(input_type_reflection(
                    host.element_get_attribute(element, "type")?.as_deref(),
                ))),
                "button" => Ok(Value::String(button_type_reflection(
                    host.element_get_attribute(element, "type")?.as_deref(),
                ))),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "index" => Ok(Value::Number(
            option_index_for_element(element, host)? as f64,
        )),
        Value::Element(element) if property == "form" => form_owner_for_element(element, host),
        Value::Element(element) if property == "disabled" => match host
            .element_tag_name(element)?
            .as_str()
        {
            "input" | "textarea" | "button" | "select" | "option" | "optgroup" | "fieldset" => Ok(
                Value::Boolean(host.element_get_attribute(element, "disabled")?.is_some()),
            ),
            _ => Err(unsupported_member_access(property, "element")),
        },
        Value::Element(element) if property == "label" => {
            match host.element_tag_name(element)?.as_str() {
                "option" => Ok(Value::String(
                    host.element_get_attribute(element, "label")?
                        .unwrap_or(host.element_text_content(element)?),
                )),
                "optgroup" => Ok(Value::String(
                    host.element_get_attribute(element, "label")?
                        .unwrap_or_default(),
                )),
                _ => Err(unsupported_member_access(property, "element")),
            }
        }
        Value::Element(element) if property == "text" => {
            if host.element_tag_name(element)? != "option" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::String(host.element_text_content(element)?))
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
        Value::Element(element) if property == "attributes" => Ok(Value::NamedNodeMap(element)),
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
        Value::Element(element) if property == "rows" => {
            if host.element_tag_name(element)? == "textarea" {
                Ok(Value::Number(positive_index_from_attribute(
                    host.element_get_attribute(element, "rows")?,
                    2,
                ) as f64))
            } else {
                Ok(Value::HtmlCollection(HtmlCollectionTarget::TableRows(
                    element,
                )))
            }
        }
        Value::Element(element) if property == "cols" => {
            if host.element_tag_name(element)? != "textarea" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::Number(positive_index_from_attribute(
                host.element_get_attribute(element, "cols")?,
                20,
            ) as f64))
        }
        Value::Element(element) if property == "wrap" => {
            if host.element_tag_name(element)? != "textarea" {
                return Err(unsupported_member_access(property, "element"));
            }

            Ok(Value::String(textarea_wrap_reflection(
                host.element_get_attribute(element, "wrap")?.as_deref(),
            )))
        }
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
        Value::Event(event) if property == "key" => {
            Ok(event.key().map(Value::String).unwrap_or(Value::Undefined))
        }
        Value::Event(event) if property == "code" => {
            Ok(event.code().map(Value::String).unwrap_or(Value::Undefined))
        }
        Value::Event(event) if property == "ctrlKey" => Ok(Value::Boolean(event.ctrl_key())),
        Value::Event(event) if property == "metaKey" => Ok(Value::Boolean(event.meta_key())),
        Value::Event(event) if property == "shiftKey" => Ok(Value::Boolean(event.shift_key())),
        Value::Event(event) if property == "altKey" => Ok(Value::Boolean(event.alt_key())),
        Value::Event(event) if property == "repeat" => Ok(Value::Boolean(event.repeat())),
        Value::Event(event) if property == "isComposing" => {
            Ok(Value::Boolean(event.is_composing()))
        }
        Value::Event(event) if property == "isTrusted" => Ok(Value::Boolean(event.is_trusted())),
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
        Value::Node(node) if property == "namespaceURI" => {
            Ok(match host.node_namespace_uri(node)? {
                Some(value) => Value::String(value),
                None => Value::Null,
            })
        }
        Value::HtmlCollection(collection) if property == "length" => {
            let length = html_collection_items(&collection, host)?.len();
            Ok(Value::Number(length as f64))
        }
        Value::HtmlCollection(collection)
            if html_collection_property_is_reserved(&collection, property) =>
        {
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
        Value::Object(object) => Ok(
            match property_value_on_value(
                &Value::Object(object.clone()),
                &property_key_from_string(property),
                env,
                host,
            )? {
                Some(value) => value,
                None => Value::Undefined,
            },
        ),
        Value::Array(array) => Ok(
            match property_value_on_value(
                &Value::Array(array.clone()),
                &property_key_from_string(property),
                env,
                host,
            )? {
                Some(value) => value,
                None => Value::Undefined,
            },
        ),
        Value::Map(map) if property == "size" => {
            Ok(Value::Number(map.0.borrow().entries.len() as f64))
        }
        Value::Map(map) => Ok(
            match property_value_on_value(
                &Value::Map(map.clone()),
                &property_key_from_string(property),
                env,
                host,
            )? {
                Some(value) => value,
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
        Value::Attribute(attr) if property == "name" => Ok(Value::String(attr.name())),
        Value::Attribute(attr) if property == "nodeName" => Ok(Value::String(attr.name())),
        Value::Attribute(attr) if property == "localName" => Ok(Value::String(attr.local_name())),
        Value::Attribute(attr) if property == "prefix" => Ok(match attr.prefix() {
            Some(value) => Value::String(value),
            None => Value::Null,
        }),
        Value::Attribute(attr) if property == "namespaceURI" => Ok(match attr.namespace_uri() {
            Some(value) => Value::String(value),
            None => Value::Null,
        }),
        Value::Attribute(_) if property == "nodeType" => Ok(Value::Number(2.0)),
        Value::Attribute(_) if property == "ownerDocument" => Ok(Value::Document),
        Value::Attribute(attr) if property == "ownerElement" => {
            Ok(attribute_owner_element_value(&attr, host)?)
        }
        Value::Attribute(_) if property == "specified" => Ok(Value::Boolean(true)),
        Value::Attribute(attr) if property == "isId" => {
            Ok(Value::Boolean(matches!(attr.name().as_str(), "id")))
        }
        Value::Attribute(_) if property == "parentNode" => Ok(Value::Null),
        Value::Attribute(_) if property == "parentElement" => Ok(Value::Null),
        Value::Attribute(attr)
            if property == "value"
                || property == "nodeValue"
                || property == "data"
                || property == "textContent" =>
        {
            Ok(Value::String(attribute_current_value(&attr, host)?))
        }
        Value::NamedNodeMap(element) if property == "length" => Ok(Value::Number(
            named_node_map_names(element, host)?.len() as f64,
        )),
        Value::ClassList(element) if property == "length" => {
            let length = class_list_tokens(element, host)?.len();
            Ok(Value::Number(length as f64))
        }
        Value::ClassList(element) if property == "value" => Ok(Value::String(
            host.element_get_attribute(element, "class")?
                .unwrap_or_default(),
        )),
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
        Value::ObjectNamespace if property == "prototype" => {
            let prototype = crate::ObjectHandle::new();
            object_set_property_value(
                &prototype,
                PropertyKey::String("hasOwnProperty".to_string()),
                Value::Function(crate::ScriptFunction::new(
                    Vec::new(),
                    format!("{NATIVE_METHOD_PREFIX}hasOwnProperty"),
                )),
                env,
                host,
            )?;
            Ok(Value::Object(prototype))
        }
        Value::ObjectNamespace
            if matches!(
                property,
                "assign"
                    | "keys"
                    | "values"
                    | "entries"
                    | "fromEntries"
                    | "getOwnPropertySymbols"
                    | "fromCharCode"
                    | "isFinite"
                    | "isNaN"
                    | "parseFloat"
                    | "parseInt"
                    | "now"
                    | "UTC"
                    | "parse"
            ) =>
        {
            Ok(native_method_function(Value::ObjectNamespace, property))
        }
        Value::ArrayNamespace if matches!(property, "isArray" | "from") => {
            Ok(native_method_function(Value::ArrayNamespace, property))
        }
        Value::IntlNamespace
            if matches!(
                property,
                "NumberFormat"
                    | "DateTimeFormat"
                    | "Collator"
                    | "min"
                    | "max"
                    | "abs"
                    | "floor"
                    | "ceil"
                    | "round"
                    | "trunc"
                    | "sign"
                    | "pow"
                    | "sqrt"
                    | "escape"
            ) =>
        {
            Ok(native_method_function(Value::IntlNamespace, property))
        }
        Value::ObjectNamespace | Value::ArrayNamespace | Value::IntlNamespace => {
            Ok(Value::Undefined)
        }
        Value::Navigator => Err(unsupported_member_access(property, "navigator")),
        Value::Clipboard => Err(unsupported_member_access(property, "clipboard")),
        Value::History => Err(unsupported_member_access(property, "history")),
        Value::Screen => Err(unsupported_member_access(property, "screen")),
        Value::RegExp(value) if property == "source" => {
            Ok(Value::String(value.pattern().to_string()))
        }
        Value::RegExp(value) if property == "flags" => Ok(Value::String(value.flags().to_string())),
        Value::RegExp(value) if property == "global" => Ok(Value::Boolean(value.is_global())),
        Value::RegExp(value) if property == "ignoreCase" => {
            Ok(Value::Boolean(value.is_ignore_case()))
        }
        Value::RegExp(value) if property == "multiline" => Ok(Value::Boolean(value.is_multiline())),
        Value::RegExp(value) if property == "dotAll" => Ok(Value::Boolean(value.is_dot_all())),
        Value::RegExp(value) if property == "unicode" => Ok(Value::Boolean(value.is_unicode())),
        Value::RegExp(value) if property == "sticky" => Ok(Value::Boolean(value.is_sticky())),
        Value::RegExp(_) if property == "lastIndex" => Ok(Value::Number(0.0)),
        Value::RegExp(value) if matches!(property, "test" | "exec" | "toString" | "valueOf") => Ok(
            native_method_function(Value::RegExp(value.clone()), property),
        ),
        Value::RegExp(_) => Err(unsupported_member_access(property, "regexp")),
        Value::Date(value)
            if matches!(
                property,
                "toLocaleDateString"
                    | "toLocaleString"
                    | "toISOString"
                    | "toJSON"
                    | "toString"
                    | "valueOf"
                    | "getTime"
            ) =>
        {
            Ok(native_method_function(Value::Date(value.clone()), property))
        }
        Value::Date(value)
            if matches!(
                property,
                "getFullYear"
                    | "getUTCFullYear"
                    | "getMonth"
                    | "getUTCMonth"
                    | "getDate"
                    | "getUTCDate"
                    | "getHours"
                    | "getUTCHours"
                    | "getMinutes"
                    | "getUTCMinutes"
                    | "getSeconds"
                    | "getUTCSeconds"
                    | "getMilliseconds"
                    | "getUTCMilliseconds"
                    | "getTimezoneOffset"
            ) =>
        {
            Ok(native_method_function(Value::Date(value.clone()), property))
        }
        Value::Date(_) => Err(unsupported_member_access(property, "date")),
        Value::IntlNumberFormat(value)
            if matches!(property, "format" | "resolvedOptions" | "formatToParts") =>
        {
            Ok(native_method_function(
                Value::IntlNumberFormat(value.clone()),
                property,
            ))
        }
        Value::IntlNumberFormat(_) => {
            Err(unsupported_member_access(property, "intl number format"))
        }
        Value::IntlDateTimeFormat(value)
            if matches!(property, "format" | "resolvedOptions" | "formatToParts") =>
        {
            Ok(native_method_function(
                Value::IntlDateTimeFormat(value.clone()),
                property,
            ))
        }
        Value::IntlDateTimeFormat(_) => {
            Err(unsupported_member_access(property, "intl datetime format"))
        }
        Value::IntlCollator(value) if matches!(property, "compare" | "resolvedOptions") => Ok(
            native_method_function(Value::IntlCollator(value.clone()), property),
        ),
        Value::IntlCollator(_) => Err(unsupported_member_access(property, "intl collator")),
        Value::Element(_) => Err(unsupported_member_access(property, "element")),
        Value::ClassList(_) => Err(unsupported_member_access(property, "class list")),
        Value::Attribute(_) => Err(unsupported_member_access(property, "attr")),
        Value::NamedNodeMap(_) => Err(unsupported_member_access(property, "named node map")),
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
        Value::TemplateContent(_) if property == "namespaceURI" => Ok(Value::Null),
        Value::TemplateContent(_) if property == "ownerDocument" => Ok(Value::Document),
        Value::Document => Err(unsupported_member_access(property, "document")),
        Value::Window => Ok(Value::Undefined),
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
        Value::Function(function) if property == "call" => Ok(native_method_function(
            Value::Function(function.clone()),
            property,
        )),
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
            let value = match args.first() {
                Some(expr) => eval_expr(expr, env, host)?,
                None => Value::Undefined,
            };
            Ok(Value::String(as_string(&value)))
        }
        Expr::Identifier(name) if name == "Boolean" => {
            let value = match args.first() {
                Some(expr) => eval_expr(expr, env, host)?,
                None => Value::Undefined,
            };
            Ok(Value::Boolean(is_truthy(&value)))
        }
        Expr::Identifier(name) if name == "Number" => {
            let value = match args.first() {
                Some(expr) => eval_expr(expr, env, host)?,
                None => Value::Undefined,
            };
            Ok(Value::Number(number_from_value(&value)?))
        }
        Expr::Identifier(name) if name == "Symbol" => {
            if args.len() > 1 {
                return Err(ScriptError::new("Symbol() accepts at most one argument"));
            }
            let description = match args.first() {
                Some(expr) => Some(as_string(&eval_expr(expr, env, host)?)),
                None => None,
            };
            Ok(Value::Symbol(crate::SymbolValue::new(description)))
        }
        Expr::Member { object, property } => {
            if let Some(value) = try_eval_location_method_call(object, property, args, env, host)? {
                return Ok(value);
            }
            let object_value = eval_expr(object, env, host)?;
            eval_method_call(object_value, property, args, env, host)
        }
        Expr::ArrowFunction(function) | Expr::FunctionExpression(function) => {
            call_script_function(function, args, env, host)
        }
        Expr::Identifier(_) => {
            let callee = eval_expr(callee, env, host)?;
            match callee {
                Value::Function(function) => call_script_function(&function, args, env, host),
                _ => Err(ScriptError::new("invalid call target")),
            }
        }
        Expr::String(_)
        | Expr::Number(_)
        | Expr::Boolean(_)
        | Expr::Null
        | Expr::Undefined
        | Expr::UnaryNeg(_) => Err(ScriptError::new("invalid call target")),
        Expr::Call { .. } | Expr::BinaryAdd { .. } => {
            Err(ScriptError::new("invalid nested call target"))
        }
        _ => Err(ScriptError::new("invalid call target")),
    }
}

fn eval_optional_member<H: HostBindings>(
    object: &Expr,
    property: &str,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let object_value = eval_expr(object, env, host)?;
    if matches!(object_value, Value::Null | Value::Undefined) {
        return Ok(Value::Undefined);
    }

    Ok(
        match property_value_on_value(
            &object_value,
            &property_key_from_string(property),
            env,
            host,
        )? {
            Some(value) => value,
            None => Value::Undefined,
        },
    )
}

fn eval_optional_member_call<H: HostBindings>(
    object: &Expr,
    property: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let object_value = eval_expr(object, env, host)?;
    if matches!(object_value, Value::Null | Value::Undefined) {
        return Ok(Value::Undefined);
    }

    eval_method_call(object_value, property, args, env, host)
}

fn call_script_function<H: HostBindings>(
    function: &crate::ScriptFunction,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let values = eval_call_argument_values(args, env, host)?;
    call_script_function_value(function, &values, Value::Undefined, env, host)
}

fn call_script_function_with_this<H: HostBindings>(
    function: &crate::ScriptFunction,
    args: &[Expr],
    this_value: Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let mut bindings = function.captured_bindings.as_ref().clone();
    bindings.extend(env.clone());
    let outer_names: Vec<String> = env.keys().cloned().collect();
    let bound_names = function_bound_names(function);

    bindings.insert("this".to_string(), this_value);

    let program = crate::parser::parse_program(&function.body_source)?;
    let local_names = collect_declared_names_from_program(&program);
    bind_function_arguments_from_exprs(function, args, env, host, &mut bindings)?;
    let control = eval_program_with_bindings(&program, host, &mut bindings)?;

    for name in outer_names {
        if name == "this" || bound_names.contains(&name) || local_names.contains(&name) {
            continue;
        }
        if let Some(value) = bindings.get(&name).cloned() {
            env.insert(name, value);
        }
    }

    match control {
        EvalControl::Continue => Ok(Value::Undefined),
        EvalControl::Return(value) => Ok(value),
        EvalControl::Break => Err(ScriptError::new("break outside loop")),
        EvalControl::ContinueLoop => Err(ScriptError::new("continue outside loop")),
    }
}

fn eval_call_argument_values<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Vec<Value>> {
    let mut values = Vec::new();
    for arg in args {
        match arg {
            Expr::Spread(expr) => {
                let value = eval_expr(expr, env, host)?;
                values.extend(array_from_value(value, env, host)?);
            }
            expr => values.push(eval_expr(expr, env, host)?),
        }
    }
    Ok(values)
}

const NATIVE_METHOD_PREFIX: &str = "__native_method__:";
const NATIVE_METHOD_RECEIVER_KEY: &str = "__native_method_receiver";
const NATIVE_GLOBAL_FUNCTION_PREFIX: &str = "__native_global_function__:";
const HTML_CONSTRUCTOR_PREFIX: &str = "__html_constructor__:";
const HTML_NAMESPACE_URI: &str = "http://www.w3.org/1999/xhtml";

fn native_method_function(receiver: Value, method: &str) -> Value {
    let mut captured_bindings = BTreeMap::new();
    captured_bindings.insert(NATIVE_METHOD_RECEIVER_KEY.to_string(), receiver);
    Value::Function(
        crate::ScriptFunction::new(Vec::new(), format!("{NATIVE_METHOD_PREFIX}{method}"))
            .with_captured_bindings(captured_bindings),
    )
}

fn native_global_function(name: &str) -> Value {
    Value::Function(crate::ScriptFunction::new(
        Vec::new(),
        format!("{NATIVE_GLOBAL_FUNCTION_PREFIX}{name}"),
    ))
}

fn html_constructor_function(name: &str) -> Value {
    Value::Function(crate::ScriptFunction::new(
        Vec::new(),
        format!("{HTML_CONSTRUCTOR_PREFIX}{name}"),
    ))
}

fn html_constructor_name(function: &crate::ScriptFunction) -> Option<&str> {
    function.body_source.strip_prefix(HTML_CONSTRUCTOR_PREFIX)
}

fn native_global_function_name(function: &crate::ScriptFunction) -> Option<&str> {
    function
        .body_source
        .strip_prefix(NATIVE_GLOBAL_FUNCTION_PREFIX)
}

fn is_html_namespace_element<H: HostBindings>(
    element: &ElementHandle,
    host: &mut H,
) -> Result<bool> {
    Ok(matches!(
        host.node_namespace_uri(NodeHandle::new(element.raw()))?,
        Some(namespace) if namespace == HTML_NAMESPACE_URI
    ))
}

fn html_element_tag_matches<H: HostBindings>(
    element: &ElementHandle,
    expected: &[&str],
    host: &mut H,
) -> Result<bool> {
    if !is_html_namespace_element(element, host)? {
        return Ok(false);
    }
    Ok(expected.iter().any(|tag| {
        host.element_tag_name(*element)
            .is_ok_and(|actual| actual == *tag)
    }))
}

fn is_instance_of_constructor<H: HostBindings>(
    value: &Value,
    constructor: &Value,
    host: &mut H,
) -> Result<bool> {
    match constructor {
        Value::Function(function) => {
            let Some(name) = html_constructor_name(function) else {
                return Ok(false);
            };
            match name {
                "Node" => Ok(matches!(
                    value,
                    Value::Document
                        | Value::Element(_)
                        | Value::Node(_)
                        | Value::TemplateContent(_)
                        | Value::Attribute(_)
                )),
                "Element" => Ok(matches!(value, Value::Element(_))),
                "HTMLElement" => match value {
                    Value::Element(element) => is_html_namespace_element(element, host),
                    _ => Ok(false),
                },
                "HTMLButtonElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["button"], host),
                    _ => Ok(false),
                },
                "HTMLSelectElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["select"], host),
                    _ => Ok(false),
                },
                "HTMLInputElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["input"], host),
                    _ => Ok(false),
                },
                "HTMLTextAreaElement" => match value {
                    Value::Element(element) => {
                        html_element_tag_matches(element, &["textarea"], host)
                    }
                    _ => Ok(false),
                },
                "HTMLFormElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["form"], host),
                    _ => Ok(false),
                },
                "HTMLOptionElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["option"], host),
                    _ => Ok(false),
                },
                "HTMLOptGroupElement" => match value {
                    Value::Element(element) => {
                        html_element_tag_matches(element, &["optgroup"], host)
                    }
                    _ => Ok(false),
                },
                "HTMLFieldSetElement" => match value {
                    Value::Element(element) => {
                        html_element_tag_matches(element, &["fieldset"], host)
                    }
                    _ => Ok(false),
                },
                "HTMLLabelElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["label"], host),
                    _ => Ok(false),
                },
                "HTMLImageElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["img"], host),
                    _ => Ok(false),
                },
                "HTMLAnchorElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["a"], host),
                    _ => Ok(false),
                },
                "HTMLAreaElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["area"], host),
                    _ => Ok(false),
                },
                "HTMLMapElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["map"], host),
                    _ => Ok(false),
                },
                "HTMLTableElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["table"], host),
                    _ => Ok(false),
                },
                "HTMLTableSectionElement" => match value {
                    Value::Element(element) => {
                        html_element_tag_matches(element, &["thead", "tbody", "tfoot"], host)
                    }
                    _ => Ok(false),
                },
                "HTMLTableRowElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["tr"], host),
                    _ => Ok(false),
                },
                "HTMLTableCellElement" => match value {
                    Value::Element(element) => {
                        html_element_tag_matches(element, &["td", "th"], host)
                    }
                    _ => Ok(false),
                },
                "HTMLUListElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["ul"], host),
                    _ => Ok(false),
                },
                "HTMLOListElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["ol"], host),
                    _ => Ok(false),
                },
                "HTMLLIElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["li"], host),
                    _ => Ok(false),
                },
                "HTMLObjectElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["object"], host),
                    _ => Ok(false),
                },
                "HTMLEmbedElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["embed"], host),
                    _ => Ok(false),
                },
                "HTMLLegendElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["legend"], host),
                    _ => Ok(false),
                },
                "HTMLDListElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["dl"], host),
                    _ => Ok(false),
                },
                "HTMLScriptElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["script"], host),
                    _ => Ok(false),
                },
                "HTMLStyleElement" => match value {
                    Value::Element(element) => html_element_tag_matches(element, &["style"], host),
                    _ => Ok(false),
                },
                _ => Ok(false),
            }
        }
        Value::ObjectNamespace => Ok(!matches!(
            value,
            Value::Undefined
                | Value::Null
                | Value::Boolean(_)
                | Value::Number(_)
                | Value::String(_)
        )),
        Value::ArrayNamespace => Ok(matches!(value, Value::Array(_))),
        _ => Err(ScriptError::new(
            "right-hand side of instanceof is not a constructor",
        )),
    }
}

fn call_script_function_value<H: HostBindings>(
    function: &crate::ScriptFunction,
    args: &[Value],
    this_value: Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if html_constructor_name(function).is_some() {
        return Err(ScriptError::new("invalid function target"));
    }
    if let Some(name) = native_global_function_name(function) {
        return call_native_global_function(name, args, env, host);
    }
    if let Some(method) = function.body_source.strip_prefix(NATIVE_METHOD_PREFIX) {
        return call_native_function(function, method, args, this_value, env, host);
    }

    let mut bindings = function.captured_bindings.as_ref().clone();
    bindings.extend(env.clone());
    let outer_names: Vec<String> = env.keys().cloned().collect();
    let bound_names = function_bound_names(function);

    bindings.insert("this".to_string(), this_value);

    let program = crate::parser::parse_program(&function.body_source)?;
    let local_names = collect_declared_names_from_program(&program);
    bind_function_arguments_from_values(function, args, env, host, &mut bindings)?;
    let control = eval_program_with_bindings(&program, host, &mut bindings)?;

    for name in outer_names {
        if name == "this" || bound_names.contains(&name) || local_names.contains(&name) {
            continue;
        }
        if let Some(value) = bindings.get(&name).cloned() {
            env.insert(name, value);
        }
    }

    match control {
        EvalControl::Continue => Ok(Value::Undefined),
        EvalControl::Return(value) => Ok(value),
        EvalControl::Break => Err(ScriptError::new("break outside loop")),
        EvalControl::ContinueLoop => Err(ScriptError::new("continue outside loop")),
    }
}

fn call_native_global_function<H: HostBindings>(
    name: &str,
    args: &[Value],
    _env: &mut BTreeMap<String, Value>,
    _host: &mut H,
) -> Result<Value> {
    match name {
        "Boolean" => {
            let value = args.first().cloned().unwrap_or(Value::Undefined);
            Ok(Value::Boolean(is_truthy(&value)))
        }
        "decodeURI" => {
            let [value] = args else {
                return Err(ScriptError::new("decodeURI() expects exactly one argument"));
            };
            Ok(Value::String(percent_decode_uri(&as_string(value), true)))
        }
        "decodeURIComponent" => {
            let [value] = args else {
                return Err(ScriptError::new(
                    "decodeURIComponent() expects exactly one argument",
                ));
            };
            Ok(Value::String(percent_decode_uri(&as_string(value), false)))
        }
        "encodeURI" => {
            let [value] = args else {
                return Err(ScriptError::new("encodeURI() expects exactly one argument"));
            };
            Ok(Value::String(percent_encode_uri(&as_string(value), true)))
        }
        "encodeURIComponent" => {
            let [value] = args else {
                return Err(ScriptError::new(
                    "encodeURIComponent() expects exactly one argument",
                ));
            };
            Ok(Value::String(percent_encode_uri(&as_string(value), false)))
        }
        other => Err(ScriptError::new(format!(
            "unsupported native global function: {other}"
        ))),
    }
}

fn collect_declared_names_from_program(program: &Program) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_declared_names_from_statements(&program.statements, &mut names);
    names
}

fn collect_declared_names_from_statements(statements: &[Statement], names: &mut BTreeSet<String>) {
    for statement in statements {
        collect_declared_names_from_statement(statement, names);
    }
}

fn collect_declared_names_from_statement(statement: &Statement, names: &mut BTreeSet<String>) {
    match statement {
        Statement::VariableDeclaration { name, .. } => {
            names.insert(name.clone());
        }
        Statement::FunctionDeclaration { name, .. } => {
            names.insert(name.clone());
        }
        Statement::If {
            then_branch,
            else_branch,
            ..
        } => {
            collect_declared_names_from_statements(then_branch, names);
            if let Some(else_branch) = else_branch {
                collect_declared_names_from_statements(else_branch, names);
            }
        }
        Statement::While { body, .. } => {
            collect_declared_names_from_statements(body, names);
        }
        Statement::For { init, body, .. } => {
            if let Some(init) = init.as_ref() {
                collect_declared_names_from_statement(init, names);
            }
            collect_declared_names_from_statements(body, names);
        }
        Statement::ForIn { binding, body, .. } | Statement::ForOf { binding, body, .. } => {
            names.insert(binding.clone());
            collect_declared_names_from_statements(body, names);
        }
        Statement::TryCatch {
            try_body,
            catch_binding,
            catch_body,
        } => {
            names.insert(catch_binding.clone());
            collect_declared_names_from_statements(try_body, names);
            collect_declared_names_from_statements(catch_body, names);
        }
        Statement::Return(_)
        | Statement::Break
        | Statement::Continue
        | Statement::Throw(_)
        | Statement::Assignment { .. }
        | Statement::Expression(_) => {}
    }
}

fn function_bound_names(function: &crate::ScriptFunction) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for param in &function.params {
        if let Some(expanded) = decode_array_destructure_param(param) {
            names.extend(expanded);
        } else if let Some(expanded) = decode_object_destructure_param(param) {
            names.extend(expanded);
        } else {
            names.insert(param.clone());
        }
    }
    names
}

fn decode_array_destructure_param(param: &str) -> Option<Vec<String>> {
    param.strip_prefix("__array_destructure__:").map(|names| {
        if names.is_empty() {
            Vec::new()
        } else {
            names
                .split(',')
                .filter_map(|name| {
                    let name = name.trim();
                    if name.is_empty() {
                        None
                    } else {
                        Some(name.to_string())
                    }
                })
                .collect()
        }
    })
}

fn decode_object_destructure_param(param: &str) -> Option<Vec<String>> {
    param.strip_prefix("__object_destructure__:").map(|names| {
        if names.is_empty() {
            Vec::new()
        } else {
            names
                .split(',')
                .filter_map(|name| {
                    let name = name.trim();
                    if name.is_empty() {
                        None
                    } else {
                        Some(name.to_string())
                    }
                })
                .collect()
        }
    })
}

fn bind_function_arguments_from_exprs<H: HostBindings>(
    function: &crate::ScriptFunction,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    bindings: &mut BTreeMap<String, Value>,
) -> Result<()> {
    let mut values = Vec::with_capacity(args.len());
    for arg in args {
        values.push(eval_expr(arg, env, host)?);
    }
    bind_function_arguments_from_values(function, &values, env, host, bindings)
}

fn bind_function_arguments_from_values<H: HostBindings>(
    function: &crate::ScriptFunction,
    args: &[Value],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    bindings: &mut BTreeMap<String, Value>,
) -> Result<()> {
    for (index, param) in function.params.iter().enumerate() {
        let value = args.get(index).cloned().unwrap_or(Value::Undefined);
        if let Some(names) = decode_array_destructure_param(param) {
            let items = array_from_value(value, env, host)?;
            for (item_index, name) in names.into_iter().enumerate() {
                bindings.insert(
                    name,
                    items.get(item_index).cloned().unwrap_or(Value::Undefined),
                );
            }
        } else if let Some(names) = decode_object_destructure_param(param) {
            for name in names {
                let key = property_key_from_string(name.clone());
                let bound =
                    property_value_on_value(&value, &key, env, host)?.unwrap_or(Value::Undefined);
                bindings.insert(name, bound);
            }
        } else {
            bindings.insert(param.clone(), value);
        }
    }
    Ok(())
}

fn call_native_function<H: HostBindings>(
    function: &crate::ScriptFunction,
    method: &str,
    args: &[Value],
    this_value: Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let receiver = function
        .captured_bindings
        .get(NATIVE_METHOD_RECEIVER_KEY)
        .cloned()
        .unwrap_or(this_value);

    let mut native_env = env.clone();
    let fake_args: Vec<Expr> = args
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let name = format!("__native_arg_{index}");
            native_env.insert(name.clone(), value.clone());
            Expr::Identifier(name)
        })
        .collect();

    eval_method_call(receiver, method, &fake_args, &mut native_env, host)
}

fn timer_callback_from_expr<H: HostBindings>(
    expr: &Expr,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<crate::ScriptFunction> {
    match eval_expr(expr, env, host)? {
        Value::Function(function) => Ok(function),
        _ => Err(ScriptError::new(format!(
            "{method}() expects a function callback"
        ))),
    }
}

fn timer_delay_from_expr<H: HostBindings>(
    expr: Option<&Expr>,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<i64> {
    let Some(expr) = expr else {
        return Ok(0);
    };
    let delay = eval_expr(expr, env, host)?;
    match delay {
        Value::Number(number) if number.is_finite() => Ok(number as i64),
        Value::String(text) => text
            .trim()
            .parse::<i64>()
            .map_err(|_| ScriptError::new(format!("{method}() expects a numeric delay"))),
        Value::Boolean(true) => Ok(1),
        Value::Boolean(false) => Ok(0),
        _ => Err(ScriptError::new(format!(
            "{method}() expects a numeric delay"
        ))),
    }
}

fn timer_handle_from_expr<H: HostBindings>(
    expr: &Expr,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<u64> {
    match eval_expr(expr, env, host)? {
        Value::Number(number) if number.is_finite() && number >= 0.0 => Ok(number.trunc() as u64),
        Value::String(text) => text
            .trim()
            .parse::<u64>()
            .map_err(|_| ScriptError::new(format!("{method}() expects a timer id"))),
        _ => Err(ScriptError::new(format!("{method}() expects a timer id"))),
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

fn percent_decode_uri(value: &str, preserve_reserved: bool) -> String {
    let reserved = b";/?:@&=+$,#";
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0usize;

    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
            {
                let byte = (hi << 4) | lo;
                if preserve_reserved && reserved.contains(&byte) {
                    out.extend_from_slice(&bytes[index..index + 3]);
                } else {
                    out.push(byte);
                }
                index += 3;
                continue;
            }
        }

        out.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn percent_encode_uri(value: &str, preserve_reserved: bool) -> String {
    let reserved = ";/?:@&=+$,#";
    let mut out = String::new();
    for ch in value.chars() {
        let keep = matches!(
            ch,
            'A'..='Z'
                | 'a'..='z'
                | '0'..='9'
                | '-'
                | '_'
                | '.'
                | '!'
                | '~'
                | '*'
                | '\''
                | '('
                | ')'
        ) || (preserve_reserved && reserved.contains(ch));
        if keep {
            out.push(ch);
            continue;
        }

        let mut buf = [0u8; 4];
        for byte in ch.encode_utf8(&mut buf).as_bytes() {
            out.push('%');
            out.push_str(&format!("{byte:02X}"));
        }
    }
    out
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
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
            "createElement" => {
                let [tag_expr] = args else {
                    return Err(ScriptError::new(
                        "document.createElement() expects exactly one argument",
                    ));
                };
                let tag_name = as_string(&eval_expr(tag_expr, env, host)?);
                Ok(Value::Element(host.document_create_element(&tag_name)?))
            }
            "createElementNS" => {
                let [namespace_expr, tag_expr] = args else {
                    return Err(ScriptError::new(
                        "document.createElementNS() expects exactly two arguments",
                    ));
                };
                let namespace_uri = as_string(&eval_expr(namespace_expr, env, host)?);
                let tag_name = as_string(&eval_expr(tag_expr, env, host)?);
                Ok(Value::Element(
                    host.document_create_element_ns(&namespace_uri, &tag_name)?,
                ))
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
            "createAttribute" => {
                let [qualified_name_expr] = args else {
                    return Err(ScriptError::new(
                        "document.createAttribute() expects exactly one argument",
                    ));
                };
                let qualified_name = as_string(&eval_expr(qualified_name_expr, env, host)?);
                create_attribute_value(None, &qualified_name)
            }
            "createAttributeNS" => {
                let [namespace_expr, qualified_name_expr] = args else {
                    return Err(ScriptError::new(
                        "document.createAttributeNS() expects exactly two arguments",
                    ));
                };
                let namespace_uri =
                    namespace_uri_from_value(&eval_expr(namespace_expr, env, host)?);
                let qualified_name = as_string(&eval_expr(qualified_name_expr, env, host)?);
                create_attribute_value(namespace_uri.as_deref(), &qualified_name)
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
            "removeChild" => node_remove_child(NodeHandle::new(0), args, env, host),
            "open" => document_open(args, env, host),
            "close" => document_close(args, env, host),
            "write" => document_write(args, env, host),
            "writeln" => document_writeln(args, env, host),
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
            "requestAnimationFrame" => {
                let [callback_expr, ..] = args else {
                    return Err(ScriptError::new(
                        "requestAnimationFrame() expects at least one argument",
                    ));
                };
                let callback =
                    timer_callback_from_expr(callback_expr, env, host, "requestAnimationFrame")?;
                Ok(Value::Number(
                    host.window_request_animation_frame(callback)? as f64,
                ))
            }
            "cancelAnimationFrame" => {
                if args.is_empty() {
                    return Err(ScriptError::new(
                        "cancelAnimationFrame() expects at least one argument",
                    ));
                }
                let handle = timer_handle_from_expr(&args[0], env, host, "cancelAnimationFrame")?;
                host.window_cancel_animation_frame(handle)?;
                Ok(Value::Undefined)
            }
            "setTimeout" => {
                let [callback_expr, ..] = args else {
                    return Err(ScriptError::new(
                        "setTimeout() expects at least one argument",
                    ));
                };
                let callback = timer_callback_from_expr(callback_expr, env, host, "setTimeout")?;
                let delay_ms = timer_delay_from_expr(args.get(1), env, host, "setTimeout")?;
                Ok(Value::Number(
                    host.window_set_timeout(callback, delay_ms)? as f64
                ))
            }
            "setInterval" => {
                let [callback_expr, ..] = args else {
                    return Err(ScriptError::new(
                        "setInterval() expects at least one argument",
                    ));
                };
                let callback = timer_callback_from_expr(callback_expr, env, host, "setInterval")?;
                let delay_ms = timer_delay_from_expr(args.get(1), env, host, "setInterval")?;
                Ok(Value::Number(
                    host.window_set_interval(callback, delay_ms)? as f64
                ))
            }
            "clearTimeout" => {
                if args.is_empty() {
                    return Err(ScriptError::new(
                        "clearTimeout() expects at least one argument",
                    ));
                }
                let handle = timer_handle_from_expr(&args[0], env, host, "clearTimeout")?;
                host.window_clear_timeout(handle)?;
                Ok(Value::Undefined)
            }
            "clearInterval" => {
                if args.is_empty() {
                    return Err(ScriptError::new(
                        "clearInterval() expects at least one argument",
                    ));
                }
                let handle = timer_handle_from_expr(&args[0], env, host, "clearInterval")?;
                host.window_clear_interval(handle)?;
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
            "click" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("click() expects no arguments"));
                }
                host.element_click(element)?;
                Ok(Value::Undefined)
            }
            "focus" => {
                if args.len() > 1 {
                    return Err(ScriptError::new("focus() expects at most one argument"));
                }
                host.element_focus(element)?;
                Ok(Value::Undefined)
            }
            "blur" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("blur() expects no arguments"));
                }
                host.element_blur(element)?;
                Ok(Value::Undefined)
            }
            "getAttribute" => element_get_attribute(element, args, env, host),
            "setAttribute" => element_set_attribute(element, args, env, host),
            "removeAttribute" => element_remove_attribute(element, args, env, host),
            "hasAttribute" => element_has_attribute(element, args, env, host),
            "toggleAttribute" => element_toggle_attribute(element, args, env, host),
            "getAttributeNode" => element_get_attribute_node(element, args, env, host),
            "getAttributeNodeNS" => element_get_attribute_node_ns(element, args, env, host),
            "setAttributeNode" => element_set_attribute_node(element, args, env, host),
            "setAttributeNodeNS" => element_set_attribute_node_ns(element, args, env, host),
            "removeAttributeNode" => element_remove_attribute_node(element, args, env, host),
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
            "removeChild" => node_remove_child(NodeHandle::new(element.raw()), args, env, host),
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
            "removeChild" => node_remove_child(NodeHandle::new(element.raw()), args, env, host),
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
        Value::HtmlCollection(collection) => {
            if matches!(collection, HtmlCollectionTarget::DocumentPlugins) && method == "refresh" {
                return plugins_refresh(args);
            }

            match method {
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
            }
        }
        Value::NamedNodeMap(element) => match method {
            "item" => named_node_map_item(&element, args, env, host),
            "getNamedItem" => named_node_map_get_named_item(&element, args, env, host),
            "getNamedItemNS" => named_node_map_get_named_item_ns(&element, args, env, host),
            "setNamedItem" => named_node_map_set_named_item(&element, args, env, host),
            "setNamedItemNS" => named_node_map_set_named_item_ns(&element, args, env, host),
            "removeNamedItem" => named_node_map_remove_named_item(&element, args, env, host),
            "removeNamedItemNS" => named_node_map_remove_named_item_ns(&element, args, env, host),
            "keys" => named_node_map_keys(&element, host),
            "values" => named_node_map_values(&element, host),
            "entries" => named_node_map_entries(&element, host),
            "forEach" => named_node_map_for_each(&element, args, env, host),
            "toString" => collection_to_string("NamedNodeMap", args),
            other => Err(ScriptError::new(format!(
                "unsupported NamedNodeMap method: {other}"
            ))),
        },
        Value::StyleSheetList(target) => match method {
            "item" => style_sheet_list_item(&target, args, env, host),
            "namedItem" => style_sheet_list_named_item(&target, args, env, host),
            "forEach" => style_sheet_list_for_each(&target, args, env, host),
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
        Value::Clipboard => match method {
            "writeText" => {
                let [text_expr] = args else {
                    return Err(ScriptError::new(
                        "navigator.clipboard.writeText() expects exactly one argument",
                    ));
                };
                let text = as_string(&eval_expr(text_expr, env, host)?);
                host.clipboard_write_text(&text)?;
                Ok(Value::Undefined)
            }
            "readText" => {
                if !args.is_empty() {
                    return Err(ScriptError::new(
                        "navigator.clipboard.readText() expects no arguments",
                    ));
                }
                Ok(Value::String(host.clipboard_read_text()?))
            }
            other => Err(ScriptError::new(format!(
                "cannot call `{other}` on a clipboard value"
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
        Value::String(value) => match method {
            "trim" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("trim() expects no arguments"));
                }
                Ok(Value::String(value.trim().to_string()))
            }
            "toString" | "valueOf" => {
                if !args.is_empty() {
                    return Err(ScriptError::new(format!("{method}() expects no arguments")));
                }
                Ok(Value::String(value))
            }
            "split" => string_split(&value, args, env, host),
            "replace" => string_replace(&value, args, false, env, host),
            "replaceAll" => string_replace(&value, args, true, env, host),
            "match" => string_match(&value, args, env, host),
            "search" => string_search(&value, args, env, host),
            "toLowerCase" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("toLowerCase() expects no arguments"));
                }
                Ok(Value::String(value.to_lowercase()))
            }
            "toUpperCase" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("toUpperCase() expects no arguments"));
                }
                Ok(Value::String(value.to_uppercase()))
            }
            "includes" => string_contains(&value, args, env, host),
            "startsWith" => string_starts_with(&value, args, env, host),
            "endsWith" => string_ends_with(&value, args, env, host),
            "indexOf" => string_index_of(&value, args, env, host),
            "lastIndexOf" => string_last_index_of(&value, args, env, host),
            "slice" => string_slice(&value, args, env, host),
            "substring" => string_substring(&value, args, env, host),
            "charAt" => string_char_at(&value, args, env, host),
            "charCodeAt" => string_char_code_at(&value, args, env, host),
            "concat" => string_concat(&value, args, env, host),
            "repeat" => string_repeat(&value, args, env, host),
            "padStart" => string_pad_start(&value, args, env, host),
            "normalize" => string_normalize(&value, args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported string method: {other}"
            ))),
        },
        Value::RegExp(value) => match method {
            "test" => {
                let [arg] = args else {
                    return Err(ScriptError::new("test() expects exactly one argument"));
                };
                let text = as_string(&eval_expr(arg, env, host)?);
                regex_test(&text, &value)
            }
            "exec" => {
                let [arg] = args else {
                    return Err(ScriptError::new("exec() expects exactly one argument"));
                };
                let text = as_string(&eval_expr(arg, env, host)?);
                regex_exec(&text, &value)
            }
            "toString" | "valueOf" => {
                if !args.is_empty() {
                    return Err(ScriptError::new(format!("{method}() expects no arguments")));
                }
                Ok(Value::String(as_string(&Value::RegExp(value.clone()))))
            }
            other => Err(ScriptError::new(format!(
                "unsupported regexp method: {other}"
            ))),
        },
        Value::Date(date) => match method {
            "toLocaleDateString" => date_to_locale_date_string(&date, args, env, host),
            "toLocaleString" => date_to_locale_string(&date, args, env, host),
            "toISOString" => date_to_iso_string_method(&date, args, env, host),
            "toJSON" => date_to_json(&date, args, env, host),
            "toString" => date_to_string(&date, args, env, host),
            "valueOf" | "getTime" => date_value_of(&date, args),
            "getFullYear" => date_get_year(&date, args, false, false),
            "getUTCFullYear" => date_get_year(&date, args, true, false),
            "getMonth" => date_get_month(&date, args, false),
            "getUTCMonth" => date_get_month(&date, args, true),
            "getDate" => date_get_day(&date, args, false),
            "getUTCDate" => date_get_day(&date, args, true),
            "getHours" => date_get_hours(&date, args, false),
            "getUTCHours" => date_get_hours(&date, args, true),
            "getMinutes" => date_get_minutes(&date, args, false),
            "getUTCMinutes" => date_get_minutes(&date, args, true),
            "getSeconds" => date_get_seconds(&date, args, false),
            "getUTCSeconds" => date_get_seconds(&date, args, true),
            "getMilliseconds" => date_get_milliseconds(&date, args, false),
            "getUTCMilliseconds" => date_get_milliseconds(&date, args, true),
            "getTimezoneOffset" => date_get_timezone_offset(&date, args),
            other => Err(ScriptError::new(format!(
                "unsupported Date method: {other}"
            ))),
        },
        Value::IntlNumberFormat(formatter) => match method {
            "format" => intl_number_format_format(&formatter, args, env, host),
            "resolvedOptions" => intl_number_format_resolved_options(&formatter, args, env, host),
            "formatToParts" => intl_number_format_to_parts(&formatter, args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported Intl.NumberFormat method: {other}"
            ))),
        },
        Value::IntlDateTimeFormat(formatter) => match method {
            "format" => intl_date_time_format_format(&formatter, args, env, host),
            "resolvedOptions" => {
                intl_date_time_format_resolved_options(&formatter, args, env, host)
            }
            "formatToParts" => intl_date_time_format_to_parts(&formatter, args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported Intl.DateTimeFormat method: {other}"
            ))),
        },
        Value::IntlCollator(collator) => match method {
            "compare" => intl_collator_compare(&collator, args, env, host),
            "resolvedOptions" => intl_collator_resolved_options(&collator, args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported Intl.Collator method: {other}"
            ))),
        },
        Value::ObjectNamespace => match method {
            "assign" => object_namespace_assign(args, env, host),
            "keys" => object_namespace_keys(args, env, host),
            "values" => object_namespace_values(args, env, host),
            "entries" => object_namespace_entries(args, env, host),
            "fromEntries" => object_namespace_from_entries(args, env, host),
            "getOwnPropertySymbols" => object_namespace_get_own_property_symbols(args, env, host),
            "fromCharCode" => string_namespace_from_char_code(args, env, host),
            "isFinite" => number_namespace_is_finite(args, env, host),
            "isNaN" => number_namespace_is_nan(args, env, host),
            "parseFloat" => number_namespace_parse_float(args, env, host),
            "parseInt" => number_namespace_parse_int(args, env, host),
            "now" => date_namespace_now(args, env, host),
            "UTC" => date_namespace_utc(args, env, host),
            "parse" => date_namespace_parse(args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported Object method: {other}"
            ))),
        },
        Value::ArrayNamespace => match method {
            "isArray" => array_namespace_is_array(args, env, host),
            "from" => array_namespace_from(args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported Array method: {other}"
            ))),
        },
        Value::Function(function) if method == "call" => {
            let this_value = match args.first() {
                Some(expr) => eval_expr(expr, env, host)?,
                None => Value::Undefined,
            };
            let values = eval_call_argument_values(&args[1..], env, host)?;
            call_script_function_value(&function, &values, this_value, env, host)
        }
        Value::Object(object) => match method {
            "hasOwnProperty" => {
                let [key_expr] = args else {
                    return Err(ScriptError::new(
                        "hasOwnProperty() expects exactly one argument",
                    ));
                };
                let key = property_key_from_value(&eval_expr(key_expr, env, host)?);
                Ok(Value::Boolean(
                    property_value_on_value(&Value::Object(object.clone()), &key, env, host)?
                        .is_some(),
                ))
            }
            "toString" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("toString() expects no arguments"));
                }
                Ok(Value::String("[object Object]".to_string()))
            }
            "valueOf" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("valueOf() expects no arguments"));
                }
                Ok(Value::Object(object))
            }
            other => match property_value_on_value(
                &Value::Object(object.clone()),
                &property_key_from_string(other),
                env,
                host,
            )? {
                Some(Value::Function(function)) => {
                    let values = args
                        .iter()
                        .map(|expr| eval_expr(expr, env, host))
                        .collect::<Result<Vec<_>>>()?;
                    call_script_function_value(&function, &values, Value::Object(object), env, host)
                }
                Some(_) | None => Err(ScriptError::new(format!(
                    "unsupported Object instance method: {other}"
                ))),
            },
        },
        Value::Array(array) => match method {
            "join" => array_join(&array, args, env, host),
            "map" => array_map(&array, args, env, host),
            "filter" => array_filter(&array, args, env, host),
            "forEach" => array_for_each(&array, args, env, host),
            "some" => array_some(&array, args, env, host),
            "every" => array_every(&array, args, env, host),
            "subarray" => array_slice(&array, args, env, host),
            "flatMap" => array_flat_map(&array, args, env, host),
            "flat" => array_flat(&array, args, env, host),
            "fill" => array_fill(&array, args, env, host),
            "push" => array_push(&array, args, env, host),
            "pop" => array_pop(&array, args, env, host),
            "slice" => array_slice(&array, args, env, host),
            "concat" => array_concat(&array, args, env, host),
            "sort" => array_sort(&array, args, env, host),
            "includes" => array_includes(&array, args, env, host),
            "indexOf" => array_index_of(&array, args, env, host, false),
            "lastIndexOf" => array_index_of(&array, args, env, host, true),
            "reverse" => array_reverse(&array, args, env, host),
            "reduce" => array_reduce(&array, args, env, host),
            "toString" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("toString() expects no arguments"));
                }
                Ok(Value::String(array_to_string(&array, ",", None)))
            }
            other => Err(ScriptError::new(format!(
                "unsupported Array instance method: {other}"
            ))),
        },
        Value::Map(map) => match method {
            "get" => map_get(&map, args, env, host),
            "set" => map_set(&map, args, env, host),
            "has" => map_has(&map, args, env, host),
            "delete" => map_delete(&map, args, env, host),
            "clear" => map_clear(&map, args, env, host),
            "keys" => map_keys(&map, host),
            "values" => map_values(&map, host),
            "entries" => map_entries(&map, host),
            "forEach" => map_for_each(&map, args, env, host),
            "toString" => {
                if !args.is_empty() {
                    return Err(ScriptError::new("toString() expects no arguments"));
                }
                Ok(Value::String("[object Map]".to_string()))
            }
            other => Err(ScriptError::new(format!(
                "unsupported Map instance method: {other}"
            ))),
        },
        Value::IntlNamespace => match method {
            "NumberFormat" => Err(ScriptError::new(
                "Intl.NumberFormat() is not yet supported in this runtime",
            )),
            "DateTimeFormat" => Err(ScriptError::new(
                "Intl.DateTimeFormat() is not yet supported in this runtime",
            )),
            "min" => math_namespace_min(args, env, host),
            "max" => math_namespace_max(args, env, host),
            "abs" => math_namespace_abs(args, env, host),
            "floor" => math_namespace_floor(args, env, host),
            "ceil" => math_namespace_ceil(args, env, host),
            "round" => math_namespace_round(args, env, host),
            "trunc" => math_namespace_trunc(args, env, host),
            "sign" => math_namespace_sign(args, env, host),
            "pow" => math_namespace_pow(args, env, host),
            "sqrt" => math_namespace_sqrt(args, env, host),
            "escape" => css_namespace_escape(args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported Intl method: {other}"
            ))),
        },
        Value::Symbol(_) => Err(ScriptError::new(format!(
            "cannot call `{method}` on a symbol value"
        ))),
        Value::ClassList(element) => match method {
            "item" => class_list_item(element, args, env, host),
            "contains" => class_list_contains(element, args, env, host),
            "forEach" => class_list_for_each(element, args, env, host),
            "keys" => class_list_keys(element, host),
            "values" => class_list_values(element, host),
            "entries" => class_list_entries(element, host),
            "add" => class_list_add(element, args, env, host),
            "remove" => class_list_remove(element, args, env, host),
            "replace" => class_list_replace(element, args, env, host),
            "toggle" => class_list_toggle(element, args, env, host),
            "toString" => class_list_to_string(element, args, host),
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
        Value::MediaQueryList(list) => match method {
            "addListener" => media_query_list_add_listener(&list, args, env, host),
            "removeListener" => media_query_list_remove_listener(&list, args, env, host),
            other => Err(ScriptError::new(format!(
                "cannot call `{other}` on a media query list value"
            ))),
        },
        Value::StringList(list) => match method {
            "item" => string_list_item(&list, args, env, host),
            "contains" => string_list_contains(&list, args, env, host),
            "forEach" => string_list_for_each(&list, args, env, host),
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
            "forEach" => mime_type_array_for_each(&list, args, env, host),
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
        Value::Attribute(_) => match method {
            "toString" => collection_to_string("Attr", args),
            other => Err(ScriptError::new(format!(
                "cannot call `{other}` on an attr value"
            ))),
        },
        Value::IteratorResult(_) => Err(ScriptError::new(format!(
            "cannot call `{method}` on an iterator result value"
        ))),
        Value::Number(number) => match method {
            "toFixed" => number_method_to_fixed(number, args, env, host),
            "toExponential" => number_method_to_exponential(number, args, env, host),
            "toPrecision" => number_method_to_precision(number, args, env, host),
            "toString" => number_method_to_string(number, args, env, host),
            other => Err(ScriptError::new(format!(
                "unsupported method call on number value: {other}"
            ))),
        },
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
                "addEventListener() requires a function callback",
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

fn style_sheet_list_for_each<H: HostBindings>(
    target: &StyleSheetListTarget,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new(
                "StyleSheetList.forEach() expects one or two arguments",
            ));
        }
    };

    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => {
            return Err(ScriptError::new(
                "StyleSheetList.forEach() requires an arrow function callback",
            ));
        }
    };
    if let Some(this_arg_expr) = this_arg_expr {
        let _ = eval_expr(this_arg_expr, env, host)?;
    }

    let items = style_sheet_list_items(target, host)?
        .into_iter()
        .map(|handle| Value::StyleSheet(StyleSheetTarget::OwnerNode(handle)))
        .collect();
    let collection_value = Value::StyleSheetList(target.clone());
    for_each_over_items(&callback, items, collection_value, env, host)
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
        HtmlCollectionTarget::DocumentPlugins => {
            host.html_collection_tag_name_items(HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "embed".to_string(),
            })
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
        HtmlCollectionTarget::DocumentPlugins => host
            .html_collection_tag_name_named_item(
                HtmlCollectionTarget::ByTagName {
                    scope: HtmlCollectionScope::Document,
                    tag_name: "embed".to_string(),
                },
                name,
            )
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

fn string_list_for_each<H: HostBindings>(
    list: &StringListState,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new(
                "navigator.languages.forEach() expects one or two arguments",
            ));
        }
    };

    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => {
            return Err(ScriptError::new(
                "navigator.languages.forEach() requires an arrow function callback",
            ));
        }
    };
    if let Some(this_arg_expr) = this_arg_expr {
        let _ = eval_expr(this_arg_expr, env, host)?;
    }

    let items = list.items().iter().cloned().map(Value::String).collect();
    let collection_value = Value::StringList(list.clone());
    for_each_over_items(&callback, items, collection_value, env, host)
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

fn namespace_uri_from_value(value: &Value) -> Option<String> {
    match value {
        Value::Null | Value::Undefined => None,
        _ => {
            let text = as_string(value);
            if text.is_empty() { None } else { Some(text) }
        }
    }
}

fn create_attribute_value(namespace_uri: Option<&str>, qualified_name: &str) -> Result<Value> {
    let qualified_name = qualified_name.trim();
    if qualified_name.is_empty() {
        return Err(ScriptError::new("attribute name must not be empty"));
    }

    if let Some(colon) = qualified_name.find(':') {
        if colon == 0 || colon + 1 >= qualified_name.len() {
            return Err(ScriptError::new(format!(
                "invalid qualified attribute name: `{qualified_name}`"
            )));
        }
        if qualified_name[colon + 1..].contains(':') {
            return Err(ScriptError::new(format!(
                "invalid qualified attribute name: `{qualified_name}`"
            )));
        }
        if namespace_uri.is_none() {
            return Err(ScriptError::new(format!(
                "invalid qualified attribute name: `{qualified_name}`"
            )));
        }
    }

    Ok(Value::Attribute(AttributeHandle::new(
        namespace_uri.map(str::to_string),
        qualified_name,
        "",
        None,
    )))
}

fn attribute_name_matches_local_name(attribute_name: &str, local_name: &str) -> bool {
    if attribute_name == local_name {
        return true;
    }

    match attribute_name.split_once(':') {
        Some((_, suffix)) => suffix == local_name,
        None => false,
    }
}

fn attribute_name_matches_namespace_local_name(
    attribute_name: &str,
    namespace_uri: Option<&str>,
    local_name: &str,
) -> bool {
    let has_colon = attribute_name.contains(':');
    match namespace_uri {
        None => !has_colon && attribute_name == local_name,
        Some(_) => has_colon && attribute_name_matches_local_name(attribute_name, local_name),
    }
}

fn attribute_current_value<H: HostBindings>(
    attr: &AttributeHandle,
    host: &mut H,
) -> Result<String> {
    let state = attr.0.borrow();
    if let Some(owner_element) = state.owner_element {
        if host.element_has_attribute(owner_element, &state.name)? {
            return Ok(host
                .element_get_attribute(owner_element, &state.name)?
                .unwrap_or_else(String::new));
        }
    }

    Ok(state.value.clone())
}

fn attribute_owner_element_value<H: HostBindings>(
    attr: &AttributeHandle,
    host: &mut H,
) -> Result<Value> {
    let state = attr.0.borrow();
    if let Some(owner_element) = state.owner_element {
        if host.element_has_attribute(owner_element, &state.name)? {
            return Ok(Value::Element(owner_element));
        }
    }

    Ok(Value::Null)
}

fn attribute_set_value<H: HostBindings>(
    attr: AttributeHandle,
    value: &str,
    host: &mut H,
) -> Result<()> {
    let owner_element = attr.owner_element();
    attr.set_value(value);
    if let Some(owner_element) = owner_element {
        host.element_set_attribute(owner_element, &attr.name(), value)?;
    }
    Ok(())
}

fn get_attribute_node_snapshot<H: HostBindings>(
    host: &mut H,
    element: ElementHandle,
    namespace_uri: Option<&str>,
    local_name: &str,
) -> Result<Option<Value>> {
    for name in named_node_map_names(element, host)? {
        if !attribute_name_matches_namespace_local_name(&name, namespace_uri, local_name) {
            continue;
        }

        let Some(value) = host.element_get_attribute(element, &name)? else {
            continue;
        };
        return Ok(Some(Value::Attribute(AttributeHandle::new(
            namespace_uri.map(str::to_string),
            name,
            value,
            Some(element),
        ))));
    }

    Ok(None)
}

fn element_get_attribute_node<H: HostBindings>(
    element: ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [name_expr] = args else {
        return Err(ScriptError::new(
            "getAttributeNode() expects exactly one argument",
        ));
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    Ok(
        match get_attribute_node_snapshot(host, element, None, &name)? {
            Some(snapshot) => snapshot,
            None => Value::Null,
        },
    )
}

fn element_get_attribute_node_ns<H: HostBindings>(
    element: ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [namespace_expr, local_name_expr] = args else {
        return Err(ScriptError::new(
            "getAttributeNodeNS() expects exactly two arguments",
        ));
    };

    let namespace_uri = namespace_uri_from_value(&eval_expr(namespace_expr, env, host)?);
    let local_name = as_string(&eval_expr(local_name_expr, env, host)?);
    Ok(
        match get_attribute_node_snapshot(host, element, namespace_uri.as_deref(), &local_name)? {
            Some(snapshot) => snapshot,
            None => Value::Null,
        },
    )
}

fn element_set_attribute_node<H: HostBindings>(
    element: ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [attr_expr] = args else {
        return Err(ScriptError::new(
            "setAttributeNode() expects exactly one argument",
        ));
    };

    let attr = match eval_expr(attr_expr, env, host)? {
        Value::Attribute(attr) => attr,
        _ => {
            return Err(ScriptError::new(
                "setAttributeNode() requires an Attr argument",
            ));
        }
    };

    if let Some(owner_element) = attr.owner_element() {
        if owner_element != element {
            return Err(ScriptError::new(
                "setAttributeNode() requires an Attr that belongs to the element",
            ));
        }
    }

    let previous = host.element_get_attribute(element, &attr.name())?;
    host.element_set_attribute(element, &attr.name(), &attr.value())?;
    attr.set_owner_element(Some(element));

    Ok(match previous {
        Some(text) => Value::Attribute(AttributeHandle::new(
            attr.namespace_uri(),
            attr.name(),
            text,
            None,
        )),
        None => Value::Null,
    })
}

fn element_set_attribute_node_ns<H: HostBindings>(
    element: ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [attr_expr] = args else {
        return Err(ScriptError::new(
            "setAttributeNodeNS() expects exactly one argument",
        ));
    };

    let attr = match eval_expr(attr_expr, env, host)? {
        Value::Attribute(attr) => attr,
        _ => {
            return Err(ScriptError::new(
                "setAttributeNodeNS() requires an Attr argument",
            ));
        }
    };

    if let Some(owner_element) = attr.owner_element() {
        if owner_element != element {
            return Err(ScriptError::new(
                "setAttributeNodeNS() requires an Attr that belongs to the element",
            ));
        }
    }

    let previous = host.element_get_attribute(element, &attr.name())?;
    host.element_set_attribute(element, &attr.name(), &attr.value())?;
    attr.set_owner_element(Some(element));

    Ok(match previous {
        Some(text) => Value::Attribute(AttributeHandle::new(
            attr.namespace_uri(),
            attr.name(),
            text,
            None,
        )),
        None => Value::Null,
    })
}

fn element_remove_attribute_node<H: HostBindings>(
    element: ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [attr_expr] = args else {
        return Err(ScriptError::new(
            "removeAttributeNode() expects exactly one argument",
        ));
    };

    let attr = match eval_expr(attr_expr, env, host)? {
        Value::Attribute(attr) => attr,
        _ => {
            return Err(ScriptError::new(
                "removeAttributeNode() requires an Attr argument",
            ));
        }
    };

    match attr.owner_element() {
        Some(owner_element) if owner_element == element => {}
        _ => {
            return Err(ScriptError::new(
                "removeAttributeNode() requires an Attr that belongs to the element",
            ));
        }
    }

    let current = host.element_get_attribute(element, &attr.name())?;
    let current_text = current.ok_or_else(|| {
        ScriptError::new("removeAttributeNode() requires an Attr that belongs to the element")
    })?;
    host.element_remove_attribute(element, &attr.name())?;
    attr.set_owner_element(None);

    Ok(Value::Attribute(AttributeHandle::new(
        attr.namespace_uri(),
        attr.name(),
        current_text,
        None,
    )))
}

fn named_node_map_names<H: HostBindings>(
    element: ElementHandle,
    host: &mut H,
) -> Result<Vec<String>> {
    host.element_attribute_names(element)
}

fn named_node_map_item<H: HostBindings>(
    element: &ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [index_expr] = args else {
        return Err(ScriptError::new(
            "NamedNodeMap.item() expects exactly one argument",
        ));
    };

    let index_value = eval_expr(index_expr, env, host)?;
    let Some(index) = index_from_value(&index_value) else {
        return Ok(Value::Null);
    };

    named_node_map_item_value(*element, index, host)
}

fn named_node_map_get_named_item<H: HostBindings>(
    element: &ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [name_expr] = args else {
        return Err(ScriptError::new(
            "NamedNodeMap.getNamedItem() expects exactly one argument",
        ));
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    named_node_map_get_named_item_value(*element, &name, host)
}

fn named_node_map_get_named_item_ns<H: HostBindings>(
    element: &ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [namespace_expr, local_name_expr] = args else {
        return Err(ScriptError::new(
            "NamedNodeMap.getNamedItemNS() expects exactly two arguments",
        ));
    };

    let namespace_uri = namespace_uri_from_value(&eval_expr(namespace_expr, env, host)?);
    let local_name = as_string(&eval_expr(local_name_expr, env, host)?);
    named_node_map_get_named_item_ns_value(*element, namespace_uri, &local_name, host)
}

fn named_node_map_current_values<H: HostBindings>(
    element: ElementHandle,
    host: &mut H,
) -> Result<Vec<Value>> {
    let names = named_node_map_names(element, host)?;
    let mut values = Vec::with_capacity(names.len());
    for name in names {
        let Some(value) = host.element_get_attribute(element, &name)? else {
            continue;
        };
        values.push(Value::Attribute(AttributeHandle::new(
            None,
            name,
            value,
            Some(element),
        )));
    }
    Ok(values)
}

fn named_node_map_item_value<H: HostBindings>(
    element: ElementHandle,
    index: usize,
    host: &mut H,
) -> Result<Value> {
    let names = named_node_map_names(element, host)?;
    let Some(name) = names.get(index) else {
        return Ok(Value::Null);
    };
    let Some(value) = host.element_get_attribute(element, name)? else {
        return Ok(Value::Null);
    };
    Ok(Value::Attribute(AttributeHandle::new(
        None,
        name.clone(),
        value,
        Some(element),
    )))
}

fn named_node_map_get_named_item_value<H: HostBindings>(
    element: ElementHandle,
    name: &str,
    host: &mut H,
) -> Result<Value> {
    let Some(value) = host.element_get_attribute(element, name)? else {
        return Ok(Value::Null);
    };
    Ok(Value::Attribute(AttributeHandle::new(
        None,
        name.to_string(),
        value,
        Some(element),
    )))
}

fn named_node_map_get_named_item_ns_value<H: HostBindings>(
    element: ElementHandle,
    namespace_uri: Option<String>,
    local_name: &str,
    host: &mut H,
) -> Result<Value> {
    for name in named_node_map_names(element, host)? {
        if !attribute_name_matches_namespace_local_name(&name, namespace_uri.as_deref(), local_name)
        {
            continue;
        }

        let Some(value) = host.element_get_attribute(element, &name)? else {
            continue;
        };
        return Ok(Value::Attribute(AttributeHandle::new(
            namespace_uri.clone(),
            name,
            value,
            Some(element),
        )));
    }

    Ok(Value::Null)
}

fn named_node_map_set_named_item<H: HostBindings>(
    element: &ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    element_set_attribute_node(*element, args, env, host)
}

fn named_node_map_set_named_item_ns<H: HostBindings>(
    element: &ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    element_set_attribute_node_ns(*element, args, env, host)
}

fn named_node_map_remove_named_item<H: HostBindings>(
    element: &ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [name_expr] = args else {
        return Err(ScriptError::new(
            "NamedNodeMap.removeNamedItem() expects exactly one argument",
        ));
    };

    let name = as_string(&eval_expr(name_expr, env, host)?);
    let Some(value) = host.element_get_attribute(*element, &name)? else {
        return Err(ScriptError::new(
            "NamedNodeMap.removeNamedItem() requires an existing attribute",
        ));
    };

    host.element_remove_attribute(*element, &name)?;
    Ok(Value::Attribute(AttributeHandle::new(
        None, name, value, None,
    )))
}

fn named_node_map_remove_named_item_ns<H: HostBindings>(
    element: &ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [namespace_expr, local_name_expr] = args else {
        return Err(ScriptError::new(
            "NamedNodeMap.removeNamedItemNS() expects exactly two arguments",
        ));
    };

    let namespace_uri = namespace_uri_from_value(&eval_expr(namespace_expr, env, host)?);
    let local_name = as_string(&eval_expr(local_name_expr, env, host)?);

    for name in named_node_map_names(*element, host)? {
        if !attribute_name_matches_namespace_local_name(
            &name,
            namespace_uri.as_deref(),
            &local_name,
        ) {
            continue;
        }

        let Some(value) = host.element_get_attribute(*element, &name)? else {
            continue;
        };

        host.element_remove_attribute(*element, &name)?;
        return Ok(Value::Attribute(AttributeHandle::new(
            namespace_uri,
            name,
            value,
            None,
        )));
    }

    Err(ScriptError::new(
        "NamedNodeMap.removeNamedItemNS() requires an existing attribute",
    ))
}

fn named_node_map_keys(element: &ElementHandle, host: &mut impl HostBindings) -> Result<Value> {
    let names = named_node_map_names(*element, host)?;
    Ok(collection_iterator(
        (0..names.len())
            .map(|index| Value::Number(index as f64))
            .collect(),
    ))
}

fn named_node_map_values(element: &ElementHandle, host: &mut impl HostBindings) -> Result<Value> {
    Ok(collection_iterator(named_node_map_current_values(
        *element, host,
    )?))
}

fn named_node_map_entries(element: &ElementHandle, host: &mut impl HostBindings) -> Result<Value> {
    Ok(collection_entries(named_node_map_current_values(
        *element, host,
    )?))
}

fn named_node_map_for_each<H: HostBindings>(
    element: &ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new(
                "NamedNodeMap.forEach() expects one or two arguments",
            ));
        }
    };

    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => {
            return Err(ScriptError::new(
                "NamedNodeMap.forEach() requires an arrow function callback",
            ));
        }
    };
    if let Some(this_arg_expr) = this_arg_expr {
        let _ = eval_expr(this_arg_expr, env, host)?;
    }

    let items = named_node_map_current_values(*element, host)?;
    let collection_value = Value::NamedNodeMap(*element);
    for_each_over_items(&callback, items, collection_value, env, host)
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

fn mime_type_array_for_each<H: HostBindings>(
    list: &MimeTypeArrayState,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new(
                "navigator.mimeTypes.forEach() expects one or two arguments",
            ));
        }
    };

    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => {
            return Err(ScriptError::new(
                "navigator.mimeTypes.forEach() requires an arrow function callback",
            ));
        }
    };
    if let Some(this_arg_expr) = this_arg_expr {
        let _ = eval_expr(this_arg_expr, env, host)?;
    }

    let items = list.items().iter().cloned().map(Value::String).collect();
    let collection_value = Value::MimeTypeArray(list.clone());
    for_each_over_items(&callback, items, collection_value, env, host)
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
        let outer_keys: Vec<String> = env.keys().cloned().collect();
        let param_snapshots: Vec<(String, Option<Value>)> = callback
            .params
            .iter()
            .map(|param| (param.clone(), env.get(param).cloned()))
            .collect();

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
        match eval_program_with_bindings(&program, host, &mut bindings)? {
            EvalControl::Continue | EvalControl::Return(_) => {}
            EvalControl::Break => return Err(ScriptError::new("break outside loop")),
            EvalControl::ContinueLoop => {
                return Err(ScriptError::new("continue outside loop"));
            }
        }

        for key in outer_keys {
            if callback.params.iter().any(|param| param == &key) {
                continue;
            }
            if let Some(value) = bindings.get(&key).cloned() {
                env.insert(key, value);
            } else {
                env.remove(&key);
            }
        }

        for (param, previous) in param_snapshots {
            match previous {
                Some(value) => {
                    env.insert(param, value);
                }
                None => {
                    env.remove(&param);
                }
            }
        }
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

fn window_property_is_reserved(property: &str) -> bool {
    matches!(
        property,
        "closed"
            | "document"
            | "frameElement"
            | "frames"
            | "history"
            | "length"
            | "navigator"
            | "opener"
            | "origin"
            | "parent"
            | "screen"
            | "self"
            | "top"
            | "window"
            | "devicePixelRatio"
            | "innerWidth"
            | "innerHeight"
            | "outerWidth"
            | "outerHeight"
            | "screenX"
            | "screenY"
            | "screenLeft"
            | "screenTop"
            | "scrollX"
            | "scrollY"
            | "pageXOffset"
            | "pageYOffset"
    )
}

fn html_collection_property_is_reserved(collection: &HtmlCollectionTarget, property: &str) -> bool {
    matches!(
        property,
        "item" | "namedItem" | "forEach" | "keys" | "values" | "entries" | "toString"
    ) || matches!(collection, HtmlCollectionTarget::DocumentPlugins) && property == "refresh"
}

fn plugins_refresh(args: &[Expr]) -> Result<Value> {
    let [] = args else {
        return Err(ScriptError::new(
            "navigator.plugins.refresh() expects no arguments",
        ));
    };

    Ok(Value::Undefined)
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

fn eval_number<H: HostBindings>(
    expr: &Expr,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    operator: &str,
) -> Result<f64> {
    number_from_value(&eval_expr(expr, env, host)?)
        .map_err(|_| ScriptError::new(format!("{operator} expects a number")))
}

fn number_from_value(value: &Value) -> Result<f64> {
    match value {
        Value::Number(number) => Ok(*number),
        Value::String(value) => {
            if value.trim().is_empty() {
                Ok(0.0)
            } else {
                value
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ScriptError::new("value cannot be converted to a number"))
            }
        }
        Value::Boolean(value) => Ok(if *value { 1.0 } else { 0.0 }),
        Value::Null => Ok(0.0),
        Value::Undefined => Ok(f64::NAN),
        _ => Err(ScriptError::new("value cannot be converted to a number")),
    }
}

fn eval_equality(left: &Value, right: &Value, strict: bool) -> bool {
    if strict {
        return left == right;
    }

    if matches!(left, Value::Null | Value::Undefined)
        && matches!(right, Value::Null | Value::Undefined)
    {
        return true;
    }

    match (left, right) {
        (Value::Boolean(lhs), rhs) => {
            let lhs = if *lhs { 1.0 } else { 0.0 };
            number_from_value(rhs)
                .map(|rhs| lhs == rhs)
                .unwrap_or(false)
        }
        (lhs, Value::Boolean(rhs)) => {
            let rhs = if *rhs { 1.0 } else { 0.0 };
            number_from_value(lhs)
                .map(|lhs| lhs == rhs)
                .unwrap_or(false)
        }
        (Value::Number(lhs), Value::String(rhs)) => rhs
            .trim()
            .parse::<f64>()
            .map(|rhs| *lhs == rhs)
            .unwrap_or(false),
        (Value::String(lhs), Value::Number(rhs)) => lhs
            .trim()
            .parse::<f64>()
            .map(|lhs| lhs == *rhs)
            .unwrap_or(false),
        _ => left == right,
    }
}

fn eval_typeof(value: &Value) -> &'static str {
    match value {
        Value::Undefined => "undefined",
        Value::Null => "object",
        Value::Boolean(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::ObjectNamespace | Value::ArrayNamespace => "function",
        Value::IntlNamespace => "object",
        Value::Object(_)
        | Value::Array(_)
        | Value::Map(_)
        | Value::Symbol(_)
        | Value::RegExp(_) => "object",
        Value::Function(_) => "function",
        _ => "object",
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Undefined | Value::Null => false,
        Value::Boolean(value) => *value,
        Value::Number(value) => *value != 0.0,
        Value::String(value) => !value.is_empty(),
        Value::Object(_)
        | Value::Array(_)
        | Value::Map(_)
        | Value::Symbol(_)
        | Value::RegExp(_)
        | Value::Date(_)
        | Value::IntlNumberFormat(_)
        | Value::IntlDateTimeFormat(_)
        | Value::IntlCollator(_)
        | Value::ObjectNamespace
        | Value::ArrayNamespace
        | Value::IntlNamespace
        | Value::Element(_)
        | Value::Attribute(_)
        | Value::ClassList(_)
        | Value::Dataset(_)
        | Value::NamedNodeMap(_)
        | Value::Navigator
        | Value::Clipboard
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

fn positive_index_from_attribute(value: Option<String>, default: usize) -> usize {
    value
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn property_key_from_string(name: impl Into<String>) -> PropertyKey {
    PropertyKey::String(name.into())
}

fn property_key_from_value(value: &Value) -> PropertyKey {
    match value {
        Value::Symbol(symbol) => PropertyKey::Symbol(symbol.id()),
        other => PropertyKey::String(as_string(other)),
    }
}

fn property_key_to_string(key: &PropertyKey) -> Option<&str> {
    match key {
        PropertyKey::String(value) => Some(value.as_str()),
        PropertyKey::Symbol(_) => None,
    }
}

fn array_to_string(array: &crate::ArrayHandle, separator: &str, limit: Option<usize>) -> String {
    let items = {
        let state = array.0.borrow();
        state.items.clone()
    };
    let rendered: Vec<String> = items
        .into_iter()
        .take(limit.unwrap_or(usize::MAX))
        .map(|value| as_string(&value))
        .collect();
    rendered.join(separator)
}

#[allow(dead_code)]
fn object_property_entries(
    properties: &[(PropertyKey, crate::PropertyValue)],
) -> Vec<(PropertyKey, crate::PropertyValue)> {
    properties.to_vec()
}

fn property_key_from_expr_value(value: &Value) -> PropertyKey {
    property_key_from_value(value)
}

fn property_key_matches_string(key: &PropertyKey, name: &str) -> bool {
    matches!(key, PropertyKey::String(value) if value == name)
}

fn object_find_property<'a>(
    properties: &'a [(PropertyKey, crate::PropertyValue)],
    key: &PropertyKey,
) -> Option<&'a crate::PropertyValue> {
    properties
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value)
}

fn object_find_property_mut<'a>(
    properties: &'a mut Vec<(PropertyKey, crate::PropertyValue)>,
    key: &PropertyKey,
) -> Option<&'a mut crate::PropertyValue> {
    properties
        .iter_mut()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value)
}

#[allow(dead_code)]
fn object_remove_property(
    properties: &mut Vec<(PropertyKey, crate::PropertyValue)>,
    key: &PropertyKey,
) {
    if let Some(index) = properties
        .iter()
        .position(|(candidate, _)| candidate == key)
    {
        properties.remove(index);
    }
}

fn object_own_string_keys(object: &crate::ObjectHandle) -> Vec<String> {
    let state = object.0.borrow();
    state
        .properties
        .iter()
        .filter_map(|(key, _)| property_key_to_string(key).map(str::to_string))
        .collect()
}

#[allow(dead_code)]
fn object_own_symbol_keys(object: &crate::ObjectHandle) -> Vec<crate::SymbolValue> {
    let state = object.0.borrow();
    state
        .properties
        .iter()
        .filter_map(|(key, _)| match key {
            PropertyKey::Symbol(id) => Some(crate::SymbolValue::from_parts(*id, None)),
            _ => None,
        })
        .collect()
}

fn object_property_value<H: HostBindings>(
    object: &crate::ObjectHandle,
    key: &PropertyKey,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Option<Value>> {
    let property = {
        let state = object.0.borrow();
        object_find_property(&state.properties, key).cloned()
    };

    match property {
        Some(crate::PropertyValue::Data(value)) => Ok(Some(value)),
        Some(crate::PropertyValue::Accessor { getter, .. }) => match getter {
            Some(getter) => call_script_function_with_this(
                &getter,
                &[],
                Value::Object(object.clone()),
                env,
                host,
            )
            .map(Some),
            None => Ok(Some(Value::Undefined)),
        },
        None => Ok(None),
    }
}

fn object_set_property_value<H: HostBindings>(
    object: &crate::ObjectHandle,
    key: PropertyKey,
    value: Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<()> {
    let accessor = {
        let mut state = object.0.borrow_mut();
        match object_find_property_mut(&mut state.properties, &key) {
            Some(crate::PropertyValue::Data(existing)) => {
                *existing = value;
                return Ok(());
            }
            Some(crate::PropertyValue::Accessor { setter, .. }) => setter.clone(),
            None => {
                state
                    .properties
                    .push((key.clone(), crate::PropertyValue::Data(value)));
                return Ok(());
            }
        }
    };

    if let Some(setter) = accessor {
        call_script_function_value(&setter, &[value], Value::Object(object.clone()), env, host)?;
    }

    Ok(())
}

fn array_index_from_key(key: &PropertyKey) -> Option<usize> {
    match key {
        PropertyKey::String(value) => value.parse::<usize>().ok(),
        PropertyKey::Symbol(_) => None,
    }
}

fn array_property_value<H: HostBindings>(
    array: &crate::ArrayHandle,
    key: &PropertyKey,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Option<Value>> {
    if property_key_matches_string(key, "length") {
        return Ok(Some(Value::Number(array.0.borrow().items.len() as f64)));
    }
    if property_key_matches_string(key, "byteLength") {
        return Ok(Some(Value::Number(array.0.borrow().items.len() as f64)));
    }
    if property_key_matches_string(key, "buffer") {
        return Ok(Some(Value::Array(array.clone())));
    }

    if let Some(index) = array_index_from_key(key) {
        return Ok(array
            .0
            .borrow()
            .items
            .get(index)
            .cloned()
            .or(Some(Value::Undefined)));
    }

    let property = {
        let state = array.0.borrow();
        object_find_property(&state.properties, key).cloned()
    };

    match property {
        Some(crate::PropertyValue::Data(value)) => Ok(Some(value)),
        Some(crate::PropertyValue::Accessor { getter, .. }) => match getter {
            Some(getter) => {
                call_script_function_with_this(&getter, &[], Value::Array(array.clone()), env, host)
                    .map(Some)
            }
            None => Ok(Some(Value::Undefined)),
        },
        None => Ok(property_key_to_string(key).and_then(|name| {
            matches!(
                name,
                "join"
                    | "map"
                    | "filter"
                    | "forEach"
                    | "flatMap"
                    | "flat"
                    | "push"
                    | "pop"
                    | "slice"
                    | "concat"
                    | "sort"
                    | "reverse"
                    | "reduce"
                    | "includes"
                    | "indexOf"
                    | "lastIndexOf"
                    | "toString"
            )
            .then(|| native_method_function(Value::Array(array.clone()), name))
        })),
    }
}

fn array_set_property_value<H: HostBindings>(
    array: &crate::ArrayHandle,
    key: PropertyKey,
    value: Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<()> {
    if property_key_matches_string(&key, "length") {
        return Err(ScriptError::new(
            "cannot assign to array length in this runtime",
        ));
    }

    if let Some(index) = array_index_from_key(&key) {
        let mut state = array.0.borrow_mut();
        if index >= state.items.len() {
            state.items.resize(index + 1, Value::Undefined);
        }
        state.items[index] = value;
        return Ok(());
    }

    let accessor = {
        let mut state = array.0.borrow_mut();
        match object_find_property_mut(&mut state.properties, &key) {
            Some(crate::PropertyValue::Data(existing)) => {
                *existing = value;
                return Ok(());
            }
            Some(crate::PropertyValue::Accessor { setter, .. }) => setter.clone(),
            None => {
                state
                    .properties
                    .push((key.clone(), crate::PropertyValue::Data(value)));
                return Ok(());
            }
        }
    };

    if let Some(setter) = accessor {
        call_script_function_value(&setter, &[value], Value::Array(array.clone()), env, host)?;
    }

    Ok(())
}

#[allow(dead_code)]
fn object_value_keys(object: &crate::ObjectHandle) -> Vec<String> {
    object_own_string_keys(object)
}

#[allow(dead_code)]
fn object_value_entries(object: &crate::ObjectHandle) -> Vec<(String, Value)> {
    let state = object.0.borrow();
    state
        .properties
        .iter()
        .filter_map(|(key, value)| {
            property_key_to_string(key).and_then(|name| match value {
                crate::PropertyValue::Data(value) => Some((name.to_string(), value.clone())),
                crate::PropertyValue::Accessor { .. } => None,
            })
        })
        .collect()
}

#[allow(dead_code)]
fn array_value_entries(array: &crate::ArrayHandle) -> Vec<(String, Value)> {
    let state = array.0.borrow();
    let mut entries = state
        .items
        .iter()
        .enumerate()
        .map(|(index, value)| (index.to_string(), value.clone()))
        .collect::<Vec<_>>();
    for (key, value) in &state.properties {
        if let Some(name) = property_key_to_string(key) {
            if let crate::PropertyValue::Data(value) = value {
                entries.push((name.to_string(), value.clone()));
            }
        }
    }
    entries
}

fn property_value_on_value<H: HostBindings>(
    value: &Value,
    key: &PropertyKey,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Option<Value>> {
    match value {
        Value::Object(object) => object_property_value(object, key, env, host),
        Value::Array(array) => array_property_value(array, key, env, host),
        Value::Map(map) => map_property_value(map, key),
        _ => Ok(None),
    }
}

fn set_property_value_on_value<H: HostBindings>(
    value: &Value,
    key: PropertyKey,
    next: Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<bool> {
    match value {
        Value::Object(object) => {
            object_set_property_value(object, key, next, env, host)?;
            Ok(true)
        }
        Value::Array(array) => {
            array_set_property_value(array, key, next, env, host)?;
            Ok(true)
        }
        Value::Map(map) => {
            map_set_property_value(map, key, next);
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn map_key_from_value(value: &Value) -> MapKey {
    match value {
        Value::Undefined => MapKey::Undefined,
        Value::Null => MapKey::Null,
        Value::Boolean(value) => MapKey::Boolean(*value),
        Value::Number(value) => MapKey::Number(value.to_bits()),
        Value::String(value) => MapKey::String(value.clone()),
        Value::Symbol(symbol) => MapKey::Symbol(symbol.id()),
        Value::Object(object) => MapKey::Object(object.identity()),
        Value::Array(array) => MapKey::Array(array.identity()),
        Value::Map(map) => MapKey::Map(map.identity()),
        other => MapKey::String(as_string(other)),
    }
}

#[allow(dead_code)]
fn map_values_from_handle(map: &MapHandle) -> Vec<(MapKey, Value)> {
    map.0.borrow().entries.clone()
}

fn map_property_value(map: &MapHandle, key: &PropertyKey) -> Result<Option<Value>> {
    if property_key_matches_string(key, "size") {
        return Ok(Some(Value::Number(map.0.borrow().entries.len() as f64)));
    }

    let property = {
        let state = map.0.borrow();
        object_find_property(&state.properties, key).cloned()
    };

    match property {
        Some(PropertyValue::Data(value)) => Ok(Some(value)),
        Some(PropertyValue::Accessor { .. }) => Ok(Some(Value::Undefined)),
        None => Ok(property_key_to_string(key).and_then(|name| {
            matches!(
                name,
                "get"
                    | "set"
                    | "has"
                    | "delete"
                    | "clear"
                    | "keys"
                    | "values"
                    | "entries"
                    | "forEach"
                    | "toString"
            )
            .then(|| native_method_function(Value::Map(map.clone()), name))
        })),
    }
}

fn map_set_property_value(map: &MapHandle, key: PropertyKey, value: Value) {
    if property_key_matches_string(&key, "size") {
        return;
    }

    let mut state = map.0.borrow_mut();
    match object_find_property_mut(&mut state.properties, &key) {
        Some(PropertyValue::Data(existing)) => {
            *existing = value;
        }
        Some(PropertyValue::Accessor { .. }) => {}
        None => {
            state.properties.push((key, PropertyValue::Data(value)));
        }
    }
}

fn object_own_property_values<H: HostBindings>(
    object: &crate::ObjectHandle,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Vec<(PropertyKey, Value)>> {
    let properties = {
        let state = object.0.borrow();
        state.properties.clone()
    };

    let mut values = Vec::new();
    for (key, property) in properties {
        if let Some(value) = match property {
            PropertyValue::Data(value) => Some(value),
            PropertyValue::Accessor { getter, .. } => match getter {
                Some(getter) => Some(call_script_function_with_this(
                    &getter,
                    &[],
                    Value::Object(object.clone()),
                    env,
                    host,
                )?),
                None => Some(Value::Undefined),
            },
        } {
            values.push((key, value));
        }
    }
    Ok(values)
}

fn array_own_property_values<H: HostBindings>(
    array: &crate::ArrayHandle,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Vec<(PropertyKey, Value)>> {
    let (items, properties) = {
        let state = array.0.borrow();
        (state.items.clone(), state.properties.clone())
    };

    let mut values = Vec::new();
    for (index, item) in items.into_iter().enumerate() {
        values.push((PropertyKey::String(index.to_string()), item));
    }
    for (key, property) in properties {
        if let Some(value) = match property {
            PropertyValue::Data(value) => Some(value),
            PropertyValue::Accessor { getter, .. } => match getter {
                Some(getter) => Some(call_script_function_with_this(
                    &getter,
                    &[],
                    Value::Array(array.clone()),
                    env,
                    host,
                )?),
                None => Some(Value::Undefined),
            },
        } {
            values.push((key, value));
        }
    }
    Ok(values)
}

fn string_own_property_values(value: &str) -> Vec<(PropertyKey, Value)> {
    value
        .chars()
        .enumerate()
        .map(|(index, ch)| {
            (
                PropertyKey::String(index.to_string()),
                Value::String(ch.to_string()),
            )
        })
        .collect()
}

fn object_own_string_values<H: HostBindings>(
    value: &Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Vec<(String, Value)>> {
    match value {
        Value::Object(object) => Ok(object_own_property_values(object, env, host)?
            .into_iter()
            .filter_map(|(key, value)| match key {
                PropertyKey::String(name) => Some((name, value)),
                PropertyKey::Symbol(_) => None,
            })
            .collect()),
        Value::Array(array) => Ok(array_own_property_values(array, env, host)?
            .into_iter()
            .filter_map(|(key, value)| match key {
                PropertyKey::String(name) => Some((name, value)),
                PropertyKey::Symbol(_) => None,
            })
            .collect()),
        Value::Map(map) => {
            let state = map.0.borrow();
            Ok(state
                .properties
                .iter()
                .filter_map(|(key, property)| match key {
                    PropertyKey::String(name) => match property {
                        PropertyValue::Data(value) => Some((name.clone(), value.clone())),
                        PropertyValue::Accessor { .. } => Some((name.clone(), Value::Undefined)),
                    },
                    PropertyKey::Symbol(_) => None,
                })
                .collect())
        }
        Value::String(string) => Ok(string_own_property_values(string)
            .into_iter()
            .filter_map(|(key, value)| match key {
                PropertyKey::String(name) => Some((name, value)),
                PropertyKey::Symbol(_) => None,
            })
            .collect()),
        _ => Ok(Vec::new()),
    }
}

fn object_own_symbol_values<H: HostBindings>(
    value: &Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Vec<crate::SymbolValue>> {
    match value {
        Value::Object(object) => Ok(object_own_property_values(object, env, host)?
            .into_iter()
            .filter_map(|(key, _)| match key {
                PropertyKey::Symbol(id) => Some(crate::SymbolValue::from_parts(id, None)),
                _ => None,
            })
            .collect()),
        Value::Array(array) => Ok(array_own_property_values(array, env, host)?
            .into_iter()
            .filter_map(|(key, _)| match key {
                PropertyKey::Symbol(id) => Some(crate::SymbolValue::from_parts(id, None)),
                _ => None,
            })
            .collect()),
        Value::Map(map) => {
            let state = map.0.borrow();
            Ok(state
                .properties
                .iter()
                .filter_map(|(key, _)| match key {
                    PropertyKey::Symbol(id) => Some(crate::SymbolValue::from_parts(*id, None)),
                    _ => None,
                })
                .collect())
        }
        _ => Ok(Vec::new()),
    }
}

fn object_assign_from_source<H: HostBindings>(
    target: &crate::ObjectHandle,
    source: &Value,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<()> {
    match source {
        Value::Undefined | Value::Null => Ok(()),
        Value::String(string) => {
            for (index, ch) in string.chars().enumerate() {
                object_set_property_value(
                    target,
                    PropertyKey::String(index.to_string()),
                    Value::String(ch.to_string()),
                    env,
                    host,
                )?;
            }
            Ok(())
        }
        Value::Object(object) => {
            for (key, value) in object_own_property_values(object, env, host)? {
                object_set_property_value(target, key, value, env, host)?;
            }
            Ok(())
        }
        Value::Array(array) => {
            for (key, value) in array_own_property_values(array, env, host)? {
                object_set_property_value(target, key, value, env, host)?;
            }
            Ok(())
        }
        Value::Map(map) => {
            let state = map.0.borrow();
            for (key, property) in &state.properties {
                if let Some(name) = property_key_to_string(key) {
                    let value = match property {
                        PropertyValue::Data(value) => value.clone(),
                        PropertyValue::Accessor { getter, .. } => match getter {
                            Some(getter) => call_script_function_with_this(
                                getter,
                                &[],
                                Value::Map(map.clone()),
                                env,
                                host,
                            )?,
                            None => Value::Undefined,
                        },
                    };
                    object_set_property_value(
                        target,
                        PropertyKey::String(name.to_string()),
                        value,
                        env,
                        host,
                    )?;
                } else if let PropertyKey::Symbol(id) = key {
                    let value = match property {
                        PropertyValue::Data(value) => value.clone(),
                        PropertyValue::Accessor { getter, .. } => match getter {
                            Some(getter) => call_script_function_with_this(
                                getter,
                                &[],
                                Value::Map(map.clone()),
                                env,
                                host,
                            )?,
                            None => Value::Undefined,
                        },
                    };
                    object_set_property_value(target, PropertyKey::Symbol(*id), value, env, host)?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn array_from_value<H: HostBindings>(
    source: Value,
    _env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Vec<Value>> {
    match source {
        Value::Undefined | Value::Null => Ok(Vec::new()),
        Value::Array(array) => Ok(array.0.borrow().items.clone()),
        Value::String(string) => Ok(string
            .chars()
            .map(|ch| Value::String(ch.to_string()))
            .collect()),
        Value::NamedNodeMap(element) => named_node_map_current_values(element, host),
        Value::HtmlCollection(collection) => Ok(html_collection_items(&collection, host)?
            .into_iter()
            .map(Value::Element)
            .collect()),
        Value::NodeList(target) => Ok(node_list_items(&target, host)?
            .into_iter()
            .map(NodeListItem::into_value)
            .collect()),
        Value::RadioNodeList(target) => Ok(radio_node_list_items(&target, host)?
            .into_iter()
            .map(Value::Element)
            .collect()),
        Value::StringList(list) => Ok(list.items().iter().cloned().map(Value::String).collect()),
        Value::MimeTypeArray(list) => Ok(list.items().iter().cloned().map(Value::String).collect()),
        Value::StyleSheetList(target) => Ok(style_sheet_list_items(&target, host)?
            .into_iter()
            .map(|handle| Value::StyleSheet(StyleSheetTarget::OwnerNode(handle)))
            .collect()),
        Value::CollectionIterator(iterator) => {
            let mut out = Vec::new();
            loop {
                let next = iterator.next_result();
                if next.done() {
                    break;
                }
                if let Some(value) = next.value() {
                    out.push(value);
                }
            }
            Ok(out)
        }
        Value::Map(map) => {
            let state = map.0.borrow();
            let mut out = Vec::new();
            for (key, value) in &state.entries {
                out.push(Value::Array(ArrayHandle::new(vec![
                    match key {
                        MapKey::Undefined => Value::Undefined,
                        MapKey::Null => Value::Null,
                        MapKey::Boolean(value) => Value::Boolean(*value),
                        MapKey::Number(bits) => Value::Number(f64::from_bits(*bits)),
                        MapKey::String(value) => Value::String(value.clone()),
                        MapKey::Symbol(id) => {
                            Value::Symbol(crate::SymbolValue::from_parts(*id, None))
                        }
                        MapKey::Object(id) => Value::String(format!("[object Object:{id}]")),
                        MapKey::Array(id) => Value::String(format!("[object Array:{id}]")),
                        MapKey::Map(id) => Value::String(format!("[object Map:{id}]")),
                    },
                    value.clone(),
                ])));
            }
            Ok(out)
        }
        Value::Object(_) => Err(ScriptError::new("value is not iterable")),
        Value::Boolean(_)
        | Value::Number(_)
        | Value::Symbol(_)
        | Value::CollectionEntry(_)
        | Value::IteratorResult(_)
        | Value::Element(_)
        | Value::Attribute(_)
        | Value::ClassList(_)
        | Value::Dataset(_)
        | Value::TemplateContent(_)
        | Value::StyleSheet(_)
        | Value::Node(_)
        | Value::Date(_)
        | Value::IntlNumberFormat(_)
        | Value::IntlDateTimeFormat(_)
        | Value::IntlCollator(_)
        | Value::Storage(_)
        | Value::MediaQueryList(_)
        | Value::RegExp(_)
        | Value::Navigator
        | Value::Clipboard
        | Value::History
        | Value::Screen
        | Value::ScreenOrientation(_)
        | Value::Document
        | Value::Window
        | Value::Event(_)
        | Value::Function(_)
        | Value::ObjectNamespace
        | Value::ArrayNamespace
        | Value::IntlNamespace => Err(ScriptError::new("value is not iterable")),
    }
}

fn array_namespace_from<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Array(ArrayHandle::new(Vec::new())));
    }

    if args.len() > 3 {
        return Err(ScriptError::new(
            "Array.from() expects at most three arguments",
        ));
    }

    let source = eval_expr(&args[0], env, host)?;
    let mut items = array_from_value(source, env, host)?;
    if let Some(map_expr) = args.get(1) {
        let map_fn = match eval_expr(map_expr, env, host)? {
            Value::Function(function) => function,
            _ => return Err(ScriptError::new("Array.from() expects a function callback")),
        };
        let this_value = match args.get(2) {
            Some(expr) => eval_expr(expr, env, host)?,
            None => Value::Undefined,
        };
        let source_array = Value::Array(ArrayHandle::new(items.clone()));
        let mut mapped = Vec::with_capacity(items.len());
        for (index, item) in items.into_iter().enumerate() {
            let result = call_script_function_value(
                &map_fn,
                &[item, Value::Number(index as f64), source_array.clone()],
                this_value.clone(),
                env,
                host,
            )?;
            mapped.push(result);
        }
        items = mapped;
    }
    Ok(Value::Array(ArrayHandle::new(items)))
}

fn array_namespace_is_array(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut impl HostBindings,
) -> Result<Value> {
    let [value_expr] = args else {
        return Err(ScriptError::new(
            "Array.isArray() expects exactly one argument",
        ));
    };
    Ok(Value::Boolean(matches!(
        eval_expr(value_expr, env, host)?,
        Value::Array(_)
    )))
}

fn eval_array_literal<H: HostBindings>(
    elements: &[ArrayElement],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let mut items = Vec::new();
    for element in elements {
        match element {
            ArrayElement::Expression(expr) => items.push(eval_expr(expr, env, host)?),
            ArrayElement::Spread(expr) => {
                let value = eval_expr(expr, env, host)?;
                items.extend(array_from_value(value, env, host)?);
            }
        }
    }
    Ok(Value::Array(ArrayHandle::new(items)))
}

fn eval_object_property_name<H: HostBindings>(
    name: &ObjectPropertyName,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<PropertyKey> {
    match name {
        ObjectPropertyName::Static(name) => Ok(PropertyKey::String(name.clone())),
        ObjectPropertyName::Computed(expr) => {
            let value = eval_expr(expr, env, host)?;
            Ok(property_key_from_expr_value(&value))
        }
    }
}

fn eval_object_literal<H: HostBindings>(
    properties: &[ObjectProperty],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let object = crate::ObjectHandle::new();
    for property in properties {
        match property {
            ObjectProperty::KeyValue { name, value } => {
                let key = eval_object_property_name(name, env, host)?;
                let value = eval_expr(value, env, host)?;
                object_set_property_value(&object, key, value, env, host)?;
            }
            ObjectProperty::Method { name, function } => {
                let key = eval_object_property_name(name, env, host)?;
                object_set_property_value(
                    &object,
                    key,
                    Value::Function(function.clone().with_captured_bindings(env.clone())),
                    env,
                    host,
                )?;
            }
            ObjectProperty::Getter { name, function } => {
                let key = eval_object_property_name(name, env, host)?;
                let mut state = object.0.borrow_mut();
                match object_find_property_mut(&mut state.properties, &key) {
                    Some(PropertyValue::Accessor { getter, .. }) => {
                        *getter = Some(function.clone().with_captured_bindings(env.clone()));
                    }
                    Some(PropertyValue::Data(_)) | None => {
                        state.properties.push((
                            key,
                            PropertyValue::Accessor {
                                getter: Some(function.clone().with_captured_bindings(env.clone())),
                                setter: None,
                            },
                        ));
                    }
                }
            }
            ObjectProperty::Setter { name, function } => {
                let key = eval_object_property_name(name, env, host)?;
                let mut state = object.0.borrow_mut();
                match object_find_property_mut(&mut state.properties, &key) {
                    Some(PropertyValue::Accessor { setter, .. }) => {
                        *setter = Some(function.clone().with_captured_bindings(env.clone()));
                    }
                    Some(PropertyValue::Data(_)) | None => {
                        state.properties.push((
                            key,
                            PropertyValue::Accessor {
                                getter: None,
                                setter: Some(function.clone().with_captured_bindings(env.clone())),
                            },
                        ));
                    }
                }
            }
            ObjectProperty::Spread(expr) => {
                let source = eval_expr(expr, env, host)?;
                object_assign_from_source(&object, &source, env, host)?;
            }
        }
    }
    Ok(Value::Object(object))
}

fn eval_computed_member<H: HostBindings>(
    object: &Expr,
    property: &Expr,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let object_value = eval_expr(object, env, host)?;
    let property_value = eval_expr(property, env, host)?;
    let key = property_key_from_expr_value(&property_value);

    if matches!(object_value, Value::Window) {
        if let Some(name) = property_key_to_string(&key) {
            return Ok(env.get(name).cloned().unwrap_or(Value::Undefined));
        }
    }

    Ok(match object_value {
        Value::String(value) => match index_from_value(&property_value) {
            Some(index) => match value.chars().nth(index) {
                Some(ch) => Value::String(ch.to_string()),
                None => Value::Undefined,
            },
            None => Value::Undefined,
        },
        other => match property_value_on_value(&other, &key, env, host)? {
            Some(value) => value,
            None => Value::Undefined,
        },
    })
}

fn eval_new<H: HostBindings>(
    callee: &Expr,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    match callee {
        Expr::Identifier(name) if name == "Object" => {
            if args.len() > 1 {
                return Err(ScriptError::new("Object() expects at most one argument"));
            }
            let object = crate::ObjectHandle::new();
            if let Some(expr) = args.first() {
                let value = eval_expr(expr, env, host)?;
                object_assign_from_source(&object, &value, env, host)?;
            }
            Ok(Value::Object(object))
        }
        Expr::Identifier(name) if name == "Array" || name == "Uint8Array" => {
            if args.len() > 1 {
                return Err(ScriptError::new(format!(
                    "{name}() expects at most one argument"
                )));
            }
            if let Some(expr) = args.first() {
                let value = eval_expr(expr, env, host)?;
                match value {
                    Value::Number(length)
                        if length.is_finite() && length >= 0.0 && length.fract() == 0.0 =>
                    {
                        Ok(Value::Array(ArrayHandle::new(vec![
                            Value::Undefined;
                            length as usize
                        ])))
                    }
                    other => Ok(Value::Array(ArrayHandle::new(array_from_value(
                        other, env, host,
                    )?))),
                }
            } else {
                Ok(Value::Array(ArrayHandle::new(Vec::new())))
            }
        }
        Expr::Identifier(name) if name == "Error" => {
            if args.len() > 1 {
                return Err(ScriptError::new("Error() expects at most one argument"));
            }

            let message = match args.first() {
                Some(expr) => as_string(&eval_expr(expr, env, host)?),
                None => String::new(),
            };

            let object = crate::ObjectHandle::new();
            object_set_property_value(
                &object,
                property_key_from_string("message"),
                Value::String(message),
                env,
                host,
            )?;
            object_set_property_value(
                &object,
                property_key_from_string("name"),
                Value::String("Error".to_string()),
                env,
                host,
            )?;
            Ok(Value::Object(object))
        }
        Expr::Identifier(name) if name == "Map" => {
            let map = crate::MapHandle::new();
            if args.is_empty() {
                return Ok(Value::Map(map));
            }
            if args.len() > 1 {
                return Err(ScriptError::new("Map() expects at most one argument"));
            }
            let entries = array_from_value(eval_expr(&args[0], env, host)?, env, host)?;
            for entry in entries {
                let Value::Array(pair) = entry else {
                    return Err(ScriptError::new("Map() expects iterable pairs"));
                };
                let items = pair.0.borrow().items.clone();
                if items.is_empty() {
                    return Err(ScriptError::new("Map() expects iterable pairs"));
                }
                let key = items.first().cloned().unwrap_or(Value::Undefined);
                let value = items.get(1).cloned().unwrap_or(Value::Undefined);
                let map_key = map_key_from_value(&key);
                let mut state = map.0.borrow_mut();
                if let Some((_, existing)) = state
                    .entries
                    .iter_mut()
                    .find(|(candidate, _)| *candidate == map_key)
                {
                    *existing = value;
                } else {
                    state.entries.push((map_key, value));
                }
            }
            Ok(Value::Map(map))
        }
        Expr::Identifier(name) if name == "Date" => {
            let value = date_constructor(args, env, host)?;
            Ok(value)
        }
        Expr::Member { object, property } => {
            if matches!(eval_expr(object, env, host)?, Value::IntlNamespace)
                && matches!(
                    property.as_str(),
                    "NumberFormat" | "DateTimeFormat" | "Collator"
                )
            {
                return intl_construct(property, args, env, host);
            }
            Err(ScriptError::new("invalid new target"))
        }
        _ => Err(ScriptError::new("invalid new target")),
    }
}

fn values_array(items: Vec<Value>) -> Value {
    Value::Array(ArrayHandle::new(items))
}

fn map_key_to_value(key: &MapKey) -> Value {
    match key {
        MapKey::Undefined => Value::Undefined,
        MapKey::Null => Value::Null,
        MapKey::Boolean(value) => Value::Boolean(*value),
        MapKey::Number(bits) => Value::Number(f64::from_bits(*bits)),
        MapKey::String(value) => Value::String(value.clone()),
        MapKey::Symbol(id) => Value::Symbol(crate::SymbolValue::from_parts(*id, None)),
        MapKey::Object(id) => Value::String(format!("[object Object:{id}]")),
        MapKey::Array(id) => Value::String(format!("[object Array:{id}]")),
        MapKey::Map(id) => Value::String(format!("[object Map:{id}]")),
    }
}

fn object_namespace_assign<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.is_empty() {
        return Err(ScriptError::new(
            "Object.assign() expects at least one argument",
        ));
    }

    let target_value = eval_expr(&args[0], env, host)?;
    let target = match target_value {
        Value::Object(object) => object,
        Value::Undefined | Value::Null => {
            return Err(ScriptError::new(
                "Cannot convert undefined or null to object",
            ));
        }
        _ => crate::ObjectHandle::new(),
    };

    for source_expr in &args[1..] {
        let source = eval_expr(source_expr, env, host)?;
        object_assign_from_source(&target, &source, env, host)?;
    }

    Ok(Value::Object(target))
}

fn object_namespace_keys<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [source_expr] = args else {
        return Err(ScriptError::new(
            "Object.keys() expects exactly one argument",
        ));
    };
    let source = eval_expr(source_expr, env, host)?;
    let keys = object_own_string_values(&source, env, host)?
        .into_iter()
        .map(|(key, _)| Value::String(key))
        .collect();
    Ok(values_array(keys))
}

fn object_namespace_values<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [source_expr] = args else {
        return Err(ScriptError::new(
            "Object.values() expects exactly one argument",
        ));
    };
    let source = eval_expr(source_expr, env, host)?;
    let values = object_own_string_values(&source, env, host)?
        .into_iter()
        .map(|(_, value)| value)
        .collect();
    Ok(values_array(values))
}

fn object_namespace_entries<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [source_expr] = args else {
        return Err(ScriptError::new(
            "Object.entries() expects exactly one argument",
        ));
    };
    let source = eval_expr(source_expr, env, host)?;
    let entries = object_own_string_values(&source, env, host)?
        .into_iter()
        .map(|(key, value)| Value::Array(ArrayHandle::new(vec![Value::String(key), value])))
        .collect();
    Ok(values_array(entries))
}

fn object_namespace_from_entries<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [entries_expr] = args else {
        return Err(ScriptError::new(
            "Object.fromEntries() expects exactly one argument",
        ));
    };
    let entries = array_from_value(eval_expr(entries_expr, env, host)?, env, host)?;
    let object = crate::ObjectHandle::new();
    for entry in entries {
        let Value::Array(pair) = entry else {
            return Err(ScriptError::new(
                "Object.fromEntries() expects iterable pairs",
            ));
        };
        let items = pair.0.borrow().items.clone();
        if items.is_empty() {
            return Err(ScriptError::new(
                "Object.fromEntries() expects iterable pairs",
            ));
        }
        let key = property_key_from_value(&items[0]);
        let value = items.get(1).cloned().unwrap_or(Value::Undefined);
        object_set_property_value(&object, key, value, env, host)?;
    }
    Ok(Value::Object(object))
}

fn object_namespace_get_own_property_symbols<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [source_expr] = args else {
        return Err(ScriptError::new(
            "Object.getOwnPropertySymbols() expects exactly one argument",
        ));
    };
    let source = eval_expr(source_expr, env, host)?;
    let symbols = object_own_symbol_values(&source, env, host)?
        .into_iter()
        .map(Value::Symbol)
        .collect();
    Ok(values_array(symbols))
}

fn string_split<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 2 {
        return Err(ScriptError::new("split() expects at most two arguments"));
    }
    let separator = match args.first() {
        Some(expr) => eval_expr(expr, env, host)?,
        None => Value::Undefined,
    };
    let limit = match args.get(1) {
        Some(expr) => index_from_value(&eval_expr(expr, env, host)?).unwrap_or(usize::MAX),
        None => usize::MAX,
    };
    let items = match separator {
        Value::Undefined | Value::Null => {
            if limit == 0 {
                Vec::new()
            } else {
                vec![Value::String(value.to_string())]
            }
        }
        Value::RegExp(regex) => regex_split(value, &regex, limit)?,
        other => {
            let separator = as_string(&other);
            if limit == 0 {
                Vec::new()
            } else if separator.is_empty() {
                value
                    .chars()
                    .take(limit)
                    .map(|ch| Value::String(ch.to_string()))
                    .collect()
            } else {
                value
                    .split(&separator)
                    .take(limit)
                    .map(|part| Value::String(part.to_string()))
                    .collect()
            }
        }
    };
    Ok(Value::Array(ArrayHandle::new(items)))
}

fn string_replace<H: HostBindings>(
    value: &str,
    args: &[Expr],
    all: bool,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() != 2 {
        return Err(ScriptError::new(if all {
            "replaceAll() expects exactly two arguments"
        } else {
            "replace() expects exactly two arguments"
        }));
    }

    let search = eval_expr(&args[0], env, host)?;
    let replacement = eval_expr(&args[1], env, host)?;

    match search {
        Value::RegExp(regex) => regex_replace(value, &regex, replacement, all, env, host),
        other => {
            let search = as_string(&other);
            let mut output = String::new();
            let mut remainder = value;
            let mut offset = 0usize;
            let mut replaced_any = false;

            while let Some(index) = remainder.find(&search) {
                replaced_any = true;
                output.push_str(&remainder[..index]);
                let matched = &remainder[index..index + search.len()];
                let replacement_value = match &replacement {
                    Value::Function(function) => call_script_function_value(
                        function,
                        &[
                            Value::String(matched.to_string()),
                            Value::Number((offset + index) as f64),
                            Value::String(value.to_string()),
                        ],
                        Value::Undefined,
                        env,
                        host,
                    )?,
                    other => other.clone(),
                };
                output.push_str(&as_string(&replacement_value));
                remainder = &remainder[index + search.len()..];
                offset += index + search.len();
                if !all {
                    break;
                }
            }

            if replaced_any {
                output.push_str(remainder);
                Ok(Value::String(output))
            } else if all {
                Ok(Value::String(value.to_string()))
            } else {
                Ok(Value::String(value.replacen(
                    &search,
                    &as_string(&replacement),
                    1,
                )))
            }
        }
    }
}

fn string_match<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [search_expr] = args else {
        return Err(ScriptError::new("match() expects exactly one argument"));
    };
    let search = eval_expr(search_expr, env, host)?;
    match search {
        Value::RegExp(regex) => regex_match(value, &regex),
        other => {
            let search = as_string(&other);
            if let Some(index) = value.find(&search) {
                Ok(Value::Array(ArrayHandle::new(vec![Value::String(
                    value[index..index + search.len()].to_string(),
                )])))
            } else {
                Ok(Value::Null)
            }
        }
    }
}

fn string_search<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [search_expr] = args else {
        return Err(ScriptError::new("search() expects exactly one argument"));
    };
    let search = eval_expr(search_expr, env, host)?;
    match search {
        Value::RegExp(regex) => regex_search(value, &regex),
        other => {
            let search = as_string(&other);
            Ok(Value::Number(
                value
                    .find(&search)
                    .map(|index| index as f64)
                    .unwrap_or(-1.0),
            ))
        }
    }
}

fn regex_builder(regex: &crate::RegExpValue) -> Result<FancyRegex> {
    let mut builder = RegexBuilder::new(regex.pattern());
    builder.case_insensitive(regex.is_ignore_case());
    builder.multi_line(regex.is_multiline());
    builder.dot_matches_new_line(regex.is_dot_all());
    builder
        .build()
        .map_err(|error| ScriptError::new(format!("invalid regular expression: {error}")))
}

fn regex_capture_values(_text: &str, captures: &fancy_regex::Captures<'_>) -> Vec<Value> {
    let mut values = Vec::new();
    values.push(Value::String(
        captures
            .get(0)
            .map(|matched| matched.as_str().to_string())
            .unwrap_or_default(),
    ));
    for index in 1..captures.len() {
        values.push(match captures.get(index) {
            Some(matched) => Value::String(matched.as_str().to_string()),
            None => Value::Undefined,
        });
    }
    values
}

fn regex_exec(value: &str, regex: &crate::RegExpValue) -> Result<Value> {
    let compiled = regex_builder(regex)?;
    let captures = compiled
        .captures(value)
        .map_err(|error| ScriptError::new(format!("invalid regular expression: {error}")))?;
    Ok(match captures {
        Some(captures) => Value::Array(ArrayHandle::new(regex_capture_values(value, &captures))),
        None => Value::Null,
    })
}

fn regex_test(value: &str, regex: &crate::RegExpValue) -> Result<Value> {
    let compiled = regex_builder(regex)?;
    let matched = compiled
        .is_match(value)
        .map_err(|error| ScriptError::new(format!("invalid regular expression: {error}")))?;
    Ok(Value::Boolean(matched))
}

fn regex_match(value: &str, regex: &crate::RegExpValue) -> Result<Value> {
    let compiled = regex_builder(regex)?;
    if regex.is_global() {
        let mut values = Vec::new();
        for result in compiled.find_iter(value) {
            let matched = result.map_err(|error| {
                ScriptError::new(format!("invalid regular expression: {error}"))
            })?;
            values.push(Value::String(matched.as_str().to_string()));
        }
        if values.is_empty() {
            Ok(Value::Null)
        } else {
            Ok(Value::Array(ArrayHandle::new(values)))
        }
    } else {
        let captures = compiled
            .captures(value)
            .map_err(|error| ScriptError::new(format!("invalid regular expression: {error}")))?;
        Ok(match captures {
            Some(captures) => {
                Value::Array(ArrayHandle::new(regex_capture_values(value, &captures)))
            }
            None => Value::Null,
        })
    }
}

fn regex_search(value: &str, regex: &crate::RegExpValue) -> Result<Value> {
    let compiled = regex_builder(regex)?;
    let matched = compiled
        .find(value)
        .map_err(|error| ScriptError::new(format!("invalid regular expression: {error}")))?;
    Ok(match matched {
        Some(matched) => Value::Number(matched.start() as f64),
        None => Value::Number(-1.0),
    })
}

fn regex_split(value: &str, regex: &crate::RegExpValue, limit: usize) -> Result<Vec<Value>> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let compiled = regex_builder(regex)?;
    let mut items = Vec::new();
    let mut last_end = 0usize;
    for result in compiled.find_iter(value) {
        if items.len() + 1 == limit {
            break;
        }
        let matched = result
            .map_err(|error| ScriptError::new(format!("invalid regular expression: {error}")))?;
        items.push(Value::String(value[last_end..matched.start()].to_string()));
        last_end = matched.end();
    }

    if items.len() < limit {
        items.push(Value::String(value[last_end..].to_string()));
    }

    Ok(items)
}

fn regex_replace<H: HostBindings>(
    value: &str,
    regex: &crate::RegExpValue,
    replacement: Value,
    all: bool,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if all && !regex.is_global() {
        return Err(ScriptError::new(
            "replaceAll() expects a global regular expression",
        ));
    }

    let compiled = regex_builder(regex)?;
    let mut output = String::new();
    let mut last_end = 0usize;
    let mut replaced_any = false;
    for result in compiled.captures_iter(value) {
        let captures = result
            .map_err(|error| ScriptError::new(format!("invalid regular expression: {error}")))?;
        let Some(matched) = captures.get(0) else {
            continue;
        };
        replaced_any = true;
        output.push_str(&value[last_end..matched.start()]);
        let mut args = Vec::new();
        args.push(Value::String(matched.as_str().to_string()));
        for index in 1..captures.len() {
            args.push(match captures.get(index) {
                Some(capture) => Value::String(capture.as_str().to_string()),
                None => Value::Undefined,
            });
        }
        args.push(Value::Number(matched.start() as f64));
        args.push(Value::String(value.to_string()));
        let replacement_value = match &replacement {
            Value::Function(function) => {
                call_script_function_value(function, &args, Value::Undefined, env, host)?
            }
            other => other.clone(),
        };
        output.push_str(&as_string(&replacement_value));
        last_end = matched.end();
        if !regex.is_global() {
            break;
        }
    }

    if replaced_any {
        output.push_str(&value[last_end..]);
        Ok(Value::String(output))
    } else {
        Ok(Value::String(value.to_string()))
    }
}

fn string_contains<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [search_expr] = args else {
        return Err(ScriptError::new("includes() expects exactly one argument"));
    };
    let search = as_string(&eval_expr(search_expr, env, host)?);
    Ok(Value::Boolean(value.contains(&search)))
}

fn string_starts_with<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [search_expr] = args else {
        return Err(ScriptError::new(
            "startsWith() expects exactly one argument",
        ));
    };
    let search = as_string(&eval_expr(search_expr, env, host)?);
    Ok(Value::Boolean(value.starts_with(&search)))
}

fn string_ends_with<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [search_expr] = args else {
        return Err(ScriptError::new("endsWith() expects exactly one argument"));
    };
    let search = as_string(&eval_expr(search_expr, env, host)?);
    Ok(Value::Boolean(value.ends_with(&search)))
}

fn string_index_of<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [search_expr] = args else {
        return Err(ScriptError::new("indexOf() expects exactly one argument"));
    };
    let search = as_string(&eval_expr(search_expr, env, host)?);
    Ok(Value::Number(
        value
            .find(&search)
            .map(|index| index as f64)
            .unwrap_or(-1.0),
    ))
}

fn string_last_index_of<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [search_expr] = args else {
        return Err(ScriptError::new(
            "lastIndexOf() expects exactly one argument",
        ));
    };
    let search = as_string(&eval_expr(search_expr, env, host)?);
    Ok(Value::Number(
        value
            .rfind(&search)
            .map(|index| index as f64)
            .unwrap_or(-1.0),
    ))
}

fn string_slice<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 2 {
        return Err(ScriptError::new("slice() expects at most two arguments"));
    }
    let chars: Vec<char> = value.chars().collect();
    let len = chars.len() as i64;
    let start = match args.first() {
        Some(expr) => integer_from_value(&eval_expr(expr, env, host)?, "slice()")?,
        None => 0,
    };
    let end = match args.get(1) {
        Some(expr) => integer_from_value(&eval_expr(expr, env, host)?, "slice()")?,
        None => len,
    };
    let start = if start < 0 {
        (len + start).max(0)
    } else {
        start.min(len)
    } as usize;
    let end = if end < 0 {
        (len + end).max(0)
    } else {
        end.min(len)
    } as usize;
    Ok(Value::String(
        chars[start.min(chars.len())..end.min(chars.len())]
            .iter()
            .collect(),
    ))
}

fn string_substring<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 2 {
        return Err(ScriptError::new(
            "substring() expects at most two arguments",
        ));
    }
    let chars: Vec<char> = value.chars().collect();
    let len = chars.len() as i64;
    let mut start = match args.first() {
        Some(expr) => integer_from_value(&eval_expr(expr, env, host)?, "substring()")?,
        None => 0,
    };
    let mut end = match args.get(1) {
        Some(expr) => integer_from_value(&eval_expr(expr, env, host)?, "substring()")?,
        None => len,
    };
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }
    let start = start.clamp(0, len) as usize;
    let end = end.clamp(0, len) as usize;
    Ok(Value::String(chars[start..end].iter().collect()))
}

fn string_char_at<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 1 {
        return Err(ScriptError::new("charAt() expects at most one argument"));
    }
    let index = match args.first() {
        Some(expr) => index_from_value(&eval_expr(expr, env, host)?).unwrap_or(0),
        None => 0,
    };
    Ok(Value::String(
        value
            .chars()
            .nth(index)
            .map(|ch| ch.to_string())
            .unwrap_or_default(),
    ))
}

fn string_char_code_at<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 1 {
        return Err(ScriptError::new(
            "charCodeAt() expects at most one argument",
        ));
    }
    let index = match args.first() {
        Some(expr) => index_from_value(&eval_expr(expr, env, host)?).unwrap_or(0),
        None => 0,
    };
    Ok(match value.chars().nth(index) {
        Some(ch) => Value::Number(ch as u32 as f64),
        None => Value::Number(f64::NAN),
    })
}

fn string_concat<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let mut out = value.to_string();
    for expr in args {
        out.push_str(&as_string(&eval_expr(expr, env, host)?));
    }
    Ok(Value::String(out))
}

fn string_repeat<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [count_expr] = args else {
        return Err(ScriptError::new("repeat() expects exactly one argument"));
    };
    let count = index_from_value(&eval_expr(count_expr, env, host)?)
        .ok_or_else(|| ScriptError::new("repeat() expects a non-negative integer"))?;
    Ok(Value::String(value.repeat(count)))
}

fn string_pad_start<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [target_expr, ..] = args else {
        return Err(ScriptError::new("padStart() expects at least one argument"));
    };
    let target_length = index_from_value(&eval_expr(target_expr, env, host)?)
        .ok_or_else(|| ScriptError::new("padStart() expects a non-negative integer"))?;
    let value_chars: Vec<char> = value.chars().collect();
    if target_length <= value_chars.len() {
        return Ok(Value::String(value.to_string()));
    }

    let pad_string = match args.get(1) {
        Some(expr) => as_string(&eval_expr(expr, env, host)?),
        None => " ".to_string(),
    };
    if pad_string.is_empty() {
        return Ok(Value::String(value.to_string()));
    }

    let pad_needed = target_length - value_chars.len();
    let mut prefix = String::new();
    while prefix.chars().count() < pad_needed {
        prefix.push_str(&pad_string);
    }
    let prefix: String = prefix.chars().take(pad_needed).collect();
    Ok(Value::String(format!("{prefix}{value}")))
}

fn string_normalize<H: HostBindings>(
    value: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 1 {
        return Err(ScriptError::new("normalize() expects at most one argument"));
    }
    let form = match args.first() {
        Some(expr) => as_string(&eval_expr(expr, env, host)?),
        None => "NFC".to_string(),
    };
    let normalized = match form.as_str() {
        "NFC" => value.nfc().collect::<String>(),
        "NFD" => value.nfd().collect::<String>(),
        "NFKC" => value.nfkc().collect::<String>(),
        "NFKD" => value.nfkd().collect::<String>(),
        other => {
            return Err(ScriptError::new(format!(
                "normalize() received unsupported form: {other}"
            )));
        }
    };
    Ok(Value::String(normalized))
}

fn string_namespace_from_char_code<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let values = eval_call_argument_values(args, env, host)?;
    let mut out = String::new();
    for value in values {
        let code = number_from_value(&value).unwrap_or(0.0);
        let code_unit = (code as u32) & 0xFFFF;
        out.push(char::from_u32(code_unit).unwrap_or('\u{FFFD}'));
    }
    Ok(Value::String(out))
}

fn number_namespace_is_finite<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [value_expr] = args else {
        return Err(ScriptError::new(
            "Number.isFinite() expects exactly one argument",
        ));
    };
    Ok(Value::Boolean(matches!(
        eval_expr(value_expr, env, host)?,
        Value::Number(value) if value.is_finite()
    )))
}

fn number_namespace_is_nan<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [value_expr] = args else {
        return Err(ScriptError::new(
            "Number.isNaN() expects exactly one argument",
        ));
    };
    Ok(Value::Boolean(matches!(
        eval_expr(value_expr, env, host)?,
        Value::Number(value) if value.is_nan()
    )))
}

fn number_namespace_parse_float<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [value_expr] = args else {
        return Err(ScriptError::new(
            "Number.parseFloat() expects exactly one argument",
        ));
    };
    let value = as_string(&eval_expr(value_expr, env, host)?);
    Ok(Value::Number(
        value.trim().parse::<f64>().unwrap_or(f64::NAN),
    ))
}

fn number_namespace_parse_int<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(ScriptError::new(
            "Number.parseInt() expects one or two arguments",
        ));
    }
    let value = as_string(&eval_expr(&args[0], env, host)?);
    let radix = match args.get(1) {
        Some(expr) => index_from_value(&eval_expr(expr, env, host)?).unwrap_or(10) as u32,
        None => 10,
    };
    let parsed = match radix {
        10 => value.trim().parse::<i64>().ok().map(|value| value as f64),
        16 => i64::from_str_radix(value.trim_start_matches("0x").trim_start_matches("0X"), 16)
            .ok()
            .map(|value| value as f64),
        _ => i64::from_str_radix(value.trim(), radix)
            .ok()
            .map(|value| value as f64),
    };
    Ok(Value::Number(parsed.unwrap_or(f64::NAN)))
}

fn number_method_to_fixed<H: HostBindings>(
    value: f64,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 1 {
        return Err(ScriptError::new(
            "Number.prototype.toFixed() expects at most one argument",
        ));
    }

    let digits = match args.first() {
        Some(expr) => {
            let digits_value = eval_expr(expr, env, host)?;
            match digits_value {
                Value::Undefined | Value::Null => 0.0,
                Value::Number(number) => number,
                Value::Boolean(value) => {
                    if value {
                        1.0
                    } else {
                        0.0
                    }
                }
                Value::String(value) => {
                    if value.trim().is_empty() {
                        0.0
                    } else {
                        value.trim().parse::<f64>().map_err(|_| {
                            ScriptError::new("Number.prototype.toFixed() digits must be numeric")
                        })?
                    }
                }
                _ => {
                    return Err(ScriptError::new(
                        "cannot convert value to fixed-point digits",
                    ));
                }
            }
        }
        None => 0.0,
    };

    if !digits.is_finite() {
        return Err(ScriptError::new(
            "Number.prototype.toFixed() digits must be finite",
        ));
    }

    let digits = digits.trunc();
    if !(0.0..=100.0).contains(&digits) {
        return Err(ScriptError::new(
            "Number.prototype.toFixed() digits must be between 0 and 100",
        ));
    }

    let precision = digits as usize;
    let formatted = if value.is_nan() {
        "NaN".to_string()
    } else if value.is_infinite() {
        if value.is_sign_negative() {
            "-Infinity".to_string()
        } else {
            "Infinity".to_string()
        }
    } else {
        let normalized = if value == 0.0 { 0.0 } else { value };
        format!("{normalized:.precision$}", precision = precision)
    };

    Ok(Value::String(formatted))
}

fn number_method_to_exponential<H: HostBindings>(
    value: f64,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 1 {
        return Err(ScriptError::new(
            "Number.prototype.toExponential() expects at most one argument",
        ));
    }

    let fraction_digits = match args.first() {
        Some(expr) => {
            let digits_value = eval_expr(expr, env, host)?;
            match digits_value {
                Value::Undefined | Value::Null => 0.0,
                Value::Number(number) => number,
                Value::Boolean(value) => {
                    if value {
                        1.0
                    } else {
                        0.0
                    }
                }
                Value::String(value) => {
                    if value.trim().is_empty() {
                        0.0
                    } else {
                        value.trim().parse::<f64>().map_err(|_| {
                            ScriptError::new(
                                "Number.prototype.toExponential() digits must be numeric",
                            )
                        })?
                    }
                }
                _ => {
                    return Err(ScriptError::new(
                        "cannot convert value to exponential digits",
                    ));
                }
            }
        }
        None => 0.0,
    };

    if !fraction_digits.is_finite() {
        return Err(ScriptError::new(
            "Number.prototype.toExponential() digits must be finite",
        ));
    }

    let fraction_digits = fraction_digits.trunc();
    if !(0.0..=100.0).contains(&fraction_digits) {
        return Err(ScriptError::new(
            "Number.prototype.toExponential() digits must be between 0 and 100",
        ));
    }

    let fraction_digits = fraction_digits as usize;
    let formatted = if value.is_nan() {
        "NaN".to_string()
    } else if value.is_infinite() {
        if value.is_sign_negative() {
            "-Infinity".to_string()
        } else {
            "Infinity".to_string()
        }
    } else {
        let normalized = if value == 0.0 { 0.0 } else { value.abs() };
        let (mantissa, exponent) = scientific_notation_components(normalized, fraction_digits);
        let mut output = scientific_notation_string(&mantissa, exponent);
        if value.is_sign_negative() && value != 0.0 {
            output.insert(0, '-');
        }
        output
    };

    Ok(Value::String(formatted))
}

fn number_method_to_precision<H: HostBindings>(
    value: f64,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 1 {
        return Err(ScriptError::new(
            "Number.prototype.toPrecision() expects at most one argument",
        ));
    }

    if args.is_empty() {
        return Ok(Value::String(as_string(&Value::Number(value))));
    }

    let precision = {
        let precision_value = eval_expr(&args[0], env, host)?;
        match precision_value {
            Value::Undefined | Value::Null => 1.0,
            Value::Number(number) => number,
            Value::Boolean(value) => {
                if value {
                    1.0
                } else {
                    0.0
                }
            }
            Value::String(value) => {
                if value.trim().is_empty() {
                    1.0
                } else {
                    value.trim().parse::<f64>().map_err(|_| {
                        ScriptError::new("Number.prototype.toPrecision() precision must be numeric")
                    })?
                }
            }
            _ => {
                return Err(ScriptError::new("cannot convert value to precision digits"));
            }
        }
    };

    if !precision.is_finite() {
        return Err(ScriptError::new(
            "Number.prototype.toPrecision() precision must be finite",
        ));
    }

    let precision = precision.trunc();
    if !(1.0..=100.0).contains(&precision) {
        return Err(ScriptError::new(
            "Number.prototype.toPrecision() precision must be between 1 and 100",
        ));
    }

    let precision = precision as usize;
    let formatted = if value.is_nan() {
        "NaN".to_string()
    } else if value.is_infinite() {
        if value.is_sign_negative() {
            "-Infinity".to_string()
        } else {
            "Infinity".to_string()
        }
    } else if value == 0.0 {
        let (mantissa, exponent) = scientific_notation_components(0.0, precision.saturating_sub(1));
        scientific_notation_to_fixed(&mantissa, exponent)
    } else {
        let abs = value.abs();
        let (mantissa, exponent) = scientific_notation_components(abs, precision.saturating_sub(1));
        let mut output = if exponent < -6 || exponent >= precision as i32 {
            scientific_notation_string(&mantissa, exponent)
        } else {
            scientific_notation_to_fixed(&mantissa, exponent)
        };
        if value.is_sign_negative() {
            output.insert(0, '-');
        }
        output
    };

    Ok(Value::String(formatted))
}

fn scientific_notation_components(value: f64, fraction_digits: usize) -> (String, i32) {
    let raw = format!("{:.*e}", fraction_digits, value);
    let (mantissa, exponent) = raw.split_once('e').unwrap_or((raw.as_str(), "0"));
    let exponent = exponent.parse::<i32>().unwrap_or(0);
    (mantissa.to_string(), exponent)
}

fn scientific_notation_string(mantissa: &str, exponent: i32) -> String {
    format!("{mantissa}e{exponent:+}")
}

fn scientific_notation_to_fixed(mantissa: &str, exponent: i32) -> String {
    let digits = mantissa.replace('.', "");
    let decimal_pos = 1isize + exponent as isize;
    if decimal_pos <= 0 {
        let mut output = String::from("0.");
        output.push_str(&"0".repeat((-decimal_pos) as usize));
        output.push_str(&digits);
        output
    } else if decimal_pos as usize >= digits.len() {
        let mut output = digits;
        output.push_str(&"0".repeat(decimal_pos as usize - output.len()));
        output
    } else {
        let split = decimal_pos as usize;
        let mut output = String::new();
        output.push_str(&digits[..split]);
        output.push('.');
        output.push_str(&digits[split..]);
        output
    }
}

fn number_method_to_string<H: HostBindings>(
    value: f64,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 1 {
        return Err(ScriptError::new(
            "Number.prototype.toString() expects at most one argument",
        ));
    }

    let radix = match args.first() {
        Some(expr) => {
            let radix_value = eval_expr(expr, env, host)?;
            match radix_value {
                Value::Undefined | Value::Null => 10,
                other => integer_from_value(&other, "Number.prototype.toString() radix")? as u32,
            }
        }
        None => 10,
    };

    if !(2..=36).contains(&radix) {
        return Err(ScriptError::new(
            "Number.prototype.toString() radix must be between 2 and 36",
        ));
    }

    if value.is_nan() {
        return Ok(Value::String("NaN".to_string()));
    }
    if value.is_infinite() {
        return Ok(Value::String(if value.is_sign_negative() {
            "-Infinity".to_string()
        } else {
            "Infinity".to_string()
        }));
    }

    if radix == 10 {
        return Ok(Value::String(as_string(&Value::Number(value))));
    }

    let negative = value.is_sign_negative() && value != 0.0;
    let mut integer = value.abs().trunc() as u128;
    let digits = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = String::new();
    if integer == 0 {
        out.push('0');
    } else {
        while integer > 0 {
            let digit = (integer % radix as u128) as usize;
            out.push(digits[digit] as char);
            integer /= radix as u128;
        }
        out = out.chars().rev().collect();
    }

    if negative {
        out.insert(0, '-');
    }
    Ok(Value::String(out))
}

fn math_namespace_min<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let values = eval_call_argument_values(args, env, host)?;
    let mut result = f64::INFINITY;
    for value in values {
        result = result.min(number_from_value(&value)?);
    }
    Ok(Value::Number(result))
}

fn math_namespace_max<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let values = eval_call_argument_values(args, env, host)?;
    let mut result = f64::NEG_INFINITY;
    for value in values {
        result = result.max(number_from_value(&value)?);
    }
    Ok(Value::Number(result))
}

fn math_namespace_unary<H: HostBindings, F: FnOnce(f64) -> f64>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    name: &str,
    func: F,
) -> Result<Value> {
    let [value_expr] = args else {
        return Err(ScriptError::new(format!(
            "{name}() expects exactly one argument"
        )));
    };
    let value = number_from_value(&eval_expr(value_expr, env, host)?)?;
    Ok(Value::Number(func(value)))
}

fn math_namespace_abs<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    math_namespace_unary(args, env, host, "Math.abs", f64::abs)
}

fn math_namespace_floor<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    math_namespace_unary(args, env, host, "Math.floor", f64::floor)
}

fn math_namespace_ceil<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    math_namespace_unary(args, env, host, "Math.ceil", f64::ceil)
}

fn math_namespace_round<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    math_namespace_unary(args, env, host, "Math.round", f64::round)
}

fn math_namespace_trunc<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    math_namespace_unary(args, env, host, "Math.trunc", f64::trunc)
}

fn math_namespace_sign<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    math_namespace_unary(args, env, host, "Math.sign", f64::signum)
}

fn math_namespace_pow<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [base_expr, exponent_expr] = args else {
        return Err(ScriptError::new("Math.pow() expects exactly two arguments"));
    };
    let base = number_from_value(&eval_expr(base_expr, env, host)?)?;
    let exponent = number_from_value(&eval_expr(exponent_expr, env, host)?)?;
    Ok(Value::Number(base.powf(exponent)))
}

fn math_namespace_sqrt<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    math_namespace_unary(args, env, host, "Math.sqrt", f64::sqrt)
}

fn css_namespace_escape<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [value_expr] = args else {
        return Err(ScriptError::new(
            "CSS.escape() expects exactly one argument",
        ));
    };
    let value = as_string(&eval_expr(value_expr, env, host)?);
    Ok(Value::String(css_escape_ident(&value)))
}

fn date_namespace_now<H: HostBindings>(
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    _host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("Date.now() expects no arguments"));
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ScriptError::new("system clock is before the Unix epoch"))?;
    Ok(Value::Number(now.as_millis() as f64))
}

fn date_namespace_utc<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() < 2 {
        return Ok(Value::Number(f64::NAN));
    }
    Ok(Value::Number(
        date_epoch_from_component_args(args, env, host, false)?
            .map(|value| value as f64)
            .unwrap_or(f64::NAN),
    ))
}

fn date_namespace_parse<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [value_expr] = args else {
        return Err(ScriptError::new(
            "Date.parse() expects exactly one argument",
        ));
    };
    let value = as_string(&eval_expr(value_expr, env, host)?);
    Ok(Value::Number(
        date_parse_string(&value)
            .map(|value| value as f64)
            .unwrap_or(f64::NAN),
    ))
}

fn value_is_nullish(value: &Value) -> bool {
    matches!(value, Value::Null | Value::Undefined)
}

fn clip_epoch_ms(value: f64) -> Option<i64> {
    if value.is_finite() && value.abs() <= 8_640_000_000_000_000.0 {
        Some(value.trunc() as i64)
    } else {
        None
    }
}

fn date_epoch_ms_from_value(value: &Value) -> Option<i64> {
    match value {
        Value::Date(date) => date.epoch_ms,
        Value::String(value) => date_parse_string(value),
        other => number_from_value(other).ok().and_then(clip_epoch_ms),
    }
}

fn date_parse_string(value: &str) -> Option<i64> {
    if let Ok(datetime) = DateTime::parse_from_rfc3339(value) {
        return Some(datetime.timestamp_millis());
    }

    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return date
            .and_hms_milli_opt(0, 0, 0, 0)
            .map(|datetime| Utc.from_utc_datetime(&datetime).timestamp_millis());
    }

    None
}

fn date_datetime_utc(epoch_ms: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_millis_opt(epoch_ms).single()
}

fn date_datetime_local(epoch_ms: i64) -> Option<DateTime<Local>> {
    Local.timestamp_millis_opt(epoch_ms).single()
}

fn date_to_iso_string(epoch_ms: i64) -> Option<String> {
    let datetime = date_datetime_utc(epoch_ms)?;
    Some(datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

#[allow(dead_code)]
fn date_value_from_value(value: &Value) -> DateValue {
    DateValue {
        epoch_ms: date_epoch_ms_from_value(value),
    }
}

fn date_component_from_expr<H: HostBindings>(
    expr: Option<&Expr>,
    default: i64,
    required: bool,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Option<i64>> {
    match expr {
        Some(expr) => {
            let value = eval_expr(expr, env, host)?;
            if value_is_nullish(&value) {
                if required {
                    Ok(None)
                } else {
                    Ok(Some(default))
                }
            } else {
                Ok(number_from_value(&value).ok().and_then(clip_epoch_ms))
            }
        }
        None if required => Ok(None),
        None => Ok(Some(default)),
    }
}

fn date_epoch_from_component_args<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    local_time: bool,
) -> Result<Option<i64>> {
    if args.len() < 2 {
        return Ok(None);
    }

    let year = match date_component_from_expr(args.first(), 0, true, env, host)? {
        Some(year) => year,
        None => return Ok(None),
    };
    let month = match date_component_from_expr(args.get(1), 0, true, env, host)? {
        Some(month) => month,
        None => return Ok(None),
    };
    let day = match date_component_from_expr(args.get(2), 1, false, env, host)? {
        Some(day) => day,
        None => return Ok(None),
    };
    let hour = match date_component_from_expr(args.get(3), 0, false, env, host)? {
        Some(hour) => hour,
        None => return Ok(None),
    };
    let minute = match date_component_from_expr(args.get(4), 0, false, env, host)? {
        Some(minute) => minute,
        None => return Ok(None),
    };
    let second = match date_component_from_expr(args.get(5), 0, false, env, host)? {
        Some(second) => second,
        None => return Ok(None),
    };
    let millisecond = match date_component_from_expr(args.get(6), 0, false, env, host)? {
        Some(millisecond) => millisecond,
        None => return Ok(None),
    };

    let year = if (0..=99).contains(&year) {
        year + 1900
    } else {
        year
    };
    let Some(total_months) = year
        .checked_mul(12)
        .and_then(|value| value.checked_add(month))
    else {
        return Ok(None);
    };
    let normalized_year = total_months.div_euclid(12);
    let normalized_month = total_months.rem_euclid(12) + 1;
    let Some(normalized_year) = i32::try_from(normalized_year).ok() else {
        return Ok(None);
    };

    let Some(naive) = NaiveDate::from_ymd_opt(normalized_year, normalized_month as u32, 1)
        .and_then(|date| date.and_hms_milli_opt(0, 0, 0, 0))
    else {
        return Ok(None);
    };
    let naive = naive
        + Duration::days(day - 1)
        + Duration::hours(hour)
        + Duration::minutes(minute)
        + Duration::seconds(second)
        + Duration::milliseconds(millisecond);

    if local_time {
        match Local.from_local_datetime(&naive) {
            LocalResult::Single(datetime) => {
                Ok(Some(datetime.with_timezone(&Utc).timestamp_millis()))
            }
            LocalResult::Ambiguous(datetime, _) => {
                Ok(Some(datetime.with_timezone(&Utc).timestamp_millis()))
            }
            LocalResult::None => Ok(Some(Utc.from_utc_datetime(&naive).timestamp_millis())),
        }
    } else {
        Ok(Some(Utc.from_utc_datetime(&naive).timestamp_millis()))
    }
}

fn date_zoned_parts(
    epoch_ms: i64,
    time_zone: Option<&str>,
) -> Option<(i32, u32, u32, u32, u32, u32, i32)> {
    let utc = date_datetime_utc(epoch_ms)?;
    match time_zone {
        Some(time_zone) if !time_zone.trim().is_empty() => {
            let zone = Tz::from_str(time_zone).ok()?;
            let datetime = utc.with_timezone(&zone);
            let offset_minutes = datetime.offset().fix().local_minus_utc() / 60;
            Some((
                datetime.year(),
                datetime.month(),
                datetime.day(),
                datetime.hour(),
                datetime.minute(),
                datetime.second(),
                offset_minutes,
            ))
        }
        _ => {
            let datetime = utc.with_timezone(&Local);
            let offset_minutes = datetime.offset().fix().local_minus_utc() / 60;
            Some((
                datetime.year(),
                datetime.month(),
                datetime.day(),
                datetime.hour(),
                datetime.minute(),
                datetime.second(),
                offset_minutes,
            ))
        }
    }
}

fn date_component_string(value: u32, style: Option<&str>) -> String {
    if matches!(style, Some("2-digit")) {
        format!("{value:02}")
    } else {
        value.to_string()
    }
}

fn date_format_parts(
    epoch_ms: i64,
    formatter: &IntlDateTimeFormatValue,
) -> Option<Vec<(String, String)>> {
    let (year, month, day, hour, minute, second, _) =
        date_zoned_parts(epoch_ms, formatter.time_zone.as_deref())?;

    let mut parts = Vec::new();
    let year_style = formatter.year.as_deref();
    let month_style = formatter.month.as_deref();
    let day_style = formatter.day.as_deref();
    let hour_style = formatter.hour.as_deref();
    let minute_style = formatter.minute.as_deref();
    let second_style = formatter.second.as_deref();
    let use_hour12 = formatter.hour12.unwrap_or(false);

    let hour_value = if use_hour12 {
        let hour = hour % 12;
        if hour == 0 { 12 } else { hour }
    } else {
        hour
    };

    let want_date = year_style.is_some() || month_style.is_some() || day_style.is_some();
    let want_time = hour_style.is_some() || minute_style.is_some() || second_style.is_some();

    if let Some(style) = year_style {
        parts.push((
            "year".to_string(),
            date_component_string(year as u32, Some(style)),
        ));
    }
    if let Some(style) = month_style {
        if !parts.is_empty() {
            parts.push(("literal".to_string(), "-".to_string()));
        }
        parts.push((
            "month".to_string(),
            date_component_string(month, Some(style)),
        ));
    }
    if let Some(style) = day_style {
        if !parts.is_empty() {
            parts.push(("literal".to_string(), "-".to_string()));
        }
        parts.push(("day".to_string(), date_component_string(day, Some(style))));
    }
    if want_date && want_time {
        parts.push(("literal".to_string(), " ".to_string()));
    }
    if let Some(style) = hour_style {
        parts.push((
            "hour".to_string(),
            date_component_string(hour_value, Some(style)),
        ));
    }
    if let Some(style) = minute_style {
        if !parts.is_empty() {
            parts.push(("literal".to_string(), ":".to_string()));
        }
        parts.push((
            "minute".to_string(),
            date_component_string(minute, Some(style)),
        ));
    }
    if let Some(style) = second_style {
        if !parts.is_empty() {
            parts.push(("literal".to_string(), ":".to_string()));
        }
        parts.push((
            "second".to_string(),
            date_component_string(second, Some(style)),
        ));
    }
    if use_hour12 && hour_style.is_some() {
        if !parts.is_empty() {
            parts.push(("literal".to_string(), " ".to_string()));
        }
        parts.push((
            "dayPeriod".to_string(),
            if hour < 12 {
                "AM".to_string()
            } else {
                "PM".to_string()
            },
        ));
    }

    Some(parts)
}

fn date_format_string(epoch_ms: i64, formatter: &IntlDateTimeFormatValue) -> Option<String> {
    let parts = date_format_parts(epoch_ms, formatter)?;
    Some(parts.into_iter().map(|(_, value)| value).collect())
}

#[allow(dead_code)]
fn date_format_locale_string(epoch_ms: i64, locale: &str, include_time: bool) -> Option<String> {
    let formatter = IntlDateTimeFormatValue {
        locale: locale.to_string(),
        time_zone: None,
        year: Some("numeric".to_string()),
        month: Some("numeric".to_string()),
        day: Some("numeric".to_string()),
        hour: if include_time {
            Some("numeric".to_string())
        } else {
            None
        },
        minute: if include_time {
            Some("2-digit".to_string())
        } else {
            None
        },
        second: if include_time {
            Some("2-digit".to_string())
        } else {
            None
        },
        hour12: if include_time { Some(true) } else { None },
    };

    let mut parts = date_format_parts(epoch_ms, &formatter)?;
    if locale.starts_with("en-GB") {
        let (year, month, day, hour, minute, second, _) = date_zoned_parts(epoch_ms, None)?;
        let mut out = String::new();
        out.push_str(&date_component_string(day, Some("numeric")));
        out.push('/');
        out.push_str(&date_component_string(month, Some("numeric")));
        out.push('/');
        out.push_str(&date_component_string(year as u32, Some("numeric")));
        if include_time {
            let (display_hour, suffix) = match hour {
                0 => (12, "AM"),
                1..=11 => (hour, "AM"),
                12 => (12, "PM"),
                _ => (hour - 12, "PM"),
            };
            out.push_str(", ");
            out.push_str(&date_component_string(display_hour, Some("numeric")));
            out.push(':');
            out.push_str(&date_component_string(minute, Some("2-digit")));
            out.push(':');
            out.push_str(&date_component_string(second, Some("2-digit")));
            out.push(' ');
            out.push_str(suffix);
        }
        return Some(out);
    }

    Some(parts.drain(..).map(|(_, value)| value).collect())
}

fn object_value_from_pairs<H: HostBindings>(
    pairs: Vec<(&str, Value)>,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let object = crate::ObjectHandle::new();
    for (name, value) in pairs {
        object_set_property_value(&object, property_key_from_string(name), value, env, host)?;
    }
    Ok(Value::Object(object))
}

fn part_array_from_pairs<H: HostBindings>(
    parts: Vec<(String, String)>,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let mut items = Vec::new();
    for (part_type, part_value) in parts {
        let object = crate::ObjectHandle::new();
        object_set_property_value(
            &object,
            property_key_from_string("type"),
            Value::String(part_type),
            env,
            host,
        )?;
        object_set_property_value(
            &object,
            property_key_from_string("value"),
            Value::String(part_value),
            env,
            host,
        )?;
        items.push(Value::Object(object));
    }
    Ok(Value::Array(ArrayHandle::new(items)))
}

fn option_value_from_object<H: HostBindings>(
    options: Option<&Value>,
    name: &str,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Option<Value>> {
    match options {
        Some(options) => {
            property_value_on_value(options, &property_key_from_string(name), env, host)
        }
        None => Ok(None),
    }
}

fn option_string_from_object<H: HostBindings>(
    options: Option<&Value>,
    name: &str,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Option<String>> {
    Ok(match option_value_from_object(options, name, env, host)? {
        Some(value) if !value_is_nullish(&value) => Some(as_string(&value)),
        _ => None,
    })
}

fn option_bool_from_object<H: HostBindings>(
    options: Option<&Value>,
    name: &str,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Option<bool>> {
    Ok(match option_value_from_object(options, name, env, host)? {
        Some(value) if !value_is_nullish(&value) => Some(is_truthy(&value)),
        _ => None,
    })
}

fn option_usize_from_object<H: HostBindings>(
    options: Option<&Value>,
    name: &str,
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Option<usize>> {
    Ok(match option_value_from_object(options, name, env, host)? {
        Some(value) if !value_is_nullish(&value) => {
            let number = number_from_value(&value).ok();
            number.map(|number| number.trunc().max(0.0) as usize)
        }
        _ => None,
    })
}

fn currency_minor_digits(currency: Option<&str>) -> usize {
    match currency {
        Some("JPY") => 0,
        Some("KRW") => 0,
        Some("CLP") => 0,
        _ => 2,
    }
}

fn currency_symbol(currency: Option<&str>) -> Option<&'static str> {
    match currency {
        Some("JPY") => Some("￥"),
        Some("USD") => Some("$"),
        Some("EUR") => Some("€"),
        Some("GBP") => Some("£"),
        Some("CNY") => Some("¥"),
        Some("KRW") => Some("₩"),
        _ => None,
    }
}

fn group_integer_digits(integer: &str) -> String {
    let mut out = String::new();
    let mut count = 0usize;
    for ch in integer.chars().rev() {
        if count == 3 {
            out.push(',');
            count = 0;
        }
        out.push(ch);
        count += 1;
    }
    out.chars().rev().collect()
}

fn format_significant_number(value: f64, digits: usize) -> String {
    if value == 0.0 {
        return "0".to_string();
    }

    let digits = digits.max(1);
    let scientific = format!("{:.*e}", digits - 1, value.abs());
    let Some((mantissa, exponent)) = scientific.split_once('e') else {
        return scientific;
    };
    let exponent = exponent.parse::<i32>().unwrap_or(0);
    let mut mantissa_digits: String = mantissa.chars().filter(|ch| *ch != '.').collect();
    if mantissa_digits.is_empty() {
        mantissa_digits.push('0');
    }

    let decimal_index = 1 + exponent;
    if decimal_index <= 0 {
        let mut out = String::from("0.");
        out.push_str(&"0".repeat(decimal_index.unsigned_abs() as usize));
        out.push_str(&mantissa_digits);
        return out;
    }

    let decimal_index = decimal_index as usize;
    if decimal_index >= mantissa_digits.len() {
        let mut out = mantissa_digits;
        out.push_str(&"0".repeat(decimal_index - out.len()));
        return out;
    }

    let (left, right) = mantissa_digits.split_at(decimal_index);
    let mut out = left.to_string();
    out.push('.');
    out.push_str(right);
    out
}

fn format_decimal_number(value: f64, formatter: &IntlNumberFormatValue) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-Infinity".to_string()
        } else {
            "Infinity".to_string()
        };
    }

    let negative = value.is_sign_negative() && value != 0.0;
    let abs = value.abs();
    let mut body = if let Some(max_significant_digits) = formatter.maximum_significant_digits {
        format_significant_number(abs, max_significant_digits)
    } else if let Some(min_significant_digits) = formatter.minimum_significant_digits {
        format_significant_number(abs, min_significant_digits)
    } else {
        let default_fraction_digits = match formatter.style.as_str() {
            "currency" => currency_minor_digits(formatter.currency.as_deref()),
            _ => 3,
        };
        let min_fraction_digits = formatter
            .minimum_fraction_digits
            .unwrap_or(default_fraction_digits);
        let max_fraction_digits = formatter
            .maximum_fraction_digits
            .unwrap_or(default_fraction_digits)
            .max(min_fraction_digits);
        let precision = max_fraction_digits;
        let mut rendered = format!("{abs:.precision$}", precision = precision);
        if let Some((integer, fraction)) = rendered.split_once('.') {
            let fraction = fraction.trim_end_matches('0');
            let fraction = if fraction.len() < min_fraction_digits {
                let mut padded = fraction.to_string();
                padded.push_str(&"0".repeat(min_fraction_digits - fraction.len()));
                padded
            } else {
                fraction.to_string()
            };
            rendered = if fraction.is_empty() && min_fraction_digits == 0 {
                integer.to_string()
            } else {
                format!("{integer}.{fraction}")
            };
        } else if min_fraction_digits > 0 {
            rendered.push('.');
            rendered.push_str(&"0".repeat(min_fraction_digits));
        }
        rendered
    };

    if formatter.use_grouping {
        if let Some((integer, fraction)) = body.split_once('.') {
            let mut grouped = group_integer_digits(integer);
            grouped.push('.');
            grouped.push_str(fraction);
            body = grouped;
        } else {
            body = group_integer_digits(&body);
        }
    }

    if let Some(min_integer_digits) = formatter
        .minimum_integer_digits
        .checked_sub(body.split('.').next().map(|part| part.len()).unwrap_or(0))
    {
        if min_integer_digits > 0 {
            if let Some((integer, fraction)) = body.split_once('.') {
                let mut padded = "0".repeat(min_integer_digits);
                padded.push_str(integer);
                padded.push('.');
                padded.push_str(fraction);
                body = padded;
            } else {
                let mut padded = "0".repeat(min_integer_digits);
                padded.push_str(&body);
                body = padded;
            }
        }
    }

    let symbol = if formatter.style == "currency" {
        currency_symbol(formatter.currency.as_deref()).unwrap_or("")
    } else {
        ""
    };

    let mut out = String::new();
    if negative {
        out.push('-');
    }
    out.push_str(symbol);
    out.push_str(&body);
    out
}

fn format_number_parts(value: f64, formatter: &IntlNumberFormatValue) -> Vec<(String, String)> {
    let formatted = format_decimal_number(value, formatter);
    let mut parts = Vec::new();
    let mut rest = formatted.as_str();

    if let Some(stripped) = rest.strip_prefix('-') {
        parts.push(("minusSign".to_string(), "-".to_string()));
        rest = stripped;
    }

    if formatter.style == "currency" {
        if let Some(symbol) = currency_symbol(formatter.currency.as_deref()) {
            if let Some(stripped) = rest.strip_prefix(symbol) {
                parts.push(("currency".to_string(), symbol.to_string()));
                rest = stripped;
            }
        }
    }

    if let Some((integer, fraction)) = rest.split_once('.') {
        for segment in integer.split(',') {
            if !segment.is_empty() {
                parts.push(("integer".to_string(), segment.to_string()));
            }
            if segment != integer.rsplit(',').next().unwrap_or(integer) {
                parts.push(("group".to_string(), ",".to_string()));
            }
        }
        parts.push(("decimal".to_string(), ".".to_string()));
        if !fraction.is_empty() {
            parts.push(("fraction".to_string(), fraction.to_string()));
        }
    } else {
        for segment in rest.split(',') {
            if !segment.is_empty() {
                parts.push(("integer".to_string(), segment.to_string()));
            }
            if segment != rest.rsplit(',').next().unwrap_or(rest) {
                parts.push(("group".to_string(), ",".to_string()));
            }
        }
    }

    if parts.is_empty() {
        parts.push(("integer".to_string(), formatted));
    }

    parts
}

fn parse_intl_number_format<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<IntlNumberFormatValue> {
    let locale = match args.first() {
        Some(expr) => as_string(&eval_expr(expr, env, host)?),
        None => "en-US".to_string(),
    };
    let options = match args.get(1) {
        Some(expr) => Some(eval_expr(expr, env, host)?),
        None => None,
    };
    let style = option_string_from_object(options.as_ref(), "style", env, host)?
        .unwrap_or_else(|| "decimal".to_string());
    let currency = option_string_from_object(options.as_ref(), "currency", env, host)?;
    let use_grouping =
        option_bool_from_object(options.as_ref(), "useGrouping", env, host)?.unwrap_or(true);
    let minimum_integer_digits =
        option_usize_from_object(options.as_ref(), "minimumIntegerDigits", env, host)?
            .unwrap_or(1)
            .max(1);
    let minimum_fraction_digits =
        option_usize_from_object(options.as_ref(), "minimumFractionDigits", env, host)?;
    let maximum_fraction_digits =
        option_usize_from_object(options.as_ref(), "maximumFractionDigits", env, host)?;
    let minimum_significant_digits =
        option_usize_from_object(options.as_ref(), "minimumSignificantDigits", env, host)?;
    let maximum_significant_digits =
        option_usize_from_object(options.as_ref(), "maximumSignificantDigits", env, host)?;

    let default_fraction_digits = match style.as_str() {
        "currency" => currency_minor_digits(currency.as_deref()),
        _ => 0,
    };

    let minimum_fraction_digits = minimum_fraction_digits.or(Some(default_fraction_digits));
    let maximum_fraction_digits = maximum_fraction_digits.or(Some(match style.as_str() {
        "currency" => currency_minor_digits(currency.as_deref()),
        _ => 3,
    }));

    Ok(IntlNumberFormatValue {
        locale,
        style,
        currency,
        use_grouping,
        minimum_integer_digits,
        minimum_fraction_digits,
        maximum_fraction_digits: match (minimum_fraction_digits, maximum_fraction_digits) {
            (Some(min), Some(max)) => Some(min.max(max)),
            (Some(min), None) => Some(min),
            (None, Some(max)) => Some(max),
            (None, None) => None,
        },
        minimum_significant_digits,
        maximum_significant_digits,
    })
}

fn parse_intl_date_time_format<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<IntlDateTimeFormatValue> {
    let locale = match args.first() {
        Some(expr) => as_string(&eval_expr(expr, env, host)?),
        None => "en-US".to_string(),
    };
    let options = match args.get(1) {
        Some(expr) => Some(eval_expr(expr, env, host)?),
        None => None,
    };
    Ok(IntlDateTimeFormatValue {
        locale,
        time_zone: option_string_from_object(options.as_ref(), "timeZone", env, host)?,
        year: option_string_from_object(options.as_ref(), "year", env, host)?,
        month: option_string_from_object(options.as_ref(), "month", env, host)?,
        day: option_string_from_object(options.as_ref(), "day", env, host)?,
        hour: option_string_from_object(options.as_ref(), "hour", env, host)?,
        minute: option_string_from_object(options.as_ref(), "minute", env, host)?,
        second: option_string_from_object(options.as_ref(), "second", env, host)?,
        hour12: option_bool_from_object(options.as_ref(), "hour12", env, host)?,
    })
}

fn parse_intl_collator<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<IntlCollatorValue> {
    let locale = match args.first() {
        Some(expr) => as_string(&eval_expr(expr, env, host)?),
        None => "en-US".to_string(),
    };
    let options = match args.get(1) {
        Some(expr) => Some(eval_expr(expr, env, host)?),
        None => None,
    };
    Ok(IntlCollatorValue {
        locale,
        numeric: option_bool_from_object(options.as_ref(), "numeric", env, host)?.unwrap_or(false),
        sensitivity: option_string_from_object(options.as_ref(), "sensitivity", env, host)?,
        usage: option_string_from_object(options.as_ref(), "usage", env, host)?,
    })
}

fn intl_number_format_render(formatter: &IntlNumberFormatValue, value: f64) -> String {
    format_decimal_number(value, formatter)
}

fn intl_number_format_parts_for_value(
    formatter: &IntlNumberFormatValue,
    value: f64,
) -> Vec<(String, String)> {
    format_number_parts(value, formatter)
}

fn intl_date_time_format_render(
    formatter: &IntlDateTimeFormatValue,
    epoch_ms: i64,
) -> Option<String> {
    date_format_string(epoch_ms, formatter)
}

fn intl_date_time_format_parts_for_epoch(
    formatter: &IntlDateTimeFormatValue,
    epoch_ms: i64,
) -> Option<Vec<(String, String)>> {
    date_format_parts(epoch_ms, formatter)
}

fn intl_collator_compare_values(collator: &IntlCollatorValue, left: &str, right: &str) -> Ordering {
    if collator.numeric {
        let left_tokens = tokenize_collation_key(left, collator.locale.as_str());
        let right_tokens = tokenize_collation_key(right, collator.locale.as_str());
        return compare_collation_tokens(&left_tokens, &right_tokens);
    }

    compare_collation_strings(left, right, collator.locale.as_str())
}

fn tokenize_collation_key(value: &str, locale: &str) -> Vec<CollationToken> {
    let mut tokens = Vec::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_ascii_digit() {
            let mut digits = String::new();
            while let Some(next) = chars.peek().copied() {
                if next.is_ascii_digit() {
                    digits.push(next);
                    let _ = chars.next();
                } else {
                    break;
                }
            }
            tokens.push(CollationToken::Number(digits.parse::<u128>().unwrap_or(0)));
        } else {
            let mut text = String::new();
            while let Some(next) = chars.peek().copied() {
                if !next.is_ascii_digit() {
                    text.push(next);
                    let _ = chars.next();
                } else {
                    break;
                }
            }
            tokens.push(CollationToken::Text(collation_key_for_locale(
                &text, locale,
            )));
        }
    }
    tokens
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CollationToken {
    Number(u128),
    Text(String),
}

fn compare_collation_tokens(left: &[CollationToken], right: &[CollationToken]) -> Ordering {
    for (left_token, right_token) in left.iter().zip(right.iter()) {
        let ordering = match (left_token, right_token) {
            (CollationToken::Number(lhs), CollationToken::Number(rhs)) => lhs.cmp(rhs),
            (CollationToken::Text(lhs), CollationToken::Text(rhs)) => lhs.cmp(rhs),
            (CollationToken::Number(_), CollationToken::Text(_)) => Ordering::Less,
            (CollationToken::Text(_), CollationToken::Number(_)) => Ordering::Greater,
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    left.len().cmp(&right.len())
}

fn compare_collation_strings(left: &str, right: &str, locale: &str) -> Ordering {
    let left_key = collation_key_for_locale(left, locale);
    let right_key = collation_key_for_locale(right, locale);
    left_key.cmp(&right_key)
}

fn collation_key_for_locale(value: &str, locale: &str) -> String {
    if locale.starts_with("sv") {
        let mut out = String::new();
        for ch in value.chars() {
            match ch.to_lowercase().collect::<String>().as_str() {
                "å" => out.push_str("~a"),
                "ä" => out.push_str("~b"),
                "ö" => out.push_str("~c"),
                other => out.push_str(other),
            }
        }
        out
    } else {
        value.to_lowercase()
    }
}

fn date_to_locale_date_string<H: HostBindings>(
    date: &DateValue,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let locale = match args.first() {
        Some(expr) => as_string(&eval_expr(expr, env, host)?),
        None => "en-US".to_string(),
    };
    Ok(Value::String(match date.epoch_ms {
        Some(epoch_ms) => {
            let Some((year, month, day, _, _, _, _)) = date_zoned_parts(epoch_ms, None) else {
                return Ok(Value::String("Invalid Date".to_string()));
            };
            if locale.starts_with("en-GB") {
                format!("{}/{}/{}", day, month, year)
            } else {
                format!("{}/{}/{}", month, day, year)
            }
        }
        None => "Invalid Date".to_string(),
    }))
}

fn date_to_locale_string<H: HostBindings>(
    date: &DateValue,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let locale = match args.first() {
        Some(expr) => as_string(&eval_expr(expr, env, host)?),
        None => "en-US".to_string(),
    };
    Ok(Value::String(match date.epoch_ms {
        Some(epoch_ms) => {
            let Some((year, month, day, hour, minute, second, _)) =
                date_zoned_parts(epoch_ms, None)
            else {
                return Ok(Value::String("Invalid Date".to_string()));
            };
            let (display_hour, suffix) = match hour {
                0 => (12, "AM"),
                1..=11 => (hour, "AM"),
                12 => (12, "PM"),
                _ => (hour - 12, "PM"),
            };
            if locale.starts_with("en-GB") {
                format!(
                    "{}/{}/{} {:02}:{:02}:{:02} {}",
                    day, month, year, display_hour, minute, second, suffix
                )
            } else {
                format!(
                    "{}/{}/{} {:02}:{:02}:{:02} {}",
                    month, day, year, display_hour, minute, second, suffix
                )
            }
        }
        None => "Invalid Date".to_string(),
    }))
}

fn date_to_iso_string_method<H: HostBindings>(
    date: &DateValue,
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    _host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new(
            "Date.prototype.toISOString() expects no arguments",
        ));
    }
    match date.epoch_ms.and_then(date_to_iso_string) {
        Some(string) => Ok(Value::String(string)),
        None => Err(ScriptError::new("Invalid time value")),
    }
}

fn date_to_json<H: HostBindings>(
    date: &DateValue,
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    _host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new(
            "Date.prototype.toJSON() expects no arguments",
        ));
    }
    match date.epoch_ms.and_then(date_to_iso_string) {
        Some(string) => Ok(Value::String(string)),
        None => Ok(Value::Null),
    }
}

fn date_to_string<H: HostBindings>(
    date: &DateValue,
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    _host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new(
            "Date.prototype.toString() expects no arguments",
        ));
    }
    Ok(Value::String(match date.epoch_ms {
        Some(epoch_ms) => {
            if let Some((year, month, day, hour, minute, second, offset_minutes)) =
                date_zoned_parts(epoch_ms, None)
            {
                let sign = if offset_minutes < 0 { "-" } else { "+" };
                let offset_minutes = offset_minutes.abs();
                let offset_hours = offset_minutes / 60;
                let offset_remaining = offset_minutes % 60;
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02} GMT{}{:02}{:02}",
                    year, month, day, hour, minute, second, sign, offset_hours, offset_remaining
                )
            } else {
                "Invalid Date".to_string()
            }
        }
        None => "Invalid Date".to_string(),
    }))
}

fn date_value_of(date: &DateValue, args: &[Expr]) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new(
            "Date.prototype.valueOf() expects no arguments",
        ));
    }
    Ok(Value::Number(
        date.epoch_ms.map(|value| value as f64).unwrap_or(f64::NAN),
    ))
}

fn date_get_year(date: &DateValue, args: &[Expr], utc: bool, _legacy: bool) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("Date getter expects no arguments"));
    }
    Ok(Value::Number(match date.epoch_ms {
        Some(epoch_ms) => if utc {
            date_datetime_utc(epoch_ms).map(|dt| dt.year() as f64)
        } else {
            date_datetime_local(epoch_ms).map(|dt| dt.year() as f64)
        }
        .unwrap_or(f64::NAN),
        None => f64::NAN,
    }))
}

fn date_get_month(date: &DateValue, args: &[Expr], utc: bool) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("Date getter expects no arguments"));
    }
    Ok(Value::Number(match date.epoch_ms {
        Some(epoch_ms) => if utc {
            date_datetime_utc(epoch_ms).map(|dt| dt.month0() as f64)
        } else {
            date_datetime_local(epoch_ms).map(|dt| dt.month0() as f64)
        }
        .unwrap_or(f64::NAN),
        None => f64::NAN,
    }))
}

fn date_get_day(date: &DateValue, args: &[Expr], utc: bool) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("Date getter expects no arguments"));
    }
    Ok(Value::Number(match date.epoch_ms {
        Some(epoch_ms) => if utc {
            date_datetime_utc(epoch_ms).map(|dt| dt.day() as f64)
        } else {
            date_datetime_local(epoch_ms).map(|dt| dt.day() as f64)
        }
        .unwrap_or(f64::NAN),
        None => f64::NAN,
    }))
}

fn date_get_hours(date: &DateValue, args: &[Expr], utc: bool) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("Date getter expects no arguments"));
    }
    Ok(Value::Number(match date.epoch_ms {
        Some(epoch_ms) => if utc {
            date_datetime_utc(epoch_ms).map(|dt| dt.hour() as f64)
        } else {
            date_datetime_local(epoch_ms).map(|dt| dt.hour() as f64)
        }
        .unwrap_or(f64::NAN),
        None => f64::NAN,
    }))
}

fn date_get_minutes(date: &DateValue, args: &[Expr], utc: bool) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("Date getter expects no arguments"));
    }
    Ok(Value::Number(match date.epoch_ms {
        Some(epoch_ms) => if utc {
            date_datetime_utc(epoch_ms).map(|dt| dt.minute() as f64)
        } else {
            date_datetime_local(epoch_ms).map(|dt| dt.minute() as f64)
        }
        .unwrap_or(f64::NAN),
        None => f64::NAN,
    }))
}

fn date_get_seconds(date: &DateValue, args: &[Expr], utc: bool) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("Date getter expects no arguments"));
    }
    Ok(Value::Number(match date.epoch_ms {
        Some(epoch_ms) => if utc {
            date_datetime_utc(epoch_ms).map(|dt| dt.second() as f64)
        } else {
            date_datetime_local(epoch_ms).map(|dt| dt.second() as f64)
        }
        .unwrap_or(f64::NAN),
        None => f64::NAN,
    }))
}

fn date_get_milliseconds(date: &DateValue, args: &[Expr], utc: bool) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("Date getter expects no arguments"));
    }
    Ok(Value::Number(match date.epoch_ms {
        Some(epoch_ms) => if utc {
            date_datetime_utc(epoch_ms).map(|dt| dt.timestamp_subsec_millis() as f64)
        } else {
            date_datetime_local(epoch_ms).map(|dt| dt.timestamp_subsec_millis() as f64)
        }
        .unwrap_or(f64::NAN),
        None => f64::NAN,
    }))
}

fn date_get_timezone_offset(date: &DateValue, args: &[Expr]) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new(
            "Date.prototype.getTimezoneOffset() expects no arguments",
        ));
    }
    Ok(Value::Number(match date.epoch_ms {
        Some(epoch_ms) => {
            let offset = date_zoned_parts(epoch_ms, None)
                .map(|(_, _, _, _, _, _, offset)| offset)
                .unwrap_or(0);
            (-offset) as f64
        }
        None => f64::NAN,
    }))
}

#[allow(dead_code)]
fn date_to_locale_epoch_string(date: &DateValue, locale: &str, include_time: bool) -> String {
    match date.epoch_ms {
        Some(epoch_ms) => date_format_locale_string(epoch_ms, locale, include_time)
            .unwrap_or_else(|| "Invalid Date".to_string()),
        None => "Invalid Date".to_string(),
    }
}

fn intl_number_format_format<H: HostBindings>(
    formatter: &IntlNumberFormatValue,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let value = match args.first() {
        Some(expr) => number_from_value(&eval_expr(expr, env, host)?).unwrap_or(f64::NAN),
        None => f64::NAN,
    };
    Ok(Value::String(intl_number_format_render(formatter, value)))
}

fn intl_number_format_resolved_options<H: HostBindings>(
    formatter: &IntlNumberFormatValue,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new(
            "Intl.NumberFormat.prototype.resolvedOptions() expects no arguments",
        ));
    }
    object_value_from_pairs(
        vec![
            ("locale", Value::String(formatter.locale.clone())),
            ("style", Value::String(formatter.style.clone())),
            ("useGrouping", Value::Boolean(formatter.use_grouping)),
            (
                "minimumIntegerDigits",
                Value::Number(formatter.minimum_integer_digits as f64),
            ),
        ],
        env,
        host,
    )
}

fn intl_number_format_to_parts<H: HostBindings>(
    formatter: &IntlNumberFormatValue,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let value = match args.first() {
        Some(expr) => number_from_value(&eval_expr(expr, env, host)?).unwrap_or(f64::NAN),
        None => f64::NAN,
    };
    part_array_from_pairs(
        intl_number_format_parts_for_value(formatter, value),
        env,
        host,
    )
}

fn intl_date_time_format_format<H: HostBindings>(
    formatter: &IntlDateTimeFormatValue,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let epoch_ms = match args.first() {
        Some(expr) => date_epoch_ms_from_value(&eval_expr(expr, env, host)?),
        None => date_epoch_ms_from_value(&Value::Date(DateValue {
            epoch_ms: Some(Utc::now().timestamp_millis()),
        })),
    };
    Ok(Value::String(match epoch_ms {
        Some(epoch_ms) => intl_date_time_format_render(formatter, epoch_ms)
            .unwrap_or_else(|| "Invalid Date".to_string()),
        None => "Invalid Date".to_string(),
    }))
}

fn intl_date_time_format_resolved_options<H: HostBindings>(
    formatter: &IntlDateTimeFormatValue,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new(
            "Intl.DateTimeFormat.prototype.resolvedOptions() expects no arguments",
        ));
    }
    object_value_from_pairs(
        vec![
            ("locale", Value::String(formatter.locale.clone())),
            (
                "timeZone",
                Value::String(
                    formatter
                        .time_zone
                        .clone()
                        .unwrap_or_else(|| "UTC".to_string()),
                ),
            ),
        ],
        env,
        host,
    )
}

fn intl_date_time_format_to_parts<H: HostBindings>(
    formatter: &IntlDateTimeFormatValue,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let epoch_ms = match args.first() {
        Some(expr) => date_epoch_ms_from_value(&eval_expr(expr, env, host)?),
        None => date_epoch_ms_from_value(&Value::Date(DateValue {
            epoch_ms: Some(Utc::now().timestamp_millis()),
        })),
    };
    match epoch_ms.and_then(|epoch_ms| intl_date_time_format_parts_for_epoch(formatter, epoch_ms)) {
        Some(parts) => part_array_from_pairs(parts, env, host),
        None => Ok(Value::Array(ArrayHandle::new(Vec::new()))),
    }
}

fn intl_collator_compare<H: HostBindings>(
    collator: &IntlCollatorValue,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let left = match args.first() {
        Some(expr) => as_string(&eval_expr(expr, env, host)?),
        None => String::new(),
    };
    let right = match args.get(1) {
        Some(expr) => as_string(&eval_expr(expr, env, host)?),
        None => String::new(),
    };
    Ok(Value::Number(
        match intl_collator_compare_values(collator, &left, &right) {
            Ordering::Less => -1.0,
            Ordering::Equal => 0.0,
            Ordering::Greater => 1.0,
        },
    ))
}

fn intl_collator_resolved_options<H: HostBindings>(
    collator: &IntlCollatorValue,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new(
            "Intl.Collator.prototype.resolvedOptions() expects no arguments",
        ));
    }
    object_value_from_pairs(
        vec![
            ("locale", Value::String(collator.locale.clone())),
            ("numeric", Value::Boolean(collator.numeric)),
        ],
        env,
        host,
    )
}

fn array_join<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 1 {
        return Err(ScriptError::new("join() expects at most one argument"));
    }
    let separator = match args.first() {
        Some(expr) => as_string(&eval_expr(expr, env, host)?),
        None => ",".to_string(),
    };
    Ok(Value::String(array_to_string(array, &separator, None)))
}

fn array_map<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new("map() expects one or two arguments"));
        }
    };
    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => return Err(ScriptError::new("map() requires a function callback")),
    };
    let this_value = match this_arg_expr {
        Some(expr) => eval_expr(expr, env, host)?,
        None => Value::Undefined,
    };
    let items = array.0.borrow().items.clone();
    let mut out = Vec::with_capacity(items.len());
    for (index, item) in items.into_iter().enumerate() {
        let mapped = call_script_function_value(
            &callback,
            &[
                item.clone(),
                Value::Number(index as f64),
                Value::Array(array.clone()),
            ],
            this_value.clone(),
            env,
            host,
        )?;
        out.push(mapped);
    }
    Ok(Value::Array(ArrayHandle::new(out)))
}

fn array_filter<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new("filter() expects one or two arguments"));
        }
    };
    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => return Err(ScriptError::new("filter() requires a function callback")),
    };
    let this_value = match this_arg_expr {
        Some(expr) => eval_expr(expr, env, host)?,
        None => Value::Undefined,
    };
    let items = array.0.borrow().items.clone();
    let mut out = Vec::new();
    for (index, item) in items.into_iter().enumerate() {
        let keep = call_script_function_value(
            &callback,
            &[
                item.clone(),
                Value::Number(index as f64),
                Value::Array(array.clone()),
            ],
            this_value.clone(),
            env,
            host,
        )?;
        if is_truthy(&keep) {
            out.push(item);
        }
    }
    Ok(Value::Array(ArrayHandle::new(out)))
}

fn array_for_each<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new("forEach() expects one or two arguments"));
        }
    };
    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => return Err(ScriptError::new("forEach() requires a function callback")),
    };
    let this_value = match this_arg_expr {
        Some(expr) => eval_expr(expr, env, host)?,
        None => Value::Undefined,
    };
    let items = array.0.borrow().items.clone();
    for (index, item) in items.into_iter().enumerate() {
        let _ = call_script_function_value(
            &callback,
            &[
                item,
                Value::Number(index as f64),
                Value::Array(array.clone()),
            ],
            this_value.clone(),
            env,
            host,
        )?;
    }
    Ok(Value::Undefined)
}

fn array_some<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new("some() expects one or two arguments"));
        }
    };
    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => return Err(ScriptError::new("some() requires a function callback")),
    };
    let this_value = match this_arg_expr {
        Some(expr) => eval_expr(expr, env, host)?,
        None => Value::Undefined,
    };
    let items = array.0.borrow().items.clone();
    for (index, item) in items.into_iter().enumerate() {
        let keep = call_script_function_value(
            &callback,
            &[
                item.clone(),
                Value::Number(index as f64),
                Value::Array(array.clone()),
            ],
            this_value.clone(),
            env,
            host,
        )?;
        if is_truthy(&keep) {
            return Ok(Value::Boolean(true));
        }
    }
    Ok(Value::Boolean(false))
}

fn array_every<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new("every() expects one or two arguments"));
        }
    };
    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => return Err(ScriptError::new("every() requires a function callback")),
    };
    let this_value = match this_arg_expr {
        Some(expr) => eval_expr(expr, env, host)?,
        None => Value::Undefined,
    };
    let items = array.0.borrow().items.clone();
    for (index, item) in items.into_iter().enumerate() {
        let keep = call_script_function_value(
            &callback,
            &[
                item.clone(),
                Value::Number(index as f64),
                Value::Array(array.clone()),
            ],
            this_value.clone(),
            env,
            host,
        )?;
        if !is_truthy(&keep) {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

fn array_flat_map<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new("flatMap() expects one or two arguments"));
        }
    };
    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => return Err(ScriptError::new("flatMap() requires a function callback")),
    };
    let this_value = match this_arg_expr {
        Some(expr) => eval_expr(expr, env, host)?,
        None => Value::Undefined,
    };
    let items = array.0.borrow().items.clone();
    let mut out = Vec::new();
    for (index, item) in items.into_iter().enumerate() {
        let mapped = call_script_function_value(
            &callback,
            &[
                item.clone(),
                Value::Number(index as f64),
                Value::Array(array.clone()),
            ],
            this_value.clone(),
            env,
            host,
        )?;
        match mapped {
            Value::Array(result) => out.extend(result.0.borrow().items.clone()),
            other => out.push(other),
        }
    }
    Ok(Value::Array(ArrayHandle::new(out)))
}

fn array_flat<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 1 {
        return Err(ScriptError::new("flat() expects at most one argument"));
    }
    let depth = match args.first() {
        Some(expr) => integer_from_value(&eval_expr(expr, env, host)?, "flat()")?,
        None => 1,
    };
    if depth <= 0 {
        return Ok(Value::Array(ArrayHandle::new(
            array.0.borrow().items.clone(),
        )));
    }
    let mut out = Vec::new();
    for item in array.0.borrow().items.clone() {
        match item {
            Value::Array(result) => out.extend(result.0.borrow().items.clone()),
            other => out.push(other),
        }
    }
    Ok(Value::Array(ArrayHandle::new(out)))
}

fn array_fill<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(ScriptError::new("fill() expects one to three arguments"));
    }

    let value = eval_expr(&args[0], env, host)?;
    let len = array.0.borrow().items.len() as i64;
    let start = match args.get(1) {
        Some(expr) => integer_from_value(&eval_expr(expr, env, host)?, "fill()")?,
        None => 0,
    };
    let end = match args.get(2) {
        Some(expr) => integer_from_value(&eval_expr(expr, env, host)?, "fill()")?,
        None => len,
    };

    let start = if start < 0 {
        (len + start).max(0)
    } else {
        start.min(len)
    } as usize;
    let end = if end < 0 {
        (len + end).max(0)
    } else {
        end.min(len)
    } as usize;

    let mut state = array.0.borrow_mut();
    for index in start..end.min(state.items.len()) {
        state.items[index] = value.clone();
    }
    Ok(Value::Array(array.clone()))
}

fn array_push<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let mut state = array.0.borrow_mut();
    for expr in args {
        state.items.push(eval_expr(expr, env, host)?);
    }
    Ok(Value::Number(state.items.len() as f64))
}

fn array_pop<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    _host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("pop() expects no arguments"));
    }
    Ok(array.0.borrow_mut().items.pop().unwrap_or(Value::Undefined))
}

fn array_slice<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.len() > 2 {
        return Err(ScriptError::new("slice() expects at most two arguments"));
    }
    let items = array.0.borrow().items.clone();
    let len = items.len() as i64;
    let start = match args.first() {
        Some(expr) => integer_from_value(&eval_expr(expr, env, host)?, "slice()")?,
        None => 0,
    };
    let end = match args.get(1) {
        Some(expr) => integer_from_value(&eval_expr(expr, env, host)?, "slice()")?,
        None => len,
    };
    let start = if start < 0 {
        (len + start).max(0)
    } else {
        start.min(len)
    } as usize;
    let end = if end < 0 {
        (len + end).max(0)
    } else {
        end.min(len)
    } as usize;
    let slice = if end <= start {
        Vec::new()
    } else {
        items[start..end].to_vec()
    };
    Ok(Value::Array(ArrayHandle::new(slice)))
}

fn array_concat<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let mut out = array.0.borrow().items.clone();
    for expr in args {
        match eval_expr(expr, env, host)? {
            Value::Array(result) => out.extend(result.0.borrow().items.clone()),
            other => out.push(other),
        }
    }
    Ok(Value::Array(ArrayHandle::new(out)))
}

fn array_sort<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let mut items = array.0.borrow().items.clone();
    match args {
        [] => items.sort_by_key(|value| as_string(value)),
        [callback_expr] => {
            let callback = match eval_expr(callback_expr, env, host)? {
                Value::Function(function) => function,
                _ => return Err(ScriptError::new("sort() requires a function callback")),
            };
            items.sort_by(|left, right| {
                let result = call_script_function_value(
                    &callback,
                    &[left.clone(), right.clone()],
                    Value::Undefined,
                    env,
                    host,
                )
                .unwrap_or(Value::Number(0.0));
                let number = match result {
                    Value::Number(number) => number,
                    _ => 0.0,
                };
                number
                    .partial_cmp(&0.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        _ => return Err(ScriptError::new("sort() expects zero or one argument")),
    }
    array.0.borrow_mut().items = items;
    Ok(Value::Array(array.clone()))
}

fn array_includes<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(ScriptError::new("includes() expects one or two arguments"));
    }

    let search = eval_expr(&args[0], env, host)?;
    let from_index = match args.get(1) {
        Some(expr) => integer_from_value(&eval_expr(expr, env, host)?, "includes()")?,
        None => 0,
    };
    let items = array.0.borrow().items.clone();
    let len = items.len() as i64;
    let start = if from_index < 0 {
        (len + from_index).max(0)
    } else {
        from_index.min(len)
    } as usize;

    Ok(Value::Boolean(items[start..].iter().any(|item| {
        eval_equality(item, &search, true)
            || (matches!(item, Value::Number(number) if number.is_nan())
                && matches!(search, Value::Number(number) if number.is_nan()))
    })))
}

fn array_index_of<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    last: bool,
) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(ScriptError::new("indexOf() expects one or two arguments"));
    }

    let search = eval_expr(&args[0], env, host)?;
    let from_index = match args.get(1) {
        Some(expr) => integer_from_value(&eval_expr(expr, env, host)?, "indexOf()")?,
        None => {
            if last {
                i64::MAX
            } else {
                0
            }
        }
    };
    let items = array.0.borrow().items.clone();
    let len = items.len() as i64;

    let found = if last {
        let end = if from_index < 0 {
            (len + from_index).max(-1)
        } else {
            from_index.min(len.saturating_sub(1))
        };
        (0..=end.max(-1))
            .rev()
            .find(|index| {
                items
                    .get(*index as usize)
                    .map(|item| eval_equality(item, &search, true))
                    .unwrap_or(false)
            })
            .map(|index| index as i64)
    } else {
        let start = if from_index < 0 {
            (len + from_index).max(0)
        } else {
            from_index.min(len)
        } as usize;
        (start..items.len())
            .find(|index| {
                items
                    .get(*index)
                    .map(|item| eval_equality(item, &search, true))
                    .unwrap_or(false)
            })
            .map(|index| index as i64)
    };

    Ok(Value::Number(
        found.map(|index| index as f64).unwrap_or(-1.0),
    ))
}

fn array_reverse<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    _host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("reverse() expects no arguments"));
    }

    array.0.borrow_mut().items.reverse();
    Ok(Value::Array(array.clone()))
}

fn array_reduce<H: HostBindings>(
    array: &crate::ArrayHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, initial_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, initial_expr] => (callback_expr, Some(initial_expr)),
        _ => {
            return Err(ScriptError::new("reduce() expects one or two arguments"));
        }
    };
    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => return Err(ScriptError::new("reduce() requires a function callback")),
    };
    let items = array.0.borrow().items.clone();
    let mut index = 0usize;
    let mut accumulator = match initial_expr {
        Some(expr) => eval_expr(expr, env, host)?,
        None => {
            let Some(first) = items.first().cloned() else {
                return Err(ScriptError::new(
                    "reduce() of empty array with no initial value",
                ));
            };
            index = 1;
            first
        }
    };

    for (offset, item) in items.into_iter().enumerate().skip(index) {
        accumulator = call_script_function_value(
            &callback,
            &[
                accumulator.clone(),
                item,
                Value::Number(offset as f64),
                Value::Array(array.clone()),
            ],
            Value::Undefined,
            env,
            host,
        )?;
    }

    Ok(accumulator)
}

fn map_get<H: HostBindings>(
    map: &MapHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [key_expr, ..] = args else {
        return Err(ScriptError::new("get() expects at least one argument"));
    };
    let key = map_key_from_value(&eval_expr(key_expr, env, host)?);
    Ok(map
        .0
        .borrow()
        .entries
        .iter()
        .find(|(candidate, _)| *candidate == key)
        .map(|(_, value)| value.clone())
        .unwrap_or(Value::Undefined))
}

fn map_set<H: HostBindings>(
    map: &MapHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [key_expr, value_expr, ..] = args else {
        return Err(ScriptError::new("set() expects at least two arguments"));
    };
    let key = map_key_from_value(&eval_expr(key_expr, env, host)?);
    let value = eval_expr(value_expr, env, host)?;
    let mut state = map.0.borrow_mut();
    if let Some((_, existing)) = state
        .entries
        .iter_mut()
        .find(|(candidate, _)| *candidate == key)
    {
        *existing = value;
    } else {
        state.entries.push((key, value));
    }
    Ok(Value::Map(map.clone()))
}

fn map_has<H: HostBindings>(
    map: &MapHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [key_expr, ..] = args else {
        return Err(ScriptError::new("has() expects at least one argument"));
    };
    let key = map_key_from_value(&eval_expr(key_expr, env, host)?);
    Ok(Value::Boolean(
        map.0
            .borrow()
            .entries
            .iter()
            .any(|(candidate, _)| *candidate == key),
    ))
}

fn map_delete<H: HostBindings>(
    map: &MapHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [key_expr, ..] = args else {
        return Err(ScriptError::new("delete() expects at least one argument"));
    };
    let key = map_key_from_value(&eval_expr(key_expr, env, host)?);
    let mut state = map.0.borrow_mut();
    let len_before = state.entries.len();
    state.entries.retain(|(candidate, _)| *candidate != key);
    Ok(Value::Boolean(state.entries.len() != len_before))
}

fn map_clear<H: HostBindings>(
    map: &MapHandle,
    args: &[Expr],
    _env: &mut BTreeMap<String, Value>,
    _host: &mut H,
) -> Result<Value> {
    if !args.is_empty() {
        return Err(ScriptError::new("clear() expects no arguments"));
    }
    map.0.borrow_mut().entries.clear();
    Ok(Value::Undefined)
}

fn map_keys<H: HostBindings>(map: &MapHandle, _host: &mut H) -> Result<Value> {
    Ok(collection_iterator(
        map.0
            .borrow()
            .entries
            .iter()
            .map(|(key, _)| map_key_to_value(key))
            .collect(),
    ))
}

fn map_values<H: HostBindings>(map: &MapHandle, _host: &mut H) -> Result<Value> {
    Ok(collection_iterator(
        map.0
            .borrow()
            .entries
            .iter()
            .map(|(_, value)| value.clone())
            .collect(),
    ))
}

fn map_entries<H: HostBindings>(map: &MapHandle, _host: &mut H) -> Result<Value> {
    Ok(collection_iterator(
        map.0
            .borrow()
            .entries
            .iter()
            .map(|(key, value)| {
                Value::Array(ArrayHandle::new(vec![map_key_to_value(key), value.clone()]))
            })
            .collect(),
    ))
}

fn map_for_each<H: HostBindings>(
    map: &MapHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new("forEach() expects one or two arguments"));
        }
    };
    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => return Err(ScriptError::new("forEach() requires a function callback")),
    };
    let this_value = match this_arg_expr {
        Some(expr) => eval_expr(expr, env, host)?,
        None => Value::Undefined,
    };
    let entries = map.0.borrow().entries.clone();
    for (key, value) in entries {
        let _ = call_script_function_value(
            &callback,
            &[
                value.clone(),
                map_key_to_value(&key),
                Value::Map(map.clone()),
            ],
            this_value.clone(),
            env,
            host,
        )?;
    }
    Ok(Value::Undefined)
}

fn date_constructor<H: HostBindings>(
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let value = match args.len() {
        0 => Value::Date(DateValue {
            epoch_ms: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| ScriptError::new("system clock is before the Unix epoch"))?
                    .as_millis() as i64,
            ),
        }),
        1 => {
            let arg = eval_expr(&args[0], env, host)?;
            Value::Date(DateValue {
                epoch_ms: date_epoch_ms_from_value(&arg),
            })
        }
        _ => Value::Date(DateValue {
            epoch_ms: date_epoch_from_component_args(args, env, host, true)?,
        }),
    };
    Ok(value)
}

fn intl_construct<H: HostBindings>(
    property: &str,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    match property {
        "NumberFormat" => Ok(Value::IntlNumberFormat(parse_intl_number_format(
            args, env, host,
        )?)),
        "DateTimeFormat" => Ok(Value::IntlDateTimeFormat(parse_intl_date_time_format(
            args, env, host,
        )?)),
        "Collator" => Ok(Value::IntlCollator(parse_intl_collator(args, env, host)?)),
        other => Err(ScriptError::new(format!(
            "unsupported Intl constructor: {other}"
        ))),
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

fn media_query_list_listener<H: HostBindings>(
    list: &MediaQueryListState,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
    method: &str,
) -> Result<Value> {
    let [callback_expr] = args else {
        return Err(ScriptError::new(format!(
            "MediaQueryList.{method}() expects exactly one argument",
        )));
    };

    match eval_expr(callback_expr, env, host)? {
        Value::Function(_) => {
            match method {
                "addListener" => host.match_media_add_listener(list.media())?,
                "removeListener" => host.match_media_remove_listener(list.media())?,
                other => {
                    return Err(ScriptError::new(format!(
                        "cannot call `{other}` on a media query list value"
                    )));
                }
            }
            Ok(Value::Undefined)
        }
        _ => Err(ScriptError::new(format!(
            "MediaQueryList.{method}() requires an arrow function callback",
        ))),
    }
}

fn media_query_list_add_listener<H: HostBindings>(
    list: &MediaQueryListState,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    media_query_list_listener(list, args, env, host, "addListener")
}

fn media_query_list_remove_listener<H: HostBindings>(
    list: &MediaQueryListState,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    media_query_list_listener(list, args, env, host, "removeListener")
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

fn class_list_item<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [index_expr] = args else {
        return Err(ScriptError::new(
            "classList.item() expects exactly one argument",
        ));
    };

    let index_value = eval_expr(index_expr, env, host)?;
    let Some(index) = index_from_value(&index_value) else {
        return Ok(Value::Null);
    };

    let tokens = class_list_tokens(element, host)?;
    Ok(tokens
        .get(index)
        .cloned()
        .map(Value::String)
        .unwrap_or(Value::Null))
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

fn class_list_replace<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let [old_token_expr, new_token_expr] = args else {
        return Err(ScriptError::new(
            "classList.replace() expects exactly two arguments",
        ));
    };

    let old_token = validate_class_list_token(&as_string(&eval_expr(old_token_expr, env, host)?))?;
    let new_token = validate_class_list_token(&as_string(&eval_expr(new_token_expr, env, host)?))?;

    let mut tokens = class_list_tokens(element, host)?;
    let Some(old_index) = tokens.iter().position(|candidate| candidate == &old_token) else {
        return Ok(Value::Boolean(false));
    };

    if old_token == new_token {
        return Ok(Value::Boolean(true));
    }

    if tokens.iter().any(|candidate| candidate == &new_token) {
        tokens.retain(|candidate| candidate != &old_token);
    } else {
        tokens[old_index] = new_token;
    }
    write_class_list_tokens(element, &tokens, host)?;
    Ok(Value::Boolean(true))
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

fn class_list_for_each<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    env: &mut BTreeMap<String, Value>,
    host: &mut H,
) -> Result<Value> {
    let (callback_expr, this_arg_expr) = match args {
        [callback_expr] => (callback_expr, None),
        [callback_expr, this_arg_expr] => (callback_expr, Some(this_arg_expr)),
        _ => {
            return Err(ScriptError::new(
                "classList.forEach() expects one or two arguments",
            ));
        }
    };

    let callback = match eval_expr(callback_expr, env, host)? {
        Value::Function(function) => function,
        _ => {
            return Err(ScriptError::new(
                "classList.forEach() requires an arrow function callback",
            ));
        }
    };
    if let Some(this_arg_expr) = this_arg_expr {
        let _ = eval_expr(this_arg_expr, env, host)?;
    }

    let items = class_list_tokens(element, host)?
        .into_iter()
        .map(Value::String)
        .collect();
    let collection_value = Value::ClassList(element);
    for_each_over_items(&callback, items, collection_value, env, host)
}

fn class_list_keys<H: HostBindings>(element: crate::ElementHandle, host: &mut H) -> Result<Value> {
    let tokens = class_list_tokens(element, host)?;
    Ok(collection_iterator(
        (0..tokens.len())
            .map(|index| Value::Number(index as f64))
            .collect(),
    ))
}

fn class_list_values<H: HostBindings>(
    element: crate::ElementHandle,
    host: &mut H,
) -> Result<Value> {
    let tokens = class_list_tokens(element, host)?;
    Ok(collection_iterator(
        tokens.into_iter().map(Value::String).collect(),
    ))
}

fn class_list_entries<H: HostBindings>(
    element: crate::ElementHandle,
    host: &mut H,
) -> Result<Value> {
    let tokens = class_list_tokens(element, host)?;
    Ok(collection_entries(
        tokens.into_iter().map(Value::String).collect(),
    ))
}

fn class_list_to_string<H: HostBindings>(
    element: crate::ElementHandle,
    args: &[Expr],
    host: &mut H,
) -> Result<Value> {
    let [] = args else {
        return Err(ScriptError::new(
            "classList.toString() expects no arguments",
        ));
    };

    let tokens = class_list_tokens(element, host)?;
    Ok(Value::String(tokens.join(" ")))
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
