use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    HtmlParse(String),
    ScriptParse(String),
    ScriptRuntime(String),
    SelectorNotFound(String),
    UnsupportedSelector(String),
    TypeMismatch {
        selector: String,
        expected: String,
        actual: String,
    },
    AssertionFailed {
        selector: String,
        expected: String,
        actual: String,
        dom_snippet: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HtmlParse(msg) => write!(f, "html parse error: {msg}"),
            Self::ScriptParse(msg) => write!(f, "script parse error: {msg}"),
            Self::ScriptRuntime(msg) => write!(f, "script runtime error: {msg}"),
            Self::SelectorNotFound(selector) => write!(f, "selector not found: {selector}"),
            Self::UnsupportedSelector(selector) => write!(f, "unsupported selector: {selector}"),
            Self::TypeMismatch {
                selector,
                expected,
                actual,
            } => write!(
                f,
                "type mismatch for {selector}: expected {expected}, actual {actual}"
            ),
            Self::AssertionFailed {
                selector,
                expected,
                actual,
                dom_snippet,
            } => write!(
                f,
                "assertion failed for {selector}: expected {expected}, actual {actual}, snippet {dom_snippet}"
            ),
        }
    }
}

impl StdError for Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeId(usize);

#[derive(Debug, Clone)]
enum NodeType {
    Document,
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone)]
struct Node {
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    node_type: NodeType,
}

#[derive(Debug, Clone)]
struct Element {
    tag_name: String,
    attrs: HashMap<String, String>,
    value: String,
    checked: bool,
    disabled: bool,
}

#[derive(Debug, Clone)]
struct Dom {
    nodes: Vec<Node>,
    root: NodeId,
    id_index: HashMap<String, NodeId>,
}

impl Dom {
    fn new() -> Self {
        let root = Node {
            parent: None,
            children: Vec::new(),
            node_type: NodeType::Document,
        };
        Self {
            nodes: vec![root],
            root: NodeId(0),
            id_index: HashMap::new(),
        }
    }

    fn create_node(&mut self, parent: Option<NodeId>, node_type: NodeType) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            parent,
            children: Vec::new(),
            node_type,
        });
        if let Some(parent_id) = parent {
            self.nodes[parent_id.0].children.push(id);
        }
        id
    }

    fn create_element(
        &mut self,
        parent: NodeId,
        tag_name: String,
        attrs: HashMap<String, String>,
    ) -> NodeId {
        let value = attrs.get("value").cloned().unwrap_or_default();
        let checked = attrs.contains_key("checked");
        let disabled = attrs.contains_key("disabled");
        let element = Element {
            tag_name,
            attrs,
            value,
            checked,
            disabled,
        };
        let id = self.create_node(Some(parent), NodeType::Element(element));
        if let Some(id_attr) = self
            .element(id)
            .and_then(|element| element.attrs.get("id").cloned())
        {
            self.id_index.insert(id_attr, id);
        }
        id
    }

    fn create_text(&mut self, parent: NodeId, text: String) -> NodeId {
        self.create_node(Some(parent), NodeType::Text(text))
    }

    fn element(&self, node_id: NodeId) -> Option<&Element> {
        match &self.nodes[node_id.0].node_type {
            NodeType::Element(element) => Some(element),
            _ => None,
        }
    }

    fn element_mut(&mut self, node_id: NodeId) -> Option<&mut Element> {
        match &mut self.nodes[node_id.0].node_type {
            NodeType::Element(element) => Some(element),
            _ => None,
        }
    }

    fn tag_name(&self, node_id: NodeId) -> Option<&str> {
        self.element(node_id).map(|e| e.tag_name.as_str())
    }

    fn parent(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes[node_id.0].parent
    }

    fn by_id(&self, id: &str) -> Option<NodeId> {
        self.id_index.get(id).copied()
    }

    fn text_content(&self, node_id: NodeId) -> String {
        match &self.nodes[node_id.0].node_type {
            NodeType::Document | NodeType::Element(_) => {
                let mut out = String::new();
                for child in &self.nodes[node_id.0].children {
                    out.push_str(&self.text_content(*child));
                }
                out
            }
            NodeType::Text(text) => text.clone(),
        }
    }

    fn set_text_content(&mut self, node_id: NodeId, value: &str) -> Result<()> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "textContent target is not an element".into(),
            ));
        }
        self.nodes[node_id.0].children.clear();
        if !value.is_empty() {
            self.create_text(node_id, value.to_string());
        }
        Ok(())
    }

    fn value(&self, node_id: NodeId) -> Result<String> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("value target is not an element".into()))?;
        Ok(element.value.clone())
    }

    fn set_value(&mut self, node_id: NodeId, value: &str) -> Result<()> {
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("value target is not an element".into()))?;
        element.value = value.to_string();
        Ok(())
    }

    fn checked(&self, node_id: NodeId) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("checked target is not an element".into()))?;
        Ok(element.checked)
    }

    fn set_checked(&mut self, node_id: NodeId, checked: bool) -> Result<()> {
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("checked target is not an element".into()))?;
        element.checked = checked;
        Ok(())
    }

    fn disabled(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.disabled).unwrap_or(false)
    }

    fn attr(&self, node_id: NodeId, name: &str) -> Option<String> {
        self.element(node_id)
            .and_then(|e| e.attrs.get(name).cloned())
    }

    fn has_attr(&self, node_id: NodeId, name: &str) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("hasAttribute target is not an element".into()))?;
        Ok(element.attrs.contains_key(&name.to_ascii_lowercase()))
    }

    fn set_attr(&mut self, node_id: NodeId, name: &str, value: &str) -> Result<()> {
        let old_id = if name.eq_ignore_ascii_case("id") {
            self.element(node_id)
                .and_then(|element| element.attrs.get("id").cloned())
        } else {
            None
        };

        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("setAttribute target is not an element".into()))?;

        let lowered = name.to_ascii_lowercase();
        element.attrs.insert(lowered.clone(), value.to_string());

        if lowered == "value" {
            element.value = value.to_string();
        } else if lowered == "checked" {
            element.checked = true;
        } else if lowered == "disabled" {
            element.disabled = true;
        }

        if lowered == "id" {
            if let Some(old) = old_id {
                self.id_index.remove(&old);
            }
            self.id_index.insert(value.to_string(), node_id);
        }

        Ok(())
    }

    fn remove_attr(&mut self, node_id: NodeId, name: &str) -> Result<()> {
        let lowered = name.to_ascii_lowercase();
        let old_id = if lowered == "id" {
            self.element(node_id)
                .and_then(|element| element.attrs.get("id").cloned())
        } else {
            None
        };

        let element = self.element_mut(node_id).ok_or_else(|| {
            Error::ScriptRuntime("removeAttribute target is not an element".into())
        })?;
        element.attrs.remove(&lowered);

        if lowered == "value" {
            element.value.clear();
        } else if lowered == "checked" {
            element.checked = false;
        } else if lowered == "disabled" {
            element.disabled = false;
        }

        if lowered == "id" {
            if let Some(old) = old_id {
                self.id_index.remove(&old);
            }
        }

        Ok(())
    }

    fn dataset_get(&self, node_id: NodeId, key: &str) -> Result<String> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "dataset target is not an element".into(),
            ));
        }
        let name = dataset_key_to_attr_name(key);
        Ok(self.attr(node_id, &name).unwrap_or_default())
    }

    fn dataset_set(&mut self, node_id: NodeId, key: &str, value: &str) -> Result<()> {
        let name = dataset_key_to_attr_name(key);
        self.set_attr(node_id, &name, value)
    }

    fn style_get(&self, node_id: NodeId, key: &str) -> Result<String> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("style target is not an element".into()))?;
        let name = js_prop_to_css_name(key);
        let decls = parse_style_declarations(element.attrs.get("style").map(String::as_str));
        Ok(decls
            .iter()
            .find(|(prop, _)| prop == &name)
            .map(|(_, value)| value.clone())
            .unwrap_or_default())
    }

    fn style_set(&mut self, node_id: NodeId, key: &str, value: &str) -> Result<()> {
        let name = js_prop_to_css_name(key);
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("style target is not an element".into()))?;

        let mut decls = parse_style_declarations(element.attrs.get("style").map(String::as_str));
        if let Some(pos) = decls.iter().position(|(prop, _)| prop == &name) {
            if value.is_empty() {
                decls.remove(pos);
            } else {
                decls[pos].1 = value.to_string();
            }
        } else if !value.is_empty() {
            decls.push((name, value.to_string()));
        }

        if decls.is_empty() {
            element.attrs.remove("style");
        } else {
            element
                .attrs
                .insert("style".to_string(), serialize_style_declarations(&decls));
        }

        Ok(())
    }

    fn class_contains(&self, node_id: NodeId, class_name: &str) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("classList target is not an element".into()))?;
        Ok(has_class(element, class_name))
    }

    fn class_add(&mut self, node_id: NodeId, class_name: &str) -> Result<()> {
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("classList target is not an element".into()))?;
        let mut classes = class_tokens(element.attrs.get("class").map(String::as_str));
        if !classes.iter().any(|name| name == class_name) {
            classes.push(class_name.to_string());
        }
        set_class_attr(element, &classes);
        Ok(())
    }

    fn class_remove(&mut self, node_id: NodeId, class_name: &str) -> Result<()> {
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("classList target is not an element".into()))?;
        let mut classes = class_tokens(element.attrs.get("class").map(String::as_str));
        classes.retain(|name| name != class_name);
        set_class_attr(element, &classes);
        Ok(())
    }

    fn class_toggle(&mut self, node_id: NodeId, class_name: &str) -> Result<bool> {
        let has = self.class_contains(node_id, class_name)?;
        if has {
            self.class_remove(node_id, class_name)?;
            Ok(false)
        } else {
            self.class_add(node_id, class_name)?;
            Ok(true)
        }
    }

    fn query_selector(&self, selector: &str) -> Result<Option<NodeId>> {
        let all = self.query_selector_all(selector)?;
        Ok(all.into_iter().next())
    }

    fn query_selector_all(&self, selector: &str) -> Result<Vec<NodeId>> {
        let steps = parse_selector_chain(selector)?;

        if steps.len() == 1 {
            if let SelectorStep::Id(id) = &steps[0].step {
                return Ok(self.by_id(id).into_iter().collect());
            }
        }

        let mut ids = Vec::new();
        self.collect_elements_dfs(self.root, &mut ids);

        let mut matched = Vec::new();
        for candidate in ids {
            if self.matches_selector_chain(candidate, &steps) {
                matched.push(candidate);
            }
        }
        Ok(matched)
    }

    fn collect_elements_dfs(&self, node_id: NodeId, out: &mut Vec<NodeId>) {
        if matches!(self.nodes[node_id.0].node_type, NodeType::Element(_)) {
            out.push(node_id);
        }
        for child in &self.nodes[node_id.0].children {
            self.collect_elements_dfs(*child, out);
        }
    }

    fn all_element_nodes(&self) -> Vec<NodeId> {
        let mut out = Vec::new();
        self.collect_elements_dfs(self.root, &mut out);
        out
    }

    fn matches_selector_chain(&self, node_id: NodeId, steps: &[SelectorPart]) -> bool {
        if steps.is_empty() {
            return false;
        }
        if !self.matches_step(node_id, &steps[steps.len() - 1].step) {
            return false;
        }

        let mut current = node_id;
        for idx in (1..steps.len()).rev() {
            let prev_step = &steps[idx - 1].step;
            let combinator = steps[idx]
                .combinator
                .unwrap_or(SelectorCombinator::Descendant);

            let matched = match combinator {
                SelectorCombinator::Child => {
                    let Some(parent) = self.parent(current) else {
                        return false;
                    };
                    if self.matches_step(parent, prev_step) {
                        Some(parent)
                    } else {
                        None
                    }
                }
                SelectorCombinator::Descendant => {
                    let mut cursor = self.parent(current);
                    let mut found = None;
                    while let Some(parent) = cursor {
                        if self.matches_step(parent, prev_step) {
                            found = Some(parent);
                            break;
                        }
                        cursor = self.parent(parent);
                    }
                    found
                }
            };

            let Some(matched) = matched else {
                return false;
            };
            current = matched;
        }

        true
    }

    fn matches_step(&self, node_id: NodeId, step: &SelectorStep) -> bool {
        let Some(element) = self.element(node_id) else {
            return false;
        };

        match step {
            SelectorStep::Id(id) => element.attrs.get("id") == Some(id),
            SelectorStep::Class(class_name) => has_class(element, class_name),
            SelectorStep::Tag(tag) => element.tag_name.eq_ignore_ascii_case(tag),
            SelectorStep::TagClass { tag, class_name } => {
                element.tag_name.eq_ignore_ascii_case(tag) && has_class(element, class_name)
            }
            SelectorStep::AttrExists { key } => element.attrs.contains_key(key),
            SelectorStep::AttrEq { key, value } => element.attrs.get(key) == Some(value),
        }
    }

    fn find_ancestor_by_tag(&self, node_id: NodeId, tag: &str) -> Option<NodeId> {
        let mut cursor = self.parent(node_id);
        while let Some(current) = cursor {
            if self
                .tag_name(current)
                .map(|name| name.eq_ignore_ascii_case(tag))
                .unwrap_or(false)
            {
                return Some(current);
            }
            cursor = self.parent(current);
        }
        None
    }

    fn dump_node(&self, node_id: NodeId) -> String {
        match &self.nodes[node_id.0].node_type {
            NodeType::Document => {
                let mut out = String::new();
                for child in &self.nodes[node_id.0].children {
                    out.push_str(&self.dump_node(*child));
                }
                out
            }
            NodeType::Text(text) => text.clone(),
            NodeType::Element(element) => {
                let mut out = String::new();
                out.push('<');
                out.push_str(&element.tag_name);
                for (k, v) in &element.attrs {
                    out.push(' ');
                    out.push_str(k);
                    out.push_str("=\"");
                    out.push_str(v);
                    out.push('"');
                }
                out.push('>');
                for child in &self.nodes[node_id.0].children {
                    out.push_str(&self.dump_node(*child));
                }
                out.push_str("</");
                out.push_str(&element.tag_name);
                out.push('>');
                out
            }
        }
    }
}

fn has_class(element: &Element, class_name: &str) -> bool {
    element
        .attrs
        .get("class")
        .map(|classes| classes.split_whitespace().any(|c| c == class_name))
        .unwrap_or(false)
}

fn class_tokens(class_attr: Option<&str>) -> Vec<String> {
    class_attr
        .map(|value| {
            value
                .split_whitespace()
                .filter(|token| !token.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn set_class_attr(element: &mut Element, classes: &[String]) {
    if classes.is_empty() {
        element.attrs.remove("class");
    } else {
        element.attrs.insert("class".to_string(), classes.join(" "));
    }
}

fn dataset_key_to_attr_name(key: &str) -> String {
    format!("data-{}", js_prop_to_css_name(key))
}

fn js_prop_to_css_name(prop: &str) -> String {
    let mut out = String::new();
    for ch in prop.chars() {
        if ch.is_ascii_uppercase() {
            out.push('-');
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn parse_style_declarations(style_attr: Option<&str>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Some(style_attr) = style_attr else {
        return out;
    };

    for decl in style_attr.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        let Some((name, value)) = decl.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }
        let value = value.trim().to_string();
        if let Some(pos) = out.iter().position(|(existing, _)| existing == &name) {
            out[pos].1 = value;
        } else {
            out.push((name, value));
        }
    }

    out
}

fn serialize_style_declarations(decls: &[(String, String)]) -> String {
    let mut out = String::new();
    for (idx, (name, value)) in decls.iter().enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        out.push_str(name);
        out.push_str(": ");
        out.push_str(value);
        out.push(';');
    }
    out
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut it = value.chars();
    let mut out = String::new();
    for _ in 0..max_chars {
        let Some(ch) = it.next() else {
            return out;
        };
        out.push(ch);
    }
    if it.next().is_some() {
        out.push_str("...");
    }
    out
}

#[derive(Debug, Clone)]
enum SelectorStep {
    Id(String),
    Class(String),
    Tag(String),
    TagClass { tag: String, class_name: String },
    AttrExists { key: String },
    AttrEq { key: String, value: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectorCombinator {
    Descendant,
    Child,
}

#[derive(Debug, Clone)]
struct SelectorPart {
    step: SelectorStep,
    // Relation to previous (left) selector part.
    combinator: Option<SelectorCombinator>,
}

fn parse_selector_chain(selector: &str) -> Result<Vec<SelectorPart>> {
    let selector = selector.trim();
    if selector.is_empty() {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    let tokens = tokenize_selector(selector)?;
    let mut steps = Vec::new();
    let mut pending_combinator: Option<SelectorCombinator> = None;

    for token in tokens {
        if token == ">" {
            if pending_combinator.is_some() || steps.is_empty() {
                return Err(Error::UnsupportedSelector(selector.into()));
            }
            pending_combinator = Some(SelectorCombinator::Child);
            continue;
        }

        let step = parse_selector_step(&token)?;
        let combinator = if steps.is_empty() {
            None
        } else {
            Some(
                pending_combinator
                    .take()
                    .unwrap_or(SelectorCombinator::Descendant),
            )
        };
        steps.push(SelectorPart { step, combinator });
    }

    if steps.is_empty() || pending_combinator.is_some() {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    Ok(steps)
}

fn tokenize_selector(selector: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut bracket_depth = 0usize;

    for ch in selector.chars() {
        match ch {
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                if bracket_depth == 0 {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                bracket_depth -= 1;
                current.push(ch);
            }
            '>' if bracket_depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                }
                current.clear();
                tokens.push(">".to_string());
            }
            ch if ch.is_ascii_whitespace() && bracket_depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if bracket_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    Ok(tokens)
}

fn parse_selector_step(part: &str) -> Result<SelectorStep> {
    if let Some(stripped) = part.strip_prefix('#') {
        if stripped.is_empty() {
            return Err(Error::UnsupportedSelector(part.into()));
        }
        return Ok(SelectorStep::Id(stripped.to_string()));
    }

    if let Some(stripped) = part.strip_prefix('.') {
        if stripped.is_empty() {
            return Err(Error::UnsupportedSelector(part.into()));
        }
        return Ok(SelectorStep::Class(stripped.to_string()));
    }

    if part.starts_with('[') && part.ends_with(']') {
        let body = &part[1..part.len() - 1];
        if let Some((key, value)) = body.split_once('=') {
            let key = key.trim().to_string();
            let value = value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            if key.is_empty() {
                return Err(Error::UnsupportedSelector(part.into()));
            }
            return Ok(SelectorStep::AttrEq { key, value });
        }
        let key = body.trim().to_string();
        if key.is_empty() {
            return Err(Error::UnsupportedSelector(part.into()));
        }
        return Ok(SelectorStep::AttrExists { key });
    }

    if let Some((tag, class_name)) = part.split_once('.') {
        if tag.is_empty() || class_name.is_empty() {
            return Err(Error::UnsupportedSelector(part.into()));
        }
        return Ok(SelectorStep::TagClass {
            tag: tag.to_string(),
            class_name: class_name.to_string(),
        });
    }

    if is_ident(part) {
        return Ok(SelectorStep::Tag(part.to_string()));
    }

    Err(Error::UnsupportedSelector(part.into()))
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    String(String),
    Bool(bool),
    Number(i64),
    Node(NodeId),
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::String(v) => !v.is_empty(),
            Self::Number(v) => *v != 0,
            Self::Node(_) => true,
        }
    }

    fn as_string(&self) -> String {
        match self {
            Self::String(v) => v.clone(),
            Self::Bool(v) => {
                if *v {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            Self::Number(v) => v.to_string(),
            Self::Node(node) => format!("node-{}", node.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomProp {
    Value,
    Checked,
    TextContent,
    ClassName,
    Id,
    Name,
    Dataset(String),
    Style(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomQuery {
    ById(String),
    BySelector(String),
    BySelectorAllIndex { selector: String, index: usize },
    Var(String),
}

impl DomQuery {
    fn describe_call(&self) -> String {
        match self {
            Self::ById(id) => format!("document.getElementById('{id}')"),
            Self::BySelector(selector) => format!("document.querySelector('{selector}')"),
            Self::BySelectorAllIndex { selector, index } => {
                format!("document.querySelectorAll('{selector}')[{index}]")
            }
            Self::Var(name) => name.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassListMethod {
    Add,
    Remove,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinaryOp {
    Or,
    And,
    StrictEq,
    StrictNe,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventExprProp {
    Type,
    Target,
    CurrentTarget,
    TargetId,
    CurrentTargetId,
}

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    String(String),
    Bool(bool),
    Number(i64),
    Var(String),
    DomRef(DomQuery),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    DomRead {
        target: DomQuery,
        prop: DomProp,
    },
    ClassListContains {
        target: DomQuery,
        class_name: String,
    },
    QuerySelectorAllLength {
        selector: String,
    },
    DomGetAttribute {
        target: DomQuery,
        name: String,
    },
    DomHasAttribute {
        target: DomQuery,
        name: String,
    },
    EventProp {
        event_var: String,
        prop: EventExprProp,
    },
    Not(Box<Expr>),
    Add(Vec<Expr>),
    Ternary {
        cond: Box<Expr>,
        on_true: Box<Expr>,
        on_false: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventMethod {
    PreventDefault,
    StopPropagation,
    StopImmediatePropagation,
}

#[derive(Debug, Clone, PartialEq)]
enum Stmt {
    VarDecl {
        name: String,
        expr: Expr,
    },
    DomAssign {
        target: DomQuery,
        prop: DomProp,
        expr: Expr,
    },
    ClassListCall {
        target: DomQuery,
        method: ClassListMethod,
        class_name: String,
        force: Option<Expr>,
    },
    DomSetAttribute {
        target: DomQuery,
        name: String,
        value: Expr,
    },
    DomRemoveAttribute {
        target: DomQuery,
        name: String,
    },
    ForEach {
        selector: String,
        item_var: String,
        index_var: Option<String>,
        body: Vec<Stmt>,
    },
    If {
        cond: Expr,
        then_stmts: Vec<Stmt>,
        else_stmts: Vec<Stmt>,
    },
    EventCall {
        event_var: String,
        method: EventMethod,
    },
    ListenerMutation {
        target: DomQuery,
        op: ListenerRegistrationOp,
        event_type: String,
        capture: bool,
        handler: ScriptHandler,
    },
    DispatchEvent {
        target: DomQuery,
        event_type: Expr,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
struct ScriptHandler {
    event_param: Option<String>,
    stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
struct Listener {
    capture: bool,
    handler: ScriptHandler,
}

#[derive(Debug, Default, Clone)]
struct ListenerStore {
    map: HashMap<NodeId, HashMap<String, Vec<Listener>>>,
}

impl ListenerStore {
    fn add(&mut self, node_id: NodeId, event: String, listener: Listener) {
        self.map
            .entry(node_id)
            .or_default()
            .entry(event)
            .or_default()
            .push(listener);
    }

    fn remove(
        &mut self,
        node_id: NodeId,
        event: &str,
        capture: bool,
        handler: &ScriptHandler,
    ) -> bool {
        let Some(events) = self.map.get_mut(&node_id) else {
            return false;
        };
        let Some(listeners) = events.get_mut(event) else {
            return false;
        };

        if let Some(pos) = listeners
            .iter()
            .position(|listener| listener.capture == capture && listener.handler == *handler)
        {
            listeners.remove(pos);
            if listeners.is_empty() {
                events.remove(event);
            }
            if events.is_empty() {
                self.map.remove(&node_id);
            }
            return true;
        }

        false
    }

    fn get(&self, node_id: NodeId, event: &str, capture: bool) -> Vec<Listener> {
        self.map
            .get(&node_id)
            .and_then(|events| events.get(event))
            .map(|listeners| {
                listeners
                    .iter()
                    .filter(|listener| listener.capture == capture)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
struct EventState {
    event_type: String,
    target: NodeId,
    current_target: NodeId,
    default_prevented: bool,
    propagation_stopped: bool,
    immediate_propagation_stopped: bool,
}

impl EventState {
    fn new(event_type: &str, target: NodeId) -> Self {
        Self {
            event_type: event_type.to_string(),
            target,
            current_target: target,
            default_prevented: false,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
        }
    }
}

#[derive(Debug, Clone)]
struct ParseOutput {
    dom: Dom,
    scripts: Vec<String>,
}

pub struct Harness {
    dom: Dom,
    listeners: ListenerStore,
    trace: bool,
}

impl Harness {
    pub fn from_html(html: &str) -> Result<Self> {
        let ParseOutput { dom, scripts } = parse_html(html)?;
        let mut harness = Self {
            dom,
            listeners: ListenerStore::default(),
            trace: false,
        };

        for script in scripts {
            harness.compile_and_register_script(&script)?;
        }

        Ok(harness)
    }

    pub fn enable_trace(&mut self, enabled: bool) {
        self.trace = enabled;
    }

    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        if self.dom.disabled(target) {
            return Ok(());
        }

        let tag = self
            .dom
            .tag_name(target)
            .ok_or_else(|| Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input or textarea".into(),
                actual: "non-element".into(),
            })?
            .to_ascii_lowercase();

        if tag != "input" && tag != "textarea" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input or textarea".into(),
                actual: tag,
            });
        }

        self.dom.set_value(target, text)?;
        self.dispatch_event(target, "input")?;
        Ok(())
    }

    pub fn set_checked(&mut self, selector: &str, checked: bool) -> Result<()> {
        let target = self.select_one(selector)?;
        if self.dom.disabled(target) {
            return Ok(());
        }
        let tag = self
            .dom
            .tag_name(target)
            .unwrap_or_default()
            .to_ascii_lowercase();
        if tag != "input" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input[type=checkbox|radio]".into(),
                actual: tag,
            });
        }

        let kind = self
            .dom
            .attr(target, "type")
            .unwrap_or_else(|| "text".into())
            .to_ascii_lowercase();
        if kind != "checkbox" && kind != "radio" {
            return Err(Error::TypeMismatch {
                selector: selector.to_string(),
                expected: "input[type=checkbox|radio]".into(),
                actual: format!("input[type={kind}]"),
            });
        }

        let current = self.dom.checked(target)?;
        if current != checked {
            if kind == "radio" && checked {
                self.uncheck_other_radios_in_group(target)?;
            }
            self.dom.set_checked(target, checked)?;
            self.dispatch_event(target, "input")?;
            self.dispatch_event(target, "change")?;
        }

        Ok(())
    }

    pub fn click(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        if self.dom.disabled(target) {
            return Ok(());
        }

        let click_outcome = self.dispatch_event(target, "click")?;
        if click_outcome.default_prevented {
            return Ok(());
        }

        if is_checkbox_input(&self.dom, target) {
            let current = self.dom.checked(target)?;
            self.dom.set_checked(target, !current)?;
            self.dispatch_event(target, "input")?;
            self.dispatch_event(target, "change")?;
        }

        if is_radio_input(&self.dom, target) {
            let current = self.dom.checked(target)?;
            if !current {
                self.uncheck_other_radios_in_group(target)?;
                self.dom.set_checked(target, true)?;
                self.dispatch_event(target, "input")?;
                self.dispatch_event(target, "change")?;
            }
        }

        if is_submit_control(&self.dom, target) {
            if let Some(form_id) = self.resolve_form_for_submit(target) {
                self.dispatch_event(form_id, "submit")?;
            }
        }

        Ok(())
    }

    pub fn submit(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;

        let form = if self
            .dom
            .tag_name(target)
            .map(|t| t.eq_ignore_ascii_case("form"))
            .unwrap_or(false)
        {
            Some(target)
        } else {
            self.resolve_form_for_submit(target)
        };

        if let Some(form_id) = form {
            self.dispatch_event(form_id, "submit")?;
        }

        Ok(())
    }

    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        self.dispatch_event(target, event)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn assert_text(&self, selector: &str, expected: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.text_content(target);
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual,
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_value(&self, selector: &str, expected: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.value(target)?;
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual,
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_checked(&self, selector: &str, expected: bool) -> Result<()> {
        let target = self.select_one(selector)?;
        let actual = self.dom.checked(target)?;
        if actual != expected {
            return Err(Error::AssertionFailed {
                selector: selector.to_string(),
                expected: expected.to_string(),
                actual: actual.to_string(),
                dom_snippet: self.node_snippet(target),
            });
        }
        Ok(())
    }

    pub fn assert_exists(&self, selector: &str) -> Result<()> {
        let _ = self.select_one(selector)?;
        Ok(())
    }

    pub fn dump_dom(&self, selector: &str) -> Result<String> {
        let target = self.select_one(selector)?;
        Ok(self.dom.dump_node(target))
    }

    fn select_one(&self, selector: &str) -> Result<NodeId> {
        self.dom
            .query_selector(selector)?
            .ok_or_else(|| Error::SelectorNotFound(selector.to_string()))
    }

    fn node_snippet(&self, node_id: NodeId) -> String {
        truncate_chars(&self.dom.dump_node(node_id), 200)
    }

    fn resolve_form_for_submit(&self, target: NodeId) -> Option<NodeId> {
        if self
            .dom
            .tag_name(target)
            .map(|t| t.eq_ignore_ascii_case("form"))
            .unwrap_or(false)
        {
            return Some(target);
        }
        self.dom.find_ancestor_by_tag(target, "form")
    }

    fn form_owner(&self, node_id: NodeId) -> Option<NodeId> {
        if self
            .dom
            .tag_name(node_id)
            .map(|t| t.eq_ignore_ascii_case("form"))
            .unwrap_or(false)
        {
            Some(node_id)
        } else {
            self.dom.find_ancestor_by_tag(node_id, "form")
        }
    }

    fn uncheck_other_radios_in_group(&mut self, target: NodeId) -> Result<()> {
        let target_name = self.dom.attr(target, "name").unwrap_or_default();
        if target_name.is_empty() {
            return Ok(());
        }
        let target_form = self.form_owner(target);

        for node in self.dom.all_element_nodes() {
            if node == target {
                continue;
            }
            if !is_radio_input(&self.dom, node) {
                continue;
            }
            if self.dom.attr(node, "name").unwrap_or_default() != target_name {
                continue;
            }
            if self.form_owner(node) != target_form {
                continue;
            }
            if self.dom.checked(node)? {
                self.dom.set_checked(node, false)?;
            }
        }

        Ok(())
    }

    fn dispatch_event(&mut self, target: NodeId, event_type: &str) -> Result<EventState> {
        let mut event = EventState::new(event_type, target);

        let mut path = Vec::new();
        let mut cursor = Some(target);
        while let Some(node) = cursor {
            path.push(node);
            cursor = self.dom.parent(node);
        }
        path.reverse();

        if path.is_empty() {
            return Ok(event);
        }

        // Capture phase.
        if path.len() >= 2 {
            for node in &path[..path.len() - 1] {
                event.current_target = *node;
                self.invoke_listeners(*node, &mut event, true)?;
                if event.propagation_stopped {
                    return Ok(event);
                }
            }
        }

        // Target phase: capture listeners first.
        event.current_target = target;
        self.invoke_listeners(target, &mut event, true)?;
        if event.propagation_stopped {
            return Ok(event);
        }

        // Target phase: bubble listeners.
        self.invoke_listeners(target, &mut event, false)?;
        if event.propagation_stopped {
            return Ok(event);
        }

        // Bubble phase.
        if path.len() >= 2 {
            for node in path[..path.len() - 1].iter().rev() {
                event.current_target = *node;
                self.invoke_listeners(*node, &mut event, false)?;
                if event.propagation_stopped {
                    return Ok(event);
                }
            }
        }

        Ok(event)
    }

    fn invoke_listeners(
        &mut self,
        node_id: NodeId,
        event: &mut EventState,
        capture: bool,
    ) -> Result<()> {
        let listeners = self.listeners.get(node_id, &event.event_type, capture);
        for listener in listeners {
            if self.trace {
                let phase = if capture { "capture" } else { "bubble" };
                eprintln!(
                    "[event] {} target={} current={} phase={}",
                    event.event_type, node_id.0, event.current_target.0, phase
                );
            }
            self.execute_handler(&listener.handler, event)?;
            if event.immediate_propagation_stopped {
                break;
            }
        }
        Ok(())
    }

    fn execute_handler(&mut self, handler: &ScriptHandler, event: &mut EventState) -> Result<()> {
        let mut env: HashMap<String, Value> = HashMap::new();
        self.execute_stmts(&handler.stmts, &handler.event_param, event, &mut env)
    }

    fn execute_stmts(
        &mut self,
        stmts: &[Stmt],
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        for stmt in stmts {
            match stmt {
                Stmt::VarDecl { name, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    env.insert(name.clone(), value);
                }
                Stmt::DomAssign { target, prop, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match prop {
                        DomProp::TextContent => {
                            self.dom.set_text_content(node, &value.as_string())?
                        }
                        DomProp::Value => self.dom.set_value(node, &value.as_string())?,
                        DomProp::Checked => self.dom.set_checked(node, value.truthy())?,
                        DomProp::ClassName => {
                            self.dom.set_attr(node, "class", &value.as_string())?
                        }
                        DomProp::Id => self.dom.set_attr(node, "id", &value.as_string())?,
                        DomProp::Name => self.dom.set_attr(node, "name", &value.as_string())?,
                        DomProp::Dataset(key) => {
                            self.dom.dataset_set(node, key, &value.as_string())?
                        }
                        DomProp::Style(prop) => {
                            self.dom.style_set(node, prop, &value.as_string())?
                        }
                    }
                }
                Stmt::ClassListCall {
                    target,
                    method,
                    class_name,
                    force,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match method {
                        ClassListMethod::Add => self.dom.class_add(node, class_name)?,
                        ClassListMethod::Remove => self.dom.class_remove(node, class_name)?,
                        ClassListMethod::Toggle => {
                            if let Some(force_expr) = force {
                                let force_value = self
                                    .eval_expr(force_expr, env, event_param, event)?
                                    .truthy();
                                if force_value {
                                    self.dom.class_add(node, class_name)?;
                                } else {
                                    self.dom.class_remove(node, class_name)?;
                                }
                            } else {
                                let _ = self.dom.class_toggle(node, class_name)?;
                            }
                        }
                    }
                }
                Stmt::DomSetAttribute {
                    target,
                    name,
                    value,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let value = self.eval_expr(value, env, event_param, event)?;
                    self.dom.set_attr(node, name, &value.as_string())?;
                }
                Stmt::DomRemoveAttribute { target, name } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    self.dom.remove_attr(node, name)?;
                }
                Stmt::ForEach {
                    selector,
                    item_var,
                    index_var,
                    body,
                } => {
                    let items = self.dom.query_selector_all(selector)?;
                    let prev_item = env.get(item_var).cloned();
                    let prev_index = index_var.as_ref().and_then(|v| env.get(v).cloned());

                    for (idx, node) in items.iter().enumerate() {
                        env.insert(item_var.clone(), Value::Node(*node));
                        if let Some(index_var) = index_var {
                            env.insert(index_var.clone(), Value::Number(idx as i64));
                        }
                        self.execute_stmts(body, event_param, event, env)?;
                    }

                    if let Some(prev) = prev_item {
                        env.insert(item_var.clone(), prev);
                    } else {
                        env.remove(item_var);
                    }
                    if let Some(index_var) = index_var {
                        if let Some(prev) = prev_index {
                            env.insert(index_var.clone(), prev);
                        } else {
                            env.remove(index_var);
                        }
                    }
                }
                Stmt::If {
                    cond,
                    then_stmts,
                    else_stmts,
                } => {
                    let cond = self.eval_expr(cond, env, event_param, event)?;
                    if cond.truthy() {
                        self.execute_stmts(then_stmts, event_param, event, env)?;
                    } else {
                        self.execute_stmts(else_stmts, event_param, event, env)?;
                    }
                }
                Stmt::EventCall { event_var, method } => {
                    if let Some(param) = event_param {
                        if param == event_var {
                            match method {
                                EventMethod::PreventDefault => {
                                    event.default_prevented = true;
                                }
                                EventMethod::StopPropagation => {
                                    event.propagation_stopped = true;
                                }
                                EventMethod::StopImmediatePropagation => {
                                    event.immediate_propagation_stopped = true;
                                    event.propagation_stopped = true;
                                }
                            }
                        }
                    }
                }
                Stmt::ListenerMutation {
                    target,
                    op,
                    event_type,
                    capture,
                    handler,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match op {
                        ListenerRegistrationOp::Add => {
                            self.listeners.add(
                                node,
                                event_type.clone(),
                                Listener {
                                    capture: *capture,
                                    handler: handler.clone(),
                                },
                            );
                        }
                        ListenerRegistrationOp::Remove => {
                            let _ = self.listeners.remove(node, event_type, *capture, handler);
                        }
                    }
                }
                Stmt::DispatchEvent { target, event_type } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let event_name = self
                        .eval_expr(event_type, env, event_param, event)?
                        .as_string();
                    if event_name.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "dispatchEvent requires non-empty event type".into(),
                        ));
                    }
                    let _ = self.dispatch_event(node, &event_name)?;
                }
                Stmt::Expr(expr) => {
                    let _ = self.eval_expr(expr, env, event_param, event)?;
                }
            }
        }

        Ok(())
    }

    fn eval_expr(
        &self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match expr {
            Expr::String(value) => Ok(Value::String(value.clone())),
            Expr::Bool(value) => Ok(Value::Bool(*value)),
            Expr::Number(value) => Ok(Value::Number(*value)),
            Expr::Var(name) => env
                .get(name)
                .cloned()
                .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}"))),
            Expr::DomRef(target) => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Ok(Value::Node(node))
            }
            Expr::Binary { left, op, right } => {
                let left = self.eval_expr(left, env, event_param, event)?;
                let right = self.eval_expr(right, env, event_param, event)?;
                self.eval_binary(op, &left, &right)
            }
            Expr::DomRead { target, prop } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                match prop {
                    DomProp::Value => Ok(Value::String(self.dom.value(node)?)),
                    DomProp::Checked => Ok(Value::Bool(self.dom.checked(node)?)),
                    DomProp::TextContent => Ok(Value::String(self.dom.text_content(node))),
                    DomProp::ClassName => Ok(Value::String(
                        self.dom.attr(node, "class").unwrap_or_default(),
                    )),
                    DomProp::Id => Ok(Value::String(self.dom.attr(node, "id").unwrap_or_default())),
                    DomProp::Name => Ok(Value::String(
                        self.dom.attr(node, "name").unwrap_or_default(),
                    )),
                    DomProp::Dataset(key) => Ok(Value::String(self.dom.dataset_get(node, key)?)),
                    DomProp::Style(prop) => Ok(Value::String(self.dom.style_get(node, prop)?)),
                }
            }
            Expr::ClassListContains { target, class_name } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Ok(Value::Bool(self.dom.class_contains(node, class_name)?))
            }
            Expr::QuerySelectorAllLength { selector } => {
                let len = self.dom.query_selector_all(selector)?.len() as i64;
                Ok(Value::Number(len))
            }
            Expr::DomGetAttribute { target, name } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Ok(Value::String(self.dom.attr(node, name).unwrap_or_default()))
            }
            Expr::DomHasAttribute { target, name } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Ok(Value::Bool(self.dom.has_attr(node, name)?))
            }
            Expr::EventProp { event_var, prop } => {
                let Some(param) = event_param else {
                    return Err(Error::ScriptRuntime(format!(
                        "event variable '{}' is not available in this handler",
                        event_var
                    )));
                };
                if param != event_var {
                    return Err(Error::ScriptRuntime(format!(
                        "unknown event variable: {}",
                        event_var
                    )));
                }

                let value = match prop {
                    EventExprProp::Type => Value::String(event.event_type.clone()),
                    EventExprProp::Target => Value::String(self.event_node_label(event.target)),
                    EventExprProp::CurrentTarget => {
                        Value::String(self.event_node_label(event.current_target))
                    }
                    EventExprProp::TargetId => {
                        Value::String(self.dom.attr(event.target, "id").unwrap_or_default())
                    }
                    EventExprProp::CurrentTargetId => Value::String(
                        self.dom
                            .attr(event.current_target, "id")
                            .unwrap_or_default(),
                    ),
                };
                Ok(value)
            }
            Expr::Not(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                Ok(Value::Bool(!value.truthy()))
            }
            Expr::Add(parts) => {
                let mut out = String::new();
                for part in parts {
                    let value = self.eval_expr(part, env, event_param, event)?;
                    out.push_str(&value.as_string());
                }
                Ok(Value::String(out))
            }
            Expr::Ternary {
                cond,
                on_true,
                on_false,
            } => {
                let cond = self.eval_expr(cond, env, event_param, event)?;
                if cond.truthy() {
                    self.eval_expr(on_true, env, event_param, event)
                } else {
                    self.eval_expr(on_false, env, event_param, event)
                }
            }
        }
    }

    fn eval_binary(&self, op: &BinaryOp, left: &Value, right: &Value) -> Result<Value> {
        let out = match op {
            BinaryOp::Or => Value::Bool(left.truthy() || right.truthy()),
            BinaryOp::And => Value::Bool(left.truthy() && right.truthy()),
            BinaryOp::StrictEq => Value::Bool(self.strict_equal(left, right)),
            BinaryOp::StrictNe => Value::Bool(!self.strict_equal(left, right)),
            BinaryOp::Lt => Value::Bool(self.compare(left, right, |l, r| l < r)),
            BinaryOp::Gt => Value::Bool(self.compare(left, right, |l, r| l > r)),
            BinaryOp::Le => Value::Bool(self.compare(left, right, |l, r| l <= r)),
            BinaryOp::Ge => Value::Bool(self.compare(left, right, |l, r| l >= r)),
        };
        Ok(out)
    }

    fn strict_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Node(l), Value::Node(r)) => l == r,
            _ => false,
        }
    }

    fn compare<F>(&self, left: &Value, right: &Value, op: F) -> bool
    where
        F: Fn(i64, i64) -> bool,
    {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => op(*l, *r),
            _ => {
                let l = left.as_string();
                let r = right.as_string();
                op(l.parse::<i64>().unwrap_or(0), r.parse::<i64>().unwrap_or(0))
            }
        }
    }

    fn resolve_dom_query_static(&self, target: &DomQuery) -> Result<Option<NodeId>> {
        match target {
            DomQuery::ById(id) => Ok(self.dom.by_id(id)),
            DomQuery::BySelector(selector) => self.dom.query_selector(selector),
            DomQuery::BySelectorAllIndex { selector, index } => {
                let all = self.dom.query_selector_all(selector)?;
                Ok(all.get(*index).copied())
            }
            DomQuery::Var(_) => Err(Error::ScriptRuntime(
                "element variable cannot be resolved in static context".into(),
            )),
        }
    }

    fn resolve_dom_query_runtime(
        &self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<Option<NodeId>> {
        match target {
            DomQuery::Var(name) => match env.get(name) {
                Some(Value::Node(node)) => Ok(Some(*node)),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not an element reference",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown element variable: {}",
                    name
                ))),
            },
            _ => self.resolve_dom_query_static(target),
        }
    }

    fn resolve_dom_query_required_runtime(
        &self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<NodeId> {
        self.resolve_dom_query_runtime(target, env)?.ok_or_else(|| {
            Error::ScriptRuntime(format!("{} returned null", target.describe_call()))
        })
    }

    fn event_node_label(&self, node: NodeId) -> String {
        if let Some(id) = self.dom.attr(node, "id") {
            if !id.is_empty() {
                return id;
            }
        }
        self.dom
            .tag_name(node)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("node-{}", node.0))
    }

    fn compile_and_register_script(&mut self, script: &str) -> Result<()> {
        let mut cursor = Cursor::new(script);
        while !cursor.eof() {
            cursor.skip_ws_and_comments();
            if cursor.eof() {
                break;
            }

            let start = cursor.pos();
            let reg = parse_listener_registration(&mut cursor)?;

            let target = self.resolve_dom_query_static(&reg.target)?.ok_or_else(|| {
                Error::ScriptParse(format!(
                    "listener target {} not found in HTML",
                    reg.target.describe_call()
                ))
            })?;

            let handler = ScriptHandler {
                event_param: reg.event_param,
                stmts: parse_block_statements(&reg.body)?,
            };

            match reg.op {
                ListenerRegistrationOp::Add => {
                    self.listeners.add(
                        target,
                        reg.event_type,
                        Listener {
                            capture: reg.capture,
                            handler,
                        },
                    );
                }
                ListenerRegistrationOp::Remove => {
                    let _ = self
                        .listeners
                        .remove(target, &reg.event_type, reg.capture, &handler);
                }
            }

            cursor.skip_ws_and_comments();
            if cursor.pos() == start {
                return Err(Error::ScriptParse("failed to consume script input".into()));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListenerRegistrationOp {
    Add,
    Remove,
}

#[derive(Debug)]
struct ListenerRegistration {
    op: ListenerRegistrationOp,
    target: DomQuery,
    event_type: String,
    event_param: Option<String>,
    body: String,
    capture: bool,
}

fn parse_listener_registration(cursor: &mut Cursor<'_>) -> Result<ListenerRegistration> {
    let target = parse_document_element_call(cursor)?;

    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    let method = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse("expected listener method".into()))?;
    let op = match method.as_str() {
        "addEventListener" => ListenerRegistrationOp::Add,
        "removeEventListener" => ListenerRegistrationOp::Remove,
        _ => {
            return Err(Error::ScriptParse(format!(
                "unsupported listener method: {method}"
            )));
        }
    };
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();

    let event_type = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b',')?;
    cursor.skip_ws();

    let (event_param, body) = parse_callback(cursor)?;

    cursor.skip_ws();
    let capture = if cursor.consume_byte(b',') {
        cursor.skip_ws();
        if cursor.consume_ascii("true") {
            true
        } else if cursor.consume_ascii("false") {
            false
        } else {
            return Err(Error::ScriptParse(
                "addEventListener third argument must be true/false".into(),
            ));
        }
    } else {
        false
    };

    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    cursor.consume_byte(b';');

    Ok(ListenerRegistration {
        op,
        target,
        event_type,
        event_param,
        body,
        capture,
    })
}

fn parse_element_target(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    cursor.skip_ws();
    let start = cursor.pos();
    if let Ok(target) = parse_document_element_call(cursor) {
        return Ok(target);
    }
    cursor.set_pos(start);
    if let Some(name) = cursor.parse_identifier() {
        return Ok(DomQuery::Var(name));
    }
    Err(Error::ScriptParse(format!(
        "expected element target at {}",
        start
    )))
}

fn parse_document_element_call(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    cursor.skip_ws();
    cursor.expect_ascii("document")?;
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    let method = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse("expected document method call".into()))?;
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let arg = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();

    match method.as_str() {
        "getElementById" => Ok(DomQuery::ById(arg)),
        "querySelector" => Ok(DomQuery::BySelector(arg)),
        "querySelectorAll" => {
            if !cursor.consume_byte(b'[') {
                return Err(Error::ScriptParse(
                    "querySelectorAll used as element must have [index]".into(),
                ));
            }
            cursor.skip_ws();
            let index = cursor.parse_usize()?;
            cursor.skip_ws();
            cursor.expect_byte(b']')?;
            Ok(DomQuery::BySelectorAllIndex {
                selector: arg,
                index,
            })
        }
        _ => Err(Error::ScriptParse(format!(
            "unsupported document method: {}",
            method
        ))),
    }
}

fn parse_callback(cursor: &mut Cursor<'_>) -> Result<(Option<String>, String)> {
    cursor.skip_ws();

    let event_param = if cursor.consume_byte(b'(') {
        let params = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        let trimmed = params.trim();
        if trimmed.is_empty() {
            None
        } else {
            if !is_ident(trimmed) {
                return Err(Error::ScriptParse(format!(
                    "unsupported callback parameters: {trimmed}"
                )));
            }
            Some(trimmed.to_string())
        }
    } else {
        let ident = cursor
            .parse_identifier()
            .ok_or_else(|| Error::ScriptParse("expected callback parameter or ()".into()))?;
        Some(ident)
    };

    cursor.skip_ws();
    cursor.expect_ascii("=>")?;
    cursor.skip_ws();

    let body = cursor.read_balanced_block(b'{', b'}')?;
    Ok((event_param, body))
}

fn parse_block_statements(body: &str) -> Result<Vec<Stmt>> {
    let raw_stmts = split_top_level_statements(body);
    let mut stmts = Vec::new();

    for raw in raw_stmts {
        let stmt = raw.trim();
        if stmt.is_empty() {
            continue;
        }

        if let Some(else_branch) = parse_else_fragment(stmt)? {
            if let Some(Stmt::If { else_stmts, .. }) = stmts.last_mut() {
                if else_stmts.is_empty() {
                    *else_stmts = else_branch;
                    continue;
                }
                return Err(Error::ScriptParse(format!(
                    "duplicate else branch in: {stmt}"
                )));
            }
            return Err(Error::ScriptParse(format!(
                "unexpected else without matching if: {stmt}"
            )));
        }

        let parsed = parse_single_statement(stmt)?;
        stmts.push(parsed);
    }

    Ok(stmts)
}

fn parse_single_statement(stmt: &str) -> Result<Stmt> {
    let stmt = stmt.trim();

    if let Some(parsed) = parse_if_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_query_selector_all_foreach_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_var_decl(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_dom_assignment(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_set_attribute_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_remove_attribute_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_class_list_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_listener_mutation_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_dispatch_event_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_event_call_stmt(stmt) {
        return Ok(parsed);
    }

    let expr = parse_expr(stmt)?;
    Ok(Stmt::Expr(expr))
}

fn parse_else_fragment(stmt: &str) -> Result<Option<Vec<Stmt>>> {
    let trimmed = stmt.trim_start();
    let Some(rest) = strip_else_prefix(trimmed) else {
        return Ok(None);
    };
    let branch = parse_if_branch(rest.trim())?;
    Ok(Some(branch))
}

fn strip_else_prefix(src: &str) -> Option<&str> {
    if !src.starts_with("else") {
        return None;
    }
    let bytes = src.as_bytes();
    let after = 4;
    if after < bytes.len() && is_ident_char(bytes[after]) {
        return None;
    }
    Some(&src[after..])
}

fn parse_if_branch(src: &str) -> Result<Vec<Stmt>> {
    let src = src.trim();
    if src.is_empty() {
        return Err(Error::ScriptParse("empty if branch".into()));
    }

    if src.starts_with('{') {
        let mut cursor = Cursor::new(src);
        let body = cursor.read_balanced_block(b'{', b'}')?;
        cursor.skip_ws();
        cursor.consume_byte(b';');
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported trailing tokens in branch: {src}"
            )));
        }
        return parse_block_statements(&body);
    }

    let single = trim_optional_trailing_semicolon(src);
    if single.is_empty() {
        return Err(Error::ScriptParse("empty single statement branch".into()));
    }
    Ok(vec![parse_single_statement(single)?])
}

fn trim_optional_trailing_semicolon(src: &str) -> &str {
    let mut trimmed = src.trim_end();
    if let Some(without) = trimmed.strip_suffix(';') {
        trimmed = without.trim_end();
    }
    trimmed
}

fn find_top_level_else_keyword(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                b'e' if paren == 0 && bracket == 0 && brace == 0 => {
                    if i + 4 <= bytes.len()
                        && &bytes[i..i + 4] == b"else"
                        && (i == 0 || !is_ident_char(bytes[i - 1]))
                        && (i + 4 == bytes.len() || !is_ident_char(bytes[i + 4]))
                    {
                        return Some(i);
                    }
                }
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }
        i += 1;
    }

    None
}

fn is_ident_char(b: u8) -> bool {
    b == b'_' || b == b'$' || b.is_ascii_alphanumeric()
}

fn parse_if_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    if !cursor.consume_ascii("if") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }

    cursor.skip_ws();
    let cond_src = cursor.read_balanced_block(b'(', b')')?;
    let cond = parse_expr(cond_src.trim())?;

    let tail = cursor.src[cursor.i..].trim();
    if tail.is_empty() {
        return Err(Error::ScriptParse(format!(
            "if statement has no branch: {stmt}"
        )));
    }

    let (then_raw, else_raw) = if tail.starts_with('{') {
        let mut branch_cursor = Cursor::new(tail);
        let _ = branch_cursor.read_balanced_block(b'{', b'}')?;
        let split = branch_cursor.pos();
        let then_raw = tail
            .get(..split)
            .ok_or_else(|| Error::ScriptParse("invalid if branch slice".into()))?;
        let rest = tail
            .get(split..)
            .ok_or_else(|| Error::ScriptParse("invalid if remainder slice".into()))?
            .trim();

        if rest.is_empty() {
            (then_raw, None)
        } else if let Some(after_else) = strip_else_prefix(rest) {
            (then_raw, Some(after_else))
        } else {
            return Err(Error::ScriptParse(format!(
                "unsupported tokens after if block: {rest}"
            )));
        }
    } else {
        if let Some(pos) = find_top_level_else_keyword(tail) {
            let then_raw = tail
                .get(..pos)
                .ok_or_else(|| Error::ScriptParse("invalid then branch".into()))?;
            let else_raw = tail
                .get(pos + 4..)
                .ok_or_else(|| Error::ScriptParse("invalid else branch".into()))?;
            (then_raw, Some(else_raw))
        } else {
            (tail, None)
        }
    };

    let then_stmts = parse_if_branch(then_raw)?;
    let else_stmts = if let Some(raw) = else_raw {
        parse_if_branch(raw)?
    } else {
        Vec::new()
    };

    Ok(Some(Stmt::If {
        cond,
        then_stmts,
        else_stmts,
    }))
}

fn split_top_level_statements(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                b';' => {
                    if paren == 0 && bracket == 0 && brace == 0 {
                        if let Some(part) = body.get(start..i) {
                            out.push(part.to_string());
                        }
                        start = i + 1;
                    }
                }
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }
        i += 1;
    }

    if let Some(tail) = body.get(start..) {
        if !tail.trim().is_empty() {
            out.push(tail.to_string());
        }
    }

    out
}

fn parse_var_decl(stmt: &str) -> Result<Option<Stmt>> {
    let mut rest = None;
    for kw in ["const", "let", "var"] {
        if let Some(after) = stmt.strip_prefix(kw) {
            rest = Some(after.trim_start());
            break;
        }
    }

    let Some(rest) = rest else {
        return Ok(None);
    };

    let (name, expr_src) = rest
        .split_once('=')
        .ok_or_else(|| Error::ScriptParse(format!("invalid variable declaration: {stmt}")))?;

    let name = name.trim();
    if !is_ident(name) {
        return Err(Error::ScriptParse(format!(
            "invalid variable name '{name}' in: {stmt}"
        )));
    }

    let expr = parse_expr(expr_src.trim())?;
    Ok(Some(Stmt::VarDecl {
        name: name.to_string(),
        expr,
    }))
}

fn parse_dom_assignment(stmt: &str) -> Result<Option<Stmt>> {
    let Some(eq_pos) = find_top_level_char(stmt, b'=') else {
        return Ok(None);
    };

    let lhs = stmt[..eq_pos].trim();
    let rhs = stmt[eq_pos + 1..].trim();

    let Some((target, prop)) = parse_dom_access(lhs)? else {
        return Ok(None);
    };

    let expr = parse_expr(rhs)?;
    Ok(Some(Stmt::DomAssign { target, prop, expr }))
}

fn parse_query_selector_all_foreach_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    if !cursor.consume_ascii("document") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("querySelectorAll") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let selector = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("forEach") {
        return Ok(None);
    }
    cursor.skip_ws();

    let callback_src = cursor.read_balanced_block(b'(', b')')?;
    let (item_var, index_var, body) = parse_for_each_callback(&callback_src)?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported forEach statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::ForEach {
        selector,
        item_var,
        index_var,
        body,
    }))
}

fn parse_for_each_callback(src: &str) -> Result<(String, Option<String>, Vec<Stmt>)> {
    let mut cursor = Cursor::new(src.trim());
    cursor.skip_ws();

    let (item_var, index_var) = if cursor.consume_byte(b'(') {
        let params_src = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        let params = split_top_level_by_char(params_src.trim(), b',');
        if params.is_empty() || params.len() > 2 {
            return Err(Error::ScriptParse(format!(
                "forEach callback must have one or two parameters: {src}"
            )));
        }

        let item = params[0].trim();
        if !is_ident(item) {
            return Err(Error::ScriptParse(format!(
                "invalid forEach item parameter '{item}'"
            )));
        }

        let index = if params.len() == 2 {
            let idx = params[1].trim();
            if !is_ident(idx) {
                return Err(Error::ScriptParse(format!(
                    "invalid forEach index parameter '{idx}'"
                )));
            }
            Some(idx.to_string())
        } else {
            None
        };

        (item.to_string(), index)
    } else {
        let Some(item) = cursor.parse_identifier() else {
            return Err(Error::ScriptParse(format!(
                "invalid forEach callback parameters: {src}"
            )));
        };
        (item, None)
    };

    cursor.skip_ws();
    cursor.expect_ascii("=>")?;
    cursor.skip_ws();
    let body = cursor.read_balanced_block(b'{', b'}')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported forEach callback tail: {src}"
        )));
    }

    Ok((item_var, index_var, parse_block_statements(&body)?))
}

fn parse_set_attribute_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("setAttribute") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "setAttribute requires 2 arguments: {stmt}"
        )));
    }
    let name = parse_string_literal_exact(args[0].trim())?;
    let value = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported setAttribute statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::DomSetAttribute {
        target,
        name,
        value,
    }))
}

fn parse_remove_attribute_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("removeAttribute") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported removeAttribute statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::DomRemoveAttribute { target, name }))
}

fn parse_class_list_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("classList") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();

    let method = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse(format!("expected classList method in: {stmt}")))?;
    let method = match method.as_str() {
        "add" => ClassListMethod::Add,
        "remove" => ClassListMethod::Remove,
        "toggle" => ClassListMethod::Toggle,
        _ => return Ok(None),
    };

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() || args.len() > 2 {
        return Err(Error::ScriptParse(format!(
            "invalid classList arguments: {stmt}"
        )));
    }
    let class_name = parse_string_literal_exact(args[0].trim())?;
    let force = if args.len() == 2 {
        Some(parse_expr(args[1].trim())?)
    } else {
        None
    };

    if !matches!(method, ClassListMethod::Toggle) && force.is_some() {
        return Err(Error::ScriptParse(
            "classList add/remove do not accept a force argument".into(),
        ));
    }

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();

    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported classList statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::ClassListCall {
        target,
        method,
        class_name,
        force,
    }))
}

fn parse_dispatch_event_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("dispatchEvent") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 {
        return Err(Error::ScriptParse(format!(
            "dispatchEvent requires 1 argument: {stmt}"
        )));
    }
    let event_type = parse_expr(args[0].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported dispatchEvent statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::DispatchEvent { target, event_type }))
}

fn parse_listener_mutation_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let op = match method.as_str() {
        "addEventListener" => ListenerRegistrationOp::Add,
        "removeEventListener" => ListenerRegistrationOp::Remove,
        _ => return Ok(None),
    };
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let event_type = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b',')?;
    cursor.skip_ws();
    let (event_param, body) = parse_callback(&mut cursor)?;

    cursor.skip_ws();
    let capture = if cursor.consume_byte(b',') {
        cursor.skip_ws();
        if cursor.consume_ascii("true") {
            true
        } else if cursor.consume_ascii("false") {
            false
        } else {
            return Err(Error::ScriptParse(
                "add/removeEventListener third argument must be true/false".into(),
            ));
        }
    } else {
        false
    };

    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported listener mutation statement tail: {stmt}"
        )));
    }

    let handler = ScriptHandler {
        event_param,
        stmts: parse_block_statements(&body)?,
    };
    Ok(Some(Stmt::ListenerMutation {
        target,
        op,
        event_type,
        capture,
        handler,
    }))
}

fn parse_event_call_stmt(stmt: &str) -> Option<Stmt> {
    let stmt = stmt.trim();
    let open = stmt.find('(')?;
    let close = stmt.rfind(')')?;
    if close <= open {
        return None;
    }

    let head = stmt[..open].trim();
    let args = stmt[open + 1..close].trim();
    if !args.is_empty() {
        return None;
    }

    let (event_var, method) = head.split_once('.')?;
    if !is_ident(event_var.trim()) {
        return None;
    }

    let method = match method.trim() {
        "preventDefault" => EventMethod::PreventDefault,
        "stopPropagation" => EventMethod::StopPropagation,
        "stopImmediatePropagation" => EventMethod::StopImmediatePropagation,
        _ => return None,
    };

    Some(Stmt::EventCall {
        event_var: event_var.trim().to_string(),
        method,
    })
}

fn parse_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    if src.is_empty() {
        return Err(Error::ScriptParse("empty expression".into()));
    }

    parse_ternary_expr(src)
}

fn parse_ternary_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());

    if let Some(q_pos) = find_top_level_char(src, b'?') {
        let cond_src = src[..q_pos].trim();
        let colon_pos = find_matching_ternary_colon(src, q_pos + 1).ok_or_else(|| {
            Error::ScriptParse(format!("invalid ternary expression (missing ':'): {src}"))
        })?;
        let true_src = src[q_pos + 1..colon_pos].trim();
        let false_src = src[colon_pos + 1..].trim();

        return Ok(Expr::Ternary {
            cond: Box::new(parse_ternary_expr(cond_src)?),
            on_true: Box::new(parse_ternary_expr(true_src)?),
            on_false: Box::new(parse_ternary_expr(false_src)?),
        });
    }

    parse_logical_or_expr(src)
}

fn parse_logical_or_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["||"]);
    if ops.is_empty() {
        return parse_logical_and_expr(src);
    }
    fold_binary(parts, ops, parse_logical_and_expr, |op| match op {
        "||" => BinaryOp::Or,
        _ => unreachable!(),
    })
}

fn parse_logical_and_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["&&"]);
    if ops.is_empty() {
        return parse_equality_expr(src);
    }
    fold_binary(parts, ops, parse_equality_expr, |op| match op {
        "&&" => BinaryOp::And,
        _ => unreachable!(),
    })
}

fn parse_equality_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["!==", "==="]);
    if ops.is_empty() {
        return parse_relational_expr(src);
    }
    fold_binary(parts, ops, parse_relational_expr, |op| match op {
        "===" => BinaryOp::StrictEq,
        "!==" => BinaryOp::StrictNe,
        _ => unreachable!(),
    })
}

fn parse_relational_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["<=", ">=", "<", ">"]);
    if ops.is_empty() {
        return parse_add_expr(src);
    }
    fold_binary(parts, ops, parse_add_expr, |op| match op {
        "<" => BinaryOp::Lt,
        ">" => BinaryOp::Gt,
        "<=" => BinaryOp::Le,
        ">=" => BinaryOp::Ge,
        _ => unreachable!(),
    })
}

fn parse_add_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let plus_parts = split_top_level_by_char(src, b'+');
    if plus_parts.len() >= 2 {
        let mut parts = Vec::new();
        for part in plus_parts {
            parts.push(parse_unary_expr(part.trim())?);
        }
        return Ok(Expr::Add(parts));
    }

    parse_unary_expr(src)
}

fn parse_unary_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    if let Some(rest) = src.strip_prefix('!') {
        let inner = parse_unary_expr(rest.trim())?;
        return Ok(Expr::Not(Box::new(inner)));
    }
    parse_primary(src)
}

fn fold_binary<F, G>(parts: Vec<&str>, ops: Vec<&str>, parse_leaf: F, map_op: G) -> Result<Expr>
where
    F: Fn(&str) -> Result<Expr>,
    G: Fn(&str) -> BinaryOp,
{
    if parts.is_empty() {
        return Err(Error::ScriptParse("invalid binary expression".into()));
    }
    let mut expr = parse_leaf(parts[0].trim())?;
    for (idx, op) in ops.iter().enumerate() {
        let rhs = parse_leaf(parts[idx + 1].trim())?;
        expr = Expr::Binary {
            left: Box::new(expr),
            op: map_op(op),
            right: Box::new(rhs),
        };
    }
    Ok(expr)
}

fn parse_primary(src: &str) -> Result<Expr> {
    let src = src.trim();

    if src == "true" {
        return Ok(Expr::Bool(true));
    }
    if src == "false" {
        return Ok(Expr::Bool(false));
    }
    if src.as_bytes().iter().all(|b| b.is_ascii_digit()) {
        let n: i64 = src
            .parse()
            .map_err(|_| Error::ScriptParse(format!("invalid numeric literal: {src}")))?;
        return Ok(Expr::Number(n));
    }

    if src.starts_with('`') && src.ends_with('`') && src.len() >= 2 {
        return parse_template_literal(src);
    }

    if (src.starts_with('\'') && src.ends_with('\''))
        || (src.starts_with('"') && src.ends_with('"'))
    {
        let value = parse_string_literal_exact(src)?;
        return Ok(Expr::String(value));
    }

    if let Some((target, class_name)) = parse_class_list_contains_expr(src)? {
        return Ok(Expr::ClassListContains { target, class_name });
    }

    if let Some(selector) = parse_query_selector_all_length_expr(src)? {
        return Ok(Expr::QuerySelectorAllLength { selector });
    }

    if let Some((target, name)) = parse_get_attribute_expr(src)? {
        return Ok(Expr::DomGetAttribute { target, name });
    }

    if let Some((target, name)) = parse_has_attribute_expr(src)? {
        return Ok(Expr::DomHasAttribute { target, name });
    }

    if let Some(target) = parse_document_element_expr(src)? {
        return Ok(Expr::DomRef(target));
    }

    if let Some((event_var, prop)) = parse_event_property_expr(src)? {
        return Ok(Expr::EventProp { event_var, prop });
    }

    if let Some((target, prop)) = parse_dom_access(src)? {
        return Ok(Expr::DomRead { target, prop });
    }

    if is_ident(src) {
        return Ok(Expr::Var(src.to_string()));
    }

    Err(Error::ScriptParse(format!("unsupported expression: {src}")))
}

fn parse_document_element_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_document_element_call(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(target))
}

fn parse_template_literal(src: &str) -> Result<Expr> {
    let inner = &src[1..src.len() - 1];
    let bytes = inner.as_bytes();

    let mut parts: Vec<Expr> = Vec::new();
    let mut text_start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }

        if bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            if let Some(text) = inner.get(text_start..i) {
                let text = unescape_string(text);
                if !text.is_empty() {
                    parts.push(Expr::String(text));
                }
            }
            let expr_start = i + 2;
            let expr_end = find_matching_brace(inner, expr_start)?;
            let expr_src = inner
                .get(expr_start..expr_end)
                .ok_or_else(|| Error::ScriptParse("invalid template expression".into()))?;
            let expr = parse_expr(expr_src.trim())?;
            parts.push(expr);

            i = expr_end + 1;
            text_start = i;
            continue;
        }

        i += 1;
    }

    if let Some(text) = inner.get(text_start..) {
        let text = unescape_string(text);
        if !text.is_empty() {
            parts.push(Expr::String(text));
        }
    }

    if parts.is_empty() {
        return Ok(Expr::String(String::new()));
    }

    if parts.len() == 1 {
        return Ok(parts.remove(0));
    }

    Ok(Expr::Add(parts))
}

fn parse_dom_access(src: &str) -> Result<Option<(DomQuery, DomProp)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();

    let head = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse(format!("expected property name in: {src}")))?;

    cursor.skip_ws();
    let nested = if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        Some(
            cursor
                .parse_identifier()
                .ok_or_else(|| Error::ScriptParse(format!("expected nested property in: {src}")))?,
        )
    } else {
        None
    };

    let prop = match (head.as_str(), nested.as_ref()) {
        ("value", None) => DomProp::Value,
        ("checked", None) => DomProp::Checked,
        ("textContent", None) => DomProp::TextContent,
        ("className", None) => DomProp::ClassName,
        ("id", None) => DomProp::Id,
        ("name", None) => DomProp::Name,
        ("dataset", Some(key)) => DomProp::Dataset(key.clone()),
        ("style", Some(name)) => DomProp::Style(name.clone()),
        _ => {
            let prop_label = if let Some(nested) = nested {
                format!("{head}.{nested}")
            } else {
                head
            };
            return Err(Error::ScriptParse(format!(
                "unsupported DOM property '{}' in: {src}",
                prop_label
            )));
        }
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, prop)))
}

fn parse_get_attribute_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("getAttribute") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, name)))
}

fn parse_has_attribute_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("hasAttribute") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, name)))
}

fn parse_event_property_expr(src: &str) -> Result<Option<(String, EventExprProp)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let Some(event_var) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(head) = cursor.parse_identifier() else {
        return Ok(None);
    };

    let mut nested = None;
    cursor.skip_ws();
    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        nested = cursor.parse_identifier();
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let prop = match (head.as_str(), nested.as_deref()) {
        ("type", None) => EventExprProp::Type,
        ("target", None) => EventExprProp::Target,
        ("currentTarget", None) => EventExprProp::CurrentTarget,
        ("target", Some("id")) => EventExprProp::TargetId,
        ("currentTarget", Some("id")) => EventExprProp::CurrentTargetId,
        _ => return Ok(None),
    };

    Ok(Some((event_var, prop)))
}

fn parse_class_list_contains_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("classList") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    if !cursor.consume_ascii("contains") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let class_name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, class_name)))
}

fn parse_query_selector_all_length_expr(src: &str) -> Result<Option<String>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if !cursor.consume_ascii("document") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("querySelectorAll") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let selector = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("length") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(selector))
}

fn parse_string_literal_exact(src: &str) -> Result<String> {
    let bytes = src.as_bytes();
    if bytes.len() < 2 {
        return Err(Error::ScriptParse("invalid string literal".into()));
    }
    let quote = bytes[0];
    if (quote != b'\'' && quote != b'"') || bytes[bytes.len() - 1] != quote {
        return Err(Error::ScriptParse(format!("invalid string literal: {src}")));
    }

    let mut escaped = false;
    let mut i = 1;
    while i + 1 < bytes.len() {
        let b = bytes[i];
        if escaped {
            escaped = false;
        } else if b == b'\\' {
            escaped = true;
        } else if b == quote {
            return Err(Error::ScriptParse(format!("unexpected quote in: {src}")));
        }
        i += 1;
    }

    Ok(unescape_string(&src[1..src.len() - 1]))
}

fn unescape_string(src: &str) -> String {
    let mut out = String::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            match next {
                b'n' => out.push('\n'),
                b'r' => out.push('\r'),
                b't' => out.push('\t'),
                b'\\' => out.push('\\'),
                b'\'' => out.push('\''),
                b'"' => out.push('"'),
                b'`' => out.push('`'),
                b'$' => out.push('$'),
                _ => out.push(next as char),
            }
            i += 2;
        } else {
            out.push(b as char);
            i += 1;
        }
    }
    out
}

fn strip_outer_parens(mut src: &str) -> &str {
    loop {
        let trimmed = src.trim();
        if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
            return trimmed;
        }

        if !is_fully_wrapped_in_parens(trimmed) {
            return trimmed;
        }

        src = &trimmed[1..trimmed.len() - 1];
    }
}

fn is_fully_wrapped_in_parens(src: &str) -> bool {
    let bytes = src.as_bytes();
    let mut depth = 0isize;
    let mut i = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 && i + 1 < bytes.len() {
                        return false;
                    }
                }
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }
        i += 1;
    }

    depth == 0
}

fn find_top_level_char(src: &str, target: u8) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0usize;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                _ => {
                    if b == target && paren == 0 && bracket == 0 && brace == 0 {
                        return Some(i);
                    }
                }
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }

        i += 1;
    }

    None
}

fn find_matching_ternary_colon(src: &str, from: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = from;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;
    let mut nested_ternary = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                b'?' if paren == 0 && bracket == 0 && brace == 0 => {
                    nested_ternary += 1;
                }
                b':' if paren == 0 && bracket == 0 && brace == 0 => {
                    if nested_ternary == 0 {
                        return Some(i);
                    }
                    nested_ternary -= 1;
                }
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }

        i += 1;
    }

    None
}

fn split_top_level_by_char(src: &str, target: u8) -> Vec<&str> {
    let bytes = src.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                _ => {
                    if b == target && paren == 0 && bracket == 0 && brace == 0 {
                        if let Some(part) = src.get(start..i) {
                            parts.push(part);
                        }
                        start = i + 1;
                    }
                }
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }

        i += 1;
    }

    if let Some(last) = src.get(start..) {
        parts.push(last);
    }

    parts
}

fn split_top_level_by_ops<'a>(src: &'a str, ops: &[&'a str]) -> (Vec<&'a str>, Vec<&'a str>) {
    let bytes = src.as_bytes();
    let mut parts = Vec::new();
    let mut found_ops = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => bracket += 1,
                b']' => bracket = bracket.saturating_sub(1),
                b'{' => brace += 1,
                b'}' => brace = brace.saturating_sub(1),
                _ => {
                    if paren == 0 && bracket == 0 && brace == 0 {
                        let mut matched = None;
                        for op in ops {
                            let op_bytes = op.as_bytes();
                            if i + op_bytes.len() <= bytes.len()
                                && &bytes[i..i + op_bytes.len()] == op_bytes
                            {
                                matched = Some(*op);
                                break;
                            }
                        }
                        if let Some(op) = matched {
                            if let Some(part) = src.get(start..i) {
                                parts.push(part);
                                found_ops.push(op);
                                i += op.len();
                                start = i;
                                continue;
                            }
                        }
                    }
                }
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }
        i += 1;
    }

    if let Some(last) = src.get(start..) {
        parts.push(last);
    }

    (parts, found_ops)
}

fn find_matching_brace(src: &str, start: usize) -> Result<usize> {
    let bytes = src.as_bytes();
    let mut i = start;
    let mut depth = 0usize;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StrState {
        None,
        Single,
        Double,
        Backtick,
    }
    let mut state = StrState::None;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            StrState::None => match b {
                b'\'' => state = StrState::Single,
                b'"' => state = StrState::Double,
                b'`' => state = StrState::Backtick,
                b'{' => depth += 1,
                b'}' => {
                    if depth == 0 {
                        return Ok(i);
                    }
                    depth -= 1;
                }
                _ => {}
            },
            StrState::Single => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = StrState::None;
                }
            }
            StrState::Double => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = StrState::None;
                }
            }
            StrState::Backtick => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = StrState::None;
                }
            }
        }
        i += 1;
    }

    Err(Error::ScriptParse("unclosed template expression".into()))
}

fn is_ident(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }

    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn parse_html(html: &str) -> Result<ParseOutput> {
    let mut dom = Dom::new();
    let mut scripts = Vec::new();

    let mut stack = vec![dom.root];
    let bytes = html.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        if starts_with_at(bytes, i, b"<!--") {
            if let Some(end) = find_subslice(bytes, i + 4, b"-->") {
                i = end + 3;
            } else {
                return Err(Error::HtmlParse("unclosed HTML comment".into()));
            }
            continue;
        }

        if bytes[i] == b'<' {
            if starts_with_at(bytes, i, b"</") {
                let (tag, next) = parse_end_tag(html, i)?;
                i = next;

                while stack.len() > 1 {
                    let top = *stack
                        .last()
                        .ok_or_else(|| Error::HtmlParse("invalid stack state".into()))?;
                    let top_tag = dom.tag_name(top).unwrap_or("");
                    stack.pop();
                    if top_tag.eq_ignore_ascii_case(&tag) {
                        break;
                    }
                }
                continue;
            }

            let (tag, attrs, self_closing, next) = parse_start_tag(html, i)?;
            i = next;

            let parent = *stack
                .last()
                .ok_or_else(|| Error::HtmlParse("missing parent element".into()))?;
            let node = dom.create_element(parent, tag.clone(), attrs);

            if tag.eq_ignore_ascii_case("script") {
                let close = find_case_insensitive_end_tag(bytes, i, b"script")
                    .ok_or_else(|| Error::HtmlParse("unclosed <script>".into()))?;
                if let Some(script_body) = html.get(i..close) {
                    if !script_body.is_empty() {
                        dom.create_text(node, script_body.to_string());
                        scripts.push(script_body.to_string());
                    }
                }
                i = close;
                let (_, after_end) = parse_end_tag(html, i)?;
                i = after_end;
                continue;
            }

            if !self_closing && !is_void_tag(&tag) {
                stack.push(node);
            }
            continue;
        }

        let text_start = i;
        while i < bytes.len() && bytes[i] != b'<' {
            i += 1;
        }

        if let Some(text) = html.get(text_start..i) {
            if !text.is_empty() {
                let parent = *stack
                    .last()
                    .ok_or_else(|| Error::HtmlParse("missing parent element".into()))?;
                dom.create_text(parent, text.to_string());
            }
        }
    }

    Ok(ParseOutput { dom, scripts })
}

fn parse_start_tag(
    html: &str,
    at: usize,
) -> Result<(String, HashMap<String, String>, bool, usize)> {
    let bytes = html.as_bytes();
    let mut i = at;
    if bytes.get(i) != Some(&b'<') {
        return Err(Error::HtmlParse("expected '<'".into()));
    }
    i += 1;

    skip_ws(bytes, &mut i);
    let tag_start = i;
    while i < bytes.len() && is_tag_char(bytes[i]) {
        i += 1;
    }

    let tag = html
        .get(tag_start..i)
        .ok_or_else(|| Error::HtmlParse("invalid tag name".into()))?
        .to_ascii_lowercase();

    if tag.is_empty() {
        return Err(Error::HtmlParse("empty tag name".into()));
    }

    let mut attrs = HashMap::new();
    let mut self_closing = false;

    loop {
        skip_ws(bytes, &mut i);
        if i >= bytes.len() {
            return Err(Error::HtmlParse("unclosed start tag".into()));
        }

        if bytes[i] == b'>' {
            i += 1;
            break;
        }

        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'>' {
            self_closing = true;
            i += 2;
            break;
        }

        let name_start = i;
        while i < bytes.len() && is_attr_name_char(bytes[i]) {
            i += 1;
        }

        let name = html
            .get(name_start..i)
            .ok_or_else(|| Error::HtmlParse("invalid attribute name".into()))?
            .to_ascii_lowercase();

        if name.is_empty() {
            return Err(Error::HtmlParse("invalid attribute name".into()));
        }

        skip_ws(bytes, &mut i);

        let value = if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            skip_ws(bytes, &mut i);
            parse_attr_value(html, bytes, &mut i)?
        } else {
            "true".to_string()
        };

        attrs.insert(name, value);
    }

    Ok((tag, attrs, self_closing, i))
}

fn parse_end_tag(html: &str, at: usize) -> Result<(String, usize)> {
    let bytes = html.as_bytes();
    let mut i = at;

    if !(bytes.get(i) == Some(&b'<') && bytes.get(i + 1) == Some(&b'/')) {
        return Err(Error::HtmlParse("expected end tag".into()));
    }
    i += 2;
    skip_ws(bytes, &mut i);

    let tag_start = i;
    while i < bytes.len() && is_tag_char(bytes[i]) {
        i += 1;
    }

    let tag = html
        .get(tag_start..i)
        .ok_or_else(|| Error::HtmlParse("invalid end tag".into()))?
        .to_ascii_lowercase();

    while i < bytes.len() && bytes[i] != b'>' {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::HtmlParse("unclosed end tag".into()));
    }

    Ok((tag, i + 1))
}

fn parse_attr_value(html: &str, bytes: &[u8], i: &mut usize) -> Result<String> {
    if *i >= bytes.len() {
        return Err(Error::HtmlParse("missing attribute value".into()));
    }

    if bytes[*i] == b'\'' || bytes[*i] == b'"' {
        let quote = bytes[*i];
        *i += 1;
        let start = *i;
        while *i < bytes.len() && bytes[*i] != quote {
            *i += 1;
        }
        if *i >= bytes.len() {
            return Err(Error::HtmlParse("unclosed quoted attribute value".into()));
        }
        let value = html
            .get(start..*i)
            .ok_or_else(|| Error::HtmlParse("invalid attribute value".into()))?
            .to_string();
        *i += 1;
        return Ok(value);
    }

    let start = *i;
    while *i < bytes.len()
        && !bytes[*i].is_ascii_whitespace()
        && bytes[*i] != b'>'
        && !(bytes[*i] == b'/' && *i + 1 < bytes.len() && bytes[*i + 1] == b'>')
    {
        *i += 1;
    }

    let value = html
        .get(start..*i)
        .ok_or_else(|| Error::HtmlParse("invalid attribute value".into()))?
        .to_string();
    Ok(value)
}

fn skip_ws(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() && bytes[*i].is_ascii_whitespace() {
        *i += 1;
    }
}

fn is_tag_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn is_attr_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b':'
}

fn is_void_tag(tag: &str) -> bool {
    matches!(
        tag,
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

fn starts_with_at(bytes: &[u8], at: usize, needle: &[u8]) -> bool {
    if at + needle.len() > bytes.len() {
        return false;
    }
    &bytes[at..at + needle.len()] == needle
}

fn find_subslice(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || from > bytes.len() {
        return None;
    }

    let mut i = from;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_case_insensitive_end_tag(bytes: &[u8], from: usize, tag: &[u8]) -> Option<usize> {
    let mut needle = Vec::new();
    needle.extend_from_slice(b"</");
    needle.extend(tag.iter().map(|b| b.to_ascii_lowercase()));

    let mut i = from;
    while i + needle.len() <= bytes.len() {
        if bytes[i] == b'<' && bytes.get(i + 1) == Some(&b'/') {
            let mut matched = true;
            for j in 0..needle.len() {
                let a = bytes[i + j].to_ascii_lowercase();
                let b = needle[j];
                if a != b {
                    matched = false;
                    break;
                }
            }
            if matched {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn is_checkbox_input(dom: &Dom, node_id: NodeId) -> bool {
    let Some(element) = dom.element(node_id) else {
        return false;
    };

    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("checkbox"))
        .unwrap_or(false)
}

fn is_radio_input(dom: &Dom, node_id: NodeId) -> bool {
    let Some(element) = dom.element(node_id) else {
        return false;
    };

    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("radio"))
        .unwrap_or(false)
}

fn is_submit_control(dom: &Dom, node_id: NodeId) -> bool {
    let Some(element) = dom.element(node_id) else {
        return false;
    };

    if element.tag_name.eq_ignore_ascii_case("button") {
        return element
            .attrs
            .get("type")
            .map(|kind| kind.eq_ignore_ascii_case("submit"))
            .unwrap_or(true);
    }

    if element.tag_name.eq_ignore_ascii_case("input") {
        return element
            .attrs
            .get("type")
            .map(|kind| kind.eq_ignore_ascii_case("submit"))
            .unwrap_or(false);
    }

    false
}

#[derive(Debug)]
struct Cursor<'a> {
    src: &'a str,
    i: usize,
}

impl<'a> Cursor<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, i: 0 }
    }

    fn eof(&self) -> bool {
        self.i >= self.src.len()
    }

    fn pos(&self) -> usize {
        self.i
    }

    fn set_pos(&mut self, pos: usize) {
        self.i = pos;
    }

    fn bytes(&self) -> &'a [u8] {
        self.src.as_bytes()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes().get(self.i).copied()
    }

    fn consume_byte(&mut self, b: u8) -> bool {
        if self.peek() == Some(b) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn expect_byte(&mut self, b: u8) -> Result<()> {
        if self.consume_byte(b) {
            Ok(())
        } else {
            Err(Error::ScriptParse(format!(
                "expected '{}' at {}",
                b as char, self.i
            )))
        }
    }

    fn consume_ascii(&mut self, token: &str) -> bool {
        let bytes = self.bytes();
        if self.i + token.len() > bytes.len() {
            return false;
        }
        let got = &bytes[self.i..self.i + token.len()];
        if got == token.as_bytes() {
            self.i += token.len();
            true
        } else {
            false
        }
    }

    fn expect_ascii(&mut self, token: &str) -> Result<()> {
        if self.consume_ascii(token) {
            Ok(())
        } else {
            Err(Error::ScriptParse(format!(
                "expected '{}' at {}",
                token, self.i
            )))
        }
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if b.is_ascii_whitespace() {
                self.i += 1;
            } else {
                break;
            }
        }
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            self.skip_ws();
            if self.consume_ascii("//") {
                while let Some(b) = self.peek() {
                    self.i += 1;
                    if b == b'\n' {
                        break;
                    }
                }
                continue;
            }
            if self.consume_ascii("/*") {
                while !self.eof() {
                    if self.consume_ascii("*/") {
                        break;
                    }
                    self.i += 1;
                }
                continue;
            }
            break;
        }
    }

    fn parse_identifier(&mut self) -> Option<String> {
        let bytes = self.bytes();
        let start = self.i;
        let first = *bytes.get(self.i)?;
        if !(first == b'_' || first == b'$' || first.is_ascii_alphabetic()) {
            return None;
        }
        self.i += 1;
        while let Some(b) = bytes.get(self.i).copied() {
            if b == b'_' || b == b'$' || b.is_ascii_alphanumeric() {
                self.i += 1;
            } else {
                break;
            }
        }
        self.src.get(start..self.i).map(|s| s.to_string())
    }

    fn parse_usize(&mut self) -> Result<usize> {
        let bytes = self.bytes();
        let start = self.i;
        while let Some(b) = bytes.get(self.i).copied() {
            if b.is_ascii_digit() {
                self.i += 1;
            } else {
                break;
            }
        }
        if self.i == start {
            return Err(Error::ScriptParse(format!("expected number at {}", self.i)));
        }
        let raw = self
            .src
            .get(start..self.i)
            .ok_or_else(|| Error::ScriptParse("invalid numeric slice".into()))?;
        raw.parse::<usize>()
            .map_err(|_| Error::ScriptParse(format!("invalid number: {raw}")))
    }

    fn parse_string_literal(&mut self) -> Result<String> {
        let quote = self
            .peek()
            .ok_or_else(|| Error::ScriptParse("expected string literal".into()))?;
        if quote != b'\'' && quote != b'"' {
            return Err(Error::ScriptParse(format!(
                "expected string literal at {}",
                self.i
            )));
        }

        self.i += 1;
        let start = self.i;

        let bytes = self.bytes();
        while self.i < bytes.len() {
            let b = bytes[self.i];
            if b == b'\\' {
                self.i += 2;
                continue;
            }
            if b == quote {
                let raw = self
                    .src
                    .get(start..self.i)
                    .ok_or_else(|| Error::ScriptParse("invalid string literal".into()))?;
                self.i += 1;
                return Ok(unescape_string(raw));
            }
            self.i += 1;
        }

        Err(Error::ScriptParse("unclosed string literal".into()))
    }

    fn read_until_byte(&mut self, b: u8) -> Result<String> {
        let start = self.i;
        while let Some(current) = self.peek() {
            if current == b {
                return self
                    .src
                    .get(start..self.i)
                    .map(|s| s.to_string())
                    .ok_or_else(|| Error::ScriptParse("invalid substring".into()));
            }
            self.i += 1;
        }
        Err(Error::ScriptParse(format!(
            "expected '{}' before EOF",
            b as char
        )))
    }

    fn read_balanced_block(&mut self, open: u8, close: u8) -> Result<String> {
        self.expect_byte(open)?;
        let start = self.i;
        let bytes = self.bytes();

        let mut depth = 1usize;
        let mut idx = self.i;

        #[derive(Clone, Copy, PartialEq, Eq)]
        enum StrState {
            None,
            Single,
            Double,
            Backtick,
        }
        let mut state = StrState::None;

        while idx < bytes.len() {
            let b = bytes[idx];
            match state {
                StrState::None => match b {
                    b'\'' => state = StrState::Single,
                    b'"' => state = StrState::Double,
                    b'`' => state = StrState::Backtick,
                    _ => {
                        if b == open {
                            depth += 1;
                        } else if b == close {
                            depth -= 1;
                            if depth == 0 {
                                let body = self
                                    .src
                                    .get(start..idx)
                                    .ok_or_else(|| Error::ScriptParse("invalid block".into()))?
                                    .to_string();
                                self.i = idx + 1;
                                return Ok(body);
                            }
                        }
                    }
                },
                StrState::Single => {
                    if b == b'\\' {
                        idx += 1;
                    } else if b == b'\'' {
                        state = StrState::None;
                    }
                }
                StrState::Double => {
                    if b == b'\\' {
                        idx += 1;
                    } else if b == b'"' {
                        state = StrState::None;
                    }
                }
                StrState::Backtick => {
                    if b == b'\\' {
                        idx += 1;
                    } else if b == b'`' {
                        state = StrState::None;
                    }
                }
            }
            idx += 1;
        }

        Err(Error::ScriptParse("unclosed block".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_updates_result() -> Result<()> {
        let html = r#"
        <input id='name'>
        <input id='agree' type='checkbox'>
        <button id='submit'>Send</button>
        <p id='result'></p>
        <script>
          document.getElementById('submit').addEventListener('click', () => {
            const name = document.getElementById('name').value;
            const agree = document.getElementById('agree').checked;
            document.getElementById('result').textContent =
              agree ? `OK:${name}` : 'NG';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.type_text("#name", "Taro")?;
        h.set_checked("#agree", true)?;
        h.click("#submit")?;
        h.assert_text("#result", "OK:Taro")?;
        Ok(())
    }

    #[test]
    fn prevent_default_works_on_submit() -> Result<()> {
        let html = r#"
        <form id='f'>
          <button id='submit' type='submit'>Send</button>
        </form>
        <p id='result'></p>
        <script>
          document.getElementById('f').addEventListener('submit', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent = 'blocked';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#submit")?;
        h.assert_text("#result", "blocked")?;
        Ok(())
    }

    #[test]
    fn stop_propagation_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <button id='btn'>X</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            event.stopPropagation();
            document.getElementById('result').textContent = 'btn';
          });
          document.getElementById('root').addEventListener('click', () => {
            document.getElementById('result').textContent = 'root';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "btn")?;
        Ok(())
    }

    #[test]
    fn query_selector_if_else_and_class_list_work() -> Result<()> {
        let html = r#"
        <div id='box' class='base'></div>
        <button id='btn'>toggle</button>
        <p id='result'></p>
        <script>
          document.querySelector('#btn').addEventListener('click', () => {
            if (document.querySelector('#box').classList.contains('active')) {
              document.querySelector('#result').textContent = 'active';
            } else {
              document.querySelector('#box').classList.add('active');
              document.querySelector('#result').textContent = 'activated';
            }
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "activated")?;
        h.click("#btn")?;
        h.assert_text("#result", "active")?;
        Ok(())
    }

    #[test]
    fn class_list_toggle_and_not_condition_work() -> Result<()> {
        let html = r#"
        <div id='badge' class='badge'></div>
        <button id='btn'>toggle</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.querySelector('#badge').classList.toggle('on');
            if (!document.querySelector('#badge').classList.contains('on')) {
              document.getElementById('result').textContent = 'off';
            } else {
              document.getElementById('result').textContent = 'on';
            }
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "on")?;
        h.click("#btn")?;
        h.assert_text("#result", "off")?;
        Ok(())
    }

    #[test]
    fn query_selector_all_index_and_length_work() -> Result<()> {
        let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const second = document.querySelectorAll('.item')[1].textContent;
            document.getElementById('result').textContent =
              second + ':' + document.querySelectorAll('.item').length;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "B:2")?;
        Ok(())
    }

    #[test]
    fn query_selector_all_foreach_and_element_variables_work() -> Result<()> {
        let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.querySelectorAll('.item').forEach((item, idx) => {
              item.setAttribute('data-idx', idx);
              item.classList.toggle('picked', idx === 1);
              document.getElementById('result').textContent =
                document.getElementById('result').textContent + item.textContent + item.getAttribute('data-idx');
            });
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' + document.querySelectorAll('.item')[1].classList.contains('picked');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "A0B1:true")?;
        Ok(())
    }

    #[test]
    fn query_selector_all_foreach_single_arg_callback_works() -> Result<()> {
        let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.querySelectorAll('.item').forEach(item => {
              item.classList.add('seen');
              document.getElementById('result').textContent =
                document.getElementById('result').textContent + item.textContent;
            });
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "AB")?;
        Ok(())
    }

    #[test]
    fn foreach_supports_nested_if_else_and_event_variable() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.querySelectorAll('.item').forEach((item, idx) => {
              if (idx === 1) {
                if (event.target.id === 'btn') {
                  item.classList.add('mid');
                } else {
                  item.classList.add('other');
                }
              } else {
                item.classList.add('edge');
              }
            });
            document.getElementById('result').textContent =
              document.querySelectorAll('.edge').length + ':' +
              document.querySelectorAll('.mid').length + ':' +
              event.currentTarget.id;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "2:1:btn")?;
        Ok(())
    }

    #[test]
    fn if_without_braces_with_else_on_next_statement_works() -> Result<()> {
        let html = r#"
        <input id='agree' type='checkbox'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            if (document.getElementById('agree').checked) document.getElementById('result').textContent = 'yes';
            else document.getElementById('result').textContent = 'no';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "no")?;
        h.set_checked("#agree", true)?;
        h.click("#btn")?;
        h.assert_text("#result", "yes")?;
        Ok(())
    }

    #[test]
    fn class_list_toggle_force_argument_works() -> Result<()> {
        let html = r#"
        <input id='force' type='checkbox'>
        <div id='box' class='base'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').classList.toggle('active', document.getElementById('force').checked);
            if (document.getElementById('box').classList.contains('active'))
              document.getElementById('result').textContent = 'active';
            else
              document.getElementById('result').textContent = 'inactive';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "inactive")?;
        h.set_checked("#force", true)?;
        h.click("#btn")?;
        h.assert_text("#result", "active")?;
        h.set_checked("#force", false)?;
        h.click("#btn")?;
        h.assert_text("#result", "inactive")?;
        Ok(())
    }

    #[test]
    fn logical_and_relational_and_strict_operators_work() -> Result<()> {
        let html = r#"
        <input id='age' value='25'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const age = document.getElementById('age').value;
            const okRange = age >= 20 && age < 30;
            if ((okRange === true && age !== '40') || age === '18')
              document.getElementById('result').textContent = 'pass';
            else
              document.getElementById('result').textContent = 'fail';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "pass")?;
        h.type_text("#age", "40")?;
        h.click("#btn")?;
        h.assert_text("#result", "fail")?;
        h.type_text("#age", "18")?;
        h.click("#btn")?;
        h.assert_text("#result", "pass")?;
        Ok(())
    }

    #[test]
    fn dom_properties_and_attribute_methods_work() -> Result<()> {
        let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').setAttribute('data-x', 'v1');
            document.getElementById('box').className = 'a b';
            document.getElementById('box').id = 'box2';
            document.getElementById('box2').name = 'named';
            const x = document.getElementById('box2').getAttribute('data-x');
            document.getElementById('result').textContent =
              document.getElementById('box2').name + ':' + document.getElementById('box2').className + ':' + x;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_exists("#box2")?;
        h.assert_text("#result", "named:a b:v1")?;
        Ok(())
    }

    #[test]
    fn dataset_property_read_write_works() -> Result<()> {
        let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').dataset.userId = 'u42';
            document.getElementById('box').dataset.planType = 'pro';
            document.getElementById('result').textContent =
              document.getElementById('box').dataset.userId + ':' +
              document.getElementById('box').getAttribute('data-user-id') + ':' +
              document.getElementById('box').dataset.planType;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "u42:u42:pro")?;
        Ok(())
    }

    #[test]
    fn style_property_read_write_works() -> Result<()> {
        let html = r#"
        <div id='box' style='color: blue;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').style.backgroundColor = 'red';
            document.getElementById('box').style.color = '';
            document.getElementById('result').textContent =
              document.getElementById('box').style.backgroundColor + ':' +
              document.getElementById('box').style.color + ':' +
              document.getElementById('box').getAttribute('style');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "red::background-color: red;")?;
        Ok(())
    }

    #[test]
    fn dataset_camel_case_mapping_works() -> Result<()> {
        let html = r#"
        <div id='box' data-user-id='u1' data-plan-type='starter'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.dataset.accountStatus = 'active';
            document.getElementById('result').textContent =
              box.dataset.userId + ':' +
              box.dataset.planType + ':' +
              box.getAttribute('data-account-status');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "u1:starter:active")?;
        Ok(())
    }

    #[test]
    fn style_empty_value_removes_attribute_when_last_property() -> Result<()> {
        let html = r#"
        <div id='box' style='color: blue;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.style.color = '';
            document.getElementById('result').textContent =
              box.getAttribute('style') === '' ? 'none' : 'some';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "none")?;
        Ok(())
    }

    #[test]
    fn style_overwrite_updates_existing_declaration_without_duplicate() -> Result<()> {
        let html = r#"
        <div id='box' style='color: blue; border-color: black;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.style.color = 'red';
            box.style.backgroundColor = 'white';
            document.getElementById('result').textContent = box.getAttribute('style');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text(
            "#result",
            "color: red; border-color: black; background-color: white;",
        )?;
        Ok(())
    }

    #[test]
    fn element_reference_expression_assignment_works() -> Result<()> {
        let html = r#"
        <div id='box'></div>
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            const box = document.getElementById('box');
            const second = document.querySelectorAll('.item')[1];
            box.textContent = second.textContent + ':' + event.target.id;
            box.dataset.state = 'ok';
            document.getElementById('result').textContent =
              box.dataset.state + ':' + box.textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "ok:B:btn")?;
        Ok(())
    }

    #[test]
    fn event_properties_and_stop_immediate_propagation_work() -> Result<()> {
        let html = r#"
        <div id='root'>
          <button id='btn'>run</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.getElementById('result').textContent =
              event.type + ':' + event.target.id + ':' + event.currentTarget.id;
            event.stopImmediatePropagation();
          });
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = 'second';
          });
          document.getElementById('root').addEventListener('click', () => {
            document.getElementById('result').textContent = 'root';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "click:btn:btn")?;
        Ok(())
    }

    #[test]
    fn remove_event_listener_works_for_matching_handler() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = 'A';
          });
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          });
          document.getElementById('btn').removeEventListener('click', () => {
            document.getElementById('result').textContent = 'A';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "B")?;
        Ok(())
    }

    #[test]
    fn dispatch_event_statement_works() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('box').addEventListener('custom', (event) => {
            document.getElementById('result').textContent =
              event.type + ':' + event.target.id + ':' + event.currentTarget.id;
          });
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.dispatchEvent('custom');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "custom:box:box")?;
        Ok(())
    }

    #[test]
    fn dynamic_add_event_listener_inside_handler_works() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.addEventListener('custom', () => {
              document.getElementById('result').textContent = 'ok';
            });
            box.dispatchEvent('custom');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "ok")?;
        Ok(())
    }

    #[test]
    fn dynamic_remove_event_listener_inside_handler_works() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('box').addEventListener('custom', () => {
            document.getElementById('result').textContent = 'A';
          });
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.removeEventListener('custom', () => {
              document.getElementById('result').textContent = 'A';
            });
            box.dispatchEvent('custom');
            if (document.getElementById('result').textContent === '')
              document.getElementById('result').textContent = 'none';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "none")?;
        Ok(())
    }

    #[test]
    fn assertion_failure_contains_dom_snippet() -> Result<()> {
        let html = r#"
        <p id='result'>NG</p>
        "#;
        let h = Harness::from_html(html)?;

        let err = match h.assert_text("#result", "OK") {
            Ok(()) => panic!("assert_text should fail"),
            Err(err) => err,
        };

        match err {
            Error::AssertionFailed {
                selector,
                expected,
                actual,
                dom_snippet,
            } => {
                assert_eq!(selector, "#result");
                assert_eq!(expected, "OK");
                assert_eq!(actual, "NG");
                assert!(dom_snippet.contains("<p"));
                assert!(dom_snippet.contains("NG"));
            }
            other => panic!("unexpected error: {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn remove_and_has_attribute_work() -> Result<()> {
        let html = r#"
        <div id='box' data-x='1' class='a'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            const before = box.hasAttribute('data-x');
            box.removeAttribute('data-x');
            const after = box.hasAttribute('data-x');
            box.removeAttribute('class');
            document.getElementById('result').textContent =
              before + ':' + after + ':' + box.className + ':' + box.getAttribute('data-x');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "true:false::")?;
        Ok(())
    }

    #[test]
    fn remove_id_attribute_updates_id_selector_index() -> Result<()> {
        let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.removeAttribute('id');
            document.getElementById('result').textContent =
              document.querySelectorAll('#box').length;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "0")?;
        Ok(())
    }

    #[test]
    fn selector_child_combinator_and_attr_exists_work() -> Result<()> {
        let html = r#"
        <div id='wrap'>
          <div><span id='nested' data-role='x'></span></div>
          <span id='direct' data-role='x'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.querySelector('#wrap>[data-role]').id;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "direct")?;
        Ok(())
    }

    #[test]
    fn radio_group_exclusive_selection_works() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='r1' type='radio' name='plan'>
          <input id='r2' type='radio' name='plan'>
        </form>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#r1")?;
        h.assert_checked("#r1", true)?;
        h.assert_checked("#r2", false)?;
        h.click("#r2")?;
        h.assert_checked("#r1", false)?;
        h.assert_checked("#r2", true)?;
        Ok(())
    }

    #[test]
    fn disabled_controls_ignore_user_actions() -> Result<()> {
        let html = r#"
        <input id='name' disabled value='init'>
        <input id='agree' type='checkbox' disabled checked>
        <p id='result'></p>
        <script>
          document.getElementById('name').addEventListener('input', () => {
            document.getElementById('result').textContent = 'name-input';
          });
          document.getElementById('agree').addEventListener('change', () => {
            document.getElementById('result').textContent = 'agree-change';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.type_text("#name", "next")?;
        h.assert_value("#name", "init")?;
        h.assert_text("#result", "")?;

        h.click("#agree")?;
        h.assert_checked("#agree", true)?;
        h.assert_text("#result", "")?;

        h.set_checked("#agree", false)?;
        h.assert_checked("#agree", true)?;
        h.assert_text("#result", "")?;
        Ok(())
    }
}
